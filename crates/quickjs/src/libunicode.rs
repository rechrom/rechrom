//! Direct translation of libunicode.c. Copyright 2017-2018 Fabrice Bellard.
//! MIT; see ../LICENSE.
pub use super::libunicode_header::*;
use super::{cutils::*, libunicode_table::*};
use core::{
    ffi::{c_char, c_void},
    ptr,
};
include!("libunicode_constants.rs");

fn lre_case_conv1(c: u32, conv_type: i32) -> u32 {
    let mut res = [0; LRE_CC_RES_LEN_MAX];
    lre_case_conv(&mut res, c, conv_type);
    res[0]
}
fn lre_case_conv_entry(res: &mut [u32], mut c: u32, conv_type: i32, idx: usize, v: u32) -> i32 {
    let is_lower = (conv_type != 0) as u32;
    let type_ = (v >> (32 - 17 - 7 - 4)) & 0xf;
    let data = ((v & 0xf) << 8) | case_conv_table2[idx] as u32;
    let code = v >> (32 - 17);
    match type_ {
        RUN_TYPE_U | RUN_TYPE_L | RUN_TYPE_UF | RUN_TYPE_LF => {
            if conv_type as u32 == (type_ & 1) || (type_ >= RUN_TYPE_UF && conv_type == 2) {
                c = c - code + (case_conv_table1[data as usize] >> (32 - 17));
            }
        }
        RUN_TYPE_UL => {
            let a = c - code;
            if (a & 1) == 1 - is_lower {
                c = (a ^ 1) + code;
            }
        }
        RUN_TYPE_LSU => {
            let a = c - code;
            if a == 1 {
                c = c.wrapping_add((2 * is_lower).wrapping_sub(1));
            } else if a == (1 - is_lower) * 2 {
                c = c.wrapping_add((2 * is_lower).wrapping_sub(1).wrapping_mul(2));
            }
        }
        RUN_TYPE_U2L_399_EXT2 => {
            if is_lower == 0 {
                res[0] = c - code + case_conv_ext[(data >> 6) as usize] as u32;
                res[1] = 0x399;
                return 2;
            }
            c = c - code + case_conv_ext[(data & 0x3f) as usize] as u32;
        }
        RUN_TYPE_UF_D20 => {
            if conv_type != 1 {
                c = data + (conv_type == 2) as u32 * 0x20;
            }
        }
        RUN_TYPE_UF_D1_EXT => {
            if conv_type != 1 {
                c = case_conv_ext[data as usize] as u32 + (conv_type == 2) as u32;
            }
        }
        RUN_TYPE_U_EXT | RUN_TYPE_LF_EXT => {
            if is_lower == type_ - RUN_TYPE_U_EXT {
                c = case_conv_ext[data as usize] as u32;
            }
        }
        RUN_TYPE_LF_EXT2 => {
            if is_lower != 0 {
                res[0] = c - code + case_conv_ext[(data >> 6) as usize] as u32;
                res[1] = case_conv_ext[(data & 0x3f) as usize] as u32;
                return 2;
            }
        }
        RUN_TYPE_UF_EXT2 => {
            if conv_type != 1 {
                res[0] = c - code + case_conv_ext[(data >> 6) as usize] as u32;
                res[1] = case_conv_ext[(data & 0x3f) as usize] as u32;
                if conv_type == 2 {
                    res[0] = lre_case_conv1(res[0], 1);
                    res[1] = lre_case_conv1(res[1], 1);
                }
                return 2;
            }
        }
        _ => {
            if conv_type != 1 {
                res[0] = case_conv_ext[(data >> 8) as usize] as u32;
                res[1] = case_conv_ext[((data >> 4) & 0xf) as usize] as u32;
                res[2] = case_conv_ext[(data & 0xf) as usize] as u32;
                if conv_type == 2 {
                    res[0] = lre_case_conv1(res[0], 1);
                    res[1] = lre_case_conv1(res[1], 1);
                    res[2] = lre_case_conv1(res[2], 1);
                }
                return 3;
            }
        }
    }
    res[0] = c;
    1
}
pub fn lre_case_conv(res: &mut [u32], mut c: u32, conv_type: i32) -> i32 {
    if c < 128 {
        if conv_type != 0 {
            if c >= b'A' as u32 && c <= b'Z' as u32 {
                c = c - b'A' as u32 + b'a' as u32;
            }
        } else if c >= b'a' as u32 && c <= b'z' as u32 {
            c = c - b'a' as u32 + b'A' as u32;
        }
    } else {
        let (mut idx_min, mut idx_max) = (0i32, case_conv_table1.len() as i32 - 1);
        while idx_min <= idx_max {
            let idx = ((idx_max + idx_min) as u32 / 2) as usize;
            let v = case_conv_table1[idx];
            let code = v >> (32 - 17);
            let len = (v >> (32 - 17 - 7)) & 0x7f;
            if c < code {
                idx_max = idx as i32 - 1;
            } else if c >= code + len {
                idx_min = idx as i32 + 1;
            } else {
                return lre_case_conv_entry(res, c, conv_type, idx, v);
            }
        }
    }
    res[0] = c;
    1
}
fn lre_case_folding_entry(mut c: u32, idx: usize, v: u32, is_unicode: i32) -> u32 {
    let mut res = [0; LRE_CC_RES_LEN_MAX];
    if is_unicode != 0 {
        let len = lre_case_conv_entry(&mut res, c, 2, idx, v);
        if len == 1 {
            c = res[0];
        } else if c == 0xfb06 {
            c = 0xfb05;
        } else if c == 0x1fd3 {
            c = 0x390;
        } else if c == 0x1fe3 {
            c = 0x3b0;
        }
    } else if c < 128 {
        if c >= b'a' as u32 && c <= b'z' as u32 {
            c = c - b'a' as u32 + b'A' as u32;
        }
    } else {
        let len = lre_case_conv_entry(&mut res, c, FALSE, idx, v);
        if len == 1 && res[0] >= 128 {
            c = res[0];
        }
    }
    c
}
pub fn lre_canonicalize(mut c: u32, is_unicode: i32) -> i32 {
    if c < 128 {
        if is_unicode != 0 {
            if c >= b'A' as u32 && c <= b'Z' as u32 {
                c = c - b'A' as u32 + b'a' as u32;
            }
        } else if c >= b'a' as u32 && c <= b'z' as u32 {
            c = c - b'a' as u32 + b'A' as u32;
        }
    } else {
        let (mut idx_min, mut idx_max) = (0i32, case_conv_table1.len() as i32 - 1);
        while idx_min <= idx_max {
            let idx = ((idx_max + idx_min) as u32 / 2) as usize;
            let v = case_conv_table1[idx];
            let code = v >> (32 - 17);
            let len = (v >> (32 - 17 - 7)) & 0x7f;
            if c < code {
                idx_max = idx as i32 - 1;
            } else if c >= code + len {
                idx_min = idx as i32 + 1;
            } else {
                return lre_case_folding_entry(c, idx, v, is_unicode) as i32;
            }
        }
    }
    c as i32
}
unsafe fn get_le24(p: *const u8) -> u32 {
    *p as u32 | ((*p.add(1) as u32) << 8) | ((*p.add(2) as u32) << 16)
}
unsafe fn get_index_pos(
    pcode: &mut u32,
    c: u32,
    index_table: *const u8,
    index_table_len: i32,
) -> i32 {
    let mut idx_min = 0;
    let v = get_le24(index_table);
    let code = v & ((1 << 21) - 1);
    if c < code {
        *pcode = 0;
        return 0;
    }
    let mut idx_max = index_table_len - 1;
    let code = get_le24(index_table.add(idx_max as usize * 3));
    if c >= code {
        return -1;
    }
    while idx_max - idx_min > 1 {
        let idx = (idx_max + idx_min) / 2;
        let v = get_le24(index_table.add(idx as usize * 3));
        let code = v & ((1 << 21) - 1);
        if c < code {
            idx_max = idx;
        } else {
            idx_min = idx;
        }
    }
    let v = get_le24(index_table.add(idx_min as usize * 3));
    *pcode = v & ((1 << 21) - 1);
    (idx_min + 1) * UNICODE_INDEX_BLOCK_LEN + (v >> 21) as i32
}
unsafe fn lre_is_in_table(
    c: u32,
    table: *const u8,
    index_table: *const u8,
    index_table_len: i32,
) -> i32 {
    let mut code = 0;
    let pos = get_index_pos(&mut code, c, index_table, index_table_len);
    if pos < 0 {
        return FALSE;
    }
    let mut p = table.add(pos as usize);
    let mut bit = 0;
    loop {
        let b = *p as u32;
        p = p.add(1);
        if b < 64 {
            code += (b >> 3) + 1;
            if c < code {
                return bit;
            }
            bit ^= 1;
            code += (b & 7) + 1;
        } else if b >= 0x80 {
            code += b - 0x80 + 1;
        } else if b < 0x60 {
            code += (((b - 0x40) << 8) | *p as u32) + 1;
            p = p.add(1);
        } else {
            code += (((b - 0x60) << 16) | ((*p as u32) << 8) | *p.add(1) as u32) + 1;
            p = p.add(2);
        }
        if c < code {
            return bit;
        }
        bit ^= 1;
    }
}
pub fn lre_is_cased(c: u32) -> i32 {
    let (mut idx_min, mut idx_max) = (0i32, case_conv_table1.len() as i32 - 1);
    while idx_min <= idx_max {
        let idx = ((idx_max + idx_min) as u32 / 2) as usize;
        let v = case_conv_table1[idx];
        let code = v >> (32 - 17);
        let len = (v >> (32 - 17 - 7)) & 0x7f;
        if c < code {
            idx_max = idx as i32 - 1;
        } else if c >= code + len {
            idx_min = idx as i32 + 1;
        } else {
            return TRUE;
        }
    }
    unsafe {
        lre_is_in_table(
            c,
            unicode_prop_Cased1_table.as_ptr(),
            unicode_prop_Cased1_index.as_ptr(),
            (unicode_prop_Cased1_index.len() / 3) as i32,
        )
    }
}
pub fn lre_is_case_ignorable(c: u32) -> i32 {
    unsafe {
        lre_is_in_table(
            c,
            unicode_prop_Case_Ignorable_table.as_ptr(),
            unicode_prop_Case_Ignorable_index.as_ptr(),
            (unicode_prop_Case_Ignorable_index.len() / 3) as i32,
        )
    }
}
pub fn lre_is_id_start(c: u32) -> i32 {
    unsafe {
        lre_is_in_table(
            c,
            unicode_prop_ID_Start_table.as_ptr(),
            unicode_prop_ID_Start_index.as_ptr(),
            (unicode_prop_ID_Start_index.len() / 3) as i32,
        )
    }
}
pub fn lre_is_id_continue(c: u32) -> i32 {
    (lre_is_id_start(c) != 0
        || unsafe {
            lre_is_in_table(
                c,
                unicode_prop_ID_Continue1_table.as_ptr(),
                unicode_prop_ID_Continue1_index.as_ptr(),
                (unicode_prop_ID_Continue1_index.len() / 3) as i32,
            )
        } != 0) as i32
}
pub fn lre_is_space_non_ascii(c: u32) -> i32 {
    let mut i = 5;
    while i < char_range_s.len() {
        let low = char_range_s[i] as u32;
        let high = char_range_s[i + 1] as u32;
        if c < low {
            return FALSE;
        }
        if c < high {
            return TRUE;
        }
        i += 2;
    }
    FALSE
}
#[allow(dead_code)]
unsafe fn cr_dump(cr: *const CharRange) {
    for i in 0..(*cr).len {
        println!("{}: 0x{:04x}", i, *(*cr).points.add(i as usize));
    }
}
unsafe fn cr_default_realloc(opaque: *mut c_void, p: *mut c_void, size: usize) -> *mut c_void {
    dbuf_default_realloc(opaque, p, size)
}
pub unsafe fn cr_init(
    cr: *mut CharRange,
    mem_opaque: *mut c_void,
    realloc_func: Option<DynBufReallocFunc>,
) {
    (*cr).len = 0;
    (*cr).size = 0;
    (*cr).points = ptr::null_mut();
    (*cr).mem_opaque = mem_opaque;
    (*cr).realloc_func = Some(realloc_func.unwrap_or(cr_default_realloc));
}
pub unsafe fn cr_free(cr: *const CharRange) {
    ((*cr).realloc_func.unwrap())((*cr).mem_opaque, (*cr).points.cast(), 0);
}
pub unsafe fn cr_realloc(cr: *mut CharRange, size: i32) -> i32 {
    if size > (*cr).size {
        let new_size = max_int(size, (*cr).size * 3 / 2);
        let new_buf = ((*cr).realloc_func.unwrap())(
            (*cr).mem_opaque,
            (*cr).points.cast(),
            new_size as usize * core::mem::size_of::<u32>(),
        )
        .cast::<u32>();
        if new_buf.is_null() {
            return -1;
        }
        (*cr).points = new_buf;
        (*cr).size = new_size;
    }
    0
}
pub unsafe fn cr_copy(cr: *mut CharRange, cr1: *const CharRange) -> i32 {
    if cr_realloc(cr, (*cr1).len) != 0 {
        return -1;
    }
    memcpy_no_ub(
        (*cr).points.cast(),
        (*cr1).points.cast(),
        4 * (*cr1).len as usize,
    );
    (*cr).len = (*cr1).len;
    0
}
unsafe fn cr_compress(cr: *mut CharRange) {
    let pt = (*cr).points;
    let len = (*cr).len as usize;
    let (mut i, mut k) = (0, 0);
    while i + 1 < len {
        if *pt.add(i) == *pt.add(i + 1) {
            i += 2;
        } else {
            let mut j = i;
            while j + 3 < len && *pt.add(j + 1) == *pt.add(j + 2) {
                j += 2;
            }
            *pt.add(k) = *pt.add(i);
            *pt.add(k + 1) = *pt.add(j + 1);
            k += 2;
            i = j + 2;
        }
    }
    (*cr).len = k as i32;
}
pub unsafe fn cr_op(
    cr: *mut CharRange,
    a_pt: *const u32,
    a_len: i32,
    b_pt: *const u32,
    b_len: i32,
    op: i32,
) -> i32 {
    let (mut a_idx, mut b_idx) = (0i32, 0i32);
    loop {
        let v;
        if a_idx < a_len && b_idx < b_len {
            let (a, b) = (*a_pt.add(a_idx as usize), *b_pt.add(b_idx as usize));
            if a < b {
                v = a;
                a_idx += 1;
            } else if a == b {
                v = a;
                a_idx += 1;
                b_idx += 1;
            } else {
                v = b;
                b_idx += 1;
            }
        } else if a_idx < a_len {
            v = *a_pt.add(a_idx as usize);
            a_idx += 1;
        } else if b_idx < b_len {
            v = *b_pt.add(b_idx as usize);
            b_idx += 1;
        } else {
            break;
        }
        let is_in = match op {
            CR_OP_UNION => (a_idx & 1) | (b_idx & 1),
            CR_OP_INTER => (a_idx & 1) & (b_idx & 1),
            CR_OP_XOR => (a_idx & 1) ^ (b_idx & 1),
            CR_OP_SUB => (a_idx & 1) & ((b_idx & 1) ^ 1),
            _ => std::process::abort(),
        };
        if is_in != ((*cr).len & 1) && cr_add_point(cr, v) != 0 {
            return -1;
        }
    }
    cr_compress(cr);
    0
}
pub unsafe fn cr_op1(cr: *mut CharRange, b_pt: *const u32, b_len: i32, op: i32) -> i32 {
    let a = *cr;
    (*cr).len = 0;
    (*cr).size = 0;
    (*cr).points = ptr::null_mut();
    let ret = cr_op(cr, a.points, a.len, b_pt, b_len, op);
    cr_free(&a);
    ret
}
pub unsafe fn cr_invert(cr: *mut CharRange) -> i32 {
    let len = (*cr).len as usize;
    if cr_realloc(cr, len as i32 + 2) != 0 {
        return -1;
    }
    if len != 0 {
        ptr::copy((*cr).points, (*cr).points.add(1), len);
    }
    *(*cr).points = 0;
    *(*cr).points.add(len + 1) = u32::MAX;
    (*cr).len = len as i32 + 2;
    cr_compress(cr);
    0
}
unsafe fn unicode_case1(cr: *mut CharRange, case_mask: i32) -> i32 {
    macro_rules! MR {
        ($name:ident) => {
            1u32 << $name
        };
    }
    let tab_run_mask = [
        MR!(RUN_TYPE_U)
            | MR!(RUN_TYPE_UF)
            | MR!(RUN_TYPE_UL)
            | MR!(RUN_TYPE_LSU)
            | MR!(RUN_TYPE_U2L_399_EXT2)
            | MR!(RUN_TYPE_UF_D20)
            | MR!(RUN_TYPE_UF_D1_EXT)
            | MR!(RUN_TYPE_U_EXT)
            | MR!(RUN_TYPE_UF_EXT2)
            | MR!(RUN_TYPE_UF_EXT3),
        MR!(RUN_TYPE_L)
            | MR!(RUN_TYPE_LF)
            | MR!(RUN_TYPE_UL)
            | MR!(RUN_TYPE_LSU)
            | MR!(RUN_TYPE_U2L_399_EXT2)
            | MR!(RUN_TYPE_LF_EXT)
            | MR!(RUN_TYPE_LF_EXT2),
        MR!(RUN_TYPE_UF)
            | MR!(RUN_TYPE_LF)
            | MR!(RUN_TYPE_UL)
            | MR!(RUN_TYPE_LSU)
            | MR!(RUN_TYPE_U2L_399_EXT2)
            | MR!(RUN_TYPE_LF_EXT)
            | MR!(RUN_TYPE_LF_EXT2)
            | MR!(RUN_TYPE_UF_D20)
            | MR!(RUN_TYPE_UF_D1_EXT)
            | MR!(RUN_TYPE_LF_EXT)
            | MR!(RUN_TYPE_UF_EXT2)
            | MR!(RUN_TYPE_UF_EXT3),
    ];
    if case_mask == 0 {
        return 0;
    }
    let mut mask = 0;
    for i in 0..3 {
        if (case_mask >> i) & 1 != 0 {
            mask |= tab_run_mask[i];
        }
    }
    for v in case_conv_table1 {
        let type_ = (v >> (32 - 17 - 7 - 4)) & 0xf;
        let mut code = v >> (32 - 17);
        let len = (v >> (32 - 17 - 7)) & 0x7f;
        if (mask >> type_) & 1 != 0 {
            let both = case_mask & CASE_U != 0 && case_mask & (CASE_L | CASE_F) != 0;
            match type_ {
                RUN_TYPE_UL if !both => {
                    code += (case_mask & CASE_U != 0) as u32;
                    for i in (0..len).step_by(2) {
                        if cr_add_interval(cr, code + i, code + i + 1) != 0 {
                            return -1;
                        }
                    }
                }
                RUN_TYPE_LSU if !both => {
                    if case_mask & CASE_U == 0 && cr_add_interval(cr, code, code + 1) != 0 {
                        return -1;
                    }
                    if cr_add_interval(cr, code + 1, code + 2) != 0 {
                        return -1;
                    }
                    if case_mask & CASE_U != 0 && cr_add_interval(cr, code + 2, code + 3) != 0 {
                        return -1;
                    }
                }
                _ => {
                    if cr_add_interval(cr, code, code + len) != 0 {
                        return -1;
                    }
                }
            }
        }
    }
    0
}
unsafe fn point_cmp(p1: *const c_void, p2: *const c_void, _arg: *mut c_void) -> i32 {
    let (v1, v2) = (*p1.cast::<u32>(), *p2.cast::<u32>());
    (v1 > v2) as i32 - (v1 < v2) as i32
}
unsafe fn cr_sort_and_remove_overlap(cr: *mut CharRange) {
    rqsort(
        (*cr).points.cast(),
        ((*cr).len / 2) as usize,
        8,
        point_cmp,
        ptr::null_mut(),
    );
    let pt = (*cr).points;
    let (mut i, mut j) = (0, 0);
    while i < (*cr).len as usize {
        let start = *pt.add(i);
        let mut end = *pt.add(i + 1);
        i += 2;
        while i < (*cr).len as usize {
            let (start1, end1) = (*pt.add(i), *pt.add(i + 1));
            if start1 > end {
                break;
            } else if end1 <= end {
                i += 2;
            } else {
                end = end1;
                i += 2;
            }
        }
        *pt.add(j) = start;
        *pt.add(j + 1) = end;
        j += 2;
    }
    (*cr).len = j as i32;
}
pub unsafe fn cr_regexp_canonicalize(cr: *mut CharRange, is_unicode: i32) -> i32 {
    let (mut cr_inter, mut cr_mask, mut cr_result, mut cr_sub) = (
        CharRange::default(),
        CharRange::default(),
        CharRange::default(),
        CharRange::default(),
    );
    for p in [&mut cr_mask, &mut cr_inter, &mut cr_result, &mut cr_sub] {
        cr_init(p, (*cr).mem_opaque, (*cr).realloc_func);
    }
    let ret = (|| {
        if unicode_case1(&mut cr_mask, if is_unicode != 0 { CASE_F } else { CASE_U }) != 0 {
            return -1;
        }
        if cr_op(
            &mut cr_inter,
            cr_mask.points,
            cr_mask.len,
            (*cr).points,
            (*cr).len,
            CR_OP_INTER,
        ) != 0
        {
            return -1;
        }
        if cr_invert(&mut cr_mask) != 0 {
            return -1;
        }
        if cr_op(
            &mut cr_sub,
            cr_mask.points,
            cr_mask.len,
            (*cr).points,
            (*cr).len,
            CR_OP_INTER,
        ) != 0
        {
            return -1;
        }
        let (mut d_start, mut d_end) = (u32::MAX, u32::MAX);
        let mut idx = 0;
        let mut v = case_conv_table1[idx];
        let mut code = v >> (32 - 17);
        let mut len = (v >> (32 - 17 - 7)) & 0x7f;
        for i in (0..cr_inter.len as usize).step_by(2) {
            let (start, end) = (*cr_inter.points.add(i), *cr_inter.points.add(i + 1));
            for c in start..end {
                loop {
                    if c >= code && c < code + len {
                        break;
                    }
                    idx += 1;
                    assert!(idx < case_conv_table1.len());
                    v = case_conv_table1[idx];
                    code = v >> (32 - 17);
                    len = (v >> (32 - 17 - 7)) & 0x7f;
                }
                let d = lre_case_folding_entry(c, idx, v, is_unicode);
                if d_start == u32::MAX {
                    d_start = d;
                    d_end = d + 1;
                } else if d_end == d {
                    d_end += 1;
                } else {
                    cr_add_interval(&mut cr_result, d_start, d_end);
                    d_start = d;
                    d_end = d + 1;
                }
            }
        }
        if d_start != u32::MAX && cr_add_interval(&mut cr_result, d_start, d_end) != 0 {
            return -1;
        }
        cr_sort_and_remove_overlap(&mut cr_result);
        (*cr).len = 0;
        if cr_op(
            cr,
            cr_result.points,
            cr_result.len,
            cr_sub.points,
            cr_sub.len,
            CR_OP_UNION,
        ) != 0
        {
            return -1;
        }
        0
    })();
    for p in [&cr_inter, &cr_mask, &cr_result, &cr_sub] {
        cr_free(p);
    }
    ret
}

