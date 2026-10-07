// quickjs.c:7504-7795. MIT. Awaiting full DefineProperty and the production
// string/object/context path. Stable Rust replaces C varargs with typed
// messages; the 256-byte message buffer and truncation remain unchanged.
pub enum JSErrorMessage<'a> {
    C(*const c_char),
    Args(core::fmt::Arguments<'a>),
    Pieces(&'a [&'a [u8]]),
}
impl<'a> From<*const c_char> for JSErrorMessage<'a> {
    fn from(p: *const c_char) -> Self {
        Self::C(p)
    }
}
impl<'a> From<core::fmt::Arguments<'a>> for JSErrorMessage<'a> {
    fn from(a: core::fmt::Arguments<'a>) -> Self {
        Self::Args(a)
    }
}
struct ErrorBufferWriter<'a> {
    buf: &'a mut [u8],
    pos: usize,
}
impl core::fmt::Write for ErrorBufferWriter<'_> {
    fn write_str(&mut self, text: &str) -> core::fmt::Result {
        let remaining = self.buf.len().saturating_sub(self.pos);
        let n = remaining.min(text.len());
        self.buf[self.pos..self.pos + n].copy_from_slice(&text.as_bytes()[..n]);
        self.pos += n;
        Ok(())
    }
}
unsafe fn format_error_message(buf: &mut [u8; 256], message: JSErrorMessage<'_>) {
    match message {
        JSErrorMessage::C(p) => {
            let bytes = core::ffi::CStr::from_ptr(p).to_bytes();
            let n = bytes.len().min(255);
            buf[..n].copy_from_slice(&bytes[..n]);
            buf[n] = 0;
        }
        JSErrorMessage::Args(args) => {
            let mut writer = ErrorBufferWriter {
                buf: &mut buf[..255],
                pos: 0,
            };
            core::fmt::write(&mut writer, args).expect("bounded error writer");
            let n = writer.pos;
            buf[n] = 0;
        }
        JSErrorMessage::Pieces(parts) => {
            let mut pos = 0;
            for part in parts {
                let n = part.len().min(255 - pos);
                buf[pos..pos + n].copy_from_slice(&part[..n]);
                pos += n;
            }
            buf[pos] = 0;
        }
    }
}
unsafe fn get_prop_string(ctx: *mut JSContext, obj: JSValueConst, prop: JSAtom) -> *const c_char {
    if JS_VALUE_GET_TAG(obj) != JS_TAG_OBJECT {
        return ptr::null();
    }
    let mut p = JS_VALUE_GET_PTR(obj).cast::<JSObject>();
    let mut pr = ptr::null_mut();
    let mut prs = find_own_property(&mut pr, p, prop);
    if prs.is_null() {
        p = (*(*p).shape).proto;
        if p.is_null() {
            return ptr::null();
        }
        prs = find_own_property(&mut pr, p, prop);
        if prs.is_null() {
            return ptr::null();
        }
    }
    if (*prs).flags() as i32 & JS_PROP_TMASK != JS_PROP_NORMAL {
        return ptr::null();
    }
    let val = (*pr).u.value;
    if JS_VALUE_GET_TAG(val) != JS_TAG_STRING {
        return ptr::null();
    }
    JS_ToCStringLen2(ctx, ptr::null_mut(), val, 0)
}
const JS_BACKTRACE_FLAG_SKIP_FIRST_LEVEL: i32 = 1 << 0;
// A raw C string goes to DynBuf as bytes, preserving surrogate UTF-8 and NUL
// termination semantics. Numeric printf sites use the existing typed adapter.
unsafe fn backtrace_printf_s(
    s: *mut crate::cutils_header::DynBuf,
    prefix: &[u8],
    text: *const c_char,
) {
    let bytes = core::ffi::CStr::from_ptr(text).to_bytes();
    crate::cutils::dbuf_printf(s, |dst, cap| {
        let len = prefix.len().wrapping_add(bytes.len());
        if cap != 0 {
            let first = prefix.len().min(cap - 1);
            ptr::copy_nonoverlapping(prefix.as_ptr(), dst, first);
            let second = bytes.len().min((cap - 1).saturating_sub(first));
            ptr::copy_nonoverlapping(bytes.as_ptr(), dst.add(first), second);
            *dst.add(first + second) = 0;
        }
        len as i32
    });
}
unsafe fn backtrace_puts(s: *mut crate::cutils_header::DynBuf, text: *const c_char) {
    backtrace_printf_s(s, b"", text);
}
unsafe fn build_backtrace(
    ctx: *mut JSContext,
    error_obj: JSValueConst,
    filename: *const c_char,
    line_num: i32,
    col_num: i32,
    mut backtrace_flags: i32,
) {
    js_host_error_creation(ctx, error_obj, filename, line_num, col_num);
    use crate::cutils::{dbuf_error, dbuf_free, dbuf_printf_args, dbuf_putc};
    use crate::quickjs_atom::{
        JS_ATOM_columnNumber, JS_ATOM_fileName, JS_ATOM_lineNumber, JS_ATOM_name, JS_ATOM_stack,
    };
    if JS_IsObject(error_obj) == 0 {
        return;
    }
    let mut dbuf: crate::cutils_header::DynBuf = core::mem::zeroed();
    js_dbuf_init(ctx, &mut dbuf);
    let flags = JS_PROP_WRITABLE | JS_PROP_CONFIGURABLE;
    if !filename.is_null() {
        backtrace_printf_s(&mut dbuf, b"    at ", filename);
        if line_num != -1 {
            dbuf_printf_args(&mut dbuf, format_args!(":{}:{}", line_num, col_num));
        }
        dbuf_putc(&mut dbuf, b'\n');
        let str = JS_NewStringLen(
            ctx,
            filename,
            core::ffi::CStr::from_ptr(filename).to_bytes().len(),
        );
        if JS_IsException(str) != 0 {
            return;
        }
        if JS_DefinePropertyValue(ctx, error_obj, JS_ATOM_fileName, str, flags) < 0
            || JS_DefinePropertyValue(
                ctx,
                error_obj,
                JS_ATOM_lineNumber,
                JS_NewInt32(ctx, line_num),
                flags,
            ) < 0
            || JS_DefinePropertyValue(
                ctx,
                error_obj,
                JS_ATOM_columnNumber,
                JS_NewInt32(ctx, col_num),
                flags,
            ) < 0
        {
            return;
        }
    }
    let mut sf = (*(*ctx).rt).current_stack_frame;
    while !sf.is_null() {
        if (*sf).js_mode & JS_MODE_BACKTRACE_BARRIER != 0 {
            break;
        }
        if backtrace_flags & JS_BACKTRACE_FLAG_SKIP_FIRST_LEVEL != 0 {
            backtrace_flags &= !JS_BACKTRACE_FLAG_SKIP_FIRST_LEVEL;
            sf = (*sf).prev_frame;
            continue;
        }
        let func_name_str = get_prop_string(ctx, (*sf).cur_func, JS_ATOM_name);
        let str1 = if func_name_str.is_null() || *func_name_str == 0 {
            c"<anonymous>".as_ptr()
        } else {
            func_name_str
        };
        backtrace_printf_s(&mut dbuf, b"    at ", str1);
        JS_FreeCString(ctx, func_name_str);
        let p = JS_VALUE_GET_PTR((*sf).cur_func).cast::<JSObject>();
        if js_class_has_bytecode((*p).class_id as u32) != 0 {
            let b = (*p).u.func.function_bytecode;
            if (*b).has_debug() != 0 {
                let mut col_num1 = 0;
                let line_num1 = find_line_num(
                    ctx,
                    b,
                    ((*sf).cur_pc.offset_from((*b).byte_code_buf) - 1) as u32,
                    &mut col_num1,
                );
                let atom_str = JS_AtomToCStringLen(ctx, ptr::null_mut(), (*b).debug.filename);
                backtrace_printf_s(
                    &mut dbuf,
                    b" (",
                    if atom_str.is_null() {
                        c"<null>".as_ptr()
                    } else {
                        atom_str
                    },
                );
                JS_FreeCString(ctx, atom_str);
                if line_num1 != 0 {
                    dbuf_printf_args(&mut dbuf, format_args!(":{}:{}", line_num1, col_num1));
                }
                dbuf_putc(&mut dbuf, b')');
            }
        } else {
            backtrace_puts(&mut dbuf, c" (native)".as_ptr());
        }
        dbuf_putc(&mut dbuf, b'\n');
        sf = (*sf).prev_frame;
    }
    dbuf_putc(&mut dbuf, 0);
    let str = if dbuf_error(&dbuf) != 0 {
        JS_NULL
    } else {
        JS_NewStringLen(
            ctx,
            dbuf.buf.cast(),
            core::ffi::CStr::from_ptr(dbuf.buf.cast()).to_bytes().len(),
        )
    };
    dbuf_free(&mut dbuf);
    JS_DefinePropertyValue(ctx, error_obj, JS_ATOM_stack, str, flags);
}
unsafe fn is_backtrace_needed(_ctx: *mut JSContext, obj: JSValueConst) -> JS_BOOL {
    if JS_VALUE_GET_TAG(obj) != JS_TAG_OBJECT {
        return 0;
    }
    let p = JS_VALUE_GET_PTR(obj).cast::<JSObject>();
    if (*p).class_id as u32 != JS_CLASS_ERROR {
        return 0;
    }
    find_own_property1(p, crate::quickjs_atom::JS_ATOM_stack).is_null() as i32
}
unsafe fn JS_ThrowError2(
    ctx: *mut JSContext,
    error_num: JSErrorEnum,
    message: JSErrorMessage<'_>,
    add_backtrace: JS_BOOL,
) -> JSValue {
    let mut buf = [0u8; 256];
    format_error_message(&mut buf, message);
    let mut obj = JS_NewObjectProtoClass(
        ctx,
        (*ctx).native_error_proto[error_num as usize],
        JS_CLASS_ERROR,
    );
    if JS_IsException(obj) != 0 {
        obj = JS_NULL;
    } else {
        JS_DefinePropertyValue(
            ctx,
            obj,
            crate::quickjs_atom::JS_ATOM_message,
            JS_NewStringLen(
                ctx,
                buf.as_ptr().cast(),
                core::ffi::CStr::from_ptr(buf.as_ptr().cast())
                    .to_bytes()
                    .len(),
            ),
            JS_PROP_WRITABLE | JS_PROP_CONFIGURABLE,
        );
        if add_backtrace != 0 {
            build_backtrace(ctx, obj, ptr::null(), 0, 0, 0);
        }
    }
    JS_Throw(ctx, obj)
}
unsafe fn JS_ThrowError(
    ctx: *mut JSContext,
    error_num: JSErrorEnum,
    message: JSErrorMessage<'_>,
) -> JSValue {
    let rt = (*ctx).rt;
    let sf = (*rt).current_stack_frame;
    let add_backtrace = ((*rt).in_out_of_memory == 0
        && (sf.is_null() || JS_GetFunctionBytecode((*sf).cur_func).is_null()))
        as i32;
    JS_ThrowError2(ctx, error_num, message, add_backtrace)
}
pub unsafe fn JS_ThrowSyntaxError<'a>(
    ctx: *mut JSContext,
    message: impl Into<JSErrorMessage<'a>>,
) -> JSValue {
    JS_ThrowError(ctx, JS_SYNTAX_ERROR, message.into())
}
pub unsafe fn JS_ThrowTypeError<'a>(
    ctx: *mut JSContext,
    message: impl Into<JSErrorMessage<'a>>,
) -> JSValue {
    JS_ThrowError(ctx, JS_TYPE_ERROR, message.into())
}
pub unsafe fn JS_ThrowReferenceError<'a>(
    ctx: *mut JSContext,
    message: impl Into<JSErrorMessage<'a>>,
) -> JSValue {
    JS_ThrowError(ctx, JS_REFERENCE_ERROR, message.into())
}
pub unsafe fn JS_ThrowRangeError<'a>(
    ctx: *mut JSContext,
    message: impl Into<JSErrorMessage<'a>>,
) -> JSValue {
    JS_ThrowError(ctx, JS_RANGE_ERROR, message.into())
}
pub unsafe fn JS_ThrowInternalError<'a>(
    ctx: *mut JSContext,
    message: impl Into<JSErrorMessage<'a>>,
) -> JSValue {
    JS_ThrowError(ctx, JS_INTERNAL_ERROR, message.into())
}
pub unsafe fn JS_ThrowOutOfMemory(ctx: *mut JSContext) -> JSValue {
    let rt = (*ctx).rt;
    if (*rt).in_out_of_memory == 0 {
        (*rt).in_out_of_memory = 1;
        JS_ThrowInternalError(ctx, c"out of memory".as_ptr());
        (*rt).in_out_of_memory = 0;
    }
    JS_EXCEPTION
}
unsafe fn JS_ThrowStackOverflow(ctx: *mut JSContext) -> JSValue {
    JS_ThrowInternalError(ctx, c"stack overflow".as_ptr())
}
unsafe fn JS_ThrowTypeErrorOrFalse<'a>(
    ctx: *mut JSContext,
    flags: i32,
    message: impl Into<JSErrorMessage<'a>>,
) -> i32 {
    if flags & JS_PROP_THROW != 0 || flags & JS_PROP_THROW_STRICT != 0 && is_strict_mode(ctx) != 0 {
        JS_ThrowTypeError(ctx, message);
        -1
    } else {
        0
    }
}
unsafe fn JS_ThrowTypeErrorReadOnly(ctx: *mut JSContext, flags: i32, atom: JSAtom) -> i32 {
    if flags & JS_PROP_THROW != 0 || flags & JS_PROP_THROW_STRICT != 0 && is_strict_mode(ctx) != 0 {
        let mut buf = [0i8; ATOM_GET_STR_BUF_SIZE];
        let name = JS_AtomGetStr(ctx, buf.as_mut_ptr(), buf.len() as i32, atom);
        JS_ThrowTypeError(
            ctx,
            JSErrorMessage::Pieces(&[
                b"'",
                core::ffi::CStr::from_ptr(name).to_bytes(),
                b"' is read-only",
            ]),
        );
        -1
    } else {
        0
    }
}
unsafe fn JS_ThrowTypeErrorNotAnObject(ctx: *mut JSContext) -> JSValue {
    JS_ThrowTypeError(ctx, c"not an object".as_ptr())
}
unsafe fn JS_ThrowTypeErrorNotAConstructor(ctx: *mut JSContext, func_obj: JSValueConst) -> JSValue {
    if JS_IsFunction(ctx, func_obj) == 0 {
        return JS_ThrowTypeError(ctx, c"not a constructor".as_ptr());
    }
    let name = get_prop_string(ctx, func_obj, crate::quickjs_atom::JS_ATOM_name);
    if name.is_null() {
        return JS_ThrowTypeError(ctx, c"not a constructor".as_ptr());
    }
    JS_ThrowTypeError(
        ctx,
        JSErrorMessage::Pieces(&[
            core::ffi::CStr::from_ptr(name).to_bytes(),
            b" is not a constructor",
        ]),
    );
    JS_FreeCString(ctx, name);
    JS_EXCEPTION
}
unsafe fn JS_ThrowTypeErrorNotASymbol(ctx: *mut JSContext) -> JSValue {
    JS_ThrowTypeError(ctx, c"not a symbol".as_ptr())
}
unsafe fn JS_ThrowReferenceErrorNotDefined(ctx: *mut JSContext, name: JSAtom) -> JSValue {
    let mut buf = [0i8; ATOM_GET_STR_BUF_SIZE];
    let name = JS_AtomGetStr(ctx, buf.as_mut_ptr(), buf.len() as i32, name);
    JS_ThrowReferenceError(
        ctx,
        JSErrorMessage::Pieces(&[
            b"'",
            core::ffi::CStr::from_ptr(name).to_bytes(),
            b"' is not defined",
        ]),
    )
}
unsafe fn JS_ThrowReferenceErrorUninitialized(ctx: *mut JSContext, name: JSAtom) -> JSValue {
    let mut buf = [0i8; ATOM_GET_STR_BUF_SIZE];
    let name = if name == JS_ATOM_NULL as u32 {
        c"lexical variable".as_ptr()
    } else {
        JS_AtomGetStr(ctx, buf.as_mut_ptr(), buf.len() as i32, name)
    };
    JS_ThrowReferenceError(
        ctx,
        JSErrorMessage::Pieces(&[
            core::ffi::CStr::from_ptr(name).to_bytes(),
            b" is not initialized",
        ]),
    )
}
unsafe fn JS_ThrowReferenceErrorUninitialized2(
    ctx: *mut JSContext,
    b: *mut JSFunctionBytecode,
    idx: i32,
    is_ref: JS_BOOL,
) -> JSValue {
    let mut atom = JS_ATOM_NULL as u32;
    if is_ref != 0 {
        atom = (*(*b).closure_var.offset(idx as isize)).var_name;
    } else if !(*b).vardefs.is_null() {
        atom = (*(*b).vardefs.offset((*b).arg_count as isize + idx as isize)).var_name;
    }
    JS_ThrowReferenceErrorUninitialized(ctx, atom)
}
unsafe fn JS_ThrowTypeErrorInvalidClass(ctx: *mut JSContext, class_id: i32) -> JSValue {
    let atom = (*(*(*ctx).rt).class_array.offset(class_id as isize)).class_name;
    let mut buf = [0i8; ATOM_GET_STR_BUF_SIZE];
    let name = JS_AtomGetStr(ctx, buf.as_mut_ptr(), buf.len() as i32, atom);
    JS_ThrowTypeError(
        ctx,
        JSErrorMessage::Pieces(&[
            core::ffi::CStr::from_ptr(name).to_bytes(),
            b" object expected",
        ]),
    )
}
unsafe fn JS_ThrowInterrupted(ctx: *mut JSContext) {
    JS_ThrowInternalError(ctx, c"interrupted".as_ptr());
    JS_SetUncatchableException(ctx, 1);
}
#[inline(never)]
unsafe fn __js_poll_interrupts(ctx: *mut JSContext) -> i32 {
    __js_poll_interrupts_at(ctx, ptr::null())
}
#[inline(never)]
unsafe fn __js_poll_interrupts_at(ctx: *mut JSContext, current_pc: *const u8) -> i32 {
    let rt = (*ctx).rt;
    (*ctx).interrupt_counter = JS_INTERRUPT_COUNTER_INIT;
    js_job_stack_profile_poll(ctx, current_pc);
    if let Some(handler) = (*rt).interrupt_handler {
        if handler(rt, (*rt).interrupt_opaque) != 0 {
            JS_ThrowInterrupted(ctx);
            return -1;
        }
    }
    0
}
#[inline]
unsafe fn js_poll_interrupts(ctx: *mut JSContext) -> i32 {
    (*ctx).interrupt_counter -= 1;
    if (*ctx).interrupt_counter <= 0 {
        __js_poll_interrupts(ctx)
    } else {
        0
    }
}
// Same interrupt countdown and handler as js_poll_interrupts. The optional
// diagnostic receives the current VM PC without changing the saved frame PC.
#[inline]
unsafe fn js_poll_interrupts_at(ctx: *mut JSContext, current_pc: *const u8) -> i32 {
    (*ctx).interrupt_counter -= 1;
    if (*ctx).interrupt_counter <= 0 {
        __js_poll_interrupts_at(ctx, current_pc)
    } else {
        0
    }
}
// C atom-error helpers have exactly one %s argument. Keep its raw bytes so
// invalid UTF-8 and truncation agree with snprintf before JS_NewStringLen.
unsafe fn __JS_ThrowTypeErrorAtom(
    ctx: *mut JSContext,
    atom: JSAtom,
    fmt: *const c_char,
) -> JSValue {
    let mut buf = [0i8; ATOM_GET_STR_BUF_SIZE];
    let name = JS_AtomGetStr(ctx, buf.as_mut_ptr(), buf.len() as i32, atom);
    let fmt = core::ffi::CStr::from_ptr(fmt).to_bytes();
    let i = fmt
        .windows(2)
        .position(|p| p == b"%s")
        .expect("single atom placeholder");
    JS_ThrowTypeError(
        ctx,
        JSErrorMessage::Pieces(&[
            &fmt[..i],
            core::ffi::CStr::from_ptr(name).to_bytes(),
            &fmt[i + 2..],
        ]),
    )
}
unsafe fn __JS_ThrowSyntaxErrorAtom(
    ctx: *mut JSContext,
    atom: JSAtom,
    fmt: *const c_char,
) -> JSValue {
    let mut buf = [0i8; ATOM_GET_STR_BUF_SIZE];
    let name = JS_AtomGetStr(ctx, buf.as_mut_ptr(), buf.len() as i32, atom);
    let fmt = core::ffi::CStr::from_ptr(fmt).to_bytes();
    let i = fmt
        .windows(2)
        .position(|p| p == b"%s")
        .expect("single atom placeholder");
    JS_ThrowSyntaxError(
        ctx,
        JSErrorMessage::Pieces(&[
            &fmt[..i],
            core::ffi::CStr::from_ptr(name).to_bytes(),
            &fmt[i + 2..],
        ]),
    )
}
unsafe fn JS_ThrowTypeErrorAtom(ctx: *mut JSContext, fmt: *const c_char, atom: JSAtom) -> JSValue {
    __JS_ThrowTypeErrorAtom(ctx, atom, fmt)
}
unsafe fn JS_ThrowSyntaxErrorAtom(
    ctx: *mut JSContext,
    fmt: *const c_char,
    atom: JSAtom,
) -> JSValue {
    __JS_ThrowSyntaxErrorAtom(ctx, atom, fmt)
}
