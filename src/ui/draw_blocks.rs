use parking_lot::Mutex;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style, Stylize},
    symbols,
    text::{Line, Span},
    widgets::{
        Axis, Block, BorderType, Borders, Chart, Clear, Dataset, GraphType, List, ListItem,
        Paragraph,
    },
    Frame,
};
use std::{default::Default, time::Instant};
use std::{fmt::Display, sync::Arc};

use crate::app_data::{ContainerItem, ContainerName, Header, SortedOrder};
use crate::{
    app_data::{AppData, ByteStats, Columns, CpuStats, State, Stats},
    app_error::AppError,
};

use super::{
    gui_state::{BoxLocation, DeleteButton, Region},
    FrameData,
};
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
const RIGHT_ARROW: &str = "▶ ";
const CIRCLE: &str = "⚪ ";

const CONSTRAINT_50_50: [Constraint; 2] = [Constraint::Percentage(50), Constraint::Percentage(50)];
const CONSTRAINT_100: [Constraint; 1] = [Constraint::Percentage(100)];
const CONSTRAINT_POPUP: [Constraint; 5] = [
    Constraint::Min(2),
    Constraint::Max(1),
    Constraint::Max(1),
    Constraint::Max(3),
    Constraint::Min(1),
];

const CONSTRAINT_BUTTONS: [Constraint; 5] = [
    Constraint::Percentage(10),
    Constraint::Percentage(35),
    Constraint::Percentage(10),
    Constraint::Percentage(35),
    Constraint::Percentage(10),
];

/// From a given &str, return the maximum number of chars on a single line
fn max_line_width(text: &str) -> usize {
    text.lines()
        .map(|i| i.chars().count())
        .max()
        .unwrap_or_default()
}

/// Generate block, add a border if is the selected panel,
/// add custom title based on state of each panel
fn generate_block<'a>(
    app_data: &Arc<Mutex<AppData>>,
    area: Rect,
    fd: &FrameData,
    gui_state: &Arc<Mutex<GuiState>>,
    panel: SelectablePanel,
) -> Block<'a> {
    gui_state
        .lock()
        .update_region_map(Region::Panel(panel), area);
    let mut title = match panel {
        SelectablePanel::Containers => {
            format!("{}{}", panel.title(), app_data.lock().container_title())
        }
        SelectablePanel::Logs => {
            format!("{}{}", panel.title(), app_data.lock().get_log_title())
        }
        SelectablePanel::Commands => String::new(),
    };
    if !title.is_empty() {
        title = format!(" {title} ");
    }
    let mut block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .title(title);
    if fd.selected_panel == panel {
        block = block.border_style(Style::default().fg(Color::LightCyan));
    }
    block
}

/// Draw the command panel
pub fn commands(
    app_data: &Arc<Mutex<AppData>>,
    area: Rect,
    f: &mut Frame,
    fd: &FrameData,
    gui_state: &Arc<Mutex<GuiState>>,
) {
    let block = generate_block(app_data, area, fd, gui_state, SelectablePanel::Commands);
    let items = app_data.lock().get_control_items().map_or(vec![], |i| {
        i.iter()
            .map(|c| {
                let lines = Line::from(vec![Span::styled(
                    c.to_string(),
                    Style::default().fg(c.get_color()),
                )]);
                ListItem::new(lines)
            })
            .collect::<Vec<_>>()
    });

    if let Some(i) = app_data.lock().get_control_state() {
        let items = List::new(items)
            .block(block)
            .highlight_style(Style::default().add_modifier(Modifier::BOLD))
            .highlight_symbol(RIGHT_ARROW);
        f.render_stateful_widget(items, area, i);
    } else {
        let paragraph = Paragraph::new("").block(block).alignment(Alignment::Center);
        f.render_widget(paragraph, area);
    }
}

/// Format the container data to display nicely on the screen
fn format_containers<'a>(i: &ContainerItem, widths: &Columns) -> Line<'a> {
    let state_style = Style::default().fg(i.state.get_color());
    let blue = Style::default().fg(Color::Blue);

    // Truncate?
    Line::from(vec![
        Span::styled(
            format!(
                "{:>width$}",
                i.name.to_string(),
                width = widths.name.1.into()
            ),
            blue,
        ),
        Span::styled(
            format!(
                "{MARGIN}{:<width$}",
                i.state.to_string(),
                width = widths.state.1.into()
            ),
            state_style,
        ),
        Span::styled(
            format!(
                "{MARGIN}{:>width$}",
                i.status,
                width = &widths.status.1.into()
            ),
            state_style,
        ),
        Span::styled(
            format!(
                "{}{:>width$}",
                MARGIN,
                i.cpu_stats.back().unwrap_or(&CpuStats::default()),
                width = &widths.cpu.1.into()
            ),
            state_style,
        ),
        Span::styled(
            format!(
                "{MARGIN}{:>width_current$} / {:>width_limit$}",
                i.mem_stats.back().unwrap_or(&ByteStats::default()),
                i.mem_limit,
                width_current = &widths.mem.1.into(),
                width_limit = &widths.mem.2.into()
            ),
            state_style,
        ),
        Span::styled(
            format!(
                "{}{:>width$}",
                MARGIN,
                i.id.get_short(),
                width = &widths.id.1.into()
            ),
            blue,
        ),
        Span::styled(
            format!(
                "{MARGIN}{:>width$}",
                i.image.to_string(),
                width = widths.image.1.into()
            ),
            blue,
        ),
        Span::styled(
            format!("{MARGIN}{:>width$}", i.rx, width = widths.net_rx.1.into()),
            Style::default().fg(Color::Rgb(255, 233, 193)),
        ),
        Span::styled(
            format!("{MARGIN}{:>width$}", i.tx, width = widths.net_tx.1.into()),
            Style::default().fg(Color::Rgb(205, 140, 140)),
        ),
    ])
}

/// Draw the containers panel
pub fn containers(
    app_data: &Arc<Mutex<AppData>>,
    area: Rect,
    f: &mut Frame,
    fd: &FrameData,
    gui_state: &Arc<Mutex<GuiState>>,
) {
    let block = generate_block(app_data, area, fd, gui_state, SelectablePanel::Containers);

    let items = app_data
        .lock()
        .get_container_items()
        .iter()
        .map(|i| ListItem::new(format_containers(i, &fd.columns)))
        .collect::<Vec<_>>();

    if items.is_empty() {
        let paragraph = Paragraph::new("no containers running")
            .block(block)
            .alignment(Alignment::Center);
        f.render_widget(paragraph, area);
    } else {
        let items = List::new(items)
            .block(block)
            .highlight_style(Style::default().add_modifier(Modifier::BOLD))
            .highlight_symbol(CIRCLE);
        f.render_stateful_widget(items, area, app_data.lock().get_container_state());
    }
}

/// Draw the logs panel
pub fn logs(
    app_data: &Arc<Mutex<AppData>>,
    area: Rect,
    f: &mut Frame,
    fd: &FrameData,
    gui_state: &Arc<Mutex<GuiState>>,
) {
    let block = generate_block(app_data, area, fd, gui_state, SelectablePanel::Logs);
    if fd.init {
        let paragraph = Paragraph::new(format!("parsing logs {}", fd.loading_icon))
            .style(Style::default())
            .block(block)
            .alignment(Alignment::Center);
        f.render_widget(paragraph, area);
    } else {
        let logs = app_data.lock().get_logs();

        if logs.is_empty() {
            let paragraph = Paragraph::new("no logs found")
                .block(block)
                .alignment(Alignment::Center);
            f.render_widget(paragraph, area);
        } else {
            let items = List::new(logs)
                .block(block)
                .highlight_symbol(RIGHT_ARROW)
                .highlight_style(Style::default().add_modifier(Modifier::BOLD));
            // This should always return Some, as logs is not empty
            if let Some(log_state) = app_data.lock().get_log_state() {
                f.render_stateful_widget(items, area, log_state);
            }
        }
    }
}

// Display the ports in a formatted list
pub fn ports(
    f: &mut Frame,
    area: Rect,
    app_data: &Arc<Mutex<AppData>>,
    max_lens: (usize, usize, usize),
) {
    if let Some(ports) = app_data.lock().get_selected_ports() {
        let block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .title_alignment(Alignment::Center)
            .title(Span::styled(
                " ports ",
                Style::default()
                    .fg(ports.1.get_color())
                    .add_modifier(Modifier::BOLD),
            ));

        let (ip, private, public) = max_lens;

        if ports.0.is_empty() {
            let text = match ports.1 {
                State::Running | State::Paused | State::Restarting => "no ports",
                _ => "",
            };
            let paragraph = Paragraph::new(Span::from(text).add_modifier(Modifier::BOLD))
                .alignment(Alignment::Center)
                .block(block);
            f.render_widget(paragraph, area);
        } else {
            let mut output = vec![Line::from(
                Span::from(format!(
                    "{:>ip$}{:>private$}{:>public$}",
                    "ip", "private", "public"
                ))
                .fg(Color::Yellow),
            )];
            for item in &ports.0 {
                let fg = Color::White;
                let strings = item.print();

                let line = vec![
                    Span::from(format!("{:>ip$}", strings.0)).fg(fg),
                    Span::from(format!("{:>private$}", strings.1)).fg(fg),
                    Span::from(format!("{:>public$}", strings.2)).fg(fg),
                ];
                output.push(Line::from(line));
            }
            let paragraph = Paragraph::new(output).block(block);
            f.render_widget(paragraph, area);
        }
    }
}

/// Draw the cpu + mem charts
pub fn chart(f: &mut Frame, area: Rect, app_data: &Arc<Mutex<AppData>>) {
    if let Some((cpu, mem)) = app_data.lock().get_chart_data() {
        let area = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(CONSTRAINT_50_50)
            .split(area);

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

        let cpu_stats = CpuStats::new(cpu.0.last().map_or(0.00, |f| f.1));
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let mem_stats = ByteStats::new(mem.0.last().map_or(0, |f| f.1 as u64));
        let cpu_chart = make_chart(cpu.2, "cpu", cpu_dataset, &cpu_stats, &cpu.1);
        let mem_chart = make_chart(mem.2, "memory", mem_dataset, &mem_stats, &mem.1);

        f.render_widget(cpu_chart, area[0]);
        f.render_widget(mem_chart, area[1]);
    }
}

/// Create charts
fn make_chart<'a, T: Stats + Display>(
    state: State,
    name: &'a str,
    dataset: Vec<Dataset<'a>>,
    current: &'a T,
    max: &'a T,
) -> Chart<'a> {
    let title_color = state.get_color();
    let label_color = match state {
        State::Running => ORANGE,
        _ => state.get_color(),
    };
    Chart::new(dataset)
        .block(
            Block::default()
                .title_alignment(Alignment::Center)
                .title(Span::styled(
                    format!(" {name} {current} "),
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
                        format!("{max}"),
                        Style::default()
                            .add_modifier(Modifier::BOLD)
                            .fg(label_color),
                    ),
                ])
                // Add 0.01, so that max point is always visible?
                .bounds([0.0, max.get_value() + 0.01]),
        )
}

