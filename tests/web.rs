#![cfg(all(target_arch = "wasm32", target_os = "unknown"))]

extern crate stacker;
extern crate wasm_bindgen_test;

use wasm_bindgen_test::*;
wasm_bindgen_test_configure!(run_in_browser);

extern "C" {
    // If you define this function like that:
    //
    // ```
    // #[inline(never)]
    // fn __stacker_black_box(_: *const u8) {}
    // ```
    //
    // LLVM will be able to figure out that it is being tricked 
    // and will compile out everything, thus making the stack frame really small.
    //
    // We use externally defined function here to avoid that.
    fn __stacker_black_box(_: *mut u8);
}

#[wasm_bindgen_test]
fn deep() {
    // Conventional platforms usually has a limited set of registers.
    // Wasm is different. It doesn't have registers but has a value stack and locals.
    // Although there is no fundamental limit on count of pushed values and used locals,
    // in practice they are constrained by implementation defined limits.
    // (In theory, they can vary depending on which tier was used for compiling your code
    // and can change during the lifetime of your program).
    //
    // Unfortunately, stacker doesn't help with the value stack problem. However, rustc still
    // places certain kinds of data on the shadow stack (i.e. in linear memory). And stacker
    // helps with that kind of problem.
    // 
    // Because of the problem described above we use a setup here which differs from
    // the ones that are used in tests for native platforms: we use more space
    // for a buffer, but limit the depth of recursion.

    const RED_ZONE: usize = 384 * 1024; // 384k
    const STACK_PER_RECURSION: usize = 1 * 1024 * 1024; // 1MB
    const ALLOCATION_SIZE: usize = 256 * 1024; // 256k
    const RECURSION_COUNT: usize = 1024;

    fn foo(n: usize, s: &mut [u8]) {
        unsafe { __stacker_black_box(s.as_mut_ptr()) };
        if n > 0 {
            stacker::maybe_grow(RED_ZONE, STACK_PER_RECURSION, || {
                let mut s = [42u8; ALLOCATION_SIZE];
                foo(n - 1, &mut s);
                unsafe { __stacker_black_box(s.as_mut_ptr()) };
            })
        } else {
            println!("bottom");
        }
    }

    foo(RECURSION_COUNT, &mut []);
}

// There are no tests for panics, because wasm32-unknown-unknown supports only panic=abort 
// for the moment.
