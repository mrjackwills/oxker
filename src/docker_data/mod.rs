use bollard::{
    container::{
        ListContainersOptions, LogsOptions, MemoryStatsStats, RemoveContainerOptions,
        StartContainerOptions, Stats, StatsOptions,
    },
    service::ContainerSummary,
    Docker,
};
use futures_util::StreamExt;
use parking_lot::Mutex;
use std::{
    collections::HashMap,
    sync::{
        atomic::{AtomicBool, AtomicUsize},
        Arc,
    },
};
use tokio::{
    sync::mpsc::{Receiver, Sender},
    task::JoinHandle,
};
use uuid::Uuid;

use crate::{
    app_data::{AppData, ContainerId, DockerControls, State},
    app_error::AppError,
    parse_args::CliArgs,
    ui::{GuiState, Status},
    ENTRY_POINT,
};
mod message;
pub use message::DockerMessage;

#[derive(Debug, Clone, Eq, Hash, PartialEq)]
enum SpawnId {
    Stats((ContainerId, Binate)),
    Log(ContainerId),
}

/// Cpu & Mem stats take twice as long as the update interval to get a value, so will have two being executed at the same time
/// SpawnId::Stats takes container_id and binate value to enable both cycles of the same container_id to be inserted into the hashmap
/// Binate value is toggled when all handles have been spawned off
/// Also effectively means that the docker_update interval minimum will be 1000ms
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
    args: CliArgs,
    binate: Binate,
    docker: Arc<Docker>,
    gui_state: Arc<Mutex<GuiState>>,
    is_running: Arc<AtomicBool>,
    init: Option<Arc<AtomicUsize>>,
    receiver: Receiver<DockerMessage>,
    spawns: Arc<Mutex<HashMap<SpawnId, JoinHandle<()>>>>,
}

impl DockerData {
    /// Use docker stats to calculate current cpu usage
    #[allow(clippy::cast_precision_loss)]
    // TODO FIX: this can overflow
    fn calculate_usage(stats: &Stats) -> f64 {
        let mut cpu_percentage = 0.0;
        let previous_cpu = stats.precpu_stats.cpu_usage.total_usage;
        let cpu_delta = stats.cpu_stats.cpu_usage.total_usage as f64 - previous_cpu as f64;

        if let (Some(cpu_stats_usage), Some(precpu_stats_usage)) = (
            stats.cpu_stats.system_cpu_usage,
            stats.precpu_stats.system_cpu_usage,
        ) {
            let system_delta = (cpu_stats_usage - precpu_stats_usage) as f64;
            let online_cpus = stats.cpu_stats.online_cpus.unwrap_or_else(|| {
                stats
                    .cpu_stats
                    .cpu_usage
                    .percpu_usage
                    .as_ref()
                    .map_or(0, std::vec::Vec::len) as u64
            }) as f64;
            if system_delta > 0.0 && cpu_delta > 0.0 {
                cpu_percentage = (cpu_delta / system_delta) * online_cpus * 100.0;
            }
        }
        cpu_percentage
    }

