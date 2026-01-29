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
