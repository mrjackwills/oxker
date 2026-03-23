use std::sync::LazyLock;

use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect, Size},
    style::{Color, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, Padding, Paragraph},
};

use crate::{
    config::{AppColors, Config, Keymap},
    ui::gui_state::BoxLocation,
};

use super::{DESCRIPTION, NAME_TEXT, REPO, VERSION, popup};

macro_rules! to_u16 {
    ($value:expr) => {
        u16::try_from($value).unwrap_or_default()
    };
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum KeyDescriptions {
    Clear,
    Command,
    Exec,
    FilterMode,
    Help,
    InspectMode,
    LogHeight,
    LogVisibility,
    MouseCapture,
    Panel,
    Quit,
    Redraw,
    Save,
    ScrollEnd,
    ScrollH,
    ScrollSpeed,
    ScrollStart,
    ScrollV,
    SearchMode,
    SortCpu,
    SortHeader,
    SortId,
    SortImage,
    SortMem,
    SortName,
    SortRX,
    SortState,
    SortStatus,
    SortStop,
    SortTX,
}

type Column = Vec<(Vec<Option<String>>, KeyDescriptions)>;

#[derive(Debug, Clone, Hash)]
struct KeymapColumns {
    left: Column,
    right: Column,
}

impl KeymapColumns {
    fn default(keymap: &Keymap) -> Self {
        Self {
            left: vec![
                (
                    vec![
                        Some(keymap.quit.0.to_string()),
                        keymap.quit.1.as_ref().map(|i| i.to_string()),
                    ],
                    KeyDescriptions::Quit,
                ),
                (
                    vec![
                        Some(keymap.scroll_down.0.to_string()),
                        Some(keymap.scroll_up.0.to_string()),
                        keymap.scroll_down.1.as_ref().map(|i| i.to_string()),
                        keymap.scroll_up.1.as_ref().map(|i| i.to_string()),
                        Some(keymap.scroll_start.0.to_string()),
                        Some(keymap.scroll_end.0.to_string()),
                        keymap.scroll_start.1.as_ref().map(|i| i.to_string()),
                        keymap.scroll_end.1.as_ref().map(|i| i.to_string()),
                    ],
                    KeyDescriptions::ScrollV,
                ),
                (
                    vec![Some(keymap.scroll_many.to_string())],
                    KeyDescriptions::ScrollSpeed,
                ),
                (
                    vec![
                        Some(keymap.exec.0.to_string()),
                        keymap.exec.1.as_ref().map(|i| i.to_string()),
                    ],
                    KeyDescriptions::Exec,
                ),
                (
                    vec![
                        Some(keymap.filter_mode.0.to_string()),
                        keymap.filter_mode.1.as_ref().map(|i| i.to_string()),
                    ],
                    KeyDescriptions::FilterMode,
                ),
                (
                    vec![
                        Some(keymap.toggle_help.0.to_string()),
                        keymap.toggle_help.1.as_ref().map(|i| i.to_string()),
                    ],
                    KeyDescriptions::Help,
                ),
                (
                    vec![
                        Some(keymap.log_section_height_decrease.0.to_string()),
                        Some(keymap.log_section_height_increase.0.to_string()),
                        keymap
                            .log_section_height_decrease
                            .1
                            .as_ref()
                            .map(|i| i.to_string()),
                        keymap
                            .log_section_height_increase
                            .1
                            .as_ref()
                            .map(|i| i.to_string()),
                    ],
                    KeyDescriptions::LogHeight,
                ),
                (vec![Some("1 ~ 9".to_owned())], KeyDescriptions::SortHeader),
                (
                    vec![
                        Some(keymap.select_next_panel.0.to_string()),
                        Some(keymap.select_previous_panel.0.to_string()),
                        keymap
                            .select_previous_panel
                            .1
                            .as_ref()
                            .map(|i| i.to_string()),
                        keymap.select_next_panel.1.as_ref().map(|i| i.to_string()),
                    ],
                    KeyDescriptions::Panel,
                ),
                (
                    vec![
                        Some(keymap.save_logs.0.to_string()),
                        keymap.save_logs.1.as_ref().map(|i| i.to_string()),
                    ],
                    KeyDescriptions::Save,
                ),
            ],
            right: vec![
                (
                    vec![
                        Some(keymap.clear.0.to_string()),
                        keymap.clear.1.as_ref().map(|i| i.to_string()),
                    ],
                    KeyDescriptions::Clear,
                ),
                (
                    vec![
                        Some(keymap.scroll_back.0.to_string()),
                        Some(keymap.scroll_forward.0.to_string()),
                        keymap.scroll_back.1.as_ref().map(|i| i.to_string()),
                        keymap.scroll_forward.1.as_ref().map(|i| i.to_string()),
                    ],
                    KeyDescriptions::ScrollH,
                ),
                (vec![Some(String::from("Enter"))], KeyDescriptions::Command),
                (
                    vec![
                        Some(keymap.inspect.0.to_string()),
                        keymap.inspect.1.as_ref().map(|i| i.to_string()),
                    ],
                    KeyDescriptions::InspectMode,
                ),
                (
                    vec![
                        Some(keymap.log_search_mode.0.to_string()),
                        keymap.log_search_mode.1.as_ref().map(|i| i.to_string()),
                    ],
                    KeyDescriptions::SearchMode,
                ),
                (
                    vec![
                        Some(keymap.force_redraw.0.to_string()),
                        keymap.force_redraw.1.as_ref().map(|i| i.to_string()),
                    ],
                    KeyDescriptions::Redraw,
                ),
                (
                    vec![
                        Some(keymap.log_section_toggle.0.to_string()),
                        keymap.log_section_toggle.1.as_ref().map(|i| i.to_string()),
                    ],
                    KeyDescriptions::LogVisibility,
                ),
                (vec![Some("0".to_owned())], KeyDescriptions::SortStop),
                (
                    vec![
                        Some(keymap.toggle_mouse_capture.0.to_string()),
                        keymap
                            .toggle_mouse_capture
                            .1
                            .as_ref()
                            .map(|i| i.to_string()),
                    ],
                    KeyDescriptions::MouseCapture,
                ),
            ],
        }
    }

    fn custom(config: &Config) -> Self {
        Self {
            left: vec![
                (
                    vec![
                        Some(config.keymap.quit.0.to_string()),
                        config.keymap.quit.1.as_ref().map(|i| i.to_string()),
                    ],
                    KeyDescriptions::Quit,
                ),
                (
                    vec![
                        Some(config.keymap.scroll_down.0.to_string()),
                        Some(config.keymap.scroll_up.0.to_string()),
                        config.keymap.scroll_down.1.as_ref().map(|i| i.to_string()),
                        config.keymap.scroll_up.1.as_ref().map(|i| i.to_string()),
                    ],
                    KeyDescriptions::ScrollV,
                ),
                (
                    vec![
                        Some(config.keymap.scroll_start.0.to_string()),
                        config.keymap.scroll_start.1.as_ref().map(|i| i.to_string()),
                    ],
                    KeyDescriptions::ScrollStart,
                ),
                (
                    vec![Some(config.keymap.scroll_many.to_string())],
                    KeyDescriptions::ScrollSpeed,
                ),
                (
                    vec![
                        Some(config.keymap.exec.0.to_string()),
                        config.keymap.exec.1.as_ref().map(|i| i.to_string()),
                    ],
                    KeyDescriptions::Exec,
                ),
                (
                    vec![
                        Some(config.keymap.filter_mode.0.to_string()),
                        config.keymap.filter_mode.1.as_ref().map(|i| i.to_string()),
                    ],
                    KeyDescriptions::FilterMode,
                ),
                (
                    vec![
                        Some(config.keymap.toggle_help.0.to_string()),
                        config.keymap.toggle_help.1.as_ref().map(|i| i.to_string()),
                    ],
                    KeyDescriptions::Help,
                ),
                (
                    vec![
                        Some(config.keymap.log_section_height_decrease.0.to_string()),
                        Some(config.keymap.log_section_height_increase.0.to_string()),
                        config
                            .keymap
                            .log_section_height_decrease
                            .1
                            .as_ref()
                            .map(|i| i.to_string()),
                        config
                            .keymap
                            .log_section_height_increase
                            .1
                            .as_ref()
                            .map(|i| i.to_string()),
                    ],
                    KeyDescriptions::LogHeight,
                ),
                (
                    vec![
                        Some(config.keymap.sort_by_name.0.to_string()),
                        config.keymap.sort_by_name.1.as_ref().map(|i| i.to_string()),
                    ],
                    KeyDescriptions::SortName,
                ),
                (
                    vec![
                        Some(config.keymap.sort_by_status.0.to_string()),
                        config
                            .keymap
                            .sort_by_status
                            .1
                            .as_ref()
                            .map(|i| i.to_string()),
                    ],
                    KeyDescriptions::SortStatus,
                ),
                (
                    vec![
                        Some(config.keymap.sort_by_memory.0.to_string()),
                        config
                            .keymap
                            .sort_by_memory
                            .1
                            .as_ref()
                            .map(|i| i.to_string()),
                    ],
                    KeyDescriptions::SortMem,
                ),
                (
                    vec![
                        Some(config.keymap.sort_by_image.0.to_string()),
                        config
                            .keymap
                            .sort_by_image
                            .1
                            .as_ref()
                            .map(|i| i.to_string()),
                    ],
                    KeyDescriptions::SortImage,
                ),
                (
                    vec![
                        Some(config.keymap.sort_by_tx.0.to_string()),
                        config.keymap.sort_by_tx.1.as_ref().map(|i| i.to_string()),
                    ],
                    KeyDescriptions::SortTX,
                ),
                (
                    vec![
                        Some(config.keymap.select_next_panel.0.to_string()),
                        Some(config.keymap.select_previous_panel.0.to_string()),
                        config
                            .keymap
                            .select_previous_panel
                            .1
                            .as_ref()
                            .map(|i| i.to_string()),
                        config
                            .keymap
                            .select_next_panel
                            .1
                            .as_ref()
                            .map(|i| i.to_string()),
                    ],
                    KeyDescriptions::Panel,
                ),
                (
                    vec![
                        Some(config.keymap.save_logs.0.to_string()),
                        config.keymap.save_logs.1.as_ref().map(|i| i.to_string()),
                    ],
                    KeyDescriptions::Save,
                ),
            ],

            right: vec![
                (
                    vec![
                        Some(config.keymap.clear.0.to_string()),
                        config.keymap.clear.1.as_ref().map(|i| i.to_string()),
                    ],
                    KeyDescriptions::Clear,
                ),
                (
                    vec![
                        Some(config.keymap.scroll_back.0.to_string()),
                        Some(config.keymap.scroll_forward.0.to_string()),
                        config.keymap.scroll_back.1.as_ref().map(|i| i.to_string()),
                        config
                            .keymap
                            .scroll_forward
                            .1
                            .as_ref()
                            .map(|i| i.to_string()),
                    ],
                    KeyDescriptions::ScrollH,
                ),
                (
                    vec![
                        Some(config.keymap.scroll_end.0.to_string()),
                        config.keymap.scroll_end.1.as_ref().map(|i| i.to_string()),
                    ],
                    KeyDescriptions::ScrollEnd,
                ),
                (vec![Some(String::from("Enter"))], KeyDescriptions::Command),
                (
                    vec![
                        Some(config.keymap.inspect.0.to_string()),
                        config.keymap.inspect.1.as_ref().map(|i| i.to_string()),
                    ],
                    KeyDescriptions::InspectMode,
                ),
                (
                    vec![
                        Some(config.keymap.log_search_mode.0.to_string()),
                        config
                            .keymap
                            .log_search_mode
                            .1
                            .as_ref()
                            .map(|i| i.to_string()),
                    ],
                    KeyDescriptions::SearchMode,
                ),
                (
                    vec![
                        Some(config.keymap.force_redraw.0.to_string()),
                        config.keymap.force_redraw.1.as_ref().map(|i| i.to_string()),
                    ],
                    KeyDescriptions::Redraw,
                ),
                (
                    vec![
                        Some(config.keymap.log_section_toggle.0.to_string()),
                        config
                            .keymap
                            .log_section_toggle
                            .1
                            .as_ref()
                            .map(|i| i.to_string()),
                    ],
                    KeyDescriptions::LogVisibility,
                ),
                (
                    vec![
                        Some(config.keymap.sort_by_state.0.to_string()),
                        config
                            .keymap
                            .sort_by_state
                            .1
                            .as_ref()
                            .map(|i| i.to_string()),
                    ],
                    KeyDescriptions::SortState,
                ),
                (
                    vec![
                        Some(config.keymap.sort_by_cpu.0.to_string()),
                        config.keymap.sort_by_cpu.1.as_ref().map(|i| i.to_string()),
                    ],
                    KeyDescriptions::SortCpu,
                ),
                (
                    vec![
                        Some(config.keymap.sort_by_id.0.to_string()),
                        config.keymap.sort_by_id.1.as_ref().map(|i| i.to_string()),
                    ],
                    KeyDescriptions::SortId,
                ),
                (
                    vec![
                        Some(config.keymap.sort_by_rx.0.to_string()),
                        config.keymap.sort_by_rx.1.as_ref().map(|i| i.to_string()),
                    ],
                    KeyDescriptions::SortRX,
                ),
                (
                    vec![
                        Some(config.keymap.sort_reset.0.to_string()),
                        config.keymap.sort_reset.1.as_ref().map(|i| i.to_string()),
                    ],
                    KeyDescriptions::SortStop,
                ),
                (
                    vec![
                        Some(config.keymap.toggle_mouse_capture.0.to_string()),
                        config
                            .keymap
                            .toggle_mouse_capture
                            .1
                            .as_ref()
                            .map(|i| i.to_string()),
                    ],
                    KeyDescriptions::MouseCapture,
                ),
            ],
        }
    }

    /// Add 1 to allow spacing between the key and the definition
    fn longest_line(column: &Column) -> usize {
        column
            .iter()
            .map(|(keys, _)| {
                keys.iter()
                    .filter_map(|k| k.as_deref())
                    .collect::<Vec<_>>()
                    .join(" ")
                    .len()
            })
            .max()
            .unwrap_or(0)
            .saturating_add(1)
    }

    fn create_button_line(column: &Column, colors: &AppColors) -> Vec<Line<'static>> {
        let longest_button = Self::longest_line(column);
        column
            .iter()
            .map(|(keys, desc)| HelpInfo::create_button_line(keys, desc, *colors, longest_button))
            .collect::<Vec<_>>()
    }

    fn to_helpinfo(&self, config: &Config) -> (HelpInfo, HelpInfo) {
        let left = Self::create_button_line(&self.left, &config.app_colors);
        let right = Self::create_button_line(&self.right, &config.app_colors);

        let size_left = HelpInfo::calc_size(&left);
        let size_right = HelpInfo::calc_size(&right);
        (
            HelpInfo {
                lines: left,
                size: size_left,
            },
            HelpInfo {
                lines: right,
                size: size_right,
            },
        )
    }
}

