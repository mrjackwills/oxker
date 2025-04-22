use crossterm::event::KeyCode;
use jiff::tz::TimeZone;
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, Paragraph},
};

use crate::{
    config::{AppColors, Keymap},
    ui::gui_state::BoxLocation,
};

use super::{DESCRIPTION, NAME_TEXT, REPO, VERSION, popup};

/// Help popup box needs these three pieces of information
struct HelpInfo {
    lines: Vec<Line<'static>>,
    width: usize,
    height: usize,
}

impl HelpInfo {
    /// Find the max width of a Span in &[Line]
    fn calc_width(lines: &[Line]) -> usize {
        lines
            .iter()
            .map(ratatui::prelude::Line::width)
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
    fn text_span<'a>(input: &str, color: AppColors) -> Span<'a> {
        Self::span(input, color.popup_help.text)
    }

    /// &str to white text span
    fn highlighted_text_span<'a>(input: &str, color: AppColors) -> Span<'a> {
        Self::span(input, color.popup_help.text_highlight)
    }

    /// Generate the `oxker` name span + metadata
    fn gen_name(colors: AppColors) -> Self {
        let mut lines = NAME_TEXT
            .lines()
            .map(|i| Line::from(Self::highlighted_text_span(i, colors)))
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
    fn gen_description(colors: AppColors) -> Self {
        let lines = [
            Self::empty_span(),
            Line::from(Self::highlighted_text_span(DESCRIPTION, colors)),
            Self::empty_span(),
        ];

        Self {
            lines: lines.to_vec(),
            width: Self::calc_width(&lines),
            height: lines.len(),
        }
    }

    /// Generate the button information span + metadata
    fn gen_keymap_info(colors: AppColors, zone: Option<&TimeZone>, show_timestamp: bool) -> Self {
        let button_item = |x: &str| Self::highlighted_text_span(&format!(" ( {x} ) "), colors);
        let button_desc = |x: &str| Self::text_span(x, colors);
        let or = || button_desc("or");
        let space = || button_desc(" ");

        let descriptions = [
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
                button_desc("toggle this help information - or click heading"),
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

        let mut lines = if show_timestamp {
            Vec::from([
                Self::custom_text(colors, &Keymap::new(), zone),
                Self::empty_span(),
            ])
        } else {
            vec![]
        };

        lines.extend_from_slice(&descriptions);
        let width = Self::calc_width(&lines);
        let height = lines.len();

        Self {
            lines,
            width,
            height,
        }
    }

    /// Generate the final lines, GitHub link etc, + metadata
    fn gen_final(colors: AppColors) -> Self {
        let lines = [
            Self::empty_span(),
            Line::from(vec![Self::text_span(
                "currently an early work in progress, all and any input appreciated",
                colors,
            )]),
            Line::from(vec![Span::styled(
                REPO,
                Style::default()
                    .fg(colors.popup_help.text_highlight)
                    .add_modifier(Modifier::UNDERLINED),
            )]),
        ];

        Self {
            lines: lines.to_vec(),
            width: Self::calc_width(&lines),
            height: lines.len(),
        }
    }

    /// Display timezone in timestamps are visible
    /// Has ability to display if keymap or colors are customized, but currently not in use
    fn custom_text<'a>(colors: AppColors, _keymap: &Keymap, zone: Option<&TimeZone>) -> Line<'a> {
        let highlighted = |x: &str| Self::highlighted_text_span(x, colors);
        let text = |x: &str| Self::text_span(x, colors);

        // if keymap != &Keymap::new() {
        //     op.push(highlighted("customised keymap, "));
        // }

        // if colors != AppColors::new() {
        //     op.push(highlighted("customised app colors, "));
        // };

        let zone = zone.and_then(|i| i.iana_name()).unwrap_or("Etc/UTC");
        Line::from(Vec::from([text("logs timezone: "), highlighted(zone)])).centered()
    }

    /// Generate the display information when a custom keymap is being used
    fn gen_custom_keymap_info(
        colors: AppColors,
        km: &Keymap,
        zone: Option<&TimeZone>,
        show_timestamp: bool,
    ) -> Self {
        let button_item = |x: &str| Self::highlighted_text_span(&format!(" ( {x} ) "), colors);
        let button_desc = |x: &str| Self::text_span(x, colors);
        let or = || button_desc("or");
        let space = || button_desc(" ");

        let or_secondary = |a: (KeyCode, Option<KeyCode>), desc: &str| {
            a.1.map_or_else(
                || {
                    Line::from(vec![
                        space(),
                        button_item(&a.0.to_string()),
                        button_desc(desc),
                    ])
                },
                |secondary| {
                    Line::from(vec![
                        space(),
                        button_item(&a.0.to_string()),
                        or(),
                        button_item(&secondary.to_string()),
                        button_desc(desc),
                    ])
                },
            )
        };
        let descriptions = [
            or_secondary(km.select_next_panel, "select next panel"),
            or_secondary(km.select_previous_panel, "select previous panel"),
            or_secondary(km.scroll_down_one, "scroll list down by one"),
            or_secondary(km.scroll_up_one, "scroll list up by one"),
            or_secondary(km.scroll_down_many, "scroll list down by many"),
            or_secondary(km.scroll_up_many, "scroll list by up many"),
            or_secondary(km.scroll_end, "scroll list to end"),
            or_secondary(km.scroll_start, "scroll list to start"),
            Line::from(vec![
                space(),
                button_item("enter"),
                button_desc("send docker container command"),
            ]),
            #[cfg(not(target_os = "windows"))]
            or_secondary(km.exec, "exec into a container"),
            #[cfg(target_os = "windows")]
            or_secondary(km.exec, "exec into a container - not available on Windows"),
            or_secondary(
                km.toggle_help,
                "toggle this help information - or click heading",
            ),
            or_secondary(km.save_logs, "save logs to file"),
            or_secondary(
                km.toggle_mouse_capture,
                "toggle mouse capture - if disabled, text on screen can be selected & copied",
            ),
            or_secondary(km.filter_mode, "enter filter mode"),
            or_secondary(km.sort_reset, "reset container sorting"),
            or_secondary(km.sort_by_name, "sort containers by name"),
            or_secondary(km.sort_by_state, "sort containers by state"),
            or_secondary(km.sort_by_status, "sort containers by status"),
            or_secondary(km.sort_by_cpu, "sort containers by cpu"),
            or_secondary(km.sort_by_memory, "sort containers by memory"),
            or_secondary(km.sort_by_id, "sort containers by id"),
            or_secondary(km.sort_by_image, "sort containers by image"),
            or_secondary(km.sort_by_rx, "sort containers by rx"),
            or_secondary(km.sort_by_tx, "sort containers by tx"),
            or_secondary(km.clear, "close dialog"),
            or_secondary(km.quit, "quit at any time"),
        ];

        let mut lines = if show_timestamp {
            Vec::from([Self::custom_text(colors, km, zone), Self::empty_span()])
        } else {
            vec![]
        };

        lines.extend_from_slice(&descriptions);
        let width = Self::calc_width(&lines);
        let height = lines.len();

        Self {
            lines,
            width,
            height,
        }
    }
}

/// Draw the help box in the centre of the screen
pub fn draw(
    colors: AppColors,
    f: &mut Frame,
    keymap: &Keymap,
    show_timestamp: bool,
    zone: Option<&TimeZone>,
) {
    let title = format!(" {VERSION} ");

    let name_info = HelpInfo::gen_name(colors);
    let description_info = HelpInfo::gen_description(colors);
    let final_info = HelpInfo::gen_final(colors);

    let button_info = if keymap == &Keymap::new() {
        HelpInfo::gen_keymap_info(colors, zone, show_timestamp)
    } else {
        HelpInfo::gen_custom_keymap_info(colors, keymap, zone, show_timestamp)
    };

    let max_line_width = [
        name_info.width,
        description_info.width,
        button_info.width,
        final_info.width,
    ]
    .into_iter()
    .max()
    .unwrap_or_default()
        + 2;

    let max_height =
        name_info.height + description_info.height + button_info.height + final_info.height + 2;

    let area = popup::draw(
        max_height,
        max_line_width,
        f.area(),
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
        .style(
            Style::default()
                .bg(colors.popup_help.background)
                .fg(colors.popup_help.text_highlight),
        )
        .alignment(Alignment::Center);

    let style = || {
        Style::default()
            .bg(colors.popup_help.background)
            .fg(colors.popup_help.text)
    };
    let description_paragraph = Paragraph::new(description_info.lines)
        .style(style())
        .alignment(Alignment::Center);

    let help_paragraph = Paragraph::new(button_info.lines)
        .style(style())
        .alignment(Alignment::Left);

    let final_paragraph = Paragraph::new(final_info.lines)
        .style(style())
        .alignment(Alignment::Center);

    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(
            Style::default()
                .fg(colors.popup_help.text)
                .bg(colors.popup_help.background),
        );

    // Order is important here
    f.render_widget(Clear, area);
    f.render_widget(name_paragraph, split_popup[0]);
    f.render_widget(description_paragraph, split_popup[1]);
    f.render_widget(help_paragraph, split_popup[2]);
    f.render_widget(final_paragraph, split_popup[3]);
    f.render_widget(block, area);
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::too_many_lines)]
mod tests {
    use crate::config::{AppColors, Keymap};
    use crossterm::event::KeyCode;
    use insta::assert_snapshot;
    use jiff::tz::TimeZone;
    use ratatui::style::{Color, Modifier};

    use crate::ui::draw_blocks::tests::{get_result, test_setup};

    #[test]
    /// This will cause issues once the version has more than the current 5 chars (0.5.0)
    fn test_draw_blocks_help() {
        let mut setup = test_setup(87, 33, true, true);
        let tz = setup.app_data.lock().config.timezone.clone();

        setup
            .terminal
            .draw(|f| {
                super::draw(
                    AppColors::new(),
                    f,
                    &setup.app_data.lock().config.keymap,
                    false,
                    tz.as_ref(),
                );
            })
            .unwrap();

        assert_snapshot!(setup.terminal.backend());

        for (row_index, result_row) in get_result(&setup) {
            for (result_cell_index, result_cell) in result_row.iter().enumerate() {
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

    #[test]
    /// Test that the help panel gets drawn with custom colors
    fn test_draw_blocks_help_custom_colors() {
        let mut setup = test_setup(87, 33, true, true);
        let mut colors = AppColors::new();
        let tz = setup.app_data.lock().config.timezone.clone();

        colors.popup_help.background = Color::Black;
        colors.popup_help.text = Color::Red;
        colors.popup_help.text_highlight = Color::Yellow;

        setup
            .terminal
            .draw(|f| {
                super::draw(
                    colors,
                    f,
                    &setup.app_data.lock().config.keymap,
                    false,
                    tz.as_ref(),
                );
            })
            .unwrap();

        assert_snapshot!(setup.terminal.backend());

        for (row_index, result_row) in get_result(&setup) {
            for (result_cell_index, result_cell) in result_row.iter().enumerate() {
                match (row_index, result_cell_index) {
                    // first & last row, and first & last char on each row, is reset/reset, making sure that the help info is centered in the given area
                    (0 | 32, _) | (0..=33, 0 | 86) => {
                        assert_eq!(result_cell.bg, Color::Reset);
                        assert_eq!(result_cell.fg, Color::Reset);
                    }
                    // border is black on magenta
                    (1 | 31, _) | (1..=31, 1 | 85) => {
                        assert_eq!(result_cell.bg, Color::Black);
                        assert_eq!(result_cell.fg, Color::Red);
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
                        assert_eq!(result_cell.bg, Color::Black);
                        assert_eq!(result_cell.fg, Color::Yellow);
                    }
                    // The URL is yellow and underlined
                    (28, 25..=60) => {
                        assert_eq!(result_cell.bg, Color::Black);
                        assert_eq!(result_cell.fg, Color::Yellow);
                        assert_eq!(result_cell.modifier, Modifier::UNDERLINED);
                    }
                    // The rest is red on black
                    _ => {
                        assert_eq!(result_cell.bg, Color::Black);
                        assert_eq!(result_cell.fg, Color::Red);
                    }
                }
            }
        }
    }

    #[test]
    /// Help panel will show custom keymap if in use, with one definition for each entry
    fn test_draw_blocks_help_custom_keymap_one_definition() {
        let mut setup = test_setup(98, 47, true, true);

        let input = Keymap {
            clear: (KeyCode::Char('a'), None),
            delete_deny: (KeyCode::Char('c'), None),
            delete_confirm: (KeyCode::Char('e'), None),
            exec: (KeyCode::Char('g'), None),
            filter_mode: (KeyCode::Char('i'), None),
            quit: (KeyCode::Char('k'), None),
            save_logs: (KeyCode::Char('m'), None),
            scroll_down_many: (KeyCode::Char('o'), None),
            scroll_down_one: (KeyCode::Char('q'), None),
            scroll_end: (KeyCode::Char('s'), None),
            scroll_start: (KeyCode::Char('u'), None),
            scroll_up_many: (KeyCode::Char('w'), None),
            scroll_up_one: (KeyCode::Char('y'), None),
            select_next_panel: (KeyCode::Char('0'), None),
            select_previous_panel: (KeyCode::Char('2'), None),
            sort_by_name: (KeyCode::Char('4'), None),
            sort_by_state: (KeyCode::Char('6'), None),
            sort_by_status: (KeyCode::Char('8'), None),
            sort_by_cpu: (KeyCode::F(1), None),
            sort_by_memory: (KeyCode::Char('#'), None),
            sort_by_id: (KeyCode::Char('/'), None),
            sort_by_image: (KeyCode::Char(','), None),
            sort_by_rx: (KeyCode::Char('.'), None),
            sort_by_tx: (KeyCode::Insert, None),
            sort_reset: (KeyCode::Up, None),
            toggle_help: (KeyCode::Home, None),
            toggle_mouse_capture: (KeyCode::PageDown, None),
        };

        setup
            .terminal
            .draw(|f| {
                super::draw(AppColors::new(), f, &input, false, None);
            })
            .unwrap();

        assert_snapshot!(setup.terminal.backend());
    }

    #[test]
    /// Help panel will show custom keymap if in use, with two definition for each entry
    fn test_draw_blocks_help_custom_keymap_two_definitions() {
        let mut setup = test_setup(110, 47, true, true);

        let keymap = Keymap {
            clear: (KeyCode::Char('a'), Some(KeyCode::Char('b'))),
            delete_deny: (KeyCode::Char('c'), Some(KeyCode::Char('d'))),
            delete_confirm: (KeyCode::Char('e'), Some(KeyCode::Char('f'))),
            exec: (KeyCode::Char('g'), Some(KeyCode::Char('h'))),
            filter_mode: (KeyCode::Char('i'), Some(KeyCode::Char('j'))),
            quit: (KeyCode::Char('k'), Some(KeyCode::Char('l'))),
            save_logs: (KeyCode::Char('m'), Some(KeyCode::Char('n'))),
            scroll_down_many: (KeyCode::Char('o'), Some(KeyCode::Char('p'))),
            scroll_down_one: (KeyCode::Char('q'), Some(KeyCode::Char('r'))),
            scroll_end: (KeyCode::Char('s'), Some(KeyCode::Char('t'))),
            scroll_start: (KeyCode::Char('u'), Some(KeyCode::Char('v'))),
            scroll_up_many: (KeyCode::Char('w'), Some(KeyCode::Char('x'))),
            scroll_up_one: (KeyCode::Char('y'), Some(KeyCode::Char('z'))),
            select_next_panel: (KeyCode::Char('0'), Some(KeyCode::Char('1'))),
            select_previous_panel: (KeyCode::Char('2'), Some(KeyCode::Char('3'))),
            sort_by_name: (KeyCode::Char('4'), Some(KeyCode::Char('5'))),
            sort_by_state: (KeyCode::Char('6'), Some(KeyCode::Char('7'))),
            sort_by_status: (KeyCode::Char('8'), Some(KeyCode::Char('9'))),
            sort_by_cpu: (KeyCode::F(1), Some(KeyCode::F(12))),
            sort_by_memory: (KeyCode::Char('#'), Some(KeyCode::Char('-'))),
            sort_by_id: (KeyCode::Char('/'), Some(KeyCode::Char('='))),
            sort_by_image: (KeyCode::Char(','), Some(KeyCode::Char('\\'))),
            sort_by_rx: (KeyCode::Char('.'), Some(KeyCode::Char(']'))),
            sort_by_tx: (KeyCode::Insert, Some(KeyCode::BackTab)),
            sort_reset: (KeyCode::Up, Some(KeyCode::Down)),
            toggle_help: (KeyCode::Home, Some(KeyCode::Delete)),
            toggle_mouse_capture: (KeyCode::PageDown, Some(KeyCode::PageUp)),
        };

        setup
            .terminal
            .draw(|f| {
                super::draw(AppColors::new(), f, &keymap, false, None);
            })
            .unwrap();

        assert_snapshot!(setup.terminal.backend());
    }

    #[test]
    /// Help panel will show custom keymap if in use, with either one or two definition for each entry
    fn test_draw_blocks_help_one_and_two_definitions() {
        let mut setup = test_setup(110, 47, true, true);

        let keymap = Keymap {
            clear: (KeyCode::Char('a'), Some(KeyCode::Char('b'))),
            delete_deny: (KeyCode::Char('c'), None),
            delete_confirm: (KeyCode::Char('e'), Some(KeyCode::Char('f'))),
            exec: (KeyCode::Char('g'), None),
            filter_mode: (KeyCode::Char('i'), Some(KeyCode::Char('j'))),
            quit: (KeyCode::Char('k'), None),
            save_logs: (KeyCode::Char('m'), Some(KeyCode::Char('n'))),
            scroll_down_many: (KeyCode::Char('o'), None),
            scroll_down_one: (KeyCode::Char('q'), Some(KeyCode::Char('r'))),
            scroll_end: (KeyCode::Char('s'), None),
            scroll_start: (KeyCode::Char('u'), Some(KeyCode::Char('v'))),
            scroll_up_many: (KeyCode::Char('w'), None),
            scroll_up_one: (KeyCode::Char('y'), Some(KeyCode::Char('z'))),
            select_next_panel: (KeyCode::Char('0'), None),
            select_previous_panel: (KeyCode::Char('2'), Some(KeyCode::Char('3'))),
            sort_by_name: (KeyCode::Char('4'), None),
            sort_by_state: (KeyCode::Char('6'), Some(KeyCode::Char('7'))),
            sort_by_status: (KeyCode::Char('8'), None),
            sort_by_cpu: (KeyCode::F(1), Some(KeyCode::F(12))),
            sort_by_memory: (KeyCode::Char('#'), None),
            sort_by_id: (KeyCode::Char('/'), Some(KeyCode::Char('='))),
            sort_by_image: (KeyCode::Char(','), None),
            sort_by_rx: (KeyCode::Char('.'), Some(KeyCode::Char(']'))),
            sort_by_tx: (KeyCode::Insert, None),
            sort_reset: (KeyCode::Up, Some(KeyCode::Down)),
            toggle_help: (KeyCode::Home, None),
            toggle_mouse_capture: (KeyCode::PageDown, Some(KeyCode::PageUp)),
        };

        let tz = setup.app_data.lock().config.timezone.clone();

        setup
            .terminal
            .draw(|f| {
                super::draw(AppColors::new(), f, &keymap, false, tz.as_ref());
            })
            .unwrap();

        assert_snapshot!(setup.terminal.backend());
    }

    #[test]
    fn test_draw_blocks_help_show_timezone() {
        let mut setup = test_setup(87, 35, true, true);

        setup
            .terminal
            .draw(|f| {
                super::draw(
                    AppColors::new(),
                    f,
                    &Keymap::new(),
                    true,
                    Some(&TimeZone::get("asia/tokyo").unwrap()),
                );
            })
            .unwrap();

        assert_snapshot!(setup.terminal.backend());

        for (row_index, result_row) in get_result(&setup) {
            for (result_cell_index, result_cell) in result_row.iter().enumerate() {
                match (row_index, result_cell_index) {
                    (14, 31..=45) => {
                        assert_eq!(result_cell.fg, AppColors::new().popup_help.text);
                    }
                    (14, 46..=55) => {
                        assert_eq!(result_cell.fg, AppColors::new().popup_help.text_highlight);
                    }
                    _ => (),
                }
            }
        }
    }
}
