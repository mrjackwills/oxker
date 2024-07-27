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

use crate::app_data::{ContainerItem, ContainerName, FilterBy, Header, SortedOrder};
use crate::{
    app_data::{AppData, ByteStats, Columns, CpuStats, State, Stats},
    app_error::AppError,
};

use super::{
    gui_state::{BoxLocation, DeleteButton, Region},
    FrameData, Status,
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
    if fd.selected_panel == panel && !gui_state.lock().status_contains(&[Status::Filter]) {
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
                "{:<width$}{MARGIN}",
                i.name.to_string(),
                width = widths.name.1.into()
            ),
            blue,
        ),
        Span::styled(
            format!(
                "{:<width$}{MARGIN}",
                i.state.to_string(),
                width = widths.state.1.into()
            ),
            state_style,
        ),
        Span::styled(
            format!(
                "{:<width$}{MARGIN}",
                i.status,
                width = &widths.status.1.into()
            ),
            state_style,
        ),
        Span::styled(
            format!(
                "{:>width$}{MARGIN}",
                i.cpu_stats.back().unwrap_or(&CpuStats::default()),
                width = &widths.cpu.1.into()
            ),
            state_style,
        ),
        Span::styled(
            format!(
                "{:>width_current$} / {:>width_limit$}{MARGIN}",
                i.mem_stats.back().unwrap_or(&ByteStats::default()),
                i.mem_limit,
                width_current = &widths.mem.1.into(),
                width_limit = &widths.mem.2.into()
            ),
            state_style,
        ),
        Span::styled(
            format!(
                "{:>width$}{MARGIN}",
                i.id.get_short(),
                width = &widths.id.1.into()
            ),
            blue,
        ),
        Span::styled(
            format!(
                "{:<width$}{MARGIN}",
                i.image.to_string(),
                width = widths.image.1.into()
            ),
            blue,
        ),
        Span::styled(
            format!("{:>width$}{MARGIN}", i.rx, width = widths.net_rx.1.into()),
            Style::default().fg(Color::Rgb(255, 233, 193)),
        ),
        Span::styled(
            format!("{:>width$}{MARGIN}", i.tx, width = widths.net_tx.1.into()),
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
        let text = if app_data.lock().get_filter_term().is_some() {
            "no containers match filter"
        } else if gui_state.lock().is_loading() {
            &format!("loading {}", fd.loading_icon)
        } else {
            "no containers running"
        };

        let paragraph = Paragraph::new(text)
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

/// Create the filter_by by spans, coloured dependant on which one is selected
fn filter_by_spans(app_data: &Arc<Mutex<AppData>>) -> [Span; 4] {
    let filter_by = app_data.lock().get_filter_by();

    let selected = Style::default().bg(Color::Gray).fg(Color::Black);
    let not_selected = Style::default().bg(Color::Reset).fg(Color::Reset);

    // This should be refactored somehow
    let name = [" Name ", " Image ", " Status ", " All "];

    match filter_by {
        FilterBy::Name => [
            Span::styled(name[0], selected),
            Span::styled(name[1], not_selected),
            Span::styled(name[2], not_selected),
            Span::styled(name[3], not_selected),
        ],
        FilterBy::Image => [
            Span::styled(name[0], not_selected),
            Span::styled(name[1], selected),
            Span::styled(name[2], not_selected),
            Span::styled(name[3], not_selected),
        ],
        FilterBy::Status => [
            Span::styled(name[0], not_selected),
            Span::styled(name[1], not_selected),
            Span::styled(name[2], selected),
            Span::styled(name[3], not_selected),
        ],
        FilterBy::All => [
            Span::styled(name[0], not_selected),
            Span::styled(name[1], not_selected),
            Span::styled(name[2], not_selected),
            Span::styled(name[3], selected),
        ],
    }
}

/// Draw the filter bar
pub fn filter_bar(area: Rect, frame: &mut Frame, app_data: &Arc<Mutex<AppData>>) {
    let style_but = Style::default().fg(Color::Black).bg(Color::Magenta);
    let style_desc = Style::default().fg(Color::Gray).bg(Color::Reset);

    let mut line = vec![
        Span::styled(" Esc ", style_but),
        Span::styled(" clear ", style_desc),
        Span::styled(" ← by → ", style_but),
        Span::from(" "),
    ];
    line.extend_from_slice(&filter_by_spans(app_data));
    line.extend_from_slice(&[
        Span::styled(
            " term: ",
            Style::default()
                .fg(Color::Magenta)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            app_data
                .lock()
                .get_filter_term()
                .map_or(String::new(), std::borrow::ToOwned::to_owned),
            Style::default().fg(Color::Gray),
        ),
    ]);
    frame.render_widget(Line::from(line), area);
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
        let mut suffix = "";
        if let Some((a, b)) = &data.sorted_by {
            if x == a {
                match b {
                    SortedOrder::Asc => suffix = " ▲",
                    SortedOrder::Desc => suffix = " ▼",
                }
                color = Color::Gray;
            };
        };

        (Block::default().style(Style::default().fg(color)), suffix)
    };

    // Generate block for the headers, state and status has a specific layout, others all equal
    // width is dependant on it that column is selected to sort - or not
    let gen_header = |header: &Header, width: usize| {
        let block = header_block(header);

        // Yes this is a mess, needs documenting correctly

        let text = format!(
            "{x:<width$}{MARGIN}",
            x = format!("{header}{ic}", ic = block.1),
        );
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
    let info_text = format!("( h ) {suffix} help{MARGIN}",);
    let info_width = info_text.chars().count();

    let column_width = usize::from(area.width).saturating_sub(info_width);
    let column_width = if column_width > 0 { column_width } else { 1 };
    let splits = if data.has_containers {
        vec![
            Constraint::Max(4),
            Constraint::Max(column_width.try_into().unwrap_or_default()),
            Constraint::Max(info_width.try_into().unwrap_or_default()),
        ]
    } else {
        CONSTRAINT_100.to_vec()
    };

    let split_bar = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(splits)
        .split(area);

    // Draw loading icon, or not, and a prefix with a single space
    let loading_paragraph = Paragraph::new(format!("{:>2}", data.loading_icon))
        .block(block(Color::White))
        .alignment(Alignment::Left);
    frame.render_widget(loading_paragraph, split_bar[0]);
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
            Line::from(vec![
                space(),
                button_item("F1"),
                or(),
                button_item("/"),
                button_desc("enter filter mode"),
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
// TODO is this broken - I don't think so
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

    /// Get a single row of String's from the expected data
    fn expected_to_vec(expected: &[&str], row_index: usize) -> Vec<String> {
        expected[row_index]
            .chars()
            .map(|i| i.to_string())
            .collect::<Vec<_>>()
    }

    fn get_result(
        setup: &TuiTestSetup,
        w: u16,
    ) -> std::iter::Enumerate<std::slice::Chunks<ratatui::buffer::Cell>> {
        setup
            .terminal
            .backend()
            .buffer()
            .content
            .chunks(usize::from(w))
            .enumerate()
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

        for (row_index, row) in get_result(&setup, w) {
            let expected_row = expected_to_vec(&expected, row_index);
            for (cell_index, cell) in row.iter().enumerate() {
                assert_eq!(cell.symbol(), expected_row[cell_index]);
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

        for (row_index, result_row) in get_result(&setup, w) {
            let expected_row = expected_to_vec(&expected, row_index);
            for (result_cell_index, result_cell) in result_row.iter().enumerate() {
                assert_eq!(result_cell.symbol(), expected_row[result_cell_index]);
                assert_eq!(result_cell.bg, Color::Reset);
                match (row_index, result_cell_index) {
                    // pause
                    (1, 3..=7) => {
                        assert_eq!(result_cell.fg, Color::Yellow);
                    }
                    // restart
                    (2, 3..=9) => {
                        assert_eq!(result_cell.fg, Color::Magenta);
                    }
                    // stop
                    (3, 3..=6) => {
                        assert_eq!(result_cell.fg, Color::Red);
                    }
                    // delete
                    (4, 3..=8) => {
                        assert_eq!(result_cell.fg, Color::Gray);
                    }
                    _ => {
                        assert_eq!(result_cell.fg, Color::Reset);
                    }
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

        for (row_index, result_row) in get_result(&setup, w) {
            let expected_row = expected_to_vec(&expected, row_index);
            for (result_cell_index, result_cell) in result_row.iter().enumerate() {
                assert_eq!(result_cell.symbol(), expected_row[result_cell_index]);
                assert_eq!(result_cell.bg, Color::Reset);
                match (row_index, result_cell_index) {
                    // resume
                    (1, 3..=8) => {
                        assert_eq!(result_cell.fg, Color::Blue);
                    }
                    // stop
                    (2, 3..=6) => {
                        assert_eq!(result_cell.fg, Color::Red);
                    }
                    // delete
                    (3, 3..=8) => {
                        assert_eq!(result_cell.fg, Color::Gray);
                    }
                    _ => {
                        assert_eq!(result_cell.fg, Color::Reset);
                    }
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

        for (row_index, result_row) in get_result(&setup, w) {
            let expected_row = expected_to_vec(&expected, row_index);
            for (result_cell_index, result_cell) in result_row.iter().enumerate() {
                assert_eq!(result_cell.symbol(), expected_row[result_cell_index]);
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

        for (row_index, result_row) in get_result(&setup, w) {
            let expected_row = expected_to_vec(&expected, row_index);
            for (result_cell_index, result_cell) in result_row.iter().enumerate() {
                assert_eq!(result_cell.symbol(), expected_row[result_cell_index]);
                if row_index == 0
                    || row_index == 5
                    || result_cell_index == 0
                    || result_cell_index == 11
                {
                    assert_eq!(result_cell.fg, Color::LightCyan);
                }
                if row_index == 1 && result_cell_index > 0 && result_cell_index < 11 {
                    assert_eq!(result_cell.modifier, Modifier::BOLD);
                } else {
                    assert!(result_cell.modifier.is_empty());
                }
            }
        }
    }

    // *********************** //
    // Container summary panel //
    // *********************** //

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

        for (row_index, result_row) in get_result(&setup, w) {
            let expected_row = expected_to_vec(&expected, row_index);
            for (result_cell_index, result_cell) in result_row.iter().enumerate() {
                assert_eq!(result_cell.symbol(), expected_row[result_cell_index]);
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

        for (row_index, result_row) in get_result(&setup, w) {
            let expected_row = expected_to_vec(&expected, row_index);
            for (result_cell_index, result_cell) in result_row.iter().enumerate() {
                assert_eq!(result_cell.symbol(), expected_row[result_cell_index]);
                if BORDER_CHARS.contains(&result_cell.symbol()) {
                    assert_eq!(result_cell.fg, Color::LightCyan);
                }
            }
        }
    }

    #[test]
    /// Containers panel drawn, selected line is bold, border is blue
    fn test_draw_blocks_containers_selected_bold() {
        let (w, h) = (130, 6);
        let mut setup = test_setup(w, h, true, true);

        let expected = [
            "╭ Containers 1/3 ────────────────────────────────────────────────────────────────────────────────────────────────────────────────╮",
            "│⚪  container_1   ✓ running   Up 1 hour   00.00%   0.00 kB / 0.00 kB          1   image_1   0.00 kB   0.00 kB                    │",
            "│   container_2   ✓ running   Up 2 hour   00.00%   0.00 kB / 0.00 kB          2   image_2   0.00 kB   0.00 kB                    │",
            "│   container_3   ✓ running   Up 3 hour   00.00%   0.00 kB / 0.00 kB          3   image_3   0.00 kB   0.00 kB                    │",
            "│                                                                                                                                │",
            "╰────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────╯",
        ];

        setup
            .terminal
            .draw(|f| {
                super::containers(&setup.app_data, setup.area, f, &setup.fd, &setup.gui_state);
            })
            .unwrap();

        for (row_index, result_row) in get_result(&setup, w) {
            let expected_row = expected_to_vec(&expected, row_index);
            for (result_cell_index, result_cell) in result_row.iter().enumerate() {
                assert_eq!(result_cell.symbol(), expected_row[result_cell_index]);

                if BORDER_CHARS.contains(&result_cell.symbol()) {
                    assert_eq!(result_cell.fg, Color::LightCyan);
                }

                let not_bold = || assert!(result_cell.modifier.is_empty());
                if row_index == 1 {
                    match result_cell_index {
                        0 | 2 | 129 => {
                            not_bold();
                        }
                        _ => {
                            assert_eq!(result_cell.modifier, Modifier::BOLD);
                        }
                    }
                } else {
                    not_bold();
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

        for (row_index, result_row) in get_result(&setup, w) {
            let expected_row = expected_to_vec(&expected, row_index);
            for (result_cell_index, result_cell) in result_row.iter().enumerate() {
                assert_eq!(result_cell.symbol(), expected_row[result_cell_index]);

                if BORDER_CHARS.contains(&result_cell.symbol()) {
                    assert_eq!(result_cell.fg, Color::Reset);
                }
            }
        }
    }

    #[test]
    /// Columns on all rows are coloured correctly
    fn test_draw_blocks_containers_colors() {
        let (w, h) = (130, 6);
        let mut setup = test_setup(w, h, true, true);

        let expected = [
            "╭ Containers 1/3 ────────────────────────────────────────────────────────────────────────────────────────────────────────────────╮",
            "│⚪  container_1   ✓ running   Up 1 hour   00.00%   0.00 kB / 0.00 kB          1   image_1   0.00 kB   0.00 kB                    │",
            "│   container_2   ✓ running   Up 2 hour   00.00%   0.00 kB / 0.00 kB          2   image_2   0.00 kB   0.00 kB                    │",
            "│   container_3   ✓ running   Up 3 hour   00.00%   0.00 kB / 0.00 kB          3   image_3   0.00 kB   0.00 kB                    │",
            "│                                                                                                                                │",
            "╰────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────╯"
        ];
        let fd = FrameData::from((setup.app_data.lock(), setup.gui_state.lock()));

        setup
            .terminal
            .draw(|f| {
                super::containers(&setup.app_data, setup.area, f, &fd, &setup.gui_state);
            })
            .unwrap();

        for (row_index, result_row) in get_result(&setup, w) {
            let expected_row = expected_to_vec(&expected, row_index);

            for (result_cell_index, result_cell) in result_row.iter().enumerate() {
                assert_eq!(result_cell.symbol(), expected_row[result_cell_index]);

                match (row_index, result_cell_index) {
                    //border
                    (0 | 5, _) | (1..=4, 0 | 129) => {
                        assert_eq!(result_cell.fg, Color::LightCyan);
                    }
                    // name, id, image column
                    (1..=3, 4..=17 | 71..=91) => {
                        assert_eq!(result_cell.fg, Color::Blue);
                    }
                    // state, status, cpu, memory column
                    (1..=3, 18..=70) => {
                        assert_eq!(result_cell.fg, Color::Green);
                    }
                    // rx column
                    (1..=3, 92..=101) => {
                        assert_eq!(result_cell.fg, Color::Rgb(255, 233, 193));
                    }
                    // tx column
                    (1..=3, 102..=111) => {
                        assert_eq!(result_cell.fg, Color::Rgb(205, 140, 140));
                    }
                    _ => assert_eq!(result_cell.fg, Color::Reset),
                }
            }
        }
    }

    #[test]
    /// Long container + image name is truncated correctly
    fn test_draw_blocks_containers_long_name_image() {
        let (w, h) = (170, 6);
        let mut setup = test_setup(w, h, true, true);
        setup.app_data.lock().containers.items[0].name =
            ContainerName::from("a_long_container_name_for_the_purposes_of_this_test");
        setup.app_data.lock().containers.items[0].image =
            ContainerImage::from("a_long_image_name_for_the_purposes_of_this_test");

        let expected = [
            "╭ Containers 1/3 ────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────╮",
            "│⚪  a_long_container_name_for_the…   ॥ paused    Up 1 hour   00.00%   0.00 kB / 0.00 kB          1   a_long_image_name_for_the_pur…   0.00 kB   0.00 kB                  │",
            "│   container_2                      ✓ running   Up 2 hour   00.00%   0.00 kB / 0.00 kB          2   image_2                          0.00 kB   0.00 kB                  │",
            "│   container_3                      ✓ running   Up 3 hour   00.00%   0.00 kB / 0.00 kB          3   image_3                          0.00 kB   0.00 kB                  │",
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

        for (row_index, result_row) in get_result(&setup, w) {
            let expected_row = expected_to_vec(&expected, row_index);
            for (result_cell_index, result_cell) in result_row.iter().enumerate() {
                assert_eq!(result_cell.symbol(), expected_row[result_cell_index]);
            }
        }
    }

    // Check that the correct colour is applied to the state/status/cpu/memory section
    fn check_expected(expected: [&str; 6], w: u16, _h: u16, setup: &TuiTestSetup, color: Color) {
        for (row_index, result_row) in get_result(setup, w) {
            let expected_row = expected_to_vec(&expected, row_index);
            for (result_cell_index, result_cell) in result_row.iter().enumerate() {
                assert_eq!(result_cell.symbol(), expected_row[result_cell_index]);

                match (row_index, result_cell_index) {
                    // border
                    (0 | 5, _) | (1..=4, 0 | 129) => {
                        assert_eq!(result_cell.fg, Color::LightCyan);
                    }
                    // name, id, image column
                    (1..=3, 4..=17 | 71..=91) => {
                        assert_eq!(result_cell.fg, Color::Blue);
                    }
                    // state, status, cpu, memory column of the first row
                    (1, 18..=70) => {
                        assert_eq!(result_cell.fg, color);
                    }
                    // state, status, cpu, memory column
                    (2..=3, 4..=77) => {
                        assert_eq!(result_cell.fg, Color::Green);
                    }
                    // rx column
                    (1..=3, 92..=101) => {
                        assert_eq!(result_cell.fg, Color::Rgb(255, 233, 193));
                    }
                    // tx column
                    (1..=3, 102..=111) => {
                        assert_eq!(result_cell.fg, Color::Rgb(205, 140, 140));
                    }
                    _ => assert_eq!(result_cell.fg, Color::Reset),
                }
            }
        }
    }

    #[test]
    /// When container is paused, correct colors displayed
    fn test_draw_blocks_containers_paused() {
        let (w, h) = (130, 6);
        let mut setup = test_setup(w, h, true, true);

        let expected = [
            "╭ Containers 1/3 ────────────────────────────────────────────────────────────────────────────────────────────────────────────────╮",
            "│⚪  container_1   ॥ paused    Up 1 hour   00.00%   0.00 kB / 0.00 kB          1   image_1   0.00 kB   0.00 kB                    │",
            "│   container_2   ✓ running   Up 2 hour   00.00%   0.00 kB / 0.00 kB          2   image_2   0.00 kB   0.00 kB                    │",
            "│   container_3   ✓ running   Up 3 hour   00.00%   0.00 kB / 0.00 kB          3   image_3   0.00 kB   0.00 kB                    │",
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
            "│⚪  container_1   ✖ dead      Up 1 hour   00.00%   0.00 kB / 0.00 kB          1   image_1   0.00 kB   0.00 kB                    │",
            "│   container_2   ✓ running   Up 2 hour   00.00%   0.00 kB / 0.00 kB          2   image_2   0.00 kB   0.00 kB                    │",
            "│   container_3   ✓ running   Up 3 hour   00.00%   0.00 kB / 0.00 kB          3   image_3   0.00 kB   0.00 kB                    │",
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
            "│⚪  container_1   ✖ exited    Up 1 hour   00.00%   0.00 kB / 0.00 kB          1   image_1   0.00 kB   0.00 kB                    │",
            "│   container_2   ✓ running   Up 2 hour   00.00%   0.00 kB / 0.00 kB          2   image_2   0.00 kB   0.00 kB                    │",
            "│   container_3   ✓ running   Up 3 hour   00.00%   0.00 kB / 0.00 kB          3   image_3   0.00 kB   0.00 kB                    │",
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
            "│⚪  container_1   removing    Up 1 hour   00.00%   0.00 kB / 0.00 kB          1   image_1   0.00 kB   0.00 kB                    │",
            "│   container_2   ✓ running   Up 2 hour   00.00%   0.00 kB / 0.00 kB          2   image_2   0.00 kB   0.00 kB                    │",
            "│   container_3   ✓ running   Up 3 hour   00.00%   0.00 kB / 0.00 kB          3   image_3   0.00 kB   0.00 kB                    │",
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
            "│⚪  container_1   ↻ restarting   Up 1 hour   00.00%   0.00 kB / 0.00 kB          1   image_1   0.00 kB   0.00 kB                 │",
            "│   container_2   ✓ running      Up 2 hour   00.00%   0.00 kB / 0.00 kB          2   image_2   0.00 kB   0.00 kB                 │",
            "│   container_3   ✓ running      Up 3 hour   00.00%   0.00 kB / 0.00 kB          3   image_3   0.00 kB   0.00 kB                 │",
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

        for (row_index, result_row) in get_result(&setup, w) {
            let expected_row = expected_to_vec(&expected, row_index);
            for (result_cell_index, result_cell) in result_row.iter().enumerate() {
                assert_eq!(result_cell.symbol(), expected_row[result_cell_index]);

                match (row_index, result_cell_index) {
                    // border
                    (0 | 5, _) | (1..=4, 0 | 129) => {
                        assert_eq!(result_cell.fg, Color::LightCyan);
                    }
                    // name, id, image column
                    (1..=3, 4..=17 | 74..=94) => {
                        assert_eq!(result_cell.fg, Color::Blue);
                    }
                    // state, status, cpu, memory column of the first row
                    (1, 18..=73) => {
                        assert_eq!(result_cell.fg, Color::LightGreen);
                    }
                    // state, status, cpu, memory column
                    (2..=3, 18..=73) => {
                        assert_eq!(result_cell.fg, Color::Green);
                    }
                    // rx column
                    (1..=3, 95..=104) => {
                        assert_eq!(result_cell.fg, Color::Rgb(255, 233, 193));
                    }
                    // tx column
                    (1..=3, 105..=114) => {
                        assert_eq!(result_cell.fg, Color::Rgb(205, 140, 140));
                    }
                    _ => {
                        assert_eq!(result_cell.fg, Color::Reset);
                    }
                }
            }
        }
    }

    #[test]
    /// When container state is unknown, correct colors displayed
    fn test_draw_blocks_containers_unknown() {
        let (w, h) = (130, 6);
        let mut setup = test_setup(w, h, true, true);

        let expected = [
            "╭ Containers 1/3 ────────────────────────────────────────────────────────────────────────────────────────────────────────────────╮",
            "│⚪  container_1   ? unknown   Up 1 hour   00.00%   0.00 kB / 0.00 kB          1   image_1   0.00 kB   0.00 kB                    │",
            "│   container_2   ✓ running   Up 2 hour   00.00%   0.00 kB / 0.00 kB          2   image_2   0.00 kB   0.00 kB                    │",
            "│   container_3   ✓ running   Up 3 hour   00.00%   0.00 kB / 0.00 kB          3   image_3   0.00 kB   0.00 kB                    │",
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

        for (row_index, result_row) in get_result(&setup, w) {
            let expected_row = expected_to_vec(&expected, row_index);
            for (result_cell_index, result_cell) in result_row.iter().enumerate() {
                assert_eq!(result_cell.symbol(), expected_row[result_cell_index]);
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

        for (row_index, result_row) in get_result(&setup, w) {
            let expected_row = expected_to_vec(&expected, row_index);
            for (result_cell_index, result_cell) in result_row.iter().enumerate() {
                assert_eq!(result_cell.symbol(), expected_row[result_cell_index]);

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

        for (row_index, result_row) in get_result(&setup, w) {
            let expected_row = expected_to_vec(&expected, row_index);
            for (result_cell_index, result_cell) in result_row.iter().enumerate() {
                assert_eq!(result_cell.symbol(), expected_row[result_cell_index]);
                assert_eq!(result_cell.fg, Color::Reset);
            }
        }

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

        for (row_index, result_row) in get_result(&setup, w) {
            let expected_row = expected_to_vec(&expected, row_index);
            for (result_cell_index, result_cell) in result_row.iter().enumerate() {
                assert_eq!(result_cell.symbol(), expected_row[result_cell_index]);
                assert_eq!(result_cell.fg, Color::Reset);
            }
        }
    }

    #[test]
    /// Logs correct displayed, changing log state also draws correctly
    fn test_draw_blocks_logs_some() {
        let (w, h) = (25, 6);
        let mut setup = test_setup(w, h, true, true);

        insert_logs(&setup);

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

        for (row_index, result_row) in get_result(&setup, w) {
            let expected_row = expected_to_vec(&expected, row_index);
            for (result_cell_index, result_cell) in result_row.iter().enumerate() {
                assert_eq!(result_cell.symbol(), expected_row[result_cell_index]);
                assert_eq!(result_cell.fg, Color::Reset);

                if row_index == 3 && (1..=23).contains(&result_cell_index) {
                    assert_eq!(result_cell.modifier, Modifier::BOLD);
                } else {
                    assert!(result_cell.modifier.is_empty());
                }
            }
        }

        // Change selected log line
        setup.app_data.lock().log_previous();
        _ = FrameData::from((setup.app_data.lock(), setup.gui_state.lock()));

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
        for (row_index, result_row) in get_result(&setup, w) {
            let expected_row = expected_to_vec(&expected, row_index);
            for (result_cell_index, result_cell) in result_row.iter().enumerate() {
                assert_eq!(result_cell.symbol(), expected_row[result_cell_index]);
                assert_eq!(result_cell.fg, Color::Reset);

                if row_index == 2 && (1..=23).contains(&result_cell_index) {
                    assert_eq!(result_cell.modifier, Modifier::BOLD);
                } else {
                    assert!(result_cell.modifier.is_empty());
                }
            }
        }
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

        for (row_index, result_row) in get_result(&setup, w) {
            let expected_row = expected_to_vec(&expected, row_index);
            for (result_cell_index, result_cell) in result_row.iter().enumerate() {
                assert_eq!(result_cell.symbol(), expected_row[result_cell_index]);
            }
        }
    }

    // ************ //
    // Charts panel //
    // ************ //

    #[allow(clippy::cast_precision_loss)]
    // Add fixed data to the cpu & mem vecdeques
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

    /// CPU and Memory charts used in multiple tests, based on data from above insert_chart_data()
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

    // co-ordinates of the dots from the cpu chart
    const CPU_XY: [(usize, usize); 15] = [
        (1, 12),
        (2, 11),
        (2, 12),
        (3, 10),
        (3, 11),
        (3, 12),
        (4, 10),
        (4, 12),
        (5, 9),
        (5, 13),
        (5, 14),
        (6, 8),
        (6, 13),
        (7, 8),
        (7, 13),
    ];

    // co-ordinates of the dots from the memory chart
    const MEM_XY: [(usize, usize); 16] = [
        (1, 54),
        (1, 55),
        (2, 54),
        (2, 55),
        (3, 53),
        (3, 55),
        (4, 52),
        (4, 55),
        (5, 51),
        (5, 52),
        (5, 55),
        (5, 56),
        (6, 51),
        (6, 55),
        (7, 51),
        (7, 55),
    ];

    #[test]
    /// When status is Running, but not data, charts drawn without dots etc, colours correct
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

        for (row_index, result_row) in get_result(&setup, w) {
            let expected_row = expected_to_vec(&expected, row_index);
            for (result_cell_index, result_cell) in result_row.iter().enumerate() {
                assert_eq!(result_cell.symbol(), expected_row[result_cell_index]);

                match (row_index, result_cell_index) {
                    (0, 14..=25 | 52..=67) => {
                        assert_eq!(result_cell.fg, Color::Green);
                        assert_eq!(result_cell.modifier, Modifier::BOLD);
                    }
                    (1, 1..=6 | 41..=47) => {
                        assert_eq!(result_cell.fg, ORANGE);
                        assert_eq!(result_cell.modifier, Modifier::BOLD);
                    }
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

        for (row_index, result_row) in get_result(&setup, w) {
            let expected_row = expected_to_vec(&EXPECTED, row_index);
            for (result_cell_index, result_cell) in result_row.iter().enumerate() {
                assert_eq!(result_cell.symbol(), expected_row[result_cell_index]);

                match (row_index, result_cell_index) {
                    (0, 14..=25 | 51..=67) => {
                        assert_eq!(result_cell.fg, Color::Green);
                        assert_eq!(result_cell.modifier, Modifier::BOLD);
                    }
                    (1, 1..=6 | 41..=49) => {
                        assert_eq!(result_cell.fg, ORANGE);
                        assert_eq!(result_cell.modifier, Modifier::BOLD);
                    }
                    xy if CPU_XY.contains(&xy) => {
                        assert_eq!(result_cell.fg, Color::Magenta);
                        assert!(result_cell.modifier.is_empty());
                    }
                    xy if MEM_XY.contains(&xy) => {
                        assert_eq!(result_cell.fg, Color::Cyan);
                        assert!(result_cell.modifier.is_empty());
                    }
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

        for (row_index, result_row) in get_result(&setup, w) {
            let expected_row = expected_to_vec(&EXPECTED, row_index);
            for (result_cell_index, result_cell) in result_row.iter().enumerate() {
                assert_eq!(result_cell.symbol(), expected_row[result_cell_index]);

                match (row_index, result_cell_index) {
                    (0, 14..=25 | 51..=67) | (1, 1..=6 | 41..=49) => {
                        assert_eq!(result_cell.fg, Color::Yellow);
                        assert_eq!(result_cell.modifier, Modifier::BOLD);
                    }
                    xy if CPU_XY.contains(&xy) => {
                        assert_eq!(result_cell.fg, Color::Magenta);
                        assert!(result_cell.modifier.is_empty());
                    }
                    xy if MEM_XY.contains(&xy) => {
                        assert_eq!(result_cell.fg, Color::Cyan);
                        assert!(result_cell.modifier.is_empty());
                    }
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

        for (row_index, result_row) in get_result(&setup, w) {
            let expected_row = expected_to_vec(&EXPECTED, row_index);
            for (result_cell_index, result_cell) in result_row.iter().enumerate() {
                assert_eq!(result_cell.symbol(), expected_row[result_cell_index]);

                match (row_index, result_cell_index) {
                    (0, 14..=25 | 51..=67) | (1, 1..=6 | 41..=49) => {
                        assert_eq!(result_cell.fg, Color::Red);
                        assert_eq!(result_cell.modifier, Modifier::BOLD);
                    }
                    xy if CPU_XY.contains(&xy) => {
                        assert_eq!(result_cell.fg, Color::Magenta);
                        assert!(result_cell.modifier.is_empty());
                    }
                    xy if MEM_XY.contains(&xy) => {
                        assert_eq!(result_cell.fg, Color::Cyan);
                        assert!(result_cell.modifier.is_empty());
                    }
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

        let expected =  ["                                                                                                                          ( h ) show help   "];

        setup
            .terminal
            .draw(|f| {
                super::heading_bar(setup.area, f, &fd, &setup.gui_state);
            })
            .unwrap();

        for (row_index, result_row) in get_result(&setup, w) {
            let expected_row = expected_to_vec(&expected, row_index);
            for (result_cell_index, result_cell) in result_row.iter().enumerate() {
                assert_eq!(result_cell.symbol(), expected_row[result_cell_index]);
                assert_eq!(result_cell.bg, Color::Magenta);
                assert_eq!(result_cell.fg, Color::White);
            }
        }

        fd.help_visible = true;
        let expected =  ["                                                                                                                          ( h ) exit help   "];
        setup
            .terminal
            .draw(|f| {
                super::heading_bar(setup.area, f, &fd, &setup.gui_state);
            })
            .unwrap();

        for (row_index, result_row) in get_result(&setup, w) {
            let expected_row = expected_to_vec(&expected, row_index);
            for (result_cell_index, result_cell) in result_row.iter().enumerate() {
                assert_eq!(result_cell.symbol(), expected_row[result_cell_index]);
                assert_eq!(result_cell.bg, Color::Magenta);
                assert_eq!(result_cell.fg, Color::Black);
            }
        }
    }

    #[test]
    /// Show all headings when containers present, colors valid
    fn test_draw_blocks_headers_some_containers() {
        let (w, h) = (140, 1);
        let mut setup = test_setup(w, h, true, true);
        let fd = FrameData::from((setup.app_data.lock(), setup.gui_state.lock()));

        let expected =  ["    name          state       status      cpu      memory/limit        id         image     ↓ rx      ↑ tx                ( h ) show help   "];
        setup
            .terminal
            .draw(|f| {
                super::heading_bar(setup.area, f, &fd, &setup.gui_state);
            })
            .unwrap();

        for (row_index, result_row) in get_result(&setup, w) {
            let expected_row = expected_to_vec(&expected, row_index);
            for (result_cell_index, result_cell) in result_row.iter().enumerate() {
                assert_eq!(result_cell.symbol(), expected_row[result_cell_index]);
                assert_eq!(result_cell.bg, Color::Magenta);
                assert_eq!(
                    result_cell.fg,
                    match result_cell_index {
                        (4..=121) => Color::Black,
                        _ => Color::White,
                    }
                );
            }
        }
    }

    #[test]
    /// Only show the headings that fit the reduced-in-size header section
    fn test_draw_blocks_headers_some_containers_reduced_width() {
        let (w, h) = (80, 1);
        let mut setup = test_setup(w, h, true, true);
        let fd = FrameData::from((setup.app_data.lock(), setup.gui_state.lock()));

        let expected =
            ["    name          state       status      cpu                 ( h ) show help   "];
        setup
            .terminal
            .draw(|f| {
                super::heading_bar(setup.area, f, &fd, &setup.gui_state);
            })
            .unwrap();

        for (row_index, result_row) in get_result(&setup, w) {
            let expected_row = expected_to_vec(&expected, row_index);
            for (result_cell_index, result_cell) in result_row.iter().enumerate() {
                assert_eq!(result_cell.symbol(), expected_row[result_cell_index]);
                assert_eq!(result_cell.bg, Color::Magenta);
                assert_eq!(
                    result_cell.fg,
                    match result_cell_index {
                        (4..=61) => Color::Black,
                        _ => Color::White,
                    }
                );
            }
        }
    }

    #[test]
    /// Test all combination of headers & sort by
    fn test_draw_blocks_headers_sort_containers() {
        let (w, h) = (140, 1);
        let mut setup = test_setup(w, h, true, true);
        let mut fd = FrameData::from((setup.app_data.lock(), setup.gui_state.lock()));

        // Actual test, used for each header and sorted type
        let mut test =
            |expected: &[&str], range: RangeInclusive<usize>, x: (Header, SortedOrder)| {
                fd.sorted_by = Some(x);

                setup
                    .terminal
                    .draw(|f| {
                        super::heading_bar(setup.area, f, &fd, &setup.gui_state);
                    })
                    .unwrap();

                for (row_index, result_row) in get_result(&setup, w) {
                    let expected_row = expected_to_vec(expected, row_index);
                    for (result_cell_index, result_cell) in result_row.iter().enumerate() {
                        assert_eq!(result_cell.symbol(), expected_row[result_cell_index]);
                        assert_eq!(result_cell.bg, Color::Magenta);
                        assert_eq!(result_cell.bg, Color::Magenta);
                        assert_eq!(
                            result_cell.fg,
                            match result_cell_index {
                                0..=3 | 122..=139 => Color::White,
                                // given range | help section
                                x if range.contains(&x) => Color::Gray,
                                _ => Color::Black,
                            }
                        );
                    }
                }
            };

        // Name
        test(&["    name ▲        state       status      cpu      memory/limit        id         image     ↓ rx      ↑ tx                ( h ) show help   "], 1..=17, (Header::Name, SortedOrder::Asc));
        test(&["    name ▼        state       status      cpu      memory/limit        id         image     ↓ rx      ↑ tx                ( h ) show help   "], 1..=17, (Header::Name, SortedOrder::Desc));
        // state
        test(&["    name          state ▲     status      cpu      memory/limit        id         image     ↓ rx      ↑ tx                ( h ) show help   "],18..=29, (Header::State, SortedOrder::Asc));
        test(&["    name          state ▼     status      cpu      memory/limit        id         image     ↓ rx      ↑ tx                ( h ) show help   "], 18..=29, (Header::State, SortedOrder::Desc));
        // status
        test(&["    name          state       status ▲    cpu      memory/limit        id         image     ↓ rx      ↑ tx                ( h ) show help   "], 30..=41, (Header::Status, SortedOrder::Asc));
        test(&["    name          state       status ▼    cpu      memory/limit        id         image     ↓ rx      ↑ tx                ( h ) show help   "], 30..=41, (Header::Status, SortedOrder::Desc));
        // cpu
        test(&["    name          state       status      cpu ▲    memory/limit        id         image     ↓ rx      ↑ tx                ( h ) show help   "],42..=50, (Header::Cpu, SortedOrder::Asc));
        test(&["    name          state       status      cpu ▼    memory/limit        id         image     ↓ rx      ↑ tx                ( h ) show help   "],42..=50, (Header::Cpu, SortedOrder::Desc));
        // memory
        test(&["    name          state       status      cpu      memory/limit ▲      id         image     ↓ rx      ↑ tx                ( h ) show help   "], 51..=70, (Header::Memory, SortedOrder::Asc));
        test(&["    name          state       status      cpu      memory/limit ▼      id         image     ↓ rx      ↑ tx                ( h ) show help   "], 51..=70, (Header::Memory, SortedOrder::Desc));
        //id
        test(&["    name          state       status      cpu      memory/limit        id ▲       image     ↓ rx      ↑ tx                ( h ) show help   "], 71..=81, (Header::Id, SortedOrder::Asc));
        test(&["    name          state       status      cpu      memory/limit        id ▼       image     ↓ rx      ↑ tx                ( h ) show help   "], 71..=81, (Header::Id, SortedOrder::Desc));
        // image
        test(&["    name          state       status      cpu      memory/limit        id         image ▲   ↓ rx      ↑ tx                ( h ) show help   "], 82..=91, (Header::Image, SortedOrder::Asc));
        test(&["    name          state       status      cpu      memory/limit        id         image ▼   ↓ rx      ↑ tx                ( h ) show help   "], 82..=91, (Header::Image, SortedOrder::Desc));
        // rx
        test(&["    name          state       status      cpu      memory/limit        id         image     ↓ rx ▲    ↑ tx                ( h ) show help   "], 92..=101, (Header::Rx, SortedOrder::Asc));
        test(&["    name          state       status      cpu      memory/limit        id         image     ↓ rx ▼    ↑ tx                ( h ) show help   "], 92..=101, (Header::Rx, SortedOrder::Desc));
        // tx
        test(&["    name          state       status      cpu      memory/limit        id         image     ↓ rx      ↑ tx ▲              ( h ) show help   "], 102..=111, (Header::Tx, SortedOrder::Asc));
        test(&["    name          state       status      cpu      memory/limit        id         image     ↓ rx      ↑ tx ▼              ( h ) show help   "], 102..=111, (Header::Tx, SortedOrder::Desc));
    }

    #[test]
    /// Show animation
    fn test_draw_blocks_headers_animation() {
        let (w, h) = (140, 1);
        let mut setup = test_setup(w, h, true, true);
        let uuid = Uuid::new_v4();
        setup.gui_state.lock().next_loading(uuid);
        let fd = FrameData::from((setup.app_data.lock(), setup.gui_state.lock()));

        let expected =   [" ⠙  name          state       status      cpu      memory/limit        id         image     ↓ rx      ↑ tx                ( h ) show help   "];

        setup
            .terminal
            .draw(|f| {
                super::heading_bar(setup.area, f, &fd, &setup.gui_state);
            })
            .unwrap();

        for (row_index, result_row) in get_result(&setup, w) {
            let expected_row = expected_to_vec(&expected, row_index);
            for (result_cell_index, result_cell) in result_row.iter().enumerate() {
                assert_eq!(result_cell.symbol(), expected_row[result_cell_index]);
                assert_eq!(result_cell.bg, Color::Magenta);
                assert_eq!(
                    result_cell.fg,
                    match result_cell_index {
                        (4..=121) => Color::Black,
                        _ => Color::White,
                    }
                );
            }
        }
    }

    // ********** //
    // Help popup //
    // ********** //
    #[test]
    /// This will cause issues once the version has more than the current 5 chars (0.5.0)
    // Help  popup is drawn correctly
    fn test_draw_blocks_help() {
        let (w, h) = (87, 33);
        let mut setup = test_setup(w, h, true, true);

        setup
            .terminal
            .draw(|f| {
                super::help_box(f);
            })
            .unwrap();
        let version_row =   format!(" ╭ {VERSION} ────────────────────────────────────────────────────────────────────────────╮ ");
        let expected = [
            "                                                                                       ",
            version_row.as_str(),
            " │                                                                                   │ ",
            " │                                      88                                           │ ",
            " │                                      88                                           │ ",
            " │                                      88                                           │ ",
            " │             ,adPPYba,   8b,     ,d8  88   ,d8    ,adPPYba,  8b,dPPYba,            │ ",
            r#" │            a8"     "8a   `Y8, ,8P'   88 ,a8"    a8P_____88  88P'   "Y8            │ "#,
            r#" │            8b       d8     )888(     8888[      8PP"""""""  88                    │ "#,
            r#" │            "8a,   ,a8"   ,d8" "8b,   88`"Yba,   "8b,   ,aa  88                    │ "#,
            r#" │             `"YbbdP"'   8P'     `Y8  88   `Y8a   `"Ybbd8"'  88                    │ "#,
            " │                                                                                   │ ",
            " │                 A simple tui to view & control docker containers                  │ ",
            " │                                                                                   │ ",
            " │ ( tab ) or ( shift+tab ) change panels                                            │ ",
            " │ ( ↑ ↓ ) or ( j k ) or ( PgUp PgDown ) or ( Home End ) change selected line        │ ",
            " │ ( enter ) send docker container command                                           │ ",
            " │ ( e ) exec into a container                                                       │ ",
            " │ ( h ) toggle this help information                                                │ ",
            " │ ( s ) save logs to file                                                           │ ",
            " │ ( m ) toggle mouse capture - if disabled, text on screen can be selected & copied │ ",
            " │ ( F1 ) or ( / ) enter filter mode                                                 │ ",
            " │ ( 0 ) stop sort                                                                   │ ",
            " │ ( 1 - 9 ) sort by header - or click header                                        │ ",
            " │ ( esc ) close dialog                                                              │ ",
            " │ ( q ) quit at any time                                                            │ ",
            " │                                                                                   │ ",
            " │        currently an early work in progress, all and any input appreciated         │ ",
            " │                       https://github.com/mrjackwills/oxker                        │ ",
            " │                                                                                   │ ",
            " │                                                                                   │ ",
            " ╰───────────────────────────────────────────────────────────────────────────────────╯ ",
            "                                                                                       "
        ];

        for (row_index, result_row) in get_result(&setup, w) {
            let expected_row = expected_to_vec(&expected, row_index);
            for (result_cell_index, result_cell) in result_row.iter().enumerate() {
                assert_eq!(result_cell.symbol(), expected_row[result_cell_index]);

                match (row_index, result_cell_index) {
                    // first & last row, and first & last char on each row, is reset/reset, making sure that the help info is centered in the given area
                    (0 | 32, _) | (0..=33, 0 | 86) => {
                        assert_eq!(result_cell.bg, Color::Reset);
                        assert_eq!(result_cell.fg, Color::Reset);
                    }
                    // border is black on magenta
                    (1 | 31, _) | (1..=31, 1 | 85) => {
                        assert_eq!(result_cell.bg, Color::Magenta);
                        assert_eq!(result_cell.fg, Color::Black);
                    }
                      // oxker logo && description
                      (2..=10, 2..=85) | (12, 19..=66)
                    // button in the brackets
                    | (14, 2..=10 | 13..=27)
                    | (15, 2..=10 | 13..=21 | 24..=40 | 43..=56)
                    | (16 | 23, 2..=12)
                    | (17..=20 | 22 | 25, 2..=8)
                    | (21, 2..=9 | 12..=18)
                    | (24, 2..=10) => {
                        assert_eq!(result_cell.bg, Color::Magenta);
                        assert_eq!(result_cell.fg, Color::White);
                    }
                    // The URL is white and underlined
                    (28, 25..=60) => {
                        assert_eq!(result_cell.bg, Color::Magenta);
                        assert_eq!(result_cell.fg, Color::White);
                        assert_eq!(result_cell.modifier, Modifier::UNDERLINED);
                    }
                    // The rest is black on magenta
                    _ => {
                        assert_eq!(result_cell.bg, Color::Magenta);
                        assert_eq!(result_cell.fg, Color::Black);
                    }
                }
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

        for (row_index, result_row) in get_result(&setup, w) {
            let expected_row = expected_to_vec(&expected, row_index);
            for (result_cell_index, result_cell) in result_row.iter().enumerate() {
                assert_eq!(result_cell.symbol(), expected_row[result_cell_index]);

                match (row_index, result_cell_index) {
                    (0 | 9, _) | (1..=8, 0..=7 | 74..=81) => {
                        assert_eq!(result_cell.bg, Color::Reset);
                        assert_eq!(result_cell.fg, Color::Reset);
                    }
                    (3, 57..=67) => {
                        assert_eq!(result_cell.bg, Color::White);
                        assert_eq!(result_cell.fg, Color::Red);
                        assert_eq!(result_cell.modifier, Modifier::BOLD);
                    }
                    _ => {
                        assert_eq!(result_cell.bg, Color::White);
                        assert_eq!(result_cell.fg, Color::Black);
                    }
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

        for (row_index, result_row) in get_result(&setup, w) {
            let expected_row = expected_to_vec(&expected, row_index);
            for (result_cell_index, result_cell) in result_row.iter().enumerate() {
                assert_eq!(result_cell.symbol(), expected_row[result_cell_index]);

                match (row_index, result_cell_index) {
                    (0 | 9, _) | (1..=8, 0..=7 | 98..=106) => {
                        assert_eq!(result_cell.bg, Color::Reset);
                        assert_eq!(result_cell.fg, Color::Reset);
                    }
                    (3, 57..=91) => {
                        assert_eq!(result_cell.bg, Color::White);
                        assert_eq!(result_cell.fg, Color::Red);
                        assert_eq!(result_cell.modifier, Modifier::BOLD);
                    }
                    _ => {
                        assert_eq!(result_cell.bg, Color::White);
                        assert_eq!(result_cell.fg, Color::Black);
                    }
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

        for (row_index, result_row) in get_result(&setup, w) {
            let expected_row = expected_to_vec(&expected, row_index);
            for (result_cell_index, result_cell) in result_row.iter().enumerate() {
                assert_eq!(result_cell.symbol(), expected_row[result_cell_index]);

                let (bg, fg) = match (row_index, result_cell_index) {
                    (6..=8, 32..=44) => (Color::Blue, Color::White),
                    _ => (Color::Reset, Color::Reset),
                };
                assert_eq!(result_cell.bg, bg);
                assert_eq!(result_cell.fg, fg);
            }
        }
    }

    // ********** //
    // Filter Row //
    // ********** //

    #[test]
    #[allow(clippy::cognitive_complexity, clippy::too_many_lines)]
    /// Filter row is drawn correctly & colors are correct
    /// Colours change when filter_by option is changed
    fn test_draw_blocks_filter_row() {
        let (w, h) = (140, 1);
        let mut setup = test_setup(w, h, true, true);

        setup
            .gui_state
            .lock()
            .status_push(crate::ui::Status::Filter);
        setup
            .terminal
            .draw(|f| {
                super::filter_bar(setup.area, f, &setup.app_data);
            })
            .unwrap();

        let expected = [
            " Esc  clear  ← by →   Name  Image  Status  All  term:                                                                                        "
        ];

        for (row_index, result_row) in get_result(&setup, w) {
            let expected_row = expected_to_vec(&expected, row_index);
            for (result_cell_index, result_cell) in result_row.iter().enumerate() {
                assert_eq!(result_cell.symbol(), expected_row[result_cell_index]);
                match result_cell_index {
                    0..=4 | 12..=19 => {
                        assert_eq!(result_cell.bg, Color::Magenta);
                        assert_eq!(result_cell.fg, Color::Black);
                    }
                    5..=11 => {
                        assert_eq!(result_cell.bg, Color::Reset);
                        assert_eq!(result_cell.fg, Color::Gray);
                    }
                    21..=26 => {
                        assert_eq!(result_cell.bg, Color::Gray);
                        assert_eq!(result_cell.fg, Color::Black);
                    }
                    47..=53 => {
                        assert_eq!(result_cell.bg, Color::Reset);
                        assert_eq!(result_cell.fg, Color::Magenta);
                        assert_eq!(result_cell.modifier, Modifier::BOLD);
                    }
                    _ => {
                        assert_eq!(result_cell.bg, Color::Reset);
                        assert_eq!(result_cell.fg, Color::Reset);
                    }
                }
            }
        }

        // Test when char added to search term
        setup.app_data.lock().filter_term_push('c');
        setup.app_data.lock().filter_term_push('d');

        setup
            .terminal
            .draw(|f| {
                super::filter_bar(setup.area, f, &setup.app_data);
            })
            .unwrap();

        let expected = [
            " Esc  clear  ← by →   Name  Image  Status  All  term: cd                                                                                     "
        ];

        for (row_index, result_row) in get_result(&setup, w) {
            let expected_row = expected_to_vec(&expected, row_index);
            for (result_cell_index, result_cell) in result_row.iter().enumerate() {
                assert_eq!(result_cell.symbol(), expected_row[result_cell_index]);

                match result_cell_index {
                    0..=4 | 12..=19 => {
                        assert_eq!(result_cell.bg, Color::Magenta);
                        assert_eq!(result_cell.fg, Color::Black);
                    }
                    5..=11 | 54..=55 => {
                        assert_eq!(result_cell.bg, Color::Reset);
                        assert_eq!(result_cell.fg, Color::Gray);
                    }
                    21..=26 => {
                        assert_eq!(result_cell.bg, Color::Gray);
                        assert_eq!(result_cell.fg, Color::Black);
                    }
                    47..=53 => {
                        assert_eq!(result_cell.bg, Color::Reset);
                        assert_eq!(result_cell.fg, Color::Magenta);
                        assert_eq!(result_cell.modifier, Modifier::BOLD);
                    }
                    _ => {
                        assert_eq!(result_cell.bg, Color::Reset);
                        assert_eq!(result_cell.fg, Color::Reset);
                    }
                }
            }
        }

        // Test when filter_by chances
        setup.app_data.lock().filter_by_next();
        setup
            .terminal
            .draw(|f| {
                super::filter_bar(setup.area, f, &setup.app_data);
            })
            .unwrap();

        let expected = [
        " Esc  clear  ← by →   Name  Image  Status  All  term: cd                                                                                     "
    ];

        for (row_index, result_row) in get_result(&setup, w) {
            let expected_row = expected_to_vec(&expected, row_index);
            for (result_cell_index, result_cell) in result_row.iter().enumerate() {
                assert_eq!(result_cell.symbol(), expected_row[result_cell_index]);

                match result_cell_index {
                    0..=4 | 12..=19 => {
                        assert_eq!(result_cell.bg, Color::Magenta);
                        assert_eq!(result_cell.fg, Color::Black);
                    }
                    5..=11 | 54..=55 => {
                        assert_eq!(result_cell.bg, Color::Reset);
                        assert_eq!(result_cell.fg, Color::Gray);
                    }
                    27..=33 => {
                        assert_eq!(result_cell.bg, Color::Gray);
                        assert_eq!(result_cell.fg, Color::Black);
                    }
                    47..=53 => {
                        assert_eq!(result_cell.bg, Color::Reset);
                        assert_eq!(result_cell.fg, Color::Magenta);
                        assert_eq!(result_cell.modifier, Modifier::BOLD);
                    }
                    _ => {
                        assert_eq!(result_cell.bg, Color::Reset);
                        assert_eq!(result_cell.fg, Color::Reset);
                    }
                }
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

        let version_row = format!(" │    oxker::v{VERSION} closing in 04 seconds   │ ");
        let expected = [
            "                                              ",
            " ╭───────────────── Error ──────────────────╮ ",
            " │                                          │ ",
            " │      Unable to access docker daemon      │ ",
            " │                                          │ ",
            version_row.as_str(),
            " │                                          │ ",
            " ╰──────────────────────────────────────────╯ ",
            "                                              ",
        ];

        for (row_index, result_row) in get_result(&setup, w) {
            let expected_row = expected_to_vec(&expected, row_index);
            for (result_cell_index, result_cell) in result_row.iter().enumerate() {
                assert_eq!(result_cell.symbol(), expected_row[result_cell_index]);

                match (row_index, result_cell_index) {
                    (0 | 8, _) | (1..=7, 0 | 45) => {
                        assert_eq!(result_cell.bg, Color::Reset);
                        assert_eq!(result_cell.fg, Color::Reset);
                    }
                    _ => {
                        assert_eq!(result_cell.bg, Color::Red);
                        assert_eq!(result_cell.fg, Color::White);
                    }
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

        for (row_index, result_row) in get_result(&setup, w) {
            let expected_row = expected_to_vec(&expected, row_index);
            for (result_cell_index, result_cell) in result_row.iter().enumerate() {
                assert_eq!(result_cell.symbol(), expected_row[result_cell_index]);

                match (row_index, result_cell_index) {
                    (0 | 9, _) | (1..=8, 0 | 38) => {
                        assert_eq!(result_cell.bg, Color::Reset);
                        assert_eq!(result_cell.fg, Color::Reset);
                    }
                    _ => {
                        assert_eq!(result_cell.bg, Color::Red);
                        assert_eq!(result_cell.fg, Color::White);
                    }
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

        for (row_index, result_row) in get_result(&setup, w) {
            let expected_row = expected_to_vec(&expected, row_index);
            for (result_cell_index, result_cell) in result_row.iter().enumerate() {
                assert_eq!(result_cell.symbol(), expected_row[result_cell_index]);
                match (row_index, result_cell_index) {
                    (0, 11..=17) => {
                        assert_eq!(result_cell.bg, Color::Reset);
                        assert_eq!(result_cell.fg, Color::Green);
                        assert_eq!(result_cell.modifier, Modifier::BOLD);
                    }
                    (1, 11..=18) => {
                        assert_eq!(result_cell.bg, Color::Reset);
                        assert_eq!(result_cell.fg, Color::Reset);
                        assert_eq!(result_cell.modifier, Modifier::BOLD);
                    }
                    _ => {
                        assert_eq!(result_cell.bg, Color::Reset);
                        assert_eq!(result_cell.fg, Color::Reset);
                        assert!(result_cell.modifier.is_empty());
                    }
                }
            }
        }

        // When state is "State::Running | State::Paused | State::Restarting, won't show "no ports"
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

        for (row_index, result_row) in get_result(&setup, w) {
            let expected_row = expected_to_vec(&expected, row_index);
            for (result_cell_index, result_cell) in result_row.iter().enumerate() {
                assert_eq!(result_cell.symbol(), expected_row[result_cell_index]);
                assert_eq!(result_cell.bg, Color::Reset);
                if let (0, 11..=17) = (row_index, result_cell_index) {
                    assert_eq!(result_cell.fg, Color::Red);
                    assert_eq!(result_cell.modifier, Modifier::BOLD);
                } else {
                    assert_eq!(result_cell.fg, Color::Reset);
                    assert!(result_cell.modifier.is_empty());
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

        for (row_index, result_row) in get_result(&setup, w) {
            let expected_row = expected_to_vec(&expected, row_index);
            for (result_cell_index, result_cell) in result_row.iter().enumerate() {
                assert_eq!(result_cell.symbol(), expected_row[result_cell_index]);
                assert_eq!(result_cell.bg, Color::Reset);

                match (row_index, result_cell_index) {
                    (0, 12..=18) => {
                        assert_eq!(result_cell.fg, Color::Green);
                        assert_eq!(result_cell.modifier, Modifier::BOLD);
                    }
                    (1, 1..=28) => {
                        assert_eq!(result_cell.fg, Color::Yellow);
                        assert!(result_cell.modifier.is_empty());
                    }
                    (2..=4, 1..=28) => {
                        assert_eq!(result_cell.fg, Color::White);
                        assert!(result_cell.modifier.is_empty());
                    }

                    _ => {
                        assert_eq!(result_cell.fg, Color::Reset);
                        assert!(result_cell.modifier.is_empty());
                    }
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

        for (row_index, result_row) in get_result(&setup, w) {
            let expected_row = expected_to_vec(&expected, row_index);
            for (result_cell_index, result_cell) in result_row.iter().enumerate() {
                assert_eq!(result_cell.symbol(), expected_row[result_cell_index]);
                assert_eq!(result_cell.bg, Color::Reset);
                if let (0, 12..=18) = (row_index, result_cell_index) {
                    assert_eq!(result_cell.fg, Color::Green);
                    assert_eq!(result_cell.modifier, Modifier::BOLD);
                }
            }
        }

        setup.app_data.lock().containers.items[0].state = State::Paused;
        setup
            .terminal
            .draw(|f| {
                super::ports(f, setup.area, &setup.app_data, max_lens);
            })
            .unwrap();

        for (row_index, result_row) in get_result(&setup, w) {
            let expected_row = expected_to_vec(&expected, row_index);
            for (result_cell_index, result_cell) in result_row.iter().enumerate() {
                assert_eq!(result_cell.symbol(), expected_row[result_cell_index]);
                assert_eq!(result_cell.bg, Color::Reset);
                if let (0, 12..=18) = (row_index, result_cell_index) {
                    assert_eq!(result_cell.fg, Color::Yellow);
                    assert_eq!(result_cell.modifier, Modifier::BOLD);
                }
            }
        }

        setup.app_data.lock().containers.items[0].state = State::Exited;
        setup
            .terminal
            .draw(|f| {
                super::ports(f, setup.area, &setup.app_data, max_lens);
            })
            .unwrap();

        for (row_index, result_row) in get_result(&setup, w) {
            let expected_row = expected_to_vec(&expected, row_index);
            for (result_cell_index, result_cell) in result_row.iter().enumerate() {
                assert_eq!(result_cell.symbol(), expected_row[result_cell_index]);
                assert_eq!(result_cell.bg, Color::Reset);
                if let (0, 12..=18) = (row_index, result_cell_index) {
                    assert_eq!(result_cell.fg, Color::Red);
                    assert_eq!(result_cell.modifier, Modifier::BOLD);
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
            "    name          state       status      cpu      memory/limit          id         image     ↓ rx      ↑ tx                                  ( h ) show help   ",
            "╭ Containers 1/3 ──────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────╮╭──────────────╮",
            "│⚪  container_1   ✓ running   Up 1 hour   03.00%   30.00 kB / 30.00 kB          1   image_1   0.00 kB   0.00 kB                                ││▶ pause       │",
            "│   container_2   ✓ running   Up 2 hour   00.00%    0.00 kB /  0.00 kB          2   image_2   0.00 kB   0.00 kB                                ││  restart     │",
            "│   container_3   ✓ running   Up 3 hour   00.00%    0.00 kB /  0.00 kB          3   image_3   0.00 kB   0.00 kB                                ││  stop        │",
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

        for (row_index, result_row) in get_result(&setup, w) {
            let expected_row = expected_to_vec(&expected, row_index);
            for (result_cell_index, result_cell) in result_row.iter().enumerate() {
                assert_eq!(result_cell.symbol(), expected_row[result_cell_index]);
            }
        }
    }

    #[test]
    #[allow(clippy::too_many_lines)]
    /// Check that the whole layout is drawn correctly
    fn test_draw_blocks_whole_layout_with_filter() {
        let (w, h) = (160, 30);
        let mut setup = test_setup(w, h, true, true);
        insert_chart_data(&setup);
        insert_logs(&setup);

        setup.app_data.lock().containers.items[1]
            .ports
            .push(ContainerPorts {
                ip: Some("127.0.0.1".to_owned()),
                private: 8003,
                public: Some(8003),
            });

        let expected = [
            "    name          state       status      cpu      memory/limit          id         image     ↓ rx      ↑ tx                                  ( h ) show help   ",
            "╭ Containers 1/3 ──────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────╮╭──────────────╮",
            "│⚪  container_1   ✓ running   Up 1 hour   03.00%   30.00 kB / 30.00 kB          1   image_1   0.00 kB   0.00 kB                                ││▶ pause       │",
            "│   container_2   ✓ running   Up 2 hour   00.00%    0.00 kB /  0.00 kB          2   image_2   0.00 kB   0.00 kB                                ││  restart     │",
            "│   container_3   ✓ running   Up 3 hour   00.00%    0.00 kB /  0.00 kB          3   image_3   0.00 kB   0.00 kB                                ││  stop        │",
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
            "│      │••       •••                                            ││         │••      •••                                          ││                            │",
            "│      │                                                        ││         │                                                     ││                            │",
            "╰───────────────────────────────────────────────────────────────╯╰───────────────────────────────────────────────────────────────╯╰────────────────────────────╯",
                ];
        setup
            .terminal
            .draw(|f| {
                draw_frame(f, &setup.app_data, &setup.gui_state);
            })
            .unwrap();

        for (row_index, result_row) in get_result(&setup, w) {
            let expected_row = expected_to_vec(&expected, row_index);
            for (result_cell_index, result_cell) in result_row.iter().enumerate() {
                assert_eq!(result_cell.symbol(), expected_row[result_cell_index]);
            }
        }

        setup
            .gui_state
            .lock()
            .status_push(crate::ui::Status::Filter);
        setup.app_data.lock().filter_term_push('r');
        setup.app_data.lock().filter_term_push('_');
        setup.app_data.lock().filter_term_push('1');

        let expected = [
            "    name          state       status      cpu      memory/limit          id         image     ↓ rx      ↑ tx                                  ( h ) show help   ",
            "╭ Containers 1/1 - filtered ───────────────────────────────────────────────────────────────────────────────────────────────────────────────────╮╭──────────────╮",
            "│⚪  container_1   ✓ running   Up 1 hour   03.00%   30.00 kB / 30.00 kB          1   image_1   0.00 kB   0.00 kB                                ││▶ pause       │",
            "│                                                                                                                                              ││  restart     │",
            "│                                                                                                                                              ││  stop        │",
            "│                                                                                                                                              ││  delete      │",
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
            "│10.00%│      •••                                               ││100.00 kB│      ••                                             ││       ip   private   public│",
            "│      │    ••  •                                               ││         │    •• •                                             ││               8001         │",
            "│      │ •••     • •                                            ││         │ •••    • •                                          ││                            │",
            "│      │•        ••                                             ││         │•       ••                                           ││                            │",
            "│      │                                                        ││         │                                                     ││                            │",
            "╰───────────────────────────────────────────────────────────────╯╰───────────────────────────────────────────────────────────────╯╰────────────────────────────╯",
            " Esc  clear  ← by →   Name  Image  Status  All  term: r_1                                                                                                       ",
            ];
        setup
            .terminal
            .draw(|f| {
                draw_frame(f, &setup.app_data, &setup.gui_state);
            })
            .unwrap();

        for (row_index, result_row) in get_result(&setup, w) {
            let expected_row = expected_to_vec(&expected, row_index);
            for (result_cell_index, result_cell) in result_row.iter().enumerate() {
                assert_eq!(result_cell.symbol(), expected_row[result_cell_index]);
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
            "    name                             state       status      cpu      memory/limit          id         image                            ↓ rx      ↑ tx                      ( h ) show help   ",
            "╭ Containers 1/3 ─────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────╮╭─────────────────╮",
            "│⚪  a_long_container_name_for_the…   ✓ running   Up 1 hour   03.00%   30.00 kB / 30.00 kB          1   a_long_image_name_for_the_pur…   0.00 kB   0.00 kB                 ││▶ pause          │",
            "│   container_2                      ✓ running   Up 2 hour   00.00%    0.00 kB /  0.00 kB          2   image_2                          0.00 kB   0.00 kB                 ││  restart        │",
            "│   container_3                      ✓ running   Up 3 hour   00.00%    0.00 kB /  0.00 kB          3   image_3                          0.00 kB   0.00 kB                 ││  stop           │",
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

        for (row_index, result_row) in get_result(&setup, w) {
            let expected_row = expected_to_vec(&expected, row_index);
            for (result_cell_index, result_cell) in result_row.iter().enumerate() {
                assert_eq!(result_cell.symbol(), expected_row[result_cell_index]);
            }
        }
    }
}
