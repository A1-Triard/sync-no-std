use core::cell::UnsafeCell;
use core::mem::MaybeUninit;
use libc::{EBUSY, pthread_mutex_t, pthread_mutex_init, pthread_mutex_destroy, pthread_mutex_trylock};
use libc::{PTHREAD_MUTEX_NORMAL, pthread_mutex_lock, pthread_mutex_unlock};
use libc::{pthread_mutexattr_t, pthread_mutexattr_init, pthread_mutexattr_destroy, pthread_mutexattr_settype};
use panicking::panicking;

pub struct SysMutex(UnsafeCell<pthread_mutex_t>);

unsafe impl Send for SysMutex { }

unsafe impl Sync for SysMutex { }

impl Drop for SysMutex {
    fn drop(&mut self) {
        // We're not allowed to pthread_mutex_destroy a locked mutex,
        // so check first if it's unlocked.
        // If the mutex is locked (this happens if a MutexGuard is leaked), we just leak the Mutex too.
        if unsafe { pthread_mutex_trylock(self.0.get()) == 0 } {
            unsafe { pthread_mutex_unlock(self.0.get()) };
            let r = unsafe { pthread_mutex_destroy(self.0.get()) };
            if !panicking() {
                assert_eq!(r, 0);
            }
        }
    }
}

impl SysMutex {
    pub fn new() -> Self {
        let mut attr = MaybeUninit::uninit();
        let r = unsafe { pthread_mutexattr_init(attr.as_mut_ptr()) };
        assert_eq!(r, 0);
        let mut attr = PthreadMutexAttr(unsafe { attr.assume_init() });
        let r = unsafe { pthread_mutexattr_settype(&mut attr.0 as *mut _, PTHREAD_MUTEX_NORMAL) };
        assert_eq!(r, 0);
        let mut mutex = MaybeUninit::uninit();
        unsafe { pthread_mutex_init(mutex.as_mut_ptr(), &mut attr.0 as *mut _) };
        SysMutex(UnsafeCell::new(unsafe { mutex.assume_init() }))
    }

    pub unsafe fn lock(&self) {
        let r = pthread_mutex_lock(self.0.get());
        assert_eq!(r, 0);
    }

    pub unsafe fn unlock(&self) {
        let r = pthread_mutex_unlock(self.0.get());
        assert_eq!(r, 0);
    }

    pub unsafe fn try_lock(&self) -> bool {
        let r = pthread_mutex_trylock(self.0.get());
        if r == EBUSY { return false; }
        assert_eq!(r, 0);
        true
    }
}

struct PthreadMutexAttr(pthread_mutexattr_t);

impl Drop for PthreadMutexAttr {
    fn drop(&mut self) {
        let r = unsafe { pthread_mutexattr_destroy(&mut self.0 as *mut _) };
        if !panicking() {
            assert_eq!(r, 0);
        }
    }
}
