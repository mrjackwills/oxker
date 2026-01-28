use std::sync::Arc;

use bollard::secret::ContainerInspectResponse;
use parking_lot::Mutex;
use ratatui::{
    Frame,
    layout::{Offset, Rect},
    style::{Style, Stylize},
    text::Line,
    widgets::{Block, Borders, Paragraph, Wrap},
};

use crate::{
    config::{AppColors, Keymap},
    ui::{
        GuiState,
        draw_blocks::{DOWN_ARROW, LEFT_ARROW, RIGHT_ARROW, UP_ARROW},
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
fn generate_upper_title(data: &ContainerInspectResponse, keymap: &Keymap) -> String {
    let mut output = String::from(" inspecting: ");
    if let Some(name) = &data.name {
        let name = if name.starts_with("/") {
            &name.replacen('/', "", 1)
        } else {
            name
        };

        output.push_str(&format!(
            "{} {} ",
            name,
            data.id.as_ref().unwrap_or(&String::new())
        ));
    }
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
fn generate_lower_title(length: usize, width: usize, offset: Offset) -> String {
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
    let right_arrow = if usize::try_from(offset.x).unwrap_or_default() == width {
        " "
    } else {
        RIGHT_ARROW
    };
    let up_arrow = if offset.y == 0 { " " } else { UP_ARROW };
    let down_arrow = if usize::try_from(offset.y).unwrap_or_default() == length {
        " "
    } else {
        DOWN_ARROW
    };

    format!(
        " {up_arrow} {:>length_width$}/{:>length_width$} {down_arrow}  {left_arrow} {:>width_width$}/{:>width_width$} {right_arrow} ",
        offset.y, length, offset.x, width
    )
}

/// Find the length of the longest line of text
fn find_longest_line(input: &str) -> usize {
    let mut output = 0;
    for i in input.lines() {
        let count = i.chars().count();
        output = output.max(count);
    }
    output
}

/// Concert data into a string, remove the first and last line - they are just {...}
fn format_data(data: &ContainerInspectResponse) -> String {
    let data_as_string = serde_json::to_string_pretty(&data).unwrap_or_default();
    data_as_string
        .lines()
        .skip(1)
        .collect::<Vec<_>>()
        .split_last()
        .map(|(_, data)| data)
        .unwrap_or_default()
        .join("\n")
}

/// Generate the Lines, remove lines & chars based on the offset and viewport
fn gen_lines<'a>(str: &str, offset: &Offset, rect: &Rect) -> Vec<Line<'a>> {
    let (w, h) = (rect.width, rect.height);

    str.lines()
        .enumerate()
        .filter_map(|(index, line)| {
            let max_line =
                usize::try_from(i32::from(h).saturating_add(offset.y)).unwrap_or_default();
            let min_line = usize::try_from(offset.y.saturating_sub(offset.y)).unwrap_or_default();

            if offset.y > i32::try_from(index).unwrap_or_default()
                || !(min_line..=max_line).contains(&index)
            {
                None
            } else {
                Some(Line::from(
                    line.chars()
                        .skip(usize::try_from(offset.x).unwrap_or_default())
                        .take(w.saturating_sub(2).into())
                        .collect::<String>(),
                ))
            }
        })
        .collect::<Vec<_>>()
}

/// Draw the InspectContainer widget to the entire screen
pub fn draw(
    f: &mut Frame,
    colors: AppColors,
    data: ContainerInspectResponse,
    gui_state: &Arc<Mutex<GuiState>>,
    keymap: &Keymap,
) {
    let rect = f.area();
    let data_as_string = format_data(&data);
    let upper_title = generate_upper_title(&data, keymap);
    let offset = gui_state.lock().get_inspect_offset();
    let height = data_as_string
        .lines()
        .count()
        .saturating_sub(usize::from(rect.height))
        .saturating_add(2);
    let width = find_longest_line(&data_as_string)
        .saturating_sub(usize::from(rect.width))
        .saturating_add(2);
    let lower_title = generate_lower_title(height, width, offset);

    gui_state.lock().set_inspect_offset_max(Offset {
        x: i32::try_from(width).unwrap_or_default(),
        y: i32::try_from(height).unwrap_or_default(),
    });

    let paragraph = Paragraph::new(gen_lines(&data_as_string, &offset, &rect))
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
