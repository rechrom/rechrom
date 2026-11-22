// quickjs.c:21663..22032. Parser layouts and token constants. MIT.
use super::*;

pub(crate) const TOK_NUMBER: i32 = -128;
pub(crate) const TOK_STRING: i32 = -127;
pub(crate) const TOK_TEMPLATE: i32 = -126;
pub(crate) const TOK_IDENT: i32 = -125;
pub(crate) const TOK_REGEXP: i32 = -124;
pub(crate) const TOK_MUL_ASSIGN: i32 = -123;
pub(crate) const TOK_DIV_ASSIGN: i32 = -122;
pub(crate) const TOK_MOD_ASSIGN: i32 = -121;
pub(crate) const TOK_PLUS_ASSIGN: i32 = -120;
pub(crate) const TOK_MINUS_ASSIGN: i32 = -119;
pub(crate) const TOK_SHL_ASSIGN: i32 = -118;
pub(crate) const TOK_SAR_ASSIGN: i32 = -117;
pub(crate) const TOK_SHR_ASSIGN: i32 = -116;
pub(crate) const TOK_AND_ASSIGN: i32 = -115;
pub(crate) const TOK_XOR_ASSIGN: i32 = -114;
pub(crate) const TOK_OR_ASSIGN: i32 = -113;
pub(crate) const TOK_POW_ASSIGN: i32 = -112;
pub(crate) const TOK_LAND_ASSIGN: i32 = -111;
pub(crate) const TOK_LOR_ASSIGN: i32 = -110;
pub(crate) const TOK_DOUBLE_QUESTION_MARK_ASSIGN: i32 = -109;
pub(crate) const TOK_DEC: i32 = -108;
pub(crate) const TOK_INC: i32 = -107;
pub(crate) const TOK_SHL: i32 = -106;
pub(crate) const TOK_SAR: i32 = -105;
pub(crate) const TOK_SHR: i32 = -104;
pub(crate) const TOK_LT: i32 = -103;
pub(crate) const TOK_LTE: i32 = -102;
pub(crate) const TOK_GT: i32 = -101;
pub(crate) const TOK_GTE: i32 = -100;
pub(crate) const TOK_EQ: i32 = -99;
pub(crate) const TOK_STRICT_EQ: i32 = -98;
pub(crate) const TOK_NEQ: i32 = -97;
pub(crate) const TOK_STRICT_NEQ: i32 = -96;
pub(crate) const TOK_LAND: i32 = -95;
pub(crate) const TOK_LOR: i32 = -94;
pub(crate) const TOK_POW: i32 = -93;
pub(crate) const TOK_ARROW: i32 = -92;
pub(crate) const TOK_ELLIPSIS: i32 = -91;
pub(crate) const TOK_DOUBLE_QUESTION_MARK: i32 = -90;
pub(crate) const TOK_QUESTION_MARK_DOT: i32 = -89;
pub(crate) const TOK_ERROR: i32 = -88;
pub(crate) const TOK_PRIVATE_NAME: i32 = -87;
pub(crate) const TOK_EOF: i32 = -86;
pub(crate) const TOK_NULL: i32 = -85;
pub(crate) const TOK_FALSE: i32 = -84;
pub(crate) const TOK_TRUE: i32 = -83;
pub(crate) const TOK_IF: i32 = -82;
pub(crate) const TOK_ELSE: i32 = -81;
pub(crate) const TOK_RETURN: i32 = -80;
pub(crate) const TOK_VAR: i32 = -79;
pub(crate) const TOK_THIS: i32 = -78;
pub(crate) const TOK_DELETE: i32 = -77;
pub(crate) const TOK_VOID: i32 = -76;
pub(crate) const TOK_TYPEOF: i32 = -75;
pub(crate) const TOK_NEW: i32 = -74;
pub(crate) const TOK_IN: i32 = -73;
pub(crate) const TOK_INSTANCEOF: i32 = -72;
pub(crate) const TOK_DO: i32 = -71;
pub(crate) const TOK_WHILE: i32 = -70;
pub(crate) const TOK_FOR: i32 = -69;
pub(crate) const TOK_BREAK: i32 = -68;
pub(crate) const TOK_CONTINUE: i32 = -67;
pub(crate) const TOK_SWITCH: i32 = -66;
pub(crate) const TOK_CASE: i32 = -65;
pub(crate) const TOK_DEFAULT: i32 = -64;
pub(crate) const TOK_THROW: i32 = -63;
pub(crate) const TOK_TRY: i32 = -62;
pub(crate) const TOK_CATCH: i32 = -61;
pub(crate) const TOK_FINALLY: i32 = -60;
pub(crate) const TOK_FUNCTION: i32 = -59;
pub(crate) const TOK_DEBUGGER: i32 = -58;
pub(crate) const TOK_WITH: i32 = -57;
pub(crate) const TOK_CLASS: i32 = -56;
pub(crate) const TOK_CONST: i32 = -55;
pub(crate) const TOK_ENUM: i32 = -54;
pub(crate) const TOK_EXPORT: i32 = -53;
pub(crate) const TOK_EXTENDS: i32 = -52;
pub(crate) const TOK_IMPORT: i32 = -51;
pub(crate) const TOK_SUPER: i32 = -50;
pub(crate) const TOK_IMPLEMENTS: i32 = -49;
pub(crate) const TOK_INTERFACE: i32 = -48;
pub(crate) const TOK_LET: i32 = -47;
pub(crate) const TOK_PACKAGE: i32 = -46;
pub(crate) const TOK_PRIVATE: i32 = -45;
pub(crate) const TOK_PROTECTED: i32 = -44;
pub(crate) const TOK_PUBLIC: i32 = -43;
pub(crate) const TOK_STATIC: i32 = -42;
pub(crate) const TOK_YIELD: i32 = -41;
pub(crate) const TOK_AWAIT: i32 = -40;
pub(crate) const TOK_OF: i32 = -39;
pub(crate) const TOK_FIRST_KEYWORD: i32 = TOK_NULL;
pub(crate) const TOK_LAST_KEYWORD: i32 = TOK_AWAIT;
pub(crate) const CP_NBSP: i32 = 0x00a0;
pub(crate) const CP_BOM: i32 = 0xfeff;
pub(crate) const CP_LS: i32 = 0x2028;
pub(crate) const CP_PS: i32 = 0x2029;
pub(crate) type JSParseFunctionEnum = u32;
pub(crate) const JS_PARSE_FUNC_STATEMENT: JSParseFunctionEnum = 0;
pub(crate) const JS_PARSE_FUNC_VAR: JSParseFunctionEnum = 1;
pub(crate) const JS_PARSE_FUNC_EXPR: JSParseFunctionEnum = 2;
pub(crate) const JS_PARSE_FUNC_ARROW: JSParseFunctionEnum = 3;
pub(crate) const JS_PARSE_FUNC_GETTER: JSParseFunctionEnum = 4;
pub(crate) const JS_PARSE_FUNC_SETTER: JSParseFunctionEnum = 5;
pub(crate) const JS_PARSE_FUNC_METHOD: JSParseFunctionEnum = 6;
pub(crate) const JS_PARSE_FUNC_CLASS_STATIC_INIT: JSParseFunctionEnum = 7;
pub(crate) const JS_PARSE_FUNC_CLASS_CONSTRUCTOR: JSParseFunctionEnum = 8;
pub(crate) const JS_PARSE_FUNC_DERIVED_CLASS_CONSTRUCTOR: JSParseFunctionEnum = 9;
pub(crate) type JSParseExportEnum = u32;
pub(crate) const JS_PARSE_EXPORT_NONE: JSParseExportEnum = 0;
pub(crate) const JS_PARSE_EXPORT_NAMED: JSParseExportEnum = 1;
pub(crate) const JS_PARSE_EXPORT_DEFAULT: JSParseExportEnum = 2;
#[repr(C)]
pub(crate) struct BlockEnv {
    pub(crate) prev: *mut BlockEnv,
    pub(crate) label_name: JSAtom,
    pub(crate) label_break: i32,
    pub(crate) label_cont: i32,
    pub(crate) drop_count: i32,
    pub(crate) label_finally: i32,
    pub(crate) scope_level: i32,
    pub(crate) flags: u8,
}
impl BlockEnv {
    pub(crate) fn has_iterator(&self) -> u8 {
        self.flags & 1
    }
    pub(crate) fn set_has_iterator(&mut self, n: u8) {
        self.flags = (self.flags & !1) | (n & 1)
    }
    pub(crate) fn is_regular_stmt(&self) -> u8 {
        (self.flags >> 1) & 1
    }
    pub(crate) fn set_is_regular_stmt(&mut self, n: u8) {
        self.flags = (self.flags & !2) | ((n & 1) << 1)
    }
}
#[repr(C)]
pub(crate) struct JSGlobalVar {
    pub(crate) cpool_idx: i32,
    pub(crate) flags: u8,
    pub(crate) scope_level: i32,
    pub(crate) var_name: JSAtom,
}
impl JSGlobalVar {
    pub(crate) fn force_init(&self) -> u8 {
        self.flags & 1
    }
    pub(crate) fn set_force_init(&mut self, n: u8) {
        self.flags = (self.flags & !1) | (n & 1)
    }
    pub(crate) fn is_lexical(&self) -> u8 {
        (self.flags >> 1) & 1
    }
    pub(crate) fn set_is_lexical(&mut self, n: u8) {
        self.flags = (self.flags & !2) | ((n & 1) << 1)
    }
    pub(crate) fn is_const(&self) -> u8 {
        (self.flags >> 2) & 1
    }
    pub(crate) fn set_is_const(&mut self, n: u8) {
        self.flags = (self.flags & !4) | ((n & 1) << 2)
    }
}
#[repr(C)]
pub(crate) struct RelocEntry {
    pub(crate) next: *mut RelocEntry,
    pub(crate) addr: u32,
    pub(crate) size: i32,
}
#[repr(C)]
pub(crate) struct JumpSlot {
    pub(crate) op: i32,
    pub(crate) size: i32,
    pub(crate) pos: i32,
    pub(crate) label: i32,
}
#[repr(C)]
pub(crate) struct LabelSlot {
    pub(crate) ref_count: i32,
    pub(crate) pos: i32,
    pub(crate) pos2: i32,
    pub(crate) addr: i32,
    pub(crate) first_reloc: *mut RelocEntry,
}
#[repr(C)]
pub(crate) struct LineNumberSlot {
    pub(crate) pc: u32,
    pub(crate) source_pos: u32,
}
#[repr(C)]
pub(crate) struct GetLineColCache {
    pub(crate) ptr: *const u8,
    pub(crate) line_num: i32,
    pub(crate) col_num: i32,
    pub(crate) buf_start: *const u8,
}
#[repr(C)]
#[derive(Clone, Copy)]
pub(crate) struct JSVarScope {
    pub(crate) parent: i32,
    pub(crate) first: i32,
}
#[repr(C)]
pub(crate) struct JSVarDef {
    pub(crate) var_name: JSAtom,
    pub(crate) scope_level: i32,
    pub(crate) scope_next: i32,
    pub(crate) flags: u8,
    pub(crate) var_ref_idx: u16,
    pub(crate) func_pool_idx: i32,
}
impl JSVarDef {
    pub(crate) fn is_const(&self) -> u8 {
        self.flags & 1
    }
    pub(crate) fn set_is_const(&mut self, n: u8) {
        self.flags = (self.flags & !1) | (n & 1)
    }
    pub(crate) fn is_lexical(&self) -> u8 {
        (self.flags >> 1) & 1
    }
    pub(crate) fn set_is_lexical(&mut self, n: u8) {
        self.flags = (self.flags & !2) | ((n & 1) << 1)
    }
    pub(crate) fn is_captured(&self) -> u8 {
        (self.flags >> 2) & 1
    }
    pub(crate) fn set_is_captured(&mut self, n: u8) {
        self.flags = (self.flags & !4) | ((n & 1) << 2)
    }
    pub(crate) fn is_static_private(&self) -> u8 {
        (self.flags >> 3) & 1
    }
    pub(crate) fn set_is_static_private(&mut self, n: u8) {
        self.flags = (self.flags & !8) | ((n & 1) << 3)
    }
    pub(crate) fn var_kind(&self) -> u8 {
        self.flags >> 4
    }
    pub(crate) fn set_var_kind(&mut self, n: u8) {
        self.flags = (self.flags & 15) | ((n & 15) << 4)
    }
}
#[repr(C)]
pub(crate) struct JSFunctionDef {
    pub(crate) ctx: *mut JSContext,
    pub(crate) parent: *mut JSFunctionDef,
    pub(crate) parent_cpool_idx: i32,
    pub(crate) parent_scope_level: i32,
    pub(crate) child_list: list_head,
    pub(crate) link: list_head,
    pub(crate) is_eval: i32,
    pub(crate) eval_type: i32,
    pub(crate) is_global_var: i32,
    pub(crate) is_func_expr: i32,
    pub(crate) has_home_object: i32,
    pub(crate) has_prototype: i32,
    pub(crate) has_simple_parameter_list: i32,
    pub(crate) has_parameter_expressions: i32,
    pub(crate) has_use_strict: i32,
    pub(crate) has_eval_call: i32,
    pub(crate) has_arguments_binding: i32,
    pub(crate) has_this_binding: i32,
    pub(crate) new_target_allowed: i32,
    pub(crate) super_call_allowed: i32,
    pub(crate) super_allowed: i32,
    pub(crate) arguments_allowed: i32,
    pub(crate) is_derived_class_constructor: i32,
    pub(crate) in_function_body: i32,
    pub(crate) func_kind: u8,
    pub(crate) func_type: u8,
    pub(crate) js_mode: u8,
    pub(crate) func_name: JSAtom,
    pub(crate) vars: *mut JSVarDef,
    pub(crate) var_size: i32,
    pub(crate) var_count: i32,
    pub(crate) args: *mut JSVarDef,
    pub(crate) arg_size: i32,
    pub(crate) arg_count: i32,
    pub(crate) defined_arg_count: i32,
    pub(crate) var_ref_count: i32,
    pub(crate) var_object_idx: i32,
    pub(crate) arg_var_object_idx: i32,
    pub(crate) arguments_var_idx: i32,
    pub(crate) arguments_arg_idx: i32,
    pub(crate) func_var_idx: i32,
    pub(crate) eval_ret_idx: i32,
    pub(crate) this_var_idx: i32,
    pub(crate) new_target_var_idx: i32,
    pub(crate) this_active_func_var_idx: i32,
    pub(crate) home_object_var_idx: i32,
    pub(crate) need_home_object: i32,
    pub(crate) scope_level: i32,
    pub(crate) scope_first: i32,
    pub(crate) scope_size: i32,
    pub(crate) scope_count: i32,
    pub(crate) scopes: *mut JSVarScope,
    pub(crate) def_scope_array: [JSVarScope; 4],
    pub(crate) body_scope: i32,
    pub(crate) global_var_count: i32,
    pub(crate) global_var_size: i32,
    pub(crate) global_vars: *mut JSGlobalVar,
    pub(crate) byte_code: DynBuf,
    pub(crate) last_opcode_pos: i32,
    pub(crate) last_opcode_source_ptr: *const u8,
    pub(crate) use_short_opcodes: i32,
    pub(crate) label_slots: *mut LabelSlot,
    pub(crate) label_size: i32,
    pub(crate) label_count: i32,
    pub(crate) top_break: *mut BlockEnv,
    pub(crate) cpool: *mut JSValue,
    pub(crate) cpool_count: i32,
    pub(crate) cpool_size: i32,
    pub(crate) closure_var_count: i32,
    pub(crate) closure_var_size: i32,
    pub(crate) closure_var: *mut JSClosureVar,
    pub(crate) jump_slots: *mut JumpSlot,
    pub(crate) jump_size: i32,
    pub(crate) jump_count: i32,
    pub(crate) line_number_slots: *mut LineNumberSlot,
    pub(crate) line_number_size: i32,
    pub(crate) line_number_count: i32,
    pub(crate) line_number_last: i32,
    pub(crate) line_number_last_pc: i32,
    pub(crate) debug_flags: u32,
    pub(crate) filename: JSAtom,
    pub(crate) source_pos: u32,
    pub(crate) get_line_col_cache: *mut GetLineColCache,
    pub(crate) pc2line: DynBuf,
    pub(crate) source: *mut c_char,
    pub(crate) source_len: i32,
    pub(crate) module: *mut JSModuleDef,
    pub(crate) has_await: i32,
}
impl JSFunctionDef {
    pub(crate) fn strip_debug(&self) -> u8 {
        (self.debug_flags & 1) as u8
    }
    pub(crate) fn set_strip_debug(&mut self, n: u8) {
        self.debug_flags = (self.debug_flags & !1) | u32::from(n & 1)
    }
    pub(crate) fn strip_source(&self) -> u8 {
        ((self.debug_flags >> 1) & 1) as u8
    }
    pub(crate) fn set_strip_source(&mut self, n: u8) {
        self.debug_flags = (self.debug_flags & !2) | (u32::from(n & 1) << 1)
    }
}
#[repr(C)]
#[derive(Clone, Copy)]
pub(crate) struct JSTokenStr {
    pub(crate) str: JSValue,
    pub(crate) sep: i32,
}
#[repr(C)]
#[derive(Clone, Copy)]
pub(crate) struct JSTokenNum {
    pub(crate) val: JSValue,
}
#[repr(C)]
#[derive(Clone, Copy)]
pub(crate) struct JSTokenIdent {
    pub(crate) atom: JSAtom,
    pub(crate) has_escape: i32,
    pub(crate) is_reserved: i32,
}
#[repr(C)]
#[derive(Clone, Copy)]
pub(crate) struct JSTokenRegexp {
    pub(crate) body: JSValue,
    pub(crate) flags: JSValue,
}
#[repr(C)]
#[derive(Clone, Copy)]
pub(crate) union JSTokenUnion {
    pub(crate) str: JSTokenStr,
    pub(crate) num: JSTokenNum,
    pub(crate) ident: JSTokenIdent,
    pub(crate) regexp: JSTokenRegexp,
}
#[repr(C)]
#[derive(Clone, Copy)]
pub(crate) struct JSToken {
    pub(crate) val: i32,
    pub(crate) ptr: *const u8,
    pub(crate) u: JSTokenUnion,
}
#[repr(C)]
pub(crate) struct JSParseState {
    pub(crate) ctx: *mut JSContext,
    pub(crate) filename: *const c_char,
    pub(crate) token: JSToken,
    pub(crate) got_lf: i32,
    pub(crate) last_ptr: *const u8,
    pub(crate) buf_start: *const u8,
    pub(crate) buf_ptr: *const u8,
    pub(crate) buf_end: *const u8,
    pub(crate) cur_func: *mut JSFunctionDef,
    pub(crate) is_module: i32,
    pub(crate) allow_html_comments: i32,
    pub(crate) ext_json: i32,
    pub(crate) get_line_col_cache: GetLineColCache,
}

