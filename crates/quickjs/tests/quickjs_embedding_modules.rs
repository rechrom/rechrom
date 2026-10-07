//! Real public API gates for optional embedding behavior. Build against the
//! production crate with rustc --test; no internal engine fields or boundaries.
use quickjs::{quickjs::*, quickjs_header::*};
use std::{
    collections::HashMap,
    ffi::{c_char, c_void, CStr},
    ptr,
};

#[derive(Default)]
struct State {
    loads: Vec<String>,
    unhandled: HashMap<usize, JSValue>,
    internal: usize,
}
unsafe fn text(ctx: *mut JSContext, value: JSValue) -> String {
    let p = JS_ToCString(ctx, value);
    assert!(!p.is_null());
    let s = CStr::from_ptr(p).to_string_lossy().into_owned();
    JS_FreeCString(ctx, p);
    s
}
unsafe fn eval(ctx: *mut JSContext, code: &str, flags: i32) -> JSValue {
    let mut source = code.as_bytes().to_vec();
    source.push(0);
    let value = JS_Eval(
        ctx,
        source.as_ptr().cast(),
        code.len(),
        c"root.js".as_ptr(),
        flags,
    );
    if JS_IsException(value) != 0 {
        let error = JS_GetException(ctx);
        let message = text(ctx, error);
        JS_FreeValue(ctx, error);
        panic!("{message}");
    }
    value
}
// Loader name must stay NUL terminated: public callback already provides it.
unsafe fn loader_cstring(
    ctx: *mut JSContext,
    name: *const c_char,
    opaque: *mut c_void,
) -> *mut JSModuleDef {
    let state = &mut *opaque.cast::<State>();
    let spelling = CStr::from_ptr(name).to_str().unwrap();
    state.loads.push(spelling.into());
    let source = match spelling {
        "a" => "import 'leaf';order.push('a')",
        "b" => "order.push('b')",
        "leaf" => "order.push('leaf')",
        _ => {
            JS_ThrowReferenceError(ctx, c"missing".as_ptr());
            return ptr::null_mut();
        }
    };
    let mut code = source.as_bytes().to_vec();
    code.push(0);
    let value = JS_Eval(
        ctx,
        code.as_ptr().cast(),
        source.len(),
        name,
        JS_EVAL_TYPE_MODULE | JS_EVAL_FLAG_COMPILE_ONLY,
    );
    if JS_IsException(value) != 0 {
        return ptr::null_mut();
    }
    let module = JS_VALUE_GET_PTR(value).cast();
    JS_FreeValue(ctx, value);
    module
}
unsafe fn tracker(
    ctx: *mut JSContext,
    promise: JSValue,
    _reason: JSValue,
    handled: i32,
    opaque: *mut c_void,
) {
    let state = &mut *opaque.cast::<State>();
    let key = JS_VALUE_GET_PTR(promise) as usize;
    if handled != 0 {
        if let Some(value) = state.unhandled.remove(&key) {
            JS_FreeValue(ctx, value);
        }
    } else {
        assert!(state
            .unhandled
            .insert(key, JS_DupValue(ctx, promise))
            .is_none());
    }
}
unsafe fn internal(
    ctx: *mut JSContext,
    _module: *mut JSModuleDef,
    promise: JSValue,
    opaque: *mut c_void,
) {
    (*opaque.cast::<State>()).internal += 1;
    assert_eq!(JS_MarkPromiseHandled(ctx, promise), 0);
}
unsafe fn deny(
    ctx: *mut JSContext,
    _resolve: JSValue,
    reject: JSValue,
    _base: JSValue,
    _specifier: JSValue,
    _attributes: JSValue,
    _opaque: *mut c_void,
) -> i32 {
    let mut reason = JS_NewString(ctx, c"denied".as_ptr());
    let result = JS_Call(ctx, reject, JS_UNDEFINED, 1, &mut reason);
    let failed = JS_IsException(result);
    JS_FreeValue(ctx, result);
    JS_FreeValue(ctx, reason);
    if failed != 0 {
        -1
    } else {
        0
    }
}
unsafe fn drain(ctx: *mut JSContext) {
    let rt = JS_GetRuntime(ctx);
    loop {
        let mut jobctx = ptr::null_mut();
        let status = JS_ExecutePendingJob(rt, &mut jobctx);
        assert!(status >= 0);
        if status == 0 {
            break;
        }
    }
}
unsafe fn fixture(test: impl FnOnce(*mut JSContext, *mut State)) {
    let rt = JS_NewRuntime();
    assert!(!rt.is_null());
    let ctx = JS_NewContext(rt);
    assert!(!ctx.is_null());
    let mut state = State::default();
    let pointer = &mut state as *mut State;
    JS_SetModuleLoaderFunc(rt, None, Some(loader_cstring), pointer.cast());
    JS_SetHostPromiseRejectionTracker(rt, Some(tracker), pointer.cast());
    test(ctx, pointer);
    JS_SetHostPromiseRejectionTracker(rt, None, ptr::null_mut());
    for value in state.unhandled.into_values() {
        JS_FreeValue(ctx, value);
    }
    JS_FreeContext(ctx);
    JS_FreeRuntime(rt);
}
#[test]
fn default_compile_resolves_with_official_depth_first_order() {
    unsafe {
        fixture(|ctx, state| {
            JS_FreeValue(ctx, eval(ctx, "var order=[]", JS_EVAL_TYPE_GLOBAL));
            let module = eval(
                ctx,
                "import 'a';import 'b';order.push('root')",
                JS_EVAL_TYPE_MODULE | JS_EVAL_FLAG_COMPILE_ONLY,
            );
            assert_eq!((*state).loads, ["a", "leaf", "b"]);
            let value = JS_EvalFunction(ctx, module);
            assert_eq!(JS_IsException(value), 0);
            JS_FreeValue(ctx, value);
            let order = eval(ctx, "order.join('|')", JS_EVAL_TYPE_GLOBAL);
            assert_eq!(text(ctx, order), "leaf|a|b|root");
            JS_FreeValue(ctx, order);
        });
    }
}
#[test]
fn optional_compile_defers_resolution_and_public_request_atoms_are_owned() {
    unsafe {
        fixture(|ctx, state| {
            let module = eval(
                ctx,
                "import 'a'; import 'b';",
                JS_EVAL_TYPE_MODULE | JS_EVAL_FLAG_COMPILE_ONLY | JS_EVAL_FLAG_HOST_NO_RESOLVE,
            );
            assert!((*state).loads.is_empty());
            let definition = JS_VALUE_GET_PTR(module).cast::<JSModuleDef>();
            assert_eq!(JS_GetModuleRequestCount(ctx, definition), 2);
            for (index, name) in ["a", "b"].iter().enumerate() {
                let atom = JS_GetModuleRequestName(ctx, definition, index as i32);
                let string = JS_AtomToString(ctx, atom);
                JS_FreeAtom(ctx, atom);
                assert_eq!(text(ctx, string), *name);
                JS_FreeValue(ctx, string);
            }
            assert_eq!(JS_GetModuleRequestName(ctx, definition, 2), 0);
            let error = JS_GetException(ctx);
            assert!(text(ctx, error).starts_with("RangeError:"));
            JS_FreeValue(ctx, error);
            JS_FreeValue(ctx, module);
        });
    }
}
#[test]
fn hooks_are_optional_and_dynamic_coercion_still_returns_rejected_promises() {
    unsafe {
        fixture(|ctx, state| {
            let rt = JS_GetRuntime(ctx);
            JS_SetModuleEmbeddingHooks(rt, Some(deny), None, state.cast());
            JS_FreeValue(ctx,eval(ctx,"var order=[];import({toString(){order.push('coerce');return 'missing'}}).catch(e=>order.push(e));Promise.resolve().then(()=>order.push('job'));let p;try{p=import({toString(){throw 'coercion'}});order.push('promise')}catch(e){throw 'synchronous'};p.catch(e=>order.push(e));import('missing',{with:{type:42}}).catch(e=>order.push(e.name));",JS_EVAL_TYPE_GLOBAL));
            drain(ctx);
            let value = eval(ctx, "order.join('|')", JS_EVAL_TYPE_GLOBAL);
            assert_eq!(
                text(ctx, value),
                "coerce|promise|denied|job|coercion|TypeError"
            );
            JS_FreeValue(ctx, value);
            assert!((*state).loads.is_empty());
            assert!((*state).unhandled.is_empty());
            JS_SetModuleEmbeddingHooks(rt, None, None, ptr::null_mut());
            JS_FreeValue(
                ctx,
                eval(
                    ctx,
                    "import('missing').catch(e=>order.push(e.message))",
                    JS_EVAL_TYPE_GLOBAL,
                ),
            );
            drain(ctx);
            assert_eq!((*state).loads, ["missing"]);
        });
    }
}
#[test]
fn consuming_only_internal_module_promise_keeps_two_user_rejections() {
    unsafe {
        fixture(|ctx, state| {
            JS_SetModuleEmbeddingHooks(JS_GetRuntime(ctx), None, Some(internal), state.cast());
            JS_FreeValue(
                ctx,
                eval(
                    ctx,
                    "Promise.reject('same');Promise.reject('same')",
                    JS_EVAL_TYPE_GLOBAL,
                ),
            );
            assert_eq!((*state).unhandled.len(), 2);
            let value = eval(ctx, "throw Error('module')", JS_EVAL_TYPE_MODULE);
            JS_FreeValue(ctx, value);
            drain(ctx);
            assert_eq!((*state).internal, 1);
            assert_eq!((*state).unhandled.len(), 3);
        });
    }
}

