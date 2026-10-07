// Parser portion of libregexp.c, Copyright 2017-2018 Fabrice Bellard, MIT.
// Included in libregexp.rs to retain the original shared private symbol scope.
const TMP_BUF_SIZE: usize = 128;
#[repr(C)]
union REParseUnion {
    error_msg: [c_char; TMP_BUF_SIZE],
    tmp_buf: [c_char; TMP_BUF_SIZE],
}
#[repr(C)]
struct REParseState {
    byte_code: DynBuf,
    buf_ptr: *const u8,
    buf_end: *const u8,
    buf_start: *const u8,
    re_flags: i32,
    is_unicode: BOOL,
    unicode_sets: BOOL,
    ignore_case: BOOL,
    multi_line: BOOL,
    dotall: BOOL,
    group_name_scope: u8,
    capture_count: i32,
    total_capture_count: i32,
    has_named_captures: i32,
    opaque: *mut c_void,
    group_names: DynBuf,
    u: REParseUnion,
}
#[repr(C)]
struct REString {
    next: *mut REString,
    hash: u32,
    len: u32,
    buf: [u32; 0],
}
unsafe fn string_buf(p: *mut REString) -> *mut u32 {
    ptr::addr_of_mut!((*p).buf).cast()
}
#[repr(C)]
struct REStringList {
    cr: CharRange,
    n_strings: u32,
    hash_size: u32,
    hash_bits: i32,
    hash_table: *mut *mut REString,
}
unsafe fn dbuf_insert(s: *mut DynBuf, pos: i32, len: i32) -> i32 {
    if dbuf_claim(s, len as usize) != 0 {
        return -1;
    }
    ptr::copy(
        (*s).buf.add(pos as usize),
        (*s).buf.add((pos + len) as usize),
        (*s).size - pos as usize,
    );
    (*s).size += len as usize;
    0
}
unsafe fn re_string_hash(len: i32, buf: *const u32) -> u32 {
    let mut h = 1u32;
    for i in 0..len {
        h = h.wrapping_mul(263).wrapping_add(*buf.add(i as usize));
    }
    h.wrapping_mul(0x61c88647)
}
unsafe fn re_string_list_init(s1: *mut REParseState, s: *mut REStringList) {
    cr_init(&mut (*s).cr, (*s1).opaque, Some(lre_realloc));
    (*s).n_strings = 0;
    (*s).hash_size = 0;
    (*s).hash_bits = 0;
    (*s).hash_table = ptr::null_mut();
}
unsafe fn re_string_list_free(s: *mut REStringList) {
    for i in 0..(*s).hash_size {
        let mut p = *(*s).hash_table.add(i as usize);
        while !p.is_null() {
            let next = (*p).next;
            lre_realloc((*s).cr.mem_opaque, p.cast(), 0);
            p = next;
        }
    }
    lre_realloc((*s).cr.mem_opaque, (*s).hash_table.cast(), 0);
    cr_free(&(*s).cr);
}
unsafe fn re_string_find2(
    s: *mut REStringList,
    len: i32,
    buf: *const u32,
    h0: u32,
    add_flag: BOOL,
) -> i32 {
    let mut h = 0u32;
    if (*s).n_strings != 0 {
        h = h0 >> (32 - (*s).hash_bits);
        let mut p = *(*s).hash_table.add(h as usize);
        while !p.is_null() {
            if (*p).hash == h0 && (*p).len == len as u32 {
                let mut i = 0;
                while i < len && *string_buf(p).add(i as usize) == *buf.add(i as usize) {
                    i += 1;
                }
                if i == len {
                    return 1;
                }
            }
            p = (*p).next;
        }
    }
    if add_flag == 0 {
        return 0;
    }
    if (*s).n_strings.wrapping_add(1) > (*s).hash_size {
        let new_hash_bits = max_int((*s).hash_bits + 1, 4);
        let new_hash_size = 1u32 << new_hash_bits;
        let new_hash_table = lre_realloc(
            (*s).cr.mem_opaque,
            ptr::null_mut(),
            core::mem::size_of::<*mut REString>() * new_hash_size as usize,
        )
        .cast::<*mut REString>();
        if new_hash_table.is_null() {
            return -1;
        }
        ptr::write_bytes(new_hash_table, 0, new_hash_size as usize);
        for i in 0..(*s).hash_size {
            let mut p = *(*s).hash_table.add(i as usize);
            while !p.is_null() {
                let next = (*p).next;
                h = (*p).hash >> (32 - new_hash_bits);
                (*p).next = *new_hash_table.add(h as usize);
                *new_hash_table.add(h as usize) = p;
                p = next;
            }
        }
        lre_realloc((*s).cr.mem_opaque, (*s).hash_table.cast(), 0);
        (*s).hash_bits = new_hash_bits;
        (*s).hash_size = new_hash_size;
        (*s).hash_table = new_hash_table;
        h = h0 >> (32 - (*s).hash_bits);
    }
    let p = lre_realloc(
        (*s).cr.mem_opaque,
        ptr::null_mut(),
        core::mem::size_of::<REString>() + len as usize * 4,
    )
    .cast::<REString>();
    if p.is_null() {
        return -1;
    }
    (*p).next = *(*s).hash_table.add(h as usize);
    *(*s).hash_table.add(h as usize) = p;
    (*s).n_strings += 1;
    (*p).hash = h0;
    (*p).len = len as u32;
    memcpy_no_ub(string_buf(p).cast(), buf.cast(), len as usize * 4);
    1
}
unsafe fn re_string_find(s: *mut REStringList, len: i32, buf: *const u32, add_flag: BOOL) -> i32 {
    let h0 = re_string_hash(len, buf);
    re_string_find2(s, len, buf, h0, add_flag)
}
unsafe fn re_string_add(s: *mut REStringList, len: i32, buf: *const u32) -> i32 {
    if len == 1 {
        return cr_union_interval(&mut (*s).cr, *buf, *buf);
    }
    if re_string_find(s, len, buf, TRUE) < 0 {
        -1
    } else {
        0
    }
}
unsafe fn re_string_list_op(a: *mut REStringList, b: *mut REStringList, op: i32) -> i32 {
    if cr_op1(&mut (*a).cr, (*b).cr.points, (*b).cr.len, op) != 0 {
        return -1;
    }
    match op {
        CR_OP_UNION => {
            if (*b).n_strings != 0 {
                for i in 0..(*b).hash_size {
                    let mut p = *(*b).hash_table.add(i as usize);
                    while !p.is_null() {
                        if re_string_find2(a, (*p).len as i32, string_buf(p), (*p).hash, TRUE) < 0 {
                            return -1;
                        }
                        p = (*p).next;
                    }
                }
            }
        }
        CR_OP_INTER | CR_OP_SUB => {
            for i in 0..(*a).hash_size {
                let mut pp = (*a).hash_table.add(i as usize);
                loop {
                    let p = *pp;
                    if p.is_null() {
                        break;
                    }
                    let mut ret =
                        re_string_find2(b, (*p).len as i32, string_buf(p), (*p).hash, FALSE);
                    if op == CR_OP_SUB {
                        ret = (ret == 0) as i32;
                    }
                    if ret == 0 {
                        *pp = (*p).next;
                        (*a).n_strings -= 1;
                        lre_realloc((*a).cr.mem_opaque, p.cast(), 0);
                    } else {
                        pp = ptr::addr_of_mut!((*p).next);
                    }
                }
            }
        }
        _ => std::process::abort(),
    }
    0
}
unsafe fn re_string_list_canonicalize(
    s1: *mut REParseState,
    s: *mut REStringList,
    is_unicode: BOOL,
) -> i32 {
    if cr_regexp_canonicalize(&mut (*s).cr, is_unicode) != 0 {
        return -1;
    }
    if (*s).n_strings != 0 {
        let mut a: REStringList = core::mem::zeroed();
        re_string_list_init(s1, &mut a);
        a.n_strings = (*s).n_strings;
        a.hash_size = (*s).hash_size;
        a.hash_bits = (*s).hash_bits;
        a.hash_table = (*s).hash_table;
        (*s).n_strings = 0;
        (*s).hash_size = 0;
        (*s).hash_bits = 0;
        (*s).hash_table = ptr::null_mut();
        for i in 0..a.hash_size {
            let mut p = *a.hash_table.add(i as usize);
            while !p.is_null() {
                for j in 0..(*p).len {
                    let b = string_buf(p).add(j as usize);
                    *b = lre_canonicalize(*b, is_unicode) as u32;
                }
                if re_string_add(s, (*p).len as i32, string_buf(p)) != 0 {
                    re_string_list_free(&mut a);
                    return -1;
                }
                p = (*p).next;
            }
        }
        re_string_list_free(&mut a);
    }
    0
}
static char_range_d: [u16; 3] = [1, 0x30, 0x3a];
static char_range_s: [u16; 21] = [
    10, 9, 14, 0x20, 0x21, 0xa0, 0xa1, 0x1680, 0x1681, 0x2000, 0x200b, 0x2028, 0x202a, 0x202f,
    0x2030, 0x205f, 0x2060, 0x3000, 0x3001, 0xfeff, 0xff00,
];
static char_range_w: [u16; 9] = [4, 0x30, 0x3a, 0x41, 0x5b, 0x5f, 0x60, 0x61, 0x7b];
const CLASS_RANGE_BASE: i32 = 0x40000000;
const CHAR_RANGE_d: u32 = 0;
const CHAR_RANGE_D: u32 = 1;
const CHAR_RANGE_s: u32 = 2;
const CHAR_RANGE_S: u32 = 3;
const CHAR_RANGE_w: u32 = 4;
const CHAR_RANGE_W: u32 = 5;
const char_range_table: [*const u16; 3] = [
    char_range_d.as_ptr(),
    char_range_s.as_ptr(),
    char_range_w.as_ptr(),
];
unsafe fn cr_init_char_range(s: *mut REParseState, cr: *mut REStringList, c: u32) -> i32 {
    let invert = c & 1;
    let c_pt = char_range_table[(c >> 1) as usize];
    let len = *c_pt as i32;
    re_string_list_init(s, cr);
    for i in 0..len * 2 {
        if cr_add_point(&mut (*cr).cr, *c_pt.add(1 + i as usize) as u32) != 0 {
            re_string_list_free(cr);
            return -1;
        }
    }
    if invert != 0 && cr_invert(&mut (*cr).cr) != 0 {
        re_string_list_free(cr);
        return -1;
    }
    0
}
unsafe fn re_emit_op(s: *mut REParseState, op: u8) {
    dbuf_putc(&mut (*s).byte_code, op);
}
unsafe fn re_emit_op_u32(s: *mut REParseState, op: u8, val: u32) -> i32 {
    dbuf_putc(&mut (*s).byte_code, op);
    let pos = (*s).byte_code.size as i32;
    dbuf_put_u32(&mut (*s).byte_code, val);
    pos
}
unsafe fn re_emit_goto(s: *mut REParseState, op: u8, val: u32) -> i32 {
    dbuf_putc(&mut (*s).byte_code, op);
    let pos = (*s).byte_code.size as i32;
    dbuf_put_u32(&mut (*s).byte_code, val.wrapping_sub((pos + 4) as u32));
    pos
}
unsafe fn re_emit_goto_u8(s: *mut REParseState, op: u8, arg: u32, val: u32) -> i32 {
    dbuf_putc(&mut (*s).byte_code, op);
    dbuf_putc(&mut (*s).byte_code, arg as u8);
    let pos = (*s).byte_code.size as i32;
    dbuf_put_u32(&mut (*s).byte_code, val.wrapping_sub((pos + 4) as u32));
    pos
}
unsafe fn re_emit_goto_u8_u32(s: *mut REParseState, op: u8, arg0: u32, arg1: u32, val: u32) -> i32 {
    dbuf_putc(&mut (*s).byte_code, op);
    dbuf_putc(&mut (*s).byte_code, arg0 as u8);
    dbuf_put_u32(&mut (*s).byte_code, arg1);
    let pos = (*s).byte_code.size as i32;
    dbuf_put_u32(&mut (*s).byte_code, val.wrapping_sub((pos + 4) as u32));
    pos
}
unsafe fn re_emit_op_u8(s: *mut REParseState, op: u8, val: u32) {
    dbuf_putc(&mut (*s).byte_code, op);
    dbuf_putc(&mut (*s).byte_code, val as u8);
}
unsafe fn re_emit_op_u16(s: *mut REParseState, op: u8, val: u32) {
    dbuf_putc(&mut (*s).byte_code, op);
    dbuf_put_u16(&mut (*s).byte_code, val as u16);
}
unsafe fn re_parse_error(s: *mut REParseState, text: &str) -> i32 {
    let n = text.len().min(TMP_BUF_SIZE - 1);
    ptr::copy_nonoverlapping(text.as_ptr(), ptr::addr_of_mut!((*s).u.error_msg).cast(), n);
    (*s).u.error_msg[n] = 0;
    -1
}
unsafe fn re_parse_out_of_memory(s: *mut REParseState) -> i32 {
    re_parse_error(s, "out of memory")
}
unsafe fn parse_digits(pp: *mut *const u8, allow_overflow: BOOL) -> i32 {
    let mut p = *pp;
    let mut v = 0u64;
    loop {
        let c = *p;
        if c < b'0' || c > b'9' {
            break;
        }
        v = v * 10 + c as u64 - b'0' as u64;
        if v >= i32::MAX as u64 {
            if allow_overflow != 0 {
                v = i32::MAX as u64;
            } else {
                return -1;
            }
        }
        p = p.add(1);
    }
    *pp = p;
    v as i32
}
unsafe fn re_parse_expect(s: *mut REParseState, pp: *mut *const u8, c: u8) -> i32 {
    let p = *pp;
    if *p != c {
        return re_parse_error(s, &format!("expecting '{}'", c as char));
    }
    *pp = p.add(1);
    0
}
unsafe fn re_emit_range(s: *mut REParseState, cr: *const CharRange) -> i32 {
    let len = (*cr).len as u32 / 2;
    if len >= 65535 {
        return re_parse_error(s, "too many ranges");
    }
    if len == 0 {
        re_emit_op_u32(s, REOP_char32, u32::MAX);
    } else {
        let mut high = *(*cr).points.add((*cr).len as usize - 1);
        if high == u32::MAX {
            high = *(*cr).points.add((*cr).len as usize - 2);
        }
        if high <= 0xffff {
            re_emit_op_u16(
                s,
                if (*s).ignore_case != 0 {
                    REOP_range_i
                } else {
                    REOP_range
                },
                len,
            );
            for i in (0..(*cr).len as usize).step_by(2) {
                dbuf_put_u16(&mut (*s).byte_code, *(*cr).points.add(i) as u16);
                high = (*(*cr).points.add(i + 1)).wrapping_sub(1);
                if high == u32::MAX - 1 {
                    high = 0xffff;
                }
                dbuf_put_u16(&mut (*s).byte_code, high as u16);
            }
        } else {
            re_emit_op_u16(
                s,
                if (*s).ignore_case != 0 {
                    REOP_range32_i
                } else {
                    REOP_range32
                },
                len,
            );
            for i in (0..(*cr).len as usize).step_by(2) {
                dbuf_put_u32(&mut (*s).byte_code, *(*cr).points.add(i));
                dbuf_put_u32(
                    &mut (*s).byte_code,
                    (*(*cr).points.add(i + 1)).wrapping_sub(1),
                );
            }
        }
    }
    0
}
unsafe fn re_string_cmp_len(a: *const c_void, b: *const c_void, _arg: *mut c_void) -> i32 {
    let p1 = *a.cast::<*mut REString>();
    let p2 = *b.cast::<*mut REString>();
    ((*p1).len < (*p2).len) as i32 - ((*p1).len > (*p2).len) as i32
}
unsafe fn re_emit_char(s: *mut REParseState, c: i32) {
    if c <= 0xffff {
        re_emit_op_u16(
            s,
            if (*s).ignore_case != 0 {
                REOP_char_i
            } else {
                REOP_char
            },
            c as u32,
        );
    } else {
        re_emit_op_u32(
            s,
            if (*s).ignore_case != 0 {
                REOP_char32_i
            } else {
                REOP_char32
            },
            c as u32,
        );
    }
}
unsafe fn re_emit_string_list(s: *mut REParseState, sl: *const REStringList) -> i32 {
    if (*sl).n_strings == 0 {
        return re_emit_range(s, &(*sl).cr);
    }
    let tab = lre_realloc(
        (*s).opaque,
        ptr::null_mut(),
        core::mem::size_of::<*mut REString>() * (*sl).n_strings as usize,
    )
    .cast::<*mut REString>();
    if tab.is_null() {
        return re_parse_out_of_memory(s);
    }
    let (mut has_empty_string, mut n) = (false, 0usize);
    for i in 0..(*sl).hash_size {
        let mut p = *(*sl).hash_table.add(i as usize);
        while !p.is_null() {
            if (*p).len == 0 {
                has_empty_string = true;
            } else {
                *tab.add(n) = p;
                n += 1;
            }
            p = (*p).next;
        }
    }
    assert!(n <= (*sl).n_strings as usize);
    rqsort(
        tab.cast(),
        n,
        core::mem::size_of::<*mut REString>(),
        re_string_cmp_len,
        ptr::null_mut(),
    );
    let mut last_match_pos = -1i32;
    for i in 0..n {
        let p = *tab.add(i);
        let is_last = !has_empty_string && (*sl).cr.len == 0 && i == n - 1;
        let split_pos = if !is_last {
            re_emit_op_u32(s, REOP_split_next_first, 0)
        } else {
            0
        };
        for j in 0..(*p).len {
            re_emit_char(s, *string_buf(p).add(j as usize) as i32);
        }
        if !is_last {
            last_match_pos = re_emit_op_u32(s, REOP_goto, last_match_pos as u32);
            // The C version writes through a failed bytecode allocation here (UB).
            if dbuf_error(&(*s).byte_code) != 0 {
                lre_realloc((*s).opaque, tab.cast(), 0);
                return re_parse_out_of_memory(s);
            }
            put_u32(
                (*s).byte_code.buf.add(split_pos as usize),
                ((*s).byte_code.size as u32).wrapping_sub((split_pos + 4) as u32),
            );
        }
    }
    if (*sl).cr.len != 0 {
        let is_last = !has_empty_string;
        let split_pos = if !is_last {
            re_emit_op_u32(s, REOP_split_next_first, 0)
        } else {
            0
        };
        if re_emit_range(s, &(*sl).cr) != 0 {
            lre_realloc((*s).opaque, tab.cast(), 0);
            return -1;
        }
        if !is_last {
            if dbuf_error(&(*s).byte_code) != 0 {
                lre_realloc((*s).opaque, tab.cast(), 0);
                return re_parse_out_of_memory(s);
            }
            put_u32(
                (*s).byte_code.buf.add(split_pos as usize),
                ((*s).byte_code.size as u32).wrapping_sub((split_pos + 4) as u32),
            );
        }
    }
    if dbuf_error(&(*s).byte_code) != 0 {
        lre_realloc((*s).opaque, tab.cast(), 0);
        return re_parse_out_of_memory(s);
    }
    while last_match_pos != -1 {
        let next_pos = get_u32((*s).byte_code.buf.add(last_match_pos as usize)) as i32;
        put_u32(
            (*s).byte_code.buf.add(last_match_pos as usize),
            ((*s).byte_code.size as u32).wrapping_sub((last_match_pos + 4) as u32),
        );
        last_match_pos = next_pos;
    }
    lre_realloc((*s).opaque, tab.cast(), 0);
    0
}

