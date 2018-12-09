extern crate cc;

fn find_assembly(arch: &str, env: &str) -> Option<&'static str> {
    const ARCH_ENV_IMPL_MAP: [((&str, &str), Option<&str>); 8] = [
        (("*", "msvc"), None),
        (("x86", "*"), Some("src/arch/x86.s")),
        (("x86_64", "*"), Some("src/arch/x86_64.s")),
        (("arm", "*"), Some("src/arch/arm_aapcs.s")),
        (("armv7", "*"), Some("src/arch/arm_aapcs.s")),
        (("thumbv6", "*"), Some("src/arch/arm_aapcs.s")),
        (("thumbv7", "*"), Some("src/arch/arm_aapcs.s")),
        (("aarch64", "*"), Some("src/arch/aarch_aapcs64.s")),
    ];

    for ((exparch, expenv), file) in &ARCH_ENV_IMPL_MAP {
        if (exparch == &arch || exparch == &"*") && (expenv == &env || expenv == &"*") {
            return *file;
        }
    }
    None
}

fn main() {
    let arch = ::std::env::var("CARGO_CFG_TARGET_ARCH").unwrap();
    let env = ::std::env::var("CARGO_CFG_TARGET_ENV").unwrap();
    let os = ::std::env::var("CARGO_CFG_TARGET_OS").unwrap();
    let asm = if let Some(asm) = find_assembly(&arch, &env) {
        asm
    } else {
        eprintln!("({}, {}) arch-env pair is not supported", arch, env);
        ::std::process::abort();
    };

    let mut cfg = cc::Build::new();
    cfg.flag("-xassembler-with-cpp");
    cfg.define("CFG_TARGET_ARCH", Some(&*arch));
    cfg.define("CFG_TARGET_ENV", Some(&*env));
    cfg.define("CFG_TARGET_OS", Some(&*os));
    cfg.file(asm);
    cfg.compile("libpsm_s.a");

    // let msvc = target.contains("msvc");


    // if target.contains("linux") {
    //     cfg.define("LINUX", None);
    // } else if target.contains("darwin") {
    //     cfg.define("APPLE", None);
    // } else if target.contains("windows") {
    //     cfg.define("WINDOWS", None);
    // } else {
    //     panic!("\n\nusing currently unsupported target triple with \
    //             stacker: {}\n\n", target);
    // }

    // if target.starts_with("x86_64") {
    //     cfg.file(if msvc {"src/arch/x86_64.asm"} else {"src/arch/x86_64.S"});
    //     cfg.define("X86_64", None);
    // } else if target.contains("i686") {
    //     cfg.file(if msvc {"src/arch/i686.asm"} else {"src/arch/i686.S"});
    //     cfg.define("X86", None);
    // } else {
    //     panic!("\n\nusing currently unsupported target triple with \
    //             stacker: {}\n\n", target);
    // }

    // cfg.include("src/arch").compile("libstacker.a");
}
