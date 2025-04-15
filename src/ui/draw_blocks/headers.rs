use std::{rc::Rc, sync::Arc};

use parking_lot::Mutex;
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    widgets::{Block, Paragraph},
};

use super::{CONSTRAINT_100, MARGIN};
use crate::{
    app_data::{Header, SortedOrder},
    config::{AppColors, Keymap},
    ui::{FrameData, GuiState, Status, gui_state::Region},
};

/// Generate a header paragraph with it's width
fn gen_header<'a>(
    colors: AppColors,
    fd: &FrameData,
    header: Header,
    width: usize,
) -> (Paragraph<'a>, u16) {
    let block = gen_header_block(colors, fd, header);

    let text = format!(
        "{x:<width$}{MARGIN}",
        x = format!("{header}{ic}", ic = block.1),
    );
    let count = u16::try_from(text.chars().count()).unwrap_or_default();
    let status = Paragraph::new(text)
        .style(gen_style(None, block.0))
        .alignment(Alignment::Left);
    (status, count)
}

// Generate a block for the header, if the header is currently being used to sort a column, then highlight it white
fn gen_header_block<'a>(colors: AppColors, fd: &FrameData, header: Header) -> (Color, &'a str) {
    let mut color = colors.headers_bar.text;
    let mut suffix = "";
    if let Some((a, b)) = &fd.sorted_by {
        if &header == a {
            match b {
                SortedOrder::Asc => suffix = " ▲",
                SortedOrder::Desc => suffix = " ▼",
            }
            color = colors.headers_bar.text_selected;
        }
    }

    (color, suffix)
}

fn gen_style(bg: Option<Color>, fg: Color) -> Style {
    bg.map_or_else(
        || Style::default().fg(fg),
        |bg| Style::default().bg(bg).fg(fg),
    )
}

/// Generate the text to display on the show help section, as can change with a custom keymap
fn gen_help_text(fd: &FrameData, keymap: &Keymap) -> String {
    let suffix = if fd.status.contains(&Status::Help) {
        "exit"
    } else {
        "show"
    };

    if keymap.toggle_help == Keymap::new().toggle_help {
        format!("( h ) {suffix} help{MARGIN}")
    } else if let Some(secondary) = keymap.toggle_help.1 {
        format!(
            " ( {} | {secondary} ) {suffix} help{MARGIN}",
            keymap.toggle_help.0
        )
    } else {
        format!(" ( {} ) {suffix} help{MARGIN}", keymap.toggle_help.0)
    }
}

/// Draw the show/hide help section
fn draw_help(
    colors: AppColors,
    f: &mut Frame,
    fd: &FrameData,
    help_text: String,
    gui_state: &Arc<Mutex<GuiState>>,
    split_bar: &Rc<[Rect]>,
) {
    let help_text_color = if fd.status.contains(&Status::Help) {
        colors.headers_bar.text
    } else {
        colors.headers_bar.text_selected
    };

    let help_paragraph = Paragraph::new(help_text)
        .style(gen_style(None, help_text_color))
        .alignment(Alignment::Right);

    // If no containers, don't display the headers, could maybe do this first?
    let help_index = if fd.has_containers { 2 } else { 0 };
    gui_state
        .lock()
        .update_region_map(Region::HelpPanel, split_bar[help_index]);
    f.render_widget(help_paragraph, split_bar[help_index]);
}

// Draw loading icon, or not, and a prefix with a single space
fn draw_loading_spinner(colors: AppColors, f: &mut Frame, fd: &FrameData, rect: Rect) {
    let loading_paragraph = Paragraph::new(format!("{:>2}", fd.loading_icon))
        .style(gen_style(None, colors.headers_bar.loading_spinner))
        .alignment(Alignment::Left);
    f.render_widget(loading_paragraph, rect);
}

