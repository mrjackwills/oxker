use std::fmt::Display;

use ratatui::{
    Frame,
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style, Stylize},
    symbols::{self, Marker},
    text::{Line, Span},
    widgets::{Axis, Block, BorderType, Borders, Chart, Dataset, GraphType},
};

use super::FrameData;
use crate::{
    app_data::{State, Stats},
    config::AppColors,
};

fn make_chart<'a, T: Stats + Display>(
    state: State,
    colors: AppColors,
    dataset: Vec<Dataset<'a>>,
    current_rx: &'a T,
    max_rx: &'a T,
    current_tx: &'a T,
    max_tx: &'a T,
) -> Chart<'a> {
    let gen_color = |state: &State, default: Color| {
        if state.is_healthy() {
            default
        } else {
            state.get_color(colors)
        }
    };

    let mut labels = [
        Span::raw(""),
        Span::styled(
            format!("{max_rx}"),
            Style::default()
                .add_modifier(Modifier::BOLD)
                .fg(gen_color(&state, colors.chart_bandwidth.max_rx)),
        ),
        Span::styled(
            format!("{max_tx}"),
            Style::default()
                .add_modifier(Modifier::BOLD)
                .fg(gen_color(&state, colors.chart_bandwidth.max_tx)),
        ),
        Span::raw(""),
    ];

    // Set the order of rx/tx on the y axis, based on which is the highest value
    if max_rx.get_value() > max_tx.get_value() {
        labels.reverse();
    }

    Chart::new(dataset)
        .bg(colors.chart_bandwidth.background)
        .block(
            Block::default()
                .title_alignment(Alignment::Center)
                .title(Line::from(vec![
                    Span::styled(
                        format!(" rx: {current_rx}"),
                        Style::default()
                            .add_modifier(Modifier::BOLD)
                            .fg(gen_color(&state, colors.chart_bandwidth.title_rx)),
                    ),
                    Span::styled(
                        format!(" tx: {current_tx} "),
                        Style::default()
                            .add_modifier(Modifier::BOLD)
                            .fg(gen_color(&state, colors.chart_bandwidth.title_tx)),
                    ),
                ]))
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(colors.chart_bandwidth.border)),
        )
        .x_axis(Axis::default().bounds([0.0, 60.0]))
        .y_axis(
            Axis::default()
                .labels(labels)
                .style(Style::default().fg(colors.chart_bandwidth.y_axis))
                .bounds([0.0, (max_rx.get_value()).max(max_tx.get_value()) + 0.01]),
        )
}

