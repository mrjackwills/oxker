use ratatui::{
    layout::Rect,
    style::{Modifier, Style, Stylize},
    text::{Line, Span},
    Frame,
};

use crate::{app_data::FilterBy, config::AppColors, ui::FrameData};

/// Create the filter_by by spans, coloured dependant on which one is selected
fn filter_by_spans(colors: AppColors, fd: &FrameData) -> [Span; 4] {
    let selected = Style::default()
        .bg(colors.filter.selected_filter_background)
        .fg(colors.filter.selected_filter_text);
    let not_selected = Style::default()
        .bg(colors.filter.background)
        .fg(colors.filter.text);

    let name = [" Name ", " Image ", " Status ", " All "];

    let mut filter_spans = [
        Span::styled(name[0], not_selected),
        Span::styled(name[1], not_selected),
        Span::styled(name[2], not_selected),
        Span::styled(name[3], not_selected),
    ];

    match fd.filter_by {
        FilterBy::Name => filter_spans[0] = Span::styled(name[0], selected),
        FilterBy::Image => filter_spans[1] = Span::styled(name[1], selected),
        FilterBy::Status => filter_spans[2] = Span::styled(name[2], selected),
        FilterBy::All => filter_spans[3] = Span::styled(name[3], selected),
    }
    filter_spans
}

/// Draw the filter bar
pub fn draw(area: Rect, colors: AppColors, frame: &mut Frame, fd: &FrameData) {
    let style_but = Style::default()
        .fg(colors.filter.selected_filter_text)
        .bg(colors.filter.highlight);
    let style_desc = Style::default()
        .fg(colors.filter.text)
        .bg(colors.filter.background);

    let mut line = vec![
        Span::styled(" Esc ", style_but),
        Span::styled(" clear ", style_desc),
        Span::styled(" ← by → ", style_but),
        Span::from(" "),
    ];
    line.extend_from_slice(&filter_by_spans(colors, fd));
    line.extend_from_slice(&[
        Span::styled(
            " term: ",
            Style::default()
                .fg(colors.filter.highlight)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            fd.filter_term
                .as_ref()
                .map_or(String::new(), std::clone::Clone::clone),
            Style::default().fg(colors.filter.text),
        ),
    ]);
    frame.render_widget(Line::from(line).bg(colors.filter.background), area);
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {

    use ratatui::style::{Color, Modifier};

    use crate::{
        config::AppColors,
        ui::{
            draw_blocks::tests::{expected_to_vec, get_result, test_setup},
            FrameData,
        },
    };

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
                super::draw(setup.area, AppColors::new(), f, &setup.fd);
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
                    5..=11 | 27..=46 => {
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
        let fd = FrameData::from((&setup.app_data, &setup.gui_state));

        setup
            .terminal
            .draw(|f| {
                super::draw(setup.area, AppColors::new(), f, &fd);
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
                    5..=11 | 27..=46 | 54..=55 => {
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

        // Test when filter_by changes
        setup.app_data.lock().filter_by_next();
        let fd = FrameData::from((&setup.app_data, &setup.gui_state));
        setup
            .terminal
            .draw(|f| {
                super::draw(setup.area, AppColors::new(), f, &fd);
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
                    5..=11 | 21..=26 | 34..=46 | 54..=55 => {
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

    #[test]
    /// Make sure custom colors are applied
    fn test_draw_blocks_filter_row_custom_colors() {
        let (w, h) = (140, 1);
        let mut setup = test_setup(w, h, true, true);

        setup
            .gui_state
            .lock()
            .status_push(crate::ui::Status::Filter);

        setup.app_data.lock().filter_term_push('c');
        setup.app_data.lock().filter_term_push('d');
        let fd = FrameData::from((&setup.app_data, &setup.gui_state));

        let mut colors = AppColors::new();
        colors.filter.background = Color::White;
        colors.filter.highlight = Color::Blue;
        colors.filter.selected_filter_background = Color::Red;
        colors.filter.selected_filter_text = Color::Yellow;
        colors.filter.text = Color::Magenta;

        setup
            .terminal
            .draw(|f| {
                super::draw(setup.area, colors, f, &fd);
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
                        assert_eq!(result_cell.bg, Color::Blue);
                        assert_eq!(result_cell.fg, Color::Yellow);
                    }
                    5..=11 | 27..=46 | 54..=55 => {
                        assert_eq!(result_cell.bg, Color::White);
                        assert_eq!(result_cell.fg, Color::Magenta);
                    }
                    21..=26 => {
                        assert_eq!(result_cell.bg, Color::Red);
                        assert_eq!(result_cell.fg, Color::Yellow);
                    }
                    47..=53 => {
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
