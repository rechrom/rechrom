// Executes the real translated engine with the complete standard context.
// The unchanged official C oracle uses the same JS_NewContext bootstrap.
pub unsafe fn combined_eval_fixture() -> Vec<u8> {
    let rt = JS_NewRuntime();
    assert!(!rt.is_null());
    // Equal injected budget: Debug Rust frames and optimized C frames differ.
    JS_SetMaxStackSize(rt, 4 * 1024 * 1024);
    let ctx = JS_NewContext(rt);
    assert!(!ctx.is_null());
    let mut bytes = Vec::new();
    for (id, source) in include_str!("quickjs_combined_eval_cases.txt").lines().enumerate() {
        let text = std::ffi::CString::new(source).unwrap();
        let result = JS_Eval(ctx, text.as_ptr(), source.len(), c"combined.js".as_ptr(), 0);
        bytes.extend_from_slice(&(id as u32).to_le_bytes());
        bytes.extend_from_slice(&(JS_VALUE_GET_NORM_TAG(result) as u32).to_le_bytes());
        let value = if JS_IsException(result) != 0 { JS_GetException(ctx) } else { result };
        let mut len = 0usize;
        let encoded = JS_ToCStringLen2(ctx, &mut len, value, 0);
        assert!(!encoded.is_null(), "conversion failed in case {id}");
        bytes.extend_from_slice(&(len as u32).to_le_bytes());
        bytes.extend_from_slice(core::slice::from_raw_parts(encoded.cast::<u8>(), len));
        JS_FreeCString(ctx, encoded);
        JS_FreeValue(ctx, value);
    }
    JS_FreeContext(ctx);
    JS_FreeRuntime(rt);
    bytes
}