/// Draw heading bar at top of program, always visible
/// TODO Should separate into loading icon/headers/help functions
#[allow(clippy::too_many_lines)]
pub fn heading_bar(
    area: Rect,
    frame: &mut Frame,
    data: &FrameData,
    gui_state: &Arc<Mutex<GuiState>>,
) {
    let block = |fg: Color| Block::default().style(Style::default().bg(Color::Magenta).fg(fg));

    frame.render_widget(block(Color::Black), area);

    // Generate a block for the header, if the header is currently being used to sort a column, then highlight it white
    let header_block = |x: &Header| {
        let mut color = Color::Black;
        let mut prefix = "";
        let mut prefix_margin = 0;
        if let Some((a, b)) = &data.sorted_by {
            if x == a {
                match b {
                    SortedOrder::Asc => prefix = "▲ ",
                    SortedOrder::Desc => prefix = "▼ ",
                }
                prefix_margin = 2;
                color = Color::White;
            };
        };
        (
            Block::default().style(Style::default().bg(Color::Magenta).fg(color)),
            prefix,
            prefix_margin,
        )
    };

    // Generate block for the headers, state and status has a specific layout, others all equal
    // width is dependant on it that column is selected to sort - or not
    let gen_header = |header: &Header, width: usize| {
        let block = header_block(header);
        // Yes this is a mess, needs documenting correctly
        let text = match header {
            Header::State => format!(
                " {x:>width$}",
                x = format!("{ic}{header}", ic = block.1),
                width = width
            ),
            Header::Name => format!(
                "  {x:>width$}",
                x = format!("{ic}{header}", ic = block.1),
                width = width
            ),
            Header::Status => format!(
                "{}  {x:>width$}",
                MARGIN,
                x = format!("{ic}{header}", ic = block.1),
                width = width
            ),
            _ => format!(
                "{}{x:>width$}",
                MARGIN,
                x = format!("{ic}{header}", ic = block.1),
                width = width
            ),
        };
        let count = u16::try_from(text.chars().count()).unwrap_or_default();
        let status = Paragraph::new(text)
            .block(block.0)
            .alignment(Alignment::Left);
        (status, count)
    };

    // Meta data to iterate over to create blocks with correct widths
    let header_meta = [
        (Header::Name, data.columns.name.1),
        (Header::State, data.columns.state.1),
        (Header::Status, data.columns.status.1),
        (Header::Cpu, data.columns.cpu.1),
        (Header::Memory, data.columns.mem.1 + data.columns.mem.2 + 3),
        (Header::Id, data.columns.id.1),
        (Header::Image, data.columns.image.1),
        (Header::Rx, data.columns.net_rx.1),
        (Header::Tx, data.columns.net_tx.1),
    ];

    // Need to add widths to this

    let suffix = if data.help_visible { "exit" } else { "show" };
    let info_text = format!("( h ) {suffix} help {MARGIN}",);
    let info_width = info_text.chars().count();

    let column_width = usize::from(area.width).saturating_sub(info_width);
    let column_width = if column_width > 0 { column_width } else { 1 };
    let splits = if data.has_containers {
        vec![
            Constraint::Max(2),
            Constraint::Min(column_width.try_into().unwrap_or_default()),
            Constraint::Max(info_width.try_into().unwrap_or_default()),
        ]
    } else {
        CONSTRAINT_100.to_vec()
    };

    let split_bar = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(splits)
        .split(area);

    if data.has_containers {
        let header_section_width = split_bar[1].width;

        let mut counter = 0;

        // Only show a header if the header cumulative header width is less than the header section width
        let header_data = header_meta
            .iter()
            .filter_map(|i| {
                let header_block = gen_header(&i.0, i.1.into());
                counter += header_block.1;
                if counter <= header_section_width {
                    Some((header_block.0, i.0, Constraint::Max(header_block.1)))
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();

        // Draw loading icon, or not, and a prefix with a single space
        let loading_paragraph = Paragraph::new(format!("{:>2}", data.loading_icon))
            .block(block(Color::White))
            .alignment(Alignment::Center);
        frame.render_widget(loading_paragraph, split_bar[0]);

        let container_splits = header_data.iter().map(|i| i.2).collect::<Vec<_>>();
        let headers_section = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(container_splits)
            .split(split_bar[1]);

        for (index, (paragraph, header, _)) in header_data.into_iter().enumerate() {
            let rect = headers_section[index];
            gui_state
                .lock()
                .update_region_map(Region::Header(header), rect);
            frame.render_widget(paragraph, rect);
        }
    }

    // show/hide help
    let color = if data.help_visible {
        Color::Black
    } else {
        Color::White
    };
    let help_paragraph = Paragraph::new(info_text)
        .block(block(color))
        .alignment(Alignment::Right);

    // If no containers, don't display the headers, could maybe do this first?
    let help_index = if data.has_containers { 2 } else { 0 };
    frame.render_widget(help_paragraph, split_bar[help_index]);
}

/// Help popup box needs these three pieces of information
struct HelpInfo {
    lines: Vec<Line<'static>>,
    width: usize,
    height: usize,
}

impl HelpInfo {
    /// Find the max width of a Span in &[Line], although it isn't calculating it correctly
    fn calc_width(lines: &[Line]) -> usize {
        lines
            .iter()
            .flat_map(|x| x.spans.iter())
            .map(ratatui::text::Span::width)
            .max()
            .unwrap_or(1)
    }

    /// Just an empty span, i.e. a new line
    fn empty_span<'a>() -> Line<'a> {
        Line::from(String::new())
    }

    /// generate a span, of given &str and given color
    fn span<'a>(input: &str, color: Color) -> Span<'a> {
        Span::styled(input.to_owned(), Style::default().fg(color))
    }

    /// &str to black text span
    fn black_span<'a>(input: &str) -> Span<'a> {
        Self::span(input, Color::Black)
    }

    /// &str to white text span
    fn white_span<'a>(input: &str) -> Span<'a> {
        Self::span(input, Color::White)
    }

    /// Generate the `oxker` name span + metadata
    fn gen_name() -> Self {
        let mut lines = NAME_TEXT
            .lines()
            .map(|i| Line::from(Self::white_span(i)))
            .collect::<Vec<_>>();
        lines.insert(0, Self::empty_span());
        let width = Self::calc_width(&lines);
        let height = lines.len();
        Self {
            lines,
            width,
            height,
        }
    }

    /// Generate the description span + metadata
    fn gen_description() -> Self {
        let lines = [
            Self::empty_span(),
            Line::from(Self::white_span(DESCRIPTION)),
            Self::empty_span(),
        ];
        let width = Self::calc_width(&lines);
        let height = lines.len();
        Self {
            lines: lines.to_vec(),
            width,
            height,
        }
    }

    /// Generate the button information span + metadata
    fn gen_button() -> Self {
        let button_item = |x: &str| Self::white_span(&format!(" ( {x} ) "));
        let button_desc = |x: &str| Self::black_span(x);
        let or = || button_desc("or");
        let space = || button_desc(" ");

        let lines = [
            Line::from(vec![
                space(),
                button_item("tab"),
                or(),
                button_item("shift+tab"),
                button_desc("change panels"),
            ]),
            Line::from(vec![
                space(),
                button_item("↑ ↓"),
                or(),
                button_item("j k"),
                or(),
                button_item("PgUp PgDown"),
                or(),
                button_item("Home End"),
                button_desc("change selected line"),
            ]),
            Line::from(vec![
                space(),
                button_item("enter"),
                button_desc("send docker container command"),
            ]),
            Line::from(vec![
                space(),
                button_item("e"),
                button_desc("exec into a container"),
                #[cfg(target_os = "windows")]
                button_desc(" - not available on Windows"),
            ]),
            Line::from(vec![
                space(),
                button_item("h"),
                button_desc("toggle this help information"),
            ]),
            Line::from(vec![
                space(),
                button_item("s"),
                button_desc("save logs to file"),
            ]),
            Line::from(vec![
                space(),
                button_item("m"),
                button_desc(
                    "toggle mouse capture - if disabled, text on screen can be selected & copied",
                ),
            ]),
            Line::from(vec![space(), button_item("0"), button_desc("stop sort")]),
            Line::from(vec![
                space(),
                button_item("1 - 9"),
                button_desc("sort by header - or click header"),
            ]),
            Line::from(vec![
                space(),
                button_item("esc"),
                button_desc("close dialog"),
            ]),
            Line::from(vec![
                space(),
                button_item("q"),
                button_desc("quit at any time"),
            ]),
        ];

        let height = lines.len();
        let width = Self::calc_width(&lines);
        Self {
            lines: lines.to_vec(),
            width,
            height,
        }
    }

    /// Generate the final lines, GitHub link etc, + metadata
    fn gen_final() -> Self {
        let lines = [
            Self::empty_span(),
            Line::from(vec![Self::black_span(
                "currently an early work in progress, all and any input appreciated",
            )]),
            Line::from(vec![Span::styled(
                REPO.to_owned(),
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::UNDERLINED),
            )]),
        ];
        let height = lines.len();
        let width = Self::calc_width(&lines);
        Self {
            lines: lines.to_vec(),
            width,
            height,
        }
    }
}

/// Draw the help box in the centre of the screen
pub fn help_box(f: &mut Frame) {
    let title = format!(" {VERSION} ");

    let name_info = HelpInfo::gen_name();
    let description_info = HelpInfo::gen_description();
    let button_info = HelpInfo::gen_button();
    let final_info = HelpInfo::gen_final();

    // have to add 10, but shouldn't need to, is an error somewhere
    let max_line_width = [
        name_info.width,
        description_info.width,
        button_info.width,
        final_info.width,
    ]
    .into_iter()
    .max()
    .unwrap_or_default()
        + 10;
    let max_height =
        name_info.height + description_info.height + button_info.height + final_info.height + 2;

    let area = popup(
        max_height,
        max_line_width,
        f.size(),
        BoxLocation::MiddleCentre,
    );

    let split_popup = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Max(name_info.height.try_into().unwrap_or_default()),
            Constraint::Max(description_info.height.try_into().unwrap_or_default()),
            Constraint::Max(button_info.height.try_into().unwrap_or_default()),
            Constraint::Min(final_info.height.try_into().unwrap_or_default()),
        ])
        .split(area);

    let name_paragraph = Paragraph::new(name_info.lines)
        .style(Style::default().bg(Color::Magenta).fg(Color::White))
        .block(Block::default())
        .alignment(Alignment::Center);

    let description_paragraph = Paragraph::new(description_info.lines)
        .style(Style::default().bg(Color::Magenta).fg(Color::Black))
        .block(Block::default())
        .alignment(Alignment::Center);

    let help_paragraph = Paragraph::new(button_info.lines)
        .style(Style::default().bg(Color::Magenta).fg(Color::Black))
        .block(Block::default())
        .alignment(Alignment::Left);

    let final_paragraph = Paragraph::new(final_info.lines)
        .style(Style::default().bg(Color::Magenta).fg(Color::Black))
        .block(Block::default())
        .alignment(Alignment::Center);

    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::Black).bg(Color::Magenta));

    // Order is important here
    f.render_widget(Clear, area);
    f.render_widget(name_paragraph, split_popup[0]);
    f.render_widget(description_paragraph, split_popup[1]);
    f.render_widget(help_paragraph, split_popup[2]);
    f.render_widget(final_paragraph, split_popup[3]);
    f.render_widget(block, area);
}

/// Draw the delete confirm box in the centre of the screen
/// take in container id and container name here?
pub fn delete_confirm(f: &mut Frame, gui_state: &Arc<Mutex<GuiState>>, name: &ContainerName) {
    let block = Block::default()
        .title(" Confirm Delete ")
        .border_type(BorderType::Rounded)
        .style(Style::default().bg(Color::White).fg(Color::Black))
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL);

    let confirm = Line::from(vec![
        Span::from("Are you sure you want to delete container: "),
        Span::styled(
            name.get(),
            Style::default()
                .fg(Color::Red)
                .bg(Color::White)
                .add_modifier(Modifier::BOLD),
        ),
    ]);

    let yes_text = " (Y)es ";
    let no_text = " (N)o ";

    // Find the maximum line width & height, and add some padding
    let max_line_width = u16::try_from(confirm.width()).unwrap_or(64) + 12;
    let lines = 8;

    let confirm_para = Paragraph::new(confirm).alignment(Alignment::Center);

    let button_block = || {
        Block::default()
            .border_type(BorderType::Rounded)
            .borders(Borders::ALL)
            .style(Style::default().bg(Color::White))
    };

    let yes_para = Paragraph::new(yes_text)
        .alignment(Alignment::Center)
        .block(button_block());

    let no_para = Paragraph::new(no_text)
        .alignment(Alignment::Center)
        .block(button_block());

    let area = popup(
        lines,
        max_line_width.into(),
        f.size(),
        BoxLocation::MiddleCentre,
    );

    let split_popup = Layout::default()
        .direction(Direction::Vertical)
        .constraints(CONSTRAINT_POPUP)
        .split(area);

    let split_buttons = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(CONSTRAINT_BUTTONS)
        .split(split_popup[3]);

    let no_area = split_buttons[1];
    let yes_area = split_buttons[3];

    f.render_widget(Clear, area);
    f.render_widget(block, area);
    f.render_widget(confirm_para, split_popup[1]);
    f.render_widget(no_para, no_area);
    f.render_widget(yes_para, yes_area);
    // Insert button areas into region map, so can interact with them on click
    gui_state
        .lock()
        .update_region_map(Region::Delete(DeleteButton::No), no_area);

    gui_state
        .lock()
        .update_region_map(Region::Delete(DeleteButton::Yes), yes_area);
}

/// Draw an error popup over whole screen
pub fn error(f: &mut Frame, error: AppError, seconds: Option<u8>) {
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
        _ => String::from("\n\n ( c ) clear error\n ( q ) quit oxker "),
    };

    let mut text = format!("\n{error}");

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

    let area = popup(lines, max_line_width, f.size(), BoxLocation::MiddleCentre);

    // let (paragraph, area) = gen_error(f, error, seconds);
    f.render_widget(Clear, area);
    f.render_widget(paragraph, area);
}

/// Draw info box in one of the 9 BoxLocations
// TODO is this broken?
pub fn info(f: &mut Frame, text: &str, instant: Instant, gui_state: &Arc<Mutex<GuiState>>) {
    let block = Block::default()
        .title("")
        .title_alignment(Alignment::Center)
        .borders(Borders::NONE);

    let mut max_line_width = max_line_width(text);
    let mut lines = text.lines().count();

    // Add some horizontal & vertical margins
    max_line_width += 8;
    lines += 2;

    let paragraph = Paragraph::new(text)
        .style(Style::default().bg(Color::Blue).fg(Color::White))
        .block(block)
        .alignment(Alignment::Center);

    let area = popup(lines, max_line_width, f.size(), BoxLocation::BottomRight);
    f.render_widget(Clear, area);
    f.render_widget(paragraph, area);
    if instant.elapsed().as_millis() > 4000 {
        gui_state.lock().reset_info_box();
    }
}

/// draw a box in the one of the BoxLocations, based on max line width + number of lines
fn popup(text_lines: usize, text_width: usize, r: Rect, box_location: BoxLocation) -> Rect {
    // Make sure blank_space can't be an negative, as will crash
    let calc = |x: u16, y: usize| usize::from(x).saturating_sub(y).saturating_div(2);

    let blank_vertical = calc(r.height, text_lines);
    let blank_horizontal = calc(r.width, text_width);

    let (h_constraints, v_constraints) = box_location.get_constraints(
        blank_horizontal.try_into().unwrap_or_default(),
        blank_vertical.try_into().unwrap_or_default(),
        text_lines.try_into().unwrap_or_default(),
        text_width.try_into().unwrap_or_default(),
    );

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

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::many_single_char_names)]
mod tests {