    /// Get a single docker stat in order to update mem and cpu usage
    /// don't take &self, so that can tokio::spawn into it's own thread
    /// remove if from spawns hashmap when complete
    async fn update_container_stat(
        app_data: Arc<Mutex<AppData>>,
        docker: Arc<Docker>,
        id: ContainerId,
        init: Option<(Arc<AtomicUsize>, usize)>,
        state: State,
        spawn_id: SpawnId,
        spawns: Arc<Mutex<HashMap<SpawnId, JoinHandle<()>>>>,
    ) {
        if state.is_alive() || init.is_some() {
            let mut stream = docker
                .stats(
                    id.get(),
                    Some(StatsOptions {
                        stream: false,
                        one_shot: false,
                    }),
                )
                .take(1);

            while let Some(Ok(stats)) = stream.next().await {
                // Memory stats are only collected if the container is alive - is this the behaviour we want?
                let mem_stat = if state.is_alive() {
                    let mem_cache = stats.memory_stats.stats.map_or(0, |i| match i {
                        MemoryStatsStats::V1(x) => x.inactive_file,
                        MemoryStatsStats::V2(x) => x.inactive_file,
                    });
                    Some(
                        stats
                            .memory_stats
                            .usage
                            .unwrap_or_default()
                            .saturating_sub(mem_cache),
                    )
                } else {
                    None
                };

                let mem_limit = stats.memory_stats.limit.unwrap_or_default();

                let op_key = stats
                    .networks
                    .as_ref()
                    .and_then(|networks| networks.keys().next().cloned());

                let cpu_stats = if state.is_alive() {
                    Some(Self::calculate_usage(&stats))
                } else {
                    None
                };
                let (rx, tx) = if let Some(key) = op_key {
                    stats
                        .networks
                        .unwrap_or_default()
                        .get(&key)
                        .map_or((0, 0), |f| (f.rx_bytes, f.tx_bytes))
                } else {
                    (0, 0)
                };

                app_data
                    .lock()
                    .update_stats_by_id(&id, cpu_stats, mem_stat, mem_limit, rx, tx);
            }
        }
        spawns.lock().remove(&spawn_id);
        if let Some((target, _)) = init {
            target.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        }
    }

    /// Update all stats, spawn each container into own tokio::spawn thread
    fn update_all_container_stats(&mut self, all_ids: &[(State, ContainerId)]) {
        for (state, id) in all_ids {
            let docker = Arc::clone(&self.docker);
            let app_data = Arc::clone(&self.app_data);
            let spawns = Arc::clone(&self.spawns);
            let spawn_id = SpawnId::Stats((id.clone(), self.binate));

            let init = self.init.as_ref().map(|i| (Arc::clone(i), all_ids.len()));
            self.spawns
                .lock()
                .entry(spawn_id.clone())
                .or_insert_with(|| {
                    tokio::spawn(Self::update_container_stat(
                        app_data,
                        docker,
                        id.clone(),
                        init,
                        *state,
                        spawn_id,
                        spawns,
                    ))
                });
        }
        self.binate = self.binate.toggle();
    }

    /// Get all current containers, handle into ContainerItem in the app_data struct rather than here
    /// Just make sure that items sent are guaranteed to have an id
    /// If in a containerised runtime, will ignore any container that uses the `/app/oxker` as an entry point, unless the `-s` flag is set
    pub async fn update_all_containers(&mut self) -> Vec<(State, ContainerId)> {
        let containers = self
            .docker
            .list_containers(Some(ListContainersOptions::<String> {
                all: true,
                ..Default::default()
            }))
            .await
            .unwrap_or_default();

        let mut output = containers
            .into_iter()
            .filter_map(|f| match f.id {
                Some(_) => {
                    if self.args.in_container
                        && f.command
                            .as_ref()
                            .map_or(false, |c| c.starts_with(ENTRY_POINT))
                        && self.args.show_self
                    {
                        None
                    } else {
                        Some(f)
                    }
                }
                None => None,
            })
            .collect::<Vec<ContainerSummary>>();

        self.app_data.lock().update_containers(&mut output);

        // Just get the containers that are currently running, or being restarted, no point updating info on paused or dead containers
        output
            .into_iter()
            .filter_map(|i| {
                i.id.map(|id| (State::from(i.state), ContainerId::from(id.as_str())))
            })
            .collect::<Vec<_>>()
    }

