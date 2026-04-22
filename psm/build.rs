fn find_assembly(
    arch: &str,
    endian: &str,
    os: &str,
    env: &str,
) -> Option<(&'static str, bool)> {
    match (arch, endian, os, env) {
        // Windows: inline asm in src/asm/windows/. Stack switching (canswitch)
        // is disabled — officially requires Fibers.
        ("x86", _, "windows", _) => None,
        ("x86_64", _, "windows", _) => None,
        ("aarch64", _, "windows", _) => None,
        ("x86_64", _, "cygwin", _) => Some(("src/arch/x86_64_windows_gnu.s", false)),
        ("arm", _, "windows", "msvc") => Some(("src/arch/arm_armasm.asm", false)),
        ("arm64ec", _, "windows", "msvc") => Some(("src/arch/arm64ec_armasm.asm", false)),
        ("x86", _, _, _) => Some(("src/arch/x86.s", true)),
        ("x86_64", _, _, _) => Some(("src/arch/x86_64.s", true)),
        ("arm", _, _, _) => Some(("src/arch/arm_aapcs.s", true)),
        ("aarch64", _, _, _) => Some(("src/arch/aarch_aapcs64.s", true)),
        ("powerpc", _, _, _) => Some(("src/arch/powerpc32.s", true)),
        ("powerpc64", _, _, "musl") => Some(("src/arch/powerpc64_openpower.s", true)),
        ("powerpc64", "little", _, _) => Some(("src/arch/powerpc64_openpower.s", true)),
        ("powerpc64", _, "aix", _) => Some(("src/arch/powerpc64_aix.s", true)),
        ("powerpc64", _, _, _) => Some(("src/arch/powerpc64.s", true)),
        ("s390x", _, _, _) => Some(("src/arch/zseries_linux.s", true)),
        ("mips", _, _, _) => Some(("src/arch/mips_eabi.s", true)),
        ("mips64", _, _, _) => Some(("src/arch/mips64_eabi.s", true)),
        ("sparc64", _, _, _) => Some(("src/arch/sparc64.s", true)),
        ("sparc", _, _, _) => Some(("src/arch/sparc_sysv.s", true)),
        ("riscv32", _, _, _) => Some(("src/arch/riscv.s", true)),
        ("riscv64", _, _, _) => Some(("src/arch/riscv64.s", true)),
        ("wasm32", _, _, _) => Some(("src/arch/wasm32.o", true)),
        ("loongarch64", _, _, _) => Some(("src/arch/loongarch64.s", true)),
        _ => None,
    }
}

fn main() {
    use std::env::var;

    println!("cargo:rustc-check-cfg=cfg(switchable_stack,asm,link_asm)");

    if var("CARGO_CFG_MIRI").is_ok() {
        // Miri doesn't have a stack limit and the inline asm wouldn't work on miri anyway.
        return;
    }

    let arch = var("CARGO_CFG_TARGET_ARCH").unwrap();
    let env = var("CARGO_CFG_TARGET_ENV").unwrap();
    let os = var("CARGO_CFG_TARGET_OS").unwrap();
    let endian = var("CARGO_CFG_TARGET_ENDIAN").unwrap();

    let mut cfg = cc::Build::new();

    let msvc = cfg.get_compiler().is_like_msvc();
    let asm = if let Some((asm, canswitch)) = find_assembly(&arch, &endian, &os, &env) {
        println!("cargo:rustc-cfg=asm");
        println!("cargo:rustc-cfg=link_asm");
        if canswitch {
            println!("cargo:rustc-cfg=switchable_stack")
        }
        asm
    } else if (arch == "aarch64" || arch == "x86_64" || arch == "x86") && os == "windows" {
        println!("cargo:rustc-cfg=asm");
        return;
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
        use ar_archive_writer::{
            write_archive_to_stream, ArchiveKind, NewArchiveMember, ObjectReader,
        };
        use std::fs::{metadata, read};
        use std::io::Cursor;
        use std::path::PathBuf;

        let out_dir = std::env::var("OUT_DIR").expect("OUT_DIR environment variable not set");
        let output_path = PathBuf::from(&out_dir).join("libpsm_s.a");

        let object_data = read(asm).expect("Failed to read object file");
        let file_metadata = metadata(asm).expect("Failed to read file metadata");

        // Extract file metadata
        let mtime = file_metadata
            .modified()
            .ok()
            .and_then(|time| time.duration_since(std::time::UNIX_EPOCH).ok())
            .map(|duration| duration.as_secs())
            .unwrap_or(0);

        #[cfg(unix)]
        let (uid, gid, perms) = {
            use std::os::unix::fs::MetadataExt;
            (
                file_metadata.uid(),
                file_metadata.gid(),
                file_metadata.mode(),
            )
        };

        #[cfg(not(unix))]
        let (uid, gid, perms) = (0, 0, 0o644);

        let filename = asm.rsplit('/').next().unwrap_or(asm);
        let member = NewArchiveMember {
            buf: Box::new(object_data),
            object_reader:
                &ObjectReader {
                    get_symbols: |_data: &[u8],
                                  _callback: &mut dyn FnMut(
                        &[u8],
                    )
                        -> Result<(), std::io::Error>| Ok(true),
                    get_xcoff_member_alignment: |_data| 0,
                    is_64_bit_object_file: |_data| false,
                    is_any_arm64_coff: |_data| false,
                    is_ec_object_file: |_data| false,
                },
            member_name: filename.to_string(),
            mtime,
            uid,
            gid,
            perms,
        };

        let mut output_bytes = Cursor::new(Vec::new());
        write_archive_to_stream(
            &mut output_bytes,
            &[member],
            // Unfortunately, getDefaultKind() is not available in ar_archive_writer
            // however looking at the llvm-ar source it looks like for wasm32-any-any
            // it falls through to Gnu https://llvm.org/doxygen/Object_2Archive_8cpp_source.html
            ArchiveKind::Gnu,
            false,
            None,
        )
        .expect("Failed to write archive");

        std::fs::write(&output_path, output_bytes.into_inner())
            .expect("Failed to write archive file");

        println!("cargo:rustc-link-search=native={}", out_dir);
        println!("cargo:rustc-link-lib=static=psm_s");
    } else {
        cfg.file(asm);
        cfg.compile("libpsm_s.a");
    }
}
