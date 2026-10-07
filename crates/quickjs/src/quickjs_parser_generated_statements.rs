// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:28373. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_parse_statement(mut s: *mut JSParseState) -> i32 {
let mut vm_local_storage = Vec::<u64>::new();
let mut vm_block: usize = 1;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 28375
1 => {
return js_parse_statement_or_decl(s, (0 as i32));
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:28378. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_parse_block(mut s: *mut JSParseState) -> i32 {
let mut vm_local_storage = Vec::<u64>::new();
let mut vm_block: usize = 13;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 28394
1 => {
return (0 as i32);
}
// C line 28393
2 => {
return ((1 as i32)).wrapping_neg();
}
// C line 28392
3 => {
vm_block = if (next_token(s)) != 0 { 2 } else { 1 }; continue;
}
// C line 28390
4 => {
let _ = pop_scope(s);
vm_block = 3; continue;
}
// C line 28384
5 => {
vm_block = 9; continue;
}
// C line 28388
6 => {
vm_block = 4; continue;
}
// C line 28387
7 => {
vm_block = if ((((((*(s)).token).val) == ((125 as i32))) as i32)) != 0 { 6 } else { 5 }; continue;
}
// C line 28386
8 => {
return ((1 as i32)).wrapping_neg();
}
// C line 28385
9 => {
vm_block = if (js_parse_statement_or_decl(s, ((((((1 as i32)).wrapping_shl(((0 as i32)) as u32)) | (((1 as i32)).wrapping_shl(((1 as i32)) as u32)))) | (((1 as i32)).wrapping_shl(((2 as i32)) as u32))))) != 0 { 8 } else { 7 }; continue;
}
// C line 28383
10 => {
let _ = push_scope(s);
vm_block = 5; continue;
}
// C line 28382
11 => {
vm_block = if ((((((*(s)).token).val) != ((125 as i32))) as i32)) != 0 { 10 } else { 3 }; continue;
}
// C line 28381
12 => {
return ((1 as i32)).wrapping_neg();
}
// C line 28380
13 => {
vm_block = if (js_parse_expect(s, (123 as i32))) != 0 { 12 } else { 11 }; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:28398. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_parse_var(mut s: *mut JSParseState, mut parse_flags: i32, mut tok: i32, mut export_flag: i32) -> i32 {
let mut vm_local_storage = Vec::<u64>::new();
let mut ctx: *mut JSContext = core::ptr::null_mut();
let mut fd: *mut JSFunctionDef = core::ptr::null_mut();
let mut name: JSAtom = 0;
let mut source_ptr: *const u8 = core::ptr::null();
let mut opcode: i32 = 0;
let mut scope: i32 = 0;
let mut label: i32 = 0;
let mut name1: JSAtom = 0;
let mut skip_bits: i32 = 0;
let mut vm_block: usize = 62;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 28491
1 => {
return ((1 as i32)).wrapping_neg();
}
// C line 28490 labels: var_error
2 => {
let _ = JS_FreeAtom(ctx, name);
vm_block = 1; continue;
}
// C line 28487
3 => {
return (0 as i32);
}
// C line 28405
4 => {
vm_block = 59; continue;
}
// C line 28485
5 => {
return ((1 as i32)).wrapping_neg();
}
// C line 28484
6 => {
vm_block = if (next_token(s)) != 0 { 5 } else { 4 }; continue;
}
// C line 28483
7 => {
vm_block = 3; continue;
}
// C line 28482
8 => {
vm_block = if ((((((*(s)).token).val) != ((44 as i32))) as i32)) != 0 { 7 } else { 6 }; continue;
}
// C line 28470
9 => {
let _ = JS_FreeAtom(ctx, name);
vm_block = 8; continue;
}
// C line 28445
10 => {
let _ = put_lvalue(s, opcode, scope, name1, label, (((PUT_LVALUE_NOKEEP as i32)) as PutLValueEnum), (0 as i32));
vm_block = 9; continue;
}
// C line 28444
11 => {
let _ = emit_source_pos(s, source_ptr);
vm_block = 10; continue;
}
// C line 28443
12 => {
let _ = set_object_name(s, name);
vm_block = 11; continue;
}
// C line 28441
13 => {
vm_block = 2; continue;
}
// C line 28440
14 => {
let _ = JS_FreeAtom(ctx, name1);
vm_block = 13; continue;
}
// C line 28439
15 => {
vm_block = if (js_parse_assign_expr2(s, parse_flags)) != 0 { 14 } else { 12 }; continue;
}
// C line 28438
16 => {
vm_block = 2; continue;
}
// C line 28437
17 => {
vm_block = if ((((get_lvalue(s, core::ptr::addr_of_mut!(opcode), core::ptr::addr_of_mut!(scope), core::ptr::addr_of_mut!(name1), core::ptr::addr_of_mut!(label), core::ptr::null_mut::<i32>(), (0 as i32), (61 as i32))) < ((0 as i32))) as i32)) != 0 { 16 } else { 15 }; continue;
}
// C line 28436
18 => {
let _ = emit_u16(s, (((*(fd)).scope_level) as u16));
vm_block = 17; continue;
}
// C line 28435
19 => {
let _ = emit_atom(s, name);
vm_block = 18; continue;
}
// C line 28434
20 => {
let _ = emit_op(s, (((OP_scope_get_var as i32)) as u8));
vm_block = 19; continue;
}
// C line 28455
21 => {
let _ = emit_u16(s, (((*(fd)).scope_level) as u16));
vm_block = 9; continue;
}
// C line 28454
22 => {
let _ = emit_atom(s, name);
vm_block = 21; continue;
}
// C line 28452
23 => {
let _ = emit_op(s, ((if ((((((((tok) == ((TOK_CONST as i32))) as i32)) != 0) || (((((tok) == ((TOK_LET as i32))) as i32)) != 0)) as i32)) != 0 { (OP_scope_put_var_init as i32) } else { (OP_scope_put_var as i32) }) as u8));
vm_block = 22; continue;
}
// C line 28451
24 => {
let _ = emit_source_pos(s, source_ptr);
vm_block = 23; continue;
}
// C line 28450
25 => {
let _ = set_object_name(s, name);
vm_block = 24; continue;
}
// C line 28449
26 => {
vm_block = 2; continue;
}
// C line 28448
27 => {
vm_block = if (js_parse_assign_expr2(s, parse_flags)) != 0 { 26 } else { 25 }; continue;
}
// C line 28429
28 => {
vm_block = if (need_var_reference(s, tok)) != 0 { 20 } else { 27 }; continue;
}
// C line 28428
29 => {
vm_block = 2; continue;
}
// C line 28427
30 => {
vm_block = if (next_token(s)) != 0 { 29 } else { 28 }; continue;
}
// C line 28426
31 => {
source_ptr = ((*(s)).token).ptr;
vm_block = 30; continue;
}
// C line 28467
32 => {
let _ = emit_u16(s, (((*(fd)).scope_level) as u16));
vm_block = 9; continue;
}
// C line 28466
33 => {
let _ = emit_atom(s, name);
vm_block = 32; continue;
}
// C line 28465
34 => {
let _ = emit_op(s, (((OP_scope_put_var_init as i32)) as u8));
vm_block = 33; continue;
}
// C line 28464
35 => {
let _ = emit_op(s, (((OP_undefined as i32)) as u8));
vm_block = 34; continue;
}
// C line 28462
36 => {
vm_block = if ((((tok) == ((TOK_LET as i32))) as i32)) != 0 { 35 } else { 9 }; continue;
}
// C line 28460
37 => {
vm_block = 2; continue;
}
// C line 28459
38 => {
let _ = js_parse_error(s, c"missing initializer for const variable".as_ptr());
vm_block = 37; continue;
}
// C line 28458
39 => {
vm_block = if ((((tok) == ((TOK_CONST as i32))) as i32)) != 0 { 38 } else { 36 }; continue;
}
// C line 28425
40 => {
vm_block = if ((((((*(s)).token).val) == ((61 as i32))) as i32)) != 0 { 31 } else { 39 }; continue;
}
// C line 28422
41 => {
vm_block = 2; continue;
}
// C line 28420
42 => {
vm_block = if ((!(!(add_export_entry(s, (*((*(s)).cur_func)).module, name, name, (((JS_EXPORT_TYPE_LOCAL as i32)) as JSExportTypeEnum))).is_null()) as i32)) != 0 { 41 } else { 40 }; continue;
}
// C line 28419
43 => {
vm_block = if (export_flag) != 0 { 42 } else { 40 }; continue;
}
// C line 28418
44 => {
vm_block = 2; continue;
}
// C line 28417
45 => {
vm_block = if (js_define_var(s, name, tok)) != 0 { 44 } else { 43 }; continue;
}
// C line 28416
46 => {
vm_block = 2; continue;
}
// C line 28415
47 => {
vm_block = if (next_token(s)) != 0 { 46 } else { 45 }; continue;
}
// C line 28413
48 => {
vm_block = 2; continue;
}
// C line 28412
49 => {
let _ = js_parse_error(s, c"'let' is not a valid lexical identifier".as_ptr());
vm_block = 48; continue;
}
// C line 28411
50 => {
vm_block = if ((((((((name) == ((((crate::quickjs_atom::JS_ATOM_let as i32)) as JSAtom))) as i32)) != 0) && (((((((((tok) == ((TOK_LET as i32))) as i32)) != 0) || (((((tok) == ((TOK_CONST as i32))) as i32)) != 0)) as i32)) != 0)) as i32)) != 0 { 49 } else { 47 }; continue;
}
// C line 28410
51 => {
let _ = { let assigned = JS_DupAtom(ctx, ((((*(s)).token).u).ident).atom); name = assigned; assigned };
vm_block = 50; continue;
}
// C line 28408
52 => {
return js_parse_error_reserved_identifier(s);
}
// C line 28407
53 => {
vm_block = if (((((*(s)).token).u).ident).is_reserved) != 0 { 52 } else { 51 }; continue;
}
// C line 28477
54 => {
return ((1 as i32)).wrapping_neg();
}
// C line 28476
55 => {
vm_block = if ((((js_parse_destructuring_element(s, tok, (0 as i32), (1 as i32), ((skip_bits) & (((1 as i32)).wrapping_shl(((1 as i32)) as u32))), (1 as i32), export_flag)) < ((0 as i32))) as i32)) != 0 { 54 } else { 8 }; continue;
}
// C line 28475
56 => {
let _ = emit_op(s, (((OP_undefined as i32)) as u8));
vm_block = 55; continue;
}
// C line 28479
57 => {
return js_parse_error(s, c"variable name expected".as_ptr());
}
// C line 28473
58 => {
vm_block = if ((((((((((((((*(s)).token).val) == ((91 as i32))) as i32)) != 0) || (((((((*(s)).token).val) == ((123 as i32))) as i32)) != 0)) as i32)) != 0) && (((((js_parse_skip_parens_token(s, core::ptr::addr_of_mut!(skip_bits), (0 as i32))) == ((61 as i32))) as i32)) != 0)) as i32)) != 0 { 56 } else { 57 }; continue;
}
// C line 28406
59 => {
vm_block = if ((((((*(s)).token).val) == ((TOK_IDENT as i32))) as i32)) != 0 { 53 } else { 58 }; continue;
}
// C line 28403
60 => {
name = (((0 as i32)) as JSAtom);
vm_block = 4; continue;
}
// C line 28402
61 => {
fd = (*(s)).cur_func;
vm_block = 60; continue;
}
// C line 28401
62 => {
ctx = (*(s)).ctx;
vm_block = 61; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:28495. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn is_label(mut s: *mut JSParseState) -> i32 {
let mut vm_local_storage = Vec::<u64>::new();
let mut vm_block: usize = 1;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 28497
1 => {
return (((((((((((((*(s)).token).val) == ((TOK_IDENT as i32))) as i32)) != 0) && (((!((((((*(s)).token).u).ident).is_reserved) != 0) as i32)) != 0)) as i32)) != 0) && (((((peek_token(s, (0 as i32))) == ((58 as i32))) as i32)) != 0)) as i32);
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:28502. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn is_let(mut s: *mut JSParseState, mut decl_mask: i32) -> i32 {
let mut vm_local_storage = Vec::<u64>::new();
let mut res: i32 = 0;
let mut last_token_ptr: *const u8 = core::ptr::null();
let mut pos: JSParsePos = core::mem::zeroed();
let mut vm_block: usize = 20;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 28543
1 => {
return res;
}
// C line 28540
2 => {
let _ = { let assigned = ((1 as i32)).wrapping_neg(); res = assigned; assigned };
vm_block = 1; continue;
}
// C line 28539
3 => {
vm_block = if (js_parse_seek_token(s, core::ptr::addr_of_mut!(pos))) != 0 { 2 } else { 1 }; continue;
}
// C line 28510
4 => {
vm_block = 17; continue;
}
// C line 28537
5 => {
vm_block = 3; continue;
}
// C line 28535
6 => {
vm_block = 3; continue;
}
// C line 28533
7 => {
vm_block = 3; continue;
}
// C line 28532
8 => {
let _ = { let assigned = (1 as i32); res = assigned; assigned };
vm_block = 7; continue;
}
// C line 28530
9 => {
vm_block = if ((((((!((has_lf_in_range(last_token_ptr, ((*(s)).token).ptr)) != 0) as i32)) != 0) || ((((decl_mask) & (((1 as i32)).wrapping_shl(((2 as i32)) as u32)))) != 0)) as i32)) != 0 { 8 } else { 6 }; continue;
}
// C line 28522
10 => {
vm_block = if ((((((((((((((((((((((*(s)).token).val) == ((123 as i32))) as i32)) != 0) || (((((((((((*(s)).token).val) == ((TOK_IDENT as i32))) as i32)) != 0) && (((!((((((*(s)).token).u).ident).is_reserved) != 0) as i32)) != 0)) as i32)) != 0)) as i32)) != 0) || (((((((*(s)).token).val) == ((TOK_LET as i32))) as i32)) != 0)) as i32)) != 0) || (((((((*(s)).token).val) == ((TOK_YIELD as i32))) as i32)) != 0)) as i32)) != 0) || (((((((*(s)).token).val) == ((TOK_AWAIT as i32))) as i32)) != 0)) as i32)) != 0 { 9 } else { 5 }; continue;
}
// C line 28520
11 => {
vm_block = 3; continue;
}
// C line 28519
12 => {
let _ = { let assigned = (1 as i32); res = assigned; assigned };
vm_block = 11; continue;
}
// C line 28516
13 => {
vm_block = if ((((((*(s)).token).val) == ((91 as i32))) as i32)) != 0 { 12 } else { 10 }; continue;
}
// C line 28514
14 => {
vm_block = 3; continue;
}
// C line 28513
15 => {
let _ = { let assigned = ((1 as i32)).wrapping_neg(); res = assigned; assigned };
vm_block = 14; continue;
}
// C line 28512
16 => {
vm_block = if (next_token(s)) != 0 { 15 } else { 13 }; continue;
}
// C line 28511
17 => {
let _ = { let assigned = ((*(s)).token).ptr; last_token_ptr = assigned; assigned };
vm_block = 16; continue;
}
// C line 28509
18 => {
let _ = js_parse_get_pos(s, core::ptr::addr_of_mut!(pos));
vm_block = 4; continue;
}
// C line 28507
19 => {
vm_block = if (token_is_pseudo_keyword(s, (((crate::quickjs_atom::JS_ATOM_let as i32)) as JSAtom))) != 0 { 18 } else { 1 }; continue;
}
// C line 28504
20 => {
res = (0 as i32);
vm_block = 19; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:28548. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_parse_for_in_of(mut s: *mut JSParseState, mut label_name: i32, mut is_async: i32) -> i32 {
let mut vm_local_storage = Vec::<u64>::new();
let mut ctx: *mut JSContext = core::ptr::null_mut();
let mut fd: *mut JSFunctionDef = core::ptr::null_mut();
let mut var_name: JSAtom = 0;
let mut has_initializer: i32 = 0;
let mut is_for_of: i32 = 0;
let mut has_destructuring: i32 = 0;
let mut tok: i32 = 0;
let mut tok1: i32 = 0;
let mut opcode: i32 = 0;
let mut scope: i32 = 0;
let mut block_scope_level: i32 = 0;
let mut label_next: i32 = 0;
let mut label_expr: i32 = 0;
let mut label_cont: i32 = 0;
let mut label_body: i32 = 0;
let mut label_break: i32 = 0;
let mut pos_next: i32 = 0;
let mut pos_expr: i32 = 0;
let mut break_entry: BlockEnv = core::mem::zeroed();
let mut skip_bits: i32 = 0;
let mut lvalue_label: i32 = 0;
let mut bc: *mut DynBuf = core::ptr::null_mut();
let mut chunk_size: i32 = 0;
let mut offset: i32 = 0;
let mut i: i32 = 0;
let mut ls: *mut LabelSlot = core::ptr::null_mut();
let mut vm_block: usize = 131;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 28772
1 => {
return (0 as i32);
}
// C line 28771
2 => {
let _ = pop_scope(s);
vm_block = 1; continue;
}
// C line 28770
3 => {
let _ = pop_break_entry((*(s)).cur_func);
vm_block = 2; continue;
}
// C line 28766
4 => {
let _ = emit_op(s, (((OP_iterator_close as i32)) as u8));
vm_block = 3; continue;
}
// C line 28768
5 => {
let _ = emit_op(s, (((OP_drop as i32)) as u8));
vm_block = 3; continue;
}
// C line 28764
6 => {
vm_block = if (is_for_of) != 0 { 4 } else { 5 }; continue;
}
// C line 28763
7 => {
let _ = emit_label(s, label_break);
vm_block = 6; continue;
}
// C line 28761
8 => {
let _ = emit_op(s, (((OP_drop as i32)) as u8));
vm_block = 7; continue;
}
// C line 28759
9 => {
let _ = emit_goto(s, (OP_if_false as i32), label_next);
vm_block = 8; continue;
}
// C line 28750
10 => {
let _ = emit_op(s, (((OP_iterator_get_value_done as i32)) as u8));
vm_block = 9; continue;
}
// C line 28748
11 => {
let _ = emit_op(s, (((OP_await as i32)) as u8));
vm_block = 10; continue;
}
// C line 28746
12 => {
let _ = emit_op(s, (((OP_for_await_of_next as i32)) as u8));
vm_block = 11; continue;
}
// C line 28753
13 => {
let _ = emit_u8(s, (((0 as i32)) as u8));
vm_block = 9; continue;
}
// C line 28752
14 => {
let _ = emit_op(s, (((OP_for_of_next as i32)) as u8));
vm_block = 13; continue;
}
// C line 28743
15 => {
vm_block = if (is_async) != 0 { 12 } else { 14 }; continue;
}
// C line 28756
16 => {
let _ = emit_op(s, (((OP_for_in_next as i32)) as u8));
vm_block = 9; continue;
}
// C line 28742
17 => {
vm_block = if (is_for_of) != 0 { 15 } else { 16 }; continue;
}
// C line 28741
18 => {
let _ = emit_label(s, label_cont);
vm_block = 17; continue;
}
// C line 28739
19 => {
let _ = close_scopes(s, (*((*(s)).cur_func)).scope_level, block_scope_level);
vm_block = 18; continue;
}
// C line 28737
20 => {
return ((1 as i32)).wrapping_neg();
}
// C line 28736
21 => {
vm_block = if (js_parse_statement(s)) != 0 { 20 } else { 19 }; continue;
}
// C line 28735
22 => {
let _ = emit_label(s, label_body);
vm_block = 21; continue;
}
// C line 28728
23 => {
vm_block = if ((((i) < ((*((*(s)).cur_func)).label_count)) as i32)) != 0 { 27 } else { 22 }; continue;
}
// C line 28728
24 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 23; continue;
}
// C line 28731
25 => {
let _ = { (*(ls)).pos = (((((*(ls)).pos) as i32)).wrapping_add(offset)) as i32; (*(ls)).pos };
vm_block = 24; continue;
}
// C line 28730
26 => {
vm_block = if (((((((((*(ls)).pos) >= (pos_next)) as i32)) != 0) && ((((((*(ls)).pos) < (pos_expr)) as i32)) != 0)) as i32)) != 0 { 25 } else { 24 }; continue;
}
// C line 28729
27 => {
ls = core::ptr::addr_of_mut!(*((*((*(s)).cur_func)).label_slots).offset((i) as isize));
vm_block = 26; continue;
}
// C line 28728
28 => {
let _ = { let assigned = label_cont; i = assigned; assigned };
vm_block = 23; continue;
}
// C line 28726
29 => {
let _ = { let assigned = ((((*(bc)).size).wrapping_sub((((5 as i32)) as usize))) as i32); (*((*(s)).cur_func)).last_opcode_pos = assigned; assigned };
vm_block = 28; continue;
}
// C line 28724
30 => {
let _ = { let dst = (((((*(bc)).buf).offset(((pos_next) as isize))) as *mut c_void)) as *mut u8; core::ptr::write_bytes(dst, ((OP_nop as i32)) as u8, (((chunk_size) as usize)) as usize); dst as *mut c_void };
vm_block = 29; continue;
}
// C line 28723
31 => {
let _ = crate::cutils::dbuf_put(bc, ((*(bc)).buf).offset(((pos_next) as isize)), ((chunk_size) as usize));
vm_block = 30; continue;
}
// C line 28722
32 => {
return ((1 as i32)).wrapping_neg();
}
// C line 28721
33 => {
vm_block = if (dbuf_claim(bc, ((chunk_size) as usize))) != 0 { 32 } else { 31 }; continue;
}
// C line 28719
34 => {
offset = ((((*(bc)).size).wrapping_sub(((pos_next) as usize))) as i32);
vm_block = 33; continue;
}
// C line 28718
35 => {
chunk_size = (pos_expr).wrapping_sub(pos_next);
vm_block = 34; continue;
}
// C line 28717
36 => {
bc = core::ptr::addr_of_mut!((*((*(s)).cur_func)).byte_code);
vm_block = 35; continue;
}
// C line 28715
37 => {
vm_block = if ((1 as i32)) != 0 { 36 } else { 22 }; continue;
}
// C line 28713
38 => {
return ((1 as i32)).wrapping_neg();
}
// C line 28712
39 => {
vm_block = if (js_parse_expect(s, (41 as i32))) != 0 { 38 } else { 37 }; continue;
}
// C line 28710
40 => {
let _ = emit_goto(s, (OP_goto as i32), label_cont);
vm_block = 39; continue;
}
// C line 28702
41 => {
let _ = emit_op(s, (((OP_for_await_of_start as i32)) as u8));
vm_block = 40; continue;
}
// C line 28704
42 => {
let _ = emit_op(s, (((OP_for_of_start as i32)) as u8));
vm_block = 40; continue;
}
// C line 28701
43 => {
vm_block = if (is_async) != 0 { 41 } else { 42 }; continue;
}
// C line 28700
44 => {
let _ = { (break_entry).drop_count = (((((break_entry).drop_count) as i32)).wrapping_add((2 as i32))) as i32; (break_entry).drop_count };
vm_block = 43; continue;
}
// C line 28699
45 => {
let _ = { let assigned = (((1 as i32)) as u8); (break_entry).set_has_iterator((assigned) as _); assigned };
vm_block = 44; continue;
}
// C line 28707
46 => {
let _ = emit_op(s, (((OP_for_in_start as i32)) as u8));
vm_block = 40; continue;
}
// C line 28695
47 => {
vm_block = if (is_for_of) != 0 { 45 } else { 46 }; continue;
}
// C line 28694
48 => {
let _ = close_scopes(s, (*((*(s)).cur_func)).scope_level, block_scope_level);
vm_block = 47; continue;
}
// C line 28687
49 => {
return ((1 as i32)).wrapping_neg();
}
// C line 28686
50 => {
vm_block = if (js_parse_assign_expr(s)) != 0 { 49 } else { 48 }; continue;
}
// C line 28690
51 => {
return ((1 as i32)).wrapping_neg();
}
// C line 28689
52 => {
vm_block = if (js_parse_expr(s)) != 0 { 51 } else { 48 }; continue;
}
// C line 28685
53 => {
vm_block = if (is_for_of) != 0 { 50 } else { 52 }; continue;
}
// C line 28684
54 => {
return ((1 as i32)).wrapping_neg();
}
// C line 28683
55 => {
vm_block = if (next_token(s)) != 0 { 54 } else { 53 }; continue;
}
// C line 28669
56 => {
vm_block = 59; continue;
}
// C line 28668
57 => {
vm_block = if (has_initializer) != 0 { 56 } else { 55 }; continue;
}
// C line 28667
58 => {
let _ = { let assigned = (1 as i32); is_for_of = assigned; assigned };
vm_block = 57; continue;
}
// C line 28677 labels: initializer_error
59 => {
return js_parse_error_cargs(s, c"a declaration in the head of a for-%s loop can't have an initializer".as_ptr(), &[ParserFormatArg::C((if (is_for_of) != 0 { c"of".as_ptr() } else { c"in".as_ptr() }) as *const c_char)]);
}
// C line 28673
60 => {
vm_block = if (((((has_initializer) != 0) && (((((((((((((tok) != ((TOK_VAR as i32))) as i32)) != 0) || (((((((*(fd)).js_mode) as i32)) & (((1 as i32)).wrapping_shl(((0 as i32)) as u32)))) != 0)) as i32)) != 0) || ((has_destructuring) != 0)) as i32)) != 0)) as i32)) != 0 { 59 } else { 55 }; continue;
}
// C line 28672
61 => {
return js_parse_error(s, c"'for await' loop should be used with 'of'".as_ptr());
}
// C line 28671
62 => {
vm_block = if (is_async) != 0 { 61 } else { 60 }; continue;
}
// C line 28681
63 => {
return js_parse_error(s, c"expected 'of' or 'in' in for control expression".as_ptr());
}
// C line 28670
64 => {
vm_block = if ((((((*(s)).token).val) == ((TOK_IN as i32))) as i32)) != 0 { 62 } else { 63 }; continue;
}
// C line 28666
65 => {
vm_block = if (token_is_pseudo_keyword(s, (((crate::quickjs_atom::JS_ATOM_of as i32)) as JSAtom))) != 0 { 58 } else { 64 }; continue;
}
// C line 28664
66 => {
let _ = JS_FreeAtom(ctx, var_name);
vm_block = 65; continue;
}
// C line 28661
67 => {
let _ = emit_u16(s, (((*(fd)).scope_level) as u16));
vm_block = 66; continue;
}
// C line 28660
68 => {
let _ = emit_atom(s, var_name);
vm_block = 67; continue;
}
// C line 28659
69 => {
let _ = emit_op(s, (((OP_scope_put_var as i32)) as u8));
vm_block = 68; continue;
}
// C line 28658
70 => {
vm_block = if ((((var_name) != ((((0 as i32)) as JSAtom))) as i32)) != 0 { 69 } else { 66 }; continue;
}
// C line 28656
71 => {
return ((1 as i32)).wrapping_neg();
}
// C line 28655
72 => {
let _ = JS_FreeAtom(ctx, var_name);
vm_block = 71; continue;
}
// C line 28654
73 => {
vm_block = if (((((next_token(s)) != 0) || ((js_parse_assign_expr2(s, (0 as i32))) != 0)) as i32)) != 0 { 72 } else { 70 }; continue;
}
// C line 28650
74 => {
let _ = { let assigned = (1 as i32); has_initializer = assigned; assigned };
vm_block = 73; continue;
}
// C line 28648
75 => {
vm_block = if ((((((*(s)).token).val) == ((61 as i32))) as i32)) != 0 { 74 } else { 66 }; continue;
}
// C line 28647
76 => {
let _ = emit_label(s, label_expr);
vm_block = 75; continue;
}
// C line 28646
77 => {
let _ = { let assigned = ((((*((*(s)).cur_func)).byte_code).size) as i32); pos_expr = assigned; assigned };
vm_block = 76; continue;
}
// C line 28644
78 => {
let _ = emit_goto(s, (OP_goto as i32), label_body);
vm_block = 77; continue;
}
// C line 28607
79 => {
let _ = { let assigned = (((0 as i32)) as JSAtom); var_name = assigned; assigned };
vm_block = 78; continue;
}
// C line 28603
80 => {
let _ = { let assigned = (1 as i32); has_destructuring = assigned; assigned };
vm_block = 79; continue;
}
// C line 28602
81 => {
return ((1 as i32)).wrapping_neg();
}
// C line 28601
82 => {
vm_block = if ((((js_parse_destructuring_element(s, tok, (0 as i32), (1 as i32), ((1 as i32)).wrapping_neg(), (0 as i32), (0 as i32))) < ((0 as i32))) as i32)) != 0 { 81 } else { 80 }; continue;
}
// C line 28605
83 => {
return js_parse_error(s, c"variable name expected".as_ptr());
}
// C line 28600
84 => {
vm_block = if ((((((((((*(s)).token).val) == ((91 as i32))) as i32)) != 0) || (((((((*(s)).token).val) == ((123 as i32))) as i32)) != 0)) as i32)) != 0 { 82 } else { 83 }; continue;
}
// C line 28621
85 => {
let _ = emit_u16(s, (((*(fd)).scope_level) as u16));
vm_block = 78; continue;
}
// C line 28620
86 => {
let _ = emit_atom(s, var_name);
vm_block = 85; continue;
}
// C line 28618
87 => {
let _ = emit_op(s, ((if ((((((((tok) == ((TOK_CONST as i32))) as i32)) != 0) || (((((tok) == ((TOK_LET as i32))) as i32)) != 0)) as i32)) != 0 { (OP_scope_put_var_init as i32) } else { (OP_scope_put_var as i32) }) as u8));
vm_block = 86; continue;
}
// C line 28616
88 => {
return ((1 as i32)).wrapping_neg();
}
// C line 28615
89 => {
let _ = JS_FreeAtom((*(s)).ctx, var_name);
vm_block = 88; continue;
}
// C line 28614
90 => {
vm_block = if (js_define_var(s, var_name, tok)) != 0 { 89 } else { 87 }; continue;
}
// C line 28612
91 => {
return ((1 as i32)).wrapping_neg();
}
// C line 28611
92 => {
let _ = JS_FreeAtom((*(s)).ctx, var_name);
vm_block = 91; continue;
}
// C line 28610
93 => {
vm_block = if (next_token(s)) != 0 { 92 } else { 90 }; continue;
}
// C line 28609
94 => {
let _ = { let assigned = JS_DupAtom(ctx, ((((*(s)).token).u).ident).atom); var_name = assigned; assigned };
vm_block = 93; continue;
}
// C line 28599
95 => {
vm_block = if ((!(((((((((((*(s)).token).val) == ((TOK_IDENT as i32))) as i32)) != 0) && (((!((((((*(s)).token).u).ident).is_reserved) != 0) as i32)) != 0)) as i32)) != 0) as i32)) != 0 { 84 } else { 94 }; continue;
}
// C line 28597
96 => {
return ((1 as i32)).wrapping_neg();
}
// C line 28596
97 => {
vm_block = if (next_token(s)) != 0 { 96 } else { 95 }; continue;
}
// C line 28625
98 => {
return js_parse_error(s, c"'for of' expression cannot start with 'async'".as_ptr());
}
// C line 28642
99 => {
let _ = { let assigned = (((0 as i32)) as JSAtom); var_name = assigned; assigned };
vm_block = 78; continue;
}
// C line 28631
100 => {
return ((1 as i32)).wrapping_neg();
}
// C line 28630
101 => {
vm_block = if ((((js_parse_destructuring_element(s, (0 as i32), (0 as i32), (1 as i32), ((skip_bits) & (((1 as i32)).wrapping_shl(((1 as i32)) as u32))), (1 as i32), (0 as i32))) < ((0 as i32))) as i32)) != 0 { 100 } else { 99 }; continue;
}
// C line 28639
102 => {
let _ = put_lvalue(s, opcode, scope, var_name, lvalue_label, (((PUT_LVALUE_NOKEEP_BOTTOM as i32)) as PutLValueEnum), (0 as i32));
vm_block = 99; continue;
}
// C line 28638
103 => {
return ((1 as i32)).wrapping_neg();
}
// C line 28636
104 => {
vm_block = if (get_lvalue(s, core::ptr::addr_of_mut!(opcode), core::ptr::addr_of_mut!(scope), core::ptr::addr_of_mut!(var_name), core::ptr::addr_of_mut!(lvalue_label), core::ptr::null_mut::<i32>(), (0 as i32), (TOK_FOR as i32))) != 0 { 103 } else { 102 }; continue;
}
// C line 28635
105 => {
return ((1 as i32)).wrapping_neg();
}
// C line 28634
106 => {
vm_block = if (js_parse_left_hand_side_expr(s)) != 0 { 105 } else { 104 }; continue;
}
// C line 28628
107 => {
vm_block = if ((((((((((((((*(s)).token).val) == ((91 as i32))) as i32)) != 0) || (((((((*(s)).token).val) == ((123 as i32))) as i32)) != 0)) as i32)) != 0) && ((((((((({ let assigned = js_parse_skip_parens_token(s, core::ptr::addr_of_mut!(skip_bits), (0 as i32)); tok1 = assigned; assigned }) == ((TOK_IN as i32))) as i32)) != 0) || (((((tok1) == ((TOK_OF as i32))) as i32)) != 0)) as i32)) != 0)) as i32)) != 0 { 101 } else { 106 }; continue;
}
// C line 28623
108 => {
vm_block = if ((((((((((!((is_async) != 0) as i32)) != 0) && ((token_is_pseudo_keyword(s, (((crate::quickjs_atom::JS_ATOM_async as i32)) as JSAtom))) != 0)) as i32)) != 0) && (((((peek_token(s, (0 as i32))) == ((TOK_OF as i32))) as i32)) != 0)) as i32)) != 0 { 98 } else { 107 }; continue;
}
// C line 28595
109 => {
vm_block = if ((((((((((((tok) == ((TOK_VAR as i32))) as i32)) != 0) || (((((tok) == ((TOK_LET as i32))) as i32)) != 0)) as i32)) != 0) || (((((tok) == ((TOK_CONST as i32))) as i32)) != 0)) as i32)) != 0 { 97 } else { 108 }; continue;
}
// C line 28593
110 => {
return ((1 as i32)).wrapping_neg();
}
// C line 28591
111 => {
vm_block = 109; continue;
}
// C line 28589
112 => {
vm_block = 109; continue;
}
// C line 28588
113 => {
let _ = { let assigned = (TOK_LET as i32); tok = assigned; assigned };
vm_block = 112; continue;
}
// C line 28586
114 => {
vm_block = match is_let(s, ((1 as i32)).wrapping_shl(((2 as i32)) as u32)) { x if x == (0 as i32) => 111, x if x == (1 as i32) => 113, _ => 110, }; continue;
}
// C line 28585
115 => {
let _ = { let assigned = ((*(s)).token).val; tok = assigned; assigned };
vm_block = 114; continue;
}
// C line 28583
116 => {
let _ = emit_label(s, label_next);
vm_block = 115; continue;
}
// C line 28582
117 => {
let _ = { let assigned = ((((*((*(s)).cur_func)).byte_code).size) as i32); pos_next = assigned; assigned };
vm_block = 116; continue;
}
// C line 28580
118 => {
let _ = { let assigned = emit_goto(s, (OP_goto as i32), ((1 as i32)).wrapping_neg()); label_expr = assigned; assigned };
vm_block = 117; continue;
}
// C line 28578
119 => {
let _ = { let assigned = block_scope_level; (break_entry).scope_level = assigned; assigned };
vm_block = 118; continue;
}
// C line 28576
120 => {
let _ = push_break_entry((*(s)).cur_func, core::ptr::addr_of_mut!(break_entry), ((label_name) as JSAtom), label_break, label_cont, (1 as i32));
vm_block = 119; continue;
}
// C line 28572
121 => {
let _ = push_scope(s);
vm_block = 120; continue;
}
// C line 28567
122 => {
let _ = { let assigned = new_label(s); label_next = assigned; assigned };
vm_block = 121; continue;
}
// C line 28566
123 => {
let _ = { let assigned = new_label(s); label_break = assigned; assigned };
vm_block = 122; continue;
}
// C line 28565
124 => {
let _ = { let assigned = new_label(s); label_body = assigned; assigned };
vm_block = 123; continue;
}
// C line 28564
125 => {
let _ = { let assigned = new_label(s); label_cont = assigned; assigned };
vm_block = 124; continue;
}
// C line 28563
126 => {
let _ = { let assigned = (*(fd)).scope_level; block_scope_level = assigned; assigned };
vm_block = 125; continue;
}
// C line 28562
127 => {
let _ = { let assigned = (0 as i32); is_for_of = assigned; assigned };
vm_block = 126; continue;
}
// C line 28561
128 => {
let _ = { let assigned = (0 as i32); has_destructuring = assigned; assigned };
vm_block = 127; continue;
}
// C line 28560
129 => {
let _ = { let assigned = (0 as i32); has_initializer = assigned; assigned };
vm_block = 128; continue;
}
// C line 28552
130 => {
fd = (*(s)).cur_func;
vm_block = 129; continue;
}
// C line 28551
131 => {
ctx = (*(s)).ctx;
vm_block = 130; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:28775. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn set_eval_ret_undefined(mut s: *mut JSParseState) -> () {
let mut vm_local_storage = Vec::<u64>::new();
let mut vm_block: usize = 4;
loop { match vm_block {
// C line ?
0 => {
return ();
}
// C line 28780
1 => {
let _ = emit_u16(s, (((*((*(s)).cur_func)).eval_ret_idx) as u16));
vm_block = 0; continue;
}
// C line 28779
2 => {
let _ = emit_op(s, (((OP_put_loc as i32)) as u8));
vm_block = 1; continue;
}
// C line 28778
3 => {
let _ = emit_op(s, (((OP_undefined as i32)) as u8));
vm_block = 2; continue;
}
// C line 28777
4 => {
vm_block = if (((((*((*(s)).cur_func)).eval_ret_idx) >= ((0 as i32))) as i32)) != 0 { 3 } else { 0 }; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:28784. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_parse_statement_or_decl(mut s: *mut JSParseState, mut decl_mask: i32) -> i32 {
let mut vm_local_storage = Vec::<u64>::new();
let mut ctx: *mut JSContext = core::ptr::null_mut();
let mut label_name: JSAtom = 0;
let mut tok: i32 = 0;
let mut be: *mut BlockEnv = core::ptr::null_mut();
let mut label_break: i32 = 0;
let mut mask: i32 = 0;
let mut break_entry: BlockEnv = core::mem::zeroed();
let mut op_token_ptr: *const u8 = core::ptr::null();
let mut op_token_ptr_1: *const u8 = core::ptr::null();
let mut label1: i32 = 0;
let mut label2: i32 = 0;
let mut mask_1: i32 = 0;
let mut label_cont: i32 = 0;
let mut label_break_1: i32 = 0;
let mut break_entry_1: BlockEnv = core::mem::zeroed();
let mut label_cont_1: i32 = 0;
let mut label_break_2: i32 = 0;
let mut label1_1: i32 = 0;
let mut break_entry_2: BlockEnv = core::mem::zeroed();
let mut label_cont_2: i32 = 0;
let mut label_break_3: i32 = 0;
let mut label_body: i32 = 0;
let mut label_test: i32 = 0;
let mut pos_cont: i32 = 0;
let mut pos_body: i32 = 0;
let mut block_scope_level: i32 = 0;
let mut break_entry_3: BlockEnv = core::mem::zeroed();
let mut tok_1: i32 = 0;
let mut bits: i32 = 0;
let mut is_async: i32 = 0;
let mut bc: *mut DynBuf = core::ptr::null_mut();
let mut chunk_size: i32 = 0;
let mut offset: i32 = 0;
let mut i: i32 = 0;
let mut ls: *mut LabelSlot = core::ptr::null_mut();
let mut is_cont: i32 = 0;
let mut label: i32 = 0;
let mut label_case: i32 = 0;
let mut label_break_4: i32 = 0;
let mut label1_2: i32 = 0;
let mut default_label_pos: i32 = 0;
let mut break_entry_4: BlockEnv = core::mem::zeroed();
let mut label_catch: i32 = 0;
let mut label_catch2: i32 = 0;
let mut label_finally: i32 = 0;
let mut label_end: i32 = 0;
let mut name: JSAtom = 0;
let mut block_env: BlockEnv = core::mem::zeroed();
let mut saved_eval_ret_idx: i32 = 0;
let mut with_idx: i32 = 0;
let mut vm_block: usize = 464;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 29543
1 => {
return ((1 as i32)).wrapping_neg();
}
// C line 29542 labels: fail
2 => {
let _ = JS_FreeAtom(ctx, label_name);
vm_block = 1; continue;
}
// C line 29540
3 => {
return (0 as i32);
}
// C line 29539 labels: done
4 => {
let _ = JS_FreeAtom(ctx, label_name);
vm_block = 3; continue;
}
// C line 29536
5 => {
vm_block = 4; continue;
}
// C line 29535
6 => {
vm_block = 2; continue;
}
// C line 29534
7 => {
vm_block = if (js_parse_expect_semi(s)) != 0 { 6 } else { 5 }; continue;
}
// C line 29530
8 => {
let _ = emit_u16(s, (((*((*(s)).cur_func)).eval_ret_idx) as u16));
vm_block = 7; continue;
}
// C line 29529
9 => {
let _ = emit_op(s, (((OP_put_loc as i32)) as u8));
vm_block = 8; continue;
}
// C line 29532
10 => {
let _ = emit_op(s, (((OP_drop as i32)) as u8));
vm_block = 7; continue;
}
// C line 29526
11 => {
vm_block = if (((((*((*(s)).cur_func)).eval_ret_idx) >= ((0 as i32))) as i32)) != 0 { 9 } else { 10 }; continue;
}
// C line 29525
12 => {
vm_block = 2; continue;
}
// C line 29524
13 => {
vm_block = if (js_parse_expr(s)) != 0 { 12 } else { 11 }; continue;
}
// C line 29523 labels: hasexpr
14 => {
let _ = emit_source_pos(s, ((*(s)).token).ptr);
vm_block = 13; continue;
}
// C line 29519
15 => {
vm_block = 2; continue;
}
// C line 29518
16 => {
let _ = js_unsupported_keyword(s, ((((*(s)).token).u).ident).atom);
vm_block = 15; continue;
}
// C line 29513
17 => {
vm_block = 4; continue;
}
// C line 29512
18 => {
vm_block = 2; continue;
}
// C line 29511
19 => {
vm_block = if (js_parse_expect_semi(s)) != 0 { 18 } else { 17 }; continue;
}
// C line 29510
20 => {
vm_block = 2; continue;
}
// C line 29509
21 => {
vm_block = if (next_token(s)) != 0 { 20 } else { 19 }; continue;
}
// C line 29505
22 => {
vm_block = 4; continue;
}
// C line 29504
23 => {
return ((1 as i32)).wrapping_neg();
}
// C line 29503
24 => {
vm_block = if (js_parse_class(s, (0 as i32), (((JS_PARSE_EXPORT_NONE as i32)) as JSParseExportEnum))) != 0 { 23 } else { 22 }; continue;
}
// C line 29501
25 => {
vm_block = 2; continue;
}
// C line 29500
26 => {
let _ = js_parse_error(s, c"class declarations can't appear in single-statement context".as_ptr());
vm_block = 25; continue;
}
// C line 29499
27 => {
vm_block = if ((!((((decl_mask) & (((1 as i32)).wrapping_shl(((2 as i32)) as u32)))) != 0) as i32)) != 0 { 26 } else { 24 }; continue;
}
// C line 29496
28 => {
vm_block = 14; continue;
}
// C line 29494
29 => {
vm_block = 4; continue;
}
// C line 29493
30 => {
vm_block = 2; continue;
}
// C line 29490 labels: parse_func_var
31 => {
vm_block = if (js_parse_function_decl(s, (((JS_PARSE_FUNC_VAR as i32)) as JSParseFunctionEnum), (((JS_FUNC_NORMAL as i32)) as JSFunctionKindEnum), (((0 as i32)) as JSAtom), ((*(s)).token).ptr)) != 0 { 30 } else { 29 }; continue;
}
// C line 29487
32 => {
vm_block = 2; continue;
}
// C line 29486 labels: func_decl_error
33 => {
let _ = js_parse_error(s, c"function declarations can't appear in single-statement context".as_ptr());
vm_block = 32; continue;
}
// C line 29484
34 => {
vm_block = if ((!((((decl_mask) & (((1 as i32)).wrapping_shl(((2 as i32)) as u32)))) != 0) as i32)) != 0 { 33 } else { 31 }; continue;
}
// C line 29482
35 => {
vm_block = if (((((token_is_pseudo_keyword(s, (((crate::quickjs_atom::JS_ATOM_async as i32)) as JSAtom))) != 0) && (((((peek_token(s, (1 as i32))) == ((TOK_FUNCTION as i32))) as i32)) != 0)) as i32)) != 0 { 34 } else { 28 }; continue;
}
// C line 29480
36 => {
vm_block = 2; continue;
}
// C line 29478
37 => {
vm_block = 35; continue;
}
// C line 29476
38 => {
vm_block = 402; continue;
}
// C line 29475
39 => {
let _ = { let assigned = (TOK_LET as i32); tok = assigned; assigned };
vm_block = 38; continue;
}
// C line 29473
40 => {
vm_block = match is_let(s, decl_mask) { x if x == (0 as i32) => 37, x if x == (1 as i32) => 39, _ => 36, }; continue;
}
// C line 29470
41 => {
vm_block = 2; continue;
}
// C line 29469
42 => {
let _ = js_parse_error_reserved_identifier(s);
vm_block = 41; continue;
}
// C line 29468
43 => {
vm_block = if (((((*(s)).token).u).ident).is_reserved) != 0 { 42 } else { 40 }; continue;
}
// C line 29466
44 => {
vm_block = 31; continue;
}
// C line 29465
45 => {
vm_block = 33; continue;
}
// C line 29464
46 => {
vm_block = if ((((((!((((decl_mask) & (((1 as i32)).wrapping_shl(((2 as i32)) as u32)))) != 0) as i32)) != 0) && (((((peek_token(s, (0 as i32))) == ((42 as i32))) as i32)) != 0)) as i32)) != 0 { 45 } else { 44 }; continue;
}
// C line 29463
47 => {
vm_block = 33; continue;
}
// C line 29462
48 => {
vm_block = if ((!((((decl_mask) & (((1 as i32)).wrapping_shl(((0 as i32)) as u32)))) != 0) as i32)) != 0 { 47 } else { 46 }; continue;
}
// C line 29459
49 => {
vm_block = 4; continue;
}
// C line 29433
50 => {
vm_block = 2; continue;
}
// C line 29432
51 => {
let _ = js_parse_error(s, c"invalid keyword: with".as_ptr());
vm_block = 50; continue;
}
// C line 29457
52 => {
let _ = pop_scope(s);
vm_block = 49; continue;
}
// C line 29454
53 => {
vm_block = 2; continue;
}
// C line 29453
54 => {
vm_block = if (js_parse_statement(s)) != 0 { 53 } else { 52 }; continue;
}
// C line 29452
55 => {
let _ = set_eval_ret_undefined(s);
vm_block = 54; continue;
}
// C line 29450
56 => {
let _ = emit_u16(s, ((with_idx) as u16));
vm_block = 55; continue;
}
// C line 29449
57 => {
let _ = emit_op(s, (((OP_put_loc as i32)) as u8));
vm_block = 56; continue;
}
// C line 29448
58 => {
let _ = emit_op(s, (((OP_to_object as i32)) as u8));
vm_block = 57; continue;
}
// C line 29447
59 => {
vm_block = 2; continue;
}
// C line 29446
60 => {
vm_block = if ((((with_idx) < ((0 as i32))) as i32)) != 0 { 59 } else { 58 }; continue;
}
// C line 29444
61 => {
let _ = { let assigned = define_var(s, (*(s)).cur_func, (((crate::quickjs_atom::JS_ATOM__with_ as i32)) as JSAtom), (((JS_VAR_DEF_WITH as i32)) as JSVarDefEnum)); with_idx = assigned; assigned };
vm_block = 60; continue;
}
// C line 29443
62 => {
let _ = push_scope(s);
vm_block = 61; continue;
}
// C line 29441
63 => {
vm_block = 2; continue;
}
// C line 29440
64 => {
vm_block = if (js_parse_expr_paren(s)) != 0 { 63 } else { 62 }; continue;
}
// C line 29438
65 => {
vm_block = 2; continue;
}
// C line 29437
66 => {
vm_block = if (next_token(s)) != 0 { 65 } else { 64 }; continue;
}
// C line 29431
67 => {
vm_block = if ((((((*((*(s)).cur_func)).js_mode) as i32)) & (((1 as i32)).wrapping_shl(((0 as i32)) as u32)))) != 0 { 51 } else { 66 }; continue;
}
// C line 29429
68 => {
vm_block = 4; continue;
}
// C line 29428
69 => {
vm_block = 2; continue;
}
// C line 29427
70 => {
vm_block = if (next_token(s)) != 0 { 69 } else { 68 }; continue;
}
// C line 29424
71 => {
vm_block = 4; continue;
}
// C line 29422
72 => {
let _ = emit_label(s, label_end);
vm_block = 71; continue;
}
// C line 29421
73 => {
let _ = emit_op(s, (((OP_ret as i32)) as u8));
vm_block = 72; continue;
}
// C line 29419
74 => {
let _ = pop_break_entry((*(s)).cur_func);
vm_block = 73; continue;
}
// C line 29417
75 => {
let _ = emit_u16(s, (((*((*(s)).cur_func)).eval_ret_idx) as u16));
vm_block = 74; continue;
}
// C line 29416
76 => {
let _ = emit_op(s, (((OP_put_loc as i32)) as u8));
vm_block = 75; continue;
}
// C line 29415
77 => {
let _ = emit_u16(s, ((saved_eval_ret_idx) as u16));
vm_block = 76; continue;
}
// C line 29414
78 => {
let _ = emit_op(s, (((OP_get_loc as i32)) as u8));
vm_block = 77; continue;
}
// C line 29413
79 => {
vm_block = if (((((*((*(s)).cur_func)).eval_ret_idx) >= ((0 as i32))) as i32)) != 0 { 78 } else { 74 }; continue;
}
// C line 29411
80 => {
vm_block = 2; continue;
}
// C line 29410
81 => {
vm_block = if (js_parse_block(s)) != 0 { 80 } else { 79 }; continue;
}
// C line 29407
82 => {
let _ = set_eval_ret_undefined(s);
vm_block = 81; continue;
}
// C line 29406
83 => {
let _ = emit_u16(s, ((saved_eval_ret_idx) as u16));
vm_block = 82; continue;
}
// C line 29405
84 => {
let _ = emit_op(s, (((OP_put_loc as i32)) as u8));
vm_block = 83; continue;
}
// C line 29404
85 => {
let _ = emit_u16(s, (((*((*(s)).cur_func)).eval_ret_idx) as u16));
vm_block = 84; continue;
}
// C line 29403
86 => {
let _ = emit_op(s, (((OP_get_loc as i32)) as u8));
vm_block = 85; continue;
}
// C line 29402
87 => {
vm_block = 2; continue;
}
// C line 29401
88 => {
vm_block = if ((((saved_eval_ret_idx) < ((0 as i32))) as i32)) != 0 { 87 } else { 86 }; continue;
}
// C line 29399
89 => {
let _ = { let assigned = add_var((*(s)).ctx, (*(s)).cur_func, (((crate::quickjs_atom::JS_ATOM__ret_ as i32)) as JSAtom)); saved_eval_ret_idx = assigned; assigned };
vm_block = 88; continue;
}
// C line 29396
90 => {
vm_block = if (((((*((*(s)).cur_func)).eval_ret_idx) >= ((0 as i32))) as i32)) != 0 { 89 } else { 81 }; continue;
}
// C line 29393
91 => {
let _ = push_break_entry((*(s)).cur_func, core::ptr::addr_of_mut!(block_env), (((0 as i32)) as JSAtom), ((1 as i32)).wrapping_neg(), ((1 as i32)).wrapping_neg(), (2 as i32));
vm_block = 90; continue;
}
// C line 29391
92 => {
vm_block = 2; continue;
}
// C line 29390
93 => {
vm_block = if (next_token(s)) != 0 { 92 } else { 91 }; continue;
}
// C line 29388
94 => {
saved_eval_ret_idx = (0 as i32);
vm_block = 93; continue;
}
// C line 29387
95 => {
vm_block = if ((((((*(s)).token).val) == ((TOK_FINALLY as i32))) as i32)) != 0 { 94 } else { 73 }; continue;
}
// C line 29386
96 => {
let _ = emit_label(s, label_finally);
vm_block = 95; continue;
}
// C line 29373
97 => {
let _ = emit_op(s, (((OP_throw as i32)) as u8));
vm_block = 96; continue;
}
// C line 29372
98 => {
let _ = emit_goto(s, (OP_gosub as i32), label_finally);
vm_block = 97; continue;
}
// C line 29370
99 => {
let _ = emit_label(s, label_catch2);
vm_block = 98; continue;
}
// C line 29366
100 => {
let _ = emit_goto(s, (OP_goto as i32), label_end);
vm_block = 99; continue;
}
// C line 29365
101 => {
let _ = emit_op(s, (((OP_drop as i32)) as u8));
vm_block = 100; continue;
}
// C line 29364
102 => {
let _ = emit_goto(s, (OP_gosub as i32), label_finally);
vm_block = 101; continue;
}
// C line 29363
103 => {
let _ = emit_op(s, (((OP_undefined as i32)) as u8));
vm_block = 102; continue;
}
// C line 29360
104 => {
let _ = emit_op(s, (((OP_drop as i32)) as u8));
vm_block = 103; continue;
}
// C line 29358
105 => {
vm_block = if (js_is_live_code(s)) != 0 { 104 } else { 99 }; continue;
}
// C line 29356
106 => {
let _ = pop_scope(s);
vm_block = 105; continue;
}
// C line 29355
107 => {
let _ = pop_scope(s);
vm_block = 106; continue;
}
// C line 29354
108 => {
let _ = pop_break_entry((*(s)).cur_func);
vm_block = 107; continue;
}
// C line 29352
109 => {
vm_block = 2; continue;
}
// C line 29351
110 => {
vm_block = if (js_parse_block(s)) != 0 { 109 } else { 108 }; continue;
}
// C line 29349
111 => {
let _ = { let assigned = label_finally; (block_env).label_finally = assigned; assigned };
vm_block = 110; continue;
}
// C line 29347
112 => {
let _ = push_break_entry((*(s)).cur_func, core::ptr::addr_of_mut!(block_env), (((0 as i32)) as JSAtom), ((1 as i32)).wrapping_neg(), ((1 as i32)).wrapping_neg(), (1 as i32));
vm_block = 111; continue;
}
// C line 29346
113 => {
let _ = push_scope(s);
vm_block = 112; continue;
}
// C line 29344
114 => {
let _ = emit_goto(s, (OP_catch as i32), label_catch2);
vm_block = 113; continue;
}
// C line 29315
115 => {
let _ = emit_op(s, (((OP_drop as i32)) as u8));
vm_block = 114; continue;
}
// C line 29341
116 => {
vm_block = 2; continue;
}
// C line 29340
117 => {
vm_block = if (js_parse_expect(s, (41 as i32))) != 0 { 116 } else { 114 }; continue;
}
// C line 29323
118 => {
vm_block = 2; continue;
}
// C line 29322
119 => {
vm_block = if ((((js_parse_destructuring_element(s, (TOK_LET as i32), (0 as i32), (1 as i32), ((1 as i32)).wrapping_neg(), (1 as i32), (0 as i32))) < ((0 as i32))) as i32)) != 0 { 118 } else { 117 }; continue;
}
// C line 29326
120 => {
vm_block = 2; continue;
}
// C line 29325
121 => {
let _ = js_parse_error(s, c"identifier expected".as_ptr());
vm_block = 120; continue;
}
// C line 29320
122 => {
vm_block = if ((((((((((*(s)).token).val) == ((91 as i32))) as i32)) != 0) || (((((((*(s)).token).val) == ((123 as i32))) as i32)) != 0)) as i32)) != 0 { 119 } else { 121 }; continue;
}
// C line 29338
123 => {
let _ = emit_u16(s, (((*((*(s)).cur_func)).scope_level) as u16));
vm_block = 117; continue;
}
// C line 29337
124 => {
let _ = emit_u32(s, name);
vm_block = 123; continue;
}
// C line 29336
125 => {
let _ = emit_op(s, (((OP_scope_put_var as i32)) as u8));
vm_block = 124; continue;
}
// C line 29333
126 => {
vm_block = 2; continue;
}
// C line 29332
127 => {
let _ = JS_FreeAtom(ctx, name);
vm_block = 126; continue;
}
// C line 29330
128 => {
vm_block = if (((((next_token(s)) != 0) || (((((js_define_var(s, name, (TOK_CATCH as i32))) < ((0 as i32))) as i32)) != 0)) as i32)) != 0 { 127 } else { 125 }; continue;
}
// C line 29329
129 => {
let _ = { let assigned = JS_DupAtom(ctx, ((((*(s)).token).u).ident).atom); name = assigned; assigned };
vm_block = 128; continue;
}
// C line 29319
130 => {
vm_block = if ((!(((((((((((*(s)).token).val) == ((TOK_IDENT as i32))) as i32)) != 0) && (((!((((((*(s)).token).u).ident).is_reserved) != 0) as i32)) != 0)) as i32)) != 0) as i32)) != 0 { 122 } else { 129 }; continue;
}
// C line 29318
131 => {
vm_block = 2; continue;
}
// C line 29317
132 => {
vm_block = if (js_parse_expect(s, (40 as i32))) != 0 { 131 } else { 130 }; continue;
}
// C line 29313
133 => {
vm_block = if ((((((*(s)).token).val) == ((123 as i32))) as i32)) != 0 { 115 } else { 132 }; continue;
}
// C line 29311
134 => {
let _ = emit_label(s, label_catch);
vm_block = 133; continue;
}
// C line 29310
135 => {
let _ = push_scope(s);
vm_block = 134; continue;
}
// C line 29308
136 => {
vm_block = 2; continue;
}
// C line 29307
137 => {
vm_block = if (next_token(s)) != 0 { 136 } else { 135 }; continue;
}
// C line 29381
138 => {
let _ = emit_op(s, (((OP_throw as i32)) as u8));
vm_block = 96; continue;
}
// C line 29380
139 => {
let _ = emit_goto(s, (OP_gosub as i32), label_finally);
vm_block = 138; continue;
}
// C line 29378
140 => {
let _ = emit_label(s, label_catch);
vm_block = 139; continue;
}
// C line 29384
141 => {
vm_block = 2; continue;
}
// C line 29383
142 => {
let _ = js_parse_error(s, c"expecting catch or finally".as_ptr());
vm_block = 141; continue;
}
// C line 29375
143 => {
vm_block = if ((((((*(s)).token).val) == ((TOK_FINALLY as i32))) as i32)) != 0 { 140 } else { 142 }; continue;
}
// C line 29306
144 => {
vm_block = if ((((((*(s)).token).val) == ((TOK_CATCH as i32))) as i32)) != 0 { 137 } else { 143 }; continue;
}
// C line 29303
145 => {
let _ = emit_goto(s, (OP_goto as i32), label_end);
vm_block = 144; continue;
}
// C line 29301
146 => {
let _ = emit_op(s, (((OP_drop as i32)) as u8));
vm_block = 145; continue;
}
// C line 29300
147 => {
let _ = emit_goto(s, (OP_gosub as i32), label_finally);
vm_block = 146; continue;
}
// C line 29299
148 => {
let _ = emit_op(s, (((OP_undefined as i32)) as u8));
vm_block = 147; continue;
}
// C line 29297
149 => {
let _ = emit_op(s, (((OP_drop as i32)) as u8));
vm_block = 148; continue;
}
// C line 29295
150 => {
vm_block = if (js_is_live_code(s)) != 0 { 149 } else { 144 }; continue;
}
// C line 29293
151 => {
let _ = pop_break_entry((*(s)).cur_func);
vm_block = 150; continue;
}
// C line 29291
152 => {
vm_block = 2; continue;
}
// C line 29290
153 => {
vm_block = if (js_parse_block(s)) != 0 { 152 } else { 151 }; continue;
}
// C line 29288
154 => {
let _ = { let assigned = label_finally; (block_env).label_finally = assigned; assigned };
vm_block = 153; continue;
}
// C line 29286
155 => {
let _ = push_break_entry((*(s)).cur_func, core::ptr::addr_of_mut!(block_env), (((0 as i32)) as JSAtom), ((1 as i32)).wrapping_neg(), ((1 as i32)).wrapping_neg(), (1 as i32));
vm_block = 154; continue;
}
// C line 29284
156 => {
let _ = emit_goto(s, (OP_catch as i32), label_catch);
vm_block = 155; continue;
}
// C line 29282
157 => {
let _ = { let assigned = new_label(s); label_end = assigned; assigned };
vm_block = 156; continue;
}
// C line 29281
158 => {
let _ = { let assigned = new_label(s); label_finally = assigned; assigned };
vm_block = 157; continue;
}
// C line 29280
159 => {
let _ = { let assigned = new_label(s); label_catch2 = assigned; assigned };
vm_block = 158; continue;
}
// C line 29279
160 => {
let _ = { let assigned = new_label(s); label_catch = assigned; assigned };
vm_block = 159; continue;
}
// C line 29278
161 => {
vm_block = 2; continue;
}
// C line 29277
162 => {
vm_block = if (next_token(s)) != 0 { 161 } else { 160 }; continue;
}
// C line 29276
163 => {
let _ = set_eval_ret_undefined(s);
vm_block = 162; continue;
}
// C line 29269
164 => {
vm_block = 4; continue;
}
// C line 29267
165 => {
let _ = pop_scope(s);
vm_block = 164; continue;
}
// C line 29266
166 => {
let _ = pop_break_entry((*(s)).cur_func);
vm_block = 165; continue;
}
// C line 29264
167 => {
let _ = emit_op(s, (((OP_drop as i32)) as u8));
vm_block = 166; continue;
}
// C line 29263
168 => {
let _ = emit_label(s, label_break_4);
vm_block = 167; continue;
}
// C line 29259
169 => {
let _ = { let assigned = (default_label_pos).wrapping_add((4 as i32)); (*((*((*(s)).cur_func)).label_slots).offset((label_case) as isize)).pos = assigned; assigned };
vm_block = 168; continue;
}
// C line 29257
170 => {
let _ = put_u32((((*((*(s)).cur_func)).byte_code).buf).offset(((default_label_pos) as isize)), ((label_case) as u32));
vm_block = 169; continue;
}
// C line 29261
171 => {
let _ = emit_label(s, label_case);
vm_block = 168; continue;
}
// C line 29255
172 => {
vm_block = if ((((default_label_pos) >= ((0 as i32))) as i32)) != 0 { 170 } else { 171 }; continue;
}
// C line 29254
173 => {
vm_block = 2; continue;
}
// C line 29253
174 => {
vm_block = if (js_parse_expect(s, (125 as i32))) != 0 { 173 } else { 172 }; continue;
}
// C line 29195
175 => {
vm_block = if ((((((*(s)).token).val) != ((125 as i32))) as i32)) != 0 { 213 } else { 174 }; continue;
}
// C line 29204
176 => {
vm_block = 189; continue;
}
// C line 29215
177 => {
let _ = { let assigned = emit_goto(s, (OP_if_true as i32), label1_2); label1_2 = assigned; assigned };
vm_block = 176; continue;
}
// C line 29219
178 => {
vm_block = 175; continue;
}
// C line 29218
179 => {
let _ = emit_label(s, label1_2);
vm_block = 178; continue;
}
// C line 29217
180 => {
let _ = { let assigned = emit_goto(s, (OP_if_false as i32), ((1 as i32)).wrapping_neg()); label_case = assigned; assigned };
vm_block = 179; continue;
}
// C line 29214
181 => {
vm_block = if ((((((*(s)).token).val) == ((TOK_CASE as i32))) as i32)) != 0 { 177 } else { 180 }; continue;
}
// C line 29213
182 => {
let _ = emit_op(s, (((OP_strict_eq as i32)) as u8));
vm_block = 181; continue;
}
// C line 29212
183 => {
vm_block = 2; continue;
}
// C line 29211
184 => {
vm_block = if (js_parse_expect(s, (58 as i32))) != 0 { 183 } else { 182 }; continue;
}
// C line 29210
185 => {
vm_block = 2; continue;
}
// C line 29209
186 => {
vm_block = if (js_parse_expr(s)) != 0 { 185 } else { 184 }; continue;
}
// C line 29208
187 => {
let _ = emit_op(s, (((OP_dup as i32)) as u8));
vm_block = 186; continue;
}
// C line 29207
188 => {
vm_block = 2; continue;
}
// C line 29206
189 => {
vm_block = if (next_token(s)) != 0 { 188 } else { 187 }; continue;
}
// C line 29203
190 => {
let _ = { let assigned = ((1 as i32)).wrapping_neg(); label_case = assigned; assigned };
vm_block = 176; continue;
}
// C line 29202
191 => {
let _ = emit_label(s, label_case);
vm_block = 190; continue;
}
// C line 29200
192 => {
let _ = { let assigned = emit_goto(s, (OP_goto as i32), ((1 as i32)).wrapping_neg()); label1_2 = assigned; assigned };
vm_block = 191; continue;
}
// C line 29198
193 => {
vm_block = if ((((label_case) >= ((0 as i32))) as i32)) != 0 { 192 } else { 191 }; continue;
}
// C line 29197
194 => {
let _ = { let assigned = ((1 as i32)).wrapping_neg(); label1_2 = assigned; assigned };
vm_block = 193; continue;
}
// C line 29242
195 => {
let _ = { let assigned = (((((*((*(s)).cur_func)).byte_code).size).wrapping_sub((((4 as i32)) as usize))) as i32); default_label_pos = assigned; assigned };
vm_block = 175; continue;
}
// C line 29241
196 => {
let _ = emit_u32(s, (((0 as i32)) as u32));
vm_block = 195; continue;
}
// C line 29240
197 => {
let _ = emit_op(s, (((OP_label as i32)) as u8));
vm_block = 196; continue;
}
// C line 29233
198 => {
let _ = { let assigned = emit_goto(s, (OP_goto as i32), ((1 as i32)).wrapping_neg()); label_case = assigned; assigned };
vm_block = 197; continue;
}
// C line 29231
199 => {
vm_block = if ((((label_case) < ((0 as i32))) as i32)) != 0 { 198 } else { 197 }; continue;
}
// C line 29229
200 => {
vm_block = 2; continue;
}
// C line 29228
201 => {
let _ = js_parse_error(s, c"duplicate default".as_ptr());
vm_block = 200; continue;
}
// C line 29227
202 => {
vm_block = if ((((default_label_pos) >= ((0 as i32))) as i32)) != 0 { 201 } else { 199 }; continue;
}
// C line 29226
203 => {
vm_block = 2; continue;
}
// C line 29225
204 => {
vm_block = if (js_parse_expect(s, (58 as i32))) != 0 { 203 } else { 202 }; continue;
}
// C line 29224
205 => {
vm_block = 2; continue;
}
// C line 29223
206 => {
vm_block = if (next_token(s)) != 0 { 205 } else { 204 }; continue;
}
// C line 29250
207 => {
vm_block = 2; continue;
}
// C line 29249
208 => {
vm_block = if (js_parse_statement_or_decl(s, ((((((1 as i32)).wrapping_shl(((0 as i32)) as u32)) | (((1 as i32)).wrapping_shl(((1 as i32)) as u32)))) | (((1 as i32)).wrapping_shl(((2 as i32)) as u32))))) != 0 { 207 } else { 175 }; continue;
}
// C line 29247
209 => {
vm_block = 2; continue;
}
// C line 29246
210 => {
let _ = js_parse_error(s, c"invalid switch statement".as_ptr());
vm_block = 209; continue;
}
// C line 29244
211 => {
vm_block = if ((((label_case) < ((0 as i32))) as i32)) != 0 { 210 } else { 208 }; continue;
}
// C line 29222
212 => {
vm_block = if ((((((*(s)).token).val) == ((TOK_DEFAULT as i32))) as i32)) != 0 { 206 } else { 211 }; continue;
}
// C line 29196
213 => {
vm_block = if ((((((*(s)).token).val) == ((TOK_CASE as i32))) as i32)) != 0 { 194 } else { 212 }; continue;
}
// C line 29194
214 => {
let _ = { let assigned = ((1 as i32)).wrapping_neg(); label_case = assigned; assigned };
vm_block = 175; continue;
}
// C line 29193
215 => {
let _ = { let assigned = ((1 as i32)).wrapping_neg(); default_label_pos = assigned; assigned };
vm_block = 214; continue;
}
// C line 29191
216 => {
vm_block = 2; continue;
}
// C line 29190
217 => {
vm_block = if (js_parse_expect(s, (123 as i32))) != 0 { 216 } else { 215 }; continue;
}
// C line 29187
218 => {
let _ = push_break_entry((*(s)).cur_func, core::ptr::addr_of_mut!(break_entry_4), label_name, label_break_4, ((1 as i32)).wrapping_neg(), (1 as i32));
vm_block = 217; continue;
}
// C line 29186
219 => {
let _ = { let assigned = new_label(s); label_break_4 = assigned; assigned };
vm_block = 218; continue;
}
// C line 29185
220 => {
let _ = push_scope(s);
vm_block = 219; continue;
}
// C line 29183
221 => {
vm_block = 2; continue;
}
// C line 29182
222 => {
vm_block = if (js_parse_expr_paren(s)) != 0 { 221 } else { 220 }; continue;
}
// C line 29181
223 => {
let _ = set_eval_ret_undefined(s);
vm_block = 222; continue;
}
// C line 29179
224 => {
vm_block = 2; continue;
}
// C line 29178
225 => {
vm_block = if (next_token(s)) != 0 { 224 } else { 223 }; continue;
}
// C line 29171
226 => {
vm_block = 4; continue;
}
// C line 29169
227 => {
vm_block = 2; continue;
}
// C line 29168
228 => {
vm_block = if (js_parse_expect_semi(s)) != 0 { 227 } else { 226 }; continue;
}
// C line 29166
229 => {
vm_block = 2; continue;
}
// C line 29165
230 => {
vm_block = if (next_token(s)) != 0 { 229 } else { 228 }; continue;
}
// C line 29164
231 => {
vm_block = if ((((label) != ((0 as i32))) as i32)) != 0 { 230 } else { 228 }; continue;
}
// C line 29163
232 => {
vm_block = 2; continue;
}
// C line 29162
233 => {
vm_block = if (emit_break(s, ((label) as JSAtom), is_cont)) != 0 { 232 } else { 231 }; continue;
}
// C line 29159
234 => {
let _ = { let assigned = ((((((*(s)).token).u).ident).atom) as i32); label = assigned; assigned };
vm_block = 233; continue;
}
// C line 29161
235 => {
let _ = { let assigned = (0 as i32); label = assigned; assigned };
vm_block = 233; continue;
}
// C line 29158
236 => {
vm_block = if ((((((((((!(((*(s)).got_lf) != 0) as i32)) != 0) && (((((((*(s)).token).val) == ((TOK_IDENT as i32))) as i32)) != 0)) as i32)) != 0) && (((!((((((*(s)).token).u).ident).is_reserved) != 0) as i32)) != 0)) as i32)) != 0 { 234 } else { 235 }; continue;
}
// C line 29157
237 => {
vm_block = 2; continue;
}
// C line 29156
238 => {
vm_block = if (next_token(s)) != 0 { 237 } else { 236 }; continue;
}
// C line 29153
239 => {
is_cont = (((*(s)).token).val).wrapping_sub((TOK_BREAK as i32));
vm_block = 238; continue;
}
// C line 29149
240 => {
vm_block = 4; continue;
}
// C line 29147
241 => {
let _ = pop_scope(s);
vm_block = 240; continue;
}
// C line 29146
242 => {
let _ = pop_break_entry((*(s)).cur_func);
vm_block = 241; continue;
}
// C line 29144
243 => {
let _ = emit_label(s, label_break_3);
vm_block = 242; continue;
}
// C line 29135
244 => {
vm_block = if ((((i) < ((*((*(s)).cur_func)).label_count)) as i32)) != 0 { 248 } else { 243 }; continue;
}
// C line 29135
245 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 244; continue;
}
// C line 29138
246 => {
let _ = { (*(ls)).pos = (((((*(ls)).pos) as i32)).wrapping_add(offset)) as i32; (*(ls)).pos };
vm_block = 245; continue;
}
// C line 29137
247 => {
vm_block = if (((((((((*(ls)).pos) >= (pos_cont)) as i32)) != 0) && ((((((*(ls)).pos) < (pos_body)) as i32)) != 0)) as i32)) != 0 { 246 } else { 245 }; continue;
}
// C line 29136
248 => {
ls = core::ptr::addr_of_mut!(*((*((*(s)).cur_func)).label_slots).offset((i) as isize));
vm_block = 247; continue;
}
// C line 29135
249 => {
let _ = { let assigned = label_cont_2; i = assigned; assigned };
vm_block = 244; continue;
}
// C line 29133
250 => {
let _ = { let assigned = ((((*(bc)).size).wrapping_sub((((5 as i32)) as usize))) as i32); (*((*(s)).cur_func)).last_opcode_pos = assigned; assigned };
vm_block = 249; continue;
}
// C line 29131
251 => {
let _ = { let dst = (((((*(bc)).buf).offset(((pos_cont) as isize))) as *mut c_void)) as *mut u8; core::ptr::write_bytes(dst, ((OP_nop as i32)) as u8, (((chunk_size) as usize)) as usize); dst as *mut c_void };
vm_block = 250; continue;
}
// C line 29130
252 => {
let _ = crate::cutils::dbuf_put(bc, ((*(bc)).buf).offset(((pos_cont) as isize)), ((chunk_size) as usize));
vm_block = 251; continue;
}
// C line 29129
253 => {
vm_block = 2; continue;
}
// C line 29128
254 => {
vm_block = if (dbuf_claim(bc, ((chunk_size) as usize))) != 0 { 253 } else { 252 }; continue;
}
// C line 29126
255 => {
offset = ((((*(bc)).size).wrapping_sub(((pos_cont) as usize))) as i32);
vm_block = 254; continue;
}
// C line 29125
256 => {
chunk_size = (pos_body).wrapping_sub(pos_cont);
vm_block = 255; continue;
}
// C line 29124
257 => {
bc = core::ptr::addr_of_mut!((*((*(s)).cur_func)).byte_code);
vm_block = 256; continue;
}
// C line 29141
258 => {
let _ = emit_goto(s, (OP_goto as i32), label_cont_2);
vm_block = 243; continue;
}
// C line 29122
259 => {
vm_block = if ((((((((((1 as i32)) != 0) && (((((label_test) != (label_body)) as i32)) != 0)) as i32)) != 0) && (((((label_cont_2) != (label_test)) as i32)) != 0)) as i32)) != 0 { 257 } else { 258 }; continue;
}
// C line 29120
260 => {
let _ = close_scopes(s, (*((*(s)).cur_func)).scope_level, block_scope_level);
vm_block = 259; continue;
}
// C line 29116
261 => {
vm_block = 2; continue;
}
// C line 29115
262 => {
vm_block = if (js_parse_statement(s)) != 0 { 261 } else { 260 }; continue;
}
// C line 29114
263 => {
let _ = emit_label(s, label_body);
vm_block = 262; continue;
}
// C line 29113
264 => {
let _ = { let assigned = ((((*((*(s)).cur_func)).byte_code).size) as i32); pos_body = assigned; assigned };
vm_block = 263; continue;
}
// C line 29111
265 => {
vm_block = 2; continue;
}
// C line 29110
266 => {
vm_block = if (js_parse_expect(s, (41 as i32))) != 0 { 265 } else { 264 }; continue;
}
// C line 29097
267 => {
let _ = { let assigned = (0 as i32); pos_cont = assigned; assigned };
vm_block = 266; continue;
}
// C line 29096
268 => {
let _ = { let assigned = { let assigned = label_test; label_cont_2 = assigned; assigned }; (break_entry_3).label_cont = assigned; assigned };
vm_block = 267; continue;
}
// C line 29108
269 => {
let _ = emit_goto(s, (OP_goto as i32), label_test);
vm_block = 266; continue;
}
// C line 29107
270 => {
vm_block = if ((((label_test) != (label_body)) as i32)) != 0 { 269 } else { 266 }; continue;
}
// C line 29106
271 => {
let _ = emit_op(s, (((OP_drop as i32)) as u8));
vm_block = 270; continue;
}
// C line 29105
272 => {
vm_block = 2; continue;
}
// C line 29104
273 => {
vm_block = if (js_parse_expr(s)) != 0 { 272 } else { 271 }; continue;
}
// C line 29103
274 => {
let _ = emit_label(s, label_cont_2);
vm_block = 273; continue;
}
// C line 29102
275 => {
let _ = { let assigned = ((((*((*(s)).cur_func)).byte_code).size) as i32); pos_cont = assigned; assigned };
vm_block = 274; continue;
}
// C line 29100
276 => {
let _ = emit_goto(s, (OP_goto as i32), label_body);
vm_block = 275; continue;
}
// C line 29094
277 => {
vm_block = if ((((((*(s)).token).val) == ((41 as i32))) as i32)) != 0 { 268 } else { 276 }; continue;
}
// C line 29092
278 => {
vm_block = 2; continue;
}
// C line 29091
279 => {
vm_block = if (js_parse_expect(s, (59 as i32))) != 0 { 278 } else { 277 }; continue;
}
// C line 29084
280 => {
let _ = { let assigned = label_body; label_test = assigned; assigned };
vm_block = 279; continue;
}
// C line 29089
281 => {
let _ = emit_goto(s, (OP_if_false as i32), label_break_3);
vm_block = 279; continue;
}
// C line 29088
282 => {
vm_block = 2; continue;
}
// C line 29087
283 => {
vm_block = if (js_parse_expr(s)) != 0 { 282 } else { 281 }; continue;
}
// C line 29086
284 => {
let _ = emit_label(s, label_test);
vm_block = 283; continue;
}
// C line 29082
285 => {
vm_block = if ((((((*(s)).token).val) == ((59 as i32))) as i32)) != 0 { 280 } else { 284 }; continue;
}
// C line 29078
286 => {
let _ = push_break_entry((*(s)).cur_func, core::ptr::addr_of_mut!(break_entry_3), label_name, label_break_3, label_cont_2, (0 as i32));
vm_block = 285; continue;
}
// C line 29076
287 => {
let _ = { let assigned = new_label(s); label_break_3 = assigned; assigned };
vm_block = 286; continue;
}
// C line 29075
288 => {
let _ = { let assigned = new_label(s); label_body = assigned; assigned };
vm_block = 287; continue;
}
// C line 29074
289 => {
let _ = { let assigned = new_label(s); label_cont_2 = assigned; assigned };
vm_block = 288; continue;
}
// C line 29073
290 => {
let _ = { let assigned = new_label(s); label_test = assigned; assigned };
vm_block = 289; continue;
}
// C line 29071
291 => {
vm_block = 2; continue;
}
// C line 29070
292 => {
vm_block = if (js_parse_expect(s, (59 as i32))) != 0 { 291 } else { 290 }; continue;
}
// C line 29068
293 => {
let _ = close_scopes(s, (*((*(s)).cur_func)).scope_level, block_scope_level);
vm_block = 292; continue;
}
// C line 29060
294 => {
vm_block = 2; continue;
}
// C line 29059
295 => {
vm_block = if (js_parse_var(s, (0 as i32), tok_1, (0 as i32))) != 0 { 294 } else { 293 }; continue;
}
// C line 29058
296 => {
vm_block = 2; continue;
}
// C line 29057
297 => {
vm_block = if (next_token(s)) != 0 { 296 } else { 295 }; continue;
}
// C line 29064
298 => {
let _ = emit_op(s, (((OP_drop as i32)) as u8));
vm_block = 293; continue;
}
// C line 29063
299 => {
vm_block = 2; continue;
}
// C line 29062
300 => {
vm_block = if (js_parse_expr2(s, (0 as i32))) != 0 { 299 } else { 298 }; continue;
}
// C line 29056
301 => {
vm_block = if ((((((((((((tok_1) == ((TOK_VAR as i32))) as i32)) != 0) || (((((tok_1) == ((TOK_LET as i32))) as i32)) != 0)) as i32)) != 0) || (((((tok_1) == ((TOK_CONST as i32))) as i32)) != 0)) as i32)) != 0 { 297 } else { 300 }; continue;
}
// C line 29054
302 => {
vm_block = 2; continue;
}
// C line 29052
303 => {
vm_block = 301; continue;
}
// C line 29050
304 => {
vm_block = 301; continue;
}
// C line 29049
305 => {
let _ = { let assigned = (TOK_LET as i32); tok_1 = assigned; assigned };
vm_block = 304; continue;
}
// C line 29047
306 => {
vm_block = match is_let(s, ((1 as i32)).wrapping_shl(((2 as i32)) as u32)) { x if x == (0 as i32) => 303, x if x == (1 as i32) => 305, _ => 302, }; continue;
}
// C line 29046
307 => {
vm_block = if ((((tok_1) != ((59 as i32))) as i32)) != 0 { 306 } else { 292 }; continue;
}
// C line 29045
308 => {
let _ = { let assigned = ((*(s)).token).val; tok_1 = assigned; assigned };
vm_block = 307; continue;
}
// C line 29043
309 => {
let _ = push_scope(s);
vm_block = 308; continue;
}
// C line 29039
310 => {
let _ = { let assigned = (*((*(s)).cur_func)).scope_level; block_scope_level = assigned; assigned };
vm_block = 309; continue;
}
// C line 29037
311 => {
vm_block = 4; continue;
}
// C line 29036
312 => {
vm_block = 2; continue;
}
// C line 29035
313 => {
vm_block = if (js_parse_for_in_of(s, ((label_name) as i32), is_async)) != 0 { 312 } else { 311 }; continue;
}
// C line 29033
314 => {
vm_block = if ((!((((bits) & (((1 as i32)).wrapping_shl(((0 as i32)) as u32)))) != 0) as i32)) != 0 { 313 } else { 310 }; continue;
}
// C line 29031
315 => {
vm_block = 2; continue;
}
// C line 29030
316 => {
vm_block = if (js_parse_expect(s, (40 as i32))) != 0 { 315 } else { 314 }; continue;
}
// C line 29019
317 => {
let _ = js_parse_skip_parens_token(s, core::ptr::addr_of_mut!(bits), (0 as i32));
vm_block = 316; continue;
}
// C line 29028
318 => {
let _ = { let assigned = (1 as i32); (*((*(s)).cur_func)).has_await = assigned; assigned };
vm_block = 316; continue;
}
// C line 29027
319 => {
vm_block = 2; continue;
}
// C line 29026
320 => {
vm_block = if (next_token(s)) != 0 { 319 } else { 318 }; continue;
}
// C line 29025
321 => {
let _ = { let assigned = (1 as i32); is_async = assigned; assigned };
vm_block = 320; continue;
}
// C line 29023
322 => {
vm_block = 2; continue;
}
// C line 29022
323 => {
let _ = js_parse_error(s, c"for await is only valid in asynchronous functions".as_ptr());
vm_block = 322; continue;
}
// C line 29021
324 => {
vm_block = if ((!((((((((*((*(s)).cur_func)).func_kind as JSFunctionKindEnum)) as i32)) & ((JS_FUNC_ASYNC as i32)))) != 0) as i32)) != 0 { 323 } else { 321 }; continue;
}
// C line 29020
325 => {
vm_block = if ((((((*(s)).token).val) == ((TOK_AWAIT as i32))) as i32)) != 0 { 324 } else { 316 }; continue;
}
// C line 29018
326 => {
vm_block = if ((((((*(s)).token).val) == ((40 as i32))) as i32)) != 0 { 317 } else { 325 }; continue;
}
// C line 29017
327 => {
let _ = { let assigned = (0 as i32); is_async = assigned; assigned };
vm_block = 326; continue;
}
// C line 29016
328 => {
let _ = { let assigned = (0 as i32); bits = assigned; assigned };
vm_block = 327; continue;
}
// C line 29015
329 => {
let _ = set_eval_ret_undefined(s);
vm_block = 328; continue;
}
// C line 29013
330 => {
vm_block = 2; continue;
}
// C line 29012
331 => {
vm_block = if (next_token(s)) != 0 { 330 } else { 329 }; continue;
}
// C line 29003
332 => {
vm_block = 4; continue;
}
// C line 29001
333 => {
let _ = pop_break_entry((*(s)).cur_func);
vm_block = 332; continue;
}
// C line 28999
334 => {
let _ = emit_label(s, label_break_2);
vm_block = 333; continue;
}
// C line 28997
335 => {
let _ = emit_goto(s, (OP_if_true as i32), label1_1);
vm_block = 334; continue;
}
// C line 28995
336 => {
vm_block = 2; continue;
}
// C line 28994
337 => {
vm_block = if (next_token(s)) != 0 { 336 } else { 335 }; continue;
}
// C line 28993
338 => {
vm_block = if ((((((*(s)).token).val) == ((59 as i32))) as i32)) != 0 { 337 } else { 335 }; continue;
}
// C line 28991
339 => {
vm_block = 2; continue;
}
// C line 28990
340 => {
vm_block = if (js_parse_expr_paren(s)) != 0 { 339 } else { 338 }; continue;
}
// C line 28989
341 => {
vm_block = 2; continue;
}
// C line 28988
342 => {
vm_block = if (js_parse_expect(s, (TOK_WHILE as i32))) != 0 { 341 } else { 340 }; continue;
}
// C line 28987
343 => {
let _ = emit_label(s, label_cont_1);
vm_block = 342; continue;
}
// C line 28985
344 => {
vm_block = 2; continue;
}
// C line 28984
345 => {
vm_block = if (js_parse_statement(s)) != 0 { 344 } else { 343 }; continue;
}
// C line 28982
346 => {
let _ = set_eval_ret_undefined(s);
vm_block = 345; continue;
}
// C line 28980
347 => {
let _ = emit_label(s, label1_1);
vm_block = 346; continue;
}
// C line 28978
348 => {
vm_block = 2; continue;
}
// C line 28977
349 => {
vm_block = if (next_token(s)) != 0 { 348 } else { 347 }; continue;
}
// C line 28974
350 => {
let _ = push_break_entry((*(s)).cur_func, core::ptr::addr_of_mut!(break_entry_2), label_name, label_break_2, label_cont_1, (0 as i32));
vm_block = 349; continue;
}
// C line 28972
351 => {
let _ = { let assigned = new_label(s); label1_1 = assigned; assigned };
vm_block = 350; continue;
}
// C line 28971
352 => {
let _ = { let assigned = new_label(s); label_break_2 = assigned; assigned };
vm_block = 351; continue;
}
// C line 28970
353 => {
let _ = { let assigned = new_label(s); label_cont_1 = assigned; assigned };
vm_block = 352; continue;
}
// C line 28964
354 => {
vm_block = 4; continue;
}
// C line 28962
355 => {
let _ = pop_break_entry((*(s)).cur_func);
vm_block = 354; continue;
}
// C line 28960
356 => {
let _ = emit_label(s, label_break_1);
vm_block = 355; continue;
}
// C line 28958
357 => {
let _ = emit_goto(s, (OP_goto as i32), label_cont);
vm_block = 356; continue;
}
// C line 28957
358 => {
vm_block = 2; continue;
}
// C line 28956
359 => {
vm_block = if (js_parse_statement(s)) != 0 { 358 } else { 357 }; continue;
}
// C line 28954
360 => {
let _ = emit_goto(s, (OP_if_false as i32), label_break_1);
vm_block = 359; continue;
}
// C line 28953
361 => {
vm_block = 2; continue;
}
// C line 28952
362 => {
vm_block = if (js_parse_expr_paren(s)) != 0 { 361 } else { 360 }; continue;
}
// C line 28951
363 => {
let _ = emit_label(s, label_cont);
vm_block = 362; continue;
}
// C line 28949
364 => {
let _ = set_eval_ret_undefined(s);
vm_block = 363; continue;
}
// C line 28947
365 => {
vm_block = 2; continue;
}
// C line 28946
366 => {
vm_block = if (next_token(s)) != 0 { 365 } else { 364 }; continue;
}
// C line 28943
367 => {
let _ = push_break_entry((*(s)).cur_func, core::ptr::addr_of_mut!(break_entry_1), label_name, label_break_1, label_cont, (0 as i32));
vm_block = 366; continue;
}
// C line 28941
368 => {
let _ = { let assigned = new_label(s); label_break_1 = assigned; assigned };
vm_block = 367; continue;
}
// C line 28940
369 => {
let _ = { let assigned = new_label(s); label_cont = assigned; assigned };
vm_block = 368; continue;
}
// C line 28934
370 => {
vm_block = 4; continue;
}
// C line 28932
371 => {
let _ = pop_scope(s);
vm_block = 370; continue;
}
// C line 28931
372 => {
let _ = emit_label(s, label1);
vm_block = 371; continue;
}
// C line 28929
373 => {
let _ = { let assigned = label2; label1 = assigned; assigned };
vm_block = 372; continue;
}
// C line 28927
374 => {
vm_block = 2; continue;
}
// C line 28926
375 => {
vm_block = if (js_parse_statement_or_decl(s, mask_1)) != 0 { 374 } else { 373 }; continue;
}
// C line 28925
376 => {
let _ = emit_label(s, label1);
vm_block = 375; continue;
}
// C line 28923
377 => {
vm_block = 2; continue;
}
// C line 28922
378 => {
vm_block = if (next_token(s)) != 0 { 377 } else { 376 }; continue;
}
// C line 28921
379 => {
let _ = { let assigned = emit_goto(s, (OP_goto as i32), ((1 as i32)).wrapping_neg()); label2 = assigned; assigned };
vm_block = 378; continue;
}
// C line 28920
380 => {
vm_block = if ((((((*(s)).token).val) == ((TOK_ELSE as i32))) as i32)) != 0 { 379 } else { 372 }; continue;
}
// C line 28918
381 => {
vm_block = 2; continue;
}
// C line 28917
382 => {
vm_block = if (js_parse_statement_or_decl(s, mask_1)) != 0 { 381 } else { 380 }; continue;
}
// C line 28913
383 => {
let _ = { let assigned = (0 as i32); mask_1 = assigned; assigned };
vm_block = 382; continue;
}
// C line 28915
384 => {
let _ = { let assigned = ((1 as i32)).wrapping_shl(((0 as i32)) as u32); mask_1 = assigned; assigned };
vm_block = 382; continue;
}
// C line 28912
385 => {
vm_block = if ((((((*((*(s)).cur_func)).js_mode) as i32)) & (((1 as i32)).wrapping_shl(((0 as i32)) as u32)))) != 0 { 383 } else { 384 }; continue;
}
// C line 28911
386 => {
let _ = { let assigned = emit_goto(s, (OP_if_false as i32), ((1 as i32)).wrapping_neg()); label1 = assigned; assigned };
vm_block = 385; continue;
}
// C line 28910
387 => {
vm_block = 2; continue;
}
// C line 28909
388 => {
vm_block = if (js_parse_expr_paren(s)) != 0 { 387 } else { 386 }; continue;
}
// C line 28908
389 => {
let _ = set_eval_ret_undefined(s);
vm_block = 388; continue;
}
// C line 28907
390 => {
let _ = push_scope(s);
vm_block = 389; continue;
}
// C line 28905
391 => {
vm_block = 2; continue;
}
// C line 28904
392 => {
vm_block = if (next_token(s)) != 0 { 391 } else { 390 }; continue;
}
// C line 28900
393 => {
vm_block = 4; continue;
}
// C line 28899
394 => {
vm_block = 2; continue;
}
// C line 28898
395 => {
vm_block = if (js_parse_expect_semi(s)) != 0 { 394 } else { 393 }; continue;
}
// C line 28897
396 => {
vm_block = 2; continue;
}
// C line 28896
397 => {
vm_block = if (js_parse_var(s, (1 as i32), tok, (0 as i32))) != 0 { 396 } else { 395 }; continue;
}
// C line 28895
398 => {
vm_block = 2; continue;
}
// C line 28894
399 => {
vm_block = if (next_token(s)) != 0 { 398 } else { 397 }; continue;
}
// C line 28890
400 => {
vm_block = 2; continue;
}
// C line 28889
401 => {
let _ = js_parse_error(s, c"lexical declarations can't appear in single-statement context".as_ptr());
vm_block = 400; continue;
}
// C line 28888 labels: haslet
402 => {
vm_block = if ((!((((decl_mask) & (((1 as i32)).wrapping_shl(((2 as i32)) as u32)))) != 0) as i32)) != 0 { 401 } else { 399 }; continue;
}
// C line 28884
403 => {
vm_block = 4; continue;
}
// C line 28882
404 => {
vm_block = 2; continue;
}
// C line 28881
405 => {
vm_block = if (js_parse_expect_semi(s)) != 0 { 404 } else { 403 }; continue;
}
// C line 28880
406 => {
let _ = emit_op(s, (((OP_throw as i32)) as u8));
vm_block = 405; continue;
}
// C line 28879
407 => {
let _ = emit_source_pos(s, op_token_ptr_1);
vm_block = 406; continue;
}
// C line 28878
408 => {
vm_block = 2; continue;
}
// C line 28877
409 => {
vm_block = if (js_parse_expr(s)) != 0 { 408 } else { 407 }; continue;
}
// C line 28875
410 => {
vm_block = 2; continue;
}
// C line 28874
411 => {
let _ = js_parse_error(s, c"line terminator not allowed after throw".as_ptr());
vm_block = 410; continue;
}
// C line 28873
412 => {
vm_block = if ((*(s)).got_lf) != 0 { 411 } else { 409 }; continue;
}
// C line 28872
413 => {
vm_block = 2; continue;
}
// C line 28871
414 => {
vm_block = if (next_token(s)) != 0 { 413 } else { 412 }; continue;
}
// C line 28870
415 => {
let _ = { let assigned = ((*(s)).token).ptr; op_token_ptr_1 = assigned; assigned };
vm_block = 414; continue;
}
// C line 28866
416 => {
vm_block = 4; continue;
}
// C line 28864
417 => {
vm_block = 2; continue;
}
// C line 28863
418 => {
vm_block = if (js_parse_expect_semi(s)) != 0 { 417 } else { 416 }; continue;
}
// C line 28858
419 => {
let _ = emit_return(s, (1 as i32));
vm_block = 418; continue;
}
// C line 28857
420 => {
let _ = emit_source_pos(s, op_token_ptr);
vm_block = 419; continue;
}
// C line 28856
421 => {
vm_block = 2; continue;
}
// C line 28855
422 => {
vm_block = if (js_parse_expr(s)) != 0 { 421 } else { 420 }; continue;
}
// C line 28861
423 => {
let _ = emit_return(s, (0 as i32));
vm_block = 418; continue;
}
// C line 28860
424 => {
let _ = emit_source_pos(s, op_token_ptr);
vm_block = 423; continue;
}
// C line 28854
425 => {
vm_block = if ((((((((((((((*(s)).token).val) != ((59 as i32))) as i32)) != 0) && (((((((*(s)).token).val) != ((125 as i32))) as i32)) != 0)) as i32)) != 0) && (((!(((*(s)).got_lf) != 0) as i32)) != 0)) as i32)) != 0 { 422 } else { 424 }; continue;
}
// C line 28853
426 => {
vm_block = 2; continue;
}
// C line 28852
427 => {
vm_block = if (next_token(s)) != 0 { 426 } else { 425 }; continue;
}
// C line 28851
428 => {
let _ = { let assigned = ((*(s)).token).ptr; op_token_ptr = assigned; assigned };
vm_block = 427; continue;
}
// C line 28849
429 => {
vm_block = 2; continue;
}
// C line 28848
430 => {
let _ = js_parse_error(s, c"return in a static initializer block".as_ptr());
vm_block = 429; continue;
}
// C line 28847
431 => {
vm_block = if ((((((((*((*(s)).cur_func)).func_type as JSParseFunctionEnum)) as i32)) == ((JS_PARSE_FUNC_CLASS_STATIC_INIT as i32))) as i32)) != 0 { 430 } else { 428 }; continue;
}
// C line 28845
432 => {
vm_block = 2; continue;
}
// C line 28844
433 => {
let _ = js_parse_error(s, c"return not in a function".as_ptr());
vm_block = 432; continue;
}
// C line 28843
434 => {
vm_block = if ((*((*(s)).cur_func)).is_eval) != 0 { 433 } else { 431 }; continue;
}
// C line 28839
435 => {
vm_block = 4; continue;
}
// C line 28838
436 => {
vm_block = 2; continue;
}
// C line 28837
437 => {
vm_block = if (js_parse_block(s)) != 0 { 436 } else { 435 }; continue;
}
// C line 28835
438 => {
vm_block = match { let assigned = ((*(s)).token).val; tok = assigned; assigned } { x if x == (TOK_EXTENDS as i32) => 16, x if x == (TOK_EXPORT as i32) => 16, x if x == (TOK_ENUM as i32) => 16, x if x == (TOK_DEBUGGER as i32) => 21, x if x == (TOK_CLASS as i32) => 27, x if x == (TOK_IDENT as i32) => 43, x if x == (TOK_FUNCTION as i32) => 48, x if x == (TOK_WITH as i32) => 67, x if x == (59 as i32) => 70, x if x == (TOK_TRY as i32) => 163, x if x == (TOK_SWITCH as i32) => 225, x if x == (TOK_CONTINUE as i32) => 239, x if x == (TOK_BREAK as i32) => 239, x if x == (TOK_FOR as i32) => 331, x if x == (TOK_DO as i32) => 353, x if x == (TOK_WHILE as i32) => 369, x if x == (TOK_IF as i32) => 392, x if x == (TOK_VAR as i32) => 399, x if x == (TOK_CONST as i32) => 402, x if x == (TOK_LET as i32) => 402, x if x == (TOK_THROW as i32) => 415, x if x == (TOK_RETURN as i32) => 434, x if x == (123 as i32) => 437, _ => 14, }; continue;
}
// C line 28831
439 => {
vm_block = 4; continue;
}
// C line 28830
440 => {
let _ = pop_break_entry((*(s)).cur_func);
vm_block = 439; continue;
}
// C line 28829
441 => {
let _ = emit_label(s, label_break);
vm_block = 440; continue;
}
// C line 28828
442 => {
vm_block = 2; continue;
}
// C line 28827
443 => {
vm_block = if (js_parse_statement_or_decl(s, mask)) != 0 { 442 } else { 441 }; continue;
}
// C line 28823
444 => {
let _ = { let assigned = ((((1 as i32)).wrapping_shl(((0 as i32)) as u32)) | (((1 as i32)).wrapping_shl(((1 as i32)) as u32))); mask = assigned; assigned };
vm_block = 443; continue;
}
// C line 28825
445 => {
let _ = { let assigned = (0 as i32); mask = assigned; assigned };
vm_block = 443; continue;
}
// C line 28821
446 => {
vm_block = if ((((((!(((((((*((*(s)).cur_func)).js_mode) as i32)) & (((1 as i32)).wrapping_shl(((0 as i32)) as u32)))) != 0) as i32)) != 0) && ((((decl_mask) & (((1 as i32)).wrapping_shl(((1 as i32)) as u32)))) != 0)) as i32)) != 0 { 444 } else { 445 }; continue;
}
// C line 28820
447 => {
let _ = { let assigned = (((1 as i32)) as u8); (break_entry).set_is_regular_stmt((assigned) as _); assigned };
vm_block = 446; continue;
}
// C line 28818
448 => {
let _ = push_break_entry((*(s)).cur_func, core::ptr::addr_of_mut!(break_entry), label_name, label_break, ((1 as i32)).wrapping_neg(), (0 as i32));
vm_block = 447; continue;
}
// C line 28817
449 => {
let _ = { let assigned = new_label(s); label_break = assigned; assigned };
vm_block = 448; continue;
}
// C line 28810
450 => {
vm_block = if ((((((((((((((*(s)).token).val) != ((TOK_FOR as i32))) as i32)) != 0) && (((((((*(s)).token).val) != ((TOK_DO as i32))) as i32)) != 0)) as i32)) != 0) && (((((((*(s)).token).val) != ((TOK_WHILE as i32))) as i32)) != 0)) as i32)) != 0 { 449 } else { 438 }; continue;
}
// C line 28809
451 => {
vm_block = 2; continue;
}
// C line 28808
452 => {
vm_block = if (js_parse_expect(s, (58 as i32))) != 0 { 451 } else { 450 }; continue;
}
// C line 28807
453 => {
vm_block = 2; continue;
}
// C line 28806
454 => {
vm_block = if (next_token(s)) != 0 { 453 } else { 452 }; continue;
}
// C line 28799
455 => {
vm_block = if !(be).is_null() { 459 } else { 454 }; continue;
}
// C line 28799
456 => {
let _ = { let assigned = (*(be)).prev; be = assigned; assigned };
vm_block = 455; continue;
}
// C line 28802
457 => {
vm_block = 2; continue;
}
// C line 28801
458 => {
let _ = js_parse_error(s, c"duplicate label name".as_ptr());
vm_block = 457; continue;
}
// C line 28800
459 => {
vm_block = if (((((*(be)).label_name) == (label_name)) as i32)) != 0 { 458 } else { 456 }; continue;
}
// C line 28799
460 => {
let _ = { let assigned = (*((*(s)).cur_func)).top_break; be = assigned; assigned };
vm_block = 455; continue;
}
// C line 28797
461 => {
let _ = { let assigned = JS_DupAtom(ctx, ((((*(s)).token).u).ident).atom); label_name = assigned; assigned };
vm_block = 460; continue;
}
// C line 28794
462 => {
vm_block = if (is_label(s)) != 0 { 461 } else { 438 }; continue;
}
// C line 28793
463 => {
let _ = { let assigned = (((0 as i32)) as JSAtom); label_name = assigned; assigned };
vm_block = 462; continue;
}
// C line 28787
464 => {
ctx = (*(s)).ctx;
vm_block = 463; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:29547. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_new_module_def(mut ctx: *mut JSContext, mut name: JSAtom) -> *mut JSModuleDef {
let mut vm_local_storage = Vec::<u64>::new();
let mut m: *mut JSModuleDef = core::ptr::null_mut();
let mut vm_block: usize = 17;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 29567
1 => {
return m;
}
// C line 29566
2 => {
let _ = list_add_tail(core::ptr::addr_of_mut!((*(m)).link), core::ptr::addr_of_mut!((*(ctx)).loaded_modules));
vm_block = 1; continue;
}
// C line 29565
3 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; (*(m)).private_value = assigned; assigned };
vm_block = 2; continue;
}
// C line 29564
4 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; *(((*(m)).resolving_funcs).as_mut_ptr()).offset(((1 as i32)) as isize) = assigned; assigned };
vm_block = 3; continue;
}
// C line 29563
5 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; *(((*(m)).resolving_funcs).as_mut_ptr()).offset(((0 as i32)) as isize) = assigned; assigned };
vm_block = 4; continue;
}
// C line 29562
6 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; (*(m)).promise = assigned; assigned };
vm_block = 5; continue;
}
// C line 29561
7 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; (*(m)).meta_obj = assigned; assigned };
vm_block = 6; continue;
}
// C line 29560
8 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; (*(m)).eval_exception = assigned; assigned };
vm_block = 7; continue;
}
// C line 29559
9 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; (*(m)).func_obj = assigned; assigned };
vm_block = 8; continue;
}
// C line 29558
10 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; (*(m)).module_ns = assigned; assigned };
vm_block = 9; continue;
}
// C line 29557
11 => {
let _ = { let assigned = name; (*(m)).module_name = assigned; assigned };
vm_block = 10; continue;
}
// C line 29556
12 => {
let _ = add_gc_object((*(ctx)).rt, core::ptr::addr_of_mut!((*(m)).header), (((JS_GC_OBJ_TYPE_MODULE as i32)) as JSGCObjectTypeEnum));
vm_block = 11; continue;
}
// C line 29555
13 => {
let _ = { let assigned = (1 as i32); (*(js_rc(((m) as *mut c_void)))).ref_count = assigned; assigned };
vm_block = 12; continue;
}
// C line 29553
14 => {
return core::ptr::null_mut::<JSModuleDef>();
}
// C line 29552
15 => {
let _ = JS_FreeAtom(ctx, name);
vm_block = 14; continue;
}
// C line 29551
16 => {
vm_block = if ((!(!(m).is_null()) as i32)) != 0 { 15 } else { 13 }; continue;
}
// C line 29550
17 => {
let _ = { let assigned = ((js_mallocz(ctx, (size_of::<JSModuleDef>() as usize))) as *mut JSModuleDef); m = assigned; assigned };
vm_block = 16; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:29650. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn add_req_module_entry(mut ctx: *mut JSContext, mut m: *mut JSModuleDef, mut module_name: JSAtom) -> i32 {
let mut vm_local_storage = Vec::<u64>::new();
let mut rme: *mut JSReqModuleEntry = core::ptr::null_mut();
let mut vm_block: usize = 7;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 29664
1 => {
return ((*(m)).req_module_entries_count).wrapping_sub((1 as i32));
}
// C line 29663
2 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; (*(rme)).attributes = assigned; assigned };
vm_block = 1; continue;
}
// C line 29662
3 => {
let _ = { let assigned = core::ptr::null_mut::<JSModuleDef>(); (*(rme)).module = assigned; assigned };
vm_block = 2; continue;
}
// C line 29661
4 => {
let _ = { let assigned = JS_DupAtom(ctx, module_name); (*(rme)).module_name = assigned; assigned };
vm_block = 3; continue;
}
// C line 29660
5 => {
let _ = { let assigned = core::ptr::addr_of_mut!(*((*(m)).req_module_entries).offset(({ let old = (*(m)).req_module_entries_count; (*(m)).req_module_entries_count = ((*(m)).req_module_entries_count).wrapping_add(1); old }) as isize)); rme = assigned; assigned };
vm_block = 4; continue;
}
// C line 29659
6 => {
return ((1 as i32)).wrapping_neg();
}
// C line 29655
7 => {
vm_block = if (js_resize_array(ctx, ((core::ptr::addr_of_mut!((*(m)).req_module_entries)) as *mut *mut c_void), (((size_of::<JSReqModuleEntry>() as usize)) as i32), core::ptr::addr_of_mut!((*(m)).req_module_entries_size), ((*(m)).req_module_entries_count).wrapping_add((1 as i32)))) != 0 { 6 } else { 5 }; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:29667. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn find_export_entry(mut ctx: *mut JSContext, mut m: *mut JSModuleDef, mut export_name: JSAtom) -> *mut JSExportEntry {
let mut vm_local_storage = Vec::<u64>::new();
let mut me: *mut JSExportEntry = core::ptr::null_mut();
let mut i: i32 = 0;
let mut vm_block: usize = 7;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 29677
1 => {
return core::ptr::null_mut::<JSExportEntry>();
}
// C line 29672
2 => {
vm_block = if ((((i) < ((*(m)).export_entries_count)) as i32)) != 0 { 6 } else { 1 }; continue;
}
// C line 29672
3 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 2; continue;
}
// C line 29675
4 => {
return me;
}
// C line 29674
5 => {
vm_block = if (((((*(me)).export_name) == (export_name)) as i32)) != 0 { 4 } else { 3 }; continue;
}
// C line 29673
6 => {
let _ = { let assigned = core::ptr::addr_of_mut!(*((*(m)).export_entries).offset((i) as isize)); me = assigned; assigned };
vm_block = 5; continue;
}
// C line 29672
7 => {
let _ = { let assigned = (0 as i32); i = assigned; assigned };
vm_block = 2; continue;
}
_ => std::process::abort(),
} }
}
