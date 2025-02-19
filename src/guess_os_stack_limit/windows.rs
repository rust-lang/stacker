use libc::c_void;
use std::io;
use std::ptr;
use windows_sys::Win32::Foundation::BOOL;
use windows_sys::Win32::System::Memory::VirtualQuery;
use windows_sys::Win32::System::Threading::*;

#[inline(always)]
fn get_thread_stack_guarantee() -> usize {
    let min_guarantee = if cfg!(target_pointer_width = "32") {
        0x1000
    } else {
        0x2000
    };
    let mut stack_guarantee = 0;
    unsafe { SetThreadStackGuarantee(&mut stack_guarantee) };
    std::cmp::max(stack_guarantee, min_guarantee) as usize + 0x1000
}

#[inline(always)]
pub unsafe fn guess_os_stack_limit() -> Option<usize> {
    type QueryT = windows_sys::Win32::System::Memory::MEMORY_BASIC_INFORMATION;
    let mut mi = std::mem::MaybeUninit::<QueryT>::uninit();
    VirtualQuery(
        psm::stack_pointer() as *const _,
        mi.as_mut_ptr(),
        std::mem::size_of::<QueryT>() as usize,
    );
    Some(mi.assume_init().AllocationBase as usize + get_thread_stack_guarantee() + 0x1000)
}
