// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:32107. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn skip_lines(mut p: *const c_char, mut n: i32) -> *const c_char {
let mut vm_local_storage = Vec::<u64>::new();
let mut vm_block: usize = 2;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 32112
1 => {
return p;
}
// C line 32108
2 => {
vm_block = if (((((((({ let old = n; n = (n).wrapping_sub(1); old }) > ((0 as i32))) as i32)) != 0) && ((((*(p)) as i32)) != 0)) as i32)) != 0 { 3 } else { 1 }; continue;
}
// C line 32109
3 => {
vm_block = if (((((((*(p)) as i32)) != 0) && (((((((*({ let old = p; p = (p).offset(1); old })) as i32)) != ((10 as i32))) as i32)) != 0)) as i32)) != 0 { 4 } else { 2 }; continue;
}
// C line 32110
4 => {
vm_block = 3; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:32115. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn print_lines(mut source: *const c_char, mut line: i32, mut line1: i32) -> () {
let mut vm_local_storage = Vec::<u64>::new();
let mut s: *const c_char = core::mem::zeroed();
let mut p: *const c_char = core::mem::zeroed();
let mut vm_block: usize = 10;
loop { match vm_block {
// C line ?
0 => {
return ();
}
// C line 32119
1 => {
vm_block = if (((({ let old = line; line = (line).wrapping_add(1); old }) < (line1)) as i32)) != 0 { 7 } else { 0 }; continue;
}
// C line 32125
2 => {
vm_block = 0; continue;
}
// C line 32124
3 => {
let _ = quickjs_debug_write(&quickjs_format_print(c"\n".as_ptr(), &[]));
vm_block = 2; continue;
}
// C line 32123
4 => {
vm_block = if ((((((*(p).offset((((1 as i32)).wrapping_neg()) as isize)) as i32)) != ((10 as i32))) as i32)) != 0 { 3 } else { 2 }; continue;
}
// C line 32122
5 => {
vm_block = if ((!((*(p)) != 0) as i32)) != 0 { 4 } else { 1 }; continue;
}
// C line 32121
6 => {
let _ = quickjs_debug_write(&quickjs_format_print(c";; %.*s".as_ptr(), &[QuickJSPrintArg::Int(((((p).offset_from(s) as i64)) as i32) as u64), QuickJSPrintArg::Str(s as *const c_char)]));
vm_block = 5; continue;
}
// C line 32120
7 => {
let _ = { let assigned = skip_lines({ let assigned = p; s = assigned; assigned }, (1 as i32)); p = assigned; assigned };
vm_block = 6; continue;
}
// C line 32118
8 => {
vm_block = if (*(p)) != 0 { 1 } else { 0 }; continue;
}
// C line 32117
9 => {
p = skip_lines(s, line);
vm_block = 8; continue;
}
// C line 32116
10 => {
s = source;
vm_block = 9; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:32131. Bellard/Gordon MIT.
#[cfg(feature = "short-opcodes")]
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn dump_byte_code(mut ctx: *mut JSContext, mut pass: i32, mut tab: *const u8, mut len: i32, mut vardefs: *const JSBytecodeVarDef, mut args: *const JSVarDef, mut arg_count: i32, mut vars: *const JSVarDef, mut var_count: i32, mut closure_var: *const JSClosureVar, mut closure_var_count: i32, mut cpool: *const JSValue, mut cpool_count: u32, mut source: *const c_char, mut label_slots: *const LabelSlot, mut b: *mut JSFunctionBytecode) -> () {
let mut vm_local_storage = Vec::<u64>::new();
let mut oi: *const JSOpCode = core::mem::zeroed();
let mut pos: i32 = core::mem::zeroed();
let mut pos_next: i32 = core::mem::zeroed();
let mut op: i32 = core::mem::zeroed();
let mut size: i32 = core::mem::zeroed();
let mut idx: i32 = core::mem::zeroed();
let mut addr: i32 = core::mem::zeroed();
let mut line: i32 = core::mem::zeroed();
let mut line1: i32 = core::mem::zeroed();
let mut in_source: i32 = core::mem::zeroed();
let mut line_num: i32 = core::mem::zeroed();
let mut bits: *mut u8 = core::mem::zeroed();
let mut use_short_opcodes: i32 = core::mem::zeroed();
let mut dump_pc: i32 = core::mem::zeroed();
let mut col_num: i32 = core::mem::zeroed();
let mut col_num_1: i32 = core::mem::zeroed();
let mut i: i32 = core::mem::zeroed();
let mut x: i32 = core::mem::zeroed();
let mut x0: i32 = core::mem::zeroed();
let mut vm_block: usize = 184;
loop { match vm_block {
// C line ?
0 => {
return ();
}
// C line 32411
1 => {
let _ = js_free(ctx, ((bits) as *mut c_void));
vm_block = 0; continue;
}
// C line 32409
2 => {
let _ = print_lines(source, line, (2147483647 as i32));
vm_block = 1; continue;
}
// C line 32408
3 => {
let _ = quickjs_debug_write(&quickjs_format_print(c"\n".as_ptr(), &[]));
vm_block = 2; continue;
}
// C line 32407
4 => {
vm_block = if ((!((in_source) != 0) as i32)) != 0 { 3 } else { 2 }; continue;
}
// C line 32406
5 => {
vm_block = if !(source).is_null() { 4 } else { 1 }; continue;
}
// C line 32201
6 => {
vm_block = if ((((pos) < (len)) as i32)) != 0 { 145 } else { 5 }; continue;
}
// C line 32404
7 => {
let _ = { pos = ((((pos) as i32)).wrapping_add(((((*(oi)).size) as i32)).wrapping_sub((1 as i32)))) as i32; pos };
vm_block = 6; continue;
}
// C line 32403
8 => {
let _ = quickjs_debug_write(&quickjs_format_print(c"\n".as_ptr(), &[]));
vm_block = 7; continue;
}
// C line 32401
9 => {
vm_block = 8; continue;
}
// C line 32399
10 => {
vm_block = 8; continue;
}
// C line 32397
11 => {
let _ = print_atom(ctx, (*(closure_var).offset((idx) as isize)).var_name);
vm_block = 10; continue;
}
// C line 32396
12 => {
vm_block = if ((((idx) < (closure_var_count)) as i32)) != 0 { 11 } else { 10 }; continue;
}
// C line 32395 labels: has_var_ref
13 => {
let _ = quickjs_debug_write(&quickjs_format_print(c" %d: ".as_ptr(), &[QuickJSPrintArg::Int(idx as u64)]));
vm_block = 12; continue;
}
// C line 32393
14 => {
let _ = { let assigned = ((crate::cutils_header::get_u16((tab).offset(((pos) as isize)))) as i32); idx = assigned; assigned };
vm_block = 13; continue;
}
// C line 32391
15 => {
vm_block = 13; continue;
}
// C line 32390
16 => {
let _ = { let assigned = (((op).wrapping_sub((OP_get_var_ref0 as i32))) % ((4 as i32))); idx = assigned; assigned };
vm_block = 15; continue;
}
// C line 32388
17 => {
vm_block = 8; continue;
}
// C line 32386
18 => {
let _ = print_atom(ctx, if !(args).is_null() { (*(args).offset((idx) as isize)).var_name } else { (*(vardefs).offset((idx) as isize)).var_name });
vm_block = 17; continue;
}
// C line 32385
19 => {
vm_block = if ((((idx) < (arg_count)) as i32)) != 0 { 18 } else { 17 }; continue;
}
// C line 32384 labels: has_arg
20 => {
let _ = quickjs_debug_write(&quickjs_format_print(c" %d: ".as_ptr(), &[QuickJSPrintArg::Int(idx as u64)]));
vm_block = 19; continue;
}
// C line 32382
21 => {
let _ = { let assigned = ((crate::cutils_header::get_u16((tab).offset(((pos) as isize)))) as i32); idx = assigned; assigned };
vm_block = 20; continue;
}
// C line 32380
22 => {
vm_block = 20; continue;
}
// C line 32379
23 => {
let _ = { let assigned = (((op).wrapping_sub((OP_get_arg0 as i32))) % ((4 as i32))); idx = assigned; assigned };
vm_block = 22; continue;
}
// C line 32377
24 => {
vm_block = 8; continue;
}
// C line 32375
25 => {
let _ = print_atom(ctx, if !(vars).is_null() { (*(vars).offset((idx) as isize)).var_name } else { (*(vardefs).offset(((arg_count).wrapping_add(idx)) as isize)).var_name });
vm_block = 24; continue;
}
// C line 32374
26 => {
vm_block = if ((((idx) < (var_count)) as i32)) != 0 { 25 } else { 24 }; continue;
}
// C line 32373 labels: has_loc
27 => {
let _ = quickjs_debug_write(&quickjs_format_print(c" %d: ".as_ptr(), &[QuickJSPrintArg::Int(idx as u64)]));
vm_block = 26; continue;
}
// C line 32371
28 => {
let _ = { let assigned = ((crate::cutils_header::get_u16((tab).offset(((pos) as isize)))) as i32); idx = assigned; assigned };
vm_block = 27; continue;
}
// C line 32369
29 => {
vm_block = 27; continue;
}
// C line 32368
30 => {
let _ = { let assigned = ((crate::cutils_header::get_u8((tab).offset(((pos) as isize)))) as i32); idx = assigned; assigned };
vm_block = 29; continue;
}
// C line 32366
31 => {
vm_block = 27; continue;
}
// C line 32365
32 => {
let _ = { let assigned = (((op).wrapping_sub((OP_get_loc0 as i32))) % ((4 as i32))); idx = assigned; assigned };
vm_block = 31; continue;
}
// C line 32363
33 => {
vm_block = 8; continue;
}
// C line 32360
34 => {
let _ = quickjs_debug_write(&quickjs_format_print(c",%u".as_ptr(), &[QuickJSPrintArg::Int(crate::cutils_header::get_u8(((tab).offset(((pos) as isize))).offset((((8 as i32)) as isize))) as u64)]));
vm_block = 33; continue;
}
// C line 32362
35 => {
let _ = quickjs_debug_write(&quickjs_format_print(c",%u".as_ptr(), &[QuickJSPrintArg::Int(crate::cutils_header::get_u16(((tab).offset(((pos) as isize))).offset((((8 as i32)) as isize))) as u64)]));
vm_block = 33; continue;
}
// C line 32359
36 => {
vm_block = if (((((((*(oi)).fmt) as i32)) == ((OP_FMT_atom_label_u8 as i32))) as i32)) != 0 { 34 } else { 35 }; continue;
}
// C line 32358
37 => {
let _ = quickjs_debug_write(&quickjs_format_print(c",%u".as_ptr(), &[QuickJSPrintArg::Int(((addr).wrapping_add(pos)).wrapping_add((4 as i32)) as u64)]));
vm_block = 36; continue;
}
// C line 32357
38 => {
vm_block = if ((((pass) == ((3 as i32))) as i32)) != 0 { 37 } else { 36 }; continue;
}
// C line 32356
39 => {
let _ = quickjs_debug_write(&quickjs_format_print(c",%u:%u".as_ptr(), &[QuickJSPrintArg::Int(addr as u64), QuickJSPrintArg::Int((*(label_slots).offset((addr) as isize)).pos2 as u64)]));
vm_block = 38; continue;
}
// C line 32355
40 => {
vm_block = if ((((pass) == ((2 as i32))) as i32)) != 0 { 39 } else { 38 }; continue;
}
// C line 32354
41 => {
let _ = quickjs_debug_write(&quickjs_format_print(c",%u:%u".as_ptr(), &[QuickJSPrintArg::Int(addr as u64), QuickJSPrintArg::Int((*(label_slots).offset((addr) as isize)).pos as u64)]));
vm_block = 40; continue;
}
// C line 32353
42 => {
vm_block = if ((((pass) == ((1 as i32))) as i32)) != 0 { 41 } else { 40 }; continue;
}
// C line 32352
43 => {
let _ = { let assigned = ((crate::cutils_header::get_u32(((tab).offset(((pos) as isize))).offset((((4 as i32)) as isize)))) as i32); addr = assigned; assigned };
vm_block = 42; continue;
}
// C line 32351
44 => {
let _ = print_atom(ctx, crate::cutils_header::get_u32((tab).offset(((pos) as isize))));
vm_block = 43; continue;
}
// C line 32350
45 => {
let _ = quickjs_debug_write(&quickjs_format_print(c" ".as_ptr(), &[]));
vm_block = 44; continue;
}
// C line 32347
46 => {
vm_block = 8; continue;
}
// C line 32346
47 => {
let _ = quickjs_debug_write(&quickjs_format_print(c",%d".as_ptr(), &[QuickJSPrintArg::Int(crate::cutils_header::get_u16(((tab).offset(((pos) as isize))).offset((((4 as i32)) as isize))) as u64)]));
vm_block = 46; continue;
}
// C line 32345
48 => {
let _ = print_atom(ctx, crate::cutils_header::get_u32((tab).offset(((pos) as isize))));
vm_block = 47; continue;
}
// C line 32344
49 => {
let _ = quickjs_debug_write(&quickjs_format_print(c" ".as_ptr(), &[]));
vm_block = 48; continue;
}
// C line 32342
50 => {
vm_block = 8; continue;
}
// C line 32341
51 => {
let _ = quickjs_debug_write(&quickjs_format_print(c",%d".as_ptr(), &[QuickJSPrintArg::Int(crate::cutils_header::get_u8(((tab).offset(((pos) as isize))).offset((((4 as i32)) as isize))) as u64)]));
vm_block = 50; continue;
}
// C line 32340
52 => {
let _ = print_atom(ctx, crate::cutils_header::get_u32((tab).offset(((pos) as isize))));
vm_block = 51; continue;
}
// C line 32339
53 => {
let _ = quickjs_debug_write(&quickjs_format_print(c" ".as_ptr(), &[]));
vm_block = 52; continue;
}
// C line 32337
54 => {
vm_block = 8; continue;
}
// C line 32336
55 => {
let _ = print_atom(ctx, crate::cutils_header::get_u32((tab).offset(((pos) as isize))));
vm_block = 54; continue;
}
// C line 32335
56 => {
let _ = quickjs_debug_write(&quickjs_format_print(c" ".as_ptr(), &[]));
vm_block = 55; continue;
}
// C line 32333
57 => {
vm_block = 8; continue;
}
// C line 32331
58 => {
let _ = JS_PrintValue(ctx, Some(js_dump_value_write as JSPrintValueWrite), ((core::ptr::null_mut::<c_void>()) as *mut c_void), *(cpool).offset((idx) as isize), core::ptr::null_mut::<JSPrintValueOptions>());
vm_block = 57; continue;
}
// C line 32330
59 => {
vm_block = if ((((((idx) as u32)) < (cpool_count)) as i32)) != 0 { 58 } else { 57 }; continue;
}
// C line 32329 labels: has_pool_idx
60 => {
let _ = quickjs_debug_write(&quickjs_format_print(c" %u: ".as_ptr(), &[QuickJSPrintArg::Int(idx as u64)]));
vm_block = 59; continue;
}
// C line 32327
61 => {
vm_block = 60; continue;
}
// C line 32326
62 => {
let _ = { let assigned = ((crate::cutils_header::get_u32((tab).offset(((pos) as isize)))) as i32); idx = assigned; assigned };
vm_block = 61; continue;
}
// C line 32323
63 => {
vm_block = 60; continue;
}
// C line 32322
64 => {
let _ = { let assigned = ((crate::cutils_header::get_u8((tab).offset(((pos) as isize)))) as i32); idx = assigned; assigned };
vm_block = 63; continue;
}
// C line 32319
65 => {
vm_block = 8; continue;
}
// C line 32318
66 => {
let _ = quickjs_debug_write(&quickjs_format_print(c",%u".as_ptr(), &[QuickJSPrintArg::Int(crate::cutils_header::get_u16(((tab).offset(((pos) as isize))).offset((((4 as i32)) as isize))) as u64)]));
vm_block = 65; continue;
}
// C line 32317
67 => {
let _ = quickjs_debug_write(&quickjs_format_print(c" %u".as_ptr(), &[QuickJSPrintArg::Int((addr).wrapping_add(pos) as u64)]));
vm_block = 66; continue;
}
// C line 32316
68 => {
vm_block = if ((((pass) == ((3 as i32))) as i32)) != 0 { 67 } else { 66 }; continue;
}
// C line 32315
69 => {
let _ = quickjs_debug_write(&quickjs_format_print(c" %u:%u".as_ptr(), &[QuickJSPrintArg::Int(addr as u64), QuickJSPrintArg::Int((*(label_slots).offset((addr) as isize)).pos2 as u64)]));
vm_block = 68; continue;
}
// C line 32314
70 => {
vm_block = if ((((pass) == ((2 as i32))) as i32)) != 0 { 69 } else { 68 }; continue;
}
// C line 32313
71 => {
let _ = quickjs_debug_write(&quickjs_format_print(c" %u:%u".as_ptr(), &[QuickJSPrintArg::Int(addr as u64), QuickJSPrintArg::Int((*(label_slots).offset((addr) as isize)).pos as u64)]));
vm_block = 70; continue;
}
// C line 32312
72 => {
vm_block = if ((((pass) == ((1 as i32))) as i32)) != 0 { 71 } else { 70 }; continue;
}
// C line 32311
73 => {
let _ = { let assigned = ((crate::cutils_header::get_u32((tab).offset(((pos) as isize)))) as i32); addr = assigned; assigned };
vm_block = 72; continue;
}
// C line 32309
74 => {
vm_block = 8; continue;
}
// C line 32308
75 => {
let _ = quickjs_debug_write(&quickjs_format_print(c" %u".as_ptr(), &[QuickJSPrintArg::Int((addr).wrapping_add(pos) as u64)]));
vm_block = 74; continue;
}
// C line 32307
76 => {
vm_block = if ((((pass) == ((3 as i32))) as i32)) != 0 { 75 } else { 74 }; continue;
}
// C line 32306
77 => {
let _ = quickjs_debug_write(&quickjs_format_print(c" %u:%u".as_ptr(), &[QuickJSPrintArg::Int(addr as u64), QuickJSPrintArg::Int((*(label_slots).offset((addr) as isize)).pos2 as u64)]));
vm_block = 76; continue;
}
// C line 32305
78 => {
vm_block = if ((((pass) == ((2 as i32))) as i32)) != 0 { 77 } else { 76 }; continue;
}
// C line 32304
79 => {
let _ = quickjs_debug_write(&quickjs_format_print(c" %u:%u".as_ptr(), &[QuickJSPrintArg::Int(addr as u64), QuickJSPrintArg::Int((*(label_slots).offset((addr) as isize)).pos as u64)]));
vm_block = 78; continue;
}
// C line 32303 labels: has_addr1
80 => {
vm_block = if ((((pass) == ((1 as i32))) as i32)) != 0 { 79 } else { 78 }; continue;
}
// C line 32301
81 => {
vm_block = 80; continue;
}
// C line 32300
82 => {
let _ = { let assigned = ((crate::cutils_header::get_u32((tab).offset(((pos) as isize)))) as i32); addr = assigned; assigned };
vm_block = 81; continue;
}
// C line 32297
83 => {
vm_block = 80; continue;
}
// C line 32296
84 => {
let _ = { let assigned = crate::cutils_header::get_i16((tab).offset(((pos) as isize))); addr = assigned; assigned };
vm_block = 83; continue;
}
// C line 32294
85 => {
vm_block = 80; continue;
}
// C line 32293
86 => {
let _ = { let assigned = crate::cutils_header::get_i8((tab).offset(((pos) as isize))); addr = assigned; assigned };
vm_block = 85; continue;
}
// C line 32290
87 => {
vm_block = 8; continue;
}
// C line 32289
88 => {
let _ = quickjs_debug_write(&quickjs_format_print(c" %u".as_ptr(), &[QuickJSPrintArg::Int(crate::cutils_header::get_u32((tab).offset(((pos) as isize))) as u64)]));
vm_block = 87; continue;
}
// C line 32287
89 => {
vm_block = 8; continue;
}
// C line 32286
90 => {
let _ = quickjs_debug_write(&quickjs_format_print(c" %d".as_ptr(), &[QuickJSPrintArg::Int(crate::cutils_header::get_i32((tab).offset(((pos) as isize))) as u64)]));
vm_block = 89; continue;
}
// C line 32284
91 => {
vm_block = 8; continue;
}
// C line 32283
92 => {
let _ = quickjs_debug_write(&quickjs_format_print(c" %d".as_ptr(), &[QuickJSPrintArg::Int(crate::cutils_header::get_i16((tab).offset(((pos) as isize))) as u64)]));
vm_block = 91; continue;
}
// C line 32281
93 => {
vm_block = 8; continue;
}
// C line 32280
94 => {
let _ = quickjs_debug_write(&quickjs_format_print(c" %u,%u".as_ptr(), &[QuickJSPrintArg::Int(crate::cutils_header::get_u16((tab).offset(((pos) as isize))) as u64), QuickJSPrintArg::Int(crate::cutils_header::get_u16(((tab).offset(((pos) as isize))).offset((((2 as i32)) as isize))) as u64)]));
vm_block = 93; continue;
}
// C line 32278
95 => {
vm_block = 8; continue;
}
// C line 32277
96 => {
let _ = quickjs_debug_write(&quickjs_format_print(c" %u".as_ptr(), &[QuickJSPrintArg::Int(crate::cutils_header::get_u16((tab).offset(((pos) as isize))) as u64)]));
vm_block = 95; continue;
}
// C line 32274
97 => {
vm_block = 8; continue;
}
// C line 32273
98 => {
let _ = quickjs_debug_write(&quickjs_format_print(c" %d".as_ptr(), &[QuickJSPrintArg::Int(crate::cutils_header::get_i8((tab).offset(((pos) as isize))) as u64)]));
vm_block = 97; continue;
}
// C line 32271
99 => {
vm_block = 8; continue;
}
// C line 32270
100 => {
let _ = quickjs_debug_write(&quickjs_format_print(c" %u".as_ptr(), &[QuickJSPrintArg::Int(crate::cutils_header::get_u8((tab).offset(((pos) as isize))) as u64)]));
vm_block = 99; continue;
}
// C line 32268
101 => {
vm_block = 8; continue;
}
// C line 32267
102 => {
let _ = quickjs_debug_write(&quickjs_format_print(c" %d".as_ptr(), &[QuickJSPrintArg::Int((op).wrapping_sub((OP_call0 as i32)) as u64)]));
vm_block = 101; continue;
}
// C line 32265
103 => {
vm_block = 8; continue;
}
// C line 32264
104 => {
let _ = quickjs_debug_write(&quickjs_format_print(c" %d".as_ptr(), &[QuickJSPrintArg::Int((op).wrapping_sub((OP_push_0 as i32)) as u64)]));
vm_block = 103; continue;
}
// C line 32262
105 => {
vm_block = match (((*(oi)).fmt) as i32) { x if x == (OP_FMT_var_ref as i32) => 14, x if x == (OP_FMT_none_var_ref as i32) => 16, x if x == (OP_FMT_arg as i32) => 21, x if x == (OP_FMT_none_arg as i32) => 23, x if x == (OP_FMT_loc as i32) => 28, x if x == (OP_FMT_loc8 as i32) => 30, x if x == (OP_FMT_none_loc as i32) => 32, x if x == (OP_FMT_atom_label_u16 as i32) => 45, x if x == (OP_FMT_atom_label_u8 as i32) => 45, x if x == (OP_FMT_atom_u16 as i32) => 49, x if x == (OP_FMT_atom_u8 as i32) => 53, x if x == (OP_FMT_atom as i32) => 56, x if x == (OP_FMT_const as i32) => 62, x if x == (OP_FMT_const8 as i32) => 64, x if x == (OP_FMT_label_u16 as i32) => 73, x if x == (OP_FMT_label as i32) => 82, x if x == (OP_FMT_label16 as i32) => 84, x if x == (OP_FMT_label8 as i32) => 86, x if x == (OP_FMT_u32 as i32) => 88, x if x == (OP_FMT_i32 as i32) => 90, x if x == (OP_FMT_i16 as i32) => 92, x if x == (OP_FMT_npop_u16 as i32) => 94, x if x == (OP_FMT_npop as i32) => 96, x if x == (OP_FMT_u16 as i32) => 96, x if x == (OP_FMT_i8 as i32) => 98, x if x == (OP_FMT_u8 as i32) => 100, x if x == (OP_FMT_npopx as i32) => 102, x if x == (OP_FMT_none_int as i32) => 104, _ => 9, }; continue;
}
// C line 32261
106 => {
let _ = { let old = pos; pos = (pos).wrapping_add(1); old };
vm_block = 105; continue;
}
// C line 32260
107 => {
let _ = quickjs_debug_write(&quickjs_format_print(c"%s".as_ptr(), &[QuickJSPrintArg::Str(debug_opcode_name(op as usize, use_short_opcodes != 0) as *const c_char)]));
vm_block = 106; continue;
}
// C line 32256
108 => {
let _ = quickjs_debug_write(&quickjs_format_print(c"%5d:  ".as_ptr(), &[QuickJSPrintArg::Int(pos as u64)]));
vm_block = 107; continue;
}
// C line 32258
109 => {
let _ = quickjs_debug_write(&quickjs_format_print(c"        ".as_ptr(), &[]));
vm_block = 107; continue;
}
// C line 32255
110 => {
vm_block = if (dump_pc) != 0 { 108 } else { 109 }; continue;
}
// C line 32251
111 => {
let _ = { let assigned = (1 as i32); dump_pc = assigned; assigned };
vm_block = 110; continue;
}
// C line 32247
112 => {
let _ = quickjs_debug_write(&quickjs_format_print(c"%*s".as_ptr(), &[QuickJSPrintArg::Int(((x0).wrapping_add((20 as i32))).wrapping_sub(x) as u64), QuickJSPrintArg::Str(c"".as_ptr() as *const c_char)]));
vm_block = 111; continue;
}
// C line 32241
113 => {
vm_block = if ((((i) < (size)) as i32)) != 0 { 117 } else { 112 }; continue;
}
// C line 32241
114 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 113; continue;
}
// C line 32245
115 => {
let _ = { x = ((((x) as i32)).wrapping_add(quickjs_debug_write(&quickjs_format_print(c" %02X".as_ptr(), &[QuickJSPrintArg::Int(((*(tab).offset(((pos).wrapping_add(i)) as isize)) as i32) as u64)])))) as i32; x };
vm_block = 114; continue;
}
// C line 32243
116 => {
let _ = quickjs_debug_write(&quickjs_format_print(c"\n%*s".as_ptr(), &[QuickJSPrintArg::Int({ let assigned = x0; x = assigned; assigned } as u64), QuickJSPrintArg::Str(c"".as_ptr() as *const c_char)]));
vm_block = 115; continue;
}
// C line 32242
117 => {
vm_block = if ((((i) == ((6 as i32))) as i32)) != 0 { 116 } else { 115 }; continue;
}
// C line 32241
118 => {
let _ = { let assigned = (0 as i32); i = assigned; assigned };
vm_block = 113; continue;
}
// C line 32240
119 => {
let _ = { let assigned = { let assigned = quickjs_debug_write(&quickjs_format_print(c"%5d ".as_ptr(), &[QuickJSPrintArg::Int(pos as u64)])); x0 = assigned; assigned }; x = assigned; assigned };
vm_block = 118; continue;
}
// C line 32235
120 => {
vm_block = 5; continue;
}
// C line 32234
121 => {
let _ = quickjs_debug_write(&quickjs_format_print(c"truncated opcode (0x%02x)\n".as_ptr(), &[QuickJSPrintArg::Int(op as u64)]));
vm_block = 120; continue;
}
// C line 32233
122 => {
vm_block = if (((((pos).wrapping_add(size)) > (len)) as i32)) != 0 { 121 } else { 119 }; continue;
}
// C line 32232
123 => {
let _ = { let assigned = (((*(oi)).size) as i32); size = assigned; assigned };
vm_block = 122; continue;
}
// C line 32229
124 => {
let _ = { let assigned = core::ptr::addr_of!(*((opcode_info).as_ptr()).offset((if ((((op) >= ((OP_TEMP_START as i32))) as i32)) != 0 { (op).wrapping_add(((OP_TEMP_END as i32)).wrapping_sub((OP_TEMP_START as i32))) } else { op }) as isize)); oi = assigned; assigned };
vm_block = 123; continue;
}
// C line 32231
125 => {
let _ = { let assigned = core::ptr::addr_of!(*((opcode_info).as_ptr()).offset((op) as isize)); oi = assigned; assigned };
vm_block = 123; continue;
}
// C line 32228
126 => {
vm_block = if (use_short_opcodes) != 0 { 124 } else { 125 }; continue;
}
// C line 32226
127 => {
vm_block = 6; continue;
}
// C line 32225
128 => {
let _ = { let old = pos; pos = (pos).wrapping_add(1); old };
vm_block = 127; continue;
}
// C line 32224
129 => {
let _ = quickjs_debug_write(&quickjs_format_print(c"invalid opcode (0x%02x)\n".as_ptr(), &[QuickJSPrintArg::Int(op as u64)]));
vm_block = 128; continue;
}
// C line 32223
130 => {
vm_block = if ((((op) >= ((OP_COUNT as i32))) as i32)) != 0 { 129 } else { 126 }; continue;
}
// C line 32222
131 => {
let _ = { let assigned = (0 as i32); in_source = assigned; assigned };
vm_block = 130; continue;
}
// C line 32221
132 => {
let _ = quickjs_debug_write(&quickjs_format_print(c"\n".as_ptr(), &[]));
vm_block = 131; continue;
}
// C line 32220
133 => {
vm_block = if (in_source) != 0 { 132 } else { 131 }; continue;
}
// C line 32216
134 => {
let _ = { let assigned = line1; line = assigned; assigned };
vm_block = 133; continue;
}
// C line 32215
135 => {
let _ = print_lines(source, line, line1);
vm_block = 134; continue;
}
// C line 32214
136 => {
let _ = { let assigned = (1 as i32); in_source = assigned; assigned };
vm_block = 135; continue;
}
// C line 32213
137 => {
let _ = quickjs_debug_write(&quickjs_format_print(c"\n".as_ptr(), &[]));
vm_block = 136; continue;
}
// C line 32212
138 => {
vm_block = if ((!((in_source) != 0) as i32)) != 0 { 137 } else { 136 }; continue;
}
// C line 32211
139 => {
vm_block = if ((((line1) > (line)) as i32)) != 0 { 138 } else { 133 }; continue;
}
// C line 32206
140 => {
let _ = { let assigned = ((find_line_num(ctx, b, ((pos) as u32), core::ptr::addr_of_mut!(col_num_1))).wrapping_sub(line_num)).wrapping_add((1 as i32)); line1 = assigned; assigned };
vm_block = 139; continue;
}
// C line 32209
141 => {
let _ = { let assigned = ((((crate::cutils_header::get_u32(((tab).offset(((pos) as isize))).offset((((1 as i32)) as isize)))).wrapping_sub(((line_num) as u32))).wrapping_add((((1 as i32)) as u32))) as i32); line1 = assigned; assigned };
vm_block = 139; continue;
}
// C line 32207
142 => {
vm_block = if ((((op) == ((OP_line_num as i32))) as i32)) != 0 { 141 } else { 139 }; continue;
}
// C line 32205
143 => {
vm_block = if !(b).is_null() { 140 } else { 142 }; continue;
}
// C line 32203
144 => {
vm_block = if ((((!(source).is_null()) && (!(b).is_null())) as i32)) != 0 { 143 } else { 133 }; continue;
}
// C line 32202
145 => {
let _ = { let assigned = ((*(tab).offset((pos) as isize)) as i32); op = assigned; assigned };
vm_block = 144; continue;
}
// C line 32200
146 => {
let _ = { let assigned = (0 as i32); pos = assigned; assigned };
vm_block = 6; continue;
}
// C line 32199
147 => {
let _ = { let assigned = { let assigned = (1 as i32); line = assigned; assigned }; line1 = assigned; assigned };
vm_block = 146; continue;
}
// C line 32197
148 => {
let _ = { let assigned = (1 as i32); in_source = assigned; assigned };
vm_block = 147; continue;
}
// C line 32196
149 => {
let _ = print_lines(source, (0 as i32), (1 as i32));
vm_block = 148; continue;
}
// C line 32194
150 => {
vm_block = if !(source).is_null() { 149 } else { 147 }; continue;
}
// C line 32193
151 => {
let _ = { let assigned = (0 as i32); in_source = assigned; assigned };
vm_block = 150; continue;
}
// C line 32152
152 => {
vm_block = if ((((pos) < (len)) as i32)) != 0 { 179 } else { 151 }; continue;
}
// C line 32152
153 => {
let _ = { let assigned = pos_next; pos = assigned; assigned };
vm_block = 152; continue;
}
// C line 32189
154 => {
vm_block = 153; continue;
}
// C line 32188
155 => {
let _ = { *(bits).offset((addr) as isize) = (((((*(bits).offset((addr) as isize)) as i32)) | ((1 as i32)))) as u8; *(bits).offset((addr) as isize) };
vm_block = 154; continue;
}
// C line 32187
156 => {
vm_block = if ((((((((addr) >= ((0 as i32))) as i32)) != 0) && (((((addr) < (len)) as i32)) != 0)) as i32)) != 0 { 155 } else { 154 }; continue;
}
// C line 32186
157 => {
let _ = { addr = ((((addr) as i32)).wrapping_add(pos)) as i32; addr };
vm_block = 156; continue;
}
// C line 32185
158 => {
vm_block = if ((((pass) == ((3 as i32))) as i32)) != 0 { 157 } else { 156 }; continue;
}
// C line 32184
159 => {
let _ = { let assigned = (*(label_slots).offset((addr) as isize)).pos2; addr = assigned; assigned };
vm_block = 158; continue;
}
// C line 32183
160 => {
vm_block = if ((((pass) == ((2 as i32))) as i32)) != 0 { 159 } else { 158 }; continue;
}
// C line 32182
161 => {
let _ = { let assigned = (*(label_slots).offset((addr) as isize)).pos; addr = assigned; assigned };
vm_block = 160; continue;
}
// C line 32181 labels: has_addr
162 => {
vm_block = if ((((pass) == ((1 as i32))) as i32)) != 0 { 161 } else { 160 }; continue;
}
// C line 32179
163 => {
vm_block = 162; continue;
}
// C line 32178
164 => {
let _ = { let assigned = ((crate::cutils_header::get_u32((tab).offset(((pos) as isize)))) as i32); addr = assigned; assigned };
vm_block = 163; continue;
}
// C line 32177
165 => {
let _ = { let old = pos; pos = (pos).wrapping_add(1); old };
vm_block = 164; continue;
}
// C line 32173
166 => {
let _ = { pos = ((((pos) as i32)).wrapping_add((4 as i32))) as i32; pos };
vm_block = 165; continue;
}
// C line 32169
167 => {
vm_block = 162; continue;
}
// C line 32168
168 => {
let _ = { let assigned = ((((crate::cutils_header::get_u16((tab).offset(((pos) as isize)))) as i16)) as i32); addr = assigned; assigned };
vm_block = 167; continue;
}
// C line 32167
169 => {
let _ = { let old = pos; pos = (pos).wrapping_add(1); old };
vm_block = 168; continue;
}
// C line 32165
170 => {
vm_block = 162; continue;
}
// C line 32164
171 => {
let _ = { let assigned = ((((*(tab).offset((pos) as isize)) as i8)) as i32); addr = assigned; assigned };
vm_block = 170; continue;
}
// C line 32163
172 => {
let _ = { let old = pos; pos = (pos).wrapping_add(1); old };
vm_block = 171; continue;
}
// C line 32160
173 => {
vm_block = match (((*(oi)).fmt) as i32) { x if x == (OP_FMT_label_u16 as i32) => 165, x if x == (OP_FMT_label as i32) => 165, x if x == (OP_FMT_atom_label_u16 as i32) => 166, x if x == (OP_FMT_atom_label_u8 as i32) => 166, x if x == (OP_FMT_label16 as i32) => 169, x if x == (OP_FMT_label8 as i32) => 172, _ => 153, }; continue;
}
// C line 32159
174 => {
vm_block = if ((((op) < ((OP_COUNT as i32))) as i32)) != 0 { 173 } else { 153 }; continue;
}
// C line 32158
175 => {
let _ = { let assigned = (pos).wrapping_add((((*(oi)).size) as i32)); pos_next = assigned; assigned };
vm_block = 174; continue;
}
// C line 32155
176 => {
let _ = { let assigned = core::ptr::addr_of!(*((opcode_info).as_ptr()).offset((if ((((op) >= ((OP_TEMP_START as i32))) as i32)) != 0 { (op).wrapping_add(((OP_TEMP_END as i32)).wrapping_sub((OP_TEMP_START as i32))) } else { op }) as isize)); oi = assigned; assigned };
vm_block = 175; continue;
}
// C line 32157
177 => {
let _ = { let assigned = core::ptr::addr_of!(*((opcode_info).as_ptr()).offset((op) as isize)); oi = assigned; assigned };
vm_block = 175; continue;
}
// C line 32154
178 => {
vm_block = if (use_short_opcodes) != 0 { 176 } else { 177 }; continue;
}
// C line 32153
179 => {
let _ = { let assigned = ((*(tab).offset((pos) as isize)) as i32); op = assigned; assigned };
vm_block = 178; continue;
}
// C line 32152
180 => {
let _ = { let assigned = (0 as i32); pos = assigned; assigned };
vm_block = 152; continue;
}
// C line 32148
181 => {
let _ = { let assigned = find_line_num(ctx, b, ((((1 as i32)).wrapping_neg()) as u32), core::ptr::addr_of_mut!(col_num)); line_num = assigned; assigned };
vm_block = 180; continue;
}
// C line 32146
182 => {
vm_block = if !(b).is_null() { 181 } else { 180 }; continue;
}
// C line 32144
183 => {
use_short_opcodes = (((b) != (core::ptr::null_mut::<JSFunctionBytecode>())) as i32);
vm_block = 182; continue;
}
// C line 32143
184 => {
bits = ((js_mallocz(ctx, (((len) as usize)).wrapping_mul((size_of::<u8>() as usize)))) as *mut u8);
vm_block = 183; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:32131. Bellard/Gordon MIT.
#[cfg(not(feature = "short-opcodes"))]
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn dump_byte_code(mut ctx: *mut JSContext, mut pass: i32, mut tab: *const u8, mut len: i32, mut vardefs: *const JSBytecodeVarDef, mut args: *const JSVarDef, mut arg_count: i32, mut vars: *const JSVarDef, mut var_count: i32, mut closure_var: *const JSClosureVar, mut closure_var_count: i32, mut cpool: *const JSValue, mut cpool_count: u32, mut source: *const c_char, mut label_slots: *const LabelSlot, mut b: *mut JSFunctionBytecode) -> () {
let mut vm_local_storage = Vec::<u64>::new();
let mut oi: *const JSOpCode = core::mem::zeroed();
let mut pos: i32 = core::mem::zeroed();
let mut pos_next: i32 = core::mem::zeroed();
let mut op: i32 = core::mem::zeroed();
let mut size: i32 = core::mem::zeroed();
let mut idx: i32 = core::mem::zeroed();
let mut addr: i32 = core::mem::zeroed();
let mut line: i32 = core::mem::zeroed();
let mut line1: i32 = core::mem::zeroed();
let mut in_source: i32 = core::mem::zeroed();
let mut line_num: i32 = core::mem::zeroed();
let mut bits: *mut u8 = core::mem::zeroed();
let mut use_short_opcodes: i32 = core::mem::zeroed();
let mut dump_pc: i32 = core::mem::zeroed();
let mut col_num: i32 = core::mem::zeroed();
let mut col_num_1: i32 = core::mem::zeroed();
let mut i: i32 = core::mem::zeroed();
let mut x: i32 = core::mem::zeroed();
let mut x0: i32 = core::mem::zeroed();
let mut vm_block: usize = 162;
loop { match vm_block {
// C line ?
0 => {
return ();
}
// C line 32421
1 => {
let _ = js_free(ctx, ((bits) as *mut c_void));
vm_block = 0; continue;
}
// C line 32419
2 => {
let _ = print_lines(source, line, (2147483647 as i32));
vm_block = 1; continue;
}
// C line 32418
3 => {
let _ = quickjs_debug_write(&quickjs_format_print(c"\n".as_ptr(), &[]));
vm_block = 2; continue;
}
// C line 32417
4 => {
vm_block = if ((!((in_source) != 0) as i32)) != 0 { 3 } else { 2 }; continue;
}
// C line 32416
5 => {
vm_block = if !(source).is_null() { 4 } else { 1 }; continue;
}
// C line 32201
6 => {
vm_block = if ((((pos) < (len)) as i32)) != 0 { 129 } else { 5 }; continue;
}
// C line 32414
7 => {
let _ = { pos = ((((pos) as i32)).wrapping_add(((((*(oi)).size) as i32)).wrapping_sub((1 as i32)))) as i32; pos };
vm_block = 6; continue;
}
// C line 32413
8 => {
let _ = quickjs_debug_write(&quickjs_format_print(c"\n".as_ptr(), &[]));
vm_block = 7; continue;
}
// C line 32408
9 => {
vm_block = 8; continue;
}
// C line 32409
10 => {
vm_block = 8; continue;
}
// C line 32407
11 => {
let _ = print_atom(ctx, (*(closure_var).offset((idx) as isize)).var_name);
vm_block = 10; continue;
}
// C line 32406
12 => {
vm_block = if ((((idx) < (closure_var_count)) as i32)) != 0 { 11 } else { 10 }; continue;
}
// C line 32398 labels: has_var_ref
13 => {
let _ = quickjs_debug_write(&quickjs_format_print(c" %d: ".as_ptr(), &[QuickJSPrintArg::Int(idx as u64)]));
vm_block = 12; continue;
}
// C line 32403
14 => {
let _ = { let assigned = ((crate::cutils_header::get_u16((tab).offset(((pos) as isize)))) as i32); idx = assigned; assigned };
vm_block = 13; continue;
}
// C line 32396
15 => {
vm_block = 8; continue;
}
// C line 32394
16 => {
let _ = print_atom(ctx, if !(args).is_null() { (*(args).offset((idx) as isize)).var_name } else { (*(vardefs).offset((idx) as isize)).var_name });
vm_block = 15; continue;
}
// C line 32393
17 => {
vm_block = if ((((idx) < (arg_count)) as i32)) != 0 { 16 } else { 15 }; continue;
}
// C line 32386 labels: has_arg
18 => {
let _ = quickjs_debug_write(&quickjs_format_print(c" %d: ".as_ptr(), &[QuickJSPrintArg::Int(idx as u64)]));
vm_block = 17; continue;
}
// C line 32390
19 => {
let _ = { let assigned = ((crate::cutils_header::get_u16((tab).offset(((pos) as isize)))) as i32); idx = assigned; assigned };
vm_block = 18; continue;
}
// C line 32383
20 => {
vm_block = 8; continue;
}
// C line 32381
21 => {
let _ = print_atom(ctx, if !(vars).is_null() { (*(vars).offset((idx) as isize)).var_name } else { (*(vardefs).offset(((arg_count).wrapping_add(idx)) as isize)).var_name });
vm_block = 20; continue;
}
// C line 32380
22 => {
vm_block = if ((((idx) < (var_count)) as i32)) != 0 { 21 } else { 20 }; continue;
}
// C line 32375 labels: has_loc
23 => {
let _ = quickjs_debug_write(&quickjs_format_print(c" %d: ".as_ptr(), &[QuickJSPrintArg::Int(idx as u64)]));
vm_block = 22; continue;
}
// C line 32377
24 => {
let _ = { let assigned = ((crate::cutils_header::get_u16((tab).offset(((pos) as isize)))) as i32); idx = assigned; assigned };
vm_block = 23; continue;
}
// C line 32375
25 => {
vm_block = 23; continue;
}
// C line 32374
26 => {
let _ = { let assigned = ((crate::cutils_header::get_u8((tab).offset(((pos) as isize)))) as i32); idx = assigned; assigned };
vm_block = 25; continue;
}
// C line 32367
27 => {
vm_block = 8; continue;
}
// C line 32364
28 => {
let _ = quickjs_debug_write(&quickjs_format_print(c",%u".as_ptr(), &[QuickJSPrintArg::Int(crate::cutils_header::get_u8(((tab).offset(((pos) as isize))).offset((((8 as i32)) as isize))) as u64)]));
vm_block = 27; continue;
}
// C line 32366
29 => {
let _ = quickjs_debug_write(&quickjs_format_print(c",%u".as_ptr(), &[QuickJSPrintArg::Int(crate::cutils_header::get_u16(((tab).offset(((pos) as isize))).offset((((8 as i32)) as isize))) as u64)]));
vm_block = 27; continue;
}
// C line 32363
30 => {
vm_block = if (((((((*(oi)).fmt) as i32)) == ((OP_FMT_atom_label_u8 as i32))) as i32)) != 0 { 28 } else { 29 }; continue;
}
// C line 32362
31 => {
let _ = quickjs_debug_write(&quickjs_format_print(c",%u".as_ptr(), &[QuickJSPrintArg::Int(((addr).wrapping_add(pos)).wrapping_add((4 as i32)) as u64)]));
vm_block = 30; continue;
}
// C line 32361
32 => {
vm_block = if ((((pass) == ((3 as i32))) as i32)) != 0 { 31 } else { 30 }; continue;
}
// C line 32360
33 => {
let _ = quickjs_debug_write(&quickjs_format_print(c",%u:%u".as_ptr(), &[QuickJSPrintArg::Int(addr as u64), QuickJSPrintArg::Int((*(label_slots).offset((addr) as isize)).pos2 as u64)]));
vm_block = 32; continue;
}
// C line 32359
34 => {
vm_block = if ((((pass) == ((2 as i32))) as i32)) != 0 { 33 } else { 32 }; continue;
}
// C line 32358
35 => {
let _ = quickjs_debug_write(&quickjs_format_print(c",%u:%u".as_ptr(), &[QuickJSPrintArg::Int(addr as u64), QuickJSPrintArg::Int((*(label_slots).offset((addr) as isize)).pos as u64)]));
vm_block = 34; continue;
}
// C line 32357
36 => {
vm_block = if ((((pass) == ((1 as i32))) as i32)) != 0 { 35 } else { 34 }; continue;
}
// C line 32356
37 => {
let _ = { let assigned = ((crate::cutils_header::get_u32(((tab).offset(((pos) as isize))).offset((((4 as i32)) as isize)))) as i32); addr = assigned; assigned };
vm_block = 36; continue;
}
// C line 32355
38 => {
let _ = print_atom(ctx, crate::cutils_header::get_u32((tab).offset(((pos) as isize))));
vm_block = 37; continue;
}
// C line 32354
39 => {
let _ = quickjs_debug_write(&quickjs_format_print(c" ".as_ptr(), &[]));
vm_block = 38; continue;
}
// C line 32351
40 => {
vm_block = 8; continue;
}
// C line 32350
41 => {
let _ = quickjs_debug_write(&quickjs_format_print(c",%d".as_ptr(), &[QuickJSPrintArg::Int(crate::cutils_header::get_u16(((tab).offset(((pos) as isize))).offset((((4 as i32)) as isize))) as u64)]));
vm_block = 40; continue;
}
// C line 32349
42 => {
let _ = print_atom(ctx, crate::cutils_header::get_u32((tab).offset(((pos) as isize))));
vm_block = 41; continue;
}
// C line 32348
43 => {
let _ = quickjs_debug_write(&quickjs_format_print(c" ".as_ptr(), &[]));
vm_block = 42; continue;
}
// C line 32346
44 => {
vm_block = 8; continue;
}
// C line 32345
45 => {
let _ = quickjs_debug_write(&quickjs_format_print(c",%d".as_ptr(), &[QuickJSPrintArg::Int(crate::cutils_header::get_u8(((tab).offset(((pos) as isize))).offset((((4 as i32)) as isize))) as u64)]));
vm_block = 44; continue;
}
// C line 32344
46 => {
let _ = print_atom(ctx, crate::cutils_header::get_u32((tab).offset(((pos) as isize))));
vm_block = 45; continue;
}
// C line 32343
47 => {
let _ = quickjs_debug_write(&quickjs_format_print(c" ".as_ptr(), &[]));
vm_block = 46; continue;
}
// C line 32341
48 => {
vm_block = 8; continue;
}
// C line 32340
49 => {
let _ = print_atom(ctx, crate::cutils_header::get_u32((tab).offset(((pos) as isize))));
vm_block = 48; continue;
}
// C line 32339
50 => {
let _ = quickjs_debug_write(&quickjs_format_print(c" ".as_ptr(), &[]));
vm_block = 49; continue;
}
// C line 32337
51 => {
vm_block = 8; continue;
}
// C line 32335
52 => {
let _ = JS_PrintValue(ctx, Some(js_dump_value_write as JSPrintValueWrite), ((core::ptr::null_mut::<c_void>()) as *mut c_void), *(cpool).offset((idx) as isize), core::ptr::null_mut::<JSPrintValueOptions>());
vm_block = 51; continue;
}
// C line 32334
53 => {
vm_block = if ((((((idx) as u32)) < (cpool_count)) as i32)) != 0 { 52 } else { 51 }; continue;
}
// C line 32330 labels: has_pool_idx
54 => {
let _ = quickjs_debug_write(&quickjs_format_print(c" %u: ".as_ptr(), &[QuickJSPrintArg::Int(idx as u64)]));
vm_block = 53; continue;
}
// C line 32331
55 => {
vm_block = 54; continue;
}
// C line 32330
56 => {
let _ = { let assigned = ((crate::cutils_header::get_u32((tab).offset(((pos) as isize)))) as i32); idx = assigned; assigned };
vm_block = 55; continue;
}
// C line 32323
57 => {
vm_block = 8; continue;
}
// C line 32322
58 => {
let _ = quickjs_debug_write(&quickjs_format_print(c",%u".as_ptr(), &[QuickJSPrintArg::Int(crate::cutils_header::get_u16(((tab).offset(((pos) as isize))).offset((((4 as i32)) as isize))) as u64)]));
vm_block = 57; continue;
}
// C line 32321
59 => {
let _ = quickjs_debug_write(&quickjs_format_print(c" %u".as_ptr(), &[QuickJSPrintArg::Int((addr).wrapping_add(pos) as u64)]));
vm_block = 58; continue;
}
// C line 32320
60 => {
vm_block = if ((((pass) == ((3 as i32))) as i32)) != 0 { 59 } else { 58 }; continue;
}
// C line 32319
61 => {
let _ = quickjs_debug_write(&quickjs_format_print(c" %u:%u".as_ptr(), &[QuickJSPrintArg::Int(addr as u64), QuickJSPrintArg::Int((*(label_slots).offset((addr) as isize)).pos2 as u64)]));
vm_block = 60; continue;
}
// C line 32318
62 => {
vm_block = if ((((pass) == ((2 as i32))) as i32)) != 0 { 61 } else { 60 }; continue;
}
// C line 32317
63 => {
let _ = quickjs_debug_write(&quickjs_format_print(c" %u:%u".as_ptr(), &[QuickJSPrintArg::Int(addr as u64), QuickJSPrintArg::Int((*(label_slots).offset((addr) as isize)).pos as u64)]));
vm_block = 62; continue;
}
// C line 32316
64 => {
vm_block = if ((((pass) == ((1 as i32))) as i32)) != 0 { 63 } else { 62 }; continue;
}
// C line 32315
65 => {
let _ = { let assigned = ((crate::cutils_header::get_u32((tab).offset(((pos) as isize)))) as i32); addr = assigned; assigned };
vm_block = 64; continue;
}
// C line 32313
66 => {
vm_block = 8; continue;
}
// C line 32312
67 => {
let _ = quickjs_debug_write(&quickjs_format_print(c" %u".as_ptr(), &[QuickJSPrintArg::Int((addr).wrapping_add(pos) as u64)]));
vm_block = 66; continue;
}
// C line 32311
68 => {
vm_block = if ((((pass) == ((3 as i32))) as i32)) != 0 { 67 } else { 66 }; continue;
}
// C line 32310
69 => {
let _ = quickjs_debug_write(&quickjs_format_print(c" %u:%u".as_ptr(), &[QuickJSPrintArg::Int(addr as u64), QuickJSPrintArg::Int((*(label_slots).offset((addr) as isize)).pos2 as u64)]));
vm_block = 68; continue;
}
// C line 32309
70 => {
vm_block = if ((((pass) == ((2 as i32))) as i32)) != 0 { 69 } else { 68 }; continue;
}
// C line 32308
71 => {
let _ = quickjs_debug_write(&quickjs_format_print(c" %u:%u".as_ptr(), &[QuickJSPrintArg::Int(addr as u64), QuickJSPrintArg::Int((*(label_slots).offset((addr) as isize)).pos as u64)]));
vm_block = 70; continue;
}
// C line 32307 labels: has_addr1
72 => {
vm_block = if ((((pass) == ((1 as i32))) as i32)) != 0 { 71 } else { 70 }; continue;
}
// C line 32305
73 => {
vm_block = 72; continue;
}
// C line 32304
74 => {
let _ = { let assigned = ((crate::cutils_header::get_u32((tab).offset(((pos) as isize)))) as i32); addr = assigned; assigned };
vm_block = 73; continue;
}
// C line 32294
75 => {
vm_block = 8; continue;
}
// C line 32293
76 => {
let _ = quickjs_debug_write(&quickjs_format_print(c" %u".as_ptr(), &[QuickJSPrintArg::Int(crate::cutils_header::get_u32((tab).offset(((pos) as isize))) as u64)]));
vm_block = 75; continue;
}
// C line 32291
77 => {
vm_block = 8; continue;
}
// C line 32290
78 => {
let _ = quickjs_debug_write(&quickjs_format_print(c" %d".as_ptr(), &[QuickJSPrintArg::Int(crate::cutils_header::get_i32((tab).offset(((pos) as isize))) as u64)]));
vm_block = 77; continue;
}
// C line 32288
79 => {
vm_block = 8; continue;
}
// C line 32287
80 => {
let _ = quickjs_debug_write(&quickjs_format_print(c" %d".as_ptr(), &[QuickJSPrintArg::Int(crate::cutils_header::get_i16((tab).offset(((pos) as isize))) as u64)]));
vm_block = 79; continue;
}
// C line 32285
81 => {
vm_block = 8; continue;
}
// C line 32284
82 => {
let _ = quickjs_debug_write(&quickjs_format_print(c" %u,%u".as_ptr(), &[QuickJSPrintArg::Int(crate::cutils_header::get_u16((tab).offset(((pos) as isize))) as u64), QuickJSPrintArg::Int(crate::cutils_header::get_u16(((tab).offset(((pos) as isize))).offset((((2 as i32)) as isize))) as u64)]));
vm_block = 81; continue;
}
// C line 32282
83 => {
vm_block = 8; continue;
}
// C line 32281
84 => {
let _ = quickjs_debug_write(&quickjs_format_print(c" %u".as_ptr(), &[QuickJSPrintArg::Int(crate::cutils_header::get_u16((tab).offset(((pos) as isize))) as u64)]));
vm_block = 83; continue;
}
// C line 32278
85 => {
vm_block = 8; continue;
}
// C line 32277
86 => {
let _ = quickjs_debug_write(&quickjs_format_print(c" %d".as_ptr(), &[QuickJSPrintArg::Int(crate::cutils_header::get_i8((tab).offset(((pos) as isize))) as u64)]));
vm_block = 85; continue;
}
// C line 32275
87 => {
vm_block = 8; continue;
}
// C line 32274
88 => {
let _ = quickjs_debug_write(&quickjs_format_print(c" %u".as_ptr(), &[QuickJSPrintArg::Int(crate::cutils_header::get_u8((tab).offset(((pos) as isize))) as u64)]));
vm_block = 87; continue;
}
// C line 32262
89 => {
vm_block = match (((*(oi)).fmt) as i32) { x if x == (OP_FMT_var_ref as i32) => 14, x if x == (OP_FMT_arg as i32) => 19, x if x == (OP_FMT_loc as i32) => 24, x if x == (OP_FMT_loc8 as i32) => 26, x if x == (OP_FMT_atom_label_u16 as i32) => 39, x if x == (OP_FMT_atom_label_u8 as i32) => 39, x if x == (OP_FMT_atom_u16 as i32) => 43, x if x == (OP_FMT_atom_u8 as i32) => 47, x if x == (OP_FMT_atom as i32) => 50, x if x == (OP_FMT_const as i32) => 56, x if x == (OP_FMT_label_u16 as i32) => 65, x if x == (OP_FMT_label as i32) => 74, x if x == (OP_FMT_u32 as i32) => 76, x if x == (OP_FMT_i32 as i32) => 78, x if x == (OP_FMT_i16 as i32) => 80, x if x == (OP_FMT_npop_u16 as i32) => 82, x if x == (OP_FMT_npop as i32) => 84, x if x == (OP_FMT_u16 as i32) => 84, x if x == (OP_FMT_i8 as i32) => 86, x if x == (OP_FMT_u8 as i32) => 88, _ => 9, }; continue;
}
// C line 32261
90 => {
let _ = { let old = pos; pos = (pos).wrapping_add(1); old };
vm_block = 89; continue;
}
// C line 32260
91 => {
let _ = quickjs_debug_write(&quickjs_format_print(c"%s".as_ptr(), &[QuickJSPrintArg::Str(debug_opcode_name(op as usize, use_short_opcodes != 0) as *const c_char)]));
vm_block = 90; continue;
}
// C line 32256
92 => {
let _ = quickjs_debug_write(&quickjs_format_print(c"%5d:  ".as_ptr(), &[QuickJSPrintArg::Int(pos as u64)]));
vm_block = 91; continue;
}
// C line 32258
93 => {
let _ = quickjs_debug_write(&quickjs_format_print(c"        ".as_ptr(), &[]));
vm_block = 91; continue;
}
// C line 32255
94 => {
vm_block = if (dump_pc) != 0 { 92 } else { 93 }; continue;
}
// C line 32251
95 => {
let _ = { let assigned = (1 as i32); dump_pc = assigned; assigned };
vm_block = 94; continue;
}
// C line 32247
96 => {
let _ = quickjs_debug_write(&quickjs_format_print(c"%*s".as_ptr(), &[QuickJSPrintArg::Int(((x0).wrapping_add((20 as i32))).wrapping_sub(x) as u64), QuickJSPrintArg::Str(c"".as_ptr() as *const c_char)]));
vm_block = 95; continue;
}
// C line 32241
97 => {
vm_block = if ((((i) < (size)) as i32)) != 0 { 101 } else { 96 }; continue;
}
// C line 32241
98 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 97; continue;
}
// C line 32245
99 => {
let _ = { x = ((((x) as i32)).wrapping_add(quickjs_debug_write(&quickjs_format_print(c" %02X".as_ptr(), &[QuickJSPrintArg::Int(((*(tab).offset(((pos).wrapping_add(i)) as isize)) as i32) as u64)])))) as i32; x };
vm_block = 98; continue;
}
// C line 32243
100 => {
let _ = quickjs_debug_write(&quickjs_format_print(c"\n%*s".as_ptr(), &[QuickJSPrintArg::Int({ let assigned = x0; x = assigned; assigned } as u64), QuickJSPrintArg::Str(c"".as_ptr() as *const c_char)]));
vm_block = 99; continue;
}
// C line 32242
101 => {
vm_block = if ((((i) == ((6 as i32))) as i32)) != 0 { 100 } else { 99 }; continue;
}
// C line 32241
102 => {
let _ = { let assigned = (0 as i32); i = assigned; assigned };
vm_block = 97; continue;
}
// C line 32240
103 => {
let _ = { let assigned = { let assigned = quickjs_debug_write(&quickjs_format_print(c"%5d ".as_ptr(), &[QuickJSPrintArg::Int(pos as u64)])); x0 = assigned; assigned }; x = assigned; assigned };
vm_block = 102; continue;
}
// C line 32235
104 => {
vm_block = 5; continue;
}
// C line 32234
105 => {
let _ = quickjs_debug_write(&quickjs_format_print(c"truncated opcode (0x%02x)\n".as_ptr(), &[QuickJSPrintArg::Int(op as u64)]));
vm_block = 104; continue;
}
// C line 32233
106 => {
vm_block = if (((((pos).wrapping_add(size)) > (len)) as i32)) != 0 { 105 } else { 103 }; continue;
}
// C line 32232
107 => {
let _ = { let assigned = (((*(oi)).size) as i32); size = assigned; assigned };
vm_block = 106; continue;
}
// C line 32229
108 => {
let _ = { let assigned = core::ptr::addr_of!(*((opcode_info).as_ptr()).offset((op) as isize)); oi = assigned; assigned };
vm_block = 107; continue;
}
// C line 32231
109 => {
let _ = { let assigned = core::ptr::addr_of!(*((opcode_info).as_ptr()).offset((op) as isize)); oi = assigned; assigned };
vm_block = 107; continue;
}
// C line 32228
110 => {
vm_block = if (use_short_opcodes) != 0 { 108 } else { 109 }; continue;
}
// C line 32226
111 => {
vm_block = 6; continue;
}
// C line 32225
112 => {
let _ = { let old = pos; pos = (pos).wrapping_add(1); old };
vm_block = 111; continue;
}
// C line 32224
113 => {
let _ = quickjs_debug_write(&quickjs_format_print(c"invalid opcode (0x%02x)\n".as_ptr(), &[QuickJSPrintArg::Int(op as u64)]));
vm_block = 112; continue;
}
// C line 32223
114 => {
vm_block = if ((((op) >= ((OP_COUNT as i32))) as i32)) != 0 { 113 } else { 110 }; continue;
}
// C line 32222
115 => {
let _ = { let assigned = (0 as i32); in_source = assigned; assigned };
vm_block = 114; continue;
}
// C line 32221
116 => {
let _ = quickjs_debug_write(&quickjs_format_print(c"\n".as_ptr(), &[]));
vm_block = 115; continue;
}
// C line 32220
117 => {
vm_block = if (in_source) != 0 { 116 } else { 115 }; continue;
}
// C line 32216
118 => {
let _ = { let assigned = line1; line = assigned; assigned };
vm_block = 117; continue;
}
// C line 32215
119 => {
let _ = print_lines(source, line, line1);
vm_block = 118; continue;
}
// C line 32214
120 => {
let _ = { let assigned = (1 as i32); in_source = assigned; assigned };
vm_block = 119; continue;
}
// C line 32213
121 => {
let _ = quickjs_debug_write(&quickjs_format_print(c"\n".as_ptr(), &[]));
vm_block = 120; continue;
}
// C line 32212
122 => {
vm_block = if ((!((in_source) != 0) as i32)) != 0 { 121 } else { 120 }; continue;
}
// C line 32211
123 => {
vm_block = if ((((line1) > (line)) as i32)) != 0 { 122 } else { 117 }; continue;
}
// C line 32206
124 => {
let _ = { let assigned = ((find_line_num(ctx, b, ((pos) as u32), core::ptr::addr_of_mut!(col_num_1))).wrapping_sub(line_num)).wrapping_add((1 as i32)); line1 = assigned; assigned };
vm_block = 123; continue;
}
// C line 32209
125 => {
let _ = { let assigned = ((((crate::cutils_header::get_u32(((tab).offset(((pos) as isize))).offset((((1 as i32)) as isize)))).wrapping_sub(((line_num) as u32))).wrapping_add((((1 as i32)) as u32))) as i32); line1 = assigned; assigned };
vm_block = 123; continue;
}
// C line 32207
126 => {
vm_block = if ((((op) == ((OP_line_num as i32))) as i32)) != 0 { 125 } else { 123 }; continue;
}
// C line 32205
127 => {
vm_block = if !(b).is_null() { 124 } else { 126 }; continue;
}
// C line 32203
128 => {
vm_block = if ((((!(source).is_null()) && (!(b).is_null())) as i32)) != 0 { 127 } else { 117 }; continue;
}
// C line 32202
129 => {
let _ = { let assigned = ((*(tab).offset((pos) as isize)) as i32); op = assigned; assigned };
vm_block = 128; continue;
}
// C line 32200
130 => {
let _ = { let assigned = (0 as i32); pos = assigned; assigned };
vm_block = 6; continue;
}
// C line 32199
131 => {
let _ = { let assigned = { let assigned = (1 as i32); line = assigned; assigned }; line1 = assigned; assigned };
vm_block = 130; continue;
}
// C line 32197
132 => {
let _ = { let assigned = (1 as i32); in_source = assigned; assigned };
vm_block = 131; continue;
}
// C line 32196
133 => {
let _ = print_lines(source, (0 as i32), (1 as i32));
vm_block = 132; continue;
}
// C line 32194
134 => {
vm_block = if !(source).is_null() { 133 } else { 131 }; continue;
}
// C line 32193
135 => {
let _ = { let assigned = (0 as i32); in_source = assigned; assigned };
vm_block = 134; continue;
}
// C line 32152
136 => {
vm_block = if ((((pos) < (len)) as i32)) != 0 { 157 } else { 135 }; continue;
}
// C line 32152
137 => {
let _ = { let assigned = pos_next; pos = assigned; assigned };
vm_block = 136; continue;
}
// C line 32189
138 => {
vm_block = 137; continue;
}
// C line 32188
139 => {
let _ = { *(bits).offset((addr) as isize) = (((((*(bits).offset((addr) as isize)) as i32)) | ((1 as i32)))) as u8; *(bits).offset((addr) as isize) };
vm_block = 138; continue;
}
// C line 32187
140 => {
vm_block = if ((((((((addr) >= ((0 as i32))) as i32)) != 0) && (((((addr) < (len)) as i32)) != 0)) as i32)) != 0 { 139 } else { 138 }; continue;
}
// C line 32186
141 => {
let _ = { addr = ((((addr) as i32)).wrapping_add(pos)) as i32; addr };
vm_block = 140; continue;
}
// C line 32185
142 => {
vm_block = if ((((pass) == ((3 as i32))) as i32)) != 0 { 141 } else { 140 }; continue;
}
// C line 32184
143 => {
let _ = { let assigned = (*(label_slots).offset((addr) as isize)).pos2; addr = assigned; assigned };
vm_block = 142; continue;
}
// C line 32183
144 => {
vm_block = if ((((pass) == ((2 as i32))) as i32)) != 0 { 143 } else { 142 }; continue;
}
// C line 32182
145 => {
let _ = { let assigned = (*(label_slots).offset((addr) as isize)).pos; addr = assigned; assigned };
vm_block = 144; continue;
}
// C line 32181 labels: has_addr
146 => {
vm_block = if ((((pass) == ((1 as i32))) as i32)) != 0 { 145 } else { 144 }; continue;
}
// C line 32179
147 => {
vm_block = 146; continue;
}
// C line 32178
148 => {
let _ = { let assigned = ((crate::cutils_header::get_u32((tab).offset(((pos) as isize)))) as i32); addr = assigned; assigned };
vm_block = 147; continue;
}
// C line 32177
149 => {
let _ = { let old = pos; pos = (pos).wrapping_add(1); old };
vm_block = 148; continue;
}
// C line 32173
150 => {
let _ = { pos = ((((pos) as i32)).wrapping_add((4 as i32))) as i32; pos };
vm_block = 149; continue;
}
// C line 32160
151 => {
vm_block = match (((*(oi)).fmt) as i32) { x if x == (OP_FMT_label_u16 as i32) => 149, x if x == (OP_FMT_label as i32) => 149, x if x == (OP_FMT_atom_label_u16 as i32) => 150, x if x == (OP_FMT_atom_label_u8 as i32) => 150, _ => 137, }; continue;
}
// C line 32159
152 => {
vm_block = if ((((op) < ((OP_COUNT as i32))) as i32)) != 0 { 151 } else { 137 }; continue;
}
// C line 32158
153 => {
let _ = { let assigned = (pos).wrapping_add((((*(oi)).size) as i32)); pos_next = assigned; assigned };
vm_block = 152; continue;
}
// C line 32155
154 => {
let _ = { let assigned = core::ptr::addr_of!(*((opcode_info).as_ptr()).offset((op) as isize)); oi = assigned; assigned };
vm_block = 153; continue;
}
// C line 32157
155 => {
let _ = { let assigned = core::ptr::addr_of!(*((opcode_info).as_ptr()).offset((op) as isize)); oi = assigned; assigned };
vm_block = 153; continue;
}
// C line 32154
156 => {
vm_block = if (use_short_opcodes) != 0 { 154 } else { 155 }; continue;
}
// C line 32153
157 => {
let _ = { let assigned = ((*(tab).offset((pos) as isize)) as i32); op = assigned; assigned };
vm_block = 156; continue;
}
// C line 32152
158 => {
let _ = { let assigned = (0 as i32); pos = assigned; assigned };
vm_block = 136; continue;
}
// C line 32148
159 => {
let _ = { let assigned = find_line_num(ctx, b, ((((1 as i32)).wrapping_neg()) as u32), core::ptr::addr_of_mut!(col_num)); line_num = assigned; assigned };
vm_block = 158; continue;
}
// C line 32146
160 => {
vm_block = if !(b).is_null() { 159 } else { 158 }; continue;
}
// C line 32144
161 => {
use_short_opcodes = (((b) != (core::ptr::null_mut::<JSFunctionBytecode>())) as i32);
vm_block = 160; continue;
}
// C line 32143
162 => {
bits = ((js_mallocz(ctx, (((len) as usize)).wrapping_mul((size_of::<u8>() as usize)))) as *mut u8);
vm_block = 161; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:32414. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn dump_pc2line(mut ctx: *mut JSContext, mut buf: *const u8, mut len: i32) -> () {
let mut vm_local_storage = Vec::<u64>::new();
let mut p_end: *const u8 = core::mem::zeroed();
let mut p: *const u8 = core::mem::zeroed();
let mut pc: i32 = core::mem::zeroed();
let mut v: i32 = core::mem::zeroed();
let mut line_num: i32 = core::mem::zeroed();
let mut col_num: i32 = core::mem::zeroed();
let mut ret: i32 = core::mem::zeroed();
let mut op: u32 = core::mem::zeroed();
let mut val: u32 = core::mem::zeroed();
let mut vm_block: usize = 39;
loop { match vm_block {
// C line ? labels: fail
0 => {
return ();
}
// C line 32445
1 => {
vm_block = if ((((p) < (p_end)) as i32)) != 0 { 22 } else { 0 }; continue;
}
// C line 32469
2 => {
let _ = quickjs_debug_write(&quickjs_format_print(c"%5d %5d %5d\n".as_ptr(), &[QuickJSPrintArg::Int(pc as u64), QuickJSPrintArg::Int(line_num as u64), QuickJSPrintArg::Int(col_num as u64)]));
vm_block = 1; continue;
}
// C line 32467
3 => {
let _ = { col_num = ((((col_num) as i32)).wrapping_add(v)) as i32; col_num };
vm_block = 2; continue;
}
// C line 32466
4 => {
let _ = { p = ((((p) as *const u8)).offset(((ret) as isize))) as *const u8; p };
vm_block = 3; continue;
}
// C line 32465
5 => {
vm_block = 0; continue;
}
// C line 32464
6 => {
vm_block = if ((((ret) < ((0 as i32))) as i32)) != 0 { 5 } else { 4 }; continue;
}
// C line 32463
7 => {
let _ = { let assigned = get_sleb128(core::ptr::addr_of_mut!(v), p, p_end); ret = assigned; assigned };
vm_block = 6; continue;
}
// C line 32457
8 => {
let _ = { line_num = ((((line_num) as i32)).wrapping_add(v)) as i32; line_num };
vm_block = 7; continue;
}
// C line 32456
9 => {
let _ = { p = ((((p) as *const u8)).offset(((ret) as isize))) as *const u8; p };
vm_block = 8; continue;
}
// C line 32455
10 => {
vm_block = 0; continue;
}
// C line 32454
11 => {
vm_block = if ((((ret) < ((0 as i32))) as i32)) != 0 { 10 } else { 9 }; continue;
}
// C line 32453
12 => {
let _ = { let assigned = get_sleb128(core::ptr::addr_of_mut!(v), p, p_end); ret = assigned; assigned };
vm_block = 11; continue;
}
// C line 32452
13 => {
let _ = { p = ((((p) as *const u8)).offset(((ret) as isize))) as *const u8; p };
vm_block = 12; continue;
}
// C line 32451
14 => {
let _ = { pc = ((((pc) as u32)).wrapping_add(val)) as i32; pc };
vm_block = 13; continue;
}
// C line 32450
15 => {
vm_block = 0; continue;
}
// C line 32449
16 => {
vm_block = if ((((ret) < ((0 as i32))) as i32)) != 0 { 15 } else { 14 }; continue;
}
// C line 32448
17 => {
let _ = { let assigned = get_leb128(core::ptr::addr_of_mut!(val), p, p_end); ret = assigned; assigned };
vm_block = 16; continue;
}
// C line 32461
18 => {
let _ = { line_num = ((((line_num) as u32)).wrapping_add((((op) % ((((5 as i32)) as u32)))).wrapping_add(((((1 as i32)).wrapping_neg()) as u32)))) as i32; line_num };
vm_block = 7; continue;
}
// C line 32460
19 => {
let _ = { pc = ((((pc) as u32)).wrapping_add(((op) / ((((5 as i32)) as u32))))) as i32; pc };
vm_block = 18; continue;
}
// C line 32459
20 => {
let _ = { op = ((((op) as u32)).wrapping_sub((((1 as i32)) as u32))) as u32; op };
vm_block = 19; continue;
}
// C line 32447
21 => {
vm_block = if ((((op) == ((((0 as i32)) as u32))) as i32)) != 0 { 17 } else { 20 }; continue;
}
// C line 32446
22 => {
let _ = { let assigned = ((*({ let old = p; p = (p).offset(1); old })) as u32); op = assigned; assigned };
vm_block = 21; continue;
}
// C line 32444
23 => {
let _ = { let assigned = (0 as i32); pc = assigned; assigned };
vm_block = 1; continue;
}
// C line 32442
24 => {
let _ = quickjs_debug_write(&quickjs_format_print(c"%5s %5d %5d\n".as_ptr(), &[QuickJSPrintArg::Str(c"-".as_ptr() as *const c_char), QuickJSPrintArg::Int(line_num as u64), QuickJSPrintArg::Int(col_num as u64)]));
vm_block = 23; continue;
}
// C line 32440
25 => {
let _ = { let assigned = (((val).wrapping_add((((1 as i32)) as u32))) as i32); col_num = assigned; assigned };
vm_block = 24; continue;
}
// C line 32439
26 => {
let _ = { p = ((((p) as *const u8)).offset(((ret) as isize))) as *const u8; p };
vm_block = 25; continue;
}
// C line 32438
27 => {
vm_block = 0; continue;
}
// C line 32437
28 => {
vm_block = if ((((ret) < ((0 as i32))) as i32)) != 0 { 27 } else { 26 }; continue;
}
// C line 32436
29 => {
let _ = { let assigned = get_leb128(core::ptr::addr_of_mut!(val), p, p_end); ret = assigned; assigned };
vm_block = 28; continue;
}
// C line 32434
30 => {
let _ = { let assigned = (((val).wrapping_add((((1 as i32)) as u32))) as i32); line_num = assigned; assigned };
vm_block = 29; continue;
}
// C line 32433
31 => {
let _ = { p = ((((p) as *const u8)).offset(((ret) as isize))) as *const u8; p };
vm_block = 30; continue;
}
// C line 32432
32 => {
vm_block = 0; continue;
}
// C line 32431
33 => {
vm_block = if ((((ret) < ((0 as i32))) as i32)) != 0 { 32 } else { 31 }; continue;
}
// C line 32430
34 => {
let _ = { let assigned = get_leb128(core::ptr::addr_of_mut!(val), p, p_end); ret = assigned; assigned };
vm_block = 33; continue;
}
// C line 32427
35 => {
let _ = { let assigned = (buf).offset(((len) as isize)); p_end = assigned; assigned };
vm_block = 34; continue;
}
// C line 32426
36 => {
let _ = { let assigned = buf; p = assigned; assigned };
vm_block = 35; continue;
}
// C line 32424
37 => {
let _ = quickjs_debug_write(&quickjs_format_print(c"%5s %5s %5s\n".as_ptr(), &[QuickJSPrintArg::Str(c"PC".as_ptr() as *const c_char), QuickJSPrintArg::Str(c"LINE".as_ptr() as *const c_char), QuickJSPrintArg::Str(c"COL".as_ptr() as *const c_char)]));
vm_block = 36; continue;
}
// C line 32422
38 => {
return;
}
// C line 32421
39 => {
vm_block = if ((((len) <= ((0 as i32))) as i32)) != 0 { 38 } else { 37 }; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:32474. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_dump_function_bytecode(mut ctx: *mut JSContext, mut b: *mut JSFunctionBytecode) -> () {
let mut vm_local_storage = Vec::<u64>::new();
let mut i: i32 = core::mem::zeroed();
let mut atom_buf: [c_char; 64] = core::mem::zeroed();
let mut str: *const c_char = core::mem::zeroed();
let mut line_num: i32 = core::mem::zeroed();
let mut col_num: i32 = core::mem::zeroed();
let mut vd: *mut JSBytecodeVarDef = core::mem::zeroed();
let mut cv: *mut JSClosureVar = core::mem::zeroed();
let mut vm_block: usize = 61;
loop { match vm_block {
// C line ?
0 => {
return ();
}
// C line 32573
1 => {
let _ = quickjs_debug_write(&quickjs_format_print(c"\n".as_ptr(), &[]));
vm_block = 0; continue;
}
// C line 32571
2 => {
let _ = dump_pc2line(ctx, ((*(b)).debug).pc2line_buf, ((*(b)).debug).pc2line_len);
vm_block = 1; continue;
}
// C line 32570
3 => {
vm_block = if ((*(b)).has_debug()) != 0 { 2 } else { 1 }; continue;
}
// C line 32561
4 => {
let _ = dump_byte_code(ctx, (3 as i32), (*(b)).byte_code_buf, (*(b)).byte_code_len, (*(b)).vardefs, core::ptr::null_mut::<JSVarDef>(), (((*(b)).arg_count) as i32), core::ptr::null_mut::<JSVarDef>(), (((*(b)).var_count) as i32), (*(b)).closure_var, (*(b)).closure_var_count, (*(b)).cpool, (((*(b)).cpool_count) as u32), if ((((*(b)).has_debug()) as i32)) != 0 { ((*(b)).debug).source } else { core::ptr::null_mut::<c_char>() }, core::ptr::null_mut::<LabelSlot>(), b);
vm_block = 3; continue;
}
// C line 32560
5 => {
let _ = quickjs_debug_write(&quickjs_format_print(c"  opcodes:\n".as_ptr(), &[]));
vm_block = 4; continue;
}
// C line 32559
6 => {
let _ = quickjs_debug_write(&quickjs_format_print(c"  var_ref_count: %d\n".as_ptr(), &[QuickJSPrintArg::Int((((*(b)).var_ref_count) as i32) as u64)]));
vm_block = 5; continue;
}
// C line 32558
7 => {
let _ = quickjs_debug_write(&quickjs_format_print(c"  stack_size: %d\n".as_ptr(), &[QuickJSPrintArg::Int((((*(b)).stack_size) as i32) as u64)]));
vm_block = 6; continue;
}
// C line 32521
8 => {
vm_block = if ((((i) < ((*(b)).closure_var_count)) as i32)) != 0 { 30 } else { 7 }; continue;
}
// C line 32521
9 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 8; continue;
}
// C line 32554
10 => {
vm_block = 9; continue;
}
// C line 32553
11 => {
let _ = quickjs_debug_write(&quickjs_format_print(c" [?]\n".as_ptr(), &[]));
vm_block = 10; continue;
}
// C line 32551
12 => {
vm_block = 9; continue;
}
// C line 32550
13 => {
let _ = quickjs_debug_write(&quickjs_format_print(c" [module_import]\n".as_ptr(), &[]));
vm_block = 12; continue;
}
// C line 32548
14 => {
vm_block = 9; continue;
}
// C line 32547
15 => {
let _ = quickjs_debug_write(&quickjs_format_print(c" [module_decl]\n".as_ptr(), &[]));
vm_block = 14; continue;
}
// C line 32545
16 => {
vm_block = 9; continue;
}
// C line 32544
17 => {
let _ = quickjs_debug_write(&quickjs_format_print(c" [global]\n".as_ptr(), &[]));
vm_block = 16; continue;
}
// C line 32542
18 => {
vm_block = 9; continue;
}
// C line 32541
19 => {
let _ = quickjs_debug_write(&quickjs_format_print(c" [global_decl]\n".as_ptr(), &[]));
vm_block = 18; continue;
}
// C line 32539
20 => {
vm_block = 9; continue;
}
// C line 32538
21 => {
let _ = quickjs_debug_write(&quickjs_format_print(c" [global_ref%d]\n".as_ptr(), &[QuickJSPrintArg::Int((((*(cv)).var_idx) as i32) as u64)]));
vm_block = 20; continue;
}
// C line 32536
22 => {
vm_block = 9; continue;
}
// C line 32535
23 => {
let _ = quickjs_debug_write(&quickjs_format_print(c" [ref%d]\n".as_ptr(), &[QuickJSPrintArg::Int((((*(cv)).var_idx) as i32) as u64)]));
vm_block = 22; continue;
}
// C line 32533
24 => {
vm_block = 9; continue;
}
// C line 32532
25 => {
let _ = quickjs_debug_write(&quickjs_format_print(c" [arg%d]\n".as_ptr(), &[QuickJSPrintArg::Int((((*(cv)).var_idx) as i32) as u64)]));
vm_block = 24; continue;
}
// C line 32530
26 => {
vm_block = 9; continue;
}
// C line 32529
27 => {
let _ = quickjs_debug_write(&quickjs_format_print(c" [loc%d]\n".as_ptr(), &[QuickJSPrintArg::Int((((*(cv)).var_idx) as i32) as u64)]));
vm_block = 26; continue;
}
// C line 32527
28 => {
vm_block = match (((*(cv)).closure_type()) as i32) { x if x == (JS_CLOSURE_MODULE_IMPORT as i32) => 13, x if x == (JS_CLOSURE_MODULE_DECL as i32) => 15, x if x == (JS_CLOSURE_GLOBAL as i32) => 17, x if x == (JS_CLOSURE_GLOBAL_DECL as i32) => 19, x if x == (JS_CLOSURE_GLOBAL_REF as i32) => 21, x if x == (JS_CLOSURE_REF as i32) => 23, x if x == (JS_CLOSURE_ARG as i32) => 25, x if x == (JS_CLOSURE_LOCAL as i32) => 27, _ => 11, }; continue;
}
// C line 32523
29 => {
let _ = quickjs_debug_write(&quickjs_format_print(c"%5d: %s %s".as_ptr(), &[QuickJSPrintArg::Int(i as u64), QuickJSPrintArg::Str(if ((((*(cv)).is_const()) as i32)) != 0 { c"const".as_ptr() } else { if ((((*(cv)).is_lexical()) as i32)) != 0 { c"let".as_ptr() } else { c"var".as_ptr() } } as *const c_char), QuickJSPrintArg::Str(JS_AtomGetStr(ctx, (atom_buf).as_mut_ptr(), (((size_of::<[c_char; 64]>() as usize)) as i32), (*(cv)).var_name) as *const c_char)]));
vm_block = 28; continue;
}
// C line 32522
30 => {
cv = core::ptr::addr_of_mut!(*((*(b)).closure_var).offset((i) as isize));
vm_block = 29; continue;
}
// C line 32521
31 => {
let _ = { let assigned = (0 as i32); i = assigned; assigned };
vm_block = 8; continue;
}
// C line 32520
32 => {
let _ = quickjs_debug_write(&quickjs_format_print(c"  closure vars:\n".as_ptr(), &[]));
vm_block = 31; continue;
}
// C line 32519
33 => {
vm_block = if ((*(b)).closure_var_count) != 0 { 32 } else { 7 }; continue;
}
// C line 32505
34 => {
vm_block = if ((((i) < ((((*(b)).var_count) as i32))) as i32)) != 0 { 40 } else { 33 }; continue;
}
// C line 32505
35 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 34; continue;
}
// C line 32516
36 => {
let _ = quickjs_debug_write(&quickjs_format_print(c"\n".as_ptr(), &[]));
vm_block = 35; continue;
}
// C line 32515
37 => {
let _ = quickjs_debug_write(&quickjs_format_print(c" [next:%d]".as_ptr(), &[QuickJSPrintArg::Int((*(vd)).scope_next as u64)]));
vm_block = 36; continue;
}
// C line 32514
38 => {
vm_block = if ((*(vd)).has_scope()) != 0 { 37 } else { 36 }; continue;
}
// C line 32507
39 => {
let _ = quickjs_debug_write(&quickjs_format_print(c"%5d: %s %s".as_ptr(), &[QuickJSPrintArg::Int(i as u64), QuickJSPrintArg::Str(if (((((((*(vd)).var_kind()) as i32)) == ((JS_VAR_CATCH as i32))) as i32)) != 0 { c"catch".as_ptr() } else { if (((((((((((*(vd)).var_kind()) as i32)) == ((JS_VAR_FUNCTION_DECL as i32))) as i32)) != 0) || ((((((((*(vd)).var_kind()) as i32)) == ((JS_VAR_NEW_FUNCTION_DECL as i32))) as i32)) != 0)) as i32)) != 0 { c"function".as_ptr() } else { if ((((*(vd)).is_const()) as i32)) != 0 { c"const".as_ptr() } else { if ((((*(vd)).is_lexical()) as i32)) != 0 { c"let".as_ptr() } else { c"var".as_ptr() } } } } as *const c_char), QuickJSPrintArg::Str(JS_AtomGetStr(ctx, (atom_buf).as_mut_ptr(), (((size_of::<[c_char; 64]>() as usize)) as i32), (*(vd)).var_name) as *const c_char)]));
vm_block = 38; continue;
}
// C line 32506
40 => {
vd = core::ptr::addr_of_mut!(*((*(b)).vardefs).offset((((((*(b)).arg_count) as i32)).wrapping_add(i)) as isize));
vm_block = 39; continue;
}
// C line 32505
41 => {
let _ = { let assigned = (0 as i32); i = assigned; assigned };
vm_block = 34; continue;
}
// C line 32504
42 => {
let _ = quickjs_debug_write(&quickjs_format_print(c"  locals:\n".as_ptr(), &[]));
vm_block = 41; continue;
}
// C line 32503
43 => {
vm_block = if ((((((((*(b)).var_count) as i32)) != 0) && (!((*(b)).vardefs).is_null())) as i32)) != 0 { 42 } else { 33 }; continue;
}
// C line 32501
44 => {
let _ = quickjs_debug_write(&quickjs_format_print(c"\n".as_ptr(), &[]));
vm_block = 43; continue;
}
// C line 32497
45 => {
vm_block = if ((((i) < ((((*(b)).arg_count) as i32))) as i32)) != 0 { 47 } else { 44 }; continue;
}
// C line 32497
46 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 45; continue;
}
// C line 32498
47 => {
let _ = quickjs_debug_write(&quickjs_format_print(c" %s".as_ptr(), &[QuickJSPrintArg::Str(JS_AtomGetStr(ctx, (atom_buf).as_mut_ptr(), (((size_of::<[c_char; 64]>() as usize)) as i32), (*((*(b)).vardefs).offset((i) as isize)).var_name) as *const c_char)]));
vm_block = 46; continue;
}
// C line 32497
48 => {
let _ = { let assigned = (0 as i32); i = assigned; assigned };
vm_block = 45; continue;
}
// C line 32496
49 => {
let _ = quickjs_debug_write(&quickjs_format_print(c"  args:".as_ptr(), &[]));
vm_block = 48; continue;
}
// C line 32495
50 => {
vm_block = if ((((((((*(b)).arg_count) as i32)) != 0) && (!((*(b)).vardefs).is_null())) as i32)) != 0 { 49 } else { 43 }; continue;
}
// C line 32493
51 => {
let _ = quickjs_debug_write(&quickjs_format_print(c"\n".as_ptr(), &[]));
vm_block = 50; continue;
}
// C line 32492
52 => {
let _ = quickjs_debug_write(&quickjs_format_print(c" strict".as_ptr(), &[]));
vm_block = 51; continue;
}
// C line 32491
53 => {
vm_block = if ((((((*(b)).js_mode) as i32)) & (((1 as i32)).wrapping_shl(((0 as i32)) as u32)))) != 0 { 52 } else { 51 }; continue;
}
// C line 32490
54 => {
let _ = quickjs_debug_write(&quickjs_format_print(c"  mode:".as_ptr(), &[]));
vm_block = 53; continue;
}
// C line 32489
55 => {
vm_block = if ((*(b)).js_mode) != 0 { 54 } else { 50 }; continue;
}
// C line 32488
56 => {
let _ = quickjs_debug_write(&quickjs_format_print(c"function: %s%s\n".as_ptr(), &[QuickJSPrintArg::Str(core::ptr::addr_of!(*(c"*".as_ptr()).offset((((((((*(b)).func_kind()) as i32)) != ((JS_FUNC_GENERATOR as i32))) as i32)) as isize)) as *const c_char), QuickJSPrintArg::Str(str as *const c_char)]));
vm_block = 55; continue;
}
// C line 32487
57 => {
let _ = { let assigned = JS_AtomGetStr(ctx, (atom_buf).as_mut_ptr(), (((size_of::<[c_char; 64]>() as usize)) as i32), (*(b)).func_name); str = assigned; assigned };
vm_block = 56; continue;
}
// C line 32484
58 => {
let _ = quickjs_debug_write(&quickjs_format_print(c"%s:%d:%d: ".as_ptr(), &[QuickJSPrintArg::Str(str as *const c_char), QuickJSPrintArg::Int(line_num as u64), QuickJSPrintArg::Int(col_num as u64)]));
vm_block = 57; continue;
}
// C line 32483
59 => {
let _ = { let assigned = find_line_num(ctx, b, ((((1 as i32)).wrapping_neg()) as u32), core::ptr::addr_of_mut!(col_num)); line_num = assigned; assigned };
vm_block = 58; continue;
}
// C line 32482
60 => {
let _ = { let assigned = JS_AtomGetStr(ctx, (atom_buf).as_mut_ptr(), (((size_of::<[c_char; 64]>() as usize)) as i32), ((*(b)).debug).filename); str = assigned; assigned };
vm_block = 59; continue;
}
// C line 32480
61 => {
vm_block = if ((((((((*(b)).has_debug()) as i32)) != 0) && (((((((*(b)).debug).filename) != ((((0 as i32)) as JSAtom))) as i32)) != 0)) as i32)) != 0 { 60 } else { 57 }; continue;
}
_ => std::process::abort(),
} }
}