    /// Update single container logs
    /// remove it from spawns hashmap when complete
    async fn update_log(
        app_data: Arc<Mutex<AppData>>,
        docker: Arc<Docker>,
        id: ContainerId,
        since: u64,
        spawns: Arc<Mutex<HashMap<SpawnId, JoinHandle<()>>>>,
    ) {
        let options = Some(LogsOptions::<String> {
            stdout: true,
            timestamps: true,
            since: i64::try_from(since).unwrap_or_default(),
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
        spawns.lock().remove(&SpawnId::Log(id.clone()));
        app_data.lock().update_log_by_id(output, &id);
    }

    /// Update all logs, spawn each container into own tokio::spawn thread
    fn init_all_logs(&mut self, all_ids: &[(State, ContainerId)]) {
        for (_, id) in all_ids {
            let docker = Arc::clone(&self.docker);
            let app_data = Arc::clone(&self.app_data);
            let spawns = Arc::clone(&self.spawns);
            let key = SpawnId::Log(id.clone());
            self.spawns.lock().insert(
                key,
                tokio::spawn(Self::update_log(app_data, docker, id.clone(), 0, spawns)),
            );
        }
    }

    /// Update all cpu_mem, and selected container log (if a log update join_handle isn't currently being executed)
    async fn update_everything(&mut self) {
        let all_ids = self.update_all_containers().await;
        if let Some(container) = self.app_data.lock().get_selected_container() {
            let last_updated = container.last_updated;
            self.spawns
                .lock()
                .entry(SpawnId::Log(container.id.clone()))
                .or_insert_with(|| {
                    // MAYBE make a struct that can create this data?
                    let app_data = Arc::clone(&self.app_data);
                    let docker = Arc::clone(&self.docker);
                    let id = container.id.clone();
                    let spawns = Arc::clone(&self.spawns);
                    tokio::spawn(Self::update_log(app_data, docker, id, last_updated, spawns))
                });
        };
        self.update_all_container_stats(&all_ids);
        self.app_data.lock().sort_containers();
        self.gui_state.lock().stop_loading_animation(Uuid::nil());
    }

    /// Initialize docker container data, before any messages are received
    async fn initialise_container_data(&mut self) {
        self.gui_state.lock().status_push(Status::Init);
        let loading_uuid = Uuid::new_v4();
        GuiState::start_loading_animation(&self.gui_state, loading_uuid);
        let all_ids = self.update_all_containers().await;

        self.update_all_container_stats(&all_ids);

        self.init_all_logs(&all_ids);

        while let Some(x) = self.init.as_ref() {
            self.app_data.lock().sort_containers();
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
            if x.load(std::sync::atomic::Ordering::SeqCst) == all_ids.len() {
                self.init = None;
            }
        }
        self.gui_state.lock().stop_loading_animation(loading_uuid);
        self.gui_state.lock().status_del(Status::Init);
    }

    /// Set the global error as the docker error, and set gui_state to error
    fn set_error(
        app_data: &Arc<Mutex<AppData>>,
        error: DockerControls,
        gui_state: &Arc<Mutex<GuiState>>,
    ) {
        app_data
            .lock()
            .set_error(AppError::DockerCommand(error), gui_state, Status::Error);
    }

    /// Handle incoming messages, container controls & all container information update
    /// Spawn Docker commands off into own thread
    #[allow(clippy::too_many_lines)]
    async fn message_handler(&mut self) {
        while let Some(message) = self.receiver.recv().await {
            let docker = Arc::clone(&self.docker);
            let gui_state = Arc::clone(&self.gui_state);
            let app_data = Arc::clone(&self.app_data);
            let uuid = Uuid::new_v4();
            // TODO need to refactor these
            match message {
                DockerMessage::Exec(docker_tx) => {
                    docker_tx.send(Arc::clone(&self.docker)).ok();
                }
                DockerMessage::Pause(id) => {
                    tokio::spawn(async move {
                        GuiState::start_loading_animation(&gui_state, uuid);
                        if docker.pause_container(id.get()).await.is_err() {
                            Self::set_error(&app_data, DockerControls::Pause, &gui_state);
                        }
                        gui_state.lock().stop_loading_animation(uuid);
                    });
                    self.update_everything().await;
                }
                DockerMessage::Restart(id) => {
                    tokio::spawn(async move {
                        GuiState::start_loading_animation(&gui_state, uuid);
                        if docker.restart_container(id.get(), None).await.is_err() {
                            Self::set_error(&app_data, DockerControls::Restart, &gui_state);
                        }
                        gui_state.lock().stop_loading_animation(uuid);
                    });
                    self.update_everything().await;
                }
                DockerMessage::Start(id) => {
                    tokio::spawn(async move {
                        GuiState::start_loading_animation(&gui_state, uuid);
                        if docker
                            .start_container(id.get(), None::<StartContainerOptions<String>>)
                            .await
                            .is_err()
                        {
                            Self::set_error(&app_data, DockerControls::Start, &gui_state);
                        }
                        gui_state.lock().stop_loading_animation(uuid);
                    });
                    self.update_everything().await;
                }
                DockerMessage::Stop(id) => {
                    tokio::spawn(async move {
                        GuiState::start_loading_animation(&gui_state, uuid);
                        if docker.stop_container(id.get(), None).await.is_err() {
                            Self::set_error(&app_data, DockerControls::Stop, &gui_state);
                        }
                        gui_state.lock().stop_loading_animation(uuid);
                    });
                    self.update_everything().await;
                }
                DockerMessage::Resume(id) => {
                    tokio::spawn(async move {
                        GuiState::start_loading_animation(&gui_state, uuid);
                        if docker.unpause_container(id.get()).await.is_err() {
                            Self::set_error(&app_data, DockerControls::Resume, &gui_state);
                        }
                        gui_state.lock().stop_loading_animation(uuid);
                    });
                    self.update_everything().await;
                }
                DockerMessage::Delete(id) => {
                    tokio::spawn(async move {
                        GuiState::start_loading_animation(&gui_state, uuid);
                        if docker
                            .remove_container(
                                id.get(),
                                Some(RemoveContainerOptions {
                                    v: false,
                                    force: true,
                                    link: false,
                                }),
                            )
                            .await
                            .is_err()
                        {
                            Self::set_error(&app_data, DockerControls::Stop, &gui_state);
                        }
                        gui_state.lock().stop_loading_animation(uuid);
                    });
                    self.update_everything().await;
                    self.gui_state.lock().set_delete_container(None);
                }
                DockerMessage::ConfirmDelete(id) => {
                    self.gui_state.lock().set_delete_container(Some(id));
                }
                DockerMessage::Update => self.update_everything().await,
                DockerMessage::Quit => {
                    self.spawns
                        .lock()
                        .values()
                        .for_each(tokio::task::JoinHandle::abort);
                    self.is_running
                        .store(false, std::sync::atomic::Ordering::SeqCst);
                }
            }
        }
    }

    /// Send an update message every x ms, where x is the args.docker_interval
    fn scheduler(args: &CliArgs, docker_tx: Sender<DockerMessage>) {
        let update_duration = std::time::Duration::from_millis(u64::from(args.docker_interval));
        let mut now = std::time::Instant::now();
        tokio::spawn(async move {
            loop {
                let to_sleep = update_duration.saturating_sub(now.elapsed());
                tokio::time::sleep(to_sleep).await;
                docker_tx.send(DockerMessage::Update).await.ok();
                now = std::time::Instant::now();
            }
        });
    }

    /// Initialise self, and start the message receiving loop
    pub async fn init(
        app_data: Arc<Mutex<AppData>>,
        docker: Docker,
        docker_rx: Receiver<DockerMessage>,
        docker_tx: Sender<DockerMessage>,
        gui_state: Arc<Mutex<GuiState>>,
        is_running: Arc<AtomicBool>,
    ) {
        let args = app_data.lock().args.clone();
        if app_data.lock().get_error().is_none() {
            let mut inner = Self {
                app_data,
                args: args.clone(),
                binate: Binate::One,
                docker: Arc::new(docker),
                gui_state,
                init: Some(Arc::new(AtomicUsize::new(0))),
                is_running,
                receiver: docker_rx,
                spawns: Arc::new(Mutex::new(HashMap::new())),
            };
            inner.initialise_container_data().await;
            Self::scheduler(&args, docker_tx);
            inner.message_handler().await;
        }
    }
}

// tests, use redis-test container, check logs exists, and selector of logs, and that it increases, and matches end, when you run restart on the docker containers
#[cfg(test)]
mod tests {
    use bollard::container::{
        BlkioStats, CPUStats, CPUUsage, MemoryStats, PidsStats, StorageStats, ThrottlingData,
    };

    use super::*;

    #[allow(clippy::too_many_lines)]
    fn gen_stats(x: u64, y: u64) -> Stats {
        Stats {
            read: String::new(),
            preread: String::new(),
            num_procs: 0,
            pids_stats: PidsStats {
                current: None,
                limit: None,
            },
            network: None,
            networks: None,
            memory_stats: MemoryStats {
                stats: None,
                max_usage: None,
                usage: None,
                failcnt: None,
                limit: None,
                commit: None,
                commit_peak: None,
                commitbytes: None,
                commitpeakbytes: None,
                privateworkingset: None,
            },
            blkio_stats: BlkioStats {
                io_service_bytes_recursive: None,
                io_serviced_recursive: None,
                io_queue_recursive: None,
                io_service_time_recursive: None,
                io_wait_time_recursive: None,
                io_merged_recursive: None,
                io_time_recursive: None,
                sectors_recursive: None,
            },
            cpu_stats: CPUStats {
                cpu_usage: CPUUsage {
                    percpu_usage: Some(vec![
                        291_593_800,
                        182_192_900,
                        195_048_700,
                        23_032_300,
                        132_928_700,
                        235_555_600,
                        120_225_700,
                        175_752_000,
                        213_060_300,
                        95_321_600,
                        226_821_000,
                        0,
                        109_151_300,
                        0,
                        86_240_200,
                        1_884_400,
                        59_077_300,
                        23_224_900,
                        95_386_300,
                        144_987_400,
                    ]),
                    total_usage: 250_000_000,
                    usage_in_usermode: 1_020_000_000,
                    usage_in_kernelmode: 1_030_000_000,
                },
                system_cpu_usage: Some(x),
                online_cpus: Some(1),
                throttling_data: ThrottlingData {
                    periods: 0,
                    throttled_periods: 0,
                    throttled_time: 0,
                },
            },
            precpu_stats: CPUStats {
                cpu_usage: CPUUsage {
                    percpu_usage: Some(vec![
                        291_593_800,
                        182_192_900,
                        195_048_700,
                        23_032_300,
                        132_928_700,
                        235_555_600,
                        120_225_700,
                        175_752_000,
                        213_060_300,
                        95_321_600,
                        226_821_000,
                        0,
                        109_151_300,
                        0,
                        86_240_200,
                        1_884_400,
                        59_077_300,
                        23_224_900,
                        93_831_100,
                        144_987_400,
                    ]),
                    total_usage: 200_000_000,
                    usage_in_usermode: 1_020_000_000,
                    usage_in_kernelmode: 1_020_000_000,
                },
                system_cpu_usage: Some(y),
                online_cpus: Some(1),
                throttling_data: ThrottlingData {
                    periods: 0,
                    throttled_periods: 0,
                    throttled_time: 0,
                },
            },
            storage_stats: StorageStats {
                read_count_normalized: None,
                read_size_bytes: None,
                write_count_normalized: None,
                write_size_bytes: None,
            },
            name: "/container_1".to_owned(),
            id: "1".to_owned(),
        }
    }

    #[test]
    #[allow(clippy::float_cmp)]
    /// Test the stats calculator, had to cheat here to get round input/outputs
    fn test_calculate_usage_no_previous_cpu() {
        let stats = gen_stats(1_000_000_000, 900_000_000);
        let result = DockerData::calculate_usage(&stats);
        assert_eq!(result, 50.0);

        let stats = gen_stats(1_000_000_000, 800_000_000);
        let result = DockerData::calculate_usage(&stats);
        assert_eq!(result, 25.0);

        let stats = gen_stats(1_000_000_000, 750_000_000);
        let result = DockerData::calculate_usage(&stats);
        assert_eq!(result, 20.00);
    }
}
