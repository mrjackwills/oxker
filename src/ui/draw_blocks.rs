use parking_lot::Mutex;
use std::default::Default;
use std::{fmt::Display, sync::Arc};
use tui::{
    backend::Backend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    symbols,
    text::{Span, Spans},
    widgets::{
        Axis, Block, BorderType, Borders, Chart, Clear, Dataset, GraphType, List, ListItem,
        Paragraph,
    },
    Frame,
};

use crate::{
    app_data::{AppData, ByteStats, Columns, CpuStats, State, Stats},
    app_error::AppError,
};

use super::gui_state::BoxLocation;
use super::{GuiState, SelectablePanel};

const NAME_TEXT: &str = r#"
                          88                               
                          88                               
                          88                               
 ,adPPYba,   8b,     ,d8  88   ,d8    ,adPPYba,  8b,dPPYba,
a8"     "8a   `Y8, ,8P'   88 ,a8"    a8P_____88  88P'   "Y8
8b       d8     )888(     8888[      8PP"""""""  88        
"8a,   ,a8"   ,d8" "8b,   88`"Yba,   "8b,   ,aa  88        
 `"YbbdP"'   8P'     `Y8  88   `Y8a   `"Ybbd8"'  88        "#;

const NAME: &str = env!("CARGO_PKG_NAME");
const VERSION: &str = env!("CARGO_PKG_VERSION");
const REPO: &str = env!("CARGO_PKG_REPOSITORY");
const DESCRIPTION: &str = env!("CARGO_PKG_DESCRIPTION");
const ORANGE: Color = Color::Rgb(255, 178, 36);
const MARGIN: &str = "   ";

/// Generate block, add a border if is the selected panel,
/// add custom title based on state of each panel
fn generate_block<'a>(
    app_data: &Arc<Mutex<AppData>>,
    area: Rect,
    gui_state: &Arc<Mutex<GuiState>>,
    panel: SelectablePanel,
) -> Block<'a> {
    gui_state.lock().insert_into_area_map(panel, area);
    let mut block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded);
    let current_selected_panel = gui_state.lock().selected_panel;
    let title = match panel {
        SelectablePanel::Containers => {
            format!(
                " {} {} ",
                panel.title(),
                app_data.lock().containers.get_state_title()
            )
        }
        SelectablePanel::Logs => {
            format!(" {} {} ", panel.title(), app_data.lock().get_log_title())
        }
        _ => String::from(""),
    };
    block = block.title(title);
    if current_selected_panel == panel {
        block = block.border_style(Style::default().fg(Color::LightCyan));
    }
    block
}

/// Draw the command panel
pub fn draw_commands<B: Backend>(
    app_data: &Arc<Mutex<AppData>>,
    area: Rect,
    f: &mut Frame<'_, B>,
    gui_state: &Arc<Mutex<GuiState>>,
    index: Option<usize>,
) {
    let block = generate_block(app_data, area, gui_state, SelectablePanel::Commands);
    if let Some(i) = index {
        let items = app_data.lock().containers.items[i]
            .docker_controls
            .items
            .iter()
            .map(|i| {
                let lines = Spans::from(vec![Span::styled(
                    i.to_string(),
                    Style::default().fg(i.get_color()),
                )]);
                ListItem::new(lines)
            })
            .collect::<Vec<_>>();

        let items = List::new(items)
            .block(block)
            .highlight_style(Style::default().add_modifier(Modifier::BOLD))
            .highlight_symbol("▶ ");

        f.render_stateful_widget(
            items,
            area,
            &mut app_data.lock().containers.items[i].docker_controls.state,
        );
    } else {
        let debug_text = String::from("");
        let paragraph = Paragraph::new(debug_text)
            .block(block)
            .alignment(Alignment::Center);
        f.render_widget(paragraph, area)
    }
}

