// quickjs.c:23693..24330. Bytecode emission, labels and lexical scopes. MIT.
use crate::cutils::{dbuf_claim, dbuf_put_u16, dbuf_put_u32, dbuf_putc};
use crate::cutils_header::{dbuf_error, dbuf_set_error, put_u32};
use crate::quickjs_atom::JS_ATOM_arguments;
use crate::quickjs_opcode::*;
const GLOBAL_VAR_OFFSET: i32 = 0x40000000;
const ARGUMENT_VAR_OFFSET: i32 = 0x20000000;
const JS_MAX_LOCAL_VARS: i32 = 65534;
unsafe fn get_prev_opcode(fd: *mut JSFunctionDef) -> i32 {
    if (*fd).last_opcode_pos < 0 || dbuf_error(&(*fd).byte_code) != 0 {
        OP_invalid as i32
    } else {
        i32::from(*(*fd).byte_code.buf.offset((*fd).last_opcode_pos as isize))
    }
}
unsafe fn js_is_live_code(s: *mut JSParseState) -> i32 {
    let op = get_prev_opcode((*s).cur_func) as u16;
    if [
        OP_tail_call,
        OP_tail_call_method,
        OP_return,
        OP_return_undef,
        OP_return_async,
        OP_throw,
        OP_throw_error,
        OP_goto,
        OP_ret,
    ]
    .contains(&op)
    {
        return 0;
    }
    #[cfg(feature = "short-opcodes")]
    if op == OP_goto8 || op == OP_goto16 {
        return 0;
    }
    1
}
unsafe fn emit_u8(s: *mut JSParseState, val: u8) {
    dbuf_putc(&mut (*(*s).cur_func).byte_code, val);
}
unsafe fn emit_u16(s: *mut JSParseState, val: u16) {
    dbuf_put_u16(&mut (*(*s).cur_func).byte_code, val);
}
unsafe fn emit_u32(s: *mut JSParseState, val: u32) {
    dbuf_put_u32(&mut (*(*s).cur_func).byte_code, val);
}
unsafe fn emit_source_pos(s: *mut JSParseState, source_ptr: *const u8) {
    let fd = (*s).cur_func;
    if (*fd).last_opcode_source_ptr != source_ptr {
        dbuf_putc(&mut (*fd).byte_code, OP_line_num as u8);
        dbuf_put_u32(
            &mut (*fd).byte_code,
            source_ptr.offset_from((*s).buf_start) as u32,
        );
        (*fd).last_opcode_source_ptr = source_ptr;
    }
}
unsafe fn emit_op(s: *mut JSParseState, val: u8) {
    let fd = (*s).cur_func;
    (*fd).last_opcode_pos = (*fd).byte_code.size as i32;
    dbuf_putc(&mut (*fd).byte_code, val);
}
unsafe fn emit_atom(s: *mut JSParseState, name: JSAtom) {
    let bc = &mut (*(*s).cur_func).byte_code;
    if dbuf_claim(bc, 4) != 0 {
        return;
    }
    put_u32(bc.buf.add(bc.size), JS_DupAtom((*s).ctx, name));
    bc.size += 4;
}
unsafe fn update_label(s: *mut JSFunctionDef, label: i32, delta: i32) -> i32 {
    assert!(label >= 0 && label < (*s).label_count);
    let ls = (*s).label_slots.offset(label as isize);
    (*ls).ref_count += delta;
    assert!((*ls).ref_count >= 0);
    (*ls).ref_count
}
unsafe fn new_label_fd(fd: *mut JSFunctionDef) -> i32 {
    if js_resize_array(
        (*fd).ctx,
        ptr::addr_of_mut!((*fd).label_slots).cast(),
        size_of::<LabelSlot>() as i32,
        &mut (*fd).label_size,
        (*fd).label_count + 1,
    ) != 0
    {
        return -1;
    }
    let label = (*fd).label_count;
    (*fd).label_count += 1;
    let ls = (*fd).label_slots.offset(label as isize);
    *ls = LabelSlot {
        ref_count: 0,
        pos: -1,
        pos2: -1,
        addr: -1,
        first_reloc: ptr::null_mut(),
    };
    label
}
unsafe fn new_label(s: *mut JSParseState) -> i32 {
    let label = new_label_fd((*s).cur_func);
    if label < 0 {
        dbuf_set_error(&mut (*(*s).cur_func).byte_code);
    }
    label
}
unsafe fn emit_label_raw(s: *mut JSParseState, label: i32) {
    emit_u8(s, OP_label as u8);
    emit_u32(s, label as u32);
    (*(*(*s).cur_func).label_slots.offset(label as isize)).pos =
        (*(*s).cur_func).byte_code.size as i32;
}
unsafe fn emit_label(s: *mut JSParseState, label: i32) -> i32 {
    if label < 0 {
        return -1;
    }
    emit_op(s, OP_label as u8);
    emit_u32(s, label as u32);
    (*(*(*s).cur_func).label_slots.offset(label as isize)).pos =
        (*(*s).cur_func).byte_code.size as i32;
    (*(*s).cur_func).byte_code.size as i32 - 4
}
unsafe fn emit_goto(s: *mut JSParseState, opcode: i32, mut label: i32) -> i32 {
    if js_is_live_code(s) == 0 {
        return -1;
    }
    if label < 0 {
        label = new_label(s);
        if label < 0 {
            return -1;
        }
    }
    emit_op(s, opcode as u8);
    emit_u32(s, label as u32);
    (*(*(*s).cur_func).label_slots.offset(label as isize)).ref_count += 1;
    label
}
unsafe fn cpool_add(s: *mut JSParseState, val: JSValue) -> i32 {
    let fd = (*s).cur_func;
    if js_resize_array(
        (*s).ctx,
        ptr::addr_of_mut!((*fd).cpool).cast(),
        size_of::<JSValue>() as i32,
        &mut (*fd).cpool_size,
        (*fd).cpool_count + 1,
    ) != 0
    {
        JS_FreeValue((*s).ctx, val);
        return -1;
    }
    *(*fd).cpool.offset((*fd).cpool_count as isize) = val;
    (*fd).cpool_count += 1;
    (*fd).cpool_count - 1
}
unsafe fn emit_push_const(s: *mut JSParseState, val: JSValueConst, as_atom: i32) -> i32 {
    if JS_VALUE_GET_TAG(val) == JS_TAG_STRING && as_atom != 0 {
        JS_DupValue((*s).ctx, val);
        let atom = JS_NewAtomStr((*s).ctx, JS_VALUE_GET_PTR(val).cast());
        if atom != JS_ATOM_NULL as u32 && __JS_AtomIsTaggedInt(atom) == 0 {
            emit_op(s, OP_push_atom_value as u8);
            emit_u32(s, atom);
            return 0;
        }
    }
    let idx = cpool_add(s, JS_DupValue((*s).ctx, val));
    if idx < 0 {
        return -1;
    }
    emit_op(s, OP_push_const as u8);
    emit_u32(s, idx as u32);
    0
}
unsafe fn find_arg(_ctx: *mut JSContext, fd: *mut JSFunctionDef, name: JSAtom) -> i32 {
    let mut i = (*fd).arg_count;
    // Each complete group is in bounds. Keep original descending comparisons
    // and latest duplicate precedence, using one checked offset for each group.
    while i >= 8 {
        i -= 8;
        let entries = &*(*fd).args.offset(i as isize).cast::<[JSVarDef; 8]>();
        if entries[7].var_name == name { return (i + 7) | ARGUMENT_VAR_OFFSET; }
        if entries[6].var_name == name { return (i + 6) | ARGUMENT_VAR_OFFSET; }
        if entries[5].var_name == name { return (i + 5) | ARGUMENT_VAR_OFFSET; }
        if entries[4].var_name == name { return (i + 4) | ARGUMENT_VAR_OFFSET; }
        if entries[3].var_name == name { return (i + 3) | ARGUMENT_VAR_OFFSET; }
        if entries[2].var_name == name { return (i + 2) | ARGUMENT_VAR_OFFSET; }
        if entries[1].var_name == name { return (i + 1) | ARGUMENT_VAR_OFFSET; }
        if entries[0].var_name == name { return (i) | ARGUMENT_VAR_OFFSET; }
    }
    while i >= 4 {
        i -= 4;
        let entries = &*(*fd).args.offset(i as isize).cast::<[JSVarDef; 4]>();
        if entries[3].var_name == name { return (i + 3) | ARGUMENT_VAR_OFFSET; }
        if entries[2].var_name == name { return (i + 2) | ARGUMENT_VAR_OFFSET; }
        if entries[1].var_name == name { return (i + 1) | ARGUMENT_VAR_OFFSET; }
        if entries[0].var_name == name { return i | ARGUMENT_VAR_OFFSET; }
    }
    while i > 0 {
        i -= 1;
        if (*(*fd).args.offset(i as isize)).var_name == name {
            return i | ARGUMENT_VAR_OFFSET;
        }
    }
    -1
}
unsafe fn find_var(ctx: *mut JSContext, fd: *mut JSFunctionDef, name: JSAtom) -> i32 {
    let mut i = (*fd).var_count;
    while i >= 8 {
        i -= 8;
        let entries = &*(*fd).vars.offset(i as isize).cast::<[JSVarDef; 8]>();
        if entries[7].var_name == name && entries[7].scope_level == 0 { return i + 7; }
        if entries[6].var_name == name && entries[6].scope_level == 0 { return i + 6; }
        if entries[5].var_name == name && entries[5].scope_level == 0 { return i + 5; }
        if entries[4].var_name == name && entries[4].scope_level == 0 { return i + 4; }
        if entries[3].var_name == name && entries[3].scope_level == 0 { return i + 3; }
        if entries[2].var_name == name && entries[2].scope_level == 0 { return i + 2; }
        if entries[1].var_name == name && entries[1].scope_level == 0 { return i + 1; }
        if entries[0].var_name == name && entries[0].scope_level == 0 { return i; }
    }
    while i >= 4 {
        i -= 4;
        let entries = &*(*fd).vars.offset(i as isize).cast::<[JSVarDef; 4]>();
        if entries[3].var_name == name && entries[3].scope_level == 0 { return i + 3; }
        if entries[2].var_name == name && entries[2].scope_level == 0 { return i + 2; }
        if entries[1].var_name == name && entries[1].scope_level == 0 { return i + 1; }
        if entries[0].var_name == name && entries[0].scope_level == 0 { return i; }
    }
    while i > 0 {
        i -= 1;
        let vd = &*(*fd).vars.offset(i as isize);
        if vd.var_name == name && vd.scope_level == 0 {
            return i;
        }
    }
    find_arg(ctx, fd, name)
}
unsafe fn find_var_in_scope(
    _ctx: *mut JSContext,
    fd: *mut JSFunctionDef,
    name: JSAtom,
    scope_level: i32,
) -> i32 {
    let mut idx = (*(*fd).scopes.offset(scope_level as isize)).first;
    while idx >= 0 {
        let vd = &*(*fd).vars.offset(idx as isize);
        if vd.scope_level != scope_level {
            break;
        }
        if vd.var_name == name {
            return idx;
        }
        idx = vd.scope_next;
    }
    -1
}
unsafe fn is_child_scope(
    _ctx: *mut JSContext,
    fd: *mut JSFunctionDef,
    mut scope: i32,
    parent_scope: i32,
) -> i32 {
    while scope >= 0 {
        if scope == parent_scope {
            return 1;
        }
        scope = (*(*fd).scopes.offset(scope as isize)).parent;
    }
    0
}
unsafe fn find_var_in_child_scope(
    ctx: *mut JSContext,
    fd: *mut JSFunctionDef,
    name: JSAtom,
    scope_level: i32,
) -> i32 {
    for i in 0..(*fd).var_count {
        let vd = &*(*fd).vars.offset(i as isize);
        if vd.var_name == name
            && vd.scope_level == 0
            && is_child_scope(ctx, fd, vd.scope_next, scope_level) != 0
        {
            return i;
        }
    }
    -1
}
// Parse-program-local root declaration index. Only atom numbers and array
// positions survive a lookup; returned pointers always use the current array.
// Child function definitions are deliberately excluded from this lifetime.
#[derive(Default)]
struct CompilerGlobalVarIndex {
    function: usize,
    observed_count: i32,
    scanned_count: i32,
    positions: std::collections::HashMap<JSAtom, i32>,
}
std::thread_local! {
    static COMPILER_GLOBAL_VAR_INDEXES: std::cell::RefCell<Vec<CompilerGlobalVarIndex>> = const { std::cell::RefCell::new(Vec::new()) };
}
struct CompilerGlobalVarScope;
impl CompilerGlobalVarScope {
    fn new(fd: *mut JSFunctionDef) -> Self {
        COMPILER_GLOBAL_VAR_INDEXES.with(|indexes| indexes.borrow_mut().push(CompilerGlobalVarIndex { function: fd as usize, ..Default::default() }));
        Self
    }
}
impl Drop for CompilerGlobalVarScope {
    fn drop(&mut self) {
        COMPILER_GLOBAL_VAR_INDEXES.with(|indexes| { indexes.borrow_mut().pop(); });
    }
}
unsafe fn compiler_find_global_var(fd: *mut JSFunctionDef, name: JSAtom) -> Option<*mut JSGlobalVar> {
    COMPILER_GLOBAL_VAR_INDEXES.with(|indexes| {
        let mut indexes = indexes.borrow_mut();
        let index = indexes.last_mut()?;
        if index.function != fd as usize { return None; }
        let count = (*fd).global_var_count;
        // Observe small-array fallback calls too: a truncated suffix must not
        // survive if this same scope later grows a different suffix.
        if count < index.observed_count { index.positions.clear(); index.scanned_count = 0; }
        index.observed_count = count;
        if count < 64 { return None; }
        for position in index.scanned_count..count {
            let candidate = (*(*fd).global_vars.offset(position as isize)).var_name;
            index.positions.entry(candidate).or_insert(position);
        }
        index.scanned_count = count;
        Some(index.positions.get(&name).map_or(ptr::null_mut(), |position| (*fd).global_vars.offset(*position as isize)))
    })
}
unsafe fn find_global_var(fd: *mut JSFunctionDef, name: JSAtom) -> *mut JSGlobalVar {
    if let Some(found) = compiler_find_global_var(fd, name) { return found; }
    for i in 0..(*fd).global_var_count {
        let hf = (*fd).global_vars.offset(i as isize);
        if (*hf).var_name == name {
            return hf;
        }
    }
    ptr::null_mut()
}
unsafe fn find_lexical_global_var(fd: *mut JSFunctionDef, name: JSAtom) -> *mut JSGlobalVar {
    let hf = find_global_var(fd, name);
    if !hf.is_null() && (*hf).is_lexical() != 0 {
        hf
    } else {
        ptr::null_mut()
    }
}
unsafe fn find_lexical_decl(
    _ctx: *mut JSContext,
    fd: *mut JSFunctionDef,
    name: JSAtom,
    mut scope_idx: i32,
    check_catch_var: i32,
) -> i32 {
    while scope_idx >= 0 {
        let vd = &*(*fd).vars.offset(scope_idx as isize);
        if vd.var_name == name
            && (vd.is_lexical() != 0
                || (vd.var_kind() == JS_VAR_CATCH as u8 && check_catch_var != 0))
        {
            return scope_idx;
        }
        scope_idx = vd.scope_next;
    }
    if (*fd).is_eval != 0
        && (*fd).eval_type == JS_EVAL_TYPE_GLOBAL
        && !find_lexical_global_var(fd, name).is_null()
    {
        return GLOBAL_VAR_OFFSET;
    }
    -1
}
unsafe fn push_scope(s: *mut JSParseState) -> i32 {
    let fd = (*s).cur_func;
    if fd.is_null() {
        return 0;
    }
    let scope = (*fd).scope_count;
    if (*fd).scope_count + 1 > (*fd).scope_size {
        let mut new_size = ((*fd).scope_count + 1).max((*fd).scope_size * 3 / 2);
        let mut slack = 0usize;
        let from_static = (*fd).scopes == (*fd).def_scope_array.as_mut_ptr();
        let new_buf = js_realloc2(
            (*s).ctx,
            if from_static {
                ptr::null_mut()
            } else {
                (*fd).scopes.cast()
            },
            new_size as usize * size_of::<JSVarScope>(),
            &mut slack,
        )
        .cast::<JSVarScope>();
        if new_buf.is_null() {
            return -1;
        }
        if from_static {
            ptr::copy_nonoverlapping((*fd).scopes, new_buf, (*fd).scope_count as usize);
        }
        new_size += (slack / size_of::<JSVarScope>()) as i32;
        (*fd).scopes = new_buf;
        (*fd).scope_size = new_size;
    }
    (*fd).scope_count += 1;
    *(*fd).scopes.offset(scope as isize) = JSVarScope {
        parent: (*fd).scope_level,
        first: (*fd).scope_first,
    };
    emit_op(s, OP_enter_scope as u8);
    emit_u16(s, scope as u16);
    (*fd).scope_level = scope;
    scope
}
unsafe fn get_first_lexical_var(fd: *mut JSFunctionDef, mut scope: i32) -> i32 {
    while scope >= 0 {
        let idx = (*(*fd).scopes.offset(scope as isize)).first;
        if idx >= 0 {
            return idx;
        }
        scope = (*(*fd).scopes.offset(scope as isize)).parent;
    }
    -1
}
unsafe fn pop_scope(s: *mut JSParseState) {
    let fd = (*s).cur_func;
    if !fd.is_null() {
        let scope = (*fd).scope_level;
        emit_op(s, OP_leave_scope as u8);
        emit_u16(s, scope as u16);
        (*fd).scope_level = (*(*fd).scopes.offset(scope as isize)).parent;
        (*fd).scope_first = get_first_lexical_var(fd, (*fd).scope_level);
    }
}
unsafe fn close_scopes(s: *mut JSParseState, mut scope: i32, scope_stop: i32) {
    while scope > scope_stop {
        emit_op(s, OP_leave_scope as u8);
        emit_u16(s, scope as u16);
        scope = (*(*(*s).cur_func).scopes.offset(scope as isize)).parent;
    }
}
unsafe fn add_var(ctx: *mut JSContext, fd: *mut JSFunctionDef, name: JSAtom) -> i32 {
    if (*fd).var_count >= JS_MAX_LOCAL_VARS {
        JS_ThrowInternalError(ctx, c"too many local variables".as_ptr());
        return -1;
    }
    if js_resize_array(
        ctx,
        ptr::addr_of_mut!((*fd).vars).cast(),
        size_of::<JSVarDef>() as i32,
        &mut (*fd).var_size,
        (*fd).var_count + 1,
    ) != 0
    {
        return -1;
    }
    let vd = (*fd).vars.offset((*fd).var_count as isize);
    (*fd).var_count += 1;
    ptr::write_bytes(vd, 0, 1);
    (*vd).var_name = JS_DupAtom(ctx, name);
    (*vd).func_pool_idx = -1;
    (*fd).var_count - 1
}
unsafe fn add_scope_var(
    ctx: *mut JSContext,
    fd: *mut JSFunctionDef,
    name: JSAtom,
    var_kind: JSVarKindEnum,
) -> i32 {
    let idx = add_var(ctx, fd, name);
    if idx >= 0 {
        let vd = &mut *(*fd).vars.offset(idx as isize);
        vd.set_var_kind(var_kind as u8);
        vd.scope_level = (*fd).scope_level;
        vd.scope_next = (*fd).scope_first;
        (*(*fd).scopes.offset((*fd).scope_level as isize)).first = idx;
        (*fd).scope_first = idx;
    }
    idx
}
unsafe fn add_func_var(ctx: *mut JSContext, fd: *mut JSFunctionDef, name: JSAtom) -> i32 {
    let mut idx = (*fd).func_var_idx;
    if idx < 0 {
        idx = add_var(ctx, fd, name);
        if idx >= 0 {
            (*fd).func_var_idx = idx;
            let vd = &mut *(*fd).vars.offset(idx as isize);
            vd.set_var_kind(JS_VAR_FUNCTION_NAME as u8);
            if (*fd).js_mode & JS_MODE_STRICT as u8 != 0 {
                vd.set_is_const(1);
            }
        }
    }
    idx
}
unsafe fn add_arguments_var(ctx: *mut JSContext, fd: *mut JSFunctionDef) -> i32 {
    let mut idx = (*fd).arguments_var_idx;
    if idx < 0 {
        idx = add_var(ctx, fd, JS_ATOM_arguments);
        if idx >= 0 {
            (*fd).arguments_var_idx = idx;
        }
    }
    idx
}
unsafe fn add_arguments_arg(ctx: *mut JSContext, fd: *mut JSFunctionDef) -> i32 {
    if (*fd).arguments_arg_idx < 0 {
        let mut idx = find_var_in_scope(ctx, fd, JS_ATOM_arguments, ARG_SCOPE_INDEX);
        if idx < 0 {
            idx = add_var(ctx, fd, JS_ATOM_arguments);
            if idx < 0 {
                return -1;
            }
            let vd = &mut *(*fd).vars.offset(idx as isize);
            vd.scope_next = (*(*fd).scopes.offset(ARG_SCOPE_INDEX as isize)).first;
            (*(*fd).scopes.offset(ARG_SCOPE_INDEX as isize)).first = idx;
            vd.scope_level = ARG_SCOPE_INDEX;
            vd.set_is_lexical(1);
            (*fd).arguments_arg_idx = idx;
        }
    }
    0
}
unsafe fn add_arg(ctx: *mut JSContext, fd: *mut JSFunctionDef, name: JSAtom) -> i32 {
    if (*fd).arg_count >= JS_MAX_LOCAL_VARS {
        JS_ThrowInternalError(ctx, c"too many arguments".as_ptr());
        return -1;
    }
    if js_resize_array(
        ctx,
        ptr::addr_of_mut!((*fd).args).cast(),
        size_of::<JSVarDef>() as i32,
        &mut (*fd).arg_size,
        (*fd).arg_count + 1,
    ) != 0
    {
        return -1;
    }
    let vd = (*fd).args.offset((*fd).arg_count as isize);
    (*fd).arg_count += 1;
    ptr::write_bytes(vd, 0, 1);
    (*vd).var_name = JS_DupAtom(ctx, name);
    (*vd).func_pool_idx = -1;
    (*fd).arg_count - 1
}
unsafe fn add_global_var(
    ctx: *mut JSContext,
    s: *mut JSFunctionDef,
    name: JSAtom,
) -> *mut JSGlobalVar {
    if js_resize_array(
        ctx,
        ptr::addr_of_mut!((*s).global_vars).cast(),
        size_of::<JSGlobalVar>() as i32,
        &mut (*s).global_var_size,
        (*s).global_var_count + 1,
    ) != 0
    {
        return ptr::null_mut();
    }
    let hf = (*s).global_vars.offset((*s).global_var_count as isize);
    (*s).global_var_count += 1;
    (*hf).cpool_idx = -1;
    (*hf).flags = 0;
    (*hf).scope_level = (*s).scope_level;
    (*hf).var_name = JS_DupAtom(ctx, name);
    hf
}
type JSVarDefEnum = u32;
const JS_VAR_DEF_WITH: JSVarDefEnum = 0;
const JS_VAR_DEF_LET: JSVarDefEnum = 1;
const JS_VAR_DEF_CONST: JSVarDefEnum = 2;
const JS_VAR_DEF_FUNCTION_DECL: JSVarDefEnum = 3;
const JS_VAR_DEF_NEW_FUNCTION_DECL: JSVarDefEnum = 4;
const JS_VAR_DEF_CATCH: JSVarDefEnum = 5;
const JS_VAR_DEF_VAR: JSVarDefEnum = 6;
unsafe fn define_var(
    s: *mut JSParseState,
    fd: *mut JSFunctionDef,
    name: JSAtom,
    var_def_type: JSVarDefEnum,
) -> i32 {
    let ctx = (*s).ctx;
    match var_def_type {
        JS_VAR_DEF_WITH => add_scope_var(ctx, fd, name, JS_VAR_NORMAL),
        JS_VAR_DEF_LET
        | JS_VAR_DEF_CONST
        | JS_VAR_DEF_FUNCTION_DECL
        | JS_VAR_DEF_NEW_FUNCTION_DECL => {
            let idx = find_lexical_decl(ctx, fd, name, (*fd).scope_first, 1);
            if idx >= 0 {
                let invalid = if idx < GLOBAL_VAR_OFFSET {
                    let vd = &*(*fd).vars.offset(idx as isize);
                    if vd.scope_level == (*fd).scope_level {
                        !((*fd).js_mode & JS_MODE_STRICT as u8 == 0
                            && var_def_type == JS_VAR_DEF_FUNCTION_DECL
                            && vd.var_kind() == JS_VAR_FUNCTION_DECL as u8)
                    } else {
                        vd.var_kind() == JS_VAR_CATCH as u8
                            && vd.scope_level + 2 == (*fd).scope_level
                    }
                } else {
                    (*fd).scope_level == (*fd).body_scope
                };
                if invalid {
                    return js_parse_error(
                        s,
                        c"invalid redefinition of lexical identifier".as_ptr(),
                    );
                }
            }
            if var_def_type != JS_VAR_DEF_FUNCTION_DECL
                && var_def_type != JS_VAR_DEF_NEW_FUNCTION_DECL
                && (*fd).scope_level == (*fd).body_scope
                && find_arg(ctx, fd, name) >= 0
            {
                return js_parse_error(s, c"invalid redefinition of parameter name".as_ptr());
            }
            if find_var_in_child_scope(ctx, fd, name, (*fd).scope_level) >= 0 {
                return js_parse_error(s, c"invalid redefinition of a variable".as_ptr());
            }
            if (*fd).is_global_var != 0 {
                let hf = find_global_var(fd, name);
                if !hf.is_null()
                    && is_child_scope(ctx, fd, (*hf).scope_level, (*fd).scope_level) != 0
                {
                    return js_parse_error(
                        s,
                        c"invalid redefinition of global identifier".as_ptr(),
                    );
                }
            }
            if (*fd).is_eval != 0
                && ((*fd).eval_type == JS_EVAL_TYPE_GLOBAL
                    || (*fd).eval_type == JS_EVAL_TYPE_MODULE)
                && (*fd).scope_level == (*fd).body_scope
            {
                let hf = add_global_var(ctx, fd, name);
                if hf.is_null() {
                    return -1;
                }
                (*hf).set_is_lexical(1);
                (*hf).set_is_const((var_def_type == JS_VAR_DEF_CONST) as u8);
                GLOBAL_VAR_OFFSET
            } else {
                let var_kind = if var_def_type == JS_VAR_DEF_FUNCTION_DECL {
                    JS_VAR_FUNCTION_DECL
                } else if var_def_type == JS_VAR_DEF_NEW_FUNCTION_DECL {
                    JS_VAR_NEW_FUNCTION_DECL
                } else {
                    JS_VAR_NORMAL
                };
                let idx = add_scope_var(ctx, fd, name, var_kind);
                if idx >= 0 {
                    let vd = &mut *(*fd).vars.offset(idx as isize);
                    vd.set_is_lexical(1);
                    vd.set_is_const((var_def_type == JS_VAR_DEF_CONST) as u8);
                }
                idx
            }
        }
        JS_VAR_DEF_CATCH => add_scope_var(ctx, fd, name, JS_VAR_CATCH),
        JS_VAR_DEF_VAR => {
            if find_lexical_decl(ctx, fd, name, (*fd).scope_first, 0) >= 0 {
                return js_parse_error(s, c"invalid redefinition of lexical identifier".as_ptr());
            }
            if (*fd).is_global_var != 0 {
                let hf = find_global_var(fd, name);
                if !hf.is_null()
                    && (*hf).is_lexical() != 0
                    && (*hf).scope_level == (*fd).scope_level
                    && (*fd).eval_type == JS_EVAL_TYPE_MODULE
                {
                    return js_parse_error(
                        s,
                        c"invalid redefinition of lexical identifier".as_ptr(),
                    );
                }
                if add_global_var(ctx, fd, name).is_null() {
                    return -1;
                }
                GLOBAL_VAR_OFFSET
            } else {
                let idx = find_var(ctx, fd, name);
                if idx >= 0 {
                    return idx;
                }
                let idx = add_var(ctx, fd, name);
                if idx >= 0 {
                    if name == JS_ATOM_arguments && (*fd).has_arguments_binding != 0 {
                        (*fd).arguments_var_idx = idx;
                    }
                    (*(*fd).vars.offset(idx as isize)).scope_next = (*fd).scope_level;
                }
                idx
            }
        }
        _ => unreachable!("invalid JSVarDefEnum"),
    }
}
unsafe fn add_private_class_field(
    s: *mut JSParseState,
    fd: *mut JSFunctionDef,
    name: JSAtom,
    var_kind: JSVarKindEnum,
    is_static: i32,
) -> i32 {
    let idx = add_scope_var((*s).ctx, fd, name, var_kind);
    if idx < 0 {
        return idx;
    }
    let vd = &mut *(*fd).vars.offset(idx as isize);
    vd.set_is_lexical(1);
    vd.set_is_const(1);
    vd.set_is_static_private(is_static as u8);
    idx
}

