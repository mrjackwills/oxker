use std::sync::Arc;

use parking_lot::Mutex;
use ratatui::{
    Frame,
    layout::{Alignment, Direction, Layout},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, Paragraph},
};

use super::{CONSTRAINT_BUTTONS, CONSTRAINT_POPUP};
use crate::{
    app_data::ContainerName,
    config::{AppColors, Keymap},
    ui::{
        StopButton, GuiState,
        gui_state::{BoxLocation, Region},
    },
};

use super::popup;

/// Draw the stop confirm box in the centre of the screen
pub fn draw(
    colors: AppColors,
    f: &mut Frame,
    gui_state: &Arc<Mutex<GuiState>>,
    keymap: &Keymap,
    name: &ContainerName,
) {
    let block = Block::default()
        .title(" Confirm Stop ")
        .border_type(BorderType::Rounded)
        .style(
            Style::default()
                .bg(colors.popup_delete.background)
                .fg(colors.popup_delete.text),
        )
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL);

    let confirm = Line::from(vec![
        Span::from("Are you sure you want to stop container: "),
        Span::styled(
            name.get(),
            Style::default()
                .fg(colors.popup_delete.text_highlight)
                .bg(colors.popup_delete.background)
                .add_modifier(Modifier::BOLD),
        ),
    ]);

    let yes_text = if keymap.delete_confirm == Keymap::new().delete_confirm {
        "( y ) yes".to_owned()
    } else if let Some(secondary) = keymap.delete_confirm.1 {
        format!("( {} | {} ) yes", keymap.delete_confirm.0, secondary)
    } else {
        format!("( {} ) yes", keymap.delete_confirm.0)
    };

    let no_text = if keymap.delete_deny == Keymap::new().delete_deny {
        "( n ) no".to_owned()
    } else if let Some(secondary) = keymap.delete_deny.1 {
        format!("( {} | {} ) no", keymap.delete_deny.0, secondary)
    } else {
        format!("( {} ) no", keymap.delete_deny.0)
    };

    // Find the maximum line width & height, and add some padding
    let max_line_width = u16::try_from(confirm.width()).unwrap_or(64) + 12;
    let lines = 8;

    let confirm_para = Paragraph::new(confirm).alignment(Alignment::Center);

    let button_block = || {
        Block::default()
            .border_type(BorderType::Rounded)
            .borders(Borders::ALL)
            .style(Style::default().bg(colors.popup_delete.background))
    };

    let yes_para = Paragraph::new(yes_text)
        .alignment(Alignment::Center)
        .block(button_block());

    let no_para = Paragraph::new(no_text)
        .alignment(Alignment::Center)
        .block(button_block());

    let area = popup::draw(
        lines,
        max_line_width.into(),
        f.area(),
        BoxLocation::MiddleCentre,
    );

    let split_popup = Layout::default()
        .direction(Direction::Vertical)
        .constraints(CONSTRAINT_POPUP)
        .split(area);

    let split_buttons = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(CONSTRAINT_BUTTONS)
        .split(split_popup[3]);

    let no_area = split_buttons[1];
    let yes_area = split_buttons[3];

    f.render_widget(Clear, area);
    f.render_widget(block, area);
    f.render_widget(confirm_para, split_popup[1]);
    f.render_widget(no_para, no_area);
    f.render_widget(yes_para, yes_area);
    // Insert button areas into region map, so can interact with them on click
    gui_state
        .lock()
        .update_region_map(Region::Stop(StopButton::Cancel), no_area);

    gui_state
        .lock()
        .update_region_map(Region::Stop(StopButton::Confirm), yes_area);
}
