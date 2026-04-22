#[cfg(target_os = "windows")]
mod windows;
#[cfg(target_os = "windows")]
pub(crate) use windows::{rust_psm_stack_direction, rust_psm_stack_pointer};
#[cfg(all(target_os = "windows", switchable_stack))]
pub(crate) use windows::{rust_psm_on_stack, rust_psm_replace_stack};
