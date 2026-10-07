// quickjs.c runtime/context, GC, bigint and string layouts. MIT.
type JSGCPhaseEnum = u32;
const JS_GC_PHASE_NONE: JSGCPhaseEnum = 0;
const JS_GC_PHASE_DECREF: JSGCPhaseEnum = 1;
const JS_GC_PHASE_REMOVE_CYCLES: JSGCPhaseEnum = 2;
const JS_MODE_STRICT: i32 = 1 << 0;
const JS_MODE_ASYNC: i32 = 1 << 2;
const JS_MODE_BACKTRACE_BARRIER: i32 = 1 << 3;
type JSErrorEnum = i32;
const JS_EVAL_ERROR: JSErrorEnum = 0;
const JS_RANGE_ERROR: JSErrorEnum = 1;
const JS_REFERENCE_ERROR: JSErrorEnum = 2;
const JS_SYNTAX_ERROR: JSErrorEnum = 3;
const JS_TYPE_ERROR: JSErrorEnum = 4;
const JS_URI_ERROR: JSErrorEnum = 5;
const JS_INTERNAL_ERROR: JSErrorEnum = 6;
const JS_AGGREGATE_ERROR: JSErrorEnum = 7;
const JS_NATIVE_ERROR_COUNT: usize = 8;
#[repr(C)]
union JSModuleLoaderUnion {
    module_loader_func: Option<JSModuleLoaderFunc>,
    module_loader_func2: Option<JSModuleLoaderFunc2>,
}
#[repr(C)]
pub struct JSRuntime {
    malloc_ctx: JSMallocContext,
    rt_info: *const c_char,
    atom_hash_size: i32,
    atom_count: i32,
    atom_size: i32,
    atom_count_resize: i32,
    atom_hash: *mut u32,
    atom_array: *mut *mut JSAtomStruct,
    atom_free_index: i32,
    class_count: i32,
    class_array: *mut JSClass,
    context_list: list_head,
    gc_obj_list: list_head,
    gc_zero_ref_count_list: list_head,
    tmp_obj_list: list_head,
    gc_phase: u8,
    malloc_gc_threshold: usize,
    weakref_list: list_head,
    stack_size: usize,
    stack_top: usize,
    stack_limit: usize,
    current_exception: JSValue,
    current_exception_is_uncatchable: i8,
    in_out_of_memory: i8,
    current_stack_frame: *mut JSStackFrame,
    interrupt_handler: Option<JSInterruptHandler>,
    interrupt_opaque: *mut c_void,
    host_promise_rejection_tracker: Option<JSHostPromiseRejectionTracker>,
    host_promise_rejection_tracker_opaque: *mut c_void,
    job_list: list_head,
    module_normalize_func: Option<JSModuleNormalizeFunc>,
    module_loader_has_attr: JS_BOOL,
    u: JSModuleLoaderUnion,
    module_check_attrs: Option<JSModuleCheckSupportedImportAttributes>,
    module_loader_opaque: *mut c_void,
    module_async_evaluation_next_timestamp: i64,
    can_block: i8,
    sab_funcs: JSSharedArrayBufferFunctions,
    strip_flags: u8,
    shape_hash_bits: i32,
    shape_hash_size: i32,
    shape_hash_count: i32,
    shape_hash: *mut *mut JSShape,
    user_opaque: *mut c_void,
}
#[repr(C)]
pub struct JSClass {
    class_id: u32,
    class_name: JSAtom,
    finalizer: Option<JSClassFinalizer>,
    gc_mark: Option<JSClassGCMark>,
    call: Option<JSClassCall>,
    exotic: *const JSClassExoticMethods,
}
#[repr(C)]
struct JSStackFrame {
    prev_frame: *mut JSStackFrame,
    cur_func: JSValue,
    arg_buf: *mut JSValue,
    var_buf: *mut JSValue,
    var_refs: *mut *mut JSVarRef,
    cur_pc: *const u8,
    arg_count: i32,
    js_mode: i32,
    cur_sp: *mut JSValue,
}
type JSGCObjectTypeEnum = u32;
const JS_GC_OBJ_TYPE_JS_OBJECT: JSGCObjectTypeEnum = 0;
const JS_GC_OBJ_TYPE_FUNCTION_BYTECODE: JSGCObjectTypeEnum = 1;
const JS_GC_OBJ_TYPE_SHAPE: JSGCObjectTypeEnum = 2;
const JS_GC_OBJ_TYPE_VAR_REF: JSGCObjectTypeEnum = 3;
const JS_GC_OBJ_TYPE_ASYNC_FUNCTION: JSGCObjectTypeEnum = 4;
const JS_GC_OBJ_TYPE_JS_CONTEXT: JSGCObjectTypeEnum = 5;
const JS_GC_OBJ_TYPE_MODULE: JSGCObjectTypeEnum = 6;
#[repr(C)]
pub struct JSGCObjectHeader {
    link: list_head,
}
type JSWeakRefHeaderTypeEnum = u32;
const JS_WEAKREF_TYPE_MAP: JSWeakRefHeaderTypeEnum = 0;
const JS_WEAKREF_TYPE_WEAKREF: JSWeakRefHeaderTypeEnum = 1;
const JS_WEAKREF_TYPE_FINREC: JSWeakRefHeaderTypeEnum = 2;
#[repr(C)]
struct JSWeakRefHeader {
    link: list_head,
    weakref_type: JSWeakRefHeaderTypeEnum,
}
#[repr(C)]
#[derive(Clone, Copy)]
struct JSVarRefAttached {
    var_ref_idx: u16,
    stack_frame: *mut JSStackFrame,
}
#[repr(C)]
union JSVarRefUnion {
    value: JSValue,
    attached: JSVarRefAttached,
}
#[repr(C)]
struct JSVarRef {
    header: JSGCObjectHeader,
    is_detached: u8,
    is_lexical: u8,
    is_const: u8,
    pvalue: *mut JSValue,
    u: JSVarRefUnion,
}
#[cfg(target_pointer_width = "64")]
type js_slimb_t = i64;
#[cfg(target_pointer_width = "64")]
type js_limb_t = u64;
#[cfg(target_pointer_width = "64")]
type js_sdlimb_t = i128;
#[cfg(target_pointer_width = "64")]
type js_dlimb_t = u128;
#[cfg(target_pointer_width = "64")]
const JS_LIMB_DIGITS: i32 = 19;
#[cfg(target_pointer_width = "32")]
type js_slimb_t = i32;
#[cfg(target_pointer_width = "32")]
type js_limb_t = u32;
#[cfg(target_pointer_width = "32")]
type js_sdlimb_t = i64;
#[cfg(target_pointer_width = "32")]
type js_dlimb_t = u64;
#[cfg(target_pointer_width = "32")]
const JS_LIMB_DIGITS: i32 = 9;
#[repr(C)]
struct JSBigInt {
    len: u32,
    tab: [js_limb_t; 0],
}
#[repr(C)]
struct JSBigIntBuf {
    big_int_buf: [js_limb_t; size_of::<JSBigInt>() / size_of::<js_limb_t>()],
    tab: [js_limb_t; (64 + JS_LIMB_BITS - 1) / JS_LIMB_BITS],
}
type JSAutoInitIDEnum = u32;
const JS_AUTOINIT_ID_PROTOTYPE: JSAutoInitIDEnum = 0;
const JS_AUTOINIT_ID_MODULE_NS: JSAutoInitIDEnum = 1;
const JS_AUTOINIT_ID_PROP: JSAutoInitIDEnum = 2;
const JS_INTERRUPT_COUNTER_INIT: i32 = 10000;
#[repr(C)]
pub struct JSContext {
    header: JSGCObjectHeader,
    rt: *mut JSRuntime,
    link: list_head,
    binary_object_count: u16,
    binary_object_size: i32,
    array_shape: *mut JSShape,
    arguments_shape: *mut JSShape,
    mapped_arguments_shape: *mut JSShape,
    regexp_shape: *mut JSShape,
    regexp_result_shape: *mut JSShape,
    class_proto: *mut JSValue,
    function_proto: JSValue,
    function_ctor: JSValue,
    array_ctor: JSValue,
    regexp_ctor: JSValue,
    promise_ctor: JSValue,
    native_error_proto: [JSValue; JS_NATIVE_ERROR_COUNT],
    iterator_ctor: JSValue,
    async_iterator_proto: JSValue,
    array_proto_values: JSValue,
    throw_type_error: JSValue,
    eval_obj: JSValue,
    global_obj: JSValue,
    global_var_obj: JSValue,
    random_state: u64,
    interrupt_counter: i32,
    loaded_modules: list_head,
    compile_regexp: Option<unsafe fn(*mut JSContext, JSValueConst, JSValueConst) -> JSValue>,
    eval_internal: Option<
        unsafe fn(
            *mut JSContext,
            JSValueConst,
            *const c_char,
            usize,
            *const c_char,
            i32,
            i32,
        ) -> JSValue,
    >,
    user_opaque: *mut c_void,
}
#[repr(C)]
union JSFloat64Union {
    d: f64,
    u64: u64,
    u32: [u32; 2],
}
const JS_ATOM_TYPE_STRING: u32 = 1;
const JS_ATOM_TYPE_GLOBAL_SYMBOL: u32 = 2;
const JS_ATOM_TYPE_SYMBOL: u32 = 3;
const JS_ATOM_TYPE_PRIVATE: u32 = 4;
type JSAtomKindEnum = u32;
const JS_ATOM_KIND_STRING: JSAtomKindEnum = 0;
const JS_ATOM_KIND_SYMBOL: JSAtomKindEnum = 1;
const JS_ATOM_KIND_PRIVATE: JSAtomKindEnum = 2;
const JS_ATOM_HASH_MASK: u32 = (1 << 30) - 1;
const JS_ATOM_HASH_PRIVATE: u32 = JS_ATOM_HASH_MASK;
#[repr(C)]
union JSStringUnion {
    str8: [u8; 0],
    str16: [u16; 0],
}
#[repr(C)]
struct JSString {
    len_and_wide: u32,
    hash_and_type: u32,
    hash_next: u32,
    u: JSStringUnion,
}
impl JSString {
    fn len(&self) -> u32 {
        self.len_and_wide & 0x7fffffff
    }
    fn set_len(&mut self, n: u32) {
        self.len_and_wide = (self.len_and_wide & 0x80000000) | (n & 0x7fffffff);
    }
    fn is_wide_char(&self) -> u32 {
        self.len_and_wide >> 31
    }
    fn set_is_wide_char(&mut self, n: u32) {
        self.len_and_wide = (self.len_and_wide & 0x7fffffff) | (n << 31);
    }
    fn hash(&self) -> u32 {
        self.hash_and_type & JS_ATOM_HASH_MASK
    }
    fn set_hash(&mut self, n: u32) {
        self.hash_and_type = (self.hash_and_type & !JS_ATOM_HASH_MASK) | (n & JS_ATOM_HASH_MASK);
    }
    fn atom_type(&self) -> u32 {
        self.hash_and_type >> 30
    }
    fn set_atom_type(&mut self, n: u32) {
        self.hash_and_type = (self.hash_and_type & JS_ATOM_HASH_MASK) | (n << 30);
    }
}
type JSAtomStruct = JSString;
#[repr(C)]
struct JSStringRope {
    len: u32,
    is_wide_char: u8,
    depth: u8,
    left: JSValue,
    right: JSValue,
}
unsafe fn string_data8(p: *mut JSString) -> *mut u8 {
    ptr::addr_of_mut!((*p).u.str8).cast()
}
unsafe fn string_data16(p: *mut JSString) -> *mut u16 {
    ptr::addr_of_mut!((*p).u.str16).cast()
}
