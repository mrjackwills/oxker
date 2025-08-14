use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, Event},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use parking_lot::Mutex;
use ratatui::{
    Frame, Terminal,
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Position},
};
use std::{
    collections::HashSet,
    io::{self, Stdout, Write},
    sync::{Arc, atomic::Ordering},
    time::Duration,
};
use std::{sync::atomic::AtomicBool, time::Instant};
use tokio::sync::mpsc::Sender;
use tracing::error;

mod color_match;
mod draw_blocks;
mod gui_state;
mod redraw;
pub use redraw::Rerender;

pub use self::color_match::*;
pub use self::gui_state::{DeleteButton, GuiState, SelectablePanel, Status};
use crate::{
    app_data::{
        AppData, Columns, ContainerId, ContainerPorts, CpuTuple, FilterBy, Header, MemTuple,
        SortedOrder, State,
    },
    app_error::AppError,
    config::{AppColors, Keymap},
    exec::TerminalSize,
    input_handler::InputMessages,
};

const POLL_RATE: Duration = std::time::Duration::from_millis(50);

// could have a render struct, which takes in poll rate, and docker

pub struct Ui {
    app_data: Arc<Mutex<AppData>>,
    cursor_position: Position,
    gui_state: Arc<Mutex<GuiState>>,
    input_tx: Sender<InputMessages>,
    is_running: Arc<AtomicBool>,
    now: Instant,
    redraw: Arc<Rerender>,
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
    pub async fn start(
        app_data: Arc<Mutex<AppData>>,
        gui_state: Arc<Mutex<GuiState>>,
        input_tx: Sender<InputMessages>,
        is_running: Arc<AtomicBool>,
        redraw: Arc<Rerender>,
    ) {
        match Self::setup_terminal() {
            Ok(mut terminal) => {
                let cursor_position = terminal.get_cursor_position().unwrap_or_default();
                let mut ui = Self {
                    app_data,
                    cursor_position,
                    gui_state,
                    input_tx,
                    is_running,
                    now: Instant::now(),
                    redraw,
                    terminal,
                };
                if let Err(e) = ui.draw_ui().await {
                    error!("{e}");
                }
                if let Err(e) = ui.reset_terminal() {
                    error!("{e}");
                }
            }
            _ => {
                error!("Terminal Error");
            }
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
        let colors = self.app_data.lock().config.app_colors;
        let keymap = self.app_data.lock().config.keymap.clone();
        let mut redraw = true;
        loop {
            if self.now.elapsed() >= std::time::Duration::from_secs(1) {
                seconds -= 1;
                self.now = Instant::now();
                redraw = true;
                if seconds < 1 {
                    break;
                }
            }

            if redraw
                && self
                    .terminal
                    .draw(|f| {
                        draw_blocks::error::draw(
                            colors,
                            &AppError::DockerConnect,
                            f,
                            &keymap,
                            Some(seconds),
                        );
                    })
                    .is_err()
            {
                return Err(AppError::Terminal);
            }
            redraw = false;
            std::thread::sleep(POLL_RATE);
        }
        Ok(())
    }

    /// Use external docker cli to exec into a container
    async fn exec(&mut self) {
        let exec_mode = self.gui_state.lock().get_exec_mode();

        if let Some(mode) = exec_mode {
            self.reset_terminal().ok();
            self.terminal.clear().ok();
            if let Err(e) = mode.run(TerminalSize::new(&self.terminal)).await {
                self.app_data
                    .lock()
                    .set_error(e, &self.gui_state, Status::Error);
            }
        }
        self.terminal.clear().ok();
        self.reset_terminal().ok();
        Self::init_terminal().ok();
        self.gui_state.lock().status_del(Status::Exec);
    }

    /// Use the previously redrawn time, the current time, the docker_interval, and the redraw struct, to calculate
    /// if the screen should be redrawn or not
    fn should_redraw(&self, previous: &mut Instant, docker_interval_ms: u128) -> bool {
        let result = self.redraw.swap() || previous.elapsed().as_millis() >= docker_interval_ms;
        if result {
            *previous = std::time::Instant::now();
        }
        result
    }

    /// The loop for drawing the main UI to the terminal
    async fn gui_loop(&mut self) -> Result<(), AppError> {
        let colors = self.app_data.lock().config.app_colors;
        let keymap = self.app_data.lock().config.keymap.clone();
        let docker_interval_ms = u128::from(self.app_data.lock().config.docker_interval_ms);
        let mut drawn_at = std::time::Instant::now();

        if let Ok(size) = self.terminal.size() {
            self.gui_state.lock().set_screen_width(size.width);
        }

        while self.is_running.load(Ordering::SeqCst) {
            if self.should_redraw(&mut drawn_at, docker_interval_ms) {
                let fd = FrameData::from(&*self);

                let exec = fd.status.contains(&Status::Exec);
                if exec {
                    self.exec().await;
                }

                if self
                    .terminal
                    .draw(|frame| {
                        draw_frame(&self.app_data, colors, &keymap, frame, &fd, &self.gui_state);
                    })
                    .is_err()
                {
                    return Err(AppError::Terminal);
                }
            }

            if crossterm::event::poll(POLL_RATE).unwrap_or(false) {
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
                    } else if let Event::Resize(width, _) = event {
                        self.gui_state.lock().clear_area_map();

                        // self.gui_state.lock().set_window_height(row);

                        self.terminal.autoresize().ok();
                        // todo set screen width
                        self.gui_state.lock().set_screen_width(width);
                    }
                }
            }
        }
        Ok(())
    }

    /// Draw either the Error, or main oxker ui, to the terminal
    async fn draw_ui(&mut self) -> Result<(), AppError> {
        let status = self.gui_state.lock().get_status();
        if status.contains(&Status::DockerConnect) {
            self.err_loop()?;
        } else {
            self.gui_loop().await?;
        }
        Ok(())
    }
}

