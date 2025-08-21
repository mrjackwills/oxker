use crossterm::event::{KeyCode, KeyModifiers, MouseEvent};

#[derive(Debug, Clone, Copy)]
pub enum InputMessages {
    ButtonPress((KeyCode, KeyModifiers)),
    MouseEvent((MouseEvent, KeyModifiers)),
}
