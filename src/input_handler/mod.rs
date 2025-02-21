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
    config,
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
    keymap: config::Keymap,
    gui_state: Arc<Mutex<GuiState>>,
    is_running: Arc<AtomicBool>,
    mouse_capture: bool,
    rx: Receiver<InputMessages>,
}

impl InputHandler {
    /// Initialize self, and running the message handling loop
    pub async fn start(
        app_data: Arc<Mutex<AppData>>,
        docker_tx: Sender<DockerMessage>,
        gui_state: Arc<Mutex<GuiState>>,
        is_running: Arc<AtomicBool>,
        rx: Receiver<InputMessages>,
    ) {
        let keymap = app_data.lock().config.keymap.clone();
        let mut inner = Self {
            app_data,
            docker_tx,
            gui_state,
            is_running,
            keymap,
            rx,
            mouse_capture: true,
        };
        inner.message_handler().await;
    }

    /// check for incoming messages
    async fn message_handler(&mut self) {
        while let Some(message) = self.rx.recv().await {
            match message {
                InputMessages::ButtonPress(key) => self.button_press(key.0, key.1).await,
                InputMessages::MouseEvent(mouse_event) => {
                    let status = self.gui_state.lock().get_status();
                    let contains = |s: Status| status.contains(&s);

                    if contains(Status::DeleteConfirm) {
                        self.button_intersect(mouse_event).await;
                    } else if !contains(Status::Error)
                        | !contains(Status::Help)
                        | !contains(Status::DeleteConfirm)
                        | !contains(Status::Filter)
                    {
                        self.mouse_press(mouse_event);
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
    async fn exec_key(&self) {
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
    fn mouse_capture_key(&mut self) {
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
        let args = self.app_data.lock().config.clone();
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
                    timestamps: args.show_timestamp,
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
    async fn save_key(&self) {
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
    fn next_panel_key(&self) {
        self.gui_state.lock().next_panel();
        if self.app_data.lock().get_container_len() == 0
            && self.gui_state.lock().get_selected_panel() == SelectablePanel::Commands
        {
            self.gui_state.lock().next_panel();
        }
    }

    /// Change to previously selected panel
    /// Need to skip the commands planel if there no are current containers running
    fn previous_panel_key(&self) {
        self.gui_state.lock().previous_panel();
        if self.app_data.lock().get_container_len() == 0
            && self.gui_state.lock().get_selected_panel() == SelectablePanel::Commands
        {
            self.gui_state.lock().previous_panel();
        }
    }

    fn scroll_start_key(&self) {
        let selected_panel = self.gui_state.lock().get_selected_panel();
        match selected_panel {
            SelectablePanel::Containers => self.app_data.lock().containers_start(),
            SelectablePanel::Logs => self.app_data.lock().log_start(),
            SelectablePanel::Commands => self.app_data.lock().docker_controls_start(),
        }
    }

    /// Go to end of the list of the currently selected panel
    fn scroll_end_key(&self) {
        let selected_panel = self.gui_state.lock().get_selected_panel();
        match selected_panel {
            SelectablePanel::Containers => self.app_data.lock().containers_end(),
            SelectablePanel::Logs => self.app_data.lock().log_end(),
            SelectablePanel::Commands => self.app_data.lock().docker_controls_end(),
        }
    }

    /// Actions to take when in Help status active
    fn handle_help(&mut self, key_code: KeyCode) {
        if self.keymap.clear.0 == key_code
            || self.keymap.clear.1 == Some(key_code)
            || self.keymap.toggle_help.0 == key_code
            || self.keymap.toggle_help.1 == Some(key_code)
        {
            self.gui_state.lock().status_del(Status::Help);
        }

        if self.keymap.toggle_mouse_capture.0 == key_code
            || self.keymap.toggle_mouse_capture.1 == Some(key_code)
        {
            self.mouse_capture_key();
        }
    }

    /// Actions to take when Error status active
    fn handle_error(&self, key_code: KeyCode) {
        if self.keymap.clear.0 == key_code || self.keymap.clear.1 == Some(key_code) {
            self.app_data.lock().remove_error();
            self.gui_state.lock().status_del(Status::Error);
        }
    }

    /// Actions to take when Delete status active
    async fn handle_delete(&self, key_code: KeyCode) {
        if self.keymap.delete_confirm.0 == key_code
            || self.keymap.delete_confirm.1 == Some(key_code)
        {
            self.confirm_delete().await;
        } else if self.keymap.delete_deny.0 == key_code
            || self.keymap.delete_deny.1 == Some(key_code)
            || self.keymap.clear.0 == key_code
            || self.keymap.clear.1 == Some(key_code)
        {
            self.clear_delete();
        }
    }

    /// Actions to take when Filter status active
    fn handle_filter(&self, key_code: KeyCode) {
        match key_code {
            KeyCode::Esc => {
                self.app_data.lock().filter_term_clear();
                self.gui_state.lock().status_del(Status::Filter);
            }
            _ if KeyCode::Enter == key_code
                || self.keymap.filter_mode.0 == key_code
                || self.keymap.filter_mode.1 == Some(key_code) =>
            {
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

    /// Handle input that refers to the sorting of columns
    fn handle_sort(&self, key_code: KeyCode) {
        match key_code {
            _ if self.keymap.sort_reset.0 == key_code
                || self.keymap.sort_reset.1 == Some(key_code) =>
            {
                self.app_data.lock().reset_sorted();
            }

            _ if self.keymap.sort_by_name.0 == key_code
                || self.keymap.sort_by_name.1 == Some(key_code) =>
            {
                self.sort(Header::Name);
            }

            _ if self.keymap.sort_by_state.0 == key_code
                || self.keymap.sort_by_state.1 == Some(key_code) =>
            {
                self.sort(Header::State);
            }

            _ if self.keymap.sort_by_status.0 == key_code
                || self.keymap.sort_by_status.1 == Some(key_code) =>
            {
                self.sort(Header::Status);
            }

            _ if self.keymap.sort_by_cpu.0 == key_code
                || self.keymap.sort_by_cpu.1 == Some(key_code) =>
            {
                self.sort(Header::Cpu);
            }
            _ if self.keymap.sort_by_memory.0 == key_code
                || self.keymap.sort_by_memory.1 == Some(key_code) =>
            {
                self.sort(Header::Memory);
            }
            _ if self.keymap.sort_by_id.0 == key_code
                || self.keymap.sort_by_id.1 == Some(key_code) =>
            {
                self.sort(Header::Id);
            }
            _ if self.keymap.sort_by_image.0 == key_code
                || self.keymap.sort_by_image.1 == Some(key_code) =>
            {
                self.sort(Header::Image);
            }

            _ if self.keymap.sort_by_rx.0 == key_code
                || self.keymap.sort_by_rx.1 == Some(key_code) =>
            {
                self.sort(Header::Rx);
            }

            _ if self.keymap.sort_by_tx.0 == key_code
                || self.keymap.sort_by_tx.1 == Some(key_code) =>
            {
                self.sort(Header::Tx);
            }
            _ => (),
        }
    }

    /// Handle button presses in all other scenarios
    async fn handle_others(&mut self, key_code: KeyCode) {
        self.handle_sort(key_code);
        match key_code {
            _ if self.keymap.exec.0 == key_code || self.keymap.exec.1 == Some(key_code) => {
                self.exec_key().await;
            }

            _ if self.keymap.toggle_help.0 == key_code
                || self.keymap.toggle_help.1 == Some(key_code) =>
            {
                self.gui_state.lock().status_push(Status::Help);
            }

            _ if self.keymap.toggle_mouse_capture.0 == key_code
                || self.keymap.toggle_mouse_capture.1 == Some(key_code) =>
            {
                self.mouse_capture_key();
            }

            _ if self.keymap.save_logs.0 == key_code
                || self.keymap.save_logs.1 == Some(key_code) =>
            {
                self.save_key().await;
            }

            _ if self.keymap.select_next_panel.0 == key_code
                || self.keymap.select_next_panel.1 == Some(key_code) =>
            {
                self.next_panel_key();
            }

            _ if self.keymap.select_previous_panel.0 == key_code
                || self.keymap.select_previous_panel.1 == Some(key_code) =>
            {
                self.previous_panel_key();
            }

            _ if self.keymap.scroll_start.0 == key_code
                || self.keymap.scroll_start.1 == Some(key_code) =>
            {
                self.scroll_start_key();
            }

            _ if self.keymap.scroll_end.0 == key_code
                || self.keymap.scroll_end.1 == Some(key_code) =>
            {
                self.scroll_end_key();
            }

            _ if self.keymap.scroll_up_one.0 == key_code
                || self.keymap.scroll_up_one.1 == Some(key_code) =>
            {
                self.previous();
            }

            _ if self.keymap.scroll_up_many.0 == key_code
                || self.keymap.scroll_up_many.1 == Some(key_code) =>
            {
                for _ in 0..=6 {
                    self.previous();
                }
            }

            _ if self.keymap.scroll_down_one.0 == key_code
                || self.keymap.scroll_down_one.1 == Some(key_code) =>
            {
                self.next();
            }

            _ if self.keymap.scroll_down_many.0 == key_code
                || self.keymap.scroll_down_many.1 == Some(key_code) =>
            {
                for _ in 0..=6 {
                    self.next();
                }
            }

            _ if self.keymap.filter_mode.0 == key_code
                || self.keymap.filter_mode.1 == Some(key_code) =>
            {
                self.gui_state.lock().status_push(Status::Filter);
                self.docker_tx.send(DockerMessage::Update).await.ok();
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
            let is_q = || key_code == self.keymap.quit.0 || Some(key_code) == self.keymap.quit.1;
            if key_modifier == KeyModifiers::CONTROL && key_code == KeyCode::Char('c')
                || is_q() && !contains_filter
            {
                // Always just quit on Ctrl + c/C or q/Q, unless in Filter status active
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
            let intersect = self.gui_state.lock().get_intersect_button(Rect::new(
                mouse_event.column,
                mouse_event.row,
                1,
                1,
            ));

            if let Some(button) = intersect {
                match button {
                    DeleteButton::Confirm => self.confirm_delete().await,
                    DeleteButton::Cancel => self.clear_delete(),
                }
            }
        }
    }

    /// Handle mouse button events
    fn mouse_press(&self, mouse_event: MouseEvent) {
        let status = self.gui_state.lock().get_status();
        if status.contains(&Status::Help) {
            let mouse_point = Rect::new(mouse_event.column, mouse_event.row, 1, 1);
            let help_intersect = self.gui_state.lock().get_intersect_help(mouse_point);
            if help_intersect {
                self.gui_state.lock().status_del(Status::Help);
            }
        } else {
            match mouse_event.kind {
                MouseEventKind::ScrollUp => self.previous(),
                MouseEventKind::ScrollDown => self.next(),
                MouseEventKind::Down(MouseButton::Left) => {
                    let mouse_point = Rect::new(mouse_event.column, mouse_event.row, 1, 1);
                    let header = self.gui_state.lock().get_intersect_header(mouse_point);
                    if let Some(header) = header {
                        self.sort(header);
                    }
                    let help_intersect = self.gui_state.lock().get_intersect_help(mouse_point);
                    if help_intersect {
                        self.gui_state.lock().status_push(Status::Help);
                    }

                    self.gui_state.lock().get_intersect_panel(mouse_point);
                }
                _ => (),
            }
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
