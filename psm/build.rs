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

/// Checks if we have the Microsoft Macro Assember for the target architecture
fn has_msvc_assembler(arch: &str) -> bool {
    // See https://github.com/alexcrichton/cc-rs/blob/030baede12f079793cc50d6f90b677b889c49fcd/src/lib.rs#L1854-L1864
    // for where these values come from
    let assembler = match arch {
        "x86_64" => "ml64.exe",
        "arm" => "armasm.exe",
        "aarch64" => "armasm64.exe",
        _ => "ml.exe",
    };

    // We don't care of the command succeeds, only if we can run it
    std::process::Command::new(assembler)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map_or(false, |_s| true)
}

fn main() {
    let arch = ::std::env::var("CARGO_CFG_TARGET_ARCH").unwrap();
    let env = ::std::env::var("CARGO_CFG_TARGET_ENV").unwrap();
    let os = ::std::env::var("CARGO_CFG_TARGET_OS").unwrap();
    let endian = ::std::env::var("CARGO_CFG_TARGET_ENDIAN").unwrap();

    let mut cfg = cc::Build::new();

    let msvc = cfg.get_compiler().is_like_msvc();
    // If we're targeting msvc, either via regular MS toolchain or clang-cl, we
    // will _usually_ want to use the regular Microsoft assembler if it exists,
    // however it _probably_ won't exist if we're in a cross-compilation context
    // from a platform that can't natively run Windows executables, so in that
    // case we instead use the the equivalent GAS assembly file
    let masm = msvc && has_msvc_assembler(&arch);

    let asm = if let Some((asm, canswitch)) = find_assembly(&arch, &endian, &os, &env, masm) {
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
