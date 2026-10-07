// quickjs.c:23151..23523. JSON / extended JSON lexical analysis. MIT.
use crate::cutils::strstart;
use crate::cutils_header::from_hex;
use crate::libunicode_header::lre_is_id_continue_byte;
unsafe fn json_parse_ident(s: *mut JSParseState, pp: *mut *const u8, mut c: i32) -> JSAtom {
    let mut p = *pp;
    let mut ident_buf = [0i8; 128];
    let static_buf = ident_buf.as_mut_ptr();
    let mut buf = static_buf;
    let mut ident_size = 128usize;
    let mut ident_pos = 0usize;
    let atom = loop {
        *buf.add(ident_pos) = c as i8;
        ident_pos += 1;
        c = i32::from(*p);
        if c >= 128 || lre_is_id_continue_byte(c as u8) == 0 {
            break JS_NewAtomLen((*s).ctx, buf, ident_pos);
        }
        p = p.add(1);
        if ident_pos >= ident_size - UTF8_CHAR_LEN_MAX
            && ident_realloc((*s).ctx, &mut buf, &mut ident_size, static_buf) != 0
        {
            break JS_ATOM_NULL as u32;
        }
    };
    if buf != static_buf {
        js_free((*s).ctx, buf.cast());
    }
    *pp = p;
    atom
}
unsafe fn json_parse_string(s: *mut JSParseState, pp: *mut *const u8, sep: i32) -> i32 {
    let mut b: StringBuffer = core::mem::zeroed();
    let mut finalized = false;
    let result = (|| {
        if string_buffer_init((*s).ctx, &mut b, 32) != 0 {
            return -1;
        }
        let mut p = *pp;
        loop {
            if p >= (*s).buf_end {
                js_parse_error(s, c"Unexpected end of JSON input".as_ptr());
                return -1;
            }
            let mut c = u32::from(*p);
            p = p.add(1);
            if c == sep as u32 {
                break;
            }
            if c < 32 {
                js_parse_error_pos(
                    s,
                    p.sub(1),
                    c"Bad control character in string literal".as_ptr(),
                );
                return -1;
            }
            if c == 92 {
                c = u32::from(*p);
                p = p.add(1);
                match c {
                    98 => c = 8,
                    102 => c = 12,
                    110 => c = 10,
                    114 => c = 13,
                    116 => c = 9,
                    92 | 47 => {}
                    117 => {
                        c = 0;
                        for _ in 0..4 {
                            let h = from_hex(i32::from(*p));
                            p = p.add(1);
                            if h < 0 {
                                js_parse_error_pos(s, p.sub(1), c"Bad Unicode escape".as_ptr());
                                return -1;
                            }
                            c = (c << 4) | h as u32;
                        }
                    }
                    10 => {
                        if (*s).ext_json != 0 {
                            continue;
                        }
                        js_parse_error_pos(s, p.sub(1), c"Bad escaped character".as_ptr());
                        return -1;
                    }
                    118 => {
                        if (*s).ext_json != 0 {
                            c = 11;
                        } else {
                            js_parse_error_pos(s, p.sub(1), c"Bad escaped character".as_ptr());
                            return -1;
                        }
                    }
                    _ => {
                        if c != sep as u32 {
                            if p > (*s).buf_end {
                                js_parse_error(s, c"Unexpected end of JSON input".as_ptr());
                                return -1;
                            }
                            js_parse_error_pos(s, p.sub(1), c"Bad escaped character".as_ptr());
                            return -1;
                        }
                    }
                }
            } else if c >= 128 {
                let mut next = ptr::null();
                c = unicode_from_utf8(p.sub(1), UTF8_CHAR_LEN_MAX as i32, &mut next) as u32;
                if c > 0x10ffff {
                    js_parse_error_pos(s, p.sub(1), c"Bad UTF-8 sequence".as_ptr());
                    return -1;
                }
                p = next;
            }
            if string_buffer_putc(&mut b, c) != 0 {
                return -1;
            }
        }
        (*s).token.val = TOK_STRING;
        (*s).token.u.str.sep = sep;
        finalized = true;
        (*s).token.u.str.str = string_buffer_end(&mut b);
        *pp = p;
        0
    })();
    if result < 0 && !finalized {
        string_buffer_free(&mut b);
    }
    result
}
unsafe fn json_parse_number(s: *mut JSParseState, pp: *mut *const u8) -> i32 {
    let mut p = *pp;
    let p_start = p;
    let mut atod_mem = crate::dtoa_header::JSATODTempMem::default();
    let mut early_value = None;
    if *p == b'+' || *p == b'-' {
        p = p.add(1);
    }
    if !(*p).is_ascii_digit() {
        if (*s).ext_json != 0 {
            let mut pchar = p.cast::<c_char>();
            if strstart(pchar, c"Infinity".as_ptr(), &mut pchar) != 0 {
                p = pchar.cast();
                early_value = Some(if *p_start == b'-' {
                    f64::NEG_INFINITY
                } else {
                    f64::INFINITY
                });
            } else if strstart(pchar, c"NaN".as_ptr(), &mut pchar) != 0 {
                p = pchar.cast();
                early_value = Some(f64::NAN);
            } else if *p != b'.' {
                return js_parse_error_pos(
                    s,
                    p,
                    JSErrorMessage::Pieces(&[b"Unexpected token '", &[*p], b"'"]),
                );
            }
        } else {
            return js_parse_error_pos(
                s,
                p,
                JSErrorMessage::Pieces(&[b"Unexpected token '", &[*p], b"'"]),
            );
        }
    }
    if early_value.is_none() {
        if *p == b'0' {
            if (*s).ext_json != 0 {
                let radix = match *p.add(1) {
                    b'x' | b'X' => 16,
                    b'o' | b'O' => 8,
                    b'b' | b'B' => 2,
                    _ => 10,
                };
                if radix != 10 {
                    p = p.add(2);
                    if to_digit(i32::from(*p)) >= radix {
                        return js_parse_error_pos(
                            s,
                            p,
                            JSErrorMessage::Pieces(&[b"Unexpected token '", &[*p], b"'"]),
                        );
                    }
                    let mut end = p.cast::<c_char>();
                    let d = crate::dtoa::js_atod(
                        p_start.cast(),
                        &mut end,
                        0,
                        crate::dtoa_header::JS_ATOD_INT_ONLY
                            | crate::dtoa_header::JS_ATOD_ACCEPT_BIN_OCT,
                        &mut atod_mem,
                    );
                    p = end.cast();
                    early_value = Some(d);
                }
            }
            if early_value.is_none() && (*p.add(1)).is_ascii_digit() {
                return js_parse_error_pos(s, p, c"Unexpected number".as_ptr());
            }
        }
    }
    let d = if let Some(d) = early_value {
        d
    } else {
        while (*p).is_ascii_digit() {
            p = p.add(1);
        }
        if *p == b'.' {
            p = p.add(1);
            if !(*p).is_ascii_digit() {
                return js_parse_error_pos(s, p, c"Unterminated fractional number".as_ptr());
            }
            while (*p).is_ascii_digit() {
                p = p.add(1);
            }
        }
        if *p == b'e' || *p == b'E' {
            p = p.add(1);
            if *p == b'+' || *p == b'-' {
                p = p.add(1);
            }
            if !(*p).is_ascii_digit() {
                return js_parse_error_pos(s, p, c"Exponent part is missing a number".as_ptr());
            }
            while (*p).is_ascii_digit() {
                p = p.add(1);
            }
        }
        crate::dtoa::js_atod(p_start.cast(), ptr::null_mut(), 10, 0, &mut atod_mem)
    };
    (*s).token.val = TOK_NUMBER;
    (*s).token.u.num.val = JS_NewFloat64((*s).ctx, d);
    *pp = p;
    0
}
unsafe fn json_next_token(s: *mut JSParseState) -> i32 {
    if js_check_stack_overflow((*(*s).ctx).rt, 0) != 0 {
        return js_parse_error(s, c"stack overflow".as_ptr());
    }
    free_token(s, ptr::addr_of_mut!((*s).token));
    let mut p = (*s).buf_ptr;
    (*s).last_ptr = p;
    let ret = (|| {
        loop {
            (*s).token.ptr = p;
            let c = i32::from(*p);
            if c == 0 && p >= (*s).buf_end {
                (*s).token.val = TOK_EOF;
                break;
            }
            if c == 34 || (c == 39 && (*s).ext_json != 0) {
                p = p.add(1);
                if json_parse_string(s, &mut p, c) != 0 {
                    return -1;
                }
                break;
            }
            if c == 13 || c == 10 {
                if c == 13 && *p.add(1) == 10 {
                    p = p.add(1);
                }
                p = p.add(1);
                continue;
            }
            if c == 32 || c == 9 || ((c == 12 || c == 11) && (*s).ext_json != 0) {
                p = p.add(1);
                continue;
            }
            if c == 47 && (*s).ext_json != 0 {
                if *p.add(1) == b'*' {
                    p = p.add(2);
                    loop {
                        if *p == 0 && p >= (*s).buf_end {
                            js_parse_error(s, c"unexpected end of comment".as_ptr());
                            return -1;
                        }
                        if *p == b'*' && *p.add(1) == b'/' {
                            p = p.add(2);
                            break;
                        }
                        if *p >= 128 {
                            let c = unicode_from_utf8(p, UTF8_CHAR_LEN_MAX as i32, &mut p);
                            if c == -1 {
                                p = p.add(1);
                            }
                        } else {
                            p = p.add(1);
                        }
                    }
                    continue;
                }
                if *p.add(1) == b'/' {
                    p = p.add(2);
                    loop {
                        if (*p == 0 && p >= (*s).buf_end) || *p == 13 || *p == 10 {
                            break;
                        }
                        if *p >= 128 {
                            let c = unicode_from_utf8(p, UTF8_CHAR_LEN_MAX as i32, &mut p);
                            if c == CP_LS || c == CP_PS {
                                break;
                            } else if c == -1 {
                                p = p.add(1);
                            }
                        } else {
                            p = p.add(1);
                        }
                    }
                    continue;
                }
            }
            if (*p).is_ascii_alphabetic() || *p == b'_' || *p == b'$' {
                p = p.add(1);
                let atom = json_parse_ident(s, &mut p, c);
                if atom == JS_ATOM_NULL as u32 {
                    return -1;
                }
                (*s).token.u.ident = JSTokenIdent {
                    atom,
                    has_escape: 0,
                    is_reserved: 0,
                };
                (*s).token.val = TOK_IDENT;
                break;
            }
            if c == 45
                || (*p).is_ascii_digit()
                || (c == 43 && (*s).ext_json != 0)
                || (c == 46 && (*s).ext_json != 0 && (*p.add(1)).is_ascii_digit())
            {
                if json_parse_number(s, &mut p) != 0 {
                    return -1;
                }
                break;
            }
            if c >= 128 {
                js_parse_error(s, c"unexpected character".as_ptr());
                return -1;
            }
            (*s).token.val = c;
            p = p.add(1);
            break;
        }
        (*s).buf_ptr = p;
        0
    })();
    if ret < 0 {
        (*s).token.val = TOK_ERROR;
    }
    ret
}
