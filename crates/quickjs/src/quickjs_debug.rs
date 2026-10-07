// quickjs.c:7379-7500. Original LEB128 and PC-to-source location path. MIT.
unsafe fn dbuf_put_leb128(s: *mut crate::cutils_header::DynBuf, mut v: u32) {
    loop {
        let a = v & 0x7f;
        v >>= 7;
        if v != 0 {
            crate::cutils::dbuf_putc(s, (a | 0x80) as u8);
        } else {
            crate::cutils::dbuf_putc(s, a as u8);
            break;
        }
    }
}
unsafe fn dbuf_put_sleb128(s: *mut crate::cutils_header::DynBuf, v1: i32) {
    let v = v1 as u32;
    dbuf_put_leb128(s, v.wrapping_mul(2) ^ (v >> 31).wrapping_neg());
}
unsafe fn get_leb128(pval: *mut u32, buf: *const u8, buf_end: *const u8) -> i32 {
    let mut p = buf;
    let mut v = 0u32;
    for i in 0..5 {
        if p >= buf_end {
            break;
        }
        let a = *p as u32;
        p = p.add(1);
        v |= (a & 0x7f) << (i * 7);
        if a & 0x80 == 0 {
            *pval = v;
            return p.offset_from(buf) as i32;
        }
    }
    *pval = 0;
    -1
}
unsafe fn get_sleb128(pval: *mut i32, buf: *const u8, buf_end: *const u8) -> i32 {
    let mut val = 0;
    let ret = get_leb128(&mut val, buf, buf_end);
    if ret < 0 {
        *pval = 0;
        return -1;
    }
    *pval = ((val >> 1) ^ (val & 1).wrapping_neg()) as i32;
    ret
}
unsafe fn find_line_num(
    _ctx: *mut JSContext,
    b: *mut JSFunctionBytecode,
    pc_value: u32,
    pcol_num: *mut i32,
) -> i32 {
    // C goto fail/done become a labeled expression; no format error is thrown.
    let result = 'decode: {
        if (*b).has_debug() == 0 || (*b).debug.pc2line_buf.is_null() {
            break 'decode None;
        }
        let mut p = (*b).debug.pc2line_buf.cast_const();
        let p_end = p.offset((*b).debug.pc2line_len as isize);
        let mut val = 0;
        let mut ret = get_leb128(&mut val, p, p_end);
        if ret < 0 {
            break 'decode None;
        }
        p = p.add(ret as usize);
        let mut line_num = val.wrapping_add(1) as i32;
        ret = get_leb128(&mut val, p, p_end);
        if ret < 0 {
            break 'decode None;
        }
        p = p.add(ret as usize);
        let mut col_num = val.wrapping_add(1) as i32;
        if pc_value != u32::MAX {
            let mut pc = 0i32;
            while p < p_end {
                let mut op = *p as u32;
                p = p.add(1);
                let new_line_num;
                let mut v = 0;
                if op == 0 {
                    ret = get_leb128(&mut val, p, p_end);
                    if ret < 0 {
                        break 'decode None;
                    }
                    pc = (pc as u32).wrapping_add(val) as i32;
                    p = p.add(ret as usize);
                    ret = get_sleb128(&mut v, p, p_end);
                    if ret < 0 {
                        break 'decode None;
                    }
                    p = p.add(ret as usize);
                    new_line_num = line_num.wrapping_add(v);
                } else {
                    op = op.wrapping_sub(PC2LINE_OP_FIRST as u32);
                    pc = (pc as u32).wrapping_add(op / PC2LINE_RANGE as u32) as i32;
                    new_line_num = (line_num as u32)
                        .wrapping_add(op % PC2LINE_RANGE as u32)
                        .wrapping_add(PC2LINE_BASE as u32)
                        as i32;
                }
                ret = get_sleb128(&mut v, p, p_end);
                if ret < 0 {
                    break 'decode None;
                }
                p = p.add(ret as usize);
                let new_col_num = col_num.wrapping_add(v);
                if pc_value < pc as u32 {
                    break;
                }
                line_num = new_line_num;
                col_num = new_col_num;
            }
        }
        Some((line_num, col_num))
    };
    if let Some((line, col)) = result {
        *pcol_num = col;
        line
    } else {
        *pcol_num = 0;
        0
    }
}
