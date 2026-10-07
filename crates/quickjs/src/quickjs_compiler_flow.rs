// quickjs.c:28201..28367. Structured control-flow unwind and return emission. MIT.
use crate::quickjs_atom::JS_ATOM_return;
unsafe fn push_break_entry(
    fd: *mut JSFunctionDef,
    be: *mut BlockEnv,
    label_name: JSAtom,
    label_break: i32,
    label_cont: i32,
    drop_count: i32,
) {
    (*be).prev = (*fd).top_break;
    (*fd).top_break = be;
    (*be).label_name = label_name;
    (*be).label_break = label_break;
    (*be).label_cont = label_cont;
    (*be).drop_count = drop_count;
    (*be).label_finally = -1;
    (*be).scope_level = (*fd).scope_level;
    (*be).set_has_iterator(0);
    (*be).set_is_regular_stmt(0);
}
unsafe fn pop_break_entry(fd: *mut JSFunctionDef) {
    (*fd).top_break = (*(*fd).top_break).prev;
}
unsafe fn emit_break(s: *mut JSParseState, name: JSAtom, is_cont: i32) -> i32 {
    let mut scope_level = (*(*s).cur_func).scope_level;
    let mut top = (*(*s).cur_func).top_break;
    while !top.is_null() {
        close_scopes(s, scope_level, (*top).scope_level);
        scope_level = (*top).scope_level;
        if is_cont != 0
            && (*top).label_cont != -1
            && (name == JS_ATOM_NULL as u32 || (*top).label_name == name)
        {
            emit_goto(s, OP_goto as i32, (*top).label_cont);
            return 0;
        }
        if is_cont == 0
            && (*top).label_break != -1
            && ((name == JS_ATOM_NULL as u32 && (*top).is_regular_stmt() == 0)
                || (*top).label_name == name)
        {
            emit_goto(s, OP_goto as i32, (*top).label_break);
            return 0;
        }
        let mut i = 0;
        if (*top).has_iterator() != 0 {
            emit_op(s, OP_iterator_close as u8);
            i += 3;
        }
        while i < (*top).drop_count {
            emit_op(s, OP_drop as u8);
            i += 1;
        }
        if (*top).label_finally != -1 {
            emit_op(s, OP_undefined as u8);
            emit_goto(s, OP_gosub as i32, (*top).label_finally);
            emit_op(s, OP_drop as u8);
        }
        top = (*top).prev;
    }
    js_parse_error(
        s,
        if name == JS_ATOM_NULL as u32 {
            if is_cont != 0 {
                c"continue must be inside loop".as_ptr()
            } else {
                c"break must be inside loop or switch".as_ptr()
            }
        } else {
            c"break/continue label not found".as_ptr()
        },
    )
}
unsafe fn emit_return(s: *mut JSParseState, mut hasval: i32) {
    let fd = (*s).cur_func;
    if (*fd).func_kind != JS_FUNC_NORMAL as u8 {
        if hasval == 0 {
            emit_op(s, OP_undefined as u8);
            hasval = 1;
        } else if (*fd).func_kind == JS_FUNC_ASYNC_GENERATOR as u8 {
            emit_op(s, OP_await as u8);
        }
    }
    let mut top = (*fd).top_break;
    while !top.is_null() {
        if (*top).has_iterator() != 0 || (*top).label_finally != -1 {
            if hasval == 0 {
                emit_op(s, OP_undefined as u8);
                hasval = 1;
            }
            emit_op(s, OP_nip_catch as u8);
            if (*top).has_iterator() != 0 {
                if (*fd).func_kind == JS_FUNC_ASYNC_GENERATOR as u8 {
                    emit_op(s, OP_nip as u8);
                    emit_op(s, OP_swap as u8);
                    emit_op(s, OP_get_field2 as u8);
                    emit_atom(s, JS_ATOM_return);
                    emit_op(s, OP_dup as u8);
                    emit_op(s, OP_is_undefined_or_null as u8);
                    let label_next = emit_goto(s, OP_if_true as i32, -1);
                    emit_op(s, OP_call_method as u8);
                    emit_u16(s, 0);
                    emit_op(s, OP_iterator_check_object as u8);
                    emit_op(s, OP_await as u8);
                    let label_next2 = emit_goto(s, OP_goto as i32, -1);
                    emit_label(s, label_next);
                    emit_op(s, OP_drop as u8);
                    emit_label(s, label_next2);
                    emit_op(s, OP_drop as u8);
                } else {
                    emit_op(s, OP_rot3r as u8);
                    emit_op(s, OP_undefined as u8);
                    emit_op(s, OP_iterator_close as u8);
                }
            } else {
                emit_goto(s, OP_gosub as i32, (*top).label_finally);
            }
        }
        top = (*top).prev;
    }
    if (*fd).is_derived_class_constructor != 0 {
        let label_return = if hasval != 0 {
            emit_op(s, OP_check_ctor_return as u8);
            let label = emit_goto(s, OP_if_false as i32, -1);
            emit_op(s, OP_drop as u8);
            label
        } else {
            -1
        };
        emit_op(s, OP_scope_get_var_checkthis as u8);
        emit_atom(s, JS_ATOM_this);
        emit_u16(s, 0);
        emit_label(s, label_return);
        emit_op(s, OP_return as u8);
    } else if (*fd).func_kind != JS_FUNC_NORMAL as u8 {
        emit_op(s, OP_return_async as u8);
    } else {
        emit_op(
            s,
            if hasval != 0 {
                OP_return
            } else {
                OP_return_undef
            } as u8,
        );
    }
}
const DECL_MASK_FUNC: i32 = 1 << 0;
const DECL_MASK_FUNC_WITH_LABEL: i32 = 1 << 1;
const DECL_MASK_OTHER: i32 = 1 << 2;
const DECL_MASK_ALL: i32 = DECL_MASK_FUNC | DECL_MASK_FUNC_WITH_LABEL | DECL_MASK_OTHER;
// quickjs.c:26089..26131. Variable keyword validation.
unsafe fn js_unsupported_keyword(s: *mut JSParseState, atom: JSAtom) -> i32 {
    let mut buf = [0u8; ATOM_GET_STR_BUF_SIZE];
    let text = JS_AtomGetStr((*s).ctx, buf.as_mut_ptr().cast(), buf.len() as i32, atom);
    js_parse_error(
        s,
        JSErrorMessage::Pieces(&[
            b"unsupported keyword: ",
            core::ffi::CStr::from_ptr(text).to_bytes(),
        ]),
    )
}
unsafe fn js_define_var(s: *mut JSParseState, name: JSAtom, tok: i32) -> i32 {
    let fd = (*s).cur_func;
    if name == JS_ATOM_yield && (*fd).func_kind == JS_FUNC_GENERATOR as u8 {
        return js_parse_error(s, c"yield is a reserved identifier".as_ptr());
    }
    if (name == JS_ATOM_arguments || name == JS_ATOM_eval)
        && (*fd).js_mode & JS_MODE_STRICT as u8 != 0
    {
        return js_parse_error(s, c"invalid variable name in strict mode".as_ptr());
    }
    if name == crate::quickjs_atom::JS_ATOM_let && (tok == TOK_LET || tok == TOK_CONST) {
        return js_parse_error(s, c"invalid lexical variable name".as_ptr());
    }
    let kind = match tok {
        TOK_LET => JS_VAR_DEF_LET,
        TOK_CONST => JS_VAR_DEF_CONST,
        TOK_VAR => JS_VAR_DEF_VAR,
        TOK_CATCH => JS_VAR_DEF_CATCH,
        _ => unreachable!("invalid variable declaration token"),
    };
    if define_var(s, fd, name, kind) < 0 {
        -1
    } else {
        0
    }
}