// Auxiliary C parser/compiler/module records, quickjs.c:24485..35592.
pub(crate) const PROP_TYPE_IDENT: i32 = 0;
pub(crate) const PROP_TYPE_VAR: i32 = 1;
pub(crate) const PROP_TYPE_GET: i32 = 2;
pub(crate) const PROP_TYPE_SET: i32 = 3;
pub(crate) const PROP_TYPE_STAR: i32 = 4;
pub(crate) const PROP_TYPE_ASYNC: i32 = 5;
pub(crate) const PROP_TYPE_ASYNC_STAR: i32 = 6;
pub(crate) const PROP_TYPE_PRIVATE: i32 = 1 << 4;
pub(crate) type FuncCallType = u32;
pub(crate) const FUNC_CALL_NORMAL: FuncCallType = 0;
pub(crate) const FUNC_CALL_NEW: FuncCallType = 1;
pub(crate) const FUNC_CALL_SUPER_CTOR: FuncCallType = 2;
pub(crate) const FUNC_CALL_TEMPLATE: FuncCallType = 3;
#[repr(C)]
#[derive(Clone, Copy)]
pub(crate) struct ClassFieldsDef { pub(crate) fields_init_fd: *mut JSFunctionDef, pub(crate) computed_fields_count: i32, pub(crate) need_brand: i32, pub(crate) brand_push_pos: i32, pub(crate) is_static: i32 }
#[repr(C)]
pub(crate) struct JSResolveEntry { pub(crate) module: *mut JSModuleDef, pub(crate) name: JSAtom }
#[repr(C)]
pub(crate) struct JSResolveState { pub(crate) array: *mut JSResolveEntry, pub(crate) size: i32, pub(crate) count: i32 }
pub(crate) type JSResolveResultEnum = i32;
pub(crate) const JS_RESOLVE_RES_EXCEPTION: i32 = -1;
pub(crate) const JS_RESOLVE_RES_FOUND: i32 = 0;
pub(crate) const JS_RESOLVE_RES_NOT_FOUND: i32 = 1;
pub(crate) const JS_RESOLVE_RES_CIRCULAR: i32 = 2;
pub(crate) const JS_RESOLVE_RES_AMBIGUOUS: i32 = 3;
pub(crate) type ExportedNameEntryEnum = u32;
pub(crate) const EXPORTED_NAME_AMBIGUOUS: u32 = 0;
pub(crate) const EXPORTED_NAME_NORMAL: u32 = 1;
pub(crate) const EXPORTED_NAME_DELAYED: u32 = 2;
#[repr(C)]
#[derive(Clone, Copy)]
pub(crate) union ExportedNameEntryUnion { pub(crate) me: *mut JSExportEntry, pub(crate) var_ref: *mut JSVarRef }
#[repr(C)]
pub(crate) struct ExportedNameEntry { pub(crate) export_name: JSAtom, pub(crate) export_type: ExportedNameEntryEnum, pub(crate) u: ExportedNameEntryUnion }
#[repr(C)]
pub(crate) struct GetExportNamesState { pub(crate) modules: *mut *mut JSModuleDef, pub(crate) modules_size: i32, pub(crate) modules_count: i32, pub(crate) exported_names: *mut ExportedNameEntry, pub(crate) exported_names_size: i32, pub(crate) exported_names_count: i32 }
#[repr(C)]
pub(crate) struct ExecModuleList { pub(crate) tab: *mut *mut JSModuleDef, pub(crate) count: i32, pub(crate) size: i32 }
#[repr(C)]
pub(crate) struct CodeContext { pub(crate) bc_buf: *const u8, pub(crate) bc_len: i32, pub(crate) pos: i32, pub(crate) line_num: i32, pub(crate) op: i32, pub(crate) idx: i32, pub(crate) label: i32, pub(crate) val: i32, pub(crate) atom: JSAtom }
#[repr(C)]
pub(crate) struct StackSizeState { pub(crate) bc_len: i32, pub(crate) stack_len_max: i32, pub(crate) stack_level_tab: *mut u16, pub(crate) catch_pos_tab: *mut i32, pub(crate) pc_stack: *mut i32, pub(crate) pc_stack_len: i32, pub(crate) pc_stack_size: i32 }
