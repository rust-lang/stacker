//! A library to help grow the stack when it runs out of space.
//!
//! This is an implementation of manually instrumented segmented stacks where points in a program's
//! control flow are annotated with "maybe grow the stack here". Each point of annotation indicates
//! how far away from the end of the stack it's allowed to be, plus the amount of stack to allocate
//! if it does reach the end.
//!
//! Once a program has reached the end of its stack, a temporary stack on the heap is allocated and
//! is switched to for the duration of a closure.
//!
//! For a set of lower-level primitives, consider the `psm` crate.
//!
//! # Examples
//!
//! ```
//! // Grow the stack if we are within the "red zone" of 32K, and if we allocate
//! // a new stack allocate 1MB of stack space.
//! //
//! // If we're already in bounds, just run the provided closure on current stack.
//! stacker::maybe_grow(32 * 1024, 1024 * 1024, || {
//!     // guaranteed to have at least 32K of stack
//! });
//! ```

#![allow(improper_ctypes)]

#[macro_use]
extern crate cfg_if;
extern crate libc;
#[cfg(windows)]
extern crate windows_sys;
#[macro_use]
extern crate psm;

mod backends;

use std::cell::Cell;

/// Grows the call stack if necessary.
///
/// This function is intended to be called at manually instrumented points in a program where
/// recursion is known to happen quite a bit. This function will check to see if we're within
/// `red_zone` bytes of the end of the stack, and if so it will allocate a new stack of at least
/// `stack_size` bytes.
///
/// The closure `f` is guaranteed to run on a stack with at least `red_zone` bytes, and it will be
/// run on the current stack if there's space available.
#[inline(always)]
pub fn maybe_grow<R, F: FnOnce() -> R>(red_zone: usize, stack_size: usize, callback: F) -> R {
    // if we can't guess the remaining stack (unsupported on some platforms) we immediately grow
    // the stack and then cache the new stack size (which we do know now because we allocated it.
    let enough_space = match remaining_stack() {
        Some(remaining) => remaining >= red_zone,
        None => false,
    };
    if enough_space {
        callback()
    } else {
        grow(stack_size, callback)
    }
}

/// Always creates a new stack for the passed closure to run on.
/// The closure will still be on the same thread as the caller of `grow`.
/// This will allocate a new stack with at least `stack_size` bytes.
pub fn grow<R, F: FnOnce() -> R>(stack_size: usize, callback: F) -> R {
    // To avoid monomorphizing `_grow()` and everything it calls,
    // we convert the generic callback to a dynamic one.
    let mut opt_callback = Some(callback);
    let mut ret = None;
    let ret_ref = &mut ret;

    // This wrapper around `callback` achieves two things:
    // * It converts the `impl FnOnce` to a `dyn FnMut`.
    //   `dyn` because we want it to not be generic, and
    //   `FnMut` because we can't pass a `dyn FnOnce` around without boxing it.
    // * It eliminates the generic return value, by writing it to the stack of this function.
    //   Otherwise the closure would have to return an unsized value, which isn't possible.
    let dyn_callback: &mut dyn FnMut() = &mut || {
        let taken_callback = opt_callback.take().unwrap();
        *ret_ref = Some(taken_callback());
    };

    _grow(stack_size, dyn_callback);
    ret.unwrap()
}

/// Queries the amount of remaining stack as interpreted by this library.
///
/// This function will return the amount of stack space left which will be used
/// to determine whether a stack switch should be made or not.
pub fn remaining_stack() -> Option<usize> {
    let current_ptr = current_stack_ptr();
    get_stack_limit().map(|limit| current_ptr - limit)
}

psm_stack_information!(
    yes {
        fn current_stack_ptr() -> usize {
            psm::stack_pointer() as usize
        }
    }
    no {
        #[inline(always)]
        fn current_stack_ptr() -> usize {
            unsafe {
                let mut x = std::mem::MaybeUninit::<u8>::uninit();
                // Unlikely to be ever exercised. As a fallback we execute a volatile read to a
                // local (to hopefully defeat the optimisations that would make this local a static
                // global) and take its address. This way we get a very approximate address of the
                // current frame.
                x.as_mut_ptr().write_volatile(42);
                x.as_ptr() as usize
            }
        }
    }
);

thread_local! {
    static STACK_LIMIT: Cell<Option<usize>> = Cell::new(unsafe {
        backends::guess_os_stack_limit()
    })
}

#[inline(always)]
fn get_stack_limit() -> Option<usize> {
    STACK_LIMIT.with(|s| s.get())
}

#[inline(always)]
#[allow(unused)]
fn set_stack_limit(l: Option<usize>) {
    STACK_LIMIT.with(|s| s.set(l))
}

