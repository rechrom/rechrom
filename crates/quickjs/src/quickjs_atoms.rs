// quickjs.c runtime allocation and atom helpers. MIT.
use super::quickjs_atom::{
    js_atom_init, JS_ATOM_Private_brand, JS_ATOM_Symbol_toPrimitive, JS_ATOM_END,
};
pub unsafe fn js_malloc_rt(rt: *mut JSRuntime, size: usize) -> *mut c_void {
    __js_malloc(&mut (*rt).malloc_ctx, size)
}
pub unsafe fn js_free_rt(rt: *mut JSRuntime, p: *mut c_void) {
    __js_free(&mut (*rt).malloc_ctx, p);
}
pub unsafe fn js_realloc_rt(rt: *mut JSRuntime, p: *mut c_void, size: usize) -> *mut c_void {
    __js_realloc(&mut (*rt).malloc_ctx, p, size)
}
pub unsafe fn js_malloc_usable_size_rt(rt: *mut JSRuntime, p: *const c_void) -> usize {
    __js_malloc_usable_size(&mut (*rt).malloc_ctx, p.cast())
}
pub unsafe fn js_mallocz_rt(rt: *mut JSRuntime, size: usize) -> *mut c_void {
    let p = js_malloc_rt(rt, size);
    if !p.is_null() {
        ptr::write_bytes(p.cast::<u8>(), 0, size);
    }
    p
}
pub unsafe fn JS_GetRuntimeOpaque(rt: *mut JSRuntime) -> *mut c_void {
    (*rt).user_opaque
}
pub unsafe fn JS_SetRuntimeOpaque(rt: *mut JSRuntime, opaque: *mut c_void) {
    (*rt).user_opaque = opaque;
}
pub unsafe fn JS_GetContextOpaque(ctx: *mut JSContext) -> *mut c_void {
    (*ctx).user_opaque
}
pub unsafe fn JS_SetContextOpaque(ctx: *mut JSContext, opaque: *mut c_void) {
    (*ctx).user_opaque = opaque;
}
pub unsafe fn JS_GetRuntime(ctx: *mut JSContext) -> *mut JSRuntime {
    (*ctx).rt
}
pub unsafe fn JS_SetRuntimeInfo(rt: *mut JSRuntime, s: *const c_char) {
    if !rt.is_null() {
        (*rt).rt_info = s;
    }
}
pub unsafe fn JS_SetMemoryLimit(rt: *mut JSRuntime, limit: usize) {
    (*rt).malloc_ctx.malloc_state.malloc_limit = limit;
}
pub unsafe fn JS_SetGCThreshold(rt: *mut JSRuntime, threshold: usize) {
    (*rt).malloc_gc_threshold = threshold;
}
pub unsafe fn JS_SetInterruptHandler(
    rt: *mut JSRuntime,
    cb: Option<JSInterruptHandler>,
    opaque: *mut c_void,
) {
    (*rt).interrupt_handler = cb;
    (*rt).interrupt_opaque = opaque;
}
pub unsafe fn JS_SetCanBlock(rt: *mut JSRuntime, can_block: JS_BOOL) {
    (*rt).can_block = can_block as i8;
}
pub unsafe fn JS_SetSharedArrayBufferFunctions(
    rt: *mut JSRuntime,
    sf: *const JSSharedArrayBufferFunctions,
) {
    ptr::copy_nonoverlapping(sf, &mut (*rt).sab_funcs, 1);
}
pub unsafe fn JS_SetStripInfo(rt: *mut JSRuntime, flags: i32) {
    (*rt).strip_flags = flags as u8;
}
pub unsafe fn JS_GetStripInfo(rt: *mut JSRuntime) -> i32 {
    (*rt).strip_flags as i32
}
unsafe fn update_stack_limit(rt: *mut JSRuntime) {
    (*rt).stack_limit = if (*rt).stack_size == 0 {
        0
    } else {
        (*rt).stack_top.wrapping_sub((*rt).stack_size)
    };
}
pub unsafe fn JS_SetMaxStackSize(rt: *mut JSRuntime, size: usize) {
    (*rt).stack_size = size;
    update_stack_limit(rt);
}
unsafe fn is_strict_mode(ctx: *mut JSContext) -> JS_BOOL {
    let sf = (*(*ctx).rt).current_stack_frame;
    (!sf.is_null() && (*sf).js_mode & JS_MODE_STRICT != 0) as i32
}
unsafe fn atom_get_free(p: *const JSAtomStruct) -> u32 {
    (p as usize >> 1) as u32
}
fn atom_is_free(p: *const JSAtomStruct) -> i32 {
    (p as usize & 1) as i32
}
fn atom_set_free(v: u32) -> *mut JSAtomStruct {
    (((v as usize) << 1) | 1) as *mut JSAtomStruct
}
unsafe fn js_alloc_string_rt(rt: *mut JSRuntime, max_len: i32, is_wide_char: i32) -> *mut JSString {
    let p = js_malloc_rt(
        rt,
        size_of::<JSString>() + ((max_len as usize) << is_wide_char) + 1 - is_wide_char as usize,
    )
    .cast::<JSString>();
    if p.is_null() {
        return ptr::null_mut();
    }
    (*js_rc(p.cast())).ref_count = 1;
    (*p).len_and_wide = ((is_wide_char as u32) << 31) | (max_len as u32 & 0x7fffffff);
    (*p).hash_and_type = 0;
    (*p).hash_next = 0;
    p
}
unsafe fn js_free_string(rt: *mut JSRuntime, str: *mut JSString) {
    let rc = js_rc(str.cast());
    (*rc).ref_count -= 1;
    if (*rc).ref_count <= 0 {
        if (*str).atom_type() != 0 {
            JS_FreeAtomStruct(rt, str);
        } else {
            js_free_rt(rt, str.cast());
        }
    }
}
const JS_ATOM_TAG_INT: u32 = 1 << 31;
const JS_ATOM_MAX_INT: u32 = JS_ATOM_TAG_INT - 1;
const JS_ATOM_MAX: u32 = (1 << 30) - 1;
fn __JS_AtomIsConst(v: JSAtom) -> i32 {
    ((v as i32) < JS_ATOM_END as i32) as i32
}
fn __JS_AtomIsTaggedInt(v: JSAtom) -> i32 {
    (v & JS_ATOM_TAG_INT != 0) as i32
}
fn __JS_AtomFromUInt32(v: u32) -> JSAtom {
    v | JS_ATOM_TAG_INT
}
fn __JS_AtomToUInt32(v: JSAtom) -> u32 {
    v & !JS_ATOM_TAG_INT
}
fn is_num(c: i32) -> bool {
    c >= b'0' as i32 && c <= b'9' as i32
}
unsafe fn string_get(p: *const JSString, idx: i32) -> i32 {
    if (*p).is_wide_char() != 0 {
        *string_data16(p.cast_mut()).add(idx as usize) as i32
    } else {
        *string_data8(p.cast_mut()).add(idx as usize) as i32
    }
}
unsafe fn is_num_string(pval: *mut u32, p: *const JSString) -> i32 {
    let len = (*p).len();
    if len == 0 || len > 10 {
        return 0;
    }
    let c = string_get(p, 0);
    if !is_num(c) {
        return 0;
    }
    let mut n;
    if c == b'0' as i32 {
        if len != 1 {
            return 0;
        }
        n = 0;
    } else {
        n = (c - b'0' as i32) as u32;
        for i in 1..len {
            let c = string_get(p, i as i32);
            if !is_num(c) {
                return 0;
            }
            let n64 = n as u64 * 10 + (c - b'0' as i32) as u64;
            if n64 >> 32 != 0 {
                return 0;
            }
            n = n64 as u32;
        }
    }
    *pval = n;
    1
}
unsafe fn hash_string8(str: *const u8, len: usize, mut h: u32) -> u32 {
    for i in 0..len {
        h = h.wrapping_mul(263).wrapping_add(*str.add(i) as u32);
    }
    h
}
unsafe fn hash_string16(str: *const u16, len: usize, mut h: u32) -> u32 {
    for i in 0..len {
        h = h.wrapping_mul(263).wrapping_add(*str.add(i) as u32);
    }
    h
}
unsafe fn hash_string(str: *const JSString, h: u32) -> u32 {
    if (*str).is_wide_char() != 0 {
        hash_string16(string_data16(str.cast_mut()), (*str).len() as usize, h)
    } else {
        hash_string8(string_data8(str.cast_mut()), (*str).len() as usize, h)
    }
}
unsafe fn hash_string_rope(val: JSValueConst, h: u32) -> u32 {
    if JS_VALUE_GET_TAG(val) == JS_TAG_STRING {
        hash_string(JS_VALUE_GET_PTR(val).cast(), h)
    } else {
        let r = JS_VALUE_GET_PTR(val).cast::<JSStringRope>();
        let h = hash_string_rope((*r).left, h);
        hash_string_rope((*r).right, h)
    }
}
unsafe fn memcmp16_8(src1: *const u16, src2: *const u8, len: i32) -> i32 {
    for i in 0..len {
        let c = *src1.add(i as usize) as i32 - *src2.add(i as usize) as i32;
        if c != 0 {
            return c;
        }
    }
    0
}
unsafe fn memcmp16(src1: *const u16, src2: *const u16, len: i32) -> i32 {
    for i in 0..len {
        let c = *src1.add(i as usize) as i32 - *src2.add(i as usize) as i32;
        if c != 0 {
            return c;
        }
    }
    0
}
unsafe fn js_string_memcmp(
    p1: *const JSString,
    pos1: i32,
    p2: *const JSString,
    pos2: i32,
    len: i32,
) -> i32 {
    if (*p1).is_wide_char() == 0 {
        if (*p2).is_wide_char() == 0 {
            for i in 0..len {
                let c = *string_data8(p1.cast_mut()).add((pos1 + i) as usize) as i32
                    - *string_data8(p2.cast_mut()).add((pos2 + i) as usize) as i32;
                if c != 0 {
                    return c;
                }
            }
            0
        } else {
            -memcmp16_8(
                string_data16(p2.cast_mut()).add(pos2 as usize),
                string_data8(p1.cast_mut()).add(pos1 as usize),
                len,
            )
        }
    } else if (*p2).is_wide_char() == 0 {
        memcmp16_8(
            string_data16(p1.cast_mut()).add(pos1 as usize),
            string_data8(p2.cast_mut()).add(pos2 as usize),
            len,
        )
    } else {
        memcmp16(
            string_data16(p1.cast_mut()).add(pos1 as usize),
            string_data16(p2.cast_mut()).add(pos2 as usize),
            len,
        )
    }
}
unsafe fn js_string_eq(_ctx: *mut JSContext, p1: *const JSString, p2: *const JSString) -> i32 {
    if (*p1).len() != (*p2).len() {
        return 0;
    }
    if p1 == p2 {
        return 1;
    }
    (js_string_memcmp(p1, 0, p2, 0, (*p1).len() as i32) == 0) as i32
}
unsafe fn js_string_compare(_ctx: *mut JSContext, p1: *const JSString, p2: *const JSString) -> i32 {
    let len = (*p1).len().min((*p2).len());
    let res = js_string_memcmp(p1, 0, p2, 0, len as i32);
    if res != 0 {
        res
    } else {
        ((*p1).len() > (*p2).len()) as i32 - ((*p1).len() < (*p2).len()) as i32
    }
}

