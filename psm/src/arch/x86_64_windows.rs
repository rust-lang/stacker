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
        "mov qword ptr gs:[0x0], -1",
        "mov qword ptr gs:[0x8], rdx",
        "mov qword ptr gs:[0x10], rcx",
        "mov qword ptr gs:[0x1478], rcx",
        "mov qword ptr gs:[0x1748], 0",
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
    "rust_psm_on_stack:",
    ".seh_proc rust_psm_on_stack",

    "push qword ptr gs:[0x1748]", // GuaranteedStackBytes
    ".seh_stackalloc 8",
    "push qword ptr gs:[0x1478]", // DeallocationStack
    ".seh_stackalloc 8",
    "push qword ptr gs:[0x10]",   // StackLimit
    ".seh_stackalloc 8",
    "push qword ptr gs:[0x08]",   // StackBase
    ".seh_stackalloc 8",
    "push qword ptr gs:[0x00]",   // ExceptionList
    ".seh_stackalloc 8",
    "xchg rsp, r12",
    ".seh_setframe r12, 0",
    ".seh_endprologue",

    "mov qword ptr gs:[0x00], -1",
    "mov qword ptr gs:[0x08], rsp",
    "mov qword ptr gs:[0x10], r8",
    // TODO: these need a more proper handling
    "mov qword ptr gs:[0x1478], rsp",
    "mov qword ptr gs:[0x1748], 0",

    "call rdx",

    // Reset the state.
    "mov rsp, r12",
    "pop qword ptr gs:[0x00]",
    "pop qword ptr gs:[0x08]",
    "pop qword ptr gs:[0x10]",
    "pop qword ptr gs:[0x1478]",
    "pop qword ptr gs:[0x1748]",
    "ret",
    ".seh_endproc",
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
