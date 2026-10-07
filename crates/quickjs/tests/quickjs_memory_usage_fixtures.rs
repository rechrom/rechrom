// Original memory counters after real programs and cycle collection.
pub unsafe fn memory_usage_fixture() -> Vec<u8> {
    let rt = JS_NewRuntime(); assert!(!rt.is_null());
    let ctx = JS_NewContext(rt); assert!(!ctx.is_null());
    let mut bytes = Vec::new();
    for (id, source) in include_str!("quickjs_combined_eval_cases.txt").lines().enumerate() {
        let text = std::ffi::CString::new(source).unwrap();
        let value = JS_Eval(ctx, text.as_ptr(), source.len(), c"memory.js".as_ptr(), 0);
        assert_eq!(JS_IsException(value), 0, "fixture case {id} threw");
        for phase in 0..2 {
            if phase == 1 { JS_RunGC(rt); }
            let mut usage: JSMemoryUsage = core::mem::zeroed();
            JS_ComputeMemoryUsage(rt, &mut usage);
            bytes.extend_from_slice(&(id as u32).to_le_bytes());
            bytes.extend_from_slice(&(phase as u32).to_le_bytes());
            bytes.extend_from_slice(&usage.memory_used_size.to_le_bytes());
            bytes.extend_from_slice(&usage.memory_used_count.to_le_bytes());
            bytes.extend_from_slice(&usage.atom_count.to_le_bytes());
            bytes.extend_from_slice(&usage.atom_size.to_le_bytes());
            bytes.extend_from_slice(&usage.str_count.to_le_bytes());
            bytes.extend_from_slice(&usage.str_size.to_le_bytes());
            bytes.extend_from_slice(&usage.obj_count.to_le_bytes());
            bytes.extend_from_slice(&usage.obj_size.to_le_bytes());
            bytes.extend_from_slice(&usage.prop_count.to_le_bytes());
            bytes.extend_from_slice(&usage.prop_size.to_le_bytes());
            bytes.extend_from_slice(&usage.shape_count.to_le_bytes());
            bytes.extend_from_slice(&usage.shape_size.to_le_bytes());
            bytes.extend_from_slice(&usage.js_func_count.to_le_bytes());
            bytes.extend_from_slice(&usage.js_func_size.to_le_bytes());
            bytes.extend_from_slice(&usage.js_func_code_size.to_le_bytes());
            bytes.extend_from_slice(&usage.js_func_pc2line_count.to_le_bytes());
            bytes.extend_from_slice(&usage.js_func_pc2line_size.to_le_bytes());
            bytes.extend_from_slice(&usage.c_func_count.to_le_bytes());
            bytes.extend_from_slice(&usage.array_count.to_le_bytes());
            bytes.extend_from_slice(&usage.fast_array_count.to_le_bytes());
            bytes.extend_from_slice(&usage.fast_array_elements.to_le_bytes());
            bytes.extend_from_slice(&usage.binary_object_count.to_le_bytes());
            bytes.extend_from_slice(&usage.binary_object_size.to_le_bytes());
        }
        JS_FreeValue(ctx, value);
    }
    JS_FreeContext(ctx); JS_FreeRuntime(rt);
    bytes
}