// Compiler-pass-local acceleration of quickjs.c:32821-32840/32992-33014.
// Scope chains retain their first-match order and negative argument sentinel.
// Indexes own atom numbers and array positions, never JS heap pointers. With
// environments run the original lookup, including every var_object_test.
#[derive(Default)]
struct CompilerScopeLookup {
    own_vars: CompilerOwnVarIndex,
    closures: std::collections::HashMap<(usize, JSAtom, bool), CompilerClosureLookup>,
    closure_indexes: std::collections::HashMap<usize, CompilerClosureIndex>,
    chains: std::collections::HashMap<(usize, i32, i32), Option<(std::collections::HashMap<JSAtom, i32>, i32)>>,
}
#[derive(Default)]
struct CompilerClosureLookup {
    scanned_count: i32,
    position: Option<i32>,
    has_environment: bool,
}
// A large closure array is indexed only after repeated scans demonstrate that
// this compiler pass actually needs it. Small child functions keep the demand
// lookup above: eagerly indexing their entire parent was a measured regression.
#[derive(Default)]
struct CompilerClosureIndex {
    observed_count: i32,
    scan_queries: usize,
    scanned_count: i32,
    positions: std::collections::HashMap<JSAtom, i32>,
    first_environment: Option<i32>,
}
impl CompilerClosureIndex {
    unsafe fn find(&mut self, fd: *mut JSFunctionDef, name: JSAtom, pseudo: bool) -> Option<i32> {
        let count = (*fd).closure_var_count;
        if count < self.scanned_count { *self = Self::default(); }
        // Array growth is append-only in this pass. Keep the earliest name and
        // environment positions exactly as the source's forward scan does.
        for index in self.scanned_count..count {
            let candidate = (*(*fd).closure_var.offset(index as isize)).var_name;
            self.positions.entry(candidate).or_insert(index);
            if self.first_environment.is_none() && (candidate == crate::quickjs_atom::JS_ATOM__var_
                || candidate == crate::quickjs_atom::JS_ATOM__arg_var_
                || candidate == crate::quickjs_atom::JS_ATOM__with_) {
                self.first_environment = Some(index);
            }
        }
        self.scanned_count = count;
        let position = self.positions.get(&name).copied();
        if !pseudo {
            if let Some(environment) = self.first_environment {
                if position.is_none_or(|position| environment < position) { return None; }
            }
        }
        Some(position.unwrap_or(count))
    }
}
// Resolution of the current function only: parsing and ancestor lookups retain
// their original leaf. This pass only appends pseudo bindings to current vars;
// existing name/scope pairs stay fixed, while capture flags may change.
#[derive(Default)]
struct CompilerOwnVarIndex {
    function: usize,
    count: i32,
    scan_queries: usize,
    positions: Vec<(JSAtom, i32)>,
}
impl CompilerOwnVarIndex {
    unsafe fn find(&mut self, ctx: *mut JSContext, fd: *mut JSFunctionDef, name: JSAtom) -> i32 {
        let count = (*fd).var_count;
        if count < 128 { return find_var(ctx, fd, name); }
        if self.function != fd as usize || count < self.count { *self = Self::default(); }
        self.function = fd as usize;
        self.scan_queries += 1;
        if self.scan_queries < 16 { return find_var(ctx, fd, name); }
        if count != self.count {
            self.positions.clear();
            // This acceleration is optional, including under allocator failure.
            if self.positions.try_reserve(count as usize).is_err() {
                return find_var(ctx, fd, name);
            }
            for index in 0..count {
                let var = &*(*fd).vars.offset(index as isize);
                if var.scope_level == 0 { self.positions.push((var.var_name, index)); }
            }
            self.positions.sort_unstable();
            self.count = count;
        }
        let end = self.positions.partition_point(|&(atom, _)| atom <= name);
        if end > 0 {
            let (atom, index) = self.positions[end - 1];
            if atom == name { return index; } // latest eligible duplicate
        }
        find_arg(ctx, fd, name)
    }
}
impl CompilerScopeLookup {
    unsafe fn find_function_var(&mut self, ctx: *mut JSContext, fd: *mut JSFunctionDef, name: JSAtom) -> i32 {
        self.own_vars.find(ctx, fd, name)
    }
    unsafe fn find_closure(&mut self, fd: *mut JSFunctionDef, name: JSAtom, pseudo: bool) -> Option<i32> {
        // quickjs.c:33074-33116 first-name order. During this pass only
        // add_closure_var appends names; array reallocation preserves positions.
        // Begin with queried names only; repeated linear scans may promote a
        // large array to an index. One-off parent lookups keep the demand path.
        let count = (*fd).closure_var_count;
        let index = self.closure_indexes.entry(fd as usize).or_default();
        // Count every observation, including memo hits, so a smaller array
        // invalidates both the promoted index and old per-name suffix hits.
        // The source pass appends only; this also keeps reset/reuse defensive.
        if count < index.observed_count {
            *index = CompilerClosureIndex::default();
            self.closures.retain(|(function, _, _), _| *function != fd as usize);
        }
        index.observed_count = count;
        // Pseudo lookups skip environment tests; their successful position
        // cannot be reused for an ordinary lookup of the same atom.
        let lookup = self.closures.entry((fd as usize, name, pseudo)).or_default();
        if count < lookup.scanned_count { *lookup = CompilerClosureLookup::default(); }
        if let Some(position) = lookup.position { return Some(position); }
        if lookup.has_environment && !pseudo { return None; }
        index.scan_queries += 1;
        // Each prior scan has paid the linear cost. Amortize indexing only for
        // sufficiently large arrays with enough real scan queries; this keeps
        // one-off parent lookups on the cheap, original demand path.
        if count >= 64 && index.scan_queries >= 32 {
            let position = index.find(fd, name, pseudo);
            lookup.scanned_count = count;
            match position {
                Some(position) if position < count => lookup.position = Some(position),
                None => lookup.has_environment = true,
                _ => {}
            }
            return position;
        }
        for index in lookup.scanned_count..count {
            let candidate = (*(*fd).closure_var.offset(index as isize)).var_name;
            if candidate == name { lookup.scanned_count = count; lookup.position = Some(index); return Some(index); }
            if !pseudo && (candidate == crate::quickjs_atom::JS_ATOM__var_
                || candidate == crate::quickjs_atom::JS_ATOM__arg_var_
                || candidate == crate::quickjs_atom::JS_ATOM__with_) {
                lookup.has_environment = true;
                return None;
            }
        }
        lookup.scanned_count = count;
        Some(count)
    }

