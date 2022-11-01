#![cfg(test)]

macro_rules! tests {
    ($($(#[$meta:meta])* fn $name:ident() { $($body:tt)* })*) => {
        $(
            $(#[$meta])*
            fn $name() {
                $($body)*
            }

        )*

        static TESTS: &[(&'static str, fn())] = &[$((stringify!($name), $name)),*];
    }
}

fn ptr_distance(a: *const (), b: *const ()) -> usize {
    (a as isize).wrapping_sub(b as isize).abs() as usize
}

fn alloc_stack(size: usize) -> *mut u8 {
    const STACK_ALIGN: usize = 4096;
    unsafe {
        let layout = std::alloc::Layout::from_size_align(size, STACK_ALIGN).unwrap();
        let new_stack = std::alloc::alloc(layout);
        assert!(!new_stack.is_null(), "allocations must succeed!");
        new_stack
    }
}

#[no_mangle]
fn rust_psm_test_get_bt() -> backtrace::Backtrace {
    backtrace::Backtrace::new()
}

fn find_frame_name(bt: &backtrace::Backtrace, name: &[u8]) -> Option<(usize, usize)> {
    for (frame_idx, frame) in bt.frames().into_iter().enumerate() {
        for (symbol_idx, symbol) in frame.symbols().into_iter().enumerate() {
            if symbol.name().map(|n| n.as_bytes()) == Some(name) {
                return Some((frame_idx, symbol_idx));
            }
        }
    }
    None
}

tests! {
    fn stack_direction_always_equal() {
        assert_eq!(crate::StackDirection::new(), crate::StackDirection::new());
    }

    fn stack_direction_is_correct() {
        #[inline(never)]
        fn test_direction(previous_sp: *mut u8) {
            let current_sp = crate::stack_pointer();
            match crate::StackDirection::new() {
                crate::StackDirection::Ascending => {
                    assert!(
                        current_sp > previous_sp,
                        "the stack pointer is not ascending! current = {:p}, previous = {:p}",
                        current_sp,
                        previous_sp
                    );
                }
                crate::StackDirection::Descending => {
                    assert!(
                        current_sp < previous_sp,
                        "the stack pointer is not descending! current = {:p}, previous = {:p}",
                        current_sp,
                        previous_sp
                    );
                }
            }
        }
        test_direction(crate::stack_pointer());
    }

    fn basic_on_stack() {
        unsafe {
            let new_stack = alloc_stack(4096);
            let r = crate::on_stack(new_stack, 4096, || (crate::stack_pointer(), 42 + 42_0000));
            assert_eq!(r.1, 42_0042);
            assert!(ptr_distance(crate::stack_pointer() as _, r.0 as _) > 0x1000_0000);
            assert!(ptr_distance(new_stack as _, r.0 as _) < 4096);
        }
    }

    #[inline(never)]
    fn on_stack_basic_backtrace() {
        let bt = unsafe {
            let new_stack = alloc_stack(128 * 4096);
            crate::on_stack(new_stack, 128 * 4096, rust_psm_test_get_bt)
        };

        let test_get_bt_frame = find_frame_name(&bt, b"rust_psm_test_get_bt");
        let on_stack_bt_frame = find_frame_name(&bt, b"rust_psm_on_stack");

        assert!(test_get_bt_frame.is_some());
        assert!(on_stack_bt_frame.is_some());
        assert!(bt.frames().len() > on_stack_bt_frame.unwrap().0 + 4);
    }


    fn on_stack_panic_handling() {
        use std::panic;
        const CHAIN_DEPTH: usize = 16;
        fn panic_chain(depth: usize) {
            if depth == 0 {
                panic!("full chain!");
            } else {
                unsafe {
                    // Generating backraces (because of RUST_BACKTRACE) create a few quite large
                    // frames, so it is important, that all frames have sufficient amount of
                    // available memory to not run over the stack...
                    let new_stack = alloc_stack(128 * 4096);
                    let p = crate::on_stack(new_stack, 128 * 4096, || {
                        panic::catch_unwind(|| {
                            panic_chain(depth - 1);
                        })
                    });
                    p.map_err(panic::resume_unwind).unwrap()
                }
            }
        }
        assert!(panic::catch_unwind(|| {
            panic_chain(CHAIN_DEPTH);
        }).is_err(), "Panic did not propagate!");
    }

    fn on_stack_multithread() {
        use std::thread;
        const FIB_COUNTS: [(usize, u64); 3] = [
            (8, 21),
            (16, 987),
            (24, 46368),
        ];

        #[inline(never)]
        fn fib(n: usize) -> u64 {
            unsafe {
                let new_stack = alloc_stack(4096);
                let r = match n {
                    0 => 0,
                    1 => 1,
                    _ => {
                        crate::on_stack(new_stack, 4096, || {
                            fib(n - 1) + fib(n - 2)
                        })
                    }
                };
                r
            }
        }

        for (expected, handle) in FIB_COUNTS.iter().map(|&(n, expected)|
            (expected, thread::spawn(move || {
                fib(n)
            }))
        ) {
            if let Ok(res) = handle.join() {
                assert_eq!(res, expected);
            } else {
                panic!("joining a thread returned an Err");
            }
        }
    }

    fn replace_stack_basic() {
        unsafe {
            let init_stack_ptr = crate::stack_pointer();
            let new_stack = alloc_stack(4096 * 64);
            crate::replace_stack(new_stack, 4096 * 64, || {
                assert!(ptr_distance(crate::stack_pointer() as _, init_stack_ptr as _) > 0x1000_0000);
                assert!(ptr_distance(crate::stack_pointer() as _, new_stack as _) < 4096 * 64);
                std::process::exit(0)
            });
        }
    }

    fn replace_stack_panic() {
        unsafe {
            let new_stack = alloc_stack(4096 * 64);
            crate::replace_stack(new_stack, 4096 * 64, || {
                let unwind = std::panic::catch_unwind(|| panic!("test") );
                assert!(unwind.is_err());
                std::process::exit(0);
            });
        }
    }

    fn replace_stack_backtrace() {
        unsafe {
            let new_stack = alloc_stack(128 * 4096);
            crate::replace_stack(new_stack, 128 * 4096, || {
                let bt = rust_psm_test_get_bt();
                let test_get_bt_frame = find_frame_name(&bt, b"rust_psm_test_get_bt");
                assert!(test_get_bt_frame.is_some());
                assert!(bt.frames().len() < test_get_bt_frame.unwrap().0 + 8);
                std::process::exit(0);
            })
        }
    }
}

#[inline(never)]
pub(crate) fn run() {
    let this = std::env::args_os().next().unwrap();
    if let Some(test_to_run) = std::env::args_os().nth(1) {
        for (name, test) in TESTS {
            if test_to_run == std::ffi::OsStr::new(name) {
                test();
                std::process::exit(0);
            }
        }
    } else {
        let mut failures = 0;
        for (name, _) in TESTS {
            let mut cmd = std::process::Command::new(&this);
            cmd.arg(name);
            eprintln!("{:?}...", cmd);
            failures = failures
                + match cmd.status() {
                    Ok(s) if s.success() => continue,
                    Ok(s) => {
                        eprintln!("test did not complete successfully: {:?}", s);
                        1
                    }
                    Err(e) => {
                        eprintln!("could not spawn self for single test: {}", e);
                        1
                    }
                }
        }
        if failures != 0 {
            eprintln!("{} tests failed", failures);
            std::process::exit(failures);
        }
    }
}
