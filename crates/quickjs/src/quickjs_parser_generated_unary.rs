// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:27383. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_parse_delete(mut s: *mut JSParseState) -> i32 {
let mut vm_local_storage = Vec::<u64>::new();
let mut fd: *mut JSFunctionDef = core::ptr::null_mut();
let mut name: JSAtom = 0;
let mut opcode: i32 = 0;
let mut val: JSValue = core::mem::zeroed();
let mut ret: i32 = 0;
let mut opt_chain_label: i32 = 0;
let mut next_label: i32 = 0;
let mut opt_chain_label_1: i32 = 0;
let mut next_label_1: i32 = 0;
let mut vm_block: usize = 58;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 27472
1 => {
return (0 as i32);
}
// C line 27470
2 => {
vm_block = 1; continue;
}
// C line 27469
3 => {
let _ = emit_op(s, (((OP_push_true as i32)) as u8));
vm_block = 2; continue;
}
// C line 27468 labels: ret_true
4 => {
let _ = emit_op(s, (((OP_drop as i32)) as u8));
vm_block = 3; continue;
}
// C line 27465
5 => {
vm_block = 1; continue;
}
// C line 27464
6 => {
let _ = emit_u8(s, (((3 as i32)) as u8));
vm_block = 5; continue;
}
// C line 27463
7 => {
let _ = emit_atom(s, (((0 as i32)) as JSAtom));
vm_block = 6; continue;
}
// C line 27462
8 => {
let _ = emit_op(s, (((OP_throw_error as i32)) as u8));
vm_block = 7; continue;
}
// C line 27461
9 => {
let _ = { let assigned = ((1 as i32)).wrapping_neg(); (*(fd)).last_opcode_pos = assigned; assigned };
vm_block = 8; continue;
}
// C line 27460
10 => {
let _ = { let assigned = (((*(fd)).last_opcode_pos) as usize); ((*(fd)).byte_code).size = assigned; assigned };
vm_block = 9; continue;
}
// C line 27458
11 => {
return js_parse_error(s, c"cannot delete a private class field".as_ptr());
}
// C line 27456
12 => {
vm_block = 1; continue;
}
// C line 27452
13 => {
return js_parse_error(s, c"cannot delete a direct reference in strict mode".as_ptr());
}
// C line 27454
14 => {
let _ = { let assigned = (((OP_scope_delete_var as i32)) as u8); *(((*(fd)).byte_code).buf).offset(((*(fd)).last_opcode_pos) as isize) = assigned; assigned };
vm_block = 12; continue;
}
// C line 27451
15 => {
vm_block = if ((((((*(fd)).js_mode) as i32)) & (((1 as i32)).wrapping_shl(((0 as i32)) as u32)))) != 0 { 13 } else { 14 }; continue;
}
// C line 27450
16 => {
vm_block = 4; continue;
}
// C line 27449
17 => {
vm_block = if ((((((((name) == ((((crate::quickjs_atom::JS_ATOM_this as i32)) as JSAtom))) as i32)) != 0) || (((((name) == ((((crate::quickjs_atom::JS_ATOM_new_target as i32)) as JSAtom))) as i32)) != 0)) as i32)) != 0 { 16 } else { 15 }; continue;
}
// C line 27448
18 => {
let _ = { let assigned = get_u32(((((*(fd)).byte_code).buf).offset((((*(fd)).last_opcode_pos) as isize))).offset((((1 as i32)) as isize))); name = assigned; assigned };
vm_block = 17; continue;
}
// C line 27445
19 => {
vm_block = 1; continue;
}
// C line 27443
20 => {
let _ = { let assigned = ((1 as i32)).wrapping_neg(); (*(fd)).last_opcode_pos = assigned; assigned };
vm_block = 19; continue;
}
// C line 27442
21 => {
let _ = emit_label(s, next_label_1);
vm_block = 20; continue;
}
// C line 27441
22 => {
let _ = emit_op(s, (((OP_push_true as i32)) as u8));
vm_block = 21; continue;
}
// C line 27440
23 => {
let _ = emit_op(s, (((OP_drop as i32)) as u8));
vm_block = 22; continue;
}
// C line 27438
24 => {
let _ = emit_label(s, opt_chain_label_1);
vm_block = 23; continue;
}
// C line 27437
25 => {
let _ = { let assigned = emit_goto(s, (OP_goto as i32), ((1 as i32)).wrapping_neg()); next_label_1 = assigned; assigned };
vm_block = 24; continue;
}
// C line 27436
26 => {
let _ = emit_op(s, (((OP_delete as i32)) as u8));
vm_block = 25; continue;
}
// C line 27435
27 => {
let _ = { let assigned = (((*(fd)).last_opcode_pos) as usize); ((*(fd)).byte_code).size = assigned; assigned };
vm_block = 26; continue;
}
// C line 27433
28 => {
let _ = { let assigned = ((get_u32((((((*(fd)).byte_code).buf).offset((((*(fd)).last_opcode_pos) as isize))).offset((((1 as i32)) as isize))).offset((((1 as i32)) as isize)))) as i32); opt_chain_label_1 = assigned; assigned };
vm_block = 27; continue;
}
// C line 27429
29 => {
vm_block = 1; continue;
}
// C line 27428
30 => {
let _ = emit_op(s, (((OP_delete as i32)) as u8));
vm_block = 29; continue;
}
// C line 27427
31 => {
let _ = { let assigned = ((1 as i32)).wrapping_neg(); (*(fd)).last_opcode_pos = assigned; assigned };
vm_block = 30; continue;
}
// C line 27426
32 => {
let _ = { let assigned = (((*(fd)).last_opcode_pos) as usize); ((*(fd)).byte_code).size = assigned; assigned };
vm_block = 31; continue;
}
// C line 27424
33 => {
vm_block = 1; continue;
}
// C line 27422
34 => {
let _ = { let assigned = ((1 as i32)).wrapping_neg(); (*(fd)).last_opcode_pos = assigned; assigned };
vm_block = 33; continue;
}
// C line 27420
35 => {
let _ = emit_label(s, next_label);
vm_block = 34; continue;
}
// C line 27419
36 => {
let _ = emit_op(s, (((OP_push_true as i32)) as u8));
vm_block = 35; continue;
}
// C line 27418
37 => {
let _ = emit_op(s, (((OP_drop as i32)) as u8));
vm_block = 36; continue;
}
// C line 27416
38 => {
let _ = emit_label(s, opt_chain_label);
vm_block = 37; continue;
}
// C line 27415
39 => {
let _ = { let assigned = emit_goto(s, (OP_goto as i32), ((1 as i32)).wrapping_neg()); next_label = assigned; assigned };
vm_block = 38; continue;
}
// C line 27414
40 => {
vm_block = if ((((opt_chain_label) >= ((0 as i32))) as i32)) != 0 { 39 } else { 34 }; continue;
}
// C line 27413
41 => {
let _ = emit_op(s, (((OP_delete as i32)) as u8));
vm_block = 40; continue;
}
// C line 27412
42 => {
return ret;
}
// C line 27411
43 => {
vm_block = if (ret) != 0 { 42 } else { 41 }; continue;
}
// C line 27410
44 => {
let _ = JS_FreeAtom((*(s)).ctx, name);
vm_block = 43; continue;
}
// C line 27409
45 => {
let _ = JS_FreeValue((*(s)).ctx, val);
vm_block = 44; continue;
}
// C line 27408
46 => {
let _ = { let assigned = emit_push_const(s, val, (1 as i32)); ret = assigned; assigned };
vm_block = 45; continue;
}
// C line 27407
47 => {
let _ = { let assigned = JS_AtomToValue((*(s)).ctx, name); val = assigned; assigned };
vm_block = 46; continue;
}
// C line 27406
48 => {
let _ = { let assigned = (((*(fd)).last_opcode_pos) as usize); ((*(fd)).byte_code).size = assigned; assigned };
vm_block = 47; continue;
}
// C line 27405
49 => {
let _ = { let assigned = get_u32(((((*(fd)).byte_code).buf).offset((((*(fd)).last_opcode_pos) as isize))).offset((((1 as i32)) as isize))); name = assigned; assigned };
vm_block = 48; continue;
}
// C line 27400
50 => {
let _ = { let assigned = ((get_u32(((((((*(fd)).byte_code).buf).offset((((*(fd)).last_opcode_pos) as isize))).offset((((1 as i32)) as isize))).offset((((4 as i32)) as isize))).offset((((1 as i32)) as isize)))) as i32); opt_chain_label = assigned; assigned };
vm_block = 49; continue;
}
// C line 27403
51 => {
let _ = { let assigned = ((1 as i32)).wrapping_neg(); opt_chain_label = assigned; assigned };
vm_block = 49; continue;
}
// C line 27399
52 => {
vm_block = if ((((opcode) == ((OP_get_field_opt_chain as i32))) as i32)) != 0 { 50 } else { 51 }; continue;
}
// C line 27393
53 => {
vm_block = match { let assigned = get_prev_opcode(fd); opcode = assigned; assigned } { x if x == (OP_get_super_value as i32) => 10, x if x == (OP_scope_get_private_field as i32) => 11, x if x == (OP_scope_get_var as i32) => 18, x if x == (OP_get_array_el_opt_chain as i32) => 28, x if x == (OP_get_array_el as i32) => 32, x if x == (OP_get_field_opt_chain as i32) => 52, x if x == (OP_get_field as i32) => 52, _ => 4, }; continue;
}
// C line 27392
54 => {
return ((1 as i32)).wrapping_neg();
}
// C line 27391
55 => {
vm_block = if (js_parse_unary(s, ((1 as i32)).wrapping_shl(((3 as i32)) as u32))) != 0 { 54 } else { 53 }; continue;
}
// C line 27390
56 => {
return ((1 as i32)).wrapping_neg();
}
// C line 27389
57 => {
vm_block = if (next_token(s)) != 0 { 56 } else { 55 }; continue;
}
// C line 27385
58 => {
fd = (*(s)).cur_func;
vm_block = 57; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:27476. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_parse_unary(mut s: *mut JSParseState, mut parse_flags: i32) -> i32 {
let mut vm_local_storage = Vec::<u64>::new();
let mut op: i32 = 0;
let mut op_token_ptr: *const u8 = core::ptr::null();
let mut opcode: i32 = 0;
let mut op_1: i32 = 0;
let mut scope: i32 = 0;
let mut label: i32 = 0;
let mut name: JSAtom = 0;
let mut fd: *mut JSFunctionDef = core::ptr::null_mut();
let mut opcode_1: i32 = 0;
let mut op_2: i32 = 0;
let mut scope_1: i32 = 0;
let mut label_1: i32 = 0;
let mut name_1: JSAtom = 0;
let mut vm_block: usize = 89;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 27612
1 => {
return (0 as i32);
}
// C line 27609
2 => {
let _ = emit_op(s, (((OP_pow as i32)) as u8));
vm_block = 1; continue;
}
// C line 27608
3 => {
let _ = emit_source_pos(s, op_token_ptr);
vm_block = 2; continue;
}
// C line 27607
4 => {
return ((1 as i32)).wrapping_neg();
}
// C line 27606
5 => {
vm_block = if (js_parse_unary(s, ((1 as i32)).wrapping_shl(((2 as i32)) as u32))) != 0 { 4 } else { 3 }; continue;
}
// C line 27605
6 => {
return ((1 as i32)).wrapping_neg();
}
// C line 27604
7 => {
vm_block = if (next_token(s)) != 0 { 6 } else { 5 }; continue;
}
// C line 27603
8 => {
let _ = { let assigned = ((*(s)).token).ptr; op_token_ptr = assigned; assigned };
vm_block = 7; continue;
}
// C line 27601
9 => {
return ((1 as i32)).wrapping_neg();
}
// C line 27600
10 => {
let _ = JS_ThrowSyntaxError((*(s)).ctx, c"unparenthesized unary expression can't appear on the left-hand side of '**'".as_ptr());
vm_block = 9; continue;
}
// C line 27599
11 => {
vm_block = if (((parse_flags) & (((1 as i32)).wrapping_shl(((3 as i32)) as u32)))) != 0 { 10 } else { 8 }; continue;
}
// C line 27593
12 => {
vm_block = if ((((((*(s)).token).val) == ((TOK_POW as i32))) as i32)) != 0 { 11 } else { 1 }; continue;
}
// C line 27592
13 => {
vm_block = if (((parse_flags) & (((((1 as i32)).wrapping_shl(((2 as i32)) as u32)) | (((1 as i32)).wrapping_shl(((3 as i32)) as u32)))))) != 0 { 12 } else { 1 }; continue;
}
// C line 27590
14 => {
vm_block = 13; continue;
}
// C line 27588
15 => {
return ((1 as i32)).wrapping_neg();
}
// C line 27587
16 => {
vm_block = if (next_token(s)) != 0 { 15 } else { 14 }; continue;
}
// C line 27585
17 => {
let _ = put_lvalue(s, opcode_1, scope_1, name_1, label_1, (((PUT_LVALUE_KEEP_SECOND as i32)) as PutLValueEnum), (0 as i32));
vm_block = 16; continue;
}
// C line 27584
18 => {
let _ = emit_op(s, (((((OP_post_dec as i32)).wrapping_add(op_2)).wrapping_sub((TOK_DEC as i32))) as u8));
vm_block = 17; continue;
}
// C line 27583
19 => {
let _ = emit_source_pos(s, op_token_ptr);
vm_block = 18; continue;
}
// C line 27582
20 => {
return ((1 as i32)).wrapping_neg();
}
// C line 27581
21 => {
vm_block = if (get_lvalue(s, core::ptr::addr_of_mut!(opcode_1), core::ptr::addr_of_mut!(scope_1), core::ptr::addr_of_mut!(name_1), core::ptr::addr_of_mut!(label_1), core::ptr::null_mut::<i32>(), (1 as i32), op_2)) != 0 { 20 } else { 19 }; continue;
}
// C line 27580
22 => {
let _ = { let assigned = ((*(s)).token).ptr; op_token_ptr = assigned; assigned };
vm_block = 21; continue;
}
// C line 27579
23 => {
let _ = { let assigned = ((*(s)).token).val; op_2 = assigned; assigned };
vm_block = 22; continue;
}
// C line 27575
24 => {
vm_block = if ((((((!(((*(s)).got_lf) != 0) as i32)) != 0) && (((((((((((*(s)).token).val) == ((TOK_DEC as i32))) as i32)) != 0) || (((((((*(s)).token).val) == ((TOK_INC as i32))) as i32)) != 0)) as i32)) != 0)) as i32)) != 0 { 23 } else { 14 }; continue;
}
// C line 27574
25 => {
return ((1 as i32)).wrapping_neg();
}
// C line 27573
26 => {
vm_block = if (js_parse_postfix_expr(s, ((1 as i32)).wrapping_shl(((1 as i32)) as u32))) != 0 { 25 } else { 24 }; continue;
}
// C line 27571
27 => {
vm_block = 13; continue;
}
// C line 27570
28 => {
let _ = { let assigned = (0 as i32); parse_flags = assigned; assigned };
vm_block = 27; continue;
}
// C line 27569
29 => {
let _ = emit_op(s, (((OP_await as i32)) as u8));
vm_block = 28; continue;
}
// C line 27568
30 => {
let _ = { let assigned = (1 as i32); (*((*(s)).cur_func)).has_await = assigned; assigned };
vm_block = 29; continue;
}
// C line 27567
31 => {
return ((1 as i32)).wrapping_neg();
}
// C line 27566
32 => {
vm_block = if (js_parse_unary(s, ((1 as i32)).wrapping_shl(((3 as i32)) as u32))) != 0 { 31 } else { 30 }; continue;
}
// C line 27565
33 => {
return ((1 as i32)).wrapping_neg();
}
// C line 27564
34 => {
vm_block = if (next_token(s)) != 0 { 33 } else { 32 }; continue;
}
// C line 27563
35 => {
return js_parse_error(s, c"await in default expression".as_ptr());
}
// C line 27562
36 => {
vm_block = if ((!(((*((*(s)).cur_func)).in_function_body) != 0) as i32)) != 0 { 35 } else { 34 }; continue;
}
// C line 27561
37 => {
return js_parse_error(s, c"unexpected 'await' keyword".as_ptr());
}
// C line 27560
38 => {
vm_block = if ((!((((((((*((*(s)).cur_func)).func_kind as JSFunctionKindEnum)) as i32)) & ((JS_FUNC_ASYNC as i32)))) != 0) as i32)) != 0 { 37 } else { 36 }; continue;
}
// C line 27558
39 => {
vm_block = 13; continue;
}
// C line 27557
40 => {
let _ = { let assigned = (0 as i32); parse_flags = assigned; assigned };
vm_block = 39; continue;
}
// C line 27556
41 => {
return ((1 as i32)).wrapping_neg();
}
// C line 27555
42 => {
vm_block = if (js_parse_delete(s)) != 0 { 41 } else { 40 }; continue;
}
// C line 27553
43 => {
vm_block = 13; continue;
}
// C line 27551
44 => {
let _ = { let assigned = (0 as i32); parse_flags = assigned; assigned };
vm_block = 43; continue;
}
// C line 27550
45 => {
let _ = emit_op(s, (((OP_typeof as i32)) as u8));
vm_block = 44; continue;
}
// C line 27548
46 => {
let _ = { let assigned = (((OP_scope_get_var_undef as i32)) as u8); *(((*(fd)).byte_code).buf).offset(((*(fd)).last_opcode_pos) as isize) = assigned; assigned };
vm_block = 45; continue;
}
// C line 27547
47 => {
vm_block = if ((((get_prev_opcode(fd)) == ((OP_scope_get_var as i32))) as i32)) != 0 { 46 } else { 45 }; continue;
}
// C line 27546
48 => {
let _ = { let assigned = (*(s)).cur_func; fd = assigned; assigned };
vm_block = 47; continue;
}
// C line 27543
49 => {
return ((1 as i32)).wrapping_neg();
}
// C line 27542
50 => {
vm_block = if (js_parse_unary(s, ((1 as i32)).wrapping_shl(((3 as i32)) as u32))) != 0 { 49 } else { 48 }; continue;
}
// C line 27541
51 => {
return ((1 as i32)).wrapping_neg();
}
// C line 27540
52 => {
vm_block = if (next_token(s)) != 0 { 51 } else { 50 }; continue;
}
// C line 27536
53 => {
vm_block = 13; continue;
}
// C line 27533
54 => {
let _ = put_lvalue(s, opcode, scope, name, label, (((PUT_LVALUE_KEEP_TOP as i32)) as PutLValueEnum), (0 as i32));
vm_block = 53; continue;
}
// C line 27532
55 => {
let _ = emit_op(s, (((((OP_dec as i32)).wrapping_add(op_1)).wrapping_sub((TOK_DEC as i32))) as u8));
vm_block = 54; continue;
}
// C line 27531
56 => {
let _ = emit_source_pos(s, op_token_ptr);
vm_block = 55; continue;
}
// C line 27530
57 => {
return ((1 as i32)).wrapping_neg();
}
// C line 27529
58 => {
vm_block = if (get_lvalue(s, core::ptr::addr_of_mut!(opcode), core::ptr::addr_of_mut!(scope), core::ptr::addr_of_mut!(name), core::ptr::addr_of_mut!(label), core::ptr::null_mut::<i32>(), (1 as i32), op_1)) != 0 { 57 } else { 56 }; continue;
}
// C line 27528
59 => {
return ((1 as i32)).wrapping_neg();
}
// C line 27527
60 => {
vm_block = if (js_parse_unary(s, (0 as i32))) != 0 { 59 } else { 58 }; continue;
}
// C line 27526
61 => {
return ((1 as i32)).wrapping_neg();
}
// C line 27525
62 => {
vm_block = if (next_token(s)) != 0 { 61 } else { 60 }; continue;
}
// C line 27524
63 => {
let _ = { let assigned = ((*(s)).token).ptr; op_token_ptr = assigned; assigned };
vm_block = 62; continue;
}
// C line 27523
64 => {
let _ = { let assigned = ((*(s)).token).val; op_1 = assigned; assigned };
vm_block = 63; continue;
}
// C line 27517
65 => {
vm_block = 13; continue;
}
// C line 27516
66 => {
let _ = { let assigned = (0 as i32); parse_flags = assigned; assigned };
vm_block = 65; continue;
}
// C line 27514
67 => {
let _ = std::process::abort();
vm_block = 66; continue;
}
// C line 27512
68 => {
vm_block = 66; continue;
}
// C line 27511
69 => {
let _ = emit_op(s, (((OP_undefined as i32)) as u8));
vm_block = 68; continue;
}
// C line 27510
70 => {
let _ = emit_op(s, (((OP_drop as i32)) as u8));
vm_block = 69; continue;
}
// C line 27508
71 => {
vm_block = 66; continue;
}
// C line 27507
72 => {
let _ = emit_op(s, (((OP_not as i32)) as u8));
vm_block = 71; continue;
}
// C line 27506
73 => {
let _ = emit_source_pos(s, op_token_ptr);
vm_block = 72; continue;
}
// C line 27504
74 => {
vm_block = 66; continue;
}
// C line 27503
75 => {
let _ = emit_op(s, (((OP_lnot as i32)) as u8));
vm_block = 74; continue;
}
// C line 27501
76 => {
vm_block = 66; continue;
}
// C line 27500
77 => {
let _ = emit_op(s, (((OP_plus as i32)) as u8));
vm_block = 76; continue;
}
// C line 27499
78 => {
let _ = emit_source_pos(s, op_token_ptr);
vm_block = 77; continue;
}
// C line 27497
79 => {
vm_block = 66; continue;
}
// C line 27496
80 => {
let _ = emit_op(s, (((OP_neg as i32)) as u8));
vm_block = 79; continue;
}
// C line 27495
81 => {
let _ = emit_source_pos(s, op_token_ptr);
vm_block = 80; continue;
}
// C line 27493
82 => {
vm_block = match op { x if x == (TOK_VOID as i32) => 70, x if x == (126 as i32) => 73, x if x == (33 as i32) => 75, x if x == (43 as i32) => 78, x if x == (45 as i32) => 81, _ => 67, }; continue;
}
// C line 27492
83 => {
return ((1 as i32)).wrapping_neg();
}
// C line 27491
84 => {
vm_block = if (js_parse_unary(s, ((1 as i32)).wrapping_shl(((3 as i32)) as u32))) != 0 { 83 } else { 82 }; continue;
}
// C line 27490
85 => {
return ((1 as i32)).wrapping_neg();
}
// C line 27489
86 => {
vm_block = if (next_token(s)) != 0 { 85 } else { 84 }; continue;
}
// C line 27488
87 => {
let _ = { let assigned = ((*(s)).token).val; op = assigned; assigned };
vm_block = 86; continue;
}
// C line 27487
88 => {
let _ = { let assigned = ((*(s)).token).ptr; op_token_ptr = assigned; assigned };
vm_block = 87; continue;
}
// C line 27481
89 => {
vm_block = match ((*(s)).token).val { x if x == (TOK_AWAIT as i32) => 38, x if x == (TOK_DELETE as i32) => 42, x if x == (TOK_TYPEOF as i32) => 52, x if x == (TOK_INC as i32) => 64, x if x == (TOK_DEC as i32) => 64, x if x == (TOK_VOID as i32) => 88, x if x == (126 as i32) => 88, x if x == (33 as i32) => 88, x if x == (45 as i32) => 88, x if x == (43 as i32) => 88, _ => 26, }; continue;
}
_ => std::process::abort(),
} }
}
