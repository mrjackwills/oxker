use bollard::{
    Docker,
    models::ContainerStatsResponse,
    models::ContainerSummary,
    query_parameters::{
        InspectContainerOptions, ListContainersOptions, LogsOptions, RemoveContainerOptions,
        RestartContainerOptions, StartContainerOptions, StatsOptions, StopContainerOptions,
    },
};
use futures_util::StreamExt;
use parking_lot::Mutex;
use std::{collections::HashSet, hash::Hash, sync::Arc};
use tokio::sync::mpsc::{Receiver, Sender};
use uuid::Uuid;

use crate::{
    ENTRY_POINT,
    app_data::{AppData, ContainerId, DockerCommand, State},
    app_error::AppError,
    config::Config,
    ui::{GuiState, Status},
};
mod message;
pub use message::DockerMessage;

#[derive(Debug, Clone, Eq, Hash, PartialEq)]
enum SpawnId {
    Stats((ContainerId, Binate)),
    Log(ContainerId),
}

const CONCURRENT_FUTURES: usize = 64;

/// Cpu & Mem stats take twice as long as the update interval to get a value, so will have two being executed at the same time
/// SpawnId::Stats takes container_id and binate value to enable both cycles of the same container_id to be inserted into the hashmap
/// Binate value is toggled when all handles have been spawned off
/// Also effectively means that the minimum docker_update interval will be 1000ms
#[derive(Debug, Clone, Copy, Eq, Hash, PartialEq)]
enum Binate {
    One,
    Two,
}

impl Binate {
    const fn toggle(self) -> Self {
        match self {
            Self::One => Self::Two,
            Self::Two => Self::One,
        }
    }
}

#[derive(Debug, Clone)]
pub struct StatsData {
    pub container_id: ContainerId,
    pub cpu_stats: Option<f64>,
    pub mem_stats: Option<u64>,
    pub mem_limit: u64,
    pub rx: u64,
    pub tx: u64,
}

pub struct DockerData {
    app_data: Arc<Mutex<AppData>>,
    binate: Binate,
    config: Config,
    docker: Arc<Docker>,
    gui_state: Arc<Mutex<GuiState>>,
    receiver: Receiver<DockerMessage>,
    spawns: Arc<Mutex<HashSet<SpawnId>>>,
}

impl DockerData {
    /// Use docker stats to calculate current cpu usage
    #[allow(clippy::cast_precision_loss)]
    fn calculate_usage(stats: &ContainerStatsResponse) -> f64 {
        let mut cpu_percentage = 0.0;

        let total_usage = stats.precpu_stats.as_ref().map_or(0, |i| {
            i.cpu_usage
                .as_ref()
                .map_or(0, |i| i.total_usage.unwrap_or_default())
        });

        let cpu_delta = stats.cpu_stats.as_ref().map_or(0, |i| {
            i.cpu_usage.as_ref().map_or(0, |i| {
                i.total_usage
                    .unwrap_or_default()
                    .saturating_sub(total_usage)
            })
        }) as f64;

        if let (Some(Some(cpu_stats_usage)), Some(Some(precpu_stats_usage))) = (
            stats.cpu_stats.as_ref().map(|i| i.system_cpu_usage),
            stats.precpu_stats.as_ref().map(|i| i.system_cpu_usage),
        ) {
            let system_delta = cpu_stats_usage.saturating_sub(precpu_stats_usage) as f64;
            let online_cpus = f64::from(stats.cpu_stats.as_ref().map_or(0, |i| {
                i.online_cpus.unwrap_or_else(|| {
                    u32::try_from(
                        i.cpu_usage
                            .as_ref()
                            .and_then(|usage| usage.percpu_usage.as_ref())
                            .map_or(0, std::vec::Vec::len),
                    )
                    .unwrap_or_default()
                })
            }));
            if system_delta > 0.0 && cpu_delta > 0.0 {
                cpu_percentage = (cpu_delta / system_delta) * online_cpus * 100.0;
            }
        }
        cpu_percentage
    }

