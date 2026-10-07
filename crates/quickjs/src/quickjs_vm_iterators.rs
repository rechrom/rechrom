// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:16168. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_build_arguments(mut ctx: *mut JSContext, mut argc: i32, mut argv: *mut JSValue) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut val: JSValue = core::mem::zeroed();
let mut tab: *mut JSValue = core::mem::zeroed();
let mut props: [JSProperty; 3] = core::mem::zeroed();
let mut p: *mut JSObject = core::mem::zeroed();
let mut i: i32 = core::mem::zeroed();
let mut vm_block: usize = 22;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 16201
1 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line ? labels: fail
2 => {
let _ = JS_FreeValue(ctx, val);
vm_block = 1; continue;
}
// C line 16198
3 => {
return val;
}
// C line 16197
4 => {
let _ = { let assigned = ((argc) as u32); (((*(p)).u).array).count = assigned; assigned };
vm_block = 3; continue;
}
// C line 16196
5 => {
let _ = { let assigned = tab; ((((*(p)).u).array).u).values = assigned; assigned };
vm_block = 4; continue;
}
// C line 16192
6 => {
vm_block = if ((((i) < (argc)) as i32)) != 0 { 8 } else { 5 }; continue;
}
// C line ?
7 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 6; continue;
}
// C line 16193
8 => {
let _ = { let assigned = JS_DupValue(ctx, *(argv).offset((i) as isize)); *(tab).offset((i) as isize) = assigned; assigned };
vm_block = 7; continue;
}
// C line 16192
9 => {
let _ = { let assigned = (0 as i32); i = assigned; assigned };
vm_block = 6; continue;
}
// C line 16191
10 => {
vm_block = 2; continue;
}
// C line 16190
11 => {
vm_block = if ((!(!(tab).is_null()) as i32)) != 0 { 10 } else { 9 }; continue;
}
// C line 16189
12 => {
let _ = { let assigned = ((js_malloc(ctx, ((size_of::<JSValue>() as usize)).wrapping_mul(((argc) as usize)))) as *mut JSValue); tab = assigned; assigned };
vm_block = 11; continue;
}
// C line 16188
13 => {
vm_block = if ((((argc) > ((0 as i32))) as i32)) != 0 { 12 } else { 5 }; continue;
}
// C line 16187
14 => {
let _ = { let assigned = core::ptr::null_mut::<JSValue>(); tab = assigned; assigned };
vm_block = 13; continue;
}
// C line 16184
15 => {
let _ = { let assigned = ((((val).u).ptr) as *mut JSObject); p = assigned; assigned };
vm_block = 14; continue;
}
// C line 16183
16 => {
return val;
}
// C line 16182
17 => {
vm_block = if (JS_IsException(val)) != 0 { 16 } else { 15 }; continue;
}
// C line 16180
18 => {
let _ = { let assigned = JS_NewObjectFromShape(ctx, js_dup_shape((*(ctx)).arguments_shape), (((JS_CLASS_ARGUMENTS as i32)) as JSClassID), (props).as_mut_ptr()); val = assigned; assigned };
vm_block = 17; continue;
}
// C line 16178
19 => {
let _ = { let assigned = ((((JS_DupValue(ctx, (*(ctx)).throw_type_error)).u).ptr) as *mut JSObject); (((*((props).as_mut_ptr()).offset(((2 as i32)) as isize)).u).getset).setter = assigned; assigned };
vm_block = 18; continue;
}
// C line 16177
20 => {
let _ = { let assigned = ((((JS_DupValue(ctx, (*(ctx)).throw_type_error)).u).ptr) as *mut JSObject); (((*((props).as_mut_ptr()).offset(((2 as i32)) as isize)).u).getset).getter = assigned; assigned };
vm_block = 19; continue;
}
// C line 16176
21 => {
let _ = { let assigned = JS_DupValue(ctx, (*(ctx)).array_proto_values); ((*((props).as_mut_ptr()).offset(((1 as i32)) as isize)).u).value = assigned; assigned };
vm_block = 20; continue;
}
// C line 16175
22 => {
let _ = { let assigned = JS_NewInt32(ctx, argc); ((*((props).as_mut_ptr()).offset(((0 as i32)) as isize)).u).value = assigned; assigned };
vm_block = 21; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:16229. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_build_mapped_arguments(mut ctx: *mut JSContext, mut argc: i32, mut argv: *mut JSValue, mut sf: *mut JSStackFrame, mut arg_count: i32) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut val: JSValue = core::mem::zeroed();
let mut props: [JSProperty; 3] = core::mem::zeroed();
let mut tab: *mut *mut JSVarRef = core::mem::zeroed();
let mut var_ref: *mut JSVarRef = core::mem::zeroed();
let mut p: *mut JSObject = core::mem::zeroed();
let mut i: i32 = core::mem::zeroed();
let mut j: i32 = core::mem::zeroed();
let mut vm_block: usize = 37;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 16279
1 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line ? labels: fail
2 => {
let _ = JS_FreeValue(ctx, val);
vm_block = 1; continue;
}
// C line 16276
3 => {
return val;
}
// C line 16275
4 => {
let _ = { let assigned = ((argc) as u32); (((*(p)).u).array).count = assigned; assigned };
vm_block = 3; continue;
}
// C line 16274
5 => {
let _ = { let assigned = tab; ((((*(p)).u).array).u).var_refs = assigned; assigned };
vm_block = 4; continue;
}
// C line 16261
6 => {
vm_block = if ((((i) < (argc)) as i32)) != 0 { 17 } else { 5 }; continue;
}
// C line ?
7 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 6; continue;
}
// C line 16271
8 => {
let _ = { let assigned = var_ref; *(tab).offset((i) as isize) = assigned; assigned };
vm_block = 7; continue;
}
// C line 16270
9 => {
let _ = { let assigned = JS_DupValue(ctx, *(argv).offset((i) as isize)); ((*(var_ref)).u).value = assigned; assigned };
vm_block = 8; continue;
}
// C line 16268
10 => {
vm_block = 2; continue;
}
// C line 16267
11 => {
let _ = js_free(ctx, ((tab) as *mut c_void));
vm_block = 10; continue;
}
// C line 16265
12 => {
vm_block = if ((((j) < (i)) as i32)) != 0 { 14 } else { 11 }; continue;
}
// C line ?
13 => {
let _ = { let old = j; j = (j).wrapping_add(1); old };
vm_block = 12; continue;
}
// C line 16266
14 => {
let _ = free_var_ref((*(ctx)).rt, *(tab).offset((j) as isize));
vm_block = 13; continue;
}
// C line 16265 labels: fail1
15 => {
let _ = { let assigned = (0 as i32); j = assigned; assigned };
vm_block = 12; continue;
}
// C line 16263
16 => {
vm_block = if ((!(!(var_ref).is_null()) as i32)) != 0 { 15 } else { 9 }; continue;
}
// C line 16262
17 => {
let _ = { let assigned = js_create_var_ref(ctx, (0 as i32)); var_ref = assigned; assigned };
vm_block = 16; continue;
}
// C line 16261
18 => {
let _ = { let assigned = arg_count; i = assigned; assigned };
vm_block = 6; continue;
}
// C line 16255
19 => {
vm_block = if ((((i) < (arg_count)) as i32)) != 0 { 24 } else { 18 }; continue;
}
// C line ?
20 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 19; continue;
}
// C line 16259
21 => {
let _ = { let assigned = var_ref; *(tab).offset((i) as isize) = assigned; assigned };
vm_block = 20; continue;
}
// C line 16258
22 => {
vm_block = 15; continue;
}
// C line 16257
23 => {
vm_block = if ((!(!(var_ref).is_null()) as i32)) != 0 { 22 } else { 21 }; continue;
}
// C line 16256
24 => {
let _ = { let assigned = get_var_ref(ctx, sf, i, (1 as i32)); var_ref = assigned; assigned };
vm_block = 23; continue;
}
// C line 16255
25 => {
let _ = { let assigned = (0 as i32); i = assigned; assigned };
vm_block = 19; continue;
}
// C line 16254
26 => {
vm_block = 2; continue;
}
// C line 16253
27 => {
vm_block = if ((!(!(tab).is_null()) as i32)) != 0 { 26 } else { 25 }; continue;
}
// C line 16252
28 => {
let _ = { let assigned = ((js_malloc(ctx, ((size_of::<*mut JSVarRef>() as usize)).wrapping_mul(((argc) as usize)))) as *mut *mut JSVarRef); tab = assigned; assigned };
vm_block = 27; continue;
}
// C line 16251
29 => {
vm_block = if ((((argc) > ((0 as i32))) as i32)) != 0 { 28 } else { 5 }; continue;
}
// C line 16250
30 => {
let _ = { let assigned = core::ptr::null_mut::<*mut JSVarRef>(); tab = assigned; assigned };
vm_block = 29; continue;
}
// C line 16247
31 => {
let _ = { let assigned = ((((val).u).ptr) as *mut JSObject); p = assigned; assigned };
vm_block = 30; continue;
}
// C line 16246
32 => {
return val;
}
// C line 16245
33 => {
vm_block = if (JS_IsException(val)) != 0 { 32 } else { 31 }; continue;
}
// C line 16243
34 => {
let _ = { let assigned = JS_NewObjectFromShape(ctx, js_dup_shape((*(ctx)).mapped_arguments_shape), (((JS_CLASS_MAPPED_ARGUMENTS as i32)) as JSClassID), (props).as_mut_ptr()); val = assigned; assigned };
vm_block = 33; continue;
}
// C line 16241
35 => {
let _ = { let assigned = JS_DupValue(ctx, (*((*((*(ctx)).rt)).current_stack_frame)).cur_func); ((*((props).as_mut_ptr()).offset(((2 as i32)) as isize)).u).value = assigned; assigned };
vm_block = 34; continue;
}
// C line 16240
36 => {
let _ = { let assigned = JS_DupValue(ctx, (*(ctx)).array_proto_values); ((*((props).as_mut_ptr()).offset(((1 as i32)) as isize)).u).value = assigned; assigned };
vm_block = 35; continue;
}
// C line 16239
37 => {
let _ = { let assigned = JS_NewInt32(ctx, argc); ((*((props).as_mut_ptr()).offset(((0 as i32)) as isize)).u).value = assigned; assigned };
vm_block = 36; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:16282. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn build_for_in_iterator(mut ctx: *mut JSContext, mut obj: JSValue) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut p: *mut JSObject = core::mem::zeroed();
let mut p1: *mut JSObject = core::mem::zeroed();
let mut tab_atom: *mut JSPropertyEnum = core::mem::zeroed();
let mut i: i32 = core::mem::zeroed();
let mut enum_obj: JSValue = core::mem::zeroed();
let mut it: *mut JSForInIterator = core::mem::zeroed();
let mut tag: u32 = core::mem::zeroed();
let mut tab_atom_count: u32 = core::mem::zeroed();
let mut sh: *mut JSShape = core::mem::zeroed();
let mut prs: *mut JSShapeProperty = core::mem::zeroed();
let mut vm_block: usize = 38;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 16342
1 => {
return enum_obj;
}
// C line 16331
2 => {
let _ = { let assigned = (((*(p)).u).array).count; (*(it)).atom_count = assigned; assigned };
vm_block = 1; continue;
}
// C line 16330
3 => {
let _ = { let assigned = (((1 as i32)) as u8); (*(it)).is_array = assigned; assigned };
vm_block = 2; continue;
}
// C line 16325
4 => {
vm_block = if ((((i) < ((*(sh)).prop_count)) as i32)) != 0 { 7 } else { 3 }; continue;
}
// C line ?
5 => {
let _ = { let _ = { let old = i; i = (i).wrapping_add(1); old }; { let old = prs; prs = (prs).offset(1); old } };
vm_block = 4; continue;
}
// C line 16327
6 => {
vm_block = 14; continue;
}
// C line 16326
7 => {
vm_block = if ((((((*(prs)).flags()) as i32)) & (((1 as i32)).wrapping_shl(((2 as i32)) as u32)))) != 0 { 6 } else { 5 }; continue;
}
// C line 16325
8 => {
let _ = { let _ = { let assigned = (0 as i32); i = assigned; assigned }; { let assigned = get_shape_prop(sh); prs = assigned; assigned } };
vm_block = 4; continue;
}
// C line 16324
9 => {
let _ = { let assigned = (*(p)).shape; sh = assigned; assigned };
vm_block = 8; continue;
}
// C line 16340
10 => {
let _ = { let assigned = tab_atom_count; (*(it)).atom_count = assigned; assigned };
vm_block = 1; continue;
}
// C line 16339
11 => {
let _ = { let assigned = tab_atom; (*(it)).tab_atom = assigned; assigned };
vm_block = 10; continue;
}
// C line 16337
12 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 16336
13 => {
let _ = JS_FreeValue(ctx, enum_obj);
vm_block = 12; continue;
}
// C line 16334 labels: normal_case
14 => {
vm_block = if (JS_GetOwnPropertyNamesInternal(ctx, core::ptr::addr_of_mut!(tab_atom), core::ptr::addr_of_mut!(tab_atom_count), p, ((((1 as i32)).wrapping_shl(((0 as i32)) as u32)) | (((1 as i32)).wrapping_shl(((5 as i32)) as u32))))) != 0 { 13 } else { 11 }; continue;
}
// C line 16320
15 => {
vm_block = if ((*(p)).fast_array()) != 0 { 9 } else { 14 }; continue;
}
// C line 16319
16 => {
let _ = { let assigned = ((((obj).u).ptr) as *mut JSObject); p = assigned; assigned };
vm_block = 15; continue;
}
// C line 16317
17 => {
return enum_obj;
}
// C line 16316
18 => {
vm_block = if ((((((((tag) == ((((JS_TAG_NULL as i32)) as u32))) as i32)) != 0) || (((((tag) == ((((JS_TAG_UNDEFINED as i32)) as u32))) as i32)) != 0)) as i32)) != 0 { 17 } else { 16 }; continue;
}
// C line 16314
19 => {
let _ = { let assigned = it; ((*(p1)).u).for_in_iterator = assigned; assigned };
vm_block = 18; continue;
}
// C line 16313
20 => {
let _ = { let assigned = ((((enum_obj).u).ptr) as *mut JSObject); p1 = assigned; assigned };
vm_block = 19; continue;
}
// C line 16312
21 => {
let _ = { let assigned = (((0 as i32)) as u8); (*(it)).in_prototype_chain = assigned; assigned };
vm_block = 20; continue;
}
// C line 16311
22 => {
let _ = { let assigned = (((0 as i32)) as u32); (*(it)).atom_count = assigned; assigned };
vm_block = 21; continue;
}
// C line 16310
23 => {
let _ = { let assigned = core::ptr::null_mut::<JSPropertyEnum>(); (*(it)).tab_atom = assigned; assigned };
vm_block = 22; continue;
}
// C line 16309
24 => {
let _ = { let assigned = (((0 as i32)) as u32); (*(it)).idx = assigned; assigned };
vm_block = 23; continue;
}
// C line 16308
25 => {
let _ = { let assigned = obj; (*(it)).obj = assigned; assigned };
vm_block = 24; continue;
}
// C line 16307
26 => {
let _ = { let assigned = (((0 as i32)) as u8); (*(it)).is_array = assigned; assigned };
vm_block = 25; continue;
}
// C line 16305
27 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 16304
28 => {
let _ = JS_FreeValue(ctx, obj);
vm_block = 27; continue;
}
// C line 16303
29 => {
let _ = js_free(ctx, ((it) as *mut c_void));
vm_block = 28; continue;
}
// C line 16302
30 => {
vm_block = if (JS_IsException(enum_obj)) != 0 { 29 } else { 26 }; continue;
}
// C line 16301
31 => {
let _ = { let assigned = JS_NewObjectProtoClass(ctx, JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_NULL as i32)) as i64) }, (((JS_CLASS_FOR_IN_ITERATOR as i32)) as JSClassID)); enum_obj = assigned; assigned };
vm_block = 30; continue;
}
// C line 16299
32 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 16298
33 => {
let _ = JS_FreeValue(ctx, obj);
vm_block = 32; continue;
}
// C line 16297
34 => {
vm_block = if ((!(!(it).is_null()) as i32)) != 0 { 33 } else { 31 }; continue;
}
// C line 16296
35 => {
let _ = { let assigned = ((js_malloc(ctx, (size_of::<JSForInIterator>() as usize))) as *mut JSForInIterator); it = assigned; assigned };
vm_block = 34; continue;
}
// C line 16293
36 => {
let _ = { let assigned = JS_ToObjectFree(ctx, obj); obj = assigned; assigned };
vm_block = 35; continue;
}
// C line 16292
37 => {
vm_block = if ((((((((((((tag) != ((((JS_TAG_OBJECT as i32)) as u32))) as i32)) != 0) && (((((tag) != ((((JS_TAG_NULL as i32)) as u32))) as i32)) != 0)) as i32)) != 0) && (((((tag) != ((((JS_TAG_UNDEFINED as i32)) as u32))) as i32)) != 0)) as i32)) != 0 { 36 } else { 35 }; continue;
}
// C line 16291
38 => {
let _ = { let assigned = (((((obj).tag) as i32)) as u32); tag = assigned; assigned };
vm_block = 37; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:16346. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_for_in_start(mut ctx: *mut JSContext, mut sp: *mut JSValue) -> i32 {
let mut vm_local_storage = Vec::<u64>::new();
let mut vm_block: usize = 4;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 16351
1 => {
return (0 as i32);
}
// C line 16350
2 => {
return ((1 as i32)).wrapping_neg();
}
// C line 16349
3 => {
vm_block = if (JS_IsException(*(sp).offset((((1 as i32)).wrapping_neg()) as isize))) != 0 { 2 } else { 1 }; continue;
}
// C line 16348
4 => {
let _ = { let assigned = build_for_in_iterator(ctx, *(sp).offset((((1 as i32)).wrapping_neg()) as isize)); *(sp).offset((((1 as i32)).wrapping_neg()) as isize) = assigned; assigned };
vm_block = 3; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:16355. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_for_in_prepare_prototype_chain_enum(mut ctx: *mut JSContext, mut enum_obj: JSValue) -> i32 {
let mut vm_local_storage = Vec::<u64>::new();
let mut p: *mut JSObject = core::mem::zeroed();
let mut it: *mut JSForInIterator = core::mem::zeroed();
let mut tab_atom: *mut JSPropertyEnum = core::mem::zeroed();
let mut tab_atom_count: u32 = core::mem::zeroed();
let mut i: u32 = core::mem::zeroed();
let mut obj1: JSValue = core::mem::zeroed();
let mut vm_block: usize = 34;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line ? labels: fail
1 => {
return ((1 as i32)).wrapping_neg();
}
// C line 16412
2 => {
return (0 as i32);
}
// C line 16408
3 => {
vm_block = if ((((i) < ((*(it)).atom_count)) as i32)) != 0 { 6 } else { 2 }; continue;
}
// C line ?
4 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 3; continue;
}
// C line 16410
5 => {
vm_block = 1; continue;
}
// C line 16409
6 => {
vm_block = if ((((JS_DefinePropertyValue(ctx, enum_obj, (*((*(it)).tab_atom).offset((i) as isize)).atom, JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_NULL as i32)) as i64) }, ((1 as i32)).wrapping_shl(((2 as i32)) as u32))) < ((0 as i32))) as i32)) != 0 { 5 } else { 4 }; continue;
}
// C line 16408
7 => {
let _ = { let assigned = (((0 as i32)) as u32); i = assigned; assigned };
vm_block = 3; continue;
}
// C line 16405
8 => {
let _ = { let assigned = tab_atom_count; (*(it)).atom_count = assigned; assigned };
vm_block = 7; continue;
}
// C line 16404
9 => {
let _ = { let assigned = tab_atom; (*(it)).tab_atom = assigned; assigned };
vm_block = 8; continue;
}
// C line 16403
10 => {
let _ = { let assigned = (((0 as i32)) as u8); (*(it)).is_array = assigned; assigned };
vm_block = 9; continue;
}
// C line 16401
11 => {
vm_block = 1; continue;
}
// C line 16398
12 => {
vm_block = if (JS_GetOwnPropertyNamesInternal(ctx, core::ptr::addr_of_mut!(tab_atom), core::ptr::addr_of_mut!(tab_atom_count), (((((*(it)).obj).u).ptr) as *mut JSObject), ((((1 as i32)).wrapping_shl(((0 as i32)) as u32)) | (((1 as i32)).wrapping_shl(((5 as i32)) as u32))))) != 0 { 11 } else { 10 }; continue;
}
// C line 16397 labels: slow_path
13 => {
vm_block = if ((*(it)).is_array) != 0 { 12 } else { 7 }; continue;
}
// C line 16393
14 => {
return (1 as i32);
}
// C line 16392
15 => {
let _ = JS_FreeValue(ctx, obj1);
vm_block = 14; continue;
}
// C line 16369
16 => {
vm_block = 31; continue;
}
// C line 16389
17 => {
vm_block = 1; continue;
}
// C line 16388
18 => {
let _ = JS_FreeValue(ctx, obj1);
vm_block = 17; continue;
}
// C line 16387
19 => {
vm_block = if (js_poll_interrupts(ctx)) != 0 { 18 } else { 16 }; continue;
}
// C line 16384
20 => {
vm_block = 13; continue;
}
// C line 16383
21 => {
let _ = JS_FreeValue(ctx, obj1);
vm_block = 20; continue;
}
// C line 16382
22 => {
vm_block = if ((((tab_atom_count) != ((((0 as i32)) as u32))) as i32)) != 0 { 21 } else { 19 }; continue;
}
// C line 16381
23 => {
let _ = JS_FreePropertyEnum(ctx, tab_atom, tab_atom_count);
vm_block = 22; continue;
}
// C line 16379
24 => {
vm_block = 1; continue;
}
// C line 16378
25 => {
let _ = JS_FreeValue(ctx, obj1);
vm_block = 24; continue;
}
// C line 16375
26 => {
vm_block = if (JS_GetOwnPropertyNamesInternal(ctx, core::ptr::addr_of_mut!(tab_atom), core::ptr::addr_of_mut!(tab_atom_count), ((((obj1).u).ptr) as *mut JSObject), ((((1 as i32)).wrapping_shl(((0 as i32)) as u32)) | (((1 as i32)).wrapping_shl(((4 as i32)) as u32))))) != 0 { 25 } else { 23 }; continue;
}
// C line 16374
27 => {
vm_block = 1; continue;
}
// C line 16373
28 => {
vm_block = if (JS_IsException(obj1)) != 0 { 27 } else { 26 }; continue;
}
// C line 16372
29 => {
vm_block = 15; continue;
}
// C line 16371
30 => {
vm_block = if (JS_IsNull(obj1)) != 0 { 29 } else { 28 }; continue;
}
// C line 16370
31 => {
let _ = { let assigned = JS_GetPrototypeFree(ctx, obj1); obj1 = assigned; assigned };
vm_block = 30; continue;
}
// C line 16368
32 => {
let _ = { let assigned = JS_DupValue(ctx, (*(it)).obj); obj1 = assigned; assigned };
vm_block = 16; continue;
}
// C line 16365
33 => {
let _ = { let assigned = ((*(p)).u).for_in_iterator; it = assigned; assigned };
vm_block = 32; continue;
}
// C line 16364
34 => {
let _ = { let assigned = ((((enum_obj).u).ptr) as *mut JSObject); p = assigned; assigned };
vm_block = 33; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:16418. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_for_in_next(mut ctx: *mut JSContext, mut sp: *mut JSValue) -> i32 {
let mut vm_local_storage = Vec::<u64>::new();
let mut enum_obj: JSValue = core::mem::zeroed();
let mut p: *mut JSObject = core::mem::zeroed();
let mut prop: JSAtom = core::mem::zeroed();
let mut it: *mut JSForInIterator = core::mem::zeroed();
let mut tab_atom: *mut JSPropertyEnum = core::mem::zeroed();
let mut tab_atom_count: u32 = core::mem::zeroed();
let mut ret: i32 = core::mem::zeroed();
let mut is_enumerable: i32 = core::mem::zeroed();
let mut vm_block: usize = 58;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 16509
1 => {
return (0 as i32);
}
// C line 16508
2 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((1 as i32)) as u32)) as u64) }, tag: (((JS_TAG_BOOL as i32)) as i64) }; *(sp).offset(((1 as i32)) as isize) = assigned; assigned };
vm_block = 1; continue;
}
// C line ? labels: done
3 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; *(sp).offset(((0 as i32)) as isize) = assigned; assigned };
vm_block = 2; continue;
}
// C line 16504
4 => {
return (0 as i32);
}
// C line 16503
5 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_BOOL as i32)) as i64) }; *(sp).offset(((1 as i32)) as isize) = assigned; assigned };
vm_block = 4; continue;
}
// C line 16502
6 => {
let _ = { let assigned = JS_AtomToValue(ctx, prop); *(sp).offset(((0 as i32)) as isize) = assigned; assigned };
vm_block = 5; continue;
}
// C line 16437
7 => {
vm_block = 51; continue;
}
// C line 16468
8 => {
let _ = { let assigned = (((0 as i32)) as u32); (*(it)).idx = assigned; assigned };
vm_block = 7; continue;
}
// C line 16467
9 => {
let _ = { let assigned = tab_atom_count; (*(it)).atom_count = assigned; assigned };
vm_block = 8; continue;
}
// C line 16466
10 => {
let _ = { let assigned = tab_atom; (*(it)).tab_atom = assigned; assigned };
vm_block = 9; continue;
}
// C line 16465
11 => {
let _ = JS_FreePropertyEnum(ctx, (*(it)).tab_atom, (*(it)).atom_count);
vm_block = 10; continue;
}
// C line 16463
12 => {
return ((1 as i32)).wrapping_neg();
}
// C line 16460
13 => {
vm_block = if (JS_GetOwnPropertyNamesInternal(ctx, core::ptr::addr_of_mut!(tab_atom), core::ptr::addr_of_mut!(tab_atom_count), (((((*(it)).obj).u).ptr) as *mut JSObject), ((((1 as i32)).wrapping_shl(((0 as i32)) as u32)) | (((1 as i32)).wrapping_shl(((5 as i32)) as u32))))) != 0 { 12 } else { 11 }; continue;
}
// C line 16458
14 => {
return ((1 as i32)).wrapping_neg();
}
// C line 16457
15 => {
vm_block = if (js_poll_interrupts(ctx)) != 0 { 14 } else { 13 }; continue;
}
// C line 16454
16 => {
vm_block = 3; continue;
}
// C line 16453
17 => {
vm_block = if (JS_IsNull((*(it)).obj)) != 0 { 16 } else { 15 }; continue;
}
// C line 16452
18 => {
return ((1 as i32)).wrapping_neg();
}
// C line 16451
19 => {
vm_block = if (JS_IsException((*(it)).obj)) != 0 { 18 } else { 17 }; continue;
}
// C line 16450
20 => {
let _ = { let assigned = JS_GetPrototypeFree(ctx, (*(it)).obj); (*(it)).obj = assigned; assigned };
vm_block = 19; continue;
}
// C line 16448
21 => {
let _ = { let assigned = (((1 as i32)) as u8); (*(it)).in_prototype_chain = assigned; assigned };
vm_block = 20; continue;
}
// C line 16447
22 => {
vm_block = 3; continue;
}
// C line 16446
23 => {
vm_block = if (ret) != 0 { 22 } else { 21 }; continue;
}
// C line 16445
24 => {
return ((1 as i32)).wrapping_neg();
}
// C line 16444
25 => {
vm_block = if ((((ret) < ((0 as i32))) as i32)) != 0 { 24 } else { 23 }; continue;
}
// C line 16443
26 => {
let _ = { let assigned = js_for_in_prepare_prototype_chain_enum(ctx, enum_obj); ret = assigned; assigned };
vm_block = 25; continue;
}
// C line 16442
27 => {
vm_block = if ((!(((*(it)).in_prototype_chain) != 0) as i32)) != 0 { 26 } else { 20 }; continue;
}
// C line 16440
28 => {
vm_block = 3; continue;
}
// C line 16439
29 => {
vm_block = if (((((JS_IsNull((*(it)).obj)) != 0) || ((JS_IsUndefined((*(it)).obj)) != 0)) as i32)) != 0 { 28 } else { 27 }; continue;
}
// C line 16498
30 => {
vm_block = 6; continue;
}
// C line 16497
31 => {
vm_block = if (ret) != 0 { 30 } else { 7 }; continue;
}
// C line 16496
32 => {
return ret;
}
// C line 16495
33 => {
vm_block = if ((((ret) < ((0 as i32))) as i32)) != 0 { 32 } else { 31 }; continue;
}
// C line 16494
34 => {
let _ = { let assigned = JS_GetOwnPropertyInternal(ctx, core::ptr::null_mut::<JSPropertyDescriptor>(), (((((*(it)).obj).u).ptr) as *mut JSObject), prop); ret = assigned; assigned };
vm_block = 33; continue;
}
// C line 16472
35 => {
let _ = { let old = (*(it)).idx; (*(it)).idx = ((*(it)).idx).wrapping_add(1); old };
vm_block = 34; continue;
}
// C line 16471
36 => {
let _ = { let assigned = __JS_AtomFromUInt32((*(it)).idx); prop = assigned; assigned };
vm_block = 35; continue;
}
// C line 16491
37 => {
vm_block = 7; continue;
}
// C line 16490
38 => {
vm_block = if ((!((is_enumerable) != 0) as i32)) != 0 { 37 } else { 34 }; continue;
}
// C line 16488
39 => {
return ((1 as i32)).wrapping_neg();
}
// C line 16486
40 => {
vm_block = if ((((JS_DefinePropertyValue(ctx, enum_obj, prop, JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_NULL as i32)) as i64) }, ((1 as i32)).wrapping_shl(((2 as i32)) as u32))) < ((0 as i32))) as i32)) != 0 { 39 } else { 38 }; continue;
}
// C line 16484
41 => {
vm_block = 7; continue;
}
// C line 16483
42 => {
vm_block = if (ret) != 0 { 41 } else { 40 }; continue;
}
// C line 16482
43 => {
return ret;
}
// C line 16481
44 => {
vm_block = if ((((ret) < ((0 as i32))) as i32)) != 0 { 43 } else { 42 }; continue;
}
// C line 16480
45 => {
let _ = { let assigned = JS_GetOwnPropertyInternal(ctx, core::ptr::null_mut::<JSPropertyDescriptor>(), ((((enum_obj).u).ptr) as *mut JSObject), prop); ret = assigned; assigned };
vm_block = 44; continue;
}
// C line 16478
46 => {
vm_block = if ((*(it)).in_prototype_chain) != 0 { 45 } else { 38 }; continue;
}
// C line 16477
47 => {
let _ = { let old = (*(it)).idx; (*(it)).idx = ((*(it)).idx).wrapping_add(1); old };
vm_block = 46; continue;
}
// C line 16476
48 => {
let _ = { let assigned = (*((*(it)).tab_atom).offset(((*(it)).idx) as isize)).is_enumerable; is_enumerable = assigned; assigned };
vm_block = 47; continue;
}
// C line 16475
49 => {
let _ = { let assigned = (*((*(it)).tab_atom).offset(((*(it)).idx) as isize)).atom; prop = assigned; assigned };
vm_block = 48; continue;
}
// C line 16470
50 => {
vm_block = if ((*(it)).is_array) != 0 { 36 } else { 49 }; continue;
}
// C line 16438
51 => {
vm_block = if (((((*(it)).idx) >= ((*(it)).atom_count)) as i32)) != 0 { 29 } else { 50 }; continue;
}
// C line 16435
52 => {
let _ = { let assigned = ((*(p)).u).for_in_iterator; it = assigned; assigned };
vm_block = 7; continue;
}
// C line 16434
53 => {
vm_block = 3; continue;
}
// C line 16433
54 => {
vm_block = if (((((((*(p)).class_id) as i32)) != ((JS_CLASS_FOR_IN_ITERATOR as i32))) as i32)) != 0 { 53 } else { 52 }; continue;
}
// C line 16432
55 => {
let _ = { let assigned = ((((enum_obj).u).ptr) as *mut JSObject); p = assigned; assigned };
vm_block = 54; continue;
}
// C line 16431
56 => {
vm_block = 3; continue;
}
// C line 16430
57 => {
vm_block = if (((((((enum_obj).tag) as i32)) != ((JS_TAG_OBJECT as i32))) as i32)) != 0 { 56 } else { 55 }; continue;
}
// C line 16428
58 => {
let _ = { let assigned = *(sp).offset((((1 as i32)).wrapping_neg()) as isize); enum_obj = assigned; assigned };
vm_block = 57; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:16512. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn JS_GetIterator2(mut ctx: *mut JSContext, mut obj: JSValue, mut method: JSValue) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut enum_obj: JSValue = core::mem::zeroed();
let mut vm_block: usize = 7;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 16524
1 => {
return enum_obj;
}
// C line 16522
2 => {
return JS_ThrowTypeErrorNotAnObject(ctx);
}
// C line 16521
3 => {
let _ = JS_FreeValue(ctx, enum_obj);
vm_block = 2; continue;
}
// C line 16520
4 => {
vm_block = if ((!((JS_IsObject(enum_obj)) != 0) as i32)) != 0 { 3 } else { 1 }; continue;
}
// C line 16519
5 => {
return enum_obj;
}
// C line 16518
6 => {
vm_block = if (JS_IsException(enum_obj)) != 0 { 5 } else { 4 }; continue;
}
// C line 16517
7 => {
let _ = { let assigned = JS_Call(ctx, method, obj, (0 as i32), core::ptr::null_mut::<JSValue>()); enum_obj = assigned; assigned };
vm_block = 6; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:16527. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn JS_GetIterator(mut ctx: *mut JSContext, mut obj: JSValue, mut is_async: i32) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut method: JSValue = core::mem::zeroed();
let mut ret: JSValue = core::mem::zeroed();
let mut sync_iter: JSValue = core::mem::zeroed();
let mut vm_block: usize = 24;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 16558
1 => {
return ret;
}
// C line 16557
2 => {
let _ = JS_FreeValue(ctx, method);
vm_block = 1; continue;
}
// C line 16556
3 => {
let _ = { let assigned = JS_GetIterator2(ctx, obj, method); ret = assigned; assigned };
vm_block = 2; continue;
}
// C line 16554
4 => {
return JS_ThrowTypeError(ctx, c"value is not iterable".as_ptr());
}
// C line 16553
5 => {
let _ = JS_FreeValue(ctx, method);
vm_block = 4; continue;
}
// C line 16552
6 => {
vm_block = if ((!((JS_IsFunction(ctx, method)) != 0) as i32)) != 0 { 5 } else { 3 }; continue;
}
// C line 16545
7 => {
return ret;
}
// C line 16544
8 => {
let _ = JS_FreeValue(ctx, sync_iter);
vm_block = 7; continue;
}
// C line 16543
9 => {
let _ = { let assigned = JS_CreateAsyncFromSyncIterator(ctx, sync_iter); ret = assigned; assigned };
vm_block = 8; continue;
}
// C line 16542
10 => {
return sync_iter;
}
// C line 16541
11 => {
vm_block = if (JS_IsException(sync_iter)) != 0 { 10 } else { 9 }; continue;
}
// C line 16540
12 => {
let _ = JS_FreeValue(ctx, method);
vm_block = 11; continue;
}
// C line 16539
13 => {
let _ = { let assigned = JS_GetIterator2(ctx, obj, method); sync_iter = assigned; assigned };
vm_block = 12; continue;
}
// C line 16538
14 => {
return method;
}
// C line 16537
15 => {
vm_block = if (JS_IsException(method)) != 0 { 14 } else { 13 }; continue;
}
// C line 16536
16 => {
let _ = { let assigned = JS_GetProperty(ctx, obj, (((JS_ATOM_Symbol_iterator as i32)) as JSAtom)); method = assigned; assigned };
vm_block = 15; continue;
}
// C line 16535
17 => {
vm_block = if (((((JS_IsUndefined(method)) != 0) || ((JS_IsNull(method)) != 0)) as i32)) != 0 { 16 } else { 6 }; continue;
}
// C line 16534
18 => {
return method;
}
// C line 16533
19 => {
vm_block = if (JS_IsException(method)) != 0 { 18 } else { 17 }; continue;
}
// C line 16532
20 => {
let _ = { let assigned = JS_GetProperty(ctx, obj, (((JS_ATOM_Symbol_asyncIterator as i32)) as JSAtom)); method = assigned; assigned };
vm_block = 19; continue;
}
// C line 16550
21 => {
return method;
}
// C line 16549
22 => {
vm_block = if (JS_IsException(method)) != 0 { 21 } else { 6 }; continue;
}
// C line 16548
23 => {
let _ = { let assigned = JS_GetProperty(ctx, obj, (((JS_ATOM_Symbol_iterator as i32)) as JSAtom)); method = assigned; assigned };
vm_block = 22; continue;
}
// C line 16531
24 => {
vm_block = if (is_async) != 0 { 20 } else { 23 }; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:16562. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn JS_IteratorNext2(mut ctx: *mut JSContext, mut enum_obj: JSValue, mut method: JSValue, mut argc: i32, mut argv: *mut JSValue, mut pdone: *mut i32) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut obj: JSValue = core::mem::zeroed();
let mut p: *mut JSObject = core::mem::zeroed();
let mut func: JSCFunctionType = core::mem::zeroed();
let mut args: [JSValue; 1] = core::mem::zeroed();
let mut vm_block: usize = 19;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 16599
1 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line ? labels: fail
2 => {
let _ = { let assigned = (0 as i32); *(pdone) = assigned; assigned };
vm_block = 1; continue;
}
// C line 16596
3 => {
return obj;
}
// C line 16595
4 => {
let _ = { let assigned = (2 as i32); *(pdone) = assigned; assigned };
vm_block = 3; continue;
}
// C line 16593
5 => {
vm_block = 2; continue;
}
// C line 16592
6 => {
let _ = JS_ThrowTypeError(ctx, c"iterator must return an object".as_ptr());
vm_block = 5; continue;
}
// C line 16591
7 => {
let _ = JS_FreeValue(ctx, obj);
vm_block = 6; continue;
}
// C line 16590
8 => {
vm_block = if ((!((JS_IsObject(obj)) != 0) as i32)) != 0 { 7 } else { 4 }; continue;
}
// C line 16589
9 => {
vm_block = 2; continue;
}
// C line 16588
10 => {
vm_block = if (JS_IsException(obj)) != 0 { 9 } else { 8 }; continue;
}
// C line 16587
11 => {
let _ = { let assigned = JS_Call(ctx, method, enum_obj, argc, argv); obj = assigned; assigned };
vm_block = 10; continue;
}
// C line 16583
12 => {
return ((func).iterator_next).expect("registered function callback")(ctx, enum_obj, argc, argv, pdone, (((((*(p)).u).cfunc).magic) as i32));
}
// C line 16582
13 => {
let _ = { let assigned = (((*(p)).u).cfunc).c_function; func = assigned; assigned };
vm_block = 12; continue;
}
// C line 16580
14 => {
let _ = { let assigned = (args).as_mut_ptr(); argv = assigned; assigned };
vm_block = 13; continue;
}
// C line 16579
15 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; *((args).as_mut_ptr()).offset(((0 as i32)) as isize) = assigned; assigned };
vm_block = 14; continue;
}
// C line 16578
16 => {
vm_block = if ((((argc) == ((0 as i32))) as i32)) != 0 { 15 } else { 13 }; continue;
}
// C line 16572
17 => {
vm_block = if (((((((((((*(p)).class_id) as i32)) == ((JS_CLASS_C_FUNCTION as i32))) as i32)) != 0) && ((((((((((*(p)).u).cfunc).cproto) as i32)) == ((JS_CFUNC_iterator_next as i32))) as i32)) != 0)) as i32)) != 0 { 16 } else { 11 }; continue;
}
// C line 16571
18 => {
p = ((((method).u).ptr) as *mut JSObject);
vm_block = 17; continue;
}
// C line 16570
19 => {
vm_block = if (JS_IsObject(method)) != 0 { 18 } else { 11 }; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:16603. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn JS_IteratorNext(mut ctx: *mut JSContext, mut enum_obj: JSValue, mut method: JSValue, mut argc: i32, mut argv: *mut JSValue, mut pdone: *mut i32) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut obj: JSValue = core::mem::zeroed();
let mut value: JSValue = core::mem::zeroed();
let mut done_val: JSValue = core::mem::zeroed();
let mut done: i32 = core::mem::zeroed();
let mut vm_block: usize = 22;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 16635
1 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 16634
2 => {
let _ = { let assigned = (0 as i32); *(pdone) = assigned; assigned };
vm_block = 1; continue;
}
// C line ? labels: fail
3 => {
let _ = JS_FreeValue(ctx, obj);
vm_block = 2; continue;
}
// C line 16615
4 => {
return obj;
}
// C line 16614
5 => {
let _ = { let assigned = (0 as i32); *(pdone) = assigned; assigned };
vm_block = 4; continue;
}
// C line 16619
6 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) };
}
// C line 16618
7 => {
let _ = { let assigned = (1 as i32); *(pdone) = assigned; assigned };
vm_block = 6; continue;
}
// C line 16617
8 => {
let _ = JS_FreeValue(ctx, obj);
vm_block = 7; continue;
}
// C line 16630
9 => {
return value;
}
// C line 16629
10 => {
let _ = JS_FreeValue(ctx, obj);
vm_block = 9; continue;
}
// C line 16627
11 => {
let _ = { let assigned = JS_GetProperty(ctx, obj, (((JS_ATOM_value as i32)) as JSAtom)); value = assigned; assigned };
vm_block = 10; continue;
}
// C line 16626
12 => {
vm_block = if ((!((*(pdone)) != 0) as i32)) != 0 { 11 } else { 10 }; continue;
}
// C line 16625
13 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; value = assigned; assigned };
vm_block = 12; continue;
}
// C line 16624
14 => {
let _ = { let assigned = JS_ToBoolFree(ctx, done_val); *(pdone) = assigned; assigned };
vm_block = 13; continue;
}
// C line 16623
15 => {
vm_block = 3; continue;
}
// C line 16622
16 => {
vm_block = if (JS_IsException(done_val)) != 0 { 15 } else { 14 }; continue;
}
// C line 16621
17 => {
let _ = { let assigned = JS_GetProperty(ctx, obj, (((JS_ATOM_done as i32)) as JSAtom)); done_val = assigned; assigned };
vm_block = 16; continue;
}
// C line 16616
18 => {
vm_block = if ((((done) != ((2 as i32))) as i32)) != 0 { 8 } else { 17 }; continue;
}
// C line 16613
19 => {
vm_block = if ((((!(((!(((((done) == ((0 as i32))) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 5 } else { 18 }; continue;
}
// C line 16612
20 => {
vm_block = 3; continue;
}
// C line 16611
21 => {
vm_block = if (JS_IsException(obj)) != 0 { 20 } else { 19 }; continue;
}
// C line 16610
22 => {
let _ = { let assigned = JS_IteratorNext2(ctx, enum_obj, method, argc, argv, core::ptr::addr_of_mut!(done)); obj = assigned; assigned };
vm_block = 21; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:16639. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn JS_IteratorClose(mut ctx: *mut JSContext, mut enum_obj: JSValue, mut is_exception_pending: i32) -> i32 {
let _host_exception = JSHostPreserveException::new(ctx, is_exception_pending != 0);
let mut vm_local_storage = Vec::<u64>::new();
let mut method: JSValue = core::mem::zeroed();
let mut ret: JSValue = core::mem::zeroed();
let mut ex_obj: JSValue = core::mem::zeroed();
let mut res: i32 = core::mem::zeroed();
let mut vm_block: usize = 23;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 16675
1 => {
return res;
}
// C line 16673
2 => {
let _ = JS_Throw(ctx, ex_obj);
vm_block = 1; continue;
}
// C line 16672 labels: done
3 => {
vm_block = if (is_exception_pending) != 0 { 2 } else { 1 }; continue;
}
// C line 16670
4 => {
let _ = JS_FreeValue(ctx, ret);
vm_block = 3; continue;
}
// C line 16664
5 => {
let _ = { let assigned = ((1 as i32)).wrapping_neg(); res = assigned; assigned };
vm_block = 4; continue;
}
// C line 16667
6 => {
let _ = { let assigned = ((1 as i32)).wrapping_neg(); res = assigned; assigned };
vm_block = 4; continue;
}
// C line 16666
7 => {
let _ = JS_ThrowTypeErrorNotAnObject(ctx);
vm_block = 6; continue;
}
// C line 16665
8 => {
vm_block = if ((!((JS_IsObject(ret)) != 0) as i32)) != 0 { 7 } else { 4 }; continue;
}
// C line 16663
9 => {
vm_block = if (JS_IsException(ret)) != 0 { 5 } else { 8 }; continue;
}
// C line 16662
10 => {
vm_block = if ((!((is_exception_pending) != 0) as i32)) != 0 { 9 } else { 4 }; continue;
}
// C line 16661
11 => {
let _ = { let assigned = JS_CallFree(ctx, method, enum_obj, (0 as i32), core::ptr::null_mut::<JSValue>()); ret = assigned; assigned };
vm_block = 10; continue;
}
// C line 16659
12 => {
vm_block = 3; continue;
}
// C line 16658
13 => {
vm_block = if (((((JS_IsUndefined(method)) != 0) || ((JS_IsNull(method)) != 0)) as i32)) != 0 { 12 } else { 11 }; continue;
}
// C line 16656
14 => {
vm_block = 3; continue;
}
// C line 16655
15 => {
let _ = { let assigned = ((1 as i32)).wrapping_neg(); res = assigned; assigned };
vm_block = 14; continue;
}
// C line 16654
16 => {
vm_block = if (JS_IsException(method)) != 0 { 15 } else { 13 }; continue;
}
// C line 16653
17 => {
let _ = { let assigned = JS_GetProperty(ctx, enum_obj, (((crate::quickjs_atom::JS_ATOM_return as i32)) as JSAtom)); method = assigned; assigned };
vm_block = 16; continue;
}
// C line 16648
18 => {
let _ = { let assigned = ((1 as i32)).wrapping_neg(); res = assigned; assigned };
vm_block = 17; continue;
}
// C line 16647
19 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNINITIALIZED as i32)) as i64) }; (*((*(ctx)).rt)).current_exception = assigned; assigned };
vm_block = 18; continue;
}
// C line 16646
20 => {
let _ = { let assigned = (*((*(ctx)).rt)).current_exception; ex_obj = assigned; assigned };
vm_block = 19; continue;
}
// C line 16651
21 => {
let _ = { let assigned = (0 as i32); res = assigned; assigned };
vm_block = 17; continue;
}
// C line 16650
22 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; ex_obj = assigned; assigned };
vm_block = 21; continue;
}
// C line 16645
23 => {
vm_block = if (is_exception_pending) != 0 { 20 } else { 22 }; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:16679. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_for_of_start(mut ctx: *mut JSContext, mut sp: *mut JSValue, mut is_async: i32) -> i32 {
let mut vm_local_storage = Vec::<u64>::new();
let mut op1: JSValue = core::mem::zeroed();
let mut obj: JSValue = core::mem::zeroed();
let mut method: JSValue = core::mem::zeroed();
let mut vm_block: usize = 11;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 16693
1 => {
return (0 as i32);
}
// C line 16692
2 => {
let _ = { let assigned = method; *(sp).offset(((0 as i32)) as isize) = assigned; assigned };
vm_block = 1; continue;
}
// C line 16691
3 => {
return ((1 as i32)).wrapping_neg();
}
// C line 16690
4 => {
vm_block = if (JS_IsException(method)) != 0 { 3 } else { 2 }; continue;
}
// C line 16689
5 => {
let _ = { let assigned = JS_GetProperty(ctx, obj, (((JS_ATOM_next as i32)) as JSAtom)); method = assigned; assigned };
vm_block = 4; continue;
}
// C line 16688
6 => {
let _ = { let assigned = obj; *(sp).offset((((1 as i32)).wrapping_neg()) as isize) = assigned; assigned };
vm_block = 5; continue;
}
// C line 16687
7 => {
let _ = JS_FreeValue(ctx, op1);
vm_block = 6; continue;
}
// C line 16686
8 => {
return ((1 as i32)).wrapping_neg();
}
// C line 16685
9 => {
vm_block = if (JS_IsException(obj)) != 0 { 8 } else { 7 }; continue;
}
// C line 16684
10 => {
let _ = { let assigned = JS_GetIterator(ctx, op1, is_async); obj = assigned; assigned };
vm_block = 9; continue;
}
// C line 16683
11 => {
let _ = { let assigned = *(sp).offset((((1 as i32)).wrapping_neg()) as isize); op1 = assigned; assigned };
vm_block = 10; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:16700. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_for_of_next(mut ctx: *mut JSContext, mut sp: *mut JSValue, mut offset: i32) -> i32 {
let mut vm_local_storage = Vec::<u64>::new();
let mut value: JSValue = core::mem::zeroed();
let mut done: i32 = core::mem::zeroed();
let mut vm_block: usize = 16;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 16724
1 => {
return (0 as i32);
}
// C line 16723
2 => {
let _ = { let assigned = JS_NewBool(ctx, done); *(sp).offset(((1 as i32)) as isize) = assigned; assigned };
vm_block = 1; continue;
}
// C line 16722
3 => {
let _ = { let assigned = value; *(sp).offset(((0 as i32)) as isize) = assigned; assigned };
vm_block = 2; continue;
}
// C line 16715
4 => {
return ((1 as i32)).wrapping_neg();
}
// C line 16718
5 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; value = assigned; assigned };
vm_block = 3; continue;
}
// C line 16717
6 => {
let _ = JS_FreeValue(ctx, value);
vm_block = 5; continue;
}
// C line 16714
7 => {
vm_block = if ((((done) < ((0 as i32))) as i32)) != 0 { 4 } else { 6 }; continue;
}
// C line 16713
8 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; *(sp).offset((offset) as isize) = assigned; assigned };
vm_block = 7; continue;
}
// C line 16712
9 => {
let _ = JS_FreeValue(ctx, *(sp).offset((offset) as isize));
vm_block = 8; continue;
}
// C line 16709
10 => {
vm_block = if (done) != 0 { 9 } else { 3 }; continue;
}
// C line 16708
11 => {
let _ = { let assigned = ((1 as i32)).wrapping_neg(); done = assigned; assigned };
vm_block = 10; continue;
}
// C line 16707
12 => {
vm_block = if (JS_IsException(value)) != 0 { 11 } else { 10 }; continue;
}
// C line 16706
13 => {
let _ = { let assigned = JS_IteratorNext(ctx, *(sp).offset((offset) as isize), *(sp).offset(((offset).wrapping_add((1 as i32))) as isize), (0 as i32), core::ptr::null_mut::<JSValue>(), core::ptr::addr_of_mut!(done)); value = assigned; assigned };
vm_block = 12; continue;
}
// C line 16705
14 => {
vm_block = if ((((!(((!(((!((JS_IsUndefined(*(sp).offset((offset) as isize))) != 0) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 13 } else { 3 }; continue;
}
// C line 16703
15 => {
done = (1 as i32);
vm_block = 14; continue;
}
// C line 16702
16 => {
value = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) };
vm_block = 15; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:16727. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_for_await_of_next(mut ctx: *mut JSContext, mut sp: *mut JSValue) -> i32 {
let mut vm_local_storage = Vec::<u64>::new();
let mut obj: JSValue = core::mem::zeroed();
let mut iter: JSValue = core::mem::zeroed();
let mut next: JSValue = core::mem::zeroed();
let mut vm_block: usize = 8;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 16739
1 => {
return (0 as i32);
}
// C line 16738
2 => {
let _ = { let assigned = obj; *(sp).offset(((0 as i32)) as isize) = assigned; assigned };
vm_block = 1; continue;
}
// C line 16737
3 => {
return ((1 as i32)).wrapping_neg();
}
// C line 16736
4 => {
vm_block = if (JS_IsException(obj)) != 0 { 3 } else { 2 }; continue;
}
// C line 16735
5 => {
let _ = { let assigned = JS_Call(ctx, next, iter, (0 as i32), core::ptr::null_mut::<JSValue>()); obj = assigned; assigned };
vm_block = 4; continue;
}
// C line 16734
6 => {
let _ = { let assigned = *(sp).offset((((2 as i32)).wrapping_neg()) as isize); next = assigned; assigned };
vm_block = 5; continue;
}
// C line 16733
7 => {
let _ = { let assigned = *(sp).offset((((3 as i32)).wrapping_neg()) as isize); iter = assigned; assigned };
vm_block = 6; continue;
}
// C line 16731
8 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; *(sp).offset((((1 as i32)).wrapping_neg()) as isize) = assigned; assigned };
vm_block = 7; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:16742. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn JS_IteratorGetCompleteValue(mut ctx: *mut JSContext, mut obj: JSValue, mut pdone: *mut i32) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut done_val: JSValue = core::mem::zeroed();
let mut value: JSValue = core::mem::zeroed();
let mut done: i32 = core::mem::zeroed();
let mut vm_block: usize = 11;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 16758
1 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line ? labels: fail
2 => {
let _ = { let assigned = (0 as i32); *(pdone) = assigned; assigned };
vm_block = 1; continue;
}
// C line 16755
3 => {
return value;
}
// C line 16754
4 => {
let _ = { let assigned = done; *(pdone) = assigned; assigned };
vm_block = 3; continue;
}
// C line 16753
5 => {
vm_block = 2; continue;
}
// C line 16752
6 => {
vm_block = if (JS_IsException(value)) != 0 { 5 } else { 4 }; continue;
}
// C line 16751
7 => {
let _ = { let assigned = JS_GetProperty(ctx, obj, (((JS_ATOM_value as i32)) as JSAtom)); value = assigned; assigned };
vm_block = 6; continue;
}
// C line 16750
8 => {
let _ = { let assigned = JS_ToBoolFree(ctx, done_val); done = assigned; assigned };
vm_block = 7; continue;
}
// C line 16749
9 => {
vm_block = 2; continue;
}
// C line 16748
10 => {
vm_block = if (JS_IsException(done_val)) != 0 { 9 } else { 8 }; continue;
}
// C line 16747
11 => {
let _ = { let assigned = JS_GetProperty(ctx, obj, (((JS_ATOM_done as i32)) as JSAtom)); done_val = assigned; assigned };
vm_block = 10; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:16761. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_iterator_get_value_done(mut ctx: *mut JSContext, mut sp: *mut JSValue) -> i32 {
let mut vm_local_storage = Vec::<u64>::new();
let mut obj: JSValue = core::mem::zeroed();
let mut value: JSValue = core::mem::zeroed();
let mut done: i32 = core::mem::zeroed();
let mut vm_block: usize = 12;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 16779
1 => {
return (0 as i32);
}
// C line 16778
2 => {
let _ = { let assigned = JS_NewBool(ctx, done); *(sp).offset(((0 as i32)) as isize) = assigned; assigned };
vm_block = 1; continue;
}
// C line 16777
3 => {
let _ = { let assigned = value; *(sp).offset((((1 as i32)).wrapping_neg()) as isize) = assigned; assigned };
vm_block = 2; continue;
}
// C line 16776
4 => {
let _ = { let assigned = JS_NewCatchOffset(ctx, (0 as i32)); *(sp).offset((((2 as i32)).wrapping_neg()) as isize) = assigned; assigned };
vm_block = 3; continue;
}
// C line 16773
5 => {
let _ = JS_FreeValue(ctx, obj);
vm_block = 4; continue;
}
// C line 16772
6 => {
return ((1 as i32)).wrapping_neg();
}
// C line 16771
7 => {
vm_block = if (JS_IsException(value)) != 0 { 6 } else { 5 }; continue;
}
// C line 16770
8 => {
let _ = { let assigned = JS_IteratorGetCompleteValue(ctx, obj, core::ptr::addr_of_mut!(done)); value = assigned; assigned };
vm_block = 7; continue;
}
// C line 16768
9 => {
return ((1 as i32)).wrapping_neg();
}
// C line 16767
10 => {
let _ = JS_ThrowTypeError(ctx, c"iterator must return an object".as_ptr());
vm_block = 9; continue;
}
// C line 16766
11 => {
vm_block = if ((!((JS_IsObject(obj)) != 0) as i32)) != 0 { 10 } else { 8 }; continue;
}
// C line 16765
12 => {
let _ = { let assigned = *(sp).offset((((1 as i32)).wrapping_neg()) as isize); obj = assigned; assigned };
vm_block = 11; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:16812. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_is_fast_array(mut ctx: *mut JSContext, mut obj: JSValue) -> i32 {
let mut vm_local_storage = Vec::<u64>::new();
let mut p: *mut JSObject = core::mem::zeroed();
let mut vm_block: usize = 5;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 16821
1 => {
return (0 as i32);
}
// C line 16818
2 => {
return (1 as i32);
}
// C line 16817
3 => {
vm_block = if (((((((((((*(p)).class_id) as i32)) == ((JS_CLASS_ARRAY as i32))) as i32)) != 0) && (((((*(p)).fast_array()) as i32)) != 0)) as i32)) != 0 { 2 } else { 1 }; continue;
}
// C line 16816
4 => {
p = ((((obj).u).ptr) as *mut JSObject);
vm_block = 3; continue;
}
// C line 16815
5 => {
vm_block = if (((((((obj).tag) as i32)) == ((JS_TAG_OBJECT as i32))) as i32)) != 0 { 4 } else { 1 }; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:16825. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_get_fast_array(mut ctx: *mut JSContext, mut obj: JSValue, mut arrpp: *mut *mut JSValue, mut countp: *mut u32) -> i32 {
let mut vm_local_storage = Vec::<u64>::new();
let mut p: *mut JSObject = core::mem::zeroed();
let mut vm_block: usize = 7;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 16837
1 => {
return (0 as i32);
}
// C line 16834
2 => {
return (1 as i32);
}
// C line 16833
3 => {
let _ = { let assigned = ((((*(p)).u).array).u).values; *(arrpp) = assigned; assigned };
vm_block = 2; continue;
}
// C line 16832
4 => {
let _ = { let assigned = (((*(p)).u).array).count; *(countp) = assigned; assigned };
vm_block = 3; continue;
}
// C line 16831
5 => {
vm_block = if (((((((((((*(p)).class_id) as i32)) == ((JS_CLASS_ARRAY as i32))) as i32)) != 0) && (((((*(p)).fast_array()) as i32)) != 0)) as i32)) != 0 { 4 } else { 1 }; continue;
}
// C line 16830
6 => {
p = ((((obj).u).ptr) as *mut JSObject);
vm_block = 5; continue;
}
// C line 16829
7 => {
vm_block = if (((((((obj).tag) as i32)) == ((JS_TAG_OBJECT as i32))) as i32)) != 0 { 6 } else { 1 }; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:16840. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_append_enumerate(mut ctx: *mut JSContext, mut sp: *mut JSValue) -> i32 {
let mut vm_local_storage = Vec::<u64>::new();
let mut iterator: JSValue = core::mem::zeroed();
let mut enumobj: JSValue = core::mem::zeroed();
let mut method: JSValue = core::mem::zeroed();
let mut value: JSValue = core::mem::zeroed();
let mut is_array_iterator: i32 = core::mem::zeroed();
let mut arrp: *mut JSValue = core::mem::zeroed();
let mut i: u32 = core::mem::zeroed();
let mut count32: u32 = core::mem::zeroed();
let mut pos: u32 = core::mem::zeroed();
let mut ft: JSCFunctionType = core::mem::zeroed();
let mut len: u32 = core::mem::zeroed();
let mut done: i32 = core::mem::zeroed();
let mut vm_block: usize = 44;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 16920
1 => {
return ((1 as i32)).wrapping_neg();
}
// C line 16919
2 => {
let _ = JS_FreeValue(ctx, method);
vm_block = 1; continue;
}
// C line 16918
3 => {
let _ = JS_FreeValue(ctx, enumobj);
vm_block = 2; continue;
}
// C line ? labels: exception
4 => {
let _ = JS_IteratorClose(ctx, enumobj, (1 as i32));
vm_block = 3; continue;
}
// C line 16914
5 => {
return (0 as i32);
}
// C line 16913
6 => {
let _ = JS_FreeValue(ctx, method);
vm_block = 5; continue;
}
// C line 16912
7 => {
let _ = JS_FreeValue(ctx, enumobj);
vm_block = 6; continue;
}
// C line 16911
8 => {
let _ = { let assigned = JS_NewInt32(ctx, ((pos) as i32)); *(sp).offset((((2 as i32)).wrapping_neg()) as isize) = assigned; assigned };
vm_block = 7; continue;
}
// C line 16890
9 => {
vm_block = if ((((i) < (count32)) as i32)) != 0 { 12 } else { 8 }; continue;
}
// C line ?
10 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 9; continue;
}
// C line 16893
11 => {
vm_block = 4; continue;
}
// C line 16891
12 => {
vm_block = if ((((JS_DefinePropertyValueUint32(ctx, *(sp).offset((((3 as i32)).wrapping_neg()) as isize), { let old = pos; pos = (pos).wrapping_add(1); old }, JS_DupValue(ctx, *(arrp).offset((i) as isize)), ((((((1 as i32)).wrapping_shl(((0 as i32)) as u32)) | (((1 as i32)).wrapping_shl(((1 as i32)) as u32)))) | (((1 as i32)).wrapping_shl(((2 as i32)) as u32))))) < ((0 as i32))) as i32)) != 0 { 11 } else { 10 }; continue;
}
// C line 16890
13 => {
let _ = { let assigned = (((0 as i32)) as u32); i = assigned; assigned };
vm_block = 9; continue;
}
// C line 16888
14 => {
vm_block = 18; continue;
}
// C line 16887
15 => {
vm_block = if ((((len) != (count32)) as i32)) != 0 { 14 } else { 13 }; continue;
}
// C line 16884
16 => {
vm_block = 4; continue;
}
// C line 16883
17 => {
vm_block = if (js_get_length32(ctx, core::ptr::addr_of_mut!(len), *(sp).offset((((1 as i32)).wrapping_neg()) as isize))) != 0 { 16 } else { 15 }; continue;
}
// C line 16897 labels: general_case
18 => {
vm_block = 25; continue;
}
// C line 16907
19 => {
vm_block = 4; continue;
}
// C line 16906
20 => {
vm_block = if ((((JS_DefinePropertyValueUint32(ctx, *(sp).offset((((3 as i32)).wrapping_neg()) as isize), { let old = pos; pos = (pos).wrapping_add(1); old }, value, ((((((1 as i32)).wrapping_shl(((0 as i32)) as u32)) | (((1 as i32)).wrapping_shl(((1 as i32)) as u32)))) | (((1 as i32)).wrapping_shl(((2 as i32)) as u32))))) < ((0 as i32))) as i32)) != 0 { 19 } else { 18 }; continue;
}
// C line 16904
21 => {
vm_block = 8; continue;
}
// C line 16902
22 => {
vm_block = if (done) != 0 { 21 } else { 20 }; continue;
}
// C line 16901
23 => {
vm_block = 4; continue;
}
// C line 16900
24 => {
vm_block = if (JS_IsException(value)) != 0 { 23 } else { 22 }; continue;
}
// C line 16899
25 => {
let _ = { let assigned = JS_IteratorNext(ctx, enumobj, method, (0 as i32), core::ptr::null_mut::<JSValue>(), core::ptr::addr_of_mut!(done)); value = assigned; assigned };
vm_block = 24; continue;
}
// C line 16879
26 => {
vm_block = if (((((((((is_array_iterator) != 0) && ((JS_IsCFunction(ctx, method, (ft).generic, (0 as i32))) != 0)) as i32)) != 0) && ((js_get_fast_array(ctx, *(sp).offset((((1 as i32)).wrapping_neg()) as isize), core::ptr::addr_of_mut!(arrp), core::ptr::addr_of_mut!(count32))) != 0)) as i32)) != 0 { 17 } else { 18 }; continue;
}
// C line 16878
27 => {
let _ = { let assigned = js_array_iterator_next; (ft).iterator_next = Some(assigned); assigned };
vm_block = 26; continue;
}
// C line 16875
28 => {
return ((1 as i32)).wrapping_neg();
}
// C line 16874
29 => {
let _ = JS_FreeValue(ctx, enumobj);
vm_block = 28; continue;
}
// C line 16873
30 => {
vm_block = if (JS_IsException(method)) != 0 { 29 } else { 27 }; continue;
}
// C line 16872
31 => {
let _ = { let assigned = JS_GetProperty(ctx, enumobj, (((JS_ATOM_next as i32)) as JSAtom)); method = assigned; assigned };
vm_block = 30; continue;
}
// C line 16871
32 => {
return ((1 as i32)).wrapping_neg();
}
// C line 16870
33 => {
vm_block = if (JS_IsException(enumobj)) != 0 { 32 } else { 31 }; continue;
}
// C line 16869
34 => {
let _ = { let assigned = JS_GetIterator(ctx, *(sp).offset((((1 as i32)).wrapping_neg()) as isize), (0 as i32)); enumobj = assigned; assigned };
vm_block = 33; continue;
}
// C line 16867
35 => {
let _ = JS_FreeValue(ctx, iterator);
vm_block = 34; continue;
}
// C line 16865
36 => {
let _ = { let assigned = JS_IsCFunction(ctx, iterator, (ft).generic, (JS_ITERATOR_KIND_VALUE as i32)); is_array_iterator = assigned; assigned };
vm_block = 35; continue;
}
// C line 16864
37 => {
let _ = { let assigned = js_create_array_iterator; (ft).generic_magic = Some(assigned); assigned };
vm_block = 36; continue;
}
// C line 16863
38 => {
return ((1 as i32)).wrapping_neg();
}
// C line 16862
39 => {
vm_block = if (JS_IsException(iterator)) != 0 { 38 } else { 37 }; continue;
}
// C line 16861
40 => {
let _ = { let assigned = JS_GetProperty(ctx, *(sp).offset((((1 as i32)).wrapping_neg()) as isize), (((JS_ATOM_Symbol_iterator as i32)) as JSAtom)); iterator = assigned; assigned };
vm_block = 39; continue;
}
// C line 16853
41 => {
let _ = { let assigned = ((((((*(sp).offset((((2 as i32)).wrapping_neg()) as isize)).u).uint64) as i32)) as u32); pos = assigned; assigned };
vm_block = 40; continue;
}
// C line 16850
42 => {
return ((1 as i32)).wrapping_neg();
}
// C line 16849
43 => {
let _ = JS_ThrowInternalError(ctx, c"invalid index for append".as_ptr());
vm_block = 42; continue;
}
// C line 16848
44 => {
vm_block = if (((((((*(sp).offset((((2 as i32)).wrapping_neg()) as isize)).tag) as i32)) != ((JS_TAG_INT as i32))) as i32)) != 0 { 43 } else { 41 }; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:16001. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_has_unscopable(mut ctx: *mut JSContext, mut obj: JSValue, mut atom: JSAtom) -> i32 {
let mut vm_local_storage = Vec::<u64>::new();
let mut arr: JSValue = core::mem::zeroed();
let mut val: JSValue = core::mem::zeroed();
let mut ret: i32 = core::mem::zeroed();
let mut vm_block: usize = 9;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 16016
1 => {
return ret;
}
// C line 16015
2 => {
let _ = JS_FreeValue(ctx, arr);
vm_block = 1; continue;
}
// C line 16013
3 => {
let _ = { let assigned = JS_ToBoolFree(ctx, val); ret = assigned; assigned };
vm_block = 2; continue;
}
// C line 16012
4 => {
let _ = { let assigned = JS_GetProperty(ctx, arr, atom); val = assigned; assigned };
vm_block = 3; continue;
}
// C line 16011
5 => {
vm_block = if (JS_IsObject(arr)) != 0 { 4 } else { 2 }; continue;
}
// C line 16010
6 => {
let _ = { let assigned = (0 as i32); ret = assigned; assigned };
vm_block = 5; continue;
}
// C line 16009
7 => {
return ((1 as i32)).wrapping_neg();
}
// C line 16008
8 => {
vm_block = if (JS_IsException(arr)) != 0 { 7 } else { 6 }; continue;
}
// C line 16007
9 => {
let _ = { let assigned = JS_GetProperty(ctx, obj, (((JS_ATOM_Symbol_unscopables as i32)) as JSAtom)); arr = assigned; assigned };
vm_block = 8; continue;
}
_ => std::process::abort(),
} }
}