    use std::{ops::RangeInclusive, sync::Arc};

    use parking_lot::Mutex;
    use ratatui::{
        backend::TestBackend,
        layout::Rect,
        style::{Color, Modifier},
        Terminal,
    };
    use uuid::Uuid;

    use crate::{
        app_data::{
            AppData, ContainerId, ContainerImage, ContainerName, ContainerPorts, Header,
            SortedOrder, State, StatefulList,
        },
        app_error::AppError,
        tests::{gen_appdata, gen_container_summary, gen_containers},
        ui::{draw_frame, GuiState},
    };

    use super::{FrameData, ORANGE, VERSION};

    struct TuiTestSetup {
        app_data: Arc<Mutex<AppData>>,
        gui_state: Arc<Mutex<GuiState>>,
        fd: FrameData,
        area: Rect,
        terminal: Terminal<TestBackend>,
        ids: Vec<ContainerId>,
    }

    const BORDER_CHARS: [&str; 6] = ["╭", "╮", "─", "│", "╰", "╯"];

    /// Generate state to be used in *most* gui tests
    fn test_setup(w: u16, h: u16, control_start: bool, container_start: bool) -> TuiTestSetup {
        let backend = TestBackend::new(w, h);
        let terminal = Terminal::new(backend).unwrap();

        let (ids, containers) = gen_containers();
        let mut app_data = gen_appdata(&containers);
        if control_start {
            app_data.docker_controls_start();
        }
        if container_start {
            app_data.containers_start();
        }

        let gui_state = GuiState::default();

        let app_data = Arc::new(Mutex::new(app_data));
        let gui_state = Arc::new(Mutex::new(gui_state));

        let fd = FrameData::from((app_data.lock(), gui_state.lock()));
        let area = Rect::new(0, 0, w, h);
        TuiTestSetup {
            app_data,
            gui_state,
            fd,
            area,
            terminal,
            ids,
        }
    }

    /// Insert some logs into the first container
    fn insert_logs(setup: &TuiTestSetup) {
        let logs = (1..=3).map(|i| format!("{i} line {i}")).collect::<Vec<_>>();
        setup.app_data.lock().update_log_by_id(logs, &setup.ids[0]);
    }

    // ******************** //
    // DockerControls panel //
    // ******************** //

    #[test]
    /// Test that when DockerCommands are available, they are drawn correctly, dependant on container state
    fn test_draw_blocks_commands_none() {
        let (w, h) = (12, 6);
        let mut setup = test_setup(w, h, false, false);

        setup
            .terminal
            .draw(|f| {
                super::commands(&setup.app_data, setup.area, f, &setup.fd, &setup.gui_state);
            })
            .unwrap();

        let expected = [
            "╭──────────╮",
            "│          │",
            "│          │",
            "│          │",
            "│          │",
            "╰──────────╯",
        ];

        let result = &setup.terminal.backend().buffer().content;
        for (row_index, row) in expected.iter().enumerate() {
            for (char_index, expected_char) in row.chars().enumerate() {
                let index = row_index * usize::from(w) + char_index;
                let result_cell = &result[index];

                assert_eq!(result_cell.symbol(), expected_char.to_string());
                assert_eq!(result_cell.fg, Color::Reset);
            }
        }
    }

    #[test]
    // Test that when DockerCommands are available, they are drawn correctly, dependant on container state
    fn test_draw_blocks_commands_some() {
        let (w, h) = (12, 6);
        let mut setup = test_setup(w, h, true, true);

        setup
            .terminal
            .draw(|f| {
                super::commands(&setup.app_data, setup.area, f, &setup.fd, &setup.gui_state);
            })
            .unwrap();

        let expected = [
            "╭──────────╮",
            "│▶ pause   │",
            "│  restart │",
            "│  stop    │",
            "│  delete  │",
            "╰──────────╯",
        ];
        let result = &setup.terminal.backend().buffer().content;

        for (row_index, row) in expected.iter().enumerate() {
            for (char_index, expected_char) in row.chars().enumerate() {
                let index = row_index * usize::from(w) + char_index;
                let result_cell = &result[index];

                assert_eq!(result_cell.symbol(), expected_char.to_string());

                // Check the text color is correct
                match index {
                    // pause
                    15..=19 => {
                        assert_eq!(result_cell.fg, Color::Yellow);
                    }
                    // restart
                    27..=33 => {
                        assert_eq!(result_cell.fg, Color::Magenta);
                    }
                    // stop
                    39..=42 => {
                        assert_eq!(result_cell.fg, Color::Red);
                    }
                    // delete
                    51..=56 => {
                        assert_eq!(result_cell.fg, Color::Gray);
                    }
                    // no text
                    _ => {
                        assert_eq!(result_cell.fg, Color::Reset);
                    }
                }
                if result_cell.symbol().starts_with('▶') {
                    assert_eq!(result_cell.fg, Color::Reset);
                }
            }
        }

        // Change the controls state
        setup
            .app_data
            .lock()
            .update_containers(&mut vec![gen_container_summary(1, "paused")]);
        setup.app_data.lock().docker_controls_next();

        let expected = [
            "╭──────────╮",
            "│  resume  │",
            "│▶ stop    │",
            "│  delete  │",
            "│          │",
            "╰──────────╯",
        ];

        setup
            .terminal
            .draw(|f| {
                super::commands(&setup.app_data, setup.area, f, &setup.fd, &setup.gui_state);
            })
            .unwrap();

        let result = &setup.terminal.backend().buffer().content;

        for (row_index, row) in expected.iter().enumerate() {
            for (char_index, expected_char) in row.chars().enumerate() {
                let index = row_index * usize::from(w) + char_index;
                let result_cell = &result[index];

                assert_eq!(result_cell.symbol(), expected_char.to_string());

                // Chceck the text color is correct
                match index {
                    // resume
                    15..=20 => {
                        assert_eq!(result_cell.fg, Color::Blue);
                    }
                    // stop
                    27..=30 => {
                        assert_eq!(result_cell.fg, Color::Red);
                    }
                    // delete
                    39..=44 => {
                        assert_eq!(result_cell.fg, Color::Gray);
                    }
                    // no text
                    _ => {
                        assert_eq!(result_cell.fg, Color::Reset);
                    }
                }
                if result_cell.symbol().starts_with('▶') {
                    assert_eq!(result_cell.fg, Color::Reset);
                }
            }
        }
    }

    #[test]
    /// When control panel is selected, the border is blue, if not then white, selected text is highlighted
    fn test_draw_blocks_commands_panel_selected_color() {
        let (w, h) = (12, 6);
        let mut setup = test_setup(w, h, true, true);
        let expected = [
            "╭──────────╮",
            "│▶ pause   │",
            "│  restart │",
            "│  stop    │",
            "│  delete  │",
            "╰──────────╯",
        ];

        // Unselected, has a grey border
        setup
            .terminal
            .draw(|f| {
                super::commands(&setup.app_data, setup.area, f, &setup.fd, &setup.gui_state);
            })
            .unwrap();

        let result = &setup.terminal.backend().buffer().content;
        for (row_index, row) in expected.iter().enumerate() {
            for (char_index, expected_char) in row.chars().enumerate() {
                let index = row_index * usize::from(w) + char_index;
                let result_cell = &result[index];

                assert_eq!(result_cell.symbol(), expected_char.to_string());
                if BORDER_CHARS.contains(&result_cell.symbol()) {
                    assert_eq!(result_cell.fg, Color::Reset);
                }
            }
        }

        // Control panel now selected, should have a blue border
        setup.gui_state.lock().next_panel();
        let fd = FrameData::from((setup.app_data.lock(), setup.gui_state.lock()));
        setup
            .terminal
            .draw(|f| {
                super::commands(&setup.app_data, setup.area, f, &fd, &setup.gui_state);
            })
            .unwrap();

        let result = &setup.terminal.backend().buffer().content;
        for (row_index, row) in expected.iter().enumerate() {
            for (char_index, expected_char) in row.chars().enumerate() {
                let index = row_index * usize::from(w) + char_index;
                let result_cell = &result[index];

                assert_eq!(result_cell.symbol(), expected_char.to_string());
                if BORDER_CHARS.contains(&result_cell.symbol()) {
                    assert_eq!(result_cell.fg, Color::LightCyan);
                }
                // Make sure that the selected line has bold text
                match index {
                    // pause
                    13..=22 => {
                        assert_eq!(result_cell.modifier, Modifier::BOLD);
                    }
                    _ => {
                        assert!(result_cell.modifier.is_empty());
                    }
                }
            }
        }
    }

    // *********************** //
    // Container summary panel //
    // *********************** //

    // Check that the correct solor is applied to the state/status/cpu/memory section
    fn check_expected(expected: [&str; 6], w: u16, _h: u16, setup: &TuiTestSetup, color: Color) {
        let result = &setup.terminal.backend().buffer().content;
        for (row_index, row) in expected.iter().enumerate() {
            for (char_index, expected_char) in row.chars().enumerate() {
                let index = row_index * usize::from(w) + char_index;
                let result_cell = &result[index];

                assert_eq!(result_cell.symbol(), expected_char.to_string());
                if (145..=207).contains(&index) {
                    assert_eq!(result_cell.fg, color);
                    assert_eq!(result_cell.modifier, Modifier::BOLD);
                }
            }
        }
    }

    #[test]
    /// No containers, panel unselected, then selected, border color changes correctly
    fn test_draw_blocks_containers_none() {
        let (w, h) = (25, 6);
        let mut setup = test_setup(w, h, true, true);
        setup.app_data.lock().containers = StatefulList::new(vec![]);

        let expected = [
            "╭ Containers ───────────╮",
            "│ no containers running │",
            "│                       │",
            "│                       │",
            "│                       │",
            "╰───────────────────────╯",
        ];

        setup.gui_state.lock().next_panel();
        let fd = FrameData::from((setup.app_data.lock(), setup.gui_state.lock()));

        setup
            .terminal
            .draw(|f| {
                super::containers(&setup.app_data, setup.area, f, &fd, &setup.gui_state);
            })
            .unwrap();

        let result = &setup.terminal.backend().buffer().content;
        for (row_index, row) in expected.iter().enumerate() {
            for (char_index, expected_char) in row.chars().enumerate() {
                let index = row_index * usize::from(w) + char_index;
                let result_cell = &result[index];

                assert_eq!(result_cell.symbol(), expected_char.to_string());
                assert_eq!(result_cell.fg, Color::Reset);
            }
        }

        setup.gui_state.lock().previous_panel();
        let fd = FrameData::from((setup.app_data.lock(), setup.gui_state.lock()));

        setup
            .terminal
            .draw(|f| {
                super::containers(&setup.app_data, setup.area, f, &fd, &setup.gui_state);
            })
            .unwrap();

        let result = &setup.terminal.backend().buffer().content;
        for (row_index, row) in expected.iter().enumerate() {
            for (char_index, expected_char) in row.chars().enumerate() {
                let index = row_index * usize::from(w) + char_index;
                let result_cell = &result[index];

                assert_eq!(result_cell.symbol(), expected_char.to_string());
                if BORDER_CHARS.contains(&result_cell.symbol()) {
                    assert_eq!(result_cell.fg, Color::LightCyan);
                }
            }
        }
    }

    #[test]
    /// Containers panel drawn, selected line is bold, border is blue
    fn test_draw_blocks_containers_some() {
        let (w, h) = (130, 6);
        let mut setup = test_setup(w, h, true, true);

        let expected = [
        "╭ Containers 1/3 ────────────────────────────────────────────────────────────────────────────────────────────────────────────────╮",
        "│⚪  container_1   ✓ running            Up 1 hour    00.00%   0.00 kB / 0.00 kB          1   image_1   0.00 kB   0.00 kB          │",
        "│   container_2   ✓ running            Up 2 hour    00.00%   0.00 kB / 0.00 kB          2   image_2   0.00 kB   0.00 kB          │",
        "│   container_3   ✓ running            Up 3 hour    00.00%   0.00 kB / 0.00 kB          3   image_3   0.00 kB   0.00 kB          │",
        "│                                                                                                                                │",
        "╰────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────╯",
    ];

        setup
            .terminal
            .draw(|f| {
                super::containers(&setup.app_data, setup.area, f, &setup.fd, &setup.gui_state);
            })
            .unwrap();

        let result = &setup.terminal.backend().buffer().content;
        for (row_index, row) in expected.iter().enumerate() {
            for (char_index, expected_char) in row.chars().enumerate() {
                let index = row_index * usize::from(w) + char_index;
                let result_cell = &result[index];

                // result matches expected
                assert_eq!(result_cell.symbol(), expected_char.to_string());

                // Selected container is bold
                match index {
                    131 | 133..=258 => assert_eq!(result_cell.modifier, Modifier::BOLD),
                    _ => {
                        assert!(result_cell.modifier.is_empty());
                    }
                }

                // Border is blue
                if BORDER_CHARS.contains(&result_cell.symbol()) {
                    assert_eq!(result_cell.fg, Color::LightCyan);
                }
            }
        }

        // Change selected panel, border is now no longer blue
        setup.gui_state.lock().next_panel();
        let fd = FrameData::from((setup.app_data.lock(), setup.gui_state.lock()));
        setup
            .terminal
            .draw(|f| {
                super::containers(&setup.app_data, setup.area, f, &fd, &setup.gui_state);
            })
            .unwrap();

        let result = &setup.terminal.backend().buffer().content;
        for (row_index, row) in expected.iter().enumerate() {
            for (char_index, expected_char) in row.chars().enumerate() {
                let index = row_index * usize::from(w) + char_index;
                let result_cell = &result[index];

                assert_eq!(result_cell.symbol(), expected_char.to_string());

                // Border is gray
                if BORDER_CHARS.contains(&result_cell.symbol()) {
                    assert_eq!(result_cell.fg, Color::Reset);
                }
            }
        }
    }

