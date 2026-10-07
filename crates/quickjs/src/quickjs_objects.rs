// quickjs.c object layout and object/shape accessors. MIT.
// These names have only forward pointer declarations here; their payload
// definitions are added at their own class-specific source sections.
macro_rules! forward_object_payloads {($($name:ident),*$(,)?)=>{$(#[repr(C)]struct $name{_opaque:[u8;0],_pin:core::marker::PhantomPinned})*};}
forward_object_payloads!(
);
#[repr(C)]
#[derive(Clone, Copy)]
struct JSObjectFunc {
    function_bytecode: *mut JSFunctionBytecode,
    var_refs: *mut *mut JSVarRef,
    home_object: *mut JSObject,
}
#[repr(C)]
#[derive(Clone, Copy)]
struct JSObjectCFunc {
    realm: *mut JSContext,
    c_function: JSCFunctionType,
    length: u8,
    cproto: u8,
    magic: i16,
}
#[repr(C)]
#[derive(Clone, Copy)]
union JSArrayCapacity {
    size: u32,
    typed_array: *mut JSTypedArray,
}
#[repr(C)]
#[derive(Clone, Copy)]
union JSArrayElements {
    values: *mut JSValue,
    var_refs: *mut *mut JSVarRef,
    ptr: *mut c_void,
    int8_ptr: *mut i8,
    uint8_ptr: *mut u8,
    int16_ptr: *mut i16,
    uint16_ptr: *mut u16,
    int32_ptr: *mut i32,
    uint32_ptr: *mut u32,
    int64_ptr: *mut i64,
    uint64_ptr: *mut u64,
    fp16_ptr: *mut u16,
    float_ptr: *mut f32,
    double_ptr: *mut f64,
}
#[repr(C)]
#[derive(Clone, Copy)]
struct JSObjectArray {
    u1: JSArrayCapacity,
    u: JSArrayElements,
    count: u32,
}
#[repr(C)]
union JSObjectUnion {
    opaque: *mut c_void,
    bound_function: *mut JSBoundFunction,
    c_function_data_record: *mut JSCFunctionDataRecord,
    for_in_iterator: *mut JSForInIterator,
    array_buffer: *mut JSArrayBuffer,
    typed_array: *mut JSTypedArray,
    map_state: *mut JSMapState,
    map_iterator_data: *mut JSMapIteratorData,
    array_iterator_data: *mut JSArrayIteratorData,
    regexp_string_iterator_data: *mut JSRegExpStringIteratorData,
    generator_data: *mut JSGeneratorData,
    iterator_concat_data: *mut JSIteratorConcatData,
    iterator_helper_data: *mut JSIteratorHelperData,
    iterator_wrap_data: *mut JSIteratorWrapData,
    proxy_data: *mut JSProxyData,
    promise_data: *mut JSPromiseData,
    promise_function_data: *mut JSPromiseFunctionData,
    async_function_data: *mut JSAsyncFunctionState,
    async_from_sync_iterator_data: *mut JSAsyncFromSyncIteratorData,
    async_generator_data: *mut JSAsyncGeneratorData,
    func: JSObjectFunc,
    cfunc: JSObjectCFunc,
    array: JSObjectArray,
    regexp: JSRegExp,
    object_data: JSValue,
    global_object: JSGlobalObject,
}
#[repr(C)]
struct JSObject {
    header: JSGCObjectHeader,
    object_flags: [u8; 2],
    class_id: u16,
    weakref_count: u32,
    shape: *mut JSShape,
    prop: *mut JSProperty,
    u: JSObjectUnion,
}
macro_rules! object_flag {
    ($get:ident,$set:ident,$byte:expr,$shift:expr) => {
        fn $get(&self) -> u8 {
            (self.object_flags[$byte] >> $shift) & 1
        }
        fn $set(&mut self, n: u8) {
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
struct JSMapRecord {
    ref_count: i32,
    empty: i8,
    link: list_head,
    hash_next: *mut JSMapRecord,
    key: JSValue,
    value: JSValue,
}
#[repr(C)]
struct JSMapState {
    is_weak: JS_BOOL,
    records: list_head,
    record_count: u32,
    hash_table: *mut *mut JSMapRecord,
    hash_bits: i32,
    hash_size: u32,
    record_count_threshold: u32,
    weakref_header: JSWeakRefHeader,
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
unsafe fn JS_IsHTMLDDA(_ctx: *mut JSContext, obj: JSValueConst) -> JS_BOOL {
    if JS_VALUE_GET_TAG(obj) != JS_TAG_OBJECT {
        0
    } else {
        (*JS_VALUE_GET_PTR(obj).cast::<JSObject>()).is_HTMLDDA() as i32
    }
}
unsafe fn find_own_property1(p: *mut JSObject, atom: JSAtom) -> *mut JSShapeProperty {
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
unsafe fn find_own_property(
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
unsafe fn js_autoinit_get_realm(pr: *mut JSProperty) -> *mut JSContext {
    ((*pr).u.init.realm_and_id & !3) as *mut JSContext
}
unsafe fn js_autoinit_get_id(pr: *mut JSProperty) -> JSAutoInitIDEnum {
    ((*pr).u.init.realm_and_id & 3) as u32
}
unsafe fn set_cycle_flag(_ctx: *mut JSContext, _obj: JSValueConst) {} // Original function body is empty.
pub unsafe fn JS_DupContext(ctx: *mut JSContext) -> *mut JSContext {
    (*js_rc(ctx.cast())).ref_count += 1;
    ctx
}
pub unsafe fn JS_IsJobPending(rt: *mut JSRuntime) -> JS_BOOL {
    (list_empty(&mut (*rt).job_list) == 0) as i32
}
