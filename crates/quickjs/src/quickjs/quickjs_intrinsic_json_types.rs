// Official quickjs.c:49103..49127,49697..49704. JSON parser/reviver records. MIT.
use super::*;
#[repr(C)]
#[derive(Clone, Copy)]
pub(crate) struct JSONParseRecordObject { pub(crate) count: i32, pub(crate) hash_size: u32, pub(crate) entries: *mut JSONParseRecordEntry, pub(crate) hash_table: *mut u32 }
#[repr(C)]
#[derive(Clone, Copy)]
pub(crate) struct JSONParseRecordArray { pub(crate) count: i32, pub(crate) elements: *mut JSONParseRecord }
#[repr(C)]
#[derive(Clone, Copy)]
pub(crate) struct JSONParseRecordPrimitive { pub(crate) source_pos: u32, pub(crate) source_len: u32 }
#[repr(C)]
#[derive(Clone, Copy)]
pub(crate) union JSONParseRecordUnion { pub(crate) obj: JSONParseRecordObject, pub(crate) array: JSONParseRecordArray, pub(crate) primitive: JSONParseRecordPrimitive }
#[repr(C)]
pub(crate) struct JSONParseRecord { pub(crate) value: JSValue, pub(crate) u: JSONParseRecordUnion }
#[repr(C)]
pub(crate) struct JSONParseRecordEntry { pub(crate) atom: JSAtom, pub(crate) hash_next: u32, pub(crate) parse_record: JSONParseRecord }
#[repr(C)]
pub(crate) struct JSONStringifyContext { pub(crate) replacer_func: JSValueConst, pub(crate) stack: JSValue, pub(crate) property_list: JSValue, pub(crate) gap: JSValue, pub(crate) empty: JSValue, pub(crate) b: *mut StringBuffer }
// snprintf(buf, capacity, "\\u%04x", codepoint), used only for isolated
// UTF-16 surrogates and control bytes in the original quoted-string loop.
pub(crate) unsafe fn json_format_unicode(buf: *mut c_char, capacity: usize, c: u32) -> i32 {
    let mut bytes = [0u8; 10]; bytes[0] = b'\\'; bytes[1] = b'u';
    let hex_digits = ((32 - c.leading_zeros()).div_ceil(4) as usize).max(4);
    for i in 0..hex_digits { let d = ((c >> ((hex_digits - i - 1) * 4)) & 15) as u8; bytes[i + 2] = if d < 10 { b'0' + d } else { b'a' + d - 10 }; }
    if capacity != 0 { let n = (hex_digits + 2).min(capacity - 1); core::ptr::copy_nonoverlapping(bytes.as_ptr().cast(), buf, n); *buf.add(n) = 0; }
    (hex_digits + 2) as i32
}
