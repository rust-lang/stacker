//! A library to help grow the stack when it runs out of space.
//!
//! This is an implementation of manually instrumented segmented stacks where
//! points in a program's control flow are annotated with "maybe grow the stack
//! here". Each point of annotation indicates how far away from the end of the
//! stack it's allowed to be, plus the amount of stack to allocate if it does
//! reach the end.
//!
//! Once a program has reached the end of its stack, a temporary stack on the
//! heap is allocated and is switched to for the duration of a closure.
//!
//! # Examples
//!
//! ```
//! // Grow the stack if we are within the "red zone" of 32K, and if we allocate
//! // a new stack allocate 1MB of stack space.
//! //
//! // If we're already in bounds, however, just run the provided closure on our
//! // own stack
//! stacker::maybe_grow(32 * 1024, 1024 * 1024, || {
//!     // guaranteed to have at least 32K of stack
//! });
//! ```

#![allow(improper_ctypes)]

#[macro_use]
extern crate cfg_if;
extern crate libc;

use std::cell::Cell;

extern {
    fn __stacker_morestack_stack_limit() -> usize;
    fn __stacker_set_morestack_stack_limit(limit: usize);
    fn __stacker_stack_pointer() -> usize;
    fn __stacker_switch_stacks(new_stack: usize,
                               fnptr: *const u8,
                               dataptr: *mut u8);
}

thread_local! {
    static STACK_LIMIT: Cell<usize> = Cell::new(unsafe {
        guess_os_morestack_stack_limit()
    })
}

fn get_stack_limit() -> usize {
    STACK_LIMIT.with(|s| s.get())
}

fn set_stack_limit(l: usize) {
    STACK_LIMIT.with(|s| s.set(l))
}

/// Grows the call stack if necessary.
///
/// This function is intended to be called at manually instrumented points in a
/// program where recursion is known to happen quite a bit. This function will
/// check to see if we're within `red_zone` bytes of the end of the stack, and
/// if so it will allocate a new stack of size `stack_size`.
///
/// The closure `f` is guaranteed to run on a stack with at least `red_zone`
/// bytes, and it will be run on the current stack if there's space available.
pub fn maybe_grow<R, F: FnOnce() -> R>(red_zone: usize,
                                       stack_size: usize,
                                       f: F) -> R {
    unsafe {
        if __stacker_stack_pointer() - get_stack_limit() >= red_zone {
            f()
        } else {
            grow_the_stack(stack_size, f)
        }
    }
}

#[inline(never)]
unsafe fn grow_the_stack<R, F: FnOnce() -> R>(stack_size: usize, f: F) -> R {
    struct Context<F: FnOnce() -> R, R> {
        thunk: Option<F>,
        ret: Option<R>,
        new_limit: usize,
    }

    // Align to 16-bytes (see below for why)
    let stack_size = (stack_size + 15) / 16 * 16;

    // Allocate some new stack for oureslves
    let mut stack = Vec::<u8>::with_capacity(stack_size);
    let new_limit = stack.as_ptr() as usize + 32 * 1024;

    // Save off the old stack limits
    let old_morestack_limit = __stacker_morestack_stack_limit();
    let old_limit = get_stack_limit();

    // Prepare stack limits for the stack switch, note that the morestack stack
    // limit will be set to a real value once we've switched threads.
    __stacker_set_morestack_stack_limit(0);
    set_stack_limit(new_limit);

    // Set up the arguments and do the actual stack switch.
    let mut cx: Context<F, R> = Context {
        thunk: Some(f),
        ret: None,
        new_limit: new_limit,
    };

    // Make sure the stack is 16-byte aligned which should be enough for all
    // platforms right now. Allocations on 64-bit are already 16-byte aligned
    // and our switching routine doesn't push any other data, but the routine on
    // 32-bit pushes an argument so we need a bit of an offset to get it 16-byte
    // aligned when the call is made.
    let offset = if cfg!(target_pointer_width = "32") {
        12
    } else {
        0
    };
    __stacker_switch_stacks(stack.as_mut_ptr() as usize + stack_size - offset,
                            doit::<R, F> as usize as *const _,
                            &mut cx as *mut _ as *mut _);

    // Once we've returned reset bothe stack limits and then return value same
    // value the closure returned.
    __stacker_set_morestack_stack_limit(old_morestack_limit);
    set_stack_limit(old_limit);
    return cx.ret.unwrap();

    unsafe extern fn doit<R, F: FnOnce() -> R>(cx: &mut Context<F, R>) {
        __stacker_set_morestack_stack_limit(cx.new_limit);
        cx.ret = Some(cx.thunk.take().unwrap()());
        __stacker_set_morestack_stack_limit(0);
    }
}