impl KeyDescriptions {
    fn as_str(&self) -> &'static str {
        match self {
            Self::Clear => "close dialog",
            Self::Command => "send docker command",
            Self::Exec => "exec into a container",
            Self::FilterMode => "filter mode",
            Self::Help => "toggle this panel",
            Self::InspectMode => "container inspect mode",
            Self::LogHeight => "change log section height",
            Self::LogVisibility => "toggle of section visibility",
            Self::MouseCapture => "toggle mouse capture - allows text selection",
            Self::Panel => "change panel",
            Self::Quit => "quit",
            Self::Redraw => "force clear screen and redraw",
            Self::Save => "save logs to file",
            Self::ScrollH => "scroll horizontally",
            Self::ScrollStart => "scroll to start",
            Self::ScrollEnd => "scroll to end",
            Self::ScrollSpeed => "increase scroll speed",
            Self::ScrollV => "scroll vertically",
            Self::SearchMode => "log search mode",
            Self::SortHeader => "sort by header - or click header",
            Self::SortStop => "stop sort",
            Self::SortCpu => "sort by CPU",
            Self::SortId => "sort by ID",
            Self::SortImage => "sort by Image",
            Self::SortMem => "sort by memory",
            Self::SortName => "sort by name",
            Self::SortRX => "sort by RX",
            Self::SortState => "sort by state",
            Self::SortStatus => "sort by status",
            Self::SortTX => "sort by TX",
        }
    }
}

