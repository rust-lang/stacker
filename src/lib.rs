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

#[allow(improper_ctypes)]
extern {
    fn __stacker_stack_pointer() -> usize;
    fn __stacker_stack_limit() -> usize;
    fn __stacker_set_stack_limit(limit: usize);
    fn __stacker_switch_stacks(new_stack: usize,
                               fnptr: *const u8,
                               dataptr: *mut u8);
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
        if __stacker_stack_pointer() - __stacker_stack_limit() >= red_zone {
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
        old_limit: usize,
    }
    let mut stack = Vec::<u8>::with_capacity(stack_size);
    let mut cx: Context<F, R> = Context {
        thunk: Some(f),
        ret: None::<R>,
        new_limit: stack.as_ptr() as usize + 32 * 1024,
        old_limit: __stacker_stack_limit(),
    };

    __stacker_set_stack_limit(0);
    __stacker_switch_stacks(stack.as_mut_ptr() as usize + stack_size,
                            doit::<R, F> as usize as *const _,
                            &mut cx as *mut _ as *mut _);
    __stacker_set_stack_limit(cx.old_limit);
    return cx.ret.unwrap();

    unsafe extern fn doit<R, F: FnOnce() -> R>(cx: &mut Context<F, R>) {
        __stacker_set_stack_limit(cx.new_limit);
        cx.ret = Some(cx.thunk.take().unwrap()());
        __stacker_set_stack_limit(0);
    }
}
