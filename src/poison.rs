use core::error::Error;
use core::fmt;
#[cfg(panic="unwind")]
use core::sync::atomic::{AtomicBool, Ordering};
#[cfg(panic="unwind")]
use panicking::panicking;

pub struct Flag {
    #[cfg(panic="unwind")]
    failed: AtomicBool,
}

// Note that the Ordering uses to access the `failed` field of `Flag` below is
// always `Relaxed`, and that's because this isn't actually protecting any data,
// it's just a flag whether we've panicked or not.
//
// The actual location that this matters is when a mutex is **locked** which is
// where we have external synchronization ensuring that we see memory
// reads/writes to this flag.
//
// As a result, if it matters, we should see the correct value for `failed` in
// all cases.

impl Flag {
    pub const fn new() -> Flag {
        Flag {
            #[cfg(panic="unwind")]
            failed: AtomicBool::new(false),
        }
    }

    pub fn borrow(&self) -> LockResult<()> {
        if self.get() { Err(PoisonError::new(())) } else { Ok(()) }
    }

    pub fn guard(&self) -> LockResult<Guard> {
        let ret = Guard {
            #[cfg(panic="unwind")]
            panicking: panicking(),
        };
        if self.get() { Err(PoisonError::new(ret)) } else { Ok(ret) }
    }

    #[cfg(panic="unwind")]
    pub fn done(&self, guard: &Guard) {
        if !guard.panicking && panicking() {
            self.failed.store(true, Ordering::Relaxed);
        }
    }

    #[cfg(not(panic="unwind"))]
    pub fn done(&self, _guard: &Guard) {}

    #[cfg(panic="unwind")]
    pub fn get(&self) -> bool {
        self.failed.load(Ordering::Relaxed)
    }

    #[cfg(not(panic="unwind"))]
    pub fn get(&self) -> bool {
        false
    }

    pub fn clear(&self) {
        #[cfg(panic="unwind")]
        self.failed.store(false, Ordering::Relaxed)
    }
}

#[derive(Clone)]
pub struct Guard {
    #[cfg(panic="unwind")]
    panicking: bool,
}

pub struct PoisonError<T> {
    guard: T,
    #[cfg(not(panic="unwind"))]
    _never: !,
}

pub enum TryLockError<T> {
    Poisoned(PoisonError<T>),
    WouldBlock,
}

pub type LockResult<Guard> = Result<Guard, PoisonError<Guard>>;

pub type TryLockResult<Guard> = Result<Guard, TryLockError<Guard>>;

impl<T> fmt::Debug for PoisonError<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PoisonError").finish_non_exhaustive()
    }
}

impl<T> fmt::Display for PoisonError<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        "poisoned lock: another task failed inside".fmt(f)
    }
}

impl<T> Error for PoisonError<T> { }

impl<T> PoisonError<T> {
    #[cfg(panic="unwind")]
    pub fn new(guard: T) -> PoisonError<T> {
        PoisonError { guard }
    }

    #[cfg(not(panic="unwind"))]
    pub fn new(_guard: T) -> PoisonError<T> {
        panic!("PoisonError created in a libstd built with panic=\"abort\"")
    }

    pub fn into_inner(self) -> T {
        self.guard
    }

    pub fn get_ref(&self) -> &T {
        &self.guard
    }

    pub fn get_mut(&mut self) -> &mut T {
        &mut self.guard
    }
}

impl<T> From<PoisonError<T>> for TryLockError<T> {
    fn from(err: PoisonError<T>) -> TryLockError<T> {
        TryLockError::Poisoned(err)
    }
}

impl<T> fmt::Debug for TryLockError<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            #[cfg(panic="unwind")]
            TryLockError::Poisoned(..) => "Poisoned(..)".fmt(f),
            #[cfg(not(panic="unwind"))]
            TryLockError::Poisoned(ref p) => match p._never {},
            TryLockError::WouldBlock => "WouldBlock".fmt(f),
        }
    }
}

impl<T> fmt::Display for TryLockError<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            #[cfg(panic="unwind")]
            TryLockError::Poisoned(..) => "poisoned lock: another task failed inside",
            #[cfg(not(panic="unwind"))]
            TryLockError::Poisoned(ref p) => match p._never {},
            TryLockError::WouldBlock => "try_lock failed because the operation would block",
        }
        .fmt(f)
    }
}

impl<T> Error for TryLockError<T> { }

pub fn map_result<T, U, F>(result: LockResult<T>, f: F) -> LockResult<U>
where
    F: FnOnce(T) -> U,
{
    match result {
        Ok(t) => Ok(f(t)),
        #[cfg(panic="unwind")]
        Err(PoisonError { guard }) => Err(PoisonError::new(f(guard))),
    }
}
