extern crate cc;

use std::env;

fn main() {
    let target = env::var("TARGET").unwrap();

    if target.starts_with("wasm32") {
        // wasm32 auxilary functions are provided as a precompiled object file.
        // this is because LLVM with wasm32 support isn't widespread.
        println!("cargo:rustc-link-search={}", "src/arch/wasm32");
        println!("cargo:rustc-link-lib=stacker");
        return;
    }

    let msvc = target.contains("msvc");
    let mut cfg = cc::Build::new();

    if target.contains("linux") {
        cfg.define("LINUX", None);
    } else if target.contains("darwin") {
        cfg.define("APPLE", None);
    } else if target.contains("windows") {
        cfg.define("WINDOWS", None);
        cfg.file("src/arch/windows.c");
    } else {
        println!("cargo:warning=unsupported platform: {}", target);
        println!("cargo:rustc-cfg=unsupported");
        return
    }

    if target.starts_with("x86_64") {
        cfg.file(if msvc {"src/arch/x86_64.asm"} else {"src/arch/x86_64.S"});
        cfg.define("X86_64", None);
    } else if target.contains("i686") {
        cfg.file(if msvc {"src/arch/i686.asm"} else {"src/arch/i686.S"});
        cfg.define("X86", None);
    } else {
        println!("cargo:warning=unsupported platform: {}", target);
        println!("cargo:rustc-cfg=unsupported");
        return
    }

    cfg.include("src/arch").compile("libstacker.a");
}
