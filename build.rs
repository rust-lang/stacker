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
    cfg.file(asm);
    cfg.define(&*format!("CFG_TARGET_OS_{}", os), None);
    cfg.define(&*format!("CFG_TARGET_ARCH_{}", arch), None);
    cfg.define(&*format!("CFG_TARGET_ENV_{}", env), None);

    cfg.compile("libpsm_s.a");
}