    #[test]
    /// ALl columns on all rows are coloured correctly
    fn test_draw_blocks_containers_colors() {
        let (w, h) = (130, 6);
        let mut setup = test_setup(w, h, true, true);

        let expected = [
            "╭ Containers 1/3 ────────────────────────────────────────────────────────────────────────────────────────────────────────────────╮",
            "│⚪  container_1   ✓ running            Up 1 hour    00.00%   0.00 kB / 0.00 kB          1   image_1   0.00 kB   0.00 kB          │",
            "│   container_2   ✓ running            Up 2 hour    00.00%   0.00 kB / 0.00 kB          2   image_2   0.00 kB   0.00 kB          │",
            "│   container_3   ✓ running            Up 3 hour    00.00%   0.00 kB / 0.00 kB          3   image_3   0.00 kB   0.00 kB          │",
            "│                                                                                                                                │",
            "╰────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────╯",
        ];
        let fd = FrameData::from((setup.app_data.lock(), setup.gui_state.lock()));

        setup
            .terminal
            .draw(|f| {
                super::containers(&setup.app_data, setup.area, f, &fd, &setup.gui_state);
            })
            .unwrap();

        let index_blue = [
            134, 135, 136, 137, 138, 139, 140, 141, 142, 143, 144, 208, 209, 210, 211, 212, 213,
            214, 215, 216, 217, 218, 219, 220, 221, 222, 223, 224, 225, 226, 227, 228,
        ];
        let index_blue = index_blue
            .iter()
            .flat_map(|&x| vec![x, x + 130, x + 260])
            .collect::<Vec<_>>();
        let index_green = [
            145, 146, 147, 148, 149, 150, 151, 152, 153, 154, 155, 156, 157, 158, 159, 160, 161,
            162, 163, 164, 165, 166, 167, 168, 169, 170, 171, 172, 173, 174, 175, 176, 177, 178,
            179, 180, 181, 182, 183, 184, 185, 186, 187, 188, 189, 190, 191, 192, 193, 194, 195,
            196, 197, 198, 199, 200, 201, 202, 203, 204, 205, 206, 207,
        ];
        let index_green = index_green
            .iter()
            .flat_map(|&x| vec![x, x + 130, x + 260])
            .collect::<Vec<_>>();

        let index_rx = [229, 230, 231, 232, 233, 234, 235, 236, 237, 238];
        let index_rx = index_rx
            .iter()
            .flat_map(|&x| vec![x, x + 130, x + 260])
            .collect::<Vec<_>>();

        let index_tx = [239, 240, 241, 242, 243, 244, 245, 246, 247, 248];
        let index_tx = index_tx
            .iter()
            .flat_map(|&x| vec![x, x + 130, x + 260])
            .collect::<Vec<_>>();

        let result = &setup.terminal.backend().buffer().content;
        for (row_index, row) in expected.iter().enumerate() {
            for (char_index, expected_char) in row.chars().enumerate() {
                let index = row_index * usize::from(w) + char_index;

                let result_cell = &result[index];
                assert_eq!(result_cell.symbol(), expected_char.to_string());

                match index {
                    _x if index_blue.contains(&index) => {
                        assert_eq!(result_cell.fg, Color::Blue);
                    }
                    _x if index_green.contains(&index) => {
                        assert_eq!(result_cell.fg, Color::Green);
                    }
                    _x if index_rx.contains(&index) => {
                        assert_eq!(result_cell.fg, Color::Rgb(255, 233, 193));
                    }
                    _x if index_tx.contains(&index) => {
                        assert_eq!(result_cell.fg, Color::Rgb(205, 140, 140));
                    }
                    (0..=130) | (259..=260) | (389..=390) | (519..=520) | (649..=779) => {
                        assert_eq!(result_cell.fg, Color::LightCyan);
                    }
                    _ => {
                        assert_eq!(result_cell.fg, Color::Reset);
                    }
                }
            }
        }
    }

    #[test]
    /// When long container/image name, it is truncated correctly
    fn test_draw_blocks_containers_long_name_image() {
        let (w, h) = (170, 6);
        let mut setup = test_setup(w, h, true, true);
        setup.app_data.lock().containers.items[0].name =
            ContainerName::from("a_long_container_name_for_the_purposes_of_this_test");
        setup.app_data.lock().containers.items[0].image =
            ContainerImage::from("a_long_image_name_for_the_purposes_of_this_test");

        let expected = [
        "╭ Containers 1/3 ────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────╮",
        "│⚪  a_long_container_name_for_the…   ॥ paused             Up 1 hour    00.00%   0.00 kB / 0.00 kB          1   a_long_image_name_for_the_pur…   0.00 kB   0.00 kB        │",
        "│                      container_2   ✓ running            Up 2 hour    00.00%   0.00 kB / 0.00 kB          2                          image_2   0.00 kB   0.00 kB        │",
        "│                      container_3   ✓ running            Up 3 hour    00.00%   0.00 kB / 0.00 kB          3                          image_3   0.00 kB   0.00 kB        │",
        "│                                                                                                                                                                        │",
        "╰────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────╯",
        ];
        let fd = FrameData::from((setup.app_data.lock(), setup.gui_state.lock()));
        setup.app_data.lock().containers.items[0].state = State::Paused;

        setup
            .terminal
            .draw(|f| {
                super::containers(&setup.app_data, setup.area, f, &fd, &setup.gui_state);
            })
            .unwrap();

        let result = &setup.terminal.backend().buffer().content;
        for (row_index, row) in expected.iter().enumerate() {
            for (char_index, expected_char) in row.chars().enumerate() {
                let index = row_index * usize::from(w) + char_index;
                let result_cell = &result[index];

                assert_eq!(result_cell.symbol(), expected_char.to_string());
            }
        }

        // THis char: …
    }

    #[test]
    /// When container is paused, correct colors displayed
    fn test_draw_blocks_containers_paused() {
        let (w, h) = (130, 6);
        let mut setup = test_setup(w, h, true, true);

        let expected = [
            "╭ Containers 1/3 ────────────────────────────────────────────────────────────────────────────────────────────────────────────────╮",
        "│⚪  container_1   ॥ paused             Up 1 hour    00.00%   0.00 kB / 0.00 kB          1   image_1   0.00 kB   0.00 kB          │",
        "│   container_2   ✓ running            Up 2 hour    00.00%   0.00 kB / 0.00 kB          2   image_2   0.00 kB   0.00 kB          │",
        "│   container_3   ✓ running            Up 3 hour    00.00%   0.00 kB / 0.00 kB          3   image_3   0.00 kB   0.00 kB          │",
        "│                                                                                                                                │",
        "╰────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────╯",
        ];
        let fd = FrameData::from((setup.app_data.lock(), setup.gui_state.lock()));
        setup.app_data.lock().containers.items[0].state = State::Paused;

        setup
            .terminal
            .draw(|f| {
                super::containers(&setup.app_data, setup.area, f, &fd, &setup.gui_state);
            })
            .unwrap();

        check_expected(expected, w, h, &setup, Color::Yellow);
    }

    #[test]
    /// When container is dead, correct colors displayed
    fn test_draw_blocks_containers_dead() {
        let (w, h) = (130, 6);
        let mut setup = test_setup(w, h, true, true);

        let expected = [
            "╭ Containers 1/3 ────────────────────────────────────────────────────────────────────────────────────────────────────────────────╮",
            "│⚪  container_1   ✖ dead               Up 1 hour    00.00%   0.00 kB / 0.00 kB          1   image_1   0.00 kB   0.00 kB          │",
            "│   container_2   ✓ running            Up 2 hour    00.00%   0.00 kB / 0.00 kB          2   image_2   0.00 kB   0.00 kB          │",
            "│   container_3   ✓ running            Up 3 hour    00.00%   0.00 kB / 0.00 kB          3   image_3   0.00 kB   0.00 kB          │",
            "│                                                                                                                                │",
            "╰────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────╯",
        ];
        setup.app_data.lock().containers.items[0].state = State::Dead;
        let fd = FrameData::from((setup.app_data.lock(), setup.gui_state.lock()));

        setup
            .terminal
            .draw(|f| {
                super::containers(&setup.app_data, setup.area, f, &fd, &setup.gui_state);
            })
            .unwrap();
        check_expected(expected, w, h, &setup, Color::Red);
    }

    #[test]
    /// When container is exited, correct colors displayed
    fn test_draw_blocks_containers_exited() {
        let (w, h) = (130, 6);
        let mut setup = test_setup(w, h, true, true);

        let expected = [
            "╭ Containers 1/3 ────────────────────────────────────────────────────────────────────────────────────────────────────────────────╮",
            "│⚪  container_1   ✖ exited             Up 1 hour    00.00%   0.00 kB / 0.00 kB          1   image_1   0.00 kB   0.00 kB          │",
            "│   container_2   ✓ running            Up 2 hour    00.00%   0.00 kB / 0.00 kB          2   image_2   0.00 kB   0.00 kB          │",
            "│   container_3   ✓ running            Up 3 hour    00.00%   0.00 kB / 0.00 kB          3   image_3   0.00 kB   0.00 kB          │",
            "│                                                                                                                                │",
            "╰────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────╯",
        ];
        setup.app_data.lock().containers.items[0].state = State::Exited;
        let fd = FrameData::from((setup.app_data.lock(), setup.gui_state.lock()));

        setup
            .terminal
            .draw(|f| {
                super::containers(&setup.app_data, setup.area, f, &fd, &setup.gui_state);
            })
            .unwrap();

        check_expected(expected, w, h, &setup, Color::Red);
    }
    #[test]
    /// When container is paused, correct colors displayed
    fn test_draw_blocks_containers_removing() {
        let (w, h) = (130, 6);
        let mut setup = test_setup(w, h, true, true);

        let expected = [
            "╭ Containers 1/3 ────────────────────────────────────────────────────────────────────────────────────────────────────────────────╮",
            "│⚪  container_1   removing             Up 1 hour    00.00%   0.00 kB / 0.00 kB          1   image_1   0.00 kB   0.00 kB          │",
            "│   container_2   ✓ running            Up 2 hour    00.00%   0.00 kB / 0.00 kB          2   image_2   0.00 kB   0.00 kB          │",
            "│   container_3   ✓ running            Up 3 hour    00.00%   0.00 kB / 0.00 kB          3   image_3   0.00 kB   0.00 kB          │",
            "│                                                                                                                                │",
            "╰────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────╯",
        ];
        setup.app_data.lock().containers.items[0].state = State::Removing;
        let fd = FrameData::from((setup.app_data.lock(), setup.gui_state.lock()));

        setup
            .terminal
            .draw(|f| {
                super::containers(&setup.app_data, setup.area, f, &fd, &setup.gui_state);
            })
            .unwrap();

        check_expected(expected, w, h, &setup, Color::LightRed);
    }
    #[test]
    /// When container state is restarting, correct colors displayed
    fn test_draw_blocks_containers_restarting() {
        let (w, h) = (130, 6);
        let mut setup = test_setup(w, h, true, true);

        let expected = [
            "╭ Containers 1/3 ────────────────────────────────────────────────────────────────────────────────────────────────────────────────╮",
            "│⚪  container_1   ↻ restarting          Up 1 hour    00.00%   0.00 kB / 0.00 kB          1   image_1   0.00 kB   0.00 kB         │",
            "│   container_2   ✓ running             Up 2 hour    00.00%   0.00 kB / 0.00 kB          2   image_2   0.00 kB   0.00 kB         │",
            "│   container_3   ✓ running             Up 3 hour    00.00%   0.00 kB / 0.00 kB          3   image_3   0.00 kB   0.00 kB         │",
            "│                                                                                                                                │",
            "╰────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────╯",
        ];
        setup.app_data.lock().containers.items[0].state = State::Restarting;
        let fd = FrameData::from((setup.app_data.lock(), setup.gui_state.lock()));

        setup
            .terminal
            .draw(|f| {
                super::containers(&setup.app_data, setup.area, f, &fd, &setup.gui_state);
            })
            .unwrap();

        check_expected(expected, w, h, &setup, Color::LightGreen);
    }
    #[test]
    /// When container state is unknown, correct colors displayed
    fn test_draw_blocks_containers_unknown() {
        let (w, h) = (130, 6);
        let mut setup = test_setup(w, h, true, true);

        let expected = [
            "╭ Containers 1/3 ────────────────────────────────────────────────────────────────────────────────────────────────────────────────╮",
            "│⚪  container_1   ? unknown            Up 1 hour    00.00%   0.00 kB / 0.00 kB          1   image_1   0.00 kB   0.00 kB          │",
            "│   container_2   ✓ running            Up 2 hour    00.00%   0.00 kB / 0.00 kB          2   image_2   0.00 kB   0.00 kB          │",
            "│   container_3   ✓ running            Up 3 hour    00.00%   0.00 kB / 0.00 kB          3   image_3   0.00 kB   0.00 kB          │",
            "│                                                                                                                                │",
            "╰────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────╯",
        ];
        setup.app_data.lock().containers.items[0].state = State::Unknown;
        let fd = FrameData::from((setup.app_data.lock(), setup.gui_state.lock()));

        setup
            .terminal
            .draw(|f| {
                super::containers(&setup.app_data, setup.area, f, &fd, &setup.gui_state);
            })
            .unwrap();
        check_expected(expected, w, h, &setup, Color::Red);
    }
    // ********** //
    // Logs panel //
    // ********** //

