// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:59898. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_dataview_constructor(mut ctx: *mut JSContext, mut new_target: JSValue, mut argc: i32, mut argv: *mut JSValue) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut recompute_len: i32 = core::mem::zeroed();
let mut track_rab: i32 = core::mem::zeroed();
let mut abuf: *mut JSArrayBuffer = core::mem::zeroed();
let mut offset: u64 = core::mem::zeroed();
let mut len: u32 = core::mem::zeroed();
let mut buffer: JSValue = core::mem::zeroed();
let mut obj: JSValue = core::mem::zeroed();
let mut ta: *mut JSTypedArray = core::mem::zeroed();
let mut p: *mut JSObject = core::mem::zeroed();
let mut l: u64 = core::mem::zeroed();
let mut vm_block: usize = 49;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 59970
1 => {
return obj;
}
// C line 59969
2 => {
let _ = { let assigned = ta; ((*(p)).u).typed_array = assigned; assigned };
vm_block = 1; continue;
}
// C line 59968
3 => {
let _ = list_add_tail(core::ptr::addr_of_mut!((*(ta)).link), core::ptr::addr_of_mut!((*(abuf)).array_list));
vm_block = 2; continue;
}
// C line 59967
4 => {
let _ = { let assigned = track_rab; (*(ta)).track_rab = assigned; assigned };
vm_block = 3; continue;
}
// C line 59966
5 => {
let _ = { let assigned = len; (*(ta)).length = assigned; assigned };
vm_block = 4; continue;
}
// C line 59965
6 => {
let _ = { let assigned = ((offset) as u32); (*(ta)).offset = assigned; assigned };
vm_block = 5; continue;
}
// C line 59964
7 => {
let _ = { let assigned = ((((JS_DupValue(ctx, buffer)).u).ptr) as *mut JSObject); (*(ta)).buffer = assigned; assigned };
vm_block = 6; continue;
}
// C line 59963
8 => {
let _ = { let assigned = p; (*(ta)).obj = assigned; assigned };
vm_block = 7; continue;
}
// C line 59962
9 => {
let _ = { let assigned = ((((obj).u).ptr) as *mut JSObject); p = assigned; assigned };
vm_block = 8; continue;
}
// C line 59960
10 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line ? labels: fail
11 => {
let _ = JS_FreeValue(ctx, obj);
vm_block = 10; continue;
}
// C line 59957
12 => {
vm_block = if ((!(!(ta).is_null()) as i32)) != 0 { 11 } else { 9 }; continue;
}
// C line 59956
13 => {
let _ = { let assigned = ((js_malloc(ctx, (size_of::<JSTypedArray>() as usize))) as *mut JSTypedArray); ta = assigned; assigned };
vm_block = 12; continue;
}
// C line 59948
14 => {
vm_block = 17; continue;
}
// C line 59950
15 => {
let _ = { let assigned = ((((((*(abuf)).byte_length) as u64)).wrapping_sub(offset)) as u32); len = assigned; assigned };
vm_block = 13; continue;
}
// C line 59954
16 => {
vm_block = 11; continue;
}
// C line ? labels: out_of_bound
17 => {
let _ = JS_ThrowRangeError(ctx, c"invalid byteOffset or byteLength".as_ptr());
vm_block = 16; continue;
}
// C line 59951
18 => {
vm_block = if (((((offset).wrapping_add(((len) as u64))) > ((((*(abuf)).byte_length) as u64))) as i32)) != 0 { 17 } else { 13 }; continue;
}
// C line 59949
19 => {
vm_block = if (recompute_len) != 0 { 15 } else { 18 }; continue;
}
// C line 59947
20 => {
vm_block = if ((((offset) > ((((*(abuf)).byte_length) as u64))) as i32)) != 0 { 14 } else { 19 }; continue;
}
// C line 59944
21 => {
vm_block = 11; continue;
}
// C line 59943
22 => {
let _ = JS_ThrowTypeErrorDetachedArrayBuffer(ctx);
vm_block = 21; continue;
}
// C line 59941
23 => {
vm_block = if ((*(abuf)).detached) != 0 { 22 } else { 20 }; continue;
}
// C line 59940
24 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 59939
25 => {
vm_block = if (JS_IsException(obj)) != 0 { 24 } else { 23 }; continue;
}
// C line 59938
26 => {
let _ = { let assigned = js_create_from_ctor(ctx, new_target, (JS_CLASS_DATAVIEW as i32)); obj = assigned; assigned };
vm_block = 25; continue;
}
// C line 59932
27 => {
let _ = { let assigned = ((l) as u32); len = assigned; assigned };
vm_block = 26; continue;
}
// C line 59931
28 => {
return JS_ThrowRangeError(ctx, c"invalid byteLength".as_ptr());
}
// C line 59930
29 => {
vm_block = if ((((l) > (((len) as u64))) as i32)) != 0 { 28 } else { 27 }; continue;
}
// C line 59929
30 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 59928
31 => {
vm_block = if (JS_ToIndex(ctx, core::ptr::addr_of_mut!(l), *(argv).offset(((2 as i32)) as isize))) != 0 { 30 } else { 29 }; continue;
}
// C line 59935
32 => {
let _ = { let assigned = array_buffer_is_resizable(abuf); track_rab = assigned; assigned };
vm_block = 26; continue;
}
// C line 59934
33 => {
let _ = { let assigned = (1 as i32); recompute_len = assigned; assigned };
vm_block = 32; continue;
}
// C line 59926
34 => {
vm_block = if ((((((((argc) > ((2 as i32))) as i32)) != 0) && (((!((JS_IsUndefined(*(argv).offset(((2 as i32)) as isize))) != 0) as i32)) != 0)) as i32)) != 0 { 31 } else { 33 }; continue;
}
// C line 59925
35 => {
let _ = { let assigned = ((((((*(abuf)).byte_length) as u64)).wrapping_sub(offset)) as u32); len = assigned; assigned };
vm_block = 34; continue;
}
// C line 59924
36 => {
return JS_ThrowRangeError(ctx, c"invalid byteOffset".as_ptr());
}
// C line 59923
37 => {
vm_block = if ((((offset) > ((((*(abuf)).byte_length) as u64))) as i32)) != 0 { 36 } else { 35 }; continue;
}
// C line 59922
38 => {
return JS_ThrowTypeErrorDetachedArrayBuffer(ctx);
}
// C line 59921
39 => {
vm_block = if ((*(abuf)).detached) != 0 { 38 } else { 37 }; continue;
}
// C line 59919
40 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 59918
41 => {
vm_block = if (JS_ToIndex(ctx, core::ptr::addr_of_mut!(offset), *(argv).offset(((1 as i32)) as isize))) != 0 { 40 } else { 39 }; continue;
}
// C line 59917
42 => {
vm_block = if ((((argc) > ((1 as i32))) as i32)) != 0 { 41 } else { 39 }; continue;
}
// C line 59916
43 => {
let _ = { let assigned = (((0 as i32)) as u64); offset = assigned; assigned };
vm_block = 42; continue;
}
// C line 59915
44 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 59914
45 => {
vm_block = if ((!(!(abuf).is_null()) as i32)) != 0 { 44 } else { 43 }; continue;
}
// C line 59913
46 => {
let _ = { let assigned = js_get_array_buffer(ctx, buffer); abuf = assigned; assigned };
vm_block = 45; continue;
}
// C line 59912
47 => {
let _ = { let assigned = *(argv).offset(((0 as i32)) as isize); buffer = assigned; assigned };
vm_block = 46; continue;
}
// C line 59903
48 => {
track_rab = (0 as i32);
vm_block = 47; continue;
}
// C line 59902
49 => {
recompute_len = (0 as i32);
vm_block = 48; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:59974. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn dataview_is_oob(mut p: *mut JSObject) -> i32 {
let mut vm_local_storage = Vec::<u64>::new();
let mut abuf: *mut JSArrayBuffer = core::mem::zeroed();
let mut ta: *mut JSTypedArray = core::mem::zeroed();
let mut vm_block: usize = 10;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 59988
1 => {
return (((((((*(ta)).offset) as i64)).wrapping_add((((*(ta)).length) as i64))) > ((((*(abuf)).byte_length) as i64))) as i32);
}
// C line 59987
2 => {
return (0 as i32);
}
// C line 59986
3 => {
vm_block = if ((*(ta)).track_rab) != 0 { 2 } else { 1 }; continue;
}
// C line 59985
4 => {
return (1 as i32);
}
// C line 59984
5 => {
vm_block = if (((((*(ta)).offset) > ((((*(abuf)).byte_length) as u32))) as i32)) != 0 { 4 } else { 3 }; continue;
}
// C line 59983
6 => {
return (1 as i32);
}
// C line 59982
7 => {
vm_block = if ((*(abuf)).detached) != 0 { 6 } else { 5 }; continue;
}
// C line 59981
8 => {
let _ = { let assigned = ((*((*(ta)).buffer)).u).array_buffer; abuf = assigned; assigned };
vm_block = 7; continue;
}
// C line 59980
9 => {
let _ = { let assigned = ((*(p)).u).typed_array; ta = assigned; assigned };
vm_block = 8; continue;
}
// C line 59979
10 => {
let _ = if ((((!((((((((*(p)).class_id) as i32)) == ((JS_CLASS_DATAVIEW as i32))) as i32)) != 0) as i32)) as i64)) != 0 { std::process::abort() } else { { let _ = (0 as i32); } };
vm_block = 9; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:59991. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn get_dataview(mut ctx: *mut JSContext, mut this_val: JSValue) -> *mut JSObject {
let mut vm_local_storage = Vec::<u64>::new();
let mut p: *mut JSObject = core::mem::zeroed();
let mut vm_block: usize = 7;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 60002
1 => {
return p;
}
// C line 60000
2 => {
return core::ptr::null_mut::<JSObject>();
}
// C line ? labels: fail
3 => {
let _ = JS_ThrowTypeError(ctx, c"not a DataView".as_ptr());
vm_block = 2; continue;
}
// C line 59997
4 => {
vm_block = if (((((((*(p)).class_id) as i32)) != ((JS_CLASS_DATAVIEW as i32))) as i32)) != 0 { 3 } else { 1 }; continue;
}
// C line 59996
5 => {
let _ = { let assigned = ((((this_val).u).ptr) as *mut JSObject); p = assigned; assigned };
vm_block = 4; continue;
}
// C line 59995
6 => {
vm_block = 3; continue;
}
// C line 59994
7 => {
vm_block = if (((((((this_val).tag) as i32)) != ((JS_TAG_OBJECT as i32))) as i32)) != 0 { 6 } else { 5 }; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:60005. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_dataview_get_buffer(mut ctx: *mut JSContext, mut this_val: JSValue) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut p: *mut JSObject = core::mem::zeroed();
let mut ta: *mut JSTypedArray = core::mem::zeroed();
let mut vm_block: usize = 5;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 60013
1 => {
return JS_DupValue(ctx, JSValue { u: JSValueUnion { ptr: (((*(ta)).buffer) as *mut c_void) }, tag: (((JS_TAG_OBJECT as i32)) as i64) });
}
// C line 60012
2 => {
let _ = { let assigned = ((*(p)).u).typed_array; ta = assigned; assigned };
vm_block = 1; continue;
}
// C line 60011
3 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 60010
4 => {
vm_block = if ((!(!(p).is_null()) as i32)) != 0 { 3 } else { 2 }; continue;
}
// C line 60009
5 => {
let _ = { let assigned = get_dataview(ctx, this_val); p = assigned; assigned };
vm_block = 4; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:60016. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_dataview_get_byteLength(mut ctx: *mut JSContext, mut this_val: JSValue) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut abuf: *mut JSArrayBuffer = core::mem::zeroed();
let mut ta: *mut JSTypedArray = core::mem::zeroed();
let mut p: *mut JSObject = core::mem::zeroed();
let mut vm_block: usize = 10;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 60032
1 => {
return JS_NewUint32(ctx, (*(ta)).length);
}
// C line 60030
2 => {
return JS_NewUint32(ctx, ((((*(abuf)).byte_length) as u32)).wrapping_sub((*(ta)).offset));
}
// C line 60029
3 => {
let _ = { let assigned = ((*((*(ta)).buffer)).u).array_buffer; abuf = assigned; assigned };
vm_block = 2; continue;
}
// C line 60028
4 => {
vm_block = if ((*(ta)).track_rab) != 0 { 3 } else { 1 }; continue;
}
// C line 60027
5 => {
let _ = { let assigned = ((*(p)).u).typed_array; ta = assigned; assigned };
vm_block = 4; continue;
}
// C line 60026
6 => {
return JS_ThrowTypeErrorArrayBufferOOB(ctx);
}
// C line 60025
7 => {
vm_block = if (dataview_is_oob(p)) != 0 { 6 } else { 5 }; continue;
}
// C line 60024
8 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 60023
9 => {
vm_block = if ((!(!(p).is_null()) as i32)) != 0 { 8 } else { 7 }; continue;
}
// C line 60022
10 => {
let _ = { let assigned = get_dataview(ctx, this_val); p = assigned; assigned };
vm_block = 9; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:60035. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_dataview_get_byteOffset(mut ctx: *mut JSContext, mut this_val: JSValue) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut ta: *mut JSTypedArray = core::mem::zeroed();
let mut p: *mut JSObject = core::mem::zeroed();
let mut vm_block: usize = 7;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 60046
1 => {
return JS_NewUint32(ctx, (*(ta)).offset);
}
// C line 60045
2 => {
let _ = { let assigned = ((*(p)).u).typed_array; ta = assigned; assigned };
vm_block = 1; continue;
}
// C line 60044
3 => {
return JS_ThrowTypeErrorArrayBufferOOB(ctx);
}
// C line 60043
4 => {
vm_block = if (dataview_is_oob(p)) != 0 { 3 } else { 2 }; continue;
}
// C line 60042
5 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 60041
6 => {
vm_block = if ((!(!(p).is_null()) as i32)) != 0 { 5 } else { 4 }; continue;
}
// C line 60040
7 => {
let _ = { let assigned = get_dataview(ctx, this_val); p = assigned; assigned };
vm_block = 6; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:60049. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_dataview_getValue(mut ctx: *mut JSContext, mut this_obj: JSValue, mut argc: i32, mut argv: *mut JSValue, mut class_id: i32) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut ta: *mut JSTypedArray = core::mem::zeroed();
let mut abuf: *mut JSArrayBuffer = core::mem::zeroed();
let mut littleEndian: i32 = core::mem::zeroed();
let mut is_swap: i32 = core::mem::zeroed();
let mut size: i32 = core::mem::zeroed();
let mut ptr: *mut u8 = core::mem::zeroed();
let mut v: u32 = core::mem::zeroed();
let mut pos: u64 = core::mem::zeroed();
let mut v_1: u64 = core::mem::zeroed();
let mut v_2: u64 = core::mem::zeroed();
let mut v_3: u16 = core::mem::zeroed();
let mut u: BufferRecord60135_13 = core::mem::zeroed();
let mut u_1: BufferRecord60147_13 = core::mem::zeroed();
let mut vm_block: usize = 59;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line ?
1 => {
let _ = std::process::abort();
vm_block = 0; continue;
}
// C line 60154
2 => {
return __JS_NewFloat64(ctx, (u_1).f);
}
// C line 60153
3 => {
let _ = { let assigned = crate::cutils_header::bswap64((u_1).i); (u_1).i = assigned; assigned };
vm_block = 2; continue;
}
// C line 60152
4 => {
vm_block = if (is_swap) != 0 { 3 } else { 2 }; continue;
}
// C line 60151
5 => {
let _ = { let assigned = crate::cutils_header::get_u64(ptr); (u_1).i = assigned; assigned };
vm_block = 4; continue;
}
// C line 60143
6 => {
return __JS_NewFloat64(ctx, (((u).f) as f64));
}
// C line 60142
7 => {
let _ = { let assigned = v; (u).i = assigned; assigned };
vm_block = 6; continue;
}
// C line 60141
8 => {
let _ = { let assigned = crate::cutils_header::bswap32(v); v = assigned; assigned };
vm_block = 7; continue;
}
// C line 60140
9 => {
vm_block = if (is_swap) != 0 { 8 } else { 7 }; continue;
}
// C line 60139
10 => {
let _ = { let assigned = crate::cutils_header::get_u32(ptr); v = assigned; assigned };
vm_block = 9; continue;
}
// C line 60131
11 => {
return __JS_NewFloat64(ctx, crate::cutils_header::fromfp16(v_3));
}
// C line 60130
12 => {
let _ = { let assigned = crate::cutils_header::bswap16(v_3); v_3 = assigned; assigned };
vm_block = 11; continue;
}
// C line 60129
13 => {
vm_block = if (is_swap) != 0 { 12 } else { 11 }; continue;
}
// C line 60128
14 => {
let _ = { let assigned = ((crate::cutils_header::get_u16(ptr)) as u16); v_3 = assigned; assigned };
vm_block = 13; continue;
}
// C line 60124
15 => {
vm_block = 0; continue;
}
// C line 60122
16 => {
return JS_NewBigUint64(ctx, v_2);
}
// C line 60121
17 => {
let _ = { let assigned = crate::cutils_header::bswap64(v_2); v_2 = assigned; assigned };
vm_block = 16; continue;
}
// C line 60120
18 => {
vm_block = if (is_swap) != 0 { 17 } else { 16 }; continue;
}
// C line 60119
19 => {
let _ = { let assigned = crate::cutils_header::get_u64(ptr); v_2 = assigned; assigned };
vm_block = 18; continue;
}
// C line 60115
20 => {
vm_block = 0; continue;
}
// C line 60113
21 => {
return JS_NewBigInt64(ctx, ((v_1) as i64));
}
// C line 60112
22 => {
let _ = { let assigned = crate::cutils_header::bswap64(v_1); v_1 = assigned; assigned };
vm_block = 21; continue;
}
// C line 60111
23 => {
vm_block = if (is_swap) != 0 { 22 } else { 21 }; continue;
}
// C line 60110
24 => {
let _ = { let assigned = crate::cutils_header::get_u64(ptr); v_1 = assigned; assigned };
vm_block = 23; continue;
}
// C line 60106
25 => {
return JS_NewUint32(ctx, v);
}
// C line 60105
26 => {
let _ = { let assigned = crate::cutils_header::bswap32(v); v = assigned; assigned };
vm_block = 25; continue;
}
// C line 60104
27 => {
vm_block = if (is_swap) != 0 { 26 } else { 25 }; continue;
}
// C line 60103
28 => {
let _ = { let assigned = crate::cutils_header::get_u32(ptr); v = assigned; assigned };
vm_block = 27; continue;
}
// C line 60101
29 => {
return JS_NewInt32(ctx, ((v) as i32));
}
// C line 60100
30 => {
let _ = { let assigned = crate::cutils_header::bswap32(v); v = assigned; assigned };
vm_block = 29; continue;
}
// C line 60099
31 => {
vm_block = if (is_swap) != 0 { 30 } else { 29 }; continue;
}
// C line 60098
32 => {
let _ = { let assigned = crate::cutils_header::get_u32(ptr); v = assigned; assigned };
vm_block = 31; continue;
}
// C line 60096
33 => {
return JS_NewInt32(ctx, ((v) as i32));
}
// C line 60095
34 => {
let _ = { let assigned = ((crate::cutils_header::bswap16(((v) as u16))) as u32); v = assigned; assigned };
vm_block = 33; continue;
}
// C line 60094
35 => {
vm_block = if (is_swap) != 0 { 34 } else { 33 }; continue;
}
// C line 60093
36 => {
let _ = { let assigned = crate::cutils_header::get_u16(ptr); v = assigned; assigned };
vm_block = 35; continue;
}
// C line 60091
37 => {
return JS_NewInt32(ctx, ((((v) as i16)) as i32));
}
// C line 60090
38 => {
let _ = { let assigned = ((crate::cutils_header::bswap16(((v) as u16))) as u32); v = assigned; assigned };
vm_block = 37; continue;
}
// C line 60089
39 => {
vm_block = if (is_swap) != 0 { 38 } else { 37 }; continue;
}
// C line 60088
40 => {
let _ = { let assigned = crate::cutils_header::get_u16(ptr); v = assigned; assigned };
vm_block = 39; continue;
}
// C line 60086
41 => {
return JS_NewInt32(ctx, ((*(ptr)) as i32));
}
// C line 60084
42 => {
return JS_NewInt32(ctx, ((*(((ptr) as *mut i8))) as i32));
}
// C line 60082
43 => {
vm_block = match class_id { x if x == (JS_CLASS_FLOAT64_ARRAY as i32) => 5, x if x == (JS_CLASS_FLOAT32_ARRAY as i32) => 10, x if x == (JS_CLASS_FLOAT16_ARRAY as i32) => 14, x if x == (JS_CLASS_BIG_UINT64_ARRAY as i32) => 19, x if x == (JS_CLASS_BIG_INT64_ARRAY as i32) => 24, x if x == (JS_CLASS_UINT32_ARRAY as i32) => 28, x if x == (JS_CLASS_INT32_ARRAY as i32) => 32, x if x == (JS_CLASS_UINT16_ARRAY as i32) => 36, x if x == (JS_CLASS_INT16_ARRAY as i32) => 40, x if x == (JS_CLASS_UINT8_ARRAY as i32) => 41, x if x == (JS_CLASS_INT8_ARRAY as i32) => 42, _ => 1, }; continue;
}
// C line 60080
44 => {
let _ = { let assigned = (((*(abuf)).data).offset((((*(ta)).offset) as isize))).offset(((pos) as isize)); ptr = assigned; assigned };
vm_block = 43; continue;
}
// C line 60079
45 => {
return JS_ThrowTypeError(ctx, c"out of bound".as_ptr());
}
// C line 60078
46 => {
vm_block = if ((((((((*(ta)).offset) as i64)).wrapping_add((((*(ta)).length) as i64))) > ((((*(abuf)).byte_length) as i64))) as i32)) != 0 { 45 } else { 44 }; continue;
}
// C line 60074
47 => {
return JS_ThrowRangeError(ctx, c"out of bound".as_ptr());
}
// C line 60073
48 => {
vm_block = if (((((pos).wrapping_add(((size) as u64))) > ((((*(ta)).length) as u64))) as i32)) != 0 { 47 } else { 46 }; continue;
}
// C line 60071
49 => {
return JS_ThrowTypeErrorDetachedArrayBuffer(ctx);
}
// C line 60070
50 => {
vm_block = if ((*(abuf)).detached) != 0 { 49 } else { 48 }; continue;
}
// C line 60069
51 => {
let _ = { let assigned = ((*((*(ta)).buffer)).u).array_buffer; abuf = assigned; assigned };
vm_block = 50; continue;
}
// C line 60068
52 => {
let _ = { let assigned = ((littleEndian) ^ ((!((is_be()) != 0) as i32))); is_swap = assigned; assigned };
vm_block = 51; continue;
}
// C line 60067
53 => {
let _ = { let assigned = (((((((argc) > ((1 as i32))) as i32)) != 0) && ((JS_ToBool(ctx, *(argv).offset(((1 as i32)) as isize))) != 0)) as i32); littleEndian = assigned; assigned };
vm_block = 52; continue;
}
// C line 60066
54 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 60065
55 => {
vm_block = if (JS_ToIndex(ctx, core::ptr::addr_of_mut!(pos), *(argv).offset(((0 as i32)) as isize))) != 0 { 54 } else { 53 }; continue;
}
// C line 60064
56 => {
let _ = { let assigned = ((1 as i32)).wrapping_shl((((*((typed_array_size_log2).as_ptr()).offset(((class_id).wrapping_sub((JS_CLASS_UINT8C_ARRAY as i32))) as isize)) as i32)) as u32); size = assigned; assigned };
vm_block = 55; continue;
}
// C line 60063
57 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 60062
58 => {
vm_block = if ((!(!(ta).is_null()) as i32)) != 0 { 57 } else { 56 }; continue;
}
// C line 60061
59 => {
let _ = { let assigned = ((JS_GetOpaque2(ctx, this_obj, (((JS_CLASS_DATAVIEW as i32)) as JSClassID))) as *mut JSTypedArray); ta = assigned; assigned };
vm_block = 58; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:60161. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_dataview_setValue(mut ctx: *mut JSContext, mut this_obj: JSValue, mut argc: i32, mut argv: *mut JSValue, mut class_id: i32) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut ta: *mut JSTypedArray = core::mem::zeroed();
let mut abuf: *mut JSArrayBuffer = core::mem::zeroed();
let mut littleEndian: i32 = core::mem::zeroed();
let mut is_swap: i32 = core::mem::zeroed();
let mut size: i32 = core::mem::zeroed();
let mut ptr: *mut u8 = core::mem::zeroed();
let mut v64: u64 = core::mem::zeroed();
let mut v: u32 = core::mem::zeroed();
let mut pos: u64 = core::mem::zeroed();
let mut val: JSValue = core::mem::zeroed();
let mut d: f64 = core::mem::zeroed();
let mut u: BufferRecord60197_13 = core::mem::zeroed();
let mut u_1: JSFloat64Union = core::mem::zeroed();
let mut vm_block: usize = 51;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 60253
1 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) };
}
// C line ?
2 => {
let _ = std::process::abort();
vm_block = 1; continue;
}
// C line 60249
3 => {
vm_block = 1; continue;
}
// C line 60248
4 => {
let _ = crate::cutils_header::put_u64(ptr, v64);
vm_block = 3; continue;
}
// C line 60247
5 => {
let _ = { let assigned = crate::cutils_header::bswap64(v64); v64 = assigned; assigned };
vm_block = 4; continue;
}
// C line 60246
6 => {
vm_block = if (is_swap) != 0 { 5 } else { 4 }; continue;
}
// C line 60242
7 => {
vm_block = 1; continue;
}
// C line 60241
8 => {
let _ = crate::cutils_header::put_u32(ptr, v);
vm_block = 7; continue;
}
// C line 60240
9 => {
let _ = { let assigned = crate::cutils_header::bswap32(v); v = assigned; assigned };
vm_block = 8; continue;
}
// C line 60239
10 => {
vm_block = if (is_swap) != 0 { 9 } else { 8 }; continue;
}
// C line 60235
11 => {
vm_block = 1; continue;
}
// C line 60234
12 => {
let _ = crate::cutils_header::put_u16(ptr, ((v) as u16));
vm_block = 11; continue;
}
// C line 60233
13 => {
let _ = { let assigned = ((crate::cutils_header::bswap16(((v) as u16))) as u32); v = assigned; assigned };
vm_block = 12; continue;
}
// C line 60232
14 => {
vm_block = if (is_swap) != 0 { 13 } else { 12 }; continue;
}
// C line 60228
15 => {
vm_block = 1; continue;
}
// C line 60227
16 => {
let _ = { let assigned = ((v) as u8); *(ptr) = assigned; assigned };
vm_block = 15; continue;
}
// C line 60224
17 => {
vm_block = match class_id { x if x == (JS_CLASS_FLOAT64_ARRAY as i32) => 6, x if x == (JS_CLASS_BIG_UINT64_ARRAY as i32) => 6, x if x == (JS_CLASS_BIG_INT64_ARRAY as i32) => 6, x if x == (JS_CLASS_FLOAT32_ARRAY as i32) => 10, x if x == (JS_CLASS_UINT32_ARRAY as i32) => 10, x if x == (JS_CLASS_INT32_ARRAY as i32) => 10, x if x == (JS_CLASS_FLOAT16_ARRAY as i32) => 14, x if x == (JS_CLASS_UINT16_ARRAY as i32) => 14, x if x == (JS_CLASS_INT16_ARRAY as i32) => 14, x if x == (JS_CLASS_UINT8_ARRAY as i32) => 16, x if x == (JS_CLASS_INT8_ARRAY as i32) => 16, _ => 2, }; continue;
}
// C line 60222
18 => {
let _ = { let assigned = (((*(abuf)).data).offset((((*(ta)).offset) as isize))).offset(((pos) as isize)); ptr = assigned; assigned };
vm_block = 17; continue;
}
// C line 60221
19 => {
return JS_ThrowTypeError(ctx, c"out of bound".as_ptr());
}
// C line 60220
20 => {
vm_block = if ((((((((*(ta)).offset) as i64)).wrapping_add((((*(ta)).length) as i64))) > ((((*(abuf)).byte_length) as i64))) as i32)) != 0 { 19 } else { 18 }; continue;
}
// C line 60216
21 => {
return JS_ThrowRangeError(ctx, c"out of bound".as_ptr());
}
// C line 60215
22 => {
vm_block = if (((((pos).wrapping_add(((size) as u64))) > ((((*(ta)).length) as u64))) as i32)) != 0 { 21 } else { 20 }; continue;
}
// C line 60213
23 => {
return JS_ThrowTypeErrorDetachedArrayBuffer(ctx);
}
// C line 60212
24 => {
vm_block = if ((*(abuf)).detached) != 0 { 23 } else { 22 }; continue;
}
// C line 60211
25 => {
let _ = { let assigned = ((*((*(ta)).buffer)).u).array_buffer; abuf = assigned; assigned };
vm_block = 24; continue;
}
// C line 60210
26 => {
let _ = { let assigned = ((littleEndian) ^ ((!((is_be()) != 0) as i32))); is_swap = assigned; assigned };
vm_block = 25; continue;
}
// C line 60209
27 => {
let _ = { let assigned = (((((((argc) > ((2 as i32))) as i32)) != 0) && ((JS_ToBool(ctx, *(argv).offset(((2 as i32)) as isize))) != 0)) as i32); littleEndian = assigned; assigned };
vm_block = 26; continue;
}
// C line 60186
28 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 60185
29 => {
vm_block = if (JS_ToUint32(ctx, core::ptr::addr_of_mut!(v), val)) != 0 { 28 } else { 27 }; continue;
}
// C line 60189
30 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 60188
31 => {
vm_block = if (JS_ToBigInt64(ctx, ((core::ptr::addr_of_mut!(v64)) as *mut i64), val)) != 0 { 30 } else { 27 }; continue;
}
// C line 60195
32 => {
let _ = { let assigned = ((crate::cutils_header::tofp16(d)) as u32); v = assigned; assigned };
vm_block = 27; continue;
}
// C line 60202
33 => {
let _ = { let assigned = (u).i; v = assigned; assigned };
vm_block = 27; continue;
}
// C line 60201
34 => {
let _ = { let assigned = ((d) as f32); (u).f = assigned; assigned };
vm_block = 33; continue;
}
// C line 60206
35 => {
let _ = { let assigned = (u_1).u64; v64 = assigned; assigned };
vm_block = 27; continue;
}
// C line 60205
36 => {
let _ = { let assigned = d; (u_1).d = assigned; assigned };
vm_block = 35; continue;
}
// C line 60196
37 => {
vm_block = if ((((class_id) == ((JS_CLASS_FLOAT32_ARRAY as i32))) as i32)) != 0 { 34 } else { 36 }; continue;
}
// C line 60194
38 => {
vm_block = if ((((class_id) == ((JS_CLASS_FLOAT16_ARRAY as i32))) as i32)) != 0 { 32 } else { 37 }; continue;
}
// C line 60193
39 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 60192
40 => {
vm_block = if (JS_ToFloat64(ctx, core::ptr::addr_of_mut!(d), val)) != 0 { 39 } else { 38 }; continue;
}
// C line 60187
41 => {
vm_block = if ((((class_id) <= ((JS_CLASS_BIG_UINT64_ARRAY as i32))) as i32)) != 0 { 31 } else { 40 }; continue;
}
// C line 60184
42 => {
vm_block = if ((((class_id) <= ((JS_CLASS_UINT32_ARRAY as i32))) as i32)) != 0 { 29 } else { 41 }; continue;
}
// C line 60183
43 => {
let _ = { let assigned = (((0 as i32)) as u64); v64 = assigned; assigned };
vm_block = 42; continue;
}
// C line 60182
44 => {
let _ = { let assigned = (((0 as i32)) as u32); v = assigned; assigned };
vm_block = 43; continue;
}
// C line 60181
45 => {
let _ = { let assigned = *(argv).offset(((1 as i32)) as isize); val = assigned; assigned };
vm_block = 44; continue;
}
// C line 60180
46 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 60179
47 => {
vm_block = if (JS_ToIndex(ctx, core::ptr::addr_of_mut!(pos), *(argv).offset(((0 as i32)) as isize))) != 0 { 46 } else { 45 }; continue;
}
// C line 60178
48 => {
let _ = { let assigned = ((1 as i32)).wrapping_shl((((*((typed_array_size_log2).as_ptr()).offset(((class_id).wrapping_sub((JS_CLASS_UINT8C_ARRAY as i32))) as isize)) as i32)) as u32); size = assigned; assigned };
vm_block = 47; continue;
}
// C line 60177
49 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 60176
50 => {
vm_block = if ((!(!(ta).is_null()) as i32)) != 0 { 49 } else { 48 }; continue;
}
// C line 60175
51 => {
let _ = { let assigned = ((JS_GetOpaque2(ctx, this_obj, (((JS_CLASS_DATAVIEW as i32)) as JSClassID))) as *mut JSTypedArray); ta = assigned; assigned };
vm_block = 50; continue;
}
_ => std::process::abort(),
} }
}
