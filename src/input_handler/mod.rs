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
    app_data::{AppData, DockerControls, Header, SortedOrder},
    app_error::AppError,
    docker_data::DockerMessage,
    ui::{GuiState, SelectablePanel, Status},
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
                    let error_or_help = self
                        .gui_state
                        .lock()
                        .status_contains(&[Status::Error, Status::Help]);
                    if !error_or_help {
                        self.mouse_press(mouse_event);
                    }
                }
            }
            if !self.is_running.load(Ordering::SeqCst) {
                break;
            }
        }
    }

    /// Toggle the mouse capture (via input of the 'm' key)
    fn m_key(&mut self) {
        if self.mouse_capture {
            match execute!(std::io::stdout(), DisableMouseCapture) {
                Ok(_) => self
                    .gui_state
                    .lock()
                    .set_info_box("✖ mouse capture disabled".to_owned()),
                Err(_) => {
                    self.app_data
                        .lock()
                        .set_error(AppError::MouseCapture(false));
                }
            }
        } else {
            match execute!(std::io::stdout(), EnableMouseCapture) {
                Ok(_) => self
                    .gui_state
                    .lock()
                    .set_info_box("✓ mouse capture enabled".to_owned()),
                Err(_) => {
                    self.app_data.lock().set_error(AppError::MouseCapture(true));
                }
            }
        };

        // If the info box sleep handle is currently being executed, as in 'm' is pressed twice within a 4000ms window
        // then cancel the first handle, as a new handle will be invoked
        if let Some(info_sleep_timer) = self.info_sleep.as_ref() {
            info_sleep_timer.abort();
        }

        let gui_state = Arc::clone(&self.gui_state);
        // Show the info box - with "mouse capture enabled / disabled", for 4000 ms
        self.info_sleep = Some(tokio::spawn(async move {
            tokio::time::sleep(std::time::Duration::from_millis(4000)).await;
            gui_state.lock().reset_info_box();
        }));

        self.mouse_capture = !self.mouse_capture;
    }

    /// Sort containers based on a given header, if headings match, and already ascending, remove sorting
    fn sort(&self, selected_header: Header) {
        let mut locked_data = self.app_data.lock();
        let mut output = Some((selected_header, SortedOrder::Asc));
        if let Some((current_header, order)) = locked_data.get_sorted() {
            if current_header == selected_header {
                match order {
                    SortedOrder::Desc => output = None,
                    SortedOrder::Asc => output = Some((selected_header, SortedOrder::Desc)),
                }
            }
        }
        locked_data.set_sorted(output);
    }

    /// Send a quit message to docker, to abort all spawns, if an error is returned, set is_running to false here instead
    /// If gui_status is Error or Init, then just set the is_running to false immediately, for a quicker exit
    async fn quit(&self) {
        let error_init = self
            .gui_state
            .lock()
            .status_contains(&[Status::Error, Status::Init]);
        if error_init || self.docker_sender.send(DockerMessage::Quit).await.is_err() {
            self.is_running.store(false, Ordering::SeqCst);
        }
    }

    /// Handle any keyboard button events
    #[allow(clippy::too_many_lines)]
    async fn button_press(&mut self, key_code: KeyCode) {
        // TODO - refactor this to a single call, maybe return Error, Help or Normal
        let contains_error = self.gui_state.lock().status_contains(&[Status::Error]);
        let contains_help = self.gui_state.lock().status_contains(&[Status::Help]);

        if contains_error {
            match key_code {
                KeyCode::Char('q' | 'Q') => self.quit().await,
                KeyCode::Char('c' | 'C') => {
                    self.app_data.lock().remove_error();
                    self.gui_state.lock().status_del(Status::Error);
                }
                _ => (),
            }
        } else if contains_help {
            match key_code {
                KeyCode::Char('q' | 'Q') => self.quit().await,
                KeyCode::Char('h' | 'H') => self.gui_state.lock().status_del(Status::Help),
                KeyCode::Char('m' | 'M') => self.m_key(),
                _ => (),
            }
        } else {
            match key_code {
                KeyCode::Char('0') => self.app_data.lock().set_sorted(None),
                KeyCode::Char('1') => self.sort(Header::State),
                KeyCode::Char('2') => self.sort(Header::Status),
                KeyCode::Char('3') => self.sort(Header::Cpu),
                KeyCode::Char('4') => self.sort(Header::Memory),
                KeyCode::Char('5') => self.sort(Header::Id),
                KeyCode::Char('6') => self.sort(Header::Name),
                KeyCode::Char('7') => self.sort(Header::Image),
                KeyCode::Char('8') => self.sort(Header::Rx),
                KeyCode::Char('9') => self.sort(Header::Tx),
                KeyCode::Char('q' | 'Q') => self.quit().await,
                KeyCode::Char('h' | 'H') => self.gui_state.lock().status_push(Status::Help),
                KeyCode::Char('m' | 'M') => self.m_key(),
                KeyCode::Tab => {
                    // Skip control panel if no containers, could be refactored
                    let has_containers = self.app_data.lock().get_container_len() == 0;
                    let is_containers =
                        self.gui_state.lock().selected_panel == SelectablePanel::Containers;
                    let count = if has_containers && is_containers {
                        2
                    } else {
                        1
                    };
                    for _ in 0..count {
                        self.gui_state.lock().next_panel();
                    }
                }
                KeyCode::BackTab => {
                    // Skip control panel if no containers, could be refactored
                    let has_containers = self.app_data.lock().get_container_len() == 0;
                    let is_containers =
                        self.gui_state.lock().selected_panel == SelectablePanel::Logs;
                    let count = if has_containers && is_containers {
                        2
                    } else {
                        1
                    };
                    for _ in 0..count {
                        self.gui_state.lock().previous_panel();
                    }
                }
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
                KeyCode::Up | KeyCode::Char('k' | 'K') => self.previous(),
                KeyCode::PageUp => {
                    for _ in 0..=6 {
                        self.previous();
                    }
                }
                KeyCode::Down | KeyCode::Char('j' | 'J') => self.next(),
                KeyCode::PageDown => {
                    for _ in 0..=6 {
                        self.next();
                    }
                }
                KeyCode::Enter => {
                    // This isn't great, just means you can't send docker commands before full initialization of the program
                    let panel = self.gui_state.lock().selected_panel;
                    if panel == SelectablePanel::Commands {
                        let option_command = self.app_data.lock().get_docker_command();

                        if let Some(command) = option_command {
                            let option_id = self.app_data.lock().get_selected_container_id();
                            // Poor way of disallowing commands to be sent to a containerised okxer
                            if self.app_data.lock().selected_container_is_oxker() {
                                return;
                            };
                            if let Some(id) = option_id {
                                match command {
                                    DockerControls::Pause => self
                                        .docker_sender
                                        .send(DockerMessage::Pause(id))
                                        .await
                                        .unwrap_or(()),
                                    DockerControls::Unpause => self
                                        .docker_sender
                                        .send(DockerMessage::Unpause(id))
                                        .await
                                        .unwrap_or(()),
                                    DockerControls::Start => self
                                        .docker_sender
                                        .send(DockerMessage::Start(id))
                                        .await
                                        .unwrap_or(()),
                                    DockerControls::Stop => self
                                        .docker_sender
                                        .send(DockerMessage::Stop(id))
                                        .await
                                        .unwrap_or(()),
                                    DockerControls::Restart => self
                                        .docker_sender
                                        .send(DockerMessage::Restart(id))
                                        .await
                                        .unwrap_or(()),
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
                let header_intersects = self.gui_state.lock().header_intersect(Rect::new(
                    mouse_event.column,
                    mouse_event.row,
                    1,
                    1,
                ));

                if let Some(header) = header_intersects {
                    self.sort(header);
                }

                self.gui_state.lock().panel_intersect(Rect::new(
                    mouse_event.column,
                    mouse_event.row,
                    1,
                    1,
                ));
            }
            _ => (),
        }
    }

    /// Change state to next, depending which panel is currently in focus
    fn next(&mut self) {
        let mut locked_data = self.app_data.lock();
        match self.gui_state.lock().selected_panel {
            SelectablePanel::Containers => locked_data.containers.next(),
            SelectablePanel::Logs => locked_data.log_next(),
            SelectablePanel::Commands => locked_data.docker_command_next(),
        };
    }

    /// Change state to previous, depending which panel is currently in focus
    fn previous(&mut self) {
        let mut locked_data = self.app_data.lock();
        match self.gui_state.lock().selected_panel {
            SelectablePanel::Containers => locked_data.containers.previous(),
            SelectablePanel::Logs => locked_data.log_previous(),
            SelectablePanel::Commands => locked_data.docker_command_previous(),
        }
    }
}
