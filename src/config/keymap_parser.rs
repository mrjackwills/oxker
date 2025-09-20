use std::collections::HashSet;

use crossterm::event::{KeyCode, KeyModifiers};

/// The macro accepts a list of struct names with key names
/// Returns a struct where every key name is an Option<String>, with the correct derived attributes
macro_rules! optional_config_struct {
    ($($struct_name:ident, $($key_name:ident),*);*) => {
        $(
            #[derive(Debug, serde::Deserialize, Clone, PartialEq, Eq)]
            pub struct $struct_name {
                $(
                    $key_name: Option<Vec<String>>,
                )*
                pub scroll_many: Option<Vec<String>>,
            }
        )*
    };
}

/// The macro accepts a list of struct names with key names
/// Similar to the optional_config_struct macro as above, but returns struct where every key name is Color
macro_rules! config_struct {
    ($($struct_name:ident, $($key_name:ident),*);*) => {
        $(
            #[derive(Debug, Clone, PartialEq, Eq)]
            pub struct $struct_name {
                 $(
                    pub $key_name: (KeyCode, Option<KeyCode>),
                )*
                pub scroll_many: KeyModifiers,
            }
        )*
    };
}

optional_config_struct!(
    ConfigKeymap,
    clear,
    delete_confirm,
    delete_deny,
    exec,
    filter_mode,
    force_redraw,
    log_scroll_back,
    log_scroll_forward,
    log_search_mode,
    log_section_height_decrease,
    log_section_height_increase,
    log_section_toggle,
    quit,
    save_logs,
    scroll_down,
    scroll_end,
    scroll_start,
    scroll_up,
    select_next_panel,
    select_previous_panel,
    sort_by_cpu,
    sort_by_id,
    sort_by_image,
    sort_by_memory,
    sort_by_name,
    sort_by_rx,
    sort_by_state,
    sort_by_status,
    sort_by_tx,
    sort_reset,
    toggle_help,
    toggle_mouse_capture
);

config_struct!(
    Keymap,
    clear,
    delete_confirm,
    delete_deny,
    exec,
    filter_mode,
    force_redraw,
    log_scroll_back,
    log_scroll_forward,
    log_search_mode,
    log_section_height_decrease,
    log_section_height_increase,
    log_section_toggle,
    quit,
    save_logs,
    scroll_down,
    scroll_end,
    scroll_start,
    scroll_up,
    select_next_panel,
    select_previous_panel,
    sort_by_cpu,
    sort_by_id,
    sort_by_image,
    sort_by_memory,
    sort_by_name,
    sort_by_rx,
    sort_by_state,
    sort_by_status,
    sort_by_tx,
    sort_reset,
    toggle_help,
    toggle_mouse_capture
);

impl Keymap {
    pub const fn new() -> Self {
        Self {
            clear: (KeyCode::Char('c'), Some(KeyCode::Esc)),
            delete_confirm: (KeyCode::Char('y'), None),
            delete_deny: (KeyCode::Char('n'), None),
            exec: (KeyCode::Char('e'), None),
            filter_mode: (KeyCode::Char('/'), Some(KeyCode::F(1))),
            force_redraw: (KeyCode::Char('f'), None),
            log_scroll_back: (KeyCode::Left, None),
            log_scroll_forward: (KeyCode::Right, None),
            log_search_mode: (KeyCode::Char('#'), None),
            log_section_height_decrease: (KeyCode::Char('-'), None),
            log_section_height_increase: (KeyCode::Char('='), None),
            log_section_toggle: (KeyCode::Char('\\'), None),
            quit: (KeyCode::Char('q'), None),
            save_logs: (KeyCode::Char('s'), None),
            scroll_down: (KeyCode::Down, Some(KeyCode::Char('j'))),
            scroll_end: (KeyCode::End, None),
            scroll_many: KeyModifiers::CONTROL,
            scroll_start: (KeyCode::Home, None),
            scroll_up: (KeyCode::Up, Some(KeyCode::Char('k'))),
            select_next_panel: (KeyCode::Tab, None),
            select_previous_panel: (KeyCode::BackTab, None),
            sort_by_cpu: (KeyCode::Char('4'), None),
            sort_by_id: (KeyCode::Char('6'), None),
            sort_by_image: (KeyCode::Char('7'), None),
            sort_by_memory: (KeyCode::Char('5'), None),
            sort_by_name: (KeyCode::Char('1'), None),
            sort_by_rx: (KeyCode::Char('8'), None),
            sort_by_state: (KeyCode::Char('2'), None),
            sort_by_status: (KeyCode::Char('3'), None),
            sort_by_tx: (KeyCode::Char('9'), None),
            sort_reset: (KeyCode::Char('0'), None),
            toggle_help: (KeyCode::Char('h'), None),
            toggle_mouse_capture: (KeyCode::Char('m'), None),
        }
    }
}

