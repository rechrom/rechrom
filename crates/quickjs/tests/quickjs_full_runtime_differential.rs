// Exercise production definitions and the original C engine as independent executables.
use super::*;
include!("quickjs_combined_eval_fixtures.rs");

#[test]
fn full_standard_context_executes_official_c_programs() {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let source = root.join("../../vendor/quickjs-2026-06-04");
    let out = std::env::temp_dir().join(format!("quickjs-full-context-{}", std::process::id()));
    std::fs::create_dir_all(&out).unwrap();
    let executable = out.join("official-c");
    let mut compiler = include!("quickjs_oracle_config.rs");
    compiler.args([
        "-std=gnu11",
        "-O1",
        "-DCONFIG_VERSION=\"2026-06-04\"",
        if cfg!(feature = "short-opcodes") {
            "-DSHORT_OPCODES=1"
        } else {
            "-DSHORT_OPCODES=0"
        },
    ]);
    compiler
        .arg("-I")
        .arg(&source)
        .arg(root.join("tests/quickjs_combined_eval_oracle.c"));
    for file in ["cutils.c", "libunicode.c", "libregexp.c", "dtoa.c"] {
        compiler.arg(source.join(file));
    }
    let compiled = compiler
        .arg("-lm")
        .arg("-o")
        .arg(&executable)
        .output()
        .unwrap();
    assert!(
        compiled.status.success(),
        "{}",
        String::from_utf8_lossy(&compiled.stderr)
    );
    let oracle = std::process::Command::new(&executable)
        .arg(root.join("tests/quickjs_combined_eval_cases.txt"))
        .output()
        .unwrap();
    assert!(
        oracle.status.success(),
        "{}",
        String::from_utf8_lossy(&oracle.stderr)
    );
    let actual = std::thread::Builder::new()
        .stack_size(32 * 1024 * 1024)
        .spawn(|| unsafe { combined_eval_fixture() })
        .unwrap()
        .join()
        .unwrap();
    assert_eq!(
        actual, oracle.stdout,
        "production engine differs from official C"
    );
    let mut offset = 0;
    let mut count = 0;
    while offset < actual.len() {
        let id = u32::from_le_bytes(actual[offset..offset + 4].try_into().unwrap());
        let tag = u32::from_le_bytes(actual[offset + 4..offset + 8].try_into().unwrap());
        let len = u32::from_le_bytes(actual[offset + 8..offset + 12].try_into().unwrap()) as usize;
        assert_eq!(id as usize, count);
        let script = include_str!("quickjs_combined_eval_cases.txt")
            .lines()
            .nth(id as usize)
            .unwrap();
        if !cfg!(feature = "atomics") && script.contains("Atomics.add") {
            assert_eq!(
                tag, JS_TAG_EXCEPTION as u32,
                "disabled Atomics must throw in case {id}"
            );
        } else {
            assert_ne!(
                tag, JS_TAG_EXCEPTION as u32,
                "unexpected exception in case {id}"
            );
        }
        offset += 12 + len;
        count += 1;
    }
    assert_eq!(
        count,
        include_str!("quickjs_combined_eval_cases.txt")
            .lines()
            .count()
    );
    let _ = std::fs::remove_file(executable);
}