    unsafe fn find(&mut self, fd: *mut JSFunctionDef, first: i32, name: JSAtom) -> Option<i32> {
        if first < 0 { return Some(first); }
        // Resolve bounded short chains without allocating their atom index.
        // Reaching a with environment retains every original var_object_test.
        let mut short_index = first;
        for _ in 0..8 {
            if short_index < 0 { return Some(short_index); }
            let var = &*(*fd).vars.offset(short_index as isize);
            if var.var_name == crate::quickjs_atom::JS_ATOM__with_ { return None; }
            if var.var_name == name { return Some(short_index); }
            short_index = var.scope_next;
        }
        if short_index < 0 { return Some(short_index); }
        // Pseudo binding creation may grow/reallocate vars during resolution.
        // The count and chain head form the revision; positions stay stable.
        let chain = self.chains.entry((fd as usize, (*fd).var_count, first)).or_insert_with(|| {
            let mut positions = std::collections::HashMap::new();
            let mut index = first;
            while index >= 0 {
                let var = &*(*fd).vars.offset(index as isize);
                if var.var_name == crate::quickjs_atom::JS_ATOM__with_ {
                    return None;
                }
                positions.entry(var.var_name).or_insert(index);
                index = var.scope_next;
            }
            Some((positions, index))
        });
        chain.as_ref().map(|(positions, sentinel)| positions.get(&name).copied().unwrap_or(*sentinel))
    }
}

