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
    static STACK_LIMIT: Cell<usize> = Cell::new(guess_os_morestack_stack_limit())
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
    __stacker_switch_stacks(stack.as_mut_ptr() as usize + stack_size,
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

#[cfg(unix)]
fn guess_os_morestack_stack_limit() -> usize {
    unsafe {
        __stacker_morestack_stack_limit()
    }
}

// See this for where all this logic is coming from.
//
// https://github.com/adobe/webkit/blob/0441266/Source/WTF/wtf/StackBounds.cpp
#[cfg(windows)]
fn guess_os_morestack_stack_limit() -> usize {
    #[cfg(target_pointer_width = "32")]
    extern {
        #[link_name = "__stacker_get_tib_32"]
        fn get_tib_address() -> *const usize;
    }
    #[cfg(target_pointer_width = "64")]
    extern "system" {
        #[link_name = "NtCurrentTeb"]
        fn get_tib_address() -> *const usize;
    }
    unsafe {
        // See https://en.wikipedia.org/wiki/Win32_Thread_Information_Block for
        // the struct layout of the 32-bit TIB. It looks like the struct layout
        // of the 64-bit TIB is also the same for getting the stack limit:
        // http://doxygen.reactos.org/d3/db0/structNT__TIB64.html
        *get_tib_address().offset(2)
    }
}
