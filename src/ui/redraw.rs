use std::sync::atomic::{AtomicBool, Ordering};

#[derive(Debug)]
pub struct Rerender {
    draw: AtomicBool,
    clear: AtomicBool,
}

impl Rerender {
    pub const fn new() -> Self {
        Self {
            draw: AtomicBool::new(true),
            clear: AtomicBool::new(false),
        }
    }

    pub fn update_draw(&self) {
        self.draw.store(true, Ordering::SeqCst);
    }

    pub fn get_clear(&self) -> bool {
        if self.clear.load(Ordering::SeqCst) {
            self.clear.store(false, Ordering::SeqCst);
            true
        } else {
            false
        }
    }

    pub fn set_clear(&self) {
        self.clear.store(true, Ordering::SeqCst);
    }

    /// Return the value of the draw, and set to false
    pub fn swap_draw(&self) -> bool {
        match self
            .draw
            .compare_exchange(true, false, Ordering::SeqCst, Ordering::SeqCst)
        {
            Ok(previous_value) => previous_value,
            Err(current_value) => current_value,
        }
    }
}
