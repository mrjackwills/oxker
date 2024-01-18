use std::{
    fs::OpenOptions,
    io::{BufWriter, Write},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::SystemTime,
};

use bollard::{container::LogsOptions, Docker};
use cansi::v3::categorise_text;
use crossterm::{
    event::{DisableMouseCapture, KeyCode, KeyModifiers, MouseButton, MouseEvent, MouseEventKind},
    execute,
};
use futures_util::StreamExt;
use parking_lot::Mutex;
use ratatui::layout::Rect;
use tokio::sync::mpsc::{Receiver, Sender};
use uuid::Uuid;

mod message;
use crate::{
    app_data::{AppData, DockerControls, Header},
    app_error::AppError,
    docker_data::DockerMessage,
    exec::{tty_readable, ExecMode},
    ui::{DeleteButton, GuiState, SelectablePanel, Status, Ui},
};
pub use message::InputMessages;

/// Handle all input events
#[derive(Debug)]
pub struct InputHandler {
    app_data: Arc<Mutex<AppData>>,
    docker_tx: Sender<DockerMessage>,
    gui_state: Arc<Mutex<GuiState>>,
    is_running: Arc<AtomicBool>,
    mouse_capture: bool,
    rec: Receiver<InputMessages>,
}

impl InputHandler {
    /// Initialize self, and running the message handling loop
    pub async fn init(
        app_data: Arc<Mutex<AppData>>,
        rec: Receiver<InputMessages>,
        docker_tx: Sender<DockerMessage>,
        gui_state: Arc<Mutex<GuiState>>,
        is_running: Arc<AtomicBool>,
    ) {
        let mut inner = Self {
            app_data,
            docker_tx,
            gui_state,
            is_running,
            rec,
            mouse_capture: true,
        };
        inner.start().await;
    }

    /// check for incoming messages
    async fn start(&mut self) {
        while let Some(message) = self.rec.recv().await {
            match message {
                InputMessages::ButtonPress(key) => self.button_press(key.0, key.1).await,
                InputMessages::MouseEvent(mouse_event) => {
                    if !self.gui_state.lock().status_contains(&[
                        Status::Error,
                        Status::Help,
                        Status::DeleteConfirm,
                    ]) {
                        self.mouse_press(mouse_event);
                    }
                    let delete_confirm = self
                        .gui_state
                        .lock()
                        .status_contains(&[Status::DeleteConfirm]);
                    if delete_confirm {
                        self.button_intersect(mouse_event).await;
                    }
                }
            }
            if !self.is_running.load(Ordering::SeqCst) {
                break;
            }
        }
    }

    /// Sort the containers by a given header
    fn sort(&self, selected_header: Header) {
        self.app_data.lock().set_sort_by_header(selected_header);
    }

    /// Send a quit message to docker, to abort all spawns, if an error is returned, set is_running to false here instead
    /// If gui_status is Error or Init, then just set the is_running to false immediately, for a quicker exit
    async fn quit(&self) {
        let error_init = self
            .gui_state
            .lock()
            .status_contains(&[Status::Error, Status::Init]);
        if error_init || self.docker_tx.send(DockerMessage::Quit).await.is_err() {
            self.is_running
                .store(false, std::sync::atomic::Ordering::SeqCst);
        }
    }

    /// This is executed from the Delete Confirm dialog, and will send an internal message to actually remove the given container
    async fn confirm_delete(&self) {
        let id = self.gui_state.lock().get_delete_container();
        if let Some(id) = id {
            self.docker_tx.send(DockerMessage::Delete(id)).await.ok();
        }
    }

    /// This is executed from the Delete Confirm dialog, and will clear the delete_container information (removes id and closes panel)
    fn clear_delete(&self) {
        self.gui_state.lock().set_delete_container(None);
    }

