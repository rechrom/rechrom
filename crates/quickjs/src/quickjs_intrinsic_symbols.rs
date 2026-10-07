// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:51341. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code, unused_variables)]
unsafe fn js_symbol_constructor(mut ctx: *mut JSContext, mut new_target: JSValue, mut argc: i32, mut argv: *mut JSValue) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut str: JSValue = core::mem::zeroed();
let mut p: *mut JSString = core::mem::zeroed();
let mut vm_block: usize = 9;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 51357
1 => {
return JS_NewSymbol(ctx, p, ((JS_ATOM_TYPE_SYMBOL as i32)) as u32);
}
// C line 51350
2 => {
let _ = { let assigned = core::ptr::null_mut::<JSString>(); p = assigned; assigned };
vm_block = 1; continue;
}
// C line 51355
3 => {
let _ = { let assigned = ((((str).u).ptr) as *mut JSString); p = assigned; assigned };
vm_block = 1; continue;
}
// C line 51354
4 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 51353
5 => {
vm_block = if (JS_IsException(str)) != 0 { 4 } else { 3 }; continue;
}
// C line 51352
6 => {
let _ = { let assigned = JS_ToString(ctx, *(argv).offset(((0 as i32)) as isize)); str = assigned; assigned };
vm_block = 5; continue;
}
// C line 51349
7 => {
vm_block = if ((((((((argc) == ((0 as i32))) as i32)) != 0) || ((JS_IsUndefined(*(argv).offset(((0 as i32)) as isize))) != 0)) as i32)) != 0 { 2 } else { 6 }; continue;
}
// C line 51348
8 => {
return JS_ThrowTypeErrorNotAConstructor(ctx, new_target);
}
// C line 51347
9 => {
vm_block = if ((!((JS_IsUndefined(new_target)) != 0) as i32)) != 0 { 8 } else { 7 }; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:51360. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code, unused_variables)]
unsafe fn js_thisSymbolValue(mut ctx: *mut JSContext, mut this_val: JSValue) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut p: *mut JSObject = core::mem::zeroed();
let mut vm_block: usize = 8;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 51372
1 => {
return JS_ThrowTypeError(ctx, c"not a symbol".as_ptr());
}
// C line 51369
2 => {
return JS_DupValue(ctx, ((*(p)).u).object_data);
}
// C line 51368
3 => {
vm_block = if (((((((((*(p)).u).object_data).tag) as i32)) == ((JS_TAG_SYMBOL as i32))) as i32)) != 0 { 2 } else { 1 }; continue;
}
// C line 51367
4 => {
vm_block = if (((((((*(p)).class_id) as i32)) == ((JS_CLASS_SYMBOL as i32))) as i32)) != 0 { 3 } else { 1 }; continue;
}
// C line 51366
5 => {
p = ((((this_val).u).ptr) as *mut JSObject);
vm_block = 4; continue;
}
// C line 51365
6 => {
vm_block = if (((((((this_val).tag) as i32)) == ((JS_TAG_OBJECT as i32))) as i32)) != 0 { 5 } else { 1 }; continue;
}
// C line 51363
7 => {
return JS_DupValue(ctx, this_val);
}
// C line 51362
8 => {
vm_block = if (((((((this_val).tag) as i32)) == ((JS_TAG_SYMBOL as i32))) as i32)) != 0 { 7 } else { 6 }; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:51375. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code, unused_variables)]
unsafe fn js_symbol_toString(mut ctx: *mut JSContext, mut this_val: JSValue, mut argc: i32, mut argv: *mut JSValue) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut val: JSValue = core::mem::zeroed();
let mut ret: JSValue = core::mem::zeroed();
let mut vm_block: usize = 6;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 51385
1 => {
return ret;
}
// C line 51384
2 => {
let _ = JS_FreeValue(ctx, val);
vm_block = 1; continue;
}
// C line 51383
3 => {
let _ = { let assigned = js_string_constructor(ctx, JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }, (1 as i32), core::ptr::addr_of_mut!(val)); ret = assigned; assigned };
vm_block = 2; continue;
}
// C line 51381
4 => {
return val;
}
// C line 51380
5 => {
vm_block = if (JS_IsException(val)) != 0 { 4 } else { 3 }; continue;
}
// C line 51379
6 => {
let _ = { let assigned = js_thisSymbolValue(ctx, this_val); val = assigned; assigned };
vm_block = 5; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:51388. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code, unused_variables)]
unsafe fn js_symbol_valueOf(mut ctx: *mut JSContext, mut this_val: JSValue, mut argc: i32, mut argv: *mut JSValue) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut vm_block: usize = 1;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 51391
1 => {
return js_thisSymbolValue(ctx, this_val);
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:51394. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code, unused_variables)]
unsafe fn js_symbol_get_description(mut ctx: *mut JSContext, mut this_val: JSValue) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut val: JSValue = core::mem::zeroed();
let mut ret: JSValue = core::mem::zeroed();
let mut p: *mut JSAtomStruct = core::mem::zeroed();
let mut vm_block: usize = 9;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 51409
1 => {
return ret;
}
// C line 51408
2 => {
let _ = JS_FreeValue(ctx, val);
vm_block = 1; continue;
}
// C line 51404
3 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; ret = assigned; assigned };
vm_block = 2; continue;
}
// C line 51406
4 => {
let _ = { let assigned = JS_AtomToString(ctx, js_get_atom_index((*(ctx)).rt, p)); ret = assigned; assigned };
vm_block = 2; continue;
}
// C line 51403
5 => {
vm_block = if (((((((((((*(p)).len()) as i32)) == ((0 as i32))) as i32)) != 0) && ((((((((*(p)).is_wide_char()) as i32)) != ((0 as i32))) as i32)) != 0)) as i32)) != 0 { 3 } else { 4 }; continue;
}
// C line 51402
6 => {
let _ = { let assigned = ((((val).u).ptr) as *mut JSAtomStruct); p = assigned; assigned };
vm_block = 5; continue;
}
// C line 51401
7 => {
return val;
}
// C line 51400
8 => {
vm_block = if (JS_IsException(val)) != 0 { 7 } else { 6 }; continue;
}
// C line 51399
9 => {
let _ = { let assigned = js_thisSymbolValue(ctx, this_val); val = assigned; assigned };
vm_block = 8; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:51421. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code, unused_variables)]
unsafe fn js_symbol_for(mut ctx: *mut JSContext, mut this_val: JSValue, mut argc: i32, mut argv: *mut JSValue) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut str: JSValue = core::mem::zeroed();
let mut vm_block: usize = 4;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 51429
1 => {
return JS_NewSymbol(ctx, ((((str).u).ptr) as *mut JSString), ((JS_ATOM_TYPE_GLOBAL_SYMBOL as i32)) as u32);
}
// C line 51428
2 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 51427
3 => {
vm_block = if (JS_IsException(str)) != 0 { 2 } else { 1 }; continue;
}
// C line 51426
4 => {
let _ = { let assigned = JS_ToString(ctx, *(argv).offset(((0 as i32)) as isize)); str = assigned; assigned };
vm_block = 3; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:51432. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code, unused_variables)]
unsafe fn js_symbol_keyFor(mut ctx: *mut JSContext, mut this_val: JSValue, mut argc: i32, mut argv: *mut JSValue) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut p: *mut JSAtomStruct = core::mem::zeroed();
let mut vm_block: usize = 6;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 51442
1 => {
return JS_DupValue(ctx, JSValue { u: JSValueUnion { ptr: ((p) as *mut c_void) }, tag: (((JS_TAG_STRING as i32)) as i64) });
}
// C line 51441
2 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) };
}
// C line 51440
3 => {
vm_block = if (((((((*(p)).atom_type()) as i32)) != ((JS_ATOM_TYPE_GLOBAL_SYMBOL as i32))) as i32)) != 0 { 2 } else { 1 }; continue;
}
// C line 51439
4 => {
let _ = { let assigned = ((((*(argv).offset(((0 as i32)) as isize)).u).ptr) as *mut JSAtomStruct); p = assigned; assigned };
vm_block = 3; continue;
}
// C line 51438
5 => {
return JS_ThrowTypeError(ctx, c"not a symbol".as_ptr());
}
// C line 51437
6 => {
vm_block = if ((!((JS_IsSymbol(*(argv).offset(((0 as i32)) as isize))) != 0) as i32)) != 0 { 5 } else { 4 }; continue;
}
_ => std::process::abort(),
} }
}