    #[test]
    /// No logs, panel unselected, then selected, border color changes correctly
    fn test_draw_blocks_logs_none() {
        let (w, h) = (25, 6);
        let mut setup = test_setup(w, h, true, true);
        setup.app_data.lock().containers = StatefulList::new(vec![]);

        let expected = [
            "╭ Logs ─────────────────╮",
            "│     no logs found     │",
            "│                       │",
            "│                       │",
            "│                       │",
            "╰───────────────────────╯",
        ];

        let _fd = FrameData::from((setup.app_data.lock(), setup.gui_state.lock()));

        setup
            .terminal
            .draw(|f| {
                super::logs(&setup.app_data, setup.area, f, &setup.fd, &setup.gui_state);
            })
            .unwrap();

        let result = &setup.terminal.backend().buffer().content;
        for (row_index, row) in expected.iter().enumerate() {
            for (char_index, expected_char) in row.chars().enumerate() {
                let index = row_index * usize::from(w) + char_index;
                let result_cell = &result[index];

                assert_eq!(result_cell.symbol(), expected_char.to_string());
                assert_eq!(result_cell.fg, Color::Reset);
            }
        }

        setup.gui_state.lock().next_panel();
        setup.gui_state.lock().next_panel();
        let fd = FrameData::from((setup.app_data.lock(), setup.gui_state.lock()));

        // When selected, has a blue border
        setup
            .terminal
            .draw(|f| {
                super::logs(&setup.app_data, setup.area, f, &fd, &setup.gui_state);
            })
            .unwrap();

        let result = &setup.terminal.backend().buffer().content;
        for (row_index, row) in expected.iter().enumerate() {
            for (char_index, expected_char) in row.chars().enumerate() {
                let index = row_index * usize::from(w) + char_index;
                let result_cell = &result[index];

                assert_eq!(result_cell.symbol(), expected_char.to_string());
                if BORDER_CHARS.contains(&result_cell.symbol()) {
                    assert_eq!(result_cell.fg, Color::LightCyan);
                }
            }
        }
    }

    #[test]
    /// Parsing logs, spinner visible, and then animates by one frame
    fn test_draw_blocks_logs_parsing() {
        let (w, h) = (25, 6);
        let mut setup = test_setup(w, h, true, true);
        let uuid = Uuid::new_v4();
        setup.gui_state.lock().next_loading(uuid);

        let expected = [
            "╭ Logs - container_1 ───╮",
            "│    parsing logs ⠙     │",
            "│                       │",
            "│                       │",
            "│                       │",
            "╰───────────────────────╯",
        ];

        let mut fd = FrameData::from((setup.app_data.lock(), setup.gui_state.lock()));
        fd.init = true;

        setup
            .terminal
            .draw(|f| {
                super::logs(&setup.app_data, setup.area, f, &fd, &setup.gui_state);
            })
            .unwrap();

        let test = |terminal: &Terminal<TestBackend>, expected: [&str; 6]| {
            let result = &terminal.backend().buffer().content;
            for (row_index, row) in expected.iter().enumerate() {
                for (char_index, expected_char) in row.chars().enumerate() {
                    let index = row_index * usize::from(w) + char_index;
                    let result_cell = &result[index];

                    assert_eq!(result_cell.symbol(), expected_char.to_string());
                    assert_eq!(result_cell.fg, Color::Reset);
                }
            }
        };

        test(&setup.terminal, expected);

        // animation moved by one frame
        setup.gui_state.lock().next_loading(uuid);

        let expected = [
            "╭ Logs - container_1 ───╮",
            "│    parsing logs ⠹     │",
            "│                       │",
            "│                       │",
            "│                       │",
            "╰───────────────────────╯",
        ];

        let mut fd = FrameData::from((setup.app_data.lock(), setup.gui_state.lock()));
        fd.init = true;
        setup
            .terminal
            .draw(|f| {
                super::logs(&setup.app_data, setup.area, f, &fd, &setup.gui_state);
            })
            .unwrap();

        test(&setup.terminal, expected);
    }

    #[test]
    /// Logs correct displayed, changing log state also draws correctly
    fn test_draw_blocks_logs_some() {
        let (w, h) = (25, 6);
        let mut setup = test_setup(w, h, true, true);

        insert_logs(&setup);

        let test = |terminal: &Terminal<TestBackend>,
                    expected: [&str; 6],
                    range: RangeInclusive<usize>| {
            let result = &terminal.backend().buffer().content;

            for (row_index, row) in expected.iter().enumerate() {
                for (char_index, expected_char) in row.chars().enumerate() {
                    let index = row_index * usize::from(w) + char_index;
                    let result_cell = &result[index];

                    assert_eq!(result_cell.symbol(), expected_char.to_string());
                    assert_eq!(result_cell.fg, Color::Reset);

                    if range.contains(&index) {
                        assert_eq!(result_cell.modifier, Modifier::BOLD);
                    } else {
                        assert!(result_cell.modifier.is_empty());
                    }
                }
            }
        };

        let fd = FrameData::from((setup.app_data.lock(), setup.gui_state.lock()));
        setup
            .terminal
            .draw(|f| {
                super::logs(&setup.app_data, setup.area, f, &fd, &setup.gui_state);
            })
            .unwrap();
        let expected = [
            "╭ Logs 3/3 - container_1╮",
            "│  line 1               │",
            "│  line 2               │",
            "│▶ line 3               │",
            "│                       │",
            "╰───────────────────────╯",
        ];
        test(&setup.terminal, expected, 76..=98);

        // Change selected log line
        setup.app_data.lock().log_previous();
        let _fd = FrameData::from((setup.app_data.lock(), setup.gui_state.lock()));

        setup
            .terminal
            .draw(|f| {
                super::logs(&setup.app_data, setup.area, f, &setup.fd, &setup.gui_state);
            })
            .unwrap();

        let expected = [
            "╭ Logs 2/3 - container_1╮",
            "│  line 1               │",
            "│▶ line 2               │",
            "│  line 3               │",
            "│                       │",
            "╰───────────────────────╯",
        ];
        test(&setup.terminal, expected, 51..=73);
    }

    #[test]
    /// Full (long) name displayed in logs border
    fn test_draw_blocks_logs_long_name() {
        let (w, h) = (80, 6);
        let mut setup = test_setup(w, h, true, true);
        setup.app_data.lock().containers.items[0].name =
            ContainerName::from("a_long_container_name_for_the_purposes_of_this_test");
        setup.app_data.lock().containers.items[0].image =
            ContainerImage::from("a_long_image_name_for_the_purposes_of_this_test");

        insert_logs(&setup);

        let expected = [
            "╭ Logs 3/3 - a_long_container_name_for_the_purposes_of_this_test ──────────────╮",
            "│  line 1                                                                      │",
            "│  line 2                                                                      │",
            "│▶ line 3                                                                      │",
            "│                                                                              │",
            "╰──────────────────────────────────────────────────────────────────────────────╯",
        ];

        let fd = FrameData::from((setup.app_data.lock(), setup.gui_state.lock()));
        setup
            .terminal
            .draw(|f| {
                super::logs(&setup.app_data, setup.area, f, &fd, &setup.gui_state);
            })
            .unwrap();

        let result = &setup.terminal.backend().buffer().content;

        for (row_index, row) in expected.iter().enumerate() {
            for (char_index, expected_char) in row.chars().enumerate() {
                let index = row_index * usize::from(w) + char_index;
                let result_cell = &result[index];

                assert_eq!(result_cell.symbol(), expected_char.to_string());
            }
        }
    }

    // ************ //
    // Charts panel //
    // ************ //

    const EXPECTED: [&str; 10] = [
        "╭───────────── cpu 03.00% ─────────────╮╭────────── memory 30.00 kB ───────────╮",
        "│10.00%│    •                          ││100.00 kB│   ••                       │",
        "│      │   ••                          ││         │   ••                       │",
        "│      │  •••                          ││         │  • •                       │",
        "│      │  • •                          ││         │ •  •                       │",
        "│      │ •   ••                        ││         │••  ••                      │",
        "│      │•    •                         ││         │•   •                       │",
        "│      │•    •                         ││         │•   •                       │",
        "│      │                               ││         │                            │",
        "╰──────────────────────────────────────╯╰──────────────────────────────────────╯",
    ];
    const MEMORY_INDEX: [usize; 16] = [
        134, 135, 214, 215, 293, 295, 372, 375, 451, 452, 455, 456, 531, 535, 611, 615,
    ];

    const CPU_INDEX: [usize; 15] = [
        92, 171, 172, 250, 251, 252, 330, 332, 409, 413, 414, 488, 493, 568, 573,
    ];

    #[allow(clippy::cast_precision_loss)]
    // Add fixed data to the cpu & mem vecdeques, that match the above data
    fn insert_chart_data(setup: &TuiTestSetup) {
        for i in 1..=10 {
            setup.app_data.lock().update_stats_by_id(
                &setup.ids[0],
                Some(i as f64),
                Some(i * 10000),
                i * 10000,
                i,
                i,
            );
        }
        for i in 1..=3 {
            setup.app_data.lock().update_stats_by_id(
                &setup.ids[0],
                Some(i as f64),
                Some(i * 10000),
                i * 10000,
                i,
                i,
            );
        }
    }
    #[test]
    /// When status is Running, but not data, charts drawn without dots etc
    fn test_draw_blocks_charts_running_none() {
        let (w, h) = (80, 10);
        let mut setup = test_setup(w, h, true, true);

        setup
            .terminal
            .draw(|f| {
                super::chart(f, setup.area, &setup.app_data);
            })
            .unwrap();

        let expected = [
            "╭───────────── cpu 00.00% ─────────────╮╭─────────── memory 0.00 kB ───────────╮",
            "│00.00%│                               ││0.00 kB│                              │",
            "│      │                               ││       │                              │",
            "│      │                               ││       │                              │",
            "│      │                               ││       │                              │",
            "│      │                               ││       │                              │",
            "│      │                               ││       │                              │",
            "│      │                               ││       │                              │",
            "│      │                               ││       │                              │",
            "╰──────────────────────────────────────╯╰──────────────────────────────────────╯",
        ];

        let result = &setup.terminal.backend().buffer().content;
        for (row_index, row) in expected.iter().enumerate() {
            for (char_index, expected_char) in row.chars().enumerate() {
                let index = row_index * usize::from(w) + char_index;
                let result_cell = &result[index];

                assert_eq!(result_cell.symbol(), expected_char.to_string());

                match index {
                    // chart tiles - cpu 03.00% && memory 30.00 kB - are green
                    14..=25 | 52..=67 => {
                        assert_eq!(result_cell.fg, Color::Green);
                        assert_eq!(result_cell.modifier, Modifier::BOLD);
                    }
                    // Cpu & Memory max are orange and bold
                    81..=86 | 121..=127 => {
                        assert_eq!(result_cell.fg, ORANGE);
                        assert_eq!(result_cell.modifier, Modifier::BOLD);
                    }
                    // All others
                    _ => {
                        assert_eq!(result_cell.fg, Color::Reset);
                        assert!(result_cell.modifier.is_empty());
                    }
                }
            }
        }
    }

    #[test]
    /// When status is Running, charts correctly drawn
    fn test_draw_blocks_charts_running_some() {
        let (w, h) = (80, 10);
        let mut setup = test_setup(w, h, true, true);

        insert_chart_data(&setup);

        setup
            .terminal
            .draw(|f| {
                super::chart(f, setup.area, &setup.app_data);
            })
            .unwrap();

        let result = &setup.terminal.backend().buffer().content;
        for (row_index, row) in EXPECTED.iter().enumerate() {
            for (char_index, expected_char) in row.chars().enumerate() {
                let index = row_index * usize::from(w) + char_index;
                let result_cell = &result[index];

                assert_eq!(result_cell.symbol(), expected_char.to_string());
                match index {
                    // chart tiles - cpu 03.00% && memory 30.00 kB - are green
                    14..=25 | 51..=67 => {
                        assert_eq!(result_cell.fg, Color::Green);
                        assert_eq!(result_cell.modifier, Modifier::BOLD);
                    }
                    // Cpu & Memory max are orange and bold
                    81..=86 | 121..=129 => {
                        assert_eq!(result_cell.fg, ORANGE);
                        assert_eq!(result_cell.modifier, Modifier::BOLD);
                    }
                    // cpu dots are magenta
                    _x if CPU_INDEX.contains(&index) => {
                        assert_eq!(result_cell.fg, Color::Magenta);
                        assert!(result_cell.modifier.is_empty());
                    }
                    // memory dots are cyan
                    _x if MEMORY_INDEX.contains(&index) => {
                        assert_eq!(result_cell.fg, Color::Cyan);
                        assert!(result_cell.modifier.is_empty());
                    }
                    // All others
                    _ => {
                        assert_eq!(result_cell.fg, Color::Reset);
                        assert!(result_cell.modifier.is_empty());
                    }
                }
            }
        }
    }

