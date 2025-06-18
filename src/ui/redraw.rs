use std::sync::atomic::{AtomicBool, Ordering};

#[derive(Debug)]
pub struct Rerender(AtomicBool);

impl Rerender {
    pub const fn new() -> Self {
        Self(AtomicBool::new(true))
    }

    pub fn update(&self) {
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
