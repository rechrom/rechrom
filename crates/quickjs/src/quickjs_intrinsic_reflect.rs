// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:50185. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code, unused_variables)]
unsafe fn js_reflect_apply(mut ctx: *mut JSContext, mut this_val: JSValue, mut argc: i32, mut argv: *mut JSValue) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut vm_block: usize = 1;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 50188
1 => {
return js_function_apply(ctx, *(argv).offset(((0 as i32)) as isize), max_int((0 as i32), (argc).wrapping_sub((1 as i32))), (argv).offset((((1 as i32)) as isize)), (2 as i32));
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:50191. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code, unused_variables)]
unsafe fn js_reflect_construct(mut ctx: *mut JSContext, mut this_val: JSValue, mut argc: i32, mut argv: *mut JSValue) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut func: JSValue = core::mem::zeroed();
let mut array_arg: JSValue = core::mem::zeroed();
let mut new_target: JSValue = core::mem::zeroed();
let mut tab: *mut JSValue = core::mem::zeroed();
let mut ret: JSValue = core::mem::zeroed();
let mut len: u32 = core::mem::zeroed();
let mut vm_block: usize = 13;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 50212
1 => {
return ret;
}
// C line 50211
2 => {
let _ = free_arg_list(ctx, tab, len);
vm_block = 1; continue;
}
// C line 50210
3 => {
let _ = { let assigned = JS_CallConstructor2(ctx, func, new_target, ((len) as i32), tab); ret = assigned; assigned };
vm_block = 2; continue;
}
// C line 50209
4 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 50208
5 => {
vm_block = if ((!(!(tab).is_null()) as i32)) != 0 { 4 } else { 3 }; continue;
}
// C line 50207
6 => {
let _ = { let assigned = build_arg_list(ctx, core::ptr::addr_of_mut!(len), array_arg); tab = assigned; assigned };
vm_block = 5; continue;
}
// C line 50203
7 => {
return JS_ThrowTypeErrorNotAConstructor(ctx, new_target);
}
// C line 50202
8 => {
vm_block = if ((!((JS_IsConstructor(ctx, new_target)) != 0) as i32)) != 0 { 7 } else { 6 }; continue;
}
// C line 50201
9 => {
let _ = { let assigned = *(argv).offset(((2 as i32)) as isize); new_target = assigned; assigned };
vm_block = 8; continue;
}
// C line 50205
10 => {
let _ = { let assigned = func; new_target = assigned; assigned };
vm_block = 6; continue;
}
// C line 50200
11 => {
vm_block = if ((((argc) > ((2 as i32))) as i32)) != 0 { 9 } else { 10 }; continue;
}
// C line 50199
12 => {
let _ = { let assigned = *(argv).offset(((1 as i32)) as isize); array_arg = assigned; assigned };
vm_block = 11; continue;
}
// C line 50198
13 => {
let _ = { let assigned = *(argv).offset(((0 as i32)) as isize); func = assigned; assigned };
vm_block = 12; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:50215. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code, unused_variables)]
unsafe fn js_reflect_deleteProperty(mut ctx: *mut JSContext, mut this_val: JSValue, mut argc: i32, mut argv: *mut JSValue) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut obj: JSValue = core::mem::zeroed();
let mut atom: JSAtom = core::mem::zeroed();
let mut ret: i32 = core::mem::zeroed();
let mut vm_block: usize = 11;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 50231
1 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 50233
2 => {
return JS_NewBool(ctx, ret);
}
// C line 50230
3 => {
vm_block = if ((((ret) < ((0 as i32))) as i32)) != 0 { 1 } else { 2 }; continue;
}
// C line 50229
4 => {
let _ = JS_FreeAtom(ctx, atom);
vm_block = 3; continue;
}
// C line 50228
5 => {
let _ = { let assigned = JS_DeleteProperty(ctx, obj, atom, (0 as i32)); ret = assigned; assigned };
vm_block = 4; continue;
}
// C line 50227
6 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 50226
7 => {
vm_block = if ((((!(((!(((((atom) == ((((0 as i32)) as JSAtom))) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 6 } else { 5 }; continue;
}
// C line 50225
8 => {
let _ = { let assigned = JS_ValueToAtom(ctx, *(argv).offset(((1 as i32)) as isize)); atom = assigned; assigned };
vm_block = 7; continue;
}
// C line 50224
9 => {
return JS_ThrowTypeErrorNotAnObject(ctx);
}
// C line 50223
10 => {
vm_block = if (((((((obj).tag) as i32)) != ((JS_TAG_OBJECT as i32))) as i32)) != 0 { 9 } else { 8 }; continue;
}
// C line 50222
11 => {
let _ = { let assigned = *(argv).offset(((0 as i32)) as isize); obj = assigned; assigned };
vm_block = 10; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:50236. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code, unused_variables)]
unsafe fn js_reflect_get(mut ctx: *mut JSContext, mut this_val: JSValue, mut argc: i32, mut argv: *mut JSValue) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut obj: JSValue = core::mem::zeroed();
let mut prop: JSValue = core::mem::zeroed();
let mut receiver: JSValue = core::mem::zeroed();
let mut atom: JSAtom = core::mem::zeroed();
let mut ret: JSValue = core::mem::zeroed();
let mut vm_block: usize = 13;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 50256
1 => {
return ret;
}
// C line 50255
2 => {
let _ = JS_FreeAtom(ctx, atom);
vm_block = 1; continue;
}
// C line 50254
3 => {
let _ = { let assigned = JS_GetPropertyInternal(ctx, obj, atom, receiver, (0 as i32)); ret = assigned; assigned };
vm_block = 2; continue;
}
// C line 50253
4 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 50252
5 => {
vm_block = if ((((!(((!(((((atom) == ((((0 as i32)) as JSAtom))) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 4 } else { 3 }; continue;
}
// C line 50251
6 => {
let _ = { let assigned = JS_ValueToAtom(ctx, prop); atom = assigned; assigned };
vm_block = 5; continue;
}
// C line 50248
7 => {
let _ = { let assigned = *(argv).offset(((2 as i32)) as isize); receiver = assigned; assigned };
vm_block = 6; continue;
}
// C line 50250
8 => {
let _ = { let assigned = obj; receiver = assigned; assigned };
vm_block = 6; continue;
}
// C line 50247
9 => {
vm_block = if ((((argc) > ((2 as i32))) as i32)) != 0 { 7 } else { 8 }; continue;
}
// C line 50246
10 => {
return JS_ThrowTypeErrorNotAnObject(ctx);
}
// C line 50245
11 => {
vm_block = if (((((((obj).tag) as i32)) != ((JS_TAG_OBJECT as i32))) as i32)) != 0 { 10 } else { 9 }; continue;
}
// C line 50244
12 => {
let _ = { let assigned = *(argv).offset(((1 as i32)) as isize); prop = assigned; assigned };
vm_block = 11; continue;
}
// C line 50243
13 => {
let _ = { let assigned = *(argv).offset(((0 as i32)) as isize); obj = assigned; assigned };
vm_block = 12; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:50259. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code, unused_variables)]
unsafe fn js_reflect_has(mut ctx: *mut JSContext, mut this_val: JSValue, mut argc: i32, mut argv: *mut JSValue) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut obj: JSValue = core::mem::zeroed();
let mut prop: JSValue = core::mem::zeroed();
let mut atom: JSAtom = core::mem::zeroed();
let mut ret: i32 = core::mem::zeroed();
let mut vm_block: usize = 12;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 50276
1 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 50278
2 => {
return JS_NewBool(ctx, ret);
}
// C line 50275
3 => {
vm_block = if ((((ret) < ((0 as i32))) as i32)) != 0 { 1 } else { 2 }; continue;
}
// C line 50274
4 => {
let _ = JS_FreeAtom(ctx, atom);
vm_block = 3; continue;
}
// C line 50273
5 => {
let _ = { let assigned = JS_HasProperty(ctx, obj, atom); ret = assigned; assigned };
vm_block = 4; continue;
}
// C line 50272
6 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 50271
7 => {
vm_block = if ((((!(((!(((((atom) == ((((0 as i32)) as JSAtom))) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 6 } else { 5 }; continue;
}
// C line 50270
8 => {
let _ = { let assigned = JS_ValueToAtom(ctx, prop); atom = assigned; assigned };
vm_block = 7; continue;
}
// C line 50269
9 => {
return JS_ThrowTypeErrorNotAnObject(ctx);
}
// C line 50268
10 => {
vm_block = if (((((((obj).tag) as i32)) != ((JS_TAG_OBJECT as i32))) as i32)) != 0 { 9 } else { 8 }; continue;
}
// C line 50267
11 => {
let _ = { let assigned = *(argv).offset(((1 as i32)) as isize); prop = assigned; assigned };
vm_block = 10; continue;
}
// C line 50266
12 => {
let _ = { let assigned = *(argv).offset(((0 as i32)) as isize); obj = assigned; assigned };
vm_block = 11; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:50281. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code, unused_variables)]
unsafe fn js_reflect_set(mut ctx: *mut JSContext, mut this_val: JSValue, mut argc: i32, mut argv: *mut JSValue) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut obj: JSValue = core::mem::zeroed();
let mut prop: JSValue = core::mem::zeroed();
let mut val: JSValue = core::mem::zeroed();
let mut receiver: JSValue = core::mem::zeroed();
let mut ret: i32 = core::mem::zeroed();
let mut atom: JSAtom = core::mem::zeroed();
let mut vm_block: usize = 16;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 50304
1 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 50306
2 => {
return JS_NewBool(ctx, ret);
}
// C line 50303
3 => {
vm_block = if ((((ret) < ((0 as i32))) as i32)) != 0 { 1 } else { 2 }; continue;
}
// C line 50302
4 => {
let _ = JS_FreeAtom(ctx, atom);
vm_block = 3; continue;
}
// C line 50300
5 => {
let _ = { let assigned = JS_SetPropertyInternal(ctx, obj, atom, JS_DupValue(ctx, val), receiver, (0 as i32)); ret = assigned; assigned };
vm_block = 4; continue;
}
// C line 50299
6 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 50298
7 => {
vm_block = if ((((!(((!(((((atom) == ((((0 as i32)) as JSAtom))) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 6 } else { 5 }; continue;
}
// C line 50297
8 => {
let _ = { let assigned = JS_ValueToAtom(ctx, prop); atom = assigned; assigned };
vm_block = 7; continue;
}
// C line 50296
9 => {
return JS_ThrowTypeErrorNotAnObject(ctx);
}
// C line 50295
10 => {
vm_block = if (((((((obj).tag) as i32)) != ((JS_TAG_OBJECT as i32))) as i32)) != 0 { 9 } else { 8 }; continue;
}
// C line 50292
11 => {
let _ = { let assigned = *(argv).offset(((3 as i32)) as isize); receiver = assigned; assigned };
vm_block = 10; continue;
}
// C line 50294
12 => {
let _ = { let assigned = obj; receiver = assigned; assigned };
vm_block = 10; continue;
}
// C line 50291
13 => {
vm_block = if ((((argc) > ((3 as i32))) as i32)) != 0 { 11 } else { 12 }; continue;
}
// C line 50290
14 => {
let _ = { let assigned = *(argv).offset(((2 as i32)) as isize); val = assigned; assigned };
vm_block = 13; continue;
}
// C line 50289
15 => {
let _ = { let assigned = *(argv).offset(((1 as i32)) as isize); prop = assigned; assigned };
vm_block = 14; continue;
}
// C line 50288
16 => {
let _ = { let assigned = *(argv).offset(((0 as i32)) as isize); obj = assigned; assigned };
vm_block = 15; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:50309. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code, unused_variables)]
unsafe fn js_reflect_setPrototypeOf(mut ctx: *mut JSContext, mut this_val: JSValue, mut argc: i32, mut argv: *mut JSValue) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut ret: i32 = core::mem::zeroed();
let mut vm_block: usize = 4;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 50315
1 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 50317
2 => {
return JS_NewBool(ctx, ret);
}
// C line 50314
3 => {
vm_block = if ((((ret) < ((0 as i32))) as i32)) != 0 { 1 } else { 2 }; continue;
}
// C line 50313
4 => {
let _ = { let assigned = JS_SetPrototypeInternal(ctx, *(argv).offset(((0 as i32)) as isize), *(argv).offset(((1 as i32)) as isize), (0 as i32)); ret = assigned; assigned };
vm_block = 3; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:50320. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code, unused_variables)]
unsafe fn js_reflect_ownKeys(mut ctx: *mut JSContext, mut this_val: JSValue, mut argc: i32, mut argv: *mut JSValue) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut vm_block: usize = 3;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 50325
1 => {
return JS_GetOwnPropertyNames2(ctx, *(argv).offset(((0 as i32)) as isize), ((((1 as i32)).wrapping_shl(((0 as i32)) as u32)) | (((1 as i32)).wrapping_shl(((1 as i32)) as u32))), (JS_ITERATOR_KIND_KEY as i32));
}
// C line 50324
2 => {
return JS_ThrowTypeErrorNotAnObject(ctx);
}
// C line 50323
3 => {
vm_block = if (((((((*(argv).offset(((0 as i32)) as isize)).tag) as i32)) != ((JS_TAG_OBJECT as i32))) as i32)) != 0 { 2 } else { 1 }; continue;
}
_ => std::process::abort(),
} }
}