fn is_unicode_char(c: u8) -> bool {
    c.is_ascii_alphanumeric() || c == b'_'
}
unsafe fn seq_prop_cb(opaque: *mut c_void, seq: *const u32, seq_len: i32) {
    re_string_add(opaque.cast(), seq_len, seq);
}
unsafe fn parse_unicode_property(
    s: *mut REParseState,
    cr: *mut REStringList,
    pp: *mut *const u8,
    is_inv: BOOL,
    allow_sequence_prop: BOOL,
) -> i32 {
    let mut p = *pp;
    let mut name = [0u8; 64];
    let mut value = [0u8; 64];
    if *p != b'{' {
        return re_parse_error(s, "expecting '{' after \\p");
    }
    p = p.add(1);
    let mut n = 0;
    while is_unicode_char(*p) {
        if n >= 63 {
            return re_parse_error(s, "unknown unicode property name");
        }
        name[n] = *p;
        n += 1;
        p = p.add(1);
    }
    let mut v = 0;
    if *p == b'=' {
        p = p.add(1);
        while is_unicode_char(*p) {
            if v >= 63 {
                return re_parse_error(s, "unknown unicode property value");
            }
            value[v] = *p;
            v += 1;
            p = p.add(1);
        }
    }
    if *p != b'}' {
        return re_parse_error(s, "expecting '}'");
    }
    p = p.add(1);
    let nm = &name[..n];
    let ret;
    if matches!(nm, b"Script" | b"sc" | b"Script_Extensions" | b"scx") {
        re_string_list_init(s, cr);
        ret = unicode_script(
            &mut (*cr).cr,
            value.as_ptr().cast(),
            matches!(nm, b"Script_Extensions" | b"scx") as i32,
        );
        if ret != 0 {
            re_string_list_free(cr);
            return if ret == -2 {
                re_parse_error(s, "unknown unicode script")
            } else {
                re_parse_out_of_memory(s)
            };
        }
    } else if matches!(nm, b"General_Category" | b"gc") {
        re_string_list_init(s, cr);
        ret = unicode_general_category(&mut (*cr).cr, value.as_ptr().cast());
        if ret != 0 {
            re_string_list_free(cr);
            return if ret == -2 {
                re_parse_error(s, "unknown unicode general category")
            } else {
                re_parse_out_of_memory(s)
            };
        }
    } else if v == 0 {
        re_string_list_init(s, cr);
        let mut r = unicode_general_category(&mut (*cr).cr, name.as_ptr().cast());
        if r == -1 {
            re_string_list_free(cr);
            return re_parse_out_of_memory(s);
        }
        if r < 0 {
            r = unicode_prop(&mut (*cr).cr, name.as_ptr().cast());
            if r == -1 {
                re_string_list_free(cr);
                return re_parse_out_of_memory(s);
            }
        }
        if r < 0 && is_inv == 0 && allow_sequence_prop != 0 {
            let mut cr_tmp = CharRange::default();
            cr_init(&mut cr_tmp, (*s).opaque, Some(lre_realloc));
            r = unicode_sequence_prop(name.as_ptr().cast(), seq_prop_cb, cr.cast(), &mut cr_tmp);
            cr_free(&cr_tmp);
            if r == -1 {
                re_string_list_free(cr);
                return re_parse_out_of_memory(s);
            }
        }
        // Upstream does not free the initialized list on this error branch.
        if r < 0 {
            return re_parse_error(s, "unknown unicode property name");
        }
    } else {
        return re_parse_error(s, "unknown unicode property name");
    }
    if (*s).ignore_case != 0
        && (*s).unicode_sets != 0
        && re_string_list_canonicalize(s, cr, (*s).is_unicode) != 0
    {
        re_string_list_free(cr);
        return re_parse_out_of_memory(s);
    }
    if is_inv != 0 && cr_invert(&mut (*cr).cr) != 0 {
        re_string_list_free(cr);
        return re_parse_out_of_memory(s);
    }
    if (*s).ignore_case != 0
        && (*s).unicode_sets == 0
        && re_string_list_canonicalize(s, cr, (*s).is_unicode) != 0
    {
        re_string_list_free(cr);
        return re_parse_out_of_memory(s);
    }
    *pp = p;
    0
}
unsafe fn parse_class_string_disjunction(
    s: *mut REParseState,
    cr: *mut REStringList,
    pp: *mut *const u8,
) -> i32 {
    let mut p = *pp;
    if *p != b'{' {
        return re_parse_error(s, "expecting '{' after \\q");
    }
    let mut str: DynBuf = core::mem::zeroed();
    dbuf_init2(&mut str, (*s).opaque, Some(lre_realloc));
    re_string_list_init(s, cr);
    p = p.add(1);
    let ret = (|| {
        loop {
            str.size = 0;
            while *p != b'}' && *p != b'|' {
                let c = get_class_atom(s, ptr::null_mut(), &mut p, FALSE);
                if c < 0 {
                    return -1;
                }
                if dbuf_put_u32(&mut str, c as u32) != 0 {
                    return re_parse_out_of_memory(s);
                }
            }
            if re_string_add(cr, (str.size / 4) as i32, str.buf.cast()) != 0 {
                return re_parse_out_of_memory(s);
            }
            if *p == b'}' {
                break;
            }
            p = p.add(1);
        }
        if (*s).ignore_case != 0 && re_string_list_canonicalize(s, cr, TRUE) != 0 {
            return -1;
        }
        p = p.add(1);
        *pp = p;
        0
    })();
    dbuf_free(&mut str);
    if ret < 0 {
        re_string_list_free(cr);
    }
    ret
}
unsafe fn get_class_atom(
    s: *mut REParseState,
    cr: *mut REStringList,
    pp: *mut *const u8,
    inclass: BOOL,
) -> i32 {
    let mut p = *pp;
    let mut c = *p as u32;
    let mut normal = false;
    if c == b'\\' as u32 {
        p = p.add(1);
        if p >= (*s).buf_end {
            return re_parse_error(s, "unexpected end");
        }
        c = *p as u32;
        p = p.add(1);
        let mut default_escape = false;
        match c as u8 {
            b'd' | b'D' | b's' | b'S' | b'w' | b'W' => {
                if cr.is_null() {
                    default_escape = true;
                } else {
                    let k = match c as u8 {
                        b'd' => CHAR_RANGE_d,
                        b'D' => CHAR_RANGE_D,
                        b's' => CHAR_RANGE_s,
                        b'S' => CHAR_RANGE_S,
                        b'w' => CHAR_RANGE_w,
                        _ => CHAR_RANGE_W,
                    };
                    if cr_init_char_range(s, cr, k) != 0 {
                        return -1;
                    }
                    c = k + CLASS_RANGE_BASE as u32;
                }
            }
            b'c' => {
                c = *p as u32;
                if (c as u8).is_ascii_alphabetic()
                    || (((c as u8).is_ascii_digit() || c == b'_' as u32)
                        && inclass != 0
                        && (*s).is_unicode == 0)
                {
                    c &= 0x1f;
                    p = p.add(1);
                } else if (*s).is_unicode != 0 {
                    return re_parse_error(s, "invalid escape sequence in regular expression");
                } else {
                    p = p.sub(1);
                    c = b'\\' as u32;
                }
            }
            b'-' => {
                if inclass == 0 && (*s).is_unicode != 0 {
                    return re_parse_error(s, "invalid escape sequence in regular expression");
                }
            }
            b'^' | b'$' | b'\\' | b'.' | b'*' | b'+' | b'?' | b'(' | b')' | b'[' | b']' | b'{'
            | b'}' | b'|' | b'/' => {}
            b'p' | b'P' => {
                if (*s).is_unicode != 0 && !cr.is_null() {
                    if parse_unicode_property(
                        s,
                        cr,
                        &mut p,
                        (c == b'P' as u32) as i32,
                        (*s).unicode_sets,
                    ) != 0
                    {
                        return -1;
                    }
                    c = CLASS_RANGE_BASE as u32;
                } else {
                    default_escape = true;
                }
            }
            b'q' => {
                if (*s).unicode_sets != 0 && !cr.is_null() && inclass != 0 {
                    if parse_class_string_disjunction(s, cr, &mut p) != 0 {
                        return -1;
                    }
                    c = CLASS_RANGE_BASE as u32;
                } else {
                    default_escape = true;
                }
            }
            _ => default_escape = true,
        }
        if default_escape {
            p = p.sub(1);
            let r = lre_parse_escape(&mut p, (*s).is_unicode * 2);
            if r >= 0 {
                c = r as u32;
            } else if (*s).is_unicode != 0 {
                return re_parse_error(s, "invalid escape sequence in regular expression");
            } else {
                normal = true;
            }
        }
    } else {
        if c == 0 && p >= (*s).buf_end {
            return re_parse_error(s, "unexpected end");
        }
        if (*s).unicode_sets != 0 {
            if b"&!#$%*+,. :;<=>?@^`~".contains(&(c as u8))
                && c != b' ' as u32
                && *p.add(1) == c as u8
            {
                return re_parse_error(s, "invalid class set operation in regular expression");
            }
            if b"()[]{}/-|".contains(&(c as u8)) {
                return re_parse_error(s, "invalid character in class in regular expression");
            }
        }
        normal = true;
    }
    if normal {
        if c >= 128 {
            c = unicode_from_utf8(p, UTF8_CHAR_LEN_MAX as i32, &mut p) as u32;
            if c > 0xffff && (*s).is_unicode == 0 {
                return re_parse_error(s, "malformed unicode char");
            }
        } else {
            p = p.add(1);
        }
    }
    *pp = p;
    c as i32
}
unsafe fn re_parse_class_set_operand(
    s: *mut REParseState,
    cr: *mut REStringList,
    pp: *mut *const u8,
) -> i32 {
    if **pp == b'[' {
        return re_parse_nested_class(s, cr, pp);
    }
    let mut c = get_class_atom(s, cr, pp, TRUE);
    if c < 0 {
        return -1;
    }
    if c < CLASS_RANGE_BASE {
        re_string_list_init(s, cr);
        if (*s).ignore_case != 0 {
            c = lre_canonicalize(c as u32, (*s).is_unicode);
        }
        if cr_union_interval(&mut (*cr).cr, c as u32, c as u32) != 0 {
            re_string_list_free(cr);
            return -1;
        }
    }
    0
}
unsafe fn re_parse_nested_class(
    s: *mut REParseState,
    cr: *mut REStringList,
    pp: *mut *const u8,
) -> i32 {
    if lre_check_stack_overflow((*s).opaque, 0) != 0 {
        return re_parse_error(s, "stack overflow");
    }
    re_string_list_init(s, cr);
    let mut p = (*pp).add(1);
    let invert = *p == b'^';
    if invert {
        p = p.add(1);
    }
    let mut is_first = true;
    let mut cr1: REStringList = core::mem::zeroed();
    let ret = (|| {
        while *p != b']' {
            let mut c1;
            let nested = *p == b'[' && (*s).unicode_sets != 0;
            if nested {
                if re_parse_nested_class(s, &mut cr1, &mut p) != 0 {
                    return -1;
                }
                c1 = CLASS_RANGE_BASE as u32;
            } else {
                let c = get_class_atom(s, &mut cr1, &mut p, TRUE);
                if c < 0 {
                    return -1;
                }
                c1 = c as u32;
            }
            let mut is_range = false;
            if !nested {
                if *p == b'-'
                    && *p.add(1) != b']'
                    && !(*p.add(1) == b'-' && (*s).unicode_sets != 0 && is_first)
                {
                    if c1 >= CLASS_RANGE_BASE as u32 {
                        if (*s).is_unicode != 0 {
                            re_string_list_free(&mut cr1);
                            return re_parse_error(s, "invalid class range");
                        }
                    } else {
                        let mut p0 = p.add(1);
                        let c2 = get_class_atom(s, &mut cr1, &mut p0, TRUE);
                        if c2 < 0 {
                            return -1;
                        }
                        if c2 >= CLASS_RANGE_BASE {
                            re_string_list_free(&mut cr1);
                            if (*s).is_unicode != 0 {
                                return re_parse_error(s, "invalid class range");
                            }
                        } else {
                            p = p0;
                            if (c2 as u32) < c1 {
                                return re_parse_error(s, "invalid class range");
                            }
                            is_range = true;
                            if (*s).ignore_case != 0 {
                                let mut cr2 = CharRange::default();
                                cr_init(&mut cr2, (*s).opaque, Some(lre_realloc));
                                let r = cr_add_interval(&mut cr2, c1, c2 as u32 + 1) != 0
                                    || cr_regexp_canonicalize(&mut cr2, (*s).is_unicode) != 0
                                    || cr_op1(&mut (*cr).cr, cr2.points, cr2.len, CR_OP_UNION) != 0;
                                cr_free(&cr2);
                                if r {
                                    return re_parse_out_of_memory(s);
                                }
                            } else if cr_union_interval(&mut (*cr).cr, c1, c2 as u32) != 0 {
                                return re_parse_out_of_memory(s);
                            }
                            is_first = false;
                        }
                    }
                }
            }
            if !is_range {
                if c1 >= CLASS_RANGE_BASE as u32 {
                    let r = re_string_list_op(cr, &mut cr1, CR_OP_UNION);
                    re_string_list_free(&mut cr1);
                    if r != 0 {
                        return re_parse_out_of_memory(s);
                    }
                } else {
                    if (*s).ignore_case != 0 {
                        c1 = lre_canonicalize(c1, (*s).is_unicode) as u32;
                    }
                    if cr_union_interval(&mut (*cr).cr, c1, c1) != 0 {
                        return re_parse_out_of_memory(s);
                    }
                }
            }
            if (*s).unicode_sets != 0 && is_first {
                let inter = *p == b'&' && *p.add(1) == b'&' && *p.add(2) != b'&';
                let sub = *p == b'-' && *p.add(1) == b'-';
                if inter || sub {
                    while *p != b']' {
                        if (inter && *p == b'&' && *p.add(1) == b'&' && *p.add(2) != b'&')
                            || (sub && *p == b'-' && *p.add(1) == b'-')
                        {
                            p = p.add(2);
                        } else {
                            return re_parse_error(s, "invalid operation in regular expression");
                        }
                        if re_parse_class_set_operand(s, &mut cr1, &mut p) != 0 {
                            return -1;
                        }
                        let r = re_string_list_op(
                            cr,
                            &mut cr1,
                            if inter { CR_OP_INTER } else { CR_OP_SUB },
                        );
                        re_string_list_free(&mut cr1);
                        if r != 0 {
                            return re_parse_out_of_memory(s);
                        }
                    }
                }
            }
            is_first = false;
        }
        p = p.add(1);
        *pp = p;
        if invert {
            if (*cr).n_strings != 0 {
                return re_parse_error(
                    s,
                    "negated character class with strings in regular expression debugger eval code",
                );
            }
            if cr_invert(&mut (*cr).cr) != 0 {
                return re_parse_out_of_memory(s);
            }
        }
        0
    })();
    if ret < 0 {
        re_string_list_free(cr);
    }
    ret
}
unsafe fn re_parse_char_class(s: *mut REParseState, pp: *mut *const u8) -> i32 {
    let mut cr: REStringList = core::mem::zeroed();
    if re_parse_nested_class(s, &mut cr, pp) != 0 {
        return -1;
    }
    let r = re_emit_string_list(s, &cr);
    re_string_list_free(&mut cr);
    r
}

