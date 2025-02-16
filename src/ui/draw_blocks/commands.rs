use std::sync::Arc;

use super::RIGHT_ARROW;
use crate::{
    app_data::AppData,
    config::AppColors,
    ui::{FrameData, GuiState, SelectablePanel},
};
use parking_lot::Mutex;
use ratatui::{
    layout::{Alignment, Rect},
    style::{Modifier, Style, Stylize},
    text::{Line, Span},
    widgets::{List, ListItem, Paragraph},
    Frame,
};

use super::generate_block;

/// Draw the command panel
pub fn draw(
    app_data: &Arc<Mutex<AppData>>,
    area: Rect,
    colors: AppColors,
    f: &mut Frame,
    fd: &FrameData,
    gui_state: &Arc<Mutex<GuiState>>,
) {
    let block = generate_block(area, colors, fd, gui_state, SelectablePanel::Commands)
        .bg(colors.commands.background);
    let items = app_data.lock().get_control_items().map_or(vec![], |i| {
        i.iter()
            .map(|c| {
                let lines = Line::from(vec![Span::styled(
                    c.to_string(),
                    Style::default().fg(c.get_color(colors)),
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

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use ratatui::style::{Color, Modifier};

    use crate::{
        config::AppColors,
        tests::gen_container_summary,
        ui::{
            draw_blocks::tests::{expected_to_vec, get_result, test_setup, BORDER_CHARS},
            FrameData,
        },
    };

    // cusomt border colors
    #[test]
    /// Test that when DockerCommands are available, they are drawn correctly, dependant on container state
    fn test_draw_blocks_commands_none() {
        let (w, h) = (12, 6);
        let mut setup = test_setup(w, h, false, false);

        let colors = setup.app_data.lock().config.app_colors;
        setup
            .terminal
            .draw(|f| {
                super::draw(
                    &setup.app_data,
                    setup.area,
                    colors,
                    f,
                    &setup.fd,
                    &setup.gui_state,
                );
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
    /// Test that when DockerCommands are available, they are drawn correctly, dependant on container state
    fn test_draw_blocks_commands_some() {
        let (w, h) = (12, 6);
        let mut setup = test_setup(w, h, true, true);

        let colors = setup.app_data.lock().config.app_colors;
        setup
            .terminal
            .draw(|f| {
                super::draw(
                    &setup.app_data,
                    setup.area,
                    colors,
                    f,
                    &setup.fd,
                    &setup.gui_state,
                );
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
                    // Borders & delete
                    (0 | 5, _) | (1..=4, 0 | 11) | (4, 3..=8) => {
                        assert_eq!(result_cell.fg, Color::Gray);
                    }
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
            .update_containers(vec![gen_container_summary(1, "paused")]);
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
                super::draw(
                    &setup.app_data,
                    setup.area,
                    colors,
                    f,
                    &setup.fd,
                    &setup.gui_state,
                );
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
                    (0 | 5, _) | (1..=4, 0 | 11) | (3, 3..=8) => {
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
        let colors = setup.app_data.lock().config.app_colors;
        // Unselected, has a grey border
        setup
            .terminal
            .draw(|f| {
                super::draw(
                    &setup.app_data,
                    setup.area,
                    colors,
                    f,
                    &setup.fd,
                    &setup.gui_state,
                );
            })
            .unwrap();

        for (row_index, result_row) in get_result(&setup, w) {
            let expected_row = expected_to_vec(&expected, row_index);
            for (result_cell_index, result_cell) in result_row.iter().enumerate() {
                assert_eq!(result_cell.symbol(), expected_row[result_cell_index]);
                if BORDER_CHARS.contains(&result_cell.symbol()) {
                    assert_eq!(result_cell.fg, Color::Gray);
                }
            }
        }

        // Control panel now selected, should have a blue border
        setup.gui_state.lock().next_panel();
        let fd = FrameData::from((&setup.app_data, &setup.gui_state));
        setup
            .terminal
            .draw(|f| {
                super::draw(
                    &setup.app_data,
                    setup.area,
                    colors,
                    f,
                    &fd,
                    &setup.gui_state,
                );
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

    #[test]
    /// Custom colors are rendered correctlty
    fn test_draw_blocks_commands_custom_colors() {
        let (w, h) = (12, 6);
        let mut setup = test_setup(w, h, true, true);
        let mut colors = AppColors::new();
        colors.commands.background = Color::White;
        colors.commands.pause = Color::Black;
        colors.commands.restart = Color::Green;
        colors.commands.stop = Color::Blue;
        colors.commands.delete = Color::Magenta;
        colors.commands.resume = Color::Yellow;
        colors.commands.start = Color::Cyan;

        setup
            .terminal
            .draw(|f| {
                super::draw(
                    &setup.app_data,
                    setup.area,
                    colors,
                    f,
                    &setup.fd,
                    &setup.gui_state,
                );
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
                assert_eq!(result_cell.bg, Color::White);
                match (row_index, result_cell_index) {
                    // pause
                    (1, 3..=7) => {
                        assert_eq!(result_cell.fg, Color::Black);
                    }
                    // restart
                    (2, 3..=9) => {
                        assert_eq!(result_cell.fg, Color::Green);
                    }
                    // stop
                    (3, 3..=6) => {
                        assert_eq!(result_cell.fg, Color::Blue);
                    }
                    // delete
                    (4, 3..=8) => {
                        assert_eq!(result_cell.fg, Color::Magenta);
                    }
                    _ => (),
                }
            }
        }
        // Change the controls state
        setup
            .app_data
            .lock()
            .update_containers(vec![gen_container_summary(1, "paused")]);
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
                super::draw(
                    &setup.app_data,
                    setup.area,
                    colors,
                    f,
                    &setup.fd,
                    &setup.gui_state,
                );
            })
            .unwrap();

        for (row_index, result_row) in get_result(&setup, w) {
            let expected_row = expected_to_vec(&expected, row_index);
            for (result_cell_index, result_cell) in result_row.iter().enumerate() {
                assert_eq!(result_cell.symbol(), expected_row[result_cell_index]);
                assert_eq!(result_cell.bg, Color::White);

                match (row_index, result_cell_index) {
                    // resume
                    (1, 3..=7) => {
                        assert_eq!(result_cell.fg, Color::Yellow);
                    }
                    // stop
                    (2, 3..=6) => {
                        assert_eq!(result_cell.fg, Color::Blue);
                    }
                    // delete
                    (3, 3..=8) => {
                        assert_eq!(result_cell.fg, Color::Magenta);
                    }
                    _ => (),
                }
            }
        }
    }
}
