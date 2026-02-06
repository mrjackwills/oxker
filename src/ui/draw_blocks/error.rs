use ratatui::{
    Frame,
    layout::Alignment,
    style::Style,
    widgets::{Block, BorderType, Borders, Clear, Paragraph},
};

use super::{NAME, VERSION, max_line_width};
use crate::{
    app_error::AppError,
    config::{AppColors, Keymap},
    ui::gui_state::BoxLocation,
};

use super::popup;

const SUFFIX_CLEAR: &str = "clear error";
const SUFFIX_QUIT: &str = "quit oxker";

/// Draw an error popup over whole screen
pub fn draw(
    colors: AppColors,
    error: &AppError,
    f: &mut Frame,
    host: Option<String>,
    keymap: &Keymap,
    seconds: Option<u8>,
) {
    let block = Block::default()
        .title(" Error ")
        .border_type(BorderType::Rounded)
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL);

    let mut text = format!("\n{error}");

    if error == &AppError::DockerConnect {
        let s = if let Some(host) = host {
            format!(" @ \"{host}\"")
        } else {
            String::new()
        };
        text.push_str(&format!(
            "{}\n\n {}::v{} closing in {:02} seconds",
            s,
            NAME,
            VERSION,
            seconds.unwrap_or(5),
        ))
    } else {
        let clear_text = if keymap.clear == Keymap::new().clear {
            format!("( {} ) {SUFFIX_CLEAR}", keymap.clear.0)
        } else if let Some(secondary) = keymap.clear.1 {
            format!(" ( {} | {secondary} ) {SUFFIX_CLEAR}", keymap.clear.0)
        } else {
            format!(" ( {} ) {SUFFIX_CLEAR}", keymap.clear.0)
        };
        text.push_str(&format!("\n\n{clear_text}"));
    }

    let quit_text = if keymap.quit == Keymap::new().quit {
        format!("( {} ) {SUFFIX_QUIT}", keymap.quit.0)
    } else if let Some(secondary) = keymap.quit.1 {
        format!(" ( {} | {secondary} ) {SUFFIX_QUIT}", keymap.quit.0)
    } else {
        format!(" ( {} ) {SUFFIX_QUIT}", keymap.quit.0)
    };
    text.push_str(&format!("\n\n{quit_text}"));

    // Find the maximum line width & height
    let padded_width = max_line_width(&text) + 8;

    let line_count = text.lines().count();
    let padded_height = if line_count % 2 == 0 {
        line_count + 3
    } else {
        line_count + 2
    };

    let paragraph = Paragraph::new(text)
        .style(
            Style::default()
                .bg(colors.popup_error.background)
                .fg(colors.popup_error.text),
        )
        .block(block)
        .alignment(Alignment::Center);

    let area = popup::draw(
        padded_height,
        padded_width,
        f.area(),
        BoxLocation::MiddleCentre,
    );

    f.render_widget(Clear, area);
    f.render_widget(paragraph, area);
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use crate::{
        app_error::AppError,
        config::{AppColors, Keymap},
        ui::draw_blocks::tests::{get_result, test_setup},
    };
    use crossterm::event::KeyCode;
    use insta::assert_snapshot;
    use ratatui::style::Color;

    #[test]
    /// Test that the error popup is centered, red background, white border, white text, and displays the correct text
    fn test_draw_blocks_error_docker_connect_error() {
        let mut setup = test_setup(50, 11, true, true);
        setup
            .terminal
            .draw(|f| {
                super::draw(
                    AppColors::new(),
                    &AppError::DockerConnect,
                    f,
                    None,
                    &Keymap::new(),
                    Some(4),
                );
            })
            .unwrap();
        assert_snapshot!(setup.terminal.backend());
        for (row_index, result_row) in get_result(&setup) {
            for (result_cell_index, result_cell) in result_row.iter().enumerate() {
                match (row_index, result_cell_index) {
                    (0 | 10, _) | (_, 0 | 49) => {
                        assert_eq!(result_cell.bg, Color::Reset);
                        assert_eq!(result_cell.fg, Color::Reset);
                    }
                    _ => {
                        assert_eq!(result_cell.bg, Color::Red);
                        assert_eq!(result_cell.fg, Color::White);
                    }
                }
            }
        }
    }

    #[test]
    /// Test that the error popup is centered, red background, white border, white text, and displays the correct text with the custom docker host address
    fn test_draw_blocks_error_docker_connect_error_custom_host() {
        let mut setup = test_setup(60, 11, true, true);

        setup
            .terminal
            .draw(|f| {
                super::draw(
                    AppColors::new(),
                    &AppError::DockerConnect,
                    f,
                    Some("/test/host.sock".to_owned()),
                    &Keymap::new(),
                    Some(4),
                );
            })
            .unwrap();
        assert_snapshot!(setup.terminal.backend());
        for (row_index, result_row) in get_result(&setup) {
            for (result_cell_index, result_cell) in result_row.iter().enumerate() {
                match (row_index, result_cell_index) {
                    (0 | 10, _) | (_, 0 | 59) => {
                        assert_eq!(result_cell.bg, Color::Reset);
                        assert_eq!(result_cell.fg, Color::Reset);
                    }
                    _ => {
                        assert_eq!(result_cell.bg, Color::Red);
                        assert_eq!(result_cell.fg, Color::White);
                    }
                }
            }
        }
    }

    #[test]
    /// Test that the clearable error popup is centered, red background, white border, white text, and displays the correct text
    fn test_draw_blocks_error_clearable_error() {
        let mut setup = test_setup(39, 11, true, true);

        setup
            .terminal
            .draw(|f| {
                super::draw(
                    AppColors::new(),
                    &AppError::DockerExec,
                    f,
                    // TODO test me
                    None,
                    &Keymap::new(),
                    Some(4),
                );
            })
            .unwrap();

        assert_snapshot!(setup.terminal.backend());

        for (row_index, result_row) in get_result(&setup) {
            for (result_cell_index, result_cell) in result_row.iter().enumerate() {
                match (row_index, result_cell_index) {
                    (0 | 10, _) | (1..=9, 0 | 38) => {
                        assert_eq!(result_cell.bg, Color::Reset);
                        assert_eq!(result_cell.fg, Color::Reset);
                    }

                    _ => {
                        assert_eq!(result_cell.bg, Color::Red);
                        assert_eq!(result_cell.fg, Color::White);
                    }
                }
            }
        }
    }

    #[test]
    /// Custom colors applied to the error popup correctly
    fn test_draw_blocks_error_custom_colors() {
        let mut setup = test_setup(39, 11, true, true);

        let mut colors = AppColors::new();
        colors.popup_error.background = Color::Yellow;
        colors.popup_error.text = Color::Black;

        setup
            .terminal
            .draw(|f| {
                // TODO test me
                super::draw(
                    colors,
                    &AppError::DockerExec,
                    f,
                    None,
                    &Keymap::new(),
                    Some(4),
                );
            })
            .unwrap();

        assert_snapshot!(setup.terminal.backend());

        for (row_index, result_row) in get_result(&setup) {
            for (result_cell_index, result_cell) in result_row.iter().enumerate() {
                match (row_index, result_cell_index) {
                    (0 | 10, _) | (1..=9, 0 | 38) => {
                        assert_eq!(result_cell.bg, Color::Reset);
                        assert_eq!(result_cell.fg, Color::Reset);
                    }

                    _ => {
                        assert_eq!(result_cell.bg, Color::Yellow);
                        assert_eq!(result_cell.fg, Color::Black);
                    }
                }
            }
        }
    }

    #[test]
    /// Custom keymap applied correctly
    fn test_draw_blocks_error_custom_keymap() {
        let mut setup = test_setup(39, 11, true, true);

        let mut keymap = Keymap::new();
        keymap.clear = (KeyCode::BackTab, None);
        keymap.quit = (KeyCode::F(4), None);

        setup
            .terminal
            .draw(|f| {
                // TODO test me
                super::draw(
                    AppColors::new(),
                    &AppError::DockerExec,
                    f,
                    None,
                    &keymap,
                    None,
                );
            })
            .unwrap();
        assert_snapshot!(setup.terminal.backend());
    }
    #[test]
    /// Custom keymap applied with two definitions for each option
    fn test_draw_blocks_error_custom_keymap_two_definitions() {
        let mut setup = test_setup(39, 11, true, true);

        let mut keymap = Keymap::new();
        keymap.clear = (KeyCode::BackTab, Some(KeyCode::Char('m')));
        keymap.quit = (KeyCode::F(4), Some(KeyCode::End));

        setup
            .terminal
            .draw(|f| {
                // TODO test me
                super::draw(
                    AppColors::new(),
                    &AppError::DockerExec,
                    f,
                    None,
                    &keymap,
                    None,
                );
            })
            .unwrap();
        assert_snapshot!(setup.terminal.backend());
    }

    #[test]
    /// Custom keymap applied correctly, with 1 definition for the first option, and 2 definitions for the other
    fn test_draw_blocks_error_custom_keymap_one_two_definitions() {
        let mut setup = test_setup(39, 11, true, true);

        let mut keymap = Keymap::new();
        keymap.quit = (KeyCode::F(4), Some(KeyCode::End));

        setup
            .terminal
            .draw(|f| {
                // TODO test me
                super::draw(
                    AppColors::new(),
                    &AppError::DockerExec,
                    f,
                    None,
                    &keymap,
                    None,
                );
            })
            .unwrap();
        assert_snapshot!(setup.terminal.backend());
    }
}
