use std::sync::atomic::{AtomicBool, Ordering};

#[derive(Debug)]
pub struct Redraw(AtomicBool);

impl Redraw {
    pub const fn new() -> Self {
        Self(AtomicBool::new(true))
    }

    pub fn set_true(&self) {
        self.0.store(true, Ordering::SeqCst);
    }

    /// Return the value of the self, and set to false
    pub fn swap(&self) -> bool {
        match self
            .0
            .compare_exchange(true, false, Ordering::SeqCst, Ordering::SeqCst)
        {
            Ok(previous_value) => previous_value,
            Err(current_value) => current_value,
        }
    }
}
