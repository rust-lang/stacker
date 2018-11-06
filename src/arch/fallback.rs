#[no_mangle]
extern fn __stacker_stack_pointer() -> usize {
    panic!("not supported")
}

#[no_mangle]
unsafe extern fn __stacker_switch_stacks(
    _new_stack: usize,
    _fnptr: unsafe fn(&mut &mut FnMut()),
    _dataptr: &mut &mut FnMut(),
) {
    panic!("not supported")
}

#[no_mangle]
extern fn __stacker_black_box(_: *const u8) {}