/// Help popup box needs these three pieces of information
/// Change this to a trait
#[derive(Debug, Clone, Hash)]
struct HelpInfo {
    lines: Vec<Line<'static>>,
    size: Size,
}

static DEFAULT_NAME: LazyLock<HelpInfo> = LazyLock::new(|| {
    let colors = AppColors::new();
    HelpInfo::gen_name_description(colors)
});

static DEFAULT_COLUMNS: LazyLock<KeymapColumns> =
    LazyLock::new(|| KeymapColumns::default(&Keymap::new()));

impl HelpInfo {
    /// Find the height and width of an array of lines
    fn calc_size(lines: &[Line]) -> Size {
        Size {
            width: to_u16!(
                lines
                    .iter()
                    .map(ratatui::prelude::Line::width)
                    .max()
                    .unwrap_or(1)
            ),
            height: to_u16!(lines.len()),
        }
    }
    /// Just an empty span, i.e. a new line
    fn empty_line<'a>() -> Line<'a> {
        Line::from(String::new())
    }

    /// generate a span, of given &str and given color
    fn span<'a>(input: String, color: Color) -> Span<'a> {
        Span::styled(input, Style::default().fg(color))
    }

    /// &str to black text span
    fn text_span<'a>(input: String, color: AppColors) -> Span<'a> {
        Self::span(input, color.popup_help.text)
    }

