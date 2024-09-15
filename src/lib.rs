#![feature(negative_impls)]

#![no_std]

mod poison;

pub use poison::{LockResult, TryLockError, TryLockResult};

pub mod mutex;
