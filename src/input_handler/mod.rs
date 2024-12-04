use std::{
    fs::OpenOptions,
    io::{BufWriter, Write},
    sync::{atomic::AtomicBool, Arc},
    time::SystemTime,
};

use bollard::container::LogsOptions;
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
    app_data::{AppData, DockerCommand, Header},
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
    pub async fn start(
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
        inner.message_handler().await;
    }

    /// check for incoming messages
    async fn message_handler(&mut self) {
        while let Some(message) = self.rec.recv().await {
            match message {
                InputMessages::ButtonPress(key) => self.button_press(key.0, key.1).await,
                InputMessages::MouseEvent(mouse_event) => {
                    let status = self.gui_state.lock().get_status();
                    let contains = |s: Status| status.contains(&s);

                    if !contains(Status::Error)
                        | !contains(Status::Help)
                        | !contains(Status::DeleteConfirm)
                        | !contains(Status::Filter)
                    {
                        self.mouse_press(mouse_event);
                    }
                    if contains(Status::DeleteConfirm) {
                        self.button_intersect(mouse_event).await;
                    }
                }
            }
        }
    }

    /// Sort the containers by a given header
    fn sort(&self, selected_header: Header) {
        self.app_data.lock().set_sort_by_header(selected_header);
    }

    /// Send a quit message to docker, to abort all spawns, if an error is returned, set is_running to false here instead
    /// If gui_status is Error or Init, then just set the is_running to false immediately, for a quicker exit
    fn quit(&self) {
        let status = self.gui_state.lock().get_status();
        let contains = |s: Status| status.contains(&s);
        if !contains(Status::Error) | !contains(Status::Init) {
            self.is_running
                .store(false, std::sync::atomic::Ordering::SeqCst);
        }
    }

    /// This is executed from the Delete Confirm dialog, and will send an internal message to actually remove the given container
    async fn confirm_delete(&self) {
        let id = self.gui_state.lock().get_delete_container();
        if let Some(id) = id {
            self.docker_tx
                .send(DockerMessage::Control((DockerCommand::Delete, id)))
                .await
                .ok();
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
            GuiState::start_loading_animation(&self.gui_state, uuid);
            let (sx, rx) = tokio::sync::oneshot::channel();
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
            self.gui_state.lock().stop_loading_animation(uuid);
        }
    }

    /// Toggle the mouse capture (via input of the 'm' key)
    fn m_key(&mut self) {
        let err = || {
            self.app_data.lock().set_error(
                AppError::MouseCapture(!self.mouse_capture),
                &self.gui_state,
                Status::Error,
            );
        };
        if self.mouse_capture {
            if execute!(std::io::stdout(), DisableMouseCapture).is_ok() {
                self.gui_state
                    .lock()
                    .set_info_box("✖ mouse capture disabled");
            } else {
                err();
            }
        } else if Ui::enable_mouse_capture().is_ok() {
            self.gui_state
                .lock()
                .set_info_box("✓ mouse capture enabled");
        } else {
            err();
        };

        self.mouse_capture = !self.mouse_capture;
    }

    /// Save the currently selected containers logs into a `[container_name]_[timestamp].log` file
    async fn save_logs(&self) -> Result<(), Box<dyn std::error::Error>> {
        let args = self.app_data.lock().args.clone();
        let container = self.app_data.lock().get_selected_container_id_state_name();
        if let Some((id, _, name)) = container {
            if let Some(log_path) = args.save_dir {
                let (sx, rx) = tokio::sync::oneshot::channel();
                self.docker_tx.send(DockerMessage::Exec(sx)).await?;

                let now = SystemTime::now()
                    .duration_since(SystemTime::UNIX_EPOCH)
                    .map_or(0, |i| i.as_secs());

                let path = log_path.join(format!("{name}_{now}.log"));

                let options = Some(LogsOptions::<String> {
                    stderr: true,
                    stdout: true,
                    timestamps: args.timestamp,
                    since: 0,
                    ..Default::default()
                });
                let mut logs = rx.await?.logs(id.get(), options);
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
                            .truncate(true)
                            .open(&path)?,
                    );

                    for line in &output {
                        stream.write_all(line.as_bytes())?;
                    }
                    stream.flush()?;

                    self.gui_state
                        .lock()
                        .set_info_box(&format!("saved to {}", path.display()));
                }
            }
        }
        Ok(())
    }

    /// Attempt to save the currently selected container logs to a file
    async fn s_key(&self) {
        let status = self.gui_state.lock().get_status();
        let contains = |s: Status| status.contains(&s);

        if !contains(Status::Logs) {
            self.gui_state.lock().status_push(Status::Logs);
            let uuid = Uuid::new_v4();
            GuiState::start_loading_animation(&self.gui_state, uuid);
            if self.save_logs().await.is_err() {
                self.app_data.lock().set_error(
                    AppError::DockerLogs,
                    &self.gui_state,
                    Status::Error,
                );
            }
            self.gui_state.lock().status_del(Status::Logs);
            self.gui_state.lock().stop_loading_animation(uuid);
        }
    }

    /// Send docker command, if the Commands panel is selected
    async fn enter_key(&self) {
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
                        DockerCommand::Delete => self
                            .docker_tx
                            .send(DockerMessage::ConfirmDelete(id))
                            .await
                            .ok(),

                        _ => self
                            .docker_tx
                            .send(DockerMessage::Control((command, id)))
                            .await
                            .ok(),
                    };
                }
            }
        }
    }

    /// Change the the "next" selectable panel
    /// If no containers, and on Commands panel, skip to next panel, as Commands panel isn't visible in this state
    fn tab_key(&self) {
        self.gui_state.lock().next_panel();
        if self.app_data.lock().get_container_len() == 0
            && self.gui_state.lock().get_selected_panel() == SelectablePanel::Commands
        {
            self.gui_state.lock().next_panel();
        }
    }

    /// Change to previously selected panel
    /// Need to skip the commands planel if there no are current containers running
    fn back_tab_key(&self) {
        self.gui_state.lock().previous_panel();
        if self.app_data.lock().get_container_len() == 0
            && self.gui_state.lock().get_selected_panel() == SelectablePanel::Commands
        {
            self.gui_state.lock().previous_panel();
        }
    }

    fn home_key(&self) {
        let selected_panel = self.gui_state.lock().get_selected_panel();
        match selected_panel {
            SelectablePanel::Containers => self.app_data.lock().containers_start(),
            SelectablePanel::Logs => self.app_data.lock().log_start(),
            SelectablePanel::Commands => self.app_data.lock().docker_controls_start(),
        }
    }

    /// Go to end of the list of the currently selected panel
    fn end_key(&self) {
        let selected_panel = self.gui_state.lock().get_selected_panel();
        match selected_panel {
            SelectablePanel::Containers => self.app_data.lock().containers_end(),
            SelectablePanel::Logs => self.app_data.lock().log_end(),
            SelectablePanel::Commands => self.app_data.lock().docker_controls_end(),
        }
    }

    /// Actions to take when in Help status active
    fn handle_help(&mut self, key_code: KeyCode) {
        match key_code {
            KeyCode::Esc | KeyCode::Char('h' | 'H') => {
                self.gui_state.lock().status_del(Status::Help);
            }
            KeyCode::Char('m' | 'M') => self.m_key(),
            _ => (),
        }
    }

    /// Actions to take when Error status active
    fn handle_error(&self, key_code: KeyCode) {
        match key_code {
            KeyCode::Esc | KeyCode::Char('c' | 'C') => {
                self.app_data.lock().remove_error();
                self.gui_state.lock().status_del(Status::Error);
            }
            _ => (),
        }
    }

    /// Actions to take when Delete status active
    async fn handle_delete(&self, key_code: KeyCode) {
        match key_code {
            KeyCode::Char('y' | 'Y') => self.confirm_delete().await,
            KeyCode::Esc | KeyCode::Char('n' | 'N') => self.clear_delete(),
            _ => (),
        }
    }

    /// Actions to take when Filter status active
    fn handle_filter(&self, key_code: KeyCode) {
        match key_code {
            KeyCode::Esc => {
                self.app_data.lock().filter_term_clear();
                self.gui_state.lock().status_del(Status::Filter);
            }
            KeyCode::Enter | KeyCode::F(1) | KeyCode::Char('/') => {
                self.gui_state.lock().status_del(Status::Filter);
            }
            KeyCode::Backspace => {
                self.app_data.lock().filter_term_pop();
            }
            KeyCode::Char(x) => {
                self.app_data.lock().filter_term_push(x);
            }
            KeyCode::Right => {
                self.app_data.lock().filter_by_next();
            }
            KeyCode::Left => {
                self.app_data.lock().filter_by_prev();
            }
            _ => (),
        }
    }

    /// Handle button presses in all other scenarios
    async fn handle_others(&mut self, key_code: KeyCode) {
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
            KeyCode::F(1) | KeyCode::Char('/') => {
                self.gui_state.lock().status_push(Status::Filter);
                self.docker_tx.send(DockerMessage::Update).await.ok();
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
    /// Handle keyboard button events
    async fn button_press(&mut self, key_code: KeyCode, key_modifier: KeyModifiers) {
        let status = self.gui_state.lock().get_status();
        let contains = |s: Status| status.contains(&s);

        let contains_error = contains(Status::Error);
        let contains_help = contains(Status::Help);
        let contains_exec = contains(Status::Exec);
        let contains_filter = contains(Status::Filter);
        let contains_delete = contains(Status::DeleteConfirm);

        if !contains_exec {
            let is_c = || key_code == KeyCode::Char('c') || key_code == KeyCode::Char('C');
            let is_q = || key_code == KeyCode::Char('q') || key_code == KeyCode::Char('Q');
            if key_modifier == KeyModifiers::CONTROL && is_c() || is_q() && !contains_filter {
                // Always just quit on Ctrl + c/C or q/Q, unless in FIlter status active
                self.quit();
            }

            if contains_error {
                self.handle_error(key_code);
            } else if contains_help {
                self.handle_help(key_code);
            } else if contains_filter {
                self.handle_filter(key_code);
            } else if contains_delete {
                self.handle_delete(key_code).await;
            } else {
                self.handle_others(key_code).await;
            }
        }
    }

    /// Check if a button press interacts with either the yes or no buttons in the delete container confirm window
    async fn button_intersect(&self, mouse_event: MouseEvent) {
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
    fn mouse_press(&self, mouse_event: MouseEvent) {
        match mouse_event.kind {
            MouseEventKind::ScrollUp => self.previous(),
            MouseEventKind::ScrollDown => self.next(),
            MouseEventKind::Down(MouseButton::Left) => {
                let header = self.gui_state.lock().header_intersect(Rect::new(
                    mouse_event.column,
                    mouse_event.row,
                    1,
                    1,
                ));
                if let Some(header) = header {
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
    fn next(&self) {
        let selected_panel = self.gui_state.lock().get_selected_panel();
        match selected_panel {
            SelectablePanel::Containers => self.app_data.lock().containers_next(),
            SelectablePanel::Logs => self.app_data.lock().log_next(),
            SelectablePanel::Commands => self.app_data.lock().docker_controls_next(),
        };
    }

    /// Change state to previous, depending which panel is currently in focus
    fn previous(&self) {
        let selected_panel = self.gui_state.lock().get_selected_panel();
        match selected_panel {
            SelectablePanel::Containers => self.app_data.lock().containers_previous(),
            SelectablePanel::Logs => self.app_data.lock().log_previous(),
            SelectablePanel::Commands => self.app_data.lock().docker_controls_previous(),
        }
    }
}
