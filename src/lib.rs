#[allow(improper_ctypes)]
extern {
    fn __stacker_stack_pointer() -> usize;
    fn __stacker_stack_limit() -> usize;
    fn __stacker_set_stack_limit(limit: usize);
    fn __stacker_switch_stacks(new_stack: usize,
                               fnptr: *const u8,
                               dataptr: *mut u8);
}

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

#[test]
fn it_works() {
}
