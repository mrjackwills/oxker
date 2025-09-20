use std::sync::Arc;

use parking_lot::Mutex;
use ratatui::{
    Frame,
    layout::{Alignment, Rect},
    style::{Modifier, Style, Stylize},
    widgets::{List, Paragraph},
};

use crate::{
    app_data::AppData,
    config::AppColors,
    ui::{FrameData, GuiState, SelectablePanel, Status},
};

use super::{RIGHT_ARROW, generate_block};

/// Draw the logs panel
pub fn draw(
    app_data: &Arc<Mutex<AppData>>,
    area: Rect,
    colors: AppColors,
    f: &mut Frame,
    fd: &FrameData,
    gui_state: &Arc<Mutex<GuiState>>,
) {
    let mut block = generate_block(area, colors, fd, gui_state, SelectablePanel::Logs);
    if !fd.color_logs {
        block = block.bg(colors.logs.background);
    }

    if fd.status.contains(&Status::Init) {
        let mut paragraph = Paragraph::new(format!("parsing logs {}", fd.loading_icon))
            .block(block)
            .alignment(Alignment::Center);
        if !fd.color_logs {
            paragraph = paragraph.fg(colors.logs.text);
        }
        f.render_widget(paragraph, area);
    } else {
        let padding = usize::from(area.height / 5);
        let logs = app_data.lock().get_logs(area.as_size(), padding);
        if logs.is_empty() {
            let mut paragraph = Paragraph::new("no logs found")
                .block(block)
                .alignment(Alignment::Center);
            if !fd.color_logs {
                paragraph = paragraph.fg(colors.logs.text);
            }
            f.render_widget(paragraph, area);
        } else if fd.color_logs {
            let items = List::new(logs)
                .block(block)
                .highlight_symbol(RIGHT_ARROW)
                .scroll_padding(padding)
                .highlight_style(Style::default().add_modifier(Modifier::BOLD));
            // This should always return Some, as logs is not empty
            if let Some(log_state) = app_data.lock().get_log_state() {
                f.render_stateful_widget(items, area, log_state);
            }
        } else {
            let items = List::new(logs)
                .fg(colors.logs.text)
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

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use insta::assert_snapshot;
    use ratatui::style::{Color, Modifier};
    use uuid::Uuid;

    use crate::{
        app_data::{ContainerImage, ContainerName, ScrollDirection},
        config::AppColors,
        ui::{
            FrameData, Status,
            draw_blocks::tests::{BORDER_CHARS, get_result, insert_logs, test_setup},
        },
    };

    #[test]
    /// No logs, panel unselected, then selected, border color changes correctly
    fn test_draw_blocks_logs_none() {
        let mut setup = test_setup(35, 6, true, true);

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
                match (row_index, result_cell_index) {
                    (0 | 5, 0..=34) | (1..=4, 0) | (1..=5, 34) => {
                        assert_eq!(result_cell.bg, Color::Reset);
                        assert_eq!(result_cell.fg, Color::Gray);
                    }
                    _ => {
                        assert_eq!(result_cell.fg, Color::Reset);
                        assert_eq!(result_cell.bg, Color::Reset);
                    }
                }
            }
        }

        setup
            .gui_state
            .lock()
            .selectable_panel_next(&setup.app_data);
        setup
            .gui_state
            .lock()
            .selectable_panel_next(&setup.app_data);
        let fd = FrameData::from((&setup.app_data, &setup.gui_state));

        // When selected, has a blue border
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

        for (_, result_row) in get_result(&setup) {
            for result_cell in result_row {
                if BORDER_CHARS.contains(&result_cell.symbol()) {
                    assert_eq!(result_cell.fg, Color::LightCyan);
                }
            }
        }
    }

    #[test]
    /// Parsing logs, first frame spinner visible
    fn test_draw_blocks_logs_parsing_frame_one() {
        let mut setup = test_setup(32, 6, true, true);
        let uuid = Uuid::new_v4();
        setup.gui_state.lock().next_loading(uuid);

        let mut fd = FrameData::from((&setup.app_data, &setup.gui_state));
        fd.status.insert(Status::Init);

        setup
            .terminal
            .draw(|f| {
                super::draw(
                    &setup.app_data,
                    setup.area,
                    AppColors::new(),
                    f,
                    &fd,
                    &setup.gui_state,
                );
            })
            .unwrap();

        assert_snapshot!(setup.terminal.backend());
        for (row_index, result_row) in get_result(&setup) {
            for (result_cell_index, result_cell) in result_row.iter().enumerate() {
                match (row_index, result_cell_index) {
                    (0, 0..=31) | (1..=4, 0) | (1..=5, 31) | (5, 0..=30) => {
                        assert_eq!(result_cell.bg, Color::Reset);
                        assert_eq!(result_cell.fg, Color::Gray);
                    }
                    _ => {
                        assert_eq!(result_cell.fg, Color::Reset);
                        assert_eq!(result_cell.bg, Color::Reset);
                    }
                }
            }
        }
    }
    #[test]
    /// Parsing logs, second frame spinner visible
    fn test_draw_blocks_logs_parsing_frame_two() {
        let mut setup = test_setup(32, 6, true, true);
        let uuid = Uuid::new_v4();
        setup.gui_state.lock().next_loading(uuid);

        let mut fd = FrameData::from((&setup.app_data, &setup.gui_state));
        fd.status.insert(Status::Init);

        // animation moved by one frame
        setup.gui_state.lock().next_loading(uuid);

        let mut fd = FrameData::from((&setup.app_data, &setup.gui_state));
        fd.status.insert(Status::Init);
        setup
            .terminal
            .draw(|f| {
                super::draw(
                    &setup.app_data,
                    setup.area,
                    AppColors::new(),
                    f,
                    &fd,
                    &setup.gui_state,
                );
            })
            .unwrap();

        assert_snapshot!(setup.terminal.backend());

        for (row_index, result_row) in get_result(&setup) {
            for (result_cell_index, result_cell) in result_row.iter().enumerate() {
                match (row_index, result_cell_index) {
                    (0, 0..=31) | (1..=4, 0) | (1..=5, 31) | (5, 0..=30) => {
                        assert_eq!(result_cell.bg, Color::Reset);
                        assert_eq!(result_cell.fg, Color::Gray);
                    }
                    _ => {
                        assert_eq!(result_cell.fg, Color::Reset);
                        assert_eq!(result_cell.bg, Color::Reset);
                    }
                }
            }
        }
    }

    #[test]
    /// Logs correct displayed, changing log state also draws correctly
    fn test_draw_blocks_logs_some_line_three() {
        let mut setup = test_setup(36, 6, true, true);

        insert_logs(&setup);

        let fd = FrameData::from((&setup.app_data, &setup.gui_state));
        setup
            .terminal
            .draw(|f| {
                super::draw(
                    &setup.app_data,
                    setup.area,
                    AppColors::new(),
                    f,
                    &fd,
                    &setup.gui_state,
                );
            })
            .unwrap();

        assert_snapshot!(setup.terminal.backend());
        for (row_index, result_row) in get_result(&setup) {
            // let expected_row = expected_to_vec(&expected, row_index);
            for (result_cell_index, result_cell) in result_row.iter().enumerate() {
                // assert_eq!(result_cell.symbol(), expected_row[result_cell_index]);
                assert_eq!(result_cell.bg, Color::Reset);
                if let (1..=4, 1..=34) = (row_index, result_cell_index) {
                    assert_eq!(result_cell.fg, Color::Reset);
                } else {
                    assert_eq!(result_cell.fg, Color::Gray);
                }
                if row_index == 3 && (1..=34).contains(&result_cell_index) {
                    assert_eq!(result_cell.modifier, Modifier::BOLD);
                } else {
                    assert!(result_cell.modifier.is_empty());
                }
            }
        }
    }
    #[test]
    /// Logs correct displayed, changing log state also draws correctly
    fn test_draw_blocks_logs_some_line_two() {
        let mut setup = test_setup(36, 6, true, true);

        insert_logs(&setup);

        let fd = FrameData::from((&setup.app_data, &setup.gui_state));
        setup
            .terminal
            .draw(|f| {
                super::draw(
                    &setup.app_data,
                    setup.area,
                    AppColors::new(),
                    f,
                    &fd,
                    &setup.gui_state,
                );
            })
            .unwrap();
        setup.app_data.lock().log_scroll(&ScrollDirection::Previous);
        let fd = FrameData::from((&setup.app_data, &setup.gui_state));

        setup
            .terminal
            .draw(|f| {
                super::draw(
                    &setup.app_data,
                    setup.area,
                    AppColors::new(),
                    f,
                    &fd,
                    &setup.gui_state,
                );
            })
            .unwrap();

        assert_snapshot!(setup.terminal.backend());

        for (row_index, result_row) in get_result(&setup) {
            for (result_cell_index, result_cell) in result_row.iter().enumerate() {
                assert_eq!(result_cell.bg, Color::Reset);
                if let (1..=4, 1..=34) = (row_index, result_cell_index) {
                    assert_eq!(result_cell.fg, Color::Reset);
                } else {
                    assert_eq!(result_cell.fg, Color::Gray);
                }
                if row_index == 2 && (1..=34).contains(&result_cell_index) {
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
        let mut setup = test_setup(80, 6, true, true);
        setup.app_data.lock().containers.items[0].name =
            ContainerName::from("a_long_container_name_for_the_purposes_of_this_test");
        setup.app_data.lock().containers.items[0].image =
            ContainerImage::from("a_long_image_name_for_the_purposes_of_this_test");
        insert_logs(&setup);

        let fd = FrameData::from((&setup.app_data, &setup.gui_state));
        setup
            .terminal
            .draw(|f| {
                super::draw(
                    &setup.app_data,
                    setup.area,
                    AppColors::new(),
                    f,
                    &fd,
                    &setup.gui_state,
                );
            })
            .unwrap();

        assert_snapshot!(setup.terminal.backend());
    }

    #[test]
    fn test_draw_blocks_logs_custom_colors_parsing() {
        let mut setup = test_setup(32, 6, true, true);
        let uuid = Uuid::new_v4();
        setup.gui_state.lock().next_loading(uuid);

        let mut fd = FrameData::from((&setup.app_data, &setup.gui_state));
        fd.status.insert(Status::Init);

        let mut colors = AppColors::new();
        colors.logs.background = Color::Green;
        colors.logs.text = Color::Black;

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

        assert_snapshot!(setup.terminal.backend());

        for (row_index, result_row) in get_result(&setup) {
            for (result_cell_index, result_cell) in result_row.iter().enumerate() {
                assert_eq!(result_cell.bg, Color::Green);
                if let (1..=4, 1..=29) = (row_index, result_cell_index) {
                    assert_eq!(result_cell.fg, Color::Black);
                }
            }
        }

        fd.color_logs = true;

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
                assert_eq!(result_cell.bg, Color::Reset);
                if let (1..=4, 1..=29) = (row_index, result_cell_index) {
                    assert_eq!(result_cell.fg, Color::Reset);
                }
            }
        }
    }

    #[test]

    fn test_draw_blocks_logs_custom_colors_no_logs() {
        let mut setup = test_setup(35, 6, true, true);

        let mut colors = AppColors::new();
        colors.logs.background = Color::Green;
        colors.logs.text = Color::Black;

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
                assert_eq!(result_cell.bg, Color::Green);
                if let (1..=4, 1..=29) = (row_index, result_cell_index) {
                    assert_eq!(result_cell.fg, Color::Black);
                }
            }
        }

        setup.fd.color_logs = true;
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

        for (row_index, result_row) in get_result(&setup) {
            for (result_cell_index, result_cell) in result_row.iter().enumerate() {
                assert_eq!(result_cell.bg, Color::Reset);
                if let (1..=4, 1..=29) = (row_index, result_cell_index) {
                    assert_eq!(result_cell.fg, Color::Reset);
                }
            }
        }
    }

    #[test]
    /// Logs correct displayed with custom colors
    fn test_draw_blocks_logs_custom_colors_logs() {
        let mut setup = test_setup(36, 6, true, true);
        insert_logs(&setup);

        let mut colors = setup.app_data.lock().config.app_colors;
        colors.logs.background = Color::Green;
        colors.logs.text = Color::Black;
        let mut fd = FrameData::from((&setup.app_data, &setup.gui_state));
        fd.color_logs = true;

        // Standard colors when color_logs is true
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

        assert_snapshot!(setup.terminal.backend());
        for (row_index, result_row) in get_result(&setup) {
            for (result_cell_index, result_cell) in result_row.iter().enumerate() {
                assert_eq!(result_cell.bg, Color::Reset);
                if let (1..=4, 1..=34) = (row_index, result_cell_index) {
                    assert_eq!(result_cell.fg, Color::Reset);
                    if row_index == 3 && (1..=34).contains(&result_cell_index) {
                        assert_eq!(result_cell.modifier, Modifier::BOLD);
                    } else {
                        assert!(result_cell.modifier.is_empty());
                    }
                }
            }
        }

        fd.color_logs = false;

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
                assert_eq!(result_cell.bg, Color::Green);
                if let (1..=4, 1..=34) = (row_index, result_cell_index) {
                    assert_eq!(result_cell.fg, Color::Black);
                    if row_index == 3 && (1..=34).contains(&result_cell_index) {
                        assert_eq!(result_cell.modifier, Modifier::BOLD);
                    } else {
                        assert!(result_cell.modifier.is_empty());
                    }
                }
            }
        }
    }
}
