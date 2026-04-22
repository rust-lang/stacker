/// Always descending on aarch64.
pub(crate) unsafe fn rust_psm_stack_direction() -> u8 {
    crate::StackDirection::Descending as u8
}

/// Returns an approximate pointer to the current stack position.
///
/// May be off by one frame without `#[naked]`; safe because stacker only
/// uses this for remaining-stack estimates with large (32KB+) thresholds.
// TODO: use #[unsafe(naked)] when MSRV >= 1.79 for exact results.
pub(crate) unsafe fn rust_psm_stack_pointer() -> *mut u8 {
    let sp: *mut u8;
    core::arch::asm!("mov {}, sp", out(reg) sp, options(nomem, nostack, preserves_flags));
    sp
}

/// Sets the stack pointer to `sp` and tail-calls `callback(data)`.
#[cfg(switchable_stack)]
pub(crate) unsafe fn rust_psm_replace_stack(
    data: usize,
    callback: unsafe extern "C" fn(usize) -> !,
    sp: *mut u8,
    _stack_base: *mut u8,
) -> ! {
    core::arch::asm!(
        "mov sp, {sp}",
        "br {callback}",
        sp = in(reg) sp,
        callback = in(reg) callback,
        in("x0") data,
        options(noreturn),
    )
}

/// Switches to the provided stack, calls the callback, then restores.
#[cfg(switchable_stack)]
pub(crate) unsafe fn rust_psm_on_stack(
    data: usize,
    return_ptr: usize,
    callback: unsafe extern "C" fn(usize, usize),
    sp: *mut u8,
    _stack_base: *mut u8,
) {
    core::arch::asm!(
        "stp x29, x30, [sp, #-16]!",
        "mov x29, sp",
        "mov sp, {sp}",
        "blr {callback}",
        "mov sp, x29",
        "ldp x29, x30, [sp], #16",
        sp = in(reg) sp,
        callback = in(reg) callback,
        in("x0") data,
        in("x1") return_ptr,
        clobber_abi("C"),
    )
}
