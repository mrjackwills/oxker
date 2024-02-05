pub mod log_sanitizer {

    use cansi::{v3::categorise_text, Color as CansiColor, Intensity};
    use ratatui::{
        style::{Color, Modifier, Style},
        text::{Line, Span},
    };

    /// Attempt to colorize the given string to ratatui standards
    pub fn colorize_logs<'a>(input: &str) -> Vec<Line<'a>> {
        vec![Line::from(
            categorise_text(input)
                .iter()
                .map(|i| {
                    let mut style = Style::default()
                        .bg(color_ansi_to_tui(i.bg.unwrap_or(CansiColor::Black)))
                        .fg(color_ansi_to_tui(i.fg.unwrap_or(CansiColor::White)));
                    if i.blink.is_some() {
                        style = style.add_modifier(Modifier::SLOW_BLINK);
                    }
                    if i.underline.is_some() {
                        style = style.add_modifier(Modifier::UNDERLINED);
                    }
                    if i.reversed.is_some() {
                        style = style.add_modifier(Modifier::REVERSED);
                    }
                    if i.intensity == Some(Intensity::Bold) {
                        style = style.add_modifier(Modifier::BOLD);
                    }
                    if i.hidden.is_some() {
                        style = style.add_modifier(Modifier::HIDDEN);
                    }
                    if i.strikethrough.is_some() {
                        style = style.add_modifier(Modifier::CROSSED_OUT);
                    }
                    Span::styled(i.text.to_owned(), style)
                })
                .collect::<Vec<_>>(),
        )]
    }

    /// Remove all ansi formatting from a given string and create ratatui Lines
    pub fn remove_ansi<'a>(input: &str) -> Vec<Line<'a>> {
        raw(&categorise_text(input)
            .into_iter()
            .map(|i| i.text)
            .collect::<String>())
    }

    /// create ratatui Lines that exactly match the given strings
    pub fn raw<'a>(input: &str) -> Vec<Line<'a>> {
        vec![Line::from(Span::raw(input.to_owned()))]
    }

    /// Change from ansi to tui colors
    const fn color_ansi_to_tui(color: CansiColor) -> Color {
        match color {
            CansiColor::Black | CansiColor::BrightBlack => Color::Black,
            CansiColor::Red => Color::Red,
            CansiColor::Green => Color::Green,
            CansiColor::Yellow => Color::Yellow,
            CansiColor::Blue => Color::Blue,
            CansiColor::Magenta => Color::Magenta,
            CansiColor::Cyan => Color::Cyan,
            CansiColor::White | CansiColor::BrightWhite => Color::White,
            CansiColor::BrightRed => Color::LightRed,
            CansiColor::BrightGreen => Color::LightGreen,
            CansiColor::BrightYellow => Color::LightYellow,
            CansiColor::BrightBlue => Color::LightBlue,
            CansiColor::BrightMagenta => Color::LightMagenta,
            CansiColor::BrightCyan => Color::LightCyan,
        }
    }
}

#[cfg(test)]
mod tests {
    use ratatui::{
        style::{Color, Style},
        text::{Line, Span},
    };

    use super::log_sanitizer;

    // This spells out "oxker", with each char having a foreground and background colour
    const INPUT: &str = "\x1b[31;47mo\x1b[32;40mx\x1b[33;41mk\x1b[34;42me\x1b[35;43mr\x1b[0m";

    #[test]
    /// Return test raw, as in show escape codes
    fn color_match_raw() {
        let result = log_sanitizer::raw(INPUT);
        let expected = vec![Line {
            spans: [Span {
                content: std::borrow::Cow::Borrowed(
                    "\x1b[31;47mo\x1b[32;40mx\x1b[33;41mk\x1b[34;42me\x1b[35;43mr\x1b[0m",
                ),
                style: Style::default(),
            }].to_vec(),
            alignment: None,
        	style: Style::default()
        }];
        assert_eq!(result, expected);
    }

    #[test]
    // Use the escape codes to colorize the text
    fn color_match_colorize() {
        let result = log_sanitizer::colorize_logs(INPUT);
        let expected = vec![Line {
            spans: vec![
                Span {
                    content: std::borrow::Cow::Borrowed("o"),
                    style: Style::default().fg(Color::Red).bg(Color::White),
                },
                Span {
                    content: std::borrow::Cow::Borrowed("x"),
                    style: Style::default().fg(Color::Green).bg(Color::Black),
                },
                Span {
                    content: std::borrow::Cow::Borrowed("k"),
                    style: Style::default().fg(Color::Yellow).bg(Color::Red),
                },
                Span {
                    content: std::borrow::Cow::Borrowed("e"),
                    style: Style::default().fg(Color::Blue).bg(Color::Green),
                },
                Span {
                    content: std::borrow::Cow::Borrowed("r"),
                    style: Style::default().fg(Color::Magenta).bg(Color::Yellow),
                },
            ],
            alignment: None,
			style: Style::default()
        }];
        assert_eq!(result, expected);
    }

    #[test]
    // Remove all escape ansi codes from given input
    fn color_match_remove_ansi() {
        let result = log_sanitizer::remove_ansi(INPUT);
        let expected = vec![Line {
            spans: vec![Span {
                content: std::borrow::Cow::Borrowed("oxker"),
                style: Style::default(),
            }],
            style: Style::default(),
            alignment: None,
        }];
        assert_eq!(result, expected);
    }
}
