// Exercise production definitions and the original C engine as independent executables.
use super::*;
include!("quickjs_combined_eval_fixtures.rs");

#[test]
fn full_standard_context_executes_official_c_programs() {
    // A parser normally removes NOPs. The VM must also accept consecutive
    // NOP bytecodes (C OP_nop -> BREAK), including at the initial restart.
    unsafe {
        let rt = JS_NewRuntime();
        let ctx = JS_NewContext(rt);
        let compiled = JS_Eval(
            ctx,
            c"42".as_ptr(),
            2,
            c"nop.js".as_ptr(),
            JS_EVAL_TYPE_GLOBAL | JS_EVAL_FLAG_COMPILE_ONLY,
        );
        assert_eq!(JS_VALUE_GET_TAG(compiled), JS_TAG_FUNCTION_BYTECODE);
        let bytecode = JS_VALUE_GET_PTR(compiled).cast::<JSFunctionBytecode>();
        let original = ((*bytecode).byte_code_buf, (*bytecode).byte_code_len);
        let mut padded = vec![crate::quickjs_opcode::OP_nop as u8; 3];
        padded.extend_from_slice(std::slice::from_raw_parts(original.0, original.1 as usize));
        (*bytecode).byte_code_buf = padded.as_mut_ptr();
        (*bytecode).byte_code_len = padded.len() as i32;
        // Retain the compiled owner until its borrowed bytecode is restored.
        let result = JS_EvalFunction(ctx, JS_DupValue(ctx, compiled));
        (*bytecode).byte_code_buf = original.0;
        (*bytecode).byte_code_len = original.1;
        assert_eq!(JS_VALUE_GET_TAG(result), JS_TAG_INT);
        assert_eq!(JS_VALUE_GET_INT(result), 42);
        JS_FreeValue(ctx, result);
        JS_FreeValue(ctx, compiled);
        JS_FreeContext(ctx);
        JS_FreeRuntime(rt);
    }
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
