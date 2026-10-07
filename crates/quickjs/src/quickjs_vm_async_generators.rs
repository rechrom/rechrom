// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:21308. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_async_generator_resolve_function_create(mut ctx: *mut JSContext, mut generator: JSValue, mut resolving_funcs: *mut JSValue, mut is_resume_next: i32) -> i32 {
let mut vm_local_storage = Vec::<u64>::new();
let mut i: i32 = core::mem::zeroed();
let mut func: JSValue = core::mem::zeroed();
let mut vm_block: usize = 10;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 21326
1 => {
return (0 as i32);
}
// C line 21316
2 => {
vm_block = if ((((i) < ((2 as i32))) as i32)) != 0 { 9 } else { 1 }; continue;
}
// C line ?
3 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 2; continue;
}
// C line 21324
4 => {
let _ = { let assigned = func; *(resolving_funcs).offset((i) as isize) = assigned; assigned };
vm_block = 3; continue;
}
// C line 21322
5 => {
return ((1 as i32)).wrapping_neg();
}
// C line 21321
6 => {
let _ = JS_FreeValue(ctx, *(resolving_funcs).offset(((0 as i32)) as isize));
vm_block = 5; continue;
}
// C line 21320
7 => {
vm_block = if ((((i) == ((1 as i32))) as i32)) != 0 { 6 } else { 5 }; continue;
}
// C line 21319
8 => {
vm_block = if (JS_IsException(func)) != 0 { 7 } else { 4 }; continue;
}
// C line 21317
9 => {
let _ = { let assigned = JS_NewCFunctionData(ctx, Some(js_async_generator_resolve_function), (1 as i32), (i).wrapping_add((is_resume_next).wrapping_mul((2 as i32))), (1 as i32), core::ptr::addr_of_mut!(generator)); func = assigned; assigned };
vm_block = 8; continue;
}
// C line 21316
10 => {
let _ = { let assigned = (0 as i32); i = assigned; assigned };
vm_block = 2; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:21329. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_async_generator_await(mut ctx: *mut JSContext, mut s: *mut JSAsyncGeneratorData, mut value: JSValue) -> i32 {
let mut vm_local_storage = Vec::<u64>::new();
let mut promise: JSValue = core::mem::zeroed();
let mut resolving_funcs: [JSValue; 2] = core::mem::zeroed();
let mut resolving_funcs1: [JSValue; 2] = core::mem::zeroed();
let mut i: i32 = core::mem::zeroed();
let mut res: i32 = core::mem::zeroed();
let mut vm_block: usize = 20;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line ? labels: fail
1 => {
return ((1 as i32)).wrapping_neg();
}
// C line 21359
2 => {
return (0 as i32);
}
// C line 21358
3 => {
vm_block = 1; continue;
}
// C line 21357
4 => {
vm_block = if (res) != 0 { 3 } else { 2 }; continue;
}
// C line 21355
5 => {
vm_block = if ((((i) < ((2 as i32))) as i32)) != 0 { 7 } else { 4 }; continue;
}
// C line ?
6 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 5; continue;
}
// C line 21356
7 => {
let _ = JS_FreeValue(ctx, *((resolving_funcs).as_mut_ptr()).offset((i) as isize));
vm_block = 6; continue;
}
// C line 21355
8 => {
let _ = { let assigned = (0 as i32); i = assigned; assigned };
vm_block = 5; continue;
}
// C line 21354
9 => {
let _ = JS_FreeValue(ctx, promise);
vm_block = 8; continue;
}
// C line 21351
10 => {
let _ = { let assigned = perform_promise_then(ctx, promise, (resolving_funcs).as_mut_ptr(), (resolving_funcs1).as_mut_ptr()); res = assigned; assigned };
vm_block = 9; continue;
}
// C line 21349
11 => {
vm_block = if ((((i) < ((2 as i32))) as i32)) != 0 { 13 } else { 10 }; continue;
}
// C line ?
12 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 11; continue;
}
// C line 21350
13 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; *((resolving_funcs1).as_mut_ptr()).offset((i) as isize) = assigned; assigned };
vm_block = 12; continue;
}
// C line 21349
14 => {
let _ = { let assigned = (0 as i32); i = assigned; assigned };
vm_block = 11; continue;
}
// C line 21344
15 => {
vm_block = 1; continue;
}
// C line 21343
16 => {
let _ = JS_FreeValue(ctx, promise);
vm_block = 15; continue;
}
// C line 21341
17 => {
vm_block = if (js_async_generator_resolve_function_create(ctx, JSValue { u: JSValueUnion { ptr: (((*(s)).generator) as *mut c_void) }, tag: (((JS_TAG_OBJECT as i32)) as i64) }, (resolving_funcs).as_mut_ptr(), (0 as i32))) != 0 { 16 } else { 14 }; continue;
}
// C line 21339
18 => {
vm_block = 1; continue;
}
// C line 21338
19 => {
vm_block = if (JS_IsException(promise)) != 0 { 18 } else { 17 }; continue;
}
// C line 21336
20 => {
let _ = { let assigned = js_promise_resolve(ctx, (*(ctx)).promise_ctor, (1 as i32), core::ptr::addr_of_mut!(value), (0 as i32)); promise = assigned; assigned };
vm_block = 19; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:21364. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_async_generator_resolve_or_reject(mut ctx: *mut JSContext, mut s: *mut JSAsyncGeneratorData, mut result: JSValue, mut is_reject: i32) -> () {
let mut vm_local_storage = Vec::<u64>::new();
let mut next: *mut JSAsyncGeneratorRequest = core::mem::zeroed();
let mut ret: JSValue = core::mem::zeroed();
let mut vm_block: usize = 9;
loop { match vm_block {
// C line ?
0 => {
return ();
}
// C line 21381
1 => {
let _ = js_free(ctx, ((next) as *mut c_void));
vm_block = 0; continue;
}
// C line 21380
2 => {
let _ = JS_FreeValue(ctx, *(((*(next)).resolving_funcs).as_mut_ptr()).offset(((1 as i32)) as isize));
vm_block = 1; continue;
}
// C line 21379
3 => {
let _ = JS_FreeValue(ctx, *(((*(next)).resolving_funcs).as_mut_ptr()).offset(((0 as i32)) as isize));
vm_block = 2; continue;
}
// C line 21378
4 => {
let _ = JS_FreeValue(ctx, (*(next)).promise);
vm_block = 3; continue;
}
// C line 21377
5 => {
let _ = JS_FreeValue(ctx, (*(next)).result);
vm_block = 4; continue;
}
// C line 21376
6 => {
let _ = JS_FreeValue(ctx, ret);
vm_block = 5; continue;
}
// C line 21374
7 => {
let _ = { let assigned = JS_Call(ctx, *(((*(next)).resolving_funcs).as_mut_ptr()).offset((is_reject) as isize), JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }, (1 as i32), core::ptr::addr_of_mut!(result)); ret = assigned; assigned };
vm_block = 6; continue;
}
// C line 21373
8 => {
let _ = list_del(core::ptr::addr_of_mut!((*(next)).link));
vm_block = 7; continue;
}
// C line 21372
9 => {
let _ = { let assigned = (((((((*(s)).queue).next) as *mut u8)).offset(-(((core::mem::offset_of!(JSAsyncGeneratorRequest, link) as usize)) as isize))) as *mut JSAsyncGeneratorRequest); next = assigned; assigned };
vm_block = 8; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:21384. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_async_generator_resolve(mut ctx: *mut JSContext, mut s: *mut JSAsyncGeneratorData, mut value: JSValue, mut done: i32) -> () {
let mut vm_local_storage = Vec::<u64>::new();
let mut result: JSValue = core::mem::zeroed();
let mut vm_block: usize = 3;
loop { match vm_block {
// C line ?
0 => {
return ();
}
// C line 21393
1 => {
let _ = JS_FreeValue(ctx, result);
vm_block = 0; continue;
}
// C line 21392
2 => {
let _ = js_async_generator_resolve_or_reject(ctx, s, result, (0 as i32));
vm_block = 1; continue;
}
// C line 21390
3 => {
let _ = { let assigned = js_create_iterator_result(ctx, JS_DupValue(ctx, value), done); result = assigned; assigned };
vm_block = 2; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:21396. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_async_generator_reject(mut ctx: *mut JSContext, mut s: *mut JSAsyncGeneratorData, mut exception: JSValue) -> () {
let mut vm_local_storage = Vec::<u64>::new();
let mut vm_block: usize = 1;
loop { match vm_block {
// C line ?
0 => {
return ();
}
// C line 21400
1 => {
let _ = js_async_generator_resolve_or_reject(ctx, s, exception, (1 as i32));
vm_block = 0; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:21403. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_async_generator_complete(mut ctx: *mut JSContext, mut s: *mut JSAsyncGeneratorData) -> () {
let mut vm_local_storage = Vec::<u64>::new();
let mut vm_block: usize = 4;
loop { match vm_block {
// C line ?
0 => {
return ();
}
// C line 21409
1 => {
let _ = { let assigned = core::ptr::null_mut::<JSAsyncFunctionState>(); (*(s)).func_state = assigned; assigned };
vm_block = 0; continue;
}
// C line 21408
2 => {
let _ = async_func_free((*(ctx)).rt, (*(s)).func_state);
vm_block = 1; continue;
}
// C line 21407
3 => {
let _ = { let assigned = (((JS_ASYNC_GENERATOR_STATE_COMPLETED as i32)) as JSAsyncGeneratorStateEnum); (*(s)).state = assigned; assigned };
vm_block = 2; continue;
}
// C line 21406
4 => {
vm_block = if (((((((*(s)).state) as u32)) != ((((JS_ASYNC_GENERATOR_STATE_COMPLETED as i32)) as u32))) as i32)) != 0 { 3 } else { 0 }; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:21413. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_async_generator_completed_return(mut ctx: *mut JSContext, mut s: *mut JSAsyncGeneratorData, mut value: JSValue) -> i32 {
let mut vm_local_storage = Vec::<u64>::new();
let mut promise: JSValue = core::mem::zeroed();
let mut resolving_funcs: [JSValue; 2] = core::mem::zeroed();
let mut resolving_funcs1: [JSValue; 2] = core::mem::zeroed();
let mut res: i32 = core::mem::zeroed();
let mut err: JSValue = core::mem::zeroed();
let mut vm_block: usize = 17;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 21448
1 => {
return res;
}
// C line 21447
2 => {
let _ = JS_FreeValue(ctx, promise);
vm_block = 1; continue;
}
// C line 21446
3 => {
let _ = JS_FreeValue(ctx, *((resolving_funcs1).as_mut_ptr()).offset(((1 as i32)) as isize));
vm_block = 2; continue;
}
// C line 21445
4 => {
let _ = JS_FreeValue(ctx, *((resolving_funcs1).as_mut_ptr()).offset(((0 as i32)) as isize));
vm_block = 3; continue;
}
// C line 21442
5 => {
let _ = { let assigned = perform_promise_then(ctx, promise, (resolving_funcs1).as_mut_ptr(), (resolving_funcs).as_mut_ptr()); res = assigned; assigned };
vm_block = 4; continue;
}
// C line 21441
6 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; *((resolving_funcs).as_mut_ptr()).offset(((1 as i32)) as isize) = assigned; assigned };
vm_block = 5; continue;
}
// C line 21440
7 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; *((resolving_funcs).as_mut_ptr()).offset(((0 as i32)) as isize) = assigned; assigned };
vm_block = 6; continue;
}
// C line 21438
8 => {
return ((1 as i32)).wrapping_neg();
}
// C line 21437
9 => {
let _ = JS_FreeValue(ctx, promise);
vm_block = 8; continue;
}
// C line 21433
10 => {
vm_block = if (js_async_generator_resolve_function_create(ctx, JSValue { u: JSValueUnion { ptr: (((*(s)).generator) as *mut c_void) }, tag: (((JS_TAG_OBJECT as i32)) as i64) }, (resolving_funcs1).as_mut_ptr(), (1 as i32))) != 0 { 9 } else { 7 }; continue;
}
// C line 21431
11 => {
return ((1 as i32)).wrapping_neg();
}
// C line 21430
12 => {
vm_block = if (JS_IsException(promise)) != 0 { 11 } else { 10 }; continue;
}
// C line 21429
13 => {
let _ = JS_FreeValue(ctx, err);
vm_block = 12; continue;
}
// C line 21427
14 => {
let _ = { let assigned = js_promise_resolve(ctx, (*(ctx)).promise_ctor, (1 as i32), core::ptr::addr_of_mut!(err), (1 as i32)); promise = assigned; assigned };
vm_block = 13; continue;
}
// C line 21426
15 => {
err = JS_GetException(ctx);
vm_block = 14; continue;
}
// C line 21425
16 => {
vm_block = if (JS_IsException(promise)) != 0 { 15 } else { 10 }; continue;
}
// C line 21421
17 => {
let _ = { let assigned = js_promise_resolve(ctx, (*(ctx)).promise_ctor, (1 as i32), core::ptr::addr_of_mut!(value), (0 as i32)); promise = assigned; assigned };
vm_block = 16; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:21451. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_async_generator_resume_next(mut ctx: *mut JSContext, mut s: *mut JSAsyncGeneratorData) -> () {
let mut vm_local_storage = Vec::<u64>::new();
let mut next: *mut JSAsyncGeneratorRequest = core::mem::zeroed();
let mut func_ret: JSValue = core::mem::zeroed();
let mut value: JSValue = core::mem::zeroed();
let mut func_ret_code: i32 = core::mem::zeroed();
let mut ret: i32 = core::mem::zeroed();
let mut vm_block: usize = 1;
loop { match vm_block {
// C line ? labels: done
0 => {
return ();
}
// C line 21457
1 => {
vm_block = 57; continue;
}
// C line ?
2 => {
let _ = std::process::abort();
vm_block = 1; continue;
}
// C line 21545
3 => {
vm_block = 1; continue;
}
// C line 21509
4 => {
let _ = JS_FreeValue(ctx, value);
vm_block = 3; continue;
}
// C line 21508
5 => {
let _ = js_async_generator_reject(ctx, s, value);
vm_block = 4; continue;
}
// C line 21507
6 => {
let _ = js_async_generator_complete(ctx, s);
vm_block = 5; continue;
}
// C line 21506
7 => {
let _ = { let assigned = JS_GetException(ctx); value = assigned; assigned };
vm_block = 6; continue;
}
// C line 21514
8 => {
let _ = JS_FreeValue(ctx, func_ret);
vm_block = 3; continue;
}
// C line 21513
9 => {
let _ = js_async_generator_resolve(ctx, s, func_ret, (1 as i32));
vm_block = 8; continue;
}
// C line 21512
10 => {
let _ = js_async_generator_complete(ctx, s);
vm_block = 9; continue;
}
// C line 21505
11 => {
vm_block = if (JS_IsException(func_ret)) != 0 { 7 } else { 10 }; continue;
}
// C line ?
12 => {
let _ = std::process::abort();
vm_block = 3; continue;
}
// C line 21540
13 => {
vm_block = 0; continue;
}
// C line 21538
14 => {
vm_block = 31; continue;
}
// C line 21537
15 => {
let _ = { let assigned = (1 as i32); (*((*(s)).func_state)).throw_flag = assigned; assigned };
vm_block = 14; continue;
}
// C line 21535
16 => {
vm_block = if ((((ret) < ((0 as i32))) as i32)) != 0 { 15 } else { 13 }; continue;
}
// C line 21534
17 => {
let _ = JS_FreeValue(ctx, value);
vm_block = 16; continue;
}
// C line 21533
18 => {
let _ = { let assigned = js_async_generator_await(ctx, s, value); ret = assigned; assigned };
vm_block = 17; continue;
}
// C line 21531
19 => {
vm_block = 3; continue;
}
// C line 21530
20 => {
let _ = JS_FreeValue(ctx, value);
vm_block = 19; continue;
}
// C line 21529
21 => {
let _ = js_async_generator_resolve(ctx, s, value, (0 as i32));
vm_block = 20; continue;
}
// C line 21526
22 => {
let _ = { let assigned = (((JS_ASYNC_GENERATOR_STATE_SUSPENDED_YIELD_STAR as i32)) as JSAsyncGeneratorStateEnum); (*(s)).state = assigned; assigned };
vm_block = 21; continue;
}
// C line 21528
23 => {
let _ = { let assigned = (((JS_ASYNC_GENERATOR_STATE_SUSPENDED_YIELD as i32)) as JSAsyncGeneratorStateEnum); (*(s)).state = assigned; assigned };
vm_block = 21; continue;
}
// C line 21525
24 => {
vm_block = if ((((func_ret_code) == ((2 as i32))) as i32)) != 0 { 22 } else { 23 }; continue;
}
// C line 21522
25 => {
vm_block = match func_ret_code { x if x == (0 as i32) => 18, x if x == (2 as i32) => 24, x if x == (1 as i32) => 24, _ => 12, }; continue;
}
// C line 21521
26 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; *(((*((*(s)).func_state)).frame).cur_sp).offset((((1 as i32)).wrapping_neg()) as isize) = assigned; assigned };
vm_block = 25; continue;
}
// C line 21520
27 => {
let _ = { let assigned = *(((*((*(s)).func_state)).frame).cur_sp).offset((((1 as i32)).wrapping_neg()) as isize); value = assigned; assigned };
vm_block = 26; continue;
}
// C line 21519
28 => {
let _ = { let assigned = ((((func_ret).u).uint64) as i32); func_ret_code = assigned; assigned };
vm_block = 27; continue;
}
// C line 21518
29 => {
let _ = if ((((!((((((((func_ret).tag) as i32)) == ((JS_TAG_INT as i32))) as i32)) != 0) as i32)) as i64)) != 0 { std::process::abort() } else { { let _ = (0 as i32); } };
vm_block = 28; continue;
}
// C line 21504
30 => {
vm_block = if ((*((*(s)).func_state)).is_completed) != 0 { 11 } else { 29 }; continue;
}
// C line ? labels: resume_exec
31 => {
let _ = { let assigned = async_func_resume(ctx, (*(s)).func_state); func_ret = assigned; assigned };
vm_block = 30; continue;
}
// C line 21501
32 => {
let _ = { let assigned = (((JS_ASYNC_GENERATOR_STATE_EXECUTING as i32)) as JSAsyncGeneratorStateEnum); (*(s)).state = assigned; assigned };
vm_block = 31; continue;
}
// C line 21490
33 => {
let _ = { let assigned = (1 as i32); (*((*(s)).func_state)).throw_flag = assigned; assigned };
vm_block = 32; continue;
}
// C line 21489
34 => {
let _ = JS_Throw(ctx, value);
vm_block = 33; continue;
}
// C line ? labels: exec_no_arg
35 => {
let _ = { let assigned = (0 as i32); (*((*(s)).func_state)).throw_flag = assigned; assigned };
vm_block = 32; continue;
}
// C line 21497
36 => {
let _ = { let old = ((*((*(s)).func_state)).frame).cur_sp; ((*((*(s)).func_state)).frame).cur_sp = (((*((*(s)).func_state)).frame).cur_sp).offset(1); old };
vm_block = 35; continue;
}
// C line 21495
37 => {
let _ = { let assigned = JS_NewInt32(ctx, (*(next)).completion_type); *(((*((*(s)).func_state)).frame).cur_sp).offset(((0 as i32)) as isize) = assigned; assigned };
vm_block = 36; continue;
}
// C line 21494
38 => {
let _ = { let assigned = value; *(((*((*(s)).func_state)).frame).cur_sp).offset((((1 as i32)).wrapping_neg()) as isize) = assigned; assigned };
vm_block = 37; continue;
}
// C line 21487
39 => {
vm_block = if (((((((((*(next)).completion_type) == ((2 as i32))) as i32)) != 0) && ((((((((*(s)).state) as u32)) == ((((JS_ASYNC_GENERATOR_STATE_SUSPENDED_YIELD as i32)) as u32))) as i32)) != 0)) as i32)) != 0 { 34 } else { 38 }; continue;
}
// C line 21486
40 => {
let _ = { let assigned = JS_DupValue(ctx, (*(next)).result); value = assigned; assigned };
vm_block = 39; continue;
}
// C line 21483
41 => {
vm_block = 0; continue;
}
// C line 21476
42 => {
let _ = js_async_generator_resolve(ctx, s, JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }, (1 as i32));
vm_block = 41; continue;
}
// C line 21479
43 => {
let _ = js_async_generator_completed_return(ctx, s, (*(next)).result);
vm_block = 41; continue;
}
// C line 21478
44 => {
let _ = { let assigned = (((JS_ASYNC_GENERATOR_STATE_AWAITING_RETURN as i32)) as JSAsyncGeneratorStateEnum); (*(s)).state = assigned; assigned };
vm_block = 43; continue;
}
// C line 21481
45 => {
let _ = js_async_generator_reject(ctx, s, (*(next)).result);
vm_block = 41; continue;
}
// C line 21477
46 => {
vm_block = if (((((*(next)).completion_type) == ((1 as i32))) as i32)) != 0 { 44 } else { 45 }; continue;
}
// C line 21475
47 => {
vm_block = if (((((*(next)).completion_type) == ((0 as i32))) as i32)) != 0 { 42 } else { 46 }; continue;
}
// C line 21473
48 => {
vm_block = 1; continue;
}
// C line 21469
49 => {
vm_block = 35; continue;
}
// C line 21471
50 => {
let _ = js_async_generator_complete(ctx, s);
vm_block = 48; continue;
}
// C line 21468
51 => {
vm_block = if (((((*(next)).completion_type) == ((0 as i32))) as i32)) != 0 { 49 } else { 50 }; continue;
}
// C line 21466
52 => {
vm_block = 0; continue;
}
// C line 21464
53 => {
vm_block = 31; continue;
}
// C line 21461
54 => {
vm_block = match (((*(s)).state) as u32) { x if x == (((JS_ASYNC_GENERATOR_STATE_SUSPENDED_YIELD_STAR as i32)) as u32) => 40, x if x == (((JS_ASYNC_GENERATOR_STATE_SUSPENDED_YIELD as i32)) as u32) => 40, x if x == (((JS_ASYNC_GENERATOR_STATE_COMPLETED as i32)) as u32) => 47, x if x == (((JS_ASYNC_GENERATOR_STATE_SUSPENDED_START as i32)) as u32) => 51, x if x == (((JS_ASYNC_GENERATOR_STATE_AWAITING_RETURN as i32)) as u32) => 52, x if x == (((JS_ASYNC_GENERATOR_STATE_EXECUTING as i32)) as u32) => 53, _ => 2, }; continue;
}
// C line 21460
55 => {
let _ = { let assigned = (((((((*(s)).queue).next) as *mut u8)).offset(-(((core::mem::offset_of!(JSAsyncGeneratorRequest, link) as usize)) as isize))) as *mut JSAsyncGeneratorRequest); next = assigned; assigned };
vm_block = 54; continue;
}
// C line 21459
56 => {
vm_block = 0; continue;
}
// C line 21458
57 => {
vm_block = if (list_empty(core::ptr::addr_of_mut!((*(s)).queue))) != 0 { 56 } else { 55 }; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:21553. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_async_generator_resolve_function(mut ctx: *mut JSContext, mut this_obj: JSValue, mut argc: i32, mut argv: *mut JSValue, mut magic: i32, mut func_data: *mut JSValue) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut is_reject: i32 = core::mem::zeroed();
let mut s: *mut JSAsyncGeneratorData = core::mem::zeroed();
let mut arg: JSValue = core::mem::zeroed();
let mut vm_block: usize = 16;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 21585
1 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) };
}
// C line 21570
2 => {
let _ = js_async_generator_reject(ctx, s, arg);
vm_block = 1; continue;
}
// C line 21572
3 => {
let _ = js_async_generator_resolve(ctx, s, arg, (1 as i32));
vm_block = 1; continue;
}
// C line 21569
4 => {
vm_block = if (is_reject) != 0 { 2 } else { 3 }; continue;
}
// C line 21568
5 => {
let _ = { let assigned = (((JS_ASYNC_GENERATOR_STATE_COMPLETED as i32)) as JSAsyncGeneratorStateEnum); (*(s)).state = assigned; assigned };
vm_block = 4; continue;
}
// C line 21566
6 => {
let _ = if ((((!((((((((((((*(s)).state) as u32)) == ((((JS_ASYNC_GENERATOR_STATE_AWAITING_RETURN as i32)) as u32))) as i32)) != 0) || ((((((((*(s)).state) as u32)) == ((((JS_ASYNC_GENERATOR_STATE_COMPLETED as i32)) as u32))) as i32)) != 0)) as i32)) != 0) as i32)) as i64)) != 0 { std::process::abort() } else { { let _ = (0 as i32); } };
vm_block = 5; continue;
}
// C line 21583
7 => {
let _ = js_async_generator_resume_next(ctx, s);
vm_block = 1; continue;
}
// C line 21578
8 => {
let _ = JS_Throw(ctx, JS_DupValue(ctx, arg));
vm_block = 7; continue;
}
// C line 21581
9 => {
let _ = { let assigned = JS_DupValue(ctx, arg); *(((*((*(s)).func_state)).frame).cur_sp).offset((((1 as i32)).wrapping_neg()) as isize) = assigned; assigned };
vm_block = 7; continue;
}
// C line 21577
10 => {
vm_block = if (is_reject) != 0 { 8 } else { 9 }; continue;
}
// C line 21576
11 => {
let _ = { let assigned = is_reject; (*((*(s)).func_state)).throw_flag = assigned; assigned };
vm_block = 10; continue;
}
// C line 21574
12 => {
vm_block = if (((((((*(s)).state) as u32)) == ((((JS_ASYNC_GENERATOR_STATE_EXECUTING as i32)) as u32))) as i32)) != 0 { 11 } else { 1 }; continue;
}
// C line 21564
13 => {
vm_block = if ((((magic) >= ((2 as i32))) as i32)) != 0 { 6 } else { 12 }; continue;
}
// C line 21560
14 => {
arg = *(argv).offset(((0 as i32)) as isize);
vm_block = 13; continue;
}
// C line 21559
15 => {
s = ((JS_GetOpaque(*(func_data).offset(((0 as i32)) as isize), (((JS_CLASS_ASYNC_GENERATOR as i32)) as JSClassID))) as *mut JSAsyncGeneratorData);
vm_block = 14; continue;
}
// C line 21558
16 => {
is_reject = ((magic) & ((1 as i32)));
vm_block = 15; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:21589. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_async_generator_next(mut ctx: *mut JSContext, mut this_val: JSValue, mut argc: i32, mut argv: *mut JSValue, mut magic: i32) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut s: *mut JSAsyncGeneratorData = core::mem::zeroed();
let mut promise: JSValue = core::mem::zeroed();
let mut resolving_funcs: [JSValue; 2] = core::mem::zeroed();
let mut req: *mut JSAsyncGeneratorRequest = core::mem::zeroed();
let mut err: JSValue = core::mem::zeroed();
let mut res2: JSValue = core::mem::zeroed();
let mut vm_block: usize = 29;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 21629
1 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 21628
2 => {
let _ = JS_FreeValue(ctx, promise);
vm_block = 1; continue;
}
// C line 21627
3 => {
let _ = JS_FreeValue(ctx, *((resolving_funcs).as_mut_ptr()).offset(((1 as i32)) as isize));
vm_block = 2; continue;
}
// C line ? labels: fail
4 => {
let _ = JS_FreeValue(ctx, *((resolving_funcs).as_mut_ptr()).offset(((0 as i32)) as isize));
vm_block = 3; continue;
}
// C line 21624
5 => {
return promise;
}
// C line 21622
6 => {
let _ = js_async_generator_resume_next(ctx, s);
vm_block = 5; continue;
}
// C line 21621
7 => {
vm_block = if (((((((*(s)).state) as u32)) != ((((JS_ASYNC_GENERATOR_STATE_EXECUTING as i32)) as u32))) as i32)) != 0 { 6 } else { 5 }; continue;
}
// C line 21620
8 => {
let _ = list_add_tail(core::ptr::addr_of_mut!((*(req)).link), core::ptr::addr_of_mut!((*(s)).queue));
vm_block = 7; continue;
}
// C line 21619
9 => {
let _ = { let assigned = *((resolving_funcs).as_mut_ptr()).offset(((1 as i32)) as isize); *(((*(req)).resolving_funcs).as_mut_ptr()).offset(((1 as i32)) as isize) = assigned; assigned };
vm_block = 8; continue;
}
// C line 21618
10 => {
let _ = { let assigned = *((resolving_funcs).as_mut_ptr()).offset(((0 as i32)) as isize); *(((*(req)).resolving_funcs).as_mut_ptr()).offset(((0 as i32)) as isize) = assigned; assigned };
vm_block = 9; continue;
}
// C line 21617
11 => {
let _ = { let assigned = JS_DupValue(ctx, promise); (*(req)).promise = assigned; assigned };
vm_block = 10; continue;
}
// C line 21616
12 => {
let _ = { let assigned = JS_DupValue(ctx, *(argv).offset(((0 as i32)) as isize)); (*(req)).result = assigned; assigned };
vm_block = 11; continue;
}
// C line 21615
13 => {
let _ = { let assigned = magic; (*(req)).completion_type = assigned; assigned };
vm_block = 12; continue;
}
// C line 21614
14 => {
vm_block = 4; continue;
}
// C line 21613
15 => {
vm_block = if ((!(!(req).is_null()) as i32)) != 0 { 14 } else { 13 }; continue;
}
// C line 21612
16 => {
let _ = { let assigned = ((js_mallocz(ctx, (size_of::<JSAsyncGeneratorRequest>() as usize))) as *mut JSAsyncGeneratorRequest); req = assigned; assigned };
vm_block = 15; continue;
}
// C line 21610
17 => {
return promise;
}
// C line 21609
18 => {
let _ = JS_FreeValue(ctx, *((resolving_funcs).as_mut_ptr()).offset(((1 as i32)) as isize));
vm_block = 17; continue;
}
// C line 21608
19 => {
let _ = JS_FreeValue(ctx, *((resolving_funcs).as_mut_ptr()).offset(((0 as i32)) as isize));
vm_block = 18; continue;
}
// C line 21607
20 => {
let _ = JS_FreeValue(ctx, res2);
vm_block = 19; continue;
}
// C line 21606
21 => {
let _ = JS_FreeValue(ctx, err);
vm_block = 20; continue;
}
// C line 21604
22 => {
let _ = { let assigned = JS_Call(ctx, *((resolving_funcs).as_mut_ptr()).offset(((1 as i32)) as isize), JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }, (1 as i32), core::ptr::addr_of_mut!(err)); res2 = assigned; assigned };
vm_block = 21; continue;
}
// C line 21603
23 => {
let _ = { let assigned = JS_GetException(ctx); err = assigned; assigned };
vm_block = 22; continue;
}
// C line 21602
24 => {
let _ = JS_ThrowTypeError(ctx, c"not an AsyncGenerator object".as_ptr());
vm_block = 23; continue;
}
// C line 21600
25 => {
vm_block = if ((!(!(s).is_null()) as i32)) != 0 { 24 } else { 16 }; continue;
}
// C line 21599
26 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 21598
27 => {
vm_block = if (JS_IsException(promise)) != 0 { 26 } else { 25 }; continue;
}
// C line 21597
28 => {
let _ = { let assigned = JS_NewPromiseCapability(ctx, (resolving_funcs).as_mut_ptr()); promise = assigned; assigned };
vm_block = 27; continue;
}
// C line 21593
29 => {
s = ((JS_GetOpaque(this_val, (((JS_CLASS_ASYNC_GENERATOR as i32)) as JSClassID))) as *mut JSAsyncGeneratorData);
vm_block = 28; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:21632. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_async_generator_function_call(mut ctx: *mut JSContext, mut func_obj: JSValue, mut this_obj: JSValue, mut argc: i32, mut argv: *mut JSValue, mut flags: i32) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut obj: JSValue = core::mem::zeroed();
let mut func_ret: JSValue = core::mem::zeroed();
let mut s: *mut JSAsyncGeneratorData = core::mem::zeroed();
let mut vm_block: usize = 20;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 21663
1 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line ? labels: fail
2 => {
let _ = js_async_generator_free((*(ctx)).rt, s);
vm_block = 1; continue;
}
// C line 21660
3 => {
return obj;
}
// C line 21659
4 => {
let _ = JS_SetOpaque(obj, ((s) as *mut c_void));
vm_block = 3; continue;
}
// C line 21658
5 => {
let _ = { let assigned = ((((obj).u).ptr) as *mut JSObject); (*(s)).generator = assigned; assigned };
vm_block = 4; continue;
}
// C line 21657
6 => {
vm_block = 2; continue;
}
// C line 21656
7 => {
vm_block = if (JS_IsException(obj)) != 0 { 6 } else { 5 }; continue;
}
// C line 21655
8 => {
let _ = { let assigned = js_create_from_ctor(ctx, func_obj, (JS_CLASS_ASYNC_GENERATOR as i32)); obj = assigned; assigned };
vm_block = 7; continue;
}
// C line 21653
9 => {
let _ = JS_FreeValue(ctx, func_ret);
vm_block = 8; continue;
}
// C line 21652
10 => {
vm_block = 2; continue;
}
// C line 21651
11 => {
vm_block = if (JS_IsException(func_ret)) != 0 { 10 } else { 9 }; continue;
}
// C line 21650
12 => {
let _ = { let assigned = async_func_resume(ctx, (*(s)).func_state); func_ret = assigned; assigned };
vm_block = 11; continue;
}
// C line 21647
13 => {
vm_block = 2; continue;
}
// C line 21646
14 => {
vm_block = if ((!(!((*(s)).func_state).is_null()) as i32)) != 0 { 13 } else { 12 }; continue;
}
// C line 21645
15 => {
let _ = { let assigned = async_func_init(ctx, func_obj, this_obj, argc, argv); (*(s)).func_state = assigned; assigned };
vm_block = 14; continue;
}
// C line 21644
16 => {
let _ = init_list_head(core::ptr::addr_of_mut!((*(s)).queue));
vm_block = 15; continue;
}
// C line 21643
17 => {
let _ = { let assigned = (((JS_ASYNC_GENERATOR_STATE_SUSPENDED_START as i32)) as JSAsyncGeneratorStateEnum); (*(s)).state = assigned; assigned };
vm_block = 16; continue;
}
// C line 21642
18 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 21641
19 => {
vm_block = if ((!(!(s).is_null()) as i32)) != 0 { 18 } else { 17 }; continue;
}
// C line 21640
20 => {
let _ = { let assigned = ((js_mallocz(ctx, (size_of::<JSAsyncGeneratorData>() as usize))) as *mut JSAsyncGeneratorData); s = assigned; assigned };
vm_block = 19; continue;
}
_ => std::process::abort(),
} }
}
