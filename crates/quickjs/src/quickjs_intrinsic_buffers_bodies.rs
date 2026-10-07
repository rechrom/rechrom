// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:56512. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_array_buffer_constructor3(mut ctx: *mut JSContext, mut new_target: JSValue, mut len: u64, mut max_len: *mut u64, mut class_id: JSClassID, mut buf: *mut u8, mut free_func: Option<JSFreeArrayBufferDataFunc>, mut opaque: *mut c_void, mut alloc_flag: i32) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut rt: *mut JSRuntime = core::mem::zeroed();
let mut obj: JSValue = core::mem::zeroed();
let mut abuf: *mut JSArrayBuffer = core::mem::zeroed();
let mut sab_alloc_len: u64 = core::mem::zeroed();
let mut vm_block: usize = 43;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 56585
1 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 56584
2 => {
let _ = js_free(ctx, ((abuf) as *mut c_void));
vm_block = 1; continue;
}
// C line ? labels: fail
3 => {
let _ = JS_FreeValue(ctx, obj);
vm_block = 2; continue;
}
// C line 56581
4 => {
return obj;
}
// C line 56580
5 => {
let _ = JS_SetOpaque(obj, ((abuf) as *mut c_void));
vm_block = 4; continue;
}
// C line 56579
6 => {
let _ = { let dst = ((((*(abuf)).data) as *mut c_void)) as *mut u8; core::ptr::copy_nonoverlapping((((buf) as *const c_void)) as *const u8, dst, (((len) as usize)) as usize); dst as *mut c_void };
vm_block = 5; continue;
}
// C line 56578
7 => {
vm_block = if (((((alloc_flag) != 0) && (!(buf).is_null())) as i32)) != 0 { 6 } else { 5 }; continue;
}
// C line 56577
8 => {
let _ = { let assigned = free_func; (*(abuf)).free_func = assigned; assigned };
vm_block = 7; continue;
}
// C line 56576
9 => {
let _ = { let assigned = opaque; (*(abuf)).opaque = assigned; assigned };
vm_block = 8; continue;
}
// C line 56575
10 => {
let _ = { let assigned = (((((class_id) == ((((JS_CLASS_SHARED_ARRAY_BUFFER as i32)) as JSClassID))) as i32)) as u8); (*(abuf)).shared = assigned; assigned };
vm_block = 9; continue;
}
// C line 56574
11 => {
let _ = { let assigned = (((0 as i32)) as u8); (*(abuf)).detached = assigned; assigned };
vm_block = 10; continue;
}
// C line 56573
12 => {
let _ = init_list_head(core::ptr::addr_of_mut!((*(abuf)).array_list));
vm_block = 11; continue;
}
// C line 56559
13 => {
let _ = { let dst = ((((*(abuf)).data) as *mut c_void)) as *mut u8; core::ptr::write_bytes(dst, ((0 as i32)) as u8, (((sab_alloc_len) as usize)) as usize); dst as *mut c_void };
vm_block = 12; continue;
}
// C line 56558
14 => {
vm_block = 3; continue;
}
// C line 56557
15 => {
vm_block = if ((!(!((*(abuf)).data).is_null()) as i32)) != 0 { 14 } else { 13 }; continue;
}
// C line 56555
16 => {
let _ = { let assigned = (((((*(rt)).sab_funcs).sab_alloc).expect("registered buffer callback")(((*(rt)).sab_funcs).sab_opaque, ((max_int(((sab_alloc_len) as i32), (1 as i32))) as usize))) as *mut u8); (*(abuf)).data = assigned; assigned };
vm_block = 15; continue;
}
// C line 56554
17 => {
let _ = { let assigned = if !(max_len).is_null() { *(max_len) } else { len }; sab_alloc_len = assigned; assigned };
vm_block = 16; continue;
}
// C line 56564
18 => {
vm_block = 3; continue;
}
// C line 56563
19 => {
vm_block = if ((!(!((*(abuf)).data).is_null()) as i32)) != 0 { 18 } else { 12 }; continue;
}
// C line 56562
20 => {
let _ = { let assigned = ((js_mallocz(ctx, ((max_int(((len) as i32), (1 as i32))) as usize))) as *mut u8); (*(abuf)).data = assigned; assigned };
vm_block = 19; continue;
}
// C line 56550
21 => {
vm_block = if ((((((((class_id) == ((((JS_CLASS_SHARED_ARRAY_BUFFER as i32)) as JSClassID))) as i32)) != 0) && ((((*(rt)).sab_funcs).sab_alloc).is_some())) as i32)) != 0 { 17 } else { 20 }; continue;
}
// C line 56571
22 => {
let _ = { let assigned = buf; (*(abuf)).data = assigned; assigned };
vm_block = 12; continue;
}
// C line 56569
23 => {
let _ = (((*(rt)).sab_funcs).sab_dup).expect("registered buffer callback")(((*(rt)).sab_funcs).sab_opaque, ((buf) as *mut c_void));
vm_block = 22; continue;
}
// C line 56567
24 => {
vm_block = if ((((((((class_id) == ((((JS_CLASS_SHARED_ARRAY_BUFFER as i32)) as JSClassID))) as i32)) != 0) && ((((*(rt)).sab_funcs).sab_dup).is_some())) as i32)) != 0 { 23 } else { 22 }; continue;
}
// C line 56549
25 => {
vm_block = if (alloc_flag) != 0 { 21 } else { 24 }; continue;
}
// C line 56548
26 => {
let _ = { let assigned = ((if !(max_len).is_null() { *(max_len) } else { ((((1 as i32)).wrapping_neg()) as u64) }) as i32); (*(abuf)).max_byte_length = assigned; assigned };
vm_block = 25; continue;
}
// C line 56547
27 => {
let _ = { let assigned = ((len) as i32); (*(abuf)).byte_length = assigned; assigned };
vm_block = 26; continue;
}
// C line 56546
28 => {
vm_block = 3; continue;
}
// C line 56545
29 => {
vm_block = if ((!(!(abuf).is_null()) as i32)) != 0 { 28 } else { 27 }; continue;
}
// C line 56544
30 => {
let _ = { let assigned = ((js_malloc(ctx, (size_of::<JSArrayBuffer>() as usize))) as *mut JSArrayBuffer); abuf = assigned; assigned };
vm_block = 29; continue;
}
// C line 56542
31 => {
vm_block = 3; continue;
}
// C line 56541
32 => {
let _ = JS_ThrowRangeError(ctx, c"invalid max array buffer length".as_ptr());
vm_block = 31; continue;
}
// C line 56540
33 => {
vm_block = if ((((!(max_len).is_null()) && (((((*(max_len)) > ((((2147483647 as i32)) as u64))) as i32)) != 0)) as i32)) != 0 { 32 } else { 30 }; continue;
}
// C line 56538
34 => {
vm_block = 3; continue;
}
// C line 56537
35 => {
let _ = JS_ThrowRangeError(ctx, c"invalid array buffer length".as_ptr());
vm_block = 34; continue;
}
// C line 56536
36 => {
vm_block = if ((((len) > ((((2147483647 as i32)) as u64))) as i32)) != 0 { 35 } else { 33 }; continue;
}
// C line 56534
37 => {
return obj;
}
// C line 56533
38 => {
vm_block = if (JS_IsException(obj)) != 0 { 37 } else { 36 }; continue;
}
// C line 56532
39 => {
let _ = { let assigned = js_create_from_ctor(ctx, new_target, ((class_id) as i32)); obj = assigned; assigned };
vm_block = 38; continue;
}
// C line 56528
40 => {
return JS_ThrowInternalError(ctx, c"resizable ArrayBuffers not supported for externally managed buffers".as_ptr());
}
// C line 56525
41 => {
vm_block = if ((((((((((((((!((alloc_flag) != 0) as i32)) != 0) && (!(buf).is_null())) as i32)) != 0) && (!(max_len).is_null())) as i32)) != 0) && ((((free_func).map(|f|f as usize) != Some((js_array_buffer_free) as *const () as usize)) as i32) != 0)) as i32)) != 0 { 40 } else { 39 }; continue;
}
// C line 56522
42 => {
abuf = core::ptr::null_mut::<JSArrayBuffer>();
vm_block = 41; continue;
}
// C line 56520
43 => {
rt = (*(ctx)).rt;
vm_block = 42; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:56588. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_array_buffer_free(mut rt: *mut JSRuntime, mut opaque: *mut c_void, mut ptr: *mut c_void) -> () {
let mut vm_local_storage = Vec::<u64>::new();
let mut vm_block: usize = 1;
loop { match vm_block {
// C line ?
0 => {
return ();
}
// C line 56590
1 => {
let _ = js_free_rt(rt, ptr);
vm_block = 0; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:56593. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_array_buffer_constructor2(mut ctx: *mut JSContext, mut new_target: JSValue, mut len: u64, mut max_len: *mut u64, mut class_id: JSClassID) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut vm_block: usize = 1;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 56598
1 => {
return js_array_buffer_constructor3(ctx, new_target, len, max_len, class_id, core::ptr::null_mut::<u8>(), Some(js_array_buffer_free), core::ptr::null_mut::<c_void>(), (1 as i32));
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:56603. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_array_buffer_constructor1(mut ctx: *mut JSContext, mut new_target: JSValue, mut len: u64, mut max_len: *mut u64) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut vm_block: usize = 1;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 56607
1 => {
return js_array_buffer_constructor2(ctx, new_target, len, max_len, (((JS_CLASS_ARRAY_BUFFER as i32)) as JSClassID));
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:56611. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
pub unsafe fn JS_NewArrayBuffer(mut ctx: *mut JSContext, mut buf: *mut u8, mut len: usize, mut free_func: Option<JSFreeArrayBufferDataFunc>, mut opaque: *mut c_void, mut is_shared: i32) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut class_id: JSClassID = core::mem::zeroed();
let mut vm_block: usize = 2;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 56617
1 => {
return js_array_buffer_constructor3(ctx, JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }, ((len) as u64), core::ptr::null_mut::<u64>(), class_id, buf, free_func, opaque, (0 as i32));
}
// C line 56615
2 => {
class_id = ((if (is_shared) != 0 { (JS_CLASS_SHARED_ARRAY_BUFFER as i32) } else { (JS_CLASS_ARRAY_BUFFER as i32) }) as JSClassID);
vm_block = 1; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:56622. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
pub unsafe fn JS_NewArrayBufferCopy(mut ctx: *mut JSContext, mut buf: *const u8, mut len: usize) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut vm_block: usize = 1;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 56624
1 => {
return js_array_buffer_constructor3(ctx, JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }, ((len) as u64), core::ptr::null_mut::<u64>(), (((JS_CLASS_ARRAY_BUFFER as i32)) as JSClassID), ((buf) as *mut u8), Some(js_array_buffer_free), core::ptr::null_mut::<c_void>(), (1 as i32));
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:56631. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_array_buffer_constructor0(mut ctx: *mut JSContext, mut new_target: JSValue, mut argc: i32, mut argv: *mut JSValue, mut class_id: JSClassID) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut len: u64 = core::mem::zeroed();
let mut max_len: u64 = core::mem::zeroed();
let mut pmax_len: *mut u64 = core::mem::zeroed();
let mut obj: JSValue = core::mem::zeroed();
let mut val: JSValue = core::mem::zeroed();
let mut i: i64 = core::mem::zeroed();
let mut vm_block: usize = 23;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 56662 labels: next
1 => {
return js_array_buffer_constructor2(ctx, new_target, len, pmax_len, class_id);
}
// C line 56660
2 => {
let _ = { let assigned = core::ptr::addr_of_mut!(max_len); pmax_len = assigned; assigned };
vm_block = 1; continue;
}
// C line 56659
3 => {
let _ = { let assigned = ((i) as u64); max_len = assigned; assigned };
vm_block = 2; continue;
}
// C line 56658
4 => {
return JS_ThrowRangeError(ctx, c"invalid array buffer max length".as_ptr());
}
// C line 56657
5 => {
vm_block = if ((((((((len) > (((i) as u64))) as i32)) != 0) || (((((i) > ((((((1 as i32)) as i64)).wrapping_shl(((53 as i32)) as u32)).wrapping_sub((((1 as i32)) as i64)))) as i32)) != 0)) as i32)) != 0 { 4 } else { 3 }; continue;
}
// C line 56655
6 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 56654
7 => {
vm_block = if (JS_ToInt64Free(ctx, core::ptr::addr_of_mut!(i), val)) != 0 { 6 } else { 5 }; continue;
}
// C line 56653
8 => {
vm_block = 1; continue;
}
// C line 56652
9 => {
vm_block = if (JS_IsUndefined(val)) != 0 { 8 } else { 7 }; continue;
}
// C line 56651
10 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 56650
11 => {
vm_block = if (JS_IsException(val)) != 0 { 10 } else { 9 }; continue;
}
// C line 56649
12 => {
let _ = JS_FreeValue(ctx, obj);
vm_block = 11; continue;
}
// C line 56648
13 => {
let _ = { let assigned = JS_GetProperty(ctx, obj, (((crate::quickjs_atom::JS_ATOM_maxByteLength as i32)) as JSAtom)); val = assigned; assigned };
vm_block = 12; continue;
}
// C line 56647
14 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 56646
15 => {
vm_block = if (JS_IsException(obj)) != 0 { 14 } else { 13 }; continue;
}
// C line 56645
16 => {
let _ = { let assigned = JS_ToObject(ctx, *(argv).offset(((1 as i32)) as isize)); obj = assigned; assigned };
vm_block = 15; continue;
}
// C line 56644
17 => {
vm_block = 1; continue;
}
// C line 56643
18 => {
vm_block = if ((!((JS_IsObject(*(argv).offset(((1 as i32)) as isize))) != 0) as i32)) != 0 { 17 } else { 16 }; continue;
}
// C line 56642
19 => {
vm_block = 1; continue;
}
// C line 56641
20 => {
vm_block = if ((((argc) < ((2 as i32))) as i32)) != 0 { 19 } else { 18 }; continue;
}
// C line 56640
21 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 56639
22 => {
vm_block = if (JS_ToIndex(ctx, core::ptr::addr_of_mut!(len), *(argv).offset(((0 as i32)) as isize))) != 0 { 21 } else { 20 }; continue;
}
// C line 56635
23 => {
pmax_len = core::ptr::null_mut::<u64>();
vm_block = 22; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:56666. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_array_buffer_constructor(mut ctx: *mut JSContext, mut new_target: JSValue, mut argc: i32, mut argv: *mut JSValue) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut vm_block: usize = 1;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 56670
1 => {
return js_array_buffer_constructor0(ctx, new_target, argc, argv, (((JS_CLASS_ARRAY_BUFFER as i32)) as JSClassID));
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:56674. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_shared_array_buffer_constructor(mut ctx: *mut JSContext, mut new_target: JSValue, mut argc: i32, mut argv: *mut JSValue) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut vm_block: usize = 1;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 56678
1 => {
return js_array_buffer_constructor0(ctx, new_target, argc, argv, (((JS_CLASS_SHARED_ARRAY_BUFFER as i32)) as JSClassID));
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:56717. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_array_buffer_isView(mut ctx: *mut JSContext, mut this_val: JSValue, mut argc: i32, mut argv: *mut JSValue) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut p: *mut JSObject = core::mem::zeroed();
let mut res: i32 = core::mem::zeroed();
let mut vm_block: usize = 6;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 56731
1 => {
return JS_NewBool(ctx, res);
}
// C line 56728
2 => {
let _ = { let assigned = (1 as i32); res = assigned; assigned };
vm_block = 1; continue;
}
// C line 56726
3 => {
vm_block = if (((((((((((*(p)).class_id) as i32)) >= ((JS_CLASS_UINT8C_ARRAY as i32))) as i32)) != 0) && ((((((((*(p)).class_id) as i32)) <= ((JS_CLASS_DATAVIEW as i32))) as i32)) != 0)) as i32)) != 0 { 2 } else { 1 }; continue;
}
// C line 56725
4 => {
let _ = { let assigned = ((((*(argv).offset(((0 as i32)) as isize)).u).ptr) as *mut JSObject); p = assigned; assigned };
vm_block = 3; continue;
}
// C line 56724
5 => {
vm_block = if (((((((*(argv).offset(((0 as i32)) as isize)).tag) as i32)) == ((JS_TAG_OBJECT as i32))) as i32)) != 0 { 4 } else { 1 }; continue;
}
// C line 56723
6 => {
let _ = { let assigned = (0 as i32); res = assigned; assigned };
vm_block = 5; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:56739. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn JS_ThrowTypeErrorDetachedArrayBuffer(mut ctx: *mut JSContext) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut vm_block: usize = 1;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 56741
1 => {
return JS_ThrowTypeError(ctx, c"ArrayBuffer is detached".as_ptr());
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:56750. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_array_buffer_get_detached(mut ctx: *mut JSContext, mut this_val: JSValue) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut abuf: *mut JSArrayBuffer = core::mem::zeroed();
let mut vm_block: usize = 6;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 56758
1 => {
return JS_NewBool(ctx, (((*(abuf)).detached) as i32));
}
// C line 56757
2 => {
return JS_ThrowTypeError(ctx, c"detached called on SharedArrayBuffer".as_ptr());
}
// C line 56756
3 => {
vm_block = if ((*(abuf)).shared) != 0 { 2 } else { 1 }; continue;
}
// C line 56755
4 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 56754
5 => {
vm_block = if ((!(!(abuf).is_null()) as i32)) != 0 { 4 } else { 3 }; continue;
}
// C line 56753
6 => {
abuf = ((JS_GetOpaque2(ctx, this_val, (((JS_CLASS_ARRAY_BUFFER as i32)) as JSClassID))) as *mut JSArrayBuffer);
vm_block = 5; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:56761. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_array_buffer_get_byteLength(mut ctx: *mut JSContext, mut this_val: JSValue, mut class_id: i32) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut abuf: *mut JSArrayBuffer = core::mem::zeroed();
let mut vm_block: usize = 4;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 56769
1 => {
return JS_NewUint32(ctx, (((*(abuf)).byte_length) as u32));
}
// C line 56767
2 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 56766
3 => {
vm_block = if ((!(!(abuf).is_null()) as i32)) != 0 { 2 } else { 1 }; continue;
}
// C line 56765
4 => {
abuf = ((JS_GetOpaque2(ctx, this_val, ((class_id) as JSClassID))) as *mut JSArrayBuffer);
vm_block = 3; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:56772. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_array_buffer_get_maxByteLength(mut ctx: *mut JSContext, mut this_val: JSValue, mut class_id: i32) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut abuf: *mut JSArrayBuffer = core::mem::zeroed();
let mut vm_block: usize = 6;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 56781
1 => {
return JS_NewUint32(ctx, (((*(abuf)).byte_length) as u32));
}
// C line 56780
2 => {
return JS_NewUint32(ctx, (((*(abuf)).max_byte_length) as u32));
}
// C line 56779
3 => {
vm_block = if (array_buffer_is_resizable(abuf)) != 0 { 2 } else { 1 }; continue;
}
// C line 56778
4 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 56777
5 => {
vm_block = if ((!(!(abuf).is_null()) as i32)) != 0 { 4 } else { 3 }; continue;
}
// C line 56776
6 => {
abuf = ((JS_GetOpaque2(ctx, this_val, ((class_id) as JSClassID))) as *mut JSArrayBuffer);
vm_block = 5; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:56784. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_array_buffer_get_resizable(mut ctx: *mut JSContext, mut this_val: JSValue, mut class_id: i32) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut abuf: *mut JSArrayBuffer = core::mem::zeroed();
let mut vm_block: usize = 4;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 56791
1 => {
return JS_NewBool(ctx, array_buffer_is_resizable(abuf));
}
// C line 56790
2 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 56789
3 => {
vm_block = if ((!(!(abuf).is_null()) as i32)) != 0 { 2 } else { 1 }; continue;
}
// C line 56788
4 => {
abuf = ((JS_GetOpaque2(ctx, this_val, ((class_id) as JSClassID))) as *mut JSArrayBuffer);
vm_block = 3; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:56794. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_array_buffer_update_typed_arrays(mut abuf: *mut JSArrayBuffer) -> () {
let mut vm_local_storage = Vec::<u64>::new();
let mut size_log2: u32 = core::mem::zeroed();
let mut size_elem: u32 = core::mem::zeroed();
let mut el: *mut list_head = core::mem::zeroed();
let mut ta: *mut JSTypedArray = core::mem::zeroed();
let mut p: *mut JSObject = core::mem::zeroed();
let mut data: *mut u8 = core::mem::zeroed();
let mut len: i64 = core::mem::zeroed();
let mut vm_block: usize = 23;
loop { match vm_block {
// C line ?
0 => {
return ();
}
// C line 56806
1 => {
vm_block = if ((((el) != (core::ptr::addr_of_mut!((*(abuf)).array_list))) as i32)) != 0 { 20 } else { 0 }; continue;
}
// C line ?
2 => {
let _ = { let assigned = (*(el)).next; el = assigned; assigned };
vm_block = 1; continue;
}
// C line 56812
3 => {
let _ = { let assigned = (((len).wrapping_sub((((*(ta)).offset) as i64))) as u32); (*(ta)).length = assigned; assigned };
vm_block = 2; continue;
}
// C line 56814
4 => {
let _ = { let assigned = (((0 as i32)) as u32); (*(ta)).length = assigned; assigned };
vm_block = 2; continue;
}
// C line 56811
5 => {
vm_block = if (((((((*(ta)).offset) as i64)) < (len)) as i32)) != 0 { 3 } else { 4 }; continue;
}
// C line 56810
6 => {
vm_block = if ((*(ta)).track_rab) != 0 { 5 } else { 2 }; continue;
}
// C line 56824
7 => {
let _ = { let assigned = ((core::ptr::addr_of_mut!(*(data).offset(((*(ta)).offset) as isize))) as *mut c_void); ((((*(p)).u).array).u).ptr = assigned; assigned };
vm_block = 2; continue;
}
// C line 56823
8 => {
let _ = { let assigned = ((((len).wrapping_sub((((*(ta)).offset) as i64))).wrapping_shr((size_log2) as u32)) as u32); (((*(p)).u).array).count = assigned; assigned };
vm_block = 7; continue;
}
// C line 56822
9 => {
vm_block = if ((((len) >= (((((*(ta)).offset) as i64)).wrapping_add(((size_elem) as i64)))) as i32)) != 0 { 8 } else { 2 }; continue;
}
// C line 56829
10 => {
let _ = { let assigned = ((core::ptr::addr_of_mut!(*(data).offset(((*(ta)).offset) as isize))) as *mut c_void); ((((*(p)).u).array).u).ptr = assigned; assigned };
vm_block = 2; continue;
}
// C line 56828
11 => {
let _ = { let assigned = ((*(ta)).length).wrapping_shr((size_log2) as u32); (((*(p)).u).array).count = assigned; assigned };
vm_block = 10; continue;
}
// C line 56827
12 => {
vm_block = if ((((len) >= (((((*(ta)).offset) as i64)).wrapping_add((((*(ta)).length) as i64)))) as i32)) != 0 { 11 } else { 2 }; continue;
}
// C line 56821
13 => {
vm_block = if ((*(ta)).track_rab) != 0 { 9 } else { 12 }; continue;
}
// C line 56820
14 => {
let _ = { let assigned = ((((1 as i32)).wrapping_shl((size_log2) as u32)) as u32); size_elem = assigned; assigned };
vm_block = 13; continue;
}
// C line 56819
15 => {
let _ = { let assigned = ((*((typed_array_size_log2).as_ptr()).offset((((((*(p)).class_id) as i32)).wrapping_sub((JS_CLASS_UINT8C_ARRAY as i32))) as isize)) as u32); size_log2 = assigned; assigned };
vm_block = 14; continue;
}
// C line 56818
16 => {
let _ = { let assigned = core::ptr::null_mut::<c_void>(); ((((*(p)).u).array).u).ptr = assigned; assigned };
vm_block = 15; continue;
}
// C line 56817
17 => {
let _ = { let assigned = (((0 as i32)) as u32); (((*(p)).u).array).count = assigned; assigned };
vm_block = 16; continue;
}
// C line 56809
18 => {
vm_block = if (((((((*(p)).class_id) as i32)) == ((JS_CLASS_DATAVIEW as i32))) as i32)) != 0 { 6 } else { 17 }; continue;
}
// C line 56808
19 => {
let _ = { let assigned = (*(ta)).obj; p = assigned; assigned };
vm_block = 18; continue;
}
// C line 56807
20 => {
let _ = { let assigned = (((((el) as *mut u8)).offset(-(((core::mem::offset_of!(JSTypedArray, link) as usize)) as isize))) as *mut JSTypedArray); ta = assigned; assigned };
vm_block = 19; continue;
}
// C line ?
21 => {
let _ = { let assigned = (*(core::ptr::addr_of_mut!((*(abuf)).array_list))).next; el = assigned; assigned };
vm_block = 1; continue;
}
// C line 56804
22 => {
let _ = { let assigned = (*(abuf)).data; data = assigned; assigned };
vm_block = 21; continue;
}
// C line 56803
23 => {
let _ = { let assigned = (((*(abuf)).byte_length) as i64); len = assigned; assigned };
vm_block = 22; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:56837. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
pub unsafe fn JS_DetachArrayBuffer(mut ctx: *mut JSContext, mut obj: JSValue) -> () {
let mut vm_local_storage = Vec::<u64>::new();
let mut abuf: *mut JSArrayBuffer = core::mem::zeroed();
let mut vm_block: usize = 9;
loop { match vm_block {
// C line ?
0 => {
return ();
}
// C line 56848
1 => {
let _ = js_array_buffer_update_typed_arrays(abuf);
vm_block = 0; continue;
}
// C line 56847
2 => {
let _ = { let assigned = (((1 as i32)) as u8); (*(abuf)).detached = assigned; assigned };
vm_block = 1; continue;
}
// C line 56846
3 => {
let _ = { let assigned = (0 as i32); (*(abuf)).byte_length = assigned; assigned };
vm_block = 2; continue;
}
// C line 56845
4 => {
let _ = { let assigned = core::ptr::null_mut::<u8>(); (*(abuf)).data = assigned; assigned };
vm_block = 3; continue;
}
// C line 56844
5 => {
let _ = ((*(abuf)).free_func).expect("registered buffer callback")((*(ctx)).rt, (*(abuf)).opaque, (((*(abuf)).data) as *mut c_void));
vm_block = 4; continue;
}
// C line 56843
6 => {
vm_block = if ((*(abuf)).free_func).is_some() { 5 } else { 4 }; continue;
}
// C line 56842
7 => {
return;
}
// C line 56841
8 => {
vm_block = if ((((((!(!(abuf).is_null()) as i32)) != 0) || (((((*(abuf)).detached) as i32)) != 0)) as i32)) != 0 { 7 } else { 6 }; continue;
}
// C line 56839
9 => {
abuf = ((JS_GetOpaque(obj, (((JS_CLASS_ARRAY_BUFFER as i32)) as JSClassID))) as *mut JSArrayBuffer);
vm_block = 8; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:56852. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_get_array_buffer(mut ctx: *mut JSContext, mut obj: JSValue) -> *mut JSArrayBuffer {
let mut vm_local_storage = Vec::<u64>::new();
let mut p: *mut JSObject = core::mem::zeroed();
let mut vm_block: usize = 7;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 56864
1 => {
return ((*(p)).u).array_buffer;
}
// C line 56862
2 => {
return core::ptr::null_mut::<JSArrayBuffer>();
}
// C line ? labels: fail
3 => {
let _ = JS_ThrowTypeErrorInvalidClass(ctx, (JS_CLASS_ARRAY_BUFFER as i32));
vm_block = 2; continue;
}
// C line 56858
4 => {
vm_block = if (((((((((((*(p)).class_id) as i32)) != ((JS_CLASS_ARRAY_BUFFER as i32))) as i32)) != 0) && ((((((((*(p)).class_id) as i32)) != ((JS_CLASS_SHARED_ARRAY_BUFFER as i32))) as i32)) != 0)) as i32)) != 0 { 3 } else { 1 }; continue;
}
// C line 56857
5 => {
let _ = { let assigned = ((((obj).u).ptr) as *mut JSObject); p = assigned; assigned };
vm_block = 4; continue;
}
// C line 56856
6 => {
vm_block = 3; continue;
}
// C line 56855
7 => {
vm_block = if (((((((obj).tag) as i32)) != ((JS_TAG_OBJECT as i32))) as i32)) != 0 { 6 } else { 5 }; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:56869. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
pub unsafe fn JS_GetArrayBuffer(mut ctx: *mut JSContext, mut psize: *mut usize, mut obj: JSValue) -> *mut u8 {
let mut vm_local_storage = Vec::<u64>::new();
let mut abuf: *mut JSArrayBuffer = core::mem::zeroed();
let mut vm_block: usize = 10;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 56882
1 => {
return core::ptr::null_mut::<u8>();
}
// C line ? labels: fail
2 => {
let _ = { let assigned = (((0 as i32)) as usize); *(psize) = assigned; assigned };
vm_block = 1; continue;
}
// C line 56879
3 => {
return (*(abuf)).data;
}
// C line 56878
4 => {
let _ = { let assigned = (((*(abuf)).byte_length) as usize); *(psize) = assigned; assigned };
vm_block = 3; continue;
}
// C line 56876
5 => {
vm_block = 2; continue;
}
// C line 56875
6 => {
let _ = JS_ThrowTypeErrorDetachedArrayBuffer(ctx);
vm_block = 5; continue;
}
// C line 56874
7 => {
vm_block = if ((*(abuf)).detached) != 0 { 6 } else { 4 }; continue;
}
// C line 56873
8 => {
vm_block = 2; continue;
}
// C line 56872
9 => {
vm_block = if ((!(!(abuf).is_null()) as i32)) != 0 { 8 } else { 7 }; continue;
}
// C line 56871
10 => {
abuf = js_get_array_buffer(ctx, obj);
vm_block = 9; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:56891. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_array_buffer_transfer(mut ctx: *mut JSContext, mut this_val: JSValue, mut argc: i32, mut argv: *mut JSValue, mut transfer_to_fixed_length: i32) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut abuf: *mut JSArrayBuffer = core::mem::zeroed();
let mut new_len: u64 = core::mem::zeroed();
let mut pmax_len: *mut u64 = core::mem::zeroed();
let mut max_len: u64 = core::mem::zeroed();
let mut res: JSValue = core::mem::zeroed();
let mut old_len: u64 = core::mem::zeroed();
let mut new_abuf: *mut JSArrayBuffer = core::mem::zeroed();
let mut new_abuf_1: *mut JSArrayBuffer = core::mem::zeroed();
let mut new_bs: *mut u8 = core::mem::zeroed();
let mut vm_block: usize = 56;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 56986
1 => {
return res;
}
// C line 56928
2 => {
let _ = JS_DetachArrayBuffer(ctx, this_val);
vm_block = 1; continue;
}
// C line 56927
3 => {
return res;
}
// C line 56926
4 => {
vm_block = if (JS_IsException(res)) != 0 { 3 } else { 2 }; continue;
}
// C line 56925
5 => {
let _ = { let assigned = js_array_buffer_constructor2(ctx, JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }, (((0 as i32)) as u64), pmax_len, (((JS_CLASS_ARRAY_BUFFER as i32)) as JSClassID)); res = assigned; assigned };
vm_block = 4; continue;
}
// C line 56984
6 => {
let _ = js_array_buffer_update_typed_arrays(abuf);
vm_block = 1; continue;
}
// C line 56983
7 => {
let _ = { let assigned = (((1 as i32)) as u8); (*(abuf)).detached = assigned; assigned };
vm_block = 6; continue;
}
// C line 56982
8 => {
let _ = { let assigned = (0 as i32); (*(abuf)).byte_length = assigned; assigned };
vm_block = 7; continue;
}
// C line 56981
9 => {
let _ = { let assigned = core::ptr::null_mut::<u8>(); (*(abuf)).data = assigned; assigned };
vm_block = 8; continue;
}
// C line 56949
10 => {
let _ = ((*(abuf)).free_func).expect("registered buffer callback")((*(ctx)).rt, (*(abuf)).opaque, (((*(abuf)).data) as *mut c_void));
vm_block = 9; continue;
}
// C line 56948
11 => {
let _ = { let dst = ((((*(new_abuf)).data) as *mut c_void)) as *mut u8; core::ptr::copy_nonoverlapping(((((*(abuf)).data) as *const c_void)) as *const u8, dst, (((min_int(((old_len) as i32), ((new_len) as i32))) as usize)) as usize); dst as *mut c_void };
vm_block = 10; continue;
}
// C line 56947
12 => {
let _ = { let assigned = ((JS_GetOpaque2(ctx, res, (((JS_CLASS_ARRAY_BUFFER as i32)) as JSClassID))) as *mut JSArrayBuffer); new_abuf = assigned; assigned };
vm_block = 11; continue;
}
// C line 56946
13 => {
return res;
}
// C line 56945
14 => {
vm_block = if (JS_IsException(res)) != 0 { 13 } else { 12 }; continue;
}
// C line 56944
15 => {
let _ = { let assigned = js_array_buffer_constructor2(ctx, JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }, new_len, pmax_len, (((JS_CLASS_ARRAY_BUFFER as i32)) as JSClassID)); res = assigned; assigned };
vm_block = 14; continue;
}
// C line 56969
16 => {
let _ = { let assigned = ((new_len) as i32); (*(new_abuf_1)).byte_length = assigned; assigned };
vm_block = 9; continue;
}
// C line 56968
17 => {
let _ = { let assigned = new_bs; (*(new_abuf_1)).data = assigned; assigned };
vm_block = 16; continue;
}
// C line 56967
18 => {
let _ = js_free(ctx, (((*(new_abuf_1)).data) as *mut c_void));
vm_block = 17; continue;
}
// C line 56966
19 => {
let _ = { let assigned = ((JS_GetOpaque2(ctx, res, (((JS_CLASS_ARRAY_BUFFER as i32)) as JSClassID))) as *mut JSArrayBuffer); new_abuf_1 = assigned; assigned };
vm_block = 18; continue;
}
// C line 56965
20 => {
let _ = { let dst = ((((new_bs).offset(((old_len) as isize))) as *mut c_void)) as *mut u8; core::ptr::write_bytes(dst, ((0 as i32)) as u8, ((((new_len).wrapping_sub(old_len)) as usize)) as usize); dst as *mut c_void };
vm_block = 19; continue;
}
// C line 56964
21 => {
vm_block = if ((((new_len) > (old_len)) as i32)) != 0 { 20 } else { 19 }; continue;
}
// C line 56962
22 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 56961
23 => {
let _ = JS_FreeValue(ctx, res);
vm_block = 22; continue;
}
// C line 56960
24 => {
vm_block = if ((!(!(new_bs).is_null()) as i32)) != 0 { 23 } else { 21 }; continue;
}
// C line 56959
25 => {
let _ = { let assigned = ((js_realloc(ctx, (((*(abuf)).data) as *mut c_void), ((new_len) as usize))) as *mut u8); new_bs = assigned; assigned };
vm_block = 24; continue;
}
// C line 56958
26 => {
return res;
}
// C line 56957
27 => {
vm_block = if (JS_IsException(res)) != 0 { 26 } else { 25 }; continue;
}
// C line 56956
28 => {
let _ = { let assigned = js_array_buffer_constructor2(ctx, JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }, (((0 as i32)) as u64), pmax_len, (((JS_CLASS_ARRAY_BUFFER as i32)) as JSClassID)); res = assigned; assigned };
vm_block = 27; continue;
}
// C line 56940
29 => {
vm_block = if ((((*(abuf)).free_func).map(|f|f as usize) != Some((js_array_buffer_free) as *const () as usize)) as i32) != 0 { 15 } else { 28 }; continue;
}
// C line 56938
30 => {
return JS_ThrowRangeError(ctx, c"invalid array buffer length".as_ptr());
}
// C line 56937
31 => {
vm_block = if ((((new_len) > ((((2147483647 as i32)) as u64))) as i32)) != 0 { 30 } else { 29 }; continue;
}
// C line 56978
32 => {
return res;
}
// C line 56977
33 => {
vm_block = if (JS_IsException(res)) != 0 { 32 } else { 9 }; continue;
}
// C line 56973
34 => {
let _ = { let assigned = js_array_buffer_constructor3(ctx, JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }, new_len, pmax_len, (((JS_CLASS_ARRAY_BUFFER as i32)) as JSClassID), (*(abuf)).data, (*(abuf)).free_func, (*(abuf)).opaque, (0 as i32)); res = assigned; assigned };
vm_block = 33; continue;
}
// C line 56935
35 => {
vm_block = if ((((new_len) != (old_len)) as i32)) != 0 { 31 } else { 34 }; continue;
}
// C line 56932
36 => {
let _ = { let assigned = (((*(abuf)).byte_length) as u64); old_len = assigned; assigned };
vm_block = 35; continue;
}
// C line 56924
37 => {
vm_block = if ((((new_len) == ((((0 as i32)) as u64))) as i32)) != 0 { 5 } else { 36 }; continue;
}
// C line 56919
38 => {
let _ = { let assigned = core::ptr::addr_of_mut!(max_len); pmax_len = assigned; assigned };
vm_block = 37; continue;
}
// C line 56918
39 => {
vm_block = if ((((*(abuf)).free_func).map(|f|f as usize) == Some((js_array_buffer_free) as *const () as usize)) as i32) != 0 { 38 } else { 37 }; continue;
}
// C line 56916
40 => {
return JS_ThrowTypeError(ctx, c"invalid array buffer length".as_ptr());
}
// C line 56915
41 => {
vm_block = if ((((new_len) > (max_len)) as i32)) != 0 { 40 } else { 39 }; continue;
}
// C line 56914
42 => {
let _ = { let assigned = (((*(abuf)).max_byte_length) as u64); max_len = assigned; assigned };
vm_block = 41; continue;
}
// C line 56913
43 => {
vm_block = if (array_buffer_is_resizable(abuf)) != 0 { 42 } else { 37 }; continue;
}
// C line 56912
44 => {
vm_block = if ((!((transfer_to_fixed_length) != 0) as i32)) != 0 { 43 } else { 37 }; continue;
}
// C line 56911
45 => {
let _ = { let assigned = core::ptr::null_mut::<u64>(); pmax_len = assigned; assigned };
vm_block = 44; continue;
}
// C line 56910
46 => {
return JS_ThrowTypeErrorDetachedArrayBuffer(ctx);
}
// C line 56909
47 => {
vm_block = if ((*(abuf)).detached) != 0 { 46 } else { 45 }; continue;
}
// C line 56906
48 => {
let _ = { let assigned = (((*(abuf)).byte_length) as u64); new_len = assigned; assigned };
vm_block = 47; continue;
}
// C line 56908
49 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 56907
50 => {
vm_block = if (JS_ToIndex(ctx, core::ptr::addr_of_mut!(new_len), *(argv).offset(((0 as i32)) as isize))) != 0 { 49 } else { 47 }; continue;
}
// C line 56905
51 => {
vm_block = if ((((((((argc) < ((1 as i32))) as i32)) != 0) || ((JS_IsUndefined(*(argv).offset(((0 as i32)) as isize))) != 0)) as i32)) != 0 { 48 } else { 50 }; continue;
}
// C line 56904
52 => {
return JS_ThrowTypeError(ctx, c"cannot transfer a SharedArrayBuffer".as_ptr());
}
// C line 56903
53 => {
vm_block = if ((*(abuf)).shared) != 0 { 52 } else { 51 }; continue;
}
// C line 56902
54 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 56901
55 => {
vm_block = if ((!(!(abuf).is_null()) as i32)) != 0 { 54 } else { 53 }; continue;
}
// C line 56900
56 => {
let _ = { let assigned = ((JS_GetOpaque2(ctx, this_val, (((JS_CLASS_ARRAY_BUFFER as i32)) as JSClassID))) as *mut JSArrayBuffer); abuf = assigned; assigned };
vm_block = 55; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:56989. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_array_buffer_resize(mut ctx: *mut JSContext, mut this_val: JSValue, mut argc: i32, mut argv: *mut JSValue, mut class_id: i32) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut abuf: *mut JSArrayBuffer = core::mem::zeroed();
let mut data: *mut u8 = core::mem::zeroed();
let mut len: i64 = core::mem::zeroed();
let mut vm_block: usize = 26;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 57036
1 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) };
}
// C line 57035
2 => {
let _ = js_array_buffer_update_typed_arrays(abuf);
vm_block = 1; continue;
}
// C line 57025
3 => {
let _ = { let assigned = ((len) as i32); (*(abuf)).byte_length = assigned; assigned };
vm_block = 2; continue;
}
// C line 57017
4 => {
vm_block = 14; continue;
}
// C line 57016
5 => {
vm_block = if ((((len) < ((((*(abuf)).byte_length) as i64))) as i32)) != 0 { 4 } else { 3 }; continue;
}
// C line 57033
6 => {
let _ = { let assigned = data; (*(abuf)).data = assigned; assigned };
vm_block = 2; continue;
}
// C line 57032
7 => {
let _ = { let assigned = ((len) as i32); (*(abuf)).byte_length = assigned; assigned };
vm_block = 6; continue;
}
// C line 57031
8 => {
let _ = { let dst = (((core::ptr::addr_of_mut!(*(data).offset(((*(abuf)).byte_length) as isize))) as *mut c_void)) as *mut u8; core::ptr::write_bytes(dst, ((0 as i32)) as u8, ((((len).wrapping_sub((((*(abuf)).byte_length) as i64))) as usize)) as usize); dst as *mut c_void };
vm_block = 7; continue;
}
// C line 57030
9 => {
vm_block = if ((((len) > ((((*(abuf)).byte_length) as i64))) as i32)) != 0 { 8 } else { 7 }; continue;
}
// C line 57029
10 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 57028
11 => {
vm_block = if ((!(!(data).is_null()) as i32)) != 0 { 10 } else { 9 }; continue;
}
// C line 57027
12 => {
let _ = { let assigned = ((js_realloc(ctx, (((*(abuf)).data) as *mut c_void), ((max_int(((len) as i32), (1 as i32))) as usize))) as *mut u8); data = assigned; assigned };
vm_block = 11; continue;
}
// C line 57015
13 => {
vm_block = if ((*(abuf)).shared) != 0 { 5 } else { 12 }; continue;
}
// C line ? labels: bad_length
14 => {
return JS_ThrowRangeError(ctx, c"invalid array buffer length".as_ptr());
}
// C line 57008
15 => {
vm_block = if ((((((((len) < ((((0 as i32)) as i64))) as i32)) != 0) || (((((len) > ((((*(abuf)).max_byte_length) as i64))) as i32)) != 0)) as i32)) != 0 { 14 } else { 13 }; continue;
}
// C line 57007
16 => {
return JS_ThrowTypeError(ctx, c"external array buffer is not resizable".as_ptr());
}
// C line 57006
17 => {
vm_block = if ((((*(abuf)).free_func).map(|f|f as usize) != Some((js_array_buffer_free) as *const () as usize)) as i32) != 0 { 16 } else { 15 }; continue;
}
// C line 57004
18 => {
return JS_ThrowTypeError(ctx, c"array buffer is not resizable".as_ptr());
}
// C line 57003
19 => {
vm_block = if ((!((array_buffer_is_resizable(abuf)) != 0) as i32)) != 0 { 18 } else { 17 }; continue;
}
// C line 57002
20 => {
return JS_ThrowTypeErrorDetachedArrayBuffer(ctx);
}
// C line 57001
21 => {
vm_block = if ((*(abuf)).detached) != 0 { 20 } else { 19 }; continue;
}
// C line 57000
22 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 56999
23 => {
vm_block = if (JS_ToInt64(ctx, core::ptr::addr_of_mut!(len), *(argv).offset(((0 as i32)) as isize))) != 0 { 22 } else { 21 }; continue;
}
// C line 56998
24 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 56997
25 => {
vm_block = if ((!(!(abuf).is_null()) as i32)) != 0 { 24 } else { 23 }; continue;
}
// C line 56996
26 => {
let _ = { let assigned = ((JS_GetOpaque2(ctx, this_val, ((class_id) as JSClassID))) as *mut JSArrayBuffer); abuf = assigned; assigned };
vm_block = 25; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:57039. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_array_buffer_slice(mut ctx: *mut JSContext, mut this_val: JSValue, mut argc: i32, mut argv: *mut JSValue, mut class_id: i32) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut abuf: *mut JSArrayBuffer = core::mem::zeroed();
let mut new_abuf: *mut JSArrayBuffer = core::mem::zeroed();
let mut len: i64 = core::mem::zeroed();
let mut start: i64 = core::mem::zeroed();
let mut end: i64 = core::mem::zeroed();
let mut new_len: i64 = core::mem::zeroed();
let mut ctor: JSValue = core::mem::zeroed();
let mut new_obj: JSValue = core::mem::zeroed();
let mut args: [JSValue; 1] = core::mem::zeroed();
let mut vm_block: usize = 43;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 57102
1 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line ? labels: fail
2 => {
let _ = JS_FreeValue(ctx, new_obj);
vm_block = 1; continue;
}
// C line 57099
3 => {
return new_obj;
}
// C line 57098
4 => {
let _ = { let dst = ((((*(new_abuf)).data) as *mut c_void)) as *mut u8; core::ptr::copy_nonoverlapping((((((*(abuf)).data).offset(((start) as isize))) as *const c_void)) as *const u8, dst, (((new_len) as usize)) as usize); dst as *mut c_void };
vm_block = 3; continue;
}
// C line 57096
5 => {
vm_block = 2; continue;
}
// C line 57095
6 => {
let _ = JS_ThrowTypeErrorDetachedArrayBuffer(ctx);
vm_block = 5; continue;
}
// C line 57094
7 => {
vm_block = if ((((((((*(abuf)).detached) as i32)) != 0) || ((((((((*(abuf)).byte_length) as i64)) < ((start).wrapping_add(new_len))) as i32)) != 0)) as i32)) != 0 { 6 } else { 4 }; continue;
}
// C line 57091
8 => {
vm_block = 2; continue;
}
// C line 57090
9 => {
let _ = JS_ThrowTypeError(ctx, c"new ArrayBuffer is too small".as_ptr());
vm_block = 8; continue;
}
// C line 57089
10 => {
vm_block = if (((((((*(new_abuf)).byte_length) as i64)) < (new_len)) as i32)) != 0 { 9 } else { 7 }; continue;
}
// C line 57087
11 => {
vm_block = 2; continue;
}
// C line 57086
12 => {
let _ = JS_ThrowTypeErrorDetachedArrayBuffer(ctx);
vm_block = 11; continue;
}
// C line 57085
13 => {
vm_block = if ((*(new_abuf)).detached) != 0 { 12 } else { 10 }; continue;
}
// C line 57083
14 => {
vm_block = 2; continue;
}
// C line 57082
15 => {
let _ = JS_ThrowTypeError(ctx, c"cannot use identical ArrayBuffer".as_ptr());
vm_block = 14; continue;
}
// C line 57081
16 => {
vm_block = if (js_same_value(ctx, new_obj, this_val)) != 0 { 15 } else { 13 }; continue;
}
// C line 57080
17 => {
vm_block = 2; continue;
}
// C line 57079
18 => {
vm_block = if ((!(!(new_abuf).is_null()) as i32)) != 0 { 17 } else { 16 }; continue;
}
// C line 57078
19 => {
let _ = { let assigned = ((JS_GetOpaque2(ctx, new_obj, ((class_id) as JSClassID))) as *mut JSArrayBuffer); new_abuf = assigned; assigned };
vm_block = 18; continue;
}
// C line 57077
20 => {
return new_obj;
}
// C line 57076
21 => {
vm_block = if (JS_IsException(new_obj)) != 0 { 20 } else { 19 }; continue;
}
// C line 57067
22 => {
let _ = { let assigned = js_array_buffer_constructor2(ctx, JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }, ((new_len) as u64), core::ptr::null_mut::<u64>(), ((class_id) as JSClassID)); new_obj = assigned; assigned };
vm_block = 21; continue;
}
// C line 57074
23 => {
let _ = JS_FreeValue(ctx, *((args).as_mut_ptr()).offset(((0 as i32)) as isize));
vm_block = 21; continue;
}
// C line 57073
24 => {
let _ = JS_FreeValue(ctx, ctor);
vm_block = 23; continue;
}
// C line 57072
25 => {
let _ = { let assigned = JS_CallConstructor(ctx, ctor, (1 as i32), (args).as_mut_ptr()); new_obj = assigned; assigned };
vm_block = 24; continue;
}
// C line 57071
26 => {
let _ = { let assigned = JS_NewInt64(ctx, new_len); *((args).as_mut_ptr()).offset(((0 as i32)) as isize) = assigned; assigned };
vm_block = 25; continue;
}
// C line 57066
27 => {
vm_block = if (JS_IsUndefined(ctor)) != 0 { 22 } else { 26 }; continue;
}
// C line 57065
28 => {
return ctor;
}
// C line 57064
29 => {
vm_block = if (JS_IsException(ctor)) != 0 { 28 } else { 27 }; continue;
}
// C line 57063
30 => {
let _ = { let assigned = JS_SpeciesConstructor(ctx, this_val, JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }); ctor = assigned; assigned };
vm_block = 29; continue;
}
// C line 57062
31 => {
let _ = { let assigned = crate::cutils_header::max_int64((end).wrapping_sub(start), (((0 as i32)) as i64)); new_len = assigned; assigned };
vm_block = 30; continue;
}
// C line 57060
32 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 57059
33 => {
vm_block = if (JS_ToInt64Clamp(ctx, core::ptr::addr_of_mut!(end), *(argv).offset(((1 as i32)) as isize), (((0 as i32)) as i64), len, len)) != 0 { 32 } else { 31 }; continue;
}
// C line 57058
34 => {
vm_block = if ((!((JS_IsUndefined(*(argv).offset(((1 as i32)) as isize))) != 0) as i32)) != 0 { 33 } else { 31 }; continue;
}
// C line 57057
35 => {
let _ = { let assigned = len; end = assigned; assigned };
vm_block = 34; continue;
}
// C line 57055
36 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 57054
37 => {
vm_block = if (JS_ToInt64Clamp(ctx, core::ptr::addr_of_mut!(start), *(argv).offset(((0 as i32)) as isize), (((0 as i32)) as i64), len, len)) != 0 { 36 } else { 35 }; continue;
}
// C line 57052
38 => {
let _ = { let assigned = (((*(abuf)).byte_length) as i64); len = assigned; assigned };
vm_block = 37; continue;
}
// C line 57051
39 => {
return JS_ThrowTypeErrorDetachedArrayBuffer(ctx);
}
// C line 57050
40 => {
vm_block = if ((*(abuf)).detached) != 0 { 39 } else { 38 }; continue;
}
// C line 57049
41 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 57048
42 => {
vm_block = if ((!(!(abuf).is_null()) as i32)) != 0 { 41 } else { 40 }; continue;
}
// C line 57047
43 => {
let _ = { let assigned = ((JS_GetOpaque2(ctx, this_val, ((class_id) as JSClassID))) as *mut JSArrayBuffer); abuf = assigned; assigned };
vm_block = 42; continue;
}
_ => std::process::abort(),
} }
}