    /// Get a single docker stat in order to update mem and cpu usage
    /// remove if from spawns hashmap when complete
    async fn update_container_stats(
        docker: Arc<Docker>,
        container_state: State,
        container_id: &ContainerId,
    ) -> Option<StatsData> {
        let mut stream = docker
            .stats(
                container_id.get(),
                Some(StatsOptions {
                    stream: false,
                    one_shot: false,
                }),
            )
            .take(1);

        let Some(Ok(stats)) = stream.next().await else {
            return None;
        };
        // Memory stats are only collected if the container is alive - is this the behaviour we want?

        let (mem_stats, cpu_stats) = if container_state.is_alive() {
            let mem_cache = stats.memory_stats.as_ref().map_or(&0, |i| {
                i.stats
                    .as_ref()
                    .map_or(&0, |i| i.get("inactive_file").unwrap_or(&0))
            });
            (
                Some(
                    stats
                        .memory_stats
                        .as_ref()
                        .map_or(0, |i| i.usage.unwrap_or_default())
                        .saturating_sub(*mem_cache),
                ),
                Some(Self::calculate_usage(&stats)),
            )
        } else {
            (None, None)
        };

        // TODO is hardcoded eth0 a good idea here?
        let (rx, tx) = stats.networks.as_ref().map_or((0, 0), |i| {
            i.get("eth0").map_or((0, 0), |x| {
                (
                    x.rx_bytes.unwrap_or_default(),
                    x.tx_bytes.unwrap_or_default(),
                )
            })
        });

        Some(StatsData {
            container_id: container_id.to_owned(),
            cpu_stats,
            mem_stats,
            mem_limit: stats
                .memory_stats
                .unwrap_or_default()
                .limit
                .unwrap_or_default(),
            rx,
            tx,
        })
    }

    fn should_run(spawns: &Arc<Mutex<HashSet<SpawnId>>>, spawn_id: &SpawnId) -> bool {
        if spawns.lock().contains(spawn_id) {
            false
        } else {
            spawns.lock().insert(spawn_id.clone());
            true
        }
    }

    // Actual method that is spawed into a tokio threa to update all the container stats
    fn update_all_container_stats_spawn(
        all_ids: Vec<(State, ContainerId, u64)>,
        app_data: Arc<Mutex<AppData>>,
        binate: Binate,
        docker: Arc<Docker>,
        spawns: Arc<Mutex<HashSet<SpawnId>>>,
        tx: Option<tokio::sync::mpsc::Sender<()>>,
    ) {
        tokio::spawn(async move {
            let data = futures::stream::iter(all_ids)
                .map(|(state, id, _since)| {
                    let spawn_id = SpawnId::Stats((id.clone(), binate));

                    let (docker, spawns) = (Arc::clone(&docker), Arc::clone(&spawns));
                    let should_run = Self::should_run(&spawns, &spawn_id);
                    async move {
                        let response = if should_run {
                            Self::update_container_stats(docker, state, &id).await
                        } else {
                            None
                        };
                        spawns.lock().remove(&spawn_id);
                        response
                    }
                })
                .buffer_unordered(CONCURRENT_FUTURES)
                .filter_map(|item| async move { item })
                .collect::<Vec<_>>()
                .await;
            app_data.lock().update_all_stats(data);
            if let Some(tx) = tx {
                tx.send(()).await.ok();
            }
        });
    }

    /// Spawn a thread to update the stats of all the containers,
    fn update_all_container_stats(&mut self, tx: Option<tokio::sync::mpsc::Sender<()>>) {
        let all_ids = self.app_data.lock().get_all_id_state();
        let binate = self.binate;

        let (app_data, docker, spawns) = (
            Arc::clone(&self.app_data),
            Arc::clone(&self.docker),
            Arc::clone(&self.spawns),
        );

        Self::update_all_container_stats_spawn(all_ids, app_data, binate, docker, spawns, tx);
        // TODO is this doing anything?
        self.binate = self.binate.toggle();
    }