/// Frequent data required by multiple frame drawing functions, can reduce mutex reads by placing it all in here
#[derive(Debug, Clone)]
#[allow(clippy::struct_excessive_bools)]
pub struct FrameData {
    chart_data: Option<(CpuTuple, MemTuple)>,
    color_logs: bool,
    columns: Columns,
    container_title: String,
    delete_confirm: Option<ContainerId>,
    filter_by: FilterBy,
    filter_term: Option<String>,
    has_containers: bool,
    log_height: u16,
    show_logs: bool,
    has_error: Option<AppError>,
    info_text: Option<(String, Instant)>,
    is_loading: bool,
    loading_icon: String,
    log_title: String,
    port_max_lens: (usize, usize, usize),
    ports: Option<(Vec<ContainerPorts>, State)>,
    selected_panel: SelectablePanel,
    scroll_title: Option<String>,
    sorted_by: Option<(Header, SortedOrder)>,
    status: HashSet<Status>,
}

impl From<&Ui> for FrameData {
    fn from(ui: &Ui) -> Self {
        let (app_data, gui_data) = (ui.app_data.lock(), ui.gui_state.lock());

        let (filter_by, filter_term) = app_data.get_filter();
        Self {
            chart_data: app_data.get_chart_data(),
            color_logs: app_data.config.color_logs,
            columns: app_data.get_width(),
            container_title: app_data.get_container_title(),
            delete_confirm: gui_data.get_delete_container(),
            filter_by,
            filter_term: filter_term.cloned(),
            has_containers: app_data.get_container_len() > 0,
            has_error: app_data.get_error(),
            info_text: gui_data.info_box_text.clone(),
            is_loading: gui_data.is_loading(),
            show_logs: gui_data.get_show_logs(),
            loading_icon: gui_data.get_loading().to_string(),
            log_height: gui_data.get_log_height(),
            log_title: app_data.get_log_title(),
            port_max_lens: app_data.get_longest_port(),
            ports: app_data.get_selected_ports(),
            scroll_title: app_data.get_scroll_title(),
            selected_panel: gui_data.get_selected_panel(),
            sorted_by: app_data.get_sorted(),
            status: gui_data.get_status(),
        }
    }
}

