extern crate gcc;

use std::env;

fn main() {
    let target = env::var("TARGET").unwrap();

    let mut cfg = gcc::Config::new();

    if target.starts_with("x86_64") {
        cfg.file("src/arch/x86_64.S");
    } else if target.contains("i686") {
        cfg.file("src/arch/i686.S");
    }

    cfg.include("src/arch").compile("libstacker.a");
}
