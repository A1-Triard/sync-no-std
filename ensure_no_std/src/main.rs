#![deny(warnings)]

#![no_std]
#![no_main]

extern crate alloc;

use composable_allocators::{AsGlobal, System};
use core::panic::PanicInfo;
use panic_no_std::panic;

#[cfg(windows)]
#[link(name="msvcrt")]
extern "C" { }

#[panic_handler]
fn panic_handler(info: &PanicInfo) -> ! {
    panic(info, 99)
}

#[unsafe(no_mangle)]
extern "C" fn rust_eh_personality() { }

#[global_allocator]
static ALLOCATOR: AsGlobal<System> = AsGlobal(System);

use core::ffi::{c_char, c_int};
use sync_no_std::mutex::Mutex;

#[unsafe(no_mangle)]
extern "C" fn main(_argc: c_int, _argv: *mut *mut c_char) -> c_int {
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
