// quickjs.c:24968..24974, 27616..27881 and 28166..28199. Expression precedence.
// MIT. Full lower unary/postfix and assignment dependencies are translated
// separately; this file must be included in their same compiler scope.
const PF_IN_ACCEPTED: i32 = 1 << 0;
const PF_POSTFIX_CALL: i32 = 1 << 1;
const PF_POW_ALLOWED: i32 = 1 << 2;
const PF_POW_FORBIDDEN: i32 = 1 << 3;
unsafe fn js_parse_expr_binary(s: *mut JSParseState, level: i32, parse_flags: i32) -> i32 {
    if level == 0 {
        return js_parse_unary(s, PF_POW_ALLOWED);
    }
    if (*s).token.val == TOK_PRIVATE_NAME
        && parse_flags & PF_IN_ACCEPTED != 0
        && level == 4
        && peek_token(s, 0) == TOK_IN
    {
        let atom = JS_DupAtom((*s).ctx, (*s).token.u.ident.atom);
        let ret = (|| {
            if next_token(s) != 0
                || (*s).token.val != TOK_IN
                || next_token(s) != 0
                || js_parse_expr_binary(s, level - 1, parse_flags) != 0
            {
                return -1;
            }
            emit_op(s, OP_scope_in_private_field as u8);
            emit_atom(s, atom);
            emit_u16(s, (*(*s).cur_func).scope_level as u16);
            0
        })();
        JS_FreeAtom((*s).ctx, atom);
        return ret;
    }
    if js_parse_expr_binary(s, level - 1, parse_flags) != 0 {
        return -1;
    }
    loop {
        let op = (*s).token.val;
        let op_token_ptr = (*s).token.ptr;
        let opcode = match level {
            1 => match op {
                42 => OP_mul,
                47 => OP_div,
                37 => OP_mod,
                _ => return 0,
            },
            2 => match op {
                43 => OP_add,
                45 => OP_sub,
                _ => return 0,
            },
            3 => match op {
                TOK_SHL => OP_shl,
                TOK_SAR => OP_sar,
                TOK_SHR => OP_shr,
                _ => return 0,
            },
            4 => match op {
                60 => OP_lt,
                62 => OP_gt,
                TOK_LTE => OP_lte,
                TOK_GTE => OP_gte,
                TOK_INSTANCEOF => OP_instanceof,
                TOK_IN if parse_flags & PF_IN_ACCEPTED != 0 => OP_in,
                _ => return 0,
            },
            5 => match op {
                TOK_EQ => OP_eq,
                TOK_NEQ => OP_neq,
                TOK_STRICT_EQ => OP_strict_eq,
                TOK_STRICT_NEQ => OP_strict_neq,
                _ => return 0,
            },
            6 => {
                if op == 38 {
                    OP_and
                } else {
                    return 0;
                }
            }
            7 => {
                if op == 94 {
                    OP_xor
                } else {
                    return 0;
                }
            }
            8 => {
                if op == 124 {
                    OP_or
                } else {
                    return 0;
                }
            }
            _ => unreachable!("invalid precedence level"),
        };
        if next_token(s) != 0 || js_parse_expr_binary(s, level - 1, parse_flags) != 0 {
            return -1;
        }
        emit_source_pos(s, op_token_ptr);
        emit_op(s, opcode as u8);
    }
}
unsafe fn js_parse_logical_and_or(s: *mut JSParseState, op: i32, parse_flags: i32) -> i32 {
    if op == TOK_LAND {
        if js_parse_expr_binary(s, 8, parse_flags) != 0 {
            return -1;
        }
    } else if js_parse_logical_and_or(s, TOK_LAND, parse_flags) != 0 {
        return -1;
    }
    if (*s).token.val == op {
        let label1 = new_label(s);
        loop {
            if next_token(s) != 0 {
                return -1;
            }
            emit_op(s, OP_dup as u8);
            emit_goto(
                s,
                if op == TOK_LAND {
                    OP_if_false
                } else {
                    OP_if_true
                } as i32,
                label1,
            );
            emit_op(s, OP_drop as u8);
            if op == TOK_LAND {
                if js_parse_expr_binary(s, 8, parse_flags) != 0 {
                    return -1;
                }
            } else if js_parse_logical_and_or(s, TOK_LAND, parse_flags) != 0 {
                return -1;
            }
            if (*s).token.val != op {
                if (*s).token.val == TOK_DOUBLE_QUESTION_MARK {
                    return js_parse_error(s, c"cannot mix ?? with && or ||".as_ptr());
                }
                break;
            }
        }
        emit_label(s, label1);
    }
    0
}
unsafe fn js_parse_coalesce_expr(s: *mut JSParseState, parse_flags: i32) -> i32 {
    if js_parse_logical_and_or(s, TOK_LOR, parse_flags) != 0 {
        return -1;
    }
    if (*s).token.val == TOK_DOUBLE_QUESTION_MARK {
        let label1 = new_label(s);
        loop {
            if next_token(s) != 0 {
                return -1;
            }
            emit_op(s, OP_dup as u8);
            emit_op(s, OP_is_undefined_or_null as u8);
            emit_goto(s, OP_if_false as i32, label1);
            emit_op(s, OP_drop as u8);
            if js_parse_expr_binary(s, 8, parse_flags) != 0 {
                return -1;
            }
            if (*s).token.val != TOK_DOUBLE_QUESTION_MARK {
                break;
            }
        }
        emit_label(s, label1);
    }
    0
}
unsafe fn js_parse_cond_expr(s: *mut JSParseState, parse_flags: i32) -> i32 {
    if js_parse_coalesce_expr(s, parse_flags) != 0 {
        return -1;
    }
    if (*s).token.val == 63 {
        if next_token(s) != 0 {
            return -1;
        }
        let label1 = emit_goto(s, OP_if_false as i32, -1);
        if js_parse_assign_expr(s) != 0 || js_parse_expect(s, 58) != 0 {
            return -1;
        }
        let label2 = emit_goto(s, OP_goto as i32, -1);
        emit_label(s, label1);
        if js_parse_assign_expr2(s, parse_flags & PF_IN_ACCEPTED) != 0 {
            return -1;
        }
        emit_label(s, label2);
    }
    0
}
unsafe fn js_parse_assign_expr(s: *mut JSParseState) -> i32 {
    js_parse_assign_expr2(s, PF_IN_ACCEPTED)
}
unsafe fn js_parse_expr2(s: *mut JSParseState, parse_flags: i32) -> i32 {
    let mut comma = false;
    loop {
        if js_parse_assign_expr2(s, parse_flags) != 0 {
            return -1;
        }
        if comma {
            (*(*s).cur_func).last_opcode_pos = -1;
        }
        if (*s).token.val != 44 {
            break;
        }
        comma = true;
        if next_token(s) != 0 {
            return -1;
        }
        emit_op(s, OP_drop as u8);
    }
    0
}
unsafe fn js_parse_expr(s: *mut JSParseState) -> i32 {
    js_parse_expr2(s, PF_IN_ACCEPTED)
}
// quickjs.c:24801..24848. Deferred anonymous function/class naming.
unsafe fn set_object_name(s: *mut JSParseState, name: JSAtom) {
    let fd = (*s).cur_func;
    let op = get_prev_opcode(fd) as u16;
    if op == OP_set_name {
        (*fd).byte_code.size = (*fd).last_opcode_pos as usize;
        (*fd).last_opcode_pos = -1;
        emit_op(s, OP_set_name as u8);
        emit_atom(s, name);
    } else if op == OP_set_class_name {
        let pos = (*fd).last_opcode_pos + 1
            - get_u32(
                (*fd)
                    .byte_code
                    .buf
                    .offset((*fd).last_opcode_pos as isize + 1),
            ) as i32;
        assert_eq!(
            *(*fd).byte_code.buf.offset(pos as isize),
            OP_define_class as u8
        );
        let atom = get_u32((*fd).byte_code.buf.offset(pos as isize + 1));
        JS_FreeAtom((*s).ctx, atom);
        put_u32(
            (*fd).byte_code.buf.offset(pos as isize + 1),
            JS_DupAtom((*s).ctx, name),
        );
        (*fd).last_opcode_pos = -1;
    }
}
unsafe fn set_object_name_computed(s: *mut JSParseState) {
    let fd = (*s).cur_func;
    let op = get_prev_opcode(fd) as u16;
    if op == OP_set_name {
        (*fd).byte_code.size = (*fd).last_opcode_pos as usize;
        (*fd).last_opcode_pos = -1;
        emit_op(s, OP_set_name_computed as u8);
    } else if op == OP_set_class_name {
        let pos = (*fd).last_opcode_pos + 1
            - get_u32(
                (*fd)
                    .byte_code
                    .buf
                    .offset((*fd).last_opcode_pos as isize + 1),
            ) as i32;
        assert_eq!(
            *(*fd).byte_code.buf.offset(pos as isize),
            OP_define_class as u8
        );
        *(*fd).byte_code.buf.offset(pos as isize) = OP_define_class_computed as u8;
        (*fd).last_opcode_pos = -1;
    }
}