    /// Validate that one can exec into a Docker container
    async fn e_key(&self) {
        let is_oxker = self.app_data.lock().is_oxker();
        if !is_oxker && tty_readable() {
            let uuid = Uuid::new_v4();
            let handle = GuiState::start_loading_animation(&self.gui_state, uuid);
            let (sx, rx) = tokio::sync::oneshot::channel::<Arc<Docker>>();
            self.docker_tx.send(DockerMessage::Exec(sx)).await.ok();

            if let Ok(docker) = rx.await {
                (ExecMode::new(&self.app_data, &docker).await).map_or_else(
                    || {
                        self.app_data.lock().set_error(
                            AppError::DockerExec,
                            &self.gui_state,
                            Status::Error,
                        );
                    },
                    |mode| {
                        self.gui_state.lock().set_exec_mode(mode);
                    },
                );
            }
            self.gui_state.lock().stop_loading_animation(&handle, uuid);
        }
    }

    /// Toggle the mouse capture (via input of the 'm' key)
    fn m_key(&mut self) {
        if self.mouse_capture {
            if execute!(std::io::stdout(), DisableMouseCapture).is_ok() {
                self.gui_state
                    .lock()
                    .set_info_box("✖ mouse capture disabled");
            } else {
                self.app_data.lock().set_error(
                    AppError::MouseCapture(false),
                    &self.gui_state,
                    Status::Error,
                );
            }
        } else if Ui::enable_mouse_capture().is_ok() {
            self.gui_state
                .lock()
                .set_info_box("✓ mouse capture enabled");
        } else {
            self.app_data.lock().set_error(
                AppError::MouseCapture(true),
                &self.gui_state,
                Status::Error,
            );
        };

        self.mouse_capture = !self.mouse_capture;
    }

    /// Save the currently selected containers logs into a `[container_name]_[timestamp].log` file
    async fn s_key(&mut self) {
        /// This is the inner workings, *inlined* here to return a Result
        async fn save_logs(
            app_data: &Arc<Mutex<AppData>>,
            gui_state: &Arc<Mutex<GuiState>>,
            docker_tx: &Sender<DockerMessage>,
        ) -> Result<(), Box<dyn std::error::Error>> {
            let args = app_data.lock().args.clone();
            let container = app_data.lock().get_selected_container_id_state_name();
            if let Some((id, _, name)) = container {
                if let Some(log_path) = args.save_dir {
                    let (sx, rx) = tokio::sync::oneshot::channel::<Arc<Docker>>();
                    docker_tx.send(DockerMessage::Exec(sx)).await?;

                    let now = SystemTime::now()
                        .duration_since(SystemTime::UNIX_EPOCH)
                        .map_or(0, |i| i.as_secs());

                    let path = log_path.join(format!("{name}_{now}.log"));

                    let docker = rx.await?;
                    let options = Some(LogsOptions::<String> {
                        stdout: true,
                        timestamps: args.timestamp,
                        since: 0,
                        ..Default::default()
                    });
                    let mut logs = docker.logs(id.get(), options);
                    let mut output = vec![];

                    while let Some(Ok(value)) = logs.next().await {
                        let data = value.to_string();
                        if !data.trim().is_empty() {
                            output.push(
                                categorise_text(&data)
                                    .into_iter()
                                    .map(|i| i.text)
                                    .collect::<String>(),
                            );
                        }
                    }
                    if !output.is_empty() {
                        let mut stream = BufWriter::new(
                            OpenOptions::new()
                                .read(true)
                                .write(true)
                                .create(true)
                                .open(&path)?,
                        );

                        for line in &output {
                            stream.write_all(line.as_bytes())?;
                        }
                        stream.flush()?;

                        gui_state
                            .lock()
                            .set_info_box(&format!("saved to {}", path.display()));
                    }
                }
            }
            Ok(())
        }

        let log_status = Status::Logs;
        let status = self.gui_state.lock().status_contains(&[log_status]);
        if !status {
            self.gui_state.lock().status_push(log_status);

            let uuid = Uuid::new_v4();
            let handle = GuiState::start_loading_animation(&self.gui_state, uuid);
            if save_logs(&self.app_data, &self.gui_state, &self.docker_tx)
                .await
                .is_err()
            {
                self.app_data.lock().set_error(
                    AppError::DockerLogs,
                    &self.gui_state,
                    Status::Error,
                );
            }
            self.gui_state.lock().status_del(log_status);
            self.gui_state.lock().stop_loading_animation(&handle, uuid);
        }
    }

