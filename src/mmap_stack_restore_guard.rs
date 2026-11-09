use crate::{get_stack_limit, set_stack_limit, CACHED_STACKS, HUGE_PAGES};
use std::cell::RefCell;
use std::sync::atomic::Ordering;

pub struct StackRestoreGuard {
    mapping: Option<Mapping>,
    old_stack_limit: Option<usize>,
}

impl StackRestoreGuard {
    pub fn new(requested_size: usize) -> StackRestoreGuard {
        let huge_pages = HUGE_PAGES.load(Ordering::Relaxed);
        let mapping = take_cached(requested_size, huge_pages)
            .unwrap_or_else(|| Mapping::new(requested_size, huge_pages));
        StackRestoreGuard {
            mapping: Some(mapping),
            old_stack_limit: get_stack_limit(),
        }
    }

    // TODO this should return a *mut [u8], but pointer slices only got proper support with Rust 1.79.
    pub fn stack_area(&self) -> (*mut u8, usize) {
        let mapping = self.mapping.as_ref().unwrap();
        (mapping.stack, mapping.stack_size)
    }
}

impl Drop for StackRestoreGuard {
    fn drop(&mut self) {
        if let Some(mapping) = self.mapping.take() {
            release(mapping);
        }
        set_stack_limit(self.old_stack_limit);
    }
}

struct Mapping {
    mapping: *mut u8,
    mapping_size: usize,
    stack: *mut u8,
    stack_size: usize,
    huge_pages: bool,
}

impl Mapping {
    fn new(requested_size: usize, huge_pages: bool) -> Mapping {
        // For maximum portability we want to produce a stack that is aligned to a page and has
        // a size that’s a multiple of page size. It is natural to use mmap to allocate
        // these pages. Furthermore, we want to allocate two extras pages for the stack guard.
        // To achieve that we do our calculations in number of pages and convert to bytes last.
        let page_size = page_size();
        let align = if huge_pages {
            huge_page_size().unwrap_or(page_size)
        } else {
            page_size
        };
        let stack_size = std::cmp::max(1, requested_size)
            .checked_add(align - 1)
            .expect("unreasonably large stack requested")
            / align
            * align;
        let mapping_size = stack_size
            .checked_add(page_size + align)
            .expect("unreasonably large stack requested");

        unsafe {
            let new_stack = libc::mmap(
                std::ptr::null_mut(),
                mapping_size,
                libc::PROT_NONE,
                libc::MAP_PRIVATE | libc::MAP_ANON,
                -1, // Some implementations assert fd = -1 if MAP_ANON is specified
                0,
            );
            assert_ne!(
                new_stack,
                libc::MAP_FAILED,
                "mmap failed to allocate stack: {}",
                std::io::Error::last_os_error()
            );
            let mapping = new_stack as *mut u8;
            let base = mapping as usize + page_size;
            let above_guard_page = mapping.add((base + align - 1) / align * align - mapping as usize);
            let mapping = Mapping {
                mapping,
                mapping_size,
                stack: above_guard_page,
                stack_size,
                huge_pages,
            };
            let above_guard_page = above_guard_page as *mut libc::c_void;
            // We leave two guard pages without read/write access in our allocation.
            // There is one guard page below the stack and another above it.
            #[cfg(not(target_os = "openbsd"))]
            let result = libc::mprotect(
                above_guard_page,
                stack_size,
                libc::PROT_READ | libc::PROT_WRITE,
            );
            #[cfg(target_os = "openbsd")]
            let result = if libc::mmap(
                above_guard_page,
                stack_size,
                libc::PROT_READ | libc::PROT_WRITE,
                libc::MAP_FIXED | libc::MAP_PRIVATE | libc::MAP_ANON | libc::MAP_STACK,
                -1,
                0,
            ) == above_guard_page
            {
                0
            } else {
                -1
            };
            assert_ne!(
                result,
                -1,
                "mprotect/mmap failed: {}",
                std::io::Error::last_os_error()
            );
            #[cfg(any(target_os = "linux", target_os = "android"))]
            if huge_pages {
                libc::madvise(above_guard_page, stack_size, libc::MADV_HUGEPAGE);
            }
            mapping
        }
    }
}

impl Drop for Mapping {
    fn drop(&mut self) {
        unsafe {
            // FIXME: check the error code and decide what to do with it.
            // Perhaps a debug_assertion?
            libc::munmap(self.mapping as *mut std::ffi::c_void, self.mapping_size);
        }
    }
}

thread_local! {
    static CACHE: RefCell<Vec<Mapping>> = const { RefCell::new(Vec::new()) };
}

fn take_cached(requested_size: usize, huge_pages: bool) -> Option<Mapping> {
    CACHE
        .try_with(|cache| {
            let mut cache = cache.borrow_mut();
            let index = cache
                .iter()
                .rposition(|m| m.huge_pages == huge_pages && m.stack_size >= requested_size)?;
            Some(cache.swap_remove(index))
        })
        .ok()
        .flatten()
}

fn release(mapping: Mapping) {
    let limit = CACHED_STACKS.load(Ordering::Relaxed);
    if limit == 0 {
        return;
    }
    let _ = CACHE.try_with(move |cache| {
        let mut cache = cache.borrow_mut();
        if cache.len() < limit {
            cache.push(mapping);
        }
    });
}

#[cfg(any(target_os = "linux", target_os = "android"))]
fn huge_page_size() -> Option<usize> {
    static SIZE: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
    match SIZE.load(Ordering::Relaxed) {
        0 => {}
        usize::MAX => return None,
        size => return Some(size),
    }
    let size = std::fs::read_to_string("/sys/kernel/mm/transparent_hugepage/hpage_pmd_size")
        .ok()
        .and_then(|s| s.trim().parse::<usize>().ok())
        .filter(|s| s.is_power_of_two());
    SIZE.store(size.unwrap_or(usize::MAX), Ordering::Relaxed);
    size
}

#[cfg(not(any(target_os = "linux", target_os = "android")))]
fn huge_page_size() -> Option<usize> {
    None
}

fn page_size() -> usize {
    // FIXME: consider caching the page size.
    unsafe { libc::sysconf(libc::_SC_PAGE_SIZE) as usize }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stack_area() {
        for stack_size_kb in 1..64 {
            let size = stack_size_kb * 1024;
            let stack = StackRestoreGuard::new(size);
            let (mut ptr, actual_size) = stack.stack_area();
            for _ in 0..actual_size {
                unsafe {
                    core::ptr::write_volatile(ptr, 0b10101011);
                    ptr = ptr.add(1)
                }
            }
            assert_eq!(
                actual_size,
                size.div_ceil(page_size()) * page_size()
            )
        }
    }
}