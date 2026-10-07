// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:50378. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code, unused_variables)]
unsafe fn get_proxy_method(mut ctx: *mut JSContext, mut pmethod: *mut JSValue, mut obj: JSValue, mut name: JSAtom) -> *mut JSProxyData {
let mut vm_local_storage = Vec::<u64>::new();
let mut s: *mut JSProxyData = core::mem::zeroed();
let mut method: JSValue = core::mem::zeroed();
let mut vm_block: usize = 14;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 50401
1 => {
return s;
}
// C line 50400
2 => {
let _ = { let assigned = method; *(pmethod) = assigned; assigned };
vm_block = 1; continue;
}
// C line 50399
3 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; method = assigned; assigned };
vm_block = 2; continue;
}
// C line 50398
4 => {
vm_block = if (JS_IsNull(method)) != 0 { 3 } else { 2 }; continue;
}
// C line 50397
5 => {
return core::ptr::null_mut::<JSProxyData>();
}
// C line 50396
6 => {
vm_block = if (JS_IsException(method)) != 0 { 5 } else { 4 }; continue;
}
// C line 50395
7 => {
let _ = { let assigned = JS_GetProperty(ctx, (*(s)).handler, name); method = assigned; assigned };
vm_block = 6; continue;
}
// C line 50393
8 => {
return core::ptr::null_mut::<JSProxyData>();
}
// C line 50392
9 => {
let _ = JS_ThrowTypeErrorRevokedProxy(ctx);
vm_block = 8; continue;
}
// C line 50391
10 => {
vm_block = if ((*(s)).is_revoked) != 0 { 9 } else { 7 }; continue;
}
// C line 50387
11 => {
return core::ptr::null_mut::<JSProxyData>();
}
// C line 50386
12 => {
let _ = JS_ThrowStackOverflow(ctx);
vm_block = 11; continue;
}
// C line 50385
13 => {
vm_block = if (js_check_stack_overflow((*(ctx)).rt, (((0 as i32)) as usize))) != 0 { 12 } else { 10 }; continue;
}
// C line 50381
14 => {
s = ((JS_GetOpaque(obj, (((JS_CLASS_PROXY as i32)) as JSClassID))) as *mut JSProxyData);
vm_block = 13; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:50404. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code, unused_variables)]
unsafe fn js_proxy_get_prototype(mut ctx: *mut JSContext, mut obj: JSValue) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut s: *mut JSProxyData = core::mem::zeroed();
let mut method: JSValue = core::mem::zeroed();
let mut ret: JSValue = core::mem::zeroed();
let mut proto1: JSValue = core::mem::zeroed();
let mut res: i32 = core::mem::zeroed();
let mut vm_block: usize = 25;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 50442
1 => {
return ret;
}
// C line 50440
2 => {
let _ = JS_FreeValue(ctx, proto1);
vm_block = 1; continue;
}
// C line 50438
3 => {
return JS_ThrowTypeError(ctx, c"proxy: inconsistent prototype".as_ptr());
}
// C line ? labels: fail
4 => {
let _ = JS_FreeValue(ctx, ret);
vm_block = 3; continue;
}
// C line 50435
5 => {
let _ = JS_FreeValue(ctx, proto1);
vm_block = 4; continue;
}
// C line 50434
6 => {
vm_block = if ((!((js_same_value(ctx, proto1, ret)) != 0) as i32)) != 0 { 5 } else { 2 }; continue;
}
// C line 50432
7 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 50431
8 => {
let _ = JS_FreeValue(ctx, ret);
vm_block = 7; continue;
}
// C line 50430
9 => {
vm_block = if (JS_IsException(proto1)) != 0 { 8 } else { 6 }; continue;
}
// C line 50429
10 => {
let _ = { let assigned = JS_GetPrototype(ctx, (*(s)).target); proto1 = assigned; assigned };
vm_block = 9; continue;
}
// C line 50427
11 => {
vm_block = if ((!((res) != 0) as i32)) != 0 { 10 } else { 1 }; continue;
}
// C line 50425
12 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 50424
13 => {
let _ = JS_FreeValue(ctx, ret);
vm_block = 12; continue;
}
// C line 50423
14 => {
vm_block = if ((((res) < ((0 as i32))) as i32)) != 0 { 13 } else { 11 }; continue;
}
// C line 50422
15 => {
let _ = { let assigned = JS_IsExtensible(ctx, (*(s)).target); res = assigned; assigned };
vm_block = 14; continue;
}
// C line 50420
16 => {
vm_block = 4; continue;
}
// C line 50418
17 => {
vm_block = if (((((((((((ret).tag) as i32)) != ((JS_TAG_NULL as i32))) as i32)) != 0) && ((((((((ret).tag) as i32)) != ((JS_TAG_OBJECT as i32))) as i32)) != 0)) as i32)) != 0 { 16 } else { 15 }; continue;
}
// C line 50417
18 => {
return ret;
}
// C line 50416
19 => {
vm_block = if (JS_IsException(ret)) != 0 { 18 } else { 17 }; continue;
}
// C line 50415
20 => {
let _ = { let assigned = JS_CallFree(ctx, method, (*(s)).handler, (1 as i32), core::ptr::addr_of_mut!((*(s)).target)); ret = assigned; assigned };
vm_block = 19; continue;
}
// C line 50414
21 => {
return JS_GetPrototype(ctx, (*(s)).target);
}
// C line 50413
22 => {
vm_block = if (JS_IsUndefined(method)) != 0 { 21 } else { 20 }; continue;
}
// C line 50412
23 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 50411
24 => {
vm_block = if ((!(!(s).is_null()) as i32)) != 0 { 23 } else { 22 }; continue;
}
// C line 50410
25 => {
let _ = { let assigned = get_proxy_method(ctx, core::ptr::addr_of_mut!(method), obj, (((crate::quickjs_atom::JS_ATOM_getPrototypeOf as i32)) as JSAtom)); s = assigned; assigned };
vm_block = 24; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:50445. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code, unused_variables)]
unsafe fn js_proxy_set_prototype(mut ctx: *mut JSContext, mut obj: JSValue, mut proto_val: JSValue) -> i32 {
let mut vm_local_storage = Vec::<u64>::new();
let mut s: *mut JSProxyData = core::mem::zeroed();
let mut method: JSValue = core::mem::zeroed();
let mut ret: JSValue = core::mem::zeroed();
let mut proto1: JSValue = core::mem::zeroed();
let mut args: [JSValue; 2] = core::mem::zeroed();
let mut res: i32 = core::mem::zeroed();
let mut res2: i32 = core::mem::zeroed();
let mut vm_block: usize = 26;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 50481
1 => {
return (1 as i32);
}
// C line 50479
2 => {
let _ = JS_FreeValue(ctx, proto1);
vm_block = 1; continue;
}
// C line 50477
3 => {
return ((1 as i32)).wrapping_neg();
}
// C line 50476
4 => {
let _ = JS_ThrowTypeError(ctx, c"proxy: inconsistent prototype".as_ptr());
vm_block = 3; continue;
}
// C line 50475
5 => {
let _ = JS_FreeValue(ctx, proto1);
vm_block = 4; continue;
}
// C line 50474
6 => {
vm_block = if ((!((js_same_value(ctx, proto_val, proto1)) != 0) as i32)) != 0 { 5 } else { 2 }; continue;
}
// C line 50473
7 => {
return ((1 as i32)).wrapping_neg();
}
// C line 50472
8 => {
vm_block = if (JS_IsException(proto1)) != 0 { 7 } else { 6 }; continue;
}
// C line 50471
9 => {
let _ = { let assigned = JS_GetPrototype(ctx, (*(s)).target); proto1 = assigned; assigned };
vm_block = 8; continue;
}
// C line 50470
10 => {
vm_block = if ((!((res2) != 0) as i32)) != 0 { 9 } else { 1 }; continue;
}
// C line 50469
11 => {
return ((1 as i32)).wrapping_neg();
}
// C line 50468
12 => {
vm_block = if ((((res2) < ((0 as i32))) as i32)) != 0 { 11 } else { 10 }; continue;
}
// C line 50467
13 => {
let _ = { let assigned = JS_IsExtensible(ctx, (*(s)).target); res2 = assigned; assigned };
vm_block = 12; continue;
}
// C line 50466
14 => {
return (0 as i32);
}
// C line 50465
15 => {
vm_block = if ((!((res) != 0) as i32)) != 0 { 14 } else { 13 }; continue;
}
// C line 50464
16 => {
let _ = { let assigned = JS_ToBoolFree(ctx, ret); res = assigned; assigned };
vm_block = 15; continue;
}
// C line 50463
17 => {
return ((1 as i32)).wrapping_neg();
}
// C line 50462
18 => {
vm_block = if (JS_IsException(ret)) != 0 { 17 } else { 16 }; continue;
}
// C line 50461
19 => {
let _ = { let assigned = JS_CallFree(ctx, method, (*(s)).handler, (2 as i32), (args).as_mut_ptr()); ret = assigned; assigned };
vm_block = 18; continue;
}
// C line 50460
20 => {
let _ = { let assigned = proto_val; *((args).as_mut_ptr()).offset(((1 as i32)) as isize) = assigned; assigned };
vm_block = 19; continue;
}
// C line 50459
21 => {
let _ = { let assigned = (*(s)).target; *((args).as_mut_ptr()).offset(((0 as i32)) as isize) = assigned; assigned };
vm_block = 20; continue;
}
// C line 50458
22 => {
return JS_SetPrototypeInternal(ctx, (*(s)).target, proto_val, (0 as i32));
}
// C line 50457
23 => {
vm_block = if (JS_IsUndefined(method)) != 0 { 22 } else { 21 }; continue;
}
// C line 50456
24 => {
return ((1 as i32)).wrapping_neg();
}
// C line 50455
25 => {
vm_block = if ((!(!(s).is_null()) as i32)) != 0 { 24 } else { 23 }; continue;
}
// C line 50454
26 => {
let _ = { let assigned = get_proxy_method(ctx, core::ptr::addr_of_mut!(method), obj, (((crate::quickjs_atom::JS_ATOM_setPrototypeOf as i32)) as JSAtom)); s = assigned; assigned };
vm_block = 25; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:50484. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code, unused_variables)]
unsafe fn js_proxy_is_extensible(mut ctx: *mut JSContext, mut obj: JSValue) -> i32 {
let mut vm_local_storage = Vec::<u64>::new();
let mut s: *mut JSProxyData = core::mem::zeroed();
let mut method: JSValue = core::mem::zeroed();
let mut ret: JSValue = core::mem::zeroed();
let mut res: i32 = core::mem::zeroed();
let mut res2: i32 = core::mem::zeroed();
let mut vm_block: usize = 16;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 50507
1 => {
return res;
}
// C line 50505
2 => {
return ((1 as i32)).wrapping_neg();
}
// C line 50504
3 => {
let _ = JS_ThrowTypeError(ctx, c"proxy: inconsistent isExtensible".as_ptr());
vm_block = 2; continue;
}
// C line 50503
4 => {
vm_block = if ((((res) != (res2)) as i32)) != 0 { 3 } else { 1 }; continue;
}
// C line 50502
5 => {
return res2;
}
// C line 50501
6 => {
vm_block = if ((((res2) < ((0 as i32))) as i32)) != 0 { 5 } else { 4 }; continue;
}
// C line 50500
7 => {
let _ = { let assigned = JS_IsExtensible(ctx, (*(s)).target); res2 = assigned; assigned };
vm_block = 6; continue;
}
// C line 50499
8 => {
let _ = { let assigned = JS_ToBoolFree(ctx, ret); res = assigned; assigned };
vm_block = 7; continue;
}
// C line 50498
9 => {
return ((1 as i32)).wrapping_neg();
}
// C line 50497
10 => {
vm_block = if (JS_IsException(ret)) != 0 { 9 } else { 8 }; continue;
}
// C line 50496
11 => {
let _ = { let assigned = JS_CallFree(ctx, method, (*(s)).handler, (1 as i32), core::ptr::addr_of_mut!((*(s)).target)); ret = assigned; assigned };
vm_block = 10; continue;
}
// C line 50495
12 => {
return JS_IsExtensible(ctx, (*(s)).target);
}
// C line 50494
13 => {
vm_block = if (JS_IsUndefined(method)) != 0 { 12 } else { 11 }; continue;
}
// C line 50493
14 => {
return ((1 as i32)).wrapping_neg();
}
// C line 50492
15 => {
vm_block = if ((!(!(s).is_null()) as i32)) != 0 { 14 } else { 13 }; continue;
}
// C line 50491
16 => {
let _ = { let assigned = get_proxy_method(ctx, core::ptr::addr_of_mut!(method), obj, (((crate::quickjs_atom::JS_ATOM_isExtensible as i32)) as JSAtom)); s = assigned; assigned };
vm_block = 15; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:50510. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code, unused_variables)]
unsafe fn js_proxy_prevent_extensions(mut ctx: *mut JSContext, mut obj: JSValue) -> i32 {
let mut vm_local_storage = Vec::<u64>::new();
let mut s: *mut JSProxyData = core::mem::zeroed();
let mut method: JSValue = core::mem::zeroed();
let mut ret: JSValue = core::mem::zeroed();
let mut res: i32 = core::mem::zeroed();
let mut res2: i32 = core::mem::zeroed();
let mut vm_block: usize = 17;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 50535
1 => {
return res;
}
// C line 50532
2 => {
return ((1 as i32)).wrapping_neg();
}
// C line 50531
3 => {
let _ = JS_ThrowTypeError(ctx, c"proxy: inconsistent preventExtensions".as_ptr());
vm_block = 2; continue;
}
// C line 50530
4 => {
vm_block = if (res2) != 0 { 3 } else { 1 }; continue;
}
// C line 50529
5 => {
return res2;
}
// C line 50528
6 => {
vm_block = if ((((res2) < ((0 as i32))) as i32)) != 0 { 5 } else { 4 }; continue;
}
// C line 50527
7 => {
let _ = { let assigned = JS_IsExtensible(ctx, (*(s)).target); res2 = assigned; assigned };
vm_block = 6; continue;
}
// C line 50526
8 => {
vm_block = if (res) != 0 { 7 } else { 1 }; continue;
}
// C line 50525
9 => {
let _ = { let assigned = JS_ToBoolFree(ctx, ret); res = assigned; assigned };
vm_block = 8; continue;
}
// C line 50524
10 => {
return ((1 as i32)).wrapping_neg();
}
// C line 50523
11 => {
vm_block = if (JS_IsException(ret)) != 0 { 10 } else { 9 }; continue;
}
// C line 50522
12 => {
let _ = { let assigned = JS_CallFree(ctx, method, (*(s)).handler, (1 as i32), core::ptr::addr_of_mut!((*(s)).target)); ret = assigned; assigned };
vm_block = 11; continue;
}
// C line 50521
13 => {
return JS_PreventExtensions(ctx, (*(s)).target);
}
// C line 50520
14 => {
vm_block = if (JS_IsUndefined(method)) != 0 { 13 } else { 12 }; continue;
}
// C line 50519
15 => {
return ((1 as i32)).wrapping_neg();
}
// C line 50518
16 => {
vm_block = if ((!(!(s).is_null()) as i32)) != 0 { 15 } else { 14 }; continue;
}
// C line 50517
17 => {
let _ = { let assigned = get_proxy_method(ctx, core::ptr::addr_of_mut!(method), obj, (((crate::quickjs_atom::JS_ATOM_preventExtensions as i32)) as JSAtom)); s = assigned; assigned };
vm_block = 16; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:50538. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code, unused_variables)]
unsafe fn js_proxy_has(mut ctx: *mut JSContext, mut obj: JSValue, mut atom: JSAtom) -> i32 {
let mut vm_local_storage = Vec::<u64>::new();
let mut s: *mut JSProxyData = core::mem::zeroed();
let mut method: JSValue = core::mem::zeroed();
let mut ret1: JSValue = core::mem::zeroed();
let mut atom_val: JSValue = core::mem::zeroed();
let mut ret: i32 = core::mem::zeroed();
let mut res: i32 = core::mem::zeroed();
let mut p: *mut JSObject = core::mem::zeroed();
let mut args: [JSValue; 2] = core::mem::zeroed();
let mut res2: i32 = core::mem::zeroed();
let mut desc: JSPropertyDescriptor = core::mem::zeroed();
let mut vm_block: usize = 28;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 50579
1 => {
return ret;
}
// C line 50575
2 => {
return ((1 as i32)).wrapping_neg();
}
// C line 50574
3 => {
let _ = JS_ThrowTypeError(ctx, c"proxy: inconsistent has".as_ptr());
vm_block = 2; continue;
}
// C line 50573
4 => {
vm_block = if (((((res2) != 0) || (((!(((*(p)).extensible()) != 0) as i32)) != 0)) as i32)) != 0 { 3 } else { 1 }; continue;
}
// C line 50572
5 => {
let _ = js_free_desc(ctx, core::ptr::addr_of_mut!(desc));
vm_block = 4; continue;
}
// C line 50571
6 => {
let _ = { let assigned = (!(((((desc).flags) & (((1 as i32)).wrapping_shl(((0 as i32)) as u32)))) != 0) as i32); res2 = assigned; assigned };
vm_block = 5; continue;
}
// C line 50570
7 => {
vm_block = if (res) != 0 { 6 } else { 1 }; continue;
}
// C line 50569
8 => {
return ((1 as i32)).wrapping_neg();
}
// C line 50568
9 => {
vm_block = if ((((res) < ((0 as i32))) as i32)) != 0 { 8 } else { 7 }; continue;
}
// C line 50567
10 => {
let _ = { let assigned = JS_GetOwnPropertyInternal(ctx, core::ptr::addr_of_mut!(desc), p, atom); res = assigned; assigned };
vm_block = 9; continue;
}
// C line 50566
11 => {
let _ = { let assigned = (((((*(s)).target).u).ptr) as *mut JSObject); p = assigned; assigned };
vm_block = 10; continue;
}
// C line 50564
12 => {
vm_block = if ((!((ret) != 0) as i32)) != 0 { 11 } else { 1 }; continue;
}
// C line 50563
13 => {
let _ = { let assigned = JS_ToBoolFree(ctx, ret1); ret = assigned; assigned };
vm_block = 12; continue;
}
// C line 50562
14 => {
return ((1 as i32)).wrapping_neg();
}
// C line 50561
15 => {
vm_block = if (JS_IsException(ret1)) != 0 { 14 } else { 13 }; continue;
}
// C line 50560
16 => {
let _ = JS_FreeValue(ctx, atom_val);
vm_block = 15; continue;
}
// C line 50559
17 => {
let _ = { let assigned = JS_CallFree(ctx, method, (*(s)).handler, (2 as i32), (args).as_mut_ptr()); ret1 = assigned; assigned };
vm_block = 16; continue;
}
// C line 50558
18 => {
let _ = { let assigned = atom_val; *((args).as_mut_ptr()).offset(((1 as i32)) as isize) = assigned; assigned };
vm_block = 17; continue;
}
// C line 50557
19 => {
let _ = { let assigned = (*(s)).target; *((args).as_mut_ptr()).offset(((0 as i32)) as isize) = assigned; assigned };
vm_block = 18; continue;
}
// C line 50555
20 => {
return ((1 as i32)).wrapping_neg();
}
// C line 50554
21 => {
let _ = JS_FreeValue(ctx, method);
vm_block = 20; continue;
}
// C line 50553
22 => {
vm_block = if (JS_IsException(atom_val)) != 0 { 21 } else { 19 }; continue;
}
// C line 50552
23 => {
let _ = { let assigned = JS_AtomToValue(ctx, atom); atom_val = assigned; assigned };
vm_block = 22; continue;
}
// C line 50551
24 => {
return JS_HasProperty(ctx, (*(s)).target, atom);
}
// C line 50550
25 => {
vm_block = if (JS_IsUndefined(method)) != 0 { 24 } else { 23 }; continue;
}
// C line 50549
26 => {
return ((1 as i32)).wrapping_neg();
}
// C line 50548
27 => {
vm_block = if ((!(!(s).is_null()) as i32)) != 0 { 26 } else { 25 }; continue;
}
// C line 50547
28 => {
let _ = { let assigned = get_proxy_method(ctx, core::ptr::addr_of_mut!(method), obj, (((crate::quickjs_atom::JS_ATOM_has as i32)) as JSAtom)); s = assigned; assigned };
vm_block = 27; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:50582. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code, unused_variables)]
unsafe fn js_proxy_get(mut ctx: *mut JSContext, mut obj: JSValue, mut atom: JSAtom, mut receiver: JSValue) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut s: *mut JSProxyData = core::mem::zeroed();
let mut method: JSValue = core::mem::zeroed();
let mut ret: JSValue = core::mem::zeroed();
let mut atom_val: JSValue = core::mem::zeroed();
let mut res: i32 = core::mem::zeroed();
let mut args: [JSValue; 3] = core::mem::zeroed();
let mut desc: JSPropertyDescriptor = core::mem::zeroed();
let mut vm_block: usize = 31;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 50629
1 => {
return ret;
}
// C line 50627
2 => {
let _ = js_free_desc(ctx, core::ptr::addr_of_mut!(desc));
vm_block = 1; continue;
}
// C line 50617
3 => {
vm_block = 7; continue;
}
// C line 50616
4 => {
vm_block = if ((!((js_same_value(ctx, (desc).value, ret)) != 0) as i32)) != 0 { 3 } else { 2 }; continue;
}
// C line 50624
5 => {
return JS_ThrowTypeError(ctx, c"proxy: inconsistent get".as_ptr());
}
// C line 50623
6 => {
let _ = JS_FreeValue(ctx, ret);
vm_block = 5; continue;
}
// C line ? labels: fail
7 => {
let _ = js_free_desc(ctx, core::ptr::addr_of_mut!(desc));
vm_block = 6; continue;
}
// C line 50620
8 => {
vm_block = if (((((JS_IsUndefined((desc).getter)) != 0) && (((!((JS_IsUndefined(ret)) != 0) as i32)) != 0)) as i32)) != 0 { 7 } else { 2 }; continue;
}
// C line 50619
9 => {
vm_block = if (((((((desc).flags) & (((((1 as i32)).wrapping_shl(((4 as i32)) as u32)) | (((1 as i32)).wrapping_shl(((0 as i32)) as u32)))))) == (((1 as i32)).wrapping_shl(((4 as i32)) as u32))) as i32)) != 0 { 8 } else { 2 }; continue;
}
// C line 50615
10 => {
vm_block = if (((((((desc).flags) & (((((((1 as i32)).wrapping_shl(((4 as i32)) as u32)) | (((1 as i32)).wrapping_shl(((0 as i32)) as u32)))) | (((1 as i32)).wrapping_shl(((1 as i32)) as u32)))))) == ((0 as i32))) as i32)) != 0 { 4 } else { 9 }; continue;
}
// C line 50614
11 => {
vm_block = if (res) != 0 { 10 } else { 1 }; continue;
}
// C line 50612
12 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 50611
13 => {
let _ = JS_FreeValue(ctx, ret);
vm_block = 12; continue;
}
// C line 50610
14 => {
vm_block = if ((((res) < ((0 as i32))) as i32)) != 0 { 13 } else { 11 }; continue;
}
// C line 50609
15 => {
let _ = { let assigned = JS_GetOwnPropertyInternal(ctx, core::ptr::addr_of_mut!(desc), (((((*(s)).target).u).ptr) as *mut JSObject), atom); res = assigned; assigned };
vm_block = 14; continue;
}
// C line 50608
16 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 50607
17 => {
vm_block = if (JS_IsException(ret)) != 0 { 16 } else { 15 }; continue;
}
// C line 50606
18 => {
let _ = JS_FreeValue(ctx, atom_val);
vm_block = 17; continue;
}
// C line 50605
19 => {
let _ = { let assigned = JS_CallFree(ctx, method, (*(s)).handler, (3 as i32), (args).as_mut_ptr()); ret = assigned; assigned };
vm_block = 18; continue;
}
// C line 50604
20 => {
let _ = { let assigned = receiver; *((args).as_mut_ptr()).offset(((2 as i32)) as isize) = assigned; assigned };
vm_block = 19; continue;
}
// C line 50603
21 => {
let _ = { let assigned = atom_val; *((args).as_mut_ptr()).offset(((1 as i32)) as isize) = assigned; assigned };
vm_block = 20; continue;
}
// C line 50602
22 => {
let _ = { let assigned = (*(s)).target; *((args).as_mut_ptr()).offset(((0 as i32)) as isize) = assigned; assigned };
vm_block = 21; continue;
}
// C line 50600
23 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 50599
24 => {
let _ = JS_FreeValue(ctx, method);
vm_block = 23; continue;
}
// C line 50598
25 => {
vm_block = if (JS_IsException(atom_val)) != 0 { 24 } else { 22 }; continue;
}
// C line 50597
26 => {
let _ = { let assigned = JS_AtomToValue(ctx, atom); atom_val = assigned; assigned };
vm_block = 25; continue;
}
// C line 50596
27 => {
return JS_GetPropertyInternal(ctx, (*(s)).target, atom, receiver, (0 as i32));
}
// C line 50595
28 => {
vm_block = if (JS_IsUndefined(method)) != 0 { 27 } else { 26 }; continue;
}
// C line 50593
29 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 50592
30 => {
vm_block = if ((!(!(s).is_null()) as i32)) != 0 { 29 } else { 28 }; continue;
}
// C line 50591
31 => {
let _ = { let assigned = get_proxy_method(ctx, core::ptr::addr_of_mut!(method), obj, (((crate::quickjs_atom::JS_ATOM_get as i32)) as JSAtom)); s = assigned; assigned };
vm_block = 30; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:50632. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code, unused_variables)]
unsafe fn js_proxy_set(mut ctx: *mut JSContext, mut obj: JSValue, mut atom: JSAtom, mut value: JSValue, mut receiver: JSValue, mut flags: i32) -> i32 {
let mut vm_local_storage = Vec::<u64>::new();
let mut s: *mut JSProxyData = core::mem::zeroed();
let mut method: JSValue = core::mem::zeroed();
let mut ret1: JSValue = core::mem::zeroed();
let mut atom_val: JSValue = core::mem::zeroed();
let mut ret: i32 = core::mem::zeroed();
let mut res: i32 = core::mem::zeroed();
let mut args: [JSValue; 4] = core::mem::zeroed();
let mut desc: JSPropertyDescriptor = core::mem::zeroed();
let mut vm_block: usize = 35;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 50687
1 => {
return ret;
}
// C line 50678
2 => {
let _ = js_free_desc(ctx, core::ptr::addr_of_mut!(desc));
vm_block = 1; continue;
}
// C line 50670
3 => {
vm_block = 7; continue;
}
// C line 50669
4 => {
vm_block = if ((!((js_same_value(ctx, (desc).value, value)) != 0) as i32)) != 0 { 3 } else { 2 }; continue;
}
// C line 50676
5 => {
return ((1 as i32)).wrapping_neg();
}
// C line 50675
6 => {
let _ = JS_ThrowTypeError(ctx, c"proxy: inconsistent set".as_ptr());
vm_block = 5; continue;
}
// C line ? labels: fail
7 => {
let _ = js_free_desc(ctx, core::ptr::addr_of_mut!(desc));
vm_block = 6; continue;
}
// C line 50672
8 => {
vm_block = if (((((((((((desc).flags) & (((((1 as i32)).wrapping_shl(((4 as i32)) as u32)) | (((1 as i32)).wrapping_shl(((0 as i32)) as u32)))))) == (((1 as i32)).wrapping_shl(((4 as i32)) as u32))) as i32)) != 0) && ((JS_IsUndefined((desc).setter)) != 0)) as i32)) != 0 { 7 } else { 2 }; continue;
}
// C line 50668
9 => {
vm_block = if (((((((desc).flags) & (((((((1 as i32)).wrapping_shl(((4 as i32)) as u32)) | (((1 as i32)).wrapping_shl(((0 as i32)) as u32)))) | (((1 as i32)).wrapping_shl(((1 as i32)) as u32)))))) == ((0 as i32))) as i32)) != 0 { 4 } else { 8 }; continue;
}
// C line 50667
10 => {
vm_block = if (res) != 0 { 9 } else { 1 }; continue;
}
// C line 50666
11 => {
return ((1 as i32)).wrapping_neg();
}
// C line 50665
12 => {
vm_block = if ((((res) < ((0 as i32))) as i32)) != 0 { 11 } else { 10 }; continue;
}
// C line 50664
13 => {
let _ = { let assigned = JS_GetOwnPropertyInternal(ctx, core::ptr::addr_of_mut!(desc), (((((*(s)).target).u).ptr) as *mut JSObject), atom); res = assigned; assigned };
vm_block = 12; continue;
}
// C line 50684
14 => {
return ((1 as i32)).wrapping_neg();
}
// C line 50683
15 => {
let _ = JS_ThrowTypeError(ctx, c"proxy: cannot set property".as_ptr());
vm_block = 14; continue;
}
// C line 50681
16 => {
vm_block = if (((((((flags) & (((1 as i32)).wrapping_shl(((14 as i32)) as u32)))) != 0) || ((((((((flags) & (((1 as i32)).wrapping_shl(((15 as i32)) as u32)))) != 0) && ((is_strict_mode(ctx)) != 0)) as i32)) != 0)) as i32)) != 0 { 15 } else { 1 }; continue;
}
// C line 50662
17 => {
vm_block = if (ret) != 0 { 13 } else { 16 }; continue;
}
// C line 50661
18 => {
let _ = { let assigned = JS_ToBoolFree(ctx, ret1); ret = assigned; assigned };
vm_block = 17; continue;
}
// C line 50660
19 => {
return ((1 as i32)).wrapping_neg();
}
// C line 50659
20 => {
vm_block = if (JS_IsException(ret1)) != 0 { 19 } else { 18 }; continue;
}
// C line 50658
21 => {
let _ = JS_FreeValue(ctx, atom_val);
vm_block = 20; continue;
}
// C line 50657
22 => {
let _ = { let assigned = JS_CallFree(ctx, method, (*(s)).handler, (4 as i32), (args).as_mut_ptr()); ret1 = assigned; assigned };
vm_block = 21; continue;
}
// C line 50656
23 => {
let _ = { let assigned = receiver; *((args).as_mut_ptr()).offset(((3 as i32)) as isize) = assigned; assigned };
vm_block = 22; continue;
}
// C line 50655
24 => {
let _ = { let assigned = value; *((args).as_mut_ptr()).offset(((2 as i32)) as isize) = assigned; assigned };
vm_block = 23; continue;
}
// C line 50654
25 => {
let _ = { let assigned = atom_val; *((args).as_mut_ptr()).offset(((1 as i32)) as isize) = assigned; assigned };
vm_block = 24; continue;
}
// C line 50653
26 => {
let _ = { let assigned = (*(s)).target; *((args).as_mut_ptr()).offset(((0 as i32)) as isize) = assigned; assigned };
vm_block = 25; continue;
}
// C line 50651
27 => {
return ((1 as i32)).wrapping_neg();
}
// C line 50650
28 => {
let _ = JS_FreeValue(ctx, method);
vm_block = 27; continue;
}
// C line 50649
29 => {
vm_block = if (JS_IsException(atom_val)) != 0 { 28 } else { 26 }; continue;
}
// C line 50648
30 => {
let _ = { let assigned = JS_AtomToValue(ctx, atom); atom_val = assigned; assigned };
vm_block = 29; continue;
}
// C line 50644
31 => {
return JS_SetPropertyInternal(ctx, (*(s)).target, atom, JS_DupValue(ctx, value), receiver, flags);
}
// C line 50643
32 => {
vm_block = if (JS_IsUndefined(method)) != 0 { 31 } else { 30 }; continue;
}
// C line 50642
33 => {
return ((1 as i32)).wrapping_neg();
}
// C line 50641
34 => {
vm_block = if ((!(!(s).is_null()) as i32)) != 0 { 33 } else { 32 }; continue;
}
// C line 50640
35 => {
let _ = { let assigned = get_proxy_method(ctx, core::ptr::addr_of_mut!(method), obj, (((crate::quickjs_atom::JS_ATOM_set as i32)) as JSAtom)); s = assigned; assigned };
vm_block = 34; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:50690. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code, unused_variables)]
unsafe fn js_create_desc(mut ctx: *mut JSContext, mut val: JSValue, mut getter: JSValue, mut setter: JSValue, mut flags: i32) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut ret: JSValue = core::mem::zeroed();
let mut vm_block: usize = 16;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 50725
1 => {
return ret;
}
// C line 50721
2 => {
let _ = JS_DefinePropertyValue(ctx, ret, (((crate::quickjs_atom::JS_ATOM_configurable as i32)) as JSAtom), JS_NewBool(ctx, ((flags) & (((1 as i32)).wrapping_shl(((0 as i32)) as u32)))), ((((((1 as i32)).wrapping_shl(((0 as i32)) as u32)) | (((1 as i32)).wrapping_shl(((1 as i32)) as u32)))) | (((1 as i32)).wrapping_shl(((2 as i32)) as u32))));
vm_block = 1; continue;
}
// C line 50720
3 => {
vm_block = if (((flags) & (((1 as i32)).wrapping_shl(((8 as i32)) as u32)))) != 0 { 2 } else { 1 }; continue;
}
// C line 50716
4 => {
let _ = JS_DefinePropertyValue(ctx, ret, (((crate::quickjs_atom::JS_ATOM_enumerable as i32)) as JSAtom), JS_NewBool(ctx, ((flags) & (((1 as i32)).wrapping_shl(((2 as i32)) as u32)))), ((((((1 as i32)).wrapping_shl(((0 as i32)) as u32)) | (((1 as i32)).wrapping_shl(((1 as i32)) as u32)))) | (((1 as i32)).wrapping_shl(((2 as i32)) as u32))));
vm_block = 3; continue;
}
// C line 50715
5 => {
vm_block = if (((flags) & (((1 as i32)).wrapping_shl(((10 as i32)) as u32)))) != 0 { 4 } else { 3 }; continue;
}
// C line 50711
6 => {
let _ = JS_DefinePropertyValue(ctx, ret, (((crate::quickjs_atom::JS_ATOM_writable as i32)) as JSAtom), JS_NewBool(ctx, ((flags) & (((1 as i32)).wrapping_shl(((1 as i32)) as u32)))), ((((((1 as i32)).wrapping_shl(((0 as i32)) as u32)) | (((1 as i32)).wrapping_shl(((1 as i32)) as u32)))) | (((1 as i32)).wrapping_shl(((2 as i32)) as u32))));
vm_block = 5; continue;
}
// C line 50710
7 => {
vm_block = if (((flags) & (((1 as i32)).wrapping_shl(((9 as i32)) as u32)))) != 0 { 6 } else { 5 }; continue;
}
// C line 50707
8 => {
let _ = JS_DefinePropertyValue(ctx, ret, (((crate::quickjs_atom::JS_ATOM_value as i32)) as JSAtom), JS_DupValue(ctx, val), ((((((1 as i32)).wrapping_shl(((0 as i32)) as u32)) | (((1 as i32)).wrapping_shl(((1 as i32)) as u32)))) | (((1 as i32)).wrapping_shl(((2 as i32)) as u32))));
vm_block = 7; continue;
}
// C line 50706
9 => {
vm_block = if (((flags) & (((1 as i32)).wrapping_shl(((13 as i32)) as u32)))) != 0 { 8 } else { 7 }; continue;
}
// C line 50703
10 => {
let _ = JS_DefinePropertyValue(ctx, ret, (((crate::quickjs_atom::JS_ATOM_set as i32)) as JSAtom), JS_DupValue(ctx, setter), ((((((1 as i32)).wrapping_shl(((0 as i32)) as u32)) | (((1 as i32)).wrapping_shl(((1 as i32)) as u32)))) | (((1 as i32)).wrapping_shl(((2 as i32)) as u32))));
vm_block = 9; continue;
}
// C line 50702
11 => {
vm_block = if (((flags) & (((1 as i32)).wrapping_shl(((12 as i32)) as u32)))) != 0 { 10 } else { 9 }; continue;
}
// C line 50699
12 => {
let _ = JS_DefinePropertyValue(ctx, ret, (((crate::quickjs_atom::JS_ATOM_get as i32)) as JSAtom), JS_DupValue(ctx, getter), ((((((1 as i32)).wrapping_shl(((0 as i32)) as u32)) | (((1 as i32)).wrapping_shl(((1 as i32)) as u32)))) | (((1 as i32)).wrapping_shl(((2 as i32)) as u32))));
vm_block = 11; continue;
}
// C line 50698
13 => {
vm_block = if (((flags) & (((1 as i32)).wrapping_shl(((11 as i32)) as u32)))) != 0 { 12 } else { 11 }; continue;
}
// C line 50697
14 => {
return ret;
}
// C line 50696
15 => {
vm_block = if (JS_IsException(ret)) != 0 { 14 } else { 13 }; continue;
}
// C line 50695
16 => {
let _ = { let assigned = JS_NewObject(ctx); ret = assigned; assigned };
vm_block = 15; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:50728. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code, unused_variables)]
unsafe fn js_proxy_get_own_property(mut ctx: *mut JSContext, mut pdesc: *mut JSPropertyDescriptor, mut obj: JSValue, mut prop: JSAtom) -> i32 {
let mut vm_local_storage = Vec::<u64>::new();
let mut s: *mut JSProxyData = core::mem::zeroed();
let mut method: JSValue = core::mem::zeroed();
let mut trap_result_obj: JSValue = core::mem::zeroed();
let mut prop_val: JSValue = core::mem::zeroed();
let mut res: i32 = core::mem::zeroed();
let mut target_desc_ret: i32 = core::mem::zeroed();
let mut ret: i32 = core::mem::zeroed();
let mut p: *mut JSObject = core::mem::zeroed();
let mut args: [JSValue; 2] = core::mem::zeroed();
let mut result_desc: JSPropertyDescriptor = core::mem::zeroed();
let mut target_desc: JSPropertyDescriptor = core::mem::zeroed();
let mut flags1: i32 = core::mem::zeroed();
let mut extensible_target: i32 = core::mem::zeroed();
let mut vm_block: usize = 63;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 50830
1 => {
return ret;
}
// C line 50772
2 => {
let _ = { let assigned = (0 as i32); ret = assigned; assigned };
vm_block = 1; continue;
}
// C line 50770
3 => {
vm_block = 11; continue;
}
// C line 50769
4 => {
vm_block = if ((((((!(((((target_desc).flags) & (((1 as i32)).wrapping_shl(((0 as i32)) as u32)))) != 0) as i32)) != 0) || (((!(((*(p)).extensible()) != 0) as i32)) != 0)) as i32)) != 0 { 3 } else { 2 }; continue;
}
// C line 50768
5 => {
vm_block = if (target_desc_ret) != 0 { 4 } else { 2 }; continue;
}
// C line 50825
6 => {
let _ = { let assigned = result_desc; *(pdesc) = assigned; assigned };
vm_block = 1; continue;
}
// C line 50827
7 => {
let _ = js_free_desc(ctx, core::ptr::addr_of_mut!(result_desc));
vm_block = 1; continue;
}
// C line 50824
8 => {
vm_block = if !(pdesc).is_null() { 6 } else { 7 }; continue;
}
// C line 50823
9 => {
let _ = { let assigned = (1 as i32); ret = assigned; assigned };
vm_block = 8; continue;
}
// C line 50820
10 => {
return ((1 as i32)).wrapping_neg();
}
// C line ? labels: fail
11 => {
let _ = JS_ThrowTypeError(ctx, c"proxy: inconsistent getOwnPropertyDescriptor".as_ptr());
vm_block = 10; continue;
}
// C line ? labels: fail1
12 => {
let _ = js_free_desc(ctx, core::ptr::addr_of_mut!(result_desc));
vm_block = 11; continue;
}
// C line 50811
13 => {
vm_block = if (((((((((((((((result_desc).flags) & (((((1 as i32)).wrapping_shl(((4 as i32)) as u32)) | (((1 as i32)).wrapping_shl(((1 as i32)) as u32)))))) == ((0 as i32))) as i32)) != 0) && ((target_desc_ret) != 0)) as i32)) != 0) && ((((((((target_desc).flags) & (((1 as i32)).wrapping_shl(((1 as i32)) as u32)))) != ((0 as i32))) as i32)) != 0)) as i32)) != 0 { 12 } else { 9 }; continue;
}
// C line 50810
14 => {
vm_block = 12; continue;
}
// C line 50809
15 => {
vm_block = if ((((((!((target_desc_ret) != 0) as i32)) != 0) || (((((target_desc).flags) & (((1 as i32)).wrapping_shl(((0 as i32)) as u32)))) != 0)) as i32)) != 0 { 14 } else { 13 }; continue;
}
// C line 50808
16 => {
vm_block = if ((!(((((result_desc).flags) & (((1 as i32)).wrapping_shl(((0 as i32)) as u32)))) != 0) as i32)) != 0 { 15 } else { 9 }; continue;
}
// C line 50803
17 => {
vm_block = 12; continue;
}
// C line 50802
18 => {
vm_block = if ((!((check_define_prop_flags((target_desc).flags, flags1)) != 0) as i32)) != 0 { 17 } else { 16 }; continue;
}
// C line 50797
19 => {
let _ = { flags1 = ((flags1) | (((((1 as i32)).wrapping_shl(((11 as i32)) as u32)) | (((1 as i32)).wrapping_shl(((12 as i32)) as u32))))); flags1 };
vm_block = 18; continue;
}
// C line 50799
20 => {
let _ = { flags1 = ((flags1) | (((((1 as i32)).wrapping_shl(((13 as i32)) as u32)) | (((1 as i32)).wrapping_shl(((9 as i32)) as u32))))); flags1 };
vm_block = 18; continue;
}
// C line 50796
21 => {
vm_block = if ((((result_desc).flags) & (((1 as i32)).wrapping_shl(((4 as i32)) as u32)))) != 0 { 19 } else { 20 }; continue;
}
// C line 50795
22 => {
let _ = { let assigned = (((((result_desc).flags) | (((1 as i32)).wrapping_shl(((8 as i32)) as u32)))) | (((1 as i32)).wrapping_shl(((10 as i32)) as u32))); flags1 = assigned; assigned };
vm_block = 21; continue;
}
// C line 50806
23 => {
vm_block = 12; continue;
}
// C line 50805
24 => {
vm_block = if ((!((extensible_target) != 0) as i32)) != 0 { 23 } else { 16 }; continue;
}
// C line 50793
25 => {
vm_block = if (target_desc_ret) != 0 { 22 } else { 24 }; continue;
}
// C line 50791
26 => {
let _ = { (result_desc).flags = (((result_desc).flags) & (((((((((1 as i32)).wrapping_shl(((0 as i32)) as u32)) | (((1 as i32)).wrapping_shl(((1 as i32)) as u32)))) | (((1 as i32)).wrapping_shl(((2 as i32)) as u32)))) | (((3 as i32)).wrapping_shl(((4 as i32)) as u32))))); (result_desc).flags };
vm_block = 25; continue;
}
// C line 50787
27 => {
let _ = { (result_desc).flags = (((result_desc).flags) | (((1 as i32)).wrapping_shl(((4 as i32)) as u32))); (result_desc).flags };
vm_block = 26; continue;
}
// C line 50789
28 => {
let _ = { (result_desc).flags = (((result_desc).flags) | (((0 as i32)).wrapping_shl(((4 as i32)) as u32))); (result_desc).flags };
vm_block = 26; continue;
}
// C line 50786
29 => {
vm_block = if ((((result_desc).flags) & (((((1 as i32)).wrapping_shl(((11 as i32)) as u32)) | (((1 as i32)).wrapping_shl(((12 as i32)) as u32)))))) != 0 { 27 } else { 28 }; continue;
}
// C line 50783
30 => {
return ((1 as i32)).wrapping_neg();
}
// C line 50782
31 => {
vm_block = if ((((res) < ((0 as i32))) as i32)) != 0 { 30 } else { 29 }; continue;
}
// C line 50781
32 => {
let _ = JS_FreeValue(ctx, trap_result_obj);
vm_block = 31; continue;
}
// C line 50780
33 => {
let _ = { let assigned = js_obj_to_desc(ctx, core::ptr::addr_of_mut!(result_desc), trap_result_obj); res = assigned; assigned };
vm_block = 32; continue;
}
// C line 50778
34 => {
return ((1 as i32)).wrapping_neg();
}
// C line 50777
35 => {
let _ = JS_FreeValue(ctx, trap_result_obj);
vm_block = 34; continue;
}
// C line 50776
36 => {
vm_block = if ((((extensible_target) < ((0 as i32))) as i32)) != 0 { 35 } else { 33 }; continue;
}
// C line 50775
37 => {
let _ = { let assigned = JS_IsExtensible(ctx, (*(s)).target); extensible_target = assigned; assigned };
vm_block = 36; continue;
}
// C line 50767
38 => {
vm_block = if (JS_IsUndefined(trap_result_obj)) != 0 { 5 } else { 37 }; continue;
}
// C line 50766
39 => {
let _ = js_free_desc(ctx, core::ptr::addr_of_mut!(target_desc));
vm_block = 38; continue;
}
// C line 50765
40 => {
vm_block = if (target_desc_ret) != 0 { 39 } else { 38 }; continue;
}
// C line 50763
41 => {
return ((1 as i32)).wrapping_neg();
}
// C line 50762
42 => {
let _ = JS_FreeValue(ctx, trap_result_obj);
vm_block = 41; continue;
}
// C line 50761
43 => {
vm_block = if ((((target_desc_ret) < ((0 as i32))) as i32)) != 0 { 42 } else { 40 }; continue;
}
// C line 50760
44 => {
let _ = { let assigned = JS_GetOwnPropertyInternal(ctx, core::ptr::addr_of_mut!(target_desc), p, prop); target_desc_ret = assigned; assigned };
vm_block = 43; continue;
}
// C line 50758
45 => {
vm_block = 11; continue;
}
// C line 50757
46 => {
let _ = JS_FreeValue(ctx, trap_result_obj);
vm_block = 45; continue;
}
// C line 50756
47 => {
vm_block = if ((((((!((JS_IsObject(trap_result_obj)) != 0) as i32)) != 0) && (((!((JS_IsUndefined(trap_result_obj)) != 0) as i32)) != 0)) as i32)) != 0 { 46 } else { 44 }; continue;
}
// C line 50755
48 => {
return ((1 as i32)).wrapping_neg();
}
// C line 50754
49 => {
vm_block = if (JS_IsException(trap_result_obj)) != 0 { 48 } else { 47 }; continue;
}
// C line 50753
50 => {
let _ = JS_FreeValue(ctx, prop_val);
vm_block = 49; continue;
}
// C line 50752
51 => {
let _ = { let assigned = JS_CallFree(ctx, method, (*(s)).handler, (2 as i32), (args).as_mut_ptr()); trap_result_obj = assigned; assigned };
vm_block = 50; continue;
}
// C line 50751
52 => {
let _ = { let assigned = prop_val; *((args).as_mut_ptr()).offset(((1 as i32)) as isize) = assigned; assigned };
vm_block = 51; continue;
}
// C line 50750
53 => {
let _ = { let assigned = (*(s)).target; *((args).as_mut_ptr()).offset(((0 as i32)) as isize) = assigned; assigned };
vm_block = 52; continue;
}
// C line 50748
54 => {
return ((1 as i32)).wrapping_neg();
}
// C line 50747
55 => {
let _ = JS_FreeValue(ctx, method);
vm_block = 54; continue;
}
// C line 50746
56 => {
vm_block = if (JS_IsException(prop_val)) != 0 { 55 } else { 53 }; continue;
}
// C line 50745
57 => {
let _ = { let assigned = JS_AtomToValue(ctx, prop); prop_val = assigned; assigned };
vm_block = 56; continue;
}
// C line 50743
58 => {
return JS_GetOwnPropertyInternal(ctx, pdesc, p, prop);
}
// C line 50742
59 => {
vm_block = if (JS_IsUndefined(method)) != 0 { 58 } else { 57 }; continue;
}
// C line 50741
60 => {
let _ = { let assigned = (((((*(s)).target).u).ptr) as *mut JSObject); p = assigned; assigned };
vm_block = 59; continue;
}
// C line 50740
61 => {
return ((1 as i32)).wrapping_neg();
}
// C line 50739
62 => {
vm_block = if ((!(!(s).is_null()) as i32)) != 0 { 61 } else { 60 }; continue;
}
// C line 50738
63 => {
let _ = { let assigned = get_proxy_method(ctx, core::ptr::addr_of_mut!(method), obj, (((crate::quickjs_atom::JS_ATOM_getOwnPropertyDescriptor as i32)) as JSAtom)); s = assigned; assigned };
vm_block = 62; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:50833. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code, unused_variables)]
unsafe fn js_proxy_define_own_property(mut ctx: *mut JSContext, mut obj: JSValue, mut prop: JSAtom, mut val: JSValue, mut getter: JSValue, mut setter: JSValue, mut flags: i32) -> i32 {
let mut vm_local_storage = Vec::<u64>::new();
let mut s: *mut JSProxyData = core::mem::zeroed();
let mut method: JSValue = core::mem::zeroed();
let mut ret1: JSValue = core::mem::zeroed();
let mut prop_val: JSValue = core::mem::zeroed();
let mut desc_val: JSValue = core::mem::zeroed();
let mut res: i32 = core::mem::zeroed();
let mut ret: i32 = core::mem::zeroed();
let mut p: *mut JSObject = core::mem::zeroed();
let mut args: [JSValue; 3] = core::mem::zeroed();
let mut desc: JSPropertyDescriptor = core::mem::zeroed();
let mut setting_not_configurable: i32 = core::mem::zeroed();
let mut vm_block: usize = 55;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 50927
1 => {
return (1 as i32);
}
// C line 50889
2 => {
vm_block = 6; continue;
}
// C line 50888
3 => {
vm_block = if ((((((!(((*(p)).extensible()) != 0) as i32)) != 0) || ((setting_not_configurable) != 0)) as i32)) != 0 { 2 } else { 1 }; continue;
}
// C line 50925
4 => {
let _ = js_free_desc(ctx, core::ptr::addr_of_mut!(desc));
vm_block = 1; continue;
}
// C line 50923
5 => {
return ((1 as i32)).wrapping_neg();
}
// C line ? labels: fail
6 => {
let _ = JS_ThrowTypeError(ctx, c"proxy: inconsistent defineProperty".as_ptr());
vm_block = 5; continue;
}
// C line ? labels: fail1
7 => {
let _ = js_free_desc(ctx, core::ptr::addr_of_mut!(desc));
vm_block = 6; continue;
}
// C line 50916
8 => {
vm_block = if (((((((((((((((desc).flags) & (((3 as i32)).wrapping_shl(((4 as i32)) as u32)))) != (((1 as i32)).wrapping_shl(((4 as i32)) as u32))) as i32)) != 0) && ((((((((desc).flags) & (((((1 as i32)).wrapping_shl(((0 as i32)) as u32)) | (((1 as i32)).wrapping_shl(((1 as i32)) as u32)))))) == (((1 as i32)).wrapping_shl(((1 as i32)) as u32))) as i32)) != 0)) as i32)) != 0) && (((((((flags) & (((((1 as i32)).wrapping_shl(((9 as i32)) as u32)) | (((1 as i32)).wrapping_shl(((1 as i32)) as u32)))))) == (((1 as i32)).wrapping_shl(((9 as i32)) as u32))) as i32)) != 0)) as i32)) != 0 { 7 } else { 4 }; continue;
}
// C line 50914
9 => {
vm_block = 7; continue;
}
// C line 50913
10 => {
vm_block = if ((((((((desc).flags) & (((1 as i32)).wrapping_shl(((0 as i32)) as u32)))) != 0) && ((setting_not_configurable) != 0)) as i32)) != 0 { 9 } else { 8 }; continue;
}
// C line 50902
11 => {
vm_block = 7; continue;
}
// C line 50900
12 => {
vm_block = if (((((((flags) & (((1 as i32)).wrapping_shl(((12 as i32)) as u32)))) != 0) && (((!((js_same_value(ctx, setter, (desc).setter)) != 0) as i32)) != 0)) as i32)) != 0 { 11 } else { 10 }; continue;
}
// C line 50898
13 => {
vm_block = 7; continue;
}
// C line 50896
14 => {
vm_block = if (((((((flags) & (((1 as i32)).wrapping_shl(((11 as i32)) as u32)))) != 0) && (((!((js_same_value(ctx, getter, (desc).getter)) != 0) as i32)) != 0)) as i32)) != 0 { 13 } else { 12 }; continue;
}
// C line 50907
15 => {
vm_block = 7; continue;
}
// C line 50905
16 => {
vm_block = if (((((((flags) & (((1 as i32)).wrapping_shl(((13 as i32)) as u32)))) != 0) && (((!((js_same_value(ctx, val, (desc).value)) != 0) as i32)) != 0)) as i32)) != 0 { 15 } else { 10 }; continue;
}
// C line 50904
17 => {
vm_block = if ((!(((((desc).flags) & (((1 as i32)).wrapping_shl(((1 as i32)) as u32)))) != 0) as i32)) != 0 { 16 } else { 10 }; continue;
}
// C line 50895
18 => {
vm_block = if (((((((desc).flags) & (((3 as i32)).wrapping_shl(((4 as i32)) as u32)))) == (((1 as i32)).wrapping_shl(((4 as i32)) as u32))) as i32)) != 0 { 14 } else { 17 }; continue;
}
// C line 50894
19 => {
vm_block = if ((!(((((desc).flags) & (((1 as i32)).wrapping_shl(((0 as i32)) as u32)))) != 0) as i32)) != 0 { 18 } else { 10 }; continue;
}
// C line 50892
20 => {
vm_block = 7; continue;
}
// C line 50891
21 => {
vm_block = if ((!((check_define_prop_flags((desc).flags, flags)) != 0) as i32)) != 0 { 20 } else { 19 }; continue;
}
// C line 50887
22 => {
vm_block = if ((!((res) != 0) as i32)) != 0 { 3 } else { 21 }; continue;
}
// C line 50884
23 => {
let _ = { let assigned = (((((flags) & (((((1 as i32)).wrapping_shl(((8 as i32)) as u32)) | (((1 as i32)).wrapping_shl(((0 as i32)) as u32)))))) == (((1 as i32)).wrapping_shl(((8 as i32)) as u32))) as i32); setting_not_configurable = assigned; assigned };
vm_block = 22; continue;
}
// C line 50883
24 => {
return ((1 as i32)).wrapping_neg();
}
// C line 50882
25 => {
vm_block = if ((((res) < ((0 as i32))) as i32)) != 0 { 24 } else { 23 }; continue;
}
// C line 50881
26 => {
let _ = { let assigned = JS_GetOwnPropertyInternal(ctx, core::ptr::addr_of_mut!(desc), p, prop); res = assigned; assigned };
vm_block = 25; continue;
}
// C line 50880
27 => {
let _ = { let assigned = (((((*(s)).target).u).ptr) as *mut JSObject); p = assigned; assigned };
vm_block = 26; continue;
}
// C line 50875
28 => {
return ((1 as i32)).wrapping_neg();
}
// C line 50874
29 => {
let _ = JS_ThrowTypeError(ctx, c"proxy: defineProperty exception".as_ptr());
vm_block = 28; continue;
}
// C line 50877
30 => {
return (0 as i32);
}
// C line 50873
31 => {
vm_block = if (((flags) & (((1 as i32)).wrapping_shl(((14 as i32)) as u32)))) != 0 { 29 } else { 30 }; continue;
}
// C line 50872
32 => {
vm_block = if ((!((ret) != 0) as i32)) != 0 { 31 } else { 27 }; continue;
}
// C line 50871
33 => {
let _ = { let assigned = JS_ToBoolFree(ctx, ret1); ret = assigned; assigned };
vm_block = 32; continue;
}
// C line 50870
34 => {
return ((1 as i32)).wrapping_neg();
}
// C line 50869
35 => {
vm_block = if (JS_IsException(ret1)) != 0 { 34 } else { 33 }; continue;
}
// C line 50868
36 => {
let _ = JS_FreeValue(ctx, desc_val);
vm_block = 35; continue;
}
// C line 50867
37 => {
let _ = JS_FreeValue(ctx, prop_val);
vm_block = 36; continue;
}
// C line 50866
38 => {
let _ = { let assigned = JS_CallFree(ctx, method, (*(s)).handler, (3 as i32), (args).as_mut_ptr()); ret1 = assigned; assigned };
vm_block = 37; continue;
}
// C line 50865
39 => {
let _ = { let assigned = desc_val; *((args).as_mut_ptr()).offset(((2 as i32)) as isize) = assigned; assigned };
vm_block = 38; continue;
}
// C line 50864
40 => {
let _ = { let assigned = prop_val; *((args).as_mut_ptr()).offset(((1 as i32)) as isize) = assigned; assigned };
vm_block = 39; continue;
}
// C line 50863
41 => {
let _ = { let assigned = (*(s)).target; *((args).as_mut_ptr()).offset(((0 as i32)) as isize) = assigned; assigned };
vm_block = 40; continue;
}
// C line 50861
42 => {
return ((1 as i32)).wrapping_neg();
}
// C line 50860
43 => {
let _ = JS_FreeValue(ctx, method);
vm_block = 42; continue;
}
// C line 50859
44 => {
let _ = JS_FreeValue(ctx, prop_val);
vm_block = 43; continue;
}
// C line 50858
45 => {
vm_block = if (JS_IsException(desc_val)) != 0 { 44 } else { 41 }; continue;
}
// C line 50857
46 => {
let _ = { let assigned = js_create_desc(ctx, val, getter, setter, flags); desc_val = assigned; assigned };
vm_block = 45; continue;
}
// C line 50855
47 => {
return ((1 as i32)).wrapping_neg();
}
// C line 50854
48 => {
let _ = JS_FreeValue(ctx, method);
vm_block = 47; continue;
}
// C line 50853
49 => {
vm_block = if (JS_IsException(prop_val)) != 0 { 48 } else { 46 }; continue;
}
// C line 50852
50 => {
let _ = { let assigned = JS_AtomToValue(ctx, prop); prop_val = assigned; assigned };
vm_block = 49; continue;
}
// C line 50850
51 => {
return JS_DefineProperty(ctx, (*(s)).target, prop, val, getter, setter, flags);
}
// C line 50849
52 => {
vm_block = if (JS_IsUndefined(method)) != 0 { 51 } else { 50 }; continue;
}
// C line 50848
53 => {
return ((1 as i32)).wrapping_neg();
}
// C line 50847
54 => {
vm_block = if ((!(!(s).is_null()) as i32)) != 0 { 53 } else { 52 }; continue;
}
// C line 50846
55 => {
let _ = { let assigned = get_proxy_method(ctx, core::ptr::addr_of_mut!(method), obj, (((crate::quickjs_atom::JS_ATOM_defineProperty as i32)) as JSAtom)); s = assigned; assigned };
vm_block = 54; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:50930. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code, unused_variables)]
unsafe fn js_proxy_delete_property(mut ctx: *mut JSContext, mut obj: JSValue, mut atom: JSAtom) -> i32 {
let mut vm_local_storage = Vec::<u64>::new();
let mut s: *mut JSProxyData = core::mem::zeroed();
let mut method: JSValue = core::mem::zeroed();
let mut ret: JSValue = core::mem::zeroed();
let mut atom_val: JSValue = core::mem::zeroed();
let mut res: i32 = core::mem::zeroed();
let mut res2: i32 = core::mem::zeroed();
let mut is_extensible: i32 = core::mem::zeroed();
let mut args: [JSValue; 2] = core::mem::zeroed();
let mut desc: JSPropertyDescriptor = core::mem::zeroed();
let mut vm_block: usize = 32;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 50978
1 => {
return res;
}
// C line 50975
2 => {
let _ = js_free_desc(ctx, core::ptr::addr_of_mut!(desc));
vm_block = 1; continue;
}
// C line 50973
3 => {
return ((1 as i32)).wrapping_neg();
}
// C line ? labels: fail1
4 => {
let _ = js_free_desc(ctx, core::ptr::addr_of_mut!(desc));
vm_block = 3; continue;
}
// C line ? labels: fail
5 => {
let _ = JS_ThrowTypeError(ctx, c"proxy: inconsistent deleteProperty".as_ptr());
vm_block = 4; continue;
}
// C line 50967
6 => {
vm_block = if ((!((is_extensible) != 0) as i32)) != 0 { 5 } else { 2 }; continue;
}
// C line 50966
7 => {
vm_block = 4; continue;
}
// C line 50965
8 => {
vm_block = if ((((is_extensible) < ((0 as i32))) as i32)) != 0 { 7 } else { 6 }; continue;
}
// C line 50964
9 => {
let _ = { let assigned = JS_IsExtensible(ctx, (*(s)).target); is_extensible = assigned; assigned };
vm_block = 8; continue;
}
// C line 50963
10 => {
vm_block = 5; continue;
}
// C line 50962
11 => {
vm_block = if ((!(((((desc).flags) & (((1 as i32)).wrapping_shl(((0 as i32)) as u32)))) != 0) as i32)) != 0 { 10 } else { 9 }; continue;
}
// C line 50961
12 => {
vm_block = if (res2) != 0 { 11 } else { 1 }; continue;
}
// C line 50960
13 => {
return ((1 as i32)).wrapping_neg();
}
// C line 50959
14 => {
vm_block = if ((((res2) < ((0 as i32))) as i32)) != 0 { 13 } else { 12 }; continue;
}
// C line 50958
15 => {
let _ = { let assigned = JS_GetOwnPropertyInternal(ctx, core::ptr::addr_of_mut!(desc), (((((*(s)).target).u).ptr) as *mut JSObject), atom); res2 = assigned; assigned };
vm_block = 14; continue;
}
// C line 50956
16 => {
vm_block = if (res) != 0 { 15 } else { 1 }; continue;
}
// C line 50955
17 => {
let _ = { let assigned = JS_ToBoolFree(ctx, ret); res = assigned; assigned };
vm_block = 16; continue;
}
// C line 50954
18 => {
return ((1 as i32)).wrapping_neg();
}
// C line 50953
19 => {
vm_block = if (JS_IsException(ret)) != 0 { 18 } else { 17 }; continue;
}
// C line 50952
20 => {
let _ = JS_FreeValue(ctx, atom_val);
vm_block = 19; continue;
}
// C line 50951
21 => {
let _ = { let assigned = JS_CallFree(ctx, method, (*(s)).handler, (2 as i32), (args).as_mut_ptr()); ret = assigned; assigned };
vm_block = 20; continue;
}
// C line 50950
22 => {
let _ = { let assigned = atom_val; *((args).as_mut_ptr()).offset(((1 as i32)) as isize) = assigned; assigned };
vm_block = 21; continue;
}
// C line 50949
23 => {
let _ = { let assigned = (*(s)).target; *((args).as_mut_ptr()).offset(((0 as i32)) as isize) = assigned; assigned };
vm_block = 22; continue;
}
// C line 50947
24 => {
return ((1 as i32)).wrapping_neg();
}
// C line 50946
25 => {
let _ = JS_FreeValue(ctx, method);
vm_block = 24; continue;
}
// C line 50945
26 => {
vm_block = if (JS_IsException(atom_val)) != 0 { 25 } else { 23 }; continue;
}
// C line 50944
27 => {
let _ = { let assigned = JS_AtomToValue(ctx, atom); atom_val = assigned; assigned };
vm_block = 26; continue;
}
// C line 50942
28 => {
return JS_DeleteProperty(ctx, (*(s)).target, atom, (0 as i32));
}
// C line 50941
29 => {
vm_block = if (JS_IsUndefined(method)) != 0 { 28 } else { 27 }; continue;
}
// C line 50940
30 => {
return ((1 as i32)).wrapping_neg();
}
// C line 50939
31 => {
vm_block = if ((!(!(s).is_null()) as i32)) != 0 { 30 } else { 29 }; continue;
}
// C line 50938
32 => {
let _ = { let assigned = get_proxy_method(ctx, core::ptr::addr_of_mut!(method), obj, (((crate::quickjs_atom::JS_ATOM_deleteProperty as i32)) as JSAtom)); s = assigned; assigned };
vm_block = 31; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:50982. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code, unused_variables)]
unsafe fn find_prop_key(mut tab: *const JSPropertyEnum, mut n: i32, mut atom: JSAtom) -> i32 {
let mut vm_local_storage = Vec::<u64>::new();
let mut i: i32 = core::mem::zeroed();
let mut vm_block: usize = 6;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 50989
1 => {
return ((1 as i32)).wrapping_neg();
}
// C line 50985
2 => {
vm_block = if ((((i) < (n)) as i32)) != 0 { 5 } else { 1 }; continue;
}
// C line ?
3 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 2; continue;
}
// C line 50987
4 => {
return i;
}
// C line 50986
5 => {
vm_block = if (((((*(tab).offset((i) as isize)).atom) == (atom)) as i32)) != 0 { 4 } else { 3 }; continue;
}
// C line 50985
6 => {
let _ = { let assigned = (0 as i32); i = assigned; assigned };
vm_block = 2; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:50992. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code, unused_variables)]
unsafe fn js_proxy_get_own_property_names(mut ctx: *mut JSContext, mut ptab: *mut *mut JSPropertyEnum, mut plen: *mut u32, mut obj: JSValue) -> i32 {
let mut vm_local_storage = Vec::<u64>::new();
let mut s: *mut JSProxyData = core::mem::zeroed();
let mut method: JSValue = core::mem::zeroed();
let mut prop_array: JSValue = core::mem::zeroed();
let mut val: JSValue = core::mem::zeroed();
let mut len: u32 = core::mem::zeroed();
let mut i: u32 = core::mem::zeroed();
let mut len2: u32 = core::mem::zeroed();
let mut tab: *mut JSPropertyEnum = core::mem::zeroed();
let mut tab2: *mut JSPropertyEnum = core::mem::zeroed();
let mut atom: JSAtom = core::mem::zeroed();
let mut desc: JSPropertyDescriptor = core::mem::zeroed();
let mut res: i32 = core::mem::zeroed();
let mut is_extensible: i32 = core::mem::zeroed();
let mut idx: i32 = core::mem::zeroed();
let mut vm_block: usize = 82;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 51107
1 => {
return ((1 as i32)).wrapping_neg();
}
// C line 51106
2 => {
let _ = JS_FreeValue(ctx, prop_array);
vm_block = 1; continue;
}
// C line 51105
3 => {
let _ = JS_FreePropertyEnum(ctx, tab, len);
vm_block = 2; continue;
}
// C line ? labels: fail
4 => {
let _ = JS_FreePropertyEnum(ctx, tab2, len2);
vm_block = 3; continue;
}
// C line 51102
5 => {
return (0 as i32);
}
// C line 51101
6 => {
let _ = { let assigned = len; *(plen) = assigned; assigned };
vm_block = 5; continue;
}
// C line 51100
7 => {
let _ = { let assigned = tab; *(ptab) = assigned; assigned };
vm_block = 6; continue;
}
// C line 51099
8 => {
let _ = JS_FreeValue(ctx, prop_array);
vm_block = 7; continue;
}
// C line 51098
9 => {
let _ = JS_FreePropertyEnum(ctx, tab2, len2);
vm_block = 8; continue;
}
// C line 51090
10 => {
vm_block = if ((((i) < (len)) as i32)) != 0 { 14 } else { 9 }; continue;
}
// C line ?
11 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 10; continue;
}
// C line 51093
12 => {
vm_block = 4; continue;
}
// C line 51092
13 => {
let _ = JS_ThrowTypeError(ctx, c"proxy: property not present in target were returned by non extensible proxy".as_ptr());
vm_block = 12; continue;
}
// C line 51091
14 => {
vm_block = if ((!(((*(tab).offset((i) as isize)).is_enumerable) != 0) as i32)) != 0 { 13 } else { 11 }; continue;
}
// C line 51090
15 => {
let _ = { let assigned = (((0 as i32)) as u32); i = assigned; assigned };
vm_block = 10; continue;
}
// C line 51088
16 => {
vm_block = if ((!((is_extensible) != 0) as i32)) != 0 { 15 } else { 9 }; continue;
}
// C line 51065
17 => {
vm_block = if ((((i) < (len2)) as i32)) != 0 { 33 } else { 16 }; continue;
}
// C line ?
18 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 17; continue;
}
// C line 51084
19 => {
let _ = { let assigned = (1 as i32); (*(tab).offset((idx) as isize)).is_enumerable = assigned; assigned };
vm_block = 18; continue;
}
// C line 51083
20 => {
vm_block = if ((!((is_extensible) != 0) as i32)) != 0 { 19 } else { 18 }; continue;
}
// C line 51080
21 => {
vm_block = 4; continue;
}
// C line 51079
22 => {
let _ = JS_ThrowTypeError(ctx, c"proxy: target property must be present in proxy ownKeys".as_ptr());
vm_block = 21; continue;
}
// C line 51078
23 => {
vm_block = if ((((idx) < ((0 as i32))) as i32)) != 0 { 22 } else { 20 }; continue;
}
// C line 51077
24 => {
let _ = { let assigned = find_prop_key(tab, ((len) as i32), (*(tab2).offset((i) as isize)).atom); idx = assigned; assigned };
vm_block = 23; continue;
}
// C line 51076
25 => {
vm_block = if ((((((!(((((desc).flags) & (((1 as i32)).wrapping_shl(((0 as i32)) as u32)))) != 0) as i32)) != 0) || (((!((is_extensible) != 0) as i32)) != 0)) as i32)) != 0 { 24 } else { 18 }; continue;
}
// C line 51075
26 => {
let _ = js_free_desc(ctx, core::ptr::addr_of_mut!(desc));
vm_block = 25; continue;
}
// C line 51074
27 => {
vm_block = if (res) != 0 { 26 } else { 18 }; continue;
}
// C line 51073
28 => {
vm_block = 4; continue;
}
// C line 51072
29 => {
vm_block = if ((((res) < ((0 as i32))) as i32)) != 0 { 28 } else { 27 }; continue;
}
// C line 51070
30 => {
let _ = { let assigned = JS_GetOwnPropertyInternal(ctx, core::ptr::addr_of_mut!(desc), (((((*(s)).target).u).ptr) as *mut JSObject), (*(tab2).offset((i) as isize)).atom); res = assigned; assigned };
vm_block = 29; continue;
}
// C line 51068
31 => {
vm_block = 4; continue;
}
// C line 51067
32 => {
let _ = JS_ThrowTypeErrorRevokedProxy(ctx);
vm_block = 31; continue;
}
// C line 51066
33 => {
vm_block = if ((*(s)).is_revoked) != 0 { 32 } else { 30 }; continue;
}
// C line 51065
34 => {
let _ = { let assigned = (((0 as i32)) as u32); i = assigned; assigned };
vm_block = 17; continue;
}
// C line 51064
35 => {
vm_block = 4; continue;
}
// C line 51062
36 => {
vm_block = if (JS_GetOwnPropertyNamesInternal(ctx, core::ptr::addr_of_mut!(tab2), core::ptr::addr_of_mut!(len2), (((((*(s)).target).u).ptr) as *mut JSObject), ((((1 as i32)).wrapping_shl(((0 as i32)) as u32)) | (((1 as i32)).wrapping_shl(((1 as i32)) as u32))))) != 0 { 35 } else { 34 }; continue;
}
// C line 51060
37 => {
vm_block = 4; continue;
}
// C line 51059
38 => {
let _ = JS_ThrowTypeErrorRevokedProxy(ctx);
vm_block = 37; continue;
}
// C line 51058
39 => {
vm_block = if ((*(s)).is_revoked) != 0 { 38 } else { 36 }; continue;
}
// C line 51055
40 => {
vm_block = 4; continue;
}
// C line 51054
41 => {
vm_block = if ((((is_extensible) < ((0 as i32))) as i32)) != 0 { 40 } else { 39 }; continue;
}
// C line 51053
42 => {
let _ = { let assigned = JS_IsExtensible(ctx, (*(s)).target); is_extensible = assigned; assigned };
vm_block = 41; continue;
}
// C line 51046
43 => {
vm_block = if ((((i) < (len)) as i32)) != 0 { 47 } else { 42 }; continue;
}
// C line ?
44 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 43; continue;
}
// C line 51049
45 => {
vm_block = 4; continue;
}
// C line 51048
46 => {
let _ = JS_ThrowTypeError(ctx, c"proxy: duplicate property".as_ptr());
vm_block = 45; continue;
}
// C line 51047
47 => {
vm_block = if ((((find_prop_key(tab, ((i) as i32), (*(tab).offset((i) as isize)).atom)) >= ((0 as i32))) as i32)) != 0 { 46 } else { 44 }; continue;
}
// C line 51046
48 => {
let _ = { let assigned = (((1 as i32)) as u32); i = assigned; assigned };
vm_block = 43; continue;
}
// C line 51027
49 => {
vm_block = if ((((i) < (len)) as i32)) != 0 { 63 } else { 48 }; continue;
}
// C line ?
50 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 49; continue;
}
// C line 51041
51 => {
let _ = { let assigned = (0 as i32); (*(tab).offset((i) as isize)).is_enumerable = assigned; assigned };
vm_block = 50; continue;
}
// C line 51040
52 => {
let _ = { let assigned = atom; (*(tab).offset((i) as isize)).atom = assigned; assigned };
vm_block = 51; continue;
}
// C line 51039
53 => {
vm_block = 4; continue;
}
// C line 51038
54 => {
vm_block = if ((((atom) == ((((0 as i32)) as JSAtom))) as i32)) != 0 { 53 } else { 52 }; continue;
}
// C line 51037
55 => {
let _ = JS_FreeValue(ctx, val);
vm_block = 54; continue;
}
// C line 51036
56 => {
let _ = { let assigned = JS_ValueToAtom(ctx, val); atom = assigned; assigned };
vm_block = 55; continue;
}
// C line 51034
57 => {
vm_block = 4; continue;
}
// C line 51033
58 => {
let _ = JS_ThrowTypeError(ctx, c"proxy: properties must be strings or symbols".as_ptr());
vm_block = 57; continue;
}
// C line 51032
59 => {
let _ = JS_FreeValue(ctx, val);
vm_block = 58; continue;
}
// C line 51031
60 => {
vm_block = if ((((((!((JS_IsString(val)) != 0) as i32)) != 0) && (((!((JS_IsSymbol(val)) != 0) as i32)) != 0)) as i32)) != 0 { 59 } else { 56 }; continue;
}
// C line 51030
61 => {
vm_block = 4; continue;
}
// C line 51029
62 => {
vm_block = if (JS_IsException(val)) != 0 { 61 } else { 60 }; continue;
}
// C line 51028
63 => {
let _ = { let assigned = JS_GetPropertyUint32(ctx, prop_array, i); val = assigned; assigned };
vm_block = 62; continue;
}
// C line 51027
64 => {
let _ = { let assigned = (((0 as i32)) as u32); i = assigned; assigned };
vm_block = 49; continue;
}
// C line 51025
65 => {
vm_block = 4; continue;
}
// C line 51024
66 => {
vm_block = if ((!(!(tab).is_null()) as i32)) != 0 { 65 } else { 64 }; continue;
}
// C line 51023
67 => {
let _ = { let assigned = ((js_mallocz(ctx, ((size_of::<JSPropertyEnum>() as usize)).wrapping_mul(((len) as usize)))) as *mut JSPropertyEnum); tab = assigned; assigned };
vm_block = 66; continue;
}
// C line 51022
68 => {
vm_block = if ((((len) > ((((0 as i32)) as u32))) as i32)) != 0 { 67 } else { 64 }; continue;
}
// C line 51021
69 => {
vm_block = 4; continue;
}
// C line 51020
70 => {
vm_block = if (js_get_length32(ctx, core::ptr::addr_of_mut!(len), prop_array)) != 0 { 69 } else { 68 }; continue;
}
// C line 51019
71 => {
let _ = { let assigned = (((0 as i32)) as u32); len2 = assigned; assigned };
vm_block = 70; continue;
}
// C line 51018
72 => {
let _ = { let assigned = core::ptr::null_mut::<JSPropertyEnum>(); tab2 = assigned; assigned };
vm_block = 71; continue;
}
// C line 51017
73 => {
let _ = { let assigned = (((0 as i32)) as u32); len = assigned; assigned };
vm_block = 72; continue;
}
// C line 51016
74 => {
let _ = { let assigned = core::ptr::null_mut::<JSPropertyEnum>(); tab = assigned; assigned };
vm_block = 73; continue;
}
// C line 51015
75 => {
return ((1 as i32)).wrapping_neg();
}
// C line 51014
76 => {
vm_block = if (JS_IsException(prop_array)) != 0 { 75 } else { 74 }; continue;
}
// C line 51013
77 => {
let _ = { let assigned = JS_CallFree(ctx, method, (*(s)).handler, (1 as i32), core::ptr::addr_of_mut!((*(s)).target)); prop_array = assigned; assigned };
vm_block = 76; continue;
}
// C line 51009
78 => {
return JS_GetOwnPropertyNamesInternal(ctx, ptab, plen, (((((*(s)).target).u).ptr) as *mut JSObject), ((((1 as i32)).wrapping_shl(((0 as i32)) as u32)) | (((1 as i32)).wrapping_shl(((1 as i32)) as u32))));
}
// C line 51008
79 => {
vm_block = if (JS_IsUndefined(method)) != 0 { 78 } else { 77 }; continue;
}
// C line 51007
80 => {
return ((1 as i32)).wrapping_neg();
}
// C line 51006
81 => {
vm_block = if ((!(!(s).is_null()) as i32)) != 0 { 80 } else { 79 }; continue;
}
// C line 51005
82 => {
let _ = { let assigned = get_proxy_method(ctx, core::ptr::addr_of_mut!(method), obj, (((crate::quickjs_atom::JS_ATOM_ownKeys as i32)) as JSAtom)); s = assigned; assigned };
vm_block = 81; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:51110. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code, unused_variables)]
unsafe fn js_proxy_call_constructor(mut ctx: *mut JSContext, mut func_obj: JSValue, mut new_target: JSValue, mut argc: i32, mut argv: *mut JSValue) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut s: *mut JSProxyData = core::mem::zeroed();
let mut method: JSValue = core::mem::zeroed();
let mut arg_array: JSValue = core::mem::zeroed();
let mut ret: JSValue = core::mem::zeroed();
let mut args: [JSValue; 3] = core::mem::zeroed();
let mut vm_block: usize = 21;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 51141
1 => {
return ret;
}
// C line 51140
2 => {
let _ = JS_FreeValue(ctx, arg_array);
vm_block = 1; continue;
}
// C line ? labels: fail
3 => {
let _ = JS_FreeValue(ctx, method);
vm_block = 2; continue;
}
// C line 51136
4 => {
let _ = { let assigned = JS_ThrowTypeErrorNotAnObject(ctx); ret = assigned; assigned };
vm_block = 3; continue;
}
// C line 51135
5 => {
let _ = JS_FreeValue(ctx, ret);
vm_block = 4; continue;
}
// C line 51134
6 => {
vm_block = if ((((((!((JS_IsException(ret)) != 0) as i32)) != 0) && ((((((((ret).tag) as i32)) != ((JS_TAG_OBJECT as i32))) as i32)) != 0)) as i32)) != 0 { 5 } else { 3 }; continue;
}
// C line 51133
7 => {
let _ = { let assigned = JS_Call(ctx, method, (*(s)).handler, (3 as i32), (args).as_mut_ptr()); ret = assigned; assigned };
vm_block = 6; continue;
}
// C line 51132
8 => {
let _ = { let assigned = new_target; *((args).as_mut_ptr()).offset(((2 as i32)) as isize) = assigned; assigned };
vm_block = 7; continue;
}
// C line 51131
9 => {
let _ = { let assigned = arg_array; *((args).as_mut_ptr()).offset(((1 as i32)) as isize) = assigned; assigned };
vm_block = 8; continue;
}
// C line 51130
10 => {
let _ = { let assigned = (*(s)).target; *((args).as_mut_ptr()).offset(((0 as i32)) as isize) = assigned; assigned };
vm_block = 9; continue;
}
// C line 51128
11 => {
vm_block = 3; continue;
}
// C line 51127
12 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) }; ret = assigned; assigned };
vm_block = 11; continue;
}
// C line 51126
13 => {
vm_block = if (JS_IsException(arg_array)) != 0 { 12 } else { 10 }; continue;
}
// C line 51125
14 => {
let _ = { let assigned = js_create_array(ctx, argc, argv); arg_array = assigned; assigned };
vm_block = 13; continue;
}
// C line 51124
15 => {
return JS_CallConstructor2(ctx, (*(s)).target, new_target, argc, argv);
}
// C line 51123
16 => {
vm_block = if (JS_IsUndefined(method)) != 0 { 15 } else { 14 }; continue;
}
// C line 51122
17 => {
return JS_ThrowTypeErrorNotAConstructor(ctx, (*(s)).target);
}
// C line 51121
18 => {
vm_block = if ((!((JS_IsConstructor(ctx, (*(s)).target)) != 0) as i32)) != 0 { 17 } else { 16 }; continue;
}
// C line 51120
19 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 51119
20 => {
vm_block = if ((!(!(s).is_null()) as i32)) != 0 { 19 } else { 18 }; continue;
}
// C line 51118
21 => {
let _ = { let assigned = get_proxy_method(ctx, core::ptr::addr_of_mut!(method), func_obj, (((crate::quickjs_atom::JS_ATOM_construct as i32)) as JSAtom)); s = assigned; assigned };
vm_block = 20; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:51144. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code, unused_variables)]
unsafe fn js_proxy_call(mut ctx: *mut JSContext, mut func_obj: JSValue, mut this_obj: JSValue, mut argc: i32, mut argv: *mut JSValue, mut flags: i32) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut s: *mut JSProxyData = core::mem::zeroed();
let mut method: JSValue = core::mem::zeroed();
let mut arg_array: JSValue = core::mem::zeroed();
let mut ret: JSValue = core::mem::zeroed();
let mut args: [JSValue; 3] = core::mem::zeroed();
let mut vm_block: usize = 21;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 51176
1 => {
return ret;
}
// C line 51175
2 => {
let _ = JS_FreeValue(ctx, arg_array);
vm_block = 1; continue;
}
// C line ? labels: fail
3 => {
let _ = JS_FreeValue(ctx, method);
vm_block = 2; continue;
}
// C line 51172
4 => {
let _ = { let assigned = JS_Call(ctx, method, (*(s)).handler, (3 as i32), (args).as_mut_ptr()); ret = assigned; assigned };
vm_block = 3; continue;
}
// C line 51171
5 => {
let _ = { let assigned = arg_array; *((args).as_mut_ptr()).offset(((2 as i32)) as isize) = assigned; assigned };
vm_block = 4; continue;
}
// C line 51170
6 => {
let _ = { let assigned = this_obj; *((args).as_mut_ptr()).offset(((1 as i32)) as isize) = assigned; assigned };
vm_block = 5; continue;
}
// C line 51169
7 => {
let _ = { let assigned = (*(s)).target; *((args).as_mut_ptr()).offset(((0 as i32)) as isize) = assigned; assigned };
vm_block = 6; continue;
}
// C line 51167
8 => {
vm_block = 3; continue;
}
// C line 51166
9 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) }; ret = assigned; assigned };
vm_block = 8; continue;
}
// C line 51165
10 => {
vm_block = if (JS_IsException(arg_array)) != 0 { 9 } else { 7 }; continue;
}
// C line 51164
11 => {
let _ = { let assigned = js_create_array(ctx, argc, argv); arg_array = assigned; assigned };
vm_block = 10; continue;
}
// C line 51163
12 => {
return JS_Call(ctx, (*(s)).target, this_obj, argc, argv);
}
// C line 51162
13 => {
vm_block = if (JS_IsUndefined(method)) != 0 { 12 } else { 11 }; continue;
}
// C line 51160
14 => {
return JS_ThrowTypeError(ctx, c"not a function".as_ptr());
}
// C line 51159
15 => {
let _ = JS_FreeValue(ctx, method);
vm_block = 14; continue;
}
// C line 51158
16 => {
vm_block = if ((!(((*(s)).is_func) != 0) as i32)) != 0 { 15 } else { 13 }; continue;
}
// C line 51157
17 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 51156
18 => {
vm_block = if ((!(!(s).is_null()) as i32)) != 0 { 17 } else { 16 }; continue;
}
// C line 51155
19 => {
let _ = { let assigned = get_proxy_method(ctx, core::ptr::addr_of_mut!(method), func_obj, (((crate::quickjs_atom::JS_ATOM_apply as i32)) as JSAtom)); s = assigned; assigned };
vm_block = 18; continue;
}
// C line 51153
20 => {
return js_proxy_call_constructor(ctx, func_obj, this_obj, argc, argv);
}
// C line 51152
21 => {
vm_block = if (((flags) & (((1 as i32)).wrapping_shl(((0 as i32)) as u32)))) != 0 { 20 } else { 19 }; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:51224. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code, unused_variables)]
unsafe fn js_proxy_constructor(mut ctx: *mut JSContext, mut this_val: JSValue, mut argc: i32, mut argv: *mut JSValue) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut target: JSValue = core::mem::zeroed();
let mut handler: JSValue = core::mem::zeroed();
let mut obj: JSValue = core::mem::zeroed();
let mut s: *mut JSProxyData = core::mem::zeroed();
let mut vm_block: usize = 18;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 51251
1 => {
return obj;
}
// C line 51250
2 => {
let _ = JS_SetConstructorBit(ctx, obj, JS_IsConstructor(ctx, target));
vm_block = 1; continue;
}
// C line 51249
3 => {
let _ = JS_SetOpaque(obj, ((s) as *mut c_void));
vm_block = 2; continue;
}
// C line 51248
4 => {
let _ = { let assigned = (((0 as i32)) as u8); (*(s)).is_revoked = assigned; assigned };
vm_block = 3; continue;
}
// C line 51247
5 => {
let _ = { let assigned = ((JS_IsFunction(ctx, target)) as u8); (*(s)).is_func = assigned; assigned };
vm_block = 4; continue;
}
// C line 51246
6 => {
let _ = { let assigned = JS_DupValue(ctx, handler); (*(s)).handler = assigned; assigned };
vm_block = 5; continue;
}
// C line 51245
7 => {
let _ = { let assigned = JS_DupValue(ctx, target); (*(s)).target = assigned; assigned };
vm_block = 6; continue;
}
// C line 51243
8 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 51242
9 => {
let _ = JS_FreeValue(ctx, obj);
vm_block = 8; continue;
}
// C line 51241
10 => {
vm_block = if ((!(!(s).is_null()) as i32)) != 0 { 9 } else { 7 }; continue;
}
// C line 51240
11 => {
let _ = { let assigned = ((js_malloc(ctx, (size_of::<JSProxyData>() as usize))) as *mut JSProxyData); s = assigned; assigned };
vm_block = 10; continue;
}
// C line 51239
12 => {
return obj;
}
// C line 51238
13 => {
vm_block = if (JS_IsException(obj)) != 0 { 12 } else { 11 }; continue;
}
// C line 51237
14 => {
let _ = { let assigned = JS_NewObjectProtoClass(ctx, JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_NULL as i32)) as i64) }, (((JS_CLASS_PROXY as i32)) as JSClassID)); obj = assigned; assigned };
vm_block = 13; continue;
}
// C line 51235
15 => {
return JS_ThrowTypeErrorNotAnObject(ctx);
}
// C line 51233
16 => {
vm_block = if (((((((((((target).tag) as i32)) != ((JS_TAG_OBJECT as i32))) as i32)) != 0) || ((((((((handler).tag) as i32)) != ((JS_TAG_OBJECT as i32))) as i32)) != 0)) as i32)) != 0 { 15 } else { 14 }; continue;
}
// C line 51232
17 => {
let _ = { let assigned = *(argv).offset(((1 as i32)) as isize); handler = assigned; assigned };
vm_block = 16; continue;
}
// C line 51231
18 => {
let _ = { let assigned = *(argv).offset(((0 as i32)) as isize); target = assigned; assigned };
vm_block = 17; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:51254. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code, unused_variables)]
unsafe fn js_proxy_revoke(mut ctx: *mut JSContext, mut this_val: JSValue, mut argc: i32, mut argv: *mut JSValue, mut magic: i32, mut func_data: *mut JSValue) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut s: *mut JSProxyData = core::mem::zeroed();
let mut vm_block: usize = 6;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 51266
1 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) };
}
// C line 51264
2 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_NULL as i32)) as i64) }; *(func_data).offset(((0 as i32)) as isize) = assigned; assigned };
vm_block = 1; continue;
}
// C line 51263
3 => {
let _ = JS_FreeValue(ctx, *(func_data).offset(((0 as i32)) as isize));
vm_block = 2; continue;
}
// C line 51262
4 => {
let _ = { let assigned = (((1 as i32)) as u8); (*(s)).is_revoked = assigned; assigned };
vm_block = 3; continue;
}
// C line 51259
5 => {
vm_block = if !(s).is_null() { 4 } else { 1 }; continue;
}
// C line 51258
6 => {
s = ((JS_GetOpaque(*(func_data).offset(((0 as i32)) as isize), (((JS_CLASS_PROXY as i32)) as JSClassID))) as *mut JSProxyData);
vm_block = 5; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:51269. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code, unused_variables)]
unsafe fn js_proxy_revoke_constructor(mut ctx: *mut JSContext, mut proxy_obj: JSValue) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut vm_block: usize = 1;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 51272
1 => {
return JS_NewCFunctionData(ctx, Some(js_proxy_revoke), (0 as i32), (0 as i32), (1 as i32), core::ptr::addr_of_mut!(proxy_obj));
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:51275. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code, unused_variables)]
unsafe fn js_proxy_revocable(mut ctx: *mut JSContext, mut this_val: JSValue, mut argc: i32, mut argv: *mut JSValue) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut proxy_obj: JSValue = core::mem::zeroed();
let mut revoke_obj: JSValue = core::mem::zeroed();
let mut obj: JSValue = core::mem::zeroed();
let mut vm_block: usize = 16;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 51296
1 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 51295
2 => {
let _ = JS_FreeValue(ctx, revoke_obj);
vm_block = 1; continue;
}
// C line ? labels: fail
3 => {
let _ = JS_FreeValue(ctx, proxy_obj);
vm_block = 2; continue;
}
// C line 51292
4 => {
return obj;
}
// C line 51291
5 => {
let _ = JS_DefinePropertyValue(ctx, obj, (((crate::quickjs_atom::JS_ATOM_revoke as i32)) as JSAtom), revoke_obj, ((((((1 as i32)).wrapping_shl(((0 as i32)) as u32)) | (((1 as i32)).wrapping_shl(((1 as i32)) as u32)))) | (((1 as i32)).wrapping_shl(((2 as i32)) as u32))));
vm_block = 4; continue;
}
// C line 51290
6 => {
let _ = JS_DefinePropertyValue(ctx, obj, (((crate::quickjs_atom::JS_ATOM_proxy as i32)) as JSAtom), proxy_obj, ((((((1 as i32)).wrapping_shl(((0 as i32)) as u32)) | (((1 as i32)).wrapping_shl(((1 as i32)) as u32)))) | (((1 as i32)).wrapping_shl(((2 as i32)) as u32))));
vm_block = 5; continue;
}
// C line 51288
7 => {
vm_block = 3; continue;
}
// C line 51287
8 => {
vm_block = if (JS_IsException(obj)) != 0 { 7 } else { 6 }; continue;
}
// C line 51286
9 => {
let _ = { let assigned = JS_NewObject(ctx); obj = assigned; assigned };
vm_block = 8; continue;
}
// C line 51285
10 => {
vm_block = 3; continue;
}
// C line 51284
11 => {
vm_block = if (JS_IsException(revoke_obj)) != 0 { 10 } else { 9 }; continue;
}
// C line 51283
12 => {
let _ = { let assigned = js_proxy_revoke_constructor(ctx, proxy_obj); revoke_obj = assigned; assigned };
vm_block = 11; continue;
}
// C line 51282
13 => {
vm_block = 3; continue;
}
// C line 51281
14 => {
vm_block = if (JS_IsException(proxy_obj)) != 0 { 13 } else { 12 }; continue;
}
// C line 51280
15 => {
let _ = { let assigned = js_proxy_constructor(ctx, JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }, argc, argv); proxy_obj = assigned; assigned };
vm_block = 14; continue;
}
// C line 51278
16 => {
revoke_obj = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) };
vm_block = 15; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:51307. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code, unused_variables)]
pub unsafe fn JS_AddIntrinsicProxy(mut ctx: *mut JSContext) -> i32 {
let mut vm_local_storage = Vec::<u64>::new();
let mut rt: *mut JSRuntime = core::mem::zeroed();
let mut obj1: JSValue = core::mem::zeroed();
let mut vm_block: usize = 17;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 51336
1 => {
return ((1 as i32)).wrapping_neg();
}
// C line ? labels: fail
2 => {
let _ = JS_FreeValue(ctx, obj1);
vm_block = 1; continue;
}
// C line 51333
3 => {
return (0 as i32);
}
// C line 51332
4 => {
vm_block = 2; continue;
}
// C line 51330
5 => {
vm_block = if ((((JS_DefinePropertyValueStr(ctx, (*(ctx)).global_obj, c"Proxy".as_ptr(), obj1, ((((1 as i32)).wrapping_shl(((1 as i32)) as u32)) | (((1 as i32)).wrapping_shl(((0 as i32)) as u32))))) < ((0 as i32))) as i32)) != 0 { 4 } else { 3 }; continue;
}
// C line 51329
6 => {
vm_block = 2; continue;
}
// C line 51327
7 => {
vm_block = if (JS_SetPropertyFunctionList(ctx, obj1, (js_proxy_funcs).as_ptr(), (((((size_of::<[JSCFunctionListEntry; 1]>() as usize)) / ((size_of::<JSCFunctionListEntry>() as usize)))) as i32))) != 0 { 6 } else { 5 }; continue;
}
// C line 51326
8 => {
let _ = JS_SetConstructorBit(ctx, obj1, (1 as i32));
vm_block = 7; continue;
}
// C line 51325
9 => {
return ((1 as i32)).wrapping_neg();
}
// C line 51324
10 => {
vm_block = if (JS_IsException(obj1)) != 0 { 9 } else { 8 }; continue;
}
// C line 51321
11 => {
let _ = { let assigned = JS_NewCFunction3(ctx, Some(js_proxy_constructor), c"Proxy".as_ptr(), (2 as i32), (((JS_CFUNC_constructor as i32)) as JSCFunctionEnum), (0 as i32), (*(ctx)).function_proto, ((((((size_of::<[JSCFunctionListEntry; 1]>() as usize)) / ((size_of::<JSCFunctionListEntry>() as usize)))).wrapping_add((((2 as i32)) as usize))) as i32)); obj1 = assigned; assigned };
vm_block = 10; continue;
}
// C line 51317
12 => {
let _ = { let assigned = js_proxy_call; (*((*(rt)).class_array).offset(((JS_CLASS_PROXY as i32)) as isize)).call = Some(assigned); assigned };
vm_block = 11; continue;
}
// C line 51316
13 => {
let _ = { let assigned = core::ptr::addr_of_mut!(js_proxy_exotic_methods); (*((*(rt)).class_array).offset(((JS_CLASS_PROXY as i32)) as isize)).exotic = assigned; assigned };
vm_block = 12; continue;
}
// C line 51315
14 => {
return ((1 as i32)).wrapping_neg();
}
// C line 51313
15 => {
vm_block = if (init_class_range(rt, (js_proxy_class_def).as_ptr(), (JS_CLASS_PROXY as i32), (((((size_of::<[JSClassShortDef; 1]>() as usize)) / ((size_of::<JSClassShortDef>() as usize)))) as i32))) != 0 { 14 } else { 13 }; continue;
}
// C line 51312
16 => {
vm_block = if ((!((JS_IsRegisteredClass(rt, (((JS_CLASS_PROXY as i32)) as JSClassID))) != 0) as i32)) != 0 { 15 } else { 11 }; continue;
}
// C line 51309
17 => {
rt = (*(ctx)).rt;
vm_block = 16; continue;
}
_ => std::process::abort(),
} }
}
