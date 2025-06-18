use std::sync::Arc;

use super::RIGHT_ARROW;
use crate::{
    app_data::AppData,
    config::AppColors,
    ui::{FrameData, GuiState, SelectablePanel},
};
use parking_lot::Mutex;
use ratatui::{
    Frame,
    layout::{Alignment, Rect},
    style::{Modifier, Style, Stylize},
    text::{Line, Span},
    widgets::{List, ListItem, Paragraph},
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
    use insta::assert_snapshot;
    use ratatui::style::{Color, Modifier};

    use crate::{
        config::AppColors,
        tests::gen_container_summary,
        ui::{
            FrameData,
            draw_blocks::tests::{BORDER_CHARS, get_result, test_setup},
        },
    };

    // cusomt border colors
    #[test]
    /// Test that when DockerCommands are available, they are drawn correctly, dependant on container state
    /// In this case, no commands are drawn
    fn test_draw_blocks_commands_none() {
        let mut setup = test_setup(12, 6, false, false);

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

        assert_snapshot!(setup.terminal.backend());
    }

    #[test]
    /// Test that when DockerCommands are available, they are drawn correctly, dependant on container state
    /// In this test, container is running
    fn test_draw_blocks_commands_some() {
        let mut setup = test_setup(12, 6, true, true);

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

        assert_snapshot!(setup.terminal.backend());

        for (row_index, result_row) in get_result(&setup) {
            for (result_cell_index, result_cell) in result_row.iter().enumerate() {
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
    }

    #[test]
    /// Test that when DockerCommands are available, they are drawn correctly, dependant on container state
    /// In this test, container is paused
    fn test_draw_blocks_commands_some_paused() {
        let mut setup = test_setup(12, 6, true, true);

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

        // Set the container state to paused
        setup
            .app_data
            .lock()
            .update_containers(vec![gen_container_summary(1, "paused")]);
        setup.app_data.lock().docker_controls_next();

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

        assert_snapshot!(setup.terminal.backend());

        for (row_index, result_row) in get_result(&setup) {
            for (result_cell_index, result_cell) in result_row.iter().enumerate() {
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
        let mut setup = test_setup(12, 6, true, true);
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

        assert_snapshot!(setup.terminal.backend());
        for (_, result_row) in get_result(&setup) {
            for result_cell in result_row {
                if BORDER_CHARS.contains(&result_cell.symbol()) {
                    assert_eq!(result_cell.fg, Color::Gray);
                }
            }
        }

        // Control panel now selected, should have a blue border
        setup
            .gui_state
            .lock()
            .selectable_panel_next(&setup.app_data);
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

        for (row_index, result_row) in get_result(&setup) {
            for (result_cell_index, result_cell) in result_row.iter().enumerate() {
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
    /// Custom colors are rendered correctly
    fn test_draw_blocks_commands_custom_colors_running() {
        let mut setup = test_setup(12, 6, true, true);
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

        assert_snapshot!(setup.terminal.backend());
        for (row_index, result_row) in get_result(&setup) {
            for (result_cell_index, result_cell) in result_row.iter().enumerate() {
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
    }
    #[test]
    /// Custom colors are rendered correctly
    fn test_draw_blocks_commands_custom_colors_paused() {
        let mut setup = test_setup(12, 6, true, true);
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

        // Set the controls state
        setup
            .app_data
            .lock()
            .update_containers(vec![gen_container_summary(1, "paused")]);
        setup.app_data.lock().docker_controls_next();

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

        assert_snapshot!(setup.terminal.backend());

        for (row_index, result_row) in get_result(&setup) {
            for (result_cell_index, result_cell) in result_row.iter().enumerate() {
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
