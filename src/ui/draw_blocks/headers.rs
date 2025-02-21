use std::sync::Arc;

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

// Draw heading bar at top of program, always visible
/// TODO Should separate into loading icon/headers/help functions
#[allow(clippy::too_many_lines)]
pub fn draw(
    area: Rect,
    colors: AppColors,
    frame: &mut Frame,
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

    frame.render_widget(
        Block::default().style(gen_style(Some(colors.headers_bar.background), Color::Reset)),
        area,
    );

    // Generate a block for the header, if the header is currently being used to sort a column, then highlight it white
    let header_block = |x: &Header, colors: AppColors| {
        let mut color = colors.headers_bar.text;
        let mut suffix = "";
        if let Some((a, b)) = &fd.sorted_by {
            if x == a {
                match b {
                    SortedOrder::Asc => suffix = " ▲",
                    SortedOrder::Desc => suffix = " ▼",
                }
                color = colors.headers_bar.text_selected;
            };
        };

        (color, suffix)
    };

    // Generate block for the headers, state and status has a specific layout, others all equal
    // width is dependant on it that column is selected to sort - or not
    // TODO - yes this is a mess, needs documenting correctly
    let gen_header = |header: &Header, width: usize, colors: AppColors| {
        let block = header_block(header, colors);

        let text = format!(
            "{x:<width$}{MARGIN}",
            x = format!("{header}{ic}", ic = block.1),
        );
        let count = u16::try_from(text.chars().count()).unwrap_or_default();
        let status = Paragraph::new(text)
            .style(gen_style(None, block.0))
            .alignment(Alignment::Left);
        (status, count)
    };

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

    let suffix = if fd.status.contains(&Status::Help) {
        "exit"
    } else {
        "show"
    };

    let info_text = if keymap.toggle_help == Keymap::new().toggle_help {
        format!("( h ) {suffix} help{MARGIN}")
    } else if let Some(secondary) = keymap.toggle_help.1 {
        format!(
            " ( {} | {secondary} ) {suffix} help{MARGIN}",
            keymap.toggle_help.0
        )
    } else {
        format!(" ( {} ) {suffix} help{MARGIN}", keymap.toggle_help.0)
    };
    let info_width = info_text.chars().count();

    let column_width = usize::from(area.width).saturating_sub(info_width);
    let column_width = if column_width > 0 { column_width } else { 1 };
    let splits = if fd.has_containers {
        vec![
            Constraint::Max(4),
            Constraint::Max(column_width.try_into().unwrap_or_default()),
            Constraint::Max(info_width.try_into().unwrap_or_default()),
        ]
    } else {
        CONSTRAINT_100.to_vec()
    };

    let split_bar = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(splits)
        .split(area);

    // Draw loading icon, or not, and a prefix with a single space
    let loading_paragraph = Paragraph::new(format!("{:>2}", fd.loading_icon))
        .style(gen_style(None, colors.headers_bar.loading_spinner))
        .alignment(Alignment::Left);
    frame.render_widget(loading_paragraph, split_bar[0]);
    if fd.has_containers {
        let header_section_width = split_bar[1].width;

        let mut counter = 0;

        // Only show a header if the header cumulative header width is less than the header section width
        let header_data = header_meta
            .iter()
            .filter_map(|i| {
                let header_block = gen_header(&i.0, i.1.into(), colors);
                counter += header_block.1;
                if counter <= header_section_width {
                    Some((header_block.0, i.0, Constraint::Max(header_block.1)))
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
            frame.render_widget(paragraph, rect);
        }
    }

    // show/hide help
    let help_text_color = if fd.status.contains(&Status::Help) {
        colors.headers_bar.text
    } else {
        colors.headers_bar.text_selected
    };

    let help_paragraph = Paragraph::new(info_text)
        .style(gen_style(None, help_text_color))
        .alignment(Alignment::Right);

    // If no containers, don't display the headers, could maybe do this first?
    let help_index = if fd.has_containers { 2 } else { 0 };
    gui_state
        .lock()
        .update_region_map(Region::HelpPanel, split_bar[help_index]);
    frame.render_widget(help_paragraph, split_bar[help_index]);
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use std::ops::RangeInclusive;

    use crossterm::event::KeyCode;
    use ratatui::style::Color;
    use uuid::Uuid;

    use crate::{
        app_data::{Header, SortedOrder, StatefulList},
        config::{AppColors, Keymap},
        ui::{
            FrameData, Status,
            draw_blocks::tests::{expected_to_vec, get_result, test_setup},
        },
    };

    #[test]
    /// Heading back only has show/exit help when no containers, correctly coloured
    fn test_draw_blocks_headers_no_containers() {
        let (w, h) = (140, 1);
        let mut setup = test_setup(w, h, true, true);
        setup.app_data.lock().containers = StatefulList::new(vec![]);

        let mut fd = FrameData::from((&setup.app_data, &setup.gui_state));

        let expected = [
            "                                                                                                                          ( h ) show help   ",
        ];

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

        for (row_index, result_row) in get_result(&setup, w) {
            let expected_row = expected_to_vec(&expected, row_index);
            for (result_cell_index, result_cell) in result_row.iter().enumerate() {
                assert_eq!(result_cell.bg, Color::Magenta);
                assert_eq!(result_cell.symbol(), expected_row[result_cell_index]);
                assert_eq!(result_cell.fg, Color::Gray,);
            }
        }

        fd.status.insert(Status::Help);
        let expected = [
            "                                                                                                                          ( h ) exit help   ",
        ];
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

        for (row_index, result_row) in get_result(&setup, w) {
            let expected_row = expected_to_vec(&expected, row_index);
            for (result_cell_index, result_cell) in result_row.iter().enumerate() {
                assert_eq!(result_cell.symbol(), expected_row[result_cell_index]);
                assert_eq!(result_cell.bg, Color::Magenta);
                assert_eq!(result_cell.fg, Color::Black);
            }
        }
    }

    #[test]
    /// Show all headings when containers present, colors valid
    fn test_draw_blocks_headers_some_containers() {
        let (w, h) = (140, 1);
        let mut setup = test_setup(w, h, true, true);
        let fd = FrameData::from((&setup.app_data, &setup.gui_state));

        let expected = [
            "    name          state       status      cpu      memory/limit        id         image     ↓ rx      ↑ tx                ( h ) show help   ",
        ];
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

        for (row_index, result_row) in get_result(&setup, w) {
            let expected_row = expected_to_vec(&expected, row_index);
            for (result_cell_index, result_cell) in result_row.iter().enumerate() {
                assert_eq!(result_cell.symbol(), expected_row[result_cell_index]);
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
        let (w, h) = (80, 1);
        let mut setup = test_setup(w, h, true, true);
        let fd = FrameData::from((&setup.app_data, &setup.gui_state));

        let expected =
            ["    name          state       status      cpu                 ( h ) show help   "];
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

        for (row_index, result_row) in get_result(&setup, w) {
            let expected_row = expected_to_vec(&expected, row_index);
            for (result_cell_index, result_cell) in result_row.iter().enumerate() {
                assert_eq!(result_cell.symbol(), expected_row[result_cell_index]);
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
    /// Test all combination of headers & sort by
    fn test_draw_blocks_headers_sort_containers() {
        let (w, h) = (140, 1);
        let mut setup = test_setup(w, h, true, true);
        let mut fd = FrameData::from((&setup.app_data, &setup.gui_state));

        // Actual test, used for each header and sorted type
        let mut test =
            |expected: &[&str], range: RangeInclusive<usize>, x: (Header, SortedOrder)| {
                fd.sorted_by = Some(x);

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

                for (row_index, result_row) in get_result(&setup, w) {
                    let expected_row = expected_to_vec(expected, row_index);
                    for (result_cell_index, result_cell) in result_row.iter().enumerate() {
                        assert_eq!(result_cell.symbol(), expected_row[result_cell_index]);

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
            };

        // Name
        test(
            &[
                "    name ▲        state       status      cpu      memory/limit        id         image     ↓ rx      ↑ tx                ( h ) show help   ",
            ],
            1..=17,
            (Header::Name, SortedOrder::Asc),
        );
        test(
            &[
                "    name ▼        state       status      cpu      memory/limit        id         image     ↓ rx      ↑ tx                ( h ) show help   ",
            ],
            1..=17,
            (Header::Name, SortedOrder::Desc),
        );
        // state
        test(
            &[
                "    name          state ▲     status      cpu      memory/limit        id         image     ↓ rx      ↑ tx                ( h ) show help   ",
            ],
            18..=29,
            (Header::State, SortedOrder::Asc),
        );
        test(
            &[
                "    name          state ▼     status      cpu      memory/limit        id         image     ↓ rx      ↑ tx                ( h ) show help   ",
            ],
            18..=29,
            (Header::State, SortedOrder::Desc),
        );
        // status
        test(
            &[
                "    name          state       status ▲    cpu      memory/limit        id         image     ↓ rx      ↑ tx                ( h ) show help   ",
            ],
            30..=41,
            (Header::Status, SortedOrder::Asc),
        );
        test(
            &[
                "    name          state       status ▼    cpu      memory/limit        id         image     ↓ rx      ↑ tx                ( h ) show help   ",
            ],
            30..=41,
            (Header::Status, SortedOrder::Desc),
        );
        // cpu
        test(
            &[
                "    name          state       status      cpu ▲    memory/limit        id         image     ↓ rx      ↑ tx                ( h ) show help   ",
            ],
            42..=50,
            (Header::Cpu, SortedOrder::Asc),
        );
        test(
            &[
                "    name          state       status      cpu ▼    memory/limit        id         image     ↓ rx      ↑ tx                ( h ) show help   ",
            ],
            42..=50,
            (Header::Cpu, SortedOrder::Desc),
        );
        // memory
        test(
            &[
                "    name          state       status      cpu      memory/limit ▲      id         image     ↓ rx      ↑ tx                ( h ) show help   ",
            ],
            51..=70,
            (Header::Memory, SortedOrder::Asc),
        );
        test(
            &[
                "    name          state       status      cpu      memory/limit ▼      id         image     ↓ rx      ↑ tx                ( h ) show help   ",
            ],
            51..=70,
            (Header::Memory, SortedOrder::Desc),
        );
        //id
        test(
            &[
                "    name          state       status      cpu      memory/limit        id ▲       image     ↓ rx      ↑ tx                ( h ) show help   ",
            ],
            71..=81,
            (Header::Id, SortedOrder::Asc),
        );
        test(
            &[
                "    name          state       status      cpu      memory/limit        id ▼       image     ↓ rx      ↑ tx                ( h ) show help   ",
            ],
            71..=81,
            (Header::Id, SortedOrder::Desc),
        );
        // image
        test(
            &[
                "    name          state       status      cpu      memory/limit        id         image ▲   ↓ rx      ↑ tx                ( h ) show help   ",
            ],
            82..=91,
            (Header::Image, SortedOrder::Asc),
        );
        test(
            &[
                "    name          state       status      cpu      memory/limit        id         image ▼   ↓ rx      ↑ tx                ( h ) show help   ",
            ],
            82..=91,
            (Header::Image, SortedOrder::Desc),
        );
        // rx
        test(
            &[
                "    name          state       status      cpu      memory/limit        id         image     ↓ rx ▲    ↑ tx                ( h ) show help   ",
            ],
            92..=101,
            (Header::Rx, SortedOrder::Asc),
        );
        test(
            &[
                "    name          state       status      cpu      memory/limit        id         image     ↓ rx ▼    ↑ tx                ( h ) show help   ",
            ],
            92..=101,
            (Header::Rx, SortedOrder::Desc),
        );
        // tx
        test(
            &[
                "    name          state       status      cpu      memory/limit        id         image     ↓ rx      ↑ tx ▲              ( h ) show help   ",
            ],
            102..=111,
            (Header::Tx, SortedOrder::Asc),
        );
        test(
            &[
                "    name          state       status      cpu      memory/limit        id         image     ↓ rx      ↑ tx ▼              ( h ) show help   ",
            ],
            102..=111,
            (Header::Tx, SortedOrder::Desc),
        );
    }

    #[test]
    /// Show animation
    fn test_draw_blocks_headers_animation() {
        let (w, h) = (140, 1);
        let mut setup = test_setup(w, h, true, true);
        let uuid = Uuid::new_v4();
        setup.gui_state.lock().next_loading(uuid);
        let fd = FrameData::from((&setup.app_data, &setup.gui_state));

        let expected = [
            " ⠙  name          state       status      cpu      memory/limit        id         image     ↓ rx      ↑ tx                ( h ) show help   ",
        ];

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

        for (row_index, result_row) in get_result(&setup, w) {
            let expected_row = expected_to_vec(&expected, row_index);
            for (result_cell_index, result_cell) in result_row.iter().enumerate() {
                assert_eq!(result_cell.symbol(), expected_row[result_cell_index]);
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
        let (w, h) = (140, 1);
        let mut setup = test_setup(w, h, true, true);
        let uuid = Uuid::new_v4();
        setup.gui_state.lock().next_loading(uuid);
        let fd = FrameData::from((&setup.app_data, &setup.gui_state));
        let keymap = &setup.app_data.lock().config.keymap;

        let mut colors = AppColors::new();
        colors.headers_bar.background = Color::Black;
        colors.headers_bar.loading_spinner = Color::Green;
        colors.headers_bar.text = Color::Blue;
        colors.headers_bar.text_selected = Color::Yellow;

        let expected = [
            " ⠙  name          state       status      cpu      memory/limit        id         image     ↓ rx      ↑ tx                ( h ) show help   ",
        ];

        setup
            .terminal
            .draw(|f| {
                super::draw(setup.area, colors, f, &fd, &setup.gui_state, keymap);
            })
            .unwrap();

        for (row_index, result_row) in get_result(&setup, w) {
            let expected_row = expected_to_vec(&expected, row_index);
            for (result_cell_index, result_cell) in result_row.iter().enumerate() {
                assert_eq!(result_cell.symbol(), expected_row[result_cell_index]);
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
    /// Custom keymap for help panel is correctly display, with one and two definitions
    fn test_draw_blocks_headers_custom_keymap() {
        let (w, h) = (140, 1);
        let mut setup = test_setup(w, h, true, true);
        let fd = FrameData::from((&setup.app_data, &setup.gui_state));
        let mut keymap = Keymap::new();

        keymap.toggle_help = (KeyCode::Char('T'), None);

        let expected = [
            "    name          state       status      cpu      memory/limit        id         image     ↓ rx      ↑ tx                ( T ) show help   ",
        ];
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

        for (row_index, result_row) in get_result(&setup, w) {
            let expected_row = expected_to_vec(&expected, row_index);
            for (result_cell_index, result_cell) in result_row.iter().enumerate() {
                assert_eq!(result_cell.symbol(), expected_row[result_cell_index]);
            }
        }

        keymap.toggle_help = (KeyCode::Char('T'), Some(KeyCode::Tab));
        let expected = [
            "    name          state       status      cpu      memory/limit        id         image     ↓ rx      ↑ tx          ( T | Tab ) show help   ",
        ];
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

        for (row_index, result_row) in get_result(&setup, w) {
            let expected_row = expected_to_vec(&expected, row_index);
            for (result_cell_index, result_cell) in result_row.iter().enumerate() {
                assert_eq!(result_cell.symbol(), expected_row[result_cell_index]);
            }
        }
    }
}
