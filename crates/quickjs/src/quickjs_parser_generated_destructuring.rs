// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:26078. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_parse_expr_paren(mut s: *mut JSParseState) -> i32 {
let mut vm_local_storage = Vec::<u64>::new();
let mut vm_block: usize = 7;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 26086
1 => {
return (0 as i32);
}
// C line 26085
2 => {
return ((1 as i32)).wrapping_neg();
}
// C line 26084
3 => {
vm_block = if (js_parse_expect(s, (41 as i32))) != 0 { 2 } else { 1 }; continue;
}
// C line 26083
4 => {
return ((1 as i32)).wrapping_neg();
}
// C line 26082
5 => {
vm_block = if (js_parse_expr(s)) != 0 { 4 } else { 3 }; continue;
}
// C line 26081
6 => {
return ((1 as i32)).wrapping_neg();
}
// C line 26080
7 => {
vm_block = if (js_parse_expect(s, (40 as i32))) != 0 { 6 } else { 5 }; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:26133. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_emit_spread_code(mut s: *mut JSParseState, mut depth: i32) -> () {
let mut vm_local_storage = Vec::<u64>::new();
let mut label_rest_next: i32 = 0;
let mut label_rest_done: i32 = 0;
let mut vm_block: usize = 14;
loop { match vm_block {
// C line ?
0 => {
return ();
}
// C line 26156
1 => {
let _ = emit_op(s, (((OP_drop as i32)) as u8));
vm_block = 0; continue;
}
// C line 26155
2 => {
let _ = emit_op(s, (((OP_drop as i32)) as u8));
vm_block = 1; continue;
}
// C line 26153
3 => {
let _ = emit_label(s, label_rest_done);
vm_block = 2; continue;
}
// C line 26152
4 => {
let _ = emit_goto(s, (OP_goto as i32), label_rest_next);
vm_block = 3; continue;
}
// C line 26151
5 => {
let _ = emit_op(s, (((OP_inc as i32)) as u8));
vm_block = 4; continue;
}
// C line 26150
6 => {
let _ = emit_op(s, (((OP_define_array_el as i32)) as u8));
vm_block = 5; continue;
}
// C line 26148
7 => {
let _ = { let assigned = emit_goto(s, (OP_if_true as i32), ((1 as i32)).wrapping_neg()); label_rest_done = assigned; assigned };
vm_block = 6; continue;
}
// C line 26147
8 => {
let _ = emit_u8(s, ((((2 as i32)).wrapping_add(depth)) as u8));
vm_block = 7; continue;
}
// C line 26146
9 => {
let _ = emit_op(s, (((OP_for_of_next as i32)) as u8));
vm_block = 8; continue;
}
// C line 26145
10 => {
let _ = emit_label(s, { let assigned = new_label(s); label_rest_next = assigned; assigned });
vm_block = 9; continue;
}
// C line 26144
11 => {
let _ = emit_u32(s, (((0 as i32)) as u32));
vm_block = 10; continue;
}
// C line 26143
12 => {
let _ = emit_op(s, (((OP_push_i32 as i32)) as u8));
vm_block = 11; continue;
}
// C line 26142
13 => {
let _ = emit_u16(s, (((0 as i32)) as u16));
vm_block = 12; continue;
}
// C line 26141
14 => {
let _ = emit_op(s, (((OP_array_from as i32)) as u8));
vm_block = 13; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:26159. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_parse_check_duplicate_parameter(mut s: *mut JSParseState, mut name: JSAtom) -> i32 {
let mut vm_local_storage = Vec::<u64>::new();
let mut fd: *mut JSFunctionDef = core::ptr::null_mut();
let mut i: i32 = 0;
let mut vm_block: usize = 13;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 26175 labels: duplicate
1 => {
return js_parse_error(s, c"duplicate parameter names not allowed in this context".as_ptr());
}
// C line 26172
2 => {
return (0 as i32);
}
// C line 26168
3 => {
vm_block = if ((((i) < ((*(fd)).var_count)) as i32)) != 0 { 6 } else { 2 }; continue;
}
// C line 26168
4 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 3; continue;
}
// C line 26170
5 => {
vm_block = 1; continue;
}
// C line 26169
6 => {
vm_block = if (((((*((*(fd)).vars).offset((i) as isize)).var_name) == (name)) as i32)) != 0 { 5 } else { 4 }; continue;
}
// C line 26168
7 => {
let _ = { let assigned = (0 as i32); i = assigned; assigned };
vm_block = 3; continue;
}
// C line 26164
8 => {
vm_block = if ((((i) < ((*(fd)).arg_count)) as i32)) != 0 { 11 } else { 7 }; continue;
}
// C line 26164
9 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 8; continue;
}
// C line 26166
10 => {
vm_block = 1; continue;
}
// C line 26165
11 => {
vm_block = if (((((*((*(fd)).args).offset((i) as isize)).var_name) == (name)) as i32)) != 0 { 10 } else { 9 }; continue;
}
// C line 26164
12 => {
let _ = { let assigned = (0 as i32); i = assigned; assigned };
vm_block = 8; continue;
}
// C line 26162
13 => {
fd = (*(s)).cur_func;
vm_block = 12; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:26183. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn need_var_reference(mut s: *mut JSParseState, mut tok: i32) -> i32 {
let mut vm_local_storage = Vec::<u64>::new();
let mut fd: *mut JSFunctionDef = core::ptr::null_mut();
let mut vm_block: usize = 9;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 26194
1 => {
return (1 as i32);
}
// C line 26192
2 => {
return (0 as i32);
}
// C line 26191
3 => {
vm_block = if ((*(s)).is_module) != 0 { 2 } else { 1 }; continue;
}
// C line 26190
4 => {
return (0 as i32);
}
// C line 26189
5 => {
vm_block = if ((!(((*(fd)).is_global_var) != 0) as i32)) != 0 { 4 } else { 3 }; continue;
}
// C line 26188
6 => {
vm_block = if ((((((*(fd)).js_mode) as i32)) & (((1 as i32)).wrapping_shl(((0 as i32)) as u32)))) != 0 { 5 } else { 1 }; continue;
}
// C line 26187
7 => {
return (0 as i32);
}
// C line 26186
8 => {
vm_block = if ((((tok) != ((TOK_VAR as i32))) as i32)) != 0 { 7 } else { 6 }; continue;
}
// C line 26185
9 => {
fd = (*(s)).cur_func;
vm_block = 8; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:26197. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_parse_destructuring_var(mut s: *mut JSParseState, mut tok: i32, mut is_arg: i32) -> JSAtom {
let mut vm_local_storage = Vec::<u64>::new();
let mut name: JSAtom = 0;
let mut vm_block: usize = 11;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 26216
1 => {
return (((0 as i32)) as JSAtom);
}
// C line 26215 labels: fail
2 => {
let _ = JS_FreeAtom((*(s)).ctx, name);
vm_block = 1; continue;
}
// C line 26213
3 => {
return name;
}
// C line 26211
4 => {
vm_block = 2; continue;
}
// C line 26210
5 => {
vm_block = if (next_token(s)) != 0 { 4 } else { 3 }; continue;
}
// C line 26209
6 => {
vm_block = 2; continue;
}
// C line 26208
7 => {
vm_block = if (((((is_arg) != 0) && ((js_parse_check_duplicate_parameter(s, name)) != 0)) as i32)) != 0 { 6 } else { 5 }; continue;
}
// C line 26207
8 => {
let _ = { let assigned = JS_DupAtom((*(s)).ctx, ((((*(s)).token).u).ident).atom); name = assigned; assigned };
vm_block = 7; continue;
}
// C line 26205
9 => {
return (((0 as i32)) as JSAtom);
}
// C line 26204
10 => {
let _ = js_parse_error(s, c"invalid destructuring target".as_ptr());
vm_block = 9; continue;
}
// C line 26201
11 => {
vm_block = if ((((((!(((((((((((*(s)).token).val) == ((TOK_IDENT as i32))) as i32)) != 0) && (((!((((((*(s)).token).u).ident).is_reserved) != 0) as i32)) != 0)) as i32)) != 0) as i32)) != 0) || (((((((((((*((*(s)).cur_func)).js_mode) as i32)) & (((1 as i32)).wrapping_shl(((0 as i32)) as u32)))) != 0) && (((((((((((((*(s)).token).u).ident).atom) == ((((crate::quickjs_atom::JS_ATOM_eval as i32)) as JSAtom))) as i32)) != 0) || (((((((((*(s)).token).u).ident).atom) == ((((crate::quickjs_atom::JS_ATOM_arguments as i32)) as JSAtom))) as i32)) != 0)) as i32)) != 0)) as i32)) != 0)) as i32)) != 0 { 10 } else { 8 }; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:26221. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_parse_destructuring_element(mut s: *mut JSParseState, mut tok: i32, mut is_arg: i32, mut hasval: i32, mut has_ellipsis: i32, mut allow_initializer: i32, mut export_flag: i32) -> i32 {
let mut vm_local_storage = Vec::<u64>::new();
let mut label_parse: i32 = 0;
let mut label_assign: i32 = 0;
let mut label_done: i32 = 0;
let mut label_lvalue: i32 = 0;
let mut depth_lvalue: i32 = 0;
let mut start_addr: i32 = 0;
let mut assign_addr: i32 = 0;
let mut prop_name: JSAtom = 0;
let mut var_name: JSAtom = 0;
let mut opcode: i32 = 0;
let mut scope: i32 = 0;
let mut tok1: i32 = 0;
let mut skip_bits: i32 = 0;
let mut has_initializer: i32 = 0;
let mut prop_type: i32 = 0;
let mut label_hasval: i32 = 0;
let mut has_spread: i32 = 0;
let mut enum_depth: i32 = 0;
let mut block_env: BlockEnv = core::mem::zeroed();
let mut label_hasval_1: i32 = 0;
let mut vm_block: usize = 299;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 26687
1 => {
return ((1 as i32)).wrapping_neg();
}
// C line 26686 labels: var_error
2 => {
let _ = JS_FreeAtom((*(s)).ctx, var_name);
vm_block = 1; continue;
}
// C line 26684 labels: prop_error
3 => {
let _ = JS_FreeAtom((*(s)).ctx, prop_name);
vm_block = 2; continue;
}
// C line 26681
4 => {
return has_initializer;
}
// C line 26666
5 => {
let _ = { let assigned = (1 as i32); has_initializer = assigned; assigned };
vm_block = 4; continue;
}
// C line 26665
6 => {
let _ = emit_label(s, label_done);
vm_block = 5; continue;
}
// C line 26664
7 => {
let _ = emit_goto(s, (OP_goto as i32), label_assign);
vm_block = 6; continue;
}
// C line 26663
8 => {
return ((1 as i32)).wrapping_neg();
}
// C line 26662
9 => {
vm_block = if (js_parse_assign_expr(s)) != 0 { 8 } else { 7 }; continue;
}
// C line 26661
10 => {
let _ = emit_op(s, (((OP_drop as i32)) as u8));
vm_block = 9; continue;
}
// C line 26660
11 => {
vm_block = if (hasval) != 0 { 10 } else { 9 }; continue;
}
// C line 26659
12 => {
let _ = emit_label(s, label_parse);
vm_block = 11; continue;
}
// C line 26658
13 => {
return ((1 as i32)).wrapping_neg();
}
// C line 26657
14 => {
vm_block = if (next_token(s)) != 0 { 13 } else { 12 }; continue;
}
// C line 26656
15 => {
let _ = { let assigned = emit_goto(s, (OP_goto as i32), ((1 as i32)).wrapping_neg()); label_done = assigned; assigned };
vm_block = 14; continue;
}
// C line 26679
16 => {
let _ = { let assigned = (0 as i32); has_initializer = assigned; assigned };
vm_block = 4; continue;
}
// C line 26678
17 => {
let _ = { let old = (*((*((*(s)).cur_func)).label_slots).offset((label_parse) as isize)).ref_count; (*((*((*(s)).cur_func)).label_slots).offset((label_parse) as isize)).ref_count = ((*((*((*(s)).cur_func)).label_slots).offset((label_parse) as isize)).ref_count).wrapping_sub(1); old };
vm_block = 16; continue;
}
// C line 26676
18 => {
let _ = { let dst = ((((((*((*(s)).cur_func)).byte_code).buf).offset(((start_addr) as isize))) as *mut c_void)) as *mut u8; core::ptr::write_bytes(dst, ((OP_nop as i32)) as u8, ((((assign_addr).wrapping_sub(start_addr)) as usize)) as usize); dst as *mut c_void };
vm_block = 17; continue;
}
// C line 26673
19 => {
return ((1 as i32)).wrapping_neg();
}
// C line 26672
20 => {
let _ = js_parse_error(s, c"too complicated destructuring expression".as_ptr());
vm_block = 19; continue;
}
// C line 26671
21 => {
vm_block = if ((!((hasval) != 0) as i32)) != 0 { 20 } else { 18 }; continue;
}
// C line 26655
22 => {
vm_block = if ((((((((((*(s)).token).val) == ((61 as i32))) as i32)) != 0) && ((allow_initializer) != 0)) as i32)) != 0 { 15 } else { 21 }; continue;
}
// C line 26535
23 => {
return ((1 as i32)).wrapping_neg();
}
// C line 26534
24 => {
vm_block = if (next_token(s)) != 0 { 23 } else { 22 }; continue;
}
// C line 26532
25 => {
let _ = emit_op(s, (((OP_drop as i32)) as u8));
vm_block = 24; continue;
}
// C line 26531
26 => {
vm_block = if (has_ellipsis) != 0 { 25 } else { 24 }; continue;
}
// C line 26530
27 => {
let _ = emit_op(s, (((OP_drop as i32)) as u8));
vm_block = 26; continue;
}
// C line 26265
28 => {
vm_block = if ((((((*(s)).token).val) != ((125 as i32))) as i32)) != 0 { 196 } else { 27 }; continue;
}
// C line 26527
29 => {
return ((1 as i32)).wrapping_neg();
}
// C line 26526
30 => {
vm_block = if (js_parse_expect(s, (44 as i32))) != 0 { 29 } else { 28 }; continue;
}
// C line 26524
31 => {
vm_block = 27; continue;
}
// C line 26523
32 => {
vm_block = if ((((((*(s)).token).val) == ((125 as i32))) as i32)) != 0 { 31 } else { 30 }; continue;
}
// C line 26520
33 => {
let _ = put_lvalue(s, opcode, scope, var_name, label_lvalue, (((PUT_LVALUE_NOKEEP_DEPTH as i32)) as PutLValueEnum), (((((((tok) == ((TOK_CONST as i32))) as i32)) != 0) || (((((tok) == ((TOK_LET as i32))) as i32)) != 0)) as i32));
vm_block = 32; continue;
}
// C line 26517
34 => {
let _ = emit_label(s, label_hasval);
vm_block = 33; continue;
}
// C line 26516
35 => {
let _ = set_object_name(s, var_name);
vm_block = 34; continue;
}
// C line 26515
36 => {
vm_block = if ((((((((opcode) == ((OP_scope_get_var as i32))) as i32)) != 0) || (((((opcode) == ((OP_get_ref_value as i32))) as i32)) != 0)) as i32)) != 0 { 35 } else { 34 }; continue;
}
// C line 26514
37 => {
vm_block = 2; continue;
}
// C line 26513
38 => {
vm_block = if (js_parse_assign_expr(s)) != 0 { 37 } else { 36 }; continue;
}
// C line 26512
39 => {
let _ = emit_op(s, (((OP_drop as i32)) as u8));
vm_block = 38; continue;
}
// C line 26511
40 => {
vm_block = 2; continue;
}
// C line 26510
41 => {
vm_block = if (next_token(s)) != 0 { 40 } else { 39 }; continue;
}
// C line 26509
42 => {
let _ = { let assigned = emit_goto(s, (OP_if_false as i32), ((1 as i32)).wrapping_neg()); label_hasval = assigned; assigned };
vm_block = 41; continue;
}
// C line 26508
43 => {
let _ = emit_op(s, (((OP_strict_eq as i32)) as u8));
vm_block = 42; continue;
}
// C line 26507
44 => {
let _ = emit_op(s, (((OP_undefined as i32)) as u8));
vm_block = 43; continue;
}
// C line 26506
45 => {
let _ = emit_op(s, (((OP_dup as i32)) as u8));
vm_block = 44; continue;
}
// C line 26504
46 => {
vm_block = if ((((((*(s)).token).val) == ((61 as i32))) as i32)) != 0 { 45 } else { 33 }; continue;
}
// C line 26502
47 => {
let _ = { let assigned = (*((*(s)).cur_func)).scope_level; scope = assigned; assigned };
vm_block = 46; continue;
}
// C line 26500
48 => {
vm_block = 2; continue;
}
// C line 26498
49 => {
vm_block = if ((!(!(add_export_entry(s, (*((*(s)).cur_func)).module, var_name, var_name, (((JS_EXPORT_TYPE_LOCAL as i32)) as JSExportTypeEnum))).is_null()) as i32)) != 0 { 48 } else { 47 }; continue;
}
// C line 26497
50 => {
vm_block = if (export_flag) != 0 { 49 } else { 47 }; continue;
}
// C line 26496
51 => {
vm_block = 2; continue;
}
// C line 26495
52 => {
vm_block = if (js_define_var(s, var_name, tok)) != 0 { 51 } else { 50 }; continue;
}
// C line 26494 labels: set_val
53 => {
vm_block = if (tok) != 0 { 52 } else { 46 }; continue;
}
// C line 26447
54 => {
let _ = emit_op(s, (((OP_get_array_el as i32)) as u8));
vm_block = 53; continue;
}
// C line 26453
55 => {
let _ = emit_u32(s, prop_name);
vm_block = 53; continue;
}
// C line 26452
56 => {
let _ = emit_op(s, (((OP_get_field as i32)) as u8));
vm_block = 55; continue;
}
// C line 26443
57 => {
vm_block = if ((((prop_name) == ((((0 as i32)) as JSAtom))) as i32)) != 0 { 54 } else { 56 }; continue;
}
// C line 26386
58 => {
vm_block = 92; continue;
}
// C line 26385
59 => {
let _ = JS_FreeAtom((*(s)).ctx, var_name);
vm_block = 58; continue;
}
// C line 26384
60 => {
let _ = emit_u16(s, (((*((*(s)).cur_func)).scope_level) as u16));
vm_block = 59; continue;
}
// C line 26383
61 => {
let _ = emit_atom(s, var_name);
vm_block = 60; continue;
}
// C line 26382
62 => {
let _ = emit_op(s, (((OP_scope_get_var as i32)) as u8));
vm_block = 61; continue;
}
// C line 26392
63 => {
let _ = { let assigned = (0 as i32); depth_lvalue = assigned; assigned };
vm_block = 57; continue;
}
// C line 26391
64 => {
let _ = { let assigned = ((1 as i32)).wrapping_neg(); label_lvalue = assigned; assigned };
vm_block = 63; continue;
}
// C line 26390
65 => {
let _ = { let assigned = (*((*(s)).cur_func)).scope_level; scope = assigned; assigned };
vm_block = 64; continue;
}
// C line 26389
66 => {
let _ = { let assigned = (OP_scope_get_var as i32); opcode = assigned; assigned };
vm_block = 65; continue;
}
// C line 26380
67 => {
vm_block = if (need_var_reference(s, tok)) != 0 { 62 } else { 66 }; continue;
}
// C line 26379
68 => {
vm_block = 3; continue;
}
// C line 26378
69 => {
vm_block = if ((((var_name) == ((((0 as i32)) as JSAtom))) as i32)) != 0 { 68 } else { 67 }; continue;
}
// C line 26377
70 => {
let _ = { let assigned = js_parse_destructuring_var(s, tok, is_arg); var_name = assigned; assigned };
vm_block = 69; continue;
}
// C line 26420
71 => {
let _ = std::process::abort();
vm_block = 57; continue;
}
// C line 26418
72 => {
vm_block = 57; continue;
}
// C line 26417
73 => {
let _ = emit_op(s, (((OP_rot5l as i32)) as u8));
vm_block = 72; continue;
}
// C line 26416
74 => {
let _ = emit_op(s, (((OP_rot5l as i32)) as u8));
vm_block = 73; continue;
}
// C line 26413
75 => {
vm_block = 57; continue;
}
// C line 26412
76 => {
let _ = emit_op(s, (((OP_swap2 as i32)) as u8));
vm_block = 75; continue;
}
// C line 26409
77 => {
vm_block = 57; continue;
}
// C line 26408
78 => {
let _ = emit_op(s, (((OP_rot3r as i32)) as u8));
vm_block = 77; continue;
}
// C line 26405
79 => {
vm_block = 57; continue;
}
// C line 26403
80 => {
vm_block = match depth_lvalue { x if x == (3 as i32) => 74, x if x == (2 as i32) => 76, x if x == (1 as i32) => 78, x if x == (0 as i32) => 79, _ => 71, }; continue;
}
// C line 26439
81 => {
let _ = std::process::abort();
vm_block = 57; continue;
}
// C line 26437
82 => {
vm_block = 57; continue;
}
// C line 26436
83 => {
let _ = emit_op(s, (((OP_rot4l as i32)) as u8));
vm_block = 82; continue;
}
// C line 26433
84 => {
vm_block = 57; continue;
}
// C line 26432
85 => {
let _ = emit_op(s, (((OP_rot3l as i32)) as u8));
vm_block = 84; continue;
}
// C line 26429
86 => {
vm_block = 57; continue;
}
// C line 26428
87 => {
let _ = emit_op(s, (((OP_swap as i32)) as u8));
vm_block = 86; continue;
}
// C line 26425
88 => {
vm_block = 57; continue;
}
// C line 26423
89 => {
vm_block = match depth_lvalue { x if x == (3 as i32) => 83, x if x == (2 as i32) => 85, x if x == (1 as i32) => 87, x if x == (0 as i32) => 88, _ => 81, }; continue;
}
// C line 26402
90 => {
vm_block = if ((((prop_name) == ((((0 as i32)) as JSAtom))) as i32)) != 0 { 80 } else { 89 }; continue;
}
// C line 26400
91 => {
vm_block = 3; continue;
}
// C line 26398 labels: lvalue1
92 => {
vm_block = if (get_lvalue(s, core::ptr::addr_of_mut!(opcode), core::ptr::addr_of_mut!(scope), core::ptr::addr_of_mut!(var_name), core::ptr::addr_of_mut!(label_lvalue), core::ptr::addr_of_mut!(depth_lvalue), (0 as i32), (123 as i32))) != 0 { 91 } else { 90 }; continue;
}
// C line 26396
93 => {
vm_block = 3; continue;
}
// C line 26395
94 => {
vm_block = if (js_parse_left_hand_side_expr(s)) != 0 { 93 } else { 92 }; continue;
}
// C line 26376
95 => {
vm_block = if (tok) != 0 { 70 } else { 94 }; continue;
}
// C line 26363
96 => {
let _ = emit_op(s, (((OP_dup1 as i32)) as u8));
vm_block = 95; continue;
}
// C line 26360
97 => {
let _ = emit_op(s, (((OP_perm3 as i32)) as u8));
vm_block = 96; continue;
}
// C line 26359
98 => {
let _ = emit_op(s, (((OP_define_array_el as i32)) as u8));
vm_block = 97; continue;
}
// C line 26358
99 => {
let _ = emit_op(s, (((OP_null as i32)) as u8));
vm_block = 98; continue;
}
// C line 26357
100 => {
let _ = emit_op(s, (((OP_perm3 as i32)) as u8));
vm_block = 99; continue;
}
// C line 26355
101 => {
vm_block = if (has_ellipsis) != 0 { 100 } else { 96 }; continue;
}
// C line 26354
102 => {
let _ = emit_op(s, (((OP_to_propkey as i32)) as u8));
vm_block = 101; continue;
}
// C line 26374
103 => {
let _ = emit_op(s, (((OP_dup as i32)) as u8));
vm_block = 95; continue;
}
// C line 26371
104 => {
let _ = emit_op(s, (((OP_swap as i32)) as u8));
vm_block = 103; continue;
}
// C line 26370
105 => {
let _ = emit_atom(s, prop_name);
vm_block = 104; continue;
}
// C line 26369
106 => {
let _ = emit_op(s, (((OP_define_field as i32)) as u8));
vm_block = 105; continue;
}
// C line 26368
107 => {
let _ = emit_op(s, (((OP_null as i32)) as u8));
vm_block = 106; continue;
}
// C line 26367
108 => {
let _ = emit_op(s, (((OP_swap as i32)) as u8));
vm_block = 107; continue;
}
// C line 26365
109 => {
vm_block = if (has_ellipsis) != 0 { 108 } else { 103 }; continue;
}
// C line 26353
110 => {
vm_block = if ((((prop_name) == ((((0 as i32)) as JSAtom))) as i32)) != 0 { 102 } else { 109 }; continue;
}
// C line 26351
111 => {
vm_block = 28; continue;
}
// C line 26350
112 => {
return ((1 as i32)).wrapping_neg();
}
// C line 26349
113 => {
vm_block = if (js_parse_expect(s, (44 as i32))) != 0 { 112 } else { 111 }; continue;
}
// C line 26347
114 => {
vm_block = 27; continue;
}
// C line 26346
115 => {
vm_block = if ((((((*(s)).token).val) == ((125 as i32))) as i32)) != 0 { 114 } else { 113 }; continue;
}
// C line 26345
116 => {
return ((1 as i32)).wrapping_neg();
}
// C line 26344
117 => {
vm_block = if ((((js_parse_destructuring_element(s, tok, is_arg, (1 as i32), ((1 as i32)).wrapping_neg(), (1 as i32), export_flag)) < ((0 as i32))) as i32)) != 0 { 116 } else { 115 }; continue;
}
// C line 26329
118 => {
let _ = emit_op(s, (((OP_get_array_el2 as i32)) as u8));
vm_block = 117; continue;
}
// C line 26326
119 => {
let _ = emit_op(s, (((OP_perm3 as i32)) as u8));
vm_block = 118; continue;
}
// C line 26325
120 => {
let _ = emit_op(s, (((OP_define_array_el as i32)) as u8));
vm_block = 119; continue;
}
// C line 26324
121 => {
let _ = emit_op(s, (((OP_null as i32)) as u8));
vm_block = 120; continue;
}
// C line 26323
122 => {
let _ = emit_op(s, (((OP_perm3 as i32)) as u8));
vm_block = 121; continue;
}
// C line 26322
123 => {
let _ = emit_op(s, (((OP_to_propkey as i32)) as u8));
vm_block = 122; continue;
}
// C line 26320
124 => {
vm_block = if (has_ellipsis) != 0 { 123 } else { 118 }; continue;
}
// C line 26342
125 => {
let _ = emit_u32(s, prop_name);
vm_block = 117; continue;
}
// C line 26341
126 => {
let _ = emit_op(s, (((OP_get_field2 as i32)) as u8));
vm_block = 125; continue;
}
// C line 26338
127 => {
let _ = emit_op(s, (((OP_swap as i32)) as u8));
vm_block = 126; continue;
}
// C line 26337
128 => {
let _ = emit_atom(s, prop_name);
vm_block = 127; continue;
}
// C line 26336
129 => {
let _ = emit_op(s, (((OP_define_field as i32)) as u8));
vm_block = 128; continue;
}
// C line 26335
130 => {
let _ = emit_op(s, (((OP_null as i32)) as u8));
vm_block = 129; continue;
}
// C line 26334
131 => {
let _ = emit_op(s, (((OP_swap as i32)) as u8));
vm_block = 130; continue;
}
// C line 26332
132 => {
vm_block = if (has_ellipsis) != 0 { 131 } else { 126 }; continue;
}
// C line 26318
133 => {
vm_block = if ((((prop_name) == ((((0 as i32)) as JSAtom))) as i32)) != 0 { 124 } else { 132 }; continue;
}
// C line 26315
134 => {
vm_block = if ((((((((((((((*(s)).token).val) == ((91 as i32))) as i32)) != 0) || (((((((*(s)).token).val) == ((123 as i32))) as i32)) != 0)) as i32)) != 0) && ((((((((((((({ let assigned = js_parse_skip_parens_token(s, core::ptr::addr_of_mut!(skip_bits), (0 as i32)); tok1 = assigned; assigned }) == ((44 as i32))) as i32)) != 0) || (((((tok1) == ((61 as i32))) as i32)) != 0)) as i32)) != 0) || (((((tok1) == ((125 as i32))) as i32)) != 0)) as i32)) != 0)) as i32)) != 0 { 133 } else { 110 }; continue;
}
// C line 26314
135 => {
vm_block = 3; continue;
}
// C line 26313
136 => {
vm_block = if (next_token(s)) != 0 { 135 } else { 134 }; continue;
}
// C line 26479
137 => {
vm_block = 92; continue;
}
// C line 26478
138 => {
let _ = emit_u16(s, (((*((*(s)).cur_func)).scope_level) as u16));
vm_block = 137; continue;
}
// C line 26477
139 => {
let _ = emit_atom(s, prop_name);
vm_block = 138; continue;
}
// C line 26476
140 => {
let _ = emit_op(s, (((OP_scope_get_var as i32)) as u8));
vm_block = 139; continue;
}
// C line 26475
141 => {
let _ = emit_op(s, (((OP_dup as i32)) as u8));
vm_block = 140; continue;
}
// C line 26490
142 => {
let _ = emit_u32(s, prop_name);
vm_block = 53; continue;
}
// C line 26489
143 => {
let _ = emit_op(s, (((OP_get_field2 as i32)) as u8));
vm_block = 142; continue;
}
// C line 26486
144 => {
let _ = { let assigned = (0 as i32); depth_lvalue = assigned; assigned };
vm_block = 143; continue;
}
// C line 26485
145 => {
let _ = { let assigned = ((1 as i32)).wrapping_neg(); label_lvalue = assigned; assigned };
vm_block = 144; continue;
}
// C line 26484
146 => {
let _ = { let assigned = (*((*(s)).cur_func)).scope_level; scope = assigned; assigned };
vm_block = 145; continue;
}
// C line 26483
147 => {
let _ = { let assigned = (OP_scope_get_var as i32); opcode = assigned; assigned };
vm_block = 146; continue;
}
// C line 26482
148 => {
let _ = { let assigned = JS_DupAtom((*(s)).ctx, prop_name); var_name = assigned; assigned };
vm_block = 147; continue;
}
// C line 26472
149 => {
vm_block = if ((((((!((tok) != 0) as i32)) != 0) || ((need_var_reference(s, tok)) != 0)) as i32)) != 0 { 141 } else { 148 }; continue;
}
// C line 26470
150 => {
let _ = emit_op(s, (((OP_swap as i32)) as u8));
vm_block = 149; continue;
}
// C line 26469
151 => {
let _ = emit_atom(s, prop_name);
vm_block = 150; continue;
}
// C line 26468
152 => {
let _ = emit_op(s, (((OP_define_field as i32)) as u8));
vm_block = 151; continue;
}
// C line 26467
153 => {
let _ = emit_op(s, (((OP_null as i32)) as u8));
vm_block = 152; continue;
}
// C line 26466
154 => {
let _ = emit_op(s, (((OP_swap as i32)) as u8));
vm_block = 153; continue;
}
// C line 26464
155 => {
vm_block = if (has_ellipsis) != 0 { 154 } else { 149 }; continue;
}
// C line 26462
156 => {
vm_block = 3; continue;
}
// C line 26461
157 => {
let _ = js_parse_error(s, c"invalid destructuring target".as_ptr());
vm_block = 156; continue;
}
// C line 26459
158 => {
vm_block = if ((((((((((*((*(s)).cur_func)).js_mode) as i32)) & (((1 as i32)).wrapping_shl(((0 as i32)) as u32)))) != 0) && (((((((((prop_name) == ((((crate::quickjs_atom::JS_ATOM_eval as i32)) as JSAtom))) as i32)) != 0) || (((((prop_name) == ((((crate::quickjs_atom::JS_ATOM_arguments as i32)) as JSAtom))) as i32)) != 0)) as i32)) != 0)) as i32)) != 0 { 157 } else { 155 }; continue;
}
// C line 26458
159 => {
vm_block = 3; continue;
}
// C line 26457
160 => {
vm_block = if (((((is_arg) != 0) && ((js_parse_check_duplicate_parameter(s, prop_name)) != 0)) as i32)) != 0 { 159 } else { 158 }; continue;
}
// C line 26312
161 => {
vm_block = if ((((prop_type) == ((0 as i32))) as i32)) != 0 { 136 } else { 160 }; continue;
}
// C line 26311
162 => {
let _ = { let assigned = (((0 as i32)) as JSAtom); var_name = assigned; assigned };
vm_block = 161; continue;
}
// C line 26310
163 => {
return ((1 as i32)).wrapping_neg();
}
// C line 26309
164 => {
vm_block = if ((((prop_type) < ((0 as i32))) as i32)) != 0 { 163 } else { 162 }; continue;
}
// C line 26308
165 => {
let _ = { let assigned = js_parse_property_name(s, core::ptr::addr_of_mut!(prop_name), (0 as i32), (1 as i32), (0 as i32)); prop_type = assigned; assigned };
vm_block = 164; continue;
}
// C line 26306
166 => {
vm_block = 53; continue;
}
// C line 26305
167 => {
let _ = emit_u8(s, (((((((0 as i32)) | (((depth_lvalue).wrapping_add((1 as i32))).wrapping_shl(((2 as i32)) as u32)))) | (((depth_lvalue).wrapping_add((2 as i32))).wrapping_shl(((5 as i32)) as u32)))) as u8));
vm_block = 166; continue;
}
// C line 26304
168 => {
let _ = emit_op(s, (((OP_copy_data_properties as i32)) as u8));
vm_block = 167; continue;
}
// C line 26303
169 => {
let _ = emit_op(s, (((OP_object as i32)) as u8));
vm_block = 168; continue;
}
// C line 26301
170 => {
vm_block = 2; continue;
}
// C line 26300
171 => {
let _ = js_parse_error(s, c"assignment rest property must be last".as_ptr());
vm_block = 170; continue;
}
// C line 26299
172 => {
vm_block = if ((((((*(s)).token).val) != ((125 as i32))) as i32)) != 0 { 171 } else { 169 }; continue;
}
// C line 26284
173 => {
vm_block = 187; continue;
}
// C line 26283
174 => {
let _ = JS_FreeAtom((*(s)).ctx, var_name);
vm_block = 173; continue;
}
// C line 26282
175 => {
let _ = emit_u16(s, (((*((*(s)).cur_func)).scope_level) as u16));
vm_block = 174; continue;
}
// C line 26281
176 => {
let _ = emit_atom(s, var_name);
vm_block = 175; continue;
}
// C line 26280
177 => {
let _ = emit_op(s, (((OP_scope_get_var as i32)) as u8));
vm_block = 176; continue;
}
// C line 26289
178 => {
let _ = { let assigned = (0 as i32); depth_lvalue = assigned; assigned };
vm_block = 172; continue;
}
// C line 26288
179 => {
let _ = { let assigned = ((1 as i32)).wrapping_neg(); label_lvalue = assigned; assigned };
vm_block = 178; continue;
}
// C line 26287
180 => {
let _ = { let assigned = (*((*(s)).cur_func)).scope_level; scope = assigned; assigned };
vm_block = 179; continue;
}
// C line 26286
181 => {
let _ = { let assigned = (OP_scope_get_var as i32); opcode = assigned; assigned };
vm_block = 180; continue;
}
// C line 26278
182 => {
vm_block = if (need_var_reference(s, tok)) != 0 { 177 } else { 181 }; continue;
}
// C line 26277
183 => {
return ((1 as i32)).wrapping_neg();
}
// C line 26276
184 => {
vm_block = if ((((var_name) == ((((0 as i32)) as JSAtom))) as i32)) != 0 { 183 } else { 182 }; continue;
}
// C line 26275
185 => {
let _ = { let assigned = js_parse_destructuring_var(s, tok, is_arg); var_name = assigned; assigned };
vm_block = 184; continue;
}
// C line 26297
186 => {
return ((1 as i32)).wrapping_neg();
}
// C line 26295 labels: lvalue0
187 => {
vm_block = if (get_lvalue(s, core::ptr::addr_of_mut!(opcode), core::ptr::addr_of_mut!(scope), core::ptr::addr_of_mut!(var_name), core::ptr::addr_of_mut!(label_lvalue), core::ptr::addr_of_mut!(depth_lvalue), (0 as i32), (123 as i32))) != 0 { 186 } else { 172 }; continue;
}
// C line 26293
188 => {
return ((1 as i32)).wrapping_neg();
}
// C line 26292
189 => {
vm_block = if (js_parse_left_hand_side_expr(s)) != 0 { 188 } else { 187 }; continue;
}
// C line 26274
190 => {
vm_block = if (tok) != 0 { 185 } else { 189 }; continue;
}
// C line 26273
191 => {
return ((1 as i32)).wrapping_neg();
}
// C line 26272
192 => {
vm_block = if (next_token(s)) != 0 { 191 } else { 190 }; continue;
}
// C line 26270
193 => {
return ((1 as i32)).wrapping_neg();
}
// C line 26269
194 => {
let _ = JS_ThrowInternalError((*(s)).ctx, c"unexpected ellipsis token".as_ptr());
vm_block = 193; continue;
}
// C line 26268
195 => {
vm_block = if ((!((has_ellipsis) != 0) as i32)) != 0 { 194 } else { 192 }; continue;
}
// C line 26267
196 => {
vm_block = if ((((((*(s)).token).val) == ((TOK_ELLIPSIS as i32))) as i32)) != 0 { 195 } else { 165 }; continue;
}
// C line 26263
197 => {
let _ = emit_op(s, (((OP_swap as i32)) as u8));
vm_block = 28; continue;
}
// C line 26262
198 => {
let _ = emit_op(s, (((OP_object as i32)) as u8));
vm_block = 197; continue;
}
// C line 26260
199 => {
vm_block = if (has_ellipsis) != 0 { 198 } else { 28 }; continue;
}
// C line 26259
200 => {
let _ = emit_op(s, (((OP_to_object as i32)) as u8));
vm_block = 199; continue;
}
// C line 26257
201 => {
return ((1 as i32)).wrapping_neg();
}
// C line 26256
202 => {
vm_block = if (next_token(s)) != 0 { 201 } else { 200 }; continue;
}
// C line 26651
203 => {
return ((1 as i32)).wrapping_neg();
}
// C line 26650
204 => {
vm_block = if (next_token(s)) != 0 { 203 } else { 22 }; continue;
}
// C line 26649
205 => {
let _ = pop_break_entry((*(s)).cur_func);
vm_block = 204; continue;
}
// C line 26648
206 => {
let _ = emit_op(s, (((OP_iterator_close as i32)) as u8));
vm_block = 205; continue;
}
// C line 26550
207 => {
vm_block = if ((((((*(s)).token).val) != ((93 as i32))) as i32)) != 0 { 274 } else { 206 }; continue;
}
// C line 26644
208 => {
return ((1 as i32)).wrapping_neg();
}
// C line 26643
209 => {
vm_block = if (js_parse_expect(s, (44 as i32))) != 0 { 208 } else { 207 }; continue;
}
// C line 26641
210 => {
return js_parse_error(s, c"rest element must be the last one".as_ptr());
}
// C line 26640
211 => {
vm_block = if (has_spread) != 0 { 210 } else { 209 }; continue;
}
// C line 26639
212 => {
vm_block = 206; continue;
}
// C line 26638
213 => {
vm_block = if ((((((*(s)).token).val) == ((93 as i32))) as i32)) != 0 { 212 } else { 211 }; continue;
}
// C line 26564
214 => {
let _ = emit_op(s, (((OP_drop as i32)) as u8));
vm_block = 213; continue;
}
// C line 26563
215 => {
let _ = emit_op(s, (((OP_drop as i32)) as u8));
vm_block = 214; continue;
}
// C line 26562
216 => {
let _ = emit_u8(s, (((0 as i32)) as u8));
vm_block = 215; continue;
}
// C line 26561
217 => {
let _ = emit_op(s, (((OP_for_of_next as i32)) as u8));
vm_block = 216; continue;
}
// C line 26578
218 => {
return ((1 as i32)).wrapping_neg();
}
// C line 26577
219 => {
vm_block = if ((((js_parse_destructuring_element(s, tok, is_arg, (1 as i32), ((skip_bits) & (((1 as i32)).wrapping_shl(((1 as i32)) as u32))), (1 as i32), export_flag)) < ((0 as i32))) as i32)) != 0 { 218 } else { 213 }; continue;
}
// C line 26571
220 => {
let _ = js_emit_spread_code(s, (0 as i32));
vm_block = 219; continue;
}
// C line 26570
221 => {
return js_parse_error(s, c"rest element cannot have a default value".as_ptr());
}
// C line 26569
222 => {
vm_block = if ((((tok1) == ((61 as i32))) as i32)) != 0 { 221 } else { 220 }; continue;
}
// C line 26575
223 => {
let _ = emit_op(s, (((OP_drop as i32)) as u8));
vm_block = 219; continue;
}
// C line 26574
224 => {
let _ = emit_u8(s, (((0 as i32)) as u8));
vm_block = 223; continue;
}
// C line 26573
225 => {
let _ = emit_op(s, (((OP_for_of_next as i32)) as u8));
vm_block = 224; continue;
}
// C line 26568
226 => {
vm_block = if (has_spread) != 0 { 222 } else { 225 }; continue;
}
// C line 26634
227 => {
let _ = put_lvalue(s, opcode, scope, var_name, label_lvalue, (((PUT_LVALUE_NOKEEP_DEPTH as i32)) as PutLValueEnum), (((((((tok) == ((TOK_CONST as i32))) as i32)) != 0) || (((((tok) == ((TOK_LET as i32))) as i32)) != 0)) as i32));
vm_block = 213; continue;
}
// C line 26631
228 => {
let _ = emit_label(s, label_hasval_1);
vm_block = 227; continue;
}
// C line 26630
229 => {
let _ = set_object_name(s, var_name);
vm_block = 228; continue;
}
// C line 26629
230 => {
vm_block = if ((((((((opcode) == ((OP_scope_get_var as i32))) as i32)) != 0) || (((((opcode) == ((OP_get_ref_value as i32))) as i32)) != 0)) as i32)) != 0 { 229 } else { 228 }; continue;
}
// C line 26628
231 => {
vm_block = 2; continue;
}
// C line 26627
232 => {
vm_block = if (js_parse_assign_expr(s)) != 0 { 231 } else { 230 }; continue;
}
// C line 26626
233 => {
let _ = emit_op(s, (((OP_drop as i32)) as u8));
vm_block = 232; continue;
}
// C line 26625
234 => {
vm_block = 2; continue;
}
// C line 26624
235 => {
vm_block = if (next_token(s)) != 0 { 234 } else { 233 }; continue;
}
// C line 26623
236 => {
let _ = { let assigned = emit_goto(s, (OP_if_false as i32), ((1 as i32)).wrapping_neg()); label_hasval_1 = assigned; assigned };
vm_block = 235; continue;
}
// C line 26622
237 => {
let _ = emit_op(s, (((OP_strict_eq as i32)) as u8));
vm_block = 236; continue;
}
// C line 26621
238 => {
let _ = emit_op(s, (((OP_undefined as i32)) as u8));
vm_block = 237; continue;
}
// C line 26620
239 => {
let _ = emit_op(s, (((OP_dup as i32)) as u8));
vm_block = 238; continue;
}
// C line 26617
240 => {
vm_block = if ((((((((((*(s)).token).val) == ((61 as i32))) as i32)) != 0) && (((!((has_spread) != 0) as i32)) != 0)) as i32)) != 0 { 239 } else { 227 }; continue;
}
// C line 26611
241 => {
let _ = js_emit_spread_code(s, enum_depth);
vm_block = 240; continue;
}
// C line 26615
242 => {
let _ = emit_op(s, (((OP_drop as i32)) as u8));
vm_block = 240; continue;
}
// C line 26614
243 => {
let _ = emit_u8(s, ((enum_depth) as u8));
vm_block = 242; continue;
}
// C line 26613
244 => {
let _ = emit_op(s, (((OP_for_of_next as i32)) as u8));
vm_block = 243; continue;
}
// C line 26610
245 => {
vm_block = if (has_spread) != 0 { 241 } else { 244 }; continue;
}
// C line 26593
246 => {
vm_block = 262; continue;
}
// C line 26592
247 => {
let _ = JS_FreeAtom((*(s)).ctx, var_name);
vm_block = 246; continue;
}
// C line 26591
248 => {
let _ = emit_u16(s, (((*((*(s)).cur_func)).scope_level) as u16));
vm_block = 247; continue;
}
// C line 26590
249 => {
let _ = emit_atom(s, var_name);
vm_block = 248; continue;
}
// C line 26589
250 => {
let _ = emit_op(s, (((OP_scope_get_var as i32)) as u8));
vm_block = 249; continue;
}
// C line 26599
251 => {
let _ = { let assigned = (0 as i32); enum_depth = assigned; assigned };
vm_block = 245; continue;
}
// C line 26598
252 => {
let _ = { let assigned = ((1 as i32)).wrapping_neg(); label_lvalue = assigned; assigned };
vm_block = 251; continue;
}
// C line 26597
253 => {
let _ = { let assigned = (*((*(s)).cur_func)).scope_level; scope = assigned; assigned };
vm_block = 252; continue;
}
// C line 26596
254 => {
let _ = { let assigned = (OP_scope_get_var as i32); opcode = assigned; assigned };
vm_block = 253; continue;
}
// C line 26587
255 => {
vm_block = if (need_var_reference(s, tok)) != 0 { 250 } else { 254 }; continue;
}
// C line 26586
256 => {
vm_block = 2; continue;
}
// C line 26585
257 => {
vm_block = if (js_define_var(s, var_name, tok)) != 0 { 256 } else { 255 }; continue;
}
// C line 26584
258 => {
vm_block = 2; continue;
}
// C line 26583
259 => {
vm_block = if ((((var_name) == ((((0 as i32)) as JSAtom))) as i32)) != 0 { 258 } else { 257 }; continue;
}
// C line 26582
260 => {
let _ = { let assigned = js_parse_destructuring_var(s, tok, is_arg); var_name = assigned; assigned };
vm_block = 259; continue;
}
// C line 26607
261 => {
return ((1 as i32)).wrapping_neg();
}
// C line 26605 labels: lvalue2
262 => {
vm_block = if (get_lvalue(s, core::ptr::addr_of_mut!(opcode), core::ptr::addr_of_mut!(scope), core::ptr::addr_of_mut!(var_name), core::ptr::addr_of_mut!(label_lvalue), core::ptr::addr_of_mut!(enum_depth), (0 as i32), (91 as i32))) != 0 { 261 } else { 245 }; continue;
}
// C line 26603
263 => {
return ((1 as i32)).wrapping_neg();
}
// C line 26602
264 => {
vm_block = if (js_parse_left_hand_side_expr(s)) != 0 { 263 } else { 262 }; continue;
}
// C line 26581
265 => {
vm_block = if (tok) != 0 { 260 } else { 264 }; continue;
}
// C line 26580
266 => {
let _ = { let assigned = (((0 as i32)) as JSAtom); var_name = assigned; assigned };
vm_block = 265; continue;
}
// C line 26565
267 => {
vm_block = if ((((((((((((((*(s)).token).val) == ((91 as i32))) as i32)) != 0) || (((((((*(s)).token).val) == ((123 as i32))) as i32)) != 0)) as i32)) != 0) && ((((((((((((({ let assigned = js_parse_skip_parens_token(s, core::ptr::addr_of_mut!(skip_bits), (0 as i32)); tok1 = assigned; assigned }) == ((44 as i32))) as i32)) != 0) || (((((tok1) == ((61 as i32))) as i32)) != 0)) as i32)) != 0) || (((((tok1) == ((93 as i32))) as i32)) != 0)) as i32)) != 0)) as i32)) != 0 { 226 } else { 266 }; continue;
}
// C line 26559
268 => {
vm_block = if ((((((*(s)).token).val) == ((44 as i32))) as i32)) != 0 { 217 } else { 267 }; continue;
}
// C line 26557
269 => {
let _ = { let assigned = (1 as i32); has_spread = assigned; assigned };
vm_block = 268; continue;
}
// C line 26556
270 => {
return js_parse_error(s, c"missing binding pattern...".as_ptr());
}
// C line 26555
271 => {
vm_block = if ((((((((((*(s)).token).val) == ((44 as i32))) as i32)) != 0) || (((((((*(s)).token).val) == ((93 as i32))) as i32)) != 0)) as i32)) != 0 { 270 } else { 269 }; continue;
}
// C line 26554
272 => {
return ((1 as i32)).wrapping_neg();
}
// C line 26553
273 => {
vm_block = if (next_token(s)) != 0 { 272 } else { 271 }; continue;
}
// C line 26552
274 => {
vm_block = if ((((((*(s)).token).val) == ((TOK_ELLIPSIS as i32))) as i32)) != 0 { 273 } else { 268 }; continue;
}
// C line 26549
275 => {
let _ = { let assigned = (0 as i32); has_spread = assigned; assigned };
vm_block = 207; continue;
}
// C line 26548
276 => {
let _ = emit_op(s, (((OP_for_of_start as i32)) as u8));
vm_block = 275; continue;
}
// C line 26547
277 => {
let _ = { let assigned = (((1 as i32)) as u8); (block_env).set_has_iterator((assigned) as _); assigned };
vm_block = 276; continue;
}
// C line 26545
278 => {
let _ = push_break_entry((*(s)).cur_func, core::ptr::addr_of_mut!(block_env), (((0 as i32)) as JSAtom), ((1 as i32)).wrapping_neg(), ((1 as i32)).wrapping_neg(), (2 as i32));
vm_block = 277; continue;
}
// C line 26542
279 => {
return ((1 as i32)).wrapping_neg();
}
// C line 26541
280 => {
vm_block = if (next_token(s)) != 0 { 279 } else { 278 }; continue;
}
// C line 26653
281 => {
return js_parse_error(s, c"invalid assignment syntax".as_ptr());
}
// C line 26536
282 => {
vm_block = if ((((((*(s)).token).val) == ((91 as i32))) as i32)) != 0 { 280 } else { 281 }; continue;
}
// C line 26255
283 => {
vm_block = if ((((((*(s)).token).val) == ((123 as i32))) as i32)) != 0 { 202 } else { 282 }; continue;
}
// C line 26254
284 => {
let _ = { let assigned = ((((*((*(s)).cur_func)).byte_code).size) as i32); assign_addr = assigned; assigned };
vm_block = 283; continue;
}
// C line 26247
285 => {
let _ = emit_label(s, label_assign);
vm_block = 284; continue;
}
// C line 26246
286 => {
let _ = emit_goto(s, (OP_if_true as i32), label_parse);
vm_block = 285; continue;
}
// C line 26245
287 => {
let _ = emit_op(s, (((OP_strict_eq as i32)) as u8));
vm_block = 286; continue;
}
// C line 26244
288 => {
let _ = emit_op(s, (((OP_undefined as i32)) as u8));
vm_block = 287; continue;
}
// C line 26243
289 => {
let _ = emit_op(s, (((OP_dup as i32)) as u8));
vm_block = 288; continue;
}
// C line 26252
290 => {
let _ = emit_op(s, (((OP_dup as i32)) as u8));
vm_block = 284; continue;
}
// C line 26250
291 => {
let _ = emit_label(s, label_assign);
vm_block = 290; continue;
}
// C line 26249
292 => {
let _ = emit_goto(s, (OP_goto as i32), label_parse);
vm_block = 291; continue;
}
// C line 26241
293 => {
vm_block = if (hasval) != 0 { 289 } else { 292 }; continue;
}
// C line 26240
294 => {
let _ = { let assigned = ((((*((*(s)).cur_func)).byte_code).size) as i32); start_addr = assigned; assigned };
vm_block = 293; continue;
}
// C line 26238
295 => {
let _ = { let assigned = new_label(s); label_assign = assigned; assigned };
vm_block = 294; continue;
}
// C line 26237
296 => {
let _ = { let assigned = new_label(s); label_parse = assigned; assigned };
vm_block = 295; continue;
}
// C line 26234
297 => {
let _ = { let assigned = ((skip_bits) & (((1 as i32)).wrapping_shl(((1 as i32)) as u32))); has_ellipsis = assigned; assigned };
vm_block = 296; continue;
}
// C line 26233
298 => {
let _ = js_parse_skip_parens_token(s, core::ptr::addr_of_mut!(skip_bits), (0 as i32));
vm_block = 297; continue;
}
// C line 26231
299 => {
vm_block = if ((((has_ellipsis) < ((0 as i32))) as i32)) != 0 { 298 } else { 296 }; continue;
}
_ => std::process::abort(),
} }
}
