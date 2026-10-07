// Original #if 0 alternatives: compiled, but intentionally never called by the CLI.
// Generated from the official C AST; original goto CFG retained.
// c: unicode_gen.c:463 original #if 0 block retained in unused copy. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn parse_unicode_data_dormant(mut filename: *const c_char) -> () {
let mut vm_local_storage = Vec::<u64>::new();
let mut f: *mut FILE = core::mem::zeroed();
let mut line: [c_char; 1024] = core::mem::zeroed();
let mut buf1: [c_char; 256] = core::mem::zeroed();
let mut p: *const c_char = core::mem::zeroed();
let mut code: i32 = core::mem::zeroed();
let mut lc: i32 = core::mem::zeroed();
let mut uc: i32 = core::mem::zeroed();
let mut last_code: i32 = core::mem::zeroed();
let mut ci: *mut CCInfo = core::mem::zeroed();
let mut tab: *mut CCInfo = core::mem::zeroed();
let mut i: i32 = core::mem::zeroed();
let mut cc: i32 = core::mem::zeroed();
let mut size: i32 = core::mem::zeroed();
let mut i_1: i32 = core::mem::zeroed();
static mut count: i32 = 0;
static mut d_count: i32 = 0;
let mut i_2: i32 = core::mem::zeroed();
let mut vm_block: usize = 87;
loop { match vm_block {
// C line ?
0 => {
return ();
}
// C line 136
1 => {
let _ = fclose(f);
vm_block = 0; continue;
}
// C line 18
2 => {
vm_block = 81; continue;
}
// C line 133
3 => {
let _ = { let assigned = code; last_code = assigned; assigned };
vm_block = 2; continue;
}
// C line 129
4 => {
vm_block = if ((((i_2) < (code)) as i32)) != 0 { 6 } else { 3 }; continue;
}
// C line ?
5 => {
let _ = { let old = i_2; i_2 = (i_2).wrapping_add(1); old };
vm_block = 4; continue;
}
// C line 130
6 => {
let _ = { let assigned = *(ci); *(unicode_db).offset((i_2) as isize) = assigned; assigned };
vm_block = 5; continue;
}
// C line 129
7 => {
let _ = { let assigned = (last_code).wrapping_add((1 as i32)); i_2 = assigned; assigned };
vm_block = 4; continue;
}
// C line 128
8 => {
let _ = if ((((!((((((((*(ci)).script_ext_len) as i32)) == ((0 as i32))) as i32)) != 0) as i32)) as i64)) != 0 { tool_assert_rtn(c"parse_unicode_data_dormant".as_ptr(), c"dormant-block.c".as_ptr(), (128 as i32), c"ci->script_ext_len == 0".as_ptr()) } else { { let _ = (0 as i32); } };
vm_block = 7; continue;
}
// C line 127
9 => {
let _ = if ((((!((((((*(ci)).decomp_len) == ((0 as i32))) as i32)) != 0) as i32)) as i64)) != 0 { tool_assert_rtn(c"parse_unicode_data_dormant".as_ptr(), c"dormant-block.c".as_ptr(), (127 as i32), c"ci->decomp_len == 0".as_ptr()) } else { { let _ = (0 as i32); } };
vm_block = 8; continue;
}
// C line 124
10 => {
vm_block = if !(strstr((buf1).as_mut_ptr(), c" Last>".as_ptr())).is_null() { 9 } else { 3 }; continue;
}
// C line 123
11 => {
let _ = get_field_buf((buf1).as_mut_ptr(), (size_of::<[c_char; 256]>() as usize), (line).as_mut_ptr(), (1 as i32));
vm_block = 10; continue;
}
// C line 119
12 => {
let _ = set_prop(((code) as u32), (PROP_Bidi_Mirrored as i32), (1 as i32));
vm_block = 11; continue;
}
// C line 118
13 => {
vm_block = if ((((!(p).is_null()) && (((((((*(p)) as i32)) == ((89 as i32))) as i32)) != 0)) as i32)) != 0 { 12 } else { 11 }; continue;
}
// C line 117
14 => {
let _ = { let assigned = get_field((line).as_mut_ptr(), (9 as i32)); p = assigned; assigned };
vm_block = 13; continue;
}
// C line 111
15 => {
let _ = { d_count = (d_count).wrapping_add((*(ci)).decomp_len); d_count };
vm_block = 14; continue;
}
// C line 110
16 => {
let _ = { let old = count; count = (count).wrapping_add(1); old };
vm_block = 15; continue;
}
// C line 109
17 => {
let _ = tool_printf(c"\n".as_ptr(), &[]);
vm_block = 16; continue;
}
// C line 107
18 => {
vm_block = if ((((i_1) < ((*(ci)).decomp_len)) as i32)) != 0 { 20 } else { 17 }; continue;
}
// C line ?
19 => {
let _ = { let old = i_1; i_1 = (i_1).wrapping_add(1); old };
vm_block = 18; continue;
}
// C line 108
20 => {
let _ = tool_printf(c" %05x".as_ptr(), &[ToolPrintArg::Int(*((*(ci)).decomp_data).offset((i_1) as isize) as u64)]);
vm_block = 19; continue;
}
// C line 107
21 => {
let _ = { let assigned = (0 as i32); i_1 = assigned; assigned };
vm_block = 18; continue;
}
// C line 106
22 => {
let _ = tool_printf(c"%05x: %c".as_ptr(), &[ToolPrintArg::Int(code as u64), ToolPrintArg::Int(if ((((*(ci)).is_compat()) as i32)) != 0 { (67 as i32) } else { (32 as i32) } as u64)]);
vm_block = 21; continue;
}
// C line 94
23 => {
vm_block = 27; continue;
}
// C line 99
24 => {
let _ = add_char(ptr::addr_of_mut!((*(ci)).decomp_data), ptr::addr_of_mut!(size), ptr::addr_of_mut!((*(ci)).decomp_len), ((strtoul(p, ((ptr::addr_of_mut!(p)) as *mut *mut c_char), (16 as i32))) as i32));
vm_block = 23; continue;
}
// C line 98
25 => {
vm_block = 22; continue;
}
// C line 97
26 => {
vm_block = if ((!((isxdigit(((*(p)) as i32))) != 0) as i32)) != 0 { 25 } else { 24 }; continue;
}
// C line 95
27 => {
vm_block = if (isspace(((*(p)) as i32))) != 0 { 28 } else { 26 }; continue;
}
// C line 96
28 => {
let _ = { let old = p; p = (p).offset(1); old };
vm_block = 27; continue;
}
// C line 93
29 => {
let _ = { let assigned = (0 as i32); size = assigned; assigned };
vm_block = 23; continue;
}
// C line 91
30 => {
let _ = { let assigned = (((1 as i32)) as u8); (*(ci)).set_is_compat((assigned) as _); assigned };
vm_block = 29; continue;
}
// C line 90
31 => {
let _ = { let old = p; p = (p).offset(1); old };
vm_block = 30; continue;
}
// C line 89
32 => {
vm_block = if ((((((*(p)) as i32)) == ((62 as i32))) as i32)) != 0 { 31 } else { 30 }; continue;
}
// C line 87
33 => {
vm_block = if ((((((((((*(p)) as i32)) != ((0 as i32))) as i32)) != 0) && (((((((*(p)) as i32)) != ((62 as i32))) as i32)) != 0)) as i32)) != 0 { 34 } else { 32 }; continue;
}
// C line 88
34 => {
let _ = { let old = p; p = (p).offset(1); old };
vm_block = 33; continue;
}
// C line 86
35 => {
vm_block = if ((((((*(p)) as i32)) == ((60 as i32))) as i32)) != 0 { 33 } else { 29 }; continue;
}
// C line 85
36 => {
let _ = { let assigned = (((0 as i32)) as u8); (*(ci)).set_is_compat((assigned) as _); assigned };
vm_block = 35; continue;
}
// C line 84
37 => {
let _ = if ((((!(((((code) <= ((1114111 as i32))) as i32)) != 0) as i32)) as i64)) != 0 { tool_assert_rtn(c"parse_unicode_data_dormant".as_ptr(), c"dormant-block.c".as_ptr(), (84 as i32), c"code <= CHARCODE_MAX".as_ptr()) } else { { let _ = (0 as i32); } };
vm_block = 36; continue;
}
// C line 82
38 => {
vm_block = if ((((((((!(p).is_null()) && (((((((*(p)) as i32)) != ((59 as i32))) as i32)) != 0)) as i32)) != 0) && (((((((*(p)) as i32)) != ((0 as i32))) as i32)) != 0)) as i32)) != 0 { 37 } else { 14 }; continue;
}
// C line 81
39 => {
let _ = { let assigned = get_field((line).as_mut_ptr(), (5 as i32)); p = assigned; assigned };
vm_block = 38; continue;
}
// C line 76
40 => {
let _ = { let assigned = ((cc) as u8); (*(ci)).combining_class = assigned; assigned };
vm_block = 39; continue;
}
// C line 75
41 => {
let _ = if ((((!(((((code) <= ((1114111 as i32))) as i32)) != 0) as i32)) as i64)) != 0 { tool_assert_rtn(c"parse_unicode_data_dormant".as_ptr(), c"dormant-block.c".as_ptr(), (75 as i32), c"code <= CHARCODE_MAX".as_ptr()) } else { { let _ = (0 as i32); } };
vm_block = 40; continue;
}
// C line 74
42 => {
vm_block = if ((((cc) != ((0 as i32))) as i32)) != 0 { 41 } else { 39 }; continue;
}
// C line 73
43 => {
let _ = { let assigned = ((strtoul(p, core::ptr::null_mut::<*mut c_char>(), (0 as i32))) as i32); cc = assigned; assigned };
vm_block = 42; continue;
}
// C line 71
44 => {
vm_block = if ((((((((!(p).is_null()) && (((((((*(p)) as i32)) != ((59 as i32))) as i32)) != 0)) as i32)) != 0) && (((((((*(p)) as i32)) != ((0 as i32))) as i32)) != 0)) as i32)) != 0 { 43 } else { 39 }; continue;
}
// C line 70
45 => {
let _ = { let assigned = get_field((line).as_mut_ptr(), (3 as i32)); p = assigned; assigned };
vm_block = 44; continue;
}
// C line 67
46 => {
let _ = { let assigned = ((i) as u8); (*(ci)).general_category = assigned; assigned };
vm_block = 45; continue;
}
// C line 65
47 => {
let _ = tool_exit((1 as i32));
vm_block = 46; continue;
}
// C line 63
48 => {
let _ = tool_fprintf(tool_stderr, c"General category '%s' not found\n".as_ptr(), &[ToolPrintArg::Str((buf1).as_mut_ptr() as *const c_char)]);
vm_block = 47; continue;
}
// C line 62
49 => {
vm_block = if ((((i) < ((0 as i32))) as i32)) != 0 { 48 } else { 46 }; continue;
}
// C line 61
50 => {
let _ = { let assigned = find_name((unicode_gc_name).as_mut_ptr(), (((((size_of::<[*const c_char; 38]>() as usize)) / ((size_of::<*const c_char>() as usize)))) as i32), (buf1).as_mut_ptr()); i = assigned; assigned };
vm_block = 49; continue;
}
// C line 60
51 => {
let _ = get_field_buf((buf1).as_mut_ptr(), (size_of::<[c_char; 256]>() as usize), (line).as_mut_ptr(), (2 as i32));
vm_block = 50; continue;
}
// C line 54
52 => {
let _ = { let assigned = lc; *(((*(ci)).l_data).as_mut_ptr()).offset(((0 as i32)) as isize) = assigned; assigned };
vm_block = 51; continue;
}
// C line 53
53 => {
let _ = { let assigned = (((1 as i32)) as u8); (*(ci)).l_len = assigned; assigned };
vm_block = 52; continue;
}
// C line 52
54 => {
let _ = if ((((!((((((((*(ci)).l_len) as i32)) == ((0 as i32))) as i32)) != 0) as i32)) as i64)) != 0 { tool_assert_rtn(c"parse_unicode_data_dormant".as_ptr(), c"dormant-block.c".as_ptr(), (52 as i32), c"ci->l_len == 0".as_ptr()) } else { { let _ = (0 as i32); } };
vm_block = 53; continue;
}
// C line 51
55 => {
vm_block = if ((((lc) > ((0 as i32))) as i32)) != 0 { 54 } else { 51 }; continue;
}
// C line 49
56 => {
let _ = { let assigned = uc; *(((*(ci)).u_data).as_mut_ptr()).offset(((0 as i32)) as isize) = assigned; assigned };
vm_block = 55; continue;
}
// C line 48
57 => {
let _ = { let assigned = (((1 as i32)) as u8); (*(ci)).u_len = assigned; assigned };
vm_block = 56; continue;
}
// C line 47
58 => {
let _ = if ((((!((((((((*(ci)).u_len) as i32)) == ((0 as i32))) as i32)) != 0) as i32)) as i64)) != 0 { tool_assert_rtn(c"parse_unicode_data_dormant".as_ptr(), c"dormant-block.c".as_ptr(), (47 as i32), c"ci->u_len == 0".as_ptr()) } else { { let _ = (0 as i32); } };
vm_block = 57; continue;
}
// C line 46
59 => {
vm_block = if ((((uc) > ((0 as i32))) as i32)) != 0 { 58 } else { 55 }; continue;
}
// C line 45
60 => {
let _ = if ((((!(((((code) <= ((1114111 as i32))) as i32)) != 0) as i32)) as i64)) != 0 { tool_assert_rtn(c"parse_unicode_data_dormant".as_ptr(), c"dormant-block.c".as_ptr(), (45 as i32), c"code <= CHARCODE_MAX".as_ptr()) } else { { let _ = (0 as i32); } };
vm_block = 59; continue;
}
// C line 44
61 => {
vm_block = if ((((((((uc) > ((0 as i32))) as i32)) != 0) || (((((lc) > ((0 as i32))) as i32)) != 0)) as i32)) != 0 { 60 } else { 51 }; continue;
}
// C line 43
62 => {
let _ = { let assigned = ptr::addr_of_mut!(*(tab).offset((code) as isize)); ci = assigned; assigned };
vm_block = 61; continue;
}
// C line 41
63 => {
let _ = { let assigned = ((strtoul(p, core::ptr::null_mut::<*mut c_char>(), (16 as i32))) as i32); lc = assigned; assigned };
vm_block = 62; continue;
}
// C line 40
64 => {
vm_block = if ((((!(p).is_null()) && (((((((*(p)) as i32)) != ((59 as i32))) as i32)) != 0)) as i32)) != 0 { 63 } else { 62 }; continue;
}
// C line 39
65 => {
let _ = { let assigned = get_field((line).as_mut_ptr(), (13 as i32)); p = assigned; assigned };
vm_block = 64; continue;
}
// C line 36
66 => {
let _ = { let assigned = ((strtoul(p, core::ptr::null_mut::<*mut c_char>(), (16 as i32))) as i32); uc = assigned; assigned };
vm_block = 65; continue;
}
// C line 35
67 => {
vm_block = if ((((!(p).is_null()) && (((((((*(p)) as i32)) != ((59 as i32))) as i32)) != 0)) as i32)) != 0 { 66 } else { 65 }; continue;
}
// C line 34
68 => {
let _ = { let assigned = get_field((line).as_mut_ptr(), (12 as i32)); p = assigned; assigned };
vm_block = 67; continue;
}
// C line 32
69 => {
let _ = { let assigned = (0 as i32); uc = assigned; assigned };
vm_block = 68; continue;
}
// C line 31
70 => {
let _ = { let assigned = (0 as i32); lc = assigned; assigned };
vm_block = 69; continue;
}
// C line 30
71 => {
let _ = { let assigned = ((strtoul(p, core::ptr::null_mut::<*mut c_char>(), (16 as i32))) as i32); code = assigned; assigned };
vm_block = 70; continue;
}
// C line 29
72 => {
vm_block = 2; continue;
}
// C line 28
73 => {
vm_block = if ((!(!(p).is_null()) as i32)) != 0 { 72 } else { 71 }; continue;
}
// C line 27
74 => {
let _ = { let assigned = get_field((line).as_mut_ptr(), (0 as i32)); p = assigned; assigned };
vm_block = 73; continue;
}
// C line 25
75 => {
vm_block = 2; continue;
}
// C line 24
76 => {
vm_block = if ((((((*(p)) as i32)) == ((35 as i32))) as i32)) != 0 { 75 } else { 74 }; continue;
}
// C line 22
77 => {
vm_block = if (isspace(((*(p)) as i32))) != 0 { 78 } else { 76 }; continue;
}
// C line 23
78 => {
let _ = { let old = p; p = (p).offset(1); old };
vm_block = 77; continue;
}
// C line 21
79 => {
let _ = { let assigned = (line).as_mut_ptr(); p = assigned; assigned };
vm_block = 77; continue;
}
// C line 20
80 => {
vm_block = 1; continue;
}
// C line 19
81 => {
vm_block = if ((!(!(get_line((line).as_mut_ptr(), (((size_of::<[c_char; 1024]>() as usize)) as i32), f)).is_null()) as i32)) != 0 { 80 } else { 79 }; continue;
}
// C line 17
82 => {
let _ = { let assigned = (0 as i32); last_code = assigned; assigned };
vm_block = 2; continue;
}
// C line 14
83 => {
let _ = tool_exit((1 as i32));
vm_block = 82; continue;
}
// C line 13
84 => {
let _ = perror(filename);
vm_block = 83; continue;
}
// C line 12
85 => {
vm_block = if ((!(!(f).is_null()) as i32)) != 0 { 84 } else { 82 }; continue;
}
// C line 11
86 => {
let _ = { let assigned = fopen(filename, c"rb".as_ptr()); f = assigned; assigned };
vm_block = 85; continue;
}
// C line 9
87 => {
tab = unicode_db;
vm_block = 86; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: unicode_gen.c:1555 original #if 0 block retained in unused copy. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn build_conv_table_dormant(mut tab: *mut CCInfo, mut RUN_TYPE_TODO: i32) -> () {
let mut vm_local_storage = Vec::<u64>::new();
let mut code: i32 = core::mem::zeroed();
let mut i: i32 = core::mem::zeroed();
let mut j: i32 = core::mem::zeroed();
let mut ci: *mut CCInfo = core::mem::zeroed();
let mut te: *mut TableEntry = core::mem::zeroed();
let mut data_index: i32 = core::mem::zeroed();
let mut p: i32 = core::mem::zeroed();
let mut v: i32 = core::mem::zeroed();
let mut p_1: i32 = core::mem::zeroed();
let mut v_1: i32 = core::mem::zeroed();
let mut vm_block: usize = 70;
loop { match vm_block {
// C line ?
0 => {
return ();
}
// C line 95
1 => {
vm_block = if ((((i) < (conv_table_len)) as i32)) != 0 { 5 } else { 0 }; continue;
}
// C line ?
2 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 1; continue;
}
// C line 100
3 => {
let _ = { let assigned = find_ext_data_index(*(((*(te)).ext_data).as_mut_ptr()).offset(((0 as i32)) as isize)); (*(te)).data_index = assigned; assigned };
vm_block = 2; continue;
}
// C line 97
4 => {
vm_block = if (((((((((((((*(te)).v_type) == ((RUN_TYPE_UF_D1_EXT as i32))) as i32)) != 0) || ((((((*(te)).v_type) == ((RUN_TYPE_U_EXT as i32))) as i32)) != 0)) as i32)) != 0) || ((((((*(te)).v_type) == ((RUN_TYPE_LF_EXT as i32))) as i32)) != 0)) as i32)) != 0 { 3 } else { 2 }; continue;
}
// C line 96
5 => {
let _ = { let assigned = ptr::addr_of_mut!(*((conv_table).as_mut_ptr()).offset((i) as isize)); te = assigned; assigned };
vm_block = 4; continue;
}
// C line 95
6 => {
let _ = { let assigned = (0 as i32); i = assigned; assigned };
vm_block = 1; continue;
}
// C line 79
7 => {
vm_block = if ((((i) < (conv_table_len)) as i32)) != 0 { 18 } else { 6 }; continue;
}
// C line ?
8 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 7; continue;
}
// C line 91
9 => {
let _ = { let assigned = v_1; (*(te)).data_index = assigned; assigned };
vm_block = 8; continue;
}
// C line 86
10 => {
vm_block = if ((((j) < ((2 as i32))) as i32)) != 0 { 14 } else { 9 }; continue;
}
// C line ?
11 => {
let _ = { let old = j; j = (j).wrapping_add(1); old };
vm_block = 10; continue;
}
// C line 89
12 => {
let _ = { let assigned = (((v_1).wrapping_shl(((6 as i32)) as u32)) | (p_1)); v_1 = assigned; assigned };
vm_block = 11; continue;
}
// C line 88
13 => {
let _ = if ((((!(((((p_1) < ((64 as i32))) as i32)) != 0) as i32)) as i64)) != 0 { tool_assert_rtn(c"build_conv_table_dormant".as_ptr(), c"dormant-block.c".as_ptr(), (88 as i32), c"p < 64".as_ptr()) } else { { let _ = (0 as i32); } };
vm_block = 12; continue;
}
// C line 87
14 => {
let _ = { let assigned = find_ext_data_index(*(((*(te)).ext_data).as_mut_ptr()).offset((j) as isize)); p_1 = assigned; assigned };
vm_block = 13; continue;
}
// C line 86
15 => {
let _ = { let assigned = (0 as i32); j = assigned; assigned };
vm_block = 10; continue;
}
// C line 85
16 => {
let _ = { let assigned = (0 as i32); v_1 = assigned; assigned };
vm_block = 15; continue;
}
// C line 81
17 => {
vm_block = if (((((((((((((*(te)).v_type) == ((RUN_TYPE_LF_EXT2 as i32))) as i32)) != 0) || ((((((*(te)).v_type) == ((RUN_TYPE_UF_EXT2 as i32))) as i32)) != 0)) as i32)) != 0) || ((((((*(te)).v_type) == ((RUN_TYPE_U2L_399_EXT2 as i32))) as i32)) != 0)) as i32)) != 0 { 16 } else { 8 }; continue;
}
// C line 80
18 => {
let _ = { let assigned = ptr::addr_of_mut!(*((conv_table).as_mut_ptr()).offset((i) as isize)); te = assigned; assigned };
vm_block = 17; continue;
}
// C line 79
19 => {
let _ = { let assigned = (0 as i32); i = assigned; assigned };
vm_block = 7; continue;
}
// C line 65
20 => {
vm_block = if ((((i) < (conv_table_len)) as i32)) != 0 { 31 } else { 19 }; continue;
}
// C line ?
21 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 20; continue;
}
// C line 75
22 => {
let _ = { let assigned = v; (*(te)).data_index = assigned; assigned };
vm_block = 21; continue;
}
// C line 70
23 => {
vm_block = if ((((j) < ((3 as i32))) as i32)) != 0 { 27 } else { 22 }; continue;
}
// C line ?
24 => {
let _ = { let old = j; j = (j).wrapping_add(1); old };
vm_block = 23; continue;
}
// C line 73
25 => {
let _ = { let assigned = (((v).wrapping_shl(((4 as i32)) as u32)) | (p)); v = assigned; assigned };
vm_block = 24; continue;
}
// C line 72
26 => {
let _ = if ((((!(((((p) < ((16 as i32))) as i32)) != 0) as i32)) as i64)) != 0 { tool_assert_rtn(c"build_conv_table_dormant".as_ptr(), c"dormant-block.c".as_ptr(), (72 as i32), c"p < 16".as_ptr()) } else { { let _ = (0 as i32); } };
vm_block = 25; continue;
}
// C line 71
27 => {
let _ = { let assigned = find_ext_data_index(*(((*(te)).ext_data).as_mut_ptr()).offset((j) as isize)); p = assigned; assigned };
vm_block = 26; continue;
}
// C line 70
28 => {
let _ = { let assigned = (0 as i32); j = assigned; assigned };
vm_block = 23; continue;
}
// C line 69
29 => {
let _ = { let assigned = (0 as i32); v = assigned; assigned };
vm_block = 28; continue;
}
// C line 67
30 => {
vm_block = if (((((*(te)).v_type) == ((RUN_TYPE_UF_EXT3 as i32))) as i32)) != 0 { 29 } else { 21 }; continue;
}
// C line 66
31 => {
let _ = { let assigned = ptr::addr_of_mut!(*((conv_table).as_mut_ptr()).offset((i) as isize)); te = assigned; assigned };
vm_block = 30; continue;
}
// C line 65
32 => {
let _ = { let assigned = (0 as i32); i = assigned; assigned };
vm_block = 20; continue;
}
// C line 28
33 => {
vm_block = if ((((i) < (conv_table_len)) as i32)) != 0 { 53 } else { 32 }; continue;
}
// C line ?
34 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 33; continue;
}
// C line 60
35 => {
vm_block = 34; continue;
}
// C line 59
36 => {
let _ = { let assigned = (*(te)).data; (*(te)).data_index = assigned; assigned };
vm_block = 35; continue;
}
// C line 57
37 => {
vm_block = 34; continue;
}
// C line 52
38 => {
let _ = tool_exit((1 as i32));
vm_block = 37; continue;
}
// C line ?
39 => {
let _ = tool_printf(c"%05x: index not found\n".as_ptr(), &[ToolPrintArg::Int((*(te)).code as u64)]);
vm_block = 38; continue;
}
// C line 49
40 => {
vm_block = 37; continue;
}
// C line 48
41 => {
let _ = { let assigned = (*(te)).data; *(((*(te)).ext_data).as_mut_ptr()).offset(((0 as i32)) as isize) = assigned; assigned };
vm_block = 40; continue;
}
// C line 47
42 => {
let _ = { let assigned = (1 as i32); (*(te)).ext_len = assigned; assigned };
vm_block = 41; continue;
}
// C line 46
43 => {
let _ = { let assigned = (RUN_TYPE_LF_EXT as i32); (*(te)).v_type = assigned; assigned };
vm_block = 42; continue;
}
// C line 44
44 => {
vm_block = 37; continue;
}
// C line 43
45 => {
let _ = { let assigned = (*(te)).data; *(((*(te)).ext_data).as_mut_ptr()).offset(((0 as i32)) as isize) = assigned; assigned };
vm_block = 44; continue;
}
// C line 42
46 => {
let _ = { let assigned = (1 as i32); (*(te)).ext_len = assigned; assigned };
vm_block = 45; continue;
}
// C line 41
47 => {
let _ = { let assigned = (RUN_TYPE_U_EXT as i32); (*(te)).v_type = assigned; assigned };
vm_block = 46; continue;
}
// C line 39
48 => {
vm_block = match (*(te)).v_type { x if x == (RUN_TYPE_LF as i32) => 43, x if x == (RUN_TYPE_U as i32) => 47, _ => 39, }; continue;
}
// C line 55
49 => {
let _ = { let assigned = data_index; (*(te)).data_index = assigned; assigned };
vm_block = 37; continue;
}
// C line 38
50 => {
vm_block = if ((((data_index) < ((0 as i32))) as i32)) != 0 { 48 } else { 49 }; continue;
}
// C line 37
51 => {
let _ = { let assigned = find_data_index((conv_table).as_mut_ptr(), conv_table_len, (*(te)).data); data_index = assigned; assigned };
vm_block = 50; continue;
}
// C line 32
52 => {
vm_block = match (*(te)).v_type { x if x == (RUN_TYPE_UF_D20 as i32) => 36, x if x == (RUN_TYPE_LF as i32) => 51, x if x == (RUN_TYPE_UF as i32) => 51, x if x == (RUN_TYPE_L as i32) => 51, x if x == (RUN_TYPE_U as i32) => 51, _ => 34, }; continue;
}
// C line 30
53 => {
let _ = { let assigned = ptr::addr_of_mut!(*((conv_table).as_mut_ptr()).offset((i) as isize)); te = assigned; assigned };
vm_block = 52; continue;
}
// C line 28
54 => {
let _ = { let assigned = (0 as i32); i = assigned; assigned };
vm_block = 33; continue;
}
// C line 25
55 => {
let _ = { let assigned = ((((te).offset_from((conv_table).as_mut_ptr()) as i64)) as i32); conv_table_len = assigned; assigned };
vm_block = 54; continue;
}
// C line 9
56 => {
vm_block = if ((((code) <= ((1114111 as i32))) as i32)) != 0 { 68 } else { 55 }; continue;
}
// C line ?
57 => {
let _ = { let old = code; code = (code).wrapping_add(1); old };
vm_block = 56; continue;
}
// C line 23
58 => {
let _ = { let old = te; te = (te).offset(1); old };
vm_block = 57; continue;
}
// C line 22
59 => {
let _ = { code = (code).wrapping_add(((*(te)).len).wrapping_sub((1 as i32))); code };
vm_block = 58; continue;
}
// C line 21
60 => {
let _ = if ((((!((((((*(te)).len) <= ((127 as i32))) as i32)) != 0) as i32)) as i64)) != 0 { tool_assert_rtn(c"build_conv_table_dormant".as_ptr(), c"dormant-block.c".as_ptr(), (21 as i32), c"te->len <= 127".as_ptr()) } else { { let _ = (0 as i32); } };
vm_block = 59; continue;
}
// C line 18
61 => {
let _ = dump_cc_info(ci, code);
vm_block = 60; continue;
}
// C line 17
62 => {
let _ = tool_printf(c"TODO: ".as_ptr(), &[]);
vm_block = 61; continue;
}
// C line 16
63 => {
vm_block = if (((((*(te)).v_type) == (RUN_TYPE_TODO)) as i32)) != 0 { 62 } else { 60 }; continue;
}
// C line 14
64 => {
let _ = find_run_type(te, tab, code);
vm_block = 63; continue;
}
// C line 13
65 => {
let _ = if ((((!(((((((((te).offset_from((conv_table).as_mut_ptr()) as i64)) as usize)) < ((((size_of::<[TableEntry; 1000]>() as usize)) / ((size_of::<TableEntry>() as usize))))) as i32)) != 0) as i32)) as i64)) != 0 { tool_assert_rtn(c"build_conv_table_dormant".as_ptr(), c"dormant-block.c".as_ptr(), (13 as i32), c"te - conv_table < countof(conv_table)".as_ptr()) } else { { let _ = (0 as i32); } };
vm_block = 64; continue;
}
// C line 12
66 => {
vm_block = 57; continue;
}
// C line 11
67 => {
vm_block = if (((((((((((((((*(ci)).u_len) as i32)) == ((0 as i32))) as i32)) != 0) && ((((((((*(ci)).l_len) as i32)) == ((0 as i32))) as i32)) != 0)) as i32)) != 0) && ((((((((*(ci)).f_len) as i32)) == ((0 as i32))) as i32)) != 0)) as i32)) != 0 { 66 } else { 65 }; continue;
}
// C line 10
68 => {
let _ = { let assigned = ptr::addr_of_mut!(*(tab).offset((code) as isize)); ci = assigned; assigned };
vm_block = 67; continue;
}
// C line 9
69 => {
let _ = { let assigned = (0 as i32); code = assigned; assigned };
vm_block = 56; continue;
}
// C line 8
70 => {
let _ = { let assigned = (conv_table).as_mut_ptr(); te = assigned; assigned };
vm_block = 69; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: unicode_gen.c:1771 original #if 0 block retained in unused copy. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn compute_internal_props_dormant() -> () {
let mut vm_local_storage = Vec::<u64>::new();
let mut i: i32 = core::mem::zeroed();
let mut has_ul: i32 = core::mem::zeroed();
let mut ci: *mut CCInfo = core::mem::zeroed();
let mut b: i32 = core::mem::zeroed();
let mut vm_block: usize = 16;
loop { match vm_block {
// C line ?
0 => {
return ();
}
// C line 7
1 => {
vm_block = if ((((i) <= ((1114111 as i32))) as i32)) != 0 { 15 } else { 0 }; continue;
}
// C line ?
2 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 1; continue;
}
// C line 35
3 => {
let _ = set_prop(((i) as u32), (PROP_Cased1 as i32), ((get_prop(((i) as u32), (PROP_Case_Ignorable as i32))) ^ (b)));
vm_block = 2; continue;
}
// C line 33
4 => {
let _ = { let assigned = (((((((((((((1 as u32)).wrapping_shl(((GCAT_Mn as i32)) as u32)) | (((1 as u32)).wrapping_shl(((GCAT_Cf as i32)) as u32)))) | (((1 as u32)).wrapping_shl(((GCAT_Lm as i32)) as u32)))) | (((1 as u32)).wrapping_shl(((GCAT_Sk as i32)) as u32)))).wrapping_shr(((((*(unicode_db).offset((i) as isize)).general_category) as i32)) as u32)) & ((((1 as i32)) as u32)))) as i32); b = assigned; assigned };
vm_block = 3; continue;
}
// C line 26
5 => {
let _ = set_prop(((i) as u32), (PROP_Changes_When_NFKC_Casefolded1 as i32), ((get_prop(((i) as u32), (PROP_Changes_When_NFKC_Casefolded as i32))) ^ (((((((*(ci)).f_len) as i32)) != ((0 as i32))) as i32))));
vm_block = 4; continue;
}
// C line 23
6 => {
let _ = set_prop(((i) as u32), (PROP_Changes_When_Casefolded1 as i32), ((get_prop(((i) as u32), (PROP_Changes_When_Casefolded as i32))) ^ (((((((*(ci)).f_len) as i32)) != ((0 as i32))) as i32))));
vm_block = 5; continue;
}
// C line 21
7 => {
let _ = set_prop(((i) as u32), (PROP_Changes_When_Titlecased1 as i32), ((get_prop(((i) as u32), (PROP_Changes_When_Titlecased as i32))) ^ (((((((*(ci)).u_len) as i32)) != ((0 as i32))) as i32))));
vm_block = 6; continue;
}
// C line 19
8 => {
let _ = set_prop(((i) as u32), (PROP_XID_Continue1 as i32), ((get_prop(((i) as u32), (PROP_ID_Continue as i32))) ^ (get_prop(((i) as u32), (PROP_XID_Continue as i32)))));
vm_block = 7; continue;
}
// C line 17
9 => {
let _ = set_prop(((i) as u32), (PROP_XID_Start1 as i32), ((get_prop(((i) as u32), (PROP_ID_Start as i32))) ^ (get_prop(((i) as u32), (PROP_XID_Start as i32)))));
vm_block = 8; continue;
}
// C line 15
10 => {
let _ = set_prop(((i) as u32), (PROP_ID_Continue1 as i32), ((get_prop(((i) as u32), (PROP_ID_Continue as i32))) & (((get_prop(((i) as u32), (PROP_ID_Start as i32))) ^ ((1 as i32))))));
vm_block = 9; continue;
}
// C line 11
11 => {
let _ = if ((((!((get_prop(((i) as u32), (PROP_Cased as i32))) != 0) as i32)) as i64)) != 0 { tool_assert_rtn(c"compute_internal_props_dormant".as_ptr(), c"dormant-block.c".as_ptr(), (11 as i32), c"get_prop(i, PROP_Cased)".as_ptr()) } else { { let _ = (0 as i32); } };
vm_block = 10; continue;
}
// C line 13
12 => {
let _ = set_prop(((i) as u32), (PROP_Cased1 as i32), get_prop(((i) as u32), (PROP_Cased as i32)));
vm_block = 10; continue;
}
// C line 10
13 => {
vm_block = if (has_ul) != 0 { 11 } else { 12 }; continue;
}
// C line 9
14 => {
let _ = { let assigned = ((((((((((((((*(ci)).u_len) as i32)) != ((0 as i32))) as i32)) != 0) || ((((((((*(ci)).l_len) as i32)) != ((0 as i32))) as i32)) != 0)) as i32)) != 0) || ((((((((*(ci)).f_len) as i32)) != ((0 as i32))) as i32)) != 0)) as i32); has_ul = assigned; assigned };
vm_block = 13; continue;
}
// C line 8
15 => {
ci = ptr::addr_of_mut!(*(unicode_db).offset((i) as isize));
vm_block = 14; continue;
}
// C line 7
16 => {
let _ = { let assigned = (0 as i32); i = assigned; assigned };
vm_block = 1; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: unicode_gen.c:2255 original #if 0 block retained in unused copy. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn mark_zwj_string_dormant(mut sl: *mut REStringList, mut buf: *mut u32, mut len: i32, mut mod_type: i32, mut mod_pos: *mut i32, mut hc_pos: i32, mut mark_flag: i32) -> i32 {
let mut vm_local_storage = Vec::<u64>::new();
let mut p: *mut REString = core::mem::zeroed();
let mut i: i32 = core::mem::zeroed();
let mut n_mod: i32 = core::mem::zeroed();
let mut i0: i32 = core::mem::zeroed();
let mut i1: i32 = core::mem::zeroed();
let mut hc_count: i32 = core::mem::zeroed();
let mut j: i32 = core::mem::zeroed();
let mut vm_block: usize = 41;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 66
1 => {
return (TRUE as i32);
}
// C line 34
2 => {
vm_block = if ((((j) < (hc_count)) as i32)) != 0 { 25 } else { 1 }; continue;
}
// C line ?
3 => {
let _ = { let old = j; j = (j).wrapping_add(1); old };
vm_block = 2; continue;
}
// C line 35
4 => {
vm_block = if ((((i) < (n_mod)) as i32)) != 0 { 24 } else { 3 }; continue;
}
// C line ?
5 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 4; continue;
}
// C line 63
6 => {
let _ = { (*(p)).flags = (((*(p)).flags) | ((((1 as i32)) as u32))); (*(p)).flags };
vm_block = 5; continue;
}
// C line 62
7 => {
vm_block = if (mark_flag) != 0 { 6 } else { 5 }; continue;
}
// C line 61
8 => {
return (FALSE as i32);
}
// C line 60
9 => {
vm_block = if ((!(!(p).is_null()) as i32)) != 0 { 8 } else { 7 }; continue;
}
// C line 59
10 => {
let _ = { let assigned = re_string_find(sl, len, buf, (FALSE as i32)); p = assigned; assigned };
vm_block = 9; continue;
}
// C line 57
11 => {
let _ = { let assigned = ((((129456 as i32)).wrapping_add(j)) as u32); *(buf).offset((hc_pos) as isize) = assigned; assigned };
vm_block = 10; continue;
}
// C line 56
12 => {
vm_block = if ((((hc_pos) >= ((0 as i32))) as i32)) != 0 { 11 } else { 10 }; continue;
}
// C line 53
13 => {
let _ = if ((((!(((0 as i32)) != 0) as i32)) as i64)) != 0 { tool_assert_rtn(c"mark_zwj_string_dormant".as_ptr(), c"dormant-block.c".as_ptr(), (53 as i32), c"0".as_ptr()) } else { { let _ = (0 as i32); } };
vm_block = 12; continue;
}
// C line 51
14 => {
vm_block = 12; continue;
}
// C line 50
15 => {
let _ = { let assigned = ((((127995 as i32)).wrapping_add(i1)) as u32); *(buf).offset((*(mod_pos).offset(((1 as i32)) as isize)) as isize) = assigned; assigned };
vm_block = 14; continue;
}
// C line 49
16 => {
let _ = { let assigned = ((((127995 as i32)).wrapping_add(i0)) as u32); *(buf).offset((*(mod_pos).offset(((0 as i32)) as isize)) as isize) = assigned; assigned };
vm_block = 15; continue;
}
// C line 48
17 => {
let _ = { let old = i0; i0 = (i0).wrapping_add(1); old };
vm_block = 16; continue;
}
// C line 47
18 => {
vm_block = if ((((((((mod_type) == ((3 as i32))) as i32)) != 0) && (((((i0) >= (i1)) as i32)) != 0)) as i32)) != 0 { 17 } else { 16 }; continue;
}
// C line 45
19 => {
let _ = { let assigned = ((i) % ((5 as i32))); i1 = assigned; assigned };
vm_block = 18; continue;
}
// C line 44
20 => {
let _ = { let assigned = ((i) / ((5 as i32))); i0 = assigned; assigned };
vm_block = 19; continue;
}
// C line 41
21 => {
vm_block = 12; continue;
}
// C line 40
22 => {
let _ = { let assigned = ((((127995 as i32)).wrapping_add(i)) as u32); *(buf).offset((*(mod_pos).offset(((0 as i32)) as isize)) as isize) = assigned; assigned };
vm_block = 21; continue;
}
// C line 38
23 => {
vm_block = 12; continue;
}
// C line 36
24 => {
vm_block = match mod_type { x if x == (3 as i32) => 20, x if x == (2 as i32) => 20, x if x == (1 as i32) => 22, x if x == (0 as i32) => 23, _ => 13, }; continue;
}
// C line 35
25 => {
let _ = { let assigned = (0 as i32); i = assigned; assigned };
vm_block = 4; continue;
}
// C line 34
26 => {
let _ = { let assigned = (0 as i32); j = assigned; assigned };
vm_block = 2; continue;
}
// C line 30
27 => {
let _ = { let assigned = (4 as i32); hc_count = assigned; assigned };
vm_block = 26; continue;
}
// C line 32
28 => {
let _ = { let assigned = (1 as i32); hc_count = assigned; assigned };
vm_block = 26; continue;
}
// C line 29
29 => {
vm_block = if ((((hc_pos) >= ((0 as i32))) as i32)) != 0 { 27 } else { 28 }; continue;
}
// C line 27
30 => {
let _ = if ((((!(((0 as i32)) != 0) as i32)) as i64)) != 0 { tool_assert_rtn(c"mark_zwj_string_dormant".as_ptr(), c"dormant-block.c".as_ptr(), (27 as i32), c"0".as_ptr()) } else { { let _ = (0 as i32); } };
vm_block = 29; continue;
}
// C line 25
31 => {
vm_block = 29; continue;
}
// C line 24
32 => {
let _ = { let assigned = (20 as i32); n_mod = assigned; assigned };
vm_block = 31; continue;
}
// C line 22
33 => {
vm_block = 29; continue;
}
// C line 21
34 => {
let _ = { let assigned = (25 as i32); n_mod = assigned; assigned };
vm_block = 33; continue;
}
// C line 19
35 => {
vm_block = 29; continue;
}
// C line 18
36 => {
let _ = { let assigned = (5 as i32); n_mod = assigned; assigned };
vm_block = 35; continue;
}
// C line 16
37 => {
vm_block = 29; continue;
}
// C line 15
38 => {
let _ = { let assigned = (1 as i32); n_mod = assigned; assigned };
vm_block = 37; continue;
}
// C line 13
39 => {
vm_block = match mod_type { x if x == (3 as i32) => 32, x if x == (2 as i32) => 34, x if x == (1 as i32) => 36, x if x == (0 as i32) => 38, _ => 30, }; continue;
}
// C line 10
40 => {
let _ = tool_printf(c"mod_type=%d\n".as_ptr(), &[ToolPrintArg::Int(mod_type as u64)]);
vm_block = 39; continue;
}
// C line 9
41 => {
vm_block = if (mark_flag) != 0 { 40 } else { 39 }; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: unicode_gen.c:2363 original #if 0 block retained in unused copy. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn build_rgi_emoji_zwj_sequence_dormant(mut f: *mut FILE, mut sl: *mut REStringList) -> () {
let mut vm_local_storage = Vec::<u64>::new();
let mut mod_pos: [i32; 2] = core::mem::zeroed();
let mut mod_count: i32 = core::mem::zeroed();
let mut hair_color_pos: i32 = core::mem::zeroed();
let mut j: i32 = core::mem::zeroed();
let mut h: i32 = core::mem::zeroed();
let mut p: *mut REString = core::mem::zeroed();
let mut buf: [u32; 16] = core::mem::zeroed();
let mut dbuf: DynBuf = core::mem::zeroed();
let mut mod_type: i32 = core::mem::zeroed();
let mut vm_block: usize = 52;
loop { match vm_block {
// C line ?
0 => {
return ();
}
// C line 76
1 => {
let _ = dbuf_free(ptr::addr_of_mut!(dbuf));
vm_block = 0; continue;
}
// C line 74
2 => {
let _ = dump_byte_table(f, c"unicode_rgi_emoji_zwj_sequence".as_ptr(), (dbuf).buf, (((dbuf).size) as i32));
vm_block = 1; continue;
}
// C line 26
3 => {
vm_block = if ((((((h) as u32)) < ((*(sl)).hash_size)) as i32)) != 0 { 38 } else { 2 }; continue;
}
// C line ?
4 => {
let _ = { let old = h; h = (h).wrapping_add(1); old };
vm_block = 3; continue;
}
// C line 27
5 => {
vm_block = if ((((p) != (core::ptr::null_mut::<REString>())) as i32)) != 0 { 37 } else { 4 }; continue;
}
// C line ?
6 => {
let _ = { let assigned = (*(p)).next; p = assigned; assigned };
vm_block = 5; continue;
}
// C line 65
7 => {
let _ = zwj_encode_string(ptr::addr_of_mut!(dbuf), (buf).as_mut_ptr(), (((*(p)).len) as i32), mod_type, (mod_pos).as_mut_ptr(), hair_color_pos);
vm_block = 6; continue;
}
// C line 63
8 => {
let _ = { let assigned = (((129456 as i32)) as u32); *((buf).as_mut_ptr()).offset((hair_color_pos) as isize) = assigned; assigned };
vm_block = 7; continue;
}
// C line 62
9 => {
vm_block = if ((((hair_color_pos) >= ((0 as i32))) as i32)) != 0 { 8 } else { 7 }; continue;
}
// C line 52
10 => {
let _ = mark_zwj_string(sl, (buf).as_mut_ptr(), (((*(p)).len) as i32), mod_type, (mod_pos).as_mut_ptr(), hair_color_pos, (TRUE as i32));
vm_block = 9; continue;
}
// C line 56
11 => {
let _ = mark_zwj_string(sl, (buf).as_mut_ptr(), (((*(p)).len) as i32), mod_type, (mod_pos).as_mut_ptr(), hair_color_pos, (TRUE as i32));
vm_block = 9; continue;
}
// C line 59
12 => {
vm_block = 23; continue;
}
// C line 58
13 => {
let _ = dump_str(c"not_found".as_ptr(), ((((*(p)).buf).as_mut_ptr()) as *mut i32), (((*(p)).len) as i32));
vm_block = 12; continue;
}
// C line 55
14 => {
vm_block = if (mark_zwj_string(sl, (buf).as_mut_ptr(), (((*(p)).len) as i32), mod_type, (mod_pos).as_mut_ptr(), hair_color_pos, (FALSE as i32))) != 0 { 11 } else { 13 }; continue;
}
// C line 54
15 => {
let _ = { let assigned = (3 as i32); mod_type = assigned; assigned };
vm_block = 14; continue;
}
// C line 53
16 => {
vm_block = if ((((mod_type) == ((2 as i32))) as i32)) != 0 { 15 } else { 9 }; continue;
}
// C line 51
17 => {
vm_block = if (mark_zwj_string(sl, (buf).as_mut_ptr(), (((*(p)).len) as i32), mod_type, (mod_pos).as_mut_ptr(), hair_color_pos, (FALSE as i32))) != 0 { 10 } else { 16 }; continue;
}
// C line 45
18 => {
let _ = { let assigned = (0 as i32); mod_type = assigned; assigned };
vm_block = 17; continue;
}
// C line 47
19 => {
let _ = { let assigned = (1 as i32); mod_type = assigned; assigned };
vm_block = 17; continue;
}
// C line 49
20 => {
let _ = { let assigned = (2 as i32); mod_type = assigned; assigned };
vm_block = 17; continue;
}
// C line 46
21 => {
vm_block = if ((((mod_count) == ((1 as i32))) as i32)) != 0 { 19 } else { 20 }; continue;
}
// C line 44
22 => {
vm_block = if ((((mod_count) == ((0 as i32))) as i32)) != 0 { 18 } else { 21 }; continue;
}
// C line ? labels: keep
23 => {
let _ = zwj_encode_string(ptr::addr_of_mut!(dbuf), (buf).as_mut_ptr(), (((*(p)).len) as i32), (0 as i32), core::ptr::null_mut::<i32>(), ((1 as i32)).wrapping_neg());
vm_block = 6; continue;
}
// C line 42
24 => {
vm_block = if ((((((((mod_count) != ((0 as i32))) as i32)) != 0) || (((((hair_color_pos) >= ((0 as i32))) as i32)) != 0)) as i32)) != 0 { 22 } else { 23 }; continue;
}
// C line 32
25 => {
vm_block = if ((((((j) as u32)) < ((*(p)).len)) as i32)) != 0 { 32 } else { 24 }; continue;
}
// C line ?
26 => {
let _ = { let old = j; j = (j).wrapping_add(1); old };
vm_block = 25; continue;
}
// C line 39
27 => {
let _ = { let assigned = *(((*(p)).buf).as_mut_ptr()).offset((j) as isize); *((buf).as_mut_ptr()).offset((j) as isize) = assigned; assigned };
vm_block = 26; continue;
}
// C line 35
28 => {
let _ = { let assigned = j; *((mod_pos).as_mut_ptr()).offset(({ let old = mod_count; mod_count = (mod_count).wrapping_add(1); old }) as isize) = assigned; assigned };
vm_block = 27; continue;
}
// C line 34
29 => {
let _ = if ((((!(((((mod_count) < ((2 as i32))) as i32)) != 0) as i32)) as i64)) != 0 { tool_assert_rtn(c"build_rgi_emoji_zwj_sequence_dormant".as_ptr(), c"dormant-block.c".as_ptr(), (34 as i32), c"mod_count < 2".as_ptr()) } else { { let _ = (0 as i32); } };
vm_block = 28; continue;
}
// C line 37
30 => {
let _ = { let assigned = j; hair_color_pos = assigned; assigned };
vm_block = 27; continue;
}
// C line 36
31 => {
vm_block = if (is_emoji_hair_color(*(((*(p)).buf).as_mut_ptr()).offset((j) as isize))) != 0 { 30 } else { 27 }; continue;
}
// C line 33
32 => {
vm_block = if (is_emoji_modifier(*(((*(p)).buf).as_mut_ptr()).offset((j) as isize))) != 0 { 29 } else { 31 }; continue;
}
// C line 32
33 => {
let _ = { let assigned = (0 as i32); j = assigned; assigned };
vm_block = 25; continue;
}
// C line 31
34 => {
let _ = { let assigned = ((1 as i32)).wrapping_neg(); hair_color_pos = assigned; assigned };
vm_block = 33; continue;
}
// C line 30
35 => {
let _ = { let assigned = (0 as i32); mod_count = assigned; assigned };
vm_block = 34; continue;
}
// C line 29
36 => {
vm_block = 6; continue;
}
// C line 28
37 => {
vm_block = if ((*(p)).flags) != 0 { 36 } else { 35 }; continue;
}
// C line 27
38 => {
let _ = { let assigned = *((*(sl)).hash_table).offset((h) as isize); p = assigned; assigned };
vm_block = 5; continue;
}
// C line 26
39 => {
let _ = { let assigned = (0 as i32); h = assigned; assigned };
vm_block = 3; continue;
}
// C line 23
40 => {
let _ = dbuf_init(ptr::addr_of_mut!(dbuf));
vm_block = 39; continue;
}
// C line 18
41 => {
let _ = tool_exit((0 as i32));
vm_block = 40; continue;
}
// C line 11
42 => {
vm_block = if ((((((h) as u32)) < ((*(sl)).hash_size)) as i32)) != 0 { 51 } else { 41 }; continue;
}
// C line ?
43 => {
let _ = { let old = h; h = (h).wrapping_add(1); old };
vm_block = 42; continue;
}
// C line 12
44 => {
vm_block = if ((((p) != (core::ptr::null_mut::<REString>())) as i32)) != 0 { 50 } else { 43 }; continue;
}
// C line ?
45 => {
let _ = { let assigned = (*(p)).next; p = assigned; assigned };
vm_block = 44; continue;
}
// C line 15
46 => {
let _ = tool_printf(c"\n".as_ptr(), &[]);
vm_block = 45; continue;
}
// C line 13
47 => {
vm_block = if ((((((j) as u32)) < ((*(p)).len)) as i32)) != 0 { 49 } else { 46 }; continue;
}
// C line ?
48 => {
let _ = { let old = j; j = (j).wrapping_add(1); old };
vm_block = 47; continue;
}
// C line 14
49 => {
let _ = tool_printf(c" %04x".as_ptr(), &[ToolPrintArg::Int(*(((*(p)).buf).as_mut_ptr()).offset((j) as isize) as u64)]);
vm_block = 48; continue;
}
// C line 13
50 => {
let _ = { let assigned = (0 as i32); j = assigned; assigned };
vm_block = 47; continue;
}
// C line 12
51 => {
let _ = { let assigned = *((*(sl)).hash_table).offset((h) as isize); p = assigned; assigned };
vm_block = 44; continue;
}
// C line 11
52 => {
let _ = { let assigned = (0 as i32); h = assigned; assigned };
vm_block = 42; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: unicode_gen.c:3456 original #if 0 block retained in unused copy. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn build_compose_table_dormant(mut f: *mut FILE, mut tab_de: *const DecompEntry) -> () {
let mut vm_local_storage = Vec::<u64>::new();
let mut i: i32 = core::mem::zeroed();
let mut v: i32 = core::mem::zeroed();
let mut tab_ce_len: i32 = core::mem::zeroed();
let mut ce: *mut ComposeEntry = core::mem::zeroed();
let mut tab_ce: *mut ComposeEntry = core::mem::zeroed();
let mut ci: *mut CCInfo = core::mem::zeroed();
let mut vm_block: usize = 34;
loop { match vm_block {
// C line ?
0 => {
return ();
}
// C line 48
1 => {
let _ = free(((tab_ce) as *mut c_void));
vm_block = 0; continue;
}
// C line 46
2 => {
let _ = tool_fprintf(f, c"\n};\n\n".as_ptr(), &[]);
vm_block = 1; continue;
}
// C line 35
3 => {
vm_block = if ((((i) < (tab_ce_len)) as i32)) != 0 { 11 } else { 2 }; continue;
}
// C line ?
4 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 3; continue;
}
// C line 44
5 => {
let _ = tool_fprintf(f, c" 0x%04x,".as_ptr(), &[ToolPrintArg::Int(v as u64)]);
vm_block = 4; continue;
}
// C line 42
6 => {
let _ = tool_exit((1 as i32));
vm_block = 5; continue;
}
// C line 40
7 => {
let _ = tool_printf(c"ERROR: entry for c=%04x not found\n".as_ptr(), &[ToolPrintArg::Int((*(tab_ce).offset((i) as isize)).p as u64)]);
vm_block = 6; continue;
}
// C line 39
8 => {
vm_block = if ((((v) < ((0 as i32))) as i32)) != 0 { 7 } else { 5 }; continue;
}
// C line 38
9 => {
let _ = { let assigned = get_decomp_pos(tab_de, (((*(tab_ce).offset((i) as isize)).p) as i32)); v = assigned; assigned };
vm_block = 8; continue;
}
// C line 37
10 => {
let _ = tool_fprintf(f, c"\n   ".as_ptr(), &[]);
vm_block = 9; continue;
}
// C line 36
11 => {
vm_block = if ((((((i) % ((8 as i32)))) == ((0 as i32))) as i32)) != 0 { 10 } else { 9 }; continue;
}
// C line 35
12 => {
let _ = { let assigned = (0 as i32); i = assigned; assigned };
vm_block = 3; continue;
}
// C line 34
13 => {
let _ = tool_fprintf(f, c"static const uint16_t unicode_comp_table[%u] = {".as_ptr(), &[ToolPrintArg::Int(tab_ce_len as u64)]);
vm_block = 12; continue;
}
// C line 33
14 => {
let _ = { total_table_bytes = (((((total_table_bytes) as usize)).wrapping_add((((tab_ce_len) as usize)).wrapping_mul((size_of::<u16>() as usize)))) as u32); total_table_bytes };
vm_block = 13; continue;
}
// C line 32
15 => {
let _ = { let old = total_tables; total_tables = (total_tables).wrapping_add(1); old };
vm_block = 14; continue;
}
// C line 25
16 => {
vm_block = if ((((i) < (tab_ce_len)) as i32)) != 0 { 19 } else { 15 }; continue;
}
// C line ?
17 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 16; continue;
}
// C line 27
18 => {
let _ = tool_printf(c"%05x %05x %05x\n".as_ptr(), &[ToolPrintArg::Int(*(((*(ce)).c).as_mut_ptr()).offset(((0 as i32)) as isize) as u64), ToolPrintArg::Int(*(((*(ce)).c).as_mut_ptr()).offset(((1 as i32)) as isize) as u64), ToolPrintArg::Int((*(ce)).p as u64)]);
vm_block = 17; continue;
}
// C line 26
19 => {
let _ = { let assigned = ptr::addr_of_mut!(*(tab_ce).offset((i) as isize)); ce = assigned; assigned };
vm_block = 18; continue;
}
// C line 25
20 => {
let _ = { let assigned = (0 as i32); i = assigned; assigned };
vm_block = 16; continue;
}
// C line 24
21 => {
let _ = tool_printf(c"tab_ce_len=%d\n".as_ptr(), &[ToolPrintArg::Int(tab_ce_len as u64)]);
vm_block = 20; continue;
}
// C line 20
22 => {
let _ = tool_qsort(((tab_ce) as *mut c_void), ((tab_ce_len) as usize), (size_of::<ComposeEntry>() as usize), ce_cmp);
vm_block = 21; continue;
}
// C line 9
23 => {
vm_block = if ((((i) <= ((1114111 as i32))) as i32)) != 0 { 31 } else { 22 }; continue;
}
// C line ?
24 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 23; continue;
}
// C line 17
25 => {
let _ = { let assigned = ((i) as u32); (*(ce)).p = assigned; assigned };
vm_block = 24; continue;
}
// C line 16
26 => {
let _ = { let assigned = ((*((*(ci)).decomp_data).offset(((1 as i32)) as isize)) as u32); *(((*(ce)).c).as_mut_ptr()).offset(((1 as i32)) as isize) = assigned; assigned };
vm_block = 25; continue;
}
// C line 15
27 => {
let _ = { let assigned = ((*((*(ci)).decomp_data).offset(((0 as i32)) as isize)) as u32); *(((*(ce)).c).as_mut_ptr()).offset(((0 as i32)) as isize) = assigned; assigned };
vm_block = 26; continue;
}
// C line 14
28 => {
let _ = { let assigned = ptr::addr_of_mut!(*(tab_ce).offset(({ let old = tab_ce_len; tab_ce_len = (tab_ce_len).wrapping_add(1); old }) as isize)); ce = assigned; assigned };
vm_block = 27; continue;
}
// C line 13
29 => {
let _ = if ((((!(((((tab_ce_len) < ((10000 as i32))) as i32)) != 0) as i32)) as i64)) != 0 { tool_assert_rtn(c"build_compose_table_dormant".as_ptr(), c"dormant-block.c".as_ptr(), (13 as i32), c"tab_ce_len < COMPOSE_LEN_MAX".as_ptr()) } else { { let _ = (0 as i32); } };
vm_block = 28; continue;
}
// C line 11
30 => {
vm_block = if (((((((((((((*(ci)).decomp_len) == ((2 as i32))) as i32)) != 0) && (((!(((*(ci)).is_compat()) != 0) as i32)) != 0)) as i32)) != 0) && (((!(((*(ci)).is_excluded()) != 0) as i32)) != 0)) as i32)) != 0 { 29 } else { 24 }; continue;
}
// C line 10
31 => {
ci = ptr::addr_of_mut!(*(unicode_db).offset((i) as isize));
vm_block = 30; continue;
}
// C line 9
32 => {
let _ = { let assigned = (0 as i32); i = assigned; assigned };
vm_block = 23; continue;
}
// C line 8
33 => {
let _ = { let assigned = (0 as i32); tab_ce_len = assigned; assigned };
vm_block = 32; continue;
}
// C line 7
34 => {
let _ = { let assigned = ((malloc(((size_of::<ComposeEntry>() as usize)).wrapping_mul((((10000 as i32)) as usize)))) as *mut ComposeEntry); tab_ce = assigned; assigned };
vm_block = 33; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// Test-only copy of unicode_gen.c build_decompose_table constructs original table then calls dormant composition dump. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn build_decompose_table_with_dormant_composition(mut f: *mut FILE) -> () {
let mut vm_local_storage = Vec::<u64>::new();
let mut i: i32 = core::mem::zeroed();
let mut array_len: i32 = core::mem::zeroed();
let mut code_max: i32 = core::mem::zeroed();
let mut data_len: i32 = core::mem::zeroed();
let mut count: i32 = core::mem::zeroed();
let mut tab_de: *mut DecompEntry = core::mem::zeroed();
let mut de_s: DecompEntry = core::mem::zeroed();
let mut de: *mut DecompEntry = core::mem::zeroed();
let mut data_buf: *mut u8 = core::mem::zeroed();
let mut v: u32 = core::mem::zeroed();
let mut vm_block: usize = 60;
loop { match vm_block {
// C line ?
0 => {
return ();
}
// C line 103
1 => {
let _ = free(((tab_de) as *mut c_void));
vm_block = 0; continue;
}
// C line 101
2 => {
let _ = free(((data_buf) as *mut c_void));
vm_block = 1; continue;
}
// C line 99
3 => {
let _ = build_compose_table_dormant(f, tab_de);
vm_block = 2; continue;
}
// C line 97
4 => {
let _ = tool_fprintf(f, c"\n};\n\n".as_ptr(), &[]);
vm_block = 3; continue;
}
// C line 92
5 => {
vm_block = if ((((i) < (data_len)) as i32)) != 0 { 9 } else { 4 }; continue;
}
// C line ?
6 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 5; continue;
}
// C line 95
7 => {
let _ = tool_fprintf(f, c" 0x%02x,".as_ptr(), &[ToolPrintArg::Int(((*(data_buf).offset((i) as isize)) as i32) as u64)]);
vm_block = 6; continue;
}
// C line 94
8 => {
let _ = tool_fprintf(f, c"\n   ".as_ptr(), &[]);
vm_block = 7; continue;
}
// C line 93
9 => {
vm_block = if ((((((i) % ((8 as i32)))) == ((0 as i32))) as i32)) != 0 { 8 } else { 7 }; continue;
}
// C line 92
10 => {
let _ = { let assigned = (0 as i32); i = assigned; assigned };
vm_block = 5; continue;
}
// C line 91
11 => {
let _ = tool_fprintf(f, c"static const uint8_t unicode_decomp_data[%d] = {".as_ptr(), &[ToolPrintArg::Int(data_len as u64)]);
vm_block = 10; continue;
}
// C line 90
12 => {
let _ = { total_table_bytes = (total_table_bytes).wrapping_add(((data_len) as u32)); total_table_bytes };
vm_block = 11; continue;
}
// C line 89
13 => {
let _ = { let old = total_tables; total_tables = (total_tables).wrapping_add(1); old };
vm_block = 12; continue;
}
// C line 87
14 => {
let _ = tool_fprintf(f, c"\n};\n\n".as_ptr(), &[]);
vm_block = 13; continue;
}
// C line 78
15 => {
vm_block = if ((((i) <= (code_max)) as i32)) != 0 { 22 } else { 14 }; continue;
}
// C line ?
16 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 15; continue;
}
// C line 84
17 => {
let _ = { i = (i).wrapping_add(((((*(de)).len) as i32)).wrapping_sub((1 as i32))); i };
vm_block = 16; continue;
}
// C line 83
18 => {
let _ = tool_fprintf(f, c" 0x%04x,".as_ptr(), &[ToolPrintArg::Int((((*(de)).data_index) as i32) as u64)]);
vm_block = 17; continue;
}
// C line 82
19 => {
let _ = tool_fprintf(f, c"\n   ".as_ptr(), &[]);
vm_block = 18; continue;
}
// C line 81
20 => {
vm_block = if (((((({ let old = count; count = (count).wrapping_add(1); old }) % ((8 as i32)))) == ((0 as i32))) as i32)) != 0 { 19 } else { 18 }; continue;
}
// C line 80
21 => {
vm_block = if (((((((*(de)).len) as i32)) != ((0 as i32))) as i32)) != 0 { 20 } else { 16 }; continue;
}
// C line 79
22 => {
let _ = { let assigned = ptr::addr_of_mut!(*(tab_de).offset((i) as isize)); de = assigned; assigned };
vm_block = 21; continue;
}
// C line 78
23 => {
let _ = { let assigned = (0 as i32); i = assigned; assigned };
vm_block = 15; continue;
}
// C line 77
24 => {
let _ = { let assigned = (0 as i32); count = assigned; assigned };
vm_block = 23; continue;
}
// C line 76
25 => {
let _ = tool_fprintf(f, c"static const uint16_t unicode_decomp_table2[%d] = {".as_ptr(), &[ToolPrintArg::Int(array_len as u64)]);
vm_block = 24; continue;
}
// C line 75
26 => {
let _ = { total_table_bytes = (((((total_table_bytes) as usize)).wrapping_add((((array_len) as usize)).wrapping_mul((size_of::<u16>() as usize)))) as u32); total_table_bytes };
vm_block = 25; continue;
}
// C line 74
27 => {
let _ = { let old = total_tables; total_tables = (total_tables).wrapping_add(1); old };
vm_block = 26; continue;
}
// C line 72
28 => {
let _ = tool_fprintf(f, c"\n};\n\n".as_ptr(), &[]);
vm_block = 27; continue;
}
// C line 58
29 => {
vm_block = if ((((i) <= (code_max)) as i32)) != 0 { 37 } else { 28 }; continue;
}
// C line ?
30 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 29; continue;
}
// C line 69
31 => {
let _ = { i = (i).wrapping_add(((((*(de)).len) as i32)).wrapping_sub((1 as i32))); i };
vm_block = 30; continue;
}
// C line 68
32 => {
let _ = tool_fprintf(f, c" 0x%08x,".as_ptr(), &[ToolPrintArg::Int(v as u64)]);
vm_block = 31; continue;
}
// C line 64
33 => {
let _ = { let assigned = ((((((((((*(de)).code).wrapping_shl((((32 as i32)).wrapping_sub((18 as i32))) as u32)) | (((((*(de)).len) as i32)).wrapping_shl(((((32 as i32)).wrapping_sub((18 as i32))).wrapping_sub((7 as i32))) as u32)))) | (((((*(de)).v_type) as i32)).wrapping_shl((((((32 as i32)).wrapping_sub((18 as i32))).wrapping_sub((7 as i32))).wrapping_sub((6 as i32))) as u32)))) | ((((*(unicode_db).offset(((*(de)).code) as isize)).is_compat()) as i32)))) as u32); v = assigned; assigned };
vm_block = 32; continue;
}
// C line 63
34 => {
let _ = tool_fprintf(f, c"\n   ".as_ptr(), &[]);
vm_block = 33; continue;
}
// C line 62
35 => {
vm_block = if (((((({ let old = count; count = (count).wrapping_add(1); old }) % ((4 as i32)))) == ((0 as i32))) as i32)) != 0 { 34 } else { 33 }; continue;
}
// C line 60
36 => {
vm_block = if (((((((*(de)).len) as i32)) != ((0 as i32))) as i32)) != 0 { 35 } else { 30 }; continue;
}
// C line 59
37 => {
let _ = { let assigned = ptr::addr_of_mut!(*(tab_de).offset((i) as isize)); de = assigned; assigned };
vm_block = 36; continue;
}
// C line 58
38 => {
let _ = { let assigned = (0 as i32); i = assigned; assigned };
vm_block = 29; continue;
}
// C line 57
39 => {
let _ = { let assigned = (0 as i32); count = assigned; assigned };
vm_block = 38; continue;
}
// C line 56
40 => {
let _ = tool_fprintf(f, c"static const uint32_t unicode_decomp_table1[%d] = {".as_ptr(), &[ToolPrintArg::Int(array_len as u64)]);
vm_block = 39; continue;
}
// C line 55
41 => {
let _ = { total_table_bytes = (((((total_table_bytes) as usize)).wrapping_add((((array_len) as usize)).wrapping_mul((size_of::<u32>() as usize)))) as u32); total_table_bytes };
vm_block = 40; continue;
}
// C line 54
42 => {
let _ = { let old = total_tables; total_tables = (total_tables).wrapping_add(1); old };
vm_block = 41; continue;
}
// C line 21
43 => {
vm_block = if ((((i) <= (code_max)) as i32)) != 0 { 49 } else { 42 }; continue;
}
// C line ?
44 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 43; continue;
}
// C line 26
45 => {
let _ = { let old = array_len; array_len = (array_len).wrapping_add(1); old };
vm_block = 44; continue;
}
// C line 25
46 => {
let _ = { i = (i).wrapping_add(((((*(de)).len) as i32)).wrapping_sub((1 as i32))); i };
vm_block = 45; continue;
}
// C line 24
47 => {
let _ = add_decomp_data(data_buf, ptr::addr_of_mut!(data_len), de);
vm_block = 46; continue;
}
// C line 23
48 => {
vm_block = if (((((((*(de)).len) as i32)) != ((0 as i32))) as i32)) != 0 { 47 } else { 44 }; continue;
}
// C line 22
49 => {
let _ = { let assigned = ptr::addr_of_mut!(*(tab_de).offset((i) as isize)); de = assigned; assigned };
vm_block = 48; continue;
}
// C line 21
50 => {
let _ = { let assigned = (0 as i32); i = assigned; assigned };
vm_block = 43; continue;
}
// C line 20
51 => {
let _ = { let assigned = (0 as i32); array_len = assigned; assigned };
vm_block = 50; continue;
}
// C line 19
52 => {
let _ = { let assigned = (0 as i32); data_len = assigned; assigned };
vm_block = 51; continue;
}
// C line 18
53 => {
let _ = { let assigned = ((malloc((((100000 as i32)) as usize))) as *mut u8); data_buf = assigned; assigned };
vm_block = 52; continue;
}
// C line 13
54 => {
vm_block = if ((((i) >= ((0 as i32))) as i32)) != 0 { 56 } else { 53 }; continue;
}
// C line ?
55 => {
let _ = { let old = i; i = (i).wrapping_sub(1); old };
vm_block = 54; continue;
}
// C line 14
56 => {
let _ = find_decomp_run(tab_de, i);
vm_block = 55; continue;
}
// C line 13
57 => {
let _ = { let assigned = code_max; i = assigned; assigned };
vm_block = 54; continue;
}
// C line 11
58 => {
let _ = { let assigned = ((mallocz(((((code_max).wrapping_add((2 as i32))) as usize)).wrapping_mul((size_of::<DecompEntry>() as usize)))) as *mut DecompEntry); tab_de = assigned; assigned };
vm_block = 57; continue;
}
// C line 9
59 => {
let _ = { let assigned = (1114111 as i32); code_max = assigned; assigned };
vm_block = 58; continue;
}
// C line 6
60 => {
de = ptr::addr_of_mut!(de_s);
vm_block = 59; continue;
}
_ => std::process::abort(),
} }
}