#[test]
fn default_module_internal_promise_is_not_silently_consumed() {
    unsafe {
        fixture(|ctx, state| {
            JS_FreeValue(
                ctx,
                eval(
                    ctx,
                    "Promise.reject('same');Promise.reject('same')",
                    JS_EVAL_TYPE_GLOBAL,
                ),
            );
            JS_FreeValue(ctx, eval(ctx, "throw Error('module')", JS_EVAL_TYPE_MODULE));
            drain(ctx);
            assert_eq!((*state).internal, 0);
            assert_eq!((*state).unhandled.len(), 4);
        });
    }
}

#[test]
fn typed_array_prototype_keys_and_descriptor_match_official_c() {
    unsafe {
        fixture(|ctx, _state| {
            let expected = include_str!("quickjs_embedding_modules_c_expected.txt")
                .lines()
                .find_map(|line| line.strip_prefix("typed-proto="))
                .unwrap();
            let value = eval(ctx,
                "JSON.stringify([Reflect.ownKeys(Object.getPrototypeOf(Uint8Array.prototype)).map(String),Object.getOwnPropertyDescriptor(Object.getPrototypeOf(Uint8Array.prototype),'toString')])",
                JS_EVAL_TYPE_GLOBAL);
            assert_eq!(text(ctx, value), expected);
            JS_FreeValue(ctx, value);
        });
    }
}
