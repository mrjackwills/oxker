use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, Event},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use parking_lot::Mutex;
use std::{
    io::{self, Stdout, Write},
    sync::{atomic::Ordering, Arc},
    time::Duration,
};
use std::{sync::atomic::AtomicBool, time::Instant};
use tokio::sync::mpsc::Sender;
use tracing::error;
use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout},
    Frame, Terminal,
};

mod color_match;
mod draw_blocks;
mod gui_state;

pub use self::color_match::*;
pub use self::gui_state::{GuiState, SelectablePanel, Status};
use crate::{
    app_data::AppData, app_error::AppError, docker_data::DockerMessage,
    input_handler::InputMessages,
};

pub struct Ui {
    app_data: Arc<Mutex<AppData>>,
    docker_sx: Sender<DockerMessage>,
    gui_state: Arc<Mutex<GuiState>>,
    input_poll_rate: Duration,
    is_running: Arc<AtomicBool>,
    now: Instant,
    sender: Sender<InputMessages>,
    terminal: Terminal<CrosstermBackend<Stdout>>,
}

impl Ui {
    /// Enable mouse capture, but don't enable capture of all the mouse movements, doing so will improve performance, and is part of the fix for the weird mouse event output bug
    pub fn enable_mouse_capture() -> Result<()> {
        io::stdout().write_all(
            concat!(
                crossterm::csi!("?1000h"),
                crossterm::csi!("?1015h"),
                crossterm::csi!("?1006h"),
            )
            .as_bytes(),
        )?;
        Ok(())
    }

    /// Create a new Ui struct, and execute the drawing loop
    pub async fn create(
        app_data: Arc<Mutex<AppData>>,
        docker_sx: Sender<DockerMessage>,
        gui_state: Arc<Mutex<GuiState>>,
        is_running: Arc<AtomicBool>,
        sender: Sender<InputMessages>,
    ) {
        if let Ok(terminal) = Self::setup_terminal() {
            let mut ui = Self {
                app_data,
                docker_sx,
                gui_state,
                input_poll_rate: std::time::Duration::from_millis(100),
                is_running,
                now: Instant::now(),
                sender,
                terminal,
            };
            if let Err(e) = ui.draw_ui().await {
                error!("{e}");
            }
            if let Err(e) = ui.reset_terminal() {
                error!("{e}");
            };
        } else {
            error!("Terminal Error");
        }
    }

    /// Setup the terminal for full-screen drawing mode, with mouse capture
    fn setup_terminal() -> Result<Terminal<CrosstermBackend<Stdout>>> {
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen)?;
        Self::enable_mouse_capture()?;
        let backend = CrosstermBackend::new(stdout);
        Ok(Terminal::new(backend)?)
    }

    /// This is a fix for mouse-events being printed to screen, read an event and do nothing with it
    fn nullify_event_read(&self) {
        if crossterm::event::poll(self.input_poll_rate).unwrap_or(true) {
            event::read().ok();
        }
    }

    /// reset the terminal back to default settings
    pub fn reset_terminal(&mut self) -> Result<()> {
        self.terminal.clear()?;

        execute!(
            self.terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        )?;
        disable_raw_mode()?;
        self.terminal.show_cursor()?;
        Ok(())
    }

    /// Draw the the error message ui, for 5 seconds, with a countdown
    fn err_loop(&mut self) -> Result<(), AppError> {
        let mut seconds = 5;
        loop {
            if self.now.elapsed() >= std::time::Duration::from_secs(1) {
                seconds -= 1;
                self.now = Instant::now();
                if seconds < 1 {
                    break;
                }
            }

            // This is a fix for mouse-events being printed to screen
            self.nullify_event_read();

            if self
                .terminal
                .draw(|f| draw_blocks::error(f, AppError::DockerConnect, Some(seconds)))
                .is_err()
            {
                return Err(AppError::Terminal);
            }
        }
        Ok(())
    }

    /// The loop for drawing the main UI to the terminal
    async fn gui_loop(&mut self) -> Result<(), AppError> {
        let update_duration =
            std::time::Duration::from_millis(u64::from(self.app_data.lock().args.docker_interval));

        while self.is_running.load(Ordering::SeqCst) {
            if self
                .terminal
                .draw(|frame| draw_frame(frame, &self.app_data, &self.gui_state))
                .is_err()
            {
                return Err(AppError::Terminal);
            }
            if crossterm::event::poll(self.input_poll_rate).unwrap_or(false) {
                if let Ok(event) = event::read() {
                    if let Event::Key(key) = event {
                        self.sender
                            .send(InputMessages::ButtonPress(key.code))
                            .await
                            .unwrap_or(());
                    } else if let Event::Mouse(m) = event {
                        self.sender
                            .send(InputMessages::MouseEvent(m))
                            .await
                            .unwrap_or(());
                    } else if let Event::Resize(_, _) = event {
                        self.gui_state.lock().clear_area_map();
                        self.terminal.autoresize().unwrap_or(());
                    }
                }
            }

            if self.now.elapsed() >= update_duration {
                self.docker_sx
                    .send(DockerMessage::Update)
                    .await
                    .unwrap_or(());
                self.now = Instant::now();
            }
        }
        Ok(())
    }

    /// Draw either the Error, or main oxker ui, to the terminal
    async fn draw_ui(&mut self) -> Result<(), AppError> {
        let status_dockerconnect = self
            .gui_state
            .lock()
            .status_contains(&[Status::DockerConnect]);
        if status_dockerconnect {
            self.err_loop()?;
        } else {
            self.gui_loop().await?;
        }
        self.nullify_event_read();
        Ok(())
    }
}

