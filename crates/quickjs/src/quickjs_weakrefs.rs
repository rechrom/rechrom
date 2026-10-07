// quickjs.c weak reference ownership and map storage destruction. MIT.
// FinalizationRegistry scheduling is added with the original JS call/job path.
unsafe fn js_weakref_is_target(val: JSValueConst) -> JS_BOOL {
    match JS_VALUE_GET_TAG(val) {
        JS_TAG_OBJECT => 1,
        JS_TAG_SYMBOL => {
            let p = JS_VALUE_GET_PTR(val).cast::<JSAtomStruct>();
            ((*p).atom_type() == JS_ATOM_TYPE_SYMBOL && (*p).hash() != JS_ATOM_HASH_PRIVATE) as i32
        }
        _ => 0,
    }
}
unsafe fn js_weakref_is_live(val: JSValueConst) -> JS_BOOL {
    if JS_IsUndefined(val) != 0 {
        return 1;
    }
    ((*js_rc(JS_VALUE_GET_PTR(val))).ref_count != 0) as i32
}
unsafe fn js_weakref_free(rt: *mut JSRuntime, val: JSValue) {
    match JS_VALUE_GET_TAG(val) {
        JS_TAG_OBJECT => {
            let p = JS_VALUE_GET_PTR(val).cast::<JSObject>();
            assert!((*p).weakref_count >= 1);
            (*p).weakref_count -= 1;
            let rc = js_rc(p.cast());
            if (*p).weakref_count == 0
                && (*rc).ref_count == 0
                && (*rc).gc_obj_type_and_mark & 0x80 == 0
            {
                js_free_rt(rt, p.cast());
            }
        }
        JS_TAG_SYMBOL => {
            let p = JS_VALUE_GET_PTR(val).cast::<JSString>();
            assert!((*p).hash() >= 1);
            (*p).set_hash((*p).hash() - 1);
            if (*p).hash() == 0 && (*js_rc(p.cast())).ref_count == 0 {
                js_free_rt(rt, p.cast());
            }
        }
        _ => {}
    }
}
unsafe fn js_weakref_new(_ctx: *mut JSContext, val: JSValueConst) -> JSValue {
    match JS_VALUE_GET_TAG(val) {
        JS_TAG_OBJECT => {
            let p = JS_VALUE_GET_PTR(val).cast::<JSObject>();
            (*p).weakref_count = (*p).weakref_count.wrapping_add(1);
        }
        JS_TAG_SYMBOL => {
            let p = JS_VALUE_GET_PTR(val).cast::<JSString>();
            assert!((*p).hash() < JS_ATOM_HASH_MASK - 2);
            (*p).set_hash((*p).hash() + 1);
        }
        _ => assert!(JS_IsUndefined(val) != 0),
    }
    val
}
const MAGIC_SET: i32 = 1 << 0;
const MAGIC_WEAK: i32 = 1 << 1;
fn map_normalize_key(ctx: *mut JSContext, mut key: JSValue) -> JSValue {
    if JS_TAG_IS_FLOAT64(JS_VALUE_GET_TAG(key)) != 0 && JS_VALUE_GET_FLOAT64(key) == 0.0 {
        key = JS_NewInt32(ctx, 0);
    }
    key
}
fn map_normalize_key_const(ctx: *mut JSContext, key: JSValueConst) -> JSValueConst {
    map_normalize_key(ctx, key)
}
const HASH_MUL32: u32 = 0x61c88647;
const HASH_MUL64: u64 = 0x61c8864680b583eb;
fn map_hash32(a: u32, hash_bits: i32) -> u32 {
    a.wrapping_mul(HASH_MUL32) >> (32 - hash_bits)
}
fn map_hash64(a: u64, hash_bits: i32) -> u32 {
    (a.wrapping_mul(HASH_MUL64) >> (64 - hash_bits)) as u32
}
fn map_hash_pointer(a: usize, hash_bits: i32) -> u32 {
    #[cfg(target_pointer_width = "64")]
    {
        map_hash64(a as u64, hash_bits)
    }
    #[cfg(target_pointer_width = "32")]
    {
        map_hash32(a as u32, hash_bits)
    }
}
unsafe fn map_hash_key(key: JSValueConst, hash_bits: i32) -> u32 {
    let tag = JS_VALUE_GET_NORM_TAG(key);
    match tag {
        JS_TAG_BOOL => map_hash32(JS_VALUE_GET_INT(key) as u32 ^ JS_TAG_BOOL as u32, hash_bits),
        JS_TAG_STRING => map_hash32(
            hash_string(JS_VALUE_GET_PTR(key).cast(), 0) ^ JS_TAG_STRING as u32,
            hash_bits,
        ),
        JS_TAG_STRING_ROPE => {
            map_hash32(hash_string_rope(key, 0) ^ JS_TAG_STRING as u32, hash_bits)
        }
        JS_TAG_OBJECT | JS_TAG_SYMBOL => map_hash_pointer(
            JS_VALUE_GET_PTR(key) as usize ^ tag as u32 as usize,
            hash_bits,
        ),
        JS_TAG_INT | JS_TAG_FLOAT64 => {
            let mut d = if tag == JS_TAG_INT {
                JS_VALUE_GET_INT(key) as f64
            } else {
                JS_VALUE_GET_FLOAT64(key)
            };
            if d.is_nan() {
                d = JS_FLOAT64_NAN;
            }
            map_hash64(d.to_bits() ^ JS_TAG_FLOAT64 as u64, hash_bits)
        }
        JS_TAG_SHORT_BIG_INT | JS_TAG_BIG_INT => {
            let mut buf: JSBigIntBuf = core::mem::zeroed();
            let p = if tag == JS_TAG_SHORT_BIG_INT {
                js_bigint_set_short(&mut buf, key)
            } else {
                JS_VALUE_GET_PTR(key).cast()
            };
            let mut h = 1u32;
            for i in (0..(*p).len as usize).rev() {
                h = h
                    .wrapping_mul(263)
                    .wrapping_add(*ptr::addr_of!((*p).tab).cast::<js_limb_t>().add(i) as u32);
            }
            map_hash32(h ^ JS_TAG_BIG_INT as u32, hash_bits)
        }
        _ => 0,
    }
}
unsafe fn map_delete_record_internal(rt: *mut JSRuntime, s: *mut JSMapState, mr: *mut JSMapRecord) {
    if (*mr).empty != 0 {
        return;
    }
    if (*s).is_weak != 0 {
        js_weakref_free(rt, (*mr).key);
    } else {
        JS_FreeValueRT(rt, (*mr).key);
    }
    JS_FreeValueRT(rt, (*mr).value);
    (*mr).ref_count -= 1;
    if (*mr).ref_count == 0 {
        list_del(&mut (*mr).link);
        js_free_rt(rt, mr.cast());
    } else {
        (*mr).empty = 1;
        (*mr).key = JS_UNDEFINED;
        (*mr).value = JS_UNDEFINED;
    }
    (*s).record_count -= 1;
}
unsafe fn map_decref_record(rt: *mut JSRuntime, mr: *mut JSMapRecord) {
    (*mr).ref_count -= 1;
    if (*mr).ref_count == 0 {
        assert!((*mr).empty != 0);
        list_del(&mut (*mr).link);
        js_free_rt(rt, mr.cast());
    }
}
unsafe fn map_delete_weakrefs(rt: *mut JSRuntime, wh: *mut JSWeakRefHeader) {
    let s = wh
        .cast::<u8>()
        .sub(offset_of!(JSMapState, weakref_header))
        .cast::<JSMapState>();
    for el in ListIter::new(&mut (*s).records, false, true) {
        let mr = el
            .cast::<u8>()
            .sub(offset_of!(JSMapRecord, link))
            .cast::<JSMapRecord>();
        if js_weakref_is_live((*mr).key) == 0 {
            let h = map_hash_key((*mr).key, (*s).hash_bits);
            let mut pmr = (*s).hash_table.add(h as usize);
            loop {
                let mr1 = *pmr;
                if mr1.is_null() {
                    break;
                }
                if mr1 == mr {
                    *pmr = (*mr1).hash_next;
                    break;
                }
                pmr = ptr::addr_of_mut!((*mr1).hash_next);
            }
            map_delete_record_internal(rt, s, mr);
        }
    }
}
unsafe fn js_map_finalizer(rt: *mut JSRuntime, val: JSValue) {
    let s = (*JS_VALUE_GET_PTR(val).cast::<JSObject>()).u.map_state;
    if !s.is_null() {
        for el in ListIter::new(&mut (*s).records, false, true) {
            let mr = el
                .cast::<u8>()
                .sub(offset_of!(JSMapRecord, link))
                .cast::<JSMapRecord>();
            if (*mr).empty == 0 {
                if (*s).is_weak != 0 {
                    js_weakref_free(rt, (*mr).key);
                } else {
                    JS_FreeValueRT(rt, (*mr).key);
                }
                JS_FreeValueRT(rt, (*mr).value);
            }
            js_free_rt(rt, mr.cast());
        }
        js_free_rt(rt, (*s).hash_table.cast());
        if (*s).is_weak != 0 {
            list_del(&mut (*s).weakref_header.link);
        }
        js_free_rt(rt, s.cast());
    }
}
unsafe fn js_map_mark(rt: *mut JSRuntime, val: JSValueConst, mark_func: Option<JS_MarkFunc>) {
    let s = (*JS_VALUE_GET_PTR(val).cast::<JSObject>()).u.map_state;
    if !s.is_null() {
        for el in ListIter::new(&mut (*s).records, false, false) {
            let mr = el
                .cast::<u8>()
                .sub(offset_of!(JSMapRecord, link))
                .cast::<JSMapRecord>();
            if (*s).is_weak == 0 {
                JS_MarkValue(rt, (*mr).key, mark_func.unwrap());
            }
            JS_MarkValue(rt, (*mr).value, mark_func.unwrap());
        }
    }
}
#[repr(C)]
struct JSWeakRefData {
    weakref_header: JSWeakRefHeader,
    target: JSValue,
}
unsafe fn js_weakref_finalizer(rt: *mut JSRuntime, val: JSValue) {
    let wrd = JS_GetOpaque(val, JS_CLASS_WEAK_REF).cast::<JSWeakRefData>();
    if wrd.is_null() {
        return;
    }
    js_weakref_free(rt, (*wrd).target);
    list_del(&mut (*wrd).weakref_header.link);
    js_free_rt(rt, wrd.cast());
}
unsafe fn weakref_delete_weakref(rt: *mut JSRuntime, wh: *mut JSWeakRefHeader) {
    let wrd = wh
        .cast::<u8>()
        .sub(offset_of!(JSWeakRefData, weakref_header))
        .cast::<JSWeakRefData>();
    if js_weakref_is_live((*wrd).target) == 0 {
        js_weakref_free(rt, (*wrd).target);
        (*wrd).target = JS_UNDEFINED;
    }
}
#[repr(C)]
struct JSFinRecEntry {
    link: list_head,
    target: JSValue,
    held_val: JSValue,
    token: JSValue,
}
#[repr(C)]
struct JSFinalizationRegistryData {
    weakref_header: JSWeakRefHeader,
    entries: list_head,
    realm: *mut JSContext,
    cb: JSValue,
}
unsafe fn js_finrec_finalizer(rt: *mut JSRuntime, val: JSValue) {
    let frd =
        JS_GetOpaque(val, JS_CLASS_FINALIZATION_REGISTRY).cast::<JSFinalizationRegistryData>();
    if !frd.is_null() {
        for el in ListIter::new(&mut (*frd).entries, false, true) {
            let fre = el
                .cast::<u8>()
                .sub(offset_of!(JSFinRecEntry, link))
                .cast::<JSFinRecEntry>();
            js_weakref_free(rt, (*fre).target);
            js_weakref_free(rt, (*fre).token);
            JS_FreeValueRT(rt, (*fre).held_val);
            js_free_rt(rt, fre.cast());
        }
        JS_FreeValueRT(rt, (*frd).cb);
        JS_FreeContext((*frd).realm);
        list_del(&mut (*frd).weakref_header.link);
        js_free_rt(rt, frd.cast());
    }
}
unsafe fn js_finrec_mark(rt: *mut JSRuntime, val: JSValueConst, mark_func: Option<JS_MarkFunc>) {
    let frd =
        JS_GetOpaque(val, JS_CLASS_FINALIZATION_REGISTRY).cast::<JSFinalizationRegistryData>();
    if !frd.is_null() {
        for el in ListIter::new(&mut (*frd).entries, false, false) {
            let fre = el
                .cast::<u8>()
                .sub(offset_of!(JSFinRecEntry, link))
                .cast::<JSFinRecEntry>();
            JS_MarkValue(rt, (*fre).held_val, mark_func.unwrap());
        }
        JS_MarkValue(rt, (*frd).cb, mark_func.unwrap());
        mark_func.unwrap()(rt, &mut (*(*frd).realm).header);
    }
}
