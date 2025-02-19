pub(super) struct PthreadAttr(std::mem::MaybeUninit<libc::pthread_attr_t>);

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

impl PthreadAttr {
    pub unsafe fn new() -> Option<Self> {
        let mut attr = std::mem::MaybeUninit::<libc::pthread_attr_t>::uninit();
        if libc::pthread_attr_init(attr.as_mut_ptr()) != 0 {
            return None;
        }
        Some(PthreadAttr(attr))
    }

    pub fn handle_pthread_err(&self, ret: libc::c_int) -> Option<()> {
        if ret != 0 {
            return None;
        }
        Some(())
    }

    pub fn as_mut_ptr(&mut self) -> *mut libc::pthread_attr_t {
        self.0.as_mut_ptr()
    }
}