    /// &str to white text span
    fn highlighted_text_span<'a>(input: String, color: AppColors) -> Span<'a> {
        Self::span(input, color.popup_help.text_highlight)
    }

    /// Generate the `oxker` name section
    fn gen_name_description(colors: AppColors) -> Self {
        let mut lines = NAME_TEXT
            .lines()
            .map(|i| Line::from(Self::highlighted_text_span(i.to_owned(), colors)))
            .collect::<Vec<_>>();
        lines.extend([
            Self::empty_line(),
            Line::from(Self::highlighted_text_span(DESCRIPTION.to_owned(), colors)).centered(),
        ]);
        let size = Self::calc_size(&lines);
        Self { lines, size }
    }

    fn create_button<'a>(
        input: &[Option<String>], // Use a slice for better flexibility
        color: AppColors,
        spacing: usize,
    ) -> Span<'a> {
        let label = input
            .iter()
            .flatten()
            .map(|s| s.as_str())
            .collect::<Vec<_>>()
            .join(" ");

        let padded_text = format!("{label:<spacing$}", spacing = spacing);
        Self::highlighted_text_span(padded_text, color)
    }

    fn create_button_line<'a>(
        keys: &[Option<String>],
        desc: &KeyDescriptions,
        app_colors: AppColors,
        longest_button: usize,
    ) -> Line<'a> {
        Line::from(vec![
            Self::create_button(keys, app_colors, longest_button),
            Span::from(desc.as_str()),
        ])
    }

    fn gen_keymap_title(style: Style) -> Self {
        let lines = vec![
            Self::empty_line(),
            Line::from(Span::from("Keymap"))
                .style(style)
                .centered()
                .underlined(),
        ];
        let size = Self::calc_size(&lines);
        Self { lines, size }
    }

    fn gen_keymap(config: &Config) -> (Self, Self) {
        let columns = if config.keymap == Keymap::new() {
            DEFAULT_COLUMNS.clone()
        } else {
            KeymapColumns::custom(config)
        };
        columns.to_helpinfo(config)
    }

    fn gen_locations(config: &Config) -> Self {
        let mut entries = Vec::new();

        if let Some(path) = &config.dir_config {
            entries.push(("config location: ", path.display().to_string()));
        }
        if let Some(path) = &config.dir_save {
            entries.push(("export location: ", path.display().to_string()));
        }
        if config.show_timestamp {
            let tz = config
                .timezone
                .as_ref()
                .and_then(|t| t.iana_name())
                .unwrap_or("Etc/UTC");
            entries.push(("  logs timezone: ", tz.to_string()));
        }

        let max_len = entries
            .iter()
            .map(|(_, val)| val.chars().count())
            .max()
            .unwrap_or_default();

        // 3. Map entries to Lines
        let mut lines = entries
            .into_iter()
            .map(|(label, val)| {
                let spacing = " ".repeat(max_len.saturating_sub(val.chars().count()));
                Line::from(vec![
                    Self::text_span(label.to_owned(), config.app_colors),
                    Self::highlighted_text_span(format!("{spacing}{val}"), config.app_colors),
                ])
                .right_aligned()
            })
            .collect::<Vec<_>>();

        lines.extend([
            Self::empty_line(),
            Line::from(Self::text_span(
                "a work in progress, all and any input appreciated".to_owned(),
                config.app_colors,
            )),
            Self::highlighted_text_span(REPO.to_owned(), config.app_colors)
                .underlined()
                .into_centered_line(),
        ]);

        let size = Self::calc_size(&lines);
        Self { lines, size }
    }
}

// Draw the oxker name on one half, other half shoe logs location, save location, timezone
fn draw_top_section(
    f: &mut Frame,
    area: Rect,
    colors: AppColors,
    style: Style,
    name: HelpInfo,
    config_dir: HelpInfo,
) {
    let horizontal_split = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    let name_paragraph = Paragraph::new(name.lines)
        .style(
            Style::default()
                .bg(colors.popup_help.background)
                .fg(colors.popup_help.text_highlight),
        )
        .alignment(Alignment::Center);

    let location_top_padding = name.size.height.saturating_sub(config_dir.size.height);

    let right_vertical_split = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Max(location_top_padding),
            Constraint::Max(config_dir.size.height),
        ])
        .split(horizontal_split[1]);

    let config_paragraph = Paragraph::new(config_dir.lines).style(style);

    f.render_widget(name_paragraph, horizontal_split[0]);
    f.render_widget(config_paragraph, right_vertical_split[1]);
}