/// Draw bandwidth chart
pub fn draw(area: Rect, colors: AppColors, f: &mut Frame, fd: &FrameData) {
    if let Some(x) = fd.chart_data.as_ref() {
        let mut dataset = vec![
            Dataset::default()
                .marker(symbols::Marker::Dot)
                .style(Style::default().fg(colors.chart_bandwidth.points_tx))
                .graph_type(GraphType::Line)
                .marker(Marker::Dot)
                .style(Style::default().fg(colors.chart_bandwidth.points_tx))
                .data(&x.tx.dataset),
        ];
        dataset.extend(vec![
            Dataset::default()
                .marker(symbols::Marker::Dot)
                .style(Style::default().fg(colors.chart_bandwidth.points_rx))
                .marker(Marker::Dot)
                .style(Style::default().fg(colors.chart_bandwidth.points_rx))
                .graph_type(GraphType::Line)
                .data(&x.rx.dataset),
        ]);

        let chart = make_chart(
            x.state,
            colors,
            dataset,
            &x.rx.current,
            &x.rx.max,
            &x.tx.current,
            &x.tx.max,
        );

        f.render_widget(chart, area);
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use insta::assert_snapshot;
    use ratatui::style::Color;

    use crate::{
        app_data::{ContainerId, NetworkBandwidth, State},
        config::AppColors,
        ui::{
            FrameData,
            draw_blocks::tests::{COLOR_RX, COLOR_TX, get_result, test_setup},
        },
    };

    const TX_DOTS: [(usize, usize); 14] = [
        (1, 21),
        (2, 19),
        (2, 20),
        (3, 18),
        (3, 19),
        (4, 10),
        (4, 11),
        (4, 17),
        (4, 18),
        (5, 16),
        (6, 14),
        (6, 15),
        (7, 13),
        (7, 14),
    ];

    const RX_DOTS: [(usize, usize); 15] = [
        (1, 21),
        (2, 19),
        (2, 20),
        (3, 18),
        (3, 19),
        (4, 10),
        (4, 11),
        (4, 17),
        (4, 18),
        (5, 16),
        (6, 16),
        (6, 15),
        (7, 13),
        (7, 14),
        (8, 13),
    ];

    const COMBINED_DOTS_RX: [(usize, usize); 15] = [
        (1, 21),
        (2, 19),
        (2, 20),
        (3, 18),
        (3, 19),
        (4, 10),
        (4, 11),
        (4, 17),
        (4, 18),
        (5, 16),
        (6, 15),
        (6, 16),
        (7, 13),
        (7, 14),
        (8, 13),
    ];

    const COMBINED_DOTS_TX: [(usize, usize); 8] = [
        (7, 19),
        (7, 20),
        (7, 21),
        (8, 14),
        (8, 15),
        (8, 16),
        (8, 17),
        (8, 18),
    ];

    #[test]
    /// When status is Running, but not data, charts drawn without dots etc, colours correct
    fn test_draw_blocks_charts_running_none() {
        let mut setup = test_setup(40, 10, true, true);

        let fd = FrameData::from((&setup.app_data, &setup.gui_state));
        setup
            .terminal
            .draw(|f| {
                super::draw(setup.area, setup.app_data.lock().config.app_colors, f, &fd);
            })
            .unwrap();
        assert_snapshot!(setup.terminal.backend());
        for (row_index, result_row) in get_result(&setup) {
            for (result_cell_index, result_cell) in result_row.iter().enumerate() {
                match (row_index, result_cell_index) {
                    // border
                    (9, _) | (1..=9, 0 | 39) => {
                        assert_eq!(result_cell.bg, Color::Reset);
                        assert_eq!(result_cell.fg, Color::White);
                    }
                    // Border first row only
                    (0, 0..=4 | 34..=39) => {
                        assert_eq!(result_cell.bg, Color::Reset);
                        assert_eq!(result_cell.fg, Color::White);
                    }
                    // Title RX
                    (0, 5..=18) => {
                        assert_eq!(result_cell.bg, Color::Reset);
                        assert_eq!(result_cell.fg, COLOR_RX);
                    }
                    // Title TX
                    (0, 19..=33) => {
                        assert_eq!(result_cell.bg, Color::Reset);
                        assert_eq!(result_cell.fg, COLOR_TX);
                    }
                    // Y axis
                    (1..=8, 10) => {
                        assert_eq!(result_cell.bg, Color::Reset);
                        assert_eq!(result_cell.fg, Color::White);
                    }
                    // TX max
                    (4, 1..=9) => {
                        assert_eq!(result_cell.bg, Color::Reset);
                        assert_eq!(result_cell.fg, COLOR_TX);
                    }
                    // RX max
                    (6, 1..=9) => {
                        assert_eq!(result_cell.bg, Color::Reset);
                        assert_eq!(result_cell.fg, COLOR_RX);
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
    /// Test with TX data
    fn test_draw_blocks_charts_running_with_data_tx() {
        let mut setup = test_setup(40, 10, true, true);
        let mut tx = NetworkBandwidth::new();

        for i in 0..=20 {
            tx.push(1000 * i * (10 + 5 * i));
        }

        if let Some(item) = setup
            .app_data
            .lock()
            .get_container_by_id(&ContainerId::from("1"))
        {
            item.tx = tx;
        }

        let fd = FrameData::from((&setup.app_data, &setup.gui_state));

        setup
            .terminal
            .draw(|f| {
                super::draw(setup.area, setup.app_data.lock().config.app_colors, f, &fd);
            })
            .unwrap();
        assert_snapshot!(setup.terminal.backend());
        for (row_index, result_row) in get_result(&setup) {
            for (result_cell_index, result_cell) in result_row.iter().enumerate() {
                match (row_index, result_cell_index) {
                    // border
                    (9, _) | (1..=9, 0 | 39) => {
                        assert_eq!(result_cell.bg, Color::Reset);
                        assert_eq!(result_cell.fg, Color::White);
                    }
                    // Border first row only
                    (0, 0..=3 | 35..=39) => {
                        assert_eq!(result_cell.bg, Color::Reset);
                        assert_eq!(result_cell.fg, Color::White);
                    }
                    // Title RX
                    (0, 4..=17) => {
                        assert_eq!(result_cell.bg, Color::Reset);
                        assert_eq!(result_cell.fg, COLOR_RX);
                    }
                    // Title TX
                    (0, 18..=34) => {
                        assert_eq!(result_cell.bg, Color::Reset);
                        assert_eq!(result_cell.fg, COLOR_TX);
                    }
                    // Y axis
                    (1..=8, 12) => {
                        assert_eq!(result_cell.bg, Color::Reset);
                        assert_eq!(result_cell.fg, Color::White);
                    }
                    // TX max
                    (4, 1..=9) => {
                        assert_eq!(result_cell.bg, Color::Reset);
                        assert_eq!(result_cell.fg, COLOR_TX);
                    }
                    // RX max
                    (6, 1..=9) => {
                        assert_eq!(result_cell.bg, Color::Reset);
                        assert_eq!(result_cell.fg, COLOR_RX);
                    }
                    // TX dots
                    _x if TX_DOTS.contains(&(row_index, result_cell_index)) => {
                        assert_eq!(result_cell.bg, Color::Reset);
                        assert_eq!(result_cell.fg, COLOR_TX);
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
    /// Test with RX data
    fn test_draw_blocks_charts_running_with_data_rx() {
        let mut setup = test_setup(40, 10, true, true);
        let mut rx = NetworkBandwidth::new();

        for i in 0..=20 {
            rx.push(2000 * i * (10 + 7 * i));
        }

        if let Some(item) = setup
            .app_data
            .lock()
            .get_container_by_id(&ContainerId::from("1"))
        {
            item.rx = rx;
        }

        let fd = FrameData::from((&setup.app_data, &setup.gui_state));

        setup
            .terminal
            .draw(|f| {
                super::draw(setup.area, setup.app_data.lock().config.app_colors, f, &fd);
            })
            .unwrap();
        assert_snapshot!(setup.terminal.backend());
        for (row_index, result_row) in get_result(&setup) {
            for (result_cell_index, result_cell) in result_row.iter().enumerate() {
                match (row_index, result_cell_index) {
                    // border
                    (9, _) | (1..=9, 0 | 39) => {
                        assert_eq!(result_cell.bg, Color::Reset);
                        assert_eq!(result_cell.fg, Color::White);
                    }
                    // Border first row only
                    (0, 0..=3 | 35..=39) => {
                        assert_eq!(result_cell.bg, Color::Reset);
                        assert_eq!(result_cell.fg, Color::White);
                    }
                    // Title RX
                    (0, 4..=19) => {
                        assert_eq!(result_cell.bg, Color::Reset);
                        assert_eq!(result_cell.fg, COLOR_RX);
                    }
                    // Title TX
                    (0, 20..=34) => {
                        assert_eq!(result_cell.bg, Color::Reset);
                        assert_eq!(result_cell.fg, COLOR_TX);
                    }
                    // Y axis
                    (1..=8, 12) => {
                        assert_eq!(result_cell.bg, Color::Reset);
                        assert_eq!(result_cell.fg, Color::White);
                    }
                    // RX max
                    (4, 1..=9) => {
                        assert_eq!(result_cell.bg, Color::Reset);
                        assert_eq!(result_cell.fg, COLOR_RX);
                    }
                    // TX max
                    (6, 1..=9) => {
                        assert_eq!(result_cell.bg, Color::Reset);
                        assert_eq!(result_cell.fg, COLOR_TX);
                    }
                    // RX dots
                    _x if RX_DOTS.contains(&(row_index, result_cell_index)) => {
                        assert_eq!(result_cell.bg, Color::Reset);
                        assert_eq!(result_cell.fg, COLOR_RX);
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
    /// Test with RX & TX data
    fn test_draw_blocks_charts_running_with_data_tx_and_rx() {
        let mut setup = test_setup(40, 10, true, true);
        let mut rx = NetworkBandwidth::new();
        let mut tx = NetworkBandwidth::new();
        for i in 0..=20 {
            rx.push(2000 * i * (10 + 7 * i));
            tx.push(200 * i * (10 + 7 * i));
        }

        if let Some(item) = setup
            .app_data
            .lock()
            .get_container_by_id(&ContainerId::from("1"))
        {
            item.rx = rx;
            item.tx = tx;
        }

        let fd = FrameData::from((&setup.app_data, &setup.gui_state));

        setup
            .terminal
            .draw(|f| {
                super::draw(setup.area, setup.app_data.lock().config.app_colors, f, &fd);
            })
            .unwrap();
        assert_snapshot!(setup.terminal.backend());
        for (row_index, result_row) in get_result(&setup) {
            for (result_cell_index, result_cell) in result_row.iter().enumerate() {
                match (row_index, result_cell_index) {
                    // border
                    (9, _) | (1..=9, 0 | 39) => {
                        assert_eq!(result_cell.bg, Color::Reset);
                        assert_eq!(result_cell.fg, Color::White);
                    }
                    // Border first row only
                    (0, 0..=3 | 36..=39) => {
                        assert_eq!(result_cell.bg, Color::Reset);
                        assert_eq!(result_cell.fg, Color::White);
                    }
                    // Title RX
                    (0, 4..=19) => {
                        assert_eq!(result_cell.bg, Color::Reset);
                        assert_eq!(result_cell.fg, COLOR_RX);
                    }
                    // Title TX
                    (0, 20..=35) => {
                        assert_eq!(result_cell.bg, Color::Reset);
                        assert_eq!(result_cell.fg, COLOR_TX);
                    }
                    // Y axis
                    (1..=8, 12) => {
                        assert_eq!(result_cell.bg, Color::Reset);
                        assert_eq!(result_cell.fg, Color::White);
                    }
                    // RX max
                    (4, 1..=9) => {
                        assert_eq!(result_cell.bg, Color::Reset);
                        assert_eq!(result_cell.fg, COLOR_RX);
                    }
                    // TX max
                    (6, 1..=10) => {
                        assert_eq!(result_cell.bg, Color::Reset);
                        assert_eq!(result_cell.fg, COLOR_TX);
                    }
                    // TX dots
                    _x if COMBINED_DOTS_TX.contains(&(row_index, result_cell_index)) => {
                        assert_eq!(result_cell.bg, Color::Reset);
                        assert_eq!(result_cell.fg, COLOR_TX);
                    }
                    // RX dots
                    _x if COMBINED_DOTS_RX.contains(&(row_index, result_cell_index)) => {
                        assert_eq!(result_cell.bg, Color::Reset);
                        assert_eq!(result_cell.fg, COLOR_RX);
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
    /// Whens status paused, some text is now Yellow
    fn test_draw_blocks_charts_paused() {
        let mut setup = test_setup(40, 10, true, true);
        setup.app_data.lock().containers.items[0].state = State::Paused;

        let fd = FrameData::from((&setup.app_data, &setup.gui_state));
        setup
            .terminal
            .draw(|f| {
                super::draw(setup.area, setup.app_data.lock().config.app_colors, f, &fd);
            })
            .unwrap();
        assert_snapshot!(setup.terminal.backend());
        for (row_index, result_row) in get_result(&setup) {
            for (result_cell_index, result_cell) in result_row.iter().enumerate() {
                match (row_index, result_cell_index) {
                    // border
                    (9, _) | (1..=9, 0 | 39) => {
                        assert_eq!(result_cell.bg, Color::Reset);
                        assert_eq!(result_cell.fg, Color::White);
                    }
                    // Border first row only
                    (0, 0..=4 | 34..=39) => {
                        assert_eq!(result_cell.bg, Color::Reset);
                        assert_eq!(result_cell.fg, Color::White);
                    }
                    // Title & y-axis max
                    (0, 5..=33) | (4 | 6, 1..=9) => {
                        assert_eq!(result_cell.bg, Color::Reset);
                        assert_eq!(result_cell.fg, Color::Yellow);
                    }
                    // Y axis
                    (1..=8, 10) => {
                        assert_eq!(result_cell.bg, Color::Reset);
                        assert_eq!(result_cell.fg, Color::White);
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
    /// Whens status dead, some text is now red
    fn test_draw_blocks_charts_dead() {
        let mut setup = test_setup(40, 10, true, true);
        setup.app_data.lock().containers.items[0].state = State::Dead;

        let fd = FrameData::from((&setup.app_data, &setup.gui_state));
        setup
            .terminal
            .draw(|f| {
                super::draw(setup.area, setup.app_data.lock().config.app_colors, f, &fd);
            })
            .unwrap();
        assert_snapshot!(setup.terminal.backend());
        for (row_index, result_row) in get_result(&setup) {
            for (result_cell_index, result_cell) in result_row.iter().enumerate() {
                match (row_index, result_cell_index) {
                    // border
                    (9, _) | (1..=9, 0 | 39) => {
                        assert_eq!(result_cell.bg, Color::Reset);
                        assert_eq!(result_cell.fg, Color::White);
                    }
                    // Border first row only
                    (0, 0..=4 | 34..=39) => {
                        assert_eq!(result_cell.bg, Color::Reset);
                        assert_eq!(result_cell.fg, Color::White);
                    }
                    // Title & y-axis max
                    (0, 5..=33) | (4 | 6, 1..=9) => {
                        assert_eq!(result_cell.bg, Color::Reset);
                        assert_eq!(result_cell.fg, Color::Red);
                    }
                    // Y axis
                    (1..=8, 10) => {
                        assert_eq!(result_cell.bg, Color::Reset);
                        assert_eq!(result_cell.fg, Color::White);
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
    /// Custom colours correctly applied to each part of the charts
    fn test_draw_blocks_charts_custom_colors() {
        let mut colors = AppColors::new();

        colors.chart_bandwidth.background = Color::White;
        colors.chart_bandwidth.border = Color::Red;
        colors.chart_bandwidth.max_rx = Color::Green;
        colors.chart_bandwidth.max_tx = Color::Magenta;
        colors.chart_bandwidth.title_rx = Color::LightGreen;
        colors.chart_bandwidth.title_tx = Color::LightRed;
        colors.chart_bandwidth.points_rx = Color::Black;
        colors.chart_bandwidth.points_tx = Color::Blue;
        colors.chart_bandwidth.y_axis = Color::Yellow;

        let mut setup = test_setup(40, 10, true, true);

        let mut rx = NetworkBandwidth::new();
        let mut tx = NetworkBandwidth::new();
        for i in 0..=20 {
            rx.push(2000 * i * (10 + 7 * i));
            tx.push(200 * i * (10 + 7 * i));
        }

        if let Some(item) = setup
            .app_data
            .lock()
            .get_container_by_id(&ContainerId::from("1"))
        {
            item.rx = rx;
            item.tx = tx;
        }

        let fd = FrameData::from((&setup.app_data, &setup.gui_state));

        setup
            .terminal
            .draw(|f| {
                super::draw(setup.area, colors, f, &fd);
            })
            .unwrap();

        assert_snapshot!(setup.terminal.backend());

        for (row_index, result_row) in get_result(&setup) {
            for (result_cell_index, result_cell) in result_row.iter().enumerate() {
                match (row_index, result_cell_index) {
                    // border
                    (9, _) | (1..=9, 0 | 39) => {
                        assert_eq!(result_cell.bg, Color::White);
                        assert_eq!(result_cell.fg, Color::Red);
                    }
                    // Border first row only
                    (0, 0..=3 | 36..=39) => {
                        assert_eq!(result_cell.bg, Color::White);
                        assert_eq!(result_cell.fg, Color::Red);
                    }
                    // Title RX
                    (0, 4..=19) => {
                        assert_eq!(result_cell.bg, Color::White);
                        assert_eq!(result_cell.fg, Color::LightGreen);
                    }
                    // Title TX
                    (0, 20..=35) => {
                        assert_eq!(result_cell.bg, Color::White);
                        assert_eq!(result_cell.fg, Color::LightRed);
                    }
                    // Y axis
                    (1..=8, 12) => {
                        assert_eq!(result_cell.bg, Color::White);
                        assert_eq!(result_cell.fg, Color::Yellow);
                    }
                    // RX max
                    (4, 1..=11) => {
                        assert_eq!(result_cell.bg, Color::White);
                        assert_eq!(result_cell.fg, Color::Green);
                    }
                    // TX max
                    (6, 1..=10) => {
                        assert_eq!(result_cell.bg, Color::White);
                        assert_eq!(result_cell.fg, Color::Magenta);
                    }
                    // TX dots
                    _x if COMBINED_DOTS_TX.contains(&(row_index, result_cell_index)) => {
                        assert_eq!(result_cell.bg, Color::White);
                        assert_eq!(result_cell.fg, Color::Blue);
                    }
                    // RX dots
                    _x if COMBINED_DOTS_RX.contains(&(row_index, result_cell_index)) => {
                        assert_eq!(result_cell.bg, Color::White);
                        assert_eq!(result_cell.fg, Color::Black);
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
