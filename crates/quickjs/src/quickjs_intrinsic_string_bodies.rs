// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:45040. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_string_constructor(mut ctx: *mut JSContext, mut new_target: JSValue, mut argc: i32, mut argv: *mut JSValue) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut val: JSValue = core::mem::zeroed();
let mut obj: JSValue = core::mem::zeroed();
let mut p: *mut JSAtomStruct = core::mem::zeroed();
let mut p1: *mut JSString = core::mem::zeroed();
let mut vm_block: usize = 17;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 45066
1 => {
return obj;
}
// C line 45061
2 => {
let _ = JS_FreeValue(ctx, val);
vm_block = 1; continue;
}
// C line 45064
3 => {
let _ = JS_DefinePropertyValue(ctx, obj, (((crate::quickjs_atom::JS_ATOM_length as i32)) as JSAtom), JS_NewInt32(ctx, (((*(p1)).len()) as i32)), (0 as i32));
vm_block = 1; continue;
}
// C line 45063
4 => {
let _ = JS_SetObjectData(ctx, obj, val);
vm_block = 3; continue;
}
// C line 45060
5 => {
vm_block = if (JS_IsException(obj)) != 0 { 2 } else { 4 }; continue;
}
// C line 45059
6 => {
let _ = { let assigned = js_create_from_ctor(ctx, new_target, (JS_CLASS_STRING as i32)); obj = assigned; assigned };
vm_block = 5; continue;
}
// C line 45057
7 => {
p1 = ((((val).u).ptr) as *mut JSString);
vm_block = 6; continue;
}
// C line 45068
8 => {
return val;
}
// C line 45056
9 => {
vm_block = if ((!((JS_IsUndefined(new_target)) != 0) as i32)) != 0 { 7 } else { 8 }; continue;
}
// C line 45045
10 => {
let _ = { let assigned = JS_AtomToString(ctx, (((crate::quickjs_atom::JS_ATOM_empty_string as i32)) as JSAtom)); val = assigned; assigned };
vm_block = 9; continue;
}
// C line 45054
11 => {
return val;
}
// C line 45053
12 => {
vm_block = if (JS_IsException(val)) != 0 { 11 } else { 9 }; continue;
}
// C line 45049
13 => {
let _ = { let assigned = JS_ConcatString3(ctx, c"Symbol(".as_ptr(), JS_AtomToString(ctx, js_get_atom_index((*(ctx)).rt, p)), c")".as_ptr()); val = assigned; assigned };
vm_block = 12; continue;
}
// C line 45048
14 => {
p = ((((*(argv).offset(((0 as i32)) as isize)).u).ptr) as *mut JSAtomStruct);
vm_block = 13; continue;
}
// C line 45051
15 => {
let _ = { let assigned = JS_ToString(ctx, *(argv).offset(((0 as i32)) as isize)); val = assigned; assigned };
vm_block = 12; continue;
}
// C line 45047
16 => {
vm_block = if (((((JS_IsUndefined(new_target)) != 0) && ((JS_IsSymbol(*(argv).offset(((0 as i32)) as isize))) != 0)) as i32)) != 0 { 14 } else { 15 }; continue;
}
// C line 45044
17 => {
vm_block = if ((((argc) == ((0 as i32))) as i32)) != 0 { 10 } else { 16 }; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:45072. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_thisStringValue(mut ctx: *mut JSContext, mut this_val: JSValue) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut p: *mut JSObject = core::mem::zeroed();
let mut vm_block: usize = 8;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 45085
1 => {
return JS_ThrowTypeError(ctx, c"not a string".as_ptr());
}
// C line 45082
2 => {
return JS_DupValue(ctx, ((*(p)).u).object_data);
}
// C line 45081
3 => {
vm_block = if (((((((((*(p)).u).object_data).tag) as i32)) == ((JS_TAG_STRING as i32))) as i32)) != 0 { 2 } else { 1 }; continue;
}
// C line 45080
4 => {
vm_block = if (((((((*(p)).class_id) as i32)) == ((JS_CLASS_STRING as i32))) as i32)) != 0 { 3 } else { 1 }; continue;
}
// C line 45079
5 => {
p = ((((this_val).u).ptr) as *mut JSObject);
vm_block = 4; continue;
}
// C line 45078
6 => {
vm_block = if (((((((this_val).tag) as i32)) == ((JS_TAG_OBJECT as i32))) as i32)) != 0 { 5 } else { 1 }; continue;
}
// C line 45076
7 => {
return JS_DupValue(ctx, this_val);
}
// C line 45074
8 => {
vm_block = if (((((((((((this_val).tag) as i32)) == ((JS_TAG_STRING as i32))) as i32)) != 0) || ((((((((this_val).tag) as i32)) == ((JS_TAG_STRING_ROPE as i32))) as i32)) != 0)) as i32)) != 0 { 7 } else { 6 }; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:45088. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_string_fromCharCode(mut ctx: *mut JSContext, mut this_val: JSValue, mut argc: i32, mut argv: *mut JSValue) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut i: i32 = core::mem::zeroed();
let mut b_s: StringBuffer = core::mem::zeroed();
let mut b: *mut StringBuffer = core::mem::zeroed();
let mut c: i32 = core::mem::zeroed();
let mut vm_block: usize = 9;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 45103
1 => {
return string_buffer_end(b);
}
// C line 45096
2 => {
vm_block = if ((((i) < (argc)) as i32)) != 0 { 6 } else { 1 }; continue;
}
// C line ?
3 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 2; continue;
}
// C line 45100
4 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 45099
5 => {
let _ = string_buffer_free(b);
vm_block = 4; continue;
}
// C line 45098
6 => {
vm_block = if (((((JS_ToInt32(ctx, core::ptr::addr_of_mut!(c), *(argv).offset((i) as isize))) != 0) || ((string_buffer_putc16(b, ((((c) & ((65535 as i32)))) as u32))) != 0)) as i32)) != 0 { 5 } else { 3 }; continue;
}
// C line 45096
7 => {
let _ = { let assigned = (0 as i32); i = assigned; assigned };
vm_block = 2; continue;
}
// C line 45094
8 => {
let _ = string_buffer_init(ctx, b, argc);
vm_block = 7; continue;
}
// C line 45092
9 => {
b = core::ptr::addr_of_mut!(b_s);
vm_block = 8; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:45106. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_string_fromCodePoint(mut ctx: *mut JSContext, mut this_val: JSValue, mut argc: i32, mut argv: *mut JSValue) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut d: f64 = core::mem::zeroed();
let mut i: i32 = core::mem::zeroed();
let mut c: i32 = core::mem::zeroed();
let mut b_s: StringBuffer = core::mem::zeroed();
let mut b: *mut StringBuffer = core::mem::zeroed();
let mut vm_block: usize = 20;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 45137
1 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line ? labels: fail
2 => {
let _ = string_buffer_free(b);
vm_block = 1; continue;
}
// C line ? labels: range_error
3 => {
let _ = JS_ThrowRangeError(ctx, c"invalid code point".as_ptr());
vm_block = 2; continue;
}
// C line 45131
4 => {
return string_buffer_end(b);
}
// C line 45117
5 => {
vm_block = if ((((i) < (argc)) as i32)) != 0 { 16 } else { 4 }; continue;
}
// C line ?
6 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 5; continue;
}
// C line 45129
7 => {
vm_block = 2; continue;
}
// C line 45128
8 => {
vm_block = if (string_buffer_putc(b, ((c) as u32))) != 0 { 7 } else { 6 }; continue;
}
// C line 45121
9 => {
vm_block = 3; continue;
}
// C line 45120
10 => {
vm_block = if ((((((((c) < ((0 as i32))) as i32)) != 0) || (((((c) > ((1114111 as i32))) as i32)) != 0)) as i32)) != 0 { 9 } else { 8 }; continue;
}
// C line 45119
11 => {
let _ = { let assigned = ((((*(argv).offset((i) as isize)).u).uint64) as i32); c = assigned; assigned };
vm_block = 10; continue;
}
// C line 45126
12 => {
vm_block = 3; continue;
}
// C line 45125
13 => {
vm_block = if (((((((((((((if (((((size_of::<f64>() as usize)) == ((size_of::<f32>() as usize))) as i32)) != 0 { (((d) as f32)).is_nan() as i32 } else { if (((((size_of::<f64>() as usize)) == ((size_of::<f64>() as usize))) as i32)) != 0 { (d).is_nan() as i32 } else { (d).is_nan() as i32 } }) != 0) || (((((d) < ((((0 as i32)) as f64))) as i32)) != 0)) as i32)) != 0) || (((((d) > ((((1114111 as i32)) as f64))) as i32)) != 0)) as i32)) != 0) || ((((((({ let assigned = ((d) as i32); c = assigned; assigned }) as f64)) != (d)) as i32)) != 0)) as i32)) != 0 { 12 } else { 8 }; continue;
}
// C line 45124
14 => {
vm_block = 2; continue;
}
// C line 45123
15 => {
vm_block = if (JS_ToFloat64(ctx, core::ptr::addr_of_mut!(d), *(argv).offset((i) as isize))) != 0 { 14 } else { 13 }; continue;
}
// C line 45118
16 => {
vm_block = if (((((((*(argv).offset((i) as isize)).tag) as i32)) == ((JS_TAG_INT as i32))) as i32)) != 0 { 11 } else { 15 }; continue;
}
// C line 45117
17 => {
let _ = { let assigned = (0 as i32); i = assigned; assigned };
vm_block = 5; continue;
}
// C line 45116
18 => {
vm_block = 2; continue;
}
// C line 45115
19 => {
vm_block = if (string_buffer_init(ctx, b, argc)) != 0 { 18 } else { 17 }; continue;
}
// C line 45111
20 => {
b = core::ptr::addr_of_mut!(b_s);
vm_block = 19; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:45140. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_string_raw(mut ctx: *mut JSContext, mut this_val: JSValue, mut argc: i32, mut argv: *mut JSValue) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut cooked: JSValue = core::mem::zeroed();
let mut val: JSValue = core::mem::zeroed();
let mut raw: JSValue = core::mem::zeroed();
let mut b_s: StringBuffer = core::mem::zeroed();
let mut b: *mut StringBuffer = core::mem::zeroed();
let mut i: i64 = core::mem::zeroed();
let mut n: i64 = core::mem::zeroed();
let mut vm_block: usize = 28;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 45177
1 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 45176
2 => {
let _ = string_buffer_free(b);
vm_block = 1; continue;
}
// C line 45175
3 => {
let _ = JS_FreeValue(ctx, raw);
vm_block = 2; continue;
}
// C line ? labels: exception
4 => {
let _ = JS_FreeValue(ctx, cooked);
vm_block = 3; continue;
}
// C line 45171
5 => {
return string_buffer_end(b);
}
// C line 45170
6 => {
let _ = JS_FreeValue(ctx, raw);
vm_block = 5; continue;
}
// C line 45169
7 => {
let _ = JS_FreeValue(ctx, cooked);
vm_block = 6; continue;
}
// C line 45159
8 => {
vm_block = if ((((i) < (n)) as i32)) != 0 { 16 } else { 7 }; continue;
}
// C line ?
9 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 8; continue;
}
// C line 45166
10 => {
vm_block = 4; continue;
}
// C line 45165
11 => {
vm_block = if (string_buffer_concat_value(b, *(argv).offset(((i).wrapping_add((((1 as i32)) as i64))) as isize))) != 0 { 10 } else { 9 }; continue;
}
// C line 45164
12 => {
vm_block = if ((((((((i) < ((n).wrapping_sub((((1 as i32)) as i64)))) as i32)) != 0) && ((((((i).wrapping_add((((1 as i32)) as i64))) < (((argc) as i64))) as i32)) != 0)) as i32)) != 0 { 11 } else { 9 }; continue;
}
// C line 45163
13 => {
let _ = string_buffer_concat_value_free(b, val);
vm_block = 12; continue;
}
// C line 45162
14 => {
vm_block = 4; continue;
}
// C line 45161
15 => {
vm_block = if (JS_IsException(val)) != 0 { 14 } else { 13 }; continue;
}
// C line 45160
16 => {
let _ = { let assigned = JS_ToStringFree(ctx, JS_GetPropertyInt64(ctx, raw, i)); val = assigned; assigned };
vm_block = 15; continue;
}
// C line 45159
17 => {
let _ = { let assigned = (((0 as i32)) as i64); i = assigned; assigned };
vm_block = 8; continue;
}
// C line 45157
18 => {
vm_block = 4; continue;
}
// C line 45156
19 => {
vm_block = if ((((js_get_length64(ctx, core::ptr::addr_of_mut!(n), raw)) < ((0 as i32))) as i32)) != 0 { 18 } else { 17 }; continue;
}
// C line 45155
20 => {
vm_block = 4; continue;
}
// C line 45154
21 => {
vm_block = if (JS_IsException(raw)) != 0 { 20 } else { 19 }; continue;
}
// C line 45153
22 => {
let _ = { let assigned = JS_ToObjectFree(ctx, JS_GetProperty(ctx, cooked, (((crate::quickjs_atom::JS_ATOM_raw as i32)) as JSAtom))); raw = assigned; assigned };
vm_block = 21; continue;
}
// C line 45152
23 => {
vm_block = 4; continue;
}
// C line 45151
24 => {
vm_block = if (JS_IsException(cooked)) != 0 { 23 } else { 22 }; continue;
}
// C line 45150
25 => {
let _ = { let assigned = JS_ToObject(ctx, *(argv).offset(((0 as i32)) as isize)); cooked = assigned; assigned };
vm_block = 24; continue;
}
// C line 45149
26 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; raw = assigned; assigned };
vm_block = 25; continue;
}
// C line 45148
27 => {
let _ = string_buffer_init(ctx, b, (0 as i32));
vm_block = 26; continue;
}
// C line 45145
28 => {
b = core::ptr::addr_of_mut!(b_s);
vm_block = 27; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:45181. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
#[doc(hidden)]
pub unsafe fn js_string_codePointRange(mut ctx: *mut JSContext, mut this_val: JSValue, mut argc: i32, mut argv: *mut JSValue) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut start: u32 = core::mem::zeroed();
let mut end: u32 = core::mem::zeroed();
let mut i: u32 = core::mem::zeroed();
let mut n: u32 = core::mem::zeroed();
let mut b_s: StringBuffer = core::mem::zeroed();
let mut b: *mut StringBuffer = core::mem::zeroed();
let mut vm_block: usize = 16;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 45204
1 => {
return string_buffer_end(b);
}
// C line 45201
2 => {
vm_block = if ((((i) < (end)) as i32)) != 0 { 4 } else { 1 }; continue;
}
// C line ?
3 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 2; continue;
}
// C line 45202
4 => {
let _ = string_buffer_putc(b, i);
vm_block = 3; continue;
}
// C line 45201
5 => {
let _ = { let assigned = start; i = assigned; assigned };
vm_block = 2; continue;
}
// C line 45200
6 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 45199
7 => {
vm_block = if (string_buffer_init2(ctx, b, ((n) as i32), (((end) >= ((((256 as i32)) as u32))) as i32))) != 0 { 6 } else { 5 }; continue;
}
// C line 45197
8 => {
let _ = { n = (n).wrapping_add((end).wrapping_sub(crate::cutils_header::max_uint32(start, (((65536 as i32)) as u32)))); n };
vm_block = 7; continue;
}
// C line 45196
9 => {
vm_block = if ((((end) > ((((65536 as i32)) as u32))) as i32)) != 0 { 8 } else { 7 }; continue;
}
// C line 45195
10 => {
let _ = { let assigned = (end).wrapping_sub(start); n = assigned; assigned };
vm_block = 9; continue;
}
// C line 45193
11 => {
let _ = { let assigned = end; start = assigned; assigned };
vm_block = 10; continue;
}
// C line 45192
12 => {
vm_block = if ((((start) > (end)) as i32)) != 0 { 11 } else { 10 }; continue;
}
// C line 45190
13 => {
let _ = { let assigned = crate::cutils_header::min_uint32(end, ((((1114111 as i32)).wrapping_add((1 as i32))) as u32)); end = assigned; assigned };
vm_block = 12; continue;
}
// C line 45189
14 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 45187
15 => {
vm_block = if (((((JS_ToUint32(ctx, core::ptr::addr_of_mut!(start), *(argv).offset(((0 as i32)) as isize))) != 0) || ((JS_ToUint32(ctx, core::ptr::addr_of_mut!(end), *(argv).offset(((1 as i32)) as isize))) != 0)) as i32)) != 0 { 14 } else { 13 }; continue;
}
// C line 45185
16 => {
b = core::ptr::addr_of_mut!(b_s);
vm_block = 15; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:45218. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_string_charCodeAt(mut ctx: *mut JSContext, mut this_val: JSValue, mut argc: i32, mut argv: *mut JSValue) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut val: JSValue = core::mem::zeroed();
let mut ret: JSValue = core::mem::zeroed();
let mut p: *mut JSString = core::mem::zeroed();
let mut idx: i32 = core::mem::zeroed();
let mut c: i32 = core::mem::zeroed();
let mut vm_block: usize = 13;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 45240
1 => {
return ret;
}
// C line 45239
2 => {
let _ = JS_FreeValue(ctx, val);
vm_block = 1; continue;
}
// C line 45234
3 => {
let _ = { let assigned = JSValue { u: JSValueUnion { float64: ((f32::NAN) as f64) }, tag: (((JS_TAG_FLOAT64 as i32)) as i64) }; ret = assigned; assigned };
vm_block = 2; continue;
}
// C line 45237
4 => {
let _ = { let assigned = JS_NewInt32(ctx, c); ret = assigned; assigned };
vm_block = 2; continue;
}
// C line 45236
5 => {
let _ = { let assigned = string_get(p, idx); c = assigned; assigned };
vm_block = 4; continue;
}
// C line 45233
6 => {
vm_block = if ((((((((idx) < ((0 as i32))) as i32)) != 0) || (((((idx) >= ((((*(p)).len()) as i32))) as i32)) != 0)) as i32)) != 0 { 3 } else { 5 }; continue;
}
// C line 45231
7 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 45230
8 => {
let _ = JS_FreeValue(ctx, val);
vm_block = 7; continue;
}
// C line 45229
9 => {
vm_block = if (JS_ToInt32Sat(ctx, core::ptr::addr_of_mut!(idx), *(argv).offset(((0 as i32)) as isize))) != 0 { 8 } else { 6 }; continue;
}
// C line 45228
10 => {
let _ = { let assigned = ((((val).u).ptr) as *mut JSString); p = assigned; assigned };
vm_block = 9; continue;
}
// C line 45227
11 => {
return val;
}
// C line 45226
12 => {
vm_block = if (JS_IsException(val)) != 0 { 11 } else { 10 }; continue;
}
// C line 45225
13 => {
let _ = { let assigned = JS_ToStringCheckObject(ctx, this_val); val = assigned; assigned };
vm_block = 12; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:45243. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_string_charAt(mut ctx: *mut JSContext, mut this_val: JSValue, mut argc: i32, mut argv: *mut JSValue, mut is_at: i32) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut val: JSValue = core::mem::zeroed();
let mut ret: JSValue = core::mem::zeroed();
let mut p: *mut JSString = core::mem::zeroed();
let mut idx: i32 = core::mem::zeroed();
let mut c: i32 = core::mem::zeroed();
let mut vm_block: usize = 17;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 45270
1 => {
return ret;
}
// C line 45269
2 => {
let _ = JS_FreeValue(ctx, val);
vm_block = 1; continue;
}
// C line 45262
3 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; ret = assigned; assigned };
vm_block = 2; continue;
}
// C line 45264
4 => {
let _ = { let assigned = JS_AtomToString(ctx, (((crate::quickjs_atom::JS_ATOM_empty_string as i32)) as JSAtom)); ret = assigned; assigned };
vm_block = 2; continue;
}
// C line 45261
5 => {
vm_block = if (is_at) != 0 { 3 } else { 4 }; continue;
}
// C line 45267
6 => {
let _ = { let assigned = js_new_string_char(ctx, ((c) as u16)); ret = assigned; assigned };
vm_block = 2; continue;
}
// C line 45266
7 => {
let _ = { let assigned = string_get(p, idx); c = assigned; assigned };
vm_block = 6; continue;
}
// C line 45260
8 => {
vm_block = if ((((((((idx) < ((0 as i32))) as i32)) != 0) || (((((idx) >= ((((*(p)).len()) as i32))) as i32)) != 0)) as i32)) != 0 { 5 } else { 7 }; continue;
}
// C line 45259
9 => {
let _ = { idx = (idx).wrapping_add((((*(p)).len()) as i32)); idx };
vm_block = 8; continue;
}
// C line 45258
10 => {
vm_block = if ((((((((idx) < ((0 as i32))) as i32)) != 0) && ((is_at) != 0)) as i32)) != 0 { 9 } else { 8 }; continue;
}
// C line 45256
11 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 45255
12 => {
let _ = JS_FreeValue(ctx, val);
vm_block = 11; continue;
}
// C line 45254
13 => {
vm_block = if (JS_ToInt32Sat(ctx, core::ptr::addr_of_mut!(idx), *(argv).offset(((0 as i32)) as isize))) != 0 { 12 } else { 10 }; continue;
}
// C line 45253
14 => {
let _ = { let assigned = ((((val).u).ptr) as *mut JSString); p = assigned; assigned };
vm_block = 13; continue;
}
// C line 45252
15 => {
return val;
}
// C line 45251
16 => {
vm_block = if (JS_IsException(val)) != 0 { 15 } else { 14 }; continue;
}
// C line 45250
17 => {
let _ = { let assigned = JS_ToStringCheckObject(ctx, this_val); val = assigned; assigned };
vm_block = 16; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:45273. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_string_codePointAt(mut ctx: *mut JSContext, mut this_val: JSValue, mut argc: i32, mut argv: *mut JSValue) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut val: JSValue = core::mem::zeroed();
let mut ret: JSValue = core::mem::zeroed();
let mut p: *mut JSString = core::mem::zeroed();
let mut idx: i32 = core::mem::zeroed();
let mut c: i32 = core::mem::zeroed();
let mut vm_block: usize = 13;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 45295
1 => {
return ret;
}
// C line 45294
2 => {
let _ = JS_FreeValue(ctx, val);
vm_block = 1; continue;
}
// C line 45289
3 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; ret = assigned; assigned };
vm_block = 2; continue;
}
// C line 45292
4 => {
let _ = { let assigned = JS_NewInt32(ctx, c); ret = assigned; assigned };
vm_block = 2; continue;
}
// C line 45291
5 => {
let _ = { let assigned = string_getc(p, core::ptr::addr_of_mut!(idx)); c = assigned; assigned };
vm_block = 4; continue;
}
// C line 45288
6 => {
vm_block = if ((((((((idx) < ((0 as i32))) as i32)) != 0) || (((((idx) >= ((((*(p)).len()) as i32))) as i32)) != 0)) as i32)) != 0 { 3 } else { 5 }; continue;
}
// C line 45286
7 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 45285
8 => {
let _ = JS_FreeValue(ctx, val);
vm_block = 7; continue;
}
// C line 45284
9 => {
vm_block = if (JS_ToInt32Sat(ctx, core::ptr::addr_of_mut!(idx), *(argv).offset(((0 as i32)) as isize))) != 0 { 8 } else { 6 }; continue;
}
// C line 45283
10 => {
let _ = { let assigned = ((((val).u).ptr) as *mut JSString); p = assigned; assigned };
vm_block = 9; continue;
}
// C line 45282
11 => {
return val;
}
// C line 45281
12 => {
vm_block = if (JS_IsException(val)) != 0 { 11 } else { 10 }; continue;
}
// C line 45280
13 => {
let _ = { let assigned = JS_ToStringCheckObject(ctx, this_val); val = assigned; assigned };
vm_block = 12; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:45298. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_string_concat(mut ctx: *mut JSContext, mut this_val: JSValue, mut argc: i32, mut argv: *mut JSValue) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut r: JSValue = core::mem::zeroed();
let mut i: i32 = core::mem::zeroed();
let mut vm_block: usize = 8;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 45313
1 => {
return r;
}
// C line 45308
2 => {
vm_block = if ((((i) < (argc)) as i32)) != 0 { 6 } else { 1 }; continue;
}
// C line ?
3 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 2; continue;
}
// C line 45311
4 => {
let _ = { let assigned = JS_ConcatString(ctx, r, JS_DupValue(ctx, *(argv).offset((i) as isize))); r = assigned; assigned };
vm_block = 3; continue;
}
// C line 45310
5 => {
vm_block = 1; continue;
}
// C line 45309
6 => {
vm_block = if (JS_IsException(r)) != 0 { 5 } else { 4 }; continue;
}
// C line 45308
7 => {
let _ = { let assigned = (0 as i32); i = assigned; assigned };
vm_block = 2; continue;
}
// C line 45307
8 => {
let _ = { let assigned = JS_ToStringCheckObject(ctx, this_val); r = assigned; assigned };
vm_block = 7; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:45316. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn string_cmp(mut p1: *mut JSString, mut p2: *mut JSString, mut x1: i32, mut x2: i32, mut len: i32) -> i32 {
let mut vm_local_storage = Vec::<u64>::new();
let mut i: i32 = core::mem::zeroed();
let mut c1: i32 = core::mem::zeroed();
let mut c2: i32 = core::mem::zeroed();
let mut vm_block: usize = 6;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 45323
1 => {
return (0 as i32);
}
// C line 45319
2 => {
vm_block = if ((((i) < (len)) as i32)) != 0 { 5 } else { 1 }; continue;
}
// C line ?
3 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 2; continue;
}
// C line 45321
4 => {
return (c1).wrapping_sub(c2);
}
// C line 45320
5 => {
vm_block = if (((({ let assigned = string_get(p1, (x1).wrapping_add(i)); c1 = assigned; assigned }) != ({ let assigned = string_get(p2, (x2).wrapping_add(i)); c2 = assigned; assigned })) as i32)) != 0 { 4 } else { 3 }; continue;
}
// C line 45319
6 => {
let _ = { let assigned = (0 as i32); i = assigned; assigned };
vm_block = 2; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:45326. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn string_indexof_char(mut p: *mut JSString, mut c: i32, mut from: i32) -> i32 {
let mut vm_local_storage = Vec::<u64>::new();
let mut i: i32 = core::mem::zeroed();
let mut len: i32 = core::mem::zeroed();
let mut vm_block: usize = 14;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 45343
1 => {
return ((1 as i32)).wrapping_neg();
}
// C line 45331
2 => {
vm_block = if ((((i) < (len)) as i32)) != 0 { 5 } else { 1 }; continue;
}
// C line ?
3 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 2; continue;
}
// C line 45333
4 => {
return i;
}
// C line 45332
5 => {
vm_block = if ((((((*((((*(p)).u).str16).as_mut_ptr()).offset((i) as isize)) as i32)) == (c)) as i32)) != 0 { 4 } else { 3 }; continue;
}
// C line 45331
6 => {
let _ = { let assigned = from; i = assigned; assigned };
vm_block = 2; continue;
}
// C line 45337
7 => {
vm_block = if ((((i) < (len)) as i32)) != 0 { 10 } else { 1 }; continue;
}
// C line ?
8 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 7; continue;
}
// C line 45339
9 => {
return i;
}
// C line 45338
10 => {
vm_block = if ((((((*((((*(p)).u).str8).as_mut_ptr()).offset((i) as isize)) as i32)) == (((((c) as u8)) as i32))) as i32)) != 0 { 9 } else { 8 }; continue;
}
// C line 45337
11 => {
let _ = { let assigned = from; i = assigned; assigned };
vm_block = 7; continue;
}
// C line 45336
12 => {
vm_block = if ((((((c) & ((!((255 as i32)))))) == ((0 as i32))) as i32)) != 0 { 11 } else { 1 }; continue;
}
// C line 45330
13 => {
vm_block = if ((*(p)).is_wide_char()) != 0 { 6 } else { 12 }; continue;
}
// C line 45329
14 => {
len = (((*(p)).len()) as i32);
vm_block = 13; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:45346. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn string_indexof(mut p1: *mut JSString, mut p2: *mut JSString, mut from: i32) -> i32 {
let mut vm_local_storage = Vec::<u64>::new();
let mut c: i32 = core::mem::zeroed();
let mut i: i32 = core::mem::zeroed();
let mut j: i32 = core::mem::zeroed();
let mut len1: i32 = core::mem::zeroed();
let mut len2: i32 = core::mem::zeroed();
let mut vm_block: usize = 12;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 45359
1 => {
return ((1 as i32)).wrapping_neg();
}
// C line 45352
2 => {
vm_block = if (((((i).wrapping_add(len2)) <= (len1)) as i32)) != 0 { 8 } else { 1 }; continue;
}
// C line ?
3 => {
let _ = { let assigned = (j).wrapping_add((1 as i32)); i = assigned; assigned };
vm_block = 2; continue;
}
// C line 45357
4 => {
return j;
}
// C line 45356
5 => {
vm_block = if ((!((string_cmp(p1, p2, (j).wrapping_add((1 as i32)), (1 as i32), (len2).wrapping_sub((1 as i32)))) != 0) as i32)) != 0 { 4 } else { 3 }; continue;
}
// C line 45355
6 => {
vm_block = 1; continue;
}
// C line 45354
7 => {
vm_block = if ((((((((j) < ((0 as i32))) as i32)) != 0) || ((((((j).wrapping_add(len2)) > (len1)) as i32)) != 0)) as i32)) != 0 { 6 } else { 5 }; continue;
}
// C line 45353
8 => {
let _ = { let assigned = string_indexof_char(p1, c, i); j = assigned; assigned };
vm_block = 7; continue;
}
// C line 45352
9 => {
let _ = { let _ = { let assigned = from; i = assigned; assigned }; { let assigned = string_get(p2, (0 as i32)); c = assigned; assigned } };
vm_block = 2; continue;
}
// C line 45351
10 => {
return from;
}
// C line 45350
11 => {
vm_block = if ((((len2) == ((0 as i32))) as i32)) != 0 { 10 } else { 9 }; continue;
}
// C line 45349
12 => {
len1 = (((*(p1)).len()) as i32);
len2 = (((*(p2)).len()) as i32);
vm_block = 11; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:45362. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn string_advance_index(mut p: *mut JSString, mut index: i64, mut unicode: i32) -> i64 {
let mut vm_local_storage = Vec::<u64>::new();
let mut index32: i32 = core::mem::zeroed();
let mut vm_block: usize = 6;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 45371
1 => {
return index;
}
// C line 45365
2 => {
let _ = { let old = index; index = (index).wrapping_add(1); old };
vm_block = 1; continue;
}
// C line 45369
3 => {
let _ = { let assigned = ((index32) as i64); index = assigned; assigned };
vm_block = 1; continue;
}
// C line 45368
4 => {
let _ = string_getc(p, core::ptr::addr_of_mut!(index32));
vm_block = 3; continue;
}
// C line 45367
5 => {
index32 = ((index) as i32);
vm_block = 4; continue;
}
// C line 45364
6 => {
vm_block = if ((((((((((!((unicode) != 0) as i32)) != 0) || (((((index) >= ((((*(p)).len()) as i64))) as i32)) != 0)) as i32)) != 0) || (((!(((*(p)).is_wide_char()) != 0) as i32)) != 0)) as i32)) != 0 { 2 } else { 5 }; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:45395. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_string_isWellFormed(mut ctx: *mut JSContext, mut this_val: JSValue, mut argc: i32, mut argv: *mut JSValue) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut str: JSValue = core::mem::zeroed();
let mut p: *mut JSString = core::mem::zeroed();
let mut ret: i32 = core::mem::zeroed();
let mut vm_block: usize = 7;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 45408
1 => {
return JS_NewBool(ctx, ret);
}
// C line 45407
2 => {
let _ = JS_FreeValue(ctx, str);
vm_block = 1; continue;
}
// C line 45406
3 => {
let _ = { let assigned = (((js_string_find_invalid_codepoint(p)) < ((0 as i32))) as i32); ret = assigned; assigned };
vm_block = 2; continue;
}
// C line 45405
4 => {
let _ = { let assigned = ((((str).u).ptr) as *mut JSString); p = assigned; assigned };
vm_block = 3; continue;
}
// C line 45404
5 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 45403
6 => {
vm_block = if (JS_IsException(str)) != 0 { 5 } else { 4 }; continue;
}
// C line 45402
7 => {
let _ = { let assigned = JS_ToStringCheckObject(ctx, this_val); str = assigned; assigned };
vm_block = 6; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:45411. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_string_toWellFormed(mut ctx: *mut JSContext, mut this_val: JSValue, mut argc: i32, mut argv: *mut JSValue) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut str: JSValue = core::mem::zeroed();
let mut ret: JSValue = core::mem::zeroed();
let mut p: *mut JSString = core::mem::zeroed();
let mut i: i32 = core::mem::zeroed();
let mut c: u32 = core::mem::zeroed();
let mut vm_block: usize = 20;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 45445
1 => {
return ret;
}
// C line 45434
2 => {
vm_block = if ((((i) < ((((*(p)).len()) as i32))) as i32)) != 0 { 8 } else { 1 }; continue;
}
// C line ?
3 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 2; continue;
}
// C line 45439
4 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 3; continue;
}
// C line 45441
5 => {
let _ = { let assigned = (((65533 as i32)) as u16); *((((*(p)).u).str16).as_mut_ptr()).offset((i) as isize) = assigned; assigned };
vm_block = 3; continue;
}
// C line 45437
6 => {
vm_block = if (((((((((is_hi_surrogate(c)) != 0) && ((((((i).wrapping_add((1 as i32))) < ((((*(p)).len()) as i32))) as i32)) != 0)) as i32)) != 0) && ((is_lo_surrogate(((*((((*(p)).u).str16).as_mut_ptr()).offset(((i).wrapping_add((1 as i32))) as isize)) as u32))) != 0)) as i32)) != 0 { 4 } else { 5 }; continue;
}
// C line 45436
7 => {
vm_block = if (crate::cutils_header::is_surrogate(c)) != 0 { 6 } else { 3 }; continue;
}
// C line 45435
8 => {
c = ((*((((*(p)).u).str16).as_mut_ptr()).offset((i) as isize)) as u32);
vm_block = 7; continue;
}
// C line 45433
9 => {
let _ = { let assigned = ((((ret).u).ptr) as *mut JSString); p = assigned; assigned };
vm_block = 2; continue;
}
// C line 45431
10 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 45430
11 => {
vm_block = if (JS_IsException(ret)) != 0 { 10 } else { 9 }; continue;
}
// C line 45429
12 => {
let _ = JS_FreeValue(ctx, str);
vm_block = 11; continue;
}
// C line 45428
13 => {
let _ = { let assigned = js_new_string16_len(ctx, (((*(p)).u).str16).as_mut_ptr(), (((*(p)).len()) as i32)); ret = assigned; assigned };
vm_block = 12; continue;
}
// C line 45426
14 => {
return str;
}
// C line 45425
15 => {
vm_block = if ((((i) < ((0 as i32))) as i32)) != 0 { 14 } else { 13 }; continue;
}
// C line 45424
16 => {
let _ = { let assigned = js_string_find_invalid_codepoint(p); i = assigned; assigned };
vm_block = 15; continue;
}
// C line 45422
17 => {
let _ = { let assigned = ((((str).u).ptr) as *mut JSString); p = assigned; assigned };
vm_block = 16; continue;
}
// C line 45420
18 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 45419
19 => {
vm_block = if (JS_IsException(str)) != 0 { 18 } else { 17 }; continue;
}
// C line 45418
20 => {
let _ = { let assigned = JS_ToStringCheckObject(ctx, this_val); str = assigned; assigned };
vm_block = 19; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:45448. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_string_indexOf(mut ctx: *mut JSContext, mut this_val: JSValue, mut argc: i32, mut argv: *mut JSValue, mut lastIndexOf: i32) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut str: JSValue = core::mem::zeroed();
let mut v: JSValue = core::mem::zeroed();
let mut i: i32 = core::mem::zeroed();
let mut len: i32 = core::mem::zeroed();
let mut v_len: i32 = core::mem::zeroed();
let mut pos: i32 = core::mem::zeroed();
let mut start: i32 = core::mem::zeroed();
let mut stop: i32 = core::mem::zeroed();
let mut ret: i32 = core::mem::zeroed();
let mut inc: i32 = core::mem::zeroed();
let mut p: *mut JSString = core::mem::zeroed();
let mut p1: *mut JSString = core::mem::zeroed();
let mut d: f64 = core::mem::zeroed();
let mut vm_block: usize = 46;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 45510
1 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 45509
2 => {
let _ = JS_FreeValue(ctx, v);
vm_block = 1; continue;
}
// C line ? labels: fail
3 => {
let _ = JS_FreeValue(ctx, str);
vm_block = 2; continue;
}
// C line 45505
4 => {
return JS_NewInt32(ctx, ret);
}
// C line 45504
5 => {
let _ = JS_FreeValue(ctx, v);
vm_block = 4; continue;
}
// C line 45503
6 => {
let _ = JS_FreeValue(ctx, str);
vm_block = 5; continue;
}
// C line 45494
7 => {
vm_block = 13; continue;
}
// C line ?
8 => {
let _ = { i = (i).wrapping_add(inc); i };
vm_block = 7; continue;
}
// C line 45500
9 => {
vm_block = 6; continue;
}
// C line 45499
10 => {
vm_block = if ((((i) == (stop)) as i32)) != 0 { 9 } else { 8 }; continue;
}
// C line 45497
11 => {
vm_block = 6; continue;
}
// C line 45496
12 => {
let _ = { let assigned = i; ret = assigned; assigned };
vm_block = 11; continue;
}
// C line 45495
13 => {
vm_block = if ((!((string_cmp(p, p1, i, (0 as i32), v_len)) != 0) as i32)) != 0 { 12 } else { 10 }; continue;
}
// C line 45494
14 => {
let _ = { let assigned = start; i = assigned; assigned };
vm_block = 7; continue;
}
// C line 45493
15 => {
vm_block = if ((((((((len) >= (v_len)) as i32)) != 0) && ((((((inc).wrapping_mul((stop).wrapping_sub(start))) >= ((0 as i32))) as i32)) != 0)) as i32)) != 0 { 14 } else { 6 }; continue;
}
// C line 45492
16 => {
let _ = { let assigned = ((1 as i32)).wrapping_neg(); ret = assigned; assigned };
vm_block = 15; continue;
}
// C line 45481
17 => {
let _ = { let assigned = ((1 as i32)).wrapping_neg(); inc = assigned; assigned };
vm_block = 16; continue;
}
// C line 45480
18 => {
let _ = { let assigned = (0 as i32); stop = assigned; assigned };
vm_block = 17; continue;
}
// C line 45479
19 => {
let _ = { let assigned = pos; start = assigned; assigned };
vm_block = 18; continue;
}
// C line 45474
20 => {
let _ = { let assigned = (0 as i32); pos = assigned; assigned };
vm_block = 19; continue;
}
// C line 45476
21 => {
let _ = { let assigned = ((d) as i32); pos = assigned; assigned };
vm_block = 19; continue;
}
// C line 45475
22 => {
vm_block = if ((((d) < (((pos) as f64))) as i32)) != 0 { 21 } else { 19 }; continue;
}
// C line 45473
23 => {
vm_block = if ((((d) <= ((((0 as i32)) as f64))) as i32)) != 0 { 20 } else { 22 }; continue;
}
// C line 45472
24 => {
vm_block = if ((!((if (((((size_of::<f64>() as usize)) == ((size_of::<f32>() as usize))) as i32)) != 0 { (((d) as f32)).is_nan() as i32 } else { if (((((size_of::<f64>() as usize)) == ((size_of::<f64>() as usize))) as i32)) != 0 { (d).is_nan() as i32 } else { (d).is_nan() as i32 } }) != 0) as i32)) != 0 { 23 } else { 19 }; continue;
}
// C line 45471
25 => {
vm_block = 3; continue;
}
// C line 45470
26 => {
vm_block = if (JS_ToFloat64(ctx, core::ptr::addr_of_mut!(d), *(argv).offset(((1 as i32)) as isize))) != 0 { 25 } else { 24 }; continue;
}
// C line 45468
27 => {
vm_block = if ((((argc) > ((1 as i32))) as i32)) != 0 { 26 } else { 19 }; continue;
}
// C line 45467
28 => {
let _ = { let assigned = (len).wrapping_sub(v_len); pos = assigned; assigned };
vm_block = 27; continue;
}
// C line 45490
29 => {
let _ = { let assigned = (1 as i32); inc = assigned; assigned };
vm_block = 16; continue;
}
// C line 45489
30 => {
let _ = { let assigned = (len).wrapping_sub(v_len); stop = assigned; assigned };
vm_block = 29; continue;
}
// C line 45488
31 => {
let _ = { let assigned = pos; start = assigned; assigned };
vm_block = 30; continue;
}
// C line 45486
32 => {
vm_block = 3; continue;
}
// C line 45485
33 => {
vm_block = if (JS_ToInt32Clamp(ctx, core::ptr::addr_of_mut!(pos), *(argv).offset(((1 as i32)) as isize), (0 as i32), len, (0 as i32))) != 0 { 32 } else { 31 }; continue;
}
// C line 45484
34 => {
vm_block = if ((((argc) > ((1 as i32))) as i32)) != 0 { 33 } else { 31 }; continue;
}
// C line 45483
35 => {
let _ = { let assigned = (0 as i32); pos = assigned; assigned };
vm_block = 34; continue;
}
// C line 45466
36 => {
vm_block = if (lastIndexOf) != 0 { 28 } else { 35 }; continue;
}
// C line 45465
37 => {
let _ = { let assigned = (((*(p1)).len()) as i32); v_len = assigned; assigned };
vm_block = 36; continue;
}
// C line 45464
38 => {
let _ = { let assigned = (((*(p)).len()) as i32); len = assigned; assigned };
vm_block = 37; continue;
}
// C line 45463
39 => {
let _ = { let assigned = ((((v).u).ptr) as *mut JSString); p1 = assigned; assigned };
vm_block = 38; continue;
}
// C line 45462
40 => {
let _ = { let assigned = ((((str).u).ptr) as *mut JSString); p = assigned; assigned };
vm_block = 39; continue;
}
// C line 45461
41 => {
vm_block = 3; continue;
}
// C line 45460
42 => {
vm_block = if (JS_IsException(v)) != 0 { 41 } else { 40 }; continue;
}
// C line 45459
43 => {
let _ = { let assigned = JS_ToString(ctx, *(argv).offset(((0 as i32)) as isize)); v = assigned; assigned };
vm_block = 42; continue;
}
// C line 45458
44 => {
return str;
}
// C line 45457
45 => {
vm_block = if (JS_IsException(str)) != 0 { 44 } else { 43 }; continue;
}
// C line 45456
46 => {
let _ = { let assigned = JS_ToStringCheckObject(ctx, this_val); str = assigned; assigned };
vm_block = 45; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:45516. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_string_includes(mut ctx: *mut JSContext, mut this_val: JSValue, mut argc: i32, mut argv: *mut JSValue, mut magic: i32) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut str: JSValue = core::mem::zeroed();
let mut v: JSValue = core::mem::zeroed();
let mut i: i32 = core::mem::zeroed();
let mut len: i32 = core::mem::zeroed();
let mut v_len: i32 = core::mem::zeroed();
let mut pos: i32 = core::mem::zeroed();
let mut start: i32 = core::mem::zeroed();
let mut stop: i32 = core::mem::zeroed();
let mut ret: i32 = core::mem::zeroed();
let mut p: *mut JSString = core::mem::zeroed();
let mut p1: *mut JSString = core::mem::zeroed();
let mut vm_block: usize = 45;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 45577
1 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 45576
2 => {
let _ = JS_FreeValue(ctx, v);
vm_block = 1; continue;
}
// C line ? labels: fail
3 => {
let _ = JS_FreeValue(ctx, str);
vm_block = 2; continue;
}
// C line 45572
4 => {
return JS_NewBool(ctx, ret);
}
// C line 45571
5 => {
let _ = JS_FreeValue(ctx, v);
vm_block = 4; continue;
}
// C line ? labels: done
6 => {
let _ = JS_FreeValue(ctx, str);
vm_block = 5; continue;
}
// C line 45560
7 => {
vm_block = 13; continue;
}
// C line ?
8 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 7; continue;
}
// C line 45566
9 => {
vm_block = 6; continue;
}
// C line 45565
10 => {
vm_block = if ((((i) == (stop)) as i32)) != 0 { 9 } else { 8 }; continue;
}
// C line 45563
11 => {
vm_block = 6; continue;
}
// C line 45562
12 => {
let _ = { let assigned = (1 as i32); ret = assigned; assigned };
vm_block = 11; continue;
}
// C line 45561
13 => {
vm_block = if ((!((string_cmp(p, p1, i, (0 as i32), v_len)) != 0) as i32)) != 0 { 12 } else { 10 }; continue;
}
// C line 45560
14 => {
let _ = { let assigned = start; i = assigned; assigned };
vm_block = 7; continue;
}
// C line 45559
15 => {
vm_block = if ((((((((start) >= ((0 as i32))) as i32)) != 0) && (((((start) <= (stop)) as i32)) != 0)) as i32)) != 0 { 14 } else { 6 }; continue;
}
// C line 45549
16 => {
let _ = { let assigned = len; stop = assigned; assigned };
vm_block = 15; continue;
}
// C line 45548
17 => {
let _ = { let assigned = pos; start = assigned; assigned };
vm_block = 16; continue;
}
// C line 45557
18 => {
let _ = { let assigned = { let assigned = pos; stop = assigned; assigned }; start = assigned; assigned };
vm_block = 15; continue;
}
// C line 45553
19 => {
vm_block = 6; continue;
}
// C line 45552
20 => {
vm_block = if ((((pos) > (len)) as i32)) != 0 { 19 } else { 18 }; continue;
}
// C line 45555
21 => {
let _ = { pos = (pos).wrapping_sub(v_len); pos };
vm_block = 18; continue;
}
// C line 45551
22 => {
vm_block = if ((((magic) == ((1 as i32))) as i32)) != 0 { 20 } else { 21 }; continue;
}
// C line 45547
23 => {
vm_block = if ((((magic) == ((0 as i32))) as i32)) != 0 { 17 } else { 22 }; continue;
}
// C line 45546
24 => {
let _ = { let assigned = (0 as i32); ret = assigned; assigned };
vm_block = 23; continue;
}
// C line 45545
25 => {
let _ = { len = (len).wrapping_sub(v_len); len };
vm_block = 24; continue;
}
// C line 45543
26 => {
vm_block = 3; continue;
}
// C line 45542
27 => {
vm_block = if (JS_ToInt32Clamp(ctx, core::ptr::addr_of_mut!(pos), *(argv).offset(((1 as i32)) as isize), (0 as i32), len, (0 as i32))) != 0 { 26 } else { 25 }; continue;
}
// C line 45541
28 => {
vm_block = if ((((((((argc) > ((1 as i32))) as i32)) != 0) && (((!((JS_IsUndefined(*(argv).offset(((1 as i32)) as isize))) != 0) as i32)) != 0)) as i32)) != 0 { 27 } else { 25 }; continue;
}
// C line 45540
29 => {
let _ = { let assigned = if ((((magic) == ((2 as i32))) as i32)) != 0 { len } else { (0 as i32) }; pos = assigned; assigned };
vm_block = 28; continue;
}
// C line 45539
30 => {
let _ = { let assigned = (((*(p1)).len()) as i32); v_len = assigned; assigned };
vm_block = 29; continue;
}
// C line 45538
31 => {
let _ = { let assigned = (((*(p)).len()) as i32); len = assigned; assigned };
vm_block = 30; continue;
}
// C line 45537
32 => {
let _ = { let assigned = ((((v).u).ptr) as *mut JSString); p1 = assigned; assigned };
vm_block = 31; continue;
}
// C line 45536
33 => {
let _ = { let assigned = ((((str).u).ptr) as *mut JSString); p = assigned; assigned };
vm_block = 32; continue;
}
// C line 45535
34 => {
vm_block = 3; continue;
}
// C line 45534
35 => {
vm_block = if (JS_IsException(v)) != 0 { 34 } else { 33 }; continue;
}
// C line 45533
36 => {
let _ = { let assigned = JS_ToString(ctx, *(argv).offset(((0 as i32)) as isize)); v = assigned; assigned };
vm_block = 35; continue;
}
// C line 45531
37 => {
vm_block = 3; continue;
}
// C line 45530
38 => {
let _ = JS_ThrowTypeError(ctx, c"regexp not supported".as_ptr());
vm_block = 37; continue;
}
// C line 45529
39 => {
vm_block = if ((((ret) > ((0 as i32))) as i32)) != 0 { 38 } else { 37 }; continue;
}
// C line 45528
40 => {
vm_block = if (ret) != 0 { 39 } else { 36 }; continue;
}
// C line 45527
41 => {
let _ = { let assigned = js_is_regexp(ctx, *(argv).offset(((0 as i32)) as isize)); ret = assigned; assigned };
vm_block = 40; continue;
}
// C line 45526
42 => {
return str;
}
// C line 45525
43 => {
vm_block = if (JS_IsException(str)) != 0 { 42 } else { 41 }; continue;
}
// C line 45524
44 => {
let _ = { let assigned = JS_ToStringCheckObject(ctx, this_val); str = assigned; assigned };
vm_block = 43; continue;
}
// C line 45519
45 => {
v = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) };
vm_block = 44; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:45580. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn check_regexp_g_flag(mut ctx: *mut JSContext, mut regexp: JSValue) -> i32 {
let mut vm_local_storage = Vec::<u64>::new();
let mut ret: i32 = core::mem::zeroed();
let mut flags: JSValue = core::mem::zeroed();
let mut vm_block: usize = 19;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 45606
1 => {
return (0 as i32);
}
// C line 45603
2 => {
return ((1 as i32)).wrapping_neg();
}
// C line 45602
3 => {
let _ = JS_ThrowTypeError(ctx, c"regexp must have the 'g' flag".as_ptr());
vm_block = 2; continue;
}
// C line 45601
4 => {
vm_block = if ((((ret) < ((0 as i32))) as i32)) != 0 { 3 } else { 1 }; continue;
}
// C line 45600
5 => {
let _ = JS_FreeValue(ctx, flags);
vm_block = 4; continue;
}
// C line 45599
6 => {
let _ = { let assigned = string_indexof_char(((((flags).u).ptr) as *mut JSString), (103 as i32), (0 as i32)); ret = assigned; assigned };
vm_block = 5; continue;
}
// C line 45598
7 => {
return ((1 as i32)).wrapping_neg();
}
// C line 45597
8 => {
vm_block = if (JS_IsException(flags)) != 0 { 7 } else { 6 }; continue;
}
// C line 45596
9 => {
let _ = { let assigned = JS_ToStringFree(ctx, flags); flags = assigned; assigned };
vm_block = 8; continue;
}
// C line 45594
10 => {
return ((1 as i32)).wrapping_neg();
}
// C line 45593
11 => {
let _ = JS_ThrowTypeError(ctx, c"cannot convert to object".as_ptr());
vm_block = 10; continue;
}
// C line 45592
12 => {
vm_block = if (((((JS_IsUndefined(flags)) != 0) || ((JS_IsNull(flags)) != 0)) as i32)) != 0 { 11 } else { 9 }; continue;
}
// C line 45591
13 => {
return ((1 as i32)).wrapping_neg();
}
// C line 45590
14 => {
vm_block = if (JS_IsException(flags)) != 0 { 13 } else { 12 }; continue;
}
// C line 45589
15 => {
let _ = { let assigned = JS_GetProperty(ctx, regexp, (((crate::quickjs_atom::JS_ATOM_flags as i32)) as JSAtom)); flags = assigned; assigned };
vm_block = 14; continue;
}
// C line 45588
16 => {
vm_block = if (ret) != 0 { 15 } else { 1 }; continue;
}
// C line 45587
17 => {
return ((1 as i32)).wrapping_neg();
}
// C line 45586
18 => {
vm_block = if ((((ret) < ((0 as i32))) as i32)) != 0 { 17 } else { 16 }; continue;
}
// C line 45585
19 => {
let _ = { let assigned = js_is_regexp(ctx, regexp); ret = assigned; assigned };
vm_block = 18; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:45609. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_string_match(mut ctx: *mut JSContext, mut this_val: JSValue, mut argc: i32, mut argv: *mut JSValue, mut atom: i32) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut O: JSValue = core::mem::zeroed();
let mut regexp: JSValue = core::mem::zeroed();
let mut args: [JSValue; 2] = core::mem::zeroed();
let mut matcher: JSValue = core::mem::zeroed();
let mut S: JSValue = core::mem::zeroed();
let mut rx: JSValue = core::mem::zeroed();
let mut result: JSValue = core::mem::zeroed();
let mut str: JSValue = core::mem::zeroed();
let mut args_len: i32 = core::mem::zeroed();
let mut vm_block: usize = 32;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 45656
1 => {
return result;
}
// C line 45655
2 => {
let _ = JS_FreeValue(ctx, S);
vm_block = 1; continue;
}
// C line 45654
3 => {
let _ = { let assigned = JS_InvokeFree(ctx, rx, ((atom) as JSAtom), (1 as i32), core::ptr::addr_of_mut!(S)); result = assigned; assigned };
vm_block = 2; continue;
}
// C line 45652
4 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line ? labels: fail
5 => {
let _ = JS_FreeValue(ctx, S);
vm_block = 4; continue;
}
// C line 45649
6 => {
vm_block = if (JS_IsException(rx)) != 0 { 5 } else { 3 }; continue;
}
// C line 45648
7 => {
let _ = JS_FreeValue(ctx, str);
vm_block = 6; continue;
}
// C line 45647
8 => {
let _ = { let assigned = JS_CallConstructor(ctx, (*(ctx)).regexp_ctor, args_len, (args).as_mut_ptr()); rx = assigned; assigned };
vm_block = 7; continue;
}
// C line 45645
9 => {
let _ = { let assigned = str; *((args).as_mut_ptr()).offset(({ let old = args_len; args_len = (args_len).wrapping_add(1); old }) as isize) = assigned; assigned };
vm_block = 8; continue;
}
// C line 45644
10 => {
vm_block = 5; continue;
}
// C line 45643
11 => {
vm_block = if (JS_IsException(str)) != 0 { 10 } else { 9 }; continue;
}
// C line 45642
12 => {
let _ = { let assigned = js_new_string8(ctx, c"g".as_ptr()); str = assigned; assigned };
vm_block = 11; continue;
}
// C line 45641
13 => {
vm_block = if ((((atom) == ((crate::quickjs_atom::JS_ATOM_Symbol_matchAll as i32))) as i32)) != 0 { 12 } else { 8 }; continue;
}
// C line 45640
14 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; str = assigned; assigned };
vm_block = 13; continue;
}
// C line 45639
15 => {
let _ = { let assigned = regexp; *((args).as_mut_ptr()).offset(((0 as i32)) as isize) = assigned; assigned };
vm_block = 14; continue;
}
// C line 45638
16 => {
let _ = { let assigned = (1 as i32); args_len = assigned; assigned };
vm_block = 15; continue;
}
// C line 45637
17 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 45636
18 => {
vm_block = if (JS_IsException(S)) != 0 { 17 } else { 16 }; continue;
}
// C line 45635
19 => {
let _ = { let assigned = JS_ToString(ctx, O); S = assigned; assigned };
vm_block = 18; continue;
}
// C line 45632
20 => {
return JS_CallFree(ctx, matcher, regexp, (1 as i32), core::ptr::addr_of_mut!(O));
}
// C line 45631
21 => {
vm_block = if ((((((!((JS_IsUndefined(matcher)) != 0) as i32)) != 0) && (((!((JS_IsNull(matcher)) != 0) as i32)) != 0)) as i32)) != 0 { 20 } else { 19 }; continue;
}
// C line 45628
22 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 45627
23 => {
let _ = JS_FreeValue(ctx, matcher);
vm_block = 22; continue;
}
// C line 45626
24 => {
vm_block = if ((((check_regexp_g_flag(ctx, regexp)) < ((0 as i32))) as i32)) != 0 { 23 } else { 21 }; continue;
}
// C line 45625
25 => {
vm_block = if ((((atom) == ((crate::quickjs_atom::JS_ATOM_Symbol_matchAll as i32))) as i32)) != 0 { 24 } else { 21 }; continue;
}
// C line 45624
26 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 45623
27 => {
vm_block = if (JS_IsException(matcher)) != 0 { 26 } else { 25 }; continue;
}
// C line 45622
28 => {
let _ = { let assigned = JS_GetProperty(ctx, regexp, ((atom) as JSAtom)); matcher = assigned; assigned };
vm_block = 27; continue;
}
// C line 45621
29 => {
vm_block = if (JS_IsObject(regexp)) != 0 { 28 } else { 19 }; continue;
}
// C line 45619
30 => {
return JS_ThrowTypeError(ctx, c"cannot convert to object".as_ptr());
}
// C line 45618
31 => {
vm_block = if (((((JS_IsUndefined(O)) != 0) || ((JS_IsNull(O)) != 0)) as i32)) != 0 { 30 } else { 29 }; continue;
}
// C line 45614
32 => {
O = this_val;
regexp = *(argv).offset(((0 as i32)) as isize);
vm_block = 31; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:45661. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_string_GetSubstitution(mut ctx: *mut JSContext, mut b: *mut StringBuffer, mut matched: JSValue, mut sp: *mut JSString, mut position: u32, mut captures_val: JSValue, mut namedCaptures: JSValue, mut rep: JSValue, mut captures: *mut *mut u8, mut captures_len: u32) -> i32 {
let mut vm_local_storage = Vec::<u64>::new();
let mut capture: JSValue = core::mem::zeroed();
let mut name: JSValue = core::mem::zeroed();
let mut s: JSValue = core::mem::zeroed();
let mut len: u32 = core::mem::zeroed();
let mut matched_len: u32 = core::mem::zeroed();
let mut i: i32 = core::mem::zeroed();
let mut j: i32 = core::mem::zeroed();
let mut j0: i32 = core::mem::zeroed();
let mut k: i32 = core::mem::zeroed();
let mut k1: i32 = core::mem::zeroed();
let mut shift: i32 = core::mem::zeroed();
let mut c: i32 = core::mem::zeroed();
let mut c1: i32 = core::mem::zeroed();
let mut rp: *mut JSString = core::mem::zeroed();
let mut start: i32 = core::mem::zeroed();
let mut end: i32 = core::mem::zeroed();
let mut vm_block: usize = 74;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line ? labels: exception
1 => {
return ((1 as i32)).wrapping_neg();
}
// C line 45776
2 => {
return (0 as i32);
}
// C line 45775
3 => {
let _ = string_buffer_concat(b, rp, ((i) as u32), (*(rp)).len());
vm_block = 2; continue;
}
// C line 45699
4 => {
vm_block = 59; continue;
}
// C line 45773
5 => {
let _ = { let assigned = j; i = assigned; assigned };
vm_block = 4; continue;
}
// C line 45707
6 => {
let _ = string_buffer_putc8(b, (((36 as i32)) as u32));
vm_block = 5; continue;
}
// C line 45710
7 => {
let _ = string_buffer_concat(b, sp, position, (position).wrapping_add(matched_len));
vm_block = 5; continue;
}
// C line 45713
8 => {
vm_block = 1; continue;
}
// C line 45712
9 => {
vm_block = if (string_buffer_concat_value(b, matched)) != 0 { 8 } else { 5 }; continue;
}
// C line 45709
10 => {
vm_block = if !(captures).is_null() { 7 } else { 9 }; continue;
}
// C line 45716
11 => {
let _ = string_buffer_concat(b, sp, (((0 as i32)) as u32), position);
vm_block = 5; continue;
}
// C line 45718
12 => {
let _ = string_buffer_concat(b, sp, (position).wrapping_add(matched_len), (*(sp)).len());
vm_block = 5; continue;
}
// C line 45740
13 => {
let _ = string_buffer_concat(b, sp, ((start) as u32), ((end) as u32));
vm_block = 5; continue;
}
// C line 45739
14 => {
let _ = { let assigned = (((((*(captures).offset(((((2 as i32)).wrapping_mul(k)).wrapping_add((1 as i32))) as isize)).offset_from((((*(sp)).u).str8).as_mut_ptr()) as i64)).wrapping_shr((shift) as u32)) as i32); end = assigned; assigned };
vm_block = 13; continue;
}
// C line 45738
15 => {
let _ = { let assigned = (((((*(captures).offset((((2 as i32)).wrapping_mul(k)) as isize)).offset_from((((*(sp)).u).str8).as_mut_ptr()) as i64)).wrapping_shr((shift) as u32)) as i32); start = assigned; assigned };
vm_block = 14; continue;
}
// C line 45737
16 => {
vm_block = if ((((!(*(captures).offset((((2 as i32)).wrapping_mul(k)) as isize)).is_null()) && (!(*(captures).offset(((((2 as i32)).wrapping_mul(k)).wrapping_add((1 as i32))) as isize)).is_null())) as i32)) != 0 { 15 } else { 5 }; continue;
}
// C line 45748
17 => {
vm_block = 1; continue;
}
// C line 45747
18 => {
vm_block = if (string_buffer_concat_value_free(b, s)) != 0 { 17 } else { 5 }; continue;
}
// C line 45746
19 => {
vm_block = if ((!((JS_IsUndefined(s)) != 0) as i32)) != 0 { 18 } else { 5 }; continue;
}
// C line 45745
20 => {
vm_block = 1; continue;
}
// C line 45744
21 => {
vm_block = if (JS_IsException(s)) != 0 { 20 } else { 19 }; continue;
}
// C line 45743
22 => {
let _ = { let assigned = JS_GetPropertyInt64(ctx, captures_val, ((k) as i64)); s = assigned; assigned };
vm_block = 21; continue;
}
// C line 45735
23 => {
vm_block = if !(captures).is_null() { 16 } else { 22 }; continue;
}
// C line 45752
24 => {
vm_block = 47; continue;
}
// C line 45734
25 => {
vm_block = if ((((((((k) >= ((1 as i32))) as i32)) != 0) && (((((((k) as u32)) < (captures_len)) as i32)) != 0)) as i32)) != 0 { 23 } else { 24 }; continue;
}
// C line 45730
26 => {
let _ = { let old = j; j = (j).wrapping_add(1); old };
vm_block = 25; continue;
}
// C line 45729
27 => {
let _ = { let assigned = k1; k = assigned; assigned };
vm_block = 26; continue;
}
// C line 45728
28 => {
vm_block = if ((((((((k1) >= ((1 as i32))) as i32)) != 0) && (((((((k1) as u32)) < (captures_len)) as i32)) != 0)) as i32)) != 0 { 27 } else { 25 }; continue;
}
// C line 45727
29 => {
let _ = { let assigned = (((k).wrapping_mul((10 as i32))).wrapping_add(c1)).wrapping_sub((48 as i32)); k1 = assigned; assigned };
vm_block = 28; continue;
}
// C line 45723
30 => {
vm_block = if ((((((((c1) >= ((48 as i32))) as i32)) != 0) && (((((c1) <= ((57 as i32))) as i32)) != 0)) as i32)) != 0 { 29 } else { 25 }; continue;
}
// C line 45722
31 => {
let _ = { let assigned = string_get(rp, j); c1 = assigned; assigned };
vm_block = 30; continue;
}
// C line 45721
32 => {
vm_block = if ((((((j) as u32)) < (len)) as i32)) != 0 { 31 } else { 25 }; continue;
}
// C line 45720
33 => {
let _ = { let assigned = (c).wrapping_sub((48 as i32)); k = assigned; assigned };
vm_block = 32; continue;
}
// C line 45768
34 => {
let _ = { let assigned = (k).wrapping_add((1 as i32)); j = assigned; assigned };
vm_block = 5; continue;
}
// C line 45766
35 => {
vm_block = 1; continue;
}
// C line 45765
36 => {
vm_block = if (string_buffer_concat_value_free(b, capture)) != 0 { 35 } else { 34 }; continue;
}
// C line 45764
37 => {
vm_block = if ((!((JS_IsUndefined(capture)) != 0) as i32)) != 0 { 36 } else { 34 }; continue;
}
// C line 45763
38 => {
vm_block = 1; continue;
}
// C line 45762
39 => {
vm_block = if (JS_IsException(capture)) != 0 { 38 } else { 37 }; continue;
}
// C line 45761
40 => {
let _ = { let assigned = JS_GetPropertyValue(ctx, namedCaptures, name); capture = assigned; assigned };
vm_block = 39; continue;
}
// C line 45760
41 => {
vm_block = 1; continue;
}
// C line 45759
42 => {
vm_block = if (JS_IsException(name)) != 0 { 41 } else { 40 }; continue;
}
// C line 45758
43 => {
let _ = { let assigned = js_sub_string(ctx, rp, j, k); name = assigned; assigned };
vm_block = 42; continue;
}
// C line 45757
44 => {
vm_block = 47; continue;
}
// C line 45756
45 => {
vm_block = if ((((k) < ((0 as i32))) as i32)) != 0 { 44 } else { 43 }; continue;
}
// C line 45755
46 => {
let _ = { let assigned = string_indexof_char(rp, (62 as i32), j); k = assigned; assigned };
vm_block = 45; continue;
}
// C line ? labels: norep
47 => {
let _ = string_buffer_concat(b, rp, ((j0) as u32), ((j) as u32));
vm_block = 5; continue;
}
// C line 45754
48 => {
vm_block = if ((((((((c) == ((60 as i32))) as i32)) != 0) && (((!((JS_IsUndefined(namedCaptures)) != 0) as i32)) != 0)) as i32)) != 0 { 46 } else { 47 }; continue;
}
// C line 45719
49 => {
vm_block = if ((((((((c) >= ((48 as i32))) as i32)) != 0) && (((((c) <= ((57 as i32))) as i32)) != 0)) as i32)) != 0 { 33 } else { 48 }; continue;
}
// C line 45717
50 => {
vm_block = if ((((c) == ((39 as i32))) as i32)) != 0 { 12 } else { 49 }; continue;
}
// C line 45715
51 => {
vm_block = if ((((c) == ((96 as i32))) as i32)) != 0 { 11 } else { 50 }; continue;
}
// C line 45708
52 => {
vm_block = if ((((c) == ((38 as i32))) as i32)) != 0 { 10 } else { 51 }; continue;
}
// C line 45706
53 => {
vm_block = if ((((c) == ((36 as i32))) as i32)) != 0 { 6 } else { 52 }; continue;
}
// C line 45705
54 => {
let _ = { let assigned = string_get(rp, { let old = j; j = (j).wrapping_add(1); old }); c = assigned; assigned };
vm_block = 53; continue;
}
// C line 45704
55 => {
let _ = { let assigned = { let old = j; j = (j).wrapping_add(1); old }; j0 = assigned; assigned };
vm_block = 54; continue;
}
// C line 45703
56 => {
let _ = string_buffer_concat(b, rp, ((i) as u32), ((j) as u32));
vm_block = 55; continue;
}
// C line 45702
57 => {
vm_block = 3; continue;
}
// C line 45701
58 => {
vm_block = if ((((((((j) < ((0 as i32))) as i32)) != 0) || ((((((((j).wrapping_add((1 as i32))) as u32)) >= (len)) as i32)) != 0)) as i32)) != 0 { 57 } else { 56 }; continue;
}
// C line 45700
59 => {
let _ = { let assigned = string_indexof_char(rp, (36 as i32), i); j = assigned; assigned };
vm_block = 58; continue;
}
// C line 45698
60 => {
let _ = { let assigned = (0 as i32); i = assigned; assigned };
vm_block = 4; continue;
}
// C line 45697
61 => {
let _ = { let assigned = (*(rp)).len(); len = assigned; assigned };
vm_block = 60; continue;
}
// C line 45686
62 => {
let _ = { let assigned = (((((*(captures).offset(((1 as i32)) as isize)).offset_from(*(captures).offset(((0 as i32)) as isize)) as i64)).wrapping_shr((shift) as u32)) as u32); matched_len = assigned; assigned };
vm_block = 61; continue;
}
// C line 45694
63 => {
vm_block = 1; continue;
}
// C line 45693
64 => {
vm_block = if (js_get_length32(ctx, core::ptr::addr_of_mut!(matched_len), matched)) != 0 { 63 } else { 61 }; continue;
}
// C line 45691
65 => {
vm_block = 1; continue;
}
// C line 45690
66 => {
vm_block = if (js_get_length32(ctx, core::ptr::addr_of_mut!(captures_len), captures_val)) != 0 { 65 } else { 64 }; continue;
}
// C line 45689
67 => {
vm_block = if ((!((JS_IsUndefined(captures_val)) != 0) as i32)) != 0 { 66 } else { 64 }; continue;
}
// C line 45688
68 => {
let _ = { let assigned = (((0 as i32)) as u32); captures_len = assigned; assigned };
vm_block = 67; continue;
}
// C line 45685
69 => {
vm_block = if !(captures).is_null() { 62 } else { 68 }; continue;
}
// C line 45683
70 => {
let _ = { let assigned = ((((rep).u).ptr) as *mut JSString); rp = assigned; assigned };
vm_block = 69; continue;
}
// C line 45682
71 => {
let _ = { let assigned = (((*(sp)).is_wide_char()) as i32); shift = assigned; assigned };
vm_block = 70; continue;
}
// C line 45680
72 => {
vm_block = 1; continue;
}
// C line 45679
73 => {
let _ = JS_ThrowTypeError(ctx, c"not a string".as_ptr());
vm_block = 72; continue;
}
// C line 45678
74 => {
vm_block = if (((((((rep).tag) as i32)) != ((JS_TAG_STRING as i32))) as i32)) != 0 { 73 } else { 71 }; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:45781. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_string_replace(mut ctx: *mut JSContext, mut this_val: JSValue, mut argc: i32, mut argv: *mut JSValue, mut is_replaceAll: i32) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut O: JSValue = core::mem::zeroed();
let mut searchValue: JSValue = core::mem::zeroed();
let mut replaceValue: JSValue = core::mem::zeroed();
let mut args: [JSValue; 3] = core::mem::zeroed();
let mut str: JSValue = core::mem::zeroed();
let mut search_str: JSValue = core::mem::zeroed();
let mut replaceValue_str: JSValue = core::mem::zeroed();
let mut repl_str: JSValue = core::mem::zeroed();
let mut sp: *mut JSString = core::mem::zeroed();
let mut searchp: *mut JSString = core::mem::zeroed();
let mut b_s: StringBuffer = core::mem::zeroed();
let mut b: *mut StringBuffer = core::mem::zeroed();
let mut pos: i32 = core::mem::zeroed();
let mut functionalReplace: i32 = core::mem::zeroed();
let mut endOfLastMatch: i32 = core::mem::zeroed();
let mut is_first: i32 = core::mem::zeroed();
let mut replacer: JSValue = core::mem::zeroed();
let mut vm_block: usize = 74;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 45891
1 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 45890
2 => {
let _ = JS_FreeValue(ctx, str);
vm_block = 1; continue;
}
// C line 45889
3 => {
let _ = JS_FreeValue(ctx, replaceValue_str);
vm_block = 2; continue;
}
// C line 45888
4 => {
let _ = JS_FreeValue(ctx, search_str);
vm_block = 3; continue;
}
// C line ? labels: exception
5 => {
let _ = string_buffer_free(b);
vm_block = 4; continue;
}
// C line 45884
6 => {
return string_buffer_end(b);
}
// C line 45883
7 => {
let _ = JS_FreeValue(ctx, str);
vm_block = 6; continue;
}
// C line 45882
8 => {
let _ = JS_FreeValue(ctx, replaceValue_str);
vm_block = 7; continue;
}
// C line 45881
9 => {
let _ = JS_FreeValue(ctx, search_str);
vm_block = 8; continue;
}
// C line 45880
10 => {
let _ = string_buffer_concat(b, sp, ((endOfLastMatch) as u32), (*(sp)).len());
vm_block = 9; continue;
}
// C line 45835
11 => {
vm_block = 40; continue;
}
// C line 45878
12 => {
vm_block = 10; continue;
}
// C line 45877
13 => {
vm_block = if ((!((is_replaceAll) != 0) as i32)) != 0 { 12 } else { 11 }; continue;
}
// C line 45876
14 => {
let _ = { let assigned = (0 as i32); is_first = assigned; assigned };
vm_block = 13; continue;
}
// C line 45875
15 => {
let _ = { let assigned = (pos).wrapping_add((((*(searchp)).len()) as i32)); endOfLastMatch = assigned; assigned };
vm_block = 14; continue;
}
// C line 45866
16 => {
let _ = string_buffer_concat_value_free(b, repl_str);
vm_block = 15; continue;
}
// C line 45865
17 => {
vm_block = 5; continue;
}
// C line 45864
18 => {
vm_block = if (JS_IsException(repl_str)) != 0 { 17 } else { 16 }; continue;
}
// C line 45863
19 => {
let _ = { let assigned = JS_ToStringFree(ctx, JS_Call(ctx, replaceValue, JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }, (3 as i32), (args).as_mut_ptr())); repl_str = assigned; assigned };
vm_block = 18; continue;
}
// C line 45862
20 => {
let _ = { let assigned = str; *((args).as_mut_ptr()).offset(((2 as i32)) as isize) = assigned; assigned };
vm_block = 19; continue;
}
// C line 45861
21 => {
let _ = { let assigned = JS_NewInt32(ctx, pos); *((args).as_mut_ptr()).offset(((1 as i32)) as isize) = assigned; assigned };
vm_block = 20; continue;
}
// C line 45860
22 => {
let _ = { let assigned = search_str; *((args).as_mut_ptr()).offset(((0 as i32)) as isize) = assigned; assigned };
vm_block = 21; continue;
}
// C line 45871
23 => {
vm_block = 5; continue;
}
// C line 45868
24 => {
vm_block = if (js_string_GetSubstitution(ctx, b, search_str, sp, ((pos) as u32), JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }, JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }, replaceValue_str, core::ptr::null_mut::<*mut u8>(), (((0 as i32)) as u32))) != 0 { 23 } else { 15 }; continue;
}
// C line 45859
25 => {
vm_block = if (functionalReplace) != 0 { 22 } else { 24 }; continue;
}
// C line 45857
26 => {
let _ = string_buffer_concat(b, sp, ((endOfLastMatch) as u32), ((pos) as u32));
vm_block = 25; continue;
}
// C line 45851
27 => {
return str;
}
// C line 45850
28 => {
let _ = JS_FreeValue(ctx, replaceValue_str);
vm_block = 27; continue;
}
// C line 45849
29 => {
let _ = JS_FreeValue(ctx, search_str);
vm_block = 28; continue;
}
// C line 45848
30 => {
let _ = string_buffer_free(b);
vm_block = 29; continue;
}
// C line 45853
31 => {
vm_block = 10; continue;
}
// C line 45847
32 => {
vm_block = if (is_first) != 0 { 30 } else { 31 }; continue;
}
// C line 45846
33 => {
vm_block = if ((((pos) < ((0 as i32))) as i32)) != 0 { 32 } else { 26 }; continue;
}
// C line 45838
34 => {
let _ = { let assigned = (0 as i32); pos = assigned; assigned };
vm_block = 33; continue;
}
// C line 45840
35 => {
let _ = { let assigned = ((1 as i32)).wrapping_neg(); pos = assigned; assigned };
vm_block = 33; continue;
}
// C line 45842
36 => {
let _ = { let assigned = (endOfLastMatch).wrapping_add((1 as i32)); pos = assigned; assigned };
vm_block = 33; continue;
}
// C line 45839
37 => {
vm_block = if ((((endOfLastMatch) >= ((((*(sp)).len()) as i32))) as i32)) != 0 { 35 } else { 36 }; continue;
}
// C line 45837
38 => {
vm_block = if (is_first) != 0 { 34 } else { 37 }; continue;
}
// C line 45844
39 => {
let _ = { let assigned = string_indexof(sp, searchp, endOfLastMatch); pos = assigned; assigned };
vm_block = 33; continue;
}
// C line 45836
40 => {
vm_block = if ((((!(((!((((((((*(searchp)).len()) as i32)) == ((0 as i32))) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 38 } else { 39 }; continue;
}
// C line 45834
41 => {
let _ = { let assigned = (1 as i32); is_first = assigned; assigned };
vm_block = 11; continue;
}
// C line 45833
42 => {
let _ = { let assigned = (0 as i32); endOfLastMatch = assigned; assigned };
vm_block = 41; continue;
}
// C line 45832
43 => {
let _ = { let assigned = ((((search_str).u).ptr) as *mut JSString); searchp = assigned; assigned };
vm_block = 42; continue;
}
// C line 45831
44 => {
let _ = { let assigned = ((((str).u).ptr) as *mut JSString); sp = assigned; assigned };
vm_block = 43; continue;
}
// C line 45828
45 => {
vm_block = 5; continue;
}
// C line 45827
46 => {
vm_block = if (JS_IsException(replaceValue_str)) != 0 { 45 } else { 44 }; continue;
}
// C line 45826
47 => {
let _ = { let assigned = JS_ToString(ctx, replaceValue); replaceValue_str = assigned; assigned };
vm_block = 46; continue;
}
// C line 45825
48 => {
vm_block = if ((!((functionalReplace) != 0) as i32)) != 0 { 47 } else { 44 }; continue;
}
// C line 45824
49 => {
let _ = { let assigned = JS_IsFunction(ctx, replaceValue); functionalReplace = assigned; assigned };
vm_block = 48; continue;
}
// C line 45823
50 => {
vm_block = 5; continue;
}
// C line 45822
51 => {
vm_block = if (JS_IsException(search_str)) != 0 { 50 } else { 49 }; continue;
}
// C line 45821
52 => {
let _ = { let assigned = JS_ToString(ctx, searchValue); search_str = assigned; assigned };
vm_block = 51; continue;
}
// C line 45820
53 => {
vm_block = 5; continue;
}
// C line 45819
54 => {
vm_block = if (JS_IsException(str)) != 0 { 53 } else { 52 }; continue;
}
// C line 45818
55 => {
let _ = { let assigned = JS_ToString(ctx, O); str = assigned; assigned };
vm_block = 54; continue;
}
// C line 45816
56 => {
let _ = string_buffer_init(ctx, b, (0 as i32));
vm_block = 55; continue;
}
// C line 45813
57 => {
return JS_CallFree(ctx, replacer, searchValue, (2 as i32), (args).as_mut_ptr());
}
// C line 45812
58 => {
let _ = { let assigned = replaceValue; *((args).as_mut_ptr()).offset(((1 as i32)) as isize) = assigned; assigned };
vm_block = 57; continue;
}
// C line 45811
59 => {
let _ = { let assigned = O; *((args).as_mut_ptr()).offset(((0 as i32)) as isize) = assigned; assigned };
vm_block = 58; continue;
}
// C line 45810
60 => {
vm_block = if ((((((!((JS_IsUndefined(replacer)) != 0) as i32)) != 0) && (((!((JS_IsNull(replacer)) != 0) as i32)) != 0)) as i32)) != 0 { 59 } else { 56 }; continue;
}
// C line 45809
61 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 45808
62 => {
vm_block = if (JS_IsException(replacer)) != 0 { 61 } else { 60 }; continue;
}
// C line 45807
63 => {
let _ = { let assigned = JS_GetProperty(ctx, searchValue, (((crate::quickjs_atom::JS_ATOM_Symbol_replace as i32)) as JSAtom)); replacer = assigned; assigned };
vm_block = 62; continue;
}
// C line 45805
64 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 45804
65 => {
vm_block = if ((((check_regexp_g_flag(ctx, searchValue)) < ((0 as i32))) as i32)) != 0 { 64 } else { 63 }; continue;
}
// C line 45803
66 => {
vm_block = if (is_replaceAll) != 0 { 65 } else { 63 }; continue;
}
// C line 45801
67 => {
vm_block = if (JS_IsObject(searchValue)) != 0 { 66 } else { 56 }; continue;
}
// C line 45799
68 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; repl_str = assigned; assigned };
vm_block = 67; continue;
}
// C line 45798
69 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; replaceValue_str = assigned; assigned };
vm_block = 68; continue;
}
// C line 45797
70 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; search_str = assigned; assigned };
vm_block = 69; continue;
}
// C line 45795
71 => {
return JS_ThrowTypeError(ctx, c"cannot convert to object".as_ptr());
}
// C line 45794
72 => {
vm_block = if (((((JS_IsUndefined(O)) != 0) || ((JS_IsNull(O)) != 0)) as i32)) != 0 { 71 } else { 70 }; continue;
}
// C line 45790
73 => {
b = core::ptr::addr_of_mut!(b_s);
vm_block = 72; continue;
}
// C line 45786
74 => {
O = this_val;
searchValue = *(argv).offset(((0 as i32)) as isize);
replaceValue = *(argv).offset(((1 as i32)) as isize);
vm_block = 73; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:45894. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_string_split(mut ctx: *mut JSContext, mut this_val: JSValue, mut argc: i32, mut argv: *mut JSValue) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut O: JSValue = core::mem::zeroed();
let mut separator: JSValue = core::mem::zeroed();
let mut limit: JSValue = core::mem::zeroed();
let mut args: [JSValue; 2] = core::mem::zeroed();
let mut S: JSValue = core::mem::zeroed();
let mut A: JSValue = core::mem::zeroed();
let mut R: JSValue = core::mem::zeroed();
let mut T: JSValue = core::mem::zeroed();
let mut lim: u32 = core::mem::zeroed();
let mut lengthA: u32 = core::mem::zeroed();
let mut p: i64 = core::mem::zeroed();
let mut q: i64 = core::mem::zeroed();
let mut s: i64 = core::mem::zeroed();
let mut r: i64 = core::mem::zeroed();
let mut e: i64 = core::mem::zeroed();
let mut sp: *mut JSString = core::mem::zeroed();
let mut rp: *mut JSString = core::mem::zeroed();
let mut splitter: JSValue = core::mem::zeroed();
let mut vm_block: usize = 66;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 45980
1 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 45979
2 => {
let _ = JS_FreeValue(ctx, R);
vm_block = 1; continue;
}
// C line 45978
3 => {
let _ = JS_FreeValue(ctx, S);
vm_block = 2; continue;
}
// C line ? labels: exception
4 => {
let _ = JS_FreeValue(ctx, A);
vm_block = 3; continue;
}
// C line 45974
5 => {
return A;
}
// C line 45973
6 => {
let _ = JS_FreeValue(ctx, R);
vm_block = 5; continue;
}
// C line ? labels: done
7 => {
let _ = JS_FreeValue(ctx, S);
vm_block = 6; continue;
}
// C line 45970
8 => {
vm_block = 4; continue;
}
// C line 45969
9 => {
vm_block = if ((((JS_CreateDataPropertyUint32(ctx, A, (({ let old = lengthA; lengthA = (lengthA).wrapping_add(1); old }) as i64), T, (0 as i32))) < ((0 as i32))) as i32)) != 0 { 8 } else { 7 }; continue;
}
// C line 45968
10 => {
vm_block = 4; continue;
}
// C line 45967
11 => {
vm_block = if (JS_IsException(T)) != 0 { 10 } else { 9 }; continue;
}
// C line ? labels: add_tail
12 => {
let _ = { let assigned = js_sub_string(ctx, sp, ((p) as i32), ((s) as i32)); T = assigned; assigned };
vm_block = 11; continue;
}
// C line 45953
13 => {
vm_block = if (((({ q = (q).wrapping_add((((!((r) != 0) as i32)) as i64)); q }) <= (((s).wrapping_sub(r)).wrapping_sub((((!((r) != 0) as i32)) as i64)))) as i32)) != 0 { 24 } else { 12 }; continue;
}
// C line ?
14 => {
let _ = { let assigned = { let assigned = (e).wrapping_add(r); p = assigned; assigned }; q = assigned; assigned };
vm_block = 13; continue;
}
// C line 45963
15 => {
vm_block = 7; continue;
}
// C line 45962
16 => {
vm_block = if ((((lengthA) == (lim)) as i32)) != 0 { 15 } else { 14 }; continue;
}
// C line 45961
17 => {
vm_block = 4; continue;
}
// C line 45960
18 => {
vm_block = if ((((JS_CreateDataPropertyUint32(ctx, A, (({ let old = lengthA; lengthA = (lengthA).wrapping_add(1); old }) as i64), T, (0 as i32))) < ((0 as i32))) as i32)) != 0 { 17 } else { 16 }; continue;
}
// C line 45959
19 => {
vm_block = 4; continue;
}
// C line 45958
20 => {
vm_block = if (JS_IsException(T)) != 0 { 19 } else { 18 }; continue;
}
// C line 45957
21 => {
let _ = { let assigned = js_sub_string(ctx, sp, ((p) as i32), ((e) as i32)); T = assigned; assigned };
vm_block = 20; continue;
}
// C line 45956
22 => {
vm_block = 12; continue;
}
// C line 45955
23 => {
vm_block = if ((((e) < ((((0 as i32)) as i64))) as i32)) != 0 { 22 } else { 21 }; continue;
}
// C line 45954
24 => {
let _ = { let assigned = ((string_indexof(sp, rp, ((q) as i32))) as i64); e = assigned; assigned };
vm_block = 23; continue;
}
// C line 45953
25 => {
let _ = { let assigned = p; q = assigned; assigned };
vm_block = 13; continue;
}
// C line 45951
26 => {
vm_block = 7; continue;
}
// C line 45950
27 => {
vm_block = 12; continue;
}
// C line 45949
28 => {
vm_block = if ((((r) != ((((0 as i32)) as i64))) as i32)) != 0 { 27 } else { 26 }; continue;
}
// C line 45948
29 => {
vm_block = if ((((s) == ((((0 as i32)) as i64))) as i32)) != 0 { 28 } else { 25 }; continue;
}
// C line 45947
30 => {
vm_block = 12; continue;
}
// C line 45946
31 => {
vm_block = if (JS_IsUndefined(separator)) != 0 { 30 } else { 29 }; continue;
}
// C line 45945
32 => {
vm_block = 7; continue;
}
// C line 45944
33 => {
vm_block = if ((((lim) == ((((0 as i32)) as u32))) as i32)) != 0 { 32 } else { 31 }; continue;
}
// C line 45943
34 => {
let _ = { let assigned = (((0 as i32)) as i64); p = assigned; assigned };
vm_block = 33; continue;
}
// C line 45942
35 => {
let _ = { let assigned = (((*(rp)).len()) as i64); r = assigned; assigned };
vm_block = 34; continue;
}
// C line 45941
36 => {
let _ = { let assigned = ((((R).u).ptr) as *mut JSString); rp = assigned; assigned };
vm_block = 35; continue;
}
// C line 45940
37 => {
vm_block = 4; continue;
}
// C line 45939
38 => {
vm_block = if (JS_IsException(R)) != 0 { 37 } else { 36 }; continue;
}
// C line 45938
39 => {
let _ = { let assigned = JS_ToString(ctx, separator); R = assigned; assigned };
vm_block = 38; continue;
}
// C line 45937
40 => {
let _ = { let assigned = (((*(sp)).len()) as i64); s = assigned; assigned };
vm_block = 39; continue;
}
// C line 45936
41 => {
let _ = { let assigned = ((((S).u).ptr) as *mut JSString); sp = assigned; assigned };
vm_block = 40; continue;
}
// C line 45931
42 => {
let _ = { let assigned = (4294967295 as u32); lim = assigned; assigned };
vm_block = 41; continue;
}
// C line 45934
43 => {
vm_block = 4; continue;
}
// C line 45933
44 => {
vm_block = if ((((JS_ToUint32(ctx, core::ptr::addr_of_mut!(lim), limit)) < ((0 as i32))) as i32)) != 0 { 43 } else { 41 }; continue;
}
// C line 45930
45 => {
vm_block = if (JS_IsUndefined(limit)) != 0 { 42 } else { 44 }; continue;
}
// C line 45929
46 => {
let _ = { let assigned = (((0 as i32)) as u32); lengthA = assigned; assigned };
vm_block = 45; continue;
}
// C line 45928
47 => {
vm_block = 4; continue;
}
// C line 45927
48 => {
vm_block = if (JS_IsException(A)) != 0 { 47 } else { 46 }; continue;
}
// C line 45926
49 => {
let _ = { let assigned = JS_NewArray(ctx); A = assigned; assigned };
vm_block = 48; continue;
}
// C line 45925
50 => {
vm_block = 4; continue;
}
// C line 45924
51 => {
vm_block = if (JS_IsException(S)) != 0 { 50 } else { 49 }; continue;
}
// C line 45923
52 => {
let _ = { let assigned = JS_ToString(ctx, O); S = assigned; assigned };
vm_block = 51; continue;
}
// C line 45920
53 => {
return JS_CallFree(ctx, splitter, separator, (2 as i32), (args).as_mut_ptr());
}
// C line 45919
54 => {
let _ = { let assigned = limit; *((args).as_mut_ptr()).offset(((1 as i32)) as isize) = assigned; assigned };
vm_block = 53; continue;
}
// C line 45918
55 => {
let _ = { let assigned = O; *((args).as_mut_ptr()).offset(((0 as i32)) as isize) = assigned; assigned };
vm_block = 54; continue;
}
// C line 45917
56 => {
vm_block = if ((((((!((JS_IsUndefined(splitter)) != 0) as i32)) != 0) && (((!((JS_IsNull(splitter)) != 0) as i32)) != 0)) as i32)) != 0 { 55 } else { 52 }; continue;
}
// C line 45916
57 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 45915
58 => {
vm_block = if (JS_IsException(splitter)) != 0 { 57 } else { 56 }; continue;
}
// C line 45914
59 => {
let _ = { let assigned = JS_GetProperty(ctx, separator, (((crate::quickjs_atom::JS_ATOM_Symbol_split as i32)) as JSAtom)); splitter = assigned; assigned };
vm_block = 58; continue;
}
// C line 45912
60 => {
vm_block = if (JS_IsObject(separator)) != 0 { 59 } else { 52 }; continue;
}
// C line 45910
61 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; R = assigned; assigned };
vm_block = 60; continue;
}
// C line 45909
62 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; A = assigned; assigned };
vm_block = 61; continue;
}
// C line 45908
63 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; S = assigned; assigned };
vm_block = 62; continue;
}
// C line 45906
64 => {
return JS_ThrowTypeError(ctx, c"cannot convert to object".as_ptr());
}
// C line 45905
65 => {
vm_block = if (((((JS_IsUndefined(O)) != 0) || ((JS_IsNull(O)) != 0)) as i32)) != 0 { 64 } else { 63 }; continue;
}
// C line 45898
66 => {
O = this_val;
separator = *(argv).offset(((0 as i32)) as isize);
limit = *(argv).offset(((1 as i32)) as isize);
vm_block = 65; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:45983. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_string_substring(mut ctx: *mut JSContext, mut this_val: JSValue, mut argc: i32, mut argv: *mut JSValue) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut str: JSValue = core::mem::zeroed();
let mut ret: JSValue = core::mem::zeroed();
let mut a: i32 = core::mem::zeroed();
let mut b: i32 = core::mem::zeroed();
let mut start: i32 = core::mem::zeroed();
let mut end: i32 = core::mem::zeroed();
let mut p: *mut JSString = core::mem::zeroed();
let mut vm_block: usize = 20;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 46014
1 => {
return ret;
}
// C line 46013
2 => {
let _ = JS_FreeValue(ctx, str);
vm_block = 1; continue;
}
// C line 46012
3 => {
let _ = { let assigned = js_sub_string(ctx, p, start, end); ret = assigned; assigned };
vm_block = 2; continue;
}
// C line 46007
4 => {
let _ = { let assigned = b; end = assigned; assigned };
vm_block = 3; continue;
}
// C line 46006
5 => {
let _ = { let assigned = a; start = assigned; assigned };
vm_block = 4; continue;
}
// C line 46010
6 => {
let _ = { let assigned = a; end = assigned; assigned };
vm_block = 3; continue;
}
// C line 46009
7 => {
let _ = { let assigned = b; start = assigned; assigned };
vm_block = 6; continue;
}
// C line 46005
8 => {
vm_block = if ((((a) < (b)) as i32)) != 0 { 5 } else { 7 }; continue;
}
// C line 46002
9 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 46001
10 => {
let _ = JS_FreeValue(ctx, str);
vm_block = 9; continue;
}
// C line 46000
11 => {
vm_block = if (JS_ToInt32Clamp(ctx, core::ptr::addr_of_mut!(b), *(argv).offset(((1 as i32)) as isize), (0 as i32), (((*(p)).len()) as i32), (0 as i32))) != 0 { 10 } else { 8 }; continue;
}
// C line 45999
12 => {
vm_block = if ((!((JS_IsUndefined(*(argv).offset(((1 as i32)) as isize))) != 0) as i32)) != 0 { 11 } else { 8 }; continue;
}
// C line 45998
13 => {
let _ = { let assigned = (((*(p)).len()) as i32); b = assigned; assigned };
vm_block = 12; continue;
}
// C line 45996
14 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 45995
15 => {
let _ = JS_FreeValue(ctx, str);
vm_block = 14; continue;
}
// C line 45994
16 => {
vm_block = if (JS_ToInt32Clamp(ctx, core::ptr::addr_of_mut!(a), *(argv).offset(((0 as i32)) as isize), (0 as i32), (((*(p)).len()) as i32), (0 as i32))) != 0 { 15 } else { 13 }; continue;
}
// C line 45993
17 => {
let _ = { let assigned = ((((str).u).ptr) as *mut JSString); p = assigned; assigned };
vm_block = 16; continue;
}
// C line 45992
18 => {
return str;
}
// C line 45991
19 => {
vm_block = if (JS_IsException(str)) != 0 { 18 } else { 17 }; continue;
}
// C line 45990
20 => {
let _ = { let assigned = JS_ToStringCheckObject(ctx, this_val); str = assigned; assigned };
vm_block = 19; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:46017. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_string_substr(mut ctx: *mut JSContext, mut this_val: JSValue, mut argc: i32, mut argv: *mut JSValue) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut str: JSValue = core::mem::zeroed();
let mut ret: JSValue = core::mem::zeroed();
let mut a: i32 = core::mem::zeroed();
let mut len: i32 = core::mem::zeroed();
let mut n: i32 = core::mem::zeroed();
let mut p: *mut JSString = core::mem::zeroed();
let mut vm_block: usize = 16;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 46042
1 => {
return ret;
}
// C line 46041
2 => {
let _ = JS_FreeValue(ctx, str);
vm_block = 1; continue;
}
// C line 46040
3 => {
let _ = { let assigned = js_sub_string(ctx, p, a, (a).wrapping_add(n)); ret = assigned; assigned };
vm_block = 2; continue;
}
// C line 46037
4 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 46036
5 => {
let _ = JS_FreeValue(ctx, str);
vm_block = 4; continue;
}
// C line 46035
6 => {
vm_block = if (JS_ToInt32Clamp(ctx, core::ptr::addr_of_mut!(n), *(argv).offset(((1 as i32)) as isize), (0 as i32), (len).wrapping_sub(a), (0 as i32))) != 0 { 5 } else { 3 }; continue;
}
// C line 46034
7 => {
vm_block = if ((!((JS_IsUndefined(*(argv).offset(((1 as i32)) as isize))) != 0) as i32)) != 0 { 6 } else { 3 }; continue;
}
// C line 46033
8 => {
let _ = { let assigned = (len).wrapping_sub(a); n = assigned; assigned };
vm_block = 7; continue;
}
// C line 46031
9 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 46030
10 => {
let _ = JS_FreeValue(ctx, str);
vm_block = 9; continue;
}
// C line 46029
11 => {
vm_block = if (JS_ToInt32Clamp(ctx, core::ptr::addr_of_mut!(a), *(argv).offset(((0 as i32)) as isize), (0 as i32), len, len)) != 0 { 10 } else { 8 }; continue;
}
// C line 46028
12 => {
let _ = { let assigned = (((*(p)).len()) as i32); len = assigned; assigned };
vm_block = 11; continue;
}
// C line 46027
13 => {
let _ = { let assigned = ((((str).u).ptr) as *mut JSString); p = assigned; assigned };
vm_block = 12; continue;
}
// C line 46026
14 => {
return str;
}
// C line 46025
15 => {
vm_block = if (JS_IsException(str)) != 0 { 14 } else { 13 }; continue;
}
// C line 46024
16 => {
let _ = { let assigned = JS_ToStringCheckObject(ctx, this_val); str = assigned; assigned };
vm_block = 15; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:46045. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_string_slice(mut ctx: *mut JSContext, mut this_val: JSValue, mut argc: i32, mut argv: *mut JSValue) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut str: JSValue = core::mem::zeroed();
let mut ret: JSValue = core::mem::zeroed();
let mut len: i32 = core::mem::zeroed();
let mut start: i32 = core::mem::zeroed();
let mut end: i32 = core::mem::zeroed();
let mut p: *mut JSString = core::mem::zeroed();
let mut vm_block: usize = 16;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 46070
1 => {
return ret;
}
// C line 46069
2 => {
let _ = JS_FreeValue(ctx, str);
vm_block = 1; continue;
}
// C line 46068
3 => {
let _ = { let assigned = js_sub_string(ctx, p, start, crate::cutils_header::max_int(end, start)); ret = assigned; assigned };
vm_block = 2; continue;
}
// C line 46065
4 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 46064
5 => {
let _ = JS_FreeValue(ctx, str);
vm_block = 4; continue;
}
// C line 46063
6 => {
vm_block = if (JS_ToInt32Clamp(ctx, core::ptr::addr_of_mut!(end), *(argv).offset(((1 as i32)) as isize), (0 as i32), len, len)) != 0 { 5 } else { 3 }; continue;
}
// C line 46062
7 => {
vm_block = if ((!((JS_IsUndefined(*(argv).offset(((1 as i32)) as isize))) != 0) as i32)) != 0 { 6 } else { 3 }; continue;
}
// C line 46061
8 => {
let _ = { let assigned = len; end = assigned; assigned };
vm_block = 7; continue;
}
// C line 46059
9 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 46058
10 => {
let _ = JS_FreeValue(ctx, str);
vm_block = 9; continue;
}
// C line 46057
11 => {
vm_block = if (JS_ToInt32Clamp(ctx, core::ptr::addr_of_mut!(start), *(argv).offset(((0 as i32)) as isize), (0 as i32), len, len)) != 0 { 10 } else { 8 }; continue;
}
// C line 46056
12 => {
let _ = { let assigned = (((*(p)).len()) as i32); len = assigned; assigned };
vm_block = 11; continue;
}
// C line 46055
13 => {
let _ = { let assigned = ((((str).u).ptr) as *mut JSString); p = assigned; assigned };
vm_block = 12; continue;
}
// C line 46054
14 => {
return str;
}
// C line 46053
15 => {
vm_block = if (JS_IsException(str)) != 0 { 14 } else { 13 }; continue;
}
// C line 46052
16 => {
let _ = { let assigned = JS_ToStringCheckObject(ctx, this_val); str = assigned; assigned };
vm_block = 15; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:46073. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_string_pad(mut ctx: *mut JSContext, mut this_val: JSValue, mut argc: i32, mut argv: *mut JSValue, mut padEnd: i32) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut str: JSValue = core::mem::zeroed();
let mut v: JSValue = core::mem::zeroed();
let mut b_s: StringBuffer = core::mem::zeroed();
let mut b: *mut StringBuffer = core::mem::zeroed();
let mut p: *mut JSString = core::mem::zeroed();
let mut p1: *mut JSString = core::mem::zeroed();
let mut n: i32 = core::mem::zeroed();
let mut len: i32 = core::mem::zeroed();
let mut c: i32 = core::mem::zeroed();
let mut chunk: i32 = core::mem::zeroed();
let mut vm_block: usize = 51;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line ? labels: fail1
1 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line ? labels: fail2
2 => {
let _ = JS_FreeValue(ctx, str);
vm_block = 1; continue;
}
// C line ? labels: fail3
3 => {
let _ = JS_FreeValue(ctx, v);
vm_block = 2; continue;
}
// C line ? labels: fail
4 => {
let _ = string_buffer_free(b);
vm_block = 3; continue;
}
// C line 46132
5 => {
return string_buffer_end(b);
}
// C line 46131
6 => {
let _ = JS_FreeValue(ctx, str);
vm_block = 5; continue;
}
// C line 46130
7 => {
let _ = JS_FreeValue(ctx, v);
vm_block = 6; continue;
}
// C line 46128
8 => {
vm_block = 4; continue;
}
// C line 46127
9 => {
vm_block = if (string_buffer_concat(b, p, (((0 as i32)) as u32), ((len) as u32))) != 0 { 8 } else { 7 }; continue;
}
// C line 46126
10 => {
vm_block = if ((!((padEnd) != 0) as i32)) != 0 { 9 } else { 7 }; continue;
}
// C line 46116
11 => {
vm_block = if ((((n) > ((0 as i32))) as i32)) != 0 { 15 } else { 10 }; continue;
}
// C line 46120
12 => {
let _ = { n = (n).wrapping_sub(chunk); n };
vm_block = 11; continue;
}
// C line 46119
13 => {
vm_block = 4; continue;
}
// C line 46118
14 => {
vm_block = if (string_buffer_concat(b, p1, (((0 as i32)) as u32), ((chunk) as u32))) != 0 { 13 } else { 12 }; continue;
}
// C line 46117
15 => {
chunk = crate::cutils_header::min_int(n, (((*(p1)).len()) as i32));
vm_block = 14; continue;
}
// C line 46124
16 => {
vm_block = 4; continue;
}
// C line 46123
17 => {
vm_block = if (string_buffer_fill(b, c, n)) != 0 { 16 } else { 10 }; continue;
}
// C line 46115
18 => {
vm_block = if !(p1).is_null() { 11 } else { 17 }; continue;
}
// C line 46113
19 => {
vm_block = 4; continue;
}
// C line 46112
20 => {
vm_block = if (string_buffer_concat(b, p, (((0 as i32)) as u32), ((len) as u32))) != 0 { 19 } else { 18 }; continue;
}
// C line 46111
21 => {
vm_block = if (padEnd) != 0 { 20 } else { 18 }; continue;
}
// C line 46110
22 => {
let _ = { n = (n).wrapping_sub(len); n };
vm_block = 21; continue;
}
// C line 46109
23 => {
vm_block = 3; continue;
}
// C line 46108
24 => {
vm_block = if (string_buffer_init(ctx, b, n)) != 0 { 23 } else { 22 }; continue;
}
// C line 46106
25 => {
vm_block = 3; continue;
}
// C line 46105
26 => {
let _ = JS_ThrowRangeError(ctx, c"invalid string length".as_ptr());
vm_block = 25; continue;
}
// C line 46104
27 => {
vm_block = if ((((n) > ((((1 as i32)).wrapping_shl(((30 as i32)) as u32)).wrapping_sub((1 as i32)))) as i32)) != 0 { 26 } else { 24 }; continue;
}
// C line 46101
28 => {
let _ = { let assigned = core::ptr::null_mut::<JSString>(); p1 = assigned; assigned };
vm_block = 27; continue;
}
// C line 46100
29 => {
let _ = { let assigned = string_get(p1, (0 as i32)); c = assigned; assigned };
vm_block = 28; continue;
}
// C line 46099
30 => {
vm_block = if (((((((*(p1)).len()) as i32)) == ((1 as i32))) as i32)) != 0 { 29 } else { 27 }; continue;
}
// C line 46097
31 => {
return str;
}
// C line 46096
32 => {
let _ = JS_FreeValue(ctx, v);
vm_block = 31; continue;
}
// C line 46095
33 => {
vm_block = if (((((((*(p1)).len()) as i32)) == ((0 as i32))) as i32)) != 0 { 32 } else { 30 }; continue;
}
// C line 46094
34 => {
let _ = { let assigned = ((((v).u).ptr) as *mut JSString); p1 = assigned; assigned };
vm_block = 33; continue;
}
// C line 46093
35 => {
vm_block = 2; continue;
}
// C line 46092
36 => {
vm_block = if (JS_IsException(v)) != 0 { 35 } else { 34 }; continue;
}
// C line 46091
37 => {
let _ = { let assigned = JS_ToString(ctx, *(argv).offset(((1 as i32)) as isize)); v = assigned; assigned };
vm_block = 36; continue;
}
// C line 46090
38 => {
vm_block = if ((((((((argc) > ((1 as i32))) as i32)) != 0) && (((!((JS_IsUndefined(*(argv).offset(((1 as i32)) as isize))) != 0) as i32)) != 0)) as i32)) != 0 { 37 } else { 27 }; continue;
}
// C line 46089
39 => {
return str;
}
// C line 46088
40 => {
vm_block = if ((((len) >= (n)) as i32)) != 0 { 39 } else { 38 }; continue;
}
// C line 46087
41 => {
let _ = { let assigned = (((*(p)).len()) as i32); len = assigned; assigned };
vm_block = 40; continue;
}
// C line 46086
42 => {
let _ = { let assigned = ((((str).u).ptr) as *mut JSString); p = assigned; assigned };
vm_block = 41; continue;
}
// C line 46085
43 => {
vm_block = 2; continue;
}
// C line 46084
44 => {
vm_block = if (JS_ToInt32Sat(ctx, core::ptr::addr_of_mut!(n), *(argv).offset(((0 as i32)) as isize))) != 0 { 43 } else { 42 }; continue;
}
// C line 46083
45 => {
vm_block = 1; continue;
}
// C line 46082
46 => {
vm_block = if (JS_IsException(str)) != 0 { 45 } else { 44 }; continue;
}
// C line 46081
47 => {
let _ = { let assigned = JS_ToStringCheckObject(ctx, this_val); str = assigned; assigned };
vm_block = 46; continue;
}
// C line 46079
48 => {
c = (32 as i32);
vm_block = 47; continue;
}
// C line 46078
49 => {
p1 = core::ptr::null_mut::<JSString>();
vm_block = 48; continue;
}
// C line 46077
50 => {
b = core::ptr::addr_of_mut!(b_s);
vm_block = 49; continue;
}
// C line 46076
51 => {
v = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) };
vm_block = 50; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:46144. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_string_repeat(mut ctx: *mut JSContext, mut this_val: JSValue, mut argc: i32, mut argv: *mut JSValue) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut str: JSValue = core::mem::zeroed();
let mut b_s: StringBuffer = core::mem::zeroed();
let mut b: *mut StringBuffer = core::mem::zeroed();
let mut p: *mut JSString = core::mem::zeroed();
let mut val: i64 = core::mem::zeroed();
let mut n: i32 = core::mem::zeroed();
let mut len: i32 = core::mem::zeroed();
let mut vm_block: usize = 27;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 46186
1 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line ? labels: fail
2 => {
let _ = JS_FreeValue(ctx, str);
vm_block = 1; continue;
}
// C line 46182
3 => {
return string_buffer_end(b);
}
// C line 46181
4 => {
let _ = JS_FreeValue(ctx, str);
vm_block = 3; continue;
}
// C line 46175
5 => {
let _ = string_buffer_fill(b, string_get(p, (0 as i32)), n);
vm_block = 4; continue;
}
// C line 46177
6 => {
vm_block = if (((({ let old = n; n = (n).wrapping_sub(1); old }) > ((0 as i32))) as i32)) != 0 { 7 } else { 4 }; continue;
}
// C line 46178
7 => {
let _ = string_buffer_concat(b, p, (((0 as i32)) as u32), ((len) as u32));
vm_block = 6; continue;
}
// C line 46174
8 => {
vm_block = if ((((len) == ((1 as i32))) as i32)) != 0 { 5 } else { 6 }; continue;
}
// C line 46173
9 => {
vm_block = 2; continue;
}
// C line 46172
10 => {
vm_block = if (string_buffer_init2(ctx, b, (n).wrapping_mul(len), (((*(p)).is_wide_char()) as i32))) != 0 { 9 } else { 8 }; continue;
}
// C line 46170
11 => {
vm_block = 2; continue;
}
// C line 46169
12 => {
let _ = JS_ThrowRangeError(ctx, c"invalid string length".as_ptr());
vm_block = 11; continue;
}
// C line 46168
13 => {
vm_block = if (((((val).wrapping_mul(((len) as i64))) > ((((((1 as i32)).wrapping_shl(((30 as i32)) as u32)).wrapping_sub((1 as i32))) as i64))) as i32)) != 0 { 12 } else { 10 }; continue;
}
// C line 46166
14 => {
return str;
}
// C line 46165
15 => {
vm_block = if ((((((((len) == ((0 as i32))) as i32)) != 0) || (((((n) == ((1 as i32))) as i32)) != 0)) as i32)) != 0 { 14 } else { 13 }; continue;
}
// C line 46164
16 => {
let _ = { let assigned = (((*(p)).len()) as i32); len = assigned; assigned };
vm_block = 15; continue;
}
// C line 46163
17 => {
let _ = { let assigned = ((((str).u).ptr) as *mut JSString); p = assigned; assigned };
vm_block = 16; continue;
}
// C line 46162
18 => {
let _ = { let assigned = ((val) as i32); n = assigned; assigned };
vm_block = 17; continue;
}
// C line 46160
19 => {
vm_block = 2; continue;
}
// C line 46159
20 => {
let _ = JS_ThrowRangeError(ctx, c"invalid repeat count".as_ptr());
vm_block = 19; continue;
}
// C line 46158
21 => {
vm_block = if ((((((((val) < ((((0 as i32)) as i64))) as i32)) != 0) || (((((val) > ((((2147483647 as i32)) as i64))) as i32)) != 0)) as i32)) != 0 { 20 } else { 18 }; continue;
}
// C line 46157
22 => {
vm_block = 2; continue;
}
// C line 46156
23 => {
vm_block = if (JS_ToInt64Sat(ctx, core::ptr::addr_of_mut!(val), *(argv).offset(((0 as i32)) as isize))) != 0 { 22 } else { 21 }; continue;
}
// C line 46155
24 => {
vm_block = 2; continue;
}
// C line 46154
25 => {
vm_block = if (JS_IsException(str)) != 0 { 24 } else { 23 }; continue;
}
// C line 46153
26 => {
let _ = { let assigned = JS_ToStringCheckObject(ctx, this_val); str = assigned; assigned };
vm_block = 25; continue;
}
// C line 46148
27 => {
b = core::ptr::addr_of_mut!(b_s);
vm_block = 26; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:46189. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_string_trim(mut ctx: *mut JSContext, mut this_val: JSValue, mut argc: i32, mut argv: *mut JSValue, mut magic: i32) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut str: JSValue = core::mem::zeroed();
let mut ret: JSValue = core::mem::zeroed();
let mut a: i32 = core::mem::zeroed();
let mut b: i32 = core::mem::zeroed();
let mut len: i32 = core::mem::zeroed();
let mut p: *mut JSString = core::mem::zeroed();
let mut vm_block: usize = 15;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 46212
1 => {
return ret;
}
// C line 46211
2 => {
let _ = JS_FreeValue(ctx, str);
vm_block = 1; continue;
}
// C line 46210
3 => {
let _ = { let assigned = js_sub_string(ctx, p, a, b); ret = assigned; assigned };
vm_block = 2; continue;
}
// C line 46207
4 => {
vm_block = if ((((((((b) > (a)) as i32)) != 0) && ((crate::libunicode::lre_is_space(((string_get(p, (b).wrapping_sub((1 as i32)))) as u32))) != 0)) as i32)) != 0 { 5 } else { 3 }; continue;
}
// C line 46208
5 => {
let _ = { let old = b; b = (b).wrapping_sub(1); old };
vm_block = 4; continue;
}
// C line 46206
6 => {
vm_block = if (((magic) & ((2 as i32)))) != 0 { 4 } else { 3 }; continue;
}
// C line 46203
7 => {
vm_block = if ((((((((a) < (len)) as i32)) != 0) && ((crate::libunicode::lre_is_space(((string_get(p, a)) as u32))) != 0)) as i32)) != 0 { 8 } else { 6 }; continue;
}
// C line 46204
8 => {
let _ = { let old = a; a = (a).wrapping_add(1); old };
vm_block = 7; continue;
}
// C line 46202
9 => {
vm_block = if (((magic) & ((1 as i32)))) != 0 { 7 } else { 6 }; continue;
}
// C line 46201
10 => {
let _ = { let assigned = { let assigned = (((*(p)).len()) as i32); len = assigned; assigned }; b = assigned; assigned };
vm_block = 9; continue;
}
// C line 46200
11 => {
let _ = { let assigned = (0 as i32); a = assigned; assigned };
vm_block = 10; continue;
}
// C line 46199
12 => {
let _ = { let assigned = ((((str).u).ptr) as *mut JSString); p = assigned; assigned };
vm_block = 11; continue;
}
// C line 46198
13 => {
return str;
}
// C line 46197
14 => {
vm_block = if (JS_IsException(str)) != 0 { 13 } else { 12 }; continue;
}
// C line 46196
15 => {
let _ = { let assigned = JS_ToStringCheckObject(ctx, this_val); str = assigned; assigned };
vm_block = 14; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:46216. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn string_prevc(mut p: *mut JSString, mut pidx: *mut i32) -> i32 {
let mut vm_local_storage = Vec::<u64>::new();
let mut idx: i32 = core::mem::zeroed();
let mut c: i32 = core::mem::zeroed();
let mut c1: i32 = core::mem::zeroed();
let mut vm_block: usize = 14;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 46237
1 => {
return c;
}
// C line 46236
2 => {
let _ = { let assigned = idx; *(pidx) = assigned; assigned };
vm_block = 1; continue;
}
// C line 46230
3 => {
let _ = { let old = idx; idx = (idx).wrapping_sub(1); old };
vm_block = 2; continue;
}
// C line 46229
4 => {
let _ = { let assigned = ((from_surrogate(((c1) as u32), ((c) as u32))) as i32); c = assigned; assigned };
vm_block = 3; continue;
}
// C line 46228
5 => {
vm_block = if (is_hi_surrogate(((c1) as u32))) != 0 { 4 } else { 2 }; continue;
}
// C line 46227
6 => {
let _ = { let assigned = ((*((((*(p)).u).str16).as_mut_ptr()).offset(((idx).wrapping_sub((1 as i32))) as isize)) as i32); c1 = assigned; assigned };
vm_block = 5; continue;
}
// C line 46226
7 => {
vm_block = if (((((is_lo_surrogate(((c) as u32))) != 0) && (((((idx) > ((0 as i32))) as i32)) != 0)) as i32)) != 0 { 6 } else { 2 }; continue;
}
// C line 46225
8 => {
let _ = { let assigned = ((*((((*(p)).u).str16).as_mut_ptr()).offset((idx) as isize)) as i32); c = assigned; assigned };
vm_block = 7; continue;
}
// C line 46234
9 => {
let _ = { let assigned = ((*((((*(p)).u).str8).as_mut_ptr()).offset((idx) as isize)) as i32); c = assigned; assigned };
vm_block = 2; continue;
}
// C line 46224
10 => {
vm_block = if ((*(p)).is_wide_char()) != 0 { 8 } else { 9 }; continue;
}
// C line 46223
11 => {
let _ = { let old = idx; idx = (idx).wrapping_sub(1); old };
vm_block = 10; continue;
}
// C line 46222
12 => {
return (0 as i32);
}
// C line 46221
13 => {
vm_block = if ((((idx) <= ((0 as i32))) as i32)) != 0 { 12 } else { 11 }; continue;
}
// C line 46220
14 => {
let _ = { let assigned = *(pidx); idx = assigned; assigned };
vm_block = 13; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:46240. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn test_final_sigma(mut p: *mut JSString, mut sigma_pos: i32) -> i32 {
let mut vm_local_storage = Vec::<u64>::new();
let mut k: i32 = core::mem::zeroed();
let mut c1: i32 = core::mem::zeroed();
let mut vm_block: usize = 15;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 46265
1 => {
return (!((crate::libunicode::lre_is_cased(((c1) as u32))) != 0) as i32);
}
// C line 46258
2 => {
vm_block = 7; continue;
}
// C line 46263
3 => {
vm_block = 1; continue;
}
// C line 46262
4 => {
vm_block = if ((!((crate::libunicode::lre_is_case_ignorable(((c1) as u32))) != 0) as i32)) != 0 { 3 } else { 2 }; continue;
}
// C line 46261
5 => {
let _ = { let assigned = string_getc(p, core::ptr::addr_of_mut!(k)); c1 = assigned; assigned };
vm_block = 4; continue;
}
// C line 46260
6 => {
return (1 as i32);
}
// C line 46259
7 => {
vm_block = if ((((k) >= ((((*(p)).len()) as i32))) as i32)) != 0 { 6 } else { 5 }; continue;
}
// C line 46257
8 => {
let _ = { let assigned = (sigma_pos).wrapping_add((1 as i32)); k = assigned; assigned };
vm_block = 2; continue;
}
// C line 46253
9 => {
return (0 as i32);
}
// C line 46252
10 => {
vm_block = if ((!((crate::libunicode::lre_is_cased(((c1) as u32))) != 0) as i32)) != 0 { 9 } else { 8 }; continue;
}
// C line 46247
11 => {
vm_block = 14; continue;
}
// C line 46250
12 => {
vm_block = 10; continue;
}
// C line 46249
13 => {
vm_block = if ((!((crate::libunicode::lre_is_case_ignorable(((c1) as u32))) != 0) as i32)) != 0 { 12 } else { 11 }; continue;
}
// C line 46248
14 => {
let _ = { let assigned = string_prevc(p, core::ptr::addr_of_mut!(k)); c1 = assigned; assigned };
vm_block = 13; continue;
}
// C line 46246
15 => {
let _ = { let assigned = sigma_pos; k = assigned; assigned };
vm_block = 11; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:46268. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_string_toLowerCase(mut ctx: *mut JSContext, mut this_val: JSValue, mut argc: i32, mut argv: *mut JSValue, mut to_lower: i32) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut val: JSValue = core::mem::zeroed();
let mut b_s: StringBuffer = core::mem::zeroed();
let mut b: *mut StringBuffer = core::mem::zeroed();
let mut p: *mut JSString = core::mem::zeroed();
let mut i: i32 = core::mem::zeroed();
let mut c: i32 = core::mem::zeroed();
let mut j: i32 = core::mem::zeroed();
let mut l: i32 = core::mem::zeroed();
let mut res: [u32; 3] = core::mem::zeroed();
let mut vm_block: usize = 26;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 46303
1 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 46302
2 => {
let _ = string_buffer_free(b);
vm_block = 1; continue;
}
// C line ? labels: fail
3 => {
let _ = JS_FreeValue(ctx, val);
vm_block = 2; continue;
}
// C line 46299
4 => {
return string_buffer_end(b);
}
// C line 46298
5 => {
let _ = JS_FreeValue(ctx, val);
vm_block = 4; continue;
}
// C line 46285
6 => {
vm_block = if ((((i) < ((((*(p)).len()) as i32))) as i32)) != 0 { 16 } else { 5 }; continue;
}
// C line 46293
7 => {
vm_block = if ((((j) < (l)) as i32)) != 0 { 10 } else { 6 }; continue;
}
// C line ?
8 => {
let _ = { let old = j; j = (j).wrapping_add(1); old };
vm_block = 7; continue;
}
// C line 46295
9 => {
vm_block = 3; continue;
}
// C line 46294
10 => {
vm_block = if (string_buffer_putc(b, *((res).as_mut_ptr()).offset((j) as isize))) != 0 { 9 } else { 8 }; continue;
}
// C line 46293
11 => {
let _ = { let assigned = (0 as i32); j = assigned; assigned };
vm_block = 7; continue;
}
// C line 46289
12 => {
let _ = { let assigned = (1 as i32); l = assigned; assigned };
vm_block = 11; continue;
}
// C line 46288
13 => {
let _ = { let assigned = (((962 as i32)) as u32); *((res).as_mut_ptr()).offset(((0 as i32)) as isize) = assigned; assigned };
vm_block = 12; continue;
}
// C line 46291
14 => {
let _ = { let assigned = crate::libunicode::lre_case_conv(core::slice::from_raw_parts_mut((res).as_mut_ptr(), crate::libunicode_header::LRE_CC_RES_LEN_MAX), ((c) as u32), to_lower); l = assigned; assigned };
vm_block = 11; continue;
}
// C line 46287
15 => {
vm_block = if ((((((((((((c) == ((931 as i32))) as i32)) != 0) && ((to_lower) != 0)) as i32)) != 0) && ((test_final_sigma(p, (i).wrapping_sub((1 as i32)))) != 0)) as i32)) != 0 { 13 } else { 14 }; continue;
}
// C line 46286
16 => {
let _ = { let assigned = string_getc(p, core::ptr::addr_of_mut!(i)); c = assigned; assigned };
vm_block = 15; continue;
}
// C line 46285
17 => {
let _ = { let assigned = (0 as i32); i = assigned; assigned };
vm_block = 6; continue;
}
// C line 46284
18 => {
vm_block = 3; continue;
}
// C line 46283
19 => {
vm_block = if (string_buffer_init(ctx, b, (((*(p)).len()) as i32))) != 0 { 18 } else { 17 }; continue;
}
// C line 46282
20 => {
return val;
}
// C line 46281
21 => {
vm_block = if (((((((*(p)).len()) as i32)) == ((0 as i32))) as i32)) != 0 { 20 } else { 19 }; continue;
}
// C line 46280
22 => {
let _ = { let assigned = ((((val).u).ptr) as *mut JSString); p = assigned; assigned };
vm_block = 21; continue;
}
// C line 46279
23 => {
return val;
}
// C line 46278
24 => {
vm_block = if (JS_IsException(val)) != 0 { 23 } else { 22 }; continue;
}
// C line 46277
25 => {
let _ = { let assigned = JS_ToStringCheckObject(ctx, this_val); val = assigned; assigned };
vm_block = 24; continue;
}
// C line 46272
26 => {
b = core::ptr::addr_of_mut!(b_s);
vm_block = 25; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:46309. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn JS_ToUTF32String(mut ctx: *mut JSContext, mut pbuf: *mut *mut u32, mut val1: JSValue) -> i32 {
let mut vm_local_storage = Vec::<u64>::new();
let mut val: JSValue = core::mem::zeroed();
let mut p: *mut JSString = core::mem::zeroed();
let mut buf: *mut u32 = core::mem::zeroed();
let mut i: i32 = core::mem::zeroed();
let mut j: i32 = core::mem::zeroed();
let mut len: i32 = core::mem::zeroed();
let mut vm_block: usize = 17;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 46334
1 => {
return ((1 as i32)).wrapping_neg();
}
// C line ? labels: fail
2 => {
let _ = { let assigned = core::ptr::null_mut::<u32>(); *(pbuf) = assigned; assigned };
vm_block = 1; continue;
}
// C line 46331
3 => {
return j;
}
// C line 46330
4 => {
let _ = { let assigned = buf; *(pbuf) = assigned; assigned };
vm_block = 3; continue;
}
// C line 46329
5 => {
let _ = JS_FreeValue(ctx, val);
vm_block = 4; continue;
}
// C line 46327
6 => {
vm_block = if ((((i) < (len)) as i32)) != 0 { 7 } else { 5 }; continue;
}
// C line 46328
7 => {
let _ = { let assigned = ((string_getc(p, core::ptr::addr_of_mut!(i))) as u32); *(buf).offset(({ let old = j; j = (j).wrapping_add(1); old }) as isize) = assigned; assigned };
vm_block = 6; continue;
}
// C line 46327
8 => {
let _ = { let assigned = { let assigned = (0 as i32); j = assigned; assigned }; i = assigned; assigned };
vm_block = 6; continue;
}
// C line 46325
9 => {
vm_block = 2; continue;
}
// C line 46324
10 => {
let _ = JS_FreeValue(ctx, val);
vm_block = 9; continue;
}
// C line 46323
11 => {
vm_block = if ((!(!(buf).is_null()) as i32)) != 0 { 10 } else { 8 }; continue;
}
// C line 46322
12 => {
let _ = { let assigned = ((js_malloc(ctx, ((size_of::<u32>() as usize)).wrapping_mul(((crate::cutils_header::max_int(len, (1 as i32))) as usize)))) as *mut u32); buf = assigned; assigned };
vm_block = 11; continue;
}
// C line 46320
13 => {
let _ = { let assigned = (((*(p)).len()) as i32); len = assigned; assigned };
vm_block = 12; continue;
}
// C line 46319
14 => {
let _ = { let assigned = ((((val).u).ptr) as *mut JSString); p = assigned; assigned };
vm_block = 13; continue;
}
// C line 46318
15 => {
return ((1 as i32)).wrapping_neg();
}
// C line 46317
16 => {
vm_block = if (JS_IsException(val)) != 0 { 15 } else { 14 }; continue;
}
// C line 46316
17 => {
let _ = { let assigned = JS_ToString(ctx, val1); val = assigned; assigned };
vm_block = 16; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:46337. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn JS_NewUTF32String(mut ctx: *mut JSContext, mut buf: *const u32, mut len: i32) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut i: i32 = core::mem::zeroed();
let mut b_s: StringBuffer = core::mem::zeroed();
let mut b: *mut StringBuffer = core::mem::zeroed();
let mut vm_block: usize = 11;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 46350
1 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line ? labels: fail
2 => {
let _ = string_buffer_free(b);
vm_block = 1; continue;
}
// C line 46347
3 => {
return string_buffer_end(b);
}
// C line 46343
4 => {
vm_block = if ((((i) < (len)) as i32)) != 0 { 7 } else { 3 }; continue;
}
// C line ?
5 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 4; continue;
}
// C line 46345
6 => {
vm_block = 2; continue;
}
// C line 46344
7 => {
vm_block = if (string_buffer_putc(b, *(buf).offset((i) as isize))) != 0 { 6 } else { 5 }; continue;
}
// C line 46343
8 => {
let _ = { let assigned = (0 as i32); i = assigned; assigned };
vm_block = 4; continue;
}
// C line 46342
9 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 46341
10 => {
vm_block = if (string_buffer_init(ctx, b, len)) != 0 { 9 } else { 8 }; continue;
}
// C line 46340
11 => {
b = core::ptr::addr_of_mut!(b_s);
vm_block = 10; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:46353. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_string_normalize1(mut ctx: *mut JSContext, mut pout_buf: *mut *mut u32, mut val: JSValue, mut n_type: crate::libunicode_header::UnicodeNormalizationEnum) -> i32 {
let mut vm_local_storage = Vec::<u64>::new();
let mut buf_len: i32 = core::mem::zeroed();
let mut out_len: i32 = core::mem::zeroed();
let mut buf: *mut u32 = core::mem::zeroed();
let mut out_buf: *mut u32 = core::mem::zeroed();
let mut vm_block: usize = 9;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 46369
1 => {
return out_len;
}
// C line 46368
2 => {
let _ = { let assigned = out_buf; *(pout_buf) = assigned; assigned };
vm_block = 1; continue;
}
// C line 46367
3 => {
return ((1 as i32)).wrapping_neg();
}
// C line 46366
4 => {
vm_block = if ((((out_len) < ((0 as i32))) as i32)) != 0 { 3 } else { 2 }; continue;
}
// C line 46365
5 => {
let _ = js_free(ctx, ((buf) as *mut c_void));
vm_block = 4; continue;
}
// C line 46363
6 => {
let _ = { let assigned = crate::libunicode::unicode_normalize(core::ptr::addr_of_mut!(out_buf), buf, buf_len, n_type, (((*(ctx)).rt) as *mut c_void), Some(js_string_normalize_realloc)); out_len = assigned; assigned };
vm_block = 5; continue;
}
// C line 46362
7 => {
return ((1 as i32)).wrapping_neg();
}
// C line 46361
8 => {
vm_block = if ((((buf_len) < ((0 as i32))) as i32)) != 0 { 7 } else { 6 }; continue;
}
// C line 46360
9 => {
let _ = { let assigned = JS_ToUTF32String(ctx, core::ptr::addr_of_mut!(buf), val); buf_len = assigned; assigned };
vm_block = 8; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:46372. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_string_normalize(mut ctx: *mut JSContext, mut this_val: JSValue, mut argc: i32, mut argv: *mut JSValue) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut form: *const c_char = core::mem::zeroed();
let mut p: *const c_char = core::mem::zeroed();
let mut form_len: usize = core::mem::zeroed();
let mut is_compat: i32 = core::mem::zeroed();
let mut out_len: i32 = core::mem::zeroed();
let mut n_type: crate::libunicode_header::UnicodeNormalizationEnum = core::mem::zeroed();
let mut val: JSValue = core::mem::zeroed();
let mut out_buf: *mut u32 = core::mem::zeroed();
let mut vm_block: usize = 32;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 46422
1 => {
return val;
}
// C line 46421
2 => {
let _ = js_free(ctx, ((out_buf) as *mut c_void));
vm_block = 1; continue;
}
// C line 46420
3 => {
let _ = { let assigned = JS_NewUTF32String(ctx, out_buf, out_len); val = assigned; assigned };
vm_block = 2; continue;
}
// C line 46419
4 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 46418
5 => {
vm_block = if ((((out_len) < ((0 as i32))) as i32)) != 0 { 4 } else { 3 }; continue;
}
// C line 46417
6 => {
let _ = JS_FreeValue(ctx, val);
vm_block = 5; continue;
}
// C line 46416
7 => {
let _ = { let assigned = js_string_normalize1(ctx, core::ptr::addr_of_mut!(out_buf), val, n_type); out_len = assigned; assigned };
vm_block = 6; continue;
}
// C line 46387
8 => {
let _ = { let assigned = (((crate::libunicode_header::UNICODE_NFC as i32)) as crate::libunicode_header::UnicodeNormalizationEnum); n_type = assigned; assigned };
vm_block = 7; continue;
}
// C line 46413
9 => {
let _ = JS_FreeCString(ctx, form);
vm_block = 7; continue;
}
// C line 46404
10 => {
vm_block = 16; continue;
}
// C line 46403
11 => {
vm_block = if (((((((((p).offset((((1 as i32)) as isize))).offset_from(form) as i64)) as usize)) != (form_len)) as i32)) != 0 { 10 } else { 9 }; continue;
}
// C line 46402
12 => {
let _ = { let assigned = (((((crate::libunicode_header::UNICODE_NFC as i32)).wrapping_add((is_compat).wrapping_mul((2 as i32)))).wrapping_add((((*(p)) as i32)).wrapping_sub((67 as i32)))) as crate::libunicode_header::UnicodeNormalizationEnum); n_type = assigned; assigned };
vm_block = 11; continue;
}
// C line 46411
13 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line ? labels: fail1
14 => {
let _ = JS_FreeValue(ctx, val);
vm_block = 13; continue;
}
// C line 46408
15 => {
let _ = JS_ThrowRangeError(ctx, c"bad normalization form".as_ptr());
vm_block = 14; continue;
}
// C line ? labels: bad_form
16 => {
let _ = JS_FreeCString(ctx, form);
vm_block = 15; continue;
}
// C line 46401
17 => {
vm_block = if ((((((((((*(p)) as i32)) == ((67 as i32))) as i32)) != 0) || (((((((*(p)) as i32)) == ((68 as i32))) as i32)) != 0)) as i32)) != 0 { 12 } else { 16 }; continue;
}
// C line 46399
18 => {
let _ = { let old = p; p = (p).offset(1); old };
vm_block = 17; continue;
}
// C line 46398
19 => {
let _ = { let assigned = (1 as i32); is_compat = assigned; assigned };
vm_block = 18; continue;
}
// C line 46397
20 => {
vm_block = if ((((((*(p)) as i32)) == ((75 as i32))) as i32)) != 0 { 19 } else { 17 }; continue;
}
// C line 46396
21 => {
let _ = { let assigned = (0 as i32); is_compat = assigned; assigned };
vm_block = 20; continue;
}
// C line 46395
22 => {
let _ = { p = (p).offset((((2 as i32)) as isize)); p };
vm_block = 21; continue;
}
// C line 46394
23 => {
vm_block = 16; continue;
}
// C line 46393
24 => {
vm_block = if ((((((((((*(p).offset(((0 as i32)) as isize)) as i32)) != ((78 as i32))) as i32)) != 0) || (((((((*(p).offset(((1 as i32)) as isize)) as i32)) != ((70 as i32))) as i32)) != 0)) as i32)) != 0 { 23 } else { 22 }; continue;
}
// C line 46392
25 => {
let _ = { let assigned = form; p = assigned; assigned };
vm_block = 24; continue;
}
// C line 46391
26 => {
vm_block = 14; continue;
}
// C line 46390
27 => {
vm_block = if ((!(!(form).is_null()) as i32)) != 0 { 26 } else { 25 }; continue;
}
// C line 46389
28 => {
let _ = { let assigned = JS_ToCStringLen(ctx, core::ptr::addr_of_mut!(form_len), *(argv).offset(((0 as i32)) as isize)); form = assigned; assigned };
vm_block = 27; continue;
}
// C line 46386
29 => {
vm_block = if ((((((((argc) == ((0 as i32))) as i32)) != 0) || ((JS_IsUndefined(*(argv).offset(((0 as i32)) as isize))) != 0)) as i32)) != 0 { 8 } else { 28 }; continue;
}
// C line 46384
30 => {
return val;
}
// C line 46383
31 => {
vm_block = if (JS_IsException(val)) != 0 { 30 } else { 29 }; continue;
}
// C line 46382
32 => {
let _ = { let assigned = JS_ToStringCheckObject(ctx, this_val); val = assigned; assigned };
vm_block = 31; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:46426. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_UTF32_compare(mut buf1: *const u32, mut buf1_len: i32, mut buf2: *const u32, mut buf2_len: i32) -> i32 {
let mut vm_local_storage = Vec::<u64>::new();
let mut i: i32 = core::mem::zeroed();
let mut len: i32 = core::mem::zeroed();
let mut c: i32 = core::mem::zeroed();
let mut res: i32 = core::mem::zeroed();
let mut vm_block: usize = 13;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 46443
1 => {
return res;
}
// C line 46438
2 => {
let _ = { let assigned = (0 as i32); res = assigned; assigned };
vm_block = 1; continue;
}
// C line 46440
3 => {
let _ = { let assigned = ((1 as i32)).wrapping_neg(); res = assigned; assigned };
vm_block = 1; continue;
}
// C line 46442
4 => {
let _ = { let assigned = (1 as i32); res = assigned; assigned };
vm_block = 1; continue;
}
// C line 46439
5 => {
vm_block = if ((((buf1_len) < (buf2_len)) as i32)) != 0 { 3 } else { 4 }; continue;
}
// C line 46437
6 => {
vm_block = if ((((buf1_len) == (buf2_len)) as i32)) != 0 { 2 } else { 5 }; continue;
}
// C line 46431
7 => {
vm_block = if ((((i) < (len)) as i32)) != 0 { 11 } else { 6 }; continue;
}
// C line ?
8 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 7; continue;
}
// C line 46435
9 => {
return c;
}
// C line 46434
10 => {
vm_block = if ((((c) != ((0 as i32))) as i32)) != 0 { 9 } else { 8 }; continue;
}
// C line 46433
11 => {
let _ = { let assigned = (((*(buf1).offset((i) as isize)).wrapping_sub(*(buf2).offset((i) as isize))) as i32); c = assigned; assigned };
vm_block = 10; continue;
}
// C line 46431
12 => {
let _ = { let assigned = (0 as i32); i = assigned; assigned };
vm_block = 7; continue;
}
// C line 46430
13 => {
let _ = { let assigned = crate::cutils_header::min_int(buf1_len, buf2_len); len = assigned; assigned };
vm_block = 12; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:46446. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_string_localeCompare(mut ctx: *mut JSContext, mut this_val: JSValue, mut argc: i32, mut argv: *mut JSValue) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut a: JSValue = core::mem::zeroed();
let mut b: JSValue = core::mem::zeroed();
let mut cmp: i32 = core::mem::zeroed();
let mut a_len: i32 = core::mem::zeroed();
let mut b_len: i32 = core::mem::zeroed();
let mut a_buf: *mut u32 = core::mem::zeroed();
let mut b_buf: *mut u32 = core::mem::zeroed();
let mut vm_block: usize = 21;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 46477
1 => {
return JS_NewInt32(ctx, cmp);
}
// C line 46476
2 => {
let _ = js_free(ctx, ((b_buf) as *mut c_void));
vm_block = 1; continue;
}
// C line 46475
3 => {
let _ = js_free(ctx, ((a_buf) as *mut c_void));
vm_block = 2; continue;
}
// C line 46474
4 => {
let _ = { let assigned = js_UTF32_compare(a_buf, a_len, b_buf, b_len); cmp = assigned; assigned };
vm_block = 3; continue;
}
// C line 46472
5 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 46471
6 => {
let _ = js_free(ctx, ((a_buf) as *mut c_void));
vm_block = 5; continue;
}
// C line 46470
7 => {
vm_block = if ((((b_len) < ((0 as i32))) as i32)) != 0 { 6 } else { 4 }; continue;
}
// C line 46469
8 => {
let _ = JS_FreeValue(ctx, b);
vm_block = 7; continue;
}
// C line 46468
9 => {
let _ = { let assigned = js_string_normalize1(ctx, core::ptr::addr_of_mut!(b_buf), b, (((crate::libunicode_header::UNICODE_NFC as i32)) as crate::libunicode_header::UnicodeNormalizationEnum)); b_len = assigned; assigned };
vm_block = 8; continue;
}
// C line 46465
10 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 46464
11 => {
let _ = JS_FreeValue(ctx, b);
vm_block = 10; continue;
}
// C line 46463
12 => {
vm_block = if ((((a_len) < ((0 as i32))) as i32)) != 0 { 11 } else { 9 }; continue;
}
// C line 46462
13 => {
let _ = JS_FreeValue(ctx, a);
vm_block = 12; continue;
}
// C line 46461
14 => {
let _ = { let assigned = js_string_normalize1(ctx, core::ptr::addr_of_mut!(a_buf), a, (((crate::libunicode_header::UNICODE_NFC as i32)) as crate::libunicode_header::UnicodeNormalizationEnum)); a_len = assigned; assigned };
vm_block = 13; continue;
}
// C line 46459
15 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 46458
16 => {
let _ = JS_FreeValue(ctx, a);
vm_block = 15; continue;
}
// C line 46457
17 => {
vm_block = if (JS_IsException(b)) != 0 { 16 } else { 14 }; continue;
}
// C line 46456
18 => {
let _ = { let assigned = JS_ToString(ctx, *(argv).offset(((0 as i32)) as isize)); b = assigned; assigned };
vm_block = 17; continue;
}
// C line 46455
19 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 46454
20 => {
vm_block = if (JS_IsException(a)) != 0 { 19 } else { 18 }; continue;
}
// C line 46453
21 => {
let _ = { let assigned = JS_ToStringCheckObject(ctx, this_val); a = assigned; assigned };
vm_block = 20; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:46502. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_string_toString(mut ctx: *mut JSContext, mut this_val: JSValue, mut argc: i32, mut argv: *mut JSValue) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut vm_block: usize = 1;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 46505
1 => {
return js_thisStringValue(ctx, this_val);
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:46510. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_string_iterator_next(mut ctx: *mut JSContext, mut this_val: JSValue, mut argc: i32, mut argv: *mut JSValue, mut pdone: *mut i32, mut magic: i32) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut it: *mut JSArrayIteratorData = core::mem::zeroed();
let mut idx: u32 = core::mem::zeroed();
let mut c: u32 = core::mem::zeroed();
let mut start: u32 = core::mem::zeroed();
let mut p: *mut JSString = core::mem::zeroed();
let mut vm_block: usize = 20;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 46540
1 => {
return js_new_string_char(ctx, ((c) as u16));
}
// C line 46542
2 => {
return js_new_string16_len(ctx, ((((*(p)).u).str16).as_mut_ptr()).offset(((start) as isize)), (2 as i32));
}
// C line 46539
3 => {
vm_block = if ((((c) <= ((((65535 as i32)) as u32))) as i32)) != 0 { 1 } else { 2 }; continue;
}
// C line 46538
4 => {
let _ = { let assigned = (0 as i32); *(pdone) = assigned; assigned };
vm_block = 3; continue;
}
// C line 46537
5 => {
let _ = { let assigned = idx; (*(it)).idx = assigned; assigned };
vm_block = 4; continue;
}
// C line 46536
6 => {
let _ = { let assigned = ((string_getc(p, ((core::ptr::addr_of_mut!(idx)) as *mut i32))) as u32); c = assigned; assigned };
vm_block = 5; continue;
}
// C line 46535
7 => {
let _ = { let assigned = idx; start = assigned; assigned };
vm_block = 6; continue;
}
// C line 46532
8 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) };
}
// C line ? labels: done
9 => {
let _ = { let assigned = (1 as i32); *(pdone) = assigned; assigned };
vm_block = 8; continue;
}
// C line 46529
10 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; (*(it)).obj = assigned; assigned };
vm_block = 9; continue;
}
// C line 46528
11 => {
let _ = JS_FreeValue(ctx, (*(it)).obj);
vm_block = 10; continue;
}
// C line 46527
12 => {
vm_block = if ((((idx) >= ((*(p)).len())) as i32)) != 0 { 11 } else { 7 }; continue;
}
// C line 46526
13 => {
let _ = { let assigned = (*(it)).idx; idx = assigned; assigned };
vm_block = 12; continue;
}
// C line 46525
14 => {
let _ = { let assigned = (((((*(it)).obj).u).ptr) as *mut JSString); p = assigned; assigned };
vm_block = 13; continue;
}
// C line 46524
15 => {
vm_block = 9; continue;
}
// C line 46523
16 => {
vm_block = if (JS_IsUndefined((*(it)).obj)) != 0 { 15 } else { 14 }; continue;
}
// C line 46521
17 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 46520
18 => {
let _ = { let assigned = (0 as i32); *(pdone) = assigned; assigned };
vm_block = 17; continue;
}
// C line 46519
19 => {
vm_block = if ((!(!(it).is_null()) as i32)) != 0 { 18 } else { 16 }; continue;
}
// C line 46518
20 => {
let _ = { let assigned = ((JS_GetOpaque2(ctx, this_val, (((JS_CLASS_STRING_ITERATOR as i32)) as JSClassID))) as *mut JSArrayIteratorData); it = assigned; assigned };
vm_block = 19; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:46689. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
pub unsafe fn JS_AddIntrinsicStringNormalize(mut ctx: *mut JSContext) -> i32 {
let mut vm_local_storage = Vec::<u64>::new();
let mut vm_block: usize = 1;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 46691
1 => {
return JS_SetPropertyFunctionList(ctx, *((*(ctx)).class_proto).offset(((JS_CLASS_STRING as i32)) as isize), (js_string_proto_normalize).as_ptr(), (((((size_of::<[JSCFunctionListEntry; 2]>() as usize)) / ((size_of::<JSCFunctionListEntry>() as usize)))) as i32));
}
_ => std::process::abort(),
} }
}
