use std::fmt::Display;

use ratatui::{
    layout::{Alignment, Direction, Layout, Rect},
    style::{Color, Modifier, Style, Stylize},
    symbols,
    text::Span,
    widgets::{Axis, Block, BorderType, Borders, Chart, Dataset, GraphType},
    Frame,
};

use super::{FrameData, CONSTRAINT_50_50};
use crate::{
    app_data::{ByteStats, CpuStats, State, Stats},
    config::AppColors,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ChartType {
    Cpu,
    Memory,
}

impl ChartType {
    const fn name(self) -> &'static str {
        match self {
            Self::Cpu => "cpu",
            Self::Memory => "memory",
        }
    }

    const fn get_title_color(self, colors: AppColors, state: State) -> Color {
        if state.is_healthy() {
            match self {
                Self::Cpu => colors.chart_cpu.title,
                Self::Memory => colors.chart_memory.title,
            }
        } else {
            state.get_color(colors)
        }
    }

    const fn get_bg_color(self, colors: AppColors) -> Color {
        match self {
            Self::Cpu => colors.chart_cpu.background,
            Self::Memory => colors.chart_memory.background,
        }
    }

    const fn get_border_color(self, colors: AppColors) -> Color {
        match self {
            Self::Cpu => colors.chart_cpu.border,
            Self::Memory => colors.chart_memory.border,
        }
    }

    const fn get_y_axis_color(self, colors: AppColors) -> Color {
        match self {
            Self::Cpu => colors.chart_cpu.y_axis,
            Self::Memory => colors.chart_memory.y_axis,
        }
    }

    const fn get_max_color(self, colors: AppColors, state: State) -> Color {
        if state.is_healthy() {
            match self {
                Self::Cpu => colors.chart_cpu.max,
                Self::Memory => colors.chart_memory.max,
            }
        } else {
            state.get_color(colors)
        }
    }
}

/// Create charts
fn make_chart<'a, T: Stats + Display>(
    chart_type: ChartType,
    colors: AppColors,
    current: &'a T,
    dataset: Vec<Dataset<'a>>,
    max: &'a T,
    state: State,
) -> Chart<'a> {
    let max_color = chart_type.get_max_color(colors, state);

    Chart::new(dataset)
        .bg(chart_type.get_bg_color(colors))
        .block(
            Block::default()
                .style(Style::default().bg(chart_type.get_bg_color(colors)))
                .title_alignment(Alignment::Center)
                .title(Span::styled(
                    format!(" {} {current} ", chart_type.name()),
                    Style::default()
                        .fg(chart_type.get_title_color(colors, state))
                        .add_modifier(Modifier::BOLD),
                ))
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(chart_type.get_border_color(colors))),
        )
        .x_axis(Axis::default().bounds([0.00, 60.0]))
        .y_axis(
            Axis::default()
                .labels(vec![
                    Span::styled("", Style::default().fg(max_color)),
                    Span::styled(
                        format!("{max}"),
                        Style::default().add_modifier(Modifier::BOLD).fg(max_color),
                    ),
                ])
                .style(Style::new().fg(chart_type.get_y_axis_color(colors)))
                // Add 0.01, so that max point is always visible?
                .bounds([0.0, max.get_value() + 0.01]),
        )
}

