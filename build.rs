extern crate cc;

fn find_assembly(arch: &str, endian: &str, os: &str, env: &str) -> Option<(&'static str, bool)> {
    match (arch, endian, os, env) {
        ("x86",        _,        "windows", "msvc") => Some(("src/arch/x86_msvc.s", false)),
        ("x86_64",     _,        "windows", "msvc") => Some(("src/arch/x86_64_msvc.s", false)),
        ("arm",        _,        "windows", "msvc") => Some(("src/arch/arm_armasm.s", false)),
        ("aarch64",    _,        "windows", "msvc") => Some(("src/arch/aarch64_armasm.s", false)),
        ("x86",        _,        "windows", _)      => Some(("src/arch/x86.s", false)),
        ("x86_64",     _,        "windows", _)      => Some(("src/arch/x86_64.s", false)),

        ("x86",        _,        _,         _) => Some(("src/arch/x86.s", true)),
        ("x86_64",     _,        _,         _) => Some(("src/arch/x86_64.s", true)),
        ("arm",        _,        _,         _) => Some(("src/arch/arm_aapcs.s", true)),
        ("armv7",      _,        _,         _) => Some(("src/arch/arm_aapcs.s", true)),
        ("thumbv6",    _,        _,         _) => Some(("src/arch/arm_aapcs.s", true)),
        ("thumbv7",    _,        _,         _) => Some(("src/arch/arm_aapcs.s", true)),
        ("aarch64",    _,        _,         _) => Some(("src/arch/aarch_aapcs64.s", true)),
        ("powerpc",    _,        _,         _) => Some(("src/arch/powerpc32.s", true)),
        ("powerpc64",  "little", _,         _) => Some(("src/arch/powerpc64_openpower.s", true)),
        ("powerpc64",  _,        _,         _) => Some(("src/arch/powerpc64.s", true)),
        ("s390x",      _,        _,         _) => Some(("src/arch/zseries_linux.s", true)),
        ("mips",       _,        _,         _) => Some(("src/arch/mips_eabi.s", true)),
        ("mips64",     _,        _,         _) => Some(("src/arch/mips64_eabi.s", true)),
        ("sparc64",    _,        _,         _) => Some(("src/arch/sparc64.s", true)),
        ("sparc",    _,          _,         _) => Some(("src/arch/sparc_sysv.s", true)),
        _ => None,
    }
}

fn main() {
    let arch = ::std::env::var("CARGO_CFG_TARGET_ARCH").unwrap();
    let env = ::std::env::var("CARGO_CFG_TARGET_ENV").unwrap();
    let os = ::std::env::var("CARGO_CFG_TARGET_OS").unwrap();
    let endian = ::std::env::var("CARGO_CFG_TARGET_ENDIAN").unwrap();
    let (asm, canswitch) = if let Some(val) = find_assembly(&arch, &endian, &os, &env) {
        val
    } else {
        eprintln!("Target {}-{}-{} is not supported", arch, os, env);
        ::std::process::abort();
    };

    if canswitch {
        println!("cargo:rustc-cfg=switchable_stack")
    }

    let mut cfg = cc::Build::new();
    let msvc = cfg.get_compiler().is_like_msvc();

    if !msvc {
        cfg.flag("-xassembler-with-cpp");
    }
    cfg.file(asm);

    cfg.define(&*format!("CFG_TARGET_OS_{}", os), None);
    cfg.define(&*format!("CFG_TARGET_ARCH_{}", arch), None);
    cfg.define(&*format!("CFG_TARGET_ENV_{}", env), None);

    cfg.compile("libpsm_s.a");
}
