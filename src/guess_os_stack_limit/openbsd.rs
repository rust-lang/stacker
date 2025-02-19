pub unsafe fn guess_os_stack_limit() -> Option<usize> {
    let mut stackinfo = std::mem::MaybeUninit::<libc::stack_t>::uninit();
    let res = libc::pthread_stackseg_np(libc::pthread_self(), stackinfo.as_mut_ptr());
    if res != 0 {
        return None;
    }
    Some(stackinfo.assume_init().ss_sp as usize - stackinfo.assume_init().ss_size)
}
