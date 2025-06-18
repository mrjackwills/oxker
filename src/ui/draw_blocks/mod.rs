use std::sync::Arc;

use parking_lot::Mutex;
use ratatui::{
    layout::{Constraint, Rect},
    style::Style,
    widgets::{Block, BorderType, Borders},
};

use crate::config::AppColors;

use super::{FrameData, GuiState, SelectablePanel, Status, gui_state::Region};

pub mod charts;
pub mod commands;
pub mod containers;
pub mod delete_confirm;
pub mod error;
pub mod filter;
pub mod headers;
pub mod help;
pub mod info;
pub mod logs;
pub mod popup;
pub mod ports;

pub const NAME_TEXT: &str = r#"
                          88                               
                          88                               
                          88                               
 ,adPPYba,   8b,     ,d8  88   ,d8    ,adPPYba,  8b,dPPYba,
a8"     "8a   `Y8, ,8P'   88 ,a8"    a8P_____88  88P'   "Y8
8b       d8     )888(     8888[      8PP"""""""  88        
"8a,   ,a8"   ,d8" "8b,   88`"Yba,   "8b,   ,aa  88        
 `"YbbdP"'   8P'     `Y8  88   `Y8a   `"Ybbd8"'  88        "#;

pub const NAME: &str = env!("CARGO_PKG_NAME");
pub const REPO: &str = env!("CARGO_PKG_REPOSITORY");
pub const DESCRIPTION: &str = env!("CARGO_PKG_DESCRIPTION");
pub const MARGIN: &str = "   ";
pub const RIGHT_ARROW: &str = "▶ ";
pub const CIRCLE: &str = "⚪ ";

#[cfg(not(test))]
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
#[cfg(test)]
pub const VERSION: &str = "0.00.000";

pub const CONSTRAINT_50_50: [Constraint; 2] =
    [Constraint::Percentage(50), Constraint::Percentage(50)];
pub const CONSTRAINT_100: [Constraint; 1] = [Constraint::Percentage(100)];
pub const CONSTRAINT_POPUP: [Constraint; 5] = [
    Constraint::Min(2),
    Constraint::Max(1),
    Constraint::Max(1),
    Constraint::Max(3),
    Constraint::Min(1),
];

pub const CONSTRAINT_BUTTONS: [Constraint; 5] = [
    Constraint::Percentage(10),
    Constraint::Percentage(35),
    Constraint::Percentage(10),
    Constraint::Percentage(35),
    Constraint::Percentage(10),
];

/// From a given &str, return the maximum number of chars on a single line
pub fn max_line_width(text: &str) -> usize {
    text.lines()
        .map(|i| i.chars().count())
        .max()
        .unwrap_or_default()
}

/// Generate block, add a border if is the selected panel,
/// add custom title based on state of each panel
fn generate_block<'a>(
    area: Rect,
    colors: AppColors,
    fd: &FrameData,
    gui_state: &Arc<Mutex<GuiState>>,
    panel: SelectablePanel,
) -> Block<'a> {
    gui_state
        .lock()
        .update_region_map(Region::Panel(panel), area);

    let mut title = match panel {
        SelectablePanel::Containers => {
            format!("{}{}", panel.title(), fd.container_title)
        }
        SelectablePanel::Logs => {
            format!("{}{}", panel.title(), fd.log_title)
        }
        SelectablePanel::Commands => String::new(),
    };
    if !title.is_empty() {
        title = format!(" {title} ");
    }
    let mut block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .title(title);
    if !fd.status.contains(&Status::Filter) {
        if fd.selected_panel == panel {
            block = block.border_style(Style::default().fg(colors.borders.selected));
        } else {
            block = block.border_style(Style::default().fg(colors.borders.unselected));
        }
    }
    block
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
pub mod tests {

    use std::{
        net::{IpAddr, Ipv4Addr},
        sync::Arc,
    };

    use insta::assert_snapshot;
    use parking_lot::Mutex;
    use ratatui::{Terminal, backend::TestBackend, layout::Rect, style::Color};

    use crate::{
        app_data::{AppData, ContainerId, ContainerImage, ContainerName, ContainerPorts},
        app_error::AppError,
        tests::{gen_appdata, gen_containers},
        ui::{GuiState, Rerender, Status, draw_frame},
    };

    use super::FrameData;

    pub struct TuiTestSetup {
        pub app_data: Arc<Mutex<AppData>>,
        pub gui_state: Arc<Mutex<GuiState>>,
        pub fd: FrameData,
        pub area: Rect,
        pub terminal: Terminal<TestBackend>,
        pub ids: Vec<ContainerId>,
    }

