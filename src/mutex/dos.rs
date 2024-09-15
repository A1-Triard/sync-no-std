use core::sync::atomics::AtomicBool;

pub struct SysMutex(AtomicBool);

impl SysMutex {
    pub fn new() -> Self {
        SysMutex(AtomicBool::new(false))
    }

    pub unsafe fn lock(&self) {
        assert_eq!(self.locked.replace(true), false, "cannot recursively acquire mutex");
    }

    pub unsafe fn unlock(&self) {
        self.locked.set(false);
    }

    pub unsafe fn try_lock(&self) -> bool {
        self.locked.replace(true) == false
    }
}
