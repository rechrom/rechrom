// Expression shared by every test C compiler. Official source stays read-only.
// quickjs.c defines SHORT_OPCODES and CONFIG_ATOMICS internally, so a -D/-U
// alone cannot override those two settings. Only this temporary configuration
// copy changes their preprocessor values; every C algorithm remains unchanged.
{
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
    let upstream = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../vendor/quickjs-2026-06-04");
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