    pub const BORDER_CHARS: [&str; 6] = ["╭", "╮", "─", "│", "╰", "╯"];
    pub const COLOR_RX: Color = Color::Rgb(255, 233, 193);
    pub const COLOR_TX: Color = Color::Rgb(205, 140, 140);
    pub const COLOR_ORANGE: Color = Color::Rgb(255, 178, 36);

    /// Create a FrameData struct from two Arc<mutex>'s, instead of from UI
    impl From<(&Arc<Mutex<AppData>>, &Arc<Mutex<GuiState>>)> for FrameData {
        fn from(data: (&Arc<Mutex<AppData>>, &Arc<Mutex<GuiState>>)) -> Self {
            let (app_data, gui_data) = (data.0.lock(), data.1.lock());

            // let container_section_height = app_data.get_container_len();
            // let container_section_height = if container_section_height < 12 {
            //     u16::try_from(container_section_height + 5).unwrap_or_default()
            // } else {
            //     12
            // };

            let (filter_by, filter_term) = app_data.get_filter();
            Self {
                chart_data: app_data.get_chart_data(),
                color_logs: app_data.config.color_logs,
                columns: app_data.get_width(),
                // container_section_height,
                container_title: app_data.get_container_title(),
                delete_confirm: gui_data.get_delete_container(),
                filter_by,
                filter_term: filter_term.cloned(),
                has_containers: app_data.get_container_len() > 0,
                has_error: app_data.get_error(),
                show_logs: gui_data.get_show_logs(),
                info_text: gui_data.info_box_text.clone(),
                is_loading: gui_data.is_loading(),
                loading_icon: gui_data.get_loading().to_string(),
                log_height: gui_data.get_log_height(),
                log_title: app_data.get_log_title(),
                port_max_lens: app_data.get_longest_port(),
                ports: app_data.get_selected_ports(),
                selected_panel: gui_data.get_selected_panel(),
                sorted_by: app_data.get_sorted(),
                status: gui_data.get_status(),
            }
        }
    }

    /// Generate state to be used in *most* gui tests
    pub fn test_setup(w: u16, h: u16, control_start: bool, container_start: bool) -> TuiTestSetup {
        let backend = TestBackend::new(w, h);
        let terminal = Terminal::new(backend).unwrap();

        let (ids, containers) = gen_containers();
        let mut app_data = gen_appdata(&containers);
        if control_start {
            app_data.docker_controls_start();
        }
        if container_start {
            app_data.containers_start();
        }

        let redraw = Arc::new(Rerender::new());
        let gui_state = GuiState::new(&redraw, app_data.config.show_logs);

        let app_data = Arc::new(Mutex::new(app_data));
        let gui_state = Arc::new(Mutex::new(gui_state));
        let fd = FrameData::from((&app_data, &gui_state));
        let area = Rect::new(0, 0, w, h);
        TuiTestSetup {
            app_data,
            gui_state,
            fd,
            area,
            terminal,
            ids,
        }
    }

    /// Just a shorthand for when enumerating over result cells
    pub fn get_result(
        setup: &TuiTestSetup,
        // w: u16,
    ) -> std::iter::Enumerate<std::slice::Chunks<ratatui::buffer::Cell>> {
        setup
            .terminal
            .backend()
            .buffer()
            .content
            .chunks(usize::from(setup.area.width))
            .enumerate()
    }

    /// Insert some logs into the first container
    pub fn insert_logs(setup: &TuiTestSetup) {
        let logs = (1..=3).map(|i| format!("{i} line {i}")).collect::<Vec<_>>();
        setup.app_data.lock().update_log_by_id(logs, &setup.ids[0]);
    }

    #[allow(clippy::cast_precision_loss)]
    // Add fixed data to the cpu & mem vecdeques
    pub fn insert_chart_data(setup: &TuiTestSetup) {
        for i in 1..=10 {
            setup.app_data.lock().update_stats_by_id(
                &setup.ids[0],
                Some(i as f64),
                Some(i * 10000),
                i * 10000,
                i,
                i,
            );
        }
        for i in 1..=3 {
            setup.app_data.lock().update_stats_by_id(
                &setup.ids[0],
                Some(i as f64),
                Some(i * 10000),
                i * 10000,
                i,
                i,
            );
        }
    }

