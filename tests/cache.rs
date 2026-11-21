#![cfg(not(any(target_arch = "wasm32", windows, miri)))]

extern crate stacker;

use std::thread;

#[inline(never)]
fn deep_marker() -> *mut u8 {
    let local = 0u8;
    let bottom = (&local as *const u8 as usize - stacker::remaining_stack().unwrap()) & !4095;
    (bottom + 64) as *mut u8
}

#[test]
fn cached_stack_is_reused() {
    stacker::set_cached_stacks(1);
    thread::spawn(|| {
        let written = stacker::grow(1024 * 1024, || {
            let marker = deep_marker();
            unsafe { marker.write_volatile(0xab) };
            marker as usize
        });
        let (read, value) = stacker::grow(1024 * 1024, || {
            let marker = deep_marker();
            (marker as usize, unsafe { marker.read_volatile() })
        });
        assert_eq!(written, read);
        assert_eq!(value, 0xab);
    })
    .join()
    .unwrap();
}
