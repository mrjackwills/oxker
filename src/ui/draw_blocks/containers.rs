use std::sync::Arc;

use super::MARGIN;
use parking_lot::Mutex;
use ratatui::{
    Frame,
    layout::{Alignment, Rect},
    style::{Modifier, Style, Stylize},
    text::{Line, Span},
    widgets::{List, ListItem, Paragraph},
};

use crate::{
    app_data::{AppData, ByteStats, Columns, ContainerItem, CpuStats},
    config::AppColors,
    ui::{FrameData, GuiState, SelectablePanel},
};

use super::{CIRCLE, generate_block};

/// Format the container data to display nicely on the screen
fn format_containers<'a>(colors: AppColors, i: &ContainerItem, widths: &Columns) -> Line<'a> {
    let state_style = Style::default().fg(i.state.get_color(colors));

    Line::from(vec![
        Span::styled(
            format!(
                "{:<width$}{MARGIN}",
                i.name.to_string(),
                width = widths.name.1.into()
            ),
            colors.containers.text,
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
                i.status.get(),
                width = &widths.status.1.into()
            ),
            state_style,
        ),
        Span::styled(
            format!(
                "{:>width$}{MARGIN}",
                i.cpu_stats.back().map_or_else(CpuStats::default, |f| *f),
                width = &widths.cpu.1.into()
            ),
            state_style,
        ),
        Span::styled(
            format!(
                "{:>width_current$} / {:>width_limit$}{MARGIN}",
                i.mem_stats.back().map_or_else(ByteStats::default, |f| *f),
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
            colors.containers.text,
        ),
        Span::styled(
            format!(
                "{:<width$}{MARGIN}",
                i.image.to_string(),
                width = widths.image.1.into()
            ),
            colors.containers.text,
        ),
        Span::styled(
            format!("{:>width$}{MARGIN}", i.rx, width = widths.net_rx.1.into()),
            Style::default().fg(colors.containers.text_rx),
        ),
        Span::styled(
            format!("{:>width$}{MARGIN}", i.tx, width = widths.net_tx.1.into()),
            Style::default().fg(colors.containers.text_tx),
        ),
    ])
}