    /// Send docker command, if the Commands panel is selected
    async fn enter_key(&mut self) {
        // This isn't great, just means you can't send docker commands before full initialization of the program
        let panel = self.gui_state.lock().get_selected_panel();
        if panel == SelectablePanel::Commands {
            let option_command = self.app_data.lock().selected_docker_controls();

            if let Some(command) = option_command {
                // Poor way of disallowing commands to be sent to a containerised okxer
                if self.app_data.lock().is_oxker_in_container() {
                    return;
                };
                let option_id = self.app_data.lock().get_selected_container_id();
                if let Some(id) = option_id {
                    match command {
                        DockerControls::Delete => self
                            .docker_tx
                            .send(DockerMessage::ConfirmDelete(id))
                            .await
                            .ok(),
                        DockerControls::Pause => {
                            self.docker_tx.send(DockerMessage::Pause(id)).await.ok()
                        }
                        DockerControls::Resume => {
                            self.docker_tx.send(DockerMessage::Resume(id)).await.ok()
                        }
                        DockerControls::Start => {
                            self.docker_tx.send(DockerMessage::Start(id)).await.ok()
                        }
                        DockerControls::Stop => {
                            self.docker_tx.send(DockerMessage::Stop(id)).await.ok()
                        }
                        DockerControls::Restart => {
                            self.docker_tx.send(DockerMessage::Restart(id)).await.ok()
                        }
                    };
                }
            }
        }
    }

    /// Change the the "next" seletable panel
    fn tab_key(&mut self) {
        let is_containers =
            self.gui_state.lock().get_selected_panel() == SelectablePanel::Containers;
        let count = if self.app_data.lock().get_container_len() == 0 && is_containers {
            2
        } else {
            1
        };
        for _ in 0..count {
            self.gui_state.lock().next_panel();
        }
    }

    /// Change to previously selected panel
    fn back_tab_key(&mut self) {
        let is_containers = self.gui_state.lock().get_selected_panel() == SelectablePanel::Logs;
        let count = if self.app_data.lock().get_container_len() == 0 && is_containers {
            2
        } else {
            1
        };
        for _ in 0..count {
            self.gui_state.lock().previous_panel();
        }
    }

    fn home_key(&mut self) {
        let mut locked_data = self.app_data.lock();
        let selected_panel = self.gui_state.lock().get_selected_panel();
        match selected_panel {
            SelectablePanel::Containers => locked_data.containers_start(),
            SelectablePanel::Logs => locked_data.log_start(),
            SelectablePanel::Commands => locked_data.docker_controls_start(),
        }
    }

    /// Go to end of the list of the currently selected panel
    fn end_key(&mut self) {
        let mut locked_data = self.app_data.lock();
        let selected_panel = self.gui_state.lock().get_selected_panel();
        match selected_panel {
            SelectablePanel::Containers => locked_data.containers_end(),
            SelectablePanel::Logs => locked_data.log_end(),
            SelectablePanel::Commands => locked_data.docker_controls_end(),
        }
    }

