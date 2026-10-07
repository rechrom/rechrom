// quickjs.c:22074..23200. JavaScript lexical analysis. MIT.
use crate::cutils::{unicode_from_utf8, unicode_to_utf8};
use crate::cutils_header::UTF8_CHAR_LEN_MAX;
use crate::libregexp::lre_parse_escape;
use crate::libunicode_header::{lre_is_space, lre_js_is_ident_first, lre_js_is_ident_next};
use crate::quickjs_atom::{
    JS_ATOM_await, JS_ATOM_yield, JS_ATOM_LAST_KEYWORD, JS_ATOM_LAST_STRICT_KEYWORD,
};

#[cfg(test)]
mod compiler_source_position_tests {
    use super::*;

    unsafe fn check_positions(input: &[u8], offsets: &[usize]) {
        let mut cache: GetLineColCache = core::mem::zeroed();
        cache.buf_start = input.as_ptr();
        cache.ptr = input.as_ptr();
        for &offset in offsets {
            let mut expected_col = 0;
            let expected_line = get_line_col(&mut expected_col, input.as_ptr(), offset);
            let mut actual_col = 0;
            let actual_line = get_line_col_cached(&mut cache, &mut actual_col, input.as_ptr().add(offset));
            assert_eq!((actual_line, actual_col), (expected_line, expected_col), "offset={offset}");
            assert_eq!((cache.ptr, cache.line_num, cache.col_num), (input.as_ptr().add(offset), expected_line, expected_col));
        }
    }

    #[test]
    fn compiler_source_checkpoints_match_official_byte_counting() { unsafe {
        // Include every byte value, partial/invalid UTF-8, LF, CR and Unicode
        // separators. The compiler rule counts only LF and leading bytes.
        let input: Vec<u8> = (0..16384).map(|index| (index % 256) as u8).collect();
        let _scope = CompilerSourcePositionScope::new(input.as_ptr().cast(), input.len());
        let mut offsets = vec![input.len(), 0, input.len(), 31, 8193, 1, 4095, 4096, 4097];
        offsets.extend((0..input.len()).step_by(127));
        offsets.extend((0..input.len()).rev().step_by(131));
        check_positions(&input, &offsets);
    } }

