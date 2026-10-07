//! quickjs.h. Copyright 2017-2021 Fabrice Bellard and Charlie Gordon; MIT.
//! Value representation, public types, constants and inline ownership wrappers.
//! Runtime-backed entry points are implemented in quickjs.c's Rust translation.
//! C differential oracles run independently; the runtime uses no C QuickJS FFI.
pub use super::quickjs::{JSClass, JSContext, JSGCObjectHeader, JSModuleDef, JSRuntime};
use core::ffi::c_void;
pub type JS_BOOL = i32;
pub type JSAtom = u32;
pub type JSClassID = u32;
pub const JS_LIMB_BITS: usize = usize::BITS as usize;
pub const JS_SHORT_BIG_INT_BITS: usize = JS_LIMB_BITS;
include!("quickjs_header_constants.rs");
#[repr(C)]
pub struct JSRefCountHeader {
    pub ref_count: i32,
}
#[cfg(target_pointer_width = "64")]
#[repr(C)]
#[derive(Clone, Copy)]
pub union JSValueUnion {
    pub uint64: u64,
    pub float64: f64,
    pub ptr: *mut c_void,
    pub short_big_int: i64,
}
#[cfg(target_pointer_width = "64")]
#[repr(C)]
#[derive(Clone, Copy)]
pub struct JSValue {
    pub u: JSValueUnion,
    pub tag: i64,
}
#[cfg(target_pointer_width = "32")]
pub type JSValue = u64;
pub type JSValueConst = JSValue;
pub const JS_FLOAT64_NAN: f64 = f64::NAN;
pub const JS_FLOAT64_TAG_ADDEND: i32 = 0x7ff80000 - JS_TAG_FIRST + 1;
#[cfg(target_pointer_width = "64")]
pub const JS_NAN: JSValue = JSValue {
    u: JSValueUnion {
        float64: JS_FLOAT64_NAN,
    },
    tag: JS_TAG_FLOAT64 as i64,
};
#[cfg(target_pointer_width = "32")]
pub const JS_NAN: JSValue =
    0x7ff8000000000000u64.wrapping_sub((JS_FLOAT64_TAG_ADDEND as u64) << 32);
