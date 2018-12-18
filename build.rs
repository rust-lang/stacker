extern crate cc;

fn find_assembly(arch: &str, endian: &str, env: &str) -> Option<&'static str> {
    match (arch, endian, env) {
        (_,            _,        "msvc") => None,
        ("x86",        _,        _) => Some("src/arch/x86.s"),
        ("x86_64",     _,        _) => Some("src/arch/x86_64.s"),
        ("arm",        _,        _) => Some("src/arch/arm_aapcs.s"),
        ("armv7",      _,        _) => Some("src/arch/arm_aapcs.s"),
        ("thumbv6",    _,        _) => Some("src/arch/arm_aapcs.s"),
        ("thumbv7",    _,        _) => Some("src/arch/arm_aapcs.s"),
        ("aarch64",    _,        _) => Some("src/arch/aarch_aapcs64.s"),
        ("powerpc",    _,        _) => Some("src/arch/powerpc32.s"),
        ("powerpc64",  "little", _) => Some("src/arch/powerpc64_openpower.s"),
        ("powerpc64",  _,        _) => Some("src/arch/powerpc64.s"),
        ("s390x",      _,        _) => Some("src/arch/zseries_linux.s"),
        ("mips",       _,        _) => Some("src/arch/mips_eabi.s"),
        ("mips64",     _,        _) => Some("src/arch/mips64_eabi.s"),
        _ => None,
    }
}

fn main() {
    let arch = ::std::env::var("CARGO_CFG_TARGET_ARCH").unwrap();
    let env = ::std::env::var("CARGO_CFG_TARGET_ENV").unwrap();
    let os = ::std::env::var("CARGO_CFG_TARGET_OS").unwrap();
    let endian = ::std::env::var("CARGO_CFG_TARGET_ENDIAN").unwrap();
    let asm = if let Some(asm) = find_assembly(&arch, &endian, &env) {
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
