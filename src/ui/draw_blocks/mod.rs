use std::sync::Arc;

use parking_lot::Mutex;
use ratatui::{
    layout::{Constraint, Rect},
    style::Style,
    widgets::{Block, BorderType, Borders},
};

use crate::config::AppColors;

use super::{gui_state::Region, FrameData, GuiState, SelectablePanel, Status};

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
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const REPO: &str = env!("CARGO_PKG_REPOSITORY");
pub const DESCRIPTION: &str = env!("CARGO_PKG_DESCRIPTION");
pub const MARGIN: &str = "   ";
pub const RIGHT_ARROW: &str = "▶ ";
pub const CIRCLE: &str = "⚪ ";

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

    use parking_lot::Mutex;
    use ratatui::{backend::TestBackend, layout::Rect, style::Color, Terminal};

    use crate::{
        app_data::{AppData, ContainerId, ContainerImage, ContainerName, ContainerPorts},
        tests::{gen_appdata, gen_containers},
        ui::{draw_frame, GuiState},
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

            // set max height for container section, needs +5 to deal with docker commands list and borders
            let height = app_data.get_container_len();
            let height = if height < 12 {
                u16::try_from(height + 5).unwrap_or_default()
            } else {
                12
            };

            let (filter_by, filter_term) = app_data.get_filter();
            Self {
                chart_data: app_data.get_chart_data(),
                columns: app_data.get_width(),
                color_logs: app_data.config.color_logs,
                container_title: app_data.get_container_title(),
                delete_confirm: gui_data.get_delete_container(),
                filter_by,
                filter_term: filter_term.cloned(),
                has_containers: app_data.get_container_len() > 0,
                has_error: app_data.get_error(),
                height,
                ports: app_data.get_selected_ports(),
                port_max_lens: app_data.get_longest_port(),
                info_text: gui_data.info_box_text.clone(),
                is_loading: gui_data.is_loading(),
                loading_icon: gui_data.get_loading().to_string(),
                log_title: app_data.get_log_title(),
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

        let gui_state = GuiState::default();

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

    /// Get a single row of String's from the expected data
    pub fn expected_to_vec(expected: &[&str], row_index: usize) -> Vec<String> {
        expected[row_index]
            .chars()
            .map(|i| i.to_string())
            .collect::<Vec<_>>()
    }

    /// Just a shorthand for when enumerating over result cells
    pub fn get_result(
        setup: &TuiTestSetup,
        w: u16,
    ) -> std::iter::Enumerate<std::slice::Chunks<ratatui::buffer::Cell>> {
        setup
            .terminal
            .backend()
            .buffer()
            .content
            .chunks(usize::from(w))
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
        let (w, h) = (160, 30);
        let mut setup = test_setup(w, h, true, true);

        insert_chart_data(&setup);
        insert_logs(&setup);
        setup.app_data.lock().containers.items[0]
            .ports
            .push(ContainerPorts {
                ip: Some(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1))),
                private: 8003,
                public: Some(8003),
            });

        let expected = [
            "    name          state       status      cpu      memory/limit          id         image     ↓ rx      ↑ tx                                  ( h ) show help   ",
            "╭ Containers 1/3 ──────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────╮╭──────────────╮",
            "│⚪  container_1   ✓ running   Up 1 hour   03.00%   30.00 kB / 30.00 kB          1   image_1   0.00 kB   0.00 kB                                ││▶ pause       │",
            "│   container_2   ✓ running   Up 2 hour   00.00%    0.00 kB /  0.00 kB          2   image_2   0.00 kB   0.00 kB                                ││  restart     │",
            "│   container_3   ✓ running   Up 3 hour   00.00%    0.00 kB /  0.00 kB          3   image_3   0.00 kB   0.00 kB                                ││  stop        │",
            "│                                                                                                                                              ││  delete      │",
            "│                                                                                                                                              ││              │",
            "│                                                                                                                                              ││              │",
            "╰──────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────╯╰──────────────╯",
            "╭ Logs 3/3 - container_1 - image_1 ────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────╮",
            "│  line 1                                                                                                                                                      │",
            "│  line 2                                                                                                                                                      │",
            "│▶ line 3                                                                                                                                                      │",
            "│                                                                                                                                                              │",
            "│                                                                                                                                                              │",
            "│                                                                                                                                                              │",
            "│                                                                                                                                                              │",
            "│                                                                                                                                                              │",
            "│                                                                                                                                                              │",
            "│                                                                                                                                                              │",
            "│                                                                                                                                                              │",
            "│                                                                                                                                                              │",
            "│                                                                                                                                                              │",
            "╰──────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────╯",
            "╭───────────────────────── cpu 03.00% ──────────────────────────╮╭─────────────────────── memory 30.00 kB ───────────────────────╮╭────────── ports ───────────╮",
            "│10.00%│     ••••                                               ││100.00 kB│     •••                                             ││       ip   private   public│",
            "│      │  •••   •                                               ││         │  •••  •                                             ││               8001         │",
            "│      │••       •••                                            ││         │••      •••                                          ││127.0.0.1      8003     8003│",
            "│      │                                                        ││         │                                                     ││                            │",
            "╰───────────────────────────────────────────────────────────────╯╰───────────────────────────────────────────────────────────────╯╰────────────────────────────╯",
        ];
        let colors = setup.app_data.lock().config.app_colors;
        let keymap = setup.app_data.lock().config.keymap.clone();

        let fd = FrameData::from((&setup.app_data, &setup.gui_state));
        setup
            .terminal
            .draw(|f| {
                draw_frame(&setup.app_data, colors, &keymap, f, &fd, &setup.gui_state);
            })
            .unwrap();

        for (row_index, result_row) in get_result(&setup, w) {
            let expected_row = expected_to_vec(&expected, row_index);
            for (result_cell_index, result_cell) in result_row.iter().enumerate() {
                assert_eq!(result_cell.symbol(), expected_row[result_cell_index]);
            }
        }
    }

    #[test]
    #[allow(clippy::too_many_lines)]
    /// Check that the whole layout is drawn correctly
    fn test_draw_blocks_whole_layout_with_filter() {
        let (w, h) = (160, 30);
        let mut setup = test_setup(w, h, true, true);
        insert_chart_data(&setup);
        insert_logs(&setup);

        setup.app_data.lock().containers.items[1]
            .ports
            .push(ContainerPorts {
                ip: Some(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1))),
                private: 8003,
                public: Some(8003),
            });

        let expected = [
            "    name          state       status      cpu      memory/limit          id         image     ↓ rx      ↑ tx                                  ( h ) show help   ",
            "╭ Containers 1/3 ──────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────╮╭──────────────╮",
            "│⚪  container_1   ✓ running   Up 1 hour   03.00%   30.00 kB / 30.00 kB          1   image_1   0.00 kB   0.00 kB                                ││▶ pause       │",
            "│   container_2   ✓ running   Up 2 hour   00.00%    0.00 kB /  0.00 kB          2   image_2   0.00 kB   0.00 kB                                ││  restart     │",
            "│   container_3   ✓ running   Up 3 hour   00.00%    0.00 kB /  0.00 kB          3   image_3   0.00 kB   0.00 kB                                ││  stop        │",
            "│                                                                                                                                              ││  delete      │",
            "│                                                                                                                                              ││              │",
            "│                                                                                                                                              ││              │",
            "╰──────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────╯╰──────────────╯",
            "╭ Logs 3/3 - container_1 - image_1 ────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────╮",
            "│  line 1                                                                                                                                                      │",
            "│  line 2                                                                                                                                                      │",
            "│▶ line 3                                                                                                                                                      │",
            "│                                                                                                                                                              │",
            "│                                                                                                                                                              │",
            "│                                                                                                                                                              │",
            "│                                                                                                                                                              │",
            "│                                                                                                                                                              │",
            "│                                                                                                                                                              │",
            "│                                                                                                                                                              │",
            "│                                                                                                                                                              │",
            "│                                                                                                                                                              │",
            "│                                                                                                                                                              │",
            "╰──────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────╯",
            "╭───────────────────────── cpu 03.00% ──────────────────────────╮╭─────────────────────── memory 30.00 kB ───────────────────────╮╭────────── ports ───────────╮",
            "│10.00%│     ••••                                               ││100.00 kB│     •••                                             ││       ip   private   public│",
            "│      │  •••   •                                               ││         │  •••  •                                             ││               8001         │",
            "│      │••       •••                                            ││         │••      •••                                          ││                            │",
            "│      │                                                        ││         │                                                     ││                            │",
            "╰───────────────────────────────────────────────────────────────╯╰───────────────────────────────────────────────────────────────╯╰────────────────────────────╯",
                ];
        let fd = FrameData::from((&setup.app_data, &setup.gui_state));
        let colors = setup.app_data.lock().config.app_colors;
        let keymap = setup.app_data.lock().config.keymap.clone();
        setup
            .terminal
            .draw(|f| {
                draw_frame(&setup.app_data, colors, &keymap, f, &fd, &setup.gui_state);
            })
            .unwrap();

        for (row_index, result_row) in get_result(&setup, w) {
            let expected_row = expected_to_vec(&expected, row_index);
            for (result_cell_index, result_cell) in result_row.iter().enumerate() {
                assert_eq!(result_cell.symbol(), expected_row[result_cell_index]);
            }
        }

        setup
            .gui_state
            .lock()
            .status_push(crate::ui::Status::Filter);
        setup.app_data.lock().filter_term_push('r');
        setup.app_data.lock().filter_term_push('_');
        setup.app_data.lock().filter_term_push('1');

        let expected = [
            "    name          state       status      cpu      memory/limit          id         image     ↓ rx      ↑ tx                                  ( h ) show help   ",
            "╭ Containers 1/1 - filtered ───────────────────────────────────────────────────────────────────────────────────────────────────────────────────╮╭──────────────╮",
            "│⚪  container_1   ✓ running   Up 1 hour   03.00%   30.00 kB / 30.00 kB          1   image_1   0.00 kB   0.00 kB                                ││▶ pause       │",
            "│                                                                                                                                              ││  restart     │",
            "│                                                                                                                                              ││  stop        │",
            "│                                                                                                                                              ││  delete      │",
            "╰──────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────╯╰──────────────╯",
            "╭ Logs 3/3 - container_1 - image_1 ────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────╮",
            "│  line 1                                                                                                                                                      │",
            "│  line 2                                                                                                                                                      │",
            "│▶ line 3                                                                                                                                                      │",
            "│                                                                                                                                                              │",
            "│                                                                                                                                                              │",
            "│                                                                                                                                                              │",
            "│                                                                                                                                                              │",
            "│                                                                                                                                                              │",
            "│                                                                                                                                                              │",
            "│                                                                                                                                                              │",
            "│                                                                                                                                                              │",
            "│                                                                                                                                                              │",
            "│                                                                                                                                                              │",
            "╰──────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────╯",
            "╭───────────────────────── cpu 03.00% ──────────────────────────╮╭─────────────────────── memory 30.00 kB ───────────────────────╮╭────────── ports ───────────╮",
            "│10.00%│      •••                                               ││100.00 kB│      ••                                             ││       ip   private   public│",
            "│      │    ••  •                                               ││         │    •• •                                             ││               8001         │",
            "│      │ •••     • •                                            ││         │ •••    • •                                          ││                            │",
            "│      │•        ••                                             ││         │•       ••                                           ││                            │",
            "│      │                                                        ││         │                                                     ││                            │",
            "╰───────────────────────────────────────────────────────────────╯╰───────────────────────────────────────────────────────────────╯╰────────────────────────────╯",
            " Esc  clear  ← by →   Name  Image  Status  All  term: r_1                                                                                                       ",
            ];
        let fd = FrameData::from((&setup.app_data, &setup.gui_state));
        setup
            .terminal
            .draw(|f| {
                draw_frame(&setup.app_data, colors, &keymap, f, &fd, &setup.gui_state);
            })
            .unwrap();

        for (row_index, result_row) in get_result(&setup, w) {
            let expected_row = expected_to_vec(&expected, row_index);
            for (result_cell_index, result_cell) in result_row.iter().enumerate() {
                assert_eq!(result_cell.symbol(), expected_row[result_cell_index]);
            }
        }
    }

    #[test]
    /// Check that the whole layout is drawn correctly when have long container name and long image name
    fn test_draw_blocks_whole_layout_long_name() {
        let (w, h) = (190, 30);
        let mut setup = test_setup(w, h, true, true);

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

        let expected = [
            "    name                             state       status      cpu      memory/limit          id         image                            ↓ rx      ↑ tx                      ( h ) show help   ",
            "╭ Containers 1/3 ─────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────╮╭─────────────────╮",
            "│⚪  a_long_container_name_for_the…   ✓ running   Up 1 hour   03.00%   30.00 kB / 30.00 kB          1   a_long_image_name_for_the_pur…   0.00 kB   0.00 kB                 ││▶ pause          │",
            "│   container_2                      ✓ running   Up 2 hour   00.00%    0.00 kB /  0.00 kB          2   image_2                          0.00 kB   0.00 kB                 ││  restart        │",
            "│   container_3                      ✓ running   Up 3 hour   00.00%    0.00 kB /  0.00 kB          3   image_3                          0.00 kB   0.00 kB                 ││  stop           │",
            "│                                                                                                                                                                         ││  delete         │",
            "│                                                                                                                                                                         ││                 │",
            "│                                                                                                                                                                         ││                 │",
            "╰─────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────╯╰─────────────────╯",
            "╭ Logs 3/3 - a_long_container_name_for_the_purposes_of_this_test - a_long_image_name_for_the_purposes_of_this_test ──────────────────────────────────────────────────────────────────────────╮",
            "│  line 1                                                                                                                                                                                    │",
            "│  line 2                                                                                                                                                                                    │",
            "│▶ line 3                                                                                                                                                                                    │",
            "│                                                                                                                                                                                            │",
            "│                                                                                                                                                                                            │",
            "│                                                                                                                                                                                            │",
            "│                                                                                                                                                                                            │",
            "│                                                                                                                                                                                            │",
            "│                                                                                                                                                                                            │",
            "│                                                                                                                                                                                            │",
            "│                                                                                                                                                                                            │",
            "│                                                                                                                                                                                            │",
            "│                                                                                                                                                                                            │",
            "╰────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────╯",
            "╭───────────────────────────────── cpu 03.00% ─────────────────────────────────╮╭────────────────────────────── memory 30.00 kB ───────────────────────────────╮╭────────── ports ───────────╮",
            "│10.00%│       ••••                                                            ││100.00 kB│      •••••                                                         ││       ip   private   public│",
            "│      │   ••••   •                                                            ││         │   •••    •                                                         ││               8001         │",
            "│      │•••        ••••                                                        ││         │•••        •••                                                      ││127.0.0.1      8003     8003│",
            "│      │                                                                       ││         │                                                                    ││                            │",
            "╰──────────────────────────────────────────────────────────────────────────────╯╰──────────────────────────────────────────────────────────────────────────────╯╰────────────────────────────╯",
        ];
        let fd = FrameData::from((&setup.app_data, &setup.gui_state));
        let colors = setup.app_data.lock().config.app_colors;
        let keymap = setup.app_data.lock().config.keymap.clone();
        setup
            .terminal
            .draw(|f| {
                draw_frame(&setup.app_data, colors, &keymap, f, &fd, &setup.gui_state);
            })
            .unwrap();

        for (row_index, result_row) in get_result(&setup, w) {
            let expected_row = expected_to_vec(&expected, row_index);
            for (result_cell_index, result_cell) in result_row.iter().enumerate() {
                assert_eq!(result_cell.symbol(), expected_row[result_cell_index]);
            }
        }
    }
}