fn draw_keymap_title(f: &mut Frame, area: Rect, title: HelpInfo) {
    f.render_widget(Paragraph::new(title.lines), area);
}

fn draw_keymap(f: &mut Frame, area: Rect, style: Style, columns: (HelpInfo, HelpInfo)) {
    // Calculate some padding
    let horizontal_padding = area
        .width
        .saturating_sub(columns.0.size.width)
        .saturating_sub(columns.1.size.width)
        .saturating_div(2)
        .saturating_sub(1);

    let horizontal_split = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Max(horizontal_padding),
            Constraint::Max(columns.0.size.width),
            Constraint::Max(2),
            Constraint::Max(columns.1.size.width),
            Constraint::Max(horizontal_padding),
        ])
        .split(area);

    let left_column = Paragraph::new(columns.0.lines).style(style).left_aligned();
    let right_column = Paragraph::new(columns.1.lines).style(style).left_aligned();
    f.render_widget(left_column, horizontal_split[1]);
    f.render_widget(right_column, horizontal_split[3]);
}

pub fn draw(config: &Config, f: &mut Frame) {
    let default_colors = config.app_colors == AppColors::new();
    let title = format!(" {VERSION} ");
    let style = Style::default()
        .bg(config.app_colors.popup_help.background)
        .fg(config.app_colors.popup_help.text);

    let name_info = if default_colors {
        DEFAULT_NAME.clone()
    } else {
        HelpInfo::gen_name_description(config.app_colors)
    };
    let locations = HelpInfo::gen_locations(config);
    let keymap_title = HelpInfo::gen_keymap_title(style);
    let keymap_columns = HelpInfo::gen_keymap(config);

    let total_width = name_info
        .size
        .width
        // Account for spacing between the two sections
        .saturating_add(locations.size.width)
        .saturating_add(2)
        .max(
            keymap_columns
                .0
                .size
                .width
                .saturating_add(keymap_columns.1.size.width)
                // Account for the spacing spacing between each column
                .saturating_add(2),
        );
    let top_height = name_info.size.height.max(locations.size.height);
    let keymap_height = keymap_columns
        .0
        .size
        .height
        .max(keymap_columns.1.size.height);
    let total_height = top_height
        .saturating_add(keymap_title.size.height)
        .saturating_add(keymap_height);

    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(style)
        .style(style)
        .padding(Padding::horizontal(1));

    let area = popup::draw(
        (total_height + 2).into(),
        (total_width + 4).into(),
        f.area(),
        BoxLocation::MiddleCentre,
    );

    let inner_area = block.inner(area);

    let vertical_split = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(top_height),
            Constraint::Length(keymap_title.size.height),
            Constraint::Length(keymap_height),
        ])
        .split(inner_area);

    f.render_widget(Clear, area);
    f.render_widget(block, area);
    draw_top_section(
        f,
        vertical_split[0],
        config.app_colors,
        style,
        name_info,
        locations,
    );
    draw_keymap_title(f, vertical_split[1], keymap_title);
    draw_keymap(f, vertical_split[2], style, keymap_columns);
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::too_many_lines)]
mod tests {
    use std::path::PathBuf;

    use crate::config::{AppColors, Keymap};
    use crossterm::event::{KeyCode, KeyModifiers};
    use insta::assert_snapshot;
    use ratatui::style::Color;

    use crate::ui::draw_blocks::tests::{get_result, test_setup};

    #[test]
    /// This test is incredibly annoying
    /// println!("{} {} {} {} {}", row_index, result_cell_index, result_cell.symbol(), result_cell.bg, result_cell.fg);
    fn test_draw_blocks_help() {
        let mut setup = test_setup(118, 25, true, true);
        setup.app_data.lock().config.dir_save = Some(PathBuf::from("/test_dir"));
        setup.app_data.lock().config.dir_config =
            Some(PathBuf::from("/home/user/.config/oxker/config.toml"));
        setup.app_data.lock().config.show_timestamp = true;

        setup
            .terminal
            .draw(|f| {
                super::draw(&setup.app_data.lock().config, f);
            })
            .unwrap();

        assert_snapshot!(setup.terminal.backend());

        for (row_index, result_row) in get_result(&setup) {
            for (result_cell_index, result_cell) in result_row.iter().enumerate() {
                match (row_index, result_cell_index) {
                    // The space around the popup
                    (0|24, _) | (_, 0|117) => assert_eq!((result_cell.bg, result_cell.fg), (Color::Reset, Color::Reset)),
                    // The borders
                    (1|23, 1..=23) | (_, 1|116) => assert_eq!((result_cell.bg, result_cell.fg), (Color::Magenta, Color::Black)),
                    // The oxker logo
                    // The description
                    (2..=10, 3..=58)|
                    // Config location
                    (5, 79..=114) |
                    // Export location
                    (6, 79..=114) |
                    // Timezone
                    (7, 79..=114) |
                    //url
                    (10, 69..=104) |
                    // Left column
                    (13..=22, 4..=24) |
                    // Right Column
                    (13..=21,59..=69)
                     => assert_eq!((result_cell.bg, result_cell.fg), (Color::Magenta, Color::White)),
                    _ =>  assert_eq!((result_cell.bg, result_cell.fg), (Color::Magenta, Color::Black)),
                };
            }
        }
    }