#[cfg(test)]
mod compiler_adaptive_closure_tests {
    use super::*;

    unsafe fn source_scan(fd: &JSFunctionDef, name: JSAtom, pseudo: bool) -> Option<i32> {
        for index in 0..fd.closure_var_count {
            let candidate = (*fd.closure_var.offset(index as isize)).var_name;
            if candidate == name { return Some(index); }
            if !pseudo && [crate::quickjs_atom::JS_ATOM__var_, crate::quickjs_atom::JS_ATOM__arg_var_, crate::quickjs_atom::JS_ATOM__with_].contains(&candidate) {
                return None;
            }
        }
        Some(fd.closure_var_count)
    }

    #[test]
    fn adaptive_closure_preserves_first_name_and_environment_order() { unsafe {
        for environment in [None, Some(0), Some(17), Some(63), Some(127)] {
            for environment_name in [crate::quickjs_atom::JS_ATOM__var_, crate::quickjs_atom::JS_ATOM__arg_var_, crate::quickjs_atom::JS_ATOM__with_] {
                let mut entries: Vec<JSClosureVar> = (0..128).map(|index| {
                    let mut entry: JSClosureVar = core::mem::zeroed();
                    entry.var_name = 1000 + (index % 81) as JSAtom;
                    entry
                }).collect();
                if let Some(index) = environment { entries[index].var_name = environment_name; }
                let mut fd: JSFunctionDef = core::mem::zeroed();
                fd.closure_var = entries.as_mut_ptr(); fd.closure_var_count = entries.len() as i32;
                for modes in [[false, true], [true, false]] {
                    let mut lookup = CompilerScopeLookup::default();
                    for _ in 0..2 {
                        for name in 990..1100 {
                            for pseudo in modes {
                                assert_eq!(lookup.find_closure(&mut fd, name, pseudo), source_scan(&fd, name, pseudo), "environment={environment:?} name={name} pseudo={pseudo}");
                            }
                        }
                    }
                }
                // Check the indexed source operation itself across both modes,
                // including names equal to the environment atom.
                let mut index = CompilerClosureIndex::default();
                for pseudo in [false, true] {
                    for name in [environment_name, 1000, 1080, 1099] {
                        assert_eq!(index.find(&mut fd, name, pseudo), source_scan(&fd, name, pseudo));
                    }
                }
            }
        }
    } }