    // *************** //
    // The whole layout //
    // **************** //
    #[test]
    /// Check that the whole layout is drawn correctly
    fn test_draw_blocks_whole_layout() {
        let mut setup = test_setup(160, 30, true, true);

        insert_chart_data(&setup);
        insert_logs(&setup);
        setup.app_data.lock().containers.items[0]
            .ports
            .push(ContainerPorts {
                ip: Some(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1))),
                private: 8003,
                public: Some(8003),
            });
        let colors = setup.app_data.lock().config.app_colors;
        let keymap = setup.app_data.lock().config.keymap.clone();

        let fd = FrameData::from((&setup.app_data, &setup.gui_state));
        setup
            .terminal
            .draw(|f| {
                draw_frame(&setup.app_data, colors, &keymap, f, &fd, &setup.gui_state);
            })
            .unwrap();

        assert_snapshot!(setup.terminal.backend());
    }

    #[test]
    #[allow(clippy::too_many_lines)]
    /// Check that the whole layout is drawn correctly
    fn test_draw_blocks_whole_layout_with_filter_bar() {
        let mut setup = test_setup(160, 30, true, true);
        insert_chart_data(&setup);
        insert_logs(&setup);

        setup.app_data.lock().containers.items[1]
            .ports
            .push(ContainerPorts {
                ip: Some(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1))),
                private: 8003,
                public: Some(8003),
            });

        let colors = setup.app_data.lock().config.app_colors;
        let keymap = setup.app_data.lock().config.keymap.clone();
        setup
            .gui_state
            .lock()
            .status_push(crate::ui::Status::Filter);
        setup.app_data.lock().filter_term_push('r');
        setup.app_data.lock().filter_term_push('_');
        setup.app_data.lock().filter_term_push('1');
        let fd = FrameData::from((&setup.app_data, &setup.gui_state));
        setup
            .terminal
            .draw(|f| {
                draw_frame(&setup.app_data, colors, &keymap, f, &fd, &setup.gui_state);
            })
            .unwrap();

        assert_snapshot!(setup.terminal.backend());
    }

    #[test]
    /// Check that the whole layout is drawn correctly when have long container name and long image name
    fn test_draw_blocks_whole_layout_long_name() {
        let mut setup = test_setup(190, 30, true, true);

        insert_chart_data(&setup);
        insert_logs(&setup);
        setup.app_data.lock().containers.items[0]
            .ports
            .push(ContainerPorts {
                ip: Some(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1))),
                private: 8003,
                public: Some(8003),
            });

        setup.app_data.lock().containers.items[0].name =
            ContainerName::from("a_long_container_name_for_the_purposes_of_this_test");
        setup.app_data.lock().containers.items[0].image =
            ContainerImage::from("a_long_image_name_for_the_purposes_of_this_test");

        let fd = FrameData::from((&setup.app_data, &setup.gui_state));
        let colors = setup.app_data.lock().config.app_colors;
        let keymap = setup.app_data.lock().config.keymap.clone();
        setup
            .terminal
            .draw(|f| {
                draw_frame(&setup.app_data, colors, &keymap, f, &fd, &setup.gui_state);
            })
            .unwrap();

        assert_snapshot!(setup.terminal.backend());
    }

    #[test]
    /// Check that the whole layout is drawn correctly when the logs panel is removed
    fn test_draw_blocks_whole_layout_no_logs() {
        let mut setup = test_setup(160, 30, true, true);

        insert_chart_data(&setup);
        insert_logs(&setup);
        setup.app_data.lock().containers.items[0]
            .ports
            .push(ContainerPorts {
                ip: Some(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1))),
                private: 8003,
                public: Some(8003),
            });
        let colors = setup.app_data.lock().config.app_colors;
        let keymap = setup.app_data.lock().config.keymap.clone();
        setup.gui_state.lock().log_height_zero();

        let fd = FrameData::from((&setup.app_data, &setup.gui_state));
        setup
            .terminal
            .draw(|f| {
                draw_frame(&setup.app_data, colors, &keymap, f, &fd, &setup.gui_state);
            })
            .unwrap();

        assert_snapshot!(setup.terminal.backend());
    }

    #[test]
    /// Check that the whole layout is drawn correctly when the logs panel height is ~4
    fn test_draw_blocks_whole_layout_short_height_logs() {
        let mut setup = test_setup(160, 30, true, true);

        insert_chart_data(&setup);
        insert_logs(&setup);
        setup.app_data.lock().containers.items[0]
            .ports
            .push(ContainerPorts {
                ip: Some(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1))),
                private: 8003,
                public: Some(8003),
            });
        let colors = setup.app_data.lock().config.app_colors;
        let keymap = setup.app_data.lock().config.keymap.clone();
        setup.gui_state.lock().log_height_zero();

        for _ in 0..=3 {
            setup.gui_state.lock().log_height_increase();
        }
        let fd = FrameData::from((&setup.app_data, &setup.gui_state));
        setup
            .terminal
            .draw(|f| {
                draw_frame(&setup.app_data, colors, &keymap, f, &fd, &setup.gui_state);
            })
            .unwrap();

        assert_snapshot!(setup.terminal.backend());
    }

    #[test]
    /// Check that the whole layout is drawn with the help panel visible
    fn test_draw_blocks_whole_layout_help_panel() {
        let mut setup = test_setup(160, 40, true, true);

        insert_chart_data(&setup);
        insert_logs(&setup);
        setup.app_data.lock().containers.items[0]
            .ports
            .push(ContainerPorts {
                ip: Some(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1))),
                private: 8003,
                public: Some(8003),
            });
        let colors = setup.app_data.lock().config.app_colors;
        let keymap = setup.app_data.lock().config.keymap.clone();

        setup.gui_state.lock().status_push(Status::Help);

        let fd = FrameData::from((&setup.app_data, &setup.gui_state));
        setup
            .terminal
            .draw(|f| {
                draw_frame(&setup.app_data, colors, &keymap, f, &fd, &setup.gui_state);
            })
            .unwrap();

        assert_snapshot!(setup.terminal.backend());
    }

    #[test]
    /// Check that the whole layout is drawn with the error box is visible
    fn test_draw_blocks_whole_layout_error() {
        let mut setup = test_setup(160, 40, true, true);

        insert_chart_data(&setup);
        insert_logs(&setup);
        setup.app_data.lock().containers.items[0]
            .ports
            .push(ContainerPorts {
                ip: Some(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1))),
                private: 8003,
                public: Some(8003),
            });
        let colors = setup.app_data.lock().config.app_colors;
        let keymap = setup.app_data.lock().config.keymap.clone();

        setup.app_data.lock().set_error(
            AppError::DockerCommand(crate::app_data::DockerCommand::Pause),
            &setup.gui_state,
            Status::Error,
        );

        let fd = FrameData::from((&setup.app_data, &setup.gui_state));
        setup
            .terminal
            .draw(|f| {
                draw_frame(&setup.app_data, colors, &keymap, f, &fd, &setup.gui_state);
            })
            .unwrap();

        assert_snapshot!(setup.terminal.backend());
    }

    #[test]
    /// Check that the whole layout is drawn with the delete box is visible
    fn test_draw_blocks_whole_layout_delete() {
        let mut setup = test_setup(160, 40, true, true);

        insert_chart_data(&setup);
        insert_logs(&setup);
        setup.app_data.lock().containers.items[0]
            .ports
            .push(ContainerPorts {
                ip: Some(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1))),
                private: 8003,
                public: Some(8003),
            });
        let colors = setup.app_data.lock().config.app_colors;
        let keymap = setup.app_data.lock().config.keymap.clone();
        setup
            .gui_state
            .lock()
            .set_delete_container(setup.app_data.lock().get_selected_container_id());

        let fd = FrameData::from((&setup.app_data, &setup.gui_state));
        setup
            .terminal
            .draw(|f| {
                draw_frame(&setup.app_data, colors, &keymap, f, &fd, &setup.gui_state);
            })
            .unwrap();

        assert_snapshot!(setup.terminal.backend());
    }

    #[test]
    /// Check that the whole layout is drawn with the info box is visible
    fn test_draw_blocks_whole_layout_info_box() {
        let mut setup = test_setup(160, 40, true, true);

        insert_chart_data(&setup);
        insert_logs(&setup);
        setup.app_data.lock().containers.items[0]
            .ports
            .push(ContainerPorts {
                ip: Some(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1))),
                private: 8003,
                public: Some(8003),
            });
        let colors = setup.app_data.lock().config.app_colors;
        let keymap = setup.app_data.lock().config.keymap.clone();
        setup.gui_state.lock().set_info_box("This is a test");
        let fd = FrameData::from((&setup.app_data, &setup.gui_state));
        setup
            .terminal
            .draw(|f| {
                draw_frame(&setup.app_data, colors, &keymap, f, &fd, &setup.gui_state);
            })
            .unwrap();

        assert_snapshot!(setup.terminal.backend());
    }
}
