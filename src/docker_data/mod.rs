use bollard::{
    container::{ListContainersOptions, LogsOptions, Stats, StatsOptions},
    Docker,
};
use futures_util::{future::join_all, StreamExt};
use parking_lot::Mutex;
use std::{
    sync::Arc,
    time::{Duration, Instant},
};

use crate::{app_data::AppData, parse_args::CliArgs, ui::GuiState};

pub struct DockerData {
    app_data: Arc<Mutex<AppData>>,
    docker: Arc<Docker>,
    gui_state: Arc<Mutex<GuiState>>,
    initialised: bool,
    sleep_duration: Duration,
    timestamps: bool,
}

impl DockerData {
    /// Use docker stats for work out current cpu usage
    fn calculate_usage(stats: &Stats) -> f64 {
        let mut cpu_percentage = 0.0;
        let previous_cpu = stats.precpu_stats.cpu_usage.total_usage;
        let cpu_delta = stats.cpu_stats.cpu_usage.total_usage as f64 - previous_cpu as f64;
        if stats.cpu_stats.system_cpu_usage.is_some()
            && stats.precpu_stats.system_cpu_usage.is_some()
        {
            let system_delta = (stats.cpu_stats.system_cpu_usage.unwrap()
                - stats.precpu_stats.system_cpu_usage.unwrap())
                as f64;
            let online_cpus = stats.cpu_stats.online_cpus.unwrap_or_else(|| {
                stats
                    .cpu_stats
                    .cpu_usage
                    .percpu_usage
                    .clone()
                    .unwrap_or_default()
                    .len() as u64
            }) as f64;
            if system_delta > 0.0 && cpu_delta > 0.0 {
                cpu_percentage = (cpu_delta / system_delta) * online_cpus * 100.0;
            }
        }
        cpu_percentage
    }

    /// Get a single docker stat in order to update mem and cpu usage
    /// don't take &self, so that can tokio::spawn into it's on thread
    async fn update_container_stat(
        docker: Arc<Docker>,
        id: String,
        app_data: Arc<Mutex<AppData>>,
        is_running: bool,
    ) {
        let mut stream = docker
            .stats(
                &id,
                Some(StatsOptions {
                    stream: false,
                    one_shot: !is_running,
                }),
            )
            .take(1);

        while let Some(Ok(stats)) = stream.next().await {
            let mem_stat = stats.memory_stats.usage.unwrap_or(0);
            let mem_limit = stats.memory_stats.limit.unwrap_or(0);

            let key = if let Some(networks) = &stats.networks {
                networks.keys().next().map(|x| x.to_owned())
            } else {
                None
            };

            let cpu_stats = Self::calculate_usage(&stats);

            let (rx, tx) = if let Some(k) = key {
                let ii = stats.networks.unwrap();
                let v = ii.get(&k).unwrap();
                (v.rx_bytes.to_owned(), v.tx_bytes.to_owned())
            } else {
                (0, 0)
            };

            if is_running {
                app_data.lock().update_stats(
                    id.clone(),
                    Some(cpu_stats),
                    Some(mem_stat),
                    mem_limit,
                    rx,
                    tx,
                );
            } else {
                app_data
                    .lock()
                    .update_stats(id.clone(), None, None, mem_limit, rx, tx);
            }
        }
    }

    /// Update all stats, spawn each container into own tokio::spawn thread
    async fn update_all_container_stats(&mut self, all_ids: &[(bool, String)]) {
        for (is_running, id) in all_ids.iter() {
            let docker = Arc::clone(&self.docker);
            let app_data = Arc::clone(&self.app_data);
            let is_running = *is_running;
            let id = id.to_owned();
            tokio::spawn(async move {
                Self::update_container_stat(docker, id, app_data, is_running).await
            });
        }
    }

    /// Get all current containers, handle into ContainerItem in the app_data struct rather than here
    /// Just make sure that items sent are guaranteed to have an id
    pub async fn update_all_containers(&mut self) -> Vec<(bool, String)> {
        let containers = self
            .docker
            .list_containers(Some(ListContainersOptions::<String> {
                all: true,
                ..Default::default()
            }))
            .await
            .unwrap();

        let mut output = vec![];
        // iter over containers, to only send ones which have an id, as use ID for extensivley!
        // alternative is to create my own container struct, and will out with details
        containers.iter().filter(|i| i.id.is_some()).for_each(|c| {
            output.push(c.to_owned());
        });

        self.app_data.lock().update_containers(&output);
        output
            .iter()
            .map(|i| {
                (
                    i.state.as_ref().unwrap() == "running",
                    i.id.as_ref().unwrap().to_owned(),
                )
            })
            .collect::<Vec<_>>()
    }

