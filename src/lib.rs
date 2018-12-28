//! # **P**ortable **S**tack **M**anipulator
//!
//! This crate provides portable functions to control the stack pointer and inspect the properties
//! of the stack. This crate does not attempt to provide safe abstractions to any operations, the
//! only goals are correctness, portability and efficiency (in that exact order).  As a consequence
//! most functions you’ll see in this crate are unsafe.
//!
//! Note, that the stack allocation is left up to the user. Unless you’re writing a safe
//! abstraction over stack manipulation, this is not the crate you want. Instead consider one of
//! the safe abstractions over this crate. A good place to look at is the crates.io’s reverse
//! dependency list.
// TODO: insert a link to stacker once it is moved on top of this crate.

#![allow(unused_macros)]
#![no_std]

macro_rules! extern_item {
    (unsafe $($toks: tt)+) => {
        unsafe extern "C" $($toks)+
    };
    ($($toks: tt)+) => {
        extern "C" $($toks)+
    };
}

// Surprising: turns out subsequent macro_rules! override previous definitions, instead of
// erroring? Convenient for us in this case, though.
#[cfg(target_arch = "x86_64")]
macro_rules! extern_item {
    (unsafe $($toks: tt)+) => {
        unsafe extern "sysv64" $($toks)+
    };
    ($($toks: tt)+) => {
        extern "sysv64" $($toks)+
    };
}
#[cfg(target_arch = "x86")]
macro_rules! extern_item {
    (unsafe $($toks: tt)+) => {
        unsafe extern "fastcall" $($toks)+
    };
    ($($toks: tt)+) => {
        extern "fastcall" $($toks)+
    };
}

extern_item! { {
    fn rust_psm_stack_direction() -> u8;
    fn rust_psm_stack_pointer() -> *mut u8;
    #[cfg(switchable_stack)]
    fn rust_psm_replace_stack(data: usize, callback: extern_item!(unsafe fn(usize) -> !), sp: *mut u8) -> !;
    #[cfg(switchable_stack)]
    fn rust_psm_on_stack(data: usize, return_ptr: usize,
                         callback: extern_item!(unsafe fn(usize, usize)), sp: *mut u8);
} }

/// Run the closure on the provided stack.
///
/// Once the closure completes its execution, the original stack pointer is restored and execution
/// returns to the caller.
///
/// `base` address must be the low address of the stack memory region, regardless of the stack
/// growth direction. It is not necessary for the whole region `[base; base + size]` to be usable
/// at the time this function called, however it is required that at least the following hold:
///
/// * Both `base` and `base + size` are aligned up to the target-specific requirements;
/// * Depending on `StackDirection` value for the platform, the end of the stack memory region,
///   which would end up containing the first frame(s), must have sufficient number of pages
///   allocated to execute code until more pages are commited. The other end should contain a guard
///   page (not writable, readable or executable) to ensure Rust’s soundness guarantees.
///
/// Note, that some or all of these considerations are irrelevant to some applications. For
/// example, Rust’s soundness story relies on all stacks having a guard-page, however if the user
/// is able to guarantee that the memory region used for stack cannot be exceeded, a guard page may
/// end up being an expensive unnecessity.
///
/// The previous stack may not be deallocated. If an ability to deallocate the old stack is desired
/// consider `replace_stack` instead.
///
/// # Guidelines
///
/// Memory regions that are aligned to a single page (usually 4kB) are an extremely portable choice
/// for stacks.
///
/// Allocate at least 4kB of stack. Some architectures (such as SPARC) consume stack memory
/// significantly faster compared to the more usual architectures such as x86 or ARM. Allocating
/// less than 4kB of memory may make it impossible to commit more pages without overflowing the
/// stack later on.
///
/// # Unsafety
///
/// The stack `base` address must be aligned as appropriate for the target.
///
/// The stack `size` must be a multiple of stack alignment required by target.
///
/// The `size` must not overflow `isize`.
///
/// `callback` must not unwind or return control flow by any other means than directly returning.
///
/// # Examples
///
/// ```
/// use std::alloc;
/// const STACK_ALIGN: usize = 4096;
/// const STACK_SIZE: usize = 4096;
/// unsafe {
///     let layout = alloc::Layout::from_size_align(STACK_SIZE, STACK_ALIGN).unwrap();
///     let new_stack = alloc::alloc(layout);
///     assert!(!new_stack.is_null(), "allocations must succeed!");
///     let (stack, result) = psm::on_stack(new_stack, STACK_SIZE, || {
///         (psm::stack_pointer(), 4 + 4)
///     });
///     println!("4 + 4 = {} has been calculated on stack {:p}", result, stack);
/// }
/// ```
#[cfg(switchable_stack)]
pub unsafe fn on_stack<R, F: FnOnce() -> R>(base: *mut u8, size: usize, callback: F) -> R {
    extern_item!{ unsafe fn with_on_stack<R, F: FnOnce() -> R>(d: usize, return_ptr: usize) {
        // Safe to move out from `F`, because closure in is forgotten in `on_stack` and dropping
        // only occurs in this callback.
        ::core::ptr::write(return_ptr as *mut R, ::core::ptr::read(d as *const F)());
    } }
    let sp = match StackDirection::new() {
        StackDirection::Ascending => base,
        StackDirection::Descending => base.offset(size as isize),
    };
    // FIXME: Use MaybeUninit once it is stable...
    let mut return_value: R = ::core::mem::uninitialized();
    rust_psm_on_stack(&callback as *const F as usize, &mut return_value as *mut R as usize,
                      with_on_stack::<R, F>, sp);
    // Moved out in with_on_stack by `ptr::read`.
    ::core::mem::forget(callback);
    return return_value;
}