    #[test]
    fn compiler_source_checkpoints_restore_outer_source_after_nested_eval() { unsafe {
        let outer = vec![b'a'; 32768];
        let inner = vec![b'\n'; 8192];
        let _outer_scope = CompilerSourcePositionScope::new(outer.as_ptr().cast(), outer.len());
        check_positions(&outer, &[32768, 0, 8193, 1]);
        {
            let _inner_scope = CompilerSourcePositionScope::new(inner.as_ptr().cast(), inner.len());
            // An unrelated cache must retain the original scan while the
            // inner source owns the active index.
            check_positions(&outer, &[32768, 0, 32768, 4]);
            check_positions(&inner, &[8192, 0, 4097, 1]);
        }
        check_positions(&outer, &[32768, 0, 16383, 256]);
    } }
}
unsafe fn free_token(s: *mut JSParseState, token: *mut JSToken) {
    match (*token).val {
        TOK_NUMBER => JS_FreeValue((*s).ctx, (*token).u.num.val),
        TOK_STRING | TOK_TEMPLATE => JS_FreeValue((*s).ctx, (*token).u.str.str),
        TOK_REGEXP => {
            JS_FreeValue((*s).ctx, (*token).u.regexp.body);
            JS_FreeValue((*s).ctx, (*token).u.regexp.flags);
        }
        TOK_IDENT | TOK_PRIVATE_NAME => JS_FreeAtom((*s).ctx, (*token).u.ident.atom),
        n if (TOK_FIRST_KEYWORD..=TOK_LAST_KEYWORD).contains(&n) => {
            JS_FreeAtom((*s).ctx, (*token).u.ident.atom)
        }
        _ => {}
    }
}
unsafe fn get_line_col(pcol_num: *mut i32, buf: *const u8, len: usize) -> i32 {
    let mut line_num = 0;
    let mut col_num = 0;
    for i in 0..len {
        let c = *buf.add(i);
        if c == b'\n' {
            line_num += 1;
            col_num = 0;
        } else if c < 0x80 || c >= 0xc0 {
            col_num += 1;
        }
    }
    *pcol_num = col_num;
    line_num
}
// quickjs.c:34445 proposes source-byte checkpoints for large backwards jumps
// during pc2line generation. The original cache remains the cheap path for
// nearby positions. Checkpoints have exactly get_line_col's byte semantics,
// including LF-only line counting and UTF-8 leading-byte column counting.
const COMPILER_SOURCE_POSITION_STRIDE: usize = 256;
const COMPILER_SOURCE_POSITION_MIN_JUMP: usize = 4096;
struct CompilerSourcePositions {
    start: usize,
    len: usize,
    checkpoints: Vec<(i32, i32)>,
}
std::thread_local! {
    static COMPILER_SOURCE_POSITIONS: std::cell::RefCell<Vec<CompilerSourcePositions>> = const { std::cell::RefCell::new(Vec::new()) };
}
struct CompilerSourcePositionScope;
impl CompilerSourcePositionScope {
    fn new(input: *const c_char, len: usize) -> Self {
        COMPILER_SOURCE_POSITIONS.with(|positions| positions.borrow_mut().push(CompilerSourcePositions {
            start: input as usize, len, checkpoints: Vec::new(),
        }));
        Self
    }
}
impl Drop for CompilerSourcePositionScope {
    fn drop(&mut self) {
        COMPILER_SOURCE_POSITIONS.with(|positions| { positions.borrow_mut().pop(); });
    }
}
unsafe fn compiler_source_position(cache: &GetLineColCache, p: *const u8) -> Option<(i32, i32)> {
    COMPILER_SOURCE_POSITIONS.with(|positions| {
        let mut positions = positions.borrow_mut();
        let source = positions.last_mut()?;
        if source.start != cache.buf_start as usize { return None; }
        let offset = (p as usize).checked_sub(source.start)?;
        let previous = (cache.ptr as usize).checked_sub(source.start)?;
        if offset > source.len || previous > source.len { return None; }
        // A forward scan alone is linear already. Build lazily only when an
        // actual large backwards jump would make source bytes get rescanned.
        if source.checkpoints.is_empty() {
            if offset >= previous { return None; }
            source.checkpoints.push((0, 0));
        }
        let target = offset / COMPILER_SOURCE_POSITION_STRIDE;
        while source.checkpoints.len() <= target {
            let index = source.checkpoints.len() - 1;
            let (line, col) = source.checkpoints[index];
            let mut delta_col = 0;
            let delta_line = get_line_col(&mut delta_col,
                cache.buf_start.add(index * COMPILER_SOURCE_POSITION_STRIDE), COMPILER_SOURCE_POSITION_STRIDE);
            source.checkpoints.push((line + delta_line, if delta_line == 0 { col + delta_col } else { delta_col }));
        }
        let (line, col) = source.checkpoints[target];
        let start = target * COMPILER_SOURCE_POSITION_STRIDE;
        let mut delta_col = 0;
        let delta_line = get_line_col(&mut delta_col, cache.buf_start.add(start), offset - start);
        Some((line + delta_line, if delta_line == 0 { col + delta_col } else { delta_col }))
    })
}
unsafe fn get_line_col_cached(s: *mut GetLineColCache, pcol_num: *mut i32, p: *const u8) -> i32 {
    if (p as usize).abs_diff((*s).ptr as usize) >= COMPILER_SOURCE_POSITION_MIN_JUMP {
        if let Some((line, col)) = compiler_source_position(&*s, p) {
            (*s).ptr = p;
            (*s).line_num = line;
            (*s).col_num = col;
            *pcol_num = col;
            return line;
        }
    }
    let mut col_num = 0;
    if p >= (*s).ptr {
        let lines = get_line_col(&mut col_num, (*s).ptr, p.offset_from((*s).ptr) as usize);
        if lines == 0 {
            (*s).col_num += col_num
        } else {
            (*s).line_num += lines;
            (*s).col_num = col_num
        }
    } else {
        let lines = get_line_col(&mut col_num, p, (*s).ptr.offset_from(p) as usize);
        if lines == 0 {
            (*s).col_num -= col_num
        } else {
            (*s).line_num -= lines;
            col_num = 0;
            let mut q = p;
            while q > (*s).buf_start {
                q = q.sub(1);
                if *q == b'\n' {
                    break;
                } else if *q < 0x80 || *q >= 0xc0 {
                    col_num += 1;
                }
            }
            (*s).col_num = col_num;
        }
    }
    (*s).ptr = p;
    *pcol_num = (*s).col_num;
    (*s).line_num
}
unsafe fn js_parse_error_v(s: *mut JSParseState, p: *const u8, message: JSErrorMessage<'_>) -> i32 {
    let ctx = (*s).ctx;
    let mut col_num = 0;
    let line_num = get_line_col(
        &mut col_num,
        (*s).buf_start,
        p.offset_from((*s).buf_start) as usize,
    );
    JS_ThrowError2(ctx, JS_SYNTAX_ERROR, message, 0);
    build_backtrace(
        ctx,
        (*(*ctx).rt).current_exception,
        (*s).filename,
        line_num + 1,
        col_num + 1,
        0,
    );
    -1
}
unsafe fn js_parse_error_pos<'a>(
    s: *mut JSParseState,
    p: *const u8,
    message: impl Into<JSErrorMessage<'a>>,
) -> i32 {
    js_parse_error_v(s, p, message.into())
}
unsafe fn js_parse_error<'a>(s: *mut JSParseState, message: impl Into<JSErrorMessage<'a>>) -> i32 {
    js_parse_error_v(s, (*s).token.ptr, message.into())
}
unsafe fn js_parse_expect(s: *mut JSParseState, tok: i32) -> i32 {
    if (*s).token.val != tok {
        return js_parse_error(s, format_args!("expecting '{}'", char::from(tok as u8)));
    }
    next_token(s)
}
unsafe fn js_parse_expect_semi(s: *mut JSParseState) -> i32 {
    if (*s).token.val != i32::from(b';') {
        if (*s).token.val == TOK_EOF || (*s).token.val == i32::from(b'}') || (*s).got_lf != 0 {
            return 0;
        }
        return js_parse_error(s, c"expecting ';'".as_ptr());
    }
    next_token(s)
}
unsafe fn js_parse_error_reserved_identifier(s: *mut JSParseState) -> i32 {
    let mut buf = [0u8; ATOM_GET_STR_BUF_SIZE];
    let text = JS_AtomGetStr(
        (*s).ctx,
        buf.as_mut_ptr().cast(),
        buf.len() as i32,
        (*s).token.u.ident.atom,
    );
    js_parse_error(
        s,
        JSErrorMessage::Pieces(&[
            b"'",
            core::ffi::CStr::from_ptr(text).to_bytes(),
            b"' is a reserved identifier",
        ]),
    )
}
unsafe fn js_parse_template_part(s: *mut JSParseState, mut p: *const u8) -> i32 {
    let mut b: StringBuffer = core::mem::zeroed();
    let mut finalized = false;
    let result = (|| {
        if string_buffer_init((*s).ctx, &mut b, 32) != 0 {
            return -1;
        }
        let mut c;
        loop {
            if p >= (*s).buf_end {
                js_parse_error(s, c"unexpected end of string".as_ptr());
                return -1;
            }
            c = u32::from(*p);
            p = p.add(1);
            if c == u32::from(b'`') {
                break;
            }
            if c == u32::from(b'$') && *p == b'{' {
                p = p.add(1);
                break;
            }
            if c == u32::from(b'\\') {
                if string_buffer_putc8(&mut b, c) != 0 {
                    return -1;
                }
                if p >= (*s).buf_end {
                    js_parse_error(s, c"unexpected end of string".as_ptr());
                    return -1;
                }
                c = u32::from(*p);
                p = p.add(1);
            }
            if c == u32::from(b'\r') {
                if *p == b'\n' {
                    p = p.add(1);
                }
                c = u32::from(b'\n');
            }
            if c >= 0x80 {
                let mut next = ptr::null();
                c = unicode_from_utf8(p.sub(1), UTF8_CHAR_LEN_MAX as i32, &mut next) as u32;
                if c > 0x10ffff {
                    js_parse_error_pos(s, p.sub(1), c"invalid UTF-8 sequence".as_ptr());
                    return -1;
                }
                p = next;
            }
            if string_buffer_putc(&mut b, c) != 0 {
                return -1;
            }
        }
        finalized = true;
        let str = string_buffer_end(&mut b);
        if JS_IsException(str) != 0 {
            return -1;
        }
        (*s).token.val = TOK_TEMPLATE;
        (*s).token.u.str = JSTokenStr { str, sep: c as i32 };
        (*s).buf_ptr = p;
        0
    })();
    if result < 0 && !finalized {
        string_buffer_free(&mut b);
    }
    result
}
unsafe fn js_parse_string(
    s: *mut JSParseState,
    sep: i32,
    do_throw: i32,
    mut p: *const u8,
    token: *mut JSToken,
    pp: *mut *const u8,
) -> i32 {
    let mut b: StringBuffer = core::mem::zeroed();
    let mut finalized = false;
    let result = (|| {
        if string_buffer_init((*s).ctx, &mut b, 32) != 0 {
            return -1;
        }
        let mut c;
        loop {
            if p >= (*s).buf_end {
                if do_throw != 0 {
                    js_parse_error(s, c"unexpected end of string".as_ptr());
                }
                return -1;
            }
            c = u32::from(*p);
            if c < 0x20 {
                if sep == i32::from(b'`') {
                    if c == u32::from(b'\r') {
                        if *p.add(1) == b'\n' {
                            p = p.add(1);
                        }
                        c = u32::from(b'\n');
                    }
                } else if c == u32::from(b'\n') || c == u32::from(b'\r') {
                    if do_throw != 0 {
                        js_parse_error(s, c"unexpected end of string".as_ptr());
                    }
                    return -1;
                }
            }
            p = p.add(1);
            if c == sep as u32 {
                break;
            }
            if c == u32::from(b'$') && *p == b'{' && sep == i32::from(b'`') {
                p = p.add(1);
                break;
            }
            if c == u32::from(b'\\') {
                let p_escape = p.sub(1);
                c = u32::from(*p);
                match c {
                    0 => {
                        if p >= (*s).buf_end {
                            if do_throw != 0 {
                                js_parse_error(s, c"unexpected end of string".as_ptr());
                            }
                            return -1;
                        }
                        p = p.add(1);
                    }
                    39 | 34 | 92 => p = p.add(1),
                    13 | 10 => {
                        if c == 13 && *p.add(1) == 10 {
                            p = p.add(1);
                        }
                        p = p.add(1);
                        continue;
                    }
                    _ => {
                        let mut parse_escape = false;
                        if (48..=57).contains(&c) {
                            if (*(*s).cur_func).js_mode & JS_MODE_STRICT as u8 == 0
                                && sep != i32::from(b'`')
                            {
                                parse_escape = true;
                            } else if c == 48 && !(*p.add(1)).is_ascii_digit() {
                                p = p.add(1);
                                c = 0;
                            } else {
                                if do_throw != 0 {
                                    if c >= 56 || sep == i32::from(b'`') {
                                        js_parse_error_pos(
                                            s,
                                            p_escape,
                                            c"malformed escape sequence in string literal".as_ptr(),
                                        );
                                    } else {
                                        js_parse_error_pos(s,p_escape,c"octal escape sequences are not allowed in strict mode".as_ptr());
                                    }
                                }
                                return -1;
                            }
                        } else if c >= 128 {
                            let mut next = ptr::null();
                            c = unicode_from_utf8(p, UTF8_CHAR_LEN_MAX as i32, &mut next) as u32;
                            if c > 0x10ffff {
                                if do_throw != 0 {
                                    js_parse_error(s, c"invalid UTF-8 sequence".as_ptr());
                                }
                                return -1;
                            }
                            p = next;
                            if c == CP_LS as u32 || c == CP_PS as u32 {
                                continue;
                            }
                        } else {
                            parse_escape = true;
                        }
                        if parse_escape {
                            let ret = lre_parse_escape(&mut p, 1);
                            if ret == -1 {
                                if do_throw != 0 {
                                    js_parse_error_pos(
                                        s,
                                        p_escape,
                                        c"malformed escape sequence in string literal".as_ptr(),
                                    );
                                }
                                return -1;
                            } else if ret < 0 {
                                p = p.add(1);
                            } else {
                                c = ret as u32;
                            }
                        }
                    }
                }
            } else if c >= 128 {
                let mut next = ptr::null();
                c = unicode_from_utf8(p.sub(1), UTF8_CHAR_LEN_MAX as i32, &mut next) as u32;
                if c > 0x10ffff {
                    if do_throw != 0 {
                        js_parse_error(s, c"invalid UTF-8 sequence".as_ptr());
                    }
                    return -1;
                }
                p = next;
            }
            if string_buffer_putc(&mut b, c) != 0 {
                return -1;
            }
        }
        finalized = true;
        let str = string_buffer_end(&mut b);
        if JS_IsException(str) != 0 {
            return -1;
        }
        (*token).val = TOK_STRING;
        (*token).u.str = JSTokenStr { str, sep: c as i32 };
        *pp = p;
        0
    })();
    if result < 0 && !finalized {
        string_buffer_free(&mut b);
    }
    result
}
unsafe fn token_is_pseudo_keyword(s: *mut JSParseState, atom: JSAtom) -> i32 {
    ((*s).token.val == TOK_IDENT
        && (*s).token.u.ident.atom == atom
        && (*s).token.u.ident.has_escape == 0) as i32
}
unsafe fn js_parse_regexp(s: *mut JSParseState) -> i32 {
    let mut p = (*s).buf_ptr.add(1);
    let mut in_class = false;
    let mut b: StringBuffer = core::mem::zeroed();
    let mut b2: StringBuffer = core::mem::zeroed();
    if string_buffer_init((*s).ctx, &mut b, 32) != 0 {
        return -1;
    }
    let mut finalized = false;
    let result = (|| {
        if string_buffer_init((*s).ctx, &mut b2, 1) != 0 {
            return -1;
        }
        loop {
            if p >= (*s).buf_end {
                js_parse_error(s, c"unexpected end of regexp".as_ptr());
                return -1;
            }
            let mut c = u32::from(*p);
            p = p.add(1);
            if c == 10 || c == 13 {
                js_parse_error_pos(
                    s,
                    p.sub(1),
                    c"unexpected line terminator in regexp".as_ptr(),
                );
                return -1;
            } else if c == 47 {
                if !in_class {
                    break;
                }
            } else if c == 91 {
                in_class = true;
            } else if c == 93 {
                in_class = false;
            } else if c == 92 {
                if string_buffer_putc8(&mut b, c) != 0 {
                    return -1;
                }
                c = u32::from(*p);
                p = p.add(1);
                if c == 10 || c == 13 {
                    js_parse_error_pos(
                        s,
                        p.sub(1),
                        c"unexpected line terminator in regexp".as_ptr(),
                    );
                    return -1;
                } else if c == 0 && p >= (*s).buf_end {
                    js_parse_error(s, c"unexpected end of regexp".as_ptr());
                    return -1;
                } else if c >= 128 {
                    let mut next = ptr::null();
                    c = unicode_from_utf8(p.sub(1), UTF8_CHAR_LEN_MAX as i32, &mut next) as u32;
                    if c > 0x10ffff {
                        js_parse_error_pos(s, p.sub(1), c"invalid UTF-8 sequence".as_ptr());
                        return -1;
                    }
                    p = next;
                    if c == CP_LS as u32 || c == CP_PS as u32 {
                        js_parse_error_pos(
                            s,
                            p.sub(1),
                            c"unexpected line terminator in regexp".as_ptr(),
                        );
                        return -1;
                    }
                }
            } else if c >= 128 {
                let mut next = ptr::null();
                c = unicode_from_utf8(p.sub(1), UTF8_CHAR_LEN_MAX as i32, &mut next) as u32;
                if c > 0x10ffff {
                    js_parse_error_pos(s, p.sub(1), c"invalid UTF-8 sequence".as_ptr());
                    return -1;
                }
                if c == CP_LS as u32 || c == CP_PS as u32 {
                    js_parse_error_pos(
                        s,
                        p.sub(1),
                        c"unexpected line terminator in regexp".as_ptr(),
                    );
                    return -1;
                }
                p = next;
            }
            if string_buffer_putc(&mut b, c) != 0 {
                return -1;
            }
        }
        loop {
            let mut next = p.add(1);
            let mut c = u32::from(*p);
            if c >= 128 {
                c = unicode_from_utf8(p, UTF8_CHAR_LEN_MAX as i32, &mut next) as u32;
                if c > 0x10ffff {
                    p = p.add(1);
                    js_parse_error_pos(s, p.sub(1), c"invalid UTF-8 sequence".as_ptr());
                    return -1;
                }
            }
            if lre_js_is_ident_next(c) == 0 {
                break;
            }
            if string_buffer_putc(&mut b2, c) != 0 {
                return -1;
            }
            p = next;
        }
        finalized = true;
        let body = string_buffer_end(&mut b);
        let flags = string_buffer_end(&mut b2);
        if JS_IsException(body) != 0 || JS_IsException(flags) != 0 {
            JS_FreeValue((*s).ctx, body);
            JS_FreeValue((*s).ctx, flags);
            return -1;
        }
        (*s).token.val = TOK_REGEXP;
        (*s).token.u.regexp = JSTokenRegexp { body, flags };
        (*s).buf_ptr = p;
        0
    })();
    if result < 0 && !finalized {
        string_buffer_free(&mut b);
        string_buffer_free(&mut b2);
    }
    result
}
unsafe fn ident_realloc(
    ctx: *mut JSContext,
    pbuf: *mut *mut c_char,
    psize: *mut usize,
    static_buf: *mut c_char,
) -> i32 {
    let buf = *pbuf;
    let size = *psize;
    let new_size = if size >= (usize::MAX / 3) * 2 {
        usize::MAX
    } else {
        size + (size >> 1)
    };
    let new_buf = if buf == static_buf {
        let q = js_malloc(ctx, new_size).cast::<c_char>();
        if q.is_null() {
            return -1;
        }
        ptr::copy_nonoverlapping(buf, q, size);
        q
    } else {
        let q = js_realloc(ctx, buf.cast(), new_size).cast::<c_char>();
        if q.is_null() {
            return -1;
        }
        q
    };
    *pbuf = new_buf;
    *psize = new_size;
    0
}
unsafe fn update_token_ident(s: *mut JSParseState) {
    let atom = (*s).token.u.ident.atom;
    let fd = (*s).cur_func;
    let parent = (*fd).parent;
    if atom <= JS_ATOM_LAST_KEYWORD
        || (atom <= JS_ATOM_LAST_STRICT_KEYWORD && (*fd).js_mode & JS_MODE_STRICT as u8 != 0)
        || (atom == JS_ATOM_yield
            && ((*fd).func_kind & JS_FUNC_GENERATOR as u8 != 0
                || ((*fd).func_type == JS_PARSE_FUNC_ARROW as u8
                    && (*fd).in_function_body == 0
                    && !parent.is_null()
                    && (*parent).func_kind & JS_FUNC_GENERATOR as u8 != 0)))
        || (atom == JS_ATOM_await
            && ((*s).is_module != 0
                || (*fd).func_kind & JS_FUNC_ASYNC as u8 != 0
                || (*fd).func_type == JS_PARSE_FUNC_CLASS_STATIC_INIT as u8
                || ((*fd).func_type == JS_PARSE_FUNC_ARROW as u8
                    && (*fd).in_function_body == 0
                    && !parent.is_null()
                    && ((*parent).func_kind & JS_FUNC_ASYNC as u8 != 0
                        || (*parent).func_type == JS_PARSE_FUNC_CLASS_STATIC_INIT as u8))))
    {
        if (*s).token.u.ident.has_escape != 0 {
            (*s).token.u.ident.is_reserved = 1;
            (*s).token.val = TOK_IDENT;
        } else {
            (*s).token.val = (atom - 1) as i32 + TOK_FIRST_KEYWORD;
        }
    }
}
unsafe fn reparse_ident_token(s: *mut JSParseState) {
    if (*s).token.val == TOK_IDENT
        || (TOK_FIRST_KEYWORD..=TOK_LAST_KEYWORD).contains(&(*s).token.val)
    {
        (*s).token.val = TOK_IDENT;
        (*s).token.u.ident.is_reserved = 0;
        update_token_ident(s);
    }
}
unsafe fn parse_ident(
    s: *mut JSParseState,
    pp: *mut *const u8,
    pident_has_escape: *mut i32,
    mut c: i32,
    is_private: i32,
) -> JSAtom {
    let mut p = *pp;
    let mut ident_buf = [0i8; 128];
    let static_buf = ident_buf.as_mut_ptr();
    let mut buf = static_buf;
    let mut ident_size = 128usize;
    let mut ident_pos = 0usize;
    if is_private != 0 {
        *buf = b'#' as i8;
        ident_pos += 1;
    }
    let atom = loop {
        let mut p1 = p;
        if c < 128 {
            *buf.add(ident_pos) = c as i8;
            ident_pos += 1;
        } else {
            ident_pos += unicode_to_utf8(buf.add(ident_pos).cast(), c as u32) as usize;
        }
        c = i32::from(*p1);
        p1 = p1.add(1);
        if c == i32::from(b'\\') && *p1 == b'u' {
            c = lre_parse_escape(&mut p1, 1);
            *pident_has_escape = 1;
        } else if c >= 128 {
            c = unicode_from_utf8(p, UTF8_CHAR_LEN_MAX as i32, &mut p1);
        }
        if lre_js_is_ident_next(c as u32) == 0 {
            break JS_NewAtomLen((*s).ctx, buf, ident_pos);
        }
        p = p1;
        if ident_pos >= ident_size - UTF8_CHAR_LEN_MAX as usize
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
unsafe fn next_token(s: *mut JSParseState) -> i32 {
    if js_check_stack_overflow((*(*s).ctx).rt, 0) != 0 {
        return js_parse_error(s, c"stack overflow".as_ptr());
    }
    free_token(s, ptr::addr_of_mut!((*s).token));
    let mut p = (*s).buf_ptr;
    (*s).last_ptr = p;
    (*s).got_lf = 0;
    let ret = (|| {
        loop {
            (*s).token.ptr = p;
            let mut c = i32::from(*p);
            // The redo label in the C lexer becomes this loop. Its token source
            // position is updated after each skipped whitespace or comment.
            if c == 0 && p >= (*s).buf_end {
                (*s).token.val = TOK_EOF;
                break;
            }
            if c == i32::from(b'`') {
                if js_parse_template_part(s, p.add(1)) != 0 {
                    return -1;
                }
                p = (*s).buf_ptr;
                break;
            }
            if c == 39 || c == 34 {
                if js_parse_string(s, c, 1, p.add(1), ptr::addr_of_mut!((*s).token), &mut p) != 0 {
                    return -1;
                }
                break;
            }
            if c == 13 || c == 10 {
                if c == 13 && *p.add(1) == 10 {
                    p = p.add(1);
                }
                p = p.add(1);
                (*s).got_lf = 1;
                continue;
            }
            if c == 12 || c == 11 || c == 32 || c == 9 {
                p = p.add(1);
                continue;
            }
            if c == 47 && *p.add(1) == b'*' {
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
                    if *p == 10 || *p == 13 {
                        (*s).got_lf = 1;
                        p = p.add(1);
                    } else if *p >= 128 {
                        let cp = unicode_from_utf8(p, UTF8_CHAR_LEN_MAX as i32, &mut p);
                        if cp == CP_LS || cp == CP_PS {
                            (*s).got_lf = 1;
                        } else if cp == -1 {
                            p = p.add(1);
                        }
                    } else {
                        p = p.add(1);
                    }
                }
                continue;
            }
            let slash_comment = c == 47 && *p.add(1) == b'/';
            let html_end = c == 45
                && *p.add(1) == b'-'
                && (*s).allow_html_comments != 0
                && *p.add(2) == b'>'
                && ((*s).got_lf != 0 || (*s).last_ptr == (*s).buf_start);
            let html_start = c == 60
                && (*s).allow_html_comments != 0
                && *p.add(1) == b'!'
                && *p.add(2) == b'-'
                && *p.add(3) == b'-';
            if slash_comment || html_end || html_start {
                if slash_comment {
                    p = p.add(2);
                }
                loop {
                    if (*p == 0 && p >= (*s).buf_end) || *p == 13 || *p == 10 {
                        break;
                    }
                    if *p >= 128 {
                        let cp = unicode_from_utf8(p, UTF8_CHAR_LEN_MAX as i32, &mut p);
                        if cp == CP_LS || cp == CP_PS {
                            break;
                        } else if cp == -1 {
                            p = p.add(1);
                        }
                    } else {
                        p = p.add(1);
                    }
                }
                continue;
            }
            let mut ident = false;
            let mut ident_has_escape = 0;
            if c == 92 && *p.add(1) == b'u' {
                let mut p1 = p.add(1);
                let c1 = lre_parse_escape(&mut p1, 1);
                if c1 >= 0 && lre_js_is_ident_first(c1 as u32) != 0 {
                    c = c1;
                    p = p1;
                    ident = true;
                    ident_has_escape = 1;
                }
            } else if (*p).is_ascii_alphabetic() || *p == b'_' || *p == b'$' {
                p = p.add(1);
                ident = true;
            }
            if ident {
                let atom = parse_ident(s, &mut p, &mut ident_has_escape, c, 0);
                if atom == JS_ATOM_NULL as u32 {
                    return -1;
                }
                (*s).token.u.ident = JSTokenIdent {
                    atom,
                    has_escape: ident_has_escape,
                    is_reserved: 0,
                };
                (*s).token.val = TOK_IDENT;
                update_token_ident(s);
                break;
            }
            if c == 35 {
                p = p.add(1);
                let mut p1 = p.add(1);
                c = i32::from(*p);
                if c == 92 && *p1 == b'u' {
                    c = lre_parse_escape(&mut p1, 1);
                } else if c >= 128 {
                    c = unicode_from_utf8(p, UTF8_CHAR_LEN_MAX as i32, &mut p1);
                }
                if lre_js_is_ident_first(c as u32) == 0 {
                    js_parse_error(s, c"invalid first character of private name".as_ptr());
                    return -1;
                }
                p = p1;
                let mut escape = 0;
                let atom = parse_ident(s, &mut p, &mut escape, c, 1);
                if atom == JS_ATOM_NULL as u32 {
                    return -1;
                }
                (*s).token.u.ident.atom = atom;
                (*s).token.val = TOK_PRIVATE_NAME;
                break;
            }
            if c == 46 && *p.add(1) == b'.' && *p.add(2) == b'.' {
                p = p.add(3);
                (*s).token.val = TOK_ELLIPSIS;
                break;
            }
            if (*p).is_ascii_digit() || (c == 46 && (*p.add(1)).is_ascii_digit()) {
                if c == 48
                    && (*p.add(1)).is_ascii_digit()
                    && (*(*s).cur_func).js_mode & JS_MODE_STRICT as u8 != 0
                {
                    js_parse_error(s, c"octal literals are deprecated in strict mode".as_ptr());
                    return -1;
                }
                let flags = ATOD_ACCEPT_BIN_OCT
                    | ATOD_ACCEPT_LEGACY_OCTAL
                    | ATOD_ACCEPT_UNDERSCORES
                    | ATOD_ACCEPT_SUFFIX;
                let mut pchar = p.cast::<c_char>();
                let val = js_atof((*s).ctx, pchar, &mut pchar, 0, flags);
                p = pchar.cast();
                if JS_IsException(val) != 0 {
                    return -1;
                }
                let mut p1 = ptr::null();
                if JS_VALUE_IS_NAN(val) != 0
                    || lre_js_is_ident_next(
                        unicode_from_utf8(p, UTF8_CHAR_LEN_MAX as i32, &mut p1) as u32
                    ) != 0
                {
                    JS_FreeValue((*s).ctx, val);
                    js_parse_error(s, c"invalid number literal".as_ptr());
                    return -1;
                }
                (*s).token.val = TOK_NUMBER;
                (*s).token.u.num.val = val;
                break;
            }
            let p1 = *p.add(1);
            let mut n = 1usize;
            let tok = match c {
                47 => {
                    if p1 == b'=' {
                        n = 2;
                        TOK_DIV_ASSIGN
                    } else {
                        c
                    }
                }
                42 => {
                    if p1 == b'=' {
                        n = 2;
                        TOK_MUL_ASSIGN
                    } else if p1 == b'*' {
                        if *p.add(2) == b'=' {
                            n = 3;
                            TOK_POW_ASSIGN
                        } else {
                            n = 2;
                            TOK_POW
                        }
                    } else {
                        c
                    }
                }
                37 => {
                    if p1 == b'=' {
                        n = 2;
                        TOK_MOD_ASSIGN
                    } else {
                        c
                    }
                }
                43 => {
                    if p1 == b'=' {
                        n = 2;
                        TOK_PLUS_ASSIGN
                    } else if p1 == b'+' {
                        n = 2;
                        TOK_INC
                    } else {
                        c
                    }
                }
                45 => {
                    if p1 == b'=' {
                        n = 2;
                        TOK_MINUS_ASSIGN
                    } else if p1 == b'-' {
                        n = 2;
                        TOK_DEC
                    } else {
                        c
                    }
                }
                60 => {
                    if p1 == b'=' {
                        n = 2;
                        TOK_LTE
                    } else if p1 == b'<' {
                        if *p.add(2) == b'=' {
                            n = 3;
                            TOK_SHL_ASSIGN
                        } else {
                            n = 2;
                            TOK_SHL
                        }
                    } else {
                        c
                    }
                }
                62 => {
                    if p1 == b'=' {
                        n = 2;
                        TOK_GTE
                    } else if p1 == b'>' {
                        if *p.add(2) == b'>' {
                            if *p.add(3) == b'=' {
                                n = 4;
                                TOK_SHR_ASSIGN
                            } else {
                                n = 3;
                                TOK_SHR
                            }
                        } else if *p.add(2) == b'=' {
                            n = 3;
                            TOK_SAR_ASSIGN
                        } else {
                            n = 2;
                            TOK_SAR
                        }
                    } else {
                        c
                    }
                }
                61 => {
                    if p1 == b'=' {
                        if *p.add(2) == b'=' {
                            n = 3;
                            TOK_STRICT_EQ
                        } else {
                            n = 2;
                            TOK_EQ
                        }
                    } else if p1 == b'>' {
                        n = 2;
                        TOK_ARROW
                    } else {
                        c
                    }
                }
                33 => {
                    if p1 == b'=' {
                        if *p.add(2) == b'=' {
                            n = 3;
                            TOK_STRICT_NEQ
                        } else {
                            n = 2;
                            TOK_NEQ
                        }
                    } else {
                        c
                    }
                }
                38 => {
                    if p1 == b'=' {
                        n = 2;
                        TOK_AND_ASSIGN
                    } else if p1 == b'&' {
                        if *p.add(2) == b'=' {
                            n = 3;
                            TOK_LAND_ASSIGN
                        } else {
                            n = 2;
                            TOK_LAND
                        }
                    } else {
                        c
                    }
                }
                94 => {
                    if p1 == b'=' {
                        n = 2;
                        TOK_XOR_ASSIGN
                    } else {
                        c
                    }
                }
                124 => {
                    if p1 == b'=' {
                        n = 2;
                        TOK_OR_ASSIGN
                    } else if p1 == b'|' {
                        if *p.add(2) == b'=' {
                            n = 3;
                            TOK_LOR_ASSIGN
                        } else {
                            n = 2;
                            TOK_LOR
                        }
                    } else {
                        c
                    }
                }
                63 => {
                    if p1 == b'?' {
                        if *p.add(2) == b'=' {
                            n = 3;
                            TOK_DOUBLE_QUESTION_MARK_ASSIGN
                        } else {
                            n = 2;
                            TOK_DOUBLE_QUESTION_MARK
                        }
                    } else if p1 == b'.' && !(*p.add(2)).is_ascii_digit() {
                        n = 2;
                        TOK_QUESTION_MARK_DOT
                    } else {
                        c
                    }
                }
                _ => {
                    if c >= 128 {
                        c = unicode_from_utf8(p, UTF8_CHAR_LEN_MAX as i32, &mut p);
                        if c == CP_PS || c == CP_LS {
                            (*s).got_lf = 1;
                            continue;
                        }
                        if lre_is_space(c as u32) != 0 {
                            continue;
                        }
                        if lre_js_is_ident_first(c as u32) != 0 {
                            let mut escape = 0;
                            let atom = parse_ident(s, &mut p, &mut escape, c, 0);
                            if atom == JS_ATOM_NULL as u32 {
                                return -1;
                            }
                            (*s).token.val = TOK_IDENT;
                            (*s).token.u.ident = JSTokenIdent {
                                atom,
                                has_escape: escape,
                                is_reserved: 0,
                            };
                            update_token_ident(s);
                            break;
                        }
                        js_parse_error(s, c"unexpected character".as_ptr());
                        return -1;
                    }
                    c
                }
            };
            (*s).token.val = tok;
            p = p.add(n);
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
