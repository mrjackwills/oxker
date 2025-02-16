use ratatui::{
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Paragraph},
    Frame,
};

use crate::{app_data::State, config::AppColors, ui::FrameData};

/// Get the port title color, at the moment the color is only customizable if the container is alive
const fn get_port_title_color(colors: AppColors, state: State) -> Color {
    if state.is_alive() {
        colors.chart_ports.title
    } else {
        state.get_color(colors)
    }
}

/// Display the ports in a formatted list
pub fn draw(area: Rect, colors: AppColors, f: &mut Frame, fd: &FrameData) {
    if let Some(ports) = fd.ports.as_ref() {
        let block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .style(Style::new().fg(colors.chart_ports.border))
            // .bg(colors.chart_ports.border))
            .title_alignment(Alignment::Center)
            .title(Span::styled(
                " ports ",
                Style::default()
                    .fg(get_port_title_color(colors, ports.1))
                    .add_modifier(Modifier::BOLD),
            ));

        let (ip, private, public) = fd.port_max_lens;

        if ports.0.is_empty() {
            let text = match ports.1 {
                State::Running(_) | State::Paused | State::Restarting => "no ports",
                _ => "",
            };
            let paragraph = Paragraph::new(Span::from(text).add_modifier(Modifier::BOLD))
                .alignment(Alignment::Center)
                .block(block);
            f.render_widget(paragraph, area);
        } else {
            let mut output = vec![Line::from(
                Span::from(format!(
                    "{:>ip$}{:>private$}{:>public$}",
                    "ip", "private", "public"
                ))
                .fg(colors.chart_ports.headings),
            )];
            for item in &ports.0 {
                let strings = item.get_all();

                let line = vec![
                    Span::from(format!("{:>ip$}", strings.0)).fg(colors.chart_ports.text),
                    Span::from(format!("{:>private$}", strings.1)).fg(colors.chart_ports.text),
                    Span::from(format!("{:>public$}", strings.2)).fg(colors.chart_ports.text),
                ];
                output.push(Line::from(line));
            }
            let paragraph = Paragraph::new(output).block(block);
            f.render_widget(paragraph, area);
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use std::net::{IpAddr, Ipv4Addr};

    use ratatui::style::{Color, Modifier};

    use crate::{
        app_data::{ContainerPorts, State},
        ui::{
            draw_blocks::tests::{expected_to_vec, get_result, test_setup},
            FrameData,
        },
    };

    #[test]
    /// Port section when container has no ports
    fn test_draw_blocks_ports_no_ports() {
        let (w, h) = (30, 8);
        let mut setup = test_setup(w, h, true, true);
        setup.app_data.lock().containers.items[0].ports = vec![];

        let fd = FrameData::from((&setup.app_data, &setup.gui_state));
        setup
            .terminal
            .draw(|f| {
                super::draw(setup.area, setup.app_data.lock().config.app_colors, f, &fd);
            })
            .unwrap();

        let expected = [
            "╭────────── ports ───────────╮",
            "│          no ports          │",
            "│                            │",
            "│                            │",
            "│                            │",
            "│                            │",
            "│                            │",
            "╰────────────────────────────╯",
        ];

        for (row_index, result_row) in get_result(&setup, w) {
            let expected_row = expected_to_vec(&expected, row_index);
            for (result_cell_index, result_cell) in result_row.iter().enumerate() {
                assert_eq!(result_cell.symbol(), expected_row[result_cell_index]);
                match (row_index, result_cell_index) {
                    (0, 11..=17) => {
                        assert_eq!(result_cell.bg, Color::Reset);
                        assert_eq!(result_cell.fg, Color::Green);
                        assert_eq!(result_cell.modifier, Modifier::BOLD);
                    }
                    (1, 11..=18) => {
                        assert_eq!(result_cell.bg, Color::Reset);
                        assert_eq!(result_cell.fg, Color::White);
                        assert_eq!(result_cell.modifier, Modifier::BOLD);
                    }
                    _ => {
                        assert_eq!(result_cell.bg, Color::Reset);
                        assert_eq!(result_cell.fg, Color::White);
                        assert!(result_cell.modifier.is_empty());
                    }
                }
            }
        }

        // When state is "State::Running | State::Paused | State::Restarting, won't show "no ports"
        setup.app_data.lock().containers.items[0].state = State::Dead;

        let fd = FrameData::from((&setup.app_data, &setup.gui_state));
        setup
            .terminal
            .draw(|f| {
                super::draw(setup.area, setup.app_data.lock().config.app_colors, f, &fd);
            })
            .unwrap();

        let expected = [
            "╭────────── ports ───────────╮",
            "│                            │",
            "│                            │",
            "│                            │",
            "│                            │",
            "│                            │",
            "│                            │",
            "╰────────────────────────────╯",
        ];

        for (row_index, result_row) in get_result(&setup, w) {
            let expected_row = expected_to_vec(&expected, row_index);
            for (result_cell_index, result_cell) in result_row.iter().enumerate() {
                assert_eq!(result_cell.symbol(), expected_row[result_cell_index]);
                assert_eq!(result_cell.bg, Color::Reset);
                if let (0, 11..=17) = (row_index, result_cell_index) {
                    assert_eq!(result_cell.fg, Color::Red);
                    assert_eq!(result_cell.modifier, Modifier::BOLD);
                } else {
                    assert_eq!(result_cell.fg, Color::White);
                    assert!(result_cell.modifier.is_empty());
                }
            }
        }
    }

    #[test]
    /// Port section when container has multiple ports
    fn test_draw_blocks_ports_multiple_ports() {
        let (w, h) = (32, 8);
        let mut setup = test_setup(w, h, true, true);
        setup.app_data.lock().containers.items[0]
            .ports
            .push(ContainerPorts {
                ip: None,
                private: 8002,
                public: None,
            });
        setup.app_data.lock().containers.items[0]
            .ports
            .push(ContainerPorts {
                ip: Some(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1))),
                private: 8003,
                public: Some(8003),
            });

        let fd = FrameData::from((&setup.app_data, &setup.gui_state));
        setup
            .terminal
            .draw(|f| {
                super::draw(setup.area, setup.app_data.lock().config.app_colors, f, &fd);
            })
            .unwrap();

        let expected = [
            "╭─────────── ports ────────────╮",
            "│       ip   private   public  │",
            "│               8001           │",
            "│               8002           │",
            "│127.0.0.1      8003     8003  │",
            "│                              │",
            "│                              │",
            "╰──────────────────────────────╯",
        ];

        for (row_index, result_row) in get_result(&setup, w) {
            let expected_row = expected_to_vec(&expected, row_index);
            for (result_cell_index, result_cell) in result_row.iter().enumerate() {
                assert_eq!(result_cell.symbol(), expected_row[result_cell_index]);
                assert_eq!(result_cell.bg, Color::Reset);

                match (row_index, result_cell_index) {
                    (0, 12..=18) => {
                        assert_eq!(result_cell.fg, Color::Green);
                        assert_eq!(result_cell.modifier, Modifier::BOLD);
                    }
                    (1, 1..=28) => {
                        assert_eq!(result_cell.fg, Color::Yellow);
                        assert!(result_cell.modifier.is_empty());
                    }
                    (2..=4, 1..=28) | (0 | 2..=9, 0..=31) | (1, 0 | 29..=31) => {
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
    /// Port section title color correct dependant on state
    fn test_draw_blocks_ports_container_state() {
        let (w, h) = (32, 8);
        let mut setup = test_setup(w, h, true, true);

        let fd = FrameData::from((&setup.app_data, &setup.gui_state));
        setup
            .terminal
            .draw(|f| {
                super::draw(setup.area, setup.app_data.lock().config.app_colors, f, &fd);
            })
            .unwrap();

        let expected = [
            "╭─────────── ports ────────────╮",
            "│   ip   private   public      │",
            "│           8001               │",
            "│                              │",
            "│                              │",
            "│                              │",
            "│                              │",
            "╰──────────────────────────────╯",
        ];

        for (row_index, result_row) in get_result(&setup, w) {
            let expected_row = expected_to_vec(&expected, row_index);
            for (result_cell_index, result_cell) in result_row.iter().enumerate() {
                assert_eq!(result_cell.symbol(), expected_row[result_cell_index]);
                assert_eq!(result_cell.bg, Color::Reset);
                if let (0, 12..=18) = (row_index, result_cell_index) {
                    assert_eq!(result_cell.fg, Color::Green);
                    assert_eq!(result_cell.modifier, Modifier::BOLD);
                }
            }
        }

        setup.app_data.lock().containers.items[0].state = State::Paused;
        let fd = FrameData::from((&setup.app_data, &setup.gui_state));
        setup
            .terminal
            .draw(|f| {
                super::draw(setup.area, setup.app_data.lock().config.app_colors, f, &fd);
            })
            .unwrap();

        for (row_index, result_row) in get_result(&setup, w) {
            let expected_row = expected_to_vec(&expected, row_index);
            for (result_cell_index, result_cell) in result_row.iter().enumerate() {
                assert_eq!(result_cell.symbol(), expected_row[result_cell_index]);
                assert_eq!(result_cell.bg, Color::Reset);
                if let (0, 12..=18) = (row_index, result_cell_index) {
                    assert_eq!(result_cell.fg, Color::Yellow);
                    assert_eq!(result_cell.modifier, Modifier::BOLD);
                }
            }
        }

        setup.app_data.lock().containers.items[0].state = State::Exited;
        let fd = FrameData::from((&setup.app_data, &setup.gui_state));
        setup
            .terminal
            .draw(|f| {
                super::draw(setup.area, setup.app_data.lock().config.app_colors, f, &fd);
            })
            .unwrap();

        for (row_index, result_row) in get_result(&setup, w) {
            let expected_row = expected_to_vec(&expected, row_index);
            for (result_cell_index, result_cell) in result_row.iter().enumerate() {
                assert_eq!(result_cell.symbol(), expected_row[result_cell_index]);
                assert_eq!(result_cell.bg, Color::Reset);
                if let (0, 12..=18) = (row_index, result_cell_index) {
                    assert_eq!(result_cell.fg, Color::Red);
                    assert_eq!(result_cell.modifier, Modifier::BOLD);
                }
            }
        }
    }
}