/// Run the provided non-terminating computation on an entirely new stack.
///
/// `base` address must be the low address of the stack memory region, regardless of the stack
/// growth direction. It is not necessary for the whole region `[base; base + size]` to be usable
/// at the time this function called, however it is required that at least the following hold:
///
/// * Both `base` and `base + size` are aligned up to the target-specific requirements;
/// * Depending on `StackDirection` value for the platform, the end of the stack memory region,
///   which would end up containing the first frame(s), must have sufficient number of pages
///   allocated to execute code until more pages are commited. The other end should contain a guard
///   page (not writable, readable or executable) to ensure Rust’s soundness guarantees.
///
/// Note, that some or all of these considerations are irrelevant to some applications. For
/// example, Rust’s soundness story relies on all stacks having a guard-page, however if the user
/// is able to guarantee that the memory region used for stack cannot be exceeded, a guard page may
/// end up being an expensive unnecessity.
///
/// The previous stack is not deallocated and may not be deallocated unless the data on the old
/// stack is not referenced in any way (by e.g. the `callback` closure).
///
/// On platforms where multiple stack pointers are available, the “current” stack pointer is
/// replaced.
///
/// # Guidelines
///
/// Memory regions that are aligned to a single page (usually 4kB) are an extremely portable choice
/// for stacks.
///
/// Allocate at least 4kB of stack. Some architectures (such as SPARC) consume stack memory
/// significantly faster compared to the more usual architectures such as x86 or ARM. Allocating
/// less than 4kB of memory may make it impossible to commit more pages without overflowing the
/// stack later on.
///
/// # Unsafety
///
/// The stack `base` address must be aligned as appropriate for the target.
///
/// The stack `size` must be a multiple of stack alignment required by target.
///
/// The `size` must not overflow `isize`.
///
/// `callback` must not return (not enforced by typesystem currently because `!` is unstable),
/// unwind or otherwise return control flow to any of the previous frames.
#[cfg(switchable_stack)]
pub unsafe fn replace_stack<F: FnOnce()>(base: *mut u8, size: usize, callback: F) -> ! {
    extern_item! { unsafe fn with_replaced_stack<F: FnOnce()>(d: usize) -> ! {
        // Safe to move out, because the closure is essentially forgotten by
        // this being required to never return...
        ::core::ptr::read(d as *const F)();
        ::core::hint::unreachable_unchecked();
    } }
    let sp = match StackDirection::new() {
        StackDirection::Ascending => base,
        StackDirection::Descending => base.offset(size as isize),
    };
    rust_psm_replace_stack(&callback as *const F as usize, with_replaced_stack::<F>, sp);
}

/// The direction into which stack grows as stack frames are made.
///
/// This is a target-specific property that can be obtained at runtime by calling
/// `StackDirection::new()`.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum StackDirection {
    Ascending = 1,
    Descending = 2,
}

impl StackDirection {
    /// Obtain the stack growth direction.
    #[inline(always)]
    pub fn new() -> StackDirection {
        const ASC: u8 = StackDirection::Ascending as u8;
        const DSC: u8 = StackDirection::Descending as u8;

        // FIXME: consider adding ability to cache this :)
        unsafe {
            match rust_psm_stack_direction() {
                ASC => StackDirection::Ascending,
                DSC => StackDirection::Descending,
                _ => ::core::hint::unreachable_unchecked(),
            }
        }
    }
}

/// Returns current stack pointer.
///
/// Note, that the stack pointer returned is from the perspective of the caller. From the
/// perspective of `stack_pointer` function the pointer returned is the frame pointer.
///
/// While it is a goal to minimize the amount of stack used by this function, implementations for
/// some targets may be unable to avoid allocating a stack frame. This makes this function
/// suitable for stack exhaustion detection only in conjunction with sufficient padding.
///
/// Using `stack_pointer` to check for stack exhaustion is tricky to get right. It is impossible to
/// know the callee’s frame size, therefore such value must be derived some other way. A common
/// approach is to use stack padding (reserve enough stack space for any function to be called) and
/// check against the padded threshold. If padding is chosen incorrectly, a situation similar to
/// one described below may occur:
///
/// 1. For stack exhaustion check, remaining stack is checked against `stack_pointer` with the
///    padding applied;
/// 2. Callee allocates more stack than was accounted for with padding, and accesses pages outside
///    the stack, crashing the program.
#[inline(always)]
pub fn stack_pointer() -> *mut u8 {
    unsafe {
        rust_psm_stack_pointer()
    }
}
