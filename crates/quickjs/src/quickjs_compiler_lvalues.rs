use crate::cutils_header::get_u32;
// quickjs.c:25785..26076. Lvalue extraction and writeback bytecode. MIT.
use crate::cutils_header::get_u16;
use crate::quickjs_atom::{JS_ATOM__with_, JS_ATOM_eval, JS_ATOM_new_target, JS_ATOM_this};
unsafe fn has_with_scope(mut fd: *mut JSFunctionDef, mut scope_level: i32) -> i32 {
    while !fd.is_null() {
        if (*fd).js_mode & JS_MODE_STRICT as u8 == 0 {
            let mut scope_idx = (*(*fd).scopes.offset(scope_level as isize)).first;
            while scope_idx >= 0 {
                let vd = &*(*fd).vars.offset(scope_idx as isize);
                if vd.var_name == JS_ATOM__with_ {
                    return 1;
                }
                scope_idx = vd.scope_next;
            }
        }
        scope_level = (*fd).parent_scope_level;
        fd = (*fd).parent;
    }
    0
}
unsafe fn get_lvalue(
    s: *mut JSParseState,
    popcode: *mut i32,
    pscope: *mut i32,
    pname: *mut JSAtom,
    plabel: *mut i32,
    pdepth: *mut i32,
    keep: i32,
    tok: i32,
) -> i32 {
    let fd = (*s).cur_func;
    let mut scope = 0;
    let mut name = JS_ATOM_NULL as u32;
    let mut label = -1;
    let mut depth = 0;
    let mut opcode = get_prev_opcode(fd) as u16;
    let mut invalid = false;
    match opcode {
        OP_scope_get_var => {
            name = get_u32(
                (*fd)
                    .byte_code
                    .buf
                    .offset((*fd).last_opcode_pos as isize + 1),
            );
            scope = get_u16(
                (*fd)
                    .byte_code
                    .buf
                    .offset((*fd).last_opcode_pos as isize + 5),
            ) as i32;
            if (name == JS_ATOM_arguments || name == JS_ATOM_eval)
                && (*fd).js_mode & JS_MODE_STRICT as u8 != 0
            {
                return js_parse_error(s, c"invalid lvalue in strict mode".as_ptr());
            }
            if name == JS_ATOM_this || name == JS_ATOM_new_target {
                invalid = true;
            } else {
                depth = if has_with_scope(fd, scope) != 0 { 2 } else { 0 };
            }
        }
        OP_get_field => {
            name = get_u32(
                (*fd)
                    .byte_code
                    .buf
                    .offset((*fd).last_opcode_pos as isize + 1),
            );
            depth = 1;
        }
        OP_scope_get_private_field => {
            name = get_u32(
                (*fd)
                    .byte_code
                    .buf
                    .offset((*fd).last_opcode_pos as isize + 1),
            );
            scope = get_u16(
                (*fd)
                    .byte_code
                    .buf
                    .offset((*fd).last_opcode_pos as isize + 5),
            ) as i32;
            depth = 1;
        }
        OP_get_array_el => depth = 2,
        OP_get_super_value => depth = 3,
        _ => invalid = true,
    }
    if invalid {
        return js_parse_error(
            s,
            if tok == TOK_FOR {
                c"invalid for in/of left hand-side".as_ptr()
            } else if tok == TOK_INC || tok == TOK_DEC {
                c"invalid increment/decrement operand".as_ptr()
            } else if tok == 91 || tok == 123 {
                c"invalid destructuring target".as_ptr()
            } else {
                c"invalid assignment left-hand side".as_ptr()
            },
        );
    }
    (*fd).byte_code.size = (*fd).last_opcode_pos as usize;
    (*fd).last_opcode_pos = -1;
    if keep != 0 {
        match opcode {
            OP_scope_get_var => {
                if depth != 0 {
                    label = new_label(s);
                    if label < 0 {
                        return -1;
                    }
                    emit_op(s, OP_scope_make_ref as u8);
                    emit_atom(s, name);
                    emit_u32(s, label as u32);
                    emit_u16(s, scope as u16);
                    update_label(fd, label, 1);
                    emit_op(s, OP_get_ref_value as u8);
                    opcode = OP_get_ref_value;
                } else {
                    emit_op(s, OP_scope_get_var as u8);
                    emit_atom(s, name);
                    emit_u16(s, scope as u16);
                }
            }
            OP_get_field => {
                emit_op(s, OP_get_field2 as u8);
                emit_atom(s, name);
            }
            OP_scope_get_private_field => {
                emit_op(s, OP_scope_get_private_field2 as u8);
                emit_atom(s, name);
                emit_u16(s, scope as u16);
            }
            OP_get_array_el => emit_op(s, OP_get_array_el3 as u8),
            OP_get_super_value => {
                emit_op(s, OP_to_propkey as u8);
                emit_op(s, OP_dup3 as u8);
                emit_op(s, OP_get_super_value as u8);
            }
            _ => unreachable!("validated lvalue opcode"),
        }
    } else if opcode == OP_scope_get_var && depth != 0 {
        label = new_label(s);
        if label < 0 {
            return -1;
        }
        emit_op(s, OP_scope_make_ref as u8);
        emit_atom(s, name);
        emit_u32(s, label as u32);
        emit_u16(s, scope as u16);
        update_label(fd, label, 1);
        opcode = OP_get_ref_value;
    }
    *popcode = opcode as i32;
    *pscope = scope;
    *pname = name;
    *plabel = label;
    if !pdepth.is_null() {
        *pdepth = depth;
    }
    0
}
type PutLValueEnum = u32;
const PUT_LVALUE_NOKEEP: PutLValueEnum = 0;
const PUT_LVALUE_NOKEEP_DEPTH: PutLValueEnum = 1;
const PUT_LVALUE_KEEP_TOP: PutLValueEnum = 2;
const PUT_LVALUE_KEEP_SECOND: PutLValueEnum = 3;
const PUT_LVALUE_NOKEEP_BOTTOM: PutLValueEnum = 4;
unsafe fn put_lvalue(
    s: *mut JSParseState,
    opcode: i32,
    scope: i32,
    name: JSAtom,
    label: i32,
    special: PutLValueEnum,
    is_let: i32,
) {
    let op = opcode as u16;
    match op {
        OP_scope_get_var => match special {
            PUT_LVALUE_NOKEEP
            | PUT_LVALUE_NOKEEP_DEPTH
            | PUT_LVALUE_KEEP_SECOND
            | PUT_LVALUE_NOKEEP_BOTTOM => {}
            PUT_LVALUE_KEEP_TOP => emit_op(s, OP_dup as u8),
            _ => unreachable!("invalid PutLValueEnum"),
        },
        OP_get_field | OP_scope_get_private_field => match special {
            PUT_LVALUE_NOKEEP | PUT_LVALUE_NOKEEP_DEPTH => {}
            PUT_LVALUE_KEEP_TOP => emit_op(s, OP_insert2 as u8),
            PUT_LVALUE_KEEP_SECOND => emit_op(s, OP_perm3 as u8),
            PUT_LVALUE_NOKEEP_BOTTOM => emit_op(s, OP_swap as u8),
            _ => unreachable!("invalid PutLValueEnum"),
        },
        OP_get_array_el | OP_get_ref_value => {
            if op == OP_get_ref_value {
                JS_FreeAtom((*s).ctx, name);
                emit_label(s, label);
            }
            match special {
                PUT_LVALUE_NOKEEP => emit_op(s, OP_nop as u8),
                PUT_LVALUE_NOKEEP_DEPTH => {}
                PUT_LVALUE_KEEP_TOP => emit_op(s, OP_insert3 as u8),
                PUT_LVALUE_KEEP_SECOND => emit_op(s, OP_perm4 as u8),
                PUT_LVALUE_NOKEEP_BOTTOM => emit_op(s, OP_rot3l as u8),
                _ => unreachable!("invalid PutLValueEnum"),
            }
        }
        OP_get_super_value => match special {
            PUT_LVALUE_NOKEEP | PUT_LVALUE_NOKEEP_DEPTH => {}
            PUT_LVALUE_KEEP_TOP => emit_op(s, OP_insert4 as u8),
            PUT_LVALUE_KEEP_SECOND => emit_op(s, OP_perm5 as u8),
            PUT_LVALUE_NOKEEP_BOTTOM => emit_op(s, OP_rot4l as u8),
            _ => unreachable!("invalid PutLValueEnum"),
        },
        _ => {}
    }
    match op {
        OP_scope_get_var => {
            emit_op(
                s,
                if is_let != 0 {
                    OP_scope_put_var_init
                } else {
                    OP_scope_put_var
                } as u8,
            );
            emit_u32(s, name);
            emit_u16(s, scope as u16);
        }
        OP_get_field => {
            emit_op(s, OP_put_field as u8);
            emit_u32(s, name);
        }
        OP_scope_get_private_field => {
            emit_op(s, OP_scope_put_private_field as u8);
            emit_u32(s, name);
            emit_u16(s, scope as u16);
        }
        OP_get_array_el => emit_op(s, OP_put_array_el as u8),
        OP_get_ref_value => emit_op(s, OP_put_ref_value as u8),
        OP_get_super_value => emit_op(s, OP_put_super_value as u8),
        _ => unreachable!("validated lvalue opcode"),
    }
}
