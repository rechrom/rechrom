// quickjs.c:23524..23690 and 24623..24800. Parser lookahead. MIT.
use crate::quickjs_atom::JS_ATOM_of;
unsafe fn match_identifier(mut p: *const u8, mut text: *const c_char) -> i32 {
    while *text != 0 {
        let c = *text as u8;
        text = text.add(1);
        if c != *p {
            return 0;
        }
        p = p.add(1);
    }
    let mut c = u32::from(*p);
    if c >= 128 {
        c = unicode_from_utf8(p, UTF8_CHAR_LEN_MAX as i32, &mut p) as u32;
    }
    (lre_js_is_ident_next(c) == 0) as i32
}
unsafe fn simple_next_token(pp: *mut *const u8, no_line_terminator: i32) -> i32 {
    let mut p = *pp;
    loop {
        let mut c = u32::from(*p);
        p = p.add(1);
        match c {
            13 | 10 => {
                if no_line_terminator != 0 {
                    return 10;
                }
                continue;
            }
            32 | 9 | 11 | 12 => continue,
            47 => {
                if *p == b'/' {
                    if no_line_terminator != 0 {
                        return 10;
                    }
                    while *p != 0 && *p != 13 && *p != 10 {
                        p = p.add(1);
                    }
                    continue;
                }
                if *p == b'*' {
                    loop {
                        p = p.add(1);
                        if *p == 0 {
                            break;
                        }
                        if (*p == 13 || *p == 10) && no_line_terminator != 0 {
                            return 10;
                        }
                        if *p == b'*' && *p.add(1) == b'/' {
                            p = p.add(2);
                            break;
                        }
                    }
                    continue;
                }
            }
            61 => {
                if *p == b'>' {
                    return TOK_ARROW;
                }
            }
            105 => {
                if match_identifier(p, c"n".as_ptr()) != 0 {
                    return TOK_IN;
                }
                if match_identifier(p, c"mport".as_ptr()) != 0 {
                    *pp = p.add(5);
                    return TOK_IMPORT;
                }
                return TOK_IDENT;
            }
            111 => {
                if match_identifier(p, c"f".as_ptr()) != 0 {
                    return TOK_OF;
                }
                return TOK_IDENT;
            }
            101 => {
                if match_identifier(p, c"xport".as_ptr()) != 0 {
                    return TOK_EXPORT;
                }
                return TOK_IDENT;
            }
            102 => {
                if match_identifier(p, c"unction".as_ptr()) != 0 {
                    return TOK_FUNCTION;
                }
                return TOK_IDENT;
            }
            92 => {
                if *p == b'u' && lre_js_is_ident_first(lre_parse_escape(&mut p, 1) as u32) != 0 {
                    return TOK_IDENT;
                }
            }
            _ => {
                if c >= 128 {
                    c = unicode_from_utf8(p.sub(1), UTF8_CHAR_LEN_MAX as i32, &mut p) as u32;
                    if no_line_terminator != 0 && (c == CP_PS as u32 || c == CP_LS as u32) {
                        return 10;
                    }
                }
                if lre_is_space(c) != 0 {
                    continue;
                }
                if lre_js_is_ident_first(c) != 0 {
                    return TOK_IDENT;
                }
            }
        }
        return c as i32;
    }
}
unsafe fn peek_token(s: *mut JSParseState, no_line_terminator: i32) -> i32 {
    let mut p = (*s).buf_ptr;
    simple_next_token(&mut p, no_line_terminator)
}
unsafe fn skip_shebang(pp: *mut *const u8, buf_end: *const u8) {
    let mut p = *pp;
    if *p == b'#' && *p.add(1) == b'!' {
        p = p.add(2);
        while p < buf_end {
            if *p == 10 || *p == 13 {
                break;
            } else if *p >= 128 {
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
        *pp = p;
    }
}
pub unsafe fn JS_DetectModule(input: *const c_char, input_len: usize) -> i32 {
    let mut p = input.cast::<u8>();
    skip_shebang(&mut p, p.add(input_len));
    match simple_next_token(&mut p, 0) {
        TOK_IMPORT => {
            let tok = simple_next_token(&mut p, 0);
            (tok != i32::from(b'.') && tok != i32::from(b'(')) as i32
        }
        TOK_EXPORT => 1,
        _ => 0,
    }
}
#[repr(C)]
struct JSParsePos {
    got_lf: i32,
    ptr: *const u8,
}
unsafe fn js_parse_get_pos(s: *mut JSParseState, sp: *mut JSParsePos) -> i32 {
    (*sp).ptr = (*s).token.ptr;
    (*sp).got_lf = (*s).got_lf;
    0
}
unsafe fn js_parse_seek_token(s: *mut JSParseState, sp: *const JSParsePos) -> i32 {
    (*s).buf_ptr = (*sp).ptr;
    (*s).got_lf = (*sp).got_lf;
    next_token(s)
}
unsafe fn is_regexp_allowed(tok: i32) -> i32 {
    (![
        TOK_NUMBER, TOK_STRING, TOK_REGEXP, TOK_DEC, TOK_INC, TOK_NULL, TOK_FALSE, TOK_TRUE,
        TOK_THIS, 41, 93, 125, TOK_IDENT,
    ]
    .contains(&tok)) as i32
}
const SKIP_HAS_SEMI: i32 = 1 << 0;
const SKIP_HAS_ELLIPSIS: i32 = 1 << 1;
const SKIP_HAS_ASSIGNMENT: i32 = 1 << 2;
unsafe fn has_lf_in_range(mut p1: *const u8, mut p2: *const u8) -> i32 {
    if p1 > p2 {
        core::mem::swap(&mut p1, &mut p2);
    }
    while p1 < p2 {
        if *p1 == 10 {
            return 1;
        }
        p1 = p1.add(1);
    }
    0
}
unsafe fn js_parse_skip_parens_token(
    s: *mut JSParseState,
    pbits: *mut i32,
    no_line_terminator: i32,
) -> i32 {
    let mut state = [0u8; 256];
    let mut level = 1usize;
    let mut pos: JSParsePos = core::mem::zeroed();
    js_parse_get_pos(s, &mut pos);
    let mut last_tok = 0;
    let mut tok = TOK_EOF;
    let mut bits = 0;
    loop {
        match (*s).token.val {
            40 | 91 | 123 => {
                if level >= state.len() {
                    break;
                }
                state[level] = (*s).token.val as u8;
                level += 1;
            }
            41 => {
                level -= 1;
                if state[level] != b'(' {
                    break;
                }
            }
            93 => {
                level -= 1;
                if state[level] != b'[' {
                    break;
                }
            }
            125 => {
                level -= 1;
                let c = state[level];
                if c == b'`' {
                    free_token(s, ptr::addr_of_mut!((*s).token));
                    (*s).got_lf = 0;
                    if js_parse_template_part(s, (*s).buf_ptr) != 0 {
                        break;
                    }
                    if (*s).token.u.str.sep != i32::from(b'`') {
                        if level >= state.len() {
                            break;
                        }
                        state[level] = b'`';
                        level += 1;
                    }
                } else if c != b'{' {
                    break;
                }
            }
            TOK_TEMPLATE => {
                if (*s).token.u.str.sep != i32::from(b'`') {
                    if level >= state.len() {
                        break;
                    }
                    state[level] = b'`';
                    level += 1;
                }
            }
            TOK_EOF => break,
            59 => {
                if level == 2 {
                    bits |= SKIP_HAS_SEMI;
                }
            }
            TOK_ELLIPSIS => {
                if level == 2 {
                    bits |= SKIP_HAS_ELLIPSIS;
                }
            }
            61 => bits |= SKIP_HAS_ASSIGNMENT,
            TOK_DIV_ASSIGN | 47 => {
                let len = if (*s).token.val == TOK_DIV_ASSIGN {
                    2
                } else {
                    1
                };
                if is_regexp_allowed(last_tok) != 0 {
                    (*s).buf_ptr = (*s).buf_ptr.sub(len);
                    if js_parse_regexp(s) != 0 {
                        break;
                    }
                }
            }
            _ => {}
        }
        last_tok = if (*s).token.val == TOK_IDENT
            && (token_is_pseudo_keyword(s, JS_ATOM_of) != 0
                || token_is_pseudo_keyword(s, JS_ATOM_yield) != 0)
        {
            TOK_OF
        } else {
            (*s).token.val
        };
        let last_token_ptr = (*s).token.ptr;
        if next_token(s) != 0 {
            break;
        }
        if level <= 1 {
            tok = (*s).token.val;
            if token_is_pseudo_keyword(s, JS_ATOM_of) != 0 {
                tok = TOK_OF;
            }
            if no_line_terminator != 0 && has_lf_in_range(last_token_ptr, (*s).token.ptr) != 0 {
                tok = 10;
            }
            break;
        }
    }
    if !pbits.is_null() {
        *pbits = bits;
    }
    if js_parse_seek_token(s, &pos) != 0 {
        return -1;
    }
    tok
}
// quickjs.c:37013..37029. Shared script/JSON parse state initialization.
unsafe fn js_parse_init(
    ctx: *mut JSContext,
    s: *mut JSParseState,
    input: *const c_char,
    input_len: usize,
    filename: *const c_char,
) {
    ptr::write_bytes(s, 0, 1);
    (*s).ctx = ctx;
    (*s).filename = filename;
    (*s).buf_start = input.cast();
    (*s).buf_ptr = (*s).buf_start;
    (*s).buf_end = (*s).buf_ptr.add(input_len);
    (*s).token.val = i32::from(b' ');
    (*s).token.ptr = (*s).buf_ptr;
    (*s).get_line_col_cache.ptr = (*s).buf_start;
    (*s).get_line_col_cache.buf_start = (*s).buf_start;
    (*s).get_line_col_cache.line_num = 0;
    (*s).get_line_col_cache.col_num = 0;
}
