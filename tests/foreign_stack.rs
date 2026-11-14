//! Hosts such as Ruby (Fibers) and C coroutine libraries run code on stacks
//! they allocate and switch to themselves, without stacker knowing. stacker
//! caches the bounds of the OS thread's stack, so on such a stack
//! `remaining_stack` must still not report more stack than the current stack
//! has, or almost none when nearly all of it is free.

use psm::psm_stack_manipulation;

psm_stack_manipulation! {
    yes {
        use std::alloc::{alloc, dealloc, Layout};
        use std::thread;

        const STACK_SIZE: usize = 1024 * 1024;
        const STACK_ALIGN: usize = 4096;

        fn layout() -> Layout {
            Layout::from_size_align(STACK_SIZE, STACK_ALIGN).unwrap()
        }

        /// A stack allocated and switched to without stacker's involvement,
        /// the way a Ruby Fiber's machine stack is.
        struct ForeignStack(*mut u8);

        impl ForeignStack {
            fn new() -> Self {
                let base = unsafe { alloc(layout()) };
                assert!(!base.is_null());
                ForeignStack(base)
            }
        }

        impl Drop for ForeignStack {
            fn drop(&mut self) {
                unsafe { dealloc(self.0, layout()) }
            }
        }

        /// Calls `remaining_stack` near the top of the stack at `base`. The
        /// callback must not unwind, so assertions happen after switching back.
        fn remaining_stack_on(base: usize) -> Option<usize> {
            unsafe { psm::on_stack(base as *mut u8, STACK_SIZE, stacker::remaining_stack) }
        }

        /// Nearly all of a fresh stack is free, and nothing beyond its size is.
        /// `None` ("unknown") is also an honest answer.
        fn assert_plausible(remaining: Option<usize>) {
            if let Some(remaining) = remaining {
                assert!(
                    remaining > STACK_SIZE / 2 && remaining <= STACK_SIZE,
                    "remaining_stack() reported {remaining} bytes on a fresh {STACK_SIZE} byte stack"
                );
            }
        }

        #[test]
        #[cfg_attr(target_arch = "wasm32", ignore)]
        fn foreign_stack_below_the_thread_stack() {
            // Allocated after this thread's stack was mapped, so usually at a
            // lower address: the stack pointer is below the cached limit.
            let stack = ForeignStack::new();
            assert_plausible(remaining_stack_on(stack.0 as usize));
        }

        #[test]
        #[cfg_attr(target_arch = "wasm32", ignore)]
        fn foreign_stack_above_the_thread_stack() {
            // Allocated before the thread's stack is mapped, so usually at a
            // higher address: the stack pointer is above the thread's whole stack.
            let stack = ForeignStack::new();
            let base = stack.0 as usize;
            let remaining = thread::spawn(move || remaining_stack_on(base))
                .join()
                .unwrap();
            assert_plausible(remaining);
        }
    }

    no {}
}