fn unicode_get_short_code(c: u32) -> u32 {
    static unicode_short_table: [u16; 2] = [0x2044, 0x2215];
    if c < 0x80 {
        c
    } else if c < 0x80 + 0x50 {
        c - 0x80 + 0x300
    } else {
        unicode_short_table[(c - 0x80 - 0x50) as usize] as u32
    }
}
fn unicode_get_lower_simple(mut c: u32) -> u32 {
    if c < 0x100 || (c >= 0x410 && c <= 0x42f) {
        c += 0x20;
    } else {
        c += 1;
    }
    c
}
unsafe fn unicode_get16(p: *const u8) -> u16 {
    *p as u16 | ((*p.add(1) as u16) << 8)
}
unsafe fn unicode_decomp_entry(
    res: &mut [u32],
    mut c: u32,
    idx: usize,
    code: u32,
    len: u32,
    type_: u32,
) -> i32 {
    if type_ == DECOMP_TYPE_C1 {
        res[0] = unicode_decomp_table2[idx] as u32;
        return 1;
    }
    let mut d = unicode_decomp_data
        .as_ptr()
        .add(unicode_decomp_table2[idx] as usize);
    match type_ {
        DECOMP_TYPE_L1..=DECOMP_TYPE_L7 => {
            let l = (type_ - DECOMP_TYPE_L1 + 1) as usize;
            d = d.add((c - code) as usize * l * 2);
            for i in 0..l {
                res[i] = unicode_get16(d.add(2 * i)) as u32;
                if res[i] == 0 {
                    return 0;
                }
            }
            l as i32
        }
        DECOMP_TYPE_LL1 | DECOMP_TYPE_LL2 => {
            let l = (type_ - DECOMP_TYPE_LL1 + 1) as usize;
            let mut k = (c - code) as usize * l;
            let p = len as usize * l * 2;
            for i in 0..l {
                let c1 = unicode_get16(d.add(2 * k)) as u32
                    | (((*d.add(p + k / 4) as u32 >> ((k % 4) * 2)) & 3) << 16);
                if c1 == 0 {
                    return 0;
                }
                res[i] = c1;
                k += 1;
            }
            l as i32
        }
        DECOMP_TYPE_S1..=DECOMP_TYPE_S5 => {
            let l = (type_ - DECOMP_TYPE_S1 + 1) as usize;
            d = d.add((c - code) as usize * l);
            for i in 0..l {
                res[i] = unicode_get_short_code(*d.add(i) as u32);
                if res[i] == 0 {
                    return 0;
                }
            }
            l as i32
        }
        DECOMP_TYPE_I1..=DECOMP_TYPE_I4_2 => {
            let (l, p) = if type_ == DECOMP_TYPE_I1 {
                (1, 0)
            } else {
                let l = 2 + ((type_ - DECOMP_TYPE_I2_0) >> 1);
                (l, ((type_ - DECOMP_TYPE_I2_0) & 1) + (l > 2) as u32)
            };
            for i in 0..l {
                let mut c1 = unicode_get16(d.add(2 * i as usize)) as u32;
                if i == p {
                    c1 += c - code;
                }
                res[i as usize] = c1;
            }
            l as i32
        }
        DECOMP_TYPE_B1..=DECOMP_TYPE_B18 => {
            let l = if type_ == DECOMP_TYPE_B18 {
                18
            } else {
                type_ - DECOMP_TYPE_B1 + 1
            };
            let c_min = unicode_get16(d) as u32;
            d = d.add(2 + (c - code) as usize * l as usize);
            for i in 0..l as usize {
                let c1 = *d.add(i) as u32;
                res[i] = if c1 == 0xff { 0x20 } else { c1 + c_min };
            }
            l as i32
        }
        DECOMP_TYPE_LS2 => {
            d = d.add((c - code) as usize * 3);
            res[0] = unicode_get16(d) as u32;
            if res[0] == 0 {
                return 0;
            }
            res[1] = unicode_get_short_code(*d.add(2) as u32);
            2
        }
        DECOMP_TYPE_PAT3 => {
            res[0] = unicode_get16(d) as u32;
            res[2] = unicode_get16(d.add(2)) as u32;
            d = d.add(4 + (c - code) as usize * 2);
            res[1] = unicode_get16(d) as u32;
            3
        }
        DECOMP_TYPE_S2_UL | DECOMP_TYPE_LS2_UL => {
            let c1 = c - code;
            if type_ == DECOMP_TYPE_S2_UL {
                d = d.add((c1 & !1) as usize);
                c = unicode_get_short_code(*d as u32);
                d = d.add(1);
            } else {
                d = d.add((c1 >> 1) as usize * 3);
                c = unicode_get16(d) as u32;
                d = d.add(2);
            }
            if c1 & 1 != 0 {
                c = unicode_get_lower_simple(c);
            }
            res[0] = c;
            res[1] = unicode_get_short_code(*d as u32);
            2
        }
        _ => 0,
    }
}
#[doc(hidden)]
pub unsafe fn unicode_decomp_char(res: &mut [u32], c: u32, is_compat1: i32) -> i32 {
    let (mut idx_min, mut idx_max) = (0i32, unicode_decomp_table1.len() as i32 - 1);
    while idx_min <= idx_max {
        let idx = ((idx_max + idx_min) / 2) as usize;
        let v = unicode_decomp_table1[idx];
        let code = v >> (32 - 18);
        let len = (v >> (32 - 18 - 7)) & 0x7f;
        if c < code {
            idx_max = idx as i32 - 1;
        } else if c >= code + len {
            idx_min = idx as i32 + 1;
        } else {
            let is_compat = v & 1;
            if (is_compat1 as u32) < is_compat {
                break;
            }
            let type_ = (v >> (32 - 18 - 7 - 6)) & 0x3f;
            return unicode_decomp_entry(res, c, idx, code, len, type_);
        }
    }
    0
}
#[doc(hidden)]
pub unsafe fn unicode_compose_pair(c0: u32, c1: u32) -> u32 {
    let (mut idx_min, mut idx_max) = (0i32, unicode_comp_table.len() as i32 - 1);
    while idx_min <= idx_max {
        let idx = ((idx_max + idx_min) / 2) as usize;
        let idx1 = unicode_comp_table[idx] as u32;
        let d_idx = idx1 >> 6;
        let d_offset = idx1 & 0x3f;
        let v = unicode_decomp_table1[d_idx as usize];
        let code = v >> (32 - 18);
        let len = (v >> (32 - 18 - 7)) & 0x7f;
        let type_ = (v >> (32 - 18 - 7 - 6)) & 0x3f;
        let ch = code + d_offset;
        let mut pair = [0; 2];
        unicode_decomp_entry(&mut pair, ch, d_idx as usize, code, len, type_);
        let mut d = c0.wrapping_sub(pair[0]) as i32;
        if d == 0 {
            d = c1.wrapping_sub(pair[1]) as i32;
        }
        if d < 0 {
            idx_max = idx as i32 - 1;
        } else if d > 0 {
            idx_min = idx as i32 + 1;
        } else {
            return ch;
        }
    }
    0
}
#[doc(hidden)]
pub unsafe fn unicode_get_cc(c: u32) -> i32 {
    let mut code = 0;
    let pos = get_index_pos(
        &mut code,
        c,
        unicode_cc_index.as_ptr(),
        (unicode_cc_index.len() / 3) as i32,
    );
    if pos < 0 {
        return 0;
    }
    let mut p = unicode_cc_table.as_ptr().add(pos as usize);
    loop {
        let b = *p as u32;
        p = p.add(1);
        let type_ = b >> 6;
        let mut n = b & 0x3f;
        if n < 48 {
        } else if n < 56 {
            n = (n - 48) << 8;
            n |= *p as u32;
            p = p.add(1);
            n += 48;
        } else {
            n = (n - 56) << 8;
            n |= (*p as u32) << 8;
            p = p.add(1);
            n |= *p as u32;
            p = p.add(1);
            n += 48 + (1 << 11);
        }
        if type_ <= 1 {
            p = p.add(1);
        }
        let c1 = code + n + 1;
        if c < c1 {
            return match type_ {
                0 => *p.sub(1) as i32,
                1 => (*p.sub(1) as u32 + c - code) as i32,
                2 => 0,
                _ => 230,
            };
        }
        code = c1;
    }
}
unsafe fn sort_cc(buf: *mut i32, len: i32) {
    let mut i = 0;
    while i < len {
        let cc = unicode_get_cc(*buf.add(i as usize) as u32);
        if cc != 0 {
            let start = i;
            let mut j = i + 1;
            while j < len {
                let ch1 = *buf.add(j as usize);
                let cc1 = unicode_get_cc(ch1 as u32);
                if cc1 == 0 {
                    break;
                }
                let mut k = j - 1;
                while k >= start {
                    if unicode_get_cc(*buf.add(k as usize) as u32) <= cc1 {
                        break;
                    }
                    *buf.add((k + 1) as usize) = *buf.add(k as usize);
                    k -= 1;
                }
                *buf.add((k + 1) as usize) = ch1;
                j += 1;
            }
            i = j;
        }
        i += 1;
    }
}
unsafe fn to_nfd_rec(dbuf: *mut DynBuf, src: *const i32, src_len: i32, is_compat: i32) {
    let mut res = [0; UNICODE_DECOMP_LEN_MAX];
    for i in 0..src_len {
        let mut c = *src.add(i as usize) as u32;
        if c >= 0xac00 && c < 0xd7a4 {
            c -= 0xac00;
            dbuf_put_u32(dbuf, 0x1100 + c / 588);
            dbuf_put_u32(dbuf, 0x1161 + (c % 588) / 28);
            let v = c % 28;
            if v != 0 {
                dbuf_put_u32(dbuf, 0x11a7 + v);
            }
        } else {
            let l = unicode_decomp_char(&mut res, c, is_compat);
            if l != 0 {
                to_nfd_rec(dbuf, res.as_ptr().cast(), l, is_compat);
            } else {
                dbuf_put_u32(dbuf, c);
            }
        }
    }
}
unsafe fn compose_pair(c0: u32, c1: u32) -> u32 {
    if c0 >= 0x1100 && c0 < 0x1100 + 19 && c1 >= 0x1161 && c1 < 0x1161 + 21 {
        0xac00 + (c0 - 0x1100) * 588 + (c1 - 0x1161) * 28
    } else if c0 >= 0xac00
        && c0 < 0xac00 + 11172
        && (c0 - 0xac00) % 28 == 0
        && c1 >= 0x11a7
        && c1 < 0x11a7 + 28
    {
        c0 + c1 - 0x11a7
    } else {
        unicode_compose_pair(c0, c1)
    }
}
pub unsafe fn unicode_normalize(
    pdst: *mut *mut u32,
    src: *const u32,
    src_len: i32,
    n_type: UnicodeNormalizationEnum,
    opaque: *mut c_void,
    realloc_func: Option<DynBufReallocFunc>,
) -> i32 {
    let is_compat = n_type >> 1;
    let mut dbuf = DynBuf::default();
    dbuf_init2(&mut dbuf, opaque, realloc_func);
    if dbuf_claim(&mut dbuf, 4 * src_len as usize) != 0 {
        *pdst = ptr::null_mut();
        return -1;
    }
    if n_type == UNICODE_NFC {
        let mut i = 0;
        while i < src_len && *src.add(i as usize) < 0x100 {
            i += 1;
        }
        if i == src_len {
            memcpy_no_ub(dbuf.buf.cast(), src.cast(), src_len as usize * 4);
            *pdst = dbuf.buf.cast();
            return src_len;
        }
    }
    to_nfd_rec(&mut dbuf, src.cast(), src_len, is_compat);
    // Preserve the upstream failure path's ownership behavior; no Drop on
    // DynBuf. The source does not free its accumulated buffer on this path.
    if dbuf_error(&dbuf) != 0 {
        *pdst = ptr::null_mut();
        return -1;
    }
    let buf = dbuf.buf.cast::<i32>();
    let buf_len = (dbuf.size / 4) as i32;
    sort_cc(buf, buf_len);
    if buf_len <= 1 || n_type & 1 != 0 {
        *pdst = buf.cast();
        return buf_len;
    }
    let (mut i, mut out_len) = (1, 1);
    while i < buf_len {
        let mut last_cc = unicode_get_cc(*buf.add(i as usize) as u32);
        let mut starter_pos = out_len - 1;
        let mut blocked = false;
        while starter_pos >= 0 {
            let cc = unicode_get_cc(*buf.add(starter_pos as usize) as u32);
            if cc == 0 {
                break;
            }
            if cc >= last_cc {
                blocked = true;
                break;
            }
            last_cc = 256;
            starter_pos -= 1;
        }
        let p = if !blocked && starter_pos >= 0 {
            compose_pair(
                *buf.add(starter_pos as usize) as u32,
                *buf.add(i as usize) as u32,
            )
        } else {
            0
        };
        if p != 0 {
            *buf.add(starter_pos as usize) = p as i32;
            i += 1;
        } else {
            *buf.add(out_len as usize) = *buf.add(i as usize);
            out_len += 1;
            i += 1;
        }
    }
    *pdst = buf.cast();
    out_len
}

