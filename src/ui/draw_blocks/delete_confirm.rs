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
        DeleteButton, GuiState,
        gui_state::{BoxLocation, Region},
    },
};

use super::popup;

/// Draw the delete confirm box in the centre of the screen
/// take in container id and container name here?
pub fn draw(
    colors: AppColors,
    f: &mut Frame,
    gui_state: &Arc<Mutex<GuiState>>,
    keymap: &Keymap,
    name: &ContainerName,
) {
    let block = Block::default()
        .title(" Confirm Delete ")
        .border_type(BorderType::Rounded)
        .style(
            Style::default()
                .bg(colors.popup_delete.background)
                .fg(colors.popup_delete.text),
        )
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL);

    let confirm = Line::from(vec![
        Span::from("Are you sure you want to delete container: "),
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
        .update_region_map(Region::Delete(DeleteButton::Cancel), no_area);

    gui_state
        .lock()
        .update_region_map(Region::Delete(DeleteButton::Confirm), yes_area);
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use crossterm::event::KeyCode;
    use insta::assert_snapshot;
    use ratatui::style::{Color, Modifier};

    use crate::{
        app_data::ContainerName,
        config::{AppColors, Keymap},
        ui::draw_blocks::tests::{get_result, test_setup},
    };

    #[test]
    /// Delete container popup is drawn correctly
    fn test_draw_blocks_delete() {
        let (w, h) = (82, 10);
        let mut setup = test_setup(w, h, true, true);

        let colors = setup.app_data.lock().config.app_colors;
        let keymap = &setup.app_data.lock().config.keymap;

        setup
            .terminal
            .draw(|f| {
                super::draw(
                    colors,
                    f,
                    &setup.gui_state,
                    keymap,
                    &ContainerName::from("container_1"),
                );
            })
            .unwrap();

        assert_snapshot!(setup.terminal.backend());
        for (row_index, result_row) in get_result(&setup) {
            for (result_cell_index, result_cell) in result_row.iter().enumerate() {
                match (row_index, result_cell_index) {
                    (0 | 9, _) | (1..=8, 0..=7 | 74..=81) => {
                        assert_eq!(result_cell.bg, Color::Reset);
                        assert_eq!(result_cell.fg, Color::Reset);
                    }
                    (3, 57..=67) => {
                        assert_eq!(result_cell.bg, Color::White);
                        assert_eq!(result_cell.fg, Color::Red);
                        assert_eq!(result_cell.modifier, Modifier::BOLD);
                    }
                    _ => {
                        assert_eq!(result_cell.bg, Color::White);
                        assert_eq!(result_cell.fg, Color::Black);
                    }
                }
            }
        }
    }

    #[test]
    /// Delete container popup is drawn correctly
    fn test_draw_blocks_delete_long_name() {
        let (w, h) = (106, 10);
        let mut setup = test_setup(w, h, true, true);
        let name = ContainerName::from("container_1_container_1_container_1");
        setup.app_data.lock().containers.items[0].name = name.clone();

        let colors = setup.app_data.lock().config.app_colors;
        let keymap = &setup.app_data.lock().config.keymap;

        setup
            .terminal
            .draw(|f| {
                super::draw(colors, f, &setup.gui_state, keymap, &name);
            })
            .unwrap();
        assert_snapshot!(setup.terminal.backend());
        for (row_index, result_row) in get_result(&setup) {
            for (result_cell_index, result_cell) in result_row.iter().enumerate() {
                match (row_index, result_cell_index) {
                    (0 | 9, _) | (1..=8, 0..=7 | 98..=106) => {
                        assert_eq!(result_cell.bg, Color::Reset);
                        assert_eq!(result_cell.fg, Color::Reset);
                    }
                    (3, 57..=91) => {
                        assert_eq!(result_cell.bg, Color::White);
                        assert_eq!(result_cell.fg, Color::Red);
                        assert_eq!(result_cell.modifier, Modifier::BOLD);
                    }
                    _ => {
                        assert_eq!(result_cell.bg, Color::White);
                        assert_eq!(result_cell.fg, Color::Black);
                    }
                }
            }
        }
    }

    #[test]
    /// Custom colors applied correctly to delete popup
    fn test_draw_blocks_delete_custom_colors() {
        let (w, h) = (82, 10);
        let mut setup = test_setup(w, h, true, true);
        let mut colors = AppColors::new();
        colors.popup_delete.background = Color::Black;
        colors.popup_delete.text = Color::Yellow;
        colors.popup_delete.text_highlight = Color::Green;

        setup
            .terminal
            .draw(|f| {
                super::draw(
                    colors,
                    f,
                    &setup.gui_state,
                    &Keymap::new(),
                    &ContainerName::from("container_1"),
                );
            })
            .unwrap();

        assert_snapshot!(setup.terminal.backend());

        for (row_index, result_row) in get_result(&setup) {
            for (result_cell_index, result_cell) in result_row.iter().enumerate() {
                match (row_index, result_cell_index) {
                    (0 | 9, _) | (1..=8, 0..=7 | 74..=81) => {
                        assert_eq!(result_cell.bg, Color::Reset);
                        assert_eq!(result_cell.fg, Color::Reset);
                    }
                    (3, 57..=67) => {
                        assert_eq!(result_cell.bg, Color::Black);
                        assert_eq!(result_cell.fg, Color::Green);
                        assert_eq!(result_cell.modifier, Modifier::BOLD);
                    }
                    _ => {
                        assert_eq!(result_cell.bg, Color::Black);
                        assert_eq!(result_cell.fg, Color::Yellow);
                    }
                }
            }
        }
    }

    #[test]
    /// Custom keymap, with multiple definitions for each button, applied correctly to delete popup
    fn test_draw_blocks_delete_custom_keymap_one_definition() {
        let (w, h) = (82, 10);
        let mut setup = test_setup(w, h, true, true);
        let mut keymap = Keymap::new();
        keymap.delete_confirm = (KeyCode::F(10), None);
        keymap.delete_deny = (KeyCode::End, None);
        setup
            .terminal
            .draw(|f| {
                super::draw(
                    AppColors::new(),
                    f,
                    &setup.gui_state,
                    &keymap,
                    &ContainerName::from("container_1"),
                );
            })
            .unwrap();
        assert_snapshot!(setup.terminal.backend());
    }
    #[test]
    /// Custom keymap, with multiple definitions for each button, applied correctly to delete popup
    fn test_draw_blocks_delete_custom_keymap_two_definition() {
        let (w, h) = (82, 10);
        let mut setup = test_setup(w, h, true, true);
        let mut keymap = Keymap::new();
        keymap.delete_confirm = (KeyCode::F(10), Some(KeyCode::Char('L')));
        keymap.delete_deny = (KeyCode::End, Some(KeyCode::Up));
        setup
            .terminal
            .draw(|f| {
                super::draw(
                    AppColors::new(),
                    f,
                    &setup.gui_state,
                    &keymap,
                    &ContainerName::from("container_1"),
                );
            })
            .unwrap();
        assert_snapshot!(setup.terminal.backend());
    }
    #[test]
    /// Custom keymap, with multiple definitions for each button, applied correctly to delete popup
    fn test_draw_blocks_delete_custom_keymap_one_two_definition() {
        let (w, h) = (82, 10);
        let mut setup = test_setup(w, h, true, true);
        let mut keymap = Keymap::new();
        keymap.delete_confirm = (KeyCode::F(10), None);
        keymap.delete_deny = (KeyCode::End, Some(KeyCode::Up));
        setup
            .terminal
            .draw(|f| {
                super::draw(
                    AppColors::new(),
                    f,
                    &setup.gui_state,
                    &keymap,
                    &ContainerName::from("container_1"),
                );
            })
            .unwrap();
        assert_snapshot!(setup.terminal.backend());
    }
}