    #[test]
    fn adaptive_closure_resets_index_and_query_metadata_on_shrink() { unsafe {
        let mut entries: Vec<JSClosureVar> = (0..100).map(|position| {
            let mut entry: JSClosureVar = core::mem::zeroed(); entry.var_name = 1000 + position as JSAtom; entry
        }).collect();
        let mut fd: JSFunctionDef = core::mem::zeroed();
        fd.closure_var = entries.as_mut_ptr(); fd.closure_var_count = 100;
        let mut lookup = CompilerScopeLookup::default();
        for name in 1000..1100 { assert_eq!(lookup.find_closure(&mut fd, name, false), source_scan(&fd, name, false)); }
        let key = &mut fd as *mut JSFunctionDef as usize;
        assert_eq!(lookup.closure_indexes[&key].scanned_count, 100);
        // Observing shrink through an old successful memo hit must still reset
        // the table; then a different appended suffix cannot return old names.
        fd.closure_var_count = 20;
        assert_eq!(lookup.find_closure(&mut fd, 1000, false), Some(0));
        assert_eq!(lookup.closure_indexes[&key].scanned_count, 0);
        assert_eq!(lookup.closure_indexes[&key].scan_queries, 1);
        for position in 20..100 { entries[position].var_name = 2000 + position as JSAtom; }
        fd.closure_var_count = 100;
        for name in 1000..2120 { assert_eq!(lookup.find_closure(&mut fd, name, false), source_scan(&fd, name, false)); }
    } }