unsafe fn re_need_check_adv_and_capture_init(
    pneed_capture_init: *mut BOOL,
    bc_buf: *const u8,
    bc_buf_len: i32,
) -> BOOL {
    let mut pos = 0;
    let mut adv = TRUE;
    let mut cap = FALSE;
    while pos < bc_buf_len {
        let op = *bc_buf.add(pos as usize);
        let mut len = reopcode_info[op as usize] as i32;
        match op {
            REOP_range | REOP_range_i => {
                len += get_u16(bc_buf.add(pos as usize + 1)) as i32 * 4;
                adv = FALSE;
            }
            REOP_range32 | REOP_range32_i => {
                len += get_u16(bc_buf.add(pos as usize + 1)) as i32 * 8;
                adv = FALSE;
            }
            REOP_char | REOP_char_i | REOP_char32 | REOP_char32_i | REOP_dot | REOP_any
            | REOP_space | REOP_not_space => adv = FALSE,
            REOP_line_start
            | REOP_line_start_m
            | REOP_line_end
            | REOP_line_end_m
            | REOP_set_i32
            | REOP_set_char_pos
            | REOP_word_boundary
            | REOP_word_boundary_i
            | REOP_not_word_boundary
            | REOP_not_word_boundary_i
            | REOP_prev
            | REOP_save_start
            | REOP_save_end
            | REOP_save_reset => {}
            REOP_back_reference
            | REOP_back_reference_i
            | REOP_backward_back_reference
            | REOP_backward_back_reference_i => {
                len += *bc_buf.add(pos as usize + 1) as i32;
                cap = TRUE;
            }
            _ => {
                cap = TRUE;
                break;
            }
        }
        pos += len;
    }
    *pneed_capture_init = cap;
    adv
}
unsafe fn re_parse_group_name(buf: *mut c_char, buf_size: i32, pp: *mut *const u8) -> i32 {
    let mut p = *pp;
    let mut q = buf.cast::<u8>();
    loop {
        let mut c = *p as u32;
        if c == b'\\' as u32 {
            p = p.add(1);
            if *p != b'u' {
                return -1;
            }
            c = lre_parse_escape(&mut p, 2) as u32;
        } else if c == b'>' as u32 {
            break;
        } else if c >= 128 {
            c = unicode_from_utf8(p, UTF8_CHAR_LEN_MAX as i32, &mut p) as u32;
            if is_hi_surrogate(c) != 0 {
                let mut p1 = ptr::null();
                let d = unicode_from_utf8(p, UTF8_CHAR_LEN_MAX as i32, &mut p1) as u32;
                if is_lo_surrogate(d) != 0 {
                    c = from_surrogate(c, d);
                    p = p1;
                }
            }
        } else {
            p = p.add(1);
        }
        if c > 0x10ffff {
            return -1;
        }
        if q == buf.cast() {
            if lre_js_is_ident_first(c) == 0 {
                return -1;
            }
        } else if lre_js_is_ident_next(c) == 0 {
            return -1;
        }
        if q.offset_from(buf.cast()) + UTF8_CHAR_LEN_MAX as isize + 1 > buf_size as isize {
            return -1;
        }
        if c < 128 {
            *q = c as u8;
            q = q.add(1);
        } else {
            q = q.add(unicode_to_utf8(q, c) as usize);
        }
    }
    if q == buf.cast() {
        return -1;
    }
    *q = 0;
    *pp = p.add(1);
    0
}
unsafe fn re_parse_captures(
    s: *mut REParseState,
    phas_named_captures: *mut i32,
    capture_name: *const c_char,
    emit_group_index: BOOL,
) -> i32 {
    let mut p = (*s).buf_start;
    let mut capture_index = 1;
    let mut n = 0;
    *phas_named_captures = 0;
    let mut name = [0 as c_char; TMP_BUF_SIZE];
    while p < (*s).buf_end {
        match *p {
            b'(' => {
                let cap;
                if *p.add(1) == b'?' {
                    cap = *p.add(2) == b'<' && *p.add(3) != b'=' && *p.add(3) != b'!';
                    if cap {
                        *phas_named_captures = 1;
                        if !capture_name.is_null() {
                            p = p.add(3);
                            if re_parse_group_name(name.as_mut_ptr(), TMP_BUF_SIZE as i32, &mut p)
                                == 0
                                && core::ffi::CStr::from_ptr(name.as_ptr())
                                    == core::ffi::CStr::from_ptr(capture_name)
                            {
                                if emit_group_index != 0 {
                                    dbuf_putc(&mut (*s).byte_code, capture_index as u8);
                                }
                                n += 1;
                            }
                        }
                    }
                } else {
                    cap = true;
                }
                if cap {
                    capture_index += 1;
                    if capture_index >= CAPTURE_COUNT_MAX {
                        break;
                    }
                }
            }
            b'\\' => p = p.add(1),
            b'[' => {
                p = p.add(1 + (*p == b']') as usize);
                while p < (*s).buf_end && *p != b']' {
                    if *p == b'\\' {
                        p = p.add(1);
                    }
                    p = p.add(1);
                }
            }
            _ => {}
        }
        p = p.add(1);
    }
    if capture_name.is_null() {
        capture_index
    } else {
        n
    }
}
unsafe fn re_count_captures(s: *mut REParseState) -> i32 {
    if (*s).total_capture_count < 0 {
        (*s).total_capture_count =
            re_parse_captures(s, &mut (*s).has_named_captures, ptr::null(), FALSE);
    }
    (*s).total_capture_count
}
unsafe fn re_has_named_captures(s: *mut REParseState) -> BOOL {
    if (*s).has_named_captures < 0 {
        re_count_captures(s);
    }
    (*s).has_named_captures
}
unsafe fn find_group_name(
    s: *mut REParseState,
    name: *const c_char,
    emit_group_index: BOOL,
) -> i32 {
    let mut p = (*s).group_names.buf;
    if p.is_null() {
        return 0;
    }
    let end = p.add((*s).group_names.size);
    let name = core::ffi::CStr::from_ptr(name).to_bytes();
    let (mut index, mut n) = (1, 0);
    while p < end {
        let entry = core::ffi::CStr::from_ptr(p.cast()).to_bytes();
        if entry == name {
            if emit_group_index != 0 {
                dbuf_putc(&mut (*s).byte_code, index);
            }
            n += 1;
        }
        p = p.add(entry.len() + LRE_GROUP_NAME_TRAILER_LEN as usize);
        index += 1;
    }
    n
}
unsafe fn is_duplicate_group_name(s: *mut REParseState, name: *const c_char, scope: i32) -> BOOL {
    let mut p = (*s).group_names.buf;
    if p.is_null() {
        return FALSE;
    }
    let end = p.add((*s).group_names.size);
    let name = core::ffi::CStr::from_ptr(name).to_bytes();
    while p < end {
        let entry = core::ffi::CStr::from_ptr(p.cast()).to_bytes();
        if entry == name && *p.add(entry.len() + 1) as i32 == scope {
            return TRUE;
        }
        p = p.add(entry.len() + LRE_GROUP_NAME_TRAILER_LEN as usize);
    }
    FALSE
}
unsafe fn re_parse_modifiers(s: *mut REParseState, pp: *mut *const u8) -> i32 {
    let mut p = *pp;
    let mut mask = 0;
    loop {
        let val = match *p {
            b'i' => LRE_FLAG_IGNORECASE,
            b'm' => LRE_FLAG_MULTILINE,
            b's' => LRE_FLAG_DOTALL,
            _ => break,
        };
        if mask & val != 0 {
            return re_parse_error(s, &format!("duplicate modifier: '{}'", *p as char));
        }
        mask |= val;
        p = p.add(1);
    }
    *pp = p;
    mask
}
fn update_modifier(mut val: BOOL, add_mask: i32, remove_mask: i32, mask: i32) -> BOOL {
    if add_mask & mask != 0 {
        val = TRUE;
    }
    if remove_mask & mask != 0 {
        val = FALSE;
    }
    val
}

