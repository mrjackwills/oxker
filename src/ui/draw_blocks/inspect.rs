use std::sync::Arc;

use parking_lot::Mutex;
use ratatui::{
    Frame,
    layout::Rect,
    style::{Style, Stylize},
    text::Line,
    widgets::{Block, Borders, Paragraph, Wrap},
};

use crate::{
    app_data::InspectData,
    config::{AppColors, Keymap},
    ui::{
        GuiState,
        draw_blocks::{DOWN_ARROW, LEFT_ARROW, RIGHT_ARROW, UP_ARROW},
        gui_state::ScrollOffset,
    },
};

/// Create a bordered block with a title.
fn title_block<'a>(upper_title: &'a str, lower_title: &'a str, colors: &AppColors) -> Block<'a> {
    Block::default()
        .borders(Borders::all())
        .border_type(ratatui::widgets::BorderType::Rounded)
        .border_style(Style::default().fg(colors.borders.selected))
        .title(upper_title.bold().into_centered_line())
        .title_bottom(lower_title.bold().into_centered_line())
}

/// Create the upper title, with container name, id, and keymap to clear
fn generate_upper_title(data: &InspectData, keymap: &Keymap) -> String {
    let mut output = String::from(" inspecting: ");
    let name = if data.name.starts_with("/") {
        data.name.replacen('/', "", 1)
    } else {
        data.name.clone()
    };

    output.push_str(&format!("{} {} ", name, data.id.get_short()));
    let mut inspect_key = keymap.inspect.0.to_string();
    if let Some(x) = keymap.inspect.1 {
        inspect_key.push_str(&format!(" or {x}"));
    }
    let mut clear_key = keymap.clear.0.to_string();
    if let Some(x) = keymap.clear.1 {
        clear_key.push_str(&format!(" or {x}"));
    }
    output.push_str(&format!(" - {clear_key} or {inspect_key} to exit"));
    output.push(' ');
    output
}

/// Generate the lower title, with the current scroll and the scrolling limits
fn generate_lower_title(length: usize, width: usize, offset: ScrollOffset) -> String {
    let length_width = length
        .to_string()
        .chars()
        .count()
        .max(offset.y.to_string().chars().count());
    let width_width = width
        .to_string()
        .chars()
        .count()
        .max(offset.x.to_string().chars().count());

    let left_arrow = if offset.x == 0 { " " } else { LEFT_ARROW };
    let right_arrow = if offset.x == width { " " } else { RIGHT_ARROW };
    let up_arrow = if offset.y == 0 { " " } else { UP_ARROW };
    let down_arrow = if offset.y == length { " " } else { DOWN_ARROW };

    format!(
        " {up_arrow} {:>length_width$}/{:>length_width$} {down_arrow}  {left_arrow} {:>width_width$}/{:>width_width$} {right_arrow} ",
        offset.y, length, offset.x, width
    )
}

/// Generate the Lines, remove lines & chars based on the offset and viewport
fn gen_lines<'a>(data_as_str: &'a str, offset: &ScrollOffset, rect: &Rect) -> Vec<Line<'a>> {
    let first_line_index = offset.y.max(0);
    let first_char_index = offset.x.max(0);
    let last_char_index = usize::from(rect.width.saturating_sub(2));
    let take_lines = usize::from(rect.height);
    //todo see ig log scrolling does this

    data_as_str
        .lines()
        .skip(first_line_index)
        .take(take_lines)
        .map(|line| {
            Line::from(
                line.chars()
                    .skip(first_char_index)
                    .take(last_char_index)
                    .collect::<String>(),
            )
        })
        .collect()
}

// TODO refactor h/w into struct - is it used elsewhere?

