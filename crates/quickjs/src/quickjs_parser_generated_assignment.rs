// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:27882. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_parse_assign_expr2(mut s: *mut JSParseState, mut parse_flags: i32) -> i32 {
let mut vm_local_storage = Vec::<u64>::new();
let mut opcode: i32 = 0;
let mut op: i32 = 0;
let mut scope: i32 = 0;
let mut skip_bits: i32 = 0;
let mut name0: JSAtom = 0;
let mut name: JSAtom = 0;
let mut is_star: i32 = 0;
let mut is_async: i32 = 0;
let mut label_loop: i32 = 0;
let mut label_return: i32 = 0;
let mut label_next: i32 = 0;
let mut label_return1: i32 = 0;
let mut label_yield: i32 = 0;
let mut label_throw: i32 = 0;
let mut label_throw1: i32 = 0;
let mut label_throw2: i32 = 0;
let mut label_next_1: i32 = 0;
let mut source_ptr: *const u8 = core::ptr::null();
let mut tok: i32 = 0;
let mut pos: JSParsePos = core::mem::zeroed();
let mut label: i32 = 0;
let mut op_token_ptr: *const u8 = core::ptr::null();
let mut assign_opcodes: [u8; 12] = core::mem::zeroed();
let mut label_1: i32 = 0;
let mut label1: i32 = 0;
let mut depth_lvalue: i32 = 0;
let mut label2: i32 = 0;
let mut vm_block: usize = 175;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 28163
1 => {
return (0 as i32);
}
// C line 28104
2 => {
let _ = put_lvalue(s, opcode, scope, name, label, (((PUT_LVALUE_KEEP_TOP as i32)) as PutLValueEnum), (0 as i32));
vm_block = 1; continue;
}
// C line 28092
3 => {
let _ = set_object_name(s, name);
vm_block = 2; continue;
}
// C line 28091
4 => {
vm_block = if ((((((((((((opcode) == ((OP_get_ref_value as i32))) as i32)) != 0) || (((((opcode) == ((OP_scope_get_var as i32))) as i32)) != 0)) as i32)) != 0) && (((((name) == (name0)) as i32)) != 0)) as i32)) != 0 { 3 } else { 2 }; continue;
}
// C line 28102
5 => {
let _ = emit_op(s, ((op) as u8));
vm_block = 2; continue;
}
// C line 28101
6 => {
let _ = emit_source_pos(s, op_token_ptr);
vm_block = 5; continue;
}
// C line 28100
7 => {
let _ = { let assigned = ((*((assign_opcodes).as_ptr()).offset(((op).wrapping_sub((TOK_MUL_ASSIGN as i32))) as isize)) as i32); op = assigned; assigned };
vm_block = 6; continue;
}
// C line 28095
8 => {
assign_opcodes = [(((OP_mul as i32)) as u8), (((OP_div as i32)) as u8), (((OP_mod as i32)) as u8), (((OP_add as i32)) as u8), (((OP_sub as i32)) as u8), (((OP_shl as i32)) as u8), (((OP_sar as i32)) as u8), (((OP_shr as i32)) as u8), (((OP_and as i32)) as u8), (((OP_xor as i32)) as u8), (((OP_or as i32)) as u8), (((OP_pow as i32)) as u8)];
vm_block = 7; continue;
}
// C line 28090
9 => {
vm_block = if ((((op) == ((61 as i32))) as i32)) != 0 { 4 } else { 8 }; continue;
}
// C line 28087
10 => {
return ((1 as i32)).wrapping_neg();
}
// C line 28086
11 => {
let _ = JS_FreeAtom((*(s)).ctx, name);
vm_block = 10; continue;
}
// C line 28085
12 => {
vm_block = if (js_parse_assign_expr2(s, parse_flags)) != 0 { 11 } else { 9 }; continue;
}
// C line 28083
13 => {
return ((1 as i32)).wrapping_neg();
}
// C line 28082
14 => {
vm_block = if ((((get_lvalue(s, core::ptr::addr_of_mut!(opcode), core::ptr::addr_of_mut!(scope), core::ptr::addr_of_mut!(name), core::ptr::addr_of_mut!(label), core::ptr::null_mut::<i32>(), (((op) != ((61 as i32))) as i32), op)) < ((0 as i32))) as i32)) != 0 { 13 } else { 12 }; continue;
}
// C line 28081
15 => {
return ((1 as i32)).wrapping_neg();
}
// C line 28080
16 => {
vm_block = if (next_token(s)) != 0 { 15 } else { 14 }; continue;
}
// C line 28079
17 => {
let _ = { let assigned = ((*(s)).token).ptr; op_token_ptr = assigned; assigned };
vm_block = 16; continue;
}
// C line 28161
18 => {
let _ = emit_label(s, label2);
vm_block = 1; continue;
}
// C line 28156
19 => {
vm_block = if ((((depth_lvalue) != ((0 as i32))) as i32)) != 0 { 21 } else { 18 }; continue;
}
// C line 28158
20 => {
let _ = { let old = depth_lvalue; depth_lvalue = (depth_lvalue).wrapping_sub(1); old };
vm_block = 19; continue;
}
// C line 28157
21 => {
let _ = emit_op(s, (((OP_nip as i32)) as u8));
vm_block = 20; continue;
}
// C line 28153
22 => {
let _ = emit_label(s, label1);
vm_block = 19; continue;
}
// C line 28151
23 => {
let _ = { let assigned = emit_goto(s, (OP_goto as i32), ((1 as i32)).wrapping_neg()); label2 = assigned; assigned };
vm_block = 22; continue;
}
// C line 28149
24 => {
let _ = put_lvalue(s, opcode, scope, name, label_1, (((PUT_LVALUE_NOKEEP_DEPTH as i32)) as PutLValueEnum), (0 as i32));
vm_block = 23; continue;
}
// C line 28144
25 => {
let _ = std::process::abort();
vm_block = 24; continue;
}
// C line 28142
26 => {
vm_block = 24; continue;
}
// C line 28141
27 => {
let _ = emit_op(s, (((OP_insert4 as i32)) as u8));
vm_block = 26; continue;
}
// C line 28139
28 => {
vm_block = 24; continue;
}
// C line 28138
29 => {
let _ = emit_op(s, (((OP_insert3 as i32)) as u8));
vm_block = 28; continue;
}
// C line 28136
30 => {
vm_block = 24; continue;
}
// C line 28135
31 => {
let _ = emit_op(s, (((OP_insert2 as i32)) as u8));
vm_block = 30; continue;
}
// C line 28133
32 => {
vm_block = 24; continue;
}
// C line 28132
33 => {
let _ = emit_op(s, (((OP_dup as i32)) as u8));
vm_block = 32; continue;
}
// C line 28130
34 => {
vm_block = match depth_lvalue { x if x == (3 as i32) => 27, x if x == (2 as i32) => 29, x if x == (1 as i32) => 31, x if x == (0 as i32) => 33, _ => 25, }; continue;
}
// C line 28127
35 => {
let _ = set_object_name(s, name);
vm_block = 34; continue;
}
// C line 28126
36 => {
vm_block = if ((((((((((((opcode) == ((OP_get_ref_value as i32))) as i32)) != 0) || (((((opcode) == ((OP_scope_get_var as i32))) as i32)) != 0)) as i32)) != 0) && (((((name) == (name0)) as i32)) != 0)) as i32)) != 0 { 35 } else { 34 }; continue;
}
// C line 28123
37 => {
return ((1 as i32)).wrapping_neg();
}
// C line 28122
38 => {
let _ = JS_FreeAtom((*(s)).ctx, name);
vm_block = 37; continue;
}
// C line 28121
39 => {
vm_block = if (js_parse_assign_expr2(s, parse_flags)) != 0 { 38 } else { 36 }; continue;
}
// C line 28119
40 => {
let _ = emit_op(s, (((OP_drop as i32)) as u8));
vm_block = 39; continue;
}
// C line 28117
41 => {
let _ = { let assigned = emit_goto(s, if ((((op) == ((TOK_LOR_ASSIGN as i32))) as i32)) != 0 { (OP_if_true as i32) } else { (OP_if_false as i32) }, ((1 as i32)).wrapping_neg()); label1 = assigned; assigned };
vm_block = 40; continue;
}
// C line 28116
42 => {
let _ = emit_op(s, (((OP_is_undefined_or_null as i32)) as u8));
vm_block = 41; continue;
}
// C line 28115
43 => {
vm_block = if ((((op) == ((TOK_DOUBLE_QUESTION_MARK_ASSIGN as i32))) as i32)) != 0 { 42 } else { 41 }; continue;
}
// C line 28114
44 => {
let _ = emit_op(s, (((OP_dup as i32)) as u8));
vm_block = 43; continue;
}
// C line 28112
45 => {
return ((1 as i32)).wrapping_neg();
}
// C line 28110
46 => {
vm_block = if ((((get_lvalue(s, core::ptr::addr_of_mut!(opcode), core::ptr::addr_of_mut!(scope), core::ptr::addr_of_mut!(name), core::ptr::addr_of_mut!(label_1), core::ptr::addr_of_mut!(depth_lvalue), (1 as i32), op)) < ((0 as i32))) as i32)) != 0 { 45 } else { 44 }; continue;
}
// C line 28109
47 => {
return ((1 as i32)).wrapping_neg();
}
// C line 28108
48 => {
vm_block = if (next_token(s)) != 0 { 47 } else { 46 }; continue;
}
// C line 28105
49 => {
vm_block = if ((((((((op) >= ((TOK_LAND_ASSIGN as i32))) as i32)) != 0) && (((((op) <= ((TOK_DOUBLE_QUESTION_MARK_ASSIGN as i32))) as i32)) != 0)) as i32)) != 0 { 48 } else { 1 }; continue;
}
// C line 28076
50 => {
vm_block = if ((((((((op) == ((61 as i32))) as i32)) != 0) || (((((((((op) >= ((TOK_MUL_ASSIGN as i32))) as i32)) != 0) && (((((op) <= ((TOK_POW_ASSIGN as i32))) as i32)) != 0)) as i32)) != 0)) as i32)) != 0 { 17 } else { 49 }; continue;
}
// C line 28075
51 => {
let _ = { let assigned = ((*(s)).token).val; op = assigned; assigned };
vm_block = 50; continue;
}
// C line 28073
52 => {
return ((1 as i32)).wrapping_neg();
}
// C line 28072
53 => {
vm_block = if (js_parse_cond_expr(s, parse_flags)) != 0 { 52 } else { 51 }; continue;
}
// C line 28070
54 => {
let _ = { let assigned = ((((*(s)).token).u).ident).atom; name0 = assigned; assigned };
vm_block = 53; continue;
}
// C line 28068 labels: next
55 => {
vm_block = if ((((((*(s)).token).val) == ((TOK_IDENT as i32))) as i32)) != 0 { 54 } else { 53 }; continue;
}
// C line 28024
56 => {
return (0 as i32);
}
// C line 28013
57 => {
let _ = emit_op(s, (((OP_nip as i32)) as u8));
vm_block = 56; continue;
}
// C line 28012
58 => {
let _ = emit_op(s, (((OP_nip as i32)) as u8));
vm_block = 57; continue;
}
// C line 28010
59 => {
let _ = emit_op(s, (((OP_nip as i32)) as u8));
vm_block = 58; continue;
}
// C line 28009
60 => {
let _ = emit_atom(s, (((crate::quickjs_atom::JS_ATOM_value as i32)) as JSAtom));
vm_block = 59; continue;
}
// C line 28008
61 => {
let _ = emit_op(s, (((OP_get_field as i32)) as u8));
vm_block = 60; continue;
}
// C line 28007
62 => {
let _ = emit_label(s, label_next);
vm_block = 61; continue;
}
// C line 28005
63 => {
let _ = emit_u8(s, (((4 as i32)) as u8));
vm_block = 62; continue;
}
// C line 28004
64 => {
let _ = emit_atom(s, (((0 as i32)) as JSAtom));
vm_block = 63; continue;
}
// C line 28003
65 => {
let _ = emit_op(s, (((OP_throw_error as i32)) as u8));
vm_block = 64; continue;
}
// C line 28001
66 => {
let _ = emit_label(s, label_throw2);
vm_block = 65; continue;
}
// C line 28000
67 => {
let _ = emit_op(s, (((OP_await as i32)) as u8));
vm_block = 66; continue;
}
// C line 27999
68 => {
vm_block = if (is_async) != 0 { 67 } else { 66 }; continue;
}
// C line 27998
69 => {
let _ = { let assigned = emit_goto(s, (OP_if_true as i32), ((1 as i32)).wrapping_neg()); label_throw2 = assigned; assigned };
vm_block = 68; continue;
}
// C line 27997
70 => {
let _ = emit_u8(s, (((2 as i32)) as u8));
vm_block = 69; continue;
}
// C line 27996
71 => {
let _ = emit_op(s, (((OP_iterator_call as i32)) as u8));
vm_block = 70; continue;
}
// C line 27995
72 => {
let _ = emit_label(s, label_throw1);
vm_block = 71; continue;
}
// C line 27993
73 => {
let _ = emit_goto(s, (OP_goto as i32), label_next);
vm_block = 72; continue;
}
// C line 27992
74 => {
let _ = emit_goto(s, (OP_if_false as i32), label_yield);
vm_block = 73; continue;
}
// C line 27991
75 => {
let _ = emit_atom(s, (((crate::quickjs_atom::JS_ATOM_done as i32)) as JSAtom));
vm_block = 74; continue;
}
// C line 27990
76 => {
let _ = emit_op(s, (((OP_get_field2 as i32)) as u8));
vm_block = 75; continue;
}
// C line 27989
77 => {
let _ = emit_op(s, (((OP_iterator_check_object as i32)) as u8));
vm_block = 76; continue;
}
// C line 27988
78 => {
let _ = emit_op(s, (((OP_await as i32)) as u8));
vm_block = 77; continue;
}
// C line 27987
79 => {
vm_block = if (is_async) != 0 { 78 } else { 77 }; continue;
}
// C line 27986
80 => {
let _ = { let assigned = emit_goto(s, (OP_if_true as i32), ((1 as i32)).wrapping_neg()); label_throw1 = assigned; assigned };
vm_block = 79; continue;
}
// C line 27985
81 => {
let _ = emit_u8(s, (((1 as i32)) as u8));
vm_block = 80; continue;
}
// C line 27984
82 => {
let _ = emit_op(s, (((OP_iterator_call as i32)) as u8));
vm_block = 81; continue;
}
// C line 27983
83 => {
let _ = emit_label(s, label_throw);
vm_block = 82; continue;
}
// C line 27980
84 => {
let _ = emit_return(s, (1 as i32));
vm_block = 83; continue;
}
// C line 27979
85 => {
let _ = emit_op(s, (((OP_nip as i32)) as u8));
vm_block = 84; continue;
}
// C line 27978
86 => {
let _ = emit_op(s, (((OP_nip as i32)) as u8));
vm_block = 85; continue;
}
// C line 27977
87 => {
let _ = emit_op(s, (((OP_nip as i32)) as u8));
vm_block = 86; continue;
}
// C line 27976
88 => {
let _ = emit_label(s, label_return1);
vm_block = 87; continue;
}
// C line 27974
89 => {
let _ = emit_atom(s, (((crate::quickjs_atom::JS_ATOM_value as i32)) as JSAtom));
vm_block = 88; continue;
}
// C line 27973
90 => {
let _ = emit_op(s, (((OP_get_field as i32)) as u8));
vm_block = 89; continue;
}
// C line 27971
91 => {
let _ = emit_goto(s, (OP_if_false as i32), label_yield);
vm_block = 90; continue;
}
// C line 27970
92 => {
let _ = emit_atom(s, (((crate::quickjs_atom::JS_ATOM_done as i32)) as JSAtom));
vm_block = 91; continue;
}
// C line 27969
93 => {
let _ = emit_op(s, (((OP_get_field2 as i32)) as u8));
vm_block = 92; continue;
}
// C line 27968
94 => {
let _ = emit_op(s, (((OP_iterator_check_object as i32)) as u8));
vm_block = 93; continue;
}
// C line 27967
95 => {
let _ = emit_op(s, (((OP_await as i32)) as u8));
vm_block = 94; continue;
}
// C line 27966
96 => {
vm_block = if (is_async) != 0 { 95 } else { 94 }; continue;
}
// C line 27965
97 => {
let _ = { let assigned = emit_goto(s, (OP_if_true as i32), ((1 as i32)).wrapping_neg()); label_return1 = assigned; assigned };
vm_block = 96; continue;
}
// C line 27964
98 => {
let _ = emit_u8(s, (((0 as i32)) as u8));
vm_block = 97; continue;
}
// C line 27963
99 => {
let _ = emit_op(s, (((OP_iterator_call as i32)) as u8));
vm_block = 98; continue;
}
// C line 27962
100 => {
let _ = emit_op(s, (((OP_await as i32)) as u8));
vm_block = 99; continue;
}
// C line 27961
101 => {
vm_block = if (is_async) != 0 { 100 } else { 99 }; continue;
}
// C line 27958
102 => {
let _ = { let assigned = emit_goto(s, (OP_if_true as i32), ((1 as i32)).wrapping_neg()); label_throw = assigned; assigned };
vm_block = 101; continue;
}
// C line 27957
103 => {
let _ = emit_op(s, (((OP_strict_eq as i32)) as u8));
vm_block = 102; continue;
}
// C line 27956
104 => {
let _ = emit_u32(s, (((2 as i32)) as u32));
vm_block = 103; continue;
}
// C line 27955
105 => {
let _ = emit_op(s, (((OP_push_i32 as i32)) as u8));
vm_block = 104; continue;
}
// C line 27954
106 => {
let _ = emit_label(s, label_return);
vm_block = 105; continue;
}
// C line 27952
107 => {
let _ = emit_goto(s, (OP_goto as i32), label_loop);
vm_block = 106; continue;
}
// C line 27951
108 => {
let _ = emit_op(s, (((OP_drop as i32)) as u8));
vm_block = 107; continue;
}
// C line 27950
109 => {
let _ = { let assigned = emit_goto(s, (OP_if_true as i32), ((1 as i32)).wrapping_neg()); label_return = assigned; assigned };
vm_block = 108; continue;
}
// C line 27949
110 => {
let _ = emit_op(s, (((OP_dup as i32)) as u8));
vm_block = 109; continue;
}
// C line 27944
111 => {
let _ = emit_op(s, (((OP_async_yield_star as i32)) as u8));
vm_block = 110; continue;
}
// C line 27943
112 => {
let _ = emit_atom(s, (((crate::quickjs_atom::JS_ATOM_value as i32)) as JSAtom));
vm_block = 111; continue;
}
// C line 27942
113 => {
let _ = emit_op(s, (((OP_get_field as i32)) as u8));
vm_block = 112; continue;
}
// C line 27947
114 => {
let _ = emit_op(s, (((OP_yield_star as i32)) as u8));
vm_block = 110; continue;
}
// C line 27940
115 => {
vm_block = if (is_async) != 0 { 113 } else { 114 }; continue;
}
// C line 27939
116 => {
let _ = emit_label(s, label_yield);
vm_block = 115; continue;
}
// C line 27938
117 => {
let _ = { let assigned = emit_goto(s, (OP_if_true as i32), ((1 as i32)).wrapping_neg()); label_next = assigned; assigned };
vm_block = 116; continue;
}
// C line 27937
118 => {
let _ = emit_atom(s, (((crate::quickjs_atom::JS_ATOM_done as i32)) as JSAtom));
vm_block = 117; continue;
}
// C line 27936
119 => {
let _ = emit_op(s, (((OP_get_field2 as i32)) as u8));
vm_block = 118; continue;
}
// C line 27935
120 => {
let _ = emit_op(s, (((OP_iterator_check_object as i32)) as u8));
vm_block = 119; continue;
}
// C line 27934
121 => {
let _ = emit_op(s, (((OP_await as i32)) as u8));
vm_block = 120; continue;
}
// C line 27933
122 => {
vm_block = if (is_async) != 0 { 121 } else { 120 }; continue;
}
// C line 27932
123 => {
let _ = emit_op(s, (((OP_iterator_next as i32)) as u8));
vm_block = 122; continue;
}
// C line 27931
124 => {
let _ = emit_label(s, label_loop);
vm_block = 123; continue;
}
// C line 27929
125 => {
let _ = emit_op(s, (((OP_undefined as i32)) as u8));
vm_block = 124; continue;
}
// C line 27927
126 => {
let _ = emit_op(s, (((OP_undefined as i32)) as u8));
vm_block = 125; continue;
}
// C line 27926
127 => {
let _ = emit_op(s, (((OP_drop as i32)) as u8));
vm_block = 126; continue;
}
// C line 27922
128 => {
let _ = emit_op(s, ((if (is_async) != 0 { (OP_for_await_of_start as i32) } else { (OP_for_of_start as i32) }) as u8));
vm_block = 127; continue;
}
// C line 27920
129 => {
let _ = { let assigned = new_label(s); label_yield = assigned; assigned };
vm_block = 128; continue;
}
// C line 27919
130 => {
let _ = { let assigned = new_label(s); label_loop = assigned; assigned };
vm_block = 129; continue;
}
// C line 28022
131 => {
let _ = emit_label(s, label_next_1);
vm_block = 56; continue;
}
// C line 28021
132 => {
let _ = emit_return(s, (1 as i32));
vm_block = 131; continue;
}
// C line 28020
133 => {
let _ = { let assigned = emit_goto(s, (OP_if_false as i32), ((1 as i32)).wrapping_neg()); label_next_1 = assigned; assigned };
vm_block = 132; continue;
}
// C line 28019
134 => {
let _ = emit_op(s, (((OP_yield as i32)) as u8));
vm_block = 133; continue;
}
// C line 28018
135 => {
let _ = emit_op(s, (((OP_await as i32)) as u8));
vm_block = 134; continue;
}
// C line 28017
136 => {
vm_block = if (is_async) != 0 { 135 } else { 134 }; continue;
}
// C line 27914
137 => {
vm_block = if (is_star) != 0 { 130 } else { 136 }; continue;
}
// C line 27912
138 => {
let _ = { let assigned = (((((((*((*(s)).cur_func)).func_kind as JSFunctionKindEnum)) as i32)) == ((JS_FUNC_ASYNC_GENERATOR as i32))) as i32); is_async = assigned; assigned };
vm_block = 137; continue;
}
// C line 27908
139 => {
return ((1 as i32)).wrapping_neg();
}
// C line 27907
140 => {
vm_block = if (js_parse_assign_expr2(s, parse_flags)) != 0 { 139 } else { 138 }; continue;
}
// C line 27905
141 => {
return ((1 as i32)).wrapping_neg();
}
// C line 27904
142 => {
vm_block = if (next_token(s)) != 0 { 141 } else { 140 }; continue;
}
// C line 27903
143 => {
let _ = { let assigned = (1 as i32); is_star = assigned; assigned };
vm_block = 142; continue;
}
// C line 27902
144 => {
vm_block = if ((((((*(s)).token).val) == ((42 as i32))) as i32)) != 0 { 143 } else { 140 }; continue;
}
// C line 27910
145 => {
let _ = emit_op(s, (((OP_undefined as i32)) as u8));
vm_block = 138; continue;
}
// C line 27899
146 => {
vm_block = if ((((((((((((((((((((((((((((((*(s)).token).val) != ((59 as i32))) as i32)) != 0) && (((((((*(s)).token).val) != ((41 as i32))) as i32)) != 0)) as i32)) != 0) && (((((((*(s)).token).val) != ((93 as i32))) as i32)) != 0)) as i32)) != 0) && (((((((*(s)).token).val) != ((125 as i32))) as i32)) != 0)) as i32)) != 0) && (((((((*(s)).token).val) != ((44 as i32))) as i32)) != 0)) as i32)) != 0) && (((((((*(s)).token).val) != ((58 as i32))) as i32)) != 0)) as i32)) != 0) && (((!(((*(s)).got_lf) != 0) as i32)) != 0)) as i32)) != 0 { 144 } else { 145 }; continue;
}
// C line 27896
147 => {
return ((1 as i32)).wrapping_neg();
}
// C line 27895
148 => {
vm_block = if (next_token(s)) != 0 { 147 } else { 146 }; continue;
}
// C line 27894
149 => {
return js_parse_error(s, c"yield in default expression".as_ptr());
}
// C line 27893
150 => {
vm_block = if ((!(((*((*(s)).cur_func)).in_function_body) != 0) as i32)) != 0 { 149 } else { 148 }; continue;
}
// C line 27892
151 => {
return js_parse_error(s, c"unexpected 'yield' keyword".as_ptr());
}
// C line 27891
152 => {
vm_block = if ((!((((((((*((*(s)).cur_func)).func_kind as JSFunctionKindEnum)) as i32)) & ((JS_FUNC_GENERATOR as i32)))) != 0) as i32)) != 0 { 151 } else { 150 }; continue;
}
// C line 27889
153 => {
is_star = (0 as i32);
vm_block = 152; continue;
}
// C line 28027
154 => {
return js_parse_function_decl(s, (((JS_PARSE_FUNC_ARROW as i32)) as JSParseFunctionEnum), (((JS_FUNC_NORMAL as i32)) as JSFunctionKindEnum), (((0 as i32)) as JSAtom), ((*(s)).token).ptr);
}
// C line 28048
155 => {
return js_parse_function_decl(s, (((JS_PARSE_FUNC_ARROW as i32)) as JSParseFunctionEnum), (((JS_FUNC_ASYNC as i32)) as JSFunctionKindEnum), (((0 as i32)) as JSAtom), source_ptr);
}
// C line 28054
156 => {
return ((1 as i32)).wrapping_neg();
}
// C line 28053
157 => {
vm_block = if (js_parse_seek_token(s, core::ptr::addr_of_mut!(pos))) != 0 { 156 } else { 55 }; continue;
}
// C line 28044
158 => {
vm_block = if ((((((((((((((*(s)).token).val) == ((40 as i32))) as i32)) != 0) && (((((js_parse_skip_parens_token(s, core::ptr::null_mut::<i32>(), (1 as i32))) == ((TOK_ARROW as i32))) as i32)) != 0)) as i32)) != 0) || (((((((((((((((*(s)).token).val) == ((TOK_IDENT as i32))) as i32)) != 0) && (((!((((((*(s)).token).u).ident).is_reserved) != 0) as i32)) != 0)) as i32)) != 0) && (((((peek_token(s, (1 as i32))) == ((TOK_ARROW as i32))) as i32)) != 0)) as i32)) != 0)) as i32)) != 0 { 155 } else { 157 }; continue;
}
// C line 28043
159 => {
return ((1 as i32)).wrapping_neg();
}
// C line 28042
160 => {
vm_block = if (next_token(s)) != 0 { 159 } else { 158 }; continue;
}
// C line 28041
161 => {
let _ = js_parse_get_pos(s, core::ptr::addr_of_mut!(pos));
vm_block = 160; continue;
}
// C line 28040
162 => {
let _ = { let assigned = ((*(s)).token).ptr; source_ptr = assigned; assigned };
vm_block = 161; continue;
}
// C line 28038
163 => {
vm_block = 55; continue;
}
// C line 28037
164 => {
vm_block = if ((((((((tok) == ((TOK_FUNCTION as i32))) as i32)) != 0) || (((((tok) == ((10 as i32))) as i32)) != 0)) as i32)) != 0 { 163 } else { 162 }; continue;
}
// C line 28036
165 => {
let _ = { let assigned = peek_token(s, (1 as i32)); tok = assigned; assigned };
vm_block = 164; continue;
}
// C line 28058
166 => {
return js_parse_function_decl(s, (((JS_PARSE_FUNC_ARROW as i32)) as JSParseFunctionEnum), (((JS_FUNC_NORMAL as i32)) as JSFunctionKindEnum), (((0 as i32)) as JSAtom), ((*(s)).token).ptr);
}
// C line 28065
167 => {
return (0 as i32);
}
// C line 28064
168 => {
return ((1 as i32)).wrapping_neg();
}
// C line 28063
169 => {
vm_block = if ((((js_parse_destructuring_element(s, (0 as i32), (0 as i32), (0 as i32), ((skip_bits) & (((1 as i32)).wrapping_shl(((1 as i32)) as u32))), (1 as i32), (0 as i32))) < ((0 as i32))) as i32)) != 0 { 168 } else { 167 }; continue;
}
// C line 28061
170 => {
vm_block = if ((((((((((((((*(s)).token).val) == ((123 as i32))) as i32)) != 0) || (((((((*(s)).token).val) == ((91 as i32))) as i32)) != 0)) as i32)) != 0) && (((((js_parse_skip_parens_token(s, core::ptr::addr_of_mut!(skip_bits), (0 as i32))) == ((61 as i32))) as i32)) != 0)) as i32)) != 0 { 169 } else { 55 }; continue;
}
// C line 28056
171 => {
vm_block = if ((((((((((*(s)).token).val) == ((TOK_IDENT as i32))) as i32)) != 0) && (((((peek_token(s, (1 as i32))) == ((TOK_ARROW as i32))) as i32)) != 0)) as i32)) != 0 { 166 } else { 170 }; continue;
}
// C line 28030
172 => {
vm_block = if (token_is_pseudo_keyword(s, (((crate::quickjs_atom::JS_ATOM_async as i32)) as JSAtom))) != 0 { 165 } else { 171 }; continue;
}
// C line 28025
173 => {
vm_block = if ((((((((((*(s)).token).val) == ((40 as i32))) as i32)) != 0) && (((((js_parse_skip_parens_token(s, core::ptr::null_mut::<i32>(), (1 as i32))) == ((TOK_ARROW as i32))) as i32)) != 0)) as i32)) != 0 { 154 } else { 172 }; continue;
}
// C line 27888
174 => {
vm_block = if ((((((*(s)).token).val) == ((TOK_YIELD as i32))) as i32)) != 0 { 153 } else { 173 }; continue;
}
// C line 27885
175 => {
name0 = (((0 as i32)) as JSAtom);
vm_block = 174; continue;
}
_ => std::process::abort(),
} }
}