    /// Get all current containers, handle into ContainerItem in the app_data struct rather than here
    /// Just make sure that items sent are guaranteed to have an id
    /// If in a containerised runtime, will ignore any container that uses the `/app/oxker` as an entry point, unless the `-s` flag is set
    async fn get_container_summaries(&self) {
        let containers = self
            .docker
            .list_containers(Some(ListContainersOptions {
                all: true,
                ..Default::default()
            }))
            .await
            .unwrap_or_default();

        let output = containers
            .into_iter()
            .filter_map(|f| match f.id {
                Some(_) => {
                    if self.config.in_container
                        && f.command
                            .as_ref()
                            .is_some_and(|c| c.starts_with(ENTRY_POINT))
                        && !self.config.show_self
                    {
                        None
                    } else {
                        Some(f)
                    }
                }
                None => None,
            })
            .collect::<Vec<ContainerSummary>>();
        self.app_data.lock().update_summaries(output);
    }

    /// Update single container logs
    /// remove it from spawns hashmap when complete
    async fn update_log(
        docker: Arc<Docker>,
        id: ContainerId,
        since: u64,
        stderr: bool,
    ) -> (Vec<String>, ContainerId) {
        let options = Some(LogsOptions {
            stdout: true,
            stderr,
            timestamps: true,
            since: i32::try_from(since).unwrap_or_default(),
            ..Default::default()
        });

        let mut logs = docker.logs(id.get(), options);
        let mut output = vec![];

        while let Some(Ok(value)) = logs.next().await {
            let data = value.to_string();
            if !data.trim().is_empty() {
                output.push(data);
            }
        }
        (output, id)
    }

    /// Update all logs, use a future iter stream
    async fn update_all_logs(
        all_ids: Vec<(State, ContainerId, u64)>,
        app_data: Arc<Mutex<AppData>>,
        docker: Arc<Docker>,
        spawns: Arc<Mutex<HashSet<SpawnId>>>,
        std_err: bool,
        tx: Option<tokio::sync::mpsc::Sender<()>>,
    ) {
        let data = futures::stream::iter(all_ids)
            .map(|(_state, id, since)| {
                let (docker, spawns) = (Arc::clone(&docker), Arc::clone(&spawns));
                let spawn_id = SpawnId::Log(id.clone());
                let should_run = Self::should_run(&spawns, &spawn_id);
                async move {
                    if should_run {
                        let data = Self::update_log(docker, id, since, std_err).await;
                        spawns.lock().remove(&spawn_id);
                        Some(data)
                    } else {
                        None
                    }
                }
            })
            .buffer_unordered(CONCURRENT_FUTURES)
            .filter_map(|item| async move { item })
            .collect::<Vec<_>>()
            .await;
        app_data.lock().update_all_container_logs(data);
        if let Some(tx) = tx {
            tx.send(()).await.ok();
        }
    }

    /// Initialize docker container data, before any messages are received
    async fn initialise_container_data(&mut self) {
        let loading_uuid = Uuid::new_v4();
        GuiState::start_loading_animation(&self.gui_state, loading_uuid);
        self.gui_state.lock().status_push(Status::Init);
        // Want to know when to stop loading, use a rx/tx! and wait for both message!

        self.get_container_summaries().await;
        let all_ids = self.app_data.lock().get_all_id_state();
        let (tx, mut rx) = tokio::sync::mpsc::channel(2);
        self.update_all_container_stats(Some(tx.clone()));
        tokio::spawn(Self::update_all_logs(
            all_ids,
            Arc::clone(&self.app_data),
            Arc::clone(&self.docker),
            Arc::clone(&self.spawns),
            self.config.show_std_err,
            Some(tx),
        ));
        rx.recv_many(&mut vec![], 2).await;
        self.gui_state.lock().status_del(Status::Init);
        self.gui_state.lock().stop_loading_animation(loading_uuid);
    }

    /// Update all cpu_mem, and selected container log (if a log update join_handle isn't currently being executed)
    async fn update_everything(&mut self) {
        self.get_container_summaries().await;
        self.update_selected_log();
    }

