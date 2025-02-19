pub unsafe fn guess_os_stack_limit() -> Option<usize> {
    let mut stackinfo = std::mem::MaybeUninit::<libc::stack_t>::uninit();
    assert_eq!(
        libc::pthread_stackseg_np(libc::pthread_self(), stackinfo.as_mut_ptr()),
        0
    );
    Some(stackinfo.assume_init().ss_sp as usize - stackinfo.assume_init().ss_size)
}
