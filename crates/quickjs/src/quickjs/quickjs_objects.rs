// quickjs.c object layout and object/shape accessors. MIT.
// These names have only forward pointer declarations here; their payload
// definitions are added at their own class-specific source sections.
use super::*;
macro_rules! forward_object_payloads {($($name:ident),*$(,)?)=>{$(#[repr(C)]pub(crate) struct $name{_opaque:[u8;0],_pin:core::marker::PhantomPinned})*};}
forward_object_payloads!();
#[repr(C)]
#[derive(Clone, Copy)]
pub(crate) struct JSObjectFunc {
    pub(crate) function_bytecode: *mut JSFunctionBytecode,
    pub(crate) var_refs: *mut *mut JSVarRef,
    pub(crate) home_object: *mut JSObject,
}
#[repr(C)]
#[derive(Clone, Copy)]
pub(crate) struct JSObjectCFunc {
    pub(crate) realm: *mut JSContext,
    pub(crate) c_function: JSCFunctionType,
    pub(crate) length: u8,
    pub(crate) cproto: u8,
    pub(crate) magic: i16,
}
#[repr(C)]
#[derive(Clone, Copy)]
pub(crate) union JSArrayCapacity {
    pub(crate) size: u32,
    pub(crate) typed_array: *mut JSTypedArray,
}
#[repr(C)]
#[derive(Clone, Copy)]
pub(crate) union JSArrayElements {
    pub(crate) values: *mut JSValue,
    pub(crate) var_refs: *mut *mut JSVarRef,
    pub(crate) ptr: *mut c_void,
    pub(crate) int8_ptr: *mut i8,
    pub(crate) uint8_ptr: *mut u8,
    pub(crate) int16_ptr: *mut i16,
    pub(crate) uint16_ptr: *mut u16,
    pub(crate) int32_ptr: *mut i32,
    pub(crate) uint32_ptr: *mut u32,
    pub(crate) int64_ptr: *mut i64,
    pub(crate) uint64_ptr: *mut u64,
    pub(crate) fp16_ptr: *mut u16,
    pub(crate) float_ptr: *mut f32,
    pub(crate) double_ptr: *mut f64,
}
#[repr(C)]
#[derive(Clone, Copy)]
pub(crate) struct JSObjectArray {
    pub(crate) u1: JSArrayCapacity,
    pub(crate) u: JSArrayElements,
    pub(crate) count: u32,
}
#[repr(C)]
pub(crate) union JSObjectUnion {
    pub(crate) opaque: *mut c_void,
    pub(crate) bound_function: *mut JSBoundFunction,
    pub(crate) c_function_data_record: *mut JSCFunctionDataRecord,
    pub(crate) for_in_iterator: *mut JSForInIterator,
    pub(crate) array_buffer: *mut JSArrayBuffer,
    pub(crate) typed_array: *mut JSTypedArray,
    pub(crate) map_state: *mut JSMapState,
    pub(crate) map_iterator_data: *mut JSMapIteratorData,
    pub(crate) array_iterator_data: *mut JSArrayIteratorData,
    pub(crate) regexp_string_iterator_data: *mut JSRegExpStringIteratorData,
    pub(crate) generator_data: *mut JSGeneratorData,
    pub(crate) iterator_concat_data: *mut JSIteratorConcatData,
    pub(crate) iterator_helper_data: *mut JSIteratorHelperData,
    pub(crate) iterator_wrap_data: *mut JSIteratorWrapData,
    pub(crate) proxy_data: *mut JSProxyData,
    pub(crate) promise_data: *mut JSPromiseData,
    pub(crate) promise_function_data: *mut JSPromiseFunctionData,
    pub(crate) async_function_data: *mut JSAsyncFunctionState,
    pub(crate) async_from_sync_iterator_data: *mut JSAsyncFromSyncIteratorData,
    pub(crate) async_generator_data: *mut JSAsyncGeneratorData,
    pub(crate) func: JSObjectFunc,
    pub(crate) cfunc: JSObjectCFunc,
    pub(crate) array: JSObjectArray,
    pub(crate) regexp: JSRegExp,
    pub(crate) object_data: JSValue,
    pub(crate) global_object: JSGlobalObject,
}
#[repr(C)]
pub(crate) struct JSObject {
    pub(crate) header: JSGCObjectHeader,
    pub(crate) object_flags: [u8; 2],
    pub(crate) class_id: u16,
    pub(crate) weakref_count: u32,
    pub(crate) shape: *mut JSShape,
    pub(crate) prop: *mut JSProperty,
    pub(crate) u: JSObjectUnion,
}
macro_rules! object_flag {
    ($get:ident,$set:ident,$byte:expr,$shift:expr) => {
        pub(crate) fn $get(&self) -> u8 {
            (self.object_flags[$byte] >> $shift) & 1
        }
        pub(crate) fn $set(&mut self, n: u8) {
            self.object_flags[$byte] =
                (self.object_flags[$byte] & !(1 << $shift)) | ((n & 1) << $shift);
        }
    };
}
impl JSObject {
    object_flag!(is_std_array_prototype, set_is_std_array_prototype, 0, 0);
    object_flag!(extensible, set_extensible, 0, 1);
    object_flag!(free_mark, set_free_mark, 0, 2);
    object_flag!(is_exotic, set_is_exotic, 0, 3);
    object_flag!(fast_array, set_fast_array, 0, 4);
    object_flag!(is_constructor, set_is_constructor, 0, 5);
    object_flag!(has_immutable_prototype, set_has_immutable_prototype, 0, 6);
    object_flag!(tmp_mark, set_tmp_mark, 0, 7);
    object_flag!(is_HTMLDDA, set_is_HTMLDDA, 1, 0);
}
#[repr(C)]
pub(crate) struct JSMapRecord {
    pub(crate) ref_count: i32,
    pub(crate) empty: i8,
    pub(crate) link: list_head,
    pub(crate) hash_next: *mut JSMapRecord,
    pub(crate) key: JSValue,
    pub(crate) value: JSValue,
}
#[repr(C)]
pub(crate) struct JSMapState {
    pub(crate) is_weak: JS_BOOL,
    pub(crate) records: list_head,
    pub(crate) record_count: u32,
    pub(crate) hash_table: *mut *mut JSMapRecord,
    pub(crate) hash_bits: i32,
    pub(crate) hash_size: u32,
    pub(crate) record_count_threshold: u32,
    pub(crate) weakref_header: JSWeakRefHeader,
}
pub unsafe fn JS_GetClassID(v: JSValue) -> JSClassID {
    if JS_VALUE_GET_TAG(v) != JS_TAG_OBJECT {
        return JS_INVALID_CLASS_ID as u32;
    }
    (*JS_VALUE_GET_PTR(v).cast::<JSObject>()).class_id as u32
}
pub unsafe fn JS_IsFunction(ctx: *mut JSContext, val: JSValueConst) -> JS_BOOL {
    if JS_VALUE_GET_TAG(val) != JS_TAG_OBJECT {
        return 0;
    }
    let p = JS_VALUE_GET_PTR(val).cast::<JSObject>();
    match (*p).class_id as u32 {
        JS_CLASS_BYTECODE_FUNCTION => 1,
        JS_CLASS_PROXY => (*(*p).u.proxy_data).is_func as i32,
        _ => (*(*(*ctx).rt).class_array.add((*p).class_id as usize))
            .call
            .is_some() as i32,
    }
}
pub unsafe fn JS_IsCFunction(
    _ctx: *mut JSContext,
    val: JSValueConst,
    func: Option<JSCFunction>,
    magic: i32,
) -> JS_BOOL {
    if JS_VALUE_GET_TAG(val) != JS_TAG_OBJECT {
        return 0;
    }
    let p = JS_VALUE_GET_PTR(val).cast::<JSObject>();
    if (*p).class_id as u32 != JS_CLASS_C_FUNCTION {
        return 0;
    }
    // Compare actual callback identities; Rust's ordinary fn equality emits an
    // unrelated codegen warning and does not express the original pointer intent.
    let a = (*p).u.cfunc.c_function.generic.map(|f| f as usize);
    let b = func.map(|f| f as usize);
    (a == b && (*p).u.cfunc.magic as i32 == magic) as i32
}
pub unsafe fn JS_IsConstructor(_ctx: *mut JSContext, val: JSValueConst) -> JS_BOOL {
    if JS_VALUE_GET_TAG(val) != JS_TAG_OBJECT {
        0
    } else {
        (*JS_VALUE_GET_PTR(val).cast::<JSObject>()).is_constructor() as i32
    }
}
pub unsafe fn JS_SetConstructorBit(
    _ctx: *mut JSContext,
    func_obj: JSValueConst,
    val: JS_BOOL,
) -> JS_BOOL {
    if JS_VALUE_GET_TAG(func_obj) != JS_TAG_OBJECT {
        return 0;
    }
    (*JS_VALUE_GET_PTR(func_obj).cast::<JSObject>()).set_is_constructor(val as u8);
    1
}
pub unsafe fn JS_IsError(_ctx: *mut JSContext, val: JSValueConst) -> JS_BOOL {
    if JS_VALUE_GET_TAG(val) != JS_TAG_OBJECT {
        0
    } else {
        ((*JS_VALUE_GET_PTR(val).cast::<JSObject>()).class_id as u32 == JS_CLASS_ERROR) as i32
    }
}
pub unsafe fn JS_SetUncatchableException(ctx: *mut JSContext, flag: JS_BOOL) {
    (*(*ctx).rt).current_exception_is_uncatchable = flag as i8;
}

/// Non-consuming owner query used to prevent host callback reentry after a
/// native interrupt has terminated the current VM execution.
pub unsafe fn JS_IsUncatchableException(ctx: *mut JSContext) -> JS_BOOL {
    (*(*ctx).rt).current_exception_is_uncatchable as JS_BOOL
}
pub unsafe fn JS_SetOpaque(obj: JSValue, opaque: *mut c_void) {
    if JS_VALUE_GET_TAG(obj) == JS_TAG_OBJECT {
        (*JS_VALUE_GET_PTR(obj).cast::<JSObject>()).u.opaque = opaque;
    }
}
pub unsafe fn JS_GetOpaque(obj: JSValueConst, class_id: JSClassID) -> *mut c_void {
    if JS_VALUE_GET_TAG(obj) != JS_TAG_OBJECT {
        return ptr::null_mut();
    }
    let p = JS_VALUE_GET_PTR(obj).cast::<JSObject>();
    if (*p).class_id as u32 != class_id {
        return ptr::null_mut();
    }
    (*p).u.opaque
}
pub unsafe fn JS_SetIsHTMLDDA(_ctx: *mut JSContext, obj: JSValueConst) {
    if JS_VALUE_GET_TAG(obj) == JS_TAG_OBJECT {
        (*JS_VALUE_GET_PTR(obj).cast::<JSObject>()).set_is_HTMLDDA(1);
    }
}
pub(crate) unsafe fn JS_IsHTMLDDA(_ctx: *mut JSContext, obj: JSValueConst) -> JS_BOOL {
    if JS_VALUE_GET_TAG(obj) != JS_TAG_OBJECT {
        0
    } else {
        (*JS_VALUE_GET_PTR(obj).cast::<JSObject>()).is_HTMLDDA() as i32
    }
}
pub(crate) unsafe fn find_own_property1(p: *mut JSObject, atom: JSAtom) -> *mut JSShapeProperty {
    let sh = (*p).shape;
    let mut h = *ptr::addr_of!((*sh).hash_table)
        .cast::<u32>()
        .add((atom & (*sh).prop_hash_mask) as usize);
    let prop = get_shape_prop(sh);
    while h != 0 {
        let pr = prop.add(h as usize - 1);
        if (*pr).atom == atom {
            return pr;
        }
        h = (*pr).hash_next();
    }
    ptr::null_mut()
}
pub(crate) unsafe fn find_own_property(
    ppr: *mut *mut JSProperty,
    p: *mut JSObject,
    atom: JSAtom,
) -> *mut JSShapeProperty {
    let sh = (*p).shape;
    let mut h = *ptr::addr_of!((*sh).hash_table)
        .cast::<u32>()
        .add((atom & (*sh).prop_hash_mask) as usize);
    let prop = get_shape_prop(sh);
    while h != 0 {
        let pr = prop.add(h as usize - 1);
        if (*pr).atom == atom {
            *ppr = (*p).prop.add(h as usize - 1);
            return pr;
        }
        h = (*pr).hash_next();
    }
    *ppr = ptr::null_mut();
    ptr::null_mut()
}
pub(crate) unsafe fn js_autoinit_get_realm(pr: *mut JSProperty) -> *mut JSContext {
    ((*pr).u.init.realm_and_id & !3) as *mut JSContext
}
pub(crate) unsafe fn js_autoinit_get_id(pr: *mut JSProperty) -> JSAutoInitIDEnum {
    ((*pr).u.init.realm_and_id & 3) as u32
}
pub(crate) unsafe fn set_cycle_flag(_ctx: *mut JSContext, _obj: JSValueConst) {} // Original function body is empty.
pub unsafe fn JS_DupContext(ctx: *mut JSContext) -> *mut JSContext {
    (*js_rc(ctx.cast())).ref_count += 1;
    ctx
}
pub unsafe fn JS_IsJobPending(rt: *mut JSRuntime) -> JS_BOOL {
    (list_empty(&mut (*rt).job_list) == 0) as i32
}