    /// Update all cpu_mem, and selected container log (if a log update join_handle isn't currently being executed)
    fn update_selected_log(&mut self) {
        if let Some(selected_container) = self
            .app_data
            .lock()
            .get_selected_container_id_state_last_updated()
        {
            let (app_data, docker, spawns) = (
                Arc::clone(&self.app_data),
                Arc::clone(&self.docker),
                Arc::clone(&self.spawns),
            );
            tokio::spawn(Self::update_all_logs(
                vec![selected_container],
                app_data,
                docker,
                spawns,
                self.config.show_std_err,
                None,
            ));
        }
        self.update_all_container_stats(None);
    }

    /// Set the global error as the docker error, and set gui_state to error
    fn set_error(
        app_data: &Arc<Mutex<AppData>>,
        error: DockerCommand,
        gui_state: &Arc<Mutex<GuiState>>,
    ) {
        app_data
            .lock()
            .set_error(AppError::DockerCommand(error), gui_state, Status::Error);
    }

    /// Execute a docker command, is spawned off into it's own tokio thread
    async fn execute_command_inner(
        app_data: Arc<Mutex<AppData>>,
        control: DockerCommand,
        docker: Arc<Docker>,
        gui_state: Arc<Mutex<GuiState>>,
        id: ContainerId,
    ) {
        let uuid = Uuid::new_v4();
        GuiState::start_loading_animation(&gui_state, uuid);
        if match control {
            DockerCommand::Delete => {
                gui_state.lock().set_delete_container(None);
                docker
                    .remove_container(
                        id.get(),
                        Some(RemoveContainerOptions {
                            v: false,
                            force: true,
                            link: false,
                        }),
                    )
                    .await
            }
            DockerCommand::Pause => docker.pause_container(id.get()).await,
            DockerCommand::Restart => {
                docker
                    .restart_container(id.get(), None::<RestartContainerOptions>)
                    .await
            }
            DockerCommand::Resume => docker.unpause_container(id.get()).await,
            DockerCommand::Start => {
                docker
                    .start_container(id.get(), None::<StartContainerOptions>)
                    .await
            }
            DockerCommand::Stop => {
                docker
                    .stop_container(id.get(), None::<StopContainerOptions>)
                    .await
            }
        }
        .is_err()
        {
            Self::set_error(&app_data, control, &gui_state);
        }
        gui_state.lock().stop_loading_animation(uuid);
    }

    /// Execute docker commands (start, stop etc) on it's own tokio thread
    async fn execute_command(&mut self, control: DockerCommand, id: ContainerId) {
        let (app_data, docker, gui_state) = (
            Arc::clone(&self.app_data),
            Arc::clone(&self.docker),
            Arc::clone(&self.gui_state),
        );
        tokio::spawn(Self::execute_command_inner(
            app_data, control, docker, gui_state, id,
        ));

        self.update_everything().await;
    }

    /// Handle incoming messages, container controls & all container information update
    /// Spawn Docker commands off into own thread
    async fn message_handler(&mut self) {
        while let Some(message) = self.receiver.recv().await {
            match message {
                DockerMessage::ConfirmDelete(id) => {
                    self.gui_state.lock().set_delete_container(Some(id));
                }
                DockerMessage::Control((command, id)) => self.execute_command(command, id).await,
                DockerMessage::Exec(docker_tx) => {
                    docker_tx.send(Arc::clone(&self.docker)).ok();
                }
                DockerMessage::UpdateEverything => self.update_everything().await,
                DockerMessage::UpdateSelectedLog => self.update_everything().await,
                DockerMessage::Inspect(id) => {
                    let t = self
                        .docker
                        .inspect_container(id.get(), Some(InspectContainerOptions { size: true }))
                        .await;
                    if let Ok(t) = t {
                        self.app_data.lock().set_inspect_data(t);
                        self.gui_state.lock().status_push(Status::Inspect);
                    } else {
                        self.app_data.lock().set_error(
                            AppError::DockerInspect,
                            &self.gui_state,
                            Status::Error,
                        );
                    }
                }
            }
        }
    }