    #[test]
    fn test_draw_blocks_help_no_config() {
        let mut setup = test_setup(116, 25, true, true);
        setup.app_data.lock().config.dir_save = Some(PathBuf::from("/test_dir"));
        setup.app_data.lock().config.show_timestamp = true;

        setup
            .terminal
            .draw(|f| {
                super::draw(&setup.app_data.lock().config, f);
            })
            .unwrap();

        assert_snapshot!(setup.terminal.backend());

        for (row_index, result_row) in get_result(&setup) {
            for (result_cell_index, result_cell) in result_row.iter().enumerate() {
                match (row_index, result_cell_index) {
                    // The space around the popup
                    (0|24, _) | (_, 0|115) => assert_eq!((result_cell.bg, result_cell.fg), (Color::Reset, Color::Reset)),
                    // The borders
                    (1|23, 1..=23) | (_, 1|114) => assert_eq!((result_cell.bg, result_cell.fg), (Color::Magenta, Color::Black)),
                    // The oxker logo
                    // The description
                    (2..=10, 3..=57)|
                    // Export location
                    (6, 104..=112) |
                    // Timezone
                    (7, 104..=112) |
                    //url
                    (10, 67..=102) |
                    // Left column
                    (13..=22, 3..=23) |
                    // Right Column
                    (13..=21,58..=68)
                     => assert_eq!((result_cell.bg, result_cell.fg), (Color::Magenta, Color::White)),
                    _ =>  assert_eq!((result_cell.bg, result_cell.fg), (Color::Magenta, Color::Black)),
            };
            }
        }
    }

    #[test]
    fn test_draw_blocks_help_no_save() {
        let mut setup = test_setup(118, 25, true, true);
        setup.app_data.lock().config.dir_config =
            Some(PathBuf::from("/home/user/.config/oxker/config.toml"));
        setup.app_data.lock().config.show_timestamp = true;

        setup
            .terminal
            .draw(|f| {
                super::draw(&setup.app_data.lock().config, f);
            })
            .unwrap();

        assert_snapshot!(setup.terminal.backend());

        for (row_index, result_row) in get_result(&setup) {
            for (result_cell_index, result_cell) in result_row.iter().enumerate() {
                match (row_index, result_cell_index) {
                    // The space around the popup
                    (0|24, _) | (_, 0|117) => assert_eq!((result_cell.bg, result_cell.fg), (Color::Reset, Color::Reset)),
                    // The borders
                    (1|23, 1..=23) | (_, 1|116) => assert_eq!((result_cell.bg, result_cell.fg), (Color::Magenta, Color::Black)),
                    // The oxker logo
                    // The description
                    (2..=10, 3..=58)|
                    // Config location
                    (6, 79..=114) |
                    // Timezone
                    (7, 79..=114) |
                    //url
                    (10, 69..=104) |
                    // Left column
                    (13..=22, 4..=24) |
                    // Right Column
                    (13..=21,59..=69)
                     => assert_eq!((result_cell.bg, result_cell.fg), (Color::Magenta, Color::White)),
                    _ =>  assert_eq!((result_cell.bg, result_cell.fg), (Color::Magenta, Color::Black)),
                };
            }
        }
    }

    #[test]
    fn test_draw_blocks_help_no_timezone() {
        let mut setup = test_setup(118, 25, true, true);
        setup.app_data.lock().config.dir_save = Some(PathBuf::from("/test_dir"));
        setup.app_data.lock().config.dir_config =
            Some(PathBuf::from("/home/user/.config/oxker/config.toml"));

        setup
            .terminal
            .draw(|f| {
                super::draw(&setup.app_data.lock().config, f);
            })
            .unwrap();

        assert_snapshot!(setup.terminal.backend());

        for (row_index, result_row) in get_result(&setup) {
            for (result_cell_index, result_cell) in result_row.iter().enumerate() {
                match (row_index, result_cell_index) {
                    // The space around the popup
                    (0|24, _) | (_, 0|117) => assert_eq!((result_cell.bg, result_cell.fg), (Color::Reset, Color::Reset)),
                    // The borders
                    (1|23, 1..=23) | (_, 1|116) => assert_eq!((result_cell.bg, result_cell.fg), (Color::Magenta, Color::Black)),
                    // The oxker logo
                    // The description
                    (2..=10, 3..=58)|
                    // Config location
                    (6, 79..=114) |
                    // Export location
                    (7, 79..=114) |
                    //url
                    (10, 69..=104) |
                    // Left column
                    (13..=22, 4..=24) |
                    // Right Column
                    (13..=21,59..=69)
                     => assert_eq!((result_cell.bg, result_cell.fg), (Color::Magenta, Color::White)),
                    _ =>  assert_eq!((result_cell.bg, result_cell.fg), (Color::Magenta, Color::Black)),
                };
            }
        }
    }

