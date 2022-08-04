pub mod log_sanitizer {

    use cansi::{v3::categorise_text, Color as CansiColor, Intensity};
    use tui::{
        style::{Color, Modifier, Style},
        text::{Span, Spans},
    };

    /// Attempt to colorize the given string to tui-rs standars
    pub fn colorize_logs(input: String) -> Vec<Spans<'static>> {
        vec![Spans::from(
            categorise_text(&input)
                .into_iter()
                .map(|i| {
                    let fg_color = color_ansi_to_tui(i.fg.unwrap_or(CansiColor::White));
                    let bg_color = color_ansi_to_tui(i.bg.unwrap_or(CansiColor::Black));
                    let style = Style::default().bg(bg_color).fg(fg_color);
                    if i.blink.is_some() {
                        style.add_modifier(Modifier::SLOW_BLINK);
                    }
                    if i.underline.is_some() {
                        style.add_modifier(Modifier::UNDERLINED);
                    }
                    if i.reversed.is_some() {
                        style.add_modifier(Modifier::REVERSED);
                    }
                    if i.intensity == Some(Intensity::Bold) {
                        style.add_modifier(Modifier::BOLD);
                    }
                    if i.hidden.is_some() {
                        style.add_modifier(Modifier::HIDDEN);
                    }
                    if i.strikethrough.is_some() {
                        style.add_modifier(Modifier::CROSSED_OUT);
                    }
                    Span::styled(i.text.to_owned(), style)
                })
                .collect::<Vec<_>>(),
        )]
    }

    /// Remove all ansi formatting from a given string and create tui-rs spans
    pub fn remove_ansi(input: String) -> Vec<Spans<'static>> {
        let mut output = String::from("");
        for i in categorise_text(&input) {
            output.push_str(i.text)
        }
        raw(output)
    }

    /// create tui-rs spans that exactly match the given strings
    pub fn raw(input: String) -> Vec<Spans<'static>> {
        vec![Spans::from(Span::raw(input))]
    }

    /// Change from ansi to tui colors
    fn color_ansi_to_tui(color: CansiColor) -> Color {
        match color {
            CansiColor::Black => Color::Black,
            CansiColor::Red => Color::Red,
            CansiColor::Green => Color::Green,
            CansiColor::Yellow => Color::Yellow,
            CansiColor::Blue => Color::Blue,
            CansiColor::Magenta => Color::Magenta,
            CansiColor::Cyan => Color::Cyan,
            CansiColor::White => Color::White,
            CansiColor::BrightBlack => Color::Black,
            CansiColor::BrightRed => Color::LightRed,
            CansiColor::BrightGreen => Color::LightGreen,
            CansiColor::BrightYellow => Color::LightYellow,
            CansiColor::BrightBlue => Color::LightBlue,
            CansiColor::BrightMagenta => Color::LightMagenta,
            CansiColor::BrightCyan => Color::LightCyan,
            CansiColor::BrightWhite => Color::White,
        }
    }
}