/// Draw the main ui to a frame of the terminal
fn draw_frame<B: Backend>(
    f: &mut Frame<'_, B>,
    app_data: &Arc<Mutex<AppData>>,
    gui_state: &Arc<Mutex<GuiState>>,
) {
    // set max height for container section, needs +4 to deal with docker commands list and borders
    let height = app_data.lock().get_container_len();
    let height = if height < 12 { height + 4 } else { 12 };

    let column_widths = app_data.lock().get_width();
    let has_containers = app_data.lock().get_container_len() > 0;
    let has_error = app_data.lock().get_error();
    let sorted_by = app_data.lock().get_sorted();

    let show_help = gui_state.lock().status_contains(&[Status::Help]);
    let info_text = gui_state.lock().info_box_text.clone();
    let loading_icon = gui_state.lock().get_loading();

    let whole_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Min(100)].as_ref())
        .split(f.size());

    // Split into 3, containers+controls, logs, then graphs
    let upper_main = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Max(height.try_into().unwrap_or_default()),
                Constraint::Percentage(50),
            ]
            .as_ref(),
        )
        .split(whole_layout[1]);

    let top_split = if has_containers {
        vec![Constraint::Percentage(90), Constraint::Percentage(10)]
    } else {
        vec![Constraint::Percentage(100)]
    };
    // Containers + docker commands
    let top_panel = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(top_split.as_ref())
        .split(upper_main[0]);

    let lower_split = if has_containers {
        vec![Constraint::Percentage(75), Constraint::Percentage(25)]
    } else {
        vec![Constraint::Percentage(100)]
    };

    // Split into 2, logs, and optional charts
    let lower_main = Layout::default()
        .direction(Direction::Vertical)
        .constraints(lower_split.as_ref())
        .split(upper_main[1]);

    draw_blocks::containers(app_data, top_panel[0], f, gui_state, &column_widths);

    if has_containers {
        draw_blocks::commands(app_data, top_panel[1], f, gui_state);
    }

    draw_blocks::logs(app_data, lower_main[0], f, gui_state, &loading_icon);

    draw_blocks::heading_bar(
        whole_layout[0],
        &column_widths,
        f,
        has_containers,
        &loading_icon,
        sorted_by,
        gui_state,
    );

    // only draw charts if there are containers
    if has_containers {
        draw_blocks::chart(f, lower_main[1], app_data);
    }

    if let Some(info) = info_text {
        draw_blocks::info(f, info);
    }

    // Check if error, and show popup if so
    if show_help {
        draw_blocks::help_box(f);
    }

    if let Some(error) = has_error {
        draw_blocks::error(f, error, None);
    }
}
