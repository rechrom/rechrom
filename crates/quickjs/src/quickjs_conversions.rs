// quickjs.c truth-value conversion, lexical helpers and bytecode buffer policy. MIT.
unsafe fn JS_ToBoolFree(ctx: *mut JSContext, val: JSValue) -> i32 {
    let tag = JS_VALUE_GET_TAG(val);
    match tag {
        JS_TAG_INT => (JS_VALUE_GET_INT(val) != 0) as i32,
        JS_TAG_BOOL | JS_TAG_NULL | JS_TAG_UNDEFINED => JS_VALUE_GET_INT(val),
        JS_TAG_EXCEPTION => -1,
        JS_TAG_STRING => {
            let ret = ((*JS_VALUE_GET_PTR(val).cast::<JSString>()).len() != 0) as i32;
            JS_FreeValue(ctx, val);
            ret
        }
        JS_TAG_STRING_ROPE => {
            let ret = ((*JS_VALUE_GET_PTR(val).cast::<JSStringRope>()).len != 0) as i32;
            JS_FreeValue(ctx, val);
            ret
        }
        JS_TAG_SHORT_BIG_INT => (JS_VALUE_GET_SHORT_BIG_INT(val) != 0) as i32,
        JS_TAG_BIG_INT => {
            let p = JS_VALUE_GET_PTR(val).cast::<JSBigInt>();
            let mut ret = 0;
            for i in (0..(*p).len as usize).rev() {
                if *ptr::addr_of!((*p).tab).cast::<js_limb_t>().add(i) != 0 {
                    ret = 1;
                    break;
                }
            }
            JS_FreeValue(ctx, val);
            ret
        }
        JS_TAG_OBJECT => {
            let ret = ((*JS_VALUE_GET_PTR(val).cast::<JSObject>()).is_HTMLDDA() == 0) as i32;
            JS_FreeValue(ctx, val);
            ret
        }
        _ => {
            if JS_TAG_IS_FLOAT64(tag) != 0 {
                let d = JS_VALUE_GET_FLOAT64(val);
                (!d.is_nan() && d != 0.0) as i32
            } else {
                JS_FreeValue(ctx, val);
                1
            }
        }
    }
}
pub unsafe fn JS_ToBool(ctx: *mut JSContext, val: JSValueConst) -> i32 {
    JS_ToBoolFree(ctx, JS_DupValue(ctx, val))
}
unsafe fn skip_spaces(pc: *const c_char) -> i32 {
    let p_start = pc.cast::<u8>();
    let mut p = p_start;
    loop {
        let mut c = *p as u32;
        if c < 128 {
            if !((9..=13).contains(&c) || c == 32) {
                break;
            }
            p = p.add(1);
        } else {
            let mut p_next = ptr::null();
            c = super::cutils::unicode_from_utf8(
                p,
                super::cutils_header::UTF8_CHAR_LEN_MAX as i32,
                &mut p_next,
            ) as u32;
            if super::libunicode_header::lre_is_space(c) == 0 {
                break;
            }
            p = p_next;
        }
    }
    p.offset_from(p_start) as i32
}
fn to_digit(c: i32) -> i32 {
    match c {
        48..=57 => c - 48,
        65..=90 => c - 65 + 10,
        97..=122 => c - 97 + 10,
        _ => 36,
    }
}
fn is_digit(c: i32) -> i32 {
    (48..=57).contains(&c) as i32
}
const MAX_SAFE_INTEGER: i64 = (1i64 << 53) - 1;
fn is_safe_integer(d: f64) -> JS_BOOL {
    (d.is_finite() && d.floor() == d && d.abs() <= MAX_SAFE_INTEGER as f64) as i32
}
unsafe fn js_dbuf_init(ctx: *mut JSContext, s: *mut super::cutils_header::DynBuf) {
    super::cutils::dbuf_init2(s, (*ctx).rt.cast(), Some(js_realloc_dbuf_rt));
}
// C casts JSRuntime* to the DynBuf void* callback ABI; Rust makes that bridge explicit.
unsafe fn js_realloc_dbuf_rt(opaque: *mut c_void, p: *mut c_void, size: usize) -> *mut c_void {
    js_realloc_rt(opaque.cast(), p, size)
}
unsafe fn js_realloc_bytecode_rt(opaque: *mut c_void, p: *mut c_void, size: usize) -> *mut c_void {
    if size > i32::MAX as usize / 2 {
        ptr::null_mut()
    } else {
        js_realloc_rt(opaque.cast(), p, size)
    }
}
unsafe fn js_dbuf_bytecode_init(ctx: *mut JSContext, s: *mut super::cutils_header::DynBuf) {
    super::cutils::dbuf_init2(s, (*ctx).rt.cast(), Some(js_realloc_bytecode_rt));
}
pub unsafe fn JS_GetGlobalObject(ctx: *mut JSContext) -> JSValue {
    JS_DupValue(ctx, (*ctx).global_obj)
}
pub unsafe fn JS_SetModulePrivateValue(
    ctx: *mut JSContext,
    m: *mut JSModuleDef,
    val: JSValue,
) -> i32 {
    set_value(ctx, &mut (*m).private_value, val);
    0
}
pub unsafe fn JS_GetModulePrivateValue(ctx: *mut JSContext, m: *mut JSModuleDef) -> JSValue {
    JS_DupValue(ctx, (*m).private_value)
}
pub unsafe fn JS_GetModuleName(ctx: *mut JSContext, m: *mut JSModuleDef) -> JSAtom {
    JS_DupAtom(ctx, (*m).module_name)
}
pub unsafe fn JS_SetModuleLoaderFunc(
    rt: *mut JSRuntime,
    module_normalize: Option<JSModuleNormalizeFunc>,
    module_loader: Option<JSModuleLoaderFunc>,
    opaque: *mut c_void,
) {
    (*rt).module_normalize_func = module_normalize;
    (*rt).module_loader_has_attr = 0;
    (*rt).u.module_loader_func = module_loader;
    (*rt).module_check_attrs = None;
    (*rt).module_loader_opaque = opaque;
}
pub unsafe fn JS_SetModuleLoaderFunc2(
    rt: *mut JSRuntime,
    module_normalize: Option<JSModuleNormalizeFunc>,
    module_loader: Option<JSModuleLoaderFunc2>,
    module_check_attrs: Option<JSModuleCheckSupportedImportAttributes>,
    opaque: *mut c_void,
) {
    (*rt).module_normalize_func = module_normalize;
    (*rt).module_loader_has_attr = 1;
    (*rt).u.module_loader_func2 = module_loader;
    (*rt).module_check_attrs = module_check_attrs;
    (*rt).module_loader_opaque = opaque;
}
#[repr(C)]
struct JSClassShortDef {
    class_name: JSAtom,
    finalizer: Option<JSClassFinalizer>,
    gc_mark: Option<JSClassGCMark>,
}
unsafe fn init_class_range(
    rt: *mut JSRuntime,
    tab: *const JSClassShortDef,
    start: i32,
    count: i32,
) -> i32 {
    for i in 0..count {
        let mut cm: JSClassDef = core::mem::zeroed();
        cm.finalizer = (*tab.add(i as usize)).finalizer;
        cm.gc_mark = (*tab.add(i as usize)).gc_mark;
        if JS_NewClass1(
            rt,
            (i + start) as u32,
            &cm,
            (*tab.add(i as usize)).class_name,
        ) < 0
        {
            return -1;
        }
    }
    0
}
