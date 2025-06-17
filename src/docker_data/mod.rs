use bollard::{
    Docker,
    query_parameters::{ListContainersOptions, LogsOptions, RemoveContainerOptions, StatsOptions},
    secret::ContainerStatsResponse,
    service::ContainerSummary,
};
use futures_util::StreamExt;
use parking_lot::Mutex;
use std::{
    collections::HashMap,
    sync::{Arc, atomic::AtomicUsize},
};
use tokio::{
    sync::mpsc::{Receiver, Sender},
    task::JoinHandle,
};
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

impl SpawnId {
    /// Extract the &ContainerId out of self
    const fn get_id(&self) -> &ContainerId {
        match self {
            Self::Log(id) | Self::Stats((id, _)) => id,
        }
    }
}

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

pub struct DockerData {
    app_data: Arc<Mutex<AppData>>,
    binate: Binate,
    config: Config,
    docker: Arc<Docker>,
    gui_state: Arc<Mutex<GuiState>>,
    receiver: Receiver<DockerMessage>,
    spawns: Arc<Mutex<HashMap<SpawnId, JoinHandle<()>>>>,
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
                        stats
                            .cpu_stats
                            .clone()
                            .unwrap_or_default()
                            .cpu_usage
                            .unwrap_or_default()
                            .percpu_usage
                            .as_ref()
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
    /// don't take &self, so that can tokio::spawn into it's own thread
    /// remove if from spawns hashmap when complete
    /// Get a single docker stat in order to update mem and cpu usage
    /// don't take &self, so that can tokio::spawn into it's own thread
    /// remove if from spawns hashmap when complete
    async fn update_container_stat(
        app_data: Arc<Mutex<AppData>>,
        docker: Arc<Docker>,
        state: State,
        spawn_id: SpawnId,
        spawns: Arc<Mutex<HashMap<SpawnId, JoinHandle<()>>>>,
    ) {
        let id = spawn_id.get_id();
        let mut stream = docker
            .stats(
                id.get(),
                Some(StatsOptions {
                    stream: false,
                    one_shot: false,
                }),
            )
            .take(1);

        // some err here
        while let Some(Ok(stats)) = stream.next().await {
            // Memory stats are only collected if the container is alive - is this the behaviour we want?

            let (mem_stat, cpu_stats) = if state.is_alive() {
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

            let (rx, tx) = stats.networks.as_ref().map_or((0, 0), |i| {
                (
                    i.rx_bytes.unwrap_or_default(),
                    i.tx_bytes.unwrap_or_default(),
                )
            });

            app_data.lock().update_stats_by_id(
                id,
                cpu_stats,
                mem_stat,
                stats
                    .memory_stats
                    .unwrap_or_default()
                    .limit
                    .unwrap_or_default(),
                rx,
                tx,
            );
        }
        spawns.lock().remove(&spawn_id);
    }

    /// Update all stats, spawn each container into own tokio::spawn thread
    fn update_all_container_stats(&mut self) {
        let all_ids = self.app_data.lock().get_all_id_state();
        for (state, id) in all_ids {
            let spawn_id = SpawnId::Stats((id, self.binate));

            if let std::collections::hash_map::Entry::Vacant(spawns) =
                self.spawns.lock().entry(spawn_id.clone())
            {
                spawns.insert(tokio::spawn(Self::update_container_stat(
                    Arc::clone(&self.app_data),
                    Arc::clone(&self.docker),
                    state,
                    spawn_id,
                    Arc::clone(&self.spawns),
                )));
            }
        }
        self.binate = self.binate.toggle();
    }

    /// Get all current containers, handle into ContainerItem in the app_data struct rather than here
    /// Just make sure that items sent are guaranteed to have an id
    /// If in a containerised runtime, will ignore any container that uses the `/app/oxker` as an entry point, unless the `-s` flag is set
    async fn update_all_containers(&self) {
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
                        && self.config.show_self
                    {
                        None
                    } else {
                        Some(f)
                    }
                }
                None => None,
            })
            .collect::<Vec<ContainerSummary>>();

        self.app_data.lock().update_containers(output);
    }

    /// Update single container logs
    /// remove it from spawns hashmap when complete
    async fn update_log(
        app_data: Arc<Mutex<AppData>>,
        docker: Arc<Docker>,
        id: ContainerId,
        since: u64,
        spawns: Arc<Mutex<HashMap<SpawnId, JoinHandle<()>>>>,
        stderr: bool,
    ) {
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
        app_data.lock().update_log_by_id(output, &id);
        spawns.lock().remove(&SpawnId::Log(id));
    }

    /// Update all logs, spawn each container into own tokio::spawn thread
    fn init_all_logs(&self, all_ids: Vec<(State, ContainerId)>) -> Arc<AtomicUsize> {
        let init = Arc::new(AtomicUsize::new(0));
        for (_, id) in all_ids {
            let app_data: Arc<parking_lot::lock_api::Mutex<parking_lot::RawMutex, AppData>> =
                Arc::clone(&self.app_data);
            let docker = Arc::clone(&self.docker);
            let spawns = Arc::clone(&self.spawns);
            let std_err = self.config.show_std_err;
            let init = Arc::clone(&init);
            self.spawns.lock().insert(
                SpawnId::Log(id.clone()),
                tokio::spawn(async move {
                    Self::update_log(app_data, docker, id, 0, spawns, std_err).await;
                    init.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                }),
            );
        }
        init
    }

    /// Initialize docker container data, before any messages are received
    async fn initialise_container_data(&mut self) {
        self.gui_state.lock().status_push(Status::Init);
        let loading_uuid = Uuid::new_v4();
        GuiState::start_loading_animation(&self.gui_state, loading_uuid);
        self.update_all_containers().await;
        let all_ids = self.app_data.lock().get_all_id_state();
        let all_ids_len = all_ids.len();
        let init = self.init_all_logs(all_ids);
        self.update_all_container_stats();

        while init.load(std::sync::atomic::Ordering::SeqCst) != all_ids_len {
            self.app_data.lock().sort_containers();
            tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        }
        self.gui_state.lock().stop_loading_animation(loading_uuid);
        self.gui_state.lock().status_del(Status::Init);
    }

    /// Update all cpu_mem, and selected container log (if a log update join_handle isn't currently being executed)
    async fn update_everything(&mut self) {
        self.update_all_containers().await;
        if let Some(container) = self.app_data.lock().get_selected_container() {
            let last_updated = container.last_updated;
            let spawn_id = SpawnId::Log(container.id.clone());
            // Only spawn if not already spawned with a given id/binate pair
            if let std::collections::hash_map::Entry::Vacant(spawns) =
                self.spawns.lock().entry(spawn_id)
            {
                spawns.insert(tokio::spawn(Self::update_log(
                    Arc::clone(&self.app_data),
                    Arc::clone(&self.docker),
                    container.id.clone(),
                    last_updated,
                    Arc::clone(&self.spawns),
                    self.config.show_std_err,
                )));
            }
        }
        self.update_all_container_stats();
        self.app_data.lock().sort_containers();
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

    /// Execute docker commands (start, stop etc) on it's own tokio thread
    async fn execute_command(&mut self, control: DockerCommand, id: ContainerId) {
        let (app_data, docker, gui_state) = (
            Arc::clone(&self.app_data),
            Arc::clone(&self.docker),
            Arc::clone(&self.gui_state),
        );
        tokio::spawn(async move {
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
                        .restart_container(
                            id.get(),
                            None::<bollard::query_parameters::RestartContainerOptions>,
                        )
                        .await
                }
                DockerCommand::Resume => docker.unpause_container(id.get()).await,
                DockerCommand::Start => {
                    docker
                        .start_container(
                            id.get(),
                            None::<bollard::query_parameters::StartContainerOptions>,
                        )
                        .await
                }
                DockerCommand::Stop => {
                    docker
                        .stop_container(
                            id.get(),
                            None::<bollard::query_parameters::StopContainerOptions>,
                        )
                        .await
                }
            }
            .is_err()
            {
                Self::set_error(&app_data, control, &gui_state);
            }
            gui_state.lock().stop_loading_animation(uuid);
        });

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
                DockerMessage::Update => self.update_everything().await,
            }
        }
    }

    /// Send an update message every x ms, where x is the args.docker_interval
    fn heartbeat(config: &Config, docker_tx: Sender<DockerMessage>) {
        let update_duration =
            std::time::Duration::from_millis(u64::from(config.docker_interval_ms));
        let mut now = std::time::Instant::now();
        tokio::spawn(async move {
            loop {
                docker_tx.send(DockerMessage::Update).await.ok();
                if let Some(to_sleep) = update_duration.checked_sub(now.elapsed()) {
                    tokio::time::sleep(to_sleep).await;
                }
                now = std::time::Instant::now();
            }
        });
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
                spawns: Arc::new(Mutex::new(HashMap::new())),
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

    use bollard::secret::{ContainerCpuStats, ContainerCpuUsage};

    use super::*;

    fn gen_stats() -> ContainerStatsResponse {
        ContainerStatsResponse {
            read: None,
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
