// Official quickjs.c:49103..49127,49697..49704. JSON parser/reviver records. MIT.
#[repr(C)]
#[derive(Clone, Copy)]
struct JSONParseRecordObject { count: i32, hash_size: u32, entries: *mut JSONParseRecordEntry, hash_table: *mut u32 }
#[repr(C)]
#[derive(Clone, Copy)]
struct JSONParseRecordArray { count: i32, elements: *mut JSONParseRecord }
#[repr(C)]
#[derive(Clone, Copy)]
struct JSONParseRecordPrimitive { source_pos: u32, source_len: u32 }
#[repr(C)]
#[derive(Clone, Copy)]
union JSONParseRecordUnion { obj: JSONParseRecordObject, array: JSONParseRecordArray, primitive: JSONParseRecordPrimitive }
#[repr(C)]
struct JSONParseRecord { value: JSValue, u: JSONParseRecordUnion }
#[repr(C)]
struct JSONParseRecordEntry { atom: JSAtom, hash_next: u32, parse_record: JSONParseRecord }
#[repr(C)]
struct JSONStringifyContext { replacer_func: JSValueConst, stack: JSValue, property_list: JSValue, gap: JSValue, empty: JSValue, b: *mut StringBuffer }
// snprintf(buf, capacity, "\\u%04x", codepoint), used only for isolated
// UTF-16 surrogates and control bytes in the original quoted-string loop.
unsafe fn json_format_unicode(buf: *mut c_char, capacity: usize, c: u32) -> i32 {
    let mut bytes = [0u8; 10]; bytes[0] = b'\\'; bytes[1] = b'u';
    let hex_digits = ((32 - c.leading_zeros()).div_ceil(4) as usize).max(4);
    for i in 0..hex_digits { let d = ((c >> ((hex_digits - i - 1) * 4)) & 15) as u8; bytes[i + 2] = if d < 10 { b'0' + d } else { b'a' + d - 10 }; }
    if capacity != 0 { let n = (hex_digits + 2).min(capacity - 1); core::ptr::copy_nonoverlapping(bytes.as_ptr().cast(), buf, n); *buf.add(n) = 0; }
    (hex_digits + 2) as i32
}
