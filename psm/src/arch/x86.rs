pub(crate) fn stack_direction() -> crate::StackDirection {
    crate::StackDirection::Descending
}

pub(crate) fn stack_pointer() -> *mut u8 {
    let mut ret;
    unsafe {
        core::arch::asm! {
            "mov {ret}, esp",
            ret = lateout(reg) ret,
            options(preserves_flags, nomem),
        }
    }
    ret
}

pub(crate) unsafe fn replace_stack(
    data: usize,
    callback: unsafe extern "fastcall" fn(usize) -> !,
    sp: *mut u8,
    _: *mut u8,
) -> ! {
    core::arch::asm! {
        "lea esp, [{sp} - 12]",
        "jmp {callback}",
        sp = in(reg) sp,
        callback = in(reg) callback,
        in("ecx") data,
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
    "xchg esp, edi",
    ".cfi_def_cfa_register edi",
    "call eax",
    "mov esp, edi",
    "ret",
    ".cfi_endproc",
}

pub(crate) unsafe fn on_stack(
    data: usize,
    return_ptr: usize,
    callback: unsafe extern "fastcall" fn(usize, usize),
    sp: *mut u8,
    _: *mut u8,
) {
    core::arch::asm! {
        "call rust_psm_on_stack",
        in("ecx") data,
        in("edx") return_ptr,
        in("eax") callback,
        inout("edi") sp => _,
        clobber_abi("fastcall"),
    }
}