    /// Update single container logs
    /// don't take &self, so that can tokio::spawn into it's on thread
    async fn update_log(
        docker: Arc<Docker>,
        id: String,
        timestamps: bool,
        since: i64,
    ) -> Vec<String> {
        let options = Some(LogsOptions::<String> {
            stdout: true,
            timestamps,
            since,
            ..Default::default()
        });

        let mut logs = docker.logs(&id, options);

        let mut output = vec![];

        while let Some(value) = logs.next().await {
            if let Ok(data) = value {
                let log_string = data.to_string();
                if !log_string.trim().is_empty() {
                    output.push(log_string);
                }
            }
        }
        output
    }

    /// Update all logs, spawn each container into own tokio::spawn thread
    // rename init all logs, as only gets run once
    async fn update_all_logs(&mut self, all_ids: &[(bool, String)]) {
        let mut handles = vec![];

        for (_, id) in all_ids.iter() {
            let docker = Arc::clone(&self.docker);
            let timestamps = self.timestamps;
            let id = id.to_owned();
            handles.push(Self::update_log(docker, id, timestamps, 0));
        }
        let all_logs = join_all(handles).await;
        self.app_data.lock().update_all_logs(all_logs);
    }

    async fn update_everything(&mut self) {
        let all_ids = self.update_all_containers().await;
        let op_index = self.app_data.lock().get_selected_log_index();
        if let Some(index) = op_index {
            let docker = Arc::clone(&self.docker);
            let since = self.app_data.lock().containers.items[index].last_updated as i64;
            let timestamps = self.timestamps;
            let id = self.app_data.lock().containers.items[index].id.to_owned();
            let logs = Self::update_log(docker, id, timestamps, since).await;
            self.app_data.lock().update_log_by_index(logs, index);
        };

        self.update_all_container_stats(&all_ids).await;
    }

    /// Initialise self, and start the updated loop
    pub async fn init(
        args: CliArgs,
        app_data: Arc<Mutex<AppData>>,
        docker: Arc<Docker>,
        gui_state: Arc<Mutex<GuiState>>,
    ) {
        if app_data.lock().get_error().is_none() {
            let mut inner = Self {
                app_data,
                docker,
                gui_state,
                initialised: false,
                sleep_duration: Duration::from_millis(args.docker as u64),
                timestamps: args.timestamp,
            };
            inner.initialise_container_data().await;
            inner.update_loop().await;
        }
    }

    async fn initialise_container_data(&mut self) {
        let gui_state = Arc::clone(&self.gui_state);
        // could also just loop while init is false, would need to move an arc mutex into here
        // so instead just abort at end of function
        let loading_spin = tokio::spawn(async move {
            loop {
                tokio::time::sleep(std::time::Duration::from_millis(100)).await;
                gui_state.lock().next_loading();
            }
        });

        let all_ids = self.update_all_containers().await;
        self.update_all_container_stats(&all_ids).await;

        // Maybe only do a single one at first?
        self.update_all_logs(&all_ids).await;

        if all_ids.is_empty() {
            self.initialised = true;
        }

        // wait until all logs have initialised
        while !self.initialised {
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
            self.initialised = self.app_data.lock().initialised(&all_ids);
        }
        self.app_data.lock().init = true;
        loading_spin.abort();
        self.gui_state.lock().reset_loading();
    }

    /// Update all items, wait until all complete
    /// sleep for CliArgs.docker ms before updating next
    async fn update_loop(&mut self) {
        loop {
            let start = Instant::now();
            self.update_everything().await;

            let elapsed = start.elapsed();
            if elapsed < self.sleep_duration {
                tokio::time::sleep(self.sleep_duration - elapsed).await;
            }
        }
    }
}

// tests, use redis-test container, check logs exists, and selector of logs, and that it increases, and matches end, when you run restart on the docker containers
