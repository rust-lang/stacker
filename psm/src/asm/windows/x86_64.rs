/// Always descending on x86_64.
pub(crate) unsafe fn rust_psm_stack_direction() -> u8 {
    crate::StackDirection::Descending as u8
}

/// Returns an approximate pointer to the current stack position.
///
/// May be off by one frame without `#[naked]`; safe because stacker only
/// uses this for remaining-stack estimates with large (32KB+) thresholds.
// TODO: use #[unsafe(naked)] when MSRV >= 1.79 for exact results.
#[inline(always)]
pub(crate) unsafe fn rust_psm_stack_pointer() -> *mut u8 {
    let sp: *mut u8;
    core::arch::asm!("mov {}, rsp", out(reg) sp, options(nomem, nostack, preserves_flags));
    sp
}

/// Reads the current TIB StackBase (gs:0x08), sets rsp to that minus 8,
/// and tail-calls `callback(data)`. The `sp` and `stack_base` parameters
/// are ignored — the TIB values are used directly, matching the original
/// assembly behavior.
#[cfg(switchable_stack)]
pub(crate) unsafe fn rust_psm_replace_stack(
    data: usize,
    callback: unsafe extern "sysv64" fn(usize) -> !,
    _sp: *mut u8,
    _stack_base: *mut u8,
) -> ! {
    core::arch::asm!(
        "mov rax, gs:[0x08]",
        "lea rsp, [rax - 8]",
        "jmp {callback}",
        callback = in(reg) callback,
        in("rdi") data,
        out("rax") _,
        options(noreturn),
    )
}

/// Switches to the provided stack, calls the callback, then restores.
///
/// Saves and restores the Windows TIB stack limits at gs:0x08 (StackBase)
/// and gs:0x10 (StackLimit) so SEH exception handling works on the new stack.
#[cfg(switchable_stack)]
pub(crate) unsafe fn rust_psm_on_stack(
    data: usize,
    return_ptr: usize,
    callback: unsafe extern "sysv64" fn(usize, usize),
    sp: *mut u8,
    stack_base: *mut u8,
) {
    core::arch::asm!(
        // Save frame pointer and TIB stack limits
        "push rbp",
        "mov rax, gs:[0x08]",
        "push rax",
        "mov rax, gs:[0x10]",
        "push rax",
        // Switch to new stack, update TIB
        "mov rbp, rsp",
        "mov gs:[0x08], {sp}",
        "mov gs:[0x10], {stack_base}",
        "mov rsp, {sp}",
        // Call the callback
        "call {callback}",
        // Restore original stack and TIB
        "mov rsp, rbp",
        "pop rax",
        "mov gs:[0x10], rax",
        "pop rax",
        "mov gs:[0x08], rax",
        "pop rbp",
        sp = in(reg) sp,
        stack_base = in(reg) stack_base,
        callback = in(reg) callback,
        in("rdi") data,
        in("rsi") return_ptr,
        out("rax") _,
        clobber_abi("sysv64"),
    )
}
