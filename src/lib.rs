#![feature(allocator_api)]
#![feature(negative_impls)]
#![feature(never_type)]

#![deny(warnings)]

#![no_std]

extern crate alloc;

mod lazy_box;

mod poison;

pub use poison::{LockResult, TryLockError, TryLockResult};

pub mod mutex;
