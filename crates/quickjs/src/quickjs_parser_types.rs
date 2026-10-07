// quickjs.c:21663..22032. Parser layouts and token constants. MIT.
use crate::cutils_header::DynBuf;
const TOK_NUMBER: i32 = -128;
const TOK_STRING: i32 = -127;
const TOK_TEMPLATE: i32 = -126;
const TOK_IDENT: i32 = -125;
const TOK_REGEXP: i32 = -124;
const TOK_MUL_ASSIGN: i32 = -123;
const TOK_DIV_ASSIGN: i32 = -122;
const TOK_MOD_ASSIGN: i32 = -121;
const TOK_PLUS_ASSIGN: i32 = -120;
const TOK_MINUS_ASSIGN: i32 = -119;
const TOK_SHL_ASSIGN: i32 = -118;
const TOK_SAR_ASSIGN: i32 = -117;
const TOK_SHR_ASSIGN: i32 = -116;
const TOK_AND_ASSIGN: i32 = -115;
const TOK_XOR_ASSIGN: i32 = -114;
const TOK_OR_ASSIGN: i32 = -113;
const TOK_POW_ASSIGN: i32 = -112;
const TOK_LAND_ASSIGN: i32 = -111;
const TOK_LOR_ASSIGN: i32 = -110;
const TOK_DOUBLE_QUESTION_MARK_ASSIGN: i32 = -109;
const TOK_DEC: i32 = -108;
const TOK_INC: i32 = -107;
const TOK_SHL: i32 = -106;
const TOK_SAR: i32 = -105;
const TOK_SHR: i32 = -104;
const TOK_LT: i32 = -103;
const TOK_LTE: i32 = -102;
const TOK_GT: i32 = -101;
const TOK_GTE: i32 = -100;
const TOK_EQ: i32 = -99;
const TOK_STRICT_EQ: i32 = -98;
const TOK_NEQ: i32 = -97;
const TOK_STRICT_NEQ: i32 = -96;
const TOK_LAND: i32 = -95;
const TOK_LOR: i32 = -94;
const TOK_POW: i32 = -93;
const TOK_ARROW: i32 = -92;
const TOK_ELLIPSIS: i32 = -91;
const TOK_DOUBLE_QUESTION_MARK: i32 = -90;
const TOK_QUESTION_MARK_DOT: i32 = -89;
const TOK_ERROR: i32 = -88;
const TOK_PRIVATE_NAME: i32 = -87;
const TOK_EOF: i32 = -86;
const TOK_NULL: i32 = -85;
const TOK_FALSE: i32 = -84;
const TOK_TRUE: i32 = -83;
const TOK_IF: i32 = -82;
const TOK_ELSE: i32 = -81;
const TOK_RETURN: i32 = -80;
const TOK_VAR: i32 = -79;
const TOK_THIS: i32 = -78;
const TOK_DELETE: i32 = -77;
const TOK_VOID: i32 = -76;
const TOK_TYPEOF: i32 = -75;
const TOK_NEW: i32 = -74;
const TOK_IN: i32 = -73;
const TOK_INSTANCEOF: i32 = -72;
const TOK_DO: i32 = -71;
const TOK_WHILE: i32 = -70;
const TOK_FOR: i32 = -69;
const TOK_BREAK: i32 = -68;
const TOK_CONTINUE: i32 = -67;
const TOK_SWITCH: i32 = -66;
const TOK_CASE: i32 = -65;
const TOK_DEFAULT: i32 = -64;
const TOK_THROW: i32 = -63;
const TOK_TRY: i32 = -62;
const TOK_CATCH: i32 = -61;
const TOK_FINALLY: i32 = -60;
const TOK_FUNCTION: i32 = -59;
const TOK_DEBUGGER: i32 = -58;
const TOK_WITH: i32 = -57;
const TOK_CLASS: i32 = -56;
const TOK_CONST: i32 = -55;
const TOK_ENUM: i32 = -54;
const TOK_EXPORT: i32 = -53;
const TOK_EXTENDS: i32 = -52;
const TOK_IMPORT: i32 = -51;
const TOK_SUPER: i32 = -50;
const TOK_IMPLEMENTS: i32 = -49;
const TOK_INTERFACE: i32 = -48;
const TOK_LET: i32 = -47;
const TOK_PACKAGE: i32 = -46;
const TOK_PRIVATE: i32 = -45;
const TOK_PROTECTED: i32 = -44;
const TOK_PUBLIC: i32 = -43;
const TOK_STATIC: i32 = -42;
const TOK_YIELD: i32 = -41;
const TOK_AWAIT: i32 = -40;
const TOK_OF: i32 = -39;
const TOK_FIRST_KEYWORD: i32 = TOK_NULL;
const TOK_LAST_KEYWORD: i32 = TOK_AWAIT;
const CP_NBSP: i32 = 0x00a0;
const CP_BOM: i32 = 0xfeff;
const CP_LS: i32 = 0x2028;
const CP_PS: i32 = 0x2029;
type JSParseFunctionEnum = u32;
const JS_PARSE_FUNC_STATEMENT: JSParseFunctionEnum = 0;
const JS_PARSE_FUNC_VAR: JSParseFunctionEnum = 1;
const JS_PARSE_FUNC_EXPR: JSParseFunctionEnum = 2;
const JS_PARSE_FUNC_ARROW: JSParseFunctionEnum = 3;
const JS_PARSE_FUNC_GETTER: JSParseFunctionEnum = 4;
const JS_PARSE_FUNC_SETTER: JSParseFunctionEnum = 5;
const JS_PARSE_FUNC_METHOD: JSParseFunctionEnum = 6;
const JS_PARSE_FUNC_CLASS_STATIC_INIT: JSParseFunctionEnum = 7;
const JS_PARSE_FUNC_CLASS_CONSTRUCTOR: JSParseFunctionEnum = 8;
const JS_PARSE_FUNC_DERIVED_CLASS_CONSTRUCTOR: JSParseFunctionEnum = 9;
type JSParseExportEnum = u32;
const JS_PARSE_EXPORT_NONE: JSParseExportEnum = 0;
const JS_PARSE_EXPORT_NAMED: JSParseExportEnum = 1;
const JS_PARSE_EXPORT_DEFAULT: JSParseExportEnum = 2;
#[repr(C)]
struct BlockEnv {
    prev: *mut BlockEnv,
    label_name: JSAtom,
    label_break: i32,
    label_cont: i32,
    drop_count: i32,
    label_finally: i32,
    scope_level: i32,
    flags: u8,
}
impl BlockEnv {
    fn has_iterator(&self) -> u8 {
        self.flags & 1
    }
    fn set_has_iterator(&mut self, n: u8) {
        self.flags = (self.flags & !1) | (n & 1)
    }
    fn is_regular_stmt(&self) -> u8 {
        (self.flags >> 1) & 1
    }
    fn set_is_regular_stmt(&mut self, n: u8) {
        self.flags = (self.flags & !2) | ((n & 1) << 1)
    }
}
#[repr(C)]
struct JSGlobalVar {
    cpool_idx: i32,
    flags: u8,
    scope_level: i32,
    var_name: JSAtom,
}
impl JSGlobalVar {
    fn force_init(&self) -> u8 {
        self.flags & 1
    }
    fn set_force_init(&mut self, n: u8) {
        self.flags = (self.flags & !1) | (n & 1)
    }
    fn is_lexical(&self) -> u8 {
        (self.flags >> 1) & 1
    }
    fn set_is_lexical(&mut self, n: u8) {
        self.flags = (self.flags & !2) | ((n & 1) << 1)
    }
    fn is_const(&self) -> u8 {
        (self.flags >> 2) & 1
    }
    fn set_is_const(&mut self, n: u8) {
        self.flags = (self.flags & !4) | ((n & 1) << 2)
    }
}
#[repr(C)]
struct RelocEntry {
    next: *mut RelocEntry,
    addr: u32,
    size: i32,
}
#[repr(C)]
struct JumpSlot {
    op: i32,
    size: i32,
    pos: i32,
    label: i32,
}
#[repr(C)]
struct LabelSlot {
    ref_count: i32,
    pos: i32,
    pos2: i32,
    addr: i32,
    first_reloc: *mut RelocEntry,
}
#[repr(C)]
struct LineNumberSlot {
    pc: u32,
    source_pos: u32,
}
#[repr(C)]
struct GetLineColCache {
    ptr: *const u8,
    line_num: i32,
    col_num: i32,
    buf_start: *const u8,
}
#[repr(C)]
#[derive(Clone, Copy)]
struct JSVarScope {
    parent: i32,
    first: i32,
}
#[repr(C)]
struct JSVarDef {
    var_name: JSAtom,
    scope_level: i32,
    scope_next: i32,
    flags: u8,
    var_ref_idx: u16,
    func_pool_idx: i32,
}
impl JSVarDef {
    fn is_const(&self) -> u8 {
        self.flags & 1
    }
    fn set_is_const(&mut self, n: u8) {
        self.flags = (self.flags & !1) | (n & 1)
    }
    fn is_lexical(&self) -> u8 {
        (self.flags >> 1) & 1
    }
    fn set_is_lexical(&mut self, n: u8) {
        self.flags = (self.flags & !2) | ((n & 1) << 1)
    }
    fn is_captured(&self) -> u8 {
        (self.flags >> 2) & 1
    }
    fn set_is_captured(&mut self, n: u8) {
        self.flags = (self.flags & !4) | ((n & 1) << 2)
    }
    fn is_static_private(&self) -> u8 {
        (self.flags >> 3) & 1
    }
    fn set_is_static_private(&mut self, n: u8) {
        self.flags = (self.flags & !8) | ((n & 1) << 3)
    }
    fn var_kind(&self) -> u8 {
        self.flags >> 4
    }
    fn set_var_kind(&mut self, n: u8) {
        self.flags = (self.flags & 15) | ((n & 15) << 4)
    }
}
#[repr(C)]
struct JSFunctionDef {
    ctx: *mut JSContext,
    parent: *mut JSFunctionDef,
    parent_cpool_idx: i32,
    parent_scope_level: i32,
    child_list: list_head,
    link: list_head,
    is_eval: i32,
    eval_type: i32,
    is_global_var: i32,
    is_func_expr: i32,
    has_home_object: i32,
    has_prototype: i32,
    has_simple_parameter_list: i32,
    has_parameter_expressions: i32,
    has_use_strict: i32,
    has_eval_call: i32,
    has_arguments_binding: i32,
    has_this_binding: i32,
    new_target_allowed: i32,
    super_call_allowed: i32,
    super_allowed: i32,
    arguments_allowed: i32,
    is_derived_class_constructor: i32,
    in_function_body: i32,
    func_kind: u8,
    func_type: u8,
    js_mode: u8,
    func_name: JSAtom,
    vars: *mut JSVarDef,
    var_size: i32,
    var_count: i32,
    args: *mut JSVarDef,
    arg_size: i32,
    arg_count: i32,
    defined_arg_count: i32,
    var_ref_count: i32,
    var_object_idx: i32,
    arg_var_object_idx: i32,
    arguments_var_idx: i32,
    arguments_arg_idx: i32,
    func_var_idx: i32,
    eval_ret_idx: i32,
    this_var_idx: i32,
    new_target_var_idx: i32,
    this_active_func_var_idx: i32,
    home_object_var_idx: i32,
    need_home_object: i32,
    scope_level: i32,
    scope_first: i32,
    scope_size: i32,
    scope_count: i32,
    scopes: *mut JSVarScope,
    def_scope_array: [JSVarScope; 4],
    body_scope: i32,
    global_var_count: i32,
    global_var_size: i32,
    global_vars: *mut JSGlobalVar,
    byte_code: DynBuf,
    last_opcode_pos: i32,
    last_opcode_source_ptr: *const u8,
    use_short_opcodes: i32,
    label_slots: *mut LabelSlot,
    label_size: i32,
    label_count: i32,
    top_break: *mut BlockEnv,
    cpool: *mut JSValue,
    cpool_count: i32,
    cpool_size: i32,
    closure_var_count: i32,
    closure_var_size: i32,
    closure_var: *mut JSClosureVar,
    jump_slots: *mut JumpSlot,
    jump_size: i32,
    jump_count: i32,
    line_number_slots: *mut LineNumberSlot,
    line_number_size: i32,
    line_number_count: i32,
    line_number_last: i32,
    line_number_last_pc: i32,
    debug_flags: u32,
    filename: JSAtom,
    source_pos: u32,
    get_line_col_cache: *mut GetLineColCache,
    pc2line: DynBuf,
    source: *mut c_char,
    source_len: i32,
    module: *mut JSModuleDef,
    has_await: i32,
}
impl JSFunctionDef {
    fn strip_debug(&self) -> u8 {
        (self.debug_flags & 1) as u8
    }
    fn set_strip_debug(&mut self, n: u8) {
        self.debug_flags = (self.debug_flags & !1) | u32::from(n & 1)
    }
    fn strip_source(&self) -> u8 {
        ((self.debug_flags >> 1) & 1) as u8
    }
    fn set_strip_source(&mut self, n: u8) {
        self.debug_flags = (self.debug_flags & !2) | (u32::from(n & 1) << 1)
    }
}
#[repr(C)]
#[derive(Clone, Copy)]
struct JSTokenStr {
    str: JSValue,
    sep: i32,
}
#[repr(C)]
#[derive(Clone, Copy)]
struct JSTokenNum {
    val: JSValue,
}
#[repr(C)]
#[derive(Clone, Copy)]
struct JSTokenIdent {
    atom: JSAtom,
    has_escape: i32,
    is_reserved: i32,
}
#[repr(C)]
#[derive(Clone, Copy)]
struct JSTokenRegexp {
    body: JSValue,
    flags: JSValue,
}
#[repr(C)]
#[derive(Clone, Copy)]
union JSTokenUnion {
    str: JSTokenStr,
    num: JSTokenNum,
    ident: JSTokenIdent,
    regexp: JSTokenRegexp,
}
#[repr(C)]
#[derive(Clone, Copy)]
struct JSToken {
    val: i32,
    ptr: *const u8,
    u: JSTokenUnion,
}
#[repr(C)]
struct JSParseState {
    ctx: *mut JSContext,
    filename: *const c_char,
    token: JSToken,
    got_lf: i32,
    last_ptr: *const u8,
    buf_start: *const u8,
    buf_ptr: *const u8,
    buf_end: *const u8,
    cur_func: *mut JSFunctionDef,
    is_module: i32,
    allow_html_comments: i32,
    ext_json: i32,
    get_line_col_cache: GetLineColCache,
}

