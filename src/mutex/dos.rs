use core::alloc::Allocator;
use core::sync::atomic::{AtomicBool, Ordering};

pub struct SysMutex<A: Allocator + Clone>(AtomicBool, A);

#[allow(clippy::missing_safety_doc)]
#[allow(clippy::new_without_default)]
impl<A: Allocator + Clone> SysMutex<A> {
    pub const fn new_in(allocator: A) -> Self {
        SysMutex(AtomicBool::new(false), allocator)
    }

    pub const fn allocator(&self) -> &A { &self.1 }

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