    /// The spawned heartbeat function
    async fn heartbeat_inner(
        docker_tx: Sender<DockerMessage>,
        update_duration: std::time::Duration,
    ) {
        let mut now = std::time::Instant::now();

        loop {
            docker_tx.send(DockerMessage::UpdateEverything).await.ok();
            if let Some(to_sleep) = update_duration.checked_sub(now.elapsed()) {
                tokio::time::sleep(to_sleep).await;
            }
            now = std::time::Instant::now();
        }
    }

    /// Send an update message every x ms, where x is the args.docker_interval
    fn heartbeat(config: &Config, docker_tx: Sender<DockerMessage>) {
        let update_duration =
            std::time::Duration::from_millis(u64::from(config.docker_interval_ms));
        tokio::spawn(Self::heartbeat_inner(docker_tx, update_duration));
    }

    /// Initialise self, and start the message receiving loop
    pub async fn start(
        app_data: Arc<Mutex<AppData>>,
        docker: Docker,
        docker_rx: Receiver<DockerMessage>,
        docker_tx: Sender<DockerMessage>,
        gui_state: Arc<Mutex<GuiState>>,
    ) {
        let args = app_data.lock().config.clone();
        if app_data.lock().get_error().is_none() {
            let mut inner = Self {
                app_data,
                config: args,
                binate: Binate::One,
                docker: Arc::new(docker),
                gui_state,
                receiver: docker_rx,
                spawns: Arc::new(Mutex::new(HashSet::new())),
            };
            inner.initialise_container_data().await;
            Self::heartbeat(&inner.config, docker_tx);
            inner.message_handler().await;
        }
    }
}

// tests, use redis-test container, check logs exists, and selector of logs, and that it increases, and matches end, when you run restart on the docker containers
#[cfg(test)]
#[allow(clippy::float_cmp)]
mod tests {

    use bollard::models::{ContainerCpuStats, ContainerCpuUsage};

    use super::*;

    fn gen_stats() -> ContainerStatsResponse {
        ContainerStatsResponse {
            read: None,
            os_type: None,
            preread: None,
            num_procs: Some(1),
            pids_stats: None,
            networks: None,
            memory_stats: None,
            blkio_stats: None,
            cpu_stats: Some(ContainerCpuStats {
                cpu_usage: Some(ContainerCpuUsage {
                    percpu_usage: Some(vec![50]),
                    usage_in_usermode: Some(10),
                    total_usage: Some(100),
                    usage_in_kernelmode: Some(20),
                }),
                system_cpu_usage: Some(400),
                online_cpus: Some(1),
                throttling_data: None,
            }),
            precpu_stats: Some(ContainerCpuStats {
                cpu_usage: Some(ContainerCpuUsage {
                    percpu_usage: Some(vec![50]),
                    usage_in_usermode: Some(10),
                    total_usage: Some(100),
                    usage_in_kernelmode: Some(20),
                }),
                system_cpu_usage: Some(400),
                online_cpus: Some(1),
                throttling_data: None,
            }),
            storage_stats: None,
            name: None,
            id: None,
        }
    }

    #[test]
    fn test_calculate_usage_50() {
        let mut stats = gen_stats();
        stats.precpu_stats = Some(ContainerCpuStats {
            cpu_usage: Some(ContainerCpuUsage {
                percpu_usage: Some(vec![50]),
                usage_in_usermode: Some(10),
                total_usage: Some(100),
                usage_in_kernelmode: Some(20),
            }),
            system_cpu_usage: Some(400),
            online_cpus: Some(1),
            throttling_data: None,
        });
        stats.cpu_stats = Some(ContainerCpuStats {
            cpu_usage: Some(ContainerCpuUsage {
                percpu_usage: Some(vec![150]),
                usage_in_usermode: Some(20),
                total_usage: Some(150),
                usage_in_kernelmode: Some(30),
            }),
            system_cpu_usage: Some(500),
            online_cpus: Some(1),
            throttling_data: None,
        });
        let cpu_percentage = DockerData::calculate_usage(&stats);
        assert_eq!(50.0, cpu_percentage);
    }