cfg_if! {
    if #[cfg(windows)] {
        // See this for where all this logic is coming from.
        //
        // https://github.com/adobe/webkit/blob/0441266/Source/WTF/wtf
        //                   /StackBounds.cpp
        unsafe fn guess_os_morestack_stack_limit() -> usize {
            #[cfg(target_pointer_width = "32")]
            extern {
                #[link_name = "__stacker_get_tib_32"]
                fn get_tib_address() -> *const usize;
            }
            #[cfg(target_pointer_width = "64")]
            extern "system" {
                #[cfg_attr(target_env = "msvc", link_name = "NtCurrentTeb")]
                #[cfg_attr(target_env = "gnu", link_name = "__stacker_get_tib_64")]
                fn get_tib_address() -> *const usize;
            }
            // https://en.wikipedia.org/wiki/Win32_Thread_Information_Block for
            // the struct layout of the 32-bit TIB. It looks like the struct
            // layout of the 64-bit TIB is also the same for getting the stack
            // limit: http://doxygen.reactos.org/d3/db0/structNT__TIB64.html
            *get_tib_address().offset(2)
        }
    } else if #[cfg(target_os = "linux")] {
        use libc::{pthread_attr_t, c_int, size_t, c_void, pthread_t};
        use std::mem;

        unsafe fn guess_os_morestack_stack_limit() -> usize {
            let mut attr: libc::pthread_attr_t = mem::zeroed();
            assert_eq!(pthread_attr_init(&mut attr), 0);
            assert_eq!(pthread_getattr_np(pthread_self(), &mut attr), 0);
            let mut stackaddr = 0 as *mut _;
            let mut stacksize = 0;
            assert_eq!(pthread_attr_getstack(&attr, &mut stackaddr,
                                             &mut stacksize), 0);
            assert_eq!(pthread_attr_destroy(&mut attr), 0);
            stackaddr as usize
        }

        extern {
            fn pthread_self() -> pthread_t;
            fn pthread_attr_init(attr: *mut pthread_attr_t) -> c_int;
            fn pthread_attr_destroy(attr: *mut pthread_attr_t) -> c_int;
            fn pthread_attr_getstack(attr: *const pthread_attr_t,
                                     stackaddr: *mut *mut c_void,
                                     stacksize: *mut size_t) -> c_int;
            fn pthread_getattr_np(native: pthread_t,
                                  attr: *mut pthread_attr_t) -> c_int;
        }
    } else if #[cfg(target_os = "macos")] {
        use libc::{c_void, pthread_t, size_t};

        unsafe fn guess_os_morestack_stack_limit() -> usize {
            pthread_get_stackaddr_np(pthread_self()) as usize -
                pthread_get_stacksize_np(pthread_self()) as usize
        }

        extern {
            fn pthread_self() -> pthread_t;
            fn pthread_get_stackaddr_np(thread: pthread_t) -> *mut c_void;
            fn pthread_get_stacksize_np(thread: pthread_t) -> size_t;
        }
    } else {
        unsafe fn guess_os_morestack_stack_limit() -> usize {
            panic!("cannot guess the stack limit on this platform");
        }
    }
}