unsafe fn JS_ResizeAtomHash(rt: *mut JSRuntime, new_hash_size: i32) -> i32 {
    assert!(new_hash_size & (new_hash_size - 1) == 0);
    let mask = new_hash_size as u32 - 1;
    let new_hash = js_mallocz_rt(rt, size_of::<u32>() * new_hash_size as usize).cast::<u32>();
    if new_hash.is_null() {
        return -1;
    }
    for i in 0..(*rt).atom_hash_size {
        let mut h = *(*rt).atom_hash.add(i as usize);
        while h != 0 {
            let p = *(*rt).atom_array.add(h as usize);
            let next = (*p).hash_next;
            let j = (*p).hash() & mask;
            (*p).hash_next = *new_hash.add(j as usize);
            *new_hash.add(j as usize) = h;
            h = next;
        }
    }
    js_free_rt(rt, (*rt).atom_hash.cast());
    (*rt).atom_hash = new_hash;
    (*rt).atom_hash_size = new_hash_size;
    (*rt).atom_count_resize = new_hash_size * 2;
    0
}
unsafe fn JS_InitAtoms(rt: *mut JSRuntime) -> i32 {
    (*rt).atom_hash_size = 0;
    (*rt).atom_hash = ptr::null_mut();
    (*rt).atom_count = 0;
    (*rt).atom_size = 0;
    (*rt).atom_free_index = 0;
    if JS_ResizeAtomHash(rt, 512) != 0 {
        return -1;
    }
    let mut p = js_atom_init.as_ptr();
    for i in 1..JS_ATOM_END {
        let atom_type = if i == JS_ATOM_Private_brand {
            JS_ATOM_TYPE_PRIVATE
        } else if i >= JS_ATOM_Symbol_toPrimitive {
            JS_ATOM_TYPE_SYMBOL
        } else {
            JS_ATOM_TYPE_STRING
        };
        let len = core::ffi::CStr::from_ptr(p.cast()).to_bytes().len();
        if __JS_NewAtomInit(rt, p.cast(), len as i32, atom_type) == JS_ATOM_NULL as u32 {
            return -1;
        }
        p = p.add(len + 1);
    }
    0
}
unsafe fn JS_DupAtomRT(rt: *mut JSRuntime, v: JSAtom) -> JSAtom {
    if __JS_AtomIsConst(v) == 0 {
        let p = *(*rt).atom_array.add(v as usize);
        (*js_rc(p.cast())).ref_count += 1;
    }
    v
}
pub unsafe fn JS_DupAtom(ctx: *mut JSContext, v: JSAtom) -> JSAtom {
    if __JS_AtomIsConst(v) == 0 {
        let p = *(*(*ctx).rt).atom_array.add(v as usize);
        (*js_rc(p.cast())).ref_count += 1;
    }
    v
}
unsafe fn JS_AtomGetKind(ctx: *mut JSContext, v: JSAtom) -> JSAtomKindEnum {
    if __JS_AtomIsTaggedInt(v) != 0 {
        return JS_ATOM_KIND_STRING;
    }
    let p = *(*(*ctx).rt).atom_array.add(v as usize);
    match (*p).atom_type() {
        JS_ATOM_TYPE_STRING => JS_ATOM_KIND_STRING,
        JS_ATOM_TYPE_GLOBAL_SYMBOL => JS_ATOM_KIND_SYMBOL,
        JS_ATOM_TYPE_SYMBOL => {
            if (*p).hash() == JS_ATOM_HASH_PRIVATE {
                JS_ATOM_KIND_PRIVATE
            } else {
                JS_ATOM_KIND_SYMBOL
            }
        }
        _ => std::process::abort(),
    }
}
unsafe fn JS_AtomIsString(ctx: *mut JSContext, v: JSAtom) -> i32 {
    (JS_AtomGetKind(ctx, v) == JS_ATOM_KIND_STRING) as i32
}
unsafe fn js_get_atom_index(rt: *mut JSRuntime, p: *mut JSAtomStruct) -> JSAtom {
    let mut i = (*p).hash_next;
    if (*p).atom_type() != JS_ATOM_TYPE_SYMBOL {
        i = *(*rt)
            .atom_hash
            .add(((*p).hash() & ((*rt).atom_hash_size as u32 - 1)) as usize);
        let mut p1 = *(*rt).atom_array.add(i as usize);
        while p1 != p {
            assert!(i != 0);
            i = (*p1).hash_next;
            p1 = *(*rt).atom_array.add(i as usize);
        }
    }
    i
}
unsafe fn __JS_NewAtom(rt: *mut JSRuntime, str: *mut JSString, mut atom_type: u32) -> JSAtom {
    let (h, h1);
    if atom_type < JS_ATOM_TYPE_SYMBOL {
        if (*str).atom_type() == atom_type {
            let i = js_get_atom_index(rt, str);
            if __JS_AtomIsConst(i) != 0 {
                (*js_rc(str.cast())).ref_count -= 1;
            }
            return i;
        }
        let len = (*str).len();
        h = hash_string(str, atom_type) & JS_ATOM_HASH_MASK;
        h1 = h & ((*rt).atom_hash_size as u32 - 1);
        let mut i = *(*rt).atom_hash.add(h1 as usize);
        while i != 0 {
            let p = *(*rt).atom_array.add(i as usize);
            if (*p).hash() == h
                && (*p).atom_type() == atom_type
                && (*p).len() == len
                && js_string_memcmp(p, 0, str, 0, len as i32) == 0
            {
                if __JS_AtomIsConst(i) == 0 {
                    (*js_rc(p.cast())).ref_count += 1;
                }
                js_free_string(rt, str);
                return i;
            }
            i = (*p).hash_next;
        }
    } else {
        h1 = 0;
        if atom_type == JS_ATOM_TYPE_SYMBOL {
            h = 0;
        } else {
            h = JS_ATOM_HASH_PRIVATE;
            atom_type = JS_ATOM_TYPE_SYMBOL;
        }
    }
    let result = (|| {
        if (*rt).atom_free_index == 0 {
            let new_size = 711i32.max((*rt).atom_size * 3 / 2) as u32;
            if new_size > JS_ATOM_MAX {
                return JS_ATOM_NULL as u32;
            }
            let new_array = js_realloc_rt(
                rt,
                (*rt).atom_array.cast(),
                size_of::<*mut JSAtomStruct>() * new_size as usize,
            )
            .cast::<*mut JSAtomStruct>();
            if new_array.is_null() {
                return JS_ATOM_NULL as u32;
            }
            let mut start = (*rt).atom_size as u32;
            if start == 0 {
                let p = js_mallocz_rt(rt, size_of::<JSAtomStruct>()).cast::<JSAtomStruct>();
                if p.is_null() {
                    js_free_rt(rt, new_array.cast());
                    return JS_ATOM_NULL as u32;
                }
                (*js_rc(p.cast())).ref_count = 1;
                (*p).set_atom_type(JS_ATOM_TYPE_SYMBOL);
                *new_array = p;
                (*rt).atom_count += 1;
                start = 1;
            }
            (*rt).atom_size = new_size as i32;
            (*rt).atom_array = new_array;
            (*rt).atom_free_index = start as i32;
            for i in start..new_size {
                *(*rt).atom_array.add(i as usize) =
                    atom_set_free(if i == new_size - 1 { 0 } else { i + 1 });
            }
        }
        let p;
        if !str.is_null() {
            if (*str).atom_type() == 0 {
                p = str;
                (*p).set_atom_type(atom_type);
            } else {
                let len = (*str).len();
                let wide = (*str).is_wide_char();
                let bytes = ((len as usize) << wide) + 1 - wide as usize;
                p = js_malloc_rt(rt, size_of::<JSString>() + bytes).cast::<JSString>();
                if p.is_null() {
                    return JS_ATOM_NULL as u32;
                }
                (*js_rc(p.cast())).ref_count = 1;
                (*p).len_and_wide = (*str).len_and_wide;
                (*p).hash_and_type = 0;
                ptr::copy_nonoverlapping(string_data8(str), string_data8(p), bytes);
                js_free_string(rt, str);
            }
        } else {
            p = js_malloc_rt(rt, size_of::<JSAtomStruct>()).cast::<JSAtomStruct>();
            if p.is_null() {
                return JS_ATOM_NULL as u32;
            }
            (*js_rc(p.cast())).ref_count = 1;
            (*p).len_and_wide = 1 << 31;
            (*p).hash_and_type = 0;
        }
        let i = (*rt).atom_free_index as u32;
        (*rt).atom_free_index = atom_get_free(*(*rt).atom_array.add(i as usize)) as i32;
        *(*rt).atom_array.add(i as usize) = p;
        (*p).set_hash(h);
        (*p).hash_next = i;
        (*p).set_atom_type(atom_type);
        (*rt).atom_count += 1;
        if atom_type != JS_ATOM_TYPE_SYMBOL {
            (*p).hash_next = *(*rt).atom_hash.add(h1 as usize);
            *(*rt).atom_hash.add(h1 as usize) = i;
            if (*rt).atom_count >= (*rt).atom_count_resize {
                JS_ResizeAtomHash(rt, (*rt).atom_hash_size * 2);
            }
        }
        i
    })();
    // C's fail/done labels release str only on failure; on success ownership has
    // either been transferred or explicitly released in the copying branch.
    if result == JS_ATOM_NULL as u32 && !str.is_null() {
        js_free_string(rt, str);
    }
    result
}
unsafe fn __JS_NewAtomInit(
    rt: *mut JSRuntime,
    str: *const c_char,
    len: i32,
    atom_type: u32,
) -> JSAtom {
    let p = js_alloc_string_rt(rt, len, 0);
    if p.is_null() {
        return JS_ATOM_NULL as u32;
    }
    ptr::copy_nonoverlapping(str.cast::<u8>(), string_data8(p), len as usize);
    *string_data8(p).add(len as usize) = 0;
    __JS_NewAtom(rt, p, atom_type)
}
unsafe fn __JS_FindAtom(
    rt: *mut JSRuntime,
    str: *const c_char,
    len: usize,
    _atom_type: u32,
) -> JSAtom {
    let h = hash_string8(str.cast(), len, JS_ATOM_TYPE_STRING) & JS_ATOM_HASH_MASK;
    let h1 = h & ((*rt).atom_hash_size as u32 - 1);
    let mut i = *(*rt).atom_hash.add(h1 as usize);
    while i != 0 {
        let p = *(*rt).atom_array.add(i as usize);
        if (*p).hash() == h
            && (*p).atom_type() == JS_ATOM_TYPE_STRING
            && (*p).len() as usize == len
            && (*p).is_wide_char() == 0
            && core::slice::from_raw_parts(string_data8(p), len)
                == core::slice::from_raw_parts(str.cast::<u8>(), len)
        {
            if __JS_AtomIsConst(i) == 0 {
                (*js_rc(p.cast())).ref_count += 1;
            }
            return i;
        }
        i = (*p).hash_next;
    }
    JS_ATOM_NULL as u32
}
unsafe fn JS_FreeAtomStruct(rt: *mut JSRuntime, p: *mut JSAtomStruct) {
    let mut i = (*p).hash_next;
    if (*p).atom_type() != JS_ATOM_TYPE_SYMBOL {
        let h0 = (*p).hash() & ((*rt).atom_hash_size as u32 - 1);
        i = *(*rt).atom_hash.add(h0 as usize);
        let mut p1 = *(*rt).atom_array.add(i as usize);
        if p1 == p {
            *(*rt).atom_hash.add(h0 as usize) = (*p1).hash_next;
        } else {
            loop {
                assert!(i != 0);
                let p0 = p1;
                i = (*p1).hash_next;
                p1 = *(*rt).atom_array.add(i as usize);
                if p1 == p {
                    (*p0).hash_next = (*p1).hash_next;
                    break;
                }
            }
        }
    }
    *(*rt).atom_array.add(i as usize) = atom_set_free((*rt).atom_free_index as u32);
    (*rt).atom_free_index = i as i32;
    if !((*p).atom_type() == JS_ATOM_TYPE_SYMBOL
        && (*p).hash() != JS_ATOM_HASH_PRIVATE
        && (*p).hash() != 0)
    {
        js_free_rt(rt, p.cast());
    }
    (*rt).atom_count -= 1;
    assert!((*rt).atom_count >= 0);
}
unsafe fn __JS_FreeAtom(rt: *mut JSRuntime, i: u32) {
    let p = *(*rt).atom_array.add(i as usize);
    let rc = js_rc(p.cast());
    (*rc).ref_count -= 1;
    if (*rc).ref_count > 0 {
        return;
    }
    JS_FreeAtomStruct(rt, p);
}
unsafe fn JS_NewAtomStr(ctx: *mut JSContext, p: *mut JSString) -> JSAtom {
    let rt = (*ctx).rt;
    let mut n = 0;
    if is_num_string(&mut n, p) != 0 && n <= JS_ATOM_MAX_INT {
        js_free_string(rt, p);
        return __JS_AtomFromUInt32(n);
    }
    __JS_NewAtom(rt, p, JS_ATOM_TYPE_STRING)
}
pub unsafe fn JS_FreeAtom(ctx: *mut JSContext, v: JSAtom) {
    if __JS_AtomIsConst(v) == 0 {
        __JS_FreeAtom((*ctx).rt, v);
    }
}
pub unsafe fn JS_FreeAtomRT(rt: *mut JSRuntime, v: JSAtom) {
    if __JS_AtomIsConst(v) == 0 {
        __JS_FreeAtom(rt, v);
    }
}
unsafe fn JS_AtomSymbolHasDescription(ctx: *mut JSContext, v: JSAtom) -> i32 {
    if __JS_AtomIsTaggedInt(v) != 0 {
        return 0;
    }
    let p = *(*(*ctx).rt).atom_array.add(v as usize);
    ((((*p).atom_type() == JS_ATOM_TYPE_SYMBOL && (*p).hash() != JS_ATOM_HASH_PRIVATE)
        || (*p).atom_type() == JS_ATOM_TYPE_GLOBAL_SYMBOL)
        && !((*p).len() == 0 && (*p).is_wide_char() != 0)) as i32
}
unsafe fn count_ascii(buf: *const u8, len: usize) -> usize {
    let mut n = 0;
    while n < len && *buf.add(n) < 128 {
        n += 1;
    }
    n
}
unsafe fn copy_str16(dst: *mut u16, p: *const JSString, offset: i32, len: i32) {
    if (*p).is_wide_char() != 0 {
        ptr::copy_nonoverlapping(
            string_data16(p.cast_mut()).add(offset as usize),
            dst,
            len as usize,
        );
    } else {
        let src = string_data8(p.cast_mut()).add(offset as usize);
        for i in 0..len as usize {
            *dst.add(i) = *src.add(i) as u16;
        }
    }
}
