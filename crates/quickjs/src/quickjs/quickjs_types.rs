// quickjs.c runtime/context, GC, bigint and string layouts. MIT.
use super::*;
pub(crate) type JSGCPhaseEnum = u32;
pub(crate) const JS_GC_PHASE_NONE: JSGCPhaseEnum = 0;
pub(crate) const JS_GC_PHASE_DECREF: JSGCPhaseEnum = 1;
pub(crate) const JS_GC_PHASE_REMOVE_CYCLES: JSGCPhaseEnum = 2;
pub(crate) const JS_MODE_STRICT: i32 = 1 << 0;
pub(crate) const JS_MODE_ASYNC: i32 = 1 << 2;
pub(crate) const JS_MODE_BACKTRACE_BARRIER: i32 = 1 << 3;
pub(crate) type JSErrorEnum = i32;
pub(crate) const JS_EVAL_ERROR: JSErrorEnum = 0;
pub(crate) const JS_RANGE_ERROR: JSErrorEnum = 1;
pub(crate) const JS_REFERENCE_ERROR: JSErrorEnum = 2;
pub(crate) const JS_SYNTAX_ERROR: JSErrorEnum = 3;
pub(crate) const JS_TYPE_ERROR: JSErrorEnum = 4;
pub(crate) const JS_URI_ERROR: JSErrorEnum = 5;
pub(crate) const JS_INTERNAL_ERROR: JSErrorEnum = 6;
pub(crate) const JS_AGGREGATE_ERROR: JSErrorEnum = 7;
pub(crate) const JS_NATIVE_ERROR_COUNT: usize = 8;
#[repr(C)]
pub(crate) union JSModuleLoaderUnion {
    pub(crate) module_loader_func: Option<JSModuleLoaderFunc>,
    pub(crate) module_loader_func2: Option<JSModuleLoaderFunc2>,
}
#[repr(C)]
pub struct JSRuntime {
    pub(crate) continuation_data_slot: *const std::cell::RefCell<JSContinuationData>,
    pub(crate) malloc_ctx: JSMallocContext,
    pub(crate) rt_info: *const c_char,
    pub(crate) atom_hash_size: i32,
    pub(crate) atom_count: i32,
    pub(crate) atom_size: i32,
    pub(crate) atom_count_resize: i32,
    pub(crate) atom_hash: *mut u32,
    pub(crate) atom_array: *mut *mut JSAtomStruct,
    pub(crate) atom_free_index: i32,
    pub(crate) class_count: i32,
    pub(crate) class_array: *mut JSClass,
    pub(crate) context_list: list_head,
    pub(crate) gc_obj_list: list_head,
    pub(crate) gc_zero_ref_count_list: list_head,
    pub(crate) tmp_obj_list: list_head,
    pub(crate) gc_phase: u8,
    pub(crate) malloc_gc_threshold: usize,
    pub(crate) weakref_list: list_head,
    pub(crate) stack_size: usize,
    pub(crate) stack_top: usize,
    pub(crate) stack_limit: usize,
    pub(crate) current_exception: JSValue,
    pub(crate) current_exception_is_uncatchable: i8,
    pub(crate) in_out_of_memory: i8,
    pub(crate) current_stack_frame: *mut JSStackFrame,
    pub(crate) interrupt_handler: Option<JSInterruptHandler>,
    pub(crate) interrupt_opaque: *mut c_void,
    // Embedder-only native regexp limit; zero retains official QuickJS semantics.
    pub(crate) regexp_backtrack_limit: u32,
    pub(crate) host_promise_rejection_tracker: Option<JSHostPromiseRejectionTracker>,
    pub(crate) host_promise_rejection_tracker_opaque: *mut c_void,
    pub(crate) job_list: list_head,
    pub(crate) module_normalize_func: Option<JSModuleNormalizeFunc>,
    pub(crate) module_loader_has_attr: JS_BOOL,
    pub(crate) u: JSModuleLoaderUnion,
    pub(crate) module_check_attrs: Option<JSModuleCheckSupportedImportAttributes>,
    pub(crate) module_loader_opaque: *mut c_void,
    pub(crate) module_async_evaluation_next_timestamp: i64,
    pub(crate) can_block: i8,
    pub(crate) sab_funcs: JSSharedArrayBufferFunctions,
    pub(crate) strip_flags: u8,
    pub(crate) shape_hash_bits: i32,
    pub(crate) shape_hash_size: i32,
    pub(crate) shape_hash_count: i32,
    pub(crate) shape_hash: *mut *mut JSShape,
    pub(crate) user_opaque: *mut c_void,
}
#[repr(C)]
pub struct JSClass {
    pub(crate) class_id: u32,
    pub(crate) class_name: JSAtom,
    pub(crate) finalizer: Option<JSClassFinalizer>,
    pub(crate) gc_mark: Option<JSClassGCMark>,
    pub(crate) call: Option<JSClassCall>,
    pub(crate) exotic: *const JSClassExoticMethods,
}
#[repr(C)]
pub(crate) struct JSStackFrame {
    pub(crate) prev_frame: *mut JSStackFrame,
    pub(crate) cur_func: JSValue,
    pub(crate) arg_buf: *mut JSValue,
    pub(crate) var_buf: *mut JSValue,
    pub(crate) var_refs: *mut *mut JSVarRef,
    pub(crate) cur_pc: *const u8,
    pub(crate) arg_count: i32,
    pub(crate) js_mode: i32,
    pub(crate) cur_sp: *mut JSValue,
}
pub(crate) type JSGCObjectTypeEnum = u32;
pub(crate) const JS_GC_OBJ_TYPE_JS_OBJECT: JSGCObjectTypeEnum = 0;
pub(crate) const JS_GC_OBJ_TYPE_FUNCTION_BYTECODE: JSGCObjectTypeEnum = 1;
pub(crate) const JS_GC_OBJ_TYPE_SHAPE: JSGCObjectTypeEnum = 2;
pub(crate) const JS_GC_OBJ_TYPE_VAR_REF: JSGCObjectTypeEnum = 3;
pub(crate) const JS_GC_OBJ_TYPE_ASYNC_FUNCTION: JSGCObjectTypeEnum = 4;
pub(crate) const JS_GC_OBJ_TYPE_JS_CONTEXT: JSGCObjectTypeEnum = 5;
pub(crate) const JS_GC_OBJ_TYPE_MODULE: JSGCObjectTypeEnum = 6;
#[repr(C)]
pub struct JSGCObjectHeader {
    pub(crate) link: list_head,
}
pub(crate) type JSWeakRefHeaderTypeEnum = u32;
pub(crate) const JS_WEAKREF_TYPE_MAP: JSWeakRefHeaderTypeEnum = 0;
pub(crate) const JS_WEAKREF_TYPE_WEAKREF: JSWeakRefHeaderTypeEnum = 1;
pub(crate) const JS_WEAKREF_TYPE_FINREC: JSWeakRefHeaderTypeEnum = 2;
#[repr(C)]
pub(crate) struct JSWeakRefHeader {
    pub(crate) link: list_head,
    pub(crate) weakref_type: JSWeakRefHeaderTypeEnum,
}
#[repr(C)]
#[derive(Clone, Copy)]
pub(crate) struct JSVarRefAttached {
    pub(crate) var_ref_idx: u16,
    pub(crate) stack_frame: *mut JSStackFrame,
}
#[repr(C)]
pub(crate) union JSVarRefUnion {
    pub(crate) value: JSValue,
    pub(crate) attached: JSVarRefAttached,
}
#[repr(C)]
pub(crate) struct JSVarRef {
    pub(crate) header: JSGCObjectHeader,
    pub(crate) is_detached: u8,
    pub(crate) is_lexical: u8,
    pub(crate) is_const: u8,
    pub(crate) pvalue: *mut JSValue,
    pub(crate) u: JSVarRefUnion,
}
#[cfg(target_pointer_width = "64")]
pub(crate) type js_slimb_t = i64;
#[cfg(target_pointer_width = "64")]
pub(crate) type js_limb_t = u64;
#[cfg(target_pointer_width = "64")]
pub(crate) type js_sdlimb_t = i128;
#[cfg(target_pointer_width = "64")]
pub(crate) type js_dlimb_t = u128;
#[cfg(target_pointer_width = "64")]
pub(crate) const JS_LIMB_DIGITS: i32 = 19;
#[cfg(target_pointer_width = "32")]
pub(crate) type js_slimb_t = i32;
#[cfg(target_pointer_width = "32")]
pub(crate) type js_limb_t = u32;
#[cfg(target_pointer_width = "32")]
pub(crate) type js_sdlimb_t = i64;
#[cfg(target_pointer_width = "32")]
pub(crate) type js_dlimb_t = u64;
#[cfg(target_pointer_width = "32")]
pub(crate) const JS_LIMB_DIGITS: i32 = 9;
#[repr(C)]
pub(crate) struct JSBigInt {
    pub(crate) len: u32,
    pub(crate) tab: [js_limb_t; 0],
}
#[repr(C)]
pub(crate) struct JSBigIntBuf {
    pub(crate) big_int_buf: [js_limb_t; size_of::<JSBigInt>() / size_of::<js_limb_t>()],
    pub(crate) tab: [js_limb_t; (64 + JS_LIMB_BITS - 1) / JS_LIMB_BITS],
}
pub(crate) type JSAutoInitIDEnum = u32;
pub(crate) const JS_AUTOINIT_ID_PROTOTYPE: JSAutoInitIDEnum = 0;
pub(crate) const JS_AUTOINIT_ID_MODULE_NS: JSAutoInitIDEnum = 1;
pub(crate) const JS_AUTOINIT_ID_PROP: JSAutoInitIDEnum = 2;
pub(crate) const JS_INTERRUPT_COUNTER_INIT: i32 = 10000;
#[repr(C)]
pub struct JSContext {
    pub(crate) header: JSGCObjectHeader,
    pub(crate) rt: *mut JSRuntime,
    pub(crate) link: list_head,
    pub(crate) binary_object_count: u16,
    pub(crate) binary_object_size: i32,
    pub(crate) array_shape: *mut JSShape,
    pub(crate) arguments_shape: *mut JSShape,
    pub(crate) mapped_arguments_shape: *mut JSShape,
    pub(crate) regexp_shape: *mut JSShape,
    pub(crate) regexp_result_shape: *mut JSShape,
    pub(crate) class_proto: *mut JSValue,
    pub(crate) function_proto: JSValue,
    pub(crate) function_ctor: JSValue,
    pub(crate) array_ctor: JSValue,
    pub(crate) regexp_ctor: JSValue,
    pub(crate) promise_ctor: JSValue,
    pub(crate) error_proto: [JSValue; JS_NATIVE_ERROR_COUNT],
    pub(crate) iterator_ctor: JSValue,
    pub(crate) async_iterator_proto: JSValue,
    pub(crate) array_proto_values: JSValue,
    pub(crate) throw_type_error: JSValue,
    pub(crate) eval_obj: JSValue,
    pub(crate) global_obj: JSValue,
    pub(crate) global_var_obj: JSValue,
    pub(crate) random_state: u64,
    pub(crate) interrupt_counter: i32,
    pub(crate) loaded_modules: list_head,
    pub(crate) compile_regexp: Option<unsafe fn(*mut JSContext, JSValueConst, JSValueConst) -> JSValue>,
    pub(crate) eval_internal: Option<
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
    pub(crate) user_opaque: *mut c_void,
}
#[repr(C)]
pub(crate) union JSFloat64Union {
    pub(crate) d: f64,
    pub(crate) u64: u64,
    pub(crate) u32: [u32; 2],
}
pub(crate) const JS_ATOM_TYPE_STRING: u32 = 1;
pub(crate) const JS_ATOM_TYPE_GLOBAL_SYMBOL: u32 = 2;
pub(crate) const JS_ATOM_TYPE_SYMBOL: u32 = 3;
pub(crate) const JS_ATOM_TYPE_PRIVATE: u32 = 4;
pub(crate) type JSAtomKindEnum = u32;
pub(crate) const JS_ATOM_KIND_STRING: JSAtomKindEnum = 0;
pub(crate) const JS_ATOM_KIND_SYMBOL: JSAtomKindEnum = 1;
pub(crate) const JS_ATOM_KIND_PRIVATE: JSAtomKindEnum = 2;
pub(crate) const JS_ATOM_HASH_MASK: u32 = (1 << 30) - 1;
pub(crate) const JS_ATOM_HASH_PRIVATE: u32 = JS_ATOM_HASH_MASK;
#[repr(C)]
pub(crate) union JSStringUnion {
    pub(crate) str8: [u8; 0],
    pub(crate) str16: [u16; 0],
}
#[repr(C)]
pub(crate) struct JSString {
    pub(crate) len_and_wide: u32,
    pub(crate) hash_and_type: u32,
    pub(crate) hash_next: u32,
    pub(crate) u: JSStringUnion,
}
impl JSString {
    pub(crate) fn len(&self) -> u32 {
        self.len_and_wide & 0x7fffffff
    }
    pub(crate) fn set_len(&mut self, n: u32) {
        self.len_and_wide = (self.len_and_wide & 0x80000000) | (n & 0x7fffffff);
    }
    pub(crate) fn is_wide_char(&self) -> u32 {
        self.len_and_wide >> 31
    }
    pub(crate) fn set_is_wide_char(&mut self, n: u32) {
        self.len_and_wide = (self.len_and_wide & 0x7fffffff) | (n << 31);
    }
    pub(crate) fn hash(&self) -> u32 {
        self.hash_and_type & JS_ATOM_HASH_MASK
    }
    pub(crate) fn set_hash(&mut self, n: u32) {
        self.hash_and_type = (self.hash_and_type & !JS_ATOM_HASH_MASK) | (n & JS_ATOM_HASH_MASK);
    }
    pub(crate) fn atom_type(&self) -> u32 {
        self.hash_and_type >> 30
    }
    pub(crate) fn set_atom_type(&mut self, n: u32) {
        self.hash_and_type = (self.hash_and_type & JS_ATOM_HASH_MASK) | (n << 30);
    }
}
pub(crate) type JSAtomStruct = JSString;
#[repr(C)]
pub(crate) struct JSStringRope {
    pub(crate) len: u32,
    pub(crate) is_wide_char: u8,
    pub(crate) depth: u8,
    pub(crate) left: JSValue,
    pub(crate) right: JSValue,
}
pub(crate) unsafe fn string_data8(p: *mut JSString) -> *mut u8 {
    ptr::addr_of_mut!((*p).u.str8).cast()
}
pub(crate) unsafe fn string_data16(p: *mut JSString) -> *mut u16 {
    ptr::addr_of_mut!((*p).u.str16).cast()
}
