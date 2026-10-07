// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:57196. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn validate_typed_array(mut ctx: *mut JSContext, mut this_val: JSValue) -> i32 {
let mut vm_local_storage = Vec::<u64>::new();
let mut p: *mut JSObject = core::mem::zeroed();
let mut vm_block: usize = 7;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 57206
1 => {
return (0 as i32);
}
// C line 57204
2 => {
return ((1 as i32)).wrapping_neg();
}
// C line 57203
3 => {
let _ = JS_ThrowTypeErrorArrayBufferOOB(ctx);
vm_block = 2; continue;
}
// C line 57202
4 => {
vm_block = if (typed_array_is_oob(p)) != 0 { 3 } else { 1 }; continue;
}
// C line 57201
5 => {
return ((1 as i32)).wrapping_neg();
}
// C line 57200
6 => {
vm_block = if ((!(!(p).is_null()) as i32)) != 0 { 5 } else { 4 }; continue;
}
// C line 57199
7 => {
let _ = { let assigned = get_typed_array(ctx, this_val); p = assigned; assigned };
vm_block = 6; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:57209. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_typed_array_get_length(mut ctx: *mut JSContext, mut this_val: JSValue) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut p: *mut JSObject = core::mem::zeroed();
let mut vm_block: usize = 4;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 57216
1 => {
return JS_NewInt32(ctx, (((((*(p)).u).array).count) as i32));
}
// C line 57215
2 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 57214
3 => {
vm_block = if ((!(!(p).is_null()) as i32)) != 0 { 2 } else { 1 }; continue;
}
// C line 57213
4 => {
let _ = { let assigned = get_typed_array(ctx, this_val); p = assigned; assigned };
vm_block = 3; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:57219. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_typed_array_get_buffer(mut ctx: *mut JSContext, mut this_val: JSValue) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut p: *mut JSObject = core::mem::zeroed();
let mut ta: *mut JSTypedArray = core::mem::zeroed();
let mut vm_block: usize = 5;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 57228
1 => {
return JS_DupValue(ctx, JSValue { u: JSValueUnion { ptr: (((*(ta)).buffer) as *mut c_void) }, tag: (((JS_TAG_OBJECT as i32)) as i64) });
}
// C line 57227
2 => {
let _ = { let assigned = ((*(p)).u).typed_array; ta = assigned; assigned };
vm_block = 1; continue;
}
// C line 57226
3 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 57225
4 => {
vm_block = if ((!(!(p).is_null()) as i32)) != 0 { 3 } else { 2 }; continue;
}
// C line 57224
5 => {
let _ = { let assigned = get_typed_array(ctx, this_val); p = assigned; assigned };
vm_block = 4; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:57231. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_typed_array_get_byteLength(mut ctx: *mut JSContext, mut this_val: JSValue) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut p: *mut JSObject = core::mem::zeroed();
let mut ta: *mut JSTypedArray = core::mem::zeroed();
let mut size_log2: i32 = core::mem::zeroed();
let mut vm_block: usize = 10;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 57247
1 => {
return JS_NewInt64(ctx, ((((((*(p)).u).array).count) as i64)).wrapping_shl((size_log2) as u32));
}
// C line 57246
2 => {
let _ = { let assigned = ((*((typed_array_size_log2).as_ptr()).offset((((((*(p)).class_id) as i32)).wrapping_sub((JS_CLASS_UINT8C_ARRAY as i32))) as isize)) as i32); size_log2 = assigned; assigned };
vm_block = 1; continue;
}
// C line 57245
3 => {
return JS_NewUint32(ctx, (*(ta)).length);
}
// C line 57244
4 => {
vm_block = if ((!(((*(ta)).track_rab) != 0) as i32)) != 0 { 3 } else { 2 }; continue;
}
// C line 57243
5 => {
let _ = { let assigned = ((*(p)).u).typed_array; ta = assigned; assigned };
vm_block = 4; continue;
}
// C line 57242
6 => {
return JS_NewInt32(ctx, (0 as i32));
}
// C line 57241
7 => {
vm_block = if (typed_array_is_oob(p)) != 0 { 6 } else { 5 }; continue;
}
// C line 57240
8 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 57239
9 => {
vm_block = if ((!(!(p).is_null()) as i32)) != 0 { 8 } else { 7 }; continue;
}
// C line 57238
10 => {
let _ = { let assigned = get_typed_array(ctx, this_val); p = assigned; assigned };
vm_block = 9; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:57250. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_typed_array_get_byteOffset(mut ctx: *mut JSContext, mut this_val: JSValue) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut p: *mut JSObject = core::mem::zeroed();
let mut ta: *mut JSTypedArray = core::mem::zeroed();
let mut vm_block: usize = 7;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 57261
1 => {
return JS_NewUint32(ctx, (*(ta)).offset);
}
// C line 57260
2 => {
let _ = { let assigned = ((*(p)).u).typed_array; ta = assigned; assigned };
vm_block = 1; continue;
}
// C line 57259
3 => {
return JS_NewInt32(ctx, (0 as i32));
}
// C line 57258
4 => {
vm_block = if (typed_array_is_oob(p)) != 0 { 3 } else { 2 }; continue;
}
// C line 57257
5 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 57256
6 => {
vm_block = if ((!(!(p).is_null()) as i32)) != 0 { 5 } else { 4 }; continue;
}
// C line 57255
7 => {
let _ = { let assigned = get_typed_array(ctx, this_val); p = assigned; assigned };
vm_block = 6; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:57264. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
pub unsafe fn JS_NewTypedArray(mut ctx: *mut JSContext, mut argc: i32, mut argv: *mut JSValue, mut v_type: JSTypedArrayEnum) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut vm_block: usize = 3;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 57270
1 => {
return js_typed_array_constructor(ctx, JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }, argc, argv, ((((((JS_CLASS_UINT8C_ARRAY as i32)) as u32)).wrapping_add(((v_type) as u32))) as i32));
}
// C line 57268
2 => {
return JS_ThrowRangeError(ctx, c"invalid typed array type".as_ptr());
}
// C line 57267
3 => {
vm_block = if ((((((((((v_type) as u32)) < ((((JS_TYPED_ARRAY_UINT8C as i32)) as u32))) as i32)) != 0) || (((((((v_type) as u32)) > ((((JS_TYPED_ARRAY_FLOAT64 as i32)) as u32))) as i32)) != 0)) as i32)) != 0 { 2 } else { 1 }; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:57277. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
pub unsafe fn JS_GetTypedArrayBuffer(mut ctx: *mut JSContext, mut obj: JSValue, mut pbyte_offset: *mut usize, mut pbyte_length: *mut usize, mut pbytes_per_element: *mut usize) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut p: *mut JSObject = core::mem::zeroed();
let mut ta: *mut JSTypedArray = core::mem::zeroed();
let mut vm_block: usize = 13;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 57297
1 => {
return JS_DupValue(ctx, JSValue { u: JSValueUnion { ptr: (((*(ta)).buffer) as *mut c_void) }, tag: (((JS_TAG_OBJECT as i32)) as i64) });
}
// C line 57295
2 => {
let _ = { let assigned = ((((1 as i32)).wrapping_shl((((*((typed_array_size_log2).as_ptr()).offset((((((*(p)).class_id) as i32)).wrapping_sub((JS_CLASS_UINT8C_ARRAY as i32))) as isize)) as i32)) as u32)) as usize); *(pbytes_per_element) = assigned; assigned };
vm_block = 1; continue;
}
// C line 57294
3 => {
vm_block = if !(pbytes_per_element).is_null() { 2 } else { 1 }; continue;
}
// C line 57293
4 => {
let _ = { let assigned = (((*(ta)).length) as usize); *(pbyte_length) = assigned; assigned };
vm_block = 3; continue;
}
// C line 57292
5 => {
vm_block = if !(pbyte_length).is_null() { 4 } else { 3 }; continue;
}
// C line 57291
6 => {
let _ = { let assigned = (((*(ta)).offset) as usize); *(pbyte_offset) = assigned; assigned };
vm_block = 5; continue;
}
// C line 57290
7 => {
vm_block = if !(pbyte_offset).is_null() { 6 } else { 5 }; continue;
}
// C line 57289
8 => {
let _ = { let assigned = ((*(p)).u).typed_array; ta = assigned; assigned };
vm_block = 7; continue;
}
// C line 57288
9 => {
return JS_ThrowTypeErrorArrayBufferOOB(ctx);
}
// C line 57287
10 => {
vm_block = if (typed_array_is_oob(p)) != 0 { 9 } else { 8 }; continue;
}
// C line 57286
11 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 57285
12 => {
vm_block = if ((!(!(p).is_null()) as i32)) != 0 { 11 } else { 10 }; continue;
}
// C line 57284
13 => {
let _ = { let assigned = get_typed_array(ctx, obj); p = assigned; assigned };
vm_block = 12; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:57300. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_typed_array_get_toStringTag(mut ctx: *mut JSContext, mut this_val: JSValue) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut p: *mut JSObject = core::mem::zeroed();
let mut vm_block: usize = 6;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 57310
1 => {
return JS_AtomToString(ctx, (*((*((*(ctx)).rt)).class_array).offset(((*(p)).class_id) as isize)).class_name);
}
// C line 57309
2 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) };
}
// C line 57307
3 => {
vm_block = if ((!((((((((((((*(p)).class_id) as i32)) >= ((JS_CLASS_UINT8C_ARRAY as i32))) as i32)) != 0) && ((((((((*(p)).class_id) as i32)) <= ((JS_CLASS_FLOAT64_ARRAY as i32))) as i32)) != 0)) as i32)) != 0) as i32)) != 0 { 2 } else { 1 }; continue;
}
// C line 57306
4 => {
let _ = { let assigned = ((((this_val).u).ptr) as *mut JSObject); p = assigned; assigned };
vm_block = 3; continue;
}
// C line 57305
5 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) };
}
// C line 57304
6 => {
vm_block = if (((((((this_val).tag) as i32)) != ((JS_TAG_OBJECT as i32))) as i32)) != 0 { 5 } else { 4 }; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:57313. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_typed_array_set_internal(mut ctx: *mut JSContext, mut dst: JSValue, mut src: JSValue, mut off: JSValue) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut p: *mut JSObject = core::mem::zeroed();
let mut src_p: *mut JSObject = core::mem::zeroed();
let mut i: u32 = core::mem::zeroed();
let mut dst_len: i64 = core::mem::zeroed();
let mut src_len: i64 = core::mem::zeroed();
let mut offset: i64 = core::mem::zeroed();
let mut val: JSValue = core::mem::zeroed();
let mut src_obj: JSValue = core::mem::zeroed();
let mut dest_ta: *mut JSTypedArray = core::mem::zeroed();
let mut dest_abuf: *mut JSArrayBuffer = core::mem::zeroed();
let mut src_ta: *mut JSTypedArray = core::mem::zeroed();
let mut src_abuf: *mut JSArrayBuffer = core::mem::zeroed();
let mut shift: i32 = core::mem::zeroed();
let mut vm_block: usize = 48;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 57391
1 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line ? labels: fail
2 => {
let _ = JS_FreeValue(ctx, src_obj);
vm_block = 1; continue;
}
// C line 57388
3 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) };
}
// C line ? labels: done
4 => {
let _ = JS_FreeValue(ctx, src_obj);
vm_block = 3; continue;
}
// C line 57379
5 => {
vm_block = if ((((((i) as i64)) < (src_len)) as i32)) != 0 { 11 } else { 4 }; continue;
}
// C line ?
6 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 5; continue;
}
// C line 57384
7 => {
vm_block = 2; continue;
}
// C line 57383
8 => {
vm_block = if ((((JS_SetPropertyUint32(ctx, dst, (((offset).wrapping_add(((i) as i64))) as u32), val)) < ((0 as i32))) as i32)) != 0 { 7 } else { 6 }; continue;
}
// C line 57382
9 => {
vm_block = 2; continue;
}
// C line 57381
10 => {
vm_block = if (JS_IsException(val)) != 0 { 9 } else { 8 }; continue;
}
// C line 57380
11 => {
let _ = { let assigned = JS_GetPropertyUint32(ctx, src_obj, i); val = assigned; assigned };
vm_block = 10; continue;
}
// C line 57379
12 => {
let _ = { let assigned = (((0 as i32)) as u32); i = assigned; assigned };
vm_block = 5; continue;
}
// C line 57363
13 => {
vm_block = if (((((*(dest_abuf)).data) == ((*(src_abuf)).data)) as i32)) != 0 { 12 } else { 12 }; continue;
}
// C line 57361
14 => {
vm_block = 4; continue;
}
// C line 57359
15 => {
let _ = { core::ptr::copy((((((*(src_abuf)).data).offset((((*(src_ta)).offset) as isize))) as *const c_void)).cast::<u8>(), ((((((*(dest_abuf)).data).offset((((*(dest_ta)).offset) as isize))).offset((((offset).wrapping_shl((shift) as u32)) as isize))) as *mut c_void)).cast::<u8>(), (((src_len).wrapping_shl((shift) as u32)) as usize)); (((((*(dest_abuf)).data).offset((((*(dest_ta)).offset) as isize))).offset((((offset).wrapping_shl((shift) as u32)) as isize))) as *mut c_void) };
vm_block = 14; continue;
}
// C line 57357
16 => {
vm_block = if (((((((*(src_p)).class_id) as i32)) == ((((*(p)).class_id) as i32))) as i32)) != 0 { 15 } else { 13 }; continue;
}
// C line 57354
17 => {
vm_block = 28; continue;
}
// C line 57353
18 => {
vm_block = if ((((offset) > ((dst_len).wrapping_sub(src_len))) as i32)) != 0 { 17 } else { 16 }; continue;
}
// C line 57352
19 => {
let _ = { let assigned = (((((*(src_p)).u).array).count) as i64); src_len = assigned; assigned };
vm_block = 18; continue;
}
// C line 57350
20 => {
vm_block = 39; continue;
}
// C line 57349
21 => {
vm_block = if (typed_array_is_oob(src_p)) != 0 { 20 } else { 19 }; continue;
}
// C line 57347
22 => {
shift = ((*((typed_array_size_log2).as_ptr()).offset((((((*(p)).class_id) as i32)).wrapping_sub((JS_CLASS_UINT8C_ARRAY as i32))) as isize)) as i32);
vm_block = 21; continue;
}
// C line 57346
23 => {
src_abuf = ((*((*(src_ta)).buffer)).u).array_buffer;
vm_block = 22; continue;
}
// C line 57345
24 => {
src_ta = ((*(src_p)).u).typed_array;
vm_block = 23; continue;
}
// C line 57344
25 => {
dest_abuf = ((*((*(dest_ta)).buffer)).u).array_buffer;
vm_block = 24; continue;
}
// C line 57343
26 => {
dest_ta = ((*(p)).u).typed_array;
vm_block = 25; continue;
}
// C line 57376
27 => {
vm_block = 2; continue;
}
// C line ? labels: range_error
28 => {
let _ = JS_ThrowRangeError(ctx, c"invalid array length".as_ptr());
vm_block = 27; continue;
}
// C line 57373
29 => {
vm_block = if ((((offset) > ((dst_len).wrapping_sub(src_len))) as i32)) != 0 { 28 } else { 12 }; continue;
}
// C line 57372
30 => {
vm_block = 2; continue;
}
// C line 57371
31 => {
vm_block = if (js_get_length64(ctx, core::ptr::addr_of_mut!(src_len), src_obj)) != 0 { 30 } else { 29 }; continue;
}
// C line 57341
32 => {
vm_block = if (((((((((((*(src_p)).class_id) as i32)) >= ((JS_CLASS_UINT8C_ARRAY as i32))) as i32)) != 0) && ((((((((*(src_p)).class_id) as i32)) <= ((JS_CLASS_FLOAT64_ARRAY as i32))) as i32)) != 0)) as i32)) != 0 { 26 } else { 31 }; continue;
}
// C line 57340
33 => {
let _ = { let assigned = ((((src_obj).u).ptr) as *mut JSObject); src_p = assigned; assigned };
vm_block = 32; continue;
}
// C line 57339
34 => {
vm_block = 2; continue;
}
// C line 57338
35 => {
vm_block = if (JS_IsException(src_obj)) != 0 { 34 } else { 33 }; continue;
}
// C line 57337
36 => {
let _ = { let assigned = JS_ToObject(ctx, src); src_obj = assigned; assigned };
vm_block = 35; continue;
}
// C line 57336
37 => {
let _ = { let assigned = (((((*(p)).u).array).count) as i64); dst_len = assigned; assigned };
vm_block = 36; continue;
}
// C line 57334
38 => {
vm_block = 2; continue;
}
// C line ? labels: detached
39 => {
let _ = JS_ThrowTypeErrorArrayBufferOOB(ctx);
vm_block = 38; continue;
}
// C line 57331
40 => {
vm_block = if (typed_array_is_oob(p)) != 0 { 39 } else { 37 }; continue;
}
// C line 57330
41 => {
vm_block = 28; continue;
}
// C line 57329
42 => {
vm_block = if ((((offset) < ((((0 as i32)) as i64))) as i32)) != 0 { 41 } else { 40 }; continue;
}
// C line 57328
43 => {
vm_block = 2; continue;
}
// C line 57327
44 => {
vm_block = if (JS_ToInt64Sat(ctx, core::ptr::addr_of_mut!(offset), off)) != 0 { 43 } else { 42 }; continue;
}
// C line 57326
45 => {
vm_block = 2; continue;
}
// C line 57325
46 => {
vm_block = if ((!(!(p).is_null()) as i32)) != 0 { 45 } else { 44 }; continue;
}
// C line 57324
47 => {
let _ = { let assigned = get_typed_array(ctx, dst); p = assigned; assigned };
vm_block = 46; continue;
}
// C line 57322
48 => {
src_obj = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) };
vm_block = 47; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:57394. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_typed_array_at(mut ctx: *mut JSContext, mut this_val: JSValue, mut argc: i32, mut argv: *mut JSValue) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut p: *mut JSObject = core::mem::zeroed();
let mut idx: i64 = core::mem::zeroed();
let mut len: i64 = core::mem::zeroed();
let mut vm_block: usize = 14;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 57418
1 => {
return JS_GetPropertyInt64(ctx, this_val, idx);
}
// C line 57417
2 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) };
}
// C line 57416
3 => {
vm_block = if ((((((((idx) < ((((0 as i32)) as i64))) as i32)) != 0) || (((((idx) >= (len)) as i32)) != 0)) as i32)) != 0 { 2 } else { 1 }; continue;
}
// C line 57415
4 => {
let _ = { let assigned = (((((*(p)).u).array).count) as i64); len = assigned; assigned };
vm_block = 3; continue;
}
// C line 57413
5 => {
let _ = { let assigned = (len).wrapping_add(idx); idx = assigned; assigned };
vm_block = 4; continue;
}
// C line 57412
6 => {
vm_block = if ((((idx) < ((((0 as i32)) as i64))) as i32)) != 0 { 5 } else { 4 }; continue;
}
// C line 57410
7 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 57409
8 => {
vm_block = if (JS_ToInt64Sat(ctx, core::ptr::addr_of_mut!(idx), *(argv).offset(((0 as i32)) as isize))) != 0 { 7 } else { 6 }; continue;
}
// C line 57406
9 => {
let _ = { let assigned = (((((*(p)).u).array).count) as i64); len = assigned; assigned };
vm_block = 8; continue;
}
// C line 57405
10 => {
return JS_ThrowTypeErrorDetachedArrayBuffer(ctx);
}
// C line 57404
11 => {
vm_block = if (typed_array_is_oob(p)) != 0 { 10 } else { 9 }; continue;
}
// C line 57402
12 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 57401
13 => {
vm_block = if ((!(!(p).is_null()) as i32)) != 0 { 12 } else { 11 }; continue;
}
// C line 57400
14 => {
let _ = { let assigned = get_typed_array(ctx, this_val); p = assigned; assigned };
vm_block = 13; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:57421. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_typed_array_with(mut ctx: *mut JSContext, mut this_val: JSValue, mut argc: i32, mut argv: *mut JSValue) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut arr: JSValue = core::mem::zeroed();
let mut val: JSValue = core::mem::zeroed();
let mut p: *mut JSObject = core::mem::zeroed();
let mut idx: i64 = core::mem::zeroed();
let mut len: i64 = core::mem::zeroed();
let mut vm_block: usize = 23;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 57460
1 => {
return arr;
}
// C line 57458
2 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 57457
3 => {
let _ = JS_FreeValue(ctx, arr);
vm_block = 2; continue;
}
// C line 57456
4 => {
vm_block = if ((((JS_SetPropertyInt64(ctx, arr, idx, val)) < ((0 as i32))) as i32)) != 0 { 3 } else { 1 }; continue;
}
// C line 57454
5 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 57453
6 => {
let _ = JS_FreeValue(ctx, val);
vm_block = 5; continue;
}
// C line 57452
7 => {
vm_block = if (JS_IsException(arr)) != 0 { 6 } else { 4 }; continue;
}
// C line 57450
8 => {
let _ = { let assigned = js_typed_array_constructor_ta(ctx, JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }, this_val, (((*(p)).class_id) as i32), ((len) as u32)); arr = assigned; assigned };
vm_block = 7; continue;
}
// C line 57446
9 => {
return JS_ThrowRangeError(ctx, c"invalid array index".as_ptr());
}
// C line 57445
10 => {
vm_block = if (((((((((typed_array_is_oob(p)) != 0) || (((((idx) < ((((0 as i32)) as i64))) as i32)) != 0)) as i32)) != 0) || (((((idx) >= ((((((*(p)).u).array).count) as i64))) as i32)) != 0)) as i32)) != 0 { 9 } else { 8 }; continue;
}
// C line 57443
11 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 57442
12 => {
vm_block = if (JS_IsException(val)) != 0 { 11 } else { 10 }; continue;
}
// C line 57441
13 => {
let _ = { let assigned = JS_ToPrimitive(ctx, *(argv).offset(((1 as i32)) as isize), (1 as i32)); val = assigned; assigned };
vm_block = 12; continue;
}
// C line 57439
14 => {
let _ = { let assigned = (len).wrapping_add(idx); idx = assigned; assigned };
vm_block = 13; continue;
}
// C line 57438
15 => {
vm_block = if ((((idx) < ((((0 as i32)) as i64))) as i32)) != 0 { 14 } else { 13 }; continue;
}
// C line 57436
16 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 57435
17 => {
vm_block = if (JS_ToInt64Sat(ctx, core::ptr::addr_of_mut!(idx), *(argv).offset(((0 as i32)) as isize))) != 0 { 16 } else { 15 }; continue;
}
// C line 57434
18 => {
let _ = { let assigned = (((((*(p)).u).array).count) as i64); len = assigned; assigned };
vm_block = 17; continue;
}
// C line 57432
19 => {
return JS_ThrowTypeErrorDetachedArrayBuffer(ctx);
}
// C line 57431
20 => {
vm_block = if (typed_array_is_oob(p)) != 0 { 19 } else { 18 }; continue;
}
// C line 57430
21 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 57429
22 => {
vm_block = if ((!(!(p).is_null()) as i32)) != 0 { 21 } else { 20 }; continue;
}
// C line 57428
23 => {
let _ = { let assigned = get_typed_array(ctx, this_val); p = assigned; assigned };
vm_block = 22; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:57463. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_typed_array_set(mut ctx: *mut JSContext, mut this_val: JSValue, mut argc: i32, mut argv: *mut JSValue) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut offset: JSValue = core::mem::zeroed();
let mut vm_block: usize = 4;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 57471
1 => {
return js_typed_array_set_internal(ctx, this_val, *(argv).offset(((0 as i32)) as isize), offset);
}
// C line 57469
2 => {
let _ = { let assigned = *(argv).offset(((1 as i32)) as isize); offset = assigned; assigned };
vm_block = 1; continue;
}
// C line 57468
3 => {
vm_block = if ((((argc) > ((1 as i32))) as i32)) != 0 { 2 } else { 1 }; continue;
}
// C line 57467
4 => {
offset = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) };
vm_block = 3; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:57474. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_create_typed_array_iterator(mut ctx: *mut JSContext, mut this_val: JSValue, mut argc: i32, mut argv: *mut JSValue, mut magic: i32) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut vm_block: usize = 3;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 57479
1 => {
return js_create_array_iterator(ctx, this_val, argc, argv, magic);
}
// C line 57478
2 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 57477
3 => {
vm_block = if (validate_typed_array(ctx, this_val)) != 0 { 2 } else { 1 }; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:57482. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_typed_array_create(mut ctx: *mut JSContext, mut ctor: JSValue, mut argc: i32, mut argv: *mut JSValue) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut ret: JSValue = core::mem::zeroed();
let mut new_len: i32 = core::mem::zeroed();
let mut len: i64 = core::mem::zeroed();
let mut vm_block: usize = 14;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 57507
1 => {
return ret;
}
// C line 57504
2 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line ? labels: fail
3 => {
let _ = JS_FreeValue(ctx, ret);
vm_block = 2; continue;
}
// C line 57501
4 => {
let _ = JS_ThrowTypeError(ctx, c"TypedArray length is too small".as_ptr());
vm_block = 3; continue;
}
// C line 57500
5 => {
vm_block = if ((((((new_len) as i64)) < (len)) as i32)) != 0 { 4 } else { 1 }; continue;
}
// C line 57499
6 => {
vm_block = 3; continue;
}
// C line 57498
7 => {
vm_block = if (JS_ToLengthFree(ctx, core::ptr::addr_of_mut!(len), JS_DupValue(ctx, *(argv).offset(((0 as i32)) as isize)))) != 0 { 6 } else { 5 }; continue;
}
// C line 57496
8 => {
vm_block = if ((((argc) == ((1 as i32))) as i32)) != 0 { 7 } else { 1 }; continue;
}
// C line 57495
9 => {
vm_block = 3; continue;
}
// C line 57494
10 => {
vm_block = if ((((new_len) < ((0 as i32))) as i32)) != 0 { 9 } else { 8 }; continue;
}
// C line 57493
11 => {
let _ = { let assigned = js_typed_array_get_length_unsafe(ctx, ret); new_len = assigned; assigned };
vm_block = 10; continue;
}
// C line 57491
12 => {
return ret;
}
// C line 57490
13 => {
vm_block = if (JS_IsException(ret)) != 0 { 12 } else { 11 }; continue;
}
// C line 57489
14 => {
let _ = { let assigned = JS_CallConstructor(ctx, ctor, argc, argv); ret = assigned; assigned };
vm_block = 13; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:57519. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_typed_array___speciesCreate(mut ctx: *mut JSContext, mut this_val: JSValue, mut argc: i32, mut argv: *mut JSValue) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut obj: JSValue = core::mem::zeroed();
let mut p: *mut JSObject = core::mem::zeroed();
let mut ctor: JSValue = core::mem::zeroed();
let mut ret: JSValue = core::mem::zeroed();
let mut argc1: i32 = core::mem::zeroed();
let mut vm_block: usize = 13;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 57543
1 => {
return ret;
}
// C line 57537
2 => {
let _ = { let assigned = js_typed_array_constructor(ctx, JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }, argc1, (argv).offset((((1 as i32)) as isize)), (((*(p)).class_id) as i32)); ret = assigned; assigned };
vm_block = 1; continue;
}
// C line 57541
3 => {
let _ = JS_FreeValue(ctx, ctor);
vm_block = 1; continue;
}
// C line 57540
4 => {
let _ = { let assigned = js_typed_array_create(ctx, ctor, argc1, (argv).offset((((1 as i32)) as isize))); ret = assigned; assigned };
vm_block = 3; continue;
}
// C line 57536
5 => {
vm_block = if (JS_IsUndefined(ctor)) != 0 { 2 } else { 4 }; continue;
}
// C line 57535
6 => {
let _ = { let assigned = max_int((argc).wrapping_sub((1 as i32)), (0 as i32)); argc1 = assigned; assigned };
vm_block = 5; continue;
}
// C line 57534
7 => {
return ctor;
}
// C line 57533
8 => {
vm_block = if (JS_IsException(ctor)) != 0 { 7 } else { 6 }; continue;
}
// C line 57532
9 => {
let _ = { let assigned = JS_SpeciesConstructor(ctx, obj, JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }); ctor = assigned; assigned };
vm_block = 8; continue;
}
// C line 57531
10 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 57530
11 => {
vm_block = if ((!(!(p).is_null()) as i32)) != 0 { 10 } else { 9 }; continue;
}
// C line 57529
12 => {
let _ = { let assigned = get_typed_array(ctx, obj); p = assigned; assigned };
vm_block = 11; continue;
}
// C line 57528
13 => {
let _ = { let assigned = *(argv).offset(((0 as i32)) as isize); obj = assigned; assigned };
vm_block = 12; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:57546. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_typed_array_from(mut ctx: *mut JSContext, mut this_val: JSValue, mut argc: i32, mut argv: *mut JSValue) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut items: JSValue = core::mem::zeroed();
let mut mapfn: JSValue = core::mem::zeroed();
let mut this_arg: JSValue = core::mem::zeroed();
let mut args: [JSValue; 2] = core::mem::zeroed();
let mut iter: JSValue = core::mem::zeroed();
let mut arr: JSValue = core::mem::zeroed();
let mut r: JSValue = core::mem::zeroed();
let mut v: JSValue = core::mem::zeroed();
let mut v2: JSValue = core::mem::zeroed();
let mut k: i64 = core::mem::zeroed();
let mut len: i64 = core::mem::zeroed();
let mut mapping: i32 = core::mem::zeroed();
let mut len1: u32 = core::mem::zeroed();
let mut vm_block: usize = 59;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 57622
1 => {
return r;
}
// C line 57621
2 => {
let _ = JS_FreeValue(ctx, iter);
vm_block = 1; continue;
}
// C line ? labels: done
3 => {
let _ = JS_FreeValue(ctx, arr);
vm_block = 2; continue;
}
// C line 57618
4 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) }; r = assigned; assigned };
vm_block = 3; continue;
}
// C line ? labels: exception
5 => {
let _ = JS_FreeValue(ctx, r);
vm_block = 4; continue;
}
// C line 57615
6 => {
vm_block = 3; continue;
}
// C line 57599
7 => {
vm_block = if ((((k) < (len)) as i32)) != 0 { 21 } else { 6 }; continue;
}
// C line ?
8 => {
let _ = { let old = k; k = (k).wrapping_add(1); old };
vm_block = 7; continue;
}
// C line 57613
9 => {
vm_block = 5; continue;
}
// C line 57612
10 => {
vm_block = if ((((JS_SetPropertyInt64(ctx, r, k, v)) < ((0 as i32))) as i32)) != 0 { 9 } else { 8 }; continue;
}
// C line 57610
11 => {
vm_block = 5; continue;
}
// C line 57609
12 => {
vm_block = if (JS_IsException(v)) != 0 { 11 } else { 10 }; continue;
}
// C line 57608
13 => {
let _ = { let assigned = v2; v = assigned; assigned };
vm_block = 12; continue;
}
// C line 57607
14 => {
let _ = JS_FreeValue(ctx, v);
vm_block = 13; continue;
}
// C line 57606
15 => {
let _ = { let assigned = JS_Call(ctx, mapfn, this_arg, (2 as i32), (args).as_mut_ptr()); v2 = assigned; assigned };
vm_block = 14; continue;
}
// C line 57605
16 => {
let _ = { let assigned = JS_NewInt32(ctx, ((k) as i32)); *((args).as_mut_ptr()).offset(((1 as i32)) as isize) = assigned; assigned };
vm_block = 15; continue;
}
// C line 57604
17 => {
let _ = { let assigned = v; *((args).as_mut_ptr()).offset(((0 as i32)) as isize) = assigned; assigned };
vm_block = 16; continue;
}
// C line 57603
18 => {
vm_block = if (mapping) != 0 { 17 } else { 10 }; continue;
}
// C line 57602
19 => {
vm_block = 5; continue;
}
// C line 57601
20 => {
vm_block = if (JS_IsException(v)) != 0 { 19 } else { 18 }; continue;
}
// C line 57600
21 => {
let _ = { let assigned = JS_GetPropertyInt64(ctx, arr, k); v = assigned; assigned };
vm_block = 20; continue;
}
// C line 57599
22 => {
let _ = { let assigned = (((0 as i32)) as i64); k = assigned; assigned };
vm_block = 7; continue;
}
// C line 57598
23 => {
vm_block = 5; continue;
}
// C line 57597
24 => {
vm_block = if (JS_IsException(r)) != 0 { 23 } else { 22 }; continue;
}
// C line 57596
25 => {
let _ = JS_FreeValue(ctx, v);
vm_block = 24; continue;
}
// C line 57595
26 => {
let _ = { let assigned = js_typed_array_create(ctx, this_val, (1 as i32), (args).as_mut_ptr()); r = assigned; assigned };
vm_block = 25; continue;
}
// C line 57594
27 => {
let _ = { let assigned = v; *((args).as_mut_ptr()).offset(((0 as i32)) as isize) = assigned; assigned };
vm_block = 26; continue;
}
// C line 57593
28 => {
let _ = { let assigned = JS_NewInt64(ctx, len); v = assigned; assigned };
vm_block = 27; continue;
}
// C line 57585
29 => {
let _ = { let assigned = ((len1) as i64); len = assigned; assigned };
vm_block = 28; continue;
}
// C line 57584
30 => {
vm_block = 5; continue;
}
// C line 57583
31 => {
vm_block = if (JS_IsException(arr)) != 0 { 30 } else { 29 }; continue;
}
// C line 57582
32 => {
let _ = { let assigned = js_array_from_iterator(ctx, core::ptr::addr_of_mut!(len1), items, iter); arr = assigned; assigned };
vm_block = 31; continue;
}
// C line 57580
33 => {
vm_block = 5; continue;
}
// C line 57579
34 => {
let _ = JS_ThrowTypeError(ctx, c"value is not iterable".as_ptr());
vm_block = 33; continue;
}
// C line 57578
35 => {
vm_block = if ((!((JS_IsFunction(ctx, iter)) != 0) as i32)) != 0 { 34 } else { 32 }; continue;
}
// C line 57591
36 => {
vm_block = 5; continue;
}
// C line 57590
37 => {
vm_block = if ((((js_get_length64(ctx, core::ptr::addr_of_mut!(len), arr)) < ((0 as i32))) as i32)) != 0 { 36 } else { 28 }; continue;
}
// C line 57589
38 => {
vm_block = 5; continue;
}
// C line 57588
39 => {
vm_block = if (JS_IsException(arr)) != 0 { 38 } else { 37 }; continue;
}
// C line 57587
40 => {
let _ = { let assigned = JS_ToObject(ctx, items); arr = assigned; assigned };
vm_block = 39; continue;
}
// C line 57576
41 => {
vm_block = if ((((((!((JS_IsUndefined(iter)) != 0) as i32)) != 0) && (((!((JS_IsNull(iter)) != 0) as i32)) != 0)) as i32)) != 0 { 35 } else { 40 }; continue;
}
// C line 57575
42 => {
vm_block = 5; continue;
}
// C line 57574
43 => {
vm_block = if (JS_IsException(iter)) != 0 { 42 } else { 41 }; continue;
}
// C line 57573
44 => {
let _ = { let assigned = JS_GetProperty(ctx, items, (((crate::quickjs_atom::JS_ATOM_Symbol_iterator as i32)) as JSAtom)); iter = assigned; assigned };
vm_block = 43; continue;
}
// C line 57570
45 => {
let _ = { let assigned = *(argv).offset(((2 as i32)) as isize); this_arg = assigned; assigned };
vm_block = 44; continue;
}
// C line 57569
46 => {
vm_block = if ((((argc) > ((2 as i32))) as i32)) != 0 { 45 } else { 44 }; continue;
}
// C line 57568
47 => {
let _ = { let assigned = (1 as i32); mapping = assigned; assigned };
vm_block = 46; continue;
}
// C line 57567
48 => {
vm_block = 5; continue;
}
// C line 57566
49 => {
vm_block = if (check_function(ctx, mapfn)) != 0 { 48 } else { 47 }; continue;
}
// C line 57565
50 => {
vm_block = if ((!((JS_IsUndefined(mapfn)) != 0) as i32)) != 0 { 49 } else { 44 }; continue;
}
// C line 57564
51 => {
let _ = { let assigned = *(argv).offset(((1 as i32)) as isize); mapfn = assigned; assigned };
vm_block = 50; continue;
}
// C line 57563
52 => {
vm_block = if ((((argc) > ((1 as i32))) as i32)) != 0 { 51 } else { 44 }; continue;
}
// C line 57561
53 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; iter = assigned; assigned };
vm_block = 52; continue;
}
// C line 57560
54 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; arr = assigned; assigned };
vm_block = 53; continue;
}
// C line 57559
55 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; r = assigned; assigned };
vm_block = 54; continue;
}
// C line 57558
56 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; this_arg = assigned; assigned };
vm_block = 55; continue;
}
// C line 57557
57 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; mapfn = assigned; assigned };
vm_block = 56; continue;
}
// C line 57556
58 => {
let _ = { let assigned = (0 as i32); mapping = assigned; assigned };
vm_block = 57; continue;
}
// C line 57550
59 => {
items = *(argv).offset(((0 as i32)) as isize);
vm_block = 58; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:57625. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_typed_array_of(mut ctx: *mut JSContext, mut this_val: JSValue, mut argc: i32, mut argv: *mut JSValue) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut obj: JSValue = core::mem::zeroed();
let mut args: [JSValue; 1] = core::mem::zeroed();
let mut i: i32 = core::mem::zeroed();
let mut vm_block: usize = 11;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 57643
1 => {
return obj;
}
// C line 57637
2 => {
vm_block = if ((((i) < (argc)) as i32)) != 0 { 6 } else { 1 }; continue;
}
// C line ?
3 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 2; continue;
}
// C line 57640
4 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 57639
5 => {
let _ = JS_FreeValue(ctx, obj);
vm_block = 4; continue;
}
// C line 57638
6 => {
vm_block = if ((((JS_SetPropertyUint32(ctx, obj, ((i) as u32), JS_DupValue(ctx, *(argv).offset((i) as isize)))) < ((0 as i32))) as i32)) != 0 { 5 } else { 3 }; continue;
}
// C line 57637
7 => {
let _ = { let assigned = (0 as i32); i = assigned; assigned };
vm_block = 2; continue;
}
// C line 57635
8 => {
return obj;
}
// C line 57634
9 => {
vm_block = if (JS_IsException(obj)) != 0 { 8 } else { 7 }; continue;
}
// C line 57633
10 => {
let _ = { let assigned = js_typed_array_create(ctx, this_val, (1 as i32), (args).as_mut_ptr()); obj = assigned; assigned };
vm_block = 9; continue;
}
// C line 57632
11 => {
let _ = { let assigned = JS_NewInt32(ctx, argc); *((args).as_mut_ptr()).offset(((0 as i32)) as isize) = assigned; assigned };
vm_block = 10; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:57646. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_typed_array_copyWithin(mut ctx: *mut JSContext, mut this_val: JSValue, mut argc: i32, mut argv: *mut JSValue) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut p: *mut JSObject = core::mem::zeroed();
let mut len: i32 = core::mem::zeroed();
let mut to: i32 = core::mem::zeroed();
let mut from: i32 = core::mem::zeroed();
let mut v_final: i32 = core::mem::zeroed();
let mut count: i32 = core::mem::zeroed();
let mut shift: i32 = core::mem::zeroed();
let mut space: i32 = core::mem::zeroed();
let mut vm_block: usize = 23;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 57684
1 => {
return JS_DupValue(ctx, this_val);
}
// C line 57680
2 => {
let _ = { core::ptr::copy(((((((((*(p)).u).array).u).uint8_ptr).offset((((from).wrapping_shl((shift) as u32)) as isize))) as *const c_void)).cast::<u8>(), ((((((((*(p)).u).array).u).uint8_ptr).offset((((to).wrapping_shl((shift) as u32)) as isize))) as *mut c_void)).cast::<u8>(), (((count).wrapping_shl((shift) as u32)) as usize)); (((((((*(p)).u).array).u).uint8_ptr).offset((((to).wrapping_shl((shift) as u32)) as isize))) as *mut c_void) };
vm_block = 1; continue;
}
// C line 57679
3 => {
let _ = { let assigned = ((*((typed_array_size_log2).as_ptr()).offset((((((*(p)).class_id) as i32)).wrapping_sub((JS_CLASS_UINT8C_ARRAY as i32))) as isize)) as i32); shift = assigned; assigned };
vm_block = 2; continue;
}
// C line 57678
4 => {
vm_block = if ((((count) > ((0 as i32))) as i32)) != 0 { 3 } else { 1 }; continue;
}
// C line 57677
5 => {
let _ = { let assigned = min_int(count, space); count = assigned; assigned };
vm_block = 4; continue;
}
// C line 57676
6 => {
let _ = { let assigned = min_int((v_final).wrapping_sub(from), (len).wrapping_sub(to)); count = assigned; assigned };
vm_block = 5; continue;
}
// C line 57675
7 => {
let _ = { let assigned = ((((((*(p)).u).array).count).wrapping_sub(((max_int(to, from)) as u32))) as i32); space = assigned; assigned };
vm_block = 6; continue;
}
// C line 57672
8 => {
return JS_ThrowTypeErrorArrayBufferOOB(ctx);
}
// C line 57671
9 => {
vm_block = if (typed_array_is_oob(p)) != 0 { 8 } else { 7 }; continue;
}
// C line 57668
10 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 57667
11 => {
vm_block = if (JS_ToInt32Clamp(ctx, core::ptr::addr_of_mut!(v_final), *(argv).offset(((2 as i32)) as isize), (0 as i32), len, len)) != 0 { 10 } else { 9 }; continue;
}
// C line 57666
12 => {
vm_block = if ((((((((argc) > ((2 as i32))) as i32)) != 0) && (((!((JS_IsUndefined(*(argv).offset(((2 as i32)) as isize))) != 0) as i32)) != 0)) as i32)) != 0 { 11 } else { 9 }; continue;
}
// C line 57665
13 => {
let _ = { let assigned = len; v_final = assigned; assigned };
vm_block = 12; continue;
}
// C line 57663
14 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 57662
15 => {
vm_block = if (JS_ToInt32Clamp(ctx, core::ptr::addr_of_mut!(from), *(argv).offset(((1 as i32)) as isize), (0 as i32), len, len)) != 0 { 14 } else { 13 }; continue;
}
// C line 57660
16 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 57659
17 => {
vm_block = if (JS_ToInt32Clamp(ctx, core::ptr::addr_of_mut!(to), *(argv).offset(((0 as i32)) as isize), (0 as i32), len, len)) != 0 { 16 } else { 15 }; continue;
}
// C line 57657
18 => {
let _ = { let assigned = (((((*(p)).u).array).count) as i32); len = assigned; assigned };
vm_block = 17; continue;
}
// C line 57656
19 => {
return JS_ThrowTypeErrorArrayBufferOOB(ctx);
}
// C line 57655
20 => {
vm_block = if (typed_array_is_oob(p)) != 0 { 19 } else { 18 }; continue;
}
// C line 57654
21 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 57653
22 => {
vm_block = if ((!(!(p).is_null()) as i32)) != 0 { 21 } else { 20 }; continue;
}
// C line 57652
23 => {
let _ = { let assigned = get_typed_array(ctx, this_val); p = assigned; assigned };
vm_block = 22; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:57687. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_typed_array_fill(mut ctx: *mut JSContext, mut this_val: JSValue, mut argc: i32, mut argv: *mut JSValue) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut p: *mut JSObject = core::mem::zeroed();
let mut len: i32 = core::mem::zeroed();
let mut k: i32 = core::mem::zeroed();
let mut v_final: i32 = core::mem::zeroed();
let mut shift: i32 = core::mem::zeroed();
let mut v64: u64 = core::mem::zeroed();
let mut v: i32 = core::mem::zeroed();
let mut v_1: u32 = core::mem::zeroed();
let mut d: f64 = core::mem::zeroed();
let mut u: BufferRecord57721_13 = core::mem::zeroed();
let mut u_1: JSFloat64Union = core::mem::zeroed();
let mut vm_block: usize = 56;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 57776
1 => {
return JS_DupValue(ctx, this_val);
}
// C line ?
2 => {
let _ = std::process::abort();
vm_block = 1; continue;
}
// C line 57772
3 => {
vm_block = 1; continue;
}
// C line 57769
4 => {
vm_block = if ((((k) < (v_final)) as i32)) != 0 { 6 } else { 3 }; continue;
}
// C line ?
5 => {
let _ = { let old = k; k = (k).wrapping_add(1); old };
vm_block = 4; continue;
}
// C line 57770
6 => {
let _ = { let assigned = v64; *(((((*(p)).u).array).u).uint64_ptr).offset((k) as isize) = assigned; assigned };
vm_block = 5; continue;
}
// C line 57767
7 => {
vm_block = 1; continue;
}
// C line 57764
8 => {
vm_block = if ((((k) < (v_final)) as i32)) != 0 { 10 } else { 7 }; continue;
}
// C line ?
9 => {
let _ = { let old = k; k = (k).wrapping_add(1); old };
vm_block = 8; continue;
}
// C line 57765
10 => {
let _ = { let assigned = ((v64) as u32); *(((((*(p)).u).array).u).uint32_ptr).offset((k) as isize) = assigned; assigned };
vm_block = 9; continue;
}
// C line 57762
11 => {
vm_block = 1; continue;
}
// C line 57759
12 => {
vm_block = if ((((k) < (v_final)) as i32)) != 0 { 14 } else { 11 }; continue;
}
// C line ?
13 => {
let _ = { let old = k; k = (k).wrapping_add(1); old };
vm_block = 12; continue;
}
// C line 57760
14 => {
let _ = { let assigned = ((v64) as u16); *(((((*(p)).u).array).u).uint16_ptr).offset((k) as isize) = assigned; assigned };
vm_block = 13; continue;
}
// C line 57757
15 => {
vm_block = 1; continue;
}
// C line 57755
16 => {
let _ = { let dst = ((((((((*(p)).u).array).u).uint8_ptr).offset(((k) as isize))) as *mut c_void)) as *mut u8; core::ptr::write_bytes(dst, (((v64) as i32)) as u8, ((((v_final).wrapping_sub(k)) as usize)) as usize); dst as *mut c_void };
vm_block = 15; continue;
}
// C line 57754
17 => {
vm_block = if ((((k) < (v_final)) as i32)) != 0 { 16 } else { 15 }; continue;
}
// C line 57752
18 => {
vm_block = match shift { x if x == (3 as i32) => 4, x if x == (2 as i32) => 8, x if x == (1 as i32) => 12, x if x == (0 as i32) => 17, _ => 2, }; continue;
}
// C line 57751
19 => {
let _ = { let assigned = ((*((typed_array_size_log2).as_ptr()).offset((((((*(p)).class_id) as i32)).wrapping_sub((JS_CLASS_UINT8C_ARRAY as i32))) as isize)) as i32); shift = assigned; assigned };
vm_block = 18; continue;
}
// C line 57750
20 => {
let _ = { let assigned = min_int(v_final, (((((*(p)).u).array).count) as i32)); v_final = assigned; assigned };
vm_block = 19; continue;
}
// C line 57747
21 => {
return JS_ThrowTypeErrorArrayBufferOOB(ctx);
}
// C line 57746
22 => {
vm_block = if (typed_array_is_oob(p)) != 0 { 21 } else { 20 }; continue;
}
// C line 57743
23 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 57742
24 => {
vm_block = if (JS_ToInt32Clamp(ctx, core::ptr::addr_of_mut!(v_final), *(argv).offset(((2 as i32)) as isize), (0 as i32), len, len)) != 0 { 23 } else { 22 }; continue;
}
// C line 57741
25 => {
vm_block = if ((((((((argc) > ((2 as i32))) as i32)) != 0) && (((!((JS_IsUndefined(*(argv).offset(((2 as i32)) as isize))) != 0) as i32)) != 0)) as i32)) != 0 { 24 } else { 22 }; continue;
}
// C line 57740
26 => {
let _ = { let assigned = len; v_final = assigned; assigned };
vm_block = 25; continue;
}
// C line 57737
27 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 57736
28 => {
vm_block = if (JS_ToInt32Clamp(ctx, core::ptr::addr_of_mut!(k), *(argv).offset(((1 as i32)) as isize), (0 as i32), len, len)) != 0 { 27 } else { 26 }; continue;
}
// C line 57735
29 => {
vm_block = if ((((argc) > ((1 as i32))) as i32)) != 0 { 28 } else { 26 }; continue;
}
// C line 57734
30 => {
let _ = { let assigned = (0 as i32); k = assigned; assigned };
vm_block = 29; continue;
}
// C line 57705
31 => {
let _ = { let assigned = ((v) as u64); v64 = assigned; assigned };
vm_block = 30; continue;
}
// C line 57704
32 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 57703
33 => {
vm_block = if (JS_ToUint8ClampFree(ctx, core::ptr::addr_of_mut!(v), JS_DupValue(ctx, *(argv).offset(((0 as i32)) as isize)))) != 0 { 32 } else { 31 }; continue;
}
// C line 57710
34 => {
let _ = { let assigned = ((v_1) as u64); v64 = assigned; assigned };
vm_block = 30; continue;
}
// C line 57709
35 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 57708
36 => {
vm_block = if (JS_ToUint32(ctx, core::ptr::addr_of_mut!(v_1), *(argv).offset(((0 as i32)) as isize))) != 0 { 35 } else { 34 }; continue;
}
// C line 57713
37 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 57712
38 => {
vm_block = if (JS_ToBigInt64(ctx, ((core::ptr::addr_of_mut!(v64)) as *mut i64), *(argv).offset(((0 as i32)) as isize))) != 0 { 37 } else { 30 }; continue;
}
// C line 57719
39 => {
let _ = { let assigned = ((crate::cutils_header::tofp16(d)) as u64); v64 = assigned; assigned };
vm_block = 30; continue;
}
// C line 57726
40 => {
let _ = { let assigned = (((u).u32) as u64); v64 = assigned; assigned };
vm_block = 30; continue;
}
// C line 57725
41 => {
let _ = { let assigned = ((d) as f32); (u).f = assigned; assigned };
vm_block = 40; continue;
}
// C line 57730
42 => {
let _ = { let assigned = (u_1).u64; v64 = assigned; assigned };
vm_block = 30; continue;
}
// C line 57729
43 => {
let _ = { let assigned = d; (u_1).d = assigned; assigned };
vm_block = 42; continue;
}
// C line 57720
44 => {
vm_block = if (((((((*(p)).class_id) as i32)) == ((JS_CLASS_FLOAT32_ARRAY as i32))) as i32)) != 0 { 41 } else { 43 }; continue;
}
// C line 57718
45 => {
vm_block = if (((((((*(p)).class_id) as i32)) == ((JS_CLASS_FLOAT16_ARRAY as i32))) as i32)) != 0 { 39 } else { 44 }; continue;
}
// C line 57717
46 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 57716
47 => {
vm_block = if (JS_ToFloat64(ctx, core::ptr::addr_of_mut!(d), *(argv).offset(((0 as i32)) as isize))) != 0 { 46 } else { 45 }; continue;
}
// C line 57711
48 => {
vm_block = if (((((((*(p)).class_id) as i32)) <= ((JS_CLASS_BIG_UINT64_ARRAY as i32))) as i32)) != 0 { 38 } else { 47 }; continue;
}
// C line 57706
49 => {
vm_block = if (((((((*(p)).class_id) as i32)) <= ((JS_CLASS_UINT32_ARRAY as i32))) as i32)) != 0 { 36 } else { 48 }; continue;
}
// C line 57701
50 => {
vm_block = if (((((((*(p)).class_id) as i32)) == ((JS_CLASS_UINT8C_ARRAY as i32))) as i32)) != 0 { 33 } else { 49 }; continue;
}
// C line 57699
51 => {
let _ = { let assigned = (((((*(p)).u).array).count) as i32); len = assigned; assigned };
vm_block = 50; continue;
}
// C line 57698
52 => {
return JS_ThrowTypeErrorArrayBufferOOB(ctx);
}
// C line 57697
53 => {
vm_block = if (typed_array_is_oob(p)) != 0 { 52 } else { 51 }; continue;
}
// C line 57696
54 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 57695
55 => {
vm_block = if ((!(!(p).is_null()) as i32)) != 0 { 54 } else { 53 }; continue;
}
// C line 57694
56 => {
let _ = { let assigned = get_typed_array(ctx, this_val); p = assigned; assigned };
vm_block = 55; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:57779. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_typed_array_find(mut ctx: *mut JSContext, mut this_val: JSValue, mut argc: i32, mut argv: *mut JSValue, mut mode: i32) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut func: JSValue = core::mem::zeroed();
let mut this_arg: JSValue = core::mem::zeroed();
let mut args: [JSValue; 3] = core::mem::zeroed();
let mut val: JSValue = core::mem::zeroed();
let mut index_val: JSValue = core::mem::zeroed();
let mut res: JSValue = core::mem::zeroed();
let mut len: i32 = core::mem::zeroed();
let mut k: i32 = core::mem::zeroed();
let mut end: i32 = core::mem::zeroed();
let mut dir: i32 = core::mem::zeroed();
let mut vm_block: usize = 40;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 57838
1 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line ? labels: exception
2 => {
let _ = JS_FreeValue(ctx, val);
vm_block = 1; continue;
}
// C line 57832
3 => {
return JS_NewInt32(ctx, ((1 as i32)).wrapping_neg());
}
// C line 57834
4 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) };
}
// C line 57831
5 => {
vm_block = if ((((((((mode) == ((ArrayFindIndex as i32))) as i32)) != 0) || (((((mode) == ((ArrayFindLastIndex as i32))) as i32)) != 0)) as i32)) != 0 { 3 } else { 4 }; continue;
}
// C line 57810
6 => {
vm_block = if ((((k) != (end)) as i32)) != 0 { 23 } else { 5 }; continue;
}
// C line ?
7 => {
let _ = { k = (k).wrapping_add(dir); k };
vm_block = 6; continue;
}
// C line 57829
8 => {
let _ = JS_FreeValue(ctx, val);
vm_block = 7; continue;
}
// C line 57824
9 => {
return index_val;
}
// C line 57823
10 => {
let _ = JS_FreeValue(ctx, val);
vm_block = 9; continue;
}
// C line 57826
11 => {
return val;
}
// C line 57822
12 => {
vm_block = if ((((((((mode) == ((ArrayFindIndex as i32))) as i32)) != 0) || (((((mode) == ((ArrayFindLastIndex as i32))) as i32)) != 0)) as i32)) != 0 { 10 } else { 11 }; continue;
}
// C line 57821
13 => {
vm_block = if (JS_ToBoolFree(ctx, res)) != 0 { 12 } else { 8 }; continue;
}
// C line 57820
14 => {
vm_block = 2; continue;
}
// C line 57819
15 => {
vm_block = if (JS_IsException(res)) != 0 { 14 } else { 13 }; continue;
}
// C line 57818
16 => {
let _ = { let assigned = JS_Call(ctx, func, this_arg, (3 as i32), (args).as_mut_ptr()); res = assigned; assigned };
vm_block = 15; continue;
}
// C line 57817
17 => {
let _ = { let assigned = this_val; *((args).as_mut_ptr()).offset(((2 as i32)) as isize) = assigned; assigned };
vm_block = 16; continue;
}
// C line 57816
18 => {
let _ = { let assigned = index_val; *((args).as_mut_ptr()).offset(((1 as i32)) as isize) = assigned; assigned };
vm_block = 17; continue;
}
// C line 57815
19 => {
let _ = { let assigned = val; *((args).as_mut_ptr()).offset(((0 as i32)) as isize) = assigned; assigned };
vm_block = 18; continue;
}
// C line 57814
20 => {
vm_block = 2; continue;
}
// C line 57813
21 => {
vm_block = if (JS_IsException(val)) != 0 { 20 } else { 19 }; continue;
}
// C line 57812
22 => {
let _ = { let assigned = JS_GetPropertyValue(ctx, this_val, index_val); val = assigned; assigned };
vm_block = 21; continue;
}
// C line 57811
23 => {
let _ = { let assigned = JS_NewInt32(ctx, k); index_val = assigned; assigned };
vm_block = 22; continue;
}
// C line 57807
24 => {
let _ = { let assigned = ((1 as i32)).wrapping_neg(); end = assigned; assigned };
vm_block = 6; continue;
}
// C line 57806
25 => {
let _ = { let assigned = ((1 as i32)).wrapping_neg(); dir = assigned; assigned };
vm_block = 24; continue;
}
// C line 57805
26 => {
let _ = { let assigned = (len).wrapping_sub((1 as i32)); k = assigned; assigned };
vm_block = 25; continue;
}
// C line 57804
27 => {
vm_block = if ((((((((mode) == ((ArrayFindLast as i32))) as i32)) != 0) || (((((mode) == ((ArrayFindLastIndex as i32))) as i32)) != 0)) as i32)) != 0 { 26 } else { 6 }; continue;
}
// C line 57803
28 => {
let _ = { let assigned = len; end = assigned; assigned };
vm_block = 27; continue;
}
// C line 57802
29 => {
let _ = { let assigned = (1 as i32); dir = assigned; assigned };
vm_block = 28; continue;
}
// C line 57801
30 => {
let _ = { let assigned = (0 as i32); k = assigned; assigned };
vm_block = 29; continue;
}
// C line 57799
31 => {
let _ = { let assigned = *(argv).offset(((1 as i32)) as isize); this_arg = assigned; assigned };
vm_block = 30; continue;
}
// C line 57798
32 => {
vm_block = if ((((argc) > ((1 as i32))) as i32)) != 0 { 31 } else { 30 }; continue;
}
// C line 57797
33 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; this_arg = assigned; assigned };
vm_block = 32; continue;
}
// C line 57795
34 => {
vm_block = 2; continue;
}
// C line 57794
35 => {
vm_block = if (check_function(ctx, func)) != 0 { 34 } else { 33 }; continue;
}
// C line 57793
36 => {
let _ = { let assigned = *(argv).offset(((0 as i32)) as isize); func = assigned; assigned };
vm_block = 35; continue;
}
// C line 57791
37 => {
vm_block = 2; continue;
}
// C line 57790
38 => {
vm_block = if ((((len) < ((0 as i32))) as i32)) != 0 { 37 } else { 36 }; continue;
}
// C line 57789
39 => {
let _ = { let assigned = js_typed_array_get_length_unsafe(ctx, this_val); len = assigned; assigned };
vm_block = 38; continue;
}
// C line 57788
40 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; val = assigned; assigned };
vm_block = 39; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:57845. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_typed_array_indexOf(mut ctx: *mut JSContext, mut this_val: JSValue, mut argc: i32, mut argv: *mut JSValue, mut special: i32) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut p: *mut JSObject = core::mem::zeroed();
let mut len: i32 = core::mem::zeroed();
let mut tag: i32 = core::mem::zeroed();
let mut is_int: i32 = core::mem::zeroed();
let mut is_bigint: i32 = core::mem::zeroed();
let mut k: i32 = core::mem::zeroed();
let mut stop: i32 = core::mem::zeroed();
let mut inc: i32 = core::mem::zeroed();
let mut res: i32 = core::mem::zeroed();
let mut v64: i64 = core::mem::zeroed();
let mut d: f64 = core::mem::zeroed();
let mut f: f32 = core::mem::zeroed();
let mut hf: u16 = core::mem::zeroed();
let mut k1: i64 = core::mem::zeroed();
let mut buf1: JSBigIntBuf = core::mem::zeroed();
let mut p1: *mut JSBigInt = core::mem::zeroed();
let mut sz: i32 = core::mem::zeroed();
let mut pv: *const u8 = core::mem::zeroed();
let mut pp: *const u8 = core::mem::zeroed();
let mut v: u16 = core::mem::zeroed();
let mut pv_1: *const u16 = core::mem::zeroed();
let mut v_1: u16 = core::mem::zeroed();
let mut pv_2: *const u32 = core::mem::zeroed();
let mut v_2: u32 = core::mem::zeroed();
let mut pv_3: *const u16 = core::mem::zeroed();
let mut pv_4: *const u16 = core::mem::zeroed();
let mut pv_5: *const u16 = core::mem::zeroed();
let mut pv_6: *const f32 = core::mem::zeroed();
let mut pv_7: *const f32 = core::mem::zeroed();
let mut pv_8: *const f64 = core::mem::zeroed();
let mut pv_9: *const f64 = core::mem::zeroed();
let mut pv_10: *const u64 = core::mem::zeroed();
let mut v_3: u64 = core::mem::zeroed();
let mut vm_block: usize = 190;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line ? labels: exception
1 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 58124
2 => {
return JS_NewBool(ctx, (((res) >= ((0 as i32))) as i32));
}
// C line 58126
3 => {
return JS_NewInt32(ctx, res);
}
// C line 58123 labels: done
4 => {
vm_block = if ((((special) == (((1 as i32)).wrapping_neg())) as i32)) != 0 { 2 } else { 3 }; continue;
}
// C line 58119
5 => {
vm_block = 4; continue;
}
// C line 58112
6 => {
vm_block = if ((((k) != (stop)) as i32)) != 0 { 10 } else { 5 }; continue;
}
// C line ?
7 => {
let _ = { k = (k).wrapping_add(inc); k };
vm_block = 6; continue;
}
// C line 58115
8 => {
vm_block = 5; continue;
}
// C line 58114
9 => {
let _ = { let assigned = k; res = assigned; assigned };
vm_block = 8; continue;
}
// C line 58113
10 => {
vm_block = if ((((*(pv_10).offset((k) as isize)) == (v_3)) as i32)) != 0 { 9 } else { 7 }; continue;
}
// C line 58111
11 => {
let _ = { let assigned = ((v64) as u64); v_3 = assigned; assigned };
vm_block = 6; continue;
}
// C line ? labels: scan64
12 => {
let _ = { let assigned = ((((*(p)).u).array).u).uint64_ptr; pv_10 = assigned; assigned };
vm_block = 11; continue;
}
// C line 58106
13 => {
vm_block = if (is_bigint) != 0 { 12 } else { 5 }; continue;
}
// C line 58104
14 => {
vm_block = 4; continue;
}
// C line 58102
15 => {
vm_block = 12; continue;
}
// C line 58101
16 => {
vm_block = if (is_bigint) != 0 { 15 } else { 14 }; continue;
}
// C line 58099
17 => {
vm_block = 4; continue;
}
// C line 58084
18 => {
vm_block = if ((((k) != (stop)) as i32)) != 0 { 22 } else { 17 }; continue;
}
// C line ?
19 => {
let _ = { k = (k).wrapping_add(inc); k };
vm_block = 18; continue;
}
// C line 58087
20 => {
vm_block = 17; continue;
}
// C line 58086
21 => {
let _ = { let assigned = k; res = assigned; assigned };
vm_block = 20; continue;
}
// C line 58085
22 => {
vm_block = if (if (((((size_of::<f64>() as usize)) == ((size_of::<f32>() as usize))) as i32)) != 0 { (((*(pv_8).offset((k) as isize)) as f32)).is_nan() as i32 } else { if (((((size_of::<f64>() as usize)) == ((size_of::<f64>() as usize))) as i32)) != 0 { (*(pv_8).offset((k) as isize)).is_nan() as i32 } else { (*(pv_8).offset((k) as isize)).is_nan() as i32 } }) != 0 { 21 } else { 19 }; continue;
}
// C line 58083
23 => {
vm_block = 4; continue;
}
// C line 58082
24 => {
vm_block = if ((((special) != (((1 as i32)).wrapping_neg())) as i32)) != 0 { 23 } else { 18 }; continue;
}
// C line 58080
25 => {
pv_8 = ((((*(p)).u).array).u).double_ptr;
vm_block = 24; continue;
}
// C line 58092
26 => {
vm_block = if ((((k) != (stop)) as i32)) != 0 { 30 } else { 17 }; continue;
}
// C line ?
27 => {
let _ = { k = (k).wrapping_add(inc); k };
vm_block = 26; continue;
}
// C line 58095
28 => {
vm_block = 17; continue;
}
// C line 58094
29 => {
let _ = { let assigned = k; res = assigned; assigned };
vm_block = 28; continue;
}
// C line 58093
30 => {
vm_block = if ((((*(pv_9).offset((k) as isize)) == (d)) as i32)) != 0 { 29 } else { 27 }; continue;
}
// C line 58091
31 => {
pv_9 = ((((*(p)).u).array).u).double_ptr;
vm_block = 26; continue;
}
// C line 58079
32 => {
vm_block = if (if (((((size_of::<f64>() as usize)) == ((size_of::<f32>() as usize))) as i32)) != 0 { (((d) as f32)).is_nan() as i32 } else { if (((((size_of::<f64>() as usize)) == ((size_of::<f64>() as usize))) as i32)) != 0 { (d).is_nan() as i32 } else { (d).is_nan() as i32 } }) != 0 { 25 } else { 31 }; continue;
}
// C line 58078
33 => {
vm_block = 4; continue;
}
// C line 58077
34 => {
vm_block = if (is_bigint) != 0 { 33 } else { 32 }; continue;
}
// C line 58075
35 => {
vm_block = 4; continue;
}
// C line 58060
36 => {
vm_block = if ((((k) != (stop)) as i32)) != 0 { 40 } else { 35 }; continue;
}
// C line ?
37 => {
let _ = { k = (k).wrapping_add(inc); k };
vm_block = 36; continue;
}
// C line 58063
38 => {
vm_block = 35; continue;
}
// C line 58062
39 => {
let _ = { let assigned = k; res = assigned; assigned };
vm_block = 38; continue;
}
// C line 58061
40 => {
vm_block = if (if (((((size_of::<f32>() as usize)) == ((size_of::<f32>() as usize))) as i32)) != 0 { (*(pv_6).offset((k) as isize)).is_nan() as i32 } else { if (((((size_of::<f32>() as usize)) == ((size_of::<f64>() as usize))) as i32)) != 0 { (((*(pv_6).offset((k) as isize)) as f64)).is_nan() as i32 } else { (((*(pv_6).offset((k) as isize)) as f64)).is_nan() as i32 } }) != 0 { 39 } else { 37 }; continue;
}
// C line 58059
41 => {
vm_block = 4; continue;
}
// C line 58058
42 => {
vm_block = if ((((special) != (((1 as i32)).wrapping_neg())) as i32)) != 0 { 41 } else { 36 }; continue;
}
// C line 58056
43 => {
pv_6 = ((((*(p)).u).array).u).float_ptr;
vm_block = 42; continue;
}
// C line 58068
44 => {
vm_block = if ((((k) != (stop)) as i32)) != 0 { 48 } else { 35 }; continue;
}
// C line ?
45 => {
let _ = { k = (k).wrapping_add(inc); k };
vm_block = 44; continue;
}
// C line 58071
46 => {
vm_block = 35; continue;
}
// C line 58070
47 => {
let _ = { let assigned = k; res = assigned; assigned };
vm_block = 46; continue;
}
// C line 58069
48 => {
vm_block = if ((((*(pv_7).offset((k) as isize)) == (f)) as i32)) != 0 { 47 } else { 45 }; continue;
}
// C line 58067
49 => {
pv_7 = ((((*(p)).u).array).u).float_ptr;
vm_block = 44; continue;
}
// C line 58066
50 => {
vm_block = if (((((({ let assigned = ((d) as f32); f = assigned; assigned }) as f64)) == (d)) as i32)) != 0 { 49 } else { 35 }; continue;
}
// C line 58055
51 => {
vm_block = if (if (((((size_of::<f64>() as usize)) == ((size_of::<f32>() as usize))) as i32)) != 0 { (((d) as f32)).is_nan() as i32 } else { if (((((size_of::<f64>() as usize)) == ((size_of::<f64>() as usize))) as i32)) != 0 { (d).is_nan() as i32 } else { (d).is_nan() as i32 } }) != 0 { 43 } else { 50 }; continue;
}
// C line 58054
52 => {
vm_block = 4; continue;
}
// C line 58053
53 => {
vm_block = if (is_bigint) != 0 { 52 } else { 51 }; continue;
}
// C line 58051
54 => {
vm_block = 4; continue;
}
// C line 58027
55 => {
vm_block = if ((((k) != (stop)) as i32)) != 0 { 59 } else { 54 }; continue;
}
// C line ?
56 => {
let _ = { k = (k).wrapping_add(inc); k };
vm_block = 55; continue;
}
// C line 58030
57 => {
vm_block = 54; continue;
}
// C line 58029
58 => {
let _ = { let assigned = k; res = assigned; assigned };
vm_block = 57; continue;
}
// C line 58028
59 => {
vm_block = if (crate::cutils_header::isfp16nan(*(pv_3).offset((k) as isize))) != 0 { 58 } else { 56 }; continue;
}
// C line 58026
60 => {
vm_block = 4; continue;
}
// C line 58025
61 => {
vm_block = if ((((special) != (((1 as i32)).wrapping_neg())) as i32)) != 0 { 60 } else { 55 }; continue;
}
// C line 58023
62 => {
pv_3 = ((((*(p)).u).array).u).fp16_ptr;
vm_block = 61; continue;
}
// C line 58036
63 => {
vm_block = if ((((k) != (stop)) as i32)) != 0 { 67 } else { 54 }; continue;
}
// C line ?
64 => {
let _ = { k = (k).wrapping_add(inc); k };
vm_block = 63; continue;
}
// C line 58039
65 => {
vm_block = 54; continue;
}
// C line 58038
66 => {
let _ = { let assigned = k; res = assigned; assigned };
vm_block = 65; continue;
}
// C line 58037
67 => {
vm_block = if (crate::cutils_header::isfp16zero(*(pv_4).offset((k) as isize))) != 0 { 66 } else { 64 }; continue;
}
// C line 58035
68 => {
pv_4 = ((((*(p)).u).array).u).fp16_ptr;
vm_block = 63; continue;
}
// C line 58044
69 => {
vm_block = if ((((k) != (stop)) as i32)) != 0 { 73 } else { 54 }; continue;
}
// C line ?
70 => {
let _ = { k = (k).wrapping_add(inc); k };
vm_block = 69; continue;
}
// C line 58047
71 => {
vm_block = 54; continue;
}
// C line 58046
72 => {
let _ = { let assigned = k; res = assigned; assigned };
vm_block = 71; continue;
}
// C line 58045
73 => {
vm_block = if ((((((*(pv_5).offset((k) as isize)) as i32)) == (((hf) as i32))) as i32)) != 0 { 72 } else { 70 }; continue;
}
// C line 58043
74 => {
pv_5 = ((((*(p)).u).array).u).fp16_ptr;
vm_block = 69; continue;
}
// C line 58042
75 => {
vm_block = if ({ let _ = { let assigned = crate::cutils_header::tofp16(d); hf = assigned; assigned }; (((d) == (crate::cutils_header::fromfp16(hf))) as i32) }) != 0 { 74 } else { 54 }; continue;
}
// C line 58033
76 => {
vm_block = if ((((d) == ((((0 as i32)) as f64))) as i32)) != 0 { 68 } else { 75 }; continue;
}
// C line 58022
77 => {
vm_block = if (if (((((size_of::<f64>() as usize)) == ((size_of::<f32>() as usize))) as i32)) != 0 { (((d) as f32)).is_nan() as i32 } else { if (((((size_of::<f64>() as usize)) == ((size_of::<f64>() as usize))) as i32)) != 0 { (d).is_nan() as i32 } else { (d).is_nan() as i32 } }) != 0 { 62 } else { 76 }; continue;
}
// C line 58021
78 => {
vm_block = 4; continue;
}
// C line 58020
79 => {
vm_block = if (is_bigint) != 0 { 78 } else { 77 }; continue;
}
// C line 58018
80 => {
vm_block = 4; continue;
}
// C line 58011
81 => {
vm_block = if ((((k) != (stop)) as i32)) != 0 { 85 } else { 80 }; continue;
}
// C line ?
82 => {
let _ = { k = (k).wrapping_add(inc); k };
vm_block = 81; continue;
}
// C line 58014
83 => {
vm_block = 80; continue;
}
// C line 58013
84 => {
let _ = { let assigned = k; res = assigned; assigned };
vm_block = 83; continue;
}
// C line 58012
85 => {
vm_block = if ((((*(pv_2).offset((k) as isize)) == (v_2)) as i32)) != 0 { 84 } else { 82 }; continue;
}
// C line 58010
86 => {
let _ = { let assigned = ((v64) as u32); v_2 = assigned; assigned };
vm_block = 81; continue;
}
// C line ? labels: scan32
87 => {
let _ = { let assigned = ((((*(p)).u).array).u).uint32_ptr; pv_2 = assigned; assigned };
vm_block = 86; continue;
}
// C line 58005
88 => {
vm_block = if (((((is_int) != 0) && (((((((((v64) as u32)) as i64)) == (v64)) as i32)) != 0)) as i32)) != 0 { 87 } else { 80 }; continue;
}
// C line 58003
89 => {
vm_block = 4; continue;
}
// C line 58002
90 => {
vm_block = 87; continue;
}
// C line 58001
91 => {
vm_block = if (((((is_int) != 0) && (((((((((v64) as i32)) as i64)) == (v64)) as i32)) != 0)) as i32)) != 0 { 90 } else { 89 }; continue;
}
// C line 57999
92 => {
vm_block = 4; continue;
}
// C line 57992
93 => {
vm_block = if ((((k) != (stop)) as i32)) != 0 { 97 } else { 92 }; continue;
}
// C line ?
94 => {
let _ = { k = (k).wrapping_add(inc); k };
vm_block = 93; continue;
}
// C line 57995
95 => {
vm_block = 92; continue;
}
// C line 57994
96 => {
let _ = { let assigned = k; res = assigned; assigned };
vm_block = 95; continue;
}
// C line 57993
97 => {
vm_block = if ((((((*(pv_1).offset((k) as isize)) as i32)) == (((v_1) as i32))) as i32)) != 0 { 96 } else { 94 }; continue;
}
// C line 57991
98 => {
let _ = { let assigned = ((v64) as u16); v_1 = assigned; assigned };
vm_block = 93; continue;
}
// C line ? labels: scan16
99 => {
let _ = { let assigned = ((((*(p)).u).array).u).uint16_ptr; pv_1 = assigned; assigned };
vm_block = 98; continue;
}
// C line 57986
100 => {
vm_block = if (((((is_int) != 0) && (((((((((v64) as u16)) as i64)) == (v64)) as i32)) != 0)) as i32)) != 0 { 99 } else { 92 }; continue;
}
// C line 57984
101 => {
vm_block = 4; continue;
}
// C line 57983
102 => {
vm_block = 99; continue;
}
// C line 57982
103 => {
vm_block = if (((((is_int) != 0) && (((((((((v64) as i16)) as i64)) == (v64)) as i32)) != 0)) as i32)) != 0 { 102 } else { 101 }; continue;
}
// C line 57980
104 => {
vm_block = 4; continue;
}
// C line 57970
105 => {
let _ = { let assigned = ((((pp).offset_from(pv) as i64)) as i32); res = assigned; assigned };
vm_block = 104; continue;
}
// C line 57969
106 => {
vm_block = if !(pp).is_null() { 105 } else { 104 }; continue;
}
// C line 57968
107 => {
let _ = { let assigned = ((buffer_memchr((((pv).offset(((k) as isize))) as *const c_void), ((v) as i32), (((len).wrapping_sub(k)) as usize))) as *const u8); pp = assigned; assigned };
vm_block = 106; continue;
}
// C line 57967
108 => {
vm_block = if !(pv).is_null() { 107 } else { 106 }; continue;
}
// C line 57966
109 => {
let _ = { let assigned = core::ptr::null_mut::<u8>(); pp = assigned; assigned };
vm_block = 108; continue;
}
// C line 57972
110 => {
vm_block = if ((((k) != (stop)) as i32)) != 0 { 114 } else { 104 }; continue;
}
// C line ?
111 => {
let _ = { k = (k).wrapping_add(inc); k };
vm_block = 110; continue;
}
// C line 57975
112 => {
vm_block = 104; continue;
}
// C line 57974
113 => {
let _ = { let assigned = k; res = assigned; assigned };
vm_block = 112; continue;
}
// C line 57973
114 => {
vm_block = if ((((((*(pv).offset((k) as isize)) as i32)) == (((v) as i32))) as i32)) != 0 { 113 } else { 111 }; continue;
}
// C line 57965
115 => {
vm_block = if ((((inc) > ((0 as i32))) as i32)) != 0 { 109 } else { 110 }; continue;
}
// C line 57964
116 => {
let _ = { let assigned = ((v64) as u16); v = assigned; assigned };
vm_block = 115; continue;
}
// C line ? labels: scan8
117 => {
let _ = { let assigned = ((((*(p)).u).array).u).uint8_ptr; pv = assigned; assigned };
vm_block = 116; continue;
}
// C line 57959
118 => {
vm_block = if (((((is_int) != 0) && (((((((((v64) as u8)) as i64)) == (v64)) as i32)) != 0)) as i32)) != 0 { 117 } else { 104 }; continue;
}
// C line 57956
119 => {
vm_block = 4; continue;
}
// C line 57955
120 => {
vm_block = 117; continue;
}
// C line 57954
121 => {
vm_block = if (((((is_int) != 0) && (((((((((v64) as i8)) as i64)) == (v64)) as i32)) != 0)) as i32)) != 0 { 120 } else { 119 }; continue;
}
// C line 57952
122 => {
vm_block = match (((*(p)).class_id) as i32) { x if x == (JS_CLASS_BIG_UINT64_ARRAY as i32) => 13, x if x == (JS_CLASS_BIG_INT64_ARRAY as i32) => 16, x if x == (JS_CLASS_FLOAT64_ARRAY as i32) => 34, x if x == (JS_CLASS_FLOAT32_ARRAY as i32) => 53, x if x == (JS_CLASS_FLOAT16_ARRAY as i32) => 79, x if x == (JS_CLASS_UINT32_ARRAY as i32) => 88, x if x == (JS_CLASS_INT32_ARRAY as i32) => 91, x if x == (JS_CLASS_UINT16_ARRAY as i32) => 100, x if x == (JS_CLASS_INT16_ARRAY as i32) => 103, x if x == (JS_CLASS_UINT8_ARRAY as i32) => 118, x if x == (JS_CLASS_UINT8C_ARRAY as i32) => 118, x if x == (JS_CLASS_INT8_ARRAY as i32) => 121, _ => 4, }; continue;
}
// C line 57911
123 => {
let _ = { let assigned = ((v64) as f64); d = assigned; assigned };
vm_block = 122; continue;
}
// C line 57910
124 => {
let _ = { let assigned = ((((((*(argv).offset(((0 as i32)) as isize)).u).uint64) as i32)) as i64); v64 = assigned; assigned };
vm_block = 123; continue;
}
// C line 57909
125 => {
let _ = { let assigned = (1 as i32); is_int = assigned; assigned };
vm_block = 124; continue;
}
// C line 57917
126 => {
let _ = { let assigned = (((((v64) as f64)) == (d)) as i32); is_int = assigned; assigned };
vm_block = 122; continue;
}
// C line 57916
127 => {
let _ = { let assigned = ((d) as i64); v64 = assigned; assigned };
vm_block = 126; continue;
}
// C line 57915
128 => {
vm_block = if ((((((((d) >= ((((((9223372036854775807 as i64)).wrapping_neg()).wrapping_sub((((1 as i32)) as i64))) as f64))) as i32)) != 0) && (((((d) < ((9.2233720368547758E+18 as f64))) as i32)) != 0)) as i32)) != 0 { 127 } else { 122 }; continue;
}
// C line 57914
129 => {
let _ = { let assigned = ((*(argv).offset(((0 as i32)) as isize)).u).float64; d = assigned; assigned };
vm_block = 128; continue;
}
// C line 57947
130 => {
let _ = { let assigned = (1 as i32); is_bigint = assigned; assigned };
vm_block = 122; continue;
}
// C line 57946
131 => {
let _ = { let assigned = (((0 as i32)) as f64); d = assigned; assigned };
vm_block = 130; continue;
}
// C line 57945
132 => {
vm_block = 1; continue;
}
// C line 57944
133 => {
vm_block = if (JS_ToBigInt64(ctx, core::ptr::addr_of_mut!(v64), *(argv).offset(((0 as i32)) as isize))) != 0 { 132 } else { 131 }; continue;
}
// C line 57930
134 => {
vm_block = 4; continue;
}
// C line 57929
135 => {
vm_block = if (((((*(p1)).len) > (((sz) as u32))) as i32)) != 0 { 134 } else { 133 }; continue;
}
// C line 57939
136 => {
vm_block = 4; continue;
}
// C line ?
137 => {
vm_block = if (((((((((*(p1)).len) == ((((sz).wrapping_add((1 as i32))) as u32))) as i32)) != 0) && (((((*(((*(p1)).tab).as_mut_ptr()).offset((sz) as isize)) == ((((0 as i32)) as js_limb_t))) as i32)) != 0)) as i32)) != 0 { 133 } else { 136 }; continue;
}
// C line 57934
138 => {
vm_block = if (((((*(p1)).len) <= (((sz) as u32))) as i32)) != 0 { 133 } else { 137 }; continue;
}
// C line 57933
139 => {
vm_block = 4; continue;
}
// C line 57932
140 => {
vm_block = if (js_bigint_sign(p1)) != 0 { 139 } else { 138 }; continue;
}
// C line 57942
141 => {
vm_block = 4; continue;
}
// C line 57931
142 => {
vm_block = if (((((((*(p)).class_id) as i32)) == ((JS_CLASS_BIG_UINT64_ARRAY as i32))) as i32)) != 0 { 140 } else { 141 }; continue;
}
// C line 57928
143 => {
vm_block = if (((((((*(p)).class_id) as i32)) == ((JS_CLASS_BIG_INT64_ARRAY as i32))) as i32)) != 0 { 135 } else { 142 }; continue;
}
// C line 57924
144 => {
let _ = { let assigned = js_bigint_set_short(core::ptr::addr_of_mut!(buf1), *(argv).offset(((0 as i32)) as isize)); p1 = assigned; assigned };
vm_block = 143; continue;
}
// C line 57926
145 => {
let _ = { let assigned = ((((*(argv).offset(((0 as i32)) as isize)).u).ptr) as *mut JSBigInt); p1 = assigned; assigned };
vm_block = 143; continue;
}
// C line 57923
146 => {
vm_block = if ((((tag) == ((JS_TAG_SHORT_BIG_INT as i32))) as i32)) != 0 { 144 } else { 145 }; continue;
}
// C line 57922
147 => {
sz = (((64 as i32)) / ((64 as i32)));
vm_block = 146; continue;
}
// C line 57949
148 => {
vm_block = 4; continue;
}
// C line 57919
149 => {
vm_block = if ((((((((tag) == ((JS_TAG_BIG_INT as i32))) as i32)) != 0) || (((((tag) == ((JS_TAG_SHORT_BIG_INT as i32))) as i32)) != 0)) as i32)) != 0 { 147 } else { 148 }; continue;
}
// C line 57913
150 => {
vm_block = if ((((tag) == ((JS_TAG_FLOAT64 as i32))) as i32)) != 0 { 129 } else { 149 }; continue;
}
// C line 57908
151 => {
vm_block = if ((((tag) == ((JS_TAG_INT as i32))) as i32)) != 0 { 125 } else { 150 }; continue;
}
// C line 57907
152 => {
let _ = { let assigned = (((*(argv).offset(((0 as i32)) as isize)).tag) as i32); tag = assigned; assigned };
vm_block = 151; continue;
}
// C line 57906
153 => {
let _ = { let assigned = (((0 as i32)) as i64); v64 = assigned; assigned };
vm_block = 152; continue;
}
// C line 57905
154 => {
let _ = { let assigned = (0 as i32); is_int = assigned; assigned };
vm_block = 153; continue;
}
// C line 57904
155 => {
let _ = { let assigned = (0 as i32); is_bigint = assigned; assigned };
vm_block = 154; continue;
}
// C line 57902
156 => {
let _ = { let assigned = min_int(stop, len); stop = assigned; assigned };
vm_block = 155; continue;
}
// C line 57899
157 => {
let _ = { let assigned = min_int(k, (len).wrapping_sub((1 as i32))); k = assigned; assigned };
vm_block = 156; continue;
}
// C line 57901
158 => {
let _ = { let assigned = min_int(k, len); k = assigned; assigned };
vm_block = 156; continue;
}
// C line 57898
159 => {
vm_block = if ((((special) == ((1 as i32))) as i32)) != 0 { 157 } else { 158 }; continue;
}
// C line 57897
160 => {
vm_block = 4; continue;
}
// C line 57896
161 => {
vm_block = if ((((len) == ((0 as i32))) as i32)) != 0 { 160 } else { 159 }; continue;
}
// C line 57895
162 => {
let _ = { let assigned = min_int(len, (((((*(p)).u).array).count) as i32)); len = assigned; assigned };
vm_block = 161; continue;
}
// C line 57891
163 => {
vm_block = 4; continue;
}
// C line 57890
164 => {
let _ = { let assigned = (0 as i32); res = assigned; assigned };
vm_block = 163; continue;
}
// C line 57888
165 => {
vm_block = if ((((((((((((((((((len) as u32)) > ((((*(p)).u).array).count)) as i32)) != 0) && (((((special) == (((1 as i32)).wrapping_neg())) as i32)) != 0)) as i32)) != 0) && ((JS_IsUndefined(*(argv).offset(((0 as i32)) as isize))) != 0)) as i32)) != 0) && (((((k) < (len)) as i32)) != 0)) as i32)) != 0 { 164 } else { 162 }; continue;
}
// C line 57876
166 => {
let _ = { let assigned = ((1 as i32)).wrapping_neg(); inc = assigned; assigned };
vm_block = 165; continue;
}
// C line 57875
167 => {
let _ = { let assigned = ((1 as i32)).wrapping_neg(); stop = assigned; assigned };
vm_block = 166; continue;
}
// C line 57873
168 => {
vm_block = 4; continue;
}
// C line 57872
169 => {
vm_block = if ((((k) < ((0 as i32))) as i32)) != 0 { 168 } else { 167 }; continue;
}
// C line 57871
170 => {
let _ = { let assigned = ((k1) as i32); k = assigned; assigned };
vm_block = 169; continue;
}
// C line 57870
171 => {
vm_block = 1; continue;
}
// C line 57869
172 => {
vm_block = if (JS_ToInt64Clamp(ctx, core::ptr::addr_of_mut!(k1), *(argv).offset(((1 as i32)) as isize), ((((1 as i32)).wrapping_neg()) as i64), (((len).wrapping_sub((1 as i32))) as i64), ((len) as i64))) != 0 { 171 } else { 170 }; continue;
}
// C line 57867
173 => {
vm_block = if ((((argc) > ((1 as i32))) as i32)) != 0 { 172 } else { 167 }; continue;
}
// C line 57866
174 => {
let _ = { let assigned = (len).wrapping_sub((1 as i32)); k = assigned; assigned };
vm_block = 173; continue;
}
// C line 57884
175 => {
let _ = { let assigned = (1 as i32); inc = assigned; assigned };
vm_block = 165; continue;
}
// C line 57883
176 => {
let _ = { let assigned = len; stop = assigned; assigned };
vm_block = 175; continue;
}
// C line 57881
177 => {
vm_block = 1; continue;
}
// C line 57880
178 => {
vm_block = if (JS_ToInt32Clamp(ctx, core::ptr::addr_of_mut!(k), *(argv).offset(((1 as i32)) as isize), (0 as i32), len, len)) != 0 { 177 } else { 176 }; continue;
}
// C line 57879
179 => {
vm_block = if ((((argc) > ((1 as i32))) as i32)) != 0 { 178 } else { 176 }; continue;
}
// C line 57878
180 => {
let _ = { let assigned = (0 as i32); k = assigned; assigned };
vm_block = 179; continue;
}
// C line 57865
181 => {
vm_block = if ((((special) == ((1 as i32))) as i32)) != 0 { 174 } else { 180 }; continue;
}
// C line 57863
182 => {
vm_block = 4; continue;
}
// C line 57862
183 => {
vm_block = if ((((len) == ((0 as i32))) as i32)) != 0 { 182 } else { 181 }; continue;
}
// C line 57860
184 => {
let _ = { let assigned = (((((*(p)).u).array).count) as i32); len = assigned; assigned };
vm_block = 183; continue;
}
// C line 57859
185 => {
return JS_ThrowTypeErrorArrayBufferOOB(ctx);
}
// C line 57858
186 => {
vm_block = if (typed_array_is_oob(p)) != 0 { 185 } else { 184 }; continue;
}
// C line 57857
187 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 57856
188 => {
vm_block = if ((!(!(p).is_null()) as i32)) != 0 { 187 } else { 186 }; continue;
}
// C line 57855
189 => {
let _ = { let assigned = get_typed_array(ctx, this_val); p = assigned; assigned };
vm_block = 188; continue;
}
// C line 57849
190 => {
res = ((1 as i32)).wrapping_neg();
vm_block = 189; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:58132. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_typed_array_join(mut ctx: *mut JSContext, mut this_val: JSValue, mut argc: i32, mut argv: *mut JSValue, mut toLocaleString: i32) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut sep: JSValue = core::mem::zeroed();
let mut el: JSValue = core::mem::zeroed();
let mut b_s: StringBuffer = core::mem::zeroed();
let mut b: *mut StringBuffer = core::mem::zeroed();
let mut s: *mut JSString = core::mem::zeroed();
let mut p: *mut JSObject = core::mem::zeroed();
let mut i: i32 = core::mem::zeroed();
let mut len: i32 = core::mem::zeroed();
let mut oldlen: i32 = core::mem::zeroed();
let mut newlen: i32 = core::mem::zeroed();
let mut c: i32 = core::mem::zeroed();
let mut vm_block: usize = 51;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line ? labels: exception
1 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 58206
2 => {
let _ = JS_FreeValue(ctx, sep);
vm_block = 1; continue;
}
// C line ? labels: fail
3 => {
let _ = string_buffer_free(b);
vm_block = 2; continue;
}
// C line 58202
4 => {
return string_buffer_end(b);
}
// C line 58201
5 => {
let _ = JS_FreeValue(ctx, sep);
vm_block = 4; continue;
}
// C line 58191
6 => {
vm_block = if ((((i) < (oldlen)) as i32)) != 0 { 12 } else { 5 }; continue;
}
// C line ?
7 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 6; continue;
}
// C line 58194
8 => {
vm_block = 3; continue;
}
// C line 58193
9 => {
vm_block = if (string_buffer_putc8(b, ((c) as u32))) != 0 { 8 } else { 7 }; continue;
}
// C line 58197
10 => {
vm_block = 3; continue;
}
// C line 58196
11 => {
vm_block = if (string_buffer_concat(b, s, (((0 as i32)) as u32), (*(s)).len())) != 0 { 10 } else { 7 }; continue;
}
// C line 58192
12 => {
vm_block = if ((((c) >= ((0 as i32))) as i32)) != 0 { 9 } else { 11 }; continue;
}
// C line 58190
13 => {
let _ = { let assigned = max_int((1 as i32), newlen); i = assigned; assigned };
vm_block = 6; continue;
}
// C line 58166
14 => {
vm_block = if ((((i) < (len)) as i32)) != 0 { 29 } else { 13 }; continue;
}
// C line ?
15 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 14; continue;
}
// C line 58185
16 => {
vm_block = 3; continue;
}
// C line 58184
17 => {
vm_block = if (string_buffer_concat_value_free(b, el)) != 0 { 16 } else { 15 }; continue;
}
// C line 58182
18 => {
let _ = { let assigned = JS_ToLocaleStringFree(ctx, el); el = assigned; assigned };
vm_block = 17; continue;
}
// C line 58181
19 => {
vm_block = if (toLocaleString) != 0 { 18 } else { 17 }; continue;
}
// C line 58180
20 => {
vm_block = 3; continue;
}
// C line 58179
21 => {
vm_block = if (JS_IsException(el)) != 0 { 20 } else { 19 }; continue;
}
// C line 58178
22 => {
vm_block = if ((((((!((JS_IsNull(el)) != 0) as i32)) != 0) && (((!((JS_IsUndefined(el)) != 0) as i32)) != 0)) as i32)) != 0 { 21 } else { 15 }; continue;
}
// C line 58176
23 => {
let _ = { let assigned = JS_GetPropertyUint32(ctx, this_val, ((i) as u32)); el = assigned; assigned };
vm_block = 22; continue;
}
// C line 58170
24 => {
vm_block = 3; continue;
}
// C line 58169
25 => {
vm_block = if (string_buffer_putc8(b, ((c) as u32))) != 0 { 24 } else { 23 }; continue;
}
// C line 58173
26 => {
vm_block = 3; continue;
}
// C line 58172
27 => {
vm_block = if (string_buffer_concat(b, s, (((0 as i32)) as u32), (*(s)).len())) != 0 { 26 } else { 23 }; continue;
}
// C line 58168
28 => {
vm_block = if ((((c) >= ((0 as i32))) as i32)) != 0 { 25 } else { 27 }; continue;
}
// C line 58167
29 => {
vm_block = if ((((i) > ((0 as i32))) as i32)) != 0 { 28 } else { 23 }; continue;
}
// C line 58166
30 => {
let _ = { let assigned = (0 as i32); i = assigned; assigned };
vm_block = 14; continue;
}
// C line 58163
31 => {
let _ = string_buffer_init(ctx, b, (0 as i32));
vm_block = 30; continue;
}
// C line 58161
32 => {
let _ = { let assigned = min_int(len, newlen); len = assigned; assigned };
vm_block = 31; continue;
}
// C line 58160
33 => {
let _ = { let assigned = (((((*(p)).u).array).count) as i32); newlen = assigned; assigned };
vm_block = 32; continue;
}
// C line 58156
34 => {
let _ = { let assigned = ((*((((*(s)).u).str8).as_mut_ptr()).offset(((0 as i32)) as isize)) as i32); c = assigned; assigned };
vm_block = 33; continue;
}
// C line 58158
35 => {
let _ = { let assigned = ((1 as i32)).wrapping_neg(); c = assigned; assigned };
vm_block = 33; continue;
}
// C line 58155
36 => {
vm_block = if (((((((((((*(s)).len()) as i32)) == ((1 as i32))) as i32)) != 0) && (((!(((*(s)).is_wide_char()) != 0) as i32)) != 0)) as i32)) != 0 { 34 } else { 35 }; continue;
}
// C line 58154
37 => {
let _ = { let assigned = ((((sep).u).ptr) as *mut JSString); s = assigned; assigned };
vm_block = 36; continue;
}
// C line 58153
38 => {
vm_block = 1; continue;
}
// C line 58152
39 => {
vm_block = if (JS_IsException(sep)) != 0 { 38 } else { 37 }; continue;
}
// C line 58151
40 => {
let _ = { let assigned = JS_ToString(ctx, *(argv).offset(((0 as i32)) as isize)); sep = assigned; assigned };
vm_block = 39; continue;
}
// C line 58150
41 => {
vm_block = if ((((((((((!((toLocaleString) != 0) as i32)) != 0) && (((((argc) > ((0 as i32))) as i32)) != 0)) as i32)) != 0) && (((!((JS_IsUndefined(*(argv).offset(((0 as i32)) as isize))) != 0) as i32)) != 0)) as i32)) != 0 { 40 } else { 31 }; continue;
}
// C line 58149
42 => {
let _ = { let assigned = (44 as i32); c = assigned; assigned };
vm_block = 41; continue;
}
// C line 58147
43 => {
let _ = { let assigned = { let assigned = { let assigned = (((((*(p)).u).array).count) as i32); newlen = assigned; assigned }; oldlen = assigned; assigned }; len = assigned; assigned };
vm_block = 42; continue;
}
// C line 58146
44 => {
return JS_ThrowTypeErrorArrayBufferOOB(ctx);
}
// C line 58145
45 => {
vm_block = if (typed_array_is_oob(p)) != 0 { 44 } else { 43 }; continue;
}
// C line 58144
46 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 58143
47 => {
vm_block = if ((!(!(p).is_null()) as i32)) != 0 { 46 } else { 45 }; continue;
}
// C line 58142
48 => {
let _ = { let assigned = get_typed_array(ctx, this_val); p = assigned; assigned };
vm_block = 47; continue;
}
// C line 58137
49 => {
s = core::ptr::null_mut::<JSString>();
vm_block = 48; continue;
}
// C line 58136
50 => {
b = core::ptr::addr_of_mut!(b_s);
vm_block = 49; continue;
}
// C line 58135
51 => {
sep = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) };
vm_block = 50; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:58211. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_typed_array_reverse(mut ctx: *mut JSContext, mut this_val: JSValue, mut argc: i32, mut argv: *mut JSValue) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut p: *mut JSObject = core::mem::zeroed();
let mut len: i32 = core::mem::zeroed();
let mut p1: *mut u8 = core::mem::zeroed();
let mut p2: *mut u8 = core::mem::zeroed();
let mut v: u8 = core::mem::zeroed();
let mut p1_1: *mut u16 = core::mem::zeroed();
let mut p2_1: *mut u16 = core::mem::zeroed();
let mut v_1: u16 = core::mem::zeroed();
let mut p1_2: *mut u32 = core::mem::zeroed();
let mut p2_2: *mut u32 = core::mem::zeroed();
let mut v_2: u32 = core::mem::zeroed();
let mut p1_3: *mut u64 = core::mem::zeroed();
let mut p2_3: *mut u64 = core::mem::zeroed();
let mut v_3: u64 = core::mem::zeroed();
let mut vm_block: usize = 36;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 58271
1 => {
return JS_DupValue(ctx, this_val);
}
// C line ?
2 => {
let _ = std::process::abort();
vm_block = 1; continue;
}
// C line 58266
3 => {
vm_block = 1; continue;
}
// C line 58260
4 => {
vm_block = if ((((p1_3) < (p2_3)) as i32)) != 0 { 7 } else { 3 }; continue;
}
// C line 58263
5 => {
let _ = { let assigned = v_3; *({ let old = p2_3; p2_3 = (p2_3).offset(-1); old }) = assigned; assigned };
vm_block = 4; continue;
}
// C line 58262
6 => {
let _ = { let assigned = *(p2_3); *({ let old = p1_3; p1_3 = (p1_3).offset(1); old }) = assigned; assigned };
vm_block = 5; continue;
}
// C line 58261
7 => {
v_3 = *(p1_3);
vm_block = 6; continue;
}
// C line 58259
8 => {
p2_3 = ((p1_3).offset(((len) as isize))).offset(-(((1 as i32)) as isize));
vm_block = 4; continue;
}
// C line 58258
9 => {
p1_3 = ((((*(p)).u).array).u).uint64_ptr;
vm_block = 8; continue;
}
// C line 58255
10 => {
vm_block = 1; continue;
}
// C line 58249
11 => {
vm_block = if ((((p1_2) < (p2_2)) as i32)) != 0 { 14 } else { 10 }; continue;
}
// C line 58252
12 => {
let _ = { let assigned = v_2; *({ let old = p2_2; p2_2 = (p2_2).offset(-1); old }) = assigned; assigned };
vm_block = 11; continue;
}
// C line 58251
13 => {
let _ = { let assigned = *(p2_2); *({ let old = p1_2; p1_2 = (p1_2).offset(1); old }) = assigned; assigned };
vm_block = 12; continue;
}
// C line 58250
14 => {
v_2 = *(p1_2);
vm_block = 13; continue;
}
// C line 58248
15 => {
p2_2 = ((p1_2).offset(((len) as isize))).offset(-(((1 as i32)) as isize));
vm_block = 11; continue;
}
// C line 58247
16 => {
p1_2 = ((((*(p)).u).array).u).uint32_ptr;
vm_block = 15; continue;
}
// C line 58244
17 => {
vm_block = 1; continue;
}
// C line 58238
18 => {
vm_block = if ((((p1_1) < (p2_1)) as i32)) != 0 { 21 } else { 17 }; continue;
}
// C line 58241
19 => {
let _ = { let assigned = v_1; *({ let old = p2_1; p2_1 = (p2_1).offset(-1); old }) = assigned; assigned };
vm_block = 18; continue;
}
// C line 58240
20 => {
let _ = { let assigned = *(p2_1); *({ let old = p1_1; p1_1 = (p1_1).offset(1); old }) = assigned; assigned };
vm_block = 19; continue;
}
// C line 58239
21 => {
v_1 = *(p1_1);
vm_block = 20; continue;
}
// C line 58237
22 => {
p2_1 = ((p1_1).offset(((len) as isize))).offset(-(((1 as i32)) as isize));
vm_block = 18; continue;
}
// C line 58236
23 => {
p1_1 = ((((*(p)).u).array).u).uint16_ptr;
vm_block = 22; continue;
}
// C line 58233
24 => {
vm_block = 1; continue;
}
// C line 58227
25 => {
vm_block = if ((((p1) < (p2)) as i32)) != 0 { 28 } else { 24 }; continue;
}
// C line 58230
26 => {
let _ = { let assigned = v; *({ let old = p2; p2 = (p2).offset(-1); old }) = assigned; assigned };
vm_block = 25; continue;
}
// C line 58229
27 => {
let _ = { let assigned = *(p2); *({ let old = p1; p1 = (p1).offset(1); old }) = assigned; assigned };
vm_block = 26; continue;
}
// C line 58228
28 => {
v = *(p1);
vm_block = 27; continue;
}
// C line 58226
29 => {
p2 = ((p1).offset(((len) as isize))).offset(-(((1 as i32)) as isize));
vm_block = 25; continue;
}
// C line 58225
30 => {
p1 = ((((*(p)).u).array).u).uint8_ptr;
vm_block = 29; continue;
}
// C line 58222
31 => {
vm_block = match ((*((typed_array_size_log2).as_ptr()).offset((((((*(p)).class_id) as i32)).wrapping_sub((JS_CLASS_UINT8C_ARRAY as i32))) as isize)) as i32) { x if x == (3 as i32) => 9, x if x == (2 as i32) => 16, x if x == (1 as i32) => 23, x if x == (0 as i32) => 30, _ => 2, }; continue;
}
// C line 58221
32 => {
let _ = { let assigned = ((((this_val).u).ptr) as *mut JSObject); p = assigned; assigned };
vm_block = 31; continue;
}
// C line 58220
33 => {
vm_block = if ((((len) > ((0 as i32))) as i32)) != 0 { 32 } else { 1 }; continue;
}
// C line 58219
34 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 58218
35 => {
vm_block = if ((((len) < ((0 as i32))) as i32)) != 0 { 34 } else { 33 }; continue;
}
// C line 58217
36 => {
let _ = { let assigned = js_typed_array_get_length_unsafe(ctx, this_val); len = assigned; assigned };
vm_block = 35; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:58274. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_typed_array_toReversed(mut ctx: *mut JSContext, mut this_val: JSValue, mut argc: i32, mut argv: *mut JSValue) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut arr: JSValue = core::mem::zeroed();
let mut ret: JSValue = core::mem::zeroed();
let mut p: *mut JSObject = core::mem::zeroed();
let mut vm_block: usize = 9;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 58289
1 => {
return ret;
}
// C line 58288
2 => {
let _ = JS_FreeValue(ctx, arr);
vm_block = 1; continue;
}
// C line 58287
3 => {
let _ = { let assigned = js_typed_array_reverse(ctx, arr, argc, argv); ret = assigned; assigned };
vm_block = 2; continue;
}
// C line 58286
4 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 58285
5 => {
vm_block = if (JS_IsException(arr)) != 0 { 4 } else { 3 }; continue;
}
// C line 58283
6 => {
let _ = { let assigned = js_typed_array_constructor_ta(ctx, JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }, this_val, (((*(p)).class_id) as i32), (((*(p)).u).array).count); arr = assigned; assigned };
vm_block = 5; continue;
}
// C line 58282
7 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 58281
8 => {
vm_block = if ((!(!(p).is_null()) as i32)) != 0 { 7 } else { 6 }; continue;
}
// C line 58280
9 => {
let _ = { let assigned = get_typed_array(ctx, this_val); p = assigned; assigned };
vm_block = 8; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:58292. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn slice_memcpy(mut dst: *mut u8, mut src: *const u8, mut len: usize) -> () {
let mut vm_local_storage = Vec::<u64>::new();
let mut vm_block: usize = 4;
loop { match vm_block {
// C line ?
0 => {
return ();
}
// C line 58296
1 => {
let _ = { let dst = (((dst) as *mut c_void)) as *mut u8; core::ptr::copy_nonoverlapping((((src) as *const c_void)) as *const u8, dst, (len) as usize); dst as *mut c_void };
vm_block = 0; continue;
}
// C line 58299
2 => {
vm_block = if (((({ let old = len; len = (len).wrapping_sub(1); old }) != ((((0 as i32)) as usize))) as i32)) != 0 { 3 } else { 0 }; continue;
}
// C line 58300
3 => {
let _ = { let assigned = *({ let old = src; src = (src).offset(1); old }); *({ let old = dst; dst = (dst).offset(1); old }) = assigned; assigned };
vm_block = 2; continue;
}
// C line 58294
4 => {
vm_block = if (((((((((dst).offset(((len) as isize))) <= (((src) as *mut u8))) as i32)) != 0) || (((((dst) >= ((((src).offset(((len) as isize))) as *mut u8))) as i32)) != 0)) as i32)) != 0 { 1 } else { 2 }; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:58304. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_typed_array_slice(mut ctx: *mut JSContext, mut this_val: JSValue, mut argc: i32, mut argv: *mut JSValue) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut args: [JSValue; 2] = core::mem::zeroed();
let mut arr: JSValue = core::mem::zeroed();
let mut val: JSValue = core::mem::zeroed();
let mut p: *mut JSObject = core::mem::zeroed();
let mut p1: *mut JSObject = core::mem::zeroed();
let mut n: i32 = core::mem::zeroed();
let mut len: i32 = core::mem::zeroed();
let mut start: i32 = core::mem::zeroed();
let mut v_final: i32 = core::mem::zeroed();
let mut count: i32 = core::mem::zeroed();
let mut shift: i32 = core::mem::zeroed();
let mut space: i32 = core::mem::zeroed();
let mut vm_block: usize = 39;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 58364
1 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line ? labels: exception
2 => {
let _ = JS_FreeValue(ctx, arr);
vm_block = 1; continue;
}
// C line 58360
3 => {
return arr;
}
// C line 58346
4 => {
let _ = slice_memcpy(((((*(p1)).u).array).u).uint8_ptr, (((((*(p)).u).array).u).uint8_ptr).offset((((start).wrapping_shl((shift) as u32)) as isize)), (((count).wrapping_shl((shift) as u32)) as usize));
vm_block = 3; continue;
}
// C line 58350
5 => {
vm_block = if ((((n) < (count)) as i32)) != 0 { 11 } else { 3 }; continue;
}
// C line ?
6 => {
let _ = { let old = n; n = (n).wrapping_add(1); old };
vm_block = 5; continue;
}
// C line 58356
7 => {
vm_block = 2; continue;
}
// C line 58354
8 => {
vm_block = if ((((JS_SetPropertyValue(ctx, arr, JS_NewInt32(ctx, n), val, ((1 as i32)).wrapping_shl(((14 as i32)) as u32))) < ((0 as i32))) as i32)) != 0 { 7 } else { 6 }; continue;
}
// C line 58353
9 => {
vm_block = 2; continue;
}
// C line 58352
10 => {
vm_block = if (JS_IsException(val)) != 0 { 9 } else { 8 }; continue;
}
// C line 58351
11 => {
let _ = { let assigned = JS_GetPropertyValue(ctx, this_val, JS_NewInt32(ctx, (start).wrapping_add(n))); val = assigned; assigned };
vm_block = 10; continue;
}
// C line 58350
12 => {
let _ = { let assigned = (0 as i32); n = assigned; assigned };
vm_block = 5; continue;
}
// C line 58345
13 => {
vm_block = if ((((((((p1) != (core::ptr::null_mut::<JSObject>())) as i32)) != 0) && ((((((((*(p)).class_id) as i32)) == ((((*(p1)).class_id) as i32))) as i32)) != 0)) as i32)) != 0 { 4 } else { 12 }; continue;
}
// C line 58344
14 => {
let _ = { let assigned = min_int(count, space); count = assigned; assigned };
vm_block = 13; continue;
}
// C line 58343
15 => {
let _ = { let assigned = max_int((0 as i32), ((((((*(p)).u).array).count).wrapping_sub(((start) as u32))) as i32)); space = assigned; assigned };
vm_block = 14; continue;
}
// C line 58342
16 => {
let _ = { let assigned = get_typed_array(ctx, arr); p1 = assigned; assigned };
vm_block = 15; continue;
}
// C line 58340
17 => {
vm_block = 2; continue;
}
// C line 58338
18 => {
vm_block = if (((((validate_typed_array(ctx, this_val)) != 0) || ((validate_typed_array(ctx, arr)) != 0)) as i32)) != 0 { 17 } else { 16 }; continue;
}
// C line 58337
19 => {
vm_block = if ((((count) > ((0 as i32))) as i32)) != 0 { 18 } else { 3 }; continue;
}
// C line 58335
20 => {
vm_block = 2; continue;
}
// C line 58334
21 => {
vm_block = if (JS_IsException(arr)) != 0 { 20 } else { 19 }; continue;
}
// C line 58333
22 => {
let _ = { let assigned = js_typed_array___speciesCreate(ctx, JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }, (2 as i32), (args).as_mut_ptr()); arr = assigned; assigned };
vm_block = 21; continue;
}
// C line 58332
23 => {
let _ = { let assigned = JS_NewInt32(ctx, count); *((args).as_mut_ptr()).offset(((1 as i32)) as isize) = assigned; assigned };
vm_block = 22; continue;
}
// C line 58331
24 => {
let _ = { let assigned = this_val; *((args).as_mut_ptr()).offset(((0 as i32)) as isize) = assigned; assigned };
vm_block = 23; continue;
}
// C line 58329
25 => {
let _ = { let assigned = ((*((typed_array_size_log2).as_ptr()).offset((((((*(p)).class_id) as i32)).wrapping_sub((JS_CLASS_UINT8C_ARRAY as i32))) as isize)) as i32); shift = assigned; assigned };
vm_block = 24; continue;
}
// C line 58327
26 => {
let _ = { let assigned = max_int((v_final).wrapping_sub(start), (0 as i32)); count = assigned; assigned };
vm_block = 25; continue;
}
// C line 58325
27 => {
vm_block = 2; continue;
}
// C line 58324
28 => {
vm_block = if (JS_ToInt32Clamp(ctx, core::ptr::addr_of_mut!(v_final), *(argv).offset(((1 as i32)) as isize), (0 as i32), len, len)) != 0 { 27 } else { 26 }; continue;
}
// C line 58323
29 => {
vm_block = if ((!((JS_IsUndefined(*(argv).offset(((1 as i32)) as isize))) != 0) as i32)) != 0 { 28 } else { 26 }; continue;
}
// C line 58322
30 => {
let _ = { let assigned = len; v_final = assigned; assigned };
vm_block = 29; continue;
}
// C line 58321
31 => {
vm_block = 2; continue;
}
// C line 58320
32 => {
vm_block = if (JS_ToInt32Clamp(ctx, core::ptr::addr_of_mut!(start), *(argv).offset(((0 as i32)) as isize), (0 as i32), len, len)) != 0 { 31 } else { 30 }; continue;
}
// C line 58318
33 => {
let _ = { let assigned = (((((*(p)).u).array).count) as i32); len = assigned; assigned };
vm_block = 32; continue;
}
// C line 58317
34 => {
return JS_ThrowTypeErrorArrayBufferOOB(ctx);
}
// C line 58316
35 => {
vm_block = if (typed_array_is_oob(p)) != 0 { 34 } else { 33 }; continue;
}
// C line 58315
36 => {
vm_block = 2; continue;
}
// C line 58314
37 => {
vm_block = if ((!(!(p).is_null()) as i32)) != 0 { 36 } else { 35 }; continue;
}
// C line 58313
38 => {
let _ = { let assigned = get_typed_array(ctx, this_val); p = assigned; assigned };
vm_block = 37; continue;
}
// C line 58312
39 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; arr = assigned; assigned };
vm_block = 38; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:58367. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_typed_array_subarray(mut ctx: *mut JSContext, mut this_val: JSValue, mut argc: i32, mut argv: *mut JSValue) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut args: [JSValue; 4] = core::mem::zeroed();
let mut arr: JSValue = core::mem::zeroed();
let mut ta_buffer: JSValue = core::mem::zeroed();
let mut ta: *mut JSTypedArray = core::mem::zeroed();
let mut p: *mut JSObject = core::mem::zeroed();
let mut len: i32 = core::mem::zeroed();
let mut start: i32 = core::mem::zeroed();
let mut v_final: i32 = core::mem::zeroed();
let mut count: i32 = core::mem::zeroed();
let mut shift: i32 = core::mem::zeroed();
let mut offset: i32 = core::mem::zeroed();
let mut is_auto: i32 = core::mem::zeroed();
let mut vm_block: usize = 27;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line ? labels: exception
1 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 58407
2 => {
return arr;
}
// C line 58406
3 => {
let _ = JS_FreeValue(ctx, ta_buffer);
vm_block = 2; continue;
}
// C line 58405
4 => {
let _ = { let assigned = js_typed_array___speciesCreate(ctx, JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }, if (is_auto) != 0 { (3 as i32) } else { (4 as i32) }, (args).as_mut_ptr()); arr = assigned; assigned };
vm_block = 3; continue;
}
// C line 58404
5 => {
let _ = { let assigned = JS_NewInt32(ctx, count); *((args).as_mut_ptr()).offset(((3 as i32)) as isize) = assigned; assigned };
vm_block = 4; continue;
}
// C line 58403
6 => {
let _ = { let assigned = JS_NewInt32(ctx, offset); *((args).as_mut_ptr()).offset(((2 as i32)) as isize) = assigned; assigned };
vm_block = 5; continue;
}
// C line 58402
7 => {
let _ = { let assigned = ta_buffer; *((args).as_mut_ptr()).offset(((1 as i32)) as isize) = assigned; assigned };
vm_block = 6; continue;
}
// C line 58401
8 => {
let _ = { let assigned = this_val; *((args).as_mut_ptr()).offset(((0 as i32)) as isize) = assigned; assigned };
vm_block = 7; continue;
}
// C line 58400
9 => {
vm_block = 1; continue;
}
// C line 58399
10 => {
vm_block = if (JS_IsException(ta_buffer)) != 0 { 9 } else { 8 }; continue;
}
// C line 58398
11 => {
let _ = { let assigned = js_typed_array_get_buffer(ctx, this_val); ta_buffer = assigned; assigned };
vm_block = 10; continue;
}
// C line 58397
12 => {
let _ = { let assigned = max_int((v_final).wrapping_sub(start), (0 as i32)); count = assigned; assigned };
vm_block = 11; continue;
}
// C line 58391
13 => {
let _ = { let assigned = (*(ta)).track_rab; is_auto = assigned; assigned };
vm_block = 12; continue;
}
// C line 58395
14 => {
vm_block = 1; continue;
}
// C line 58394
15 => {
vm_block = if (JS_ToInt32Clamp(ctx, core::ptr::addr_of_mut!(v_final), *(argv).offset(((1 as i32)) as isize), (0 as i32), len, len)) != 0 { 14 } else { 12 }; continue;
}
// C line 58393
16 => {
let _ = { let assigned = (0 as i32); is_auto = assigned; assigned };
vm_block = 15; continue;
}
// C line 58390
17 => {
vm_block = if (JS_IsUndefined(*(argv).offset(((1 as i32)) as isize))) != 0 { 13 } else { 16 }; continue;
}
// C line 58389
18 => {
let _ = { let assigned = len; v_final = assigned; assigned };
vm_block = 17; continue;
}
// C line 58387
19 => {
let _ = { let assigned = ((((*(ta)).offset).wrapping_add((((start).wrapping_shl((shift) as u32)) as u32))) as i32); offset = assigned; assigned };
vm_block = 18; continue;
}
// C line 58385
20 => {
let _ = { let assigned = ((*(p)).u).typed_array; ta = assigned; assigned };
vm_block = 19; continue;
}
// C line 58384
21 => {
let _ = { let assigned = ((*((typed_array_size_log2).as_ptr()).offset((((((*(p)).class_id) as i32)).wrapping_sub((JS_CLASS_UINT8C_ARRAY as i32))) as isize)) as i32); shift = assigned; assigned };
vm_block = 20; continue;
}
// C line 58382
22 => {
vm_block = 1; continue;
}
// C line 58381
23 => {
vm_block = if (JS_ToInt32Clamp(ctx, core::ptr::addr_of_mut!(start), *(argv).offset(((0 as i32)) as isize), (0 as i32), len, len)) != 0 { 22 } else { 21 }; continue;
}
// C line 58380
24 => {
let _ = { let assigned = (((((*(p)).u).array).count) as i32); len = assigned; assigned };
vm_block = 23; continue;
}
// C line 58379
25 => {
vm_block = 1; continue;
}
// C line 58378
26 => {
vm_block = if ((!(!(p).is_null()) as i32)) != 0 { 25 } else { 24 }; continue;
}
// C line 58377
27 => {
let _ = { let assigned = get_typed_array(ctx, this_val); p = assigned; assigned };
vm_block = 26; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:58415. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_cmp_doubles(mut x: f64, mut y: f64) -> i32 {
let mut vm_local_storage = Vec::<u64>::new();
let mut vm_block: usize = 13;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line ?
1 => {
return if (if (((((size_of::<f64>() as usize)) == ((size_of::<f32>() as usize))) as i32)) != 0 { (((y) as f32)).is_sign_negative() as i32 } else { if (((((size_of::<f64>() as usize)) == ((size_of::<f64>() as usize))) as i32)) != 0 { (y).is_sign_negative() as i32 } else { (y).is_sign_negative() as i32 } }) != 0 { (0 as i32) } else { ((1 as i32)).wrapping_neg() };
}
// C line 58423
2 => {
return if (if (((((size_of::<f64>() as usize)) == ((size_of::<f32>() as usize))) as i32)) != 0 { (((y) as f32)).is_sign_negative() as i32 } else { if (((((size_of::<f64>() as usize)) == ((size_of::<f64>() as usize))) as i32)) != 0 { (y).is_sign_negative() as i32 } else { (y).is_sign_negative() as i32 } }) != 0 { (1 as i32) } else { (0 as i32) };
}
// C line 58422
3 => {
vm_block = if (if (((((size_of::<f64>() as usize)) == ((size_of::<f32>() as usize))) as i32)) != 0 { (((x) as f32)).is_sign_negative() as i32 } else { if (((((size_of::<f64>() as usize)) == ((size_of::<f64>() as usize))) as i32)) != 0 { (x).is_sign_negative() as i32 } else { (x).is_sign_negative() as i32 } }) != 0 { 1 } else { 2 }; continue;
}
// C line ?
4 => {
return (0 as i32);
}
// C line 58421
5 => {
vm_block = if ((((x) != ((((0 as i32)) as f64))) as i32)) != 0 { 4 } else { 3 }; continue;
}
// C line ?
6 => {
return (1 as i32);
}
// C line 58420
7 => {
vm_block = if ((((x) > (y)) as i32)) != 0 { 6 } else { 5 }; continue;
}
// C line ?
8 => {
return ((1 as i32)).wrapping_neg();
}
// C line 58419
9 => {
vm_block = if ((((x) < (y)) as i32)) != 0 { 8 } else { 7 }; continue;
}
// C line ?
10 => {
return ((1 as i32)).wrapping_neg();
}
// C line 58418
11 => {
vm_block = if (if (((((size_of::<f64>() as usize)) == ((size_of::<f32>() as usize))) as i32)) != 0 { (((y) as f32)).is_nan() as i32 } else { if (((((size_of::<f64>() as usize)) == ((size_of::<f64>() as usize))) as i32)) != 0 { (y).is_nan() as i32 } else { (y).is_nan() as i32 } }) != 0 { 10 } else { 9 }; continue;
}
// C line ?
12 => {
return if (if (((((size_of::<f64>() as usize)) == ((size_of::<f32>() as usize))) as i32)) != 0 { (((y) as f32)).is_nan() as i32 } else { if (((((size_of::<f64>() as usize)) == ((size_of::<f64>() as usize))) as i32)) != 0 { (y).is_nan() as i32 } else { (y).is_nan() as i32 } }) != 0 { (0 as i32) } else { (1 as i32) };
}
// C line 58417
13 => {
vm_block = if (if (((((size_of::<f64>() as usize)) == ((size_of::<f32>() as usize))) as i32)) != 0 { (((x) as f32)).is_nan() as i32 } else { if (((((size_of::<f64>() as usize)) == ((size_of::<f64>() as usize))) as i32)) != 0 { (x).is_nan() as i32 } else { (x).is_nan() as i32 } }) != 0 { 12 } else { 11 }; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:58426. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_TA_cmp_int8(mut a: *const c_void, mut b: *const c_void, mut opaque: *mut c_void) -> i32 {
let mut vm_local_storage = Vec::<u64>::new();
let mut vm_block: usize = 1;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 58427
1 => {
return (((*(((a) as *const i8))) as i32)).wrapping_sub(((*(((b) as *const i8))) as i32));
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:58430. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_TA_cmp_uint8(mut a: *const c_void, mut b: *const c_void, mut opaque: *mut c_void) -> i32 {
let mut vm_local_storage = Vec::<u64>::new();
let mut vm_block: usize = 1;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 58431
1 => {
return (((*(((a) as *const u8))) as i32)).wrapping_sub(((*(((b) as *const u8))) as i32));
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:58434. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_TA_cmp_int16(mut a: *const c_void, mut b: *const c_void, mut opaque: *mut c_void) -> i32 {
let mut vm_local_storage = Vec::<u64>::new();
let mut vm_block: usize = 1;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 58435
1 => {
return (((*(((a) as *const i16))) as i32)).wrapping_sub(((*(((b) as *const i16))) as i32));
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:58438. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_TA_cmp_uint16(mut a: *const c_void, mut b: *const c_void, mut opaque: *mut c_void) -> i32 {
let mut vm_local_storage = Vec::<u64>::new();
let mut vm_block: usize = 1;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 58439
1 => {
return (((*(((a) as *const u16))) as i32)).wrapping_sub(((*(((b) as *const u16))) as i32));
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:58442. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_TA_cmp_int32(mut a: *const c_void, mut b: *const c_void, mut opaque: *mut c_void) -> i32 {
let mut vm_local_storage = Vec::<u64>::new();
let mut x: i32 = core::mem::zeroed();
let mut y: i32 = core::mem::zeroed();
let mut vm_block: usize = 3;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 58445
1 => {
return ((((y) < (x)) as i32)).wrapping_sub((((y) > (x)) as i32));
}
// C line 58444
2 => {
y = *(((b) as *const i32));
vm_block = 1; continue;
}
// C line 58443
3 => {
x = *(((a) as *const i32));
vm_block = 2; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:58448. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_TA_cmp_uint32(mut a: *const c_void, mut b: *const c_void, mut opaque: *mut c_void) -> i32 {
let mut vm_local_storage = Vec::<u64>::new();
let mut x: u32 = core::mem::zeroed();
let mut y: u32 = core::mem::zeroed();
let mut vm_block: usize = 3;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 58451
1 => {
return ((((y) < (x)) as i32)).wrapping_sub((((y) > (x)) as i32));
}
// C line 58450
2 => {
y = *(((b) as *const u32));
vm_block = 1; continue;
}
// C line 58449
3 => {
x = *(((a) as *const u32));
vm_block = 2; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:58454. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_TA_cmp_int64(mut a: *const c_void, mut b: *const c_void, mut opaque: *mut c_void) -> i32 {
let mut vm_local_storage = Vec::<u64>::new();
let mut x: i64 = core::mem::zeroed();
let mut y: i64 = core::mem::zeroed();
let mut vm_block: usize = 3;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 58457
1 => {
return ((((y) < (x)) as i32)).wrapping_sub((((y) > (x)) as i32));
}
// C line 58456
2 => {
y = *(((b) as *const i64));
vm_block = 1; continue;
}
// C line 58455
3 => {
x = *(((a) as *const i64));
vm_block = 2; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:58460. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_TA_cmp_uint64(mut a: *const c_void, mut b: *const c_void, mut opaque: *mut c_void) -> i32 {
let mut vm_local_storage = Vec::<u64>::new();
let mut x: u64 = core::mem::zeroed();
let mut y: u64 = core::mem::zeroed();
let mut vm_block: usize = 3;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 58463
1 => {
return ((((y) < (x)) as i32)).wrapping_sub((((y) > (x)) as i32));
}
// C line 58462
2 => {
y = *(((b) as *const u64));
vm_block = 1; continue;
}
// C line 58461
3 => {
x = *(((a) as *const u64));
vm_block = 2; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:58466. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_TA_cmp_float16(mut a: *const c_void, mut b: *const c_void, mut opaque: *mut c_void) -> i32 {
let mut vm_local_storage = Vec::<u64>::new();
let mut vm_block: usize = 1;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 58467
1 => {
return js_cmp_doubles(crate::cutils_header::fromfp16(*(((a) as *const u16))), crate::cutils_header::fromfp16(*(((b) as *const u16))));
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:58471. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_TA_cmp_float32(mut a: *const c_void, mut b: *const c_void, mut opaque: *mut c_void) -> i32 {
let mut vm_local_storage = Vec::<u64>::new();
let mut vm_block: usize = 1;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 58472
1 => {
return js_cmp_doubles(((*(((a) as *const f32))) as f64), ((*(((b) as *const f32))) as f64));
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:58475. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_TA_cmp_float64(mut a: *const c_void, mut b: *const c_void, mut opaque: *mut c_void) -> i32 {
let mut vm_local_storage = Vec::<u64>::new();
let mut vm_block: usize = 1;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 58476
1 => {
return js_cmp_doubles(*(((a) as *const f64)), *(((b) as *const f64)));
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:58479. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_TA_get_int8(mut ctx: *mut JSContext, mut a: *const c_void) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut vm_block: usize = 1;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 58480
1 => {
return JS_NewInt32(ctx, ((*(((a) as *const i8))) as i32));
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:58483. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_TA_get_uint8(mut ctx: *mut JSContext, mut a: *const c_void) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut vm_block: usize = 1;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 58484
1 => {
return JS_NewInt32(ctx, ((*(((a) as *const u8))) as i32));
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:58487. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_TA_get_int16(mut ctx: *mut JSContext, mut a: *const c_void) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut vm_block: usize = 1;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 58488
1 => {
return JS_NewInt32(ctx, ((*(((a) as *const i16))) as i32));
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:58491. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_TA_get_uint16(mut ctx: *mut JSContext, mut a: *const c_void) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut vm_block: usize = 1;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 58492
1 => {
return JS_NewInt32(ctx, ((*(((a) as *const u16))) as i32));
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:58495. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_TA_get_int32(mut ctx: *mut JSContext, mut a: *const c_void) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut vm_block: usize = 1;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 58496
1 => {
return JS_NewInt32(ctx, *(((a) as *const i32)));
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:58499. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_TA_get_uint32(mut ctx: *mut JSContext, mut a: *const c_void) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut vm_block: usize = 1;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 58500
1 => {
return JS_NewUint32(ctx, *(((a) as *const u32)));
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:58503. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_TA_get_int64(mut ctx: *mut JSContext, mut a: *const c_void) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut vm_block: usize = 1;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 58504
1 => {
return JS_NewBigInt64(ctx, *(((a) as *mut i64)));
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:58507. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_TA_get_uint64(mut ctx: *mut JSContext, mut a: *const c_void) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut vm_block: usize = 1;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 58508
1 => {
return JS_NewBigUint64(ctx, *(((a) as *mut u64)));
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:58511. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_TA_get_float16(mut ctx: *mut JSContext, mut a: *const c_void) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut vm_block: usize = 1;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 58512
1 => {
return __JS_NewFloat64(ctx, crate::cutils_header::fromfp16(*(((a) as *const u16))));
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:58515. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_TA_get_float32(mut ctx: *mut JSContext, mut a: *const c_void) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut vm_block: usize = 1;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 58516
1 => {
return __JS_NewFloat64(ctx, ((*(((a) as *const f32))) as f64));
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:58519. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_TA_get_float64(mut ctx: *mut JSContext, mut a: *const c_void) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut vm_block: usize = 1;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 58520
1 => {
return __JS_NewFloat64(ctx, *(((a) as *const f64)));
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:58532. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_TA_cmp_generic(mut a: *const c_void, mut b: *const c_void, mut opaque: *mut c_void) -> i32 {
let mut vm_local_storage = Vec::<u64>::new();
let mut psc: *mut TA_sort_context = core::mem::zeroed();
let mut ctx: *mut JSContext = core::mem::zeroed();
let mut a_idx: u32 = core::mem::zeroed();
let mut b_idx: u32 = core::mem::zeroed();
let mut argv: [JSValue; 2] = core::mem::zeroed();
let mut res: JSValue = core::mem::zeroed();
let mut cmp: i32 = core::mem::zeroed();
let mut val: i32 = core::mem::zeroed();
let mut val_1: f64 = core::mem::zeroed();
let mut vm_block: usize = 24;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 58575
1 => {
return cmp;
}
// C line 58573
2 => {
let _ = JS_FreeValue(ctx, *((argv).as_mut_ptr()).offset(((1 as i32)) as isize));
vm_block = 1; continue;
}
// C line ? labels: done
3 => {
let _ = JS_FreeValue(ctx, *((argv).as_mut_ptr()).offset(((0 as i32)) as isize));
vm_block = 2; continue;
}
// C line 58569
4 => {
let _ = { let assigned = ((((a_idx) > (b_idx)) as i32)).wrapping_sub((((a_idx) < (b_idx)) as i32)); cmp = assigned; assigned };
vm_block = 3; continue;
}
// C line 58567
5 => {
vm_block = if ((((cmp) == ((0 as i32))) as i32)) != 0 { 4 } else { 3 }; continue;
}
// C line 58557
6 => {
let _ = { let assigned = ((((val) > ((0 as i32))) as i32)).wrapping_sub((((val) < ((0 as i32))) as i32)); cmp = assigned; assigned };
vm_block = 5; continue;
}
// C line 58556
7 => {
val = ((((res).u).uint64) as i32);
vm_block = 6; continue;
}
// C line 58562
8 => {
vm_block = 3; continue;
}
// C line 58561
9 => {
let _ = { let assigned = (1 as i32); (*(psc)).exception = assigned; assigned };
vm_block = 8; continue;
}
// C line 58564
10 => {
let _ = { let assigned = ((((val_1) > ((((0 as i32)) as f64))) as i32)).wrapping_sub((((val_1) < ((((0 as i32)) as f64))) as i32)); cmp = assigned; assigned };
vm_block = 5; continue;
}
// C line 58560
11 => {
vm_block = if ((((JS_ToFloat64Free(ctx, core::ptr::addr_of_mut!(val_1), res)) < ((0 as i32))) as i32)) != 0 { 9 } else { 10 }; continue;
}
// C line 58555
12 => {
vm_block = if (((((((res).tag) as i32)) == ((JS_TAG_INT as i32))) as i32)) != 0 { 7 } else { 11 }; continue;
}
// C line 58553
13 => {
vm_block = 3; continue;
}
// C line 58552
14 => {
let _ = { let assigned = (1 as i32); (*(psc)).exception = assigned; assigned };
vm_block = 13; continue;
}
// C line 58551
15 => {
vm_block = if (JS_IsException(res)) != 0 { 14 } else { 12 }; continue;
}
// C line 58550
16 => {
let _ = { let assigned = JS_Call(ctx, (*(psc)).cmp, JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }, (2 as i32), (argv).as_mut_ptr()); res = assigned; assigned };
vm_block = 15; continue;
}
// C line 58548
17 => {
let _ = { let assigned = ((*(psc)).getfun).expect("registered buffer callback")(ctx, ((((*(psc)).array).offset((((((b_idx) as usize)).wrapping_mul((((*(psc)).elt_size) as usize))) as isize))) as *const c_void)); *((argv).as_mut_ptr()).offset(((1 as i32)) as isize) = assigned; assigned };
vm_block = 16; continue;
}
// C line 58546
18 => {
let _ = { let assigned = ((*(psc)).getfun).expect("registered buffer callback")(ctx, ((((*(psc)).array).offset((((((a_idx) as usize)).wrapping_mul((((*(psc)).elt_size) as usize))) as isize))) as *const c_void)); *((argv).as_mut_ptr()).offset(((0 as i32)) as isize) = assigned; assigned };
vm_block = 17; continue;
}
// C line 58545
19 => {
let _ = { let assigned = *(((b) as *mut u32)); b_idx = assigned; assigned };
vm_block = 18; continue;
}
// C line 58544
20 => {
let _ = { let assigned = *(((a) as *mut u32)); a_idx = assigned; assigned };
vm_block = 19; continue;
}
// C line 58541
21 => {
vm_block = if ((!(((*(psc)).exception) != 0) as i32)) != 0 { 20 } else { 1 }; continue;
}
// C line 58540
22 => {
let _ = { let assigned = (0 as i32); cmp = assigned; assigned };
vm_block = 21; continue;
}
// C line 58534
23 => {
ctx = (*(psc)).ctx;
vm_block = 22; continue;
}
// C line 58533
24 => {
psc = ((opaque) as *mut TA_sort_context);
vm_block = 23; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:58578. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_typed_array_sort(mut ctx: *mut JSContext, mut this_val: JSValue, mut argc: i32, mut argv: *mut JSValue) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut p: *mut JSObject = core::mem::zeroed();
let mut len: i32 = core::mem::zeroed();
let mut elt_size: usize = core::mem::zeroed();
let mut tsc: TA_sort_context = core::mem::zeroed();
let mut cmpfun: Option<unsafe fn(*const c_void, *const c_void, *mut c_void) -> i32> = core::mem::zeroed();
let mut array_idx: *mut u32 = core::mem::zeroed();
let mut array: *mut c_void = core::mem::zeroed();
let mut i: usize = core::mem::zeroed();
let mut j: usize = core::mem::zeroed();
let mut array_ptr: *mut c_void = core::mem::zeroed();
let mut vm_block: usize = 101;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 58720
1 => {
return JS_DupValue(ctx, this_val);
}
// C line 58713
2 => {
let _ = js_free(ctx, array);
vm_block = 1; continue;
}
// C line 58712
3 => {
let _ = js_free(ctx, ((array_idx) as *mut c_void));
vm_block = 2; continue;
}
// C line 58677
4 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 58676
5 => {
let _ = js_free(ctx, array);
vm_block = 4; continue;
}
// C line 58675
6 => {
let _ = js_free(ctx, ((array_idx) as *mut c_void));
vm_block = 5; continue;
}
// C line 58674
7 => {
vm_block = if (((((tsc).exception) == ((1 as i32))) as i32)) != 0 { 6 } else { 3 }; continue;
}
// C line ?
8 => {
let _ = std::process::abort();
vm_block = 3; continue;
}
// C line 58707
9 => {
vm_block = 3; continue;
}
// C line 58703
10 => {
vm_block = if ((((i) < (((len) as usize))) as i32)) != 0 { 13 } else { 9 }; continue;
}
// C line ?
11 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 10; continue;
}
// C line 58705
12 => {
let _ = { let assigned = *(((array) as *mut u64)).offset((j) as isize); *(((array_ptr) as *mut u64)).offset((i) as isize) = assigned; assigned };
vm_block = 11; continue;
}
// C line 58704
13 => {
let _ = { let assigned = ((*(array_idx).offset((i) as isize)) as usize); j = assigned; assigned };
vm_block = 12; continue;
}
// C line 58703
14 => {
let _ = { let assigned = (((0 as i32)) as usize); i = assigned; assigned };
vm_block = 10; continue;
}
// C line 58701
15 => {
vm_block = 3; continue;
}
// C line 58697
16 => {
vm_block = if ((((i) < (((len) as usize))) as i32)) != 0 { 19 } else { 15 }; continue;
}
// C line ?
17 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 16; continue;
}
// C line 58699
18 => {
let _ = { let assigned = *(((array) as *mut u32)).offset((j) as isize); *(((array_ptr) as *mut u32)).offset((i) as isize) = assigned; assigned };
vm_block = 17; continue;
}
// C line 58698
19 => {
let _ = { let assigned = ((*(array_idx).offset((i) as isize)) as usize); j = assigned; assigned };
vm_block = 18; continue;
}
// C line 58697
20 => {
let _ = { let assigned = (((0 as i32)) as usize); i = assigned; assigned };
vm_block = 16; continue;
}
// C line 58695
21 => {
vm_block = 3; continue;
}
// C line 58691
22 => {
vm_block = if ((((i) < (((len) as usize))) as i32)) != 0 { 25 } else { 21 }; continue;
}
// C line ?
23 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 22; continue;
}
// C line 58693
24 => {
let _ = { let assigned = *(((array) as *mut u16)).offset((j) as isize); *(((array_ptr) as *mut u16)).offset((i) as isize) = assigned; assigned };
vm_block = 23; continue;
}
// C line 58692
25 => {
let _ = { let assigned = ((*(array_idx).offset((i) as isize)) as usize); j = assigned; assigned };
vm_block = 24; continue;
}
// C line 58691
26 => {
let _ = { let assigned = (((0 as i32)) as usize); i = assigned; assigned };
vm_block = 22; continue;
}
// C line 58689
27 => {
vm_block = 3; continue;
}
// C line 58685
28 => {
vm_block = if ((((i) < (((len) as usize))) as i32)) != 0 { 31 } else { 27 }; continue;
}
// C line ?
29 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 28; continue;
}
// C line 58687
30 => {
let _ = { let assigned = *(((array) as *mut u8)).offset((j) as isize); *(((array_ptr) as *mut u8)).offset((i) as isize) = assigned; assigned };
vm_block = 29; continue;
}
// C line 58686
31 => {
let _ = { let assigned = ((*(array_idx).offset((i) as isize)) as usize); j = assigned; assigned };
vm_block = 30; continue;
}
// C line 58685
32 => {
let _ = { let assigned = (((0 as i32)) as usize); i = assigned; assigned };
vm_block = 28; continue;
}
// C line 58683
33 => {
vm_block = match elt_size { x if x == (((8 as i32)) as usize) => 14, x if x == (((4 as i32)) as usize) => 20, x if x == (((2 as i32)) as usize) => 26, x if x == (((1 as i32)) as usize) => 32, _ => 8, }; continue;
}
// C line 58682
34 => {
let _ = { let assigned = min_int(len, (((((*(p)).u).array).count) as i32)); len = assigned; assigned };
vm_block = 33; continue;
}
// C line 58681
35 => {
array_ptr = ((((*(p)).u).array).u).ptr;
vm_block = 34; continue;
}
// C line 58673
36 => {
vm_block = if ((tsc).exception) != 0 { 7 } else { 35 }; continue;
}
// C line 58671
37 => {
let _ = crate::cutils::rqsort(((array_idx) as *mut c_void), ((len) as usize), (size_of::<u32>() as usize), js_TA_cmp_generic, ((core::ptr::addr_of_mut!(tsc)) as *mut c_void));
vm_block = 36; continue;
}
// C line 58670
38 => {
let _ = { let assigned = ((array) as *mut u8); (tsc).array = assigned; assigned };
vm_block = 37; continue;
}
// C line 58669
39 => {
let _ = { let assigned = ((elt_size) as i32); (tsc).elt_size = assigned; assigned };
vm_block = 38; continue;
}
// C line 58667
40 => {
vm_block = if ((((i) < (((len) as usize))) as i32)) != 0 { 42 } else { 39 }; continue;
}
// C line ?
41 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 40; continue;
}
// C line 58668
42 => {
let _ = { let assigned = ((i) as u32); *(array_idx).offset((i) as isize) = assigned; assigned };
vm_block = 41; continue;
}
// C line 58667
43 => {
let _ = { let assigned = (((0 as i32)) as usize); i = assigned; assigned };
vm_block = 40; continue;
}
// C line 58665
44 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 58664
45 => {
let _ = js_free(ctx, array);
vm_block = 44; continue;
}
// C line 58663
46 => {
vm_block = if ((!(!(array_idx).is_null()) as i32)) != 0 { 45 } else { 43 }; continue;
}
// C line 58662
47 => {
let _ = { let assigned = ((js_malloc(ctx, (((len) as usize)).wrapping_mul((size_of::<u32>() as usize)))) as *mut u32); array_idx = assigned; assigned };
vm_block = 46; continue;
}
// C line 58659
48 => {
let _ = { let dst = (array) as *mut u8; core::ptr::copy_nonoverlapping((((((*(p)).u).array).u).ptr) as *const u8, dst, ((((len) as usize)).wrapping_mul(elt_size)) as usize); dst as *mut c_void };
vm_block = 47; continue;
}
// C line 58658
49 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 58657
50 => {
vm_block = if ((!(!(array).is_null()) as i32)) != 0 { 49 } else { 48 }; continue;
}
// C line 58656
51 => {
let _ = { let assigned = js_malloc(ctx, (((len) as usize)).wrapping_mul(elt_size)); array = assigned; assigned };
vm_block = 50; continue;
}
// C line 58717
52 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 58716
53 => {
vm_block = if ((tsc).exception) != 0 { 52 } else { 1 }; continue;
}
// C line 58715
54 => {
let _ = crate::cutils::rqsort(((((*(p)).u).array).u).ptr, ((len) as usize), elt_size, cmpfun.expect("selected sort comparator"), ((core::ptr::addr_of_mut!(tsc)) as *mut c_void));
vm_block = 53; continue;
}
// C line 58649
55 => {
vm_block = if ((!((JS_IsUndefined((tsc).cmp)) != 0) as i32)) != 0 { 51 } else { 54 }; continue;
}
// C line 58648
56 => {
let _ = { let assigned = ((((1 as i32)).wrapping_shl((((*((typed_array_size_log2).as_ptr()).offset((((((*(p)).class_id) as i32)).wrapping_sub((JS_CLASS_UINT8C_ARRAY as i32))) as isize)) as i32)) as u32)) as usize); elt_size = assigned; assigned };
vm_block = 55; continue;
}
// C line ?
57 => {
let _ = std::process::abort();
vm_block = 56; continue;
}
// C line 58644
58 => {
vm_block = 56; continue;
}
// C line 58643
59 => {
let _ = { let assigned = js_TA_cmp_float64; cmpfun = Some(assigned); assigned };
vm_block = 58; continue;
}
// C line 58642
60 => {
let _ = { let assigned = js_TA_get_float64; (tsc).getfun = Some(assigned); assigned };
vm_block = 59; continue;
}
// C line 58640
61 => {
vm_block = 56; continue;
}
// C line 58639
62 => {
let _ = { let assigned = js_TA_cmp_float32; cmpfun = Some(assigned); assigned };
vm_block = 61; continue;
}
// C line 58638
63 => {
let _ = { let assigned = js_TA_get_float32; (tsc).getfun = Some(assigned); assigned };
vm_block = 62; continue;
}
// C line 58636
64 => {
vm_block = 56; continue;
}
// C line 58635
65 => {
let _ = { let assigned = js_TA_cmp_float16; cmpfun = Some(assigned); assigned };
vm_block = 64; continue;
}
// C line 58634
66 => {
let _ = { let assigned = js_TA_get_float16; (tsc).getfun = Some(assigned); assigned };
vm_block = 65; continue;
}
// C line 58632
67 => {
vm_block = 56; continue;
}
// C line 58631
68 => {
let _ = { let assigned = js_TA_cmp_uint64; cmpfun = Some(assigned); assigned };
vm_block = 67; continue;
}
// C line 58630
69 => {
let _ = { let assigned = js_TA_get_uint64; (tsc).getfun = Some(assigned); assigned };
vm_block = 68; continue;
}
// C line 58628
70 => {
vm_block = 56; continue;
}
// C line 58627
71 => {
let _ = { let assigned = js_TA_cmp_int64; cmpfun = Some(assigned); assigned };
vm_block = 70; continue;
}
// C line 58626
72 => {
let _ = { let assigned = js_TA_get_int64; (tsc).getfun = Some(assigned); assigned };
vm_block = 71; continue;
}
// C line 58624
73 => {
vm_block = 56; continue;
}
// C line 58623
74 => {
let _ = { let assigned = js_TA_cmp_uint32; cmpfun = Some(assigned); assigned };
vm_block = 73; continue;
}
// C line 58622
75 => {
let _ = { let assigned = js_TA_get_uint32; (tsc).getfun = Some(assigned); assigned };
vm_block = 74; continue;
}
// C line 58620
76 => {
vm_block = 56; continue;
}
// C line 58619
77 => {
let _ = { let assigned = js_TA_cmp_int32; cmpfun = Some(assigned); assigned };
vm_block = 76; continue;
}
// C line 58618
78 => {
let _ = { let assigned = js_TA_get_int32; (tsc).getfun = Some(assigned); assigned };
vm_block = 77; continue;
}
// C line 58616
79 => {
vm_block = 56; continue;
}
// C line 58615
80 => {
let _ = { let assigned = js_TA_cmp_uint16; cmpfun = Some(assigned); assigned };
vm_block = 79; continue;
}
// C line 58614
81 => {
let _ = { let assigned = js_TA_get_uint16; (tsc).getfun = Some(assigned); assigned };
vm_block = 80; continue;
}
// C line 58612
82 => {
vm_block = 56; continue;
}
// C line 58611
83 => {
let _ = { let assigned = js_TA_cmp_int16; cmpfun = Some(assigned); assigned };
vm_block = 82; continue;
}
// C line 58610
84 => {
let _ = { let assigned = js_TA_get_int16; (tsc).getfun = Some(assigned); assigned };
vm_block = 83; continue;
}
// C line 58608
85 => {
vm_block = 56; continue;
}
// C line 58607
86 => {
let _ = { let assigned = js_TA_cmp_uint8; cmpfun = Some(assigned); assigned };
vm_block = 85; continue;
}
// C line 58606
87 => {
let _ = { let assigned = js_TA_get_uint8; (tsc).getfun = Some(assigned); assigned };
vm_block = 86; continue;
}
// C line 58603
88 => {
vm_block = 56; continue;
}
// C line 58602
89 => {
let _ = { let assigned = js_TA_cmp_int8; cmpfun = Some(assigned); assigned };
vm_block = 88; continue;
}
// C line 58601
90 => {
let _ = { let assigned = js_TA_get_int8; (tsc).getfun = Some(assigned); assigned };
vm_block = 89; continue;
}
// C line 58599
91 => {
vm_block = match (((*(p)).class_id) as i32) { x if x == (JS_CLASS_FLOAT64_ARRAY as i32) => 60, x if x == (JS_CLASS_FLOAT32_ARRAY as i32) => 63, x if x == (JS_CLASS_FLOAT16_ARRAY as i32) => 66, x if x == (JS_CLASS_BIG_UINT64_ARRAY as i32) => 69, x if x == (JS_CLASS_BIG_INT64_ARRAY as i32) => 72, x if x == (JS_CLASS_UINT32_ARRAY as i32) => 75, x if x == (JS_CLASS_INT32_ARRAY as i32) => 78, x if x == (JS_CLASS_UINT16_ARRAY as i32) => 81, x if x == (JS_CLASS_INT16_ARRAY as i32) => 84, x if x == (JS_CLASS_UINT8_ARRAY as i32) => 87, x if x == (JS_CLASS_UINT8C_ARRAY as i32) => 87, x if x == (JS_CLASS_INT8_ARRAY as i32) => 90, _ => 57, }; continue;
}
// C line 58598
92 => {
let _ = { let assigned = ((((this_val).u).ptr) as *mut JSObject); p = assigned; assigned };
vm_block = 91; continue;
}
// C line 58597
93 => {
vm_block = if ((((len) > ((1 as i32))) as i32)) != 0 { 92 } else { 1 }; continue;
}
// C line 58595
94 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 58594
95 => {
vm_block = if ((((len) < ((0 as i32))) as i32)) != 0 { 94 } else { 93 }; continue;
}
// C line 58593
96 => {
let _ = { let assigned = js_typed_array_get_length_unsafe(ctx, this_val); len = assigned; assigned };
vm_block = 95; continue;
}
// C line 58592
97 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 58591
98 => {
vm_block = if ((((((!((JS_IsUndefined((tsc).cmp)) != 0) as i32)) != 0) && ((check_function(ctx, (tsc).cmp)) != 0)) as i32)) != 0 { 97 } else { 96 }; continue;
}
// C line 58589
99 => {
let _ = { let assigned = *(argv).offset(((0 as i32)) as isize); (tsc).cmp = assigned; assigned };
vm_block = 98; continue;
}
// C line 58588
100 => {
let _ = { let assigned = (0 as i32); (tsc).exception = assigned; assigned };
vm_block = 99; continue;
}
// C line 58587
101 => {
let _ = { let assigned = ctx; (tsc).ctx = assigned; assigned };
vm_block = 100; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:58723. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_typed_array_toSorted(mut ctx: *mut JSContext, mut this_val: JSValue, mut argc: i32, mut argv: *mut JSValue) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut arr: JSValue = core::mem::zeroed();
let mut ret: JSValue = core::mem::zeroed();
let mut p: *mut JSObject = core::mem::zeroed();
let mut vm_block: usize = 9;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 58738
1 => {
return ret;
}
// C line 58737
2 => {
let _ = JS_FreeValue(ctx, arr);
vm_block = 1; continue;
}
// C line 58736
3 => {
let _ = { let assigned = js_typed_array_sort(ctx, arr, argc, argv); ret = assigned; assigned };
vm_block = 2; continue;
}
// C line 58735
4 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 58734
5 => {
vm_block = if (JS_IsException(arr)) != 0 { 4 } else { 3 }; continue;
}
// C line 58732
6 => {
let _ = { let assigned = js_typed_array_constructor_ta(ctx, JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }, this_val, (((*(p)).class_id) as i32), (((*(p)).u).array).count); arr = assigned; assigned };
vm_block = 5; continue;
}
// C line 58731
7 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 58730
8 => {
vm_block = if ((!(!(p).is_null()) as i32)) != 0 { 7 } else { 6 }; continue;
}
// C line 58729
9 => {
let _ = { let assigned = get_typed_array(ctx, this_val); p = assigned; assigned };
vm_block = 8; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:58845. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn b64_encode(mut src: *const u8, mut len: usize, mut dst: *mut c_char, mut alpha: *const u8) -> usize {
let mut vm_local_storage = Vec::<u64>::new();
let mut i: usize = core::mem::zeroed();
let mut j: usize = core::mem::zeroed();
let mut v: u32 = core::mem::zeroed();
let mut rem: usize = core::mem::zeroed();
let mut v_1: u32 = core::mem::zeroed();
let mut v_2: u32 = core::mem::zeroed();
let mut vm_block: usize = 22;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 58872
1 => {
return j;
}
// C line 58864
2 => {
let _ = { let assigned = (((61 as i32)) as c_char); *(dst).offset(({ let old = j; j = (j).wrapping_add(1); old }) as isize) = assigned; assigned };
vm_block = 1; continue;
}
// C line 58863
3 => {
let _ = { let assigned = (((61 as i32)) as c_char); *(dst).offset(({ let old = j; j = (j).wrapping_add(1); old }) as isize) = assigned; assigned };
vm_block = 2; continue;
}
// C line 58862
4 => {
let _ = { let assigned = ((*(alpha).offset(((((v_1).wrapping_shr(((12 as i32)) as u32)) & ((((63 as i32)) as u32)))) as isize)) as c_char); *(dst).offset(({ let old = j; j = (j).wrapping_add(1); old }) as isize) = assigned; assigned };
vm_block = 3; continue;
}
// C line 58861
5 => {
let _ = { let assigned = ((*(alpha).offset(((((v_1).wrapping_shr(((18 as i32)) as u32)) & ((((63 as i32)) as u32)))) as isize)) as c_char); *(dst).offset(({ let old = j; j = (j).wrapping_add(1); old }) as isize) = assigned; assigned };
vm_block = 4; continue;
}
// C line 58860
6 => {
v_1 = ((((65536 as i32)).wrapping_mul(((*(src).offset((i) as isize)) as i32))) as u32);
vm_block = 5; continue;
}
// C line 58870
7 => {
let _ = { let assigned = (((61 as i32)) as c_char); *(dst).offset(({ let old = j; j = (j).wrapping_add(1); old }) as isize) = assigned; assigned };
vm_block = 1; continue;
}
// C line 58869
8 => {
let _ = { let assigned = ((*(alpha).offset(((((v_2).wrapping_shr(((6 as i32)) as u32)) & ((((63 as i32)) as u32)))) as isize)) as c_char); *(dst).offset(({ let old = j; j = (j).wrapping_add(1); old }) as isize) = assigned; assigned };
vm_block = 7; continue;
}
// C line 58868
9 => {
let _ = { let assigned = ((*(alpha).offset(((((v_2).wrapping_shr(((12 as i32)) as u32)) & ((((63 as i32)) as u32)))) as isize)) as c_char); *(dst).offset(({ let old = j; j = (j).wrapping_add(1); old }) as isize) = assigned; assigned };
vm_block = 8; continue;
}
// C line 58867
10 => {
let _ = { let assigned = ((*(alpha).offset(((((v_2).wrapping_shr(((18 as i32)) as u32)) & ((((63 as i32)) as u32)))) as isize)) as c_char); *(dst).offset(({ let old = j; j = (j).wrapping_add(1); old }) as isize) = assigned; assigned };
vm_block = 9; continue;
}
// C line 58866
11 => {
v_2 = (((((65536 as i32)).wrapping_mul(((*(src).offset((i) as isize)) as i32))).wrapping_add(((256 as i32)).wrapping_mul(((*(src).offset(((i).wrapping_add((((1 as i32)) as usize))) as isize)) as i32)))) as u32);
vm_block = 10; continue;
}
// C line 58865
12 => {
vm_block = if ((((rem) == ((((2 as i32)) as usize))) as i32)) != 0 { 11 } else { 1 }; continue;
}
// C line 58859
13 => {
vm_block = if ((((rem) == ((((1 as i32)) as usize))) as i32)) != 0 { 6 } else { 12 }; continue;
}
// C line 58858
14 => {
rem = (len).wrapping_sub(i);
vm_block = 13; continue;
}
// C line 58850
15 => {
vm_block = if (((((i).wrapping_add((((3 as i32)) as usize))) <= (len)) as i32)) != 0 { 21 } else { 14 }; continue;
}
// C line ?
16 => {
let _ = { let _ = { i = (i).wrapping_add((((3 as i32)) as usize)); i }; { j = (j).wrapping_add((((4 as i32)) as usize)); j } };
vm_block = 15; continue;
}
// C line 58855
17 => {
let _ = { let assigned = ((*(alpha).offset((((v) & ((((63 as i32)) as u32)))) as isize)) as c_char); *(dst).offset(((j).wrapping_add((((3 as i32)) as usize))) as isize) = assigned; assigned };
vm_block = 16; continue;
}
// C line 58854
18 => {
let _ = { let assigned = ((*(alpha).offset(((((v).wrapping_shr(((6 as i32)) as u32)) & ((((63 as i32)) as u32)))) as isize)) as c_char); *(dst).offset(((j).wrapping_add((((2 as i32)) as usize))) as isize) = assigned; assigned };
vm_block = 17; continue;
}
// C line 58853
19 => {
let _ = { let assigned = ((*(alpha).offset(((((v).wrapping_shr(((12 as i32)) as u32)) & ((((63 as i32)) as u32)))) as isize)) as c_char); *(dst).offset(((j).wrapping_add((((1 as i32)) as usize))) as isize) = assigned; assigned };
vm_block = 18; continue;
}
// C line 58852
20 => {
let _ = { let assigned = ((*(alpha).offset(((((v).wrapping_shr(((18 as i32)) as u32)) & ((((63 as i32)) as u32)))) as isize)) as c_char); *(dst).offset(((j).wrapping_add((((0 as i32)) as usize))) as isize) = assigned; assigned };
vm_block = 19; continue;
}
// C line 58851
21 => {
v = ((((((65536 as i32)).wrapping_mul(((*(src).offset((i) as isize)) as i32))).wrapping_add(((256 as i32)).wrapping_mul(((*(src).offset(((i).wrapping_add((((1 as i32)) as usize))) as isize)) as i32)))).wrapping_add(((*(src).offset(((i).wrapping_add((((2 as i32)) as usize))) as isize)) as i32))) as u32);
vm_block = 20; continue;
}
// C line 58850
22 => {
let _ = { let _ = { let assigned = (((0 as i32)) as usize); i = assigned; assigned }; { let assigned = (((0 as i32)) as usize); j = assigned; assigned } };
vm_block = 15; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:58875. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn b64_skip_ws(mut src: *const c_char, mut len: usize, mut index: usize, mut dec_table: *const u8) -> usize {
let mut vm_local_storage = Vec::<u64>::new();
let mut vm_block: usize = 2;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 58880
1 => {
return index;
}
// C line 58878
2 => {
vm_block = if ((((((((index) < (len)) as i32)) != 0) && (((((((*(dec_table).offset((((*(src).offset((index) as isize)) as u8)) as isize)) as i32)) == ((64 as i32))) as i32)) != 0)) as i32)) != 0 { 3 } else { 1 }; continue;
}
// C line 58879
3 => {
let _ = { let old = index; index = (index).wrapping_add(1); old };
vm_block = 2; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:58891. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn from_base64(mut src: *const c_char, mut src_len: usize, mut dst: *mut u8, mut max_len: usize, mut dec_table: *const u8, mut last_chunk: i32, mut p_read: *mut usize, mut p_err: *mut i32) -> usize {
let mut vm_local_storage = Vec::<u64>::new();
let mut read: usize = core::mem::zeroed();
let mut written: usize = core::mem::zeroed();
let mut v: u32 = core::mem::zeroed();
let mut acc: u32 = core::mem::zeroed();
let mut seen: i32 = core::mem::zeroed();
let mut index: usize = core::mem::zeroed();
let mut ch: u8 = core::mem::zeroed();
let mut v0: u32 = core::mem::zeroed();
let mut v1: u32 = core::mem::zeroed();
let mut v2: u32 = core::mem::zeroed();
let mut v3: u32 = core::mem::zeroed();
let mut mask: u32 = core::mem::zeroed();
let mut remaining: usize = core::mem::zeroed();
let mut vm_block: usize = 98;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 59042
1 => {
return written;
}
// C line 59041
2 => {
let _ = { let assigned = src_len; *(p_read) = assigned; assigned };
vm_block = 1; continue;
}
// C line 59035
3 => {
let _ = { let assigned = (((acc).wrapping_shr(((4 as i32)) as u32)) as u8); *(dst).offset(({ let old = written; written = (written).wrapping_add(1); old }) as isize) = assigned; assigned };
vm_block = 2; continue;
}
// C line 59039
4 => {
let _ = { written = (written).wrapping_add((((2 as i32)) as usize)); written };
vm_block = 2; continue;
}
// C line 59038
5 => {
let _ = { let assigned = (((acc).wrapping_shr(((2 as i32)) as u32)) as u8); *(dst).offset(((written).wrapping_add((((1 as i32)) as usize))) as isize) = assigned; assigned };
vm_block = 4; continue;
}
// C line 59037
6 => {
let _ = { let assigned = (((acc).wrapping_shr(((10 as i32)) as u32)) as u8); *(dst).offset((written) as isize) = assigned; assigned };
vm_block = 5; continue;
}
// C line 59036
7 => {
vm_block = if ((((seen) == ((3 as i32))) as i32)) != 0 { 6 } else { 2 }; continue;
}
// C line 59034
8 => {
vm_block = if ((((seen) == ((2 as i32))) as i32)) != 0 { 3 } else { 7 }; continue;
}
// C line 58909
9 => {
vm_block = 90; continue;
}
// C line 59029
10 => {
return written;
}
// C line 59028
11 => {
let _ = { let assigned = read; *(p_read) = assigned; assigned };
vm_block = 10; continue;
}
// C line 59027
12 => {
vm_block = if ((((written) >= (max_len)) as i32)) != 0 { 11 } else { 9 }; continue;
}
// C line 59026
13 => {
let _ = { let assigned = index; read = assigned; assigned };
vm_block = 12; continue;
}
// C line 59025
14 => {
let _ = { let assigned = (0 as i32); seen = assigned; assigned };
vm_block = 13; continue;
}
// C line 59024
15 => {
let _ = { let assigned = (((0 as i32)) as u32); acc = assigned; assigned };
vm_block = 14; continue;
}
// C line 59023
16 => {
let _ = { written = (written).wrapping_add((((3 as i32)) as usize)); written };
vm_block = 15; continue;
}
// C line 59022
17 => {
let _ = { let assigned = ((acc) as u8); *(dst).offset(((written).wrapping_add((((2 as i32)) as usize))) as isize) = assigned; assigned };
vm_block = 16; continue;
}
// C line 59021
18 => {
let _ = { let assigned = (((acc).wrapping_shr(((8 as i32)) as u32)) as u8); *(dst).offset(((written).wrapping_add((((1 as i32)) as usize))) as isize) = assigned; assigned };
vm_block = 17; continue;
}
// C line 59020
19 => {
let _ = { let assigned = (((acc).wrapping_shr(((16 as i32)) as u32)) as u8); *(dst).offset((written) as isize) = assigned; assigned };
vm_block = 18; continue;
}
// C line 59019
20 => {
vm_block = if ((((seen) == ((4 as i32))) as i32)) != 0 { 19 } else { 9 }; continue;
}
// C line 59017
21 => {
let _ = { let old = seen; seen = (seen).wrapping_add(1); old };
vm_block = 20; continue;
}
// C line 59016
22 => {
let _ = { let assigned = (((acc).wrapping_shl(((6 as i32)) as u32)) | (v)); acc = assigned; assigned };
vm_block = 21; continue;
}
// C line 59012
23 => {
return written;
}
// C line 59011
24 => {
let _ = { let assigned = read; *(p_read) = assigned; assigned };
vm_block = 23; continue;
}
// C line 59009
25 => {
vm_block = if ((((((((((((remaining) == ((((1 as i32)) as usize))) as i32)) != 0) && (((((seen) == ((2 as i32))) as i32)) != 0)) as i32)) != 0) || (((((((((remaining) == ((((2 as i32)) as usize))) as i32)) != 0) && (((((seen) == ((3 as i32))) as i32)) != 0)) as i32)) != 0)) as i32)) != 0 { 24 } else { 22 }; continue;
}
// C line 59008
26 => {
remaining = (max_len).wrapping_sub(written);
vm_block = 25; continue;
}
// C line 59003
27 => {
return (((0 as i32)) as usize);
}
// C line 59002
28 => {
let _ = { let assigned = (1 as i32); *(p_err) = assigned; assigned };
vm_block = 27; continue;
}
// C line 59001
29 => {
vm_block = if ((((v) >= ((((64 as i32)) as u32))) as i32)) != 0 { 28 } else { 26 }; continue;
}
// C line 59000
30 => {
let _ = { let assigned = ((*(dec_table).offset((ch) as isize)) as u32); v = assigned; assigned };
vm_block = 29; continue;
}
// C line 58997
31 => {
vm_block = 8; continue;
}
// C line 58994
32 => {
return (((0 as i32)) as usize);
}
// C line 58993
33 => {
let _ = { let assigned = (1 as i32); *(p_err) = assigned; assigned };
vm_block = 32; continue;
}
// C line 58992
34 => {
vm_block = if (((acc) & (mask))) != 0 { 33 } else { 31 }; continue;
}
// C line 58991
35 => {
mask = ((if ((((seen) == ((2 as i32))) as i32)) != 0 { (15 as i32) } else { (3 as i32) }) as u32);
vm_block = 34; continue;
}
// C line 58990
36 => {
vm_block = if ((((last_chunk) == ((B64_LAST_STRICT as i32))) as i32)) != 0 { 35 } else { 31 }; continue;
}
// C line 58988
37 => {
return (((0 as i32)) as usize);
}
// C line 58987
38 => {
let _ = { let assigned = (1 as i32); *(p_err) = assigned; assigned };
vm_block = 37; continue;
}
// C line 58986
39 => {
vm_block = if ((((index) != (src_len)) as i32)) != 0 { 38 } else { 36 }; continue;
}
// C line 58979
40 => {
let _ = { let assigned = b64_skip_ws(src, src_len, index, dec_table); index = assigned; assigned };
vm_block = 39; continue;
}
// C line 58978
41 => {
let _ = { let old = index; index = (index).wrapping_add(1); old };
vm_block = 40; continue;
}
// C line 58982
42 => {
return (((0 as i32)) as usize);
}
// C line 58981
43 => {
let _ = { let assigned = (1 as i32); *(p_err) = assigned; assigned };
vm_block = 42; continue;
}
// C line 58977
44 => {
vm_block = if ((((((*(src).offset((index) as isize)) as i32)) == ((61 as i32))) as i32)) != 0 { 41 } else { 43 }; continue;
}
// C line 58975
45 => {
return (((0 as i32)) as usize);
}
// C line 58974
46 => {
let _ = { let assigned = (1 as i32); *(p_err) = assigned; assigned };
vm_block = 45; continue;
}
// C line 58972
47 => {
return written;
}
// C line 58971
48 => {
let _ = { let assigned = read; *(p_read) = assigned; assigned };
vm_block = 47; continue;
}
// C line 58970
49 => {
vm_block = if ((((last_chunk) == ((B64_LAST_STOP_BEFORE_PARTIAL as i32))) as i32)) != 0 { 48 } else { 46 }; continue;
}
// C line 58969
50 => {
vm_block = if ((((index) == (src_len)) as i32)) != 0 { 49 } else { 44 }; continue;
}
// C line 58968
51 => {
vm_block = if ((((seen) == ((2 as i32))) as i32)) != 0 { 50 } else { 39 }; continue;
}
// C line 58967
52 => {
let _ = { let assigned = b64_skip_ws(src, src_len, index, dec_table); index = assigned; assigned };
vm_block = 51; continue;
}
// C line 58965
53 => {
return (((0 as i32)) as usize);
}
// C line 58964
54 => {
let _ = { let assigned = (1 as i32); *(p_err) = assigned; assigned };
vm_block = 53; continue;
}
// C line 58963
55 => {
vm_block = if ((((seen) < ((2 as i32))) as i32)) != 0 { 54 } else { 52 }; continue;
}
// C line 58962
56 => {
vm_block = if ((((((ch) as i32)) == ((61 as i32))) as i32)) != 0 { 55 } else { 30 }; continue;
}
// C line 58960
57 => {
let _ = { let assigned = ((*(src).offset(({ let old = index; index = (index).wrapping_add(1); old }) as isize)) as u8); ch = assigned; assigned };
vm_block = 56; continue;
}
// C line 58957
58 => {
return written;
}
// C line 58956
59 => {
let _ = { let assigned = src_len; *(p_read) = assigned; assigned };
vm_block = 58; continue;
}
// C line 58954
60 => {
vm_block = 8; continue;
}
// C line 58952
61 => {
return (((0 as i32)) as usize);
}
// C line 58951
62 => {
let _ = { let assigned = (1 as i32); *(p_err) = assigned; assigned };
vm_block = 61; continue;
}
// C line 58950
63 => {
vm_block = if ((((seen) == ((1 as i32))) as i32)) != 0 { 62 } else { 60 }; continue;
}
// C line 58947
64 => {
return (((0 as i32)) as usize);
}
// C line 58946
65 => {
let _ = { let assigned = (1 as i32); *(p_err) = assigned; assigned };
vm_block = 64; continue;
}
// C line 58945
66 => {
vm_block = if ((((last_chunk) == ((B64_LAST_STRICT as i32))) as i32)) != 0 { 65 } else { 63 }; continue;
}
// C line 58943
67 => {
return written;
}
// C line 58942
68 => {
let _ = { let assigned = read; *(p_read) = assigned; assigned };
vm_block = 67; continue;
}
// C line 58941
69 => {
vm_block = if ((((last_chunk) == ((B64_LAST_STOP_BEFORE_PARTIAL as i32))) as i32)) != 0 { 68 } else { 66 }; continue;
}
// C line 58940
70 => {
vm_block = if ((((seen) > ((0 as i32))) as i32)) != 0 { 69 } else { 59 }; continue;
}
// C line 58939
71 => {
vm_block = if ((((index) == (src_len)) as i32)) != 0 { 70 } else { 57 }; continue;
}
// C line 58937
72 => {
let _ = { let assigned = b64_skip_ws(src, src_len, index, dec_table); index = assigned; assigned };
vm_block = 71; continue;
}
// C line 58932
73 => {
return written;
}
// C line 58931
74 => {
let _ = { let assigned = read; *(p_read) = assigned; assigned };
vm_block = 73; continue;
}
// C line 58930
75 => {
vm_block = if ((((written) >= (max_len)) as i32)) != 0 { 74 } else { 72 }; continue;
}
// C line 58928
76 => {
let _ = { let assigned = index; read = assigned; assigned };
vm_block = 75; continue;
}
// C line 58913
77 => {
vm_block = if (((((((((index).wrapping_add((((4 as i32)) as usize))) <= (src_len)) as i32)) != 0) && ((((((written).wrapping_add((((3 as i32)) as usize))) <= (max_len)) as i32)) != 0)) as i32)) != 0 { 89 } else { 76 }; continue;
}
// C line 58926
78 => {
let _ = { index = (index).wrapping_add((((4 as i32)) as usize)); index };
vm_block = 77; continue;
}
// C line 58925
79 => {
let _ = { written = (written).wrapping_add((((3 as i32)) as usize)); written };
vm_block = 78; continue;
}
// C line 58924
80 => {
let _ = { let assigned = ((v) as u8); *(dst).offset(((written).wrapping_add((((2 as i32)) as usize))) as isize) = assigned; assigned };
vm_block = 79; continue;
}
// C line 58923
81 => {
let _ = { let assigned = (((v).wrapping_shr(((8 as i32)) as u32)) as u8); *(dst).offset(((written).wrapping_add((((1 as i32)) as usize))) as isize) = assigned; assigned };
vm_block = 80; continue;
}
// C line 58922
82 => {
let _ = { let assigned = (((v).wrapping_shr(((16 as i32)) as u32)) as u8); *(dst).offset((written) as isize) = assigned; assigned };
vm_block = 81; continue;
}
// C line 58921
83 => {
let _ = { let assigned = (((((((v0).wrapping_shl(((18 as i32)) as u32)) | ((v1).wrapping_shl(((12 as i32)) as u32)))) | ((v2).wrapping_shl(((6 as i32)) as u32)))) | (v3)); v = assigned; assigned };
vm_block = 82; continue;
}
// C line 58920
84 => {
vm_block = 76; continue;
}
// C line 58919
85 => {
vm_block = if ((((((((((v0) | (v1))) | (v2))) | (v3))) >= ((((64 as i32)) as u32))) as i32)) != 0 { 84 } else { 83 }; continue;
}
// C line 58918
86 => {
let _ = { let assigned = ((*(dec_table).offset((((*(src).offset(((index).wrapping_add((((3 as i32)) as usize))) as isize)) as u8)) as isize)) as u32); v3 = assigned; assigned };
vm_block = 85; continue;
}
// C line 58917
87 => {
let _ = { let assigned = ((*(dec_table).offset((((*(src).offset(((index).wrapping_add((((2 as i32)) as usize))) as isize)) as u8)) as isize)) as u32); v2 = assigned; assigned };
vm_block = 86; continue;
}
// C line 58916
88 => {
let _ = { let assigned = ((*(dec_table).offset((((*(src).offset(((index).wrapping_add((((1 as i32)) as usize))) as isize)) as u8)) as isize)) as u32); v1 = assigned; assigned };
vm_block = 87; continue;
}
// C line 58915
89 => {
let _ = { let assigned = ((*(dec_table).offset((((*(src).offset((index) as isize)) as u8)) as isize)) as u32); v0 = assigned; assigned };
vm_block = 88; continue;
}
// C line 58910
90 => {
vm_block = if ((((seen) == ((0 as i32))) as i32)) != 0 { 77 } else { 72 }; continue;
}
// C line 58906
91 => {
return (((0 as i32)) as usize);
}
// C line 58905
92 => {
let _ = { let assigned = (((0 as i32)) as usize); *(p_read) = assigned; assigned };
vm_block = 91; continue;
}
// C line 58904
93 => {
vm_block = if ((((max_len) == ((((0 as i32)) as usize))) as i32)) != 0 { 92 } else { 9 }; continue;
}
// C line 58902
94 => {
let _ = { let assigned = (0 as i32); *(p_err) = assigned; assigned };
vm_block = 93; continue;
}
// C line 58899
95 => {
index = (((0 as i32)) as usize);
vm_block = 94; continue;
}
// C line 58898
96 => {
seen = (0 as i32);
vm_block = 95; continue;
}
// C line 58897
97 => {
acc = (((0 as i32)) as u32);
vm_block = 96; continue;
}
// C line 58896
98 => {
read = (((0 as i32)) as usize);
written = (((0 as i32)) as usize);
vm_block = 97; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:59048. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn u8a_hex_encode(mut src: *const u8, mut len: usize, mut dst: *mut c_char) -> usize {
let mut vm_local_storage = Vec::<u64>::new();
let mut i: usize = core::mem::zeroed();
let mut vm_block: usize = 6;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 59054
1 => {
return (len).wrapping_mul((((2 as i32)) as usize));
}
// C line 59050
2 => {
vm_block = if ((((i) < (len)) as i32)) != 0 { 5 } else { 1 }; continue;
}
// C line ?
3 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 2; continue;
}
// C line 59052
4 => {
let _ = { let assigned = *((u8a_hex_digits).as_ptr()).offset((((((*(src).offset((i) as isize)) as i32)) & ((15 as i32)))) as isize); *(dst).offset((((i).wrapping_mul((((2 as i32)) as usize))).wrapping_add((((1 as i32)) as usize))) as isize) = assigned; assigned };
vm_block = 3; continue;
}
// C line 59051
5 => {
let _ = { let assigned = *((u8a_hex_digits).as_ptr()).offset(((((*(src).offset((i) as isize)) as i32)).wrapping_shr(((4 as i32)) as u32)) as isize); *(dst).offset(((i).wrapping_mul((((2 as i32)) as usize))) as isize) = assigned; assigned };
vm_block = 4; continue;
}
// C line 59050
6 => {
i = (((0 as i32)) as usize);
vm_block = 2; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:59059. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn u8a_hex_decode(mut src: *const c_char, mut src_len: usize, mut dst: *mut u8, mut max_len: usize, mut p_read: *mut usize, mut p_err: *mut i32) -> usize {
let mut vm_local_storage = Vec::<u64>::new();
let mut written: usize = core::mem::zeroed();
let mut i: usize = core::mem::zeroed();
let mut hi: i32 = core::mem::zeroed();
let mut lo: i32 = core::mem::zeroed();
let mut vm_block: usize = 15;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 59083
1 => {
return written;
}
// C line 59082
2 => {
let _ = { let assigned = i; *(p_read) = assigned; assigned };
vm_block = 1; continue;
}
// C line 59071
3 => {
vm_block = if ((((((((i) < (src_len)) as i32)) != 0) && (((((written) < (max_len)) as i32)) != 0)) as i32)) != 0 { 10 } else { 2 }; continue;
}
// C line 59079
4 => {
let _ = { i = (i).wrapping_add((((2 as i32)) as usize)); i };
vm_block = 3; continue;
}
// C line 59078
5 => {
let _ = { let assigned = (((((hi).wrapping_shl(((4 as i32)) as u32)) | (lo))) as u8); *(dst).offset(({ let old = written; written = (written).wrapping_add(1); old }) as isize) = assigned; assigned };
vm_block = 4; continue;
}
// C line 59076
6 => {
return (((0 as i32)) as usize);
}
// C line 59075
7 => {
let _ = { let assigned = (1 as i32); *(p_err) = assigned; assigned };
vm_block = 6; continue;
}
// C line 59074
8 => {
vm_block = if ((((((((hi) < ((0 as i32))) as i32)) != 0) || (((((lo) < ((0 as i32))) as i32)) != 0)) as i32)) != 0 { 7 } else { 5 }; continue;
}
// C line 59073
9 => {
lo = from_hex(((*(src).offset(((i).wrapping_add((((1 as i32)) as usize))) as isize)) as i32));
vm_block = 8; continue;
}
// C line 59072
10 => {
hi = from_hex(((*(src).offset((i) as isize)) as i32));
vm_block = 9; continue;
}
// C line 59068
11 => {
return (((0 as i32)) as usize);
}
// C line 59067
12 => {
let _ = { let assigned = (1 as i32); *(p_err) = assigned; assigned };
vm_block = 11; continue;
}
// C line 59066
13 => {
vm_block = if (((src_len) & ((((1 as i32)) as usize)))) != 0 { 12 } else { 3 }; continue;
}
// C line 59064
14 => {
let _ = { let assigned = (0 as i32); *(p_err) = assigned; assigned };
vm_block = 13; continue;
}
// C line 59063
15 => {
written = (((0 as i32)) as usize);
i = (((0 as i32)) as usize);
vm_block = 14; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:59086. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn JS_NewUint8ArrayCopy(mut ctx: *mut JSContext, mut buf: *const u8, mut len: usize) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut buffer: JSValue = core::mem::zeroed();
let mut obj: JSValue = core::mem::zeroed();
let mut abuf: *mut JSArrayBuffer = core::mem::zeroed();
let mut vm_block: usize = 13;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 59110
1 => {
return obj;
}
// C line 59108
2 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 59107
3 => {
let _ = JS_FreeValue(ctx, obj);
vm_block = 2; continue;
}
// C line 59105
4 => {
vm_block = if (typed_array_init(ctx, obj, buffer, (((0 as i32)) as u64), (((*(abuf)).byte_length) as u64), (0 as i32))) != 0 { 3 } else { 1 }; continue;
}
// C line 59104
5 => {
let _ = if ((((!(((((abuf) != (core::ptr::null_mut::<JSArrayBuffer>())) as i32)) != 0) as i32)) as i64)) != 0 { std::process::abort() } else { { let _ = (0 as i32); } };
vm_block = 4; continue;
}
// C line 59103
6 => {
let _ = { let assigned = js_get_array_buffer(ctx, buffer); abuf = assigned; assigned };
vm_block = 5; continue;
}
// C line 59101
7 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 59100
8 => {
let _ = JS_FreeValue(ctx, buffer);
vm_block = 7; continue;
}
// C line 59099
9 => {
vm_block = if (JS_IsException(obj)) != 0 { 8 } else { 6 }; continue;
}
// C line 59098
10 => {
let _ = { let assigned = js_create_from_ctor(ctx, JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }, (JS_CLASS_UINT8_ARRAY as i32)); obj = assigned; assigned };
vm_block = 9; continue;
}
// C line 59097
11 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 59096
12 => {
vm_block = if (JS_IsException(buffer)) != 0 { 11 } else { 10 }; continue;
}
// C line 59091
13 => {
let _ = { let assigned = js_array_buffer_constructor3(ctx, JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }, ((len) as u64), core::ptr::null_mut::<u64>(), (((JS_CLASS_ARRAY_BUFFER as i32)) as JSClassID), ((buf) as *mut u8), Some(js_array_buffer_free), core::ptr::null_mut::<c_void>(), (1 as i32)); buffer = assigned; assigned };
vm_block = 12; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:59115. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn check_uint8array(mut ctx: *mut JSContext, mut this_val: JSValue) -> *mut JSObject {
let mut vm_local_storage = Vec::<u64>::new();
let mut p: *mut JSObject = core::mem::zeroed();
let mut vm_block: usize = 8;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 59127
1 => {
return core::ptr::null_mut::<JSObject>();
}
// C line ? labels: fail
2 => {
let _ = JS_ThrowTypeError(ctx, c"not a Uint8Array".as_ptr());
vm_block = 1; continue;
}
// C line 59124
3 => {
return p;
}
// C line 59123
4 => {
vm_block = 2; continue;
}
// C line 59122
5 => {
vm_block = if (((((((*(p)).class_id) as i32)) != ((JS_CLASS_UINT8_ARRAY as i32))) as i32)) != 0 { 4 } else { 3 }; continue;
}
// C line 59121
6 => {
let _ = { let assigned = ((((this_val).u).ptr) as *mut JSObject); p = assigned; assigned };
vm_block = 5; continue;
}
// C line 59120
7 => {
vm_block = 2; continue;
}
// C line 59119
8 => {
vm_block = if (((((((this_val).tag) as i32)) != ((JS_TAG_OBJECT as i32))) as i32)) != 0 { 7 } else { 6 }; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:59133. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn get_uint8array_bytes(mut ctx: *mut JSContext, mut p: *mut JSObject, mut pdata: *mut *mut u8, mut plen: *mut usize) -> i32 {
let mut vm_local_storage = Vec::<u64>::new();
let mut vm_block: usize = 8;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 59144
1 => {
return (0 as i32);
}
// C line 59143
2 => {
let _ = { let assigned = (((((*(p)).u).array).count) as usize); *(plen) = assigned; assigned };
vm_block = 1; continue;
}
// C line 59142
3 => {
let _ = { let assigned = ((((*(p)).u).array).u).uint8_ptr; *(pdata) = assigned; assigned };
vm_block = 2; continue;
}
// C line 59140
4 => {
return ((1 as i32)).wrapping_neg();
}
// C line 59139
5 => {
let _ = { let assigned = (((0 as i32)) as usize); *(plen) = assigned; assigned };
vm_block = 4; continue;
}
// C line 59138
6 => {
let _ = { let assigned = core::ptr::null_mut::<u8>(); *(pdata) = assigned; assigned };
vm_block = 5; continue;
}
// C line 59137
7 => {
let _ = JS_ThrowTypeErrorArrayBufferOOB(ctx);
vm_block = 6; continue;
}
// C line 59136
8 => {
vm_block = if (typed_array_is_oob(p)) != 0 { 7 } else { 3 }; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:59149. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn check_options_object(mut ctx: *mut JSContext, mut options: JSValue) -> i32 {
let mut vm_local_storage = Vec::<u64>::new();
let mut vm_block: usize = 6;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 59157
1 => {
return (0 as i32);
}
// C line 59155
2 => {
return ((1 as i32)).wrapping_neg();
}
// C line 59154
3 => {
let _ = JS_ThrowTypeError(ctx, c"options must be an object".as_ptr());
vm_block = 2; continue;
}
// C line 59153
4 => {
vm_block = if ((!((JS_IsObject(options)) != 0) as i32)) != 0 { 3 } else { 1 }; continue;
}
// C line 59152
5 => {
return (0 as i32);
}
// C line 59151
6 => {
vm_block = if (JS_IsUndefined(options)) != 0 { 5 } else { 4 }; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:59162. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn parse_alphabet_option(mut ctx: *mut JSContext, mut options: JSValue) -> i32 {
let mut vm_local_storage = Vec::<u64>::new();
let mut val: JSValue = core::mem::zeroed();
let mut str: *const c_char = core::mem::zeroed();
let mut ret: i32 = core::mem::zeroed();
let mut vm_block: usize = 23;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 59196
1 => {
return ret;
}
// C line 59195
2 => {
let _ = JS_FreeCString(ctx, str);
vm_block = 1; continue;
}
// C line 59188
3 => {
let _ = { let assigned = (B64_ALPHABET_BASE64 as i32); ret = assigned; assigned };
vm_block = 2; continue;
}
// C line 59190
4 => {
let _ = { let assigned = (B64_ALPHABET_BASE64URL as i32); ret = assigned; assigned };
vm_block = 2; continue;
}
// C line 59193
5 => {
let _ = { let assigned = ((1 as i32)).wrapping_neg(); ret = assigned; assigned };
vm_block = 2; continue;
}
// C line 59192
6 => {
let _ = JS_ThrowTypeError(ctx, c"invalid alphabet".as_ptr());
vm_block = 5; continue;
}
// C line 59189
7 => {
vm_block = if ((!((buffer_strcmp(str, c"base64url".as_ptr())) != 0) as i32)) != 0 { 4 } else { 6 }; continue;
}
// C line 59187
8 => {
vm_block = if ((!((buffer_strcmp(str, c"base64".as_ptr())) != 0) as i32)) != 0 { 3 } else { 7 }; continue;
}
// C line 59185
9 => {
return ((1 as i32)).wrapping_neg();
}
// C line 59184
10 => {
vm_block = if ((!(!(str).is_null()) as i32)) != 0 { 9 } else { 8 }; continue;
}
// C line 59183
11 => {
let _ = JS_FreeValue(ctx, val);
vm_block = 10; continue;
}
// C line 59182
12 => {
let _ = { let assigned = JS_ToCString(ctx, val); str = assigned; assigned };
vm_block = 11; continue;
}
// C line 59179
13 => {
return ((1 as i32)).wrapping_neg();
}
// C line 59178
14 => {
let _ = JS_ThrowTypeError(ctx, c"expected string for alphabet".as_ptr());
vm_block = 13; continue;
}
// C line 59177
15 => {
let _ = JS_FreeValue(ctx, val);
vm_block = 14; continue;
}
// C line 59176
16 => {
vm_block = if ((!((JS_IsString(val)) != 0) as i32)) != 0 { 15 } else { 12 }; continue;
}
// C line 59175
17 => {
return (B64_ALPHABET_BASE64 as i32);
}
// C line 59174
18 => {
vm_block = if (JS_IsUndefined(val)) != 0 { 17 } else { 16 }; continue;
}
// C line 59173
19 => {
return ((1 as i32)).wrapping_neg();
}
// C line 59172
20 => {
vm_block = if (JS_IsException(val)) != 0 { 19 } else { 18 }; continue;
}
// C line 59171
21 => {
let _ = { let assigned = JS_GetProperty(ctx, options, (((crate::quickjs_atom::JS_ATOM_alphabet as i32)) as JSAtom)); val = assigned; assigned };
vm_block = 20; continue;
}
// C line 59169
22 => {
return (B64_ALPHABET_BASE64 as i32);
}
// C line 59168
23 => {
vm_block = if (JS_IsUndefined(options)) != 0 { 22 } else { 21 }; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:59200. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn parse_last_chunk_option(mut ctx: *mut JSContext, mut options: JSValue) -> i32 {
let mut vm_local_storage = Vec::<u64>::new();
let mut val: JSValue = core::mem::zeroed();
let mut str: *const c_char = core::mem::zeroed();
let mut ret: i32 = core::mem::zeroed();
let mut vm_block: usize = 25;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 59236
1 => {
return ret;
}
// C line 59235
2 => {
let _ = JS_FreeCString(ctx, str);
vm_block = 1; continue;
}
// C line 59226
3 => {
let _ = { let assigned = (B64_LAST_LOOSE as i32); ret = assigned; assigned };
vm_block = 2; continue;
}
// C line 59228
4 => {
let _ = { let assigned = (B64_LAST_STRICT as i32); ret = assigned; assigned };
vm_block = 2; continue;
}
// C line 59230
5 => {
let _ = { let assigned = (B64_LAST_STOP_BEFORE_PARTIAL as i32); ret = assigned; assigned };
vm_block = 2; continue;
}
// C line 59233
6 => {
let _ = { let assigned = ((1 as i32)).wrapping_neg(); ret = assigned; assigned };
vm_block = 2; continue;
}
// C line 59232
7 => {
let _ = JS_ThrowTypeError(ctx, c"invalid lastChunkHandling option".as_ptr());
vm_block = 6; continue;
}
// C line 59229
8 => {
vm_block = if ((!((buffer_strcmp(str, c"stop-before-partial".as_ptr())) != 0) as i32)) != 0 { 5 } else { 7 }; continue;
}
// C line 59227
9 => {
vm_block = if ((!((buffer_strcmp(str, c"strict".as_ptr())) != 0) as i32)) != 0 { 4 } else { 8 }; continue;
}
// C line 59225
10 => {
vm_block = if ((!((buffer_strcmp(str, c"loose".as_ptr())) != 0) as i32)) != 0 { 3 } else { 9 }; continue;
}
// C line 59223
11 => {
return ((1 as i32)).wrapping_neg();
}
// C line 59222
12 => {
vm_block = if ((!(!(str).is_null()) as i32)) != 0 { 11 } else { 10 }; continue;
}
// C line 59221
13 => {
let _ = JS_FreeValue(ctx, val);
vm_block = 12; continue;
}
// C line 59220
14 => {
let _ = { let assigned = JS_ToCString(ctx, val); str = assigned; assigned };
vm_block = 13; continue;
}
// C line 59217
15 => {
return ((1 as i32)).wrapping_neg();
}
// C line 59216
16 => {
let _ = JS_ThrowTypeError(ctx, c"expected string for lastChunkHandling".as_ptr());
vm_block = 15; continue;
}
// C line 59215
17 => {
let _ = JS_FreeValue(ctx, val);
vm_block = 16; continue;
}
// C line 59214
18 => {
vm_block = if ((!((JS_IsString(val)) != 0) as i32)) != 0 { 17 } else { 14 }; continue;
}
// C line 59213
19 => {
return (B64_LAST_LOOSE as i32);
}
// C line 59212
20 => {
vm_block = if (JS_IsUndefined(val)) != 0 { 19 } else { 18 }; continue;
}
// C line 59211
21 => {
return ((1 as i32)).wrapping_neg();
}
// C line 59210
22 => {
vm_block = if (JS_IsException(val)) != 0 { 21 } else { 20 }; continue;
}
// C line 59209
23 => {
let _ = { let assigned = JS_GetProperty(ctx, options, (((crate::quickjs_atom::JS_ATOM_lastChunkHandling as i32)) as JSAtom)); val = assigned; assigned };
vm_block = 22; continue;
}
// C line 59207
24 => {
return (B64_LAST_LOOSE as i32);
}
// C line 59206
25 => {
vm_block = if (JS_IsUndefined(options)) != 0 { 24 } else { 23 }; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:59240. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_uint8array_to_base64(mut ctx: *mut JSContext, mut this_val: JSValue, mut argc: i32, mut argv: *mut JSValue) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut data: *mut u8 = core::mem::zeroed();
let mut len: usize = core::mem::zeroed();
let mut options: JSValue = core::mem::zeroed();
let mut p: *mut JSObject = core::mem::zeroed();
let mut alphabet: i32 = core::mem::zeroed();
let mut omit_padding: i32 = core::mem::zeroed();
let mut out_len: usize = core::mem::zeroed();
let mut written: usize = core::mem::zeroed();
let mut ostr: *mut JSString = core::mem::zeroed();
let mut dst: *mut c_char = core::mem::zeroed();
let mut op_val: JSValue = core::mem::zeroed();
let mut vm_block: usize = 32;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 59294
1 => {
return JSValue { u: JSValueUnion { ptr: ((ostr) as *mut c_void) }, tag: (((JS_TAG_STRING as i32)) as i64) };
}
// C line 59293
2 => {
let _ = { let assigned = ((written) as u32); (*(ostr)).set_len((assigned) as _); assigned };
vm_block = 1; continue;
}
// C line 59291
3 => {
let _ = { let assigned = (((0 as i32)) as c_char); *(dst).offset((written) as isize) = assigned; assigned };
vm_block = 2; continue;
}
// C line 59288
4 => {
vm_block = if ((((((((written) > ((((0 as i32)) as usize))) as i32)) != 0) && (((((((*(dst).offset(((written).wrapping_sub((((1 as i32)) as usize))) as isize)) as i32)) == ((61 as i32))) as i32)) != 0)) as i32)) != 0 { 5 } else { 3 }; continue;
}
// C line 59289
5 => {
let _ = { let old = written; written = (written).wrapping_sub(1); old };
vm_block = 4; continue;
}
// C line 59287
6 => {
vm_block = if (omit_padding) != 0 { 4 } else { 3 }; continue;
}
// C line 59285
7 => {
let _ = { let assigned = b64_encode(data, len, dst, if ((((alphabet) == ((B64_ALPHABET_BASE64URL as i32))) as i32)) != 0 { (b64url_enc).as_ptr() } else { (b64_enc).as_ptr() }); written = assigned; assigned };
vm_block = 6; continue;
}
// C line 59284
8 => {
let _ = { let assigned = (((((*(ostr)).u).str8).as_mut_ptr()) as *mut c_char); dst = assigned; assigned };
vm_block = 7; continue;
}
// C line 59282
9 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 59281
10 => {
vm_block = if ((!(!(ostr).is_null()) as i32)) != 0 { 9 } else { 8 }; continue;
}
// C line 59280
11 => {
let _ = { let assigned = js_alloc_string(ctx, ((out_len) as i32), (0 as i32)); ostr = assigned; assigned };
vm_block = 10; continue;
}
// C line 59278
12 => {
return JS_ThrowRangeError(ctx, c"output too large".as_ptr());
}
// C line 59277
13 => {
vm_block = if ((((!(((!(((((out_len) > ((((((1 as i32)).wrapping_shl(((30 as i32)) as u32)).wrapping_sub((1 as i32))) as usize))) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 12 } else { 11 }; continue;
}
// C line 59275
14 => {
let _ = { let assigned = ((((4 as i32)) as usize)).wrapping_mul((((len).wrapping_add((((2 as i32)) as usize))) / ((((3 as i32)) as usize)))); out_len = assigned; assigned };
vm_block = 13; continue;
}
// C line 59273
15 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 59272
16 => {
vm_block = if (get_uint8array_bytes(ctx, p, core::ptr::addr_of_mut!(data), core::ptr::addr_of_mut!(len))) != 0 { 15 } else { 14 }; continue;
}
// C line 59269
17 => {
let _ = JS_FreeValue(ctx, op_val);
vm_block = 16; continue;
}
// C line 59268
18 => {
let _ = { let assigned = JS_ToBool(ctx, op_val); omit_padding = assigned; assigned };
vm_block = 17; continue;
}
// C line 59267
19 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 59266
20 => {
vm_block = if (JS_IsException(op_val)) != 0 { 19 } else { 18 }; continue;
}
// C line 59265
21 => {
op_val = JS_GetProperty(ctx, options, (((crate::quickjs_atom::JS_ATOM_omitPadding as i32)) as JSAtom));
vm_block = 20; continue;
}
// C line 59264
22 => {
vm_block = if ((!((JS_IsUndefined(options)) != 0) as i32)) != 0 { 21 } else { 16 }; continue;
}
// C line 59263
23 => {
let _ = { let assigned = (0 as i32); omit_padding = assigned; assigned };
vm_block = 22; continue;
}
// C line 59261
24 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 59260
25 => {
vm_block = if ((((alphabet) < ((0 as i32))) as i32)) != 0 { 24 } else { 23 }; continue;
}
// C line 59259
26 => {
let _ = { let assigned = parse_alphabet_option(ctx, options); alphabet = assigned; assigned };
vm_block = 25; continue;
}
// C line 59258
27 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 59257
28 => {
vm_block = if (check_options_object(ctx, options)) != 0 { 27 } else { 26 }; continue;
}
// C line 59256
29 => {
let _ = { let assigned = if ((((argc) > ((0 as i32))) as i32)) != 0 { *(argv).offset(((0 as i32)) as isize) } else { JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) } }; options = assigned; assigned };
vm_block = 28; continue;
}
// C line 59254
30 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 59253
31 => {
vm_block = if ((!(!(p).is_null()) as i32)) != 0 { 30 } else { 29 }; continue;
}
// C line 59252
32 => {
let _ = { let assigned = check_uint8array(ctx, this_val); p = assigned; assigned };
vm_block = 31; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:59298. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_uint8array_to_hex(mut ctx: *mut JSContext, mut this_val: JSValue, mut argc: i32, mut argv: *mut JSValue) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut data: *mut u8 = core::mem::zeroed();
let mut len: usize = core::mem::zeroed();
let mut out_len: usize = core::mem::zeroed();
let mut p: *mut JSObject = core::mem::zeroed();
let mut ostr: *mut JSString = core::mem::zeroed();
let mut vm_block: usize = 14;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 59322
1 => {
return JSValue { u: JSValueUnion { ptr: ((ostr) as *mut c_void) }, tag: (((JS_TAG_STRING as i32)) as i64) };
}
// C line 59321
2 => {
let _ = { let assigned = (((0 as i32)) as u8); *((((*(ostr)).u).str8).as_mut_ptr()).offset((out_len) as isize) = assigned; assigned };
vm_block = 1; continue;
}
// C line 59320
3 => {
let _ = u8a_hex_encode(data, len, (((((*(ostr)).u).str8).as_mut_ptr()) as *mut c_char));
vm_block = 2; continue;
}
// C line 59318
4 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 59317
5 => {
vm_block = if ((!(!(ostr).is_null()) as i32)) != 0 { 4 } else { 3 }; continue;
}
// C line 59316
6 => {
let _ = { let assigned = js_alloc_string(ctx, ((out_len) as i32), (0 as i32)); ostr = assigned; assigned };
vm_block = 5; continue;
}
// C line 59314
7 => {
return JS_ThrowRangeError(ctx, c"output too large".as_ptr());
}
// C line 59313
8 => {
vm_block = if ((((!(((!(((((out_len) > ((((((1 as i32)).wrapping_shl(((30 as i32)) as u32)).wrapping_sub((1 as i32))) as usize))) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 7 } else { 6 }; continue;
}
// C line 59312
9 => {
let _ = { let assigned = (len).wrapping_mul((((2 as i32)) as usize)); out_len = assigned; assigned };
vm_block = 8; continue;
}
// C line 59310
10 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 59309
11 => {
vm_block = if (get_uint8array_bytes(ctx, p, core::ptr::addr_of_mut!(data), core::ptr::addr_of_mut!(len))) != 0 { 10 } else { 9 }; continue;
}
// C line 59308
12 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 59307
13 => {
vm_block = if ((!(!(p).is_null()) as i32)) != 0 { 12 } else { 11 }; continue;
}
// C line 59306
14 => {
let _ = { let assigned = check_uint8array(ctx, this_val); p = assigned; assigned };
vm_block = 13; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:59326. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_uint8array_from_base64(mut ctx: *mut JSContext, mut this_val: JSValue, mut argc: i32, mut argv: *mut JSValue) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut str: *const c_char = core::mem::zeroed();
let mut str_len: usize = core::mem::zeroed();
let mut read_pos: usize = core::mem::zeroed();
let mut decoded_len: usize = core::mem::zeroed();
let mut out_cap: usize = core::mem::zeroed();
let mut alphabet: i32 = core::mem::zeroed();
let mut last_chunk: i32 = core::mem::zeroed();
let mut err: i32 = core::mem::zeroed();
let mut buf: *mut u8 = core::mem::zeroed();
let mut result: JSValue = core::mem::zeroed();
let mut options: JSValue = core::mem::zeroed();
let mut vm_block: usize = 30;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 59379
1 => {
return result;
}
// C line 59378
2 => {
let _ = js_free(ctx, ((buf) as *mut c_void));
vm_block = 1; continue;
}
// C line 59377
3 => {
let _ = { let assigned = JS_NewUint8ArrayCopy(ctx, buf, decoded_len); result = assigned; assigned };
vm_block = 2; continue;
}
// C line 59374
4 => {
return JS_ThrowSyntaxError(ctx, c"invalid base64 string".as_ptr());
}
// C line 59373
5 => {
let _ = js_free(ctx, ((buf) as *mut c_void));
vm_block = 4; continue;
}
// C line 59372
6 => {
vm_block = if (err) != 0 { 5 } else { 3 }; continue;
}
// C line 59370
7 => {
let _ = JS_FreeCString(ctx, str);
vm_block = 6; continue;
}
// C line 59366
8 => {
let _ = { let assigned = from_base64(str, str_len, buf, out_cap, if ((((alphabet) == ((B64_ALPHABET_BASE64URL as i32))) as i32)) != 0 { (b64url_dec).as_ptr() } else { (b64_dec).as_ptr() }, last_chunk, core::ptr::addr_of_mut!(read_pos), core::ptr::addr_of_mut!(err)); decoded_len = assigned; assigned };
vm_block = 7; continue;
}
// C line 59363
9 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 59362
10 => {
let _ = JS_FreeCString(ctx, str);
vm_block = 9; continue;
}
// C line 59361
11 => {
vm_block = if ((!(!(buf).is_null()) as i32)) != 0 { 10 } else { 8 }; continue;
}
// C line 59360
12 => {
let _ = { let assigned = ((js_malloc(ctx, out_cap)) as *mut u8); buf = assigned; assigned };
vm_block = 11; continue;
}
// C line 59359
13 => {
let _ = { let assigned = ((((str_len) / ((((4 as i32)) as usize)))).wrapping_mul((((3 as i32)) as usize))).wrapping_add((((3 as i32)) as usize)); out_cap = assigned; assigned };
vm_block = 12; continue;
}
// C line 59356
14 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 59355
15 => {
let _ = JS_FreeCString(ctx, str);
vm_block = 14; continue;
}
// C line 59354
16 => {
vm_block = if ((((last_chunk) < ((0 as i32))) as i32)) != 0 { 15 } else { 13 }; continue;
}
// C line 59353
17 => {
let _ = { let assigned = parse_last_chunk_option(ctx, options); last_chunk = assigned; assigned };
vm_block = 16; continue;
}
// C line 59351
18 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 59350
19 => {
let _ = JS_FreeCString(ctx, str);
vm_block = 18; continue;
}
// C line 59349
20 => {
vm_block = if ((((alphabet) < ((0 as i32))) as i32)) != 0 { 19 } else { 17 }; continue;
}
// C line 59348
21 => {
let _ = { let assigned = parse_alphabet_option(ctx, options); alphabet = assigned; assigned };
vm_block = 20; continue;
}
// C line 59346
22 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 59345
23 => {
let _ = JS_FreeCString(ctx, str);
vm_block = 22; continue;
}
// C line 59344
24 => {
vm_block = if (check_options_object(ctx, options)) != 0 { 23 } else { 21 }; continue;
}
// C line 59343
25 => {
let _ = { let assigned = if ((((argc) > ((1 as i32))) as i32)) != 0 { *(argv).offset(((1 as i32)) as isize) } else { JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) } }; options = assigned; assigned };
vm_block = 24; continue;
}
// C line 59341
26 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 59340
27 => {
vm_block = if ((!(!(str).is_null()) as i32)) != 0 { 26 } else { 25 }; continue;
}
// C line 59339
28 => {
let _ = { let assigned = JS_ToCStringLen(ctx, core::ptr::addr_of_mut!(str_len), *(argv).offset(((0 as i32)) as isize)); str = assigned; assigned };
vm_block = 27; continue;
}
// C line 59337
29 => {
return JS_ThrowTypeError(ctx, c"expected string".as_ptr());
}
// C line 59336
30 => {
vm_block = if ((!((JS_IsString(*(argv).offset(((0 as i32)) as isize))) != 0) as i32)) != 0 { 29 } else { 28 }; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:59383. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_uint8array_from_hex(mut ctx: *mut JSContext, mut this_val: JSValue, mut argc: i32, mut argv: *mut JSValue) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut str: *const c_char = core::mem::zeroed();
let mut str_len: usize = core::mem::zeroed();
let mut read_pos: usize = core::mem::zeroed();
let mut decoded_len: usize = core::mem::zeroed();
let mut out_cap: usize = core::mem::zeroed();
let mut err: i32 = core::mem::zeroed();
let mut buf: *mut u8 = core::mem::zeroed();
let mut result: JSValue = core::mem::zeroed();
let mut vm_block: usize = 18;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 59417
1 => {
return result;
}
// C line 59416
2 => {
let _ = js_free(ctx, ((buf) as *mut c_void));
vm_block = 1; continue;
}
// C line 59415
3 => {
let _ = { let assigned = JS_NewUint8ArrayCopy(ctx, buf, decoded_len); result = assigned; assigned };
vm_block = 2; continue;
}
// C line 59411
4 => {
return JS_ThrowSyntaxError(ctx, c"invalid hex string".as_ptr());
}
// C line 59410
5 => {
let _ = js_free(ctx, ((buf) as *mut c_void));
vm_block = 4; continue;
}
// C line 59409
6 => {
vm_block = if (err) != 0 { 5 } else { 3 }; continue;
}
// C line 59407
7 => {
let _ = JS_FreeCString(ctx, str);
vm_block = 6; continue;
}
// C line 59406
8 => {
let _ = { let assigned = u8a_hex_decode(str, str_len, buf, out_cap, core::ptr::addr_of_mut!(read_pos), core::ptr::addr_of_mut!(err)); decoded_len = assigned; assigned };
vm_block = 7; continue;
}
// C line 59403
9 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 59402
10 => {
let _ = JS_FreeCString(ctx, str);
vm_block = 9; continue;
}
// C line 59401
11 => {
vm_block = if ((!(!(buf).is_null()) as i32)) != 0 { 10 } else { 8 }; continue;
}
// C line 59400
12 => {
let _ = { let assigned = ((js_malloc(ctx, out_cap)) as *mut u8); buf = assigned; assigned };
vm_block = 11; continue;
}
// C line 59399
13 => {
let _ = { let assigned = (((str_len) / ((((2 as i32)) as usize)))).wrapping_add((((1 as i32)) as usize)); out_cap = assigned; assigned };
vm_block = 12; continue;
}
// C line 59397
14 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 59396
15 => {
vm_block = if ((!(!(str).is_null()) as i32)) != 0 { 14 } else { 13 }; continue;
}
// C line 59395
16 => {
let _ = { let assigned = JS_ToCStringLen(ctx, core::ptr::addr_of_mut!(str_len), *(argv).offset(((0 as i32)) as isize)); str = assigned; assigned };
vm_block = 15; continue;
}
// C line 59393
17 => {
return JS_ThrowTypeError(ctx, c"expected string".as_ptr());
}
// C line 59392
18 => {
vm_block = if ((!((JS_IsString(*(argv).offset(((0 as i32)) as isize))) != 0) as i32)) != 0 { 17 } else { 16 }; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:59421. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_make_read_written(mut ctx: *mut JSContext, mut read: usize, mut written: usize) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut obj: JSValue = core::mem::zeroed();
let mut vm_block: usize = 10;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 59435
1 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line ? labels: fail
2 => {
let _ = JS_FreeValue(ctx, obj);
vm_block = 1; continue;
}
// C line 59432
3 => {
return obj;
}
// C line 59431
4 => {
vm_block = 2; continue;
}
// C line 59429
5 => {
vm_block = if ((((JS_DefinePropertyValueStr(ctx, obj, c"written".as_ptr(), JS_NewUint32(ctx, ((written) as u32)), ((((((1 as i32)).wrapping_shl(((0 as i32)) as u32)) | (((1 as i32)).wrapping_shl(((1 as i32)) as u32)))) | (((1 as i32)).wrapping_shl(((2 as i32)) as u32))))) < ((0 as i32))) as i32)) != 0 { 4 } else { 3 }; continue;
}
// C line 59428
6 => {
vm_block = 2; continue;
}
// C line 59426
7 => {
vm_block = if ((((JS_DefinePropertyValueStr(ctx, obj, c"read".as_ptr(), JS_NewUint32(ctx, ((read) as u32)), ((((((1 as i32)).wrapping_shl(((0 as i32)) as u32)) | (((1 as i32)).wrapping_shl(((1 as i32)) as u32)))) | (((1 as i32)).wrapping_shl(((2 as i32)) as u32))))) < ((0 as i32))) as i32)) != 0 { 6 } else { 5 }; continue;
}
// C line 59425
8 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 59424
9 => {
vm_block = if (JS_IsException(obj)) != 0 { 8 } else { 7 }; continue;
}
// C line 59423
10 => {
obj = JS_NewObject(ctx);
vm_block = 9; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:59439. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_uint8array_set_from_base64(mut ctx: *mut JSContext, mut this_val: JSValue, mut argc: i32, mut argv: *mut JSValue) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut data: *mut u8 = core::mem::zeroed();
let mut len: usize = core::mem::zeroed();
let mut str: *const c_char = core::mem::zeroed();
let mut str_len: usize = core::mem::zeroed();
let mut read_pos: usize = core::mem::zeroed();
let mut decoded_len: usize = core::mem::zeroed();
let mut p: *mut JSObject = core::mem::zeroed();
let mut alphabet: i32 = core::mem::zeroed();
let mut last_chunk: i32 = core::mem::zeroed();
let mut err: i32 = core::mem::zeroed();
let mut options: JSValue = core::mem::zeroed();
let mut vm_block: usize = 28;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 59492
1 => {
return js_make_read_written(ctx, read_pos, decoded_len);
}
// C line 59490
2 => {
return JS_ThrowSyntaxError(ctx, c"invalid base64 string".as_ptr());
}
// C line 59489
3 => {
vm_block = if (err) != 0 { 2 } else { 1 }; continue;
}
// C line 59487
4 => {
let _ = JS_FreeCString(ctx, str);
vm_block = 3; continue;
}
// C line 59483
5 => {
let _ = { let assigned = from_base64(str, str_len, data, len, if ((((alphabet) == ((B64_ALPHABET_BASE64URL as i32))) as i32)) != 0 { (b64url_dec).as_ptr() } else { (b64_dec).as_ptr() }, last_chunk, core::ptr::addr_of_mut!(read_pos), core::ptr::addr_of_mut!(err)); decoded_len = assigned; assigned };
vm_block = 4; continue;
}
// C line 59480
6 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 59479
7 => {
let _ = JS_FreeCString(ctx, str);
vm_block = 6; continue;
}
// C line 59478
8 => {
vm_block = if (get_uint8array_bytes(ctx, p, core::ptr::addr_of_mut!(data), core::ptr::addr_of_mut!(len))) != 0 { 7 } else { 5 }; continue;
}
// C line 59475
9 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 59474
10 => {
let _ = JS_FreeCString(ctx, str);
vm_block = 9; continue;
}
// C line 59473
11 => {
vm_block = if ((((last_chunk) < ((0 as i32))) as i32)) != 0 { 10 } else { 8 }; continue;
}
// C line 59472
12 => {
let _ = { let assigned = parse_last_chunk_option(ctx, options); last_chunk = assigned; assigned };
vm_block = 11; continue;
}
// C line 59470
13 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 59469
14 => {
let _ = JS_FreeCString(ctx, str);
vm_block = 13; continue;
}
// C line 59468
15 => {
vm_block = if ((((alphabet) < ((0 as i32))) as i32)) != 0 { 14 } else { 12 }; continue;
}
// C line 59467
16 => {
let _ = { let assigned = parse_alphabet_option(ctx, options); alphabet = assigned; assigned };
vm_block = 15; continue;
}
// C line 59465
17 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 59464
18 => {
let _ = JS_FreeCString(ctx, str);
vm_block = 17; continue;
}
// C line 59463
19 => {
vm_block = if (check_options_object(ctx, options)) != 0 { 18 } else { 16 }; continue;
}
// C line 59462
20 => {
let _ = { let assigned = if ((((argc) > ((1 as i32))) as i32)) != 0 { *(argv).offset(((1 as i32)) as isize) } else { JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) } }; options = assigned; assigned };
vm_block = 19; continue;
}
// C line 59460
21 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 59459
22 => {
vm_block = if ((!(!(str).is_null()) as i32)) != 0 { 21 } else { 20 }; continue;
}
// C line 59458
23 => {
let _ = { let assigned = JS_ToCStringLen(ctx, core::ptr::addr_of_mut!(str_len), *(argv).offset(((0 as i32)) as isize)); str = assigned; assigned };
vm_block = 22; continue;
}
// C line 59456
24 => {
return JS_ThrowTypeError(ctx, c"expected string".as_ptr());
}
// C line 59455
25 => {
vm_block = if ((!((JS_IsString(*(argv).offset(((0 as i32)) as isize))) != 0) as i32)) != 0 { 24 } else { 23 }; continue;
}
// C line 59453
26 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 59452
27 => {
vm_block = if ((!(!(p).is_null()) as i32)) != 0 { 26 } else { 25 }; continue;
}
// C line 59451
28 => {
let _ = { let assigned = check_uint8array(ctx, this_val); p = assigned; assigned };
vm_block = 27; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:59496. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_uint8array_set_from_hex(mut ctx: *mut JSContext, mut this_val: JSValue, mut argc: i32, mut argv: *mut JSValue) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut data: *mut u8 = core::mem::zeroed();
let mut len: usize = core::mem::zeroed();
let mut str: *const c_char = core::mem::zeroed();
let mut str_len: usize = core::mem::zeroed();
let mut read_pos: usize = core::mem::zeroed();
let mut decoded_len: usize = core::mem::zeroed();
let mut p: *mut JSObject = core::mem::zeroed();
let mut err: i32 = core::mem::zeroed();
let mut vm_block: usize = 16;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 59529
1 => {
return js_make_read_written(ctx, read_pos, decoded_len);
}
// C line 59527
2 => {
return JS_ThrowSyntaxError(ctx, c"invalid hex string".as_ptr());
}
// C line 59526
3 => {
vm_block = if (err) != 0 { 2 } else { 1 }; continue;
}
// C line 59524
4 => {
let _ = JS_FreeCString(ctx, str);
vm_block = 3; continue;
}
// C line 59523
5 => {
let _ = { let assigned = u8a_hex_decode(str, str_len, data, len, core::ptr::addr_of_mut!(read_pos), core::ptr::addr_of_mut!(err)); decoded_len = assigned; assigned };
vm_block = 4; continue;
}
// C line 59520
6 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 59519
7 => {
let _ = JS_FreeCString(ctx, str);
vm_block = 6; continue;
}
// C line 59518
8 => {
vm_block = if (get_uint8array_bytes(ctx, p, core::ptr::addr_of_mut!(data), core::ptr::addr_of_mut!(len))) != 0 { 7 } else { 5 }; continue;
}
// C line 59516
9 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 59515
10 => {
vm_block = if ((!(!(str).is_null()) as i32)) != 0 { 9 } else { 8 }; continue;
}
// C line 59514
11 => {
let _ = { let assigned = JS_ToCStringLen(ctx, core::ptr::addr_of_mut!(str_len), *(argv).offset(((0 as i32)) as isize)); str = assigned; assigned };
vm_block = 10; continue;
}
// C line 59512
12 => {
return JS_ThrowTypeError(ctx, c"expected string".as_ptr());
}
// C line 59511
13 => {
vm_block = if ((!((JS_IsString(*(argv).offset(((0 as i32)) as isize))) != 0) as i32)) != 0 { 12 } else { 11 }; continue;
}
// C line 59509
14 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 59508
15 => {
vm_block = if ((!(!(p).is_null()) as i32)) != 0 { 14 } else { 13 }; continue;
}
// C line 59507
16 => {
let _ = { let assigned = check_uint8array(ctx, this_val); p = assigned; assigned };
vm_block = 15; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:59599. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_typed_array_base_constructor(mut ctx: *mut JSContext, mut this_val: JSValue, mut argc: i32, mut argv: *mut JSValue) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut vm_block: usize = 1;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 59603
1 => {
return JS_ThrowTypeError(ctx, c"cannot be called".as_ptr());
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:59607. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn typed_array_init(mut ctx: *mut JSContext, mut obj: JSValue, mut buffer: JSValue, mut offset: u64, mut len: u64, mut track_rab: i32) -> i32 {
let mut vm_local_storage = Vec::<u64>::new();
let mut ta: *mut JSTypedArray = core::mem::zeroed();
let mut p: *mut JSObject = core::mem::zeroed();
let mut pbuffer: *mut JSObject = core::mem::zeroed();
let mut abuf: *mut JSArrayBuffer = core::mem::zeroed();
let mut size_log2: i32 = core::mem::zeroed();
let mut vm_block: usize = 18;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 59634
1 => {
return (0 as i32);
}
// C line 59633
2 => {
let _ = { let assigned = ((((*(abuf)).data).offset(((offset) as isize))) as *mut c_void); ((((*(p)).u).array).u).ptr = assigned; assigned };
vm_block = 1; continue;
}
// C line 59632
3 => {
let _ = { let assigned = ((len) as u32); (((*(p)).u).array).count = assigned; assigned };
vm_block = 2; continue;
}
// C line 59631
4 => {
let _ = { let assigned = ta; ((*(p)).u).typed_array = assigned; assigned };
vm_block = 3; continue;
}
// C line 59630
5 => {
let _ = list_add_tail(core::ptr::addr_of_mut!((*(ta)).link), core::ptr::addr_of_mut!((*(abuf)).array_list));
vm_block = 4; continue;
}
// C line 59629
6 => {
let _ = { let assigned = track_rab; (*(ta)).track_rab = assigned; assigned };
vm_block = 5; continue;
}
// C line 59628
7 => {
let _ = { let assigned = (((len).wrapping_shl((size_log2) as u32)) as u32); (*(ta)).length = assigned; assigned };
vm_block = 6; continue;
}
// C line 59627
8 => {
let _ = { let assigned = ((offset) as u32); (*(ta)).offset = assigned; assigned };
vm_block = 7; continue;
}
// C line 59626
9 => {
let _ = { let assigned = pbuffer; (*(ta)).buffer = assigned; assigned };
vm_block = 8; continue;
}
// C line 59625
10 => {
let _ = { let assigned = p; (*(ta)).obj = assigned; assigned };
vm_block = 9; continue;
}
// C line 59624
11 => {
let _ = { let assigned = ((*(pbuffer)).u).array_buffer; abuf = assigned; assigned };
vm_block = 10; continue;
}
// C line 59623
12 => {
let _ = { let assigned = ((((buffer).u).ptr) as *mut JSObject); pbuffer = assigned; assigned };
vm_block = 11; continue;
}
// C line 59621
13 => {
return ((1 as i32)).wrapping_neg();
}
// C line 59620
14 => {
let _ = JS_FreeValue(ctx, buffer);
vm_block = 13; continue;
}
// C line 59619
15 => {
vm_block = if ((!(!(ta).is_null()) as i32)) != 0 { 14 } else { 12 }; continue;
}
// C line 59618
16 => {
let _ = { let assigned = ((js_malloc(ctx, (size_of::<JSTypedArray>() as usize))) as *mut JSTypedArray); ta = assigned; assigned };
vm_block = 15; continue;
}
// C line 59617
17 => {
let _ = { let assigned = ((*((typed_array_size_log2).as_ptr()).offset((((((*(p)).class_id) as i32)).wrapping_sub((JS_CLASS_UINT8C_ARRAY as i32))) as isize)) as i32); size_log2 = assigned; assigned };
vm_block = 16; continue;
}
// C line 59616
18 => {
let _ = { let assigned = ((((obj).u).ptr) as *mut JSObject); p = assigned; assigned };
vm_block = 17; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:59638. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_array_from_iterator(mut ctx: *mut JSContext, mut plen: *mut u32, mut obj: JSValue, mut method: JSValue) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut arr: JSValue = core::mem::zeroed();
let mut iter: JSValue = core::mem::zeroed();
let mut next_method: JSValue = core::mem::zeroed();
let mut val: JSValue = core::mem::zeroed();
let mut done: i32 = core::mem::zeroed();
let mut k: u32 = core::mem::zeroed();
let mut vm_block: usize = 29;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 59674
1 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 59673
2 => {
let _ = JS_FreeValue(ctx, arr);
vm_block = 1; continue;
}
// C line 59672
3 => {
let _ = JS_FreeValue(ctx, iter);
vm_block = 2; continue;
}
// C line ? labels: fail
4 => {
let _ = JS_FreeValue(ctx, next_method);
vm_block = 3; continue;
}
// C line 59669
5 => {
return arr;
}
// C line 59668
6 => {
let _ = { let assigned = k; *(plen) = assigned; assigned };
vm_block = 5; continue;
}
// C line 59667
7 => {
let _ = JS_FreeValue(ctx, iter);
vm_block = 6; continue;
}
// C line 59666
8 => {
let _ = JS_FreeValue(ctx, next_method);
vm_block = 7; continue;
}
// C line 59656
9 => {
vm_block = 17; continue;
}
// C line 59664
10 => {
let _ = { let old = k; k = (k).wrapping_add(1); old };
vm_block = 9; continue;
}
// C line 59663
11 => {
vm_block = 4; continue;
}
// C line 59662
12 => {
vm_block = if ((((JS_CreateDataPropertyUint32(ctx, arr, ((k) as i64), val, ((1 as i32)).wrapping_shl(((14 as i32)) as u32))) < ((0 as i32))) as i32)) != 0 { 11 } else { 10 }; continue;
}
// C line 59661
13 => {
vm_block = 8; continue;
}
// C line 59660
14 => {
vm_block = if (done) != 0 { 13 } else { 12 }; continue;
}
// C line 59659
15 => {
vm_block = 4; continue;
}
// C line 59658
16 => {
vm_block = if (JS_IsException(val)) != 0 { 15 } else { 14 }; continue;
}
// C line 59657
17 => {
let _ = { let assigned = JS_IteratorNext(ctx, iter, next_method, (0 as i32), core::ptr::null_mut::<JSValue>(), core::ptr::addr_of_mut!(done)); val = assigned; assigned };
vm_block = 16; continue;
}
// C line 59655
18 => {
let _ = { let assigned = (((0 as i32)) as u32); k = assigned; assigned };
vm_block = 9; continue;
}
// C line 59654
19 => {
vm_block = 4; continue;
}
// C line 59653
20 => {
vm_block = if (JS_IsException(next_method)) != 0 { 19 } else { 18 }; continue;
}
// C line 59652
21 => {
let _ = { let assigned = JS_GetProperty(ctx, iter, (((crate::quickjs_atom::JS_ATOM_next as i32)) as JSAtom)); next_method = assigned; assigned };
vm_block = 20; continue;
}
// C line 59651
22 => {
vm_block = 4; continue;
}
// C line 59650
23 => {
vm_block = if (JS_IsException(iter)) != 0 { 22 } else { 21 }; continue;
}
// C line 59649
24 => {
let _ = { let assigned = JS_GetIterator2(ctx, obj, method); iter = assigned; assigned };
vm_block = 23; continue;
}
// C line 59648
25 => {
return arr;
}
// C line 59647
26 => {
vm_block = if (JS_IsException(arr)) != 0 { 25 } else { 24 }; continue;
}
// C line 59646
27 => {
let _ = { let assigned = JS_NewArray(ctx); arr = assigned; assigned };
vm_block = 26; continue;
}
// C line 59645
28 => {
let _ = { let assigned = (((0 as i32)) as u32); *(plen) = assigned; assigned };
vm_block = 27; continue;
}
// C line 59641
29 => {
next_method = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) };
vm_block = 28; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:59677. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_typed_array_constructor_obj(mut ctx: *mut JSContext, mut new_target: JSValue, mut obj: JSValue, mut classid: i32) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut iter: JSValue = core::mem::zeroed();
let mut ret: JSValue = core::mem::zeroed();
let mut arr: JSValue = core::mem::zeroed();
let mut val: JSValue = core::mem::zeroed();
let mut buffer: JSValue = core::mem::zeroed();
let mut i: u32 = core::mem::zeroed();
let mut size_log2: i32 = core::mem::zeroed();
let mut len: i64 = core::mem::zeroed();
let mut len1: u32 = core::mem::zeroed();
let mut vm_block: usize = 35;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 59728
1 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 59727
2 => {
let _ = JS_FreeValue(ctx, ret);
vm_block = 1; continue;
}
// C line ? labels: fail
3 => {
let _ = JS_FreeValue(ctx, arr);
vm_block = 2; continue;
}
// C line 59724
4 => {
return ret;
}
// C line 59723
5 => {
let _ = JS_FreeValue(ctx, arr);
vm_block = 4; continue;
}
// C line 59716
6 => {
vm_block = if ((((((i) as i64)) < (len)) as i32)) != 0 { 12 } else { 5 }; continue;
}
// C line ?
7 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 6; continue;
}
// C line 59721
8 => {
vm_block = 3; continue;
}
// C line 59720
9 => {
vm_block = if ((((JS_SetPropertyUint32(ctx, ret, i, val)) < ((0 as i32))) as i32)) != 0 { 8 } else { 7 }; continue;
}
// C line 59719
10 => {
vm_block = 3; continue;
}
// C line 59718
11 => {
vm_block = if (JS_IsException(val)) != 0 { 10 } else { 9 }; continue;
}
// C line 59717
12 => {
let _ = { let assigned = JS_GetPropertyUint32(ctx, arr, i); val = assigned; assigned };
vm_block = 11; continue;
}
// C line 59716
13 => {
let _ = { let assigned = (((0 as i32)) as u32); i = assigned; assigned };
vm_block = 6; continue;
}
// C line 59714
14 => {
vm_block = 3; continue;
}
// C line 59713
15 => {
vm_block = if (typed_array_init(ctx, ret, buffer, (((0 as i32)) as u64), ((len) as u64), (0 as i32))) != 0 { 14 } else { 13 }; continue;
}
// C line 59712
16 => {
vm_block = 3; continue;
}
// C line 59711
17 => {
vm_block = if (JS_IsException(buffer)) != 0 { 16 } else { 15 }; continue;
}
// C line 59708
18 => {
let _ = { let assigned = js_array_buffer_constructor1(ctx, JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }, (((len).wrapping_shl((size_log2) as u32)) as u64), core::ptr::null_mut::<u64>()); buffer = assigned; assigned };
vm_block = 17; continue;
}
// C line 59701
19 => {
let _ = { let assigned = ((len1) as i64); len = assigned; assigned };
vm_block = 18; continue;
}
// C line 59700
20 => {
vm_block = 3; continue;
}
// C line 59699
21 => {
vm_block = if (JS_IsException(arr)) != 0 { 20 } else { 19 }; continue;
}
// C line 59698
22 => {
let _ = JS_FreeValue(ctx, iter);
vm_block = 21; continue;
}
// C line 59697
23 => {
let _ = { let assigned = js_array_from_iterator(ctx, core::ptr::addr_of_mut!(len1), obj, iter); arr = assigned; assigned };
vm_block = 22; continue;
}
// C line 59705
24 => {
let _ = { let assigned = JS_DupValue(ctx, obj); arr = assigned; assigned };
vm_block = 18; continue;
}
// C line 59704
25 => {
vm_block = 3; continue;
}
// C line 59703
26 => {
vm_block = if (js_get_length64(ctx, core::ptr::addr_of_mut!(len), obj)) != 0 { 25 } else { 24 }; continue;
}
// C line 59695
27 => {
vm_block = if ((((((!((JS_IsUndefined(iter)) != 0) as i32)) != 0) && (((!((JS_IsNull(iter)) != 0) as i32)) != 0)) as i32)) != 0 { 23 } else { 26 }; continue;
}
// C line 59694
28 => {
vm_block = 3; continue;
}
// C line 59693
29 => {
vm_block = if (JS_IsException(iter)) != 0 { 28 } else { 27 }; continue;
}
// C line 59692
30 => {
let _ = { let assigned = JS_GetProperty(ctx, obj, (((crate::quickjs_atom::JS_ATOM_Symbol_iterator as i32)) as JSAtom)); iter = assigned; assigned };
vm_block = 29; continue;
}
// C line 59690
31 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 59689
32 => {
vm_block = if (JS_IsException(ret)) != 0 { 31 } else { 30 }; continue;
}
// C line 59688
33 => {
let _ = { let assigned = js_create_from_ctor(ctx, new_target, classid); ret = assigned; assigned };
vm_block = 32; continue;
}
// C line 59687
34 => {
let _ = { let assigned = ((*((typed_array_size_log2).as_ptr()).offset(((classid).wrapping_sub((JS_CLASS_UINT8C_ARRAY as i32))) as isize)) as i32); size_log2 = assigned; assigned };
vm_block = 33; continue;
}
// C line 59682
35 => {
arr = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) };
vm_block = 34; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:59731. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_typed_array_constructor_ta(mut ctx: *mut JSContext, mut new_target: JSValue, mut src_obj: JSValue, mut classid: i32, mut len: u32) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut p: *mut JSObject = core::mem::zeroed();
let mut src_buffer: *mut JSObject = core::mem::zeroed();
let mut ta: *mut JSTypedArray = core::mem::zeroed();
let mut obj: JSValue = core::mem::zeroed();
let mut buffer: JSValue = core::mem::zeroed();
let mut i: u32 = core::mem::zeroed();
let mut size_log2: i32 = core::mem::zeroed();
let mut src_abuf: *mut JSArrayBuffer = core::mem::zeroed();
let mut abuf: *mut JSArrayBuffer = core::mem::zeroed();
let mut val: JSValue = core::mem::zeroed();
let mut vm_block: usize = 34;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 59786
1 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line ? labels: fail
2 => {
let _ = JS_FreeValue(ctx, obj);
vm_block = 1; continue;
}
// C line 59783
3 => {
return obj;
}
// C line 59772
4 => {
let _ = { let dst = ((((*(abuf)).data) as *mut c_void)) as *mut u8; core::ptr::copy_nonoverlapping((((((*(src_abuf)).data).offset((((*(ta)).offset) as isize))) as *const c_void)) as *const u8, dst, ((((*(abuf)).byte_length) as usize)) as usize); dst as *mut c_void };
vm_block = 3; continue;
}
// C line 59774
5 => {
vm_block = if ((((i) < (len)) as i32)) != 0 { 11 } else { 3 }; continue;
}
// C line ?
6 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 5; continue;
}
// C line 59780
7 => {
vm_block = 2; continue;
}
// C line 59779
8 => {
vm_block = if ((((JS_SetPropertyUint32(ctx, obj, i, val)) < ((0 as i32))) as i32)) != 0 { 7 } else { 6 }; continue;
}
// C line 59778
9 => {
vm_block = 2; continue;
}
// C line 59777
10 => {
vm_block = if (JS_IsException(val)) != 0 { 9 } else { 8 }; continue;
}
// C line 59776
11 => {
let _ = { let assigned = JS_GetPropertyUint32(ctx, src_obj, i); val = assigned; assigned };
vm_block = 10; continue;
}
// C line 59774
12 => {
let _ = { let assigned = (((0 as i32)) as u32); i = assigned; assigned };
vm_block = 5; continue;
}
// C line 59769
13 => {
vm_block = if (((((((((((*(p)).class_id) as i32)) == (classid)) as i32)) != 0) && (((((((((*(ta)).offset) as i64)).wrapping_add((((*(abuf)).byte_length) as i64))) <= ((((*(src_abuf)).byte_length) as i64))) as i32)) != 0)) as i32)) != 0 { 4 } else { 12 }; continue;
}
// C line 59768
14 => {
let _ = { let assigned = ((*(src_buffer)).u).array_buffer; src_abuf = assigned; assigned };
vm_block = 13; continue;
}
// C line 59767
15 => {
let _ = { let assigned = (*(ta)).buffer; src_buffer = assigned; assigned };
vm_block = 14; continue;
}
// C line 59766
16 => {
let _ = { let assigned = ((*(p)).u).typed_array; ta = assigned; assigned };
vm_block = 15; continue;
}
// C line 59765
17 => {
vm_block = 2; continue;
}
// C line 59764
18 => {
vm_block = if (typed_array_init(ctx, obj, buffer, (((0 as i32)) as u64), ((len) as u64), (0 as i32))) != 0 { 17 } else { 16 }; continue;
}
// C line 59763
19 => {
let _ = { let assigned = ((JS_GetOpaque(buffer, (((JS_CLASS_ARRAY_BUFFER as i32)) as JSClassID))) as *mut JSArrayBuffer); abuf = assigned; assigned };
vm_block = 18; continue;
}
// C line 59761
20 => {
vm_block = 2; continue;
}
// C line 59760
21 => {
let _ = JS_ThrowTypeErrorArrayBufferOOB(ctx);
vm_block = 20; continue;
}
// C line 59759
22 => {
let _ = JS_FreeValue(ctx, buffer);
vm_block = 21; continue;
}
// C line 59758
23 => {
vm_block = if (typed_array_is_oob(p)) != 0 { 22 } else { 19 }; continue;
}
// C line 59756
24 => {
vm_block = 2; continue;
}
// C line 59755
25 => {
vm_block = if (JS_IsException(buffer)) != 0 { 24 } else { 23 }; continue;
}
// C line 59752
26 => {
let _ = { let assigned = js_array_buffer_constructor1(ctx, JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }, (((len) as u64)).wrapping_shl((size_log2) as u32), core::ptr::null_mut::<u64>()); buffer = assigned; assigned };
vm_block = 25; continue;
}
// C line 59751
27 => {
let _ = { let assigned = ((*((typed_array_size_log2).as_ptr()).offset(((classid).wrapping_sub((JS_CLASS_UINT8C_ARRAY as i32))) as isize)) as i32); size_log2 = assigned; assigned };
vm_block = 26; continue;
}
// C line 59749
28 => {
vm_block = 2; continue;
}
// C line 59748
29 => {
let _ = JS_ThrowTypeErrorArrayBufferOOB(ctx);
vm_block = 28; continue;
}
// C line 59747
30 => {
vm_block = if (typed_array_is_oob(p)) != 0 { 29 } else { 27 }; continue;
}
// C line 59746
31 => {
let _ = { let assigned = ((((src_obj).u).ptr) as *mut JSObject); p = assigned; assigned };
vm_block = 30; continue;
}
// C line 59745
32 => {
return obj;
}
// C line 59744
33 => {
vm_block = if (JS_IsException(obj)) != 0 { 32 } else { 31 }; continue;
}
// C line 59743
34 => {
let _ = { let assigned = js_create_from_ctor(ctx, new_target, classid); obj = assigned; assigned };
vm_block = 33; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:59789. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_typed_array_constructor(mut ctx: *mut JSContext, mut new_target: JSValue, mut argc: i32, mut argv: *mut JSValue, mut classid: i32) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut track_rab: i32 = core::mem::zeroed();
let mut buffer: JSValue = core::mem::zeroed();
let mut obj: JSValue = core::mem::zeroed();
let mut abuf: *mut JSArrayBuffer = core::mem::zeroed();
let mut size_log2: i32 = core::mem::zeroed();
let mut len: u64 = core::mem::zeroed();
let mut offset: u64 = core::mem::zeroed();
let mut p: *mut JSObject = core::mem::zeroed();
let mut vm_block: usize = 51;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 59870
1 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line ? labels: fail
2 => {
let _ = JS_FreeValue(ctx, obj);
vm_block = 1; continue;
}
// C line 59867
3 => {
return obj;
}
// C line 59866
4 => {
vm_block = 2; continue;
}
// C line 59865
5 => {
vm_block = if (typed_array_init(ctx, obj, buffer, offset, len, track_rab)) != 0 { 4 } else { 3 }; continue;
}
// C line 59812
6 => {
let _ = { let assigned = (((0 as i32)) as u64); offset = assigned; assigned };
vm_block = 5; continue;
}
// C line 59811
7 => {
vm_block = 2; continue;
}
// C line 59810
8 => {
vm_block = if (JS_IsException(buffer)) != 0 { 7 } else { 6 }; continue;
}
// C line 59807
9 => {
let _ = { let assigned = js_array_buffer_constructor1(ctx, JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }, (len).wrapping_shl((size_log2) as u32), core::ptr::null_mut::<u64>()); buffer = assigned; assigned };
vm_block = 8; continue;
}
// C line 59806
10 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 59805
11 => {
vm_block = if (JS_IsException(obj)) != 0 { 10 } else { 9 }; continue;
}
// C line 59804
12 => {
let _ = { let assigned = js_create_from_ctor(ctx, new_target, classid); obj = assigned; assigned };
vm_block = 11; continue;
}
// C line 59803
13 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 59802
14 => {
vm_block = if (JS_ToIndex(ctx, core::ptr::addr_of_mut!(len), *(argv).offset(((0 as i32)) as isize))) != 0 { 13 } else { 12 }; continue;
}
// C line 59854
15 => {
let _ = { let assigned = JS_DupValue(ctx, *(argv).offset(((0 as i32)) as isize)); buffer = assigned; assigned };
vm_block = 5; continue;
}
// C line 59840
16 => {
let _ = { let assigned = (((((*(abuf)).byte_length) as u64)).wrapping_sub(offset)).wrapping_shr((size_log2) as u32); len = assigned; assigned };
vm_block = 15; continue;
}
// C line 59838
17 => {
vm_block = 28; continue;
}
// C line 59837
18 => {
vm_block = if (((((((*(abuf)).byte_length) & ((((1 as i32)).wrapping_shl((size_log2) as u32)).wrapping_sub((1 as i32))))) != ((0 as i32))) as i32)) != 0 { 17 } else { 16 }; continue;
}
// C line 59836
19 => {
vm_block = if ((!((track_rab) != 0) as i32)) != 0 { 18 } else { 16 }; continue;
}
// C line 59835
20 => {
let _ = { let assigned = array_buffer_is_resizable(abuf); track_rab = assigned; assigned };
vm_block = 19; continue;
}
// C line 59833
21 => {
vm_block = 2; continue;
}
// C line ? labels: invalid_offset
22 => {
let _ = JS_ThrowRangeError(ctx, c"invalid offset".as_ptr());
vm_block = 21; continue;
}
// C line 59830
23 => {
vm_block = if ((((offset) > ((((*(abuf)).byte_length) as u64))) as i32)) != 0 { 22 } else { 20 }; continue;
}
// C line 59828
24 => {
vm_block = 2; continue;
}
// C line 59827
25 => {
let _ = JS_ThrowTypeErrorDetachedArrayBuffer(ctx);
vm_block = 24; continue;
}
// C line 59826
26 => {
vm_block = if ((*(abuf)).detached) != 0 { 25 } else { 23 }; continue;
}
// C line 59851
27 => {
vm_block = 2; continue;
}
// C line ? labels: invalid_length
28 => {
let _ = JS_ThrowRangeError(ctx, c"invalid length".as_ptr());
vm_block = 27; continue;
}
// C line 59848
29 => {
vm_block = if (((((offset).wrapping_add((len).wrapping_shl((size_log2) as u32))) > ((((*(abuf)).byte_length) as u64))) as i32)) != 0 { 28 } else { 15 }; continue;
}
// C line 59846
30 => {
vm_block = 2; continue;
}
// C line 59845
31 => {
let _ = JS_ThrowTypeErrorDetachedArrayBuffer(ctx);
vm_block = 30; continue;
}
// C line 59844
32 => {
vm_block = if ((*(abuf)).detached) != 0 { 31 } else { 29 }; continue;
}
// C line 59843
33 => {
vm_block = 2; continue;
}
// C line 59842
34 => {
vm_block = if (JS_ToIndex(ctx, core::ptr::addr_of_mut!(len), *(argv).offset(((2 as i32)) as isize))) != 0 { 33 } else { 32 }; continue;
}
// C line 59825
35 => {
vm_block = if (JS_IsUndefined(*(argv).offset(((2 as i32)) as isize))) != 0 { 26 } else { 34 }; continue;
}
// C line 59824
36 => {
let _ = { let assigned = ((*(p)).u).array_buffer; abuf = assigned; assigned };
vm_block = 35; continue;
}
// C line 59823
37 => {
vm_block = 22; continue;
}
// C line 59822
38 => {
vm_block = if ((((((offset) & ((((((1 as i32)).wrapping_shl((size_log2) as u32)).wrapping_sub((1 as i32))) as u64)))) != ((((0 as i32)) as u64))) as i32)) != 0 { 37 } else { 36 }; continue;
}
// C line 59821
39 => {
vm_block = 2; continue;
}
// C line 59820
40 => {
vm_block = if (JS_ToIndex(ctx, core::ptr::addr_of_mut!(offset), *(argv).offset(((1 as i32)) as isize))) != 0 { 39 } else { 38 }; continue;
}
// C line 59819
41 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 59818
42 => {
vm_block = if (JS_IsException(obj)) != 0 { 41 } else { 40 }; continue;
}
// C line 59817
43 => {
let _ = { let assigned = js_create_from_ctor(ctx, new_target, classid); obj = assigned; assigned };
vm_block = 42; continue;
}
// C line 59858
44 => {
return js_typed_array_constructor_ta(ctx, new_target, *(argv).offset(((0 as i32)) as isize), classid, (((*(p)).u).array).count);
}
// C line 59861
45 => {
return js_typed_array_constructor_obj(ctx, new_target, *(argv).offset(((0 as i32)) as isize), classid);
}
// C line 59856
46 => {
vm_block = if (((((((((((*(p)).class_id) as i32)) >= ((JS_CLASS_UINT8C_ARRAY as i32))) as i32)) != 0) && ((((((((*(p)).class_id) as i32)) <= ((JS_CLASS_FLOAT64_ARRAY as i32))) as i32)) != 0)) as i32)) != 0 { 44 } else { 45 }; continue;
}
// C line 59815
47 => {
vm_block = if (((((((((((*(p)).class_id) as i32)) == ((JS_CLASS_ARRAY_BUFFER as i32))) as i32)) != 0) || ((((((((*(p)).class_id) as i32)) == ((JS_CLASS_SHARED_ARRAY_BUFFER as i32))) as i32)) != 0)) as i32)) != 0 { 43 } else { 46 }; continue;
}
// C line 59814
48 => {
p = ((((*(argv).offset(((0 as i32)) as isize)).u).ptr) as *mut JSObject);
vm_block = 47; continue;
}
// C line 59801
49 => {
vm_block = if (((((((*(argv).offset(((0 as i32)) as isize)).tag) as i32)) != ((JS_TAG_OBJECT as i32))) as i32)) != 0 { 14 } else { 48 }; continue;
}
// C line 59800
50 => {
let _ = { let assigned = ((*((typed_array_size_log2).as_ptr()).offset(((classid).wrapping_sub((JS_CLASS_UINT8C_ARRAY as i32))) as isize)) as i32); size_log2 = assigned; assigned };
vm_block = 49; continue;
}
// C line 59794
51 => {
track_rab = (0 as i32);
vm_block = 50; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:60804. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
pub unsafe fn JS_AddIntrinsicTypedArrays(mut ctx: *mut JSContext) -> i32 {
let mut vm_local_storage = Vec::<u64>::new();
let mut typed_array_base_func: JSValue = core::mem::zeroed();
let mut typed_array_base_proto: JSValue = core::mem::zeroed();
let mut obj: JSValue = core::mem::zeroed();
let mut i: i32 = core::mem::zeroed();
let mut ret: i32 = core::mem::zeroed();
let mut ft: JSCFunctionType = core::mem::zeroed();
let mut buf: [c_char; 64] = core::mem::zeroed();
let mut name: *const c_char = core::mem::zeroed();
let mut bpe: *const JSCFunctionListEntry = core::mem::zeroed();
let mut vm_block: usize = 42;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 60903
1 => {
return (0 as i32);
}
// C line 60901
2 => {
return ((1 as i32)).wrapping_neg();
}
// C line 60900
3 => {
vm_block = if (if cfg!(feature="atomics") { JS_AddIntrinsicAtomics(ctx) } else { 0 }) != 0 { 2 } else { 1 }; continue;
}
// C line 60896
4 => {
let _ = JS_FreeValue(ctx, obj);
vm_block = 3; continue;
}
// C line 60895
5 => {
return ((1 as i32)).wrapping_neg();
}
// C line 60894
6 => {
vm_block = if (JS_IsException(obj)) != 0 { 5 } else { 4 }; continue;
}
// C line 60888
7 => {
let _ = { let assigned = JS_NewCConstructor(ctx, (JS_CLASS_DATAVIEW as i32), c"DataView".as_ptr(), Some(js_dataview_constructor), (1 as i32), (((JS_CFUNC_constructor as i32)) as JSCFunctionEnum), (0 as i32), JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }, core::ptr::null_mut::<JSCFunctionListEntry>(), (0 as i32), ptr::addr_of!(js_dataview_proto_funcs).cast::<JSCFunctionListEntry>(), (((((size_of::<[JSCFunctionListEntry; 26]>() as usize)) / ((size_of::<JSCFunctionListEntry>() as usize)))) as i32), (0 as i32)); obj = assigned; assigned };
vm_block = 6; continue;
}
// C line 60885
8 => {
let _ = JS_FreeValue(ctx, typed_array_base_func);
vm_block = 7; continue;
}
// C line 60856
9 => {
vm_block = if ((((i) < (((JS_CLASS_UINT8C_ARRAY as i32)).wrapping_add((((JS_CLASS_FLOAT64_ARRAY as i32)).wrapping_sub((JS_CLASS_UINT8C_ARRAY as i32))).wrapping_add((1 as i32))))) as i32)) != 0 { 19 } else { 8 }; continue;
}
// C line ?
10 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 9; continue;
}
// C line 60883
11 => {
let _ = JS_FreeValue(ctx, obj);
vm_block = 10; continue;
}
// C line 60881
12 => {
return ((1 as i32)).wrapping_neg();
}
// C line ? labels: fail
13 => {
let _ = JS_FreeValue(ctx, typed_array_base_func);
vm_block = 12; continue;
}
// C line 60878
14 => {
vm_block = if (JS_IsException(obj)) != 0 { 13 } else { 11 }; continue;
}
// C line 60863
15 => {
let _ = { let assigned = JS_NewCConstructor(ctx, i, name, (ft).generic, (3 as i32), (((JS_CFUNC_constructor_magic as i32)) as JSCFunctionEnum), i, typed_array_base_func, ptr::addr_of!(js_uint8array_funcs).cast::<JSCFunctionListEntry>(), (((((size_of::<[JSCFunctionListEntry; 3]>() as usize)) / ((size_of::<JSCFunctionListEntry>() as usize)))) as i32), ptr::addr_of!(js_uint8array_proto_funcs).cast::<JSCFunctionListEntry>(), (((((size_of::<[JSCFunctionListEntry; 5]>() as usize)) / ((size_of::<JSCFunctionListEntry>() as usize)))) as i32), (0 as i32)); obj = assigned; assigned };
vm_block = 14; continue;
}
// C line 60871
16 => {
let _ = { let assigned = JS_NewCConstructor(ctx, i, name, (ft).generic, (3 as i32), (((JS_CFUNC_constructor_magic as i32)) as JSCFunctionEnum), i, typed_array_base_func, bpe, (1 as i32), bpe, (1 as i32), (0 as i32)); obj = assigned; assigned };
vm_block = 14; continue;
}
// C line 60870
17 => {
bpe = (ptr::addr_of!(js_typed_array_funcs).cast::<JSCFunctionListEntry>()).offset(((((*((typed_array_size_log2).as_ptr()).offset(((i).wrapping_sub((JS_CLASS_UINT8C_ARRAY as i32))) as isize)) as i32)) as isize));
vm_block = 16; continue;
}
// C line 60862
18 => {
vm_block = if ((((i) == ((JS_CLASS_UINT8_ARRAY as i32))) as i32)) != 0 { 15 } else { 17 }; continue;
}
// C line 60860
19 => {
let _ = { let assigned = JS_AtomGetStr(ctx, (buf).as_mut_ptr(), (((size_of::<[c_char; 64]>() as usize)) as i32), (((((crate::quickjs_atom::JS_ATOM_Uint8ClampedArray as i32)).wrapping_add(i)).wrapping_sub((JS_CLASS_UINT8C_ARRAY as i32))) as JSAtom)); name = assigned; assigned };
vm_block = 18; continue;
}
// C line 60856
20 => {
let _ = { let assigned = (JS_CLASS_UINT8C_ARRAY as i32); i = assigned; assigned };
vm_block = 9; continue;
}
// C line 60855
21 => {
ft = JSCFunctionType { generic_magic: Some(js_typed_array_constructor) };
vm_block = 20; continue;
}
// C line 60852
22 => {
vm_block = 13; continue;
}
// C line 60851
23 => {
vm_block = if ((((ret) < ((0 as i32))) as i32)) != 0 { 22 } else { 21 }; continue;
}
// C line 60850
24 => {
let _ = JS_FreeValue(ctx, typed_array_base_proto);
vm_block = 23; continue;
}
// C line 60848
25 => {
let _ = { let assigned = JS_DefinePropertyValue(ctx, typed_array_base_proto, (((crate::quickjs_atom::JS_ATOM_toString as i32)) as JSAtom), obj, ((((1 as i32)).wrapping_shl(((1 as i32)) as u32)) | (((1 as i32)).wrapping_shl(((0 as i32)) as u32)))); ret = assigned; assigned };
vm_block = 24; continue;
}
// C line 60847
26 => {
vm_block = 13; continue;
}
// C line 60846
27 => {
vm_block = if (JS_IsException(typed_array_base_proto)) != 0 { 26 } else { 25 }; continue;
}
// C line 60845
28 => {
let _ = { let assigned = JS_GetProperty(ctx, typed_array_base_func, (((crate::quickjs_atom::JS_ATOM_prototype as i32)) as JSAtom)); typed_array_base_proto = assigned; assigned };
vm_block = 27; continue;
}
// C line 60843
29 => {
vm_block = 13; continue;
}
// C line 60842
30 => {
vm_block = if (JS_IsException(obj)) != 0 { 29 } else { 28 }; continue;
}
// C line 60841
31 => {
let _ = { let assigned = JS_GetProperty(ctx, *((*(ctx)).class_proto).offset(((JS_CLASS_ARRAY as i32)) as isize), (((crate::quickjs_atom::JS_ATOM_toString as i32)) as JSAtom)); obj = assigned; assigned };
vm_block = 30; continue;
}
// C line 60838
32 => {
return ((1 as i32)).wrapping_neg();
}
// C line 60837
33 => {
vm_block = if (JS_IsException(typed_array_base_func)) != 0 { 32 } else { 31 }; continue;
}
// C line 60830
34 => {
let _ = { let assigned = JS_NewCConstructor(ctx, ((1 as i32)).wrapping_neg(), c"TypedArray".as_ptr(), Some(js_typed_array_base_constructor), (0 as i32), (((JS_CFUNC_constructor_or_func as i32)) as JSCFunctionEnum), (0 as i32), JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }, ptr::addr_of!(js_typed_array_base_funcs).cast::<JSCFunctionListEntry>(), (((((size_of::<[JSCFunctionListEntry; 3]>() as usize)) / ((size_of::<JSCFunctionListEntry>() as usize)))) as i32), ptr::addr_of!(js_typed_array_base_proto_funcs).cast::<JSCFunctionListEntry>(), (((((size_of::<[JSCFunctionListEntry; 36]>() as usize)) / ((size_of::<JSCFunctionListEntry>() as usize)))) as i32), ((1 as i32)).wrapping_shl(((0 as i32)) as u32)); typed_array_base_func = assigned; assigned };
vm_block = 33; continue;
}
// C line 60827
35 => {
let _ = JS_FreeValue(ctx, obj);
vm_block = 34; continue;
}
// C line 60826
36 => {
return ((1 as i32)).wrapping_neg();
}
// C line 60825
37 => {
vm_block = if (JS_IsException(obj)) != 0 { 36 } else { 35 }; continue;
}
// C line 60819
38 => {
let _ = { let assigned = JS_NewCConstructor(ctx, (JS_CLASS_SHARED_ARRAY_BUFFER as i32), c"SharedArrayBuffer".as_ptr(), Some(js_shared_array_buffer_constructor), (1 as i32), (((JS_CFUNC_constructor as i32)) as JSCFunctionEnum), (0 as i32), JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }, ptr::addr_of!(js_shared_array_buffer_funcs).cast::<JSCFunctionListEntry>(), (((((size_of::<[JSCFunctionListEntry; 1]>() as usize)) / ((size_of::<JSCFunctionListEntry>() as usize)))) as i32), ptr::addr_of!(js_shared_array_buffer_proto_funcs).cast::<JSCFunctionListEntry>(), (((((size_of::<[JSCFunctionListEntry; 6]>() as usize)) / ((size_of::<JSCFunctionListEntry>() as usize)))) as i32), (0 as i32)); obj = assigned; assigned };
vm_block = 37; continue;
}
// C line 60817
39 => {
let _ = JS_FreeValue(ctx, obj);
vm_block = 38; continue;
}
// C line 60816
40 => {
return ((1 as i32)).wrapping_neg();
}
// C line 60815
41 => {
vm_block = if (JS_IsException(obj)) != 0 { 40 } else { 39 }; continue;
}
// C line 60809
42 => {
let _ = { let assigned = JS_NewCConstructor(ctx, (JS_CLASS_ARRAY_BUFFER as i32), c"ArrayBuffer".as_ptr(), Some(js_array_buffer_constructor), (1 as i32), (((JS_CFUNC_constructor as i32)) as JSCFunctionEnum), (0 as i32), JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }, ptr::addr_of!(js_array_buffer_funcs).cast::<JSCFunctionListEntry>(), (((((size_of::<[JSCFunctionListEntry; 2]>() as usize)) / ((size_of::<JSCFunctionListEntry>() as usize)))) as i32), ptr::addr_of!(js_array_buffer_proto_funcs).cast::<JSCFunctionListEntry>(), (((((size_of::<[JSCFunctionListEntry; 9]>() as usize)) / ((size_of::<JSCFunctionListEntry>() as usize)))) as i32), (0 as i32)); obj = assigned; assigned };
vm_block = 41; continue;
}
_ => std::process::abort(),
} }
}
