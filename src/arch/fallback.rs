#[no_mangle]
extern fn __stacker_stack_pointer() -> usize {
    0
}

#[no_mangle]
unsafe extern fn __stacker_switch_stacks(
    _new_stack: usize,
    fnptr: unsafe fn(&mut &mut FnMut()),
    dataptr: &mut &mut FnMut(),
) {
    fnptr(dataptr)
}