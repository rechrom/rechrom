// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:55955. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn JS_ToBigIntCtorFree(mut ctx: *mut JSContext, mut val: JSValue) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut tag: u32 = core::mem::zeroed();
let mut d: f64 = core::mem::zeroed();
let mut r: *mut JSBigInt = core::mem::zeroed();
let mut res: i32 = core::mem::zeroed();
let mut vm_block: usize = 24;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 56002
1 => {
return val;
}
// C line 56000
2 => {
return JS_ThrowTypeError(ctx, c"cannot convert to BigInt".as_ptr());
}
// C line ?
3 => {
let _ = JS_FreeValue(ctx, val);
vm_block = 2; continue;
}
// C line 55995
4 => {
vm_block = 24; continue;
}
// C line 55994
5 => {
vm_block = 1; continue;
}
// C line 55993
6 => {
vm_block = if (JS_IsException(val)) != 0 { 5 } else { 4 }; continue;
}
// C line 55992
7 => {
let _ = { let assigned = JS_ToPrimitiveFree(ctx, val, (1 as i32)); val = assigned; assigned };
vm_block = 6; continue;
}
// C line 55990
8 => {
vm_block = 1; continue;
}
// C line 55989
9 => {
let _ = { let assigned = JS_StringToBigIntErr(ctx, val); val = assigned; assigned };
vm_block = 8; continue;
}
// C line 55986
10 => {
vm_block = 1; continue;
}
// C line 55977
11 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) }; val = assigned; assigned };
vm_block = 10; continue;
}
// C line 55979
12 => {
let _ = { let assigned = JS_ThrowRangeError(ctx, c"cannot convert to BigInt: not an integer".as_ptr()); val = assigned; assigned };
vm_block = 10; continue;
}
// C line ?
13 => {
let _ = { let assigned = JS_ThrowRangeError(ctx, c"cannot convert NaN or Infinity to BigInt".as_ptr()); val = assigned; assigned };
vm_block = 10; continue;
}
// C line 55978
14 => {
vm_block = if ((((res) == ((1 as i32))) as i32)) != 0 { 12 } else { 13 }; continue;
}
// C line 55976
15 => {
vm_block = if ((((res) == ((0 as i32))) as i32)) != 0 { 11 } else { 14 }; continue;
}
// C line 55983
16 => {
let _ = { let assigned = JS_CompactBigInt(ctx, r); val = assigned; assigned };
vm_block = 10; continue;
}
// C line 55975
17 => {
vm_block = if ((!(!(r).is_null()) as i32)) != 0 { 15 } else { 16 }; continue;
}
// C line 55974
18 => {
let _ = { let assigned = js_bigint_from_float64(ctx, core::ptr::addr_of_mut!(res), d); r = assigned; assigned };
vm_block = 17; continue;
}
// C line 55971
19 => {
d = ((val).u).float64;
vm_block = 18; continue;
}
// C line 55968
20 => {
vm_block = 1; continue;
}
// C line 55965
21 => {
vm_block = 1; continue;
}
// C line 55964
22 => {
let _ = { let assigned = JS_NewBigInt64(ctx, ((((((val).u).uint64) as i32)) as i64)); val = assigned; assigned };
vm_block = 21; continue;
}
// C line 55961
23 => {
vm_block = match tag { x if x == (((JS_TAG_UNDEFINED as i32)) as u32) => 3, x if x == (((JS_TAG_NULL as i32)) as u32) => 3, x if x == (((JS_TAG_OBJECT as i32)) as u32) => 7, x if x == (((JS_TAG_STRING_ROPE as i32)) as u32) => 9, x if x == (((JS_TAG_STRING as i32)) as u32) => 9, x if x == (((JS_TAG_FLOAT64 as i32)) as u32) => 19, x if x == (((JS_TAG_BIG_INT as i32)) as u32) => 20, x if x == (((JS_TAG_SHORT_BIG_INT as i32)) as u32) => 20, x if x == (((JS_TAG_BOOL as i32)) as u32) => 22, x if x == (((JS_TAG_INT as i32)) as u32) => 22, _ => 3, }; continue;
}
// C line ? labels: redo
24 => {
let _ = { let assigned = (((((val).tag) as i32)) as u32); tag = assigned; assigned };
vm_block = 23; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:56005. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_bigint_constructor(mut ctx: *mut JSContext, mut new_target: JSValue, mut argc: i32, mut argv: *mut JSValue) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut vm_block: usize = 3;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 56011
1 => {
return JS_ToBigIntCtorFree(ctx, JS_DupValue(ctx, *(argv).offset(((0 as i32)) as isize)));
}
// C line 56010
2 => {
return JS_ThrowTypeErrorNotAConstructor(ctx, new_target);
}
// C line 56009
3 => {
vm_block = if ((!((JS_IsUndefined(new_target)) != 0) as i32)) != 0 { 2 } else { 1 }; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:56014. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_thisBigIntValue(mut ctx: *mut JSContext, mut this_val: JSValue) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut p: *mut JSObject = core::mem::zeroed();
let mut vm_block: usize = 8;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 56026
1 => {
return JS_ThrowTypeError(ctx, c"not a BigInt".as_ptr());
}
// C line 56023
2 => {
return JS_DupValue(ctx, ((*(p)).u).object_data);
}
// C line 56022
3 => {
vm_block = if (JS_IsBigInt(ctx, ((*(p)).u).object_data)) != 0 { 2 } else { 1 }; continue;
}
// C line 56021
4 => {
vm_block = if (((((((*(p)).class_id) as i32)) == ((JS_CLASS_BIG_INT as i32))) as i32)) != 0 { 3 } else { 1 }; continue;
}
// C line 56020
5 => {
p = ((((this_val).u).ptr) as *mut JSObject);
vm_block = 4; continue;
}
// C line 56019
6 => {
vm_block = if (((((((this_val).tag) as i32)) == ((JS_TAG_OBJECT as i32))) as i32)) != 0 { 5 } else { 1 }; continue;
}
// C line 56017
7 => {
return JS_DupValue(ctx, this_val);
}
// C line 56016
8 => {
vm_block = if (JS_IsBigInt(ctx, this_val)) != 0 { 7 } else { 6 }; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:56029. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_bigint_toString(mut ctx: *mut JSContext, mut this_val: JSValue, mut argc: i32, mut argv: *mut JSValue) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut val: JSValue = core::mem::zeroed();
let mut base: i32 = core::mem::zeroed();
let mut ret: JSValue = core::mem::zeroed();
let mut vm_block: usize = 13;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 56051
1 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line ? labels: fail
2 => {
let _ = JS_FreeValue(ctx, val);
vm_block = 1; continue;
}
// C line 56048
3 => {
return ret;
}
// C line 56047
4 => {
let _ = JS_FreeValue(ctx, val);
vm_block = 3; continue;
}
// C line 56046
5 => {
let _ = { let assigned = js_bigint_to_string1(ctx, val, base); ret = assigned; assigned };
vm_block = 4; continue;
}
// C line 56040
6 => {
let _ = { let assigned = (10 as i32); base = assigned; assigned };
vm_block = 5; continue;
}
// C line 56044
7 => {
vm_block = 2; continue;
}
// C line 56043
8 => {
vm_block = if ((((base) < ((0 as i32))) as i32)) != 0 { 7 } else { 5 }; continue;
}
// C line 56042
9 => {
let _ = { let assigned = js_get_radix(ctx, *(argv).offset(((0 as i32)) as isize)); base = assigned; assigned };
vm_block = 8; continue;
}
// C line 56039
10 => {
vm_block = if ((((((((argc) == ((0 as i32))) as i32)) != 0) || ((JS_IsUndefined(*(argv).offset(((0 as i32)) as isize))) != 0)) as i32)) != 0 { 6 } else { 9 }; continue;
}
// C line 56038
11 => {
return val;
}
// C line 56037
12 => {
vm_block = if (JS_IsException(val)) != 0 { 11 } else { 10 }; continue;
}
// C line 56036
13 => {
let _ = { let assigned = js_thisBigIntValue(ctx, this_val); val = assigned; assigned };
vm_block = 12; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:56054. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_bigint_valueOf(mut ctx: *mut JSContext, mut this_val: JSValue, mut argc: i32, mut argv: *mut JSValue) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut vm_block: usize = 1;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 56057
1 => {
return js_thisBigIntValue(ctx, this_val);
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:56060. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_bigint_asUintN(mut ctx: *mut JSContext, mut this_val: JSValue, mut argc: i32, mut argv: *mut JSValue, mut asIntN: i32) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut bits: u64 = core::mem::zeroed();
let mut res: JSValue = core::mem::zeroed();
let mut a: JSValue = core::mem::zeroed();
let mut v: u64 = core::mem::zeroed();
let mut shift: i32 = core::mem::zeroed();
let mut r: *mut JSBigInt = core::mem::zeroed();
let mut p: *mut JSBigInt = core::mem::zeroed();
let mut len: i32 = core::mem::zeroed();
let mut shift_1: i32 = core::mem::zeroed();
let mut i: i32 = core::mem::zeroed();
let mut v_1: js_limb_t = core::mem::zeroed();
let mut vm_block: usize = 41;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 56120
1 => {
return res;
}
// C line 56074
2 => {
let _ = { let assigned = __JS_NewShortBigInt(ctx, (((0 as i32)) as i64)); res = assigned; assigned };
vm_block = 1; continue;
}
// C line 56073
3 => {
let _ = JS_FreeValue(ctx, a);
vm_block = 2; continue;
}
// C line 56078
4 => {
let _ = { let assigned = a; res = assigned; assigned };
vm_block = 1; continue;
}
// C line 56089
5 => {
let _ = { let assigned = __JS_NewShortBigInt(ctx, ((v) as i64)); res = assigned; assigned };
vm_block = 1; continue;
}
// C line 56086
6 => {
let _ = { let assigned = (((((v) as i64)).wrapping_shr((shift) as u32)) as u64); v = assigned; assigned };
vm_block = 5; continue;
}
// C line 56088
7 => {
let _ = { let assigned = (v).wrapping_shr((shift) as u32); v = assigned; assigned };
vm_block = 5; continue;
}
// C line 56085
8 => {
vm_block = if (asIntN) != 0 { 6 } else { 7 }; continue;
}
// C line 56084
9 => {
let _ = { let assigned = (v).wrapping_shl((shift) as u32); v = assigned; assigned };
vm_block = 8; continue;
}
// C line 56083
10 => {
let _ = { let assigned = ((((a).u).short_big_int) as u64); v = assigned; assigned };
vm_block = 9; continue;
}
// C line 56082
11 => {
let _ = { let assigned = ((((((64 as i32)) as u64)).wrapping_sub(bits)) as i32); shift = assigned; assigned };
vm_block = 10; continue;
}
// C line 56077
12 => {
vm_block = if ((((bits) >= ((((64 as i32)) as u64))) as i32)) != 0 { 4 } else { 11 }; continue;
}
// C line 56094
13 => {
let _ = { let assigned = a; res = assigned; assigned };
vm_block = 1; continue;
}
// C line 56117
14 => {
let _ = { let assigned = JS_CompactBigInt(ctx, r); res = assigned; assigned };
vm_block = 1; continue;
}
// C line 56116
15 => {
let _ = JS_FreeValue(ctx, a);
vm_block = 14; continue;
}
// C line 56115
16 => {
let _ = { let assigned = js_bigint_normalize(ctx, r); r = assigned; assigned };
vm_block = 15; continue;
}
// C line 56114
17 => {
let _ = { let assigned = v_1; *(((*(r)).tab).as_mut_ptr()).offset(((len).wrapping_sub((1 as i32))) as isize) = assigned; assigned };
vm_block = 16; continue;
}
// C line 56111
18 => {
let _ = { let assigned = (((((v_1) as js_slimb_t)).wrapping_shr((shift_1) as u32)) as js_limb_t); v_1 = assigned; assigned };
vm_block = 17; continue;
}
// C line 56113
19 => {
let _ = { let assigned = (v_1).wrapping_shr((shift_1) as u32); v_1 = assigned; assigned };
vm_block = 17; continue;
}
// C line 56110
20 => {
vm_block = if (asIntN) != 0 { 18 } else { 19 }; continue;
}
// C line 56109
21 => {
let _ = { let assigned = (*(((*(p)).tab).as_mut_ptr()).offset(((len).wrapping_sub((1 as i32))) as isize)).wrapping_shl((shift_1) as u32); v_1 = assigned; assigned };
vm_block = 20; continue;
}
// C line 56107
22 => {
let _ = { let assigned = (((((bits).wrapping_neg()) & (((((64 as i32)).wrapping_sub((1 as i32))) as u64)))) as i32); shift_1 = assigned; assigned };
vm_block = 21; continue;
}
// C line 56105
23 => {
vm_block = if ((((i) < ((len).wrapping_sub((1 as i32)))) as i32)) != 0 { 25 } else { 22 }; continue;
}
// C line ?
24 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 23; continue;
}
// C line 56106
25 => {
let _ = { let assigned = *(((*(p)).tab).as_mut_ptr()).offset((i) as isize); *(((*(r)).tab).as_mut_ptr()).offset((i) as isize) = assigned; assigned };
vm_block = 24; continue;
}
// C line 56105
26 => {
let _ = { let assigned = (0 as i32); i = assigned; assigned };
vm_block = 23; continue;
}
// C line 56104
27 => {
let _ = { let assigned = ((len) as u32); (*(r)).len = assigned; assigned };
vm_block = 26; continue;
}
// C line 56102
28 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 56101
29 => {
let _ = JS_FreeValue(ctx, a);
vm_block = 28; continue;
}
// C line 56100
30 => {
vm_block = if ((!(!(r).is_null()) as i32)) != 0 { 29 } else { 27 }; continue;
}
// C line 56099
31 => {
let _ = { let assigned = js_bigint_new(ctx, len); r = assigned; assigned };
vm_block = 30; continue;
}
// C line 56098
32 => {
let _ = { let assigned = ((((((bits).wrapping_add((((64 as i32)) as u64))).wrapping_sub((((1 as i32)) as u64))) / ((((64 as i32)) as u64)))) as i32); len = assigned; assigned };
vm_block = 31; continue;
}
// C line 56093
33 => {
vm_block = if ((((bits) >= (((((*(p)).len).wrapping_mul((((64 as i32)) as u32))) as u64))) as i32)) != 0 { 13 } else { 32 }; continue;
}
// C line 56092
34 => {
p = ((((a).u).ptr) as *mut JSBigInt);
vm_block = 33; continue;
}
// C line 56075
35 => {
vm_block = if (((((((a).tag) as i32)) == ((JS_TAG_SHORT_BIG_INT as i32))) as i32)) != 0 { 12 } else { 34 }; continue;
}
// C line 56072
36 => {
vm_block = if ((((bits) == ((((0 as i32)) as u64))) as i32)) != 0 { 3 } else { 35 }; continue;
}
// C line 56071
37 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 56070
38 => {
vm_block = if (JS_IsException(a)) != 0 { 37 } else { 36 }; continue;
}
// C line 56069
39 => {
let _ = { let assigned = JS_ToBigInt(ctx, *(argv).offset(((1 as i32)) as isize)); a = assigned; assigned };
vm_block = 38; continue;
}
// C line 56068
40 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 56067
41 => {
vm_block = if (JS_ToIndex(ctx, core::ptr::addr_of_mut!(bits), *(argv).offset(((0 as i32)) as isize))) != 0 { 40 } else { 39 }; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:56134. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn JS_AddIntrinsicBigInt(mut ctx: *mut JSContext) -> i32 {
let mut vm_local_storage = Vec::<u64>::new();
let mut obj1: JSValue = core::mem::zeroed();
let mut vm_block: usize = 5;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 56147
1 => {
return (0 as i32);
}
// C line 56146
2 => {
let _ = JS_FreeValue(ctx, obj1);
vm_block = 1; continue;
}
// C line 56145
3 => {
return ((1 as i32)).wrapping_neg();
}
// C line 56144
4 => {
vm_block = if (JS_IsException(obj1)) != 0 { 3 } else { 2 }; continue;
}
// C line 56138
5 => {
let _ = { let assigned = JS_NewCConstructor(ctx, (JS_CLASS_BIG_INT as i32), c"BigInt".as_ptr(), Some(js_bigint_constructor), (1 as i32), (((JS_CFUNC_constructor_or_func as i32)) as JSCFunctionEnum), (0 as i32), JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }, ptr::addr_of!(js_bigint_funcs).cast::<JSCFunctionListEntry>(), (((((size_of::<[JSCFunctionListEntry; 2]>() as usize)) / ((size_of::<JSCFunctionListEntry>() as usize)))) as i32), ptr::addr_of!(js_bigint_proto_funcs).cast::<JSCFunctionListEntry>(), (((((size_of::<[JSCFunctionListEntry; 3]>() as usize)) / ((size_of::<JSCFunctionListEntry>() as usize)))) as i32), (0 as i32)); obj1 = assigned; assigned };
vm_block = 4; continue;
}
_ => std::process::abort(),
} }
}