    #[test]
    fn adaptive_closure_extends_misses_after_array_reallocation() { unsafe {
        let mut entries: Vec<JSClosureVar> = (0..80).map(|index| {
            let mut entry: JSClosureVar = core::mem::zeroed(); entry.var_name = 1000 + index as JSAtom; entry
        }).collect();
        let mut fd: JSFunctionDef = core::mem::zeroed();
        fd.closure_var = entries.as_mut_ptr(); fd.closure_var_count = entries.len() as i32;
        let mut lookup = CompilerScopeLookup::default();
        for name in 1100..1140 { assert_eq!(lookup.find_closure(&mut fd, name, false), Some(80)); }
        assert_eq!(lookup.closure_indexes[&(&mut fd as *mut JSFunctionDef as usize)].scanned_count, 80);
        for name in [1107, 1000, 1139, 1107] {
            let mut entry: JSClosureVar = core::mem::zeroed(); entry.var_name = name; entries.push(entry);
            fd.closure_var = entries.as_mut_ptr(); fd.closure_var_count = entries.len() as i32;
            for query in [1000, 1107, 1139, 1140] {
                assert_eq!(lookup.find_closure(&mut fd, query, false), source_scan(&fd, query, false));
            }
        }
    } }
}

#[cfg(test)]
mod compiler_global_var_index_tests {
    use super::*;
    unsafe fn globals(first: JSAtom, count: usize) -> Vec<JSGlobalVar> {
        (0..count).map(|position| { let mut entry: JSGlobalVar = core::mem::zeroed(); entry.var_name = first + position as JSAtom; entry }).collect()
    }
    #[test]
    fn global_var_index_preserves_nested_scope_and_reallocation() { unsafe {
        let mut outer_entries = globals(1000, 80);
        let mut outer: JSFunctionDef = core::mem::zeroed();
        outer.global_vars = outer_entries.as_mut_ptr(); outer.global_var_count = outer_entries.len() as i32;
        let mut inner_entries = globals(2000, 100);
        let mut inner: JSFunctionDef = core::mem::zeroed();
        inner.global_vars = inner_entries.as_mut_ptr(); inner.global_var_count = inner_entries.len() as i32;
        assert!(compiler_find_global_var(&mut outer, 1007).is_none());
        {
            let _outer_scope = CompilerGlobalVarScope::new(&mut outer);
            assert_eq!(compiler_find_global_var(&mut outer, 1007), Some(outer.global_vars.add(7)));
            assert_eq!(compiler_find_global_var(&mut outer, 1080), Some(ptr::null_mut()));
            {
                let _inner_scope = CompilerGlobalVarScope::new(&mut inner);
                assert!(compiler_find_global_var(&mut outer, 1007).is_none());
                assert_eq!(compiler_find_global_var(&mut inner, 2009), Some(inner.global_vars.add(9)));
            }
            assert_eq!(compiler_find_global_var(&mut outer, 1007), Some(outer.global_vars.add(7)));
            // Force array reallocation and include a duplicate: the original
            // forward lookup returns the earliest duplicate, as does the index.
            outer_entries.extend(globals(1000, 80));
            outer_entries.extend(globals(1080, 10));
            outer.global_vars = outer_entries.as_mut_ptr(); outer.global_var_count = outer_entries.len() as i32;
            for name in 990..1100 {
                let found = find_global_var(&mut outer, name);
                let position = outer_entries.iter().position(|entry| entry.var_name == name);
                assert_eq!(found, position.map_or(ptr::null_mut(), |position| outer.global_vars.add(position)));
            }
            outer.global_var_count = 20;
            assert!(compiler_find_global_var(&mut outer, 1007).is_none());
            for position in 20..outer_entries.len() { outer_entries[position].var_name = 3000 + position as JSAtom; }
            outer.global_var_count = outer_entries.len() as i32;
            for name in [1007, 1079, 1080, 3020, 3169, 3999] {
                let found = find_global_var(&mut outer, name);
                let position = outer_entries.iter().position(|entry| entry.var_name == name);
                assert_eq!(found, position.map_or(ptr::null_mut(), |position| outer.global_vars.add(position)));
            }
        }
        assert!(compiler_find_global_var(&mut outer, 1007).is_none());
    } }
}

