use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, Event},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use parking_lot::{Mutex, MutexGuard};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Position},
    Frame, Terminal,
};
use std::{
    io::{self, Stdout, Write},
    sync::{atomic::Ordering, Arc},
    time::Duration,
};
use std::{sync::atomic::AtomicBool, time::Instant};
use tokio::sync::mpsc::Sender;
use tracing::error;

mod color_match;
mod draw_blocks;
mod gui_state;

pub use self::color_match::*;
pub use self::gui_state::{DeleteButton, GuiState, SelectablePanel, Status};
use crate::{
    app_data::{AppData, Columns, ContainerId, FilterBy, Header, SortedOrder},
    app_error::AppError,
    exec::TerminalSize,
    input_handler::InputMessages,
};

pub const ORANGE: ratatui::style::Color = ratatui::style::Color::Rgb(255, 178, 36);

pub struct Ui {
    app_data: Arc<Mutex<AppData>>,
    gui_state: Arc<Mutex<GuiState>>,
    input_poll_rate: Duration,
    input_tx: Sender<InputMessages>,
    is_running: Arc<AtomicBool>,
    now: Instant,
    terminal: Terminal<CrosstermBackend<Stdout>>,
    cursor_position: Position,
}

impl Ui {
    /// Enable mouse capture, but don't enable capture of all the mouse movements, doing so will improve performance, and is part of the fix for the weird mouse event output bug
    pub fn enable_mouse_capture() -> Result<()> {
        Ok(io::stdout().write_all(
            concat!(
                crossterm::csi!("?1000h"),
                crossterm::csi!("?1015h"),
                crossterm::csi!("?1006h"),
            )
            .as_bytes(),
        )?)
    }

