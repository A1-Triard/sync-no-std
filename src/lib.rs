#![feature(negative_impls)]
#![feature(never_type)]

#![deny(warnings)]

#![no_std]

mod poison;

pub use poison::{LockResult, TryLockError, TryLockResult};

pub mod mutex;