// Auxiliary C parser/compiler/module records, quickjs.c:24485..35592.
const PROP_TYPE_IDENT: i32 = 0;
const PROP_TYPE_VAR: i32 = 1;
const PROP_TYPE_GET: i32 = 2;
const PROP_TYPE_SET: i32 = 3;
const PROP_TYPE_STAR: i32 = 4;
const PROP_TYPE_ASYNC: i32 = 5;
const PROP_TYPE_ASYNC_STAR: i32 = 6;
const PROP_TYPE_PRIVATE: i32 = 1 << 4;
type FuncCallType = u32;
const FUNC_CALL_NORMAL: FuncCallType = 0;
const FUNC_CALL_NEW: FuncCallType = 1;
const FUNC_CALL_SUPER_CTOR: FuncCallType = 2;
const FUNC_CALL_TEMPLATE: FuncCallType = 3;
#[repr(C)]
#[derive(Clone, Copy)]
struct ClassFieldsDef { fields_init_fd: *mut JSFunctionDef, computed_fields_count: i32, need_brand: i32, brand_push_pos: i32, is_static: i32 }
#[repr(C)]
struct JSResolveEntry { module: *mut JSModuleDef, name: JSAtom }
#[repr(C)]
struct JSResolveState { array: *mut JSResolveEntry, size: i32, count: i32 }
type JSResolveResultEnum = i32;
const JS_RESOLVE_RES_EXCEPTION: i32 = -1;
const JS_RESOLVE_RES_FOUND: i32 = 0;
const JS_RESOLVE_RES_NOT_FOUND: i32 = 1;
const JS_RESOLVE_RES_CIRCULAR: i32 = 2;
const JS_RESOLVE_RES_AMBIGUOUS: i32 = 3;
type ExportedNameEntryEnum = u32;
const EXPORTED_NAME_AMBIGUOUS: u32 = 0;
const EXPORTED_NAME_NORMAL: u32 = 1;
const EXPORTED_NAME_DELAYED: u32 = 2;
#[repr(C)]
#[derive(Clone, Copy)]
union ExportedNameEntryUnion { me: *mut JSExportEntry, var_ref: *mut JSVarRef }
#[repr(C)]
struct ExportedNameEntry { export_name: JSAtom, export_type: ExportedNameEntryEnum, u: ExportedNameEntryUnion }
#[repr(C)]
struct GetExportNamesState { modules: *mut *mut JSModuleDef, modules_size: i32, modules_count: i32, exported_names: *mut ExportedNameEntry, exported_names_size: i32, exported_names_count: i32 }
#[repr(C)]
struct ExecModuleList { tab: *mut *mut JSModuleDef, count: i32, size: i32 }
#[repr(C)]
struct CodeContext { bc_buf: *const u8, bc_len: i32, pos: i32, line_num: i32, op: i32, idx: i32, label: i32, val: i32, atom: JSAtom }
#[repr(C)]
struct StackSizeState { bc_len: i32, stack_len_max: i32, stack_level_tab: *mut u16, catch_pos_tab: *mut i32, pc_stack: *mut i32, pc_stack_len: i32, pc_stack_size: i32 }
