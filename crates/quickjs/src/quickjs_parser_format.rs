// Rust adapter for the C parser's printf varargs. This is not a replacement
// parser function: its callers keep original formats and argument order.
// Messages retain the upstream bounded 256-byte buffer, raw %s bytes and
// byte-valued %c, including NUL termination and truncation.
enum ParserFormatArg {
    C(*const c_char),
    Slice(*const c_char, i32),
    Char(u8),
    Signed(i32),
    Unsigned(u32),
}
unsafe fn parser_format_message(format: *const c_char, args: &[ParserFormatArg]) -> [u8; 256] {
    let fmt = core::ffi::CStr::from_ptr(format).to_bytes();
    let mut buf = [0u8; 256];
    let mut pos = 0usize;
    let mut at = 0usize;
    let mut arg = 0usize;
    let append = |buf: &mut [u8; 256], pos: &mut usize, text: &[u8]| {
        let n = (255 - *pos).min(text.len());
        buf[*pos..*pos + n].copy_from_slice(&text[..n]);
        *pos += n;
    };
    while at < fmt.len() {
        if fmt[at] != b'%' {
            append(&mut buf, &mut pos, &fmt[at..at + 1]);
            at += 1;
            continue;
        }
        at += 1;
        let mut spec = fmt[at];
        at += 1;
        if spec == b'.' {
            assert_eq!(fmt[at], b'*');
            assert_eq!(fmt[at + 1], b's');
            at += 2;
            spec = b'S';
        }
        if spec == b'%' {
            append(&mut buf, &mut pos, b"%");
            continue;
        }
        match (&args[arg], spec) {
            (ParserFormatArg::C(p), b's') => {
                append(
                    &mut buf,
                    &mut pos,
                    if p.is_null() {
                        b"(null)"
                    } else {
                        core::ffi::CStr::from_ptr(*p).to_bytes()
                    },
                );
            }
            (ParserFormatArg::Slice(p, n), b'S') => {
                let text = core::ffi::CStr::from_ptr(*p).to_bytes();
                append(
                    &mut buf,
                    &mut pos,
                    &text[..if *n < 0 {
                        text.len()
                    } else {
                        text.len().min(*n as usize)
                    }],
                );
            }
            (ParserFormatArg::Char(c), b'c') => append(&mut buf, &mut pos, &[*c]),
            (ParserFormatArg::Signed(n), b'd') => {
                let mut w = ErrorBufferWriter {
                    buf: &mut buf[pos..255],
                    pos: 0,
                };
                core::fmt::write(&mut w, format_args!("{}", n))
                    .expect("bounded parser error writer");
                pos += w.pos;
            }
            (ParserFormatArg::Unsigned(n), b'u') => {
                let mut w = ErrorBufferWriter {
                    buf: &mut buf[pos..255],
                    pos: 0,
                };
                core::fmt::write(&mut w, format_args!("{}", n))
                    .expect("bounded parser error writer");
                pos += w.pos;
            }
            _ => unreachable!("parser printf spec/type mismatch"),
        }
        arg += 1;
    }
    buf[pos] = 0;
    buf
}
unsafe fn js_parse_error_cargs(
    s: *mut JSParseState,
    format: *const c_char,
    args: &[ParserFormatArg],
) -> i32 {
    let buf = parser_format_message(format, args);
    js_parse_error(s, buf.as_ptr().cast::<c_char>())
}
unsafe fn js_parse_error_pos_cargs(
    s: *mut JSParseState,
    p: *const u8,
    format: *const c_char,
    args: &[ParserFormatArg],
) -> i32 {
    let buf = parser_format_message(format, args);
    js_parse_error_pos(s, p, buf.as_ptr().cast::<c_char>())
}
unsafe fn JS_ThrowSyntaxError_cargs(
    ctx: *mut JSContext,
    format: *const c_char,
    args: &[ParserFormatArg],
) -> JSValue {
    let buf = parser_format_message(format, args);
    JS_ThrowSyntaxError(ctx, buf.as_ptr().cast::<c_char>())
}
unsafe fn JS_ThrowTypeError_cargs(
    ctx: *mut JSContext,
    format: *const c_char,
    args: &[ParserFormatArg],
) -> JSValue {
    let buf = parser_format_message(format, args);
    JS_ThrowTypeError(ctx, buf.as_ptr().cast::<c_char>())
}
unsafe fn JS_ThrowInternalError_cargs(
    ctx: *mut JSContext,
    format: *const c_char,
    args: &[ParserFormatArg],
) -> JSValue {
    let buf = parser_format_message(format, args);
    JS_ThrowInternalError(ctx, buf.as_ptr().cast::<c_char>())
}
unsafe fn JS_ThrowReferenceError_cargs(ctx: *mut JSContext, format: *const c_char, args: &[ParserFormatArg]) -> JSValue {
    let buf = parser_format_message(format, args);
    JS_ThrowReferenceError(ctx, buf.as_ptr().cast::<c_char>())
}
