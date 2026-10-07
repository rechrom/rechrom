// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:60299. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_atomics_get_buf(mut ctx: *mut JSContext, mut obj: JSValue, mut idx_val: JSValue, mut pidx: *mut u64, mut is_waitable: i32) -> *mut JSObject {
let mut vm_local_storage = Vec::<u64>::new();
let mut p: *mut JSObject = core::mem::zeroed();
let mut ta: *mut JSTypedArray = core::mem::zeroed();
let mut abuf: *mut JSArrayBuffer = core::mem::zeroed();
let mut idx: u64 = core::mem::zeroed();
let mut err: i32 = core::mem::zeroed();
let mut old_len: i32 = core::mem::zeroed();
let mut vm_block: usize = 32;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 60359
1 => {
return p;
}
// C line 60358
2 => {
let _ = { let assigned = idx; *(pidx) = assigned; assigned };
vm_block = 1; continue;
}
// C line 60354
3 => {
return core::ptr::null_mut::<JSObject>();
}
// C line ? labels: oob
4 => {
let _ = JS_ThrowRangeError(ctx, c"out-of-bound access".as_ptr());
vm_block = 3; continue;
}
// C line 60351
5 => {
vm_block = if ((((idx) >= ((((((*(p)).u).array).count) as u64))) as i32)) != 0 { 4 } else { 2 }; continue;
}
// C line 60349
6 => {
return core::ptr::null_mut::<JSObject>();
}
// C line 60348
7 => {
let _ = JS_ThrowTypeErrorArrayBufferOOB(ctx);
vm_block = 6; continue;
}
// C line 60347
8 => {
vm_block = if (typed_array_is_oob(p)) != 0 { 7 } else { 5 }; continue;
}
// C line 60345
9 => {
vm_block = if ((((is_waitable) != ((1 as i32))) as i32)) != 0 { 8 } else { 2 }; continue;
}
// C line 60343
10 => {
vm_block = 4; continue;
}
// C line 60342
11 => {
vm_block = if ((((idx) >= (((old_len) as u64))) as i32)) != 0 { 10 } else { 9 }; continue;
}
// C line 60339
12 => {
return core::ptr::null_mut::<JSObject>();
}
// C line 60338
13 => {
vm_block = if (JS_ToIndex(ctx, core::ptr::addr_of_mut!(idx), idx_val)) != 0 { 12 } else { 11 }; continue;
}
// C line 60336
14 => {
let _ = { let assigned = (((((*(p)).u).array).count) as i32); old_len = assigned; assigned };
vm_block = 13; continue;
}
// C line 60333
15 => {
return core::ptr::null_mut::<JSObject>();
}
// C line 60332
16 => {
let _ = JS_ThrowTypeErrorDetachedArrayBuffer(ctx);
vm_block = 15; continue;
}
// C line 60331
17 => {
vm_block = if ((*(abuf)).detached) != 0 { 16 } else { 14 }; continue;
}
// C line 60329
18 => {
return core::ptr::null_mut::<JSObject>();
}
// C line 60328
19 => {
let _ = JS_ThrowTypeError(ctx, c"not a SharedArrayBuffer TypedArray".as_ptr());
vm_block = 18; continue;
}
// C line 60327
20 => {
vm_block = if ((((is_waitable) == ((2 as i32))) as i32)) != 0 { 19 } else { 17 }; continue;
}
// C line 60326
21 => {
vm_block = if ((!(((*(abuf)).shared) != 0) as i32)) != 0 { 20 } else { 14 }; continue;
}
// C line 60325
22 => {
let _ = { let assigned = ((*((*(ta)).buffer)).u).array_buffer; abuf = assigned; assigned };
vm_block = 21; continue;
}
// C line 60324
23 => {
let _ = { let assigned = ((*(p)).u).typed_array; ta = assigned; assigned };
vm_block = 22; continue;
}
// C line 60322
24 => {
return core::ptr::null_mut::<JSObject>();
}
// C line ? labels: fail
25 => {
let _ = JS_ThrowTypeError(ctx, c"integer TypedArray expected".as_ptr());
vm_block = 24; continue;
}
// C line 60319
26 => {
vm_block = if (err) != 0 { 25 } else { 23 }; continue;
}
// C line 60314
27 => {
let _ = { let assigned = ((((((((((*(p)).class_id) as i32)) != ((JS_CLASS_INT32_ARRAY as i32))) as i32)) != 0) && ((((((((*(p)).class_id) as i32)) != ((JS_CLASS_BIG_INT64_ARRAY as i32))) as i32)) != 0)) as i32); err = assigned; assigned };
vm_block = 26; continue;
}
// C line 60317
28 => {
let _ = { let assigned = (!((((((((((((*(p)).class_id) as i32)) >= ((JS_CLASS_INT8_ARRAY as i32))) as i32)) != 0) && ((((((((*(p)).class_id) as i32)) <= ((JS_CLASS_BIG_UINT64_ARRAY as i32))) as i32)) != 0)) as i32)) != 0) as i32); err = assigned; assigned };
vm_block = 26; continue;
}
// C line 60313
29 => {
vm_block = if (is_waitable) != 0 { 27 } else { 28 }; continue;
}
// C line 60312
30 => {
let _ = { let assigned = ((((obj).u).ptr) as *mut JSObject); p = assigned; assigned };
vm_block = 29; continue;
}
// C line 60311
31 => {
vm_block = 25; continue;
}
// C line 60310
32 => {
vm_block = if (((((((obj).tag) as i32)) != ((JS_TAG_OBJECT as i32))) as i32)) != 0 { 31 } else { 30 }; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:60362. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_atomics_op(mut ctx: *mut JSContext, mut this_obj: JSValue, mut argc: i32, mut argv: *mut JSValue, mut op: i32) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut size_log2: i32 = core::mem::zeroed();
let mut v: u64 = core::mem::zeroed();
let mut a: u64 = core::mem::zeroed();
let mut rep_val: u64 = core::mem::zeroed();
let mut idx: u64 = core::mem::zeroed();
let mut ptr: *mut c_void = core::mem::zeroed();
let mut ret: JSValue = core::mem::zeroed();
let mut p: *mut JSObject = core::mem::zeroed();
let mut v64: i64 = core::mem::zeroed();
let mut v32: u32 = core::mem::zeroed();
let mut v1: u8 = core::mem::zeroed();
let mut v1_1: u16 = core::mem::zeroed();
let mut v1_2: u32 = core::mem::zeroed();
let mut v1_3: u64 = core::mem::zeroed();
let mut vm_block: usize = 120;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 60506
1 => {
return ret;
}
// C line ?
2 => {
let _ = std::process::abort();
vm_block = 1; continue;
}
// C line 60502
3 => {
vm_block = 1; continue;
}
// C line 60501
4 => {
let _ = { let assigned = JS_NewBigUint64(ctx, a); ret = assigned; assigned };
vm_block = 3; continue;
}
// C line 60499
5 => {
vm_block = 1; continue;
}
// C line 60498
6 => {
let _ = { let assigned = JS_NewBigInt64(ctx, ((a) as i64)); ret = assigned; assigned };
vm_block = 5; continue;
}
// C line 60496
7 => {
vm_block = 1; continue;
}
// C line 60495
8 => {
let _ = { let assigned = JS_NewUint32(ctx, ((a) as u32)); ret = assigned; assigned };
vm_block = 7; continue;
}
// C line 60493
9 => {
vm_block = 1; continue;
}
// C line ? labels: done
10 => {
let _ = { let assigned = JS_NewInt32(ctx, ((a) as i32)); ret = assigned; assigned };
vm_block = 9; continue;
}
// C line 60489
11 => {
vm_block = 10; continue;
}
// C line 60488
12 => {
let _ = { let assigned = ((((a) as u16)) as u64); a = assigned; assigned };
vm_block = 11; continue;
}
// C line 60486
13 => {
vm_block = 10; continue;
}
// C line 60485
14 => {
let _ = { let assigned = ((((a) as i16)) as u64); a = assigned; assigned };
vm_block = 13; continue;
}
// C line 60483
15 => {
vm_block = 10; continue;
}
// C line 60482
16 => {
let _ = { let assigned = ((((a) as u8)) as u64); a = assigned; assigned };
vm_block = 15; continue;
}
// C line 60480
17 => {
vm_block = 10; continue;
}
// C line 60479
18 => {
let _ = { let assigned = ((((a) as i8)) as u64); a = assigned; assigned };
vm_block = 17; continue;
}
// C line 60477
19 => {
vm_block = match (((*(p)).class_id) as i32) { x if x == (JS_CLASS_BIG_UINT64_ARRAY as i32) => 4, x if x == (JS_CLASS_BIG_INT64_ARRAY as i32) => 6, x if x == (JS_CLASS_UINT32_ARRAY as i32) => 8, x if x == (JS_CLASS_INT32_ARRAY as i32) => 10, x if x == (JS_CLASS_UINT16_ARRAY as i32) => 12, x if x == (JS_CLASS_INT16_ARRAY as i32) => 14, x if x == (JS_CLASS_UINT8_ARRAY as i32) => 16, x if x == (JS_CLASS_INT8_ARRAY as i32) => 18, _ => 2, }; continue;
}
// C line ?
20 => {
let _ = std::process::abort();
vm_block = 19; continue;
}
// C line 60472
21 => {
vm_block = 19; continue;
}
// C line 60470
22 => {
let _ = { let assigned = v1_3; a = assigned; assigned };
vm_block = 21; continue;
}
// C line 60469
23 => {
let _ = { let expected = core::ptr::addr_of_mut!(v1_3); match (&*(((ptr) as *mut core::sync::atomic::AtomicU64))).compare_exchange(*expected, rep_val, core::sync::atomic::Ordering::SeqCst, core::sync::atomic::Ordering::SeqCst) { Ok(_) => true, Err(previous) => { *expected = previous; false } } };
vm_block = 22; continue;
}
// C line 60468
24 => {
v1_3 = v;
vm_block = 23; continue;
}
// C line 60465
25 => {
vm_block = 19; continue;
}
// C line 60463
26 => {
let _ = { let assigned = ((v1_2) as u64); a = assigned; assigned };
vm_block = 25; continue;
}
// C line 60462
27 => {
let _ = { let expected = core::ptr::addr_of_mut!(v1_2); match (&*(((ptr) as *mut core::sync::atomic::AtomicU32))).compare_exchange(*expected, ((rep_val) as u32), core::sync::atomic::Ordering::SeqCst, core::sync::atomic::Ordering::SeqCst) { Ok(_) => true, Err(previous) => { *expected = previous; false } } };
vm_block = 26; continue;
}
// C line 60461
28 => {
v1_2 = ((v) as u32);
vm_block = 27; continue;
}
// C line 60458
29 => {
vm_block = 19; continue;
}
// C line 60456
30 => {
let _ = { let assigned = ((v1_1) as u64); a = assigned; assigned };
vm_block = 29; continue;
}
// C line 60455
31 => {
let _ = { let expected = core::ptr::addr_of_mut!(v1_1); match (&*(((ptr) as *mut core::sync::atomic::AtomicU16))).compare_exchange(*expected, ((rep_val) as u16), core::sync::atomic::Ordering::SeqCst, core::sync::atomic::Ordering::SeqCst) { Ok(_) => true, Err(previous) => { *expected = previous; false } } };
vm_block = 30; continue;
}
// C line 60454
32 => {
v1_1 = ((v) as u16);
vm_block = 31; continue;
}
// C line 60451
33 => {
vm_block = 19; continue;
}
// C line 60449
34 => {
let _ = { let assigned = ((v1) as u64); a = assigned; assigned };
vm_block = 33; continue;
}
// C line 60448
35 => {
let _ = { let expected = core::ptr::addr_of_mut!(v1); match (&*(((ptr) as *mut core::sync::atomic::AtomicU8))).compare_exchange(*expected, ((rep_val) as u8), core::sync::atomic::Ordering::SeqCst, core::sync::atomic::Ordering::SeqCst) { Ok(_) => true, Err(previous) => { *expected = previous; false } } };
vm_block = 34; continue;
}
// C line 60447
36 => {
v1 = ((v) as u8);
vm_block = 35; continue;
}
// C line 60443
37 => {
vm_block = 19; continue;
}
// C line 60442
38 => {
let _ = { let assigned = (&*(((ptr) as *mut core::sync::atomic::AtomicU64))).load(core::sync::atomic::Ordering::SeqCst); a = assigned; assigned };
vm_block = 37; continue;
}
// C line 60440
39 => {
vm_block = 19; continue;
}
// C line 60439
40 => {
let _ = { let assigned = (((&*(((ptr) as *mut core::sync::atomic::AtomicU32))).load(core::sync::atomic::Ordering::SeqCst)) as u64); a = assigned; assigned };
vm_block = 39; continue;
}
// C line 60437
41 => {
vm_block = 19; continue;
}
// C line 60436
42 => {
let _ = { let assigned = (((&*(((ptr) as *mut core::sync::atomic::AtomicU16))).load(core::sync::atomic::Ordering::SeqCst)) as u64); a = assigned; assigned };
vm_block = 41; continue;
}
// C line 60434
43 => {
vm_block = 19; continue;
}
// C line 60433
44 => {
let _ = { let assigned = (((&*(((ptr) as *mut core::sync::atomic::AtomicU8))).load(core::sync::atomic::Ordering::SeqCst)) as u64); a = assigned; assigned };
vm_block = 43; continue;
}
// C line 60429
45 => {
vm_block = 19; continue;
}
// C line 60429
46 => {
let _ = { let assigned = (&*(((ptr) as *mut core::sync::atomic::AtomicU64))).swap(v, core::sync::atomic::Ordering::SeqCst); a = assigned; assigned };
vm_block = 45; continue;
}
// C line 60429
47 => {
vm_block = 19; continue;
}
// C line 60429
48 => {
let _ = { let assigned = (((&*(((ptr) as *mut core::sync::atomic::AtomicU32))).swap(((v) as u32), core::sync::atomic::Ordering::SeqCst)) as u64); a = assigned; assigned };
vm_block = 47; continue;
}
// C line 60429
49 => {
vm_block = 19; continue;
}
// C line 60429
50 => {
let _ = { let assigned = (((&*(((ptr) as *mut core::sync::atomic::AtomicU16))).swap(((v) as u16), core::sync::atomic::Ordering::SeqCst)) as u64); a = assigned; assigned };
vm_block = 49; continue;
}
// C line 60429
51 => {
vm_block = 19; continue;
}
// C line 60429
52 => {
let _ = { let assigned = (((&*(((ptr) as *mut core::sync::atomic::AtomicU8))).swap(((v) as u8), core::sync::atomic::Ordering::SeqCst)) as u64); a = assigned; assigned };
vm_block = 51; continue;
}
// C line 60428
53 => {
vm_block = 19; continue;
}
// C line 60428
54 => {
let _ = { let assigned = (&*(((ptr) as *mut core::sync::atomic::AtomicU64))).fetch_xor(v, core::sync::atomic::Ordering::SeqCst); a = assigned; assigned };
vm_block = 53; continue;
}
// C line 60428
55 => {
vm_block = 19; continue;
}
// C line 60428
56 => {
let _ = { let assigned = (((&*(((ptr) as *mut core::sync::atomic::AtomicU32))).fetch_xor(((v) as u32), core::sync::atomic::Ordering::SeqCst)) as u64); a = assigned; assigned };
vm_block = 55; continue;
}
// C line 60428
57 => {
vm_block = 19; continue;
}
// C line 60428
58 => {
let _ = { let assigned = (((&*(((ptr) as *mut core::sync::atomic::AtomicU16))).fetch_xor(((v) as u16), core::sync::atomic::Ordering::SeqCst)) as u64); a = assigned; assigned };
vm_block = 57; continue;
}
// C line 60428
59 => {
vm_block = 19; continue;
}
// C line 60428
60 => {
let _ = { let assigned = (((&*(((ptr) as *mut core::sync::atomic::AtomicU8))).fetch_xor(((v) as u8), core::sync::atomic::Ordering::SeqCst)) as u64); a = assigned; assigned };
vm_block = 59; continue;
}
// C line 60427
61 => {
vm_block = 19; continue;
}
// C line 60427
62 => {
let _ = { let assigned = (&*(((ptr) as *mut core::sync::atomic::AtomicU64))).fetch_sub(v, core::sync::atomic::Ordering::SeqCst); a = assigned; assigned };
vm_block = 61; continue;
}
// C line 60427
63 => {
vm_block = 19; continue;
}
// C line 60427
64 => {
let _ = { let assigned = (((&*(((ptr) as *mut core::sync::atomic::AtomicU32))).fetch_sub(((v) as u32), core::sync::atomic::Ordering::SeqCst)) as u64); a = assigned; assigned };
vm_block = 63; continue;
}
// C line 60427
65 => {
vm_block = 19; continue;
}
// C line 60427
66 => {
let _ = { let assigned = (((&*(((ptr) as *mut core::sync::atomic::AtomicU16))).fetch_sub(((v) as u16), core::sync::atomic::Ordering::SeqCst)) as u64); a = assigned; assigned };
vm_block = 65; continue;
}
// C line 60427
67 => {
vm_block = 19; continue;
}
// C line 60427
68 => {
let _ = { let assigned = (((&*(((ptr) as *mut core::sync::atomic::AtomicU8))).fetch_sub(((v) as u8), core::sync::atomic::Ordering::SeqCst)) as u64); a = assigned; assigned };
vm_block = 67; continue;
}
// C line 60426
69 => {
vm_block = 19; continue;
}
// C line 60426
70 => {
let _ = { let assigned = (&*(((ptr) as *mut core::sync::atomic::AtomicU64))).fetch_or(v, core::sync::atomic::Ordering::SeqCst); a = assigned; assigned };
vm_block = 69; continue;
}
// C line 60426
71 => {
vm_block = 19; continue;
}
// C line 60426
72 => {
let _ = { let assigned = (((&*(((ptr) as *mut core::sync::atomic::AtomicU32))).fetch_or(((v) as u32), core::sync::atomic::Ordering::SeqCst)) as u64); a = assigned; assigned };
vm_block = 71; continue;
}
// C line 60426
73 => {
vm_block = 19; continue;
}
// C line 60426
74 => {
let _ = { let assigned = (((&*(((ptr) as *mut core::sync::atomic::AtomicU16))).fetch_or(((v) as u16), core::sync::atomic::Ordering::SeqCst)) as u64); a = assigned; assigned };
vm_block = 73; continue;
}
// C line 60426
75 => {
vm_block = 19; continue;
}
// C line 60426
76 => {
let _ = { let assigned = (((&*(((ptr) as *mut core::sync::atomic::AtomicU8))).fetch_or(((v) as u8), core::sync::atomic::Ordering::SeqCst)) as u64); a = assigned; assigned };
vm_block = 75; continue;
}
// C line 60425
77 => {
vm_block = 19; continue;
}
// C line 60425
78 => {
let _ = { let assigned = (&*(((ptr) as *mut core::sync::atomic::AtomicU64))).fetch_and(v, core::sync::atomic::Ordering::SeqCst); a = assigned; assigned };
vm_block = 77; continue;
}
// C line 60425
79 => {
vm_block = 19; continue;
}
// C line 60425
80 => {
let _ = { let assigned = (((&*(((ptr) as *mut core::sync::atomic::AtomicU32))).fetch_and(((v) as u32), core::sync::atomic::Ordering::SeqCst)) as u64); a = assigned; assigned };
vm_block = 79; continue;
}
// C line 60425
81 => {
vm_block = 19; continue;
}
// C line 60425
82 => {
let _ = { let assigned = (((&*(((ptr) as *mut core::sync::atomic::AtomicU16))).fetch_and(((v) as u16), core::sync::atomic::Ordering::SeqCst)) as u64); a = assigned; assigned };
vm_block = 81; continue;
}
// C line 60425
83 => {
vm_block = 19; continue;
}
// C line 60425
84 => {
let _ = { let assigned = (((&*(((ptr) as *mut core::sync::atomic::AtomicU8))).fetch_and(((v) as u8), core::sync::atomic::Ordering::SeqCst)) as u64); a = assigned; assigned };
vm_block = 83; continue;
}
// C line 60424
85 => {
vm_block = 19; continue;
}
// C line 60424
86 => {
let _ = { let assigned = (&*(((ptr) as *mut core::sync::atomic::AtomicU64))).fetch_add(v, core::sync::atomic::Ordering::SeqCst); a = assigned; assigned };
vm_block = 85; continue;
}
// C line 60424
87 => {
vm_block = 19; continue;
}
// C line 60424
88 => {
let _ = { let assigned = (((&*(((ptr) as *mut core::sync::atomic::AtomicU32))).fetch_add(((v) as u32), core::sync::atomic::Ordering::SeqCst)) as u64); a = assigned; assigned };
vm_block = 87; continue;
}
// C line 60424
89 => {
vm_block = 19; continue;
}
// C line 60424
90 => {
let _ = { let assigned = (((&*(((ptr) as *mut core::sync::atomic::AtomicU16))).fetch_add(((v) as u16), core::sync::atomic::Ordering::SeqCst)) as u64); a = assigned; assigned };
vm_block = 89; continue;
}
// C line 60424
91 => {
vm_block = 19; continue;
}
// C line 60424
92 => {
let _ = { let assigned = (((&*(((ptr) as *mut core::sync::atomic::AtomicU8))).fetch_add(((v) as u8), core::sync::atomic::Ordering::SeqCst)) as u64); a = assigned; assigned };
vm_block = 91; continue;
}
// C line 60408
93 => {
vm_block = match ((op) | ((size_log2).wrapping_shl(((3 as i32)) as u32))) { x if x == (((ATOMICS_OP_COMPARE_EXCHANGE as i32)) | (((3 as i32)).wrapping_shl(((3 as i32)) as u32))) => 24, x if x == (((ATOMICS_OP_COMPARE_EXCHANGE as i32)) | (((2 as i32)).wrapping_shl(((3 as i32)) as u32))) => 28, x if x == (((ATOMICS_OP_COMPARE_EXCHANGE as i32)) | (((1 as i32)).wrapping_shl(((3 as i32)) as u32))) => 32, x if x == (((ATOMICS_OP_COMPARE_EXCHANGE as i32)) | (((0 as i32)).wrapping_shl(((3 as i32)) as u32))) => 36, x if x == (((ATOMICS_OP_LOAD as i32)) | (((3 as i32)).wrapping_shl(((3 as i32)) as u32))) => 38, x if x == (((ATOMICS_OP_LOAD as i32)) | (((2 as i32)).wrapping_shl(((3 as i32)) as u32))) => 40, x if x == (((ATOMICS_OP_LOAD as i32)) | (((1 as i32)).wrapping_shl(((3 as i32)) as u32))) => 42, x if x == (((ATOMICS_OP_LOAD as i32)) | (((0 as i32)).wrapping_shl(((3 as i32)) as u32))) => 44, x if x == (((ATOMICS_OP_EXCHANGE as i32)) | (((3 as i32)).wrapping_shl(((3 as i32)) as u32))) => 46, x if x == (((ATOMICS_OP_EXCHANGE as i32)) | (((2 as i32)).wrapping_shl(((3 as i32)) as u32))) => 48, x if x == (((ATOMICS_OP_EXCHANGE as i32)) | (((1 as i32)).wrapping_shl(((3 as i32)) as u32))) => 50, x if x == (((ATOMICS_OP_EXCHANGE as i32)) | (((0 as i32)).wrapping_shl(((3 as i32)) as u32))) => 52, x if x == (((ATOMICS_OP_XOR as i32)) | (((3 as i32)).wrapping_shl(((3 as i32)) as u32))) => 54, x if x == (((ATOMICS_OP_XOR as i32)) | (((2 as i32)).wrapping_shl(((3 as i32)) as u32))) => 56, x if x == (((ATOMICS_OP_XOR as i32)) | (((1 as i32)).wrapping_shl(((3 as i32)) as u32))) => 58, x if x == (((ATOMICS_OP_XOR as i32)) | (((0 as i32)).wrapping_shl(((3 as i32)) as u32))) => 60, x if x == (((ATOMICS_OP_SUB as i32)) | (((3 as i32)).wrapping_shl(((3 as i32)) as u32))) => 62, x if x == (((ATOMICS_OP_SUB as i32)) | (((2 as i32)).wrapping_shl(((3 as i32)) as u32))) => 64, x if x == (((ATOMICS_OP_SUB as i32)) | (((1 as i32)).wrapping_shl(((3 as i32)) as u32))) => 66, x if x == (((ATOMICS_OP_SUB as i32)) | (((0 as i32)).wrapping_shl(((3 as i32)) as u32))) => 68, x if x == (((ATOMICS_OP_OR as i32)) | (((3 as i32)).wrapping_shl(((3 as i32)) as u32))) => 70, x if x == (((ATOMICS_OP_OR as i32)) | (((2 as i32)).wrapping_shl(((3 as i32)) as u32))) => 72, x if x == (((ATOMICS_OP_OR as i32)) | (((1 as i32)).wrapping_shl(((3 as i32)) as u32))) => 74, x if x == (((ATOMICS_OP_OR as i32)) | (((0 as i32)).wrapping_shl(((3 as i32)) as u32))) => 76, x if x == (((ATOMICS_OP_AND as i32)) | (((3 as i32)).wrapping_shl(((3 as i32)) as u32))) => 78, x if x == (((ATOMICS_OP_AND as i32)) | (((2 as i32)).wrapping_shl(((3 as i32)) as u32))) => 80, x if x == (((ATOMICS_OP_AND as i32)) | (((1 as i32)).wrapping_shl(((3 as i32)) as u32))) => 82, x if x == (((ATOMICS_OP_AND as i32)) | (((0 as i32)).wrapping_shl(((3 as i32)) as u32))) => 84, x if x == (((ATOMICS_OP_ADD as i32)) | (((3 as i32)).wrapping_shl(((3 as i32)) as u32))) => 86, x if x == (((ATOMICS_OP_ADD as i32)) | (((2 as i32)).wrapping_shl(((3 as i32)) as u32))) => 88, x if x == (((ATOMICS_OP_ADD as i32)) | (((1 as i32)).wrapping_shl(((3 as i32)) as u32))) => 90, x if x == (((ATOMICS_OP_ADD as i32)) | (((0 as i32)).wrapping_shl(((3 as i32)) as u32))) => 92, _ => 20, }; continue;
}
// C line 60406
94 => {
let _ = { let assigned = (((((((*(p)).u).array).u).uint8_ptr).offset((((((idx) as usize)).wrapping_shl((size_log2) as u32)) as isize))) as *mut c_void); ptr = assigned; assigned };
vm_block = 93; continue;
}
// C line 60378
95 => {
let _ = { let assigned = (((0 as i32)) as u64); v = assigned; assigned };
vm_block = 94; continue;
}
// C line 60404
96 => {
return JS_ThrowRangeError(ctx, c"out-of-bound access".as_ptr());
}
// C line 60403
97 => {
vm_block = if ((((idx) >= ((((((*(p)).u).array).count) as u64))) as i32)) != 0 { 96 } else { 94 }; continue;
}
// C line 60402
98 => {
return JS_ThrowTypeErrorDetachedArrayBuffer(ctx);
}
// C line 60401
99 => {
vm_block = if (typed_array_is_oob(p)) != 0 { 98 } else { 97 }; continue;
}
// C line 60388
100 => {
let _ = { let assigned = ((v64) as u64); rep_val = assigned; assigned };
vm_block = 99; continue;
}
// C line 60387
101 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 60386
102 => {
vm_block = if (JS_ToBigInt64(ctx, core::ptr::addr_of_mut!(v64), *(argv).offset(((3 as i32)) as isize))) != 0 { 101 } else { 100 }; continue;
}
// C line 60385
103 => {
vm_block = if ((((op) == ((ATOMICS_OP_COMPARE_EXCHANGE as i32))) as i32)) != 0 { 102 } else { 99 }; continue;
}
// C line 60384
104 => {
let _ = { let assigned = ((v64) as u64); v = assigned; assigned };
vm_block = 103; continue;
}
// C line 60383
105 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 60382
106 => {
vm_block = if (JS_ToBigInt64(ctx, core::ptr::addr_of_mut!(v64), *(argv).offset(((2 as i32)) as isize))) != 0 { 105 } else { 104 }; continue;
}
// C line 60398
107 => {
let _ = { let assigned = ((v32) as u64); rep_val = assigned; assigned };
vm_block = 99; continue;
}
// C line 60397
108 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 60396
109 => {
vm_block = if (JS_ToUint32(ctx, core::ptr::addr_of_mut!(v32), *(argv).offset(((3 as i32)) as isize))) != 0 { 108 } else { 107 }; continue;
}
// C line 60395
110 => {
vm_block = if ((((op) == ((ATOMICS_OP_COMPARE_EXCHANGE as i32))) as i32)) != 0 { 109 } else { 99 }; continue;
}
// C line 60394
111 => {
let _ = { let assigned = ((v32) as u64); v = assigned; assigned };
vm_block = 110; continue;
}
// C line 60393
112 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 60392
113 => {
vm_block = if (JS_ToUint32(ctx, core::ptr::addr_of_mut!(v32), *(argv).offset(((2 as i32)) as isize))) != 0 { 112 } else { 111 }; continue;
}
// C line 60380
114 => {
vm_block = if ((((size_log2) == ((3 as i32))) as i32)) != 0 { 106 } else { 113 }; continue;
}
// C line 60377
115 => {
vm_block = if ((((op) == ((ATOMICS_OP_LOAD as i32))) as i32)) != 0 { 95 } else { 114 }; continue;
}
// C line 60376
116 => {
let _ = { let assigned = (((0 as i32)) as u64); rep_val = assigned; assigned };
vm_block = 115; continue;
}
// C line 60375
117 => {
let _ = { let assigned = ((*((typed_array_size_log2).as_ptr()).offset((((((*(p)).class_id) as i32)).wrapping_sub((JS_CLASS_UINT8C_ARRAY as i32))) as isize)) as i32); size_log2 = assigned; assigned };
vm_block = 116; continue;
}
// C line 60374
118 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 60373
119 => {
vm_block = if ((!(!(p).is_null()) as i32)) != 0 { 118 } else { 117 }; continue;
}
// C line 60372
120 => {
let _ = { let assigned = js_atomics_get_buf(ctx, *(argv).offset(((0 as i32)) as isize), *(argv).offset(((1 as i32)) as isize), core::ptr::addr_of_mut!(idx), (0 as i32)); p = assigned; assigned };
vm_block = 119; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:60509. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_atomics_store(mut ctx: *mut JSContext, mut this_obj: JSValue, mut argc: i32, mut argv: *mut JSValue) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut size_log2: i32 = core::mem::zeroed();
let mut ptr: *mut c_void = core::mem::zeroed();
let mut ret: JSValue = core::mem::zeroed();
let mut p: *mut JSObject = core::mem::zeroed();
let mut idx: u64 = core::mem::zeroed();
let mut v: i64 = core::mem::zeroed();
let mut v32: u32 = core::mem::zeroed();
let mut vm_block: usize = 34;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 60567
1 => {
return ret;
}
// C line ?
2 => {
let _ = std::process::abort();
vm_block = 1; continue;
}
// C line 60563
3 => {
vm_block = 1; continue;
}
// C line 60562
4 => {
let _ = (&*(((ptr) as *mut core::sync::atomic::AtomicU64))).store(((v) as u64), core::sync::atomic::Ordering::SeqCst);
vm_block = 3; continue;
}
// C line 60560
5 => {
vm_block = 1; continue;
}
// C line 60559
6 => {
let _ = (&*(((ptr) as *mut core::sync::atomic::AtomicU32))).store(((v) as u32), core::sync::atomic::Ordering::SeqCst);
vm_block = 5; continue;
}
// C line 60557
7 => {
vm_block = 1; continue;
}
// C line 60556
8 => {
let _ = (&*(((ptr) as *mut core::sync::atomic::AtomicU16))).store(((v) as u16), core::sync::atomic::Ordering::SeqCst);
vm_block = 7; continue;
}
// C line 60554
9 => {
vm_block = 1; continue;
}
// C line 60553
10 => {
let _ = (&*(((ptr) as *mut core::sync::atomic::AtomicU8))).store(((v) as u8), core::sync::atomic::Ordering::SeqCst);
vm_block = 9; continue;
}
// C line 60551
11 => {
vm_block = match size_log2 { x if x == (3 as i32) => 4, x if x == (2 as i32) => 6, x if x == (1 as i32) => 8, x if x == (0 as i32) => 10, _ => 2, }; continue;
}
// C line 60549
12 => {
let _ = { let assigned = (((((((*(p)).u).array).u).uint8_ptr).offset((((((idx) as usize)).wrapping_shl((size_log2) as u32)) as isize))) as *mut c_void); ptr = assigned; assigned };
vm_block = 11; continue;
}
// C line 60547
13 => {
return JS_ThrowRangeError(ctx, c"out-of-bound access".as_ptr());
}
// C line 60546
14 => {
vm_block = if ((((idx) >= ((((((*(p)).u).array).count) as u64))) as i32)) != 0 { 13 } else { 12 }; continue;
}
// C line 60545
15 => {
return JS_ThrowTypeErrorDetachedArrayBuffer(ctx);
}
// C line 60544
16 => {
vm_block = if (typed_array_is_oob(p)) != 0 { 15 } else { 14 }; continue;
}
// C line 60530
17 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 60529
18 => {
let _ = JS_FreeValue(ctx, ret);
vm_block = 17; continue;
}
// C line 60528
19 => {
vm_block = if (JS_ToBigInt64(ctx, core::ptr::addr_of_mut!(v), ret)) != 0 { 18 } else { 16 }; continue;
}
// C line 60527
20 => {
return ret;
}
// C line 60526
21 => {
vm_block = if (JS_IsException(ret)) != 0 { 20 } else { 19 }; continue;
}
// C line 60525
22 => {
let _ = { let assigned = JS_ToBigIntFree(ctx, JS_DupValue(ctx, *(argv).offset(((2 as i32)) as isize))); ret = assigned; assigned };
vm_block = 21; continue;
}
// C line 60542
23 => {
let _ = { let assigned = ((v32) as i64); v = assigned; assigned };
vm_block = 16; continue;
}
// C line 60540
24 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 60539
25 => {
let _ = JS_FreeValue(ctx, ret);
vm_block = 24; continue;
}
// C line 60538
26 => {
vm_block = if (JS_ToUint32(ctx, core::ptr::addr_of_mut!(v32), ret)) != 0 { 25 } else { 23 }; continue;
}
// C line 60537
27 => {
return ret;
}
// C line 60536
28 => {
vm_block = if (JS_IsException(ret)) != 0 { 27 } else { 26 }; continue;
}
// C line 60535
29 => {
let _ = { let assigned = JS_ToIntegerFree(ctx, JS_DupValue(ctx, *(argv).offset(((2 as i32)) as isize))); ret = assigned; assigned };
vm_block = 28; continue;
}
// C line 60524
30 => {
vm_block = if ((((size_log2) == ((3 as i32))) as i32)) != 0 { 22 } else { 29 }; continue;
}
// C line 60523
31 => {
let _ = { let assigned = ((*((typed_array_size_log2).as_ptr()).offset((((((*(p)).class_id) as i32)).wrapping_sub((JS_CLASS_UINT8C_ARRAY as i32))) as isize)) as i32); size_log2 = assigned; assigned };
vm_block = 30; continue;
}
// C line 60522
32 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 60521
33 => {
vm_block = if ((!(!(p).is_null()) as i32)) != 0 { 32 } else { 31 }; continue;
}
// C line 60520
34 => {
let _ = { let assigned = js_atomics_get_buf(ctx, *(argv).offset(((0 as i32)) as isize), *(argv).offset(((1 as i32)) as isize), core::ptr::addr_of_mut!(idx), (0 as i32)); p = assigned; assigned };
vm_block = 33; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:60570. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_atomics_isLockFree(mut ctx: *mut JSContext, mut this_obj: JSValue, mut argc: i32, mut argv: *mut JSValue) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut v: i32 = core::mem::zeroed();
let mut ret: i32 = core::mem::zeroed();
let mut vm_block: usize = 4;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 60578
1 => {
return JS_NewBool(ctx, ret);
}
// C line 60577
2 => {
let _ = { let assigned = (((((((((((((((v) == ((1 as i32))) as i32)) != 0) || (((((v) == ((2 as i32))) as i32)) != 0)) as i32)) != 0) || (((((v) == ((4 as i32))) as i32)) != 0)) as i32)) != 0) || (((((v) == ((8 as i32))) as i32)) != 0)) as i32); ret = assigned; assigned };
vm_block = 1; continue;
}
// C line 60576
3 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 60575
4 => {
vm_block = if (JS_ToInt32Sat(ctx, core::ptr::addr_of_mut!(v), *(argv).offset(((0 as i32)) as isize))) != 0 { 3 } else { 2 }; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:60611. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_atomics_pause(mut ctx: *mut JSContext, mut this_obj: JSValue, mut argc: i32, mut argv: *mut JSValue) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut d: f64 = core::mem::zeroed();
let mut vm_block: usize = 10;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 60632
1 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) };
}
// C line 60631
2 => {
let _ = cpu_pause();
vm_block = 1; continue;
}
// C line 60628
3 => {
vm_block = 2; continue;
}
// C line ?
4 => {
return JS_ThrowTypeError(ctx, c"not an integral number".as_ptr());
}
// C line 60622
5 => {
vm_block = 2; continue;
}
// C line 60621
6 => {
vm_block = if (((((((0 as i32)) as f64)) == (buffer_modf(d, core::ptr::addr_of_mut!(d)))) as i32)) != 0 { 5 } else { 4 }; continue;
}
// C line 60620
7 => {
vm_block = if (if (((((size_of::<f64>() as usize)) == ((size_of::<f32>() as usize))) as i32)) != 0 { (((d) as f32)).is_finite() as i32 } else { if (((((size_of::<f64>() as usize)) == ((size_of::<f64>() as usize))) as i32)) != 0 { (d).is_finite() as i32 } else { (d).is_finite() as i32 } }) != 0 { 6 } else { 4 }; continue;
}
// C line 60619
8 => {
let _ = { let assigned = ((*(argv).offset(((0 as i32)) as isize)).u).float64; d = assigned; assigned };
vm_block = 7; continue;
}
// C line 60617
9 => {
vm_block = match (((*(argv).offset(((0 as i32)) as isize)).tag) as i32) { x if x == (JS_TAG_INT as i32) => 3, x if x == (JS_TAG_UNDEFINED as i32) => 3, x if x == (JS_TAG_FLOAT64 as i32) => 8, _ => 4, }; continue;
}
// C line 60616
10 => {
vm_block = if ((((argc) > ((0 as i32))) as i32)) != 0 { 9 } else { 2 }; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:60796. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn JS_AddIntrinsicAtomics(mut ctx: *mut JSContext) -> i32 {
let mut vm_local_storage = Vec::<u64>::new();
let mut vm_block: usize = 1;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 60799
1 => {
return JS_SetPropertyFunctionList(ctx, (*(ctx)).global_obj, ptr::addr_of!(js_atomics_obj).cast::<JSCFunctionListEntry>(), (((((size_of::<[JSCFunctionListEntry; 1]>() as usize)) / ((size_of::<JSCFunctionListEntry>() as usize)))) as i32));
}
_ => std::process::abort(),
} }
}
