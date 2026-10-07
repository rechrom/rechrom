// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:24358. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn seal_template_obj(mut ctx: *mut JSContext, mut obj: JSValue) -> i32 {
let mut vm_local_storage = Vec::<u64>::new();
let mut p: *mut JSObject = core::ptr::null_mut();
let mut prs: *mut JSShapeProperty = core::ptr::null_mut();
let mut vm_block: usize = 7;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 24371
1 => {
return (0 as i32);
}
// C line 24370
2 => {
let _ = { let assigned = (((0 as i32)) as u8); (*(p)).set_extensible((assigned) as _); assigned };
vm_block = 1; continue;
}
// C line 24368
3 => {
return ((1 as i32)).wrapping_neg();
}
// C line 24366
4 => {
vm_block = if (js_update_property_flags(ctx, p, core::ptr::addr_of_mut!(prs), (((((*(prs)).flags()) as i32)) & ((!(((((1 as i32)).wrapping_shl(((0 as i32)) as u32)) | (((1 as i32)).wrapping_shl(((1 as i32)) as u32))))))))) != 0 { 3 } else { 2 }; continue;
}
// C line 24365
5 => {
vm_block = if !(prs).is_null() { 4 } else { 2 }; continue;
}
// C line 24364
6 => {
let _ = { let assigned = find_own_property1(p, (((crate::quickjs_atom::JS_ATOM_length as i32)) as JSAtom)); prs = assigned; assigned };
vm_block = 5; continue;
}
// C line 24363
7 => {
let _ = { let assigned = ((((obj).u).ptr) as *mut JSObject); p = assigned; assigned };
vm_block = 6; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:24374. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_parse_template(mut s: *mut JSParseState, mut call: i32, mut argc: *mut i32) -> i32 {
let mut vm_local_storage = Vec::<u64>::new();
let mut ctx: *mut JSContext = core::ptr::null_mut();
let mut raw_array: JSValue = core::mem::zeroed();
let mut template_object: JSValue = core::mem::zeroed();
let mut cooked: JSToken = core::mem::zeroed();
let mut depth: i32 = 0;
let mut ret: i32 = 0;
let mut p: *const u8 = core::ptr::null();
let mut str: *mut JSString = core::ptr::null_mut();
let mut vm_block: usize = 65;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 24481 labels: done1
1 => {
return next_token(s);
}
// C line 24475
2 => {
let _ = { let assigned = (depth).wrapping_add((1 as i32)); *(argc) = assigned; assigned };
vm_block = 1; continue;
}
// C line 24474
3 => {
let _ = seal_template_obj(ctx, template_object);
vm_block = 2; continue;
}
// C line 24473
4 => {
let _ = seal_template_obj(ctx, raw_array);
vm_block = 3; continue;
}
// C line 24478
5 => {
let _ = emit_u16(s, (((depth).wrapping_sub((1 as i32))) as u16));
vm_block = 1; continue;
}
// C line 24477
6 => {
let _ = emit_op(s, (((OP_call_method as i32)) as u8));
vm_block = 5; continue;
}
// C line 24471 labels: done
7 => {
vm_block = if (call) != 0 { 4 } else { 6 }; continue;
}
// C line 24468
8 => {
return js_parse_expect(s, (TOK_TEMPLATE as i32));
}
// C line 24404
9 => {
vm_block = if ((((((*(s)).token).val) == ((TOK_TEMPLATE as i32))) as i32)) != 0 { 48 } else { 8 }; continue;
}
// C line 24466
10 => {
return ((1 as i32)).wrapping_neg();
}
// C line 24465
11 => {
vm_block = if (js_parse_template_part(s, (*(s)).buf_ptr)) != 0 { 10 } else { 9 }; continue;
}
// C line 24464
12 => {
let _ = { let assigned = (0 as i32); (*(s)).got_lf = assigned; assigned };
vm_block = 11; continue;
}
// C line 24461
13 => {
let _ = free_token(s, core::ptr::addr_of_mut!((*(s)).token));
vm_block = 12; continue;
}
// C line 24458
14 => {
return js_parse_error(s, c"expected '}' after template expression".as_ptr());
}
// C line 24457
15 => {
vm_block = if ((((((*(s)).token).val) != ((125 as i32))) as i32)) != 0 { 14 } else { 13 }; continue;
}
// C line 24456
16 => {
let _ = { let old = depth; depth = (depth).wrapping_add(1); old };
vm_block = 15; continue;
}
// C line 24455
17 => {
return ((1 as i32)).wrapping_neg();
}
// C line 24454
18 => {
vm_block = if (js_parse_expr(s)) != 0 { 17 } else { 16 }; continue;
}
// C line 24453
19 => {
return ((1 as i32)).wrapping_neg();
}
// C line 24452
20 => {
vm_block = if (next_token(s)) != 0 { 19 } else { 18 }; continue;
}
// C line 24451
21 => {
vm_block = 7; continue;
}
// C line 24450
22 => {
vm_block = if ((((((((*(s)).token).u).str).sep) == ((96 as i32))) as i32)) != 0 { 21 } else { 20 }; continue;
}
// C line 24422
23 => {
return ((1 as i32)).wrapping_neg();
}
// C line 24419
24 => {
vm_block = if ((((JS_DefinePropertyValueUint32(ctx, template_object, ((depth) as u32), (((cooked).u).str).str, ((((1 as i32)).wrapping_shl(((2 as i32)) as u32)) | (((1 as i32)).wrapping_shl(((14 as i32)) as u32))))) < ((0 as i32))) as i32)) != 0 { 23 } else { 22 }; continue;
}
// C line 24417
25 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; (((cooked).u).str).str = assigned; assigned };
vm_block = 24; continue;
}
// C line 24416
26 => {
vm_block = if (js_parse_string(s, (96 as i32), (0 as i32), p, core::ptr::addr_of_mut!(cooked), core::ptr::addr_of_mut!(p))) != 0 { 25 } else { 24 }; continue;
}
// C line 24411
27 => {
return ((1 as i32)).wrapping_neg();
}
// C line 24408
28 => {
vm_block = if ((((JS_DefinePropertyValueUint32(ctx, raw_array, ((depth) as u32), JS_DupValue(ctx, ((((*(s)).token).u).str).str), ((((1 as i32)).wrapping_shl(((2 as i32)) as u32)) | (((1 as i32)).wrapping_shl(((14 as i32)) as u32))))) < ((0 as i32))) as i32)) != 0 { 27 } else { 26 }; continue;
}
// C line 24445
29 => {
let _ = { let old = depth; depth = (depth).wrapping_add(1); old };
vm_block = 22; continue;
}
// C line 24443
30 => {
let _ = emit_atom(s, (((crate::quickjs_atom::JS_ATOM_concat as i32)) as JSAtom));
vm_block = 29; continue;
}
// C line 24442
31 => {
let _ = emit_op(s, (((OP_get_field2 as i32)) as u8));
vm_block = 30; continue;
}
// C line 24441
32 => {
vm_block = 1; continue;
}
// C line 24440
33 => {
vm_block = if ((((((((*(s)).token).u).str).sep) == ((96 as i32))) as i32)) != 0 { 32 } else { 31 }; continue;
}
// C line 24439
34 => {
vm_block = if ((((depth) == ((0 as i32))) as i32)) != 0 { 33 } else { 29 }; continue;
}
// C line 24438
35 => {
return ((1 as i32)).wrapping_neg();
}
// C line 24437
36 => {
vm_block = if (ret) != 0 { 35 } else { 34 }; continue;
}
// C line 24436
37 => {
let _ = JS_FreeValue((*(s)).ctx, (((cooked).u).str).str);
vm_block = 36; continue;
}
// C line 24435
38 => {
let _ = { let assigned = emit_push_const(s, (((cooked).u).str).str, (1 as i32)); ret = assigned; assigned };
vm_block = 37; continue;
}
// C line 24447
39 => {
let _ = JS_FreeValue((*(s)).ctx, (((cooked).u).str).str);
vm_block = 22; continue;
}
// C line 24434
40 => {
vm_block = if (((((((((((*(str)).len()) as i32)) != ((0 as i32))) as i32)) != 0) || (((((depth) == ((0 as i32))) as i32)) != 0)) as i32)) != 0 { 38 } else { 39 }; continue;
}
// C line 24433
41 => {
let _ = { let assigned = (((((((cooked).u).str).str).u).ptr) as *mut JSString); str = assigned; assigned };
vm_block = 40; continue;
}
// C line 24432
42 => {
return ((1 as i32)).wrapping_neg();
}
// C line 24431
43 => {
vm_block = if (js_parse_string(s, (96 as i32), (1 as i32), p, core::ptr::addr_of_mut!(cooked), core::ptr::addr_of_mut!(p))) != 0 { 42 } else { 41 }; continue;
}
// C line 24430
44 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; ((((*(s)).token).u).str).str = assigned; assigned };
vm_block = 43; continue;
}
// C line 24429
45 => {
let _ = JS_FreeValue(ctx, ((((*(s)).token).u).str).str);
vm_block = 44; continue;
}
// C line 24407
46 => {
vm_block = if (call) != 0 { 28 } else { 45 }; continue;
}
// C line 24406
47 => {
let _ = { let assigned = (*(s)).token; cooked = assigned; assigned };
vm_block = 46; continue;
}
// C line 24405
48 => {
p = (((*(s)).token).ptr).offset((((1 as i32)) as isize));
vm_block = 47; continue;
}
// C line 24403
49 => {
let _ = { let assigned = (0 as i32); depth = assigned; assigned };
vm_block = 9; continue;
}
// C line 24399
50 => {
return ((1 as i32)).wrapping_neg();
}
// C line 24397
51 => {
vm_block = if ((((JS_DefinePropertyValue(ctx, template_object, (((crate::quickjs_atom::JS_ATOM_raw as i32)) as JSAtom), raw_array, ((1 as i32)).wrapping_shl(((14 as i32)) as u32))) < ((0 as i32))) as i32)) != 0 { 50 } else { 49 }; continue;
}
// C line 24396
52 => {
return ((1 as i32)).wrapping_neg();
}
// C line 24395
53 => {
vm_block = if (JS_IsException(raw_array)) != 0 { 52 } else { 51 }; continue;
}
// C line 24394
54 => {
let _ = { let assigned = JS_NewArray(ctx); raw_array = assigned; assigned };
vm_block = 53; continue;
}
// C line 24393
55 => {
return ((1 as i32)).wrapping_neg();
}
// C line 24392
56 => {
vm_block = if (ret) != 0 { 55 } else { 54 }; continue;
}
// C line 24391
57 => {
let _ = JS_FreeValue(ctx, template_object);
vm_block = 56; continue;
}
// C line 24390
58 => {
let _ = { let assigned = emit_push_const(s, template_object, (0 as i32)); ret = assigned; assigned };
vm_block = 57; continue;
}
// C line 24388
59 => {
return ((1 as i32)).wrapping_neg();
}
// C line 24387
60 => {
vm_block = if (JS_IsException(template_object)) != 0 { 59 } else { 58 }; continue;
}
// C line 24386
61 => {
let _ = { let assigned = JS_NewArray(ctx); template_object = assigned; assigned };
vm_block = 60; continue;
}
// C line 24383
62 => {
vm_block = if (call) != 0 { 61 } else { 49 }; continue;
}
// C line 24382
63 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; template_object = assigned; assigned };
vm_block = 62; continue;
}
// C line 24381
64 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; raw_array = assigned; assigned };
vm_block = 63; continue;
}
// C line 24376
65 => {
ctx = (*(s)).ctx;
vm_block = 64; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:24495. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn token_is_ident(mut tok: i32) -> i32 {
let mut vm_local_storage = Vec::<u64>::new();
let mut vm_block: usize = 1;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 24498
1 => {
return (((((((tok) == ((TOK_IDENT as i32))) as i32)) != 0) || (((((((((tok) >= ((TOK_NULL as i32))) as i32)) != 0) && (((((tok) <= ((TOK_AWAIT as i32))) as i32)) != 0)) as i32)) != 0)) as i32);
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:24504. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_parse_property_name(mut s: *mut JSParseState, mut pname: *mut JSAtom, mut allow_method: i32, mut allow_var: i32, mut allow_private: i32) -> i32 {
let mut vm_local_storage = Vec::<u64>::new();
let mut is_private: i32 = 0;
let mut is_non_reserved_ident: i32 = 0;
let mut name: JSAtom = 0;
let mut prop_type: i32 = 0;
let mut val: JSValue = core::mem::zeroed();
let mut vm_block: usize = 73;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 24620
1 => {
return ((1 as i32)).wrapping_neg();
}
// C line 24619 labels: fail
2 => {
let _ = { let assigned = (((0 as i32)) as JSAtom); *(pname) = assigned; assigned };
vm_block = 1; continue;
}
// C line 24617 labels: fail1
3 => {
let _ = JS_FreeAtom((*(s)).ctx, name);
vm_block = 2; continue;
}
// C line 24615
4 => {
return ((prop_type) | (is_private));
}
// C line 24614
5 => {
let _ = { let assigned = name; *(pname) = assigned; assigned };
vm_block = 4; continue;
}
// C line 24612
6 => {
vm_block = 2; continue;
}
// C line 24611 labels: invalid_prop
7 => {
let _ = js_parse_error(s, c"invalid property name".as_ptr());
vm_block = 6; continue;
}
// C line 24609
8 => {
let _ = JS_FreeAtom((*(s)).ctx, name);
vm_block = 7; continue;
}
// C line 24607
9 => {
vm_block = if ((((((((((((prop_type) != ((0 as i32))) as i32)) != 0) && (((((prop_type) != ((1 as i32))) as i32)) != 0)) as i32)) != 0) && (((((((*(s)).token).val) != ((40 as i32))) as i32)) != 0)) as i32)) != 0 { 8 } else { 5 }; continue;
}
// C line 24574
10 => {
let _ = { let assigned = (1 as i32); prop_type = assigned; assigned };
vm_block = 9; continue;
}
// C line 24572
11 => {
vm_block = if ((!(((((((((((*(s)).token).val) == ((58 as i32))) as i32)) != 0) || (((((((((((*(s)).token).val) == ((40 as i32))) as i32)) != 0) && ((allow_method) != 0)) as i32)) != 0)) as i32)) != 0) as i32)) != 0 { 10 } else { 9 }; continue;
}
// C line 24570 labels: ident_found
12 => {
vm_block = if (((((((((is_non_reserved_ident) != 0) && (((((prop_type) == ((0 as i32))) as i32)) != 0)) as i32)) != 0) && ((allow_var) != 0)) as i32)) != 0 { 11 } else { 9 }; continue;
}
// C line 24568
13 => {
vm_block = 3; continue;
}
// C line 24567
14 => {
vm_block = if (next_token(s)) != 0 { 13 } else { 12 }; continue;
}
// C line 24566
15 => {
let _ = { let assigned = JS_DupAtom((*(s)).ctx, ((((*(s)).token).u).ident).atom); name = assigned; assigned };
vm_block = 14; continue;
}
// C line 24563
16 => {
let _ = { let assigned = (((((((((*(s)).token).val) == ((TOK_IDENT as i32))) as i32)) != 0) && (((!((((((*(s)).token).u).ident).is_reserved) != 0) as i32)) != 0)) as i32); is_non_reserved_ident = assigned; assigned };
vm_block = 15; continue;
}
// C line 24582
17 => {
vm_block = 3; continue;
}
// C line 24581
18 => {
vm_block = if (next_token(s)) != 0 { 17 } else { 9 }; continue;
}
// C line 24580
19 => {
vm_block = 2; continue;
}
// C line 24579
20 => {
vm_block = if ((((name) == ((((0 as i32)) as JSAtom))) as i32)) != 0 { 19 } else { 18 }; continue;
}
// C line 24578
21 => {
let _ = { let assigned = JS_ValueToAtom((*(s)).ctx, ((((*(s)).token).u).str).str); name = assigned; assigned };
vm_block = 20; continue;
}
// C line 24590
22 => {
vm_block = 3; continue;
}
// C line 24589
23 => {
vm_block = if (next_token(s)) != 0 { 22 } else { 9 }; continue;
}
// C line 24588
24 => {
vm_block = 2; continue;
}
// C line 24587
25 => {
vm_block = if ((((name) == ((((0 as i32)) as JSAtom))) as i32)) != 0 { 24 } else { 23 }; continue;
}
// C line 24586
26 => {
let _ = { let assigned = JS_ValueToAtom((*(s)).ctx, val); name = assigned; assigned };
vm_block = 25; continue;
}
// C line 24585
27 => {
let _ = { let assigned = ((((*(s)).token).u).num).val; val = assigned; assigned };
vm_block = 26; continue;
}
// C line 24598
28 => {
let _ = { let assigned = (((0 as i32)) as JSAtom); name = assigned; assigned };
vm_block = 9; continue;
}
// C line 24597
29 => {
vm_block = 2; continue;
}
// C line 24596
30 => {
vm_block = if (js_parse_expect(s, (93 as i32))) != 0 { 29 } else { 28 }; continue;
}
// C line 24595
31 => {
vm_block = 2; continue;
}
// C line 24594
32 => {
vm_block = if (js_parse_assign_expr(s)) != 0 { 31 } else { 30 }; continue;
}
// C line 24593
33 => {
vm_block = 2; continue;
}
// C line 24592
34 => {
vm_block = if (next_token(s)) != 0 { 33 } else { 32 }; continue;
}
// C line 24603
35 => {
let _ = { let assigned = ((1 as i32)).wrapping_shl(((4 as i32)) as u32); is_private = assigned; assigned };
vm_block = 9; continue;
}
// C line 24602
36 => {
vm_block = 3; continue;
}
// C line 24601
37 => {
vm_block = if (next_token(s)) != 0 { 36 } else { 35 }; continue;
}
// C line 24600
38 => {
let _ = { let assigned = JS_DupAtom((*(s)).ctx, ((((*(s)).token).u).ident).atom); name = assigned; assigned };
vm_block = 37; continue;
}
// C line 24605
39 => {
vm_block = 7; continue;
}
// C line 24599
40 => {
vm_block = if ((((((((((*(s)).token).val) == ((TOK_PRIVATE_NAME as i32))) as i32)) != 0) && ((allow_private) != 0)) as i32)) != 0 { 38 } else { 39 }; continue;
}
// C line 24591
41 => {
vm_block = if ((((((*(s)).token).val) == ((91 as i32))) as i32)) != 0 { 34 } else { 40 }; continue;
}
// C line 24583
42 => {
vm_block = if ((((((*(s)).token).val) == ((TOK_NUMBER as i32))) as i32)) != 0 { 27 } else { 41 }; continue;
}
// C line 24577
43 => {
vm_block = if ((((((*(s)).token).val) == ((TOK_STRING as i32))) as i32)) != 0 { 21 } else { 42 }; continue;
}
// C line 24561
44 => {
vm_block = if (token_is_ident(((*(s)).token).val)) != 0 { 16 } else { 43 }; continue;
}
// C line 24534
45 => {
let _ = JS_FreeAtom((*(s)).ctx, name);
vm_block = 44; continue;
}
// C line 24533
46 => {
let _ = { let assigned = ((2 as i32)).wrapping_add((((name) == ((((crate::quickjs_atom::JS_ATOM_set as i32)) as JSAtom))) as i32)); prop_type = assigned; assigned };
vm_block = 45; continue;
}
// C line 24531
47 => {
vm_block = 12; continue;
}
// C line 24530
48 => {
let _ = { let assigned = (1 as i32); is_non_reserved_ident = assigned; assigned };
vm_block = 47; continue;
}
// C line 24526
49 => {
vm_block = if ((((((((((((((((((((((((((*(s)).token).val) == ((58 as i32))) as i32)) != 0) || (((((((*(s)).token).val) == ((44 as i32))) as i32)) != 0)) as i32)) != 0) || (((((((*(s)).token).val) == ((125 as i32))) as i32)) != 0)) as i32)) != 0) || (((((((*(s)).token).val) == ((40 as i32))) as i32)) != 0)) as i32)) != 0) || (((((((*(s)).token).val) == ((61 as i32))) as i32)) != 0)) as i32)) != 0) || (((((((((((*(s)).token).val) == ((59 as i32))) as i32)) != 0) && ((allow_private) != 0)) as i32)) != 0)) as i32)) != 0 { 48 } else { 46 }; continue;
}
// C line 24525
50 => {
vm_block = 3; continue;
}
// C line 24524
51 => {
vm_block = if (next_token(s)) != 0 { 50 } else { 49 }; continue;
}
// C line 24523
52 => {
let _ = { let assigned = JS_DupAtom((*(s)).ctx, ((((*(s)).token).u).ident).atom); name = assigned; assigned };
vm_block = 51; continue;
}
// C line 24538
53 => {
let _ = { let assigned = (4 as i32); prop_type = assigned; assigned };
vm_block = 44; continue;
}
// C line 24537
54 => {
vm_block = 2; continue;
}
// C line 24536
55 => {
vm_block = if (next_token(s)) != 0 { 54 } else { 53 }; continue;
}
// C line 24554
56 => {
let _ = { let assigned = (6 as i32); prop_type = assigned; assigned };
vm_block = 44; continue;
}
// C line 24553
57 => {
vm_block = 2; continue;
}
// C line 24552
58 => {
vm_block = if (next_token(s)) != 0 { 57 } else { 56 }; continue;
}
// C line 24556
59 => {
let _ = { let assigned = (5 as i32); prop_type = assigned; assigned };
vm_block = 44; continue;
}
// C line 24551
60 => {
vm_block = if ((((((*(s)).token).val) == ((42 as i32))) as i32)) != 0 { 58 } else { 59 }; continue;
}
// C line 24550
61 => {
let _ = JS_FreeAtom((*(s)).ctx, name);
vm_block = 60; continue;
}
// C line 24548
62 => {
vm_block = 12; continue;
}
// C line 24547
63 => {
let _ = { let assigned = (1 as i32); is_non_reserved_ident = assigned; assigned };
vm_block = 62; continue;
}
// C line 24544
64 => {
vm_block = if ((((((((((((((((((((((*(s)).token).val) == ((58 as i32))) as i32)) != 0) || (((((((*(s)).token).val) == ((44 as i32))) as i32)) != 0)) as i32)) != 0) || (((((((*(s)).token).val) == ((125 as i32))) as i32)) != 0)) as i32)) != 0) || (((((((*(s)).token).val) == ((40 as i32))) as i32)) != 0)) as i32)) != 0) || (((((((*(s)).token).val) == ((61 as i32))) as i32)) != 0)) as i32)) != 0 { 63 } else { 61 }; continue;
}
// C line 24543
65 => {
vm_block = 3; continue;
}
// C line 24542
66 => {
vm_block = if (next_token(s)) != 0 { 65 } else { 64 }; continue;
}
// C line 24541
67 => {
let _ = { let assigned = JS_DupAtom((*(s)).ctx, ((((*(s)).token).u).ident).atom); name = assigned; assigned };
vm_block = 66; continue;
}
// C line 24539
68 => {
vm_block = if (((((token_is_pseudo_keyword(s, (((crate::quickjs_atom::JS_ATOM_async as i32)) as JSAtom))) != 0) && (((((peek_token(s, (1 as i32))) != ((10 as i32))) as i32)) != 0)) as i32)) != 0 { 67 } else { 44 }; continue;
}
// C line 24535
69 => {
vm_block = if ((((((*(s)).token).val) == ((42 as i32))) as i32)) != 0 { 55 } else { 68 }; continue;
}
// C line 24519
70 => {
vm_block = if (((((((((token_is_pseudo_keyword(s, (((crate::quickjs_atom::JS_ATOM_get as i32)) as JSAtom))) != 0) || ((token_is_pseudo_keyword(s, (((crate::quickjs_atom::JS_ATOM_set as i32)) as JSAtom))) != 0)) as i32)) != 0) && (((((((!((allow_private) != 0) as i32)) != 0) || (((((peek_token(s, (1 as i32))) != ((10 as i32))) as i32)) != 0)) as i32)) != 0)) as i32)) != 0 { 52 } else { 69 }; continue;
}
// C line 24515
71 => {
vm_block = if (allow_method) != 0 { 70 } else { 44 }; continue;
}
// C line 24514
72 => {
let _ = { let assigned = (0 as i32); prop_type = assigned; assigned };
vm_block = 71; continue;
}
// C line 24509
73 => {
is_private = (0 as i32);
vm_block = 72; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:24850. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_parse_object_literal(mut s: *mut JSParseState) -> i32 {
let mut vm_local_storage = Vec::<u64>::new();
let mut name: JSAtom = 0;
let mut start_ptr: *const u8 = core::ptr::null();
let mut prop_type: i32 = 0;
let mut has_proto: i32 = 0;
let mut is_getset: i32 = 0;
let mut func_type: JSParseFunctionEnum = core::mem::zeroed();
let mut func_kind: JSFunctionKindEnum = core::mem::zeroed();
let mut op_flags: i32 = 0;
let mut vm_block: usize = 79;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 24964
1 => {
return ((1 as i32)).wrapping_neg();
}
// C line 24963 labels: fail
2 => {
let _ = JS_FreeAtom((*(s)).ctx, name);
vm_block = 1; continue;
}
// C line 24961
3 => {
return (0 as i32);
}
// C line 24960
4 => {
vm_block = 2; continue;
}
// C line 24959
5 => {
vm_block = if (js_parse_expect(s, (125 as i32))) != 0 { 4 } else { 3 }; continue;
}
// C line 24862
6 => {
vm_block = if ((((((*(s)).token).val) != ((125 as i32))) as i32)) != 0 { 74 } else { 5 }; continue;
}
// C line 24957
7 => {
vm_block = 2; continue;
}
// C line 24956
8 => {
vm_block = if (next_token(s)) != 0 { 7 } else { 6 }; continue;
}
// C line 24955
9 => {
vm_block = 5; continue;
}
// C line 24954
10 => {
vm_block = if ((((((*(s)).token).val) != ((44 as i32))) as i32)) != 0 { 9 } else { 8 }; continue;
}
// C line 24953 labels: next
11 => {
let _ = { let assigned = (((0 as i32)) as JSAtom); name = assigned; assigned };
vm_block = 10; continue;
}
// C line 24951
12 => {
let _ = JS_FreeAtom((*(s)).ctx, name);
vm_block = 11; continue;
}
// C line 24889
13 => {
let _ = emit_atom(s, name);
vm_block = 12; continue;
}
// C line 24888
14 => {
let _ = emit_op(s, (((OP_define_field as i32)) as u8));
vm_block = 13; continue;
}
// C line 24887
15 => {
let _ = emit_u16(s, (((*((*(s)).cur_func)).scope_level) as u16));
vm_block = 14; continue;
}
// C line 24886
16 => {
let _ = emit_atom(s, name);
vm_block = 15; continue;
}
// C line 24885
17 => {
let _ = emit_op(s, (((OP_scope_get_var as i32)) as u8));
vm_block = 16; continue;
}
// C line 24924
18 => {
let _ = emit_u8(s, ((((op_flags) | ((4 as i32)))) as u8));
vm_block = 12; continue;
}
// C line 24919
19 => {
let _ = { let assigned = (((1 as i32)).wrapping_add(prop_type)).wrapping_sub((2 as i32)); op_flags = assigned; assigned };
vm_block = 18; continue;
}
// C line 24922
20 => {
let _ = { let assigned = (0 as i32); op_flags = assigned; assigned };
vm_block = 18; continue;
}
// C line 24918
21 => {
vm_block = if (is_getset) != 0 { 19 } else { 20 }; continue;
}
// C line 24913
22 => {
let _ = emit_op(s, (((OP_define_method_computed as i32)) as u8));
vm_block = 21; continue;
}
// C line 24916
23 => {
let _ = emit_atom(s, name);
vm_block = 21; continue;
}
// C line 24915
24 => {
let _ = emit_op(s, (((OP_define_method as i32)) as u8));
vm_block = 23; continue;
}
// C line 24912
25 => {
vm_block = if ((((name) == ((((0 as i32)) as JSAtom))) as i32)) != 0 { 22 } else { 24 }; continue;
}
// C line 24911
26 => {
vm_block = 2; continue;
}
// C line 24909
27 => {
vm_block = if (js_parse_function_decl(s, func_type, func_kind, (((0 as i32)) as JSAtom), start_ptr)) != 0 { 26 } else { 25 }; continue;
}
// C line 24899
28 => {
let _ = { let assigned = (((((JS_PARSE_FUNC_GETTER as i32)).wrapping_add(prop_type)).wrapping_sub((2 as i32))) as JSParseFunctionEnum); func_type = assigned; assigned };
vm_block = 27; continue;
}
// C line 24903
29 => {
let _ = { let assigned = (((JS_FUNC_GENERATOR as i32)) as JSFunctionKindEnum); func_kind = assigned; assigned };
vm_block = 27; continue;
}
// C line 24905
30 => {
let _ = { let assigned = (((JS_FUNC_ASYNC as i32)) as JSFunctionKindEnum); func_kind = assigned; assigned };
vm_block = 27; continue;
}
// C line 24907
31 => {
let _ = { let assigned = (((JS_FUNC_ASYNC_GENERATOR as i32)) as JSFunctionKindEnum); func_kind = assigned; assigned };
vm_block = 27; continue;
}
// C line 24906
32 => {
vm_block = if ((((prop_type) == ((6 as i32))) as i32)) != 0 { 31 } else { 27 }; continue;
}
// C line 24904
33 => {
vm_block = if ((((prop_type) == ((5 as i32))) as i32)) != 0 { 30 } else { 32 }; continue;
}
// C line 24902
34 => {
vm_block = if ((((prop_type) == ((4 as i32))) as i32)) != 0 { 29 } else { 33 }; continue;
}
// C line 24901
35 => {
let _ = { let assigned = (((JS_PARSE_FUNC_METHOD as i32)) as JSParseFunctionEnum); func_type = assigned; assigned };
vm_block = 34; continue;
}
// C line 24898
36 => {
vm_block = if (is_getset) != 0 { 28 } else { 35 }; continue;
}
// C line 24897
37 => {
let _ = { let assigned = (((JS_FUNC_NORMAL as i32)) as JSFunctionKindEnum); func_kind = assigned; assigned };
vm_block = 36; continue;
}
// C line 24891
38 => {
is_getset = (((((((prop_type) == ((2 as i32))) as i32)) != 0) || (((((prop_type) == ((3 as i32))) as i32)) != 0)) as i32);
vm_block = 37; continue;
}
// C line 24937
39 => {
let _ = emit_op(s, (((OP_drop as i32)) as u8));
vm_block = 12; continue;
}
// C line 24936
40 => {
let _ = emit_op(s, (((OP_define_array_el as i32)) as u8));
vm_block = 39; continue;
}
// C line 24935
41 => {
let _ = set_object_name_computed(s);
vm_block = 40; continue;
}
// C line 24944
42 => {
let _ = { let assigned = (1 as i32); has_proto = assigned; assigned };
vm_block = 12; continue;
}
// C line 24943
43 => {
let _ = emit_op(s, (((OP_set_proto as i32)) as u8));
vm_block = 42; continue;
}
// C line 24941
44 => {
vm_block = 2; continue;
}
// C line 24940
45 => {
let _ = js_parse_error(s, c"duplicate __proto__ property name".as_ptr());
vm_block = 44; continue;
}
// C line 24939
46 => {
vm_block = if (has_proto) != 0 { 45 } else { 43 }; continue;
}
// C line 24948
47 => {
let _ = emit_atom(s, name);
vm_block = 12; continue;
}
// C line 24947
48 => {
let _ = emit_op(s, (((OP_define_field as i32)) as u8));
vm_block = 47; continue;
}
// C line 24946
49 => {
let _ = set_object_name(s, name);
vm_block = 48; continue;
}
// C line 24938
50 => {
vm_block = if ((((name) == ((((crate::quickjs_atom::JS_ATOM___proto__ as i32)) as JSAtom))) as i32)) != 0 { 46 } else { 49 }; continue;
}
// C line 24934
51 => {
vm_block = if ((((name) == ((((0 as i32)) as JSAtom))) as i32)) != 0 { 41 } else { 50 }; continue;
}
// C line 24933
52 => {
vm_block = 2; continue;
}
// C line 24932
53 => {
vm_block = if (js_parse_assign_expr(s)) != 0 { 52 } else { 51 }; continue;
}
// C line 24931
54 => {
vm_block = 2; continue;
}
// C line 24930
55 => {
vm_block = if (js_parse_expect(s, (58 as i32))) != 0 { 54 } else { 53 }; continue;
}
// C line 24928
56 => {
let _ = emit_op(s, (((OP_to_propkey as i32)) as u8));
vm_block = 55; continue;
}
// C line 24926
57 => {
vm_block = if ((((name) == ((((0 as i32)) as JSAtom))) as i32)) != 0 { 56 } else { 55 }; continue;
}
// C line 24890
58 => {
vm_block = if ((((((*(s)).token).val) == ((40 as i32))) as i32)) != 0 { 38 } else { 57 }; continue;
}
// C line 24883
59 => {
vm_block = if ((((prop_type) == ((1 as i32))) as i32)) != 0 { 17 } else { 58 }; continue;
}
// C line 24881
60 => {
vm_block = 2; continue;
}
// C line 24880
61 => {
vm_block = if ((((prop_type) < ((0 as i32))) as i32)) != 0 { 60 } else { 59 }; continue;
}
// C line 24879
62 => {
let _ = { let assigned = js_parse_property_name(s, core::ptr::addr_of_mut!(name), (1 as i32), (1 as i32), (0 as i32)); prop_type = assigned; assigned };
vm_block = 61; continue;
}
// C line 24876
63 => {
vm_block = 11; continue;
}
// C line 24875
64 => {
let _ = emit_op(s, (((OP_drop as i32)) as u8));
vm_block = 63; continue;
}
// C line 24874
65 => {
let _ = emit_op(s, (((OP_drop as i32)) as u8));
vm_block = 64; continue;
}
// C line 24873
66 => {
let _ = emit_u8(s, (((((((2 as i32)) | (((1 as i32)).wrapping_shl(((2 as i32)) as u32)))) | (((0 as i32)).wrapping_shl(((5 as i32)) as u32)))) as u8));
vm_block = 65; continue;
}
// C line 24872
67 => {
let _ = emit_op(s, (((OP_copy_data_properties as i32)) as u8));
vm_block = 66; continue;
}
// C line 24871
68 => {
let _ = emit_op(s, (((OP_null as i32)) as u8));
vm_block = 67; continue;
}
// C line 24870
69 => {
return ((1 as i32)).wrapping_neg();
}
// C line 24869
70 => {
vm_block = if (js_parse_assign_expr(s)) != 0 { 69 } else { 68 }; continue;
}
// C line 24868
71 => {
return ((1 as i32)).wrapping_neg();
}
// C line 24867
72 => {
vm_block = if (next_token(s)) != 0 { 71 } else { 70 }; continue;
}
// C line 24866
73 => {
vm_block = if ((((((*(s)).token).val) == ((TOK_ELLIPSIS as i32))) as i32)) != 0 { 72 } else { 62 }; continue;
}
// C line 24864
74 => {
let _ = { let assigned = ((*(s)).token).ptr; start_ptr = assigned; assigned };
vm_block = 73; continue;
}
// C line 24861
75 => {
let _ = { let assigned = (0 as i32); has_proto = assigned; assigned };
vm_block = 6; continue;
}
// C line 24860
76 => {
let _ = emit_op(s, (((OP_object as i32)) as u8));
vm_block = 75; continue;
}
// C line 24858
77 => {
vm_block = 2; continue;
}
// C line 24857
78 => {
vm_block = if (next_token(s)) != 0 { 77 } else { 76 }; continue;
}
// C line 24852
79 => {
name = (((0 as i32)) as JSAtom);
vm_block = 78; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:24987. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_parse_left_hand_side_expr(mut s: *mut JSParseState) -> i32 {
let mut vm_local_storage = Vec::<u64>::new();
let mut vm_block: usize = 1;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 24989
1 => {
return js_parse_postfix_expr(s, ((1 as i32)).wrapping_shl(((1 as i32)) as u32));
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:24992. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_parse_class_default_ctor(mut s: *mut JSParseState, mut has_super: i32, mut pfd: *mut *mut JSFunctionDef) -> i32 {
let mut vm_local_storage = Vec::<u64>::new();
let mut func_type: JSParseFunctionEnum = core::mem::zeroed();
let mut fd: *mut JSFunctionDef = core::ptr::null_mut();
let mut idx: i32 = 0;
let mut vm_block: usize = 35;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 25045
1 => {
return (0 as i32);
}
// C line 25043
2 => {
let _ = { let assigned = idx; (*(fd)).parent_cpool_idx = assigned; assigned };
vm_block = 1; continue;
}
// C line 25042
3 => {
let _ = { let assigned = cpool_add(s, JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_NULL as i32)) as i64) }); idx = assigned; assigned };
vm_block = 2; continue;
}
// C line 25039
4 => {
let _ = { let assigned = fd; *(pfd) = assigned; assigned };
vm_block = 3; continue;
}
// C line 25038
5 => {
vm_block = if !(pfd).is_null() { 4 } else { 3 }; continue;
}
// C line 25037
6 => {
let _ = { let assigned = (*(fd)).parent; (*(s)).cur_func = assigned; assigned };
vm_block = 5; continue;
}
// C line 25035
7 => {
let _ = emit_return(s, (0 as i32));
vm_block = 6; continue;
}
// C line 25034
8 => {
let _ = { let assigned = func_type; (*(fd)).func_type = (assigned) as u8; assigned };
vm_block = 7; continue;
}
// C line 25033
9 => {
let _ = { let assigned = (((JS_FUNC_NORMAL as i32)) as JSFunctionKindEnum); (*(fd)).func_kind = (assigned) as u8; assigned };
vm_block = 8; continue;
}
// C line 25025
10 => {
let _ = emit_class_field_init(s);
vm_block = 9; continue;
}
// C line 25024
11 => {
let _ = emit_u16(s, (((0 as i32)) as u16));
vm_block = 10; continue;
}
// C line 25023
12 => {
let _ = emit_atom(s, (((crate::quickjs_atom::JS_ATOM_this as i32)) as JSAtom));
vm_block = 11; continue;
}
// C line 25022
13 => {
let _ = emit_op(s, (((OP_scope_put_var_init as i32)) as u8));
vm_block = 12; continue;
}
// C line 25020
14 => {
let _ = emit_op(s, (((OP_init_ctor as i32)) as u8));
vm_block = 13; continue;
}
// C line 25019
15 => {
let _ = { let assigned = (((JS_PARSE_FUNC_DERIVED_CLASS_CONSTRUCTOR as i32)) as JSParseFunctionEnum); func_type = assigned; assigned };
vm_block = 14; continue;
}
// C line 25018
16 => {
let _ = { let assigned = (1 as i32); (*(fd)).has_arguments_binding = assigned; assigned };
vm_block = 15; continue;
}
// C line 25017
17 => {
let _ = { let assigned = (1 as i32); (*(fd)).arguments_allowed = assigned; assigned };
vm_block = 16; continue;
}
// C line 25016
18 => {
let _ = { let assigned = (1 as i32); (*(fd)).super_call_allowed = assigned; assigned };
vm_block = 17; continue;
}
// C line 25015
19 => {
let _ = { let assigned = (1 as i32); (*(fd)).is_derived_class_constructor = assigned; assigned };
vm_block = 18; continue;
}
// C line 25030
20 => {
let _ = emit_class_field_init(s);
vm_block = 9; continue;
}
// C line 25029
21 => {
let _ = emit_op(s, (((OP_check_ctor as i32)) as u8));
vm_block = 20; continue;
}
// C line 25027
22 => {
let _ = { let assigned = (((JS_PARSE_FUNC_CLASS_CONSTRUCTOR as i32)) as JSParseFunctionEnum); func_type = assigned; assigned };
vm_block = 21; continue;
}
// C line 25014
23 => {
vm_block = if (has_super) != 0 { 19 } else { 22 }; continue;
}
// C line 25013
24 => {
let _ = { let assigned = (*(fd)).scope_level; (*(fd)).body_scope = assigned; assigned };
vm_block = 23; continue;
}
// C line 25012
25 => {
let _ = push_scope(s);
vm_block = 24; continue;
}
// C line 25010
26 => {
let _ = { let assigned = (1 as i32); (*(fd)).new_target_allowed = assigned; assigned };
vm_block = 25; continue;
}
// C line 25009
27 => {
let _ = { let assigned = (1 as i32); (*(fd)).has_this_binding = assigned; assigned };
vm_block = 26; continue;
}
// C line 25008
28 => {
let _ = { let assigned = (0 as i32); (*(fd)).has_prototype = assigned; assigned };
vm_block = 27; continue;
}
// C line 25007
29 => {
let _ = { let assigned = (1 as i32); (*(fd)).super_allowed = assigned; assigned };
vm_block = 28; continue;
}
// C line 25006
30 => {
let _ = { let assigned = (1 as i32); (*(fd)).has_home_object = assigned; assigned };
vm_block = 29; continue;
}
// C line 25005
31 => {
let _ = { let assigned = fd; (*(s)).cur_func = assigned; assigned };
vm_block = 30; continue;
}
// C line 25003
32 => {
return ((1 as i32)).wrapping_neg();
}
// C line 25002
33 => {
vm_block = if ((!(!(fd).is_null()) as i32)) != 0 { 32 } else { 31 }; continue;
}
// C line 25000
34 => {
let _ = { let assigned = js_new_function_def((*(s)).ctx, fd, (0 as i32), (0 as i32), (*(s)).filename, ((*(s)).token).ptr, core::ptr::addr_of_mut!((*(s)).get_line_col_cache)); fd = assigned; assigned };
vm_block = 33; continue;
}
// C line 24997
35 => {
fd = (*(s)).cur_func;
vm_block = 34; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:25049. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn find_private_class_field(mut ctx: *mut JSContext, mut fd: *mut JSFunctionDef, mut name: JSAtom, mut scope_level: i32) -> i32 {
let mut vm_local_storage = Vec::<u64>::new();
let mut idx: i32 = 0;
let mut vm_block: usize = 8;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 25061
1 => {
return ((1 as i32)).wrapping_neg();
}
// C line 25054
2 => {
vm_block = if ((((idx) != (((1 as i32)).wrapping_neg())) as i32)) != 0 { 7 } else { 1 }; continue;
}
// C line 25059
3 => {
let _ = { let assigned = (*((*(fd)).vars).offset((idx) as isize)).scope_next; idx = assigned; assigned };
vm_block = 2; continue;
}
// C line 25058
4 => {
return idx;
}
// C line 25057
5 => {
vm_block = if (((((*((*(fd)).vars).offset((idx) as isize)).var_name) == (name)) as i32)) != 0 { 4 } else { 3 }; continue;
}
// C line 25056
6 => {
vm_block = 1; continue;
}
// C line 25055
7 => {
vm_block = if (((((*((*(fd)).vars).offset((idx) as isize)).scope_level) != (scope_level)) as i32)) != 0 { 6 } else { 5 }; continue;
}
// C line 25053
8 => {
let _ = { let assigned = (*((*(fd)).scopes).offset((scope_level) as isize)).first; idx = assigned; assigned };
vm_block = 2; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:25067. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn emit_class_field_init(mut s: *mut JSParseState) -> () {
let mut vm_local_storage = Vec::<u64>::new();
let mut label_next: i32 = 0;
let mut vm_block: usize = 13;
loop { match vm_block {
// C line ?
0 => {
return ();
}
// C line 25089
1 => {
let _ = emit_op(s, (((OP_drop as i32)) as u8));
vm_block = 0; continue;
}
// C line 25088
2 => {
let _ = emit_label(s, label_next);
vm_block = 1; continue;
}
// C line 25086
3 => {
let _ = emit_u16(s, (((0 as i32)) as u16));
vm_block = 2; continue;
}
// C line 25085
4 => {
let _ = emit_op(s, (((OP_call_method as i32)) as u8));
vm_block = 3; continue;
}
// C line 25083
5 => {
let _ = emit_op(s, (((OP_swap as i32)) as u8));
vm_block = 4; continue;
}
// C line 25081
6 => {
let _ = emit_u16(s, (((0 as i32)) as u16));
vm_block = 5; continue;
}
// C line 25080
7 => {
let _ = emit_atom(s, (((crate::quickjs_atom::JS_ATOM_this as i32)) as JSAtom));
vm_block = 6; continue;
}
// C line 25079
8 => {
let _ = emit_op(s, (((OP_scope_get_var as i32)) as u8));
vm_block = 7; continue;
}
// C line 25077
9 => {
let _ = { let assigned = emit_goto(s, (OP_if_false as i32), ((1 as i32)).wrapping_neg()); label_next = assigned; assigned };
vm_block = 8; continue;
}
// C line 25076
10 => {
let _ = emit_op(s, (((OP_dup as i32)) as u8));
vm_block = 9; continue;
}
// C line 25073
11 => {
let _ = emit_u16(s, (((*((*(s)).cur_func)).scope_level) as u16));
vm_block = 10; continue;
}
// C line 25072
12 => {
let _ = emit_atom(s, (((crate::quickjs_atom::JS_ATOM_class_fields_init as i32)) as JSAtom));
vm_block = 11; continue;
}
// C line 25071
13 => {
let _ = emit_op(s, (((OP_scope_get_var as i32)) as u8));
vm_block = 12; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:25093. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn get_private_setter_name(mut ctx: *mut JSContext, mut name: JSAtom) -> JSAtom {
let mut vm_local_storage = Vec::<u64>::new();
let mut vm_block: usize = 1;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 25095
1 => {
return js_atom_concat_str(ctx, name, c"<set>".as_ptr());
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:25106. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn emit_class_init_start(mut s: *mut JSParseState, mut cf: *mut ClassFieldsDef) -> i32 {
let mut vm_local_storage = Vec::<u64>::new();
let mut label_add_brand: i32 = 0;
let mut vm_block: usize = 18;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 25138
1 => {
return (0 as i32);
}
// C line 25137
2 => {
let _ = { let assigned = (*((*(s)).cur_func)).parent; (*(s)).cur_func = assigned; assigned };
vm_block = 1; continue;
}
// C line 25135
3 => {
let _ = emit_label(s, label_add_brand);
vm_block = 2; continue;
}
// C line 25133
4 => {
let _ = emit_op(s, (((OP_add_brand as i32)) as u8));
vm_block = 3; continue;
}
// C line 25131
5 => {
let _ = emit_u16(s, (((0 as i32)) as u16));
vm_block = 4; continue;
}
// C line 25130
6 => {
let _ = emit_atom(s, (((crate::quickjs_atom::JS_ATOM_home_object as i32)) as JSAtom));
vm_block = 5; continue;
}
// C line 25129
7 => {
let _ = emit_op(s, (((OP_scope_get_var as i32)) as u8));
vm_block = 6; continue;
}
// C line 25127
8 => {
let _ = emit_u16(s, (((0 as i32)) as u16));
vm_block = 7; continue;
}
// C line 25126
9 => {
let _ = emit_atom(s, (((crate::quickjs_atom::JS_ATOM_this as i32)) as JSAtom));
vm_block = 8; continue;
}
// C line 25125
10 => {
let _ = emit_op(s, (((OP_scope_get_var as i32)) as u8));
vm_block = 9; continue;
}
// C line 25123
11 => {
let _ = { let assigned = emit_goto(s, (OP_if_false as i32), ((1 as i32)).wrapping_neg()); label_add_brand = assigned; assigned };
vm_block = 10; continue;
}
// C line 25122
12 => {
let _ = { let assigned = (*((*(cf)).fields_init_fd)).last_opcode_pos; (*(cf)).brand_push_pos = assigned; assigned };
vm_block = 11; continue;
}
// C line 25121
13 => {
let _ = emit_op(s, (((OP_push_false as i32)) as u8));
vm_block = 12; continue;
}
// C line 25117
14 => {
vm_block = if ((!(((*(cf)).is_static) != 0) as i32)) != 0 { 13 } else { 2 }; continue;
}
// C line 25115
15 => {
let _ = { let assigned = (*(cf)).fields_init_fd; (*(s)).cur_func = assigned; assigned };
vm_block = 14; continue;
}
// C line 25113
16 => {
return ((1 as i32)).wrapping_neg();
}
// C line 25112
17 => {
vm_block = if ((!(!((*(cf)).fields_init_fd).is_null()) as i32)) != 0 { 16 } else { 15 }; continue;
}
// C line 25111
18 => {
let _ = { let assigned = js_parse_function_class_fields_init(s); (*(cf)).fields_init_fd = assigned; assigned };
vm_block = 17; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:25141. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn emit_class_init_end(mut s: *mut JSParseState, mut cf: *mut ClassFieldsDef) -> () {
let mut vm_local_storage = Vec::<u64>::new();
let mut cpool_idx: i32 = 0;
let mut vm_block: usize = 8;
loop { match vm_block {
// C line ?
0 => {
return ();
}
// C line 25153
1 => {
let _ = emit_op(s, (((OP_set_home_object as i32)) as u8));
vm_block = 0; continue;
}
// C line 25152
2 => {
let _ = emit_u32(s, ((cpool_idx) as u32));
vm_block = 1; continue;
}
// C line 25151
3 => {
let _ = emit_op(s, (((OP_fclosure as i32)) as u8));
vm_block = 2; continue;
}
// C line 25150
4 => {
let _ = { let assigned = cpool_idx; (*((*(cf)).fields_init_fd)).parent_cpool_idx = assigned; assigned };
vm_block = 3; continue;
}
// C line 25149
5 => {
let _ = { let assigned = cpool_add(s, JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_NULL as i32)) as i64) }); cpool_idx = assigned; assigned };
vm_block = 4; continue;
}
// C line 25147
6 => {
let _ = { let assigned = (*((*(s)).cur_func)).parent; (*(s)).cur_func = assigned; assigned };
vm_block = 5; continue;
}
// C line 25146
7 => {
let _ = emit_op(s, (((OP_return_undef as i32)) as u8));
vm_block = 6; continue;
}
// C line 25145
8 => {
let _ = { let assigned = (*(cf)).fields_init_fd; (*(s)).cur_func = assigned; assigned };
vm_block = 7; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:25157. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_parse_class(mut s: *mut JSParseState, mut is_class_expr: i32, mut export_flag: JSParseExportEnum) -> i32 {
let mut vm_local_storage = Vec::<u64>::new();
let mut ctx: *mut JSContext = core::ptr::null_mut();
let mut fd: *mut JSFunctionDef = core::ptr::null_mut();
let mut name: JSAtom = 0;
let mut class_name: JSAtom = 0;
let mut class_name1: JSAtom = 0;
let mut class_var_name: JSAtom = 0;
let mut method_fd: *mut JSFunctionDef = core::ptr::null_mut();
let mut ctor_fd: *mut JSFunctionDef = core::ptr::null_mut();
let mut saved_js_mode: i32 = 0;
let mut class_name_var_idx: i32 = 0;
let mut prop_type: i32 = 0;
let mut ctor_cpool_offset: i32 = 0;
let mut class_flags: i32 = 0;
let mut i: i32 = 0;
let mut define_class_offset: i32 = 0;
let mut is_static: i32 = 0;
let mut is_private: i32 = 0;
let mut class_start_ptr: *const u8 = core::ptr::null();
let mut start_ptr: *const u8 = core::ptr::null();
let mut class_fields: [ClassFieldsDef; 2] = core::mem::zeroed();
let mut cf: *mut ClassFieldsDef = core::ptr::null_mut();
let mut next: i32 = 0;
let mut cf_1: *mut ClassFieldsDef = core::ptr::null_mut();
let mut init: *mut JSFunctionDef = core::ptr::null_mut();
let mut is_set: i32 = 0;
let mut method_fd_1: *mut JSFunctionDef = core::ptr::null_mut();
let mut idx: i32 = 0;
let mut var_kind: i32 = 0;
let mut is_static1: i32 = 0;
let mut setter_name: JSAtom = 0;
let mut ret: i32 = 0;
let mut cf_2: *mut ClassFieldsDef = core::ptr::null_mut();
let mut field_var_name: JSAtom = 0;
let mut func_type: JSParseFunctionEnum = core::mem::zeroed();
let mut func_kind: JSFunctionKindEnum = core::mem::zeroed();
let mut cf_3: *mut ClassFieldsDef = core::ptr::null_mut();
let mut var_idx: i32 = 0;
let mut cf_4: *mut ClassFieldsDef = core::ptr::null_mut();
let mut vm_block: usize = 325;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 25666
1 => {
return ((1 as i32)).wrapping_neg();
}
// C line 25665
2 => {
let _ = { let assigned = ((saved_js_mode) as u8); (*(fd)).js_mode = (assigned) as u8; assigned };
vm_block = 1; continue;
}
// C line 25664
3 => {
let _ = JS_FreeAtom(ctx, class_var_name);
vm_block = 2; continue;
}
// C line 25663
4 => {
let _ = JS_FreeAtom(ctx, class_name);
vm_block = 3; continue;
}
// C line 25662 labels: fail
5 => {
let _ = JS_FreeAtom(ctx, name);
vm_block = 4; continue;
}
// C line 25660
6 => {
return (0 as i32);
}
// C line 25659
7 => {
let _ = { let assigned = ((saved_js_mode) as u8); (*(fd)).js_mode = (assigned) as u8; assigned };
vm_block = 6; continue;
}
// C line 25658
8 => {
let _ = JS_FreeAtom(ctx, class_var_name);
vm_block = 7; continue;
}
// C line 25657
9 => {
let _ = JS_FreeAtom(ctx, class_name);
vm_block = 8; continue;
}
// C line 25654
10 => {
vm_block = 5; continue;
}
// C line 25650
11 => {
vm_block = if ((!(!(add_export_entry(s, (*(fd)).module, class_var_name, if ((((((export_flag) as u32)) == ((((JS_PARSE_EXPORT_NAMED as i32)) as u32))) as i32)) != 0 { class_var_name } else { (((crate::quickjs_atom::JS_ATOM_default as i32)) as JSAtom) }, (((JS_EXPORT_TYPE_LOCAL as i32)) as JSExportTypeEnum))).is_null()) as i32)) != 0 { 10 } else { 9 }; continue;
}
// C line 25649
12 => {
vm_block = if ((((((export_flag) as u32)) != ((((JS_PARSE_EXPORT_NONE as i32)) as u32))) as i32)) != 0 { 11 } else { 9 }; continue;
}
// C line 25638
13 => {
let _ = emit_u16(s, (((*(fd)).scope_level) as u16));
vm_block = 12; continue;
}
// C line 25637
14 => {
let _ = emit_atom(s, class_var_name);
vm_block = 13; continue;
}
// C line 25636
15 => {
let _ = emit_op(s, (((OP_scope_put_var_init as i32)) as u8));
vm_block = 14; continue;
}
// C line 25635
16 => {
vm_block = 5; continue;
}
// C line 25634
17 => {
vm_block = if ((((define_var(s, fd, class_var_name, (((JS_VAR_DEF_LET as i32)) as JSVarDefEnum))) < ((0 as i32))) as i32)) != 0 { 16 } else { 15 }; continue;
}
// C line 25645
18 => {
let _ = emit_u32(s, (((((*(fd)).last_opcode_pos).wrapping_add((1 as i32))).wrapping_sub(define_class_offset)) as u32));
vm_block = 12; continue;
}
// C line 25644
19 => {
let _ = emit_op(s, (((OP_set_class_name as i32)) as u8));
vm_block = 18; continue;
}
// C line 25640
20 => {
vm_block = if ((((class_name) == ((((0 as i32)) as JSAtom))) as i32)) != 0 { 19 } else { 12 }; continue;
}
// C line 25633
21 => {
vm_block = if ((((class_var_name) != ((((0 as i32)) as JSAtom))) as i32)) != 0 { 17 } else { 20 }; continue;
}
// C line 25630
22 => {
let _ = pop_scope(s);
vm_block = 21; continue;
}
// C line 25629
23 => {
let _ = pop_scope(s);
vm_block = 22; continue;
}
// C line 25626
24 => {
let _ = emit_op(s, (((OP_drop as i32)) as u8));
vm_block = 23; continue;
}
// C line 25625
25 => {
let _ = emit_u16(s, (((0 as i32)) as u16));
vm_block = 24; continue;
}
// C line 25624
26 => {
let _ = emit_op(s, (((OP_call_method as i32)) as u8));
vm_block = 25; continue;
}
// C line 25623
27 => {
let _ = emit_class_init_end(s, cf_4);
vm_block = 26; continue;
}
// C line 25622
28 => {
let _ = emit_op(s, (((OP_dup as i32)) as u8));
vm_block = 27; continue;
}
// C line 25621
29 => {
cf_4 = core::ptr::addr_of_mut!(*((class_fields).as_mut_ptr()).offset(((1 as i32)) as isize));
vm_block = 28; continue;
}
// C line 25620
30 => {
vm_block = if (((((*((class_fields).as_mut_ptr()).offset(((1 as i32)) as isize)).fields_init_fd) != (core::ptr::null_mut::<JSFunctionDef>())) as i32)) != 0 { 29 } else { 23 }; continue;
}
// C line 25616
31 => {
let _ = emit_u16(s, (((*(fd)).scope_level) as u16));
vm_block = 30; continue;
}
// C line 25615
32 => {
let _ = emit_atom(s, class_name);
vm_block = 31; continue;
}
// C line 25614
33 => {
let _ = emit_op(s, (((OP_scope_put_var_init as i32)) as u8));
vm_block = 32; continue;
}
// C line 25613
34 => {
let _ = emit_op(s, (((OP_dup as i32)) as u8));
vm_block = 33; continue;
}
// C line 25609
35 => {
vm_block = if ((((class_name) != ((((0 as i32)) as JSAtom))) as i32)) != 0 { 34 } else { 30 }; continue;
}
// C line 25606
36 => {
let _ = emit_op(s, (((OP_add_brand as i32)) as u8));
vm_block = 35; continue;
}
// C line 25605
37 => {
let _ = emit_op(s, (((OP_dup as i32)) as u8));
vm_block = 36; continue;
}
// C line 25604
38 => {
let _ = emit_op(s, (((OP_dup as i32)) as u8));
vm_block = 37; continue;
}
// C line 25602
39 => {
vm_block = if ((*((class_fields).as_mut_ptr()).offset(((1 as i32)) as isize)).need_brand) != 0 { 38 } else { 35 }; continue;
}
// C line 25600
40 => {
let _ = emit_op(s, (((OP_drop as i32)) as u8));
vm_block = 39; continue;
}
// C line 25596
41 => {
let _ = emit_u16(s, (((*((*(s)).cur_func)).scope_level) as u16));
vm_block = 40; continue;
}
// C line 25595
42 => {
let _ = emit_atom(s, (((crate::quickjs_atom::JS_ATOM_class_fields_init as i32)) as JSAtom));
vm_block = 41; continue;
}
// C line 25594
43 => {
let _ = emit_op(s, (((OP_scope_put_var_init as i32)) as u8));
vm_block = 42; continue;
}
// C line 25590
44 => {
let _ = emit_class_init_end(s, cf_3);
vm_block = 43; continue;
}
// C line 25592
45 => {
let _ = emit_op(s, (((OP_undefined as i32)) as u8));
vm_block = 43; continue;
}
// C line 25589
46 => {
vm_block = if !((*(cf_3)).fields_init_fd).is_null() { 44 } else { 45 }; continue;
}
// C line 25588
47 => {
vm_block = 5; continue;
}
// C line 25587
48 => {
vm_block = if ((((var_idx) < ((0 as i32))) as i32)) != 0 { 47 } else { 46 }; continue;
}
// C line 25585
49 => {
let _ = { let assigned = define_var(s, fd, (((crate::quickjs_atom::JS_ATOM_class_fields_init as i32)) as JSAtom), (((JS_VAR_DEF_CONST as i32)) as JSVarDefEnum)); var_idx = assigned; assigned };
vm_block = 48; continue;
}
// C line 25580
50 => {
let _ = { let assigned = (((OP_push_true as i32)) as u8); *(((*((*(cf_3)).fields_init_fd)).byte_code).buf).offset(((*(cf_3)).brand_push_pos) as isize) = assigned; assigned };
vm_block = 49; continue;
}
// C line 25576
51 => {
vm_block = 5; continue;
}
// C line 25575
52 => {
vm_block = if (emit_class_init_start(s, cf_3)) != 0 { 51 } else { 50 }; continue;
}
// C line 25574
53 => {
vm_block = if ((!(!((*(cf_3)).fields_init_fd).is_null()) as i32)) != 0 { 52 } else { 50 }; continue;
}
// C line 25571
54 => {
let _ = emit_op(s, (((OP_add_brand as i32)) as u8));
vm_block = 53; continue;
}
// C line 25570
55 => {
let _ = emit_op(s, (((OP_swap as i32)) as u8));
vm_block = 54; continue;
}
// C line 25569
56 => {
let _ = emit_op(s, (((OP_null as i32)) as u8));
vm_block = 55; continue;
}
// C line 25568
57 => {
let _ = emit_op(s, (((OP_dup as i32)) as u8));
vm_block = 56; continue;
}
// C line 25566
58 => {
vm_block = if ((*(cf_3)).need_brand) != 0 { 57 } else { 49 }; continue;
}
// C line 25563
59 => {
cf_3 = core::ptr::addr_of_mut!(*((class_fields).as_mut_ptr()).offset(((0 as i32)) as isize));
vm_block = 58; continue;
}
// C line 25560
60 => {
vm_block = 5; continue;
}
// C line 25559
61 => {
vm_block = if (next_token(s)) != 0 { 60 } else { 59 }; continue;
}
// C line 25555
62 => {
vm_block = 5; continue;
}
// C line 25554
63 => {
vm_block = if ((!(!((*(ctor_fd)).source).is_null()) as i32)) != 0 { 62 } else { 61 }; continue;
}
// C line 25552
64 => {
let _ = { let assigned = js_strndup(ctx, ((class_start_ptr) as *const c_char), (((*(ctor_fd)).source_len) as usize)); (*(ctor_fd)).source = assigned; assigned };
vm_block = 63; continue;
}
// C line 25551
65 => {
let _ = { let assigned = (((((*(s)).buf_ptr).offset_from(class_start_ptr) as i64)) as i32); (*(ctor_fd)).source_len = assigned; assigned };
vm_block = 64; continue;
}
// C line 25550
66 => {
let _ = js_free(ctx, (((*(ctor_fd)).source) as *mut c_void));
vm_block = 65; continue;
}
// C line 25549
67 => {
vm_block = if ((!(((*(fd)).strip_source()) != 0) as i32)) != 0 { 66 } else { 61 }; continue;
}
// C line 25546
68 => {
let _ = put_u32((((*(fd)).byte_code).buf).offset(((ctor_cpool_offset) as isize)), (((*(ctor_fd)).parent_cpool_idx) as u32));
vm_block = 67; continue;
}
// C line 25543
69 => {
vm_block = 5; continue;
}
// C line 25542
70 => {
vm_block = if (js_parse_class_default_ctor(s, ((class_flags) & (((1 as i32)).wrapping_shl(((0 as i32)) as u32))), core::ptr::addr_of_mut!(ctor_fd))) != 0 { 69 } else { 68 }; continue;
}
// C line 25541
71 => {
vm_block = if ((!(!(ctor_fd).is_null()) as i32)) != 0 { 70 } else { 68 }; continue;
}
// C line 25538
72 => {
vm_block = 5; continue;
}
// C line 25537
73 => {
let _ = js_parse_error_cargs(s, c"expecting '%c'".as_ptr(), &[ParserFormatArg::Char(((125 as i32)) as u8)]);
vm_block = 72; continue;
}
// C line 25536
74 => {
vm_block = if ((((((*(s)).token).val) != ((125 as i32))) as i32)) != 0 { 73 } else { 71 }; continue;
}
// C line 25249
75 => {
vm_block = if ((((((*(s)).token).val) != ((125 as i32))) as i32)) != 0 { 264 } else { 74 }; continue;
}
// C line 25533
76 => {
let _ = { let assigned = (((0 as i32)) as JSAtom); name = assigned; assigned };
vm_block = 75; continue;
}
// C line 25532
77 => {
let _ = JS_FreeAtom(ctx, name);
vm_block = 76; continue;
}
// C line 25531
78 => {
let _ = emit_op(s, (((OP_swap as i32)) as u8));
vm_block = 77; continue;
}
// C line 25530
79 => {
vm_block = if (is_static) != 0 { 78 } else { 77 }; continue;
}
// C line 25378
80 => {
let _ = emit_u16(s, (((*((*(s)).cur_func)).scope_level) as u16));
vm_block = 79; continue;
}
// C line 25374
81 => {
vm_block = 5; continue;
}
// C line 25373
82 => {
vm_block = if ((((ret) < ((0 as i32))) as i32)) != 0 { 81 } else { 80 }; continue;
}
// C line 25372
83 => {
let _ = JS_FreeAtom(ctx, setter_name);
vm_block = 82; continue;
}
// C line 25370
84 => {
let _ = { let assigned = add_private_class_field(s, fd, setter_name, (((JS_VAR_PRIVATE_SETTER as i32)) as JSVarKindEnum), is_static); ret = assigned; assigned };
vm_block = 83; continue;
}
// C line 25369
85 => {
let _ = emit_atom(s, setter_name);
vm_block = 84; continue;
}
// C line 25368
86 => {
vm_block = 5; continue;
}
// C line 25367
87 => {
vm_block = if ((((setter_name) == ((((0 as i32)) as JSAtom))) as i32)) != 0 { 86 } else { 85 }; continue;
}
// C line 25366
88 => {
let _ = { let assigned = get_private_setter_name(ctx, name); setter_name = assigned; assigned };
vm_block = 87; continue;
}
// C line 25376
89 => {
let _ = emit_atom(s, name);
vm_block = 80; continue;
}
// C line 25362
90 => {
vm_block = if (is_set) != 0 { 88 } else { 89 }; continue;
}
// C line 25361
91 => {
let _ = emit_op(s, (((OP_scope_put_var_init as i32)) as u8));
vm_block = 90; continue;
}
// C line 25359
92 => {
let _ = emit_op(s, (((OP_set_home_object as i32)) as u8));
vm_block = 91; continue;
}
// C line 25358
93 => {
let _ = { let assigned = (1 as i32); (*(method_fd_1)).need_home_object = assigned; assigned };
vm_block = 92; continue;
}
// C line 25386
94 => {
let _ = emit_u8(s, ((((1 as i32)).wrapping_add(is_set)) as u8));
vm_block = 79; continue;
}
// C line 25381
95 => {
let _ = emit_op(s, (((OP_define_method_computed as i32)) as u8));
vm_block = 94; continue;
}
// C line 25384
96 => {
let _ = emit_atom(s, name);
vm_block = 94; continue;
}
// C line 25383
97 => {
let _ = emit_op(s, (((OP_define_method as i32)) as u8));
vm_block = 96; continue;
}
// C line 25380
98 => {
vm_block = if ((((name) == ((((0 as i32)) as JSAtom))) as i32)) != 0 { 95 } else { 97 }; continue;
}
// C line 25357
99 => {
vm_block = if (is_private) != 0 { 93 } else { 98 }; continue;
}
// C line 25356
100 => {
vm_block = 5; continue;
}
// C line 25352
101 => {
vm_block = if (js_parse_function_decl2(s, ((((JS_PARSE_FUNC_GETTER as i32)).wrapping_add(is_set)) as JSParseFunctionEnum), (((JS_FUNC_NORMAL as i32)) as JSFunctionKindEnum), (((0 as i32)) as JSAtom), start_ptr, (((JS_PARSE_EXPORT_NONE as i32)) as JSParseExportEnum), core::ptr::addr_of_mut!(method_fd_1))) != 0 { 100 } else { 99 }; continue;
}
// C line 25349
102 => {
let _ = { let assigned = (1 as i32); (*((class_fields).as_mut_ptr()).offset((is_static) as isize)).need_brand = assigned; assigned };
vm_block = 101; continue;
}
// C line 25343
103 => {
let _ = { let assigned = (((JS_VAR_PRIVATE_GETTER_SETTER as i32)) as u8); (*((*(fd)).vars).offset((idx) as isize)).set_var_kind((assigned) as _); assigned };
vm_block = 102; continue;
}
// C line 25341
104 => {
vm_block = 186; continue;
}
// C line 25335
105 => {
vm_block = if ((((((((((((((((((((var_kind) == ((JS_VAR_PRIVATE_FIELD as i32))) as i32)) != 0) || (((((var_kind) == ((JS_VAR_PRIVATE_METHOD as i32))) as i32)) != 0)) as i32)) != 0) || (((((var_kind) == ((JS_VAR_PRIVATE_GETTER_SETTER as i32))) as i32)) != 0)) as i32)) != 0) || (((((var_kind) == (((JS_VAR_PRIVATE_GETTER as i32)).wrapping_add(is_set))) as i32)) != 0)) as i32)) != 0) || (((((((((var_kind) == ((((JS_VAR_PRIVATE_GETTER as i32)).wrapping_add((1 as i32))).wrapping_sub(is_set))) as i32)) != 0) && (((((is_static) != (is_static1)) as i32)) != 0)) as i32)) != 0)) as i32)) != 0 { 104 } else { 103 }; continue;
}
// C line 25334
106 => {
let _ = { let assigned = (((*((*(fd)).vars).offset((idx) as isize)).is_static_private()) as i32); is_static1 = assigned; assigned };
vm_block = 105; continue;
}
// C line 25333
107 => {
let _ = { let assigned = (((*((*(fd)).vars).offset((idx) as isize)).var_kind()) as i32); var_kind = assigned; assigned };
vm_block = 106; continue;
}
// C line 25347
108 => {
vm_block = 5; continue;
}
// C line 25345
109 => {
vm_block = if ((((add_private_class_field(s, fd, name, ((((JS_VAR_PRIVATE_GETTER as i32)).wrapping_add(is_set)) as JSVarKindEnum), is_static)) < ((0 as i32))) as i32)) != 0 { 108 } else { 102 }; continue;
}
// C line 25332
110 => {
vm_block = if ((((idx) >= ((0 as i32))) as i32)) != 0 { 107 } else { 109 }; continue;
}
// C line 25331
111 => {
let _ = { let assigned = find_private_class_field(ctx, fd, name, (*(fd)).scope_level); idx = assigned; assigned };
vm_block = 110; continue;
}
// C line 25329
112 => {
vm_block = if (is_private) != 0 { 111 } else { 101 }; continue;
}
// C line 25326
113 => {
is_set = (prop_type).wrapping_sub((2 as i32));
vm_block = 112; continue;
}
// C line 25472
114 => {
vm_block = 5; continue;
}
// C line 25471
115 => {
vm_block = if (js_parse_expect_semi(s)) != 0 { 114 } else { 79 }; continue;
}
// C line 25470
116 => {
let _ = { let assigned = (*((*(s)).cur_func)).parent; (*(s)).cur_func = assigned; assigned };
vm_block = 115; continue;
}
// C line 25460
117 => {
let _ = emit_op(s, (((OP_define_private_field as i32)) as u8));
vm_block = 116; continue;
}
// C line 25459
118 => {
let _ = set_object_name_computed(s);
vm_block = 117; continue;
}
// C line 25464
119 => {
let _ = emit_op(s, (((OP_drop as i32)) as u8));
vm_block = 116; continue;
}
// C line 25463
120 => {
let _ = emit_op(s, (((OP_define_array_el as i32)) as u8));
vm_block = 119; continue;
}
// C line 25462
121 => {
let _ = set_object_name_computed(s);
vm_block = 120; continue;
}
// C line 25468
122 => {
let _ = emit_atom(s, name);
vm_block = 116; continue;
}
// C line 25467
123 => {
let _ = emit_op(s, (((OP_define_field as i32)) as u8));
vm_block = 122; continue;
}
// C line 25466
124 => {
let _ = set_object_name(s, name);
vm_block = 123; continue;
}
// C line 25461
125 => {
vm_block = if ((((name) == ((((0 as i32)) as JSAtom))) as i32)) != 0 { 121 } else { 124 }; continue;
}
// C line 25458
126 => {
vm_block = if (is_private) != 0 { 118 } else { 125 }; continue;
}
// C line 25454
127 => {
vm_block = 5; continue;
}
// C line 25453
128 => {
vm_block = if (js_parse_assign_expr(s)) != 0 { 127 } else { 126 }; continue;
}
// C line 25452
129 => {
vm_block = 5; continue;
}
// C line 25451
130 => {
vm_block = if (next_token(s)) != 0 { 129 } else { 128 }; continue;
}
// C line 25456
131 => {
let _ = emit_op(s, (((OP_undefined as i32)) as u8));
vm_block = 126; continue;
}
// C line 25450
132 => {
vm_block = if ((((((*(s)).token).val) == ((61 as i32))) as i32)) != 0 { 130 } else { 131 }; continue;
}
// C line 25443
133 => {
let _ = JS_FreeAtom(ctx, field_var_name);
vm_block = 132; continue;
}
// C line 25442
134 => {
let _ = { let old = (*(cf_2)).computed_fields_count; (*(cf_2)).computed_fields_count = ((*(cf_2)).computed_fields_count).wrapping_add(1); old };
vm_block = 133; continue;
}
// C line 25441
135 => {
let _ = emit_u16(s, (((*((*(s)).cur_func)).scope_level) as u16));
vm_block = 134; continue;
}
// C line 25440
136 => {
let _ = emit_atom(s, field_var_name);
vm_block = 135; continue;
}
// C line 25439
137 => {
let _ = emit_op(s, (((OP_scope_get_var as i32)) as u8));
vm_block = 136; continue;
}
// C line 25447
138 => {
let _ = emit_u16(s, (((*((*(s)).cur_func)).scope_level) as u16));
vm_block = 132; continue;
}
// C line 25446
139 => {
let _ = emit_atom(s, name);
vm_block = 138; continue;
}
// C line 25445
140 => {
let _ = emit_op(s, (((OP_scope_get_var as i32)) as u8));
vm_block = 139; continue;
}
// C line 25444
141 => {
vm_block = if (is_private) != 0 { 140 } else { 132 }; continue;
}
// C line 25438
142 => {
vm_block = if ((((name) == ((((0 as i32)) as JSAtom))) as i32)) != 0 { 137 } else { 141 }; continue;
}
// C line 25436
143 => {
let _ = emit_u16(s, (((0 as i32)) as u16));
vm_block = 142; continue;
}
// C line 25435
144 => {
let _ = emit_atom(s, (((crate::quickjs_atom::JS_ATOM_this as i32)) as JSAtom));
vm_block = 143; continue;
}
// C line 25434
145 => {
let _ = emit_op(s, (((OP_scope_get_var as i32)) as u8));
vm_block = 144; continue;
}
// C line 25433
146 => {
let _ = { let assigned = (*(cf_2)).fields_init_fd; (*(s)).cur_func = assigned; assigned };
vm_block = 145; continue;
}
// C line 25431
147 => {
let _ = emit_u16(s, (((*((*(s)).cur_func)).scope_level) as u16));
vm_block = 146; continue;
}
// C line 25430
148 => {
let _ = emit_atom(s, field_var_name);
vm_block = 147; continue;
}
// C line 25429
149 => {
let _ = emit_op(s, (((OP_scope_put_var_init as i32)) as u8));
vm_block = 148; continue;
}
// C line 25428
150 => {
let _ = emit_op(s, (((OP_to_propkey as i32)) as u8));
vm_block = 149; continue;
}
// C line 25426
151 => {
vm_block = 5; continue;
}
// C line 25425
152 => {
let _ = JS_FreeAtom(ctx, field_var_name);
vm_block = 151; continue;
}
// C line 25424
153 => {
vm_block = if ((((define_var(s, fd, field_var_name, (((JS_VAR_DEF_CONST as i32)) as JSVarDefEnum))) < ((0 as i32))) as i32)) != 0 { 152 } else { 150 }; continue;
}
// C line 25423
154 => {
vm_block = 5; continue;
}
// C line 25422
155 => {
vm_block = if ((((field_var_name) == ((((0 as i32)) as JSAtom))) as i32)) != 0 { 154 } else { 153 }; continue;
}
// C line 25421
156 => {
let _ = { let assigned = js_atom_concat_num(ctx, ((((crate::quickjs_atom::JS_ATOM_computed_field as i32)).wrapping_add(is_static)) as JSAtom), (((*(cf_2)).computed_fields_count) as u32)); field_var_name = assigned; assigned };
vm_block = 155; continue;
}
// C line 25419
157 => {
vm_block = if ((((name) == ((((0 as i32)) as JSAtom))) as i32)) != 0 { 156 } else { 146 }; continue;
}
// C line 25417
158 => {
vm_block = 5; continue;
}
// C line 25416
159 => {
vm_block = if (emit_class_init_start(s, cf_2)) != 0 { 158 } else { 157 }; continue;
}
// C line 25415
160 => {
vm_block = if ((!(!((*(cf_2)).fields_init_fd).is_null()) as i32)) != 0 { 159 } else { 157 }; continue;
}
// C line 25412
161 => {
let _ = emit_u16(s, (((*((*(s)).cur_func)).scope_level) as u16));
vm_block = 160; continue;
}
// C line 25411
162 => {
let _ = emit_atom(s, name);
vm_block = 161; continue;
}
// C line 25410
163 => {
let _ = emit_op(s, (((OP_scope_put_var_init as i32)) as u8));
vm_block = 162; continue;
}
// C line 25409
164 => {
let _ = emit_atom(s, name);
vm_block = 163; continue;
}
// C line 25408
165 => {
let _ = emit_op(s, (((OP_private_symbol as i32)) as u8));
vm_block = 164; continue;
}
// C line 25407
166 => {
vm_block = 5; continue;
}
// C line 25405
167 => {
vm_block = if ((((add_private_class_field(s, fd, name, (((JS_VAR_PRIVATE_FIELD as i32)) as JSVarKindEnum), is_static)) < ((0 as i32))) as i32)) != 0 { 166 } else { 165 }; continue;
}
// C line 25403
168 => {
vm_block = 186; continue;
}
// C line 25401
169 => {
vm_block = if ((((find_private_class_field(ctx, fd, name, (*(fd)).scope_level)) >= ((0 as i32))) as i32)) != 0 { 168 } else { 167 }; continue;
}
// C line 25400
170 => {
vm_block = if (is_private) != 0 { 169 } else { 160 }; continue;
}
// C line 25397
171 => {
vm_block = 5; continue;
}
// C line 25396
172 => {
let _ = js_parse_error(s, c"invalid field name".as_ptr());
vm_block = 171; continue;
}
// C line 25395
173 => {
vm_block = if ((((((((name) == ((((crate::quickjs_atom::JS_ATOM_constructor as i32)) as JSAtom))) as i32)) != 0) || (((((name) == ((((crate::quickjs_atom::JS_ATOM_prototype as i32)) as JSAtom))) as i32)) != 0)) as i32)) != 0 { 172 } else { 170 }; continue;
}
// C line 25390
174 => {
field_var_name = (((0 as i32)) as JSAtom);
vm_block = 173; continue;
}
// C line 25389
175 => {
cf_2 = core::ptr::addr_of_mut!(*((class_fields).as_mut_ptr()).offset((is_static) as isize));
vm_block = 174; continue;
}
// C line 25502
176 => {
let _ = { let assigned = method_fd; ctor_fd = assigned; assigned };
vm_block = 79; continue;
}
// C line 25519
177 => {
let _ = emit_u16(s, (((*((*(s)).cur_func)).scope_level) as u16));
vm_block = 79; continue;
}
// C line 25518
178 => {
let _ = emit_atom(s, name);
vm_block = 177; continue;
}
// C line 25517
179 => {
let _ = emit_op(s, (((OP_scope_put_var_init as i32)) as u8));
vm_block = 178; continue;
}
// C line 25516
180 => {
let _ = emit_atom(s, name);
vm_block = 179; continue;
}
// C line 25515
181 => {
let _ = emit_op(s, (((OP_set_name as i32)) as u8));
vm_block = 180; continue;
}
// C line 25514
182 => {
let _ = emit_op(s, (((OP_set_home_object as i32)) as u8));
vm_block = 181; continue;
}
// C line 25513
183 => {
vm_block = 5; continue;
}
// C line 25511
184 => {
vm_block = if ((((add_private_class_field(s, fd, name, (((JS_VAR_PRIVATE_METHOD as i32)) as JSVarKindEnum), is_static)) < ((0 as i32))) as i32)) != 0 { 183 } else { 182 }; continue;
}
// C line 25509
185 => {
vm_block = 5; continue;
}
// C line 25508 labels: private_field_already_defined
186 => {
let _ = js_parse_error(s, c"private class field is already defined".as_ptr());
vm_block = 185; continue;
}
// C line 25505
187 => {
vm_block = if ((((find_private_class_field(ctx, fd, name, (*(fd)).scope_level)) >= ((0 as i32))) as i32)) != 0 { 186 } else { 184 }; continue;
}
// C line 25504
188 => {
let _ = { let assigned = (1 as i32); (*(method_fd)).need_home_object = assigned; assigned };
vm_block = 187; continue;
}
// C line 25527
189 => {
let _ = emit_u8(s, (((0 as i32)) as u8));
vm_block = 79; continue;
}
// C line 25522
190 => {
let _ = emit_op(s, (((OP_define_method_computed as i32)) as u8));
vm_block = 189; continue;
}
// C line 25525
191 => {
let _ = emit_atom(s, name);
vm_block = 189; continue;
}
// C line 25524
192 => {
let _ = emit_op(s, (((OP_define_method as i32)) as u8));
vm_block = 191; continue;
}
// C line 25521
193 => {
vm_block = if ((((name) == ((((0 as i32)) as JSAtom))) as i32)) != 0 { 190 } else { 192 }; continue;
}
// C line 25503
194 => {
vm_block = if (is_private) != 0 { 188 } else { 193 }; continue;
}
// C line 25500
195 => {
vm_block = if ((((((((((func_type) as u32)) == ((((JS_PARSE_FUNC_DERIVED_CLASS_CONSTRUCTOR as i32)) as u32))) as i32)) != 0) || (((((((func_type) as u32)) == ((((JS_PARSE_FUNC_CLASS_CONSTRUCTOR as i32)) as u32))) as i32)) != 0)) as i32)) != 0 { 176 } else { 194 }; continue;
}
// C line 25499
196 => {
vm_block = 5; continue;
}
// C line 25498
197 => {
vm_block = if (js_parse_function_decl2(s, func_type, func_kind, (((0 as i32)) as JSAtom), start_ptr, (((JS_PARSE_EXPORT_NONE as i32)) as JSParseExportEnum), core::ptr::addr_of_mut!(method_fd))) != 0 { 196 } else { 195 }; continue;
}
// C line 25496
198 => {
let _ = { let assigned = (1 as i32); (*((class_fields).as_mut_ptr()).offset((is_static) as isize)).need_brand = assigned; assigned };
vm_block = 197; continue;
}
// C line 25495
199 => {
vm_block = if (is_private) != 0 { 198 } else { 197 }; continue;
}
// C line 25480
200 => {
let _ = { let assigned = (((JS_FUNC_GENERATOR as i32)) as JSFunctionKindEnum); func_kind = assigned; assigned };
vm_block = 199; continue;
}
// C line 25482
201 => {
let _ = { let assigned = (((JS_FUNC_ASYNC as i32)) as JSFunctionKindEnum); func_kind = assigned; assigned };
vm_block = 199; continue;
}
// C line 25484
202 => {
let _ = { let assigned = (((JS_FUNC_ASYNC_GENERATOR as i32)) as JSFunctionKindEnum); func_kind = assigned; assigned };
vm_block = 199; continue;
}
// C line 25491
203 => {
let _ = { let assigned = (((JS_PARSE_FUNC_DERIVED_CLASS_CONSTRUCTOR as i32)) as JSParseFunctionEnum); func_type = assigned; assigned };
vm_block = 199; continue;
}
// C line 25493
204 => {
let _ = { let assigned = (((JS_PARSE_FUNC_CLASS_CONSTRUCTOR as i32)) as JSParseFunctionEnum); func_type = assigned; assigned };
vm_block = 199; continue;
}
// C line 25490
205 => {
vm_block = if (((class_flags) & (((1 as i32)).wrapping_shl(((0 as i32)) as u32)))) != 0 { 203 } else { 204 }; continue;
}
// C line 25488
206 => {
vm_block = 5; continue;
}
// C line 25487
207 => {
let _ = js_parse_error(s, c"property constructor appears more than once".as_ptr());
vm_block = 206; continue;
}
// C line 25486
208 => {
vm_block = if !(ctor_fd).is_null() { 207 } else { 205 }; continue;
}
// C line 25485
209 => {
vm_block = if ((((((((name) == ((((crate::quickjs_atom::JS_ATOM_constructor as i32)) as JSAtom))) as i32)) != 0) && (((!((is_static) != 0) as i32)) != 0)) as i32)) != 0 { 208 } else { 199 }; continue;
}
// C line 25483
210 => {
vm_block = if ((((prop_type) == ((6 as i32))) as i32)) != 0 { 202 } else { 209 }; continue;
}
// C line 25481
211 => {
vm_block = if ((((prop_type) == ((5 as i32))) as i32)) != 0 { 201 } else { 210 }; continue;
}
// C line 25479
212 => {
vm_block = if ((((prop_type) == ((4 as i32))) as i32)) != 0 { 200 } else { 211 }; continue;
}
// C line 25478
213 => {
let _ = { let assigned = (((JS_FUNC_NORMAL as i32)) as JSFunctionKindEnum); func_kind = assigned; assigned };
vm_block = 212; continue;
}
// C line 25477
214 => {
let _ = { let assigned = (((JS_PARSE_FUNC_METHOD as i32)) as JSParseFunctionEnum); func_type = assigned; assigned };
vm_block = 213; continue;
}
// C line 25388
215 => {
vm_block = if ((((((((prop_type) == ((0 as i32))) as i32)) != 0) && (((((((*(s)).token).val) != ((40 as i32))) as i32)) != 0)) as i32)) != 0 { 175 } else { 214 }; continue;
}
// C line 25325
216 => {
vm_block = if ((((((((prop_type) == ((2 as i32))) as i32)) != 0) || (((((prop_type) == ((3 as i32))) as i32)) != 0)) as i32)) != 0 { 113 } else { 215 }; continue;
}
// C line 25323
217 => {
vm_block = 5; continue;
}
// C line 25322
218 => {
let _ = js_parse_error(s, c"invalid method name".as_ptr());
vm_block = 217; continue;
}
// C line 25318
219 => {
vm_block = if ((((((((((((((((((((name) == ((((crate::quickjs_atom::JS_ATOM_constructor as i32)) as JSAtom))) as i32)) != 0) && (((!((is_static) != 0) as i32)) != 0)) as i32)) != 0) && (((((prop_type) != ((0 as i32))) as i32)) != 0)) as i32)) != 0) || (((((((((name) == ((((crate::quickjs_atom::JS_ATOM_prototype as i32)) as JSAtom))) as i32)) != 0) && ((is_static) != 0)) as i32)) != 0)) as i32)) != 0) || (((((name) == ((((crate::quickjs_atom::JS_ATOM_hash_constructor as i32)) as JSAtom))) as i32)) != 0)) as i32)) != 0 { 218 } else { 216 }; continue;
}
// C line 25316
220 => {
let _ = { prop_type = (((((prop_type) as i32)) & ((!(((1 as i32)).wrapping_shl(((4 as i32)) as u32)))))) as i32; prop_type };
vm_block = 219; continue;
}
// C line 25315
221 => {
let _ = { let assigned = ((prop_type) & (((1 as i32)).wrapping_shl(((4 as i32)) as u32))); is_private = assigned; assigned };
vm_block = 220; continue;
}
// C line 25313
222 => {
vm_block = 5; continue;
}
// C line 25312
223 => {
vm_block = if ((((prop_type) < ((0 as i32))) as i32)) != 0 { 222 } else { 221 }; continue;
}
// C line 25311
224 => {
let _ = { let assigned = js_parse_property_name(s, core::ptr::addr_of_mut!(name), (1 as i32), (0 as i32), (1 as i32)); prop_type = assigned; assigned };
vm_block = 223; continue;
}
// C line 25310
225 => {
vm_block = if ((((prop_type) < ((0 as i32))) as i32)) != 0 { 224 } else { 221 }; continue;
}
// C line 25309
226 => {
let _ = { let assigned = ((*(s)).token).ptr; start_ptr = assigned; assigned };
vm_block = 225; continue;
}
// C line 25308
227 => {
let _ = emit_op(s, (((OP_swap as i32)) as u8));
vm_block = 226; continue;
}
// C line 25307
228 => {
vm_block = if (is_static) != 0 { 227 } else { 226 }; continue;
}
// C line 25304
229 => {
let _ = { let assigned = (0 as i32); prop_type = assigned; assigned };
vm_block = 228; continue;
}
// C line 25303
230 => {
let _ = { let assigned = JS_DupAtom(ctx, (((crate::quickjs_atom::JS_ATOM_static as i32)) as JSAtom)); name = assigned; assigned };
vm_block = 229; continue;
}
// C line 25302
231 => {
let _ = { let assigned = (0 as i32); is_static = assigned; assigned };
vm_block = 230; continue;
}
// C line 25301
232 => {
vm_block = if ((((((((((*(s)).token).val) == ((59 as i32))) as i32)) != 0) || (((((((*(s)).token).val) == ((61 as i32))) as i32)) != 0)) as i32)) != 0 { 231 } else { 228 }; continue;
}
// C line 25298
233 => {
vm_block = 75; continue;
}
// C line 25297
234 => {
let _ = { let assigned = (*((*(s)).cur_func)).parent; (*(s)).cur_func = assigned; assigned };
vm_block = 233; continue;
}
// C line 25296
235 => {
let _ = pop_scope(s);
vm_block = 234; continue;
}
// C line 25294
236 => {
let _ = emit_op(s, (((OP_drop as i32)) as u8));
vm_block = 235; continue;
}
// C line 25292
237 => {
let _ = emit_u16(s, (((0 as i32)) as u16));
vm_block = 236; continue;
}
// C line 25291
238 => {
let _ = emit_op(s, (((OP_call_method as i32)) as u8));
vm_block = 237; continue;
}
// C line 25289
239 => {
let _ = emit_op(s, (((OP_swap as i32)) as u8));
vm_block = 238; continue;
}
// C line 25287
240 => {
let _ = emit_u16(s, (((0 as i32)) as u16));
vm_block = 239; continue;
}
// C line 25286
241 => {
let _ = emit_atom(s, (((crate::quickjs_atom::JS_ATOM_this as i32)) as JSAtom));
vm_block = 240; continue;
}
// C line 25285
242 => {
let _ = emit_op(s, (((OP_scope_get_var as i32)) as u8));
vm_block = 241; continue;
}
// C line 25284
243 => {
let _ = push_scope(s);
vm_block = 242; continue;
}
// C line 25281
244 => {
vm_block = 5; continue;
}
// C line 25277
245 => {
vm_block = if ((((js_parse_function_decl2(s, (((JS_PARSE_FUNC_CLASS_STATIC_INIT as i32)) as JSParseFunctionEnum), (((JS_FUNC_NORMAL as i32)) as JSFunctionKindEnum), (((0 as i32)) as JSAtom), ((*(s)).token).ptr, (((JS_PARSE_EXPORT_NONE as i32)) as JSParseExportEnum), core::ptr::addr_of_mut!(init))) < ((0 as i32))) as i32)) != 0 { 244 } else { 243 }; continue;
}
// C line 25272
246 => {
let _ = { let assigned = (*(cf_1)).fields_init_fd; (*(s)).cur_func = assigned; assigned };
vm_block = 245; continue;
}
// C line 25270
247 => {
vm_block = 5; continue;
}
// C line 25269
248 => {
vm_block = if (emit_class_init_start(s, cf_1)) != 0 { 247 } else { 246 }; continue;
}
// C line 25268
249 => {
vm_block = if ((!(!((*(cf_1)).fields_init_fd).is_null()) as i32)) != 0 { 248 } else { 246 }; continue;
}
// C line 25266
250 => {
cf_1 = core::ptr::addr_of_mut!(*((class_fields).as_mut_ptr()).offset((is_static) as isize));
vm_block = 249; continue;
}
// C line 25265
251 => {
vm_block = if ((((((*(s)).token).val) == ((123 as i32))) as i32)) != 0 { 250 } else { 232 }; continue;
}
// C line 25264
252 => {
vm_block = 5; continue;
}
// C line 25263
253 => {
vm_block = if (next_token(s)) != 0 { 252 } else { 251 }; continue;
}
// C line 25262
254 => {
vm_block = if (is_static) != 0 { 253 } else { 228 }; continue;
}
// C line 25261
255 => {
let _ = { let assigned = ((1 as i32)).wrapping_neg(); prop_type = assigned; assigned };
vm_block = 254; continue;
}
// C line 25259
256 => {
let _ = { let assigned = (1 as i32); is_static = assigned; assigned };
vm_block = 255; continue;
}
// C line 25258
257 => {
vm_block = if ((!(((((((((((((((((next) == ((59 as i32))) as i32)) != 0) || (((((next) == ((125 as i32))) as i32)) != 0)) as i32)) != 0) || (((((next) == ((40 as i32))) as i32)) != 0)) as i32)) != 0) || (((((next) == ((61 as i32))) as i32)) != 0)) as i32)) != 0) as i32)) != 0 { 256 } else { 255 }; continue;
}
// C line 25257
258 => {
next = peek_token(s, (1 as i32));
vm_block = 257; continue;
}
// C line 25256
259 => {
vm_block = if ((((((*(s)).token).val) == ((TOK_STATIC as i32))) as i32)) != 0 { 258 } else { 255 }; continue;
}
// C line 25255
260 => {
let _ = { let assigned = (0 as i32); is_static = assigned; assigned };
vm_block = 259; continue;
}
// C line 25253
261 => {
vm_block = 75; continue;
}
// C line 25252
262 => {
vm_block = 5; continue;
}
// C line 25251
263 => {
vm_block = if (next_token(s)) != 0 { 262 } else { 261 }; continue;
}
// C line 25250
264 => {
vm_block = if ((((((*(s)).token).val) == ((59 as i32))) as i32)) != 0 { 263 } else { 260 }; continue;
}
// C line 25248
265 => {
let _ = { let assigned = core::ptr::null_mut::<JSFunctionDef>(); ctor_fd = assigned; assigned };
vm_block = 75; continue;
}
// C line 25240
266 => {
vm_block = if ((((i) < ((2 as i32))) as i32)) != 0 { 272 } else { 265 }; continue;
}
// C line 25240
267 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 266; continue;
}
// C line 25245
268 => {
let _ = { let assigned = i; (*(cf)).is_static = assigned; assigned };
vm_block = 267; continue;
}
// C line 25244
269 => {
let _ = { let assigned = (0 as i32); (*(cf)).need_brand = assigned; assigned };
vm_block = 268; continue;
}
// C line 25243
270 => {
let _ = { let assigned = (0 as i32); (*(cf)).computed_fields_count = assigned; assigned };
vm_block = 269; continue;
}
// C line 25242
271 => {
let _ = { let assigned = core::ptr::null_mut::<JSFunctionDef>(); (*(cf)).fields_init_fd = assigned; assigned };
vm_block = 270; continue;
}
// C line 25241
272 => {
cf = core::ptr::addr_of_mut!(*((class_fields).as_mut_ptr()).offset((i) as isize));
vm_block = 271; continue;
}
// C line 25240
273 => {
let _ = { let assigned = (0 as i32); i = assigned; assigned };
vm_block = 266; continue;
}
// C line 25238
274 => {
let _ = { let assigned = (*(fd)).last_opcode_pos; define_class_offset = assigned; assigned };
vm_block = 273; continue;
}
// C line 25237
275 => {
let _ = emit_u8(s, ((class_flags) as u8));
vm_block = 274; continue;
}
// C line 25236
276 => {
let _ = emit_atom(s, class_name1);
vm_block = 275; continue;
}
// C line 25235
277 => {
let _ = emit_op(s, (((OP_define_class as i32)) as u8));
vm_block = 276; continue;
}
// C line 25228
278 => {
let _ = { let assigned = (((crate::quickjs_atom::JS_ATOM_default as i32)) as JSAtom); class_name1 = assigned; assigned };
vm_block = 277; continue;
}
// C line 25230
279 => {
let _ = { let assigned = (((crate::quickjs_atom::JS_ATOM_empty_string as i32)) as JSAtom); class_name1 = assigned; assigned };
vm_block = 277; continue;
}
// C line 25227
280 => {
vm_block = if ((((class_var_name) != ((((0 as i32)) as JSAtom))) as i32)) != 0 { 278 } else { 279 }; continue;
}
// C line 25232
281 => {
let _ = { let assigned = class_name; class_name1 = assigned; assigned };
vm_block = 277; continue;
}
// C line 25226
282 => {
vm_block = if ((((class_name) == ((((0 as i32)) as JSAtom))) as i32)) != 0 { 280 } else { 281 }; continue;
}
// C line 25224
283 => {
let _ = emit_u32(s, (((0 as i32)) as u32));
vm_block = 282; continue;
}
// C line 25223
284 => {
let _ = { let assigned = ((((*(fd)).byte_code).size) as i32); ctor_cpool_offset = assigned; assigned };
vm_block = 283; continue;
}
// C line 25222
285 => {
let _ = emit_op(s, (((OP_push_const as i32)) as u8));
vm_block = 284; continue;
}
// C line 25220
286 => {
let _ = push_scope(s);
vm_block = 285; continue;
}
// C line 25217
287 => {
vm_block = 5; continue;
}
// C line 25216
288 => {
vm_block = if (js_parse_expect(s, (123 as i32))) != 0 { 287 } else { 286 }; continue;
}
// C line 25213
289 => {
vm_block = 5; continue;
}
// C line 25212
290 => {
vm_block = if ((((class_name_var_idx) < ((0 as i32))) as i32)) != 0 { 289 } else { 288 }; continue;
}
// C line 25211
291 => {
let _ = { let assigned = define_var(s, fd, class_name, (((JS_VAR_DEF_CONST as i32)) as JSVarDefEnum)); class_name_var_idx = assigned; assigned };
vm_block = 290; continue;
}
// C line 25210
292 => {
vm_block = if ((((class_name) != ((((0 as i32)) as JSAtom))) as i32)) != 0 { 291 } else { 288 }; continue;
}
// C line 25204
293 => {
vm_block = 5; continue;
}
// C line 25203
294 => {
vm_block = if (js_parse_left_hand_side_expr(s)) != 0 { 293 } else { 292 }; continue;
}
// C line 25202
295 => {
vm_block = 5; continue;
}
// C line 25201
296 => {
vm_block = if (next_token(s)) != 0 { 295 } else { 294 }; continue;
}
// C line 25200
297 => {
let _ = { let assigned = ((1 as i32)).wrapping_shl(((0 as i32)) as u32); class_flags = assigned; assigned };
vm_block = 296; continue;
}
// C line 25206
298 => {
let _ = emit_op(s, (((OP_undefined as i32)) as u8));
vm_block = 292; continue;
}
// C line 25199
299 => {
vm_block = if ((((((*(s)).token).val) == ((TOK_EXTENDS as i32))) as i32)) != 0 { 297 } else { 298 }; continue;
}
// C line 25197
300 => {
let _ = push_scope(s);
vm_block = 299; continue;
}
// C line 25194
301 => {
let _ = { let assigned = JS_DupAtom(ctx, class_var_name); class_var_name = assigned; assigned };
vm_block = 300; continue;
}
// C line 25191
302 => {
let _ = { let assigned = (((crate::quickjs_atom::JS_ATOM__default_ as i32)) as JSAtom); class_var_name = assigned; assigned };
vm_block = 301; continue;
}
// C line 25193
303 => {
let _ = { let assigned = class_name; class_var_name = assigned; assigned };
vm_block = 301; continue;
}
// C line 25190
304 => {
vm_block = if ((((class_name) == ((((0 as i32)) as JSAtom))) as i32)) != 0 { 302 } else { 303 }; continue;
}
// C line 25189
305 => {
vm_block = if ((!((is_class_expr) != 0) as i32)) != 0 { 304 } else { 300 }; continue;
}
// C line 25184
306 => {
vm_block = 5; continue;
}
// C line 25183
307 => {
vm_block = if (next_token(s)) != 0 { 306 } else { 305 }; continue;
}
// C line 25182
308 => {
let _ = { let assigned = JS_DupAtom(ctx, ((((*(s)).token).u).ident).atom); class_name = assigned; assigned };
vm_block = 307; continue;
}
// C line 25180
309 => {
vm_block = 5; continue;
}
// C line 25179
310 => {
let _ = js_parse_error_reserved_identifier(s);
vm_block = 309; continue;
}
// C line 25178
311 => {
vm_block = if (((((*(s)).token).u).ident).is_reserved) != 0 { 310 } else { 308 }; continue;
}
// C line 25187
312 => {
vm_block = 5; continue;
}
// C line 25186
313 => {
let _ = js_parse_error(s, c"class statement requires a name".as_ptr());
vm_block = 312; continue;
}
// C line 25185
314 => {
vm_block = if ((((((!((is_class_expr) != 0) as i32)) != 0) && (((((((export_flag) as u32)) != ((((JS_PARSE_EXPORT_DEFAULT as i32)) as u32))) as i32)) != 0)) as i32)) != 0 { 313 } else { 305 }; continue;
}
// C line 25177
315 => {
vm_block = if ((((((*(s)).token).val) == ((TOK_IDENT as i32))) as i32)) != 0 { 311 } else { 314 }; continue;
}
// C line 25176
316 => {
vm_block = 5; continue;
}
// C line 25175
317 => {
vm_block = if (next_token(s)) != 0 { 316 } else { 315 }; continue;
}
// C line 25174
318 => {
let _ = { (*(fd)).js_mode = (((((((*(fd)).js_mode) as i32)) | (((1 as i32)).wrapping_shl(((0 as i32)) as u32)))) as u8) as u8; (*(fd)).js_mode };
vm_block = 317; continue;
}
// C line 25173
319 => {
let _ = { let assigned = (((*(fd)).js_mode) as i32); saved_js_mode = assigned; assigned };
vm_block = 318; continue;
}
// C line 25168
320 => {
class_start_ptr = ((*(s)).token).ptr;
vm_block = 319; continue;
}
// C line 25166
321 => {
class_flags = (0 as i32);
vm_block = 320; continue;
}
// C line 25163
322 => {
class_var_name = (((0 as i32)) as JSAtom);
vm_block = 321; continue;
}
// C line 25162
323 => {
name = (((0 as i32)) as JSAtom);
class_name = (((0 as i32)) as JSAtom);
vm_block = 322; continue;
}
// C line 25161
324 => {
fd = (*(s)).cur_func;
vm_block = 323; continue;
}
// C line 25160
325 => {
ctx = (*(s)).ctx;
vm_block = 324; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:25669. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_parse_array_literal(mut s: *mut JSParseState) -> i32 {
let mut vm_local_storage = Vec::<u64>::new();
let mut idx: u32 = 0;
let mut need_length: i32 = 0;
let mut vm_block: usize = 65;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 25792 labels: done
1 => {
return js_parse_expect(s, (93 as i32));
}
// C line 25787
2 => {
let _ = emit_atom(s, (((crate::quickjs_atom::JS_ATOM_length as i32)) as JSAtom));
vm_block = 1; continue;
}
// C line 25786
3 => {
let _ = emit_op(s, (((OP_put_field as i32)) as u8));
vm_block = 2; continue;
}
// C line 25785
4 => {
let _ = emit_op(s, (((OP_dup1 as i32)) as u8));
vm_block = 3; continue;
}
// C line 25789
5 => {
let _ = emit_op(s, (((OP_drop as i32)) as u8));
vm_block = 1; continue;
}
// C line 25782
6 => {
vm_block = if (need_length) != 0 { 4 } else { 5 }; continue;
}
// C line 25733
7 => {
vm_block = if ((((((*(s)).token).val) != ((93 as i32))) as i32)) != 0 { 24 } else { 6 }; continue;
}
// C line 25780
8 => {
return ((1 as i32)).wrapping_neg();
}
// C line 25779
9 => {
vm_block = if (next_token(s)) != 0 { 8 } else { 7 }; continue;
}
// C line 25778
10 => {
vm_block = 6; continue;
}
// C line 25777
11 => {
vm_block = if ((((((*(s)).token).val) != ((44 as i32))) as i32)) != 0 { 10 } else { 9 }; continue;
}
// C line 25740
12 => {
let _ = emit_op(s, (((OP_append as i32)) as u8));
vm_block = 11; continue;
}
// C line 25738
13 => {
return ((1 as i32)).wrapping_neg();
}
// C line 25737
14 => {
vm_block = if (js_parse_assign_expr(s)) != 0 { 13 } else { 12 }; continue;
}
// C line 25736
15 => {
return ((1 as i32)).wrapping_neg();
}
// C line 25735
16 => {
vm_block = if (next_token(s)) != 0 { 15 } else { 14 }; continue;
}
// C line 25775
17 => {
let _ = emit_op(s, (((OP_inc as i32)) as u8));
vm_block = 11; continue;
}
// C line 25773
18 => {
let _ = { let assigned = (0 as i32); need_length = assigned; assigned };
vm_block = 17; continue;
}
// C line 25772
19 => {
let _ = emit_op(s, (((OP_define_array_el as i32)) as u8));
vm_block = 18; continue;
}
// C line 25770
20 => {
return ((1 as i32)).wrapping_neg();
}
// C line 25769
21 => {
vm_block = if (js_parse_assign_expr(s)) != 0 { 20 } else { 19 }; continue;
}
// C line 25768
22 => {
vm_block = if ((((((*(s)).token).val) != ((44 as i32))) as i32)) != 0 { 21 } else { 17 }; continue;
}
// C line 25767
23 => {
let _ = { let assigned = (1 as i32); need_length = assigned; assigned };
vm_block = 22; continue;
}
// C line 25734
24 => {
vm_block = if ((((((*(s)).token).val) == ((TOK_ELLIPSIS as i32))) as i32)) != 0 { 16 } else { 23 }; continue;
}
// C line 25730
25 => {
let _ = emit_u32(s, idx);
vm_block = 7; continue;
}
// C line 25729
26 => {
let _ = emit_op(s, (((OP_push_i32 as i32)) as u8));
vm_block = 25; continue;
}
// C line 25725
27 => {
vm_block = 1; continue;
}
// C line 25723
28 => {
let _ = emit_atom(s, (((crate::quickjs_atom::JS_ATOM_length as i32)) as JSAtom));
vm_block = 27; continue;
}
// C line 25722
29 => {
let _ = emit_op(s, (((OP_put_field as i32)) as u8));
vm_block = 28; continue;
}
// C line 25721
30 => {
let _ = emit_u32(s, idx);
vm_block = 29; continue;
}
// C line 25720
31 => {
let _ = emit_op(s, (((OP_push_i32 as i32)) as u8));
vm_block = 30; continue;
}
// C line 25719
32 => {
let _ = emit_op(s, (((OP_dup as i32)) as u8));
vm_block = 31; continue;
}
// C line 25716
33 => {
vm_block = if (need_length) != 0 { 32 } else { 27 }; continue;
}
// C line 25715
34 => {
vm_block = if ((((((*(s)).token).val) == ((93 as i32))) as i32)) != 0 { 33 } else { 26 }; continue;
}
// C line 25697
35 => {
vm_block = if ((((((((((*(s)).token).val) != ((93 as i32))) as i32)) != 0) && (((((idx) < ((((2147483647 as i32)) as u32))) as i32)) != 0)) as i32)) != 0 { 48 } else { 34 }; continue;
}
// C line 25712
36 => {
return ((1 as i32)).wrapping_neg();
}
// C line 25711
37 => {
vm_block = if (next_token(s)) != 0 { 36 } else { 35 }; continue;
}
// C line 25710
38 => {
vm_block = if ((((((*(s)).token).val) == ((44 as i32))) as i32)) != 0 { 37 } else { 35 }; continue;
}
// C line 25708
39 => {
let _ = { let old = idx; idx = (idx).wrapping_add(1); old };
vm_block = 38; continue;
}
// C line 25706
40 => {
let _ = { let assigned = (0 as i32); need_length = assigned; assigned };
vm_block = 39; continue;
}
// C line 25705
41 => {
let _ = emit_u32(s, __JS_AtomFromUInt32(idx));
vm_block = 40; continue;
}
// C line 25704
42 => {
let _ = emit_op(s, (((OP_define_field as i32)) as u8));
vm_block = 41; continue;
}
// C line 25703
43 => {
return ((1 as i32)).wrapping_neg();
}
// C line 25702
44 => {
vm_block = if (js_parse_assign_expr(s)) != 0 { 43 } else { 42 }; continue;
}
// C line 25701
45 => {
vm_block = if ((((((*(s)).token).val) != ((44 as i32))) as i32)) != 0 { 44 } else { 39 }; continue;
}
// C line 25700
46 => {
let _ = { let assigned = (1 as i32); need_length = assigned; assigned };
vm_block = 45; continue;
}
// C line 25699
47 => {
vm_block = 34; continue;
}
// C line 25698
48 => {
vm_block = if ((((((*(s)).token).val) == ((TOK_ELLIPSIS as i32))) as i32)) != 0 { 47 } else { 46 }; continue;
}
// C line 25696
49 => {
let _ = { let assigned = (0 as i32); need_length = assigned; assigned };
vm_block = 35; continue;
}
// C line 25693
50 => {
let _ = emit_u16(s, ((idx) as u16));
vm_block = 49; continue;
}
// C line 25692
51 => {
let _ = emit_op(s, (((OP_array_from as i32)) as u8));
vm_block = 50; continue;
}
// C line 25678
52 => {
vm_block = if ((((((((((*(s)).token).val) != ((93 as i32))) as i32)) != 0) && (((((idx) < ((((32 as i32)) as u32))) as i32)) != 0)) as i32)) != 0 { 62 } else { 51 }; continue;
}
// C line 25687
53 => {
return ((1 as i32)).wrapping_neg();
}
// C line 25686
54 => {
vm_block = if (next_token(s)) != 0 { 53 } else { 52 }; continue;
}
// C line 25690
55 => {
vm_block = 1; continue;
}
// C line 25689
56 => {
vm_block = if ((((((*(s)).token).val) != ((93 as i32))) as i32)) != 0 { 55 } else { 52 }; continue;
}
// C line 25685
57 => {
vm_block = if ((((((*(s)).token).val) == ((44 as i32))) as i32)) != 0 { 54 } else { 56 }; continue;
}
// C line 25683
58 => {
let _ = { let old = idx; idx = (idx).wrapping_add(1); old };
vm_block = 57; continue;
}
// C line 25682
59 => {
return ((1 as i32)).wrapping_neg();
}
// C line 25681
60 => {
vm_block = if (js_parse_assign_expr(s)) != 0 { 59 } else { 58 }; continue;
}
// C line 25680
61 => {
vm_block = 51; continue;
}
// C line 25679
62 => {
vm_block = if ((((((((((*(s)).token).val) == ((44 as i32))) as i32)) != 0) || (((((((*(s)).token).val) == ((TOK_ELLIPSIS as i32))) as i32)) != 0)) as i32)) != 0 { 61 } else { 60 }; continue;
}
// C line 25677
63 => {
let _ = { let assigned = (((0 as i32)) as u32); idx = assigned; assigned };
vm_block = 52; continue;
}
// C line 25675
64 => {
return ((1 as i32)).wrapping_neg();
}
// C line 25674
65 => {
vm_block = if (next_token(s)) != 0 { 64 } else { 63 }; continue;
}
_ => std::process::abort(),
} }
}
