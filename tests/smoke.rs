extern crate stacker;

use std::sync::mpsc;
use std::thread;

extern {
    fn __stacker_black_box(t: *const u8);
}

#[test]
fn deep() {
    fn foo(n: usize, s: &mut [u8]) {
        unsafe { __stacker_black_box(s.as_ptr()); }
        if n > 0 {
            stacker::maybe_grow(64 * 1024, 1024 * 1024, || {
                let mut s = [0u8; 1024];
                foo(n - 1, &mut s);
                unsafe { __stacker_black_box(s.as_ptr()); }
            })
        } else {
            println!("bottom");
        }
    }

    foo(256 * 1024, &mut []);
}

#[test]
#[ignore]
fn panic() {
    fn foo(n: usize, s: &mut [u8]) {
        unsafe { __stacker_black_box(s.as_ptr()); }
        if n > 0 {
            stacker::maybe_grow(64 * 1024, 1024 * 1024, || {
                let mut s = [0u8; 1024];
                foo(n - 1, &mut s);
                unsafe { __stacker_black_box(s.as_ptr()); }
            })
        } else {
            panic!("bottom");
        }
    }

    let (tx, rx) = mpsc::channel::<()>();
    thread::spawn(move || {
        foo(64 * 1024, &mut []);
        drop(tx);
    }).join().unwrap_err();

    assert!(rx.recv().is_err());
}
