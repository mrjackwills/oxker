use app_data::AppData;
use app_error::AppError;
use bollard::{Docker, API_DEFAULT_VERSION};
use docker_data::DockerData;
use input_handler::InputMessages;
use parking_lot::Mutex;
use parse_args::CliArgs;
use std::{
    process,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};
use tokio::sync::mpsc::{Receiver, Sender};
use tracing::{error, info, Level};

mod app_data;
mod app_error;
mod docker_data;
mod exec;
mod input_handler;
mod parse_args;
mod ui;

use ui::{GuiState, Status, Ui};

use crate::docker_data::DockerMessage;

/// This is the entry point when running as a Docker Container, and is used, in conjunction with the `CONTAINER_ENV` ENV, to check if we are running as a Docker Container
const ENTRY_POINT: &str = "/app/oxker";
const ENV_KEY: &str = "OXKER_RUNTIME";
const ENV_VALUE: &str = "container";
const DOCKER_HOST: &str = "DOCKER_HOST";

/// Enable tracing, only really used in debug mode, for now
/// write to file if `-g` is set?
fn setup_tracing() {
    tracing_subscriber::fmt().with_max_level(Level::INFO).init();
}

/// Read the optional docker_host path, the cli args take priority over the DOCKER_HOST env
fn read_docker_host(args: &CliArgs) -> Option<String> {
    args.host
        .as_ref()
        .map_or_else(|| std::env::var(DOCKER_HOST).ok(), |x| Some(x.to_string()))
}

/// Create docker daemon handler, and only spawn up the docker data handler if a ping returns non-error
async fn docker_init(
    app_data: &Arc<Mutex<AppData>>,
    docker_rx: Receiver<DockerMessage>,
    docker_tx: Sender<DockerMessage>,
    gui_state: &Arc<Mutex<GuiState>>,
) {
    let host = read_docker_host(&app_data.lock().args);

    let connection = host.map_or_else(Docker::connect_with_socket_defaults, |host| {
        Docker::connect_with_socket(&host, 120, API_DEFAULT_VERSION)
    });

    if let Ok(docker) = connection {
        if docker.ping().await.is_ok() {
            tokio::spawn(DockerData::start(
                Arc::clone(app_data),
                docker,
                docker_rx,
                docker_tx,
                Arc::clone(gui_state),
            ));
            return;
        }
    }
    app_data
        .lock()
        .set_error(AppError::DockerConnect, gui_state, Status::DockerConnect);
}

/// Create data for, and then spawn a tokio thread, for the input handler
fn handler_init(
    app_data: &Arc<Mutex<AppData>>,
    docker_sx: &Sender<DockerMessage>,
    gui_state: &Arc<Mutex<GuiState>>,
    input_rx: Receiver<InputMessages>,
    is_running: &Arc<AtomicBool>,
) {
    tokio::spawn(input_handler::InputHandler::start(
        Arc::clone(app_data),
        input_rx,
        docker_sx.clone(),
        Arc::clone(gui_state),
        Arc::clone(is_running),
    ));
}

#[tokio::main]
async fn main() {
    setup_tracing();

    let args = CliArgs::new();

    let app_data = Arc::new(Mutex::new(AppData::default(args.clone())));
    let gui_state = Arc::new(Mutex::new(GuiState::default()));
    let is_running = Arc::new(AtomicBool::new(true));
    let (docker_tx, docker_rx) = tokio::sync::mpsc::channel(32);

    docker_init(&app_data, docker_rx, docker_tx.clone(), &gui_state).await;

    if args.gui {
        let (input_tx, input_rx) = tokio::sync::mpsc::channel(32);
        handler_init(&app_data, &docker_tx, &gui_state, input_rx, &is_running);
        Ui::start(app_data, gui_state, input_tx, is_running).await;
    } else {
        info!("in debug mode\n");
        let mut now = std::time::Instant::now();
        // Debug mode for testing, less pointless now, will display some basic information
        while is_running.load(Ordering::SeqCst) {
            let err = app_data.lock().get_error();
            if let Some(err) = err {
                error!("{}", err);
                process::exit(1);
            }
            if let Some(Ok(to_sleep)) = u128::from(args.docker_interval)
                .checked_sub(now.elapsed().as_millis())
                .map(u64::try_from)
            {
                tokio::time::sleep(std::time::Duration::from_millis(to_sleep)).await;
            }
            let containers = app_data
                .lock()
                .get_container_items()
                .iter()
                .map(|i| format!("{i}"))
                .collect::<Vec<_>>();

            if !containers.is_empty() {
                for item in containers {
                    info!("{item}");
                }
                println!();
            }
            now = std::time::Instant::now();
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {

    use bollard::service::{ContainerSummary, Port};

    use crate::{
        app_data::{
            AppData, ContainerId, ContainerItem, ContainerPorts, ContainerStatus, Filter,
            RunningState, State, StatefulList,
        },
        parse_args::CliArgs,
    };

    pub const fn gen_args() -> CliArgs {
        CliArgs {
            color: false,
            docker_interval: 1000,
            gui: true,
            host: None,
            std_err: false,
            in_container: false,
            save_dir: None,
            raw: false,
            show_self: false,
            timestamp: false,
            use_cli: false,
        }
    }

    pub fn gen_item(id: &ContainerId, index: usize) -> ContainerItem {
        ContainerItem::new(
            u64::try_from(index).unwrap(),
            id.clone(),
            format!("image_{index}"),
            false,
            format!("container_{index}"),
            vec![ContainerPorts {
                ip: None,
                private: u16::try_from(index).unwrap_or(1) + 8000,
                public: None,
            }],
            State::Running(RunningState::Healthy),
            ContainerStatus::from(format!("Up {index} hour")),
        )
    }

    pub fn gen_appdata(containers: &[ContainerItem]) -> AppData {
        AppData {
            containers: StatefulList::new(containers.to_vec()),
            hidden_containers: vec![],
            error: None,
            sorted_by: None,
            filter: Filter::new(),
            args: gen_args(),
        }
    }

    pub fn gen_containers() -> (Vec<ContainerId>, Vec<ContainerItem>) {
        let ids = (1..=3)
            .map(|i| ContainerId::from(format!("{i}").as_str()))
            .collect::<Vec<_>>();
        let containers = ids
            .iter()
            .enumerate()
            .map(|(index, id)| gen_item(id, index + 1))
            .collect::<Vec<_>>();
        (ids, containers)
    }

    pub fn gen_container_summary(index: usize, state: &str) -> ContainerSummary {
        ContainerSummary {
            id: Some(format!("{index}")),
            names: Some(vec![format!("container_{}", index)]),
            image: Some(format!("image_{index}")),
            image_id: Some(format!("{index}")),
            command: None,
            created: Some(i64::try_from(index).unwrap()),
            ports: Some(vec![Port {
                ip: None,
                private_port: u16::try_from(index).unwrap_or(1) + 8000,
                public_port: None,
                typ: None,
            }]),
            size_rw: None,
            size_root_fs: None,
            labels: None,
            state: Some(state.to_owned()),
            status: Some(format!("Up {index} hour")),
            host_config: None,
            network_settings: None,
            mounts: None,
        }
    }
}