/// Draw the InspectContainer widget to the entire screen
pub fn draw(
    f: &mut Frame,
    colors: AppColors,
    data: InspectData,
    gui_state: &Arc<Mutex<GuiState>>,
    keymap: &Keymap,
) {
    let rect = f.area();
    let offset = gui_state.lock().get_inspect_offset();
    // +2 to account for the border
    let height = data
        .height
        .saturating_sub(usize::from(rect.height))
        .saturating_add(2);
    let width = data
        .width
        .saturating_sub(usize::from(rect.width))
        .saturating_add(2);
    let upper_title = generate_upper_title(&data, keymap);
    let lower_title = generate_lower_title(height, width, offset);

    gui_state.lock().set_inspect_offset_max(ScrollOffset {
        x: width,
        y: height,
    });

    let paragraph = Paragraph::new(gen_lines(&data.as_string, &offset, &rect))
        .block(title_block(&upper_title, &lower_title, &colors))
        .gray()
        .left_aligned()
        .wrap(Wrap { trim: false });
    f.render_widget(paragraph, rect);
}

// TODO TESTS
// Test keymap
// Test colors
// Test offset y & x

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use std::{collections::HashMap, sync::LazyLock};

    use crate::{
        app_data::InspectData,
        config::{AppColors, Keymap},
        ui::draw_blocks::tests::{get_result, test_setup},
    };
    use bollard::secret::{
        ContainerConfig, ContainerInspectResponse, ContainerState, ContainerStateStatusEnum,
        DriverData, EndpointSettings, HostConfig, HostConfigLogConfig, MountPoint,
        MountPointTypeEnum, NetworkSettings, RestartPolicy, RestartPolicyNameEnum,
    };
    use crossterm::event::KeyCode;
    use insta::assert_snapshot;
    use ratatui::style::Color;

    static INSPECT_DATA: LazyLock<InspectData> =
        LazyLock::new(|| InspectData::from(gen_container_inspect_response()));

    #[test]
    /// Test a inspect container with default settings, keymap, and position
    fn test_draw_blocks_inspect_default_valid() {
        let mut setup = test_setup(100, 50, true, true);
        setup
            .terminal
            .draw(|f| {
                super::draw(
                    f,
                    AppColors::new(),
                    INSPECT_DATA.clone(),
                    &setup.gui_state,
                    &Keymap::new(),
                );
            })
            .unwrap();
        assert_snapshot!(setup.terminal.backend());

        // Assert border colors
        for (row_index, result_row) in get_result(&setup) {
            for (result_cell_index, result_cell) in result_row.iter().enumerate() {
                match (row_index, result_cell_index) {
                    (0 | 49, _) | (_, 0 | 99) => {
                        assert_eq!(result_cell.bg, Color::Reset);
                        assert_eq!(result_cell.fg, Color::LightCyan);
                    }
                    _ => {
                        assert_eq!(result_cell.bg, Color::Reset);
                        assert_eq!(result_cell.fg, Color::Gray);
                    }
                }
            }
        }
    }

    #[test]
    /// Test a inspect container with custom colors
    fn test_draw_blocks_inspect_custom_color() {
        let mut setup = test_setup(100, 50, true, true);

        let mut colors = AppColors::new();
        colors.borders.selected = Color::Red;
        setup
            .terminal
            .draw(|f| {
                super::draw(
                    f,
                    colors,
                    INSPECT_DATA.clone(),
                    &setup.gui_state,
                    &Keymap::new(),
                );
            })
            .unwrap();
        assert_snapshot!(setup.terminal.backend());

        // Assert custom border colors
        for (row_index, result_row) in get_result(&setup) {
            for (result_cell_index, result_cell) in result_row.iter().enumerate() {
                match (row_index, result_cell_index) {
                    (0 | 49, _) | (_, 0 | 99) => {
                        assert_eq!(result_cell.bg, Color::Reset);
                        assert_eq!(result_cell.fg, Color::Red);
                    }
                    _ => {
                        assert_eq!(result_cell.bg, Color::Reset);
                        assert_eq!(result_cell.fg, Color::Gray);
                    }
                }
            }
        }
    }

    #[test]
    /// Test a inspect container with custom keymap for one clear key
    fn test_draw_blocks_inspect_custom_keymap_clear_one() {
        let mut setup = test_setup(100, 50, true, true);

        let mut keymap = Keymap::new();

        keymap.clear.0 = KeyCode::Char('F');

        setup
            .terminal
            .draw(|f| {
                super::draw(
                    f,
                    AppColors::new(),
                    INSPECT_DATA.clone(),
                    &setup.gui_state,
                    &keymap,
                );
            })
            .unwrap();
        assert_snapshot!(setup.terminal.backend());

        // Assert border colors
        for (row_index, result_row) in get_result(&setup) {
            for (result_cell_index, result_cell) in result_row.iter().enumerate() {
                match (row_index, result_cell_index) {
                    (0 | 49, _) | (_, 0 | 99) => {
                        assert_eq!(result_cell.bg, Color::Reset);
                        assert_eq!(result_cell.fg, Color::LightCyan);
                    }
                    _ => {
                        assert_eq!(result_cell.bg, Color::Reset);
                        assert_eq!(result_cell.fg, Color::Gray);
                    }
                }
            }
        }
    }

    #[test]
    /// Test a inspect container with custom keymap for both clear keys
    fn test_draw_blocks_inspect_custom_keymap_clear_two() {
        let mut setup = test_setup(100, 50, true, true);

        let mut keymap = Keymap::new();

        keymap.clear.0 = KeyCode::Char('F');
        keymap.clear.1 = Some(KeyCode::Char('Z'));

        setup
            .terminal
            .draw(|f| {
                super::draw(
                    f,
                    AppColors::new(),
                    INSPECT_DATA.clone(),
                    &setup.gui_state,
                    &keymap,
                );
            })
            .unwrap();
        assert_snapshot!(setup.terminal.backend());

        // Assert border colors
        for (row_index, result_row) in get_result(&setup) {
            for (result_cell_index, result_cell) in result_row.iter().enumerate() {
                match (row_index, result_cell_index) {
                    (0 | 49, _) | (_, 0 | 99) => {
                        assert_eq!(result_cell.bg, Color::Reset);
                        assert_eq!(result_cell.fg, Color::LightCyan);
                    }
                    _ => {
                        assert_eq!(result_cell.bg, Color::Reset);
                        assert_eq!(result_cell.fg, Color::Gray);
                    }
                }
            }
        }
    }

    #[test]
    /// Test a inspect container with custom keymap for one inspect key
    fn test_draw_blocks_inspect_custom_keymap_inspect_one() {
        let mut setup = test_setup(100, 50, true, true);

        let mut keymap = Keymap::new();

        keymap.inspect.0 = KeyCode::Char('4');

        setup
            .terminal
            .draw(|f| {
                super::draw(
                    f,
                    AppColors::new(),
                    INSPECT_DATA.clone(),
                    &setup.gui_state,
                    &keymap,
                );
            })
            .unwrap();
        assert_snapshot!(setup.terminal.backend());

        // Assert border colors
        for (row_index, result_row) in get_result(&setup) {
            for (result_cell_index, result_cell) in result_row.iter().enumerate() {
                match (row_index, result_cell_index) {
                    (0 | 49, _) | (_, 0 | 99) => {
                        assert_eq!(result_cell.bg, Color::Reset);
                        assert_eq!(result_cell.fg, Color::LightCyan);
                    }
                    _ => {
                        assert_eq!(result_cell.bg, Color::Reset);
                        assert_eq!(result_cell.fg, Color::Gray);
                    }
                }
            }
        }
    }

    #[test]
    /// Test a inspect container with custom keymap for both inspect keys
    fn test_draw_blocks_inspect_custom_keymap_inspect_two() {
        let mut setup = test_setup(100, 50, true, true);

        let mut keymap = Keymap::new();

        keymap.inspect.0 = KeyCode::Char('4');
        keymap.inspect.1 = Some(KeyCode::Char('5'));

        setup
            .terminal
            .draw(|f| {
                super::draw(
                    f,
                    AppColors::new(),
                    INSPECT_DATA.clone(),
                    &setup.gui_state,
                    &keymap,
                );
            })
            .unwrap();
        assert_snapshot!(setup.terminal.backend());

        // Assert border colors
        for (row_index, result_row) in get_result(&setup) {
            for (result_cell_index, result_cell) in result_row.iter().enumerate() {
                match (row_index, result_cell_index) {
                    (0 | 49, _) | (_, 0 | 99) => {
                        assert_eq!(result_cell.bg, Color::Reset);
                        assert_eq!(result_cell.fg, Color::LightCyan);
                    }
                    _ => {
                        assert_eq!(result_cell.bg, Color::Reset);
                        assert_eq!(result_cell.fg, Color::Gray);
                    }
                }
            }
        }
    }

    #[test]
    /// Test a inspect container with all custom keymaps
    fn test_draw_blocks_inspect_custom_keymap_all() {
        let mut setup = test_setup(100, 50, true, true);

        let mut keymap = Keymap::new();

        keymap.clear.0 = KeyCode::Char('F');
        keymap.clear.1 = Some(KeyCode::Char('Z'));
        keymap.inspect.0 = KeyCode::Char('4');
        keymap.inspect.1 = Some(KeyCode::Char('5'));

        setup
            .terminal
            .draw(|f| {
                super::draw(
                    f,
                    AppColors::new(),
                    INSPECT_DATA.clone(),
                    &setup.gui_state,
                    &keymap,
                );
            })
            .unwrap();
        assert_snapshot!(setup.terminal.backend());

        // Assert border colors
        for (row_index, result_row) in get_result(&setup) {
            for (result_cell_index, result_cell) in result_row.iter().enumerate() {
                match (row_index, result_cell_index) {
                    (0 | 49, _) | (_, 0 | 99) => {
                        assert_eq!(result_cell.bg, Color::Reset);
                        assert_eq!(result_cell.fg, Color::LightCyan);
                    }
                    _ => {
                        assert_eq!(result_cell.bg, Color::Reset);
                        assert_eq!(result_cell.fg, Color::Gray);
                    }
                }
            }
        }
    }

    #[test]
    /// Inspect details are offset 10 in x and y axis
    fn test_draw_blocks_inspect_offset() {
        let mut setup = test_setup(100, 50, true, true);

        // Why does one need to draw first, although it *should* be impossible to scroll before an inital drawing
        setup
            .terminal
            .draw(|f| {
                super::draw(
                    f,
                    AppColors::new(),
                    INSPECT_DATA.clone(),
                    &setup.gui_state,
                    &Keymap::new(),
                );
            })
            .unwrap();

        {
            let mut gui_state = setup.gui_state.lock();
            for _ in 0..=9 {
                gui_state.set_inspect_offset(&crate::app_data::ScrollDirection::Down);
                gui_state.set_inspect_offset(&crate::app_data::ScrollDirection::Right);
            }
        }
        setup
            .terminal
            .draw(|f| {
                super::draw(
                    f,
                    AppColors::new(),
                    INSPECT_DATA.clone(),
                    &setup.gui_state,
                    &Keymap::new(),
                );
            })
            .unwrap();
        assert_snapshot!(setup.terminal.backend());

        for (row_index, result_row) in get_result(&setup) {
            for (result_cell_index, result_cell) in result_row.iter().enumerate() {
                match (row_index, result_cell_index) {
                    (0 | 49, _) | (_, 0 | 99) => {
                        assert_eq!(result_cell.bg, Color::Reset);
                        assert_eq!(result_cell.fg, Color::LightCyan);
                    }
                    _ => {
                        assert_eq!(result_cell.bg, Color::Reset);
                        assert_eq!(result_cell.fg, Color::Gray);
                    }
                }
            }
        }
    }

    #[test]
    /// Inspect details are offset to the maximum allowed
    fn test_draw_blocks_inspect_offset_max() {
        let mut setup = test_setup(100, 50, true, true);

        // Why does one need to draw first, although it *should* be impossible to scroll before an inital drawing
        setup
            .terminal
            .draw(|f| {
                super::draw(
                    f,
                    AppColors::new(),
                    INSPECT_DATA.clone(),
                    &setup.gui_state,
                    &Keymap::new(),
                );
            })
            .unwrap();

        // Lazy way of getting the max offset
        {
            let mut gui_state = setup.gui_state.lock();
            for _ in 0..=1000 {
                gui_state.set_inspect_offset(&crate::app_data::ScrollDirection::Down);
                gui_state.set_inspect_offset(&crate::app_data::ScrollDirection::Right);
            }
        }
        setup
            .terminal
            .draw(|f| {
                super::draw(
                    f,
                    AppColors::new(),
                    INSPECT_DATA.clone(),
                    &setup.gui_state,
                    &Keymap::new(),
                );
            })
            .unwrap();
        assert_snapshot!(setup.terminal.backend());

        for (row_index, result_row) in get_result(&setup) {
            for (result_cell_index, result_cell) in result_row.iter().enumerate() {
                match (row_index, result_cell_index) {
                    (0 | 49, _) | (_, 0 | 99) => {
                        assert_eq!(result_cell.bg, Color::Reset);
                        assert_eq!(result_cell.fg, Color::LightCyan);
                    }
                    _ => {
                        assert_eq!(result_cell.bg, Color::Reset);
                        assert_eq!(result_cell.fg, Color::Gray);
                    }
                }
            }
        }
    }

    fn gen_container_inspect_response() -> ContainerInspectResponse {
        ContainerInspectResponse {
    id: Some("0bdea64212f9c75eb4a1184dd406c2c79a986a7a889a23c85358456cc1bb60c7".to_owned()),
    created: Some("2026-01-23T22:20:19.927967311Z".to_owned()),
    path: Some("docker-entrypoint.sh".to_owned()),
    args: Some(vec!["postgres".to_owned()]),
    state: Some(ContainerState {
        status: Some(ContainerStateStatusEnum::RUNNING),
        running: Some(true),
        paused: Some(false),
        restarting: Some(false),
        oom_killed: Some(false),
        dead: Some(false),
        pid: Some(782),
        exit_code: Some(0),
        error: Some("".to_owned()),
        started_at: Some("2026-01-30T08:09:01.574885915Z".to_owned()),
        finished_at: Some("2026-01-30T08:09:01.180567927Z".to_owned()),
        health: None,
    }),
    image: Some("sha256:aa3668fcbcb5ded731b7d5c27065a4edf545debb7f27bf514c709b1b4e032352".to_owned()),
    resolv_conf_path: Some("/var/lib/docker/containers/0bdea64212f9c75eb4a1184dd406c2c79a986a7a889a23c85358456cc1bb60c7/resolv.conf".to_owned()),
    hostname_path: Some("/var/lib/docker/containers/0bdea64212f9c75eb4a1184dd406c2c79a986a7a889a23c85358456cc1bb60c7/hostname".to_owned()),
    hosts_path: Some("/var/lib/docker/containers/0bdea64212f9c75eb4a1184dd406c2c79a986a7a889a23c85358456cc1bb60c7/hosts".to_owned()),
    log_path: Some("/var/lib/docker/containers/0bdea64212f9c75eb4a1184dd406c2c79a986a7a889a23c85358456cc1bb60c7/0bdea64212f9c75eb4a1184dd406c2c79a986a7a889a23c85358456cc1bb60c7-json.log".to_owned()),
    name: Some("/postgres".to_owned()),
    restart_count: Some(0),
    driver: Some("overlay2".to_owned()),
    platform: Some("linux".to_owned()),
    image_manifest_descriptor: None,
    mount_label: Some("".to_owned()),
    process_label: Some("".to_owned()),
    app_armor_profile: Some("".to_owned()),
    exec_ids: None,
    host_config: Some(HostConfig {
        cpu_shares: Some(0),
        memory: Some(1073741824),
        cgroup_parent: Some("".to_owned()),
        blkio_weight: Some(0),
        blkio_weight_device: None,
        blkio_device_read_bps: None,
        blkio_device_write_bps: None,
        blkio_device_read_iops: None,
        blkio_device_write_iops: None,
        cpu_period: Some(0),
        cpu_quota: Some(0),
        cpu_realtime_period: Some(0),
        cpu_realtime_runtime: Some(0),
        cpuset_cpus: Some("".to_owned()),
        cpuset_mems: Some("".to_owned()),
        devices: None,
        device_cgroup_rules: None,
        device_requests: None,
        memory_reservation: Some(0),
        memory_swap: Some(2147483648),
        memory_swappiness: None,
        nano_cpus: Some(0),
        oom_kill_disable: Some(false),
        init: None,
        pids_limit: None,
        ulimits: None,
        cpu_count: Some(0),
        cpu_percent: Some(0),
        io_maximum_iops: Some(0),
        io_maximum_bandwidth: Some(0),
        binds: None,
        container_id_file: Some("".to_owned()),
        log_config: Some(HostConfigLogConfig {
            typ: Some("json-file".to_owned()),
            config: Some(HashMap::new()),
        }),
        network_mode: Some("oxker-examaple-net".to_owned()),
        port_bindings: Some(HashMap::new()),
        restart_policy: Some(RestartPolicy {
            name: Some(RestartPolicyNameEnum::ALWAYS),
            maximum_retry_count: Some(0),
        }),
        auto_remove: Some(false),
        volume_driver: Some("".to_owned()),
        volumes_from: None,
        mounts: None,
        console_size: Some(vec![0, 0]),
        annotations: None,
        cap_add: None,
        cap_drop: None,
        cgroupns_mode: Some(bollard::secret::HostConfigCgroupnsModeEnum::HOST),
        dns: Some(vec![]),
        dns_options: Some(vec![]),
        dns_search: Some(vec![]),
        extra_hosts: Some(vec![]),
        group_add: None,
        ipc_mode: Some("private".to_owned()),
        cgroup: Some("".to_owned()),
        links: None,
        oom_score_adj: Some(0),
        pid_mode: Some("".to_owned()),
        privileged: Some(false),
        publish_all_ports: Some(false),
        readonly_rootfs: Some(false),
        security_opt: None,
        storage_opt: None,
        tmpfs: None,
        uts_mode: Some("".to_owned()),
        userns_mode: Some("".to_owned()),
        shm_size: Some(268435456),
        sysctls: None,
        runtime: Some("runc".to_owned()),
        isolation: Some(bollard::secret::HostConfigIsolationEnum::EMPTY),
        masked_paths: Some(vec![
            "/proc/acpi".to_owned(),
            "/proc/asound".to_owned(),
            "/proc/interrupts".to_owned(),
            "/proc/kcore".to_owned(),
            "/proc/keys".to_owned(),
            "/proc/latency_stats".to_owned(),
            "/proc/sched_debug".to_owned(),
            "/proc/scsi".to_owned(),
            "/proc/timer_list".to_owned(),
            "/proc/timer_stats".to_owned(),
            "/sys/devices/virtual/powercap".to_owned(),
            "/sys/firmware".to_owned(),
        ]),
        readonly_paths: Some(vec![
            "/proc/bus".to_owned(),
            "/proc/fs".to_owned(),
            "/proc/irq".to_owned(),
            "/proc/sys".to_owned(),
            "/proc/sysrq-trigger".to_owned(),
        ]),
    }),
    graph_driver: Some(DriverData {
        name: "overlay2".to_owned(),
        data: HashMap::from([
            ("LowerDir".to_owned(), "/var/lib/docker/overlay2/b8dae7c82251b8dadc084dbcaceec47b3d48a5ba9d055a59934a8b88d18569ea-init/diff:/var/lib/docker/overlay2/51b93846f7ba3e00cb1ed86564e3e1d7c30df2bb1cd5a8469d54625f1e5a2eca/diff:/var/lib/docker/overlay2/c1364ead843d3af87ce286013b6301329d3089422b22b001e156e45d29b5b4dd/diff:/var/lib/docker/overlay2/0e6dc322cad77b1db3906a3a4e5e6d6b80fbffd138437e550d8849fcf4f4c1f2/diff:/var/lib/docker/overlay2/cc0f967a7471cf06e0c9ad3d474650c668a4cf0c02efe20e9c250c436f93033b/diff:/var/lib/docker/overlay2/5c59e0919969987c96a5d0e0a512a0a1a0f67ea747596af9a9c14a9566198d91/diff:/var/lib/docker/overlay2/d7709b7685c9704e1e392c515b6155517270541f6ccde426ef784403e1681fca/diff:/var/lib/docker/overlay2/c891528563fff91bffaf07416e77bcd3bdebb03e5d32ed0e3d4ee1ec5e80e880/diff:/var/lib/docker/overlay2/2b25c179a432c35cc599a082cd709c8c9a1523f8d1959f72fda21fc76e50ad00/diff:/var/lib/docker/overlay2/3b409d2f7a2455578148892302823a7f03c7c36482d08bb68fd6c1aeeec05f05/diff:/var/lib/docker/overlay2/55dbb2fab0ae8bb3bfe8183093cdd576686f7333e2b2c41e6e4178a7b6407554/diff".to_owned()),
            ("MergedDir".to_owned(), "/var/lib/docker/overlay2/b8dae7c82251b8dadc084dbcaceec47b3d48a5ba9d055a59934a8b88d18569ea/merged".to_owned()),
            ("WorkDir".to_owned(), "/var/lib/docker/overlay2/b8dae7c82251b8dadc084dbcaceec47b3d48a5ba9d055a59934a8b88d18569ea/work".to_owned()),
            ("ID".to_owned(), "0bdea64212f9c75eb4a1184dd406c2c79a986a7a889a23c85358456cc1bb60c7".to_owned()),
            ("UpperDir".to_owned(), "/var/lib/docker/overlay2/b8dae7c82251b8dadc084dbcaceec47b3d48a5ba9d055a59934a8b88d18569ea/diff".to_owned()),
        ]),
    }),
    storage: None,
    size_rw: None,
    size_root_fs: None,
    mounts: Some(vec![MountPoint {
        typ: Some(MountPointTypeEnum::VOLUME),
        name: Some("93bc4e4c8d3823964b58105a99a7b3a7e02c801d5560338bdaf7589966a1b02d".to_owned()),
        source: Some("/var/lib/docker/volumes/93bc4e4c8d3823964b58105a99a7b3a7e02c801d5560338bdaf7589966a1b02d/_data".to_owned()),
        destination: Some("/var/lib/postgresql/data".to_owned()),
        driver: Some("local".to_owned()),
        mode: Some("".to_owned()),
        rw: Some(true),
        propagation: Some("".to_owned()),
    }]),
    config: Some(ContainerConfig {
        hostname: Some("0bdea64212f9".to_owned()),
        domainname: Some("".to_owned()),
        user: Some("".to_owned()),
        attach_stdin: Some(false),
        attach_stdout: Some(true),
        attach_stderr: Some(true),
        exposed_ports: Some(vec!["5432/tcp".to_owned()]),
        tty: Some(false),
        open_stdin: Some(false),
        stdin_once: Some(false),
        env: Some(vec![
            "POSTGRES_PASSWORD=never_use_this_password_in_production".to_owned(),
            "PATH=/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin".to_owned(),
            "GOSU_VERSION=1.19".to_owned(),
            "LANG=en_US.utf8".to_owned(),
            "PG_MAJOR=17".to_owned(),
            "PG_VERSION=17.7".to_owned(),
            "PG_SHA256=ef9e343302eccd33112f1b2f0247be493cb5768313adeb558b02de8797a2e9b5".to_owned(),
            "DOCKER_PG_LLVM_DEPS=llvm19-dev \t\tclang19".to_owned(),
            "PGDATA=/var/lib/postgresql/data".to_owned(),
        ]),
        cmd: Some(vec!["postgres".to_owned()]),
        healthcheck: None,
        args_escaped: None,
        image: Some("postgres:17-alpine".to_owned()),
        volumes: Some(vec!["/var/lib/postgresql/data".to_owned()]),
        working_dir: Some("/".to_owned()),
        entrypoint: Some(vec!["docker-entrypoint.sh".to_owned()]),
        network_disabled: None,
        on_build: None,
        labels: Some(HashMap::from([
            ("com.docker.compose.oneoff".to_owned(), "False".to_owned()),
            ("com.docker.compose.project.config_files".to_owned(), "/workspaces/oxker/docker/docker-compose.yml".to_owned()),
            ("com.docker.compose.image".to_owned(), "sha256:aa3668fcbcb5ded731b7d5c27065a4edf545debb7f27bf514c709b1b4e032352".to_owned()),
            ("com.docker.compose.project.working_dir".to_owned(), "/workspaces/oxker/docker".to_owned()),
            ("com.docker.compose.service".to_owned(), "postgres".to_owned()),
            ("com.docker.compose.config-hash".to_owned(), "e06d69ffb3f9b69dd51b356b60c2297df57caf0da16792ccafaabffdb920e443".to_owned()),
            ("com.docker.compose.depends_on".to_owned(), "".to_owned()),
            ("com.docker.compose.container-number".to_owned(), "1".to_owned()),
            ("com.docker.compose.version".to_owned(), "2.40.3".to_owned()),
            ("com.docker.compose.project".to_owned(), "docker".to_owned()),
        ])),
        stop_signal: Some("SIGINT".to_owned()),
        stop_timeout: None,
        shell: None,
    }),
    network_settings: Some(NetworkSettings {
        sandbox_id: Some("dab64a66594dd8d06478184e2928c81acdcd9c931f643bd5ca62b7edb6345f8d".to_owned()),
        sandbox_key: Some("/var/run/docker/netns/dab64a66594d".to_owned()),
        ports: Some(HashMap::from([("5432/tcp".to_owned(), None)])),
        networks: Some(HashMap::from([(
            "oxker-examaple-net".to_owned(),
            EndpointSettings {
                ipam_config: None,
                links: None,
                mac_address: Some("a2:bd:4e:61:25:c7".to_owned()),
                aliases: Some(vec!["postgres".to_owned(), "postgres".to_owned()]),
                driver_opts: None,
                gw_priority: Some(0),
                network_id: Some("3cbeb475d81676f89a7aa205d8749ec2ad78d685e45d77b638992956f6dc569a".to_owned()),
                endpoint_id: Some("31718069b2a3ea77487f3ece36b014d5d1329bc3294568e2621e5c0999071bed".to_owned()),
                gateway: Some("172.19.0.1".to_owned()),
                ip_address: Some("172.19.0.4".to_owned()),
                ip_prefix_len: Some(16),
                ipv6_gateway: Some("".to_owned()),
                global_ipv6_address: Some("".to_owned()),
                global_ipv6_prefix_len: Some(0),
                dns_names: Some(vec!["postgres".to_owned(), "0bdea64212f9".to_owned()]),
            },
        )])),
    }),
}
    }
}
