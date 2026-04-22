/// Always descending on x86.
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
    core::arch::asm!("mov {}, esp", out(reg) sp, options(nomem, nostack, preserves_flags));
    sp
}

/// Stack switching is not currently enabled for i686 Windows (canswitch = false).
///
/// Officially, switching stacks without Fibers is not supported on Windows.
/// The implementations below are preserved from the original x86_msvc.asm
/// for reference. They manipulate the TIB (Thread Information Block) at
/// fs:04h (StackLimit), fs:08h (StackBase), and fs:0E0Ch (DeallocationStack)
/// for SEH compatibility.
///
/// Original x86_msvc.asm replace_stack body:
/// ```asm
/// ; extern "fastcall" fn(%ecx: usize, %edx: extern "fastcall" fn(usize),
/// ;                      4(%esp): *mut u8, 8(%esp): *mut u8)
/// mov eax, dword ptr [esp + 8]
/// mov fs:[08h], eax
/// mov esp, dword ptr [esp + 4]
/// mov fs:[04h], esp
/// jmp edx
/// ```
#[cfg(switchable_stack)]
pub(crate) unsafe fn rust_psm_replace_stack(
    _data: usize,
    _callback: unsafe extern "fastcall" fn(usize) -> !,
    _sp: *mut u8,
    _stack_base: *mut u8,
) -> ! {
    todo!("i686 Windows stack switching requires Fibers; see psm/src/arch/x86_msvc.asm")
}

/// Original x86_msvc.asm on_stack body:
/// ```asm
/// ; extern "fastcall" fn(%ecx: usize, %edx: usize,
/// ;                      4(%esp): extern "fastcall" fn(usize, usize),
/// ;                      8(%esp): *mut u8, 12(%esp): *mut u8)
/// push ebp
/// mov ebp, esp
/// push fs:[0E0Ch]
/// push fs:[08h]
/// mov eax, dword ptr [ebp + 4 + 12]
/// mov dword ptr fs:[08h], eax
/// mov dword ptr fs:[0E0Ch], eax
/// push fs:[04h]
/// mov esp, dword ptr [ebp + 4 + 8]
/// mov dword ptr fs:[04h], esp
/// call dword ptr [ebp + 4 + 4]
/// lea esp, [ebp - 12]
/// pop fs:[04h]
/// pop fs:[08h]
/// pop fs:[0E0Ch]
/// pop ebp
/// ret 12
/// ```
#[cfg(switchable_stack)]
pub(crate) unsafe fn rust_psm_on_stack(
    _data: usize,
    _return_ptr: usize,
    _callback: unsafe extern "fastcall" fn(usize, usize),
    _sp: *mut u8,
    _stack_base: *mut u8,
) {
    todo!("i686 Windows stack switching requires Fibers; see psm/src/arch/x86_msvc.asm")
}