    #[test]
    /// Whens status paused, some text is now Yellow
    fn test_draw_blocks_charts_paused() {
        let (w, h) = (80, 10);
        let mut setup = test_setup(w, h, true, true);

        insert_chart_data(&setup);
        setup.app_data.lock().containers.items[0].state = State::Paused;

        setup
            .terminal
            .draw(|f| {
                super::chart(f, setup.area, &setup.app_data);
            })
            .unwrap();

        let result = &setup.terminal.backend().buffer().content;
        for (row_index, row) in EXPECTED.iter().enumerate() {
            for (char_index, expected_char) in row.chars().enumerate() {
                let index = row_index * usize::from(w) + char_index;
                let result_cell = &result[index];

                assert_eq!(result_cell.symbol(), expected_char.to_string());
                match index {
                    // Titles and y axis are yellow
                    14..=25 | 51..=67 | 81..=86 | 121..=129 => {
                        assert_eq!(result_cell.fg, Color::Yellow);
                        assert_eq!(result_cell.modifier, Modifier::BOLD);
                    }
                    _x if CPU_INDEX.contains(&index) => {
                        assert_eq!(result_cell.fg, Color::Magenta);
                        assert!(result_cell.modifier.is_empty());
                    }
                    // memory dots are cyan
                    _x if MEMORY_INDEX.contains(&index) => {
                        assert_eq!(result_cell.fg, Color::Cyan);
                        assert!(result_cell.modifier.is_empty());
                    }
                    // All others
                    _ => {
                        assert_eq!(result_cell.fg, Color::Reset);
                        assert!(result_cell.modifier.is_empty());
                    }
                }
            }
        }
    }

    #[test]
    /// When dead, text is read
    fn test_draw_blocks_charts_dead() {
        let (w, h) = (80, 10);
        let mut setup = test_setup(w, h, true, true);
        insert_chart_data(&setup);
        setup.app_data.lock().containers.items[0].state = State::Dead;

        setup
            .terminal
            .draw(|f| {
                super::chart(f, setup.area, &setup.app_data);
            })
            .unwrap();

        let result = &setup.terminal.backend().buffer().content;
        for (row_index, row) in EXPECTED.iter().enumerate() {
            for (char_index, expected_char) in row.chars().enumerate() {
                let index = row_index * usize::from(w) + char_index;
                let result_cell = &result[index];

                assert_eq!(result_cell.symbol(), expected_char.to_string());
                match index {
                    // Titles and y axis are red
                    14..=25 | 51..=67 | 81..=86 | 121..=129 => {
                        assert_eq!(result_cell.fg, Color::Red);
                        assert_eq!(result_cell.modifier, Modifier::BOLD);
                    }
                    // cpu dots are magenta
                    _x if CPU_INDEX.contains(&index) => {
                        assert_eq!(result_cell.fg, Color::Magenta);
                        assert!(result_cell.modifier.is_empty());
                    }
                    // memory dots are cyan
                    _x if MEMORY_INDEX.contains(&index) => {
                        assert_eq!(result_cell.fg, Color::Cyan);
                        assert!(result_cell.modifier.is_empty());
                    }
                    // All others
                    _ => {
                        assert_eq!(result_cell.fg, Color::Reset);
                        assert!(result_cell.modifier.is_empty());
                    }
                }
            }
        }
    }

    // ******* //
    // Headers //
    // ******* //

    #[test]
    /// Heading back only has show/exit help when no containers, correctly coloured
    fn test_draw_blocks_headers_no_containers() {
        let (w, h) = (140, 1);
        let mut setup = test_setup(w, h, true, true);
        setup.app_data.lock().containers = StatefulList::new(vec![]);

        let mut fd = FrameData::from((setup.app_data.lock(), setup.gui_state.lock()));

        let expected =  "                                                                                                                         ( h ) show help    ";

        setup
            .terminal
            .draw(|f| {
                super::heading_bar(setup.area, f, &fd, &setup.gui_state);
            })
            .unwrap();

        let result = &setup.terminal.backend().buffer().content;
        for (index, expected_char) in expected.chars().enumerate() {
            let result_cell = &result[index];

            assert_eq!(result_cell.symbol(), expected_char.to_string());
            assert_eq!(result_cell.bg, Color::Magenta);
            assert_eq!(result_cell.fg, Color::White);
        }

        fd.help_visible = true;
        let expected =  "                                                                                                                         ( h ) exit help    ";
        setup
            .terminal
            .draw(|f| {
                super::heading_bar(setup.area, f, &fd, &setup.gui_state);
            })
            .unwrap();

        let result = &setup.terminal.backend().buffer().content;
        for (index, expected_char) in expected.chars().enumerate() {
            let result_cell = &result[index];

            assert_eq!(result_cell.symbol(), expected_char.to_string());
            assert_eq!(result_cell.bg, Color::Magenta);
            assert_eq!(result_cell.fg, Color::Black);
        }
    }

    #[test]
    /// Show all headings when containers present, colors valid
    fn test_draw_blocks_headers_some_containers() {
        let (w, h) = (140, 1);
        let mut setup = test_setup(w, h, true, true);
        let fd = FrameData::from((setup.app_data.lock(), setup.gui_state.lock()));

        let expected =   "           name       state               status       cpu        memory/limit         id     image      ↓ rx      ↑ tx    ( h ) show help  ";
        setup
            .terminal
            .draw(|f| {
                super::heading_bar(setup.area, f, &fd, &setup.gui_state);
            })
            .unwrap();

        let result = &setup.terminal.backend().buffer().content;
        for (index, expected_char) in expected.chars().enumerate() {
            let result_cell = &result[index];

            assert_eq!(result_cell.symbol(), expected_char.to_string());
            assert_eq!(result_cell.bg, Color::Magenta);
            assert_eq!(
                result_cell.fg,
                match index {
                    (2..=122) => Color::Black,
                    _ => Color::White,
                }
            );
        }
    }

    #[test]
    /// Only show the headings that fit the reduced-in-size header section
    fn test_draw_blocks_headers_some_containers_reduced_width() {
        let (w, h) = (80, 1);
        let mut setup = test_setup(w, h, true, true);
        let fd = FrameData::from((setup.app_data.lock(), setup.gui_state.lock()));

        let expected =
            "           name       state               status       cpu     ( h ) show help  ";
        setup
            .terminal
            .draw(|f| {
                super::heading_bar(setup.area, f, &fd, &setup.gui_state);
            })
            .unwrap();

        let result = &setup.terminal.backend().buffer().content;
        for (index, expected_char) in expected.chars().enumerate() {
            let result_cell = &result[index];

            assert_eq!(result_cell.symbol(), expected_char.to_string());
            assert_eq!(result_cell.bg, Color::Magenta);
            assert_eq!(
                result_cell.fg,
                match index {
                    (2..=62) => Color::Black,
                    _ => Color::White,
                }
            );
        }
    }

    #[test]
    /// Test all combination of headers & sort by
    fn test_draw_blocks_headers_sort_containers() {
        let (w, h) = (140, 1);
        let mut setup = test_setup(w, h, true, true);
        let mut fd = FrameData::from((setup.app_data.lock(), setup.gui_state.lock()));
        let mut test = |expected: &str, range: RangeInclusive<usize>, x: (Header, SortedOrder)| {
            fd.sorted_by = Some(x);

            setup
                .terminal
                .draw(|f| {
                    super::heading_bar(setup.area, f, &fd, &setup.gui_state);
                })
                .unwrap();

            let result = &setup.terminal.backend().buffer().content;
            for (index, expected_char) in expected.chars().enumerate() {
                let result_cell = &result[index];
                assert_eq!(result_cell.symbol(), expected_char.to_string());
                assert_eq!(result_cell.bg, Color::Magenta);
                assert_eq!(
                    result_cell.fg,
                    match index {
                        0 | 1 => Color::White,
                        // given range | help section
                        x if range.contains(&x) || (123..=139).contains(&x) => Color::White,
                        _ => Color::Black,
                    }
                );
            }
        };

        // Name
        test("         ▲ name       state               status       cpu        memory/limit         id     image      ↓ rx      ↑ tx    ( h ) show help  ", 1..=14, (Header::Name, SortedOrder::Asc));
        test("         ▼ name       state               status       cpu        memory/limit         id     image      ↓ rx      ↑ tx    ( h ) show help  ", 1..=14, (Header::Name, SortedOrder::Desc));

        // state
        test("           name     ▲ state               status       cpu        memory/limit         id     image      ↓ rx      ↑ tx    ( h ) show help  ", 15..=26, (Header::State, SortedOrder::Asc));
        test("           name     ▼ state               status       cpu        memory/limit         id     image      ↓ rx      ↑ tx    ( h ) show help  ", 15..=26, (Header::State, SortedOrder::Desc));

        // status
        test("           name       state             ▲ status       cpu        memory/limit         id     image      ↓ rx      ↑ tx    ( h ) show help  ", 27..=47, (Header::Status, SortedOrder::Asc));
        test("           name       state             ▼ status       cpu        memory/limit         id     image      ↓ rx      ↑ tx    ( h ) show help  ", 27..=47, (Header::Status, SortedOrder::Desc));

        // cpu
        test("           name       state               status     ▲ cpu        memory/limit         id     image      ↓ rx      ↑ tx    ( h ) show help  ", 48..=57, (Header::Cpu, SortedOrder::Asc));
        test("           name       state               status     ▼ cpu        memory/limit         id     image      ↓ rx      ↑ tx    ( h ) show help  ", 48..=57, (Header::Cpu, SortedOrder::Desc));

        // mem
        test("           name       state               status       cpu      ▲ memory/limit         id     image      ↓ rx      ↑ tx    ( h ) show help  ", 58..=77, (Header::Memory, SortedOrder::Asc));
        test("           name       state               status       cpu      ▼ memory/limit         id     image      ↓ rx      ↑ tx    ( h ) show help  ", 58..=77, (Header::Memory, SortedOrder::Desc));

        // id
        test("           name       state               status       cpu        memory/limit       ▲ id     image      ↓ rx      ↑ tx    ( h ) show help  ", 78..=88, (Header::Id, SortedOrder::Asc));
        test("           name       state               status       cpu        memory/limit       ▼ id     image      ↓ rx      ↑ tx    ( h ) show help  ", 78..=88, (Header::Id, SortedOrder::Desc));

        // image
        test("           name       state               status       cpu        memory/limit         id   ▲ image      ↓ rx      ↑ tx    ( h ) show help  ", 89..=98, (Header::Image, SortedOrder::Asc));
        test("           name       state               status       cpu        memory/limit         id   ▼ image      ↓ rx      ↑ tx    ( h ) show help  ", 89..=98, (Header::Image, SortedOrder::Desc));

        // rx
        test("           name       state               status       cpu        memory/limit         id     image    ▲ ↓ rx      ↑ tx    ( h ) show help  ", 99..=108, (Header::Rx, SortedOrder::Asc));
        test("           name       state               status       cpu        memory/limit         id     image    ▼ ↓ rx      ↑ tx    ( h ) show help  ", 99..=108, (Header::Rx, SortedOrder::Desc));

        // tx
        test("           name       state               status       cpu        memory/limit         id     image      ↓ rx    ▲ ↑ tx    ( h ) show help  ", 109..=118, (Header::Tx, SortedOrder::Asc));
        test("           name       state               status       cpu        memory/limit         id     image      ↓ rx    ▼ ↑ tx    ( h ) show help  ", 109..=118, (Header::Tx, SortedOrder::Desc));
    }

    #[test]
    /// Show animation
    fn test_draw_blocks_headers_animation() {
        let (w, h) = (140, 1);
        let mut setup = test_setup(w, h, true, true);
        let uuid = Uuid::new_v4();
        setup.gui_state.lock().next_loading(uuid);
        let fd = FrameData::from((setup.app_data.lock(), setup.gui_state.lock()));

        setup
            .terminal
            .draw(|f| {
                super::heading_bar(setup.area, f, &fd, &setup.gui_state);
            })
            .unwrap();

        let expected =   " ⠙         name       state               status       cpu        memory/limit         id     image      ↓ rx      ↑ tx    ( h ) show help  ";

        let result = &setup.terminal.backend().buffer().content;
        for (index, expected_char) in expected.chars().enumerate() {
            let result_cell = &result[index];

            assert_eq!(result_cell.symbol(), expected_char.to_string());
            assert_eq!(result_cell.bg, Color::Magenta);
            assert_eq!(
                result_cell.fg,
                match index {
                    (2..=122) => Color::Black,
                    _ => Color::White,
                }
            );
        }
    }