/// Draw the cpu + mem charts
pub fn draw(area: Rect, colors: AppColors, f: &mut Frame, fd: &FrameData) {
    if let Some((cpu, mem)) = fd.chart_data.as_ref() {
        let area = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(CONSTRAINT_50_50)
            .split(area);

        let cpu_dataset = vec![Dataset::default()
            .marker(symbols::Marker::Dot)
            .style(Style::default().fg(colors.chart_cpu.points))
            .graph_type(GraphType::Line)
            .data(&cpu.0)];
        let mem_dataset = vec![Dataset::default()
            .marker(symbols::Marker::Dot)
            .style(Style::default().fg(colors.chart_memory.points))
            .graph_type(GraphType::Line)
            .data(&mem.0)];

        let cpu_stats = CpuStats::new(cpu.0.last().map_or(0.00, |f| f.1));
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let mem_stats = ByteStats::new(mem.0.last().map_or(0, |f| f.1 as u64));
        let cpu_chart = make_chart(
            ChartType::Cpu,
            colors,
            &cpu_stats,
            cpu_dataset,
            &cpu.1,
            cpu.2,
        );
        let mem_chart = make_chart(
            ChartType::Memory,
            colors,
            &mem_stats,
            mem_dataset,
            &mem.1,
            mem.2,
        );

        f.render_widget(cpu_chart, area[0]);
        f.render_widget(mem_chart, area[1]);
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use ratatui::style::{Color, Modifier};

    use crate::{
        app_data::State,
        config::AppColors,
        ui::{
            draw_blocks::tests::{
                expected_to_vec, get_result, insert_chart_data, test_setup, COLOR_ORANGE,
            },
            FrameData,
        },
    };

    /// CPU and Memory charts used in multiple tests, based on data from above insert_chart_data()
    const EXPECTED: [&str; 10] = [
        "╭───────────── cpu 03.00% ─────────────╮╭────────── memory 30.00 kB ───────────╮",
        "│10.00%│    •                          ││100.00 kB│   ••                       │",
        "│      │   ••                          ││         │   ••                       │",
        "│      │  •••                          ││         │  • •                       │",
        "│      │  • •                          ││         │ •  •                       │",
        "│      │ •   ••                        ││         │••  ••                      │",
        "│      │•    •                         ││         │•   •                       │",
        "│      │•    •                         ││         │•   •                       │",
        "│      │                               ││         │                            │",
        "╰──────────────────────────────────────╯╰──────────────────────────────────────╯",
    ];

    // co-ordinates of the dots from the cpu chart
    const CPU_XY: [(usize, usize); 15] = [
        (1, 12),
        (2, 11),
        (2, 12),
        (3, 10),
        (3, 11),
        (3, 12),
        (4, 10),
        (4, 12),
        (5, 9),
        (5, 13),
        (5, 14),
        (6, 8),
        (6, 13),
        (7, 8),
        (7, 13),
    ];

    // co-ordinates of the dots from the memory chart
    const MEM_XY: [(usize, usize); 16] = [
        (1, 54),
        (1, 55),
        (2, 54),
        (2, 55),
        (3, 53),
        (3, 55),
        (4, 52),
        (4, 55),
        (5, 51),
        (5, 52),
        (5, 55),
        (5, 56),
        (6, 51),
        (6, 55),
        (7, 51),
        (7, 55),
    ];

    #[test]
    /// When status is Running, but not data, charts drawn without dots etc, colours correct
    fn test_draw_blocks_charts_running_none() {
        let (w, h) = (80, 10);
        let mut setup = test_setup(w, h, true, true);

        let fd = FrameData::from((&setup.app_data, &setup.gui_state));
        setup
            .terminal
            .draw(|f| {
                super::draw(setup.area, setup.app_data.lock().config.app_colors, f, &fd);
            })
            .unwrap();

        let expected = [
            "╭───────────── cpu 00.00% ─────────────╮╭─────────── memory 0.00 kB ───────────╮",
            "│00.00%│                               ││0.00 kB│                              │",
            "│      │                               ││       │                              │",
            "│      │                               ││       │                              │",
            "│      │                               ││       │                              │",
            "│      │                               ││       │                              │",
            "│      │                               ││       │                              │",
            "│      │                               ││       │                              │",
            "│      │                               ││       │                              │",
            "╰──────────────────────────────────────╯╰──────────────────────────────────────╯",
        ];

        for (row_index, result_row) in get_result(&setup, w) {
            let expected_row = expected_to_vec(&expected, row_index);
            for (result_cell_index, result_cell) in result_row.iter().enumerate() {
                assert_eq!(result_cell.symbol(), expected_row[result_cell_index]);

                match (row_index, result_cell_index) {
                    (0, 14..=25 | 52..=67) => {
                        assert_eq!(result_cell.fg, Color::Green);
                        assert_eq!(result_cell.modifier, Modifier::BOLD);
                    }
                    (1, 1..=6 | 41..=47) => {
                        assert_eq!(result_cell.fg, COLOR_ORANGE);
                        assert_eq!(result_cell.modifier, Modifier::BOLD);
                    }
                    (2..=8, 1..=6 | 8..=38 | 49..=78 | 41..=47) | (1, 8..=38 | 49..=78) => {
                        assert_eq!(result_cell.fg, Color::Reset);
                        assert!(result_cell.modifier.is_empty());
                    }
                    _ => {
                        assert_eq!(result_cell.fg, Color::White);
                        assert!(result_cell.modifier.is_empty());
                    }
                }
            }
        }
    }

    #[test]
    /// When status is Running, charts correctly drawn
    fn test_draw_blocks_charts_running_some() {
        let (w, h) = (80, 10);
        let mut setup = test_setup(w, h, true, true);

        insert_chart_data(&setup);
        let fd = FrameData::from((&setup.app_data, &setup.gui_state));

        setup
            .terminal
            .draw(|f| {
                super::draw(setup.area, setup.app_data.lock().config.app_colors, f, &fd);
            })
            .unwrap();

        for (row_index, result_row) in get_result(&setup, w) {
            let expected_row = expected_to_vec(&EXPECTED, row_index);
            for (result_cell_index, result_cell) in result_row.iter().enumerate() {
                assert_eq!(result_cell.symbol(), expected_row[result_cell_index]);

                match (row_index, result_cell_index) {
                    (0, 14..=25 | 51..=67) => {
                        assert_eq!(result_cell.fg, Color::Green);
                        assert_eq!(result_cell.modifier, Modifier::BOLD);
                    }
                    (1, 1..=6 | 41..=49) => {
                        assert_eq!(result_cell.fg, COLOR_ORANGE);
                        assert_eq!(result_cell.modifier, Modifier::BOLD);
                    }
                    xy if CPU_XY.contains(&xy) => {
                        assert_eq!(result_cell.fg, Color::Magenta);
                        assert!(result_cell.modifier.is_empty());
                    }
                    xy if MEM_XY.contains(&xy) => {
                        assert_eq!(result_cell.fg, Color::Cyan);
                        assert!(result_cell.modifier.is_empty());
                    }
                    (0 | 9, 0..=80) | (1..=9, 0 | 7 | 39 | 40 | 50 | 79) => {
                        assert_eq!(result_cell.fg, Color::White);
                        assert!(result_cell.modifier.is_empty());
                    }
                    _ => {
                        assert_eq!(result_cell.fg, Color::Reset);
                        assert!(result_cell.modifier.is_empty());
                    }
                }
            }
        }
    }

    #[test]
    /// Whens status paused, some text is now Yellow
    fn test_draw_blocks_charts_paused() {
        let (w, h) = (80, 10);
        let mut setup = test_setup(w, h, true, true);

        insert_chart_data(&setup);
        setup.app_data.lock().containers.items[0].state = State::Paused;
        let fd = FrameData::from((&setup.app_data, &setup.gui_state));

        setup
            .terminal
            .draw(|f| {
                super::draw(setup.area, setup.app_data.lock().config.app_colors, f, &fd);
            })
            .unwrap();

        for (row_index, result_row) in get_result(&setup, w) {
            let expected_row = expected_to_vec(&EXPECTED, row_index);
            for (result_cell_index, result_cell) in result_row.iter().enumerate() {
                assert_eq!(result_cell.symbol(), expected_row[result_cell_index]);

                match (row_index, result_cell_index) {
                    (0, 14..=25 | 51..=67) | (1, 1..=6 | 41..=49) => {
                        assert_eq!(result_cell.fg, Color::Yellow);
                        assert_eq!(result_cell.modifier, Modifier::BOLD);
                    }
                    xy if CPU_XY.contains(&xy) => {
                        assert_eq!(result_cell.fg, Color::Magenta);
                        assert!(result_cell.modifier.is_empty());
                    }
                    xy if MEM_XY.contains(&xy) => {
                        assert_eq!(result_cell.fg, Color::Cyan);
                        assert!(result_cell.modifier.is_empty());
                    }
                    (0 | 9, 0..=80) | (1..=9, 0 | 7 | 39 | 40 | 50 | 79) => {
                        assert_eq!(result_cell.fg, Color::White);
                        assert!(result_cell.modifier.is_empty());
                    }
                    _ => {
                        assert_eq!(result_cell.fg, Color::Reset);
                        assert!(result_cell.modifier.is_empty());
                    }
                }
            }
        }
    }

    #[test]
    /// When dead, text is red
    fn test_draw_blocks_charts_dead() {
        let (w, h) = (80, 10);
        let mut setup = test_setup(w, h, true, true);
        insert_chart_data(&setup);
        setup.app_data.lock().containers.items[0].state = State::Dead;
        let fd = FrameData::from((&setup.app_data, &setup.gui_state));

        setup
            .terminal
            .draw(|f| {
                super::draw(setup.area, setup.app_data.lock().config.app_colors, f, &fd);
            })
            .unwrap();

        for (row_index, result_row) in get_result(&setup, w) {
            let expected_row = expected_to_vec(&EXPECTED, row_index);
            for (result_cell_index, result_cell) in result_row.iter().enumerate() {
                assert_eq!(result_cell.symbol(), expected_row[result_cell_index]);

                match (row_index, result_cell_index) {
                    (0, 14..=25 | 51..=67) | (1, 1..=6 | 41..=49) => {
                        assert_eq!(result_cell.fg, Color::Red);
                        assert_eq!(result_cell.modifier, Modifier::BOLD);
                    }
                    xy if CPU_XY.contains(&xy) => {
                        assert_eq!(result_cell.fg, Color::Magenta);
                        assert!(result_cell.modifier.is_empty());
                    }
                    xy if MEM_XY.contains(&xy) => {
                        assert_eq!(result_cell.fg, Color::Cyan);
                        assert!(result_cell.modifier.is_empty());
                    }
                    (0 | 9, 0..=80) | (1..=9, 0 | 7 | 39 | 40 | 50 | 79) => {
                        assert_eq!(result_cell.fg, Color::White);
                        assert!(result_cell.modifier.is_empty());
                    }
                    _ => {
                        assert_eq!(result_cell.fg, Color::Reset);
                        assert!(result_cell.modifier.is_empty());
                    }
                }
            }
        }
    }

    #[test]
    /// Custom colos correctly applied to each part of the charts
    fn test_draw_blocks_charts_custom_colors() {
        let mut colors = AppColors::new();

        colors.chart_cpu.background = Color::White;
        colors.chart_cpu.border = Color::Red;
        colors.chart_cpu.title = Color::Green;
        colors.chart_cpu.max = Color::Magenta;
        colors.chart_cpu.points = Color::Black;
        colors.chart_cpu.y_axis = Color::Blue;

        colors.chart_memory.background = Color::White;
        colors.chart_memory.border = Color::Red;
        colors.chart_memory.title = Color::Green;
        colors.chart_memory.max = Color::Magenta;
        colors.chart_memory.points = Color::Black;
        colors.chart_memory.y_axis = Color::Blue;

        let (w, h) = (80, 10);
        let mut setup = test_setup(w, h, true, true);

        insert_chart_data(&setup);
        let fd = FrameData::from((&setup.app_data, &setup.gui_state));

        setup
            .terminal
            .draw(|f| {
                super::draw(setup.area, colors, f, &fd);
            })
            .unwrap();

        for (row_index, result_row) in get_result(&setup, w) {
            let expected_row = expected_to_vec(&EXPECTED, row_index);
            for (result_cell_index, result_cell) in result_row.iter().enumerate() {
                assert_eq!(result_cell.symbol(), expected_row[result_cell_index]);
                assert_eq!(result_cell.bg, Color::White);

                match (row_index, result_cell_index) {
                    // border
                    (0, 0..=13 | 26..=50 | 68..=79) | (9, _) | (1..=8, 0 | 39 | 40 | 79) => {
                        assert_eq!(result_cell.fg, Color::Red);
                    }
                    // title
                    (0, 14..=25 | 51..=67) => {
                        assert_eq!(result_cell.fg, Color::Green);
                    }
                    // max label
                    (1, 1..=6 | 41..=49) => {
                        assert_eq!(result_cell.fg, Color::Magenta);
                    }
                    // data points
                    xy if CPU_XY.contains(&xy) | MEM_XY.contains(&xy) => {
                        assert_eq!(result_cell.fg, Color::Black);
                    }
                    // y axis
                    (1..=8, 7 | 50) => {
                        assert_eq!(result_cell.fg, Color::Blue);
                    }
                    _ => {
                        assert_eq!(result_cell.fg, Color::Reset);
                    }
                }
            }
        }
    }
}
