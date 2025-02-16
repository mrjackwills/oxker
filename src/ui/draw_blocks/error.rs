use ratatui::{
    layout::Alignment,
    style::Style,
    widgets::{Block, BorderType, Borders, Clear, Paragraph},
    Frame,
};

use super::{max_line_width, NAME, VERSION};
use crate::{
    app_error::AppError,
    config::{AppColors, Keymap},
    ui::gui_state::BoxLocation,
};

use super::popup;

/// Draw an error popup over whole screen
pub fn draw(
    f: &mut Frame,
    error: &AppError,
    keymap: &Keymap,
    seconds: Option<u8>,
    colors: AppColors,
) {
    let block = Block::default()
        .title(" Error ")
        .border_type(BorderType::Rounded)
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL);

    let to_push = if matches!(error, AppError::DockerConnect) {
        format!(
            "\n\n {}::v{} closing in {:02} seconds",
            NAME,
            VERSION,
            seconds.unwrap_or(5)
        )
    } else {
        let clear_suffix = "clear error";
        let clear_text = if keymap.clear == Keymap::new().clear {
            format!("( {} ) {clear_suffix}", keymap.clear.0)
        } else if let Some(secondary) = keymap.clear.1 {
            format!(" ( {} | {secondary} ) {clear_suffix}", keymap.clear.0)
        } else {
            format!(" ( {} ) {clear_suffix}", keymap.clear.0)
        };

        let quit_suffix = "quit oxker";
        let quit_text = if keymap.quit == Keymap::new().quit {
            format!("( {} ) {quit_suffix}", keymap.quit.0)
        } else if let Some(secondary) = keymap.quit.1 {
            format!(" ( {} | {secondary} ) {quit_suffix}", keymap.quit.0)
        } else {
            format!(" ( {} ) {quit_suffix}", keymap.quit.0)
        };

        format!("\n\n{clear_text}\n\n{quit_text}")
    };

    let mut text = format!("\n{error}");

    text.push_str(to_push.as_str());

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

    use super::VERSION;
    use crate::{
        app_error::AppError,
        config::{AppColors, Keymap},
        ui::draw_blocks::tests::{expected_to_vec, get_result, test_setup},
    };
    use crossterm::event::KeyCode;
    use ratatui::style::Color;

    #[test]
    /// Test that the error popup is centered, red background, white border, white text, and displays the correct text
    fn test_draw_blocks_docker_connect_error() {
        let (w, h) = (46, 9);
        let mut setup = test_setup(w, h, true, true);
        let app_colors = setup.app_data.lock().config.app_colors;
        let keymap = &setup.app_data.lock().config.keymap;

        setup
            .terminal
            .draw(|f| {
                super::draw(f, &AppError::DockerConnect, keymap, Some(4), app_colors);
            })
            .unwrap();

        let version_row = format!(" │    oxker::v{VERSION} closing in 04 seconds   │ ");
        let expected = [
            "                                              ",
            " ╭───────────────── Error ──────────────────╮ ",
            " │                                          │ ",
            " │      Unable to access docker daemon      │ ",
            " │                                          │ ",
            version_row.as_str(),
            " │                                          │ ",
            " ╰──────────────────────────────────────────╯ ",
            "                                              ",
        ];

        for (row_index, result_row) in get_result(&setup, w) {
            let expected_row = expected_to_vec(&expected, row_index);
            for (result_cell_index, result_cell) in result_row.iter().enumerate() {
                assert_eq!(result_cell.symbol(), expected_row[result_cell_index]);

                match (row_index, result_cell_index) {
                    (0 | 8, _) | (1..=7, 0 | 45) => {
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
    fn test_draw_blocks_clearable_error() {
        let (w, h) = (39, 11);
        let mut setup = test_setup(w, h, true, true);

        let app_colors = setup.app_data.lock().config.app_colors;
        let keymap = &setup.app_data.lock().config.keymap;

        setup
            .terminal
            .draw(|f| {
                super::draw(f, &AppError::DockerExec, keymap, Some(4), app_colors);
            })
            .unwrap();

        let expected = [
            "                                       ",
            " ╭────────────── Error ──────────────╮ ",
            " │                                   │ ",
            " │   Unable to exec into container   │ ",
            " │                                   │ ",
            " │         ( c ) clear error         │ ",
            " │                                   │ ",
            " │         ( q ) quit oxker          │ ",
            " │                                   │ ",
            " ╰───────────────────────────────────╯ ",
            "                                       ",
        ];

        for (row_index, result_row) in get_result(&setup, w) {
            let expected_row = expected_to_vec(&expected, row_index);
            for (result_cell_index, result_cell) in result_row.iter().enumerate() {
                assert_eq!(result_cell.symbol(), expected_row[result_cell_index]);

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
    fn test_draw_blocks_clearable_error_custom_colors() {
        let (w, h) = (39, 11);
        let mut setup = test_setup(w, h, true, true);

        let keymap = &setup.app_data.lock().config.keymap;

        let mut colors = AppColors::new();
        colors.popup_error.background = Color::Yellow;
        colors.popup_error.text = Color::Black;

        setup
            .terminal
            .draw(|f| {
                super::draw(f, &AppError::DockerExec, keymap, Some(4), colors);
            })
            .unwrap();

        let expected = [
            "                                       ",
            " ╭────────────── Error ──────────────╮ ",
            " │                                   │ ",
            " │   Unable to exec into container   │ ",
            " │                                   │ ",
            " │         ( c ) clear error         │ ",
            " │                                   │ ",
            " │         ( q ) quit oxker          │ ",
            " │                                   │ ",
            " ╰───────────────────────────────────╯ ",
            "                                       ",
        ];

        for (row_index, result_row) in get_result(&setup, w) {
            let expected_row = expected_to_vec(&expected, row_index);
            for (result_cell_index, result_cell) in result_row.iter().enumerate() {
                assert_eq!(result_cell.symbol(), expected_row[result_cell_index]);

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
    /// Custom keymap applied correct with both 1 and 2 definitions
    fn test_draw_blocks_clearable_error_custom_keymap() {
        let (w, h) = (39, 11);
        let mut setup = test_setup(w, h, true, true);

        let mut keymap = Keymap::new();
        keymap.clear = (KeyCode::BackTab, None);
        keymap.quit = (KeyCode::F(4), None);

        setup
            .terminal
            .draw(|f| {
                super::draw(f, &AppError::DockerExec, &keymap, None, AppColors::new());
            })
            .unwrap();

        let expected = [
            "                                       ",
            " ╭────────────── Error ──────────────╮ ",
            " │                                   │ ",
            " │   Unable to exec into container   │ ",
            " │                                   │ ",
            " │      ( Back Tab ) clear error     │ ",
            " │                                   │ ",
            " │         ( F4 ) quit oxker         │ ",
            " │                                   │ ",
            " ╰───────────────────────────────────╯ ",
            "                                       ",
        ];

        for (row_index, result_row) in get_result(&setup, w) {
            let expected_row = expected_to_vec(&expected, row_index);
            for (result_cell_index, result_cell) in result_row.iter().enumerate() {
                assert_eq!(result_cell.symbol(), expected_row[result_cell_index]);
            }
        }

        let mut keymap = Keymap::new();
        keymap.clear = (KeyCode::BackTab, Some(KeyCode::Char('m')));
        keymap.quit = (KeyCode::F(4), Some(KeyCode::End));

        setup
            .terminal
            .draw(|f| {
                super::draw(f, &AppError::DockerExec, &keymap, None, AppColors::new());
            })
            .unwrap();

        let expected = [
            "                                       ",
            " ╭────────────── Error ──────────────╮ ",
            " │                                   │ ",
            " │   Unable to exec into container   │ ",
            " │                                   │ ",
            " │    ( Back Tab | m ) clear error   │ ",
            " │                                   │ ",
            " │      ( F4 | End ) quit oxker      │ ",
            " │                                   │ ",
            " ╰───────────────────────────────────╯ ",
            "                                       ",
        ];

        for (row_index, result_row) in get_result(&setup, w) {
            let expected_row = expected_to_vec(&expected, row_index);
            for (result_cell_index, result_cell) in result_row.iter().enumerate() {
                assert_eq!(result_cell.symbol(), expected_row[result_cell_index]);
            }
        }

        let mut keymap = Keymap::new();
        keymap.quit = (KeyCode::F(4), Some(KeyCode::End));

        setup
            .terminal
            .draw(|f| {
                super::draw(f, &AppError::DockerExec, &keymap, None, AppColors::new());
            })
            .unwrap();

        let expected = [
            "                                       ",
            " ╭────────────── Error ──────────────╮ ",
            " │                                   │ ",
            " │   Unable to exec into container   │ ",
            " │                                   │ ",
            " │         ( c ) clear error         │ ",
            " │                                   │ ",
            " │      ( F4 | End ) quit oxker      │ ",
            " │                                   │ ",
            " ╰───────────────────────────────────╯ ",
            "                                       ",
        ];

        for (row_index, result_row) in get_result(&setup, w) {
            let expected_row = expected_to_vec(&expected, row_index);
            for (result_cell_index, result_cell) in result_row.iter().enumerate() {
                assert_eq!(result_cell.symbol(), expected_row[result_cell_index]);
            }
        }
    }
}