    // ********** //
    // Help popup //
    // ********** //
    #[test]
    /// This will cause issues once the version has more than the current 5 chars (0.5.0)
    // Help  popup is drawn correctly
    fn test_draw_blocks_help() {
        let (w, h) = (87, 32);
        let mut setup = test_setup(w, h, true, true);

        setup
            .terminal
            .draw(|f| {
                super::help_box(f);
            })
            .unwrap();
        let expected = [
            "                                                                                       ".to_owned(),
            format!(" ╭ {VERSION} ────────────────────────────────────────────────────────────────────────────╮ "),
            " │                                                                                   │ ".to_owned(),
            " │                                      88                                           │ ".to_owned(),
            " │                                      88                                           │ ".to_owned(),
            " │                                      88                                           │ ".to_owned(),
            " │             ,adPPYba,   8b,     ,d8  88   ,d8    ,adPPYba,  8b,dPPYba,            │ ".to_owned(),
            r#" │            a8"     "8a   `Y8, ,8P'   88 ,a8"    a8P_____88  88P'   "Y8            │ "#.to_owned(),
            r#" │            8b       d8     )888(     8888[      8PP"""""""  88                    │ "#.to_owned(),
            r#" │            "8a,   ,a8"   ,d8" "8b,   88`"Yba,   "8b,   ,aa  88                    │ "#.to_owned(),
            r#" │             `"YbbdP"'   8P'     `Y8  88   `Y8a   `"Ybbd8"'  88                    │ "#.to_owned(),
            " │                                                                                   │ ".to_owned(),
            " │                 A simple tui to view & control docker containers                  │ ".to_owned(),
            " │                                                                                   │ ".to_owned(),
            " │ ( tab ) or ( shift+tab ) change panels                                            │ ".to_owned(),
            " │ ( ↑ ↓ ) or ( j k ) or ( PgUp PgDown ) or ( Home End ) change selected line        │ ".to_owned(),
            " │ ( enter ) send docker container command                                           │ ".to_owned(),
            " │ ( e ) exec into a container                                                       │ ".to_owned(),
            " │ ( h ) toggle this help information                                                │ ".to_owned(),
            " │ ( s ) save logs to file                                                           │ ".to_owned(),
            " │ ( m ) toggle mouse capture - if disabled, text on screen can be selected & copied │ ".to_owned(),
            " │ ( 0 ) stop sort                                                                   │ ".to_owned(),
            " │ ( 1 - 9 ) sort by header - or click header                                        │ ".to_owned(),
            " │ ( esc ) close dialog                                                              │ ".to_owned(),
            " │ ( q ) quit at any time                                                            │ ".to_owned(),
            " │                                                                                   │ ".to_owned(),
            " │        currently an early work in progress, all and any input appreciated         │ ".to_owned(),
            " │                       https://github.com/mrjackwills/oxker                        │ ".to_owned(),
            " │                                                                                   │ ".to_owned(),
            " │                                                                                   │ ".to_owned(),
            " ╰───────────────────────────────────────────────────────────────────────────────────╯ ".to_owned(),
        ];

        for (row_index, row) in expected.iter().enumerate() {
            let mut bracket_key = vec![];
            let mut push_bracket_key = false;

            let result = &setup.terminal.backend().buffer().content;
            for (char_index, expected_char) in row.chars().enumerate() {
                let index = row_index * usize::from(w) + char_index;
                let result_cell = &result[index];
                let result_str = result_cell.symbol();

                assert_eq!(result_cell.symbol(), expected_char.to_string());

                // First and last row, and first char and last char in each row, is empty
                if row_index == 0
                    || row_index == usize::from(h - 1)
                    || char_index == 0
                    || char_index == usize::from(w - 1)
                {
                    assert_eq!(result_cell.fg, Color::Reset);
                    assert_eq!(result_cell.bg, Color::Reset);
                // Borders
                } else if BORDER_CHARS.contains(&result_str) {
                    assert_eq!(result_cell.fg, Color::Black);
                    assert_eq!(result_cell.bg, Color::Magenta);
                // everything else has a magenta background
                } else {
                    assert_eq!(result_cell.bg, Color::Magenta);
                }

                // check that ( [key] ) is white
                if result_str == "(" {
                    push_bracket_key = true;
                    bracket_key.push(result_cell);
                }
                if push_bracket_key {
                    bracket_key.push(result_cell);
                    if result_str == ")" {
                        push_bracket_key = false;
                        for i in &bracket_key {
                            assert_eq!(i.fg, Color::White);
                        }
                        bracket_key.clear();
                    }
                }
                // TODO should really be testing every color of every str here
            }
        }
    }

    // ************ //
    // Delete popup //
    // ************ //

    #[test]
    /// Delete container popup is drawn correctly
    fn test_draw_blocks_delete() {
        let (w, h) = (82, 10);
        let mut setup = test_setup(w, h, true, true);

        let expected = [
            "                                                                                  ",
            "        ╭──────────────────────── Confirm Delete ────────────────────────╮        ",
            "        │                                                                │        ",
            "        │     Are you sure you want to delete container: container_1     │        ",
            "        │                                                                │        ",
            "        │      ╭─────────────────────╮      ╭─────────────────────╮      │        ",
            "        │      │        (N)o         │      │        (Y)es        │      │        ",
            "        │      ╰─────────────────────╯      ╰─────────────────────╯      │        ",
            "        ╰────────────────────────────────────────────────────────────────╯        ",
            "                                                                                  ",
        ];

        setup
            .terminal
            .draw(|f| {
                super::delete_confirm(f, &setup.gui_state, &ContainerName::from("container_1"));
            })
            .unwrap();

        let result = &setup.terminal.backend().buffer().content;
        for (row_index, row) in expected.iter().enumerate() {
            for (char_index, expected_char) in row.chars().enumerate() {
                let index = row_index * usize::from(w) + char_index;
                let result_cell = &result[index];

                assert_eq!(result_cell.symbol(), expected_char.to_string());

                if row_index == 0
                    || row_index == usize::from(h - 1)
                    || char_index < 8
                    || char_index > usize::from(w - 9)
                {
                    assert_eq!(result_cell.fg, Color::Reset);
                    assert_eq!(result_cell.bg, Color::Reset);
                } else {
                    assert_eq!(result_cell.bg, Color::White);
                }

                // Borders are black
                if BORDER_CHARS.contains(&result_cell.symbol()) {
                    assert_eq!(result_cell.fg, Color::Black);
                    // Container name is red
                } else if row_index == 3 && (57..=67).contains(&char_index) {
                    assert_eq!(result_cell.fg, Color::Red);
                    // All other text is black
                } else if !row_index == 0
                    && !row_index == usize::from(h - 1)
                    && !char_index < 8
                    && !char_index > usize::from(w - 9)
                {
                    assert_eq!(result_cell.fg, Color::Black);
                }
            }
        }
    }

    #[test]
    /// Delete container popup is drawn correctly
    fn test_draw_blocks_delete_long_name() {
        let (w, h) = (106, 10);
        let mut setup = test_setup(w, h, true, true);
        let name = ContainerName::from("container_1_container_1_container_1");
        setup.app_data.lock().containers.items[0].name = name.clone();

        let expected = [
            "                                                                                                          ",
            "        ╭──────────────────────────────────── Confirm Delete ────────────────────────────────────╮        ",
            "        │                                                                                        │        ",
            "        │     Are you sure you want to delete container: container_1_container_1_container_1     │        ",
            "        │                                                                                        │        ",
            "        │        ╭──────────────────────────────╮         ╭─────────────────────────────╮        │        ",
            "        │        │             (N)o             │         │            (Y)es            │        │        ",
            "        │        ╰──────────────────────────────╯         ╰─────────────────────────────╯        │        ",
            "        ╰────────────────────────────────────────────────────────────────────────────────────────╯        ",
            "                                                                                                          ",
        ];

        setup
            .terminal
            .draw(|f| {
                super::delete_confirm(f, &setup.gui_state, &name);
            })
            .unwrap();

        let result = &setup.terminal.backend().buffer().content;
        for (row_index, row) in expected.iter().enumerate() {
            for (char_index, expected_char) in row.chars().enumerate() {
                let index = row_index * usize::from(w) + char_index;
                let result_cell = &result[index];
                assert_eq!(result_cell.symbol(), expected_char.to_string());

                if row_index == 0
                    || row_index == usize::from(h - 1)
                    || char_index < 8
                    || char_index > usize::from(w - 9)
                {
                    assert_eq!(result_cell.fg, Color::Reset);
                    assert_eq!(result_cell.bg, Color::Reset);
                } else {
                    assert_eq!(result_cell.bg, Color::White);
                }

                // Borders are black
                if BORDER_CHARS.contains(&result_cell.symbol()) {
                    assert_eq!(result_cell.fg, Color::Black);
                // Container name is red
                } else if row_index == 3 && (57..=82).contains(&char_index) {
                    assert_eq!(result_cell.fg, Color::Red);
                // All other text is black
                } else if !row_index == 0
                    && !row_index == usize::from(h - 1)
                    && !char_index < 8
                    && !char_index > usize::from(w - 9)
                {
                    assert_eq!(result_cell.fg, Color::Black);
                }
            }
        }
    }

    // ***** //
    // popup //
    // ***** //

    #[test]
    /// Info box drawn in bottom right
    fn test_draw_blocks_info() {
        let (w, h) = (45, 9);
        let mut setup = test_setup(w, h, true, true);

        let expected = [
            "                                             ",
            "                                             ",
            "                                             ",
            "                                             ",
            "                                             ",
            "                                             ",
            "                                             ",
            "                                    test     ",
            "                                             ",
        ];

        setup
            .terminal
            .draw(|f| {
                super::info(f, "test", std::time::Instant::now(), &setup.gui_state);
            })
            .unwrap();

        let result = &setup.terminal.backend().buffer().content;

        for (row_index, row) in expected.iter().enumerate() {
            for (char_index, expected_char) in row.chars().enumerate() {
                let index = row_index * usize::from(w) + char_index;
                let result_cell = &result[index];

                assert_eq!(expected_char.to_string(), result_cell.symbol());
                let (fg, bg) = if row_index >= 6 && char_index >= 32 {
                    (Color::White, Color::Blue)
                } else {
                    (Color::Reset, Color::Reset)
                };

                assert_eq!(result_cell.fg, fg);
                assert_eq!(result_cell.bg, bg);
            }
        }
    }

    // *********** //
    // Error popup //
    // *********** //

    #[test]
    /// Test that the error popup is centered, red background, white border, white text, and displays the correct text
    fn test_draw_blocks_docker_connect_error() {
        let (w, h) = (46, 9);
        let mut setup = test_setup(w, h, true, true);

        setup
            .terminal
            .draw(|f| {
                super::error(f, AppError::DockerConnect, Some(4));
            })
            .unwrap();

        let expected = vec![
            "                                              ".to_owned(),
            " ╭───────────────── Error ──────────────────╮ ".to_owned(),
            " │                                          │ ".to_owned(),
            " │      Unable to access docker daemon      │ ".to_owned(),
            " │                                          │ ".to_owned(),
            format!(" │    oxker::v{VERSION} closing in 04 seconds   │ "),
            " │                                          │ ".to_owned(),
            " ╰──────────────────────────────────────────╯ ".to_owned(),
            "                                              ".to_owned(),
        ];

        let result = &setup.terminal.backend().buffer().content;
        for (row_index, row) in expected.iter().enumerate() {
            for (char_index, expected_char) in row.chars().enumerate() {
                let index = row_index * usize::from(w) + char_index;
                let result_cell = &result[index];

                assert_eq!(result_cell.symbol(), expected_char.to_string());

                if (1..=usize::from(h) - 2).contains(&row_index)
                    && (1..=usize::from(w) - 2).contains(&char_index)
                {
                    assert_eq!(result_cell.bg, Color::Red);
                }
                if result_cell
                    .symbol()
                    .chars()
                    .next()
                    .unwrap()
                    .is_alphanumeric()
                {
                    assert_eq!(result_cell.fg, Color::White);
                }
            }
        }
    }

    #[test]
    /// Test that the clearable error popup is centered, red background, white border, white text, and displays the correct text
    fn test_draw_blocks_clearable_error() {
        let (w, h) = (39, 10);
        let mut setup = test_setup(w, h, true, true);

        setup
            .terminal
            .draw(|f| {
                super::error(f, AppError::DockerExec, Some(4));
            })
            .unwrap();

        let expected = [
            "                                       ",
            " ╭────────────── Error ──────────────╮ ",
            " │                                   │ ",
            " │   Unable to exec into container   │ ",
            " │                                   │ ",
            " │         ( c ) clear error         │ ",
            " │         ( q ) quit oxker          │ ",
            " │                                   │ ",
            " ╰───────────────────────────────────╯ ",
            "                                       ",
        ];

        let result = &setup.terminal.backend().buffer().content;
        for (row_index, row) in expected.iter().enumerate() {
            for (char_index, expected_char) in row.chars().enumerate() {
                let index = row_index * usize::from(w) + char_index;
                let result_cell = &result[index];

                assert_eq!(result_cell.symbol(), expected_char.to_string());
                if (1..=usize::from(h) - 2).contains(&row_index)
                    && (1..=usize::from(w) - 2).contains(&char_index)
                {
                    assert_eq!(result_cell.bg, Color::Red);
                }
                if result_cell
                    .symbol()
                    .chars()
                    .next()
                    .unwrap()
                    .is_alphanumeric()
                    || ["(", ")"].contains(&result_cell.symbol())
                {
                    assert_eq!(result_cell.fg, Color::White);
                }
            }
        }
    }