/// Draw the containers panel
pub fn draw_containers<B: Backend>(
    app_data: &Arc<Mutex<AppData>>,
    area: Rect,
    f: &mut Frame<'_, B>,
    gui_state: &Arc<Mutex<GuiState>>,
    widths: &Columns,
) {
    let block = generate_block(app_data, area, gui_state, SelectablePanel::Containers);

    let items = app_data
        .lock()
        .containers
        .items
        .iter()
        .map(|i| {
            let state_style = Style::default().fg(i.state.get_color());
            let blue = Style::default().fg(Color::Blue);

            let mems = format!(
                "{:>1} / {:>1}",
                i.mem_stats.back().unwrap_or(&ByteStats::new(0)),
                i.mem_limit
            );

            let lines = Spans::from(vec![
                Span::styled(
                    format!("{:<width$}", i.state.to_string(), width = widths.state.1),
                    state_style,
                ),
                Span::styled(
                    format!("{}{:>width$}", MARGIN, i.status, width = widths.status.1),
                    state_style,
                ),
                Span::styled(
                    format!(
                        "{}{:>width$}",
                        MARGIN,
                        i.cpu_stats.back().unwrap_or(&CpuStats::new(0.0)),
                        width = widths.cpu.1
                    ),
                    state_style,
                ),
                Span::styled(
                    format!("{}{:>width$}", MARGIN, mems, width = widths.mem.1),
                    state_style,
                ),
                Span::styled(
                    format!(
                        "{}{:>width$}",
                        MARGIN,
                        i.id.chars().take(8).collect::<String>(),
                        width = widths.id.1
                    ),
                    blue,
                ),
                Span::styled(
                    format!("{}{:>width$}", MARGIN, i.name, width = widths.name.1),
                    blue,
                ),
                Span::styled(
                    format!("{}{:>width$}", MARGIN, i.image, width = widths.image.1),
                    blue,
                ),
                Span::styled(
                    format!("{}{:>width$}", MARGIN, i.net_rx, width = widths.net_rx.1),
                    Style::default().fg(Color::Rgb(255, 233, 193)),
                ),
                Span::styled(
                    format!("{}{:>width$}", MARGIN, i.net_tx, width = widths.net_tx.1),
                    Style::default().fg(Color::Rgb(205, 140, 140)),
                ),
            ]);
            ListItem::new(lines)
        })
        .collect::<Vec<_>>();

    if items.is_empty() {
        let debug_text = String::from("no containers running");
        let paragraph = Paragraph::new(debug_text)
            .block(block)
            .alignment(Alignment::Center);
        f.render_widget(paragraph, area)
    } else {
        let items = List::new(items)
            .block(block)
            .highlight_style(Style::default().add_modifier(Modifier::BOLD))
            .highlight_symbol("⚪ ");

        f.render_stateful_widget(items, area, &mut app_data.lock().containers.state);
    }
}

/// Draw the logs panel
pub fn draw_logs<B: Backend>(
    app_data: &Arc<Mutex<AppData>>,
    area: Rect,
    f: &mut Frame<'_, B>,
    gui_state: &Arc<Mutex<GuiState>>,
    index: Option<usize>,
    loading_icon: String,
) {
    let block = generate_block(app_data, area, gui_state, SelectablePanel::Logs);

    let init = app_data.lock().init;
    if !init {
        let parsing_logs = format!("parsing logs {}", loading_icon);
        let paragraph = Paragraph::new(parsing_logs)
            .style(Style::default())
            .block(block)
            .alignment(Alignment::Center);
        f.render_widget(paragraph, area)
    } else if let Some(index) = index {
        let items = app_data.lock().containers.items[index]
            .logs
            .items
            .iter()
            .enumerate()
            .map(|i| i.1.to_owned())
            .collect::<Vec<_>>();

        let items = List::new(items)
            .block(block)
            .highlight_symbol("▶ ")
            .highlight_style(Style::default().add_modifier(Modifier::BOLD));
        f.render_stateful_widget(
            items,
            area,
            &mut app_data.lock().containers.items[index].logs.state,
        );
    } else {
        let debug_text = String::from("no logs found");
        let paragraph = Paragraph::new(debug_text)
            .block(block)
            .alignment(Alignment::Center);
        f.render_widget(paragraph, area)
    }
}

