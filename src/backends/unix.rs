#[cfg(any(target_os = "freebsd", target_os = "dragonfly", target_os = "illumos"))]
use libc::pthread_attr_get_np as get_attr;
#[cfg(any(target_os = "linux", target_os = "solaris", target_os = "netbsd"))]
use libc::pthread_getattr_np as get_attr;

pub unsafe fn guess_os_stack_limit() -> Option<usize> {
    let mut attr = PthreadAttr::new()?;

    handle_pthread_err(get_attr(libc::pthread_self(), attr.as_mut_ptr()))?;

    let mut stackaddr = std::ptr::null_mut();
    let mut stacksize = 0;
    handle_pthread_err(libc::pthread_attr_getstack(
        attr.as_mut_ptr(),
        &mut stackaddr,
        &mut stacksize,
    ))?;

    Some(stackaddr as usize)
}

struct PthreadAttr(std::mem::MaybeUninit<libc::pthread_attr_t>);

impl Drop for PthreadAttr {
    fn drop(&mut self) {
        unsafe {
            let ret = libc::pthread_attr_destroy(self.0.as_mut_ptr());
            if ret != 0 {
                let err = std::io::Error::last_os_error();
                panic!(
                    "pthread_attr_destroy failed with error code {}: {}",
                    ret, err
                );
            }
        }
    }
}

fn handle_pthread_err(ret: libc::c_int) -> Option<()> {
    if ret != 0 {
        return None;
    }
    Some(())
}

impl PthreadAttr {
    unsafe fn new() -> Option<Self> {
        let mut attr = std::mem::MaybeUninit::<libc::pthread_attr_t>::uninit();
        if libc::pthread_attr_init(attr.as_mut_ptr()) != 0 {
            return None;
        }
        Some(PthreadAttr(attr))
    }

    fn as_mut_ptr(&mut self) -> *mut libc::pthread_attr_t {
        self.0.as_mut_ptr()
    }
}
