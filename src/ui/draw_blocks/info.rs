use std::{sync::Arc, time::Instant};

use parking_lot::Mutex;
use ratatui::{
    Frame,
    layout::Alignment,
    style::Style,
    widgets::{Block, Borders, Clear, Paragraph},
};

use crate::{
    config::AppColors,
    ui::{GuiState, gui_state::BoxLocation},
};

use super::{max_line_width, popup};

/// Draw info box in one of the 9 BoxLocations
// TODO is this broken - I don't think so
pub fn draw(
    colors: AppColors,
    f: &mut Frame,
    gui_state: &Arc<Mutex<GuiState>>,
    instant: &Instant,
    msg: String,
) {
    let block = Block::default()
        .title("")
        .title_alignment(Alignment::Center)
        .style(
            Style::default()
                .bg(colors.popup_info.background)
                .fg(colors.popup_info.text),
        )
        .borders(Borders::NONE);

    let max_line_width = max_line_width(&msg) + 8;
    let lines = msg.lines().count() + 2;

    let paragraph = Paragraph::new(msg)
        .block(block)
        .style(
            Style::default()
                .bg(colors.popup_info.background)
                .fg(colors.popup_info.text),
        )
        .alignment(Alignment::Center);

    let area = popup::draw(lines, max_line_width, f.area(), BoxLocation::BottomRight);
    f.render_widget(Clear, area);
    f.render_widget(paragraph, area);
    if instant.elapsed().as_millis() > 4000 {
        gui_state.lock().reset_info_box();
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use ratatui::style::Color;

    use crate::{
        config::AppColors,
        ui::draw_blocks::tests::{expected_to_vec, get_result, test_setup},
    };

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
        let colors = setup.app_data.lock().config.app_colors;

        setup
            .terminal
            .draw(|f| {
                super::draw(
                    colors,
                    f,
                    &setup.gui_state,
                    &std::time::Instant::now(),
                    "test".to_owned(),
                );
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

    #[test]
    /// Info box drawn in bottom right with custom colors applied
    fn test_draw_blocks_info_custom_color() {
        let (w, h) = (45, 9);
        let mut setup = test_setup(w, h, true, true);

        let mut colors = AppColors::new();
        colors.popup_info.background = Color::Red;
        colors.popup_info.text = Color::Black;
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
                super::draw(
                    colors,
                    f,
                    &setup.gui_state,
                    &std::time::Instant::now(),
                    "test".to_owned(),
                );
            })
            .unwrap();

        for (row_index, result_row) in get_result(&setup, w) {
            let expected_row = expected_to_vec(&expected, row_index);
            for (result_cell_index, result_cell) in result_row.iter().enumerate() {
                assert_eq!(result_cell.symbol(), expected_row[result_cell_index]);

                let (bg, fg) = match (row_index, result_cell_index) {
                    (6..=8, 32..=44) => (Color::Red, Color::Black),
                    _ => (Color::Reset, Color::Reset),
                };
                assert_eq!(result_cell.bg, bg);
                assert_eq!(result_cell.fg, fg);
            }
        }
    }
}
