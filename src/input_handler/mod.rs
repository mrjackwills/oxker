use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

use crossterm::{
    event::{
        DisableMouseCapture, EnableMouseCapture, KeyCode, MouseButton, MouseEvent, MouseEventKind,
    },
    execute,
};
use parking_lot::Mutex;
use tokio::{
    sync::mpsc::{Receiver, Sender},
    task::JoinHandle,
};
use tui::layout::Rect;

mod message;
use crate::{
    app_data::{AppData, DockerControls},
    app_error::AppError,
    docker_data::DockerMessage,
    ui::{GuiState, SelectablePanel},
};
pub use message::InputMessages;

/// Handle all input events
#[derive(Debug)]
pub struct InputHandler {
    app_data: Arc<Mutex<AppData>>,
    docker_sender: Sender<DockerMessage>,
    gui_state: Arc<Mutex<GuiState>>,
    info_sleep: Option<JoinHandle<()>>,
    is_running: Arc<AtomicBool>,
    mouse_capture: bool,
    rec: Receiver<InputMessages>,
}

impl InputHandler {
    /// Initialize self, and running the message handling loop
    pub async fn init(
        app_data: Arc<Mutex<AppData>>,
        rec: Receiver<InputMessages>,
        docker_sender: Sender<DockerMessage>,
        gui_state: Arc<Mutex<GuiState>>,
        is_running: Arc<AtomicBool>,
    ) {
        let mut inner = Self {
            app_data,
            docker_sender,
            gui_state,
            is_running,
            rec,
            mouse_capture: true,
            info_sleep: None,
        };
        inner.start().await;
    }

    /// check for incoming messages
    async fn start(&mut self) {
        while let Some(message) = self.rec.recv().await {
            match message {
                InputMessages::ButtonPress(key_code) => self.button_press(key_code).await,
                InputMessages::MouseEvent(mouse_event) => {
                    let show_error = self.app_data.lock().show_error;
                    let show_info = self.gui_state.lock().show_help;
                    if !show_error && !show_info {
                        self.mouse_press(mouse_event);
                    }
                }
            }
            if !self.is_running.load(Ordering::SeqCst) {
                break;
            }
        }
    }

    fn m_button(&mut self) {
        if self.mouse_capture {
            match execute!(std::io::stdout(), DisableMouseCapture) {
                Ok(_) => self
                    .gui_state
                    .lock()
                    .set_info_box("✖ mouse capture disabled".to_owned()),
                Err(_) => self
                    .app_data
                    .lock()
                    .set_error(AppError::MouseCapture(false)),
            }
        } else {
            match execute!(std::io::stdout(), EnableMouseCapture) {
                Ok(_) => self
                    .gui_state
                    .lock()
                    .set_info_box("✓ mouse capture enabled".to_owned()),
                Err(_) => self.app_data.lock().set_error(AppError::MouseCapture(true)),
            }
        };

        let gui_state = Arc::clone(&self.gui_state);

        if self.info_sleep.is_some() {
            self.info_sleep.as_ref().unwrap().abort()
        }
        self.info_sleep = Some(tokio::spawn(async move {
            tokio::time::sleep(std::time::Duration::from_millis(4000)).await;
            gui_state.lock().reset_info_box()
        }));

        self.mouse_capture = !self.mouse_capture;
    }