/// Draw the cpu + mem charts
pub fn draw_chart<B: Backend>(
    f: &mut Frame<'_, B>,
    area: Rect,
    app_data: &Arc<Mutex<AppData>>,
    index: Option<usize>,
) {
    if let Some(index) = index {
        let area = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
            .split(area);

        // Check is some, else can cause out of bounds error, if containers get removed before a docker update
        if let Some(data) = app_data.lock().containers.items.get(index) {
            let (cpu, mem) = data.get_chart_data();
            let cpu_dataset = vec![Dataset::default()
                .marker(symbols::Marker::Dot)
                .style(Style::default().fg(Color::Magenta))
                .graph_type(GraphType::Line)
                .data(&cpu.0)];
            let mem_dataset = vec![Dataset::default()
                .marker(symbols::Marker::Dot)
                .style(Style::default().fg(Color::Cyan))
                .graph_type(GraphType::Line)
                .data(&mem.0)];
            let cpu_chart = make_chart(
                cpu.2,
                String::from("cpu"),
                cpu_dataset,
                CpuStats::new(cpu.0.last().unwrap_or(&(0.00, 0.00)).1),
                cpu.1,
            );
            let mem_chart = make_chart(
                mem.2,
                String::from("memory"),
                mem_dataset,
                ByteStats::new(mem.0.last().unwrap_or(&(0.0, 0.0)).1 as u64),
                mem.1,
            );

            f.render_widget(cpu_chart, area[0]);
            f.render_widget(mem_chart, area[1]);
        }
    }
}

/// Create charts
fn make_chart<T: Stats + Display>(
    state: State,
    name: String,
    dataset: Vec<Dataset>,
    current: T,
    max: T,
) -> Chart {
    let title_color = match state {
        State::Running => Color::Green,
        _ => state.get_color(),
    };
    let label_color = match state {
        State::Running => ORANGE,
        _ => state.get_color(),
    };
    Chart::new(dataset)
        .block(
            Block::default()
                .title_alignment(Alignment::Center)
                .title(Span::styled(
                    format!(" {} {} ", name, current),
                    Style::default()
                        .fg(title_color)
                        .add_modifier(Modifier::BOLD),
                ))
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded),
        )
        .x_axis(
            Axis::default()
                .style(Style::default().fg(title_color))
                .bounds([0.00, 60.0]),
        )
        .y_axis(
            Axis::default()
                .labels(vec![
                    Span::styled("", Style::default().fg(label_color)),
                    Span::styled(
                        format!("{}", max),
                        Style::default()
                            .add_modifier(Modifier::BOLD)
                            .fg(label_color),
                    ),
                ])
                // Add 0.01, so that max point is always visible?
                .bounds([0.0, max.get_value() + 0.01]),
        )
}