unsafe fn unicode_find_name(name_table: *const u8, name: *const c_char) -> i32 {
    let mut p = name_table;
    let mut pos = 0;
    let mut name_len = 0;
    while *name.add(name_len) != 0 {
        name_len += 1;
    }
    while *p != 0 {
        loop {
            let mut len = 0;
            while *p.add(len) != 0 && *p.add(len) != b',' {
                len += 1;
            }
            let comma = *p.add(len) == b',';
            if len == name_len {
                let mut i = 0;
                while i < len && *p.add(i) == *name.add(i) as u8 {
                    i += 1;
                }
                if i == len {
                    return pos;
                }
            }
            p = p.add(len + 1);
            if !comma {
                break;
            }
        }
        pos += 1;
    }
    -1
}
pub unsafe fn unicode_script(cr: *mut CharRange, script_name: *const c_char, is_ext: i32) -> i32 {
    let script_idx = unicode_find_name(unicode_script_name_table.as_ptr(), script_name);
    if script_idx < 0 {
        return -2;
    }
    let is_common = script_idx == UNICODE_SCRIPT_Common || script_idx == UNICODE_SCRIPT_Inherited;
    let (mut cr1_s, mut cr2_s) = (CharRange::default(), CharRange::default());
    let cr1 = if is_ext != 0 {
        cr_init(&mut cr1_s, (*cr).mem_opaque, (*cr).realloc_func);
        cr_init(&mut cr2_s, (*cr).mem_opaque, (*cr).realloc_func);
        &mut cr1_s as *mut CharRange
    } else {
        cr
    };
    let cr2 = &mut cr2_s as *mut CharRange;
    let ret = (|| {
        let mut p = unicode_script_table.as_ptr();
        let p_end = p.add(unicode_script_table.len());
        let mut c = 0;
        while p < p_end {
            let b = *p as u32;
            p = p.add(1);
            let type_ = b >> 7;
            let mut n = b & 0x7f;
            if n < 96 {
            } else if n < 112 {
                n = (n - 96) << 8;
                n |= *p as u32;
                p = p.add(1);
                n += 96;
            } else {
                n = (n - 112) << 16;
                n |= (*p as u32) << 8;
                p = p.add(1);
                n |= *p as u32;
                p = p.add(1);
                n += 96 + (1 << 12);
            }
            let c1 = c + n + 1;
            if type_ != 0 {
                let v = *p as i32;
                p = p.add(1);
                if (v == script_idx || script_idx == UNICODE_SCRIPT_Unknown)
                    && cr_add_interval(cr1, c, c1) != 0
                {
                    return -1;
                }
            }
            c = c1;
        }
        if script_idx == UNICODE_SCRIPT_Unknown && cr_invert(cr1) != 0 {
            return -1;
        }
        if is_ext != 0 {
            p = unicode_script_ext_table.as_ptr();
            let p_end = p.add(unicode_script_ext_table.len());
            c = 0;
            while p < p_end {
                let b = *p as u32;
                p = p.add(1);
                let mut n;
                if b < 128 {
                    n = b;
                } else if b < 192 {
                    n = (b - 128) << 8;
                    n |= *p as u32;
                    p = p.add(1);
                    n += 128;
                } else {
                    n = (b - 192) << 16;
                    n |= (*p as u32) << 8;
                    p = p.add(1);
                    n |= *p as u32;
                    p = p.add(1);
                    n += 128 + (1 << 14);
                }
                let c1 = c + n + 1;
                let v_len = *p as usize;
                p = p.add(1);
                if is_common {
                    if v_len != 0 && cr_add_interval(cr2, c, c1) != 0 {
                        return -1;
                    }
                } else {
                    for i in 0..v_len {
                        if *p.add(i) as i32 == script_idx {
                            if cr_add_interval(cr2, c, c1) != 0 {
                                return -1;
                            }
                            break;
                        }
                    }
                }
                p = p.add(v_len);
                c = c1;
            }
            if is_common {
                if cr_invert(cr2) != 0 {
                    return -1;
                }
                if cr_op(
                    cr,
                    (*cr1).points,
                    (*cr1).len,
                    (*cr2).points,
                    (*cr2).len,
                    CR_OP_INTER,
                ) != 0
                {
                    return -1;
                }
            } else if cr_op(
                cr,
                (*cr1).points,
                (*cr1).len,
                (*cr2).points,
                (*cr2).len,
                CR_OP_UNION,
            ) != 0
            {
                return -1;
            }
        }
        0
    })();
    if ret != 0 {
        // Official 2026-06-04 source ends its fail label with `goto fail`.
        // Non-extension allocation failure consequently never returns. Keep
        // that defined behavior visible. Extension failure double-frees in C
        // (undefined behavior); fail-stop after the first cleanup avoids UB.
        if is_ext != 0 {
            cr_free(cr1);
            cr_free(cr2);
            std::process::abort();
        }
        loop {
            core::hint::spin_loop();
        }
    }
    if is_ext != 0 {
        cr_free(cr1);
        cr_free(cr2);
    }
    0
}
macro_rules! M {
    ($name:ident) => {
        1u32 << $name
    };
}
unsafe fn unicode_general_category1(cr: *mut CharRange, gc_mask: u32) -> i32 {
    let mut p = unicode_gc_table.as_ptr();
    let p_end = p.add(unicode_gc_table.len());
    let mut c = 0;
    while p < p_end {
        let b = *p as u32;
        p = p.add(1);
        let mut n = b >> 5;
        let v = b & 0x1f;
        if n == 7 {
            n = *p as u32;
            p = p.add(1);
            if n < 128 {
                n += 7;
            } else if n < 192 {
                n = (n - 128) << 8;
                n |= *p as u32;
                p = p.add(1);
                n += 7 + 128;
            } else {
                n = (n - 192) << 16;
                n |= (*p as u32) << 8;
                p = p.add(1);
                n |= *p as u32;
                p = p.add(1);
                n += 7 + 128 + (1 << 14);
            }
        }
        let mut c0 = c;
        c += n + 1;
        if v == 31 {
            let b = gc_mask & (M!(UNICODE_GC_Lu) | M!(UNICODE_GC_Ll));
            if b != 0 {
                if b == (M!(UNICODE_GC_Lu) | M!(UNICODE_GC_Ll)) {
                    if cr_add_interval(cr, c0, c) != 0 {
                        return -1;
                    }
                } else {
                    c0 += (gc_mask & M!(UNICODE_GC_Ll) != 0) as u32;
                    while c0 < c {
                        if cr_add_interval(cr, c0, c0 + 1) != 0 {
                            return -1;
                        }
                        c0 += 2;
                    }
                }
            }
        } else if (gc_mask >> v) & 1 != 0 && cr_add_interval(cr, c0, c) != 0 {
            return -1;
        }
    }
    0
}
unsafe fn unicode_prop1(cr: *mut CharRange, prop_idx: usize) -> i32 {
    let mut p = unicode_prop_table[prop_idx];
    let p_end = p.add(unicode_prop_len_table[prop_idx] as usize);
    let (mut c, mut bit) = (0, 0);
    while p < p_end {
        let mut c0 = c;
        let b = *p as u32;
        p = p.add(1);
        if b < 64 {
            c += (b >> 3) + 1;
            if bit != 0 && cr_add_interval(cr, c0, c) != 0 {
                return -1;
            }
            bit ^= 1;
            c0 = c;
            c += (b & 7) + 1;
        } else if b >= 0x80 {
            c += b - 0x80 + 1;
        } else if b < 0x60 {
            c += (((b - 0x40) << 8) | *p as u32) + 1;
            p = p.add(1);
        } else {
            c += (((b - 0x60) << 16) | ((*p as u32) << 8) | *p.add(1) as u32) + 1;
            p = p.add(2);
        }
        if bit != 0 && cr_add_interval(cr, c0, c) != 0 {
            return -1;
        }
        bit ^= 1;
    }
    0
}
unsafe fn unicode_prop_ops(cr: *mut CharRange, ops: &[u32]) -> i32 {
    // C's variadic instruction stream is a typed slice, with the same POP
    // opcodes and operand order. Preserve the fixed four-entry range stack.
    let mut stack = [CharRange::default(); POP_STACK_LEN_MAX];
    let (mut stack_len, mut ip) = (0, 0);
    let ret = (|| loop {
        let op = ops[ip];
        ip += 1;
        match op {
            POP_GC | POP_PROP | POP_CASE => {
                assert!(stack_len < POP_STACK_LEN_MAX);
                let a = ops[ip];
                ip += 1;
                cr_init(&mut stack[stack_len], (*cr).mem_opaque, (*cr).realloc_func);
                stack_len += 1;
                let result = match op {
                    POP_GC => unicode_general_category1(&mut stack[stack_len - 1], a),
                    POP_PROP => unicode_prop1(&mut stack[stack_len - 1], a as usize),
                    _ => unicode_case1(&mut stack[stack_len - 1], a as i32),
                };
                if result != 0 {
                    return -1;
                }
            }
            POP_UNION | POP_INTER | POP_XOR => {
                assert!(stack_len >= 2 && stack_len < POP_STACK_LEN_MAX);
                let cr1 = stack.as_mut_ptr().add(stack_len - 2);
                let cr2 = stack.as_mut_ptr().add(stack_len - 1);
                let cr3 = stack.as_mut_ptr().add(stack_len);
                stack_len += 1;
                cr_init(cr3, (*cr).mem_opaque, (*cr).realloc_func);
                if cr_op(
                    cr3,
                    (*cr1).points,
                    (*cr1).len,
                    (*cr2).points,
                    (*cr2).len,
                    (op - POP_UNION) as i32 + CR_OP_UNION,
                ) != 0
                {
                    return -1;
                }
                cr_free(cr1);
                cr_free(cr2);
                *cr1 = *cr3;
                stack_len -= 2;
            }
            POP_INVERT => {
                assert!(stack_len >= 1);
                if cr_invert(&mut stack[stack_len - 1]) != 0 {
                    return -1;
                }
            }
            POP_END => {
                assert_eq!(stack_len, 1);
                return cr_copy(cr, &stack[0]);
            }
            _ => std::process::abort(),
        }
    })();
    if ret != 0 {
        for i in 0..stack_len {
            cr_free(&stack[i]);
        }
    } else {
        cr_free(&stack[0]);
    }
    ret
}
static unicode_gc_mask_table: [u32; 8] = [
    M!(UNICODE_GC_Lu) | M!(UNICODE_GC_Ll) | M!(UNICODE_GC_Lt),
    M!(UNICODE_GC_Lu)
        | M!(UNICODE_GC_Ll)
        | M!(UNICODE_GC_Lt)
        | M!(UNICODE_GC_Lm)
        | M!(UNICODE_GC_Lo),
    M!(UNICODE_GC_Mn) | M!(UNICODE_GC_Mc) | M!(UNICODE_GC_Me),
    M!(UNICODE_GC_Nd) | M!(UNICODE_GC_Nl) | M!(UNICODE_GC_No),
    M!(UNICODE_GC_Sm) | M!(UNICODE_GC_Sc) | M!(UNICODE_GC_Sk) | M!(UNICODE_GC_So),
    M!(UNICODE_GC_Pc)
        | M!(UNICODE_GC_Pd)
        | M!(UNICODE_GC_Ps)
        | M!(UNICODE_GC_Pe)
        | M!(UNICODE_GC_Pi)
        | M!(UNICODE_GC_Pf)
        | M!(UNICODE_GC_Po),
    M!(UNICODE_GC_Zs) | M!(UNICODE_GC_Zl) | M!(UNICODE_GC_Zp),
    M!(UNICODE_GC_Cc)
        | M!(UNICODE_GC_Cf)
        | M!(UNICODE_GC_Cs)
        | M!(UNICODE_GC_Co)
        | M!(UNICODE_GC_Cn),
];
pub unsafe fn unicode_general_category(cr: *mut CharRange, gc_name: *const c_char) -> i32 {
    let gc_idx = unicode_find_name(unicode_gc_name_table.as_ptr(), gc_name);
    if gc_idx < 0 {
        return -2;
    }
    let gc_mask = if gc_idx <= UNICODE_GC_Co {
        1u32 << gc_idx
    } else {
        unicode_gc_mask_table[(gc_idx - UNICODE_GC_LC) as usize]
    };
    unicode_general_category1(cr, gc_mask)
}
include!("libunicode_property_dispatch.rs");

