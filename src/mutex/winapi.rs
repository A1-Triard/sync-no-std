// The Windows implementation of mutexes is a little odd and it might not be
// immediately obvious what's going on. The primary oddness is that SRWLock is
// used instead of CriticalSection, and this is done because:
//
// 1. SRWLock is several times faster than CriticalSection according to
//    benchmarks performed on both Windows 8 and Windows 7.
//
// 2. CriticalSection allows recursive locking while SRWLock deadlocks. The
//    Unix implementation deadlocks so consistency is preferred. See #19962 for
//    more details.
//
// 3. While CriticalSection is fair and SRWLock is not, the current Rust policy
//    is that there are no guarantees of fairness.

use core::alloc::Allocator;
use core::cell::UnsafeCell;
use winapi::um::synchapi::{SRWLOCK, SRWLOCK_INIT};
use winapi::um::synchapi::{AcquireSRWLockExclusive, TryAcquireSRWLockExclusive, ReleaseSRWLockExclusive};

pub struct SysMutex<A: Allocator + Clone>(UnsafeCell<SRWLOCK>, A);

unsafe impl<A: Allocator + Clone> Send for SysMutex<A> { }

unsafe impl<A: Allocator + Clone> Sync for SysMutex<A> { }

#[allow(clippy::missing_safety_doc)]
#[allow(clippy::new_without_default)]
impl<A: Allocator + Clone> SysMutex<A> {
    pub const fn new_in(allocator: A) -> Self {
        SysMutex(UnsafeCell::new(SRWLOCK_INIT), allocator)
    }

    pub unsafe fn lock(&self) {
        AcquireSRWLockExclusive(self.0.get());
    }

    pub unsafe fn try_lock(&self) -> bool {
        TryAcquireSRWLockExclusive(self.0.get()) != 0
    }

    pub unsafe fn unlock(&self) {
        ReleaseSRWLockExclusive(self.0.get());
    }
}