#[cfg(target_pointer_width = "64")]
#[inline]
pub const fn JS_VALUE_GET_TAG(v: JSValue) -> i32 {
    v.tag as i32
}
#[cfg(target_pointer_width = "32")]
#[inline]
pub const fn JS_VALUE_GET_TAG(v: JSValue) -> i32 {
    (v >> 32) as i32
}
#[inline]
pub fn JS_VALUE_GET_NORM_TAG(v: JSValue) -> i32 {
    let tag = JS_VALUE_GET_TAG(v);
    if JS_TAG_IS_FLOAT64(tag) != 0 {
        JS_TAG_FLOAT64
    } else {
        tag
    }
}
#[cfg(target_pointer_width = "64")]
#[inline]
pub fn JS_VALUE_GET_INT(v: JSValue) -> i32 {
    unsafe { v.u.uint64 as i32 }
}
#[cfg(target_pointer_width = "32")]
#[inline]
pub fn JS_VALUE_GET_INT(v: JSValue) -> i32 {
    v as i32
}
#[inline]
pub fn JS_VALUE_GET_BOOL(v: JSValue) -> i32 {
    JS_VALUE_GET_INT(v)
}
#[cfg(target_pointer_width = "64")]
#[inline]
pub fn JS_VALUE_GET_FLOAT64(v: JSValue) -> f64 {
    unsafe { v.u.float64 }
}
#[cfg(target_pointer_width = "32")]
#[inline]
pub fn JS_VALUE_GET_FLOAT64(v: JSValue) -> f64 {
    f64::from_bits(v.wrapping_add((JS_FLOAT64_TAG_ADDEND as u64) << 32))
}
#[cfg(target_pointer_width = "64")]
#[inline]
pub fn JS_VALUE_GET_SHORT_BIG_INT(v: JSValue) -> i64 {
    unsafe { v.u.short_big_int }
}
#[cfg(target_pointer_width = "32")]
#[inline]
pub fn JS_VALUE_GET_SHORT_BIG_INT(v: JSValue) -> i32 {
    v as i32
}
#[cfg(target_pointer_width = "64")]
#[inline]
pub fn JS_VALUE_GET_PTR(v: JSValue) -> *mut c_void {
    unsafe { v.u.ptr }
}
#[cfg(target_pointer_width = "32")]
#[inline]
pub fn JS_VALUE_GET_PTR(v: JSValue) -> *mut c_void {
    v as usize as *mut c_void
}
#[cfg(target_pointer_width = "64")]
#[inline]
pub const fn JS_MKVAL(tag: i32, val: i32) -> JSValue {
    JSValue {
        u: JSValueUnion {
            uint64: val as u32 as u64,
        },
        tag: tag as i64,
    }
}
#[cfg(target_pointer_width = "32")]
#[inline]
pub const fn JS_MKVAL(tag: i32, val: i32) -> JSValue {
    ((tag as u64) << 32) | val as u32 as u64
}
#[cfg(target_pointer_width = "64")]
#[inline]
pub fn JS_MKPTR(tag: i32, p: *mut c_void) -> JSValue {
    JSValue {
        u: JSValueUnion { ptr: p },
        tag: tag as i64,
    }
}
#[cfg(target_pointer_width = "32")]
#[inline]
pub fn JS_MKPTR(tag: i32, p: *mut c_void) -> JSValue {
    ((tag as u64) << 32) | p as usize as u64
}
#[inline]
pub fn JS_TAG_IS_FLOAT64(tag: i32) -> JS_BOOL {
    if cfg!(target_pointer_width = "32") {
        ((tag.wrapping_sub(JS_TAG_FIRST) as u32) >= (JS_TAG_FLOAT64 - JS_TAG_FIRST) as u32) as i32
    } else {
        ((tag as u32) == JS_TAG_FLOAT64 as u32) as i32
    }
}
#[inline]
pub fn JS_VALUE_IS_NAN(v: JSValue) -> JS_BOOL {
    #[cfg(target_pointer_width = "64")]
    {
        if v.tag != JS_TAG_FLOAT64 as i64 {
            return 0;
        }
        ((unsafe { v.u.float64 }.to_bits() & 0x7fffffffffffffff) > 0x7ff0000000000000) as i32
    }
    #[cfg(target_pointer_width = "32")]
    {
        (JS_VALUE_GET_TAG(v) == (JS_NAN >> 32) as i32) as i32
    }
}
#[inline]
pub fn JS_VALUE_IS_BOTH_INT(v1: JSValue, v2: JSValue) -> JS_BOOL {
    ((JS_VALUE_GET_TAG(v1) | JS_VALUE_GET_TAG(v2)) == 0) as i32
}
#[inline]
pub fn JS_VALUE_IS_BOTH_FLOAT(v1: JSValue, v2: JSValue) -> JS_BOOL {
    (JS_TAG_IS_FLOAT64(JS_VALUE_GET_TAG(v1)) != 0 && JS_TAG_IS_FLOAT64(JS_VALUE_GET_TAG(v2)) != 0)
        as i32
}
#[inline]
pub fn JS_VALUE_HAS_REF_COUNT(v: JSValue) -> JS_BOOL {
    ((JS_VALUE_GET_TAG(v) as u32) >= JS_TAG_FIRST as u32) as i32
}
pub const JS_NULL: JSValue = JS_MKVAL(JS_TAG_NULL, 0);
pub const JS_UNDEFINED: JSValue = JS_MKVAL(JS_TAG_UNDEFINED, 0);
pub const JS_FALSE: JSValue = JS_MKVAL(JS_TAG_BOOL, 0);
pub const JS_TRUE: JSValue = JS_MKVAL(JS_TAG_BOOL, 1);
pub const JS_EXCEPTION: JSValue = JS_MKVAL(JS_TAG_EXCEPTION, 0);
pub const JS_UNINITIALIZED: JSValue = JS_MKVAL(JS_TAG_UNINITIALIZED, 0);
#[inline]
pub fn __JS_NewFloat64(_ctx: *mut JSContext, d: f64) -> JSValue {
    #[cfg(target_pointer_width = "64")]
    {
        JSValue {
            u: JSValueUnion { float64: d },
            tag: JS_TAG_FLOAT64 as i64,
        }
    }
    #[cfg(target_pointer_width = "32")]
    {
        let bits = d.to_bits();
        if (bits & 0x7fffffffffffffff) > 0x7ff0000000000000 {
            JS_NAN
        } else {
            bits.wrapping_sub((JS_FLOAT64_TAG_ADDEND as u64) << 32)
        }
    }
}
#[cfg(target_pointer_width = "64")]
#[inline]
pub fn __JS_NewShortBigInt(_ctx: *mut JSContext, d: i64) -> JSValue {
    JSValue {
        u: JSValueUnion { short_big_int: d },
        tag: JS_TAG_SHORT_BIG_INT as i64,
    }
}
#[cfg(target_pointer_width = "32")]
#[inline]
pub fn __JS_NewShortBigInt(_ctx: *mut JSContext, d: i32) -> JSValue {
    JS_MKVAL(JS_TAG_SHORT_BIG_INT, d)
}
#[inline]
pub fn JS_NewBool(_ctx: *mut JSContext, val: JS_BOOL) -> JSValue {
    JS_MKVAL(JS_TAG_BOOL, (val != 0) as i32)
}
#[inline]
pub fn JS_NewInt32(_ctx: *mut JSContext, val: i32) -> JSValue {
    JS_MKVAL(JS_TAG_INT, val)
}
#[inline]
pub fn JS_NewCatchOffset(_ctx: *mut JSContext, val: i32) -> JSValue {
    JS_MKVAL(JS_TAG_CATCH_OFFSET, val)
}
#[inline]
pub fn JS_NewInt64(ctx: *mut JSContext, val: i64) -> JSValue {
    if val == val as i32 as i64 {
        JS_NewInt32(ctx, val as i32)
    } else {
        __JS_NewFloat64(ctx, val as f64)
    }
}
#[inline]
pub fn JS_NewUint32(ctx: *mut JSContext, val: u32) -> JSValue {
    if val <= 0x7fffffff {
        JS_NewInt32(ctx, val as i32)
    } else {
        __JS_NewFloat64(ctx, val as f64)
    }
}
#[inline]
pub fn JS_NewFloat64(ctx: *mut JSContext, d: f64) -> JSValue {
    if d >= i32::MIN as f64 && d <= i32::MAX as f64 {
        let val = d as i32;
        if d.to_bits() == (val as f64).to_bits() {
            return JS_MKVAL(JS_TAG_INT, val);
        }
    }
    __JS_NewFloat64(ctx, d)
}
#[inline]
pub fn JS_IsNumber(v: JSValueConst) -> JS_BOOL {
    let tag = JS_VALUE_GET_TAG(v);
    (tag == JS_TAG_INT || JS_TAG_IS_FLOAT64(tag) != 0) as i32
}
#[inline]
pub fn JS_IsBigInt(_ctx: *mut JSContext, v: JSValueConst) -> JS_BOOL {
    let tag = JS_VALUE_GET_TAG(v);
    (tag == JS_TAG_BIG_INT || tag == JS_TAG_SHORT_BIG_INT) as i32
}
#[inline]
pub fn JS_IsBool(v: JSValueConst) -> JS_BOOL {
    (JS_VALUE_GET_TAG(v) == JS_TAG_BOOL) as i32
}
#[inline]
pub fn JS_IsNull(v: JSValueConst) -> JS_BOOL {
    (JS_VALUE_GET_TAG(v) == JS_TAG_NULL) as i32
}
#[inline]
pub fn JS_IsUndefined(v: JSValueConst) -> JS_BOOL {
    (JS_VALUE_GET_TAG(v) == JS_TAG_UNDEFINED) as i32
}
#[inline]
pub fn JS_IsException(v: JSValueConst) -> JS_BOOL {
    (JS_VALUE_GET_TAG(v) == JS_TAG_EXCEPTION) as i32
}
#[inline]
pub fn JS_IsUninitialized(v: JSValueConst) -> JS_BOOL {
    (JS_VALUE_GET_TAG(v) == JS_TAG_UNINITIALIZED) as i32
}
#[inline]
pub fn JS_IsString(v: JSValueConst) -> JS_BOOL {
    let tag = JS_VALUE_GET_TAG(v);
    (tag == JS_TAG_STRING || tag == JS_TAG_STRING_ROPE) as i32
}
#[inline]
pub fn JS_IsSymbol(v: JSValueConst) -> JS_BOOL {
    (JS_VALUE_GET_TAG(v) == JS_TAG_SYMBOL) as i32
}
#[inline]
pub fn JS_IsObject(v: JSValueConst) -> JS_BOOL {
    (JS_VALUE_GET_TAG(v) == JS_TAG_OBJECT) as i32
}
#[inline]
pub unsafe fn __js_rc(p: *mut c_void) -> *mut JSRefCountHeader {
    p.cast::<u32>().sub(1).cast()
}
#[inline]
pub unsafe fn JS_DupValue(_ctx: *mut JSContext, v: JSValueConst) -> JSValue {
    if JS_VALUE_HAS_REF_COUNT(v) != 0 {
        let p = __js_rc(JS_VALUE_GET_PTR(v));
        (*p).ref_count = (*p).ref_count.wrapping_add(1);
    }
    v
}
#[inline]
pub unsafe fn JS_DupValueRT(_rt: *mut JSRuntime, v: JSValueConst) -> JSValue {
    if JS_VALUE_HAS_REF_COUNT(v) != 0 {
        let p = __js_rc(JS_VALUE_GET_PTR(v));
        (*p).ref_count = (*p).ref_count.wrapping_add(1);
    }
    v
}
pub type JSCFunction = unsafe fn(*mut JSContext, JSValueConst, i32, *mut JSValueConst) -> JSValue;
pub type JSCFunctionMagic =
    unsafe fn(*mut JSContext, JSValueConst, i32, *mut JSValueConst, i32) -> JSValue;
