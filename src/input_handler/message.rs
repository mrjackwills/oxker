use crossterm::event::{KeyCode, MouseEvent};

#[derive(Debug, Clone)]
pub enum InputMessages {
    ButtonPress(KeyCode),
    MouseEvent(MouseEvent),
}
