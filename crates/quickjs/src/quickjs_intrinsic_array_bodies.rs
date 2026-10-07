// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:41482. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn JS_CopySubArray(mut ctx: *mut JSContext, mut obj: JSValue, mut to_pos: i64, mut from_pos: i64, mut count: i64, mut dir: i32) -> i32 {
let mut vm_local_storage = Vec::<u64>::new();
let mut p: *mut JSObject = core::mem::zeroed();
let mut i: i64 = core::mem::zeroed();
let mut from: i64 = core::mem::zeroed();
let mut to: i64 = core::mem::zeroed();
let mut len: i64 = core::mem::zeroed();
let mut val: JSValue = core::mem::zeroed();
let mut fromPresent: i32 = core::mem::zeroed();
let mut l: i64 = core::mem::zeroed();
let mut j: i64 = core::mem::zeroed();
let mut vm_block: usize = 39;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line ? labels: exception
1 => {
return ((1 as i32)).wrapping_neg();
}
// C line 41546
2 => {
return (0 as i32);
}
// C line 41499
3 => {
vm_block = if ((((i) < (count)) as i32)) != 0 { 33 } else { 2 }; continue;
}
// C line 41530
4 => {
let _ = { i = (i).wrapping_add(l); i };
vm_block = 3; continue;
}
// C line 41518
5 => {
vm_block = if ((((j) < (l)) as i32)) != 0 { 7 } else { 4 }; continue;
}
// C line ?
6 => {
let _ = { let old = j; j = (j).wrapping_add(1); old };
vm_block = 5; continue;
}
// C line 41519
7 => {
let _ = set_value(ctx, core::ptr::addr_of_mut!(*(((((*(p)).u).array).u).values).offset(((to).wrapping_sub(j)) as isize)), JS_DupValue(ctx, *(((((*(p)).u).array).u).values).offset(((from).wrapping_sub(j)) as isize)));
vm_block = 6; continue;
}
// C line 41518
8 => {
let _ = { let assigned = (((0 as i32)) as i64); j = assigned; assigned };
vm_block = 5; continue;
}
// C line 41517
9 => {
let _ = { let assigned = crate::cutils_header::min_int64(l, (to).wrapping_add((((1 as i32)) as i64))); l = assigned; assigned };
vm_block = 8; continue;
}
// C line 41516
10 => {
let _ = { let assigned = crate::cutils_header::min_int64(l, (from).wrapping_add((((1 as i32)) as i64))); l = assigned; assigned };
vm_block = 9; continue;
}
// C line 41525
11 => {
vm_block = if ((((j) < (l)) as i32)) != 0 { 13 } else { 4 }; continue;
}
// C line ?
12 => {
let _ = { let old = j; j = (j).wrapping_add(1); old };
vm_block = 11; continue;
}
// C line 41526
13 => {
let _ = set_value(ctx, core::ptr::addr_of_mut!(*(((((*(p)).u).array).u).values).offset(((to).wrapping_add(j)) as isize)), JS_DupValue(ctx, *(((((*(p)).u).array).u).values).offset(((from).wrapping_add(j)) as isize)));
vm_block = 12; continue;
}
// C line 41525
14 => {
let _ = { let assigned = (((0 as i32)) as i64); j = assigned; assigned };
vm_block = 11; continue;
}
// C line 41524
15 => {
let _ = { let assigned = crate::cutils_header::min_int64(l, (len).wrapping_sub(to)); l = assigned; assigned };
vm_block = 14; continue;
}
// C line 41523
16 => {
let _ = { let assigned = crate::cutils_header::min_int64(l, (len).wrapping_sub(from)); l = assigned; assigned };
vm_block = 15; continue;
}
// C line 41515
17 => {
vm_block = if ((((dir) < ((0 as i32))) as i32)) != 0 { 10 } else { 16 }; continue;
}
// C line 41514
18 => {
let _ = { let assigned = (count).wrapping_sub(i); l = assigned; assigned };
vm_block = 17; continue;
}
// C line 41543
19 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 3; continue;
}
// C line 41538
20 => {
vm_block = 1; continue;
}
// C line 41537
21 => {
vm_block = if ((((JS_SetPropertyInt64(ctx, obj, to, val)) < ((0 as i32))) as i32)) != 0 { 20 } else { 19 }; continue;
}
// C line 41541
22 => {
vm_block = 1; continue;
}
// C line 41540
23 => {
vm_block = if ((((JS_DeletePropertyInt64(ctx, obj, to, ((1 as i32)).wrapping_shl(((14 as i32)) as u32))) < ((0 as i32))) as i32)) != 0 { 22 } else { 19 }; continue;
}
// C line 41536
24 => {
vm_block = if (fromPresent) != 0 { 21 } else { 23 }; continue;
}
// C line 41534
25 => {
vm_block = 1; continue;
}
// C line 41533
26 => {
vm_block = if ((((fromPresent) < ((0 as i32))) as i32)) != 0 { 25 } else { 24 }; continue;
}
// C line 41532
27 => {
let _ = { let assigned = JS_TryGetPropertyInt64(ctx, obj, from, core::ptr::addr_of_mut!(val)); fromPresent = assigned; assigned };
vm_block = 26; continue;
}
// C line 41507
28 => {
vm_block = if ((((((((((((((((((((!(p).is_null()) && (((((*(p)).fast_array()) as i32)) != 0)) as i32)) != 0) && (((((from) >= ((((0 as i32)) as i64))) as i32)) != 0)) as i32)) != 0) && (((((from) < ({ let assigned = (((((*(p)).u).array).count) as i64); len = assigned; assigned })) as i32)) != 0)) as i32)) != 0) && (((((to) >= ((((0 as i32)) as i64))) as i32)) != 0)) as i32)) != 0) && (((((to) < (len)) as i32)) != 0)) as i32)) != 0 { 18 } else { 27 }; continue;
}
// C line 41502
29 => {
let _ = { let assigned = (((to_pos).wrapping_add(count)).wrapping_sub(i)).wrapping_sub((((1 as i32)) as i64)); to = assigned; assigned };
vm_block = 28; continue;
}
// C line 41501
30 => {
let _ = { let assigned = (((from_pos).wrapping_add(count)).wrapping_sub(i)).wrapping_sub((((1 as i32)) as i64)); from = assigned; assigned };
vm_block = 29; continue;
}
// C line 41505
31 => {
let _ = { let assigned = (to_pos).wrapping_add(i); to = assigned; assigned };
vm_block = 28; continue;
}
// C line 41504
32 => {
let _ = { let assigned = (from_pos).wrapping_add(i); from = assigned; assigned };
vm_block = 31; continue;
}
// C line 41500
33 => {
vm_block = if ((((dir) < ((0 as i32))) as i32)) != 0 { 30 } else { 32 }; continue;
}
// C line 41499
34 => {
let _ = { let assigned = (((0 as i32)) as i64); i = assigned; assigned };
vm_block = 3; continue;
}
// C line 41495
35 => {
let _ = { let assigned = core::ptr::null_mut::<JSObject>(); p = assigned; assigned };
vm_block = 34; continue;
}
// C line 41494
36 => {
vm_block = if (((((((((((*(p)).class_id) as i32)) != ((JS_CLASS_ARRAY as i32))) as i32)) != 0) || (((!(((*(p)).fast_array()) != 0) as i32)) != 0)) as i32)) != 0 { 35 } else { 34 }; continue;
}
// C line 41493
37 => {
let _ = { let assigned = ((((obj).u).ptr) as *mut JSObject); p = assigned; assigned };
vm_block = 36; continue;
}
// C line 41492
38 => {
vm_block = if (((((((obj).tag) as i32)) == ((JS_TAG_OBJECT as i32))) as i32)) != 0 { 37 } else { 34 }; continue;
}
// C line 41491
39 => {
let _ = { let assigned = core::ptr::null_mut::<JSObject>(); p = assigned; assigned };
vm_block = 38; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:41552. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_array_constructor(mut ctx: *mut JSContext, mut new_target: JSValue, mut argc: i32, mut argv: *mut JSValue) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut obj: JSValue = core::mem::zeroed();
let mut i: i32 = core::mem::zeroed();
let mut len: u32 = core::mem::zeroed();
let mut vm_block: usize = 16;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 41576
1 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line ? labels: fail
2 => {
let _ = JS_FreeValue(ctx, obj);
vm_block = 1; continue;
}
// C line 41573
3 => {
return obj;
}
// C line 41566
4 => {
vm_block = 2; continue;
}
// C line 41565
5 => {
vm_block = if ((((JS_SetProperty(ctx, obj, (((crate::quickjs_atom::JS_ATOM_length as i32)) as JSAtom), JS_NewUint32(ctx, len))) < ((0 as i32))) as i32)) != 0 { 4 } else { 3 }; continue;
}
// C line 41564
6 => {
vm_block = 2; continue;
}
// C line 41563
7 => {
vm_block = if (JS_ToArrayLengthFree(ctx, core::ptr::addr_of_mut!(len), JS_DupValue(ctx, *(argv).offset(((0 as i32)) as isize)), (1 as i32))) != 0 { 6 } else { 5 }; continue;
}
// C line 41568
8 => {
vm_block = if ((((i) < (argc)) as i32)) != 0 { 11 } else { 3 }; continue;
}
// C line ?
9 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 8; continue;
}
// C line 41570
10 => {
vm_block = 2; continue;
}
// C line 41569
11 => {
vm_block = if ((((JS_SetPropertyUint32(ctx, obj, ((i) as u32), JS_DupValue(ctx, *(argv).offset((i) as isize)))) < ((0 as i32))) as i32)) != 0 { 10 } else { 9 }; continue;
}
// C line 41568
12 => {
let _ = { let assigned = (0 as i32); i = assigned; assigned };
vm_block = 8; continue;
}
// C line 41561
13 => {
vm_block = if ((((((((argc) == ((1 as i32))) as i32)) != 0) && ((JS_IsNumber(*(argv).offset(((0 as i32)) as isize))) != 0)) as i32)) != 0 { 7 } else { 12 }; continue;
}
// C line 41560
14 => {
return obj;
}
// C line 41559
15 => {
vm_block = if (JS_IsException(obj)) != 0 { 14 } else { 13 }; continue;
}
// C line 41558
16 => {
let _ = { let assigned = js_create_from_ctor(ctx, new_target, (JS_CLASS_ARRAY as i32)); obj = assigned; assigned };
vm_block = 15; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:41579. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_array_from(mut ctx: *mut JSContext, mut this_val: JSValue, mut argc: i32, mut argv: *mut JSValue) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut items: JSValue = core::mem::zeroed();
let mut mapfn: JSValue = core::mem::zeroed();
let mut this_arg: JSValue = core::mem::zeroed();
let mut args: [JSValue; 2] = core::mem::zeroed();
let mut iter: JSValue = core::mem::zeroed();
let mut r: JSValue = core::mem::zeroed();
let mut v: JSValue = core::mem::zeroed();
let mut v2: JSValue = core::mem::zeroed();
let mut arrayLike: JSValue = core::mem::zeroed();
let mut next_method: JSValue = core::mem::zeroed();
let mut enum_obj: JSValue = core::mem::zeroed();
let mut k: i64 = core::mem::zeroed();
let mut len: i64 = core::mem::zeroed();
let mut done: i32 = core::mem::zeroed();
let mut mapping: i32 = core::mem::zeroed();
let mut vm_block: usize = 93;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 41695
1 => {
return r;
}
// C line 41694
2 => {
let _ = JS_FreeValue(ctx, next_method);
vm_block = 1; continue;
}
// C line 41693
3 => {
let _ = JS_FreeValue(ctx, enum_obj);
vm_block = 2; continue;
}
// C line 41692
4 => {
let _ = JS_FreeValue(ctx, iter);
vm_block = 3; continue;
}
// C line ? labels: done
5 => {
let _ = JS_FreeValue(ctx, arrayLike);
vm_block = 4; continue;
}
// C line 41689
6 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) }; r = assigned; assigned };
vm_block = 5; continue;
}
// C line ? labels: exception
7 => {
let _ = JS_FreeValue(ctx, r);
vm_block = 6; continue;
}
// C line ? labels: exception_close
8 => {
let _ = JS_IteratorClose(ctx, enum_obj, (1 as i32));
vm_block = 7; continue;
}
// C line 41683
9 => {
vm_block = 5; continue;
}
// C line 41682
10 => {
vm_block = 7; continue;
}
// C line 41681
11 => {
vm_block = if ((((JS_SetProperty(ctx, r, (((crate::quickjs_atom::JS_ATOM_length as i32)) as JSAtom), JS_NewUint32(ctx, ((k) as u32)))) < ((0 as i32))) as i32)) != 0 { 10 } else { 9 }; continue;
}
// C line 41628
12 => {
vm_block = 28; continue;
}
// C line ?
13 => {
let _ = { let old = k; k = (k).wrapping_add(1); old };
vm_block = 12; continue;
}
// C line 41645
14 => {
vm_block = 8; continue;
}
// C line 41643
15 => {
vm_block = if ((((JS_DefinePropertyValueInt64(ctx, r, k, v, ((((((((1 as i32)).wrapping_shl(((0 as i32)) as u32)) | (((1 as i32)).wrapping_shl(((1 as i32)) as u32)))) | (((1 as i32)).wrapping_shl(((2 as i32)) as u32)))) | (((1 as i32)).wrapping_shl(((14 as i32)) as u32))))) < ((0 as i32))) as i32)) != 0 { 14 } else { 13 }; continue;
}
// C line 41641
16 => {
vm_block = 8; continue;
}
// C line 41640
17 => {
vm_block = if (JS_IsException(v)) != 0 { 16 } else { 15 }; continue;
}
// C line 41639
18 => {
let _ = { let assigned = v2; v = assigned; assigned };
vm_block = 17; continue;
}
// C line 41638
19 => {
let _ = JS_FreeValue(ctx, v);
vm_block = 18; continue;
}
// C line 41637
20 => {
let _ = { let assigned = JS_Call(ctx, mapfn, this_arg, (2 as i32), (args).as_mut_ptr()); v2 = assigned; assigned };
vm_block = 19; continue;
}
// C line 41636
21 => {
let _ = { let assigned = JS_NewInt32(ctx, ((k) as i32)); *((args).as_mut_ptr()).offset(((1 as i32)) as isize) = assigned; assigned };
vm_block = 20; continue;
}
// C line 41635
22 => {
let _ = { let assigned = v; *((args).as_mut_ptr()).offset(((0 as i32)) as isize) = assigned; assigned };
vm_block = 21; continue;
}
// C line 41634
23 => {
vm_block = if (mapping) != 0 { 22 } else { 15 }; continue;
}
// C line 41633
24 => {
vm_block = 11; continue;
}
// C line 41632
25 => {
vm_block = if (done) != 0 { 24 } else { 23 }; continue;
}
// C line 41631
26 => {
vm_block = 7; continue;
}
// C line 41630
27 => {
vm_block = if (JS_IsException(v)) != 0 { 26 } else { 25 }; continue;
}
// C line 41629
28 => {
let _ = { let assigned = JS_IteratorNext(ctx, enum_obj, next_method, (0 as i32), core::ptr::null_mut::<JSValue>(), core::ptr::addr_of_mut!(done)); v = assigned; assigned };
vm_block = 27; continue;
}
// C line 41628
29 => {
let _ = { let assigned = (((0 as i32)) as i64); k = assigned; assigned };
vm_block = 12; continue;
}
// C line 41627
30 => {
vm_block = 7; continue;
}
// C line 41626
31 => {
vm_block = if (JS_IsException(next_method)) != 0 { 30 } else { 29 }; continue;
}
// C line 41625
32 => {
let _ = { let assigned = JS_GetProperty(ctx, enum_obj, (((crate::quickjs_atom::JS_ATOM_next as i32)) as JSAtom)); next_method = assigned; assigned };
vm_block = 31; continue;
}
// C line 41624
33 => {
vm_block = 7; continue;
}
// C line 41623
34 => {
vm_block = if (JS_IsException(enum_obj)) != 0 { 33 } else { 32 }; continue;
}
// C line 41622
35 => {
let _ = { let assigned = JS_GetIterator2(ctx, items, iter); enum_obj = assigned; assigned };
vm_block = 34; continue;
}
// C line 41621
36 => {
vm_block = 7; continue;
}
// C line 41620
37 => {
vm_block = if (JS_IsException(r)) != 0 { 36 } else { 35 }; continue;
}
// C line 41617
38 => {
let _ = { let assigned = JS_CallConstructor(ctx, this_val, (0 as i32), core::ptr::null_mut::<JSValue>()); r = assigned; assigned };
vm_block = 37; continue;
}
// C line 41619
39 => {
let _ = { let assigned = JS_NewArray(ctx); r = assigned; assigned };
vm_block = 37; continue;
}
// C line 41616
40 => {
vm_block = if (JS_IsConstructor(ctx, this_val)) != 0 { 38 } else { 39 }; continue;
}
// C line 41614
41 => {
vm_block = 7; continue;
}
// C line 41613
42 => {
let _ = JS_ThrowTypeError(ctx, c"value is not iterable".as_ptr());
vm_block = 41; continue;
}
// C line 41612
43 => {
vm_block = if ((!((JS_IsFunction(ctx, iter)) != 0) as i32)) != 0 { 42 } else { 40 }; continue;
}
// C line 41663
44 => {
vm_block = if ((((k) < (len)) as i32)) != 0 { 58 } else { 11 }; continue;
}
// C line ?
45 => {
let _ = { let old = k; k = (k).wrapping_add(1); old };
vm_block = 44; continue;
}
// C line 41678
46 => {
vm_block = 7; continue;
}
// C line 41676
47 => {
vm_block = if ((((JS_DefinePropertyValueInt64(ctx, r, k, v, ((((((((1 as i32)).wrapping_shl(((0 as i32)) as u32)) | (((1 as i32)).wrapping_shl(((1 as i32)) as u32)))) | (((1 as i32)).wrapping_shl(((2 as i32)) as u32)))) | (((1 as i32)).wrapping_shl(((14 as i32)) as u32))))) < ((0 as i32))) as i32)) != 0 { 46 } else { 45 }; continue;
}
// C line 41674
48 => {
vm_block = 7; continue;
}
// C line 41673
49 => {
vm_block = if (JS_IsException(v)) != 0 { 48 } else { 47 }; continue;
}
// C line 41672
50 => {
let _ = { let assigned = v2; v = assigned; assigned };
vm_block = 49; continue;
}
// C line 41671
51 => {
let _ = JS_FreeValue(ctx, v);
vm_block = 50; continue;
}
// C line 41670
52 => {
let _ = { let assigned = JS_Call(ctx, mapfn, this_arg, (2 as i32), (args).as_mut_ptr()); v2 = assigned; assigned };
vm_block = 51; continue;
}
// C line 41669
53 => {
let _ = { let assigned = JS_NewInt32(ctx, ((k) as i32)); *((args).as_mut_ptr()).offset(((1 as i32)) as isize) = assigned; assigned };
vm_block = 52; continue;
}
// C line 41668
54 => {
let _ = { let assigned = v; *((args).as_mut_ptr()).offset(((0 as i32)) as isize) = assigned; assigned };
vm_block = 53; continue;
}
// C line 41667
55 => {
vm_block = if (mapping) != 0 { 54 } else { 47 }; continue;
}
// C line 41666
56 => {
vm_block = 7; continue;
}
// C line 41665
57 => {
vm_block = if (JS_IsException(v)) != 0 { 56 } else { 55 }; continue;
}
// C line 41664
58 => {
let _ = { let assigned = JS_GetPropertyInt64(ctx, arrayLike, k); v = assigned; assigned };
vm_block = 57; continue;
}
// C line 41663
59 => {
let _ = { let assigned = (((0 as i32)) as i64); k = assigned; assigned };
vm_block = 44; continue;
}
// C line 41662
60 => {
vm_block = 7; continue;
}
// C line 41661
61 => {
vm_block = if (JS_IsException(r)) != 0 { 60 } else { 59 }; continue;
}
// C line 41660
62 => {
let _ = JS_FreeValue(ctx, v);
vm_block = 61; continue;
}
// C line 41656
63 => {
let _ = { let assigned = JS_CallConstructor(ctx, this_val, (1 as i32), (args).as_mut_ptr()); r = assigned; assigned };
vm_block = 62; continue;
}
// C line 41658
64 => {
let _ = { let assigned = js_array_constructor(ctx, JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }, (1 as i32), (args).as_mut_ptr()); r = assigned; assigned };
vm_block = 62; continue;
}
// C line 41655
65 => {
vm_block = if (JS_IsConstructor(ctx, this_val)) != 0 { 63 } else { 64 }; continue;
}
// C line 41654
66 => {
let _ = { let assigned = v; *((args).as_mut_ptr()).offset(((0 as i32)) as isize) = assigned; assigned };
vm_block = 65; continue;
}
// C line 41653
67 => {
let _ = { let assigned = JS_NewInt64(ctx, len); v = assigned; assigned };
vm_block = 66; continue;
}
// C line 41652
68 => {
vm_block = 7; continue;
}
// C line 41651
69 => {
vm_block = if ((((js_get_length64(ctx, core::ptr::addr_of_mut!(len), arrayLike)) < ((0 as i32))) as i32)) != 0 { 68 } else { 67 }; continue;
}
// C line 41650
70 => {
vm_block = 7; continue;
}
// C line 41649
71 => {
vm_block = if (JS_IsException(arrayLike)) != 0 { 70 } else { 69 }; continue;
}
// C line 41648
72 => {
let _ = { let assigned = JS_ToObject(ctx, items); arrayLike = assigned; assigned };
vm_block = 71; continue;
}
// C line 41611
73 => {
vm_block = if ((((((!((JS_IsUndefined(iter)) != 0) as i32)) != 0) && (((!((JS_IsNull(iter)) != 0) as i32)) != 0)) as i32)) != 0 { 43 } else { 72 }; continue;
}
// C line 41610
74 => {
vm_block = 7; continue;
}
// C line 41609
75 => {
vm_block = if (JS_IsException(iter)) != 0 { 74 } else { 73 }; continue;
}
// C line 41608
76 => {
let _ = { let assigned = JS_GetProperty(ctx, items, (((crate::quickjs_atom::JS_ATOM_Symbol_iterator as i32)) as JSAtom)); iter = assigned; assigned };
vm_block = 75; continue;
}
// C line 41605
77 => {
let _ = { let assigned = *(argv).offset(((2 as i32)) as isize); this_arg = assigned; assigned };
vm_block = 76; continue;
}
// C line 41604
78 => {
vm_block = if ((((argc) > ((2 as i32))) as i32)) != 0 { 77 } else { 76 }; continue;
}
// C line 41603
79 => {
let _ = { let assigned = (1 as i32); mapping = assigned; assigned };
vm_block = 78; continue;
}
// C line 41602
80 => {
vm_block = 7; continue;
}
// C line 41601
81 => {
vm_block = if (check_function(ctx, mapfn)) != 0 { 80 } else { 79 }; continue;
}
// C line 41600
82 => {
vm_block = if ((!((JS_IsUndefined(mapfn)) != 0) as i32)) != 0 { 81 } else { 76 }; continue;
}
// C line 41599
83 => {
let _ = { let assigned = *(argv).offset(((1 as i32)) as isize); mapfn = assigned; assigned };
vm_block = 82; continue;
}
// C line 41598
84 => {
vm_block = if ((((argc) > ((1 as i32))) as i32)) != 0 { 83 } else { 76 }; continue;
}
// C line 41596
85 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; next_method = assigned; assigned };
vm_block = 84; continue;
}
// C line 41595
86 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; enum_obj = assigned; assigned };
vm_block = 85; continue;
}
// C line 41594
87 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; iter = assigned; assigned };
vm_block = 86; continue;
}
// C line 41593
88 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; arrayLike = assigned; assigned };
vm_block = 87; continue;
}
// C line 41592
89 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; r = assigned; assigned };
vm_block = 88; continue;
}
// C line 41591
90 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; this_arg = assigned; assigned };
vm_block = 89; continue;
}
// C line 41590
91 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; mapfn = assigned; assigned };
vm_block = 90; continue;
}
// C line 41589
92 => {
let _ = { let assigned = (0 as i32); mapping = assigned; assigned };
vm_block = 91; continue;
}
// C line 41583
93 => {
items = *(argv).offset(((0 as i32)) as isize);
vm_block = 92; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:41698. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_array_of(mut ctx: *mut JSContext, mut this_val: JSValue, mut argc: i32, mut argv: *mut JSValue) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut obj: JSValue = core::mem::zeroed();
let mut args: [JSValue; 1] = core::mem::zeroed();
let mut i: i32 = core::mem::zeroed();
let mut vm_block: usize = 15;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 41723
1 => {
return obj;
}
// C line 41721
2 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line ? labels: fail
3 => {
let _ = JS_FreeValue(ctx, obj);
vm_block = 2; continue;
}
// C line 41718
4 => {
vm_block = if ((((JS_SetProperty(ctx, obj, (((crate::quickjs_atom::JS_ATOM_length as i32)) as JSAtom), JS_NewUint32(ctx, ((argc) as u32)))) < ((0 as i32))) as i32)) != 0 { 3 } else { 1 }; continue;
}
// C line 41712
5 => {
vm_block = if ((((i) < (argc)) as i32)) != 0 { 8 } else { 4 }; continue;
}
// C line ?
6 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 5; continue;
}
// C line 41715
7 => {
vm_block = 3; continue;
}
// C line 41713
8 => {
vm_block = if ((((JS_CreateDataPropertyUint32(ctx, obj, ((i) as i64), JS_DupValue(ctx, *(argv).offset((i) as isize)), ((1 as i32)).wrapping_shl(((14 as i32)) as u32))) < ((0 as i32))) as i32)) != 0 { 7 } else { 6 }; continue;
}
// C line 41712
9 => {
let _ = { let assigned = (0 as i32); i = assigned; assigned };
vm_block = 5; continue;
}
// C line 41711
10 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 41710
11 => {
vm_block = if (JS_IsException(obj)) != 0 { 10 } else { 9 }; continue;
}
// C line 41706
12 => {
let _ = { let assigned = JS_CallConstructor(ctx, this_val, (1 as i32), (args).as_mut_ptr()); obj = assigned; assigned };
vm_block = 11; continue;
}
// C line 41705
13 => {
let _ = { let assigned = JS_NewInt32(ctx, argc); *((args).as_mut_ptr()).offset(((0 as i32)) as isize) = assigned; assigned };
vm_block = 12; continue;
}
// C line 41708
14 => {
let _ = { let assigned = JS_NewArray(ctx); obj = assigned; assigned };
vm_block = 11; continue;
}
// C line 41704
15 => {
vm_block = if (JS_IsConstructor(ctx, this_val)) != 0 { 13 } else { 14 }; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:41726. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_array_isArray(mut ctx: *mut JSContext, mut this_val: JSValue, mut argc: i32, mut argv: *mut JSValue) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut ret: i32 = core::mem::zeroed();
let mut vm_block: usize = 4;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 41732
1 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 41734
2 => {
return JS_NewBool(ctx, ret);
}
// C line 41731
3 => {
vm_block = if ((((ret) < ((0 as i32))) as i32)) != 0 { 1 } else { 2 }; continue;
}
// C line 41730
4 => {
let _ = { let assigned = JS_IsArray(ctx, *(argv).offset(((0 as i32)) as isize)); ret = assigned; assigned };
vm_block = 3; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:41737. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_get_this(mut ctx: *mut JSContext, mut this_val: JSValue) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut vm_block: usize = 1;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 41740
1 => {
return JS_DupValue(ctx, this_val);
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:41743. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn JS_ArraySpeciesCreate(mut ctx: *mut JSContext, mut obj: JSValue, mut len_val: JSValue) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut ctor: JSValue = core::mem::zeroed();
let mut ret: JSValue = core::mem::zeroed();
let mut species: JSValue = core::mem::zeroed();
let mut res: i32 = core::mem::zeroed();
let mut realm: *mut JSContext = core::mem::zeroed();
let mut vm_block: usize = 29;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 41781
1 => {
return js_array_constructor(ctx, JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }, (1 as i32), core::ptr::addr_of_mut!(len_val));
}
// C line 41785
2 => {
return ret;
}
// C line 41784
3 => {
let _ = JS_FreeValue(ctx, ctor);
vm_block = 2; continue;
}
// C line 41783
4 => {
let _ = { let assigned = JS_CallConstructor(ctx, ctor, (1 as i32), core::ptr::addr_of_mut!(len_val)); ret = assigned; assigned };
vm_block = 3; continue;
}
// C line 41780
5 => {
vm_block = if (JS_IsUndefined(ctor)) != 0 { 1 } else { 4 }; continue;
}
// C line 41778
6 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; ctor = assigned; assigned };
vm_block = 5; continue;
}
// C line 41777
7 => {
vm_block = if (JS_IsNull(ctor)) != 0 { 6 } else { 5 }; continue;
}
// C line 41776
8 => {
let _ = { let assigned = species; ctor = assigned; assigned };
vm_block = 7; continue;
}
// C line 41775
9 => {
return species;
}
// C line 41774
10 => {
vm_block = if (JS_IsException(species)) != 0 { 9 } else { 8 }; continue;
}
// C line 41773
11 => {
let _ = JS_FreeValue(ctx, ctor);
vm_block = 10; continue;
}
// C line 41772
12 => {
let _ = { let assigned = JS_GetProperty(ctx, ctor, (((crate::quickjs_atom::JS_ATOM_Symbol_species as i32)) as JSAtom)); species = assigned; assigned };
vm_block = 11; continue;
}
// C line 41771
13 => {
vm_block = if (JS_IsObject(ctor)) != 0 { 12 } else { 5 }; continue;
}
// C line 41768
14 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; ctor = assigned; assigned };
vm_block = 13; continue;
}
// C line 41767
15 => {
let _ = JS_FreeValue(ctx, ctor);
vm_block = 14; continue;
}
// C line 41765
16 => {
vm_block = if ((((((((realm) != (ctx)) as i32)) != 0) && ((js_same_value(ctx, ctor, (*(realm)).array_ctor)) != 0)) as i32)) != 0 { 15 } else { 13 }; continue;
}
// C line 41763
17 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 41762
18 => {
let _ = JS_FreeValue(ctx, ctor);
vm_block = 17; continue;
}
// C line 41761
19 => {
vm_block = if ((!(!(realm).is_null()) as i32)) != 0 { 18 } else { 16 }; continue;
}
// C line 41760
20 => {
let _ = { let assigned = JS_GetFunctionRealm(ctx, ctor); realm = assigned; assigned };
vm_block = 19; continue;
}
// C line 41758
21 => {
vm_block = if (JS_IsConstructor(ctx, ctor)) != 0 { 20 } else { 13 }; continue;
}
// C line 41757
22 => {
return ctor;
}
// C line 41756
23 => {
vm_block = if (JS_IsException(ctor)) != 0 { 22 } else { 21 }; continue;
}
// C line 41755
24 => {
let _ = { let assigned = JS_GetProperty(ctx, obj, (((crate::quickjs_atom::JS_ATOM_constructor as i32)) as JSAtom)); ctor = assigned; assigned };
vm_block = 23; continue;
}
// C line 41754
25 => {
return js_array_constructor(ctx, JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }, (1 as i32), core::ptr::addr_of_mut!(len_val));
}
// C line 41753
26 => {
vm_block = if ((!((res) != 0) as i32)) != 0 { 25 } else { 24 }; continue;
}
// C line 41752
27 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 41751
28 => {
vm_block = if ((((res) < ((0 as i32))) as i32)) != 0 { 27 } else { 26 }; continue;
}
// C line 41750
29 => {
let _ = { let assigned = JS_IsArray(ctx, obj); res = assigned; assigned };
vm_block = 28; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:41796. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn JS_isConcatSpreadable(mut ctx: *mut JSContext, mut obj: JSValue) -> i32 {
let mut vm_local_storage = Vec::<u64>::new();
let mut val: JSValue = core::mem::zeroed();
let mut vm_block: usize = 8;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 41807
1 => {
return JS_IsArray(ctx, obj);
}
// C line 41806
2 => {
return JS_ToBoolFree(ctx, val);
}
// C line 41805
3 => {
vm_block = if ((!((JS_IsUndefined(val)) != 0) as i32)) != 0 { 2 } else { 1 }; continue;
}
// C line 41804
4 => {
return ((1 as i32)).wrapping_neg();
}
// C line 41803
5 => {
vm_block = if (JS_IsException(val)) != 0 { 4 } else { 3 }; continue;
}
// C line 41802
6 => {
let _ = { let assigned = JS_GetProperty(ctx, obj, (((crate::quickjs_atom::JS_ATOM_Symbol_isConcatSpreadable as i32)) as JSAtom)); val = assigned; assigned };
vm_block = 5; continue;
}
// C line 41801
7 => {
return (0 as i32);
}
// C line 41800
8 => {
vm_block = if ((!((JS_IsObject(obj)) != 0) as i32)) != 0 { 7 } else { 6 }; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:41810. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_array_at(mut ctx: *mut JSContext, mut this_val: JSValue, mut argc: i32, mut argv: *mut JSValue) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut obj: JSValue = core::mem::zeroed();
let mut ret: JSValue = core::mem::zeroed();
let mut len: i64 = core::mem::zeroed();
let mut idx: i64 = core::mem::zeroed();
let mut arrp: *mut JSValue = core::mem::zeroed();
let mut count: u32 = core::mem::zeroed();
let mut present: i32 = core::mem::zeroed();
let mut vm_block: usize = 20;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 41842
1 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line ? labels: exception
2 => {
let _ = JS_FreeValue(ctx, obj);
vm_block = 1; continue;
}
// C line 41839
3 => {
return ret;
}
// C line 41838
4 => {
let _ = JS_FreeValue(ctx, obj);
vm_block = 3; continue;
}
// C line 41828
5 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; ret = assigned; assigned };
vm_block = 4; continue;
}
// C line 41830
6 => {
let _ = { let assigned = JS_DupValue(ctx, *(arrp).offset((idx) as isize)); ret = assigned; assigned };
vm_block = 4; continue;
}
// C line 41836
7 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; ret = assigned; assigned };
vm_block = 4; continue;
}
// C line 41835
8 => {
vm_block = if ((!((present) != 0) as i32)) != 0 { 7 } else { 4 }; continue;
}
// C line 41834
9 => {
vm_block = 2; continue;
}
// C line 41833
10 => {
vm_block = if ((((present) < ((0 as i32))) as i32)) != 0 { 9 } else { 8 }; continue;
}
// C line 41832
11 => {
present = JS_TryGetPropertyInt64(ctx, obj, idx, core::ptr::addr_of_mut!(ret));
vm_block = 10; continue;
}
// C line 41829
12 => {
vm_block = if (((((js_get_fast_array(ctx, obj, core::ptr::addr_of_mut!(arrp), core::ptr::addr_of_mut!(count))) != 0) && (((((idx) < (((count) as i64))) as i32)) != 0)) as i32)) != 0 { 6 } else { 11 }; continue;
}
// C line 41827
13 => {
vm_block = if ((((((((idx) < ((((0 as i32)) as i64))) as i32)) != 0) || (((((idx) >= (len)) as i32)) != 0)) as i32)) != 0 { 5 } else { 12 }; continue;
}
// C line 41826
14 => {
let _ = { let assigned = (len).wrapping_add(idx); idx = assigned; assigned };
vm_block = 13; continue;
}
// C line 41825
15 => {
vm_block = if ((((idx) < ((((0 as i32)) as i64))) as i32)) != 0 { 14 } else { 13 }; continue;
}
// C line 41823
16 => {
vm_block = 2; continue;
}
// C line 41822
17 => {
vm_block = if (JS_ToInt64Sat(ctx, core::ptr::addr_of_mut!(idx), *(argv).offset(((0 as i32)) as isize))) != 0 { 16 } else { 15 }; continue;
}
// C line 41820
18 => {
vm_block = 2; continue;
}
// C line 41819
19 => {
vm_block = if (js_get_length64(ctx, core::ptr::addr_of_mut!(len), obj)) != 0 { 18 } else { 17 }; continue;
}
// C line 41818
20 => {
let _ = { let assigned = JS_ToObject(ctx, this_val); obj = assigned; assigned };
vm_block = 19; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:41845. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_array_with(mut ctx: *mut JSContext, mut this_val: JSValue, mut argc: i32, mut argv: *mut JSValue) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut arr: JSValue = core::mem::zeroed();
let mut obj: JSValue = core::mem::zeroed();
let mut ret: JSValue = core::mem::zeroed();
let mut arrp: *mut JSValue = core::mem::zeroed();
let mut pval: *mut JSValue = core::mem::zeroed();
let mut p: *mut JSObject = core::mem::zeroed();
let mut i: i64 = core::mem::zeroed();
let mut len: i64 = core::mem::zeroed();
let mut idx: i64 = core::mem::zeroed();
let mut count32: u32 = core::mem::zeroed();
let mut vm_block: usize = 42;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 41900
1 => {
return ret;
}
// C line 41899
2 => {
let _ = JS_FreeValue(ctx, obj);
vm_block = 1; continue;
}
// C line ? labels: exception
3 => {
let _ = JS_FreeValue(ctx, arr);
vm_block = 2; continue;
}
// C line 41895
4 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; arr = assigned; assigned };
vm_block = 3; continue;
}
// C line 41894
5 => {
let _ = { let assigned = arr; ret = assigned; assigned };
vm_block = 4; continue;
}
// C line 41881
6 => {
vm_block = if ((((i) < (len)) as i32)) != 0 { 8 } else { 5 }; continue;
}
// C line ?
7 => {
let _ = { let _ = { let old = i; i = (i).wrapping_add(1); old }; { let old = pval; pval = (pval).offset(1); old } };
vm_block = 6; continue;
}
// C line 41882
8 => {
let _ = { let assigned = JS_DupValue(ctx, *(arrp).offset((i) as isize)); *(pval) = assigned; assigned };
vm_block = 7; continue;
}
// C line 41881
9 => {
let _ = { let _ = { let old = i; i = (i).wrapping_add(1); old }; { let old = pval; pval = (pval).offset(1); old } };
vm_block = 6; continue;
}
// C line 41880
10 => {
let _ = { let assigned = JS_DupValue(ctx, *(argv).offset(((1 as i32)) as isize)); *(pval) = assigned; assigned };
vm_block = 9; continue;
}
// C line 41878
11 => {
vm_block = if ((((i) < (idx)) as i32)) != 0 { 13 } else { 10 }; continue;
}
// C line ?
12 => {
let _ = { let _ = { let old = i; i = (i).wrapping_add(1); old }; { let old = pval; pval = (pval).offset(1); old } };
vm_block = 11; continue;
}
// C line 41879
13 => {
let _ = { let assigned = JS_DupValue(ctx, *(arrp).offset((i) as isize)); *(pval) = assigned; assigned };
vm_block = 12; continue;
}
// C line 41888
14 => {
vm_block = if ((((i) < (len)) as i32)) != 0 { 17 } else { 5 }; continue;
}
// C line ?
15 => {
let _ = { let _ = { let old = i; i = (i).wrapping_add(1); old }; { let old = pval; pval = (pval).offset(1); old } };
vm_block = 14; continue;
}
// C line 41890
16 => {
vm_block = 3; continue;
}
// C line 41889
17 => {
vm_block = if ((((((1 as i32)).wrapping_neg()) == (JS_TryGetPropertyInt64(ctx, obj, i, pval))) as i32)) != 0 { 16 } else { 15 }; continue;
}
// C line 41888
18 => {
let _ = { let _ = { let old = i; i = (i).wrapping_add(1); old }; { let old = pval; pval = (pval).offset(1); old } };
vm_block = 14; continue;
}
// C line 41887
19 => {
let _ = { let assigned = JS_DupValue(ctx, *(argv).offset(((1 as i32)) as isize)); *(pval) = assigned; assigned };
vm_block = 18; continue;
}
// C line 41884
20 => {
vm_block = if ((((i) < (idx)) as i32)) != 0 { 23 } else { 19 }; continue;
}
// C line ?
21 => {
let _ = { let _ = { let old = i; i = (i).wrapping_add(1); old }; { let old = pval; pval = (pval).offset(1); old } };
vm_block = 20; continue;
}
// C line 41886
22 => {
vm_block = 3; continue;
}
// C line 41885
23 => {
vm_block = if ((((((1 as i32)).wrapping_neg()) == (JS_TryGetPropertyInt64(ctx, obj, i, pval))) as i32)) != 0 { 22 } else { 21 }; continue;
}
// C line 41877
24 => {
vm_block = if (((((js_get_fast_array(ctx, obj, core::ptr::addr_of_mut!(arrp), core::ptr::addr_of_mut!(count32))) != 0) && (((((((count32) as i64)) == (len)) as i32)) != 0)) as i32)) != 0 { 11 } else { 20 }; continue;
}
// C line 41876
25 => {
let _ = { let assigned = ((((*(p)).u).array).u).values; pval = assigned; assigned };
vm_block = 24; continue;
}
// C line 41875
26 => {
let _ = { let assigned = (((0 as i32)) as i64); i = assigned; assigned };
vm_block = 25; continue;
}
// C line 41874
27 => {
let _ = { let assigned = ((((arr).u).ptr) as *mut JSObject); p = assigned; assigned };
vm_block = 26; continue;
}
// C line 41872
28 => {
vm_block = 3; continue;
}
// C line 41871
29 => {
vm_block = if (JS_IsException(arr)) != 0 { 28 } else { 27 }; continue;
}
// C line 41870
30 => {
let _ = { let assigned = js_allocate_fast_array(ctx, len); arr = assigned; assigned };
vm_block = 29; continue;
}
// C line 41867
31 => {
vm_block = 3; continue;
}
// C line 41866
32 => {
let _ = JS_ThrowRangeError(ctx, format_args!("invalid array index: {}", idx));
vm_block = 31; continue;
}
// C line 41865
33 => {
vm_block = if ((((((((idx) < ((((0 as i32)) as i64))) as i32)) != 0) || (((((idx) >= (len)) as i32)) != 0)) as i32)) != 0 { 32 } else { 30 }; continue;
}
// C line 41863
34 => {
let _ = { let assigned = (len).wrapping_add(idx); idx = assigned; assigned };
vm_block = 33; continue;
}
// C line 41862
35 => {
vm_block = if ((((idx) < ((((0 as i32)) as i64))) as i32)) != 0 { 34 } else { 33 }; continue;
}
// C line 41860
36 => {
vm_block = 3; continue;
}
// C line 41859
37 => {
vm_block = if (JS_ToInt64Sat(ctx, core::ptr::addr_of_mut!(idx), *(argv).offset(((0 as i32)) as isize))) != 0 { 36 } else { 35 }; continue;
}
// C line 41857
38 => {
vm_block = 3; continue;
}
// C line 41856
39 => {
vm_block = if (js_get_length64(ctx, core::ptr::addr_of_mut!(len), obj)) != 0 { 38 } else { 37 }; continue;
}
// C line 41855
40 => {
let _ = { let assigned = JS_ToObject(ctx, this_val); obj = assigned; assigned };
vm_block = 39; continue;
}
// C line 41854
41 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; arr = assigned; assigned };
vm_block = 40; continue;
}
// C line 41853
42 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) }; ret = assigned; assigned };
vm_block = 41; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:41903. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_array_concat(mut ctx: *mut JSContext, mut this_val: JSValue, mut argc: i32, mut argv: *mut JSValue) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut obj: JSValue = core::mem::zeroed();
let mut arr: JSValue = core::mem::zeroed();
let mut val: JSValue = core::mem::zeroed();
let mut e: JSValue = core::mem::zeroed();
let mut len: i64 = core::mem::zeroed();
let mut k: i64 = core::mem::zeroed();
let mut n: i64 = core::mem::zeroed();
let mut i: i32 = core::mem::zeroed();
let mut res: i32 = core::mem::zeroed();
let mut vm_block: usize = 45;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 41966
1 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 41965
2 => {
let _ = JS_FreeValue(ctx, obj);
vm_block = 1; continue;
}
// C line ? labels: exception
3 => {
let _ = JS_FreeValue(ctx, arr);
vm_block = 2; continue;
}
// C line 41961
4 => {
return arr;
}
// C line 41960
5 => {
let _ = JS_FreeValue(ctx, obj);
vm_block = 4; continue;
}
// C line 41958
6 => {
vm_block = 3; continue;
}
// C line 41957
7 => {
vm_block = if ((((JS_SetProperty(ctx, arr, (((crate::quickjs_atom::JS_ATOM_length as i32)) as JSAtom), JS_NewInt64(ctx, n))) < ((0 as i32))) as i32)) != 0 { 6 } else { 5 }; continue;
}
// C line 41920
8 => {
vm_block = if ((((i) < (argc)) as i32)) != 0 { 36 } else { 7 }; continue;
}
// C line ?
9 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 8; continue;
}
// C line 41936
10 => {
vm_block = if ((((k) < (len)) as i32)) != 0 { 17 } else { 9 }; continue;
}
// C line ?
11 => {
let _ = { let _ = { let old = k; k = (k).wrapping_add(1); old }; { let old = n; n = (n).wrapping_add(1); old } };
vm_block = 10; continue;
}
// C line 41943
12 => {
vm_block = 3; continue;
}
// C line 41941
13 => {
vm_block = if ((((JS_DefinePropertyValueInt64(ctx, arr, n, val, ((((((((1 as i32)).wrapping_shl(((0 as i32)) as u32)) | (((1 as i32)).wrapping_shl(((1 as i32)) as u32)))) | (((1 as i32)).wrapping_shl(((2 as i32)) as u32)))) | (((1 as i32)).wrapping_shl(((14 as i32)) as u32))))) < ((0 as i32))) as i32)) != 0 { 12 } else { 11 }; continue;
}
// C line 41940
14 => {
vm_block = if (res) != 0 { 13 } else { 11 }; continue;
}
// C line 41939
15 => {
vm_block = 3; continue;
}
// C line 41938
16 => {
vm_block = if ((((res) < ((0 as i32))) as i32)) != 0 { 15 } else { 14 }; continue;
}
// C line 41937
17 => {
let _ = { let assigned = JS_TryGetPropertyInt64(ctx, e, k, core::ptr::addr_of_mut!(val)); res = assigned; assigned };
vm_block = 16; continue;
}
// C line 41936
18 => {
let _ = { let assigned = (((0 as i32)) as i64); k = assigned; assigned };
vm_block = 10; continue;
}
// C line 41934
19 => {
vm_block = 3; continue;
}
// C line 41933
20 => {
let _ = JS_ThrowTypeError(ctx, c"Array loo long".as_ptr());
vm_block = 19; continue;
}
// C line 41932
21 => {
vm_block = if (((((n).wrapping_add(len)) > ((((((1 as i32)) as i64)).wrapping_shl(((53 as i32)) as u32)).wrapping_sub((((1 as i32)) as i64)))) as i32)) != 0 { 20 } else { 18 }; continue;
}
// C line 41931
22 => {
vm_block = 3; continue;
}
// C line 41930
23 => {
vm_block = if (js_get_length64(ctx, core::ptr::addr_of_mut!(len), e)) != 0 { 22 } else { 21 }; continue;
}
// C line 41954
24 => {
let _ = { let old = n; n = (n).wrapping_add(1); old };
vm_block = 9; continue;
}
// C line 41953
25 => {
vm_block = 3; continue;
}
// C line 41951
26 => {
vm_block = if ((((JS_DefinePropertyValueInt64(ctx, arr, n, JS_DupValue(ctx, e), ((((((((1 as i32)).wrapping_shl(((0 as i32)) as u32)) | (((1 as i32)).wrapping_shl(((1 as i32)) as u32)))) | (((1 as i32)).wrapping_shl(((2 as i32)) as u32)))) | (((1 as i32)).wrapping_shl(((14 as i32)) as u32))))) < ((0 as i32))) as i32)) != 0 { 25 } else { 24 }; continue;
}
// C line 41949
27 => {
vm_block = 3; continue;
}
// C line 41948
28 => {
let _ = JS_ThrowTypeError(ctx, c"Array loo long".as_ptr());
vm_block = 27; continue;
}
// C line 41947
29 => {
vm_block = if ((((n) >= ((((((1 as i32)) as i64)).wrapping_shl(((53 as i32)) as u32)).wrapping_sub((((1 as i32)) as i64)))) as i32)) != 0 { 28 } else { 26 }; continue;
}
// C line 41929
30 => {
vm_block = if (res) != 0 { 23 } else { 29 }; continue;
}
// C line 41928
31 => {
vm_block = 3; continue;
}
// C line 41927
32 => {
vm_block = if ((((res) < ((0 as i32))) as i32)) != 0 { 31 } else { 30 }; continue;
}
// C line 41926
33 => {
let _ = { let assigned = JS_isConcatSpreadable(ctx, e); res = assigned; assigned };
vm_block = 32; continue;
}
// C line 41922
34 => {
let _ = { let assigned = obj; e = assigned; assigned };
vm_block = 33; continue;
}
// C line 41924
35 => {
let _ = { let assigned = *(argv).offset((i) as isize); e = assigned; assigned };
vm_block = 33; continue;
}
// C line 41921
36 => {
vm_block = if ((((i) < ((0 as i32))) as i32)) != 0 { 34 } else { 35 }; continue;
}
// C line 41920
37 => {
let _ = { let assigned = ((1 as i32)).wrapping_neg(); i = assigned; assigned };
vm_block = 8; continue;
}
// C line 41919
38 => {
let _ = { let assigned = (((0 as i32)) as i64); n = assigned; assigned };
vm_block = 37; continue;
}
// C line 41918
39 => {
vm_block = 3; continue;
}
// C line 41917
40 => {
vm_block = if (JS_IsException(arr)) != 0 { 39 } else { 38 }; continue;
}
// C line 41916
41 => {
let _ = { let assigned = JS_ArraySpeciesCreate(ctx, obj, JS_NewInt32(ctx, (0 as i32))); arr = assigned; assigned };
vm_block = 40; continue;
}
// C line 41914
42 => {
vm_block = 3; continue;
}
// C line 41913
43 => {
vm_block = if (JS_IsException(obj)) != 0 { 42 } else { 41 }; continue;
}
// C line 41912
44 => {
let _ = { let assigned = JS_ToObject(ctx, this_val); obj = assigned; assigned };
vm_block = 43; continue;
}
// C line 41911
45 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; arr = assigned; assigned };
vm_block = 44; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:41980. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_array_every(mut ctx: *mut JSContext, mut this_val: JSValue, mut argc: i32, mut argv: *mut JSValue, mut special: i32) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut obj: JSValue = core::mem::zeroed();
let mut val: JSValue = core::mem::zeroed();
let mut index_val: JSValue = core::mem::zeroed();
let mut res: JSValue = core::mem::zeroed();
let mut ret: JSValue = core::mem::zeroed();
let mut args: [JSValue; 3] = core::mem::zeroed();
let mut func: JSValue = core::mem::zeroed();
let mut this_arg: JSValue = core::mem::zeroed();
let mut len: i64 = core::mem::zeroed();
let mut k: i64 = core::mem::zeroed();
let mut n: i64 = core::mem::zeroed();
let mut present: i32 = core::mem::zeroed();
let mut arr: JSValue = core::mem::zeroed();
let mut vm_block: usize = 105;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 42131
1 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 42130
2 => {
let _ = JS_FreeValue(ctx, obj);
vm_block = 1; continue;
}
// C line 42129
3 => {
let _ = JS_FreeValue(ctx, val);
vm_block = 2; continue;
}
// C line ? labels: exception
4 => {
let _ = JS_FreeValue(ctx, ret);
vm_block = 3; continue;
}
// C line 42125
5 => {
return ret;
}
// C line 42124
6 => {
let _ = JS_FreeValue(ctx, obj);
vm_block = 5; continue;
}
// C line 42123
7 => {
let _ = JS_FreeValue(ctx, val);
vm_block = 6; continue;
}
// C line 42121
8 => {
let _ = { let assigned = arr; ret = assigned; assigned };
vm_block = 7; continue;
}
// C line 42120
9 => {
let _ = JS_FreeValue(ctx, ret);
vm_block = 8; continue;
}
// C line 42118
10 => {
vm_block = 4; continue;
}
// C line 42117
11 => {
let _ = JS_FreeValue(ctx, arr);
vm_block = 10; continue;
}
// C line 42116
12 => {
vm_block = if (check_exception_free(ctx, res)) != 0 { 11 } else { 9 }; continue;
}
// C line 42115
13 => {
let _ = { let assigned = JS_Invoke(ctx, arr, (((crate::quickjs_atom::JS_ATOM_set as i32)) as JSAtom), (1 as i32), (args).as_mut_ptr()); res = assigned; assigned };
vm_block = 12; continue;
}
// C line 42114
14 => {
let _ = { let assigned = ret; *((args).as_mut_ptr()).offset(((0 as i32)) as isize) = assigned; assigned };
vm_block = 13; continue;
}
// C line 42113
15 => {
vm_block = 4; continue;
}
// C line 42112
16 => {
vm_block = if (JS_IsException(arr)) != 0 { 15 } else { 14 }; continue;
}
// C line 42111
17 => {
let _ = { let assigned = js_typed_array___speciesCreate(ctx, JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }, (2 as i32), (args).as_mut_ptr()); arr = assigned; assigned };
vm_block = 16; continue;
}
// C line 42110
18 => {
let _ = { let assigned = JS_NewInt32(ctx, ((n) as i32)); *((args).as_mut_ptr()).offset(((1 as i32)) as isize) = assigned; assigned };
vm_block = 17; continue;
}
// C line 42109
19 => {
let _ = { let assigned = obj; *((args).as_mut_ptr()).offset(((0 as i32)) as isize) = assigned; assigned };
vm_block = 18; continue;
}
// C line 42107 labels: done
20 => {
vm_block = if ((((special) == ((((4 as i32)) | ((8 as i32))))) as i32)) != 0 { 19 } else { 7 }; continue;
}
// C line 42044
21 => {
vm_block = if ((((k) < (len)) as i32)) != 0 { 64 } else { 20 }; continue;
}
// C line ?
22 => {
let _ = { let old = k; k = (k).wrapping_add(1); old };
vm_block = 21; continue;
}
// C line 42103
23 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; val = assigned; assigned };
vm_block = 22; continue;
}
// C line 42102
24 => {
let _ = JS_FreeValue(ctx, val);
vm_block = 23; continue;
}
// C line 42100
25 => {
vm_block = 24; continue;
}
// C line ?
26 => {
let _ = JS_FreeValue(ctx, res);
vm_block = 25; continue;
}
// C line 42097
27 => {
vm_block = 24; continue;
}
// C line 42095
28 => {
vm_block = 4; continue;
}
// C line 42093
29 => {
vm_block = if ((((JS_DefinePropertyValueInt64(ctx, ret, { let old = n; n = (n).wrapping_add(1); old }, JS_DupValue(ctx, val), ((((((((1 as i32)).wrapping_shl(((0 as i32)) as u32)) | (((1 as i32)).wrapping_shl(((1 as i32)) as u32)))) | (((1 as i32)).wrapping_shl(((2 as i32)) as u32)))) | (((1 as i32)).wrapping_shl(((14 as i32)) as u32))))) < ((0 as i32))) as i32)) != 0 { 28 } else { 27 }; continue;
}
// C line 42092
30 => {
vm_block = if (JS_ToBoolFree(ctx, res)) != 0 { 29 } else { 27 }; continue;
}
// C line 42089
31 => {
vm_block = 24; continue;
}
// C line 42088
32 => {
vm_block = 4; continue;
}
// C line 42087
33 => {
vm_block = if ((((JS_SetPropertyValue(ctx, ret, JS_NewInt32(ctx, ((k) as i32)), res, ((1 as i32)).wrapping_shl(((14 as i32)) as u32))) < ((0 as i32))) as i32)) != 0 { 32 } else { 31 }; continue;
}
// C line 42085
34 => {
vm_block = 24; continue;
}
// C line 42084
35 => {
vm_block = 4; continue;
}
// C line 42082
36 => {
vm_block = if ((((JS_DefinePropertyValueInt64(ctx, ret, k, res, ((((((((1 as i32)).wrapping_shl(((0 as i32)) as u32)) | (((1 as i32)).wrapping_shl(((1 as i32)) as u32)))) | (((1 as i32)).wrapping_shl(((2 as i32)) as u32)))) | (((1 as i32)).wrapping_shl(((14 as i32)) as u32))))) < ((0 as i32))) as i32)) != 0 { 35 } else { 34 }; continue;
}
// C line 42080
37 => {
vm_block = 24; continue;
}
// C line 42078
38 => {
vm_block = 20; continue;
}
// C line 42077
39 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((1 as i32)) as u32)) as u64) }, tag: (((JS_TAG_BOOL as i32)) as i64) }; ret = assigned; assigned };
vm_block = 38; continue;
}
// C line 42076
40 => {
vm_block = if (JS_ToBoolFree(ctx, res)) != 0 { 39 } else { 37 }; continue;
}
// C line 42073
41 => {
vm_block = 24; continue;
}
// C line 42071
42 => {
vm_block = 20; continue;
}
// C line 42070
43 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_BOOL as i32)) as i64) }; ret = assigned; assigned };
vm_block = 42; continue;
}
// C line 42069
44 => {
vm_block = if ((!((JS_ToBoolFree(ctx, res)) != 0) as i32)) != 0 { 43 } else { 41 }; continue;
}
// C line 42066
45 => {
vm_block = match special { x if x == (((4 as i32)) | ((8 as i32))) => 30, x if x == (4 as i32) => 30, x if x == (((3 as i32)) | ((8 as i32))) => 33, x if x == (3 as i32) => 36, x if x == (((1 as i32)) | ((8 as i32))) => 40, x if x == (1 as i32) => 40, x if x == (((0 as i32)) | ((8 as i32))) => 44, x if x == (0 as i32) => 44, _ => 26, }; continue;
}
// C line 42065
46 => {
vm_block = 4; continue;
}
// C line 42064
47 => {
vm_block = if (JS_IsException(res)) != 0 { 46 } else { 45 }; continue;
}
// C line 42063
48 => {
let _ = JS_FreeValue(ctx, index_val);
vm_block = 47; continue;
}
// C line 42062
49 => {
let _ = { let assigned = JS_Call(ctx, func, this_arg, (3 as i32), (args).as_mut_ptr()); res = assigned; assigned };
vm_block = 48; continue;
}
// C line 42061
50 => {
let _ = { let assigned = obj; *((args).as_mut_ptr()).offset(((2 as i32)) as isize) = assigned; assigned };
vm_block = 49; continue;
}
// C line 42060
51 => {
let _ = { let assigned = index_val; *((args).as_mut_ptr()).offset(((1 as i32)) as isize) = assigned; assigned };
vm_block = 50; continue;
}
// C line 42059
52 => {
let _ = { let assigned = val; *((args).as_mut_ptr()).offset(((0 as i32)) as isize) = assigned; assigned };
vm_block = 51; continue;
}
// C line 42058
53 => {
vm_block = 4; continue;
}
// C line 42057
54 => {
vm_block = if (JS_IsException(index_val)) != 0 { 53 } else { 52 }; continue;
}
// C line 42056
55 => {
let _ = { let assigned = JS_NewInt64(ctx, k); index_val = assigned; assigned };
vm_block = 54; continue;
}
// C line 42055
56 => {
vm_block = if (present) != 0 { 55 } else { 22 }; continue;
}
// C line 42049
57 => {
let _ = { let assigned = (1 as i32); present = assigned; assigned };
vm_block = 56; continue;
}
// C line 42048
58 => {
vm_block = 4; continue;
}
// C line 42047
59 => {
vm_block = if (JS_IsException(val)) != 0 { 58 } else { 57 }; continue;
}
// C line 42046
60 => {
let _ = { let assigned = JS_GetPropertyInt64(ctx, obj, k); val = assigned; assigned };
vm_block = 59; continue;
}
// C line 42053
61 => {
vm_block = 4; continue;
}
// C line 42052
62 => {
vm_block = if ((((present) < ((0 as i32))) as i32)) != 0 { 61 } else { 56 }; continue;
}
// C line 42051
63 => {
let _ = { let assigned = JS_TryGetPropertyInt64(ctx, obj, k, core::ptr::addr_of_mut!(val)); present = assigned; assigned };
vm_block = 62; continue;
}
// C line 42045
64 => {
vm_block = if (((special) & ((8 as i32)))) != 0 { 60 } else { 63 }; continue;
}
// C line 42044
65 => {
let _ = { let assigned = (((0 as i32)) as i64); k = assigned; assigned };
vm_block = 21; continue;
}
// C line 42042
66 => {
let _ = { let assigned = (((0 as i32)) as i64); n = assigned; assigned };
vm_block = 65; continue;
}
// C line 42040
67 => {
vm_block = 66; continue;
}
// C line 42039
68 => {
vm_block = 4; continue;
}
// C line 42038
69 => {
vm_block = if (JS_IsException(ret)) != 0 { 68 } else { 67 }; continue;
}
// C line 42037
70 => {
let _ = { let assigned = JS_NewArray(ctx); ret = assigned; assigned };
vm_block = 69; continue;
}
// C line 42035
71 => {
vm_block = 66; continue;
}
// C line 42034
72 => {
vm_block = 4; continue;
}
// C line 42033
73 => {
vm_block = if (JS_IsException(ret)) != 0 { 72 } else { 71 }; continue;
}
// C line 42032
74 => {
let _ = { let assigned = js_typed_array___speciesCreate(ctx, JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }, (2 as i32), (args).as_mut_ptr()); ret = assigned; assigned };
vm_block = 73; continue;
}
// C line 42031
75 => {
let _ = { let assigned = JS_NewInt32(ctx, ((len) as i32)); *((args).as_mut_ptr()).offset(((1 as i32)) as isize) = assigned; assigned };
vm_block = 74; continue;
}
// C line 42030
76 => {
let _ = { let assigned = obj; *((args).as_mut_ptr()).offset(((0 as i32)) as isize) = assigned; assigned };
vm_block = 75; continue;
}
// C line 42028
77 => {
vm_block = 66; continue;
}
// C line 42027
78 => {
vm_block = 4; continue;
}
// C line 42026
79 => {
vm_block = if (JS_IsException(ret)) != 0 { 78 } else { 77 }; continue;
}
// C line 42025
80 => {
let _ = { let assigned = JS_ArraySpeciesCreate(ctx, obj, JS_NewInt32(ctx, (0 as i32))); ret = assigned; assigned };
vm_block = 79; continue;
}
// C line 42023
81 => {
vm_block = 66; continue;
}
// C line 42022
82 => {
vm_block = 4; continue;
}
// C line 42021
83 => {
vm_block = if (JS_IsException(ret)) != 0 { 82 } else { 81 }; continue;
}
// C line 42020
84 => {
let _ = { let assigned = JS_ArraySpeciesCreate(ctx, obj, JS_NewInt64(ctx, len)); ret = assigned; assigned };
vm_block = 83; continue;
}
// C line 42017
85 => {
vm_block = 66; continue;
}
// C line 42016
86 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_BOOL as i32)) as i64) }; ret = assigned; assigned };
vm_block = 85; continue;
}
// C line 42013
87 => {
vm_block = 66; continue;
}
// C line 42012
88 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((1 as i32)) as u32)) as u64) }, tag: (((JS_TAG_BOOL as i32)) as i64) }; ret = assigned; assigned };
vm_block = 87; continue;
}
// C line 42009
89 => {
vm_block = match special { x if x == (((4 as i32)) | ((8 as i32))) => 70, x if x == (((3 as i32)) | ((8 as i32))) => 76, x if x == (4 as i32) => 80, x if x == (3 as i32) => 84, x if x == (((1 as i32)) | ((8 as i32))) => 86, x if x == (1 as i32) => 86, x if x == (((0 as i32)) | ((8 as i32))) => 88, x if x == (0 as i32) => 88, _ => 66, }; continue;
}
// C line 42007
90 => {
vm_block = 4; continue;
}
// C line 42006
91 => {
vm_block = if (check_function(ctx, func)) != 0 { 90 } else { 89 }; continue;
}
// C line 42004
92 => {
let _ = { let assigned = *(argv).offset(((1 as i32)) as isize); this_arg = assigned; assigned };
vm_block = 91; continue;
}
// C line 42003
93 => {
vm_block = if ((((argc) > ((1 as i32))) as i32)) != 0 { 92 } else { 91 }; continue;
}
// C line 42002
94 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; this_arg = assigned; assigned };
vm_block = 93; continue;
}
// C line 42001
95 => {
let _ = { let assigned = *(argv).offset(((0 as i32)) as isize); func = assigned; assigned };
vm_block = 94; continue;
}
// C line 41995
96 => {
vm_block = 4; continue;
}
// C line 41994
97 => {
vm_block = if ((((len) < ((((0 as i32)) as i64))) as i32)) != 0 { 96 } else { 95 }; continue;
}
// C line 41993
98 => {
let _ = { let assigned = ((js_typed_array_get_length_unsafe(ctx, obj)) as i64); len = assigned; assigned };
vm_block = 97; continue;
}
// C line 41992
99 => {
let _ = { let assigned = JS_DupValue(ctx, this_val); obj = assigned; assigned };
vm_block = 98; continue;
}
// C line 41999
100 => {
vm_block = 4; continue;
}
// C line 41998
101 => {
vm_block = if (js_get_length64(ctx, core::ptr::addr_of_mut!(len), obj)) != 0 { 100 } else { 95 }; continue;
}
// C line 41997
102 => {
let _ = { let assigned = JS_ToObject(ctx, this_val); obj = assigned; assigned };
vm_block = 101; continue;
}
// C line 41991
103 => {
vm_block = if (((special) & ((8 as i32)))) != 0 { 99 } else { 102 }; continue;
}
// C line 41990
104 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; val = assigned; assigned };
vm_block = 103; continue;
}
// C line 41989
105 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; ret = assigned; assigned };
vm_block = 104; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:42137. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_array_reduce(mut ctx: *mut JSContext, mut this_val: JSValue, mut argc: i32, mut argv: *mut JSValue, mut special: i32) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut obj: JSValue = core::mem::zeroed();
let mut val: JSValue = core::mem::zeroed();
let mut index_val: JSValue = core::mem::zeroed();
let mut acc: JSValue = core::mem::zeroed();
let mut acc1: JSValue = core::mem::zeroed();
let mut args: [JSValue; 4] = core::mem::zeroed();
let mut func: JSValue = core::mem::zeroed();
let mut len: i64 = core::mem::zeroed();
let mut k: i64 = core::mem::zeroed();
let mut k1: i64 = core::mem::zeroed();
let mut present: i32 = core::mem::zeroed();
let mut vm_block: usize = 65;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 42225
1 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 42224
2 => {
let _ = JS_FreeValue(ctx, obj);
vm_block = 1; continue;
}
// C line 42223
3 => {
let _ = JS_FreeValue(ctx, val);
vm_block = 2; continue;
}
// C line ? labels: exception
4 => {
let _ = JS_FreeValue(ctx, acc);
vm_block = 3; continue;
}
// C line 42219
5 => {
return acc;
}
// C line 42218
6 => {
let _ = JS_FreeValue(ctx, obj);
vm_block = 5; continue;
}
// C line 42188
7 => {
vm_block = if ((((k) < (len)) as i32)) != 0 { 33 } else { 6 }; continue;
}
// C line ?
8 => {
let _ = { let old = k; k = (k).wrapping_add(1); old };
vm_block = 7; continue;
}
// C line 42215
9 => {
let _ = { let assigned = acc1; acc = assigned; assigned };
vm_block = 8; continue;
}
// C line 42214
10 => {
let _ = JS_FreeValue(ctx, acc);
vm_block = 9; continue;
}
// C line 42213
11 => {
vm_block = 4; continue;
}
// C line 42212
12 => {
vm_block = if (JS_IsException(acc1)) != 0 { 11 } else { 10 }; continue;
}
// C line 42211
13 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; val = assigned; assigned };
vm_block = 12; continue;
}
// C line 42210
14 => {
let _ = JS_FreeValue(ctx, val);
vm_block = 13; continue;
}
// C line 42209
15 => {
let _ = JS_FreeValue(ctx, index_val);
vm_block = 14; continue;
}
// C line 42208
16 => {
let _ = { let assigned = JS_Call(ctx, func, JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }, (4 as i32), (args).as_mut_ptr()); acc1 = assigned; assigned };
vm_block = 15; continue;
}
// C line 42207
17 => {
let _ = { let assigned = obj; *((args).as_mut_ptr()).offset(((3 as i32)) as isize) = assigned; assigned };
vm_block = 16; continue;
}
// C line 42206
18 => {
let _ = { let assigned = index_val; *((args).as_mut_ptr()).offset(((2 as i32)) as isize) = assigned; assigned };
vm_block = 17; continue;
}
// C line 42205
19 => {
let _ = { let assigned = val; *((args).as_mut_ptr()).offset(((1 as i32)) as isize) = assigned; assigned };
vm_block = 18; continue;
}
// C line 42204
20 => {
let _ = { let assigned = acc; *((args).as_mut_ptr()).offset(((0 as i32)) as isize) = assigned; assigned };
vm_block = 19; continue;
}
// C line 42203
21 => {
vm_block = 4; continue;
}
// C line 42202
22 => {
vm_block = if (JS_IsException(index_val)) != 0 { 21 } else { 20 }; continue;
}
// C line 42201
23 => {
let _ = { let assigned = JS_NewInt64(ctx, k1); index_val = assigned; assigned };
vm_block = 22; continue;
}
// C line 42200
24 => {
vm_block = if (present) != 0 { 23 } else { 8 }; continue;
}
// C line 42194
25 => {
let _ = { let assigned = (1 as i32); present = assigned; assigned };
vm_block = 24; continue;
}
// C line 42193
26 => {
vm_block = 4; continue;
}
// C line 42192
27 => {
vm_block = if (JS_IsException(val)) != 0 { 26 } else { 25 }; continue;
}
// C line 42191
28 => {
let _ = { let assigned = JS_GetPropertyInt64(ctx, obj, k1); val = assigned; assigned };
vm_block = 27; continue;
}
// C line 42198
29 => {
vm_block = 4; continue;
}
// C line 42197
30 => {
vm_block = if ((((present) < ((0 as i32))) as i32)) != 0 { 29 } else { 24 }; continue;
}
// C line 42196
31 => {
let _ = { let assigned = JS_TryGetPropertyInt64(ctx, obj, k1, core::ptr::addr_of_mut!(val)); present = assigned; assigned };
vm_block = 30; continue;
}
// C line 42190
32 => {
vm_block = if (((special) & ((8 as i32)))) != 0 { 28 } else { 31 }; continue;
}
// C line 42189
33 => {
let _ = { let assigned = if (((special) & ((1 as i32)))) != 0 { ((len).wrapping_sub(k)).wrapping_sub((((1 as i32)) as i64)) } else { k }; k1 = assigned; assigned };
vm_block = 32; continue;
}
// C line 42165
34 => {
let _ = { let assigned = JS_DupValue(ctx, *(argv).offset(((1 as i32)) as isize)); acc = assigned; assigned };
vm_block = 7; continue;
}
// C line 42167
35 => {
vm_block = 50; continue;
}
// C line 42178
36 => {
vm_block = 7; continue;
}
// C line 42177
37 => {
vm_block = 4; continue;
}
// C line 42176
38 => {
vm_block = if (JS_IsException(acc)) != 0 { 37 } else { 36 }; continue;
}
// C line 42175
39 => {
let _ = { let assigned = JS_GetPropertyInt64(ctx, obj, k1); acc = assigned; assigned };
vm_block = 38; continue;
}
// C line 42184
40 => {
vm_block = 7; continue;
}
// C line 42183
41 => {
vm_block = if (present) != 0 { 40 } else { 35 }; continue;
}
// C line 42182
42 => {
vm_block = 4; continue;
}
// C line 42181
43 => {
vm_block = if ((((present) < ((0 as i32))) as i32)) != 0 { 42 } else { 41 }; continue;
}
// C line 42180
44 => {
let _ = { let assigned = JS_TryGetPropertyInt64(ctx, obj, k1, core::ptr::addr_of_mut!(acc)); present = assigned; assigned };
vm_block = 43; continue;
}
// C line 42174
45 => {
vm_block = if (((special) & ((8 as i32)))) != 0 { 39 } else { 44 }; continue;
}
// C line 42173
46 => {
let _ = { let old = k; k = (k).wrapping_add(1); old };
vm_block = 45; continue;
}
// C line 42172
47 => {
let _ = { let assigned = if (((special) & ((1 as i32)))) != 0 { ((len).wrapping_sub(k)).wrapping_sub((((1 as i32)) as i64)) } else { k }; k1 = assigned; assigned };
vm_block = 46; continue;
}
// C line 42170
48 => {
vm_block = 4; continue;
}
// C line 42169
49 => {
let _ = JS_ThrowTypeError(ctx, c"empty array".as_ptr());
vm_block = 48; continue;
}
// C line 42168
50 => {
vm_block = if ((((k) >= (len)) as i32)) != 0 { 49 } else { 47 }; continue;
}
// C line 42164
51 => {
vm_block = if ((((argc) > ((1 as i32))) as i32)) != 0 { 34 } else { 35 }; continue;
}
// C line 42163
52 => {
let _ = { let assigned = (((0 as i32)) as i64); k = assigned; assigned };
vm_block = 51; continue;
}
// C line 42161
53 => {
vm_block = 4; continue;
}
// C line 42160
54 => {
vm_block = if (check_function(ctx, func)) != 0 { 53 } else { 52 }; continue;
}
// C line 42158
55 => {
let _ = { let assigned = *(argv).offset(((0 as i32)) as isize); func = assigned; assigned };
vm_block = 54; continue;
}
// C line 42152
56 => {
vm_block = 4; continue;
}
// C line 42151
57 => {
vm_block = if ((((len) < ((((0 as i32)) as i64))) as i32)) != 0 { 56 } else { 55 }; continue;
}
// C line 42150
58 => {
let _ = { let assigned = ((js_typed_array_get_length_unsafe(ctx, obj)) as i64); len = assigned; assigned };
vm_block = 57; continue;
}
// C line 42149
59 => {
let _ = { let assigned = JS_DupValue(ctx, this_val); obj = assigned; assigned };
vm_block = 58; continue;
}
// C line 42156
60 => {
vm_block = 4; continue;
}
// C line 42155
61 => {
vm_block = if (js_get_length64(ctx, core::ptr::addr_of_mut!(len), obj)) != 0 { 60 } else { 55 }; continue;
}
// C line 42154
62 => {
let _ = { let assigned = JS_ToObject(ctx, this_val); obj = assigned; assigned };
vm_block = 61; continue;
}
// C line 42148
63 => {
vm_block = if (((special) & ((8 as i32)))) != 0 { 59 } else { 62 }; continue;
}
// C line 42147
64 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; val = assigned; assigned };
vm_block = 63; continue;
}
// C line 42146
65 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; acc = assigned; assigned };
vm_block = 64; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:42228. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_array_fill(mut ctx: *mut JSContext, mut this_val: JSValue, mut argc: i32, mut argv: *mut JSValue) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut obj: JSValue = core::mem::zeroed();
let mut len: i64 = core::mem::zeroed();
let mut start: i64 = core::mem::zeroed();
let mut end: i64 = core::mem::zeroed();
let mut vm_block: usize = 18;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 42261
1 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line ? labels: exception
2 => {
let _ = JS_FreeValue(ctx, obj);
vm_block = 1; continue;
}
// C line 42257
3 => {
return obj;
}
// C line 42251
4 => {
vm_block = if ((((start) < (end)) as i32)) != 0 { 7 } else { 3 }; continue;
}
// C line 42255
5 => {
let _ = { let old = start; start = (start).wrapping_add(1); old };
vm_block = 4; continue;
}
// C line 42254
6 => {
vm_block = 2; continue;
}
// C line 42252
7 => {
vm_block = if ((((JS_SetPropertyInt64(ctx, obj, start, JS_DupValue(ctx, *(argv).offset(((0 as i32)) as isize)))) < ((0 as i32))) as i32)) != 0 { 6 } else { 5 }; continue;
}
// C line 42247
8 => {
vm_block = 2; continue;
}
// C line 42246
9 => {
vm_block = if (JS_ToInt64Clamp(ctx, core::ptr::addr_of_mut!(end), *(argv).offset(((2 as i32)) as isize), (((0 as i32)) as i64), len, len)) != 0 { 8 } else { 4 }; continue;
}
// C line 42245
10 => {
vm_block = if ((((((((argc) > ((2 as i32))) as i32)) != 0) && (((!((JS_IsUndefined(*(argv).offset(((2 as i32)) as isize))) != 0) as i32)) != 0)) as i32)) != 0 { 9 } else { 4 }; continue;
}
// C line 42244
11 => {
let _ = { let assigned = len; end = assigned; assigned };
vm_block = 10; continue;
}
// C line 42241
12 => {
vm_block = 2; continue;
}
// C line 42240
13 => {
vm_block = if (JS_ToInt64Clamp(ctx, core::ptr::addr_of_mut!(start), *(argv).offset(((1 as i32)) as isize), (((0 as i32)) as i64), len, len)) != 0 { 12 } else { 11 }; continue;
}
// C line 42239
14 => {
vm_block = if ((((((((argc) > ((1 as i32))) as i32)) != 0) && (((!((JS_IsUndefined(*(argv).offset(((1 as i32)) as isize))) != 0) as i32)) != 0)) as i32)) != 0 { 13 } else { 11 }; continue;
}
// C line 42238
15 => {
let _ = { let assigned = (((0 as i32)) as i64); start = assigned; assigned };
vm_block = 14; continue;
}
// C line 42236
16 => {
vm_block = 2; continue;
}
// C line 42235
17 => {
vm_block = if (js_get_length64(ctx, core::ptr::addr_of_mut!(len), obj)) != 0 { 16 } else { 15 }; continue;
}
// C line 42234
18 => {
let _ = { let assigned = JS_ToObject(ctx, this_val); obj = assigned; assigned };
vm_block = 17; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:42264. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_array_includes(mut ctx: *mut JSContext, mut this_val: JSValue, mut argc: i32, mut argv: *mut JSValue) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut obj: JSValue = core::mem::zeroed();
let mut val: JSValue = core::mem::zeroed();
let mut len: i64 = core::mem::zeroed();
let mut n: i64 = core::mem::zeroed();
let mut arrp: *mut JSValue = core::mem::zeroed();
let mut count: u32 = core::mem::zeroed();
let mut res: i32 = core::mem::zeroed();
let mut vm_block: usize = 27;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 42311
1 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line ? labels: exception
2 => {
let _ = JS_FreeValue(ctx, obj);
vm_block = 1; continue;
}
// C line 42307
3 => {
return JS_NewBool(ctx, res);
}
// C line ? labels: done
4 => {
let _ = JS_FreeValue(ctx, obj);
vm_block = 3; continue;
}
// C line 42294
5 => {
vm_block = if ((((n) < (len)) as i32)) != 0 { 12 } else { 4 }; continue;
}
// C line ?
6 => {
let _ = { let old = n; n = (n).wrapping_add(1); old };
vm_block = 5; continue;
}
// C line 42301
7 => {
vm_block = 4; continue;
}
// C line 42300
8 => {
let _ = { let assigned = (1 as i32); res = assigned; assigned };
vm_block = 7; continue;
}
// C line 42298
9 => {
vm_block = if (js_strict_eq2(ctx, JS_DupValue(ctx, *(argv).offset(((0 as i32)) as isize)), val, (((JS_EQ_SAME_VALUE_ZERO as i32)) as JSStrictEqModeEnum))) != 0 { 8 } else { 6 }; continue;
}
// C line 42297
10 => {
vm_block = 2; continue;
}
// C line 42296
11 => {
vm_block = if (JS_IsException(val)) != 0 { 10 } else { 9 }; continue;
}
// C line 42295
12 => {
let _ = { let assigned = JS_GetPropertyInt64(ctx, obj, n); val = assigned; assigned };
vm_block = 11; continue;
}
// C line 42285
13 => {
vm_block = if ((((n) < (((count) as i64))) as i32)) != 0 { 17 } else { 5 }; continue;
}
// C line ?
14 => {
let _ = { let old = n; n = (n).wrapping_add(1); old };
vm_block = 13; continue;
}
// C line 42290
15 => {
vm_block = 4; continue;
}
// C line 42289
16 => {
let _ = { let assigned = (1 as i32); res = assigned; assigned };
vm_block = 15; continue;
}
// C line 42286
17 => {
vm_block = if (js_strict_eq2(ctx, JS_DupValue(ctx, *(argv).offset(((0 as i32)) as isize)), JS_DupValue(ctx, *(arrp).offset((n) as isize)), (((JS_EQ_SAME_VALUE_ZERO as i32)) as JSStrictEqModeEnum))) != 0 { 16 } else { 14 }; continue;
}
// C line 42284
18 => {
vm_block = if (js_get_fast_array(ctx, obj, core::ptr::addr_of_mut!(arrp), core::ptr::addr_of_mut!(count))) != 0 { 13 } else { 5 }; continue;
}
// C line 42282
19 => {
vm_block = 2; continue;
}
// C line 42281
20 => {
vm_block = if (JS_ToInt64Clamp(ctx, core::ptr::addr_of_mut!(n), *(argv).offset(((1 as i32)) as isize), (((0 as i32)) as i64), len, len)) != 0 { 19 } else { 18 }; continue;
}
// C line 42280
21 => {
vm_block = if ((((argc) > ((1 as i32))) as i32)) != 0 { 20 } else { 18 }; continue;
}
// C line 42279
22 => {
let _ = { let assigned = (((0 as i32)) as i64); n = assigned; assigned };
vm_block = 21; continue;
}
// C line 42278
23 => {
vm_block = if ((((len) > ((((0 as i32)) as i64))) as i32)) != 0 { 22 } else { 4 }; continue;
}
// C line 42277
24 => {
let _ = { let assigned = (0 as i32); res = assigned; assigned };
vm_block = 23; continue;
}
// C line 42275
25 => {
vm_block = 2; continue;
}
// C line 42274
26 => {
vm_block = if (js_get_length64(ctx, core::ptr::addr_of_mut!(len), obj)) != 0 { 25 } else { 24 }; continue;
}
// C line 42273
27 => {
let _ = { let assigned = JS_ToObject(ctx, this_val); obj = assigned; assigned };
vm_block = 26; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:42314. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_array_indexOf(mut ctx: *mut JSContext, mut this_val: JSValue, mut argc: i32, mut argv: *mut JSValue) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut obj: JSValue = core::mem::zeroed();
let mut val: JSValue = core::mem::zeroed();
let mut len: i64 = core::mem::zeroed();
let mut n: i64 = core::mem::zeroed();
let mut res: i64 = core::mem::zeroed();
let mut arrp: *mut JSValue = core::mem::zeroed();
let mut count: u32 = core::mem::zeroed();
let mut present: i32 = core::mem::zeroed();
let mut vm_block: usize = 28;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 42360
1 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line ? labels: exception
2 => {
let _ = JS_FreeValue(ctx, obj);
vm_block = 1; continue;
}
// C line 42356
3 => {
return JS_NewInt64(ctx, res);
}
// C line ? labels: done
4 => {
let _ = JS_FreeValue(ctx, obj);
vm_block = 3; continue;
}
// C line 42342
5 => {
vm_block = if ((((n) < (len)) as i32)) != 0 { 13 } else { 4 }; continue;
}
// C line ?
6 => {
let _ = { let old = n; n = (n).wrapping_add(1); old };
vm_block = 5; continue;
}
// C line 42349
7 => {
vm_block = 4; continue;
}
// C line 42348
8 => {
let _ = { let assigned = n; res = assigned; assigned };
vm_block = 7; continue;
}
// C line 42347
9 => {
vm_block = if (js_strict_eq2(ctx, JS_DupValue(ctx, *(argv).offset(((0 as i32)) as isize)), val, (((JS_EQ_STRICT as i32)) as JSStrictEqModeEnum))) != 0 { 8 } else { 6 }; continue;
}
// C line 42346
10 => {
vm_block = if (present) != 0 { 9 } else { 6 }; continue;
}
// C line 42345
11 => {
vm_block = 2; continue;
}
// C line 42344
12 => {
vm_block = if ((((present) < ((0 as i32))) as i32)) != 0 { 11 } else { 10 }; continue;
}
// C line 42343
13 => {
present = JS_TryGetPropertyInt64(ctx, obj, n, core::ptr::addr_of_mut!(val));
vm_block = 12; continue;
}
// C line 42334
14 => {
vm_block = if ((((n) < (((count) as i64))) as i32)) != 0 { 18 } else { 5 }; continue;
}
// C line ?
15 => {
let _ = { let old = n; n = (n).wrapping_add(1); old };
vm_block = 14; continue;
}
// C line 42338
16 => {
vm_block = 4; continue;
}
// C line 42337
17 => {
let _ = { let assigned = n; res = assigned; assigned };
vm_block = 16; continue;
}
// C line 42335
18 => {
vm_block = if (js_strict_eq2(ctx, JS_DupValue(ctx, *(argv).offset(((0 as i32)) as isize)), JS_DupValue(ctx, *(arrp).offset((n) as isize)), (((JS_EQ_STRICT as i32)) as JSStrictEqModeEnum))) != 0 { 17 } else { 15 }; continue;
}
// C line 42333
19 => {
vm_block = if (js_get_fast_array(ctx, obj, core::ptr::addr_of_mut!(arrp), core::ptr::addr_of_mut!(count))) != 0 { 14 } else { 5 }; continue;
}
// C line 42331
20 => {
vm_block = 2; continue;
}
// C line 42330
21 => {
vm_block = if (JS_ToInt64Clamp(ctx, core::ptr::addr_of_mut!(n), *(argv).offset(((1 as i32)) as isize), (((0 as i32)) as i64), len, len)) != 0 { 20 } else { 19 }; continue;
}
// C line 42329
22 => {
vm_block = if ((((argc) > ((1 as i32))) as i32)) != 0 { 21 } else { 19 }; continue;
}
// C line 42328
23 => {
let _ = { let assigned = (((0 as i32)) as i64); n = assigned; assigned };
vm_block = 22; continue;
}
// C line 42327
24 => {
vm_block = if ((((len) > ((((0 as i32)) as i64))) as i32)) != 0 { 23 } else { 4 }; continue;
}
// C line 42326
25 => {
let _ = { let assigned = ((((1 as i32)).wrapping_neg()) as i64); res = assigned; assigned };
vm_block = 24; continue;
}
// C line 42324
26 => {
vm_block = 2; continue;
}
// C line 42323
27 => {
vm_block = if (js_get_length64(ctx, core::ptr::addr_of_mut!(len), obj)) != 0 { 26 } else { 25 }; continue;
}
// C line 42322
28 => {
let _ = { let assigned = JS_ToObject(ctx, this_val); obj = assigned; assigned };
vm_block = 27; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:42363. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_array_lastIndexOf(mut ctx: *mut JSContext, mut this_val: JSValue, mut argc: i32, mut argv: *mut JSValue) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut obj: JSValue = core::mem::zeroed();
let mut val: JSValue = core::mem::zeroed();
let mut len: i64 = core::mem::zeroed();
let mut n: i64 = core::mem::zeroed();
let mut res: i64 = core::mem::zeroed();
let mut present: i32 = core::mem::zeroed();
let mut vm_block: usize = 22;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 42399
1 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line ? labels: exception
2 => {
let _ = JS_FreeValue(ctx, obj);
vm_block = 1; continue;
}
// C line 42395
3 => {
return JS_NewInt64(ctx, res);
}
// C line 42394
4 => {
let _ = JS_FreeValue(ctx, obj);
vm_block = 3; continue;
}
// C line 42382
5 => {
vm_block = if ((((n) >= ((((0 as i32)) as i64))) as i32)) != 0 { 13 } else { 4 }; continue;
}
// C line ?
6 => {
let _ = { let old = n; n = (n).wrapping_sub(1); old };
vm_block = 5; continue;
}
// C line 42389
7 => {
vm_block = 4; continue;
}
// C line 42388
8 => {
let _ = { let assigned = n; res = assigned; assigned };
vm_block = 7; continue;
}
// C line 42387
9 => {
vm_block = if (js_strict_eq2(ctx, JS_DupValue(ctx, *(argv).offset(((0 as i32)) as isize)), val, (((JS_EQ_STRICT as i32)) as JSStrictEqModeEnum))) != 0 { 8 } else { 6 }; continue;
}
// C line 42386
10 => {
vm_block = if (present) != 0 { 9 } else { 6 }; continue;
}
// C line 42385
11 => {
vm_block = 2; continue;
}
// C line 42384
12 => {
vm_block = if ((((present) < ((0 as i32))) as i32)) != 0 { 11 } else { 10 }; continue;
}
// C line 42383
13 => {
let _ = { let assigned = JS_TryGetPropertyInt64(ctx, obj, n, core::ptr::addr_of_mut!(val)); present = assigned; assigned };
vm_block = 12; continue;
}
// C line 42379
14 => {
vm_block = 2; continue;
}
// C line 42378
15 => {
vm_block = if (JS_ToInt64Clamp(ctx, core::ptr::addr_of_mut!(n), *(argv).offset(((1 as i32)) as isize), ((((1 as i32)).wrapping_neg()) as i64), (len).wrapping_sub((((1 as i32)) as i64)), len)) != 0 { 14 } else { 5 }; continue;
}
// C line 42377
16 => {
vm_block = if ((((argc) > ((1 as i32))) as i32)) != 0 { 15 } else { 5 }; continue;
}
// C line 42376
17 => {
let _ = { let assigned = (len).wrapping_sub((((1 as i32)) as i64)); n = assigned; assigned };
vm_block = 16; continue;
}
// C line 42375
18 => {
vm_block = if ((((len) > ((((0 as i32)) as i64))) as i32)) != 0 { 17 } else { 4 }; continue;
}
// C line 42374
19 => {
let _ = { let assigned = ((((1 as i32)).wrapping_neg()) as i64); res = assigned; assigned };
vm_block = 18; continue;
}
// C line 42372
20 => {
vm_block = 2; continue;
}
// C line 42371
21 => {
vm_block = if (js_get_length64(ctx, core::ptr::addr_of_mut!(len), obj)) != 0 { 20 } else { 19 }; continue;
}
// C line 42370
22 => {
let _ = { let assigned = JS_ToObject(ctx, this_val); obj = assigned; assigned };
vm_block = 21; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:42409. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_array_find(mut ctx: *mut JSContext, mut this_val: JSValue, mut argc: i32, mut argv: *mut JSValue, mut mode: i32) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut func: JSValue = core::mem::zeroed();
let mut this_arg: JSValue = core::mem::zeroed();
let mut args: [JSValue; 3] = core::mem::zeroed();
let mut obj: JSValue = core::mem::zeroed();
let mut val: JSValue = core::mem::zeroed();
let mut index_val: JSValue = core::mem::zeroed();
let mut res: JSValue = core::mem::zeroed();
let mut len: i64 = core::mem::zeroed();
let mut k: i64 = core::mem::zeroed();
let mut end: i64 = core::mem::zeroed();
let mut dir: i32 = core::mem::zeroed();
let mut vm_block: usize = 50;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 42479
1 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 42478
2 => {
let _ = JS_FreeValue(ctx, obj);
vm_block = 1; continue;
}
// C line 42477
3 => {
let _ = JS_FreeValue(ctx, val);
vm_block = 2; continue;
}
// C line ? labels: exception
4 => {
let _ = JS_FreeValue(ctx, index_val);
vm_block = 3; continue;
}
// C line 42471
5 => {
return JS_NewInt32(ctx, ((1 as i32)).wrapping_neg());
}
// C line 42473
6 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) };
}
// C line 42470
7 => {
vm_block = if ((((((((mode) == ((ArrayFindIndex as i32))) as i32)) != 0) || (((((mode) == ((ArrayFindLastIndex as i32))) as i32)) != 0)) as i32)) != 0 { 5 } else { 6 }; continue;
}
// C line 42469
8 => {
let _ = JS_FreeValue(ctx, obj);
vm_block = 7; continue;
}
// C line 42442
9 => {
vm_block = if ((((k) != (end)) as i32)) != 0 { 32 } else { 8 }; continue;
}
// C line ?
10 => {
let _ = { k = (k).wrapping_add(((dir) as i64)); k };
vm_block = 9; continue;
}
// C line 42467
11 => {
let _ = JS_FreeValue(ctx, index_val);
vm_block = 10; continue;
}
// C line 42466
12 => {
let _ = JS_FreeValue(ctx, val);
vm_block = 11; continue;
}
// C line 42459
13 => {
return index_val;
}
// C line 42458
14 => {
let _ = JS_FreeValue(ctx, obj);
vm_block = 13; continue;
}
// C line 42457
15 => {
let _ = JS_FreeValue(ctx, val);
vm_block = 14; continue;
}
// C line 42463
16 => {
return val;
}
// C line 42462
17 => {
let _ = JS_FreeValue(ctx, obj);
vm_block = 16; continue;
}
// C line 42461
18 => {
let _ = JS_FreeValue(ctx, index_val);
vm_block = 17; continue;
}
// C line 42456
19 => {
vm_block = if ((((((((mode) == ((ArrayFindIndex as i32))) as i32)) != 0) || (((((mode) == ((ArrayFindLastIndex as i32))) as i32)) != 0)) as i32)) != 0 { 15 } else { 18 }; continue;
}
// C line 42455
20 => {
vm_block = if (JS_ToBoolFree(ctx, res)) != 0 { 19 } else { 12 }; continue;
}
// C line 42454
21 => {
vm_block = 4; continue;
}
// C line 42453
22 => {
vm_block = if (JS_IsException(res)) != 0 { 21 } else { 20 }; continue;
}
// C line 42452
23 => {
let _ = { let assigned = JS_Call(ctx, func, this_arg, (3 as i32), (args).as_mut_ptr()); res = assigned; assigned };
vm_block = 22; continue;
}
// C line 42451
24 => {
let _ = { let assigned = this_val; *((args).as_mut_ptr()).offset(((2 as i32)) as isize) = assigned; assigned };
vm_block = 23; continue;
}
// C line 42450
25 => {
let _ = { let assigned = index_val; *((args).as_mut_ptr()).offset(((1 as i32)) as isize) = assigned; assigned };
vm_block = 24; continue;
}
// C line 42449
26 => {
let _ = { let assigned = val; *((args).as_mut_ptr()).offset(((0 as i32)) as isize) = assigned; assigned };
vm_block = 25; continue;
}
// C line 42448
27 => {
vm_block = 4; continue;
}
// C line 42447
28 => {
vm_block = if (JS_IsException(val)) != 0 { 27 } else { 26 }; continue;
}
// C line 42446
29 => {
let _ = { let assigned = JS_GetPropertyValue(ctx, obj, index_val); val = assigned; assigned };
vm_block = 28; continue;
}
// C line 42445
30 => {
vm_block = 4; continue;
}
// C line 42444
31 => {
vm_block = if (JS_IsException(index_val)) != 0 { 30 } else { 29 }; continue;
}
// C line 42443
32 => {
let _ = { let assigned = JS_NewInt64(ctx, k); index_val = assigned; assigned };
vm_block = 31; continue;
}
// C line 42438
33 => {
let _ = { let assigned = ((((1 as i32)).wrapping_neg()) as i64); end = assigned; assigned };
vm_block = 9; continue;
}
// C line 42437
34 => {
let _ = { let assigned = ((1 as i32)).wrapping_neg(); dir = assigned; assigned };
vm_block = 33; continue;
}
// C line 42436
35 => {
let _ = { let assigned = (len).wrapping_sub((((1 as i32)) as i64)); k = assigned; assigned };
vm_block = 34; continue;
}
// C line 42435
36 => {
vm_block = if ((((((((mode) == ((ArrayFindLast as i32))) as i32)) != 0) || (((((mode) == ((ArrayFindLastIndex as i32))) as i32)) != 0)) as i32)) != 0 { 35 } else { 9 }; continue;
}
// C line 42434
37 => {
let _ = { let assigned = len; end = assigned; assigned };
vm_block = 36; continue;
}
// C line 42433
38 => {
let _ = { let assigned = (1 as i32); dir = assigned; assigned };
vm_block = 37; continue;
}
// C line 42432
39 => {
let _ = { let assigned = (((0 as i32)) as i64); k = assigned; assigned };
vm_block = 38; continue;
}
// C line 42430
40 => {
let _ = { let assigned = *(argv).offset(((1 as i32)) as isize); this_arg = assigned; assigned };
vm_block = 39; continue;
}
// C line 42429
41 => {
vm_block = if ((((argc) > ((1 as i32))) as i32)) != 0 { 40 } else { 39 }; continue;
}
// C line 42428
42 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; this_arg = assigned; assigned };
vm_block = 41; continue;
}
// C line 42426
43 => {
vm_block = 4; continue;
}
// C line 42425
44 => {
vm_block = if (check_function(ctx, func)) != 0 { 43 } else { 42 }; continue;
}
// C line 42424
45 => {
let _ = { let assigned = *(argv).offset(((0 as i32)) as isize); func = assigned; assigned };
vm_block = 44; continue;
}
// C line 42422
46 => {
vm_block = 4; continue;
}
// C line 42421
47 => {
vm_block = if (js_get_length64(ctx, core::ptr::addr_of_mut!(len), obj)) != 0 { 46 } else { 45 }; continue;
}
// C line 42420
48 => {
let _ = { let assigned = JS_ToObject(ctx, this_val); obj = assigned; assigned };
vm_block = 47; continue;
}
// C line 42419
49 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; val = assigned; assigned };
vm_block = 48; continue;
}
// C line 42418
50 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; index_val = assigned; assigned };
vm_block = 49; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:42482. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_array_toString(mut ctx: *mut JSContext, mut this_val: JSValue, mut argc: i32, mut argv: *mut JSValue) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut obj: JSValue = core::mem::zeroed();
let mut method: JSValue = core::mem::zeroed();
let mut ret: JSValue = core::mem::zeroed();
let mut vm_block: usize = 12;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 42502
1 => {
return ret;
}
// C line 42501
2 => {
let _ = JS_FreeValue(ctx, obj);
vm_block = 1; continue;
}
// C line 42492
3 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) }; ret = assigned; assigned };
vm_block = 2; continue;
}
// C line 42497
4 => {
let _ = { let assigned = js_object_toString(ctx, obj, (0 as i32), core::ptr::null_mut::<JSValue>()); ret = assigned; assigned };
vm_block = 2; continue;
}
// C line 42496
5 => {
let _ = JS_FreeValue(ctx, method);
vm_block = 4; continue;
}
// C line 42499
6 => {
let _ = { let assigned = JS_CallFree(ctx, method, obj, (0 as i32), core::ptr::null_mut::<JSValue>()); ret = assigned; assigned };
vm_block = 2; continue;
}
// C line 42494
7 => {
vm_block = if ((!((JS_IsFunction(ctx, method)) != 0) as i32)) != 0 { 5 } else { 6 }; continue;
}
// C line 42491
8 => {
vm_block = if (JS_IsException(method)) != 0 { 3 } else { 7 }; continue;
}
// C line 42490
9 => {
let _ = { let assigned = JS_GetProperty(ctx, obj, (((crate::quickjs_atom::JS_ATOM_join as i32)) as JSAtom)); method = assigned; assigned };
vm_block = 8; continue;
}
// C line 42489
10 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 42488
11 => {
vm_block = if (JS_IsException(obj)) != 0 { 10 } else { 9 }; continue;
}
// C line 42487
12 => {
let _ = { let assigned = JS_ToObject(ctx, this_val); obj = assigned; assigned };
vm_block = 11; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:42505. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_array_join(mut ctx: *mut JSContext, mut this_val: JSValue, mut argc: i32, mut argv: *mut JSValue, mut toLocaleString: i32) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut obj: JSValue = core::mem::zeroed();
let mut sep: JSValue = core::mem::zeroed();
let mut el: JSValue = core::mem::zeroed();
let mut b_s: StringBuffer = core::mem::zeroed();
let mut b: *mut StringBuffer = core::mem::zeroed();
let mut p: *mut JSString = core::mem::zeroed();
let mut i: i64 = core::mem::zeroed();
let mut n: i64 = core::mem::zeroed();
let mut c: i32 = core::mem::zeroed();
let mut vm_block: usize = 38;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 42559
1 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line ? labels: exception
2 => {
let _ = JS_FreeValue(ctx, obj);
vm_block = 1; continue;
}
// C line 42556
3 => {
let _ = JS_FreeValue(ctx, sep);
vm_block = 2; continue;
}
// C line ? labels: fail
4 => {
let _ = string_buffer_free(b);
vm_block = 3; continue;
}
// C line 42552
5 => {
return string_buffer_end(b);
}
// C line 42551
6 => {
let _ = JS_FreeValue(ctx, obj);
vm_block = 5; continue;
}
// C line 42550
7 => {
let _ = JS_FreeValue(ctx, sep);
vm_block = 6; continue;
}
// C line 42531
8 => {
vm_block = if ((((i) < (n)) as i32)) != 0 { 21 } else { 7 }; continue;
}
// C line ?
9 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 8; continue;
}
// C line 42547
10 => {
vm_block = 4; continue;
}
// C line 42546
11 => {
vm_block = if (string_buffer_concat_value_free(b, el)) != 0 { 10 } else { 9 }; continue;
}
// C line 42544
12 => {
let _ = { let assigned = JS_ToLocaleStringFree(ctx, el); el = assigned; assigned };
vm_block = 11; continue;
}
// C line 42543
13 => {
vm_block = if (toLocaleString) != 0 { 12 } else { 11 }; continue;
}
// C line 42542
14 => {
vm_block = if ((((((!((JS_IsNull(el)) != 0) as i32)) != 0) && (((!((JS_IsUndefined(el)) != 0) as i32)) != 0)) as i32)) != 0 { 13 } else { 9 }; continue;
}
// C line 42541
15 => {
vm_block = 4; continue;
}
// C line 42540
16 => {
vm_block = if (JS_IsException(el)) != 0 { 15 } else { 14 }; continue;
}
// C line 42539
17 => {
let _ = { let assigned = JS_GetPropertyUint32(ctx, obj, ((i) as u32)); el = assigned; assigned };
vm_block = 16; continue;
}
// C line 42534
18 => {
let _ = string_buffer_putc8(b, ((c) as u32));
vm_block = 17; continue;
}
// C line 42536
19 => {
let _ = string_buffer_concat(b, p, (((0 as i32)) as u32), (*(p)).len());
vm_block = 17; continue;
}
// C line 42533
20 => {
vm_block = if ((((c) >= ((0 as i32))) as i32)) != 0 { 18 } else { 19 }; continue;
}
// C line 42532
21 => {
vm_block = if ((((i) > ((((0 as i32)) as i64))) as i32)) != 0 { 20 } else { 17 }; continue;
}
// C line 42531
22 => {
let _ = { let assigned = (((0 as i32)) as i64); i = assigned; assigned };
vm_block = 8; continue;
}
// C line 42529
23 => {
let _ = string_buffer_init(ctx, b, (0 as i32));
vm_block = 22; continue;
}
// C line 42525
24 => {
let _ = { let assigned = ((*((((*(p)).u).str8).as_mut_ptr()).offset(((0 as i32)) as isize)) as i32); c = assigned; assigned };
vm_block = 23; continue;
}
// C line 42527
25 => {
let _ = { let assigned = ((1 as i32)).wrapping_neg(); c = assigned; assigned };
vm_block = 23; continue;
}
// C line 42524
26 => {
vm_block = if (((((((((((*(p)).len()) as i32)) == ((1 as i32))) as i32)) != 0) && (((!(((*(p)).is_wide_char()) != 0) as i32)) != 0)) as i32)) != 0 { 24 } else { 25 }; continue;
}
// C line 42523
27 => {
let _ = { let assigned = ((((sep).u).ptr) as *mut JSString); p = assigned; assigned };
vm_block = 26; continue;
}
// C line 42522
28 => {
vm_block = 2; continue;
}
// C line 42521
29 => {
vm_block = if (JS_IsException(sep)) != 0 { 28 } else { 27 }; continue;
}
// C line 42520
30 => {
let _ = { let assigned = JS_ToString(ctx, *(argv).offset(((0 as i32)) as isize)); sep = assigned; assigned };
vm_block = 29; continue;
}
// C line 42519
31 => {
vm_block = if ((((((((((!((toLocaleString) != 0) as i32)) != 0) && (((((argc) > ((0 as i32))) as i32)) != 0)) as i32)) != 0) && (((!((JS_IsUndefined(*(argv).offset(((0 as i32)) as isize))) != 0) as i32)) != 0)) as i32)) != 0 { 30 } else { 23 }; continue;
}
// C line 42518
32 => {
let _ = { let assigned = (44 as i32); c = assigned; assigned };
vm_block = 31; continue;
}
// C line 42516
33 => {
vm_block = 2; continue;
}
// C line 42515
34 => {
vm_block = if (js_get_length64(ctx, core::ptr::addr_of_mut!(n), obj)) != 0 { 33 } else { 32 }; continue;
}
// C line 42514
35 => {
let _ = { let assigned = JS_ToObject(ctx, this_val); obj = assigned; assigned };
vm_block = 34; continue;
}
// C line 42510
36 => {
p = core::ptr::null_mut::<JSString>();
vm_block = 35; continue;
}
// C line 42509
37 => {
b = core::ptr::addr_of_mut!(b_s);
vm_block = 36; continue;
}
// C line 42508
38 => {
sep = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) };
vm_block = 37; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:42562. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_array_pop(mut ctx: *mut JSContext, mut this_val: JSValue, mut argc: i32, mut argv: *mut JSValue, mut shift: i32) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut obj: JSValue = core::mem::zeroed();
let mut res: JSValue = core::mem::zeroed();
let mut len: i64 = core::mem::zeroed();
let mut newLen: i64 = core::mem::zeroed();
let mut arrp: *mut JSValue = core::mem::zeroed();
let mut count32: u32 = core::mem::zeroed();
let mut p: *mut JSObject = core::mem::zeroed();
let mut vm_block: usize = 33;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 42612
1 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 42611
2 => {
let _ = JS_FreeValue(ctx, obj);
vm_block = 1; continue;
}
// C line ? labels: exception
3 => {
let _ = JS_FreeValue(ctx, res);
vm_block = 2; continue;
}
// C line 42607
4 => {
return res;
}
// C line 42606
5 => {
let _ = JS_FreeValue(ctx, obj);
vm_block = 4; continue;
}
// C line 42604
6 => {
vm_block = 3; continue;
}
// C line 42603
7 => {
vm_block = if ((((JS_SetProperty(ctx, obj, (((crate::quickjs_atom::JS_ATOM_length as i32)) as JSAtom), JS_NewInt64(ctx, newLen))) < ((0 as i32))) as i32)) != 0 { 6 } else { 5 }; continue;
}
// C line 42582
8 => {
let _ = { let old = (((*(p)).u).array).count; (((*(p)).u).array).count = ((((*(p)).u).array).count).wrapping_sub(1); old };
vm_block = 7; continue;
}
// C line 42581
9 => {
let _ = { core::ptr::copy(((((arrp).offset((((1 as i32)) as isize))) as *const c_void)).cast::<u8>(), (((arrp) as *mut c_void)).cast::<u8>(), ((((count32).wrapping_sub((((1 as i32)) as u32))) as usize)).wrapping_mul((size_of::<JSValue>() as usize))); ((arrp) as *mut c_void) };
vm_block = 8; continue;
}
// C line 42580
10 => {
let _ = { let assigned = *(arrp).offset(((0 as i32)) as isize); res = assigned; assigned };
vm_block = 9; continue;
}
// C line 42585
11 => {
let _ = { let old = (((*(p)).u).array).count; (((*(p)).u).array).count = ((((*(p)).u).array).count).wrapping_sub(1); old };
vm_block = 7; continue;
}
// C line 42584
12 => {
let _ = { let assigned = *(arrp).offset(((count32).wrapping_sub((((1 as i32)) as u32))) as isize); res = assigned; assigned };
vm_block = 11; continue;
}
// C line 42579
13 => {
vm_block = if (shift) != 0 { 10 } else { 12 }; continue;
}
// C line 42578
14 => {
p = ((((obj).u).ptr) as *mut JSObject);
vm_block = 13; continue;
}
// C line 42600
15 => {
vm_block = 3; continue;
}
// C line 42599
16 => {
vm_block = if ((((JS_DeletePropertyInt64(ctx, obj, newLen, ((1 as i32)).wrapping_shl(((14 as i32)) as u32))) < ((0 as i32))) as i32)) != 0 { 15 } else { 7 }; continue;
}
// C line 42593
17 => {
vm_block = 3; continue;
}
// C line 42592
18 => {
vm_block = if (JS_CopySubArray(ctx, obj, (((0 as i32)) as i64), (((1 as i32)) as i64), (len).wrapping_sub((((1 as i32)) as i64)), (1 as i32))) != 0 { 17 } else { 16 }; continue;
}
// C line 42591
19 => {
vm_block = 3; continue;
}
// C line 42590
20 => {
vm_block = if (JS_IsException(res)) != 0 { 19 } else { 18 }; continue;
}
// C line 42589
21 => {
let _ = { let assigned = JS_GetPropertyInt64(ctx, obj, (((0 as i32)) as i64)); res = assigned; assigned };
vm_block = 20; continue;
}
// C line 42597
22 => {
vm_block = 3; continue;
}
// C line 42596
23 => {
vm_block = if (JS_IsException(res)) != 0 { 22 } else { 16 }; continue;
}
// C line 42595
24 => {
let _ = { let assigned = JS_GetPropertyInt64(ctx, obj, newLen); res = assigned; assigned };
vm_block = 23; continue;
}
// C line 42588
25 => {
vm_block = if (shift) != 0 { 21 } else { 24 }; continue;
}
// C line 42577
26 => {
vm_block = if (((((js_get_fast_array(ctx, obj, core::ptr::addr_of_mut!(arrp), core::ptr::addr_of_mut!(count32))) != 0) && (((((((count32) as i64)) == (len)) as i32)) != 0)) as i32)) != 0 { 14 } else { 25 }; continue;
}
// C line 42575
27 => {
let _ = { let assigned = (len).wrapping_sub((((1 as i32)) as i64)); newLen = assigned; assigned };
vm_block = 26; continue;
}
// C line 42574
28 => {
vm_block = if ((((len) > ((((0 as i32)) as i64))) as i32)) != 0 { 27 } else { 7 }; continue;
}
// C line 42573
29 => {
let _ = { let assigned = (((0 as i32)) as i64); newLen = assigned; assigned };
vm_block = 28; continue;
}
// C line 42572
30 => {
vm_block = 3; continue;
}
// C line 42571
31 => {
vm_block = if (js_get_length64(ctx, core::ptr::addr_of_mut!(len), obj)) != 0 { 30 } else { 29 }; continue;
}
// C line 42570
32 => {
let _ = { let assigned = JS_ToObject(ctx, this_val); obj = assigned; assigned };
vm_block = 31; continue;
}
// C line 42565
33 => {
res = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) };
vm_block = 32; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:42615. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_array_push(mut ctx: *mut JSContext, mut this_val: JSValue, mut argc: i32, mut argv: *mut JSValue, mut unshift: i32) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut obj: JSValue = core::mem::zeroed();
let mut i: i32 = core::mem::zeroed();
let mut len: i64 = core::mem::zeroed();
let mut from: i64 = core::mem::zeroed();
let mut newLen: i64 = core::mem::zeroed();
let mut p: *mut JSObject = core::mem::zeroed();
let mut new_len: u32 = core::mem::zeroed();
let mut vm_block: usize = 38;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 42672
1 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line ? labels: exception
2 => {
let _ = JS_FreeValue(ctx, obj);
vm_block = 1; continue;
}
// C line 42668
3 => {
return JS_NewInt64(ctx, newLen);
}
// C line 42667
4 => {
let _ = JS_FreeValue(ctx, obj);
vm_block = 3; continue;
}
// C line 42665
5 => {
vm_block = 2; continue;
}
// C line 42664
6 => {
vm_block = if ((((JS_SetProperty(ctx, obj, (((crate::quickjs_atom::JS_ATOM_length as i32)) as JSAtom), JS_NewInt64(ctx, newLen))) < ((0 as i32))) as i32)) != 0 { 5 } else { 4 }; continue;
}
// C line 42659
7 => {
vm_block = if ((((i) < (argc)) as i32)) != 0 { 10 } else { 6 }; continue;
}
// C line ?
8 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 7; continue;
}
// C line 42662
9 => {
vm_block = 2; continue;
}
// C line 42660
10 => {
vm_block = if ((((JS_SetPropertyInt64(ctx, obj, (from).wrapping_add(((i) as i64)), JS_DupValue(ctx, *(argv).offset((i) as isize)))) < ((0 as i32))) as i32)) != 0 { 9 } else { 8 }; continue;
}
// C line 42659
11 => {
let _ = { let assigned = (0 as i32); i = assigned; assigned };
vm_block = 7; continue;
}
// C line 42657
12 => {
let _ = { let assigned = (((0 as i32)) as i64); from = assigned; assigned };
vm_block = 11; continue;
}
// C line 42656
13 => {
vm_block = 2; continue;
}
// C line 42655
14 => {
vm_block = if (JS_CopySubArray(ctx, obj, ((argc) as i64), (((0 as i32)) as i64), len, ((1 as i32)).wrapping_neg())) != 0 { 13 } else { 12 }; continue;
}
// C line 42654
15 => {
vm_block = if (((((unshift) != 0) && (((((argc) > ((0 as i32))) as i32)) != 0)) as i32)) != 0 { 14 } else { 11 }; continue;
}
// C line 42653
16 => {
let _ = { let assigned = len; from = assigned; assigned };
vm_block = 15; continue;
}
// C line 42651
17 => {
vm_block = 2; continue;
}
// C line 42650
18 => {
let _ = JS_ThrowTypeError(ctx, c"Array loo long".as_ptr());
vm_block = 17; continue;
}
// C line 42649
19 => {
vm_block = if ((((newLen) > ((((((1 as i32)) as i64)).wrapping_shl(((53 as i32)) as u32)).wrapping_sub((((1 as i32)) as i64)))) as i32)) != 0 { 18 } else { 16 }; continue;
}
// C line 42648
20 => {
let _ = { let assigned = (len).wrapping_add(((argc) as i64)); newLen = assigned; assigned };
vm_block = 19; continue;
}
// C line 42647
21 => {
vm_block = 2; continue;
}
// C line 42646
22 => {
vm_block = if (js_get_length64(ctx, core::ptr::addr_of_mut!(len), obj)) != 0 { 21 } else { 20 }; continue;
}
// C line 42645
23 => {
let _ = { let assigned = JS_ToObject(ctx, this_val); obj = assigned; assigned };
vm_block = 22; continue;
}
// C line 42641
24 => {
return JS_NewInt32(ctx, ((new_len) as i32));
}
// C line 42640
25 => {
let _ = { let assigned = new_len; (((*(p)).u).array).count = assigned; assigned };
vm_block = 24; continue;
}
// C line 42639
26 => {
let _ = { let assigned = JS_NewInt32(ctx, ((new_len) as i32)); ((*((*(p)).prop).offset(((0 as i32)) as isize)).u).value = assigned; assigned };
vm_block = 25; continue;
}
// C line 42637
27 => {
vm_block = if ((((i) < (argc)) as i32)) != 0 { 29 } else { 26 }; continue;
}
// C line ?
28 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 27; continue;
}
// C line 42638
29 => {
let _ = { let assigned = JS_DupValue(ctx, *(argv).offset((i) as isize)); *(((((*(p)).u).array).u).values).offset((((((*(p)).u).array).count).wrapping_add(((i) as u32))) as isize) = assigned; assigned };
vm_block = 28; continue;
}
// C line 42637
30 => {
let _ = { let assigned = (0 as i32); i = assigned; assigned };
vm_block = 27; continue;
}
// C line 42635
31 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 42634
32 => {
vm_block = if (expand_fast_array(ctx, p, new_len)) != 0 { 31 } else { 30 }; continue;
}
// C line 42633
33 => {
vm_block = if ((((!(((!(((((new_len) > (((((*(p)).u).array).u1).size)) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 32 } else { 30 }; continue;
}
// C line 42632
34 => {
vm_block = if ((((!(((!(((((new_len) <= ((((2147483647 as i32)) as u32))) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 33 } else { 23 }; continue;
}
// C line 42631
35 => {
let _ = { let assigned = ((((*(p)).u).array).count).wrapping_add(((argc) as u32)); new_len = assigned; assigned };
vm_block = 34; continue;
}
// C line 42624
36 => {
vm_block = if ((((!(((!((((((((((((((((((((((((((((*(p)).class_id) as i32)) == ((JS_CLASS_ARRAY as i32))) as i32)) != 0) && (((((*(p)).fast_array()) as i32)) != 0)) as i32)) != 0) && ((can_extend_fast_array(p)) != 0)) as i32)) != 0) && ((((((((((*((*(p)).prop).offset(((0 as i32)) as isize)).u).value).tag) as i32)) == ((JS_TAG_INT as i32))) as i32)) != 0)) as i32)) != 0) && (((((((((((((*((*(p)).prop).offset(((0 as i32)) as isize)).u).value).u).uint64) as i32)) as u32)) == ((((*(p)).u).array).count)) as i32)) != 0)) as i32)) != 0) && ((((((((((*(get_shape_prop((*(p)).shape))).flags()) as i32)) & (((1 as i32)).wrapping_shl(((1 as i32)) as u32)))) != ((0 as i32))) as i32)) != 0)) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 35 } else { 23 }; continue;
}
// C line 42623
37 => {
p = ((((this_val).u).ptr) as *mut JSObject);
vm_block = 36; continue;
}
// C line 42622
38 => {
vm_block = if ((((!(((!((((((((((((this_val).tag) as i32)) == ((JS_TAG_OBJECT as i32))) as i32)) != 0) && (((!((unshift) != 0) as i32)) != 0)) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 37 } else { 23 }; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:42675. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_array_reverse(mut ctx: *mut JSContext, mut this_val: JSValue, mut argc: i32, mut argv: *mut JSValue) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut obj: JSValue = core::mem::zeroed();
let mut lval: JSValue = core::mem::zeroed();
let mut hval: JSValue = core::mem::zeroed();
let mut arrp: *mut JSValue = core::mem::zeroed();
let mut len: i64 = core::mem::zeroed();
let mut l: i64 = core::mem::zeroed();
let mut h: i64 = core::mem::zeroed();
let mut l_present: i32 = core::mem::zeroed();
let mut h_present: i32 = core::mem::zeroed();
let mut count32: u32 = core::mem::zeroed();
let mut ll: u32 = core::mem::zeroed();
let mut hh: u32 = core::mem::zeroed();
let mut vm_block: usize = 43;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 42741
1 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 42740
2 => {
let _ = JS_FreeValue(ctx, obj);
vm_block = 1; continue;
}
// C line ? labels: exception
3 => {
let _ = JS_FreeValue(ctx, lval);
vm_block = 2; continue;
}
// C line 42736
4 => {
return obj;
}
// C line 42703
5 => {
vm_block = if ((((l) < (h)) as i32)) != 0 { 29 } else { 4 }; continue;
}
// C line ?
6 => {
let _ = { let _ = { let old = l; l = (l).wrapping_add(1); old }; { let old = h; h = (h).wrapping_sub(1); old } };
vm_block = 5; continue;
}
// C line 42719
7 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; lval = assigned; assigned };
vm_block = 6; continue;
}
// C line 42717
8 => {
vm_block = 3; continue;
}
// C line 42716
9 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; lval = assigned; assigned };
vm_block = 8; continue;
}
// C line 42715
10 => {
vm_block = if ((((JS_SetPropertyInt64(ctx, obj, h, lval)) < ((0 as i32))) as i32)) != 0 { 9 } else { 7 }; continue;
}
// C line 42722
11 => {
vm_block = 3; continue;
}
// C line 42721
12 => {
vm_block = if ((((JS_DeletePropertyInt64(ctx, obj, h, ((1 as i32)).wrapping_shl(((14 as i32)) as u32))) < ((0 as i32))) as i32)) != 0 { 11 } else { 6 }; continue;
}
// C line 42714
13 => {
vm_block = if (l_present) != 0 { 10 } else { 12 }; continue;
}
// C line 42712
14 => {
vm_block = 3; continue;
}
// C line 42711
15 => {
vm_block = if ((((JS_SetPropertyInt64(ctx, obj, l, hval)) < ((0 as i32))) as i32)) != 0 { 14 } else { 13 }; continue;
}
// C line 42732
16 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; lval = assigned; assigned };
vm_block = 6; continue;
}
// C line 42730
17 => {
vm_block = 3; continue;
}
// C line 42729
18 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; lval = assigned; assigned };
vm_block = 17; continue;
}
// C line 42728
19 => {
vm_block = if ((((JS_SetPropertyInt64(ctx, obj, h, lval)) < ((0 as i32))) as i32)) != 0 { 18 } else { 16 }; continue;
}
// C line 42727
20 => {
vm_block = 3; continue;
}
// C line 42726
21 => {
vm_block = if ((((JS_DeletePropertyInt64(ctx, obj, l, ((1 as i32)).wrapping_shl(((14 as i32)) as u32))) < ((0 as i32))) as i32)) != 0 { 20 } else { 19 }; continue;
}
// C line 42725
22 => {
vm_block = if (l_present) != 0 { 21 } else { 6 }; continue;
}
// C line 42710
23 => {
vm_block = if (h_present) != 0 { 15 } else { 22 }; continue;
}
// C line 42709
24 => {
vm_block = 3; continue;
}
// C line 42708
25 => {
vm_block = if ((((h_present) < ((0 as i32))) as i32)) != 0 { 24 } else { 23 }; continue;
}
// C line 42707
26 => {
let _ = { let assigned = JS_TryGetPropertyInt64(ctx, obj, h, core::ptr::addr_of_mut!(hval)); h_present = assigned; assigned };
vm_block = 25; continue;
}
// C line 42706
27 => {
vm_block = 3; continue;
}
// C line 42705
28 => {
vm_block = if ((((l_present) < ((0 as i32))) as i32)) != 0 { 27 } else { 26 }; continue;
}
// C line 42704
29 => {
let _ = { let assigned = JS_TryGetPropertyInt64(ctx, obj, l, core::ptr::addr_of_mut!(lval)); l_present = assigned; assigned };
vm_block = 28; continue;
}
// C line 42703
30 => {
let _ = { let _ = { let assigned = (((0 as i32)) as i64); l = assigned; assigned }; { let assigned = (len).wrapping_sub((((1 as i32)) as i64)); h = assigned; assigned } };
vm_block = 5; continue;
}
// C line 42700
31 => {
return obj;
}
// C line 42694
32 => {
vm_block = if ((((ll) < (hh)) as i32)) != 0 { 36 } else { 31 }; continue;
}
// C line ?
33 => {
let _ = { let _ = { let old = ll; ll = (ll).wrapping_add(1); old }; { let old = hh; hh = (hh).wrapping_sub(1); old } };
vm_block = 32; continue;
}
// C line 42697
34 => {
let _ = { let assigned = lval; *(arrp).offset((hh) as isize) = assigned; assigned };
vm_block = 33; continue;
}
// C line 42696
35 => {
let _ = { let assigned = *(arrp).offset((hh) as isize); *(arrp).offset((ll) as isize) = assigned; assigned };
vm_block = 34; continue;
}
// C line 42695
36 => {
let _ = { let assigned = *(arrp).offset((ll) as isize); lval = assigned; assigned };
vm_block = 35; continue;
}
// C line 42694
37 => {
let _ = { let _ = { let assigned = (((0 as i32)) as u32); ll = assigned; assigned }; { let assigned = (count32).wrapping_sub((((1 as i32)) as u32)); hh = assigned; assigned } };
vm_block = 32; continue;
}
// C line 42693
38 => {
vm_block = if ((((count32) > ((((1 as i32)) as u32))) as i32)) != 0 { 37 } else { 31 }; continue;
}
// C line 42690
39 => {
vm_block = if (((((js_get_fast_array(ctx, obj, core::ptr::addr_of_mut!(arrp), core::ptr::addr_of_mut!(count32))) != 0) && (((((((count32) as i64)) == (len)) as i32)) != 0)) as i32)) != 0 { 38 } else { 30 }; continue;
}
// C line 42687
40 => {
vm_block = 3; continue;
}
// C line 42686
41 => {
vm_block = if (js_get_length64(ctx, core::ptr::addr_of_mut!(len), obj)) != 0 { 40 } else { 39 }; continue;
}
// C line 42685
42 => {
let _ = { let assigned = JS_ToObject(ctx, this_val); obj = assigned; assigned };
vm_block = 41; continue;
}
// C line 42684
43 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; lval = assigned; assigned };
vm_block = 42; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:42748. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_array_toReversed(mut ctx: *mut JSContext, mut this_val: JSValue, mut argc: i32, mut argv: *mut JSValue) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut arr: JSValue = core::mem::zeroed();
let mut obj: JSValue = core::mem::zeroed();
let mut ret: JSValue = core::mem::zeroed();
let mut arrp: *mut JSValue = core::mem::zeroed();
let mut pval: *mut JSValue = core::mem::zeroed();
let mut p: *mut JSObject = core::mem::zeroed();
let mut i: i64 = core::mem::zeroed();
let mut len: i64 = core::mem::zeroed();
let mut count32: u32 = core::mem::zeroed();
let mut vm_block: usize = 25;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 42789
1 => {
return ret;
}
// C line 42788
2 => {
let _ = JS_FreeValue(ctx, obj);
vm_block = 1; continue;
}
// C line ? labels: exception
3 => {
let _ = JS_FreeValue(ctx, arr);
vm_block = 2; continue;
}
// C line 42784
4 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; arr = assigned; assigned };
vm_block = 3; continue;
}
// C line 42783
5 => {
let _ = { let assigned = arr; ret = assigned; assigned };
vm_block = 4; continue;
}
// C line 42772
6 => {
vm_block = if ((((i) >= ((((0 as i32)) as i64))) as i32)) != 0 { 8 } else { 5 }; continue;
}
// C line ?
7 => {
let _ = { let _ = { let old = i; i = (i).wrapping_sub(1); old }; { let old = pval; pval = (pval).offset(1); old } };
vm_block = 6; continue;
}
// C line 42773
8 => {
let _ = { let assigned = JS_DupValue(ctx, *(arrp).offset((i) as isize)); *(pval) = assigned; assigned };
vm_block = 7; continue;
}
// C line 42776
9 => {
vm_block = if ((((i) >= ((((0 as i32)) as i64))) as i32)) != 0 { 12 } else { 5 }; continue;
}
// C line ?
10 => {
let _ = { let _ = { let old = i; i = (i).wrapping_sub(1); old }; { let old = pval; pval = (pval).offset(1); old } };
vm_block = 9; continue;
}
// C line 42778
11 => {
vm_block = 3; continue;
}
// C line 42777
12 => {
vm_block = if ((((((1 as i32)).wrapping_neg()) == (JS_TryGetPropertyInt64(ctx, obj, i, pval))) as i32)) != 0 { 11 } else { 10 }; continue;
}
// C line 42771
13 => {
vm_block = if (((((js_get_fast_array(ctx, obj, core::ptr::addr_of_mut!(arrp), core::ptr::addr_of_mut!(count32))) != 0) && (((((((count32) as i64)) == (len)) as i32)) != 0)) as i32)) != 0 { 6 } else { 9 }; continue;
}
// C line 42770
14 => {
let _ = { let assigned = ((((*(p)).u).array).u).values; pval = assigned; assigned };
vm_block = 13; continue;
}
// C line 42769
15 => {
let _ = { let assigned = (len).wrapping_sub((((1 as i32)) as i64)); i = assigned; assigned };
vm_block = 14; continue;
}
// C line 42767
16 => {
let _ = { let assigned = ((((arr).u).ptr) as *mut JSObject); p = assigned; assigned };
vm_block = 15; continue;
}
// C line 42766
17 => {
vm_block = if ((((len) > ((((0 as i32)) as i64))) as i32)) != 0 { 16 } else { 5 }; continue;
}
// C line 42764
18 => {
vm_block = 3; continue;
}
// C line 42763
19 => {
vm_block = if (JS_IsException(arr)) != 0 { 18 } else { 17 }; continue;
}
// C line 42762
20 => {
let _ = { let assigned = js_allocate_fast_array(ctx, len); arr = assigned; assigned };
vm_block = 19; continue;
}
// C line 42760
21 => {
vm_block = 3; continue;
}
// C line 42759
22 => {
vm_block = if (js_get_length64(ctx, core::ptr::addr_of_mut!(len), obj)) != 0 { 21 } else { 20 }; continue;
}
// C line 42758
23 => {
let _ = { let assigned = JS_ToObject(ctx, this_val); obj = assigned; assigned };
vm_block = 22; continue;
}
// C line 42757
24 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; arr = assigned; assigned };
vm_block = 23; continue;
}
// C line 42756
25 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) }; ret = assigned; assigned };
vm_block = 24; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:42792. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_array_slice(mut ctx: *mut JSContext, mut this_val: JSValue, mut argc: i32, mut argv: *mut JSValue, mut splice: i32) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut obj: JSValue = core::mem::zeroed();
let mut arr: JSValue = core::mem::zeroed();
let mut val: JSValue = core::mem::zeroed();
let mut len_val: JSValue = core::mem::zeroed();
let mut len: i64 = core::mem::zeroed();
let mut start: i64 = core::mem::zeroed();
let mut k: i64 = core::mem::zeroed();
let mut v_final: i64 = core::mem::zeroed();
let mut n: i64 = core::mem::zeroed();
let mut count: i64 = core::mem::zeroed();
let mut del_count: i64 = core::mem::zeroed();
let mut new_len: i64 = core::mem::zeroed();
let mut kPresent: i32 = core::mem::zeroed();
let mut arrp: *mut JSValue = core::mem::zeroed();
let mut count32: u32 = core::mem::zeroed();
let mut i: u32 = core::mem::zeroed();
let mut item_count: u32 = core::mem::zeroed();
let mut vm_block: usize = 70;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 42896
1 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 42895
2 => {
let _ = JS_FreeValue(ctx, arr);
vm_block = 1; continue;
}
// C line ? labels: exception
3 => {
let _ = JS_FreeValue(ctx, obj);
vm_block = 2; continue;
}
// C line 42891
4 => {
return arr;
}
// C line 42890
5 => {
let _ = JS_FreeValue(ctx, obj);
vm_block = 4; continue;
}
// C line 42888
6 => {
vm_block = 3; continue;
}
// C line 42887
7 => {
vm_block = if ((((JS_SetProperty(ctx, obj, (((crate::quickjs_atom::JS_ATOM_length as i32)) as JSAtom), JS_NewInt64(ctx, new_len))) < ((0 as i32))) as i32)) != 0 { 6 } else { 5 }; continue;
}
// C line 42883
8 => {
vm_block = if ((((i) < (item_count)) as i32)) != 0 { 11 } else { 7 }; continue;
}
// C line ?
9 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 8; continue;
}
// C line 42885
10 => {
vm_block = 3; continue;
}
// C line 42884
11 => {
vm_block = if ((((JS_SetPropertyInt64(ctx, obj, (start).wrapping_add(((i) as i64)), JS_DupValue(ctx, *(argv).offset(((i).wrapping_add((((2 as i32)) as u32))) as isize)))) < ((0 as i32))) as i32)) != 0 { 10 } else { 9 }; continue;
}
// C line 42883
12 => {
let _ = { let assigned = (((0 as i32)) as u32); i = assigned; assigned };
vm_block = 8; continue;
}
// C line 42878
13 => {
vm_block = if (((({ let old = k; k = (k).wrapping_sub(1); old }) > (new_len)) as i32)) != 0 { 15 } else { 12 }; continue;
}
// C line 42880
14 => {
vm_block = 3; continue;
}
// C line 42879
15 => {
vm_block = if ((((JS_DeletePropertyInt64(ctx, obj, k, ((1 as i32)).wrapping_shl(((14 as i32)) as u32))) < ((0 as i32))) as i32)) != 0 { 14 } else { 13 }; continue;
}
// C line 42878
16 => {
let _ = { let assigned = len; k = assigned; assigned };
vm_block = 13; continue;
}
// C line 42876
17 => {
vm_block = 3; continue;
}
// C line 42873
18 => {
vm_block = if ((((JS_CopySubArray(ctx, obj, (start).wrapping_add(((item_count) as i64)), (start).wrapping_add(del_count), (len).wrapping_sub((start).wrapping_add(del_count)), if ((((((item_count) as i64)) <= (del_count)) as i32)) != 0 { (1 as i32) } else { ((1 as i32)).wrapping_neg() })) < ((0 as i32))) as i32)) != 0 { 17 } else { 16 }; continue;
}
// C line 42872
19 => {
vm_block = if ((((((item_count) as i64)) != (del_count)) as i32)) != 0 { 18 } else { 12 }; continue;
}
// C line 42871
20 => {
let _ = { let assigned = ((len).wrapping_add(((item_count) as i64))).wrapping_sub(del_count); new_len = assigned; assigned };
vm_block = 19; continue;
}
// C line 42870
21 => {
vm_block = if (splice) != 0 { 20 } else { 5 }; continue;
}
// C line 42868
22 => {
vm_block = 3; continue;
}
// C line 42867
23 => {
vm_block = if ((((JS_SetProperty(ctx, arr, (((crate::quickjs_atom::JS_ATOM_length as i32)) as JSAtom), JS_NewInt64(ctx, n))) < ((0 as i32))) as i32)) != 0 { 22 } else { 21 }; continue;
}
// C line 42858
24 => {
vm_block = if ((((k) < (v_final)) as i32)) != 0 { 31 } else { 23 }; continue;
}
// C line ?
25 => {
let _ = { let _ = { let old = k; k = (k).wrapping_add(1); old }; { let old = n; n = (n).wrapping_add(1); old } };
vm_block = 24; continue;
}
// C line 42864
26 => {
vm_block = 3; continue;
}
// C line 42863
27 => {
vm_block = if ((((JS_CreateDataPropertyUint32(ctx, arr, n, val, ((1 as i32)).wrapping_shl(((14 as i32)) as u32))) < ((0 as i32))) as i32)) != 0 { 26 } else { 25 }; continue;
}
// C line 42862
28 => {
vm_block = if (kPresent) != 0 { 27 } else { 25 }; continue;
}
// C line 42861
29 => {
vm_block = 3; continue;
}
// C line 42860
30 => {
vm_block = if ((((kPresent) < ((0 as i32))) as i32)) != 0 { 29 } else { 28 }; continue;
}
// C line 42859
31 => {
let _ = { let assigned = JS_TryGetPropertyInt64(ctx, obj, k, core::ptr::addr_of_mut!(val)); kPresent = assigned; assigned };
vm_block = 30; continue;
}
// C line 42852
32 => {
vm_block = if ((((((((k) < (v_final)) as i32)) != 0) && (((((k) < (((count32) as i64))) as i32)) != 0)) as i32)) != 0 { 35 } else { 24 }; continue;
}
// C line ?
33 => {
let _ = { let _ = { let old = k; k = (k).wrapping_add(1); old }; { let old = n; n = (n).wrapping_add(1); old } };
vm_block = 32; continue;
}
// C line 42854
34 => {
vm_block = 3; continue;
}
// C line 42853
35 => {
vm_block = if ((((JS_CreateDataPropertyUint32(ctx, arr, n, JS_DupValue(ctx, *(arrp).offset((k) as isize)), ((1 as i32)).wrapping_shl(((14 as i32)) as u32))) < ((0 as i32))) as i32)) != 0 { 34 } else { 33 }; continue;
}
// C line 42849
36 => {
vm_block = if (((((js_get_fast_array(ctx, obj, core::ptr::addr_of_mut!(arrp), core::ptr::addr_of_mut!(count32))) != 0) && ((js_is_fast_array(ctx, arr)) != 0)) as i32)) != 0 { 32 } else { 24 }; continue;
}
// C line 42844
37 => {
let _ = { let assigned = (((0 as i32)) as i64); n = assigned; assigned };
vm_block = 36; continue;
}
// C line 42843
38 => {
let _ = { let assigned = (start).wrapping_add(count); v_final = assigned; assigned };
vm_block = 37; continue;
}
// C line 42842
39 => {
let _ = { let assigned = start; k = assigned; assigned };
vm_block = 38; continue;
}
// C line 42840
40 => {
vm_block = 3; continue;
}
// C line 42839
41 => {
vm_block = if (JS_IsException(arr)) != 0 { 40 } else { 39 }; continue;
}
// C line 42838
42 => {
let _ = JS_FreeValue(ctx, len_val);
vm_block = 41; continue;
}
// C line 42837
43 => {
let _ = { let assigned = JS_ArraySpeciesCreate(ctx, obj, len_val); arr = assigned; assigned };
vm_block = 42; continue;
}
// C line 42836
44 => {
let _ = { let assigned = JS_NewInt64(ctx, count); len_val = assigned; assigned };
vm_block = 43; continue;
}
// C line 42826
45 => {
let _ = { let assigned = del_count; count = assigned; assigned };
vm_block = 44; continue;
}
// C line 42824
46 => {
vm_block = 3; continue;
}
// C line 42823
47 => {
let _ = JS_ThrowTypeError(ctx, c"Array loo long".as_ptr());
vm_block = 46; continue;
}
// C line 42822
48 => {
vm_block = if ((((((len).wrapping_add(((item_count) as i64))).wrapping_sub(del_count)) > ((((((1 as i32)) as i64)).wrapping_shl(((53 as i32)) as u32)).wrapping_sub((((1 as i32)) as i64)))) as i32)) != 0 { 47 } else { 45 }; continue;
}
// C line 42812
49 => {
let _ = { let assigned = (((0 as i32)) as i64); del_count = assigned; assigned };
vm_block = 48; continue;
}
// C line 42811
50 => {
let _ = { let assigned = (((0 as i32)) as u32); item_count = assigned; assigned };
vm_block = 49; continue;
}
// C line 42816
51 => {
let _ = { let assigned = (len).wrapping_sub(start); del_count = assigned; assigned };
vm_block = 48; continue;
}
// C line 42815
52 => {
let _ = { let assigned = (((0 as i32)) as u32); item_count = assigned; assigned };
vm_block = 51; continue;
}
// C line 42820
53 => {
vm_block = 3; continue;
}
// C line 42819
54 => {
vm_block = if (JS_ToInt64Clamp(ctx, core::ptr::addr_of_mut!(del_count), *(argv).offset(((1 as i32)) as isize), (((0 as i32)) as i64), (len).wrapping_sub(start), (((0 as i32)) as i64))) != 0 { 53 } else { 48 }; continue;
}
// C line 42818
55 => {
let _ = { let assigned = (((argc).wrapping_sub((2 as i32))) as u32); item_count = assigned; assigned };
vm_block = 54; continue;
}
// C line 42814
56 => {
vm_block = if ((((argc) == ((1 as i32))) as i32)) != 0 { 52 } else { 55 }; continue;
}
// C line 42810
57 => {
vm_block = if ((((argc) == ((0 as i32))) as i32)) != 0 { 50 } else { 56 }; continue;
}
// C line 42834
58 => {
let _ = { let assigned = crate::cutils_header::max_int64((v_final).wrapping_sub(start), (((0 as i32)) as i64)); count = assigned; assigned };
vm_block = 44; continue;
}
// C line 42832
59 => {
vm_block = 3; continue;
}
// C line 42831
60 => {
vm_block = if (JS_ToInt64Clamp(ctx, core::ptr::addr_of_mut!(v_final), *(argv).offset(((1 as i32)) as isize), (((0 as i32)) as i64), len, len)) != 0 { 59 } else { 58 }; continue;
}
// C line 42830
61 => {
vm_block = if ((!((JS_IsUndefined(*(argv).offset(((1 as i32)) as isize))) != 0) as i32)) != 0 { 60 } else { 58 }; continue;
}
// C line 42829
62 => {
let _ = { let assigned = len; v_final = assigned; assigned };
vm_block = 61; continue;
}
// C line 42828
63 => {
let _ = { let assigned = (((0 as i32)) as u32); item_count = assigned; assigned };
vm_block = 62; continue;
}
// C line 42809
64 => {
vm_block = if (splice) != 0 { 57 } else { 63 }; continue;
}
// C line 42807
65 => {
vm_block = 3; continue;
}
// C line 42806
66 => {
vm_block = if (JS_ToInt64Clamp(ctx, core::ptr::addr_of_mut!(start), *(argv).offset(((0 as i32)) as isize), (((0 as i32)) as i64), len, len)) != 0 { 65 } else { 64 }; continue;
}
// C line 42804
67 => {
vm_block = 3; continue;
}
// C line 42803
68 => {
vm_block = if (js_get_length64(ctx, core::ptr::addr_of_mut!(len), obj)) != 0 { 67 } else { 66 }; continue;
}
// C line 42802
69 => {
let _ = { let assigned = JS_ToObject(ctx, this_val); obj = assigned; assigned };
vm_block = 68; continue;
}
// C line 42801
70 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; arr = assigned; assigned };
vm_block = 69; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:42899. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_array_toSpliced(mut ctx: *mut JSContext, mut this_val: JSValue, mut argc: i32, mut argv: *mut JSValue) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut arr: JSValue = core::mem::zeroed();
let mut obj: JSValue = core::mem::zeroed();
let mut ret: JSValue = core::mem::zeroed();
let mut arrp: *mut JSValue = core::mem::zeroed();
let mut pval: *mut JSValue = core::mem::zeroed();
let mut last: *mut JSValue = core::mem::zeroed();
let mut p: *mut JSObject = core::mem::zeroed();
let mut i: i64 = core::mem::zeroed();
let mut j: i64 = core::mem::zeroed();
let mut len: i64 = core::mem::zeroed();
let mut newlen: i64 = core::mem::zeroed();
let mut start: i64 = core::mem::zeroed();
let mut add: i64 = core::mem::zeroed();
let mut del: i64 = core::mem::zeroed();
let mut count32: u32 = core::mem::zeroed();
let mut vm_block: usize = 65;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 42976
1 => {
return ret;
}
// C line 42975
2 => {
let _ = JS_FreeValue(ctx, obj);
vm_block = 1; continue;
}
// C line ? labels: exception
3 => {
let _ = JS_FreeValue(ctx, arr);
vm_block = 2; continue;
}
// C line 42971
4 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; arr = assigned; assigned };
vm_block = 3; continue;
}
// C line ? labels: done
5 => {
let _ = { let assigned = arr; ret = assigned; assigned };
vm_block = 4; continue;
}
// C line 42967
6 => {
let _ = if ((((!(((((pval) == (last)) as i32)) != 0) as i32)) as i64)) != 0 { std::process::abort() } else { { let _ = (0 as i32); } };
vm_block = 5; continue;
}
// C line 42954
7 => {
vm_block = if ((((i) < (len)) as i32)) != 0 { 9 } else { 6 }; continue;
}
// C line ?
8 => {
let _ = { let _ = { let old = i; i = (i).wrapping_add(1); old }; { let old = pval; pval = (pval).offset(1); old } };
vm_block = 7; continue;
}
// C line 42955
9 => {
let _ = { let assigned = JS_DupValue(ctx, *(arrp).offset((i) as isize)); *(pval) = assigned; assigned };
vm_block = 8; continue;
}
// C line 42954
10 => {
let _ = { i = (i).wrapping_add(del); i };
vm_block = 7; continue;
}
// C line 42952
11 => {
vm_block = if ((((j) < (add)) as i32)) != 0 { 13 } else { 10 }; continue;
}
// C line ?
12 => {
let _ = { let _ = { let old = j; j = (j).wrapping_add(1); old }; { let old = pval; pval = (pval).offset(1); old } };
vm_block = 11; continue;
}
// C line 42953
13 => {
let _ = { let assigned = JS_DupValue(ctx, *(argv).offset((((((2 as i32)) as i64)).wrapping_add(j)) as isize)); *(pval) = assigned; assigned };
vm_block = 12; continue;
}
// C line 42952
14 => {
let _ = { let assigned = (((0 as i32)) as i64); j = assigned; assigned };
vm_block = 11; continue;
}
// C line 42950
15 => {
vm_block = if ((((i) < (start)) as i32)) != 0 { 17 } else { 14 }; continue;
}
// C line ?
16 => {
let _ = { let _ = { let old = i; i = (i).wrapping_add(1); old }; { let old = pval; pval = (pval).offset(1); old } };
vm_block = 15; continue;
}
// C line 42951
17 => {
let _ = { let assigned = JS_DupValue(ctx, *(arrp).offset((i) as isize)); *(pval) = assigned; assigned };
vm_block = 16; continue;
}
// C line 42950
18 => {
let _ = { let assigned = (((0 as i32)) as i64); i = assigned; assigned };
vm_block = 15; continue;
}
// C line 42962
19 => {
vm_block = if ((((i) < (len)) as i32)) != 0 { 22 } else { 6 }; continue;
}
// C line ?
20 => {
let _ = { let _ = { let old = i; i = (i).wrapping_add(1); old }; { let old = pval; pval = (pval).offset(1); old } };
vm_block = 19; continue;
}
// C line 42964
21 => {
vm_block = 3; continue;
}
// C line 42963
22 => {
vm_block = if ((((((1 as i32)).wrapping_neg()) == (JS_TryGetPropertyInt64(ctx, obj, i, pval))) as i32)) != 0 { 21 } else { 20 }; continue;
}
// C line 42962
23 => {
let _ = { i = (i).wrapping_add(del); i };
vm_block = 19; continue;
}
// C line 42960
24 => {
vm_block = if ((((j) < (add)) as i32)) != 0 { 26 } else { 23 }; continue;
}
// C line ?
25 => {
let _ = { let _ = { let old = j; j = (j).wrapping_add(1); old }; { let old = pval; pval = (pval).offset(1); old } };
vm_block = 24; continue;
}
// C line 42961
26 => {
let _ = { let assigned = JS_DupValue(ctx, *(argv).offset((((((2 as i32)) as i64)).wrapping_add(j)) as isize)); *(pval) = assigned; assigned };
vm_block = 25; continue;
}
// C line 42960
27 => {
let _ = { let assigned = (((0 as i32)) as i64); j = assigned; assigned };
vm_block = 24; continue;
}
// C line 42957
28 => {
vm_block = if ((((i) < (start)) as i32)) != 0 { 31 } else { 27 }; continue;
}
// C line ?
29 => {
let _ = { let _ = { let old = i; i = (i).wrapping_add(1); old }; { let old = pval; pval = (pval).offset(1); old } };
vm_block = 28; continue;
}
// C line 42959
30 => {
vm_block = 3; continue;
}
// C line 42958
31 => {
vm_block = if ((((((1 as i32)).wrapping_neg()) == (JS_TryGetPropertyInt64(ctx, obj, i, pval))) as i32)) != 0 { 30 } else { 29 }; continue;
}
// C line 42957
32 => {
let _ = { let assigned = (((0 as i32)) as i64); i = assigned; assigned };
vm_block = 28; continue;
}
// C line 42949
33 => {
vm_block = if (((((js_get_fast_array(ctx, obj, core::ptr::addr_of_mut!(arrp), core::ptr::addr_of_mut!(count32))) != 0) && (((((((count32) as i64)) == (len)) as i32)) != 0)) as i32)) != 0 { 18 } else { 32 }; continue;
}
// C line 42947
34 => {
let _ = { let assigned = core::ptr::addr_of_mut!(*(((((*(p)).u).array).u).values).offset((newlen) as isize)); last = assigned; assigned };
vm_block = 33; continue;
}
// C line 42946
35 => {
let _ = { let assigned = core::ptr::addr_of_mut!(*(((((*(p)).u).array).u).values).offset(((0 as i32)) as isize)); pval = assigned; assigned };
vm_block = 34; continue;
}
// C line 42945
36 => {
let _ = { let assigned = ((((arr).u).ptr) as *mut JSObject); p = assigned; assigned };
vm_block = 35; continue;
}
// C line 42943
37 => {
vm_block = 5; continue;
}
// C line 42942
38 => {
vm_block = if ((((newlen) <= ((((0 as i32)) as i64))) as i32)) != 0 { 37 } else { 36 }; continue;
}
// C line 42940
39 => {
vm_block = 3; continue;
}
// C line 42939
40 => {
vm_block = if (JS_IsException(arr)) != 0 { 39 } else { 38 }; continue;
}
// C line 42938
41 => {
let _ = { let assigned = js_allocate_fast_array(ctx, newlen); arr = assigned; assigned };
vm_block = 40; continue;
}
// C line 42935
42 => {
vm_block = 3; continue;
}
// C line 42934
43 => {
let _ = JS_ThrowTypeError(ctx, c"invalid array length".as_ptr());
vm_block = 42; continue;
}
// C line 42933
44 => {
vm_block = if ((((newlen) > ((((((1 as i32)) as i64)).wrapping_shl(((53 as i32)) as u32)).wrapping_sub((((1 as i32)) as i64)))) as i32)) != 0 { 43 } else { 41 }; continue;
}
// C line 42932
45 => {
let _ = { let assigned = ((len).wrapping_add(add)).wrapping_sub(del); newlen = assigned; assigned };
vm_block = 44; continue;
}
// C line 42930
46 => {
let _ = { let assigned = (((argc).wrapping_sub((2 as i32))) as i64); add = assigned; assigned };
vm_block = 45; continue;
}
// C line 42929
47 => {
vm_block = if ((((argc) > ((2 as i32))) as i32)) != 0 { 46 } else { 45 }; continue;
}
// C line 42928
48 => {
let _ = { let assigned = (((0 as i32)) as i64); add = assigned; assigned };
vm_block = 47; continue;
}
// C line 42926
49 => {
vm_block = 3; continue;
}
// C line 42925
50 => {
vm_block = if (JS_ToInt64Clamp(ctx, core::ptr::addr_of_mut!(del), *(argv).offset(((1 as i32)) as isize), (((0 as i32)) as i64), del, (((0 as i32)) as i64))) != 0 { 49 } else { 48 }; continue;
}
// C line 42924
51 => {
vm_block = if ((((argc) > ((1 as i32))) as i32)) != 0 { 50 } else { 48 }; continue;
}
// C line 42923
52 => {
let _ = { let assigned = (len).wrapping_sub(start); del = assigned; assigned };
vm_block = 51; continue;
}
// C line 42922
53 => {
vm_block = if ((((argc) > ((0 as i32))) as i32)) != 0 { 52 } else { 51 }; continue;
}
// C line 42921
54 => {
let _ = { let assigned = (((0 as i32)) as i64); del = assigned; assigned };
vm_block = 53; continue;
}
// C line 42919
55 => {
vm_block = 3; continue;
}
// C line 42918
56 => {
vm_block = if (JS_ToInt64Clamp(ctx, core::ptr::addr_of_mut!(start), *(argv).offset(((0 as i32)) as isize), (((0 as i32)) as i64), len, len)) != 0 { 55 } else { 54 }; continue;
}
// C line 42917
57 => {
vm_block = if ((((argc) > ((0 as i32))) as i32)) != 0 { 56 } else { 54 }; continue;
}
// C line 42916
58 => {
let _ = { let assigned = (((0 as i32)) as i64); start = assigned; assigned };
vm_block = 57; continue;
}
// C line 42914
59 => {
vm_block = 3; continue;
}
// C line 42913
60 => {
vm_block = if (js_get_length64(ctx, core::ptr::addr_of_mut!(len), obj)) != 0 { 59 } else { 58 }; continue;
}
// C line 42912
61 => {
let _ = { let assigned = JS_ToObject(ctx, this_val); obj = assigned; assigned };
vm_block = 60; continue;
}
// C line 42910
62 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; arr = assigned; assigned };
vm_block = 61; continue;
}
// C line 42909
63 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) }; ret = assigned; assigned };
vm_block = 62; continue;
}
// C line 42908
64 => {
let _ = { let assigned = core::ptr::null_mut::<JSValue>(); last = assigned; assigned };
vm_block = 63; continue;
}
// C line 42907
65 => {
let _ = { let assigned = core::ptr::null_mut::<JSValue>(); pval = assigned; assigned };
vm_block = 64; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:42979. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_array_copyWithin(mut ctx: *mut JSContext, mut this_val: JSValue, mut argc: i32, mut argv: *mut JSValue) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut obj: JSValue = core::mem::zeroed();
let mut len: i64 = core::mem::zeroed();
let mut from: i64 = core::mem::zeroed();
let mut to: i64 = core::mem::zeroed();
let mut v_final: i64 = core::mem::zeroed();
let mut count: i64 = core::mem::zeroed();
let mut vm_block: usize = 17;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 43011
1 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line ? labels: exception
2 => {
let _ = JS_FreeValue(ctx, obj);
vm_block = 1; continue;
}
// C line 43007
3 => {
return obj;
}
// C line 43005
4 => {
vm_block = 2; continue;
}
// C line 43003
5 => {
vm_block = if (JS_CopySubArray(ctx, obj, to, from, count, if ((((((((from) < (to)) as i32)) != 0) && (((((to) < ((from).wrapping_add(count))) as i32)) != 0)) as i32)) != 0 { ((1 as i32)).wrapping_neg() } else { (1 as i32) })) != 0 { 4 } else { 3 }; continue;
}
// C line 43001
6 => {
let _ = { let assigned = crate::cutils_header::min_int64((v_final).wrapping_sub(from), (len).wrapping_sub(to)); count = assigned; assigned };
vm_block = 5; continue;
}
// C line 42998
7 => {
vm_block = 2; continue;
}
// C line 42997
8 => {
vm_block = if (JS_ToInt64Clamp(ctx, core::ptr::addr_of_mut!(v_final), *(argv).offset(((2 as i32)) as isize), (((0 as i32)) as i64), len, len)) != 0 { 7 } else { 6 }; continue;
}
// C line 42996
9 => {
vm_block = if ((((((((argc) > ((2 as i32))) as i32)) != 0) && (((!((JS_IsUndefined(*(argv).offset(((2 as i32)) as isize))) != 0) as i32)) != 0)) as i32)) != 0 { 8 } else { 6 }; continue;
}
// C line 42995
10 => {
let _ = { let assigned = len; v_final = assigned; assigned };
vm_block = 9; continue;
}
// C line 42993
11 => {
vm_block = 2; continue;
}
// C line 42992
12 => {
vm_block = if (JS_ToInt64Clamp(ctx, core::ptr::addr_of_mut!(from), *(argv).offset(((1 as i32)) as isize), (((0 as i32)) as i64), len, len)) != 0 { 11 } else { 10 }; continue;
}
// C line 42990
13 => {
vm_block = 2; continue;
}
// C line 42989
14 => {
vm_block = if (JS_ToInt64Clamp(ctx, core::ptr::addr_of_mut!(to), *(argv).offset(((0 as i32)) as isize), (((0 as i32)) as i64), len, len)) != 0 { 13 } else { 12 }; continue;
}
// C line 42987
15 => {
vm_block = 2; continue;
}
// C line 42986
16 => {
vm_block = if (js_get_length64(ctx, core::ptr::addr_of_mut!(len), obj)) != 0 { 15 } else { 14 }; continue;
}
// C line 42985
17 => {
let _ = { let assigned = JS_ToObject(ctx, this_val); obj = assigned; assigned };
vm_block = 16; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:43014. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn JS_FlattenIntoArray(mut ctx: *mut JSContext, mut target: JSValue, mut source: JSValue, mut sourceLen: i64, mut targetIndex: i64, mut depth: i32, mut mapperFunction: JSValue, mut thisArg: JSValue) -> i64 {
let mut vm_local_storage = Vec::<u64>::new();
let mut element: JSValue = core::mem::zeroed();
let mut sourceIndex: i64 = core::mem::zeroed();
let mut elementLen: i64 = core::mem::zeroed();
let mut present: i32 = core::mem::zeroed();
let mut is_array: i32 = core::mem::zeroed();
let mut args: [JSValue; 3] = core::mem::zeroed();
let mut vm_block: usize = 39;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 43073
1 => {
return ((((1 as i32)).wrapping_neg()) as i64);
}
// C line ? labels: fail
2 => {
let _ = JS_FreeValue(ctx, element);
vm_block = 1; continue;
}
// C line 43069
3 => {
return targetIndex;
}
// C line 43029
4 => {
vm_block = if ((((sourceIndex) < (sourceLen)) as i32)) != 0 { 35 } else { 3 }; continue;
}
// C line ?
5 => {
let _ = { let old = sourceIndex; sourceIndex = (sourceIndex).wrapping_add(1); old };
vm_block = 4; continue;
}
// C line 43067
6 => {
let _ = { let old = targetIndex; targetIndex = (targetIndex).wrapping_add(1); old };
vm_block = 5; continue;
}
// C line 43066
7 => {
return ((((1 as i32)).wrapping_neg()) as i64);
}
// C line 43064
8 => {
vm_block = if ((((JS_DefinePropertyValueInt64(ctx, target, targetIndex, element, ((((((((1 as i32)).wrapping_shl(((0 as i32)) as u32)) | (((1 as i32)).wrapping_shl(((1 as i32)) as u32)))) | (((1 as i32)).wrapping_shl(((2 as i32)) as u32)))) | (((1 as i32)).wrapping_shl(((14 as i32)) as u32))))) < ((0 as i32))) as i32)) != 0 { 7 } else { 6 }; continue;
}
// C line 43062
9 => {
vm_block = 2; continue;
}
// C line 43061
10 => {
let _ = JS_ThrowTypeError(ctx, c"Array too long".as_ptr());
vm_block = 9; continue;
}
// C line 43060
11 => {
vm_block = if ((((targetIndex) >= ((((((1 as i32)) as i64)).wrapping_shl(((53 as i32)) as u32)).wrapping_sub((((1 as i32)) as i64)))) as i32)) != 0 { 10 } else { 8 }; continue;
}
// C line 43057
12 => {
vm_block = 5; continue;
}
// C line 43056
13 => {
let _ = JS_FreeValue(ctx, element);
vm_block = 12; continue;
}
// C line 43055
14 => {
vm_block = 2; continue;
}
// C line 43054
15 => {
vm_block = if ((((targetIndex) < ((((0 as i32)) as i64))) as i32)) != 0 { 14 } else { 13 }; continue;
}
// C line 43050
16 => {
let _ = { let assigned = JS_FlattenIntoArray(ctx, target, element, elementLen, targetIndex, (depth).wrapping_sub((1 as i32)), JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }, JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }); targetIndex = assigned; assigned };
vm_block = 15; continue;
}
// C line 43049
17 => {
vm_block = 2; continue;
}
// C line 43048
18 => {
vm_block = if ((((js_get_length64(ctx, core::ptr::addr_of_mut!(elementLen), element)) < ((0 as i32))) as i32)) != 0 { 17 } else { 16 }; continue;
}
// C line 43047
19 => {
vm_block = if (is_array) != 0 { 18 } else { 11 }; continue;
}
// C line 43046
20 => {
vm_block = 2; continue;
}
// C line 43045
21 => {
vm_block = if ((((is_array) < ((0 as i32))) as i32)) != 0 { 20 } else { 19 }; continue;
}
// C line 43044
22 => {
let _ = { let assigned = JS_IsArray(ctx, element); is_array = assigned; assigned };
vm_block = 21; continue;
}
// C line 43043
23 => {
vm_block = if ((((depth) > ((0 as i32))) as i32)) != 0 { 22 } else { 11 }; continue;
}
// C line 43041
24 => {
return ((((1 as i32)).wrapping_neg()) as i64);
}
// C line 43040
25 => {
vm_block = if (JS_IsException(element)) != 0 { 24 } else { 23 }; continue;
}
// C line 43039
26 => {
let _ = JS_FreeValue(ctx, *((args).as_mut_ptr()).offset(((1 as i32)) as isize));
vm_block = 25; continue;
}
// C line 43038
27 => {
let _ = JS_FreeValue(ctx, *((args).as_mut_ptr()).offset(((0 as i32)) as isize));
vm_block = 26; continue;
}
// C line 43037
28 => {
let _ = { let assigned = JS_Call(ctx, mapperFunction, thisArg, (3 as i32), (args).as_mut_ptr()); element = assigned; assigned };
vm_block = 27; continue;
}
// C line 43036
29 => {
args = [element, JS_NewInt64(ctx, sourceIndex), source];
vm_block = 28; continue;
}
// C line 43035
30 => {
vm_block = if ((!((JS_IsUndefined(mapperFunction)) != 0) as i32)) != 0 { 29 } else { 23 }; continue;
}
// C line 43034
31 => {
vm_block = 5; continue;
}
// C line 43033
32 => {
vm_block = if ((!((present) != 0) as i32)) != 0 { 31 } else { 30 }; continue;
}
// C line 43032
33 => {
return ((((1 as i32)).wrapping_neg()) as i64);
}
// C line 43031
34 => {
vm_block = if ((((present) < ((0 as i32))) as i32)) != 0 { 33 } else { 32 }; continue;
}
// C line 43030
35 => {
let _ = { let assigned = JS_TryGetPropertyInt64(ctx, source, sourceIndex, core::ptr::addr_of_mut!(element)); present = assigned; assigned };
vm_block = 34; continue;
}
// C line 43029
36 => {
let _ = { let assigned = (((0 as i32)) as i64); sourceIndex = assigned; assigned };
vm_block = 4; continue;
}
// C line 43026
37 => {
return ((((1 as i32)).wrapping_neg()) as i64);
}
// C line 43025
38 => {
let _ = JS_ThrowStackOverflow(ctx);
vm_block = 37; continue;
}
// C line 43024
39 => {
vm_block = if (js_check_stack_overflow((*(ctx)).rt, (((0 as i32)) as usize))) != 0 { 38 } else { 36 }; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:43076. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_array_flatten(mut ctx: *mut JSContext, mut this_val: JSValue, mut argc: i32, mut argv: *mut JSValue, mut map: i32) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut obj: JSValue = core::mem::zeroed();
let mut arr: JSValue = core::mem::zeroed();
let mut mapperFunction: JSValue = core::mem::zeroed();
let mut thisArg: JSValue = core::mem::zeroed();
let mut sourceLen: i64 = core::mem::zeroed();
let mut depthNum: i32 = core::mem::zeroed();
let mut vm_block: usize = 26;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 43117
1 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 43116
2 => {
let _ = JS_FreeValue(ctx, arr);
vm_block = 1; continue;
}
// C line ? labels: exception
3 => {
let _ = JS_FreeValue(ctx, obj);
vm_block = 2; continue;
}
// C line 43112
4 => {
return arr;
}
// C line 43111
5 => {
let _ = JS_FreeValue(ctx, obj);
vm_block = 4; continue;
}
// C line 43110
6 => {
vm_block = 3; continue;
}
// C line 43108
7 => {
vm_block = if ((((JS_FlattenIntoArray(ctx, arr, obj, sourceLen, (((0 as i32)) as i64), depthNum, mapperFunction, thisArg)) < ((((0 as i32)) as i64))) as i32)) != 0 { 6 } else { 5 }; continue;
}
// C line 43107
8 => {
vm_block = 3; continue;
}
// C line 43106
9 => {
vm_block = if (JS_IsException(arr)) != 0 { 8 } else { 7 }; continue;
}
// C line 43105
10 => {
let _ = { let assigned = JS_ArraySpeciesCreate(ctx, obj, JS_NewInt32(ctx, (0 as i32))); arr = assigned; assigned };
vm_block = 9; continue;
}
// C line 43098
11 => {
vm_block = 3; continue;
}
// C line 43097
12 => {
vm_block = if (check_function(ctx, mapperFunction)) != 0 { 11 } else { 10 }; continue;
}
// C line 43095
13 => {
let _ = { let assigned = *(argv).offset(((1 as i32)) as isize); thisArg = assigned; assigned };
vm_block = 12; continue;
}
// C line 43094
14 => {
vm_block = if ((((argc) > ((1 as i32))) as i32)) != 0 { 13 } else { 12 }; continue;
}
// C line 43093
15 => {
let _ = { let assigned = *(argv).offset(((0 as i32)) as isize); mapperFunction = assigned; assigned };
vm_block = 14; continue;
}
// C line 43102
16 => {
vm_block = 3; continue;
}
// C line 43101
17 => {
vm_block = if ((((JS_ToInt32Sat(ctx, core::ptr::addr_of_mut!(depthNum), *(argv).offset(((0 as i32)) as isize))) < ((0 as i32))) as i32)) != 0 { 16 } else { 10 }; continue;
}
// C line 43100
18 => {
vm_block = if ((((((((argc) > ((0 as i32))) as i32)) != 0) && (((!((JS_IsUndefined(*(argv).offset(((0 as i32)) as isize))) != 0) as i32)) != 0)) as i32)) != 0 { 17 } else { 10 }; continue;
}
// C line 43092
19 => {
vm_block = if (map) != 0 { 15 } else { 18 }; continue;
}
// C line 43091
20 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; thisArg = assigned; assigned };
vm_block = 19; continue;
}
// C line 43090
21 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; mapperFunction = assigned; assigned };
vm_block = 20; continue;
}
// C line 43089
22 => {
let _ = { let assigned = (1 as i32); depthNum = assigned; assigned };
vm_block = 21; continue;
}
// C line 43087
23 => {
vm_block = 3; continue;
}
// C line 43086
24 => {
vm_block = if (js_get_length64(ctx, core::ptr::addr_of_mut!(sourceLen), obj)) != 0 { 23 } else { 22 }; continue;
}
// C line 43085
25 => {
let _ = { let assigned = JS_ToObject(ctx, this_val); obj = assigned; assigned };
vm_block = 24; continue;
}
// C line 43084
26 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; arr = assigned; assigned };
vm_block = 25; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:43135. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_array_cmp_generic(mut a: *const c_void, mut b: *const c_void, mut opaque: *mut c_void) -> i32 {
let mut vm_local_storage = Vec::<u64>::new();
let mut psc: *mut array_sort_context = core::mem::zeroed();
let mut ctx: *mut JSContext = core::mem::zeroed();
let mut argv: [JSValue; 2] = core::mem::zeroed();
let mut res: JSValue = core::mem::zeroed();
let mut ap: *mut ValueSlot = core::mem::zeroed();
let mut bp: *mut ValueSlot = core::mem::zeroed();
let mut cmp: i32 = core::mem::zeroed();
let mut val: i32 = core::mem::zeroed();
let mut val_1: f64 = core::mem::zeroed();
let mut str: JSValue = core::mem::zeroed();
let mut str_1: JSValue = core::mem::zeroed();
let mut vm_block: usize = 36;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 43193
1 => {
return (0 as i32);
}
// C line ? labels: exception
2 => {
let _ = { let assigned = (1 as i32); (*(psc)).exception = assigned; assigned };
vm_block = 1; continue;
}
// C line ? labels: cmp_same
3 => {
return (((((*(ap)).pos) > ((*(bp)).pos)) as i32)).wrapping_sub(((((*(ap)).pos) < ((*(bp)).pos)) as i32));
}
// C line 43186
4 => {
return cmp;
}
// C line 43185
5 => {
vm_block = if ((((cmp) != ((0 as i32))) as i32)) != 0 { 4 } else { 3 }; continue;
}
// C line 43160
6 => {
let _ = { let assigned = ((((val) > ((0 as i32))) as i32)).wrapping_sub((((val) < ((0 as i32))) as i32)); cmp = assigned; assigned };
vm_block = 5; continue;
}
// C line 43159
7 => {
val = ((((res).u).uint64) as i32);
vm_block = 6; continue;
}
// C line 43165
8 => {
let _ = { let assigned = ((((val_1) > ((((0 as i32)) as f64))) as i32)).wrapping_sub((((val_1) < ((((0 as i32)) as f64))) as i32)); cmp = assigned; assigned };
vm_block = 5; continue;
}
// C line 43164
9 => {
vm_block = 2; continue;
}
// C line 43163
10 => {
vm_block = if ((((JS_ToFloat64Free(ctx, core::ptr::addr_of_mut!(val_1), res)) < ((0 as i32))) as i32)) != 0 { 9 } else { 8 }; continue;
}
// C line 43158
11 => {
vm_block = if (((((((res).tag) as i32)) == ((JS_TAG_INT as i32))) as i32)) != 0 { 7 } else { 10 }; continue;
}
// C line 43157
12 => {
vm_block = 2; continue;
}
// C line 43156
13 => {
vm_block = if (JS_IsException(res)) != 0 { 12 } else { 11 }; continue;
}
// C line 43155
14 => {
let _ = { let assigned = JS_Call(ctx, (*(psc)).method, JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }, (2 as i32), (argv).as_mut_ptr()); res = assigned; assigned };
vm_block = 13; continue;
}
// C line 43154
15 => {
let _ = { let assigned = (*(bp)).val; *((argv).as_mut_ptr()).offset(((1 as i32)) as isize) = assigned; assigned };
vm_block = 14; continue;
}
// C line 43153
16 => {
let _ = { let assigned = (*(ap)).val; *((argv).as_mut_ptr()).offset(((0 as i32)) as isize) = assigned; assigned };
vm_block = 15; continue;
}
// C line 43152
17 => {
vm_block = 3; continue;
}
// C line 43151
18 => {
vm_block = if ((!((js_array_memcmp(((core::ptr::addr_of_mut!((*(ap)).val)) as *const c_void), ((core::ptr::addr_of_mut!((*(bp)).val)) as *const c_void), (size_of::<JSValue>() as usize))) != 0) as i32)) != 0 { 17 } else { 16 }; continue;
}
// C line 43183
19 => {
let _ = { let assigned = js_string_compare(ctx, (*(ap)).str, (*(bp)).str); cmp = assigned; assigned };
vm_block = 5; continue;
}
// C line 43181
20 => {
let _ = { let assigned = ((((str_1).u).ptr) as *mut JSString); (*(bp)).str = assigned; assigned };
vm_block = 19; continue;
}
// C line 43180
21 => {
vm_block = 2; continue;
}
// C line 43179
22 => {
vm_block = if (JS_IsException(str_1)) != 0 { 21 } else { 20 }; continue;
}
// C line 43178
23 => {
str_1 = JS_ToString(ctx, (*(bp)).val);
vm_block = 22; continue;
}
// C line 43177
24 => {
vm_block = if ((!(!((*(bp)).str).is_null()) as i32)) != 0 { 23 } else { 19 }; continue;
}
// C line 43175
25 => {
let _ = { let assigned = ((((str).u).ptr) as *mut JSString); (*(ap)).str = assigned; assigned };
vm_block = 24; continue;
}
// C line 43174
26 => {
vm_block = 2; continue;
}
// C line 43173
27 => {
vm_block = if (JS_IsException(str)) != 0 { 26 } else { 25 }; continue;
}
// C line 43172
28 => {
str = JS_ToString(ctx, (*(ap)).val);
vm_block = 27; continue;
}
// C line 43171
29 => {
vm_block = if ((!(!((*(ap)).str).is_null()) as i32)) != 0 { 28 } else { 24 }; continue;
}
// C line 43147
30 => {
vm_block = if ((*(psc)).has_method) != 0 { 18 } else { 29 }; continue;
}
// C line 43145
31 => {
return (0 as i32);
}
// C line 43144
32 => {
vm_block = if ((*(psc)).exception) != 0 { 31 } else { 30 }; continue;
}
// C line 43141
33 => {
bp = ((((b) as *mut c_void)) as *mut ValueSlot);
vm_block = 32; continue;
}
// C line 43140
34 => {
ap = ((((a) as *mut c_void)) as *mut ValueSlot);
vm_block = 33; continue;
}
// C line 43137
35 => {
ctx = (*(psc)).ctx;
vm_block = 34; continue;
}
// C line 43136
36 => {
psc = ((opaque) as *mut array_sort_context);
vm_block = 35; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:43196. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_array_sort(mut ctx: *mut JSContext, mut this_val: JSValue, mut argc: i32, mut argv: *mut JSValue) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut asc: array_sort_context = core::mem::zeroed();
let mut obj: JSValue = core::mem::zeroed();
let mut array: *mut ValueSlot = core::mem::zeroed();
let mut array_size: usize = core::mem::zeroed();
let mut pos: usize = core::mem::zeroed();
let mut n: usize = core::mem::zeroed();
let mut i: i64 = core::mem::zeroed();
let mut len: i64 = core::mem::zeroed();
let mut undefined_count: i64 = core::mem::zeroed();
let mut present: i32 = core::mem::zeroed();
let mut new_size: usize = core::mem::zeroed();
let mut slack: usize = core::mem::zeroed();
let mut new_array: *mut ValueSlot = core::mem::zeroed();
let mut vm_block: usize = 65;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 43279
1 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line ? labels: fail
2 => {
let _ = JS_FreeValue(ctx, obj);
vm_block = 1; continue;
}
// C line 43276
3 => {
let _ = js_free(ctx, ((array) as *mut c_void));
vm_block = 2; continue;
}
// C line 43271 labels: exception
4 => {
vm_block = if ((((n) < (pos)) as i32)) != 0 { 8 } else { 3 }; continue;
}
// C line ?
5 => {
let _ = { let old = n; n = (n).wrapping_add(1); old };
vm_block = 4; continue;
}
// C line 43274
6 => {
let _ = JS_FreeValue(ctx, JSValue { u: JSValueUnion { ptr: (((*(array).offset((n) as isize)).str) as *mut c_void) }, tag: (((JS_TAG_STRING as i32)) as i64) });
vm_block = 5; continue;
}
// C line 43273
7 => {
vm_block = if !((*(array).offset((n) as isize)).str).is_null() { 6 } else { 5 }; continue;
}
// C line 43272
8 => {
let _ = JS_FreeValue(ctx, (*(array).offset((n) as isize)).val);
vm_block = 7; continue;
}
// C line 43268
9 => {
return obj;
}
// C line 43264
10 => {
vm_block = if ((((i) < (len)) as i32)) != 0 { 13 } else { 9 }; continue;
}
// C line ?
11 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 10; continue;
}
// C line 43266
12 => {
vm_block = 2; continue;
}
// C line 43265
13 => {
vm_block = if ((((JS_DeletePropertyInt64(ctx, obj, i, ((1 as i32)).wrapping_shl(((14 as i32)) as u32))) < ((0 as i32))) as i32)) != 0 { 12 } else { 11 }; continue;
}
// C line 43260
14 => {
vm_block = if (((({ let old = undefined_count; undefined_count = (undefined_count).wrapping_sub(1); old }) > ((((0 as i32)) as i64))) as i32)) != 0 { 17 } else { 10 }; continue;
}
// C line ?
15 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 14; continue;
}
// C line 43262
16 => {
vm_block = 2; continue;
}
// C line 43261
17 => {
vm_block = if ((((JS_SetPropertyInt64(ctx, obj, i, JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) })) < ((0 as i32))) as i32)) != 0 { 16 } else { 15 }; continue;
}
// C line 43260
18 => {
let _ = { let assigned = ((n) as i64); i = assigned; assigned };
vm_block = 14; continue;
}
// C line 43259
19 => {
let _ = js_free(ctx, ((array) as *mut c_void));
vm_block = 18; continue;
}
// C line 43246
20 => {
vm_block = if ((((n) < (pos)) as i32)) != 0 { 28 } else { 19 }; continue;
}
// C line 43257
21 => {
let _ = { let old = n; n = (n).wrapping_add(1); old };
vm_block = 20; continue;
}
// C line 43250
22 => {
let _ = JS_FreeValue(ctx, (*(array).offset((n) as isize)).val);
vm_block = 21; continue;
}
// C line 43254
23 => {
vm_block = 4; continue;
}
// C line 43253
24 => {
let _ = { let old = n; n = (n).wrapping_add(1); old };
vm_block = 23; continue;
}
// C line 43252
25 => {
vm_block = if ((((JS_SetPropertyInt64(ctx, obj, ((n) as i64), (*(array).offset((n) as isize)).val)) < ((0 as i32))) as i32)) != 0 { 24 } else { 21 }; continue;
}
// C line 43249
26 => {
vm_block = if (((((((*(array).offset((n) as isize)).pos) as u64)) == (((n) as u64))) as i32)) != 0 { 22 } else { 25 }; continue;
}
// C line 43248
27 => {
let _ = JS_FreeValue(ctx, JSValue { u: JSValueUnion { ptr: (((*(array).offset((n) as isize)).str) as *mut c_void) }, tag: (((JS_TAG_STRING as i32)) as i64) });
vm_block = 26; continue;
}
// C line 43247
28 => {
vm_block = if !((*(array).offset((n) as isize)).str).is_null() { 27 } else { 26 }; continue;
}
// C line 43243
29 => {
vm_block = 4; continue;
}
// C line 43242
30 => {
vm_block = if ((asc).exception) != 0 { 29 } else { 20 }; continue;
}
// C line 43241
31 => {
let _ = crate::cutils::rqsort(((array) as *mut c_void), pos, (size_of::<ValueSlot>() as usize), js_array_cmp_generic, ((core::ptr::addr_of_mut!(asc)) as *mut c_void));
vm_block = 30; continue;
}
// C line 43216
32 => {
vm_block = if ((((i) < (len)) as i32)) != 0 { 52 } else { 31 }; continue;
}
// C line ?
33 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 32; continue;
}
// C line 43239
34 => {
let _ = { let old = pos; pos = (pos).wrapping_add(1); old };
vm_block = 33; continue;
}
// C line 43238
35 => {
let _ = { let assigned = i; (*(array).offset((pos) as isize)).pos = assigned; assigned };
vm_block = 34; continue;
}
// C line 43237
36 => {
let _ = { let assigned = core::ptr::null_mut::<JSString>(); (*(array).offset((pos) as isize)).str = assigned; assigned };
vm_block = 35; continue;
}
// C line 43235
37 => {
vm_block = 33; continue;
}
// C line 43234
38 => {
let _ = { let old = undefined_count; undefined_count = (undefined_count).wrapping_add(1); old };
vm_block = 37; continue;
}
// C line 43233
39 => {
vm_block = if (JS_IsUndefined((*(array).offset((pos) as isize)).val)) != 0 { 38 } else { 36 }; continue;
}
// C line 43232
40 => {
vm_block = 33; continue;
}
// C line 43231
41 => {
vm_block = if ((((present) == ((0 as i32))) as i32)) != 0 { 40 } else { 39 }; continue;
}
// C line 43230
42 => {
vm_block = 4; continue;
}
// C line 43229
43 => {
vm_block = if ((((present) < ((0 as i32))) as i32)) != 0 { 42 } else { 41 }; continue;
}
// C line 43228
44 => {
let _ = { let assigned = JS_TryGetPropertyInt64(ctx, obj, i, core::ptr::addr_of_mut!((*(array).offset((pos) as isize)).val)); present = assigned; assigned };
vm_block = 43; continue;
}
// C line 43226
45 => {
let _ = { let assigned = new_size; array_size = assigned; assigned };
vm_block = 44; continue;
}
// C line 43225
46 => {
let _ = { let assigned = new_array; array = assigned; assigned };
vm_block = 45; continue;
}
// C line 43224
47 => {
let _ = { new_size = (new_size).wrapping_add(((slack) / ((size_of::<ValueSlot>() as usize)))); new_size };
vm_block = 46; continue;
}
// C line 43223
48 => {
vm_block = 4; continue;
}
// C line 43222
49 => {
vm_block = if ((((new_array) == (core::ptr::null_mut::<ValueSlot>())) as i32)) != 0 { 48 } else { 47 }; continue;
}
// C line 43221
50 => {
let _ = { let assigned = ((js_realloc2(ctx, ((array) as *mut c_void), (new_size).wrapping_mul((size_of::<ValueSlot>() as usize)), core::ptr::addr_of_mut!(slack))) as *mut ValueSlot); new_array = assigned; assigned };
vm_block = 49; continue;
}
// C line 43220
51 => {
let _ = { let assigned = ((((array_size).wrapping_add((array_size).wrapping_shr(((1 as i32)) as u32))).wrapping_add((((31 as i32)) as usize))) & ((((!((15 as i32)))) as usize))); new_size = assigned; assigned };
vm_block = 50; continue;
}
// C line 43217
52 => {
vm_block = if ((((pos) >= (array_size)) as i32)) != 0 { 51 } else { 44 }; continue;
}
// C line 43216
53 => {
let _ = { let assigned = (((0 as i32)) as i64); i = assigned; assigned };
vm_block = 32; continue;
}
// C line 43213
54 => {
vm_block = 4; continue;
}
// C line 43212
55 => {
vm_block = if (js_get_length64(ctx, core::ptr::addr_of_mut!(len), obj)) != 0 { 54 } else { 53 }; continue;
}
// C line 43211
56 => {
let _ = { let assigned = JS_ToObject(ctx, this_val); obj = assigned; assigned };
vm_block = 55; continue;
}
// C line 43209
57 => {
let _ = { let assigned = (1 as i32); (asc).has_method = assigned; assigned };
vm_block = 56; continue;
}
// C line 43208
58 => {
vm_block = 4; continue;
}
// C line 43207
59 => {
vm_block = if (check_function(ctx, (asc).method)) != 0 { 58 } else { 57 }; continue;
}
// C line 43206
60 => {
vm_block = if ((!((JS_IsUndefined((asc).method)) != 0) as i32)) != 0 { 59 } else { 56 }; continue;
}
// C line 43203
61 => {
undefined_count = (((0 as i32)) as i64);
vm_block = 60; continue;
}
// C line 43202
62 => {
array_size = (((0 as i32)) as usize);
pos = (((0 as i32)) as usize);
n = (((0 as i32)) as usize);
vm_block = 61; continue;
}
// C line 43201
63 => {
array = core::ptr::null_mut::<ValueSlot>();
vm_block = 62; continue;
}
// C line 43200
64 => {
obj = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) };
vm_block = 63; continue;
}
// C line 43199
65 => {
asc = array_sort_context {ctx: ctx, exception: (0 as i32), has_method: (0 as i32), method: *(argv).offset(((0 as i32)) as isize)};
vm_block = 64; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:43286. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_array_toSorted(mut ctx: *mut JSContext, mut this_val: JSValue, mut argc: i32, mut argv: *mut JSValue) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut arr: JSValue = core::mem::zeroed();
let mut obj: JSValue = core::mem::zeroed();
let mut ret: JSValue = core::mem::zeroed();
let mut arrp: *mut JSValue = core::mem::zeroed();
let mut pval: *mut JSValue = core::mem::zeroed();
let mut p: *mut JSObject = core::mem::zeroed();
let mut i: i64 = core::mem::zeroed();
let mut len: i64 = core::mem::zeroed();
let mut count32: u32 = core::mem::zeroed();
let mut ok: i32 = core::mem::zeroed();
let mut vm_block: usize = 32;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 43335
1 => {
return ret;
}
// C line 43334
2 => {
let _ = JS_FreeValue(ctx, obj);
vm_block = 1; continue;
}
// C line ? labels: exception
3 => {
let _ = JS_FreeValue(ctx, arr);
vm_block = 2; continue;
}
// C line 43330
4 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; arr = assigned; assigned };
vm_block = 3; continue;
}
// C line 43329
5 => {
let _ = { let assigned = arr; ret = assigned; assigned };
vm_block = 4; continue;
}
// C line 43327
6 => {
let _ = JS_FreeValue(ctx, ret);
vm_block = 5; continue;
}
// C line 43326
7 => {
vm_block = 3; continue;
}
// C line 43325
8 => {
vm_block = if (JS_IsException(ret)) != 0 { 7 } else { 6 }; continue;
}
// C line 43324
9 => {
let _ = { let assigned = js_array_sort(ctx, arr, argc, argv); ret = assigned; assigned };
vm_block = 8; continue;
}
// C line 43314
10 => {
vm_block = if ((((i) < (len)) as i32)) != 0 { 12 } else { 9 }; continue;
}
// C line ?
11 => {
let _ = { let _ = { let old = i; i = (i).wrapping_add(1); old }; { let old = pval; pval = (pval).offset(1); old } };
vm_block = 10; continue;
}
// C line 43315
12 => {
let _ = { let assigned = JS_DupValue(ctx, *(arrp).offset((i) as isize)); *(pval) = assigned; assigned };
vm_block = 11; continue;
}
// C line 43317
13 => {
vm_block = if ((((i) < (len)) as i32)) != 0 { 16 } else { 9 }; continue;
}
// C line ?
14 => {
let _ = { let _ = { let old = i; i = (i).wrapping_add(1); old }; { let old = pval; pval = (pval).offset(1); old } };
vm_block = 13; continue;
}
// C line 43319
15 => {
vm_block = 3; continue;
}
// C line 43318
16 => {
vm_block = if ((((((1 as i32)).wrapping_neg()) == (JS_TryGetPropertyInt64(ctx, obj, i, pval))) as i32)) != 0 { 15 } else { 14 }; continue;
}
// C line 43313
17 => {
vm_block = if (((((js_get_fast_array(ctx, obj, core::ptr::addr_of_mut!(arrp), core::ptr::addr_of_mut!(count32))) != 0) && (((((((count32) as i64)) == (len)) as i32)) != 0)) as i32)) != 0 { 10 } else { 13 }; continue;
}
// C line 43312
18 => {
let _ = { let assigned = ((((*(p)).u).array).u).values; pval = assigned; assigned };
vm_block = 17; continue;
}
// C line 43311
19 => {
let _ = { let assigned = (((0 as i32)) as i64); i = assigned; assigned };
vm_block = 18; continue;
}
// C line 43310
20 => {
let _ = { let assigned = ((((arr).u).ptr) as *mut JSObject); p = assigned; assigned };
vm_block = 19; continue;
}
// C line 43309
21 => {
vm_block = if ((((len) > ((((0 as i32)) as i64))) as i32)) != 0 { 20 } else { 9 }; continue;
}
// C line 43307
22 => {
vm_block = 3; continue;
}
// C line 43306
23 => {
vm_block = if (JS_IsException(arr)) != 0 { 22 } else { 21 }; continue;
}
// C line 43305
24 => {
let _ = { let assigned = js_allocate_fast_array(ctx, len); arr = assigned; assigned };
vm_block = 23; continue;
}
// C line 43303
25 => {
vm_block = 3; continue;
}
// C line 43302
26 => {
vm_block = if (js_get_length64(ctx, core::ptr::addr_of_mut!(len), obj)) != 0 { 25 } else { 24 }; continue;
}
// C line 43301
27 => {
let _ = { let assigned = JS_ToObject(ctx, this_val); obj = assigned; assigned };
vm_block = 26; continue;
}
// C line 43300
28 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; arr = assigned; assigned };
vm_block = 27; continue;
}
// C line 43299
29 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) }; ret = assigned; assigned };
vm_block = 28; continue;
}
// C line 43297
30 => {
return JS_ThrowTypeError(ctx, c"not a function".as_ptr());
}
// C line 43296
31 => {
vm_block = if ((!((ok) != 0) as i32)) != 0 { 30 } else { 29 }; continue;
}
// C line 43295
32 => {
let _ = { let assigned = ((((JS_IsUndefined(*(argv).offset(((0 as i32)) as isize))) != 0) || ((JS_IsFunction(ctx, *(argv).offset(((0 as i32)) as isize))) != 0)) as i32); ok = assigned; assigned };
vm_block = 31; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:43364. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_create_array_iterator(mut ctx: *mut JSContext, mut this_val: JSValue, mut argc: i32, mut argv: *mut JSValue, mut magic: i32) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut enum_obj: JSValue = core::mem::zeroed();
let mut arr: JSValue = core::mem::zeroed();
let mut it: *mut JSArrayIteratorData = core::mem::zeroed();
let mut kind: JSIteratorKindEnum = core::mem::zeroed();
let mut class_id: i32 = core::mem::zeroed();
let mut vm_block: usize = 22;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 43398
1 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line ? labels: fail
2 => {
let _ = JS_FreeValue(ctx, arr);
vm_block = 1; continue;
}
// C line ? labels: fail1
3 => {
let _ = JS_FreeValue(ctx, enum_obj);
vm_block = 2; continue;
}
// C line 43393
4 => {
return enum_obj;
}
// C line 43392
5 => {
let _ = JS_SetOpaque(enum_obj, ((it) as *mut c_void));
vm_block = 4; continue;
}
// C line 43391
6 => {
let _ = { let assigned = (((0 as i32)) as u32); (*(it)).idx = assigned; assigned };
vm_block = 5; continue;
}
// C line 43390
7 => {
let _ = { let assigned = kind; (*(it)).kind = assigned; assigned };
vm_block = 6; continue;
}
// C line 43389
8 => {
let _ = { let assigned = arr; (*(it)).obj = assigned; assigned };
vm_block = 7; continue;
}
// C line 43388
9 => {
vm_block = 3; continue;
}
// C line 43387
10 => {
vm_block = if ((!(!(it).is_null()) as i32)) != 0 { 9 } else { 8 }; continue;
}
// C line 43386
11 => {
let _ = { let assigned = ((js_malloc(ctx, (size_of::<JSArrayIteratorData>() as usize))) as *mut JSArrayIteratorData); it = assigned; assigned };
vm_block = 10; continue;
}
// C line 43385
12 => {
vm_block = 2; continue;
}
// C line 43384
13 => {
vm_block = if (JS_IsException(enum_obj)) != 0 { 12 } else { 11 }; continue;
}
// C line 43383
14 => {
let _ = { let assigned = JS_NewObjectClass(ctx, class_id); enum_obj = assigned; assigned };
vm_block = 13; continue;
}
// C line 43382
15 => {
vm_block = 2; continue;
}
// C line 43381
16 => {
vm_block = if (JS_IsException(arr)) != 0 { 15 } else { 14 }; continue;
}
// C line 43376
17 => {
let _ = { let assigned = (JS_CLASS_STRING_ITERATOR as i32); class_id = assigned; assigned };
vm_block = 16; continue;
}
// C line 43375
18 => {
let _ = { let assigned = JS_ToStringCheckObject(ctx, this_val); arr = assigned; assigned };
vm_block = 17; continue;
}
// C line 43379
19 => {
let _ = { let assigned = (JS_CLASS_ARRAY_ITERATOR as i32); class_id = assigned; assigned };
vm_block = 16; continue;
}
// C line 43378
20 => {
let _ = { let assigned = JS_ToObject(ctx, this_val); arr = assigned; assigned };
vm_block = 19; continue;
}
// C line 43373
21 => {
vm_block = if (((magic) & ((4 as i32)))) != 0 { 18 } else { 20 }; continue;
}
// C line 43372
22 => {
let _ = { let assigned = ((((magic) & ((3 as i32)))) as JSIteratorKindEnum); kind = assigned; assigned };
vm_block = 21; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:43401. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_array_iterator_next(mut ctx: *mut JSContext, mut this_val: JSValue, mut argc: i32, mut argv: *mut JSValue, mut pdone: *mut i32, mut magic: i32) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut it: *mut JSArrayIteratorData = core::mem::zeroed();
let mut len: u32 = core::mem::zeroed();
let mut idx: u32 = core::mem::zeroed();
let mut val: JSValue = core::mem::zeroed();
let mut obj: JSValue = core::mem::zeroed();
let mut p: *mut JSObject = core::mem::zeroed();
let mut args: [JSValue; 2] = core::mem::zeroed();
let mut num: JSValue = core::mem::zeroed();
let mut vm_block: usize = 36;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 43441
1 => {
return JS_NewUint32(ctx, idx);
}
// C line 43447
2 => {
return val;
}
// C line 43457
3 => {
return obj;
}
// C line 43456
4 => {
let _ = JS_FreeValue(ctx, num);
vm_block = 3; continue;
}
// C line 43455
5 => {
let _ = JS_FreeValue(ctx, val);
vm_block = 4; continue;
}
// C line 43454
6 => {
let _ = { let assigned = js_create_array(ctx, (2 as i32), (args).as_mut_ptr()); obj = assigned; assigned };
vm_block = 5; continue;
}
// C line 43453
7 => {
let _ = { let assigned = val; *((args).as_mut_ptr()).offset(((1 as i32)) as isize) = assigned; assigned };
vm_block = 6; continue;
}
// C line 43452
8 => {
let _ = { let assigned = num; *((args).as_mut_ptr()).offset(((0 as i32)) as isize) = assigned; assigned };
vm_block = 7; continue;
}
// C line 43451
9 => {
let _ = { let assigned = JS_NewUint32(ctx, idx); num = assigned; assigned };
vm_block = 8; continue;
}
// C line 43446
10 => {
vm_block = if (((((((*(it)).kind) as u32)) == ((((JS_ITERATOR_KIND_VALUE as i32)) as u32))) as i32)) != 0 { 2 } else { 9 }; continue;
}
// C line 43445
11 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 43444
12 => {
vm_block = if (JS_IsException(val)) != 0 { 11 } else { 10 }; continue;
}
// C line 43443
13 => {
let _ = { let assigned = JS_GetPropertyUint32(ctx, (*(it)).obj, idx); val = assigned; assigned };
vm_block = 12; continue;
}
// C line 43440
14 => {
vm_block = if (((((((*(it)).kind) as u32)) == ((((JS_ITERATOR_KIND_KEY as i32)) as u32))) as i32)) != 0 { 1 } else { 13 }; continue;
}
// C line 43439
15 => {
let _ = { let assigned = (0 as i32); *(pdone) = assigned; assigned };
vm_block = 14; continue;
}
// C line 43438
16 => {
let _ = { let assigned = (idx).wrapping_add((((1 as i32)) as u32)); (*(it)).idx = assigned; assigned };
vm_block = 15; continue;
}
// C line 43436
17 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) };
}
// C line ? labels: done
18 => {
let _ = { let assigned = (1 as i32); *(pdone) = assigned; assigned };
vm_block = 17; continue;
}
// C line 43433
19 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; (*(it)).obj = assigned; assigned };
vm_block = 18; continue;
}
// C line 43432
20 => {
let _ = JS_FreeValue(ctx, (*(it)).obj);
vm_block = 19; continue;
}
// C line 43431
21 => {
vm_block = if ((((idx) >= (len)) as i32)) != 0 { 20 } else { 16 }; continue;
}
// C line 43430
22 => {
let _ = { let assigned = (*(it)).idx; idx = assigned; assigned };
vm_block = 21; continue;
}
// C line 43422
23 => {
let _ = { let assigned = (((*(p)).u).array).count; len = assigned; assigned };
vm_block = 22; continue;
}
// C line 43420
24 => {
vm_block = 28; continue;
}
// C line 43419
25 => {
let _ = JS_ThrowTypeErrorArrayBufferOOB(ctx);
vm_block = 24; continue;
}
// C line 43418
26 => {
vm_block = if (typed_array_is_oob(p)) != 0 { 25 } else { 23 }; continue;
}
// C line 43427
27 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line ? labels: fail1
28 => {
let _ = { let assigned = (0 as i32); *(pdone) = assigned; assigned };
vm_block = 27; continue;
}
// C line 43424
29 => {
vm_block = if (js_get_length32(ctx, core::ptr::addr_of_mut!(len), (*(it)).obj)) != 0 { 28 } else { 22 }; continue;
}
// C line 43416
30 => {
vm_block = if (((((((((((*(p)).class_id) as i32)) >= ((JS_CLASS_UINT8C_ARRAY as i32))) as i32)) != 0) && ((((((((*(p)).class_id) as i32)) <= ((JS_CLASS_FLOAT64_ARRAY as i32))) as i32)) != 0)) as i32)) != 0 { 26 } else { 29 }; continue;
}
// C line 43415
31 => {
let _ = { let assigned = (((((*(it)).obj).u).ptr) as *mut JSObject); p = assigned; assigned };
vm_block = 30; continue;
}
// C line 43414
32 => {
vm_block = 18; continue;
}
// C line 43413
33 => {
vm_block = if (JS_IsUndefined((*(it)).obj)) != 0 { 32 } else { 31 }; continue;
}
// C line 43412
34 => {
vm_block = 28; continue;
}
// C line 43411
35 => {
vm_block = if ((!(!(it).is_null()) as i32)) != 0 { 34 } else { 33 }; continue;
}
// C line 43410
36 => {
let _ = { let assigned = ((JS_GetOpaque2(ctx, this_val, (((JS_CLASS_ARRAY_ITERATOR as i32)) as JSClassID))) as *mut JSArrayIteratorData); it = assigned; assigned };
vm_block = 35; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:43491. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_iterator_wrap_next(mut ctx: *mut JSContext, mut this_val: JSValue, mut argc: i32, mut argv: *mut JSValue, mut pdone: *mut i32, mut magic: i32) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut it: *mut JSIteratorWrapData = core::mem::zeroed();
let mut method: JSValue = core::mem::zeroed();
let mut ret: JSValue = core::mem::zeroed();
let mut vm_block: usize = 14;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 43501
1 => {
return JS_IteratorNext(ctx, (*(it)).wrapped_iter, (*(it)).wrapped_next, (0 as i32), core::ptr::null_mut::<JSValue>(), pdone);
}
// C line 43512
2 => {
return ret;
}
// C line 43511
3 => {
let _ = JS_FreeValue(ctx, method);
vm_block = 2; continue;
}
// C line 43510
4 => {
let _ = { let assigned = JS_IteratorNext2(ctx, (*(it)).wrapped_iter, method, (0 as i32), core::ptr::null_mut::<JSValue>(), pdone); ret = assigned; assigned };
vm_block = 3; continue;
}
// C line 43508
5 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) };
}
// C line 43507
6 => {
let _ = { let assigned = (1 as i32); *(pdone) = assigned; assigned };
vm_block = 5; continue;
}
// C line 43506
7 => {
vm_block = if (((((JS_IsNull(method)) != 0) || ((JS_IsUndefined(method)) != 0)) as i32)) != 0 { 6 } else { 4 }; continue;
}
// C line 43505
8 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 43504
9 => {
vm_block = if (JS_IsException(method)) != 0 { 8 } else { 7 }; continue;
}
// C line 43503
10 => {
let _ = { let assigned = JS_GetProperty(ctx, (*(it)).wrapped_iter, (((crate::quickjs_atom::JS_ATOM_return as i32)) as JSAtom)); method = assigned; assigned };
vm_block = 9; continue;
}
// C line 43500
11 => {
vm_block = if ((((magic) == ((0 as i32))) as i32)) != 0 { 1 } else { 10 }; continue;
}
// C line 43499
12 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 43498
13 => {
vm_block = if ((!(!(it).is_null()) as i32)) != 0 { 12 } else { 11 }; continue;
}
// C line 43497
14 => {
let _ = { let assigned = ((JS_GetOpaque2(ctx, this_val, (((JS_CLASS_ITERATOR_WRAP as i32)) as JSClassID))) as *mut JSIteratorWrapData); it = assigned; assigned };
vm_block = 13; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:43523. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_iterator_constructor_getset(mut ctx: *mut JSContext, mut this_val: JSValue, mut argc: i32, mut argv: *mut JSValue, mut magic: i32, mut func_data: *mut JSValue) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut ret: i32 = core::mem::zeroed();
let mut vm_block: usize = 8;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 43539
1 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) };
}
// C line 43538
2 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 43537
3 => {
vm_block = if ((((ret) < ((0 as i32))) as i32)) != 0 { 2 } else { 1 }; continue;
}
// C line 43534
4 => {
let _ = { let assigned = JS_DefinePropertyValue(ctx, this_val, (((crate::quickjs_atom::JS_ATOM_constructor as i32)) as JSAtom), JS_DupValue(ctx, *(argv).offset(((0 as i32)) as isize)), ((((1 as i32)).wrapping_shl(((1 as i32)) as u32)) | (((1 as i32)).wrapping_shl(((0 as i32)) as u32)))); ret = assigned; assigned };
vm_block = 3; continue;
}
// C line 43533
5 => {
return JS_ThrowTypeErrorNotAnObject(ctx);
}
// C line 43532
6 => {
vm_block = if ((!((JS_IsObject(*(argv).offset(((0 as i32)) as isize))) != 0) as i32)) != 0 { 5 } else { 4 }; continue;
}
// C line 43541
7 => {
return JS_DupValue(ctx, *(func_data).offset(((0 as i32)) as isize));
}
// C line 43531
8 => {
vm_block = if ((((argc) > ((0 as i32))) as i32)) != 0 { 6 } else { 7 }; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:43545. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_iterator_constructor(mut ctx: *mut JSContext, mut new_target: JSValue, mut argc: i32, mut argv: *mut JSValue) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut p: *mut JSObject = core::mem::zeroed();
let mut vm_block: usize = 6;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 43557
1 => {
return js_create_from_ctor(ctx, new_target, (JS_CLASS_ITERATOR as i32));
}
// C line 43555
2 => {
return JS_ThrowTypeError(ctx, c"abstract class not constructable".as_ptr());
}
// C line 43553
3 => {
vm_block = if (((((((((((*(p)).class_id) as i32)) == ((JS_CLASS_C_FUNCTION as i32))) as i32)) != 0) && ((((((((*(p)).u).cfunc).c_function).generic).map(|f| f as usize) == Some(js_iterator_constructor as usize)) as i32) != 0)) as i32)) != 0 { 2 } else { 1 }; continue;
}
// C line 43552
4 => {
let _ = { let assigned = ((((new_target).u).ptr) as *mut JSObject); p = assigned; assigned };
vm_block = 3; continue;
}
// C line 43551
5 => {
return JS_ThrowTypeError(ctx, c"constructor requires 'new'".as_ptr());
}
// C line 43550
6 => {
vm_block = if (((((JS_TAG_OBJECT as i32)) != ((((new_target).tag) as i32))) as i32)) != 0 { 5 } else { 4 }; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:43594. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_iterator_concat_next(mut ctx: *mut JSContext, mut this_val: JSValue, mut argc: i32, mut argv: *mut JSValue, mut pdone: *mut i32, mut magic: i32) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut iter: JSValue = core::mem::zeroed();
let mut item: JSValue = core::mem::zeroed();
let mut next: JSValue = core::mem::zeroed();
let mut val: JSValue = core::mem::zeroed();
let mut obj: *mut JSValue = core::mem::zeroed();
let mut meth: *mut JSValue = core::mem::zeroed();
let mut ret: JSValue = core::mem::zeroed();
let mut it: *mut JSIteratorConcatData = core::mem::zeroed();
let mut done: i32 = core::mem::zeroed();
let mut vm_block: usize = 54;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 43666
1 => {
return ret;
}
// C line 43665
2 => {
let _ = { let assigned = (0 as i32); (*(it)).running = assigned; assigned };
vm_block = 1; continue;
}
// C line 43611
3 => {
vm_block = 47; continue;
}
// C line 43638
4 => {
vm_block = 2; continue;
}
// C line 43637
5 => {
let _ = { let assigned = item; ret = assigned; assigned };
vm_block = 4; continue;
}
// C line 43652
6 => {
vm_block = 2; continue;
}
// C line 43651
7 => {
let _ = JS_FreeValue(ctx, item);
vm_block = 6; continue;
}
// C line 43650
8 => {
let _ = { let assigned = JS_GetProperty(ctx, item, (((crate::quickjs_atom::JS_ATOM_value as i32)) as JSAtom)); ret = assigned; assigned };
vm_block = 7; continue;
}
// C line 43649
9 => {
vm_block = 24; continue;
}
// C line 43648
10 => {
vm_block = if (done) != 0 { 9 } else { 8 }; continue;
}
// C line 43647
11 => {
let _ = { let assigned = JS_ToBoolFree(ctx, val); done = assigned; assigned };
vm_block = 10; continue;
}
// C line 43645
12 => {
vm_block = 2; continue;
}
// C line ? labels: fail
13 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) }; ret = assigned; assigned };
vm_block = 12; continue;
}
// C line 43642
14 => {
let _ = JS_FreeValue(ctx, item);
vm_block = 13; continue;
}
// C line 43641
15 => {
vm_block = if (JS_IsException(val)) != 0 { 14 } else { 11 }; continue;
}
// C line 43640
16 => {
let _ = { let assigned = JS_GetProperty(ctx, item, (((crate::quickjs_atom::JS_ATOM_done as i32)) as JSAtom)); val = assigned; assigned };
vm_block = 15; continue;
}
// C line 43662
17 => {
let _ = { (*(it)).index = ((*(it)).index).wrapping_add((2 as i32)); (*(it)).index };
vm_block = 3; continue;
}
// C line 43661
18 => {
let _ = JS_FreeValue(ctx, *(obj));
vm_block = 17; continue;
}
// C line 43660
19 => {
let _ = JS_FreeValue(ctx, *(meth));
vm_block = 18; continue;
}
// C line 43659
20 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; (*(it)).next = assigned; assigned };
vm_block = 19; continue;
}
// C line 43658
21 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; (*(it)).iter = assigned; assigned };
vm_block = 20; continue;
}
// C line 43657
22 => {
let _ = JS_FreeValue(ctx, next);
vm_block = 21; continue;
}
// C line 43656
23 => {
let _ = JS_FreeValue(ctx, iter);
vm_block = 22; continue;
}
// C line ? labels: done_next
24 => {
let _ = JS_FreeValue(ctx, item);
vm_block = 23; continue;
}
// C line 43639
25 => {
vm_block = if ((((done) == ((2 as i32))) as i32)) != 0 { 16 } else { 24 }; continue;
}
// C line 43636
26 => {
vm_block = if ((((done) == ((0 as i32))) as i32)) != 0 { 5 } else { 25 }; continue;
}
// C line 43635
27 => {
vm_block = 13; continue;
}
// C line 43634
28 => {
vm_block = if (JS_IsException(item)) != 0 { 27 } else { 26 }; continue;
}
// C line 43633
29 => {
let _ = { let assigned = JS_IteratorNext2(ctx, iter, next, (0 as i32), core::ptr::null_mut::<JSValue>(), core::ptr::addr_of_mut!(done)); item = assigned; assigned };
vm_block = 28; continue;
}
// C line 43631
30 => {
let _ = { let assigned = next; (*(it)).next = assigned; assigned };
vm_block = 29; continue;
}
// C line 43630
31 => {
vm_block = 13; continue;
}
// C line 43629
32 => {
vm_block = if (JS_IsException(next)) != 0 { 31 } else { 30 }; continue;
}
// C line 43628
33 => {
let _ = { let assigned = JS_GetProperty(ctx, iter, (((crate::quickjs_atom::JS_ATOM_next as i32)) as JSAtom)); next = assigned; assigned };
vm_block = 32; continue;
}
// C line 43627
34 => {
vm_block = if (JS_IsUndefined(next)) != 0 { 33 } else { 29 }; continue;
}
// C line 43626
35 => {
let _ = { let assigned = (*(it)).next; next = assigned; assigned };
vm_block = 34; continue;
}
// C line 43624
36 => {
let _ = { let assigned = iter; (*(it)).iter = assigned; assigned };
vm_block = 35; continue;
}
// C line 43623
37 => {
vm_block = 13; continue;
}
// C line 43622
38 => {
vm_block = if (JS_IsException(iter)) != 0 { 37 } else { 36 }; continue;
}
// C line 43621
39 => {
let _ = { let assigned = JS_GetIterator2(ctx, *(obj), *(meth)); iter = assigned; assigned };
vm_block = 38; continue;
}
// C line 43620
40 => {
vm_block = if (JS_IsUndefined(iter)) != 0 { 39 } else { 35 }; continue;
}
// C line 43619
41 => {
let _ = { let assigned = (*(it)).iter; iter = assigned; assigned };
vm_block = 40; continue;
}
// C line 43618
42 => {
let _ = { let assigned = core::ptr::addr_of_mut!(*(((*(it)).values).as_mut_ptr()).offset((((*(it)).index).wrapping_add((1 as i32))) as isize)); meth = assigned; assigned };
vm_block = 41; continue;
}
// C line 43617
43 => {
let _ = { let assigned = core::ptr::addr_of_mut!(*(((*(it)).values).as_mut_ptr()).offset((((*(it)).index).wrapping_add((0 as i32))) as isize)); obj = assigned; assigned };
vm_block = 42; continue;
}
// C line 43615
44 => {
vm_block = 2; continue;
}
// C line 43614
45 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; ret = assigned; assigned };
vm_block = 44; continue;
}
// C line 43613
46 => {
let _ = { let assigned = (1 as i32); *(pdone) = assigned; assigned };
vm_block = 45; continue;
}
// C line 43612
47 => {
vm_block = if (((((*(it)).index) >= ((*(it)).count)) as i32)) != 0 { 46 } else { 43 }; continue;
}
// C line 43610
48 => {
let _ = { let assigned = (1 as i32); (*(it)).running = assigned; assigned };
vm_block = 3; continue;
}
// C line 43608
49 => {
return JS_ThrowTypeError(ctx, c"already running".as_ptr());
}
// C line 43607
50 => {
vm_block = if ((*(it)).running) != 0 { 49 } else { 48 }; continue;
}
// C line 43606
51 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 43605
52 => {
vm_block = if ((!(!(it).is_null()) as i32)) != 0 { 51 } else { 50 }; continue;
}
// C line 43604
53 => {
let _ = { let assigned = ((JS_GetOpaque2(ctx, this_val, (((JS_CLASS_ITERATOR_CONCAT as i32)) as JSClassID))) as *mut JSIteratorConcatData); it = assigned; assigned };
vm_block = 52; continue;
}
// C line 43602
54 => {
let _ = { let assigned = (0 as i32); *(pdone) = assigned; assigned };
vm_block = 53; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:43669. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_iterator_concat_return(mut ctx: *mut JSContext, mut this_val: JSValue, mut argc: i32, mut argv: *mut JSValue) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut it: *mut JSIteratorConcatData = core::mem::zeroed();
let mut ret: JSValue = core::mem::zeroed();
let mut vm_block: usize = 21;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 43697
1 => {
return ret;
}
// C line 43696
2 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; (*(it)).next = assigned; assigned };
vm_block = 1; continue;
}
// C line 43695
3 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; (*(it)).iter = assigned; assigned };
vm_block = 2; continue;
}
// C line 43694
4 => {
let _ = JS_FreeValue(ctx, (*(it)).next);
vm_block = 3; continue;
}
// C line 43693
5 => {
let _ = JS_FreeValue(ctx, (*(it)).iter);
vm_block = 4; continue;
}
// C line 43691
6 => {
vm_block = if (((((*(it)).index) < ((*(it)).count)) as i32)) != 0 { 7 } else { 5 }; continue;
}
// C line 43692
7 => {
let _ = JS_FreeValue(ctx, *(((*(it)).values).as_mut_ptr()).offset(({ let old = (*(it)).index; (*(it)).index = ((*(it)).index).wrapping_add(1); old }) as isize));
vm_block = 6; continue;
}
// C line 43689
8 => {
let _ = { let assigned = (0 as i32); (*(it)).running = assigned; assigned };
vm_block = 6; continue;
}
// C line 43688
9 => {
let _ = { let assigned = JS_CallFree(ctx, ret, (*(it)).iter, (0 as i32), core::ptr::null_mut::<JSValue>()); ret = assigned; assigned };
vm_block = 8; continue;
}
// C line 43686
10 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 43685
11 => {
let _ = { let assigned = (0 as i32); (*(it)).running = assigned; assigned };
vm_block = 10; continue;
}
// C line 43684
12 => {
vm_block = if (JS_IsException(ret)) != 0 { 11 } else { 9 }; continue;
}
// C line 43683
13 => {
let _ = { let assigned = JS_GetProperty(ctx, (*(it)).iter, (((crate::quickjs_atom::JS_ATOM_return as i32)) as JSAtom)); ret = assigned; assigned };
vm_block = 12; continue;
}
// C line 43682
14 => {
let _ = { let assigned = (1 as i32); (*(it)).running = assigned; assigned };
vm_block = 13; continue;
}
// C line 43681
15 => {
vm_block = if ((!((JS_IsUndefined((*(it)).iter)) != 0) as i32)) != 0 { 14 } else { 6 }; continue;
}
// C line 43680
16 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; ret = assigned; assigned };
vm_block = 15; continue;
}
// C line 43679
17 => {
return JS_ThrowTypeError(ctx, c"already running".as_ptr());
}
// C line 43678
18 => {
vm_block = if ((*(it)).running) != 0 { 17 } else { 16 }; continue;
}
// C line 43677
19 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 43676
20 => {
vm_block = if ((!(!(it).is_null()) as i32)) != 0 { 19 } else { 18 }; continue;
}
// C line 43675
21 => {
let _ = { let assigned = ((JS_GetOpaque2(ctx, this_val, (((JS_CLASS_ITERATOR_CONCAT as i32)) as JSClassID))) as *mut JSIteratorConcatData); it = assigned; assigned };
vm_block = 20; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:43706. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_iterator_concat(mut ctx: *mut JSContext, mut this_val: JSValue, mut argc: i32, mut argv: *mut JSValue) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut it: *mut JSIteratorConcatData = core::mem::zeroed();
let mut obj: JSValue = core::mem::zeroed();
let mut method: JSValue = core::mem::zeroed();
let mut i: i32 = core::mem::zeroed();
let mut obj_1: JSValue = core::mem::zeroed();
let mut i_1: i32 = core::mem::zeroed();
let mut vm_block: usize = 35;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 43746
1 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 43745
2 => {
let _ = js_free(ctx, ((it) as *mut c_void));
vm_block = 1; continue;
}
// C line 43743
3 => {
vm_block = if ((((i_1) < ((*(it)).count)) as i32)) != 0 { 5 } else { 2 }; continue;
}
// C line ?
4 => {
let _ = { let old = i_1; i_1 = (i_1).wrapping_add(1); old };
vm_block = 3; continue;
}
// C line 43744
5 => {
let _ = JS_FreeValue(ctx, *(((*(it)).values).as_mut_ptr()).offset((i_1) as isize));
vm_block = 4; continue;
}
// C line 43743 labels: fail
6 => {
i_1 = (0 as i32);
vm_block = 3; continue;
}
// C line 43741
7 => {
return obj;
}
// C line 43740
8 => {
let _ = JS_SetOpaque(obj, ((it) as *mut c_void));
vm_block = 7; continue;
}
// C line 43739
9 => {
vm_block = 6; continue;
}
// C line 43738
10 => {
vm_block = if (JS_IsException(obj)) != 0 { 9 } else { 8 }; continue;
}
// C line 43737
11 => {
let _ = { let assigned = JS_NewObjectClass(ctx, (JS_CLASS_ITERATOR_CONCAT as i32)); obj = assigned; assigned };
vm_block = 10; continue;
}
// C line 43720
12 => {
vm_block = if ((((i) < (argc)) as i32)) != 0 { 26 } else { 11 }; continue;
}
// C line ?
13 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 12; continue;
}
// C line 43735
14 => {
let _ = { let assigned = method; *(((*(it)).values).as_mut_ptr()).offset(({ let old = (*(it)).count; (*(it)).count = ((*(it)).count).wrapping_add(1); old }) as isize) = assigned; assigned };
vm_block = 13; continue;
}
// C line 43734
15 => {
let _ = { let assigned = JS_DupValue(ctx, obj_1); *(((*(it)).values).as_mut_ptr()).offset(({ let old = (*(it)).count; (*(it)).count = ((*(it)).count).wrapping_add(1); old }) as isize) = assigned; assigned };
vm_block = 14; continue;
}
// C line 43732
16 => {
vm_block = 6; continue;
}
// C line 43731
17 => {
let _ = JS_FreeValue(ctx, method);
vm_block = 16; continue;
}
// C line 43730
18 => {
let _ = JS_ThrowTypeError(ctx, c"not a function".as_ptr());
vm_block = 17; continue;
}
// C line 43729
19 => {
vm_block = if ((!((JS_IsFunction(ctx, method)) != 0) as i32)) != 0 { 18 } else { 15 }; continue;
}
// C line 43728
20 => {
vm_block = 6; continue;
}
// C line 43727
21 => {
vm_block = if (JS_IsException(method)) != 0 { 20 } else { 19 }; continue;
}
// C line 43726
22 => {
let _ = { let assigned = JS_GetProperty(ctx, obj_1, (((crate::quickjs_atom::JS_ATOM_Symbol_iterator as i32)) as JSAtom)); method = assigned; assigned };
vm_block = 21; continue;
}
// C line 43724
23 => {
vm_block = 6; continue;
}
// C line 43723
24 => {
let _ = JS_ThrowTypeErrorNotAnObject(ctx);
vm_block = 23; continue;
}
// C line 43722
25 => {
vm_block = if ((!((JS_IsObject(obj_1)) != 0) as i32)) != 0 { 24 } else { 22 }; continue;
}
// C line 43721
26 => {
obj_1 = *(argv).offset((i) as isize);
vm_block = 25; continue;
}
// C line 43720
27 => {
i = (0 as i32);
vm_block = 12; continue;
}
// C line 43719
28 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; (*(it)).next = assigned; assigned };
vm_block = 27; continue;
}
// C line 43718
29 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; (*(it)).iter = assigned; assigned };
vm_block = 28; continue;
}
// C line 43717
30 => {
let _ = { let assigned = (0 as i32); (*(it)).count = assigned; assigned };
vm_block = 29; continue;
}
// C line 43716
31 => {
let _ = { let assigned = (0 as i32); (*(it)).index = assigned; assigned };
vm_block = 30; continue;
}
// C line 43715
32 => {
let _ = { let assigned = (0 as i32); (*(it)).running = assigned; assigned };
vm_block = 31; continue;
}
// C line 43714
33 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 43713
34 => {
vm_block = if ((!(!(it).is_null()) as i32)) != 0 { 33 } else { 32 }; continue;
}
// C line 43712
35 => {
let _ = { let assigned = ((js_malloc(ctx, ((size_of::<JSIteratorConcatData>() as usize)).wrapping_add((((((2 as i32)).wrapping_mul(argc)) as usize)).wrapping_mul((size_of::<JSValue>() as usize))))) as *mut JSIteratorConcatData); it = assigned; assigned };
vm_block = 34; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:43749. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_iterator_from(mut ctx: *mut JSContext, mut this_val: JSValue, mut argc: i32, mut argv: *mut JSValue) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut obj: JSValue = core::mem::zeroed();
let mut method: JSValue = core::mem::zeroed();
let mut iter: JSValue = core::mem::zeroed();
let mut wrapper: JSValue = core::mem::zeroed();
let mut it: *mut JSIteratorWrapData = core::mem::zeroed();
let mut ret: i32 = core::mem::zeroed();
let mut vm_block: usize = 37;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 43801
1 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 43800
2 => {
let _ = JS_FreeValue(ctx, wrapper);
vm_block = 1; continue;
}
// C line 43799
3 => {
let _ = JS_FreeValue(ctx, iter);
vm_block = 2; continue;
}
// C line ? labels: fail
4 => {
let _ = JS_FreeValue(ctx, method);
vm_block = 3; continue;
}
// C line 43795
5 => {
return wrapper;
}
// C line 43794
6 => {
let _ = JS_SetOpaque(wrapper, ((it) as *mut c_void));
vm_block = 5; continue;
}
// C line 43793
7 => {
let _ = { let assigned = method; (*(it)).wrapped_next = assigned; assigned };
vm_block = 6; continue;
}
// C line 43792
8 => {
let _ = { let assigned = iter; (*(it)).wrapped_iter = assigned; assigned };
vm_block = 7; continue;
}
// C line 43791
9 => {
vm_block = 4; continue;
}
// C line 43790
10 => {
vm_block = if ((!(!(it).is_null()) as i32)) != 0 { 9 } else { 8 }; continue;
}
// C line 43789
11 => {
let _ = { let assigned = ((js_malloc(ctx, (size_of::<JSIteratorWrapData>() as usize))) as *mut JSIteratorWrapData); it = assigned; assigned };
vm_block = 10; continue;
}
// C line 43788
12 => {
vm_block = 4; continue;
}
// C line 43787
13 => {
vm_block = if (JS_IsException(wrapper)) != 0 { 12 } else { 11 }; continue;
}
// C line 43786
14 => {
let _ = { let assigned = JS_NewObjectClass(ctx, (JS_CLASS_ITERATOR_WRAP as i32)); wrapper = assigned; assigned };
vm_block = 13; continue;
}
// C line 43783
15 => {
return iter;
}
// C line 43782
16 => {
let _ = JS_FreeValue(ctx, method);
vm_block = 15; continue;
}
// C line 43781
17 => {
vm_block = if (ret) != 0 { 16 } else { 14 }; continue;
}
// C line 43780
18 => {
vm_block = 4; continue;
}
// C line 43779
19 => {
vm_block = if ((((ret) < ((0 as i32))) as i32)) != 0 { 18 } else { 17 }; continue;
}
// C line 43778
20 => {
let _ = { let assigned = JS_OrdinaryIsInstanceOf(ctx, iter, (*(ctx)).iterator_ctor); ret = assigned; assigned };
vm_block = 19; continue;
}
// C line 43776
21 => {
vm_block = 4; continue;
}
// C line 43775
22 => {
vm_block = if (JS_IsException(method)) != 0 { 21 } else { 20 }; continue;
}
// C line 43774
23 => {
let _ = { let assigned = JS_GetProperty(ctx, iter, (((crate::quickjs_atom::JS_ATOM_next as i32)) as JSAtom)); method = assigned; assigned };
vm_block = 22; continue;
}
// C line 43773
24 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; wrapper = assigned; assigned };
vm_block = 23; continue;
}
// C line 43765
25 => {
let _ = { let assigned = JS_DupValue(ctx, obj); iter = assigned; assigned };
vm_block = 24; continue;
}
// C line 43770
26 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 43769
27 => {
vm_block = if (JS_IsException(iter)) != 0 { 26 } else { 24 }; continue;
}
// C line 43768
28 => {
let _ = JS_FreeValue(ctx, method);
vm_block = 27; continue;
}
// C line 43767
29 => {
let _ = { let assigned = JS_GetIterator2(ctx, obj, method); iter = assigned; assigned };
vm_block = 28; continue;
}
// C line 43764
30 => {
vm_block = if (((((JS_IsNull(method)) != 0) || ((JS_IsUndefined(method)) != 0)) as i32)) != 0 { 25 } else { 29 }; continue;
}
// C line 43763
31 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 43762
32 => {
vm_block = if (JS_IsException(method)) != 0 { 31 } else { 30 }; continue;
}
// C line 43761
33 => {
let _ = { let assigned = JS_GetProperty(ctx, obj, (((crate::quickjs_atom::JS_ATOM_Symbol_iterator as i32)) as JSAtom)); method = assigned; assigned };
vm_block = 32; continue;
}
// C line 43759
34 => {
return JS_ThrowTypeError(ctx, c"Iterator.from called on non-object".as_ptr());
}
// C line 43758
35 => {
vm_block = if ((!((JS_IsString(obj)) != 0) as i32)) != 0 { 34 } else { 33 }; continue;
}
// C line 43757
36 => {
vm_block = if ((!((JS_IsObject(obj)) != 0) as i32)) != 0 { 35 } else { 33 }; continue;
}
// C line 43752
37 => {
obj = *(argv).offset(((0 as i32)) as isize);
vm_block = 36; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:43827. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_create_iterator_helper(mut ctx: *mut JSContext, mut this_val: JSValue, mut argc: i32, mut argv: *mut JSValue, mut magic: i32) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut func: JSValue = core::mem::zeroed();
let mut obj: JSValue = core::mem::zeroed();
let mut method: JSValue = core::mem::zeroed();
let mut count: i64 = core::mem::zeroed();
let mut it: *mut JSIteratorHelperData = core::mem::zeroed();
let mut v: JSValue = core::mem::zeroed();
let mut dlimit: f64 = core::mem::zeroed();
let mut vm_block: usize = 58;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 43917
1 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line ? labels: fail
2 => {
let _ = JS_IteratorClose(ctx, this_val, (1 as i32));
vm_block = 1; continue;
}
// C line ? labels: range_error
3 => {
let _ = JS_ThrowRangeError(ctx, c"must be positive".as_ptr());
vm_block = 2; continue;
}
// C line 43912
4 => {
return obj;
}
// C line 43911
5 => {
let _ = JS_SetOpaque(obj, ((it) as *mut c_void));
vm_block = 4; continue;
}
// C line 43910
6 => {
let _ = { let assigned = (((0 as i32)) as u8); (*(it)).set_done((assigned) as _); assigned };
vm_block = 5; continue;
}
// C line 43909
7 => {
let _ = { let assigned = (((0 as i32)) as u8); (*(it)).set_executing((assigned) as _); assigned };
vm_block = 6; continue;
}
// C line 43908
8 => {
let _ = { let assigned = count; (*(it)).count = assigned; assigned };
vm_block = 7; continue;
}
// C line 43907
9 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; (*(it)).inner = assigned; assigned };
vm_block = 8; continue;
}
// C line 43906
10 => {
let _ = { let assigned = method; (*(it)).next = assigned; assigned };
vm_block = 9; continue;
}
// C line 43905
11 => {
let _ = { let assigned = JS_DupValue(ctx, func); (*(it)).func = assigned; assigned };
vm_block = 10; continue;
}
// C line 43904
12 => {
let _ = { let assigned = JS_DupValue(ctx, this_val); (*(it)).obj = assigned; assigned };
vm_block = 11; continue;
}
// C line 43903
13 => {
let _ = { let assigned = ((magic) as JSIteratorHelperKindEnum); (*(it)).set_kind((assigned) as _); assigned };
vm_block = 12; continue;
}
// C line 43901
14 => {
vm_block = 2; continue;
}
// C line 43900
15 => {
let _ = JS_FreeValue(ctx, method);
vm_block = 14; continue;
}
// C line 43899
16 => {
let _ = JS_FreeValue(ctx, obj);
vm_block = 15; continue;
}
// C line 43898
17 => {
vm_block = if ((!(!(it).is_null()) as i32)) != 0 { 16 } else { 13 }; continue;
}
// C line 43897
18 => {
let _ = { let assigned = ((js_malloc(ctx, (size_of::<JSIteratorHelperData>() as usize))) as *mut JSIteratorHelperData); it = assigned; assigned };
vm_block = 17; continue;
}
// C line 43895
19 => {
vm_block = 2; continue;
}
// C line 43894
20 => {
let _ = JS_FreeValue(ctx, method);
vm_block = 19; continue;
}
// C line 43893
21 => {
vm_block = if (JS_IsException(obj)) != 0 { 20 } else { 18 }; continue;
}
// C line 43892
22 => {
let _ = { let assigned = JS_NewObjectClass(ctx, (JS_CLASS_ITERATOR_HELPER as i32)); obj = assigned; assigned };
vm_block = 21; continue;
}
// C line 43891
23 => {
vm_block = 2; continue;
}
// C line 43890
24 => {
vm_block = if (JS_IsException(method)) != 0 { 23 } else { 22 }; continue;
}
// C line 43889
25 => {
let _ = { let assigned = JS_GetProperty(ctx, this_val, (((crate::quickjs_atom::JS_ATOM_next as i32)) as JSAtom)); method = assigned; assigned };
vm_block = 24; continue;
}
// C line 43886
26 => {
vm_block = 25; continue;
}
// C line ?
27 => {
let _ = std::process::abort();
vm_block = 26; continue;
}
// C line 43883
28 => {
vm_block = 25; continue;
}
// C line 43881
29 => {
vm_block = 2; continue;
}
// C line 43880
30 => {
vm_block = if (check_function(ctx, func)) != 0 { 29 } else { 28 }; continue;
}
// C line 43879
31 => {
let _ = { let assigned = *(argv).offset(((0 as i32)) as isize); func = assigned; assigned };
vm_block = 30; continue;
}
// C line 43874
32 => {
vm_block = 25; continue;
}
// C line 43872
33 => {
vm_block = 3; continue;
}
// C line 43871
34 => {
vm_block = if ((((count) < ((((0 as i32)) as i64))) as i32)) != 0 { 33 } else { 32 }; continue;
}
// C line 43861
35 => {
vm_block = 3; continue;
}
// C line 43863
36 => {
let _ = { let assigned = (((((1 as i32)) as i64)).wrapping_shl(((53 as i32)) as u32)).wrapping_sub((((1 as i32)) as i64)); count = assigned; assigned };
vm_block = 34; continue;
}
// C line 43860
37 => {
vm_block = if ((((dlimit) < ((((0 as i32)) as f64))) as i32)) != 0 { 35 } else { 36 }; continue;
}
// C line 43859
38 => {
let _ = JS_FreeValue(ctx, v);
vm_block = 37; continue;
}
// C line 43869
39 => {
vm_block = 2; continue;
}
// C line 43868
40 => {
vm_block = if (JS_ToInt64Free(ctx, core::ptr::addr_of_mut!(count), v)) != 0 { 39 } else { 34 }; continue;
}
// C line 43867
41 => {
vm_block = 2; continue;
}
// C line 43866
42 => {
vm_block = if (JS_IsException(v)) != 0 { 41 } else { 40 }; continue;
}
// C line 43865
43 => {
let _ = { let assigned = JS_ToIntegerFree(ctx, v); v = assigned; assigned };
vm_block = 42; continue;
}
// C line 43858
44 => {
vm_block = if ((!((if (((((size_of::<f64>() as usize)) == ((size_of::<f32>() as usize))) as i32)) != 0 { (((dlimit) as f32)).is_finite() as i32 } else { if (((((size_of::<f64>() as usize)) == ((size_of::<f64>() as usize))) as i32)) != 0 { (dlimit).is_finite() as i32 } else { (dlimit).is_finite() as i32 } }) != 0) as i32)) != 0 { 38 } else { 43 }; continue;
}
// C line 43856
45 => {
vm_block = 3; continue;
}
// C line 43855
46 => {
let _ = JS_FreeValue(ctx, v);
vm_block = 45; continue;
}
// C line 43854
47 => {
vm_block = if (if (((((size_of::<f64>() as usize)) == ((size_of::<f32>() as usize))) as i32)) != 0 { (((dlimit) as f32)).is_nan() as i32 } else { if (((((size_of::<f64>() as usize)) == ((size_of::<f64>() as usize))) as i32)) != 0 { (dlimit).is_nan() as i32 } else { (dlimit).is_nan() as i32 } }) != 0 { 46 } else { 44 }; continue;
}
// C line 43852
48 => {
vm_block = 2; continue;
}
// C line 43851
49 => {
let _ = JS_FreeValue(ctx, v);
vm_block = 48; continue;
}
// C line 43850
50 => {
vm_block = if (JS_ToFloat64(ctx, core::ptr::addr_of_mut!(dlimit), v)) != 0 { 49 } else { 47 }; continue;
}
// C line 43848
51 => {
vm_block = 2; continue;
}
// C line 43847
52 => {
vm_block = if (JS_IsException(v)) != 0 { 51 } else { 50 }; continue;
}
// C line 43846
53 => {
let _ = { let assigned = JS_ToNumber(ctx, *(argv).offset(((0 as i32)) as isize)); v = assigned; assigned };
vm_block = 52; continue;
}
// C line 43840
54 => {
vm_block = match magic { x if x == (JS_ITERATOR_HELPER_KIND_MAP as i32) => 31, x if x == (JS_ITERATOR_HELPER_KIND_FLAT_MAP as i32) => 31, x if x == (JS_ITERATOR_HELPER_KIND_FILTER as i32) => 31, x if x == (JS_ITERATOR_HELPER_KIND_TAKE as i32) => 53, x if x == (JS_ITERATOR_HELPER_KIND_DROP as i32) => 53, _ => 27, }; continue;
}
// C line 43838
55 => {
let _ = { let assigned = (((0 as i32)) as i64); count = assigned; assigned };
vm_block = 54; continue;
}
// C line 43837
56 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; func = assigned; assigned };
vm_block = 55; continue;
}
// C line 43836
57 => {
return JS_ThrowTypeErrorNotAnObject(ctx);
}
// C line 43835
58 => {
vm_block = if ((!((JS_IsObject(this_val)) != 0) as i32)) != 0 { 57 } else { 56 }; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:43920. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_iterator_proto_func(mut ctx: *mut JSContext, mut this_val: JSValue, mut argc: i32, mut argv: *mut JSValue, mut magic: i32) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut item: JSValue = core::mem::zeroed();
let mut method: JSValue = core::mem::zeroed();
let mut ret: JSValue = core::mem::zeroed();
let mut func: JSValue = core::mem::zeroed();
let mut index_val: JSValue = core::mem::zeroed();
let mut r: JSValue = core::mem::zeroed();
let mut args: [JSValue; 2] = core::mem::zeroed();
let mut idx: i64 = core::mem::zeroed();
let mut done: i32 = core::mem::zeroed();
let mut vm_block: usize = 121;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 44072
1 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 44071
2 => {
let _ = JS_FreeValue(ctx, method);
vm_block = 1; continue;
}
// C line ? labels: fail_no_close
3 => {
let _ = JS_FreeValue(ctx, func);
vm_block = 2; continue;
}
// C line ? labels: fail
4 => {
let _ = JS_IteratorClose(ctx, this_val, (1 as i32));
vm_block = 3; continue;
}
// C line 44066
5 => {
return r;
}
// C line 44065
6 => {
let _ = JS_FreeValue(ctx, method);
vm_block = 5; continue;
}
// C line 44064
7 => {
let _ = JS_FreeValue(ctx, func);
vm_block = 6; continue;
}
// C line 44061
8 => {
vm_block = 7; continue;
}
// C line ?
9 => {
let _ = std::process::abort();
vm_block = 8; continue;
}
// C line 44058
10 => {
vm_block = 7; continue;
}
// C line 44032
11 => {
vm_block = 33; continue;
}
// C line ?
12 => {
let _ = { let old = idx; idx = (idx).wrapping_add(1); old };
vm_block = 11; continue;
}
// C line 44055
13 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; item = assigned; assigned };
vm_block = 12; continue;
}
// C line 44054
14 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; ret = assigned; assigned };
vm_block = 13; continue;
}
// C line 44053
15 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; index_val = assigned; assigned };
vm_block = 14; continue;
}
// C line 44051
16 => {
vm_block = 10; continue;
}
// C line 44048
17 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) }; r = assigned; assigned };
vm_block = 16; continue;
}
// C line 44050
18 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((1 as i32)) as u32)) as u64) }, tag: (((JS_TAG_BOOL as i32)) as i64) }; r = assigned; assigned };
vm_block = 16; continue;
}
// C line 44047
19 => {
vm_block = if ((((JS_IteratorClose(ctx, this_val, (0 as i32))) < ((0 as i32))) as i32)) != 0 { 17 } else { 18 }; continue;
}
// C line 44046
20 => {
vm_block = if (JS_ToBoolFree(ctx, ret)) != 0 { 19 } else { 15 }; continue;
}
// C line 44045
21 => {
vm_block = 4; continue;
}
// C line 44044
22 => {
vm_block = if (JS_IsException(ret)) != 0 { 21 } else { 20 }; continue;
}
// C line 44043
23 => {
let _ = JS_FreeValue(ctx, index_val);
vm_block = 22; continue;
}
// C line 44042
24 => {
let _ = JS_FreeValue(ctx, item);
vm_block = 23; continue;
}
// C line 44041
25 => {
let _ = { let assigned = JS_Call(ctx, func, JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }, (((((size_of::<[JSValue; 2]>() as usize)) / ((size_of::<JSValue>() as usize)))) as i32), (args).as_mut_ptr()); ret = assigned; assigned };
vm_block = 24; continue;
}
// C line 44040
26 => {
let _ = { let assigned = index_val; *((args).as_mut_ptr()).offset(((1 as i32)) as isize) = assigned; assigned };
vm_block = 25; continue;
}
// C line 44039
27 => {
let _ = { let assigned = item; *((args).as_mut_ptr()).offset(((0 as i32)) as isize) = assigned; assigned };
vm_block = 26; continue;
}
// C line 44038
28 => {
let _ = { let assigned = JS_NewInt64(ctx, idx); index_val = assigned; assigned };
vm_block = 27; continue;
}
// C line 44037
29 => {
vm_block = 10; continue;
}
// C line 44036
30 => {
vm_block = if (done) != 0 { 29 } else { 28 }; continue;
}
// C line 44035
31 => {
vm_block = 3; continue;
}
// C line 44034
32 => {
vm_block = if (JS_IsException(item)) != 0 { 31 } else { 30 }; continue;
}
// C line 44033
33 => {
let _ = { let assigned = JS_IteratorNext(ctx, this_val, method, (0 as i32), core::ptr::null_mut::<JSValue>(), core::ptr::addr_of_mut!(done)); item = assigned; assigned };
vm_block = 32; continue;
}
// C line 44032
34 => {
let _ = { let assigned = (((0 as i32)) as i64); idx = assigned; assigned };
vm_block = 11; continue;
}
// C line 44031
35 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_BOOL as i32)) as i64) }; r = assigned; assigned };
vm_block = 34; continue;
}
// C line 44028
36 => {
vm_block = 7; continue;
}
// C line 44008
37 => {
vm_block = 55; continue;
}
// C line ?
38 => {
let _ = { let old = idx; idx = (idx).wrapping_add(1); old };
vm_block = 37; continue;
}
// C line 44025
39 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; item = assigned; assigned };
vm_block = 38; continue;
}
// C line 44024
40 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; ret = assigned; assigned };
vm_block = 39; continue;
}
// C line 44023
41 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; index_val = assigned; assigned };
vm_block = 40; continue;
}
// C line 44022
42 => {
let _ = JS_FreeValue(ctx, ret);
vm_block = 41; continue;
}
// C line 44021
43 => {
vm_block = 4; continue;
}
// C line 44020
44 => {
vm_block = if (JS_IsException(ret)) != 0 { 43 } else { 42 }; continue;
}
// C line 44019
45 => {
let _ = JS_FreeValue(ctx, index_val);
vm_block = 44; continue;
}
// C line 44018
46 => {
let _ = JS_FreeValue(ctx, item);
vm_block = 45; continue;
}
// C line 44017
47 => {
let _ = { let assigned = JS_Call(ctx, func, JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }, (((((size_of::<[JSValue; 2]>() as usize)) / ((size_of::<JSValue>() as usize)))) as i32), (args).as_mut_ptr()); ret = assigned; assigned };
vm_block = 46; continue;
}
// C line 44016
48 => {
let _ = { let assigned = index_val; *((args).as_mut_ptr()).offset(((1 as i32)) as isize) = assigned; assigned };
vm_block = 47; continue;
}
// C line 44015
49 => {
let _ = { let assigned = item; *((args).as_mut_ptr()).offset(((0 as i32)) as isize) = assigned; assigned };
vm_block = 48; continue;
}
// C line 44014
50 => {
let _ = { let assigned = JS_NewInt64(ctx, idx); index_val = assigned; assigned };
vm_block = 49; continue;
}
// C line 44013
51 => {
vm_block = 36; continue;
}
// C line 44012
52 => {
vm_block = if (done) != 0 { 51 } else { 50 }; continue;
}
// C line 44011
53 => {
vm_block = 3; continue;
}
// C line 44010
54 => {
vm_block = if (JS_IsException(item)) != 0 { 53 } else { 52 }; continue;
}
// C line 44009
55 => {
let _ = { let assigned = JS_IteratorNext(ctx, this_val, method, (0 as i32), core::ptr::null_mut::<JSValue>(), core::ptr::addr_of_mut!(done)); item = assigned; assigned };
vm_block = 54; continue;
}
// C line 44008
56 => {
let _ = { let assigned = (((0 as i32)) as i64); idx = assigned; assigned };
vm_block = 37; continue;
}
// C line 44005
57 => {
vm_block = 7; continue;
}
// C line 43975
58 => {
vm_block = 82; continue;
}
// C line ?
59 => {
let _ = { let old = idx; idx = (idx).wrapping_add(1); old };
vm_block = 58; continue;
}
// C line 44002
60 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; item = assigned; assigned };
vm_block = 59; continue;
}
// C line 44001
61 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; ret = assigned; assigned };
vm_block = 60; continue;
}
// C line 44000
62 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; index_val = assigned; assigned };
vm_block = 61; continue;
}
// C line 43999
63 => {
let _ = JS_FreeValue(ctx, item);
vm_block = 62; continue;
}
// C line 43997
64 => {
vm_block = 57; continue;
}
// C line 43993
65 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) }; r = assigned; assigned };
vm_block = 64; continue;
}
// C line 43992
66 => {
let _ = JS_FreeValue(ctx, item);
vm_block = 65; continue;
}
// C line 43995
67 => {
let _ = { let assigned = item; r = assigned; assigned };
vm_block = 64; continue;
}
// C line 43991
68 => {
vm_block = if ((((JS_IteratorClose(ctx, this_val, (0 as i32))) < ((0 as i32))) as i32)) != 0 { 66 } else { 67 }; continue;
}
// C line 43990
69 => {
vm_block = if (JS_ToBoolFree(ctx, ret)) != 0 { 68 } else { 63 }; continue;
}
// C line 43988
70 => {
vm_block = 4; continue;
}
// C line 43987
71 => {
let _ = JS_FreeValue(ctx, item);
vm_block = 70; continue;
}
// C line 43986
72 => {
vm_block = if (JS_IsException(ret)) != 0 { 71 } else { 69 }; continue;
}
// C line 43985
73 => {
let _ = JS_FreeValue(ctx, index_val);
vm_block = 72; continue;
}
// C line 43984
74 => {
let _ = { let assigned = JS_Call(ctx, func, JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }, (((((size_of::<[JSValue; 2]>() as usize)) / ((size_of::<JSValue>() as usize)))) as i32), (args).as_mut_ptr()); ret = assigned; assigned };
vm_block = 73; continue;
}
// C line 43983
75 => {
let _ = { let assigned = index_val; *((args).as_mut_ptr()).offset(((1 as i32)) as isize) = assigned; assigned };
vm_block = 74; continue;
}
// C line 43982
76 => {
let _ = { let assigned = item; *((args).as_mut_ptr()).offset(((0 as i32)) as isize) = assigned; assigned };
vm_block = 75; continue;
}
// C line 43981
77 => {
let _ = { let assigned = JS_NewInt64(ctx, idx); index_val = assigned; assigned };
vm_block = 76; continue;
}
// C line 43980
78 => {
vm_block = 57; continue;
}
// C line 43979
79 => {
vm_block = if (done) != 0 { 78 } else { 77 }; continue;
}
// C line 43978
80 => {
vm_block = 3; continue;
}
// C line 43977
81 => {
vm_block = if (JS_IsException(item)) != 0 { 80 } else { 79 }; continue;
}
// C line 43976
82 => {
let _ = { let assigned = JS_IteratorNext(ctx, this_val, method, (0 as i32), core::ptr::null_mut::<JSValue>(), core::ptr::addr_of_mut!(done)); item = assigned; assigned };
vm_block = 81; continue;
}
// C line 43975
83 => {
let _ = { let assigned = (((0 as i32)) as i64); idx = assigned; assigned };
vm_block = 58; continue;
}
// C line 43972
84 => {
vm_block = 7; continue;
}
// C line 43946
85 => {
vm_block = 107; continue;
}
// C line ?
86 => {
let _ = { let old = idx; idx = (idx).wrapping_add(1); old };
vm_block = 85; continue;
}
// C line 43969
87 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; item = assigned; assigned };
vm_block = 86; continue;
}
// C line 43968
88 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; ret = assigned; assigned };
vm_block = 87; continue;
}
// C line 43967
89 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; index_val = assigned; assigned };
vm_block = 88; continue;
}
// C line 43965
90 => {
vm_block = 84; continue;
}
// C line 43962
91 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) }; r = assigned; assigned };
vm_block = 90; continue;
}
// C line 43964
92 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_BOOL as i32)) as i64) }; r = assigned; assigned };
vm_block = 90; continue;
}
// C line 43961
93 => {
vm_block = if ((((JS_IteratorClose(ctx, this_val, (0 as i32))) < ((0 as i32))) as i32)) != 0 { 91 } else { 92 }; continue;
}
// C line 43960
94 => {
vm_block = if ((!((JS_ToBoolFree(ctx, ret)) != 0) as i32)) != 0 { 93 } else { 89 }; continue;
}
// C line 43959
95 => {
vm_block = 4; continue;
}
// C line 43958
96 => {
vm_block = if (JS_IsException(ret)) != 0 { 95 } else { 94 }; continue;
}
// C line 43957
97 => {
let _ = JS_FreeValue(ctx, index_val);
vm_block = 96; continue;
}
// C line 43956
98 => {
let _ = JS_FreeValue(ctx, item);
vm_block = 97; continue;
}
// C line 43955
99 => {
let _ = { let assigned = JS_Call(ctx, func, JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }, (((((size_of::<[JSValue; 2]>() as usize)) / ((size_of::<JSValue>() as usize)))) as i32), (args).as_mut_ptr()); ret = assigned; assigned };
vm_block = 98; continue;
}
// C line 43954
100 => {
let _ = { let assigned = index_val; *((args).as_mut_ptr()).offset(((1 as i32)) as isize) = assigned; assigned };
vm_block = 99; continue;
}
// C line 43953
101 => {
let _ = { let assigned = item; *((args).as_mut_ptr()).offset(((0 as i32)) as isize) = assigned; assigned };
vm_block = 100; continue;
}
// C line 43952
102 => {
let _ = { let assigned = JS_NewInt64(ctx, idx); index_val = assigned; assigned };
vm_block = 101; continue;
}
// C line 43951
103 => {
vm_block = 84; continue;
}
// C line 43950
104 => {
vm_block = if (done) != 0 { 103 } else { 102 }; continue;
}
// C line 43949
105 => {
vm_block = 3; continue;
}
// C line 43948
106 => {
vm_block = if (JS_IsException(item)) != 0 { 105 } else { 104 }; continue;
}
// C line 43947
107 => {
let _ = { let assigned = JS_IteratorNext(ctx, this_val, method, (0 as i32), core::ptr::null_mut::<JSValue>(), core::ptr::addr_of_mut!(done)); item = assigned; assigned };
vm_block = 106; continue;
}
// C line 43946
108 => {
let _ = { let assigned = (((0 as i32)) as i64); idx = assigned; assigned };
vm_block = 85; continue;
}
// C line 43945
109 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((1 as i32)) as u32)) as u64) }, tag: (((JS_TAG_BOOL as i32)) as i64) }; r = assigned; assigned };
vm_block = 108; continue;
}
// C line 43942
110 => {
vm_block = match magic { x if x == (JS_ITERATOR_HELPER_KIND_SOME as i32) => 35, x if x == (JS_ITERATOR_HELPER_KIND_FOR_EACH as i32) => 56, x if x == (JS_ITERATOR_HELPER_KIND_FIND as i32) => 83, x if x == (JS_ITERATOR_HELPER_KIND_EVERY as i32) => 109, _ => 9, }; continue;
}
// C line 43940
111 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; r = assigned; assigned };
vm_block = 110; continue;
}
// C line 43938
112 => {
vm_block = 3; continue;
}
// C line 43937
113 => {
vm_block = if (JS_IsException(method)) != 0 { 112 } else { 111 }; continue;
}
// C line 43936
114 => {
let _ = { let assigned = JS_GetProperty(ctx, this_val, (((crate::quickjs_atom::JS_ATOM_next as i32)) as JSAtom)); method = assigned; assigned };
vm_block = 113; continue;
}
// C line 43935
115 => {
let _ = { let assigned = JS_DupValue(ctx, *(argv).offset(((0 as i32)) as isize)); func = assigned; assigned };
vm_block = 114; continue;
}
// C line 43934
116 => {
vm_block = 4; continue;
}
// C line 43933
117 => {
vm_block = if (check_function(ctx, *(argv).offset(((0 as i32)) as isize))) != 0 { 116 } else { 115 }; continue;
}
// C line 43931
118 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; method = assigned; assigned };
vm_block = 117; continue;
}
// C line 43930
119 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; func = assigned; assigned };
vm_block = 118; continue;
}
// C line 43929
120 => {
return JS_ThrowTypeErrorNotAnObject(ctx);
}
// C line 43928
121 => {
vm_block = if ((!((JS_IsObject(this_val)) != 0) as i32)) != 0 { 120 } else { 119 }; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:44075. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_iterator_proto_reduce(mut ctx: *mut JSContext, mut this_val: JSValue, mut argc: i32, mut argv: *mut JSValue) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut item: JSValue = core::mem::zeroed();
let mut method: JSValue = core::mem::zeroed();
let mut ret: JSValue = core::mem::zeroed();
let mut func: JSValue = core::mem::zeroed();
let mut index_val: JSValue = core::mem::zeroed();
let mut acc: JSValue = core::mem::zeroed();
let mut args: [JSValue; 3] = core::mem::zeroed();
let mut idx: i64 = core::mem::zeroed();
let mut done: i32 = core::mem::zeroed();
let mut vm_block: usize = 50;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 44137
1 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 44136
2 => {
let _ = JS_FreeValue(ctx, method);
vm_block = 1; continue;
}
// C line 44135
3 => {
let _ = JS_FreeValue(ctx, func);
vm_block = 2; continue;
}
// C line ? labels: exception_no_close
4 => {
let _ = JS_FreeValue(ctx, acc);
vm_block = 3; continue;
}
// C line ? labels: exception
5 => {
let _ = JS_IteratorClose(ctx, this_val, (1 as i32));
vm_block = 4; continue;
}
// C line 44130
6 => {
return acc;
}
// C line 44129
7 => {
let _ = JS_FreeValue(ctx, method);
vm_block = 6; continue;
}
// C line 44128
8 => {
let _ = JS_FreeValue(ctx, func);
vm_block = 7; continue;
}
// C line 44107
9 => {
vm_block = 29; continue;
}
// C line 44107
10 => {
let _ = { let old = idx; idx = (idx).wrapping_add(1); old };
vm_block = 9; continue;
}
// C line 44126
11 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; item = assigned; assigned };
vm_block = 10; continue;
}
// C line 44125
12 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; ret = assigned; assigned };
vm_block = 11; continue;
}
// C line 44124
13 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; index_val = assigned; assigned };
vm_block = 12; continue;
}
// C line 44123
14 => {
let _ = { let assigned = ret; acc = assigned; assigned };
vm_block = 13; continue;
}
// C line 44122
15 => {
let _ = JS_FreeValue(ctx, acc);
vm_block = 14; continue;
}
// C line 44121
16 => {
vm_block = 5; continue;
}
// C line 44120
17 => {
vm_block = if (JS_IsException(ret)) != 0 { 16 } else { 15 }; continue;
}
// C line 44119
18 => {
let _ = JS_FreeValue(ctx, index_val);
vm_block = 17; continue;
}
// C line 44118
19 => {
let _ = JS_FreeValue(ctx, item);
vm_block = 18; continue;
}
// C line 44117
20 => {
let _ = { let assigned = JS_Call(ctx, func, JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }, (((((size_of::<[JSValue; 3]>() as usize)) / ((size_of::<JSValue>() as usize)))) as i32), (args).as_mut_ptr()); ret = assigned; assigned };
vm_block = 19; continue;
}
// C line 44116
21 => {
let _ = { let assigned = index_val; *((args).as_mut_ptr()).offset(((2 as i32)) as isize) = assigned; assigned };
vm_block = 20; continue;
}
// C line 44115
22 => {
let _ = { let assigned = item; *((args).as_mut_ptr()).offset(((1 as i32)) as isize) = assigned; assigned };
vm_block = 21; continue;
}
// C line 44114
23 => {
let _ = { let assigned = acc; *((args).as_mut_ptr()).offset(((0 as i32)) as isize) = assigned; assigned };
vm_block = 22; continue;
}
// C line 44113
24 => {
let _ = { let assigned = JS_NewInt64(ctx, idx); index_val = assigned; assigned };
vm_block = 23; continue;
}
// C line 44112
25 => {
vm_block = 8; continue;
}
// C line 44111
26 => {
vm_block = if (done) != 0 { 25 } else { 24 }; continue;
}
// C line 44110
27 => {
vm_block = 4; continue;
}
// C line 44109
28 => {
vm_block = if (JS_IsException(item)) != 0 { 27 } else { 26 }; continue;
}
// C line 44108
29 => {
let _ = { let assigned = JS_IteratorNext(ctx, this_val, method, (0 as i32), core::ptr::null_mut::<JSValue>(), core::ptr::addr_of_mut!(done)); item = assigned; assigned };
vm_block = 28; continue;
}
// C line 44096
30 => {
let _ = { let assigned = (((0 as i32)) as i64); idx = assigned; assigned };
vm_block = 9; continue;
}
// C line 44095
31 => {
let _ = { let assigned = JS_DupValue(ctx, *(argv).offset(((1 as i32)) as isize)); acc = assigned; assigned };
vm_block = 30; continue;
}
// C line 44105
32 => {
let _ = { let assigned = (((1 as i32)) as i64); idx = assigned; assigned };
vm_block = 9; continue;
}
// C line 44103
33 => {
vm_block = 5; continue;
}
// C line 44102
34 => {
let _ = JS_ThrowTypeError(ctx, c"empty iterator".as_ptr());
vm_block = 33; continue;
}
// C line 44101
35 => {
vm_block = if (done) != 0 { 34 } else { 32 }; continue;
}
// C line 44100
36 => {
vm_block = 4; continue;
}
// C line 44099
37 => {
vm_block = if (JS_IsException(acc)) != 0 { 36 } else { 35 }; continue;
}
// C line 44098
38 => {
let _ = { let assigned = JS_IteratorNext(ctx, this_val, method, (0 as i32), core::ptr::null_mut::<JSValue>(), core::ptr::addr_of_mut!(done)); acc = assigned; assigned };
vm_block = 37; continue;
}
// C line 44094
39 => {
vm_block = if ((((argc) > ((1 as i32))) as i32)) != 0 { 31 } else { 38 }; continue;
}
// C line 44093
40 => {
vm_block = 5; continue;
}
// C line 44092
41 => {
vm_block = if (JS_IsException(method)) != 0 { 40 } else { 39 }; continue;
}
// C line 44091
42 => {
let _ = { let assigned = JS_GetProperty(ctx, this_val, (((crate::quickjs_atom::JS_ATOM_next as i32)) as JSAtom)); method = assigned; assigned };
vm_block = 41; continue;
}
// C line 44090
43 => {
let _ = { let assigned = JS_DupValue(ctx, *(argv).offset(((0 as i32)) as isize)); func = assigned; assigned };
vm_block = 42; continue;
}
// C line 44089
44 => {
vm_block = 5; continue;
}
// C line 44088
45 => {
vm_block = if (check_function(ctx, *(argv).offset(((0 as i32)) as isize))) != 0 { 44 } else { 43 }; continue;
}
// C line 44087
46 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; method = assigned; assigned };
vm_block = 45; continue;
}
// C line 44086
47 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; func = assigned; assigned };
vm_block = 46; continue;
}
// C line 44085
48 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; acc = assigned; assigned };
vm_block = 47; continue;
}
// C line 44084
49 => {
return JS_ThrowTypeErrorNotAnObject(ctx);
}
// C line 44083
50 => {
vm_block = if ((!((JS_IsObject(this_val)) != 0) as i32)) != 0 { 49 } else { 48 }; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:44140. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_iterator_proto_toArray(mut ctx: *mut JSContext, mut this_val: JSValue, mut argc: i32, mut argv: *mut JSValue) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut item: JSValue = core::mem::zeroed();
let mut method: JSValue = core::mem::zeroed();
let mut result: JSValue = core::mem::zeroed();
let mut idx: i64 = core::mem::zeroed();
let mut done: i32 = core::mem::zeroed();
let mut vm_block: usize = 26;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 44173
1 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 44172
2 => {
let _ = JS_FreeValue(ctx, method);
vm_block = 1; continue;
}
// C line ? labels: exception
3 => {
let _ = JS_FreeValue(ctx, result);
vm_block = 2; continue;
}
// C line 44169
4 => {
return result;
}
// C line 44168
5 => {
let _ = JS_FreeValue(ctx, method);
vm_block = 4; continue;
}
// C line 44167
6 => {
vm_block = 3; continue;
}
// C line 44166
7 => {
vm_block = if ((((JS_SetProperty(ctx, result, (((crate::quickjs_atom::JS_ATOM_length as i32)) as JSAtom), JS_NewUint32(ctx, ((idx) as u32)))) < ((0 as i32))) as i32)) != 0 { 6 } else { 5 }; continue;
}
// C line 44156
8 => {
vm_block = 16; continue;
}
// C line ?
9 => {
let _ = { let old = idx; idx = (idx).wrapping_add(1); old };
vm_block = 8; continue;
}
// C line 44164
10 => {
vm_block = 3; continue;
}
// C line 44162
11 => {
vm_block = if ((((JS_DefinePropertyValueInt64(ctx, result, idx, item, ((((((((1 as i32)).wrapping_shl(((0 as i32)) as u32)) | (((1 as i32)).wrapping_shl(((1 as i32)) as u32)))) | (((1 as i32)).wrapping_shl(((2 as i32)) as u32)))) | (((1 as i32)).wrapping_shl(((14 as i32)) as u32))))) < ((0 as i32))) as i32)) != 0 { 10 } else { 9 }; continue;
}
// C line 44161
12 => {
vm_block = 7; continue;
}
// C line 44160
13 => {
vm_block = if (done) != 0 { 12 } else { 11 }; continue;
}
// C line 44159
14 => {
vm_block = 3; continue;
}
// C line 44158
15 => {
vm_block = if (JS_IsException(item)) != 0 { 14 } else { 13 }; continue;
}
// C line 44157
16 => {
let _ = { let assigned = JS_IteratorNext(ctx, this_val, method, (0 as i32), core::ptr::null_mut::<JSValue>(), core::ptr::addr_of_mut!(done)); item = assigned; assigned };
vm_block = 15; continue;
}
// C line 44156
17 => {
let _ = { let assigned = (((0 as i32)) as i64); idx = assigned; assigned };
vm_block = 8; continue;
}
// C line 44155
18 => {
vm_block = 3; continue;
}
// C line 44154
19 => {
vm_block = if (JS_IsException(result)) != 0 { 18 } else { 17 }; continue;
}
// C line 44153
20 => {
let _ = { let assigned = JS_NewArray(ctx); result = assigned; assigned };
vm_block = 19; continue;
}
// C line 44152
21 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 44151
22 => {
vm_block = if (JS_IsException(method)) != 0 { 21 } else { 20 }; continue;
}
// C line 44150
23 => {
let _ = { let assigned = JS_GetProperty(ctx, this_val, (((crate::quickjs_atom::JS_ATOM_next as i32)) as JSAtom)); method = assigned; assigned };
vm_block = 22; continue;
}
// C line 44149
24 => {
return JS_ThrowTypeErrorNotAnObject(ctx);
}
// C line 44148
25 => {
vm_block = if ((!((JS_IsObject(this_val)) != 0) as i32)) != 0 { 24 } else { 23 }; continue;
}
// C line 44147
26 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; result = assigned; assigned };
vm_block = 25; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:44176. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_iterator_proto_iterator(mut ctx: *mut JSContext, mut this_val: JSValue, mut argc: i32, mut argv: *mut JSValue) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut vm_block: usize = 1;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 44179
1 => {
return JS_DupValue(ctx, this_val);
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:44182. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_iterator_proto_get_toStringTag(mut ctx: *mut JSContext, mut this_val: JSValue) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut vm_block: usize = 1;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 44184
1 => {
return JS_AtomToString(ctx, (((crate::quickjs_atom::JS_ATOM_Iterator as i32)) as JSAtom));
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:44187. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_iterator_proto_set_toStringTag(mut ctx: *mut JSContext, mut this_val: JSValue, mut val: JSValue) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut res: i32 = core::mem::zeroed();
let mut vm_block: usize = 13;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 44205
1 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) };
}
// C line 44200
2 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 44199
3 => {
vm_block = if ((((JS_SetProperty(ctx, this_val, (((crate::quickjs_atom::JS_ATOM_Symbol_toStringTag as i32)) as JSAtom), JS_DupValue(ctx, val))) < ((0 as i32))) as i32)) != 0 { 2 } else { 1 }; continue;
}
// C line 44203
4 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 44202
5 => {
vm_block = if ((((JS_DefinePropertyValue(ctx, this_val, (((crate::quickjs_atom::JS_ATOM_Symbol_toStringTag as i32)) as JSAtom), JS_DupValue(ctx, val), ((((((1 as i32)).wrapping_shl(((0 as i32)) as u32)) | (((1 as i32)).wrapping_shl(((1 as i32)) as u32)))) | (((1 as i32)).wrapping_shl(((2 as i32)) as u32))))) < ((0 as i32))) as i32)) != 0 { 4 } else { 1 }; continue;
}
// C line 44198
6 => {
vm_block = if (res) != 0 { 3 } else { 5 }; continue;
}
// C line 44197
7 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 44196
8 => {
vm_block = if ((((res) < ((0 as i32))) as i32)) != 0 { 7 } else { 6 }; continue;
}
// C line 44195
9 => {
let _ = { let assigned = JS_GetOwnProperty(ctx, core::ptr::null_mut::<JSPropertyDescriptor>(), this_val, (((crate::quickjs_atom::JS_ATOM_Symbol_toStringTag as i32)) as JSAtom)); res = assigned; assigned };
vm_block = 8; continue;
}
// C line 44194
10 => {
return JS_ThrowTypeError(ctx, c"Cannot assign to read only property".as_ptr());
}
// C line 44193
11 => {
vm_block = if (js_same_value(ctx, this_val, *((*(ctx)).class_proto).offset(((JS_CLASS_ITERATOR as i32)) as isize))) != 0 { 10 } else { 9 }; continue;
}
// C line 44192
12 => {
return JS_ThrowTypeErrorNotAnObject(ctx);
}
// C line 44191
13 => {
vm_block = if ((!((JS_IsObject(this_val)) != 0) as i32)) != 0 { 12 } else { 11 }; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:44236. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_iterator_helper_next(mut ctx: *mut JSContext, mut this_val: JSValue, mut argc: i32, mut argv: *mut JSValue, mut pdone: *mut i32, mut magic: i32) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut it: *mut JSIteratorHelperData = core::mem::zeroed();
let mut ret: JSValue = core::mem::zeroed();
let mut item: JSValue = core::mem::zeroed();
let mut method: JSValue = core::mem::zeroed();
let mut item_1: JSValue = core::mem::zeroed();
let mut method_1: JSValue = core::mem::zeroed();
let mut selected: JSValue = core::mem::zeroed();
let mut index_val: JSValue = core::mem::zeroed();
let mut args: [JSValue; 2] = core::mem::zeroed();
let mut item_2: JSValue = core::mem::zeroed();
let mut method_2: JSValue = core::mem::zeroed();
let mut index_val_1: JSValue = core::mem::zeroed();
let mut iter: JSValue = core::mem::zeroed();
let mut args_1: [JSValue; 2] = core::mem::zeroed();
let mut item_3: JSValue = core::mem::zeroed();
let mut method_3: JSValue = core::mem::zeroed();
let mut index_val_2: JSValue = core::mem::zeroed();
let mut args_2: [JSValue; 2] = core::mem::zeroed();
let mut item_4: JSValue = core::mem::zeroed();
let mut method_4: JSValue = core::mem::zeroed();
let mut vm_block: usize = 174;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 44489
1 => {
vm_block = 6; continue;
}
// C line ? labels: fail_no_close
2 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) }; ret = assigned; assigned };
vm_block = 1; continue;
}
// C line ? labels: fail
3 => {
let _ = JS_IteratorClose(ctx, (*(it)).obj, (1 as i32));
vm_block = 2; continue;
}
// C line 44483
4 => {
return ret;
}
// C line 44482
5 => {
let _ = { let assigned = (((0 as i32)) as u8); (*(it)).set_executing((assigned) as _); assigned };
vm_block = 4; continue;
}
// C line ? labels: done
6 => {
let _ = { let assigned = ((if ((((magic) == ((0 as i32))) as i32)) != 0 { *(pdone) } else { (1 as i32) }) as u8); (*(it)).set_done((assigned) as _); assigned };
vm_block = 5; continue;
}
// C line ?
7 => {
let _ = std::process::abort();
vm_block = 6; continue;
}
// C line 44475
8 => {
vm_block = 6; continue;
}
// C line 44473
9 => {
vm_block = 6; continue;
}
// C line 44470
10 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) }; ret = assigned; assigned };
vm_block = 9; continue;
}
// C line 44472
11 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; ret = assigned; assigned };
vm_block = 9; continue;
}
// C line 44469
12 => {
vm_block = if (JS_IteratorClose(ctx, (*(it)).obj, (0 as i32))) != 0 { 10 } else { 11 }; continue;
}
// C line 44468
13 => {
let _ = { let assigned = (1 as i32); *(pdone) = assigned; assigned };
vm_block = 12; continue;
}
// C line 44465
14 => {
vm_block = 6; continue;
}
// C line 44464
15 => {
let _ = { let assigned = item_4; ret = assigned; assigned };
vm_block = 14; continue;
}
// C line 44463
16 => {
vm_block = 2; continue;
}
// C line 44462
17 => {
vm_block = if (JS_IsException(item_4)) != 0 { 16 } else { 15 }; continue;
}
// C line 44461
18 => {
let _ = JS_FreeValue(ctx, method_4);
vm_block = 17; continue;
}
// C line 44460
19 => {
let _ = { let assigned = JS_IteratorNext(ctx, (*(it)).obj, method_4, (0 as i32), core::ptr::null_mut::<JSValue>(), pdone); item_4 = assigned; assigned };
vm_block = 18; continue;
}
// C line 44459
20 => {
let _ = { let old = (*(it)).count; (*(it)).count = ((*(it)).count).wrapping_sub(1); old };
vm_block = 19; continue;
}
// C line 44453
21 => {
let _ = { let assigned = JS_DupValue(ctx, (*(it)).next); method_4 = assigned; assigned };
vm_block = 20; continue;
}
// C line 44457
22 => {
vm_block = 3; continue;
}
// C line 44456
23 => {
vm_block = if (JS_IsException(method_4)) != 0 { 22 } else { 20 }; continue;
}
// C line 44455
24 => {
let _ = { let assigned = JS_GetProperty(ctx, (*(it)).obj, (((crate::quickjs_atom::JS_ATOM_return as i32)) as JSAtom)); method_4 = assigned; assigned };
vm_block = 23; continue;
}
// C line 44452
25 => {
vm_block = if ((((magic) == ((0 as i32))) as i32)) != 0 { 21 } else { 24 }; continue;
}
// C line 44451
26 => {
vm_block = if (((((*(it)).count) > ((((0 as i32)) as i64))) as i32)) != 0 { 25 } else { 13 }; continue;
}
// C line 44447
27 => {
vm_block = 6; continue;
}
// C line 44445
28 => {
vm_block = 6; continue;
}
// C line 44444
29 => {
vm_block = 3; continue;
}
// C line 44443
30 => {
vm_block = if (JS_IsException(ret)) != 0 { 29 } else { 28 }; continue;
}
// C line 44442
31 => {
let _ = JS_FreeValue(ctx, item_3);
vm_block = 30; continue;
}
// C line 44441
32 => {
let _ = JS_FreeValue(ctx, index_val_2);
vm_block = 31; continue;
}
// C line 44440
33 => {
let _ = { let assigned = JS_Call(ctx, (*(it)).func, JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }, (((((size_of::<[JSValue; 2]>() as usize)) / ((size_of::<JSValue>() as usize)))) as i32), (args_2).as_mut_ptr()); ret = assigned; assigned };
vm_block = 32; continue;
}
// C line 44439
34 => {
let _ = { let assigned = index_val_2; *((args_2).as_mut_ptr()).offset(((1 as i32)) as isize) = assigned; assigned };
vm_block = 33; continue;
}
// C line 44438
35 => {
let _ = { let assigned = item_3; *((args_2).as_mut_ptr()).offset(((0 as i32)) as isize) = assigned; assigned };
vm_block = 34; continue;
}
// C line 44437
36 => {
let _ = { let assigned = JS_NewInt64(ctx, { let old = (*(it)).count; (*(it)).count = ((*(it)).count).wrapping_add(1); old }); index_val_2 = assigned; assigned };
vm_block = 35; continue;
}
// C line 44435
37 => {
vm_block = 6; continue;
}
// C line 44434
38 => {
let _ = { let assigned = item_3; ret = assigned; assigned };
vm_block = 37; continue;
}
// C line 44433
39 => {
vm_block = if (((((*(pdone)) != 0) || (((((magic) == ((1 as i32))) as i32)) != 0)) as i32)) != 0 { 38 } else { 36 }; continue;
}
// C line 44432
40 => {
vm_block = 2; continue;
}
// C line 44431
41 => {
vm_block = if (JS_IsException(item_3)) != 0 { 40 } else { 39 }; continue;
}
// C line 44430
42 => {
let _ = JS_FreeValue(ctx, method_3);
vm_block = 41; continue;
}
// C line 44429
43 => {
let _ = { let assigned = JS_IteratorNext(ctx, (*(it)).obj, method_3, (0 as i32), core::ptr::null_mut::<JSValue>(), pdone); item_3 = assigned; assigned };
vm_block = 42; continue;
}
// C line 44423
44 => {
let _ = { let assigned = JS_DupValue(ctx, (*(it)).next); method_3 = assigned; assigned };
vm_block = 43; continue;
}
// C line 44427
45 => {
vm_block = 3; continue;
}
// C line 44426
46 => {
vm_block = if (JS_IsException(method_3)) != 0 { 45 } else { 43 }; continue;
}
// C line 44425
47 => {
let _ = { let assigned = JS_GetProperty(ctx, (*(it)).obj, (((crate::quickjs_atom::JS_ATOM_return as i32)) as JSAtom)); method_3 = assigned; assigned };
vm_block = 46; continue;
}
// C line 44422
48 => {
vm_block = if ((((magic) == ((0 as i32))) as i32)) != 0 { 44 } else { 47 }; continue;
}
// C line 44417
49 => {
vm_block = 6; continue;
}
// C line 44415
50 => {
vm_block = 6; continue;
}
// C line 44414
51 => {
let _ = { let assigned = item_2; ret = assigned; assigned };
vm_block = 50; continue;
}
// C line 44412
52 => {
vm_block = 109; continue;
}
// C line 44411
53 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; (*(it)).inner = assigned; assigned };
vm_block = 52; continue;
}
// C line 44410
54 => {
let _ = JS_FreeValue(ctx, (*(it)).inner);
vm_block = 53; continue;
}
// C line 44409
55 => {
let _ = JS_IteratorClose(ctx, (*(it)).inner, (0 as i32));
vm_block = 54; continue;
}
// C line ? labels: inner_end
56 => {
let _ = { let assigned = (0 as i32); *(pdone) = assigned; assigned };
vm_block = 55; continue;
}
// C line 44406
57 => {
vm_block = if (*(pdone)) != 0 { 56 } else { 51 }; continue;
}
// C line 44399
58 => {
vm_block = 56; continue;
}
// C line 44404
59 => {
vm_block = 67; continue;
}
// C line 44403
60 => {
vm_block = if (JS_IsException(item_2)) != 0 { 59 } else { 57 }; continue;
}
// C line 44402
61 => {
let _ = JS_FreeValue(ctx, method_2);
vm_block = 60; continue;
}
// C line 44401
62 => {
let _ = { let assigned = JS_IteratorNext(ctx, (*(it)).inner, method_2, (0 as i32), core::ptr::null_mut::<JSValue>(), pdone); item_2 = assigned; assigned };
vm_block = 61; continue;
}
// C line 44398
63 => {
vm_block = if ((((((((magic) == ((1 as i32))) as i32)) != 0) && ((((((JS_IsUndefined(method_2)) != 0) || ((JS_IsNull(method_2)) != 0)) as i32)) != 0)) as i32)) != 0 { 58 } else { 62 }; continue;
}
// C line 44396
64 => {
vm_block = 3; continue;
}
// C line 44395
65 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; (*(it)).inner = assigned; assigned };
vm_block = 64; continue;
}
// C line 44394
66 => {
let _ = JS_FreeValue(ctx, (*(it)).inner);
vm_block = 65; continue;
}
// C line ? labels: inner_fail
67 => {
let _ = JS_IteratorClose(ctx, (*(it)).inner, (0 as i32));
vm_block = 66; continue;
}
// C line 44391
68 => {
vm_block = if (JS_IsException(method_2)) != 0 { 67 } else { 63 }; continue;
}
// C line 44388
69 => {
let _ = { let assigned = JS_GetProperty(ctx, (*(it)).inner, (((crate::quickjs_atom::JS_ATOM_next as i32)) as JSAtom)); method_2 = assigned; assigned };
vm_block = 68; continue;
}
// C line 44390
70 => {
let _ = { let assigned = JS_GetProperty(ctx, (*(it)).inner, (((crate::quickjs_atom::JS_ATOM_return as i32)) as JSAtom)); method_2 = assigned; assigned };
vm_block = 68; continue;
}
// C line 44387
71 => {
vm_block = if ((((magic) == ((0 as i32))) as i32)) != 0 { 69 } else { 70 }; continue;
}
// C line 44384
72 => {
let _ = { let assigned = iter; (*(it)).inner = assigned; assigned };
vm_block = 71; continue;
}
// C line 44375
73 => {
let _ = { let assigned = ret; iter = assigned; assigned };
vm_block = 72; continue;
}
// C line 44374
74 => {
let _ = JS_FreeValue(ctx, method_2);
vm_block = 73; continue;
}
// C line 44381
75 => {
vm_block = 3; continue;
}
// C line 44380
76 => {
vm_block = if (JS_IsException(iter)) != 0 { 75 } else { 72 }; continue;
}
// C line 44379
77 => {
let _ = JS_FreeValue(ctx, ret);
vm_block = 76; continue;
}
// C line 44378
78 => {
let _ = JS_FreeValue(ctx, method_2);
vm_block = 77; continue;
}
// C line 44377
79 => {
let _ = { let assigned = JS_GetIterator2(ctx, ret, method_2); iter = assigned; assigned };
vm_block = 78; continue;
}
// C line 44373
80 => {
vm_block = if (((((JS_IsNull(method_2)) != 0) || ((JS_IsUndefined(method_2)) != 0)) as i32)) != 0 { 74 } else { 79 }; continue;
}
// C line 44371
81 => {
vm_block = 3; continue;
}
// C line 44370
82 => {
let _ = JS_FreeValue(ctx, ret);
vm_block = 81; continue;
}
// C line 44369
83 => {
vm_block = if (JS_IsException(method_2)) != 0 { 82 } else { 80 }; continue;
}
// C line 44368
84 => {
let _ = { let assigned = JS_GetProperty(ctx, ret, (((crate::quickjs_atom::JS_ATOM_Symbol_iterator as i32)) as JSAtom)); method_2 = assigned; assigned };
vm_block = 83; continue;
}
// C line 44366
85 => {
vm_block = 3; continue;
}
// C line 44365
86 => {
let _ = JS_ThrowTypeError(ctx, c"not an object".as_ptr());
vm_block = 85; continue;
}
// C line 44364
87 => {
let _ = JS_FreeValue(ctx, ret);
vm_block = 86; continue;
}
// C line 44363
88 => {
vm_block = if ((!((JS_IsObject(ret)) != 0) as i32)) != 0 { 87 } else { 84 }; continue;
}
// C line 44362
89 => {
vm_block = 3; continue;
}
// C line 44361
90 => {
vm_block = if (JS_IsException(ret)) != 0 { 89 } else { 88 }; continue;
}
// C line 44360
91 => {
let _ = JS_FreeValue(ctx, index_val_1);
vm_block = 90; continue;
}
// C line 44359
92 => {
let _ = JS_FreeValue(ctx, item_2);
vm_block = 91; continue;
}
// C line 44358
93 => {
let _ = { let assigned = JS_Call(ctx, (*(it)).func, JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }, (((((size_of::<[JSValue; 2]>() as usize)) / ((size_of::<JSValue>() as usize)))) as i32), (args_1).as_mut_ptr()); ret = assigned; assigned };
vm_block = 92; continue;
}
// C line 44357
94 => {
let _ = { let assigned = index_val_1; *((args_1).as_mut_ptr()).offset(((1 as i32)) as isize) = assigned; assigned };
vm_block = 93; continue;
}
// C line 44356
95 => {
let _ = { let assigned = item_2; *((args_1).as_mut_ptr()).offset(((0 as i32)) as isize) = assigned; assigned };
vm_block = 94; continue;
}
// C line 44355
96 => {
let _ = { let assigned = JS_NewInt64(ctx, { let old = (*(it)).count; (*(it)).count = ((*(it)).count).wrapping_add(1); old }); index_val_1 = assigned; assigned };
vm_block = 95; continue;
}
// C line 44353
97 => {
vm_block = 6; continue;
}
// C line 44352
98 => {
let _ = { let assigned = item_2; ret = assigned; assigned };
vm_block = 97; continue;
}
// C line 44351
99 => {
vm_block = if (((((*(pdone)) != 0) || (((((magic) == ((1 as i32))) as i32)) != 0)) as i32)) != 0 { 98 } else { 96 }; continue;
}
// C line 44350
100 => {
vm_block = 2; continue;
}
// C line 44349
101 => {
vm_block = if (JS_IsException(item_2)) != 0 { 100 } else { 99 }; continue;
}
// C line 44348
102 => {
let _ = JS_FreeValue(ctx, method_2);
vm_block = 101; continue;
}
// C line 44347
103 => {
let _ = { let assigned = JS_IteratorNext(ctx, (*(it)).obj, method_2, (0 as i32), core::ptr::null_mut::<JSValue>(), pdone); item_2 = assigned; assigned };
vm_block = 102; continue;
}
// C line 44341
104 => {
let _ = { let assigned = JS_DupValue(ctx, (*(it)).next); method_2 = assigned; assigned };
vm_block = 103; continue;
}
// C line 44345
105 => {
vm_block = 3; continue;
}
// C line 44344
106 => {
vm_block = if (JS_IsException(method_2)) != 0 { 105 } else { 103 }; continue;
}
// C line 44343
107 => {
let _ = { let assigned = JS_GetProperty(ctx, (*(it)).obj, (((crate::quickjs_atom::JS_ATOM_return as i32)) as JSAtom)); method_2 = assigned; assigned };
vm_block = 106; continue;
}
// C line 44340
108 => {
vm_block = if ((((magic) == ((0 as i32))) as i32)) != 0 { 104 } else { 107 }; continue;
}
// C line 44339 labels: flat_map_again
109 => {
vm_block = if (JS_IsUndefined((*(it)).inner)) != 0 { 108 } else { 71 }; continue;
}
// C line 44333
110 => {
vm_block = 6; continue;
}
// C line 44331
111 => {
vm_block = 133; continue;
}
// C line 44330
112 => {
let _ = JS_FreeValue(ctx, item_1);
vm_block = 111; continue;
}
// C line 44328
113 => {
vm_block = 6; continue;
}
// C line 44327
114 => {
let _ = { let assigned = item_1; ret = assigned; assigned };
vm_block = 113; continue;
}
// C line 44326
115 => {
let _ = JS_FreeValue(ctx, method_1);
vm_block = 114; continue;
}
// C line 44325
116 => {
vm_block = if (JS_ToBoolFree(ctx, selected)) != 0 { 115 } else { 112 }; continue;
}
// C line 44323
117 => {
vm_block = 3; continue;
}
// C line 44322
118 => {
let _ = JS_FreeValue(ctx, method_1);
vm_block = 117; continue;
}
// C line 44321
119 => {
let _ = JS_FreeValue(ctx, item_1);
vm_block = 118; continue;
}
// C line 44320
120 => {
vm_block = if (JS_IsException(selected)) != 0 { 119 } else { 116 }; continue;
}
// C line 44319
121 => {
let _ = JS_FreeValue(ctx, index_val);
vm_block = 120; continue;
}
// C line 44318
122 => {
let _ = { let assigned = JS_Call(ctx, (*(it)).func, JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }, (((((size_of::<[JSValue; 2]>() as usize)) / ((size_of::<JSValue>() as usize)))) as i32), (args).as_mut_ptr()); selected = assigned; assigned };
vm_block = 121; continue;
}
// C line 44317
123 => {
let _ = { let assigned = index_val; *((args).as_mut_ptr()).offset(((1 as i32)) as isize) = assigned; assigned };
vm_block = 122; continue;
}
// C line 44316
124 => {
let _ = { let assigned = item_1; *((args).as_mut_ptr()).offset(((0 as i32)) as isize) = assigned; assigned };
vm_block = 123; continue;
}
// C line 44315
125 => {
let _ = { let assigned = JS_NewInt64(ctx, { let old = (*(it)).count; (*(it)).count = ((*(it)).count).wrapping_add(1); old }); index_val = assigned; assigned };
vm_block = 124; continue;
}
// C line 44313
126 => {
vm_block = 6; continue;
}
// C line 44312
127 => {
let _ = { let assigned = item_1; ret = assigned; assigned };
vm_block = 126; continue;
}
// C line 44311
128 => {
let _ = JS_FreeValue(ctx, method_1);
vm_block = 127; continue;
}
// C line 44310
129 => {
vm_block = if (((((*(pdone)) != 0) || (((((magic) == ((1 as i32))) as i32)) != 0)) as i32)) != 0 { 128 } else { 125 }; continue;
}
// C line 44308
130 => {
vm_block = 2; continue;
}
// C line 44307
131 => {
let _ = JS_FreeValue(ctx, method_1);
vm_block = 130; continue;
}
// C line 44306
132 => {
vm_block = if (JS_IsException(item_1)) != 0 { 131 } else { 129 }; continue;
}
// C line ? labels: filter_again
133 => {
let _ = { let assigned = JS_IteratorNext(ctx, (*(it)).obj, method_1, (0 as i32), core::ptr::null_mut::<JSValue>(), pdone); item_1 = assigned; assigned };
vm_block = 132; continue;
}
// C line 44298
134 => {
let _ = { let assigned = JS_DupValue(ctx, (*(it)).next); method_1 = assigned; assigned };
vm_block = 133; continue;
}
// C line 44302
135 => {
vm_block = 3; continue;
}
// C line 44301
136 => {
vm_block = if (JS_IsException(method_1)) != 0 { 135 } else { 133 }; continue;
}
// C line 44300
137 => {
let _ = { let assigned = JS_GetProperty(ctx, (*(it)).obj, (((crate::quickjs_atom::JS_ATOM_return as i32)) as JSAtom)); method_1 = assigned; assigned };
vm_block = 136; continue;
}
// C line 44297
138 => {
vm_block = if ((((magic) == ((0 as i32))) as i32)) != 0 { 134 } else { 137 }; continue;
}
// C line 44292
139 => {
vm_block = 6; continue;
}
// C line 44290
140 => {
vm_block = 6; continue;
}
// C line 44289
141 => {
let _ = { let assigned = item; ret = assigned; assigned };
vm_block = 140; continue;
}
// C line 44288
142 => {
vm_block = 2; continue;
}
// C line 44287
143 => {
vm_block = if (JS_IsException(item)) != 0 { 142 } else { 141 }; continue;
}
// C line 44286
144 => {
let _ = JS_FreeValue(ctx, method);
vm_block = 143; continue;
}
// C line 44285
145 => {
let _ = { let assigned = JS_IteratorNext(ctx, (*(it)).obj, method, (0 as i32), core::ptr::null_mut::<JSValue>(), pdone); item = assigned; assigned };
vm_block = 144; continue;
}
// C line 44268
146 => {
vm_block = if (((((*(it)).count) > ((((0 as i32)) as i64))) as i32)) != 0 { 158 } else { 145 }; continue;
}
// C line 44281
147 => {
vm_block = 6; continue;
}
// C line 44280
148 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; ret = assigned; assigned };
vm_block = 147; continue;
}
// C line 44279
149 => {
let _ = JS_FreeValue(ctx, method);
vm_block = 148; continue;
}
// C line 44278
150 => {
vm_block = if (*(pdone)) != 0 { 149 } else { 146 }; continue;
}
// C line 44277
151 => {
let _ = { let assigned = (1 as i32); *(pdone) = assigned; assigned };
vm_block = 150; continue;
}
// C line 44276
152 => {
vm_block = if ((((magic) == ((1 as i32))) as i32)) != 0 { 151 } else { 150 }; continue;
}
// C line 44275
153 => {
let _ = JS_FreeValue(ctx, item);
vm_block = 152; continue;
}
// C line 44273
154 => {
vm_block = 2; continue;
}
// C line 44272
155 => {
let _ = JS_FreeValue(ctx, method);
vm_block = 154; continue;
}
// C line 44271
156 => {
vm_block = if (JS_IsException(item)) != 0 { 155 } else { 153 }; continue;
}
// C line 44270
157 => {
let _ = { let assigned = JS_IteratorNext(ctx, (*(it)).obj, method, (0 as i32), core::ptr::null_mut::<JSValue>(), pdone); item = assigned; assigned };
vm_block = 156; continue;
}
// C line 44269
158 => {
let _ = { let old = (*(it)).count; (*(it)).count = ((*(it)).count).wrapping_sub(1); old };
vm_block = 157; continue;
}
// C line 44262
159 => {
let _ = { let assigned = JS_DupValue(ctx, (*(it)).next); method = assigned; assigned };
vm_block = 146; continue;
}
// C line 44266
160 => {
vm_block = 3; continue;
}
// C line 44265
161 => {
vm_block = if (JS_IsException(method)) != 0 { 160 } else { 146 }; continue;
}
// C line 44264
162 => {
let _ = { let assigned = JS_GetProperty(ctx, (*(it)).obj, (((crate::quickjs_atom::JS_ATOM_return as i32)) as JSAtom)); method = assigned; assigned };
vm_block = 161; continue;
}
// C line 44261
163 => {
vm_block = if ((((magic) == ((0 as i32))) as i32)) != 0 { 159 } else { 162 }; continue;
}
// C line 44257
164 => {
vm_block = match (((*(it)).kind()) as i32) { x if x == (JS_ITERATOR_HELPER_KIND_TAKE as i32) => 26, x if x == (JS_ITERATOR_HELPER_KIND_MAP as i32) => 48, x if x == (JS_ITERATOR_HELPER_KIND_FLAT_MAP as i32) => 109, x if x == (JS_ITERATOR_HELPER_KIND_FILTER as i32) => 138, x if x == (JS_ITERATOR_HELPER_KIND_DROP as i32) => 163, _ => 7, }; continue;
}
// C line 44255
165 => {
let _ = { let assigned = (((1 as i32)) as u8); (*(it)).set_executing((assigned) as _); assigned };
vm_block = 164; continue;
}
// C line 44252
166 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) };
}
// C line 44251
167 => {
let _ = { let assigned = (1 as i32); *(pdone) = assigned; assigned };
vm_block = 166; continue;
}
// C line 44250
168 => {
vm_block = if ((*(it)).done()) != 0 { 167 } else { 165 }; continue;
}
// C line 44249
169 => {
return JS_ThrowTypeError(ctx, c"cannot invoke a running iterator".as_ptr());
}
// C line 44248
170 => {
vm_block = if ((*(it)).executing()) != 0 { 169 } else { 168 }; continue;
}
// C line 44247
171 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 44246
172 => {
vm_block = if ((!(!(it).is_null()) as i32)) != 0 { 171 } else { 170 }; continue;
}
// C line 44245
173 => {
let _ = { let assigned = ((JS_GetOpaque2(ctx, this_val, (((JS_CLASS_ITERATOR_HELPER as i32)) as JSClassID))) as *mut JSIteratorHelperData); it = assigned; assigned };
vm_block = 172; continue;
}
// C line 44243
174 => {
let _ = { let assigned = (0 as i32); *(pdone) = assigned; assigned };
vm_block = 173; continue;
}
_ => std::process::abort(),
} }
}
