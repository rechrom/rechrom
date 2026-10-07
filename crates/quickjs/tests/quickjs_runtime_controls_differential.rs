unsafe fn test_interrupt(_rt: *mut JSRuntime, _opaque: *mut c_void) -> i32 {
    0
}
unsafe fn test_sab_alloc(_opaque: *mut c_void, _n: usize) -> *mut c_void {
    ptr::null_mut()
}
unsafe fn test_sab_free(_opaque: *mut c_void, _p: *mut c_void) {}
unsafe fn test_sab_dup(_opaque: *mut c_void, _p: *mut c_void) {}
#[test]
fn official_c_runtime_controls_and_injected_callbacks_match() {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let upstream = root.join("../../vendor/quickjs-2026-06-04");
    let c = std::fs::read_to_string(upstream.join("quickjs.c")).unwrap();
    let mut oracle = source::source_prefix(&c);
    source::append_functions(
        &mut oracle,
        &c,
        &[
            "JS_GetRuntimeOpaque",
            "JS_SetRuntimeOpaque",
            "JS_GetContextOpaque",
            "JS_SetContextOpaque",
            "JS_GetRuntime",
            "JS_SetRuntimeInfo",
            "JS_SetMemoryLimit",
            "JS_SetGCThreshold",
            "JS_SetInterruptHandler",
            "JS_SetCanBlock",
            "JS_SetSharedArrayBufferFunctions",
            "JS_SetStripInfo",
            "JS_GetStripInfo",
            "update_stack_limit",
            "JS_SetMaxStackSize",
            "is_strict_mode",
        ],
    );
    oracle.push_str(
        "static void num(uint64_t n,int size){for(int i=0;i<size;i++)putchar((n>>(8*i))&255);}\n",
    );
    oracle.push_str(include_str!("quickjs_runtime_controls_oracle.c"));
    let directory =
        std::env::temp_dir().join(format!("quickjs-controls-oracle-{}", std::process::id()));
    std::fs::create_dir_all(&directory).unwrap();
    let path = directory.join("controls.c");
    let executable = directory.join("controls");
    std::fs::write(&path, oracle).unwrap();
    let result = include!("quickjs_oracle_config.rs")
        .args(["-std=c11", "-O2", "-I"])
        .arg(upstream)
        .arg(&path)
        .arg("-o")
        .arg(&executable)
        .output()
        .unwrap();
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    let c = std::process::Command::new(&executable).output().unwrap();
    let _ = std::fs::remove_dir_all(directory);
    assert!(c.status.success());
    let mut out = Vec::new();
    unsafe {
        let mut rt: JSRuntime = core::mem::zeroed();
        let rt = ptr::from_mut(&mut rt);
        let mut ctx: JSContext = core::mem::zeroed();
        ctx.rt = rt;
        let mut frame: JSStackFrame = core::mem::zeroed();
        JS_SetRuntimeInfo(ptr::null_mut(), b"info\0".as_ptr().cast());
        let mut rng = 0x123456789abcdef0u64;
        for step in 0..16384 {
            rng ^= rng << 13;
            rng ^= rng >> 7;
            rng ^= rng << 17;
            JS_SetRuntimeOpaque(rt, rng as usize as *mut c_void);
            JS_SetContextOpaque(&mut ctx, (!rng) as usize as *mut c_void);
            JS_SetMemoryLimit(rt, rng as usize);
            JS_SetGCThreshold(rt, (!rng) as usize);
            JS_SetRuntimeInfo(
                rt,
                if step & 1 != 0 {
                    b"info\0".as_ptr().cast()
                } else {
                    ptr::null()
                },
            );
            JS_SetInterruptHandler(
                rt,
                if step & 1 != 0 {
                    Some(test_interrupt)
                } else {
                    None
                },
                (rng >> 1) as usize as *mut c_void,
            );
            JS_SetCanBlock(rt, rng as i32);
            JS_SetStripInfo(rt, rng as i32);
            let sf = JSSharedArrayBufferFunctions {
                sab_alloc: if step & 1 != 0 {
                    Some(test_sab_alloc)
                } else {
                    None
                },
                sab_free: if step & 2 != 0 {
                    Some(test_sab_free)
                } else {
                    None
                },
                sab_dup: if step & 4 != 0 {
                    Some(test_sab_dup)
                } else {
                    None
                },
                sab_opaque: (rng >> 2) as usize as *mut c_void,
            };
            JS_SetSharedArrayBufferFunctions(rt, &sf);
            (*rt).stack_top = rng as usize;
            JS_SetMaxStackSize(rt, if step % 8 != 0 { (!rng) as usize } else { 0 });
            (*rt).current_stack_frame = if step & 1 != 0 {
                &mut frame
            } else {
                ptr::null_mut()
            };
            frame.js_mode = rng as i32;
            num(&mut out, JS_GetRuntimeOpaque(rt) as usize as u64, 8);
            num(&mut out, JS_GetContextOpaque(&mut ctx) as usize as u64, 8);
            num(&mut out, (JS_GetRuntime(&mut ctx) == rt) as u64, 4);
            num(
                &mut out,
                (*rt).malloc_ctx.malloc_state.malloc_limit as u64,
                8,
            );
            num(&mut out, (*rt).malloc_gc_threshold as u64, 8);
            num(&mut out, !(*rt).rt_info.is_null() as u64, 4);
            num(&mut out, (*rt).interrupt_handler.is_some() as u64, 4);
            num(&mut out, (*rt).interrupt_opaque as usize as u64, 8);
            num(&mut out, (*rt).can_block as u64, 4);
            num(&mut out, JS_GetStripInfo(rt) as u64, 4);
            num(&mut out, (*rt).sab_funcs.sab_alloc.is_some() as u64, 4);
            num(&mut out, (*rt).sab_funcs.sab_free.is_some() as u64, 4);
            num(&mut out, (*rt).sab_funcs.sab_dup.is_some() as u64, 4);
            num(&mut out, (*rt).sab_funcs.sab_opaque as usize as u64, 8);
            num(&mut out, (*rt).stack_size as u64, 8);
            num(&mut out, (*rt).stack_limit as u64, 8);
            num(&mut out, is_strict_mode(&mut ctx) as u64, 4);
        }
    }
    assert_eq!(out.len(), c.stdout.len());
    if let Some(i) = out.iter().zip(&c.stdout).position(|(r, c)| r != c) {
        panic!(
            "runtime controls C/Rust mismatch at byte {i}: Rust={}, C={}",
            out[i], c.stdout[i]
        );
    }
    eprintln!(
        "quickjs.c runtime controls parity: 16384 configurations, {} bytes",
        out.len()
    );
}