#[cfg(test)]
mod compiler_grouped_lookup_boundary_tests {
    use super::*;
    fn entries(count: usize, scopes: bool) -> Vec<JSVarDef> {
        (0..count).map(|index| JSVarDef {
            var_name: ((index * 5 + count) % 7 + 1) as JSAtom,
            scope_level: if scopes && index % 3 == 0 { 2 } else { 0 },
            scope_next: -1, flags: 0, var_ref_idx: 0, func_pool_idx: -1,
        }).collect()
    }
    #[test]
    fn compiler_grouped_lookup_tails_duplicates_and_arg_fallback() { unsafe {
        // Exact allocation sizes expose complete groups and every short tail;
        // repeated names include newest lexical entries hiding older globals.
        for var_count in 0..65 {
            for arg_count in 0..13 {
                let mut vars = entries(var_count, true);
                let mut args = entries(arg_count, false);
                let mut fd: JSFunctionDef = core::mem::zeroed();
                fd.vars = if vars.is_empty() { ptr::null_mut() } else { vars.as_mut_ptr() };
                fd.args = if args.is_empty() { ptr::null_mut() } else { args.as_mut_ptr() };
                fd.var_count = var_count as i32; fd.arg_count = arg_count as i32;
                for name in 0..10 {
                    let arg = args.iter().rposition(|entry| entry.var_name == name)
                        .map_or(-1, |index| index as i32 | ARGUMENT_VAR_OFFSET);
                    let expected = vars.iter().rposition(|entry| entry.var_name == name && entry.scope_level == 0)
                        .map_or(arg, |index| index as i32);
                    assert_eq!(find_arg(ptr::null_mut(), &mut fd, name), arg,
                        "arg_count={arg_count} name={name}");
                    assert_eq!(find_var(ptr::null_mut(), &mut fd, name), expected,
                        "var_count={var_count} arg_count={arg_count} name={name}");
                }
            }
        }
    } }
    #[test]
    fn compiler_grouped_lookup_nonpositive_counts_keep_empty_scan() { unsafe {
        // Original signed scans never touch pointers for nonpositive counts.
        let mut fd: JSFunctionDef = core::mem::zeroed();
        for count in [-1, -4, i32::MIN, 0] {
            fd.var_count = count; fd.arg_count = count;
            assert_eq!(find_arg(ptr::null_mut(), &mut fd, 1), -1);
            assert_eq!(find_var(ptr::null_mut(), &mut fd, 1), -1);
        }
    } }
}