    #[test]
    fn test_draw_blocks_help_custom_color() {
        let mut setup = test_setup(118, 25, true, true);
        setup.app_data.lock().config.dir_save = Some(PathBuf::from("/test_dir"));
        setup.app_data.lock().config.dir_config =
            Some(PathBuf::from("/home/user/.config/oxker/config.toml"));
        setup.app_data.lock().config.show_timestamp = true;

        let mut colors = AppColors::new();
        colors.popup_help.background = Color::Black;
        colors.popup_help.text = Color::Red;
        colors.popup_help.text_highlight = Color::Yellow;

        setup.app_data.lock().config.app_colors = colors;

        setup
            .terminal
            .draw(|f| {
                super::draw(&setup.app_data.lock().config, f);
            })
            .unwrap();

        assert_snapshot!(setup.terminal.backend());

        for (row_index, result_row) in get_result(&setup) {
            for (result_cell_index, result_cell) in result_row.iter().enumerate() {
                match (row_index, result_cell_index) {
                    // The space around the popup
                    (0|24, _) | (_, 0|117) => assert_eq!((result_cell.bg, result_cell.fg), (Color::Reset, Color::Reset)),
                    // The borders
                    (1|23, 1..=23) | (_, 1|116) => assert_eq!((result_cell.bg, result_cell.fg), (Color::Black, Color::Red)),
                    // The oxker logo
                    // The description
                    (2..=10, 3..=58)|
                    // Config location
                    (5, 79..=114) |
                    // Export location
                    (6, 79..=114) |
                    // Timezone
                    (7, 79..=114) |
                    //url
                    (10, 69..=104) |
                    // Left column
                    (13..=22, 4..=24) |
                    // Right Column
                    (13..=21,59..=69)
                     => assert_eq!((result_cell.bg, result_cell.fg), (Color::Black, Color::Yellow)),
                    _ =>  assert_eq!((result_cell.bg, result_cell.fg), (Color::Black, Color::Red)),
                };
            }
        }
    }

    #[test]
    /// Help panel will show custom keymap if in use, with one definition for each entry
    fn test_draw_blocks_help_custom_keymap_one_definition() {
        let mut setup = test_setup(118, 25, true, true);

        setup.app_data.lock().config.dir_save = Some(PathBuf::from("/test_dir"));
        setup.app_data.lock().config.dir_config =
            Some(PathBuf::from("/home/user/.config/oxker/config.toml"));
        setup.app_data.lock().config.show_timestamp = true;

        let keymap = Keymap {
            clear: (KeyCode::Char('a'), None),
            delete_confirm: (KeyCode::Char('b'), None),
            delete_deny: (KeyCode::Char('c'), None),
            exec: (KeyCode::Char('d'), None),
            inspect: (KeyCode::Char('e'), None),
            filter_mode: (KeyCode::Char('f'), None),
            log_search_mode: (KeyCode::Char('g'), None),
            force_redraw: (KeyCode::Char('h'), None),
            scroll_back: (KeyCode::Char('i'), None),
            scroll_forward: (KeyCode::Char('j'), None),
            log_section_height_decrease: (KeyCode::Char('k'), None),
            log_section_height_increase: (KeyCode::Char('l'), None),
            log_section_toggle: (KeyCode::Char('m'), None),
            quit: (KeyCode::Char('n'), None),
            save_logs: (KeyCode::Char('o'), None),
            scroll_down: (KeyCode::Char('p'), None),
            scroll_end: (KeyCode::Char('q'), None),
            scroll_many: KeyModifiers::ALT,
            scroll_start: (KeyCode::Char('r'), None),
            scroll_up: (KeyCode::Char('s'), None),
            select_next_panel: (KeyCode::Char('t'), None),
            select_previous_panel: (KeyCode::Char('u'), None),
            sort_by_cpu: (KeyCode::Char('v'), None),
            sort_by_id: (KeyCode::Char('w'), None),
            sort_by_image: (KeyCode::Char('x'), None),
            sort_by_memory: (KeyCode::Char('y'), None),
            sort_by_name: (KeyCode::Char('z'), None),
            sort_by_rx: (KeyCode::Char('0'), None),
            sort_by_state: (KeyCode::Char('1'), None),
            sort_by_status: (KeyCode::Char('2'), None),
            sort_by_tx: (KeyCode::Char('3'), None),
            sort_reset: (KeyCode::Char('4'), None),
            toggle_help: (KeyCode::Char('5'), None),
            toggle_mouse_capture: (KeyCode::Char('6'), None),
        };

        setup.app_data.lock().config.keymap = keymap;

        setup
            .terminal
            .draw(|f| {
                super::draw(&setup.app_data.lock().config, f);
            })
            .unwrap();

        assert_snapshot!(setup.terminal.backend());
    }