/// Draw the containers panel
pub fn draw(
    app_data: &Arc<Mutex<AppData>>,
    area: Rect,
    colors: AppColors,
    f: &mut Frame,
    fd: &FrameData,
    gui_state: &Arc<Mutex<GuiState>>,
) {
    let block = generate_block(area, colors, fd, gui_state, SelectablePanel::Containers)
        .bg(colors.containers.background);

    let items = app_data
        .lock()
        .get_container_items()
        .iter()
        .map(|i| ListItem::new(format_containers(colors, i, &fd.columns)))
        .collect::<Vec<_>>();

    if items.is_empty() {
        let text = if fd.filter_term.is_some() {
            "no containers match filter"
        } else if fd.is_loading {
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

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use ratatui::style::{Color, Modifier};

    use crate::{
        app_data::{ContainerImage, ContainerName, ContainerStatus, State, StatefulList},
        config::AppColors,
        ui::{
            FrameData,
            draw_blocks::tests::{
                BORDER_CHARS, COLOR_ORANGE, COLOR_RX, COLOR_TX, TuiTestSetup, expected_to_vec,
                get_result, test_setup,
            },
        },
    };

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
        let fd = FrameData::from((&setup.app_data, &setup.gui_state));
        let colors = setup.app_data.lock().config.app_colors;

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
                if BORDER_CHARS.contains(&result_cell.symbol()) {
                    assert_eq!(result_cell.fg, Color::Gray);
                }
            }
        }

        setup.gui_state.lock().previous_panel();
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

                if BORDER_CHARS.contains(&result_cell.symbol()) {
                    assert_eq!(result_cell.fg, Color::Gray);
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
            "╰────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────╯",
        ];
        let fd = FrameData::from((&setup.app_data, &setup.gui_state));
        let colors = setup.app_data.lock().config.app_colors;

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
                        assert_eq!(result_cell.fg, COLOR_RX);
                    }
                    // tx column
                    (1..=3, 102..=111) => {
                        assert_eq!(result_cell.fg, COLOR_TX);
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
        let fd = FrameData::from((&setup.app_data, &setup.gui_state));
        let colors = setup.app_data.lock().config.app_colors;
        setup.app_data.lock().containers.items[0].state = State::Paused;

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
                        assert_eq!(result_cell.fg, COLOR_RX);
                    }
                    // tx column
                    (1..=3, 102..=111) => {
                        assert_eq!(result_cell.fg, COLOR_TX);
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
        let fd = FrameData::from((&setup.app_data, &setup.gui_state));
        let colors = setup.app_data.lock().config.app_colors;
        setup.app_data.lock().containers.items[0].state = State::Paused;

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
        let fd = FrameData::from((&setup.app_data, &setup.gui_state));
        let colors = setup.app_data.lock().config.app_colors;

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
        let fd = FrameData::from((&setup.app_data, &setup.gui_state));
        let colors = setup.app_data.lock().config.app_colors;

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
        let fd = FrameData::from((&setup.app_data, &setup.gui_state));
        let colors = setup.app_data.lock().config.app_colors;

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
        let fd = FrameData::from((&setup.app_data, &setup.gui_state));
        let colors = setup.app_data.lock().config.app_colors;

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
                        assert_eq!(result_cell.fg, COLOR_RX);
                    }
                    // tx column
                    (1..=3, 105..=114) => {
                        assert_eq!(result_cell.fg, COLOR_TX);
                    }
                    _ => {
                        assert_eq!(result_cell.fg, Color::Reset);
                    }
                }
            }
        }
    }

    #[test]
    /// When container state is unhealthy, correct colors displayed
    fn test_draw_blocks_containers_unhealthy() {
        let (w, h) = (130, 6);
        let mut setup = test_setup(w, h, true, true);

        let status = ContainerStatus::from("Up 1 hour (unhealthy)".to_owned());
        setup.app_data.lock().containers.items[0].state = State::from(("running", &status));
        setup.app_data.lock().containers.items[0].status = status;

        let expected = [
            "╭ Containers 1/3 ────────────────────────────────────────────────────────────────────────────────────────────────────────────────╮",
            "│⚪  container_1   ! running   Up 1 hour (unhealthy)   00.00%   0.00 kB / 0.00 kB          1   image_1   0.00 kB   0.00 kB        │",
            "│   container_2   ✓ running   Up 2 hour               00.00%   0.00 kB / 0.00 kB          2   image_2   0.00 kB   0.00 kB        │",
            "│   container_3   ✓ running   Up 3 hour               00.00%   0.00 kB / 0.00 kB          3   image_3   0.00 kB   0.00 kB        │",
            "│                                                                                                                                │",
            "╰────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────╯",
        ];
        let fd = FrameData::from((&setup.app_data, &setup.gui_state));
        let colors = setup.app_data.lock().config.app_colors;

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
                match (row_index, result_cell_index) {
                    // border
                    (0 | 5, _) | (1..=4, 0 | 129) => {
                        assert_eq!(result_cell.fg, Color::LightCyan);
                    }
                    // name, id, image column
                    (1..=3, 4..=17 | 83..=103) => {
                        assert_eq!(result_cell.fg, Color::Blue);
                    }
                    // state, status, cpu, memory column of the first row
                    (1, 18..=82) => {
                        assert_eq!(result_cell.fg, COLOR_ORANGE);
                    }
                    // state, status, cpu, memory column
                    (2..=3, 18..=82) => {
                        assert_eq!(result_cell.fg, Color::Green);
                    }
                    // rx column
                    (1..=3, 104..=113) => {
                        assert_eq!(result_cell.fg, COLOR_RX);
                    }
                    // tx column
                    (1..=3, 114..=123) => {
                        assert_eq!(result_cell.fg, COLOR_TX);
                    }
                    _ => assert_eq!(result_cell.fg, Color::Reset),
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
        let fd = FrameData::from((&setup.app_data, &setup.gui_state));
        let colors = setup.app_data.lock().config.app_colors;

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

        check_expected(expected, w, h, &setup, Color::Red);
    }

    #[test]
    /// Custom colors applied correctly
    fn test_draw_blocks_containers_custom_colors() {
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
        let fd = FrameData::from((&setup.app_data, &setup.gui_state));
        let mut colors = AppColors::new();
        colors.borders.selected = Color::Green;
        colors.containers.background = Color::Black;
        colors.containers.text = Color::Yellow;
        colors.containers.text_rx = Color::Red;
        colors.containers.text_tx = Color::Blue;

        colors.container_state.running_healthy = Color::Magenta;

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
                // The highlight symbol can't correctly be colored
                if (row_index, result_cell_index) != (1, 2) {
                    assert_eq!(result_cell.bg, Color::Black);
                }
                match (row_index, result_cell_index) {
                    //border
                    (0 | 5, _) | (1..=4, 0 | 129) => {
                        assert_eq!(result_cell.fg, Color::Green);
                    }
                    // name, id, image column
                    (1..=3, 4..=17 | 71..=91) => {
                        assert_eq!(result_cell.fg, Color::Yellow);
                    }
                    // state, status, cpu, memory column
                    (1..=3, 18..=70) => {
                        assert_eq!(result_cell.fg, Color::Magenta);
                    }
                    // rx column
                    (1..=3, 92..=101) => {
                        assert_eq!(result_cell.fg, Color::Red);
                    }
                    // tx column
                    (1..=3, 102..=111) => {
                        assert_eq!(result_cell.fg, Color::Blue);
                    }
                    _ => assert_eq!(result_cell.fg, Color::Reset),
                }
            }
        }
    }

    #[test]
    /// Make sure that the state has the correctly color applied to it
    fn test_draw_blocks_containers_custom_colors_state_healthy() {
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
        let fd = FrameData::from((&setup.app_data, &setup.gui_state));

        // Healthy
        let mut colors = AppColors::new();
        colors.container_state.running_healthy = Color::Magenta;

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
                if let (1..=3, 18..=70) = (row_index, result_cell_index) {
                    assert_eq!(result_cell.fg, Color::Magenta);
                }
            }
        }
    }
    #[test]
    /// Make sure that the state has the correctly color applied to it
    fn test_draw_blocks_containers_custom_colors_state_unhealthy() {
        let (w, h) = (130, 6);
        let mut setup = test_setup(w, h, true, true);

        // Unhealthy
        let expected = [
            "╭ Containers 1/3 ────────────────────────────────────────────────────────────────────────────────────────────────────────────────╮",
            "│⚪  container_1   ! running   Up 1 hour (unhealthy)   00.00%   0.00 kB / 0.00 kB          1   image_1   0.00 kB   0.00 kB        │",
            "│   container_2   ✓ running   Up 2 hour   00.00%   0.00 kB / 0.00 kB          2   image_2   0.00 kB   0.00 kB                    │",
            "│   container_3   ✓ running   Up 3 hour   00.00%   0.00 kB / 0.00 kB          3   image_3   0.00 kB   0.00 kB                    │",
            "│                                                                                                                                │",
            "╰────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────╯",
        ];
        let fd = FrameData::from((&setup.app_data, &setup.gui_state));

        let mut colors = AppColors::new();
        colors.container_state.running_unhealthy = Color::Red;
        let status = ContainerStatus::from("Up 1 hour (unhealthy)".to_owned());
        setup.app_data.lock().containers.items[0].state = State::from(("running", &status));
        setup.app_data.lock().containers.items[0].status = status;

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
                if let (1, 18..=70) = (row_index, result_cell_index) {
                    assert_eq!(result_cell.fg, Color::Red);
                }
            }
        }
    }

    #[test]
    /// Make sure that the state has the correctly color applied to it
    fn test_draw_blocks_containers_custom_colors_state_dead() {
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

        let fd = FrameData::from((&setup.app_data, &setup.gui_state));

        let mut colors = AppColors::new();
        colors.container_state.dead = Color::Magenta;
        setup.app_data.lock().containers.items[0].state = State::Dead;

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
                if let (1, 18..=70) = (row_index, result_cell_index) {
                    assert_eq!(result_cell.fg, Color::Magenta);
                }
            }
        }
    }

    #[test]
    /// Make sure that the state has the correctly color applied to it
    fn test_draw_blocks_containers_custom_colors_state_exited() {
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

        let fd = FrameData::from((&setup.app_data, &setup.gui_state));

        let mut colors = AppColors::new();
        colors.container_state.exited = Color::Gray;
        setup.app_data.lock().containers.items[0].state = State::Exited;

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
                if let (1, 18..=70) = (row_index, result_cell_index) {
                    assert_eq!(result_cell.fg, Color::Gray);
                }
            }
        }
    }

    #[test]
    /// Make sure that the state has the correctly color applied to it
    fn test_draw_blocks_containers_custom_colors_state_paused() {
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

        let fd = FrameData::from((&setup.app_data, &setup.gui_state));

        let mut colors = AppColors::new();
        colors.container_state.paused = Color::Cyan;
        setup.app_data.lock().containers.items[0].state = State::Paused;

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
                if let (1, 18..=70) = (row_index, result_cell_index) {
                    assert_eq!(result_cell.fg, Color::Cyan);
                }
            }
        }
    }

    #[test]
    /// Make sure that the state has the correctly color applied to it
    fn test_draw_blocks_containers_custom_colors_state_removing() {
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

        let fd = FrameData::from((&setup.app_data, &setup.gui_state));

        let mut colors = AppColors::new();
        colors.container_state.removing = Color::White;
        setup.app_data.lock().containers.items[0].state = State::Removing;

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
                if let (1, 18..=70) = (row_index, result_cell_index) {
                    assert_eq!(result_cell.fg, Color::White);
                }
            }
        }
    }

    #[test]
    /// Make sure that the state has the correctly color applied to it
    fn test_draw_blocks_containers_custom_colors_state_restarting() {
        let (w, h) = (130, 6);
        let mut setup = test_setup(w, h, true, true);
        let expected = [
            "╭ Containers 1/3 ────────────────────────────────────────────────────────────────────────────────────────────────────────────────╮",
            "│⚪  container_1   ↻ restarting   Up 1 hour   00.00%   0.00 kB / 0.00 kB          1   image_1   0.00 kB   0.00 kB                 │",
            "│   container_2   ✓ running   Up 2 hour   00.00%   0.00 kB / 0.00 kB          2   image_2   0.00 kB   0.00 kB                    │",
            "│   container_3   ✓ running   Up 3 hour   00.00%   0.00 kB / 0.00 kB          3   image_3   0.00 kB   0.00 kB                    │",
            "│                                                                                                                                │",
            "╰────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────╯",
        ];

        let fd = FrameData::from((&setup.app_data, &setup.gui_state));

        let mut colors = AppColors::new();
        colors.container_state.restarting = Color::LightYellow;
        setup.app_data.lock().containers.items[0].state = State::Restarting;

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
                if let (1, 18..=70) = (row_index, result_cell_index) {
                    assert_eq!(result_cell.fg, Color::LightYellow);
                }
            }
        }
    }

    #[test]
    /// Make sure that the state has the correctly color applied to it
    fn test_draw_blocks_containers_custom_colors_state_unknown() {
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

        let fd = FrameData::from((&setup.app_data, &setup.gui_state));

        let mut colors = AppColors::new();
        colors.container_state.unknown = COLOR_ORANGE;
        setup.app_data.lock().containers.items[0].state = State::Unknown;

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
                if let (1, 18..=70) = (row_index, result_cell_index) {
                    assert_eq!(result_cell.fg, COLOR_ORANGE);
                }
            }
        }
    }
}
