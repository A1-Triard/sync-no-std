use core::sync::atomics::AtomicBool;

pub struct SysMutex(AtomicBool);

#[allow(clippy::missing_safety_doc)]
#[allow(clippy::new_without_default)]
impl SysMutex {
    pub fn new() -> Self {
        SysMutex(AtomicBool::new(false))
    }

    pub unsafe fn lock(&self) {
        assert_eq!(self.0.replace(true), false, "cannot recursively acquire mutex");
    }

    pub unsafe fn unlock(&self) {
        self.0.set(false);
    }

    pub unsafe fn try_lock(&self) -> bool {
        self.0.replace(true) == false
    }
}