    #[test]
    /// Port section when container has no ports
    fn test_draw_blocks_ports_no_ports() {
        let (w, h) = (30, 8);
        let mut setup = test_setup(w, h, true, true);
        setup.app_data.lock().containers.items[0].ports = vec![];

        let max_lens = setup.app_data.lock().get_longest_port();
        setup
            .terminal
            .draw(|f| {
                super::ports(f, setup.area, &setup.app_data, max_lens);
            })
            .unwrap();

        let expected = [
            "╭────────── ports ───────────╮",
            "│          no ports          │",
            "│                            │",
            "│                            │",
            "│                            │",
            "│                            │",
            "│                            │",
            "╰────────────────────────────╯",
        ];

        let result = &setup.terminal.backend().buffer().content;
        for (row_index, row) in expected.iter().enumerate() {
            for (char_index, expected_char) in row.chars().enumerate() {
                let index = row_index * usize::from(w) + char_index;
                let result_cell = &result[index];

                assert_eq!(expected_char.to_string(), result_cell.symbol());
                if row_index == 0 && !BORDER_CHARS.contains(&result_cell.symbol()) {
                    assert_eq!(result_cell.fg, Color::Green);
                    assert_eq!(result_cell.modifier, Modifier::BOLD);
                } else {
                    assert_eq!(result_cell.fg, Color::Reset);
                }
            }
        }

        // when state is "State::Running | State::Paused | State::Restarting, won't show "no ports"
        setup.app_data.lock().containers.items[0].state = State::Dead;
        let max_lens = setup.app_data.lock().get_longest_port();
        setup
            .terminal
            .draw(|f| {
                super::ports(f, setup.area, &setup.app_data, max_lens);
            })
            .unwrap();

        let expected = [
            "╭────────── ports ───────────╮",
            "│                            │",
            "│                            │",
            "│                            │",
            "│                            │",
            "│                            │",
            "│                            │",
            "╰────────────────────────────╯",
        ];

        let result = &setup.terminal.backend().buffer().content;
        for (row_index, row) in expected.iter().enumerate() {
            for (char_index, expected_char) in row.chars().enumerate() {
                let index = row_index * usize::from(w) + char_index;
                let result_cell = &result[index];

                assert_eq!(expected_char.to_string(), result_cell.symbol());
                if row_index == 0 && !BORDER_CHARS.contains(&result_cell.symbol()) {
                    assert_eq!(result_cell.fg, Color::Red);
                    assert_eq!(result_cell.modifier, Modifier::BOLD);
                } else {
                    assert_eq!(result_cell.fg, Color::Reset);
                }
            }
        }
    }

    #[test]
    /// Port section when container has multiple ports
    fn test_draw_blocks_ports_multiple_ports() {
        let (w, h) = (32, 8);
        let mut setup = test_setup(w, h, true, true);
        setup.app_data.lock().containers.items[0]
            .ports
            .push(ContainerPorts {
                ip: None,
                private: 8002,
                public: None,
            });
        setup.app_data.lock().containers.items[0]
            .ports
            .push(ContainerPorts {
                ip: Some("127.0.0.1".to_owned()),
                private: 8003,
                public: Some(8003),
            });

        let max_lens = setup.app_data.lock().get_longest_port();

        setup
            .terminal
            .draw(|f| {
                super::ports(f, setup.area, &setup.app_data, max_lens);
            })
            .unwrap();

        let expected = [
            "╭─────────── ports ────────────╮",
            "│       ip   private   public  │",
            "│               8001           │",
            "│               8002           │",
            "│127.0.0.1      8003     8003  │",
            "│                              │",
            "│                              │",
            "╰──────────────────────────────╯",
        ];

        let result = &setup.terminal.backend().buffer().content;
        for (row_index, row) in expected.iter().enumerate() {
            for (char_index, expected_char) in row.chars().enumerate() {
                let index = row_index * usize::from(w) + char_index;
                let result_cell = &result[index];

                assert_eq!(expected_char.to_string(), result_cell.symbol());

                let result_cell_as_char = result_cell
                    .symbol()
                    .chars()
                    .next()
                    .unwrap()
                    .is_ascii_alphanumeric();
                if row_index == 0 && result_cell_as_char {
                    assert_eq!(result_cell.fg, Color::Green);
                }
                if row_index == 1 && result_cell_as_char {
                    assert_eq!(result_cell.fg, Color::Yellow);
                }
                if (2..=3).contains(&row_index) && result_cell_as_char {
                    assert_eq!(result_cell.fg, Color::White);
                }
                if row_index == 4 && result_cell_as_char {
                    assert_eq!(result_cell.fg, Color::White);
                }
            }
        }
    }

    #[test]
    /// Port section title color correct dependant on state
    fn test_draw_blocks_ports_container_state() {
        let (w, h) = (32, 8);
        let mut setup = test_setup(w, h, true, true);
        let max_lens = setup.app_data.lock().get_longest_port();

        setup.app_data.lock().containers.items[0].state = State::Paused;
        setup
            .terminal
            .draw(|f| {
                super::ports(f, setup.area, &setup.app_data, max_lens);
            })
            .unwrap();

        let expected = [
            "╭─────────── ports ────────────╮",
            "│   ip   private   public      │",
            "│           8001               │",
            "│                              │",
            "│                              │",
            "│                              │",
            "│                              │",
            "╰──────────────────────────────╯",
        ];

        let result = &setup.terminal.backend().buffer().content;
        for (row_index, row) in expected.iter().enumerate() {
            for (char_index, expected_char) in row.chars().enumerate() {
                let index = row_index * usize::from(w) + char_index;
                let result_cell = &result[index];

                assert_eq!(expected_char.to_string(), result_cell.symbol());

                if row_index == 0
                    && result_cell
                        .symbol()
                        .chars()
                        .next()
                        .unwrap()
                        .is_ascii_alphanumeric()
                {
                    assert_eq!(result_cell.fg, Color::Yellow);
                }
            }
        }

        setup.app_data.lock().containers.items[0].state = State::Dead;
        setup
            .terminal
            .draw(|f| {
                super::ports(f, setup.area, &setup.app_data, max_lens);
            })
            .unwrap();

        // This is wrong - why?
        let expected = [
            "╭─────────── ports ────────────╮",
            "│   ip   private   public      │",
            "│           8001               │",
            "│                              │",
            "│                              │",
            "│                              │",
            "│                              │",
            "╰──────────────────────────────╯",
        ];

        let result = &setup.terminal.backend().buffer().content;
        for (row_index, row) in expected.iter().enumerate() {
            for (char_index, expected_char) in row.chars().enumerate() {
                let index = row_index * usize::from(w) + char_index;
                let result_cell = &result[index];

                assert_eq!(expected_char.to_string(), result_cell.symbol());

                if row_index == 0
                    && result_cell
                        .symbol()
                        .chars()
                        .next()
                        .unwrap()
                        .is_ascii_alphanumeric()
                {
                    assert_eq!(result_cell.fg, Color::Red);
                }
            }
        }
    }

    // *************** //
    // The whole layout //
    // **************** //
    #[test]
    /// Check that the whole layout is drawn correctly
    fn test_draw_blocks_whole_layout() {
        let (w, h) = (160, 30);
        let mut setup = test_setup(w, h, true, true);

        insert_chart_data(&setup);
        insert_logs(&setup);
        setup.app_data.lock().containers.items[0]
            .ports
            .push(ContainerPorts {
                ip: Some("127.0.0.1".to_owned()),
                private: 8003,
                public: Some(8003),
            });

        let expected = [
            "           name       state               status       cpu          memory/limit         id     image      ↓ rx      ↑ tx                      ( h ) show help  ",
        "╭ Containers 1/3 ──────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────╮╭──────────────╮",
        "│⚪  container_1   ✓ running            Up 1 hour    03.00%   30.00 kB / 30.00 kB          1   image_1   0.00 kB   0.00 kB                      ││▶ pause       │",
        "│   container_2   ✓ running            Up 2 hour    00.00%    0.00 kB /  0.00 kB          2   image_2   0.00 kB   0.00 kB                      ││  restart     │",
        "│   container_3   ✓ running            Up 3 hour    00.00%    0.00 kB /  0.00 kB          3   image_3   0.00 kB   0.00 kB                      ││  stop        │",
        "│                                                                                                                                              ││  delete      │",
        "│                                                                                                                                              ││              │",
        "│                                                                                                                                              ││              │",
        "╰──────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────╯╰──────────────╯",
        "╭ Logs 3/3 - container_1 ──────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────╮",
        "│  line 1                                                                                                                                                      │",
        "│  line 2                                                                                                                                                      │",
        "│▶ line 3                                                                                                                                                      │",
        "│                                                                                                                                                              │",
        "│                                                                                                                                                              │",
        "│                                                                                                                                                              │",
        "│                                                                                                                                                              │",
        "│                                                                                                                                                              │",
        "│                                                                                                                                                              │",
        "│                                                                                                                                                              │",
        "│                                                                                                                                                              │",
        "│                                                                                                                                                              │",
        "│                                                                                                                                                              │",
        "╰──────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────╯",
        "╭───────────────────────── cpu 03.00% ──────────────────────────╮╭─────────────────────── memory 30.00 kB ───────────────────────╮╭────────── ports ───────────╮",
        "│10.00%│     ••••                                               ││100.00 kB│     •••                                             ││       ip   private   public│",
        "│      │  •••   •                                               ││         │  •••  •                                             ││               8001         │",
        "│      │••       •••                                            ││         │••      •••                                          ││127.0.0.1      8003     8003│",
        "│      │                                                        ││         │                                                     ││                            │",
        "╰───────────────────────────────────────────────────────────────╯╰───────────────────────────────────────────────────────────────╯╰────────────────────────────╯",
        ];
        setup
            .terminal
            .draw(|f| {
                draw_frame(f, &setup.app_data, &setup.gui_state);
            })
            .unwrap();

        let result = &setup.terminal.backend().buffer().content;
        for (row_index, row) in expected.iter().enumerate() {
            for (char_index, expected_char) in row.chars().enumerate() {
                let index = row_index * usize::from(w) + char_index;
                let result_cell = &result[index];

                assert_eq!(result_cell.symbol(), expected_char.to_string(),);
            }
        }
    }

    #[test]
    /// Check that the whole layout is drawn correctly when have long container name and long image name
    fn test_draw_blocks_whole_layout_long_name() {
        let (w, h) = (190, 30);
        let mut setup = test_setup(w, h, true, true);

        insert_chart_data(&setup);
        insert_logs(&setup);
        setup.app_data.lock().containers.items[0]
            .ports
            .push(ContainerPorts {
                ip: Some("127.0.0.1".to_owned()),
                private: 8003,
                public: Some(8003),
            });

        setup.app_data.lock().containers.items[0].name =
            ContainerName::from("a_long_container_name_for_the_purposes_of_this_test");
        setup.app_data.lock().containers.items[0].image =
            ContainerImage::from("a_long_image_name_for_the_purposes_of_this_test");

        let expected = [
        "                              name       state               status       cpu          memory/limit         id                            image      ↓ rx      ↑ tx          ( h ) show help  ",
        "╭ Containers 1/3 ─────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────╮╭─────────────────╮",
        "│⚪  a_long_container_name_for_the…   ✓ running            Up 1 hour    03.00%   30.00 kB / 30.00 kB          1   a_long_image_name_for_the_pur…   0.00 kB   0.00 kB       ││▶ pause          │",
        "│                      container_2   ✓ running            Up 2 hour    00.00%    0.00 kB /  0.00 kB          2                          image_2   0.00 kB   0.00 kB       ││  restart        │",
        "│                      container_3   ✓ running            Up 3 hour    00.00%    0.00 kB /  0.00 kB          3                          image_3   0.00 kB   0.00 kB       ││  stop           │",
        "│                                                                                                                                                                         ││  delete         │",
        "│                                                                                                                                                                         ││                 │",
        "│                                                                                                                                                                         ││                 │",
        "╰─────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────╯╰─────────────────╯",
        "╭ Logs 3/3 - a_long_container_name_for_the_purposes_of_this_test ────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────╮",
        "│  line 1                                                                                                                                                                                    │",
        "│  line 2                                                                                                                                                                                    │",
        "│▶ line 3                                                                                                                                                                                    │",
        "│                                                                                                                                                                                            │",
        "│                                                                                                                                                                                            │",
        "│                                                                                                                                                                                            │",
        "│                                                                                                                                                                                            │",
        "│                                                                                                                                                                                            │",
        "│                                                                                                                                                                                            │",
        "│                                                                                                                                                                                            │",
        "│                                                                                                                                                                                            │",
        "│                                                                                                                                                                                            │",
        "│                                                                                                                                                                                            │",
        "╰────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────╯",
        "╭───────────────────────────────── cpu 03.00% ─────────────────────────────────╮╭────────────────────────────── memory 30.00 kB ───────────────────────────────╮╭────────── ports ───────────╮",
        "│10.00%│       ••••                                                            ││100.00 kB│      •••••                                                         ││       ip   private   public│",
        "│      │   ••••   •                                                            ││         │   •••    •                                                         ││               8001         │",
        "│      │•••        ••••                                                        ││         │•••        •••                                                      ││127.0.0.1      8003     8003│",
        "│      │                                                                       ││         │                                                                    ││                            │",
        "╰──────────────────────────────────────────────────────────────────────────────╯╰──────────────────────────────────────────────────────────────────────────────╯╰────────────────────────────╯",
        ];
        setup
            .terminal
            .draw(|f| {
                draw_frame(f, &setup.app_data, &setup.gui_state);
            })
            .unwrap();

        let result = &setup.terminal.backend().buffer().content;
        for (row_index, row) in expected.iter().enumerate() {
            for (char_index, expected_char) in row.chars().enumerate() {
                let index = row_index * usize::from(w) + char_index;
                let result_cell = &result[index];

                assert_eq!(result_cell.symbol(), expected_char.to_string(),);
            }
        }
    }
}
