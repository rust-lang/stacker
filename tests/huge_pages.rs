#![cfg(all(target_os = "linux", not(miri)))]

extern crate stacker;

use std::thread;

#[test]
fn huge_page_stack_is_rounded_up() {
    let size = match std::fs::read_to_string("/sys/kernel/mm/transparent_hugepage/hpage_pmd_size")
    {
        Ok(size) => size.trim().parse::<usize>().unwrap(),
        Err(_) => return,
    };
    stacker::set_huge_pages(true);
    thread::spawn(move || {
        stacker::grow(64 * 1024, || {
            assert!(stacker::remaining_stack().unwrap() > size / 2);
        });
    })
    .join()
    .unwrap();
}