unsafe fn unicode_sequence_prop1(
    seq_prop_idx: i32,
    cb: UnicodeSequencePropCB,
    opaque: *mut c_void,
    cr: *mut CharRange,
) -> i32 {
    let mut seq = [0u32; SEQ_MAX_LEN];
    match seq_prop_idx {
        UNICODE_SEQUENCE_PROP_Basic_Emoji => {
            if unicode_prop1(cr, UNICODE_PROP_Basic_Emoji1 as usize) < 0 {
                return -1;
            }
            for i in (0..(*cr).len as usize).step_by(2) {
                for c in *(*cr).points.add(i)..*(*cr).points.add(i + 1) {
                    seq[0] = c;
                    cb(opaque, seq.as_ptr(), 1);
                }
            }
            (*cr).len = 0;
            if unicode_prop1(cr, UNICODE_PROP_Basic_Emoji2 as usize) < 0 {
                return -1;
            }
            for i in (0..(*cr).len as usize).step_by(2) {
                for c in *(*cr).points.add(i)..*(*cr).points.add(i + 1) {
                    seq[0] = c;
                    seq[1] = 0xfe0f;
                    cb(opaque, seq.as_ptr(), 2);
                }
            }
        }
        UNICODE_SEQUENCE_PROP_RGI_Emoji_Modifier_Sequence => {
            if unicode_prop1(cr, UNICODE_PROP_Emoji_Modifier_Base as usize) < 0 {
                return -1;
            }
            for i in (0..(*cr).len as usize).step_by(2) {
                for c in *(*cr).points.add(i)..*(*cr).points.add(i + 1) {
                    for j in 0..5 {
                        seq[0] = c;
                        seq[1] = 0x1f3fb + j;
                        cb(opaque, seq.as_ptr(), 2);
                    }
                }
            }
        }
        UNICODE_SEQUENCE_PROP_RGI_Emoji_Flag_Sequence => {
            if unicode_prop1(cr, UNICODE_PROP_RGI_Emoji_Flag_Sequence as usize) < 0 {
                return -1;
            }
            for i in (0..(*cr).len as usize).step_by(2) {
                for c in *(*cr).points.add(i)..*(*cr).points.add(i + 1) {
                    let c0 = c / 26;
                    let c1 = c % 26;
                    seq[0] = 0x1f1e6 + c0;
                    seq[1] = 0x1f1e6 + c1;
                    cb(opaque, seq.as_ptr(), 2);
                }
            }
        }
        UNICODE_SEQUENCE_PROP_RGI_Emoji_ZWJ_Sequence => {
            let tab = &unicode_rgi_emoji_zwj_sequence;
            let mut i = 0;
            while i < tab.len() {
                let len = tab[i] as usize;
                i += 1;
                let (mut k, mut mod_, mut mod_count, mut hc_pos) = (0usize, 0u32, 0usize, -1i32);
                let mut mod_pos = [0usize; 2];
                for j in 0..len {
                    let mut code = tab[i] as u32;
                    i += 1;
                    code |= (tab[i] as u32) << 8;
                    i += 1;
                    let pres = code >> 15;
                    let mod1 = (code >> 13) & 3;
                    code &= 0x1fff;
                    let c = if code < 0x1000 {
                        code + 0x2000
                    } else {
                        0x1f000 + (code - 0x1000)
                    };
                    if c == 0x1f9b0 {
                        hc_pos = k as i32;
                    }
                    seq[k] = c;
                    k += 1;
                    if mod1 != 0 {
                        assert!(mod_count < 2);
                        mod_ = mod1;
                        mod_pos[mod_count] = k;
                        mod_count += 1;
                        seq[k] = 0;
                        k += 1;
                    }
                    if pres != 0 {
                        seq[k] = 0xfe0f;
                        k += 1;
                    }
                    if j < len - 1 {
                        seq[k] = 0x200d;
                        k += 1;
                    }
                }
                let n_mod = match mod_ {
                    1 => 5,
                    2 => 25,
                    3 => 20,
                    _ => 1,
                };
                let n_hc = if hc_pos >= 0 { 4 } else { 1 };
                for hc_idx in 0..n_hc {
                    for mod_idx in 0..n_mod {
                        if hc_pos >= 0 {
                            seq[hc_pos as usize] = 0x1f9b0 + hc_idx;
                        }
                        match mod_ {
                            1 => {
                                seq[mod_pos[0]] = 0x1f3fb + mod_idx;
                            }
                            2 | 3 => {
                                let mut i0 = mod_idx / 5;
                                let i1 = mod_idx % 5;
                                if mod_ == 3 && i0 >= i1 {
                                    i0 += 1;
                                }
                                seq[mod_pos[0]] = 0x1f3fb + i0;
                                seq[mod_pos[1]] = 0x1f3fb + i1;
                            }
                            _ => {}
                        }
                        cb(opaque, seq.as_ptr(), k as i32);
                    }
                }
            }
        }
        UNICODE_SEQUENCE_PROP_RGI_Emoji_Tag_Sequence => {
            let mut i = 0;
            while i < unicode_rgi_emoji_tag_sequence.len() {
                let mut j = 0;
                seq[j] = 0x1f3f4;
                j += 1;
                loop {
                    let c = unicode_rgi_emoji_tag_sequence[i] as u32;
                    i += 1;
                    if c == 0 {
                        break;
                    }
                    seq[j] = 0xe0000 + c;
                    j += 1;
                }
                seq[j] = 0xe007f;
                j += 1;
                cb(opaque, seq.as_ptr(), j as i32);
            }
        }
        UNICODE_SEQUENCE_PROP_Emoji_Keycap_Sequence => {
            if unicode_prop1(cr, UNICODE_PROP_Emoji_Keycap_Sequence as usize) < 0 {
                return -1;
            }
            for i in (0..(*cr).len as usize).step_by(2) {
                for c in *(*cr).points.add(i)..*(*cr).points.add(i + 1) {
                    seq[0] = c;
                    seq[1] = 0xfe0f;
                    seq[2] = 0x20e3;
                    cb(opaque, seq.as_ptr(), 3);
                }
            }
        }
        UNICODE_SEQUENCE_PROP_RGI_Emoji => {
            for i in
                UNICODE_SEQUENCE_PROP_Basic_Emoji..=UNICODE_SEQUENCE_PROP_RGI_Emoji_ZWJ_Sequence
            {
                let ret = unicode_sequence_prop1(i, cb, opaque, cr);
                if ret < 0 {
                    return ret;
                }
                (*cr).len = 0;
            }
        }
        _ => return -2,
    }
    0
}
pub unsafe fn unicode_sequence_prop(
    prop_name: *const c_char,
    cb: UnicodeSequencePropCB,
    opaque: *mut c_void,
    cr: *mut CharRange,
) -> i32 {
    let seq_prop_idx = unicode_find_name(unicode_sequence_prop_name_table.as_ptr(), prop_name);
    if seq_prop_idx < 0 {
        return -2;
    }
    unicode_sequence_prop1(seq_prop_idx, cb, opaque, cr)
}
#[cfg(test)]
#[path = "../tests/unicode_differential.rs"]
mod tests;
