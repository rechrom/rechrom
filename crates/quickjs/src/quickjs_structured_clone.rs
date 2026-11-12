// Native slot access for the embedding's source ValueSerializer translation.
// No JS globals, property getters, @@toStringTag or iterators are used for brands.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum JSCloneKind {
    Unsupported = 0,
    Object = 1,
    Array = 2,
    Date = 3,
    Boxed = 4,
    Map = 5,
    Set = 6,
    Buffer = 7,
    View = 8,
    RegExp = 9,
    Error = 10,
    BigInt = 11,
}
pub unsafe fn JS_CloneKind(ctx: *mut JSContext, value: JSValueConst) -> JSCloneKind {
    use JSCloneKind::*;
    if JS_IsBigInt(ctx, value) != 0 {
        return BigInt;
    }
    match JS_GetClassID(value) {
        JS_CLASS_OBJECT => Object,
        JS_CLASS_ARRAY => Array,
        JS_CLASS_DATE => Date,
        JS_CLASS_NUMBER | JS_CLASS_BOOLEAN | JS_CLASS_STRING | JS_CLASS_BIG_INT => Boxed,
        JS_CLASS_MAP => Map,
        JS_CLASS_SET => Set,
        JS_CLASS_ARRAY_BUFFER => Buffer,
        JS_CLASS_UINT8C_ARRAY..=JS_CLASS_DATAVIEW => View,
        JS_CLASS_REGEXP => RegExp,
        JS_CLASS_ERROR => Error,
        _ => Unsupported,
    }
}
pub unsafe fn JS_CloneCheckStack(ctx: *mut JSContext) -> i32 {
    if js_check_stack_overflow((*ctx).rt, 0) != 0 {
        JS_ThrowStackOverflow(ctx);
        -1
    } else {
        0
    }
}
pub unsafe fn JS_CloneObjectData(ctx: *mut JSContext, value: JSValueConst) -> JSValue {
    JS_DupValue(
        ctx,
        (*JS_VALUE_GET_PTR(value).cast::<JSObject>()).u.object_data,
    )
}
pub unsafe fn JS_CloneMapEntries(ctx: *mut JSContext, value: JSValueConst) -> Vec<JSValue> {
    let p = JS_VALUE_GET_PTR(value).cast::<JSObject>();
    let state = (*p).u.map_state;
    let head = core::ptr::addr_of_mut!((*state).records);
    let mut link = (*head).next;
    let is_map = (*p).class_id as u32 == JS_CLASS_MAP;
    let mut entries =
        Vec::with_capacity((*state).record_count as usize * if is_map { 2 } else { 1 });
    while link != head {
        let record = link
            .cast::<u8>()
            .sub(core::mem::offset_of!(JSMapRecord, link))
            .cast::<JSMapRecord>();
        if (*record).empty == 0 {
            entries.push(JS_DupValue(ctx, (*record).key));
            if is_map {
                entries.push(JS_DupValue(ctx, (*record).value));
            }
        }
        link = (*link).next;
    }
    entries
}
pub unsafe fn JS_CloneNewMap(ctx: *mut JSContext, is_set: bool) -> JSValue {
    js_map_constructor(ctx, JS_UNDEFINED, 0, core::ptr::null_mut(), is_set as i32)
}
pub unsafe fn JS_CloneMapAdd(
    ctx: *mut JSContext,
    map: JSValueConst,
    key: JSValueConst,
    value: JSValueConst,
    is_set: bool,
) -> JSValue {
    let mut args = [key, value];
    js_map_set(ctx, map, 2, args.as_mut_ptr(), is_set as i32)
}
/// Append to final storage exactly once, without a temporary byte buffer.
pub unsafe fn JS_CloneCopyBufferBytes(value: JSValueConst, output: &mut Vec<u8>) {
    let ab = (*JS_VALUE_GET_PTR(value).cast::<JSObject>()).u.array_buffer;
    if (*ab).byte_length > 0 {
        output.extend_from_slice(std::slice::from_raw_parts(
            (*ab).data,
            (*ab).byte_length as usize,
        ));
    }
}
pub unsafe fn JS_CloneBuffer(value: JSValueConst) -> Option<(u32, Option<u32>)> {
    let ab = (*JS_VALUE_GET_PTR(value).cast::<JSObject>()).u.array_buffer;
    if (*ab).detached != 0 {
        return None;
    }
    Some((
        (*ab).byte_length as u32,
        ((*ab).max_byte_length >= 0).then_some((*ab).max_byte_length as u32),
    ))
}
pub unsafe fn JS_CloneNewBuffer(ctx: *mut JSContext, bytes: &[u8], max: Option<u32>) -> JSValue {
    let mut max_len = max.unwrap_or(0) as u64;
    let result = js_array_buffer_constructor2(
        ctx,
        JS_UNDEFINED,
        bytes.len() as u64,
        if max.is_some() {
            &mut max_len
        } else {
            core::ptr::null_mut()
        },
        JS_CLASS_ARRAY_BUFFER,
    );
    if JS_IsException(result) == 0 && !bytes.is_empty() {
        let ab = (*JS_VALUE_GET_PTR(result).cast::<JSObject>())
            .u
            .array_buffer;
        core::ptr::copy_nonoverlapping(bytes.as_ptr(), (*ab).data, bytes.len());
    }
    result
}
/// Native view kind is the stable QuickJS typed-array ordinal, plus 12 DataView.
/// The embedding maps it to its own versioned wire tags, never class IDs/pointers.
pub unsafe fn JS_CloneView(
    ctx: *mut JSContext,
    value: JSValueConst,
) -> Option<(u8, JSValue, u32, u32, bool)> {
    let p = JS_VALUE_GET_PTR(value).cast::<JSObject>();
    if (*p).class_id as u32 != JS_CLASS_DATAVIEW && typed_array_is_oob(p) != 0 {
        return None;
    }
    let ta = (*p).u.typed_array;
    let buffer = JSValue {
        tag: JS_TAG_OBJECT as i64,
        u: JSValueUnion {
            ptr: (*ta).buffer.cast(),
        },
    };
    if (*(*ta).buffer).u.array_buffer.is_null() || (*(*(*ta).buffer).u.array_buffer).detached != 0 {
        return None;
    }
    let ab = (*(*ta).buffer).u.array_buffer;
    let available = (*ab).byte_length as u32;
    if (*ta).offset > available || ((*ta).track_rab == 0 && (*ta).length > available - (*ta).offset)
    {
        return None;
    }
    let kind = ((*p).class_id as u32 - JS_CLASS_UINT8C_ARRAY) as u8;
    let length = if (*ta).track_rab != 0 {
        let remaining = available - (*ta).offset;
        if kind == 12 {
            remaining
        } else {
            let size = 1u32 << typed_array_size_log2[kind as usize];
            remaining / size * size
        }
    } else {
        (*ta).length
    };
    Some((
        kind,
        JS_DupValue(ctx, buffer),
        (*ta).offset,
        length,
        (*ta).track_rab != 0,
    ))
}
pub unsafe fn JS_CloneNewView(
    ctx: *mut JSContext,
    kind: u8,
    buffer: JSValueConst,
    offset: u32,
    byte_length: u32,
    tracking: bool,
) -> JSValue {
    if kind > 12 {
        return JS_ThrowTypeError(ctx, c"invalid clone view kind".as_ptr());
    }
    let mut args = [buffer, JS_NewUint32(ctx, offset), JS_UNDEFINED];
    let argc = if tracking { 2 } else { 3 };
    if kind == 12 {
        if !tracking {
            args[2] = JS_NewUint32(ctx, byte_length);
        }
        js_dataview_constructor(ctx, JS_UNDEFINED, argc, args.as_mut_ptr())
    } else {
        let size = 1u32 << typed_array_size_log2[kind as usize];
        if byte_length % size != 0 {
            return JS_ThrowTypeError(ctx, c"invalid clone view length".as_ptr());
        }
        if !tracking {
            args[2] = JS_NewUint32(ctx, byte_length / size);
        }
        js_typed_array_constructor(
            ctx,
            JS_UNDEFINED,
            argc,
            args.as_mut_ptr(),
            JS_CLASS_UINT8C_ARRAY as i32 + kind as i32,
        )
    }
}
pub unsafe fn JS_CloneNewBox(ctx: *mut JSContext, value: JSValueConst, is_date: bool) -> JSValue {
    if !is_date {
        return JS_ToObject(ctx, value);
    }
    let class = JS_CLASS_DATE;
    let object = JS_NewObjectClass(ctx, class as i32);
    if JS_IsException(object) == 0 {
        (*JS_VALUE_GET_PTR(object).cast::<JSObject>()).u.object_data = JS_DupValue(ctx, value);
    }
    object
}
pub unsafe fn JS_CloneBigIntFromString(ctx: *mut JSContext, value: JSValueConst) -> JSValue {
    JS_ToBigIntCtorFree(ctx, JS_DupValue(ctx, value))
}
pub unsafe fn JS_CloneRegExp(ctx: *mut JSContext, value: JSValueConst) -> (JSValue, u32) {
    let re = &(*JS_VALUE_GET_PTR(value).cast::<JSObject>()).u.regexp;
    (
        JS_DupValue(
            ctx,
            JSValue {
                tag: JS_TAG_STRING as i64,
                u: JSValueUnion {
                    ptr: re.pattern.cast(),
                },
            },
        ),
        lre_get_flags((*re.bytecode).u.str8.as_ptr()) as u32,
    )
}
pub unsafe fn JS_CloneNewRegExp(
    ctx: *mut JSContext,
    pattern: JSValueConst,
    flags: JSValueConst,
) -> JSValue {
    let object = JS_NewObjectClass(ctx, JS_CLASS_REGEXP as i32);
    if JS_IsException(object) != 0 {
        return object;
    }
    let bytecode = js_compile_regexp(ctx, pattern, flags);
    if JS_IsException(bytecode) != 0 {
        JS_FreeValue(ctx, object);
        return bytecode;
    }
    js_regexp_set_internal(ctx, object, JS_DupValue(ctx, pattern), bytecode)
}
pub unsafe fn JS_CloneNewError(ctx: *mut JSContext, kind: u8) -> JSValue {
    let proto = if kind == 0 {
        *(*ctx).class_proto.add(JS_CLASS_ERROR as usize)
    } else if kind <= 6 {
        (*ctx).error_proto[(kind - 1) as usize]
    } else {
        return JS_ThrowTypeError(ctx, c"invalid clone error kind".as_ptr());
    };
    JS_NewObjectProtoClass(ctx, proto, JS_CLASS_ERROR)
}

// Private key-enumerator mode for ValueSerializer's dense-array skip_indices.
// Other public QuickJS enumeration callers retain their original flags/results.
const JS_CLONE_SKIP_INDICES: i32 = 1 << 30;
pub unsafe fn JS_CloneOwnPropertyNames(
    ctx: *mut JSContext,
    ptab: *mut *mut JSPropertyEnum,
    plen: *mut u32,
    value: JSValueConst,
    skip_indices: bool,
) -> i32 {
    debug_assert!(!skip_indices || JS_GetClassID(value) == JS_CLASS_ARRAY);
    JS_GetOwnPropertyNames(
        ctx,
        ptab,
        plen,
        value,
        JS_GPN_STRING_MASK
            | JS_GPN_ENUM_ONLY
            | if skip_indices {
                JS_CLONE_SKIP_INDICES
            } else {
                0
            },
    )
}
pub unsafe fn JS_CloneArrayIsPacked(value: JSValueConst, length: u32) -> bool {
    let object = JS_VALUE_GET_PTR(value).cast::<JSObject>();
    (*object).fast_array() != 0 && (*object).u.array.count == length
}
