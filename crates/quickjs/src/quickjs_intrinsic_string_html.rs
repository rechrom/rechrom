// quickjs.c 46546..46617, Annex B String.CreateHTML. MIT.
const magic_string_anchor: i32 = 0;
const magic_string_big: i32 = 1;
const magic_string_blink: i32 = 2;
const magic_string_bold: i32 = 3;
const magic_string_fixed: i32 = 4;
const magic_string_fontcolor: i32 = 5;
const magic_string_fontsize: i32 = 6;
const magic_string_italics: i32 = 7;
const magic_string_link: i32 = 8;
const magic_string_small: i32 = 9;
const magic_string_strike: i32 = 10;
const magic_string_sub: i32 = 11;
const magic_string_sup: i32 = 12;
unsafe fn js_string_CreateHTML(ctx: *mut JSContext, this_val: JSValueConst, _argc: i32, argv: *mut JSValueConst, magic: i32) -> JSValue {
    // Keeping references to CStr gives immutable Rust-owned pointer storage.
    let defs = [
        (c"a", Some(c"name")), (c"big", None), (c"blink", None), (c"b", None),
        (c"tt", None), (c"font", Some(c"color")), (c"font", Some(c"size")),
        (c"i", None), (c"a", Some(c"href")), (c"small", None),
        (c"strike", None), (c"sub", None), (c"sup", None),
    ];
    let str = JS_ToStringCheckObject(ctx, this_val);
    if JS_IsException(str) != 0 { return JS_EXCEPTION; }
    let mut b_s: StringBuffer = core::mem::zeroed();
    let b = &mut b_s;
    string_buffer_init(ctx, b, 7);
    string_buffer_putc8(b, b'<' as u32);
    let (tag, attr) = defs[magic as usize];
    string_buffer_puts8(b, tag.as_ptr());
    if let Some(attr) = attr {
        string_buffer_putc8(b, b' ' as u32);
        string_buffer_puts8(b, attr.as_ptr());
        string_buffer_puts8(b, c"=\"".as_ptr());
        let value = JS_ToStringCheckObject(ctx, *argv);
        if JS_IsException(value) != 0 {
            JS_FreeValue(ctx, str);
            string_buffer_free(b);
            return JS_EXCEPTION;
        }
        let p = JS_VALUE_GET_PTR(value).cast::<JSString>();
        for i in 0..(*p).len() {
            let c = string_get(p, i as i32);
            if c == b'"' as i32 {
                string_buffer_puts8(b, c"&quot;".as_ptr());
            } else {
                string_buffer_putc16(b, c as u32);
            }
        }
        JS_FreeValue(ctx, value);
        string_buffer_putc8(b, b'"' as u32);
    }
    string_buffer_putc8(b, b'>' as u32);
    string_buffer_concat_value_free(b, str);
    string_buffer_puts8(b, c"</".as_ptr());
    string_buffer_puts8(b, tag.as_ptr());
    string_buffer_putc8(b, b'>' as u32);
    string_buffer_end(b)
}

// Original DynBufReallocFunc cast, with an explicit Rust typed boundary.
unsafe fn js_string_normalize_realloc(opaque:*mut c_void, p:*mut c_void, size:usize)->*mut c_void { js_realloc_rt(opaque.cast::<JSRuntime>(), p, size) }
