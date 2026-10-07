//! libregexp.c. Copyright 2017-2018 Fabrice Bellard; MIT, see ../LICENSE.
//! Parser, bytecode compiler and backtracking VM follow the official C source.
pub use super::libregexp_header::*;
use super::{cutils::*, libregexp_opcode::*, libunicode::*};
use core::{
    ffi::{c_char, c_void},
    ptr,
};
const CAPTURE_COUNT_MAX: i32 = 255;
const REGISTER_COUNT_MAX: i32 = 255;
const INTERRUPT_COUNTER_INIT: i32 = 10000;
const CP_LS: u32 = 0x2028;
const CP_PS: u32 = 0x2029;
const RE_HEADER_FLAGS: usize = 0;
const RE_HEADER_CAPTURE_COUNT: usize = 2;
const RE_HEADER_REGISTER_COUNT: usize = 3;
const RE_HEADER_BYTECODE_LEN: usize = 4;
const RE_HEADER_LEN: usize = 8;
fn is_digit(c: i32) -> bool {
    c >= b'0' as i32 && c <= b'9' as i32
}
pub unsafe fn lre_parse_escape(pp: *mut *const u8, allow_utf16: i32) -> i32 {
    let mut p = *pp;
    let mut c = *p as u32;
    p = p.add(1);
    match c as u8 {
        b'b' => c = 8,
        b'f' => c = 12,
        b'n' => c = 10,
        b'r' => c = 13,
        b't' => c = 9,
        b'v' => c = 11,
        b'x' => {
            let h0 = from_hex(*p as i32);
            p = p.add(1);
            if h0 < 0 {
                return -1;
            }
            let h1 = from_hex(*p as i32);
            p = p.add(1);
            if h1 < 0 {
                return -1;
            }
            c = ((h0 << 4) | h1) as u32;
        }
        b'u' => {
            if *p == b'{' && allow_utf16 != 0 {
                p = p.add(1);
                c = 0;
                loop {
                    let h = from_hex(*p as i32);
                    p = p.add(1);
                    if h < 0 {
                        return -1;
                    }
                    c = (c << 4) | h as u32;
                    if c > 0x10ffff {
                        return -1;
                    }
                    if *p == b'}' {
                        break;
                    }
                }
                p = p.add(1);
            } else {
                c = 0;
                for _ in 0..4 {
                    let h = from_hex(*p as i32);
                    p = p.add(1);
                    if h < 0 {
                        return -1;
                    }
                    c = (c << 4) | h as u32;
                }
                if is_hi_surrogate(c) != 0 && allow_utf16 == 2 && *p == b'\\' && *p.add(1) == b'u' {
                    let mut c1 = 0;
                    let mut i = 0;
                    while i < 4 {
                        let h = from_hex(*p.add(2 + i) as i32);
                        if h < 0 {
                            break;
                        }
                        c1 = (c1 << 4) | h as u32;
                        i += 1;
                    }
                    if i == 4 && is_lo_surrogate(c1) != 0 {
                        p = p.add(6);
                        c = from_surrogate(c, c1);
                    }
                }
            }
        }
        b'0'..=b'7' => {
            c -= b'0' as u32;
            if allow_utf16 == 2 {
                if c != 0 || is_digit(*p as i32) {
                    return -1;
                }
            } else {
                let mut v = (*p as u32).wrapping_sub(b'0' as u32);
                if v <= 7 {
                    c = (c << 3) | v;
                    p = p.add(1);
                    if c < 32 {
                        v = (*p as u32).wrapping_sub(b'0' as u32);
                        if v <= 7 {
                            c = (c << 3) | v;
                            p = p.add(1);
                        }
                    }
                }
            }
        }
        _ => return -2,
    }
    *pp = p;
    c as i32
}
pub unsafe fn lre_get_alloc_count(bc_buf: *const u8) -> i32 {
    *bc_buf.add(RE_HEADER_CAPTURE_COUNT) as i32 * 2 + *bc_buf.add(RE_HEADER_REGISTER_COUNT) as i32
}
pub unsafe fn lre_get_capture_count(bc_buf: *const u8) -> i32 {
    *bc_buf.add(RE_HEADER_CAPTURE_COUNT) as i32
}
pub unsafe fn lre_get_flags(bc_buf: *const u8) -> i32 {
    get_u16(bc_buf.add(RE_HEADER_FLAGS)) as i32
}
pub unsafe fn lre_get_groupnames(bc_buf: *const u8) -> *const c_char {
    if lre_get_flags(bc_buf) & LRE_FLAG_NAMED_GROUPS == 0 {
        return ptr::null();
    }
    let len = get_u32(bc_buf.add(RE_HEADER_BYTECODE_LEN));
    bc_buf.add(RE_HEADER_LEN + len as usize).cast()
}
fn is_line_terminator(c: u32) -> bool {
    c == b'\n' as u32 || c == b'\r' as u32 || c == CP_LS || c == CP_PS
}
unsafe fn GET_CHAR(cptr: &mut *const u8, cbuf_end: *const u8, cbuf_type: i32) -> u32 {
    if cbuf_type == 0 {
        let c = **cptr as u32;
        *cptr = (*cptr).add(1);
        c
    } else {
        let mut p = (*cptr).cast::<u16>();
        let end = cbuf_end.cast::<u16>();
        let mut c = *p as u32;
        p = p.add(1);
        if is_hi_surrogate(c) != 0 && cbuf_type == 2 && p < end && is_lo_surrogate(*p as u32) != 0 {
            c = from_surrogate(c, *p as u32);
            p = p.add(1);
        }
        *cptr = p.cast();
        c
    }
}
unsafe fn PEEK_CHAR(cptr: *const u8, cbuf_end: *const u8, cbuf_type: i32) -> u32 {
    let mut p = cptr;
    GET_CHAR(&mut p, cbuf_end, cbuf_type)
}
unsafe fn GET_PREV_CHAR(cptr: &mut *const u8, cbuf_start: *const u8, cbuf_type: i32) -> u32 {
    if cbuf_type == 0 {
        *cptr = (*cptr).sub(1);
        **cptr as u32
    } else {
        let mut p = (*cptr).cast::<u16>().sub(1);
        let start = cbuf_start.cast::<u16>();
        let mut c = *p as u32;
        if is_lo_surrogate(c) != 0
            && cbuf_type == 2
            && p > start
            && is_hi_surrogate(*p.sub(1) as u32) != 0
        {
            p = p.sub(1);
            c = from_surrogate(*p as u32, c);
        }
        *cptr = p.cast();
        c
    }
}
unsafe fn PEEK_PREV_CHAR(cptr: *const u8, cbuf_start: *const u8, cbuf_type: i32) -> u32 {
    let mut p = cptr;
    GET_PREV_CHAR(&mut p, cbuf_start, cbuf_type)
}
unsafe fn PREV_CHAR(cptr: &mut *const u8, cbuf_start: *const u8, cbuf_type: i32) {
    GET_PREV_CHAR(cptr, cbuf_start, cbuf_type);
}
const RE_EXEC_STATE_SPLIT: usize = 0;
const RE_EXEC_STATE_LOOKAHEAD: usize = 1;
const RE_EXEC_STATE_NEGATIVE_LOOKAHEAD: usize = 2;
const BP_TYPE_BITS: usize = if usize::BITS >= 64 { 3 } else { 2 };
const BP_VALUE_BITS: usize = usize::BITS as usize - BP_TYPE_BITS;
const BP_VALUE_MASK: usize = usize::MAX >> BP_TYPE_BITS;
#[repr(C)]
#[derive(Clone, Copy)]
union StackElem {
    ptr: *mut u8,
    val: isize,
    bits: usize,
}
impl StackElem {
    unsafe fn bp_val(&self) -> usize {
        self.bits & BP_VALUE_MASK
    }
    unsafe fn bp_type(&self) -> usize {
        self.bits >> BP_VALUE_BITS
    }
    fn bp(value: usize, type_: usize) -> Self {
        Self {
            bits: (value & BP_VALUE_MASK) | (type_ << BP_VALUE_BITS),
        }
    }
}
#[repr(C)]
struct REExecContext {
    cbuf: *const u8,
    cbuf_end: *const u8,
    cbuf_type: i32,
    capture_count: i32,
    is_unicode: BOOL,
    interrupt_counter: i32,
    opaque: *mut c_void,
    stack_buf: *mut StackElem,
    stack_size: usize,
    static_stack_buf: [StackElem; 32],
}
unsafe fn lre_poll_timeout(s: *mut REExecContext) -> i32 {
    (*s).interrupt_counter -= 1;
    if (*s).interrupt_counter <= 0 {
        (*s).interrupt_counter = INTERRUPT_COUNTER_INIT;
        if lre_check_timeout((*s).opaque) != 0 {
            return LRE_RET_TIMEOUT;
        }
    }
    0
}
#[inline(never)]
unsafe fn stack_realloc(s: *mut REExecContext, n: usize) -> i32 {
    let mut new_size = (*s).stack_size * 3 / 2;
    if new_size < n {
        new_size = n;
    }
    let new_stack = if (*s).stack_buf == ptr::addr_of_mut!((*s).static_stack_buf).cast() {
        let new = lre_realloc(
            (*s).opaque,
            ptr::null_mut(),
            new_size * core::mem::size_of::<StackElem>(),
        )
        .cast::<StackElem>();
        if new.is_null() {
            return -1;
        }
        ptr::copy_nonoverlapping((*s).stack_buf, new, (*s).stack_size);
        new
    } else {
        let new = lre_realloc(
            (*s).opaque,
            (*s).stack_buf.cast(),
            new_size * core::mem::size_of::<StackElem>(),
        )
        .cast::<StackElem>();
        if new.is_null() {
            return -1;
        }
        new
    };
    (*s).stack_size = new_size;
    (*s).stack_buf = new_stack;
    0
}
#[allow(unused_assignments)] // Preserve the C negative-lookahead unwind assignments.
unsafe fn lre_exec_backtrack(
    s: *mut REExecContext,
    capture: *mut *mut u8,
    mut pc: *const u8,
    mut cptr: *const u8,
) -> isize {
    let cbuf_type = (*s).cbuf_type;
    let cbuf_end = (*s).cbuf_end;
    let mut sp = (*s).stack_buf;
    let mut bp = (*s).stack_buf;
    let mut stack_end = (*s).stack_buf.add((*s).stack_size);
    macro_rules! CHECK_STACK_SPACE {
        ($n:expr) => {{
            let n = $n as usize;
            if stack_end.offset_from(sp) < n as isize {
                let saved_sp = sp.offset_from((*s).stack_buf) as usize;
                let saved_bp = bp.offset_from((*s).stack_buf) as usize;
                if stack_realloc(s, saved_sp + n) != 0 {
                    return LRE_RET_MEMORY_ERROR as isize;
                }
                stack_end = (*s).stack_buf.add((*s).stack_size);
                sp = (*s).stack_buf.add(saved_sp);
                bp = (*s).stack_buf.add(saved_bp);
            }
        }};
    }
    macro_rules! SAVE_CAPTURE {
        ($idx:expr,$value:expr) => {{
            let idx = $idx as usize;
            let value = $value;
            CHECK_STACK_SPACE!(2);
            (*sp).val = idx as isize;
            (*sp.add(1)).ptr = *capture.add(idx);
            sp = sp.add(2);
            *capture.add(idx) = value;
        }};
    }
    macro_rules! SAVE_CAPTURE_CHECK {
        ($idx:expr,$value:expr) => {{
            let idx = $idx as usize;
            let value = $value;
            let mut sp1 = sp;
            loop {
                if sp1 > bp {
                    if (*sp1.sub(2)).val == idx as isize {
                        break;
                    }
                    sp1 = sp1.sub(2);
                } else {
                    CHECK_STACK_SPACE!(2);
                    (*sp).val = idx as isize;
                    (*sp.add(1)).ptr = *capture.add(idx);
                    sp = sp.add(2);
                    break;
                }
            }
            *capture.add(idx) = value;
        }};
    }
    macro_rules! no_match{($dispatch:lifetime)=>{{
  loop{
   if bp==(*s).stack_buf{return 0;}
   while sp>bp{*capture.add((*sp.sub(2)).val as usize)=(*sp.sub(1)).ptr;sp=sp.sub(2);}
   pc=(*sp.sub(3)).ptr;cptr=(*sp.sub(2)).ptr;let type_=(*sp.sub(1)).bp_type();bp=(*s).stack_buf.add((*sp.sub(1)).bp_val());sp=sp.sub(3);
   if type_!=RE_EXEC_STATE_LOOKAHEAD{break;}
  }
  if lre_poll_timeout(s)!=0{return LRE_RET_TIMEOUT as isize;}
  continue $dispatch;
 }};}
    'dispatch: loop {
        let opcode = *pc;
        pc = pc.add(1);
        match opcode {
            REOP_match => return 1,
            REOP_lookahead_match => {
                let sp_top = sp;
                loop {
                    let sp1 = sp;
                    sp = bp;
                    pc = (*sp.sub(3)).ptr;
                    cptr = (*sp.sub(2)).ptr;
                    let type_ = (*sp.sub(1)).bp_type();
                    bp = (*s).stack_buf.add((*sp.sub(1)).bp_val());
                    (*sp.sub(1)).ptr = sp1.cast();
                    sp = sp.sub(3);
                    if type_ == RE_EXEC_STATE_LOOKAHEAD {
                        break;
                    }
                }
                if sp != (*s).stack_buf {
                    let mut sp1 = sp;
                    while sp1 < sp_top {
                        let next_sp = (*sp1.add(2)).ptr.cast::<StackElem>();
                        sp1 = sp1.add(3);
                        while sp1 < next_sp {
                            *sp = *sp1;
                            sp = sp.add(1);
                            sp1 = sp1.add(1);
                        }
                    }
                }
            }
            REOP_negative_lookahead_match => {
                loop {
                    while sp > bp {
                        *capture.add((*sp.sub(2)).val as usize) = (*sp.sub(1)).ptr;
                        sp = sp.sub(2);
                    }
                    pc = (*sp.sub(3)).ptr;
                    cptr = (*sp.sub(2)).ptr;
                    let type_ = (*sp.sub(1)).bp_type();
                    bp = (*s).stack_buf.add((*sp.sub(1)).bp_val());
                    sp = sp.sub(3);
                    if type_ == RE_EXEC_STATE_NEGATIVE_LOOKAHEAD {
                        break;
                    }
                }
                no_match!('dispatch);
            }
            REOP_char32 | REOP_char32_i | REOP_char | REOP_char_i => {
                let val = if opcode == REOP_char32 || opcode == REOP_char32_i {
                    let v = get_u32(pc);
                    pc = pc.add(4);
                    v
                } else {
                    let v = get_u16(pc);
                    pc = pc.add(2);
                    v
                };
                if cptr >= cbuf_end {
                    no_match!('dispatch);
                }
                let mut c = GET_CHAR(&mut cptr, cbuf_end, cbuf_type);
                if opcode == REOP_char_i || opcode == REOP_char32_i {
                    c = lre_canonicalize(c, (*s).is_unicode) as u32;
                }
                if val != c {
                    no_match!('dispatch);
                }
            }
            REOP_split_goto_first | REOP_split_next_first => {
                let val = get_u32(pc);
                pc = pc.add(4);
                let pc1;
                if opcode == REOP_split_next_first {
                    pc1 = pc.offset(val as i32 as isize);
                } else {
                    pc1 = pc;
                    pc = pc.offset(val as i32 as isize);
                }
                CHECK_STACK_SPACE!(3);
                (*sp).ptr = pc1.cast_mut();
                (*sp.add(1)).ptr = cptr.cast_mut();
                *sp.add(2) =
                    StackElem::bp(bp.offset_from((*s).stack_buf) as usize, RE_EXEC_STATE_SPLIT);
                sp = sp.add(3);
                bp = sp;
            }
            REOP_lookahead | REOP_negative_lookahead => {
                let val = get_u32(pc);
                pc = pc.add(4);
                CHECK_STACK_SPACE!(3);
                (*sp).ptr = pc.offset(val as i32 as isize).cast_mut();
                (*sp.add(1)).ptr = cptr.cast_mut();
                *sp.add(2) = StackElem::bp(
                    bp.offset_from((*s).stack_buf) as usize,
                    RE_EXEC_STATE_LOOKAHEAD + (opcode - REOP_lookahead) as usize,
                );
                sp = sp.add(3);
                bp = sp;
            }
            REOP_goto => {
                let val = get_u32(pc);
                pc = pc.offset(4 + val as i32 as isize);
                if lre_poll_timeout(s) != 0 {
                    return LRE_RET_TIMEOUT as isize;
                }
            }
            REOP_line_start | REOP_line_start_m => {
                if cptr != (*s).cbuf {
                    if opcode == REOP_line_start {
                        no_match!('dispatch);
                    }
                    let c = PEEK_PREV_CHAR(cptr, (*s).cbuf, cbuf_type);
                    if !is_line_terminator(c) {
                        no_match!('dispatch);
                    }
                }
            }
            REOP_line_end | REOP_line_end_m => {
                if cptr != cbuf_end {
                    if opcode == REOP_line_end {
                        no_match!('dispatch);
                    }
                    let c = PEEK_CHAR(cptr, cbuf_end, cbuf_type);
                    if !is_line_terminator(c) {
                        no_match!('dispatch);
                    }
                }
            }
            REOP_dot | REOP_any | REOP_space | REOP_not_space => {
                if cptr == cbuf_end {
                    no_match!('dispatch);
                }
                let c = GET_CHAR(&mut cptr, cbuf_end, cbuf_type);
                if (opcode == REOP_dot && is_line_terminator(c))
                    || (opcode == REOP_space && lre_is_space(c) == 0)
                    || (opcode == REOP_not_space && lre_is_space(c) != 0)
                {
                    no_match!('dispatch);
                }
            }
            REOP_save_start | REOP_save_end => {
                let val = *pc as usize;
                pc = pc.add(1);
                assert!(val < (*s).capture_count as usize);
                let idx = 2 * val + (opcode - REOP_save_start) as usize;
                SAVE_CAPTURE!(idx, cptr.cast_mut());
            }
            REOP_save_reset => {
                let mut val = *pc as usize;
                let val2 = *pc.add(1) as usize;
                pc = pc.add(2);
                assert!(val2 < (*s).capture_count as usize);
                CHECK_STACK_SPACE!(2 * (val2 - val + 1));
                while val <= val2 {
                    SAVE_CAPTURE!(2 * val, ptr::null_mut());
                    SAVE_CAPTURE!(2 * val + 1, ptr::null_mut());
                    val += 1;
                }
            }
            REOP_set_i32 => {
                let idx = 2 * (*s).capture_count as usize + *pc as usize;
                let val = get_u32(pc.add(1));
                pc = pc.add(5);
                SAVE_CAPTURE_CHECK!(idx, val as usize as *mut u8);
            }
            REOP_loop => {
                let idx = 2 * (*s).capture_count as usize + *pc as usize;
                let val = get_u32(pc.add(1));
                pc = pc.add(5);
                let val2 = (*capture.add(idx) as usize).wrapping_sub(1) as u32;
                SAVE_CAPTURE_CHECK!(idx, val2 as usize as *mut u8);
                if val2 != 0 {
                    pc = pc.offset(val as i32 as isize);
                    if lre_poll_timeout(s) != 0 {
                        return LRE_RET_TIMEOUT as isize;
                    }
                }
            }
            REOP_loop_split_goto_first
            | REOP_loop_split_next_first
            | REOP_loop_check_adv_split_goto_first
            | REOP_loop_check_adv_split_next_first => {
                let idx = 2 * (*s).capture_count as usize + *pc as usize;
                let limit = get_u32(pc.add(1));
                let val = get_u32(pc.add(5));
                pc = pc.add(9);
                let val2 = (*capture.add(idx) as usize).wrapping_sub(1) as u32;
                SAVE_CAPTURE_CHECK!(idx, val2 as usize as *mut u8);
                if val2 > limit {
                    pc = pc.offset(val as i32 as isize);
                    if lre_poll_timeout(s) != 0 {
                        return LRE_RET_TIMEOUT as isize;
                    }
                } else {
                    if (opcode == REOP_loop_check_adv_split_goto_first
                        || opcode == REOP_loop_check_adv_split_next_first)
                        && *capture.add(idx + 1) == cptr.cast_mut()
                        && val2 != limit
                    {
                        no_match!('dispatch);
                    }
                    if val2 != 0 {
                        let pc1;
                        if opcode == REOP_loop_split_next_first
                            || opcode == REOP_loop_check_adv_split_next_first
                        {
                            pc1 = pc.offset(val as i32 as isize);
                        } else {
                            pc1 = pc;
                            pc = pc.offset(val as i32 as isize);
                        }
                        CHECK_STACK_SPACE!(3);
                        (*sp).ptr = pc1.cast_mut();
                        (*sp.add(1)).ptr = cptr.cast_mut();
                        *sp.add(2) = StackElem::bp(
                            bp.offset_from((*s).stack_buf) as usize,
                            RE_EXEC_STATE_SPLIT,
                        );
                        sp = sp.add(3);
                        bp = sp;
                    }
                }
            }
            REOP_set_char_pos => {
                let idx = 2 * (*s).capture_count as usize + *pc as usize;
                pc = pc.add(1);
                SAVE_CAPTURE_CHECK!(idx, cptr.cast_mut());
            }
            REOP_check_advance => {
                let idx = 2 * (*s).capture_count as usize + *pc as usize;
                pc = pc.add(1);
                if *capture.add(idx) == cptr.cast_mut() {
                    no_match!('dispatch);
                }
            }
            REOP_word_boundary
            | REOP_word_boundary_i
            | REOP_not_word_boundary
            | REOP_not_word_boundary_i => {
                let ignore_case =
                    opcode == REOP_word_boundary_i || opcode == REOP_not_word_boundary_i;
                let is_boundary = opcode == REOP_word_boundary || opcode == REOP_word_boundary_i;
                let v1 = if cptr == (*s).cbuf {
                    false
                } else {
                    let c = PEEK_PREV_CHAR(cptr, (*s).cbuf, cbuf_type);
                    if c < 256 {
                        lre_is_word_byte(c as u8) != 0
                    } else {
                        ignore_case && (c == 0x17f || c == 0x212a)
                    }
                };
                let v2 = if cptr >= cbuf_end {
                    false
                } else {
                    let c = PEEK_CHAR(cptr, cbuf_end, cbuf_type);
                    if c < 256 {
                        lre_is_word_byte(c as u8) != 0
                    } else {
                        ignore_case && (c == 0x17f || c == 0x212a)
                    }
                };
                if v1 ^ v2 ^ is_boundary {
                    no_match!('dispatch);
                }
            }
            REOP_back_reference
            | REOP_back_reference_i
            | REOP_backward_back_reference
            | REOP_backward_back_reference_i => {
                let n = *pc as usize;
                pc = pc.add(1);
                let pc1 = pc;
                pc = pc.add(n);
                for i in 0..n {
                    let val = *pc1.add(i) as usize;
                    if val >= (*s).capture_count as usize {
                        no_match!('dispatch);
                    }
                    let cptr1_start = *capture.add(2 * val) as *const u8;
                    let cptr1_end = *capture.add(2 * val + 1) as *const u8;
                    if !cptr1_start.is_null() && !cptr1_end.is_null() {
                        if opcode == REOP_back_reference || opcode == REOP_back_reference_i {
                            let mut cptr1 = cptr1_start;
                            while cptr1 < cptr1_end {
                                if cptr >= cbuf_end {
                                    no_match!('dispatch);
                                }
                                let mut c1 = GET_CHAR(&mut cptr1, cptr1_end, cbuf_type);
                                let mut c2 = GET_CHAR(&mut cptr, cbuf_end, cbuf_type);
                                if opcode == REOP_back_reference_i {
                                    c1 = lre_canonicalize(c1, (*s).is_unicode) as u32;
                                    c2 = lre_canonicalize(c2, (*s).is_unicode) as u32;
                                }
                                if c1 != c2 {
                                    no_match!('dispatch);
                                }
                            }
                        } else {
                            let mut cptr1 = cptr1_end;
                            while cptr1 > cptr1_start {
                                if cptr == (*s).cbuf {
                                    no_match!('dispatch);
                                }
                                let mut c1 = GET_PREV_CHAR(&mut cptr1, cptr1_start, cbuf_type);
                                let mut c2 = GET_PREV_CHAR(&mut cptr, (*s).cbuf, cbuf_type);
                                if opcode == REOP_backward_back_reference_i {
                                    c1 = lre_canonicalize(c1, (*s).is_unicode) as u32;
                                    c2 = lre_canonicalize(c2, (*s).is_unicode) as u32;
                                }
                                if c1 != c2 {
                                    no_match!('dispatch);
                                }
                            }
                        }
                        break;
                    }
                }
            }
            REOP_range | REOP_range_i | REOP_range32 | REOP_range32_i => {
                let n = get_u16(pc) as usize;
                pc = pc.add(2);
                if cptr >= cbuf_end {
                    no_match!('dispatch);
                }
                let mut c = GET_CHAR(&mut cptr, cbuf_end, cbuf_type);
                if opcode == REOP_range_i || opcode == REOP_range32_i {
                    c = lre_canonicalize(c, (*s).is_unicode) as u32;
                }
                let wide = opcode == REOP_range32 || opcode == REOP_range32_i;
                let width = if wide { 4 } else { 2 };
                let stride = 2 * width;
                let read = |idx: usize| {
                    if wide {
                        get_u32(pc.add(idx))
                    } else {
                        get_u16(pc.add(idx))
                    }
                };
                let (mut idx_min, mut idx_max) = (0i32, n as i32 - 1);
                let low = read(0);
                if c < low {
                    no_match!('dispatch);
                }
                let high = read(idx_max as usize * stride + width);
                let mut matched = !wide && c >= 0xffff && high == 0xffff;
                if !matched {
                    if c > high {
                        no_match!('dispatch);
                    }
                    while idx_min <= idx_max {
                        let idx = (idx_min + idx_max) / 2;
                        let low = read(idx as usize * stride);
                        let high = read(idx as usize * stride + width);
                        if c < low {
                            idx_max = idx - 1;
                        } else if c > high {
                            idx_min = idx + 1;
                        } else {
                            matched = true;
                            break;
                        }
                    }
                    if !matched {
                        no_match!('dispatch);
                    }
                }
                pc = pc.add(stride * n);
            }
            REOP_prev => {
                if cptr == (*s).cbuf {
                    no_match!('dispatch);
                }
                PREV_CHAR(&mut cptr, (*s).cbuf, cbuf_type);
            }
            _ => std::process::abort(),
        }
    }
}
pub unsafe fn lre_exec(
    capture: *mut *mut u8,
    bc_buf: *const u8,
    cbuf: *const u8,
    cindex: i32,
    clen: i32,
    cbuf_type: i32,
    opaque: *mut c_void,
) -> i32 {
    let re_flags = lre_get_flags(bc_buf);
    let is_unicode = (re_flags & (LRE_FLAG_UNICODE | LRE_FLAG_UNICODE_SETS) != 0) as i32;
    let mut s = REExecContext {
        cbuf,
        cbuf_end: cbuf.add((clen as usize) << cbuf_type),
        cbuf_type,
        capture_count: *bc_buf.add(RE_HEADER_CAPTURE_COUNT) as i32,
        is_unicode,
        interrupt_counter: INTERRUPT_COUNTER_INIT,
        opaque,
        stack_buf: ptr::null_mut(),
        stack_size: 32,
        static_stack_buf: [StackElem { bits: 0 }; 32],
    };
    if s.cbuf_type == 1 && s.is_unicode != 0 {
        s.cbuf_type = 2;
    }
    s.stack_buf = s.static_stack_buf.as_mut_ptr();
    for i in 0..s.capture_count * 2 {
        *capture.add(i as usize) = ptr::null_mut();
    }
    let mut cptr = cbuf.add((cindex as usize) << cbuf_type);
    if cindex > 0 && cindex < clen && s.cbuf_type == 2 {
        let p = cptr.cast::<u16>();
        if is_lo_surrogate(*p as u32) != 0 && is_hi_surrogate(*p.sub(1) as u32) != 0 {
            cptr = p.sub(1).cast();
        }
    }
    let ret = lre_exec_backtrack(&mut s, capture, bc_buf.add(RE_HEADER_LEN), cptr) as i32;
    if s.stack_buf != s.static_stack_buf.as_mut_ptr() {
        lre_realloc(s.opaque, s.stack_buf.cast(), 0);
    }
    ret
}
#[cfg(test)]
#[path = "../tests/regexp_exec_differential.rs"]
mod tests;

#[cfg(test)]
#[path = "../tests/regexp_compile_differential.rs"]
mod compile_tests;

include!("libregexp_parser.rs");