/// Draw heading bar at top  of program, always visible
pub fn draw_heading_bar<B: Backend>(
    area: Rect,
    columns: &Columns,
    f: &mut Frame<'_, B>,
    has_containers: bool,
    loading_icon: String,
    info_visible: bool,
) {
    let block = || Block::default().style(Style::default().bg(Color::Magenta).fg(Color::Black));

    f.render_widget(block(), area);

    let mut column_headings = format!(
        " {}{:>width$}",
        loading_icon,
        columns.state.0,
        width = columns.state.1
    );
    column_headings.push_str(
        format!(
            "{}  {:>width$}",
            MARGIN,
            columns.status.0,
            width = columns.status.1
        )
        .as_str(),
    );
    column_headings
        .push_str(format!("{}{:>width$}", MARGIN, columns.cpu.0, width = columns.cpu.1).as_str());
    column_headings
        .push_str(format!("{}{:>width$}", MARGIN, columns.mem.0, width = columns.mem.1).as_str());
    column_headings
        .push_str(format!("{}{:>width$}", MARGIN, columns.id.0, width = columns.id.1).as_str());
    column_headings.push_str(
        format!(
            "{}{:>width$}",
            MARGIN,
            columns.name.0,
            width = columns.name.1
        )
        .as_str(),
    );
    column_headings.push_str(
        format!(
            "{}{:>width$}",
            MARGIN,
            columns.image.0,
            width = columns.image.1
        )
        .as_str(),
    );
    column_headings.push_str(
        format!(
            "{}{:>width$}",
            MARGIN,
            columns.net_rx.0,
            width = columns.net_rx.1
        )
        .as_str(),
    );
    column_headings.push_str(
        format!(
            "{}{:>width$}",
            MARGIN,
            columns.net_tx.0,
            width = columns.net_tx.1
        )
        .as_str(),
    );

    let suffix = if info_visible { "exit" } else { "show" };
    let info_text = format!("( h ) to {} help {}", suffix, MARGIN);
    let info_width = info_text.chars().count();

    let column_width = column_headings.chars().count();

    let splits = if has_containers {
        vec![
            Constraint::Min(column_width as u16),
            Constraint::Min(info_width as u16),
        ]
    } else {
        vec![Constraint::Percentage(100)]
    };

    let split_bar = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(splits.as_ref())
        .split(area);

    if has_containers {
        let paragraph = Paragraph::new(column_headings)
            .block(block())
            .alignment(Alignment::Left);
        f.render_widget(paragraph, split_bar[0]);
    }

    let paragraph = Paragraph::new(info_text)
        .block(block())
        .alignment(Alignment::Right);

    let index = if has_containers { 1 } else { 0 };
    f.render_widget(paragraph, split_bar[index]);
}

/// From a given &String, return the maximum number of chars on a single line
fn max_line_width(text: &str) -> usize {
    let mut max_line_width = 0;
    text.lines().into_iter().for_each(|line| {
        let width = line.chars().count();
        if width > max_line_width {
            max_line_width = width;
        }
    });
    max_line_width
}

/// Draw the help box in the centre of the screen
pub fn draw_help_box<B: Backend>(f: &mut Frame<'_, B>) {
    let title = format!(" {} ", VERSION);

    let description_text = format!("\n{}", DESCRIPTION);

    let mut help_text = String::from("\n  ( tab )  or ( shift+tab ) to change panels");
    help_text
        .push_str("\n  ( ↑ ↓ ) or ( j k ) or (PgUp PgDown) or (Home End) to change selected line");
    help_text.push_str("\n  ( enter ) to send docker container commands");
    help_text.push_str("\n  ( h ) to toggle this help information");
    help_text.push_str(
        "\n  ( m ) to toggle mouse capture - if disabled, text on screen can be selected & copied",
    );
    help_text.push_str("\n  ( q ) to quit at any time");
    help_text.push_str("\n  mouse scrolling & clicking also available");
    help_text.push_str("\n\n  currenty an early work in progress, all and any input appreciated");
    help_text.push_str(format!("\n  {}", REPO.trim()).as_str());

    // Find the maximum line widths & height
    let all_text = format!("{}{}{}", NAME_TEXT, description_text, help_text);
    let mut max_line_width = max_line_width(&all_text);
    let mut lines = all_text.lines().count();

    // Add some vertical and horizontal padding to the info box
    lines += 3;
    max_line_width += 4;

    let name_paragraph = Paragraph::new(NAME_TEXT)
        .style(Style::default().bg(Color::Magenta).fg(Color::White))
        .block(Block::default())
        .alignment(Alignment::Center);

    let description_paragrpah = Paragraph::new(description_text.as_str())
        .style(Style::default().bg(Color::Magenta).fg(Color::Black))
        .block(Block::default())
        .alignment(Alignment::Center);

    let help_paragraph = Paragraph::new(help_text.as_str())
        .style(Style::default().bg(Color::Magenta).fg(Color::Black))
        .block(Block::default())
        .alignment(Alignment::Left);

    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::Black));

    let area = draw_popup(
        lines as u16,
        max_line_width as u16,
        f.size(),
        BoxLocation::MiddleCentre,
    );

    let split_popup = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Max(NAME_TEXT.lines().count() as u16),
                Constraint::Max(description_text.lines().count() as u16),
                Constraint::Max(help_text.lines().count() as u16),
            ]
            .as_ref(),
        )
        .split(area);

    // Order is important here
    f.render_widget(Clear, area);
    f.render_widget(name_paragraph, split_popup[0]);
    f.render_widget(description_paragrpah, split_popup[1]);
    f.render_widget(help_paragraph, split_popup[2]);
    f.render_widget(block, area);
}

