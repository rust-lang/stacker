pub(crate) fn stack_direction() -> crate::StackDirection {
    crate::StackDirection::Descending
}

pub(crate) fn stack_pointer() -> *mut u8 {
    let mut ret;
    unsafe {
        core::arch::asm! {
            "mov {ret}, sp",
            ret = lateout(reg) ret,
            options(preserves_flags, nomem),
        }
    }
    ret
}

pub(crate) unsafe extern "C" fn replace_stack(
    data: usize,
    callback: unsafe extern "C" fn(usize) -> !,
    sp: *mut u8,
    _: *mut u8,
) -> ! {
    core::arch::asm! {
        "mov sp, {new_sp}",
        "br {callback}",
        "ud2",
        new_sp = in(reg) sp,
        callback = in(reg) callback,
        in("r0") data,
        options(noreturn, nostack),
    }
}

core::arch::global_asm! {
    ".balign 8",
    ".local rust_psm_on_stack",
    ".hidden rust_psm_on_stack",
    ".type rust_psm_on_stack,@function",
    "rust_psm_on_stack:",
    ".cfi_startproc",
    "stp fp, lr, [sp, #-16]!"
    ".cfi_offset lr, -8",
    ".cfi_offset fp, -16",
    "mov fp, sp",
    ".cfi_def_cfa_register fp",
    "mov sp, x3",
    "blr x2",
    "mov sp, fp",
    ".cfi_def_cfa_register sp",
    "ldp fp, lr, [fp], #16",
    "ret",
    ".cfi_endproc",
}

pub(crate) unsafe extern "C" fn on_stack(
    data: usize,
    return_ptr: usize,
    callback: unsafe extern "C" fn(usize, usize),
    sp: *mut u8,
    _: *mut u8,
) {
    core::arch::asm! {
        "call rust_psm_on_stack",
        in("x0") data,
        in("x1") return_ptr,
        in("x2") callback,
        in("x3") sp,
        clobber_abi("C"),
    }
}
