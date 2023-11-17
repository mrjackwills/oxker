#![forbid(unsafe_code)]
#![warn(
    clippy::expect_used,
    clippy::nursery,
    clippy::pedantic,
    clippy::todo,
    clippy::unused_async,
    clippy::unwrap_used
)]
#![allow(
    clippy::module_name_repetitions,
    clippy::doc_markdown,
    clippy::similar_names
)]
// Only allow when debugging
#![allow(unused)]

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
    is_running: &Arc<AtomicBool>,
    host: Option<String>,
) {
    let connection = host.map_or_else(Docker::connect_with_socket_defaults, |host| {
        Docker::connect_with_socket(&host, 120, API_DEFAULT_VERSION)
    });

    if let Ok(docker) = connection {
        if docker.ping().await.is_ok() {
            let app_data = Arc::clone(app_data);
            let gui_state = Arc::clone(gui_state);
            let is_running = Arc::clone(is_running);

            tokio::spawn(DockerData::init(
                app_data, docker, docker_rx, docker_tx, gui_state, is_running,
            ));
        } else {
            app_data
                .lock()
                .set_error(AppError::DockerConnect, gui_state, Status::DockerConnect);
        }
    } else {
        app_data
            .lock()
            .set_error(AppError::DockerConnect, gui_state, Status::DockerConnect);
    }
}

/// Create data for, and then spawn a tokio thread, for the input handler
fn handler_init(
    app_data: &Arc<Mutex<AppData>>,
    docker_sx: &Sender<DockerMessage>,
    gui_state: &Arc<Mutex<GuiState>>,
    input_rx: Receiver<InputMessages>,
    is_running: &Arc<AtomicBool>,
) {
    let app_data = Arc::clone(app_data);
    let gui_state = Arc::clone(gui_state);
    let is_running = Arc::clone(is_running);
    tokio::spawn(input_handler::InputHandler::init(
        app_data,
        input_rx,
        docker_sx.clone(),
        gui_state,
        is_running,
    ));
}

#[tokio::main]
async fn main() {
    setup_tracing();

    let args = CliArgs::new();

    if args.in_container {
        std::thread::sleep(std::time::Duration::from_millis(250));
    }
    let host = read_docker_host(&args);

    let app_data = Arc::new(Mutex::new(AppData::default(args.clone())));
    let gui_state = Arc::new(Mutex::new(GuiState::default()));
    let is_running = Arc::new(AtomicBool::new(true));
    let (docker_tx, docker_rx) = tokio::sync::mpsc::channel(32);

    docker_init(
        &app_data,
        docker_rx,
        docker_tx.clone(),
        &gui_state,
        &is_running,
        host,
    )
    .await;

    if args.gui {
        let (input_sx, input_rx) = tokio::sync::mpsc::channel(32);
        handler_init(&app_data, &docker_tx, &gui_state, input_rx, &is_running);
        Ui::create(app_data, docker_tx.clone(), gui_state, is_running, input_sx).await;
    } else {
        info!("in debug mode\n");
        // Debug mode for testing, less pointless now, will display some basic information
        while is_running.load(Ordering::SeqCst) {
            if let Some(err) = app_data.lock().get_error() {
                error!("{}", err);
                process::exit(1);
            }
            tokio::time::sleep(std::time::Duration::from_millis(u64::from(
                args.docker_interval,
            )))
            .await;
            let containers = app_data
                .lock()
                .get_container_items()
                .clone()
                .iter()
                .map(|i| format!("{i}"))
                .collect::<Vec<_>>();

            if !containers.is_empty() {
                for item in containers {
                    info!("{item}");
                }
                println!();
            }
        }
    }
}
