// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:39769. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_global_isNaN(mut ctx: *mut JSContext, mut this_val: JSValue, mut argc: i32, mut argv: *mut JSValue) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut d: f64 = core::mem::zeroed();
let mut vm_block: usize = 3;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 39776
1 => {
return JS_NewBool(ctx, if (((((size_of::<f64>() as usize)) == ((size_of::<f32>() as usize))) as i32)) != 0 { (((d) as f32)).is_nan() as i32 } else { if (((((size_of::<f64>() as usize)) == ((size_of::<f64>() as usize))) as i32)) != 0 { (d).is_nan() as i32 } else { (d).is_nan() as i32 } });
}
// C line 39775
2 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 39774
3 => {
vm_block = if ((((!(((!((JS_ToFloat64(ctx, core::ptr::addr_of_mut!(d), *(argv).offset(((0 as i32)) as isize))) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 2 } else { 1 }; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:39779. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_global_isFinite(mut ctx: *mut JSContext, mut this_val: JSValue, mut argc: i32, mut argv: *mut JSValue) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut d: f64 = core::mem::zeroed();
let mut vm_block: usize = 3;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 39785
1 => {
return JS_NewBool(ctx, if (((((size_of::<f64>() as usize)) == ((size_of::<f32>() as usize))) as i32)) != 0 { (((d) as f32)).is_finite() as i32 } else { if (((((size_of::<f64>() as usize)) == ((size_of::<f64>() as usize))) as i32)) != 0 { (d).is_finite() as i32 } else { (d).is_finite() as i32 } });
}
// C line 39784
2 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 39783
3 => {
vm_block = if ((((!(((!((JS_ToFloat64(ctx, core::ptr::addr_of_mut!(d), *(argv).offset(((0 as i32)) as isize))) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 2 } else { 1 }; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:44588. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_number_constructor(mut ctx: *mut JSContext, mut new_target: JSValue, mut argc: i32, mut argv: *mut JSValue) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut val: JSValue = core::mem::zeroed();
let mut obj: JSValue = core::mem::zeroed();
let mut p: *mut JSBigInt = core::mem::zeroed();
let mut d: f64 = core::mem::zeroed();
let mut vm_block: usize = 22;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 44621
1 => {
return obj;
}
// C line 44620
2 => {
let _ = JS_SetObjectData(ctx, obj, val);
vm_block = 1; continue;
}
// C line 44619
3 => {
vm_block = if ((!((JS_IsException(obj)) != 0) as i32)) != 0 { 2 } else { 1 }; continue;
}
// C line 44618
4 => {
let _ = { let assigned = js_create_from_ctor(ctx, new_target, (JS_CLASS_NUMBER as i32)); obj = assigned; assigned };
vm_block = 3; continue;
}
// C line 44623
5 => {
return val;
}
// C line 44617
6 => {
vm_block = if ((!((JS_IsUndefined(new_target)) != 0) as i32)) != 0 { 4 } else { 5 }; continue;
}
// C line 44593
7 => {
let _ = { let assigned = JS_NewInt32(ctx, (0 as i32)); val = assigned; assigned };
vm_block = 6; continue;
}
// C line ?
8 => {
vm_block = 6; continue;
}
// C line 44612
9 => {
vm_block = 6; continue;
}
// C line 44610
10 => {
let _ = { let assigned = JS_NewFloat64(ctx, d); val = assigned; assigned };
vm_block = 9; continue;
}
// C line 44609
11 => {
let _ = JS_FreeValue(ctx, val);
vm_block = 10; continue;
}
// C line 44608
12 => {
let _ = { let assigned = js_bigint_to_float64(ctx, p); d = assigned; assigned };
vm_block = 11; continue;
}
// C line 44606
13 => {
p = ((((val).u).ptr) as *mut JSBigInt);
vm_block = 12; continue;
}
// C line 44603
14 => {
vm_block = 6; continue;
}
// C line 44602
15 => {
return val;
}
// C line 44601
16 => {
vm_block = if (JS_IsException(val)) != 0 { 15 } else { 14 }; continue;
}
// C line 44600
17 => {
let _ = { let assigned = JS_NewInt64(ctx, ((val).u).short_big_int); val = assigned; assigned };
vm_block = 16; continue;
}
// C line 44598
18 => {
vm_block = match (((val).tag) as i32) { x if x == (JS_TAG_BIG_INT as i32) => 13, x if x == (JS_TAG_SHORT_BIG_INT as i32) => 17, _ => 8, }; continue;
}
// C line 44597
19 => {
return val;
}
// C line 44596
20 => {
vm_block = if (JS_IsException(val)) != 0 { 19 } else { 18 }; continue;
}
// C line 44595
21 => {
let _ = { let assigned = JS_ToNumeric(ctx, *(argv).offset(((0 as i32)) as isize)); val = assigned; assigned };
vm_block = 20; continue;
}
// C line 44592
22 => {
vm_block = if ((((argc) == ((0 as i32))) as i32)) != 0 { 7 } else { 21 }; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:44644. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_number_isNaN(mut ctx: *mut JSContext, mut this_val: JSValue, mut argc: i32, mut argv: *mut JSValue) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut vm_block: usize = 3;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 44649
1 => {
return js_global_isNaN(ctx, this_val, argc, argv);
}
// C line 44648
2 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_BOOL as i32)) as i64) };
}
// C line 44647
3 => {
vm_block = if ((!((JS_IsNumber(*(argv).offset(((0 as i32)) as isize))) != 0) as i32)) != 0 { 2 } else { 1 }; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:44652. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_number_isFinite(mut ctx: *mut JSContext, mut this_val: JSValue, mut argc: i32, mut argv: *mut JSValue) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut vm_block: usize = 3;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 44657
1 => {
return js_global_isFinite(ctx, this_val, argc, argv);
}
// C line 44656
2 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_BOOL as i32)) as i64) };
}
// C line 44655
3 => {
vm_block = if ((!((JS_IsNumber(*(argv).offset(((0 as i32)) as isize))) != 0) as i32)) != 0 { 2 } else { 1 }; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:44660. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_number_isInteger(mut ctx: *mut JSContext, mut this_val: JSValue, mut argc: i32, mut argv: *mut JSValue) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut ret: i32 = core::mem::zeroed();
let mut vm_block: usize = 4;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 44666
1 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 44668
2 => {
return JS_NewBool(ctx, ret);
}
// C line 44665
3 => {
vm_block = if ((((ret) < ((0 as i32))) as i32)) != 0 { 1 } else { 2 }; continue;
}
// C line 44664
4 => {
let _ = { let assigned = JS_NumberIsInteger(ctx, *(argv).offset(((0 as i32)) as isize)); ret = assigned; assigned };
vm_block = 3; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:44671. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_number_isSafeInteger(mut ctx: *mut JSContext, mut this_val: JSValue, mut argc: i32, mut argv: *mut JSValue) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut d: f64 = core::mem::zeroed();
let mut vm_block: usize = 5;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 44679
1 => {
return JS_NewBool(ctx, is_safe_integer(d));
}
// C line 44678
2 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 44677
3 => {
vm_block = if ((((!(((!((JS_ToFloat64(ctx, core::ptr::addr_of_mut!(d), *(argv).offset(((0 as i32)) as isize))) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 2 } else { 1 }; continue;
}
// C line 44676
4 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_BOOL as i32)) as i64) };
}
// C line 44675
5 => {
vm_block = if ((!((JS_IsNumber(*(argv).offset(((0 as i32)) as isize))) != 0) as i32)) != 0 { 4 } else { 3 }; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:44702. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_thisNumberValue(mut ctx: *mut JSContext, mut this_val: JSValue) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut p: *mut JSObject = core::mem::zeroed();
let mut vm_block: usize = 8;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 44714
1 => {
return JS_ThrowTypeError(ctx, c"not a number".as_ptr());
}
// C line 44711
2 => {
return JS_DupValue(ctx, ((*(p)).u).object_data);
}
// C line 44710
3 => {
vm_block = if (JS_IsNumber(((*(p)).u).object_data)) != 0 { 2 } else { 1 }; continue;
}
// C line 44709
4 => {
vm_block = if (((((((*(p)).class_id) as i32)) == ((JS_CLASS_NUMBER as i32))) as i32)) != 0 { 3 } else { 1 }; continue;
}
// C line 44708
5 => {
p = ((((this_val).u).ptr) as *mut JSObject);
vm_block = 4; continue;
}
// C line 44707
6 => {
vm_block = if (((((((this_val).tag) as i32)) == ((JS_TAG_OBJECT as i32))) as i32)) != 0 { 5 } else { 1 }; continue;
}
// C line 44705
7 => {
return JS_DupValue(ctx, this_val);
}
// C line 44704
8 => {
vm_block = if (JS_IsNumber(this_val)) != 0 { 7 } else { 6 }; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:44717. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_number_valueOf(mut ctx: *mut JSContext, mut this_val: JSValue, mut argc: i32, mut argv: *mut JSValue) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut vm_block: usize = 1;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 44720
1 => {
return js_thisNumberValue(ctx, this_val);
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:44723. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_get_radix(mut ctx: *mut JSContext, mut val: JSValue) -> i32 {
let mut vm_local_storage = Vec::<u64>::new();
let mut radix: i32 = core::mem::zeroed();
let mut vm_block: usize = 6;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 44732
1 => {
return radix;
}
// C line 44730
2 => {
return ((1 as i32)).wrapping_neg();
}
// C line 44729
3 => {
let _ = JS_ThrowRangeError(ctx, c"radix must be between 2 and 36".as_ptr());
vm_block = 2; continue;
}
// C line 44728
4 => {
vm_block = if ((((((((radix) < ((2 as i32))) as i32)) != 0) || (((((radix) > ((36 as i32))) as i32)) != 0)) as i32)) != 0 { 3 } else { 1 }; continue;
}
// C line 44727
5 => {
return ((1 as i32)).wrapping_neg();
}
// C line 44726
6 => {
vm_block = if (JS_ToInt32Sat(ctx, core::ptr::addr_of_mut!(radix), val)) != 0 { 5 } else { 4 }; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:44735. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_number_toString(mut ctx: *mut JSContext, mut this_val: JSValue, mut argc: i32, mut argv: *mut JSValue, mut magic: i32) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut val: JSValue = core::mem::zeroed();
let mut base: i32 = core::mem::zeroed();
let mut flags: i32 = core::mem::zeroed();
let mut d: f64 = core::mem::zeroed();
let mut buf1: [c_char; 70] = core::mem::zeroed();
let mut len: i32 = core::mem::zeroed();
let mut vm_block: usize = 19;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 44766
1 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line ? labels: fail
2 => {
let _ = JS_FreeValue(ctx, val);
vm_block = 1; continue;
}
// C line 44763
3 => {
return js_dtoa2(ctx, d, base, (0 as i32), flags);
}
// C line 44762
4 => {
let _ = { flags = ((flags) | (((2 as i32)).wrapping_shl(((2 as i32)) as u32))); flags };
vm_block = 3; continue;
}
// C line 44761
5 => {
vm_block = if ((((base) != ((10 as i32))) as i32)) != 0 { 4 } else { 3 }; continue;
}
// C line 44760
6 => {
let _ = { let assigned = ((0 as i32)).wrapping_shl(((0 as i32)) as u32); flags = assigned; assigned };
vm_block = 5; continue;
}
// C line 44759
7 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 44758
8 => {
vm_block = if (JS_ToFloat64Free(ctx, core::ptr::addr_of_mut!(d), val)) != 0 { 7 } else { 6 }; continue;
}
// C line 44756
9 => {
return js_new_string8_len(ctx, (buf1).as_mut_ptr(), len);
}
// C line 44755
10 => {
let _ = { let assigned = ((crate::dtoa::i64toa_radix((buf1).as_mut_ptr(), ((((((val).u).uint64) as i32)) as i64), ((base) as u32))) as i32); len = assigned; assigned };
vm_block = 9; continue;
}
// C line 44752
11 => {
vm_block = if (((((((val).tag) as i32)) == ((JS_TAG_INT as i32))) as i32)) != 0 { 10 } else { 8 }; continue;
}
// C line 44746
12 => {
let _ = { let assigned = (10 as i32); base = assigned; assigned };
vm_block = 11; continue;
}
// C line 44750
13 => {
vm_block = 2; continue;
}
// C line 44749
14 => {
vm_block = if ((((base) < ((0 as i32))) as i32)) != 0 { 13 } else { 11 }; continue;
}
// C line 44748
15 => {
let _ = { let assigned = js_get_radix(ctx, *(argv).offset(((0 as i32)) as isize)); base = assigned; assigned };
vm_block = 14; continue;
}
// C line 44745
16 => {
vm_block = if (((((magic) != 0) || ((JS_IsUndefined(*(argv).offset(((0 as i32)) as isize))) != 0)) as i32)) != 0 { 12 } else { 15 }; continue;
}
// C line 44744
17 => {
return val;
}
// C line 44743
18 => {
vm_block = if (JS_IsException(val)) != 0 { 17 } else { 16 }; continue;
}
// C line 44742
19 => {
let _ = { let assigned = js_thisNumberValue(ctx, this_val); val = assigned; assigned };
vm_block = 18; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:44769. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_number_toFixed(mut ctx: *mut JSContext, mut this_val: JSValue, mut argc: i32, mut argv: *mut JSValue) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut val: JSValue = core::mem::zeroed();
let mut f: i32 = core::mem::zeroed();
let mut flags: i32 = core::mem::zeroed();
let mut d: f64 = core::mem::zeroed();
let mut vm_block: usize = 13;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 44789
1 => {
return js_dtoa2(ctx, d, (10 as i32), f, flags);
}
// C line 44786
2 => {
let _ = { let assigned = ((0 as i32)).wrapping_shl(((0 as i32)) as u32); flags = assigned; assigned };
vm_block = 1; continue;
}
// C line 44788
3 => {
let _ = { let assigned = ((2 as i32)).wrapping_shl(((0 as i32)) as u32); flags = assigned; assigned };
vm_block = 1; continue;
}
// C line 44785
4 => {
vm_block = if (((((d).abs()) >= ((1.0E+21 as f64))) as i32)) != 0 { 2 } else { 3 }; continue;
}
// C line 44784
5 => {
return JS_ThrowRangeError(ctx, c"invalid number of digits".as_ptr());
}
// C line 44783
6 => {
vm_block = if ((((((((f) < ((0 as i32))) as i32)) != 0) || (((((f) > ((100 as i32))) as i32)) != 0)) as i32)) != 0 { 5 } else { 4 }; continue;
}
// C line 44782
7 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 44781
8 => {
vm_block = if (JS_ToInt32Sat(ctx, core::ptr::addr_of_mut!(f), *(argv).offset(((0 as i32)) as isize))) != 0 { 7 } else { 6 }; continue;
}
// C line 44780
9 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 44779
10 => {
vm_block = if (JS_ToFloat64Free(ctx, core::ptr::addr_of_mut!(d), val)) != 0 { 9 } else { 8 }; continue;
}
// C line 44778
11 => {
return val;
}
// C line 44777
12 => {
vm_block = if (JS_IsException(val)) != 0 { 11 } else { 10 }; continue;
}
// C line 44776
13 => {
let _ = { let assigned = js_thisNumberValue(ctx, this_val); val = assigned; assigned };
vm_block = 12; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:44792. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_number_toExponential(mut ctx: *mut JSContext, mut this_val: JSValue, mut argc: i32, mut argv: *mut JSValue) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut val: JSValue = core::mem::zeroed();
let mut f: i32 = core::mem::zeroed();
let mut flags: i32 = core::mem::zeroed();
let mut d: f64 = core::mem::zeroed();
let mut vm_block: usize = 17;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 44818
1 => {
return js_dtoa2(ctx, d, (10 as i32), f, ((flags) | (((1 as i32)).wrapping_shl(((2 as i32)) as u32))));
}
// C line 44811
2 => {
let _ = { let assigned = (0 as i32); f = assigned; assigned };
vm_block = 1; continue;
}
// C line 44810
3 => {
let _ = { let assigned = ((0 as i32)).wrapping_shl(((0 as i32)) as u32); flags = assigned; assigned };
vm_block = 2; continue;
}
// C line 44816
4 => {
let _ = { let assigned = ((1 as i32)).wrapping_shl(((0 as i32)) as u32); flags = assigned; assigned };
vm_block = 1; continue;
}
// C line 44815
5 => {
let _ = { let old = f; f = (f).wrapping_add(1); old };
vm_block = 4; continue;
}
// C line 44814
6 => {
return JS_ThrowRangeError(ctx, c"invalid number of digits".as_ptr());
}
// C line 44813
7 => {
vm_block = if ((((((((f) < ((0 as i32))) as i32)) != 0) || (((((f) > ((100 as i32))) as i32)) != 0)) as i32)) != 0 { 6 } else { 5 }; continue;
}
// C line 44809
8 => {
vm_block = if (JS_IsUndefined(*(argv).offset(((0 as i32)) as isize))) != 0 { 3 } else { 7 }; continue;
}
// C line 44807
9 => {
return JS_ToStringFree(ctx, __JS_NewFloat64(ctx, d));
}
// C line 44806
10 => {
vm_block = if ((!((if (((((size_of::<f64>() as usize)) == ((size_of::<f32>() as usize))) as i32)) != 0 { (((d) as f32)).is_finite() as i32 } else { if (((((size_of::<f64>() as usize)) == ((size_of::<f64>() as usize))) as i32)) != 0 { (d).is_finite() as i32 } else { (d).is_finite() as i32 } }) != 0) as i32)) != 0 { 9 } else { 8 }; continue;
}
// C line 44805
11 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 44804
12 => {
vm_block = if (JS_ToInt32Sat(ctx, core::ptr::addr_of_mut!(f), *(argv).offset(((0 as i32)) as isize))) != 0 { 11 } else { 10 }; continue;
}
// C line 44803
13 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 44802
14 => {
vm_block = if (JS_ToFloat64Free(ctx, core::ptr::addr_of_mut!(d), val)) != 0 { 13 } else { 12 }; continue;
}
// C line 44801
15 => {
return val;
}
// C line 44800
16 => {
vm_block = if (JS_IsException(val)) != 0 { 15 } else { 14 }; continue;
}
// C line 44799
17 => {
let _ = { let assigned = js_thisNumberValue(ctx, this_val); val = assigned; assigned };
vm_block = 16; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:44821. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_number_toPrecision(mut ctx: *mut JSContext, mut this_val: JSValue, mut argc: i32, mut argv: *mut JSValue) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut val: JSValue = core::mem::zeroed();
let mut p: i32 = core::mem::zeroed();
let mut d: f64 = core::mem::zeroed();
let mut vm_block: usize = 14;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 44843
1 => {
return js_dtoa2(ctx, d, (10 as i32), p, ((1 as i32)).wrapping_shl(((0 as i32)) as u32));
}
// C line 44842
2 => {
return JS_ThrowRangeError(ctx, c"invalid number of digits".as_ptr());
}
// C line 44841
3 => {
vm_block = if ((((((((p) < ((1 as i32))) as i32)) != 0) || (((((p) > ((100 as i32))) as i32)) != 0)) as i32)) != 0 { 2 } else { 1 }; continue;
}
// C line ? labels: to_string
4 => {
return JS_ToStringFree(ctx, __JS_NewFloat64(ctx, d));
}
// C line 44837
5 => {
vm_block = if ((!((if (((((size_of::<f64>() as usize)) == ((size_of::<f32>() as usize))) as i32)) != 0 { (((d) as f32)).is_finite() as i32 } else { if (((((size_of::<f64>() as usize)) == ((size_of::<f64>() as usize))) as i32)) != 0 { (d).is_finite() as i32 } else { (d).is_finite() as i32 } }) != 0) as i32)) != 0 { 4 } else { 3 }; continue;
}
// C line 44836
6 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 44835
7 => {
vm_block = if (JS_ToInt32Sat(ctx, core::ptr::addr_of_mut!(p), *(argv).offset(((0 as i32)) as isize))) != 0 { 6 } else { 5 }; continue;
}
// C line 44834
8 => {
vm_block = 4; continue;
}
// C line 44833
9 => {
vm_block = if (JS_IsUndefined(*(argv).offset(((0 as i32)) as isize))) != 0 { 8 } else { 7 }; continue;
}
// C line 44832
10 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 44831
11 => {
vm_block = if (JS_ToFloat64Free(ctx, core::ptr::addr_of_mut!(d), val)) != 0 { 10 } else { 9 }; continue;
}
// C line 44830
12 => {
return val;
}
// C line 44829
13 => {
vm_block = if (JS_IsException(val)) != 0 { 12 } else { 11 }; continue;
}
// C line 44828
14 => {
let _ = { let assigned = js_thisNumberValue(ctx, this_val); val = assigned; assigned };
vm_block = 13; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:44855. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_parseInt(mut ctx: *mut JSContext, mut this_val: JSValue, mut argc: i32, mut argv: *mut JSValue) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut str: *const c_char = core::mem::zeroed();
let mut p: *const c_char = core::mem::zeroed();
let mut radix: i32 = core::mem::zeroed();
let mut flags: i32 = core::mem::zeroed();
let mut ret: JSValue = core::mem::zeroed();
let mut vm_block: usize = 14;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 44878
1 => {
return ret;
}
// C line 44877
2 => {
let _ = JS_FreeCString(ctx, str);
vm_block = 1; continue;
}
// C line 44870
3 => {
let _ = { let assigned = JSValue { u: JSValueUnion { float64: ((f32::NAN) as f64) }, tag: (((JS_TAG_FLOAT64 as i32)) as i64) }; ret = assigned; assigned };
vm_block = 2; continue;
}
// C line 44875
4 => {
let _ = { let assigned = js_atof(ctx, p, (core::ptr::null_mut::<*mut c_char>()) as *mut *const c_char, radix, flags); ret = assigned; assigned };
vm_block = 2; continue;
}
// C line 44874
5 => {
let _ = { let assigned = ((((1 as i32)).wrapping_shl(((0 as i32)) as u32)) | (((1 as i32)).wrapping_shl(((10 as i32)) as u32))); flags = assigned; assigned };
vm_block = 4; continue;
}
// C line 44873
6 => {
let _ = { p = (p).offset(((skip_spaces(p)) as isize)); p };
vm_block = 5; continue;
}
// C line 44872
7 => {
let _ = { let assigned = str; p = assigned; assigned };
vm_block = 6; continue;
}
// C line 44869
8 => {
vm_block = if ((((((((radix) != ((0 as i32))) as i32)) != 0) && (((((((((radix) < ((2 as i32))) as i32)) != 0) || (((((radix) > ((36 as i32))) as i32)) != 0)) as i32)) != 0)) as i32)) != 0 { 3 } else { 7 }; continue;
}
// C line 44867
9 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 44866
10 => {
let _ = JS_FreeCString(ctx, str);
vm_block = 9; continue;
}
// C line 44865
11 => {
vm_block = if (JS_ToInt32(ctx, core::ptr::addr_of_mut!(radix), *(argv).offset(((1 as i32)) as isize))) != 0 { 10 } else { 8 }; continue;
}
// C line 44864
12 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 44863
13 => {
vm_block = if ((!(!(str).is_null()) as i32)) != 0 { 12 } else { 11 }; continue;
}
// C line 44862
14 => {
let _ = { let assigned = JS_ToCString(ctx, *(argv).offset(((0 as i32)) as isize)); str = assigned; assigned };
vm_block = 13; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:44881. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_parseFloat(mut ctx: *mut JSContext, mut this_val: JSValue, mut argc: i32, mut argv: *mut JSValue) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut str: *const c_char = core::mem::zeroed();
let mut p: *const c_char = core::mem::zeroed();
let mut ret: JSValue = core::mem::zeroed();
let mut vm_block: usize = 8;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 44894
1 => {
return ret;
}
// C line 44893
2 => {
let _ = JS_FreeCString(ctx, str);
vm_block = 1; continue;
}
// C line 44892
3 => {
let _ = { let assigned = js_atof(ctx, p, (core::ptr::null_mut::<*mut c_char>()) as *mut *const c_char, (10 as i32), (0 as i32)); ret = assigned; assigned };
vm_block = 2; continue;
}
// C line 44891
4 => {
let _ = { p = (p).offset(((skip_spaces(p)) as isize)); p };
vm_block = 3; continue;
}
// C line 44890
5 => {
let _ = { let assigned = str; p = assigned; assigned };
vm_block = 4; continue;
}
// C line 44889
6 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 44888
7 => {
vm_block = if ((!(!(str).is_null()) as i32)) != 0 { 6 } else { 5 }; continue;
}
// C line 44887
8 => {
let _ = { let assigned = JS_ToCString(ctx, *(argv).offset(((0 as i32)) as isize)); str = assigned; assigned };
vm_block = 7; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:44898. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_boolean_constructor(mut ctx: *mut JSContext, mut new_target: JSValue, mut argc: i32, mut argv: *mut JSValue) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut val: JSValue = core::mem::zeroed();
let mut obj: JSValue = core::mem::zeroed();
let mut vm_block: usize = 7;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 44907
1 => {
return obj;
}
// C line 44906
2 => {
let _ = JS_SetObjectData(ctx, obj, val);
vm_block = 1; continue;
}
// C line 44905
3 => {
vm_block = if ((!((JS_IsException(obj)) != 0) as i32)) != 0 { 2 } else { 1 }; continue;
}
// C line 44904
4 => {
let _ = { let assigned = js_create_from_ctor(ctx, new_target, (JS_CLASS_BOOLEAN as i32)); obj = assigned; assigned };
vm_block = 3; continue;
}
// C line 44909
5 => {
return val;
}
// C line 44903
6 => {
vm_block = if ((!((JS_IsUndefined(new_target)) != 0) as i32)) != 0 { 4 } else { 5 }; continue;
}
// C line 44902
7 => {
let _ = { let assigned = JS_NewBool(ctx, JS_ToBool(ctx, *(argv).offset(((0 as i32)) as isize))); val = assigned; assigned };
vm_block = 6; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:44913. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_thisBooleanValue(mut ctx: *mut JSContext, mut this_val: JSValue) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut p: *mut JSObject = core::mem::zeroed();
let mut vm_block: usize = 8;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 44925
1 => {
return JS_ThrowTypeError(ctx, c"not a boolean".as_ptr());
}
// C line 44922
2 => {
return ((*(p)).u).object_data;
}
// C line 44921
3 => {
vm_block = if (((((((((*(p)).u).object_data).tag) as i32)) == ((JS_TAG_BOOL as i32))) as i32)) != 0 { 2 } else { 1 }; continue;
}
// C line 44920
4 => {
vm_block = if (((((((*(p)).class_id) as i32)) == ((JS_CLASS_BOOLEAN as i32))) as i32)) != 0 { 3 } else { 1 }; continue;
}
// C line 44919
5 => {
p = ((((this_val).u).ptr) as *mut JSObject);
vm_block = 4; continue;
}
// C line 44918
6 => {
vm_block = if (((((((this_val).tag) as i32)) == ((JS_TAG_OBJECT as i32))) as i32)) != 0 { 5 } else { 1 }; continue;
}
// C line 44916
7 => {
return JS_DupValue(ctx, this_val);
}
// C line 44915
8 => {
vm_block = if (((((((this_val).tag) as i32)) == ((JS_TAG_BOOL as i32))) as i32)) != 0 { 7 } else { 6 }; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:44928. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_boolean_toString(mut ctx: *mut JSContext, mut this_val: JSValue, mut argc: i32, mut argv: *mut JSValue) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut val: JSValue = core::mem::zeroed();
let mut vm_block: usize = 4;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 44934
1 => {
return JS_AtomToString(ctx, ((if (((((val).u).uint64) as i32)) != 0 { (crate::quickjs_atom::JS_ATOM_true as i32) } else { (crate::quickjs_atom::JS_ATOM_false as i32) }) as JSAtom));
}
// C line 44933
2 => {
return val;
}
// C line 44932
3 => {
vm_block = if (JS_IsException(val)) != 0 { 2 } else { 1 }; continue;
}
// C line 44931
4 => {
val = js_thisBooleanValue(ctx, this_val);
vm_block = 3; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:44938. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_boolean_valueOf(mut ctx: *mut JSContext, mut this_val: JSValue, mut argc: i32, mut argv: *mut JSValue) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut vm_block: usize = 1;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 44941
1 => {
return js_thisBooleanValue(ctx, this_val);
}
_ => std::process::abort(),
} }
}
