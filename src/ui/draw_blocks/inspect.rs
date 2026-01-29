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
    use std::sync::LazyLock;

    use crate::{
        app_data::InspectData,
        config::{AppColors, Keymap},
        ui::draw_blocks::tests::{get_result, test_setup},
    };
    use bollard::secret::ContainerInspectResponse;
    use crossterm::event::KeyCode;
    use insta::assert_snapshot;
    use ratatui::style::Color;

    static INSPECT_DATA: LazyLock<InspectData> = LazyLock::new(|| {
        InspectData::from(
            serde_json::from_str::<ContainerInspectResponse>(include_str!("./inspect.json"))
                .unwrap(),
        )
    });

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
}
