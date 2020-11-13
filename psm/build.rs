extern crate cc;

fn find_assembly(
    arch: &str,
    endian: &str,
    os: &str,
    env: &str,
    masm: bool,
) -> Option<(&'static str, bool)> {
    match (arch, endian, os, env) {
        // The implementations for stack switching exist, but, officially, doing so without Fibers
        // is not supported in Windows. For x86_64 the implementation actually works locally,
        // but failed tests in CI (???). Might want to have a feature for experimental support
        // here.
        ("x86", _, "windows", "msvc") => {
            if masm {
                Some(("src/arch/x86_msvc.asm", false))
            } else {
                Some(("src/arch/x86_windows_gnu.s", false))
            }
        }
        ("x86_64", _, "windows", "msvc") => {
            if masm {
                Some(("src/arch/x86_64_msvc.asm", false))
            } else {
                Some(("src/arch/x86_64_windows_gnu.s", false))
            }
        }
        ("arm", _, "windows", "msvc") => Some(("src/arch/arm_armasm.asm", false)),
        ("aarch64", _, "windows", "msvc") => {
            if masm {
                Some(("src/arch/aarch64_armasm.asm", false))
            } else {
                Some(("src/arch/aarch_aapcs64.s", false))
            }
        }
        ("x86", _, "windows", _) => Some(("src/arch/x86_windows_gnu.s", false)),
        ("x86_64", _, "windows", _) => Some(("src/arch/x86_64_windows_gnu.s", false)),

        ("x86", _, _, _) => Some(("src/arch/x86.s", true)),
        ("x86_64", _, _, _) => Some(("src/arch/x86_64.s", true)),
        ("arm", _, _, _) => Some(("src/arch/arm_aapcs.s", true)),
        ("aarch64", _, _, _) => Some(("src/arch/aarch_aapcs64.s", true)),
        ("powerpc", _, _, _) => Some(("src/arch/powerpc32.s", true)),
        ("powerpc64", "little", _, _) => Some(("src/arch/powerpc64_openpower.s", true)),
        ("powerpc64", _, _, _) => Some(("src/arch/powerpc64.s", true)),
        ("s390x", _, _, _) => Some(("src/arch/zseries_linux.s", true)),
        ("mips", _, _, _) => Some(("src/arch/mips_eabi.s", true)),
        ("mips64", _, _, _) => Some(("src/arch/mips64_eabi.s", true)),
        ("sparc64", _, _, _) => Some(("src/arch/sparc64.s", true)),
        ("sparc", _, _, _) => Some(("src/arch/sparc_sysv.s", true)),
        ("riscv32", _, _, _) => Some(("src/arch/riscv.s", true)),
        ("riscv64", _, _, _) => Some(("src/arch/riscv64.s", true)),
        ("wasm32", _, _, _) => Some(("src/arch/wasm32.o", true)),
        _ => None,
    }
}

fn main() {
    let arch = ::std::env::var("CARGO_CFG_TARGET_ARCH").unwrap();
    let env = ::std::env::var("CARGO_CFG_TARGET_ENV").unwrap();
    let os = ::std::env::var("CARGO_CFG_TARGET_OS").unwrap();
    let endian = ::std::env::var("CARGO_CFG_TARGET_ENDIAN").unwrap();

    // We are only assembling a single file and any flags in the environment probably
    // don't apply in this case, so we don't want to use them. Unfortunately, cc
    // doesn't provide a way to clear/ignore flags set from the environment, so
    // we manually remove them instead
    for key in
        std::env::vars().filter_map(|(k, _)| if k.contains("CFLAGS") { Some(k) } else { None })
    {
        std::env::remove_var(key);
    }

    let mut cfg = cc::Build::new();
    // There seems to be a bug with clang-cl where using
    // `/clang:-xassembler-with-cpp` is apparently accepted (ie no warnings
    // about unused/unknown arguments), but results in the same exact error
    // as if the flag was not present, so instead we take advantage of the
    // fact that we're not actually compiling any C/C++ code, only assembling
    // and can just use clang directly.
    if cfg
        .get_compiler()
        .path()
        .file_name()
        .and_then(|f| f.to_str())
        .map(|fname| fname.contains("clang-cl"))
        .unwrap_or(false)
    {
        cfg.compiler("clang");
    }

    let msvc = cfg.get_compiler().is_like_msvc();
    let asm =
        if let Some((asm, canswitch)) = find_assembly(&arch, &endian, &os, &env, msvc) {
            println!("cargo:rustc-cfg=asm");
            if canswitch {
                println!("cargo:rustc-cfg=switchable_stack")
            }
            asm
        } else {
            println!(
                "cargo:warning=Target {}-{}-{} has no assembly files!",
                arch, os, env
            );
            return;
        };

    if !msvc {
        cfg.flag("-xassembler-with-cpp");
        cfg.define(&*format!("CFG_TARGET_OS_{}", os), None);
        cfg.define(&*format!("CFG_TARGET_ARCH_{}", arch), None);
        cfg.define(&*format!("CFG_TARGET_ENV_{}", env), None);
    }

    // For wasm targets we ship a precompiled `*.o` file so we just pass that
    // directly to `ar` to assemble an archive. Otherwise we're actually
    // compiling the source assembly file.
    if asm.ends_with(".o") {
        cfg.object(asm);
    } else {
        cfg.file(asm);
    }

    cfg.compile("libpsm_s.a");
}
