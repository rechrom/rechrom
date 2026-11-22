//! Shared helpers of the C differential oracles; compiled only for tests.
pub(crate) mod c_source;

// The directory of the official, unmodified QuickJS release recorded in crates/quickjs/UPSTREAM.json.
// The release is not stored in the repository. It is downloaded once into the
// Cargo target directory, verified against the pinned SHA-256 and reused.
// Set RECHROM_QUICKJS_UPSTREAM_DIR to use an already extracted copy.
pub(crate) fn quickjs_upstream_source() -> std::path::PathBuf {
    const RELEASE: &str = "quickjs-2026-06-04";
    const URL: &str = "https://bellard.org/quickjs/quickjs-2026-06-04.tar.xz";
    const SHA256: &str = "b376e839b322978313d929fd20663b11ba58b75df5a46c126dd19ea2fa70ad2a";
    let manifest = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    if let Some(dir) = std::env::var_os("RECHROM_QUICKJS_UPSTREAM_DIR") {
        std::path::PathBuf::from(dir)
    } else {
        let cache = std::env::var_os("CARGO_TARGET_DIR")
            .map(std::path::PathBuf::from)
            .unwrap_or_else(|| manifest.join("../../target"))
            .join("quickjs-upstream");
        let source = cache.join(RELEASE);
        if !source.join("quickjs.c").exists() {
            // Each process stages privately, then renames into place, so
            // concurrently running test binaries never observe a partial tree.
            let staging = cache.join(format!(
                "staging-{}-{}",
                std::process::id(),
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_nanos()
            ));
            std::fs::create_dir_all(&staging).unwrap();
            let archive = "quickjs.tar.xz";
            let status = std::process::Command::new("curl")
                .args(["-sSfL", "-o", archive, URL])
                .current_dir(&staging)
                .status()
                .expect("curl is required to fetch the QuickJS oracle source");
            assert!(status.success(), "failed to download {URL}");
            let bytes = std::fs::read(staging.join(archive)).unwrap();
            let digest = <sha2::Sha256 as sha2::Digest>::digest(&bytes);
            let digest = digest
                .iter()
                .map(|byte| format!("{byte:02x}"))
                .collect::<String>();
            assert_eq!(digest, SHA256, "{URL} does not match the pinned release");
            // Relative paths only: GNU tar treats `C:` in a path as a host.
            let status = std::process::Command::new("tar")
                .args(["-xJf", archive])
                .current_dir(&staging)
                .status()
                .expect("tar with xz support is required to unpack the QuickJS oracle");
            assert!(status.success(), "failed to unpack {archive}");
            if std::fs::rename(staging.join(RELEASE), &source).is_err() {
                assert!(
                    source.join("quickjs.c").exists(),
                    "could not install {}",
                    source.display()
                );
            }
            let _ = std::fs::remove_dir_all(&staging);
        }
        source
    }
}

// The C compiler command shared by every oracle. Official source stays read-only.
// quickjs.c defines SHORT_OPCODES and CONFIG_ATOMICS internally, so a -D/-U
// alone cannot override those two settings. Only this temporary configuration
// copy changes their preprocessor values; every C algorithm remains unchanged.
pub(crate) fn quickjs_oracle_compiler() -> std::process::Command {
    let mut compiler = std::process::Command::new("clang");
    let short = cfg!(feature = "short-opcodes");
    let atomics = cfg!(feature = "atomics");
    let iter = cfg!(feature = "malloc-iter");
    let dump = cfg!(feature = "dump-read-object");
    let force_gc = cfg!(feature = "force-gc-at-malloc");
    let large = cfg!(feature = "malloc-large-blocks");
    compiler.arg(if short { "-DSHORT_OPCODES=1" } else { "-DSHORT_OPCODES=0" });
    compiler.arg(if atomics { "-DCONFIG_ATOMICS=1" } else { "-UCONFIG_ATOMICS" });
    // Upstream guards this macro with #ifdef, so defining it as 0 enables it.
    compiler.arg(if iter { "-DJS_MALLOC_USE_ITER=1" } else { "-UJS_MALLOC_USE_ITER" });
    compiler.arg(if dump {"-DDUMP_READ_OBJECT=1"} else {"-UDUMP_READ_OBJECT"});
    compiler.arg(if force_gc {"-DFORCE_GC_AT_MALLOC=1"} else {"-UFORCE_GC_AT_MALLOC"});
    // Only select the original allocation policy; this gate does not enable
    // compiler sanitizer instrumentation on either implementation.
    compiler.arg(if large {"-D__SANITIZE_ADDRESS__=1"} else {"-U__SANITIZE_ADDRESS__"});
    let upstream = quickjs_upstream_source();
    let directory = std::env::temp_dir().join(format!(
        "quickjs-oracle-feature-{}-{}{}{}{}{}{}", std::process::id(), short as u8,
        atomics as u8, iter as u8, dump as u8, force_gc as u8, large as u8));
    std::fs::create_dir_all(&directory).unwrap();
    let configured = directory.join("quickjs.c");
    if !configured.exists() {
        let mut source = std::fs::read_to_string(upstream.join("quickjs.c")).unwrap();
        if !short { source = source.replace("#define SHORT_OPCODES    1", "#define SHORT_OPCODES    0"); }
        if !atomics { source = source.replace("#define CONFIG_ATOMICS", "/* CONFIG_ATOMICS disabled by Cargo feature */"); }
        // Upstream's disabled diagnostic has one stale field spelling.
        if dump { source = source.replace("bc_read_trace(s, \"source: %d bytes\\n\", b->source_len);", "bc_read_trace(s, \"source: %d bytes\\n\", b->debug.source_len);"); }
        let pending = directory.join(format!("pending-{:?}.c", std::thread::current().id()));
        std::fs::write(&pending, source).unwrap();
        // Concurrent oracle compilers see only a complete, identical copy.
        std::fs::rename(&pending, &configured).unwrap();
    }
    compiler.arg("-I").arg(directory);
    compiler
}
