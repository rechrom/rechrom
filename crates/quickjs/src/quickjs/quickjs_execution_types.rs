// quickjs.c closure, bytecode, module and job layouts, C lines 612..945. MIT.
use super::*;
pub(crate) type JSClosureTypeEnum = u32;
pub(crate) const JS_CLOSURE_LOCAL: JSClosureTypeEnum = 0;
pub(crate) const JS_CLOSURE_ARG: JSClosureTypeEnum = 1;
pub(crate) const JS_CLOSURE_REF: JSClosureTypeEnum = 2;
pub(crate) const JS_CLOSURE_GLOBAL_REF: JSClosureTypeEnum = 3;
pub(crate) const JS_CLOSURE_GLOBAL_DECL: JSClosureTypeEnum = 4;
pub(crate) const JS_CLOSURE_GLOBAL: JSClosureTypeEnum = 5;
pub(crate) const JS_CLOSURE_MODULE_DECL: JSClosureTypeEnum = 6;
pub(crate) const JS_CLOSURE_MODULE_IMPORT: JSClosureTypeEnum = 7;
#[repr(C)]
pub(crate) struct JSClosureVar {
    pub(crate) closure_flags: u8,
    pub(crate) var_kind_storage: u8,
    pub(crate) var_idx: u16,
    pub(crate) var_name: JSAtom,
}
impl JSClosureVar {
    pub(crate) fn closure_type(&self) -> JSClosureTypeEnum {
        (self.closure_flags & 7) as u32
    }
    pub(crate) fn set_closure_type(&mut self, n: JSClosureTypeEnum) {
        self.closure_flags = (self.closure_flags & !7) | (n as u8 & 7);
    }
    pub(crate) fn is_lexical(&self) -> u8 {
        (self.closure_flags >> 3) & 1
    }
    pub(crate) fn set_is_lexical(&mut self, n: u8) {
        self.closure_flags = (self.closure_flags & !(1 << 3)) | ((n & 1) << 3);
    }
    pub(crate) fn is_const(&self) -> u8 {
        (self.closure_flags >> 4) & 1
    }
    pub(crate) fn set_is_const(&mut self, n: u8) {
        self.closure_flags = (self.closure_flags & !(1 << 4)) | ((n & 1) << 4);
    }
    pub(crate) fn var_kind(&self) -> u8 {
        self.var_kind_storage & 15
    }
    pub(crate) fn set_var_kind(&mut self, n: u8) {
        self.var_kind_storage = (self.var_kind_storage & !15) | (n & 15);
    }
}
pub(crate) const ARG_SCOPE_INDEX: i32 = 1;
pub(crate) const ARG_SCOPE_END: i32 = -2;
pub(crate) type JSVarKindEnum = u32;
pub(crate) const JS_VAR_NORMAL: JSVarKindEnum = 0;
pub(crate) const JS_VAR_FUNCTION_DECL: JSVarKindEnum = 1;
pub(crate) const JS_VAR_NEW_FUNCTION_DECL: JSVarKindEnum = 2;
pub(crate) const JS_VAR_CATCH: JSVarKindEnum = 3;
pub(crate) const JS_VAR_FUNCTION_NAME: JSVarKindEnum = 4;
pub(crate) const JS_VAR_PRIVATE_FIELD: JSVarKindEnum = 5;
pub(crate) const JS_VAR_PRIVATE_METHOD: JSVarKindEnum = 6;
pub(crate) const JS_VAR_PRIVATE_GETTER: JSVarKindEnum = 7;
pub(crate) const JS_VAR_PRIVATE_SETTER: JSVarKindEnum = 8;
pub(crate) const JS_VAR_PRIVATE_GETTER_SETTER: JSVarKindEnum = 9;
pub(crate) const JS_VAR_GLOBAL_FUNCTION_DECL: JSVarKindEnum = 10;
#[repr(C)]
pub(crate) struct JSBytecodeVarDef {
    pub(crate) var_name: JSAtom,
    pub(crate) scope_next: i32,
    pub(crate) var_flags: u8,
    pub(crate) var_ref_idx: u16,
}
impl JSBytecodeVarDef {
    pub(crate) fn is_const(&self) -> u8 {
        self.var_flags & 1
    }
    pub(crate) fn set_is_const(&mut self, n: u8) {
        self.var_flags = (self.var_flags & !1) | (n & 1);
    }
    pub(crate) fn is_lexical(&self) -> u8 {
        (self.var_flags >> 1) & 1
    }
    pub(crate) fn set_is_lexical(&mut self, n: u8) {
        self.var_flags = (self.var_flags & !2) | ((n & 1) << 1);
    }
    pub(crate) fn is_captured(&self) -> u8 {
        (self.var_flags >> 2) & 1
    }
    pub(crate) fn set_is_captured(&mut self, n: u8) {
        self.var_flags = (self.var_flags & !4) | ((n & 1) << 2);
    }
    pub(crate) fn has_scope(&self) -> u8 {
        (self.var_flags >> 3) & 1
    }
    pub(crate) fn set_has_scope(&mut self, n: u8) {
        self.var_flags = (self.var_flags & !8) | ((n & 1) << 3);
    }
    pub(crate) fn var_kind(&self) -> u8 {
        self.var_flags >> 4
    }
    pub(crate) fn set_var_kind(&mut self, n: u8) {
        self.var_flags = (self.var_flags & 15) | (n << 4);
    }
}
pub(crate) const PC2LINE_BASE: i32 = -1;
pub(crate) const PC2LINE_RANGE: i32 = 5;
pub(crate) const PC2LINE_OP_FIRST: i32 = 1;
pub(crate) const PC2LINE_DIFF_PC_MAX: i32 = (255 - PC2LINE_OP_FIRST) / PC2LINE_RANGE;
pub(crate) type JSFunctionKindEnum = u32;
pub(crate) const JS_FUNC_NORMAL: JSFunctionKindEnum = 0;
pub(crate) const JS_FUNC_GENERATOR: JSFunctionKindEnum = 1;
pub(crate) const JS_FUNC_ASYNC: JSFunctionKindEnum = 2;
pub(crate) const JS_FUNC_ASYNC_GENERATOR: JSFunctionKindEnum = 3;
#[repr(C)]
pub(crate) struct JSFunctionDebug {
    pub(crate) filename: JSAtom,
    pub(crate) source_len: i32,
    pub(crate) pc2line_len: i32,
    pub(crate) pc2line_buf: *mut u8,
    pub(crate) source: *mut c_char,
}
#[repr(C)]
pub(crate) struct JSFunctionBytecode {
    pub(crate) header: JSGCObjectHeader,
    pub(crate) js_mode: u8,
    pub(crate) function_flags: [u8; 2],
    pub(crate) byte_code_buf: *mut u8,
    pub(crate) byte_code_len: i32,
    pub(crate) func_name: JSAtom,
    pub(crate) vardefs: *mut JSBytecodeVarDef,
    pub(crate) closure_var: *mut JSClosureVar,
    pub(crate) arg_count: u16,
    pub(crate) var_count: u16,
    pub(crate) defined_arg_count: u16,
    pub(crate) stack_size: u16,
    pub(crate) var_ref_count: u16,
    pub(crate) realm: *mut JSContext,
    pub(crate) cpool: *mut JSValue,
    pub(crate) cpool_count: i32,
    pub(crate) closure_var_count: i32,
    pub(crate) debug: JSFunctionDebug,
}
// Bitfields follow the original uint8_t allocation units (8 + 5 bits).
macro_rules! function_flag {
    ($get:ident,$set:ident,$byte:expr,$shift:expr,$mask:expr) => {
        pub(crate) fn $get(&self) -> u8 {
            (self.function_flags[$byte] >> $shift) & $mask
        }
        pub(crate) fn $set(&mut self, n: u8) {
            self.function_flags[$byte] =
                (self.function_flags[$byte] & !($mask << $shift)) | ((n & $mask) << $shift);
        }
    };
}
impl JSFunctionBytecode {
    function_flag!(has_prototype, set_has_prototype, 0, 0, 1);
    function_flag!(
        has_simple_parameter_list,
        set_has_simple_parameter_list,
        0,
        1,
        1
    );
    function_flag!(
        is_derived_class_constructor,
        set_is_derived_class_constructor,
        0,
        2,
        1
    );
    function_flag!(need_home_object, set_need_home_object, 0, 3, 1);
    function_flag!(func_kind, set_func_kind, 0, 4, 3);
    function_flag!(new_target_allowed, set_new_target_allowed, 0, 6, 1);
    function_flag!(super_call_allowed, set_super_call_allowed, 0, 7, 1);
    function_flag!(super_allowed, set_super_allowed, 1, 0, 1);
    function_flag!(arguments_allowed, set_arguments_allowed, 1, 1, 1);
    function_flag!(has_debug, set_has_debug, 1, 2, 1);
    function_flag!(read_only_bytecode, set_read_only_bytecode, 1, 3, 1);
    function_flag!(
        is_direct_or_indirect_eval,
        set_is_direct_or_indirect_eval,
        1,
        4,
        1
    );
}
#[repr(C)]
pub(crate) struct JSBoundFunction {
    pub(crate) func_obj: JSValue,
    pub(crate) this_val: JSValue,
    pub(crate) argc: i32,
    pub(crate) argv: [JSValue; 0],
}
pub(crate) type JSIteratorKindEnum = u32;
pub(crate) const JS_ITERATOR_KIND_KEY: JSIteratorKindEnum = 0;
pub(crate) const JS_ITERATOR_KIND_VALUE: JSIteratorKindEnum = 1;
pub(crate) const JS_ITERATOR_KIND_KEY_AND_VALUE: JSIteratorKindEnum = 2;
#[repr(C)]
pub(crate) struct JSForInIterator {
    pub(crate) obj: JSValue,
    pub(crate) idx: u32,
    pub(crate) atom_count: u32,
    pub(crate) in_prototype_chain: u8,
    pub(crate) is_array: u8,
    pub(crate) tab_atom: *mut JSPropertyEnum,
}
#[repr(C)]
#[derive(Clone, Copy)]
pub(crate) struct JSRegExp {
    pub(crate) pattern: *mut JSString,
    pub(crate) bytecode: *mut JSString,
}
#[repr(C)]
pub(crate) struct JSProxyData {
    pub(crate) target: JSValue,
    pub(crate) handler: JSValue,
    pub(crate) is_func: u8,
    pub(crate) is_revoked: u8,
}
#[repr(C)]
pub(crate) struct JSArrayBuffer {
    pub(crate) byte_length: i32,
    pub(crate) max_byte_length: i32,
    pub(crate) detached: u8,
    pub(crate) shared: u8,
    pub(crate) data: *mut u8,
    pub(crate) array_list: list_head,
    pub(crate) opaque: *mut c_void,
    pub(crate) free_func: Option<JSFreeArrayBufferDataFunc>,
}
#[repr(C)]
pub(crate) struct JSTypedArray {
    pub(crate) link: list_head,
    pub(crate) obj: *mut JSObject,
    pub(crate) buffer: *mut JSObject,
    pub(crate) offset: u32,
    pub(crate) length: u32,
    pub(crate) track_rab: JS_BOOL,
}
#[repr(C)]
#[derive(Clone, Copy)]
pub(crate) struct JSGlobalObject {
    pub(crate) uninitialized_vars: JSValue,
}
#[repr(C)]
pub(crate) struct JSAsyncFunctionState {
    pub(crate) header: JSGCObjectHeader,
    pub(crate) this_val: JSValue,
    pub(crate) argc: i32,
    pub(crate) throw_flag: JS_BOOL,
    pub(crate) is_completed: JS_BOOL,
    pub(crate) resolving_funcs: [JSValue; 2],
    pub(crate) frame: JSStackFrame,
}
pub(crate) type JSOverloadableOperatorEnum = u32;
pub(crate) const JS_OVOP_ADD: JSOverloadableOperatorEnum = 0;
pub(crate) const JS_OVOP_SUB: JSOverloadableOperatorEnum = 1;
pub(crate) const JS_OVOP_MUL: JSOverloadableOperatorEnum = 2;
pub(crate) const JS_OVOP_DIV: JSOverloadableOperatorEnum = 3;
pub(crate) const JS_OVOP_MOD: JSOverloadableOperatorEnum = 4;
pub(crate) const JS_OVOP_POW: JSOverloadableOperatorEnum = 5;
pub(crate) const JS_OVOP_OR: JSOverloadableOperatorEnum = 6;
pub(crate) const JS_OVOP_AND: JSOverloadableOperatorEnum = 7;
pub(crate) const JS_OVOP_XOR: JSOverloadableOperatorEnum = 8;
pub(crate) const JS_OVOP_SHL: JSOverloadableOperatorEnum = 9;
pub(crate) const JS_OVOP_SAR: JSOverloadableOperatorEnum = 10;
pub(crate) const JS_OVOP_SHR: JSOverloadableOperatorEnum = 11;
pub(crate) const JS_OVOP_EQ: JSOverloadableOperatorEnum = 12;
pub(crate) const JS_OVOP_LESS: JSOverloadableOperatorEnum = 13;
pub(crate) const JS_OVOP_BINARY_COUNT: usize = 14;
pub(crate) const JS_OVOP_POS: JSOverloadableOperatorEnum = 14;
pub(crate) const JS_OVOP_NEG: JSOverloadableOperatorEnum = 15;
pub(crate) const JS_OVOP_INC: JSOverloadableOperatorEnum = 16;
pub(crate) const JS_OVOP_DEC: JSOverloadableOperatorEnum = 17;
pub(crate) const JS_OVOP_NOT: JSOverloadableOperatorEnum = 18;
pub(crate) const JS_OVOP_COUNT: usize = 19;
#[repr(C)]
pub(crate) struct JSBinaryOperatorDefEntry {
    pub(crate) operator_index: u32,
    pub(crate) ops: [*mut JSObject; JS_OVOP_BINARY_COUNT],
}
#[repr(C)]
pub(crate) struct JSBinaryOperatorDef {
    pub(crate) count: i32,
    pub(crate) tab: *mut JSBinaryOperatorDefEntry,
}
#[repr(C)]
pub(crate) struct JSOperatorSetData {
    pub(crate) operator_counter: u32,
    pub(crate) is_primitive: JS_BOOL,
    pub(crate) self_ops: [*mut JSObject; JS_OVOP_COUNT],
    pub(crate) left: JSBinaryOperatorDef,
    pub(crate) right: JSBinaryOperatorDef,
}
#[repr(C)]
pub(crate) struct JSReqModuleEntry {
    pub(crate) module_name: JSAtom,
    pub(crate) module: *mut JSModuleDef,
    pub(crate) attributes: JSValue,
}
pub(crate) type JSExportTypeEnum = u32;
pub(crate) const JS_EXPORT_TYPE_LOCAL: JSExportTypeEnum = 0;
pub(crate) const JS_EXPORT_TYPE_INDIRECT: JSExportTypeEnum = 1;
#[repr(C)]
#[derive(Clone, Copy)]
pub(crate) struct JSLocalExport {
    pub(crate) var_idx: i32,
    pub(crate) var_ref: *mut JSVarRef,
}
#[repr(C)]
pub(crate) union JSExportUnion {
    pub(crate) local: JSLocalExport,
    pub(crate) req_module_idx: i32,
}
#[repr(C)]
pub(crate) struct JSExportEntry {
    pub(crate) u: JSExportUnion,
    pub(crate) export_type: JSExportTypeEnum,
    pub(crate) local_name: JSAtom,
    pub(crate) export_name: JSAtom,
}
#[repr(C)]
pub(crate) struct JSStarExportEntry {
    pub(crate) req_module_idx: i32,
}
#[repr(C)]
pub(crate) struct JSImportEntry {
    pub(crate) var_idx: i32,
    pub(crate) is_star: JS_BOOL,
    pub(crate) import_name: JSAtom,
    pub(crate) req_module_idx: i32,
}
pub(crate) type JSModuleStatus = u32;
pub(crate) const JS_MODULE_STATUS_UNLINKED: JSModuleStatus = 0;
pub(crate) const JS_MODULE_STATUS_LINKING: JSModuleStatus = 1;
pub(crate) const JS_MODULE_STATUS_LINKED: JSModuleStatus = 2;
pub(crate) const JS_MODULE_STATUS_EVALUATING: JSModuleStatus = 3;
pub(crate) const JS_MODULE_STATUS_EVALUATING_ASYNC: JSModuleStatus = 4;
pub(crate) const JS_MODULE_STATUS_EVALUATED: JSModuleStatus = 5;
#[repr(C)]
pub struct JSModuleDef {
    pub(crate) header: JSGCObjectHeader,
    pub(crate) module_name: JSAtom,
    pub(crate) link: list_head,
    pub(crate) req_module_entries: *mut JSReqModuleEntry,
    pub(crate) req_module_entries_count: i32,
    pub(crate) req_module_entries_size: i32,
    pub(crate) export_entries: *mut JSExportEntry,
    pub(crate) export_entries_count: i32,
    pub(crate) export_entries_size: i32,
    pub(crate) star_export_entries: *mut JSStarExportEntry,
    pub(crate) star_export_entries_count: i32,
    pub(crate) star_export_entries_size: i32,
    pub(crate) import_entries: *mut JSImportEntry,
    pub(crate) import_entries_count: i32,
    pub(crate) import_entries_size: i32,
    pub(crate) module_ns: JSValue,
    pub(crate) func_obj: JSValue,
    pub(crate) init_func: Option<JSModuleInitFunc>,
    pub(crate) has_tla: i8,
    pub(crate) resolved: i8,
    pub(crate) func_created: i8,
    pub(crate) status: u8,
    pub(crate) dfs_index: i32,
    pub(crate) dfs_ancestor_index: i32,
    pub(crate) stack_prev: *mut JSModuleDef,
    pub(crate) async_parent_modules: *mut *mut JSModuleDef,
    pub(crate) async_parent_modules_count: i32,
    pub(crate) async_parent_modules_size: i32,
    pub(crate) pending_async_dependencies: i32,
    pub(crate) async_evaluation: JS_BOOL,
    pub(crate) async_evaluation_timestamp: i64,
    pub(crate) cycle_root: *mut JSModuleDef,
    pub(crate) promise: JSValue,
    pub(crate) resolving_funcs: [JSValue; 2],
    pub(crate) eval_has_exception: i8,
    pub(crate) eval_exception: JSValue,
    pub(crate) meta_obj: JSValue,
    pub(crate) private_value: JSValue,
}
#[repr(C)]
pub(crate) struct JSJobEntry {
    pub(crate) continuation_data: *mut JSContinuationData,
    pub(crate) link: list_head,
    pub(crate) realm: *mut JSContext,
    pub(crate) job_func: Option<JSJobFunc>,
    pub(crate) argc: i32,
    pub(crate) argv: [JSValue; 0],
}
