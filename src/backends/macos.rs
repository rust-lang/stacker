pub unsafe fn guess_os_stack_bounds() -> Option<crate::StackBounds> {
    // pthread_get_stackaddr_np returns the top of the stack
    let end = libc::pthread_get_stackaddr_np(libc::pthread_self()) as usize;
    let size = libc::pthread_get_stacksize_np(libc::pthread_self()) as usize;
    Some(crate::StackBounds {
        start: end - size,
        end,
    })
}
