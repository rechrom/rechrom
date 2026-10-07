// quickjs.c closure, bytecode, module and job layouts, C lines 612..945. MIT.
type JSClosureTypeEnum = u32;
const JS_CLOSURE_LOCAL: JSClosureTypeEnum = 0;
const JS_CLOSURE_ARG: JSClosureTypeEnum = 1;
const JS_CLOSURE_REF: JSClosureTypeEnum = 2;
const JS_CLOSURE_GLOBAL_REF: JSClosureTypeEnum = 3;
const JS_CLOSURE_GLOBAL_DECL: JSClosureTypeEnum = 4;
const JS_CLOSURE_GLOBAL: JSClosureTypeEnum = 5;
const JS_CLOSURE_MODULE_DECL: JSClosureTypeEnum = 6;
const JS_CLOSURE_MODULE_IMPORT: JSClosureTypeEnum = 7;
#[repr(C)]
struct JSClosureVar {
    closure_flags: u8,
    var_kind_storage: u8,
    var_idx: u16,
    var_name: JSAtom,
}
impl JSClosureVar {
    fn closure_type(&self) -> JSClosureTypeEnum {
        (self.closure_flags & 7) as u32
    }
    fn set_closure_type(&mut self, n: JSClosureTypeEnum) {
        self.closure_flags = (self.closure_flags & !7) | (n as u8 & 7);
    }
    fn is_lexical(&self) -> u8 {
        (self.closure_flags >> 3) & 1
    }
    fn set_is_lexical(&mut self, n: u8) {
        self.closure_flags = (self.closure_flags & !(1 << 3)) | ((n & 1) << 3);
    }
    fn is_const(&self) -> u8 {
        (self.closure_flags >> 4) & 1
    }
    fn set_is_const(&mut self, n: u8) {
        self.closure_flags = (self.closure_flags & !(1 << 4)) | ((n & 1) << 4);
    }
    fn var_kind(&self) -> u8 {
        self.var_kind_storage & 15
    }
    fn set_var_kind(&mut self, n: u8) {
        self.var_kind_storage = (self.var_kind_storage & !15) | (n & 15);
    }
}
const ARG_SCOPE_INDEX: i32 = 1;
const ARG_SCOPE_END: i32 = -2;
type JSVarKindEnum = u32;
const JS_VAR_NORMAL: JSVarKindEnum = 0;
const JS_VAR_FUNCTION_DECL: JSVarKindEnum = 1;
const JS_VAR_NEW_FUNCTION_DECL: JSVarKindEnum = 2;
const JS_VAR_CATCH: JSVarKindEnum = 3;
const JS_VAR_FUNCTION_NAME: JSVarKindEnum = 4;
const JS_VAR_PRIVATE_FIELD: JSVarKindEnum = 5;
const JS_VAR_PRIVATE_METHOD: JSVarKindEnum = 6;
const JS_VAR_PRIVATE_GETTER: JSVarKindEnum = 7;
const JS_VAR_PRIVATE_SETTER: JSVarKindEnum = 8;
const JS_VAR_PRIVATE_GETTER_SETTER: JSVarKindEnum = 9;
const JS_VAR_GLOBAL_FUNCTION_DECL: JSVarKindEnum = 10;
#[repr(C)]
struct JSBytecodeVarDef {
    var_name: JSAtom,
    scope_next: i32,
    var_flags: u8,
    var_ref_idx: u16,
}
impl JSBytecodeVarDef {
    fn is_const(&self) -> u8 {
        self.var_flags & 1
    }
    fn set_is_const(&mut self, n: u8) {
        self.var_flags = (self.var_flags & !1) | (n & 1);
    }
    fn is_lexical(&self) -> u8 {
        (self.var_flags >> 1) & 1
    }
    fn set_is_lexical(&mut self, n: u8) {
        self.var_flags = (self.var_flags & !2) | ((n & 1) << 1);
    }
    fn is_captured(&self) -> u8 {
        (self.var_flags >> 2) & 1
    }
    fn set_is_captured(&mut self, n: u8) {
        self.var_flags = (self.var_flags & !4) | ((n & 1) << 2);
    }
    fn has_scope(&self) -> u8 {
        (self.var_flags >> 3) & 1
    }
    fn set_has_scope(&mut self, n: u8) {
        self.var_flags = (self.var_flags & !8) | ((n & 1) << 3);
    }
    fn var_kind(&self) -> u8 {
        self.var_flags >> 4
    }
    fn set_var_kind(&mut self, n: u8) {
        self.var_flags = (self.var_flags & 15) | (n << 4);
    }
}
const PC2LINE_BASE: i32 = -1;
const PC2LINE_RANGE: i32 = 5;
const PC2LINE_OP_FIRST: i32 = 1;
const PC2LINE_DIFF_PC_MAX: i32 = (255 - PC2LINE_OP_FIRST) / PC2LINE_RANGE;
type JSFunctionKindEnum = u32;
const JS_FUNC_NORMAL: JSFunctionKindEnum = 0;
const JS_FUNC_GENERATOR: JSFunctionKindEnum = 1;
const JS_FUNC_ASYNC: JSFunctionKindEnum = 2;
const JS_FUNC_ASYNC_GENERATOR: JSFunctionKindEnum = 3;
#[repr(C)]
struct JSFunctionDebug {
    filename: JSAtom,
    source_len: i32,
    pc2line_len: i32,
    pc2line_buf: *mut u8,
    source: *mut c_char,
}
#[repr(C)]
struct JSFunctionBytecode {
    header: JSGCObjectHeader,
    js_mode: u8,
    function_flags: [u8; 2],
    byte_code_buf: *mut u8,
    byte_code_len: i32,
    func_name: JSAtom,
    vardefs: *mut JSBytecodeVarDef,
    closure_var: *mut JSClosureVar,
    arg_count: u16,
    var_count: u16,
    defined_arg_count: u16,
    stack_size: u16,
    var_ref_count: u16,
    realm: *mut JSContext,
    cpool: *mut JSValue,
    cpool_count: i32,
    closure_var_count: i32,
    debug: JSFunctionDebug,
}
// Bitfields follow the original uint8_t allocation units (8 + 5 bits).
macro_rules! function_flag {
    ($get:ident,$set:ident,$byte:expr,$shift:expr,$mask:expr) => {
        fn $get(&self) -> u8 {
            (self.function_flags[$byte] >> $shift) & $mask
        }
        fn $set(&mut self, n: u8) {
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
struct JSBoundFunction {
    func_obj: JSValue,
    this_val: JSValue,
    argc: i32,
    argv: [JSValue; 0],
}
type JSIteratorKindEnum = u32;
const JS_ITERATOR_KIND_KEY: JSIteratorKindEnum = 0;
const JS_ITERATOR_KIND_VALUE: JSIteratorKindEnum = 1;
const JS_ITERATOR_KIND_KEY_AND_VALUE: JSIteratorKindEnum = 2;
#[repr(C)]
struct JSForInIterator {
    obj: JSValue,
    idx: u32,
    atom_count: u32,
    in_prototype_chain: u8,
    is_array: u8,
    tab_atom: *mut JSPropertyEnum,
}
#[repr(C)]
#[derive(Clone, Copy)]
struct JSRegExp {
    pattern: *mut JSString,
    bytecode: *mut JSString,
}
#[repr(C)]
struct JSProxyData {
    target: JSValue,
    handler: JSValue,
    is_func: u8,
    is_revoked: u8,
}
#[repr(C)]
struct JSArrayBuffer {
    byte_length: i32,
    max_byte_length: i32,
    detached: u8,
    shared: u8,
    data: *mut u8,
    array_list: list_head,
    opaque: *mut c_void,
    free_func: Option<JSFreeArrayBufferDataFunc>,
}
#[repr(C)]
struct JSTypedArray {
    link: list_head,
    obj: *mut JSObject,
    buffer: *mut JSObject,
    offset: u32,
    length: u32,
    track_rab: JS_BOOL,
}
#[repr(C)]
#[derive(Clone, Copy)]
struct JSGlobalObject {
    uninitialized_vars: JSValue,
}
#[repr(C)]
struct JSAsyncFunctionState {
    header: JSGCObjectHeader,
    this_val: JSValue,
    argc: i32,
    throw_flag: JS_BOOL,
    is_completed: JS_BOOL,
    resolving_funcs: [JSValue; 2],
    frame: JSStackFrame,
}
type JSOverloadableOperatorEnum = u32;
const JS_OVOP_ADD: JSOverloadableOperatorEnum = 0;
const JS_OVOP_SUB: JSOverloadableOperatorEnum = 1;
const JS_OVOP_MUL: JSOverloadableOperatorEnum = 2;
const JS_OVOP_DIV: JSOverloadableOperatorEnum = 3;
const JS_OVOP_MOD: JSOverloadableOperatorEnum = 4;
const JS_OVOP_POW: JSOverloadableOperatorEnum = 5;
const JS_OVOP_OR: JSOverloadableOperatorEnum = 6;
const JS_OVOP_AND: JSOverloadableOperatorEnum = 7;
const JS_OVOP_XOR: JSOverloadableOperatorEnum = 8;
const JS_OVOP_SHL: JSOverloadableOperatorEnum = 9;
const JS_OVOP_SAR: JSOverloadableOperatorEnum = 10;
const JS_OVOP_SHR: JSOverloadableOperatorEnum = 11;
const JS_OVOP_EQ: JSOverloadableOperatorEnum = 12;
const JS_OVOP_LESS: JSOverloadableOperatorEnum = 13;
const JS_OVOP_BINARY_COUNT: usize = 14;
const JS_OVOP_POS: JSOverloadableOperatorEnum = 14;
const JS_OVOP_NEG: JSOverloadableOperatorEnum = 15;
const JS_OVOP_INC: JSOverloadableOperatorEnum = 16;
const JS_OVOP_DEC: JSOverloadableOperatorEnum = 17;
const JS_OVOP_NOT: JSOverloadableOperatorEnum = 18;
const JS_OVOP_COUNT: usize = 19;
#[repr(C)]
struct JSBinaryOperatorDefEntry {
    operator_index: u32,
    ops: [*mut JSObject; JS_OVOP_BINARY_COUNT],
}
#[repr(C)]
struct JSBinaryOperatorDef {
    count: i32,
    tab: *mut JSBinaryOperatorDefEntry,
}
#[repr(C)]
struct JSOperatorSetData {
    operator_counter: u32,
    is_primitive: JS_BOOL,
    self_ops: [*mut JSObject; JS_OVOP_COUNT],
    left: JSBinaryOperatorDef,
    right: JSBinaryOperatorDef,
}
#[repr(C)]
struct JSReqModuleEntry {
    module_name: JSAtom,
    module: *mut JSModuleDef,
    attributes: JSValue,
}
type JSExportTypeEnum = u32;
const JS_EXPORT_TYPE_LOCAL: JSExportTypeEnum = 0;
const JS_EXPORT_TYPE_INDIRECT: JSExportTypeEnum = 1;
#[repr(C)]
#[derive(Clone, Copy)]
struct JSLocalExport {
    var_idx: i32,
    var_ref: *mut JSVarRef,
}
#[repr(C)]
union JSExportUnion {
    local: JSLocalExport,
    req_module_idx: i32,
}
#[repr(C)]
struct JSExportEntry {
    u: JSExportUnion,
    export_type: JSExportTypeEnum,
    local_name: JSAtom,
    export_name: JSAtom,
}
#[repr(C)]
struct JSStarExportEntry {
    req_module_idx: i32,
}
#[repr(C)]
struct JSImportEntry {
    var_idx: i32,
    is_star: JS_BOOL,
    import_name: JSAtom,
    req_module_idx: i32,
}
type JSModuleStatus = u32;
const JS_MODULE_STATUS_UNLINKED: JSModuleStatus = 0;
const JS_MODULE_STATUS_LINKING: JSModuleStatus = 1;
const JS_MODULE_STATUS_LINKED: JSModuleStatus = 2;
const JS_MODULE_STATUS_EVALUATING: JSModuleStatus = 3;
const JS_MODULE_STATUS_EVALUATING_ASYNC: JSModuleStatus = 4;
const JS_MODULE_STATUS_EVALUATED: JSModuleStatus = 5;
#[repr(C)]
pub struct JSModuleDef {
    header: JSGCObjectHeader,
    module_name: JSAtom,
    link: list_head,
    req_module_entries: *mut JSReqModuleEntry,
    req_module_entries_count: i32,
    req_module_entries_size: i32,
    export_entries: *mut JSExportEntry,
    export_entries_count: i32,
    export_entries_size: i32,
    star_export_entries: *mut JSStarExportEntry,
    star_export_entries_count: i32,
    star_export_entries_size: i32,
    import_entries: *mut JSImportEntry,
    import_entries_count: i32,
    import_entries_size: i32,
    module_ns: JSValue,
    func_obj: JSValue,
    init_func: Option<JSModuleInitFunc>,
    has_tla: i8,
    resolved: i8,
    func_created: i8,
    status: u8,
    dfs_index: i32,
    dfs_ancestor_index: i32,
    stack_prev: *mut JSModuleDef,
    async_parent_modules: *mut *mut JSModuleDef,
    async_parent_modules_count: i32,
    async_parent_modules_size: i32,
    pending_async_dependencies: i32,
    async_evaluation: JS_BOOL,
    async_evaluation_timestamp: i64,
    cycle_root: *mut JSModuleDef,
    promise: JSValue,
    resolving_funcs: [JSValue; 2],
    eval_has_exception: i8,
    eval_exception: JSValue,
    meta_obj: JSValue,
    private_value: JSValue,
}
#[repr(C)]
struct JSJobEntry {
    link: list_head,
    realm: *mut JSContext,
    job_func: Option<JSJobFunc>,
    argc: i32,
    argv: [JSValue; 0],
}