/// Draw the sortable column headers (name/state/status etc)
fn draw_columns(
    colors: AppColors,
    f: &mut Frame,
    fd: &FrameData,
    gui_state: &Arc<Mutex<GuiState>>,
    split_bar: &Rc<[Rect]>,
) {
    if fd.has_containers {
        let header_section_width = split_bar[1].width;

        let mut counter = 0;

        // Meta data to iterate over to create blocks with correct widths
        let header_meta = [
            (Header::Name, fd.columns.name.1),
            (Header::State, fd.columns.state.1),
            (Header::Status, fd.columns.status.1),
            (Header::Cpu, fd.columns.cpu.1),
            (Header::Memory, fd.columns.mem.1 + fd.columns.mem.2 + 3),
            (Header::Id, fd.columns.id.1),
            (Header::Image, fd.columns.image.1),
            (Header::Rx, fd.columns.net_rx.1),
            (Header::Tx, fd.columns.net_tx.1),
        ];

        // Only show a header if the header cumulative header width is less than the header section width
        let header_data = header_meta
            .into_iter()
            .filter_map(|(header, width)| {
                let header_block = gen_header(colors, fd, header, usize::from(width));
                counter += header_block.1;
                if counter <= header_section_width {
                    Some((header_block.0, header, Constraint::Max(header_block.1)))
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();

        let container_splits = header_data.iter().map(|i| i.2).collect::<Vec<_>>();
        let headers_section = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(container_splits)
            .split(split_bar[1]);

        for (index, (paragraph, header, _)) in header_data.into_iter().enumerate() {
            let rect = headers_section[index];
            gui_state
                .lock()
                .update_region_map(Region::Header(header), rect);
            f.render_widget(paragraph, rect);
        }
    }
}

// Draw heading bar at top of program, always visible
pub fn draw(
    area: Rect,
    colors: AppColors,
    f: &mut Frame,
    fd: &FrameData,
    gui_state: &Arc<Mutex<GuiState>>,
    keymap: &Keymap,
) {
    let gen_style = |bg: Option<Color>, fg: Color| {
        bg.map_or_else(
            || Style::default().fg(fg),
            |bg| Style::default().bg(bg).fg(fg),
        )
    };

    f.render_widget(
        Block::default().style(gen_style(Some(colors.headers_bar.background), Color::Reset)),
        area,
    );

    let help_text = gen_help_text(fd, keymap);
    let help_width = help_text.chars().count();

    let column_width = usize::from(area.width).saturating_sub(help_width);
    let column_width = if column_width > 0 { column_width } else { 1 };
    let splits = if fd.has_containers {
        vec![
            Constraint::Max(4),
            Constraint::Max(column_width.try_into().unwrap_or_default()),
            Constraint::Max(help_width.try_into().unwrap_or_default()),
        ]
    } else {
        CONSTRAINT_100.to_vec()
    };

    let split_bar = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(splits)
        .split(area);

    draw_loading_spinner(colors, f, fd, split_bar[0]);
    draw_columns(colors, f, fd, gui_state, &split_bar);
    draw_help(colors, f, fd, help_text, gui_state, &split_bar);
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use std::ops::RangeInclusive;

    use crossterm::event::KeyCode;
    use insta::assert_snapshot;
    use ratatui::style::Color;
    use uuid::Uuid;

    use crate::{
        app_data::{Header, SortedOrder, StatefulList},
        config::{AppColors, Keymap},
        ui::{
            FrameData, Status,
            draw_blocks::tests::{TuiTestSetup, get_result, test_setup},
        },
    };

    #[test]
    /// Heading back only has show/exit help when no containers, correctly coloured
    fn test_draw_blocks_headers_no_containers_show_help() {
        let mut setup = test_setup(140, 1, true, true);
        setup.app_data.lock().containers = StatefulList::new(vec![]);

        let fd = FrameData::from((&setup.app_data, &setup.gui_state));

        setup
            .terminal
            .draw(|f| {
                super::draw(
                    setup.area,
                    AppColors::new(),
                    f,
                    &fd,
                    &setup.gui_state,
                    &Keymap::new(),
                );
            })
            .unwrap();

        assert_snapshot!(setup.terminal.backend());

        for (_, result_row) in get_result(&setup) {
            for result_cell in result_row {
                assert_eq!(result_cell.bg, Color::Magenta);
                assert_eq!(result_cell.fg, Color::Gray,);
            }
        }
    }

    #[test]
    /// Heading back only has show/exit help when no containers, correctly coloured
    fn test_draw_blocks_headers_no_containers_exit_help() {
        let mut setup = test_setup(140, 1, true, true);
        setup.app_data.lock().containers = StatefulList::new(vec![]);

        let mut fd = FrameData::from((&setup.app_data, &setup.gui_state));
        fd.status.insert(Status::Help);
        setup
            .terminal
            .draw(|f| {
                super::draw(
                    setup.area,
                    AppColors::new(),
                    f,
                    &fd,
                    &setup.gui_state,
                    &Keymap::new(),
                );
            })
            .unwrap();
        assert_snapshot!(setup.terminal.backend());

        for (_, result_row) in get_result(&setup) {
            for result_cell in result_row {
                assert_eq!(result_cell.bg, Color::Magenta);
                assert_eq!(result_cell.fg, Color::Black);
            }
        }
    }

    #[test]
    /// Show all headings when containers present, colors valid
    fn test_draw_blocks_headers_some_containers() {
        let mut setup = test_setup(140, 1, true, true);
        let fd = FrameData::from((&setup.app_data, &setup.gui_state));
        setup
            .terminal
            .draw(|f| {
                super::draw(
                    setup.area,
                    AppColors::new(),
                    f,
                    &fd,
                    &setup.gui_state,
                    &Keymap::new(),
                );
            })
            .unwrap();
        assert_snapshot!(setup.terminal.backend());

        for (_, result_row) in get_result(&setup) {
            for (result_cell_index, result_cell) in result_row.iter().enumerate() {
                assert_eq!(result_cell.bg, Color::Magenta);
                assert_eq!(
                    result_cell.fg,
                    match result_cell_index {
                        0..=3 => Color::White,
                        4..=111 => Color::Black,
                        112..=121 => Color::Reset,
                        _ => Color::Gray,
                    }
                );
            }
        }
    }

    #[test]
    /// Only show the headings that fit the reduced-in-size header section
    fn test_draw_blocks_headers_some_containers_reduced_width() {
        let mut setup = test_setup(80, 1, true, true);
        let fd = FrameData::from((&setup.app_data, &setup.gui_state));

        setup
            .terminal
            .draw(|f| {
                super::draw(
                    setup.area,
                    AppColors::new(),
                    f,
                    &fd,
                    &setup.gui_state,
                    &Keymap::new(),
                );
            })
            .unwrap();

        assert_snapshot!(setup.terminal.backend());
        for (_, result_row) in get_result(&setup) {
            for (result_cell_index, result_cell) in result_row.iter().enumerate() {
                assert_eq!(result_cell.bg, Color::Magenta);
                assert_eq!(
                    result_cell.fg,
                    match result_cell_index {
                        0..=3 => Color::White,
                        4..=50 => Color::Black,
                        51..=61 => Color::Reset,
                        _ => Color::Gray,
                    }
                );
            }
        }
    }

    #[test]
    /// Show animation
    fn test_draw_blocks_headers_animation() {
        let mut setup = test_setup(140, 1, true, true);
        let uuid = Uuid::new_v4();
        setup.gui_state.lock().next_loading(uuid);
        let fd = FrameData::from((&setup.app_data, &setup.gui_state));

        setup
            .terminal
            .draw(|f| {
                super::draw(
                    setup.area,
                    AppColors::new(),
                    f,
                    &fd,
                    &setup.gui_state,
                    &Keymap::new(),
                );
            })
            .unwrap();

        assert_snapshot!(setup.terminal.backend());
        for (_, result_row) in get_result(&setup) {
            for (result_cell_index, result_cell) in result_row.iter().enumerate() {
                assert_eq!(result_cell.bg, Color::Magenta);
                assert_eq!(
                    result_cell.fg,
                    match result_cell_index {
                        0..=3 => Color::White,
                        4..=111 => Color::Black,
                        122..=140 => Color::Gray,
                        _ => Color::Reset,
                    }
                );
            }
        }
    }

    #[test]
    /// Custom colors are applied correctly
    fn test_draw_blocks_headers_custom_colors() {
        let mut setup = test_setup(140, 1, true, true);
        let uuid = Uuid::new_v4();
        setup.gui_state.lock().next_loading(uuid);
        let fd = FrameData::from((&setup.app_data, &setup.gui_state));
        let keymap = &setup.app_data.lock().config.keymap;

        let mut colors = AppColors::new();
        colors.headers_bar.background = Color::Black;
        colors.headers_bar.loading_spinner = Color::Green;
        colors.headers_bar.text = Color::Blue;
        colors.headers_bar.text_selected = Color::Yellow;

        setup
            .terminal
            .draw(|f| {
                super::draw(setup.area, colors, f, &fd, &setup.gui_state, keymap);
            })
            .unwrap();

        assert_snapshot!(setup.terminal.backend());

        for (_, result_row) in get_result(&setup) {
            for (result_cell_index, result_cell) in result_row.iter().enumerate() {
                assert_eq!(result_cell.bg, Color::Black);
                assert_eq!(
                    result_cell.fg,
                    match result_cell_index {
                        0..=3 => Color::Green,
                        4..=111 => Color::Blue,
                        122..=140 => Color::Yellow,
                        _ => Color::Reset,
                    }
                );
            }
        }
    }

    #[test]
    /// Custom keymap for help panel is correctly display, with one definitions
    fn test_draw_blocks_headers_custom_keymap_one_definition() {
        let mut setup = test_setup(140, 1, true, true);
        let fd = FrameData::from((&setup.app_data, &setup.gui_state));
        let mut keymap = Keymap::new();

        keymap.toggle_help = (KeyCode::Char('T'), None);

        setup
            .terminal
            .draw(|f| {
                super::draw(
                    setup.area,
                    AppColors::new(),
                    f,
                    &fd,
                    &setup.gui_state,
                    &keymap,
                );
            })
            .unwrap();

        assert_snapshot!(setup.terminal.backend());
    }
    // split here
    #[test]
    /// Custom keymap for help panel is correctly display, two definitions
    fn test_draw_blocks_headers_custom_keymap_two_definitions() {
        let mut setup = test_setup(140, 1, true, true);
        let fd = FrameData::from((&setup.app_data, &setup.gui_state));
        let mut keymap = Keymap::new();

        keymap.toggle_help = (KeyCode::Char('T'), Some(KeyCode::Tab));
        setup
            .terminal
            .draw(|f| {
                super::draw(
                    setup.area,
                    AppColors::new(),
                    f,
                    &fd,
                    &setup.gui_state,
                    &keymap,
                );
            })
            .unwrap();

        assert_snapshot!(setup.terminal.backend());
    }

    fn check_color(setup: &TuiTestSetup, range: RangeInclusive<usize>) {
        for (_, result_row) in get_result(setup) {
            for (result_cell_index, result_cell) in result_row.iter().enumerate() {
                assert_eq!(result_cell.bg, Color::Magenta);
                assert_eq!(
                    result_cell.fg,
                    match result_cell_index {
                        0..=3 => Color::White,
                        122..=139 => Color::Gray,
                        // given range | help section
                        x if range.contains(&x) => Color::Gray,
                        112..=121 => Color::Reset,
                        _ => Color::Black,
                    }
                );
            }
        }
    }

    /// As a macro - headers test, check for asc/desc icon and colors
    macro_rules! test_draw_blocks_headers_sort {
        ($name:ident, $header:expr, $order:expr, $color_range:expr) => {
            #[test]
            fn $name() {
                let mut setup = test_setup(140, 1, true, true);
                let mut fd = FrameData::from((&setup.app_data, &setup.gui_state));
                fd.sorted_by = Some(($header, $order));
                setup
                    .terminal
                    .draw(|f| {
                        super::draw(
                            setup.area,
                            AppColors::new(),
                            f,
                            &fd,
                            &setup.gui_state,
                            &Keymap::new(),
                        );
                    })
                    .unwrap();
                assert_snapshot!(setup.terminal.backend());
                check_color(&setup, $color_range);
            }
        };
    }

    test_draw_blocks_headers_sort!(
        test_draw_blocks_headers_sort_containers_name_asc,
        Header::Name,
        SortedOrder::Asc,
        1..=17
    );

    test_draw_blocks_headers_sort!(
        test_draw_blocks_headers_sort_containers_name_desc,
        Header::Name,
        SortedOrder::Desc,
        1..=17
    );

    test_draw_blocks_headers_sort!(
        test_draw_blocks_headers_sort_containers_state_asc,
        Header::State,
        SortedOrder::Asc,
        18..=29
    );

    test_draw_blocks_headers_sort!(
        test_draw_blocks_headers_sort_containers_state_desc,
        Header::State,
        SortedOrder::Desc,
        18..=29
    );

    test_draw_blocks_headers_sort!(
        test_draw_blocks_headers_sort_containers_status_asc,
        Header::Status,
        SortedOrder::Asc,
        30..=41
    );

    test_draw_blocks_headers_sort!(
        test_draw_blocks_headers_sort_containers_status_desc,
        Header::Status,
        SortedOrder::Desc,
        30..=41
    );

    test_draw_blocks_headers_sort!(
        test_draw_blocks_headers_sort_containers_cpu_asc,
        Header::Cpu,
        SortedOrder::Asc,
        42..=50
    );

    test_draw_blocks_headers_sort!(
        test_draw_blocks_headers_sort_containers_cpu_desc,
        Header::Cpu,
        SortedOrder::Desc,
        42..=50
    );

    test_draw_blocks_headers_sort!(
        test_draw_blocks_headers_sort_containers_memory_asc,
        Header::Memory,
        SortedOrder::Asc,
        51..=70
    );

    test_draw_blocks_headers_sort!(
        test_draw_blocks_headers_sort_containers_memory_desc,
        Header::Memory,
        SortedOrder::Desc,
        51..=70
    );

    test_draw_blocks_headers_sort!(
        test_draw_blocks_headers_sort_containers_id_asc,
        Header::Id,
        SortedOrder::Asc,
        71..=81
    );

    test_draw_blocks_headers_sort!(
        test_draw_blocks_headers_sort_containers_id_desc,
        Header::Id,
        SortedOrder::Desc,
        71..=81
    );

    test_draw_blocks_headers_sort!(
        test_draw_blocks_headers_sort_containers_image_asc,
        Header::Image,
        SortedOrder::Asc,
        82..=91
    );

    test_draw_blocks_headers_sort!(
        test_draw_blocks_headers_sort_containers_image_desc,
        Header::Image,
        SortedOrder::Desc,
        82..=91
    );

    test_draw_blocks_headers_sort!(
        test_draw_blocks_headers_sort_containers_rx_asc,
        Header::Rx,
        SortedOrder::Asc,
        92..=101
    );

    test_draw_blocks_headers_sort!(
        test_draw_blocks_headers_sort_containers_rx_desc,
        Header::Rx,
        SortedOrder::Desc,
        92..=101
    );

    test_draw_blocks_headers_sort!(
        test_draw_blocks_headers_sort_containers_tx_asc,
        Header::Tx,
        SortedOrder::Asc,
        102..=111
    );

    test_draw_blocks_headers_sort!(
        test_draw_blocks_headers_sort_containers_tx_desc,
        Header::Tx,
        SortedOrder::Desc,
        102..=111
    );
}
