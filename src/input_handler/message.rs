use crossterm::event::{KeyCode, MouseEvent};

#[derive(Debug, Clone, Copy)]
pub enum InputMessages {
    ButtonPress(KeyCode),
    MouseEvent(MouseEvent),
}
