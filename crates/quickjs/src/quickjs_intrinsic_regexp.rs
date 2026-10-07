// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:47338. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code, unused_variables)]
unsafe fn js_compile_regexp(mut ctx: *mut JSContext, mut pattern: JSValue, mut flags: JSValue) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut str: *const c_char = core::mem::zeroed();
let mut re_flags: i32 = core::mem::zeroed();
let mut mask: i32 = core::mem::zeroed();
let mut re_bytecode_buf: *mut u8 = core::mem::zeroed();
let mut i: usize = core::mem::zeroed();
let mut len: usize = core::mem::zeroed();
let mut re_bytecode_len: i32 = core::mem::zeroed();
let mut ret: JSValue = core::mem::zeroed();
let mut error_msg: [c_char; 64] = core::mem::zeroed();
let mut vm_block: usize = 44;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 47413
1 => {
return ret;
}
// C line 47412
2 => {
let _ = js_free(ctx, ((re_bytecode_buf) as *mut c_void));
vm_block = 1; continue;
}
// C line 47411
3 => {
let _ = { let assigned = js_new_string8_len(ctx, ((re_bytecode_buf) as *const c_char), re_bytecode_len); ret = assigned; assigned };
vm_block = 2; continue;
}
// C line 47408
4 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 47407
5 => {
let _ = JS_ThrowSyntaxError(ctx, JSErrorMessage::Pieces(&[core::ffi::CStr::from_ptr((error_msg).as_mut_ptr()).to_bytes()]));
vm_block = 4; continue;
}
// C line 47406
6 => {
vm_block = if ((!(!(re_bytecode_buf).is_null()) as i32)) != 0 { 5 } else { 3 }; continue;
}
// C line 47405
7 => {
let _ = JS_FreeCString(ctx, str);
vm_block = 6; continue;
}
// C line 47403
8 => {
let _ = { let assigned = lre_compile(core::ptr::addr_of_mut!(re_bytecode_len), (error_msg).as_mut_ptr(), (((size_of::<[c_char; 64]>() as usize)) as i32), str, len, re_flags, ((ctx) as *mut c_void)); re_bytecode_buf = assigned; assigned };
vm_block = 7; continue;
}
// C line 47402
9 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 47401
10 => {
vm_block = if ((!(!(str).is_null()) as i32)) != 0 { 9 } else { 8 }; continue;
}
// C line 47400
11 => {
let _ = { let assigned = JS_ToCStringLen2(ctx, core::ptr::addr_of_mut!(len), pattern, (!((((re_flags) & (((((1 as i32)).wrapping_shl(((4 as i32)) as u32)) | (((1 as i32)).wrapping_shl(((8 as i32)) as u32)))))) != 0) as i32)); str = assigned; assigned };
vm_block = 10; continue;
}
// C line ? labels: bad_flags1
12 => {
return JS_ThrowSyntaxError(ctx, c"invalid regular expression flags".as_ptr());
}
// C line 47395
13 => {
vm_block = if (((((((re_flags) & (((1 as i32)).wrapping_shl(((8 as i32)) as u32)))) != 0) && ((((re_flags) & (((1 as i32)).wrapping_shl(((4 as i32)) as u32)))) != 0)) as i32)) != 0 { 12 } else { 11 }; continue;
}
// C line 47391
14 => {
let _ = JS_FreeCString(ctx, str);
vm_block = 13; continue;
}
// C line 47355
15 => {
vm_block = if ((((i) < (len)) as i32)) != 0 { 38 } else { 14 }; continue;
}
// C line ?
16 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 15; continue;
}
// C line 47389
17 => {
let _ = { re_flags = ((re_flags) | (mask)); re_flags };
vm_block = 16; continue;
}
// C line 47387
18 => {
vm_block = 12; continue;
}
// C line ? labels: bad_flags
19 => {
let _ = JS_FreeCString(ctx, str);
vm_block = 18; continue;
}
// C line 47384
20 => {
vm_block = if ((((((re_flags) & (mask))) != ((0 as i32))) as i32)) != 0 { 19 } else { 17 }; continue;
}
// C line ?
21 => {
vm_block = 19; continue;
}
// C line 47380
22 => {
vm_block = 20; continue;
}
// C line 47379
23 => {
let _ = { let assigned = ((1 as i32)).wrapping_shl(((5 as i32)) as u32); mask = assigned; assigned };
vm_block = 22; continue;
}
// C line 47377
24 => {
vm_block = 20; continue;
}
// C line 47376
25 => {
let _ = { let assigned = ((1 as i32)).wrapping_shl(((8 as i32)) as u32); mask = assigned; assigned };
vm_block = 24; continue;
}
// C line 47374
26 => {
vm_block = 20; continue;
}
// C line 47373
27 => {
let _ = { let assigned = ((1 as i32)).wrapping_shl(((4 as i32)) as u32); mask = assigned; assigned };
vm_block = 26; continue;
}
// C line 47371
28 => {
vm_block = 20; continue;
}
// C line 47370
29 => {
let _ = { let assigned = ((1 as i32)).wrapping_shl(((3 as i32)) as u32); mask = assigned; assigned };
vm_block = 28; continue;
}
// C line 47368
30 => {
vm_block = 20; continue;
}
// C line 47367
31 => {
let _ = { let assigned = ((1 as i32)).wrapping_shl(((2 as i32)) as u32); mask = assigned; assigned };
vm_block = 30; continue;
}
// C line 47365
32 => {
vm_block = 20; continue;
}
// C line 47364
33 => {
let _ = { let assigned = ((1 as i32)).wrapping_shl(((1 as i32)) as u32); mask = assigned; assigned };
vm_block = 32; continue;
}
// C line 47362
34 => {
vm_block = 20; continue;
}
// C line 47361
35 => {
let _ = { let assigned = ((1 as i32)).wrapping_shl(((0 as i32)) as u32); mask = assigned; assigned };
vm_block = 34; continue;
}
// C line 47359
36 => {
vm_block = 20; continue;
}
// C line 47358
37 => {
let _ = { let assigned = ((1 as i32)).wrapping_shl(((6 as i32)) as u32); mask = assigned; assigned };
vm_block = 36; continue;
}
// C line 47356
38 => {
vm_block = match ((*(str).offset((i) as isize)) as i32) { x if x == (121 as i32) => 23, x if x == (118 as i32) => 25, x if x == (117 as i32) => 27, x if x == (115 as i32) => 29, x if x == (109 as i32) => 31, x if x == (105 as i32) => 33, x if x == (103 as i32) => 35, x if x == (100 as i32) => 37, _ => 21, }; continue;
}
// C line 47355
39 => {
let _ = { let assigned = (((0 as i32)) as usize); i = assigned; assigned };
vm_block = 15; continue;
}
// C line 47353
40 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 47352
41 => {
vm_block = if ((!(!(str).is_null()) as i32)) != 0 { 40 } else { 39 }; continue;
}
// C line 47351
42 => {
let _ = { let assigned = JS_ToCStringLen(ctx, core::ptr::addr_of_mut!(len), flags); str = assigned; assigned };
vm_block = 41; continue;
}
// C line 47350
43 => {
vm_block = if ((!((JS_IsUndefined(flags)) != 0) as i32)) != 0 { 42 } else { 13 }; continue;
}
// C line 47349
44 => {
let _ = { let assigned = (0 as i32); re_flags = assigned; assigned };
vm_block = 43; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:47417. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code, unused_variables)]
unsafe fn JS_NewRegexp(mut ctx: *mut JSContext, mut pattern: JSValue, mut bc: JSValue) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut obj: JSValue = core::mem::zeroed();
let mut props: [JSProperty; 1] = core::mem::zeroed();
let mut p: *mut JSObject = core::mem::zeroed();
let mut re: *mut JSRegExp = core::mem::zeroed();
let mut vm_block: usize = 15;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 47442
1 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 47441
2 => {
let _ = JS_FreeValue(ctx, pattern);
vm_block = 1; continue;
}
// C line ? labels: fail
3 => {
let _ = JS_FreeValue(ctx, bc);
vm_block = 2; continue;
}
// C line 47438
4 => {
return obj;
}
// C line 47437
5 => {
let _ = { let assigned = ((((bc).u).ptr) as *mut JSString); (*(re)).bytecode = assigned; assigned };
vm_block = 4; continue;
}
// C line 47436
6 => {
let _ = { let assigned = ((((pattern).u).ptr) as *mut JSString); (*(re)).pattern = assigned; assigned };
vm_block = 5; continue;
}
// C line 47435
7 => {
let _ = { let assigned = core::ptr::addr_of_mut!(((*(p)).u).regexp); re = assigned; assigned };
vm_block = 6; continue;
}
// C line 47434
8 => {
let _ = { let assigned = ((((obj).u).ptr) as *mut JSObject); p = assigned; assigned };
vm_block = 7; continue;
}
// C line 47433
9 => {
vm_block = 3; continue;
}
// C line 47432
10 => {
vm_block = if (JS_IsException(obj)) != 0 { 9 } else { 8 }; continue;
}
// C line 47431
11 => {
let _ = { let assigned = JS_NewObjectFromShape(ctx, js_dup_shape((*(ctx)).regexp_shape), (((JS_CLASS_REGEXP as i32)) as JSClassID), (props).as_mut_ptr()); obj = assigned; assigned };
vm_block = 10; continue;
}
// C line 47430
12 => {
let _ = { let assigned = JS_NewInt32(ctx, (0 as i32)); ((*((props).as_mut_ptr()).offset(((0 as i32)) as isize)).u).value = assigned; assigned };
vm_block = 11; continue;
}
// C line 47428
13 => {
vm_block = 3; continue;
}
// C line 47427
14 => {
let _ = JS_ThrowTypeError(ctx, c"string expected".as_ptr());
vm_block = 13; continue;
}
// C line 47425
15 => {
vm_block = if ((((!(((!((((((((((((bc).tag) as i32)) != ((JS_TAG_STRING as i32))) as i32)) != 0) || ((((((((pattern).tag) as i32)) != ((JS_TAG_STRING as i32))) as i32)) != 0)) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 14 } else { 12 }; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:47446. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code, unused_variables)]
unsafe fn js_regexp_set_internal(mut ctx: *mut JSContext, mut obj: JSValue, mut pattern: JSValue, mut bc: JSValue) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut p: *mut JSObject = core::mem::zeroed();
let mut re: *mut JSRegExp = core::mem::zeroed();
let mut vm_block: usize = 12;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 47470
1 => {
return obj;
}
// C line 47468
2 => {
let _ = JS_DefinePropertyValue(ctx, obj, (((crate::quickjs_atom::JS_ATOM_lastIndex as i32)) as JSAtom), JS_NewInt32(ctx, (0 as i32)), ((1 as i32)).wrapping_shl(((1 as i32)) as u32));
vm_block = 1; continue;
}
// C line 47466
3 => {
let _ = { let assigned = ((((bc).u).ptr) as *mut JSString); (*(re)).bytecode = assigned; assigned };
vm_block = 2; continue;
}
// C line 47465
4 => {
let _ = { let assigned = ((((pattern).u).ptr) as *mut JSString); (*(re)).pattern = assigned; assigned };
vm_block = 3; continue;
}
// C line 47464
5 => {
let _ = { let assigned = core::ptr::addr_of_mut!(((*(p)).u).regexp); re = assigned; assigned };
vm_block = 4; continue;
}
// C line 47463
6 => {
let _ = { let assigned = ((((obj).u).ptr) as *mut JSObject); p = assigned; assigned };
vm_block = 5; continue;
}
// C line 47460
7 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 47459
8 => {
let _ = JS_FreeValue(ctx, pattern);
vm_block = 7; continue;
}
// C line 47458
9 => {
let _ = JS_FreeValue(ctx, bc);
vm_block = 8; continue;
}
// C line 47457
10 => {
let _ = JS_FreeValue(ctx, obj);
vm_block = 9; continue;
}
// C line 47456
11 => {
let _ = JS_ThrowTypeError(ctx, c"string expected".as_ptr());
vm_block = 10; continue;
}
// C line 47454
12 => {
vm_block = if ((((!(((!((((((((((((bc).tag) as i32)) != ((JS_TAG_STRING as i32))) as i32)) != 0) || ((((((((pattern).tag) as i32)) != ((JS_TAG_STRING as i32))) as i32)) != 0)) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 11 } else { 6 }; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:47473. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code, unused_variables)]
unsafe fn js_get_regexp(mut ctx: *mut JSContext, mut obj: JSValue, mut throw_error: i32) -> *mut JSRegExp {
let mut vm_local_storage = Vec::<u64>::new();
let mut p: *mut JSObject = core::mem::zeroed();
let mut vm_block: usize = 7;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 47483
1 => {
return core::ptr::null_mut::<JSRegExp>();
}
// C line 47481
2 => {
let _ = JS_ThrowTypeErrorInvalidClass(ctx, (JS_CLASS_REGEXP as i32));
vm_block = 1; continue;
}
// C line 47480
3 => {
vm_block = if (throw_error) != 0 { 2 } else { 1 }; continue;
}
// C line 47478
4 => {
return core::ptr::addr_of_mut!(((*(p)).u).regexp);
}
// C line 47477
5 => {
vm_block = if (((((((*(p)).class_id) as i32)) == ((JS_CLASS_REGEXP as i32))) as i32)) != 0 { 4 } else { 3 }; continue;
}
// C line 47476
6 => {
p = ((((obj).u).ptr) as *mut JSObject);
vm_block = 5; continue;
}
// C line 47475
7 => {
vm_block = if (((((((obj).tag) as i32)) == ((JS_TAG_OBJECT as i32))) as i32)) != 0 { 6 } else { 3 }; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:47487. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code, unused_variables)]
unsafe fn js_is_regexp(mut ctx: *mut JSContext, mut obj: JSValue) -> i32 {
let mut vm_local_storage = Vec::<u64>::new();
let mut m: JSValue = core::mem::zeroed();
let mut vm_block: usize = 8;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 47498
1 => {
return (((js_get_regexp(ctx, obj, (0 as i32))) != (core::ptr::null_mut::<JSRegExp>())) as i32);
}
// C line 47497
2 => {
return JS_ToBoolFree(ctx, m);
}
// C line 47496
3 => {
vm_block = if ((!((JS_IsUndefined(m)) != 0) as i32)) != 0 { 2 } else { 1 }; continue;
}
// C line 47495
4 => {
return ((1 as i32)).wrapping_neg();
}
// C line 47494
5 => {
vm_block = if (JS_IsException(m)) != 0 { 4 } else { 3 }; continue;
}
// C line 47493
6 => {
let _ = { let assigned = JS_GetProperty(ctx, obj, (((crate::quickjs_atom::JS_ATOM_Symbol_match as i32)) as JSAtom)); m = assigned; assigned };
vm_block = 5; continue;
}
// C line 47492
7 => {
return (0 as i32);
}
// C line 47491
8 => {
vm_block = if ((!((JS_IsObject(obj)) != 0) as i32)) != 0 { 7 } else { 6 }; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:47501. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code, unused_variables)]
unsafe fn js_regexp_constructor(mut ctx: *mut JSContext, mut new_target: JSValue, mut argc: i32, mut argv: *mut JSValue) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut pattern: JSValue = core::mem::zeroed();
let mut flags: JSValue = core::mem::zeroed();
let mut bc: JSValue = core::mem::zeroed();
let mut val: JSValue = core::mem::zeroed();
let mut obj: JSValue = core::mem::zeroed();
let mut pat: JSValue = core::mem::zeroed();
let mut flags1: JSValue = core::mem::zeroed();
let mut re: *mut JSRegExp = core::mem::zeroed();
let mut pat_is_regexp: i32 = core::mem::zeroed();
let mut ctor: JSValue = core::mem::zeroed();
let mut res: i32 = core::mem::zeroed();
let mut vm_block: usize = 57;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 47581
1 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 47580
2 => {
let _ = JS_FreeValue(ctx, obj);
vm_block = 1; continue;
}
// C line 47579
3 => {
let _ = JS_FreeValue(ctx, flags);
vm_block = 2; continue;
}
// C line ? labels: fail
4 => {
let _ = JS_FreeValue(ctx, pattern);
vm_block = 3; continue;
}
// C line ? labels: no_compilation
5 => {
return js_regexp_set_internal(ctx, obj, pattern, bc);
}
// C line 47574
6 => {
let _ = JS_FreeValue(ctx, flags);
vm_block = 5; continue;
}
// C line 47573
7 => {
vm_block = 4; continue;
}
// C line 47572
8 => {
vm_block = if (JS_IsException(bc)) != 0 { 7 } else { 6 }; continue;
}
// C line 47571
9 => {
let _ = { let assigned = js_compile_regexp(ctx, pattern, flags); bc = assigned; assigned };
vm_block = 8; continue;
}
// C line 47570
10 => {
vm_block = 4; continue;
}
// C line 47569
11 => {
vm_block = if (JS_IsException(obj)) != 0 { 10 } else { 9 }; continue;
}
// C line 47568
12 => {
let _ = { let assigned = js_create_from_ctor(ctx, new_target, (JS_CLASS_REGEXP as i32)); obj = assigned; assigned };
vm_block = 11; continue;
}
// C line 47538
13 => {
vm_block = 5; continue;
}
// C line 47537
14 => {
vm_block = 4; continue;
}
// C line 47536
15 => {
vm_block = if (JS_IsException(obj)) != 0 { 14 } else { 13 }; continue;
}
// C line 47535
16 => {
let _ = { let assigned = js_create_from_ctor(ctx, new_target, (JS_CLASS_REGEXP as i32)); obj = assigned; assigned };
vm_block = 15; continue;
}
// C line 47534
17 => {
let _ = { let assigned = JS_DupValue(ctx, JSValue { u: JSValueUnion { ptr: (((*(re)).bytecode) as *mut c_void) }, tag: (((JS_TAG_STRING as i32)) as i64) }); bc = assigned; assigned };
vm_block = 16; continue;
}
// C line 47540
18 => {
let _ = { let assigned = JS_DupValue(ctx, flags1); flags = assigned; assigned };
vm_block = 12; continue;
}
// C line 47533
19 => {
vm_block = if (JS_IsUndefined(flags1)) != 0 { 17 } else { 18 }; continue;
}
// C line 47532
20 => {
let _ = { let assigned = JS_DupValue(ctx, JSValue { u: JSValueUnion { ptr: (((*(re)).pattern) as *mut c_void) }, tag: (((JS_TAG_STRING as i32)) as i64) }); pattern = assigned; assigned };
vm_block = 19; continue;
}
// C line 47559
21 => {
let _ = { let assigned = JS_AtomToString(ctx, (((crate::quickjs_atom::JS_ATOM_empty_string as i32)) as JSAtom)); pattern = assigned; assigned };
vm_block = 12; continue;
}
// C line 47565
22 => {
vm_block = 4; continue;
}
// C line 47564
23 => {
vm_block = if (JS_IsException(pattern)) != 0 { 22 } else { 12 }; continue;
}
// C line 47563
24 => {
let _ = JS_FreeValue(ctx, val);
vm_block = 23; continue;
}
// C line 47562
25 => {
let _ = { let assigned = JS_ToString(ctx, val); pattern = assigned; assigned };
vm_block = 24; continue;
}
// C line 47561
26 => {
let _ = { let assigned = pattern; val = assigned; assigned };
vm_block = 25; continue;
}
// C line 47558
27 => {
vm_block = if (JS_IsUndefined(pattern)) != 0 { 21 } else { 26 }; continue;
}
// C line 47550
28 => {
vm_block = 4; continue;
}
// C line 47549
29 => {
vm_block = if (JS_IsException(flags)) != 0 { 28 } else { 27 }; continue;
}
// C line 47548
30 => {
let _ = { let assigned = JS_GetProperty(ctx, pat, (((crate::quickjs_atom::JS_ATOM_flags as i32)) as JSAtom)); flags = assigned; assigned };
vm_block = 29; continue;
}
// C line 47552
31 => {
let _ = { let assigned = JS_DupValue(ctx, flags1); flags = assigned; assigned };
vm_block = 27; continue;
}
// C line 47547
32 => {
vm_block = if (JS_IsUndefined(flags1)) != 0 { 30 } else { 31 }; continue;
}
// C line 47546
33 => {
vm_block = 4; continue;
}
// C line 47545
34 => {
vm_block = if (JS_IsException(pattern)) != 0 { 33 } else { 32 }; continue;
}
// C line 47544
35 => {
let _ = { let assigned = JS_GetProperty(ctx, pat, (((crate::quickjs_atom::JS_ATOM_source as i32)) as JSAtom)); pattern = assigned; assigned };
vm_block = 34; continue;
}
// C line 47556
36 => {
let _ = { let assigned = JS_DupValue(ctx, flags1); flags = assigned; assigned };
vm_block = 27; continue;
}
// C line 47555
37 => {
let _ = { let assigned = JS_DupValue(ctx, pat); pattern = assigned; assigned };
vm_block = 36; continue;
}
// C line 47543
38 => {
vm_block = if (pat_is_regexp) != 0 { 35 } else { 37 }; continue;
}
// C line 47531
39 => {
vm_block = if !(re).is_null() { 20 } else { 38 }; continue;
}
// C line 47530
40 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; flags = assigned; assigned };
vm_block = 39; continue;
}
// C line 47529
41 => {
let _ = { let assigned = js_get_regexp(ctx, pat, (0 as i32)); re = assigned; assigned };
vm_block = 40; continue;
}
// C line 47526
42 => {
return JS_DupValue(ctx, pat);
}
// C line 47525
43 => {
vm_block = if (res) != 0 { 42 } else { 41 }; continue;
}
// C line 47524
44 => {
let _ = JS_FreeValue(ctx, ctor);
vm_block = 43; continue;
}
// C line 47523
45 => {
let _ = { let assigned = js_same_value(ctx, ctor, new_target); res = assigned; assigned };
vm_block = 44; continue;
}
// C line 47522
46 => {
return ctor;
}
// C line 47521
47 => {
vm_block = if (JS_IsException(ctor)) != 0 { 46 } else { 45 }; continue;
}
// C line 47520
48 => {
let _ = { let assigned = JS_GetProperty(ctx, pat, (((crate::quickjs_atom::JS_ATOM_constructor as i32)) as JSAtom)); ctor = assigned; assigned };
vm_block = 47; continue;
}
// C line 47517
49 => {
vm_block = if (((((pat_is_regexp) != 0) && ((JS_IsUndefined(flags1)) != 0)) as i32)) != 0 { 48 } else { 41 }; continue;
}
// C line 47516
50 => {
let _ = { let assigned = JS_GetActiveFunction(ctx); new_target = assigned; assigned };
vm_block = 49; continue;
}
// C line 47514
51 => {
vm_block = if (JS_IsUndefined(new_target)) != 0 { 50 } else { 41 }; continue;
}
// C line 47513
52 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 47512
53 => {
vm_block = if ((((pat_is_regexp) < ((0 as i32))) as i32)) != 0 { 52 } else { 51 }; continue;
}
// C line 47511
54 => {
let _ = { let assigned = js_is_regexp(ctx, pat); pat_is_regexp = assigned; assigned };
vm_block = 53; continue;
}
// C line 47510
55 => {
let _ = { let assigned = *(argv).offset(((1 as i32)) as isize); flags1 = assigned; assigned };
vm_block = 54; continue;
}
// C line 47509
56 => {
let _ = { let assigned = *(argv).offset(((0 as i32)) as isize); pat = assigned; assigned };
vm_block = 55; continue;
}
// C line 47504
57 => {
obj = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) };
vm_block = 56; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:47584. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code, unused_variables)]
unsafe fn js_regexp_compile(mut ctx: *mut JSContext, mut this_val: JSValue, mut argc: i32, mut argv: *mut JSValue) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut re1: *mut JSRegExp = core::mem::zeroed();
let mut re: *mut JSRegExp = core::mem::zeroed();
let mut pattern1: JSValue = core::mem::zeroed();
let mut flags1: JSValue = core::mem::zeroed();
let mut bc: JSValue = core::mem::zeroed();
let mut pattern: JSValue = core::mem::zeroed();
let mut vm_block: usize = 30;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 47625
1 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 47624
2 => {
let _ = JS_FreeValue(ctx, bc);
vm_block = 1; continue;
}
// C line ? labels: fail
3 => {
let _ = JS_FreeValue(ctx, pattern);
vm_block = 2; continue;
}
// C line 47621
4 => {
return JS_DupValue(ctx, this_val);
}
// C line 47620
5 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 47618
6 => {
vm_block = if ((((JS_SetProperty(ctx, this_val, (((crate::quickjs_atom::JS_ATOM_lastIndex as i32)) as JSAtom), JS_NewInt32(ctx, (0 as i32)))) < ((0 as i32))) as i32)) != 0 { 5 } else { 4 }; continue;
}
// C line 47617
7 => {
let _ = { let assigned = ((((bc).u).ptr) as *mut JSString); (*(re)).bytecode = assigned; assigned };
vm_block = 6; continue;
}
// C line 47616
8 => {
let _ = { let assigned = ((((pattern).u).ptr) as *mut JSString); (*(re)).pattern = assigned; assigned };
vm_block = 7; continue;
}
// C line 47615
9 => {
let _ = JS_FreeValue(ctx, JSValue { u: JSValueUnion { ptr: (((*(re)).bytecode) as *mut c_void) }, tag: (((JS_TAG_STRING as i32)) as i64) });
vm_block = 8; continue;
}
// C line 47614
10 => {
let _ = JS_FreeValue(ctx, JSValue { u: JSValueUnion { ptr: (((*(re)).pattern) as *mut c_void) }, tag: (((JS_TAG_STRING as i32)) as i64) });
vm_block = 9; continue;
}
// C line 47601
11 => {
let _ = { let assigned = JS_DupValue(ctx, JSValue { u: JSValueUnion { ptr: (((*(re1)).bytecode) as *mut c_void) }, tag: (((JS_TAG_STRING as i32)) as i64) }); bc = assigned; assigned };
vm_block = 10; continue;
}
// C line 47600
12 => {
let _ = { let assigned = JS_DupValue(ctx, JSValue { u: JSValueUnion { ptr: (((*(re1)).pattern) as *mut c_void) }, tag: (((JS_TAG_STRING as i32)) as i64) }); pattern = assigned; assigned };
vm_block = 11; continue;
}
// C line 47599
13 => {
return JS_ThrowTypeError(ctx, c"flags must be undefined".as_ptr());
}
// C line 47598
14 => {
vm_block = if ((!((JS_IsUndefined(flags1)) != 0) as i32)) != 0 { 13 } else { 12 }; continue;
}
// C line 47612
15 => {
vm_block = 3; continue;
}
// C line 47611
16 => {
vm_block = if (JS_IsException(bc)) != 0 { 15 } else { 10 }; continue;
}
// C line 47610
17 => {
let _ = { let assigned = js_compile_regexp(ctx, pattern, flags1); bc = assigned; assigned };
vm_block = 16; continue;
}
// C line 47609
18 => {
vm_block = 3; continue;
}
// C line 47608
19 => {
vm_block = if (JS_IsException(pattern)) != 0 { 18 } else { 17 }; continue;
}
// C line 47605
20 => {
let _ = { let assigned = JS_AtomToString(ctx, (((crate::quickjs_atom::JS_ATOM_empty_string as i32)) as JSAtom)); pattern = assigned; assigned };
vm_block = 19; continue;
}
// C line 47607
21 => {
let _ = { let assigned = JS_ToString(ctx, pattern1); pattern = assigned; assigned };
vm_block = 19; continue;
}
// C line 47604
22 => {
vm_block = if (JS_IsUndefined(pattern1)) != 0 { 20 } else { 21 }; continue;
}
// C line 47603
23 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; bc = assigned; assigned };
vm_block = 22; continue;
}
// C line 47597
24 => {
vm_block = if !(re1).is_null() { 14 } else { 23 }; continue;
}
// C line 47596
25 => {
let _ = { let assigned = js_get_regexp(ctx, pattern1, (0 as i32)); re1 = assigned; assigned };
vm_block = 24; continue;
}
// C line 47595
26 => {
let _ = { let assigned = *(argv).offset(((1 as i32)) as isize); flags1 = assigned; assigned };
vm_block = 25; continue;
}
// C line 47594
27 => {
let _ = { let assigned = *(argv).offset(((0 as i32)) as isize); pattern1 = assigned; assigned };
vm_block = 26; continue;
}
// C line 47593
28 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 47592
29 => {
vm_block = if ((!(!(re).is_null()) as i32)) != 0 { 28 } else { 27 }; continue;
}
// C line 47591
30 => {
let _ = { let assigned = js_get_regexp(ctx, this_val, (1 as i32)); re = assigned; assigned };
vm_block = 29; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:47628. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code, unused_variables)]
unsafe fn js_regexp_get_source(mut ctx: *mut JSContext, mut this_val: JSValue) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut re: *mut JSRegExp = core::mem::zeroed();
let mut p: *mut JSString = core::mem::zeroed();
let mut b_s: StringBuffer = core::mem::zeroed();
let mut b: *mut StringBuffer = core::mem::zeroed();
let mut i: i32 = core::mem::zeroed();
let mut n: i32 = core::mem::zeroed();
let mut c: i32 = core::mem::zeroed();
let mut c2: i32 = core::mem::zeroed();
let mut bra: i32 = core::mem::zeroed();
let mut vm_block: usize = 41;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 47691
1 => {
return string_buffer_end(b);
}
// C line 47655
2 => {
vm_block = if ((((i) < (n)) as i32)) != 0 { 27 } else { 1 }; continue;
}
// C line 47689
3 => {
let _ = string_buffer_putc16(b, ((c2) as u32));
vm_block = 2; continue;
}
// C line 47688
4 => {
vm_block = if ((((c2) >= ((0 as i32))) as i32)) != 0 { 3 } else { 2 }; continue;
}
// C line 47687
5 => {
let _ = string_buffer_putc16(b, ((c) as u32));
vm_block = 4; continue;
}
// C line 47685
6 => {
vm_block = 5; continue;
}
// C line 47683
7 => {
let _ = { let assigned = (47 as i32); c2 = assigned; assigned };
vm_block = 6; continue;
}
// C line 47682
8 => {
let _ = { let assigned = (92 as i32); c = assigned; assigned };
vm_block = 7; continue;
}
// C line 47681
9 => {
vm_block = if ((!((bra) != 0) as i32)) != 0 { 8 } else { 6 }; continue;
}
// C line 47679
10 => {
vm_block = 5; continue;
}
// C line 47678
11 => {
let _ = { let assigned = (114 as i32); c2 = assigned; assigned };
vm_block = 10; continue;
}
// C line 47677
12 => {
let _ = { let assigned = (92 as i32); c = assigned; assigned };
vm_block = 11; continue;
}
// C line 47675
13 => {
vm_block = 5; continue;
}
// C line 47674
14 => {
let _ = { let assigned = (110 as i32); c2 = assigned; assigned };
vm_block = 13; continue;
}
// C line 47673
15 => {
let _ = { let assigned = (92 as i32); c = assigned; assigned };
vm_block = 14; continue;
}
// C line 47671
16 => {
vm_block = 5; continue;
}
// C line 47669
17 => {
let _ = { let assigned = (1 as i32); bra = assigned; assigned };
vm_block = 16; continue;
}
// C line 47668
18 => {
let _ = { let assigned = string_get(p, { let old = i; i = (i).wrapping_add(1); old }); c2 = assigned; assigned };
vm_block = 17; continue;
}
// C line 47667
19 => {
vm_block = if ((((((((i) < (n)) as i32)) != 0) && (((((string_get(p, i)) == ((93 as i32))) as i32)) != 0)) as i32)) != 0 { 18 } else { 17 }; continue;
}
// C line 47666
20 => {
vm_block = if ((!((bra) != 0) as i32)) != 0 { 19 } else { 16 }; continue;
}
// C line 47664
21 => {
vm_block = 5; continue;
}
// C line 47663
22 => {
let _ = { let assigned = (0 as i32); bra = assigned; assigned };
vm_block = 21; continue;
}
// C line 47661
23 => {
vm_block = 5; continue;
}
// C line 47660
24 => {
let _ = { let assigned = string_get(p, { let old = i; i = (i).wrapping_add(1); old }); c2 = assigned; assigned };
vm_block = 23; continue;
}
// C line 47659
25 => {
vm_block = if ((((i) < (n)) as i32)) != 0 { 24 } else { 23 }; continue;
}
// C line 47657
26 => {
vm_block = match { let assigned = string_get(p, { let old = i; i = (i).wrapping_add(1); old }); c = assigned; assigned } { x if x == (47 as i32) => 9, x if x == (13 as i32) => 12, x if x == (10 as i32) => 15, x if x == (91 as i32) => 20, x if x == (93 as i32) => 22, x if x == (92 as i32) => 25, _ => 5, }; continue;
}
// C line 47656
27 => {
let _ = { let assigned = ((1 as i32)).wrapping_neg(); c2 = assigned; assigned };
vm_block = 26; continue;
}
// C line 47655
28 => {
let _ = { let _ = { let assigned = (0 as i32); i = assigned; assigned }; { let assigned = (((*(p)).len()) as i32); n = assigned; assigned } };
vm_block = 2; continue;
}
// C line 47654
29 => {
let _ = { let assigned = (0 as i32); bra = assigned; assigned };
vm_block = 28; continue;
}
// C line 47651
30 => {
let _ = string_buffer_init2(ctx, b, (((*(p)).len()) as i32), (((*(p)).is_wide_char()) as i32));
vm_block = 29; continue;
}
// C line ? labels: empty_regex
31 => {
return js_new_string8(ctx, c"(?:)".as_ptr());
}
// C line 47647
32 => {
vm_block = if (((((((*(p)).len()) as i32)) == ((0 as i32))) as i32)) != 0 { 31 } else { 30 }; continue;
}
// C line 47645
33 => {
let _ = { let assigned = (*(re)).pattern; p = assigned; assigned };
vm_block = 32; continue;
}
// C line 47643
34 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 47642
35 => {
vm_block = if ((!(!(re).is_null()) as i32)) != 0 { 34 } else { 33 }; continue;
}
// C line 47641
36 => {
let _ = { let assigned = js_get_regexp(ctx, this_val, (1 as i32)); re = assigned; assigned };
vm_block = 35; continue;
}
// C line 47639
37 => {
vm_block = 31; continue;
}
// C line 47638
38 => {
vm_block = if (js_same_value(ctx, this_val, *((*(ctx)).class_proto).offset(((JS_CLASS_REGEXP as i32)) as isize))) != 0 { 37 } else { 36 }; continue;
}
// C line 47636
39 => {
return JS_ThrowTypeErrorNotAnObject(ctx);
}
// C line 47635
40 => {
vm_block = if (((((((this_val).tag) as i32)) != ((JS_TAG_OBJECT as i32))) as i32)) != 0 { 39 } else { 38 }; continue;
}
// C line 47632
41 => {
b = core::ptr::addr_of_mut!(b_s);
vm_block = 40; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:47694. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code, unused_variables)]
unsafe fn js_regexp_get_flag(mut ctx: *mut JSContext, mut this_val: JSValue, mut mask: i32) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut re: *mut JSRegExp = core::mem::zeroed();
let mut flags: i32 = core::mem::zeroed();
let mut vm_block: usize = 9;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 47711
1 => {
return JS_NewBool(ctx, ((flags) & (mask)));
}
// C line 47710
2 => {
let _ = { let assigned = lre_get_flags((((*((*(re)).bytecode)).u).str8).as_mut_ptr()); flags = assigned; assigned };
vm_block = 1; continue;
}
// C line 47705
3 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) };
}
// C line 47707
4 => {
return JS_ThrowTypeErrorInvalidClass(ctx, (JS_CLASS_REGEXP as i32));
}
// C line 47704
5 => {
vm_block = if (js_same_value(ctx, this_val, *((*(ctx)).class_proto).offset(((JS_CLASS_REGEXP as i32)) as isize))) != 0 { 3 } else { 4 }; continue;
}
// C line 47703
6 => {
vm_block = if ((!(!(re).is_null()) as i32)) != 0 { 5 } else { 2 }; continue;
}
// C line 47702
7 => {
let _ = { let assigned = js_get_regexp(ctx, this_val, (0 as i32)); re = assigned; assigned };
vm_block = 6; continue;
}
// C line 47700
8 => {
return JS_ThrowTypeErrorNotAnObject(ctx);
}
// C line 47699
9 => {
vm_block = if (((((((this_val).tag) as i32)) != ((JS_TAG_OBJECT as i32))) as i32)) != 0 { 8 } else { 7 }; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:47716. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code, unused_variables)]
unsafe fn js_regexp_get_flags(mut ctx: *mut JSContext, mut this_val: JSValue) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut str: [c_char; 8] = core::mem::zeroed();
let mut p: *mut c_char = core::mem::zeroed();
let mut res: i32 = core::mem::zeroed();
let mut i: i32 = core::mem::zeroed();
let mut flag_atom: [i32; 8] = core::mem::zeroed();
let mut flag_char: [c_char; 8] = core::mem::zeroed();
let mut vm_block: usize = 15;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line ? labels: exception
1 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 47742
2 => {
return JS_NewStringLen(ctx, (str).as_mut_ptr(), ((((p).offset_from((str).as_mut_ptr()) as i64)) as usize));
}
// C line 47735
3 => {
vm_block = if ((((i) < ((8 as i32))) as i32)) != 0 { 9 } else { 2 }; continue;
}
// C line ?
4 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 3; continue;
}
// C line 47740
5 => {
let _ = { let assigned = *((flag_char).as_ptr()).offset((i) as isize); *({ let old = p; p = (p).offset(1); old }) = assigned; assigned };
vm_block = 4; continue;
}
// C line 47739
6 => {
vm_block = if (res) != 0 { 5 } else { 4 }; continue;
}
// C line 47738
7 => {
vm_block = 1; continue;
}
// C line 47737
8 => {
vm_block = if ((((res) < ((0 as i32))) as i32)) != 0 { 7 } else { 6 }; continue;
}
// C line 47736
9 => {
let _ = { let assigned = JS_ToBoolFree(ctx, JS_GetProperty(ctx, this_val, ((*((flag_atom).as_ptr()).offset((i) as isize)) as JSAtom))); res = assigned; assigned };
vm_block = 8; continue;
}
// C line 47735
10 => {
let _ = { let assigned = (0 as i32); i = assigned; assigned };
vm_block = 3; continue;
}
// C line 47733
11 => {
return JS_ThrowTypeErrorNotAnObject(ctx);
}
// C line 47732
12 => {
vm_block = if (((((((this_val).tag) as i32)) != ((JS_TAG_OBJECT as i32))) as i32)) != 0 { 11 } else { 10 }; continue;
}
// C line 47730
13 => {
flag_char = [(((100 as i32)) as c_char), (((103 as i32)) as c_char), (((105 as i32)) as c_char), (((109 as i32)) as c_char), (((115 as i32)) as c_char), (((117 as i32)) as c_char), (((118 as i32)) as c_char), (((121 as i32)) as c_char)];
vm_block = 12; continue;
}
// C line 47720
14 => {
flag_atom = [(crate::quickjs_atom::JS_ATOM_hasIndices as i32), (crate::quickjs_atom::JS_ATOM_global as i32), (crate::quickjs_atom::JS_ATOM_ignoreCase as i32), (crate::quickjs_atom::JS_ATOM_multiline as i32), (crate::quickjs_atom::JS_ATOM_dotAll as i32), (crate::quickjs_atom::JS_ATOM_unicode as i32), (crate::quickjs_atom::JS_ATOM_unicodeSets as i32), (crate::quickjs_atom::JS_ATOM_sticky as i32)];
vm_block = 13; continue;
}
// C line 47718
15 => {
p = (str).as_mut_ptr();
vm_block = 14; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:47748. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code, unused_variables)]
unsafe fn js_regexp_toString(mut ctx: *mut JSContext, mut this_val: JSValue, mut argc: i32, mut argv: *mut JSValue) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut pattern: JSValue = core::mem::zeroed();
let mut flags: JSValue = core::mem::zeroed();
let mut b_s: StringBuffer = core::mem::zeroed();
let mut b: *mut StringBuffer = core::mem::zeroed();
let mut vm_block: usize = 15;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 47770
1 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line ? labels: fail
2 => {
let _ = string_buffer_free(b);
vm_block = 1; continue;
}
// C line 47766
3 => {
return string_buffer_end(b);
}
// C line 47765
4 => {
vm_block = 2; continue;
}
// C line 47764
5 => {
vm_block = if (string_buffer_concat_value_free(b, flags)) != 0 { 4 } else { 3 }; continue;
}
// C line 47763
6 => {
let _ = { let assigned = JS_GetProperty(ctx, this_val, (((crate::quickjs_atom::JS_ATOM_flags as i32)) as JSAtom)); flags = assigned; assigned };
vm_block = 5; continue;
}
// C line 47762
7 => {
let _ = string_buffer_putc8(b, (((47 as i32)) as u32));
vm_block = 6; continue;
}
// C line 47761
8 => {
vm_block = 2; continue;
}
// C line 47760
9 => {
vm_block = if (string_buffer_concat_value_free(b, pattern)) != 0 { 8 } else { 7 }; continue;
}
// C line 47759
10 => {
let _ = { let assigned = JS_GetProperty(ctx, this_val, (((crate::quickjs_atom::JS_ATOM_source as i32)) as JSAtom)); pattern = assigned; assigned };
vm_block = 9; continue;
}
// C line 47758
11 => {
let _ = string_buffer_putc8(b, (((47 as i32)) as u32));
vm_block = 10; continue;
}
// C line 47757
12 => {
let _ = string_buffer_init(ctx, b, (0 as i32));
vm_block = 11; continue;
}
// C line 47755
13 => {
return JS_ThrowTypeErrorNotAnObject(ctx);
}
// C line 47754
14 => {
vm_block = if ((!((JS_IsObject(this_val)) != 0) as i32)) != 0 { 13 } else { 12 }; continue;
}
// C line 47752
15 => {
b = core::ptr::addr_of_mut!(b_s);
vm_block = 14; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:47773. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code, unused_variables)]
unsafe fn lre_check_stack_overflow(mut opaque: *mut c_void, mut alloca_size: usize) -> i32 {
let mut vm_local_storage = Vec::<u64>::new();
let mut ctx: *mut JSContext = core::mem::zeroed();
let mut vm_block: usize = 2;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 47776
1 => {
return js_check_stack_overflow((*(ctx)).rt, alloca_size);
}
// C line 47775
2 => {
ctx = ((opaque) as *mut JSContext);
vm_block = 1; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:47779. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code, unused_variables)]
unsafe fn lre_check_timeout(mut opaque: *mut c_void) -> i32 {
let mut vm_local_storage = Vec::<u64>::new();
let mut ctx: *mut JSContext = core::mem::zeroed();
let mut rt: *mut JSRuntime = core::mem::zeroed();
let mut vm_block: usize = 3;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 47783
1 => {
return (((((*(rt)).interrupt_handler).is_some()) && ((((*(rt)).interrupt_handler).expect("registered interrupt callback")(rt, (*(rt)).interrupt_opaque)) != 0)) as i32);
}
// C line 47782
2 => {
rt = (*(ctx)).rt;
vm_block = 1; continue;
}
// C line 47781
3 => {
ctx = ((opaque) as *mut JSContext);
vm_block = 2; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:47787. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code, unused_variables)]
unsafe fn lre_realloc(mut opaque: *mut c_void, mut ptr: *mut c_void, mut size: usize) -> *mut c_void {
let mut vm_local_storage = Vec::<u64>::new();
let mut ctx: *mut JSContext = core::mem::zeroed();
let mut vm_block: usize = 2;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 47791
1 => {
return js_realloc_rt((*(ctx)).rt, ptr, size);
}
// C line 47789
2 => {
ctx = ((opaque) as *mut JSContext);
vm_block = 1; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:47794. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code, unused_variables)]
unsafe fn js_regexp_escape(mut ctx: *mut JSContext, mut this_val: JSValue, mut argc: i32, mut argv: *mut JSValue) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut str: JSValue = core::mem::zeroed();
let mut b_s: StringBuffer = core::mem::zeroed();
let mut b: *mut StringBuffer = core::mem::zeroed();
let mut p: *mut JSString = core::mem::zeroed();
let mut c: u32 = core::mem::zeroed();
let mut s: [c_char; 16] = core::mem::zeroed();
let mut i: i32 = core::mem::zeroed();
let mut i0: i32 = core::mem::zeroed();
let mut vm_block: usize = 35;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 47845
1 => {
return string_buffer_end(b);
}
// C line 47844
2 => {
let _ = JS_FreeValue(ctx, str);
vm_block = 1; continue;
}
// C line 47811
3 => {
vm_block = if ((((i) < ((((*(p)).len()) as i32))) as i32)) != 0 { 26 } else { 2 }; continue;
}
// C line 47817
4 => {
let _ = string_buffer_putc8(b, ((*(c"tnvfr".as_ptr()).offset(((c).wrapping_sub((((9 as i32)) as u32))) as isize)) as u32));
vm_block = 3; continue;
}
// C line 47816
5 => {
let _ = string_buffer_putc8(b, (((92 as i32)) as u32));
vm_block = 4; continue;
}
// C line 47819
6 => {
vm_block = 17; continue;
}
// C line 47815
7 => {
vm_block = if ((((((((c) >= ((((9 as i32)) as u32))) as i32)) != 0) && (((((c) <= ((((13 as i32)) as u32))) as i32)) != 0)) as i32)) != 0 { 5 } else { 6 }; continue;
}
// C line 47832
8 => {
let _ = string_buffer_putc8(b, c);
vm_block = 3; continue;
}
// C line 47826
9 => {
vm_block = 17; continue;
}
// C line 47825
10 => {
vm_block = if ((((i0) == ((0 as i32))) as i32)) != 0 { 9 } else { 8 }; continue;
}
// C line 47828
11 => {
vm_block = 17; continue;
}
// C line 47830
12 => {
let _ = string_buffer_putc8(b, (((92 as i32)) as u32));
vm_block = 8; continue;
}
// C line 47829
13 => {
vm_block = if ((((c) != ((((95 as i32)) as u32))) as i32)) != 0 { 12 } else { 8 }; continue;
}
// C line 47827
14 => {
vm_block = if !(regexp_strchr(c",-=<>#&!%:;@~'`\"".as_ptr(), ((c) as i32))).is_null() { 11 } else { 13 }; continue;
}
// C line 47822
15 => {
vm_block = if ((((((((((((((((c) >= ((((48 as i32)) as u32))) as i32)) != 0) && (((((c) <= ((((57 as i32)) as u32))) as i32)) != 0)) as i32)) != 0) || (((((((((c) >= ((((65 as i32)) as u32))) as i32)) != 0) && (((((c) <= ((((90 as i32)) as u32))) as i32)) != 0)) as i32)) != 0)) as i32)) != 0) || (((((((((c) >= ((((97 as i32)) as u32))) as i32)) != 0) && (((((c) <= ((((122 as i32)) as u32))) as i32)) != 0)) as i32)) != 0)) as i32)) != 0 { 10 } else { 14 }; continue;
}
// C line 47836
16 => {
let _ = string_buffer_puts8(b, (s).as_mut_ptr());
vm_block = 3; continue;
}
// C line 47835 labels: hex2
17 => {
let _ = regexp_escape_hex((s).as_mut_ptr(), (size_of::<[c_char; 16]>() as usize), c, 2);
vm_block = 16; continue;
}
// C line 47839
18 => {
let _ = string_buffer_puts8(b, (s).as_mut_ptr());
vm_block = 3; continue;
}
// C line 47838
19 => {
let _ = regexp_escape_hex((s).as_mut_ptr(), (size_of::<[c_char; 16]>() as usize), c, 4);
vm_block = 18; continue;
}
// C line 47841
20 => {
let _ = string_buffer_putc(b, c);
vm_block = 3; continue;
}
// C line 47837
21 => {
vm_block = if (((((crate::cutils_header::is_surrogate(c)) != 0) || ((crate::libunicode_header::lre_is_space(c)) != 0)) as i32)) != 0 { 19 } else { 20 }; continue;
}
// C line 47833
22 => {
vm_block = if ((((c) < ((((256 as i32)) as u32))) as i32)) != 0 { 17 } else { 21 }; continue;
}
// C line 47821
23 => {
vm_block = if ((((c) < ((((128 as i32)) as u32))) as i32)) != 0 { 15 } else { 22 }; continue;
}
// C line 47814
24 => {
vm_block = if ((((c) < ((((33 as i32)) as u32))) as i32)) != 0 { 7 } else { 23 }; continue;
}
// C line 47813
25 => {
let _ = { let assigned = ((string_getc(p, core::ptr::addr_of_mut!(i))) as u32); c = assigned; assigned };
vm_block = 24; continue;
}
// C line 47812
26 => {
let _ = { let assigned = i; i0 = assigned; assigned };
vm_block = 25; continue;
}
// C line 47811
27 => {
let _ = { let assigned = (0 as i32); i = assigned; assigned };
vm_block = 3; continue;
}
// C line 47810
28 => {
let _ = string_buffer_init2(ctx, b, (0 as i32), (((*(p)).is_wide_char()) as i32));
vm_block = 27; continue;
}
// C line 47809
29 => {
let _ = { let assigned = ((((str).u).ptr) as *mut JSString); p = assigned; assigned };
vm_block = 28; continue;
}
// C line 47808
30 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 47807
31 => {
vm_block = if (JS_IsException(str)) != 0 { 30 } else { 29 }; continue;
}
// C line 47806
32 => {
let _ = { let assigned = JS_ToString(ctx, *(argv).offset(((0 as i32)) as isize)); str = assigned; assigned };
vm_block = 31; continue;
}
// C line 47805
33 => {
return JS_ThrowTypeError(ctx, c"not a string".as_ptr());
}
// C line 47804
34 => {
vm_block = if ((!((JS_IsString(*(argv).offset(((0 as i32)) as isize))) != 0) as i32)) != 0 { 33 } else { 32 }; continue;
}
// C line 47798
35 => {
b = core::ptr::addr_of_mut!(b_s);
vm_block = 34; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:47849. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code, unused_variables)]
unsafe fn js_regexp_get_lastIndex(mut ctx: *mut JSContext, mut plast_index: *mut i64, mut this_val: JSValue) -> i32 {
let mut vm_local_storage = Vec::<u64>::new();
let mut p: *mut JSObject = core::mem::zeroed();
let mut vm_block: usize = 5;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 47857
1 => {
return (0 as i32);
}
// C line 47856
2 => {
let _ = { let assigned = ((max_int(((((((*((*(p)).prop).offset(((0 as i32)) as isize)).u).value).u).uint64) as i32), (0 as i32))) as i64); *(plast_index) = assigned; assigned };
vm_block = 1; continue;
}
// C line 47859
3 => {
return JS_ToLengthFree(ctx, plast_index, JS_DupValue(ctx, ((*((*(p)).prop).offset(((0 as i32)) as isize)).u).value));
}
// C line 47855
4 => {
vm_block = if ((((!(((!((((((((((*((*(p)).prop).offset(((0 as i32)) as isize)).u).value).tag) as i32)) == ((JS_TAG_INT as i32))) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 2 } else { 3 }; continue;
}
// C line 47852
5 => {
p = ((((this_val).u).ptr) as *mut JSObject);
vm_block = 4; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:47864. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code, unused_variables)]
unsafe fn js_regexp_set_lastIndex(mut ctx: *mut JSContext, mut this_val: JSValue, mut last_index: i32) -> i32 {
let mut vm_local_storage = Vec::<u64>::new();
let mut p: *mut JSObject = core::mem::zeroed();
let mut vm_block: usize = 6;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 47878
1 => {
return (0 as i32);
}
// C line 47872
2 => {
let _ = set_value(ctx, core::ptr::addr_of_mut!(((*((*(p)).prop).offset(((0 as i32)) as isize)).u).value), JS_NewInt32(ctx, last_index));
vm_block = 1; continue;
}
// C line 47876
3 => {
return ((1 as i32)).wrapping_neg();
}
// C line 47874
4 => {
vm_block = if ((((JS_SetProperty(ctx, this_val, (((crate::quickjs_atom::JS_ATOM_lastIndex as i32)) as JSAtom), JS_NewInt32(ctx, last_index))) < ((0 as i32))) as i32)) != 0 { 3 } else { 1 }; continue;
}
// C line 47870
5 => {
vm_block = if ((((!(((!((((((((((((((*((*(p)).prop).offset(((0 as i32)) as isize)).u).value).tag) as i32)) == ((JS_TAG_INT as i32))) as i32)) != 0) && (((((((*(get_shape_prop((*(p)).shape))).flags()) as i32)) & (((1 as i32)).wrapping_shl(((1 as i32)) as u32)))) != 0)) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 2 } else { 4 }; continue;
}
// C line 47867
6 => {
p = ((((this_val).u).ptr) as *mut JSObject);
vm_block = 5; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:47881. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code, unused_variables)]
unsafe fn js_regexp_exec(mut ctx: *mut JSContext, mut this_val: JSValue, mut argc: i32, mut argv: *mut JSValue) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut re: *mut JSRegExp = core::mem::zeroed();
let mut str: *mut JSString = core::mem::zeroed();
let mut t: JSValue = core::mem::zeroed();
let mut ret: JSValue = core::mem::zeroed();
let mut str_val: JSValue = core::mem::zeroed();
let mut obj: JSValue = core::mem::zeroed();
let mut groups: JSValue = core::mem::zeroed();
let mut indices: JSValue = core::mem::zeroed();
let mut indices_groups: JSValue = core::mem::zeroed();
let mut re_bytecode: *mut u8 = core::mem::zeroed();
let mut capture: *mut *mut u8 = core::mem::zeroed();
let mut str_buf: *mut u8 = core::mem::zeroed();
let mut rc: i32 = core::mem::zeroed();
let mut capture_count: i32 = core::mem::zeroed();
let mut shift: i32 = core::mem::zeroed();
let mut i: i32 = core::mem::zeroed();
let mut re_flags: i32 = core::mem::zeroed();
let mut alloc_count: i32 = core::mem::zeroed();
let mut last_index: i64 = core::mem::zeroed();
let mut group_name_ptr: *const c_char = core::mem::zeroed();
let mut p_obj: *mut JSObject = core::mem::zeroed();
let mut group_name: JSAtom = core::mem::zeroed();
let mut prop_flags: i32 = core::mem::zeroed();
let mut props: [JSProperty; 4] = core::mem::zeroed();
let mut v_match: *mut *mut u8 = core::mem::zeroed();
let mut start: i32 = core::mem::zeroed();
let mut end: i32 = core::mem::zeroed();
let mut val: JSValue = core::mem::zeroed();
let mut vm_block: usize = 132;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 48097
1 => {
return ret;
}
// C line 48096
2 => {
let _ = js_free(ctx, ((capture) as *mut c_void));
vm_block = 1; continue;
}
// C line 48095
3 => {
let _ = JS_FreeValue(ctx, obj);
vm_block = 2; continue;
}
// C line 48094
4 => {
let _ = JS_FreeValue(ctx, groups);
vm_block = 3; continue;
}
// C line 48093
5 => {
let _ = JS_FreeValue(ctx, str_val);
vm_block = 4; continue;
}
// C line 48092
6 => {
let _ = JS_FreeValue(ctx, indices);
vm_block = 5; continue;
}
// C line 48091
7 => {
let _ = JS_FreeValue(ctx, indices_groups);
vm_block = 6; continue;
}
// C line ? labels: fail
8 => {
let _ = JS_FreeAtom(ctx, group_name);
vm_block = 7; continue;
}
// C line 48088
9 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; obj = assigned; assigned };
vm_block = 8; continue;
}
// C line 48087
10 => {
let _ = { let assigned = obj; ret = assigned; assigned };
vm_block = 9; continue;
}
// C line 47940
11 => {
vm_block = 8; continue;
}
// C line 47939
12 => {
vm_block = if ((((js_regexp_set_lastIndex(ctx, this_val, (0 as i32))) < ((0 as i32))) as i32)) != 0 { 11 } else { 10 }; continue;
}
// C line 47938
13 => {
vm_block = if ((((((((rc) == ((2 as i32))) as i32)) != 0) || ((((re_flags) & (((((1 as i32)).wrapping_shl(((0 as i32)) as u32)) | (((1 as i32)).wrapping_shl(((5 as i32)) as u32)))))) != 0)) as i32)) != 0 { 12 } else { 10 }; continue;
}
// C line 47948
14 => {
vm_block = 8; continue;
}
// C line 47944
15 => {
let _ = JS_ThrowInterrupted(ctx);
vm_block = 14; continue;
}
// C line 47946
16 => {
let _ = JS_ThrowInternalError(ctx, c"out of memory in regexp execution".as_ptr());
vm_block = 14; continue;
}
// C line 47943
17 => {
vm_block = if ((((rc) == (((2 as i32)).wrapping_neg())) as i32)) != 0 { 15 } else { 16 }; continue;
}
// C line 47937
18 => {
vm_block = if ((((rc) >= ((0 as i32))) as i32)) != 0 { 13 } else { 17 }; continue;
}
// C line 48083
19 => {
vm_block = 8; continue;
}
// C line 48081
20 => {
vm_block = if ((((JS_DefinePropertyValue(ctx, obj, (((crate::quickjs_atom::JS_ATOM_indices as i32)) as JSAtom), t, prop_flags)) < ((0 as i32))) as i32)) != 0 { 19 } else { 10 }; continue;
}
// C line 48080
21 => {
let _ = { let _ = { let assigned = indices; t = assigned; assigned }; { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; indices = assigned; assigned } };
vm_block = 20; continue;
}
// C line 48078
22 => {
vm_block = 8; continue;
}
// C line 48076
23 => {
vm_block = if ((((JS_DefinePropertyValue(ctx, indices, (((crate::quickjs_atom::JS_ATOM_groups as i32)) as JSAtom), t, prop_flags)) < ((0 as i32))) as i32)) != 0 { 22 } else { 21 }; continue;
}
// C line 48075
24 => {
let _ = { let _ = { let assigned = indices_groups; t = assigned; assigned }; { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; indices_groups = assigned; assigned } };
vm_block = 23; continue;
}
// C line 48074
25 => {
vm_block = if ((!((JS_IsUndefined(indices)) != 0) as i32)) != 0 { 24 } else { 10 }; continue;
}
// C line 47992
26 => {
vm_block = if ((((i) < (capture_count)) as i32)) != 0 { 71 } else { 25 }; continue;
}
// C line ?
27 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 26; continue;
}
// C line 48071
28 => {
let _ = { let assigned = val; *(((((*(p_obj)).u).array).u).values).offset(({ let old = (((*(p_obj)).u).array).count; (((*(p_obj)).u).array).count = ((((*(p_obj)).u).array).count).wrapping_add(1); old }) as isize) = assigned; assigned };
vm_block = 27; continue;
}
// C line 48069
29 => {
let _ = { let assigned = (((0 as i32)) as JSAtom); group_name = assigned; assigned };
vm_block = 28; continue;
}
// C line 48068
30 => {
let _ = JS_FreeAtom(ctx, group_name);
vm_block = 29; continue;
}
// C line 48065
31 => {
vm_block = 8; continue;
}
// C line 48064
32 => {
let _ = JS_FreeValue(ctx, val);
vm_block = 31; continue;
}
// C line 48061
33 => {
vm_block = if ((((JS_DefinePropertyValue(ctx, groups, group_name, JS_DupValue(ctx, val), prop_flags)) < ((0 as i32))) as i32)) != 0 { 32 } else { 30 }; continue;
}
// C line 48059
34 => {
vm_block = if ((((((!((JS_IsUndefined(val)) != 0) as i32)) != 0) || (((!((JS_HasProperty(ctx, groups, group_name)) != 0) as i32)) != 0)) as i32)) != 0 { 33 } else { 30 }; continue;
}
// C line 48057
35 => {
vm_block = if ((((group_name) != ((((0 as i32)) as JSAtom))) as i32)) != 0 { 34 } else { 28 }; continue;
}
// C line 48054
36 => {
vm_block = 8; continue;
}
// C line 48053
37 => {
vm_block = if (JS_IsException(val)) != 0 { 36 } else { 35 }; continue;
}
// C line 48052
38 => {
let _ = { let assigned = js_sub_string(ctx, str, start, end); val = assigned; assigned };
vm_block = 37; continue;
}
// C line 48051
39 => {
vm_block = if ((((start) != (((1 as i32)).wrapping_neg())) as i32)) != 0 { 38 } else { 35 }; continue;
}
// C line 48050
40 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; val = assigned; assigned };
vm_block = 39; continue;
}
// C line 48046
41 => {
vm_block = 8; continue;
}
// C line 48044
42 => {
vm_block = if ((((JS_DefinePropertyValueUint32(ctx, indices, ((i) as u32), val, prop_flags)) < ((0 as i32))) as i32)) != 0 { 41 } else { 40 }; continue;
}
// C line 48040
43 => {
vm_block = 8; continue;
}
// C line 48039
44 => {
let _ = JS_FreeValue(ctx, val);
vm_block = 43; continue;
}
// C line 48037
45 => {
vm_block = if ((((JS_DefinePropertyValue(ctx, indices_groups, group_name, JS_DupValue(ctx, val), prop_flags)) < ((0 as i32))) as i32)) != 0 { 44 } else { 42 }; continue;
}
// C line 48035
46 => {
vm_block = if ((((((!((JS_IsUndefined(val)) != 0) as i32)) != 0) || (((!((JS_HasProperty(ctx, indices_groups, group_name)) != 0) as i32)) != 0)) as i32)) != 0 { 45 } else { 42 }; continue;
}
// C line 48033
47 => {
vm_block = if ((((group_name) != ((((0 as i32)) as JSAtom))) as i32)) != 0 { 46 } else { 42 }; continue;
}
// C line 48030
48 => {
vm_block = 8; continue;
}
// C line 48029
49 => {
let _ = JS_FreeValue(ctx, val);
vm_block = 48; continue;
}
// C line 48026
50 => {
vm_block = if ((((JS_DefinePropertyValueUint32(ctx, val, (((1 as i32)) as u32), JS_NewInt32(ctx, end), prop_flags)) < ((0 as i32))) as i32)) != 0 { 49 } else { 47 }; continue;
}
// C line 48024
51 => {
vm_block = 8; continue;
}
// C line 48023
52 => {
let _ = JS_FreeValue(ctx, val);
vm_block = 51; continue;
}
// C line 48020
53 => {
vm_block = if ((((JS_DefinePropertyValueUint32(ctx, val, (((0 as i32)) as u32), JS_NewInt32(ctx, start), prop_flags)) < ((0 as i32))) as i32)) != 0 { 52 } else { 50 }; continue;
}
// C line 48019
54 => {
vm_block = 8; continue;
}
// C line 48018
55 => {
vm_block = if (JS_IsException(val)) != 0 { 54 } else { 53 }; continue;
}
// C line 48017
56 => {
let _ = { let assigned = JS_NewArray(ctx); val = assigned; assigned };
vm_block = 55; continue;
}
// C line 48016
57 => {
vm_block = if ((((start) != (((1 as i32)).wrapping_neg())) as i32)) != 0 { 56 } else { 47 }; continue;
}
// C line 48015
58 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; val = assigned; assigned };
vm_block = 57; continue;
}
// C line 48014
59 => {
vm_block = if ((!((JS_IsUndefined(indices)) != 0) as i32)) != 0 { 58 } else { 40 }; continue;
}
// C line 48011
60 => {
let _ = { let assigned = (((((*(v_match).offset(((1 as i32)) as isize)).offset_from(str_buf) as i64)).wrapping_shr((shift) as u32)) as i32); end = assigned; assigned };
vm_block = 59; continue;
}
// C line 48010
61 => {
let _ = { let assigned = (((((*(v_match).offset(((0 as i32)) as isize)).offset_from(str_buf) as i64)).wrapping_shr((shift) as u32)) as i32); start = assigned; assigned };
vm_block = 60; continue;
}
// C line 48009
62 => {
vm_block = if ((((!(*(v_match).offset(((0 as i32)) as isize)).is_null()) && (!(*(v_match).offset(((1 as i32)) as isize)).is_null())) as i32)) != 0 { 61 } else { 59 }; continue;
}
// C line 48006
63 => {
let _ = { group_name_ptr = (group_name_ptr).offset((((regexp_strlen(group_name_ptr)).wrapping_add((((2 as i32)) as usize))) as isize)); group_name_ptr };
vm_block = 62; continue;
}
// C line 48004
64 => {
vm_block = 8; continue;
}
// C line 48003
65 => {
vm_block = if ((((group_name) == ((((0 as i32)) as JSAtom))) as i32)) != 0 { 64 } else { 63 }; continue;
}
// C line 48002
66 => {
let _ = { let assigned = JS_NewAtom(ctx, group_name_ptr); group_name = assigned; assigned };
vm_block = 65; continue;
}
// C line 47999
67 => {
vm_block = if (*(group_name_ptr)) != 0 { 66 } else { 63 }; continue;
}
// C line 47998
68 => {
vm_block = if ((((!(group_name_ptr).is_null()) && (((((i) > ((0 as i32))) as i32)) != 0)) as i32)) != 0 { 67 } else { 62 }; continue;
}
// C line 47995
69 => {
end = ((1 as i32)).wrapping_neg();
vm_block = 68; continue;
}
// C line 47994
70 => {
start = ((1 as i32)).wrapping_neg();
vm_block = 69; continue;
}
// C line 47993
71 => {
v_match = core::ptr::addr_of_mut!(*(capture).offset((((2 as i32)).wrapping_mul(i)) as isize));
vm_block = 70; continue;
}
// C line 47992
72 => {
let _ = { let assigned = (0 as i32); i = assigned; assigned };
vm_block = 26; continue;
}
// C line 47990
73 => {
vm_block = 8; continue;
}
// C line 47989
74 => {
vm_block = if (expand_fast_array(ctx, p_obj, ((capture_count) as u32))) != 0 { 73 } else { 72 }; continue;
}
// C line 47988
75 => {
let _ = { let assigned = ((((obj).u).ptr) as *mut JSObject); p_obj = assigned; assigned };
vm_block = 74; continue;
}
// C line 47986
76 => {
vm_block = 8; continue;
}
// C line 47985
77 => {
vm_block = if (JS_IsException(obj)) != 0 { 76 } else { 75 }; continue;
}
// C line 47983
78 => {
let _ = { let assigned = JS_NewObjectFromShape(ctx, js_dup_shape((*(ctx)).regexp_result_shape), (((JS_CLASS_ARRAY as i32)) as JSClassID), (props).as_mut_ptr()); obj = assigned; assigned };
vm_block = 77; continue;
}
// C line 47982
79 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; str_val = assigned; assigned };
vm_block = 78; continue;
}
// C line 47980
80 => {
let _ = { let assigned = JS_DupValue(ctx, groups); ((*((props).as_mut_ptr()).offset(((3 as i32)) as isize)).u).value = assigned; assigned };
vm_block = 79; continue;
}
// C line 47979
81 => {
let _ = { let assigned = str_val; ((*((props).as_mut_ptr()).offset(((2 as i32)) as isize)).u).value = assigned; assigned };
vm_block = 80; continue;
}
// C line 47978
82 => {
let _ = { let assigned = JS_NewInt32(ctx, (((((*(capture).offset(((0 as i32)) as isize)).offset_from(str_buf) as i64)).wrapping_shr((shift) as u32)) as i32)); ((*((props).as_mut_ptr()).offset(((1 as i32)) as isize)).u).value = assigned; assigned };
vm_block = 81; continue;
}
// C line 47977
83 => {
let _ = { let assigned = JS_NewInt32(ctx, capture_count); ((*((props).as_mut_ptr()).offset(((0 as i32)) as isize)).u).value = assigned; assigned };
vm_block = 82; continue;
}
// C line 47973
84 => {
vm_block = 8; continue;
}
// C line 47972
85 => {
vm_block = if (JS_IsException(indices_groups)) != 0 { 84 } else { 83 }; continue;
}
// C line 47971
86 => {
let _ = { let assigned = JS_NewObjectProto(ctx, JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_NULL as i32)) as i64) }); indices_groups = assigned; assigned };
vm_block = 85; continue;
}
// C line 47970
87 => {
vm_block = if !(group_name_ptr).is_null() { 86 } else { 83 }; continue;
}
// C line 47969
88 => {
vm_block = 8; continue;
}
// C line 47968
89 => {
vm_block = if (JS_IsException(indices)) != 0 { 88 } else { 87 }; continue;
}
// C line 47967
90 => {
let _ = { let assigned = JS_NewArray(ctx); indices = assigned; assigned };
vm_block = 89; continue;
}
// C line 47966
91 => {
vm_block = if (((re_flags) & (((1 as i32)).wrapping_shl(((6 as i32)) as u32)))) != 0 { 90 } else { 83 }; continue;
}
// C line 47964
92 => {
vm_block = 8; continue;
}
// C line 47963
93 => {
vm_block = if (JS_IsException(groups)) != 0 { 92 } else { 91 }; continue;
}
// C line 47962
94 => {
let _ = { let assigned = JS_NewObjectProto(ctx, JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_NULL as i32)) as i64) }); groups = assigned; assigned };
vm_block = 93; continue;
}
// C line 47961
95 => {
vm_block = if !(group_name_ptr).is_null() { 94 } else { 91 }; continue;
}
// C line 47960
96 => {
let _ = { let assigned = lre_get_groupnames(re_bytecode); group_name_ptr = assigned; assigned };
vm_block = 95; continue;
}
// C line 47959
97 => {
let _ = { let assigned = ((((((((1 as i32)).wrapping_shl(((0 as i32)) as u32)) | (((1 as i32)).wrapping_shl(((1 as i32)) as u32)))) | (((1 as i32)).wrapping_shl(((2 as i32)) as u32)))) | (((1 as i32)).wrapping_shl(((14 as i32)) as u32))); prop_flags = assigned; assigned };
vm_block = 96; continue;
}
// C line 47957
98 => {
vm_block = 8; continue;
}
// C line 47955
99 => {
vm_block = if ((((js_regexp_set_lastIndex(ctx, this_val, (((((*(capture).offset(((1 as i32)) as isize)).offset_from(str_buf) as i64)).wrapping_shr((shift) as u32)) as i32))) < ((0 as i32))) as i32)) != 0 { 98 } else { 97 }; continue;
}
// C line 47954
100 => {
vm_block = if (((re_flags) & (((((1 as i32)).wrapping_shl(((0 as i32)) as u32)) | (((1 as i32)).wrapping_shl(((5 as i32)) as u32)))))) != 0 { 99 } else { 97 }; continue;
}
// C line 47936
101 => {
vm_block = if ((((rc) != ((1 as i32))) as i32)) != 0 { 18 } else { 100 }; continue;
}
// C line 47930
102 => {
let _ = { let assigned = (2 as i32); rc = assigned; assigned };
vm_block = 101; continue;
}
// C line 47932
103 => {
let _ = { let assigned = lre_exec(capture, re_bytecode, str_buf, ((last_index) as i32), (((*(str)).len()) as i32), shift, ((ctx) as *mut c_void)); rc = assigned; assigned };
vm_block = 101; continue;
}
// C line 47929
104 => {
vm_block = if ((((last_index) > ((((*(str)).len()) as i64))) as i32)) != 0 { 102 } else { 103 }; continue;
}
// C line 47928
105 => {
let _ = { let assigned = (((*(str)).u).str8).as_mut_ptr(); str_buf = assigned; assigned };
vm_block = 104; continue;
}
// C line 47927
106 => {
let _ = { let assigned = (((*(str)).is_wide_char()) as i32); shift = assigned; assigned };
vm_block = 105; continue;
}
// C line 47926
107 => {
let _ = { let assigned = lre_get_capture_count(re_bytecode); capture_count = assigned; assigned };
vm_block = 106; continue;
}
// C line 47924
108 => {
vm_block = 8; continue;
}
// C line 47923
109 => {
vm_block = if ((!(!(capture).is_null()) as i32)) != 0 { 108 } else { 107 }; continue;
}
// C line 47922
110 => {
let _ = { let assigned = ((js_malloc(ctx, ((size_of::<*mut u8>() as usize)).wrapping_mul(((alloc_count) as usize)))) as *mut *mut u8); capture = assigned; assigned };
vm_block = 109; continue;
}
// C line 47921
111 => {
vm_block = if ((((alloc_count) > ((0 as i32))) as i32)) != 0 { 110 } else { 107 }; continue;
}
// C line 47920
112 => {
let _ = { let assigned = lre_get_alloc_count(re_bytecode); alloc_count = assigned; assigned };
vm_block = 111; continue;
}
// C line 47919
113 => {
let _ = { let assigned = ((((str_val).u).ptr) as *mut JSString); str = assigned; assigned };
vm_block = 112; continue;
}
// C line 47917
114 => {
let _ = { let assigned = (((0 as i32)) as i64); last_index = assigned; assigned };
vm_block = 113; continue;
}
// C line 47916
115 => {
vm_block = if ((((((re_flags) & (((((1 as i32)).wrapping_shl(((0 as i32)) as u32)) | (((1 as i32)).wrapping_shl(((5 as i32)) as u32)))))) == ((0 as i32))) as i32)) != 0 { 114 } else { 113 }; continue;
}
// C line 47915
116 => {
let _ = { let assigned = lre_get_flags(re_bytecode); re_flags = assigned; assigned };
vm_block = 115; continue;
}
// C line 47914
117 => {
let _ = { let assigned = (((*((*(re)).bytecode)).u).str8).as_mut_ptr(); re_bytecode = assigned; assigned };
vm_block = 116; continue;
}
// C line 47912
118 => {
vm_block = 8; continue;
}
// C line 47911
119 => {
vm_block = if (js_regexp_get_lastIndex(ctx, core::ptr::addr_of_mut!(last_index), this_val)) != 0 { 118 } else { 117 }; continue;
}
// C line 47909
120 => {
let _ = { let assigned = (((0 as i32)) as JSAtom); group_name = assigned; assigned };
vm_block = 119; continue;
}
// C line 47908
121 => {
let _ = { let assigned = core::ptr::null_mut::<*mut u8>(); capture = assigned; assigned };
vm_block = 120; continue;
}
// C line 47907
122 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; indices_groups = assigned; assigned };
vm_block = 121; continue;
}
// C line 47906
123 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; indices = assigned; assigned };
vm_block = 122; continue;
}
// C line 47905
124 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; groups = assigned; assigned };
vm_block = 123; continue;
}
// C line 47904
125 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_NULL as i32)) as i64) }; obj = assigned; assigned };
vm_block = 124; continue;
}
// C line 47903
126 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) }; ret = assigned; assigned };
vm_block = 125; continue;
}
// C line 47901
127 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 47900
128 => {
vm_block = if (JS_IsException(str_val)) != 0 { 127 } else { 126 }; continue;
}
// C line 47899
129 => {
let _ = { let assigned = JS_ToString(ctx, *(argv).offset(((0 as i32)) as isize)); str_val = assigned; assigned };
vm_block = 128; continue;
}
// C line 47897
130 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 47896
131 => {
vm_block = if ((!(!(re).is_null()) as i32)) != 0 { 130 } else { 129 }; continue;
}
// C line 47884
132 => {
re = js_get_regexp(ctx, this_val, (1 as i32));
vm_block = 131; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:48101. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code, unused_variables)]
unsafe fn js_regexp_replace(mut ctx: *mut JSContext, mut this_val: JSValue, mut arg: JSValue, mut rep_val: JSValue) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut re: *mut JSRegExp = core::mem::zeroed();
let mut str: *mut JSString = core::mem::zeroed();
let mut str_val: JSValue = core::mem::zeroed();
let mut re_bytecode: *mut u8 = core::mem::zeroed();
let mut ret: i32 = core::mem::zeroed();
let mut capture: *mut *mut u8 = core::mem::zeroed();
let mut str_buf: *mut u8 = core::mem::zeroed();
let mut capture_count: i32 = core::mem::zeroed();
let mut alloc_count: i32 = core::mem::zeroed();
let mut shift: i32 = core::mem::zeroed();
let mut re_flags: i32 = core::mem::zeroed();
let mut next_src_pos: i32 = core::mem::zeroed();
let mut start: i32 = core::mem::zeroed();
let mut end: i32 = core::mem::zeroed();
let mut last_index: i64 = core::mem::zeroed();
let mut b_s: StringBuffer = core::mem::zeroed();
let mut b: *mut StringBuffer = core::mem::zeroed();
let mut rp: *mut JSString = core::mem::zeroed();
let mut group_name_ptr: *const c_char = core::mem::zeroed();
let mut fullUnicode: i32 = core::mem::zeroed();
let mut vm_block: usize = 74;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 48214
1 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 48213
2 => {
let _ = string_buffer_free(b);
vm_block = 1; continue;
}
// C line 48212
3 => {
let _ = js_free(ctx, ((capture) as *mut c_void));
vm_block = 2; continue;
}
// C line ? labels: fail
4 => {
let _ = JS_FreeValue(ctx, str_val);
vm_block = 3; continue;
}
// C line 48209
5 => {
return string_buffer_end(b);
}
// C line 48208
6 => {
let _ = js_free(ctx, ((capture) as *mut c_void));
vm_block = 5; continue;
}
// C line 48207
7 => {
let _ = JS_FreeValue(ctx, str_val);
vm_block = 6; continue;
}
// C line 48206
8 => {
vm_block = 4; continue;
}
// C line 48205
9 => {
vm_block = if (string_buffer_concat(b, str, ((next_src_pos) as u32), (*(str)).len())) != 0 { 8 } else { 7 }; continue;
}
// C line 48155
10 => {
vm_block = 41; continue;
}
// C line 48203
11 => {
let _ = { let assigned = ((end) as i64); last_index = assigned; assigned };
vm_block = 10; continue;
}
// C line 48201
12 => {
let _ = { let assigned = ((string_advance_index(str, ((end) as i64), fullUnicode)) as i32); end = assigned; assigned };
vm_block = 11; continue;
}
// C line 48200
13 => {
vm_block = if ((((end) == (start)) as i32)) != 0 { 12 } else { 11 }; continue;
}
// C line 48198
14 => {
vm_block = 9; continue;
}
// C line 48196
15 => {
vm_block = 4; continue;
}
// C line 48195
16 => {
vm_block = if ((((js_regexp_set_lastIndex(ctx, this_val, end)) < ((0 as i32))) as i32)) != 0 { 15 } else { 14 }; continue;
}
// C line 48194
17 => {
vm_block = if (((re_flags) & (((1 as i32)).wrapping_shl(((5 as i32)) as u32)))) != 0 { 16 } else { 14 }; continue;
}
// C line 48193
18 => {
vm_block = if ((!((((re_flags) & (((1 as i32)).wrapping_shl(((0 as i32)) as u32)))) != 0) as i32)) != 0 { 17 } else { 13 }; continue;
}
// C line 48192
19 => {
let _ = { let assigned = end; next_src_pos = assigned; assigned };
vm_block = 18; continue;
}
// C line 48189
20 => {
vm_block = 4; continue;
}
// C line 48186
21 => {
vm_block = if (js_string_GetSubstitution(ctx, b, JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }, str, ((start) as u32), JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }, JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }, rep_val, capture, ((capture_count) as u32))) != 0 { 20 } else { 19 }; continue;
}
// C line 48185
22 => {
vm_block = if (((((((*(rp)).len()) as i32)) != ((0 as i32))) as i32)) != 0 { 21 } else { 19 }; continue;
}
// C line 48183
23 => {
vm_block = 4; continue;
}
// C line 48182
24 => {
vm_block = if (string_buffer_concat(b, str, ((next_src_pos) as u32), ((start) as u32))) != 0 { 23 } else { 22 }; continue;
}
// C line 48181
25 => {
vm_block = if ((((next_src_pos) < (start)) as i32)) != 0 { 24 } else { 22 }; continue;
}
// C line 48180
26 => {
let _ = { let assigned = ((end) as i64); last_index = assigned; assigned };
vm_block = 25; continue;
}
// C line 48179
27 => {
let _ = { let assigned = (((((*(capture).offset(((1 as i32)) as isize)).offset_from(str_buf) as i64)).wrapping_shr((shift) as u32)) as i32); end = assigned; assigned };
vm_block = 26; continue;
}
// C line 48178
28 => {
let _ = { let assigned = (((((*(capture).offset(((0 as i32)) as isize)).offset_from(str_buf) as i64)).wrapping_shr((shift) as u32)) as i32); start = assigned; assigned };
vm_block = 27; continue;
}
// C line 48176
29 => {
vm_block = 9; continue;
}
// C line 48166
30 => {
vm_block = 4; continue;
}
// C line 48165
31 => {
vm_block = if ((((js_regexp_set_lastIndex(ctx, this_val, (0 as i32))) < ((0 as i32))) as i32)) != 0 { 30 } else { 29 }; continue;
}
// C line 48164
32 => {
vm_block = if ((((((((ret) == ((2 as i32))) as i32)) != 0) || ((((re_flags) & (((((1 as i32)).wrapping_shl(((0 as i32)) as u32)) | (((1 as i32)).wrapping_shl(((5 as i32)) as u32)))))) != 0)) as i32)) != 0 { 31 } else { 29 }; continue;
}
// C line 48174
33 => {
vm_block = 4; continue;
}
// C line 48170
34 => {
let _ = JS_ThrowInterrupted(ctx);
vm_block = 33; continue;
}
// C line 48172
35 => {
let _ = JS_ThrowInternalError(ctx, c"out of memory in regexp execution".as_ptr());
vm_block = 33; continue;
}
// C line 48169
36 => {
vm_block = if ((((ret) == (((2 as i32)).wrapping_neg())) as i32)) != 0 { 34 } else { 35 }; continue;
}
// C line 48163
37 => {
vm_block = if ((((ret) >= ((0 as i32))) as i32)) != 0 { 32 } else { 36 }; continue;
}
// C line 48162
38 => {
vm_block = if ((((ret) != ((1 as i32))) as i32)) != 0 { 37 } else { 28 }; continue;
}
// C line 48157
39 => {
let _ = { let assigned = (0 as i32); ret = assigned; assigned };
vm_block = 38; continue;
}
// C line 48159
40 => {
let _ = { let assigned = lre_exec(capture, re_bytecode, str_buf, ((last_index) as i32), (((*(str)).len()) as i32), shift, ((ctx) as *mut c_void)); ret = assigned; assigned };
vm_block = 38; continue;
}
// C line 48156
41 => {
vm_block = if ((((last_index) > ((((*(str)).len()) as i64))) as i32)) != 0 { 39 } else { 40 }; continue;
}
// C line 48154
42 => {
let _ = { let assigned = (0 as i32); next_src_pos = assigned; assigned };
vm_block = 10; continue;
}
// C line 48153
43 => {
let _ = { let assigned = (((*(str)).u).str8).as_mut_ptr(); str_buf = assigned; assigned };
vm_block = 42; continue;
}
// C line 48152
44 => {
let _ = { let assigned = (((*(str)).is_wide_char()) as i32); shift = assigned; assigned };
vm_block = 43; continue;
}
// C line 48151
45 => {
let _ = { let assigned = (((((re_flags) & (((((1 as i32)).wrapping_shl(((4 as i32)) as u32)) | (((1 as i32)).wrapping_shl(((8 as i32)) as u32)))))) != ((0 as i32))) as i32); fullUnicode = assigned; assigned };
vm_block = 44; continue;
}
// C line 48150
46 => {
let _ = { let assigned = lre_get_capture_count(re_bytecode); capture_count = assigned; assigned };
vm_block = 45; continue;
}
// C line 48148
47 => {
vm_block = 4; continue;
}
// C line 48147
48 => {
vm_block = if ((!(!(capture).is_null()) as i32)) != 0 { 47 } else { 46 }; continue;
}
// C line 48146
49 => {
let _ = { let assigned = ((js_malloc(ctx, ((size_of::<*mut u8>() as usize)).wrapping_mul(((alloc_count) as usize)))) as *mut *mut u8); capture = assigned; assigned };
vm_block = 48; continue;
}
// C line 48145
50 => {
vm_block = if ((((alloc_count) > ((0 as i32))) as i32)) != 0 { 49 } else { 46 }; continue;
}
// C line 48144
51 => {
let _ = { let assigned = lre_get_alloc_count(re_bytecode); alloc_count = assigned; assigned };
vm_block = 50; continue;
}
// C line 48139
52 => {
let _ = { let assigned = (((0 as i32)) as i64); last_index = assigned; assigned };
vm_block = 51; continue;
}
// C line 48142
53 => {
vm_block = 4; continue;
}
// C line 48141
54 => {
vm_block = if (js_regexp_get_lastIndex(ctx, core::ptr::addr_of_mut!(last_index), this_val)) != 0 { 53 } else { 51 }; continue;
}
// C line 48138
55 => {
vm_block = if ((((((re_flags) & (((((1 as i32)).wrapping_shl(((0 as i32)) as u32)) | (((1 as i32)).wrapping_shl(((5 as i32)) as u32)))))) == ((0 as i32))) as i32)) != 0 { 52 } else { 54 }; continue;
}
// C line 48136
56 => {
vm_block = 4; continue;
}
// C line 48135
57 => {
vm_block = if (js_regexp_set_lastIndex(ctx, this_val, (0 as i32))) != 0 { 56 } else { 55 }; continue;
}
// C line 48134
58 => {
vm_block = if (((re_flags) & (((1 as i32)).wrapping_shl(((0 as i32)) as u32)))) != 0 { 57 } else { 55 }; continue;
}
// C line 48132
59 => {
let _ = { let assigned = lre_get_flags(re_bytecode); re_flags = assigned; assigned };
vm_block = 58; continue;
}
// C line 48131
60 => {
let _ = { let assigned = ((((str_val).u).ptr) as *mut JSString); str = assigned; assigned };
vm_block = 59; continue;
}
// C line 48130
61 => {
vm_block = 4; continue;
}
// C line 48129
62 => {
vm_block = if (JS_IsException(str_val)) != 0 { 61 } else { 60 }; continue;
}
// C line 48128
63 => {
let _ = { let assigned = JS_ToString(ctx, arg); str_val = assigned; assigned };
vm_block = 62; continue;
}
// C line 48127
64 => {
let _ = { let assigned = core::ptr::null_mut::<*mut u8>(); capture = assigned; assigned };
vm_block = 63; continue;
}
// C line 48125
65 => {
let _ = string_buffer_init(ctx, b, (0 as i32));
vm_block = 64; continue;
}
// C line 48123
66 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) };
}
// C line 48122
67 => {
vm_block = if !(group_name_ptr).is_null() { 66 } else { 65 }; continue;
}
// C line 48121
68 => {
let _ = { let assigned = lre_get_groupnames(re_bytecode); group_name_ptr = assigned; assigned };
vm_block = 67; continue;
}
// C line 48120
69 => {
let _ = { let assigned = (((*((*(re)).bytecode)).u).str8).as_mut_ptr(); re_bytecode = assigned; assigned };
vm_block = 68; continue;
}
// C line 48119
70 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 48118
71 => {
vm_block = if ((!(!(re).is_null()) as i32)) != 0 { 70 } else { 69 }; continue;
}
// C line 48114
72 => {
rp = ((((rep_val).u).ptr) as *mut JSString);
vm_block = 71; continue;
}
// C line 48113
73 => {
b = core::ptr::addr_of_mut!(b_s);
vm_block = 72; continue;
}
// C line 48104
74 => {
re = js_get_regexp(ctx, this_val, (1 as i32));
vm_block = 73; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:48217. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code, unused_variables)]
unsafe fn JS_RegExpExec(mut ctx: *mut JSContext, mut r: JSValue, mut s: JSValue) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut method: JSValue = core::mem::zeroed();
let mut ret: JSValue = core::mem::zeroed();
let mut vm_block: usize = 13;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 48235
1 => {
return js_regexp_exec(ctx, r, (1 as i32), core::ptr::addr_of_mut!(s));
}
// C line 48234
2 => {
let _ = JS_FreeValue(ctx, method);
vm_block = 1; continue;
}
// C line 48232
3 => {
return ret;
}
// C line 48230
4 => {
return JS_ThrowTypeError(ctx, c"RegExp exec method must return an object or null".as_ptr());
}
// C line 48229
5 => {
let _ = JS_FreeValue(ctx, ret);
vm_block = 4; continue;
}
// C line 48228
6 => {
vm_block = if ((((((!((JS_IsObject(ret)) != 0) as i32)) != 0) && (((!((JS_IsNull(ret)) != 0) as i32)) != 0)) as i32)) != 0 { 5 } else { 3 }; continue;
}
// C line 48227
7 => {
return ret;
}
// C line 48226
8 => {
vm_block = if (JS_IsException(ret)) != 0 { 7 } else { 6 }; continue;
}
// C line 48225
9 => {
let _ = { let assigned = JS_CallFree(ctx, method, r, (1 as i32), core::ptr::addr_of_mut!(s)); ret = assigned; assigned };
vm_block = 8; continue;
}
// C line 48224
10 => {
vm_block = if (JS_IsFunction(ctx, method)) != 0 { 9 } else { 2 }; continue;
}
// C line 48223
11 => {
return method;
}
// C line 48222
12 => {
vm_block = if (JS_IsException(method)) != 0 { 11 } else { 10 }; continue;
}
// C line 48221
13 => {
let _ = { let assigned = JS_GetProperty(ctx, r, (((crate::quickjs_atom::JS_ATOM_exec as i32)) as JSAtom)); method = assigned; assigned };
vm_block = 12; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:48238. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code, unused_variables)]
unsafe fn js_regexp_test(mut ctx: *mut JSContext, mut this_val: JSValue, mut argc: i32, mut argv: *mut JSValue) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut val: JSValue = core::mem::zeroed();
let mut ret: i32 = core::mem::zeroed();
let mut vm_block: usize = 6;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 48249
1 => {
return JS_NewBool(ctx, ret);
}
// C line 48248
2 => {
let _ = JS_FreeValue(ctx, val);
vm_block = 1; continue;
}
// C line 48247
3 => {
let _ = { let assigned = (!((JS_IsNull(val)) != 0) as i32); ret = assigned; assigned };
vm_block = 2; continue;
}
// C line 48246
4 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 48245
5 => {
vm_block = if (JS_IsException(val)) != 0 { 4 } else { 3 }; continue;
}
// C line 48244
6 => {
let _ = { let assigned = JS_RegExpExec(ctx, this_val, *(argv).offset(((0 as i32)) as isize)); val = assigned; assigned };
vm_block = 5; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:48252. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code, unused_variables)]
unsafe fn js_regexp_Symbol_match(mut ctx: *mut JSContext, mut this_val: JSValue, mut argc: i32, mut argv: *mut JSValue) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut rx: JSValue = core::mem::zeroed();
let mut A: JSValue = core::mem::zeroed();
let mut S: JSValue = core::mem::zeroed();
let mut flags: JSValue = core::mem::zeroed();
let mut result: JSValue = core::mem::zeroed();
let mut matchStr: JSValue = core::mem::zeroed();
let mut global: i32 = core::mem::zeroed();
let mut n: i32 = core::mem::zeroed();
let mut fullUnicode: i32 = core::mem::zeroed();
let mut isEmpty: i32 = core::mem::zeroed();
let mut p: *mut JSString = core::mem::zeroed();
let mut thisIndex: i64 = core::mem::zeroed();
let mut nextIndex: i64 = core::mem::zeroed();
let mut vm_block: usize = 59;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 48332
1 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 48331
2 => {
let _ = JS_FreeValue(ctx, S);
vm_block = 1; continue;
}
// C line 48330
3 => {
let _ = JS_FreeValue(ctx, flags);
vm_block = 2; continue;
}
// C line 48329
4 => {
let _ = JS_FreeValue(ctx, result);
vm_block = 3; continue;
}
// C line ? labels: exception
5 => {
let _ = JS_FreeValue(ctx, A);
vm_block = 4; continue;
}
// C line 48325
6 => {
return A;
}
// C line 48324
7 => {
let _ = JS_FreeValue(ctx, S);
vm_block = 6; continue;
}
// C line 48323
8 => {
let _ = JS_FreeValue(ctx, flags);
vm_block = 7; continue;
}
// C line 48322
9 => {
let _ = JS_FreeValue(ctx, result);
vm_block = 8; continue;
}
// C line 48282
10 => {
let _ = { let assigned = JS_RegExpExec(ctx, rx, S); A = assigned; assigned };
vm_block = 9; continue;
}
// C line 48319
11 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_NULL as i32)) as i64) }; A = assigned; assigned };
vm_block = 9; continue;
}
// C line 48318
12 => {
let _ = JS_FreeValue(ctx, A);
vm_block = 11; continue;
}
// C line 48317
13 => {
vm_block = if ((((n) == ((0 as i32))) as i32)) != 0 { 12 } else { 9 }; continue;
}
// C line 48293
14 => {
vm_block = 33; continue;
}
// C line 48314
15 => {
vm_block = 5; continue;
}
// C line 48313
16 => {
vm_block = if ((((JS_SetProperty(ctx, rx, (((crate::quickjs_atom::JS_ATOM_lastIndex as i32)) as JSAtom), JS_NewInt64(ctx, nextIndex))) < ((0 as i32))) as i32)) != 0 { 15 } else { 14 }; continue;
}
// C line 48312
17 => {
let _ = { let assigned = string_advance_index(p, thisIndex, fullUnicode); nextIndex = assigned; assigned };
vm_block = 16; continue;
}
// C line 48311
18 => {
let _ = { let assigned = ((((S).u).ptr) as *mut JSString); p = assigned; assigned };
vm_block = 17; continue;
}
// C line 48310
19 => {
vm_block = 5; continue;
}
// C line 48308
20 => {
vm_block = if ((((JS_ToLengthFree(ctx, core::ptr::addr_of_mut!(thisIndex), JS_GetProperty(ctx, rx, (((crate::quickjs_atom::JS_ATOM_lastIndex as i32)) as JSAtom)))) < ((0 as i32))) as i32)) != 0 { 19 } else { 18 }; continue;
}
// C line 48306
21 => {
vm_block = if (isEmpty) != 0 { 20 } else { 14 }; continue;
}
// C line 48305
22 => {
vm_block = 5; continue;
}
// C line 48304
23 => {
vm_block = if ((((JS_DefinePropertyValueInt64(ctx, A, (({ let old = n; n = (n).wrapping_add(1); old }) as i64), matchStr, ((((((((1 as i32)).wrapping_shl(((0 as i32)) as u32)) | (((1 as i32)).wrapping_shl(((1 as i32)) as u32)))) | (((1 as i32)).wrapping_shl(((2 as i32)) as u32)))) | (((1 as i32)).wrapping_shl(((14 as i32)) as u32))))) < ((0 as i32))) as i32)) != 0 { 22 } else { 21 }; continue;
}
// C line 48303
24 => {
let _ = { let assigned = JS_IsEmptyString(matchStr); isEmpty = assigned; assigned };
vm_block = 23; continue;
}
// C line 48302
25 => {
vm_block = 5; continue;
}
// C line 48301
26 => {
vm_block = if (JS_IsException(matchStr)) != 0 { 25 } else { 24 }; continue;
}
// C line 48300
27 => {
let _ = { let assigned = JS_ToStringFree(ctx, JS_GetPropertyInt64(ctx, result, (((0 as i32)) as i64))); matchStr = assigned; assigned };
vm_block = 26; continue;
}
// C line 48299
28 => {
vm_block = 13; continue;
}
// C line 48298
29 => {
vm_block = if (JS_IsNull(result)) != 0 { 28 } else { 27 }; continue;
}
// C line 48297
30 => {
vm_block = 5; continue;
}
// C line 48296
31 => {
vm_block = if (JS_IsException(result)) != 0 { 30 } else { 29 }; continue;
}
// C line 48295
32 => {
let _ = { let assigned = JS_RegExpExec(ctx, rx, S); result = assigned; assigned };
vm_block = 31; continue;
}
// C line 48294
33 => {
let _ = JS_FreeValue(ctx, result);
vm_block = 32; continue;
}
// C line 48292
34 => {
let _ = { let assigned = (0 as i32); n = assigned; assigned };
vm_block = 14; continue;
}
// C line 48291
35 => {
vm_block = 5; continue;
}
// C line 48290
36 => {
vm_block = if (JS_IsException(A)) != 0 { 35 } else { 34 }; continue;
}
// C line 48289
37 => {
let _ = { let assigned = JS_NewArray(ctx); A = assigned; assigned };
vm_block = 36; continue;
}
// C line 48288
38 => {
vm_block = 5; continue;
}
// C line 48287
39 => {
vm_block = if ((((JS_SetProperty(ctx, rx, (((crate::quickjs_atom::JS_ATOM_lastIndex as i32)) as JSAtom), JS_NewInt32(ctx, (0 as i32)))) < ((0 as i32))) as i32)) != 0 { 38 } else { 37 }; continue;
}
// C line 48284
40 => {
let _ = { let assigned = (((((((string_indexof_char(p, (117 as i32), (0 as i32))) >= ((0 as i32))) as i32)) != 0) || (((((string_indexof_char(p, (118 as i32), (0 as i32))) >= ((0 as i32))) as i32)) != 0)) as i32); fullUnicode = assigned; assigned };
vm_block = 39; continue;
}
// C line 48281
41 => {
vm_block = if ((!((global) != 0) as i32)) != 0 { 10 } else { 40 }; continue;
}
// C line 48280
42 => {
let _ = { let assigned = (((((1 as i32)).wrapping_neg()) != (string_indexof_char(p, (103 as i32), (0 as i32)))) as i32); global = assigned; assigned };
vm_block = 41; continue;
}
// C line 48278
43 => {
let _ = { let assigned = ((((flags).u).ptr) as *mut JSString); p = assigned; assigned };
vm_block = 42; continue;
}
// C line 48277
44 => {
vm_block = 5; continue;
}
// C line 48276
45 => {
vm_block = if (JS_IsException(flags)) != 0 { 44 } else { 43 }; continue;
}
// C line 48275
46 => {
let _ = { let assigned = JS_ToStringFree(ctx, flags); flags = assigned; assigned };
vm_block = 45; continue;
}
// C line 48274
47 => {
vm_block = 5; continue;
}
// C line 48273
48 => {
vm_block = if (JS_IsException(flags)) != 0 { 47 } else { 46 }; continue;
}
// C line 48272
49 => {
let _ = { let assigned = JS_GetProperty(ctx, rx, (((crate::quickjs_atom::JS_ATOM_flags as i32)) as JSAtom)); flags = assigned; assigned };
vm_block = 48; continue;
}
// C line 48270
50 => {
vm_block = 5; continue;
}
// C line 48269
51 => {
vm_block = if (JS_IsException(S)) != 0 { 50 } else { 49 }; continue;
}
// C line 48268
52 => {
let _ = { let assigned = JS_ToString(ctx, *(argv).offset(((0 as i32)) as isize)); S = assigned; assigned };
vm_block = 51; continue;
}
// C line 48267
53 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; matchStr = assigned; assigned };
vm_block = 52; continue;
}
// C line 48266
54 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; result = assigned; assigned };
vm_block = 53; continue;
}
// C line 48265
55 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; flags = assigned; assigned };
vm_block = 54; continue;
}
// C line 48264
56 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; A = assigned; assigned };
vm_block = 55; continue;
}
// C line 48262
57 => {
return JS_ThrowTypeErrorNotAnObject(ctx);
}
// C line 48261
58 => {
vm_block = if ((!((JS_IsObject(rx)) != 0) as i32)) != 0 { 57 } else { 56 }; continue;
}
// C line 48256
59 => {
rx = this_val;
vm_block = 58; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:48365. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code, unused_variables)]
unsafe fn js_regexp_string_iterator_next(mut ctx: *mut JSContext, mut this_val: JSValue, mut argc: i32, mut argv: *mut JSValue, mut pdone: *mut i32, mut magic: i32) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut it: *mut JSRegExpStringIteratorData = core::mem::zeroed();
let mut R: JSValue = core::mem::zeroed();
let mut S: JSValue = core::mem::zeroed();
let mut matchStr: JSValue = core::mem::zeroed();
let mut v_match: JSValue = core::mem::zeroed();
let mut sp: *mut JSString = core::mem::zeroed();
let mut thisIndex: i64 = core::mem::zeroed();
let mut nextIndex: i64 = core::mem::zeroed();
let mut vm_block: usize = 35;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 48416
1 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 48415
2 => {
let _ = { let assigned = (0 as i32); *(pdone) = assigned; assigned };
vm_block = 1; continue;
}
// C line 48414
3 => {
let _ = JS_FreeValue(ctx, matchStr);
vm_block = 2; continue;
}
// C line ? labels: exception
4 => {
let _ = JS_FreeValue(ctx, v_match);
vm_block = 3; continue;
}
// C line 48411
5 => {
return v_match;
}
// C line 48410
6 => {
let _ = { let assigned = (0 as i32); *(pdone) = assigned; assigned };
vm_block = 5; continue;
}
// C line 48390
7 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) };
}
// C line 48389
8 => {
let _ = { let assigned = (1 as i32); *(pdone) = assigned; assigned };
vm_block = 7; continue;
}
// C line 48388
9 => {
let _ = { let assigned = (1 as i32); (*(it)).done = assigned; assigned };
vm_block = 8; continue;
}
// C line 48406
10 => {
let _ = JS_FreeValue(ctx, matchStr);
vm_block = 6; continue;
}
// C line 48404
11 => {
vm_block = 4; continue;
}
// C line 48402
12 => {
vm_block = if ((((JS_SetProperty(ctx, R, (((crate::quickjs_atom::JS_ATOM_lastIndex as i32)) as JSAtom), JS_NewInt64(ctx, nextIndex))) < ((0 as i32))) as i32)) != 0 { 11 } else { 10 }; continue;
}
// C line 48401
13 => {
let _ = { let assigned = string_advance_index(sp, thisIndex, (*(it)).unicode); nextIndex = assigned; assigned };
vm_block = 12; continue;
}
// C line 48400
14 => {
let _ = { let assigned = ((((S).u).ptr) as *mut JSString); sp = assigned; assigned };
vm_block = 13; continue;
}
// C line 48399
15 => {
vm_block = 4; continue;
}
// C line 48397
16 => {
vm_block = if ((((JS_ToLengthFree(ctx, core::ptr::addr_of_mut!(thisIndex), JS_GetProperty(ctx, R, (((crate::quickjs_atom::JS_ATOM_lastIndex as i32)) as JSAtom)))) < ((0 as i32))) as i32)) != 0 { 15 } else { 14 }; continue;
}
// C line 48395
17 => {
vm_block = if (JS_IsEmptyString(matchStr)) != 0 { 16 } else { 10 }; continue;
}
// C line 48394
18 => {
vm_block = 4; continue;
}
// C line 48393
19 => {
vm_block = if (JS_IsException(matchStr)) != 0 { 18 } else { 17 }; continue;
}
// C line 48392
20 => {
let _ = { let assigned = JS_ToStringFree(ctx, JS_GetPropertyInt64(ctx, v_match, (((0 as i32)) as i64))); matchStr = assigned; assigned };
vm_block = 19; continue;
}
// C line 48408
21 => {
let _ = { let assigned = (1 as i32); (*(it)).done = assigned; assigned };
vm_block = 6; continue;
}
// C line 48391
22 => {
vm_block = if ((*(it)).global) != 0 { 20 } else { 21 }; continue;
}
// C line 48387
23 => {
vm_block = if (JS_IsNull(v_match)) != 0 { 9 } else { 22 }; continue;
}
// C line 48386
24 => {
vm_block = 4; continue;
}
// C line 48385
25 => {
vm_block = if (JS_IsException(v_match)) != 0 { 24 } else { 23 }; continue;
}
// C line 48384
26 => {
let _ = { let assigned = JS_RegExpExec(ctx, R, S); v_match = assigned; assigned };
vm_block = 25; continue;
}
// C line 48383
27 => {
let _ = { let assigned = (*(it)).iterated_string; S = assigned; assigned };
vm_block = 26; continue;
}
// C line 48382
28 => {
let _ = { let assigned = (*(it)).iterating_regexp; R = assigned; assigned };
vm_block = 27; continue;
}
// C line 48380
29 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) };
}
// C line 48379
30 => {
let _ = { let assigned = (1 as i32); *(pdone) = assigned; assigned };
vm_block = 29; continue;
}
// C line 48378
31 => {
vm_block = if ((*(it)).done) != 0 { 30 } else { 28 }; continue;
}
// C line 48377
32 => {
vm_block = 4; continue;
}
// C line 48376
33 => {
vm_block = if ((!(!(it).is_null()) as i32)) != 0 { 32 } else { 31 }; continue;
}
// C line 48375
34 => {
let _ = { let assigned = ((JS_GetOpaque2(ctx, this_val, (((JS_CLASS_REGEXP_STRING_ITERATOR as i32)) as JSClassID))) as *mut JSRegExpStringIteratorData); it = assigned; assigned };
vm_block = 33; continue;
}
// C line 48372
35 => {
matchStr = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) };
v_match = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) };
vm_block = 34; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:48419. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code, unused_variables)]
unsafe fn js_regexp_Symbol_matchAll(mut ctx: *mut JSContext, mut this_val: JSValue, mut argc: i32, mut argv: *mut JSValue) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut R: JSValue = core::mem::zeroed();
let mut S: JSValue = core::mem::zeroed();
let mut C: JSValue = core::mem::zeroed();
let mut flags: JSValue = core::mem::zeroed();
let mut matcher: JSValue = core::mem::zeroed();
let mut iter: JSValue = core::mem::zeroed();
let mut args: [JSValue; 2] = core::mem::zeroed();
let mut strp: *mut JSString = core::mem::zeroed();
let mut lastIndex: i64 = core::mem::zeroed();
let mut it: *mut JSRegExpStringIteratorData = core::mem::zeroed();
let mut vm_block: usize = 47;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 48483
1 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 48482
2 => {
let _ = JS_FreeValue(ctx, iter);
vm_block = 1; continue;
}
// C line 48481
3 => {
let _ = JS_FreeValue(ctx, matcher);
vm_block = 2; continue;
}
// C line 48480
4 => {
let _ = JS_FreeValue(ctx, flags);
vm_block = 3; continue;
}
// C line 48479
5 => {
let _ = JS_FreeValue(ctx, C);
vm_block = 4; continue;
}
// C line ? labels: exception
6 => {
let _ = JS_FreeValue(ctx, S);
vm_block = 5; continue;
}
// C line 48476
7 => {
return iter;
}
// C line 48475
8 => {
let _ = JS_FreeValue(ctx, flags);
vm_block = 7; continue;
}
// C line 48474
9 => {
let _ = JS_FreeValue(ctx, C);
vm_block = 8; continue;
}
// C line 48472
10 => {
let _ = JS_SetOpaque(iter, ((it) as *mut c_void));
vm_block = 9; continue;
}
// C line 48471
11 => {
let _ = { let assigned = (0 as i32); (*(it)).done = assigned; assigned };
vm_block = 10; continue;
}
// C line 48469
12 => {
let _ = { let assigned = (((((((string_indexof_char(strp, (117 as i32), (0 as i32))) >= ((0 as i32))) as i32)) != 0) || (((((string_indexof_char(strp, (118 as i32), (0 as i32))) >= ((0 as i32))) as i32)) != 0)) as i32); (*(it)).unicode = assigned; assigned };
vm_block = 11; continue;
}
// C line 48468
13 => {
let _ = { let assigned = (((string_indexof_char(strp, (103 as i32), (0 as i32))) >= ((0 as i32))) as i32); (*(it)).global = assigned; assigned };
vm_block = 12; continue;
}
// C line 48467
14 => {
let _ = { let assigned = ((((flags).u).ptr) as *mut JSString); strp = assigned; assigned };
vm_block = 13; continue;
}
// C line 48466
15 => {
let _ = { let assigned = S; (*(it)).iterated_string = assigned; assigned };
vm_block = 14; continue;
}
// C line 48465
16 => {
let _ = { let assigned = matcher; (*(it)).iterating_regexp = assigned; assigned };
vm_block = 15; continue;
}
// C line 48464
17 => {
vm_block = 6; continue;
}
// C line 48463
18 => {
vm_block = if ((!(!(it).is_null()) as i32)) != 0 { 17 } else { 16 }; continue;
}
// C line 48462
19 => {
let _ = { let assigned = ((js_malloc(ctx, (size_of::<JSRegExpStringIteratorData>() as usize))) as *mut JSRegExpStringIteratorData); it = assigned; assigned };
vm_block = 18; continue;
}
// C line 48461
20 => {
vm_block = 6; continue;
}
// C line 48460
21 => {
vm_block = if (JS_IsException(iter)) != 0 { 20 } else { 19 }; continue;
}
// C line 48459
22 => {
let _ = { let assigned = JS_NewObjectClass(ctx, (JS_CLASS_REGEXP_STRING_ITERATOR as i32)); iter = assigned; assigned };
vm_block = 21; continue;
}
// C line 48457
23 => {
vm_block = 6; continue;
}
// C line 48455
24 => {
vm_block = if ((((JS_SetProperty(ctx, matcher, (((crate::quickjs_atom::JS_ATOM_lastIndex as i32)) as JSAtom), JS_NewInt64(ctx, lastIndex))) < ((0 as i32))) as i32)) != 0 { 23 } else { 22 }; continue;
}
// C line 48454
25 => {
vm_block = 6; continue;
}
// C line 48452
26 => {
vm_block = if (JS_ToLengthFree(ctx, core::ptr::addr_of_mut!(lastIndex), JS_GetProperty(ctx, R, (((crate::quickjs_atom::JS_ATOM_lastIndex as i32)) as JSAtom)))) != 0 { 25 } else { 24 }; continue;
}
// C line 48451
27 => {
vm_block = 6; continue;
}
// C line 48450
28 => {
vm_block = if (JS_IsException(matcher)) != 0 { 27 } else { 26 }; continue;
}
// C line 48449
29 => {
let _ = { let assigned = JS_CallConstructor(ctx, C, (2 as i32), (args).as_mut_ptr()); matcher = assigned; assigned };
vm_block = 28; continue;
}
// C line 48448
30 => {
let _ = { let assigned = flags; *((args).as_mut_ptr()).offset(((1 as i32)) as isize) = assigned; assigned };
vm_block = 29; continue;
}
// C line 48447
31 => {
let _ = { let assigned = R; *((args).as_mut_ptr()).offset(((0 as i32)) as isize) = assigned; assigned };
vm_block = 30; continue;
}
// C line 48446
32 => {
vm_block = 6; continue;
}
// C line 48445
33 => {
vm_block = if (JS_IsException(flags)) != 0 { 32 } else { 31 }; continue;
}
// C line 48444
34 => {
let _ = { let assigned = JS_ToStringFree(ctx, JS_GetProperty(ctx, R, (((crate::quickjs_atom::JS_ATOM_flags as i32)) as JSAtom))); flags = assigned; assigned };
vm_block = 33; continue;
}
// C line 48443
35 => {
vm_block = 6; continue;
}
// C line 48442
36 => {
vm_block = if (JS_IsException(C)) != 0 { 35 } else { 34 }; continue;
}
// C line 48441
37 => {
let _ = { let assigned = JS_SpeciesConstructor(ctx, R, (*(ctx)).regexp_ctor); C = assigned; assigned };
vm_block = 36; continue;
}
// C line 48440
38 => {
vm_block = 6; continue;
}
// C line 48439
39 => {
vm_block = if (JS_IsException(S)) != 0 { 38 } else { 37 }; continue;
}
// C line 48438
40 => {
let _ = { let assigned = JS_ToString(ctx, *(argv).offset(((0 as i32)) as isize)); S = assigned; assigned };
vm_block = 39; continue;
}
// C line 48436
41 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; iter = assigned; assigned };
vm_block = 40; continue;
}
// C line 48435
42 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; matcher = assigned; assigned };
vm_block = 41; continue;
}
// C line 48434
43 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; flags = assigned; assigned };
vm_block = 42; continue;
}
// C line 48433
44 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; C = assigned; assigned };
vm_block = 43; continue;
}
// C line 48431
45 => {
return JS_ThrowTypeErrorNotAnObject(ctx);
}
// C line 48430
46 => {
vm_block = if ((!((JS_IsObject(R)) != 0) as i32)) != 0 { 45 } else { 44 }; continue;
}
// C line 48423
47 => {
R = this_val;
vm_block = 46; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:48495. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code, unused_variables)]
unsafe fn value_buffer_init(mut ctx: *mut JSContext, mut b: *mut ValueBuffer) -> i32 {
let mut vm_local_storage = Vec::<u64>::new();
let mut vm_block: usize = 6;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 48502
1 => {
return (0 as i32);
}
// C line 48501
2 => {
let _ = { let assigned = ((*(b)).def).as_mut_ptr(); (*(b)).arr = assigned; assigned };
vm_block = 1; continue;
}
// C line 48500
3 => {
let _ = { let assigned = (0 as i32); (*(b)).error_status = assigned; assigned };
vm_block = 2; continue;
}
// C line 48499
4 => {
let _ = { let assigned = (4 as i32); (*(b)).size = assigned; assigned };
vm_block = 3; continue;
}
// C line 48498
5 => {
let _ = { let assigned = (0 as i32); (*(b)).len = assigned; assigned };
vm_block = 4; continue;
}
// C line 48497
6 => {
let _ = { let assigned = ctx; (*(b)).ctx = assigned; assigned };
vm_block = 5; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:48505. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code, unused_variables)]
unsafe fn value_buffer_free(mut b: *mut ValueBuffer) -> () {
let mut vm_local_storage = Vec::<u64>::new();
let mut vm_block: usize = 5;
loop { match vm_block {
// C line ?
0 => {
return ();
}
// C line 48512
1 => {
let _ = { let assigned = (4 as i32); (*(b)).size = assigned; assigned };
vm_block = 0; continue;
}
// C line 48511
2 => {
let _ = { let assigned = ((*(b)).def).as_mut_ptr(); (*(b)).arr = assigned; assigned };
vm_block = 1; continue;
}
// C line 48510
3 => {
let _ = js_free((*(b)).ctx, (((*(b)).arr) as *mut c_void));
vm_block = 2; continue;
}
// C line 48509
4 => {
vm_block = if (((((*(b)).arr) != (((*(b)).def).as_mut_ptr())) as i32)) != 0 { 3 } else { 2 }; continue;
}
// C line 48507
5 => {
vm_block = if (((((*(b)).len) > ((0 as i32))) as i32)) != 0 { 6 } else { 4 }; continue;
}
// C line 48508
6 => {
let _ = JS_FreeValue((*(b)).ctx, *((*(b)).arr).offset(({ (*(b)).len = ((*(b)).len).wrapping_sub(1); (*(b)).len }) as isize));
vm_block = 5; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:48515. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code, unused_variables)]
unsafe fn value_buffer_append(mut b: *mut ValueBuffer, mut val: JSValue) -> i32 {
let mut vm_local_storage = Vec::<u64>::new();
let mut new_size: i32 = core::mem::zeroed();
let mut slack: usize = core::mem::zeroed();
let mut new_arr: *mut JSValue = core::mem::zeroed();
let mut vm_block: usize = 19;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 48543
1 => {
return (0 as i32);
}
// C line 48542
2 => {
let _ = { let assigned = val; *((*(b)).arr).offset(({ let old = (*(b)).len; (*(b)).len = ((*(b)).len).wrapping_add(1); old }) as isize) = assigned; assigned };
vm_block = 1; continue;
}
// C line 48540
3 => {
let _ = { let assigned = new_size; (*(b)).size = assigned; assigned };
vm_block = 2; continue;
}
// C line 48539
4 => {
let _ = { let assigned = new_arr; (*(b)).arr = assigned; assigned };
vm_block = 3; continue;
}
// C line 48538
5 => {
let _ = { let assigned = ((new_size) as usize).wrapping_add(((slack) / ((size_of::<JSValue>() as usize)))) as i32; new_size = assigned; assigned };
vm_block = 4; continue;
}
// C line 48536
6 => {
return ((1 as i32)).wrapping_neg();
}
// C line 48535
7 => {
let _ = { let assigned = ((1 as i32)).wrapping_neg(); (*(b)).error_status = assigned; assigned };
vm_block = 6; continue;
}
// C line 48534
8 => {
let _ = JS_FreeValue((*(b)).ctx, val);
vm_block = 7; continue;
}
// C line 48533
9 => {
let _ = value_buffer_free(b);
vm_block = 8; continue;
}
// C line 48532
10 => {
vm_block = if ((!(!(new_arr).is_null()) as i32)) != 0 { 9 } else { 5 }; continue;
}
// C line 48528
11 => {
let _ = { let dst = (((new_arr) as *mut c_void)) as *mut u8; core::ptr::copy_nonoverlapping((((((*(b)).def).as_mut_ptr()) as *const c_void)) as *const u8, dst, ((size_of::<[JSValue; 4]>() as usize)) as usize); dst as *mut c_void };
vm_block = 10; continue;
}
// C line 48527
12 => {
vm_block = if !(new_arr).is_null() { 11 } else { 10 }; continue;
}
// C line 48526
13 => {
let _ = { let assigned = ((js_realloc2((*(b)).ctx, core::ptr::null_mut::<c_void>(), ((size_of::<JSValue>() as usize)).wrapping_mul(((new_size) as usize)), core::ptr::addr_of_mut!(slack))) as *mut JSValue); new_arr = assigned; assigned };
vm_block = 12; continue;
}
// C line 48530
14 => {
let _ = { let assigned = ((js_realloc2((*(b)).ctx, (((*(b)).arr) as *mut c_void), ((size_of::<JSValue>() as usize)).wrapping_mul(((new_size) as usize)), core::ptr::addr_of_mut!(slack))) as *mut JSValue); new_arr = assigned; assigned };
vm_block = 10; continue;
}
// C line 48525
15 => {
vm_block = if (((((*(b)).arr) == (((*(b)).def).as_mut_ptr())) as i32)) != 0 { 13 } else { 14 }; continue;
}
// C line 48521
16 => {
new_size = (((((*(b)).len).wrapping_add(((*(b)).len).wrapping_shr(((1 as i32)) as u32))).wrapping_add((31 as i32))) & ((!((16 as i32)))));
vm_block = 15; continue;
}
// C line 48520
17 => {
vm_block = if (((((*(b)).len) >= ((*(b)).size)) as i32)) != 0 { 16 } else { 2 }; continue;
}
// C line 48518
18 => {
return ((1 as i32)).wrapping_neg();
}
// C line 48517
19 => {
vm_block = if ((*(b)).error_status) != 0 { 18 } else { 17 }; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:48547. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code, unused_variables)]
unsafe fn find_property_regexp(mut ppr: *mut *mut JSProperty, mut p: *mut JSObject, mut atom: JSAtom) -> *mut JSShapeProperty {
let mut vm_local_storage = Vec::<u64>::new();
let mut prs: *mut JSShapeProperty = core::mem::zeroed();
let mut vm_block: usize = 1;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 48552
1 => {
vm_block = 9; continue;
}
// C line 48560
2 => {
return core::ptr::null_mut::<JSShapeProperty>();
}
// C line 48559
3 => {
vm_block = if ((*(p)).is_exotic()) != 0 { 2 } else { 1 }; continue;
}
// C line 48558
4 => {
return core::ptr::null_mut::<JSShapeProperty>();
}
// C line 48557
5 => {
vm_block = if ((!(!(p).is_null()) as i32)) != 0 { 4 } else { 3 }; continue;
}
// C line 48556
6 => {
let _ = { let assigned = (*((*(p)).shape)).proto; p = assigned; assigned };
vm_block = 5; continue;
}
// C line 48555
7 => {
return prs;
}
// C line 48554
8 => {
vm_block = if !(prs).is_null() { 7 } else { 6 }; continue;
}
// C line 48553
9 => {
let _ = { let assigned = find_own_property(ppr, p, atom); prs = assigned; assigned };
vm_block = 8; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:48564. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code, unused_variables)]
unsafe fn check_regexp_getter(mut ctx: *mut JSContext, mut p: *mut JSObject, mut atom: JSAtom, mut func: Option<JSCFunction>, mut magic: i32) -> i32 {
let mut vm_local_storage = Vec::<u64>::new();
let mut pr: *mut JSProperty = core::mem::zeroed();
let mut prs: *mut JSShapeProperty = core::mem::zeroed();
let mut vm_block: usize = 8;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 48578
1 => {
return JS_IsCFunction(ctx, JSValue { u: JSValueUnion { ptr: (((((*(pr)).u).getset).getter) as *mut c_void) }, tag: (((JS_TAG_OBJECT as i32)) as i64) }, func, magic);
}
// C line 48577
2 => {
return (0 as i32);
}
// C line 48576
3 => {
vm_block = if ((!(!((((*(pr)).u).getset).getter).is_null()) as i32)) != 0 { 2 } else { 1 }; continue;
}
// C line 48575
4 => {
return (0 as i32);
}
// C line 48574
5 => {
vm_block = if (((((((((*(prs)).flags()) as i32)) & (((3 as i32)).wrapping_shl(((4 as i32)) as u32)))) != (((1 as i32)).wrapping_shl(((4 as i32)) as u32))) as i32)) != 0 { 4 } else { 3 }; continue;
}
// C line 48573
6 => {
return (0 as i32);
}
// C line 48572
7 => {
vm_block = if ((!(!(prs).is_null()) as i32)) != 0 { 6 } else { 5 }; continue;
}
// C line 48571
8 => {
let _ = { let assigned = find_property_regexp(core::ptr::addr_of_mut!(pr), p, atom); prs = assigned; assigned };
vm_block = 7; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:48582. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code, unused_variables)]
unsafe fn js_is_standard_regexp(mut ctx: *mut JSContext, mut obj: JSValue) -> i32 {
let mut vm_local_storage = Vec::<u64>::new();
let mut p: *mut JSObject = core::mem::zeroed();
let mut pr: *mut JSProperty = core::mem::zeroed();
let mut prs: *mut JSShapeProperty = core::mem::zeroed();
let mut ft: JSCFunctionType = core::mem::zeroed();
let mut vm_block: usize = 26;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 48619
1 => {
return (1 as i32);
}
// C line 48617
2 => {
return (0 as i32);
}
// C line 48616
3 => {
vm_block = if ((!((check_regexp_getter(ctx, p, (((crate::quickjs_atom::JS_ATOM_unicode as i32)) as JSAtom), (ft).generic, ((1 as i32)).wrapping_shl(((4 as i32)) as u32))) != 0) as i32)) != 0 { 2 } else { 1 }; continue;
}
// C line 48615
4 => {
return (0 as i32);
}
// C line 48614
5 => {
vm_block = if ((!((check_regexp_getter(ctx, p, (((crate::quickjs_atom::JS_ATOM_global as i32)) as JSAtom), (ft).generic, ((1 as i32)).wrapping_shl(((0 as i32)) as u32))) != 0) as i32)) != 0 { 4 } else { 3 }; continue;
}
// C line 48613
6 => {
let _ = { let assigned = js_regexp_get_flag; (ft).getter_magic = Some(assigned); assigned };
vm_block = 5; continue;
}
// C line 48612
7 => {
return (0 as i32);
}
// C line 48611
8 => {
vm_block = if ((!((check_regexp_getter(ctx, p, (((crate::quickjs_atom::JS_ATOM_flags as i32)) as JSAtom), (ft).generic, (0 as i32))) != 0) as i32)) != 0 { 7 } else { 6 }; continue;
}
// C line 48610
9 => {
let _ = { let assigned = js_regexp_get_flags; (ft).getter = Some(assigned); assigned };
vm_block = 8; continue;
}
// C line 48608
10 => {
return (0 as i32);
}
// C line 48607
11 => {
vm_block = if ((!((JS_IsCFunction(ctx, ((*(pr)).u).value, Some(js_regexp_exec), (0 as i32))) != 0) as i32)) != 0 { 10 } else { 9 }; continue;
}
// C line 48606
12 => {
return (0 as i32);
}
// C line 48605
13 => {
vm_block = if (((((((((*(prs)).flags()) as i32)) & (((3 as i32)).wrapping_shl(((4 as i32)) as u32)))) != (((0 as i32)).wrapping_shl(((4 as i32)) as u32))) as i32)) != 0 { 12 } else { 11 }; continue;
}
// C line 48604
14 => {
return (0 as i32);
}
// C line 48603
15 => {
vm_block = if ((!(!(prs).is_null()) as i32)) != 0 { 14 } else { 13 }; continue;
}
// C line 48602
16 => {
let _ = { let assigned = find_property_regexp(core::ptr::addr_of_mut!(pr), p, (((crate::quickjs_atom::JS_ATOM_exec as i32)) as JSAtom)); prs = assigned; assigned };
vm_block = 15; continue;
}
// C line 48599
17 => {
return (0 as i32);
}
// C line 48598
18 => {
vm_block = if ((!((JS_IsNumber(((*(pr)).u).value)) != 0) as i32)) != 0 { 17 } else { 16 }; continue;
}
// C line 48597
19 => {
return (0 as i32);
}
// C line 48596
20 => {
vm_block = if ((!(!(prs).is_null()) as i32)) != 0 { 19 } else { 18 }; continue;
}
// C line 48595
21 => {
let _ = { let assigned = find_own_property(core::ptr::addr_of_mut!(pr), p, (((crate::quickjs_atom::JS_ATOM_lastIndex as i32)) as JSAtom)); prs = assigned; assigned };
vm_block = 20; continue;
}
// C line 48593
22 => {
return (0 as i32);
}
// C line 48592
23 => {
vm_block = if (((((((*(p)).class_id) as i32)) != ((JS_CLASS_REGEXP as i32))) as i32)) != 0 { 22 } else { 21 }; continue;
}
// C line 48591
24 => {
let _ = { let assigned = ((((obj).u).ptr) as *mut JSObject); p = assigned; assigned };
vm_block = 23; continue;
}
// C line 48590
25 => {
return (0 as i32);
}
// C line 48589
26 => {
vm_block = if (((((((obj).tag) as i32)) != ((JS_TAG_OBJECT as i32))) as i32)) != 0 { 25 } else { 24 }; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:48622. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code, unused_variables)]
unsafe fn js_regexp_Symbol_replace(mut ctx: *mut JSContext, mut this_val: JSValue, mut argc: i32, mut argv: *mut JSValue) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut rx: JSValue = core::mem::zeroed();
let mut rep: JSValue = core::mem::zeroed();
let mut args: [JSValue; 6] = core::mem::zeroed();
let mut flags: JSValue = core::mem::zeroed();
let mut str: JSValue = core::mem::zeroed();
let mut rep_val: JSValue = core::mem::zeroed();
let mut matched: JSValue = core::mem::zeroed();
let mut tab: JSValue = core::mem::zeroed();
let mut rep_str: JSValue = core::mem::zeroed();
let mut namedCaptures: JSValue = core::mem::zeroed();
let mut res: JSValue = core::mem::zeroed();
let mut p: *mut JSString = core::mem::zeroed();
let mut sp: *mut JSString = core::mem::zeroed();
let mut b_s: StringBuffer = core::mem::zeroed();
let mut b: *mut StringBuffer = core::mem::zeroed();
let mut v_b: ValueBuffer = core::mem::zeroed();
let mut results: *mut ValueBuffer = core::mem::zeroed();
let mut nextSourcePosition: i32 = core::mem::zeroed();
let mut n: i32 = core::mem::zeroed();
let mut j: i32 = core::mem::zeroed();
let mut functionalReplace: i32 = core::mem::zeroed();
let mut is_global: i32 = core::mem::zeroed();
let mut fullUnicode: i32 = core::mem::zeroed();
let mut nCaptures: u32 = core::mem::zeroed();
let mut position: i64 = core::mem::zeroed();
let mut result: JSValue = core::mem::zeroed();
let mut thisIndex: i64 = core::mem::zeroed();
let mut nextIndex: i64 = core::mem::zeroed();
let mut result_1: JSValue = core::mem::zeroed();
let mut capN: JSValue = core::mem::zeroed();
let mut namedCaptures1: JSValue = core::mem::zeroed();
let mut b1_s: StringBuffer = core::mem::zeroed();
let mut b1: *mut StringBuffer = core::mem::zeroed();
let mut ret: i32 = core::mem::zeroed();
let mut vm_block: usize = 143;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 48814
1 => {
return res;
}
// C line 48813
2 => {
let _ = JS_FreeValue(ctx, str);
vm_block = 1; continue;
}
// C line 48812
3 => {
let _ = JS_FreeValue(ctx, namedCaptures);
vm_block = 2; continue;
}
// C line 48811
4 => {
let _ = JS_FreeValue(ctx, rep_str);
vm_block = 3; continue;
}
// C line 48810
5 => {
let _ = JS_FreeValue(ctx, tab);
vm_block = 4; continue;
}
// C line 48809
6 => {
let _ = JS_FreeValue(ctx, flags);
vm_block = 5; continue;
}
// C line 48808
7 => {
let _ = JS_FreeValue(ctx, matched);
vm_block = 6; continue;
}
// C line 48807
8 => {
let _ = JS_FreeValue(ctx, rep_val);
vm_block = 7; continue;
}
// C line ? labels: done1
9 => {
let _ = value_buffer_free(results);
vm_block = 8; continue;
}
// C line ? labels: done
10 => {
let _ = string_buffer_free(b);
vm_block = 9; continue;
}
// C line ? labels: exception
11 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) }; res = assigned; assigned };
vm_block = 10; continue;
}
// C line 48799
12 => {
vm_block = 9; continue;
}
// C line 48798
13 => {
let _ = { let assigned = string_buffer_end(b); res = assigned; assigned };
vm_block = 12; continue;
}
// C line 48797
14 => {
let _ = string_buffer_concat(b, sp, ((nextSourcePosition) as u32), (*(sp)).len());
vm_block = 13; continue;
}
// C line 48711
15 => {
vm_block = if ((((j) < ((*(results)).len)) as i32)) != 0 { 82 } else { 14 }; continue;
}
// C line ?
16 => {
let _ = { let old = j; j = (j).wrapping_add(1); old };
vm_block = 15; continue;
}
// C line 48794
17 => {
let _ = { let assigned = (((position).wrapping_add((((*(((((matched).u).ptr) as *mut JSString))).len()) as i64))) as i32); nextSourcePosition = assigned; assigned };
vm_block = 16; continue;
}
// C line 48793
18 => {
let _ = string_buffer_concat_value(b, rep_str);
vm_block = 17; continue;
}
// C line 48792
19 => {
let _ = string_buffer_concat(b, sp, ((nextSourcePosition) as u32), ((position) as u32));
vm_block = 18; continue;
}
// C line 48791
20 => {
vm_block = if ((((position) >= (((nextSourcePosition) as i64))) as i32)) != 0 { 19 } else { 16 }; continue;
}
// C line 48790
21 => {
vm_block = 11; continue;
}
// C line 48789
22 => {
vm_block = if (JS_IsException(rep_str)) != 0 { 21 } else { 20 }; continue;
}
// C line 48765
23 => {
let _ = { let assigned = JS_ToStringFree(ctx, js_function_apply(ctx, rep, (2 as i32), (args).as_mut_ptr(), (0 as i32))); rep_str = assigned; assigned };
vm_block = 22; continue;
}
// C line 48764
24 => {
let _ = JS_FreeValue(ctx, rep_str);
vm_block = 23; continue;
}
// C line 48763
25 => {
let _ = { let assigned = tab; *((args).as_mut_ptr()).offset(((1 as i32)) as isize) = assigned; assigned };
vm_block = 24; continue;
}
// C line 48762
26 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; *((args).as_mut_ptr()).offset(((0 as i32)) as isize) = assigned; assigned };
vm_block = 25; continue;
}
// C line 48760
27 => {
vm_block = 11; continue;
}
// C line 48759
28 => {
vm_block = if ((((JS_DefinePropertyValueInt64(ctx, tab, (({ let old = n; n = (n).wrapping_add(1); old }) as i64), JS_DupValue(ctx, namedCaptures), ((((((((1 as i32)).wrapping_shl(((0 as i32)) as u32)) | (((1 as i32)).wrapping_shl(((1 as i32)) as u32)))) | (((1 as i32)).wrapping_shl(((2 as i32)) as u32)))) | (((1 as i32)).wrapping_shl(((14 as i32)) as u32))))) < ((0 as i32))) as i32)) != 0 { 27 } else { 26 }; continue;
}
// C line 48758
29 => {
vm_block = if ((!((JS_IsUndefined(namedCaptures)) != 0) as i32)) != 0 { 28 } else { 26 }; continue;
}
// C line 48757
30 => {
vm_block = 11; continue;
}
// C line 48756
31 => {
vm_block = if ((((JS_DefinePropertyValueInt64(ctx, tab, (({ let old = n; n = (n).wrapping_add(1); old }) as i64), JS_DupValue(ctx, str), ((((((((1 as i32)).wrapping_shl(((0 as i32)) as u32)) | (((1 as i32)).wrapping_shl(((1 as i32)) as u32)))) | (((1 as i32)).wrapping_shl(((2 as i32)) as u32)))) | (((1 as i32)).wrapping_shl(((14 as i32)) as u32))))) < ((0 as i32))) as i32)) != 0 { 30 } else { 29 }; continue;
}
// C line 48755
32 => {
vm_block = 11; continue;
}
// C line 48754
33 => {
vm_block = if ((((JS_DefinePropertyValueInt64(ctx, tab, (({ let old = n; n = (n).wrapping_add(1); old }) as i64), JS_NewInt32(ctx, ((position) as i32)), ((((((((1 as i32)).wrapping_shl(((0 as i32)) as u32)) | (((1 as i32)).wrapping_shl(((1 as i32)) as u32)))) | (((1 as i32)).wrapping_shl(((2 as i32)) as u32)))) | (((1 as i32)).wrapping_shl(((14 as i32)) as u32))))) < ((0 as i32))) as i32)) != 0 { 32 } else { 31 }; continue;
}
// C line 48787
34 => {
vm_block = 11; continue;
}
// C line 48786
35 => {
vm_block = if (ret) != 0 { 34 } else { 22 }; continue;
}
// C line 48785
36 => {
let _ = JS_FreeValue(ctx, namedCaptures1);
vm_block = 35; continue;
}
// C line 48784
37 => {
let _ = { let assigned = string_buffer_end(b1); rep_str = assigned; assigned };
vm_block = 36; continue;
}
// C line 48781
38 => {
let _ = { let assigned = js_string_GetSubstitution(ctx, b1, matched, sp, ((position) as u32), tab, namedCaptures1, rep_val, core::ptr::null_mut::<*mut u8>(), (((0 as i32)) as u32)); ret = assigned; assigned };
vm_block = 37; continue;
}
// C line 48780
39 => {
let _ = string_buffer_init(ctx, b1, (0 as i32));
vm_block = 38; continue;
}
// C line 48778
40 => {
let _ = JS_FreeValue(ctx, rep_str);
vm_block = 39; continue;
}
// C line 48774
41 => {
vm_block = 11; continue;
}
// C line 48773
42 => {
vm_block = if (JS_IsException(namedCaptures1)) != 0 { 41 } else { 40 }; continue;
}
// C line 48772
43 => {
let _ = { let assigned = JS_ToObject(ctx, namedCaptures); namedCaptures1 = assigned; assigned };
vm_block = 42; continue;
}
// C line 48776
44 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; namedCaptures1 = assigned; assigned };
vm_block = 40; continue;
}
// C line 48771
45 => {
vm_block = if ((!((JS_IsUndefined(namedCaptures)) != 0) as i32)) != 0 { 43 } else { 44 }; continue;
}
// C line 48768
46 => {
b1 = core::ptr::addr_of_mut!(b1_s);
vm_block = 45; continue;
}
// C line 48753
47 => {
vm_block = if (functionalReplace) != 0 { 33 } else { 46 }; continue;
}
// C line 48752
48 => {
vm_block = 11; continue;
}
// C line 48751
49 => {
vm_block = if (JS_IsException(namedCaptures)) != 0 { 48 } else { 47 }; continue;
}
// C line 48750
50 => {
let _ = { let assigned = JS_GetProperty(ctx, result_1, (((crate::quickjs_atom::JS_ATOM_groups as i32)) as JSAtom)); namedCaptures = assigned; assigned };
vm_block = 49; continue;
}
// C line 48749
51 => {
let _ = JS_FreeValue(ctx, namedCaptures);
vm_block = 50; continue;
}
// C line 48735
52 => {
vm_block = if ((((((n) as u32)) < (nCaptures)) as i32)) != 0 { 62 } else { 51 }; continue;
}
// C line ?
53 => {
let _ = { let old = n; n = (n).wrapping_add(1); old };
vm_block = 52; continue;
}
// C line 48747
54 => {
vm_block = 11; continue;
}
// C line 48745
55 => {
vm_block = if ((((JS_DefinePropertyValueInt64(ctx, tab, ((n) as i64), capN, ((((((((1 as i32)).wrapping_shl(((0 as i32)) as u32)) | (((1 as i32)).wrapping_shl(((1 as i32)) as u32)))) | (((1 as i32)).wrapping_shl(((2 as i32)) as u32)))) | (((1 as i32)).wrapping_shl(((14 as i32)) as u32))))) < ((0 as i32))) as i32)) != 0 { 54 } else { 53 }; continue;
}
// C line 48743
56 => {
vm_block = 11; continue;
}
// C line 48742
57 => {
vm_block = if (JS_IsException(capN)) != 0 { 56 } else { 55 }; continue;
}
// C line 48741
58 => {
let _ = { let assigned = JS_ToStringFree(ctx, capN); capN = assigned; assigned };
vm_block = 57; continue;
}
// C line 48740
59 => {
vm_block = if ((!((JS_IsUndefined(capN)) != 0) as i32)) != 0 { 58 } else { 55 }; continue;
}
// C line 48739
60 => {
vm_block = 11; continue;
}
// C line 48738
61 => {
vm_block = if (JS_IsException(capN)) != 0 { 60 } else { 59 }; continue;
}
// C line 48737
62 => {
let _ = { let assigned = JS_GetPropertyInt64(ctx, result_1, ((n) as i64)); capN = assigned; assigned };
vm_block = 61; continue;
}
// C line 48735
63 => {
let _ = { let assigned = (1 as i32); n = assigned; assigned };
vm_block = 52; continue;
}
// C line 48734
64 => {
vm_block = 11; continue;
}
// C line 48732
65 => {
vm_block = if ((((JS_DefinePropertyValueInt64(ctx, tab, (((0 as i32)) as i64), JS_DupValue(ctx, matched), ((((((((1 as i32)).wrapping_shl(((0 as i32)) as u32)) | (((1 as i32)).wrapping_shl(((1 as i32)) as u32)))) | (((1 as i32)).wrapping_shl(((2 as i32)) as u32)))) | (((1 as i32)).wrapping_shl(((14 as i32)) as u32))))) < ((0 as i32))) as i32)) != 0 { 64 } else { 63 }; continue;
}
// C line 48731
66 => {
vm_block = 11; continue;
}
// C line 48730
67 => {
vm_block = if (JS_IsException(tab)) != 0 { 66 } else { 65 }; continue;
}
// C line 48729
68 => {
let _ = { let assigned = JS_NewArray(ctx); tab = assigned; assigned };
vm_block = 67; continue;
}
// C line 48728
69 => {
let _ = JS_FreeValue(ctx, tab);
vm_block = 68; continue;
}
// C line 48723
70 => {
let _ = { let assigned = (((*(sp)).len()) as i64); position = assigned; assigned };
vm_block = 69; continue;
}
// C line 48725
71 => {
let _ = { let assigned = (((0 as i32)) as i64); position = assigned; assigned };
vm_block = 69; continue;
}
// C line 48724
72 => {
vm_block = if ((((position) < ((((0 as i32)) as i64))) as i32)) != 0 { 71 } else { 69 }; continue;
}
// C line 48722
73 => {
vm_block = if ((((position) > ((((*(sp)).len()) as i64))) as i32)) != 0 { 70 } else { 72 }; continue;
}
// C line 48721
74 => {
vm_block = 11; continue;
}
// C line 48720
75 => {
vm_block = if (JS_ToLengthFree(ctx, core::ptr::addr_of_mut!(position), JS_GetProperty(ctx, result_1, (((crate::quickjs_atom::JS_ATOM_index as i32)) as JSAtom)))) != 0 { 74 } else { 73 }; continue;
}
// C line 48719
76 => {
vm_block = 11; continue;
}
// C line 48718
77 => {
vm_block = if (JS_IsException(matched)) != 0 { 76 } else { 75 }; continue;
}
// C line 48717
78 => {
let _ = { let assigned = JS_ToStringFree(ctx, JS_GetPropertyInt64(ctx, result_1, (((0 as i32)) as i64))); matched = assigned; assigned };
vm_block = 77; continue;
}
// C line 48716
79 => {
let _ = JS_FreeValue(ctx, matched);
vm_block = 78; continue;
}
// C line 48715
80 => {
vm_block = 11; continue;
}
// C line 48714
81 => {
vm_block = if ((((js_get_length32(ctx, core::ptr::addr_of_mut!(nCaptures), result_1)) < ((0 as i32))) as i32)) != 0 { 80 } else { 79 }; continue;
}
// C line 48713
82 => {
let _ = { let assigned = *((*(results)).arr).offset((j) as isize); result_1 = assigned; assigned };
vm_block = 81; continue;
}
// C line 48711
83 => {
let _ = { let assigned = (0 as i32); j = assigned; assigned };
vm_block = 15; continue;
}
// C line 48710
84 => {
let _ = { let assigned = (0 as i32); nextSourcePosition = assigned; assigned };
vm_block = 83; continue;
}
// C line 48685
85 => {
vm_block = 104; continue;
}
// C line 48707
86 => {
vm_block = 11; continue;
}
// C line 48706
87 => {
vm_block = if ((((JS_SetProperty(ctx, rx, (((crate::quickjs_atom::JS_ATOM_lastIndex as i32)) as JSAtom), JS_NewInt64(ctx, nextIndex))) < ((0 as i32))) as i32)) != 0 { 86 } else { 85 }; continue;
}
// C line 48705
88 => {
let _ = { let assigned = string_advance_index(sp, thisIndex, fullUnicode); nextIndex = assigned; assigned };
vm_block = 87; continue;
}
// C line 48704
89 => {
vm_block = 11; continue;
}
// C line 48703
90 => {
vm_block = if ((((JS_ToLengthFree(ctx, core::ptr::addr_of_mut!(thisIndex), JS_GetProperty(ctx, rx, (((crate::quickjs_atom::JS_ATOM_lastIndex as i32)) as JSAtom)))) < ((0 as i32))) as i32)) != 0 { 89 } else { 88 }; continue;
}
// C line 48700
91 => {
vm_block = if (JS_IsEmptyString(matched)) != 0 { 90 } else { 85 }; continue;
}
// C line 48699
92 => {
vm_block = 11; continue;
}
// C line 48698
93 => {
vm_block = if (JS_IsException(matched)) != 0 { 92 } else { 91 }; continue;
}
// C line 48697
94 => {
let _ = { let assigned = JS_ToStringFree(ctx, JS_GetPropertyInt64(ctx, result, (((0 as i32)) as i64))); matched = assigned; assigned };
vm_block = 93; continue;
}
// C line 48696
95 => {
let _ = JS_FreeValue(ctx, matched);
vm_block = 94; continue;
}
// C line 48695
96 => {
vm_block = 84; continue;
}
// C line 48694
97 => {
vm_block = if ((!((is_global) != 0) as i32)) != 0 { 96 } else { 95 }; continue;
}
// C line 48693
98 => {
vm_block = 11; continue;
}
// C line 48692
99 => {
vm_block = if ((((value_buffer_append(results, result)) < ((0 as i32))) as i32)) != 0 { 98 } else { 97 }; continue;
}
// C line 48691
100 => {
vm_block = 84; continue;
}
// C line 48690
101 => {
vm_block = if (JS_IsNull(result)) != 0 { 100 } else { 99 }; continue;
}
// C line 48689
102 => {
vm_block = 11; continue;
}
// C line 48688
103 => {
vm_block = if (JS_IsException(result)) != 0 { 102 } else { 101 }; continue;
}
// C line 48687
104 => {
let _ = { let assigned = JS_RegExpExec(ctx, rx, str); result = assigned; assigned };
vm_block = 103; continue;
}
// C line 48682
105 => {
vm_block = 11; continue;
}
// C line 48681
106 => {
vm_block = if ((((JS_SetProperty(ctx, rx, (((crate::quickjs_atom::JS_ATOM_lastIndex as i32)) as JSAtom), JS_NewInt32(ctx, (0 as i32)))) < ((0 as i32))) as i32)) != 0 { 105 } else { 85 }; continue;
}
// C line 48679
107 => {
let _ = { let assigned = (((((((string_indexof_char(p, (117 as i32), (0 as i32))) >= ((0 as i32))) as i32)) != 0) || (((((string_indexof_char(p, (118 as i32), (0 as i32))) >= ((0 as i32))) as i32)) != 0)) as i32); fullUnicode = assigned; assigned };
vm_block = 106; continue;
}
// C line 48678
108 => {
vm_block = if (is_global) != 0 { 107 } else { 85 }; continue;
}
// C line 48677
109 => {
let _ = { let assigned = (((((1 as i32)).wrapping_neg()) != (string_indexof_char(p, (103 as i32), (0 as i32)))) as i32); is_global = assigned; assigned };
vm_block = 108; continue;
}
// C line 48676
110 => {
let _ = { let assigned = (0 as i32); fullUnicode = assigned; assigned };
vm_block = 109; continue;
}
// C line 48674
111 => {
let _ = { let assigned = ((((flags).u).ptr) as *mut JSString); p = assigned; assigned };
vm_block = 110; continue;
}
// C line 48673
112 => {
vm_block = 11; continue;
}
// C line 48672
113 => {
vm_block = if (JS_IsException(flags)) != 0 { 112 } else { 111 }; continue;
}
// C line 48671
114 => {
let _ = { let assigned = JS_ToStringFree(ctx, flags); flags = assigned; assigned };
vm_block = 113; continue;
}
// C line 48670
115 => {
vm_block = 11; continue;
}
// C line 48669
116 => {
vm_block = if (JS_IsException(flags)) != 0 { 115 } else { 114 }; continue;
}
// C line 48668
117 => {
let _ = { let assigned = JS_GetProperty(ctx, rx, (((crate::quickjs_atom::JS_ATOM_flags as i32)) as JSAtom)); flags = assigned; assigned };
vm_block = 116; continue;
}
// C line 48665
118 => {
vm_block = 10; continue;
}
// C line 48664
119 => {
vm_block = if ((!((JS_IsUndefined(res)) != 0) as i32)) != 0 { 118 } else { 117 }; continue;
}
// C line 48663
120 => {
let _ = { let assigned = js_regexp_replace(ctx, rx, str, rep_val); res = assigned; assigned };
vm_block = 119; continue;
}
// C line 48661
121 => {
vm_block = if ((((((!((functionalReplace) != 0) as i32)) != 0) && ((js_is_standard_regexp(ctx, rx)) != 0)) as i32)) != 0 { 120 } else { 117 }; continue;
}
// C line 48658
122 => {
vm_block = 11; continue;
}
// C line 48657
123 => {
vm_block = if (JS_IsException(rep_val)) != 0 { 122 } else { 121 }; continue;
}
// C line 48656
124 => {
let _ = { let assigned = JS_ToString(ctx, rep); rep_val = assigned; assigned };
vm_block = 123; continue;
}
// C line 48655
125 => {
vm_block = if ((!((functionalReplace) != 0) as i32)) != 0 { 124 } else { 121 }; continue;
}
// C line 48654
126 => {
let _ = { let assigned = JS_IsFunction(ctx, rep); functionalReplace = assigned; assigned };
vm_block = 125; continue;
}
// C line 48653
127 => {
let _ = { let assigned = ((((str).u).ptr) as *mut JSString); sp = assigned; assigned };
vm_block = 126; continue;
}
// C line 48651
128 => {
vm_block = 11; continue;
}
// C line 48650
129 => {
vm_block = if (JS_IsException(str)) != 0 { 128 } else { 127 }; continue;
}
// C line 48649
130 => {
let _ = { let assigned = JS_ToString(ctx, *(argv).offset(((0 as i32)) as isize)); str = assigned; assigned };
vm_block = 129; continue;
}
// C line 48647
131 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; namedCaptures = assigned; assigned };
vm_block = 130; continue;
}
// C line 48646
132 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; rep_str = assigned; assigned };
vm_block = 131; continue;
}
// C line 48645
133 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; flags = assigned; assigned };
vm_block = 132; continue;
}
// C line 48644
134 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; tab = assigned; assigned };
vm_block = 133; continue;
}
// C line 48643
135 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; matched = assigned; assigned };
vm_block = 134; continue;
}
// C line 48642
136 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; rep_val = assigned; assigned };
vm_block = 135; continue;
}
// C line 48640
137 => {
let _ = value_buffer_init(ctx, results);
vm_block = 136; continue;
}
// C line 48639
138 => {
let _ = string_buffer_init(ctx, b, (0 as i32));
vm_block = 137; continue;
}
// C line 48637
139 => {
return JS_ThrowTypeErrorNotAnObject(ctx);
}
// C line 48636
140 => {
vm_block = if ((!((JS_IsObject(rx)) != 0) as i32)) != 0 { 139 } else { 138 }; continue;
}
// C line 48631
141 => {
results = core::ptr::addr_of_mut!(v_b);
vm_block = 140; continue;
}
// C line 48630
142 => {
b = core::ptr::addr_of_mut!(b_s);
vm_block = 141; continue;
}
// C line 48626
143 => {
rx = this_val;
rep = *(argv).offset(((1 as i32)) as isize);
vm_block = 142; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:48817. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code, unused_variables)]
unsafe fn js_regexp_Symbol_search(mut ctx: *mut JSContext, mut this_val: JSValue, mut argc: i32, mut argv: *mut JSValue) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut rx: JSValue = core::mem::zeroed();
let mut str: JSValue = core::mem::zeroed();
let mut previousLastIndex: JSValue = core::mem::zeroed();
let mut currentLastIndex: JSValue = core::mem::zeroed();
let mut result: JSValue = core::mem::zeroed();
let mut index: JSValue = core::mem::zeroed();
let mut vm_block: usize = 38;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 48872
1 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 48871
2 => {
let _ = JS_FreeValue(ctx, previousLastIndex);
vm_block = 1; continue;
}
// C line 48870
3 => {
let _ = JS_FreeValue(ctx, currentLastIndex);
vm_block = 2; continue;
}
// C line 48869
4 => {
let _ = JS_FreeValue(ctx, str);
vm_block = 3; continue;
}
// C line ? labels: exception
5 => {
let _ = JS_FreeValue(ctx, result);
vm_block = 4; continue;
}
// C line 48860
6 => {
return JS_NewInt32(ctx, ((1 as i32)).wrapping_neg());
}
// C line 48864
7 => {
return index;
}
// C line 48863
8 => {
let _ = JS_FreeValue(ctx, result);
vm_block = 7; continue;
}
// C line 48862
9 => {
let _ = { let assigned = JS_GetProperty(ctx, result, (((crate::quickjs_atom::JS_ATOM_index as i32)) as JSAtom)); index = assigned; assigned };
vm_block = 8; continue;
}
// C line 48859
10 => {
vm_block = if (JS_IsNull(result)) != 0 { 6 } else { 9 }; continue;
}
// C line 48857
11 => {
let _ = JS_FreeValue(ctx, currentLastIndex);
vm_block = 10; continue;
}
// C line 48856
12 => {
let _ = JS_FreeValue(ctx, str);
vm_block = 11; continue;
}
// C line 48849
13 => {
let _ = JS_FreeValue(ctx, previousLastIndex);
vm_block = 12; continue;
}
// C line 48853
14 => {
vm_block = 5; continue;
}
// C line 48852
15 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; previousLastIndex = assigned; assigned };
vm_block = 14; continue;
}
// C line 48851
16 => {
vm_block = if ((((JS_SetProperty(ctx, rx, (((crate::quickjs_atom::JS_ATOM_lastIndex as i32)) as JSAtom), previousLastIndex)) < ((0 as i32))) as i32)) != 0 { 15 } else { 12 }; continue;
}
// C line 48848
17 => {
vm_block = if (js_same_value(ctx, currentLastIndex, previousLastIndex)) != 0 { 13 } else { 16 }; continue;
}
// C line 48847
18 => {
vm_block = 5; continue;
}
// C line 48846
19 => {
vm_block = if (JS_IsException(currentLastIndex)) != 0 { 18 } else { 17 }; continue;
}
// C line 48845
20 => {
let _ = { let assigned = JS_GetProperty(ctx, rx, (((crate::quickjs_atom::JS_ATOM_lastIndex as i32)) as JSAtom)); currentLastIndex = assigned; assigned };
vm_block = 19; continue;
}
// C line 48844
21 => {
vm_block = 5; continue;
}
// C line 48843
22 => {
vm_block = if (JS_IsException(result)) != 0 { 21 } else { 20 }; continue;
}
// C line 48842
23 => {
let _ = { let assigned = JS_RegExpExec(ctx, rx, str); result = assigned; assigned };
vm_block = 22; continue;
}
// C line 48839
24 => {
vm_block = 5; continue;
}
// C line 48838
25 => {
vm_block = if ((((JS_SetProperty(ctx, rx, (((crate::quickjs_atom::JS_ATOM_lastIndex as i32)) as JSAtom), JS_NewInt32(ctx, (0 as i32)))) < ((0 as i32))) as i32)) != 0 { 24 } else { 23 }; continue;
}
// C line 48837
26 => {
vm_block = if ((!((js_same_value(ctx, previousLastIndex, JS_NewInt32(ctx, (0 as i32)))) != 0) as i32)) != 0 { 25 } else { 23 }; continue;
}
// C line 48835
27 => {
vm_block = 5; continue;
}
// C line 48834
28 => {
vm_block = if (JS_IsException(previousLastIndex)) != 0 { 27 } else { 26 }; continue;
}
// C line 48833
29 => {
let _ = { let assigned = JS_GetProperty(ctx, rx, (((crate::quickjs_atom::JS_ATOM_lastIndex as i32)) as JSAtom)); previousLastIndex = assigned; assigned };
vm_block = 28; continue;
}
// C line 48831
30 => {
vm_block = 5; continue;
}
// C line 48830
31 => {
vm_block = if (JS_IsException(str)) != 0 { 30 } else { 29 }; continue;
}
// C line 48829
32 => {
let _ = { let assigned = JS_ToString(ctx, *(argv).offset(((0 as i32)) as isize)); str = assigned; assigned };
vm_block = 31; continue;
}
// C line 48828
33 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; previousLastIndex = assigned; assigned };
vm_block = 32; continue;
}
// C line 48827
34 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; currentLastIndex = assigned; assigned };
vm_block = 33; continue;
}
// C line 48826
35 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; result = assigned; assigned };
vm_block = 34; continue;
}
// C line 48824
36 => {
return JS_ThrowTypeErrorNotAnObject(ctx);
}
// C line 48823
37 => {
vm_block = if ((!((JS_IsObject(rx)) != 0) as i32)) != 0 { 36 } else { 35 }; continue;
}
// C line 48820
38 => {
rx = this_val;
vm_block = 37; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:48875. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code, unused_variables)]
unsafe fn js_regexp_Symbol_split(mut ctx: *mut JSContext, mut this_val: JSValue, mut argc: i32, mut argv: *mut JSValue) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut rx: JSValue = core::mem::zeroed();
let mut args: [JSValue; 2] = core::mem::zeroed();
let mut str: JSValue = core::mem::zeroed();
let mut ctor: JSValue = core::mem::zeroed();
let mut splitter: JSValue = core::mem::zeroed();
let mut A: JSValue = core::mem::zeroed();
let mut flags: JSValue = core::mem::zeroed();
let mut z: JSValue = core::mem::zeroed();
let mut sub: JSValue = core::mem::zeroed();
let mut strp: *mut JSString = core::mem::zeroed();
let mut lim: u32 = core::mem::zeroed();
let mut size: u32 = core::mem::zeroed();
let mut p: u32 = core::mem::zeroed();
let mut q: u32 = core::mem::zeroed();
let mut unicodeMatching: i32 = core::mem::zeroed();
let mut lengthA: i64 = core::mem::zeroed();
let mut e: i64 = core::mem::zeroed();
let mut numberOfCaptures: i64 = core::mem::zeroed();
let mut i: i64 = core::mem::zeroed();
let mut vm_block: usize = 100;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 48999
1 => {
return A;
}
// C line 48998
2 => {
let _ = JS_FreeValue(ctx, z);
vm_block = 1; continue;
}
// C line 48997
3 => {
let _ = JS_FreeValue(ctx, flags);
vm_block = 2; continue;
}
// C line 48996
4 => {
let _ = JS_FreeValue(ctx, splitter);
vm_block = 3; continue;
}
// C line 48995
5 => {
let _ = JS_FreeValue(ctx, ctor);
vm_block = 4; continue;
}
// C line ? labels: done
6 => {
let _ = JS_FreeValue(ctx, str);
vm_block = 5; continue;
}
// C line 48992
7 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) }; A = assigned; assigned };
vm_block = 6; continue;
}
// C line ? labels: exception
8 => {
let _ = JS_FreeValue(ctx, A);
vm_block = 7; continue;
}
// C line 48989
9 => {
vm_block = 6; continue;
}
// C line 48988
10 => {
vm_block = 8; continue;
}
// C line 48987
11 => {
vm_block = if ((((JS_DefinePropertyValueInt64(ctx, A, { let old = lengthA; lengthA = (lengthA).wrapping_add(1); old }, sub, ((((((((1 as i32)).wrapping_shl(((0 as i32)) as u32)) | (((1 as i32)).wrapping_shl(((1 as i32)) as u32)))) | (((1 as i32)).wrapping_shl(((2 as i32)) as u32)))) | (((1 as i32)).wrapping_shl(((14 as i32)) as u32))))) < ((0 as i32))) as i32)) != 0 { 10 } else { 9 }; continue;
}
// C line 48986
12 => {
vm_block = 8; continue;
}
// C line 48985
13 => {
vm_block = if (JS_IsException(sub)) != 0 { 12 } else { 11 }; continue;
}
// C line 48984
14 => {
let _ = { let assigned = js_sub_string(ctx, strp, ((p) as i32), ((size) as i32)); sub = assigned; assigned };
vm_block = 13; continue;
}
// C line 48983
15 => {
let _ = { let assigned = size; p = assigned; assigned };
vm_block = 14; continue;
}
// C line 48982 labels: add_tail
16 => {
vm_block = if ((((p) > (size)) as i32)) != 0 { 15 } else { 14 }; continue;
}
// C line 48940
17 => {
vm_block = if ((((q) < (size)) as i32)) != 0 { 52 } else { 16 }; continue;
}
// C line 48948
18 => {
let _ = { let assigned = ((string_advance_index(strp, ((q) as i64), unicodeMatching)) as u32); q = assigned; assigned };
vm_block = 17; continue;
}
// C line 48955
19 => {
let _ = { let assigned = ((string_advance_index(strp, ((q) as i64), unicodeMatching)) as u32); q = assigned; assigned };
vm_block = 17; continue;
}
// C line 48977
20 => {
let _ = { let assigned = p; q = assigned; assigned };
vm_block = 17; continue;
}
// C line 48968
21 => {
vm_block = if ((((i) < (numberOfCaptures)) as i32)) != 0 { 29 } else { 20 }; continue;
}
// C line ?
22 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 21; continue;
}
// C line 48975
23 => {
vm_block = 6; continue;
}
// C line 48974
24 => {
vm_block = if ((((lengthA) == (((lim) as i64))) as i32)) != 0 { 23 } else { 22 }; continue;
}
// C line 48973
25 => {
vm_block = 8; continue;
}
// C line 48972
26 => {
vm_block = if ((((JS_DefinePropertyValueInt64(ctx, A, { let old = lengthA; lengthA = (lengthA).wrapping_add(1); old }, sub, ((((((((1 as i32)).wrapping_shl(((0 as i32)) as u32)) | (((1 as i32)).wrapping_shl(((1 as i32)) as u32)))) | (((1 as i32)).wrapping_shl(((2 as i32)) as u32)))) | (((1 as i32)).wrapping_shl(((14 as i32)) as u32))))) < ((0 as i32))) as i32)) != 0 { 25 } else { 24 }; continue;
}
// C line 48971
27 => {
vm_block = 8; continue;
}
// C line 48970
28 => {
vm_block = if (JS_IsException(sub)) != 0 { 27 } else { 26 }; continue;
}
// C line 48969
29 => {
let _ = { let assigned = JS_GetPropertyInt64(ctx, z, i); sub = assigned; assigned };
vm_block = 28; continue;
}
// C line 48968
30 => {
let _ = { let assigned = (((1 as i32)) as i64); i = assigned; assigned };
vm_block = 21; continue;
}
// C line 48967
31 => {
vm_block = 8; continue;
}
// C line 48966
32 => {
vm_block = if (js_get_length64(ctx, core::ptr::addr_of_mut!(numberOfCaptures), z)) != 0 { 31 } else { 30 }; continue;
}
// C line 48965
33 => {
let _ = { let assigned = ((e) as u32); p = assigned; assigned };
vm_block = 32; continue;
}
// C line 48964
34 => {
vm_block = 6; continue;
}
// C line 48963
35 => {
vm_block = if ((((lengthA) == (((lim) as i64))) as i32)) != 0 { 34 } else { 33 }; continue;
}
// C line 48962
36 => {
vm_block = 8; continue;
}
// C line 48960
37 => {
vm_block = if ((((JS_DefinePropertyValueInt64(ctx, A, { let old = lengthA; lengthA = (lengthA).wrapping_add(1); old }, sub, ((((((((1 as i32)).wrapping_shl(((0 as i32)) as u32)) | (((1 as i32)).wrapping_shl(((1 as i32)) as u32)))) | (((1 as i32)).wrapping_shl(((2 as i32)) as u32)))) | (((1 as i32)).wrapping_shl(((14 as i32)) as u32))))) < ((0 as i32))) as i32)) != 0 { 36 } else { 35 }; continue;
}
// C line 48959
38 => {
vm_block = 8; continue;
}
// C line 48958
39 => {
vm_block = if (JS_IsException(sub)) != 0 { 38 } else { 37 }; continue;
}
// C line 48957
40 => {
let _ = { let assigned = js_sub_string(ctx, strp, ((p) as i32), ((q) as i32)); sub = assigned; assigned };
vm_block = 39; continue;
}
// C line 48954
41 => {
vm_block = if ((((e) == (((p) as i64))) as i32)) != 0 { 19 } else { 40 }; continue;
}
// C line 48953
42 => {
let _ = { let assigned = ((size) as i64); e = assigned; assigned };
vm_block = 41; continue;
}
// C line 48952
43 => {
vm_block = if ((((e) > (((size) as i64))) as i32)) != 0 { 42 } else { 41 }; continue;
}
// C line 48951
44 => {
vm_block = 8; continue;
}
// C line 48950
45 => {
vm_block = if (JS_ToLengthFree(ctx, core::ptr::addr_of_mut!(e), JS_GetProperty(ctx, splitter, (((crate::quickjs_atom::JS_ATOM_lastIndex as i32)) as JSAtom)))) != 0 { 44 } else { 43 }; continue;
}
// C line 48947
46 => {
vm_block = if (JS_IsNull(z)) != 0 { 18 } else { 45 }; continue;
}
// C line 48946
47 => {
vm_block = 8; continue;
}
// C line 48945
48 => {
vm_block = if (JS_IsException(z)) != 0 { 47 } else { 46 }; continue;
}
// C line 48944
49 => {
let _ = { let assigned = JS_RegExpExec(ctx, splitter, str); z = assigned; assigned };
vm_block = 48; continue;
}
// C line 48943
50 => {
let _ = JS_FreeValue(ctx, z);
vm_block = 49; continue;
}
// C line 48942
51 => {
vm_block = 8; continue;
}
// C line 48941
52 => {
vm_block = if ((((JS_SetProperty(ctx, splitter, (((crate::quickjs_atom::JS_ATOM_lastIndex as i32)) as JSAtom), JS_NewInt32(ctx, ((q) as i32)))) < ((0 as i32))) as i32)) != 0 { 51 } else { 50 }; continue;
}
// C line 48938
53 => {
vm_block = 6; continue;
}
// C line 48937
54 => {
vm_block = 16; continue;
}
// C line 48936
55 => {
vm_block = if (JS_IsNull(z)) != 0 { 54 } else { 53 }; continue;
}
// C line 48935
56 => {
vm_block = 8; continue;
}
// C line 48934
57 => {
vm_block = if (JS_IsException(z)) != 0 { 56 } else { 55 }; continue;
}
// C line 48933
58 => {
let _ = { let assigned = JS_RegExpExec(ctx, splitter, str); z = assigned; assigned };
vm_block = 57; continue;
}
// C line 48932
59 => {
vm_block = if ((((size) == ((((0 as i32)) as u32))) as i32)) != 0 { 58 } else { 17 }; continue;
}
// C line 48931
60 => {
let _ = { let assigned = (*(strp)).len(); size = assigned; assigned };
vm_block = 59; continue;
}
// C line 48930
61 => {
let _ = { let assigned = { let assigned = (((0 as i32)) as u32); q = assigned; assigned }; p = assigned; assigned };
vm_block = 60; continue;
}
// C line 48929
62 => {
let _ = { let assigned = ((((str).u).ptr) as *mut JSString); strp = assigned; assigned };
vm_block = 61; continue;
}
// C line 48922
63 => {
let _ = { let assigned = (4294967295 as u32); lim = assigned; assigned };
vm_block = 62; continue;
}
// C line 48927
64 => {
vm_block = 6; continue;
}
// C line 48926
65 => {
vm_block = if ((((lim) == ((((0 as i32)) as u32))) as i32)) != 0 { 64 } else { 62 }; continue;
}
// C line 48925
66 => {
vm_block = 8; continue;
}
// C line 48924
67 => {
vm_block = if ((((JS_ToUint32(ctx, core::ptr::addr_of_mut!(lim), *(argv).offset(((1 as i32)) as isize))) < ((0 as i32))) as i32)) != 0 { 66 } else { 65 }; continue;
}
// C line 48921
68 => {
vm_block = if (JS_IsUndefined(*(argv).offset(((1 as i32)) as isize))) != 0 { 63 } else { 67 }; continue;
}
// C line 48920
69 => {
let _ = { let assigned = (((0 as i32)) as i64); lengthA = assigned; assigned };
vm_block = 68; continue;
}
// C line 48919
70 => {
vm_block = 8; continue;
}
// C line 48918
71 => {
vm_block = if (JS_IsException(A)) != 0 { 70 } else { 69 }; continue;
}
// C line 48917
72 => {
let _ = { let assigned = JS_NewArray(ctx); A = assigned; assigned };
vm_block = 71; continue;
}
// C line 48916
73 => {
vm_block = 8; continue;
}
// C line 48915
74 => {
vm_block = if (JS_IsException(splitter)) != 0 { 73 } else { 72 }; continue;
}
// C line 48914
75 => {
let _ = { let assigned = JS_CallConstructor(ctx, ctor, (2 as i32), (args).as_mut_ptr()); splitter = assigned; assigned };
vm_block = 74; continue;
}
// C line 48913
76 => {
let _ = { let assigned = flags; *((args).as_mut_ptr()).offset(((1 as i32)) as isize) = assigned; assigned };
vm_block = 75; continue;
}
// C line 48912
77 => {
let _ = { let assigned = rx; *((args).as_mut_ptr()).offset(((0 as i32)) as isize) = assigned; assigned };
vm_block = 76; continue;
}
// C line 48910
78 => {
vm_block = 8; continue;
}
// C line 48909
79 => {
vm_block = if (JS_IsException(flags)) != 0 { 78 } else { 77 }; continue;
}
// C line 48908
80 => {
let _ = { let assigned = JS_ConcatString3(ctx, c"".as_ptr(), flags, c"y".as_ptr()); flags = assigned; assigned };
vm_block = 79; continue;
}
// C line 48907
81 => {
vm_block = if ((((string_indexof_char(strp, (121 as i32), (0 as i32))) < ((0 as i32))) as i32)) != 0 { 80 } else { 77 }; continue;
}
// C line 48905
82 => {
let _ = { let assigned = (((((((string_indexof_char(strp, (117 as i32), (0 as i32))) >= ((0 as i32))) as i32)) != 0) || (((((string_indexof_char(strp, (118 as i32), (0 as i32))) >= ((0 as i32))) as i32)) != 0)) as i32); unicodeMatching = assigned; assigned };
vm_block = 81; continue;
}
// C line 48904
83 => {
let _ = { let assigned = ((((flags).u).ptr) as *mut JSString); strp = assigned; assigned };
vm_block = 82; continue;
}
// C line 48903
84 => {
vm_block = 8; continue;
}
// C line 48902
85 => {
vm_block = if (JS_IsException(flags)) != 0 { 84 } else { 83 }; continue;
}
// C line 48901
86 => {
let _ = { let assigned = JS_ToStringFree(ctx, JS_GetProperty(ctx, rx, (((crate::quickjs_atom::JS_ATOM_flags as i32)) as JSAtom))); flags = assigned; assigned };
vm_block = 85; continue;
}
// C line 48900
87 => {
vm_block = 8; continue;
}
// C line 48899
88 => {
vm_block = if (JS_IsException(ctor)) != 0 { 87 } else { 86 }; continue;
}
// C line 48898
89 => {
let _ = { let assigned = JS_SpeciesConstructor(ctx, rx, (*(ctx)).regexp_ctor); ctor = assigned; assigned };
vm_block = 88; continue;
}
// C line 48897
90 => {
vm_block = 8; continue;
}
// C line 48896
91 => {
vm_block = if (JS_IsException(str)) != 0 { 90 } else { 89 }; continue;
}
// C line 48895
92 => {
let _ = { let assigned = JS_ToString(ctx, *(argv).offset(((0 as i32)) as isize)); str = assigned; assigned };
vm_block = 91; continue;
}
// C line 48894
93 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; z = assigned; assigned };
vm_block = 92; continue;
}
// C line 48893
94 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; flags = assigned; assigned };
vm_block = 93; continue;
}
// C line 48892
95 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; A = assigned; assigned };
vm_block = 94; continue;
}
// C line 48891
96 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; splitter = assigned; assigned };
vm_block = 95; continue;
}
// C line 48890
97 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; ctor = assigned; assigned };
vm_block = 96; continue;
}
// C line 48888
98 => {
return JS_ThrowTypeErrorNotAnObject(ctx);
}
// C line 48887
99 => {
vm_block = if ((!((JS_IsObject(rx)) != 0) as i32)) != 0 { 98 } else { 97 }; continue;
}
// C line 48879
100 => {
rx = this_val;
vm_block = 99; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:49034. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code, unused_variables)]
pub unsafe fn JS_AddIntrinsicRegExpCompiler(mut ctx: *mut JSContext) -> () {
let mut vm_local_storage = Vec::<u64>::new();
let mut vm_block: usize = 1;
loop { match vm_block {
// C line ?
0 => {
return ();
}
// C line 49036
1 => {
let _ = { let assigned = js_compile_regexp; (*(ctx)).compile_regexp = Some(assigned); assigned };
vm_block = 0; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:49039. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code, unused_variables)]
pub unsafe fn JS_AddIntrinsicRegExp(mut ctx: *mut JSContext) -> i32 {
let mut vm_local_storage = Vec::<u64>::new();
let mut obj: JSValue = core::mem::zeroed();
let mut vm_block: usize = 25;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 49087
1 => {
return (0 as i32);
}
// C line 49085
2 => {
return ((1 as i32)).wrapping_neg();
}
// C line 49083
3 => {
vm_block = if (add_shape_property(ctx, core::ptr::addr_of_mut!((*(ctx)).regexp_result_shape), core::ptr::null_mut::<JSObject>(), (((crate::quickjs_atom::JS_ATOM_groups as i32)) as JSAtom), ((((((1 as i32)).wrapping_shl(((0 as i32)) as u32)) | (((1 as i32)).wrapping_shl(((1 as i32)) as u32)))) | (((1 as i32)).wrapping_shl(((2 as i32)) as u32))))) != 0 { 2 } else { 1 }; continue;
}
// C line 49082
4 => {
return ((1 as i32)).wrapping_neg();
}
// C line 49080
5 => {
vm_block = if (add_shape_property(ctx, core::ptr::addr_of_mut!((*(ctx)).regexp_result_shape), core::ptr::null_mut::<JSObject>(), (((crate::quickjs_atom::JS_ATOM_input as i32)) as JSAtom), ((((((1 as i32)).wrapping_shl(((0 as i32)) as u32)) | (((1 as i32)).wrapping_shl(((1 as i32)) as u32)))) | (((1 as i32)).wrapping_shl(((2 as i32)) as u32))))) != 0 { 4 } else { 3 }; continue;
}
// C line 49079
6 => {
return ((1 as i32)).wrapping_neg();
}
// C line 49077
7 => {
vm_block = if (add_shape_property(ctx, core::ptr::addr_of_mut!((*(ctx)).regexp_result_shape), core::ptr::null_mut::<JSObject>(), (((crate::quickjs_atom::JS_ATOM_index as i32)) as JSAtom), ((((((1 as i32)).wrapping_shl(((0 as i32)) as u32)) | (((1 as i32)).wrapping_shl(((1 as i32)) as u32)))) | (((1 as i32)).wrapping_shl(((2 as i32)) as u32))))) != 0 { 6 } else { 5 }; continue;
}
// C line 49076
8 => {
return ((1 as i32)).wrapping_neg();
}
// C line 49074
9 => {
vm_block = if (add_shape_property(ctx, core::ptr::addr_of_mut!((*(ctx)).regexp_result_shape), core::ptr::null_mut::<JSObject>(), (((crate::quickjs_atom::JS_ATOM_length as i32)) as JSAtom), ((((1 as i32)).wrapping_shl(((1 as i32)) as u32)) | (((1 as i32)).wrapping_shl(((3 as i32)) as u32))))) != 0 { 8 } else { 7 }; continue;
}
// C line 49073
10 => {
return ((1 as i32)).wrapping_neg();
}
// C line 49072
11 => {
vm_block = if ((!(!((*(ctx)).regexp_result_shape).is_null()) as i32)) != 0 { 10 } else { 9 }; continue;
}
// C line 49070
12 => {
let _ = { let assigned = js_new_shape2(ctx, get_proto_obj(*((*(ctx)).class_proto).offset(((JS_CLASS_ARRAY as i32)) as isize)), (4 as i32), (4 as i32)); (*(ctx)).regexp_result_shape = assigned; assigned };
vm_block = 11; continue;
}
// C line 49068
13 => {
return ((1 as i32)).wrapping_neg();
}
// C line 49066
14 => {
vm_block = if (add_shape_property(ctx, core::ptr::addr_of_mut!((*(ctx)).regexp_shape), core::ptr::null_mut::<JSObject>(), (((crate::quickjs_atom::JS_ATOM_lastIndex as i32)) as JSAtom), ((1 as i32)).wrapping_shl(((1 as i32)) as u32))) != 0 { 13 } else { 12 }; continue;
}
// C line 49065
15 => {
return ((1 as i32)).wrapping_neg();
}
// C line 49064
16 => {
vm_block = if ((!(!((*(ctx)).regexp_shape).is_null()) as i32)) != 0 { 15 } else { 14 }; continue;
}
// C line 49062
17 => {
let _ = { let assigned = js_new_shape2(ctx, get_proto_obj(*((*(ctx)).class_proto).offset(((JS_CLASS_REGEXP as i32)) as isize)), (4 as i32), (1 as i32)); (*(ctx)).regexp_shape = assigned; assigned };
vm_block = 16; continue;
}
// C line 49060
18 => {
return ((1 as i32)).wrapping_neg();
}
// C line 49059
19 => {
vm_block = if (JS_IsException(*((*(ctx)).class_proto).offset(((JS_CLASS_REGEXP_STRING_ITERATOR as i32)) as isize))) != 0 { 18 } else { 17 }; continue;
}
// C line 49055
20 => {
let _ = { let assigned = JS_NewObjectProtoList(ctx, *((*(ctx)).class_proto).offset(((JS_CLASS_ITERATOR as i32)) as isize), (js_regexp_string_iterator_proto_funcs).as_ptr(), (((((size_of::<[JSCFunctionListEntry; 2]>() as usize)) / ((size_of::<JSCFunctionListEntry>() as usize)))) as i32)); *((*(ctx)).class_proto).offset(((JS_CLASS_REGEXP_STRING_ITERATOR as i32)) as isize) = assigned; assigned };
vm_block = 19; continue;
}
// C line 49053
21 => {
let _ = { let assigned = obj; (*(ctx)).regexp_ctor = assigned; assigned };
vm_block = 20; continue;
}
// C line 49052
22 => {
return ((1 as i32)).wrapping_neg();
}
// C line 49051
23 => {
vm_block = if (JS_IsException(obj)) != 0 { 22 } else { 21 }; continue;
}
// C line 49045
24 => {
let _ = { let assigned = JS_NewCConstructor(ctx, (JS_CLASS_REGEXP as i32), c"RegExp".as_ptr(), Some(js_regexp_constructor), (2 as i32), (((JS_CFUNC_constructor_or_func as i32)) as JSCFunctionEnum), (0 as i32), JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }, (js_regexp_funcs).as_ptr(), (((((size_of::<[JSCFunctionListEntry; 2]>() as usize)) / ((size_of::<JSCFunctionListEntry>() as usize)))) as i32), (js_regexp_proto_funcs).as_ptr(), (((((size_of::<[JSCFunctionListEntry; 19]>() as usize)) / ((size_of::<JSCFunctionListEntry>() as usize)))) as i32), (0 as i32)); obj = assigned; assigned };
vm_block = 23; continue;
}
// C line 49043
25 => {
let _ = JS_AddIntrinsicRegExpCompiler(ctx);
vm_block = 24; continue;
}
_ => std::process::abort(),
} }
}