psm_stack_manipulation! {
    yes {
        struct StackRestoreGuard {
            new_stack: *mut std::ffi::c_void,
            stack_bytes: usize,
            old_stack_limit: Option<usize>,
        }

        impl StackRestoreGuard {
            #[cfg(any(target_arch = "wasm32",target_os = "hermit"))]
            unsafe fn new(stack_bytes: usize, _page_size: usize) -> StackRestoreGuard {
                let layout = std::alloc::Layout::from_size_align(stack_bytes, 16).unwrap();
                let ptr = std::alloc::alloc(layout);
                assert!(!ptr.is_null(), "unable to allocate stack");
                StackRestoreGuard {
                    new_stack: ptr as *mut _,
                    stack_bytes,
                    old_stack_limit: get_stack_limit(),
                }
            }

            #[cfg(not(any(target_arch = "wasm32",target_os = "hermit")))]
            unsafe fn new(stack_bytes: usize, page_size: usize) -> StackRestoreGuard {
                let new_stack = libc::mmap(
                    std::ptr::null_mut(),
                    stack_bytes,
                    libc::PROT_NONE,
                    libc::MAP_PRIVATE |
                    libc::MAP_ANON,
                    -1, // Some implementations assert fd = -1 if MAP_ANON is specified
                    0
                );
                assert_ne!(
                    new_stack,
                    libc::MAP_FAILED,
                    "mmap failed to allocate stack: {}",
                    std::io::Error::last_os_error()
                );
                let guard = StackRestoreGuard {
                    new_stack,
                    stack_bytes,
                    old_stack_limit: get_stack_limit(),
                };
                let above_guard_page = new_stack.add(page_size);
                #[cfg(not(target_os = "openbsd"))]
                let result = libc::mprotect(
                    above_guard_page,
                    stack_bytes - page_size,
                    libc::PROT_READ | libc::PROT_WRITE
                );
                #[cfg(target_os = "openbsd")]
                let result = if libc::mmap(
                        above_guard_page,
                        stack_bytes - page_size,
                        libc::PROT_READ | libc::PROT_WRITE,
                        libc::MAP_FIXED | libc::MAP_PRIVATE | libc::MAP_ANON | libc::MAP_STACK,
                        -1,
                        0) == above_guard_page {
                    0
                } else {
                    -1
                };
                assert_ne!(
                    result,
                    -1,
                    "mprotect/mmap failed: {}",
                    std::io::Error::last_os_error()
                );
                guard
            }
        }

        impl Drop for StackRestoreGuard {
            fn drop(&mut self) {
                #[cfg(any(target_arch = "wasm32",target_os = "hermit"))]
                unsafe {
                    std::alloc::dealloc(
                        self.new_stack as *mut u8,
                        std::alloc::Layout::from_size_align_unchecked(self.stack_bytes, 16),
                    );
                }
                #[cfg(not(any(target_arch = "wasm32",target_os = "hermit")))]
                unsafe {
                    // FIXME: check the error code and decide what to do with it.
                    // Perhaps a debug_assertion?
                    libc::munmap(self.new_stack, self.stack_bytes);
                }
                set_stack_limit(self.old_stack_limit);
            }
        }

        fn _grow(stack_size: usize, callback: &mut dyn FnMut()) {
            // Calculate a number of pages we want to allocate for the new stack.
            // For maximum portability we want to produce a stack that is aligned to a page and has
            // a size that’s a multiple of page size. Furthermore we want to allocate two extras pages
            // for the stack guard. To achieve that we do our calculations in number of pages and
            // convert to bytes last.
            let page_size = page_size();
            let requested_pages = stack_size
                .checked_add(page_size - 1)
                .expect("unreasonably large stack requested") / page_size;
            let stack_pages = std::cmp::max(1, requested_pages) + 2;
            let stack_bytes = stack_pages.checked_mul(page_size)
                .expect("unreasonably large stack requested");

            // Next, there are a couple of approaches to how we allocate the new stack. We take the
            // most obvious path and use `mmap`. We also `mprotect` a guard page into our
            // allocation.
            //
            // We use a guard pattern to ensure we deallocate the allocated stack when we leave
            // this function and also try to uphold various safety invariants required by `psm`
            // (such as not unwinding from the callback we pass to it).
            //
            // Other than that this code has no meaningful gotchas.
            unsafe {
                let guard = StackRestoreGuard::new(stack_bytes, page_size);
                let above_guard_page = guard.new_stack.add(page_size);
                set_stack_limit(Some(above_guard_page as usize));
                let panic = psm::on_stack(above_guard_page as *mut _, stack_size, move || {
                    std::panic::catch_unwind(std::panic::AssertUnwindSafe(callback)).err()
                });
                drop(guard);
                if let Some(p) = panic {
                    std::panic::resume_unwind(p);
                }
            }
        }

        fn page_size() -> usize {
            // FIXME: consider caching the page size.
            #[cfg(not(any(target_arch = "wasm32",target_os = "hermit")))]
            unsafe { libc::sysconf(libc::_SC_PAGE_SIZE) as usize }
            #[cfg(any(target_arch = "wasm32",target_os = "hermit"))]
            { 65536 }
        }
    }

    no {
        #[cfg(not(windows))]
        fn _grow(stack_size: usize, callback: &mut dyn FnMut()) {
            let _ = stack_size;
            callback();
        }
        #[cfg(windows)]
        use backends::windows::_grow;
    }
}
