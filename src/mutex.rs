use alloc::alloc::Global;
use core::alloc::Allocator;
use core::cell::UnsafeCell;
use core::fmt;
use core::ops::{Deref, DerefMut};
use crate::poison::{self, LockResult, TryLockError, TryLockResult};

#[cfg(all(not(target_os="dos"), not(windows)))]
mod posix;

#[cfg(all(not(target_os="dos"), windows))]
mod winapi;

#[cfg(target_os="dos")]
mod dos;

#[cfg(all(not(target_os="dos"), not(windows)))]
use posix::SysMutex;

#[cfg(all(not(target_os="dos"), windows))]
use winapi::SysMutex;

#[cfg(target_os="dos")]
use dos::SysMutex;

pub struct Mutex<T: ?Sized, A: Allocator + Clone = Global> {
    inner: SysMutex<A>,
    poison: poison::Flag,
    data: UnsafeCell<T>,
}

unsafe impl<T: ?Sized + Send, A: Allocator + Clone> Send for Mutex<T, A> { }

unsafe impl<T: ?Sized + Send, A: Allocator + Clone> Sync for Mutex<T, A> { }

#[must_use="if unused the Mutex will immediately unlock"]
pub struct MutexGuard<'a, T: ?Sized + 'a, A: Allocator + Clone = Global> {
    lock: &'a Mutex<T, A>,
    poison: poison::Guard,
}

impl<T: ?Sized, A: Allocator + Clone> !Send for MutexGuard<'_, T, A> { }

unsafe impl<T: ?Sized + Sync, A: Allocator + Clone> Sync for MutexGuard<'_, T, A> { }

impl<T, A: Allocator + Clone> Mutex<T, A> {
    pub const fn new_in(t: T, allocator: A) -> Mutex<T, A> {
        Mutex { inner: SysMutex::new_in(allocator), poison: poison::Flag::new(), data: UnsafeCell::new(t) }
    }
}

impl<T> Mutex<T> {
    pub const fn new(t: T) -> Mutex<T> {
        Mutex::new_in(t, Global)
    }
}

impl<T: ?Sized, A: Allocator + Clone> Mutex<T, A> {
    pub fn lock(&self) -> LockResult<MutexGuard<'_, T, A>> {
        unsafe {
            self.inner.lock();
            MutexGuard::new(self)
        }
    }

    pub fn try_lock(&self) -> TryLockResult<MutexGuard<'_, T, A>> {
        unsafe {
            if self.inner.try_lock() {
                Ok(MutexGuard::new(self)?)
            } else {
                Err(TryLockError::WouldBlock)
            }
        }
    }

    pub fn is_poisoned(&self) -> bool {
        self.poison.get()
    }

    pub fn clear_poison(&self) {
        self.poison.clear();
    }

    pub fn into_inner(self) -> LockResult<T>
    where
        T: Sized,
    {
        let data = self.data.into_inner();
        poison::map_result(self.poison.borrow(), |()| data)
    }

    pub fn get_mut(&mut self) -> LockResult<&mut T> {
        let data = self.data.get_mut();
        poison::map_result(self.poison.borrow(), |()| data)
    }
}

impl<T> From<T> for Mutex<T> {
    fn from(t: T) -> Self {
        Mutex::new(t)
    }
}

impl<T: Default> Default for Mutex<T> {
    fn default() -> Mutex<T> {
        Mutex::new(Default::default())
    }
}

impl<T: ?Sized + fmt::Debug, A: Allocator + Clone> fmt::Debug for Mutex<T, A> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut d = f.debug_struct("Mutex");
        match self.try_lock() {
            Ok(guard) => {
                d.field("data", &&*guard);
            }
            Err(TryLockError::Poisoned(err)) => {
                d.field("data", &&**err.get_ref());
            }
            Err(TryLockError::WouldBlock) => {
                d.field("data", &format_args!("<locked>"));
            }
        }
        d.field("poisoned", &self.poison.get());
        d.finish_non_exhaustive()
    }
}

impl<'mutex, T: ?Sized, A: Allocator + Clone> MutexGuard<'mutex, T, A> {
    unsafe fn new(lock: &'mutex Mutex<T, A>) -> LockResult<MutexGuard<'mutex, T, A>> {
        poison::map_result(lock.poison.guard(), |guard| MutexGuard { lock, poison: guard })
    }
}

impl<T: ?Sized, A: Allocator + Clone> Deref for MutexGuard<'_, T, A> {
    type Target = T;

    fn deref(&self) -> &T {
        unsafe { &*self.lock.data.get() }
    }
}

impl<T: ?Sized, A: Allocator + Clone> DerefMut for MutexGuard<'_, T, A> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe { &mut *self.lock.data.get() }
    }
}

impl<T: ?Sized, A: Allocator + Clone> Drop for MutexGuard<'_, T, A> {
    fn drop(&mut self) {
        unsafe {
            self.lock.poison.done(&self.poison);
            self.lock.inner.unlock();
        }
    }
}

impl<T: ?Sized + fmt::Debug, A: Allocator + Clone> fmt::Debug for MutexGuard<'_, T, A> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&**self, f)
    }
}

impl<T: ?Sized + fmt::Display, A: Allocator + Clone> fmt::Display for MutexGuard<'_, T, A> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        (**self).fmt(f)
    }
}
