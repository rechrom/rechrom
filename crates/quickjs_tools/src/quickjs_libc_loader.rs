// Official quickjs-libc.c:486..546,684..735. Host module loading.
#[cfg(unix)]
unsafe fn js_module_loader_so(ctx: *mut JSContext, name: *const c_char) -> *mut JSModuleDef {
    let bytes = CStr::from_ptr(name).to_bytes();
    let filename = if bytes.contains(&b'/') { name.cast_mut() } else {
        let allocated = js_malloc(ctx, bytes.len() + 3).cast::<c_char>();
        if allocated.is_null() { return ptr::null_mut(); }
        ptr::copy_nonoverlapping(c"./".as_ptr(), allocated, 2);
        ptr::copy_nonoverlapping(name, allocated.add(2), bytes.len() + 1);
        allocated
    };
    let handle = libc::dlopen(filename, libc::RTLD_NOW | libc::RTLD_LOCAL);
    if filename != name.cast_mut() { js_free(ctx, filename.cast()); }
    let description = CStr::from_ptr(name).to_string_lossy();
    if handle.is_null() {
        JS_ThrowReferenceError(ctx, format_args!("could not load module filename '{}' as shared library", description));
        return ptr::null_mut();
    }
    let symbol = libc::dlsym(handle, c"js_init_module".as_ptr());
    if symbol.is_null() {
        JS_ThrowReferenceError(ctx, format_args!("could not load module filename '{}': js_init_module not found", description));
        libc::dlclose(handle);
        return ptr::null_mut();
    }
    let init: unsafe extern "C" fn(*mut JSContext, *const c_char) -> *mut JSModuleDef = std::mem::transmute(symbol);
    let module = init(ctx, name);
    if module.is_null() {
        JS_ThrowReferenceError(ctx, format_args!("could not load module filename '{}': initialization error", description));
        libc::dlclose(handle);
    }
    // Original keeps a successful library loaded for the module's lifetime.
    module
}
#[cfg(windows)]
unsafe fn js_module_loader_so(ctx: *mut JSContext, _name: *const c_char) -> *mut JSModuleDef {
    JS_ThrowReferenceError(ctx, c"shared library modules are not supported yet".as_ptr());
    ptr::null_mut()
}

pub unsafe fn js_module_loader(ctx: *mut JSContext, name: *const c_char,
                              _opaque: *mut c_void, attributes: JSValueConst) -> *mut JSModuleDef {
    let bytes = CStr::from_ptr(name).to_bytes();
    if bytes.ends_with(b".so") { return js_module_loader_so(ctx, name); }
    let mut length = 0;
    let buffer = js_load_file(ctx, &mut length, name);
    if buffer.is_null() {
        JS_ThrowReferenceError(ctx, format_args!("could not load module filename '{}'", CStr::from_ptr(name).to_string_lossy()));
        return ptr::null_mut();
    }
    let json_kind = js_module_test_json(ctx, attributes);
    if bytes.ends_with(b".json") || json_kind > 0 {
        let value = JS_ParseJSON2(ctx, buffer.cast(), length, name,
            if json_kind == 2 { JS_PARSE_JSON_EXT } else { 0 });
        js_free(ctx, buffer.cast());
        if JS_IsException(value) != 0 { return ptr::null_mut(); }
        create_json_module(ctx, name, value)
    } else {
        let value = JS_Eval(ctx, buffer.cast(), length, name, JS_EVAL_TYPE_MODULE | JS_EVAL_FLAG_COMPILE_ONLY);
        js_free(ctx, buffer.cast());
        if JS_IsException(value) != 0 { return ptr::null_mut(); }
        // Upstream deliberately does not propagate import-meta failure here.
        js_module_set_import_meta(ctx, value, 1, 0);
        let module = JS_VALUE_GET_PTR(value).cast();
        JS_FreeValue(ctx, value);
        module
    }
}
