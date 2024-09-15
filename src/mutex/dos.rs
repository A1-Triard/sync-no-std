use core::sync::atomic::{AtomicBool, Ordering};

pub struct SysMutex(AtomicBool);

#[allow(clippy::missing_safety_doc)]
#[allow(clippy::new_without_default)]
impl SysMutex {
    pub fn new() -> Self {
        SysMutex(AtomicBool::new(false))
    }

    pub unsafe fn lock(&self) {
        let ok = self.0.compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed).is_ok();
        assert!(ok, "cannot recursively acquire mutex");
    }

    pub unsafe fn unlock(&self) {
        self.0.store(false, Ordering::Release);
    }

    pub unsafe fn try_lock(&self) -> bool {
        self.0.compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed).is_ok()
    }
}