impl From<Option<ConfigKeymap>> for Keymap {
    /// Probably a better way to do this, but for now it works
    fn from(value: Option<ConfigKeymap>) -> Self {
        let mut keymap = Self::new();

        let mut clash = HashSet::new();
        let mut counter = 0;

        let mut update_keymap =
            |vec_str: Option<Vec<String>>,
             keymap_field: &mut (KeyCode, Option<KeyCode>),
             keymap_clash: &mut HashSet<KeyCode>| {
                if let Some(vec_str) = vec_str {
                    if let Some(vec_keycode) = Self::try_parse_keycode(&vec_str) {
                        if let Some(first) = vec_keycode.first() {
                            keymap_clash.insert(*first);
                            counter += 1;
                            keymap_field.0 = *first;
                        }
                        if let Some(second) = vec_keycode.get(1) {
                            keymap_clash.insert(*second);
                            counter += 1;
                            keymap_field.1 = Some(*second);
                        } else {
                            keymap_field.1 = None;
                        }
                    }
                }
            };

        if let Some(ck) = value {
            update_keymap(ck.clear, &mut keymap.clear, &mut clash);
            update_keymap(ck.delete_deny, &mut keymap.delete_deny, &mut clash);
            update_keymap(ck.delete_confirm, &mut keymap.delete_confirm, &mut clash);
            update_keymap(
                ck.log_section_height_decrease,
                &mut keymap.log_section_height_decrease,
                &mut clash,
            );
            update_keymap(
                ck.log_section_height_increase,
                &mut keymap.log_section_height_increase,
                &mut clash,
            );
            update_keymap(
                ck.log_section_toggle,
                &mut keymap.log_section_toggle,
                &mut clash,
            );

            update_keymap(ck.exec, &mut keymap.exec, &mut clash);
            update_keymap(ck.filter_mode, &mut keymap.filter_mode, &mut clash);
            update_keymap(ck.force_redraw, &mut keymap.force_redraw, &mut clash);
            update_keymap(ck.quit, &mut keymap.quit, &mut clash);
            update_keymap(ck.save_logs, &mut keymap.save_logs, &mut clash);
            update_keymap(ck.scroll_down, &mut keymap.scroll_down, &mut clash);
            update_keymap(ck.scroll_end, &mut keymap.scroll_end, &mut clash);
            update_keymap(ck.scroll_start, &mut keymap.scroll_start, &mut clash);
            update_keymap(ck.scroll_up, &mut keymap.scroll_up, &mut clash);
            update_keymap(ck.log_search_mode, &mut keymap.log_search_mode, &mut clash);
            update_keymap(
                ck.log_scroll_forward,
                &mut keymap.log_scroll_forward,
                &mut clash,
            );
            update_keymap(ck.log_scroll_back, &mut keymap.log_scroll_back, &mut clash);
            update_keymap(
                ck.select_next_panel,
                &mut keymap.select_next_panel,
                &mut clash,
            );
            update_keymap(
                ck.select_previous_panel,
                &mut keymap.select_previous_panel,
                &mut clash,
            );
            update_keymap(ck.sort_by_name, &mut keymap.sort_by_name, &mut clash);
            update_keymap(ck.sort_by_state, &mut keymap.sort_by_state, &mut clash);
            update_keymap(ck.sort_by_status, &mut keymap.sort_by_status, &mut clash);
            update_keymap(ck.sort_by_cpu, &mut keymap.sort_by_cpu, &mut clash);
            update_keymap(ck.sort_by_memory, &mut keymap.sort_by_memory, &mut clash);
            update_keymap(ck.sort_by_id, &mut keymap.sort_by_id, &mut clash);
            update_keymap(ck.sort_by_image, &mut keymap.sort_by_image, &mut clash);
            update_keymap(ck.sort_by_rx, &mut keymap.sort_by_rx, &mut clash);
            update_keymap(ck.sort_by_tx, &mut keymap.sort_by_tx, &mut clash);
            update_keymap(ck.sort_reset, &mut keymap.sort_reset, &mut clash);
            update_keymap(ck.toggle_help, &mut keymap.toggle_help, &mut clash);
            update_keymap(
                ck.toggle_mouse_capture,
                &mut keymap.toggle_mouse_capture,
                &mut clash,
            );
            // TODO need to check for clashes when using additional modifiers
            if let Some(scroll_many) = Self::try_parse_modifier(ck.scroll_many) {
                keymap.scroll_many = scroll_many;
            }
        }
        // A very basic clash check, every key has been inserted into a hashset, and a counter has been increased
        // if the counter and hashet length don't match, then there's a clash, and we just return the default keymap
        if counter == clash.len() {
            keymap
        } else {
            Self::new()
        }
    }
}

