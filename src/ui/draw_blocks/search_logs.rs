use crossterm::event::KeyCode;
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
};

use crate::{
    app_data::LogsButton,
    config::{AppColors, Keymap},
    ui::FrameData,
};

// background, text, selected_text, highlight;
/// Draw the filter bar
pub fn draw(area: Rect, colors: AppColors, frame: &mut Frame, fd: &FrameData, keymap: &Keymap) {
    let style_but = Style::default()
        .fg(colors.log_search.button_text)
        .bg(colors.log_search.highlight);
    let style_desc = Style::default()
        .fg(colors.log_search.text)
        .bg(colors.log_search.background);
    let space = || Span::from(" ");

    let mut line = vec![
        Span::styled(" Esc ", style_but),
        Span::styled(" clear ", style_desc),
        space(),
    ];
    line.extend([Span::styled(
        " search term: ",
        Style::default()
            .fg(colors.log_search.highlight)
            .bg(colors.log_search.background)
            .add_modifier(Modifier::BOLD),
    )]);

    if let Some(log_search) = fd.log_search.as_ref() {
        line.extend([
            Span::styled(
                log_search
                    .term
                    .as_ref()
                    .map_or(String::new(), std::clone::Clone::clone),
                Style::default()
                    .fg(colors.log_search.text)
                    .bg(colors.log_search.background),
            ),
            space(),
        ]);
    }

    let left_text = Paragraph::new(Line::from(line))
        .alignment(ratatui::layout::Alignment::Left)
        .style(Style::default().bg(colors.log_search.background));
    let mut line = vec![];
    if let Some(log_search) = fd.log_search.as_ref() {
        if let Some(buttons) = log_search.buttons.as_ref() {
            let down = if keymap.scroll_down.0 == KeyCode::Down {
                "↑".to_owned()
            } else {
                keymap.scroll_down.0.to_string()
            };
            let up = if keymap.scroll_up.0 == KeyCode::Up {
                "↓".to_owned()
            } else {
                keymap.scroll_up.0.to_string()
            };
            let next = [
                space(),
                Span::styled(format!(" {up} "), style_but),
                Span::styled(" next ", style_desc),
            ];
            let previous = [
                space(),
                Span::styled(format!(" {down} "), style_but),
                Span::styled(" previous ", style_desc),
            ];

            match buttons {
                LogsButton::Both => line.extend(previous.into_iter().chain(next)),
                LogsButton::Next => line.extend(next),
                LogsButton::Previous => line.extend(previous),
            }
        }

        if let Some(results) = log_search.result.as_ref() {
            line.extend([
                Span::styled(
                    " matches: ",
                    Style::default()
                        .fg(colors.log_search.highlight)
                        .bg(colors.log_search.background)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    results,
                    Style::default()
                        .fg(colors.log_search.text)
                        .bg(colors.log_search.background),
                ),
            ]);
        }
    }
    let right_text = Paragraph::new(Line::from(line))
        .alignment(ratatui::layout::Alignment::Right)
        .style(Style::default().bg(colors.log_search.background));

    let line_split = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(vec![Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);
    frame.render_widget(left_text, line_split[0]);
    frame.render_widget(right_text, line_split[1]);
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {

    use crossterm::event::KeyCode;
    use insta::assert_snapshot;
    use ratatui::style::{Color, Modifier};

    use crate::{
        config::{AppColors, Keymap},
        ui::{
            FrameData,
            draw_blocks::tests::{get_result, insert_logs, test_setup},
        },
    };

    #[test]
    /// Filter row is drawn correctly & colors are correct
    /// Colours change when filter_by option is changed
    fn test_draw_blocks_log_search_row() {
        let mut setup = test_setup(140, 1, true, true);

        setup
            .gui_state
            .lock()
            .status_push(crate::ui::Status::SearchLogs);
        setup
            .terminal
            .draw(|f| {
                super::draw(setup.area, AppColors::new(), f, &setup.fd, &Keymap::new());
            })
            .unwrap();

        assert_snapshot!(setup.terminal.backend());

        for (_, result_row) in get_result(&setup) {
            for (result_cell_index, result_cell) in result_row.iter().enumerate() {
                match result_cell_index {
                    0..=4 => {
                        assert_eq!(result_cell.bg, Color::Magenta);
                        assert_eq!(result_cell.fg, Color::Black);
                    }
                    5..=11 => {
                        assert_eq!(result_cell.bg, Color::Reset);
                        assert_eq!(result_cell.fg, Color::Gray);
                    }
                    13..=26 => {
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

    #[test]
    /// Log item found, previous button visible
    fn test_draw_blocks_log_search_match_previous() {
        let mut setup = test_setup(140, 1, true, true);

        insert_logs(&setup);
        setup
            .gui_state
            .lock()
            .status_push(crate::ui::Status::SearchLogs);

        setup.app_data.lock().log_search_push('e');

        let fd = FrameData::from((&setup.app_data, &setup.gui_state));

        setup
            .terminal
            .draw(|f| {
                super::draw(setup.area, AppColors::new(), f, &fd, &Keymap::new());
            })
            .unwrap();

        assert_snapshot!(setup.terminal.backend());

        for (_, result_row) in get_result(&setup) {
            for (result_cell_index, result_cell) in result_row.iter().enumerate() {
                match result_cell_index {
                    0..=4 | 114..=116 => {
                        assert_eq!(result_cell.bg, Color::Magenta);
                        assert_eq!(result_cell.fg, Color::Black);
                    }
                    5..=11 | 27 | 117..=126 | 137..=139 => {
                        assert_eq!(result_cell.bg, Color::Reset);
                        assert_eq!(result_cell.fg, Color::Gray);
                    }
                    13..=26 | 127..=136 => {
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

    #[test]
    /// Log item found, next button visible
    fn test_draw_blocks_log_search_match_next() {
        let mut setup = test_setup(140, 1, true, true);

        insert_logs(&setup);

        setup
            .gui_state
            .lock()
            .status_push(crate::ui::Status::SearchLogs);

        setup.app_data.lock().log_search_push('e');
        setup
            .app_data
            .lock()
            .log_scroll(&crate::app_data::ScrollDirection::Previous);
        setup
            .app_data
            .lock()
            .log_scroll(&crate::app_data::ScrollDirection::Previous);

        let fd = FrameData::from((&setup.app_data, &setup.gui_state));
        setup
            .terminal
            .draw(|f| {
                super::draw(setup.area, AppColors::new(), f, &fd, &Keymap::new());
            })
            .unwrap();

        assert_snapshot!(setup.terminal.backend());

        for (_, result_row) in get_result(&setup) {
            for (result_cell_index, result_cell) in result_row.iter().enumerate() {
                match result_cell_index {
                    0..=4 | 118..=120 => {
                        assert_eq!(result_cell.bg, Color::Magenta);
                        assert_eq!(result_cell.fg, Color::Black);
                    }
                    5..=11 | 27 | 121..=126 | 137..=139 => {
                        assert_eq!(result_cell.bg, Color::Reset);
                        assert_eq!(result_cell.fg, Color::Gray);
                    }
                    13..=26 | 127..=136 => {
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

    #[test]
    /// Log item found, next & previous button visible
    fn test_draw_blocks_log_search_match_both_next_previous() {
        let mut setup = test_setup(140, 1, true, true);

        insert_logs(&setup);

        setup
            .gui_state
            .lock()
            .status_push(crate::ui::Status::SearchLogs);

        setup.app_data.lock().log_search_push('e');
        setup
            .app_data
            .lock()
            .log_scroll(&crate::app_data::ScrollDirection::Previous);

        let fd = FrameData::from((&setup.app_data, &setup.gui_state));
        setup
            .terminal
            .draw(|f| {
                super::draw(setup.area, AppColors::new(), f, &fd, &Keymap::new());
            })
            .unwrap();

        assert_snapshot!(setup.terminal.backend());

        for (_, result_row) in get_result(&setup) {
            for (result_cell_index, result_cell) in result_row.iter().enumerate() {
                match result_cell_index {
                    0..=4 | 104..=106 | 118..=120 => {
                        assert_eq!(result_cell.bg, Color::Magenta);
                        assert_eq!(result_cell.fg, Color::Black);
                    }
                    5..=11 | 27 | 107..=116 | 121..=126 | 137..=139 => {
                        assert_eq!(result_cell.bg, Color::Reset);
                        assert_eq!(result_cell.fg, Color::Gray);
                    }
                    13..=26 | 127..=136 => {
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

    #[test]
    /// No log item found
    fn test_draw_blocks_log_search_match_none() {
        let mut setup = test_setup(140, 1, true, true);

        insert_logs(&setup);

        setup
            .gui_state
            .lock()
            .status_push(crate::ui::Status::SearchLogs);

        setup.app_data.lock().log_search_push('z');
        let fd = FrameData::from((&setup.app_data, &setup.gui_state));

        setup
            .terminal
            .draw(|f| {
                super::draw(setup.area, AppColors::new(), f, &fd, &Keymap::new());
            })
            .unwrap();

        assert_snapshot!(setup.terminal.backend());

        for (_, result_row) in get_result(&setup) {
            for (result_cell_index, result_cell) in result_row.iter().enumerate() {
                match result_cell_index {
                    0..=4 => {
                        assert_eq!(result_cell.bg, Color::Magenta);
                        assert_eq!(result_cell.fg, Color::Black);
                    }
                    5..=11 | 27 => {
                        assert_eq!(result_cell.bg, Color::Reset);
                        assert_eq!(result_cell.fg, Color::Gray);
                    }
                    13..=26 => {
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

    #[test]
    /// Custom keymap for scroll buttons
    fn test_draw_blocks_log_search_keymap() {
        let mut setup = test_setup(140, 1, true, true);

        insert_logs(&setup);

        let mut keymap = setup.app_data.lock().config.keymap.clone();
        keymap.scroll_up = (KeyCode::Char('a'), None);
        keymap.scroll_down = (KeyCode::Char('b'), None);

        setup
            .gui_state
            .lock()
            .status_push(crate::ui::Status::SearchLogs);

        setup.app_data.lock().log_search_push('e');
        setup
            .app_data
            .lock()
            .log_scroll(&crate::app_data::ScrollDirection::Previous);
        let fd = FrameData::from((&setup.app_data, &setup.gui_state));

        setup
            .terminal
            .draw(|f| {
                super::draw(setup.area, AppColors::new(), f, &fd, &keymap);
            })
            .unwrap();

        assert_snapshot!(setup.terminal.backend());
    }

    #[test]
    /// Custom colours applied
    fn test_draw_blocks_log_search_colors() {
        let mut setup = test_setup(140, 1, true, true);

        insert_logs(&setup);

        setup
            .gui_state
            .lock()
            .status_push(crate::ui::Status::SearchLogs);

        setup.app_data.lock().log_search_push('e');
        setup
            .app_data
            .lock()
            .log_scroll(&crate::app_data::ScrollDirection::Previous);

        let mut colors = AppColors::new();

        colors.log_search.background = Color::White;
        colors.log_search.highlight = Color::Blue;
        colors.log_search.button_text = Color::Yellow;
        colors.log_search.text = Color::Magenta;

        let fd = FrameData::from((&setup.app_data, &setup.gui_state));
        setup
            .terminal
            .draw(|f| {
                super::draw(setup.area, colors, f, &fd, &Keymap::new());
            })
            .unwrap();

        assert_snapshot!(setup.terminal.backend());

        for (_, result_row) in get_result(&setup) {
            for (result_cell_index, result_cell) in result_row.iter().enumerate() {
                match result_cell_index {
                    0..=4 | 104..=106 | 118..=120 => {
                        assert_eq!(result_cell.bg, Color::Blue);
                        assert_eq!(result_cell.fg, Color::Yellow);
                    }
                    5..=11 | 27 | 107..=116 | 121..=126 | 137..=139 => {
                        assert_eq!(result_cell.bg, Color::White);
                        assert_eq!(result_cell.fg, Color::Magenta);
                    }
                    13..=26 | 127..=136 => {
                        assert_eq!(result_cell.bg, Color::White);
                        assert_eq!(result_cell.fg, Color::Blue);
                        assert_eq!(result_cell.modifier, Modifier::BOLD);
                    }
                    _ => {
                        assert_eq!(result_cell.bg, Color::White);
                        assert_eq!(result_cell.fg, Color::Reset);
                    }
                }
            }
        }
    }
}
