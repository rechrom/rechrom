// Native Rust host output for qjsc executable mode. The original parser,
// module compiler, bytecode writer, feature selection and C output stay intact.
fn rust_source(state: &CompilerState) -> Vec<u8> {
    let mut src=b"#![allow(non_snake_case,non_upper_case_globals,unused_imports)]\nuse quickjs::{quickjs::*,quickjs_header::*};\nuse quickjs_tools::quickjs_libc::*;\nuse std::{ffi::CString,ptr};\n".to_vec();
    // Embed the native argv adapter so emitted programs remain self-contained
    // and preserve C's byte arguments on both Unix and Windows.
    src.extend_from_slice(include_str!("host_args.rs").as_bytes());
    for obj in &state.objects {
        src.extend_from_slice(b"static ");
        src.extend_from_slice(&obj.name);
        writeln!(src, ": &[u8] = &{:?};", obj.bytes).unwrap();
        if let Some(name) = &obj.module_name {
            src.extend_from_slice(b"static ");
            src.extend_from_slice(&obj.name);
            writeln!(src, "_module_name: &[u8] = &{:?};", name).unwrap();
        }
    }
    src.extend_from_slice(b"unsafe fn JS_NewCustomContext(rt:*mut JSRuntime)->*mut JSContext {let ctx=JS_NewContextRaw(rt);if ctx.is_null(){return ctx;}JS_AddIntrinsicBaseObjects(ctx);\n");
    for (i, (_, name)) in FEATURES.iter().enumerate() {
        if state.options.feature_bitmap & (1 << i) != 0 {
            if let Some(name) = name {
                writeln!(src, "JS_AddIntrinsic{name}(ctx);").unwrap();
            }
        }
    }
    for e in &state.init_module_list.array {
        let short = e.short_name.as_deref().unwrap_or(b"");
        let symbol = joined(&[b"js_init_module_", short]);
        if short != b"std" && short != b"os" {
            src.extend_from_slice(&joined(&[
                b"extern \"C\" { fn ",
                &symbol,
                b"(ctx:*mut JSContext,name:*const std::ffi::c_char)->*mut JSModuleDef; }\n",
            ]));
        }
        // Numeric byte literals retain invalid UTF-8 module names without escaping loss.
        let mut name = e.name.clone();
        name.push(0);
        src.extend_from_slice(&symbol);
        writeln!(
            src,
            "(ctx,{{static NAME:&[u8]=&{name:?};NAME.as_ptr().cast()}});"
        )
        .unwrap();
    }
    for obj in &state.objects {
        if obj.kind == CNAME_TYPE_MODULE {
            src.extend_from_slice(&joined(&[
                b"js_std_eval_binary(ctx,",
                &obj.name,
                b".as_ptr(),",
                &obj.name,
                b".len(),1);\n",
            ]));
        } else if obj.kind == CNAME_TYPE_JSON_MODULE {
            src.extend_from_slice(&joined(&[
                b"js_std_eval_binary_json_module(ctx,",
                &obj.name,
                b".as_ptr(),",
                &obj.name,
                b".len(),",
                &obj.name,
                b"_module_name.as_ptr().cast());\n",
            ]));
        }
    }
    src.extend_from_slice(b"ctx}\nfn main(){unsafe{let rt=JS_NewRuntime();js_std_set_worker_new_context_func(Some(JS_NewCustomContext));js_std_init_handlers(rt);\n");
    if state.options.stack_size != 0 {
        writeln!(
            src,
            "JS_SetMaxStackSize(rt,{});",
            state.options.stack_size as u32
        )
        .unwrap();
    }
    if state.options.feature_bitmap & (1 << 9) != 0 {
        src.extend_from_slice(b"JS_SetModuleLoaderFunc2(rt,None,Some(js_module_loader),Some(js_module_check_attributes),ptr::null_mut());\n");
    }
    src.extend_from_slice(b"let ctx=JS_NewCustomContext(rt);let args=args_bytes().expect(\"native command-line arguments unavailable\").into_iter().map(|s|CString::new(s).expect(\"OS arguments contain no NUL\")).collect::<Vec<_>>();let mut argv=args.iter().map(|s|s.as_ptr().cast_mut()).collect::<Vec<_>>();js_std_add_helpers(ctx,argv.len()as i32,argv.as_mut_ptr());\n");
    for obj in &state.objects {
        if obj.kind == CNAME_TYPE_SCRIPT {
            src.extend_from_slice(&joined(&[
                b"js_std_eval_binary(ctx,",
                &obj.name,
                b".as_ptr(),",
                &obj.name,
                b".len(),0);\n",
            ]));
        }
    }
    src.extend_from_slice(
        b"js_std_loop(ctx);js_std_free_handlers(rt);JS_FreeContext(ctx);JS_FreeRuntime(rt);}}\n",
    );
    src
}
fn rust_dependency_dir(use_lto: bool) -> Result<std::path::PathBuf, ToolError> {
    if use_lto {
        if let Some(path) = std::env::var_os("QJSC_RUST_LTO_DEPS") {
            return Ok(path.into());
        }
    }
    if let Some(path) = std::env::var_os("QJSC_RUST_DEPS") {
        return Ok(path.into());
    }
    let exe = std::env::current_exe()?;
    let parent = exe
        .parent()
        .ok_or_else(|| ToolError::new("qjsc: executable has no parent directory\n"))?;
    let dir = if parent.file_name().is_some_and(|s| s == "deps") {
        parent.to_path_buf()
    } else {
        parent.join("deps")
    };
    if dir.is_dir() {
        Ok(dir)
    } else {
        Err(ToolError::new("qjsc: Rust dependencies not found; set QJSC_RUST_DEPS to the built Cargo deps directory\n"))
    }
}
fn newest_rlib(dir: &std::path::Path, name: &str) -> Result<std::path::PathBuf, ToolError> {
    let prefix = format!("lib{name}-");
    let dir_bytes = os_bytes(dir.as_os_str().to_os_string())?;
    fs::read_dir(dir)?
        .filter_map(Result::ok)
        .map(|e| e.path())
        .filter(|p| {
            p.file_name()
                .is_some_and(|s| s.as_encoded_bytes().starts_with(prefix.as_bytes()))
                && p.extension().is_some_and(|s| s == "rlib")
        })
        .max_by_key(|p| p.metadata().and_then(|m| m.modified()).ok())
        .ok_or_else(|| {
            raw_error(
                format!("qjsc: Rust library {name} not found in ").as_bytes(),
                &dir_bytes,
                b"\n",
            )
        })
}
fn output_rust_executable(
    state: &CompilerState,
    out: &[u8],
    use_lto: bool,
    verbose: i32,
    dynamic_export: bool,
) -> Result<i32, ToolError> {
    let deps = rust_dependency_dir(use_lto)?;
    let engine = newest_rlib(&deps, "quickjs")?;
    let tools = newest_rlib(&deps, "quickjs_tools")?;
    let file = std::env::temp_dir().join(format!("qjsc-{}-host.rs", std::process::id()));
    fs::write(&file, rust_source(state))?;
    // rustup directory overrides change with the input working directory.
    // Retain the compiler toolchain that built these Rust metadata files.
    let mut args: Vec<Vec<u8>> = if let Some(path) = std::env::var_os("QJSC_RUSTC") {
        vec![os_bytes(path)?]
    } else if let Some(toolchain) = option_env!("RUSTUP_TOOLCHAIN") {
        vec![
            b"rustup".to_vec(),
            b"run".to_vec(),
            toolchain.as_bytes().to_vec(),
            b"rustc".to_vec(),
        ]
    } else {
        vec![b"rustc".to_vec()]
    };
    let engine = os_bytes(engine.into_os_string())?;
    let tools = os_bytes(tools.into_os_string())?;
    let deps = os_bytes(deps.into_os_string())?;
    let source = os_bytes(file.clone().into_os_string())?;
    args.extend([
        b"--edition=2021".to_vec(),
        b"--crate-name".to_vec(),
        b"qjsc_program".to_vec(),
        b"-C".to_vec(),
        b"opt-level=2".to_vec(),
        b"-C".to_vec(),
        b"debuginfo=0".to_vec(),
        b"--extern".to_vec(),
        joined(&[b"quickjs=", &engine]),
        b"--extern".to_vec(),
        joined(&[b"quickjs_tools=", &tools]),
        b"-L".to_vec(),
        joined(&[b"dependency=", &deps]),
        b"-o".to_vec(),
        out.to_vec(),
        source,
    ]);
    if use_lto {
        args.extend([b"-C".to_vec(), b"lto=thin".to_vec()]);
    }
    if dynamic_export {
        args.extend([b"-C".to_vec(), b"link-arg=-Wl,-export_dynamic".to_vec()]);
    }
    if let Some(extra) = std::env::var_os("QJSC_RUST_LINK_ARGS") {
        let extra = os_bytes(extra)?;
        for arg in extra
            .split(u8::is_ascii_whitespace)
            .filter(|s| !s.is_empty())
        {
            args.extend([b"-C".to_vec(), joined(&[b"link-arg=", arg])]);
        }
    }
    if verbose != 0 {
        print_command(&args)?;
    }
    let result = exec_cmd_bytes(&args).unwrap_or(1);
    let _ = fs::remove_file(file);
    Ok(result)
}