    /// Create a new Ui struct, and execute the drawing loop
    pub async fn start(
        app_data: Arc<Mutex<AppData>>,
        gui_state: Arc<Mutex<GuiState>>,
        input_tx: Sender<InputMessages>,
        is_running: Arc<AtomicBool>,
    ) {
        if let Ok(mut terminal) = Self::setup_terminal() {
            let cursor_position = terminal.get_cursor_position().unwrap_or_default();
            let mut ui = Self {
                app_data,
                cursor_position,
                gui_state,
                input_poll_rate: std::time::Duration::from_millis(100),
                input_tx,
                is_running,
                now: Instant::now(),
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
        let stdout = Self::init_terminal()?;
        let backend = CrosstermBackend::new(stdout);
        Ok(Terminal::new(backend)?)
    }

    fn init_terminal() -> Result<Stdout> {
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen)?;
        Self::enable_mouse_capture()?;
        Ok(stdout)
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
        self.terminal.clear().ok();
        self.terminal.set_cursor_position(self.cursor_position)?;
        Ok(self.terminal.show_cursor()?)
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

    /// Use exeternal docker cli to exec into a container
    async fn exec(&mut self) {
        let exec_mode = self.gui_state.lock().get_exec_mode();

        if let Some(mode) = exec_mode {
            self.reset_terminal().ok();
            self.terminal.clear().ok();
            if let Err(e) = mode.run(TerminalSize::new(&self.terminal)).await {
                self.app_data
                    .lock()
                    .set_error(e, &self.gui_state, Status::Error);
            };
        }
        self.terminal.clear().ok();
        self.reset_terminal().ok();
        Self::init_terminal().ok();
        self.gui_state.lock().status_del(Status::Exec);
    }

    /// The loop for drawing the main UI to the terminal
    async fn gui_loop(&mut self) -> Result<(), AppError> {
        while self.is_running.load(Ordering::SeqCst) {
            let exec = self.gui_state.lock().status_contains(&[Status::Exec]);
            if exec {
                self.exec().await;
            }

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
                        if key.kind == event::KeyEventKind::Press {
                            self.input_tx
                                .send(InputMessages::ButtonPress((key.code, key.modifiers)))
                                .await
                                .ok();
                        }
                    } else if let Event::Mouse(m) = event {
                        match m.kind {
                            event::MouseEventKind::Down(_)
                            | event::MouseEventKind::ScrollDown
                            | event::MouseEventKind::ScrollUp => {
                                self.input_tx.send(InputMessages::MouseEvent(m)).await.ok();
                            }
                            _ => (),
                        }
                    } else if let Event::Resize(_, _) = event {
                        self.gui_state.lock().clear_area_map();
                        self.terminal.autoresize().ok();
                    }
                }
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
        Ok(())
    }
}

/// Frequent data required by multiple framde drawing functions, can reduce mutex reads by placing it all in here
/// TODO add more items to this, and split up into parts
#[derive(Debug, Clone)]
#[allow(clippy::struct_excessive_bools)]
pub struct FrameData {
    columns: Columns,
    container_title: String,
    delete_confirm: Option<ContainerId>,
    filter_by: FilterBy,
    filter_term: Option<String>,
    has_containers: bool,
    has_error: Option<AppError>,
    height: u16,
    help_visible: bool,
    info_text: Option<(String, Instant)>,
    init: bool,
    is_loading: bool,
    loading_icon: String,
    log_title: String,
    selected_panel: SelectablePanel,
    sorted_by: Option<(Header, SortedOrder)>,
}

impl From<(MutexGuard<'_, AppData>, MutexGuard<'_, GuiState>)> for FrameData {
    fn from(data: (MutexGuard<'_, AppData>, MutexGuard<'_, GuiState>)) -> Self {
        // set max height for container section, needs +5 to deal with docker commands list and borders
        let height = data.0.get_container_len();
        let height = if height < 12 {
            u16::try_from(height + 5).unwrap_or_default()
        } else {
            12
        };

        let (filter_by, filter_term) = data.0.get_filter();
        Self {
            columns: data.0.get_width(),
            container_title: data.0.get_container_title(),
            delete_confirm: data.1.get_delete_container(),
            filter_by,
            filter_term: filter_term.cloned(),
            has_containers: data.0.get_container_len() > 0,
            has_error: data.0.get_error(),
            height,
            help_visible: data.1.status_contains(&[Status::Help]),
            info_text: data.1.info_box_text.clone(),
            init: data.1.status_contains(&[Status::Init]),
            is_loading: data.1.is_loading(),
            loading_icon: data.1.get_loading().to_string(),
            log_title: data.0.get_log_title(),
            selected_panel: data.1.get_selected_panel(),
            sorted_by: data.0.get_sorted(),
        }
    }
}

/// Draw the main ui to a frame of the terminal
fn draw_frame(f: &mut Frame, app_data: &Arc<Mutex<AppData>>, gui_state: &Arc<Mutex<GuiState>>) {
    let fd = FrameData::from((app_data.lock(), gui_state.lock()));
    let contains_filter = gui_state.lock().status_contains(&[Status::Filter]);

    let whole_constraints = if contains_filter {
        vec![Constraint::Max(1), Constraint::Min(1), Constraint::Max(1)]
    } else {
        vec![Constraint::Max(1), Constraint::Min(1)]
    };

    let whole_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(whole_constraints)
        .split(f.area());

    // Split into 3, containers+controls, logs, then graphs
    let upper_main = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Max(fd.height), Constraint::Min(1)].as_ref())
        .split(whole_layout[1]);

    let top_split = if fd.has_containers {
        vec![Constraint::Percentage(90), Constraint::Percentage(10)]
    } else {
        vec![Constraint::Percentage(100)]
    };
    // Containers + docker commands
    let top_panel = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(top_split)
        .split(upper_main[0]);

    let lower_split = if fd.has_containers {
        vec![Constraint::Percentage(70), Constraint::Percentage(30)]
    } else {
        vec![Constraint::Percentage(100)]
    };

    // Split into 2, logs, and optional charts
    let lower_main = Layout::default()
        .direction(Direction::Vertical)
        .constraints(lower_split)
        .split(upper_main[1]);

    draw_blocks::containers(app_data, top_panel[0], f, &fd, gui_state);

    draw_blocks::logs(app_data, lower_main[0], f, &fd, gui_state);

    draw_blocks::heading_bar(whole_layout[0], f, &fd, gui_state);

    // Draw filter bar
    if let Some(rect) = whole_layout.get(2) {
        draw_blocks::filter_bar(*rect, f, &fd);
    }

    if let Some(id) = fd.delete_confirm.as_ref() {
        app_data.lock().get_container_name_by_id(id).map_or_else(
            || {
                // If a container is deleted outside of oxker but whilst the Delete Confirm dialog is open, it can get caught in kind of a dead lock situation
                // so if in that unique situation, just clear the delete_container id
                gui_state.lock().set_delete_container(None);
            },
            |name| {
                draw_blocks::delete_confirm(f, gui_state, name);
            },
        );
    }

    // only draw commands + charts if there are containers
    if let Some(rect) = top_panel.get(1) {
        draw_blocks::commands(app_data, *rect, f, &fd, gui_state);

        // Can calculate the max string length here, and then use that to keep the ports section as small as possible (+4 for some padding + border)
        let max_lens = app_data.lock().get_longest_port();
        let ports_len = u16::try_from(max_lens.0 + max_lens.1 + max_lens.2 + 2).unwrap_or(26);

        let lower = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Min(1), Constraint::Max(ports_len)])
            .split(lower_main[1]);

        draw_blocks::chart(f, lower[0], app_data);
        draw_blocks::ports(f, lower[1], app_data, max_lens);
    }

    if let Some((text, instant)) = fd.info_text {
        draw_blocks::info(f, text, instant, gui_state);
    }

    // Check if error, and show popup if so
    if fd.help_visible {
        draw_blocks::help_box(f);
    }

    if let Some(error) = fd.has_error {
        draw_blocks::error(f, error, None);
    }
}