    #[test]
    /// Help panel will show custom keymap if in use, with two definitions for each entry
    fn test_draw_blocks_help_custom_keymap_two_definition() {
        let mut setup = test_setup(124, 30, true, true);

        setup.app_data.lock().config.dir_save = Some(PathBuf::from("/test_dir"));
        setup.app_data.lock().config.dir_config =
            Some(PathBuf::from("/home/user/.config/oxker/config.toml"));
        setup.app_data.lock().config.show_timestamp = true;

        let keymap = Keymap {
            clear: (KeyCode::Char('a'), Some(KeyCode::Char('b'))),
            delete_confirm: (KeyCode::Char('c'), Some(KeyCode::Char('d'))),
            delete_deny: (KeyCode::Char('e'), Some(KeyCode::Char('f'))),
            exec: (KeyCode::Char('g'), Some(KeyCode::Char('h'))),
            inspect: (KeyCode::Char('i'), Some(KeyCode::Char('j'))),
            filter_mode: (KeyCode::Char('k'), Some(KeyCode::Char('l'))),
            log_search_mode: (KeyCode::Char('m'), Some(KeyCode::Char('n'))),
            force_redraw: (KeyCode::Char('o'), Some(KeyCode::Char('p'))),
            scroll_back: (KeyCode::Char('q'), Some(KeyCode::Char('r'))),
            scroll_forward: (KeyCode::Char('s'), Some(KeyCode::Char('t'))),
            log_section_height_decrease: (KeyCode::Char('u'), Some(KeyCode::Char('v'))),
            log_section_height_increase: (KeyCode::Char('w'), Some(KeyCode::Char('x'))),
            log_section_toggle: (KeyCode::Char('y'), Some(KeyCode::Char('z'))),
            quit: (KeyCode::Char('0'), Some(KeyCode::Char('1'))),
            save_logs: (KeyCode::Char('2'), Some(KeyCode::Char('3'))),
            scroll_down: (KeyCode::Char('4'), Some(KeyCode::Char('5'))),
            scroll_end: (KeyCode::Char('6'), Some(KeyCode::Char('7'))),
            scroll_many: KeyModifiers::ALT,
            scroll_start: (KeyCode::Char('8'), Some(KeyCode::Char('9'))),
            scroll_up: (KeyCode::CapsLock, Some(KeyCode::ScrollLock)),
            select_next_panel: (KeyCode::PrintScreen, Some(KeyCode::Right)),
            select_previous_panel: (KeyCode::Left, Some(KeyCode::Up)),
            sort_by_cpu: (KeyCode::Down, Some(KeyCode::Esc)),
            sort_by_id: (KeyCode::BackTab, Some(KeyCode::Insert)),
            sort_by_image: (KeyCode::End, Some(KeyCode::Menu)),
            sort_by_memory: (KeyCode::Home, Some(KeyCode::PageDown)),
            sort_by_name: (KeyCode::KeypadBegin, Some(KeyCode::Tab)),
            sort_by_rx: (KeyCode::NumLock, Some(KeyCode::Pause)),
            sort_by_state: (KeyCode::PageUp, Some(KeyCode::F(1))),
            sort_by_status: (KeyCode::PrintScreen, Some(KeyCode::F(2))),
            sort_by_tx: (KeyCode::F(3), Some(KeyCode::F(4))),
            sort_reset: (KeyCode::F(5), Some(KeyCode::F(6))),
            toggle_help: (KeyCode::F(7), Some(KeyCode::F(8))),
            toggle_mouse_capture: (KeyCode::F(9), Some(KeyCode::F(10))),
        };

        setup.app_data.lock().config.keymap = keymap;

        setup
            .terminal
            .draw(|f| {
                super::draw(&setup.app_data.lock().config, f);
            })
            .unwrap();

        assert_snapshot!(setup.terminal.backend());
    }

    #[test]
    /// Help panel will show custom keymap if in use, with one or two definitions for each entry
    fn test_draw_blocks_help_custom_keymap_one_two_definition() {
        let mut setup = test_setup(124, 30, true, true);

        setup.app_data.lock().config.dir_save = Some(PathBuf::from("/test_dir"));
        setup.app_data.lock().config.dir_config =
            Some(PathBuf::from("/home/user/.config/oxker/config.toml"));
        setup.app_data.lock().config.show_timestamp = true;

        let keymap = Keymap {
            clear: (KeyCode::Char('a'), Some(KeyCode::Char('b'))),
            delete_confirm: (KeyCode::Char('c'), None),
            delete_deny: (KeyCode::Char('e'), Some(KeyCode::Char('f'))),
            exec: (KeyCode::Char('g'), None),
            inspect: (KeyCode::Char('i'), Some(KeyCode::Char('j'))),
            filter_mode: (KeyCode::Char('k'), None),
            log_search_mode: (KeyCode::Char('m'), Some(KeyCode::Char('n'))),
            force_redraw: (KeyCode::Char('o'), None),
            scroll_back: (KeyCode::Char('q'), Some(KeyCode::Char('r'))),
            scroll_forward: (KeyCode::Char('s'), None),
            log_section_height_decrease: (KeyCode::Char('u'), Some(KeyCode::Char('v'))),
            log_section_height_increase: (KeyCode::Char('w'), None),
            log_section_toggle: (KeyCode::Char('y'), Some(KeyCode::Char('z'))),
            quit: (KeyCode::Char('0'), None),
            save_logs: (KeyCode::Char('2'), Some(KeyCode::Char('3'))),
            scroll_down: (KeyCode::Char('4'), None),
            scroll_end: (KeyCode::Char('6'), Some(KeyCode::Char('7'))),
            scroll_many: KeyModifiers::ALT,
            scroll_start: (KeyCode::Char('8'), None),
            scroll_up: (KeyCode::CapsLock, Some(KeyCode::ScrollLock)),
            select_next_panel: (KeyCode::PrintScreen, None),
            select_previous_panel: (KeyCode::Left, Some(KeyCode::Up)),
            sort_by_cpu: (KeyCode::Down, None),
            sort_by_id: (KeyCode::BackTab, None),
            sort_by_image: (KeyCode::End, Some(KeyCode::Esc)),
            sort_by_memory: (KeyCode::Home, None),
            sort_by_name: (KeyCode::KeypadBegin, Some(KeyCode::Menu)),
            sort_by_rx: (KeyCode::NumLock, None),
            sort_by_state: (KeyCode::PageUp, Some(KeyCode::Pause)),
            sort_by_status: (KeyCode::PrintScreen, None),
            sort_by_tx: (KeyCode::F(1), Some(KeyCode::F(2))),
            sort_reset: (KeyCode::F(3), None),
            toggle_help: (KeyCode::F(5), Some(KeyCode::F(6))),
            toggle_mouse_capture: (KeyCode::F(7), None),
        };

        setup.app_data.lock().config.keymap = keymap;

        setup
            .terminal
            .draw(|f| {
                super::draw(&setup.app_data.lock().config, f);
            })
            .unwrap();

        assert_snapshot!(setup.terminal.backend());
    }
}
