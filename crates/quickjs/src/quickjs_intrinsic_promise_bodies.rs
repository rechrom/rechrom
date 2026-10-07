// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:53159. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn promise_reaction_job(mut ctx: *mut JSContext, mut argc: i32, mut argv: *mut JSValue) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut handler: JSValue = core::mem::zeroed();
let mut arg: JSValue = core::mem::zeroed();
let mut func: JSValue = core::mem::zeroed();
let mut res: JSValue = core::mem::zeroed();
let mut res2: JSValue = core::mem::zeroed();
let mut is_reject: i32 = core::mem::zeroed();
let mut vm_block: usize = 18;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 53198
1 => {
return res2;
}
// C line 53196
2 => {
let _ = JS_FreeValue(ctx, res);
vm_block = 1; continue;
}
// C line 53191
3 => {
let _ = { let assigned = JS_Call(ctx, func, JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }, (1 as i32), core::ptr::addr_of_mut!(res)); res2 = assigned; assigned };
vm_block = 2; continue;
}
// C line 53194
4 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; res2 = assigned; assigned };
vm_block = 2; continue;
}
// C line 53190
5 => {
vm_block = if ((!((JS_IsUndefined(func)) != 0) as i32)) != 0 { 3 } else { 4 }; continue;
}
// C line 53186
6 => {
let _ = { let assigned = *(argv).offset((is_reject) as isize); func = assigned; assigned };
vm_block = 5; continue;
}
// C line 53185
7 => {
let _ = { let assigned = JS_GetException(ctx); res = assigned; assigned };
vm_block = 6; continue;
}
// C line 53184
8 => {
vm_block = if (is_reject) != 0 { 7 } else { 6 }; continue;
}
// C line 53183
9 => {
let _ = { let assigned = JS_IsException(res); is_reject = assigned; assigned };
vm_block = 8; continue;
}
// C line 53176
10 => {
let _ = { let assigned = JS_Throw(ctx, JS_DupValue(ctx, arg)); res = assigned; assigned };
vm_block = 9; continue;
}
// C line 53178
11 => {
let _ = { let assigned = JS_DupValue(ctx, arg); res = assigned; assigned };
vm_block = 9; continue;
}
// C line 53175
12 => {
vm_block = if (is_reject) != 0 { 10 } else { 11 }; continue;
}
// C line 53181
13 => {
let _ = { let assigned = JS_Call(ctx, handler, JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }, (1 as i32), core::ptr::addr_of_mut!(arg)); res = assigned; assigned };
vm_block = 9; continue;
}
// C line 53174
14 => {
vm_block = if (JS_IsUndefined(handler)) != 0 { 12 } else { 13 }; continue;
}
// C line 53169
15 => {
let _ = { let assigned = *(argv).offset(((4 as i32)) as isize); arg = assigned; assigned };
vm_block = 14; continue;
}
// C line 53168
16 => {
let _ = { let assigned = JS_ToBool(ctx, *(argv).offset(((3 as i32)) as isize)); is_reject = assigned; assigned };
vm_block = 15; continue;
}
// C line 53167
17 => {
let _ = { let assigned = *(argv).offset(((2 as i32)) as isize); handler = assigned; assigned };
vm_block = 16; continue;
}
// C line 53166
18 => {
let _ = if ((((!(((((argc) == ((5 as i32))) as i32)) != 0) as i32)) as i64)) != 0 { std::process::abort() } else { { let _ = (0 as i32); } };
vm_block = 17; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:53209. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn fulfill_or_reject_promise(mut ctx: *mut JSContext, mut promise: JSValue, mut value: JSValue, mut is_reject: i32) -> () {
let mut vm_local_storage = Vec::<u64>::new();
let mut s: *mut JSPromiseData = core::mem::zeroed();
let mut el: *mut list_head = core::mem::zeroed();
let mut el1: *mut list_head = core::mem::zeroed();
let mut rd: *mut JSPromiseReactionData = core::mem::zeroed();
let mut args: [JSValue; 5] = core::mem::zeroed();
let mut rt: *mut JSRuntime = core::mem::zeroed();
let mut vm_block: usize = 27;
loop { match vm_block {
// C line ?
0 => {
return ();
}
// C line 53244
1 => {
vm_block = if ((((el) != (core::ptr::addr_of_mut!(*(((*(s)).promise_reactions).as_mut_ptr()).offset((((1 as i32)).wrapping_sub(is_reject)) as isize)))) as i32)) != 0 { 5 } else { 0 }; continue;
}
// C line ?
2 => {
let _ = { let _ = { let assigned = el1; el = assigned; assigned }; { let assigned = (*(el)).next; el1 = assigned; assigned } };
vm_block = 1; continue;
}
// C line 53247
3 => {
let _ = promise_reaction_data_free((*(ctx)).rt, rd);
vm_block = 2; continue;
}
// C line 53246
4 => {
let _ = list_del(core::ptr::addr_of_mut!((*(rd)).link));
vm_block = 3; continue;
}
// C line 53245
5 => {
let _ = { let assigned = (((((el) as *mut u8)).offset(-(((core::mem::offset_of!(JSPromiseReactionData, link) as usize)) as isize))) as *mut JSPromiseReactionData); rd = assigned; assigned };
vm_block = 4; continue;
}
// C line ?
6 => {
let _ = { let _ = { let assigned = (*(core::ptr::addr_of_mut!(*(((*(s)).promise_reactions).as_mut_ptr()).offset((((1 as i32)).wrapping_sub(is_reject)) as isize)))).next; el = assigned; assigned }; { let assigned = (*(el)).next; el1 = assigned; assigned } };
vm_block = 1; continue;
}
// C line 53232
7 => {
vm_block = if ((((el) != (core::ptr::addr_of_mut!(*(((*(s)).promise_reactions).as_mut_ptr()).offset((is_reject) as isize)))) as i32)) != 0 { 17 } else { 6 }; continue;
}
// C line ?
8 => {
let _ = { let _ = { let assigned = el1; el = assigned; assigned }; { let assigned = (*(el)).next; el1 = assigned; assigned } };
vm_block = 7; continue;
}
// C line 53241
9 => {
let _ = promise_reaction_data_free((*(ctx)).rt, rd);
vm_block = 8; continue;
}
// C line 53240
10 => {
let _ = list_del(core::ptr::addr_of_mut!((*(rd)).link));
vm_block = 9; continue;
}
// C line 53239
11 => {
let _ = JS_EnqueueJob(ctx, Some(promise_reaction_job), (5 as i32), (args).as_mut_ptr());
vm_block = 10; continue;
}
// C line 53238
12 => {
let _ = { let assigned = value; *((args).as_mut_ptr()).offset(((4 as i32)) as isize) = assigned; assigned };
vm_block = 11; continue;
}
// C line 53237
13 => {
let _ = { let assigned = JS_NewBool(ctx, is_reject); *((args).as_mut_ptr()).offset(((3 as i32)) as isize) = assigned; assigned };
vm_block = 12; continue;
}
// C line 53236
14 => {
let _ = { let assigned = (*(rd)).handler; *((args).as_mut_ptr()).offset(((2 as i32)) as isize) = assigned; assigned };
vm_block = 13; continue;
}
// C line 53235
15 => {
let _ = { let assigned = *(((*(rd)).resolving_funcs).as_mut_ptr()).offset(((1 as i32)) as isize); *((args).as_mut_ptr()).offset(((1 as i32)) as isize) = assigned; assigned };
vm_block = 14; continue;
}
// C line 53234
16 => {
let _ = { let assigned = *(((*(rd)).resolving_funcs).as_mut_ptr()).offset(((0 as i32)) as isize); *((args).as_mut_ptr()).offset(((0 as i32)) as isize) = assigned; assigned };
vm_block = 15; continue;
}
// C line 53233
17 => {
let _ = { let assigned = (((((el) as *mut u8)).offset(-(((core::mem::offset_of!(JSPromiseReactionData, link) as usize)) as isize))) as *mut JSPromiseReactionData); rd = assigned; assigned };
vm_block = 16; continue;
}
// C line ?
18 => {
let _ = { let _ = { let assigned = (*(core::ptr::addr_of_mut!(*(((*(s)).promise_reactions).as_mut_ptr()).offset((is_reject) as isize)))).next; el = assigned; assigned }; { let assigned = (*(el)).next; el1 = assigned; assigned } };
vm_block = 7; continue;
}
// C line 53227
19 => {
let _ = ((*(rt)).host_promise_rejection_tracker).expect("installed rejection tracker")(ctx, promise, value, (0 as i32), (*(rt)).host_promise_rejection_tracker_opaque);
vm_block = 18; continue;
}
// C line 53226
20 => {
vm_block = if ((*(rt)).host_promise_rejection_tracker).is_some() { 19 } else { 18 }; continue;
}
// C line 53225
21 => {
rt = (*(ctx)).rt;
vm_block = 20; continue;
}
// C line 53224
22 => {
vm_block = if (((((((((((*(s)).promise_state) as u32)) == ((((JS_PROMISE_REJECTED as i32)) as u32))) as i32)) != 0) && (((!(((*(s)).is_handled) != 0) as i32)) != 0)) as i32)) != 0 { 21 } else { 18 }; continue;
}
// C line 53220
23 => {
let _ = { let assigned = ((((JS_PROMISE_FULFILLED as i32)).wrapping_add(is_reject)) as JSPromiseStateEnum); (*(s)).promise_state = assigned; assigned };
vm_block = 22; continue;
}
// C line 53219
24 => {
let _ = set_value(ctx, core::ptr::addr_of_mut!((*(s)).promise_result), JS_DupValue(ctx, value));
vm_block = 23; continue;
}
// C line 53218
25 => {
return;
}
// C line 53217
26 => {
vm_block = if ((((((!(!(s).is_null()) as i32)) != 0) || ((((((((*(s)).promise_state) as u32)) != ((((JS_PROMISE_PENDING as i32)) as u32))) as i32)) != 0)) as i32)) != 0 { 25 } else { 24 }; continue;
}
// C line 53212
27 => {
s = ((JS_GetOpaque(promise, (((JS_CLASS_PROMISE as i32)) as JSClassID))) as *mut JSPromiseData);
vm_block = 26; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:53251. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn reject_promise(mut ctx: *mut JSContext, mut promise: JSValue, mut value: JSValue) -> () {
let mut vm_local_storage = Vec::<u64>::new();
let mut vm_block: usize = 1;
loop { match vm_block {
// C line ?
0 => {
return ();
}
// C line 53254
1 => {
let _ = fulfill_or_reject_promise(ctx, promise, value, (1 as i32));
vm_block = 0; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:53257. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_promise_resolve_thenable_job(mut ctx: *mut JSContext, mut argc: i32, mut argv: *mut JSValue) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut promise: JSValue = core::mem::zeroed();
let mut thenable: JSValue = core::mem::zeroed();
let mut then: JSValue = core::mem::zeroed();
let mut args: [JSValue; 2] = core::mem::zeroed();
let mut res: JSValue = core::mem::zeroed();
let mut error: JSValue = core::mem::zeroed();
let mut vm_block: usize = 14;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 53280
1 => {
return res;
}
// C line 53279
2 => {
let _ = JS_FreeValue(ctx, *((args).as_mut_ptr()).offset(((1 as i32)) as isize));
vm_block = 1; continue;
}
// C line 53278
3 => {
let _ = JS_FreeValue(ctx, *((args).as_mut_ptr()).offset(((0 as i32)) as isize));
vm_block = 2; continue;
}
// C line 53276
4 => {
let _ = JS_FreeValue(ctx, error);
vm_block = 3; continue;
}
// C line 53275
5 => {
let _ = { let assigned = JS_Call(ctx, *((args).as_mut_ptr()).offset(((1 as i32)) as isize), JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }, (1 as i32), core::ptr::addr_of_mut!(error)); res = assigned; assigned };
vm_block = 4; continue;
}
// C line 53274
6 => {
error = JS_GetException(ctx);
vm_block = 5; continue;
}
// C line 53273
7 => {
vm_block = if (JS_IsException(res)) != 0 { 6 } else { 3 }; continue;
}
// C line 53272
8 => {
let _ = { let assigned = JS_Call(ctx, then, thenable, (2 as i32), (args).as_mut_ptr()); res = assigned; assigned };
vm_block = 7; continue;
}
// C line 53271
9 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 53270
10 => {
vm_block = if ((((js_create_resolving_functions(ctx, (args).as_mut_ptr(), promise)) < ((0 as i32))) as i32)) != 0 { 9 } else { 8 }; continue;
}
// C line 53269
11 => {
let _ = { let assigned = *(argv).offset(((2 as i32)) as isize); then = assigned; assigned };
vm_block = 10; continue;
}
// C line 53268
12 => {
let _ = { let assigned = *(argv).offset(((1 as i32)) as isize); thenable = assigned; assigned };
vm_block = 11; continue;
}
// C line 53267
13 => {
let _ = { let assigned = *(argv).offset(((0 as i32)) as isize); promise = assigned; assigned };
vm_block = 12; continue;
}
// C line 53266
14 => {
let _ = if ((((!(((((argc) == ((3 as i32))) as i32)) != 0) as i32)) as i64)) != 0 { std::process::abort() } else { { let _ = (0 as i32); } };
vm_block = 13; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:53291. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_create_resolving_functions(mut ctx: *mut JSContext, mut resolving_funcs: *mut JSValue, mut promise: JSValue) -> i32 {
let mut vm_local_storage = Vec::<u64>::new();
let mut obj: JSValue = core::mem::zeroed();
let mut s: *mut JSPromiseFunctionData = core::mem::zeroed();
let mut sr: *mut JSPromiseFunctionDataResolved = core::mem::zeroed();
let mut i: i32 = core::mem::zeroed();
let mut ret: i32 = core::mem::zeroed();
let mut vm_block: usize = 27;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 53330
1 => {
return ret;
}
// C line 53329
2 => {
let _ = js_promise_resolve_function_free_resolved((*(ctx)).rt, sr);
vm_block = 1; continue;
}
// C line 53307
3 => {
vm_block = if ((((i) < ((2 as i32))) as i32)) != 0 { 20 } else { 2 }; continue;
}
// C line ?
4 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 3; continue;
}
// C line 53327
5 => {
let _ = { let assigned = obj; *(resolving_funcs).offset((i) as isize) = assigned; assigned };
vm_block = 4; continue;
}
// C line 53326
6 => {
let _ = js_function_set_properties(ctx, obj, (((crate::quickjs_atom::JS_ATOM_empty_string as i32)) as JSAtom), (1 as i32));
vm_block = 5; continue;
}
// C line 53325
7 => {
let _ = JS_SetOpaque(obj, ((s) as *mut c_void));
vm_block = 6; continue;
}
// C line 53324
8 => {
let _ = { let assigned = JS_DupValue(ctx, promise); (*(s)).promise = assigned; assigned };
vm_block = 7; continue;
}
// C line 53323
9 => {
let _ = { let assigned = sr; (*(s)).presolved = assigned; assigned };
vm_block = 8; continue;
}
// C line 53322
10 => {
let _ = { let old = (*(sr)).ref_count; (*(sr)).ref_count = ((*(sr)).ref_count).wrapping_add(1); old };
vm_block = 9; continue;
}
// C line 53320
11 => {
vm_block = 2; continue;
}
// C line 53319
12 => {
let _ = { let assigned = ((1 as i32)).wrapping_neg(); ret = assigned; assigned };
vm_block = 11; continue;
}
// C line 53318
13 => {
let _ = JS_FreeValue(ctx, *(resolving_funcs).offset(((0 as i32)) as isize));
vm_block = 12; continue;
}
// C line 53317 labels: fail
14 => {
vm_block = if ((((i) != ((0 as i32))) as i32)) != 0 { 13 } else { 12 }; continue;
}
// C line 53314
15 => {
let _ = JS_FreeValue(ctx, obj);
vm_block = 14; continue;
}
// C line 53313
16 => {
vm_block = if ((!(!(s).is_null()) as i32)) != 0 { 15 } else { 10 }; continue;
}
// C line 53312
17 => {
let _ = { let assigned = ((js_malloc(ctx, (size_of::<JSPromiseFunctionData>() as usize))) as *mut JSPromiseFunctionData); s = assigned; assigned };
vm_block = 16; continue;
}
// C line 53311
18 => {
vm_block = 14; continue;
}
// C line 53310
19 => {
vm_block = if (JS_IsException(obj)) != 0 { 18 } else { 17 }; continue;
}
// C line 53308
20 => {
let _ = { let assigned = JS_NewObjectProtoClass(ctx, (*(ctx)).function_proto, ((((JS_CLASS_PROMISE_RESOLVE_FUNCTION as i32)).wrapping_add(i)) as JSClassID)); obj = assigned; assigned };
vm_block = 19; continue;
}
// C line 53307
21 => {
let _ = { let assigned = (0 as i32); i = assigned; assigned };
vm_block = 3; continue;
}
// C line 53306
22 => {
let _ = { let assigned = (0 as i32); ret = assigned; assigned };
vm_block = 21; continue;
}
// C line 53305
23 => {
let _ = { let assigned = (0 as i32); (*(sr)).already_resolved = assigned; assigned };
vm_block = 22; continue;
}
// C line 53304
24 => {
let _ = { let assigned = (1 as i32); (*(sr)).ref_count = assigned; assigned };
vm_block = 23; continue;
}
// C line 53303
25 => {
return ((1 as i32)).wrapping_neg();
}
// C line 53302
26 => {
vm_block = if ((!(!(sr).is_null()) as i32)) != 0 { 25 } else { 24 }; continue;
}
// C line 53301
27 => {
let _ = { let assigned = ((js_malloc(ctx, (size_of::<JSPromiseFunctionDataResolved>() as usize))) as *mut JSPromiseFunctionDataResolved); sr = assigned; assigned };
vm_block = 26; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:53352. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_promise_resolve_function_call(mut ctx: *mut JSContext, mut func_obj: JSValue, mut this_val: JSValue, mut argc: i32, mut argv: *mut JSValue, mut flags: i32) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut p: *mut JSObject = core::mem::zeroed();
let mut s: *mut JSPromiseFunctionData = core::mem::zeroed();
let mut resolution: JSValue = core::mem::zeroed();
let mut args: [JSValue; 3] = core::mem::zeroed();
let mut then: JSValue = core::mem::zeroed();
let mut is_reject: i32 = core::mem::zeroed();
let mut error: JSValue = core::mem::zeroed();
let mut vm_block: usize = 28;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 53402
1 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) };
}
// C line 53390
2 => {
let _ = JS_FreeValue(ctx, error);
vm_block = 1; continue;
}
// C line 53389
3 => {
let _ = reject_promise(ctx, (*(s)).promise, error);
vm_block = 2; continue;
}
// C line ? labels: fail_reject
4 => {
let _ = { let assigned = JS_GetException(ctx); error = assigned; assigned };
vm_block = 3; continue;
}
// C line ? labels: done
5 => {
let _ = fulfill_or_reject_promise(ctx, (*(s)).promise, resolution, is_reject);
vm_block = 1; continue;
}
// C line 53392
6 => {
let _ = JS_FreeValue(ctx, then);
vm_block = 5; continue;
}
// C line 53400
7 => {
let _ = JS_FreeValue(ctx, then);
vm_block = 1; continue;
}
// C line 53399
8 => {
let _ = JS_EnqueueJob(ctx, Some(js_promise_resolve_thenable_job), (3 as i32), (args).as_mut_ptr());
vm_block = 7; continue;
}
// C line 53398
9 => {
let _ = { let assigned = then; *((args).as_mut_ptr()).offset(((2 as i32)) as isize) = assigned; assigned };
vm_block = 8; continue;
}
// C line 53397
10 => {
let _ = { let assigned = resolution; *((args).as_mut_ptr()).offset(((1 as i32)) as isize) = assigned; assigned };
vm_block = 9; continue;
}
// C line 53396
11 => {
let _ = { let assigned = (*(s)).promise; *((args).as_mut_ptr()).offset(((0 as i32)) as isize) = assigned; assigned };
vm_block = 10; continue;
}
// C line 53391
12 => {
vm_block = if ((!((JS_IsFunction(ctx, then)) != 0) as i32)) != 0 { 6 } else { 11 }; continue;
}
// C line 53385
13 => {
vm_block = if (JS_IsException(then)) != 0 { 4 } else { 12 }; continue;
}
// C line 53384
14 => {
let _ = { let assigned = JS_GetProperty(ctx, resolution, (((crate::quickjs_atom::JS_ATOM_then as i32)) as JSAtom)); then = assigned; assigned };
vm_block = 13; continue;
}
// C line 53379
15 => {
vm_block = 5; continue;
}
// C line 53382
16 => {
vm_block = 4; continue;
}
// C line 53381
17 => {
let _ = JS_ThrowTypeError(ctx, c"promise self resolution".as_ptr());
vm_block = 16; continue;
}
// C line 53380
18 => {
vm_block = if (js_same_value(ctx, resolution, (*(s)).promise)) != 0 { 17 } else { 14 }; continue;
}
// C line 53378
19 => {
vm_block = if (((((is_reject) != 0) || (((!((JS_IsObject(resolution)) != 0) as i32)) != 0)) as i32)) != 0 { 15 } else { 18 }; continue;
}
// C line 53370
20 => {
let _ = { let assigned = *(argv).offset(((0 as i32)) as isize); resolution = assigned; assigned };
vm_block = 19; continue;
}
// C line 53372
21 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; resolution = assigned; assigned };
vm_block = 19; continue;
}
// C line 53369
22 => {
vm_block = if ((((argc) > ((0 as i32))) as i32)) != 0 { 20 } else { 21 }; continue;
}
// C line 53368
23 => {
let _ = { let assigned = ((((*(p)).class_id) as i32)).wrapping_sub((JS_CLASS_PROMISE_RESOLVE_FUNCTION as i32)); is_reject = assigned; assigned };
vm_block = 22; continue;
}
// C line 53367
24 => {
let _ = { let assigned = (1 as i32); (*((*(s)).presolved)).already_resolved = assigned; assigned };
vm_block = 23; continue;
}
// C line 53366
25 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) };
}
// C line 53365
26 => {
vm_block = if ((((((!(!(s).is_null()) as i32)) != 0) || (((*((*(s)).presolved)).already_resolved) != 0)) as i32)) != 0 { 25 } else { 24 }; continue;
}
// C line 53364
27 => {
let _ = { let assigned = ((*(p)).u).promise_function_data; s = assigned; assigned };
vm_block = 26; continue;
}
// C line 53358
28 => {
p = ((((func_obj).u).ptr) as *mut JSObject);
vm_block = 27; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:53445. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_promise_constructor(mut ctx: *mut JSContext, mut new_target: JSValue, mut argc: i32, mut argv: *mut JSValue) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut executor: JSValue = core::mem::zeroed();
let mut obj: JSValue = core::mem::zeroed();
let mut s: *mut JSPromiseData = core::mem::zeroed();
let mut args: [JSValue; 2] = core::mem::zeroed();
let mut ret: JSValue = core::mem::zeroed();
let mut i: i32 = core::mem::zeroed();
let mut ret2: JSValue = core::mem::zeroed();
let mut error: JSValue = core::mem::zeroed();
let mut vm_block: usize = 35;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 53490
1 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line ? labels: fail
2 => {
let _ = JS_FreeValue(ctx, obj);
vm_block = 1; continue;
}
// C line 53487
3 => {
let _ = JS_FreeValue(ctx, *((args).as_mut_ptr()).offset(((1 as i32)) as isize));
vm_block = 2; continue;
}
// C line ? labels: fail1
4 => {
let _ = JS_FreeValue(ctx, *((args).as_mut_ptr()).offset(((0 as i32)) as isize));
vm_block = 3; continue;
}
// C line 53484
5 => {
return obj;
}
// C line 53483
6 => {
let _ = JS_FreeValue(ctx, *((args).as_mut_ptr()).offset(((1 as i32)) as isize));
vm_block = 5; continue;
}
// C line 53482
7 => {
let _ = JS_FreeValue(ctx, *((args).as_mut_ptr()).offset(((0 as i32)) as isize));
vm_block = 6; continue;
}
// C line 53481
8 => {
let _ = JS_FreeValue(ctx, ret);
vm_block = 7; continue;
}
// C line 53479
9 => {
let _ = JS_FreeValue(ctx, ret2);
vm_block = 8; continue;
}
// C line 53478
10 => {
vm_block = 4; continue;
}
// C line 53477
11 => {
vm_block = if (JS_IsException(ret2)) != 0 { 10 } else { 9 }; continue;
}
// C line 53476
12 => {
let _ = JS_FreeValue(ctx, error);
vm_block = 11; continue;
}
// C line 53475
13 => {
let _ = { let assigned = JS_Call(ctx, *((args).as_mut_ptr()).offset(((1 as i32)) as isize), JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }, (1 as i32), core::ptr::addr_of_mut!(error)); ret2 = assigned; assigned };
vm_block = 12; continue;
}
// C line 53474
14 => {
let _ = { let assigned = JS_GetException(ctx); error = assigned; assigned };
vm_block = 13; continue;
}
// C line 53472
15 => {
vm_block = if (JS_IsException(ret)) != 0 { 14 } else { 8 }; continue;
}
// C line 53471
16 => {
let _ = { let assigned = JS_Call(ctx, executor, JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }, (2 as i32), (args).as_mut_ptr()); ret = assigned; assigned };
vm_block = 15; continue;
}
// C line 53470
17 => {
vm_block = 2; continue;
}
// C line 53469
18 => {
vm_block = if (js_create_resolving_functions(ctx, (args).as_mut_ptr(), obj)) != 0 { 17 } else { 16 }; continue;
}
// C line 53468
19 => {
let _ = JS_SetOpaque(obj, ((s) as *mut c_void));
vm_block = 18; continue;
}
// C line 53467
20 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; (*(s)).promise_result = assigned; assigned };
vm_block = 19; continue;
}
// C line 53465
21 => {
vm_block = if ((((i) < ((2 as i32))) as i32)) != 0 { 23 } else { 20 }; continue;
}
// C line ?
22 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 21; continue;
}
// C line 53466
23 => {
let _ = init_list_head(core::ptr::addr_of_mut!(*(((*(s)).promise_reactions).as_mut_ptr()).offset((i) as isize)));
vm_block = 22; continue;
}
// C line 53465
24 => {
let _ = { let assigned = (0 as i32); i = assigned; assigned };
vm_block = 21; continue;
}
// C line 53464
25 => {
let _ = { let assigned = (0 as i32); (*(s)).is_handled = assigned; assigned };
vm_block = 24; continue;
}
// C line 53463
26 => {
let _ = { let assigned = (((JS_PROMISE_PENDING as i32)) as JSPromiseStateEnum); (*(s)).promise_state = assigned; assigned };
vm_block = 25; continue;
}
// C line 53462
27 => {
vm_block = 2; continue;
}
// C line 53461
28 => {
vm_block = if ((!(!(s).is_null()) as i32)) != 0 { 27 } else { 26 }; continue;
}
// C line 53460
29 => {
let _ = { let assigned = ((js_mallocz(ctx, (size_of::<JSPromiseData>() as usize))) as *mut JSPromiseData); s = assigned; assigned };
vm_block = 28; continue;
}
// C line 53459
30 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 53458
31 => {
vm_block = if (JS_IsException(obj)) != 0 { 30 } else { 29 }; continue;
}
// C line 53457
32 => {
let _ = { let assigned = js_create_from_ctor(ctx, new_target, (JS_CLASS_PROMISE as i32)); obj = assigned; assigned };
vm_block = 31; continue;
}
// C line 53456
33 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 53455
34 => {
vm_block = if (check_function(ctx, executor)) != 0 { 33 } else { 32 }; continue;
}
// C line 53454
35 => {
let _ = { let assigned = *(argv).offset(((0 as i32)) as isize); executor = assigned; assigned };
vm_block = 34; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:53493. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_promise_executor(mut ctx: *mut JSContext, mut this_val: JSValue, mut argc: i32, mut argv: *mut JSValue, mut magic: i32, mut func_data: *mut JSValue) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut i: i32 = core::mem::zeroed();
let mut vm_block: usize = 7;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 53505
1 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) };
}
// C line 53500
2 => {
vm_block = if ((((i) < ((2 as i32))) as i32)) != 0 { 6 } else { 1 }; continue;
}
// C line ?
3 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 2; continue;
}
// C line 53503
4 => {
let _ = { let assigned = JS_DupValue(ctx, *(argv).offset((i) as isize)); *(func_data).offset((i) as isize) = assigned; assigned };
vm_block = 3; continue;
}
// C line 53502
5 => {
return JS_ThrowTypeError(ctx, c"resolving function already set".as_ptr());
}
// C line 53501
6 => {
vm_block = if ((!((JS_IsUndefined(*(func_data).offset((i) as isize))) != 0) as i32)) != 0 { 5 } else { 4 }; continue;
}
// C line 53500
7 => {
let _ = { let assigned = (0 as i32); i = assigned; assigned };
vm_block = 2; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:53508. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_promise_executor_new(mut ctx: *mut JSContext) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut func_data: [JSValue; 2] = core::mem::zeroed();
let mut vm_block: usize = 3;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 53514
1 => {
return JS_NewCFunctionData(ctx, Some(js_promise_executor), (2 as i32), (0 as i32), (2 as i32), (func_data).as_mut_ptr());
}
// C line 53513
2 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; *((func_data).as_mut_ptr()).offset(((1 as i32)) as isize) = assigned; assigned };
vm_block = 1; continue;
}
// C line 53512
3 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; *((func_data).as_mut_ptr()).offset(((0 as i32)) as isize) = assigned; assigned };
vm_block = 2; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:53518. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_new_promise_capability(mut ctx: *mut JSContext, mut resolving_funcs: *mut JSValue, mut ctor: JSValue) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut executor: JSValue = core::mem::zeroed();
let mut result_promise: JSValue = core::mem::zeroed();
let mut s: *mut JSCFunctionDataRecord = core::mem::zeroed();
let mut i: i32 = core::mem::zeroed();
let mut vm_block: usize = 23;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 53551
1 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 53550
2 => {
let _ = JS_FreeValue(ctx, result_promise);
vm_block = 1; continue;
}
// C line ? labels: fail
3 => {
let _ = JS_FreeValue(ctx, executor);
vm_block = 2; continue;
}
// C line 53547
4 => {
return result_promise;
}
// C line 53546
5 => {
let _ = JS_FreeValue(ctx, executor);
vm_block = 4; continue;
}
// C line 53544
6 => {
vm_block = if ((((i) < ((2 as i32))) as i32)) != 0 { 8 } else { 5 }; continue;
}
// C line ?
7 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 6; continue;
}
// C line 53545
8 => {
let _ = { let assigned = JS_DupValue(ctx, *(((*(s)).data).as_mut_ptr()).offset((i) as isize)); *(resolving_funcs).offset((i) as isize) = assigned; assigned };
vm_block = 7; continue;
}
// C line 53544
9 => {
let _ = { let assigned = (0 as i32); i = assigned; assigned };
vm_block = 6; continue;
}
// C line 53540
10 => {
vm_block = if ((((i) < ((2 as i32))) as i32)) != 0 { 13 } else { 9 }; continue;
}
// C line ?
11 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 10; continue;
}
// C line 53542
12 => {
vm_block = 3; continue;
}
// C line 53541
13 => {
vm_block = if (check_function(ctx, *(((*(s)).data).as_mut_ptr()).offset((i) as isize))) != 0 { 12 } else { 11 }; continue;
}
// C line 53540
14 => {
let _ = { let assigned = (0 as i32); i = assigned; assigned };
vm_block = 10; continue;
}
// C line 53539
15 => {
let _ = { let assigned = ((JS_GetOpaque(executor, (((JS_CLASS_C_FUNCTION_DATA as i32)) as JSClassID))) as *mut JSCFunctionDataRecord); s = assigned; assigned };
vm_block = 14; continue;
}
// C line 53538
16 => {
vm_block = 3; continue;
}
// C line 53537
17 => {
vm_block = if (JS_IsException(result_promise)) != 0 { 16 } else { 15 }; continue;
}
// C line 53531
18 => {
let _ = { let assigned = js_promise_constructor(ctx, ctor, (1 as i32), core::ptr::addr_of_mut!(executor)); result_promise = assigned; assigned };
vm_block = 17; continue;
}
// C line 53534
19 => {
let _ = { let assigned = JS_CallConstructor(ctx, ctor, (1 as i32), core::ptr::addr_of_mut!(executor)); result_promise = assigned; assigned };
vm_block = 17; continue;
}
// C line 53530
20 => {
vm_block = if (JS_IsUndefined(ctor)) != 0 { 18 } else { 19 }; continue;
}
// C line 53528
21 => {
return executor;
}
// C line 53527
22 => {
vm_block = if (JS_IsException(executor)) != 0 { 21 } else { 20 }; continue;
}
// C line 53526
23 => {
let _ = { let assigned = js_promise_executor_new(ctx); executor = assigned; assigned };
vm_block = 22; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:53554. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
pub unsafe fn JS_NewPromiseCapability(mut ctx: *mut JSContext, mut resolving_funcs: *mut JSValue) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut vm_block: usize = 1;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 53556
1 => {
return js_new_promise_capability(ctx, resolving_funcs, JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) });
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:53559. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_promise_resolve(mut ctx: *mut JSContext, mut this_val: JSValue, mut argc: i32, mut argv: *mut JSValue, mut magic: i32) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut result_promise: JSValue = core::mem::zeroed();
let mut resolving_funcs: [JSValue; 2] = core::mem::zeroed();
let mut ret: JSValue = core::mem::zeroed();
let mut is_reject: i32 = core::mem::zeroed();
let mut ctor: JSValue = core::mem::zeroed();
let mut is_same: i32 = core::mem::zeroed();
let mut vm_block: usize = 22;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 53589
1 => {
return result_promise;
}
// C line 53588
2 => {
let _ = JS_FreeValue(ctx, ret);
vm_block = 1; continue;
}
// C line 53586
3 => {
return ret;
}
// C line 53585
4 => {
let _ = JS_FreeValue(ctx, result_promise);
vm_block = 3; continue;
}
// C line 53584
5 => {
vm_block = if (JS_IsException(ret)) != 0 { 4 } else { 2 }; continue;
}
// C line 53583
6 => {
let _ = JS_FreeValue(ctx, *((resolving_funcs).as_mut_ptr()).offset(((1 as i32)) as isize));
vm_block = 5; continue;
}
// C line 53582
7 => {
let _ = JS_FreeValue(ctx, *((resolving_funcs).as_mut_ptr()).offset(((0 as i32)) as isize));
vm_block = 6; continue;
}
// C line 53581
8 => {
let _ = { let assigned = JS_Call(ctx, *((resolving_funcs).as_mut_ptr()).offset((is_reject) as isize), JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }, (1 as i32), argv); ret = assigned; assigned };
vm_block = 7; continue;
}
// C line 53580
9 => {
return result_promise;
}
// C line 53579
10 => {
vm_block = if (JS_IsException(result_promise)) != 0 { 9 } else { 8 }; continue;
}
// C line 53578
11 => {
let _ = { let assigned = js_new_promise_capability(ctx, (resolving_funcs).as_mut_ptr(), this_val); result_promise = assigned; assigned };
vm_block = 10; continue;
}
// C line 53576
12 => {
return JS_DupValue(ctx, *(argv).offset(((0 as i32)) as isize));
}
// C line 53575
13 => {
vm_block = if (is_same) != 0 { 12 } else { 11 }; continue;
}
// C line 53574
14 => {
let _ = JS_FreeValue(ctx, ctor);
vm_block = 13; continue;
}
// C line 53573
15 => {
let _ = { let assigned = js_same_value(ctx, ctor, this_val); is_same = assigned; assigned };
vm_block = 14; continue;
}
// C line 53572
16 => {
return ctor;
}
// C line 53571
17 => {
vm_block = if (JS_IsException(ctor)) != 0 { 16 } else { 15 }; continue;
}
// C line 53570
18 => {
let _ = { let assigned = JS_GetProperty(ctx, *(argv).offset(((0 as i32)) as isize), (((crate::quickjs_atom::JS_ATOM_constructor as i32)) as JSAtom)); ctor = assigned; assigned };
vm_block = 17; continue;
}
// C line 53567
19 => {
vm_block = if ((((((!((is_reject) != 0) as i32)) != 0) && (!(JS_GetOpaque(*(argv).offset(((0 as i32)) as isize), (((JS_CLASS_PROMISE as i32)) as JSClassID))).is_null())) as i32)) != 0 { 18 } else { 11 }; continue;
}
// C line 53566
20 => {
return JS_ThrowTypeErrorNotAnObject(ctx);
}
// C line 53565
21 => {
vm_block = if ((!((JS_IsObject(this_val)) != 0) as i32)) != 0 { 20 } else { 19 }; continue;
}
// C line 53563
22 => {
is_reject = magic;
vm_block = 21; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:53592. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_promise_withResolvers(mut ctx: *mut JSContext, mut this_val: JSValue, mut argc: i32, mut argv: *mut JSValue) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut result_promise: JSValue = core::mem::zeroed();
let mut resolving_funcs: [JSValue; 2] = core::mem::zeroed();
let mut obj: JSValue = core::mem::zeroed();
let mut vm_block: usize = 22;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 53625
1 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 53624
2 => {
let _ = JS_FreeValue(ctx, obj);
vm_block = 1; continue;
}
// C line 53623
3 => {
let _ = JS_FreeValue(ctx, result_promise);
vm_block = 2; continue;
}
// C line 53622
4 => {
let _ = JS_FreeValue(ctx, *((resolving_funcs).as_mut_ptr()).offset(((1 as i32)) as isize));
vm_block = 3; continue;
}
// C line ? labels: exception
5 => {
let _ = JS_FreeValue(ctx, *((resolving_funcs).as_mut_ptr()).offset(((0 as i32)) as isize));
vm_block = 4; continue;
}
// C line 53619
6 => {
return obj;
}
// C line 53617
7 => {
vm_block = 5; continue;
}
// C line 53615
8 => {
vm_block = if ((((JS_DefinePropertyValue(ctx, obj, (((crate::quickjs_atom::JS_ATOM_reject as i32)) as JSAtom), *((resolving_funcs).as_mut_ptr()).offset(((1 as i32)) as isize), ((((((1 as i32)).wrapping_shl(((0 as i32)) as u32)) | (((1 as i32)).wrapping_shl(((1 as i32)) as u32)))) | (((1 as i32)).wrapping_shl(((2 as i32)) as u32))))) < ((0 as i32))) as i32)) != 0 { 7 } else { 6 }; continue;
}
// C line 53614
9 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; *((resolving_funcs).as_mut_ptr()).offset(((0 as i32)) as isize) = assigned; assigned };
vm_block = 8; continue;
}
// C line 53612
10 => {
vm_block = 5; continue;
}
// C line 53610
11 => {
vm_block = if ((((JS_DefinePropertyValue(ctx, obj, (((crate::quickjs_atom::JS_ATOM_resolve as i32)) as JSAtom), *((resolving_funcs).as_mut_ptr()).offset(((0 as i32)) as isize), ((((((1 as i32)).wrapping_shl(((0 as i32)) as u32)) | (((1 as i32)).wrapping_shl(((1 as i32)) as u32)))) | (((1 as i32)).wrapping_shl(((2 as i32)) as u32))))) < ((0 as i32))) as i32)) != 0 { 10 } else { 9 }; continue;
}
// C line 53609
12 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; result_promise = assigned; assigned };
vm_block = 11; continue;
}
// C line 53607
13 => {
vm_block = 5; continue;
}
// C line 53605
14 => {
vm_block = if ((((JS_DefinePropertyValue(ctx, obj, (((crate::quickjs_atom::JS_ATOM_promise as i32)) as JSAtom), result_promise, ((((((1 as i32)).wrapping_shl(((0 as i32)) as u32)) | (((1 as i32)).wrapping_shl(((1 as i32)) as u32)))) | (((1 as i32)).wrapping_shl(((2 as i32)) as u32))))) < ((0 as i32))) as i32)) != 0 { 13 } else { 12 }; continue;
}
// C line 53604
15 => {
vm_block = 5; continue;
}
// C line 53603
16 => {
vm_block = if (JS_IsException(obj)) != 0 { 15 } else { 14 }; continue;
}
// C line 53602
17 => {
let _ = { let assigned = JS_NewObject(ctx); obj = assigned; assigned };
vm_block = 16; continue;
}
// C line 53601
18 => {
return result_promise;
}
// C line 53600
19 => {
vm_block = if (JS_IsException(result_promise)) != 0 { 18 } else { 17 }; continue;
}
// C line 53599
20 => {
let _ = { let assigned = js_new_promise_capability(ctx, (resolving_funcs).as_mut_ptr(), this_val); result_promise = assigned; assigned };
vm_block = 19; continue;
}
// C line 53598
21 => {
return JS_ThrowTypeErrorNotAnObject(ctx);
}
// C line 53597
22 => {
vm_block = if ((!((JS_IsObject(this_val)) != 0) as i32)) != 0 { 21 } else { 20 }; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:53628. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_promise_try(mut ctx: *mut JSContext, mut this_val: JSValue, mut argc: i32, mut argv: *mut JSValue) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut result_promise: JSValue = core::mem::zeroed();
let mut resolving_funcs: [JSValue; 2] = core::mem::zeroed();
let mut ret: JSValue = core::mem::zeroed();
let mut ret2: JSValue = core::mem::zeroed();
let mut is_reject: i32 = core::mem::zeroed();
let mut vm_block: usize = 19;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 53653
1 => {
return result_promise;
}
// C line 53652
2 => {
let _ = JS_FreeValue(ctx, ret2);
vm_block = 1; continue;
}
// C line 53650
3 => {
return ret2;
}
// C line 53649
4 => {
let _ = JS_FreeValue(ctx, result_promise);
vm_block = 3; continue;
}
// C line 53648
5 => {
vm_block = if (JS_IsException(ret2)) != 0 { 4 } else { 2 }; continue;
}
// C line 53647
6 => {
let _ = JS_FreeValue(ctx, ret);
vm_block = 5; continue;
}
// C line 53646
7 => {
let _ = JS_FreeValue(ctx, *((resolving_funcs).as_mut_ptr()).offset(((1 as i32)) as isize));
vm_block = 6; continue;
}
// C line 53645
8 => {
let _ = JS_FreeValue(ctx, *((resolving_funcs).as_mut_ptr()).offset(((0 as i32)) as isize));
vm_block = 7; continue;
}
// C line 53644
9 => {
let _ = { let assigned = JS_Call(ctx, *((resolving_funcs).as_mut_ptr()).offset((is_reject) as isize), JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }, (1 as i32), core::ptr::addr_of_mut!(ret)); ret2 = assigned; assigned };
vm_block = 8; continue;
}
// C line 53642
10 => {
let _ = { let assigned = JS_GetException(ctx); ret = assigned; assigned };
vm_block = 9; continue;
}
// C line 53641
11 => {
let _ = { let assigned = (1 as i32); is_reject = assigned; assigned };
vm_block = 10; continue;
}
// C line 53640
12 => {
vm_block = if (JS_IsException(ret)) != 0 { 11 } else { 9 }; continue;
}
// C line 53639
13 => {
let _ = { let assigned = JS_Call(ctx, *(argv).offset(((0 as i32)) as isize), JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }, (argc).wrapping_sub((1 as i32)), (argv).offset((((1 as i32)) as isize))); ret = assigned; assigned };
vm_block = 12; continue;
}
// C line 53638
14 => {
return result_promise;
}
// C line 53637
15 => {
vm_block = if (JS_IsException(result_promise)) != 0 { 14 } else { 13 }; continue;
}
// C line 53636
16 => {
let _ = { let assigned = js_new_promise_capability(ctx, (resolving_funcs).as_mut_ptr(), this_val); result_promise = assigned; assigned };
vm_block = 15; continue;
}
// C line 53635
17 => {
return JS_ThrowTypeErrorNotAnObject(ctx);
}
// C line 53634
18 => {
vm_block = if ((!((JS_IsObject(this_val)) != 0) as i32)) != 0 { 17 } else { 16 }; continue;
}
// C line 53632
19 => {
is_reject = (0 as i32);
vm_block = 18; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:53656. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn remainingElementsCount_add(mut ctx: *mut JSContext, mut resolve_element_env: JSValue, mut addend: i32) -> i32 {
let mut vm_local_storage = Vec::<u64>::new();
let mut val: JSValue = core::mem::zeroed();
let mut remainingElementsCount: i32 = core::mem::zeroed();
let mut vm_block: usize = 9;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 53672
1 => {
return (((remainingElementsCount) == ((0 as i32))) as i32);
}
// C line 53671
2 => {
return ((1 as i32)).wrapping_neg();
}
// C line 53669
3 => {
vm_block = if ((((JS_SetPropertyUint32(ctx, resolve_element_env, (((0 as i32)) as u32), JS_NewInt32(ctx, remainingElementsCount))) < ((0 as i32))) as i32)) != 0 { 2 } else { 1 }; continue;
}
// C line 53668
4 => {
let _ = { remainingElementsCount = (remainingElementsCount).wrapping_add(addend); remainingElementsCount };
vm_block = 3; continue;
}
// C line 53667
5 => {
return ((1 as i32)).wrapping_neg();
}
// C line 53666
6 => {
vm_block = if (JS_ToInt32Free(ctx, core::ptr::addr_of_mut!(remainingElementsCount), val)) != 0 { 5 } else { 4 }; continue;
}
// C line 53665
7 => {
return ((1 as i32)).wrapping_neg();
}
// C line 53664
8 => {
vm_block = if (JS_IsException(val)) != 0 { 7 } else { 6 }; continue;
}
// C line 53663
9 => {
let _ = { let assigned = JS_GetPropertyUint32(ctx, resolve_element_env, (((0 as i32)) as u32)); val = assigned; assigned };
vm_block = 8; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:53679. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_promise_all_resolve_element(mut ctx: *mut JSContext, mut this_val: JSValue, mut argc: i32, mut argv: *mut JSValue, mut magic: i32, mut func_data: *mut JSValue) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut resolve_type: i32 = core::mem::zeroed();
let mut is_reject: i32 = core::mem::zeroed();
let mut alreadyCalled: i32 = core::mem::zeroed();
let mut values: JSValue = core::mem::zeroed();
let mut resolve: JSValue = core::mem::zeroed();
let mut resolve_element_env: JSValue = core::mem::zeroed();
let mut ret: JSValue = core::mem::zeroed();
let mut obj: JSValue = core::mem::zeroed();
let mut is_zero: i32 = core::mem::zeroed();
let mut index: i32 = core::mem::zeroed();
let mut str: JSValue = core::mem::zeroed();
let mut error: JSValue = core::mem::zeroed();
let mut vm_block: usize = 41;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 53746
1 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) };
}
// C line 53744
2 => {
let _ = JS_FreeValue(ctx, ret);
vm_block = 1; continue;
}
// C line 53743
3 => {
return ret;
}
// C line 53742
4 => {
vm_block = if (JS_IsException(ret)) != 0 { 3 } else { 2 }; continue;
}
// C line 53738
5 => {
let _ = JS_FreeValue(ctx, error);
vm_block = 4; continue;
}
// C line 53737
6 => {
let _ = { let assigned = JS_Call(ctx, resolve, JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }, (1 as i32), core::ptr::addr_of_mut!(error)); ret = assigned; assigned };
vm_block = 5; continue;
}
// C line 53736
7 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 53735
8 => {
vm_block = if (JS_IsException(error)) != 0 { 7 } else { 6 }; continue;
}
// C line 53734
9 => {
let _ = { let assigned = js_aggregate_error_constructor(ctx, values); error = assigned; assigned };
vm_block = 8; continue;
}
// C line 53740
10 => {
let _ = { let assigned = JS_Call(ctx, resolve, JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }, (1 as i32), core::ptr::addr_of_mut!(values)); ret = assigned; assigned };
vm_block = 4; continue;
}
// C line 53732
11 => {
vm_block = if ((((resolve_type) == ((2 as i32))) as i32)) != 0 { 9 } else { 10 }; continue;
}
// C line 53731
12 => {
vm_block = if (is_zero) != 0 { 11 } else { 1 }; continue;
}
// C line 53730
13 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 53729
14 => {
vm_block = if ((((is_zero) < ((0 as i32))) as i32)) != 0 { 13 } else { 12 }; continue;
}
// C line 53728
15 => {
let _ = { let assigned = remainingElementsCount_add(ctx, resolve_element_env, ((1 as i32)).wrapping_neg()); is_zero = assigned; assigned };
vm_block = 14; continue;
}
// C line 53726
16 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 53724
17 => {
vm_block = if ((((JS_DefinePropertyValueUint32(ctx, values, ((index) as u32), obj, ((((((1 as i32)).wrapping_shl(((0 as i32)) as u32)) | (((1 as i32)).wrapping_shl(((1 as i32)) as u32)))) | (((1 as i32)).wrapping_shl(((2 as i32)) as u32))))) < ((0 as i32))) as i32)) != 0 { 16 } else { 15 }; continue;
}
// C line 53719
18 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line ? labels: fail1
19 => {
let _ = JS_FreeValue(ctx, obj);
vm_block = 18; continue;
}
// C line 53713
20 => {
vm_block = if ((((JS_DefinePropertyValue(ctx, obj, ((if (is_reject) != 0 { (crate::quickjs_atom::JS_ATOM_reason as i32) } else { (crate::quickjs_atom::JS_ATOM_value as i32) }) as JSAtom), JS_DupValue(ctx, *(argv).offset(((0 as i32)) as isize)), ((((((1 as i32)).wrapping_shl(((0 as i32)) as u32)) | (((1 as i32)).wrapping_shl(((1 as i32)) as u32)))) | (((1 as i32)).wrapping_shl(((2 as i32)) as u32))))) < ((0 as i32))) as i32)) != 0 { 19 } else { 17 }; continue;
}
// C line 53712
21 => {
vm_block = 19; continue;
}
// C line 53709
22 => {
vm_block = if ((((JS_DefinePropertyValue(ctx, obj, (((crate::quickjs_atom::JS_ATOM_status as i32)) as JSAtom), str, ((((((1 as i32)).wrapping_shl(((0 as i32)) as u32)) | (((1 as i32)).wrapping_shl(((1 as i32)) as u32)))) | (((1 as i32)).wrapping_shl(((2 as i32)) as u32))))) < ((0 as i32))) as i32)) != 0 { 21 } else { 20 }; continue;
}
// C line 53708
23 => {
vm_block = 19; continue;
}
// C line 53707
24 => {
vm_block = if (JS_IsException(str)) != 0 { 23 } else { 22 }; continue;
}
// C line 53706
25 => {
let _ = { let assigned = js_new_string8(ctx, if (is_reject) != 0 { c"rejected".as_ptr() } else { c"fulfilled".as_ptr() }); str = assigned; assigned };
vm_block = 24; continue;
}
// C line 53705
26 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 53704
27 => {
vm_block = if (JS_IsException(obj)) != 0 { 26 } else { 25 }; continue;
}
// C line 53703
28 => {
let _ = { let assigned = JS_NewObject(ctx); obj = assigned; assigned };
vm_block = 27; continue;
}
// C line 53722
29 => {
let _ = { let assigned = JS_DupValue(ctx, *(argv).offset(((0 as i32)) as isize)); obj = assigned; assigned };
vm_block = 17; continue;
}
// C line 53700
30 => {
vm_block = if ((((resolve_type) == ((1 as i32))) as i32)) != 0 { 28 } else { 29 }; continue;
}
// C line 53698
31 => {
let _ = { let assigned = JS_NewBool(ctx, (1 as i32)); *(func_data).offset(((0 as i32)) as isize) = assigned; assigned };
vm_block = 30; continue;
}
// C line 53697
32 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) };
}
// C line 53696
33 => {
vm_block = if (alreadyCalled) != 0 { 32 } else { 31 }; continue;
}
// C line 53695
34 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 53694
35 => {
vm_block = if (JS_ToInt32(ctx, core::ptr::addr_of_mut!(index), *(func_data).offset(((1 as i32)) as isize))) != 0 { 34 } else { 33 }; continue;
}
// C line 53690
36 => {
resolve_element_env = *(func_data).offset(((4 as i32)) as isize);
vm_block = 35; continue;
}
// C line 53689
37 => {
resolve = *(func_data).offset(((3 as i32)) as isize);
vm_block = 36; continue;
}
// C line 53688
38 => {
values = *(func_data).offset(((2 as i32)) as isize);
vm_block = 37; continue;
}
// C line 53687
39 => {
alreadyCalled = JS_ToBool(ctx, *(func_data).offset(((0 as i32)) as isize));
vm_block = 38; continue;
}
// C line 53686
40 => {
is_reject = ((magic) & ((4 as i32)));
vm_block = 39; continue;
}
// C line 53685
41 => {
resolve_type = ((magic) & ((3 as i32)));
vm_block = 40; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:53750. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_promise_all(mut ctx: *mut JSContext, mut this_val: JSValue, mut argc: i32, mut argv: *mut JSValue, mut magic: i32) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut result_promise: JSValue = core::mem::zeroed();
let mut resolving_funcs: [JSValue; 2] = core::mem::zeroed();
let mut item: JSValue = core::mem::zeroed();
let mut next_promise: JSValue = core::mem::zeroed();
let mut ret: JSValue = core::mem::zeroed();
let mut next_method: JSValue = core::mem::zeroed();
let mut values: JSValue = core::mem::zeroed();
let mut resolve_element_env: JSValue = core::mem::zeroed();
let mut resolve_element: JSValue = core::mem::zeroed();
let mut reject_element: JSValue = core::mem::zeroed();
let mut promise_resolve: JSValue = core::mem::zeroed();
let mut iter: JSValue = core::mem::zeroed();
let mut then_args: [JSValue; 2] = core::mem::zeroed();
let mut resolve_element_data: [JSValue; 5] = core::mem::zeroed();
let mut done: i32 = core::mem::zeroed();
let mut index: i32 = core::mem::zeroed();
let mut is_zero: i32 = core::mem::zeroed();
let mut is_promise_any: i32 = core::mem::zeroed();
let mut error: JSValue = core::mem::zeroed();
let mut error_1: JSValue = core::mem::zeroed();
let mut vm_block: usize = 100;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 53892
1 => {
vm_block = 11; continue;
}
// C line 53891
2 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) }; result_promise = assigned; assigned };
vm_block = 1; continue;
}
// C line ? labels: fail
3 => {
let _ = JS_FreeValue(ctx, result_promise);
vm_block = 2; continue;
}
// C line 53888
4 => {
return result_promise;
}
// C line 53887
5 => {
let _ = JS_FreeValue(ctx, *((resolving_funcs).as_mut_ptr()).offset(((1 as i32)) as isize));
vm_block = 4; continue;
}
// C line 53886
6 => {
let _ = JS_FreeValue(ctx, *((resolving_funcs).as_mut_ptr()).offset(((0 as i32)) as isize));
vm_block = 5; continue;
}
// C line 53885
7 => {
let _ = JS_FreeValue(ctx, iter);
vm_block = 6; continue;
}
// C line 53884
8 => {
let _ = JS_FreeValue(ctx, next_method);
vm_block = 7; continue;
}
// C line 53883
9 => {
let _ = JS_FreeValue(ctx, values);
vm_block = 8; continue;
}
// C line 53882
10 => {
let _ = JS_FreeValue(ctx, resolve_element_env);
vm_block = 9; continue;
}
// C line ? labels: done
11 => {
let _ = JS_FreeValue(ctx, promise_resolve);
vm_block = 10; continue;
}
// C line 53780
12 => {
let _ = JS_FreeValue(ctx, ret);
vm_block = 11; continue;
}
// C line 53779
13 => {
vm_block = 3; continue;
}
// C line 53778
14 => {
vm_block = if (JS_IsException(ret)) != 0 { 13 } else { 12 }; continue;
}
// C line 53777
15 => {
let _ = JS_FreeValue(ctx, error);
vm_block = 14; continue;
}
// C line 53775
16 => {
let _ = { let assigned = JS_Call(ctx, *((resolving_funcs).as_mut_ptr()).offset(((1 as i32)) as isize), JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }, (1 as i32), core::ptr::addr_of_mut!(error)); ret = assigned; assigned };
vm_block = 15; continue;
}
// C line ? labels: fail_reject
17 => {
let _ = { let assigned = JS_GetException(ctx); error = assigned; assigned };
vm_block = 16; continue;
}
// C line 53877
18 => {
vm_block = 17; continue;
}
// C line 53876
19 => {
vm_block = if (check_exception_free(ctx, ret)) != 0 { 18 } else { 11 }; continue;
}
// C line 53874
20 => {
let _ = { let assigned = JS_Call(ctx, *((resolving_funcs).as_mut_ptr()).offset((is_promise_any) as isize), JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }, (1 as i32), core::ptr::addr_of_mut!(values)); ret = assigned; assigned };
vm_block = 19; continue;
}
// C line 53872
21 => {
let _ = { let assigned = error_1; values = assigned; assigned };
vm_block = 20; continue;
}
// C line 53871
22 => {
let _ = JS_FreeValue(ctx, values);
vm_block = 21; continue;
}
// C line 53870
23 => {
vm_block = 17; continue;
}
// C line 53869
24 => {
vm_block = if (JS_IsException(error_1)) != 0 { 23 } else { 22 }; continue;
}
// C line 53868
25 => {
let _ = { let assigned = js_aggregate_error_constructor(ctx, values); error_1 = assigned; assigned };
vm_block = 24; continue;
}
// C line 53866
26 => {
vm_block = if ((((magic) == ((2 as i32))) as i32)) != 0 { 25 } else { 20 }; continue;
}
// C line 53865
27 => {
vm_block = if (is_zero) != 0 { 26 } else { 11 }; continue;
}
// C line 53864
28 => {
vm_block = 17; continue;
}
// C line 53863
29 => {
vm_block = if ((((is_zero) < ((0 as i32))) as i32)) != 0 { 28 } else { 27 }; continue;
}
// C line 53862
30 => {
let _ = { let assigned = remainingElementsCount_add(ctx, resolve_element_env, ((1 as i32)).wrapping_neg()); is_zero = assigned; assigned };
vm_block = 29; continue;
}
// C line 53798
31 => {
vm_block = 74; continue;
}
// C line 53859
32 => {
let _ = { let old = index; index = (index).wrapping_add(1); old };
vm_block = 31; continue;
}
// C line 53858
33 => {
vm_block = 66; continue;
}
// C line 53857
34 => {
vm_block = if (check_exception_free(ctx, ret)) != 0 { 33 } else { 32 }; continue;
}
// C line 53856
35 => {
let _ = JS_FreeValue(ctx, reject_element);
vm_block = 34; continue;
}
// C line 53855
36 => {
let _ = JS_FreeValue(ctx, resolve_element);
vm_block = 35; continue;
}
// C line 53854
37 => {
let _ = { let assigned = JS_InvokeFree(ctx, next_promise, (((crate::quickjs_atom::JS_ATOM_then as i32)) as JSAtom), (2 as i32), (then_args).as_mut_ptr()); ret = assigned; assigned };
vm_block = 36; continue;
}
// C line 53853
38 => {
let _ = { let assigned = reject_element; *((then_args).as_mut_ptr()).offset(((1 as i32)) as isize) = assigned; assigned };
vm_block = 37; continue;
}
// C line 53852
39 => {
let _ = { let assigned = resolve_element; *((then_args).as_mut_ptr()).offset(((0 as i32)) as isize) = assigned; assigned };
vm_block = 38; continue;
}
// C line 53849
40 => {
vm_block = 66; continue;
}
// C line 53848
41 => {
let _ = JS_FreeValue(ctx, reject_element);
vm_block = 40; continue;
}
// C line 53847
42 => {
let _ = JS_FreeValue(ctx, resolve_element);
vm_block = 41; continue;
}
// C line 53846
43 => {
let _ = JS_FreeValue(ctx, next_promise);
vm_block = 42; continue;
}
// C line 53845
44 => {
vm_block = if ((((remainingElementsCount_add(ctx, resolve_element_env, (1 as i32))) < ((0 as i32))) as i32)) != 0 { 43 } else { 39 }; continue;
}
// C line 53833
45 => {
vm_block = 66; continue;
}
// C line 53832
46 => {
let _ = JS_FreeValue(ctx, next_promise);
vm_block = 45; continue;
}
// C line 53831
47 => {
vm_block = if (JS_IsException(reject_element)) != 0 { 46 } else { 44 }; continue;
}
// C line 53828
48 => {
let _ = { let assigned = JS_NewCFunctionData(ctx, Some(js_promise_all_resolve_element), (1 as i32), ((magic) | ((4 as i32))), (5 as i32), (resolve_element_data).as_mut_ptr()); reject_element = assigned; assigned };
vm_block = 47; continue;
}
// C line 53840
49 => {
let _ = { let assigned = JS_DupValue(ctx, *((resolving_funcs).as_mut_ptr()).offset(((0 as i32)) as isize)); resolve_element = assigned; assigned };
vm_block = 44; continue;
}
// C line 53839
50 => {
let _ = { let assigned = resolve_element; reject_element = assigned; assigned };
vm_block = 49; continue;
}
// C line 53838
51 => {
vm_block = 66; continue;
}
// C line 53836
52 => {
vm_block = if ((((JS_DefinePropertyValueUint32(ctx, values, ((index) as u32), JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }, ((((((1 as i32)).wrapping_shl(((0 as i32)) as u32)) | (((1 as i32)).wrapping_shl(((1 as i32)) as u32)))) | (((1 as i32)).wrapping_shl(((2 as i32)) as u32))))) < ((0 as i32))) as i32)) != 0 { 51 } else { 50 }; continue;
}
// C line 53842
53 => {
let _ = { let assigned = JS_DupValue(ctx, *((resolving_funcs).as_mut_ptr()).offset(((1 as i32)) as isize)); reject_element = assigned; assigned };
vm_block = 44; continue;
}
// C line 53835
54 => {
vm_block = if ((((magic) == ((2 as i32))) as i32)) != 0 { 52 } else { 53 }; continue;
}
// C line 53827
55 => {
vm_block = if ((((magic) == ((1 as i32))) as i32)) != 0 { 48 } else { 54 }; continue;
}
// C line 53824
56 => {
vm_block = 66; continue;
}
// C line 53823
57 => {
let _ = JS_FreeValue(ctx, next_promise);
vm_block = 56; continue;
}
// C line 53822
58 => {
vm_block = if (JS_IsException(resolve_element)) != 0 { 57 } else { 55 }; continue;
}
// C line 53819
59 => {
let _ = { let assigned = JS_NewCFunctionData(ctx, Some(js_promise_all_resolve_element), (1 as i32), magic, (5 as i32), (resolve_element_data).as_mut_ptr()); resolve_element = assigned; assigned };
vm_block = 58; continue;
}
// C line 53818
60 => {
let _ = { let assigned = resolve_element_env; *((resolve_element_data).as_mut_ptr()).offset(((4 as i32)) as isize) = assigned; assigned };
vm_block = 59; continue;
}
// C line 53817
61 => {
let _ = { let assigned = *((resolving_funcs).as_mut_ptr()).offset((is_promise_any) as isize); *((resolve_element_data).as_mut_ptr()).offset(((3 as i32)) as isize) = assigned; assigned };
vm_block = 60; continue;
}
// C line 53816
62 => {
let _ = { let assigned = values; *((resolve_element_data).as_mut_ptr()).offset(((2 as i32)) as isize) = assigned; assigned };
vm_block = 61; continue;
}
// C line 53815
63 => {
let _ = { let assigned = JS_NewInt32(ctx, index); *((resolve_element_data).as_mut_ptr()).offset(((1 as i32)) as isize) = assigned; assigned };
vm_block = 62; continue;
}
// C line 53814
64 => {
let _ = { let assigned = JS_NewBool(ctx, (0 as i32)); *((resolve_element_data).as_mut_ptr()).offset(((0 as i32)) as isize) = assigned; assigned };
vm_block = 63; continue;
}
// C line 53812
65 => {
vm_block = 17; continue;
}
// C line ? labels: fail_reject1
66 => {
let _ = JS_IteratorClose(ctx, iter, (1 as i32));
vm_block = 65; continue;
}
// C line 53809
67 => {
vm_block = if (JS_IsException(next_promise)) != 0 { 66 } else { 64 }; continue;
}
// C line 53808
68 => {
let _ = JS_FreeValue(ctx, item);
vm_block = 67; continue;
}
// C line 53806
69 => {
let _ = { let assigned = JS_Call(ctx, promise_resolve, this_val, (1 as i32), core::ptr::addr_of_mut!(item)); next_promise = assigned; assigned };
vm_block = 68; continue;
}
// C line 53805
70 => {
vm_block = 30; continue;
}
// C line 53804
71 => {
vm_block = if (done) != 0 { 70 } else { 69 }; continue;
}
// C line 53803
72 => {
vm_block = 17; continue;
}
// C line 53802
73 => {
vm_block = if (JS_IsException(item)) != 0 { 72 } else { 71 }; continue;
}
// C line 53801
74 => {
let _ = { let assigned = JS_IteratorNext(ctx, iter, next_method, (0 as i32), core::ptr::null_mut::<JSValue>(), core::ptr::addr_of_mut!(done)); item = assigned; assigned };
vm_block = 73; continue;
}
// C line 53797
75 => {
let _ = { let assigned = (0 as i32); index = assigned; assigned };
vm_block = 31; continue;
}
// C line 53795
76 => {
vm_block = 17; continue;
}
// C line 53792
77 => {
vm_block = if ((((JS_DefinePropertyValueUint32(ctx, resolve_element_env, (((0 as i32)) as u32), JS_NewInt32(ctx, (1 as i32)), ((((((1 as i32)).wrapping_shl(((0 as i32)) as u32)) | (((1 as i32)).wrapping_shl(((2 as i32)) as u32)))) | (((1 as i32)).wrapping_shl(((1 as i32)) as u32))))) < ((0 as i32))) as i32)) != 0 { 76 } else { 75 }; continue;
}
// C line 53790
78 => {
vm_block = 17; continue;
}
// C line 53789
79 => {
vm_block = if (JS_IsException(resolve_element_env)) != 0 { 78 } else { 77 }; continue;
}
// C line 53788
80 => {
let _ = { let assigned = JS_NewArray(ctx); resolve_element_env = assigned; assigned };
vm_block = 79; continue;
}
// C line 53787
81 => {
vm_block = 17; continue;
}
// C line 53786
82 => {
vm_block = if (JS_IsException(values)) != 0 { 81 } else { 80 }; continue;
}
// C line 53785
83 => {
let _ = { let assigned = JS_NewArray(ctx); values = assigned; assigned };
vm_block = 82; continue;
}
// C line 53784
84 => {
vm_block = 17; continue;
}
// C line 53783
85 => {
vm_block = if (JS_IsException(next_method)) != 0 { 84 } else { 83 }; continue;
}
// C line 53782
86 => {
let _ = { let assigned = JS_GetProperty(ctx, iter, (((crate::quickjs_atom::JS_ATOM_next as i32)) as JSAtom)); next_method = assigned; assigned };
vm_block = 85; continue;
}
// C line 53771
87 => {
vm_block = if (JS_IsException(iter)) != 0 { 17 } else { 86 }; continue;
}
// C line 53770
88 => {
let _ = { let assigned = JS_GetIterator(ctx, *(argv).offset(((0 as i32)) as isize), (0 as i32)); iter = assigned; assigned };
vm_block = 87; continue;
}
// C line 53769
89 => {
vm_block = 17; continue;
}
// C line 53767
90 => {
vm_block = if (((((JS_IsException(promise_resolve)) != 0) || ((check_function(ctx, promise_resolve)) != 0)) as i32)) != 0 { 89 } else { 88 }; continue;
}
// C line 53766
91 => {
let _ = { let assigned = JS_GetProperty(ctx, this_val, (((crate::quickjs_atom::JS_ATOM_resolve as i32)) as JSAtom)); promise_resolve = assigned; assigned };
vm_block = 90; continue;
}
// C line 53765
92 => {
return result_promise;
}
// C line 53764
93 => {
vm_block = if (JS_IsException(result_promise)) != 0 { 92 } else { 91 }; continue;
}
// C line 53763
94 => {
let _ = { let assigned = js_new_promise_capability(ctx, (resolving_funcs).as_mut_ptr(), this_val); result_promise = assigned; assigned };
vm_block = 93; continue;
}
// C line 53762
95 => {
return JS_ThrowTypeErrorNotAnObject(ctx);
}
// C line 53761
96 => {
vm_block = if ((!((JS_IsObject(this_val)) != 0) as i32)) != 0 { 95 } else { 94 }; continue;
}
// C line 53759
97 => {
is_promise_any = (((magic) == ((2 as i32))) as i32);
vm_block = 96; continue;
}
// C line 53756
98 => {
promise_resolve = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) };
iter = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) };
vm_block = 97; continue;
}
// C line 53755
99 => {
resolve_element_env = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) };
vm_block = 98; continue;
}
// C line 53754
100 => {
next_method = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) };
values = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) };
vm_block = 99; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:53895. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_promise_race(mut ctx: *mut JSContext, mut this_val: JSValue, mut argc: i32, mut argv: *mut JSValue) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut result_promise: JSValue = core::mem::zeroed();
let mut resolving_funcs: [JSValue; 2] = core::mem::zeroed();
let mut item: JSValue = core::mem::zeroed();
let mut next_promise: JSValue = core::mem::zeroed();
let mut ret: JSValue = core::mem::zeroed();
let mut next_method: JSValue = core::mem::zeroed();
let mut iter: JSValue = core::mem::zeroed();
let mut promise_resolve: JSValue = core::mem::zeroed();
let mut done: i32 = core::mem::zeroed();
let mut error: JSValue = core::mem::zeroed();
let mut vm_block: usize = 44;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 53961
1 => {
vm_block = 9; continue;
}
// C line 53960
2 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) }; result_promise = assigned; assigned };
vm_block = 1; continue;
}
// C line ? labels: fail
3 => {
let _ = JS_FreeValue(ctx, result_promise);
vm_block = 2; continue;
}
// C line 53956
4 => {
return result_promise;
}
// C line 53955
5 => {
let _ = JS_FreeValue(ctx, *((resolving_funcs).as_mut_ptr()).offset(((1 as i32)) as isize));
vm_block = 4; continue;
}
// C line 53954
6 => {
let _ = JS_FreeValue(ctx, *((resolving_funcs).as_mut_ptr()).offset(((0 as i32)) as isize));
vm_block = 5; continue;
}
// C line 53953
7 => {
let _ = JS_FreeValue(ctx, iter);
vm_block = 6; continue;
}
// C line 53952
8 => {
let _ = JS_FreeValue(ctx, next_method);
vm_block = 7; continue;
}
// C line ? labels: done
9 => {
let _ = JS_FreeValue(ctx, promise_resolve);
vm_block = 8; continue;
}
// C line 53922
10 => {
let _ = JS_FreeValue(ctx, ret);
vm_block = 9; continue;
}
// C line 53921
11 => {
vm_block = 3; continue;
}
// C line 53920
12 => {
vm_block = if (JS_IsException(ret)) != 0 { 11 } else { 10 }; continue;
}
// C line 53919
13 => {
let _ = JS_FreeValue(ctx, error);
vm_block = 12; continue;
}
// C line 53917
14 => {
let _ = { let assigned = JS_Call(ctx, *((resolving_funcs).as_mut_ptr()).offset(((1 as i32)) as isize), JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }, (1 as i32), core::ptr::addr_of_mut!(error)); ret = assigned; assigned };
vm_block = 13; continue;
}
// C line ? labels: fail_reject
15 => {
let _ = { let assigned = JS_GetException(ctx); error = assigned; assigned };
vm_block = 14; continue;
}
// C line 53928
16 => {
vm_block = 29; continue;
}
// C line 53947
17 => {
vm_block = 21; continue;
}
// C line 53946
18 => {
vm_block = if (check_exception_free(ctx, ret)) != 0 { 17 } else { 16 }; continue;
}
// C line 53944
19 => {
let _ = { let assigned = JS_InvokeFree(ctx, next_promise, (((crate::quickjs_atom::JS_ATOM_then as i32)) as JSAtom), (2 as i32), (resolving_funcs).as_mut_ptr()); ret = assigned; assigned };
vm_block = 18; continue;
}
// C line 53942
20 => {
vm_block = 15; continue;
}
// C line ? labels: fail_reject1
21 => {
let _ = JS_IteratorClose(ctx, iter, (1 as i32));
vm_block = 20; continue;
}
// C line 53939
22 => {
vm_block = if (JS_IsException(next_promise)) != 0 { 21 } else { 19 }; continue;
}
// C line 53938
23 => {
let _ = JS_FreeValue(ctx, item);
vm_block = 22; continue;
}
// C line 53936
24 => {
let _ = { let assigned = JS_Call(ctx, promise_resolve, this_val, (1 as i32), core::ptr::addr_of_mut!(item)); next_promise = assigned; assigned };
vm_block = 23; continue;
}
// C line 53935
25 => {
vm_block = 9; continue;
}
// C line 53934
26 => {
vm_block = if (done) != 0 { 25 } else { 24 }; continue;
}
// C line 53933
27 => {
vm_block = 15; continue;
}
// C line 53932
28 => {
vm_block = if (JS_IsException(item)) != 0 { 27 } else { 26 }; continue;
}
// C line 53931
29 => {
let _ = { let assigned = JS_IteratorNext(ctx, iter, next_method, (0 as i32), core::ptr::null_mut::<JSValue>(), core::ptr::addr_of_mut!(done)); item = assigned; assigned };
vm_block = 28; continue;
}
// C line 53926
30 => {
vm_block = 15; continue;
}
// C line 53925
31 => {
vm_block = if (JS_IsException(next_method)) != 0 { 30 } else { 16 }; continue;
}
// C line 53924
32 => {
let _ = { let assigned = JS_GetProperty(ctx, iter, (((crate::quickjs_atom::JS_ATOM_next as i32)) as JSAtom)); next_method = assigned; assigned };
vm_block = 31; continue;
}
// C line 53913
33 => {
vm_block = if (JS_IsException(iter)) != 0 { 15 } else { 32 }; continue;
}
// C line 53912
34 => {
let _ = { let assigned = JS_GetIterator(ctx, *(argv).offset(((0 as i32)) as isize), (0 as i32)); iter = assigned; assigned };
vm_block = 33; continue;
}
// C line 53911
35 => {
vm_block = 15; continue;
}
// C line 53909
36 => {
vm_block = if (((((JS_IsException(promise_resolve)) != 0) || ((check_function(ctx, promise_resolve)) != 0)) as i32)) != 0 { 35 } else { 34 }; continue;
}
// C line 53908
37 => {
let _ = { let assigned = JS_GetProperty(ctx, this_val, (((crate::quickjs_atom::JS_ATOM_resolve as i32)) as JSAtom)); promise_resolve = assigned; assigned };
vm_block = 36; continue;
}
// C line 53907
38 => {
return result_promise;
}
// C line 53906
39 => {
vm_block = if (JS_IsException(result_promise)) != 0 { 38 } else { 37 }; continue;
}
// C line 53905
40 => {
let _ = { let assigned = js_new_promise_capability(ctx, (resolving_funcs).as_mut_ptr(), this_val); result_promise = assigned; assigned };
vm_block = 39; continue;
}
// C line 53904
41 => {
return JS_ThrowTypeErrorNotAnObject(ctx);
}
// C line 53903
42 => {
vm_block = if ((!((JS_IsObject(this_val)) != 0) as i32)) != 0 { 41 } else { 40 }; continue;
}
// C line 53900
43 => {
promise_resolve = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) };
vm_block = 42; continue;
}
// C line 53899
44 => {
next_method = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) };
iter = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) };
vm_block = 43; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:53964. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn perform_promise_then(mut ctx: *mut JSContext, mut promise: JSValue, mut resolve_reject: *mut JSValue, mut cap_resolving_funcs: *mut JSValue) -> i32 {
let mut vm_local_storage = Vec::<u64>::new();
let mut s: *mut JSPromiseData = core::mem::zeroed();
let mut rd_array: [*mut JSPromiseReactionData; 2] = core::mem::zeroed();
let mut rd: *mut JSPromiseReactionData = core::mem::zeroed();
let mut i: i32 = core::mem::zeroed();
let mut j: i32 = core::mem::zeroed();
let mut handler: JSValue = core::mem::zeroed();
let mut args: [JSValue; 5] = core::mem::zeroed();
let mut rt: *mut JSRuntime = core::mem::zeroed();
let mut vm_block: usize = 43;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 54016
1 => {
return (0 as i32);
}
// C line 54015
2 => {
let _ = { let assigned = (1 as i32); (*(s)).is_handled = assigned; assigned };
vm_block = 1; continue;
}
// C line 53993
3 => {
vm_block = if ((((i) < ((2 as i32))) as i32)) != 0 { 5 } else { 2 }; continue;
}
// C line ?
4 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 3; continue;
}
// C line 53994
5 => {
let _ = list_add_tail(core::ptr::addr_of_mut!((*(*((rd_array).as_mut_ptr()).offset((i) as isize))).link), core::ptr::addr_of_mut!(*(((*(s)).promise_reactions).as_mut_ptr()).offset((i) as isize)));
vm_block = 4; continue;
}
// C line 53993
6 => {
let _ = { let assigned = (0 as i32); i = assigned; assigned };
vm_block = 3; continue;
}
// C line 54012
7 => {
vm_block = if ((((i) < ((2 as i32))) as i32)) != 0 { 9 } else { 2 }; continue;
}
// C line ?
8 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 7; continue;
}
// C line 54013
9 => {
let _ = promise_reaction_data_free((*(ctx)).rt, *((rd_array).as_mut_ptr()).offset((i) as isize));
vm_block = 8; continue;
}
// C line 54012
10 => {
let _ = { let assigned = (0 as i32); i = assigned; assigned };
vm_block = 7; continue;
}
// C line 54011
11 => {
let _ = JS_EnqueueJob(ctx, Some(promise_reaction_job), (5 as i32), (args).as_mut_ptr());
vm_block = 10; continue;
}
// C line 54010
12 => {
let _ = { let assigned = (*(s)).promise_result; *((args).as_mut_ptr()).offset(((4 as i32)) as isize) = assigned; assigned };
vm_block = 11; continue;
}
// C line 54009
13 => {
let _ = { let assigned = JS_NewBool(ctx, i); *((args).as_mut_ptr()).offset(((3 as i32)) as isize) = assigned; assigned };
vm_block = 12; continue;
}
// C line 54008
14 => {
let _ = { let assigned = (*(rd)).handler; *((args).as_mut_ptr()).offset(((2 as i32)) as isize) = assigned; assigned };
vm_block = 13; continue;
}
// C line 54007
15 => {
let _ = { let assigned = *(((*(rd)).resolving_funcs).as_mut_ptr()).offset(((1 as i32)) as isize); *((args).as_mut_ptr()).offset(((1 as i32)) as isize) = assigned; assigned };
vm_block = 14; continue;
}
// C line 54006
16 => {
let _ = { let assigned = *(((*(rd)).resolving_funcs).as_mut_ptr()).offset(((0 as i32)) as isize); *((args).as_mut_ptr()).offset(((0 as i32)) as isize) = assigned; assigned };
vm_block = 15; continue;
}
// C line 54005
17 => {
let _ = { let assigned = *((rd_array).as_mut_ptr()).offset((i) as isize); rd = assigned; assigned };
vm_block = 16; continue;
}
// C line 54004
18 => {
let _ = { let assigned = ((((((*(s)).promise_state) as u32)).wrapping_sub((((JS_PROMISE_FULFILLED as i32)) as u32))) as i32); i = assigned; assigned };
vm_block = 17; continue;
}
// C line 54000
19 => {
let _ = ((*(rt)).host_promise_rejection_tracker).expect("installed rejection tracker")(ctx, promise, (*(s)).promise_result, (1 as i32), (*(rt)).host_promise_rejection_tracker_opaque);
vm_block = 18; continue;
}
// C line 53999
20 => {
vm_block = if ((*(rt)).host_promise_rejection_tracker).is_some() { 19 } else { 18 }; continue;
}
// C line 53998
21 => {
rt = (*(ctx)).rt;
vm_block = 20; continue;
}
// C line 53997
22 => {
vm_block = if (((((((((((*(s)).promise_state) as u32)) == ((((JS_PROMISE_REJECTED as i32)) as u32))) as i32)) != 0) && (((!(((*(s)).is_handled) != 0) as i32)) != 0)) as i32)) != 0 { 21 } else { 18 }; continue;
}
// C line 53992
23 => {
vm_block = if (((((((*(s)).promise_state) as u32)) == ((((JS_PROMISE_PENDING as i32)) as u32))) as i32)) != 0 { 6 } else { 22 }; continue;
}
// C line 53975
24 => {
vm_block = if ((((i) < ((2 as i32))) as i32)) != 0 { 39 } else { 23 }; continue;
}
// C line ?
25 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 24; continue;
}
// C line 53989
26 => {
let _ = { let assigned = rd; *((rd_array).as_mut_ptr()).offset((i) as isize) = assigned; assigned };
vm_block = 25; continue;
}
// C line 53988
27 => {
let _ = { let assigned = JS_DupValue(ctx, handler); (*(rd)).handler = assigned; assigned };
vm_block = 26; continue;
}
// C line 53987
28 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; handler = assigned; assigned };
vm_block = 27; continue;
}
// C line 53986
29 => {
vm_block = if ((!((JS_IsFunction(ctx, handler)) != 0) as i32)) != 0 { 28 } else { 27 }; continue;
}
// C line 53985
30 => {
let _ = { let assigned = *(resolve_reject).offset((i) as isize); handler = assigned; assigned };
vm_block = 29; continue;
}
// C line 53983
31 => {
vm_block = if ((((j) < ((2 as i32))) as i32)) != 0 { 33 } else { 30 }; continue;
}
// C line ?
32 => {
let _ = { let old = j; j = (j).wrapping_add(1); old };
vm_block = 31; continue;
}
// C line 53984
33 => {
let _ = { let assigned = JS_DupValue(ctx, *(cap_resolving_funcs).offset((j) as isize)); *(((*(rd)).resolving_funcs).as_mut_ptr()).offset((j) as isize) = assigned; assigned };
vm_block = 32; continue;
}
// C line 53983
34 => {
let _ = { let assigned = (0 as i32); j = assigned; assigned };
vm_block = 31; continue;
}
// C line 53981
35 => {
return ((1 as i32)).wrapping_neg();
}
// C line 53980
36 => {
let _ = promise_reaction_data_free((*(ctx)).rt, *((rd_array).as_mut_ptr()).offset(((0 as i32)) as isize));
vm_block = 35; continue;
}
// C line 53979
37 => {
vm_block = if ((((i) == ((1 as i32))) as i32)) != 0 { 36 } else { 35 }; continue;
}
// C line 53978
38 => {
vm_block = if ((!(!(rd).is_null()) as i32)) != 0 { 37 } else { 34 }; continue;
}
// C line 53977
39 => {
let _ = { let assigned = ((js_mallocz(ctx, (size_of::<JSPromiseReactionData>() as usize))) as *mut JSPromiseReactionData); rd = assigned; assigned };
vm_block = 38; continue;
}
// C line 53975
40 => {
let _ = { let assigned = (0 as i32); i = assigned; assigned };
vm_block = 24; continue;
}
// C line 53974
41 => {
let _ = { let assigned = core::ptr::null_mut::<JSPromiseReactionData>(); *((rd_array).as_mut_ptr()).offset(((1 as i32)) as isize) = assigned; assigned };
vm_block = 40; continue;
}
// C line 53973
42 => {
let _ = { let assigned = core::ptr::null_mut::<JSPromiseReactionData>(); *((rd_array).as_mut_ptr()).offset(((0 as i32)) as isize) = assigned; assigned };
vm_block = 41; continue;
}
// C line 53969
43 => {
s = ((JS_GetOpaque(promise, (((JS_CLASS_PROMISE as i32)) as JSClassID))) as *mut JSPromiseData);
vm_block = 42; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:54019. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_promise_then(mut ctx: *mut JSContext, mut this_val: JSValue, mut argc: i32, mut argv: *mut JSValue) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut ctor: JSValue = core::mem::zeroed();
let mut result_promise: JSValue = core::mem::zeroed();
let mut resolving_funcs: [JSValue; 2] = core::mem::zeroed();
let mut s: *mut JSPromiseData = core::mem::zeroed();
let mut i: i32 = core::mem::zeroed();
let mut ret: i32 = core::mem::zeroed();
let mut vm_block: usize = 19;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 54045
1 => {
return result_promise;
}
// C line 54043
2 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 54042
3 => {
let _ = JS_FreeValue(ctx, result_promise);
vm_block = 2; continue;
}
// C line 54041
4 => {
vm_block = if (ret) != 0 { 3 } else { 1 }; continue;
}
// C line 54039
5 => {
vm_block = if ((((i) < ((2 as i32))) as i32)) != 0 { 7 } else { 4 }; continue;
}
// C line ?
6 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 5; continue;
}
// C line 54040
7 => {
let _ = JS_FreeValue(ctx, *((resolving_funcs).as_mut_ptr()).offset((i) as isize));
vm_block = 6; continue;
}
// C line 54039
8 => {
let _ = { let assigned = (0 as i32); i = assigned; assigned };
vm_block = 5; continue;
}
// C line 54037
9 => {
let _ = { let assigned = perform_promise_then(ctx, this_val, argv, (resolving_funcs).as_mut_ptr()); ret = assigned; assigned };
vm_block = 8; continue;
}
// C line 54036
10 => {
return result_promise;
}
// C line 54035
11 => {
vm_block = if (JS_IsException(result_promise)) != 0 { 10 } else { 9 }; continue;
}
// C line 54034
12 => {
let _ = JS_FreeValue(ctx, ctor);
vm_block = 11; continue;
}
// C line 54033
13 => {
let _ = { let assigned = js_new_promise_capability(ctx, (resolving_funcs).as_mut_ptr(), ctor); result_promise = assigned; assigned };
vm_block = 12; continue;
}
// C line 54032
14 => {
return ctor;
}
// C line 54031
15 => {
vm_block = if (JS_IsException(ctor)) != 0 { 14 } else { 13 }; continue;
}
// C line 54030
16 => {
let _ = { let assigned = JS_SpeciesConstructor(ctx, this_val, JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }); ctor = assigned; assigned };
vm_block = 15; continue;
}
// C line 54028
17 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 54027
18 => {
vm_block = if ((!(!(s).is_null()) as i32)) != 0 { 17 } else { 16 }; continue;
}
// C line 54026
19 => {
let _ = { let assigned = ((JS_GetOpaque2(ctx, this_val, (((JS_CLASS_PROMISE as i32)) as JSClassID))) as *mut JSPromiseData); s = assigned; assigned };
vm_block = 18; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:54048. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_promise_catch(mut ctx: *mut JSContext, mut this_val: JSValue, mut argc: i32, mut argv: *mut JSValue) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut args: [JSValue; 2] = core::mem::zeroed();
let mut vm_block: usize = 3;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 54054
1 => {
return JS_Invoke(ctx, this_val, (((crate::quickjs_atom::JS_ATOM_then as i32)) as JSAtom), (2 as i32), (args).as_mut_ptr());
}
// C line 54053
2 => {
let _ = { let assigned = *(argv).offset(((0 as i32)) as isize); *((args).as_mut_ptr()).offset(((1 as i32)) as isize) = assigned; assigned };
vm_block = 1; continue;
}
// C line 54052
3 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; *((args).as_mut_ptr()).offset(((0 as i32)) as isize) = assigned; assigned };
vm_block = 2; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:54057. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_promise_finally_value_thunk(mut ctx: *mut JSContext, mut this_val: JSValue, mut argc: i32, mut argv: *mut JSValue, mut magic: i32, mut func_data: *mut JSValue) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut vm_block: usize = 1;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 54061
1 => {
return JS_DupValue(ctx, *(func_data).offset(((0 as i32)) as isize));
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:54064. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_promise_finally_thrower(mut ctx: *mut JSContext, mut this_val: JSValue, mut argc: i32, mut argv: *mut JSValue, mut magic: i32, mut func_data: *mut JSValue) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut vm_block: usize = 1;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 54068
1 => {
return JS_Throw(ctx, JS_DupValue(ctx, *(func_data).offset(((0 as i32)) as isize)));
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:54071. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_promise_then_finally_func(mut ctx: *mut JSContext, mut this_val: JSValue, mut argc: i32, mut argv: *mut JSValue, mut magic: i32, mut func_data: *mut JSValue) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut ctor: JSValue = core::mem::zeroed();
let mut onFinally: JSValue = core::mem::zeroed();
let mut res: JSValue = core::mem::zeroed();
let mut promise: JSValue = core::mem::zeroed();
let mut ret: JSValue = core::mem::zeroed();
let mut then_func: JSValue = core::mem::zeroed();
let mut vm_block: usize = 18;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 54099
1 => {
return ret;
}
// C line 54098
2 => {
let _ = JS_FreeValue(ctx, then_func);
vm_block = 1; continue;
}
// C line 54097
3 => {
let _ = { let assigned = JS_InvokeFree(ctx, promise, (((crate::quickjs_atom::JS_ATOM_then as i32)) as JSAtom), (1 as i32), core::ptr::addr_of_mut!(then_func)); ret = assigned; assigned };
vm_block = 2; continue;
}
// C line 54095
4 => {
return then_func;
}
// C line 54094
5 => {
let _ = JS_FreeValue(ctx, promise);
vm_block = 4; continue;
}
// C line 54093
6 => {
vm_block = if (JS_IsException(then_func)) != 0 { 5 } else { 3 }; continue;
}
// C line 54087
7 => {
let _ = { let assigned = JS_NewCFunctionData(ctx, Some(js_promise_finally_value_thunk), (0 as i32), (0 as i32), (1 as i32), argv); then_func = assigned; assigned };
vm_block = 6; continue;
}
// C line 54090
8 => {
let _ = { let assigned = JS_NewCFunctionData(ctx, Some(js_promise_finally_thrower), (0 as i32), (0 as i32), (1 as i32), argv); then_func = assigned; assigned };
vm_block = 6; continue;
}
// C line 54086
9 => {
vm_block = if ((((magic) == ((0 as i32))) as i32)) != 0 { 7 } else { 8 }; continue;
}
// C line 54085
10 => {
return promise;
}
// C line 54084
11 => {
vm_block = if (JS_IsException(promise)) != 0 { 10 } else { 9 }; continue;
}
// C line 54083
12 => {
let _ = JS_FreeValue(ctx, res);
vm_block = 11; continue;
}
// C line 54082
13 => {
let _ = { let assigned = js_promise_resolve(ctx, ctor, (1 as i32), core::ptr::addr_of_mut!(res), (0 as i32)); promise = assigned; assigned };
vm_block = 12; continue;
}
// C line 54081
14 => {
return res;
}
// C line 54080
15 => {
vm_block = if (JS_IsException(res)) != 0 { 14 } else { 13 }; continue;
}
// C line 54079
16 => {
let _ = { let assigned = JS_Call(ctx, onFinally, JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }, (0 as i32), core::ptr::null_mut::<JSValue>()); res = assigned; assigned };
vm_block = 15; continue;
}
// C line 54076
17 => {
onFinally = *(func_data).offset(((1 as i32)) as isize);
vm_block = 16; continue;
}
// C line 54075
18 => {
ctor = *(func_data).offset(((0 as i32)) as isize);
vm_block = 17; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:54102. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_promise_finally(mut ctx: *mut JSContext, mut this_val: JSValue, mut argc: i32, mut argv: *mut JSValue) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut onFinally: JSValue = core::mem::zeroed();
let mut ctor: JSValue = core::mem::zeroed();
let mut ret: JSValue = core::mem::zeroed();
let mut then_funcs: [JSValue; 2] = core::mem::zeroed();
let mut func_data: [JSValue; 2] = core::mem::zeroed();
let mut i: i32 = core::mem::zeroed();
let mut vm_block: usize = 23;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 54134
1 => {
return ret;
}
// C line 54133
2 => {
let _ = JS_FreeValue(ctx, *((then_funcs).as_mut_ptr()).offset(((1 as i32)) as isize));
vm_block = 1; continue;
}
// C line 54132
3 => {
let _ = JS_FreeValue(ctx, *((then_funcs).as_mut_ptr()).offset(((0 as i32)) as isize));
vm_block = 2; continue;
}
// C line 54131
4 => {
let _ = { let assigned = JS_Invoke(ctx, this_val, (((crate::quickjs_atom::JS_ATOM_then as i32)) as JSAtom), (2 as i32), (then_funcs).as_mut_ptr()); ret = assigned; assigned };
vm_block = 3; continue;
}
// C line 54130
5 => {
let _ = JS_FreeValue(ctx, ctor);
vm_block = 4; continue;
}
// C line 54116
6 => {
let _ = { let assigned = JS_DupValue(ctx, onFinally); *((then_funcs).as_mut_ptr()).offset(((1 as i32)) as isize) = assigned; assigned };
vm_block = 5; continue;
}
// C line 54115
7 => {
let _ = { let assigned = JS_DupValue(ctx, onFinally); *((then_funcs).as_mut_ptr()).offset(((0 as i32)) as isize) = assigned; assigned };
vm_block = 6; continue;
}
// C line 54120
8 => {
vm_block = if ((((i) < ((2 as i32))) as i32)) != 0 { 15 } else { 5 }; continue;
}
// C line ?
9 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 8; continue;
}
// C line 54126
10 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 54125
11 => {
let _ = JS_FreeValue(ctx, ctor);
vm_block = 10; continue;
}
// C line 54124
12 => {
let _ = JS_FreeValue(ctx, *((then_funcs).as_mut_ptr()).offset(((0 as i32)) as isize));
vm_block = 11; continue;
}
// C line 54123
13 => {
vm_block = if ((((i) == ((1 as i32))) as i32)) != 0 { 12 } else { 11 }; continue;
}
// C line 54122
14 => {
vm_block = if (JS_IsException(*((then_funcs).as_mut_ptr()).offset((i) as isize))) != 0 { 13 } else { 9 }; continue;
}
// C line 54121
15 => {
let _ = { let assigned = JS_NewCFunctionData(ctx, Some(js_promise_then_finally_func), (1 as i32), i, (2 as i32), (func_data).as_mut_ptr()); *((then_funcs).as_mut_ptr()).offset((i) as isize) = assigned; assigned };
vm_block = 14; continue;
}
// C line 54120
16 => {
let _ = { let assigned = (0 as i32); i = assigned; assigned };
vm_block = 8; continue;
}
// C line 54119
17 => {
let _ = { let assigned = onFinally; *((func_data).as_mut_ptr()).offset(((1 as i32)) as isize) = assigned; assigned };
vm_block = 16; continue;
}
// C line 54118
18 => {
let _ = { let assigned = ctor; *((func_data).as_mut_ptr()).offset(((0 as i32)) as isize) = assigned; assigned };
vm_block = 17; continue;
}
// C line 54114
19 => {
vm_block = if ((!((JS_IsFunction(ctx, onFinally)) != 0) as i32)) != 0 { 7 } else { 18 }; continue;
}
// C line 54113
20 => {
return ctor;
}
// C line 54112
21 => {
vm_block = if (JS_IsException(ctor)) != 0 { 20 } else { 19 }; continue;
}
// C line 54111
22 => {
let _ = { let assigned = JS_SpeciesConstructor(ctx, this_val, JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }); ctor = assigned; assigned };
vm_block = 21; continue;
}
// C line 54105
23 => {
onFinally = *(argv).offset(((0 as i32)) as isize);
vm_block = 22; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:54196. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn JS_CreateAsyncFromSyncIterator(mut ctx: *mut JSContext, mut sync_iter: JSValue) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut async_iter: JSValue = core::mem::zeroed();
let mut next_method: JSValue = core::mem::zeroed();
let mut s: *mut JSAsyncFromSyncIteratorData = core::mem::zeroed();
let mut vm_block: usize = 16;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 54219
1 => {
return async_iter;
}
// C line 54218
2 => {
let _ = JS_SetOpaque(async_iter, ((s) as *mut c_void));
vm_block = 1; continue;
}
// C line 54217
3 => {
let _ = { let assigned = next_method; (*(s)).next_method = assigned; assigned };
vm_block = 2; continue;
}
// C line 54216
4 => {
let _ = { let assigned = JS_DupValue(ctx, sync_iter); (*(s)).sync_iter = assigned; assigned };
vm_block = 3; continue;
}
// C line 54214
5 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 54213
6 => {
let _ = JS_FreeValue(ctx, next_method);
vm_block = 5; continue;
}
// C line 54212
7 => {
let _ = JS_FreeValue(ctx, async_iter);
vm_block = 6; continue;
}
// C line 54211
8 => {
vm_block = if ((!(!(s).is_null()) as i32)) != 0 { 7 } else { 4 }; continue;
}
// C line 54210
9 => {
let _ = { let assigned = ((js_mallocz(ctx, (size_of::<JSAsyncFromSyncIteratorData>() as usize))) as *mut JSAsyncFromSyncIteratorData); s = assigned; assigned };
vm_block = 8; continue;
}
// C line 54208
10 => {
return async_iter;
}
// C line 54207
11 => {
let _ = JS_FreeValue(ctx, next_method);
vm_block = 10; continue;
}
// C line 54206
12 => {
vm_block = if (JS_IsException(async_iter)) != 0 { 11 } else { 9 }; continue;
}
// C line 54205
13 => {
let _ = { let assigned = JS_NewObjectClass(ctx, (JS_CLASS_ASYNC_FROM_SYNC_ITERATOR as i32)); async_iter = assigned; assigned };
vm_block = 12; continue;
}
// C line 54204
14 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 54203
15 => {
vm_block = if (JS_IsException(next_method)) != 0 { 14 } else { 13 }; continue;
}
// C line 54202
16 => {
let _ = { let assigned = JS_GetProperty(ctx, sync_iter, (((crate::quickjs_atom::JS_ATOM_next as i32)) as JSAtom)); next_method = assigned; assigned };
vm_block = 15; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:54222. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_async_from_sync_iterator_unwrap(mut ctx: *mut JSContext, mut this_val: JSValue, mut argc: i32, mut argv: *mut JSValue, mut magic: i32, mut func_data: *mut JSValue) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut vm_block: usize = 1;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 54227
1 => {
return js_create_iterator_result(ctx, JS_DupValue(ctx, *(argv).offset(((0 as i32)) as isize)), JS_ToBool(ctx, *(func_data).offset(((0 as i32)) as isize)));
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:54231. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_async_from_sync_iterator_unwrap_func_create(mut ctx: *mut JSContext, mut done: i32) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut func_data: [JSValue; 1] = core::mem::zeroed();
let mut vm_block: usize = 2;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 54237
1 => {
return JS_NewCFunctionData(ctx, Some(js_async_from_sync_iterator_unwrap), (1 as i32), (0 as i32), (1 as i32), (func_data).as_mut_ptr());
}
// C line 54236
2 => {
let _ = { let assigned = JS_NewBool(ctx, done); *((func_data).as_mut_ptr()).offset(((0 as i32)) as isize) = assigned; assigned };
vm_block = 1; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:54241. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_async_from_sync_iterator_close_wrap(mut ctx: *mut JSContext, mut this_val: JSValue, mut argc: i32, mut argv: *mut JSValue, mut magic: i32, mut func_data: *mut JSValue) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut vm_block: usize = 3;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 54248
1 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 54247
2 => {
let _ = JS_IteratorClose(ctx, *(func_data).offset(((0 as i32)) as isize), (1 as i32));
vm_block = 1; continue;
}
// C line 54246
3 => {
let _ = JS_Throw(ctx, JS_DupValue(ctx, *(argv).offset(((0 as i32)) as isize)));
vm_block = 2; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:54251. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_async_from_sync_iterator_close_wrap_func_create(mut ctx: *mut JSContext, mut sync_iter: JSValue) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut vm_block: usize = 1;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 54253
1 => {
return JS_NewCFunctionData(ctx, Some(js_async_from_sync_iterator_close_wrap), (1 as i32), (0 as i32), (1 as i32), core::ptr::addr_of_mut!(sync_iter));
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:54257. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_async_from_sync_iterator_next(mut ctx: *mut JSContext, mut this_val: JSValue, mut argc: i32, mut argv: *mut JSValue, mut magic: i32) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut promise: JSValue = core::mem::zeroed();
let mut resolving_funcs: [JSValue; 2] = core::mem::zeroed();
let mut value: JSValue = core::mem::zeroed();
let mut err: JSValue = core::mem::zeroed();
let mut method: JSValue = core::mem::zeroed();
let mut s: *mut JSAsyncFromSyncIteratorData = core::mem::zeroed();
let mut done: i32 = core::mem::zeroed();
let mut is_reject: i32 = core::mem::zeroed();
let mut obj: JSValue = core::mem::zeroed();
let mut value_wrapper_promise: JSValue = core::mem::zeroed();
let mut resolve_reject: [JSValue; 2] = core::mem::zeroed();
let mut res: i32 = core::mem::zeroed();
let mut res2: JSValue = core::mem::zeroed();
let mut vm_block: usize = 73;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 54373
1 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 54372
2 => {
let _ = JS_FreeValue(ctx, promise);
vm_block = 1; continue;
}
// C line 54371
3 => {
let _ = JS_FreeValue(ctx, *((resolving_funcs).as_mut_ptr()).offset(((1 as i32)) as isize));
vm_block = 2; continue;
}
// C line 54370
4 => {
let _ = JS_FreeValue(ctx, *((resolving_funcs).as_mut_ptr()).offset(((0 as i32)) as isize));
vm_block = 3; continue;
}
// C line ? labels: fail
5 => {
let _ = JS_FreeValue(ctx, value);
vm_block = 4; continue;
}
// C line 54367
6 => {
return promise;
}
// C line 54364
7 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 54363
8 => {
let _ = JS_FreeValue(ctx, promise);
vm_block = 7; continue;
}
// C line 54362
9 => {
vm_block = if (res) != 0 { 8 } else { 6 }; continue;
}
// C line 54361
10 => {
let _ = JS_FreeValue(ctx, *((resolving_funcs).as_mut_ptr()).offset(((1 as i32)) as isize));
vm_block = 9; continue;
}
// C line 54360
11 => {
let _ = JS_FreeValue(ctx, *((resolving_funcs).as_mut_ptr()).offset(((0 as i32)) as isize));
vm_block = 10; continue;
}
// C line 54359
12 => {
let _ = JS_FreeValue(ctx, value_wrapper_promise);
vm_block = 11; continue;
}
// C line 54358
13 => {
let _ = JS_FreeValue(ctx, *((resolve_reject).as_mut_ptr()).offset(((1 as i32)) as isize));
vm_block = 12; continue;
}
// C line 54357
14 => {
let _ = JS_FreeValue(ctx, *((resolve_reject).as_mut_ptr()).offset(((0 as i32)) as isize));
vm_block = 13; continue;
}
// C line 54354
15 => {
let _ = { let assigned = perform_promise_then(ctx, value_wrapper_promise, (resolve_reject).as_mut_ptr(), (resolving_funcs).as_mut_ptr()); res = assigned; assigned };
vm_block = 14; continue;
}
// C line 54353
16 => {
let _ = JS_FreeValue(ctx, value);
vm_block = 15; continue;
}
// C line 54343
17 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; *((resolve_reject).as_mut_ptr()).offset(((1 as i32)) as isize) = assigned; assigned };
vm_block = 16; continue;
}
// C line 54350
18 => {
vm_block = 5; continue;
}
// C line 54349
19 => {
let _ = JS_FreeValue(ctx, *((resolve_reject).as_mut_ptr()).offset(((0 as i32)) as isize));
vm_block = 18; continue;
}
// C line 54348
20 => {
let _ = JS_FreeValue(ctx, value_wrapper_promise);
vm_block = 19; continue;
}
// C line 54347
21 => {
vm_block = if (JS_IsException(*((resolve_reject).as_mut_ptr()).offset(((1 as i32)) as isize))) != 0 { 20 } else { 16 }; continue;
}
// C line 54345
22 => {
let _ = { let assigned = js_async_from_sync_iterator_close_wrap_func_create(ctx, (*(s)).sync_iter); *((resolve_reject).as_mut_ptr()).offset(((1 as i32)) as isize) = assigned; assigned };
vm_block = 21; continue;
}
// C line 54342
23 => {
vm_block = if (((((done) != 0) || (((((magic) == ((1 as i32))) as i32)) != 0)) as i32)) != 0 { 17 } else { 22 }; continue;
}
// C line 54340
24 => {
vm_block = 5; continue;
}
// C line 54339
25 => {
let _ = JS_FreeValue(ctx, value_wrapper_promise);
vm_block = 24; continue;
}
// C line 54338
26 => {
vm_block = if (JS_IsException(*((resolve_reject).as_mut_ptr()).offset(((0 as i32)) as isize))) != 0 { 25 } else { 23 }; continue;
}
// C line 54336
27 => {
let _ = { let assigned = js_async_from_sync_iterator_unwrap_func_create(ctx, done); *((resolve_reject).as_mut_ptr()).offset(((0 as i32)) as isize) = assigned; assigned };
vm_block = 26; continue;
}
// C line 54333
28 => {
return promise;
}
// C line 54332
29 => {
let _ = JS_FreeValue(ctx, *((resolving_funcs).as_mut_ptr()).offset(((1 as i32)) as isize));
vm_block = 28; continue;
}
// C line 54331
30 => {
let _ = JS_FreeValue(ctx, *((resolving_funcs).as_mut_ptr()).offset(((0 as i32)) as isize));
vm_block = 29; continue;
}
// C line 54330
31 => {
let _ = JS_FreeValue(ctx, res2);
vm_block = 30; continue;
}
// C line 54329
32 => {
let _ = JS_FreeValue(ctx, err);
vm_block = 31; continue;
}
// C line 54327 labels: done_resolve
33 => {
let _ = { let assigned = JS_Call(ctx, *((resolving_funcs).as_mut_ptr()).offset((is_reject) as isize), JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }, (1 as i32), core::ptr::addr_of_mut!(err)); res2 = assigned; assigned };
vm_block = 32; continue;
}
// C line 54325
34 => {
let _ = { let assigned = (1 as i32); is_reject = assigned; assigned };
vm_block = 33; continue;
}
// C line ? labels: reject
35 => {
let _ = { let assigned = JS_GetException(ctx); err = assigned; assigned };
vm_block = 34; continue;
}
// C line 54321
36 => {
let _ = JS_IteratorClose(ctx, (*(s)).sync_iter, (1 as i32));
vm_block = 35; continue;
}
// C line 54320
37 => {
vm_block = if ((((((((magic) != ((1 as i32))) as i32)) != 0) && (((!((done) != 0) as i32)) != 0)) as i32)) != 0 { 36 } else { 35 }; continue;
}
// C line 54319
38 => {
let _ = JS_FreeValue(ctx, value);
vm_block = 37; continue;
}
// C line 54317
39 => {
vm_block = if (JS_IsException(value_wrapper_promise)) != 0 { 38 } else { 27 }; continue;
}
// C line 54315
40 => {
let _ = { let assigned = js_promise_resolve(ctx, (*(ctx)).promise_ctor, (1 as i32), core::ptr::addr_of_mut!(value), (0 as i32)); value_wrapper_promise = assigned; assigned };
vm_block = 39; continue;
}
// C line 54310
41 => {
vm_block = 35; continue;
}
// C line 54309
42 => {
vm_block = if (JS_IsException(value)) != 0 { 41 } else { 40 }; continue;
}
// C line 54306
43 => {
vm_block = 35; continue;
}
// C line 54305
44 => {
vm_block = if (JS_IsException(value)) != 0 { 43 } else { 42 }; continue;
}
// C line 54304
45 => {
let _ = JS_FreeValue(ctx, obj);
vm_block = 44; continue;
}
// C line 54303
46 => {
let _ = { let assigned = JS_IteratorGetCompleteValue(ctx, obj, core::ptr::addr_of_mut!(done)); value = assigned; assigned };
vm_block = 45; continue;
}
// C line 54302
47 => {
obj = value;
vm_block = 46; continue;
}
// C line 54301
48 => {
vm_block = if ((((done) == ((2 as i32))) as i32)) != 0 { 47 } else { 42 }; continue;
}
// C line 54300
49 => {
vm_block = 35; continue;
}
// C line 54299
50 => {
vm_block = if (JS_IsException(value)) != 0 { 49 } else { 48 }; continue;
}
// C line 54298
51 => {
let _ = JS_FreeValue(ctx, method);
vm_block = 50; continue;
}
// C line 54296
52 => {
let _ = { let assigned = JS_IteratorNext2(ctx, (*(s)).sync_iter, method, if ((((argc) >= ((1 as i32))) as i32)) != 0 { (1 as i32) } else { (0 as i32) }, argv, core::ptr::addr_of_mut!(done)); value = assigned; assigned };
vm_block = 51; continue;
}
// C line 54276
53 => {
let _ = { let assigned = JS_DupValue(ctx, (*(s)).next_method); method = assigned; assigned };
vm_block = 52; continue;
}
// C line 54287
54 => {
vm_block = 33; continue;
}
// C line 54286
55 => {
let _ = { let assigned = (0 as i32); is_reject = assigned; assigned };
vm_block = 54; continue;
}
// C line 54285
56 => {
let _ = { let assigned = js_create_iterator_result(ctx, JS_DupValue(ctx, *(argv).offset(((0 as i32)) as isize)), (1 as i32)); err = assigned; assigned };
vm_block = 55; continue;
}
// C line 54292
57 => {
vm_block = 35; continue;
}
// C line 54291
58 => {
let _ = JS_ThrowTypeError(ctx, c"throw is not a method".as_ptr());
vm_block = 57; continue;
}
// C line 54290
59 => {
vm_block = 35; continue;
}
// C line 54289
60 => {
vm_block = if (JS_IteratorClose(ctx, (*(s)).sync_iter, (0 as i32))) != 0 { 59 } else { 58 }; continue;
}
// C line 54284
61 => {
vm_block = if ((((magic) == ((1 as i32))) as i32)) != 0 { 56 } else { 60 }; continue;
}
// C line 54283
62 => {
vm_block = if (((((JS_IsUndefined(method)) != 0) || ((JS_IsNull(method)) != 0)) as i32)) != 0 { 61 } else { 52 }; continue;
}
// C line 54282
63 => {
vm_block = 35; continue;
}
// C line 54281
64 => {
vm_block = if (JS_IsException(method)) != 0 { 63 } else { 62 }; continue;
}
// C line 54278
65 => {
let _ = { let assigned = JS_GetProperty(ctx, (*(s)).sync_iter, ((if ((((magic) == ((1 as i32))) as i32)) != 0 { (crate::quickjs_atom::JS_ATOM_return as i32) } else { (crate::quickjs_atom::JS_ATOM_throw as i32) }) as JSAtom)); method = assigned; assigned };
vm_block = 64; continue;
}
// C line 54275
66 => {
vm_block = if ((((magic) == ((0 as i32))) as i32)) != 0 { 53 } else { 65 }; continue;
}
// C line 54272
67 => {
vm_block = 35; continue;
}
// C line 54271
68 => {
let _ = JS_ThrowTypeError(ctx, c"not an Async-from-Sync Iterator".as_ptr());
vm_block = 67; continue;
}
// C line 54270
69 => {
vm_block = if ((!(!(s).is_null()) as i32)) != 0 { 68 } else { 66 }; continue;
}
// C line 54269
70 => {
let _ = { let assigned = ((JS_GetOpaque(this_val, (((JS_CLASS_ASYNC_FROM_SYNC_ITERATOR as i32)) as JSClassID))) as *mut JSAsyncFromSyncIteratorData); s = assigned; assigned };
vm_block = 69; continue;
}
// C line 54268
71 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 54267
72 => {
vm_block = if (JS_IsException(promise)) != 0 { 71 } else { 70 }; continue;
}
// C line 54266
73 => {
let _ = { let assigned = JS_NewPromiseCapability(ctx, (resolving_funcs).as_mut_ptr()); promise = assigned; assigned };
vm_block = 72; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:54409. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
pub unsafe fn JS_AddIntrinsicPromise(mut ctx: *mut JSContext) -> i32 {
let mut vm_local_storage = Vec::<u64>::new();
let mut rt: *mut JSRuntime = core::mem::zeroed();
let mut obj1: JSValue = core::mem::zeroed();
let mut ft: JSCFunctionType = core::mem::zeroed();
let mut vm_block: usize = 34;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 54486
1 => {
return JS_SetConstructor2(ctx, *((*(ctx)).class_proto).offset(((JS_CLASS_ASYNC_GENERATOR_FUNCTION as i32)) as isize), *((*(ctx)).class_proto).offset(((JS_CLASS_ASYNC_GENERATOR as i32)) as isize), ((1 as i32)).wrapping_shl(((0 as i32)) as u32), ((1 as i32)).wrapping_shl(((0 as i32)) as u32));
}
// C line 54484
2 => {
let _ = JS_FreeValue(ctx, obj1);
vm_block = 1; continue;
}
// C line 54483
3 => {
return ((1 as i32)).wrapping_neg();
}
// C line 54482
4 => {
vm_block = if (JS_IsException(obj1)) != 0 { 3 } else { 2 }; continue;
}
// C line 54476
5 => {
let _ = { let assigned = JS_NewCConstructor(ctx, (JS_CLASS_ASYNC_GENERATOR_FUNCTION as i32), c"AsyncGeneratorFunction".as_ptr(), (ft).generic, (1 as i32), (((JS_CFUNC_constructor_or_func_magic as i32)) as JSCFunctionEnum), (JS_FUNC_ASYNC_GENERATOR as i32), (*(ctx)).function_ctor, core::ptr::null_mut::<JSCFunctionListEntry>(), (0 as i32), ptr::addr_of!(js_async_generator_function_proto_funcs).cast::<JSCFunctionListEntry>(), (((((size_of::<[JSCFunctionListEntry; 1]>() as usize)) / ((size_of::<JSCFunctionListEntry>() as usize)))) as i32), ((((1 as i32)).wrapping_shl(((0 as i32)) as u32)) | (((1 as i32)).wrapping_shl(((3 as i32)) as u32)))); obj1 = assigned; assigned };
vm_block = 4; continue;
}
// C line 54475
6 => {
let _ = { let assigned = js_function_constructor; (ft).generic_magic = Some(assigned); assigned };
vm_block = 5; continue;
}
// C line 54472
7 => {
return ((1 as i32)).wrapping_neg();
}
// C line 54471
8 => {
vm_block = if (JS_IsException(*((*(ctx)).class_proto).offset(((JS_CLASS_ASYNC_GENERATOR as i32)) as isize))) != 0 { 7 } else { 6 }; continue;
}
// C line 54467
9 => {
let _ = { let assigned = JS_NewObjectProtoList(ctx, (*(ctx)).async_iterator_proto, ptr::addr_of!(js_async_generator_proto_funcs).cast::<JSCFunctionListEntry>(), (((((size_of::<[JSCFunctionListEntry; 4]>() as usize)) / ((size_of::<JSCFunctionListEntry>() as usize)))) as i32)); *((*(ctx)).class_proto).offset(((JS_CLASS_ASYNC_GENERATOR as i32)) as isize) = assigned; assigned };
vm_block = 8; continue;
}
// C line 54464
10 => {
return ((1 as i32)).wrapping_neg();
}
// C line 54463
11 => {
vm_block = if (JS_IsException(*((*(ctx)).class_proto).offset(((JS_CLASS_ASYNC_FROM_SYNC_ITERATOR as i32)) as isize))) != 0 { 10 } else { 9 }; continue;
}
// C line 54459
12 => {
let _ = { let assigned = JS_NewObjectProtoList(ctx, (*(ctx)).async_iterator_proto, ptr::addr_of!(js_async_from_sync_iterator_proto_funcs).cast::<JSCFunctionListEntry>(), (((((size_of::<[JSCFunctionListEntry; 3]>() as usize)) / ((size_of::<JSCFunctionListEntry>() as usize)))) as i32)); *((*(ctx)).class_proto).offset(((JS_CLASS_ASYNC_FROM_SYNC_ITERATOR as i32)) as isize) = assigned; assigned };
vm_block = 11; continue;
}
// C line 54456
13 => {
return ((1 as i32)).wrapping_neg();
}
// C line 54455
14 => {
vm_block = if (JS_IsException((*(ctx)).async_iterator_proto)) != 0 { 13 } else { 12 }; continue;
}
// C line 54451
15 => {
let _ = { let assigned = JS_NewObjectProtoList(ctx, *((*(ctx)).class_proto).offset(((JS_CLASS_OBJECT as i32)) as isize), ptr::addr_of!(js_async_iterator_proto_funcs).cast::<JSCFunctionListEntry>(), (((((size_of::<[JSCFunctionListEntry; 1]>() as usize)) / ((size_of::<JSCFunctionListEntry>() as usize)))) as i32)); (*(ctx)).async_iterator_proto = assigned; assigned };
vm_block = 14; continue;
}
// C line 54448
16 => {
let _ = JS_FreeValue(ctx, obj1);
vm_block = 15; continue;
}
// C line 54447
17 => {
return ((1 as i32)).wrapping_neg();
}
// C line 54446
18 => {
vm_block = if (JS_IsException(obj1)) != 0 { 17 } else { 16 }; continue;
}
// C line 54440
19 => {
let _ = { let assigned = JS_NewCConstructor(ctx, (JS_CLASS_ASYNC_FUNCTION as i32), c"AsyncFunction".as_ptr(), (ft).generic, (1 as i32), (((JS_CFUNC_constructor_or_func_magic as i32)) as JSCFunctionEnum), (JS_FUNC_ASYNC as i32), (*(ctx)).function_ctor, core::ptr::null_mut::<JSCFunctionListEntry>(), (0 as i32), ptr::addr_of!(js_async_function_proto_funcs).cast::<JSCFunctionListEntry>(), (((((size_of::<[JSCFunctionListEntry; 1]>() as usize)) / ((size_of::<JSCFunctionListEntry>() as usize)))) as i32), ((((1 as i32)).wrapping_shl(((0 as i32)) as u32)) | (((1 as i32)).wrapping_shl(((3 as i32)) as u32)))); obj1 = assigned; assigned };
vm_block = 18; continue;
}
// C line 54439
20 => {
let _ = { let assigned = js_function_constructor; (ft).generic_magic = Some(assigned); assigned };
vm_block = 19; continue;
}
// C line 54436
21 => {
let _ = { let assigned = obj1; (*(ctx)).promise_ctor = assigned; assigned };
vm_block = 20; continue;
}
// C line 54435
22 => {
return ((1 as i32)).wrapping_neg();
}
// C line 54434
23 => {
vm_block = if (JS_IsException(obj1)) != 0 { 22 } else { 21 }; continue;
}
// C line 54428
24 => {
let _ = { let assigned = JS_NewCConstructor(ctx, (JS_CLASS_PROMISE as i32), c"Promise".as_ptr(), Some(js_promise_constructor), (1 as i32), (((JS_CFUNC_constructor as i32)) as JSCFunctionEnum), (0 as i32), JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }, ptr::addr_of!(js_promise_funcs).cast::<JSCFunctionListEntry>(), (((((size_of::<[JSCFunctionListEntry; 9]>() as usize)) / ((size_of::<JSCFunctionListEntry>() as usize)))) as i32), ptr::addr_of!(js_promise_proto_funcs).cast::<JSCFunctionListEntry>(), (((((size_of::<[JSCFunctionListEntry; 4]>() as usize)) / ((size_of::<JSCFunctionListEntry>() as usize)))) as i32), (0 as i32)); obj1 = assigned; assigned };
vm_block = 23; continue;
}
// C line 54424
25 => {
let _ = { let assigned = js_async_generator_function_call; (*((*(rt)).class_array).offset(((JS_CLASS_ASYNC_GENERATOR_FUNCTION as i32)) as isize)).call = Some(assigned); assigned };
vm_block = 24; continue;
}
// C line 54423
26 => {
let _ = { let assigned = js_async_function_resolve_call; (*((*(rt)).class_array).offset(((JS_CLASS_ASYNC_FUNCTION_REJECT as i32)) as isize)).call = Some(assigned); assigned };
vm_block = 25; continue;
}
// C line 54422
27 => {
let _ = { let assigned = js_async_function_resolve_call; (*((*(rt)).class_array).offset(((JS_CLASS_ASYNC_FUNCTION_RESOLVE as i32)) as isize)).call = Some(assigned); assigned };
vm_block = 26; continue;
}
// C line 54421
28 => {
let _ = { let assigned = js_async_function_call; (*((*(rt)).class_array).offset(((JS_CLASS_ASYNC_FUNCTION as i32)) as isize)).call = Some(assigned); assigned };
vm_block = 27; continue;
}
// C line 54420
29 => {
let _ = { let assigned = js_promise_resolve_function_call; (*((*(rt)).class_array).offset(((JS_CLASS_PROMISE_REJECT_FUNCTION as i32)) as isize)).call = Some(assigned); assigned };
vm_block = 28; continue;
}
// C line 54419
30 => {
let _ = { let assigned = js_promise_resolve_function_call; (*((*(rt)).class_array).offset(((JS_CLASS_PROMISE_RESOLVE_FUNCTION as i32)) as isize)).call = Some(assigned); assigned };
vm_block = 29; continue;
}
// C line 54418
31 => {
return ((1 as i32)).wrapping_neg();
}
// C line 54416
32 => {
vm_block = if (init_class_range(rt, ptr::addr_of!(js_async_class_def).cast::<JSClassShortDef>(), (JS_CLASS_PROMISE as i32), (((((size_of::<[JSClassShortDef; 9]>() as usize)) / ((size_of::<JSClassShortDef>() as usize)))) as i32))) != 0 { 31 } else { 30 }; continue;
}
// C line 54415
33 => {
vm_block = if ((!((JS_IsRegisteredClass(rt, (((JS_CLASS_PROMISE as i32)) as JSClassID))) != 0) as i32)) != 0 { 32 } else { 24 }; continue;
}
// C line 54411
34 => {
rt = (*(ctx)).rt;
vm_block = 33; continue;
}
_ => std::process::abort(),
} }
}