/// Draw the main ui to a frame of the terminal
fn draw_frame(
    app_data: &Arc<Mutex<AppData>>,
    colors: AppColors,
    keymap: &Keymap,
    f: &mut Frame,
    fd: &FrameData,
    gui_state: &Arc<Mutex<GuiState>>,
) {
    let whole_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(if fd.status.contains(&Status::Filter) {
            vec![Constraint::Max(1), Constraint::Min(1), Constraint::Max(1)]
        } else {
            vec![Constraint::Max(1), Constraint::Min(1)]
        })
        .split(f.area());

    draw_blocks::headers::draw(whole_layout[0], colors, f, fd, gui_state, keymap);

    // If required, draw filter bar
    if let Some(rect) = whole_layout.get(2) {
        draw_blocks::filter::draw(*rect, colors, f, fd);
    }

    let upper_main = Layout::default()
        .direction(Direction::Vertical)
        .constraints(if fd.has_containers {
            vec![Constraint::Percentage(75), Constraint::Percentage(25)]
        } else {
            vec![Constraint::Percentage(100), Constraint::Percentage(0)]
        })
        .split(whole_layout[1]);

    let containers_logs_section = Layout::default()
        .direction(Direction::Vertical)
        .constraints(if fd.show_logs {
            vec![Constraint::Min(6), Constraint::Percentage(fd.log_height)]
        } else {
            vec![Constraint::Percentage(100)]
        })
        .split(upper_main[0]);

    // Containers + docker commands
    let containers_commands = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(if fd.has_containers {
            vec![Constraint::Percentage(90), Constraint::Percentage(10)]
        } else {
            vec![Constraint::Percentage(100)]
        })
        .split(containers_logs_section[0]);

    draw_blocks::containers::draw(app_data, containers_commands[0], colors, f, fd, gui_state);

    if fd.show_logs {
        draw_blocks::logs::draw(
            app_data,
            containers_logs_section[1],
            colors,
            f,
            fd,
            gui_state,
        );
    }

    if let Some(id) = fd.delete_confirm.as_ref() {
        app_data.lock().get_container_name_by_id(id).map_or_else(
            || {
                // If a container is deleted outside of oxker but whilst the Delete Confirm dialog is open, it can get caught in kind of a dead lock situation
                // so if in that unique situation, just clear the delete_container id
                gui_state.lock().set_delete_container(None);
            },
            |name| {
                draw_blocks::delete_confirm::draw(colors, f, gui_state, keymap, name);
            },
        );
    }

    // only draw commands + charts if there are containers
    if let Some(rect) = containers_commands.get(1) {
        draw_blocks::commands::draw(app_data, *rect, colors, f, fd, gui_state);

        // Can calculate the max string length here, and then use that to keep the ports section as small as possible (+4 for some padding + border)
        let ports_len =
            u16::try_from(fd.port_max_lens.0 + fd.port_max_lens.1 + fd.port_max_lens.2 + 2)
                .unwrap_or(26);

        let lower = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Min(1), Constraint::Max(ports_len)])
            .split(upper_main[1]);

        draw_blocks::charts::draw(lower[0], colors, f, fd);
        draw_blocks::ports::draw(lower[1], colors, f, fd);
    }

    if let Some((text, instant)) = fd.info_text.as_ref() {
        draw_blocks::info::draw(colors, f, gui_state, instant, text.to_owned());
    }

    // Check if error, and show popup if so
    if fd.status.contains(&Status::Help) {
        let tz = app_data.lock().config.timezone.clone();
        draw_blocks::help::draw(
            colors,
            f,
            keymap,
            app_data.lock().config.show_timestamp,
            tz.as_ref(),
        );
    }

    if let Some(error) = fd.has_error.as_ref() {
        draw_blocks::error::draw(colors, error, f, keymap, None);
    }
}