    /// Handle any keyboard button events
    async fn button_press(&mut self, key_code: KeyCode) {
        let show_error = self.app_data.lock().show_error;
        let show_info = self.gui_state.lock().show_help;

        if show_error {
            match key_code {
                KeyCode::Char('q') => {
                    self.is_running.store(false, Ordering::SeqCst);
                }
                KeyCode::Char('c') => {
                    self.app_data.lock().show_error = false;
                    self.app_data.lock().remove_error();
                }
                _ => (),
            }
        } else if show_info {
            match key_code {
                KeyCode::Char('q') => self.is_running.store(false, Ordering::SeqCst),
                KeyCode::Char('h') => self.gui_state.lock().show_help = false,
                KeyCode::Char('m') => self.m_button(),
                _ => (),
            }
        } else {
            match key_code {
                KeyCode::Char('q') => self.is_running.store(false, Ordering::SeqCst),
                KeyCode::Char('h') => self.gui_state.lock().show_help = true,
                KeyCode::Char('m') => self.m_button(),
                KeyCode::Tab => self.gui_state.lock().next_panel(),
                KeyCode::BackTab => self.gui_state.lock().previous_panel(),
                KeyCode::Home => {
                    let mut locked_data = self.app_data.lock();
                    match self.gui_state.lock().selected_panel {
                        SelectablePanel::Containers => locked_data.containers.start(),
                        SelectablePanel::Logs => locked_data.log_start(),
                        SelectablePanel::Commands => locked_data.docker_command_start(),
                    }
                }
                KeyCode::End => {
                    let mut locked_data = self.app_data.lock();
                    match self.gui_state.lock().selected_panel {
                        SelectablePanel::Containers => locked_data.containers.end(),
                        SelectablePanel::Logs => locked_data.log_end(),
                        SelectablePanel::Commands => locked_data.docker_command_end(),
                    }
                }
                KeyCode::Up => self.previous(),
                KeyCode::PageUp => {
                    for _ in 0..=6 {
                        self.previous()
                    }
                }
                KeyCode::Down => self.next(),
                KeyCode::PageDown => {
                    for _ in 0..=6 {
                        self.next()
                    }
                }
                KeyCode::Enter => {
                    // This isn't great, just means you can't send docker commands before full initialization of the program
                    // could change to to if loading = true, although at the moment don't have a loading bool
                    // Does is matter though?
                    let panel = self.gui_state.lock().selected_panel;
                    if panel == SelectablePanel::Commands {
                        let command = self.app_data.lock().get_docker_command();

                        if command.is_some() {
                            let id = self.app_data.lock().get_selected_container_id();
                            if id.is_some() {
                                let id = id.unwrap();
                                match command.unwrap() {
                                    // TODO handle theses errors?
                                    DockerControls::Pause => self
                                        .docker_sender
                                        .send(DockerMessage::Pause(id))
                                        .await
                                        .unwrap(),
                                    DockerControls::Unpause => self
                                        .docker_sender
                                        .send(DockerMessage::Unpause(id))
                                        .await
                                        .unwrap(),
                                    DockerControls::Start => self
                                        .docker_sender
                                        .send(DockerMessage::Start(id))
                                        .await
                                        .unwrap(),
                                    DockerControls::Stop => self
                                        .docker_sender
                                        .send(DockerMessage::Stop(id))
                                        .await
                                        .unwrap(),
                                    DockerControls::Restart => self
                                        .docker_sender
                                        .send(DockerMessage::Restart(id))
                                        .await
                                        .unwrap(),
                                }
                            }
                        }
                    }
                }
                _ => (),
            }
        }
    }

    /// Handle mouse button events
    fn mouse_press(&mut self, mouse_event: MouseEvent) {
        match mouse_event.kind {
            MouseEventKind::ScrollUp => self.previous(),
            MouseEventKind::ScrollDown => self.next(),
            MouseEventKind::Down(MouseButton::Left) => {
                self.gui_state.lock().rect_insersects(Rect::new(
                    mouse_event.column,
                    mouse_event.row,
                    1,
                    1,
                ));
            }
            _ => (),
        }
    }

    /// Change state of selected container
    fn next(&mut self) {
        let mut locked_data = self.app_data.lock();
        match self.gui_state.lock().selected_panel {
            SelectablePanel::Containers => locked_data.containers.next(),
            SelectablePanel::Logs => locked_data.log_next(),
            SelectablePanel::Commands => locked_data.docker_command_next(),
        };
    }

    /// Change state of selected container
    fn previous(&mut self) {
        let mut locked_data = self.app_data.lock();
        match self.gui_state.lock().selected_panel {
            SelectablePanel::Containers => locked_data.containers.previous(),
            SelectablePanel::Logs => locked_data.log_previous(),
            SelectablePanel::Commands => locked_data.docker_command_previous(),
        }
    }
}