/// Draw an error popup over whole screen
pub fn draw_error<B: Backend>(f: &mut Frame<'_, B>, error: AppError, seconds: Option<u8>) {
    let block = Block::default()
        .title(" Error ")
        .border_type(BorderType::Rounded)
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL);

    let to_push = match error {
        AppError::DockerConnect => {
            format!(
                "\n\n {}::v{} closing in {:02} seconds",
                NAME,
                VERSION,
                seconds.unwrap_or(5)
            )
        }
        _ => String::from("\n\n ( c ) to clear error\n ( q ) to quit oxker"),
    };

    let mut text = format!("\n{}", error);

    text.push_str(to_push.as_str());

    // Find the maximum line width & height
    let mut max_line_width = max_line_width(&text);
    let mut lines = text.lines().count();

    // Add some horizontal & vertical margins
    max_line_width += 8;
    lines += 3;

    let paragraph = Paragraph::new(text)
        .style(Style::default().bg(Color::Red).fg(Color::White))
        .block(block)
        .alignment(Alignment::Center);

    let area = draw_popup(
        lines as u16,
        max_line_width as u16,
        f.size(),
        BoxLocation::MiddleCentre,
    );
    f.render_widget(Clear, area);
    f.render_widget(paragraph, area);
}

/// Draw info box in one of the 9 BoxLocations
pub fn draw_info<B: Backend>(f: &mut Frame<'_, B>, text: String) {
    let block = Block::default()
        .title("")
        .title_alignment(Alignment::Center)
        .borders(Borders::NONE);

    let mut max_line_width = max_line_width(&text);
    let mut lines = text.lines().count();

    // Add some horizontal & vertical margins
    max_line_width += 8;
    lines += 2;

    let paragraph = Paragraph::new(text)
        .style(Style::default().bg(Color::Blue).fg(Color::White))
        .block(block)
        .alignment(Alignment::Center);

    let area = draw_popup(
        lines as u16,
        max_line_width as u16,
        f.size(),
        BoxLocation::BottomRight,
    );
    f.render_widget(Clear, area);
    f.render_widget(paragraph, area);
}

/// draw a box in the one of the BoxLocations, based on max line width + number of lines
fn draw_popup(text_lines: u16, text_width: u16, r: Rect, box_location: BoxLocation) -> Rect {
    // Make sure blank_space can't be an negative, as will crash
    let blank_vertical = if r.height > text_lines {
        (r.height - text_lines) / 2
    } else {
        1
    };
    let blank_horizontal = if r.width > text_width {
        (r.width - text_width) / 2
    } else {
        1
    };

    let v_constraints = box_location.get_vertical_constraints(blank_vertical, text_lines);
    let h_constraints = box_location.get_horizontal_constraints(blank_horizontal, text_width);

    let indexes = box_location.get_indexes();

    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(v_constraints)
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints(h_constraints)
        .split(popup_layout[indexes.0])[indexes.1]
}
