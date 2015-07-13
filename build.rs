extern crate gcc;

use std::env;

fn main() {
    let target = env::var("TARGET").unwrap();

    let mut cfg = gcc::Config::new();

    if target.contains("linux") {
        cfg.define("LINUX", None);
    } else if target.contains("darwin") {
        cfg.define("APPLE", None);
    // } else if target.contains("windows") {
    //     cfg.define("WINDOWS", None);
    } else {
        panic!("\n\nusing currently unsupported target triple with \
                stacker: {}\n\n", target);
    }

    if target.starts_with("x86_64") {
        cfg.file("src/arch/x86_64.S");
    } else if target.contains("i686") {
        cfg.file("src/arch/i686.S");
    } else {
        panic!("\n\nusing currently unsupported target triple with \
                stacker: {}\n\n", target);
    }

    cfg.include("src/arch").compile("libstacker.a");
}
