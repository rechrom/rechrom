// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:26697. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn optional_chain_test(mut s: *mut JSParseState, mut poptional_chaining_label: *mut i32, mut drop_count: i32) -> () {
let mut vm_local_storage = Vec::<u64>::new();
let mut label_next: i32 = 0;
let mut i: i32 = 0;
let mut vm_block: usize = 12;
loop { match vm_block {
// C line ?
0 => {
return ();
}
// C line 26711
1 => {
let _ = emit_label(s, label_next);
vm_block = 0; continue;
}
// C line 26710
2 => {
let _ = emit_goto(s, (OP_goto as i32), *(poptional_chaining_label));
vm_block = 1; continue;
}
// C line 26709
3 => {
let _ = emit_op(s, (((OP_undefined as i32)) as u8));
vm_block = 2; continue;
}
// C line 26707
4 => {
vm_block = if ((((i) < (drop_count)) as i32)) != 0 { 6 } else { 3 }; continue;
}
// C line 26707
5 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 4; continue;
}
// C line 26708
6 => {
let _ = emit_op(s, (((OP_drop as i32)) as u8));
vm_block = 5; continue;
}
// C line 26707
7 => {
let _ = { let assigned = (0 as i32); i = assigned; assigned };
vm_block = 4; continue;
}
// C line 26706
8 => {
let _ = { let assigned = emit_goto(s, (OP_if_false as i32), ((1 as i32)).wrapping_neg()); label_next = assigned; assigned };
vm_block = 7; continue;
}
// C line 26705
9 => {
let _ = emit_op(s, (((OP_is_undefined_or_null as i32)) as u8));
vm_block = 8; continue;
}
// C line 26704
10 => {
let _ = emit_op(s, (((OP_dup as i32)) as u8));
vm_block = 9; continue;
}
// C line 26702
11 => {
let _ = { let assigned = new_label(s); *(poptional_chaining_label) = assigned; assigned };
vm_block = 10; continue;
}
// C line 26701
12 => {
vm_block = if ((((*(poptional_chaining_label)) < ((0 as i32))) as i32)) != 0 { 11 } else { 10 }; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:26715. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_parse_postfix_expr(mut s: *mut JSParseState, mut parse_flags: i32) -> i32 {
let host_expression_start = (*(s)).token.ptr;
let mut vm_local_storage = Vec::<u64>::new();
let mut call_type: FuncCallType = core::mem::zeroed();
let mut optional_chaining_label: i32 = 0;
let mut accept_lparen: i32 = 0;
let mut op_token_ptr: *const u8 = core::ptr::null();
let mut val: JSValue = core::mem::zeroed();
let mut v: i64 = 0;
let mut str: JSValue = core::mem::zeroed();
let mut ret: i32 = 0;
let mut line_num: i32 = 0;
let mut col_num: i32 = 0;
let mut name: JSAtom = 0;
let mut source_ptr: *const u8 = core::ptr::null();
let mut fd: *mut JSFunctionDef = core::ptr::null_mut();
let mut has_optional_chain: i32 = 0;
let mut opcode: i32 = 0;
let mut arg_count: i32 = 0;
let mut drop_count: i32 = 0;
let mut opt_chain_label: i32 = 0;
let mut next_label: i32 = 0;
let mut opt_chain_label_1: i32 = 0;
let mut next_label_1: i32 = 0;
let mut name_1: JSAtom = 0;
let mut scope: i32 = 0;
let mut val_1: JSValue = core::mem::zeroed();
let mut ret_1: i32 = 0;
let mut prev_op: i32 = 0;
let mut fd_1: *mut JSFunctionDef = core::ptr::null_mut();
let mut opcode_1: i32 = 0;
let mut vm_block: usize = 415;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 27380
1 => {
return (0 as i32);
}
// C line 27375
2 => {
let _ = { let assigned = ((opcode_1) as u8); *(((*(fd_1)).byte_code).buf).offset(((*(fd_1)).last_opcode_pos) as isize) = assigned; assigned };
vm_block = 1; continue;
}
// C line 27372
3 => {
let _ = { let assigned = (OP_get_field_opt_chain as i32); opcode_1 = assigned; assigned };
vm_block = 2; continue;
}
// C line 27374
4 => {
let _ = { let assigned = (OP_get_array_el_opt_chain as i32); opcode_1 = assigned; assigned };
vm_block = 2; continue;
}
// C line 27371
5 => {
vm_block = if ((((opcode_1) == ((OP_get_field as i32))) as i32)) != 0 { 3 } else { 4 }; continue;
}
// C line 27377
6 => {
let _ = { let assigned = ((1 as i32)).wrapping_neg(); (*(fd_1)).last_opcode_pos = assigned; assigned };
vm_block = 1; continue;
}
// C line 27370
7 => {
vm_block = if ((((((((opcode_1) == ((OP_get_field as i32))) as i32)) != 0) || (((((opcode_1) == ((OP_get_array_el as i32))) as i32)) != 0)) as i32)) != 0 { 5 } else { 6 }; continue;
}
// C line 27369
8 => {
let _ = { let assigned = get_prev_opcode(fd_1); opcode_1 = assigned; assigned };
vm_block = 7; continue;
}
// C line 27366
9 => {
let _ = emit_label_raw(s, optional_chaining_label);
vm_block = 8; continue;
}
// C line 27364
10 => {
fd_1 = (*(s)).cur_func;
vm_block = 9; continue;
}
// C line 27363
11 => {
vm_block = if ((((optional_chaining_label) >= ((0 as i32))) as i32)) != 0 { 10 } else { 1 }; continue;
}
// C line 26984
12 => {
vm_block = 234; continue;
}
// C line 26997
13 => {
vm_block = 185; continue;
}
// C line 26999
14 => {
vm_block = 225; continue;
}
// C line 27001
15 => {
vm_block = 209; continue;
}
// C line 26998
16 => {
vm_block = if ((((((*(s)).token).val) == ((91 as i32))) as i32)) != 0 { 14 } else { 15 }; continue;
}
// C line 26996
17 => {
vm_block = if ((((((((((*(s)).token).val) == ((40 as i32))) as i32)) != 0) && ((accept_lparen) != 0)) as i32)) != 0 { 13 } else { 16 }; continue;
}
// C line 26995
18 => {
let _ = { let assigned = (1 as i32); has_optional_chain = assigned; assigned };
vm_block = 17; continue;
}
// C line 26994
19 => {
return ((1 as i32)).wrapping_neg();
}
// C line 26993
20 => {
vm_block = if (next_token(s)) != 0 { 19 } else { 18 }; continue;
}
// C line 26991
21 => {
let _ = { let assigned = ((*(s)).token).ptr; op_token_ptr = assigned; assigned };
vm_block = 20; continue;
}
// C line 26990
22 => {
return js_parse_error(s, c"new keyword cannot be used with an optional chain".as_ptr());
}
// C line 26989
23 => {
vm_block = if ((((((parse_flags) & (((1 as i32)).wrapping_shl(((1 as i32)) as u32)))) == ((0 as i32))) as i32)) != 0 { 22 } else { 21 }; continue;
}
// C line 27010
24 => {
vm_block = 180; continue;
}
// C line 27009
25 => {
let _ = { let assigned = ((*(s)).token).ptr; op_token_ptr = assigned; assigned };
vm_block = 24; continue;
}
// C line 27008
26 => {
let _ = { let assigned = (((FUNC_CALL_TEMPLATE as i32)) as FuncCallType); call_type = assigned; assigned };
vm_block = 25; continue;
}
// C line 27006
27 => {
return js_parse_error(s, c"template literal cannot appear in an optional chain".as_ptr());
}
// C line 27005
28 => {
vm_block = if ((((optional_chaining_label) >= ((0 as i32))) as i32)) != 0 { 27 } else { 26 }; continue;
}
// C line 27298
29 => {
let _ = { let assigned = (((FUNC_CALL_NORMAL as i32)) as FuncCallType); call_type = assigned; assigned };
vm_block = 12; continue;
}
// C line 27255
30 => {
vm_block = 29; continue;
}
// C line 27242
31 => {
let _ = emit_class_field_init(s);
vm_block = 30; continue;
}
// C line 27240
32 => {
let _ = emit_u16(s, (((0 as i32)) as u16));
vm_block = 31; continue;
}
// C line 27239
33 => {
let _ = emit_atom(s, (((crate::quickjs_atom::JS_ATOM_this as i32)) as JSAtom));
vm_block = 32; continue;
}
// C line 27238
34 => {
let _ = emit_op(s, (((OP_scope_put_var_init as i32)) as u8));
vm_block = 33; continue;
}
// C line 27237
35 => {
let _ = emit_op(s, (((OP_dup as i32)) as u8));
vm_block = 34; continue;
}
// C line 27235
36 => {
let _ = emit_u16(s, (((1 as i32)) as u16));
vm_block = 35; continue;
}
// C line 27234
37 => {
let _ = emit_op(s, (((OP_apply as i32)) as u8));
vm_block = 36; continue;
}
// C line 27247
38 => {
let _ = emit_u16(s, (((1 as i32)) as u16));
vm_block = 30; continue;
}
// C line 27246
39 => {
let _ = emit_op(s, (((OP_apply as i32)) as u8));
vm_block = 38; continue;
}
// C line 27245
40 => {
let _ = emit_op(s, (((OP_perm3 as i32)) as u8));
vm_block = 39; continue;
}
// C line 27253
41 => {
let _ = emit_u16(s, (((0 as i32)) as u16));
vm_block = 30; continue;
}
// C line 27252
42 => {
let _ = emit_op(s, (((OP_apply as i32)) as u8));
vm_block = 41; continue;
}
// C line 27251
43 => {
let _ = emit_op(s, (((OP_swap as i32)) as u8));
vm_block = 42; continue;
}
// C line 27250
44 => {
let _ = emit_op(s, (((OP_undefined as i32)) as u8));
vm_block = 43; continue;
}
// C line 27243
45 => {
vm_block = if ((((((call_type) as u32)) == ((((FUNC_CALL_NEW as i32)) as u32))) as i32)) != 0 { 40 } else { 44 }; continue;
}
// C line 27233
46 => {
vm_block = if ((((((call_type) as u32)) == ((((FUNC_CALL_SUPER_CTOR as i32)) as u32))) as i32)) != 0 { 37 } else { 45 }; continue;
}
// C line 27231
47 => {
vm_block = 29; continue;
}
// C line 27230
48 => {
let _ = { let assigned = (1 as i32); (*(fd)).has_eval_call = assigned; assigned };
vm_block = 47; continue;
}
// C line 27229
49 => {
let _ = emit_u16(s, (((*(fd)).scope_level) as u16));
vm_block = 48; continue;
}
// C line 27228
50 => {
let _ = emit_op(s, (((OP_apply_eval as i32)) as u8));
vm_block = 49; continue;
}
// C line 27226
51 => {
vm_block = 29; continue;
}
// C line 27225
52 => {
let _ = emit_u16(s, (((((((call_type) as u32)) == ((((FUNC_CALL_NEW as i32)) as u32))) as i32)) as u16));
vm_block = 51; continue;
}
// C line 27224
53 => {
let _ = emit_op(s, (((OP_apply as i32)) as u8));
vm_block = 52; continue;
}
// C line 27223
54 => {
let _ = emit_op(s, (((OP_perm3 as i32)) as u8));
vm_block = 53; continue;
}
// C line 27217
55 => {
vm_block = match opcode { x if x == (OP_eval as i32) => 50, x if x == (OP_scope_get_ref as i32) => 54, x if x == (OP_get_array_el as i32) => 54, x if x == (OP_scope_get_private_field as i32) => 54, x if x == (OP_get_field as i32) => 54, _ => 46, }; continue;
}
// C line 27215
56 => {
js_host_parser_callsite(s, op_token_ptr, host_expression_start);
let _ = emit_source_pos(s, op_token_ptr);
vm_block = 55; continue;
}
// C line 27213
57 => {
let _ = emit_op(s, (((OP_drop as i32)) as u8));
vm_block = 56; continue;
}
// C line 27211
58 => {
return ((1 as i32)).wrapping_neg();
}
// C line 27210
59 => {
vm_block = if (next_token(s)) != 0 { 58 } else { 57 }; continue;
}
// C line 27163
60 => {
vm_block = if ((((((*(s)).token).val) != ((41 as i32))) as i32)) != 0 { 74 } else { 59 }; continue;
}
// C line 27208
61 => {
return ((1 as i32)).wrapping_neg();
}
// C line 27207
62 => {
vm_block = if (js_parse_expect(s, (44 as i32))) != 0 { 61 } else { 60 }; continue;
}
// C line 27205
63 => {
vm_block = 59; continue;
}
// C line 27204
64 => {
vm_block = if ((((((*(s)).token).val) == ((41 as i32))) as i32)) != 0 { 63 } else { 62 }; continue;
}
// C line 27171
65 => {
let _ = emit_op(s, (((OP_append as i32)) as u8));
vm_block = 64; continue;
}
// C line 27168
66 => {
return ((1 as i32)).wrapping_neg();
}
// C line 27167
67 => {
vm_block = if (js_parse_assign_expr(s)) != 0 { 66 } else { 65 }; continue;
}
// C line 27166
68 => {
return ((1 as i32)).wrapping_neg();
}
// C line 27165
69 => {
vm_block = if (next_token(s)) != 0 { 68 } else { 67 }; continue;
}
// C line 27202
70 => {
let _ = emit_op(s, (((OP_inc as i32)) as u8));
vm_block = 64; continue;
}
// C line 27201
71 => {
let _ = emit_op(s, (((OP_define_array_el as i32)) as u8));
vm_block = 70; continue;
}
// C line 27199
72 => {
return ((1 as i32)).wrapping_neg();
}
// C line 27198
73 => {
vm_block = if (js_parse_assign_expr(s)) != 0 { 72 } else { 71 }; continue;
}
// C line 27164
74 => {
vm_block = if ((((((*(s)).token).val) == ((TOK_ELLIPSIS as i32))) as i32)) != 0 { 69 } else { 73 }; continue;
}
// C line 27160
75 => {
let _ = emit_u32(s, ((arg_count) as u32));
vm_block = 60; continue;
}
// C line 27159
76 => {
let _ = emit_op(s, (((OP_push_i32 as i32)) as u8));
vm_block = 75; continue;
}
// C line 27158
77 => {
let _ = emit_u16(s, ((arg_count) as u16));
vm_block = 76; continue;
}
// C line 27157
78 => {
let _ = emit_op(s, (((OP_array_from as i32)) as u8));
vm_block = 77; continue;
}
// C line 27295
79 => {
vm_block = 29; continue;
}
// C line 27287
80 => {
let _ = emit_class_field_init(s);
vm_block = 79; continue;
}
// C line 27285
81 => {
let _ = emit_u16(s, (((0 as i32)) as u16));
vm_block = 80; continue;
}
// C line 27284
82 => {
let _ = emit_atom(s, (((crate::quickjs_atom::JS_ATOM_this as i32)) as JSAtom));
vm_block = 81; continue;
}
// C line 27283
83 => {
let _ = emit_op(s, (((OP_scope_put_var_init as i32)) as u8));
vm_block = 82; continue;
}
// C line 27282
84 => {
let _ = emit_op(s, (((OP_dup as i32)) as u8));
vm_block = 83; continue;
}
// C line 27279
85 => {
let _ = emit_u16(s, ((arg_count) as u16));
vm_block = 84; continue;
}
// C line 27278
86 => {
let _ = emit_op(s, (((OP_call_constructor as i32)) as u8));
vm_block = 85; continue;
}
// C line 27290
87 => {
let _ = emit_u16(s, ((arg_count) as u16));
vm_block = 79; continue;
}
// C line 27289
88 => {
let _ = emit_op(s, (((OP_call_constructor as i32)) as u8));
vm_block = 87; continue;
}
// C line 27293
89 => {
let _ = emit_u16(s, ((arg_count) as u16));
vm_block = 79; continue;
}
// C line 27292
90 => {
let _ = emit_op(s, (((OP_call as i32)) as u8));
vm_block = 89; continue;
}
// C line 27288
91 => {
vm_block = if ((((((call_type) as u32)) == ((((FUNC_CALL_NEW as i32)) as u32))) as i32)) != 0 { 88 } else { 90 }; continue;
}
// C line 27277
92 => {
vm_block = if ((((((call_type) as u32)) == ((((FUNC_CALL_SUPER_CTOR as i32)) as u32))) as i32)) != 0 { 86 } else { 91 }; continue;
}
// C line 27275
93 => {
vm_block = 29; continue;
}
// C line 27274
94 => {
let _ = { let assigned = (1 as i32); (*(fd)).has_eval_call = assigned; assigned };
vm_block = 93; continue;
}
// C line 27273
95 => {
let _ = emit_u16(s, (((*(fd)).scope_level) as u16));
vm_block = 94; continue;
}
// C line 27272
96 => {
let _ = emit_u16(s, ((arg_count) as u16));
vm_block = 95; continue;
}
// C line 27271
97 => {
let _ = emit_op(s, (((OP_eval as i32)) as u8));
vm_block = 96; continue;
}
// C line 27269
98 => {
vm_block = 29; continue;
}
// C line 27268
99 => {
let _ = emit_u16(s, ((arg_count) as u16));
vm_block = 98; continue;
}
// C line 27267
100 => {
let _ = emit_op(s, (((OP_call_method as i32)) as u8));
vm_block = 99; continue;
}
// C line 27262
101 => {
vm_block = match opcode { x if x == (OP_eval as i32) => 97, x if x == (OP_scope_get_ref as i32) => 100, x if x == (OP_get_array_el as i32) => 100, x if x == (OP_scope_get_private_field as i32) => 100, x if x == (OP_get_field as i32) => 100, _ => 92, }; continue;
}
// C line 27261 labels: emit_func_call
102 => {
js_host_parser_callsite(s, op_token_ptr, host_expression_start);
let _ = emit_source_pos(s, op_token_ptr);
vm_block = 101; continue;
}
// C line 27259
103 => {
return ((1 as i32)).wrapping_neg();
}
// C line 27258
104 => {
vm_block = if (next_token(s)) != 0 { 103 } else { 102 }; continue;
}
// C line 27156
105 => {
vm_block = if ((((((*(s)).token).val) == ((TOK_ELLIPSIS as i32))) as i32)) != 0 { 78 } else { 104 }; continue;
}
// C line 27141
106 => {
vm_block = if ((((((*(s)).token).val) != ((41 as i32))) as i32)) != 0 { 117 } else { 105 }; continue;
}
// C line 27154
107 => {
return ((1 as i32)).wrapping_neg();
}
// C line 27153
108 => {
vm_block = if (js_parse_expect(s, (44 as i32))) != 0 { 107 } else { 106 }; continue;
}
// C line 27151
109 => {
vm_block = 105; continue;
}
// C line 27150
110 => {
vm_block = if ((((((*(s)).token).val) == ((41 as i32))) as i32)) != 0 { 109 } else { 108 }; continue;
}
// C line 27149
111 => {
let _ = { let old = arg_count; arg_count = (arg_count).wrapping_add(1); old };
vm_block = 110; continue;
}
// C line 27148
112 => {
return ((1 as i32)).wrapping_neg();
}
// C line 27147
113 => {
vm_block = if (js_parse_assign_expr(s)) != 0 { 112 } else { 111 }; continue;
}
// C line 27146
114 => {
vm_block = 105; continue;
}
// C line 27145
115 => {
vm_block = if ((((((*(s)).token).val) == ((TOK_ELLIPSIS as i32))) as i32)) != 0 { 114 } else { 113 }; continue;
}
// C line 27143
116 => {
return js_parse_error(s, c"Too many call arguments".as_ptr());
}
// C line 27142
117 => {
vm_block = if ((((arg_count) >= ((65535 as i32))) as i32)) != 0 { 116 } else { 115 }; continue;
}
// C line 27140
118 => {
let _ = { let assigned = (0 as i32); arg_count = assigned; assigned };
vm_block = 106; continue;
}
// C line 27124
119 => {
vm_block = 102; continue;
}
// C line 27123
120 => {
return ((1 as i32)).wrapping_neg();
}
// C line 27122
121 => {
vm_block = if (js_parse_template(s, (1 as i32), core::ptr::addr_of_mut!(arg_count))) != 0 { 120 } else { 119 }; continue;
}
// C line 27134
122 => {
let _ = emit_u16(s, (((0 as i32)) as u16));
vm_block = 118; continue;
}
// C line 27133
123 => {
let _ = emit_atom(s, (((crate::quickjs_atom::JS_ATOM_new_target as i32)) as JSAtom));
vm_block = 122; continue;
}
// C line 27132
124 => {
let _ = emit_op(s, (((OP_scope_get_var as i32)) as u8));
vm_block = 123; continue;
}
// C line 27130
125 => {
let _ = emit_op(s, (((OP_get_super as i32)) as u8));
vm_block = 124; continue;
}
// C line 27128
126 => {
let _ = emit_u16(s, (((0 as i32)) as u16));
vm_block = 125; continue;
}
// C line 27127
127 => {
let _ = emit_atom(s, (((crate::quickjs_atom::JS_ATOM_this_active_func as i32)) as JSAtom));
vm_block = 126; continue;
}
// C line 27126
128 => {
let _ = emit_op(s, (((OP_scope_get_var as i32)) as u8));
vm_block = 127; continue;
}
// C line 27136
129 => {
let _ = emit_op(s, (((OP_dup as i32)) as u8));
vm_block = 118; continue;
}
// C line 27135
130 => {
vm_block = if ((((((call_type) as u32)) == ((((FUNC_CALL_NEW as i32)) as u32))) as i32)) != 0 { 129 } else { 118 }; continue;
}
// C line 27125
131 => {
vm_block = if ((((((call_type) as u32)) == ((((FUNC_CALL_SUPER_CTOR as i32)) as u32))) as i32)) != 0 { 128 } else { 130 }; continue;
}
// C line 27121
132 => {
vm_block = if ((((((call_type) as u32)) == ((((FUNC_CALL_TEMPLATE as i32)) as u32))) as i32)) != 0 { 121 } else { 131 }; continue;
}
// C line 27114
133 => {
let _ = optional_chain_test(s, core::ptr::addr_of_mut!(optional_chaining_label), drop_count);
vm_block = 132; continue;
}
// C line 27113
134 => {
vm_block = if (has_optional_chain) != 0 { 133 } else { 132 }; continue;
}
// C line 27111
135 => {
vm_block = 134; continue;
}
// C line 27110
136 => {
let _ = { let assigned = (1 as i32); drop_count = assigned; assigned };
vm_block = 135; continue;
}
// C line 27109
137 => {
let _ = { let assigned = (OP_invalid as i32); opcode = assigned; assigned };
vm_block = 136; continue;
}
// C line 27107
138 => {
vm_block = 134; continue;
}
// C line 27106
139 => {
let _ = { let assigned = (2 as i32); drop_count = assigned; assigned };
vm_block = 138; continue;
}
// C line 27105
140 => {
let _ = { let assigned = (OP_get_array_el as i32); opcode = assigned; assigned };
vm_block = 139; continue;
}
// C line 27103
141 => {
let _ = { let assigned = (((OP_get_array_el as i32)) as u8); *(((*(fd)).byte_code).buf).offset(((*(fd)).last_opcode_pos) as isize) = assigned; assigned };
vm_block = 140; continue;
}
// C line 27101
142 => {
vm_block = 134; continue;
}
// C line 27099
143 => {
let _ = { let assigned = (1 as i32); drop_count = assigned; assigned };
vm_block = 142; continue;
}
// C line 27084
144 => {
let _ = { let assigned = (OP_eval as i32); opcode = assigned; assigned };
vm_block = 143; continue;
}
// C line 27096
145 => {
let _ = { let assigned = ((opcode) as u8); *(((*(fd)).byte_code).buf).offset(((*(fd)).last_opcode_pos) as isize) = assigned; assigned };
vm_block = 143; continue;
}
// C line 27095
146 => {
let _ = { let assigned = (OP_scope_get_ref as i32); opcode = assigned; assigned };
vm_block = 145; continue;
}
// C line 27094
147 => {
vm_block = if (has_with_scope(fd, scope)) != 0 { 146 } else { 143 }; continue;
}
// C line 27082
148 => {
vm_block = if ((((((((((((name_1) == ((((crate::quickjs_atom::JS_ATOM_eval as i32)) as JSAtom))) as i32)) != 0) && (((((((call_type) as u32)) == ((((FUNC_CALL_NORMAL as i32)) as u32))) as i32)) != 0)) as i32)) != 0) && (((!((has_optional_chain) != 0) as i32)) != 0)) as i32)) != 0 { 144 } else { 147 }; continue;
}
// C line 27081
149 => {
let _ = { let assigned = ((get_u16(((((*(fd)).byte_code).buf).offset((((*(fd)).last_opcode_pos) as isize))).offset((((5 as i32)) as isize)))) as i32); scope = assigned; assigned };
vm_block = 148; continue;
}
// C line 27080
150 => {
let _ = { let assigned = get_u32(((((*(fd)).byte_code).buf).offset((((*(fd)).last_opcode_pos) as isize))).offset((((1 as i32)) as isize))); name_1 = assigned; assigned };
vm_block = 149; continue;
}
// C line 27075
151 => {
vm_block = 134; continue;
}
// C line 27073
152 => {
let _ = { let assigned = (OP_get_array_el as i32); opcode = assigned; assigned };
vm_block = 151; continue;
}
// C line 27072
153 => {
let _ = { let assigned = (2 as i32); drop_count = assigned; assigned };
vm_block = 152; continue;
}
// C line 27071
154 => {
let _ = emit_label(s, next_label_1);
vm_block = 153; continue;
}
// C line 27070
155 => {
let _ = emit_op(s, (((OP_undefined as i32)) as u8));
vm_block = 154; continue;
}
// C line 27066
156 => {
let _ = emit_label(s, opt_chain_label_1);
vm_block = 155; continue;
}
// C line 27065
157 => {
let _ = { let assigned = emit_goto(s, (OP_goto as i32), ((1 as i32)).wrapping_neg()); next_label_1 = assigned; assigned };
vm_block = 156; continue;
}
// C line 27064
158 => {
let _ = { let assigned = ((((*(fd)).last_opcode_pos).wrapping_add((1 as i32))) as usize); ((*(fd)).byte_code).size = assigned; assigned };
vm_block = 157; continue;
}
// C line 27063
159 => {
let _ = { let assigned = (((OP_get_array_el2 as i32)) as u8); *(((*(fd)).byte_code).buf).offset(((*(fd)).last_opcode_pos) as isize) = assigned; assigned };
vm_block = 158; continue;
}
// C line 27060
160 => {
let _ = { let assigned = ((get_u32((((((*(fd)).byte_code).buf).offset((((*(fd)).last_opcode_pos) as isize))).offset((((1 as i32)) as isize))).offset((((1 as i32)) as isize)))) as i32); opt_chain_label_1 = assigned; assigned };
vm_block = 159; continue;
}
// C line 27056
161 => {
vm_block = 134; continue;
}
// C line 27055
162 => {
let _ = { let assigned = (2 as i32); drop_count = assigned; assigned };
vm_block = 161; continue;
}
// C line 27054
163 => {
let _ = { let assigned = (((OP_get_array_el2 as i32)) as u8); *(((*(fd)).byte_code).buf).offset(((*(fd)).last_opcode_pos) as isize) = assigned; assigned };
vm_block = 162; continue;
}
// C line 27051
164 => {
vm_block = 134; continue;
}
// C line 27050
165 => {
let _ = { let assigned = (2 as i32); drop_count = assigned; assigned };
vm_block = 164; continue;
}
// C line 27049
166 => {
let _ = { let assigned = (((OP_scope_get_private_field2 as i32)) as u8); *(((*(fd)).byte_code).buf).offset(((*(fd)).last_opcode_pos) as isize) = assigned; assigned };
vm_block = 165; continue;
}
// C line 27046
167 => {
vm_block = 134; continue;
}
// C line 27044
168 => {
let _ = { let assigned = (OP_get_field as i32); opcode = assigned; assigned };
vm_block = 167; continue;
}
// C line 27043
169 => {
let _ = { let assigned = (2 as i32); drop_count = assigned; assigned };
vm_block = 168; continue;
}
// C line 27042
170 => {
let _ = emit_label(s, next_label);
vm_block = 169; continue;
}
// C line 27041
171 => {
let _ = emit_op(s, (((OP_undefined as i32)) as u8));
vm_block = 170; continue;
}
// C line 27037
172 => {
let _ = emit_label(s, opt_chain_label);
vm_block = 171; continue;
}
// C line 27036
173 => {
let _ = { let assigned = emit_goto(s, (OP_goto as i32), ((1 as i32)).wrapping_neg()); next_label = assigned; assigned };
vm_block = 172; continue;
}
// C line 27035
174 => {
let _ = { let assigned = (((((*(fd)).last_opcode_pos).wrapping_add((1 as i32))).wrapping_add((4 as i32))) as usize); ((*(fd)).byte_code).size = assigned; assigned };
vm_block = 173; continue;
}
// C line 27034
175 => {
let _ = { let assigned = (((OP_get_field2 as i32)) as u8); *(((*(fd)).byte_code).buf).offset(((*(fd)).last_opcode_pos) as isize) = assigned; assigned };
vm_block = 174; continue;
}
// C line 27031
176 => {
let _ = { let assigned = ((get_u32(((((((*(fd)).byte_code).buf).offset((((*(fd)).last_opcode_pos) as isize))).offset((((1 as i32)) as isize))).offset((((4 as i32)) as isize))).offset((((1 as i32)) as isize)))) as i32); opt_chain_label = assigned; assigned };
vm_block = 175; continue;
}
// C line 27027
177 => {
vm_block = 134; continue;
}
// C line 27026
178 => {
let _ = { let assigned = (2 as i32); drop_count = assigned; assigned };
vm_block = 177; continue;
}
// C line 27025
179 => {
let _ = { let assigned = (((OP_get_field2 as i32)) as u8); *(((*(fd)).byte_code).buf).offset(((*(fd)).last_opcode_pos) as isize) = assigned; assigned };
vm_block = 178; continue;
}
// C line 27022 labels: parse_func_call2
180 => {
vm_block = match { let assigned = get_prev_opcode(fd); opcode = assigned; assigned } { x if x == (OP_get_super_value as i32) => 141, x if x == (OP_scope_get_var as i32) => 150, x if x == (OP_get_array_el_opt_chain as i32) => 160, x if x == (OP_get_array_el as i32) => 163, x if x == (OP_scope_get_private_field as i32) => 166, x if x == (OP_get_field_opt_chain as i32) => 176, x if x == (OP_get_field as i32) => 179, _ => 137, }; continue;
}
// C line 27118
181 => {
let _ = { let assigned = (OP_invalid as i32); opcode = assigned; assigned };
vm_block = 132; continue;
}
// C line 27020
182 => {
vm_block = if ((((((call_type) as u32)) == ((((FUNC_CALL_NORMAL as i32)) as u32))) as i32)) != 0 { 180 } else { 181 }; continue;
}
// C line 27018
183 => {
return ((1 as i32)).wrapping_neg();
}
// C line 27017
184 => {
vm_block = if (next_token(s)) != 0 { 183 } else { 182 }; continue;
}
// C line 27016 labels: parse_func_call
185 => {
let _ = { let assigned = ((*(s)).token).ptr; op_token_ptr = assigned; assigned };
vm_block = 184; continue;
}
// C line 27338
186 => {
return ((1 as i32)).wrapping_neg();
}
// C line 27337
187 => {
vm_block = if (next_token(s)) != 0 { 186 } else { 12 }; continue;
}
// C line 27315
188 => {
let _ = emit_u16(s, (((*((*(s)).cur_func)).scope_level) as u16));
vm_block = 187; continue;
}
// C line 27314
189 => {
let _ = emit_atom(s, ((((*(s)).token).u).ident).atom);
vm_block = 188; continue;
}
// C line 27313
190 => {
let _ = emit_op(s, (((OP_scope_get_private_field as i32)) as u8));
vm_block = 189; continue;
}
// C line 27311
191 => {
let _ = optional_chain_test(s, core::ptr::addr_of_mut!(optional_chaining_label), (1 as i32));
vm_block = 190; continue;
}
// C line 27310
192 => {
vm_block = if (has_optional_chain) != 0 { 191 } else { 190 }; continue;
}
// C line 27308
193 => {
return js_parse_error(s, c"private class field forbidden after super".as_ptr());
}
// C line 27307
194 => {
vm_block = if ((((get_prev_opcode(fd)) == ((OP_get_super as i32))) as i32)) != 0 { 193 } else { 192 }; continue;
}
// C line 27328
195 => {
let _ = emit_op(s, (((OP_get_super_value as i32)) as u8));
vm_block = 187; continue;
}
// C line 27327
196 => {
return ((1 as i32)).wrapping_neg();
}
// C line 27326
197 => {
vm_block = if (ret_1) != 0 { 196 } else { 195 }; continue;
}
// C line 27325
198 => {
let _ = JS_FreeValue((*(s)).ctx, val_1);
vm_block = 197; continue;
}
// C line 27324
199 => {
let _ = { let assigned = emit_push_const(s, val_1, (1 as i32)); ret_1 = assigned; assigned };
vm_block = 198; continue;
}
// C line 27323
200 => {
let _ = { let assigned = JS_AtomToValue((*(s)).ctx, ((((*(s)).token).u).ident).atom); val_1 = assigned; assigned };
vm_block = 199; continue;
}
// C line 27334
201 => {
let _ = emit_atom(s, ((((*(s)).token).u).ident).atom);
vm_block = 187; continue;
}
// C line 27333
202 => {
let _ = emit_op(s, (((OP_get_field as i32)) as u8));
vm_block = 201; continue;
}
// C line 27331
203 => {
let _ = optional_chain_test(s, core::ptr::addr_of_mut!(optional_chaining_label), (1 as i32));
vm_block = 202; continue;
}
// C line 27330
204 => {
vm_block = if (has_optional_chain) != 0 { 203 } else { 202 }; continue;
}
// C line 27320
205 => {
vm_block = if ((((get_prev_opcode(fd)) == ((OP_get_super as i32))) as i32)) != 0 { 200 } else { 204 }; continue;
}
// C line 27318
206 => {
return js_parse_error(s, c"expecting field name".as_ptr());
}
// C line 27317
207 => {
vm_block = if ((!((token_is_ident(((*(s)).token).val)) != 0) as i32)) != 0 { 206 } else { 205 }; continue;
}
// C line 27305
208 => {
vm_block = if ((((((*(s)).token).val) == ((TOK_PRIVATE_NAME as i32))) as i32)) != 0 { 194 } else { 207 }; continue;
}
// C line 27304 labels: parse_property
209 => {
let _ = emit_source_pos(s, op_token_ptr);
vm_block = 208; continue;
}
// C line 27302
210 => {
return ((1 as i32)).wrapping_neg();
}
// C line 27301
211 => {
vm_block = if (next_token(s)) != 0 { 210 } else { 209 }; continue;
}
// C line 27300
212 => {
let _ = { let assigned = ((*(s)).token).ptr; op_token_ptr = assigned; assigned };
vm_block = 211; continue;
}
// C line 27355
213 => {
let _ = emit_op(s, (((OP_get_super_value as i32)) as u8));
vm_block = 12; continue;
}
// C line 27357
214 => {
let _ = emit_op(s, (((OP_get_array_el as i32)) as u8));
vm_block = 12; continue;
}
// C line 27354
215 => {
vm_block = if ((((prev_op) == ((OP_get_super as i32))) as i32)) != 0 { 213 } else { 214 }; continue;
}
// C line 27353
216 => {
let _ = emit_source_pos(s, op_token_ptr);
vm_block = 215; continue;
}
// C line 27352
217 => {
return ((1 as i32)).wrapping_neg();
}
// C line 27351
218 => {
vm_block = if (js_parse_expect(s, (93 as i32))) != 0 { 217 } else { 216 }; continue;
}
// C line 27350
219 => {
return ((1 as i32)).wrapping_neg();
}
// C line 27349
220 => {
vm_block = if (js_parse_expr(s)) != 0 { 219 } else { 218 }; continue;
}
// C line 27348
221 => {
return ((1 as i32)).wrapping_neg();
}
// C line 27347
222 => {
vm_block = if (next_token(s)) != 0 { 221 } else { 220 }; continue;
}
// C line 27345
223 => {
let _ = optional_chain_test(s, core::ptr::addr_of_mut!(optional_chaining_label), (1 as i32));
vm_block = 222; continue;
}
// C line 27344
224 => {
vm_block = if (has_optional_chain) != 0 { 223 } else { 222 }; continue;
}
// C line 27343 labels: parse_array_access
225 => {
let _ = { let assigned = get_prev_opcode(fd); prev_op = assigned; assigned };
vm_block = 224; continue;
}
// C line 27341
226 => {
let _ = { let assigned = ((*(s)).token).ptr; op_token_ptr = assigned; assigned };
vm_block = 225; continue;
}
// C line 27360
227 => {
vm_block = 11; continue;
}
// C line 27339
228 => {
vm_block = if ((((((*(s)).token).val) == ((91 as i32))) as i32)) != 0 { 226 } else { 227 }; continue;
}
// C line 27299
229 => {
vm_block = if ((((((*(s)).token).val) == ((46 as i32))) as i32)) != 0 { 212 } else { 228 }; continue;
}
// C line 27011
230 => {
vm_block = if ((((((((((*(s)).token).val) == ((40 as i32))) as i32)) != 0) && ((accept_lparen) != 0)) as i32)) != 0 { 185 } else { 229 }; continue;
}
// C line 27003
231 => {
vm_block = if ((((((((((*(s)).token).val) == ((TOK_TEMPLATE as i32))) as i32)) != 0) && (((((((call_type) as u32)) == ((((FUNC_CALL_NORMAL as i32)) as u32))) as i32)) != 0)) as i32)) != 0 { 28 } else { 230 }; continue;
}
// C line 26988
232 => {
vm_block = if ((((((*(s)).token).val) == ((TOK_QUESTION_MARK_DOT as i32))) as i32)) != 0 { 23 } else { 231 }; continue;
}
// C line 26986
233 => {
has_optional_chain = (0 as i32);
vm_block = 232; continue;
}
// C line 26985
234 => {
fd = (*(s)).cur_func;
vm_block = 233; continue;
}
// C line 26983
235 => {
let _ = { let assigned = ((1 as i32)).wrapping_neg(); optional_chaining_label = assigned; assigned };
vm_block = 12; continue;
}
// C line 26979
236 => {
return js_parse_error_cargs(s, c"unexpected token in expression: '%.*s'".as_ptr(), &[ParserFormatArg::Slice((((*(s)).token).ptr) as *const c_char, ((((((*(s)).buf_ptr).offset_from(((*(s)).token).ptr) as i64)) as i32)) as i32)]);
}
// C line 26977
237 => {
vm_block = 235; continue;
}
// C line 26948
238 => {
let _ = emit_u8(s, (((OP_SPECIAL_OBJECT_IMPORT_META as i32)) as u8));
vm_block = 237; continue;
}
// C line 26947
239 => {
let _ = emit_op(s, (((OP_special_object as i32)) as u8));
vm_block = 238; continue;
}
// C line 26946
240 => {
return ((1 as i32)).wrapping_neg();
}
// C line 26945
241 => {
vm_block = if (next_token(s)) != 0 { 240 } else { 239 }; continue;
}
// C line 26944
242 => {
return js_parse_error(s, c"import.meta only valid in module code".as_ptr());
}
// C line 26943
243 => {
vm_block = if ((!(((*(s)).is_module) != 0) as i32)) != 0 { 242 } else { 241 }; continue;
}
// C line 26942
244 => {
return js_parse_error(s, c"meta expected".as_ptr());
}
// C line 26941
245 => {
vm_block = if ((!((token_is_pseudo_keyword(s, (((crate::quickjs_atom::JS_ATOM_meta as i32)) as JSAtom))) != 0) as i32)) != 0 { 244 } else { 243 }; continue;
}
// C line 26940
246 => {
return ((1 as i32)).wrapping_neg();
}
// C line 26939
247 => {
vm_block = if (next_token(s)) != 0 { 246 } else { 245 }; continue;
}
// C line 26975
248 => {
let _ = emit_op(s, (((OP_import as i32)) as u8));
vm_block = 237; continue;
}
// C line 26974
249 => {
return ((1 as i32)).wrapping_neg();
}
// C line 26973
250 => {
vm_block = if (js_parse_expect(s, (41 as i32))) != 0 { 249 } else { 248 }; continue;
}
// C line 26965
251 => {
return ((1 as i32)).wrapping_neg();
}
// C line 26964
252 => {
vm_block = if (next_token(s)) != 0 { 251 } else { 250 }; continue;
}
// C line 26963
253 => {
vm_block = if ((((((*(s)).token).val) == ((44 as i32))) as i32)) != 0 { 252 } else { 250 }; continue;
}
// C line 26961
254 => {
return ((1 as i32)).wrapping_neg();
}
// C line 26960
255 => {
vm_block = if (js_parse_assign_expr(s)) != 0 { 254 } else { 253 }; continue;
}
// C line 26968
256 => {
let _ = emit_op(s, (((OP_undefined as i32)) as u8));
vm_block = 250; continue;
}
// C line 26959
257 => {
vm_block = if ((((((*(s)).token).val) != ((41 as i32))) as i32)) != 0 { 255 } else { 256 }; continue;
}
// C line 26958
258 => {
return ((1 as i32)).wrapping_neg();
}
// C line 26957
259 => {
vm_block = if (next_token(s)) != 0 { 258 } else { 257 }; continue;
}
// C line 26971
260 => {
let _ = emit_op(s, (((OP_undefined as i32)) as u8));
vm_block = 250; continue;
}
// C line 26956
261 => {
vm_block = if ((((((*(s)).token).val) == ((44 as i32))) as i32)) != 0 { 259 } else { 260 }; continue;
}
// C line 26955
262 => {
return ((1 as i32)).wrapping_neg();
}
// C line 26954
263 => {
vm_block = if (js_parse_assign_expr(s)) != 0 { 262 } else { 261 }; continue;
}
// C line 26953
264 => {
return js_parse_error(s, c"invalid use of 'import()'".as_ptr());
}
// C line 26952
265 => {
vm_block = if ((!((accept_lparen) != 0) as i32)) != 0 { 264 } else { 263 }; continue;
}
// C line 26951
266 => {
return ((1 as i32)).wrapping_neg();
}
// C line 26950
267 => {
vm_block = if (js_parse_expect(s, (40 as i32))) != 0 { 266 } else { 265 }; continue;
}
// C line 26938
268 => {
vm_block = if ((((((*(s)).token).val) == ((46 as i32))) as i32)) != 0 { 247 } else { 267 }; continue;
}
// C line 26937
269 => {
return ((1 as i32)).wrapping_neg();
}
// C line 26936
270 => {
vm_block = if (next_token(s)) != 0 { 269 } else { 268 }; continue;
}
// C line 26934
271 => {
vm_block = 235; continue;
}
// C line 26920
272 => {
let _ = { let assigned = (((FUNC_CALL_SUPER_CTOR as i32)) as FuncCallType); call_type = assigned; assigned };
vm_block = 271; continue;
}
// C line 26919
273 => {
return js_parse_error(s, c"super() is only valid in a derived class constructor".as_ptr());
}
// C line 26918
274 => {
vm_block = if ((!(((*((*(s)).cur_func)).super_call_allowed) != 0) as i32)) != 0 { 273 } else { 272 }; continue;
}
// C line 26930
275 => {
let _ = emit_op(s, (((OP_get_super as i32)) as u8));
vm_block = 271; continue;
}
// C line 26929
276 => {
let _ = emit_u16(s, (((0 as i32)) as u16));
vm_block = 275; continue;
}
// C line 26928
277 => {
let _ = emit_atom(s, (((crate::quickjs_atom::JS_ATOM_home_object as i32)) as JSAtom));
vm_block = 276; continue;
}
// C line 26927
278 => {
let _ = emit_op(s, (((OP_scope_get_var as i32)) as u8));
vm_block = 277; continue;
}
// C line 26926
279 => {
let _ = emit_u16(s, (((0 as i32)) as u16));
vm_block = 278; continue;
}
// C line 26925
280 => {
let _ = emit_atom(s, (((crate::quickjs_atom::JS_ATOM_this as i32)) as JSAtom));
vm_block = 279; continue;
}
// C line 26924
281 => {
let _ = emit_op(s, (((OP_scope_get_var as i32)) as u8));
vm_block = 280; continue;
}
// C line 26923
282 => {
return js_parse_error(s, c"'super' is only valid in a method".as_ptr());
}
// C line 26922
283 => {
vm_block = if ((!(((*((*(s)).cur_func)).super_allowed) != 0) as i32)) != 0 { 282 } else { 281 }; continue;
}
// C line 26932
284 => {
return js_parse_error(s, c"invalid use of 'super'".as_ptr());
}
// C line 26921
285 => {
vm_block = if ((((((((((*(s)).token).val) == ((46 as i32))) as i32)) != 0) || (((((((*(s)).token).val) == ((91 as i32))) as i32)) != 0)) as i32)) != 0 { 283 } else { 284 }; continue;
}
// C line 26917
286 => {
vm_block = if ((((((*(s)).token).val) == ((40 as i32))) as i32)) != 0 { 274 } else { 285 }; continue;
}
// C line 26916
287 => {
return ((1 as i32)).wrapping_neg();
}
// C line 26915
288 => {
vm_block = if (next_token(s)) != 0 { 287 } else { 286 }; continue;
}
// C line 26913
289 => {
vm_block = 235; continue;
}
// C line 26898
290 => {
let _ = emit_u16(s, (((0 as i32)) as u16));
vm_block = 289; continue;
}
// C line 26897
291 => {
let _ = emit_atom(s, (((crate::quickjs_atom::JS_ATOM_new_target as i32)) as JSAtom));
vm_block = 290; continue;
}
// C line 26896
292 => {
let _ = emit_op(s, (((OP_scope_get_var as i32)) as u8));
vm_block = 291; continue;
}
// C line 26895
293 => {
return ((1 as i32)).wrapping_neg();
}
// C line 26894
294 => {
vm_block = if (next_token(s)) != 0 { 293 } else { 292 }; continue;
}
// C line 26893
295 => {
return js_parse_error(s, c"new.target only allowed within functions".as_ptr());
}
// C line 26892
296 => {
vm_block = if ((!(((*((*(s)).cur_func)).new_target_allowed) != 0) as i32)) != 0 { 295 } else { 294 }; continue;
}
// C line 26891
297 => {
return js_parse_error(s, c"expecting target".as_ptr());
}
// C line 26890
298 => {
vm_block = if ((!((token_is_pseudo_keyword(s, (((crate::quickjs_atom::JS_ATOM_target as i32)) as JSAtom))) != 0) as i32)) != 0 { 297 } else { 296 }; continue;
}
// C line 26889
299 => {
return ((1 as i32)).wrapping_neg();
}
// C line 26888
300 => {
vm_block = if (next_token(s)) != 0 { 299 } else { 298 }; continue;
}
// C line 26908
301 => {
let _ = emit_u16(s, (((0 as i32)) as u16));
vm_block = 289; continue;
}
// C line 26907
302 => {
let _ = emit_op(s, (((OP_call_constructor as i32)) as u8));
vm_block = 301; continue;
}
// C line 26906
303 => {
let _ = emit_op(s, (((OP_dup as i32)) as u8));
vm_block = 302; continue;
}
// C line 26905
304 => {
js_host_parser_callsite(s, (*(s)).token.ptr, host_expression_start);
let _ = emit_source_pos(s, ((*(s)).token).ptr);
vm_block = 303; continue;
}
// C line 26910
305 => {
let _ = { let assigned = (((FUNC_CALL_NEW as i32)) as FuncCallType); call_type = assigned; assigned };
vm_block = 289; continue;
}
// C line 26903
306 => {
vm_block = if ((((((*(s)).token).val) != ((40 as i32))) as i32)) != 0 { 304 } else { 305 }; continue;
}
// C line 26902
307 => {
let _ = { let assigned = (1 as i32); accept_lparen = assigned; assigned };
vm_block = 306; continue;
}
// C line 26901
308 => {
return ((1 as i32)).wrapping_neg();
}
// C line 26900
309 => {
vm_block = if (js_parse_postfix_expr(s, (0 as i32))) != 0 { 308 } else { 307 }; continue;
}
// C line 26887
310 => {
vm_block = if ((((((*(s)).token).val) == ((46 as i32))) as i32)) != 0 { 300 } else { 309 }; continue;
}
// C line 26886
311 => {
return ((1 as i32)).wrapping_neg();
}
// C line 26885
312 => {
vm_block = if (next_token(s)) != 0 { 311 } else { 310 }; continue;
}
// C line 26883
313 => {
vm_block = 235; continue;
}
// C line 26878
314 => {
return ((1 as i32)).wrapping_neg();
}
// C line 26877
315 => {
vm_block = if (js_parse_object_literal(s)) != 0 { 314 } else { 313 }; continue;
}
// C line 26881
316 => {
return ((1 as i32)).wrapping_neg();
}
// C line 26880
317 => {
vm_block = if (js_parse_array_literal(s)) != 0 { 316 } else { 313 }; continue;
}
// C line 26876
318 => {
vm_block = if ((((((*(s)).token).val) == ((123 as i32))) as i32)) != 0 { 315 } else { 317 }; continue;
}
// C line 26873
319 => {
vm_block = 235; continue;
}
// C line 26850
320 => {
return ((1 as i32)).wrapping_neg();
}
// C line 26847
321 => {
vm_block = if (js_parse_function_decl(s, (((JS_PARSE_FUNC_EXPR as i32)) as JSParseFunctionEnum), (((JS_FUNC_ASYNC as i32)) as JSFunctionKindEnum), (((0 as i32)) as JSAtom), source_ptr)) != 0 { 320 } else { 319 }; continue;
}
// C line 26853
322 => {
vm_block = 330; continue;
}
// C line 26852
323 => {
let _ = { let assigned = JS_DupAtom((*(s)).ctx, (((crate::quickjs_atom::JS_ATOM_async as i32)) as JSAtom)); name = assigned; assigned };
vm_block = 322; continue;
}
// C line 26846
324 => {
vm_block = if ((((((*(s)).token).val) == ((TOK_FUNCTION as i32))) as i32)) != 0 { 321 } else { 323 }; continue;
}
// C line 26845
325 => {
return ((1 as i32)).wrapping_neg();
}
// C line 26844
326 => {
vm_block = if (next_token(s)) != 0 { 325 } else { 324 }; continue;
}
// C line 26870
327 => {
let _ = emit_u16(s, (((*((*(s)).cur_func)).scope_level) as u16));
vm_block = 319; continue;
}
// C line 26869
328 => {
let _ = emit_u32(s, name);
vm_block = 327; continue;
}
// C line 26868
329 => {
let _ = emit_op(s, (((OP_scope_get_var as i32)) as u8));
vm_block = 328; continue;
}
// C line 26867 labels: do_get_var
330 => {
let _ = emit_source_pos(s, source_ptr);
vm_block = 329; continue;
}
// C line 26864
331 => {
return ((1 as i32)).wrapping_neg();
}
// C line 26863
332 => {
let _ = JS_FreeAtom((*(s)).ctx, name);
vm_block = 331; continue;
}
// C line 26862
333 => {
vm_block = if (next_token(s)) != 0 { 332 } else { 330 }; continue;
}
// C line 26861
334 => {
let _ = { let assigned = JS_DupAtom((*(s)).ctx, ((((*(s)).token).u).ident).atom); name = assigned; assigned };
vm_block = 333; continue;
}
// C line 26859
335 => {
return ((1 as i32)).wrapping_neg();
}
// C line 26858
336 => {
let _ = js_parse_error(s, c"'arguments' identifier is not allowed in class field initializer".as_ptr());
vm_block = 335; continue;
}
// C line 26856
337 => {
vm_block = if ((((((((((((*(s)).token).u).ident).atom) == ((((crate::quickjs_atom::JS_ATOM_arguments as i32)) as JSAtom))) as i32)) != 0) && (((!(((*((*(s)).cur_func)).arguments_allowed) != 0) as i32)) != 0)) as i32)) != 0 { 336 } else { 334 }; continue;
}
// C line 26842
338 => {
vm_block = if (((((token_is_pseudo_keyword(s, (((crate::quickjs_atom::JS_ATOM_async as i32)) as JSAtom))) != 0) && (((((peek_token(s, (1 as i32))) != ((10 as i32))) as i32)) != 0)) as i32)) != 0 { 326 } else { 337 }; continue;
}
// C line 26841
339 => {
let _ = { let assigned = ((*(s)).token).ptr; source_ptr = assigned; assigned };
vm_block = 338; continue;
}
// C line 26839
340 => {
return js_parse_error_reserved_identifier(s);
}
// C line 26838
341 => {
vm_block = if (((((*(s)).token).u).ident).is_reserved) != 0 { 340 } else { 339 }; continue;
}
// C line 26833
342 => {
vm_block = 235; continue;
}
// C line 26832
343 => {
let _ = emit_op(s, (((OP_push_true as i32)) as u8));
vm_block = 342; continue;
}
// C line 26831
344 => {
return ((1 as i32)).wrapping_neg();
}
// C line 26830
345 => {
vm_block = if (next_token(s)) != 0 { 344 } else { 343 }; continue;
}
// C line 26828
346 => {
vm_block = 235; continue;
}
// C line 26827
347 => {
let _ = emit_op(s, (((OP_push_false as i32)) as u8));
vm_block = 346; continue;
}
// C line 26826
348 => {
return ((1 as i32)).wrapping_neg();
}
// C line 26825
349 => {
vm_block = if (next_token(s)) != 0 { 348 } else { 347 }; continue;
}
// C line 26823
350 => {
vm_block = 235; continue;
}
// C line 26822
351 => {
let _ = emit_u16(s, (((0 as i32)) as u16));
vm_block = 350; continue;
}
// C line 26821
352 => {
let _ = emit_atom(s, (((crate::quickjs_atom::JS_ATOM_this as i32)) as JSAtom));
vm_block = 351; continue;
}
// C line 26820
353 => {
let _ = emit_op(s, (((OP_scope_get_var as i32)) as u8));
vm_block = 352; continue;
}
// C line 26819
354 => {
return ((1 as i32)).wrapping_neg();
}
// C line 26818
355 => {
vm_block = if (next_token(s)) != 0 { 354 } else { 353 }; continue;
}
// C line 26816
356 => {
vm_block = 235; continue;
}
// C line 26815
357 => {
let _ = emit_op(s, (((OP_null as i32)) as u8));
vm_block = 356; continue;
}
// C line 26814
358 => {
return ((1 as i32)).wrapping_neg();
}
// C line 26813
359 => {
vm_block = if (next_token(s)) != 0 { 358 } else { 357 }; continue;
}
// C line 26811
360 => {
vm_block = 235; continue;
}
// C line 26810
361 => {
return ((1 as i32)).wrapping_neg();
}
// C line 26809
362 => {
vm_block = if (js_parse_class(s, (1 as i32), (((JS_PARSE_EXPORT_NONE as i32)) as JSParseExportEnum))) != 0 { 361 } else { 360 }; continue;
}
// C line 26807
363 => {
vm_block = 235; continue;
}
// C line 26806
364 => {
return ((1 as i32)).wrapping_neg();
}
// C line 26803
365 => {
vm_block = if (js_parse_function_decl(s, (((JS_PARSE_FUNC_EXPR as i32)) as JSParseFunctionEnum), (((JS_FUNC_NORMAL as i32)) as JSFunctionKindEnum), (((0 as i32)) as JSAtom), ((*(s)).token).ptr)) != 0 { 364 } else { 363 }; continue;
}
// C line 26801
366 => {
vm_block = 235; continue;
}
// C line 26800
367 => {
return ((1 as i32)).wrapping_neg();
}
// C line 26799
368 => {
vm_block = if (js_parse_expr_paren(s)) != 0 { 367 } else { 366 }; continue;
}
// C line 26797
369 => {
vm_block = 235; continue;
}
// C line 26795
370 => {
return ((1 as i32)).wrapping_neg();
}
// C line 26794
371 => {
vm_block = if (next_token(s)) != 0 { 370 } else { 369 }; continue;
}
// C line 26793
372 => {
let _ = emit_op(s, (((OP_regexp as i32)) as u8));
vm_block = 371; continue;
}
// C line 26789
373 => {
return ((1 as i32)).wrapping_neg();
}
// C line 26788
374 => {
vm_block = if (ret) != 0 { 373 } else { 372 }; continue;
}
// C line 26787
375 => {
let _ = JS_FreeValue((*(s)).ctx, str);
vm_block = 374; continue;
}
// C line 26786
376 => {
let _ = { let assigned = emit_push_const(s, str, (0 as i32)); ret = assigned; assigned };
vm_block = 375; continue;
}
// C line 26784
377 => {
return ((1 as i32)).wrapping_neg();
}
// C line 26782
378 => {
let _ = build_backtrace((*(s)).ctx, (*((*((*(s)).ctx)).rt)).current_exception, (*(s)).filename, (line_num).wrapping_add((1 as i32)), (col_num).wrapping_add((1 as i32)), (0 as i32));
vm_block = 377; continue;
}
// C line 26781
379 => {
let _ = { let assigned = get_line_col(core::ptr::addr_of_mut!(col_num), (*(s)).buf_start, ((((((*(s)).token).ptr).offset_from((*(s)).buf_start) as i64)) as usize)); line_num = assigned; assigned };
vm_block = 378; continue;
}
// C line 26778
380 => {
vm_block = if (JS_IsException(str)) != 0 { 379 } else { 376 }; continue;
}
// C line 26776
381 => {
let _ = { let assigned = ((*((*(s)).ctx)).compile_regexp).expect("registered parser callback")((*(s)).ctx, ((((*(s)).token).u).regexp).body, ((((*(s)).token).u).regexp).flags); str = assigned; assigned };
vm_block = 380; continue;
}
// C line 26775
382 => {
let _ = { let assigned = emit_push_const(s, ((((*(s)).token).u).regexp).body, (0 as i32)); ret = assigned; assigned };
vm_block = 381; continue;
}
// C line 26774
383 => {
return ((1 as i32)).wrapping_neg();
}
// C line 26773
384 => {
vm_block = if (js_parse_regexp(s)) != 0 { 383 } else { 382 }; continue;
}
// C line 26771
385 => {
return js_parse_error(s, c"RegExp are not supported".as_ptr());
}
// C line 26770 labels: parse_regexp
386 => {
vm_block = if ((!(((*((*(s)).ctx)).compile_regexp).is_some()) as i32)) != 0 { 385 } else { 384 }; continue;
}
// C line 26765
387 => {
let _ = { let old = (*(s)).buf_ptr; (*(s)).buf_ptr = ((*(s)).buf_ptr).offset(-1); old };
vm_block = 386; continue;
}
// C line 26763
388 => {
vm_block = 386; continue;
}
// C line 26762
389 => {
let _ = { (*(s)).buf_ptr = (((((*(s)).buf_ptr) as *const u8)).offset(-(((2 as i32)) as isize))) as *const u8; (*(s)).buf_ptr };
vm_block = 388; continue;
}
// C line 26759
390 => {
vm_block = 235; continue;
}
// C line 26758
391 => {
return ((1 as i32)).wrapping_neg();
}
// C line 26757
392 => {
vm_block = if (next_token(s)) != 0 { 391 } else { 390 }; continue;
}
// C line 26756
393 => {
return ((1 as i32)).wrapping_neg();
}
// C line 26755
394 => {
vm_block = if (emit_push_const(s, ((((*(s)).token).u).str).str, (1 as i32))) != 0 { 393 } else { 392 }; continue;
}
// C line 26753
395 => {
vm_block = 235; continue;
}
// C line 26752
396 => {
return ((1 as i32)).wrapping_neg();
}
// C line 26751
397 => {
vm_block = if (js_parse_template(s, (0 as i32), core::ptr::null_mut::<i32>())) != 0 { 396 } else { 395 }; continue;
}
// C line 26749
398 => {
vm_block = 235; continue;
}
// C line 26748
399 => {
return ((1 as i32)).wrapping_neg();
}
// C line 26747
400 => {
vm_block = if (next_token(s)) != 0 { 399 } else { 398 }; continue;
}
// C line 26731
401 => {
let _ = emit_u32(s, ((((((val).u).uint64) as i32)) as u32));
vm_block = 400; continue;
}
// C line 26730
402 => {
let _ = emit_op(s, (((OP_push_i32 as i32)) as u8));
vm_block = 401; continue;
}
// C line 26737
403 => {
let _ = emit_u32(s, ((v) as u32));
vm_block = 400; continue;
}
// C line 26736
404 => {
let _ = emit_op(s, (((OP_push_bigint_i32 as i32)) as u8));
vm_block = 403; continue;
}
// C line 26739
405 => {
vm_block = 409; continue;
}
// C line 26735
406 => {
vm_block = if ((((((((v) >= ((((((2147483647 as i32)).wrapping_neg()).wrapping_sub((1 as i32))) as i64))) as i32)) != 0) && (((((v) <= ((((2147483647 as i32)) as i64))) as i32)) != 0)) as i32)) != 0 { 404 } else { 405 }; continue;
}
// C line 26734
407 => {
let _ = { let assigned = ((val).u).short_big_int; v = assigned; assigned };
vm_block = 406; continue;
}
// C line 26744
408 => {
return ((1 as i32)).wrapping_neg();
}
// C line 26743 labels: large_number
409 => {
vm_block = if ((((emit_push_const(s, val, (0 as i32))) < ((0 as i32))) as i32)) != 0 { 408 } else { 400 }; continue;
}
// C line 26732
410 => {
vm_block = if (((((((val).tag) as i32)) == ((JS_TAG_SHORT_BIG_INT as i32))) as i32)) != 0 { 407 } else { 409 }; continue;
}
// C line 26729
411 => {
vm_block = if (((((((val).tag) as i32)) == ((JS_TAG_INT as i32))) as i32)) != 0 { 402 } else { 410 }; continue;
}
// C line 26727
412 => {
let _ = { let assigned = ((((*(s)).token).u).num).val; val = assigned; assigned };
vm_block = 411; continue;
}
// C line 26723
413 => {
vm_block = match ((*(s)).token).val { x if x == (TOK_IMPORT as i32) => 270, x if x == (TOK_SUPER as i32) => 288, x if x == (TOK_NEW as i32) => 312, x if x == (91 as i32) => 318, x if x == (123 as i32) => 318, x if x == (TOK_IDENT as i32) => 341, x if x == (TOK_TRUE as i32) => 345, x if x == (TOK_FALSE as i32) => 349, x if x == (TOK_THIS as i32) => 355, x if x == (TOK_NULL as i32) => 359, x if x == (TOK_CLASS as i32) => 362, x if x == (TOK_FUNCTION as i32) => 365, x if x == (40 as i32) => 368, x if x == (47 as i32) => 387, x if x == (TOK_DIV_ASSIGN as i32) => 389, x if x == (TOK_STRING as i32) => 394, x if x == (TOK_TEMPLATE as i32) => 397, x if x == (TOK_NUMBER as i32) => 412, _ => 236, }; continue;
}
// C line 26722
414 => {
let _ = { let assigned = (((FUNC_CALL_NORMAL as i32)) as FuncCallType); call_type = assigned; assigned };
vm_block = 413; continue;
}
// C line 26719
415 => {
accept_lparen = (((((parse_flags) & (((1 as i32)).wrapping_shl(((1 as i32)) as u32)))) != ((0 as i32))) as i32);
vm_block = 414; continue;
}
_ => std::process::abort(),
} }
}