impl Keymap {
    // Allowable key modifiers are only `shift`, `control`, `alt`
    fn try_parse_modifier(input: Option<Vec<String>>) -> Option<KeyModifiers> {
        input.and_then(|input| {
            input
                .first()
                .and_then(|input| match input.to_lowercase().trim() {
                    "control" => Some(KeyModifiers::CONTROL),
                    "alt" => Some(KeyModifiers::ALT),
                    "shift" => Some(KeyModifiers::SHIFT),
                    _ => None,
                })
        })
    }

    /// Try to parse a &[String] into a Vec of keycodes, at most the output will have 2 entries
    /// This might fail on MacOS due to Backspace and Delete working in a different manner as to how they work on Linux & Windows
    /// I think that on MacOS `Del` becomes `Fwd Del`, and `Backspace` becomes `Delete`
    fn try_parse_keycode(input: &[String]) -> Option<Vec<KeyCode>> {
        let mut output = vec![];

        for key in input.iter().take(2) {
            if key.chars().count() == 1 {
                if let Some(first_char) = key.chars().next() {
                    if let Some(first_char) = match first_char {
                        x if x.is_ascii_alphabetic() || x.is_ascii_digit() => Some(first_char),
                        '/' | '\\' | ',' | '.' | '#' | '\'' | '[' | ']' | ';' | '=' | '-' => {
                            Some(first_char)
                        }
                        _ => None,
                    } {
                        output.push(KeyCode::Char(first_char));
                    }
                }
            } else {
                let keycode = match key.to_lowercase().as_str() {
                    "f1" => Some(KeyCode::F(1)),
                    "f2" => Some(KeyCode::F(2)),
                    "f3" => Some(KeyCode::F(3)),
                    "f4" => Some(KeyCode::F(4)),
                    "f5" => Some(KeyCode::F(5)),
                    "f6" => Some(KeyCode::F(6)),
                    "f7" => Some(KeyCode::F(7)),
                    "f8" => Some(KeyCode::F(8)),
                    "f9" => Some(KeyCode::F(9)),
                    "f10" => Some(KeyCode::F(10)),
                    "f11" => Some(KeyCode::F(11)),
                    "f12" => Some(KeyCode::F(12)),
                    // Might fail on MacOS, see note above
                    "backspace" => Some(KeyCode::Backspace),
                    "backtab" => Some(KeyCode::BackTab),
                    // Might fail on MacOS, see note above
                    "delete" => Some(KeyCode::Delete),
                    "down" => Some(KeyCode::Down),
                    "end" => Some(KeyCode::End),
                    "esc" => Some(KeyCode::Esc),
                    "home" => Some(KeyCode::Home),
                    "insert" => Some(KeyCode::Insert),
                    "left" => Some(KeyCode::Left),
                    "pagedown" => Some(KeyCode::PageDown),
                    "pageup" => Some(KeyCode::PageUp),
                    "right" => Some(KeyCode::Right),
                    "tab" => Some(KeyCode::Tab),
                    "up" => Some(KeyCode::Up),
                    _ => None,
                };
                if let Some(a) = keycode {
                    output.push(a);
                }
            }
        }
        if output.is_empty() {
            None
        } else {
            // Remove any duplicates for a single deinition
            if output.first() == output.get(1) {
                output.pop();
            }
            Some(output)
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use crossterm::event::{KeyCode, KeyModifiers};

    use crate::config::keymap_parser::ConfigKeymap;

    use super::Keymap;

    #[test]
    /// Only allow two definitions to be parsed
    fn test_return_max_two() {
        let result = Keymap::try_parse_keycode(&["a".to_owned(), "b".to_owned(), "c".to_owned()]);
        assert_eq!(result, Some(vec![KeyCode::Char('a'), KeyCode::Char('b')]));

        let result = Keymap::try_parse_keycode(&["0".to_owned(), "1".to_owned(), "2".to_owned()]);
        assert_eq!(result, Some(vec![KeyCode::Char('0'), KeyCode::Char('1')]));

        let result =
            Keymap::try_parse_keycode(&["esc".to_owned(), "tab".to_owned(), "backtab".to_owned()]);
        assert_eq!(result, Some(vec![KeyCode::Esc, KeyCode::Tab]));
    }

    #[test]
    /// If a single definition has two identical entries, just return a single entry
    fn test_duplicate_definition() {
        let result = Keymap::try_parse_keycode(&["c".to_owned(), "c".to_owned()]);
        assert_eq!(result, Some(vec![KeyCode::Char('c')]));

        let result = Keymap::try_parse_keycode(&["0".to_owned(), "0".to_owned()]);
        assert_eq!(result, Some(vec![KeyCode::Char('0')]));

        let result = Keymap::try_parse_keycode(&["esc".to_owned(), "esc".to_owned()]);
        assert_eq!(result, Some(vec![KeyCode::Esc]));
    }

    #[test]
    /// Return None is invalid key definition is provided
    fn test_invalid_key() {
        let result = Keymap::try_parse_keycode(&["(".to_owned(), "*".to_owned()]);
        assert!(result.is_none());

        let result = Keymap::try_parse_keycode(&["enter".to_owned(), "shift".to_owned()]);
        assert!(result.is_none());

        let result = Keymap::try_parse_keycode(&["ö".to_owned(), "ä".to_owned()]);
        assert!(result.is_none());
    }

    #[test]
    /// If any key definitions clash, just return the default keymap
    fn test_clash_returns_default() {
        let input = ConfigKeymap {
            clear: Some(vec!["s".to_owned()]),
            delete_deny: Some(vec!["s".to_owned()]),
            delete_confirm: None,
            exec: None,
            filter_mode: None,
            force_redraw: None,
            log_scroll_back: None,
            log_search_mode: None,
            log_scroll_forward: None,
            log_section_height_decrease: None,
            log_section_height_increase: None,
            log_section_toggle: None,
            quit: None,
            save_logs: None,
            scroll_down: None,
            scroll_end: None,
            scroll_start: None,
            scroll_many: None,
            scroll_up: None,
            select_next_panel: None,
            select_previous_panel: None,
            sort_by_cpu: None,
            sort_by_id: None,
            sort_by_image: None,
            sort_by_memory: None,
            sort_by_name: None,
            sort_by_rx: None,
            sort_by_state: None,
            sort_by_status: None,
            sort_by_tx: None,
            sort_reset: None,
            toggle_help: None,
            toggle_mouse_capture: None,
        };

        let result = Keymap::from(Some(input));

        assert_eq!(result, Keymap::new());
    }

    #[test]
    /// Custom keymap definition creation
    fn test_valid_custom_keymap() {
        let gen_v = |a: (&str, &str)| Some(vec![a.0.to_owned(), a.1.to_owned()]);

        let input = ConfigKeymap {
            clear: gen_v(("a", "b")),
            delete_confirm: gen_v(("c", "d")),
            delete_deny: gen_v(("e", "f")),
            exec: gen_v(("g", "h")),
            filter_mode: gen_v(("i", "j")),
            force_redraw: gen_v(("k", "l")),
            log_scroll_back: gen_v(("s", "t")),
            log_scroll_forward: gen_v(("q", "r")),
            log_search_mode: gen_v(("1", "2")),
            log_section_height_decrease: gen_v(("m", "n")),
            log_section_height_increase: gen_v(("o", "p")),
            log_section_toggle: gen_v(("u", "v")),
            quit: gen_v(("w", "x")),
            save_logs: gen_v(("y", "z")),
            scroll_down: gen_v(("3", "4")),
            scroll_end: gen_v(("5", "6")),
            scroll_many: Some(vec!["alt".to_owned()]),
            scroll_start: gen_v(("7", "8")),
            scroll_up: gen_v(("F1", "F2")),
            select_next_panel: gen_v(("F3", "F4")),
            select_previous_panel: gen_v(("F5", "F6")),
            sort_by_cpu: gen_v(("F7", "F8")),
            sort_by_id: gen_v(("F9", "F10")),
            sort_by_image: gen_v(("F11", "F12")),
            sort_by_memory: gen_v(("HOME", "END")),
            sort_by_name: gen_v(("UP", "DOWN")),
            sort_by_rx: gen_v(("LEFT", "RIGHT")),
            sort_by_state: gen_v(("[", "]")),
            sort_by_status: gen_v(("INSERTt", "TAB")),
            sort_by_tx: gen_v(("PAGEDOWN", "PAGEUP")),
            sort_reset: gen_v((",", ".")),
            toggle_help: gen_v(("-", "=")),
            toggle_mouse_capture: gen_v(("\\", "/")),
        };

        let result = Keymap::from(Some(input));

        let expected = Keymap {
            clear: (KeyCode::Char('a'), Some(KeyCode::Char('b'))),
            delete_confirm: (KeyCode::Char('c'), Some(KeyCode::Char('d'))),
            delete_deny: (KeyCode::Char('e'), Some(KeyCode::Char('f'))),
            exec: (KeyCode::Char('g'), Some(KeyCode::Char('h'))),
            filter_mode: (KeyCode::Char('i'), Some(KeyCode::Char('j'))),
            force_redraw: (KeyCode::Char('k'), Some(KeyCode::Char('l'))),
            log_scroll_back: (KeyCode::Char('s'), Some(KeyCode::Char('t'))),
            log_scroll_forward: (KeyCode::Char('q'), Some(KeyCode::Char('r'))),
            log_search_mode: (KeyCode::Char('1'), Some(KeyCode::Char('2'))),
            log_section_height_decrease: (KeyCode::Char('m'), Some(KeyCode::Char('n'))),
            log_section_height_increase: (KeyCode::Char('o'), Some(KeyCode::Char('p'))),
            log_section_toggle: (KeyCode::Char('u'), Some(KeyCode::Char('v'))),
            quit: (KeyCode::Char('w'), Some(KeyCode::Char('x'))),
            save_logs: (KeyCode::Char('y'), Some(KeyCode::Char('z'))),
            scroll_down: (KeyCode::Char('3'), Some(KeyCode::Char('4'))),
            scroll_end: (KeyCode::Char('5'), Some(KeyCode::Char('6'))),
            scroll_many: KeyModifiers::ALT,
            scroll_start: (KeyCode::Char('7'), Some(KeyCode::Char('8'))),
            scroll_up: (KeyCode::F(1), Some(KeyCode::F(2))),
            select_next_panel: (KeyCode::F(3), Some(KeyCode::F(4))),
            select_previous_panel: (KeyCode::F(5), Some(KeyCode::F(6))),
            sort_by_cpu: (KeyCode::F(7), Some(KeyCode::F(8))),
            sort_by_id: (KeyCode::F(9), Some(KeyCode::F(10))),
            sort_by_image: (KeyCode::F(11), Some(KeyCode::F(12))),
            sort_by_memory: (KeyCode::Home, Some(KeyCode::End)),
            sort_by_name: (KeyCode::Up, Some(KeyCode::Down)),
            sort_by_rx: (KeyCode::Left, Some(KeyCode::Right)),
            sort_by_state: (KeyCode::Char('['), Some(KeyCode::Char(']'))),
            sort_by_status: (KeyCode::Tab, None),
            sort_by_tx: (KeyCode::PageDown, Some(KeyCode::PageUp)),
            sort_reset: (KeyCode::Char(','), Some(KeyCode::Char('.'))),
            toggle_help: (KeyCode::Char('-'), Some(KeyCode::Char('='))),
            toggle_mouse_capture: (KeyCode::Char('\\'), Some(KeyCode::Char('/'))),
        };
        assert_eq!(expected, result);
    }
}
