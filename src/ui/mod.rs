use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use parking_lot::Mutex;
use std::{
    io::{self, Write},
    sync::{atomic::Ordering, Arc},
};
use std::{sync::atomic::AtomicBool, time::Instant};
use tokio::sync::mpsc::Sender;
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

/// Take control of the terminal in order to draw gui
pub async fn create_ui(
    app_data: Arc<Mutex<AppData>>,
    docker_sx: Sender<DockerMessage>,
    gui_state: Arc<Mutex<GuiState>>,
    is_running: Arc<AtomicBool>,
    sender: Sender<InputMessages>,
) -> Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let res = run_app(
        app_data,
        docker_sx,
        gui_state,
        is_running,
        sender,
        &mut terminal,
    )
    .await;
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{err}");
    }
	std::io::stdout().flush().unwrap_or(());
    Ok(())
}

/// Run a loop to draw the gui
async fn run_app<B: Backend + Send>(
    app_data: Arc<Mutex<AppData>>,
    docker_sx: Sender<DockerMessage>,
    gui_state: Arc<Mutex<GuiState>>,
    is_running: Arc<AtomicBool>,
    sender: Sender<InputMessages>,
    terminal: &mut Terminal<B>,
) -> Result<(), AppError> {
    let update_duration =
        std::time::Duration::from_millis(u64::from(app_data.lock().args.docker_interval));
    let input_poll_rate = std::time::Duration::from_millis(75);
    let status_dockerconnect = gui_state.lock().status_contains(&[Status::DockerConnect]);
    let mut now = Instant::now();
    if status_dockerconnect {
        let mut seconds = 5;
        loop {
            if seconds < 1 {
                break;
            }
            if now.elapsed() >= std::time::Duration::from_secs(1) {
                seconds -= 1;
                now = Instant::now();
            }
            if terminal
                .draw(|f| draw_blocks::error(f, AppError::DockerConnect, Some(seconds)))
                .is_err()
            {
                return Err(AppError::Terminal);
            }
        }
    } else {
        while is_running.load(Ordering::Relaxed) {
            if crossterm::event::poll(input_poll_rate).unwrap_or(false) {
                if let Ok(event) = event::read() {
                    if let Event::Key(key) = event {
                        sender
                            .send(InputMessages::ButtonPress(key.code))
                            .await
                            .unwrap_or(());
                    } else if let Event::Mouse(m) = event {
                        sender
                            .send(InputMessages::MouseEvent(m))
                            .await
                            .unwrap_or(());
                    } else if let Event::Resize(_, _) = event {
                        gui_state.lock().clear_area_map();
                        terminal.autoresize().unwrap_or(());
                    }
                }
            }

            if now.elapsed() >= update_duration {
                docker_sx.send(DockerMessage::Update).await.unwrap_or(());
                now = Instant::now();
            }
            if terminal.draw(|f| ui(f, &app_data, &gui_state)).is_err() {
                return Err(AppError::Terminal);
            }
        }
    }
    terminal.clear().unwrap_or(());
    Ok(())
}

fn ui<B: Backend>(
    f: &mut Frame<'_, B>,
    app_data: &Arc<Mutex<AppData>>,
    gui_state: &Arc<Mutex<GuiState>>,
) {
    // set max height for container section, needs +4 to deal with docker commands list and borders
    let height = app_data.lock().get_container_len();
    let height = if height < 12 { height + 4 } else { 12 };

    let column_widths = app_data.lock().get_width();
    let has_containers = !app_data.lock().containers.items.is_empty();
    let has_error = app_data.lock().get_error();
    let log_index = app_data.lock().get_selected_log_index();
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

    // Split into 3, containers+controls, logs, then graphs
    let lower_main = Layout::default()
        .direction(Direction::Vertical)
        .constraints(lower_split.as_ref())
        .split(upper_main[1]);

    draw_blocks::containers(app_data, top_panel[0], f, gui_state, &column_widths);

    if has_containers {
        draw_blocks::commands(app_data, top_panel[1], f, gui_state, log_index);
    }

    draw_blocks::logs(
        app_data,
        lower_main[0],
        f,
        gui_state,
        log_index,
        &loading_icon,
    );

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
        draw_blocks::chart(f, lower_main[1], app_data, log_index);
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