    #[test]
    fn test_calculate_usage_25() {
        let mut stats = gen_stats();
        stats.precpu_stats = Some(ContainerCpuStats {
            cpu_usage: Some(ContainerCpuUsage {
                percpu_usage: Some(vec![50]),
                usage_in_usermode: Some(10),
                total_usage: Some(100),
                usage_in_kernelmode: Some(20),
            }),
            system_cpu_usage: Some(400),
            online_cpus: Some(1),
            throttling_data: None,
        });
        stats.cpu_stats = Some(ContainerCpuStats {
            cpu_usage: Some(ContainerCpuUsage {
                percpu_usage: Some(vec![75]),
                usage_in_usermode: Some(20),
                total_usage: Some(125),
                usage_in_kernelmode: Some(30),
            }),
            system_cpu_usage: Some(500),
            online_cpus: Some(1),
            throttling_data: None,
        });
        let cpu_percentage = DockerData::calculate_usage(&stats);
        assert_eq!(25.0, cpu_percentage);
    }

    #[test]
    fn test_calculate_usage_75() {
        let mut stats = gen_stats();
        stats.precpu_stats = Some(ContainerCpuStats {
            cpu_usage: Some(ContainerCpuUsage {
                percpu_usage: Some(vec![50]),
                usage_in_usermode: Some(10),
                total_usage: Some(100),
                usage_in_kernelmode: Some(20),
            }),
            system_cpu_usage: Some(400),
            online_cpus: Some(1),
            throttling_data: None,
        });
        stats.cpu_stats = Some(ContainerCpuStats {
            cpu_usage: Some(ContainerCpuUsage {
                percpu_usage: Some(vec![175]),
                usage_in_usermode: Some(20),
                total_usage: Some(175),
                usage_in_kernelmode: Some(30),
            }),
            system_cpu_usage: Some(500),
            online_cpus: Some(1),
            throttling_data: None,
        });
        let cpu_percentage = DockerData::calculate_usage(&stats);
        assert_eq!(75.0, cpu_percentage);
    }

    #[test]
    fn test_calculate_usage_100() {
        let mut stats = gen_stats();
        stats.precpu_stats = Some(ContainerCpuStats {
            cpu_usage: Some(ContainerCpuUsage {
                percpu_usage: Some(vec![50]),
                usage_in_usermode: Some(10),
                total_usage: Some(100),
                usage_in_kernelmode: Some(20),
            }),
            system_cpu_usage: Some(400),
            online_cpus: Some(1),
            throttling_data: None,
        });
        stats.cpu_stats = Some(ContainerCpuStats {
            cpu_usage: Some(ContainerCpuUsage {
                percpu_usage: Some(vec![200]),
                usage_in_usermode: Some(20),
                total_usage: Some(200),
                usage_in_kernelmode: Some(30),
            }),
            system_cpu_usage: Some(500),
            online_cpus: Some(1),
            throttling_data: None,
        });
        let cpu_percentage = DockerData::calculate_usage(&stats);
        assert_eq!(100.0, cpu_percentage);
    }

    #[test]
    fn test_calculate_usage_175() {
        let mut stats = gen_stats();
        stats.precpu_stats = Some(ContainerCpuStats {
            cpu_usage: Some(ContainerCpuUsage {
                percpu_usage: Some(vec![50]),
                usage_in_usermode: Some(10),
                total_usage: Some(100),
                usage_in_kernelmode: Some(20),
            }),
            system_cpu_usage: Some(400),
            online_cpus: Some(1),
            throttling_data: None,
        });
        stats.cpu_stats = Some(ContainerCpuStats {
            cpu_usage: Some(ContainerCpuUsage {
                percpu_usage: Some(vec![275]),
                usage_in_usermode: Some(20),
                total_usage: Some(275),
                usage_in_kernelmode: Some(30),
            }),
            system_cpu_usage: Some(500),
            online_cpus: Some(1),
            throttling_data: None,
        });
        let cpu_percentage = DockerData::calculate_usage(&stats);
        assert_eq!(175.0, cpu_percentage);
    }
}
