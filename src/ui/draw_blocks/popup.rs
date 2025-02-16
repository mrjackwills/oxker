use ratatui::layout::{Direction, Layout, Rect};

use crate::ui::gui_state::BoxLocation;

/// draw a box in the one of the BoxLocations, based on max line width + number of lines
pub fn draw(text_lines: usize, text_width: usize, r: Rect, box_location: BoxLocation) -> Rect {
    // Make sure blank_space can't be an negative, as will crash
    let calc = |x: u16, y: usize| usize::from(x).saturating_sub(y).saturating_div(2);

    let blank_vertical = calc(r.height, text_lines);
    let blank_horizontal = calc(r.width, text_width);

    let (h_constraints, v_constraints) = box_location.get_constraints(
        blank_horizontal.try_into().unwrap_or_default(),
        blank_vertical.try_into().unwrap_or_default(),
        text_lines.try_into().unwrap_or_default(),
        text_width.try_into().unwrap_or_default(),
    );

    let indexes = box_location.get_indexes();

    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(v_constraints)
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints(h_constraints)
        .split(popup_layout[indexes.0])[indexes.1]
}
