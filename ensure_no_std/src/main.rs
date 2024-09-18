#![feature(start)]

#![deny(warnings)]

#![no_std]

extern crate alloc;

use composable_allocators::{AsGlobal, System};
use core::panic::PanicInfo;
use panic_no_std::panic;

#[cfg(windows)]
#[link(name="msvcrt")]
extern { }

#[panic_handler]
fn panic_handler(info: &PanicInfo) -> ! {
    panic(info, 99)
}

#[no_mangle]
extern "C" fn rust_eh_personality() { }

#[global_allocator]
static ALLOCATOR: AsGlobal<System> = AsGlobal(System);

use sync_no_std::mutex::Mutex;

#[start]
pub fn main(_argc: isize, _argv: *const *const u8) -> isize {
    let mutex = Mutex::new(0_i32);
    {
        let mut lock = mutex.lock().unwrap();
        *lock = 1;
    }
    {
        let lock = mutex.lock().unwrap();
        assert_eq!(*lock, 1);
    }
    0
}
