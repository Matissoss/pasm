// pasm - src/shr/semaphore.rs
// ---------------------------
// made by matissoss
// licensed under MPL 2.0

use std::sync::atomic::*;

#[repr(transparent)]
pub struct Semaphore {
    content: AtomicUsize,
}

impl Semaphore {
    pub const fn new(limit: usize) -> Self {
        Self {
            content: AtomicUsize::new(limit),
        }
    }
    pub fn acquire(&mut self) {
        loop {
            let val = self.content.load(Ordering::Acquire);
            if val > 0 {
                if self
                    .content
                    .compare_exchange(val, val - 1, Ordering::Acquire, Ordering::Relaxed)
                    .is_ok()
                {
                    break;
                }
            } else {
                std::thread::sleep(std::time::Duration::from_millis(crate::conf::RETRY_TIME_MS));
                continue;
            }
        }
    }
    pub fn release(&mut self) {
        self.content.fetch_add(1, Ordering::Release);
    }
}