    /// Handle keyboard button events
    async fn button_press(&mut self, key_code: KeyCode, key_modififer: KeyModifiers) {
        let contains_delete = self
            .gui_state
            .lock()
            .status_contains(&[Status::DeleteConfirm]);

        let contains = |s: Status| self.gui_state.lock().status_contains(&[s]);

        let contains_error = contains(Status::Error);
        let contains_help = contains(Status::Help);
        let contains_exec = contains(Status::Exec);

        if !contains_exec {
            // Always just quit on Ctrl + c/C or q/Q
            let is_c = || key_code == KeyCode::Char('c') || key_code == KeyCode::Char('C');
            let is_q = || key_code == KeyCode::Char('q') || key_code == KeyCode::Char('Q');
            if key_modififer == KeyModifiers::CONTROL && is_c() || is_q() {
                self.quit().await;
            }

            if contains_error {
                if let KeyCode::Char('c' | 'C') = key_code {
                    self.app_data.lock().remove_error();
                    self.gui_state.lock().status_del(Status::Error);
                }
            } else if contains_help {
                match key_code {
                    KeyCode::Char('h' | 'H') => self.gui_state.lock().status_del(Status::Help),
                    KeyCode::Char('m' | 'M') => self.m_key(),
                    _ => (),
                }
            } else if contains_delete {
                match key_code {
                    KeyCode::Char('y' | 'Y') => self.confirm_delete().await,
                    KeyCode::Char('n' | 'N') => self.clear_delete(),
                    _ => (),
                }
            } else {
                match key_code {
                    KeyCode::Char('0') => self.app_data.lock().reset_sorted(),
                    KeyCode::Char('1') => self.sort(Header::Name),
                    KeyCode::Char('2') => self.sort(Header::State),
                    KeyCode::Char('3') => self.sort(Header::Status),
                    KeyCode::Char('4') => self.sort(Header::Cpu),
                    KeyCode::Char('5') => self.sort(Header::Memory),
                    KeyCode::Char('6') => self.sort(Header::Id),
                    KeyCode::Char('7') => self.sort(Header::Image),
                    KeyCode::Char('8') => self.sort(Header::Rx),
                    KeyCode::Char('9') => self.sort(Header::Tx),
                    KeyCode::Char('e' | 'E') => self.e_key().await,
                    KeyCode::Char('h' | 'H') => self.gui_state.lock().status_push(Status::Help),
                    KeyCode::Char('m' | 'M') => self.m_key(),
                    KeyCode::Char('s' | 'S') => self.s_key().await,
                    KeyCode::Tab => self.tab_key(),
                    KeyCode::BackTab => self.back_tab_key(),
                    KeyCode::Home => self.home_key(),
                    KeyCode::End => self.end_key(),
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
                    KeyCode::Enter => self.enter_key().await,
                    _ => (),
                }
            }
        }
    }

    /// Check if a button press interacts with either the yes or no buttons in the delete container confirm window
    async fn button_intersect(&mut self, mouse_event: MouseEvent) {
        if mouse_event.kind == MouseEventKind::Down(MouseButton::Left) {
            let intersect = self.gui_state.lock().button_intersect(Rect::new(
                mouse_event.column,
                mouse_event.row,
                1,
                1,
            ));

            if let Some(button) = intersect {
                match button {
                    DeleteButton::Yes => self.confirm_delete().await,
                    DeleteButton::No => self.clear_delete(),
                }
            }
        }
    }

    /// Handle mouse button events
    fn mouse_press(&mut self, mouse_event: MouseEvent) {
        match mouse_event.kind {
            MouseEventKind::ScrollUp => self.previous(),
            MouseEventKind::ScrollDown => self.next(),
            MouseEventKind::Down(MouseButton::Left) => {
                if let Some(header) = self.gui_state.lock().header_intersect(Rect::new(
                    mouse_event.column,
                    mouse_event.row,
                    1,
                    1,
                )) {
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
        let selected_panel = self.gui_state.lock().get_selected_panel();
        match selected_panel {
            SelectablePanel::Containers => locked_data.containers_next(),
            SelectablePanel::Logs => locked_data.log_next(),
            SelectablePanel::Commands => locked_data.docker_controls_next(),
        };
    }

    /// Change state to previous, depending which panel is currently in focus
    fn previous(&mut self) {
        let mut locked_data = self.app_data.lock();
        let selected_panel = self.gui_state.lock().get_selected_panel();
        match selected_panel {
            SelectablePanel::Containers => locked_data.containers_previous(),
            SelectablePanel::Logs => locked_data.log_previous(),
            SelectablePanel::Commands => locked_data.docker_controls_previous(),
        }
    }
}
