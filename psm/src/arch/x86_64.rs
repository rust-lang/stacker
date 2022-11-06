pub(crate) fn stack_direction() -> crate::StackDirection {
    crate::StackDirection::Descending
}

pub(crate) fn stack_pointer() -> *mut u8 {
    let mut ret;
    unsafe {
        core::arch::asm! {
            "mov {ret}, rsp",
            ret = lateout(reg) ret,
            options(preserves_flags, nomem),
        }
    }
    ret
}

pub(crate) unsafe fn replace_stack(
    data: usize,
    callback: unsafe extern "sysv64" fn(usize) -> !,
    sp: *mut u8,
    _: *mut u8,
) -> ! {
    core::arch::asm! {
        "lea rsp, [{sp} - 8]",
        "jmp {callback}",
        sp = in(reg) sp,
        callback = in(reg) callback,
        in("rdi") data,
        options(noreturn, nostack),
    }
}

core::arch::global_asm! {
    ".balign 16",
    ".local rust_psm_on_stack",
    ".hidden rust_psm_on_stack",
    ".type rust_psm_on_stack,@function",
    "rust_psm_on_stack:",
    ".cfi_startproc",
    "xchg rsp, r12",
    ".cfi_def_cfa_register r12",
    "call rdx",
    "mov rsp, r12",
    "ret",
    ".cfi_endproc",
}

pub(crate) unsafe fn on_stack(
    data: usize,
    return_ptr: usize,
    callback: unsafe extern "sysv64" fn(usize, usize),
    sp: *mut u8,
    _: *mut u8,
) {
    core::arch::asm! {
        "call rust_psm_on_stack",
        in("rdi") data,
        in("rsi") return_ptr,
        in("rdx") callback,
        inout("r12") sp => _,
        clobber_abi("sysv64"),
    }
}
