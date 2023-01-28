#![forbid(unsafe_code)]
#![warn(clippy::unused_async, clippy::unwrap_used, clippy::expect_used)]
// Warning - These are indeed pedantic
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![allow(
    clippy::module_name_repetitions,
    clippy::doc_markdown,
    clippy::similar_names
)]
// Only allow when debugging
// #![allow(unused)]

use app_data::AppData;
use app_error::AppError;
use bollard::Docker;
use docker_data::DockerData;
use input_handler::InputMessages;
use parking_lot::Mutex;
use parse_args::CliArgs;
use std::sync::{atomic::AtomicBool, Arc};
use tokio::sync::mpsc::{Receiver, Sender};
use tracing::{info, Level};

mod app_data;
mod app_error;
mod docker_data;
mod input_handler;
mod parse_args;
mod ui;

use ui::{create_ui, GuiState, Status};

use crate::docker_data::DockerMessage;

const ENTRY_POINT: &str = "./start_oxker.sh";

/// write to file if `-g` is set?
fn setup_tracing() {
    tracing_subscriber::fmt().with_max_level(Level::INFO).init();
}

/// Create docker daemon handler, and only spawn up the docker data handler if a ping returns non-error
async fn docker_init(
    app_data: &Arc<Mutex<AppData>>,
    docker_rx: Receiver<DockerMessage>,
    gui_state: &Arc<Mutex<GuiState>>,
    is_running: &Arc<AtomicBool>,
) {
    if let Ok(docker) = Docker::connect_with_socket_defaults() {
        if docker.ping().await.is_ok() {
            let app_data = Arc::clone(app_data);
            let gui_state = Arc::clone(gui_state);
            let is_running = Arc::clone(is_running);
            tokio::spawn(DockerData::init(
                app_data, docker, docker_rx, gui_state, is_running,
            ));
        } else {
            app_data.lock().set_error(AppError::DockerConnect);
            gui_state.lock().status_push(Status::DockerConnect);
        }
    } else {
        app_data.lock().set_error(AppError::DockerConnect);
        gui_state.lock().status_push(Status::DockerConnect);
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
    let input_app_data = Arc::clone(app_data);
    let input_gui_state = Arc::clone(gui_state);
    let input_is_running = Arc::clone(is_running);
    tokio::spawn(input_handler::InputHandler::init(
        input_app_data,
        input_rx,
        docker_sx.clone(),
        input_gui_state,
        input_is_running,
    ));
}

#[tokio::main]
async fn main() {
    setup_tracing();
    let args = CliArgs::new();
    let app_data = Arc::new(Mutex::new(AppData::default(args)));
    let gui_state = Arc::new(Mutex::new(GuiState::default()));
    let is_running = Arc::new(AtomicBool::new(true));
    let (docker_sx, docker_rx) = tokio::sync::mpsc::channel(16);
    let (input_sx, input_rx) = tokio::sync::mpsc::channel(16);

    docker_init(&app_data, docker_rx, &gui_state, &is_running).await;

    handler_init(&app_data, &docker_sx, &gui_state, input_rx, &is_running);

    if args.gui {
        create_ui(app_data, docker_sx, gui_state, is_running, input_sx)
            .await
            .unwrap_or(());
    } else {
        // Debug mode for testing, mostly pointless, doesn't take terminal
        info!("in debug mode");
        loop {
            docker_sx.send(DockerMessage::Update).await.unwrap_or(());
            tokio::time::sleep(std::time::Duration::from_millis(u64::from(
                args.docker_interval,
            )))
            .await;
        }
    }
}