unsafe fn re_parse_term(s: *mut REParseState, is_backward_dir: BOOL) -> i32 {
    let mut p = (*s).buf_ptr;
    let mut last_atom_start = -1i32;
    let mut last_capture_count = 0;
    let mut cr: REStringList = core::mem::zeroed();
    // This block corresponds to C's atom switch, including its parse_class_atom
    // and normal_char labels. None requests the common class-atom fallback.
    let mut literal: Option<i32> = None;
    let mut parse_class_atom = false;
    match *p {
        b'^' => {
            p = p.add(1);
            re_emit_op(
                s,
                if (*s).multi_line != 0 {
                    REOP_line_start_m
                } else {
                    REOP_line_start
                },
            );
        }
        b'$' => {
            p = p.add(1);
            re_emit_op(
                s,
                if (*s).multi_line != 0 {
                    REOP_line_end_m
                } else {
                    REOP_line_end
                },
            );
        }
        b'.' => {
            p = p.add(1);
            last_atom_start = (*s).byte_code.size as i32;
            last_capture_count = (*s).capture_count;
            if is_backward_dir != 0 {
                re_emit_op(s, REOP_prev);
            }
            re_emit_op(s, if (*s).dotall != 0 { REOP_any } else { REOP_dot });
            if is_backward_dir != 0 {
                re_emit_op(s, REOP_prev);
            }
        }
        b'{' => {
            if (*s).is_unicode != 0 {
                return re_parse_error(s, "syntax error");
            }
            if is_digit(*p.add(1) as i32) {
                let mut p1 = p.add(1);
                parse_digits(&mut p1, TRUE);
                if *p1 == b',' {
                    p1 = p1.add(1);
                    if is_digit(*p1 as i32) {
                        parse_digits(&mut p1, TRUE);
                    }
                }
                if *p1 == b'}' {
                    return re_parse_error(s, "nothing to repeat");
                }
            }
            parse_class_atom = true;
        }
        b'*' | b'+' | b'?' => return re_parse_error(s, "nothing to repeat"),
        b'(' => {
            let mut capture = false;
            if *p.add(1) == b'?' {
                match *p.add(2) {
                    b':' => {
                        p = p.add(3);
                        last_atom_start = (*s).byte_code.size as i32;
                        last_capture_count = (*s).capture_count;
                        (*s).buf_ptr = p;
                        if re_parse_disjunction(s, is_backward_dir) != 0 {
                            return -1;
                        }
                        p = (*s).buf_ptr;
                        if re_parse_expect(s, &mut p, b')') != 0 {
                            return -1;
                        }
                    }
                    b'i' | b'm' | b's' | b'-' => {
                        p = p.add(2);
                        let add_mask = re_parse_modifiers(s, &mut p);
                        if add_mask < 0 {
                            return -1;
                        }
                        let mut remove_mask = 0;
                        if *p == b'-' {
                            p = p.add(1);
                            remove_mask = re_parse_modifiers(s, &mut p);
                            if remove_mask < 0 {
                                return -1;
                            }
                        }
                        if (add_mask == 0 && remove_mask == 0) || add_mask & remove_mask != 0 {
                            return re_parse_error(s, "invalid modifiers");
                        }
                        if re_parse_expect(s, &mut p, b':') != 0 {
                            return -1;
                        }
                        let saved = ((*s).ignore_case, (*s).multi_line, (*s).dotall);
                        (*s).ignore_case =
                            update_modifier(saved.0, add_mask, remove_mask, LRE_FLAG_IGNORECASE);
                        (*s).multi_line =
                            update_modifier(saved.1, add_mask, remove_mask, LRE_FLAG_MULTILINE);
                        (*s).dotall =
                            update_modifier(saved.2, add_mask, remove_mask, LRE_FLAG_DOTALL);
                        last_atom_start = (*s).byte_code.size as i32;
                        last_capture_count = (*s).capture_count;
                        (*s).buf_ptr = p;
                        if re_parse_disjunction(s, is_backward_dir) != 0 {
                            return -1;
                        }
                        p = (*s).buf_ptr;
                        if re_parse_expect(s, &mut p, b')') != 0 {
                            return -1;
                        }
                        ((*s).ignore_case, (*s).multi_line, (*s).dotall) = saved;
                    }
                    b'=' | b'!' => {
                        let is_neg = (*p.add(2) == b'!') as u8;
                        p = p.add(3);
                        if (*s).is_unicode == 0 {
                            last_atom_start = (*s).byte_code.size as i32;
                            last_capture_count = (*s).capture_count;
                        }
                        let pos = re_emit_op_u32(s, REOP_lookahead + is_neg, 0);
                        (*s).buf_ptr = p;
                        if re_parse_disjunction(s, FALSE) != 0 {
                            return -1;
                        }
                        p = (*s).buf_ptr;
                        if re_parse_expect(s, &mut p, b')') != 0 {
                            return -1;
                        }
                        re_emit_op(s, REOP_lookahead_match + is_neg);
                        if dbuf_error(&(*s).byte_code) != 0 {
                            return -1;
                        }
                        put_u32(
                            (*s).byte_code.buf.add(pos as usize),
                            ((*s).byte_code.size as u32).wrapping_sub((pos + 4) as u32),
                        );
                    }
                    b'<' => {
                        if *p.add(3) == b'=' || *p.add(3) == b'!' {
                            let is_neg = (*p.add(3) == b'!') as u8;
                            p = p.add(4);
                            let pos = re_emit_op_u32(s, REOP_lookahead + is_neg, 0);
                            (*s).buf_ptr = p;
                            if re_parse_disjunction(s, TRUE) != 0 {
                                return -1;
                            }
                            p = (*s).buf_ptr;
                            if re_parse_expect(s, &mut p, b')') != 0 {
                                return -1;
                            }
                            re_emit_op(s, REOP_lookahead_match + is_neg);
                            if dbuf_error(&(*s).byte_code) != 0 {
                                return -1;
                            }
                            put_u32(
                                (*s).byte_code.buf.add(pos as usize),
                                ((*s).byte_code.size as u32).wrapping_sub((pos + 4) as u32),
                            );
                        } else {
                            p = p.add(3);
                            let name = ptr::addr_of_mut!((*s).u.tmp_buf).cast::<c_char>();
                            if re_parse_group_name(name, TMP_BUF_SIZE as i32, &mut p) != 0 {
                                return re_parse_error(s, "invalid group name");
                            }
                            if is_duplicate_group_name(s, name, (*s).group_name_scope as i32) != 0 {
                                return re_parse_error(s, "duplicate group name");
                            }
                            dbuf_put(
                                &mut (*s).group_names,
                                name.cast(),
                                core::ffi::CStr::from_ptr(name).to_bytes().len() + 1,
                            );
                            dbuf_putc(&mut (*s).group_names, (*s).group_name_scope);
                            (*s).has_named_captures = 1;
                            capture = true;
                        }
                    }
                    _ => return re_parse_error(s, "invalid group"),
                }
            } else {
                p = p.add(1);
                dbuf_putc(&mut (*s).group_names, 0);
                dbuf_putc(&mut (*s).group_names, 0);
                capture = true;
            }
            if capture {
                if (*s).capture_count >= CAPTURE_COUNT_MAX {
                    return re_parse_error(s, "too many captures");
                }
                last_atom_start = (*s).byte_code.size as i32;
                last_capture_count = (*s).capture_count;
                let index = (*s).capture_count;
                (*s).capture_count += 1;
                re_emit_op_u8(s, REOP_save_start + is_backward_dir as u8, index as u32);
                (*s).buf_ptr = p;
                if re_parse_disjunction(s, is_backward_dir) != 0 {
                    return -1;
                }
                p = (*s).buf_ptr;
                re_emit_op_u8(s, REOP_save_start + 1 - is_backward_dir as u8, index as u32);
                if re_parse_expect(s, &mut p, b')') != 0 {
                    return -1;
                }
            }
        }
        b'\\' => {
            match *p.add(1) {
                b'b' | b'B' => {
                    let fold = (*s).ignore_case != 0 && (*s).is_unicode != 0;
                    re_emit_op(
                        s,
                        if *p.add(1) == b'B' {
                            if fold {
                                REOP_not_word_boundary_i
                            } else {
                                REOP_not_word_boundary
                            }
                        } else if fold {
                            REOP_word_boundary_i
                        } else {
                            REOP_word_boundary
                        },
                    );
                    p = p.add(2);
                }
                b'k' => {
                    // Annex B fallback jumps to parse_class_atom, preserving p.
                    'named: {
                        if *p.add(2) != b'<' {
                            if (*s).is_unicode != 0 || re_has_named_captures(s) != 0 {
                                return re_parse_error(s, "expecting group name");
                            }
                            parse_class_atom = true;
                            break 'named;
                        }
                        let mut p1 = p.add(3);
                        let name = ptr::addr_of_mut!((*s).u.tmp_buf).cast::<c_char>();
                        if re_parse_group_name(name, TMP_BUF_SIZE as i32, &mut p1) != 0 {
                            if (*s).is_unicode != 0 || re_has_named_captures(s) != 0 {
                                return re_parse_error(s, "invalid group name");
                            }
                            parse_class_atom = true;
                            break 'named;
                        }
                        let mut n = find_group_name(s, name, FALSE);
                        let mut forward = false;
                        let mut dummy = 0;
                        if n == 0 {
                            n = re_parse_captures(s, &mut dummy, name, FALSE);
                            if n == 0 {
                                if (*s).is_unicode != 0 || re_has_named_captures(s) != 0 {
                                    return re_parse_error(s, "group name not defined");
                                }
                                parse_class_atom = true;
                                break 'named;
                            }
                            forward = true;
                        }
                        last_atom_start = (*s).byte_code.size as i32;
                        last_capture_count = (*s).capture_count;
                        re_emit_op_u8(
                            s,
                            REOP_back_reference
                                + 2 * is_backward_dir as u8
                                + (*s).ignore_case as u8,
                            n as u32,
                        );
                        if forward {
                            re_parse_captures(s, &mut dummy, name, TRUE);
                        } else {
                            find_group_name(s, name, TRUE);
                        }
                        p = p1;
                    }
                }
                b'0' => {
                    p = p.add(2);
                    let mut c = 0;
                    if (*s).is_unicode != 0 {
                        if is_digit(*p as i32) {
                            return re_parse_error(
                                s,
                                "invalid decimal escape in regular expression",
                            );
                        }
                    } else if (b'0'..=b'7').contains(&*p) {
                        c = (*p - b'0') as i32;
                        p = p.add(1);
                        if (b'0'..=b'7').contains(&*p) {
                            c = (c << 3) + (*p - b'0') as i32;
                            p = p.add(1);
                        }
                    }
                    literal = Some(c);
                }
                b'1'..=b'9' => {
                    p = p.add(1);
                    let q = p;
                    let mut c = parse_digits(&mut p, FALSE);
                    if c < 0 || (c >= (*s).capture_count && c >= re_count_captures(s)) {
                        if (*s).is_unicode != 0 {
                            return re_parse_error(
                                s,
                                "back reference out of range in regular expression",
                            );
                        }
                        p = q;
                        if *p <= b'7' {
                            c = 0;
                            if *p <= b'3' {
                                c = (*p - b'0') as i32;
                                p = p.add(1);
                            }
                            if (b'0'..=b'7').contains(&*p) {
                                c = (c << 3) + (*p - b'0') as i32;
                                p = p.add(1);
                                if (b'0'..=b'7').contains(&*p) {
                                    c = (c << 3) + (*p - b'0') as i32;
                                    p = p.add(1);
                                }
                            }
                        } else {
                            c = *p as i32;
                            p = p.add(1);
                        }
                        literal = Some(c);
                    } else {
                        last_atom_start = (*s).byte_code.size as i32;
                        last_capture_count = (*s).capture_count;
                        re_emit_op_u8(
                            s,
                            REOP_back_reference
                                + 2 * is_backward_dir as u8
                                + (*s).ignore_case as u8,
                            1,
                        );
                        dbuf_putc(&mut (*s).byte_code, c as u8);
                    }
                }
                _ => parse_class_atom = true,
            }
        }
        b'[' => {
            last_atom_start = (*s).byte_code.size as i32;
            last_capture_count = (*s).capture_count;
            if is_backward_dir != 0 {
                re_emit_op(s, REOP_prev);
            }
            if re_parse_char_class(s, &mut p) != 0 {
                return -1;
            }
            if is_backward_dir != 0 {
                re_emit_op(s, REOP_prev);
            }
        }
        b']' | b'}' => {
            if (*s).is_unicode != 0 {
                return re_parse_error(s, "syntax error");
            }
            parse_class_atom = true;
        }
        _ => parse_class_atom = true,
    }
    if parse_class_atom {
        let c = get_class_atom(s, &mut cr, &mut p, FALSE);
        if c < 0 {
            return -1;
        }
        literal = Some(c);
    }
    if let Some(mut c) = literal {
        last_atom_start = (*s).byte_code.size as i32;
        last_capture_count = (*s).capture_count;
        if is_backward_dir != 0 {
            re_emit_op(s, REOP_prev);
        }
        if c >= CLASS_RANGE_BASE {
            let r = if c == CLASS_RANGE_BASE + CHAR_RANGE_s as i32 {
                re_emit_op(s, REOP_space);
                0
            } else if c == CLASS_RANGE_BASE + CHAR_RANGE_S as i32 {
                re_emit_op(s, REOP_not_space);
                0
            } else {
                re_emit_string_list(s, &cr)
            };
            re_string_list_free(&mut cr);
            if r != 0 {
                return -1;
            }
        } else {
            if (*s).ignore_case != 0 {
                c = lre_canonicalize(c as u32, (*s).is_unicode);
            }
            re_emit_char(s, c);
        }
        if is_backward_dir != 0 {
            re_emit_op(s, REOP_prev);
        }
    }
    if last_atom_start >= 0 {
        let mut quant = None;
        match *p {
            b'*' => {
                p = p.add(1);
                quant = Some((0, i32::MAX));
            }
            b'+' => {
                p = p.add(1);
                quant = Some((1, i32::MAX));
            }
            b'?' => {
                p = p.add(1);
                quant = Some((0, 1));
            }
            b'{' => {
                let p1 = p;
                if !is_digit(*p.add(1) as i32) {
                    if (*s).is_unicode != 0 {
                        return re_parse_error(s, "invalid repetition count");
                    }
                } else {
                    p = p.add(1);
                    let min = parse_digits(&mut p, TRUE);
                    let mut max = min;
                    if *p == b',' {
                        p = p.add(1);
                        if is_digit(*p as i32) {
                            max = parse_digits(&mut p, TRUE);
                            if max < min {
                                return re_parse_error(s, "invalid repetition count");
                            }
                        } else {
                            max = i32::MAX;
                        }
                    }
                    if *p != b'}' && (*s).is_unicode == 0 {
                        p = p1;
                    } else {
                        if re_parse_expect(s, &mut p, b'}') != 0 {
                            return -1;
                        }
                        quant = Some((min, max));
                    }
                }
            }
            _ => {}
        }
        if let Some((quant_min, quant_max)) = quant {
            let mut greedy = 1u8;
            if *p == b'?' {
                p = p.add(1);
                greedy = 0;
            }
            let mut need_capture_init = FALSE;
            let mut adv = re_need_check_adv_and_capture_init(
                &mut need_capture_init,
                (*s).byte_code.buf.add(last_atom_start as usize),
                (*s).byte_code.size as i32 - last_atom_start,
            );
            if need_capture_init != 0 && last_capture_count != (*s).capture_count {
                if dbuf_insert(&mut (*s).byte_code, last_atom_start, 3) != 0 {
                    return re_parse_out_of_memory(s);
                }
                let pos = last_atom_start as usize;
                *(*s).byte_code.buf.add(pos) = REOP_save_reset;
                *(*s).byte_code.buf.add(pos + 1) = last_capture_count as u8;
                *(*s).byte_code.buf.add(pos + 2) = ((*s).capture_count - 1) as u8;
            }
            let len = (*s).byte_code.size as i32 - last_atom_start;
            if quant_min == 0 {
                if need_capture_init == 0 && last_capture_count != (*s).capture_count {
                    if dbuf_insert(&mut (*s).byte_code, last_atom_start, 3) != 0 {
                        return re_parse_out_of_memory(s);
                    }
                    *(*s).byte_code.buf.add(last_atom_start as usize) = REOP_save_reset;
                    last_atom_start += 1;
                    *(*s).byte_code.buf.add(last_atom_start as usize) = last_capture_count as u8;
                    last_atom_start += 1;
                    *(*s).byte_code.buf.add(last_atom_start as usize) =
                        ((*s).capture_count - 1) as u8;
                    last_atom_start += 1;
                }
                if quant_max == 0 {
                    (*s).byte_code.size = last_atom_start as usize;
                } else if quant_max == 1 || quant_max == i32::MAX {
                    let has_goto = (quant_max == i32::MAX) as i32;
                    if dbuf_insert(&mut (*s).byte_code, last_atom_start, 5 + adv * 2) != 0 {
                        return re_parse_out_of_memory(s);
                    }
                    let pos = last_atom_start as usize;
                    *(*s).byte_code.buf.add(pos) = REOP_split_goto_first + greedy;
                    put_u32(
                        (*s).byte_code.buf.add(pos + 1),
                        (len + 5 * has_goto + adv * 4) as u32,
                    );
                    if adv != 0 {
                        *(*s).byte_code.buf.add(pos + 5) = REOP_set_char_pos;
                        *(*s).byte_code.buf.add(pos + 6) = 0;
                        re_emit_op_u8(s, REOP_check_advance, 0);
                    }
                    if has_goto != 0 {
                        re_emit_goto(s, REOP_goto, last_atom_start as u32);
                    }
                } else {
                    if dbuf_insert(&mut (*s).byte_code, last_atom_start, 11 + adv * 2) != 0 {
                        return re_parse_out_of_memory(s);
                    }
                    let mut pos = last_atom_start as usize;
                    *(*s).byte_code.buf.add(pos) = REOP_split_goto_first + greedy;
                    pos += 1;
                    put_u32((*s).byte_code.buf.add(pos), (6 + adv * 2 + len + 10) as u32);
                    pos += 4;
                    *(*s).byte_code.buf.add(pos) = REOP_set_i32;
                    pos += 1;
                    *(*s).byte_code.buf.add(pos) = 0;
                    pos += 1;
                    put_u32((*s).byte_code.buf.add(pos), quant_max as u32);
                    pos += 4;
                    last_atom_start = pos as i32;
                    if adv != 0 {
                        *(*s).byte_code.buf.add(pos) = REOP_set_char_pos;
                        *(*s).byte_code.buf.add(pos + 1) = 0;
                    }
                    re_emit_goto_u8_u32(
                        s,
                        (if adv != 0 {
                            REOP_loop_check_adv_split_next_first
                        } else {
                            REOP_loop_split_next_first
                        }) - greedy,
                        0,
                        quant_max as u32,
                        last_atom_start as u32,
                    );
                }
            } else if quant_min == 1 && quant_max == i32::MAX && adv == 0 {
                re_emit_goto(s, REOP_split_next_first - greedy, last_atom_start as u32);
            } else {
                if quant_min == quant_max {
                    adv = FALSE;
                }
                if dbuf_insert(&mut (*s).byte_code, last_atom_start, 6 + adv * 2) != 0 {
                    return re_parse_out_of_memory(s);
                }
                let mut pos = last_atom_start as usize;
                *(*s).byte_code.buf.add(pos) = REOP_set_i32;
                pos += 1;
                *(*s).byte_code.buf.add(pos) = 0;
                pos += 1;
                put_u32((*s).byte_code.buf.add(pos), quant_max as u32);
                pos += 4;
                last_atom_start = pos as i32;
                if adv != 0 {
                    *(*s).byte_code.buf.add(pos) = REOP_set_char_pos;
                    *(*s).byte_code.buf.add(pos + 1) = 0;
                }
                if quant_min == quant_max {
                    re_emit_goto_u8(s, REOP_loop, 0, last_atom_start as u32);
                } else {
                    re_emit_goto_u8_u32(
                        s,
                        (if adv != 0 {
                            REOP_loop_check_adv_split_next_first
                        } else {
                            REOP_loop_split_next_first
                        }) - greedy,
                        0,
                        (quant_max - quant_min) as u32,
                        last_atom_start as u32,
                    );
                }
            }
        }
    }
    (*s).buf_ptr = p;
    0
}
unsafe fn re_parse_alternative(s: *mut REParseState, is_backward_dir: BOOL) -> i32 {
    let start = (*s).byte_code.size;
    loop {
        let p = (*s).buf_ptr;
        if p >= (*s).buf_end || *p == b'|' || *p == b')' {
            break;
        }
        let term_start = (*s).byte_code.size;
        let ret = re_parse_term(s, is_backward_dir);
        if ret != 0 {
            return ret;
        }
        if is_backward_dir != 0 {
            let end = (*s).byte_code.size;
            let term_size = end - term_start;
            if dbuf_claim(&mut (*s).byte_code, term_size) != 0 {
                return -1;
            }
            ptr::copy(
                (*s).byte_code.buf.add(start),
                (*s).byte_code.buf.add(start + term_size),
                end - start,
            );
            memcpy_no_ub(
                (*s).byte_code.buf.add(start).cast(),
                (*s).byte_code.buf.add(end).cast(),
                term_size,
            );
        }
    }
    0
}
unsafe fn re_parse_disjunction(s: *mut REParseState, is_backward_dir: BOOL) -> i32 {
    if lre_check_stack_overflow((*s).opaque, 0) != 0 {
        return re_parse_error(s, "stack overflow");
    }
    let start = (*s).byte_code.size as i32;
    if re_parse_alternative(s, is_backward_dir) != 0 {
        return -1;
    }
    while *(*s).buf_ptr == b'|' {
        (*s).buf_ptr = (*s).buf_ptr.add(1);
        let len = (*s).byte_code.size as i32 - start;
        if dbuf_insert(&mut (*s).byte_code, start, 5) != 0 {
            return re_parse_out_of_memory(s);
        }
        *(*s).byte_code.buf.add(start as usize) = REOP_split_next_first;
        put_u32((*s).byte_code.buf.add(start as usize + 1), (len + 5) as u32);
        let pos = re_emit_op_u32(s, REOP_goto, 0);
        (*s).group_name_scope = (*s).group_name_scope.wrapping_add(1);
        if re_parse_alternative(s, is_backward_dir) != 0 {
            return -1;
        }
        if dbuf_error(&(*s).byte_code) != 0 {
            return re_parse_out_of_memory(s);
        }
        put_u32(
            (*s).byte_code.buf.add(pos as usize),
            ((*s).byte_code.size as u32).wrapping_sub((pos + 4) as u32),
        );
    }
    0
}
unsafe fn compute_register_count(bc_buf: *mut u8, bc_buf_len: i32) -> i32 {
    let bc_buf = bc_buf.add(RE_HEADER_LEN);
    let bc_buf_len = bc_buf_len - RE_HEADER_LEN as i32;
    let (mut stack_size, mut stack_size_max, mut pos) = (0i32, 0, 0);
    while pos < bc_buf_len {
        let opcode = *bc_buf.add(pos as usize);
        assert!((opcode as usize) < REOP_COUNT);
        let mut len = reopcode_info[opcode as usize] as i32;
        assert!(pos + len <= bc_buf_len);
        match opcode {
            REOP_set_i32 | REOP_set_char_pos => {
                *bc_buf.add(pos as usize + 1) = stack_size as u8;
                stack_size += 1;
                if stack_size > stack_size_max {
                    if stack_size > REGISTER_COUNT_MAX {
                        return -1;
                    }
                    stack_size_max = stack_size;
                }
            }
            REOP_check_advance
            | REOP_loop
            | REOP_loop_split_goto_first
            | REOP_loop_split_next_first => {
                assert!(stack_size > 0);
                stack_size -= 1;
                *bc_buf.add(pos as usize + 1) = stack_size as u8;
            }
            REOP_loop_check_adv_split_goto_first | REOP_loop_check_adv_split_next_first => {
                assert!(stack_size >= 2);
                stack_size -= 2;
                *bc_buf.add(pos as usize + 1) = stack_size as u8;
            }
            REOP_range | REOP_range_i => len += get_u16(bc_buf.add(pos as usize + 1)) as i32 * 4,
            REOP_range32 | REOP_range32_i => {
                len += get_u16(bc_buf.add(pos as usize + 1)) as i32 * 8
            }
            REOP_back_reference
            | REOP_back_reference_i
            | REOP_backward_back_reference
            | REOP_backward_back_reference_i => len += *bc_buf.add(pos as usize + 1) as i32,
            _ => {}
        }
        pos += len;
    }
    stack_size_max
}
unsafe fn lre_bytecode_realloc(opaque: *mut c_void, p: *mut c_void, size: usize) -> *mut c_void {
    if size > (i32::MAX / 2) as usize {
        ptr::null_mut()
    } else {
        lre_realloc(opaque, p, size)
    }
}
/// Translate lre_compile's caller-owned allocator contract. buf must contain
/// buf_len UTF-8 bytes followed by a NUL; opaque points to an injected LREHost.
pub unsafe fn lre_compile(
    plen: *mut i32,
    error_msg: *mut c_char,
    error_msg_size: i32,
    buf: *const c_char,
    buf_len: usize,
    re_flags: i32,
    opaque: *mut c_void,
) -> *mut u8 {
    let mut state: REParseState = core::mem::zeroed();
    let s = &mut state as *mut REParseState;
    (*s).opaque = opaque;
    (*s).buf_ptr = buf.cast();
    (*s).buf_end = (*s).buf_ptr.add(buf_len);
    (*s).buf_start = (*s).buf_ptr;
    (*s).re_flags = re_flags;
    (*s).is_unicode = (re_flags & (LRE_FLAG_UNICODE | LRE_FLAG_UNICODE_SETS) != 0) as BOOL;
    (*s).ignore_case = (re_flags & LRE_FLAG_IGNORECASE != 0) as BOOL;
    (*s).multi_line = (re_flags & LRE_FLAG_MULTILINE != 0) as BOOL;
    (*s).dotall = (re_flags & LRE_FLAG_DOTALL != 0) as BOOL;
    (*s).unicode_sets = (re_flags & LRE_FLAG_UNICODE_SETS != 0) as BOOL;
    (*s).capture_count = 1;
    (*s).total_capture_count = -1;
    (*s).has_named_captures = -1;
    dbuf_init2(&mut (*s).byte_code, opaque, Some(lre_bytecode_realloc));
    dbuf_init2(&mut (*s).group_names, opaque, Some(lre_realloc));
    dbuf_put_u16(&mut (*s).byte_code, re_flags as u16);
    dbuf_putc(&mut (*s).byte_code, 0);
    dbuf_putc(&mut (*s).byte_code, 0);
    dbuf_put_u32(&mut (*s).byte_code, 0);
    if re_flags & LRE_FLAG_STICKY == 0 {
        re_emit_op_u32(s, REOP_split_goto_first, 6);
        re_emit_op(s, REOP_any);
        re_emit_op_u32(s, REOP_goto, (-11i32) as u32);
    }
    re_emit_op_u8(s, REOP_save_start, 0);
    let ret = (|| {
        if re_parse_disjunction(s, FALSE) != 0 {
            return -1;
        }
        re_emit_op_u8(s, REOP_save_end, 0);
        re_emit_op(s, REOP_match);
        if *(*s).buf_ptr != 0 {
            return re_parse_error(s, "extraneous characters at the end");
        }
        if dbuf_error(&(*s).byte_code) != 0 {
            return re_parse_out_of_memory(s);
        }
        let registers = compute_register_count((*s).byte_code.buf, (*s).byte_code.size as i32);
        if registers < 0 {
            return re_parse_error(s, "too many imbricated quantifiers");
        }
        *(*s).byte_code.buf.add(RE_HEADER_CAPTURE_COUNT) = (*s).capture_count as u8;
        *(*s).byte_code.buf.add(RE_HEADER_REGISTER_COUNT) = registers as u8;
        put_u32(
            (*s).byte_code.buf.add(RE_HEADER_BYTECODE_LEN),
            ((*s).byte_code.size - RE_HEADER_LEN) as u32,
        );
        if (*s).group_names.size > ((*s).capture_count - 1) as usize * LRE_GROUP_NAME_TRAILER_LEN {
            dbuf_put(
                &mut (*s).byte_code,
                (*s).group_names.buf,
                (*s).group_names.size,
            );
            put_u16(
                (*s).byte_code.buf.add(RE_HEADER_FLAGS),
                (lre_get_flags((*s).byte_code.buf) | LRE_FLAG_NAMED_GROUPS) as u16,
            );
        }
        0
    })();
    if ret != 0 {
        dbuf_free(&mut (*s).byte_code);
        dbuf_free(&mut (*s).group_names);
        pstrcpy(
            error_msg,
            error_msg_size,
            ptr::addr_of!((*s).u.error_msg).cast(),
        );
        *plen = 0;
        return ptr::null_mut();
    }
    dbuf_free(&mut (*s).group_names);
    *error_msg = 0;
    *plen = (*s).byte_code.size as i32;
    (*s).byte_code.buf
}
