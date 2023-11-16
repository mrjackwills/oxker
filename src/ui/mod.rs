use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, Event},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use parking_lot::{Mutex, MutexGuard};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
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
    app_data::{AppData, Columns, ContainerId, Header, SortedOrder},
    app_error::AppError,
    input_handler::InputMessages,
};

pub const DOCKER_COMMAND: &str = "docker";

pub struct Ui {
    // args: CliArgs,
    app_data: Arc<Mutex<AppData>>,
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
    pub async fn create(
        app_data: Arc<Mutex<AppData>>,
        gui_state: Arc<Mutex<GuiState>>,
        is_running: Arc<AtomicBool>,
        sender: Sender<InputMessages>,
    ) {
        if let Ok(terminal) = Self::setup_terminal() {
            // let args = app_data.lock().args.clone();
            let mut ui = Self {
                app_data,
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
    fn exec(&mut self) {
        let id = self.app_data.lock().get_selected_container_id();

        if let Some(id) = id {
            // if Self::can_exec(&id).is_some() {
            if let Ok(mut child) = std::process::Command::new(DOCKER_COMMAND)
                .args(["exec", "-it", id.get(), "sh"])
                .stdin(std::process::Stdio::inherit())
                .stdout(std::process::Stdio::inherit())
                .stderr(std::process::Stdio::inherit())
                .spawn()
            {
                self.reset_terminal().ok();
                child.wait().ok();
                if child.kill().is_err() {
                    std::process::exit(1)
                }
            }
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
                self.exec();
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
                        self.sender
                            .send(InputMessages::ButtonPress((key.code, key.modifiers)))
                            .await
                            .ok();
                    } else if let Event::Mouse(m) = event {
                        match m.kind {
                            event::MouseEventKind::Down(_)
                            | event::MouseEventKind::ScrollDown
                            | event::MouseEventKind::ScrollUp => {
                                self.sender.send(InputMessages::MouseEvent(m)).await.ok();
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

// #[macro_export]
// /// This macro simplifies the definition and evaluation of variables by capturing and immediately evaluating an expression.
// macro_rules! value_capture {
//     ($name:ident, $lock_expr:expr) => {
//         let $name = || $lock_expr;
//         let $name = $name();
//     };
// }

#[cfg(not(debug_assertions))]
fn get_wholelayout(f: &Frame) -> std::rc::Rc<[ratatui::layout::Rect]> {
    Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Min(100)].as_ref())
        .split(f.size())
}

#[cfg(debug_assertions)]
fn get_wholelayout(f: &Frame) -> std::rc::Rc<[ratatui::layout::Rect]> {
    Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Min(1), Constraint::Min(100)].as_ref())
        .split(f.size())
}

/// Frequent data required by multiple framde drawing functions, can reduce mutex reads by placing it all in here
#[derive(Debug)]
pub struct FrameData {
    columns: Columns,
    delete_confirm: Option<ContainerId>,
    has_containers: bool,
    has_error: Option<AppError>,
    height: u16,
    help_visible: bool,
    init: bool,
    info_text: Option<String>,
    loading_icon: String,
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

        Self {
            columns: data.0.get_width(),
            delete_confirm: data.1.get_delete_container(),
            has_containers: data.0.get_container_len() > 1,
            has_error: data.0.get_error(),
            height,
            help_visible: data.1.status_contains(&[Status::Help]),
            init: data.1.status_contains(&[Status::Init]),
            info_text: data.1.info_box_text.clone(),
            loading_icon: data.1.get_loading().to_string(),
            selected_panel: data.1.get_selected_panel(),
            sorted_by: data.0.get_sorted(),
        }
    }
}

/// Draw the main ui to a frame of the terminal
fn draw_frame(f: &mut Frame, app_data: &Arc<Mutex<AppData>>, gui_state: &Arc<Mutex<GuiState>>) {
    let fd = FrameData::from((app_data.lock(), gui_state.lock()));

    let whole_layout = get_wholelayout(f);
    #[cfg(debug_assertions)]
    draw_blocks::debug_bar(whole_layout[0], f, app_data.lock().get_debug_string());

    #[cfg(debug_assertions)]
    let whole_layout_split = (1, 2);

    #[cfg(not(debug_assertions))]
    let whole_layout_split = (0, 1);

    // Split into 3, containers+controls, logs, then graphs
    let upper_main = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Max(fd.height), Constraint::Percentage(50)].as_ref())
        .split(whole_layout[whole_layout_split.1]);

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
        vec![Constraint::Percentage(75), Constraint::Percentage(25)]
    } else {
        vec![Constraint::Percentage(100)]
    };

    // Split into 2, logs, and optional charts
    let lower_main = Layout::default()
        .direction(Direction::Vertical)
        .constraints(lower_split)
        .split(upper_main[1]);

    draw_blocks::containers(app_data, top_panel[0], f, &fd, gui_state, &fd.columns);

    draw_blocks::logs(app_data, lower_main[0], f, &fd, gui_state);

    draw_blocks::heading_bar(whole_layout[whole_layout_split.0], f, &fd, gui_state);

    if let Some(id) = fd.delete_confirm.as_ref() {
        app_data.lock().get_container_name_by_id(id).map_or_else(
            || {
                // If a container is deleted outside of oxker but whilst the Delete Confirm dialog is open, it can get caught in kind of a dead lock situation
                // so if in that unique situation, just clear the delete_container id
                gui_state.lock().set_delete_container(None);
            },
            |name| {
                draw_blocks::delete_confirm(f, gui_state, &name);
            },
        );
    }

    // only draw commands + charts if there are containers
    if fd.has_containers {
        draw_blocks::commands(app_data, top_panel[1], f, &fd, gui_state);
        draw_blocks::chart(f, lower_main[1], app_data);
    }

    if let Some(info) = fd.info_text {
        draw_blocks::info(f, &info);
    }

    // Check if error, and show popup if so
    if fd.help_visible {
        draw_blocks::help_box(f);
    }

    if let Some(error) = fd.has_error {
        draw_blocks::error(f, error, None);
    }
}
