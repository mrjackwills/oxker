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
