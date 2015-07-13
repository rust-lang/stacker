extern crate stacker;

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
