// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:51541. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code, unused_variables)]
unsafe fn js_map_constructor(mut ctx: *mut JSContext, mut new_target: JSValue, mut argc: i32, mut argv: *mut JSValue, mut magic: i32) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut s: *mut JSMapState = core::mem::zeroed();
let mut obj: JSValue = core::mem::zeroed();
let mut adder: JSValue = core::mem::zeroed();
let mut iter: JSValue = core::mem::zeroed();
let mut next_method: JSValue = core::mem::zeroed();
let mut arr: JSValue = core::mem::zeroed();
let mut is_set: i32 = core::mem::zeroed();
let mut is_weak: i32 = core::mem::zeroed();
let mut item: JSValue = core::mem::zeroed();
let mut ret: JSValue = core::mem::zeroed();
let mut done: i32 = core::mem::zeroed();
let mut key: JSValue = core::mem::zeroed();
let mut value: JSValue = core::mem::zeroed();
let mut args: [JSValue; 2] = core::mem::zeroed();
let mut vm_block: usize = 81;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 51649
1 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 51648
2 => {
let _ = JS_FreeValue(ctx, obj);
vm_block = 1; continue;
}
// C line 51647
3 => {
let _ = JS_FreeValue(ctx, adder);
vm_block = 2; continue;
}
// C line 51646
4 => {
let _ = JS_FreeValue(ctx, iter);
vm_block = 3; continue;
}
// C line ? labels: fail
5 => {
let _ = JS_FreeValue(ctx, next_method);
vm_block = 4; continue;
}
// C line ? labels: fail_close
6 => {
let _ = JS_IteratorClose(ctx, iter, (1 as i32));
vm_block = 5; continue;
}
// C line 51640
7 => {
return obj;
}
// C line 51638
8 => {
let _ = JS_FreeValue(ctx, adder);
vm_block = 7; continue;
}
// C line 51637
9 => {
let _ = JS_FreeValue(ctx, iter);
vm_block = 8; continue;
}
// C line 51636
10 => {
let _ = JS_FreeValue(ctx, next_method);
vm_block = 9; continue;
}
// C line 51593
11 => {
vm_block = 44; continue;
}
// C line 51634
12 => {
let _ = JS_FreeValue(ctx, item);
vm_block = 11; continue;
}
// C line 51633
13 => {
let _ = JS_FreeValue(ctx, ret);
vm_block = 12; continue;
}
// C line 51603
14 => {
vm_block = 6; continue;
}
// C line 51602
15 => {
let _ = JS_FreeValue(ctx, item);
vm_block = 14; continue;
}
// C line 51601
16 => {
vm_block = if (JS_IsException(ret)) != 0 { 15 } else { 13 }; continue;
}
// C line 51600
17 => {
let _ = { let assigned = JS_Call(ctx, adder, obj, (1 as i32), core::ptr::addr_of_mut!(item)); ret = assigned; assigned };
vm_block = 16; continue;
}
// C line 51631
18 => {
let _ = JS_FreeValue(ctx, value);
vm_block = 13; continue;
}
// C line 51630
19 => {
let _ = JS_FreeValue(ctx, key);
vm_block = 18; continue;
}
// C line 51628
20 => {
vm_block = 6; continue;
}
// C line 51627
21 => {
let _ = JS_FreeValue(ctx, value);
vm_block = 20; continue;
}
// C line 51626
22 => {
let _ = JS_FreeValue(ctx, key);
vm_block = 21; continue;
}
// C line ? labels: fail1
23 => {
let _ = JS_FreeValue(ctx, item);
vm_block = 22; continue;
}
// C line 51623
24 => {
vm_block = if (JS_IsException(ret)) != 0 { 23 } else { 19 }; continue;
}
// C line 51622
25 => {
let _ = { let assigned = JS_Call(ctx, adder, obj, (2 as i32), (args).as_mut_ptr()); ret = assigned; assigned };
vm_block = 24; continue;
}
// C line 51621
26 => {
let _ = { let assigned = value; *((args).as_mut_ptr()).offset(((1 as i32)) as isize) = assigned; assigned };
vm_block = 25; continue;
}
// C line 51620
27 => {
let _ = { let assigned = key; *((args).as_mut_ptr()).offset(((0 as i32)) as isize) = assigned; assigned };
vm_block = 26; continue;
}
// C line 51619
28 => {
vm_block = 23; continue;
}
// C line 51618
29 => {
vm_block = if (JS_IsException(value)) != 0 { 28 } else { 27 }; continue;
}
// C line 51617
30 => {
let _ = { let assigned = JS_GetPropertyUint32(ctx, item, (((1 as i32)) as u32)); value = assigned; assigned };
vm_block = 29; continue;
}
// C line 51616
31 => {
vm_block = 23; continue;
}
// C line 51615
32 => {
vm_block = if (JS_IsException(key)) != 0 { 31 } else { 30 }; continue;
}
// C line 51614
33 => {
let _ = { let assigned = JS_GetPropertyUint32(ctx, item, (((0 as i32)) as u32)); key = assigned; assigned };
vm_block = 32; continue;
}
// C line 51612
34 => {
vm_block = 23; continue;
}
// C line 51611
35 => {
let _ = JS_ThrowTypeErrorNotAnObject(ctx);
vm_block = 34; continue;
}
// C line 51610
36 => {
vm_block = if ((!((JS_IsObject(item)) != 0) as i32)) != 0 { 35 } else { 33 }; continue;
}
// C line 51609
37 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; value = assigned; assigned };
vm_block = 36; continue;
}
// C line 51608
38 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; key = assigned; assigned };
vm_block = 37; continue;
}
// C line 51599
39 => {
vm_block = if (is_set) != 0 { 17 } else { 38 }; continue;
}
// C line 51598
40 => {
vm_block = 10; continue;
}
// C line 51597
41 => {
vm_block = if (done) != 0 { 40 } else { 39 }; continue;
}
// C line 51596
42 => {
vm_block = 5; continue;
}
// C line 51595
43 => {
vm_block = if (JS_IsException(item)) != 0 { 42 } else { 41 }; continue;
}
// C line 51594
44 => {
let _ = { let assigned = JS_IteratorNext(ctx, iter, next_method, (0 as i32), core::ptr::null_mut::<JSValue>(), core::ptr::addr_of_mut!(done)); item = assigned; assigned };
vm_block = 43; continue;
}
// C line 51591
45 => {
vm_block = 5; continue;
}
// C line 51590
46 => {
vm_block = if (JS_IsException(next_method)) != 0 { 45 } else { 11 }; continue;
}
// C line 51589
47 => {
let _ = { let assigned = JS_GetProperty(ctx, iter, (((crate::quickjs_atom::JS_ATOM_next as i32)) as JSAtom)); next_method = assigned; assigned };
vm_block = 46; continue;
}
// C line 51588
48 => {
vm_block = 5; continue;
}
// C line 51587
49 => {
vm_block = if (JS_IsException(iter)) != 0 { 48 } else { 47 }; continue;
}
// C line 51586
50 => {
let _ = { let assigned = JS_GetIterator(ctx, arr, (0 as i32)); iter = assigned; assigned };
vm_block = 49; continue;
}
// C line 51583
51 => {
vm_block = 5; continue;
}
// C line 51582
52 => {
let _ = JS_ThrowTypeError(ctx, c"set/add is not a function".as_ptr());
vm_block = 51; continue;
}
// C line 51581
53 => {
vm_block = if ((!((JS_IsFunction(ctx, adder)) != 0) as i32)) != 0 { 52 } else { 50 }; continue;
}
// C line 51580
54 => {
vm_block = 5; continue;
}
// C line 51579
55 => {
vm_block = if (JS_IsException(adder)) != 0 { 54 } else { 53 }; continue;
}
// C line 51578
56 => {
let _ = { let assigned = JS_GetProperty(ctx, obj, ((if (is_set) != 0 { (crate::quickjs_atom::JS_ATOM_add as i32) } else { (crate::quickjs_atom::JS_ATOM_set as i32) }) as JSAtom)); adder = assigned; assigned };
vm_block = 55; continue;
}
// C line 51574
57 => {
vm_block = if ((((((!((JS_IsUndefined(arr)) != 0) as i32)) != 0) && (((!((JS_IsNull(arr)) != 0) as i32)) != 0)) as i32)) != 0 { 56 } else { 7 }; continue;
}
// C line 51573
58 => {
let _ = { let assigned = *(argv).offset(((0 as i32)) as isize); arr = assigned; assigned };
vm_block = 57; continue;
}
// C line 51572
59 => {
vm_block = if ((((argc) > ((0 as i32))) as i32)) != 0 { 58 } else { 57 }; continue;
}
// C line 51571
60 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; arr = assigned; assigned };
vm_block = 59; continue;
}
// C line 51569
61 => {
let _ = { let assigned = (((4 as i32)) as u32); (*(s)).record_count_threshold = assigned; assigned };
vm_block = 60; continue;
}
// C line 51568
62 => {
vm_block = 5; continue;
}
// C line 51567
63 => {
vm_block = if ((!(!((*(s)).hash_table).is_null()) as i32)) != 0 { 62 } else { 61 }; continue;
}
// C line 51566
64 => {
let _ = { let assigned = ((js_mallocz(ctx, ((size_of::<*mut JSMapRecord>() as usize)).wrapping_mul((((*(s)).hash_size) as usize)))) as *mut *mut JSMapRecord); (*(s)).hash_table = assigned; assigned };
vm_block = 63; continue;
}
// C line 51565
65 => {
let _ = { let assigned = ((1 as u32)).wrapping_shl(((*(s)).hash_bits) as u32); (*(s)).hash_size = assigned; assigned };
vm_block = 64; continue;
}
// C line 51564
66 => {
let _ = { let assigned = (1 as i32); (*(s)).hash_bits = assigned; assigned };
vm_block = 65; continue;
}
// C line 51563
67 => {
let _ = JS_SetOpaque(obj, ((s) as *mut c_void));
vm_block = 66; continue;
}
// C line 51561
68 => {
let _ = list_add_tail(core::ptr::addr_of_mut!(((*(s)).weakref_header).link), core::ptr::addr_of_mut!((*((*(ctx)).rt)).weakref_list));
vm_block = 67; continue;
}
// C line 51560
69 => {
let _ = { let assigned = (((JS_WEAKREF_TYPE_MAP as i32)) as JSWeakRefHeaderTypeEnum); ((*(s)).weakref_header).weakref_type = assigned; assigned };
vm_block = 68; continue;
}
// C line 51559
70 => {
vm_block = if (is_weak) != 0 { 69 } else { 67 }; continue;
}
// C line 51558
71 => {
let _ = { let assigned = is_weak; (*(s)).is_weak = assigned; assigned };
vm_block = 70; continue;
}
// C line 51557
72 => {
let _ = init_list_head(core::ptr::addr_of_mut!((*(s)).records));
vm_block = 71; continue;
}
// C line 51556
73 => {
vm_block = 5; continue;
}
// C line 51555
74 => {
vm_block = if ((!(!(s).is_null()) as i32)) != 0 { 73 } else { 72 }; continue;
}
// C line 51554
75 => {
let _ = { let assigned = ((js_mallocz(ctx, (size_of::<JSMapState>() as usize))) as *mut JSMapState); s = assigned; assigned };
vm_block = 74; continue;
}
// C line 51553
76 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 51552
77 => {
vm_block = if (JS_IsException(obj)) != 0 { 76 } else { 75 }; continue;
}
// C line 51551
78 => {
let _ = { let assigned = js_create_from_ctor(ctx, new_target, ((JS_CLASS_MAP as i32)).wrapping_add(magic)); obj = assigned; assigned };
vm_block = 77; continue;
}
// C line 51550
79 => {
let _ = { let assigned = (((((magic) & (((1 as i32)).wrapping_shl(((1 as i32)) as u32)))) != ((0 as i32))) as i32); is_weak = assigned; assigned };
vm_block = 78; continue;
}
// C line 51549
80 => {
let _ = { let assigned = ((magic) & (((1 as i32)).wrapping_shl(((0 as i32)) as u32))); is_set = assigned; assigned };
vm_block = 79; continue;
}
// C line 51545
81 => {
adder = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) };
iter = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) };
next_method = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) };
vm_block = 80; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:51751. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code, unused_variables)]
unsafe fn map_find_record(mut ctx: *mut JSContext, mut s: *mut JSMapState, mut key: JSValue) -> *mut JSMapRecord {
let mut vm_local_storage = Vec::<u64>::new();
let mut mr: *mut JSMapRecord = core::mem::zeroed();
let mut h: u32 = core::mem::zeroed();
let mut vm_block: usize = 8;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 51765
1 => {
return core::ptr::null_mut::<JSMapRecord>();
}
// C line 51757
2 => {
vm_block = if ((((mr) != (core::ptr::null_mut::<JSMapRecord>())) as i32)) != 0 { 6 } else { 1 }; continue;
}
// C line ?
3 => {
let _ = { let assigned = (*(mr)).hash_next; mr = assigned; assigned };
vm_block = 2; continue;
}
// C line 51762
4 => {
return mr;
}
// C line 51761
5 => {
vm_block = if (js_same_value_zero(ctx, (*(mr)).key, key)) != 0 { 4 } else { 3 }; continue;
}
// C line 51758
6 => {
vm_block = if ((((((*(mr)).empty) != 0) || (((((((*(s)).is_weak) != 0) && (((!((js_weakref_is_live((*(mr)).key)) != 0) as i32)) != 0)) as i32)) != 0)) as i32)) != 0 { 3 } else { 5 }; continue;
}
// C line 51757
7 => {
let _ = { let assigned = *((*(s)).hash_table).offset((h) as isize); mr = assigned; assigned };
vm_block = 2; continue;
}
// C line 51756
8 => {
let _ = { let assigned = map_hash_key(key, (*(s)).hash_bits); h = assigned; assigned };
vm_block = 7; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:51768. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code, unused_variables)]
unsafe fn map_hash_resize(mut ctx: *mut JSContext, mut s: *mut JSMapState) -> () {
let mut vm_local_storage = Vec::<u64>::new();
let mut new_hash_size: u32 = core::mem::zeroed();
let mut h: u32 = core::mem::zeroed();
let mut new_hash_bits: i32 = core::mem::zeroed();
let mut el: *mut list_head = core::mem::zeroed();
let mut mr: *mut JSMapRecord = core::mem::zeroed();
let mut new_hash_table: *mut *mut JSMapRecord = core::mem::zeroed();
let mut vm_block: usize = 18;
loop { match vm_block {
// C line ?
0 => {
return ();
}
// C line 51797
1 => {
let _ = { let assigned = (new_hash_size).wrapping_mul((((2 as i32)) as u32)); (*(s)).record_count_threshold = assigned; assigned };
vm_block = 0; continue;
}
// C line 51796
2 => {
let _ = { let assigned = new_hash_size; (*(s)).hash_size = assigned; assigned };
vm_block = 1; continue;
}
// C line 51795
3 => {
let _ = { let assigned = new_hash_bits; (*(s)).hash_bits = assigned; assigned };
vm_block = 2; continue;
}
// C line 51794
4 => {
let _ = { let assigned = new_hash_table; (*(s)).hash_table = assigned; assigned };
vm_block = 3; continue;
}
// C line 51785
5 => {
vm_block = if ((((el) != (core::ptr::addr_of_mut!((*(s)).records))) as i32)) != 0 { 11 } else { 4 }; continue;
}
// C line ?
6 => {
let _ = { let assigned = (*(el)).next; el = assigned; assigned };
vm_block = 5; continue;
}
// C line 51791
7 => {
let _ = { let assigned = mr; *(new_hash_table).offset((h) as isize) = assigned; assigned };
vm_block = 6; continue;
}
// C line 51790
8 => {
let _ = { let assigned = *(new_hash_table).offset((h) as isize); (*(mr)).hash_next = assigned; assigned };
vm_block = 7; continue;
}
// C line 51789
9 => {
let _ = { let assigned = map_hash_key((*(mr)).key, new_hash_bits); h = assigned; assigned };
vm_block = 8; continue;
}
// C line 51787
10 => {
vm_block = if ((((((*(mr)).empty) != 0) || (((((((*(s)).is_weak) != 0) && (((!((js_weakref_is_live((*(mr)).key)) != 0) as i32)) != 0)) as i32)) != 0)) as i32)) != 0 { 6 } else { 9 }; continue;
}
// C line 51786
11 => {
let _ = { let assigned = (((((el) as *mut u8)).offset(-(((core::mem::offset_of!(JSMapRecord, link) as usize)) as isize))) as *mut JSMapRecord); mr = assigned; assigned };
vm_block = 10; continue;
}
// C line ?
12 => {
let _ = { let assigned = (*(core::ptr::addr_of_mut!((*(s)).records))).next; el = assigned; assigned };
vm_block = 5; continue;
}
// C line 51783
13 => {
let _ = { let dst = (((new_hash_table) as *mut c_void)) as *mut u8; core::ptr::write_bytes(dst, ((0 as i32)) as u8, (((size_of::<*mut JSMapRecord>() as usize)).wrapping_mul(((new_hash_size) as usize))) as usize); dst as *mut c_void };
vm_block = 12; continue;
}
// C line 51781
14 => {
return ();
}
// C line 51780
15 => {
vm_block = if ((!(!(new_hash_table).is_null()) as i32)) != 0 { 14 } else { 13 }; continue;
}
// C line 51778
16 => {
let _ = { let assigned = ((js_realloc(ctx, (((*(s)).hash_table) as *mut c_void), ((size_of::<*mut JSMapRecord>() as usize)).wrapping_mul(((new_hash_size) as usize)))) as *mut *mut JSMapRecord); new_hash_table = assigned; assigned };
vm_block = 15; continue;
}
// C line 51777
17 => {
let _ = { let assigned = ((1 as u32)).wrapping_shl((new_hash_bits) as u32); new_hash_size = assigned; assigned };
vm_block = 16; continue;
}
// C line 51776
18 => {
let _ = { let assigned = min_int(((*(s)).hash_bits).wrapping_add((1 as i32)), (31 as i32)); new_hash_bits = assigned; assigned };
vm_block = 17; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:51800. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code, unused_variables)]
unsafe fn map_add_record(mut ctx: *mut JSContext, mut s: *mut JSMapState, mut key: JSValue) -> *mut JSMapRecord {
let mut vm_local_storage = Vec::<u64>::new();
let mut h: u32 = core::mem::zeroed();
let mut mr: *mut JSMapRecord = core::mem::zeroed();
let mut vm_block: usize = 16;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 51824
1 => {
return mr;
}
// C line 51822
2 => {
let _ = map_hash_resize(ctx, s);
vm_block = 1; continue;
}
// C line 51821
3 => {
vm_block = if (((((*(s)).record_count) >= ((*(s)).record_count_threshold)) as i32)) != 0 { 2 } else { 1 }; continue;
}
// C line 51820
4 => {
let _ = { let old = (*(s)).record_count; (*(s)).record_count = ((*(s)).record_count).wrapping_add(1); old };
vm_block = 3; continue;
}
// C line 51819
5 => {
let _ = list_add_tail(core::ptr::addr_of_mut!((*(mr)).link), core::ptr::addr_of_mut!((*(s)).records));
vm_block = 4; continue;
}
// C line 51818
6 => {
let _ = { let assigned = mr; *((*(s)).hash_table).offset((h) as isize) = assigned; assigned };
vm_block = 5; continue;
}
// C line 51817
7 => {
let _ = { let assigned = *((*(s)).hash_table).offset((h) as isize); (*(mr)).hash_next = assigned; assigned };
vm_block = 6; continue;
}
// C line 51816
8 => {
let _ = { let assigned = map_hash_key(key, (*(s)).hash_bits); h = assigned; assigned };
vm_block = 7; continue;
}
// C line 51812
9 => {
let _ = { let assigned = js_weakref_new(ctx, key); (*(mr)).key = assigned; assigned };
vm_block = 8; continue;
}
// C line 51814
10 => {
let _ = { let assigned = JS_DupValue(ctx, key); (*(mr)).key = assigned; assigned };
vm_block = 8; continue;
}
// C line 51811
11 => {
vm_block = if ((*(s)).is_weak) != 0 { 9 } else { 10 }; continue;
}
// C line 51810
12 => {
let _ = { let assigned = (0 as i32); (*(mr)).empty = (assigned) as i8; assigned };
vm_block = 11; continue;
}
// C line 51809
13 => {
let _ = { let assigned = (1 as i32); (*(mr)).ref_count = assigned; assigned };
vm_block = 12; continue;
}
// C line 51808
14 => {
return core::ptr::null_mut::<JSMapRecord>();
}
// C line 51807
15 => {
vm_block = if ((!(!(mr).is_null()) as i32)) != 0 { 14 } else { 13 }; continue;
}
// C line 51806
16 => {
let _ = { let assigned = ((js_malloc(ctx, (size_of::<JSMapRecord>() as usize))) as *mut JSMapRecord); mr = assigned; assigned };
vm_block = 15; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:51827. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code, unused_variables)]
unsafe fn set_add_record(mut ctx: *mut JSContext, mut s: *mut JSMapState, mut key: JSValue) -> *mut JSMapRecord {
let mut vm_local_storage = Vec::<u64>::new();
let mut mr: *mut JSMapRecord = core::mem::zeroed();
let mut vm_block: usize = 5;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 51835
1 => {
return mr;
}
// C line 51834
2 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; (*(mr)).value = assigned; assigned };
vm_block = 1; continue;
}
// C line 51833
3 => {
return core::ptr::null_mut::<JSMapRecord>();
}
// C line 51832
4 => {
vm_block = if ((!(!(mr).is_null()) as i32)) != 0 { 3 } else { 2 }; continue;
}
// C line 51831
5 => {
let _ = { let assigned = map_add_record(ctx, s, key); mr = assigned; assigned };
vm_block = 4; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:51904. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code, unused_variables)]
unsafe fn js_map_set(mut ctx: *mut JSContext, mut this_val: JSValue, mut argc: i32, mut argv: *mut JSValue, mut magic: i32) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut s: *mut JSMapState = core::mem::zeroed();
let mut mr: *mut JSMapRecord = core::mem::zeroed();
let mut key: JSValue = core::mem::zeroed();
let mut value: JSValue = core::mem::zeroed();
let mut vm_block: usize = 17;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 51929
1 => {
return JS_DupValue(ctx, this_val);
}
// C line 51928
2 => {
let _ = { let assigned = JS_DupValue(ctx, value); (*(mr)).value = assigned; assigned };
vm_block = 1; continue;
}
// C line 51922
3 => {
let _ = JS_FreeValue(ctx, (*(mr)).value);
vm_block = 2; continue;
}
// C line 51926
4 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 51925
5 => {
vm_block = if ((!(!(mr).is_null()) as i32)) != 0 { 4 } else { 2 }; continue;
}
// C line 51924
6 => {
let _ = { let assigned = map_add_record(ctx, s, key); mr = assigned; assigned };
vm_block = 5; continue;
}
// C line 51921
7 => {
vm_block = if !(mr).is_null() { 3 } else { 6 }; continue;
}
// C line 51920
8 => {
let _ = { let assigned = map_find_record(ctx, s, key); mr = assigned; assigned };
vm_block = 7; continue;
}
// C line 51917
9 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; value = assigned; assigned };
vm_block = 8; continue;
}
// C line 51919
10 => {
let _ = { let assigned = *(argv).offset(((1 as i32)) as isize); value = assigned; assigned };
vm_block = 8; continue;
}
// C line 51916
11 => {
vm_block = if (((magic) & (((1 as i32)).wrapping_shl(((0 as i32)) as u32)))) != 0 { 9 } else { 10 }; continue;
}
// C line 51915
12 => {
return JS_ThrowTypeError(ctx, JSErrorMessage::Pieces(&[b"invalid value used as ", core::ffi::CStr::from_ptr(if (((magic) & (((1 as i32)).wrapping_shl(((0 as i32)) as u32)))) != 0 { c"WeakSet".as_ptr() } else { c"WeakMap".as_ptr() }).to_bytes(), b" key"]));
}
// C line 51914
13 => {
vm_block = if ((((((*(s)).is_weak) != 0) && (((!((js_weakref_is_target(key)) != 0) as i32)) != 0)) as i32)) != 0 { 12 } else { 11 }; continue;
}
// C line 51913
14 => {
let _ = { let assigned = map_normalize_key_const(ctx, *(argv).offset(((0 as i32)) as isize)); key = assigned; assigned };
vm_block = 13; continue;
}
// C line 51912
15 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 51911
16 => {
vm_block = if ((!(!(s).is_null()) as i32)) != 0 { 15 } else { 14 }; continue;
}
// C line 51907
17 => {
s = ((JS_GetOpaque2(ctx, this_val, ((((JS_CLASS_MAP as i32)).wrapping_add(magic)) as JSClassID))) as *mut JSMapState);
vm_block = 16; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:51932. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code, unused_variables)]
unsafe fn js_map_get(mut ctx: *mut JSContext, mut this_val: JSValue, mut argc: i32, mut argv: *mut JSValue, mut magic: i32) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut s: *mut JSMapState = core::mem::zeroed();
let mut mr: *mut JSMapRecord = core::mem::zeroed();
let mut key: JSValue = core::mem::zeroed();
let mut vm_block: usize = 8;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 51944
1 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) };
}
// C line 51946
2 => {
return JS_DupValue(ctx, (*(mr)).value);
}
// C line 51943
3 => {
vm_block = if ((!(!(mr).is_null()) as i32)) != 0 { 1 } else { 2 }; continue;
}
// C line 51942
4 => {
let _ = { let assigned = map_find_record(ctx, s, key); mr = assigned; assigned };
vm_block = 3; continue;
}
// C line 51941
5 => {
let _ = { let assigned = map_normalize_key_const(ctx, *(argv).offset(((0 as i32)) as isize)); key = assigned; assigned };
vm_block = 4; continue;
}
// C line 51940
6 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 51939
7 => {
vm_block = if ((!(!(s).is_null()) as i32)) != 0 { 6 } else { 5 }; continue;
}
// C line 51935
8 => {
s = ((JS_GetOpaque2(ctx, this_val, ((((JS_CLASS_MAP as i32)).wrapping_add(magic)) as JSClassID))) as *mut JSMapState);
vm_block = 7; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:51950. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code, unused_variables)]
unsafe fn map_delete_record(mut ctx: *mut JSContext, mut s: *mut JSMapState, mut key: JSValue) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut mr: *mut JSMapRecord = core::mem::zeroed();
let mut pmr: *mut *mut JSMapRecord = core::mem::zeroed();
let mut h: u32 = core::mem::zeroed();
let mut vm_block: usize = 14;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 51976
1 => {
return JSValue { u: JSValueUnion { uint64: (((((1 as i32)) as u32)) as u64) }, tag: (((JS_TAG_BOOL as i32)) as i64) };
}
// C line 51975
2 => {
let _ = map_delete_record_internal((*(ctx)).rt, s, mr);
vm_block = 1; continue;
}
// C line 51973
3 => {
let _ = { let assigned = (*(mr)).hash_next; *(pmr) = assigned; assigned };
vm_block = 2; continue;
}
// C line 51959
4 => {
vm_block = 11; continue;
}
// C line 51969
5 => {
let _ = { let assigned = core::ptr::addr_of_mut!((*(mr)).hash_next); pmr = assigned; assigned };
vm_block = 4; continue;
}
// C line 51967
6 => {
vm_block = 3; continue;
}
// C line 51966
7 => {
vm_block = if (js_same_value_zero(ctx, (*(mr)).key, key)) != 0 { 6 } else { 5 }; continue;
}
// C line 51963
8 => {
vm_block = if ((((((*(mr)).empty) != 0) || (((((((*(s)).is_weak) != 0) && (((!((js_weakref_is_live((*(mr)).key)) != 0) as i32)) != 0)) as i32)) != 0)) as i32)) != 0 { 5 } else { 7 }; continue;
}
// C line 51962
9 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_BOOL as i32)) as i64) };
}
// C line 51961
10 => {
vm_block = if ((((mr) == (core::ptr::null_mut::<JSMapRecord>())) as i32)) != 0 { 9 } else { 8 }; continue;
}
// C line 51960
11 => {
let _ = { let assigned = *(pmr); mr = assigned; assigned };
vm_block = 10; continue;
}
// C line 51958
12 => {
let _ = { let assigned = core::ptr::addr_of_mut!(*((*(s)).hash_table).offset((h) as isize)); pmr = assigned; assigned };
vm_block = 4; continue;
}
// C line 51957
13 => {
let _ = { let assigned = map_hash_key(key, (*(s)).hash_bits); h = assigned; assigned };
vm_block = 12; continue;
}
// C line 51955
14 => {
let _ = { let assigned = map_normalize_key_const(ctx, key); key = assigned; assigned };
vm_block = 13; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:51979. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code, unused_variables)]
unsafe fn js_map_getOrInsert(mut ctx: *mut JSContext, mut this_val: JSValue, mut argc: i32, mut argv: *mut JSValue, mut magic: i32) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut computed: i32 = core::mem::zeroed();
let mut class_id: JSClassID = core::mem::zeroed();
let mut s: *mut JSMapState = core::mem::zeroed();
let mut mr: *mut JSMapRecord = core::mem::zeroed();
let mut key: JSValue = core::mem::zeroed();
let mut value: JSValue = core::mem::zeroed();
let mut vm_block: usize = 24;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 52013
1 => {
return JS_DupValue(ctx, (*(mr)).value);
}
// C line 52011
2 => {
let _ = { let assigned = value; (*(mr)).value = assigned; assigned };
vm_block = 1; continue;
}
// C line 52009
3 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 52008
4 => {
let _ = JS_FreeValue(ctx, value);
vm_block = 3; continue;
}
// C line 52007
5 => {
vm_block = if ((!(!(mr).is_null()) as i32)) != 0 { 4 } else { 2 }; continue;
}
// C line 52006
6 => {
let _ = { let assigned = map_add_record(ctx, s, key); mr = assigned; assigned };
vm_block = 5; continue;
}
// C line 52002
7 => {
let _ = map_delete_record(ctx, s, key);
vm_block = 6; continue;
}
// C line 52001
8 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 52000
9 => {
vm_block = if (JS_IsException(value)) != 0 { 8 } else { 7 }; continue;
}
// C line 51999
10 => {
let _ = { let assigned = JS_Call(ctx, *(argv).offset(((1 as i32)) as isize), JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }, (1 as i32), core::ptr::addr_of_mut!(key)); value = assigned; assigned };
vm_block = 9; continue;
}
// C line 52004
11 => {
let _ = { let assigned = JS_DupValue(ctx, *(argv).offset(((1 as i32)) as isize)); value = assigned; assigned };
vm_block = 6; continue;
}
// C line 51998
12 => {
vm_block = if (computed) != 0 { 10 } else { 11 }; continue;
}
// C line 51997
13 => {
vm_block = if ((!(!(mr).is_null()) as i32)) != 0 { 12 } else { 1 }; continue;
}
// C line 51996
14 => {
let _ = { let assigned = map_find_record(ctx, s, key); mr = assigned; assigned };
vm_block = 13; continue;
}
// C line 51995
15 => {
return JS_ThrowTypeError(ctx, c"invalid value used as WeakMap key".as_ptr());
}
// C line 51994
16 => {
vm_block = if ((((((*(s)).is_weak) != 0) && (((!((js_weakref_is_target(key)) != 0) as i32)) != 0)) as i32)) != 0 { 15 } else { 14 }; continue;
}
// C line 51993
17 => {
let _ = { let assigned = map_normalize_key_const(ctx, *(argv).offset(((0 as i32)) as isize)); key = assigned; assigned };
vm_block = 16; continue;
}
// C line 51992
18 => {
return JS_ThrowTypeError(ctx, c"not a function".as_ptr());
}
// C line 51991
19 => {
vm_block = if (((((computed) != 0) && (((!((JS_IsFunction(ctx, *(argv).offset(((1 as i32)) as isize))) != 0) as i32)) != 0)) as i32)) != 0 { 18 } else { 17 }; continue;
}
// C line 51990
20 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 51989
21 => {
vm_block = if ((!(!(s).is_null()) as i32)) != 0 { 20 } else { 19 }; continue;
}
// C line 51984
22 => {
s = ((JS_GetOpaque2(ctx, this_val, class_id)) as *mut JSMapState);
vm_block = 21; continue;
}
// C line 51983
23 => {
class_id = (((magic).wrapping_shr(((1 as i32)) as u32)) as JSClassID);
vm_block = 22; continue;
}
// C line 51982
24 => {
computed = ((magic) & ((1 as i32)));
vm_block = 23; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:52016. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code, unused_variables)]
unsafe fn js_map_has(mut ctx: *mut JSContext, mut this_val: JSValue, mut argc: i32, mut argv: *mut JSValue, mut magic: i32) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut s: *mut JSMapState = core::mem::zeroed();
let mut mr: *mut JSMapRecord = core::mem::zeroed();
let mut key: JSValue = core::mem::zeroed();
let mut vm_block: usize = 6;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 52027
1 => {
return JS_NewBool(ctx, (((mr) != (core::ptr::null_mut::<JSMapRecord>())) as i32));
}
// C line 52026
2 => {
let _ = { let assigned = map_find_record(ctx, s, key); mr = assigned; assigned };
vm_block = 1; continue;
}
// C line 52025
3 => {
let _ = { let assigned = map_normalize_key_const(ctx, *(argv).offset(((0 as i32)) as isize)); key = assigned; assigned };
vm_block = 2; continue;
}
// C line 52024
4 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 52023
5 => {
vm_block = if ((!(!(s).is_null()) as i32)) != 0 { 4 } else { 3 }; continue;
}
// C line 52019
6 => {
s = ((JS_GetOpaque2(ctx, this_val, ((((JS_CLASS_MAP as i32)).wrapping_add(magic)) as JSClassID))) as *mut JSMapState);
vm_block = 5; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:52030. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code, unused_variables)]
unsafe fn js_map_delete(mut ctx: *mut JSContext, mut this_val: JSValue, mut argc: i32, mut argv: *mut JSValue, mut magic: i32) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut s: *mut JSMapState = core::mem::zeroed();
let mut vm_block: usize = 4;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 52036
1 => {
return map_delete_record(ctx, s, *(argv).offset(((0 as i32)) as isize));
}
// C line 52035
2 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 52034
3 => {
vm_block = if ((!(!(s).is_null()) as i32)) != 0 { 2 } else { 1 }; continue;
}
// C line 52033
4 => {
s = ((JS_GetOpaque2(ctx, this_val, ((((JS_CLASS_MAP as i32)).wrapping_add(magic)) as JSClassID))) as *mut JSMapState);
vm_block = 3; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:52039. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code, unused_variables)]
unsafe fn js_map_clear(mut ctx: *mut JSContext, mut this_val: JSValue, mut argc: i32, mut argv: *mut JSValue, mut magic: i32) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut s: *mut JSMapState = core::mem::zeroed();
let mut el: *mut list_head = core::mem::zeroed();
let mut el1: *mut list_head = core::mem::zeroed();
let mut mr: *mut JSMapRecord = core::mem::zeroed();
let mut vm_block: usize = 10;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 52056
1 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) };
}
// C line 52052
2 => {
vm_block = if ((((el) != (core::ptr::addr_of_mut!((*(s)).records))) as i32)) != 0 { 5 } else { 1 }; continue;
}
// C line ?
3 => {
let _ = { let _ = { let assigned = el1; el = assigned; assigned }; { let assigned = (*(el)).next; el1 = assigned; assigned } };
vm_block = 2; continue;
}
// C line 52054
4 => {
let _ = map_delete_record_internal((*(ctx)).rt, s, mr);
vm_block = 3; continue;
}
// C line 52053
5 => {
let _ = { let assigned = (((((el) as *mut u8)).offset(-(((core::mem::offset_of!(JSMapRecord, link) as usize)) as isize))) as *mut JSMapRecord); mr = assigned; assigned };
vm_block = 4; continue;
}
// C line ?
6 => {
let _ = { let _ = { let assigned = (*(core::ptr::addr_of_mut!((*(s)).records))).next; el = assigned; assigned }; { let assigned = (*(el)).next; el1 = assigned; assigned } };
vm_block = 2; continue;
}
// C line 52050
7 => {
let _ = { let dst = ((((*(s)).hash_table) as *mut c_void)) as *mut u8; core::ptr::write_bytes(dst, ((0 as i32)) as u8, (((size_of::<*mut JSMapRecord>() as usize)).wrapping_mul((((*(s)).hash_size) as usize))) as usize); dst as *mut c_void };
vm_block = 6; continue;
}
// C line 52047
8 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 52046
9 => {
vm_block = if ((!(!(s).is_null()) as i32)) != 0 { 8 } else { 7 }; continue;
}
// C line 52042
10 => {
s = ((JS_GetOpaque2(ctx, this_val, ((((JS_CLASS_MAP as i32)).wrapping_add(magic)) as JSClassID))) as *mut JSMapState);
vm_block = 9; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:52059. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code, unused_variables)]
unsafe fn js_map_get_size(mut ctx: *mut JSContext, mut this_val: JSValue, mut magic: i32) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut s: *mut JSMapState = core::mem::zeroed();
let mut vm_block: usize = 4;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 52064
1 => {
return JS_NewUint32(ctx, (*(s)).record_count);
}
// C line 52063
2 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 52062
3 => {
vm_block = if ((!(!(s).is_null()) as i32)) != 0 { 2 } else { 1 }; continue;
}
// C line 52061
4 => {
s = ((JS_GetOpaque2(ctx, this_val, ((((JS_CLASS_MAP as i32)).wrapping_add(magic)) as JSClassID))) as *mut JSMapState);
vm_block = 3; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:52067. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code, unused_variables)]
unsafe fn js_map_forEach(mut ctx: *mut JSContext, mut this_val: JSValue, mut argc: i32, mut argv: *mut JSValue, mut magic: i32) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut s: *mut JSMapState = core::mem::zeroed();
let mut func: JSValue = core::mem::zeroed();
let mut this_arg: JSValue = core::mem::zeroed();
let mut ret: JSValue = core::mem::zeroed();
let mut args: [JSValue; 3] = core::mem::zeroed();
let mut el: *mut list_head = core::mem::zeroed();
let mut mr: *mut JSMapRecord = core::mem::zeroed();
let mut vm_block: usize = 30;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 52112
1 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) };
}
// C line 52088
2 => {
vm_block = if ((((el) != (core::ptr::addr_of_mut!((*(s)).records))) as i32)) != 0 { 20 } else { 1 }; continue;
}
// C line 52107
3 => {
let _ = JS_FreeValue(ctx, ret);
vm_block = 2; continue;
}
// C line 52106
4 => {
return ret;
}
// C line 52105
5 => {
vm_block = if (JS_IsException(ret)) != 0 { 4 } else { 3 }; continue;
}
// C line 52104
6 => {
let _ = map_decref_record((*(ctx)).rt, mr);
vm_block = 5; continue;
}
// C line 52103
7 => {
let _ = { let assigned = (*(el)).next; el = assigned; assigned };
vm_block = 6; continue;
}
// C line 52102
8 => {
let _ = JS_FreeValue(ctx, *((args).as_mut_ptr()).offset(((1 as i32)) as isize));
vm_block = 7; continue;
}
// C line 52101
9 => {
vm_block = if ((!((magic) != 0) as i32)) != 0 { 8 } else { 7 }; continue;
}
// C line 52100
10 => {
let _ = JS_FreeValue(ctx, *((args).as_mut_ptr()).offset(((0 as i32)) as isize));
vm_block = 9; continue;
}
// C line 52099
11 => {
let _ = { let assigned = JS_Call(ctx, func, this_arg, (3 as i32), (args).as_mut_ptr()); ret = assigned; assigned };
vm_block = 10; continue;
}
// C line 52098
12 => {
let _ = { let assigned = this_val; *((args).as_mut_ptr()).offset(((2 as i32)) as isize) = assigned; assigned };
vm_block = 11; continue;
}
// C line 52095
13 => {
let _ = { let assigned = *((args).as_mut_ptr()).offset(((1 as i32)) as isize); *((args).as_mut_ptr()).offset(((0 as i32)) as isize) = assigned; assigned };
vm_block = 12; continue;
}
// C line 52097
14 => {
let _ = { let assigned = JS_DupValue(ctx, (*(mr)).value); *((args).as_mut_ptr()).offset(((0 as i32)) as isize) = assigned; assigned };
vm_block = 12; continue;
}
// C line 52094
15 => {
vm_block = if (magic) != 0 { 13 } else { 14 }; continue;
}
// C line 52093
16 => {
let _ = { let assigned = JS_DupValue(ctx, (*(mr)).key); *((args).as_mut_ptr()).offset(((1 as i32)) as isize) = assigned; assigned };
vm_block = 15; continue;
}
// C line 52091
17 => {
let _ = { let old = (*(mr)).ref_count; (*(mr)).ref_count = ((*(mr)).ref_count).wrapping_add(1); old };
vm_block = 16; continue;
}
// C line 52109
18 => {
let _ = { let assigned = (*(el)).next; el = assigned; assigned };
vm_block = 2; continue;
}
// C line 52090
19 => {
vm_block = if ((!(((*(mr)).empty) != 0) as i32)) != 0 { 17 } else { 18 }; continue;
}
// C line 52089
20 => {
let _ = { let assigned = (((((el) as *mut u8)).offset(-(((core::mem::offset_of!(JSMapRecord, link) as usize)) as isize))) as *mut JSMapRecord); mr = assigned; assigned };
vm_block = 19; continue;
}
// C line 52087
21 => {
let _ = { let assigned = ((*(s)).records).next; el = assigned; assigned };
vm_block = 2; continue;
}
// C line 52084
22 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 52083
23 => {
vm_block = if (check_function(ctx, func)) != 0 { 22 } else { 21 }; continue;
}
// C line 52080
24 => {
let _ = { let assigned = *(argv).offset(((1 as i32)) as isize); this_arg = assigned; assigned };
vm_block = 23; continue;
}
// C line 52082
25 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; this_arg = assigned; assigned };
vm_block = 23; continue;
}
// C line 52079
26 => {
vm_block = if ((((argc) > ((1 as i32))) as i32)) != 0 { 24 } else { 25 }; continue;
}
// C line 52078
27 => {
let _ = { let assigned = *(argv).offset(((0 as i32)) as isize); func = assigned; assigned };
vm_block = 26; continue;
}
// C line 52077
28 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 52076
29 => {
vm_block = if ((!(!(s).is_null()) as i32)) != 0 { 28 } else { 27 }; continue;
}
// C line 52070
30 => {
s = ((JS_GetOpaque2(ctx, this_val, ((((JS_CLASS_MAP as i32)).wrapping_add(magic)) as JSClassID))) as *mut JSMapState);
vm_block = 29; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:52319. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code, unused_variables)]
unsafe fn js_create_map_iterator(mut ctx: *mut JSContext, mut this_val: JSValue, mut argc: i32, mut argv: *mut JSValue, mut magic: i32) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut kind: JSIteratorKindEnum = core::mem::zeroed();
let mut s: *mut JSMapState = core::mem::zeroed();
let mut it: *mut JSMapIteratorData = core::mem::zeroed();
let mut enum_obj: JSValue = core::mem::zeroed();
let mut vm_block: usize = 18;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line ? labels: fail
1 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 52344
2 => {
return enum_obj;
}
// C line 52343
3 => {
let _ = JS_SetOpaque(enum_obj, ((it) as *mut c_void));
vm_block = 2; continue;
}
// C line 52342
4 => {
let _ = { let assigned = core::ptr::null_mut::<JSMapRecord>(); (*(it)).cur_record = assigned; assigned };
vm_block = 3; continue;
}
// C line 52341
5 => {
let _ = { let assigned = kind; (*(it)).kind = assigned; assigned };
vm_block = 4; continue;
}
// C line 52340
6 => {
let _ = { let assigned = JS_DupValue(ctx, this_val); (*(it)).obj = assigned; assigned };
vm_block = 5; continue;
}
// C line 52338
7 => {
vm_block = 1; continue;
}
// C line 52337
8 => {
let _ = JS_FreeValue(ctx, enum_obj);
vm_block = 7; continue;
}
// C line 52336
9 => {
vm_block = if ((!(!(it).is_null()) as i32)) != 0 { 8 } else { 6 }; continue;
}
// C line 52335
10 => {
let _ = { let assigned = ((js_malloc(ctx, (size_of::<JSMapIteratorData>() as usize))) as *mut JSMapIteratorData); it = assigned; assigned };
vm_block = 9; continue;
}
// C line 52334
11 => {
vm_block = 1; continue;
}
// C line 52333
12 => {
vm_block = if (JS_IsException(enum_obj)) != 0 { 11 } else { 10 }; continue;
}
// C line 52332
13 => {
let _ = { let assigned = JS_NewObjectClass(ctx, ((JS_CLASS_MAP_ITERATOR as i32)).wrapping_add(magic)); enum_obj = assigned; assigned };
vm_block = 12; continue;
}
// C line 52331
14 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 52330
15 => {
vm_block = if ((!(!(s).is_null()) as i32)) != 0 { 14 } else { 13 }; continue;
}
// C line 52329
16 => {
let _ = { let assigned = ((JS_GetOpaque2(ctx, this_val, ((((JS_CLASS_MAP as i32)).wrapping_add(magic)) as JSClassID))) as *mut JSMapState); s = assigned; assigned };
vm_block = 15; continue;
}
// C line 52328
17 => {
let _ = { magic = ((magic) & ((3 as i32))); magic };
vm_block = 16; continue;
}
// C line 52327
18 => {
let _ = { let assigned = (((magic).wrapping_shr(((2 as i32)) as u32)) as JSIteratorKindEnum); kind = assigned; assigned };
vm_block = 17; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:52349. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code, unused_variables)]
unsafe fn js_map_iterator_next(mut ctx: *mut JSContext, mut this_val: JSValue, mut argc: i32, mut argv: *mut JSValue, mut pdone: *mut i32, mut magic: i32) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut it: *mut JSMapIteratorData = core::mem::zeroed();
let mut s: *mut JSMapState = core::mem::zeroed();
let mut mr: *mut JSMapRecord = core::mem::zeroed();
let mut el: *mut list_head = core::mem::zeroed();
let mut args: [JSValue; 2] = core::mem::zeroed();
let mut vm_block: usize = 36;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 52398
1 => {
return JS_DupValue(ctx, (*(mr)).key);
}
// C line 52407
2 => {
return JS_DupValue(ctx, *((args).as_mut_ptr()).offset(((1 as i32)) as isize));
}
// C line 52409
3 => {
return js_create_array(ctx, (2 as i32), (args).as_mut_ptr());
}
// C line 52406
4 => {
vm_block = if (((((((*(it)).kind) as u32)) == ((((JS_ITERATOR_KIND_VALUE as i32)) as u32))) as i32)) != 0 { 2 } else { 3 }; continue;
}
// C line 52403
5 => {
let _ = { let assigned = (*(mr)).key; *((args).as_mut_ptr()).offset(((1 as i32)) as isize) = assigned; assigned };
vm_block = 4; continue;
}
// C line 52405
6 => {
let _ = { let assigned = (*(mr)).value; *((args).as_mut_ptr()).offset(((1 as i32)) as isize) = assigned; assigned };
vm_block = 4; continue;
}
// C line 52402
7 => {
vm_block = if (magic) != 0 { 5 } else { 6 }; continue;
}
// C line 52401
8 => {
let _ = { let assigned = (*(mr)).key; *((args).as_mut_ptr()).offset(((0 as i32)) as isize) = assigned; assigned };
vm_block = 7; continue;
}
// C line 52397
9 => {
vm_block = if (((((((*(it)).kind) as u32)) == ((((JS_ITERATOR_KIND_KEY as i32)) as u32))) as i32)) != 0 { 1 } else { 8 }; continue;
}
// C line 52395
10 => {
let _ = { let assigned = (0 as i32); *(pdone) = assigned; assigned };
vm_block = 9; continue;
}
// C line 52394
11 => {
let _ = { let assigned = mr; (*(it)).cur_record = assigned; assigned };
vm_block = 10; continue;
}
// C line 52393
12 => {
let _ = { let old = (*(mr)).ref_count; (*(mr)).ref_count = ((*(mr)).ref_count).wrapping_add(1); old };
vm_block = 11; continue;
}
// C line 52374
13 => {
vm_block = 23; continue;
}
// C line 52389
14 => {
let _ = { let assigned = ((*(mr)).link).next; el = assigned; assigned };
vm_block = 13; continue;
}
// C line 52387
15 => {
vm_block = 12; continue;
}
// C line 52386
16 => {
vm_block = if ((!(((*(mr)).empty) != 0) as i32)) != 0 { 15 } else { 14 }; continue;
}
// C line 52385
17 => {
let _ = { let assigned = (((((el) as *mut u8)).offset(-(((core::mem::offset_of!(JSMapRecord, link) as usize)) as isize))) as *mut JSMapRecord); mr = assigned; assigned };
vm_block = 16; continue;
}
// C line 52383
18 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) };
}
// C line ? labels: done
19 => {
let _ = { let assigned = (1 as i32); *(pdone) = assigned; assigned };
vm_block = 18; continue;
}
// C line 52379
20 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; (*(it)).obj = assigned; assigned };
vm_block = 19; continue;
}
// C line 52378
21 => {
let _ = JS_FreeValue(ctx, (*(it)).obj);
vm_block = 20; continue;
}
// C line 52377
22 => {
let _ = { let assigned = core::ptr::null_mut::<JSMapRecord>(); (*(it)).cur_record = assigned; assigned };
vm_block = 21; continue;
}
// C line 52375
23 => {
vm_block = if ((((el) == (core::ptr::addr_of_mut!((*(s)).records))) as i32)) != 0 { 22 } else { 17 }; continue;
}
// C line 52368
24 => {
let _ = { let assigned = ((*(s)).records).next; el = assigned; assigned };
vm_block = 13; continue;
}
// C line 52372
25 => {
let _ = map_decref_record((*(ctx)).rt, mr);
vm_block = 13; continue;
}
// C line 52371
26 => {
let _ = { let assigned = ((*(mr)).link).next; el = assigned; assigned };
vm_block = 25; continue;
}
// C line 52370
27 => {
let _ = { let assigned = (*(it)).cur_record; mr = assigned; assigned };
vm_block = 26; continue;
}
// C line 52367
28 => {
vm_block = if ((!(!((*(it)).cur_record).is_null()) as i32)) != 0 { 24 } else { 27 }; continue;
}
// C line 52366
29 => {
let _ = if ((((!(((((s) != (core::ptr::null_mut::<JSMapState>())) as i32)) != 0) as i32)) as i64)) != 0 { std::process::abort() } else { { let _ = (0 as i32); } };
vm_block = 28; continue;
}
// C line 52365
30 => {
let _ = { let assigned = ((JS_GetOpaque((*(it)).obj, ((((JS_CLASS_MAP as i32)).wrapping_add(magic)) as JSClassID))) as *mut JSMapState); s = assigned; assigned };
vm_block = 29; continue;
}
// C line 52364
31 => {
vm_block = 19; continue;
}
// C line 52363
32 => {
vm_block = if (JS_IsUndefined((*(it)).obj)) != 0 { 31 } else { 30 }; continue;
}
// C line 52361
33 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 52360
34 => {
let _ = { let assigned = (0 as i32); *(pdone) = assigned; assigned };
vm_block = 33; continue;
}
// C line 52359
35 => {
vm_block = if ((!(!(it).is_null()) as i32)) != 0 { 34 } else { 32 }; continue;
}
// C line 52358
36 => {
let _ = { let assigned = ((JS_GetOpaque2(ctx, this_val, ((((JS_CLASS_MAP_ITERATOR as i32)).wrapping_add(magic)) as JSClassID))) as *mut JSMapIteratorData); it = assigned; assigned };
vm_block = 35; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:52414. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code, unused_variables)]
unsafe fn get_set_record(mut ctx: *mut JSContext, mut obj: JSValue, mut psize: *mut i64, mut phas: *mut JSValue, mut pkeys: *mut JSValue) -> i32 {
let mut vm_local_storage = Vec::<u64>::new();
let mut s: *mut JSMapState = core::mem::zeroed();
let mut size: i64 = core::mem::zeroed();
let mut has: JSValue = core::mem::zeroed();
let mut keys: JSValue = core::mem::zeroed();
let mut v: JSValue = core::mem::zeroed();
let mut d: f64 = core::mem::zeroed();
let mut vm_block: usize = 48;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 52483
1 => {
return ((1 as i32)).wrapping_neg();
}
// C line 52482
2 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; *(pkeys) = assigned; assigned };
vm_block = 1; continue;
}
// C line 52481
3 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; *(phas) = assigned; assigned };
vm_block = 2; continue;
}
// C line 52480
4 => {
let _ = { let assigned = (((0 as i32)) as i64); *(psize) = assigned; assigned };
vm_block = 3; continue;
}
// C line 52479
5 => {
let _ = JS_FreeValue(ctx, keys);
vm_block = 4; continue;
}
// C line ? labels: exception
6 => {
let _ = JS_FreeValue(ctx, has);
vm_block = 5; continue;
}
// C line 52475
7 => {
return (0 as i32);
}
// C line 52474
8 => {
let _ = { let assigned = keys; *(pkeys) = assigned; assigned };
vm_block = 7; continue;
}
// C line 52473
9 => {
let _ = { let assigned = has; *(phas) = assigned; assigned };
vm_block = 8; continue;
}
// C line 52472
10 => {
let _ = { let assigned = size; *(psize) = assigned; assigned };
vm_block = 9; continue;
}
// C line 52470
11 => {
vm_block = 6; continue;
}
// C line 52469
12 => {
let _ = JS_ThrowTypeError(ctx, c".keys is not a function".as_ptr());
vm_block = 11; continue;
}
// C line 52468
13 => {
vm_block = if ((!((JS_IsFunction(ctx, keys)) != 0) as i32)) != 0 { 12 } else { 10 }; continue;
}
// C line 52466
14 => {
vm_block = 6; continue;
}
// C line 52465
15 => {
let _ = JS_ThrowTypeError(ctx, c".keys is undefined".as_ptr());
vm_block = 14; continue;
}
// C line 52464
16 => {
vm_block = if (JS_IsUndefined(keys)) != 0 { 15 } else { 13 }; continue;
}
// C line 52463
17 => {
vm_block = 6; continue;
}
// C line 52462
18 => {
vm_block = if (JS_IsException(keys)) != 0 { 17 } else { 16 }; continue;
}
// C line 52461
19 => {
let _ = { let assigned = JS_GetProperty(ctx, obj, (((crate::quickjs_atom::JS_ATOM_keys as i32)) as JSAtom)); keys = assigned; assigned };
vm_block = 18; continue;
}
// C line 52458
20 => {
vm_block = 6; continue;
}
// C line 52457
21 => {
let _ = JS_ThrowTypeError(ctx, c".has is not a function".as_ptr());
vm_block = 20; continue;
}
// C line 52456
22 => {
vm_block = if ((!((JS_IsFunction(ctx, has)) != 0) as i32)) != 0 { 21 } else { 19 }; continue;
}
// C line 52454
23 => {
vm_block = 6; continue;
}
// C line 52453
24 => {
let _ = JS_ThrowTypeError(ctx, c".has is undefined".as_ptr());
vm_block = 23; continue;
}
// C line 52452
25 => {
vm_block = if (JS_IsUndefined(has)) != 0 { 24 } else { 22 }; continue;
}
// C line 52451
26 => {
vm_block = 6; continue;
}
// C line 52450
27 => {
vm_block = if (JS_IsException(has)) != 0 { 26 } else { 25 }; continue;
}
// C line 52449
28 => {
let _ = { let assigned = JS_GetProperty(ctx, obj, (((crate::quickjs_atom::JS_ATOM_has as i32)) as JSAtom)); has = assigned; assigned };
vm_block = 27; continue;
}
// C line 52423
29 => {
let _ = { let assigned = (((*(s)).record_count) as i64); size = assigned; assigned };
vm_block = 28; continue;
}
// C line 52445
30 => {
vm_block = 6; continue;
}
// C line 52444
31 => {
let _ = JS_ThrowRangeError(ctx, c".size must be positive".as_ptr());
vm_block = 30; continue;
}
// C line 52443
32 => {
vm_block = if ((((size) < ((((0 as i32)) as i64))) as i32)) != 0 { 31 } else { 28 }; continue;
}
// C line 52438
33 => {
let _ = { let assigned = (((9223372036854775807 as i64)).wrapping_neg()).wrapping_sub((((1 as i32)) as i64)); size = assigned; assigned };
vm_block = 32; continue;
}
// C line 52440
34 => {
let _ = { let assigned = (9223372036854775807 as i64); size = assigned; assigned };
vm_block = 32; continue;
}
// C line 52442
35 => {
let _ = { let assigned = ((d) as i64); size = assigned; assigned };
vm_block = 32; continue;
}
// C line 52439
36 => {
vm_block = if ((((d) >= ((9.2233720368547758E+18 as f64))) as i32)) != 0 { 34 } else { 35 }; continue;
}
// C line 52437
37 => {
vm_block = if ((((d) < ((((((9223372036854775807 as i64)).wrapping_neg()).wrapping_sub((((1 as i32)) as i64))) as f64))) as i32)) != 0 { 33 } else { 36 }; continue;
}
// C line 52435
38 => {
vm_block = 6; continue;
}
// C line 52434
39 => {
let _ = JS_ThrowTypeError(ctx, c".size is not a number".as_ptr());
vm_block = 38; continue;
}
// C line 52433
40 => {
vm_block = if ((d).is_nan() as i32) != 0 { 39 } else { 37 }; continue;
}
// C line 52432
41 => {
vm_block = 6; continue;
}
// C line 52431
42 => {
vm_block = if ((((JS_ToFloat64Free(ctx, core::ptr::addr_of_mut!(d), v)) < ((0 as i32))) as i32)) != 0 { 41 } else { 40 }; continue;
}
// C line 52430
43 => {
vm_block = 6; continue;
}
// C line 52429
44 => {
vm_block = if (JS_IsException(v)) != 0 { 43 } else { 42 }; continue;
}
// C line 52428
45 => {
let _ = { let assigned = JS_GetProperty(ctx, obj, (((crate::quickjs_atom::JS_ATOM_size as i32)) as JSAtom)); v = assigned; assigned };
vm_block = 44; continue;
}
// C line 52422
46 => {
vm_block = if !(s).is_null() { 29 } else { 45 }; continue;
}
// C line 52421
47 => {
let _ = { let assigned = ((JS_GetOpaque(obj, (((JS_CLASS_SET as i32)) as JSClassID))) as *mut JSMapState); s = assigned; assigned };
vm_block = 46; continue;
}
// C line 52419
48 => {
has = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) };
keys = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) };
vm_block = 47; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:52487. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code, unused_variables)]
unsafe fn js_copy_set(mut ctx: *mut JSContext, mut this_val: JSValue) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut newset: JSValue = core::mem::zeroed();
let mut s: *mut JSMapState = core::mem::zeroed();
let mut t: *mut JSMapState = core::mem::zeroed();
let mut el: *mut list_head = core::mem::zeroed();
let mut mr: *mut JSMapRecord = core::mem::zeroed();
let mut vm_block: usize = 18;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 52515
1 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line ? labels: exception
2 => {
let _ = JS_FreeValue(ctx, newset);
vm_block = 1; continue;
}
// C line 52512
3 => {
return newset;
}
// C line 52505
4 => {
vm_block = if ((((el) != (core::ptr::addr_of_mut!((*(s)).records))) as i32)) != 0 { 10 } else { 3 }; continue;
}
// C line ?
5 => {
let _ = { let assigned = (*(el)).next; el = assigned; assigned };
vm_block = 4; continue;
}
// C line 52510
6 => {
vm_block = 2; continue;
}
// C line 52509
7 => {
vm_block = if ((!(!(set_add_record(ctx, t, (*(mr)).key)).is_null()) as i32)) != 0 { 6 } else { 5 }; continue;
}
// C line 52508
8 => {
vm_block = 5; continue;
}
// C line 52507
9 => {
vm_block = if ((*(mr)).empty) != 0 { 8 } else { 7 }; continue;
}
// C line 52506
10 => {
let _ = { let assigned = (((((el) as *mut u8)).offset(-(((core::mem::offset_of!(JSMapRecord, link) as usize)) as isize))) as *mut JSMapRecord); mr = assigned; assigned };
vm_block = 9; continue;
}
// C line ?
11 => {
let _ = { let assigned = (*(core::ptr::addr_of_mut!((*(s)).records))).next; el = assigned; assigned };
vm_block = 4; continue;
}
// C line 52501
12 => {
let _ = { let assigned = ((JS_GetOpaque(newset, (((JS_CLASS_SET as i32)) as JSClassID))) as *mut JSMapState); t = assigned; assigned };
vm_block = 11; continue;
}
// C line 52500
13 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 52499
14 => {
vm_block = if (JS_IsException(newset)) != 0 { 13 } else { 12 }; continue;
}
// C line 52498
15 => {
let _ = { let assigned = js_map_constructor(ctx, JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }, (0 as i32), core::ptr::null_mut::<JSValue>(), ((1 as i32)).wrapping_shl(((0 as i32)) as u32)); newset = assigned; assigned };
vm_block = 14; continue;
}
// C line 52496
16 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 52495
17 => {
vm_block = if ((!(!(s).is_null()) as i32)) != 0 { 16 } else { 15 }; continue;
}
// C line 52494
18 => {
let _ = { let assigned = ((JS_GetOpaque2(ctx, this_val, (((JS_CLASS_SET as i32)) as JSClassID))) as *mut JSMapState); s = assigned; assigned };
vm_block = 17; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:52518. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code, unused_variables)]
unsafe fn js_set_isDisjointFrom(mut ctx: *mut JSContext, mut this_val: JSValue, mut argc: i32, mut argv: *mut JSValue) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut item: JSValue = core::mem::zeroed();
let mut iter: JSValue = core::mem::zeroed();
let mut keys: JSValue = core::mem::zeroed();
let mut has: JSValue = core::mem::zeroed();
let mut next: JSValue = core::mem::zeroed();
let mut rv: JSValue = core::mem::zeroed();
let mut rval: JSValue = core::mem::zeroed();
let mut done: i32 = core::mem::zeroed();
let mut found: i32 = core::mem::zeroed();
let mut s: *mut JSMapState = core::mem::zeroed();
let mut size: i64 = core::mem::zeroed();
let mut ok: i32 = core::mem::zeroed();
let mut vm_block: usize = 50;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 52583
1 => {
return rval;
}
// C line 52582
2 => {
let _ = JS_FreeValue(ctx, next);
vm_block = 1; continue;
}
// C line 52581
3 => {
let _ = JS_FreeValue(ctx, iter);
vm_block = 2; continue;
}
// C line 52580
4 => {
let _ = JS_FreeValue(ctx, keys);
vm_block = 3; continue;
}
// C line ? labels: exception
5 => {
let _ = JS_FreeValue(ctx, has);
vm_block = 4; continue;
}
// C line 52577
6 => {
let _ = { let assigned = if ((!((found) != 0) as i32)) != 0 { JSValue { u: JSValueUnion { uint64: (((((1 as i32)) as u32)) as u64) }, tag: (((JS_TAG_BOOL as i32)) as i64) } } else { JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_BOOL as i32)) as i64) } }; rval = assigned; assigned };
vm_block = 5; continue;
}
// C line 52541
7 => {
vm_block = if ((!((found) != 0) as i32)) != 0 { 18 } else { 6 }; continue;
}
// C line 52552
8 => {
let _ = { let assigned = (((ok) > ((0 as i32))) as i32); found = assigned; assigned };
vm_block = 7; continue;
}
// C line 52551
9 => {
vm_block = 5; continue;
}
// C line 52550
10 => {
vm_block = if ((((ok) < ((0 as i32))) as i32)) != 0 { 9 } else { 8 }; continue;
}
// C line 52549
11 => {
let _ = { let assigned = JS_ToBoolFree(ctx, rv); ok = assigned; assigned };
vm_block = 10; continue;
}
// C line 52548
12 => {
let _ = JS_FreeValue(ctx, item);
vm_block = 11; continue;
}
// C line 52547
13 => {
let _ = { let assigned = JS_Call(ctx, has, *(argv).offset(((0 as i32)) as isize), (1 as i32), core::ptr::addr_of_mut!(item)); rv = assigned; assigned };
vm_block = 12; continue;
}
// C line 52546
14 => {
vm_block = 6; continue;
}
// C line 52545
15 => {
vm_block = if (done) != 0 { 14 } else { 13 }; continue;
}
// C line 52544
16 => {
vm_block = 5; continue;
}
// C line 52543
17 => {
vm_block = if (JS_IsException(item)) != 0 { 16 } else { 15 }; continue;
}
// C line 52542
18 => {
let _ = { let assigned = js_map_iterator_next(ctx, iter, (0 as i32), core::ptr::null_mut::<JSValue>(), core::ptr::addr_of_mut!(done), ((1 as i32)).wrapping_shl(((0 as i32)) as u32)); item = assigned; assigned };
vm_block = 17; continue;
}
// C line 52540
19 => {
let _ = { let assigned = (0 as i32); found = assigned; assigned };
vm_block = 18; continue;
}
// C line 52539
20 => {
vm_block = 5; continue;
}
// C line 52538
21 => {
vm_block = if (JS_IsException(iter)) != 0 { 20 } else { 19 }; continue;
}
// C line 52537
22 => {
let _ = { let assigned = js_create_map_iterator(ctx, this_val, (0 as i32), core::ptr::null_mut::<JSValue>(), ((1 as i32)).wrapping_shl(((0 as i32)) as u32)); iter = assigned; assigned };
vm_block = 21; continue;
}
// C line 52562
23 => {
vm_block = 34; continue;
}
// C line 52573
24 => {
vm_block = 6; continue;
}
// C line 52572
25 => {
let _ = JS_IteratorClose(ctx, iter, (0 as i32));
vm_block = 24; continue;
}
// C line 52571
26 => {
vm_block = if (found) != 0 { 25 } else { 23 }; continue;
}
// C line 52570
27 => {
let _ = JS_FreeValue(ctx, item);
vm_block = 26; continue;
}
// C line 52569
28 => {
let _ = { let assigned = (((core::ptr::null_mut::<JSMapRecord>()) != (map_find_record(ctx, s, item))) as i32); found = assigned; assigned };
vm_block = 27; continue;
}
// C line 52568
29 => {
let _ = { let assigned = map_normalize_key(ctx, item); item = assigned; assigned };
vm_block = 28; continue;
}
// C line 52567
30 => {
vm_block = 6; continue;
}
// C line 52566
31 => {
vm_block = if (done) != 0 { 30 } else { 29 }; continue;
}
// C line 52565
32 => {
vm_block = 5; continue;
}
// C line 52564
33 => {
vm_block = if (JS_IsException(item)) != 0 { 32 } else { 31 }; continue;
}
// C line 52563
34 => {
let _ = { let assigned = JS_IteratorNext(ctx, iter, next, (0 as i32), core::ptr::null_mut::<JSValue>(), core::ptr::addr_of_mut!(done)); item = assigned; assigned };
vm_block = 33; continue;
}
// C line 52561
35 => {
let _ = { let assigned = (0 as i32); found = assigned; assigned };
vm_block = 23; continue;
}
// C line 52560
36 => {
vm_block = 5; continue;
}
// C line 52559
37 => {
vm_block = if (JS_IsException(next)) != 0 { 36 } else { 35 }; continue;
}
// C line 52558
38 => {
let _ = { let assigned = JS_GetProperty(ctx, iter, (((crate::quickjs_atom::JS_ATOM_next as i32)) as JSAtom)); next = assigned; assigned };
vm_block = 37; continue;
}
// C line 52557
39 => {
vm_block = 5; continue;
}
// C line 52556
40 => {
vm_block = if (JS_IsException(iter)) != 0 { 39 } else { 38 }; continue;
}
// C line 52555
41 => {
let _ = { let assigned = JS_Call(ctx, keys, *(argv).offset(((0 as i32)) as isize), (0 as i32), core::ptr::null_mut::<JSValue>()); iter = assigned; assigned };
vm_block = 40; continue;
}
// C line 52536
42 => {
vm_block = if (((((((*(s)).record_count) as i64)) <= (size)) as i32)) != 0 { 22 } else { 41 }; continue;
}
// C line 52535
43 => {
vm_block = 5; continue;
}
// C line 52534
44 => {
vm_block = if ((((get_set_record(ctx, *(argv).offset(((0 as i32)) as isize), core::ptr::addr_of_mut!(size), core::ptr::addr_of_mut!(has), core::ptr::addr_of_mut!(keys))) < ((0 as i32))) as i32)) != 0 { 43 } else { 42 }; continue;
}
// C line 52533
45 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 52532
46 => {
vm_block = if ((!(!(s).is_null()) as i32)) != 0 { 45 } else { 44 }; continue;
}
// C line 52531
47 => {
let _ = { let assigned = ((JS_GetOpaque2(ctx, this_val, (((JS_CLASS_SET as i32)) as JSClassID))) as *mut JSMapState); s = assigned; assigned };
vm_block = 46; continue;
}
// C line 52530
48 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) }; rval = assigned; assigned };
vm_block = 47; continue;
}
// C line 52529
49 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; next = assigned; assigned };
vm_block = 48; continue;
}
// C line 52528
50 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; iter = assigned; assigned };
vm_block = 49; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:52586. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code, unused_variables)]
unsafe fn js_set_isSubsetOf(mut ctx: *mut JSContext, mut this_val: JSValue, mut argc: i32, mut argv: *mut JSValue) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut item: JSValue = core::mem::zeroed();
let mut iter: JSValue = core::mem::zeroed();
let mut keys: JSValue = core::mem::zeroed();
let mut has: JSValue = core::mem::zeroed();
let mut next: JSValue = core::mem::zeroed();
let mut rv: JSValue = core::mem::zeroed();
let mut rval: JSValue = core::mem::zeroed();
let mut found: i32 = core::mem::zeroed();
let mut s: *mut JSMapState = core::mem::zeroed();
let mut size: i64 = core::mem::zeroed();
let mut done: i32 = core::mem::zeroed();
let mut ok: i32 = core::mem::zeroed();
let mut vm_block: usize = 33;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 52630
1 => {
return rval;
}
// C line 52629
2 => {
let _ = JS_FreeValue(ctx, next);
vm_block = 1; continue;
}
// C line 52628
3 => {
let _ = JS_FreeValue(ctx, iter);
vm_block = 2; continue;
}
// C line 52627
4 => {
let _ = JS_FreeValue(ctx, keys);
vm_block = 3; continue;
}
// C line ? labels: exception
5 => {
let _ = JS_FreeValue(ctx, has);
vm_block = 4; continue;
}
// C line ? labels: fini
6 => {
let _ = { let assigned = if (found) != 0 { JSValue { u: JSValueUnion { uint64: (((((1 as i32)) as u32)) as u64) }, tag: (((JS_TAG_BOOL as i32)) as i64) } } else { JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_BOOL as i32)) as i64) } }; rval = assigned; assigned };
vm_block = 5; continue;
}
// C line 52610
7 => {
vm_block = if (found) != 0 { 18 } else { 6 }; continue;
}
// C line 52621
8 => {
let _ = { let assigned = (((ok) > ((0 as i32))) as i32); found = assigned; assigned };
vm_block = 7; continue;
}
// C line 52620
9 => {
vm_block = 5; continue;
}
// C line 52619
10 => {
vm_block = if ((((ok) < ((0 as i32))) as i32)) != 0 { 9 } else { 8 }; continue;
}
// C line 52618
11 => {
let _ = { let assigned = JS_ToBoolFree(ctx, rv); ok = assigned; assigned };
vm_block = 10; continue;
}
// C line 52617
12 => {
let _ = JS_FreeValue(ctx, item);
vm_block = 11; continue;
}
// C line 52616
13 => {
let _ = { let assigned = JS_Call(ctx, has, *(argv).offset(((0 as i32)) as isize), (1 as i32), core::ptr::addr_of_mut!(item)); rv = assigned; assigned };
vm_block = 12; continue;
}
// C line 52615
14 => {
vm_block = 6; continue;
}
// C line 52614
15 => {
vm_block = if (done) != 0 { 14 } else { 13 }; continue;
}
// C line 52613
16 => {
vm_block = 5; continue;
}
// C line 52612
17 => {
vm_block = if (JS_IsException(item)) != 0 { 16 } else { 15 }; continue;
}
// C line 52611
18 => {
let _ = { let assigned = js_map_iterator_next(ctx, iter, (0 as i32), core::ptr::null_mut::<JSValue>(), core::ptr::addr_of_mut!(done), ((1 as i32)).wrapping_shl(((0 as i32)) as u32)); item = assigned; assigned };
vm_block = 17; continue;
}
// C line 52609
19 => {
let _ = { let assigned = (1 as i32); found = assigned; assigned };
vm_block = 18; continue;
}
// C line 52608
20 => {
vm_block = 5; continue;
}
// C line 52607
21 => {
vm_block = if (JS_IsException(iter)) != 0 { 20 } else { 19 }; continue;
}
// C line 52606
22 => {
let _ = { let assigned = js_create_map_iterator(ctx, this_val, (0 as i32), core::ptr::null_mut::<JSValue>(), ((1 as i32)).wrapping_shl(((0 as i32)) as u32)); iter = assigned; assigned };
vm_block = 21; continue;
}
// C line 52605
23 => {
vm_block = 6; continue;
}
// C line 52604
24 => {
vm_block = if (((((((*(s)).record_count) as i64)) > (size)) as i32)) != 0 { 23 } else { 22 }; continue;
}
// C line 52603
25 => {
let _ = { let assigned = (0 as i32); found = assigned; assigned };
vm_block = 24; continue;
}
// C line 52602
26 => {
vm_block = 5; continue;
}
// C line 52601
27 => {
vm_block = if ((((get_set_record(ctx, *(argv).offset(((0 as i32)) as isize), core::ptr::addr_of_mut!(size), core::ptr::addr_of_mut!(has), core::ptr::addr_of_mut!(keys))) < ((0 as i32))) as i32)) != 0 { 26 } else { 25 }; continue;
}
// C line 52600
28 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 52599
29 => {
vm_block = if ((!(!(s).is_null()) as i32)) != 0 { 28 } else { 27 }; continue;
}
// C line 52598
30 => {
let _ = { let assigned = ((JS_GetOpaque2(ctx, this_val, (((JS_CLASS_SET as i32)) as JSClassID))) as *mut JSMapState); s = assigned; assigned };
vm_block = 29; continue;
}
// C line 52597
31 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) }; rval = assigned; assigned };
vm_block = 30; continue;
}
// C line 52596
32 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; next = assigned; assigned };
vm_block = 31; continue;
}
// C line 52595
33 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; iter = assigned; assigned };
vm_block = 32; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:52633. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code, unused_variables)]
unsafe fn js_set_isSupersetOf(mut ctx: *mut JSContext, mut this_val: JSValue, mut argc: i32, mut argv: *mut JSValue) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut item: JSValue = core::mem::zeroed();
let mut iter: JSValue = core::mem::zeroed();
let mut keys: JSValue = core::mem::zeroed();
let mut has: JSValue = core::mem::zeroed();
let mut next: JSValue = core::mem::zeroed();
let mut rval: JSValue = core::mem::zeroed();
let mut done: i32 = core::mem::zeroed();
let mut found: i32 = core::mem::zeroed();
let mut s: *mut JSMapState = core::mem::zeroed();
let mut size: i64 = core::mem::zeroed();
let mut vm_block: usize = 36;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 52681
1 => {
return rval;
}
// C line 52680
2 => {
let _ = JS_FreeValue(ctx, next);
vm_block = 1; continue;
}
// C line 52679
3 => {
let _ = JS_FreeValue(ctx, iter);
vm_block = 2; continue;
}
// C line 52678
4 => {
let _ = JS_FreeValue(ctx, keys);
vm_block = 3; continue;
}
// C line ? labels: exception
5 => {
let _ = JS_FreeValue(ctx, has);
vm_block = 4; continue;
}
// C line ? labels: fini
6 => {
let _ = { let assigned = if (found) != 0 { JSValue { u: JSValueUnion { uint64: (((((1 as i32)) as u32)) as u64) }, tag: (((JS_TAG_BOOL as i32)) as i64) } } else { JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_BOOL as i32)) as i64) } }; rval = assigned; assigned };
vm_block = 5; continue;
}
// C line 52660
7 => {
vm_block = 18; continue;
}
// C line 52671
8 => {
vm_block = 6; continue;
}
// C line 52670
9 => {
let _ = JS_IteratorClose(ctx, iter, (0 as i32));
vm_block = 8; continue;
}
// C line 52669
10 => {
vm_block = if ((!((found) != 0) as i32)) != 0 { 9 } else { 7 }; continue;
}
// C line 52668
11 => {
let _ = JS_FreeValue(ctx, item);
vm_block = 10; continue;
}
// C line 52667
12 => {
let _ = { let assigned = (((core::ptr::null_mut::<JSMapRecord>()) != (map_find_record(ctx, s, item))) as i32); found = assigned; assigned };
vm_block = 11; continue;
}
// C line 52666
13 => {
let _ = { let assigned = map_normalize_key(ctx, item); item = assigned; assigned };
vm_block = 12; continue;
}
// C line 52665
14 => {
vm_block = 6; continue;
}
// C line 52664
15 => {
vm_block = if (done) != 0 { 14 } else { 13 }; continue;
}
// C line 52663
16 => {
vm_block = 5; continue;
}
// C line 52662
17 => {
vm_block = if (JS_IsException(item)) != 0 { 16 } else { 15 }; continue;
}
// C line 52661
18 => {
let _ = { let assigned = JS_IteratorNext(ctx, iter, next, (0 as i32), core::ptr::null_mut::<JSValue>(), core::ptr::addr_of_mut!(done)); item = assigned; assigned };
vm_block = 17; continue;
}
// C line 52659
19 => {
let _ = { let assigned = (1 as i32); found = assigned; assigned };
vm_block = 7; continue;
}
// C line 52658
20 => {
vm_block = 5; continue;
}
// C line 52657
21 => {
vm_block = if (JS_IsException(next)) != 0 { 20 } else { 19 }; continue;
}
// C line 52656
22 => {
let _ = { let assigned = JS_GetProperty(ctx, iter, (((crate::quickjs_atom::JS_ATOM_next as i32)) as JSAtom)); next = assigned; assigned };
vm_block = 21; continue;
}
// C line 52655
23 => {
vm_block = 5; continue;
}
// C line 52654
24 => {
vm_block = if (JS_IsException(iter)) != 0 { 23 } else { 22 }; continue;
}
// C line 52653
25 => {
let _ = { let assigned = JS_Call(ctx, keys, *(argv).offset(((0 as i32)) as isize), (0 as i32), core::ptr::null_mut::<JSValue>()); iter = assigned; assigned };
vm_block = 24; continue;
}
// C line 52652
26 => {
vm_block = 6; continue;
}
// C line 52651
27 => {
vm_block = if (((((((*(s)).record_count) as i64)) < (size)) as i32)) != 0 { 26 } else { 25 }; continue;
}
// C line 52650
28 => {
let _ = { let assigned = (0 as i32); found = assigned; assigned };
vm_block = 27; continue;
}
// C line 52649
29 => {
vm_block = 5; continue;
}
// C line 52648
30 => {
vm_block = if ((((get_set_record(ctx, *(argv).offset(((0 as i32)) as isize), core::ptr::addr_of_mut!(size), core::ptr::addr_of_mut!(has), core::ptr::addr_of_mut!(keys))) < ((0 as i32))) as i32)) != 0 { 29 } else { 28 }; continue;
}
// C line 52647
31 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 52646
32 => {
vm_block = if ((!(!(s).is_null()) as i32)) != 0 { 31 } else { 30 }; continue;
}
// C line 52645
33 => {
let _ = { let assigned = ((JS_GetOpaque2(ctx, this_val, (((JS_CLASS_SET as i32)) as JSClassID))) as *mut JSMapState); s = assigned; assigned };
vm_block = 32; continue;
}
// C line 52644
34 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) }; rval = assigned; assigned };
vm_block = 33; continue;
}
// C line 52643
35 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; next = assigned; assigned };
vm_block = 34; continue;
}
// C line 52642
36 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; iter = assigned; assigned };
vm_block = 35; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:52684. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code, unused_variables)]
unsafe fn js_set_intersection(mut ctx: *mut JSContext, mut this_val: JSValue, mut argc: i32, mut argv: *mut JSValue) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut newset: JSValue = core::mem::zeroed();
let mut item: JSValue = core::mem::zeroed();
let mut iter: JSValue = core::mem::zeroed();
let mut keys: JSValue = core::mem::zeroed();
let mut has: JSValue = core::mem::zeroed();
let mut next: JSValue = core::mem::zeroed();
let mut rv: JSValue = core::mem::zeroed();
let mut s: *mut JSMapState = core::mem::zeroed();
let mut t: *mut JSMapState = core::mem::zeroed();
let mut mr: *mut JSMapRecord = core::mem::zeroed();
let mut size: i64 = core::mem::zeroed();
let mut done: i32 = core::mem::zeroed();
let mut ok: i32 = core::mem::zeroed();
let mut vm_block: usize = 68;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 52772
1 => {
return newset;
}
// C line 52771
2 => {
let _ = JS_FreeValue(ctx, next);
vm_block = 1; continue;
}
// C line 52770
3 => {
let _ = JS_FreeValue(ctx, iter);
vm_block = 2; continue;
}
// C line 52769
4 => {
let _ = JS_FreeValue(ctx, keys);
vm_block = 3; continue;
}
// C line ? labels: fini
5 => {
let _ = JS_FreeValue(ctx, has);
vm_block = 4; continue;
}
// C line 52766
6 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) }; newset = assigned; assigned };
vm_block = 5; continue;
}
// C line ? labels: exception
7 => {
let _ = JS_FreeValue(ctx, newset);
vm_block = 6; continue;
}
// C line 52763
8 => {
vm_block = 5; continue;
}
// C line 52712
9 => {
vm_block = 23; continue;
}
// C line 52720
10 => {
let _ = JS_FreeValue(ctx, item);
vm_block = 9; continue;
}
// C line 52722
11 => {
let _ = JS_FreeValue(ctx, item);
vm_block = 9; continue;
}
// C line 52727
12 => {
vm_block = 7; continue;
}
// C line 52726
13 => {
vm_block = if ((!(!(mr).is_null()) as i32)) != 0 { 12 } else { 9 }; continue;
}
// C line 52725
14 => {
let _ = JS_FreeValue(ctx, item);
vm_block = 13; continue;
}
// C line 52724
15 => {
let _ = { let assigned = set_add_record(ctx, t, item); mr = assigned; assigned };
vm_block = 14; continue;
}
// C line 52721
16 => {
vm_block = if !(map_find_record(ctx, t, item)).is_null() { 11 } else { 15 }; continue;
}
// C line 52719
17 => {
vm_block = if ((!(!(map_find_record(ctx, s, item)).is_null()) as i32)) != 0 { 10 } else { 16 }; continue;
}
// C line 52718
18 => {
let _ = { let assigned = map_normalize_key(ctx, item); item = assigned; assigned };
vm_block = 17; continue;
}
// C line 52717
19 => {
vm_block = 8; continue;
}
// C line 52716
20 => {
vm_block = if (done) != 0 { 19 } else { 18 }; continue;
}
// C line 52715
21 => {
vm_block = 7; continue;
}
// C line 52714
22 => {
vm_block = if (JS_IsException(item)) != 0 { 21 } else { 20 }; continue;
}
// C line 52713
23 => {
let _ = { let assigned = JS_IteratorNext(ctx, iter, next, (0 as i32), core::ptr::null_mut::<JSValue>(), core::ptr::addr_of_mut!(done)); item = assigned; assigned };
vm_block = 22; continue;
}
// C line 52711
24 => {
let _ = { let assigned = ((JS_GetOpaque(newset, (((JS_CLASS_SET as i32)) as JSClassID))) as *mut JSMapState); t = assigned; assigned };
vm_block = 9; continue;
}
// C line 52710
25 => {
vm_block = 7; continue;
}
// C line 52709
26 => {
vm_block = if (JS_IsException(newset)) != 0 { 25 } else { 24 }; continue;
}
// C line 52708
27 => {
let _ = { let assigned = js_map_constructor(ctx, JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }, (0 as i32), core::ptr::null_mut::<JSValue>(), ((1 as i32)).wrapping_shl(((0 as i32)) as u32)); newset = assigned; assigned };
vm_block = 26; continue;
}
// C line 52707
28 => {
vm_block = 7; continue;
}
// C line 52706
29 => {
vm_block = if (JS_IsException(next)) != 0 { 28 } else { 27 }; continue;
}
// C line 52705
30 => {
let _ = { let assigned = JS_GetProperty(ctx, iter, (((crate::quickjs_atom::JS_ATOM_next as i32)) as JSAtom)); next = assigned; assigned };
vm_block = 29; continue;
}
// C line 52704
31 => {
vm_block = 7; continue;
}
// C line 52703
32 => {
vm_block = if (JS_IsException(iter)) != 0 { 31 } else { 30 }; continue;
}
// C line 52702
33 => {
let _ = { let assigned = JS_Call(ctx, keys, *(argv).offset(((0 as i32)) as isize), (0 as i32), core::ptr::null_mut::<JSValue>()); iter = assigned; assigned };
vm_block = 32; continue;
}
// C line 52738
34 => {
vm_block = 52; continue;
}
// C line 52749
35 => {
let _ = JS_FreeValue(ctx, item);
vm_block = 34; continue;
}
// C line 52754
36 => {
vm_block = 7; continue;
}
// C line 52753
37 => {
vm_block = if ((!(!(mr).is_null()) as i32)) != 0 { 36 } else { 34 }; continue;
}
// C line 52752
38 => {
let _ = JS_FreeValue(ctx, item);
vm_block = 37; continue;
}
// C line 52751
39 => {
let _ = { let assigned = set_add_record(ctx, t, item); mr = assigned; assigned };
vm_block = 38; continue;
}
// C line 52748
40 => {
vm_block = if !(map_find_record(ctx, t, item)).is_null() { 35 } else { 39 }; continue;
}
// C line 52747
41 => {
let _ = { let assigned = map_normalize_key(ctx, item); item = assigned; assigned };
vm_block = 40; continue;
}
// C line 52759
42 => {
vm_block = 7; continue;
}
// C line 52758
43 => {
vm_block = if ((((ok) < ((0 as i32))) as i32)) != 0 { 42 } else { 34 }; continue;
}
// C line 52757
44 => {
let _ = JS_FreeValue(ctx, item);
vm_block = 43; continue;
}
// C line 52746
45 => {
vm_block = if ((((ok) > ((0 as i32))) as i32)) != 0 { 41 } else { 44 }; continue;
}
// C line 52745
46 => {
let _ = { let assigned = JS_ToBoolFree(ctx, rv); ok = assigned; assigned };
vm_block = 45; continue;
}
// C line 52744
47 => {
let _ = { let assigned = JS_Call(ctx, has, *(argv).offset(((0 as i32)) as isize), (1 as i32), core::ptr::addr_of_mut!(item)); rv = assigned; assigned };
vm_block = 46; continue;
}
// C line 52743
48 => {
vm_block = 8; continue;
}
// C line 52742
49 => {
vm_block = if (done) != 0 { 48 } else { 47 }; continue;
}
// C line 52741
50 => {
vm_block = 7; continue;
}
// C line 52740
51 => {
vm_block = if (JS_IsException(item)) != 0 { 50 } else { 49 }; continue;
}
// C line 52739
52 => {
let _ = { let assigned = js_map_iterator_next(ctx, iter, (0 as i32), core::ptr::null_mut::<JSValue>(), core::ptr::addr_of_mut!(done), ((1 as i32)).wrapping_shl(((0 as i32)) as u32)); item = assigned; assigned };
vm_block = 51; continue;
}
// C line 52737
53 => {
let _ = { let assigned = ((JS_GetOpaque(newset, (((JS_CLASS_SET as i32)) as JSClassID))) as *mut JSMapState); t = assigned; assigned };
vm_block = 34; continue;
}
// C line 52736
54 => {
vm_block = 7; continue;
}
// C line 52735
55 => {
vm_block = if (JS_IsException(newset)) != 0 { 54 } else { 53 }; continue;
}
// C line 52734
56 => {
let _ = { let assigned = js_map_constructor(ctx, JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }, (0 as i32), core::ptr::null_mut::<JSValue>(), ((1 as i32)).wrapping_shl(((0 as i32)) as u32)); newset = assigned; assigned };
vm_block = 55; continue;
}
// C line 52733
57 => {
vm_block = 7; continue;
}
// C line 52732
58 => {
vm_block = if (JS_IsException(iter)) != 0 { 57 } else { 56 }; continue;
}
// C line 52731
59 => {
let _ = { let assigned = js_create_map_iterator(ctx, this_val, (0 as i32), core::ptr::null_mut::<JSValue>(), ((1 as i32)).wrapping_shl(((0 as i32)) as u32)); iter = assigned; assigned };
vm_block = 58; continue;
}
// C line 52701
60 => {
vm_block = if (((((((*(s)).record_count) as i64)) > (size)) as i32)) != 0 { 33 } else { 59 }; continue;
}
// C line 52700
61 => {
vm_block = 7; continue;
}
// C line 52699
62 => {
vm_block = if ((((get_set_record(ctx, *(argv).offset(((0 as i32)) as isize), core::ptr::addr_of_mut!(size), core::ptr::addr_of_mut!(has), core::ptr::addr_of_mut!(keys))) < ((0 as i32))) as i32)) != 0 { 61 } else { 60 }; continue;
}
// C line 52698
63 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 52697
64 => {
vm_block = if ((!(!(s).is_null()) as i32)) != 0 { 63 } else { 62 }; continue;
}
// C line 52696
65 => {
let _ = { let assigned = ((JS_GetOpaque2(ctx, this_val, (((JS_CLASS_SET as i32)) as JSClassID))) as *mut JSMapState); s = assigned; assigned };
vm_block = 64; continue;
}
// C line 52695
66 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; newset = assigned; assigned };
vm_block = 65; continue;
}
// C line 52694
67 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; next = assigned; assigned };
vm_block = 66; continue;
}
// C line 52693
68 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; iter = assigned; assigned };
vm_block = 67; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:52775. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code, unused_variables)]
unsafe fn js_set_difference(mut ctx: *mut JSContext, mut this_val: JSValue, mut argc: i32, mut argv: *mut JSValue) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut newset: JSValue = core::mem::zeroed();
let mut item: JSValue = core::mem::zeroed();
let mut iter: JSValue = core::mem::zeroed();
let mut keys: JSValue = core::mem::zeroed();
let mut has: JSValue = core::mem::zeroed();
let mut next: JSValue = core::mem::zeroed();
let mut rv: JSValue = core::mem::zeroed();
let mut s: *mut JSMapState = core::mem::zeroed();
let mut t: *mut JSMapState = core::mem::zeroed();
let mut size: i64 = core::mem::zeroed();
let mut done: i32 = core::mem::zeroed();
let mut ok: i32 = core::mem::zeroed();
let mut vm_block: usize = 52;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 52845
1 => {
return newset;
}
// C line 52844
2 => {
let _ = JS_FreeValue(ctx, next);
vm_block = 1; continue;
}
// C line 52843
3 => {
let _ = JS_FreeValue(ctx, iter);
vm_block = 2; continue;
}
// C line 52842
4 => {
let _ = JS_FreeValue(ctx, keys);
vm_block = 3; continue;
}
// C line ? labels: fini
5 => {
let _ = JS_FreeValue(ctx, has);
vm_block = 4; continue;
}
// C line 52839
6 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) }; newset = assigned; assigned };
vm_block = 5; continue;
}
// C line ? labels: exception
7 => {
let _ = JS_FreeValue(ctx, newset);
vm_block = 6; continue;
}
// C line 52836
8 => {
vm_block = 5; continue;
}
// C line 52802
9 => {
vm_block = 22; continue;
}
// C line 52817
10 => {
let _ = JS_FreeValue(ctx, item);
vm_block = 9; continue;
}
// C line 52815
11 => {
let _ = map_delete_record(ctx, t, item);
vm_block = 10; continue;
}
// C line 52814
12 => {
vm_block = if (ok) != 0 { 11 } else { 10 }; continue;
}
// C line 52812
13 => {
vm_block = 7; continue;
}
// C line 52811
14 => {
let _ = JS_FreeValue(ctx, item);
vm_block = 13; continue;
}
// C line 52810
15 => {
vm_block = if ((((ok) < ((0 as i32))) as i32)) != 0 { 14 } else { 12 }; continue;
}
// C line 52809
16 => {
let _ = { let assigned = JS_ToBoolFree(ctx, rv); ok = assigned; assigned };
vm_block = 15; continue;
}
// C line 52808
17 => {
let _ = { let assigned = JS_Call(ctx, has, *(argv).offset(((0 as i32)) as isize), (1 as i32), core::ptr::addr_of_mut!(item)); rv = assigned; assigned };
vm_block = 16; continue;
}
// C line 52807
18 => {
vm_block = 8; continue;
}
// C line 52806
19 => {
vm_block = if (done) != 0 { 18 } else { 17 }; continue;
}
// C line 52805
20 => {
vm_block = 7; continue;
}
// C line 52804
21 => {
vm_block = if (JS_IsException(item)) != 0 { 20 } else { 19 }; continue;
}
// C line 52803
22 => {
let _ = { let assigned = js_map_iterator_next(ctx, iter, (0 as i32), core::ptr::null_mut::<JSValue>(), core::ptr::addr_of_mut!(done), ((1 as i32)).wrapping_shl(((0 as i32)) as u32)); item = assigned; assigned };
vm_block = 21; continue;
}
// C line 52801
23 => {
vm_block = 7; continue;
}
// C line 52800
24 => {
vm_block = if (JS_IsException(iter)) != 0 { 23 } else { 9 }; continue;
}
// C line 52799
25 => {
let _ = { let assigned = js_create_map_iterator(ctx, newset, (0 as i32), core::ptr::null_mut::<JSValue>(), ((1 as i32)).wrapping_shl(((0 as i32)) as u32)); iter = assigned; assigned };
vm_block = 24; continue;
}
// C line 52826
26 => {
vm_block = 33; continue;
}
// C line 52833
27 => {
let _ = JS_FreeValue(ctx, item);
vm_block = 26; continue;
}
// C line 52832
28 => {
let _ = map_delete_record(ctx, t, item);
vm_block = 27; continue;
}
// C line 52831
29 => {
vm_block = 8; continue;
}
// C line 52830
30 => {
vm_block = if (done) != 0 { 29 } else { 28 }; continue;
}
// C line 52829
31 => {
vm_block = 7; continue;
}
// C line 52828
32 => {
vm_block = if (JS_IsException(item)) != 0 { 31 } else { 30 }; continue;
}
// C line 52827
33 => {
let _ = { let assigned = JS_IteratorNext(ctx, iter, next, (0 as i32), core::ptr::null_mut::<JSValue>(), core::ptr::addr_of_mut!(done)); item = assigned; assigned };
vm_block = 32; continue;
}
// C line 52825
34 => {
vm_block = 7; continue;
}
// C line 52824
35 => {
vm_block = if (JS_IsException(next)) != 0 { 34 } else { 26 }; continue;
}
// C line 52823
36 => {
let _ = { let assigned = JS_GetProperty(ctx, iter, (((crate::quickjs_atom::JS_ATOM_next as i32)) as JSAtom)); next = assigned; assigned };
vm_block = 35; continue;
}
// C line 52822
37 => {
vm_block = 7; continue;
}
// C line 52821
38 => {
vm_block = if (JS_IsException(iter)) != 0 { 37 } else { 36 }; continue;
}
// C line 52820
39 => {
let _ = { let assigned = JS_Call(ctx, keys, *(argv).offset(((0 as i32)) as isize), (0 as i32), core::ptr::null_mut::<JSValue>()); iter = assigned; assigned };
vm_block = 38; continue;
}
// C line 52798
40 => {
vm_block = if (((((((*(s)).record_count) as i64)) <= (size)) as i32)) != 0 { 25 } else { 39 }; continue;
}
// C line 52796
41 => {
let _ = { let assigned = ((JS_GetOpaque(newset, (((JS_CLASS_SET as i32)) as JSClassID))) as *mut JSMapState); t = assigned; assigned };
vm_block = 40; continue;
}
// C line 52795
42 => {
vm_block = 7; continue;
}
// C line 52794
43 => {
vm_block = if (JS_IsException(newset)) != 0 { 42 } else { 41 }; continue;
}
// C line 52793
44 => {
let _ = { let assigned = js_copy_set(ctx, this_val); newset = assigned; assigned };
vm_block = 43; continue;
}
// C line 52791
45 => {
vm_block = 7; continue;
}
// C line 52790
46 => {
vm_block = if ((((get_set_record(ctx, *(argv).offset(((0 as i32)) as isize), core::ptr::addr_of_mut!(size), core::ptr::addr_of_mut!(has), core::ptr::addr_of_mut!(keys))) < ((0 as i32))) as i32)) != 0 { 45 } else { 44 }; continue;
}
// C line 52789
47 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 52788
48 => {
vm_block = if ((!(!(s).is_null()) as i32)) != 0 { 47 } else { 46 }; continue;
}
// C line 52787
49 => {
let _ = { let assigned = ((JS_GetOpaque2(ctx, this_val, (((JS_CLASS_SET as i32)) as JSClassID))) as *mut JSMapState); s = assigned; assigned };
vm_block = 48; continue;
}
// C line 52786
50 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; newset = assigned; assigned };
vm_block = 49; continue;
}
// C line 52785
51 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; next = assigned; assigned };
vm_block = 50; continue;
}
// C line 52784
52 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; iter = assigned; assigned };
vm_block = 51; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:52848. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code, unused_variables)]
unsafe fn js_set_symmetricDifference(mut ctx: *mut JSContext, mut this_val: JSValue, mut argc: i32, mut argv: *mut JSValue) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut newset: JSValue = core::mem::zeroed();
let mut item: JSValue = core::mem::zeroed();
let mut iter: JSValue = core::mem::zeroed();
let mut next: JSValue = core::mem::zeroed();
let mut has: JSValue = core::mem::zeroed();
let mut keys: JSValue = core::mem::zeroed();
let mut s: *mut JSMapState = core::mem::zeroed();
let mut t: *mut JSMapState = core::mem::zeroed();
let mut mr: *mut JSMapRecord = core::mem::zeroed();
let mut size: i64 = core::mem::zeroed();
let mut done: i32 = core::mem::zeroed();
let mut present: i32 = core::mem::zeroed();
let mut vm_block: usize = 43;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 52914
1 => {
return newset;
}
// C line 52913
2 => {
let _ = JS_FreeValue(ctx, keys);
vm_block = 1; continue;
}
// C line 52912
3 => {
let _ = JS_FreeValue(ctx, iter);
vm_block = 2; continue;
}
// C line ? labels: fini
4 => {
let _ = JS_FreeValue(ctx, next);
vm_block = 3; continue;
}
// C line 52909
5 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) }; newset = assigned; assigned };
vm_block = 4; continue;
}
// C line ? labels: exception
6 => {
let _ = JS_FreeValue(ctx, newset);
vm_block = 5; continue;
}
// C line 52906
7 => {
vm_block = 4; continue;
}
// C line 52877
8 => {
vm_block = 25; continue;
}
// C line 52896
9 => {
let _ = JS_FreeValue(ctx, item);
vm_block = 8; continue;
}
// C line 52895
10 => {
let _ = map_delete_record(ctx, t, item);
vm_block = 9; continue;
}
// C line 52898
11 => {
let _ = JS_FreeValue(ctx, item);
vm_block = 8; continue;
}
// C line 52903
12 => {
vm_block = 6; continue;
}
// C line 52902
13 => {
vm_block = if ((!(!(mr).is_null()) as i32)) != 0 { 12 } else { 8 }; continue;
}
// C line 52901
14 => {
let _ = JS_FreeValue(ctx, item);
vm_block = 13; continue;
}
// C line 52900
15 => {
let _ = { let assigned = set_add_record(ctx, t, item); mr = assigned; assigned };
vm_block = 14; continue;
}
// C line 52897
16 => {
vm_block = if !(mr).is_null() { 11 } else { 15 }; continue;
}
// C line 52894
17 => {
vm_block = if (present) != 0 { 10 } else { 16 }; continue;
}
// C line 52893
18 => {
let _ = { let assigned = map_find_record(ctx, t, item); mr = assigned; assigned };
vm_block = 17; continue;
}
// C line 52892
19 => {
let _ = { let assigned = (((core::ptr::null_mut::<JSMapRecord>()) != (map_find_record(ctx, s, item))) as i32); present = assigned; assigned };
vm_block = 18; continue;
}
// C line 52891
20 => {
let _ = { let assigned = map_normalize_key(ctx, item); item = assigned; assigned };
vm_block = 19; continue;
}
// C line 52882
21 => {
vm_block = 7; continue;
}
// C line 52881
22 => {
vm_block = if (done) != 0 { 21 } else { 20 }; continue;
}
// C line 52880
23 => {
vm_block = 6; continue;
}
// C line 52879
24 => {
vm_block = if (JS_IsException(item)) != 0 { 23 } else { 22 }; continue;
}
// C line 52878
25 => {
let _ = { let assigned = JS_IteratorNext(ctx, iter, next, (0 as i32), core::ptr::null_mut::<JSValue>(), core::ptr::addr_of_mut!(done)); item = assigned; assigned };
vm_block = 24; continue;
}
// C line 52876
26 => {
let _ = { let assigned = ((JS_GetOpaque(newset, (((JS_CLASS_SET as i32)) as JSClassID))) as *mut JSMapState); t = assigned; assigned };
vm_block = 8; continue;
}
// C line 52875
27 => {
vm_block = 6; continue;
}
// C line 52874
28 => {
vm_block = if (JS_IsException(newset)) != 0 { 27 } else { 26 }; continue;
}
// C line 52873
29 => {
let _ = { let assigned = js_copy_set(ctx, this_val); newset = assigned; assigned };
vm_block = 28; continue;
}
// C line 52872
30 => {
vm_block = 6; continue;
}
// C line 52871
31 => {
vm_block = if (JS_IsException(next)) != 0 { 30 } else { 29 }; continue;
}
// C line 52870
32 => {
let _ = { let assigned = JS_GetProperty(ctx, iter, (((crate::quickjs_atom::JS_ATOM_next as i32)) as JSAtom)); next = assigned; assigned };
vm_block = 31; continue;
}
// C line 52869
33 => {
vm_block = 6; continue;
}
// C line 52868
34 => {
vm_block = if (JS_IsException(iter)) != 0 { 33 } else { 32 }; continue;
}
// C line 52867
35 => {
let _ = { let assigned = JS_Call(ctx, keys, *(argv).offset(((0 as i32)) as isize), (0 as i32), core::ptr::null_mut::<JSValue>()); iter = assigned; assigned };
vm_block = 34; continue;
}
// C line 52866
36 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; newset = assigned; assigned };
vm_block = 35; continue;
}
// C line 52865
37 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; next = assigned; assigned };
vm_block = 36; continue;
}
// C line 52863
38 => {
let _ = JS_FreeValue(ctx, has);
vm_block = 37; continue;
}
// C line 52862
39 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 52861
40 => {
vm_block = if ((((get_set_record(ctx, *(argv).offset(((0 as i32)) as isize), core::ptr::addr_of_mut!(size), core::ptr::addr_of_mut!(has), core::ptr::addr_of_mut!(keys))) < ((0 as i32))) as i32)) != 0 { 39 } else { 38 }; continue;
}
// C line 52860
41 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 52859
42 => {
vm_block = if ((!(!(s).is_null()) as i32)) != 0 { 41 } else { 40 }; continue;
}
// C line 52858
43 => {
let _ = { let assigned = ((JS_GetOpaque2(ctx, this_val, (((JS_CLASS_SET as i32)) as JSClassID))) as *mut JSMapState); s = assigned; assigned };
vm_block = 42; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:52917. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code, unused_variables)]
unsafe fn js_set_union(mut ctx: *mut JSContext, mut this_val: JSValue, mut argc: i32, mut argv: *mut JSValue) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut newset: JSValue = core::mem::zeroed();
let mut item: JSValue = core::mem::zeroed();
let mut iter: JSValue = core::mem::zeroed();
let mut next: JSValue = core::mem::zeroed();
let mut has: JSValue = core::mem::zeroed();
let mut keys: JSValue = core::mem::zeroed();
let mut rv: JSValue = core::mem::zeroed();
let mut s: *mut JSMapState = core::mem::zeroed();
let mut size: i64 = core::mem::zeroed();
let mut done: i32 = core::mem::zeroed();
let mut vm_block: usize = 35;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 52965
1 => {
return newset;
}
// C line 52964
2 => {
let _ = JS_FreeValue(ctx, keys);
vm_block = 1; continue;
}
// C line 52963
3 => {
let _ = JS_FreeValue(ctx, iter);
vm_block = 2; continue;
}
// C line ? labels: fini
4 => {
let _ = JS_FreeValue(ctx, next);
vm_block = 3; continue;
}
// C line 52960
5 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) }; newset = assigned; assigned };
vm_block = 4; continue;
}
// C line ? labels: exception
6 => {
let _ = JS_FreeValue(ctx, newset);
vm_block = 5; continue;
}
// C line 52957
7 => {
vm_block = 4; continue;
}
// C line 52945
8 => {
vm_block = 18; continue;
}
// C line 52955
9 => {
let _ = JS_FreeValue(ctx, rv);
vm_block = 8; continue;
}
// C line 52954
10 => {
vm_block = 6; continue;
}
// C line 52953
11 => {
vm_block = if (JS_IsException(rv)) != 0 { 10 } else { 9 }; continue;
}
// C line 52952
12 => {
let _ = JS_FreeValue(ctx, item);
vm_block = 11; continue;
}
// C line 52951
13 => {
let _ = { let assigned = js_map_set(ctx, newset, (1 as i32), core::ptr::addr_of_mut!(item), ((1 as i32)).wrapping_shl(((0 as i32)) as u32)); rv = assigned; assigned };
vm_block = 12; continue;
}
// C line 52950
14 => {
vm_block = 7; continue;
}
// C line 52949
15 => {
vm_block = if (done) != 0 { 14 } else { 13 }; continue;
}
// C line 52948
16 => {
vm_block = 6; continue;
}
// C line 52947
17 => {
vm_block = if (JS_IsException(item)) != 0 { 16 } else { 15 }; continue;
}
// C line 52946
18 => {
let _ = { let assigned = JS_IteratorNext(ctx, iter, next, (0 as i32), core::ptr::null_mut::<JSValue>(), core::ptr::addr_of_mut!(done)); item = assigned; assigned };
vm_block = 17; continue;
}
// C line 52943
19 => {
vm_block = 6; continue;
}
// C line 52942
20 => {
vm_block = if (JS_IsException(newset)) != 0 { 19 } else { 8 }; continue;
}
// C line 52941
21 => {
let _ = { let assigned = js_copy_set(ctx, this_val); newset = assigned; assigned };
vm_block = 20; continue;
}
// C line 52939
22 => {
vm_block = 6; continue;
}
// C line 52938
23 => {
vm_block = if (JS_IsException(next)) != 0 { 22 } else { 21 }; continue;
}
// C line 52937
24 => {
let _ = { let assigned = JS_GetProperty(ctx, iter, (((crate::quickjs_atom::JS_ATOM_next as i32)) as JSAtom)); next = assigned; assigned };
vm_block = 23; continue;
}
// C line 52936
25 => {
vm_block = 6; continue;
}
// C line 52935
26 => {
vm_block = if (JS_IsException(iter)) != 0 { 25 } else { 24 }; continue;
}
// C line 52934
27 => {
let _ = { let assigned = JS_Call(ctx, keys, *(argv).offset(((0 as i32)) as isize), (0 as i32), core::ptr::null_mut::<JSValue>()); iter = assigned; assigned };
vm_block = 26; continue;
}
// C line 52933
28 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; newset = assigned; assigned };
vm_block = 27; continue;
}
// C line 52932
29 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; next = assigned; assigned };
vm_block = 28; continue;
}
// C line 52930
30 => {
let _ = JS_FreeValue(ctx, has);
vm_block = 29; continue;
}
// C line 52929
31 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 52928
32 => {
vm_block = if ((((get_set_record(ctx, *(argv).offset(((0 as i32)) as isize), core::ptr::addr_of_mut!(size), core::ptr::addr_of_mut!(has), core::ptr::addr_of_mut!(keys))) < ((0 as i32))) as i32)) != 0 { 31 } else { 30 }; continue;
}
// C line 52927
33 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 52926
34 => {
vm_block = if ((!(!(s).is_null()) as i32)) != 0 { 33 } else { 32 }; continue;
}
// C line 52925
35 => {
let _ = { let assigned = ((JS_GetOpaque2(ctx, this_val, (((JS_CLASS_SET as i32)) as JSClassID))) as *mut JSMapState); s = assigned; assigned };
vm_block = 34; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:53060. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code, unused_variables)]
pub unsafe fn JS_AddIntrinsicMapSet(mut ctx: *mut JSContext) -> i32 {
let mut vm_local_storage = Vec::<u64>::new();
let mut i: i32 = core::mem::zeroed();
let mut obj1: JSValue = core::mem::zeroed();
let mut buf: [c_char; 64] = core::mem::zeroed();
let mut ft: JSCFunctionType = core::mem::zeroed();
let mut name: *const c_char = core::mem::zeroed();
let mut vm_block: usize = 16;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 53090
1 => {
return (0 as i32);
}
// C line 53082
2 => {
vm_block = if ((((i) < ((2 as i32))) as i32)) != 0 { 6 } else { 1 }; continue;
}
// C line ?
3 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 2; continue;
}
// C line 53088
4 => {
return ((1 as i32)).wrapping_neg();
}
// C line 53087
5 => {
vm_block = if (JS_IsException(*((*(ctx)).class_proto).offset((((JS_CLASS_MAP_ITERATOR as i32)).wrapping_add(i)) as isize))) != 0 { 4 } else { 3 }; continue;
}
// C line 53083
6 => {
let _ = { let assigned = JS_NewObjectProtoList(ctx, *((*(ctx)).class_proto).offset(((JS_CLASS_ITERATOR as i32)) as isize), *((js_map_proto_funcs_ptr).as_ptr()).offset(((i).wrapping_add((4 as i32))) as isize), ((*((js_map_proto_funcs_count).as_ptr()).offset(((i).wrapping_add((4 as i32))) as isize)) as i32)); *((*(ctx)).class_proto).offset((((JS_CLASS_MAP_ITERATOR as i32)).wrapping_add(i)) as isize) = assigned; assigned };
vm_block = 5; continue;
}
// C line 53082
7 => {
let _ = { let assigned = (0 as i32); i = assigned; assigned };
vm_block = 2; continue;
}
// C line 53066
8 => {
vm_block = if ((((i) < ((4 as i32))) as i32)) != 0 { 15 } else { 7 }; continue;
}
// C line ?
9 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 8; continue;
}
// C line 53079
10 => {
let _ = JS_FreeValue(ctx, obj1);
vm_block = 9; continue;
}
// C line 53078
11 => {
return ((1 as i32)).wrapping_neg();
}
// C line 53077
12 => {
vm_block = if (JS_IsException(obj1)) != 0 { 11 } else { 10 }; continue;
}
// C line 53071
13 => {
let _ = { let assigned = JS_NewCConstructor(ctx, ((JS_CLASS_MAP as i32)).wrapping_add(i), name, (ft).generic, (0 as i32), (((JS_CFUNC_constructor_magic as i32)) as JSCFunctionEnum), i, JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }, (js_map_funcs).as_ptr(), ((if ((((i) < ((2 as i32))) as i32)) != 0 { (((size_of::<[JSCFunctionListEntry; 2]>() as usize)) / ((size_of::<JSCFunctionListEntry>() as usize))) } else { (((0 as i32)) as usize) }) as i32), *((js_map_proto_funcs_ptr).as_ptr()).offset((i) as isize), ((*((js_map_proto_funcs_count).as_ptr()).offset((i) as isize)) as i32), (0 as i32)); obj1 = assigned; assigned };
vm_block = 12; continue;
}
// C line 53070
14 => {
let _ = { let assigned = js_map_constructor; (ft).constructor_magic = Some(assigned); assigned };
vm_block = 13; continue;
}
// C line 53068
15 => {
name = JS_AtomGetStr(ctx, (buf).as_mut_ptr(), (((size_of::<[c_char; 64]>() as usize)) as i32), ((((crate::quickjs_atom::JS_ATOM_Map as i32)).wrapping_add(i)) as JSAtom));
vm_block = 14; continue;
}
// C line 53066
16 => {
let _ = { let assigned = (0 as i32); i = assigned; assigned };
vm_block = 8; continue;
}
_ => std::process::abort(),
} }
}