pub type JSCFunctionData =
    unsafe fn(*mut JSContext, JSValueConst, i32, *mut JSValueConst, i32, *mut JSValue) -> JSValue;
#[repr(C)]
pub struct JSMallocState {
    pub malloc_count: usize,
    pub malloc_size: usize,
    pub malloc_limit: usize,
    pub opaque: *mut c_void,
}
#[repr(C)]
pub struct JSMallocFunctions {
    pub js_malloc: Option<unsafe fn(*mut JSMallocState, usize) -> *mut c_void>,
    pub js_free: Option<unsafe fn(*mut JSMallocState, *mut c_void)>,
    pub js_realloc: Option<unsafe fn(*mut JSMallocState, *mut c_void, usize) -> *mut c_void>,
    pub js_malloc_usable_size: Option<unsafe fn(*const c_void) -> usize>,
}
#[repr(C)]
pub struct JSPropertyEnum {
    pub is_enumerable: JS_BOOL,
    pub atom: JSAtom,
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct JSPropertyDescriptor {
    pub flags: i32,
    pub value: JSValue,
    pub getter: JSValue,
    pub setter: JSValue,
}
#[repr(C)]
pub struct JSSharedArrayBufferFunctions {
    pub sab_alloc: Option<unsafe fn(*mut c_void, usize) -> *mut c_void>,
    pub sab_free: Option<unsafe fn(*mut c_void, *mut c_void)>,
    pub sab_dup: Option<unsafe fn(*mut c_void, *mut c_void)>,
    pub sab_opaque: *mut c_void,
}
pub type JSPromiseStateEnum = i32;
pub const JS_PROMISE_PENDING: JSPromiseStateEnum = 0;
pub const JS_PROMISE_FULFILLED: JSPromiseStateEnum = 1;
pub const JS_PROMISE_REJECTED: JSPromiseStateEnum = 2;
pub type JSHostPromiseRejectionTracker =
    unsafe fn(*mut JSContext, JSValueConst, JSValueConst, JS_BOOL, *mut c_void);
pub type JSInterruptHandler = unsafe fn(*mut JSRuntime, *mut c_void) -> i32;

pub type JS_MarkFunc = unsafe fn(*mut JSRuntime, *mut JSGCObjectHeader);
pub type JSFreeArrayBufferDataFunc = unsafe fn(*mut JSRuntime, *mut c_void, *mut c_void);

pub unsafe fn JS_FreeValue(ctx: *mut JSContext, v: JSValue) {
    if JS_VALUE_HAS_REF_COUNT(v) != 0 {
        let p = __js_rc(JS_VALUE_GET_PTR(v));
        (*p).ref_count -= 1;
        if (*p).ref_count <= 0 {
            super::quickjs::__JS_FreeValue(ctx, v);
        }
    }
}
pub unsafe fn JS_FreeValueRT(rt: *mut JSRuntime, v: JSValue) {
    if JS_VALUE_HAS_REF_COUNT(v) != 0 {
        let p = __js_rc(JS_VALUE_GET_PTR(v));
        (*p).ref_count -= 1;
        if (*p).ref_count <= 0 {
            super::quickjs::__JS_FreeValueRT(rt, v);
        }
    }
}
#[repr(C)]
pub struct JSMemoryUsage {
    pub malloc_size: i64,
    pub malloc_limit: i64,
    pub memory_used_size: i64,
    pub malloc_count: i64,
    pub memory_used_count: i64,
    pub atom_count: i64,
    pub atom_size: i64,
    pub str_count: i64,
    pub str_size: i64,
    pub obj_count: i64,
    pub obj_size: i64,
    pub prop_count: i64,
    pub prop_size: i64,
    pub shape_count: i64,
    pub shape_size: i64,
    pub js_func_count: i64,
    pub js_func_size: i64,
    pub js_func_code_size: i64,
    pub js_func_pc2line_count: i64,
    pub js_func_pc2line_size: i64,
    pub c_func_count: i64,
    pub array_count: i64,
    pub fast_array_count: i64,
    pub fast_array_elements: i64,
    pub binary_object_count: i64,
    pub binary_object_size: i64,
}
#[repr(C)]
pub struct JSClassExoticMethods {
    pub get_own_property:
        Option<unsafe fn(*mut JSContext, *mut JSPropertyDescriptor, JSValueConst, JSAtom) -> i32>,
    pub get_own_property_names:
        Option<unsafe fn(*mut JSContext, *mut *mut JSPropertyEnum, *mut u32, JSValueConst) -> i32>,
    pub delete_property: Option<unsafe fn(*mut JSContext, JSValueConst, JSAtom) -> i32>,
    pub define_own_property: Option<
        unsafe fn(
            *mut JSContext,
            JSValueConst,
            JSAtom,
            JSValueConst,
            JSValueConst,
            JSValueConst,
            i32,
        ) -> i32,
    >,
    pub has_property: Option<unsafe fn(*mut JSContext, JSValueConst, JSAtom) -> i32>,
    pub get_property:
        Option<unsafe fn(*mut JSContext, JSValueConst, JSAtom, JSValueConst) -> JSValue>,
    pub set_property: Option<
        unsafe fn(*mut JSContext, JSValueConst, JSAtom, JSValueConst, JSValueConst, i32) -> i32,
    >,
    pub get_prototype: Option<unsafe fn(*mut JSContext, JSValueConst) -> JSValue>,
    pub set_prototype: Option<unsafe fn(*mut JSContext, JSValueConst, JSValueConst) -> i32>,
    pub is_extensible: Option<unsafe fn(*mut JSContext, JSValueConst) -> i32>,
    pub prevent_extensions: Option<unsafe fn(*mut JSContext, JSValueConst) -> i32>,
}
pub type JSClassFinalizer = unsafe fn(*mut JSRuntime, JSValue);
pub type JSClassGCMark = unsafe fn(*mut JSRuntime, JSValueConst, Option<JS_MarkFunc>);
pub type JSClassCall =
    unsafe fn(*mut JSContext, JSValueConst, JSValueConst, i32, *mut JSValueConst, i32) -> JSValue;
#[repr(C)]
pub struct JSClassDef {
    pub class_name: *const core::ffi::c_char,
    pub finalizer: Option<JSClassFinalizer>,
    pub gc_mark: Option<JSClassGCMark>,
    pub call: Option<JSClassCall>,
    pub exotic: *mut JSClassExoticMethods,
}
pub type JSModuleNormalizeFunc = unsafe fn(
    *mut JSContext,
    *const core::ffi::c_char,
    *const core::ffi::c_char,
    *mut c_void,
) -> *mut core::ffi::c_char;
pub type JSModuleLoaderFunc =
    unsafe fn(*mut JSContext, *const core::ffi::c_char, *mut c_void) -> *mut JSModuleDef;
pub type JSModuleLoaderFunc2 = unsafe fn(
    *mut JSContext,
    *const core::ffi::c_char,
    *mut c_void,
    JSValueConst,
) -> *mut JSModuleDef;
pub type JSModuleCheckSupportedImportAttributes =
    unsafe fn(*mut JSContext, *mut c_void, JSValueConst) -> i32;
pub type JSJobFunc = unsafe fn(*mut JSContext, i32, *mut JSValueConst) -> JSValue;
pub type JSModuleInitFunc = unsafe fn(*mut JSContext, *mut JSModuleDef) -> i32;
pub type JSCFunctionEnum = i32;
pub const JS_CFUNC_generic: JSCFunctionEnum = 0;
pub const JS_CFUNC_generic_magic: JSCFunctionEnum = 1;
pub const JS_CFUNC_constructor: JSCFunctionEnum = 2;
pub const JS_CFUNC_constructor_magic: JSCFunctionEnum = 3;
pub const JS_CFUNC_constructor_or_func: JSCFunctionEnum = 4;
pub const JS_CFUNC_constructor_or_func_magic: JSCFunctionEnum = 5;
pub const JS_CFUNC_f_f: JSCFunctionEnum = 6;
pub const JS_CFUNC_f_f_f: JSCFunctionEnum = 7;
pub const JS_CFUNC_getter: JSCFunctionEnum = 8;
pub const JS_CFUNC_setter: JSCFunctionEnum = 9;
pub const JS_CFUNC_getter_magic: JSCFunctionEnum = 10;
pub const JS_CFUNC_setter_magic: JSCFunctionEnum = 11;
pub const JS_CFUNC_iterator_next: JSCFunctionEnum = 12;
#[repr(C)]
#[derive(Clone, Copy)]
pub union JSCFunctionType {
    pub generic: Option<JSCFunction>,
    pub generic_magic: Option<JSCFunctionMagic>,
    pub constructor: Option<JSCFunction>,
    pub constructor_magic: Option<JSCFunctionMagic>,
    pub constructor_or_func: Option<JSCFunction>,
    pub f_f: Option<unsafe fn(f64) -> f64>,
    pub f_f_f: Option<unsafe fn(f64, f64) -> f64>,
    pub getter: Option<unsafe fn(*mut JSContext, JSValueConst) -> JSValue>,
    pub setter: Option<unsafe fn(*mut JSContext, JSValueConst, JSValueConst) -> JSValue>,
    pub getter_magic: Option<unsafe fn(*mut JSContext, JSValueConst, i32) -> JSValue>,
    pub setter_magic: Option<unsafe fn(*mut JSContext, JSValueConst, JSValueConst, i32) -> JSValue>,
    pub iterator_next: Option<
        unsafe fn(*mut JSContext, JSValueConst, i32, *mut JSValueConst, *mut i32, i32) -> JSValue,
    >,
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct JSCFunctionListFunc {
    pub length: u8,
    pub cproto: u8,
    pub cfunc: JSCFunctionType,
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct JSCFunctionListGetSet {
    pub get: JSCFunctionType,
    pub set: JSCFunctionType,
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct JSCFunctionListAlias {
    pub name: *const core::ffi::c_char,
    pub base: i32,
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct JSCFunctionListPropList {
    pub tab: *const JSCFunctionListEntry,
    pub len: i32,
}
#[repr(C)]
#[derive(Clone, Copy)]
pub union JSCFunctionListUnion {
    pub func: JSCFunctionListFunc,
    pub getset: JSCFunctionListGetSet,
    pub alias: JSCFunctionListAlias,
    pub prop_list: JSCFunctionListPropList,
    pub str: *const core::ffi::c_char,
    pub i32: i32,
    pub i64: i64,
    pub f64: f64,
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct JSCFunctionListEntry {
    pub name: *const core::ffi::c_char,
    pub prop_flags: u8,
    pub def_type: u8,
    pub magic: i16,
    pub u: JSCFunctionListUnion,
}
#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct JSPrintValueOptions {
    flags: i32,
    pub max_depth: u32,
    pub max_string_length: u32,
    pub max_item_count: u32,
}
impl JSPrintValueOptions {
    pub fn show_hidden(&self) -> JS_BOOL {
        self.flags as i8 as i32
    }
    pub fn set_show_hidden(&mut self, v: JS_BOOL) {
        self.flags = (self.flags & !255) | (v & 255);
    }
    pub fn raw_dump(&self) -> JS_BOOL {
        (self.flags >> 8) as i8 as i32
    }
    pub fn set_raw_dump(&mut self, v: JS_BOOL) {
        self.flags = (self.flags & !(255 << 8)) | ((v & 255) << 8);
    }
}
pub type JSPrintValueWrite = unsafe fn(*mut c_void, *const core::ffi::c_char, usize);

// C's initializer macros become typed const constructors. Static table order,
// field values and union members are retained; no runtime work is introduced.
const fn function_list_entry(
    name: *const core::ffi::c_char,
    prop_flags: i32,
    def_type: i32,
    magic: i32,
    u: JSCFunctionListUnion,
) -> JSCFunctionListEntry {
    JSCFunctionListEntry {
        name,
        prop_flags: prop_flags as u8,
        def_type: def_type as u8,
        magic: magic as i16,
        u,
    }
}
pub const fn JS_CFUNC_DEF(
    name: *const core::ffi::c_char,
    length: i32,
    func: Option<JSCFunction>,
) -> JSCFunctionListEntry {
    function_list_entry(
        name,
        JS_PROP_WRITABLE | JS_PROP_CONFIGURABLE,
        JS_DEF_CFUNC,
        0,
        JSCFunctionListUnion {
            func: JSCFunctionListFunc {
                length: length as u8,
                cproto: JS_CFUNC_generic as u8,
                cfunc: JSCFunctionType { generic: func },
            },
        },
    )
}
pub const fn JS_CFUNC_MAGIC_DEF(
    name: *const core::ffi::c_char,
    length: i32,
    func: Option<JSCFunctionMagic>,
    magic: i32,
) -> JSCFunctionListEntry {
    function_list_entry(
        name,
        JS_PROP_WRITABLE | JS_PROP_CONFIGURABLE,
        JS_DEF_CFUNC,
        magic,
        JSCFunctionListUnion {
            func: JSCFunctionListFunc {
                length: length as u8,
                cproto: JS_CFUNC_generic_magic as u8,
                cfunc: JSCFunctionType {
                    generic_magic: func,
                },
            },
        },
    )
}
pub const fn JS_CFUNC_SPECIAL_DEF(
    name: *const core::ffi::c_char,
    length: i32,
    cproto: JSCFunctionEnum,
    cfunc: JSCFunctionType,
) -> JSCFunctionListEntry {
    function_list_entry(
        name,
        JS_PROP_WRITABLE | JS_PROP_CONFIGURABLE,
        JS_DEF_CFUNC,
        0,
        JSCFunctionListUnion {
            func: JSCFunctionListFunc {
                length: length as u8,
                cproto: cproto as u8,
                cfunc,
            },
        },
    )
}
pub const fn JS_ITERATOR_NEXT_DEF(
    name: *const core::ffi::c_char,
    length: i32,
    func: Option<
        unsafe fn(*mut JSContext, JSValueConst, i32, *mut JSValueConst, *mut i32, i32) -> JSValue,
    >,
    magic: i32,
) -> JSCFunctionListEntry {
    function_list_entry(
        name,
        JS_PROP_WRITABLE | JS_PROP_CONFIGURABLE,
        JS_DEF_CFUNC,
        magic,
        JSCFunctionListUnion {
            func: JSCFunctionListFunc {
                length: length as u8,
                cproto: JS_CFUNC_iterator_next as u8,
                cfunc: JSCFunctionType {
                    iterator_next: func,
                },
            },
        },
    )
}
pub const fn JS_CGETSET_DEF(
    name: *const core::ffi::c_char,
    getter: Option<unsafe fn(*mut JSContext, JSValueConst) -> JSValue>,
    setter: Option<unsafe fn(*mut JSContext, JSValueConst, JSValueConst) -> JSValue>,
) -> JSCFunctionListEntry {
    function_list_entry(
        name,
        JS_PROP_CONFIGURABLE,
        JS_DEF_CGETSET,
        0,
        JSCFunctionListUnion {
            getset: JSCFunctionListGetSet {
                get: JSCFunctionType { getter },
                set: JSCFunctionType { setter },
            },
        },
    )
}
pub const fn JS_CGETSET_MAGIC_DEF(
    name: *const core::ffi::c_char,
    getter: Option<unsafe fn(*mut JSContext, JSValueConst, i32) -> JSValue>,
    setter: Option<unsafe fn(*mut JSContext, JSValueConst, JSValueConst, i32) -> JSValue>,
    magic: i32,
) -> JSCFunctionListEntry {
    function_list_entry(
        name,
        JS_PROP_CONFIGURABLE,
        JS_DEF_CGETSET_MAGIC,
        magic,
        JSCFunctionListUnion {
            getset: JSCFunctionListGetSet {
                get: JSCFunctionType {
                    getter_magic: getter,
                },
                set: JSCFunctionType {
                    setter_magic: setter,
                },
            },
        },
    )
}
pub const fn JS_PROP_STRING_DEF(
    name: *const core::ffi::c_char,
    cstr: *const core::ffi::c_char,
    flags: i32,
) -> JSCFunctionListEntry {
    function_list_entry(
        name,
        flags,
        JS_DEF_PROP_STRING,
        0,
        JSCFunctionListUnion { str: cstr },
    )
}
pub const fn JS_PROP_INT32_DEF(
    name: *const core::ffi::c_char,
    val: i32,
    flags: i32,
) -> JSCFunctionListEntry {
    function_list_entry(
        name,
        flags,
        JS_DEF_PROP_INT32,
        0,
        JSCFunctionListUnion { i32: val },
    )
}
pub const fn JS_PROP_INT64_DEF(
    name: *const core::ffi::c_char,
    val: i64,
    flags: i32,
) -> JSCFunctionListEntry {
    function_list_entry(
        name,
        flags,
        JS_DEF_PROP_INT64,
        0,
        JSCFunctionListUnion { i64: val },
    )
}
pub const fn JS_PROP_DOUBLE_DEF(
    name: *const core::ffi::c_char,
    val: f64,
    flags: i32,
) -> JSCFunctionListEntry {
    function_list_entry(
        name,
        flags,
        JS_DEF_PROP_DOUBLE,
        0,
        JSCFunctionListUnion { f64: val },
    )
}
pub const fn JS_PROP_UNDEFINED_DEF(
    name: *const core::ffi::c_char,
    flags: i32,
) -> JSCFunctionListEntry {
    function_list_entry(
        name,
        flags,
        JS_DEF_PROP_UNDEFINED,
        0,
        JSCFunctionListUnion { i32: 0 },
    )
}
pub const fn JS_PROP_ATOM_DEF(
    name: *const core::ffi::c_char,
    val: i32,
    flags: i32,
) -> JSCFunctionListEntry {
    function_list_entry(
        name,
        flags,
        JS_DEF_PROP_ATOM,
        0,
        JSCFunctionListUnion { i32: val },
    )
}
pub const fn JS_PROP_BOOL_DEF(
    name: *const core::ffi::c_char,
    val: i32,
    flags: i32,
) -> JSCFunctionListEntry {
    function_list_entry(
        name,
        flags,
        JS_DEF_PROP_BOOL,
        0,
        JSCFunctionListUnion { i32: val },
    )
}
pub const fn JS_OBJECT_DEF(
    name: *const core::ffi::c_char,
    tab: *const JSCFunctionListEntry,
    len: i32,
    flags: i32,
) -> JSCFunctionListEntry {
    function_list_entry(
        name,
        flags,
        JS_DEF_OBJECT,
        0,
        JSCFunctionListUnion {
            prop_list: JSCFunctionListPropList { tab, len },
        },
    )
}
pub const fn JS_ALIAS_DEF(
    name: *const core::ffi::c_char,
    from: *const core::ffi::c_char,
) -> JSCFunctionListEntry {
    function_list_entry(
        name,
        JS_PROP_WRITABLE | JS_PROP_CONFIGURABLE,
        JS_DEF_ALIAS,
        0,
        JSCFunctionListUnion {
            alias: JSCFunctionListAlias {
                name: from,
                base: -1,
            },
        },
    )
}
pub const fn JS_ALIAS_BASE_DEF(
    name: *const core::ffi::c_char,
    from: *const core::ffi::c_char,
    base: i32,
) -> JSCFunctionListEntry {
    function_list_entry(
        name,
        JS_PROP_WRITABLE | JS_PROP_CONFIGURABLE,
        JS_DEF_ALIAS,
        0,
        JSCFunctionListUnion {
            alias: JSCFunctionListAlias { name: from, base },
        },
    )
}
#[cfg(test)]
#[path = "../tests/quickjs_value_differential.rs"]
mod tests;
