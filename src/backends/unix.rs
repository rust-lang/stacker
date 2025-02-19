use super::unix_pthread_wrapper::PthreadAttr;

#[cfg(any(target_os = "freebsd", target_os = "dragonfly", target_os = "illumos"))]
use libc::pthread_attr_get_np as get_attr;
#[cfg(any(target_os = "linux", target_os = "solaris", target_os = "netbsd"))]
use libc::pthread_getattr_np as get_attr;

pub unsafe fn guess_os_stack_limit() -> Option<usize> {
    let mut attr = PthreadAttr::new()?;

    let res = get_attr(libc::pthread_self(), attr.as_mut_ptr());
    attr.handle_pthread_err(res)?;

    let mut stackaddr = std::ptr::null_mut();
    let mut stacksize = 0;
    let res = libc::pthread_attr_getstack(attr.as_mut_ptr(), &mut stackaddr, &mut stacksize);
    attr.handle_pthread_err(res)?;

    Some(stackaddr as usize)
}
