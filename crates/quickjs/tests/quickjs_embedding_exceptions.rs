//! Opt-in diagnostics use public APIs; original C JS-visible bytes stay equal.
use quickjs::{quickjs::*, quickjs_header::*};
use std::ffi::CStr;
unsafe fn fixture(test: impl FnOnce(*mut JSContext)) {
    let rt = JS_NewRuntime();
    assert!(!rt.is_null());
    let ctx = JS_NewContext(rt);
    assert!(!ctx.is_null());
    test(ctx);
    JS_FreeContext(ctx);
    JS_FreeRuntime(rt);
}
unsafe fn eval(ctx: *mut JSContext, source: &str) -> JSValue {
    let mut bytes = source.as_bytes().to_vec();
    bytes.push(0);
    JS_Eval(
        ctx,
        bytes.as_ptr().cast(),
        source.len(),
        c"parity.js".as_ptr(),
        JS_EVAL_TYPE_GLOBAL,
    )
}
unsafe fn text(ctx: *mut JSContext, v: JSValue) -> String {
    let p = JS_ToCString(ctx, v);
    assert!(!p.is_null());
    let text = CStr::from_ptr(p).to_string_lossy().into_owned();
    JS_FreeCString(ctx, p);
    text
}
#[test]
fn default_and_enabled_diagnostics_match_unchanged_c_js_visible_output() {
    let sources=[
 "var e=new Error('kept');JSON.stringify([e.stack,Object.getOwnPropertyDescriptor(e,'stack'),Object.keys(e)])",
 "try{try{throw 'first'}finally{try{throw 'cleanup'}catch(e){}}}catch(e){JSON.stringify(e)}",
 "var old=new Error('old');try{throw old}catch(e){JSON.stringify([e===old,e.stack])}",
 "var events=[];try{for(const x of {[Symbol.iterator](){return {next(){return {value:1,done:false}},return(){events.push('close');throw 'ignored'}}}}){throw 'original'}}catch(e){JSON.stringify([e,events])}",
 "var result=[];try{throw 'a'}catch(e){result.push(e)}try{throw 'b'}catch(e){result.push(e)}JSON.stringify(result)"
 ];
    for enabled in [false, true] {
        unsafe {
            fixture(|ctx| {
                JS_EnableExceptionMetadata(JS_GetRuntime(ctx), enabled);
                let mut output = String::new();
                for source in sources {
                    let v = eval(ctx, source);
                    assert_eq!(JS_IsException(v), 0);
                    output.push_str(&text(ctx, v));
                    output.push('\n');
                    JS_FreeValue(ctx, v);
                }
                assert_eq!(
                    output,
                    include_str!("quickjs_embedding_exceptions_c_expected.txt")
                );
            });
        }
    }
}
#[test]
fn public_diagnostics_are_opt_in_and_exception_transfer_clears_them() {
    unsafe {
        fixture(|ctx| {
            let v = eval(ctx, "\nthrow 'primitive';");
            assert_ne!(JS_IsException(v), 0);
            assert!(JS_GetExceptionMetadata(ctx).is_none());
            JS_FreeValue(ctx, JS_GetException(ctx));
            JS_EnableExceptionMetadata(JS_GetRuntime(ctx), true);
            let v = eval(ctx, "\nthrow 'primitive';");
            assert_ne!(JS_IsException(v), 0);
            let p = JS_GetExceptionMetadata(ctx).unwrap();
            assert_eq!(
                (
                    p.filename.as_str(),
                    p.line,
                    p.column,
                    p.source_line.as_str()
                ),
                ("parity.js", 2, 0, "throw 'primitive';")
            );
            let thrown = JS_GetException(ctx);
            assert_eq!(text(ctx, thrown), "primitive");
            assert!(JS_GetExceptionMetadata(ctx).is_none());
            JS_FreeValue(ctx, thrown);
            JS_EnableExceptionMetadata(JS_GetRuntime(ctx), false);
            let v = eval(ctx, "throw 'disabled';");
            assert_ne!(JS_IsException(v), 0);
            assert!(JS_GetExceptionMetadata(ctx).is_none());
            JS_FreeValue(ctx, JS_GetException(ctx));
        });
    }
}
#[test]
fn stack_consumption_policy_is_an_explicit_host_choice() {
    unsafe {
        fixture(|ctx| {
            JS_EnableExceptionMetadata(JS_GetRuntime(ctx), true);
            for consume in [false, true] {
                JS_SetExceptionMetadataStackReadPolicy(JS_GetRuntime(ctx), consume);
                let value = eval(ctx, "var error=new Error('creation');error");
                assert_eq!(JS_IsException(value), 0);
                let before = JS_GetErrorCreationMetadata(ctx, value).unwrap();
                assert_eq!(before.filename, "parity.js");
                let stack = JS_GetPropertyStr(ctx, value, c"stack".as_ptr());
                assert_eq!(JS_IsException(stack), 0);
                assert!(text(ctx, stack).contains("parity.js:1:"));
                JS_FreeValue(ctx, stack);
                assert_eq!(JS_GetErrorCreationMetadata(ctx, value).is_none(), consume);
                JS_FreeValue(ctx, value);
            }
        });
    }
}
#[test]
fn retained_function_uses_its_original_source_even_when_filename_is_reused() {
    unsafe {
        fixture(|ctx| {
            JS_EnableExceptionMetadata(JS_GetRuntime(ctx), true);
            let function = eval(ctx, "(function first(){\n throw 'original function';\n})");
            assert_eq!(JS_IsException(function), 0);
            let other = eval(ctx, "'different source same URL'");
            JS_FreeValue(ctx, other);
            let v = JS_Call(ctx, function, JS_UNDEFINED, 0, std::ptr::null_mut());
            assert_ne!(JS_IsException(v), 0);
            let p = JS_GetExceptionMetadata(ctx).unwrap();
            assert_eq!(
                (p.line, p.column, p.source_line.as_str()),
                (2, 1, " throw 'original function';")
            );
            JS_FreeValue(ctx, JS_GetException(ctx));
            JS_FreeValue(ctx, function);
        });
    }
}
