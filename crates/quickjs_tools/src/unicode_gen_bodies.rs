// Generated from the official C AST; original goto CFG retained.
// c: unicode_gen.c:73. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn mallocz(mut size: usize) -> *mut c_void {
let mut vm_local_storage = Vec::<u64>::new();
let mut ptr: *mut c_void = core::mem::zeroed();
let mut vm_block: usize = 3;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 78
1 => {
return ptr;
}
// C line 77
2 => {
let _ = { let dst = (ptr) as *mut u8; core::ptr::write_bytes(dst, ((0 as i32)) as u8, (size) as usize); dst as *mut c_void };
vm_block = 1; continue;
}
// C line 76
3 => {
let _ = { let assigned = malloc(size); ptr = assigned; assigned };
vm_block = 2; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: unicode_gen.c:81. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn get_field(mut p: *const c_char, mut n: i32) -> *const c_char {
let mut vm_local_storage = Vec::<u64>::new();
let mut i: i32 = core::mem::zeroed();
let mut vm_block: usize = 9;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 91
1 => {
return p;
}
// C line 84
2 => {
vm_block = if ((((i) < (n)) as i32)) != 0 { 7 } else { 1 }; continue;
}
// C line ?
3 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 2; continue;
}
// C line 89
4 => {
let _ = { let old = p; p = (p).offset(1); old };
vm_block = 3; continue;
}
// C line 88
5 => {
return core::ptr::null_mut::<c_char>();
}
// C line 87
6 => {
vm_block = if ((((((*(p)) as i32)) == ((0 as i32))) as i32)) != 0 { 5 } else { 4 }; continue;
}
// C line 85
7 => {
vm_block = if ((((((((((*(p)) as i32)) != ((59 as i32))) as i32)) != 0) && (((((((*(p)) as i32)) != ((0 as i32))) as i32)) != 0)) as i32)) != 0 { 8 } else { 6 }; continue;
}
// C line 86
8 => {
let _ = { let old = p; p = (p).offset(1); old };
vm_block = 7; continue;
}
// C line 84
9 => {
let _ = { let assigned = (0 as i32); i = assigned; assigned };
vm_block = 2; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: unicode_gen.c:94. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn get_field_buf(mut buf: *mut c_char, mut buf_size: usize, mut p: *const c_char, mut n: i32) -> *const c_char {
let mut vm_local_storage = Vec::<u64>::new();
let mut q: *mut c_char = core::mem::zeroed();
let mut vm_block: usize = 8;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 105
1 => {
return buf;
}
// C line 104
2 => {
let _ = { let assigned = (((0 as i32)) as c_char); *(q) = assigned; assigned };
vm_block = 1; continue;
}
// C line 99
3 => {
vm_block = if ((((((((((*(p)) as i32)) != ((59 as i32))) as i32)) != 0) && (((((((*(p)) as i32)) != ((0 as i32))) as i32)) != 0)) as i32)) != 0 { 6 } else { 2 }; continue;
}
// C line 102
4 => {
let _ = { let old = p; p = (p).offset(1); old };
vm_block = 3; continue;
}
// C line 101
5 => {
let _ = { let assigned = *(p); *({ let old = q; q = (q).offset(1); old }) = assigned; assigned };
vm_block = 4; continue;
}
// C line 100
6 => {
vm_block = if ((((((((q).offset_from(buf) as i64)) as usize)) < ((buf_size).wrapping_sub((((1 as i32)) as usize)))) as i32)) != 0 { 5 } else { 4 }; continue;
}
// C line 98
7 => {
let _ = { let assigned = buf; q = assigned; assigned };
vm_block = 3; continue;
}
// C line 97
8 => {
let _ = { let assigned = get_field(p, n); p = assigned; assigned };
vm_block = 7; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: unicode_gen.c:108. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn add_char(mut pbuf: *mut *mut i32, mut psize: *mut i32, mut plen: *mut i32, mut c: i32) -> () {
let mut vm_local_storage = Vec::<u64>::new();
let mut len: i32 = core::mem::zeroed();
let mut size: i32 = core::mem::zeroed();
let mut buf: *mut i32 = core::mem::zeroed();
let mut vm_block: usize = 11;
loop { match vm_block {
// C line ?
0 => {
return ();
}
// C line 122
1 => {
let _ = { let assigned = len; *(plen) = assigned; assigned };
vm_block = 0; continue;
}
// C line 121
2 => {
let _ = { let assigned = c; *(buf).offset(({ let old = len; len = (len).wrapping_add(1); old }) as isize) = assigned; assigned };
vm_block = 1; continue;
}
// C line 119
3 => {
let _ = { let assigned = size; *(psize) = assigned; assigned };
vm_block = 2; continue;
}
// C line 118
4 => {
let _ = { let assigned = buf; *(pbuf) = assigned; assigned };
vm_block = 3; continue;
}
// C line 117
5 => {
let _ = { let assigned = ((realloc(((buf) as *mut c_void), ((size_of::<i32>() as usize)).wrapping_mul(((size) as usize)))) as *mut i32); buf = assigned; assigned };
vm_block = 4; continue;
}
// C line 116
6 => {
let _ = { let assigned = max_int((len).wrapping_add((1 as i32)), (((size).wrapping_mul((3 as i32))) / ((2 as i32)))); size = assigned; assigned };
vm_block = 5; continue;
}
// C line 115
7 => {
let _ = { let assigned = *(psize); size = assigned; assigned };
vm_block = 6; continue;
}
// C line 114
8 => {
vm_block = if ((((len) >= (size)) as i32)) != 0 { 7 } else { 2 }; continue;
}
// C line 113
9 => {
let _ = { let assigned = *(plen); len = assigned; assigned };
vm_block = 8; continue;
}
// C line 112
10 => {
let _ = { let assigned = *(psize); size = assigned; assigned };
vm_block = 9; continue;
}
// C line 111
11 => {
let _ = { let assigned = *(pbuf); buf = assigned; assigned };
vm_block = 10; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: unicode_gen.c:125. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn get_field_str(mut plen: *mut i32, mut str: *const c_char, mut n: i32) -> *mut i32 {
let mut vm_local_storage = Vec::<u64>::new();
let mut p: *const c_char = core::mem::zeroed();
let mut buf: *mut i32 = core::mem::zeroed();
let mut len: i32 = core::mem::zeroed();
let mut size: i32 = core::mem::zeroed();
let mut vm_block: usize = 15;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 145
1 => {
return buf;
}
// C line 144
2 => {
let _ = { let assigned = len; *(plen) = assigned; assigned };
vm_block = 1; continue;
}
// C line 137
3 => {
vm_block = 7; continue;
}
// C line 142
4 => {
let _ = add_char(ptr::addr_of_mut!(buf), ptr::addr_of_mut!(size), ptr::addr_of_mut!(len), ((strtoul(p, ((ptr::addr_of_mut!(p)) as *mut *mut c_char), (16 as i32))) as i32));
vm_block = 3; continue;
}
// C line 141
5 => {
vm_block = 2; continue;
}
// C line 140
6 => {
vm_block = if ((!((isxdigit(((*(p)) as i32))) != 0) as i32)) != 0 { 5 } else { 4 }; continue;
}
// C line 138
7 => {
vm_block = if (isspace(((*(p)) as i32))) != 0 { 8 } else { 6 }; continue;
}
// C line 139
8 => {
let _ = { let old = p; p = (p).offset(1); old };
vm_block = 7; continue;
}
// C line 136
9 => {
let _ = { let assigned = core::ptr::null_mut::<i32>(); buf = assigned; assigned };
vm_block = 3; continue;
}
// C line 135
10 => {
let _ = { let assigned = (0 as i32); size = assigned; assigned };
vm_block = 9; continue;
}
// C line 134
11 => {
let _ = { let assigned = (0 as i32); len = assigned; assigned };
vm_block = 10; continue;
}
// C line 132
12 => {
return core::ptr::null_mut::<i32>();
}
// C line 131
13 => {
let _ = { let assigned = (0 as i32); *(plen) = assigned; assigned };
vm_block = 12; continue;
}
// C line 130
14 => {
vm_block = if ((!(!(p).is_null()) as i32)) != 0 { 13 } else { 11 }; continue;
}
// C line 129
15 => {
let _ = { let assigned = get_field(str, n); p = assigned; assigned };
vm_block = 14; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: unicode_gen.c:148. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn get_line(mut buf: *mut c_char, mut buf_size: i32, mut f: *mut FILE) -> *mut c_char {
let mut vm_local_storage = Vec::<u64>::new();
let mut len: i32 = core::mem::zeroed();
let mut vm_block: usize = 6;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 156
1 => {
return buf;
}
// C line 155
2 => {
let _ = { let assigned = (((0 as i32)) as c_char); *(buf).offset(((len).wrapping_sub((1 as i32))) as isize) = assigned; assigned };
vm_block = 1; continue;
}
// C line 154
3 => {
vm_block = if ((((((((len) > ((0 as i32))) as i32)) != 0) && (((((((*(buf).offset(((len).wrapping_sub((1 as i32))) as isize)) as i32)) == ((10 as i32))) as i32)) != 0)) as i32)) != 0 { 2 } else { 1 }; continue;
}
// C line 153
4 => {
let _ = { let assigned = ((strlen(buf)) as i32); len = assigned; assigned };
vm_block = 3; continue;
}
// C line 152
5 => {
return core::ptr::null_mut::<c_char>();
}
// C line 151
6 => {
vm_block = if ((!(!(fgets(buf, buf_size, f)).is_null()) as i32)) != 0 { 5 } else { 4 }; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: unicode_gen.c:174. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn re_string_hash(mut len: i32, mut buf: *const u32) -> u32 {
let mut vm_local_storage = Vec::<u64>::new();
let mut i: i32 = core::mem::zeroed();
let mut h: u32 = core::mem::zeroed();
let mut vm_block: usize = 6;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 181
1 => {
return (h).wrapping_mul((((1640531527 as i32)) as u32));
}
// C line 179
2 => {
vm_block = if ((((i) < (len)) as i32)) != 0 { 4 } else { 1 }; continue;
}
// C line ?
3 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 2; continue;
}
// C line 180
4 => {
let _ = { let assigned = ((h).wrapping_mul((((263 as i32)) as u32))).wrapping_add(*(buf).offset((i) as isize)); h = assigned; assigned };
vm_block = 3; continue;
}
// C line 179
5 => {
let _ = { let assigned = (0 as i32); i = assigned; assigned };
vm_block = 2; continue;
}
// C line 178
6 => {
let _ = { let assigned = (((1 as i32)) as u32); h = assigned; assigned };
vm_block = 5; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: unicode_gen.c:184. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn re_string_list_init(mut s: *mut REStringList) -> () {
let mut vm_local_storage = Vec::<u64>::new();
let mut vm_block: usize = 4;
loop { match vm_block {
// C line ?
0 => {
return ();
}
// C line 189
1 => {
let _ = { let assigned = core::ptr::null_mut::<*mut REString>(); (*(s)).hash_table = assigned; assigned };
vm_block = 0; continue;
}
// C line 188
2 => {
let _ = { let assigned = (0 as i32); (*(s)).hash_bits = assigned; assigned };
vm_block = 1; continue;
}
// C line 187
3 => {
let _ = { let assigned = (((0 as i32)) as u32); (*(s)).hash_size = assigned; assigned };
vm_block = 2; continue;
}
// C line 186
4 => {
let _ = { let assigned = (((0 as i32)) as u32); (*(s)).n_strings = assigned; assigned };
vm_block = 3; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: unicode_gen.c:192. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn re_string_list_free(mut s: *mut REStringList) -> () {
let mut vm_local_storage = Vec::<u64>::new();
let mut p: *mut REString = core::mem::zeroed();
let mut p_next: *mut REString = core::mem::zeroed();
let mut i: i32 = core::mem::zeroed();
let mut vm_block: usize = 9;
loop { match vm_block {
// C line ?
0 => {
return ();
}
// C line 202
1 => {
let _ = free((((*(s)).hash_table) as *mut c_void));
vm_block = 0; continue;
}
// C line 196
2 => {
vm_block = if ((((((i) as u32)) < ((*(s)).hash_size)) as i32)) != 0 { 8 } else { 1 }; continue;
}
// C line ?
3 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 2; continue;
}
// C line 197
4 => {
vm_block = if ((((p) != (core::ptr::null_mut::<REString>())) as i32)) != 0 { 7 } else { 3 }; continue;
}
// C line ?
5 => {
let _ = { let assigned = p_next; p = assigned; assigned };
vm_block = 4; continue;
}
// C line 199
6 => {
let _ = free(((p) as *mut c_void));
vm_block = 5; continue;
}
// C line 198
7 => {
let _ = { let assigned = (*(p)).next; p_next = assigned; assigned };
vm_block = 6; continue;
}
// C line 197
8 => {
let _ = { let assigned = *((*(s)).hash_table).offset((i) as isize); p = assigned; assigned };
vm_block = 4; continue;
}
// C line 196
9 => {
let _ = { let assigned = (0 as i32); i = assigned; assigned };
vm_block = 2; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: unicode_gen.c:205. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn lre_print_char(mut c: i32, mut is_range: i32) -> () {
let mut vm_local_storage = Vec::<u64>::new();
let mut vm_block: usize = 5;
loop { match vm_block {
// C line ?
0 => {
return ();
}
// C line 209
1 => {
let _ = tool_printf(c"\\%c".as_ptr(), &[ToolPrintArg::Int(c as u64)]);
vm_block = 0; continue;
}
// C line 211
2 => {
let _ = tool_printf(c"%c".as_ptr(), &[ToolPrintArg::Int(c as u64)]);
vm_block = 0; continue;
}
// C line 213
3 => {
let _ = tool_printf(c"\\u{%04x}".as_ptr(), &[ToolPrintArg::Int(c as u64)]);
vm_block = 0; continue;
}
// C line 210
4 => {
vm_block = if ((((((((c) >= ((32 as i32))) as i32)) != 0) && (((((c) <= ((126 as i32))) as i32)) != 0)) as i32)) != 0 { 2 } else { 3 }; continue;
}
// C line 207
5 => {
vm_block = if ((((((((((((c) == ((39 as i32))) as i32)) != 0) || (((((c) == ((92 as i32))) as i32)) != 0)) as i32)) != 0) || ((((((is_range) != 0) && (((((((((c) == ((45 as i32))) as i32)) != 0) || (((((c) == ((93 as i32))) as i32)) != 0)) as i32)) != 0)) as i32)) != 0)) as i32)) != 0 { 1 } else { 4 }; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: unicode_gen.c:217. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn re_string_list_dump(mut str: *const c_char, mut s: *const REStringList) -> () {
let mut vm_local_storage = Vec::<u64>::new();
let mut p: *mut REString = core::mem::zeroed();
let mut i: i32 = core::mem::zeroed();
let mut j: i32 = core::mem::zeroed();
let mut k: i32 = core::mem::zeroed();
let mut vm_block: usize = 15;
loop { match vm_block {
// C line ?
0 => {
return ();
}
// C line 225
1 => {
vm_block = if ((((((i) as u32)) < ((*(s)).hash_size)) as i32)) != 0 { 12 } else { 0 }; continue;
}
// C line ?
2 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 1; continue;
}
// C line 226
3 => {
vm_block = if ((((p) != (core::ptr::null_mut::<REString>())) as i32)) != 0 { 11 } else { 2 }; continue;
}
// C line ?
4 => {
let _ = { let assigned = (*(p)).next; p = assigned; assigned };
vm_block = 3; continue;
}
// C line 232
5 => {
let _ = { let old = j; j = (j).wrapping_add(1); old };
vm_block = 4; continue;
}
// C line 231
6 => {
let _ = tool_printf(c"'\n".as_ptr(), &[]);
vm_block = 5; continue;
}
// C line 228
7 => {
vm_block = if ((((((k) as u32)) < ((*(p)).len)) as i32)) != 0 { 9 } else { 6 }; continue;
}
// C line ?
8 => {
let _ = { let old = k; k = (k).wrapping_add(1); old };
vm_block = 7; continue;
}
// C line 229
9 => {
let _ = lre_print_char(((*(((*(p)).buf).as_mut_ptr()).offset((k) as isize)) as i32), (FALSE as i32));
vm_block = 8; continue;
}
// C line 228
10 => {
let _ = { let assigned = (0 as i32); k = assigned; assigned };
vm_block = 7; continue;
}
// C line 227
11 => {
let _ = tool_printf(c"  %d/%d: '".as_ptr(), &[ToolPrintArg::Int(j as u64), ToolPrintArg::Int((*(s)).n_strings as u64)]);
vm_block = 10; continue;
}
// C line 226
12 => {
let _ = { let assigned = *((*(s)).hash_table).offset((i) as isize); p = assigned; assigned };
vm_block = 3; continue;
}
// C line 225
13 => {
let _ = { let assigned = (0 as i32); i = assigned; assigned };
vm_block = 1; continue;
}
// C line 224
14 => {
let _ = { let assigned = (0 as i32); j = assigned; assigned };
vm_block = 13; continue;
}
// C line 222
15 => {
let _ = tool_printf(c"%s:\n".as_ptr(), &[ToolPrintArg::Str(str as *const c_char)]);
vm_block = 14; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: unicode_gen.c:237. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn re_string_find2(mut s: *mut REStringList, mut len: i32, mut buf: *const u32, mut h0: u32, mut add_flag: i32) -> *mut REString {
let mut vm_local_storage = Vec::<u64>::new();
let mut h: u32 = core::mem::zeroed();
let mut p: *mut REString = core::mem::zeroed();
let mut new_hash_table: *mut *mut REString = core::mem::zeroed();
let mut p_next: *mut REString = core::mem::zeroed();
let mut new_hash_bits: i32 = core::mem::zeroed();
let mut i: i32 = core::mem::zeroed();
let mut new_hash_size: u32 = core::mem::zeroed();
let mut vm_block: usize = 43;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 290
1 => {
return p;
}
// C line 289
2 => {
let _ = { let dst = (((((*(p)).buf).as_mut_ptr()) as *mut c_void)) as *mut u8; core::ptr::copy_nonoverlapping((((buf) as *const c_void)) as *const u8, dst, (((size_of::<u32>() as usize)).wrapping_mul(((len) as usize))) as usize); dst as *mut c_void };
vm_block = 1; continue;
}
// C line 288
3 => {
let _ = { let assigned = (((0 as i32)) as u32); (*(p)).flags = assigned; assigned };
vm_block = 2; continue;
}
// C line 287
4 => {
let _ = { let assigned = ((len) as u32); (*(p)).len = assigned; assigned };
vm_block = 3; continue;
}
// C line 286
5 => {
let _ = { let assigned = h0; (*(p)).hash = assigned; assigned };
vm_block = 4; continue;
}
// C line 285
6 => {
let _ = { let old = (*(s)).n_strings; (*(s)).n_strings = ((*(s)).n_strings).wrapping_add(1); old };
vm_block = 5; continue;
}
// C line 284
7 => {
let _ = { let assigned = p; *((*(s)).hash_table).offset((h) as isize) = assigned; assigned };
vm_block = 6; continue;
}
// C line 283
8 => {
let _ = { let assigned = *((*(s)).hash_table).offset((h) as isize); (*(p)).next = assigned; assigned };
vm_block = 7; continue;
}
// C line 282
9 => {
return core::ptr::null_mut::<REString>();
}
// C line 281
10 => {
vm_block = if ((!(!(p).is_null()) as i32)) != 0 { 9 } else { 8 }; continue;
}
// C line 280
11 => {
let _ = { let assigned = ((malloc(((size_of::<REString>() as usize)).wrapping_add((((len) as usize)).wrapping_mul((size_of::<u32>() as usize))))) as *mut REString); p = assigned; assigned };
vm_block = 10; continue;
}
// C line 277
12 => {
let _ = { let assigned = (h0).wrapping_shr((((32 as i32)).wrapping_sub((*(s)).hash_bits)) as u32); h = assigned; assigned };
vm_block = 11; continue;
}
// C line 276
13 => {
let _ = { let assigned = new_hash_table; (*(s)).hash_table = assigned; assigned };
vm_block = 12; continue;
}
// C line 275
14 => {
let _ = { let assigned = new_hash_size; (*(s)).hash_size = assigned; assigned };
vm_block = 13; continue;
}
// C line 274
15 => {
let _ = { let assigned = new_hash_bits; (*(s)).hash_bits = assigned; assigned };
vm_block = 14; continue;
}
// C line 273
16 => {
let _ = free((((*(s)).hash_table) as *mut c_void));
vm_block = 15; continue;
}
// C line 265
17 => {
vm_block = if ((((((i) as u32)) < ((*(s)).hash_size)) as i32)) != 0 { 25 } else { 16 }; continue;
}
// C line ?
18 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 17; continue;
}
// C line 266
19 => {
vm_block = if ((((p) != (core::ptr::null_mut::<REString>())) as i32)) != 0 { 24 } else { 18 }; continue;
}
// C line ?
20 => {
let _ = { let assigned = p_next; p = assigned; assigned };
vm_block = 19; continue;
}
// C line 270
21 => {
let _ = { let assigned = p; *(new_hash_table).offset((h) as isize) = assigned; assigned };
vm_block = 20; continue;
}
// C line 269
22 => {
let _ = { let assigned = *(new_hash_table).offset((h) as isize); (*(p)).next = assigned; assigned };
vm_block = 21; continue;
}
// C line 268
23 => {
let _ = { let assigned = ((*(p)).hash).wrapping_shr((((32 as i32)).wrapping_sub(new_hash_bits)) as u32); h = assigned; assigned };
vm_block = 22; continue;
}
// C line 267
24 => {
let _ = { let assigned = (*(p)).next; p_next = assigned; assigned };
vm_block = 23; continue;
}
// C line 266
25 => {
let _ = { let assigned = *((*(s)).hash_table).offset((i) as isize); p = assigned; assigned };
vm_block = 19; continue;
}
// C line 265
26 => {
let _ = { let assigned = (0 as i32); i = assigned; assigned };
vm_block = 17; continue;
}
// C line 264
27 => {
let _ = { let dst = (((new_hash_table) as *mut c_void)) as *mut u8; core::ptr::write_bytes(dst, ((0 as i32)) as u8, (((size_of::<*mut REString>() as usize)).wrapping_mul(((new_hash_size) as usize))) as usize); dst as *mut c_void };
vm_block = 26; continue;
}
// C line 263
28 => {
return core::ptr::null_mut::<REString>();
}
// C line 262
29 => {
vm_block = if ((!(!(new_hash_table).is_null()) as i32)) != 0 { 28 } else { 27 }; continue;
}
// C line 261
30 => {
let _ = { let assigned = ((malloc(((size_of::<*mut REString>() as usize)).wrapping_mul(((new_hash_size) as usize)))) as *mut *mut REString); new_hash_table = assigned; assigned };
vm_block = 29; continue;
}
// C line 260
31 => {
let _ = { let assigned = ((((1 as i32)).wrapping_shl((new_hash_bits) as u32)) as u32); new_hash_size = assigned; assigned };
vm_block = 30; continue;
}
// C line 259
32 => {
let _ = { let assigned = max_int(((*(s)).hash_bits).wrapping_add((1 as i32)), (4 as i32)); new_hash_bits = assigned; assigned };
vm_block = 31; continue;
}
// C line 255
33 => {
vm_block = if ((((!(((!(((((((*(s)).n_strings).wrapping_add((((1 as i32)) as u32))) > ((*(s)).hash_size)) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 32 } else { 11 }; continue;
}
// C line 253
34 => {
return core::ptr::null_mut::<REString>();
}
// C line 252
35 => {
vm_block = if ((!((add_flag) != 0) as i32)) != 0 { 34 } else { 33 }; continue;
}
// C line 244
36 => {
vm_block = if ((((p) != (core::ptr::null_mut::<REString>())) as i32)) != 0 { 39 } else { 35 }; continue;
}
// C line ?
37 => {
let _ = { let assigned = (*(p)).next; p = assigned; assigned };
vm_block = 36; continue;
}
// C line 247
38 => {
return p;
}
// C line 245
39 => {
vm_block = if (((((((((((((*(p)).hash) == (h0)) as i32)) != 0) && ((((((*(p)).len) == (((len) as u32))) as i32)) != 0)) as i32)) != 0) && (((!((memcmp(((((*(p)).buf).as_mut_ptr()) as *const c_void), ((buf) as *const c_void), (((len) as usize)).wrapping_mul((size_of::<u32>() as usize)))) != 0) as i32)) != 0)) as i32)) != 0 { 38 } else { 37 }; continue;
}
// C line 244
40 => {
let _ = { let assigned = *((*(s)).hash_table).offset((h) as isize); p = assigned; assigned };
vm_block = 36; continue;
}
// C line 243
41 => {
let _ = { let assigned = (h0).wrapping_shr((((32 as i32)).wrapping_sub((*(s)).hash_bits)) as u32); h = assigned; assigned };
vm_block = 40; continue;
}
// C line 242
42 => {
vm_block = if (((((*(s)).n_strings) != ((((0 as i32)) as u32))) as i32)) != 0 { 41 } else { 35 }; continue;
}
// C line 240
43 => {
h = (((0 as i32)) as u32);
vm_block = 42; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: unicode_gen.c:293. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn re_string_find(mut s: *mut REStringList, mut len: i32, mut buf: *const u32, mut add_flag: i32) -> *mut REString {
let mut vm_local_storage = Vec::<u64>::new();
let mut h0: u32 = core::mem::zeroed();
let mut vm_block: usize = 2;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 298
1 => {
return re_string_find2(s, len, buf, h0, add_flag);
}
// C line 297
2 => {
let _ = { let assigned = re_string_hash(len, buf); h0 = assigned; assigned };
vm_block = 1; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: unicode_gen.c:301. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn re_string_add(mut s: *mut REStringList, mut len: i32, mut buf: *const u32) -> () {
let mut vm_local_storage = Vec::<u64>::new();
let mut vm_block: usize = 1;
loop { match vm_block {
// C line ?
0 => {
return ();
}
// C line 303
1 => {
let _ = re_string_find(s, len, buf, (TRUE as i32));
vm_block = 0; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: unicode_gen.c:424. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn find_name(mut tab: *mut *const c_char, mut tab_len: i32, mut name: *const c_char) -> i32 {
let mut vm_local_storage = Vec::<u64>::new();
let mut i: i32 = core::mem::zeroed();
let mut len: i32 = core::mem::zeroed();
let mut name_len: i32 = core::mem::zeroed();
let mut p: *const c_char = core::mem::zeroed();
let mut r: *const c_char = core::mem::zeroed();
let mut vm_block: usize = 16;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 445
1 => {
return ((1 as i32)).wrapping_neg();
}
// C line 430
2 => {
vm_block = if ((((i) < (tab_len)) as i32)) != 0 { 14 } else { 1 }; continue;
}
// C line ?
3 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 2; continue;
}
// C line 432
4 => {
vm_block = 13; continue;
}
// C line 442
5 => {
let _ = { let assigned = (r).offset((((1 as i32)) as isize)); p = assigned; assigned };
vm_block = 4; continue;
}
// C line 441
6 => {
vm_block = 3; continue;
}
// C line 440
7 => {
vm_block = if ((!(!(r).is_null()) as i32)) != 0 { 6 } else { 5 }; continue;
}
// C line 439
8 => {
return i;
}
// C line 438
9 => {
vm_block = if ((((((((len) == (name_len)) as i32)) != 0) && (((((memcmp(((p) as *const c_void), ((name) as *const c_void), ((len) as usize))) == ((0 as i32))) as i32)) != 0)) as i32)) != 0 { 8 } else { 7 }; continue;
}
// C line 435
10 => {
let _ = { let assigned = ((strlen(p)) as i32); len = assigned; assigned };
vm_block = 9; continue;
}
// C line 437
11 => {
let _ = { let assigned = ((((r).offset_from(p) as i64)) as i32); len = assigned; assigned };
vm_block = 9; continue;
}
// C line 434
12 => {
vm_block = if ((!(!(r).is_null()) as i32)) != 0 { 10 } else { 11 }; continue;
}
// C line 433
13 => {
let _ = { let assigned = strchr(p, (44 as i32)); r = assigned; assigned };
vm_block = 12; continue;
}
// C line 431
14 => {
let _ = { let assigned = *(tab).offset((i) as isize); p = assigned; assigned };
vm_block = 4; continue;
}
// C line 430
15 => {
let _ = { let assigned = (0 as i32); i = assigned; assigned };
vm_block = 2; continue;
}
// C line 429
16 => {
let _ = { let assigned = ((strlen(name)) as i32); name_len = assigned; assigned };
vm_block = 15; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: unicode_gen.c:448. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn get_prop(mut c: u32, mut prop_idx: i32) -> i32 {
let mut vm_local_storage = Vec::<u64>::new();
let mut vm_block: usize = 1;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 450
1 => {
return (((((*(((*(unicode_db).offset((c) as isize)).prop_bitmap_tab).as_mut_ptr()).offset(((prop_idx).wrapping_shr(((5 as i32)) as u32)) as isize)).wrapping_shr((((prop_idx) & ((31 as i32)))) as u32)) & ((((1 as i32)) as u32)))) as i32);
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: unicode_gen.c:453. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn set_prop(mut c: u32, mut prop_idx: i32, mut val: i32) -> () {
let mut vm_local_storage = Vec::<u64>::new();
let mut mask: u32 = core::mem::zeroed();
let mut vm_block: usize = 4;
loop { match vm_block {
// C line ?
0 => {
return ();
}
// C line 458
1 => {
let _ = { *(((*(unicode_db).offset((c) as isize)).prop_bitmap_tab).as_mut_ptr()).offset(((prop_idx).wrapping_shr(((5 as i32)) as u32)) as isize) = ((*(((*(unicode_db).offset((c) as isize)).prop_bitmap_tab).as_mut_ptr()).offset(((prop_idx).wrapping_shr(((5 as i32)) as u32)) as isize)) | (mask)); *(((*(unicode_db).offset((c) as isize)).prop_bitmap_tab).as_mut_ptr()).offset(((prop_idx).wrapping_shr(((5 as i32)) as u32)) as isize) };
vm_block = 0; continue;
}
// C line 460
2 => {
let _ = { *(((*(unicode_db).offset((c) as isize)).prop_bitmap_tab).as_mut_ptr()).offset(((prop_idx).wrapping_shr(((5 as i32)) as u32)) as isize) = ((*(((*(unicode_db).offset((c) as isize)).prop_bitmap_tab).as_mut_ptr()).offset(((prop_idx).wrapping_shr(((5 as i32)) as u32)) as isize)) & ((!(mask)))); *(((*(unicode_db).offset((c) as isize)).prop_bitmap_tab).as_mut_ptr()).offset(((prop_idx).wrapping_shr(((5 as i32)) as u32)) as isize) };
vm_block = 0; continue;
}
// C line 457
3 => {
vm_block = if (val) != 0 { 1 } else { 2 }; continue;
}
// C line 456
4 => {
let _ = { let assigned = ((1 as u32)).wrapping_shl((((prop_idx) & ((31 as i32)))) as u32); mask = assigned; assigned };
vm_block = 3; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: unicode_gen.c:463. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn parse_unicode_data(mut filename: *const c_char) -> () {
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
let mut vm_block: usize = 79;
loop { match vm_block {
// C line ?
0 => {
return ();
}
// C line 597
1 => {
let _ = fclose(f);
vm_block = 0; continue;
}
// C line 479
2 => {
vm_block = 73; continue;
}
// C line 594
3 => {
let _ = { let assigned = code; last_code = assigned; assigned };
vm_block = 2; continue;
}
// C line 590
4 => {
vm_block = if ((((i_1) < (code)) as i32)) != 0 { 6 } else { 3 }; continue;
}
// C line ?
5 => {
let _ = { let old = i_1; i_1 = (i_1).wrapping_add(1); old };
vm_block = 4; continue;
}
// C line 591
6 => {
let _ = { let assigned = *(ci); *(unicode_db).offset((i_1) as isize) = assigned; assigned };
vm_block = 5; continue;
}
// C line 590
7 => {
let _ = { let assigned = (last_code).wrapping_add((1 as i32)); i_1 = assigned; assigned };
vm_block = 4; continue;
}
// C line 589
8 => {
let _ = if ((((!((((((((*(ci)).script_ext_len) as i32)) == ((0 as i32))) as i32)) != 0) as i32)) as i64)) != 0 { tool_assert_rtn(c"parse_unicode_data".as_ptr(), c"unicode_gen.c".as_ptr(), (589 as i32), c"ci->script_ext_len == 0".as_ptr()) } else { { let _ = (0 as i32); } };
vm_block = 7; continue;
}
// C line 588
9 => {
let _ = if ((((!((((((*(ci)).decomp_len) == ((0 as i32))) as i32)) != 0) as i32)) as i64)) != 0 { tool_assert_rtn(c"parse_unicode_data".as_ptr(), c"unicode_gen.c".as_ptr(), (588 as i32), c"ci->decomp_len == 0".as_ptr()) } else { { let _ = (0 as i32); } };
vm_block = 8; continue;
}
// C line 585
10 => {
vm_block = if !(strstr((buf1).as_mut_ptr(), c" Last>".as_ptr())).is_null() { 9 } else { 3 }; continue;
}
// C line 584
11 => {
let _ = get_field_buf((buf1).as_mut_ptr(), (size_of::<[c_char; 256]>() as usize), (line).as_mut_ptr(), (1 as i32));
vm_block = 10; continue;
}
// C line 580
12 => {
let _ = set_prop(((code) as u32), (PROP_Bidi_Mirrored as i32), (1 as i32));
vm_block = 11; continue;
}
// C line 579
13 => {
vm_block = if ((((!(p).is_null()) && (((((((*(p)) as i32)) == ((89 as i32))) as i32)) != 0)) as i32)) != 0 { 12 } else { 11 }; continue;
}
// C line 578
14 => {
let _ = { let assigned = get_field((line).as_mut_ptr(), (9 as i32)); p = assigned; assigned };
vm_block = 13; continue;
}
// C line 555
15 => {
vm_block = 19; continue;
}
// C line 560
16 => {
let _ = add_char(ptr::addr_of_mut!((*(ci)).decomp_data), ptr::addr_of_mut!(size), ptr::addr_of_mut!((*(ci)).decomp_len), ((strtoul(p, ((ptr::addr_of_mut!(p)) as *mut *mut c_char), (16 as i32))) as i32));
vm_block = 15; continue;
}
// C line 559
17 => {
vm_block = 14; continue;
}
// C line 558
18 => {
vm_block = if ((!((isxdigit(((*(p)) as i32))) != 0) as i32)) != 0 { 17 } else { 16 }; continue;
}
// C line 556
19 => {
vm_block = if (isspace(((*(p)) as i32))) != 0 { 20 } else { 18 }; continue;
}
// C line 557
20 => {
let _ = { let old = p; p = (p).offset(1); old };
vm_block = 19; continue;
}
// C line 554
21 => {
let _ = { let assigned = (0 as i32); size = assigned; assigned };
vm_block = 15; continue;
}
// C line 552
22 => {
let _ = { let assigned = (((1 as i32)) as u8); (*(ci)).set_is_compat((assigned) as _); assigned };
vm_block = 21; continue;
}
// C line 551
23 => {
let _ = { let old = p; p = (p).offset(1); old };
vm_block = 22; continue;
}
// C line 550
24 => {
vm_block = if ((((((*(p)) as i32)) == ((62 as i32))) as i32)) != 0 { 23 } else { 22 }; continue;
}
// C line 548
25 => {
vm_block = if ((((((((((*(p)) as i32)) != ((0 as i32))) as i32)) != 0) && (((((((*(p)) as i32)) != ((62 as i32))) as i32)) != 0)) as i32)) != 0 { 26 } else { 24 }; continue;
}
// C line 549
26 => {
let _ = { let old = p; p = (p).offset(1); old };
vm_block = 25; continue;
}
// C line 547
27 => {
vm_block = if ((((((*(p)) as i32)) == ((60 as i32))) as i32)) != 0 { 25 } else { 21 }; continue;
}
// C line 546
28 => {
let _ = { let assigned = (((0 as i32)) as u8); (*(ci)).set_is_compat((assigned) as _); assigned };
vm_block = 27; continue;
}
// C line 545
29 => {
let _ = if ((((!(((((code) <= ((1114111 as i32))) as i32)) != 0) as i32)) as i64)) != 0 { tool_assert_rtn(c"parse_unicode_data".as_ptr(), c"unicode_gen.c".as_ptr(), (545 as i32), c"code <= CHARCODE_MAX".as_ptr()) } else { { let _ = (0 as i32); } };
vm_block = 28; continue;
}
// C line 543
30 => {
vm_block = if ((((((((!(p).is_null()) && (((((((*(p)) as i32)) != ((59 as i32))) as i32)) != 0)) as i32)) != 0) && (((((((*(p)) as i32)) != ((0 as i32))) as i32)) != 0)) as i32)) != 0 { 29 } else { 14 }; continue;
}
// C line 542
31 => {
let _ = { let assigned = get_field((line).as_mut_ptr(), (5 as i32)); p = assigned; assigned };
vm_block = 30; continue;
}
// C line 537
32 => {
let _ = { let assigned = ((cc) as u8); (*(ci)).combining_class = assigned; assigned };
vm_block = 31; continue;
}
// C line 536
33 => {
let _ = if ((((!(((((code) <= ((1114111 as i32))) as i32)) != 0) as i32)) as i64)) != 0 { tool_assert_rtn(c"parse_unicode_data".as_ptr(), c"unicode_gen.c".as_ptr(), (536 as i32), c"code <= CHARCODE_MAX".as_ptr()) } else { { let _ = (0 as i32); } };
vm_block = 32; continue;
}
// C line 535
34 => {
vm_block = if ((((cc) != ((0 as i32))) as i32)) != 0 { 33 } else { 31 }; continue;
}
// C line 534
35 => {
let _ = { let assigned = ((strtoul(p, core::ptr::null_mut::<*mut c_char>(), (0 as i32))) as i32); cc = assigned; assigned };
vm_block = 34; continue;
}
// C line 532
36 => {
vm_block = if ((((((((!(p).is_null()) && (((((((*(p)) as i32)) != ((59 as i32))) as i32)) != 0)) as i32)) != 0) && (((((((*(p)) as i32)) != ((0 as i32))) as i32)) != 0)) as i32)) != 0 { 35 } else { 31 }; continue;
}
// C line 531
37 => {
let _ = { let assigned = get_field((line).as_mut_ptr(), (3 as i32)); p = assigned; assigned };
vm_block = 36; continue;
}
// C line 528
38 => {
let _ = { let assigned = ((i) as u8); (*(ci)).general_category = assigned; assigned };
vm_block = 37; continue;
}
// C line 526
39 => {
let _ = tool_exit((1 as i32));
vm_block = 38; continue;
}
// C line 524
40 => {
let _ = tool_fprintf(tool_stderr, c"General category '%s' not found\n".as_ptr(), &[ToolPrintArg::Str((buf1).as_mut_ptr() as *const c_char)]);
vm_block = 39; continue;
}
// C line 523
41 => {
vm_block = if ((((i) < ((0 as i32))) as i32)) != 0 { 40 } else { 38 }; continue;
}
// C line 522
42 => {
let _ = { let assigned = find_name((unicode_gc_name).as_mut_ptr(), (((((size_of::<[*const c_char; 38]>() as usize)) / ((size_of::<*const c_char>() as usize)))) as i32), (buf1).as_mut_ptr()); i = assigned; assigned };
vm_block = 41; continue;
}
// C line 521
43 => {
let _ = get_field_buf((buf1).as_mut_ptr(), (size_of::<[c_char; 256]>() as usize), (line).as_mut_ptr(), (2 as i32));
vm_block = 42; continue;
}
// C line 515
44 => {
let _ = { let assigned = lc; *(((*(ci)).l_data).as_mut_ptr()).offset(((0 as i32)) as isize) = assigned; assigned };
vm_block = 43; continue;
}
// C line 514
45 => {
let _ = { let assigned = (((1 as i32)) as u8); (*(ci)).l_len = assigned; assigned };
vm_block = 44; continue;
}
// C line 513
46 => {
let _ = if ((((!((((((((*(ci)).l_len) as i32)) == ((0 as i32))) as i32)) != 0) as i32)) as i64)) != 0 { tool_assert_rtn(c"parse_unicode_data".as_ptr(), c"unicode_gen.c".as_ptr(), (513 as i32), c"ci->l_len == 0".as_ptr()) } else { { let _ = (0 as i32); } };
vm_block = 45; continue;
}
// C line 512
47 => {
vm_block = if ((((lc) > ((0 as i32))) as i32)) != 0 { 46 } else { 43 }; continue;
}
// C line 510
48 => {
let _ = { let assigned = uc; *(((*(ci)).u_data).as_mut_ptr()).offset(((0 as i32)) as isize) = assigned; assigned };
vm_block = 47; continue;
}
// C line 509
49 => {
let _ = { let assigned = (((1 as i32)) as u8); (*(ci)).u_len = assigned; assigned };
vm_block = 48; continue;
}
// C line 508
50 => {
let _ = if ((((!((((((((*(ci)).u_len) as i32)) == ((0 as i32))) as i32)) != 0) as i32)) as i64)) != 0 { tool_assert_rtn(c"parse_unicode_data".as_ptr(), c"unicode_gen.c".as_ptr(), (508 as i32), c"ci->u_len == 0".as_ptr()) } else { { let _ = (0 as i32); } };
vm_block = 49; continue;
}
// C line 507
51 => {
vm_block = if ((((uc) > ((0 as i32))) as i32)) != 0 { 50 } else { 47 }; continue;
}
// C line 506
52 => {
let _ = if ((((!(((((code) <= ((1114111 as i32))) as i32)) != 0) as i32)) as i64)) != 0 { tool_assert_rtn(c"parse_unicode_data".as_ptr(), c"unicode_gen.c".as_ptr(), (506 as i32), c"code <= CHARCODE_MAX".as_ptr()) } else { { let _ = (0 as i32); } };
vm_block = 51; continue;
}
// C line 505
53 => {
vm_block = if ((((((((uc) > ((0 as i32))) as i32)) != 0) || (((((lc) > ((0 as i32))) as i32)) != 0)) as i32)) != 0 { 52 } else { 43 }; continue;
}
// C line 504
54 => {
let _ = { let assigned = ptr::addr_of_mut!(*(tab).offset((code) as isize)); ci = assigned; assigned };
vm_block = 53; continue;
}
// C line 502
55 => {
let _ = { let assigned = ((strtoul(p, core::ptr::null_mut::<*mut c_char>(), (16 as i32))) as i32); lc = assigned; assigned };
vm_block = 54; continue;
}
// C line 501
56 => {
vm_block = if ((((!(p).is_null()) && (((((((*(p)) as i32)) != ((59 as i32))) as i32)) != 0)) as i32)) != 0 { 55 } else { 54 }; continue;
}
// C line 500
57 => {
let _ = { let assigned = get_field((line).as_mut_ptr(), (13 as i32)); p = assigned; assigned };
vm_block = 56; continue;
}
// C line 497
58 => {
let _ = { let assigned = ((strtoul(p, core::ptr::null_mut::<*mut c_char>(), (16 as i32))) as i32); uc = assigned; assigned };
vm_block = 57; continue;
}
// C line 496
59 => {
vm_block = if ((((!(p).is_null()) && (((((((*(p)) as i32)) != ((59 as i32))) as i32)) != 0)) as i32)) != 0 { 58 } else { 57 }; continue;
}
// C line 495
60 => {
let _ = { let assigned = get_field((line).as_mut_ptr(), (12 as i32)); p = assigned; assigned };
vm_block = 59; continue;
}
// C line 493
61 => {
let _ = { let assigned = (0 as i32); uc = assigned; assigned };
vm_block = 60; continue;
}
// C line 492
62 => {
let _ = { let assigned = (0 as i32); lc = assigned; assigned };
vm_block = 61; continue;
}
// C line 491
63 => {
let _ = { let assigned = ((strtoul(p, core::ptr::null_mut::<*mut c_char>(), (16 as i32))) as i32); code = assigned; assigned };
vm_block = 62; continue;
}
// C line 490
64 => {
vm_block = 2; continue;
}
// C line 489
65 => {
vm_block = if ((!(!(p).is_null()) as i32)) != 0 { 64 } else { 63 }; continue;
}
// C line 488
66 => {
let _ = { let assigned = get_field((line).as_mut_ptr(), (0 as i32)); p = assigned; assigned };
vm_block = 65; continue;
}
// C line 486
67 => {
vm_block = 2; continue;
}
// C line 485
68 => {
vm_block = if ((((((*(p)) as i32)) == ((35 as i32))) as i32)) != 0 { 67 } else { 66 }; continue;
}
// C line 483
69 => {
vm_block = if (isspace(((*(p)) as i32))) != 0 { 70 } else { 68 }; continue;
}
// C line 484
70 => {
let _ = { let old = p; p = (p).offset(1); old };
vm_block = 69; continue;
}
// C line 482
71 => {
let _ = { let assigned = (line).as_mut_ptr(); p = assigned; assigned };
vm_block = 69; continue;
}
// C line 481
72 => {
vm_block = 1; continue;
}
// C line 480
73 => {
vm_block = if ((!(!(get_line((line).as_mut_ptr(), (((size_of::<[c_char; 1024]>() as usize)) as i32), f)).is_null()) as i32)) != 0 { 72 } else { 71 }; continue;
}
// C line 478
74 => {
let _ = { let assigned = (0 as i32); last_code = assigned; assigned };
vm_block = 2; continue;
}
// C line 475
75 => {
let _ = tool_exit((1 as i32));
vm_block = 74; continue;
}
// C line 474
76 => {
let _ = perror(filename);
vm_block = 75; continue;
}
// C line 473
77 => {
vm_block = if ((!(!(f).is_null()) as i32)) != 0 { 76 } else { 74 }; continue;
}
// C line 472
78 => {
let _ = { let assigned = fopen(filename, c"rb".as_ptr()); f = assigned; assigned };
vm_block = 77; continue;
}
// C line 470
79 => {
tab = unicode_db;
vm_block = 78; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: unicode_gen.c:600. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn parse_special_casing(mut tab: *mut CCInfo, mut filename: *const c_char) -> () {
let mut vm_local_storage = Vec::<u64>::new();
let mut f: *mut FILE = core::mem::zeroed();
let mut line: [c_char; 1024] = core::mem::zeroed();
let mut p: *const c_char = core::mem::zeroed();
let mut code: i32 = core::mem::zeroed();
let mut ci: *mut CCInfo = core::mem::zeroed();
let mut vm_block: usize = 49;
loop { match vm_block {
// C line ?
0 => {
return ();
}
// C line 673
1 => {
let _ = fclose(f);
vm_block = 0; continue;
}
// C line 614
2 => {
vm_block = 45; continue;
}
// C line 669
3 => {
let _ = { let assigned = (((0 as i32)) as u8); (*(ci)).u_len = assigned; assigned };
vm_block = 2; continue;
}
// C line 668
4 => {
vm_block = if (((((((((((*(ci)).u_len) as i32)) == ((1 as i32))) as i32)) != 0) && (((((*(((*(ci)).u_data).as_mut_ptr()).offset(((0 as i32)) as isize)) == (code)) as i32)) != 0)) as i32)) != 0 { 3 } else { 2 }; continue;
}
// C line 659
5 => {
vm_block = 10; continue;
}
// C line 665
6 => {
let _ = { let assigned = ((strtoul(p, ((ptr::addr_of_mut!(p)) as *mut *mut c_char), (16 as i32))) as i32); *(((*(ci)).u_data).as_mut_ptr()).offset(({ let old = (*(ci)).u_len; (*(ci)).u_len = ((*(ci)).u_len).wrapping_add(1); old }) as isize) = assigned; assigned };
vm_block = 5; continue;
}
// C line 664
7 => {
let _ = if ((((!((((((((*(ci)).u_len) as i32)) < ((3 as i32))) as i32)) != 0) as i32)) as i64)) != 0 { tool_assert_rtn(c"parse_special_casing".as_ptr(), c"unicode_gen.c".as_ptr(), (664 as i32), c"ci->u_len < CC_LEN_MAX".as_ptr()) } else { { let _ = (0 as i32); } };
vm_block = 6; continue;
}
// C line 663
8 => {
vm_block = 4; continue;
}
// C line 662
9 => {
vm_block = if ((((((*(p)) as i32)) == ((59 as i32))) as i32)) != 0 { 8 } else { 7 }; continue;
}
// C line 660
10 => {
vm_block = if (isspace(((*(p)) as i32))) != 0 { 11 } else { 9 }; continue;
}
// C line 661
11 => {
let _ = { let old = p; p = (p).offset(1); old };
vm_block = 10; continue;
}
// C line 658
12 => {
let _ = { let assigned = (((0 as i32)) as u8); (*(ci)).u_len = assigned; assigned };
vm_block = 5; continue;
}
// C line 657
13 => {
vm_block = if ((((!(p).is_null()) && (((((((*(p)) as i32)) != ((59 as i32))) as i32)) != 0)) as i32)) != 0 { 12 } else { 2 }; continue;
}
// C line 656
14 => {
let _ = { let assigned = get_field((line).as_mut_ptr(), (3 as i32)); p = assigned; assigned };
vm_block = 13; continue;
}
// C line 653
15 => {
let _ = { let assigned = (((0 as i32)) as u8); (*(ci)).l_len = assigned; assigned };
vm_block = 14; continue;
}
// C line 652
16 => {
vm_block = if (((((((((((*(ci)).l_len) as i32)) == ((1 as i32))) as i32)) != 0) && (((((*(((*(ci)).l_data).as_mut_ptr()).offset(((0 as i32)) as isize)) == (code)) as i32)) != 0)) as i32)) != 0 { 15 } else { 14 }; continue;
}
// C line 643
17 => {
vm_block = 22; continue;
}
// C line 649
18 => {
let _ = { let assigned = ((strtoul(p, ((ptr::addr_of_mut!(p)) as *mut *mut c_char), (16 as i32))) as i32); *(((*(ci)).l_data).as_mut_ptr()).offset(({ let old = (*(ci)).l_len; (*(ci)).l_len = ((*(ci)).l_len).wrapping_add(1); old }) as isize) = assigned; assigned };
vm_block = 17; continue;
}
// C line 648
19 => {
let _ = if ((((!((((((((*(ci)).l_len) as i32)) < ((3 as i32))) as i32)) != 0) as i32)) as i64)) != 0 { tool_assert_rtn(c"parse_special_casing".as_ptr(), c"unicode_gen.c".as_ptr(), (648 as i32), c"ci->l_len < CC_LEN_MAX".as_ptr()) } else { { let _ = (0 as i32); } };
vm_block = 18; continue;
}
// C line 647
20 => {
vm_block = 16; continue;
}
// C line 646
21 => {
vm_block = if ((((((*(p)) as i32)) == ((59 as i32))) as i32)) != 0 { 20 } else { 19 }; continue;
}
// C line 644
22 => {
vm_block = if (isspace(((*(p)) as i32))) != 0 { 23 } else { 21 }; continue;
}
// C line 645
23 => {
let _ = { let old = p; p = (p).offset(1); old };
vm_block = 22; continue;
}
// C line 642
24 => {
let _ = { let assigned = (((0 as i32)) as u8); (*(ci)).l_len = assigned; assigned };
vm_block = 17; continue;
}
// C line 641
25 => {
vm_block = if ((((!(p).is_null()) && (((((((*(p)) as i32)) != ((59 as i32))) as i32)) != 0)) as i32)) != 0 { 24 } else { 14 }; continue;
}
// C line 640
26 => {
let _ = { let assigned = get_field((line).as_mut_ptr(), (1 as i32)); p = assigned; assigned };
vm_block = 25; continue;
}
// C line 636
27 => {
vm_block = 2; continue;
}
// C line 635
28 => {
vm_block = if ((((((((((*(p)) as i32)) != ((35 as i32))) as i32)) != 0) && (((((((*(p)) as i32)) != ((0 as i32))) as i32)) != 0)) as i32)) != 0 { 27 } else { 26 }; continue;
}
// C line 633
29 => {
vm_block = if (isspace(((*(p)) as i32))) != 0 { 30 } else { 28 }; continue;
}
// C line 634
30 => {
let _ = { let old = p; p = (p).offset(1); old };
vm_block = 29; continue;
}
// C line 631
31 => {
vm_block = if !(p).is_null() { 29 } else { 26 }; continue;
}
// C line 630
32 => {
let _ = { let assigned = get_field((line).as_mut_ptr(), (4 as i32)); p = assigned; assigned };
vm_block = 31; continue;
}
// C line 628
33 => {
let _ = { let assigned = ptr::addr_of_mut!(*(tab).offset((code) as isize)); ci = assigned; assigned };
vm_block = 32; continue;
}
// C line 627
34 => {
let _ = if ((((!(((((code) <= ((1114111 as i32))) as i32)) != 0) as i32)) as i64)) != 0 { tool_assert_rtn(c"parse_special_casing".as_ptr(), c"unicode_gen.c".as_ptr(), (627 as i32), c"code <= CHARCODE_MAX".as_ptr()) } else { { let _ = (0 as i32); } };
vm_block = 33; continue;
}
// C line 626
35 => {
let _ = { let assigned = ((strtoul(p, core::ptr::null_mut::<*mut c_char>(), (16 as i32))) as i32); code = assigned; assigned };
vm_block = 34; continue;
}
// C line 625
36 => {
vm_block = 2; continue;
}
// C line 624
37 => {
vm_block = if ((!(!(p).is_null()) as i32)) != 0 { 36 } else { 35 }; continue;
}
// C line 623
38 => {
let _ = { let assigned = get_field((line).as_mut_ptr(), (0 as i32)); p = assigned; assigned };
vm_block = 37; continue;
}
// C line 621
39 => {
vm_block = 2; continue;
}
// C line 620
40 => {
vm_block = if ((((((*(p)) as i32)) == ((35 as i32))) as i32)) != 0 { 39 } else { 38 }; continue;
}
// C line 618
41 => {
vm_block = if (isspace(((*(p)) as i32))) != 0 { 42 } else { 40 }; continue;
}
// C line 619
42 => {
let _ = { let old = p; p = (p).offset(1); old };
vm_block = 41; continue;
}
// C line 617
43 => {
let _ = { let assigned = (line).as_mut_ptr(); p = assigned; assigned };
vm_block = 41; continue;
}
// C line 616
44 => {
vm_block = 1; continue;
}
// C line 615
45 => {
vm_block = if ((!(!(get_line((line).as_mut_ptr(), (((size_of::<[c_char; 1024]>() as usize)) as i32), f)).is_null()) as i32)) != 0 { 44 } else { 43 }; continue;
}
// C line 611
46 => {
let _ = tool_exit((1 as i32));
vm_block = 2; continue;
}
// C line 610
47 => {
let _ = perror(filename);
vm_block = 46; continue;
}
// C line 609
48 => {
vm_block = if ((!(!(f).is_null()) as i32)) != 0 { 47 } else { 2 }; continue;
}
// C line 608
49 => {
let _ = { let assigned = fopen(filename, c"rb".as_ptr()); f = assigned; assigned };
vm_block = 48; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: unicode_gen.c:676. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn parse_case_folding(mut tab: *mut CCInfo, mut filename: *const c_char) -> () {
let mut vm_local_storage = Vec::<u64>::new();
let mut f: *mut FILE = core::mem::zeroed();
let mut line: [c_char; 1024] = core::mem::zeroed();
let mut p: *const c_char = core::mem::zeroed();
let mut code: i32 = core::mem::zeroed();
let mut status: i32 = core::mem::zeroed();
let mut ci: *mut CCInfo = core::mem::zeroed();
let mut vm_block: usize = 40;
loop { match vm_block {
// C line ?
0 => {
return ();
}
// C line 736
1 => {
let _ = fclose(f);
vm_block = 0; continue;
}
// C line 690
2 => {
vm_block = 36; continue;
}
// C line 726
3 => {
vm_block = 8; continue;
}
// C line 732
4 => {
let _ = { let assigned = ((strtoul(p, ((ptr::addr_of_mut!(p)) as *mut *mut c_char), (16 as i32))) as i32); *(((*(ci)).f_data).as_mut_ptr()).offset(({ let old = (*(ci)).f_len; (*(ci)).f_len = ((*(ci)).f_len).wrapping_add(1); old }) as isize) = assigned; assigned };
vm_block = 3; continue;
}
// C line 731
5 => {
let _ = if ((((!((((((((*(ci)).l_len) as i32)) < ((3 as i32))) as i32)) != 0) as i32)) as i64)) != 0 { tool_assert_rtn(c"parse_case_folding".as_ptr(), c"unicode_gen.c".as_ptr(), (731 as i32), c"ci->l_len < CC_LEN_MAX".as_ptr()) } else { { let _ = (0 as i32); } };
vm_block = 4; continue;
}
// C line 730
6 => {
vm_block = 2; continue;
}
// C line 729
7 => {
vm_block = if ((((((*(p)) as i32)) == ((59 as i32))) as i32)) != 0 { 6 } else { 5 }; continue;
}
// C line 727
8 => {
vm_block = if (isspace(((*(p)) as i32))) != 0 { 9 } else { 7 }; continue;
}
// C line 728
9 => {
let _ = { let old = p; p = (p).offset(1); old };
vm_block = 8; continue;
}
// C line 722
10 => {
let _ = { let assigned = (((0 as i32)) as u8); (*(ci)).f_len = assigned; assigned };
vm_block = 3; continue;
}
// C line 721
11 => {
let _ = if ((((!((((((((*(ci)).f_len) as i32)) >= ((2 as i32))) as i32)) != 0) as i32)) as i64)) != 0 { tool_assert_rtn(c"parse_case_folding".as_ptr(), c"unicode_gen.c".as_ptr(), (721 as i32), c"ci->f_len >= 2".as_ptr()) } else { { let _ = (0 as i32); } };
vm_block = 10; continue;
}
// C line 724
12 => {
let _ = if ((((!((((((((*(ci)).f_len) as i32)) == ((0 as i32))) as i32)) != 0) as i32)) as i64)) != 0 { tool_assert_rtn(c"parse_case_folding".as_ptr(), c"unicode_gen.c".as_ptr(), (724 as i32), c"ci->f_len == 0".as_ptr()) } else { { let _ = (0 as i32); } };
vm_block = 3; continue;
}
// C line 718
13 => {
vm_block = if ((((status) == ((83 as i32))) as i32)) != 0 { 11 } else { 12 }; continue;
}
// C line 717
14 => {
let _ = if ((((!(((((p) != (core::ptr::null_mut::<c_char>())) as i32)) != 0) as i32)) as i64)) != 0 { tool_assert_rtn(c"parse_case_folding".as_ptr(), c"unicode_gen.c".as_ptr(), (717 as i32), c"p != NULL".as_ptr()) } else { { let _ = (0 as i32); } };
vm_block = 13; continue;
}
// C line 716
15 => {
let _ = { let assigned = get_field((line).as_mut_ptr(), (2 as i32)); p = assigned; assigned };
vm_block = 14; continue;
}
// C line 714
16 => {
vm_block = 2; continue;
}
// C line 713
17 => {
vm_block = if ((((((((((((status) != ((67 as i32))) as i32)) != 0) && (((((status) != ((83 as i32))) as i32)) != 0)) as i32)) != 0) && (((((status) != ((70 as i32))) as i32)) != 0)) as i32)) != 0 { 16 } else { 15 }; continue;
}
// C line 712
18 => {
let _ = { let assigned = ((*(p)) as i32); status = assigned; assigned };
vm_block = 17; continue;
}
// C line 710
19 => {
vm_block = if (isspace(((*(p)) as i32))) != 0 { 20 } else { 18 }; continue;
}
// C line 711
20 => {
let _ = { let old = p; p = (p).offset(1); old };
vm_block = 19; continue;
}
// C line 708
21 => {
vm_block = 2; continue;
}
// C line 707
22 => {
vm_block = if ((!(!(p).is_null()) as i32)) != 0 { 21 } else { 19 }; continue;
}
// C line 706
23 => {
let _ = { let assigned = get_field((line).as_mut_ptr(), (1 as i32)); p = assigned; assigned };
vm_block = 22; continue;
}
// C line 704
24 => {
let _ = { let assigned = ptr::addr_of_mut!(*(tab).offset((code) as isize)); ci = assigned; assigned };
vm_block = 23; continue;
}
// C line 703
25 => {
let _ = if ((((!(((((code) <= ((1114111 as i32))) as i32)) != 0) as i32)) as i64)) != 0 { tool_assert_rtn(c"parse_case_folding".as_ptr(), c"unicode_gen.c".as_ptr(), (703 as i32), c"code <= CHARCODE_MAX".as_ptr()) } else { { let _ = (0 as i32); } };
vm_block = 24; continue;
}
// C line 702
26 => {
let _ = { let assigned = ((strtoul(p, core::ptr::null_mut::<*mut c_char>(), (16 as i32))) as i32); code = assigned; assigned };
vm_block = 25; continue;
}
// C line 701
27 => {
vm_block = 2; continue;
}
// C line 700
28 => {
vm_block = if ((!(!(p).is_null()) as i32)) != 0 { 27 } else { 26 }; continue;
}
// C line 699
29 => {
let _ = { let assigned = get_field((line).as_mut_ptr(), (0 as i32)); p = assigned; assigned };
vm_block = 28; continue;
}
// C line 697
30 => {
vm_block = 2; continue;
}
// C line 696
31 => {
vm_block = if ((((((*(p)) as i32)) == ((35 as i32))) as i32)) != 0 { 30 } else { 29 }; continue;
}
// C line 694
32 => {
vm_block = if (isspace(((*(p)) as i32))) != 0 { 33 } else { 31 }; continue;
}
// C line 695
33 => {
let _ = { let old = p; p = (p).offset(1); old };
vm_block = 32; continue;
}
// C line 693
34 => {
let _ = { let assigned = (line).as_mut_ptr(); p = assigned; assigned };
vm_block = 32; continue;
}
// C line 692
35 => {
vm_block = 1; continue;
}
// C line 691
36 => {
vm_block = if ((!(!(get_line((line).as_mut_ptr(), (((size_of::<[c_char; 1024]>() as usize)) as i32), f)).is_null()) as i32)) != 0 { 35 } else { 34 }; continue;
}
// C line 687
37 => {
let _ = tool_exit((1 as i32));
vm_block = 2; continue;
}
// C line 686
38 => {
let _ = perror(filename);
vm_block = 37; continue;
}
// C line 685
39 => {
vm_block = if ((!(!(f).is_null()) as i32)) != 0 { 38 } else { 2 }; continue;
}
// C line 684
40 => {
let _ = { let assigned = fopen(filename, c"rb".as_ptr()); f = assigned; assigned };
vm_block = 39; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: unicode_gen.c:739. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn parse_composition_exclusions(mut filename: *const c_char) -> () {
let mut vm_local_storage = Vec::<u64>::new();
let mut f: *mut FILE = core::mem::zeroed();
let mut line: [c_char; 4096] = core::mem::zeroed();
let mut p: *mut c_char = core::mem::zeroed();
let mut c0: u32 = core::mem::zeroed();
let mut vm_block: usize = 16;
loop { match vm_block {
// C line ?
0 => {
return ();
}
// C line 763
1 => {
let _ = fclose(f);
vm_block = 0; continue;
}
// C line 751
2 => {
vm_block = 12; continue;
}
// C line 761
3 => {
let _ = { let assigned = (((TRUE as i32)) as u8); (*(unicode_db).offset((c0) as isize)).set_is_excluded((assigned) as _); assigned };
vm_block = 2; continue;
}
// C line 760
4 => {
let _ = if ((((!(((((((((c0) > ((((0 as i32)) as u32))) as i32)) != 0) && (((((c0) <= ((((1114111 as i32)) as u32))) as i32)) != 0)) as i32)) != 0) as i32)) as i64)) != 0 { tool_assert_rtn(c"parse_composition_exclusions".as_ptr(), c"unicode_gen.c".as_ptr(), (760 as i32), c"c0 > 0 && c0 <= CHARCODE_MAX".as_ptr()) } else { { let _ = (0 as i32); } };
vm_block = 3; continue;
}
// C line 759
5 => {
let _ = { let assigned = ((strtoul(p, ptr::addr_of_mut!(p), (16 as i32))) as u32); c0 = assigned; assigned };
vm_block = 4; continue;
}
// C line 758
6 => {
vm_block = 2; continue;
}
// C line 757
7 => {
vm_block = if ((((((((((((((*(p)) as i32)) == ((35 as i32))) as i32)) != 0) || (((((((*(p)) as i32)) == ((64 as i32))) as i32)) != 0)) as i32)) != 0) || (((((((*(p)) as i32)) == ((0 as i32))) as i32)) != 0)) as i32)) != 0 { 6 } else { 5 }; continue;
}
// C line 755
8 => {
vm_block = if (isspace(((*(p)) as i32))) != 0 { 9 } else { 7 }; continue;
}
// C line 756
9 => {
let _ = { let old = p; p = (p).offset(1); old };
vm_block = 8; continue;
}
// C line 754
10 => {
let _ = { let assigned = (line).as_mut_ptr(); p = assigned; assigned };
vm_block = 8; continue;
}
// C line 753
11 => {
vm_block = 1; continue;
}
// C line 752
12 => {
vm_block = if ((!(!(get_line((line).as_mut_ptr(), (((size_of::<[c_char; 4096]>() as usize)) as i32), f)).is_null()) as i32)) != 0 { 11 } else { 10 }; continue;
}
// C line 748
13 => {
let _ = tool_exit((1 as i32));
vm_block = 2; continue;
}
// C line 747
14 => {
let _ = perror(filename);
vm_block = 13; continue;
}
// C line 746
15 => {
vm_block = if ((!(!(f).is_null()) as i32)) != 0 { 14 } else { 2 }; continue;
}
// C line 745
16 => {
let _ = { let assigned = fopen(filename, c"rb".as_ptr()); f = assigned; assigned };
vm_block = 15; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: unicode_gen.c:766. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn parse_derived_core_properties(mut filename: *const c_char) -> () {
let mut vm_local_storage = Vec::<u64>::new();
let mut f: *mut FILE = core::mem::zeroed();
let mut line: [c_char; 4096] = core::mem::zeroed();
let mut p: *mut c_char = core::mem::zeroed();
let mut buf: [c_char; 256] = core::mem::zeroed();
let mut q: *mut c_char = core::mem::zeroed();
let mut c0: u32 = core::mem::zeroed();
let mut c1: u32 = core::mem::zeroed();
let mut c: u32 = core::mem::zeroed();
let mut i: i32 = core::mem::zeroed();
let mut vm_block: usize = 39;
loop { match vm_block {
// C line ?
0 => {
return ();
}
// C line 820
1 => {
let _ = fclose(f);
vm_block = 0; continue;
}
// C line 779 labels: next
2 => {
vm_block = 35; continue;
}
// C line 814
3 => {
vm_block = if ((((c) <= (c1)) as i32)) != 0 { 5 } else { 2 }; continue;
}
// C line ?
4 => {
let _ = { let old = c; c = (c).wrapping_add(1); old };
vm_block = 3; continue;
}
// C line 815
5 => {
let _ = set_prop(c, i, (1 as i32));
vm_block = 4; continue;
}
// C line 814
6 => {
let _ = { let assigned = c0; c = assigned; assigned };
vm_block = 3; continue;
}
// C line 812
7 => {
let _ = tool_exit((1 as i32));
vm_block = 6; continue;
}
// C line 811
8 => {
let _ = tool_fprintf(tool_stderr, c"Property not found: %s\n".as_ptr(), &[ToolPrintArg::Str((buf).as_mut_ptr() as *const c_char)]);
vm_block = 7; continue;
}
// C line 810
9 => {
vm_block = 2; continue;
}
// C line 809
10 => {
vm_block = if ((!((strcmp((buf).as_mut_ptr(), c"Grapheme_Link".as_ptr())) != 0) as i32)) != 0 { 9 } else { 8 }; continue;
}
// C line 808
11 => {
vm_block = if ((((i) < ((0 as i32))) as i32)) != 0 { 10 } else { 6 }; continue;
}
// C line 806
12 => {
let _ = { let assigned = find_name((unicode_prop_name).as_mut_ptr(), (((((size_of::<[*const c_char; 80]>() as usize)) / ((size_of::<*const c_char>() as usize)))) as i32), (buf).as_mut_ptr()); i = assigned; assigned };
vm_block = 11; continue;
}
// C line 805
13 => {
let _ = { let assigned = (((0 as i32)) as c_char); *(q) = assigned; assigned };
vm_block = 12; continue;
}
// C line 800
14 => {
vm_block = if ((((((((((((((((((((((*(p)) as i32)) != ((0 as i32))) as i32)) != 0) && (((((((*(p)) as i32)) != ((32 as i32))) as i32)) != 0)) as i32)) != 0) && (((((((*(p)) as i32)) != ((35 as i32))) as i32)) != 0)) as i32)) != 0) && (((((((*(p)) as i32)) != ((9 as i32))) as i32)) != 0)) as i32)) != 0) && (((((((*(p)) as i32)) != ((59 as i32))) as i32)) != 0)) as i32)) != 0 { 17 } else { 13 }; continue;
}
// C line 803
15 => {
let _ = { let old = p; p = (p).offset(1); old };
vm_block = 14; continue;
}
// C line 802
16 => {
let _ = { let assigned = *(p); *({ let old = q; q = (q).offset(1); old }) = assigned; assigned };
vm_block = 15; continue;
}
// C line 801
17 => {
vm_block = if ((((((((q).offset_from((buf).as_mut_ptr()) as i64)) as usize)) < (((size_of::<[c_char; 256]>() as usize)).wrapping_sub((((1 as i32)) as usize)))) as i32)) != 0 { 16 } else { 15 }; continue;
}
// C line 799
18 => {
let _ = { let assigned = (buf).as_mut_ptr(); q = assigned; assigned };
vm_block = 14; continue;
}
// C line 798
19 => {
let _ = { p = (p).offset(((strspn(p, c" \t".as_ptr())) as isize)); p };
vm_block = 18; continue;
}
// C line 797
20 => {
let _ = { let old = p; p = (p).offset(1); old };
vm_block = 19; continue;
}
// C line 796
21 => {
vm_block = if ((((((*(p)) as i32)) == ((59 as i32))) as i32)) != 0 { 20 } else { 2 }; continue;
}
// C line 795
22 => {
let _ = { p = (p).offset(((strspn(p, c" \t".as_ptr())) as isize)); p };
vm_block = 21; continue;
}
// C line 794
23 => {
let _ = if ((((!(((((c1) <= ((((1114111 as i32)) as u32))) as i32)) != 0) as i32)) as i64)) != 0 { tool_assert_rtn(c"parse_derived_core_properties".as_ptr(), c"unicode_gen.c".as_ptr(), (794 as i32), c"c1 <= CHARCODE_MAX".as_ptr()) } else { { let _ = (0 as i32); } };
vm_block = 22; continue;
}
// C line 790
24 => {
let _ = { let assigned = ((strtoul(p, ptr::addr_of_mut!(p), (16 as i32))) as u32); c1 = assigned; assigned };
vm_block = 23; continue;
}
// C line 789
25 => {
let _ = { p = (p).offset((((2 as i32)) as isize)); p };
vm_block = 24; continue;
}
// C line 792
26 => {
let _ = { let assigned = c0; c1 = assigned; assigned };
vm_block = 23; continue;
}
// C line 788
27 => {
vm_block = if ((((((((((*(p)) as i32)) == ((46 as i32))) as i32)) != 0) && (((((((*(p).offset(((1 as i32)) as isize)) as i32)) == ((46 as i32))) as i32)) != 0)) as i32)) != 0 { 25 } else { 26 }; continue;
}
// C line 787
28 => {
let _ = { let assigned = ((strtoul(p, ptr::addr_of_mut!(p), (16 as i32))) as u32); c0 = assigned; assigned };
vm_block = 27; continue;
}
// C line 786
29 => {
vm_block = 2; continue;
}
// C line 785
30 => {
vm_block = if ((((((((((((((*(p)) as i32)) == ((35 as i32))) as i32)) != 0) || (((((((*(p)) as i32)) == ((64 as i32))) as i32)) != 0)) as i32)) != 0) || (((((((*(p)) as i32)) == ((0 as i32))) as i32)) != 0)) as i32)) != 0 { 29 } else { 28 }; continue;
}
// C line 783
31 => {
vm_block = if (isspace(((*(p)) as i32))) != 0 { 32 } else { 30 }; continue;
}
// C line 784
32 => {
let _ = { let old = p; p = (p).offset(1); old };
vm_block = 31; continue;
}
// C line 782
33 => {
let _ = { let assigned = (line).as_mut_ptr(); p = assigned; assigned };
vm_block = 31; continue;
}
// C line 781
34 => {
vm_block = 1; continue;
}
// C line 780
35 => {
vm_block = if ((!(!(get_line((line).as_mut_ptr(), (((size_of::<[c_char; 4096]>() as usize)) as i32), f)).is_null()) as i32)) != 0 { 34 } else { 33 }; continue;
}
// C line 776
36 => {
let _ = tool_exit((1 as i32));
vm_block = 2; continue;
}
// C line 775
37 => {
let _ = perror(filename);
vm_block = 36; continue;
}
// C line 774
38 => {
vm_block = if ((!(!(f).is_null()) as i32)) != 0 { 37 } else { 2 }; continue;
}
// C line 773
39 => {
let _ = { let assigned = fopen(filename, c"rb".as_ptr()); f = assigned; assigned };
vm_block = 38; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: unicode_gen.c:823. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn parse_derived_norm_properties(mut filename: *const c_char) -> () {
let mut vm_local_storage = Vec::<u64>::new();
let mut f: *mut FILE = core::mem::zeroed();
let mut line: [c_char; 4096] = core::mem::zeroed();
let mut p: *mut c_char = core::mem::zeroed();
let mut buf: [c_char; 256] = core::mem::zeroed();
let mut q: *mut c_char = core::mem::zeroed();
let mut c0: u32 = core::mem::zeroed();
let mut c1: u32 = core::mem::zeroed();
let mut c: u32 = core::mem::zeroed();
let mut vm_block: usize = 34;
loop { match vm_block {
// C line ?
0 => {
return ();
}
// C line 869
1 => {
let _ = fclose(f);
vm_block = 0; continue;
}
// C line 835
2 => {
vm_block = 30; continue;
}
// C line 863
3 => {
vm_block = if ((((c) <= (c1)) as i32)) != 0 { 5 } else { 2 }; continue;
}
// C line ?
4 => {
let _ = { let old = c; c = (c).wrapping_add(1); old };
vm_block = 3; continue;
}
// C line 864
5 => {
let _ = set_prop(c, (PROP_Changes_When_NFKC_Casefolded as i32), (1 as i32));
vm_block = 4; continue;
}
// C line 863
6 => {
let _ = { let assigned = c0; c = assigned; assigned };
vm_block = 3; continue;
}
// C line 862
7 => {
vm_block = if ((!((strcmp((buf).as_mut_ptr(), c"Changes_When_NFKC_Casefolded".as_ptr())) != 0) as i32)) != 0 { 6 } else { 2 }; continue;
}
// C line 861
8 => {
let _ = { let assigned = (((0 as i32)) as c_char); *(q) = assigned; assigned };
vm_block = 7; continue;
}
// C line 856
9 => {
vm_block = if ((((((((((((((((((*(p)) as i32)) != ((0 as i32))) as i32)) != 0) && (((((((*(p)) as i32)) != ((32 as i32))) as i32)) != 0)) as i32)) != 0) && (((((((*(p)) as i32)) != ((35 as i32))) as i32)) != 0)) as i32)) != 0) && (((((((*(p)) as i32)) != ((9 as i32))) as i32)) != 0)) as i32)) != 0 { 12 } else { 8 }; continue;
}
// C line 859
10 => {
let _ = { let old = p; p = (p).offset(1); old };
vm_block = 9; continue;
}
// C line 858
11 => {
let _ = { let assigned = *(p); *({ let old = q; q = (q).offset(1); old }) = assigned; assigned };
vm_block = 10; continue;
}
// C line 857
12 => {
vm_block = if ((((((((q).offset_from((buf).as_mut_ptr()) as i64)) as usize)) < (((size_of::<[c_char; 256]>() as usize)).wrapping_sub((((1 as i32)) as usize)))) as i32)) != 0 { 11 } else { 10 }; continue;
}
// C line 855
13 => {
let _ = { let assigned = (buf).as_mut_ptr(); q = assigned; assigned };
vm_block = 9; continue;
}
// C line 854
14 => {
let _ = { p = (p).offset(((strspn(p, c" \t".as_ptr())) as isize)); p };
vm_block = 13; continue;
}
// C line 853
15 => {
let _ = { let old = p; p = (p).offset(1); old };
vm_block = 14; continue;
}
// C line 852
16 => {
vm_block = if ((((((*(p)) as i32)) == ((59 as i32))) as i32)) != 0 { 15 } else { 2 }; continue;
}
// C line 851
17 => {
let _ = { p = (p).offset(((strspn(p, c" \t".as_ptr())) as isize)); p };
vm_block = 16; continue;
}
// C line 850
18 => {
let _ = if ((((!(((((c1) <= ((((1114111 as i32)) as u32))) as i32)) != 0) as i32)) as i64)) != 0 { tool_assert_rtn(c"parse_derived_norm_properties".as_ptr(), c"unicode_gen.c".as_ptr(), (850 as i32), c"c1 <= CHARCODE_MAX".as_ptr()) } else { { let _ = (0 as i32); } };
vm_block = 17; continue;
}
// C line 846
19 => {
let _ = { let assigned = ((strtoul(p, ptr::addr_of_mut!(p), (16 as i32))) as u32); c1 = assigned; assigned };
vm_block = 18; continue;
}
// C line 845
20 => {
let _ = { p = (p).offset((((2 as i32)) as isize)); p };
vm_block = 19; continue;
}
// C line 848
21 => {
let _ = { let assigned = c0; c1 = assigned; assigned };
vm_block = 18; continue;
}
// C line 844
22 => {
vm_block = if ((((((((((*(p)) as i32)) == ((46 as i32))) as i32)) != 0) && (((((((*(p).offset(((1 as i32)) as isize)) as i32)) == ((46 as i32))) as i32)) != 0)) as i32)) != 0 { 20 } else { 21 }; continue;
}
// C line 843
23 => {
let _ = { let assigned = ((strtoul(p, ptr::addr_of_mut!(p), (16 as i32))) as u32); c0 = assigned; assigned };
vm_block = 22; continue;
}
// C line 842
24 => {
vm_block = 2; continue;
}
// C line 841
25 => {
vm_block = if ((((((((((((((*(p)) as i32)) == ((35 as i32))) as i32)) != 0) || (((((((*(p)) as i32)) == ((64 as i32))) as i32)) != 0)) as i32)) != 0) || (((((((*(p)) as i32)) == ((0 as i32))) as i32)) != 0)) as i32)) != 0 { 24 } else { 23 }; continue;
}
// C line 839
26 => {
vm_block = if (isspace(((*(p)) as i32))) != 0 { 27 } else { 25 }; continue;
}
// C line 840
27 => {
let _ = { let old = p; p = (p).offset(1); old };
vm_block = 26; continue;
}
// C line 838
28 => {
let _ = { let assigned = (line).as_mut_ptr(); p = assigned; assigned };
vm_block = 26; continue;
}
// C line 837
29 => {
vm_block = 1; continue;
}
// C line 836
30 => {
vm_block = if ((!(!(get_line((line).as_mut_ptr(), (((size_of::<[c_char; 4096]>() as usize)) as i32), f)).is_null()) as i32)) != 0 { 29 } else { 28 }; continue;
}
// C line 832
31 => {
let _ = tool_exit((1 as i32));
vm_block = 2; continue;
}
// C line 831
32 => {
let _ = perror(filename);
vm_block = 31; continue;
}
// C line 830
33 => {
vm_block = if ((!(!(f).is_null()) as i32)) != 0 { 32 } else { 2 }; continue;
}
// C line 829
34 => {
let _ = { let assigned = fopen(filename, c"rb".as_ptr()); f = assigned; assigned };
vm_block = 33; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: unicode_gen.c:872. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn parse_prop_list(mut filename: *const c_char) -> () {
let mut vm_local_storage = Vec::<u64>::new();
let mut f: *mut FILE = core::mem::zeroed();
let mut line: [c_char; 4096] = core::mem::zeroed();
let mut p: *mut c_char = core::mem::zeroed();
let mut buf: [c_char; 256] = core::mem::zeroed();
let mut q: *mut c_char = core::mem::zeroed();
let mut c0: u32 = core::mem::zeroed();
let mut c1: u32 = core::mem::zeroed();
let mut c: u32 = core::mem::zeroed();
let mut i: i32 = core::mem::zeroed();
let mut vm_block: usize = 37;
loop { match vm_block {
// C line ?
0 => {
return ();
}
// C line 923
1 => {
let _ = fclose(f);
vm_block = 0; continue;
}
// C line 885
2 => {
vm_block = 33; continue;
}
// C line 918
3 => {
vm_block = if ((((c) <= (c1)) as i32)) != 0 { 5 } else { 2 }; continue;
}
// C line ?
4 => {
let _ = { let old = c; c = (c).wrapping_add(1); old };
vm_block = 3; continue;
}
// C line 919
5 => {
let _ = set_prop(c, i, (1 as i32));
vm_block = 4; continue;
}
// C line 918
6 => {
let _ = { let assigned = c0; c = assigned; assigned };
vm_block = 3; continue;
}
// C line 916
7 => {
let _ = tool_exit((1 as i32));
vm_block = 6; continue;
}
// C line 915
8 => {
let _ = tool_fprintf(tool_stderr, c"Property not found: %s\n".as_ptr(), &[ToolPrintArg::Str((buf).as_mut_ptr() as *const c_char)]);
vm_block = 7; continue;
}
// C line 914
9 => {
vm_block = if ((((i) < ((0 as i32))) as i32)) != 0 { 8 } else { 6 }; continue;
}
// C line 912
10 => {
let _ = { let assigned = find_name((unicode_prop_name).as_mut_ptr(), (((((size_of::<[*const c_char; 80]>() as usize)) / ((size_of::<*const c_char>() as usize)))) as i32), (buf).as_mut_ptr()); i = assigned; assigned };
vm_block = 9; continue;
}
// C line 911
11 => {
let _ = { let assigned = (((0 as i32)) as c_char); *(q) = assigned; assigned };
vm_block = 10; continue;
}
// C line 906
12 => {
vm_block = if ((((((((((((((((((*(p)) as i32)) != ((0 as i32))) as i32)) != 0) && (((((((*(p)) as i32)) != ((32 as i32))) as i32)) != 0)) as i32)) != 0) && (((((((*(p)) as i32)) != ((35 as i32))) as i32)) != 0)) as i32)) != 0) && (((((((*(p)) as i32)) != ((9 as i32))) as i32)) != 0)) as i32)) != 0 { 15 } else { 11 }; continue;
}
// C line 909
13 => {
let _ = { let old = p; p = (p).offset(1); old };
vm_block = 12; continue;
}
// C line 908
14 => {
let _ = { let assigned = *(p); *({ let old = q; q = (q).offset(1); old }) = assigned; assigned };
vm_block = 13; continue;
}
// C line 907
15 => {
vm_block = if ((((((((q).offset_from((buf).as_mut_ptr()) as i64)) as usize)) < (((size_of::<[c_char; 256]>() as usize)).wrapping_sub((((1 as i32)) as usize)))) as i32)) != 0 { 14 } else { 13 }; continue;
}
// C line 905
16 => {
let _ = { let assigned = (buf).as_mut_ptr(); q = assigned; assigned };
vm_block = 12; continue;
}
// C line 904
17 => {
let _ = { p = (p).offset(((strspn(p, c" \t".as_ptr())) as isize)); p };
vm_block = 16; continue;
}
// C line 903
18 => {
let _ = { let old = p; p = (p).offset(1); old };
vm_block = 17; continue;
}
// C line 902
19 => {
vm_block = if ((((((*(p)) as i32)) == ((59 as i32))) as i32)) != 0 { 18 } else { 2 }; continue;
}
// C line 901
20 => {
let _ = { p = (p).offset(((strspn(p, c" \t".as_ptr())) as isize)); p };
vm_block = 19; continue;
}
// C line 900
21 => {
let _ = if ((((!(((((c1) <= ((((1114111 as i32)) as u32))) as i32)) != 0) as i32)) as i64)) != 0 { tool_assert_rtn(c"parse_prop_list".as_ptr(), c"unicode_gen.c".as_ptr(), (900 as i32), c"c1 <= CHARCODE_MAX".as_ptr()) } else { { let _ = (0 as i32); } };
vm_block = 20; continue;
}
// C line 896
22 => {
let _ = { let assigned = ((strtoul(p, ptr::addr_of_mut!(p), (16 as i32))) as u32); c1 = assigned; assigned };
vm_block = 21; continue;
}
// C line 895
23 => {
let _ = { p = (p).offset((((2 as i32)) as isize)); p };
vm_block = 22; continue;
}
// C line 898
24 => {
let _ = { let assigned = c0; c1 = assigned; assigned };
vm_block = 21; continue;
}
// C line 894
25 => {
vm_block = if ((((((((((*(p)) as i32)) == ((46 as i32))) as i32)) != 0) && (((((((*(p).offset(((1 as i32)) as isize)) as i32)) == ((46 as i32))) as i32)) != 0)) as i32)) != 0 { 23 } else { 24 }; continue;
}
// C line 893
26 => {
let _ = { let assigned = ((strtoul(p, ptr::addr_of_mut!(p), (16 as i32))) as u32); c0 = assigned; assigned };
vm_block = 25; continue;
}
// C line 892
27 => {
vm_block = 2; continue;
}
// C line 891
28 => {
vm_block = if ((((((((((((((*(p)) as i32)) == ((35 as i32))) as i32)) != 0) || (((((((*(p)) as i32)) == ((64 as i32))) as i32)) != 0)) as i32)) != 0) || (((((((*(p)) as i32)) == ((0 as i32))) as i32)) != 0)) as i32)) != 0 { 27 } else { 26 }; continue;
}
// C line 889
29 => {
vm_block = if (isspace(((*(p)) as i32))) != 0 { 30 } else { 28 }; continue;
}
// C line 890
30 => {
let _ = { let old = p; p = (p).offset(1); old };
vm_block = 29; continue;
}
// C line 888
31 => {
let _ = { let assigned = (line).as_mut_ptr(); p = assigned; assigned };
vm_block = 29; continue;
}
// C line 887
32 => {
vm_block = 1; continue;
}
// C line 886
33 => {
vm_block = if ((!(!(get_line((line).as_mut_ptr(), (((size_of::<[c_char; 4096]>() as usize)) as i32), f)).is_null()) as i32)) != 0 { 32 } else { 31 }; continue;
}
// C line 882
34 => {
let _ = tool_exit((1 as i32));
vm_block = 2; continue;
}
// C line 881
35 => {
let _ = perror(filename);
vm_block = 34; continue;
}
// C line 880
36 => {
vm_block = if ((!(!(f).is_null()) as i32)) != 0 { 35 } else { 2 }; continue;
}
// C line 879
37 => {
let _ = { let assigned = fopen(filename, c"rb".as_ptr()); f = assigned; assigned };
vm_block = 36; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: unicode_gen.c:928. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn is_emoji_modifier(mut c: u32) -> i32 {
let mut vm_local_storage = Vec::<u64>::new();
let mut vm_block: usize = 1;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 930
1 => {
return (((((((c) >= ((((127995 as i32)) as u32))) as i32)) != 0) && (((((c) <= ((((127999 as i32)) as u32))) as i32)) != 0)) as i32);
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: unicode_gen.c:933. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn add_sequence_prop(mut idx: i32, mut seq_len: i32, mut seq: *mut i32) -> () {
let mut vm_local_storage = Vec::<u64>::new();
let mut i: i32 = core::mem::zeroed();
let mut code: i32 = core::mem::zeroed();
let mut vm_block: usize = 37;
loop { match vm_block {
// C line ?
0 => {
return ();
}
// C line 988
1 => {
let _ = if ((((!(((0 as i32)) != 0) as i32)) as i64)) != 0 { tool_assert_rtn(c"add_sequence_prop".as_ptr(), c"unicode_gen.c".as_ptr(), (988 as i32), c"0".as_ptr()) } else { { let _ = (0 as i32); } };
vm_block = 0; continue;
}
// C line 986
2 => {
vm_block = 0; continue;
}
// C line 985
3 => {
let _ = set_prop(((*(seq).offset(((0 as i32)) as isize)) as u32), (PROP_Emoji_Keycap_Sequence as i32), (1 as i32));
vm_block = 2; continue;
}
// C line 984
4 => {
let _ = if ((((!(((((*(seq).offset(((2 as i32)) as isize)) == ((8419 as i32))) as i32)) != 0) as i32)) as i64)) != 0 { tool_assert_rtn(c"add_sequence_prop".as_ptr(), c"unicode_gen.c".as_ptr(), (984 as i32), c"seq[2] == 0x20e3".as_ptr()) } else { { let _ = (0 as i32); } };
vm_block = 3; continue;
}
// C line 983
5 => {
let _ = if ((((!(((((*(seq).offset(((1 as i32)) as isize)) == ((65039 as i32))) as i32)) != 0) as i32)) as i64)) != 0 { tool_assert_rtn(c"add_sequence_prop".as_ptr(), c"unicode_gen.c".as_ptr(), (983 as i32), c"seq[1] == 0xfe0f".as_ptr()) } else { { let _ = (0 as i32); } };
vm_block = 4; continue;
}
// C line 982
6 => {
let _ = if ((((!(((((seq_len) == ((3 as i32))) as i32)) != 0) as i32)) as i64)) != 0 { tool_assert_rtn(c"add_sequence_prop".as_ptr(), c"unicode_gen.c".as_ptr(), (982 as i32), c"seq_len == 3".as_ptr()) } else { { let _ = (0 as i32); } };
vm_block = 5; continue;
}
// C line 980
7 => {
vm_block = 0; continue;
}
// C line 978
8 => {
let _ = dbuf_putc(ptr::addr_of_mut!(rgi_emoji_tag_sequence), (((0 as i32)) as u8));
vm_block = 7; continue;
}
// C line 974
9 => {
vm_block = if ((((i) < ((seq_len).wrapping_sub((1 as i32)))) as i32)) != 0 { 12 } else { 8 }; continue;
}
// C line ?
10 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 9; continue;
}
// C line 976
11 => {
let _ = dbuf_putc(ptr::addr_of_mut!(rgi_emoji_tag_sequence), (((*(seq).offset((i) as isize)).wrapping_sub((917504 as i32))) as u8));
vm_block = 10; continue;
}
// C line 975
12 => {
let _ = if ((((!(((((((((*(seq).offset((i) as isize)) >= ((917505 as i32))) as i32)) != 0) && (((((*(seq).offset((i) as isize)) <= ((917630 as i32))) as i32)) != 0)) as i32)) != 0) as i32)) as i64)) != 0 { tool_assert_rtn(c"add_sequence_prop".as_ptr(), c"unicode_gen.c".as_ptr(), (975 as i32), c"seq[i] >= 0xe0001 && seq[i] <= 0xe007e".as_ptr()) } else { { let _ = (0 as i32); } };
vm_block = 11; continue;
}
// C line 974
13 => {
let _ = { let assigned = (1 as i32); i = assigned; assigned };
vm_block = 9; continue;
}
// C line 973
14 => {
let _ = if ((((!(((((*(seq).offset(((seq_len).wrapping_sub((1 as i32))) as isize)) == ((917631 as i32))) as i32)) != 0) as i32)) as i64)) != 0 { tool_assert_rtn(c"add_sequence_prop".as_ptr(), c"unicode_gen.c".as_ptr(), (973 as i32), c"seq[seq_len - 1] == 0xE007F".as_ptr()) } else { { let _ = (0 as i32); } };
vm_block = 13; continue;
}
// C line 972
15 => {
let _ = if ((((!(((((*(seq).offset(((0 as i32)) as isize)) == ((127988 as i32))) as i32)) != 0) as i32)) as i64)) != 0 { tool_assert_rtn(c"add_sequence_prop".as_ptr(), c"unicode_gen.c".as_ptr(), (972 as i32), c"seq[0] == 0x1F3F4".as_ptr()) } else { { let _ = (0 as i32); } };
vm_block = 14; continue;
}
// C line 971
16 => {
let _ = if ((((!(((((seq_len) >= ((3 as i32))) as i32)) != 0) as i32)) as i64)) != 0 { tool_assert_rtn(c"add_sequence_prop".as_ptr(), c"unicode_gen.c".as_ptr(), (971 as i32), c"seq_len >= 3".as_ptr()) } else { { let _ = (0 as i32); } };
vm_block = 15; continue;
}
// C line 968
17 => {
vm_block = 0; continue;
}
// C line 967
18 => {
let _ = re_string_add(ptr::addr_of_mut!(rgi_emoji_zwj_sequence), seq_len, ((seq) as *mut u32));
vm_block = 17; continue;
}
// C line 965
19 => {
vm_block = 0; continue;
}
// C line 963
20 => {
let _ = set_prop(((code) as u32), (PROP_RGI_Emoji_Flag_Sequence as i32), (1 as i32));
vm_block = 19; continue;
}
// C line 961
21 => {
let _ = { let assigned = (((*(seq).offset(((0 as i32)) as isize)).wrapping_sub((127462 as i32))).wrapping_mul((26 as i32))).wrapping_add((*(seq).offset(((1 as i32)) as isize)).wrapping_sub((127462 as i32))); code = assigned; assigned };
vm_block = 20; continue;
}
// C line 960
22 => {
let _ = if ((((!(((((((((*(seq).offset(((1 as i32)) as isize)) >= ((127462 as i32))) as i32)) != 0) && (((((*(seq).offset(((1 as i32)) as isize)) <= ((127487 as i32))) as i32)) != 0)) as i32)) != 0) as i32)) as i64)) != 0 { tool_assert_rtn(c"add_sequence_prop".as_ptr(), c"unicode_gen.c".as_ptr(), (960 as i32), c"seq[1] >= 0x1F1E6 && seq[1] <= 0x1F1FF".as_ptr()) } else { { let _ = (0 as i32); } };
vm_block = 21; continue;
}
// C line 959
23 => {
let _ = if ((((!(((((((((*(seq).offset(((0 as i32)) as isize)) >= ((127462 as i32))) as i32)) != 0) && (((((*(seq).offset(((0 as i32)) as isize)) <= ((127487 as i32))) as i32)) != 0)) as i32)) != 0) as i32)) as i64)) != 0 { tool_assert_rtn(c"add_sequence_prop".as_ptr(), c"unicode_gen.c".as_ptr(), (959 as i32), c"seq[0] >= 0x1F1E6 && seq[0] <= 0x1F1FF".as_ptr()) } else { { let _ = (0 as i32); } };
vm_block = 22; continue;
}
// C line 958
24 => {
let _ = if ((((!(((((seq_len) == ((2 as i32))) as i32)) != 0) as i32)) as i64)) != 0 { tool_assert_rtn(c"add_sequence_prop".as_ptr(), c"unicode_gen.c".as_ptr(), (958 as i32), c"seq_len == 2".as_ptr()) } else { { let _ = (0 as i32); } };
vm_block = 23; continue;
}
// C line 954
25 => {
vm_block = 0; continue;
}
// C line 953
26 => {
let _ = set_prop(((*(seq).offset(((0 as i32)) as isize)) as u32), (PROP_RGI_Emoji_Modifier_Sequence as i32), (1 as i32));
vm_block = 25; continue;
}
// C line 952
27 => {
let _ = if ((((!((get_prop(((*(seq).offset(((0 as i32)) as isize)) as u32), (PROP_Emoji_Modifier_Base as i32))) != 0) as i32)) as i64)) != 0 { tool_assert_rtn(c"add_sequence_prop".as_ptr(), c"unicode_gen.c".as_ptr(), (952 as i32), c"get_prop(seq[0], PROP_Emoji_Modifier_Base)".as_ptr()) } else { { let _ = (0 as i32); } };
vm_block = 26; continue;
}
// C line 951
28 => {
let _ = if ((((!((is_emoji_modifier(((*(seq).offset(((1 as i32)) as isize)) as u32))) != 0) as i32)) as i64)) != 0 { tool_assert_rtn(c"add_sequence_prop".as_ptr(), c"unicode_gen.c".as_ptr(), (951 as i32), c"is_emoji_modifier(seq[1])".as_ptr()) } else { { let _ = (0 as i32); } };
vm_block = 27; continue;
}
// C line 950
29 => {
let _ = if ((((!(((((seq_len) == ((2 as i32))) as i32)) != 0) as i32)) as i64)) != 0 { tool_assert_rtn(c"add_sequence_prop".as_ptr(), c"unicode_gen.c".as_ptr(), (950 as i32), c"seq_len == 2".as_ptr()) } else { { let _ = (0 as i32); } };
vm_block = 28; continue;
}
// C line 948
30 => {
vm_block = 0; continue;
}
// C line 942
31 => {
let _ = set_prop(((*(seq).offset(((0 as i32)) as isize)) as u32), (PROP_Basic_Emoji1 as i32), (1 as i32));
vm_block = 30; continue;
}
// C line 944
32 => {
let _ = set_prop(((*(seq).offset(((0 as i32)) as isize)) as u32), (PROP_Basic_Emoji2 as i32), (1 as i32));
vm_block = 30; continue;
}
// C line 946
33 => {
let _ = std::process::abort();
vm_block = 30; continue;
}
// C line 943
34 => {
vm_block = if ((((((((seq_len) == ((2 as i32))) as i32)) != 0) && (((((*(seq).offset(((1 as i32)) as isize)) == ((65039 as i32))) as i32)) != 0)) as i32)) != 0 { 32 } else { 33 }; continue;
}
// C line 941
35 => {
vm_block = if ((((seq_len) == ((1 as i32))) as i32)) != 0 { 31 } else { 34 }; continue;
}
// C line 938
36 => {
vm_block = match idx { x if x == (SEQUENCE_PROP_Emoji_Keycap_Sequence as i32) => 6, x if x == (SEQUENCE_PROP_RGI_Emoji_Tag_Sequence as i32) => 16, x if x == (SEQUENCE_PROP_RGI_Emoji_ZWJ_Sequence as i32) => 18, x if x == (SEQUENCE_PROP_RGI_Emoji_Flag_Sequence as i32) => 24, x if x == (SEQUENCE_PROP_RGI_Emoji_Modifier_Sequence as i32) => 29, x if x == (SEQUENCE_PROP_Basic_Emoji as i32) => 35, _ => 1, }; continue;
}
// C line 937
37 => {
let _ = if ((((!(((((idx) < ((SEQUENCE_PROP_COUNT as i32))) as i32)) != 0) as i32)) as i64)) != 0 { tool_assert_rtn(c"add_sequence_prop".as_ptr(), c"unicode_gen.c".as_ptr(), (937 as i32), c"idx < SEQUENCE_PROP_COUNT".as_ptr()) } else { { let _ = (0 as i32); } };
vm_block = 36; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: unicode_gen.c:992. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn parse_sequence_prop_list(mut filename: *const c_char) -> () {
let mut vm_local_storage = Vec::<u64>::new();
let mut f: *mut FILE = core::mem::zeroed();
let mut line: [c_char; 4096] = core::mem::zeroed();
let mut p: *mut c_char = core::mem::zeroed();
let mut buf: [c_char; 256] = core::mem::zeroed();
let mut q: *mut c_char = core::mem::zeroed();
let mut p_start: *mut c_char = core::mem::zeroed();
let mut c0: u32 = core::mem::zeroed();
let mut c1: u32 = core::mem::zeroed();
let mut c: u32 = core::mem::zeroed();
let mut idx: i32 = core::mem::zeroed();
let mut seq_len: i32 = core::mem::zeroed();
let mut seq: [i32; 16] = core::mem::zeroed();
let mut vm_block: usize = 53;
loop { match vm_block {
// C line ?
0 => {
return ();
}
// C line 1064
1 => {
let _ = fclose(f);
vm_block = 0; continue;
}
// C line 1006
2 => {
vm_block = 49; continue;
}
// C line 1044
3 => {
vm_block = if ((((c) <= (c1)) as i32)) != 0 { 6 } else { 2 }; continue;
}
// C line ?
4 => {
let _ = { let old = c; c = (c).wrapping_add(1); old };
vm_block = 3; continue;
}
// C line 1046
5 => {
let _ = add_sequence_prop(idx, (1 as i32), (seq).as_mut_ptr());
vm_block = 4; continue;
}
// C line 1045
6 => {
let _ = { let assigned = ((c) as i32); *((seq).as_mut_ptr()).offset(((0 as i32)) as isize) = assigned; assigned };
vm_block = 5; continue;
}
// C line 1044
7 => {
let _ = { let assigned = c0; c = assigned; assigned };
vm_block = 3; continue;
}
// C line 1043
8 => {
let _ = if ((((!(((((c1) <= ((((1114111 as i32)) as u32))) as i32)) != 0) as i32)) as i64)) != 0 { tool_assert_rtn(c"parse_sequence_prop_list".as_ptr(), c"unicode_gen.c".as_ptr(), (1043 as i32), c"c1 <= CHARCODE_MAX".as_ptr()) } else { { let _ = (0 as i32); } };
vm_block = 7; continue;
}
// C line 1042
9 => {
let _ = { let assigned = ((strtoul(p, ptr::addr_of_mut!(p), (16 as i32))) as u32); c1 = assigned; assigned };
vm_block = 8; continue;
}
// C line 1041
10 => {
let _ = { p = (p).offset((((2 as i32)) as isize)); p };
vm_block = 9; continue;
}
// C line 1061
11 => {
let _ = add_sequence_prop(idx, seq_len, (seq).as_mut_ptr());
vm_block = 2; continue;
}
// C line 1051
12 => {
vm_block = 19; continue;
}
// C line 1059
13 => {
let _ = { let assigned = ((c0) as i32); *((seq).as_mut_ptr()).offset(({ let old = seq_len; seq_len = (seq_len).wrapping_add(1); old }) as isize) = assigned; assigned };
vm_block = 12; continue;
}
// C line 1058
14 => {
let _ = if ((((!(((((((seq_len) as usize)) < ((((size_of::<[i32; 16]>() as usize)) / ((size_of::<i32>() as usize))))) as i32)) != 0) as i32)) as i64)) != 0 { tool_assert_rtn(c"parse_sequence_prop_list".as_ptr(), c"unicode_gen.c".as_ptr(), (1058 as i32), c"seq_len < countof(seq)".as_ptr()) } else { { let _ = (0 as i32); } };
vm_block = 13; continue;
}
// C line 1057
15 => {
let _ = if ((((!(((((c0) <= ((((1114111 as i32)) as u32))) as i32)) != 0) as i32)) as i64)) != 0 { tool_assert_rtn(c"parse_sequence_prop_list".as_ptr(), c"unicode_gen.c".as_ptr(), (1057 as i32), c"c0 <= CHARCODE_MAX".as_ptr()) } else { { let _ = (0 as i32); } };
vm_block = 14; continue;
}
// C line 1056
16 => {
let _ = { let assigned = ((strtoul(p, ptr::addr_of_mut!(p), (16 as i32))) as u32); c0 = assigned; assigned };
vm_block = 15; continue;
}
// C line 1055
17 => {
vm_block = 11; continue;
}
// C line 1054
18 => {
vm_block = if ((((((((((*(p)) as i32)) == ((59 as i32))) as i32)) != 0) || (((((((*(p)) as i32)) == ((0 as i32))) as i32)) != 0)) as i32)) != 0 { 17 } else { 16 }; continue;
}
// C line 1052
19 => {
vm_block = if (isspace(((*(p)) as i32))) != 0 { 20 } else { 18 }; continue;
}
// C line 1053
20 => {
let _ = { let old = p; p = (p).offset(1); old };
vm_block = 19; continue;
}
// C line 1050
21 => {
let _ = { let assigned = ((c0) as i32); *((seq).as_mut_ptr()).offset(({ let old = seq_len; seq_len = (seq_len).wrapping_add(1); old }) as isize) = assigned; assigned };
vm_block = 12; continue;
}
// C line 1049
22 => {
let _ = { let assigned = (0 as i32); seq_len = assigned; assigned };
vm_block = 21; continue;
}
// C line 1040
23 => {
vm_block = if ((((((((((*(p)) as i32)) == ((46 as i32))) as i32)) != 0) && (((((((*(p).offset(((1 as i32)) as isize)) as i32)) == ((46 as i32))) as i32)) != 0)) as i32)) != 0 { 10 } else { 22 }; continue;
}
// C line 1038
24 => {
let _ = if ((((!(((((c0) <= ((((1114111 as i32)) as u32))) as i32)) != 0) as i32)) as i64)) != 0 { tool_assert_rtn(c"parse_sequence_prop_list".as_ptr(), c"unicode_gen.c".as_ptr(), (1038 as i32), c"c0 <= CHARCODE_MAX".as_ptr()) } else { { let _ = (0 as i32); } };
vm_block = 23; continue;
}
// C line 1037
25 => {
let _ = { let assigned = ((strtoul(p, ptr::addr_of_mut!(p), (16 as i32))) as u32); c0 = assigned; assigned };
vm_block = 24; continue;
}
// C line 1036
26 => {
let _ = { let assigned = p_start; p = assigned; assigned };
vm_block = 25; continue;
}
// C line 1033
27 => {
let _ = tool_exit((1 as i32));
vm_block = 26; continue;
}
// C line 1032
28 => {
let _ = tool_fprintf(tool_stderr, c"Property not found: %s\n".as_ptr(), &[ToolPrintArg::Str((buf).as_mut_ptr() as *const c_char)]);
vm_block = 27; continue;
}
// C line 1031
29 => {
vm_block = if ((((idx) < ((0 as i32))) as i32)) != 0 { 28 } else { 26 }; continue;
}
// C line 1029
30 => {
let _ = { let assigned = find_name((unicode_sequence_prop_name).as_mut_ptr(), (((((size_of::<[*const c_char; 7]>() as usize)) / ((size_of::<*const c_char>() as usize)))) as i32), (buf).as_mut_ptr()); idx = assigned; assigned };
vm_block = 29; continue;
}
// C line 1028
31 => {
let _ = { let assigned = (((0 as i32)) as c_char); *(q) = assigned; assigned };
vm_block = 30; continue;
}
// C line 1023
32 => {
vm_block = if ((((((((((((((((((((((*(p)) as i32)) != ((0 as i32))) as i32)) != 0) && (((((((*(p)) as i32)) != ((32 as i32))) as i32)) != 0)) as i32)) != 0) && (((((((*(p)) as i32)) != ((35 as i32))) as i32)) != 0)) as i32)) != 0) && (((((((*(p)) as i32)) != ((9 as i32))) as i32)) != 0)) as i32)) != 0) && (((((((*(p)) as i32)) != ((59 as i32))) as i32)) != 0)) as i32)) != 0 { 35 } else { 31 }; continue;
}
// C line 1026
33 => {
let _ = { let old = p; p = (p).offset(1); old };
vm_block = 32; continue;
}
// C line 1025
34 => {
let _ = { let assigned = *(p); *({ let old = q; q = (q).offset(1); old }) = assigned; assigned };
vm_block = 33; continue;
}
// C line 1024
35 => {
vm_block = if ((((((((q).offset_from((buf).as_mut_ptr()) as i64)) as usize)) < (((size_of::<[c_char; 256]>() as usize)).wrapping_sub((((1 as i32)) as usize)))) as i32)) != 0 { 34 } else { 33 }; continue;
}
// C line 1022
36 => {
let _ = { let assigned = (buf).as_mut_ptr(); q = assigned; assigned };
vm_block = 32; continue;
}
// C line 1021
37 => {
let _ = { p = (p).offset(((strspn(p, c" \t".as_ptr())) as isize)); p };
vm_block = 36; continue;
}
// C line 1020
38 => {
let _ = { let old = p; p = (p).offset(1); old };
vm_block = 37; continue;
}
// C line 1019
39 => {
vm_block = 2; continue;
}
// C line 1018
40 => {
vm_block = if ((!(!(p).is_null()) as i32)) != 0 { 39 } else { 38 }; continue;
}
// C line 1017
41 => {
let _ = { let assigned = strchr(p, (59 as i32)); p = assigned; assigned };
vm_block = 40; continue;
}
// C line 1014
42 => {
let _ = { let assigned = p; p_start = assigned; assigned };
vm_block = 41; continue;
}
// C line 1013
43 => {
vm_block = 2; continue;
}
// C line 1012
44 => {
vm_block = if ((((((((((((((*(p)) as i32)) == ((35 as i32))) as i32)) != 0) || (((((((*(p)) as i32)) == ((64 as i32))) as i32)) != 0)) as i32)) != 0) || (((((((*(p)) as i32)) == ((0 as i32))) as i32)) != 0)) as i32)) != 0 { 43 } else { 42 }; continue;
}
// C line 1010
45 => {
vm_block = if (isspace(((*(p)) as i32))) != 0 { 46 } else { 44 }; continue;
}
// C line 1011
46 => {
let _ = { let old = p; p = (p).offset(1); old };
vm_block = 45; continue;
}
// C line 1009
47 => {
let _ = { let assigned = (line).as_mut_ptr(); p = assigned; assigned };
vm_block = 45; continue;
}
// C line 1008
48 => {
vm_block = 1; continue;
}
// C line 1007
49 => {
vm_block = if ((!(!(get_line((line).as_mut_ptr(), (((size_of::<[c_char; 4096]>() as usize)) as i32), f)).is_null()) as i32)) != 0 { 48 } else { 47 }; continue;
}
// C line 1003
50 => {
let _ = tool_exit((1 as i32));
vm_block = 2; continue;
}
// C line 1002
51 => {
let _ = perror(filename);
vm_block = 50; continue;
}
// C line 1001
52 => {
vm_block = if ((!(!(f).is_null()) as i32)) != 0 { 51 } else { 2 }; continue;
}
// C line 1000
53 => {
let _ = { let assigned = fopen(filename, c"rb".as_ptr()); f = assigned; assigned };
vm_block = 52; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: unicode_gen.c:1067. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn parse_scripts(mut filename: *const c_char) -> () {
let mut vm_local_storage = Vec::<u64>::new();
let mut f: *mut FILE = core::mem::zeroed();
let mut line: [c_char; 4096] = core::mem::zeroed();
let mut p: *mut c_char = core::mem::zeroed();
let mut buf: [c_char; 256] = core::mem::zeroed();
let mut q: *mut c_char = core::mem::zeroed();
let mut c0: u32 = core::mem::zeroed();
let mut c1: u32 = core::mem::zeroed();
let mut c: u32 = core::mem::zeroed();
let mut i: i32 = core::mem::zeroed();
let mut vm_block: usize = 37;
loop { match vm_block {
// C line ?
0 => {
return ();
}
// C line 1117
1 => {
let _ = fclose(f);
vm_block = 0; continue;
}
// C line 1080
2 => {
vm_block = 33; continue;
}
// C line 1113
3 => {
vm_block = if ((((c) <= (c1)) as i32)) != 0 { 5 } else { 2 }; continue;
}
// C line ?
4 => {
let _ = { let old = c; c = (c).wrapping_add(1); old };
vm_block = 3; continue;
}
// C line 1114
5 => {
let _ = { let assigned = ((i) as u8); (*(unicode_db).offset((c) as isize)).script = assigned; assigned };
vm_block = 4; continue;
}
// C line 1113
6 => {
let _ = { let assigned = c0; c = assigned; assigned };
vm_block = 3; continue;
}
// C line 1111
7 => {
let _ = tool_exit((1 as i32));
vm_block = 6; continue;
}
// C line 1110
8 => {
let _ = tool_fprintf(tool_stderr, c"Unknown script: '%s'\n".as_ptr(), &[ToolPrintArg::Str((buf).as_mut_ptr() as *const c_char)]);
vm_block = 7; continue;
}
// C line 1109
9 => {
vm_block = if ((((i) < ((0 as i32))) as i32)) != 0 { 8 } else { 6 }; continue;
}
// C line 1107
10 => {
let _ = { let assigned = find_name((unicode_script_name).as_mut_ptr(), (((((size_of::<[*const c_char; 176]>() as usize)) / ((size_of::<*const c_char>() as usize)))) as i32), (buf).as_mut_ptr()); i = assigned; assigned };
vm_block = 9; continue;
}
// C line 1106
11 => {
let _ = { let assigned = (((0 as i32)) as c_char); *(q) = assigned; assigned };
vm_block = 10; continue;
}
// C line 1101
12 => {
vm_block = if ((((((((((((((((((*(p)) as i32)) != ((0 as i32))) as i32)) != 0) && (((((((*(p)) as i32)) != ((32 as i32))) as i32)) != 0)) as i32)) != 0) && (((((((*(p)) as i32)) != ((35 as i32))) as i32)) != 0)) as i32)) != 0) && (((((((*(p)) as i32)) != ((9 as i32))) as i32)) != 0)) as i32)) != 0 { 15 } else { 11 }; continue;
}
// C line 1104
13 => {
let _ = { let old = p; p = (p).offset(1); old };
vm_block = 12; continue;
}
// C line 1103
14 => {
let _ = { let assigned = *(p); *({ let old = q; q = (q).offset(1); old }) = assigned; assigned };
vm_block = 13; continue;
}
// C line 1102
15 => {
vm_block = if ((((((((q).offset_from((buf).as_mut_ptr()) as i64)) as usize)) < (((size_of::<[c_char; 256]>() as usize)).wrapping_sub((((1 as i32)) as usize)))) as i32)) != 0 { 14 } else { 13 }; continue;
}
// C line 1100
16 => {
let _ = { let assigned = (buf).as_mut_ptr(); q = assigned; assigned };
vm_block = 12; continue;
}
// C line 1099
17 => {
let _ = { p = (p).offset(((strspn(p, c" \t".as_ptr())) as isize)); p };
vm_block = 16; continue;
}
// C line 1098
18 => {
let _ = { let old = p; p = (p).offset(1); old };
vm_block = 17; continue;
}
// C line 1097
19 => {
vm_block = if ((((((*(p)) as i32)) == ((59 as i32))) as i32)) != 0 { 18 } else { 2 }; continue;
}
// C line 1096
20 => {
let _ = { p = (p).offset(((strspn(p, c" \t".as_ptr())) as isize)); p };
vm_block = 19; continue;
}
// C line 1095
21 => {
let _ = if ((((!(((((c1) <= ((((1114111 as i32)) as u32))) as i32)) != 0) as i32)) as i64)) != 0 { tool_assert_rtn(c"parse_scripts".as_ptr(), c"unicode_gen.c".as_ptr(), (1095 as i32), c"c1 <= CHARCODE_MAX".as_ptr()) } else { { let _ = (0 as i32); } };
vm_block = 20; continue;
}
// C line 1091
22 => {
let _ = { let assigned = ((strtoul(p, ptr::addr_of_mut!(p), (16 as i32))) as u32); c1 = assigned; assigned };
vm_block = 21; continue;
}
// C line 1090
23 => {
let _ = { p = (p).offset((((2 as i32)) as isize)); p };
vm_block = 22; continue;
}
// C line 1093
24 => {
let _ = { let assigned = c0; c1 = assigned; assigned };
vm_block = 21; continue;
}
// C line 1089
25 => {
vm_block = if ((((((((((*(p)) as i32)) == ((46 as i32))) as i32)) != 0) && (((((((*(p).offset(((1 as i32)) as isize)) as i32)) == ((46 as i32))) as i32)) != 0)) as i32)) != 0 { 23 } else { 24 }; continue;
}
// C line 1088
26 => {
let _ = { let assigned = ((strtoul(p, ptr::addr_of_mut!(p), (16 as i32))) as u32); c0 = assigned; assigned };
vm_block = 25; continue;
}
// C line 1087
27 => {
vm_block = 2; continue;
}
// C line 1086
28 => {
vm_block = if ((((((((((((((*(p)) as i32)) == ((35 as i32))) as i32)) != 0) || (((((((*(p)) as i32)) == ((64 as i32))) as i32)) != 0)) as i32)) != 0) || (((((((*(p)) as i32)) == ((0 as i32))) as i32)) != 0)) as i32)) != 0 { 27 } else { 26 }; continue;
}
// C line 1084
29 => {
vm_block = if (isspace(((*(p)) as i32))) != 0 { 30 } else { 28 }; continue;
}
// C line 1085
30 => {
let _ = { let old = p; p = (p).offset(1); old };
vm_block = 29; continue;
}
// C line 1083
31 => {
let _ = { let assigned = (line).as_mut_ptr(); p = assigned; assigned };
vm_block = 29; continue;
}
// C line 1082
32 => {
vm_block = 1; continue;
}
// C line 1081
33 => {
vm_block = if ((!(!(get_line((line).as_mut_ptr(), (((size_of::<[c_char; 4096]>() as usize)) as i32), f)).is_null()) as i32)) != 0 { 32 } else { 31 }; continue;
}
// C line 1077
34 => {
let _ = tool_exit((1 as i32));
vm_block = 2; continue;
}
// C line 1076
35 => {
let _ = perror(filename);
vm_block = 34; continue;
}
// C line 1075
36 => {
vm_block = if ((!(!(f).is_null()) as i32)) != 0 { 35 } else { 2 }; continue;
}
// C line 1074
37 => {
let _ = { let assigned = fopen(filename, c"rb".as_ptr()); f = assigned; assigned };
vm_block = 36; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: unicode_gen.c:1120. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn parse_script_extensions(mut filename: *const c_char) -> () {
let mut vm_local_storage = Vec::<u64>::new();
let mut f: *mut FILE = core::mem::zeroed();
let mut line: [c_char; 4096] = core::mem::zeroed();
let mut p: *mut c_char = core::mem::zeroed();
let mut buf: [c_char; 256] = core::mem::zeroed();
let mut q: *mut c_char = core::mem::zeroed();
let mut c0: u32 = core::mem::zeroed();
let mut c1: u32 = core::mem::zeroed();
let mut c: u32 = core::mem::zeroed();
let mut i: i32 = core::mem::zeroed();
let mut script_ext: [u8; 255] = core::mem::zeroed();
let mut script_ext_len: i32 = core::mem::zeroed();
let mut ci: *mut CCInfo = core::mem::zeroed();
let mut vm_block: usize = 49;
loop { match vm_block {
// C line ?
0 => {
return ();
}
// C line 1184
1 => {
let _ = fclose(f);
vm_block = 0; continue;
}
// C line 1135
2 => {
vm_block = 45; continue;
}
// C line 1175
3 => {
vm_block = if ((((c) <= (c1)) as i32)) != 0 { 11 } else { 2 }; continue;
}
// C line ?
4 => {
let _ = { let old = c; c = (c).wrapping_add(1); old };
vm_block = 3; continue;
}
// C line 1179
5 => {
vm_block = if ((((i) < (script_ext_len)) as i32)) != 0 { 7 } else { 4 }; continue;
}
// C line ?
6 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 5; continue;
}
// C line 1180
7 => {
let _ = { let assigned = *((script_ext).as_mut_ptr()).offset((i) as isize); *((*(ci)).script_ext).offset((i) as isize) = assigned; assigned };
vm_block = 6; continue;
}
// C line 1179
8 => {
let _ = { let assigned = (0 as i32); i = assigned; assigned };
vm_block = 5; continue;
}
// C line 1178
9 => {
let _ = { let assigned = ((malloc(((size_of::<u8>() as usize)).wrapping_mul(((script_ext_len) as usize)))) as *mut u8); (*(ci)).script_ext = assigned; assigned };
vm_block = 8; continue;
}
// C line 1177
10 => {
let _ = { let assigned = ((script_ext_len) as u8); (*(ci)).script_ext_len = assigned; assigned };
vm_block = 9; continue;
}
// C line 1176
11 => {
ci = ptr::addr_of_mut!(*(unicode_db).offset((c) as isize));
vm_block = 10; continue;
}
// C line 1175
12 => {
let _ = { let assigned = c0; c = assigned; assigned };
vm_block = 3; continue;
}
// C line 1155
13 => {
vm_block = 28; continue;
}
// C line 1173
14 => {
let _ = { let assigned = ((i) as u8); *((script_ext).as_mut_ptr()).offset(({ let old = script_ext_len; script_ext_len = (script_ext_len).wrapping_add(1); old }) as isize) = assigned; assigned };
vm_block = 13; continue;
}
// C line 1172
15 => {
let _ = if ((((!(((((((script_ext_len) as usize)) < ((size_of::<[u8; 255]>() as usize))) as i32)) != 0) as i32)) as i64)) != 0 { tool_assert_rtn(c"parse_script_extensions".as_ptr(), c"unicode_gen.c".as_ptr(), (1172 as i32), c"script_ext_len < sizeof(script_ext)".as_ptr()) } else { { let _ = (0 as i32); } };
vm_block = 14; continue;
}
// C line 1170
16 => {
let _ = tool_exit((1 as i32));
vm_block = 15; continue;
}
// C line 1169
17 => {
let _ = tool_fprintf(tool_stderr, c"Script not found: %s\n".as_ptr(), &[ToolPrintArg::Str((buf).as_mut_ptr() as *const c_char)]);
vm_block = 16; continue;
}
// C line 1168
18 => {
vm_block = if ((((i) < ((0 as i32))) as i32)) != 0 { 17 } else { 15 }; continue;
}
// C line 1166
19 => {
let _ = { let assigned = find_name((unicode_script_short_name).as_mut_ptr(), (((((size_of::<[*const c_char; 176]>() as usize)) / ((size_of::<*const c_char>() as usize)))) as i32), (buf).as_mut_ptr()); i = assigned; assigned };
vm_block = 18; continue;
}
// C line 1165
20 => {
vm_block = 12; continue;
}
// C line 1164
21 => {
vm_block = if ((((((*((buf).as_mut_ptr()).offset(((0 as i32)) as isize)) as i32)) == ((0 as i32))) as i32)) != 0 { 20 } else { 19 }; continue;
}
// C line 1163
22 => {
let _ = { let assigned = (((0 as i32)) as c_char); *(q) = assigned; assigned };
vm_block = 21; continue;
}
// C line 1158
23 => {
vm_block = if ((((((((((((((((((*(p)) as i32)) != ((0 as i32))) as i32)) != 0) && (((((((*(p)) as i32)) != ((32 as i32))) as i32)) != 0)) as i32)) != 0) && (((((((*(p)) as i32)) != ((35 as i32))) as i32)) != 0)) as i32)) != 0) && (((((((*(p)) as i32)) != ((9 as i32))) as i32)) != 0)) as i32)) != 0 { 26 } else { 22 }; continue;
}
// C line 1161
24 => {
let _ = { let old = p; p = (p).offset(1); old };
vm_block = 23; continue;
}
// C line 1160
25 => {
let _ = { let assigned = *(p); *({ let old = q; q = (q).offset(1); old }) = assigned; assigned };
vm_block = 24; continue;
}
// C line 1159
26 => {
vm_block = if ((((((((q).offset_from((buf).as_mut_ptr()) as i64)) as usize)) < (((size_of::<[c_char; 256]>() as usize)).wrapping_sub((((1 as i32)) as usize)))) as i32)) != 0 { 25 } else { 24 }; continue;
}
// C line 1157
27 => {
let _ = { let assigned = (buf).as_mut_ptr(); q = assigned; assigned };
vm_block = 23; continue;
}
// C line 1156
28 => {
let _ = { p = (p).offset(((strspn(p, c" \t".as_ptr())) as isize)); p };
vm_block = 27; continue;
}
// C line 1154
29 => {
let _ = { let old = p; p = (p).offset(1); old };
vm_block = 13; continue;
}
// C line 1153
30 => {
vm_block = if ((((((*(p)) as i32)) == ((59 as i32))) as i32)) != 0 { 29 } else { 2 }; continue;
}
// C line 1152
31 => {
let _ = { let assigned = (0 as i32); script_ext_len = assigned; assigned };
vm_block = 30; continue;
}
// C line 1151
32 => {
let _ = { p = (p).offset(((strspn(p, c" \t".as_ptr())) as isize)); p };
vm_block = 31; continue;
}
// C line 1150
33 => {
let _ = if ((((!(((((c1) <= ((((1114111 as i32)) as u32))) as i32)) != 0) as i32)) as i64)) != 0 { tool_assert_rtn(c"parse_script_extensions".as_ptr(), c"unicode_gen.c".as_ptr(), (1150 as i32), c"c1 <= CHARCODE_MAX".as_ptr()) } else { { let _ = (0 as i32); } };
vm_block = 32; continue;
}
// C line 1146
34 => {
let _ = { let assigned = ((strtoul(p, ptr::addr_of_mut!(p), (16 as i32))) as u32); c1 = assigned; assigned };
vm_block = 33; continue;
}
// C line 1145
35 => {
let _ = { p = (p).offset((((2 as i32)) as isize)); p };
vm_block = 34; continue;
}
// C line 1148
36 => {
let _ = { let assigned = c0; c1 = assigned; assigned };
vm_block = 33; continue;
}
// C line 1144
37 => {
vm_block = if ((((((((((*(p)) as i32)) == ((46 as i32))) as i32)) != 0) && (((((((*(p).offset(((1 as i32)) as isize)) as i32)) == ((46 as i32))) as i32)) != 0)) as i32)) != 0 { 35 } else { 36 }; continue;
}
// C line 1143
38 => {
let _ = { let assigned = ((strtoul(p, ptr::addr_of_mut!(p), (16 as i32))) as u32); c0 = assigned; assigned };
vm_block = 37; continue;
}
// C line 1142
39 => {
vm_block = 2; continue;
}
// C line 1141
40 => {
vm_block = if ((((((((((((((*(p)) as i32)) == ((35 as i32))) as i32)) != 0) || (((((((*(p)) as i32)) == ((64 as i32))) as i32)) != 0)) as i32)) != 0) || (((((((*(p)) as i32)) == ((0 as i32))) as i32)) != 0)) as i32)) != 0 { 39 } else { 38 }; continue;
}
// C line 1139
41 => {
vm_block = if (isspace(((*(p)) as i32))) != 0 { 42 } else { 40 }; continue;
}
// C line 1140
42 => {
let _ = { let old = p; p = (p).offset(1); old };
vm_block = 41; continue;
}
// C line 1138
43 => {
let _ = { let assigned = (line).as_mut_ptr(); p = assigned; assigned };
vm_block = 41; continue;
}
// C line 1137
44 => {
vm_block = 1; continue;
}
// C line 1136
45 => {
vm_block = if ((!(!(get_line((line).as_mut_ptr(), (((size_of::<[c_char; 4096]>() as usize)) as i32), f)).is_null()) as i32)) != 0 { 44 } else { 43 }; continue;
}
// C line 1132
46 => {
let _ = tool_exit((1 as i32));
vm_block = 2; continue;
}
// C line 1131
47 => {
let _ = perror(filename);
vm_block = 46; continue;
}
// C line 1130
48 => {
vm_block = if ((!(!(f).is_null()) as i32)) != 0 { 47 } else { 2 }; continue;
}
// C line 1129
49 => {
let _ = { let assigned = fopen(filename, c"rb".as_ptr()); f = assigned; assigned };
vm_block = 48; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: unicode_gen.c:1187. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn dump_cc_info(mut ci: *mut CCInfo, mut i: i32) -> () {
let mut vm_local_storage = Vec::<u64>::new();
let mut j: i32 = core::mem::zeroed();
let mut vm_block: usize = 20;
loop { match vm_block {
// C line ?
0 => {
return ();
}
// C line 1206
1 => {
let _ = tool_printf(c"\n".as_ptr(), &[]);
vm_block = 0; continue;
}
// C line 1203
2 => {
vm_block = if ((((j) < ((((*(ci)).f_len) as i32))) as i32)) != 0 { 4 } else { 1 }; continue;
}
// C line ?
3 => {
let _ = { let old = j; j = (j).wrapping_add(1); old };
vm_block = 2; continue;
}
// C line 1204
4 => {
let _ = tool_printf(c" %05x".as_ptr(), &[ToolPrintArg::Int(*(((*(ci)).f_data).as_mut_ptr()).offset((j) as isize) as u64)]);
vm_block = 3; continue;
}
// C line 1203
5 => {
let _ = { let assigned = (0 as i32); j = assigned; assigned };
vm_block = 2; continue;
}
// C line 1202
6 => {
let _ = tool_printf(c" F:".as_ptr(), &[]);
vm_block = 5; continue;
}
// C line 1201
7 => {
vm_block = if (((((((*(ci)).f_len) as i32)) != ((0 as i32))) as i32)) != 0 { 6 } else { 1 }; continue;
}
// C line 1198
8 => {
vm_block = if ((((j) < ((((*(ci)).l_len) as i32))) as i32)) != 0 { 10 } else { 7 }; continue;
}
// C line ?
9 => {
let _ = { let old = j; j = (j).wrapping_add(1); old };
vm_block = 8; continue;
}
// C line 1199
10 => {
let _ = tool_printf(c" %05x".as_ptr(), &[ToolPrintArg::Int(*(((*(ci)).l_data).as_mut_ptr()).offset((j) as isize) as u64)]);
vm_block = 9; continue;
}
// C line 1198
11 => {
let _ = { let assigned = (0 as i32); j = assigned; assigned };
vm_block = 8; continue;
}
// C line 1197
12 => {
let _ = tool_printf(c" L:".as_ptr(), &[]);
vm_block = 11; continue;
}
// C line 1196
13 => {
vm_block = if (((((((*(ci)).l_len) as i32)) != ((0 as i32))) as i32)) != 0 { 12 } else { 7 }; continue;
}
// C line 1193
14 => {
vm_block = if ((((j) < ((((*(ci)).u_len) as i32))) as i32)) != 0 { 16 } else { 13 }; continue;
}
// C line ?
15 => {
let _ = { let old = j; j = (j).wrapping_add(1); old };
vm_block = 14; continue;
}
// C line 1194
16 => {
let _ = tool_printf(c" %05x".as_ptr(), &[ToolPrintArg::Int(*(((*(ci)).u_data).as_mut_ptr()).offset((j) as isize) as u64)]);
vm_block = 15; continue;
}
// C line 1193
17 => {
let _ = { let assigned = (0 as i32); j = assigned; assigned };
vm_block = 14; continue;
}
// C line 1192
18 => {
let _ = tool_printf(c" U:".as_ptr(), &[]);
vm_block = 17; continue;
}
// C line 1191
19 => {
vm_block = if (((((((*(ci)).u_len) as i32)) != ((0 as i32))) as i32)) != 0 { 18 } else { 13 }; continue;
}
// C line 1190
20 => {
let _ = tool_printf(c"%05x:".as_ptr(), &[ToolPrintArg::Int(i as u64)]);
vm_block = 19; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: unicode_gen.c:1209. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn dump_unicode_data(mut tab: *mut CCInfo) -> () {
let mut vm_local_storage = Vec::<u64>::new();
let mut i: i32 = core::mem::zeroed();
let mut ci: *mut CCInfo = core::mem::zeroed();
let mut vm_block: usize = 6;
loop { match vm_block {
// C line ?
0 => {
return ();
}
// C line 1213
1 => {
vm_block = if ((((i) <= ((1114111 as i32))) as i32)) != 0 { 5 } else { 0 }; continue;
}
// C line ?
2 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 1; continue;
}
// C line 1216
3 => {
let _ = dump_cc_info(ci, i);
vm_block = 2; continue;
}
// C line 1215
4 => {
vm_block = if (((((((((((((((*(ci)).u_len) as i32)) != ((0 as i32))) as i32)) != 0) || ((((((((*(ci)).l_len) as i32)) != ((0 as i32))) as i32)) != 0)) as i32)) != 0) || ((((((((*(ci)).f_len) as i32)) != ((0 as i32))) as i32)) != 0)) as i32)) != 0 { 3 } else { 2 }; continue;
}
// C line 1214
5 => {
let _ = { let assigned = ptr::addr_of_mut!(*(tab).offset((i) as isize)); ci = assigned; assigned };
vm_block = 4; continue;
}
// C line 1213
6 => {
let _ = { let assigned = (0 as i32); i = assigned; assigned };
vm_block = 1; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: unicode_gen.c:1221. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn is_complicated_case(mut ci: *const CCInfo) -> i32 {
let mut vm_local_storage = Vec::<u64>::new();
let mut vm_block: usize = 1;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 1223
1 => {
return ((((((((((((((((((((((*(ci)).u_len) as i32)) > ((1 as i32))) as i32)) != 0) || ((((((((*(ci)).l_len) as i32)) > ((1 as i32))) as i32)) != 0)) as i32)) != 0) || ((((((((((((*(ci)).u_len) as i32)) > ((0 as i32))) as i32)) != 0) && ((((((((*(ci)).l_len) as i32)) > ((0 as i32))) as i32)) != 0)) as i32)) != 0)) as i32)) != 0) || ((((((((*(ci)).f_len) as i32)) != ((((*(ci)).l_len) as i32))) as i32)) != 0)) as i32)) != 0) || (((((memcmp(((((*(ci)).f_data).as_ptr()) as *const c_void), ((((*(ci)).l_data).as_ptr()) as *const c_void), ((((*(ci)).f_len) as usize)).wrapping_mul((size_of::<i32>() as usize)))) != ((0 as i32))) as i32)) != 0)) as i32);
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: unicode_gen.c:1275. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn simple_to_lower(mut tab: *mut CCInfo, mut c: i32) -> i32 {
let mut vm_local_storage = Vec::<u64>::new();
let mut vm_block: usize = 3;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 1279
1 => {
return *(((*(tab).offset((c) as isize)).l_data).as_mut_ptr()).offset(((0 as i32)) as isize);
}
// C line 1278
2 => {
return c;
}
// C line 1277
3 => {
vm_block = if (((((((*(tab).offset((c) as isize)).l_len) as i32)) != ((1 as i32))) as i32)) != 0 { 2 } else { 1 }; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: unicode_gen.c:1284. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn find_run_type(mut te: *mut TableEntry, mut tab: *mut CCInfo, mut code: i32) -> () {
let mut vm_local_storage = Vec::<u64>::new();
let mut is_lower: i32 = core::mem::zeroed();
let mut len: i32 = core::mem::zeroed();
let mut ci: *mut CCInfo = core::mem::zeroed();
let mut ci1: *mut CCInfo = core::mem::zeroed();
let mut ci2: *mut CCInfo = core::mem::zeroed();
let mut vm_block: usize = 141;
loop { match vm_block {
// C line ?
0 => {
return ();
}
// C line 1399
1 => {
let _ = { let assigned = *(((*(ci)).u_data).as_mut_ptr()).offset(((0 as i32)) as isize); (*(te)).data = assigned; assigned };
vm_block = 0; continue;
}
// C line 1398
2 => {
let _ = { let assigned = (RUN_TYPE_UF_D20 as i32); (*(te)).v_type = assigned; assigned };
vm_block = 1; continue;
}
// C line 1397
3 => {
let _ = { let assigned = (1 as i32); (*(te)).len = assigned; assigned };
vm_block = 2; continue;
}
// C line 1406
4 => {
let _ = { let assigned = (1 as i32); (*(te)).ext_len = assigned; assigned };
vm_block = 0; continue;
}
// C line 1405
5 => {
let _ = { let assigned = *(((*(ci)).u_data).as_mut_ptr()).offset(((0 as i32)) as isize); *(((*(te)).ext_data).as_mut_ptr()).offset(((0 as i32)) as isize) = assigned; assigned };
vm_block = 4; continue;
}
// C line 1404
6 => {
let _ = { let assigned = (RUN_TYPE_UF_D1_EXT as i32); (*(te)).v_type = assigned; assigned };
vm_block = 5; continue;
}
// C line 1403
7 => {
let _ = { let assigned = (1 as i32); (*(te)).len = assigned; assigned };
vm_block = 6; continue;
}
// C line 1414
8 => {
let _ = { let assigned = (2 as i32); (*(te)).ext_len = assigned; assigned };
vm_block = 0; continue;
}
// C line 1413
9 => {
let _ = { let assigned = *(((*(ci)).l_data).as_mut_ptr()).offset(((1 as i32)) as isize); *(((*(te)).ext_data).as_mut_ptr()).offset(((1 as i32)) as isize) = assigned; assigned };
vm_block = 8; continue;
}
// C line 1412
10 => {
let _ = { let assigned = *(((*(ci)).l_data).as_mut_ptr()).offset(((0 as i32)) as isize); *(((*(te)).ext_data).as_mut_ptr()).offset(((0 as i32)) as isize) = assigned; assigned };
vm_block = 9; continue;
}
// C line 1411
11 => {
let _ = { let assigned = (RUN_TYPE_LF_EXT2 as i32); (*(te)).v_type = assigned; assigned };
vm_block = 10; continue;
}
// C line 1410
12 => {
let _ = { let assigned = (1 as i32); (*(te)).len = assigned; assigned };
vm_block = 11; continue;
}
// C line 1422
13 => {
let _ = { let assigned = (2 as i32); (*(te)).ext_len = assigned; assigned };
vm_block = 0; continue;
}
// C line 1421
14 => {
let _ = { let assigned = *(((*(ci)).u_data).as_mut_ptr()).offset(((1 as i32)) as isize); *(((*(te)).ext_data).as_mut_ptr()).offset(((1 as i32)) as isize) = assigned; assigned };
vm_block = 13; continue;
}
// C line 1420
15 => {
let _ = { let assigned = *(((*(ci)).u_data).as_mut_ptr()).offset(((0 as i32)) as isize); *(((*(te)).ext_data).as_mut_ptr()).offset(((0 as i32)) as isize) = assigned; assigned };
vm_block = 14; continue;
}
// C line 1419
16 => {
let _ = { let assigned = (RUN_TYPE_UF_EXT2 as i32); (*(te)).v_type = assigned; assigned };
vm_block = 15; continue;
}
// C line 1418
17 => {
let _ = { let assigned = (1 as i32); (*(te)).len = assigned; assigned };
vm_block = 16; continue;
}
// C line 1432
18 => {
let _ = { let assigned = (3 as i32); (*(te)).ext_len = assigned; assigned };
vm_block = 0; continue;
}
// C line 1431
19 => {
let _ = { let assigned = *(((*(ci)).u_data).as_mut_ptr()).offset(((2 as i32)) as isize); *(((*(te)).ext_data).as_mut_ptr()).offset(((2 as i32)) as isize) = assigned; assigned };
vm_block = 18; continue;
}
// C line 1430
20 => {
let _ = { let assigned = *(((*(ci)).u_data).as_mut_ptr()).offset(((1 as i32)) as isize); *(((*(te)).ext_data).as_mut_ptr()).offset(((1 as i32)) as isize) = assigned; assigned };
vm_block = 19; continue;
}
// C line 1429
21 => {
let _ = { let assigned = *(((*(ci)).u_data).as_mut_ptr()).offset(((0 as i32)) as isize); *(((*(te)).ext_data).as_mut_ptr()).offset(((0 as i32)) as isize) = assigned; assigned };
vm_block = 20; continue;
}
// C line 1428
22 => {
let _ = { let assigned = (RUN_TYPE_UF_EXT3 as i32); (*(te)).v_type = assigned; assigned };
vm_block = 21; continue;
}
// C line 1427
23 => {
let _ = { let assigned = (1 as i32); (*(te)).len = assigned; assigned };
vm_block = 22; continue;
}
// C line 1440
24 => {
let _ = { let assigned = (2 as i32); (*(te)).ext_len = assigned; assigned };
vm_block = 0; continue;
}
// C line 1439
25 => {
let _ = { let assigned = *(((*(ci)).u_data).as_mut_ptr()).offset(((1 as i32)) as isize); *(((*(te)).ext_data).as_mut_ptr()).offset(((1 as i32)) as isize) = assigned; assigned };
vm_block = 24; continue;
}
// C line 1438
26 => {
let _ = { let assigned = *(((*(ci)).u_data).as_mut_ptr()).offset(((0 as i32)) as isize); *(((*(te)).ext_data).as_mut_ptr()).offset(((0 as i32)) as isize) = assigned; assigned };
vm_block = 25; continue;
}
// C line 1437
27 => {
let _ = { let assigned = (RUN_TYPE_UF_EXT2 as i32); (*(te)).v_type = assigned; assigned };
vm_block = 26; continue;
}
// C line 1436
28 => {
let _ = { let assigned = (1 as i32); (*(te)).len = assigned; assigned };
vm_block = 27; continue;
}
// C line 1435
29 => {
let _ = if ((((!(((((code) == ((64261 as i32))) as i32)) != 0) as i32)) as i64)) != 0 { tool_assert_rtn(c"find_run_type".as_ptr(), c"unicode_gen.c".as_ptr(), (1435 as i32), c"code == 0xFB05".as_ptr()) } else { { let _ = (0 as i32); } };
vm_block = 28; continue;
}
// C line 1450
30 => {
let _ = { let assigned = (3 as i32); (*(te)).ext_len = assigned; assigned };
vm_block = 0; continue;
}
// C line 1449
31 => {
let _ = { let assigned = *(((*(ci)).u_data).as_mut_ptr()).offset(((2 as i32)) as isize); *(((*(te)).ext_data).as_mut_ptr()).offset(((2 as i32)) as isize) = assigned; assigned };
vm_block = 30; continue;
}
// C line 1448
32 => {
let _ = { let assigned = *(((*(ci)).u_data).as_mut_ptr()).offset(((1 as i32)) as isize); *(((*(te)).ext_data).as_mut_ptr()).offset(((1 as i32)) as isize) = assigned; assigned };
vm_block = 31; continue;
}
// C line 1447
33 => {
let _ = { let assigned = *(((*(ci)).u_data).as_mut_ptr()).offset(((0 as i32)) as isize); *(((*(te)).ext_data).as_mut_ptr()).offset(((0 as i32)) as isize) = assigned; assigned };
vm_block = 32; continue;
}
// C line 1446
34 => {
let _ = { let assigned = (RUN_TYPE_UF_EXT3 as i32); (*(te)).v_type = assigned; assigned };
vm_block = 33; continue;
}
// C line 1445
35 => {
let _ = { let assigned = (1 as i32); (*(te)).len = assigned; assigned };
vm_block = 34; continue;
}
// C line 1444
36 => {
let _ = if ((((!(((((((((code) == ((8147 as i32))) as i32)) != 0) || (((((code) == ((8163 as i32))) as i32)) != 0)) as i32)) != 0) as i32)) as i64)) != 0 { tool_assert_rtn(c"find_run_type".as_ptr(), c"unicode_gen.c".as_ptr(), (1444 as i32), c"code == 0x1FD3 || code == 0x1FE3".as_ptr()) } else { { let _ = (0 as i32); } };
vm_block = 35; continue;
}
// C line 1454
37 => {
let _ = std::process::abort();
vm_block = 0; continue;
}
// C line 1453
38 => {
let _ = dump_cc_info(ci, code);
vm_block = 37; continue;
}
// C line 1452
39 => {
let _ = tool_printf(c"unsupported encoding case:\n".as_ptr(), &[]);
vm_block = 38; continue;
}
// C line 1441
40 => {
vm_block = if (((((((((((((((*(ci)).u_len) as i32)) == ((3 as i32))) as i32)) != 0) && ((((((((*(ci)).l_len) as i32)) == ((0 as i32))) as i32)) != 0)) as i32)) != 0) && ((((((((*(ci)).f_len) as i32)) == ((1 as i32))) as i32)) != 0)) as i32)) != 0 { 36 } else { 39 }; continue;
}
// C line 1433
41 => {
vm_block = if (((((((((((((((*(ci)).u_len) as i32)) == ((2 as i32))) as i32)) != 0) && ((((((((*(ci)).l_len) as i32)) == ((0 as i32))) as i32)) != 0)) as i32)) != 0) && ((((((((*(ci)).f_len) as i32)) == ((1 as i32))) as i32)) != 0)) as i32)) != 0 { 29 } else { 40 }; continue;
}
// C line 1423
42 => {
vm_block = if (((((((((((((((((((((((((((*(ci)).u_len) as i32)) == ((3 as i32))) as i32)) != 0) && ((((((((*(ci)).l_len) as i32)) == ((0 as i32))) as i32)) != 0)) as i32)) != 0) && ((((((((*(ci)).f_len) as i32)) == ((3 as i32))) as i32)) != 0)) as i32)) != 0) && (((((*(((*(ci)).f_data).as_mut_ptr()).offset(((0 as i32)) as isize)) == (simple_to_lower(tab, *(((*(ci)).u_data).as_mut_ptr()).offset(((0 as i32)) as isize)))) as i32)) != 0)) as i32)) != 0) && (((((*(((*(ci)).f_data).as_mut_ptr()).offset(((1 as i32)) as isize)) == (simple_to_lower(tab, *(((*(ci)).u_data).as_mut_ptr()).offset(((1 as i32)) as isize)))) as i32)) != 0)) as i32)) != 0) && (((((*(((*(ci)).f_data).as_mut_ptr()).offset(((2 as i32)) as isize)) == (simple_to_lower(tab, *(((*(ci)).u_data).as_mut_ptr()).offset(((2 as i32)) as isize)))) as i32)) != 0)) as i32)) != 0 { 23 } else { 41 }; continue;
}
// C line 1415
43 => {
vm_block = if (((((((((((((((((((((((*(ci)).u_len) as i32)) == ((2 as i32))) as i32)) != 0) && ((((((((*(ci)).l_len) as i32)) == ((0 as i32))) as i32)) != 0)) as i32)) != 0) && ((((((((*(ci)).f_len) as i32)) == ((2 as i32))) as i32)) != 0)) as i32)) != 0) && (((((*(((*(ci)).f_data).as_mut_ptr()).offset(((0 as i32)) as isize)) == (simple_to_lower(tab, *(((*(ci)).u_data).as_mut_ptr()).offset(((0 as i32)) as isize)))) as i32)) != 0)) as i32)) != 0) && (((((*(((*(ci)).f_data).as_mut_ptr()).offset(((1 as i32)) as isize)) == (simple_to_lower(tab, *(((*(ci)).u_data).as_mut_ptr()).offset(((1 as i32)) as isize)))) as i32)) != 0)) as i32)) != 0 { 17 } else { 42 }; continue;
}
// C line 1407
44 => {
vm_block = if (((((((((((((((((((((((*(ci)).l_len) as i32)) == ((2 as i32))) as i32)) != 0) && ((((((((*(ci)).u_len) as i32)) == ((0 as i32))) as i32)) != 0)) as i32)) != 0) && ((((((((*(ci)).f_len) as i32)) == ((2 as i32))) as i32)) != 0)) as i32)) != 0) && (((((*(((*(ci)).l_data).as_mut_ptr()).offset(((0 as i32)) as isize)) == (*(((*(ci)).f_data).as_mut_ptr()).offset(((0 as i32)) as isize))) as i32)) != 0)) as i32)) != 0) && (((((*(((*(ci)).l_data).as_mut_ptr()).offset(((1 as i32)) as isize)) == (*(((*(ci)).f_data).as_mut_ptr()).offset(((1 as i32)) as isize))) as i32)) != 0)) as i32)) != 0 { 12 } else { 43 }; continue;
}
// C line 1400
45 => {
vm_block = if (((((((((((((((((((*(ci)).l_len) as i32)) == ((0 as i32))) as i32)) != 0) && ((((((((*(ci)).u_len) as i32)) == ((1 as i32))) as i32)) != 0)) as i32)) != 0) && ((((((((*(ci)).f_len) as i32)) == ((1 as i32))) as i32)) != 0)) as i32)) != 0) && (((((*(((*(ci)).f_data).as_mut_ptr()).offset(((0 as i32)) as isize)) == ((*(((*(ci)).u_data).as_mut_ptr()).offset(((0 as i32)) as isize)).wrapping_add((1 as i32)))) as i32)) != 0)) as i32)) != 0 { 7 } else { 44 }; continue;
}
// C line 1393
46 => {
vm_block = if (((((((((((((((((((((((*(ci)).l_len) as i32)) == ((0 as i32))) as i32)) != 0) && ((((((((*(ci)).u_len) as i32)) == ((1 as i32))) as i32)) != 0)) as i32)) != 0) && (((((*(((*(ci)).u_data).as_mut_ptr()).offset(((0 as i32)) as isize)) < ((4096 as i32))) as i32)) != 0)) as i32)) != 0) && ((((((((*(ci)).f_len) as i32)) == ((1 as i32))) as i32)) != 0)) as i32)) != 0) && (((((*(((*(ci)).f_data).as_mut_ptr()).offset(((0 as i32)) as isize)) == ((*(((*(ci)).u_data).as_mut_ptr()).offset(((0 as i32)) as isize)).wrapping_add((32 as i32)))) as i32)) != 0)) as i32)) != 0 { 3 } else { 45 }; continue;
}
// C line 1390
47 => {
return;
}
// C line 1389
48 => {
let _ = { let assigned = *(((*(ci)).l_data).as_mut_ptr()).offset(((0 as i32)) as isize); (*(te)).data = assigned; assigned };
vm_block = 47; continue;
}
// C line 1388
49 => {
let _ = { let assigned = (RUN_TYPE_L as i32); (*(te)).v_type = assigned; assigned };
vm_block = 48; continue;
}
// C line 1387
50 => {
let _ = { let assigned = len; (*(te)).len = assigned; assigned };
vm_block = 49; continue;
}
// C line 1379
51 => {
vm_block = if (((((code).wrapping_add(len)) <= ((1114111 as i32))) as i32)) != 0 { 55 } else { 50 }; continue;
}
// C line 1385
52 => {
let _ = { let old = len; len = (len).wrapping_add(1); old };
vm_block = 51; continue;
}
// C line 1384
53 => {
vm_block = 50; continue;
}
// C line 1381
54 => {
vm_block = if ((!((((((((((((((((((((*(ci1)).l_len) as i32)) == ((1 as i32))) as i32)) != 0) && (((((*(((*(ci1)).l_data).as_mut_ptr()).offset(((0 as i32)) as isize)) == ((*(((*(ci)).l_data).as_mut_ptr()).offset(((0 as i32)) as isize)).wrapping_add(len))) as i32)) != 0)) as i32)) != 0) && ((((((((*(ci1)).u_len) as i32)) == ((0 as i32))) as i32)) != 0)) as i32)) != 0) && ((((((((*(ci1)).f_len) as i32)) == ((0 as i32))) as i32)) != 0)) as i32)) != 0) as i32)) != 0 { 53 } else { 52 }; continue;
}
// C line 1380
55 => {
let _ = { let assigned = ptr::addr_of_mut!(*(tab).offset(((code).wrapping_add(len)) as isize)); ci1 = assigned; assigned };
vm_block = 54; continue;
}
// C line 1378
56 => {
let _ = { let assigned = (1 as i32); len = assigned; assigned };
vm_block = 51; continue;
}
// C line 1377
57 => {
vm_block = if (((((((((((((((*(ci)).l_len) as i32)) == ((1 as i32))) as i32)) != 0) && ((((((((*(ci)).u_len) as i32)) == ((0 as i32))) as i32)) != 0)) as i32)) != 0) && ((((((((*(ci)).f_len) as i32)) == ((0 as i32))) as i32)) != 0)) as i32)) != 0 { 56 } else { 46 }; continue;
}
// C line 1374
58 => {
return;
}
// C line 1373
59 => {
let _ = { let assigned = (2 as i32); (*(te)).ext_len = assigned; assigned };
vm_block = 58; continue;
}
// C line 1372
60 => {
let _ = { let assigned = *(((*(ci)).l_data).as_mut_ptr()).offset(((0 as i32)) as isize); *(((*(te)).ext_data).as_mut_ptr()).offset(((1 as i32)) as isize) = assigned; assigned };
vm_block = 59; continue;
}
// C line 1371
61 => {
let _ = { let assigned = *(((*(ci)).u_data).as_mut_ptr()).offset(((0 as i32)) as isize); *(((*(te)).ext_data).as_mut_ptr()).offset(((0 as i32)) as isize) = assigned; assigned };
vm_block = 60; continue;
}
// C line 1370
62 => {
let _ = { let assigned = (RUN_TYPE_U2L_399_EXT2 as i32); (*(te)).v_type = assigned; assigned };
vm_block = 61; continue;
}
// C line 1369
63 => {
let _ = { let assigned = len; (*(te)).len = assigned; assigned };
vm_block = 62; continue;
}
// C line 1358
64 => {
vm_block = if (((((code).wrapping_add(len)) <= ((1114111 as i32))) as i32)) != 0 { 68 } else { 63 }; continue;
}
// C line 1367
65 => {
let _ = { let old = len; len = (len).wrapping_add(1); old };
vm_block = 64; continue;
}
// C line 1366
66 => {
vm_block = 63; continue;
}
// C line 1360
67 => {
vm_block = if ((!((((((((((((((((((((((((((((((((*(ci1)).u_len) as i32)) == ((2 as i32))) as i32)) != 0) && (((((*(((*(ci1)).u_data).as_mut_ptr()).offset(((1 as i32)) as isize)) == ((921 as i32))) as i32)) != 0)) as i32)) != 0) && (((((*(((*(ci1)).u_data).as_mut_ptr()).offset(((0 as i32)) as isize)) == ((*(((*(ci)).u_data).as_mut_ptr()).offset(((0 as i32)) as isize)).wrapping_add(len))) as i32)) != 0)) as i32)) != 0) && ((((((((*(ci1)).l_len) as i32)) == ((1 as i32))) as i32)) != 0)) as i32)) != 0) && (((((*(((*(ci1)).l_data).as_mut_ptr()).offset(((0 as i32)) as isize)) == ((*(((*(ci)).l_data).as_mut_ptr()).offset(((0 as i32)) as isize)).wrapping_add(len))) as i32)) != 0)) as i32)) != 0) && ((((((((*(ci1)).f_len) as i32)) == ((1 as i32))) as i32)) != 0)) as i32)) != 0) && (((((*(((*(ci1)).f_data).as_mut_ptr()).offset(((0 as i32)) as isize)) == (*(((*(ci1)).l_data).as_mut_ptr()).offset(((0 as i32)) as isize))) as i32)) != 0)) as i32)) != 0) as i32)) != 0 { 66 } else { 65 }; continue;
}
// C line 1359
68 => {
let _ = { let assigned = ptr::addr_of_mut!(*(tab).offset(((code).wrapping_add(len)) as isize)); ci1 = assigned; assigned };
vm_block = 67; continue;
}
// C line 1357
69 => {
let _ = { let assigned = (1 as i32); len = assigned; assigned };
vm_block = 64; continue;
}
// C line 1354
70 => {
vm_block = if (((((((((((((((((((((((*(ci)).u_len) as i32)) == ((2 as i32))) as i32)) != 0) && (((((*(((*(ci)).u_data).as_mut_ptr()).offset(((1 as i32)) as isize)) == ((921 as i32))) as i32)) != 0)) as i32)) != 0) && ((((((((*(ci)).l_len) as i32)) == ((1 as i32))) as i32)) != 0)) as i32)) != 0) && ((((((((*(ci)).f_len) as i32)) == ((1 as i32))) as i32)) != 0)) as i32)) != 0) && (((((*(((*(ci)).f_data).as_mut_ptr()).offset(((0 as i32)) as isize)) == (*(((*(ci)).l_data).as_mut_ptr()).offset(((0 as i32)) as isize))) as i32)) != 0)) as i32)) != 0 { 69 } else { 57 }; continue;
}
// C line 1351
71 => {
return;
}
// C line 1350
72 => {
let _ = { let assigned = (2 as i32); (*(te)).ext_len = assigned; assigned };
vm_block = 71; continue;
}
// C line 1349
73 => {
let _ = { let assigned = *(((*(ci)).u_data).as_mut_ptr()).offset(((1 as i32)) as isize); *(((*(te)).ext_data).as_mut_ptr()).offset(((1 as i32)) as isize) = assigned; assigned };
vm_block = 72; continue;
}
// C line 1348
74 => {
let _ = { let assigned = *(((*(ci)).u_data).as_mut_ptr()).offset(((0 as i32)) as isize); *(((*(te)).ext_data).as_mut_ptr()).offset(((0 as i32)) as isize) = assigned; assigned };
vm_block = 73; continue;
}
// C line 1347
75 => {
let _ = { let assigned = (RUN_TYPE_UF_EXT2 as i32); (*(te)).v_type = assigned; assigned };
vm_block = 74; continue;
}
// C line 1346
76 => {
let _ = { let assigned = len; (*(te)).len = assigned; assigned };
vm_block = 75; continue;
}
// C line 1334
77 => {
vm_block = if (((((code).wrapping_add(len)) <= ((1114111 as i32))) as i32)) != 0 { 81 } else { 76 }; continue;
}
// C line 1344
78 => {
let _ = { let old = len; len = (len).wrapping_add(1); old };
vm_block = 77; continue;
}
// C line 1343
79 => {
vm_block = 76; continue;
}
// C line 1336
80 => {
vm_block = if ((!((((((((((((((((((((((((((((((((*(ci1)).u_len) as i32)) == ((2 as i32))) as i32)) != 0) && (((((*(((*(ci1)).u_data).as_mut_ptr()).offset(((1 as i32)) as isize)) == (*(((*(ci)).u_data).as_mut_ptr()).offset(((1 as i32)) as isize))) as i32)) != 0)) as i32)) != 0) && (((((*(((*(ci1)).u_data).as_mut_ptr()).offset(((0 as i32)) as isize)) == ((*(((*(ci)).u_data).as_mut_ptr()).offset(((0 as i32)) as isize)).wrapping_add(len))) as i32)) != 0)) as i32)) != 0) && ((((((((*(ci1)).f_len) as i32)) == ((2 as i32))) as i32)) != 0)) as i32)) != 0) && (((((*(((*(ci1)).f_data).as_mut_ptr()).offset(((1 as i32)) as isize)) == (*(((*(ci)).f_data).as_mut_ptr()).offset(((1 as i32)) as isize))) as i32)) != 0)) as i32)) != 0) && (((((*(((*(ci1)).f_data).as_mut_ptr()).offset(((0 as i32)) as isize)) == ((*(((*(ci)).f_data).as_mut_ptr()).offset(((0 as i32)) as isize)).wrapping_add(len))) as i32)) != 0)) as i32)) != 0) && ((((((((*(ci1)).l_len) as i32)) == ((0 as i32))) as i32)) != 0)) as i32)) != 0) as i32)) != 0 { 79 } else { 78 }; continue;
}
// C line 1335
81 => {
let _ = { let assigned = ptr::addr_of_mut!(*(tab).offset(((code).wrapping_add(len)) as isize)); ci1 = assigned; assigned };
vm_block = 80; continue;
}
// C line 1333
82 => {
let _ = { let assigned = (1 as i32); len = assigned; assigned };
vm_block = 77; continue;
}
// C line 1329
83 => {
vm_block = if (((((((((((((((((((((((((((*(ci)).l_len) as i32)) == ((0 as i32))) as i32)) != 0) && ((((((((*(ci)).u_len) as i32)) == ((2 as i32))) as i32)) != 0)) as i32)) != 0) && (((((*(((*(ci)).u_data).as_mut_ptr()).offset(((1 as i32)) as isize)) == ((921 as i32))) as i32)) != 0)) as i32)) != 0) && ((((((((*(ci)).f_len) as i32)) == ((2 as i32))) as i32)) != 0)) as i32)) != 0) && (((((*(((*(ci)).f_data).as_mut_ptr()).offset(((1 as i32)) as isize)) == ((953 as i32))) as i32)) != 0)) as i32)) != 0) && (((((*(((*(ci)).f_data).as_mut_ptr()).offset(((0 as i32)) as isize)) == (simple_to_lower(tab, *(((*(ci)).u_data).as_mut_ptr()).offset(((0 as i32)) as isize)))) as i32)) != 0)) as i32)) != 0 { 82 } else { 70 }; continue;
}
// C line 1326
84 => {
return;
}
// C line 1325
85 => {
let _ = { let assigned = *(((*(ci)).u_data).as_mut_ptr()).offset(((0 as i32)) as isize); (*(te)).data = assigned; assigned };
vm_block = 84; continue;
}
// C line 1324
86 => {
let _ = { let assigned = (RUN_TYPE_UF as i32); (*(te)).v_type = assigned; assigned };
vm_block = 85; continue;
}
// C line 1323
87 => {
let _ = { let assigned = len; (*(te)).len = assigned; assigned };
vm_block = 86; continue;
}
// C line 1322
88 => {
vm_block = if ((((len) > ((1 as i32))) as i32)) != 0 { 87 } else { 83 }; continue;
}
// C line 1313
89 => {
vm_block = if (((((code).wrapping_add(len)) <= ((1114111 as i32))) as i32)) != 0 { 93 } else { 88 }; continue;
}
// C line 1320
90 => {
let _ = { let old = len; len = (len).wrapping_add(1); old };
vm_block = 89; continue;
}
// C line 1319
91 => {
vm_block = 88; continue;
}
// C line 1315
92 => {
vm_block = if (((((((((((((((((((((((*(ci1)).u_len) as i32)) != ((1 as i32))) as i32)) != 0) || (((((*(((*(ci1)).u_data).as_mut_ptr()).offset(((0 as i32)) as isize)) != ((*(((*(ci)).u_data).as_mut_ptr()).offset(((0 as i32)) as isize)).wrapping_add(len))) as i32)) != 0)) as i32)) != 0) || ((((((((*(ci1)).l_len) as i32)) != ((0 as i32))) as i32)) != 0)) as i32)) != 0) || ((((((((*(ci1)).f_len) as i32)) != ((1 as i32))) as i32)) != 0)) as i32)) != 0) || (((((*(((*(ci1)).f_data).as_mut_ptr()).offset(((0 as i32)) as isize)) != (*(((*(ci1)).u_data).as_mut_ptr()).offset(((0 as i32)) as isize))) as i32)) != 0)) as i32)) != 0 { 91 } else { 90 }; continue;
}
// C line 1314
93 => {
let _ = { let assigned = ptr::addr_of_mut!(*(tab).offset(((code).wrapping_add(len)) as isize)); ci1 = assigned; assigned };
vm_block = 92; continue;
}
// C line 1312
94 => {
let _ = { let assigned = (1 as i32); len = assigned; assigned };
vm_block = 89; continue;
}
// C line 1501
95 => {
let _ = { let assigned = *(((*(ci)).l_data).as_mut_ptr()).offset(((0 as i32)) as isize); (*(te)).data = assigned; assigned };
vm_block = 0; continue;
}
// C line 1500
96 => {
let _ = { let assigned = (RUN_TYPE_LF as i32); (*(te)).v_type = assigned; assigned };
vm_block = 95; continue;
}
// C line 1504
97 => {
let _ = { let assigned = *(((*(ci)).u_data).as_mut_ptr()).offset(((0 as i32)) as isize); (*(te)).data = assigned; assigned };
vm_block = 0; continue;
}
// C line 1503
98 => {
let _ = { let assigned = (RUN_TYPE_U as i32); (*(te)).v_type = assigned; assigned };
vm_block = 97; continue;
}
// C line 1499
99 => {
vm_block = if (is_lower) != 0 { 96 } else { 98 }; continue;
}
// C line 1498
100 => {
let _ = { let assigned = len; (*(te)).len = assigned; assigned };
vm_block = 99; continue;
}
// C line 1483
101 => {
vm_block = if (((((code).wrapping_add(len)) <= ((1114111 as i32))) as i32)) != 0 { 110 } else { 100 }; continue;
}
// C line 1496
102 => {
let _ = { let old = len; len = (len).wrapping_add(1); old };
vm_block = 101; continue;
}
// C line 1490
103 => {
vm_block = 100; continue;
}
// C line 1488
104 => {
vm_block = if (((((((((((*(ci1)).l_len) as i32)) != ((1 as i32))) as i32)) != 0) || (((((*(((*(ci1)).l_data).as_mut_ptr()).offset(((0 as i32)) as isize)) != ((*(((*(ci)).l_data).as_mut_ptr()).offset(((0 as i32)) as isize)).wrapping_add(len))) as i32)) != 0)) as i32)) != 0 { 103 } else { 102 }; continue;
}
// C line 1494
105 => {
vm_block = 100; continue;
}
// C line 1492
106 => {
vm_block = if (((((((((((*(ci1)).u_len) as i32)) != ((1 as i32))) as i32)) != 0) || (((((*(((*(ci1)).u_data).as_mut_ptr()).offset(((0 as i32)) as isize)) != ((*(((*(ci)).u_data).as_mut_ptr()).offset(((0 as i32)) as isize)).wrapping_add(len))) as i32)) != 0)) as i32)) != 0 { 105 } else { 102 }; continue;
}
// C line 1487
107 => {
vm_block = if (is_lower) != 0 { 104 } else { 106 }; continue;
}
// C line 1486
108 => {
vm_block = 100; continue;
}
// C line 1485
109 => {
vm_block = if (is_complicated_case(ci1)) != 0 { 108 } else { 107 }; continue;
}
// C line 1484
110 => {
let _ = { let assigned = ptr::addr_of_mut!(*(tab).offset(((code).wrapping_add(len)) as isize)); ci1 = assigned; assigned };
vm_block = 109; continue;
}
// C line 1482
111 => {
let _ = { let assigned = (1 as i32); len = assigned; assigned };
vm_block = 101; continue;
}
// C line 1481
112 => {
let _ = { let assigned = ((((((*(ci)).l_len) as i32)) > ((0 as i32))) as i32); is_lower = assigned; assigned };
vm_block = 111; continue;
}
// C line 1480
113 => {
let _ = { let assigned = ptr::addr_of_mut!(*(tab).offset((code) as isize)); ci = assigned; assigned };
vm_block = 112; continue;
}
// C line 1477
114 => {
return;
}
// C line 1476
115 => {
let _ = { let assigned = (0 as i32); (*(te)).data = assigned; assigned };
vm_block = 114; continue;
}
// C line 1475
116 => {
let _ = { let assigned = (RUN_TYPE_UL as i32); (*(te)).v_type = assigned; assigned };
vm_block = 115; continue;
}
// C line 1474
117 => {
let _ = { let assigned = len; (*(te)).len = assigned; assigned };
vm_block = 116; continue;
}
// C line 1473
118 => {
vm_block = if ((((len) > ((0 as i32))) as i32)) != 0 { 117 } else { 113 }; continue;
}
// C line 1459
119 => {
vm_block = 130; continue;
}
// C line 1471
120 => {
let _ = { len = (len).wrapping_add((2 as i32)); len };
vm_block = 119; continue;
}
// C line 1470
121 => {
vm_block = 118; continue;
}
// C line 1469
122 => {
vm_block = if (((((((((((*(ci1)).u_len) as i32)) != ((1 as i32))) as i32)) != 0) || (((((*(((*(ci1)).u_data).as_mut_ptr()).offset(((0 as i32)) as isize)) != ((code).wrapping_add(len))) as i32)) != 0)) as i32)) != 0 { 121 } else { 120 }; continue;
}
// C line 1468
123 => {
vm_block = 118; continue;
}
// C line 1467
124 => {
vm_block = if (((((((((((*(ci)).l_len) as i32)) != ((1 as i32))) as i32)) != 0) || (((((*(((*(ci)).l_data).as_mut_ptr()).offset(((0 as i32)) as isize)) != (((code).wrapping_add(len)).wrapping_add((1 as i32)))) as i32)) != 0)) as i32)) != 0 { 123 } else { 122 }; continue;
}
// C line 1465
125 => {
vm_block = 118; continue;
}
// C line 1464
126 => {
vm_block = if (((((is_complicated_case(ci)) != 0) || ((is_complicated_case(ci1)) != 0)) as i32)) != 0 { 125 } else { 124 }; continue;
}
// C line 1463
127 => {
let _ = { let assigned = ptr::addr_of_mut!(*(tab).offset((((code).wrapping_add(len)).wrapping_add((1 as i32))) as isize)); ci1 = assigned; assigned };
vm_block = 126; continue;
}
// C line 1462
128 => {
let _ = { let assigned = ptr::addr_of_mut!(*(tab).offset(((code).wrapping_add(len)) as isize)); ci = assigned; assigned };
vm_block = 127; continue;
}
// C line 1461
129 => {
vm_block = 118; continue;
}
// C line 1460
130 => {
vm_block = if ((((((((code) >= ((1114111 as i32))) as i32)) != 0) || (((((len) >= ((126 as i32))) as i32)) != 0)) as i32)) != 0 { 129 } else { 128 }; continue;
}
// C line 1458
131 => {
let _ = { let assigned = (0 as i32); len = assigned; assigned };
vm_block = 119; continue;
}
// C line 1311
132 => {
vm_block = if (is_complicated_case(ci)) != 0 { 94 } else { 131 }; continue;
}
// C line 1308
133 => {
return;
}
// C line 1307
134 => {
let _ = { let assigned = (RUN_TYPE_LSU as i32); (*(te)).v_type = assigned; assigned };
vm_block = 133; continue;
}
// C line 1306
135 => {
let _ = { let assigned = (0 as i32); (*(te)).data = assigned; assigned };
vm_block = 134; continue;
}
// C line 1305
136 => {
let _ = { let assigned = (3 as i32); (*(te)).len = assigned; assigned };
vm_block = 135; continue;
}
// C line 1294
137 => {
vm_block = if (((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((*(ci)).l_len) as i32)) == ((1 as i32))) as i32)) != 0) && (((((*(((*(ci)).l_data).as_mut_ptr()).offset(((0 as i32)) as isize)) == ((code).wrapping_add((2 as i32)))) as i32)) != 0)) as i32)) != 0) && ((((((((*(ci)).f_len) as i32)) == ((1 as i32))) as i32)) != 0)) as i32)) != 0) && (((((*(((*(ci)).f_data).as_mut_ptr()).offset(((0 as i32)) as isize)) == (*(((*(ci)).l_data).as_mut_ptr()).offset(((0 as i32)) as isize))) as i32)) != 0)) as i32)) != 0) && ((((((((*(ci)).u_len) as i32)) == ((0 as i32))) as i32)) != 0)) as i32)) != 0) && ((((((((*(ci1)).l_len) as i32)) == ((1 as i32))) as i32)) != 0)) as i32)) != 0) && (((((*(((*(ci1)).l_data).as_mut_ptr()).offset(((0 as i32)) as isize)) == ((code).wrapping_add((2 as i32)))) as i32)) != 0)) as i32)) != 0) && ((((((((*(ci1)).f_len) as i32)) == ((1 as i32))) as i32)) != 0)) as i32)) != 0) && (((((*(((*(ci1)).f_data).as_mut_ptr()).offset(((0 as i32)) as isize)) == (*(((*(ci1)).l_data).as_mut_ptr()).offset(((0 as i32)) as isize))) as i32)) != 0)) as i32)) != 0) && ((((((((*(ci1)).u_len) as i32)) == ((1 as i32))) as i32)) != 0)) as i32)) != 0) && (((((*(((*(ci1)).u_data).as_mut_ptr()).offset(((0 as i32)) as isize)) == (code)) as i32)) != 0)) as i32)) != 0) && ((((((((*(ci2)).l_len) as i32)) == ((0 as i32))) as i32)) != 0)) as i32)) != 0) && ((((((((*(ci2)).f_len) as i32)) == ((0 as i32))) as i32)) != 0)) as i32)) != 0) && ((((((((*(ci2)).u_len) as i32)) == ((1 as i32))) as i32)) != 0)) as i32)) != 0) && (((((*(((*(ci2)).u_data).as_mut_ptr()).offset(((0 as i32)) as isize)) == (code)) as i32)) != 0)) as i32)) != 0 { 136 } else { 132 }; continue;
}
// C line 1292
138 => {
let _ = { let assigned = code; (*(te)).code = assigned; assigned };
vm_block = 137; continue;
}
// C line 1291
139 => {
let _ = { let assigned = ptr::addr_of_mut!(*(tab).offset(((code).wrapping_add((2 as i32))) as isize)); ci2 = assigned; assigned };
vm_block = 138; continue;
}
// C line 1290
140 => {
let _ = { let assigned = ptr::addr_of_mut!(*(tab).offset(((code).wrapping_add((1 as i32))) as isize)); ci1 = assigned; assigned };
vm_block = 139; continue;
}
// C line 1289
141 => {
let _ = { let assigned = ptr::addr_of_mut!(*(tab).offset((code) as isize)); ci = assigned; assigned };
vm_block = 140; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: unicode_gen.c:1514. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn dump_case_conv_table1() -> () {
let mut vm_local_storage = Vec::<u64>::new();
let mut i: i32 = core::mem::zeroed();
let mut j: i32 = core::mem::zeroed();
let mut te: *const TableEntry = core::mem::zeroed();
let mut vm_block: usize = 11;
loop { match vm_block {
// C line ?
0 => {
return ();
}
// C line 1528
1 => {
let _ = tool_printf(c"table_len=%d ext_len=%d\n".as_ptr(), &[ToolPrintArg::Int(conv_table_len as u64), ToolPrintArg::Int(ext_data_len as u64)]);
vm_block = 0; continue;
}
// C line 1519
2 => {
vm_block = if ((((i) < (conv_table_len)) as i32)) != 0 { 10 } else { 1 }; continue;
}
// C line ?
3 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 2; continue;
}
// C line 1526
4 => {
let _ = tool_printf(c"\n".as_ptr(), &[]);
vm_block = 3; continue;
}
// C line 1523
5 => {
vm_block = if ((((j) < ((*(te)).ext_len)) as i32)) != 0 { 7 } else { 4 }; continue;
}
// C line ?
6 => {
let _ = { let old = j; j = (j).wrapping_add(1); old };
vm_block = 5; continue;
}
// C line 1524
7 => {
let _ = tool_printf(c" %05x".as_ptr(), &[ToolPrintArg::Int(*(((*(te)).ext_data).as_ptr()).offset((j) as isize) as u64)]);
vm_block = 6; continue;
}
// C line 1523
8 => {
let _ = { let assigned = (0 as i32); j = assigned; assigned };
vm_block = 5; continue;
}
// C line 1521
9 => {
let _ = tool_printf(c"%05x %02x %-10s %05x".as_ptr(), &[ToolPrintArg::Int((*(te)).code as u64), ToolPrintArg::Int((*(te)).len as u64), ToolPrintArg::Str(*((run_type_str).as_mut_ptr()).offset(((*(te)).v_type) as isize) as *const c_char), ToolPrintArg::Int((*(te)).data as u64)]);
vm_block = 8; continue;
}
// C line 1520
10 => {
let _ = { let assigned = ptr::addr_of_mut!(*((conv_table).as_mut_ptr()).offset((i) as isize)); te = assigned; assigned };
vm_block = 9; continue;
}
// C line 1519
11 => {
let _ = { let assigned = (0 as i32); i = assigned; assigned };
vm_block = 2; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: unicode_gen.c:1531. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn find_data_index(mut local_conv_table: *const TableEntry, mut len: i32, mut data: i32) -> i32 {
let mut vm_local_storage = Vec::<u64>::new();
let mut i: i32 = core::mem::zeroed();
let mut te: *const TableEntry = core::mem::zeroed();
let mut vm_block: usize = 7;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 1540
1 => {
return ((1 as i32)).wrapping_neg();
}
// C line 1535
2 => {
vm_block = if ((((i) < (len)) as i32)) != 0 { 6 } else { 1 }; continue;
}
// C line ?
3 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 2; continue;
}
// C line 1538
4 => {
return i;
}
// C line 1537
5 => {
vm_block = if (((((*(te)).code) == (data)) as i32)) != 0 { 4 } else { 3 }; continue;
}
// C line 1536
6 => {
let _ = { let assigned = ptr::addr_of!(*(local_conv_table).offset((i) as isize)); te = assigned; assigned };
vm_block = 5; continue;
}
// C line 1535
7 => {
let _ = { let assigned = (0 as i32); i = assigned; assigned };
vm_block = 2; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: unicode_gen.c:1543. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn find_ext_data_index(mut data: i32) -> i32 {
let mut vm_local_storage = Vec::<u64>::new();
let mut i: i32 = core::mem::zeroed();
let mut vm_block: usize = 8;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 1552
1 => {
return (ext_data_len).wrapping_sub((1 as i32));
}
// C line 1551
2 => {
let _ = { let assigned = data; *((ext_data).as_mut_ptr()).offset(({ let old = ext_data_len; ext_data_len = (ext_data_len).wrapping_add(1); old }) as isize) = assigned; assigned };
vm_block = 1; continue;
}
// C line 1550
3 => {
let _ = if ((((!(((((((ext_data_len) as usize)) < ((((size_of::<[i32; 1000]>() as usize)) / ((size_of::<i32>() as usize))))) as i32)) != 0) as i32)) as i64)) != 0 { tool_assert_rtn(c"find_ext_data_index".as_ptr(), c"unicode_gen.c".as_ptr(), (1550 as i32), c"ext_data_len < countof(ext_data)".as_ptr()) } else { { let _ = (0 as i32); } };
vm_block = 2; continue;
}
// C line 1546
4 => {
vm_block = if ((((i) < (ext_data_len)) as i32)) != 0 { 7 } else { 3 }; continue;
}
// C line ?
5 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 4; continue;
}
// C line 1548
6 => {
return i;
}
// C line 1547
7 => {
vm_block = if ((((*((ext_data).as_mut_ptr()).offset((i) as isize)) == (data)) as i32)) != 0 { 6 } else { 5 }; continue;
}
// C line 1546
8 => {
let _ = { let assigned = (0 as i32); i = assigned; assigned };
vm_block = 4; continue;
}
_ => std::process::abort(),
} }
}

#[cfg(all(not(feature="unicode-dump-case-conv-table")))]
// Generated from the official C AST; original goto CFG retained.
// c: unicode_gen.c:1555. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn build_conv_table(mut tab: *mut CCInfo) -> () {
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
let mut vm_block: usize = 67;
loop { match vm_block {
// C line ?
0 => {
return ();
}
// C line 1648
1 => {
vm_block = if ((((i) < (conv_table_len)) as i32)) != 0 { 5 } else { 0 }; continue;
}
// C line ?
2 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 1; continue;
}
// C line 1653
3 => {
let _ = { let assigned = find_ext_data_index(*(((*(te)).ext_data).as_mut_ptr()).offset(((0 as i32)) as isize)); (*(te)).data_index = assigned; assigned };
vm_block = 2; continue;
}
// C line 1650
4 => {
vm_block = if (((((((((((((*(te)).v_type) == ((RUN_TYPE_UF_D1_EXT as i32))) as i32)) != 0) || ((((((*(te)).v_type) == ((RUN_TYPE_U_EXT as i32))) as i32)) != 0)) as i32)) != 0) || ((((((*(te)).v_type) == ((RUN_TYPE_LF_EXT as i32))) as i32)) != 0)) as i32)) != 0 { 3 } else { 2 }; continue;
}
// C line 1649
5 => {
let _ = { let assigned = ptr::addr_of_mut!(*((conv_table).as_mut_ptr()).offset((i) as isize)); te = assigned; assigned };
vm_block = 4; continue;
}
// C line 1648
6 => {
let _ = { let assigned = (0 as i32); i = assigned; assigned };
vm_block = 1; continue;
}
// C line 1632
7 => {
vm_block = if ((((i) < (conv_table_len)) as i32)) != 0 { 18 } else { 6 }; continue;
}
// C line ?
8 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 7; continue;
}
// C line 1644
9 => {
let _ = { let assigned = v_1; (*(te)).data_index = assigned; assigned };
vm_block = 8; continue;
}
// C line 1639
10 => {
vm_block = if ((((j) < ((2 as i32))) as i32)) != 0 { 14 } else { 9 }; continue;
}
// C line ?
11 => {
let _ = { let old = j; j = (j).wrapping_add(1); old };
vm_block = 10; continue;
}
// C line 1642
12 => {
let _ = { let assigned = (((v_1).wrapping_shl(((6 as i32)) as u32)) | (p_1)); v_1 = assigned; assigned };
vm_block = 11; continue;
}
// C line 1641
13 => {
let _ = if ((((!(((((p_1) < ((64 as i32))) as i32)) != 0) as i32)) as i64)) != 0 { tool_assert_rtn(c"build_conv_table".as_ptr(), c"unicode_gen.c".as_ptr(), (1641 as i32), c"p < 64".as_ptr()) } else { { let _ = (0 as i32); } };
vm_block = 12; continue;
}
// C line 1640
14 => {
let _ = { let assigned = find_ext_data_index(*(((*(te)).ext_data).as_mut_ptr()).offset((j) as isize)); p_1 = assigned; assigned };
vm_block = 13; continue;
}
// C line 1639
15 => {
let _ = { let assigned = (0 as i32); j = assigned; assigned };
vm_block = 10; continue;
}
// C line 1638
16 => {
let _ = { let assigned = (0 as i32); v_1 = assigned; assigned };
vm_block = 15; continue;
}
// C line 1634
17 => {
vm_block = if (((((((((((((*(te)).v_type) == ((RUN_TYPE_LF_EXT2 as i32))) as i32)) != 0) || ((((((*(te)).v_type) == ((RUN_TYPE_UF_EXT2 as i32))) as i32)) != 0)) as i32)) != 0) || ((((((*(te)).v_type) == ((RUN_TYPE_U2L_399_EXT2 as i32))) as i32)) != 0)) as i32)) != 0 { 16 } else { 8 }; continue;
}
// C line 1633
18 => {
let _ = { let assigned = ptr::addr_of_mut!(*((conv_table).as_mut_ptr()).offset((i) as isize)); te = assigned; assigned };
vm_block = 17; continue;
}
// C line 1632
19 => {
let _ = { let assigned = (0 as i32); i = assigned; assigned };
vm_block = 7; continue;
}
// C line 1618
20 => {
vm_block = if ((((i) < (conv_table_len)) as i32)) != 0 { 31 } else { 19 }; continue;
}
// C line ?
21 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 20; continue;
}
// C line 1628
22 => {
let _ = { let assigned = v; (*(te)).data_index = assigned; assigned };
vm_block = 21; continue;
}
// C line 1623
23 => {
vm_block = if ((((j) < ((3 as i32))) as i32)) != 0 { 27 } else { 22 }; continue;
}
// C line ?
24 => {
let _ = { let old = j; j = (j).wrapping_add(1); old };
vm_block = 23; continue;
}
// C line 1626
25 => {
let _ = { let assigned = (((v).wrapping_shl(((4 as i32)) as u32)) | (p)); v = assigned; assigned };
vm_block = 24; continue;
}
// C line 1625
26 => {
let _ = if ((((!(((((p) < ((16 as i32))) as i32)) != 0) as i32)) as i64)) != 0 { tool_assert_rtn(c"build_conv_table".as_ptr(), c"unicode_gen.c".as_ptr(), (1625 as i32), c"p < 16".as_ptr()) } else { { let _ = (0 as i32); } };
vm_block = 25; continue;
}
// C line 1624
27 => {
let _ = { let assigned = find_ext_data_index(*(((*(te)).ext_data).as_mut_ptr()).offset((j) as isize)); p = assigned; assigned };
vm_block = 26; continue;
}
// C line 1623
28 => {
let _ = { let assigned = (0 as i32); j = assigned; assigned };
vm_block = 23; continue;
}
// C line 1622
29 => {
let _ = { let assigned = (0 as i32); v = assigned; assigned };
vm_block = 28; continue;
}
// C line 1620
30 => {
vm_block = if (((((*(te)).v_type) == ((RUN_TYPE_UF_EXT3 as i32))) as i32)) != 0 { 29 } else { 21 }; continue;
}
// C line 1619
31 => {
let _ = { let assigned = ptr::addr_of_mut!(*((conv_table).as_mut_ptr()).offset((i) as isize)); te = assigned; assigned };
vm_block = 30; continue;
}
// C line 1618
32 => {
let _ = { let assigned = (0 as i32); i = assigned; assigned };
vm_block = 20; continue;
}
// C line 1581
33 => {
vm_block = if ((((i) < (conv_table_len)) as i32)) != 0 { 53 } else { 32 }; continue;
}
// C line ?
34 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 33; continue;
}
// C line 1613
35 => {
vm_block = 34; continue;
}
// C line 1612
36 => {
let _ = { let assigned = (*(te)).data; (*(te)).data_index = assigned; assigned };
vm_block = 35; continue;
}
// C line 1610
37 => {
vm_block = 34; continue;
}
// C line 1605
38 => {
let _ = tool_exit((1 as i32));
vm_block = 37; continue;
}
// C line ?
39 => {
let _ = tool_printf(c"%05x: index not found\n".as_ptr(), &[ToolPrintArg::Int((*(te)).code as u64)]);
vm_block = 38; continue;
}
// C line 1602
40 => {
vm_block = 37; continue;
}
// C line 1601
41 => {
let _ = { let assigned = (*(te)).data; *(((*(te)).ext_data).as_mut_ptr()).offset(((0 as i32)) as isize) = assigned; assigned };
vm_block = 40; continue;
}
// C line 1600
42 => {
let _ = { let assigned = (1 as i32); (*(te)).ext_len = assigned; assigned };
vm_block = 41; continue;
}
// C line 1599
43 => {
let _ = { let assigned = (RUN_TYPE_LF_EXT as i32); (*(te)).v_type = assigned; assigned };
vm_block = 42; continue;
}
// C line 1597
44 => {
vm_block = 37; continue;
}
// C line 1596
45 => {
let _ = { let assigned = (*(te)).data; *(((*(te)).ext_data).as_mut_ptr()).offset(((0 as i32)) as isize) = assigned; assigned };
vm_block = 44; continue;
}
// C line 1595
46 => {
let _ = { let assigned = (1 as i32); (*(te)).ext_len = assigned; assigned };
vm_block = 45; continue;
}
// C line 1594
47 => {
let _ = { let assigned = (RUN_TYPE_U_EXT as i32); (*(te)).v_type = assigned; assigned };
vm_block = 46; continue;
}
// C line 1592
48 => {
vm_block = match (*(te)).v_type { x if x == (RUN_TYPE_LF as i32) => 43, x if x == (RUN_TYPE_U as i32) => 47, _ => 39, }; continue;
}
// C line 1608
49 => {
let _ = { let assigned = data_index; (*(te)).data_index = assigned; assigned };
vm_block = 37; continue;
}
// C line 1591
50 => {
vm_block = if ((((data_index) < ((0 as i32))) as i32)) != 0 { 48 } else { 49 }; continue;
}
// C line 1590
51 => {
let _ = { let assigned = find_data_index((conv_table).as_mut_ptr(), conv_table_len, (*(te)).data); data_index = assigned; assigned };
vm_block = 50; continue;
}
// C line 1585
52 => {
vm_block = match (*(te)).v_type { x if x == (RUN_TYPE_UF_D20 as i32) => 36, x if x == (RUN_TYPE_LF as i32) => 51, x if x == (RUN_TYPE_UF as i32) => 51, x if x == (RUN_TYPE_L as i32) => 51, x if x == (RUN_TYPE_U as i32) => 51, _ => 34, }; continue;
}
// C line 1583
53 => {
let _ = { let assigned = ptr::addr_of_mut!(*((conv_table).as_mut_ptr()).offset((i) as isize)); te = assigned; assigned };
vm_block = 52; continue;
}
// C line 1581
54 => {
let _ = { let assigned = (0 as i32); i = assigned; assigned };
vm_block = 33; continue;
}
// C line 1578
55 => {
let _ = { let assigned = ((((te).offset_from((conv_table).as_mut_ptr()) as i64)) as i32); conv_table_len = assigned; assigned };
vm_block = 54; continue;
}
// C line 1562
56 => {
vm_block = if ((((code) <= ((1114111 as i32))) as i32)) != 0 { 65 } else { 55 }; continue;
}
// C line ?
57 => {
let _ = { let old = code; code = (code).wrapping_add(1); old };
vm_block = 56; continue;
}
// C line 1576
58 => {
let _ = { let old = te; te = (te).offset(1); old };
vm_block = 57; continue;
}
// C line 1575
59 => {
let _ = { code = (code).wrapping_add(((*(te)).len).wrapping_sub((1 as i32))); code };
vm_block = 58; continue;
}
// C line 1574
60 => {
let _ = if ((((!((((((*(te)).len) <= ((127 as i32))) as i32)) != 0) as i32)) as i64)) != 0 { tool_assert_rtn(c"build_conv_table".as_ptr(), c"unicode_gen.c".as_ptr(), (1574 as i32), c"te->len <= 127".as_ptr()) } else { { let _ = (0 as i32); } };
vm_block = 59; continue;
}
// C line 1567
61 => {
let _ = find_run_type(te, tab, code);
vm_block = 60; continue;
}
// C line 1566
62 => {
let _ = if ((((!(((((((((te).offset_from((conv_table).as_mut_ptr()) as i64)) as usize)) < ((((size_of::<[TableEntry; 1000]>() as usize)) / ((size_of::<TableEntry>() as usize))))) as i32)) != 0) as i32)) as i64)) != 0 { tool_assert_rtn(c"build_conv_table".as_ptr(), c"unicode_gen.c".as_ptr(), (1566 as i32), c"te - conv_table < countof(conv_table)".as_ptr()) } else { { let _ = (0 as i32); } };
vm_block = 61; continue;
}
// C line 1565
63 => {
vm_block = 57; continue;
}
// C line 1564
64 => {
vm_block = if (((((((((((((((*(ci)).u_len) as i32)) == ((0 as i32))) as i32)) != 0) && ((((((((*(ci)).l_len) as i32)) == ((0 as i32))) as i32)) != 0)) as i32)) != 0) && ((((((((*(ci)).f_len) as i32)) == ((0 as i32))) as i32)) != 0)) as i32)) != 0 { 63 } else { 62 }; continue;
}
// C line 1563
65 => {
let _ = { let assigned = ptr::addr_of_mut!(*(tab).offset((code) as isize)); ci = assigned; assigned };
vm_block = 64; continue;
}
// C line 1562
66 => {
let _ = { let assigned = (0 as i32); code = assigned; assigned };
vm_block = 56; continue;
}
// C line 1561
67 => {
let _ = { let assigned = (conv_table).as_mut_ptr(); te = assigned; assigned };
vm_block = 66; continue;
}
_ => std::process::abort(),
} }
}

#[cfg(all(feature="unicode-dump-case-conv-table"))]
// Generated from the official C AST; original goto CFG retained.
// c: unicode_gen.c:1555. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn build_conv_table(mut tab: *mut CCInfo) -> () {
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
let mut vm_block: usize = 68;
loop { match vm_block {
// C line ?
0 => {
return ();
}
// C line 1657
1 => {
let _ = dump_case_conv_table1();
vm_block = 0; continue;
}
// C line 1648
2 => {
vm_block = if ((((i) < (conv_table_len)) as i32)) != 0 { 6 } else { 1 }; continue;
}
// C line ?
3 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 2; continue;
}
// C line 1653
4 => {
let _ = { let assigned = find_ext_data_index(*(((*(te)).ext_data).as_mut_ptr()).offset(((0 as i32)) as isize)); (*(te)).data_index = assigned; assigned };
vm_block = 3; continue;
}
// C line 1650
5 => {
vm_block = if (((((((((((((*(te)).v_type) == ((RUN_TYPE_UF_D1_EXT as i32))) as i32)) != 0) || ((((((*(te)).v_type) == ((RUN_TYPE_U_EXT as i32))) as i32)) != 0)) as i32)) != 0) || ((((((*(te)).v_type) == ((RUN_TYPE_LF_EXT as i32))) as i32)) != 0)) as i32)) != 0 { 4 } else { 3 }; continue;
}
// C line 1649
6 => {
let _ = { let assigned = ptr::addr_of_mut!(*((conv_table).as_mut_ptr()).offset((i) as isize)); te = assigned; assigned };
vm_block = 5; continue;
}
// C line 1648
7 => {
let _ = { let assigned = (0 as i32); i = assigned; assigned };
vm_block = 2; continue;
}
// C line 1632
8 => {
vm_block = if ((((i) < (conv_table_len)) as i32)) != 0 { 19 } else { 7 }; continue;
}
// C line ?
9 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 8; continue;
}
// C line 1644
10 => {
let _ = { let assigned = v_1; (*(te)).data_index = assigned; assigned };
vm_block = 9; continue;
}
// C line 1639
11 => {
vm_block = if ((((j) < ((2 as i32))) as i32)) != 0 { 15 } else { 10 }; continue;
}
// C line ?
12 => {
let _ = { let old = j; j = (j).wrapping_add(1); old };
vm_block = 11; continue;
}
// C line 1642
13 => {
let _ = { let assigned = (((v_1).wrapping_shl(((6 as i32)) as u32)) | (p_1)); v_1 = assigned; assigned };
vm_block = 12; continue;
}
// C line 1641
14 => {
let _ = if ((((!(((((p_1) < ((64 as i32))) as i32)) != 0) as i32)) as i64)) != 0 { tool_assert_rtn(c"build_conv_table".as_ptr(), c"unicode_gen.c".as_ptr(), (1641 as i32), c"p < 64".as_ptr()) } else { { let _ = (0 as i32); } };
vm_block = 13; continue;
}
// C line 1640
15 => {
let _ = { let assigned = find_ext_data_index(*(((*(te)).ext_data).as_mut_ptr()).offset((j) as isize)); p_1 = assigned; assigned };
vm_block = 14; continue;
}
// C line 1639
16 => {
let _ = { let assigned = (0 as i32); j = assigned; assigned };
vm_block = 11; continue;
}
// C line 1638
17 => {
let _ = { let assigned = (0 as i32); v_1 = assigned; assigned };
vm_block = 16; continue;
}
// C line 1634
18 => {
vm_block = if (((((((((((((*(te)).v_type) == ((RUN_TYPE_LF_EXT2 as i32))) as i32)) != 0) || ((((((*(te)).v_type) == ((RUN_TYPE_UF_EXT2 as i32))) as i32)) != 0)) as i32)) != 0) || ((((((*(te)).v_type) == ((RUN_TYPE_U2L_399_EXT2 as i32))) as i32)) != 0)) as i32)) != 0 { 17 } else { 9 }; continue;
}
// C line 1633
19 => {
let _ = { let assigned = ptr::addr_of_mut!(*((conv_table).as_mut_ptr()).offset((i) as isize)); te = assigned; assigned };
vm_block = 18; continue;
}
// C line 1632
20 => {
let _ = { let assigned = (0 as i32); i = assigned; assigned };
vm_block = 8; continue;
}
// C line 1618
21 => {
vm_block = if ((((i) < (conv_table_len)) as i32)) != 0 { 32 } else { 20 }; continue;
}
// C line ?
22 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 21; continue;
}
// C line 1628
23 => {
let _ = { let assigned = v; (*(te)).data_index = assigned; assigned };
vm_block = 22; continue;
}
// C line 1623
24 => {
vm_block = if ((((j) < ((3 as i32))) as i32)) != 0 { 28 } else { 23 }; continue;
}
// C line ?
25 => {
let _ = { let old = j; j = (j).wrapping_add(1); old };
vm_block = 24; continue;
}
// C line 1626
26 => {
let _ = { let assigned = (((v).wrapping_shl(((4 as i32)) as u32)) | (p)); v = assigned; assigned };
vm_block = 25; continue;
}
// C line 1625
27 => {
let _ = if ((((!(((((p) < ((16 as i32))) as i32)) != 0) as i32)) as i64)) != 0 { tool_assert_rtn(c"build_conv_table".as_ptr(), c"unicode_gen.c".as_ptr(), (1625 as i32), c"p < 16".as_ptr()) } else { { let _ = (0 as i32); } };
vm_block = 26; continue;
}
// C line 1624
28 => {
let _ = { let assigned = find_ext_data_index(*(((*(te)).ext_data).as_mut_ptr()).offset((j) as isize)); p = assigned; assigned };
vm_block = 27; continue;
}
// C line 1623
29 => {
let _ = { let assigned = (0 as i32); j = assigned; assigned };
vm_block = 24; continue;
}
// C line 1622
30 => {
let _ = { let assigned = (0 as i32); v = assigned; assigned };
vm_block = 29; continue;
}
// C line 1620
31 => {
vm_block = if (((((*(te)).v_type) == ((RUN_TYPE_UF_EXT3 as i32))) as i32)) != 0 { 30 } else { 22 }; continue;
}
// C line 1619
32 => {
let _ = { let assigned = ptr::addr_of_mut!(*((conv_table).as_mut_ptr()).offset((i) as isize)); te = assigned; assigned };
vm_block = 31; continue;
}
// C line 1618
33 => {
let _ = { let assigned = (0 as i32); i = assigned; assigned };
vm_block = 21; continue;
}
// C line 1581
34 => {
vm_block = if ((((i) < (conv_table_len)) as i32)) != 0 { 54 } else { 33 }; continue;
}
// C line ?
35 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 34; continue;
}
// C line 1613
36 => {
vm_block = 35; continue;
}
// C line 1612
37 => {
let _ = { let assigned = (*(te)).data; (*(te)).data_index = assigned; assigned };
vm_block = 36; continue;
}
// C line 1610
38 => {
vm_block = 35; continue;
}
// C line 1605
39 => {
let _ = tool_exit((1 as i32));
vm_block = 38; continue;
}
// C line ?
40 => {
let _ = tool_printf(c"%05x: index not found\n".as_ptr(), &[ToolPrintArg::Int((*(te)).code as u64)]);
vm_block = 39; continue;
}
// C line 1602
41 => {
vm_block = 38; continue;
}
// C line 1601
42 => {
let _ = { let assigned = (*(te)).data; *(((*(te)).ext_data).as_mut_ptr()).offset(((0 as i32)) as isize) = assigned; assigned };
vm_block = 41; continue;
}
// C line 1600
43 => {
let _ = { let assigned = (1 as i32); (*(te)).ext_len = assigned; assigned };
vm_block = 42; continue;
}
// C line 1599
44 => {
let _ = { let assigned = (RUN_TYPE_LF_EXT as i32); (*(te)).v_type = assigned; assigned };
vm_block = 43; continue;
}
// C line 1597
45 => {
vm_block = 38; continue;
}
// C line 1596
46 => {
let _ = { let assigned = (*(te)).data; *(((*(te)).ext_data).as_mut_ptr()).offset(((0 as i32)) as isize) = assigned; assigned };
vm_block = 45; continue;
}
// C line 1595
47 => {
let _ = { let assigned = (1 as i32); (*(te)).ext_len = assigned; assigned };
vm_block = 46; continue;
}
// C line 1594
48 => {
let _ = { let assigned = (RUN_TYPE_U_EXT as i32); (*(te)).v_type = assigned; assigned };
vm_block = 47; continue;
}
// C line 1592
49 => {
vm_block = match (*(te)).v_type { x if x == (RUN_TYPE_LF as i32) => 44, x if x == (RUN_TYPE_U as i32) => 48, _ => 40, }; continue;
}
// C line 1608
50 => {
let _ = { let assigned = data_index; (*(te)).data_index = assigned; assigned };
vm_block = 38; continue;
}
// C line 1591
51 => {
vm_block = if ((((data_index) < ((0 as i32))) as i32)) != 0 { 49 } else { 50 }; continue;
}
// C line 1590
52 => {
let _ = { let assigned = find_data_index((conv_table).as_mut_ptr(), conv_table_len, (*(te)).data); data_index = assigned; assigned };
vm_block = 51; continue;
}
// C line 1585
53 => {
vm_block = match (*(te)).v_type { x if x == (RUN_TYPE_UF_D20 as i32) => 37, x if x == (RUN_TYPE_LF as i32) => 52, x if x == (RUN_TYPE_UF as i32) => 52, x if x == (RUN_TYPE_L as i32) => 52, x if x == (RUN_TYPE_U as i32) => 52, _ => 35, }; continue;
}
// C line 1583
54 => {
let _ = { let assigned = ptr::addr_of_mut!(*((conv_table).as_mut_ptr()).offset((i) as isize)); te = assigned; assigned };
vm_block = 53; continue;
}
// C line 1581
55 => {
let _ = { let assigned = (0 as i32); i = assigned; assigned };
vm_block = 34; continue;
}
// C line 1578
56 => {
let _ = { let assigned = ((((te).offset_from((conv_table).as_mut_ptr()) as i64)) as i32); conv_table_len = assigned; assigned };
vm_block = 55; continue;
}
// C line 1562
57 => {
vm_block = if ((((code) <= ((1114111 as i32))) as i32)) != 0 { 66 } else { 56 }; continue;
}
// C line ?
58 => {
let _ = { let old = code; code = (code).wrapping_add(1); old };
vm_block = 57; continue;
}
// C line 1576
59 => {
let _ = { let old = te; te = (te).offset(1); old };
vm_block = 58; continue;
}
// C line 1575
60 => {
let _ = { code = (code).wrapping_add(((*(te)).len).wrapping_sub((1 as i32))); code };
vm_block = 59; continue;
}
// C line 1574
61 => {
let _ = if ((((!((((((*(te)).len) <= ((127 as i32))) as i32)) != 0) as i32)) as i64)) != 0 { tool_assert_rtn(c"build_conv_table".as_ptr(), c"unicode_gen.c".as_ptr(), (1574 as i32), c"te->len <= 127".as_ptr()) } else { { let _ = (0 as i32); } };
vm_block = 60; continue;
}
// C line 1567
62 => {
let _ = find_run_type(te, tab, code);
vm_block = 61; continue;
}
// C line 1566
63 => {
let _ = if ((((!(((((((((te).offset_from((conv_table).as_mut_ptr()) as i64)) as usize)) < ((((size_of::<[TableEntry; 1000]>() as usize)) / ((size_of::<TableEntry>() as usize))))) as i32)) != 0) as i32)) as i64)) != 0 { tool_assert_rtn(c"build_conv_table".as_ptr(), c"unicode_gen.c".as_ptr(), (1566 as i32), c"te - conv_table < countof(conv_table)".as_ptr()) } else { { let _ = (0 as i32); } };
vm_block = 62; continue;
}
// C line 1565
64 => {
vm_block = 58; continue;
}
// C line 1564
65 => {
vm_block = if (((((((((((((((*(ci)).u_len) as i32)) == ((0 as i32))) as i32)) != 0) && ((((((((*(ci)).l_len) as i32)) == ((0 as i32))) as i32)) != 0)) as i32)) != 0) && ((((((((*(ci)).f_len) as i32)) == ((0 as i32))) as i32)) != 0)) as i32)) != 0 { 64 } else { 63 }; continue;
}
// C line 1563
66 => {
let _ = { let assigned = ptr::addr_of_mut!(*(tab).offset((code) as isize)); ci = assigned; assigned };
vm_block = 65; continue;
}
// C line 1562
67 => {
let _ = { let assigned = (0 as i32); code = assigned; assigned };
vm_block = 57; continue;
}
// C line 1561
68 => {
let _ = { let assigned = (conv_table).as_mut_ptr(); te = assigned; assigned };
vm_block = 67; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: unicode_gen.c:1661. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn dump_case_conv_table(mut f: *mut FILE) -> () {
let mut vm_local_storage = Vec::<u64>::new();
let mut i: i32 = core::mem::zeroed();
let mut v: u32 = core::mem::zeroed();
let mut te: *const TableEntry = core::mem::zeroed();
let mut vm_block: usize = 36;
loop { match vm_block {
// C line ?
0 => {
return ();
}
// C line 1701
1 => {
let _ = tool_fprintf(f, c"\n};\n\n".as_ptr(), &[]);
vm_block = 0; continue;
}
// C line 1696
2 => {
vm_block = if ((((i) < (ext_data_len)) as i32)) != 0 { 6 } else { 1 }; continue;
}
// C line ?
3 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 2; continue;
}
// C line 1699
4 => {
let _ = tool_fprintf(f, c" 0x%04x,".as_ptr(), &[ToolPrintArg::Int(*((ext_data).as_mut_ptr()).offset((i) as isize) as u64)]);
vm_block = 3; continue;
}
// C line 1698
5 => {
let _ = tool_fprintf(f, c"\n   ".as_ptr(), &[]);
vm_block = 4; continue;
}
// C line 1697
6 => {
vm_block = if ((((((i) % ((8 as i32)))) == ((0 as i32))) as i32)) != 0 { 5 } else { 4 }; continue;
}
// C line 1696
7 => {
let _ = { let assigned = (0 as i32); i = assigned; assigned };
vm_block = 2; continue;
}
// C line 1695
8 => {
let _ = tool_fprintf(f, c"static const uint16_t case_conv_ext[%d] = {".as_ptr(), &[ToolPrintArg::Int(ext_data_len as u64)]);
vm_block = 7; continue;
}
// C line 1694
9 => {
let _ = { total_table_bytes = (((((total_table_bytes) as usize)).wrapping_add((((ext_data_len) as usize)).wrapping_mul((size_of::<u16>() as usize)))) as u32); total_table_bytes };
vm_block = 8; continue;
}
// C line 1693
10 => {
let _ = { let old = total_tables; total_tables = (total_tables).wrapping_add(1); old };
vm_block = 9; continue;
}
// C line 1691
11 => {
let _ = tool_fprintf(f, c"\n};\n\n".as_ptr(), &[]);
vm_block = 10; continue;
}
// C line 1685
12 => {
vm_block = if ((((i) < (conv_table_len)) as i32)) != 0 { 17 } else { 11 }; continue;
}
// C line ?
13 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 12; continue;
}
// C line 1689
14 => {
let _ = tool_fprintf(f, c" 0x%02x,".as_ptr(), &[ToolPrintArg::Int((((*(te)).data_index) & ((255 as i32))) as u64)]);
vm_block = 13; continue;
}
// C line 1688
15 => {
let _ = { let assigned = ptr::addr_of_mut!(*((conv_table).as_mut_ptr()).offset((i) as isize)); te = assigned; assigned };
vm_block = 14; continue;
}
// C line 1687
16 => {
let _ = tool_fprintf(f, c"\n   ".as_ptr(), &[]);
vm_block = 15; continue;
}
// C line 1686
17 => {
vm_block = if ((((((i) % ((8 as i32)))) == ((0 as i32))) as i32)) != 0 { 16 } else { 15 }; continue;
}
// C line 1685
18 => {
let _ = { let assigned = (0 as i32); i = assigned; assigned };
vm_block = 12; continue;
}
// C line 1684
19 => {
let _ = tool_fprintf(f, c"static const uint8_t case_conv_table2[%d] = {".as_ptr(), &[ToolPrintArg::Int(conv_table_len as u64)]);
vm_block = 18; continue;
}
// C line 1683
20 => {
let _ = { total_table_bytes = (total_table_bytes).wrapping_add(((conv_table_len) as u32)); total_table_bytes };
vm_block = 19; continue;
}
// C line 1682
21 => {
let _ = { let old = total_tables; total_tables = (total_tables).wrapping_add(1); old };
vm_block = 20; continue;
}
// C line 1680
22 => {
let _ = tool_fprintf(f, c"\n};\n\n".as_ptr(), &[]);
vm_block = 21; continue;
}
// C line 1670
23 => {
vm_block = if ((((i) < (conv_table_len)) as i32)) != 0 { 32 } else { 22 }; continue;
}
// C line ?
24 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 23; continue;
}
// C line 1678
25 => {
let _ = tool_fprintf(f, c" 0x%08x,".as_ptr(), &[ToolPrintArg::Int(v as u64)]);
vm_block = 24; continue;
}
// C line 1677
26 => {
let _ = { v = ((v) | (((((*(te)).data_index).wrapping_shr(((8 as i32)) as u32)) as u32))); v };
vm_block = 25; continue;
}
// C line 1676
27 => {
let _ = { v = ((v) | (((((*(te)).v_type).wrapping_shl((((((32 as i32)).wrapping_sub((17 as i32))).wrapping_sub((7 as i32))).wrapping_sub((4 as i32))) as u32)) as u32))); v };
vm_block = 26; continue;
}
// C line 1675
28 => {
let _ = { v = ((v) | (((((*(te)).len).wrapping_shl(((((32 as i32)).wrapping_sub((17 as i32))).wrapping_sub((7 as i32))) as u32)) as u32))); v };
vm_block = 27; continue;
}
// C line 1674
29 => {
let _ = { let assigned = ((((*(te)).code).wrapping_shl((((32 as i32)).wrapping_sub((17 as i32))) as u32)) as u32); v = assigned; assigned };
vm_block = 28; continue;
}
// C line 1673
30 => {
let _ = { let assigned = ptr::addr_of_mut!(*((conv_table).as_mut_ptr()).offset((i) as isize)); te = assigned; assigned };
vm_block = 29; continue;
}
// C line 1672
31 => {
let _ = tool_fprintf(f, c"\n   ".as_ptr(), &[]);
vm_block = 30; continue;
}
// C line 1671
32 => {
vm_block = if ((((((i) % ((4 as i32)))) == ((0 as i32))) as i32)) != 0 { 31 } else { 30 }; continue;
}
// C line 1670
33 => {
let _ = { let assigned = (0 as i32); i = assigned; assigned };
vm_block = 23; continue;
}
// C line 1669
34 => {
let _ = tool_fprintf(f, c"static const uint32_t case_conv_table1[%d] = {".as_ptr(), &[ToolPrintArg::Int(conv_table_len as u64)]);
vm_block = 33; continue;
}
// C line 1668
35 => {
let _ = { total_table_bytes = (((((total_table_bytes) as usize)).wrapping_add((((conv_table_len) as usize)).wrapping_mul((size_of::<u32>() as usize)))) as u32); total_table_bytes };
vm_block = 34; continue;
}
// C line 1667
36 => {
let _ = { let old = total_tables; total_tables = (total_tables).wrapping_add(1); old };
vm_block = 35; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: unicode_gen.c:1707. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn sp_cc_cmp(mut p1: *const c_void, mut p2: *const c_void) -> i32 {
let mut vm_local_storage = Vec::<u64>::new();
let mut c1: *mut CCInfo = core::mem::zeroed();
let mut c2: *mut CCInfo = core::mem::zeroed();
let mut vm_block: usize = 7;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 1712
1 => {
return ((1 as i32)).wrapping_neg();
}
// C line 1714
2 => {
return (1 as i32);
}
// C line 1716
3 => {
return memcmp(((((*(c1)).f_data).as_mut_ptr()) as *const c_void), ((((*(c2)).f_data).as_mut_ptr()) as *const c_void), ((size_of::<i32>() as usize)).wrapping_mul((((*(c1)).f_len) as usize)));
}
// C line 1713
4 => {
vm_block = if (((((((*(c2)).f_len) as i32)) < ((((*(c1)).f_len) as i32))) as i32)) != 0 { 2 } else { 3 }; continue;
}
// C line 1711
5 => {
vm_block = if (((((((*(c1)).f_len) as i32)) < ((((*(c2)).f_len) as i32))) as i32)) != 0 { 1 } else { 4 }; continue;
}
// C line 1710
6 => {
c2 = ptr::addr_of_mut!(*(global_tab).offset((*(((p2) as *const i32))) as isize));
vm_block = 5; continue;
}
// C line 1709
7 => {
c1 = ptr::addr_of_mut!(*(global_tab).offset((*(((p1) as *const i32))) as isize));
vm_block = 6; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: unicode_gen.c:1722. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn dump_case_folding_special_cases(mut tab: *mut CCInfo) -> () {
let mut vm_local_storage = Vec::<u64>::new();
let mut i: i32 = core::mem::zeroed();
let mut len: i32 = core::mem::zeroed();
let mut j: i32 = core::mem::zeroed();
let mut perm: *mut i32 = core::mem::zeroed();
let mut vm_block: usize = 22;
loop { match vm_block {
// C line ?
0 => {
return ();
}
// C line 1748
1 => {
let _ = { let assigned = core::ptr::null_mut::<CCInfo>(); global_tab = assigned; assigned };
vm_block = 0; continue;
}
// C line 1747
2 => {
let _ = free(((perm) as *mut c_void));
vm_block = 1; continue;
}
// C line 1732
3 => {
vm_block = if ((((i) <= ((1114111 as i32))) as i32)) != 0 { 14 } else { 2 }; continue;
}
// C line 1734
4 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 3; continue;
}
// C line 1744
5 => {
let _ = { i = (i).wrapping_add(len); i };
vm_block = 3; continue;
}
// C line 1741
6 => {
vm_block = if ((((j) < ((i).wrapping_add(len))) as i32)) != 0 { 8 } else { 5 }; continue;
}
// C line ?
7 => {
let _ = { let old = j; j = (j).wrapping_add(1); old };
vm_block = 6; continue;
}
// C line 1742
8 => {
let _ = dump_cc_info(ptr::addr_of_mut!(*(tab).offset((*(perm).offset((j) as isize)) as isize)), *(perm).offset((j) as isize));
vm_block = 7; continue;
}
// C line 1741
9 => {
let _ = { let assigned = i; j = assigned; assigned };
vm_block = 6; continue;
}
// C line 1740
10 => {
vm_block = if ((((len) > ((1 as i32))) as i32)) != 0 { 9 } else { 5 }; continue;
}
// C line 1737
11 => {
vm_block = if (((((((((i).wrapping_add(len)) <= ((1114111 as i32))) as i32)) != 0) && (((!((sp_cc_cmp(((ptr::addr_of_mut!(*(perm).offset((i) as isize))) as *const c_void), ((ptr::addr_of_mut!(*(perm).offset(((i).wrapping_add(len)) as isize))) as *const c_void))) != 0) as i32)) != 0)) as i32)) != 0 { 12 } else { 10 }; continue;
}
// C line 1738
12 => {
let _ = { let old = len; len = (len).wrapping_add(1); old };
vm_block = 11; continue;
}
// C line 1736
13 => {
let _ = { let assigned = (1 as i32); len = assigned; assigned };
vm_block = 11; continue;
}
// C line 1733
14 => {
vm_block = if (((((((*(tab).offset((*(perm).offset((i) as isize)) as isize)).f_len) as i32)) <= ((1 as i32))) as i32)) != 0 { 4 } else { 13 }; continue;
}
// C line 1732
15 => {
let _ = { let assigned = (0 as i32); i = assigned; assigned };
vm_block = 3; continue;
}
// C line 1731
16 => {
let _ = tool_qsort(((perm) as *mut c_void), ((((1114111 as i32)).wrapping_add((1 as i32))) as usize), (size_of::<i32>() as usize), sp_cc_cmp);
vm_block = 15; continue;
}
// C line 1730
17 => {
let _ = { let assigned = tab; global_tab = assigned; assigned };
vm_block = 16; continue;
}
// C line 1728
18 => {
vm_block = if ((((i) <= ((1114111 as i32))) as i32)) != 0 { 20 } else { 17 }; continue;
}
// C line ?
19 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 18; continue;
}
// C line 1729
20 => {
let _ = { let assigned = i; *(perm).offset((i) as isize) = assigned; assigned };
vm_block = 19; continue;
}
// C line 1728
21 => {
let _ = { let assigned = (0 as i32); i = assigned; assigned };
vm_block = 18; continue;
}
// C line 1727
22 => {
let _ = { let assigned = ((malloc(((size_of::<i32>() as usize)).wrapping_mul(((((1114111 as i32)).wrapping_add((1 as i32))) as usize)))) as *mut i32); perm = assigned; assigned };
vm_block = 21; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: unicode_gen.c:1752. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn tabcmp(mut tab1: *const i32, mut tab2: *const i32, mut n: i32) -> i32 {
let mut vm_local_storage = Vec::<u64>::new();
let mut i: i32 = core::mem::zeroed();
let mut vm_block: usize = 6;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 1759
1 => {
return (0 as i32);
}
// C line 1755
2 => {
vm_block = if ((((i) < (n)) as i32)) != 0 { 5 } else { 1 }; continue;
}
// C line ?
3 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 2; continue;
}
// C line 1757
4 => {
return ((1 as i32)).wrapping_neg();
}
// C line 1756
5 => {
vm_block = if ((((*(tab1).offset((i) as isize)) != (*(tab2).offset((i) as isize))) as i32)) != 0 { 4 } else { 3 }; continue;
}
// C line 1755
6 => {
let _ = { let assigned = (0 as i32); i = assigned; assigned };
vm_block = 2; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: unicode_gen.c:1762. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn dump_str(mut str: *const c_char, mut buf: *const i32, mut len: i32) -> () {
let mut vm_local_storage = Vec::<u64>::new();
let mut i: i32 = core::mem::zeroed();
let mut vm_block: usize = 6;
loop { match vm_block {
// C line ?
0 => {
return ();
}
// C line 1768
1 => {
let _ = tool_printf(c"\n".as_ptr(), &[]);
vm_block = 0; continue;
}
// C line 1766
2 => {
vm_block = if ((((i) < (len)) as i32)) != 0 { 4 } else { 1 }; continue;
}
// C line ?
3 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 2; continue;
}
// C line 1767
4 => {
let _ = tool_printf(c" %05x".as_ptr(), &[ToolPrintArg::Int(*(buf).offset((i) as isize) as u64)]);
vm_block = 3; continue;
}
// C line 1766
5 => {
let _ = { let assigned = (0 as i32); i = assigned; assigned };
vm_block = 2; continue;
}
// C line 1765
6 => {
let _ = tool_printf(c"%s=".as_ptr(), &[ToolPrintArg::Str(str as *const c_char)]);
vm_block = 5; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: unicode_gen.c:1771. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn compute_internal_props() -> () {
let mut vm_local_storage = Vec::<u64>::new();
let mut i: i32 = core::mem::zeroed();
let mut has_ul: i32 = core::mem::zeroed();
let mut ci: *mut CCInfo = core::mem::zeroed();
let mut vm_block: usize = 14;
loop { match vm_block {
// C line ?
0 => {
return ();
}
// C line 1776
1 => {
vm_block = if ((((i) <= ((1114111 as i32))) as i32)) != 0 { 13 } else { 0 }; continue;
}
// C line ?
2 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 1; continue;
}
// C line 1795
3 => {
let _ = set_prop(((i) as u32), (PROP_Changes_When_NFKC_Casefolded1 as i32), ((get_prop(((i) as u32), (PROP_Changes_When_NFKC_Casefolded as i32))) ^ (((((((*(ci)).f_len) as i32)) != ((0 as i32))) as i32))));
vm_block = 2; continue;
}
// C line 1792
4 => {
let _ = set_prop(((i) as u32), (PROP_Changes_When_Casefolded1 as i32), ((get_prop(((i) as u32), (PROP_Changes_When_Casefolded as i32))) ^ (((((((*(ci)).f_len) as i32)) != ((0 as i32))) as i32))));
vm_block = 3; continue;
}
// C line 1790
5 => {
let _ = set_prop(((i) as u32), (PROP_Changes_When_Titlecased1 as i32), ((get_prop(((i) as u32), (PROP_Changes_When_Titlecased as i32))) ^ (((((((*(ci)).u_len) as i32)) != ((0 as i32))) as i32))));
vm_block = 4; continue;
}
// C line 1788
6 => {
let _ = set_prop(((i) as u32), (PROP_XID_Continue1 as i32), ((get_prop(((i) as u32), (PROP_ID_Continue as i32))) ^ (get_prop(((i) as u32), (PROP_XID_Continue as i32)))));
vm_block = 5; continue;
}
// C line 1786
7 => {
let _ = set_prop(((i) as u32), (PROP_XID_Start1 as i32), ((get_prop(((i) as u32), (PROP_ID_Start as i32))) ^ (get_prop(((i) as u32), (PROP_XID_Start as i32)))));
vm_block = 6; continue;
}
// C line 1784
8 => {
let _ = set_prop(((i) as u32), (PROP_ID_Continue1 as i32), ((get_prop(((i) as u32), (PROP_ID_Continue as i32))) & (((get_prop(((i) as u32), (PROP_ID_Start as i32))) ^ ((1 as i32))))));
vm_block = 7; continue;
}
// C line 1780
9 => {
let _ = if ((((!((get_prop(((i) as u32), (PROP_Cased as i32))) != 0) as i32)) as i64)) != 0 { tool_assert_rtn(c"compute_internal_props".as_ptr(), c"unicode_gen.c".as_ptr(), (1780 as i32), c"get_prop(i, PROP_Cased)".as_ptr()) } else { { let _ = (0 as i32); } };
vm_block = 8; continue;
}
// C line 1782
10 => {
let _ = set_prop(((i) as u32), (PROP_Cased1 as i32), get_prop(((i) as u32), (PROP_Cased as i32)));
vm_block = 8; continue;
}
// C line 1779
11 => {
vm_block = if (has_ul) != 0 { 9 } else { 10 }; continue;
}
// C line 1778
12 => {
let _ = { let assigned = ((((((((((((((*(ci)).u_len) as i32)) != ((0 as i32))) as i32)) != 0) || ((((((((*(ci)).l_len) as i32)) != ((0 as i32))) as i32)) != 0)) as i32)) != 0) || ((((((((*(ci)).f_len) as i32)) != ((0 as i32))) as i32)) != 0)) as i32); has_ul = assigned; assigned };
vm_block = 11; continue;
}
// C line 1777
13 => {
ci = ptr::addr_of_mut!(*(unicode_db).offset((i) as isize));
vm_block = 12; continue;
}
// C line 1776
14 => {
let _ = { let assigned = (0 as i32); i = assigned; assigned };
vm_block = 1; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: unicode_gen.c:1812. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn dump_byte_table(mut f: *mut FILE, mut cname: *const c_char, mut tab: *const u8, mut len: i32) -> () {
let mut vm_local_storage = Vec::<u64>::new();
let mut i: i32 = core::mem::zeroed();
let mut vm_block: usize = 10;
loop { match vm_block {
// C line ?
0 => {
return ();
}
// C line 1824
1 => {
let _ = tool_fprintf(f, c"\n};\n\n".as_ptr(), &[]);
vm_block = 0; continue;
}
// C line 1819
2 => {
vm_block = if ((((i) < (len)) as i32)) != 0 { 6 } else { 1 }; continue;
}
// C line ?
3 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 2; continue;
}
// C line 1822
4 => {
let _ = tool_fprintf(f, c" 0x%02x,".as_ptr(), &[ToolPrintArg::Int(((*(tab).offset((i) as isize)) as i32) as u64)]);
vm_block = 3; continue;
}
// C line 1821
5 => {
let _ = tool_fprintf(f, c"\n   ".as_ptr(), &[]);
vm_block = 4; continue;
}
// C line 1820
6 => {
vm_block = if ((((((i) % ((8 as i32)))) == ((0 as i32))) as i32)) != 0 { 5 } else { 4 }; continue;
}
// C line 1819
7 => {
let _ = { let assigned = (0 as i32); i = assigned; assigned };
vm_block = 2; continue;
}
// C line 1818
8 => {
let _ = tool_fprintf(f, c"static const uint8_t %s[%d] = {".as_ptr(), &[ToolPrintArg::Str(cname as *const c_char), ToolPrintArg::Int(len as u64)]);
vm_block = 7; continue;
}
// C line 1817
9 => {
let _ = { total_table_bytes = (total_table_bytes).wrapping_add(((len) as u32)); total_table_bytes };
vm_block = 8; continue;
}
// C line 1816
10 => {
let _ = { let old = total_tables; total_tables = (total_tables).wrapping_add(1); old };
vm_block = 9; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: unicode_gen.c:1827. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn dump_index_table(mut f: *mut FILE, mut cname: *const c_char, mut tab: *const u8, mut len: i32) -> () {
let mut vm_local_storage = Vec::<u64>::new();
let mut i: i32 = core::mem::zeroed();
let mut code: i32 = core::mem::zeroed();
let mut offset: i32 = core::mem::zeroed();
let mut vm_block: usize = 11;
loop { match vm_block {
// C line ?
0 => {
return ();
}
// C line 1841
1 => {
let _ = tool_fprintf(f, c"};\n\n".as_ptr(), &[]);
vm_block = 0; continue;
}
// C line 1834
2 => {
vm_block = if ((((i) < (len)) as i32)) != 0 { 7 } else { 1 }; continue;
}
// C line ?
3 => {
let _ = { i = (i).wrapping_add((3 as i32)); i };
vm_block = 2; continue;
}
// C line 1838
4 => {
let _ = tool_fprintf(f, c"  // %6.5X at %d%s\n".as_ptr(), &[ToolPrintArg::Int(code as u64), ToolPrintArg::Int(offset as u64), ToolPrintArg::Str(if ((((i) == ((len).wrapping_sub((3 as i32)))) as i32)) != 0 { c" (upper bound)".as_ptr() } else { c"".as_ptr() } as *const c_char)]);
vm_block = 3; continue;
}
// C line 1837
5 => {
let _ = tool_fprintf(f, c"    0x%02x, 0x%02x, 0x%02x,".as_ptr(), &[ToolPrintArg::Int(((*(tab).offset((i) as isize)) as i32) as u64), ToolPrintArg::Int(((*(tab).offset(((i).wrapping_add((1 as i32))) as isize)) as i32) as u64), ToolPrintArg::Int(((*(tab).offset(((i).wrapping_add((2 as i32))) as isize)) as i32) as u64)]);
vm_block = 4; continue;
}
// C line 1836
6 => {
let _ = { let assigned = (((((i) / ((3 as i32)))).wrapping_add((1 as i32))).wrapping_mul((32 as i32))).wrapping_add((((*(tab).offset(((i).wrapping_add((2 as i32))) as isize)) as i32)).wrapping_shr(((5 as i32)) as u32)); offset = assigned; assigned };
vm_block = 5; continue;
}
// C line 1835
7 => {
let _ = { let assigned = ((((*(tab).offset((i) as isize)) as i32)).wrapping_add((((*(tab).offset(((i).wrapping_add((1 as i32))) as isize)) as i32)).wrapping_shl(((8 as i32)) as u32))).wrapping_add((((((*(tab).offset(((i).wrapping_add((2 as i32))) as isize)) as i32)) & ((31 as i32)))).wrapping_shl(((16 as i32)) as u32)); code = assigned; assigned };
vm_block = 6; continue;
}
// C line 1834
8 => {
let _ = { let assigned = (0 as i32); i = assigned; assigned };
vm_block = 2; continue;
}
// C line 1833
9 => {
let _ = tool_fprintf(f, c"static const uint8_t %s[%d] = {\n".as_ptr(), &[ToolPrintArg::Str(cname as *const c_char), ToolPrintArg::Int(len as u64)]);
vm_block = 8; continue;
}
// C line 1832
10 => {
let _ = { total_index_bytes = (total_index_bytes).wrapping_add(((len) as u32)); total_index_bytes };
vm_block = 9; continue;
}
// C line 1831
11 => {
let _ = { let old = total_index; total_index = (total_index).wrapping_add(1); old };
vm_block = 10; continue;
}
_ => std::process::abort(),
} }
}

#[cfg(all(not(feature="unicode-dump-table-size")))]
// Generated from the official C AST; original goto CFG retained.
// c: unicode_gen.c:1846. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn build_prop_table(mut f: *mut FILE, mut name: *const c_char, mut prop_index: i32, mut add_index: i32) -> () {
let mut vm_local_storage = Vec::<u64>::new();
let mut i: i32 = core::mem::zeroed();
let mut j: i32 = core::mem::zeroed();
let mut n: i32 = core::mem::zeroed();
let mut v: i32 = core::mem::zeroed();
let mut offset: i32 = core::mem::zeroed();
let mut code: i32 = core::mem::zeroed();
let mut dbuf_s: DynBuf = core::mem::zeroed();
let mut dbuf: *mut DynBuf = core::mem::zeroed();
let mut dbuf1_s: DynBuf = core::mem::zeroed();
let mut dbuf1: *mut DynBuf = core::mem::zeroed();
let mut dbuf2_s: DynBuf = core::mem::zeroed();
let mut dbuf2: *mut DynBuf = core::mem::zeroed();
let mut buf: *const u32 = core::mem::zeroed();
let mut buf_len: i32 = core::mem::zeroed();
let mut block_end_pos: i32 = core::mem::zeroed();
let mut bit: i32 = core::mem::zeroed();
let mut cname: [c_char; 128] = core::mem::zeroed();
let mut vm_block: usize = 66;
loop { match vm_block {
// C line ?
0 => {
return ();
}
// C line 1951
1 => {
let _ = dbuf_free(dbuf2);
vm_block = 0; continue;
}
// C line 1950
2 => {
let _ = dbuf_free(dbuf1);
vm_block = 1; continue;
}
// C line 1949
3 => {
let _ = dbuf_free(dbuf);
vm_block = 2; continue;
}
// C line 1946
4 => {
let _ = dump_index_table(f, (cname).as_mut_ptr(), (*(dbuf2)).buf, (((*(dbuf2)).size) as i32));
vm_block = 3; continue;
}
// C line 1945
5 => {
let _ = tool_snprintf((cname).as_mut_ptr(), (size_of::<[c_char; 128]>() as usize), c"unicode_prop_%s_index".as_ptr(), &[ToolPrintArg::Str(*((unicode_prop_name).as_mut_ptr()).offset((prop_index) as isize) as *const c_char)]);
vm_block = 4; continue;
}
// C line 1944
6 => {
vm_block = if (add_index) != 0 { 5 } else { 3 }; continue;
}
// C line 1943
7 => {
let _ = dump_byte_table(f, (cname).as_mut_ptr(), (*(dbuf)).buf, (((*(dbuf)).size) as i32));
vm_block = 6; continue;
}
// C line 1942
8 => {
let _ = tool_snprintf((cname).as_mut_ptr(), (size_of::<[c_char; 128]>() as usize), c"unicode_prop_%s_table".as_ptr(), &[ToolPrintArg::Str(*((unicode_prop_name).as_mut_ptr()).offset((prop_index) as isize) as *const c_char)]);
vm_block = 7; continue;
}
// C line 1935
9 => {
let _ = dbuf_putc(dbuf2, (((v).wrapping_shr(((16 as i32)) as u32)) as u8));
vm_block = 8; continue;
}
// C line 1934
10 => {
let _ = dbuf_putc(dbuf2, (((v).wrapping_shr(((8 as i32)) as u32)) as u8));
vm_block = 9; continue;
}
// C line 1933
11 => {
let _ = dbuf_putc(dbuf2, ((v) as u8));
vm_block = 10; continue;
}
// C line 1932
12 => {
let _ = { let assigned = code; v = assigned; assigned };
vm_block = 11; continue;
}
// C line 1930
13 => {
vm_block = if (add_index) != 0 { 12 } else { 8 }; continue;
}
// C line 1884
14 => {
vm_block = if ((((i) < (buf_len)) as i32)) != 0 { 42 } else { 13 }; continue;
}
// C line 1913
15 => {
let _ = { i = (i).wrapping_add((2 as i32)); i };
vm_block = 14; continue;
}
// C line 1912
16 => {
let _ = dbuf_putc(dbuf, (((((((v).wrapping_shl(((3 as i32)) as u32)) as u32)) | (*(buf).offset(((i).wrapping_add((1 as i32))) as isize)))) as u8));
vm_block = 15; continue;
}
// C line 1911
17 => {
let _ = { bit = ((bit) ^ ((1 as i32))); bit };
vm_block = 16; continue;
}
// C line 1910
18 => {
let _ = { code = (((((code) as u32)).wrapping_add((*(buf).offset(((i).wrapping_add((1 as i32))) as isize)).wrapping_add((((1 as i32)) as u32)))) as i32); code };
vm_block = 17; continue;
}
// C line 1916
19 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 14; continue;
}
// C line 1915
20 => {
let _ = dbuf_putc(dbuf, ((((128 as i32)).wrapping_add(v)) as u8));
vm_block = 19; continue;
}
// C line 1920
21 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 14; continue;
}
// C line 1919
22 => {
let _ = dbuf_putc(dbuf, ((v) as u8));
vm_block = 21; continue;
}
// C line 1918
23 => {
let _ = dbuf_putc(dbuf, ((((64 as i32)).wrapping_add((v).wrapping_shr(((8 as i32)) as u32))) as u8));
vm_block = 22; continue;
}
// C line 1926
24 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 14; continue;
}
// C line 1925
25 => {
let _ = dbuf_putc(dbuf, ((v) as u8));
vm_block = 24; continue;
}
// C line 1924
26 => {
let _ = dbuf_putc(dbuf, (((v).wrapping_shr(((8 as i32)) as u32)) as u8));
vm_block = 25; continue;
}
// C line 1923
27 => {
let _ = dbuf_putc(dbuf, ((((96 as i32)).wrapping_add((v).wrapping_shr(((16 as i32)) as u32))) as u8));
vm_block = 26; continue;
}
// C line 1922
28 => {
let _ = if ((((!(((((v) < (((1 as i32)).wrapping_shl(((21 as i32)) as u32))) as i32)) != 0) as i32)) as i64)) != 0 { tool_assert_rtn(c"build_prop_table".as_ptr(), c"unicode_gen.c".as_ptr(), (1922 as i32), c"v < (1 << 21)".as_ptr()) } else { { let _ = (0 as i32); } };
vm_block = 27; continue;
}
// C line 1917
29 => {
vm_block = if ((((v) < (((1 as i32)).wrapping_shl(((13 as i32)) as u32))) as i32)) != 0 { 23 } else { 28 }; continue;
}
// C line 1914
30 => {
vm_block = if ((((v) < ((128 as i32))) as i32)) != 0 { 20 } else { 29 }; continue;
}
// C line 1909
31 => {
vm_block = if ((((((((((((v) < ((8 as i32))) as i32)) != 0) && ((((((i).wrapping_add((1 as i32))) < (buf_len)) as i32)) != 0)) as i32)) != 0) && (((((*(buf).offset(((i).wrapping_add((1 as i32))) as isize)) < ((((8 as i32)) as u32))) as i32)) != 0)) as i32)) != 0 { 18 } else { 30 }; continue;
}
// C line 1908
32 => {
let _ = { bit = ((bit) ^ ((1 as i32))); bit };
vm_block = 31; continue;
}
// C line 1907
33 => {
let _ = { code = (code).wrapping_add((v).wrapping_add((1 as i32))); code };
vm_block = 32; continue;
}
// C line 1906
34 => {
let _ = { let assigned = ((*(buf).offset((i) as isize)) as i32); v = assigned; assigned };
vm_block = 33; continue;
}
// C line 1895
35 => {
let _ = { block_end_pos = (block_end_pos).wrapping_add((32 as i32)); block_end_pos };
vm_block = 34; continue;
}
// C line 1894
36 => {
let _ = dbuf_putc(dbuf2, (((v).wrapping_shr(((16 as i32)) as u32)) as u8));
vm_block = 35; continue;
}
// C line 1893
37 => {
let _ = dbuf_putc(dbuf2, (((v).wrapping_shr(((8 as i32)) as u32)) as u8));
vm_block = 36; continue;
}
// C line 1892
38 => {
let _ = dbuf_putc(dbuf2, ((v) as u8));
vm_block = 37; continue;
}
// C line 1891
39 => {
let _ = { let assigned = ((code) | ((offset).wrapping_shl(((21 as i32)) as u32))); v = assigned; assigned };
vm_block = 38; continue;
}
// C line 1890
40 => {
let _ = if ((((!(((((offset) <= ((7 as i32))) as i32)) != 0) as i32)) as i64)) != 0 { tool_assert_rtn(c"build_prop_table".as_ptr(), c"unicode_gen.c".as_ptr(), (1890 as i32), c"offset <= 7".as_ptr()) } else { { let _ = (0 as i32); } };
vm_block = 39; continue;
}
// C line 1886
41 => {
let _ = { let assigned = ((((*(dbuf)).size).wrapping_sub(((block_end_pos) as usize))) as i32); offset = assigned; assigned };
vm_block = 40; continue;
}
// C line 1885
42 => {
vm_block = if (((((((((add_index) != 0) && ((((((*(dbuf)).size) >= (((block_end_pos) as usize))) as i32)) != 0)) as i32)) != 0) && (((((bit) == ((0 as i32))) as i32)) != 0)) as i32)) != 0 { 41 } else { 34 }; continue;
}
// C line 1883
43 => {
let _ = { let assigned = (0 as i32); bit = assigned; assigned };
vm_block = 14; continue;
}
// C line 1882
44 => {
let _ = { let assigned = (0 as i32); code = assigned; assigned };
vm_block = 43; continue;
}
// C line 1881
45 => {
let _ = { let assigned = (0 as i32); i = assigned; assigned };
vm_block = 44; continue;
}
// C line 1880
46 => {
let _ = { let assigned = (32 as i32); block_end_pos = assigned; assigned };
vm_block = 45; continue;
}
// C line 1878
47 => {
let _ = if ((((!(((((get_prop((((0 as i32)) as u32), prop_index)) == ((0 as i32))) as i32)) != 0) as i32)) as i64)) != 0 { tool_assert_rtn(c"build_prop_table".as_ptr(), c"unicode_gen.c".as_ptr(), (1878 as i32), c"get_prop(0, prop_index) == 0".as_ptr()) } else { { let _ = (0 as i32); } };
vm_block = 46; continue;
}
// C line 1875
48 => {
let _ = { let assigned = (((((*(dbuf1)).size) / ((size_of::<u32>() as usize)))) as i32); buf_len = assigned; assigned };
vm_block = 47; continue;
}
// C line 1874
49 => {
let _ = { let assigned = (((*(dbuf1)).buf) as *mut u32); buf = assigned; assigned };
vm_block = 48; continue;
}
// C line 1873
50 => {
let _ = dbuf_init(dbuf2);
vm_block = 49; continue;
}
// C line 1872
51 => {
let _ = dbuf_init(dbuf);
vm_block = 50; continue;
}
// C line 1858
52 => {
vm_block = if ((((i) <= ((1114111 as i32))) as i32)) != 0 { 61 } else { 51 }; continue;
}
// C line 1869
53 => {
let _ = { i = (i).wrapping_add(n); i };
vm_block = 52; continue;
}
// C line 1868
54 => {
let _ = dbuf_put_u32(dbuf1, (((n).wrapping_sub((1 as i32))) as u32));
vm_block = 53; continue;
}
// C line 1866
55 => {
vm_block = 51; continue;
}
// C line 1865
56 => {
vm_block = if ((((((((j) == (((1114111 as i32)).wrapping_add((1 as i32)))) as i32)) != 0) && (((((v) == ((0 as i32))) as i32)) != 0)) as i32)) != 0 { 55 } else { 54 }; continue;
}
// C line 1864
57 => {
let _ = { let assigned = (j).wrapping_sub(i); n = assigned; assigned };
vm_block = 56; continue;
}
// C line 1861
58 => {
vm_block = if ((((((((j) <= ((1114111 as i32))) as i32)) != 0) && (((((get_prop(((j) as u32), prop_index)) == (v)) as i32)) != 0)) as i32)) != 0 { 59 } else { 57 }; continue;
}
// C line 1862
59 => {
let _ = { let old = j; j = (j).wrapping_add(1); old };
vm_block = 58; continue;
}
// C line 1860
60 => {
let _ = { let assigned = (i).wrapping_add((1 as i32)); j = assigned; assigned };
vm_block = 58; continue;
}
// C line 1859
61 => {
let _ = { let assigned = get_prop(((i) as u32), prop_index); v = assigned; assigned };
vm_block = 60; continue;
}
// C line 1858
62 => {
let _ = { let assigned = (0 as i32); i = assigned; assigned };
vm_block = 52; continue;
}
// C line 1856
63 => {
let _ = dbuf_init(dbuf1);
vm_block = 62; continue;
}
// C line 1851
64 => {
dbuf2 = ptr::addr_of_mut!(dbuf2_s);
vm_block = 63; continue;
}
// C line 1850
65 => {
dbuf1 = ptr::addr_of_mut!(dbuf1_s);
vm_block = 64; continue;
}
// C line 1849
66 => {
dbuf = ptr::addr_of_mut!(dbuf_s);
vm_block = 65; continue;
}
_ => std::process::abort(),
} }
}

#[cfg(all(feature="unicode-dump-table-size"))]
// Generated from the official C AST; original goto CFG retained.
// c: unicode_gen.c:1846. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn build_prop_table(mut f: *mut FILE, mut name: *const c_char, mut prop_index: i32, mut add_index: i32) -> () {
let mut vm_local_storage = Vec::<u64>::new();
let mut i: i32 = core::mem::zeroed();
let mut j: i32 = core::mem::zeroed();
let mut n: i32 = core::mem::zeroed();
let mut v: i32 = core::mem::zeroed();
let mut offset: i32 = core::mem::zeroed();
let mut code: i32 = core::mem::zeroed();
let mut dbuf_s: DynBuf = core::mem::zeroed();
let mut dbuf: *mut DynBuf = core::mem::zeroed();
let mut dbuf1_s: DynBuf = core::mem::zeroed();
let mut dbuf1: *mut DynBuf = core::mem::zeroed();
let mut dbuf2_s: DynBuf = core::mem::zeroed();
let mut dbuf2: *mut DynBuf = core::mem::zeroed();
let mut buf: *const u32 = core::mem::zeroed();
let mut buf_len: i32 = core::mem::zeroed();
let mut block_end_pos: i32 = core::mem::zeroed();
let mut bit: i32 = core::mem::zeroed();
let mut cname: [c_char; 128] = core::mem::zeroed();
let mut vm_block: usize = 67;
loop { match vm_block {
// C line ?
0 => {
return ();
}
// C line 1951
1 => {
let _ = dbuf_free(dbuf2);
vm_block = 0; continue;
}
// C line 1950
2 => {
let _ = dbuf_free(dbuf1);
vm_block = 1; continue;
}
// C line 1949
3 => {
let _ = dbuf_free(dbuf);
vm_block = 2; continue;
}
// C line 1946
4 => {
let _ = dump_index_table(f, (cname).as_mut_ptr(), (*(dbuf2)).buf, (((*(dbuf2)).size) as i32));
vm_block = 3; continue;
}
// C line 1945
5 => {
let _ = tool_snprintf((cname).as_mut_ptr(), (size_of::<[c_char; 128]>() as usize), c"unicode_prop_%s_index".as_ptr(), &[ToolPrintArg::Str(*((unicode_prop_name).as_mut_ptr()).offset((prop_index) as isize) as *const c_char)]);
vm_block = 4; continue;
}
// C line 1944
6 => {
vm_block = if (add_index) != 0 { 5 } else { 3 }; continue;
}
// C line 1943
7 => {
let _ = dump_byte_table(f, (cname).as_mut_ptr(), (*(dbuf)).buf, (((*(dbuf)).size) as i32));
vm_block = 6; continue;
}
// C line 1942
8 => {
let _ = tool_snprintf((cname).as_mut_ptr(), (size_of::<[c_char; 128]>() as usize), c"unicode_prop_%s_table".as_ptr(), &[ToolPrintArg::Str(*((unicode_prop_name).as_mut_ptr()).offset((prop_index) as isize) as *const c_char)]);
vm_block = 7; continue;
}
// C line 1939
9 => {
let _ = tool_printf(c"prop %s: length=%d bytes\n".as_ptr(), &[ToolPrintArg::Str(*((unicode_prop_name).as_mut_ptr()).offset((prop_index) as isize) as *const c_char), ToolPrintArg::Int(((((*(dbuf)).size).wrapping_add((*(dbuf2)).size)) as i32) as u64)]);
vm_block = 8; continue;
}
// C line 1935
10 => {
let _ = dbuf_putc(dbuf2, (((v).wrapping_shr(((16 as i32)) as u32)) as u8));
vm_block = 9; continue;
}
// C line 1934
11 => {
let _ = dbuf_putc(dbuf2, (((v).wrapping_shr(((8 as i32)) as u32)) as u8));
vm_block = 10; continue;
}
// C line 1933
12 => {
let _ = dbuf_putc(dbuf2, ((v) as u8));
vm_block = 11; continue;
}
// C line 1932
13 => {
let _ = { let assigned = code; v = assigned; assigned };
vm_block = 12; continue;
}
// C line 1930
14 => {
vm_block = if (add_index) != 0 { 13 } else { 9 }; continue;
}
// C line 1884
15 => {
vm_block = if ((((i) < (buf_len)) as i32)) != 0 { 43 } else { 14 }; continue;
}
// C line 1913
16 => {
let _ = { i = (i).wrapping_add((2 as i32)); i };
vm_block = 15; continue;
}
// C line 1912
17 => {
let _ = dbuf_putc(dbuf, (((((((v).wrapping_shl(((3 as i32)) as u32)) as u32)) | (*(buf).offset(((i).wrapping_add((1 as i32))) as isize)))) as u8));
vm_block = 16; continue;
}
// C line 1911
18 => {
let _ = { bit = ((bit) ^ ((1 as i32))); bit };
vm_block = 17; continue;
}
// C line 1910
19 => {
let _ = { code = (((((code) as u32)).wrapping_add((*(buf).offset(((i).wrapping_add((1 as i32))) as isize)).wrapping_add((((1 as i32)) as u32)))) as i32); code };
vm_block = 18; continue;
}
// C line 1916
20 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 15; continue;
}
// C line 1915
21 => {
let _ = dbuf_putc(dbuf, ((((128 as i32)).wrapping_add(v)) as u8));
vm_block = 20; continue;
}
// C line 1920
22 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 15; continue;
}
// C line 1919
23 => {
let _ = dbuf_putc(dbuf, ((v) as u8));
vm_block = 22; continue;
}
// C line 1918
24 => {
let _ = dbuf_putc(dbuf, ((((64 as i32)).wrapping_add((v).wrapping_shr(((8 as i32)) as u32))) as u8));
vm_block = 23; continue;
}
// C line 1926
25 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 15; continue;
}
// C line 1925
26 => {
let _ = dbuf_putc(dbuf, ((v) as u8));
vm_block = 25; continue;
}
// C line 1924
27 => {
let _ = dbuf_putc(dbuf, (((v).wrapping_shr(((8 as i32)) as u32)) as u8));
vm_block = 26; continue;
}
// C line 1923
28 => {
let _ = dbuf_putc(dbuf, ((((96 as i32)).wrapping_add((v).wrapping_shr(((16 as i32)) as u32))) as u8));
vm_block = 27; continue;
}
// C line 1922
29 => {
let _ = if ((((!(((((v) < (((1 as i32)).wrapping_shl(((21 as i32)) as u32))) as i32)) != 0) as i32)) as i64)) != 0 { tool_assert_rtn(c"build_prop_table".as_ptr(), c"unicode_gen.c".as_ptr(), (1922 as i32), c"v < (1 << 21)".as_ptr()) } else { { let _ = (0 as i32); } };
vm_block = 28; continue;
}
// C line 1917
30 => {
vm_block = if ((((v) < (((1 as i32)).wrapping_shl(((13 as i32)) as u32))) as i32)) != 0 { 24 } else { 29 }; continue;
}
// C line 1914
31 => {
vm_block = if ((((v) < ((128 as i32))) as i32)) != 0 { 21 } else { 30 }; continue;
}
// C line 1909
32 => {
vm_block = if ((((((((((((v) < ((8 as i32))) as i32)) != 0) && ((((((i).wrapping_add((1 as i32))) < (buf_len)) as i32)) != 0)) as i32)) != 0) && (((((*(buf).offset(((i).wrapping_add((1 as i32))) as isize)) < ((((8 as i32)) as u32))) as i32)) != 0)) as i32)) != 0 { 19 } else { 31 }; continue;
}
// C line 1908
33 => {
let _ = { bit = ((bit) ^ ((1 as i32))); bit };
vm_block = 32; continue;
}
// C line 1907
34 => {
let _ = { code = (code).wrapping_add((v).wrapping_add((1 as i32))); code };
vm_block = 33; continue;
}
// C line 1906
35 => {
let _ = { let assigned = ((*(buf).offset((i) as isize)) as i32); v = assigned; assigned };
vm_block = 34; continue;
}
// C line 1895
36 => {
let _ = { block_end_pos = (block_end_pos).wrapping_add((32 as i32)); block_end_pos };
vm_block = 35; continue;
}
// C line 1894
37 => {
let _ = dbuf_putc(dbuf2, (((v).wrapping_shr(((16 as i32)) as u32)) as u8));
vm_block = 36; continue;
}
// C line 1893
38 => {
let _ = dbuf_putc(dbuf2, (((v).wrapping_shr(((8 as i32)) as u32)) as u8));
vm_block = 37; continue;
}
// C line 1892
39 => {
let _ = dbuf_putc(dbuf2, ((v) as u8));
vm_block = 38; continue;
}
// C line 1891
40 => {
let _ = { let assigned = ((code) | ((offset).wrapping_shl(((21 as i32)) as u32))); v = assigned; assigned };
vm_block = 39; continue;
}
// C line 1890
41 => {
let _ = if ((((!(((((offset) <= ((7 as i32))) as i32)) != 0) as i32)) as i64)) != 0 { tool_assert_rtn(c"build_prop_table".as_ptr(), c"unicode_gen.c".as_ptr(), (1890 as i32), c"offset <= 7".as_ptr()) } else { { let _ = (0 as i32); } };
vm_block = 40; continue;
}
// C line 1886
42 => {
let _ = { let assigned = ((((*(dbuf)).size).wrapping_sub(((block_end_pos) as usize))) as i32); offset = assigned; assigned };
vm_block = 41; continue;
}
// C line 1885
43 => {
vm_block = if (((((((((add_index) != 0) && ((((((*(dbuf)).size) >= (((block_end_pos) as usize))) as i32)) != 0)) as i32)) != 0) && (((((bit) == ((0 as i32))) as i32)) != 0)) as i32)) != 0 { 42 } else { 35 }; continue;
}
// C line 1883
44 => {
let _ = { let assigned = (0 as i32); bit = assigned; assigned };
vm_block = 15; continue;
}
// C line 1882
45 => {
let _ = { let assigned = (0 as i32); code = assigned; assigned };
vm_block = 44; continue;
}
// C line 1881
46 => {
let _ = { let assigned = (0 as i32); i = assigned; assigned };
vm_block = 45; continue;
}
// C line 1880
47 => {
let _ = { let assigned = (32 as i32); block_end_pos = assigned; assigned };
vm_block = 46; continue;
}
// C line 1878
48 => {
let _ = if ((((!(((((get_prop((((0 as i32)) as u32), prop_index)) == ((0 as i32))) as i32)) != 0) as i32)) as i64)) != 0 { tool_assert_rtn(c"build_prop_table".as_ptr(), c"unicode_gen.c".as_ptr(), (1878 as i32), c"get_prop(0, prop_index) == 0".as_ptr()) } else { { let _ = (0 as i32); } };
vm_block = 47; continue;
}
// C line 1875
49 => {
let _ = { let assigned = (((((*(dbuf1)).size) / ((size_of::<u32>() as usize)))) as i32); buf_len = assigned; assigned };
vm_block = 48; continue;
}
// C line 1874
50 => {
let _ = { let assigned = (((*(dbuf1)).buf) as *mut u32); buf = assigned; assigned };
vm_block = 49; continue;
}
// C line 1873
51 => {
let _ = dbuf_init(dbuf2);
vm_block = 50; continue;
}
// C line 1872
52 => {
let _ = dbuf_init(dbuf);
vm_block = 51; continue;
}
// C line 1858
53 => {
vm_block = if ((((i) <= ((1114111 as i32))) as i32)) != 0 { 62 } else { 52 }; continue;
}
// C line 1869
54 => {
let _ = { i = (i).wrapping_add(n); i };
vm_block = 53; continue;
}
// C line 1868
55 => {
let _ = dbuf_put_u32(dbuf1, (((n).wrapping_sub((1 as i32))) as u32));
vm_block = 54; continue;
}
// C line 1866
56 => {
vm_block = 52; continue;
}
// C line 1865
57 => {
vm_block = if ((((((((j) == (((1114111 as i32)).wrapping_add((1 as i32)))) as i32)) != 0) && (((((v) == ((0 as i32))) as i32)) != 0)) as i32)) != 0 { 56 } else { 55 }; continue;
}
// C line 1864
58 => {
let _ = { let assigned = (j).wrapping_sub(i); n = assigned; assigned };
vm_block = 57; continue;
}
// C line 1861
59 => {
vm_block = if ((((((((j) <= ((1114111 as i32))) as i32)) != 0) && (((((get_prop(((j) as u32), prop_index)) == (v)) as i32)) != 0)) as i32)) != 0 { 60 } else { 58 }; continue;
}
// C line 1862
60 => {
let _ = { let old = j; j = (j).wrapping_add(1); old };
vm_block = 59; continue;
}
// C line 1860
61 => {
let _ = { let assigned = (i).wrapping_add((1 as i32)); j = assigned; assigned };
vm_block = 59; continue;
}
// C line 1859
62 => {
let _ = { let assigned = get_prop(((i) as u32), prop_index); v = assigned; assigned };
vm_block = 61; continue;
}
// C line 1858
63 => {
let _ = { let assigned = (0 as i32); i = assigned; assigned };
vm_block = 53; continue;
}
// C line 1856
64 => {
let _ = dbuf_init(dbuf1);
vm_block = 63; continue;
}
// C line 1851
65 => {
dbuf2 = ptr::addr_of_mut!(dbuf2_s);
vm_block = 64; continue;
}
// C line 1850
66 => {
dbuf1 = ptr::addr_of_mut!(dbuf1_s);
vm_block = 65; continue;
}
// C line 1849
67 => {
dbuf = ptr::addr_of_mut!(dbuf_s);
vm_block = 66; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: unicode_gen.c:1954. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn build_flags_tables(mut f: *mut FILE) -> () {
let mut vm_local_storage = Vec::<u64>::new();
let mut vm_block: usize = 4;
loop { match vm_block {
// C line ?
0 => {
return ();
}
// C line 1959
1 => {
let _ = build_prop_table(f, c"ID_Continue1".as_ptr(), (PROP_ID_Continue1 as i32), (TRUE as i32));
vm_block = 0; continue;
}
// C line 1958
2 => {
let _ = build_prop_table(f, c"ID_Start".as_ptr(), (PROP_ID_Start as i32), (TRUE as i32));
vm_block = 1; continue;
}
// C line 1957
3 => {
let _ = build_prop_table(f, c"Case_Ignorable".as_ptr(), (PROP_Case_Ignorable as i32), (TRUE as i32));
vm_block = 2; continue;
}
// C line 1956
4 => {
let _ = build_prop_table(f, c"Cased1".as_ptr(), (PROP_Cased1 as i32), (TRUE as i32));
vm_block = 3; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: unicode_gen.c:1962. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn dump_name_table(mut f: *mut FILE, mut cname: *const c_char, mut tab_name: *mut *const c_char, mut len: i32, mut tab_short_name: *mut *const c_char) -> () {
let mut vm_local_storage = Vec::<u64>::new();
let mut i: i32 = core::mem::zeroed();
let mut w: i32 = core::mem::zeroed();
let mut maxw: i32 = core::mem::zeroed();
let mut vm_block: usize = 19;
loop { match vm_block {
// C line ?
0 => {
return ();
}
// C line 1987
1 => {
let _ = tool_fprintf(f, c";\n\n".as_ptr(), &[]);
vm_block = 0; continue;
}
// C line 1979
2 => {
vm_block = if ((((i) < (len)) as i32)) != 0 { 8 } else { 1 }; continue;
}
// C line ?
3 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 2; continue;
}
// C line 1985
4 => {
let _ = tool_fprintf(f, c"\"%*s\"\\0\"\n".as_ptr(), &[ToolPrintArg::Int((((1 as i32)).wrapping_add(maxw)).wrapping_sub(w) as u64), ToolPrintArg::Str(c"".as_ptr() as *const c_char)]);
vm_block = 3; continue;
}
// C line 1983
5 => {
let _ = { w = (w).wrapping_add(tool_fprintf(f, c",%s".as_ptr(), &[ToolPrintArg::Str(*(tab_short_name).offset((i) as isize) as *const c_char)])); w };
vm_block = 4; continue;
}
// C line 1982
6 => {
vm_block = if ((((!(tab_short_name).is_null()) && (((((((*(*(tab_short_name).offset((i) as isize)).offset(((0 as i32)) as isize)) as i32)) != ((0 as i32))) as i32)) != 0)) as i32)) != 0 { 5 } else { 4 }; continue;
}
// C line 1981
7 => {
let _ = { let assigned = tool_fprintf(f, c"%s".as_ptr(), &[ToolPrintArg::Str(*(tab_name).offset((i) as isize) as *const c_char)]); w = assigned; assigned };
vm_block = 6; continue;
}
// C line 1980
8 => {
let _ = tool_fprintf(f, c"    \"".as_ptr(), &[]);
vm_block = 7; continue;
}
// C line 1979
9 => {
let _ = { let assigned = (0 as i32); i = assigned; assigned };
vm_block = 2; continue;
}
// C line 1978
10 => {
let _ = tool_fprintf(f, c"static const char %s[] =\n".as_ptr(), &[ToolPrintArg::Str(cname as *const c_char)]);
vm_block = 9; continue;
}
// C line 1968
11 => {
vm_block = if ((((i) < (len)) as i32)) != 0 { 17 } else { 10 }; continue;
}
// C line ?
12 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 11; continue;
}
// C line 1974
13 => {
let _ = { let assigned = w; maxw = assigned; assigned };
vm_block = 12; continue;
}
// C line 1973
14 => {
vm_block = if ((((maxw) < (w)) as i32)) != 0 { 13 } else { 12 }; continue;
}
// C line 1971
15 => {
let _ = { w = (((((w) as usize)).wrapping_add(((((1 as i32)) as usize)).wrapping_add(strlen(*(tab_short_name).offset((i) as isize))))) as i32); w };
vm_block = 14; continue;
}
// C line 1970
16 => {
vm_block = if ((((!(tab_short_name).is_null()) && (((((((*(*(tab_short_name).offset((i) as isize)).offset(((0 as i32)) as isize)) as i32)) != ((0 as i32))) as i32)) != 0)) as i32)) != 0 { 15 } else { 14 }; continue;
}
// C line 1969
17 => {
let _ = { let assigned = ((strlen(*(tab_name).offset((i) as isize))) as i32); w = assigned; assigned };
vm_block = 16; continue;
}
// C line 1968
18 => {
let _ = { let assigned = (0 as i32); i = assigned; assigned };
vm_block = 11; continue;
}
// C line 1967
19 => {
let _ = { let assigned = (0 as i32); maxw = assigned; assigned };
vm_block = 18; continue;
}
_ => std::process::abort(),
} }
}

#[cfg(all(not(feature="unicode-dump-table-size")))]
// Generated from the official C AST; original goto CFG retained.
// c: unicode_gen.c:1990. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn build_general_category_table(mut f: *mut FILE) -> () {
let mut vm_local_storage = Vec::<u64>::new();
let mut i: i32 = core::mem::zeroed();
let mut v: i32 = core::mem::zeroed();
let mut j: i32 = core::mem::zeroed();
let mut n: i32 = core::mem::zeroed();
let mut n1: i32 = core::mem::zeroed();
let mut dbuf_s: DynBuf = core::mem::zeroed();
let mut dbuf: *mut DynBuf = core::mem::zeroed();
let mut vm_block: usize = 47;
loop { match vm_block {
// C line ?
0 => {
return ();
}
// C line 2073
1 => {
let _ = dbuf_free(dbuf);
vm_block = 0; continue;
}
// C line 2071
2 => {
let _ = dump_byte_table(f, c"unicode_gc_table".as_ptr(), (*(dbuf)).buf, (((*(dbuf)).size) as i32));
vm_block = 1; continue;
}
// C line 2015
3 => {
vm_block = if ((((i) <= ((1114111 as i32))) as i32)) != 0 { 36 } else { 2 }; continue;
}
// C line 2062
4 => {
let _ = { i = (i).wrapping_add((n).wrapping_add((1 as i32))); i };
vm_block = 3; continue;
}
// C line 2039
5 => {
let _ = dbuf_putc(dbuf, (((((n).wrapping_shl(((5 as i32)) as u32)) | (v))) as u8));
vm_block = 4; continue;
}
// C line 2044
6 => {
let _ = dbuf_putc(dbuf, ((n1) as u8));
vm_block = 4; continue;
}
// C line 2043
7 => {
let _ = dbuf_putc(dbuf, ((((((15 as i32)).wrapping_shl(((5 as i32)) as u32)) | (v))) as u8));
vm_block = 6; continue;
}
// C line 2042
8 => {
let _ = if ((((!(((((n1) < ((128 as i32))) as i32)) != 0) as i32)) as i64)) != 0 { tool_assert_rtn(c"build_general_category_table".as_ptr(), c"unicode_gen.c".as_ptr(), (2042 as i32), c"n1 < 128".as_ptr()) } else { { let _ = (0 as i32); } };
vm_block = 7; continue;
}
// C line 2041
9 => {
let _ = { let assigned = (n).wrapping_sub((7 as i32)); n1 = assigned; assigned };
vm_block = 8; continue;
}
// C line 2050
10 => {
let _ = dbuf_putc(dbuf, ((n1) as u8));
vm_block = 4; continue;
}
// C line 2049
11 => {
let _ = dbuf_putc(dbuf, ((((n1).wrapping_shr(((8 as i32)) as u32)).wrapping_add((128 as i32))) as u8));
vm_block = 10; continue;
}
// C line 2048
12 => {
let _ = dbuf_putc(dbuf, ((((((15 as i32)).wrapping_shl(((5 as i32)) as u32)) | (v))) as u8));
vm_block = 11; continue;
}
// C line 2047
13 => {
let _ = if ((((!(((((n1) < (((1 as i32)).wrapping_shl(((14 as i32)) as u32))) as i32)) != 0) as i32)) as i64)) != 0 { tool_assert_rtn(c"build_general_category_table".as_ptr(), c"unicode_gen.c".as_ptr(), (2047 as i32), c"n1 < (1 << 14)".as_ptr()) } else { { let _ = (0 as i32); } };
vm_block = 12; continue;
}
// C line 2046
14 => {
let _ = { let assigned = (n).wrapping_sub(((7 as i32)).wrapping_add((128 as i32))); n1 = assigned; assigned };
vm_block = 13; continue;
}
// C line 2057
15 => {
let _ = dbuf_putc(dbuf, ((n1) as u8));
vm_block = 4; continue;
}
// C line 2056
16 => {
let _ = dbuf_putc(dbuf, (((n1).wrapping_shr(((8 as i32)) as u32)) as u8));
vm_block = 15; continue;
}
// C line 2055
17 => {
let _ = dbuf_putc(dbuf, (((((n1).wrapping_shr(((16 as i32)) as u32)).wrapping_add((128 as i32))).wrapping_add((64 as i32))) as u8));
vm_block = 16; continue;
}
// C line 2054
18 => {
let _ = dbuf_putc(dbuf, ((((((15 as i32)).wrapping_shl(((5 as i32)) as u32)) | (v))) as u8));
vm_block = 17; continue;
}
// C line 2053
19 => {
let _ = if ((((!(((((n1) < (((1 as i32)).wrapping_shl(((22 as i32)) as u32))) as i32)) != 0) as i32)) as i64)) != 0 { tool_assert_rtn(c"build_general_category_table".as_ptr(), c"unicode_gen.c".as_ptr(), (2053 as i32), c"n1 < (1 << 22)".as_ptr()) } else { { let _ = (0 as i32); } };
vm_block = 18; continue;
}
// C line 2052
20 => {
let _ = { let assigned = (n).wrapping_sub((((7 as i32)).wrapping_add((128 as i32))).wrapping_add(((1 as i32)).wrapping_shl(((14 as i32)) as u32))); n1 = assigned; assigned };
vm_block = 19; continue;
}
// C line 2045
21 => {
vm_block = if ((((n) < ((((7 as i32)).wrapping_add((128 as i32))).wrapping_add(((1 as i32)).wrapping_shl(((14 as i32)) as u32)))) as i32)) != 0 { 14 } else { 20 }; continue;
}
// C line 2040
22 => {
vm_block = if ((((n) < (((7 as i32)).wrapping_add((128 as i32)))) as i32)) != 0 { 9 } else { 21 }; continue;
}
// C line 2038
23 => {
vm_block = if ((((n) < ((7 as i32))) as i32)) != 0 { 5 } else { 22 }; continue;
}
// C line 2033
24 => {
let _ = { let old = n; n = (n).wrapping_sub(1); old };
vm_block = 23; continue;
}
// C line 2029
25 => {
let _ = { let assigned = n1; n = assigned; assigned };
vm_block = 24; continue;
}
// C line 2028
26 => {
let _ = { let assigned = (31 as i32); v = assigned; assigned };
vm_block = 25; continue;
}
// C line 2027
27 => {
vm_block = if ((((n1) > (n)) as i32)) != 0 { 26 } else { 24 }; continue;
}
// C line 2024
28 => {
vm_block = if (((((((((i).wrapping_add(n1)) <= ((1114111 as i32))) as i32)) != 0) && ((((((((*(unicode_db).offset(((i).wrapping_add(n1)) as isize)).general_category) as i32)) == ((v).wrapping_add(((n1) & ((1 as i32)))))) as i32)) != 0)) as i32)) != 0 { 29 } else { 27 }; continue;
}
// C line 2025
29 => {
let _ = { let old = n1; n1 = (n1).wrapping_add(1); old };
vm_block = 28; continue;
}
// C line 2023
30 => {
let _ = { let assigned = (1 as i32); n1 = assigned; assigned };
vm_block = 28; continue;
}
// C line 2022
31 => {
vm_block = if ((((v) == ((GCAT_Lu as i32))) as i32)) != 0 { 30 } else { 24 }; continue;
}
// C line 2020
32 => {
let _ = { let assigned = (j).wrapping_sub(i); n = assigned; assigned };
vm_block = 31; continue;
}
// C line 2018
33 => {
vm_block = if ((((((((j) <= ((1114111 as i32))) as i32)) != 0) && ((((((((*(unicode_db).offset((j) as isize)).general_category) as i32)) == (v)) as i32)) != 0)) as i32)) != 0 { 34 } else { 32 }; continue;
}
// C line 2019
34 => {
let _ = { let old = j; j = (j).wrapping_add(1); old };
vm_block = 33; continue;
}
// C line 2017
35 => {
let _ = { let assigned = (i).wrapping_add((1 as i32)); j = assigned; assigned };
vm_block = 33; continue;
}
// C line 2016
36 => {
let _ = { let assigned = (((*(unicode_db).offset((i) as isize)).general_category) as i32); v = assigned; assigned };
vm_block = 35; continue;
}
// C line 2015
37 => {
let _ = { let assigned = (0 as i32); i = assigned; assigned };
vm_block = 3; continue;
}
// C line 2009
38 => {
let _ = dbuf_init(dbuf);
vm_block = 37; continue;
}
// C line 2004
39 => {
let _ = dump_name_table(f, c"unicode_gc_name_table".as_ptr(), (unicode_gc_name).as_mut_ptr(), (GCAT_COUNT as i32), (unicode_gc_short_name).as_mut_ptr());
vm_block = 38; continue;
}
// C line 2002
40 => {
let _ = tool_fprintf(f, c"} UnicodeGCEnum;\n\n".as_ptr(), &[]);
vm_block = 39; continue;
}
// C line 2001
41 => {
let _ = tool_fprintf(f, c"    UNICODE_GC_COUNT,\n".as_ptr(), &[]);
vm_block = 40; continue;
}
// C line 1999
42 => {
vm_block = if ((((i) < ((GCAT_COUNT as i32))) as i32)) != 0 { 44 } else { 41 }; continue;
}
// C line ?
43 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 42; continue;
}
// C line 2000
44 => {
let _ = tool_fprintf(f, c"    UNICODE_GC_%s,\n".as_ptr(), &[ToolPrintArg::Str(*((unicode_gc_name).as_mut_ptr()).offset((i) as isize) as *const c_char)]);
vm_block = 43; continue;
}
// C line 1999
45 => {
let _ = { let assigned = (0 as i32); i = assigned; assigned };
vm_block = 42; continue;
}
// C line 1998
46 => {
let _ = tool_fprintf(f, c"typedef enum {\n".as_ptr(), &[]);
vm_block = 45; continue;
}
// C line 1993
47 => {
dbuf = ptr::addr_of_mut!(dbuf_s);
vm_block = 46; continue;
}
_ => std::process::abort(),
} }
}

#[cfg(all(feature="unicode-dump-table-size"))]
// Generated from the official C AST; original goto CFG retained.
// c: unicode_gen.c:1990. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn build_general_category_table(mut f: *mut FILE) -> () {
let mut vm_local_storage = Vec::<u64>::new();
let mut i: i32 = core::mem::zeroed();
let mut v: i32 = core::mem::zeroed();
let mut j: i32 = core::mem::zeroed();
let mut n: i32 = core::mem::zeroed();
let mut n1: i32 = core::mem::zeroed();
let mut dbuf_s: DynBuf = core::mem::zeroed();
let mut dbuf: *mut DynBuf = core::mem::zeroed();
let mut cw_count: i32 = core::mem::zeroed();
let mut cw_len_count: [i32; 4] = core::mem::zeroed();
let mut cw_start: i32 = core::mem::zeroed();
let mut vm_block: usize = 61;
loop { match vm_block {
// C line ?
0 => {
return ();
}
// C line 2073
1 => {
let _ = dbuf_free(dbuf);
vm_block = 0; continue;
}
// C line 2071
2 => {
let _ = dump_byte_table(f, c"unicode_gc_table".as_ptr(), (*(dbuf)).buf, (((*(dbuf)).size) as i32));
vm_block = 1; continue;
}
// C line 2068
3 => {
let _ = tool_printf(c" ], length=%d bytes\n".as_ptr(), &[ToolPrintArg::Int((((*(dbuf)).size) as i32) as u64)]);
vm_block = 2; continue;
}
// C line 2066
4 => {
vm_block = if ((((i) < ((4 as i32))) as i32)) != 0 { 6 } else { 3 }; continue;
}
// C line ?
5 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 4; continue;
}
// C line 2067
6 => {
let _ = tool_printf(c" %d".as_ptr(), &[ToolPrintArg::Int(*((cw_len_count).as_mut_ptr()).offset((i) as isize) as u64)]);
vm_block = 5; continue;
}
// C line 2066
7 => {
let _ = { let assigned = (0 as i32); i = assigned; assigned };
vm_block = 4; continue;
}
// C line 2065
8 => {
let _ = tool_printf(c"general category: %d entries [".as_ptr(), &[ToolPrintArg::Int(cw_count as u64)]);
vm_block = 7; continue;
}
// C line 2015
9 => {
vm_block = if ((((i) <= ((1114111 as i32))) as i32)) != 0 { 45 } else { 8 }; continue;
}
// C line 2062
10 => {
let _ = { i = (i).wrapping_add((n).wrapping_add((1 as i32))); i };
vm_block = 9; continue;
}
// C line 2060
11 => {
let _ = { let old = *((cw_len_count).as_mut_ptr()).offset(((((*(dbuf)).size).wrapping_sub(((cw_start) as usize))).wrapping_sub((((1 as i32)) as usize))) as isize); *((cw_len_count).as_mut_ptr()).offset(((((*(dbuf)).size).wrapping_sub(((cw_start) as usize))).wrapping_sub((((1 as i32)) as usize))) as isize) = (*((cw_len_count).as_mut_ptr()).offset(((((*(dbuf)).size).wrapping_sub(((cw_start) as usize))).wrapping_sub((((1 as i32)) as usize))) as isize)).wrapping_add(1); old };
vm_block = 10; continue;
}
// C line 2039
12 => {
let _ = dbuf_putc(dbuf, (((((n).wrapping_shl(((5 as i32)) as u32)) | (v))) as u8));
vm_block = 11; continue;
}
// C line 2044
13 => {
let _ = dbuf_putc(dbuf, ((n1) as u8));
vm_block = 11; continue;
}
// C line 2043
14 => {
let _ = dbuf_putc(dbuf, ((((((15 as i32)).wrapping_shl(((5 as i32)) as u32)) | (v))) as u8));
vm_block = 13; continue;
}
// C line 2042
15 => {
let _ = if ((((!(((((n1) < ((128 as i32))) as i32)) != 0) as i32)) as i64)) != 0 { tool_assert_rtn(c"build_general_category_table".as_ptr(), c"unicode_gen.c".as_ptr(), (2042 as i32), c"n1 < 128".as_ptr()) } else { { let _ = (0 as i32); } };
vm_block = 14; continue;
}
// C line 2041
16 => {
let _ = { let assigned = (n).wrapping_sub((7 as i32)); n1 = assigned; assigned };
vm_block = 15; continue;
}
// C line 2050
17 => {
let _ = dbuf_putc(dbuf, ((n1) as u8));
vm_block = 11; continue;
}
// C line 2049
18 => {
let _ = dbuf_putc(dbuf, ((((n1).wrapping_shr(((8 as i32)) as u32)).wrapping_add((128 as i32))) as u8));
vm_block = 17; continue;
}
// C line 2048
19 => {
let _ = dbuf_putc(dbuf, ((((((15 as i32)).wrapping_shl(((5 as i32)) as u32)) | (v))) as u8));
vm_block = 18; continue;
}
// C line 2047
20 => {
let _ = if ((((!(((((n1) < (((1 as i32)).wrapping_shl(((14 as i32)) as u32))) as i32)) != 0) as i32)) as i64)) != 0 { tool_assert_rtn(c"build_general_category_table".as_ptr(), c"unicode_gen.c".as_ptr(), (2047 as i32), c"n1 < (1 << 14)".as_ptr()) } else { { let _ = (0 as i32); } };
vm_block = 19; continue;
}
// C line 2046
21 => {
let _ = { let assigned = (n).wrapping_sub(((7 as i32)).wrapping_add((128 as i32))); n1 = assigned; assigned };
vm_block = 20; continue;
}
// C line 2057
22 => {
let _ = dbuf_putc(dbuf, ((n1) as u8));
vm_block = 11; continue;
}
// C line 2056
23 => {
let _ = dbuf_putc(dbuf, (((n1).wrapping_shr(((8 as i32)) as u32)) as u8));
vm_block = 22; continue;
}
// C line 2055
24 => {
let _ = dbuf_putc(dbuf, (((((n1).wrapping_shr(((16 as i32)) as u32)).wrapping_add((128 as i32))).wrapping_add((64 as i32))) as u8));
vm_block = 23; continue;
}
// C line 2054
25 => {
let _ = dbuf_putc(dbuf, ((((((15 as i32)).wrapping_shl(((5 as i32)) as u32)) | (v))) as u8));
vm_block = 24; continue;
}
// C line 2053
26 => {
let _ = if ((((!(((((n1) < (((1 as i32)).wrapping_shl(((22 as i32)) as u32))) as i32)) != 0) as i32)) as i64)) != 0 { tool_assert_rtn(c"build_general_category_table".as_ptr(), c"unicode_gen.c".as_ptr(), (2053 as i32), c"n1 < (1 << 22)".as_ptr()) } else { { let _ = (0 as i32); } };
vm_block = 25; continue;
}
// C line 2052
27 => {
let _ = { let assigned = (n).wrapping_sub((((7 as i32)).wrapping_add((128 as i32))).wrapping_add(((1 as i32)).wrapping_shl(((14 as i32)) as u32))); n1 = assigned; assigned };
vm_block = 26; continue;
}
// C line 2045
28 => {
vm_block = if ((((n) < ((((7 as i32)).wrapping_add((128 as i32))).wrapping_add(((1 as i32)).wrapping_shl(((14 as i32)) as u32)))) as i32)) != 0 { 21 } else { 27 }; continue;
}
// C line 2040
29 => {
vm_block = if ((((n) < (((7 as i32)).wrapping_add((128 as i32)))) as i32)) != 0 { 16 } else { 28 }; continue;
}
// C line 2038
30 => {
vm_block = if ((((n) < ((7 as i32))) as i32)) != 0 { 12 } else { 29 }; continue;
}
// C line 2036
31 => {
let _ = { let assigned = (((*(dbuf)).size) as i32); cw_start = assigned; assigned };
vm_block = 30; continue;
}
// C line 2035
32 => {
let _ = { let old = cw_count; cw_count = (cw_count).wrapping_add(1); old };
vm_block = 31; continue;
}
// C line 2033
33 => {
let _ = { let old = n; n = (n).wrapping_sub(1); old };
vm_block = 32; continue;
}
// C line 2029
34 => {
let _ = { let assigned = n1; n = assigned; assigned };
vm_block = 33; continue;
}
// C line 2028
35 => {
let _ = { let assigned = (31 as i32); v = assigned; assigned };
vm_block = 34; continue;
}
// C line 2027
36 => {
vm_block = if ((((n1) > (n)) as i32)) != 0 { 35 } else { 33 }; continue;
}
// C line 2024
37 => {
vm_block = if (((((((((i).wrapping_add(n1)) <= ((1114111 as i32))) as i32)) != 0) && ((((((((*(unicode_db).offset(((i).wrapping_add(n1)) as isize)).general_category) as i32)) == ((v).wrapping_add(((n1) & ((1 as i32)))))) as i32)) != 0)) as i32)) != 0 { 38 } else { 36 }; continue;
}
// C line 2025
38 => {
let _ = { let old = n1; n1 = (n1).wrapping_add(1); old };
vm_block = 37; continue;
}
// C line 2023
39 => {
let _ = { let assigned = (1 as i32); n1 = assigned; assigned };
vm_block = 37; continue;
}
// C line 2022
40 => {
vm_block = if ((((v) == ((GCAT_Lu as i32))) as i32)) != 0 { 39 } else { 33 }; continue;
}
// C line 2020
41 => {
let _ = { let assigned = (j).wrapping_sub(i); n = assigned; assigned };
vm_block = 40; continue;
}
// C line 2018
42 => {
vm_block = if ((((((((j) <= ((1114111 as i32))) as i32)) != 0) && ((((((((*(unicode_db).offset((j) as isize)).general_category) as i32)) == (v)) as i32)) != 0)) as i32)) != 0 { 43 } else { 41 }; continue;
}
// C line 2019
43 => {
let _ = { let old = j; j = (j).wrapping_add(1); old };
vm_block = 42; continue;
}
// C line 2017
44 => {
let _ = { let assigned = (i).wrapping_add((1 as i32)); j = assigned; assigned };
vm_block = 42; continue;
}
// C line 2016
45 => {
let _ = { let assigned = (((*(unicode_db).offset((i) as isize)).general_category) as i32); v = assigned; assigned };
vm_block = 44; continue;
}
// C line 2015
46 => {
let _ = { let assigned = (0 as i32); i = assigned; assigned };
vm_block = 9; continue;
}
// C line 2012
47 => {
vm_block = if ((((i) < ((4 as i32))) as i32)) != 0 { 49 } else { 46 }; continue;
}
// C line ?
48 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 47; continue;
}
// C line 2013
49 => {
let _ = { let assigned = (0 as i32); *((cw_len_count).as_mut_ptr()).offset((i) as isize) = assigned; assigned };
vm_block = 48; continue;
}
// C line 2012
50 => {
let _ = { let assigned = (0 as i32); i = assigned; assigned };
vm_block = 47; continue;
}
// C line 2011
51 => {
let _ = { let assigned = (0 as i32); cw_count = assigned; assigned };
vm_block = 50; continue;
}
// C line 2009
52 => {
let _ = dbuf_init(dbuf);
vm_block = 51; continue;
}
// C line 2004
53 => {
let _ = dump_name_table(f, c"unicode_gc_name_table".as_ptr(), (unicode_gc_name).as_mut_ptr(), (GCAT_COUNT as i32), (unicode_gc_short_name).as_mut_ptr());
vm_block = 52; continue;
}
// C line 2002
54 => {
let _ = tool_fprintf(f, c"} UnicodeGCEnum;\n\n".as_ptr(), &[]);
vm_block = 53; continue;
}
// C line 2001
55 => {
let _ = tool_fprintf(f, c"    UNICODE_GC_COUNT,\n".as_ptr(), &[]);
vm_block = 54; continue;
}
// C line 1999
56 => {
vm_block = if ((((i) < ((GCAT_COUNT as i32))) as i32)) != 0 { 58 } else { 55 }; continue;
}
// C line ?
57 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 56; continue;
}
// C line 2000
58 => {
let _ = tool_fprintf(f, c"    UNICODE_GC_%s,\n".as_ptr(), &[ToolPrintArg::Str(*((unicode_gc_name).as_mut_ptr()).offset((i) as isize) as *const c_char)]);
vm_block = 57; continue;
}
// C line 1999
59 => {
let _ = { let assigned = (0 as i32); i = assigned; assigned };
vm_block = 56; continue;
}
// C line 1998
60 => {
let _ = tool_fprintf(f, c"typedef enum {\n".as_ptr(), &[]);
vm_block = 59; continue;
}
// C line 1993
61 => {
dbuf = ptr::addr_of_mut!(dbuf_s);
vm_block = 60; continue;
}
_ => std::process::abort(),
} }
}

#[cfg(all(not(feature="unicode-dump-table-size")))]
// Generated from the official C AST; original goto CFG retained.
// c: unicode_gen.c:2076. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn build_script_table(mut f: *mut FILE) -> () {
let mut vm_local_storage = Vec::<u64>::new();
let mut i: i32 = core::mem::zeroed();
let mut v: i32 = core::mem::zeroed();
let mut j: i32 = core::mem::zeroed();
let mut n: i32 = core::mem::zeroed();
let mut n1: i32 = core::mem::zeroed();
let mut v_type: i32 = core::mem::zeroed();
let mut dbuf_s: DynBuf = core::mem::zeroed();
let mut dbuf: *mut DynBuf = core::mem::zeroed();
let mut vm_block: usize = 40;
loop { match vm_block {
// C line ?
0 => {
return ();
}
// C line 2149
1 => {
let _ = dbuf_free(dbuf);
vm_block = 0; continue;
}
// C line 2147
2 => {
let _ = dump_byte_table(f, c"unicode_script_table".as_ptr(), (*(dbuf)).buf, (((*(dbuf)).size) as i32));
vm_block = 1; continue;
}
// C line 2100
3 => {
vm_block = if ((((i) <= ((1114111 as i32))) as i32)) != 0 { 29 } else { 2 }; continue;
}
// C line 2138
4 => {
let _ = { i = (i).wrapping_add((n).wrapping_add((1 as i32))); i };
vm_block = 3; continue;
}
// C line 2133
5 => {
let _ = dbuf_putc(dbuf, ((v) as u8));
vm_block = 4; continue;
}
// C line 2132
6 => {
vm_block = if ((((v_type) != ((0 as i32))) as i32)) != 0 { 5 } else { 4 }; continue;
}
// C line 2119
7 => {
let _ = dbuf_putc(dbuf, ((((n) | ((v_type).wrapping_shl(((7 as i32)) as u32)))) as u8));
vm_block = 6; continue;
}
// C line 2124
8 => {
let _ = dbuf_putc(dbuf, ((n1) as u8));
vm_block = 6; continue;
}
// C line 2123
9 => {
let _ = dbuf_putc(dbuf, ((((((n1).wrapping_shr(((8 as i32)) as u32)).wrapping_add((96 as i32))) | ((v_type).wrapping_shl(((7 as i32)) as u32)))) as u8));
vm_block = 8; continue;
}
// C line 2122
10 => {
let _ = if ((((!(((((n1) < (((1 as i32)).wrapping_shl(((12 as i32)) as u32))) as i32)) != 0) as i32)) as i64)) != 0 { tool_assert_rtn(c"build_script_table".as_ptr(), c"unicode_gen.c".as_ptr(), (2122 as i32), c"n1 < (1 << 12)".as_ptr()) } else { { let _ = (0 as i32); } };
vm_block = 9; continue;
}
// C line 2121
11 => {
let _ = { let assigned = (n).wrapping_sub((96 as i32)); n1 = assigned; assigned };
vm_block = 10; continue;
}
// C line 2130
12 => {
let _ = dbuf_putc(dbuf, ((n1) as u8));
vm_block = 6; continue;
}
// C line 2129
13 => {
let _ = dbuf_putc(dbuf, (((n1).wrapping_shr(((8 as i32)) as u32)) as u8));
vm_block = 12; continue;
}
// C line 2128
14 => {
let _ = dbuf_putc(dbuf, ((((((n1).wrapping_shr(((16 as i32)) as u32)).wrapping_add((112 as i32))) | ((v_type).wrapping_shl(((7 as i32)) as u32)))) as u8));
vm_block = 13; continue;
}
// C line 2127
15 => {
let _ = if ((((!(((((n1) < (((1 as i32)).wrapping_shl(((20 as i32)) as u32))) as i32)) != 0) as i32)) as i64)) != 0 { tool_assert_rtn(c"build_script_table".as_ptr(), c"unicode_gen.c".as_ptr(), (2127 as i32), c"n1 < (1 << 20)".as_ptr()) } else { { let _ = (0 as i32); } };
vm_block = 14; continue;
}
// C line 2126
16 => {
let _ = { let assigned = (n).wrapping_sub(((96 as i32)).wrapping_add(((1 as i32)).wrapping_shl(((12 as i32)) as u32))); n1 = assigned; assigned };
vm_block = 15; continue;
}
// C line 2120
17 => {
vm_block = if ((((n) < (((96 as i32)).wrapping_add(((1 as i32)).wrapping_shl(((12 as i32)) as u32)))) as i32)) != 0 { 11 } else { 16 }; continue;
}
// C line 2118
18 => {
vm_block = if ((((n) < ((96 as i32))) as i32)) != 0 { 7 } else { 17 }; continue;
}
// C line 2115
19 => {
let _ = { let assigned = (0 as i32); v_type = assigned; assigned };
vm_block = 18; continue;
}
// C line 2117
20 => {
let _ = { let assigned = (1 as i32); v_type = assigned; assigned };
vm_block = 18; continue;
}
// C line 2114
21 => {
vm_block = if ((((v) == ((0 as i32))) as i32)) != 0 { 19 } else { 20 }; continue;
}
// C line 2109
22 => {
let _ = { let old = n; n = (n).wrapping_sub(1); old };
vm_block = 21; continue;
}
// C line 2107
23 => {
vm_block = 2; continue;
}
// C line 2106
24 => {
vm_block = if ((((((((v) == ((0 as i32))) as i32)) != 0) && (((((j) == (((1114111 as i32)).wrapping_add((1 as i32)))) as i32)) != 0)) as i32)) != 0 { 23 } else { 22 }; continue;
}
// C line 2105
25 => {
let _ = { let assigned = (j).wrapping_sub(i); n = assigned; assigned };
vm_block = 24; continue;
}
// C line 2103
26 => {
vm_block = if ((((((((j) <= ((1114111 as i32))) as i32)) != 0) && ((((((((*(unicode_db).offset((j) as isize)).script) as i32)) == (v)) as i32)) != 0)) as i32)) != 0 { 27 } else { 25 }; continue;
}
// C line 2104
27 => {
let _ = { let old = j; j = (j).wrapping_add(1); old };
vm_block = 26; continue;
}
// C line 2102
28 => {
let _ = { let assigned = (i).wrapping_add((1 as i32)); j = assigned; assigned };
vm_block = 26; continue;
}
// C line 2101
29 => {
let _ = { let assigned = (((*(unicode_db).offset((i) as isize)).script) as i32); v = assigned; assigned };
vm_block = 28; continue;
}
// C line 2100
30 => {
let _ = { let assigned = (0 as i32); i = assigned; assigned };
vm_block = 3; continue;
}
// C line 2094
31 => {
let _ = dbuf_init(dbuf);
vm_block = 30; continue;
}
// C line 2090
32 => {
let _ = dump_name_table(f, c"unicode_script_name_table".as_ptr(), (unicode_script_name).as_mut_ptr(), (SCRIPT_COUNT as i32), (unicode_script_short_name).as_mut_ptr());
vm_block = 31; continue;
}
// C line 2088
33 => {
let _ = tool_fprintf(f, c"} UnicodeScriptEnum;\n\n".as_ptr(), &[]);
vm_block = 32; continue;
}
// C line 2087
34 => {
let _ = tool_fprintf(f, c"    UNICODE_SCRIPT_COUNT,\n".as_ptr(), &[]);
vm_block = 33; continue;
}
// C line 2085
35 => {
vm_block = if ((((i) < ((SCRIPT_COUNT as i32))) as i32)) != 0 { 37 } else { 34 }; continue;
}
// C line ?
36 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 35; continue;
}
// C line 2086
37 => {
let _ = tool_fprintf(f, c"    UNICODE_SCRIPT_%s,\n".as_ptr(), &[ToolPrintArg::Str(*((unicode_script_name).as_mut_ptr()).offset((i) as isize) as *const c_char)]);
vm_block = 36; continue;
}
// C line 2085
38 => {
let _ = { let assigned = (0 as i32); i = assigned; assigned };
vm_block = 35; continue;
}
// C line 2084
39 => {
let _ = tool_fprintf(f, c"typedef enum {\n".as_ptr(), &[]);
vm_block = 38; continue;
}
// C line 2079
40 => {
dbuf = ptr::addr_of_mut!(dbuf_s);
vm_block = 39; continue;
}
_ => std::process::abort(),
} }
}

#[cfg(all(feature="unicode-dump-table-size"))]
// Generated from the official C AST; original goto CFG retained.
// c: unicode_gen.c:2076. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn build_script_table(mut f: *mut FILE) -> () {
let mut vm_local_storage = Vec::<u64>::new();
let mut i: i32 = core::mem::zeroed();
let mut v: i32 = core::mem::zeroed();
let mut j: i32 = core::mem::zeroed();
let mut n: i32 = core::mem::zeroed();
let mut n1: i32 = core::mem::zeroed();
let mut v_type: i32 = core::mem::zeroed();
let mut dbuf_s: DynBuf = core::mem::zeroed();
let mut dbuf: *mut DynBuf = core::mem::zeroed();
let mut cw_count: i32 = core::mem::zeroed();
let mut cw_len_count: [i32; 4] = core::mem::zeroed();
let mut cw_start: i32 = core::mem::zeroed();
let mut vm_block: usize = 54;
loop { match vm_block {
// C line ?
0 => {
return ();
}
// C line 2149
1 => {
let _ = dbuf_free(dbuf);
vm_block = 0; continue;
}
// C line 2147
2 => {
let _ = dump_byte_table(f, c"unicode_script_table".as_ptr(), (*(dbuf)).buf, (((*(dbuf)).size) as i32));
vm_block = 1; continue;
}
// C line 2144
3 => {
let _ = tool_printf(c" ], length=%d bytes\n".as_ptr(), &[ToolPrintArg::Int((((*(dbuf)).size) as i32) as u64)]);
vm_block = 2; continue;
}
// C line 2142
4 => {
vm_block = if ((((i) < ((4 as i32))) as i32)) != 0 { 6 } else { 3 }; continue;
}
// C line ?
5 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 4; continue;
}
// C line 2143
6 => {
let _ = tool_printf(c" %d".as_ptr(), &[ToolPrintArg::Int(*((cw_len_count).as_mut_ptr()).offset((i) as isize) as u64)]);
vm_block = 5; continue;
}
// C line 2142
7 => {
let _ = { let assigned = (0 as i32); i = assigned; assigned };
vm_block = 4; continue;
}
// C line 2141
8 => {
let _ = tool_printf(c"script: %d entries [".as_ptr(), &[ToolPrintArg::Int(cw_count as u64)]);
vm_block = 7; continue;
}
// C line 2100
9 => {
vm_block = if ((((i) <= ((1114111 as i32))) as i32)) != 0 { 38 } else { 8 }; continue;
}
// C line 2138
10 => {
let _ = { i = (i).wrapping_add((n).wrapping_add((1 as i32))); i };
vm_block = 9; continue;
}
// C line 2136
11 => {
let _ = { let old = *((cw_len_count).as_mut_ptr()).offset(((((*(dbuf)).size).wrapping_sub(((cw_start) as usize))).wrapping_sub((((1 as i32)) as usize))) as isize); *((cw_len_count).as_mut_ptr()).offset(((((*(dbuf)).size).wrapping_sub(((cw_start) as usize))).wrapping_sub((((1 as i32)) as usize))) as isize) = (*((cw_len_count).as_mut_ptr()).offset(((((*(dbuf)).size).wrapping_sub(((cw_start) as usize))).wrapping_sub((((1 as i32)) as usize))) as isize)).wrapping_add(1); old };
vm_block = 10; continue;
}
// C line 2133
12 => {
let _ = dbuf_putc(dbuf, ((v) as u8));
vm_block = 11; continue;
}
// C line 2132
13 => {
vm_block = if ((((v_type) != ((0 as i32))) as i32)) != 0 { 12 } else { 11 }; continue;
}
// C line 2119
14 => {
let _ = dbuf_putc(dbuf, ((((n) | ((v_type).wrapping_shl(((7 as i32)) as u32)))) as u8));
vm_block = 13; continue;
}
// C line 2124
15 => {
let _ = dbuf_putc(dbuf, ((n1) as u8));
vm_block = 13; continue;
}
// C line 2123
16 => {
let _ = dbuf_putc(dbuf, ((((((n1).wrapping_shr(((8 as i32)) as u32)).wrapping_add((96 as i32))) | ((v_type).wrapping_shl(((7 as i32)) as u32)))) as u8));
vm_block = 15; continue;
}
// C line 2122
17 => {
let _ = if ((((!(((((n1) < (((1 as i32)).wrapping_shl(((12 as i32)) as u32))) as i32)) != 0) as i32)) as i64)) != 0 { tool_assert_rtn(c"build_script_table".as_ptr(), c"unicode_gen.c".as_ptr(), (2122 as i32), c"n1 < (1 << 12)".as_ptr()) } else { { let _ = (0 as i32); } };
vm_block = 16; continue;
}
// C line 2121
18 => {
let _ = { let assigned = (n).wrapping_sub((96 as i32)); n1 = assigned; assigned };
vm_block = 17; continue;
}
// C line 2130
19 => {
let _ = dbuf_putc(dbuf, ((n1) as u8));
vm_block = 13; continue;
}
// C line 2129
20 => {
let _ = dbuf_putc(dbuf, (((n1).wrapping_shr(((8 as i32)) as u32)) as u8));
vm_block = 19; continue;
}
// C line 2128
21 => {
let _ = dbuf_putc(dbuf, ((((((n1).wrapping_shr(((16 as i32)) as u32)).wrapping_add((112 as i32))) | ((v_type).wrapping_shl(((7 as i32)) as u32)))) as u8));
vm_block = 20; continue;
}
// C line 2127
22 => {
let _ = if ((((!(((((n1) < (((1 as i32)).wrapping_shl(((20 as i32)) as u32))) as i32)) != 0) as i32)) as i64)) != 0 { tool_assert_rtn(c"build_script_table".as_ptr(), c"unicode_gen.c".as_ptr(), (2127 as i32), c"n1 < (1 << 20)".as_ptr()) } else { { let _ = (0 as i32); } };
vm_block = 21; continue;
}
// C line 2126
23 => {
let _ = { let assigned = (n).wrapping_sub(((96 as i32)).wrapping_add(((1 as i32)).wrapping_shl(((12 as i32)) as u32))); n1 = assigned; assigned };
vm_block = 22; continue;
}
// C line 2120
24 => {
vm_block = if ((((n) < (((96 as i32)).wrapping_add(((1 as i32)).wrapping_shl(((12 as i32)) as u32)))) as i32)) != 0 { 18 } else { 23 }; continue;
}
// C line 2118
25 => {
vm_block = if ((((n) < ((96 as i32))) as i32)) != 0 { 14 } else { 24 }; continue;
}
// C line 2115
26 => {
let _ = { let assigned = (0 as i32); v_type = assigned; assigned };
vm_block = 25; continue;
}
// C line 2117
27 => {
let _ = { let assigned = (1 as i32); v_type = assigned; assigned };
vm_block = 25; continue;
}
// C line 2114
28 => {
vm_block = if ((((v) == ((0 as i32))) as i32)) != 0 { 26 } else { 27 }; continue;
}
// C line 2112
29 => {
let _ = { let assigned = (((*(dbuf)).size) as i32); cw_start = assigned; assigned };
vm_block = 28; continue;
}
// C line 2111
30 => {
let _ = { let old = cw_count; cw_count = (cw_count).wrapping_add(1); old };
vm_block = 29; continue;
}
// C line 2109
31 => {
let _ = { let old = n; n = (n).wrapping_sub(1); old };
vm_block = 30; continue;
}
// C line 2107
32 => {
vm_block = 8; continue;
}
// C line 2106
33 => {
vm_block = if ((((((((v) == ((0 as i32))) as i32)) != 0) && (((((j) == (((1114111 as i32)).wrapping_add((1 as i32)))) as i32)) != 0)) as i32)) != 0 { 32 } else { 31 }; continue;
}
// C line 2105
34 => {
let _ = { let assigned = (j).wrapping_sub(i); n = assigned; assigned };
vm_block = 33; continue;
}
// C line 2103
35 => {
vm_block = if ((((((((j) <= ((1114111 as i32))) as i32)) != 0) && ((((((((*(unicode_db).offset((j) as isize)).script) as i32)) == (v)) as i32)) != 0)) as i32)) != 0 { 36 } else { 34 }; continue;
}
// C line 2104
36 => {
let _ = { let old = j; j = (j).wrapping_add(1); old };
vm_block = 35; continue;
}
// C line 2102
37 => {
let _ = { let assigned = (i).wrapping_add((1 as i32)); j = assigned; assigned };
vm_block = 35; continue;
}
// C line 2101
38 => {
let _ = { let assigned = (((*(unicode_db).offset((i) as isize)).script) as i32); v = assigned; assigned };
vm_block = 37; continue;
}
// C line 2100
39 => {
let _ = { let assigned = (0 as i32); i = assigned; assigned };
vm_block = 9; continue;
}
// C line 2097
40 => {
vm_block = if ((((i) < ((4 as i32))) as i32)) != 0 { 42 } else { 39 }; continue;
}
// C line ?
41 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 40; continue;
}
// C line 2098
42 => {
let _ = { let assigned = (0 as i32); *((cw_len_count).as_mut_ptr()).offset((i) as isize) = assigned; assigned };
vm_block = 41; continue;
}
// C line 2097
43 => {
let _ = { let assigned = (0 as i32); i = assigned; assigned };
vm_block = 40; continue;
}
// C line 2096
44 => {
let _ = { let assigned = (0 as i32); cw_count = assigned; assigned };
vm_block = 43; continue;
}
// C line 2094
45 => {
let _ = dbuf_init(dbuf);
vm_block = 44; continue;
}
// C line 2090
46 => {
let _ = dump_name_table(f, c"unicode_script_name_table".as_ptr(), (unicode_script_name).as_mut_ptr(), (SCRIPT_COUNT as i32), (unicode_script_short_name).as_mut_ptr());
vm_block = 45; continue;
}
// C line 2088
47 => {
let _ = tool_fprintf(f, c"} UnicodeScriptEnum;\n\n".as_ptr(), &[]);
vm_block = 46; continue;
}
// C line 2087
48 => {
let _ = tool_fprintf(f, c"    UNICODE_SCRIPT_COUNT,\n".as_ptr(), &[]);
vm_block = 47; continue;
}
// C line 2085
49 => {
vm_block = if ((((i) < ((SCRIPT_COUNT as i32))) as i32)) != 0 { 51 } else { 48 }; continue;
}
// C line ?
50 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 49; continue;
}
// C line 2086
51 => {
let _ = tool_fprintf(f, c"    UNICODE_SCRIPT_%s,\n".as_ptr(), &[ToolPrintArg::Str(*((unicode_script_name).as_mut_ptr()).offset((i) as isize) as *const c_char)]);
vm_block = 50; continue;
}
// C line 2085
52 => {
let _ = { let assigned = (0 as i32); i = assigned; assigned };
vm_block = 49; continue;
}
// C line 2084
53 => {
let _ = tool_fprintf(f, c"typedef enum {\n".as_ptr(), &[]);
vm_block = 52; continue;
}
// C line 2079
54 => {
dbuf = ptr::addr_of_mut!(dbuf_s);
vm_block = 53; continue;
}
_ => std::process::abort(),
} }
}

#[cfg(all(not(feature="unicode-dump-table-size")))]
// Generated from the official C AST; original goto CFG retained.
// c: unicode_gen.c:2152. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn build_script_ext_table(mut f: *mut FILE) -> () {
let mut vm_local_storage = Vec::<u64>::new();
let mut i: i32 = core::mem::zeroed();
let mut j: i32 = core::mem::zeroed();
let mut n: i32 = core::mem::zeroed();
let mut n1: i32 = core::mem::zeroed();
let mut script_ext_len: i32 = core::mem::zeroed();
let mut dbuf_s: DynBuf = core::mem::zeroed();
let mut dbuf: *mut DynBuf = core::mem::zeroed();
let mut vm_block: usize = 30;
loop { match vm_block {
// C line ?
0 => {
return ();
}
// C line 2201
1 => {
let _ = dbuf_free(dbuf);
vm_block = 0; continue;
}
// C line 2199
2 => {
let _ = dump_byte_table(f, c"unicode_script_ext_table".as_ptr(), (*(dbuf)).buf, (((*(dbuf)).size) as i32));
vm_block = 1; continue;
}
// C line 2161
3 => {
vm_block = if ((((i) <= ((1114111 as i32))) as i32)) != 0 { 27 } else { 2 }; continue;
}
// C line 2192
4 => {
let _ = { i = (i).wrapping_add((n).wrapping_add((1 as i32))); i };
vm_block = 3; continue;
}
// C line 2190
5 => {
vm_block = if ((((j) < (script_ext_len)) as i32)) != 0 { 7 } else { 4 }; continue;
}
// C line ?
6 => {
let _ = { let old = j; j = (j).wrapping_add(1); old };
vm_block = 5; continue;
}
// C line 2191
7 => {
let _ = dbuf_putc(dbuf, *((*(unicode_db).offset((i) as isize)).script_ext).offset((j) as isize));
vm_block = 6; continue;
}
// C line 2190
8 => {
let _ = { let assigned = (0 as i32); j = assigned; assigned };
vm_block = 5; continue;
}
// C line 2189
9 => {
let _ = dbuf_putc(dbuf, ((script_ext_len) as u8));
vm_block = 8; continue;
}
// C line 2176
10 => {
let _ = dbuf_putc(dbuf, ((n) as u8));
vm_block = 9; continue;
}
// C line 2181
11 => {
let _ = dbuf_putc(dbuf, ((n1) as u8));
vm_block = 9; continue;
}
// C line 2180
12 => {
let _ = dbuf_putc(dbuf, ((((n1).wrapping_shr(((8 as i32)) as u32)).wrapping_add((128 as i32))) as u8));
vm_block = 11; continue;
}
// C line 2179
13 => {
let _ = if ((((!(((((n1) < (((1 as i32)).wrapping_shl(((14 as i32)) as u32))) as i32)) != 0) as i32)) as i64)) != 0 { tool_assert_rtn(c"build_script_ext_table".as_ptr(), c"unicode_gen.c".as_ptr(), (2179 as i32), c"n1 < (1 << 14)".as_ptr()) } else { { let _ = (0 as i32); } };
vm_block = 12; continue;
}
// C line 2178
14 => {
let _ = { let assigned = (n).wrapping_sub((128 as i32)); n1 = assigned; assigned };
vm_block = 13; continue;
}
// C line 2187
15 => {
let _ = dbuf_putc(dbuf, ((n1) as u8));
vm_block = 9; continue;
}
// C line 2186
16 => {
let _ = dbuf_putc(dbuf, (((n1).wrapping_shr(((8 as i32)) as u32)) as u8));
vm_block = 15; continue;
}
// C line 2185
17 => {
let _ = dbuf_putc(dbuf, (((((n1).wrapping_shr(((16 as i32)) as u32)).wrapping_add((128 as i32))).wrapping_add((64 as i32))) as u8));
vm_block = 16; continue;
}
// C line 2184
18 => {
let _ = if ((((!(((((n1) < (((1 as i32)).wrapping_shl(((22 as i32)) as u32))) as i32)) != 0) as i32)) as i64)) != 0 { tool_assert_rtn(c"build_script_ext_table".as_ptr(), c"unicode_gen.c".as_ptr(), (2184 as i32), c"n1 < (1 << 22)".as_ptr()) } else { { let _ = (0 as i32); } };
vm_block = 17; continue;
}
// C line 2183
19 => {
let _ = { let assigned = (n).wrapping_sub(((128 as i32)).wrapping_add(((1 as i32)).wrapping_shl(((14 as i32)) as u32))); n1 = assigned; assigned };
vm_block = 18; continue;
}
// C line 2177
20 => {
vm_block = if ((((n) < (((128 as i32)).wrapping_add(((1 as i32)).wrapping_shl(((14 as i32)) as u32)))) as i32)) != 0 { 14 } else { 19 }; continue;
}
// C line 2175
21 => {
vm_block = if ((((n) < ((128 as i32))) as i32)) != 0 { 10 } else { 20 }; continue;
}
// C line 2174
22 => {
let _ = { let old = n; n = (n).wrapping_sub(1); old };
vm_block = 21; continue;
}
// C line 2170
23 => {
let _ = { let assigned = (j).wrapping_sub(i); n = assigned; assigned };
vm_block = 22; continue;
}
// C line 2164
24 => {
vm_block = if ((((((((((((j) <= ((1114111 as i32))) as i32)) != 0) && ((((((((*(unicode_db).offset((j) as isize)).script_ext_len) as i32)) == (script_ext_len)) as i32)) != 0)) as i32)) != 0) && (((!((memcmp((((*(unicode_db).offset((j) as isize)).script_ext) as *const c_void), (((*(unicode_db).offset((i) as isize)).script_ext) as *const c_void), ((script_ext_len) as usize))) != 0) as i32)) != 0)) as i32)) != 0 { 25 } else { 23 }; continue;
}
// C line 2168
25 => {
let _ = { let old = j; j = (j).wrapping_add(1); old };
vm_block = 24; continue;
}
// C line 2163
26 => {
let _ = { let assigned = (i).wrapping_add((1 as i32)); j = assigned; assigned };
vm_block = 24; continue;
}
// C line 2162
27 => {
let _ = { let assigned = (((*(unicode_db).offset((i) as isize)).script_ext_len) as i32); script_ext_len = assigned; assigned };
vm_block = 26; continue;
}
// C line 2161
28 => {
let _ = { let assigned = (0 as i32); i = assigned; assigned };
vm_block = 3; continue;
}
// C line 2160
29 => {
let _ = dbuf_init(dbuf);
vm_block = 28; continue;
}
// C line 2155
30 => {
dbuf = ptr::addr_of_mut!(dbuf_s);
vm_block = 29; continue;
}
_ => std::process::abort(),
} }
}

#[cfg(all(feature="unicode-dump-table-size"))]
// Generated from the official C AST; original goto CFG retained.
// c: unicode_gen.c:2152. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn build_script_ext_table(mut f: *mut FILE) -> () {
let mut vm_local_storage = Vec::<u64>::new();
let mut i: i32 = core::mem::zeroed();
let mut j: i32 = core::mem::zeroed();
let mut n: i32 = core::mem::zeroed();
let mut n1: i32 = core::mem::zeroed();
let mut script_ext_len: i32 = core::mem::zeroed();
let mut dbuf_s: DynBuf = core::mem::zeroed();
let mut dbuf: *mut DynBuf = core::mem::zeroed();
let mut cw_count: i32 = core::mem::zeroed();
let mut vm_block: usize = 34;
loop { match vm_block {
// C line ?
0 => {
return ();
}
// C line 2201
1 => {
let _ = dbuf_free(dbuf);
vm_block = 0; continue;
}
// C line 2199
2 => {
let _ = dump_byte_table(f, c"unicode_script_ext_table".as_ptr(), (*(dbuf)).buf, (((*(dbuf)).size) as i32));
vm_block = 1; continue;
}
// C line 2196
3 => {
let _ = tool_printf(c", length=%d bytes\n".as_ptr(), &[ToolPrintArg::Int((((*(dbuf)).size) as i32) as u64)]);
vm_block = 2; continue;
}
// C line 2195
4 => {
let _ = tool_printf(c"script_ext: %d entries".as_ptr(), &[ToolPrintArg::Int(cw_count as u64)]);
vm_block = 3; continue;
}
// C line 2161
5 => {
vm_block = if ((((i) <= ((1114111 as i32))) as i32)) != 0 { 30 } else { 4 }; continue;
}
// C line 2192
6 => {
let _ = { i = (i).wrapping_add((n).wrapping_add((1 as i32))); i };
vm_block = 5; continue;
}
// C line 2190
7 => {
vm_block = if ((((j) < (script_ext_len)) as i32)) != 0 { 9 } else { 6 }; continue;
}
// C line ?
8 => {
let _ = { let old = j; j = (j).wrapping_add(1); old };
vm_block = 7; continue;
}
// C line 2191
9 => {
let _ = dbuf_putc(dbuf, *((*(unicode_db).offset((i) as isize)).script_ext).offset((j) as isize));
vm_block = 8; continue;
}
// C line 2190
10 => {
let _ = { let assigned = (0 as i32); j = assigned; assigned };
vm_block = 7; continue;
}
// C line 2189
11 => {
let _ = dbuf_putc(dbuf, ((script_ext_len) as u8));
vm_block = 10; continue;
}
// C line 2176
12 => {
let _ = dbuf_putc(dbuf, ((n) as u8));
vm_block = 11; continue;
}
// C line 2181
13 => {
let _ = dbuf_putc(dbuf, ((n1) as u8));
vm_block = 11; continue;
}
// C line 2180
14 => {
let _ = dbuf_putc(dbuf, ((((n1).wrapping_shr(((8 as i32)) as u32)).wrapping_add((128 as i32))) as u8));
vm_block = 13; continue;
}
// C line 2179
15 => {
let _ = if ((((!(((((n1) < (((1 as i32)).wrapping_shl(((14 as i32)) as u32))) as i32)) != 0) as i32)) as i64)) != 0 { tool_assert_rtn(c"build_script_ext_table".as_ptr(), c"unicode_gen.c".as_ptr(), (2179 as i32), c"n1 < (1 << 14)".as_ptr()) } else { { let _ = (0 as i32); } };
vm_block = 14; continue;
}
// C line 2178
16 => {
let _ = { let assigned = (n).wrapping_sub((128 as i32)); n1 = assigned; assigned };
vm_block = 15; continue;
}
// C line 2187
17 => {
let _ = dbuf_putc(dbuf, ((n1) as u8));
vm_block = 11; continue;
}
// C line 2186
18 => {
let _ = dbuf_putc(dbuf, (((n1).wrapping_shr(((8 as i32)) as u32)) as u8));
vm_block = 17; continue;
}
// C line 2185
19 => {
let _ = dbuf_putc(dbuf, (((((n1).wrapping_shr(((16 as i32)) as u32)).wrapping_add((128 as i32))).wrapping_add((64 as i32))) as u8));
vm_block = 18; continue;
}
// C line 2184
20 => {
let _ = if ((((!(((((n1) < (((1 as i32)).wrapping_shl(((22 as i32)) as u32))) as i32)) != 0) as i32)) as i64)) != 0 { tool_assert_rtn(c"build_script_ext_table".as_ptr(), c"unicode_gen.c".as_ptr(), (2184 as i32), c"n1 < (1 << 22)".as_ptr()) } else { { let _ = (0 as i32); } };
vm_block = 19; continue;
}
// C line 2183
21 => {
let _ = { let assigned = (n).wrapping_sub(((128 as i32)).wrapping_add(((1 as i32)).wrapping_shl(((14 as i32)) as u32))); n1 = assigned; assigned };
vm_block = 20; continue;
}
// C line 2177
22 => {
vm_block = if ((((n) < (((128 as i32)).wrapping_add(((1 as i32)).wrapping_shl(((14 as i32)) as u32)))) as i32)) != 0 { 16 } else { 21 }; continue;
}
// C line 2175
23 => {
vm_block = if ((((n) < ((128 as i32))) as i32)) != 0 { 12 } else { 22 }; continue;
}
// C line 2174
24 => {
let _ = { let old = n; n = (n).wrapping_sub(1); old };
vm_block = 23; continue;
}
// C line 2172
25 => {
let _ = { let old = cw_count; cw_count = (cw_count).wrapping_add(1); old };
vm_block = 24; continue;
}
// C line 2170
26 => {
let _ = { let assigned = (j).wrapping_sub(i); n = assigned; assigned };
vm_block = 25; continue;
}
// C line 2164
27 => {
vm_block = if ((((((((((((j) <= ((1114111 as i32))) as i32)) != 0) && ((((((((*(unicode_db).offset((j) as isize)).script_ext_len) as i32)) == (script_ext_len)) as i32)) != 0)) as i32)) != 0) && (((!((memcmp((((*(unicode_db).offset((j) as isize)).script_ext) as *const c_void), (((*(unicode_db).offset((i) as isize)).script_ext) as *const c_void), ((script_ext_len) as usize))) != 0) as i32)) != 0)) as i32)) != 0 { 28 } else { 26 }; continue;
}
// C line 2168
28 => {
let _ = { let old = j; j = (j).wrapping_add(1); old };
vm_block = 27; continue;
}
// C line 2163
29 => {
let _ = { let assigned = (i).wrapping_add((1 as i32)); j = assigned; assigned };
vm_block = 27; continue;
}
// C line 2162
30 => {
let _ = { let assigned = (((*(unicode_db).offset((i) as isize)).script_ext_len) as i32); script_ext_len = assigned; assigned };
vm_block = 29; continue;
}
// C line 2161
31 => {
let _ = { let assigned = (0 as i32); i = assigned; assigned };
vm_block = 5; continue;
}
// C line 2160
32 => {
let _ = dbuf_init(dbuf);
vm_block = 31; continue;
}
// C line 2157
33 => {
cw_count = (0 as i32);
vm_block = 32; continue;
}
// C line 2155
34 => {
dbuf = ptr::addr_of_mut!(dbuf_s);
vm_block = 33; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: unicode_gen.c:2207. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn build_prop_list_table(mut f: *mut FILE) -> () {
let mut vm_local_storage = Vec::<u64>::new();
let mut i: i32 = core::mem::zeroed();
let mut vm_block: usize = 26;
loop { match vm_block {
// C line ?
0 => {
return ();
}
// C line 2242
1 => {
let _ = tool_fprintf(f, c"};\n\n".as_ptr(), &[]);
vm_block = 0; continue;
}
// C line 2239
2 => {
vm_block = if ((((i) < ((PROP_ASCII as i32))) as i32)) != 0 { 4 } else { 1 }; continue;
}
// C line ?
3 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 2; continue;
}
// C line 2240
4 => {
let _ = tool_fprintf(f, c"    countof(unicode_prop_%s_table),\n".as_ptr(), &[ToolPrintArg::Str(*((unicode_prop_name).as_mut_ptr()).offset((i) as isize) as *const c_char)]);
vm_block = 3; continue;
}
// C line 2239
5 => {
let _ = { let assigned = (0 as i32); i = assigned; assigned };
vm_block = 2; continue;
}
// C line 2238
6 => {
let _ = tool_fprintf(f, c"static const uint16_t unicode_prop_len_table[] = {\n".as_ptr(), &[]);
vm_block = 5; continue;
}
// C line 2236
7 => {
let _ = tool_fprintf(f, c"};\n\n".as_ptr(), &[]);
vm_block = 6; continue;
}
// C line 2233
8 => {
vm_block = if ((((i) < ((PROP_ASCII as i32))) as i32)) != 0 { 10 } else { 7 }; continue;
}
// C line ?
9 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 8; continue;
}
// C line 2234
10 => {
let _ = tool_fprintf(f, c"    unicode_prop_%s_table,\n".as_ptr(), &[ToolPrintArg::Str(*((unicode_prop_name).as_mut_ptr()).offset((i) as isize) as *const c_char)]);
vm_block = 9; continue;
}
// C line 2233
11 => {
let _ = { let assigned = (0 as i32); i = assigned; assigned };
vm_block = 8; continue;
}
// C line 2232
12 => {
let _ = tool_fprintf(f, c"static const uint8_t * const unicode_prop_table[] = {\n".as_ptr(), &[]);
vm_block = 11; continue;
}
// C line 2228
13 => {
let _ = dump_name_table(f, c"unicode_prop_name_table".as_ptr(), ((unicode_prop_name).as_mut_ptr()).offset(((i) as isize)), (((PROP_XID_Start as i32)).wrapping_sub(i)).wrapping_add((1 as i32)), ((unicode_prop_short_name).as_mut_ptr()).offset(((i) as isize)));
vm_block = 12; continue;
}
// C line 2227
14 => {
let _ = { let assigned = (PROP_ASCII_Hex_Digit as i32); i = assigned; assigned };
vm_block = 13; continue;
}
// C line 2225
15 => {
let _ = tool_fprintf(f, c"} UnicodePropertyEnum;\n\n".as_ptr(), &[]);
vm_block = 14; continue;
}
// C line 2224
16 => {
let _ = tool_fprintf(f, c"    UNICODE_PROP_COUNT,\n".as_ptr(), &[]);
vm_block = 15; continue;
}
// C line 2222
17 => {
vm_block = if ((((i) < ((PROP_COUNT as i32))) as i32)) != 0 { 19 } else { 16 }; continue;
}
// C line ?
18 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 17; continue;
}
// C line 2223
19 => {
let _ = tool_fprintf(f, c"    UNICODE_PROP_%s,\n".as_ptr(), &[ToolPrintArg::Str(*((unicode_prop_name).as_mut_ptr()).offset((i) as isize) as *const c_char)]);
vm_block = 18; continue;
}
// C line 2222
20 => {
let _ = { let assigned = (0 as i32); i = assigned; assigned };
vm_block = 17; continue;
}
// C line 2221
21 => {
let _ = tool_fprintf(f, c"typedef enum {\n".as_ptr(), &[]);
vm_block = 20; continue;
}
// C line 2211
22 => {
vm_block = if ((((i) < ((PROP_ASCII as i32))) as i32)) != 0 { 25 } else { 21 }; continue;
}
// C line ?
23 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 22; continue;
}
// C line 2217
24 => {
let _ = build_prop_table(f, *((unicode_prop_name).as_mut_ptr()).offset((i) as isize), i, (FALSE as i32));
vm_block = 23; continue;
}
// C line 2212
25 => {
vm_block = if ((((((((((((i) == ((PROP_ID_Start as i32))) as i32)) != 0) || (((((i) == ((PROP_Case_Ignorable as i32))) as i32)) != 0)) as i32)) != 0) || (((((i) == ((PROP_ID_Continue1 as i32))) as i32)) != 0)) as i32)) != 0 { 23 } else { 24 }; continue;
}
// C line 2211
26 => {
let _ = { let assigned = (0 as i32); i = assigned; assigned };
vm_block = 22; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: unicode_gen.c:2245. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn is_emoji_hair_color(mut c: u32) -> i32 {
let mut vm_local_storage = Vec::<u64>::new();
let mut vm_block: usize = 1;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 2247
1 => {
return (((((((c) >= ((((129456 as i32)) as u32))) as i32)) != 0) && (((((c) <= ((((129459 as i32)) as u32))) as i32)) != 0)) as i32);
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: unicode_gen.c:2255. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn mark_zwj_string(mut sl: *mut REStringList, mut buf: *mut u32, mut len: i32, mut mod_type: i32, mut mod_pos: *mut i32, mut hc_pos: i32, mut mark_flag: i32) -> i32 {
let mut vm_local_storage = Vec::<u64>::new();
let mut p: *mut REString = core::mem::zeroed();
let mut i: i32 = core::mem::zeroed();
let mut n_mod: i32 = core::mem::zeroed();
let mut i0: i32 = core::mem::zeroed();
let mut i1: i32 = core::mem::zeroed();
let mut hc_count: i32 = core::mem::zeroed();
let mut j: i32 = core::mem::zeroed();
let mut vm_block: usize = 39;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 2319
1 => {
return (TRUE as i32);
}
// C line 2287
2 => {
vm_block = if ((((j) < (hc_count)) as i32)) != 0 { 25 } else { 1 }; continue;
}
// C line ?
3 => {
let _ = { let old = j; j = (j).wrapping_add(1); old };
vm_block = 2; continue;
}
// C line 2288
4 => {
vm_block = if ((((i) < (n_mod)) as i32)) != 0 { 24 } else { 3 }; continue;
}
// C line ?
5 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 4; continue;
}
// C line 2316
6 => {
let _ = { (*(p)).flags = (((*(p)).flags) | ((((1 as i32)) as u32))); (*(p)).flags };
vm_block = 5; continue;
}
// C line 2315
7 => {
vm_block = if (mark_flag) != 0 { 6 } else { 5 }; continue;
}
// C line 2314
8 => {
return (FALSE as i32);
}
// C line 2313
9 => {
vm_block = if ((!(!(p).is_null()) as i32)) != 0 { 8 } else { 7 }; continue;
}
// C line 2312
10 => {
let _ = { let assigned = re_string_find(sl, len, buf, (FALSE as i32)); p = assigned; assigned };
vm_block = 9; continue;
}
// C line 2310
11 => {
let _ = { let assigned = ((((129456 as i32)).wrapping_add(j)) as u32); *(buf).offset((hc_pos) as isize) = assigned; assigned };
vm_block = 10; continue;
}
// C line 2309
12 => {
vm_block = if ((((hc_pos) >= ((0 as i32))) as i32)) != 0 { 11 } else { 10 }; continue;
}
// C line 2306
13 => {
let _ = if ((((!(((0 as i32)) != 0) as i32)) as i64)) != 0 { tool_assert_rtn(c"mark_zwj_string".as_ptr(), c"unicode_gen.c".as_ptr(), (2306 as i32), c"0".as_ptr()) } else { { let _ = (0 as i32); } };
vm_block = 12; continue;
}
// C line 2304
14 => {
vm_block = 12; continue;
}
// C line 2303
15 => {
let _ = { let assigned = ((((127995 as i32)).wrapping_add(i1)) as u32); *(buf).offset((*(mod_pos).offset(((1 as i32)) as isize)) as isize) = assigned; assigned };
vm_block = 14; continue;
}
// C line 2302
16 => {
let _ = { let assigned = ((((127995 as i32)).wrapping_add(i0)) as u32); *(buf).offset((*(mod_pos).offset(((0 as i32)) as isize)) as isize) = assigned; assigned };
vm_block = 15; continue;
}
// C line 2301
17 => {
let _ = { let old = i0; i0 = (i0).wrapping_add(1); old };
vm_block = 16; continue;
}
// C line 2300
18 => {
vm_block = if ((((((((mod_type) == ((3 as i32))) as i32)) != 0) && (((((i0) >= (i1)) as i32)) != 0)) as i32)) != 0 { 17 } else { 16 }; continue;
}
// C line 2298
19 => {
let _ = { let assigned = ((i) % ((5 as i32))); i1 = assigned; assigned };
vm_block = 18; continue;
}
// C line 2297
20 => {
let _ = { let assigned = ((i) / ((5 as i32))); i0 = assigned; assigned };
vm_block = 19; continue;
}
// C line 2294
21 => {
vm_block = 12; continue;
}
// C line 2293
22 => {
let _ = { let assigned = ((((127995 as i32)).wrapping_add(i)) as u32); *(buf).offset((*(mod_pos).offset(((0 as i32)) as isize)) as isize) = assigned; assigned };
vm_block = 21; continue;
}
// C line 2291
23 => {
vm_block = 12; continue;
}
// C line 2289
24 => {
vm_block = match mod_type { x if x == (3 as i32) => 20, x if x == (2 as i32) => 20, x if x == (1 as i32) => 22, x if x == (0 as i32) => 23, _ => 13, }; continue;
}
// C line 2288
25 => {
let _ = { let assigned = (0 as i32); i = assigned; assigned };
vm_block = 4; continue;
}
// C line 2287
26 => {
let _ = { let assigned = (0 as i32); j = assigned; assigned };
vm_block = 2; continue;
}
// C line 2283
27 => {
let _ = { let assigned = (4 as i32); hc_count = assigned; assigned };
vm_block = 26; continue;
}
// C line 2285
28 => {
let _ = { let assigned = (1 as i32); hc_count = assigned; assigned };
vm_block = 26; continue;
}
// C line 2282
29 => {
vm_block = if ((((hc_pos) >= ((0 as i32))) as i32)) != 0 { 27 } else { 28 }; continue;
}
// C line 2280
30 => {
let _ = if ((((!(((0 as i32)) != 0) as i32)) as i64)) != 0 { tool_assert_rtn(c"mark_zwj_string".as_ptr(), c"unicode_gen.c".as_ptr(), (2280 as i32), c"0".as_ptr()) } else { { let _ = (0 as i32); } };
vm_block = 29; continue;
}
// C line 2278
31 => {
vm_block = 29; continue;
}
// C line 2277
32 => {
let _ = { let assigned = (20 as i32); n_mod = assigned; assigned };
vm_block = 31; continue;
}
// C line 2275
33 => {
vm_block = 29; continue;
}
// C line 2274
34 => {
let _ = { let assigned = (25 as i32); n_mod = assigned; assigned };
vm_block = 33; continue;
}
// C line 2272
35 => {
vm_block = 29; continue;
}
// C line 2271
36 => {
let _ = { let assigned = (5 as i32); n_mod = assigned; assigned };
vm_block = 35; continue;
}
// C line 2269
37 => {
vm_block = 29; continue;
}
// C line 2268
38 => {
let _ = { let assigned = (1 as i32); n_mod = assigned; assigned };
vm_block = 37; continue;
}
// C line 2266
39 => {
vm_block = match mod_type { x if x == (3 as i32) => 32, x if x == (2 as i32) => 34, x if x == (1 as i32) => 36, x if x == (0 as i32) => 38, _ => 30, }; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: unicode_gen.c:2322. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn zwj_encode_string(mut dbuf: *mut DynBuf, mut buf: *const u32, mut len: i32, mut mod_type: i32, mut mod_pos: *mut i32, mut hc_pos: i32) -> () {
let mut vm_local_storage = Vec::<u64>::new();
let mut i: i32 = core::mem::zeroed();
let mut j: i32 = core::mem::zeroed();
let mut c: i32 = core::mem::zeroed();
let mut code: i32 = core::mem::zeroed();
let mut buf1: [u32; 16] = core::mem::zeroed();
let mut vm_block: usize = 25;
loop { match vm_block {
// C line ?
0 => {
return ();
}
// C line 2357
1 => {
vm_block = if ((((i) < (j)) as i32)) != 0 { 4 } else { 0 }; continue;
}
// C line ?
2 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 1; continue;
}
// C line 2359
3 => {
let _ = dbuf_putc(dbuf, (((*((buf1).as_mut_ptr()).offset((i) as isize)).wrapping_shr(((8 as i32)) as u32)) as u8));
vm_block = 2; continue;
}
// C line 2358
4 => {
let _ = dbuf_putc(dbuf, ((*((buf1).as_mut_ptr()).offset((i) as isize)) as u8));
vm_block = 3; continue;
}
// C line 2357
5 => {
let _ = { let assigned = (0 as i32); i = assigned; assigned };
vm_block = 1; continue;
}
// C line 2356
6 => {
let _ = dbuf_putc(dbuf, ((j) as u8));
vm_block = 5; continue;
}
// C line 2330
7 => {
vm_block = if ((((i) < (len)) as i32)) != 0 { 23 } else { 6 }; continue;
}
// C line 2354
8 => {
let _ = { let assigned = ((code) as u32); *((buf1).as_mut_ptr()).offset(({ let old = j; j = (j).wrapping_add(1); old }) as isize) = assigned; assigned };
vm_block = 7; continue;
}
// C line 2352
9 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 8; continue;
}
// C line 2351
10 => {
let _ = if ((((!(((((*(buf).offset((i) as isize)) == ((((8205 as i32)) as u32))) as i32)) != 0) as i32)) as i64)) != 0 { tool_assert_rtn(c"zwj_encode_string".as_ptr(), c"unicode_gen.c".as_ptr(), (2351 as i32), c"buf[i] == 0x200d".as_ptr()) } else { { let _ = (0 as i32); } };
vm_block = 9; continue;
}
// C line 2349
11 => {
vm_block = if ((((i) < (len)) as i32)) != 0 { 10 } else { 8 }; continue;
}
// C line 2347
12 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 11; continue;
}
// C line 2346
13 => {
let _ = { code = ((code) | ((32768 as i32))); code };
vm_block = 12; continue;
}
// C line 2344
14 => {
vm_block = if ((((((((i) < (len)) as i32)) != 0) && (((((*(buf).offset((i) as isize)) == ((((65039 as i32)) as u32))) as i32)) != 0)) as i32)) != 0 { 13 } else { 11 }; continue;
}
// C line 2342
15 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 14; continue;
}
// C line 2341
16 => {
let _ = { code = ((code) | ((mod_type).wrapping_shl(((13 as i32)) as u32))); code };
vm_block = 15; continue;
}
// C line 2339
17 => {
vm_block = if ((((((((i) < (len)) as i32)) != 0) && ((is_emoji_modifier(*(buf).offset((i) as isize))) != 0)) as i32)) != 0 { 16 } else { 14 }; continue;
}
// C line 2333
18 => {
let _ = { let assigned = (c).wrapping_sub((8192 as i32)); code = assigned; assigned };
vm_block = 17; continue;
}
// C line 2335
19 => {
let _ = { let assigned = ((c).wrapping_sub((126976 as i32))).wrapping_add((4096 as i32)); code = assigned; assigned };
vm_block = 17; continue;
}
// C line 2337
20 => {
let _ = if ((((!(((0 as i32)) != 0) as i32)) as i64)) != 0 { tool_assert_rtn(c"zwj_encode_string".as_ptr(), c"unicode_gen.c".as_ptr(), (2337 as i32), c"0".as_ptr()) } else { { let _ = (0 as i32); } };
vm_block = 17; continue;
}
// C line 2334
21 => {
vm_block = if ((((((((c) >= ((126976 as i32))) as i32)) != 0) && (((((c) <= ((131071 as i32))) as i32)) != 0)) as i32)) != 0 { 19 } else { 20 }; continue;
}
// C line 2332
22 => {
vm_block = if ((((((((c) >= ((8192 as i32))) as i32)) != 0) && (((((c) <= ((12287 as i32))) as i32)) != 0)) as i32)) != 0 { 18 } else { 21 }; continue;
}
// C line 2331
23 => {
let _ = { let assigned = ((*(buf).offset(({ let old = i; i = (i).wrapping_add(1); old }) as isize)) as i32); c = assigned; assigned };
vm_block = 22; continue;
}
// C line 2330
24 => {
let _ = { let assigned = (0 as i32); i = assigned; assigned };
vm_block = 7; continue;
}
// C line 2329
25 => {
let _ = { let assigned = (0 as i32); j = assigned; assigned };
vm_block = 24; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: unicode_gen.c:2363. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn build_rgi_emoji_zwj_sequence(mut f: *mut FILE, mut sl: *mut REStringList) -> () {
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
let mut vm_block: usize = 40;
loop { match vm_block {
// C line ?
0 => {
return ();
}
// C line 2437
1 => {
let _ = dbuf_free(ptr::addr_of_mut!(dbuf));
vm_block = 0; continue;
}
// C line 2435
2 => {
let _ = dump_byte_table(f, c"unicode_rgi_emoji_zwj_sequence".as_ptr(), (dbuf).buf, (((dbuf).size) as i32));
vm_block = 1; continue;
}
// C line 2387
3 => {
vm_block = if ((((((h) as u32)) < ((*(sl)).hash_size)) as i32)) != 0 { 38 } else { 2 }; continue;
}
// C line ?
4 => {
let _ = { let old = h; h = (h).wrapping_add(1); old };
vm_block = 3; continue;
}
// C line 2388
5 => {
vm_block = if ((((p) != (core::ptr::null_mut::<REString>())) as i32)) != 0 { 37 } else { 4 }; continue;
}
// C line ?
6 => {
let _ = { let assigned = (*(p)).next; p = assigned; assigned };
vm_block = 5; continue;
}
// C line 2426
7 => {
let _ = zwj_encode_string(ptr::addr_of_mut!(dbuf), (buf).as_mut_ptr(), (((*(p)).len) as i32), mod_type, (mod_pos).as_mut_ptr(), hair_color_pos);
vm_block = 6; continue;
}
// C line 2424
8 => {
let _ = { let assigned = (((129456 as i32)) as u32); *((buf).as_mut_ptr()).offset((hair_color_pos) as isize) = assigned; assigned };
vm_block = 7; continue;
}
// C line 2423
9 => {
vm_block = if ((((hair_color_pos) >= ((0 as i32))) as i32)) != 0 { 8 } else { 7 }; continue;
}
// C line 2413
10 => {
let _ = mark_zwj_string(sl, (buf).as_mut_ptr(), (((*(p)).len) as i32), mod_type, (mod_pos).as_mut_ptr(), hair_color_pos, (TRUE as i32));
vm_block = 9; continue;
}
// C line 2417
11 => {
let _ = mark_zwj_string(sl, (buf).as_mut_ptr(), (((*(p)).len) as i32), mod_type, (mod_pos).as_mut_ptr(), hair_color_pos, (TRUE as i32));
vm_block = 9; continue;
}
// C line 2420
12 => {
vm_block = 23; continue;
}
// C line 2419
13 => {
let _ = dump_str(c"not_found".as_ptr(), ((((*(p)).buf).as_mut_ptr()) as *mut i32), (((*(p)).len) as i32));
vm_block = 12; continue;
}
// C line 2416
14 => {
vm_block = if (mark_zwj_string(sl, (buf).as_mut_ptr(), (((*(p)).len) as i32), mod_type, (mod_pos).as_mut_ptr(), hair_color_pos, (FALSE as i32))) != 0 { 11 } else { 13 }; continue;
}
// C line 2415
15 => {
let _ = { let assigned = (3 as i32); mod_type = assigned; assigned };
vm_block = 14; continue;
}
// C line 2414
16 => {
vm_block = if ((((mod_type) == ((2 as i32))) as i32)) != 0 { 15 } else { 9 }; continue;
}
// C line 2412
17 => {
vm_block = if (mark_zwj_string(sl, (buf).as_mut_ptr(), (((*(p)).len) as i32), mod_type, (mod_pos).as_mut_ptr(), hair_color_pos, (FALSE as i32))) != 0 { 10 } else { 16 }; continue;
}
// C line 2406
18 => {
let _ = { let assigned = (0 as i32); mod_type = assigned; assigned };
vm_block = 17; continue;
}
// C line 2408
19 => {
let _ = { let assigned = (1 as i32); mod_type = assigned; assigned };
vm_block = 17; continue;
}
// C line 2410
20 => {
let _ = { let assigned = (2 as i32); mod_type = assigned; assigned };
vm_block = 17; continue;
}
// C line 2407
21 => {
vm_block = if ((((mod_count) == ((1 as i32))) as i32)) != 0 { 19 } else { 20 }; continue;
}
// C line 2405
22 => {
vm_block = if ((((mod_count) == ((0 as i32))) as i32)) != 0 { 18 } else { 21 }; continue;
}
// C line ? labels: keep
23 => {
let _ = zwj_encode_string(ptr::addr_of_mut!(dbuf), (buf).as_mut_ptr(), (((*(p)).len) as i32), (0 as i32), core::ptr::null_mut::<i32>(), ((1 as i32)).wrapping_neg());
vm_block = 6; continue;
}
// C line 2403
24 => {
vm_block = if ((((((((mod_count) != ((0 as i32))) as i32)) != 0) || (((((hair_color_pos) >= ((0 as i32))) as i32)) != 0)) as i32)) != 0 { 22 } else { 23 }; continue;
}
// C line 2393
25 => {
vm_block = if ((((((j) as u32)) < ((*(p)).len)) as i32)) != 0 { 32 } else { 24 }; continue;
}
// C line ?
26 => {
let _ = { let old = j; j = (j).wrapping_add(1); old };
vm_block = 25; continue;
}
// C line 2400
27 => {
let _ = { let assigned = *(((*(p)).buf).as_mut_ptr()).offset((j) as isize); *((buf).as_mut_ptr()).offset((j) as isize) = assigned; assigned };
vm_block = 26; continue;
}
// C line 2396
28 => {
let _ = { let assigned = j; *((mod_pos).as_mut_ptr()).offset(({ let old = mod_count; mod_count = (mod_count).wrapping_add(1); old }) as isize) = assigned; assigned };
vm_block = 27; continue;
}
// C line 2395
29 => {
let _ = if ((((!(((((mod_count) < ((2 as i32))) as i32)) != 0) as i32)) as i64)) != 0 { tool_assert_rtn(c"build_rgi_emoji_zwj_sequence".as_ptr(), c"unicode_gen.c".as_ptr(), (2395 as i32), c"mod_count < 2".as_ptr()) } else { { let _ = (0 as i32); } };
vm_block = 28; continue;
}
// C line 2398
30 => {
let _ = { let assigned = j; hair_color_pos = assigned; assigned };
vm_block = 27; continue;
}
// C line 2397
31 => {
vm_block = if (is_emoji_hair_color(*(((*(p)).buf).as_mut_ptr()).offset((j) as isize))) != 0 { 30 } else { 27 }; continue;
}
// C line 2394
32 => {
vm_block = if (is_emoji_modifier(*(((*(p)).buf).as_mut_ptr()).offset((j) as isize))) != 0 { 29 } else { 31 }; continue;
}
// C line 2393
33 => {
let _ = { let assigned = (0 as i32); j = assigned; assigned };
vm_block = 25; continue;
}
// C line 2392
34 => {
let _ = { let assigned = ((1 as i32)).wrapping_neg(); hair_color_pos = assigned; assigned };
vm_block = 33; continue;
}
// C line 2391
35 => {
let _ = { let assigned = (0 as i32); mod_count = assigned; assigned };
vm_block = 34; continue;
}
// C line 2390
36 => {
vm_block = 6; continue;
}
// C line 2389
37 => {
vm_block = if ((*(p)).flags) != 0 { 36 } else { 35 }; continue;
}
// C line 2388
38 => {
let _ = { let assigned = *((*(sl)).hash_table).offset((h) as isize); p = assigned; assigned };
vm_block = 5; continue;
}
// C line 2387
39 => {
let _ = { let assigned = (0 as i32); h = assigned; assigned };
vm_block = 3; continue;
}
// C line 2384
40 => {
let _ = dbuf_init(ptr::addr_of_mut!(dbuf));
vm_block = 39; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: unicode_gen.c:2440. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn build_sequence_prop_list_table(mut f: *mut FILE) -> () {
let mut vm_local_storage = Vec::<u64>::new();
let mut i: i32 = core::mem::zeroed();
let mut vm_block: usize = 10;
loop { match vm_block {
// C line ?
0 => {
return ();
}
// C line 2454
1 => {
let _ = build_rgi_emoji_zwj_sequence(f, ptr::addr_of_mut!(rgi_emoji_zwj_sequence));
vm_block = 0; continue;
}
// C line 2452
2 => {
let _ = dump_byte_table(f, c"unicode_rgi_emoji_tag_sequence".as_ptr(), (rgi_emoji_tag_sequence).buf, (((rgi_emoji_tag_sequence).size) as i32));
vm_block = 1; continue;
}
// C line 2449
3 => {
let _ = dump_name_table(f, c"unicode_sequence_prop_name_table".as_ptr(), (unicode_sequence_prop_name).as_mut_ptr(), (SEQUENCE_PROP_COUNT as i32), core::ptr::null_mut::<*const c_char>());
vm_block = 2; continue;
}
// C line 2447
4 => {
let _ = tool_fprintf(f, c"} UnicodeSequencePropertyEnum;\n\n".as_ptr(), &[]);
vm_block = 3; continue;
}
// C line 2446
5 => {
let _ = tool_fprintf(f, c"    UNICODE_SEQUENCE_PROP_COUNT,\n".as_ptr(), &[]);
vm_block = 4; continue;
}
// C line 2444
6 => {
vm_block = if ((((i) < ((SEQUENCE_PROP_COUNT as i32))) as i32)) != 0 { 8 } else { 5 }; continue;
}
// C line ?
7 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 6; continue;
}
// C line 2445
8 => {
let _ = tool_fprintf(f, c"    UNICODE_SEQUENCE_PROP_%s,\n".as_ptr(), &[ToolPrintArg::Str(*((unicode_sequence_prop_name).as_mut_ptr()).offset((i) as isize) as *const c_char)]);
vm_block = 7; continue;
}
// C line 2444
9 => {
let _ = { let assigned = (0 as i32); i = assigned; assigned };
vm_block = 6; continue;
}
// C line 2443
10 => {
let _ = tool_fprintf(f, c"typedef enum {\n".as_ptr(), &[]);
vm_block = 9; continue;
}
_ => std::process::abort(),
} }
}

#[cfg(all(feature="unicode-use-test"))]
// Generated from the official C AST; original goto CFG retained.
// c: unicode_gen.c:2458. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn check_conv(mut res: *mut u32, mut c: u32, mut conv_type: i32) -> i32 {
let mut vm_local_storage = Vec::<u64>::new();
let mut vm_block: usize = 1;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 2460
1 => {
return tool_lre_case_conv(res, c, conv_type);
}
_ => std::process::abort(),
} }
}

#[cfg(all(feature="unicode-use-test"))]
// Generated from the official C AST; original goto CFG retained.
// c: unicode_gen.c:2463. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn check_case_conv() -> () {
let mut vm_local_storage = Vec::<u64>::new();
let mut tab: *mut CCInfo = core::mem::zeroed();
let mut res: [u32; 3] = core::mem::zeroed();
let mut l: i32 = core::mem::zeroed();
let mut error: i32 = core::mem::zeroed();
let mut ci_s: CCInfo = core::mem::zeroed();
let mut ci1: *mut CCInfo = core::mem::zeroed();
let mut ci: *mut CCInfo = core::mem::zeroed();
let mut code: i32 = core::mem::zeroed();
let mut vm_block: usize = 32;
loop { match vm_block {
// C line ?
0 => {
return ();
}
// C line 2471
1 => {
vm_block = if ((((code) <= ((1114111 as i32))) as i32)) != 0 { 29 } else { 0 }; continue;
}
// C line ?
2 => {
let _ = { let old = code; code = (code).wrapping_add(1); old };
vm_block = 1; continue;
}
// C line 2505
3 => {
let _ = tool_exit((1 as i32));
vm_block = 2; continue;
}
// C line 2504
4 => {
let _ = dump_cc_info(ci, code);
vm_block = 3; continue;
}
// C line 2503
5 => {
vm_block = if (error) != 0 { 4 } else { 2 }; continue;
}
// C line 2501
6 => {
let _ = { let old = error; error = (error).wrapping_add(1); old };
vm_block = 5; continue;
}
// C line 2500
7 => {
let _ = tool_printf(c"ERROR: F\n".as_ptr(), &[]);
vm_block = 6; continue;
}
// C line 2499
8 => {
vm_block = if ((((((((l) != ((((*(ci)).f_len) as i32))) as i32)) != 0) || ((tabcmp((((res).as_mut_ptr()) as *mut i32), ((*(ci)).f_data).as_mut_ptr(), l)) != 0)) as i32)) != 0 { 7 } else { 5 }; continue;
}
// C line 2498
9 => {
let _ = { let assigned = check_conv((res).as_mut_ptr(), ((code) as u32), (2 as i32)); l = assigned; assigned };
vm_block = 8; continue;
}
// C line 2496
10 => {
let _ = { let old = error; error = (error).wrapping_add(1); old };
vm_block = 9; continue;
}
// C line 2495
11 => {
let _ = tool_printf(c"ERROR: U\n".as_ptr(), &[]);
vm_block = 10; continue;
}
// C line 2494
12 => {
vm_block = if ((((((((l) != ((((*(ci)).l_len) as i32))) as i32)) != 0) || ((tabcmp((((res).as_mut_ptr()) as *mut i32), ((*(ci)).l_data).as_mut_ptr(), l)) != 0)) as i32)) != 0 { 11 } else { 9 }; continue;
}
// C line 2493
13 => {
let _ = { let assigned = check_conv((res).as_mut_ptr(), ((code) as u32), (1 as i32)); l = assigned; assigned };
vm_block = 12; continue;
}
// C line 2491
14 => {
let _ = { let old = error; error = (error).wrapping_add(1); old };
vm_block = 13; continue;
}
// C line 2490
15 => {
let _ = tool_printf(c"ERROR: L\n".as_ptr(), &[]);
vm_block = 14; continue;
}
// C line 2489
16 => {
vm_block = if ((((((((l) != ((((*(ci)).u_len) as i32))) as i32)) != 0) || ((tabcmp((((res).as_mut_ptr()) as *mut i32), ((*(ci)).u_data).as_mut_ptr(), l)) != 0)) as i32)) != 0 { 15 } else { 13 }; continue;
}
// C line 2488
17 => {
let _ = { let assigned = check_conv((res).as_mut_ptr(), ((code) as u32), (0 as i32)); l = assigned; assigned };
vm_block = 16; continue;
}
// C line 2487
18 => {
let _ = { let assigned = (0 as i32); error = assigned; assigned };
vm_block = 17; continue;
}
// C line 2484
19 => {
let _ = { let assigned = code; *(((*(ci)).f_data).as_mut_ptr()).offset(((0 as i32)) as isize) = assigned; assigned };
vm_block = 18; continue;
}
// C line 2483
20 => {
let _ = { let assigned = (((1 as i32)) as u8); (*(ci)).f_len = assigned; assigned };
vm_block = 19; continue;
}
// C line 2482
21 => {
vm_block = if (((((((*(ci)).f_len) as i32)) == ((0 as i32))) as i32)) != 0 { 20 } else { 18 }; continue;
}
// C line 2480
22 => {
let _ = { let assigned = code; *(((*(ci)).u_data).as_mut_ptr()).offset(((0 as i32)) as isize) = assigned; assigned };
vm_block = 21; continue;
}
// C line 2479
23 => {
let _ = { let assigned = (((1 as i32)) as u8); (*(ci)).u_len = assigned; assigned };
vm_block = 22; continue;
}
// C line 2478
24 => {
vm_block = if (((((((*(ci)).u_len) as i32)) == ((0 as i32))) as i32)) != 0 { 23 } else { 21 }; continue;
}
// C line 2476
25 => {
let _ = { let assigned = code; *(((*(ci)).l_data).as_mut_ptr()).offset(((0 as i32)) as isize) = assigned; assigned };
vm_block = 24; continue;
}
// C line 2475
26 => {
let _ = { let assigned = (((1 as i32)) as u8); (*(ci)).l_len = assigned; assigned };
vm_block = 25; continue;
}
// C line 2474
27 => {
vm_block = if (((((((*(ci)).l_len) as i32)) == ((0 as i32))) as i32)) != 0 { 26 } else { 24 }; continue;
}
// C line 2473
28 => {
let _ = { let assigned = *(ci1); *(ci) = assigned; assigned };
vm_block = 27; continue;
}
// C line 2472
29 => {
let _ = { let assigned = ptr::addr_of_mut!(*(tab).offset((code) as isize)); ci1 = assigned; assigned };
vm_block = 28; continue;
}
// C line 2471
30 => {
let _ = { let assigned = (0 as i32); code = assigned; assigned };
vm_block = 1; continue;
}
// C line 2468
31 => {
ci = ptr::addr_of_mut!(ci_s);
vm_block = 30; continue;
}
// C line 2465
32 => {
tab = unicode_db;
vm_block = 31; continue;
}
_ => std::process::abort(),
} }
}

#[cfg(all(feature="unicode-use-test", not(feature="unicode-profile")))]
// Generated from the official C AST; original goto CFG retained.
// c: unicode_gen.c:2520. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn check_flags() -> () {
let mut vm_local_storage = Vec::<u64>::new();
let mut c: i32 = core::mem::zeroed();
let mut flag_ref: i32 = core::mem::zeroed();
let mut flag: i32 = core::mem::zeroed();
let mut vm_block: usize = 23;
loop { match vm_block {
// C line ?
0 => {
return ();
}
// C line 2524
1 => {
vm_block = if ((((c) <= ((1114111 as i32))) as i32)) != 0 { 22 } else { 0 }; continue;
}
// C line ?
2 => {
let _ = { let old = c; c = (c).wrapping_add(1); old };
vm_block = 1; continue;
}
// C line 2554
3 => {
let _ = tool_exit((1 as i32));
vm_block = 2; continue;
}
// C line 2552
4 => {
let _ = tool_printf(c"ERROR: c=%05x id_cont=%d ref=%d\n".as_ptr(), &[ToolPrintArg::Int(c as u64), ToolPrintArg::Int(flag as u64), ToolPrintArg::Int(flag_ref as u64)]);
vm_block = 3; continue;
}
// C line 2551
5 => {
vm_block = if ((((flag) != (flag_ref)) as i32)) != 0 { 4 } else { 2 }; continue;
}
// C line 2550
6 => {
let _ = { let assigned = (!(((!((quickjs::libunicode::lre_is_id_continue(((c) as u32))) != 0) as i32)) != 0) as i32); flag = assigned; assigned };
vm_block = 5; continue;
}
// C line 2549
7 => {
let _ = { let assigned = get_prop(((c) as u32), (PROP_ID_Continue as i32)); flag_ref = assigned; assigned };
vm_block = 6; continue;
}
// C line 2546
8 => {
let _ = tool_exit((1 as i32));
vm_block = 7; continue;
}
// C line 2544
9 => {
let _ = tool_printf(c"ERROR: c=%05x id_start=%d ref=%d\n".as_ptr(), &[ToolPrintArg::Int(c as u64), ToolPrintArg::Int(flag as u64), ToolPrintArg::Int(flag_ref as u64)]);
vm_block = 8; continue;
}
// C line 2543
10 => {
vm_block = if ((((flag) != (flag_ref)) as i32)) != 0 { 9 } else { 7 }; continue;
}
// C line 2542
11 => {
let _ = { let assigned = (!(((!((quickjs::libunicode::lre_is_id_start(((c) as u32))) != 0) as i32)) != 0) as i32); flag = assigned; assigned };
vm_block = 10; continue;
}
// C line 2541
12 => {
let _ = { let assigned = get_prop(((c) as u32), (PROP_ID_Start as i32)); flag_ref = assigned; assigned };
vm_block = 11; continue;
}
// C line 2538
13 => {
let _ = tool_exit((1 as i32));
vm_block = 12; continue;
}
// C line 2536
14 => {
let _ = tool_printf(c"ERROR: c=%05x case_ignorable=%d ref=%d\n".as_ptr(), &[ToolPrintArg::Int(c as u64), ToolPrintArg::Int(flag as u64), ToolPrintArg::Int(flag_ref as u64)]);
vm_block = 13; continue;
}
// C line 2535
15 => {
vm_block = if ((((flag) != (flag_ref)) as i32)) != 0 { 14 } else { 12 }; continue;
}
// C line 2534
16 => {
let _ = { let assigned = (!(((!((quickjs::libunicode::lre_is_case_ignorable(((c) as u32))) != 0) as i32)) != 0) as i32); flag = assigned; assigned };
vm_block = 15; continue;
}
// C line 2533
17 => {
let _ = { let assigned = get_prop(((c) as u32), (PROP_Case_Ignorable as i32)); flag_ref = assigned; assigned };
vm_block = 16; continue;
}
// C line 2530
18 => {
let _ = tool_exit((1 as i32));
vm_block = 17; continue;
}
// C line 2528
19 => {
let _ = tool_printf(c"ERROR: c=%05x cased=%d ref=%d\n".as_ptr(), &[ToolPrintArg::Int(c as u64), ToolPrintArg::Int(flag as u64), ToolPrintArg::Int(flag_ref as u64)]);
vm_block = 18; continue;
}
// C line 2527
20 => {
vm_block = if ((((flag) != (flag_ref)) as i32)) != 0 { 19 } else { 17 }; continue;
}
// C line 2526
21 => {
let _ = { let assigned = (!(((!((quickjs::libunicode::lre_is_cased(((c) as u32))) != 0) as i32)) != 0) as i32); flag = assigned; assigned };
vm_block = 20; continue;
}
// C line 2525
22 => {
let _ = { let assigned = get_prop(((c) as u32), (PROP_Cased as i32)); flag_ref = assigned; assigned };
vm_block = 21; continue;
}
// C line 2524
23 => {
let _ = { let assigned = (0 as i32); c = assigned; assigned };
vm_block = 1; continue;
}
_ => std::process::abort(),
} }
}

#[cfg(all(feature="unicode-use-test", feature="unicode-profile"))]
// Generated from the official C AST; original goto CFG retained.
// c: unicode_gen.c:2520. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn check_flags() -> () {
let mut vm_local_storage = Vec::<u64>::new();
let mut c: i32 = core::mem::zeroed();
let mut flag_ref: i32 = core::mem::zeroed();
let mut flag: i32 = core::mem::zeroed();
let mut ti: i64 = core::mem::zeroed();
let mut count: i64 = core::mem::zeroed();
let mut vm_block: usize = 34;
loop { match vm_block {
// C line ?
0 => {
return ();
}
// C line 2569
1 => {
let _ = tool_printf(c"flags time=%0.1f ns/char\n".as_ptr(), &[ToolPrintArg::Float(((((ti) as f64)) / (((count) as f64))) as f64)]);
vm_block = 0; continue;
}
// C line 2568
2 => {
let _ = { let assigned = (get_time_ns()).wrapping_sub(ti); ti = assigned; assigned };
vm_block = 1; continue;
}
// C line 2562
3 => {
vm_block = if ((((c) <= ((65535 as i32))) as i32)) != 0 { 8 } else { 2 }; continue;
}
// C line ?
4 => {
let _ = { let old = c; c = (c).wrapping_add(1); old };
vm_block = 3; continue;
}
// C line 2566
5 => {
let _ = { let old = count; count = (count).wrapping_add(1); old };
vm_block = 4; continue;
}
// C line 2565
6 => {
let _ = if ((((!(((((flag) == (flag_ref)) as i32)) != 0) as i32)) as i64)) != 0 { tool_assert_rtn(c"check_flags".as_ptr(), c"unicode_gen.c".as_ptr(), (2565 as i32), c"flag == flag_ref".as_ptr()) } else { { let _ = (0 as i32); } };
vm_block = 5; continue;
}
// C line 2564
7 => {
let _ = { let assigned = (!(((!((quickjs::libunicode::lre_is_id_start(((c) as u32))) != 0) as i32)) != 0) as i32); flag = assigned; assigned };
vm_block = 6; continue;
}
// C line 2563
8 => {
let _ = { let assigned = get_prop(((c) as u32), (PROP_ID_Start as i32)); flag_ref = assigned; assigned };
vm_block = 7; continue;
}
// C line 2562
9 => {
let _ = { let assigned = (32 as i32); c = assigned; assigned };
vm_block = 3; continue;
}
// C line 2561
10 => {
let _ = { let assigned = (((0 as i32)) as i64); count = assigned; assigned };
vm_block = 9; continue;
}
// C line 2560
11 => {
let _ = { let assigned = get_time_ns(); ti = assigned; assigned };
vm_block = 10; continue;
}
// C line 2524
12 => {
vm_block = if ((((c) <= ((1114111 as i32))) as i32)) != 0 { 33 } else { 11 }; continue;
}
// C line ?
13 => {
let _ = { let old = c; c = (c).wrapping_add(1); old };
vm_block = 12; continue;
}
// C line 2554
14 => {
let _ = tool_exit((1 as i32));
vm_block = 13; continue;
}
// C line 2552
15 => {
let _ = tool_printf(c"ERROR: c=%05x id_cont=%d ref=%d\n".as_ptr(), &[ToolPrintArg::Int(c as u64), ToolPrintArg::Int(flag as u64), ToolPrintArg::Int(flag_ref as u64)]);
vm_block = 14; continue;
}
// C line 2551
16 => {
vm_block = if ((((flag) != (flag_ref)) as i32)) != 0 { 15 } else { 13 }; continue;
}
// C line 2550
17 => {
let _ = { let assigned = (!(((!((quickjs::libunicode::lre_is_id_continue(((c) as u32))) != 0) as i32)) != 0) as i32); flag = assigned; assigned };
vm_block = 16; continue;
}
// C line 2549
18 => {
let _ = { let assigned = get_prop(((c) as u32), (PROP_ID_Continue as i32)); flag_ref = assigned; assigned };
vm_block = 17; continue;
}
// C line 2546
19 => {
let _ = tool_exit((1 as i32));
vm_block = 18; continue;
}
// C line 2544
20 => {
let _ = tool_printf(c"ERROR: c=%05x id_start=%d ref=%d\n".as_ptr(), &[ToolPrintArg::Int(c as u64), ToolPrintArg::Int(flag as u64), ToolPrintArg::Int(flag_ref as u64)]);
vm_block = 19; continue;
}
// C line 2543
21 => {
vm_block = if ((((flag) != (flag_ref)) as i32)) != 0 { 20 } else { 18 }; continue;
}
// C line 2542
22 => {
let _ = { let assigned = (!(((!((quickjs::libunicode::lre_is_id_start(((c) as u32))) != 0) as i32)) != 0) as i32); flag = assigned; assigned };
vm_block = 21; continue;
}
// C line 2541
23 => {
let _ = { let assigned = get_prop(((c) as u32), (PROP_ID_Start as i32)); flag_ref = assigned; assigned };
vm_block = 22; continue;
}
// C line 2538
24 => {
let _ = tool_exit((1 as i32));
vm_block = 23; continue;
}
// C line 2536
25 => {
let _ = tool_printf(c"ERROR: c=%05x case_ignorable=%d ref=%d\n".as_ptr(), &[ToolPrintArg::Int(c as u64), ToolPrintArg::Int(flag as u64), ToolPrintArg::Int(flag_ref as u64)]);
vm_block = 24; continue;
}
// C line 2535
26 => {
vm_block = if ((((flag) != (flag_ref)) as i32)) != 0 { 25 } else { 23 }; continue;
}
// C line 2534
27 => {
let _ = { let assigned = (!(((!((quickjs::libunicode::lre_is_case_ignorable(((c) as u32))) != 0) as i32)) != 0) as i32); flag = assigned; assigned };
vm_block = 26; continue;
}
// C line 2533
28 => {
let _ = { let assigned = get_prop(((c) as u32), (PROP_Case_Ignorable as i32)); flag_ref = assigned; assigned };
vm_block = 27; continue;
}
// C line 2530
29 => {
let _ = tool_exit((1 as i32));
vm_block = 28; continue;
}
// C line 2528
30 => {
let _ = tool_printf(c"ERROR: c=%05x cased=%d ref=%d\n".as_ptr(), &[ToolPrintArg::Int(c as u64), ToolPrintArg::Int(flag as u64), ToolPrintArg::Int(flag_ref as u64)]);
vm_block = 29; continue;
}
// C line 2527
31 => {
vm_block = if ((((flag) != (flag_ref)) as i32)) != 0 { 30 } else { 28 }; continue;
}
// C line 2526
32 => {
let _ = { let assigned = (!(((!((quickjs::libunicode::lre_is_cased(((c) as u32))) != 0) as i32)) != 0) as i32); flag = assigned; assigned };
vm_block = 31; continue;
}
// C line 2525
33 => {
let _ = { let assigned = get_prop(((c) as u32), (PROP_Cased as i32)); flag_ref = assigned; assigned };
vm_block = 32; continue;
}
// C line 2524
34 => {
let _ = { let assigned = (0 as i32); c = assigned; assigned };
vm_block = 12; continue;
}
_ => std::process::abort(),
} }
}

#[cfg(all(not(feature="unicode-dump-table-size"), not(feature="unicode-dump-cc-table")))]
// Generated from the official C AST; original goto CFG retained.
// c: unicode_gen.c:2579. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn build_cc_table(mut f: *mut FILE) -> () {
let mut vm_local_storage = Vec::<u64>::new();
let mut i: i32 = core::mem::zeroed();
let mut cc: i32 = core::mem::zeroed();
let mut n: i32 = core::mem::zeroed();
let mut v_type: i32 = core::mem::zeroed();
let mut n1: i32 = core::mem::zeroed();
let mut block_end_pos: i32 = core::mem::zeroed();
let mut dbuf_s: DynBuf = core::mem::zeroed();
let mut dbuf: *mut DynBuf = core::mem::zeroed();
let mut dbuf1_s: DynBuf = core::mem::zeroed();
let mut dbuf1: *mut DynBuf = core::mem::zeroed();
let mut v: u32 = core::mem::zeroed();
let mut vm_block: usize = 54;
loop { match vm_block {
// C line ?
0 => {
return ();
}
// C line 2688
1 => {
let _ = dbuf_free(dbuf1);
vm_block = 0; continue;
}
// C line 2687
2 => {
let _ = dbuf_free(dbuf);
vm_block = 1; continue;
}
// C line 2677
3 => {
let _ = dump_index_table(f, c"unicode_cc_index".as_ptr(), (*(dbuf1)).buf, (((*(dbuf1)).size) as i32));
vm_block = 2; continue;
}
// C line 2676
4 => {
let _ = dump_byte_table(f, c"unicode_cc_table".as_ptr(), (*(dbuf)).buf, (((*(dbuf)).size) as i32));
vm_block = 3; continue;
}
// C line 2674
5 => {
let _ = dbuf_putc(dbuf1, (((v).wrapping_shr(((16 as i32)) as u32)) as u8));
vm_block = 4; continue;
}
// C line 2673
6 => {
let _ = dbuf_putc(dbuf1, (((v).wrapping_shr(((8 as i32)) as u32)) as u8));
vm_block = 5; continue;
}
// C line 2672
7 => {
let _ = dbuf_putc(dbuf1, ((v) as u8));
vm_block = 6; continue;
}
// C line 2671
8 => {
let _ = { let assigned = ((i) as u32); v = assigned; assigned };
vm_block = 7; continue;
}
// C line 2599
9 => {
vm_block = if ((((i) <= ((1114111 as i32))) as i32)) != 0 { 48 } else { 8 }; continue;
}
// C line 2667
10 => {
let _ = { i = (i).wrapping_add(n); i };
vm_block = 9; continue;
}
// C line 2666
11 => {
let _ = dbuf_putc(dbuf, ((cc) as u8));
vm_block = 10; continue;
}
// C line 2665
12 => {
vm_block = if ((((((((v_type) == ((0 as i32))) as i32)) != 0) || (((((v_type) == ((1 as i32))) as i32)) != 0)) as i32)) != 0 { 11 } else { 10 }; continue;
}
// C line 2649
13 => {
let _ = dbuf_putc(dbuf, ((((n1) | ((v_type).wrapping_shl(((6 as i32)) as u32)))) as u8));
vm_block = 12; continue;
}
// C line 2653
14 => {
let _ = dbuf_putc(dbuf, ((n1) as u8));
vm_block = 12; continue;
}
// C line 2652
15 => {
let _ = dbuf_putc(dbuf, ((((((n1).wrapping_shr(((8 as i32)) as u32)).wrapping_add((48 as i32))) | ((v_type).wrapping_shl(((6 as i32)) as u32)))) as u8));
vm_block = 14; continue;
}
// C line 2651
16 => {
let _ = { n1 = (n1).wrapping_sub((48 as i32)); n1 };
vm_block = 15; continue;
}
// C line 2659
17 => {
let _ = dbuf_putc(dbuf, ((n1) as u8));
vm_block = 12; continue;
}
// C line 2658
18 => {
let _ = dbuf_putc(dbuf, (((n1).wrapping_shr(((8 as i32)) as u32)) as u8));
vm_block = 17; continue;
}
// C line 2657
19 => {
let _ = dbuf_putc(dbuf, ((((((n1).wrapping_shr(((16 as i32)) as u32)).wrapping_add((56 as i32))) | ((v_type).wrapping_shl(((6 as i32)) as u32)))) as u8));
vm_block = 18; continue;
}
// C line 2656
20 => {
let _ = if ((((!(((((n1) < (((1 as i32)).wrapping_shl(((20 as i32)) as u32))) as i32)) != 0) as i32)) as i64)) != 0 { tool_assert_rtn(c"build_cc_table".as_ptr(), c"unicode_gen.c".as_ptr(), (2656 as i32), c"n1 < (1 << 20)".as_ptr()) } else { { let _ = (0 as i32); } };
vm_block = 19; continue;
}
// C line 2655
21 => {
let _ = { n1 = (n1).wrapping_sub(((48 as i32)).wrapping_add(((1 as i32)).wrapping_shl(((11 as i32)) as u32))); n1 };
vm_block = 20; continue;
}
// C line 2650
22 => {
vm_block = if ((((n1) < (((48 as i32)).wrapping_add(((1 as i32)).wrapping_shl(((11 as i32)) as u32)))) as i32)) != 0 { 16 } else { 21 }; continue;
}
// C line 2648
23 => {
vm_block = if ((((n1) < ((48 as i32))) as i32)) != 0 { 13 } else { 22 }; continue;
}
// C line 2636
24 => {
let _ = { block_end_pos = (block_end_pos).wrapping_add((32 as i32)); block_end_pos };
vm_block = 23; continue;
}
// C line 2635
25 => {
let _ = dbuf_putc(dbuf1, (((v).wrapping_shr(((16 as i32)) as u32)) as u8));
vm_block = 24; continue;
}
// C line 2634
26 => {
let _ = dbuf_putc(dbuf1, (((v).wrapping_shr(((8 as i32)) as u32)) as u8));
vm_block = 25; continue;
}
// C line 2633
27 => {
let _ = dbuf_putc(dbuf1, ((v) as u8));
vm_block = 26; continue;
}
// C line 2632
28 => {
let _ = { let assigned = ((((((i) as usize)) | ((((*(dbuf)).size).wrapping_sub(((block_end_pos) as usize))).wrapping_shl(((21 as i32)) as u32)))) as u32); v = assigned; assigned };
vm_block = 27; continue;
}
// C line 2631
29 => {
vm_block = if (((((*(dbuf)).size) >= (((block_end_pos) as usize))) as i32)) != 0 { 28 } else { 23 }; continue;
}
// C line 2628
30 => {
let _ = { let assigned = (n).wrapping_sub((1 as i32)); n1 = assigned; assigned };
vm_block = 29; continue;
}
// C line 2624
31 => {
let _ = { let assigned = (2 as i32); v_type = assigned; assigned };
vm_block = 30; continue;
}
// C line 2626
32 => {
let _ = { let assigned = (3 as i32); v_type = assigned; assigned };
vm_block = 30; continue;
}
// C line 2625
33 => {
vm_block = if ((((cc) == ((230 as i32))) as i32)) != 0 { 32 } else { 30 }; continue;
}
// C line 2623
34 => {
vm_block = if ((((cc) == ((0 as i32))) as i32)) != 0 { 31 } else { 33 }; continue;
}
// C line 2622
35 => {
vm_block = if ((((v_type) == ((0 as i32))) as i32)) != 0 { 34 } else { 30 }; continue;
}
// C line 2618
36 => {
vm_block = 8; continue;
}
// C line 2617
37 => {
vm_block = if ((((((((cc) == ((0 as i32))) as i32)) != 0) && (((((((i).wrapping_add(n)).wrapping_sub((1 as i32))) == ((1114111 as i32))) as i32)) != 0)) as i32)) != 0 { 36 } else { 35 }; continue;
}
// C line 2608
38 => {
let _ = { let assigned = (1 as i32); v_type = assigned; assigned };
vm_block = 37; continue;
}
// C line 2612
39 => {
vm_block = if (((((((((i).wrapping_add(n)) <= ((1114111 as i32))) as i32)) != 0) && ((((((((*(unicode_db).offset(((i).wrapping_add(n)) as isize)).combining_class) as i32)) == (cc)) as i32)) != 0)) as i32)) != 0 { 40 } else { 37 }; continue;
}
// C line 2614
40 => {
let _ = { let old = n; n = (n).wrapping_add(1); old };
vm_block = 39; continue;
}
// C line 2611
41 => {
let _ = { let assigned = (1 as i32); n = assigned; assigned };
vm_block = 39; continue;
}
// C line 2610
42 => {
let _ = { let assigned = (0 as i32); v_type = assigned; assigned };
vm_block = 41; continue;
}
// C line 2607
43 => {
vm_block = if ((((n) >= ((2 as i32))) as i32)) != 0 { 38 } else { 42 }; continue;
}
// C line 2604
44 => {
vm_block = if (((((((((i).wrapping_add(n)) <= ((1114111 as i32))) as i32)) != 0) && ((((((((*(unicode_db).offset(((i).wrapping_add(n)) as isize)).combining_class) as i32)) == ((cc).wrapping_add(n))) as i32)) != 0)) as i32)) != 0 { 45 } else { 43 }; continue;
}
// C line 2606
45 => {
let _ = { let old = n; n = (n).wrapping_add(1); old };
vm_block = 44; continue;
}
// C line 2603
46 => {
let _ = { let assigned = (1 as i32); n = assigned; assigned };
vm_block = 44; continue;
}
// C line 2601
47 => {
let _ = if ((((!(((((cc) <= ((255 as i32))) as i32)) != 0) as i32)) as i64)) != 0 { tool_assert_rtn(c"build_cc_table".as_ptr(), c"unicode_gen.c".as_ptr(), (2601 as i32), c"cc <= 255".as_ptr()) } else { { let _ = (0 as i32); } };
vm_block = 46; continue;
}
// C line 2600
48 => {
let _ = { let assigned = (((*(unicode_db).offset((i) as isize)).combining_class) as i32); cc = assigned; assigned };
vm_block = 47; continue;
}
// C line 2599
49 => {
let _ = { let assigned = (0 as i32); i = assigned; assigned };
vm_block = 9; continue;
}
// C line 2598
50 => {
let _ = { let assigned = (32 as i32); block_end_pos = assigned; assigned };
vm_block = 49; continue;
}
// C line 2592
51 => {
let _ = dbuf_init(dbuf1);
vm_block = 50; continue;
}
// C line 2591
52 => {
let _ = dbuf_init(dbuf);
vm_block = 51; continue;
}
// C line 2585
53 => {
dbuf1 = ptr::addr_of_mut!(dbuf1_s);
vm_block = 52; continue;
}
// C line 2584
54 => {
dbuf = ptr::addr_of_mut!(dbuf_s);
vm_block = 53; continue;
}
_ => std::process::abort(),
} }
}

#[cfg(all(not(feature="unicode-dump-table-size"), feature="unicode-dump-cc-table"))]
// Generated from the official C AST; original goto CFG retained.
// c: unicode_gen.c:2579. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn build_cc_table(mut f: *mut FILE) -> () {
let mut vm_local_storage = Vec::<u64>::new();
let mut i: i32 = core::mem::zeroed();
let mut cc: i32 = core::mem::zeroed();
let mut n: i32 = core::mem::zeroed();
let mut v_type: i32 = core::mem::zeroed();
let mut n1: i32 = core::mem::zeroed();
let mut block_end_pos: i32 = core::mem::zeroed();
let mut dbuf_s: DynBuf = core::mem::zeroed();
let mut dbuf: *mut DynBuf = core::mem::zeroed();
let mut dbuf1_s: DynBuf = core::mem::zeroed();
let mut dbuf1: *mut DynBuf = core::mem::zeroed();
let mut cw_len_tab: [i32; 3] = core::mem::zeroed();
let mut cw_start: i32 = core::mem::zeroed();
let mut cc_table_len: i32 = core::mem::zeroed();
let mut v: u32 = core::mem::zeroed();
let mut vm_block: usize = 69;
loop { match vm_block {
// C line ?
0 => {
return ();
}
// C line 2688
1 => {
let _ = dbuf_free(dbuf1);
vm_block = 0; continue;
}
// C line 2687
2 => {
let _ = dbuf_free(dbuf);
vm_block = 1; continue;
}
// C line 2685
3 => {
let _ = tool_printf(c" ]\n".as_ptr(), &[]);
vm_block = 2; continue;
}
// C line 2683
4 => {
vm_block = if ((((((i) as usize)) < ((((size_of::<[i32; 3]>() as usize)) / ((size_of::<i32>() as usize))))) as i32)) != 0 { 6 } else { 3 }; continue;
}
// C line ?
5 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 4; continue;
}
// C line 2684
6 => {
let _ = tool_printf(c" %d".as_ptr(), &[ToolPrintArg::Int(*((cw_len_tab).as_mut_ptr()).offset((i) as isize) as u64)]);
vm_block = 5; continue;
}
// C line 2683
7 => {
let _ = { let assigned = (0 as i32); i = assigned; assigned };
vm_block = 4; continue;
}
// C line 2680
8 => {
let _ = tool_printf(c"CC table: size=%d (%d entries) [".as_ptr(), &[ToolPrintArg::Int(((((*(dbuf)).size).wrapping_add((*(dbuf1)).size)) as i32) as u64), ToolPrintArg::Int(cc_table_len as u64)]);
vm_block = 7; continue;
}
// C line 2677
9 => {
let _ = dump_index_table(f, c"unicode_cc_index".as_ptr(), (*(dbuf1)).buf, (((*(dbuf1)).size) as i32));
vm_block = 8; continue;
}
// C line 2676
10 => {
let _ = dump_byte_table(f, c"unicode_cc_table".as_ptr(), (*(dbuf)).buf, (((*(dbuf)).size) as i32));
vm_block = 9; continue;
}
// C line 2674
11 => {
let _ = dbuf_putc(dbuf1, (((v).wrapping_shr(((16 as i32)) as u32)) as u8));
vm_block = 10; continue;
}
// C line 2673
12 => {
let _ = dbuf_putc(dbuf1, (((v).wrapping_shr(((8 as i32)) as u32)) as u8));
vm_block = 11; continue;
}
// C line 2672
13 => {
let _ = dbuf_putc(dbuf1, ((v) as u8));
vm_block = 12; continue;
}
// C line 2671
14 => {
let _ = { let assigned = ((i) as u32); v = assigned; assigned };
vm_block = 13; continue;
}
// C line 2599
15 => {
vm_block = if ((((i) <= ((1114111 as i32))) as i32)) != 0 { 58 } else { 14 }; continue;
}
// C line 2667
16 => {
let _ = { i = (i).wrapping_add(n); i };
vm_block = 15; continue;
}
// C line 2666
17 => {
let _ = dbuf_putc(dbuf, ((cc) as u8));
vm_block = 16; continue;
}
// C line 2665
18 => {
vm_block = if ((((((((v_type) == ((0 as i32))) as i32)) != 0) || (((((v_type) == ((1 as i32))) as i32)) != 0)) as i32)) != 0 { 17 } else { 16 }; continue;
}
// C line 2663
19 => {
let _ = { let old = cc_table_len; cc_table_len = (cc_table_len).wrapping_add(1); old };
vm_block = 18; continue;
}
// C line 2662
20 => {
let _ = { let old = *((cw_len_tab).as_mut_ptr()).offset(((((*(dbuf)).size).wrapping_sub(((cw_start) as usize))).wrapping_sub((((1 as i32)) as usize))) as isize); *((cw_len_tab).as_mut_ptr()).offset(((((*(dbuf)).size).wrapping_sub(((cw_start) as usize))).wrapping_sub((((1 as i32)) as usize))) as isize) = (*((cw_len_tab).as_mut_ptr()).offset(((((*(dbuf)).size).wrapping_sub(((cw_start) as usize))).wrapping_sub((((1 as i32)) as usize))) as isize)).wrapping_add(1); old };
vm_block = 19; continue;
}
// C line 2649
21 => {
let _ = dbuf_putc(dbuf, ((((n1) | ((v_type).wrapping_shl(((6 as i32)) as u32)))) as u8));
vm_block = 20; continue;
}
// C line 2653
22 => {
let _ = dbuf_putc(dbuf, ((n1) as u8));
vm_block = 20; continue;
}
// C line 2652
23 => {
let _ = dbuf_putc(dbuf, ((((((n1).wrapping_shr(((8 as i32)) as u32)).wrapping_add((48 as i32))) | ((v_type).wrapping_shl(((6 as i32)) as u32)))) as u8));
vm_block = 22; continue;
}
// C line 2651
24 => {
let _ = { n1 = (n1).wrapping_sub((48 as i32)); n1 };
vm_block = 23; continue;
}
// C line 2659
25 => {
let _ = dbuf_putc(dbuf, ((n1) as u8));
vm_block = 20; continue;
}
// C line 2658
26 => {
let _ = dbuf_putc(dbuf, (((n1).wrapping_shr(((8 as i32)) as u32)) as u8));
vm_block = 25; continue;
}
// C line 2657
27 => {
let _ = dbuf_putc(dbuf, ((((((n1).wrapping_shr(((16 as i32)) as u32)).wrapping_add((56 as i32))) | ((v_type).wrapping_shl(((6 as i32)) as u32)))) as u8));
vm_block = 26; continue;
}
// C line 2656
28 => {
let _ = if ((((!(((((n1) < (((1 as i32)).wrapping_shl(((20 as i32)) as u32))) as i32)) != 0) as i32)) as i64)) != 0 { tool_assert_rtn(c"build_cc_table".as_ptr(), c"unicode_gen.c".as_ptr(), (2656 as i32), c"n1 < (1 << 20)".as_ptr()) } else { { let _ = (0 as i32); } };
vm_block = 27; continue;
}
// C line 2655
29 => {
let _ = { n1 = (n1).wrapping_sub(((48 as i32)).wrapping_add(((1 as i32)).wrapping_shl(((11 as i32)) as u32))); n1 };
vm_block = 28; continue;
}
// C line 2650
30 => {
vm_block = if ((((n1) < (((48 as i32)).wrapping_add(((1 as i32)).wrapping_shl(((11 as i32)) as u32)))) as i32)) != 0 { 24 } else { 29 }; continue;
}
// C line 2648
31 => {
vm_block = if ((((n1) < ((48 as i32))) as i32)) != 0 { 21 } else { 30 }; continue;
}
// C line 2639
32 => {
let _ = { let assigned = (((*(dbuf)).size) as i32); cw_start = assigned; assigned };
vm_block = 31; continue;
}
// C line 2636
33 => {
let _ = { block_end_pos = (block_end_pos).wrapping_add((32 as i32)); block_end_pos };
vm_block = 32; continue;
}
// C line 2635
34 => {
let _ = dbuf_putc(dbuf1, (((v).wrapping_shr(((16 as i32)) as u32)) as u8));
vm_block = 33; continue;
}
// C line 2634
35 => {
let _ = dbuf_putc(dbuf1, (((v).wrapping_shr(((8 as i32)) as u32)) as u8));
vm_block = 34; continue;
}
// C line 2633
36 => {
let _ = dbuf_putc(dbuf1, ((v) as u8));
vm_block = 35; continue;
}
// C line 2632
37 => {
let _ = { let assigned = ((((((i) as usize)) | ((((*(dbuf)).size).wrapping_sub(((block_end_pos) as usize))).wrapping_shl(((21 as i32)) as u32)))) as u32); v = assigned; assigned };
vm_block = 36; continue;
}
// C line 2631
38 => {
vm_block = if (((((*(dbuf)).size) >= (((block_end_pos) as usize))) as i32)) != 0 { 37 } else { 32 }; continue;
}
// C line 2628
39 => {
let _ = { let assigned = (n).wrapping_sub((1 as i32)); n1 = assigned; assigned };
vm_block = 38; continue;
}
// C line 2624
40 => {
let _ = { let assigned = (2 as i32); v_type = assigned; assigned };
vm_block = 39; continue;
}
// C line 2626
41 => {
let _ = { let assigned = (3 as i32); v_type = assigned; assigned };
vm_block = 39; continue;
}
// C line 2625
42 => {
vm_block = if ((((cc) == ((230 as i32))) as i32)) != 0 { 41 } else { 39 }; continue;
}
// C line 2623
43 => {
vm_block = if ((((cc) == ((0 as i32))) as i32)) != 0 { 40 } else { 42 }; continue;
}
// C line 2622
44 => {
vm_block = if ((((v_type) == ((0 as i32))) as i32)) != 0 { 43 } else { 39 }; continue;
}
// C line 2620
45 => {
let _ = tool_printf(c"%05x %6d %d %d\n".as_ptr(), &[ToolPrintArg::Int(i as u64), ToolPrintArg::Int(n as u64), ToolPrintArg::Int(v_type as u64), ToolPrintArg::Int(cc as u64)]);
vm_block = 44; continue;
}
// C line 2618
46 => {
vm_block = 14; continue;
}
// C line 2617
47 => {
vm_block = if ((((((((cc) == ((0 as i32))) as i32)) != 0) && (((((((i).wrapping_add(n)).wrapping_sub((1 as i32))) == ((1114111 as i32))) as i32)) != 0)) as i32)) != 0 { 46 } else { 45 }; continue;
}
// C line 2608
48 => {
let _ = { let assigned = (1 as i32); v_type = assigned; assigned };
vm_block = 47; continue;
}
// C line 2612
49 => {
vm_block = if (((((((((i).wrapping_add(n)) <= ((1114111 as i32))) as i32)) != 0) && ((((((((*(unicode_db).offset(((i).wrapping_add(n)) as isize)).combining_class) as i32)) == (cc)) as i32)) != 0)) as i32)) != 0 { 50 } else { 47 }; continue;
}
// C line 2614
50 => {
let _ = { let old = n; n = (n).wrapping_add(1); old };
vm_block = 49; continue;
}
// C line 2611
51 => {
let _ = { let assigned = (1 as i32); n = assigned; assigned };
vm_block = 49; continue;
}
// C line 2610
52 => {
let _ = { let assigned = (0 as i32); v_type = assigned; assigned };
vm_block = 51; continue;
}
// C line 2607
53 => {
vm_block = if ((((n) >= ((2 as i32))) as i32)) != 0 { 48 } else { 52 }; continue;
}
// C line 2604
54 => {
vm_block = if (((((((((i).wrapping_add(n)) <= ((1114111 as i32))) as i32)) != 0) && ((((((((*(unicode_db).offset(((i).wrapping_add(n)) as isize)).combining_class) as i32)) == ((cc).wrapping_add(n))) as i32)) != 0)) as i32)) != 0 { 55 } else { 53 }; continue;
}
// C line 2606
55 => {
let _ = { let old = n; n = (n).wrapping_add(1); old };
vm_block = 54; continue;
}
// C line 2603
56 => {
let _ = { let assigned = (1 as i32); n = assigned; assigned };
vm_block = 54; continue;
}
// C line 2601
57 => {
let _ = if ((((!(((((cc) <= ((255 as i32))) as i32)) != 0) as i32)) as i64)) != 0 { tool_assert_rtn(c"build_cc_table".as_ptr(), c"unicode_gen.c".as_ptr(), (2601 as i32), c"cc <= 255".as_ptr()) } else { { let _ = (0 as i32); } };
vm_block = 56; continue;
}
// C line 2600
58 => {
let _ = { let assigned = (((*(unicode_db).offset((i) as isize)).combining_class) as i32); cc = assigned; assigned };
vm_block = 57; continue;
}
// C line 2599
59 => {
let _ = { let assigned = (0 as i32); i = assigned; assigned };
vm_block = 15; continue;
}
// C line 2598
60 => {
let _ = { let assigned = (32 as i32); block_end_pos = assigned; assigned };
vm_block = 59; continue;
}
// C line 2595
61 => {
vm_block = if ((((((i) as usize)) < ((((size_of::<[i32; 3]>() as usize)) / ((size_of::<i32>() as usize))))) as i32)) != 0 { 63 } else { 60 }; continue;
}
// C line ?
62 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 61; continue;
}
// C line 2596
63 => {
let _ = { let assigned = (0 as i32); *((cw_len_tab).as_mut_ptr()).offset((i) as isize) = assigned; assigned };
vm_block = 62; continue;
}
// C line 2595
64 => {
let _ = { let assigned = (0 as i32); i = assigned; assigned };
vm_block = 61; continue;
}
// C line 2594
65 => {
let _ = { let assigned = (0 as i32); cc_table_len = assigned; assigned };
vm_block = 64; continue;
}
// C line 2592
66 => {
let _ = dbuf_init(dbuf1);
vm_block = 65; continue;
}
// C line 2591
67 => {
let _ = dbuf_init(dbuf);
vm_block = 66; continue;
}
// C line 2585
68 => {
dbuf1 = ptr::addr_of_mut!(dbuf1_s);
vm_block = 67; continue;
}
// C line 2584
69 => {
dbuf = ptr::addr_of_mut!(dbuf_s);
vm_block = 68; continue;
}
_ => std::process::abort(),
} }
}

#[cfg(all(feature="unicode-dump-table-size", not(feature="unicode-dump-cc-table")))]
// Generated from the official C AST; original goto CFG retained.
// c: unicode_gen.c:2579. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn build_cc_table(mut f: *mut FILE) -> () {
let mut vm_local_storage = Vec::<u64>::new();
let mut i: i32 = core::mem::zeroed();
let mut cc: i32 = core::mem::zeroed();
let mut n: i32 = core::mem::zeroed();
let mut v_type: i32 = core::mem::zeroed();
let mut n1: i32 = core::mem::zeroed();
let mut block_end_pos: i32 = core::mem::zeroed();
let mut dbuf_s: DynBuf = core::mem::zeroed();
let mut dbuf: *mut DynBuf = core::mem::zeroed();
let mut dbuf1_s: DynBuf = core::mem::zeroed();
let mut dbuf1: *mut DynBuf = core::mem::zeroed();
let mut cw_len_tab: [i32; 3] = core::mem::zeroed();
let mut cw_start: i32 = core::mem::zeroed();
let mut cc_table_len: i32 = core::mem::zeroed();
let mut v: u32 = core::mem::zeroed();
let mut vm_block: usize = 68;
loop { match vm_block {
// C line ?
0 => {
return ();
}
// C line 2688
1 => {
let _ = dbuf_free(dbuf1);
vm_block = 0; continue;
}
// C line 2687
2 => {
let _ = dbuf_free(dbuf);
vm_block = 1; continue;
}
// C line 2685
3 => {
let _ = tool_printf(c" ]\n".as_ptr(), &[]);
vm_block = 2; continue;
}
// C line 2683
4 => {
vm_block = if ((((((i) as usize)) < ((((size_of::<[i32; 3]>() as usize)) / ((size_of::<i32>() as usize))))) as i32)) != 0 { 6 } else { 3 }; continue;
}
// C line ?
5 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 4; continue;
}
// C line 2684
6 => {
let _ = tool_printf(c" %d".as_ptr(), &[ToolPrintArg::Int(*((cw_len_tab).as_mut_ptr()).offset((i) as isize) as u64)]);
vm_block = 5; continue;
}
// C line 2683
7 => {
let _ = { let assigned = (0 as i32); i = assigned; assigned };
vm_block = 4; continue;
}
// C line 2680
8 => {
let _ = tool_printf(c"CC table: size=%d (%d entries) [".as_ptr(), &[ToolPrintArg::Int(((((*(dbuf)).size).wrapping_add((*(dbuf1)).size)) as i32) as u64), ToolPrintArg::Int(cc_table_len as u64)]);
vm_block = 7; continue;
}
// C line 2677
9 => {
let _ = dump_index_table(f, c"unicode_cc_index".as_ptr(), (*(dbuf1)).buf, (((*(dbuf1)).size) as i32));
vm_block = 8; continue;
}
// C line 2676
10 => {
let _ = dump_byte_table(f, c"unicode_cc_table".as_ptr(), (*(dbuf)).buf, (((*(dbuf)).size) as i32));
vm_block = 9; continue;
}
// C line 2674
11 => {
let _ = dbuf_putc(dbuf1, (((v).wrapping_shr(((16 as i32)) as u32)) as u8));
vm_block = 10; continue;
}
// C line 2673
12 => {
let _ = dbuf_putc(dbuf1, (((v).wrapping_shr(((8 as i32)) as u32)) as u8));
vm_block = 11; continue;
}
// C line 2672
13 => {
let _ = dbuf_putc(dbuf1, ((v) as u8));
vm_block = 12; continue;
}
// C line 2671
14 => {
let _ = { let assigned = ((i) as u32); v = assigned; assigned };
vm_block = 13; continue;
}
// C line 2599
15 => {
vm_block = if ((((i) <= ((1114111 as i32))) as i32)) != 0 { 57 } else { 14 }; continue;
}
// C line 2667
16 => {
let _ = { i = (i).wrapping_add(n); i };
vm_block = 15; continue;
}
// C line 2666
17 => {
let _ = dbuf_putc(dbuf, ((cc) as u8));
vm_block = 16; continue;
}
// C line 2665
18 => {
vm_block = if ((((((((v_type) == ((0 as i32))) as i32)) != 0) || (((((v_type) == ((1 as i32))) as i32)) != 0)) as i32)) != 0 { 17 } else { 16 }; continue;
}
// C line 2663
19 => {
let _ = { let old = cc_table_len; cc_table_len = (cc_table_len).wrapping_add(1); old };
vm_block = 18; continue;
}
// C line 2662
20 => {
let _ = { let old = *((cw_len_tab).as_mut_ptr()).offset(((((*(dbuf)).size).wrapping_sub(((cw_start) as usize))).wrapping_sub((((1 as i32)) as usize))) as isize); *((cw_len_tab).as_mut_ptr()).offset(((((*(dbuf)).size).wrapping_sub(((cw_start) as usize))).wrapping_sub((((1 as i32)) as usize))) as isize) = (*((cw_len_tab).as_mut_ptr()).offset(((((*(dbuf)).size).wrapping_sub(((cw_start) as usize))).wrapping_sub((((1 as i32)) as usize))) as isize)).wrapping_add(1); old };
vm_block = 19; continue;
}
// C line 2649
21 => {
let _ = dbuf_putc(dbuf, ((((n1) | ((v_type).wrapping_shl(((6 as i32)) as u32)))) as u8));
vm_block = 20; continue;
}
// C line 2653
22 => {
let _ = dbuf_putc(dbuf, ((n1) as u8));
vm_block = 20; continue;
}
// C line 2652
23 => {
let _ = dbuf_putc(dbuf, ((((((n1).wrapping_shr(((8 as i32)) as u32)).wrapping_add((48 as i32))) | ((v_type).wrapping_shl(((6 as i32)) as u32)))) as u8));
vm_block = 22; continue;
}
// C line 2651
24 => {
let _ = { n1 = (n1).wrapping_sub((48 as i32)); n1 };
vm_block = 23; continue;
}
// C line 2659
25 => {
let _ = dbuf_putc(dbuf, ((n1) as u8));
vm_block = 20; continue;
}
// C line 2658
26 => {
let _ = dbuf_putc(dbuf, (((n1).wrapping_shr(((8 as i32)) as u32)) as u8));
vm_block = 25; continue;
}
// C line 2657
27 => {
let _ = dbuf_putc(dbuf, ((((((n1).wrapping_shr(((16 as i32)) as u32)).wrapping_add((56 as i32))) | ((v_type).wrapping_shl(((6 as i32)) as u32)))) as u8));
vm_block = 26; continue;
}
// C line 2656
28 => {
let _ = if ((((!(((((n1) < (((1 as i32)).wrapping_shl(((20 as i32)) as u32))) as i32)) != 0) as i32)) as i64)) != 0 { tool_assert_rtn(c"build_cc_table".as_ptr(), c"unicode_gen.c".as_ptr(), (2656 as i32), c"n1 < (1 << 20)".as_ptr()) } else { { let _ = (0 as i32); } };
vm_block = 27; continue;
}
// C line 2655
29 => {
let _ = { n1 = (n1).wrapping_sub(((48 as i32)).wrapping_add(((1 as i32)).wrapping_shl(((11 as i32)) as u32))); n1 };
vm_block = 28; continue;
}
// C line 2650
30 => {
vm_block = if ((((n1) < (((48 as i32)).wrapping_add(((1 as i32)).wrapping_shl(((11 as i32)) as u32)))) as i32)) != 0 { 24 } else { 29 }; continue;
}
// C line 2648
31 => {
vm_block = if ((((n1) < ((48 as i32))) as i32)) != 0 { 21 } else { 30 }; continue;
}
// C line 2639
32 => {
let _ = { let assigned = (((*(dbuf)).size) as i32); cw_start = assigned; assigned };
vm_block = 31; continue;
}
// C line 2636
33 => {
let _ = { block_end_pos = (block_end_pos).wrapping_add((32 as i32)); block_end_pos };
vm_block = 32; continue;
}
// C line 2635
34 => {
let _ = dbuf_putc(dbuf1, (((v).wrapping_shr(((16 as i32)) as u32)) as u8));
vm_block = 33; continue;
}
// C line 2634
35 => {
let _ = dbuf_putc(dbuf1, (((v).wrapping_shr(((8 as i32)) as u32)) as u8));
vm_block = 34; continue;
}
// C line 2633
36 => {
let _ = dbuf_putc(dbuf1, ((v) as u8));
vm_block = 35; continue;
}
// C line 2632
37 => {
let _ = { let assigned = ((((((i) as usize)) | ((((*(dbuf)).size).wrapping_sub(((block_end_pos) as usize))).wrapping_shl(((21 as i32)) as u32)))) as u32); v = assigned; assigned };
vm_block = 36; continue;
}
// C line 2631
38 => {
vm_block = if (((((*(dbuf)).size) >= (((block_end_pos) as usize))) as i32)) != 0 { 37 } else { 32 }; continue;
}
// C line 2628
39 => {
let _ = { let assigned = (n).wrapping_sub((1 as i32)); n1 = assigned; assigned };
vm_block = 38; continue;
}
// C line 2624
40 => {
let _ = { let assigned = (2 as i32); v_type = assigned; assigned };
vm_block = 39; continue;
}
// C line 2626
41 => {
let _ = { let assigned = (3 as i32); v_type = assigned; assigned };
vm_block = 39; continue;
}
// C line 2625
42 => {
vm_block = if ((((cc) == ((230 as i32))) as i32)) != 0 { 41 } else { 39 }; continue;
}
// C line 2623
43 => {
vm_block = if ((((cc) == ((0 as i32))) as i32)) != 0 { 40 } else { 42 }; continue;
}
// C line 2622
44 => {
vm_block = if ((((v_type) == ((0 as i32))) as i32)) != 0 { 43 } else { 39 }; continue;
}
// C line 2618
45 => {
vm_block = 14; continue;
}
// C line 2617
46 => {
vm_block = if ((((((((cc) == ((0 as i32))) as i32)) != 0) && (((((((i).wrapping_add(n)).wrapping_sub((1 as i32))) == ((1114111 as i32))) as i32)) != 0)) as i32)) != 0 { 45 } else { 44 }; continue;
}
// C line 2608
47 => {
let _ = { let assigned = (1 as i32); v_type = assigned; assigned };
vm_block = 46; continue;
}
// C line 2612
48 => {
vm_block = if (((((((((i).wrapping_add(n)) <= ((1114111 as i32))) as i32)) != 0) && ((((((((*(unicode_db).offset(((i).wrapping_add(n)) as isize)).combining_class) as i32)) == (cc)) as i32)) != 0)) as i32)) != 0 { 49 } else { 46 }; continue;
}
// C line 2614
49 => {
let _ = { let old = n; n = (n).wrapping_add(1); old };
vm_block = 48; continue;
}
// C line 2611
50 => {
let _ = { let assigned = (1 as i32); n = assigned; assigned };
vm_block = 48; continue;
}
// C line 2610
51 => {
let _ = { let assigned = (0 as i32); v_type = assigned; assigned };
vm_block = 50; continue;
}
// C line 2607
52 => {
vm_block = if ((((n) >= ((2 as i32))) as i32)) != 0 { 47 } else { 51 }; continue;
}
// C line 2604
53 => {
vm_block = if (((((((((i).wrapping_add(n)) <= ((1114111 as i32))) as i32)) != 0) && ((((((((*(unicode_db).offset(((i).wrapping_add(n)) as isize)).combining_class) as i32)) == ((cc).wrapping_add(n))) as i32)) != 0)) as i32)) != 0 { 54 } else { 52 }; continue;
}
// C line 2606
54 => {
let _ = { let old = n; n = (n).wrapping_add(1); old };
vm_block = 53; continue;
}
// C line 2603
55 => {
let _ = { let assigned = (1 as i32); n = assigned; assigned };
vm_block = 53; continue;
}
// C line 2601
56 => {
let _ = if ((((!(((((cc) <= ((255 as i32))) as i32)) != 0) as i32)) as i64)) != 0 { tool_assert_rtn(c"build_cc_table".as_ptr(), c"unicode_gen.c".as_ptr(), (2601 as i32), c"cc <= 255".as_ptr()) } else { { let _ = (0 as i32); } };
vm_block = 55; continue;
}
// C line 2600
57 => {
let _ = { let assigned = (((*(unicode_db).offset((i) as isize)).combining_class) as i32); cc = assigned; assigned };
vm_block = 56; continue;
}
// C line 2599
58 => {
let _ = { let assigned = (0 as i32); i = assigned; assigned };
vm_block = 15; continue;
}
// C line 2598
59 => {
let _ = { let assigned = (32 as i32); block_end_pos = assigned; assigned };
vm_block = 58; continue;
}
// C line 2595
60 => {
vm_block = if ((((((i) as usize)) < ((((size_of::<[i32; 3]>() as usize)) / ((size_of::<i32>() as usize))))) as i32)) != 0 { 62 } else { 59 }; continue;
}
// C line ?
61 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 60; continue;
}
// C line 2596
62 => {
let _ = { let assigned = (0 as i32); *((cw_len_tab).as_mut_ptr()).offset((i) as isize) = assigned; assigned };
vm_block = 61; continue;
}
// C line 2595
63 => {
let _ = { let assigned = (0 as i32); i = assigned; assigned };
vm_block = 60; continue;
}
// C line 2594
64 => {
let _ = { let assigned = (0 as i32); cc_table_len = assigned; assigned };
vm_block = 63; continue;
}
// C line 2592
65 => {
let _ = dbuf_init(dbuf1);
vm_block = 64; continue;
}
// C line 2591
66 => {
let _ = dbuf_init(dbuf);
vm_block = 65; continue;
}
// C line 2585
67 => {
dbuf1 = ptr::addr_of_mut!(dbuf1_s);
vm_block = 66; continue;
}
// C line 2584
68 => {
dbuf = ptr::addr_of_mut!(dbuf_s);
vm_block = 67; continue;
}
_ => std::process::abort(),
} }
}

#[cfg(all(feature="unicode-dump-table-size", feature="unicode-dump-cc-table"))]
// Generated from the official C AST; original goto CFG retained.
// c: unicode_gen.c:2579. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn build_cc_table(mut f: *mut FILE) -> () {
let mut vm_local_storage = Vec::<u64>::new();
let mut i: i32 = core::mem::zeroed();
let mut cc: i32 = core::mem::zeroed();
let mut n: i32 = core::mem::zeroed();
let mut v_type: i32 = core::mem::zeroed();
let mut n1: i32 = core::mem::zeroed();
let mut block_end_pos: i32 = core::mem::zeroed();
let mut dbuf_s: DynBuf = core::mem::zeroed();
let mut dbuf: *mut DynBuf = core::mem::zeroed();
let mut dbuf1_s: DynBuf = core::mem::zeroed();
let mut dbuf1: *mut DynBuf = core::mem::zeroed();
let mut cw_len_tab: [i32; 3] = core::mem::zeroed();
let mut cw_start: i32 = core::mem::zeroed();
let mut cc_table_len: i32 = core::mem::zeroed();
let mut v: u32 = core::mem::zeroed();
let mut vm_block: usize = 69;
loop { match vm_block {
// C line ?
0 => {
return ();
}
// C line 2688
1 => {
let _ = dbuf_free(dbuf1);
vm_block = 0; continue;
}
// C line 2687
2 => {
let _ = dbuf_free(dbuf);
vm_block = 1; continue;
}
// C line 2685
3 => {
let _ = tool_printf(c" ]\n".as_ptr(), &[]);
vm_block = 2; continue;
}
// C line 2683
4 => {
vm_block = if ((((((i) as usize)) < ((((size_of::<[i32; 3]>() as usize)) / ((size_of::<i32>() as usize))))) as i32)) != 0 { 6 } else { 3 }; continue;
}
// C line ?
5 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 4; continue;
}
// C line 2684
6 => {
let _ = tool_printf(c" %d".as_ptr(), &[ToolPrintArg::Int(*((cw_len_tab).as_mut_ptr()).offset((i) as isize) as u64)]);
vm_block = 5; continue;
}
// C line 2683
7 => {
let _ = { let assigned = (0 as i32); i = assigned; assigned };
vm_block = 4; continue;
}
// C line 2680
8 => {
let _ = tool_printf(c"CC table: size=%d (%d entries) [".as_ptr(), &[ToolPrintArg::Int(((((*(dbuf)).size).wrapping_add((*(dbuf1)).size)) as i32) as u64), ToolPrintArg::Int(cc_table_len as u64)]);
vm_block = 7; continue;
}
// C line 2677
9 => {
let _ = dump_index_table(f, c"unicode_cc_index".as_ptr(), (*(dbuf1)).buf, (((*(dbuf1)).size) as i32));
vm_block = 8; continue;
}
// C line 2676
10 => {
let _ = dump_byte_table(f, c"unicode_cc_table".as_ptr(), (*(dbuf)).buf, (((*(dbuf)).size) as i32));
vm_block = 9; continue;
}
// C line 2674
11 => {
let _ = dbuf_putc(dbuf1, (((v).wrapping_shr(((16 as i32)) as u32)) as u8));
vm_block = 10; continue;
}
// C line 2673
12 => {
let _ = dbuf_putc(dbuf1, (((v).wrapping_shr(((8 as i32)) as u32)) as u8));
vm_block = 11; continue;
}
// C line 2672
13 => {
let _ = dbuf_putc(dbuf1, ((v) as u8));
vm_block = 12; continue;
}
// C line 2671
14 => {
let _ = { let assigned = ((i) as u32); v = assigned; assigned };
vm_block = 13; continue;
}
// C line 2599
15 => {
vm_block = if ((((i) <= ((1114111 as i32))) as i32)) != 0 { 58 } else { 14 }; continue;
}
// C line 2667
16 => {
let _ = { i = (i).wrapping_add(n); i };
vm_block = 15; continue;
}
// C line 2666
17 => {
let _ = dbuf_putc(dbuf, ((cc) as u8));
vm_block = 16; continue;
}
// C line 2665
18 => {
vm_block = if ((((((((v_type) == ((0 as i32))) as i32)) != 0) || (((((v_type) == ((1 as i32))) as i32)) != 0)) as i32)) != 0 { 17 } else { 16 }; continue;
}
// C line 2663
19 => {
let _ = { let old = cc_table_len; cc_table_len = (cc_table_len).wrapping_add(1); old };
vm_block = 18; continue;
}
// C line 2662
20 => {
let _ = { let old = *((cw_len_tab).as_mut_ptr()).offset(((((*(dbuf)).size).wrapping_sub(((cw_start) as usize))).wrapping_sub((((1 as i32)) as usize))) as isize); *((cw_len_tab).as_mut_ptr()).offset(((((*(dbuf)).size).wrapping_sub(((cw_start) as usize))).wrapping_sub((((1 as i32)) as usize))) as isize) = (*((cw_len_tab).as_mut_ptr()).offset(((((*(dbuf)).size).wrapping_sub(((cw_start) as usize))).wrapping_sub((((1 as i32)) as usize))) as isize)).wrapping_add(1); old };
vm_block = 19; continue;
}
// C line 2649
21 => {
let _ = dbuf_putc(dbuf, ((((n1) | ((v_type).wrapping_shl(((6 as i32)) as u32)))) as u8));
vm_block = 20; continue;
}
// C line 2653
22 => {
let _ = dbuf_putc(dbuf, ((n1) as u8));
vm_block = 20; continue;
}
// C line 2652
23 => {
let _ = dbuf_putc(dbuf, ((((((n1).wrapping_shr(((8 as i32)) as u32)).wrapping_add((48 as i32))) | ((v_type).wrapping_shl(((6 as i32)) as u32)))) as u8));
vm_block = 22; continue;
}
// C line 2651
24 => {
let _ = { n1 = (n1).wrapping_sub((48 as i32)); n1 };
vm_block = 23; continue;
}
// C line 2659
25 => {
let _ = dbuf_putc(dbuf, ((n1) as u8));
vm_block = 20; continue;
}
// C line 2658
26 => {
let _ = dbuf_putc(dbuf, (((n1).wrapping_shr(((8 as i32)) as u32)) as u8));
vm_block = 25; continue;
}
// C line 2657
27 => {
let _ = dbuf_putc(dbuf, ((((((n1).wrapping_shr(((16 as i32)) as u32)).wrapping_add((56 as i32))) | ((v_type).wrapping_shl(((6 as i32)) as u32)))) as u8));
vm_block = 26; continue;
}
// C line 2656
28 => {
let _ = if ((((!(((((n1) < (((1 as i32)).wrapping_shl(((20 as i32)) as u32))) as i32)) != 0) as i32)) as i64)) != 0 { tool_assert_rtn(c"build_cc_table".as_ptr(), c"unicode_gen.c".as_ptr(), (2656 as i32), c"n1 < (1 << 20)".as_ptr()) } else { { let _ = (0 as i32); } };
vm_block = 27; continue;
}
// C line 2655
29 => {
let _ = { n1 = (n1).wrapping_sub(((48 as i32)).wrapping_add(((1 as i32)).wrapping_shl(((11 as i32)) as u32))); n1 };
vm_block = 28; continue;
}
// C line 2650
30 => {
vm_block = if ((((n1) < (((48 as i32)).wrapping_add(((1 as i32)).wrapping_shl(((11 as i32)) as u32)))) as i32)) != 0 { 24 } else { 29 }; continue;
}
// C line 2648
31 => {
vm_block = if ((((n1) < ((48 as i32))) as i32)) != 0 { 21 } else { 30 }; continue;
}
// C line 2639
32 => {
let _ = { let assigned = (((*(dbuf)).size) as i32); cw_start = assigned; assigned };
vm_block = 31; continue;
}
// C line 2636
33 => {
let _ = { block_end_pos = (block_end_pos).wrapping_add((32 as i32)); block_end_pos };
vm_block = 32; continue;
}
// C line 2635
34 => {
let _ = dbuf_putc(dbuf1, (((v).wrapping_shr(((16 as i32)) as u32)) as u8));
vm_block = 33; continue;
}
// C line 2634
35 => {
let _ = dbuf_putc(dbuf1, (((v).wrapping_shr(((8 as i32)) as u32)) as u8));
vm_block = 34; continue;
}
// C line 2633
36 => {
let _ = dbuf_putc(dbuf1, ((v) as u8));
vm_block = 35; continue;
}
// C line 2632
37 => {
let _ = { let assigned = ((((((i) as usize)) | ((((*(dbuf)).size).wrapping_sub(((block_end_pos) as usize))).wrapping_shl(((21 as i32)) as u32)))) as u32); v = assigned; assigned };
vm_block = 36; continue;
}
// C line 2631
38 => {
vm_block = if (((((*(dbuf)).size) >= (((block_end_pos) as usize))) as i32)) != 0 { 37 } else { 32 }; continue;
}
// C line 2628
39 => {
let _ = { let assigned = (n).wrapping_sub((1 as i32)); n1 = assigned; assigned };
vm_block = 38; continue;
}
// C line 2624
40 => {
let _ = { let assigned = (2 as i32); v_type = assigned; assigned };
vm_block = 39; continue;
}
// C line 2626
41 => {
let _ = { let assigned = (3 as i32); v_type = assigned; assigned };
vm_block = 39; continue;
}
// C line 2625
42 => {
vm_block = if ((((cc) == ((230 as i32))) as i32)) != 0 { 41 } else { 39 }; continue;
}
// C line 2623
43 => {
vm_block = if ((((cc) == ((0 as i32))) as i32)) != 0 { 40 } else { 42 }; continue;
}
// C line 2622
44 => {
vm_block = if ((((v_type) == ((0 as i32))) as i32)) != 0 { 43 } else { 39 }; continue;
}
// C line 2620
45 => {
let _ = tool_printf(c"%05x %6d %d %d\n".as_ptr(), &[ToolPrintArg::Int(i as u64), ToolPrintArg::Int(n as u64), ToolPrintArg::Int(v_type as u64), ToolPrintArg::Int(cc as u64)]);
vm_block = 44; continue;
}
// C line 2618
46 => {
vm_block = 14; continue;
}
// C line 2617
47 => {
vm_block = if ((((((((cc) == ((0 as i32))) as i32)) != 0) && (((((((i).wrapping_add(n)).wrapping_sub((1 as i32))) == ((1114111 as i32))) as i32)) != 0)) as i32)) != 0 { 46 } else { 45 }; continue;
}
// C line 2608
48 => {
let _ = { let assigned = (1 as i32); v_type = assigned; assigned };
vm_block = 47; continue;
}
// C line 2612
49 => {
vm_block = if (((((((((i).wrapping_add(n)) <= ((1114111 as i32))) as i32)) != 0) && ((((((((*(unicode_db).offset(((i).wrapping_add(n)) as isize)).combining_class) as i32)) == (cc)) as i32)) != 0)) as i32)) != 0 { 50 } else { 47 }; continue;
}
// C line 2614
50 => {
let _ = { let old = n; n = (n).wrapping_add(1); old };
vm_block = 49; continue;
}
// C line 2611
51 => {
let _ = { let assigned = (1 as i32); n = assigned; assigned };
vm_block = 49; continue;
}
// C line 2610
52 => {
let _ = { let assigned = (0 as i32); v_type = assigned; assigned };
vm_block = 51; continue;
}
// C line 2607
53 => {
vm_block = if ((((n) >= ((2 as i32))) as i32)) != 0 { 48 } else { 52 }; continue;
}
// C line 2604
54 => {
vm_block = if (((((((((i).wrapping_add(n)) <= ((1114111 as i32))) as i32)) != 0) && ((((((((*(unicode_db).offset(((i).wrapping_add(n)) as isize)).combining_class) as i32)) == ((cc).wrapping_add(n))) as i32)) != 0)) as i32)) != 0 { 55 } else { 53 }; continue;
}
// C line 2606
55 => {
let _ = { let old = n; n = (n).wrapping_add(1); old };
vm_block = 54; continue;
}
// C line 2603
56 => {
let _ = { let assigned = (1 as i32); n = assigned; assigned };
vm_block = 54; continue;
}
// C line 2601
57 => {
let _ = if ((((!(((((cc) <= ((255 as i32))) as i32)) != 0) as i32)) as i64)) != 0 { tool_assert_rtn(c"build_cc_table".as_ptr(), c"unicode_gen.c".as_ptr(), (2601 as i32), c"cc <= 255".as_ptr()) } else { { let _ = (0 as i32); } };
vm_block = 56; continue;
}
// C line 2600
58 => {
let _ = { let assigned = (((*(unicode_db).offset((i) as isize)).combining_class) as i32); cc = assigned; assigned };
vm_block = 57; continue;
}
// C line 2599
59 => {
let _ = { let assigned = (0 as i32); i = assigned; assigned };
vm_block = 15; continue;
}
// C line 2598
60 => {
let _ = { let assigned = (32 as i32); block_end_pos = assigned; assigned };
vm_block = 59; continue;
}
// C line 2595
61 => {
vm_block = if ((((((i) as usize)) < ((((size_of::<[i32; 3]>() as usize)) / ((size_of::<i32>() as usize))))) as i32)) != 0 { 63 } else { 60 }; continue;
}
// C line ?
62 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 61; continue;
}
// C line 2596
63 => {
let _ = { let assigned = (0 as i32); *((cw_len_tab).as_mut_ptr()).offset((i) as isize) = assigned; assigned };
vm_block = 62; continue;
}
// C line 2595
64 => {
let _ = { let assigned = (0 as i32); i = assigned; assigned };
vm_block = 61; continue;
}
// C line 2594
65 => {
let _ = { let assigned = (0 as i32); cc_table_len = assigned; assigned };
vm_block = 64; continue;
}
// C line 2592
66 => {
let _ = dbuf_init(dbuf1);
vm_block = 65; continue;
}
// C line 2591
67 => {
let _ = dbuf_init(dbuf);
vm_block = 66; continue;
}
// C line 2585
68 => {
dbuf1 = ptr::addr_of_mut!(dbuf1_s);
vm_block = 67; continue;
}
// C line 2584
69 => {
dbuf = ptr::addr_of_mut!(dbuf_s);
vm_block = 68; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: unicode_gen.c:2798. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn get_decomp_run_size(mut de: *const DecompEntry) -> i32 {
let mut vm_local_storage = Vec::<u64>::new();
let mut s: i32 = core::mem::zeroed();
let mut vm_block: usize = 22;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 2826
1 => {
return s;
}
// C line 2805
2 => {
let _ = { s = (s).wrapping_add((((((*(de)).len) as i32)).wrapping_mul((((*(de)).c_len) as i32))).wrapping_mul((2 as i32))); s };
vm_block = 1; continue;
}
// C line 2808
3 => {
let _ = { s = (s).wrapping_add(((((((((*(de)).len) as i32)).wrapping_mul((((*(de)).c_len) as i32))).wrapping_mul((18 as i32))).wrapping_add((7 as i32))) / ((8 as i32)))); s };
vm_block = 1; continue;
}
// C line 2810
4 => {
let _ = { s = (s).wrapping_add(((((*(de)).len) as i32)).wrapping_mul((((*(de)).c_len) as i32))); s };
vm_block = 1; continue;
}
// C line 2812
5 => {
let _ = { s = (s).wrapping_add(((((*(de)).c_len) as i32)).wrapping_mul((2 as i32))); s };
vm_block = 1; continue;
}
// C line 2814
6 => {
let _ = { s = (s).wrapping_add(((2 as i32)).wrapping_add(((((*(de)).len) as i32)).wrapping_mul((((*(de)).c_len) as i32)))); s };
vm_block = 1; continue;
}
// C line 2816
7 => {
let _ = { s = (s).wrapping_add(((((*(de)).len) as i32)).wrapping_mul((3 as i32))); s };
vm_block = 1; continue;
}
// C line 2818
8 => {
let _ = { s = (s).wrapping_add(((4 as i32)).wrapping_add(((((*(de)).len) as i32)).wrapping_mul((2 as i32)))); s };
vm_block = 1; continue;
}
// C line 2820
9 => {
let _ = { s = (s).wrapping_add((((*(de)).len) as i32)); s };
vm_block = 1; continue;
}
// C line 2822
10 => {
let _ = { s = (s).wrapping_add(((((((*(de)).len) as i32)) / ((2 as i32)))).wrapping_mul((3 as i32))); s };
vm_block = 1; continue;
}
// C line 2824
11 => {
let _ = std::process::abort();
vm_block = 1; continue;
}
// C line 2821
12 => {
vm_block = if (((((((*(de)).v_type) as i32)) <= ((DECOMP_TYPE_LS2_UL as i32))) as i32)) != 0 { 10 } else { 11 }; continue;
}
// C line 2819
13 => {
vm_block = if (((((((*(de)).v_type) as i32)) <= ((DECOMP_TYPE_S2_UL as i32))) as i32)) != 0 { 9 } else { 12 }; continue;
}
// C line 2817
14 => {
vm_block = if (((((((*(de)).v_type) as i32)) <= ((DECOMP_TYPE_PAT3 as i32))) as i32)) != 0 { 8 } else { 13 }; continue;
}
// C line 2815
15 => {
vm_block = if (((((((*(de)).v_type) as i32)) <= ((DECOMP_TYPE_LS2 as i32))) as i32)) != 0 { 7 } else { 14 }; continue;
}
// C line 2813
16 => {
vm_block = if (((((((*(de)).v_type) as i32)) <= ((DECOMP_TYPE_B18 as i32))) as i32)) != 0 { 6 } else { 15 }; continue;
}
// C line 2811
17 => {
vm_block = if (((((((*(de)).v_type) as i32)) <= ((DECOMP_TYPE_I4_2 as i32))) as i32)) != 0 { 5 } else { 16 }; continue;
}
// C line 2809
18 => {
vm_block = if (((((((*(de)).v_type) as i32)) <= ((DECOMP_TYPE_S5 as i32))) as i32)) != 0 { 4 } else { 17 }; continue;
}
// C line 2806
19 => {
vm_block = if (((((((*(de)).v_type) as i32)) <= ((DECOMP_TYPE_LL2 as i32))) as i32)) != 0 { 3 } else { 18 }; continue;
}
// C line ?
20 => {
vm_block = if (((((((*(de)).v_type) as i32)) <= ((DECOMP_TYPE_L7 as i32))) as i32)) != 0 { 2 } else { 19 }; continue;
}
// C line 2802
21 => {
vm_block = if (((((((*(de)).v_type) as i32)) <= ((DECOMP_TYPE_C1 as i32))) as i32)) != 0 { 1 } else { 20 }; continue;
}
// C line 2801
22 => {
let _ = { let assigned = (6 as i32); s = assigned; assigned };
vm_block = 21; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: unicode_gen.c:2832. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn get_short_code(mut c: i32) -> i32 {
let mut vm_local_storage = Vec::<u64>::new();
let mut i: i32 = core::mem::zeroed();
let mut vm_block: usize = 10;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 2836
1 => {
return c;
}
// C line 2838
2 => {
return ((c).wrapping_sub((768 as i32))).wrapping_add((128 as i32));
}
// C line 2844
3 => {
return ((1 as i32)).wrapping_neg();
}
// C line 2840
4 => {
vm_block = if ((((((i) as usize)) < ((((size_of::<[u16; 2]>() as usize)) / ((size_of::<u16>() as usize))))) as i32)) != 0 { 7 } else { 3 }; continue;
}
// C line ?
5 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 4; continue;
}
// C line 2842
6 => {
return ((i).wrapping_add((128 as i32))).wrapping_add((80 as i32));
}
// C line 2841
7 => {
vm_block = if ((((c) == (((*((unicode_short_table).as_ptr()).offset((i) as isize)) as i32))) as i32)) != 0 { 6 } else { 5 }; continue;
}
// C line 2840
8 => {
let _ = { let assigned = (0 as i32); i = assigned; assigned };
vm_block = 4; continue;
}
// C line 2837
9 => {
vm_block = if ((((((((c) >= ((768 as i32))) as i32)) != 0) && (((((c) < ((848 as i32))) as i32)) != 0)) as i32)) != 0 { 2 } else { 8 }; continue;
}
// C line 2835
10 => {
vm_block = if ((((c) < ((128 as i32))) as i32)) != 0 { 1 } else { 9 }; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: unicode_gen.c:2848. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn is_short(mut code: i32) -> i32 {
let mut vm_local_storage = Vec::<u64>::new();
let mut vm_block: usize = 1;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 2850
1 => {
return (((get_short_code(code)) >= ((0 as i32))) as i32);
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: unicode_gen.c:2853. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn is_short_tab(mut tab: *const i32, mut len: i32) -> i32 {
let mut vm_local_storage = Vec::<u64>::new();
let mut i: i32 = core::mem::zeroed();
let mut vm_block: usize = 6;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 2860
1 => {
return (TRUE as i32);
}
// C line 2856
2 => {
vm_block = if ((((i) < (len)) as i32)) != 0 { 5 } else { 1 }; continue;
}
// C line ?
3 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 2; continue;
}
// C line 2858
4 => {
return (FALSE as i32);
}
// C line 2857
5 => {
vm_block = if ((!((is_short(*(tab).offset((i) as isize))) != 0) as i32)) != 0 { 4 } else { 3 }; continue;
}
// C line 2856
6 => {
let _ = { let assigned = (0 as i32); i = assigned; assigned };
vm_block = 2; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: unicode_gen.c:2863. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn is_16bit(mut tab: *const i32, mut len: i32) -> i32 {
let mut vm_local_storage = Vec::<u64>::new();
let mut i: i32 = core::mem::zeroed();
let mut vm_block: usize = 6;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 2870
1 => {
return (TRUE as i32);
}
// C line 2866
2 => {
vm_block = if ((((i) < (len)) as i32)) != 0 { 5 } else { 1 }; continue;
}
// C line ?
3 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 2; continue;
}
// C line 2868
4 => {
return (FALSE as i32);
}
// C line 2867
5 => {
vm_block = if ((((*(tab).offset((i) as isize)) > ((65535 as i32))) as i32)) != 0 { 4 } else { 3 }; continue;
}
// C line 2866
6 => {
let _ = { let assigned = (0 as i32); i = assigned; assigned };
vm_block = 2; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: unicode_gen.c:2873. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn to_lower_simple(mut c: u32) -> u32 {
let mut vm_local_storage = Vec::<u64>::new();
let mut vm_block: usize = 4;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 2880
1 => {
return c;
}
// C line 2877
2 => {
let _ = { c = (c).wrapping_add((((32 as i32)) as u32)); c };
vm_block = 1; continue;
}
// C line 2879
3 => {
let _ = { let old = c; c = (c).wrapping_add(1); old };
vm_block = 1; continue;
}
// C line 2876
4 => {
vm_block = if ((((((((c) < ((((256 as i32)) as u32))) as i32)) != 0) || (((((((((c) >= ((((1040 as i32)) as u32))) as i32)) != 0) && (((((c) <= ((((1071 as i32)) as u32))) as i32)) != 0)) as i32)) != 0)) as i32)) != 0 { 2 } else { 3 }; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: unicode_gen.c:2884. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn find_decomp_run(mut tab_de: *mut DecompEntry, mut i: i32) -> () {
let mut vm_local_storage = Vec::<u64>::new();
let mut de_s: DecompEntry = core::mem::zeroed();
let mut de: *mut DecompEntry = core::mem::zeroed();
let mut ci: *mut CCInfo = core::mem::zeroed();
let mut ci1: *mut CCInfo = core::mem::zeroed();
let mut ci2: *mut CCInfo = core::mem::zeroed();
let mut l: i32 = core::mem::zeroed();
let mut j: i32 = core::mem::zeroed();
let mut n: i32 = core::mem::zeroed();
let mut len_max: i32 = core::mem::zeroed();
let mut c_min: i32 = core::mem::zeroed();
let mut c_max: i32 = core::mem::zeroed();
let mut c: i32 = core::mem::zeroed();
let mut idx1: i32 = core::mem::zeroed();
let mut idx: i32 = core::mem::zeroed();
let mut local_is_16bit: i32 = core::mem::zeroed();
let mut vm_block: usize = 176;
loop { match vm_block {
// C line ?
0 => {
return ();
}
// C line 3119
1 => {
vm_block = 19; continue;
}
// C line 3142
2 => {
let _ = { let assigned = *(de); *(tab_de).offset((i) as isize) = assigned; assigned };
vm_block = 1; continue;
}
// C line 3141
3 => {
vm_block = if (((((*(de)).cost) < ((*(tab_de).offset((i) as isize)).cost)) as i32)) != 0 { 2 } else { 1 }; continue;
}
// C line 3140
4 => {
let _ = { let assigned = (get_decomp_run_size(de)).wrapping_add((*(tab_de).offset(((i).wrapping_add(n)) as isize)).cost); (*(de)).cost = assigned; assigned };
vm_block = 3; continue;
}
// C line 3139
5 => {
let _ = { let assigned = ((l) as u8); (*(de)).c_len = assigned; assigned };
vm_block = 4; continue;
}
// C line 3138
6 => {
let _ = { let assigned = ((((DECOMP_TYPE_S2_UL as i32)).wrapping_add(local_is_16bit)) as u8); (*(de)).v_type = assigned; assigned };
vm_block = 5; continue;
}
// C line 3137
7 => {
let _ = { let assigned = ((n) as u8); (*(de)).len = assigned; assigned };
vm_block = 6; continue;
}
// C line 3136
8 => {
let _ = { let assigned = i; (*(de)).code = assigned; assigned };
vm_block = 7; continue;
}
// C line 3135
9 => {
let _ = { n = (n).wrapping_add((2 as i32)); n };
vm_block = 8; continue;
}
// C line 3134
10 => {
vm_block = 0; continue;
}
// C line 3130
11 => {
vm_block = if ((!((((((((((((((((((*(ci2)).decomp_len) == (l)) as i32)) != 0) && ((((((((*(ci2)).is_compat()) as i32)) == ((((*(ci)).is_compat()) as i32))) as i32)) != 0)) as i32)) != 0) && (((((((*((*(ci2)).decomp_data).offset(((0 as i32)) as isize)) as u32)) == (to_lower_simple(((*((*(ci1)).decomp_data).offset(((0 as i32)) as isize)) as u32)))) as i32)) != 0)) as i32)) != 0) && (((((*((*(ci2)).decomp_data).offset(((1 as i32)) as isize)) == (*((*(ci1)).decomp_data).offset(((1 as i32)) as isize))) as i32)) != 0)) as i32)) != 0) as i32)) != 0 { 10 } else { 9 }; continue;
}
// C line 3129
12 => {
let _ = { let assigned = ptr::addr_of_mut!(*(unicode_db).offset((((i).wrapping_add(n)).wrapping_add((1 as i32))) as isize)); ci2 = assigned; assigned };
vm_block = 11; continue;
}
// C line 3128
13 => {
let _ = { let assigned = (TRUE as i32); local_is_16bit = assigned; assigned };
vm_block = 12; continue;
}
// C line 3127
14 => {
vm_block = if ((((((!((local_is_16bit) != 0) as i32)) != 0) && (((!((is_short(*((*(ci1)).decomp_data).offset(((0 as i32)) as isize))) != 0) as i32)) != 0)) as i32)) != 0 { 13 } else { 12 }; continue;
}
// C line 3126
15 => {
vm_block = 0; continue;
}
// C line 3123
16 => {
vm_block = if ((!((((((((((((((*(ci1)).decomp_len) == (l)) as i32)) != 0) && ((((((((*(ci1)).is_compat()) as i32)) == ((((*(ci)).is_compat()) as i32))) as i32)) != 0)) as i32)) != 0) && ((is_short(*((*(ci1)).decomp_data).offset(((1 as i32)) as isize))) != 0)) as i32)) != 0) as i32)) != 0 { 15 } else { 14 }; continue;
}
// C line 3122
17 => {
let _ = { let assigned = ptr::addr_of_mut!(*(unicode_db).offset(((i).wrapping_add(n)) as isize)); ci1 = assigned; assigned };
vm_block = 16; continue;
}
// C line 3121
18 => {
vm_block = 0; continue;
}
// C line 3120
19 => {
vm_block = if ((!(((((((((((i).wrapping_add(n)).wrapping_add((1 as i32))) <= ((1114111 as i32))) as i32)) != 0) && ((((((n).wrapping_add((2 as i32))) <= (len_max)) as i32)) != 0)) as i32)) != 0) as i32)) != 0 { 18 } else { 17 }; continue;
}
// C line 3118
20 => {
let _ = { let assigned = (FALSE as i32); local_is_16bit = assigned; assigned };
vm_block = 1; continue;
}
// C line 3117
21 => {
let _ = { let assigned = (0 as i32); n = assigned; assigned };
vm_block = 20; continue;
}
// C line 3114
22 => {
vm_block = if ((((l) == ((2 as i32))) as i32)) != 0 { 21 } else { 0 }; continue;
}
// C line 3092
23 => {
vm_block = 36; continue;
}
// C line 3110
24 => {
let _ = { let old = n; n = (n).wrapping_add(1); old };
vm_block = 23; continue;
}
// C line 3109
25 => {
vm_block = 22; continue;
}
// C line 3104
26 => {
vm_block = if ((!((((((((((*(ci1)).decomp_len) == ((0 as i32))) as i32)) != 0) || ((((((((((((((((((*(ci1)).decomp_len) == (l)) as i32)) != 0) && ((((((((*(ci1)).is_compat()) as i32)) == ((((*(ci)).is_compat()) as i32))) as i32)) != 0)) as i32)) != 0) && (((((*((*(ci1)).decomp_data).offset(((0 as i32)) as isize)) <= ((65535 as i32))) as i32)) != 0)) as i32)) != 0) && ((is_short(*((*(ci1)).decomp_data).offset(((1 as i32)) as isize))) != 0)) as i32)) != 0)) as i32)) != 0) as i32)) != 0 { 25 } else { 24 }; continue;
}
// C line 3103
27 => {
let _ = { let assigned = ptr::addr_of_mut!(*(unicode_db).offset(((i).wrapping_add(n)) as isize)); ci1 = assigned; assigned };
vm_block = 26; continue;
}
// C line 3102
28 => {
vm_block = 22; continue;
}
// C line 3101
29 => {
vm_block = if ((!((((((((((i).wrapping_add(n)) <= ((1114111 as i32))) as i32)) != 0) && (((((n) < (len_max)) as i32)) != 0)) as i32)) != 0) as i32)) != 0 { 28 } else { 27 }; continue;
}
// C line 3099
30 => {
let _ = { let assigned = *(de); *(tab_de).offset((i) as isize) = assigned; assigned };
vm_block = 29; continue;
}
// C line 3098
31 => {
vm_block = if (((((*(de)).cost) < ((*(tab_de).offset((i) as isize)).cost)) as i32)) != 0 { 30 } else { 29 }; continue;
}
// C line 3097
32 => {
let _ = { let assigned = (get_decomp_run_size(de)).wrapping_add((*(tab_de).offset(((i).wrapping_add(n)) as isize)).cost); (*(de)).cost = assigned; assigned };
vm_block = 31; continue;
}
// C line 3096
33 => {
let _ = { let assigned = ((l) as u8); (*(de)).c_len = assigned; assigned };
vm_block = 32; continue;
}
// C line 3095
34 => {
let _ = { let assigned = (((DECOMP_TYPE_LS2 as i32)) as u8); (*(de)).v_type = assigned; assigned };
vm_block = 33; continue;
}
// C line 3094
35 => {
let _ = { let assigned = ((n) as u8); (*(de)).len = assigned; assigned };
vm_block = 34; continue;
}
// C line 3093
36 => {
let _ = { let assigned = i; (*(de)).code = assigned; assigned };
vm_block = 35; continue;
}
// C line 3091
37 => {
let _ = { let assigned = (1 as i32); n = assigned; assigned };
vm_block = 23; continue;
}
// C line 3090
38 => {
vm_block = if ((((((((l) == ((2 as i32))) as i32)) != 0) && ((is_short(*((*(ci)).decomp_data).offset(((1 as i32)) as isize))) != 0)) as i32)) != 0 { 37 } else { 22 }; continue;
}
// C line 3068
39 => {
vm_block = 52; continue;
}
// C line 3086
40 => {
let _ = { let old = n; n = (n).wrapping_add(1); old };
vm_block = 39; continue;
}
// C line 3085
41 => {
vm_block = 38; continue;
}
// C line 3080
42 => {
vm_block = if ((!((((((((((((((((((((((*(ci1)).decomp_len) == (l)) as i32)) != 0) && ((((((((*(ci1)).is_compat()) as i32)) == ((((*(ci)).is_compat()) as i32))) as i32)) != 0)) as i32)) != 0) && (((((*((*(ci1)).decomp_data).offset(((1 as i32)) as isize)) <= ((65535 as i32))) as i32)) != 0)) as i32)) != 0) && (((((*((*(ci1)).decomp_data).offset(((0 as i32)) as isize)) == (*((*(ci)).decomp_data).offset(((0 as i32)) as isize))) as i32)) != 0)) as i32)) != 0) && (((((*((*(ci1)).decomp_data).offset(((l).wrapping_sub((1 as i32))) as isize)) == (*((*(ci)).decomp_data).offset(((l).wrapping_sub((1 as i32))) as isize))) as i32)) != 0)) as i32)) != 0) as i32)) != 0 { 41 } else { 40 }; continue;
}
// C line 3079
43 => {
let _ = { let assigned = ptr::addr_of_mut!(*(unicode_db).offset(((i).wrapping_add(n)) as isize)); ci1 = assigned; assigned };
vm_block = 42; continue;
}
// C line 3078
44 => {
vm_block = 38; continue;
}
// C line 3077
45 => {
vm_block = if ((!((((((((((i).wrapping_add(n)) <= ((1114111 as i32))) as i32)) != 0) && (((((n) < (len_max)) as i32)) != 0)) as i32)) != 0) as i32)) != 0 { 44 } else { 43 }; continue;
}
// C line 3075
46 => {
let _ = { let assigned = *(de); *(tab_de).offset((i) as isize) = assigned; assigned };
vm_block = 45; continue;
}
// C line 3074
47 => {
vm_block = if (((((*(de)).cost) < ((*(tab_de).offset((i) as isize)).cost)) as i32)) != 0 { 46 } else { 45 }; continue;
}
// C line 3073
48 => {
let _ = { let assigned = (get_decomp_run_size(de)).wrapping_add((*(tab_de).offset(((i).wrapping_add(n)) as isize)).cost); (*(de)).cost = assigned; assigned };
vm_block = 47; continue;
}
// C line 3072
49 => {
let _ = { let assigned = ((l) as u8); (*(de)).c_len = assigned; assigned };
vm_block = 48; continue;
}
// C line 3071
50 => {
let _ = { let assigned = (((DECOMP_TYPE_PAT3 as i32)) as u8); (*(de)).v_type = assigned; assigned };
vm_block = 49; continue;
}
// C line 3070
51 => {
let _ = { let assigned = ((n) as u8); (*(de)).len = assigned; assigned };
vm_block = 50; continue;
}
// C line 3069
52 => {
let _ = { let assigned = i; (*(de)).code = assigned; assigned };
vm_block = 51; continue;
}
// C line 3067
53 => {
let _ = { let assigned = (1 as i32); n = assigned; assigned };
vm_block = 39; continue;
}
// C line 3066
54 => {
vm_block = if ((((l) == ((3 as i32))) as i32)) != 0 { 53 } else { 38 }; continue;
}
// C line 3033
55 => {
vm_block = if (((({ let assigned = *((*((decomp_incr_tab).as_ptr()).offset(((l).wrapping_sub((1 as i32))) as isize)).as_ptr()).offset((idx1) as isize); idx = assigned; assigned }) >= ((0 as i32))) as i32)) != 0 { 79 } else { 54 }; continue;
}
// C line ? labels: next1
56 => {
let _ = { let old = idx1; idx1 = (idx1).wrapping_add(1); old };
vm_block = 55; continue;
}
// C line 3035
57 => {
vm_block = 78; continue;
}
// C line 3060
58 => {
let _ = { let old = n; n = (n).wrapping_add(1); old };
vm_block = 57; continue;
}
// C line 3051
59 => {
vm_block = if ((((j) < (l)) as i32)) != 0 { 65 } else { 58 }; continue;
}
// C line ?
60 => {
let _ = { let old = j; j = (j).wrapping_add(1); old };
vm_block = 59; continue;
}
// C line 3054
61 => {
vm_block = 56; continue;
}
// C line 3053
62 => {
vm_block = if ((((*((*(ci1)).decomp_data).offset((j) as isize)) != ((*((*(ci)).decomp_data).offset((j) as isize)).wrapping_add(n))) as i32)) != 0 { 61 } else { 60 }; continue;
}
// C line 3057
63 => {
vm_block = 56; continue;
}
// C line 3056
64 => {
vm_block = if ((((*((*(ci1)).decomp_data).offset((j) as isize)) != (*((*(ci)).decomp_data).offset((j) as isize))) as i32)) != 0 { 63 } else { 60 }; continue;
}
// C line 3052
65 => {
vm_block = if ((((j) == (idx)) as i32)) != 0 { 62 } else { 64 }; continue;
}
// C line 3051
66 => {
let _ = { let assigned = (0 as i32); j = assigned; assigned };
vm_block = 59; continue;
}
// C line 3050
67 => {
vm_block = 56; continue;
}
// C line 3048
68 => {
vm_block = if ((!((((((((((*(ci1)).decomp_len) == (l)) as i32)) != 0) && ((((((((*(ci1)).is_compat()) as i32)) == ((((*(ci)).is_compat()) as i32))) as i32)) != 0)) as i32)) != 0) as i32)) != 0 { 67 } else { 66 }; continue;
}
// C line 3047
69 => {
let _ = { let assigned = ptr::addr_of_mut!(*(unicode_db).offset(((i).wrapping_add(n)) as isize)); ci1 = assigned; assigned };
vm_block = 68; continue;
}
// C line 3046
70 => {
vm_block = 56; continue;
}
// C line 3045
71 => {
vm_block = if ((!((((((((((i).wrapping_add(n)) <= ((1114111 as i32))) as i32)) != 0) && (((((n) < (len_max)) as i32)) != 0)) as i32)) != 0) as i32)) != 0 { 70 } else { 69 }; continue;
}
// C line 3042
72 => {
let _ = { let assigned = *(de); *(tab_de).offset((i) as isize) = assigned; assigned };
vm_block = 71; continue;
}
// C line 3041
73 => {
vm_block = if (((((*(de)).cost) < ((*(tab_de).offset((i) as isize)).cost)) as i32)) != 0 { 72 } else { 71 }; continue;
}
// C line 3040
74 => {
let _ = { let assigned = (get_decomp_run_size(de)).wrapping_add((*(tab_de).offset(((i).wrapping_add(n)) as isize)).cost); (*(de)).cost = assigned; assigned };
vm_block = 73; continue;
}
// C line 3039
75 => {
let _ = { let assigned = ((l) as u8); (*(de)).c_len = assigned; assigned };
vm_block = 74; continue;
}
// C line 3038
76 => {
let _ = { let assigned = ((((*((*((decomp_incr_tab).as_ptr()).offset(((l).wrapping_sub((1 as i32))) as isize)).as_ptr()).offset(((0 as i32)) as isize)).wrapping_add(idx1)).wrapping_sub((1 as i32))) as u8); (*(de)).v_type = assigned; assigned };
vm_block = 75; continue;
}
// C line 3037
77 => {
let _ = { let assigned = ((n) as u8); (*(de)).len = assigned; assigned };
vm_block = 76; continue;
}
// C line 3036
78 => {
let _ = { let assigned = i; (*(de)).code = assigned; assigned };
vm_block = 77; continue;
}
// C line 3034
79 => {
let _ = { let assigned = (1 as i32); n = assigned; assigned };
vm_block = 57; continue;
}
// C line 3033
80 => {
let _ = { let assigned = (1 as i32); idx1 = assigned; assigned };
vm_block = 55; continue;
}
// C line 3030
81 => {
vm_block = if ((((l) <= ((4 as i32))) as i32)) != 0 { 80 } else { 54 }; continue;
}
// C line 3006
82 => {
vm_block = 95; continue;
}
// C line 3025
83 => {
let _ = { let old = n; n = (n).wrapping_add(1); old };
vm_block = 82; continue;
}
// C line 3024
84 => {
vm_block = 81; continue;
}
// C line 3020
85 => {
vm_block = if ((!((((((((((*(ci1)).decomp_len) == ((0 as i32))) as i32)) != 0) || ((((((((((((((*(ci1)).decomp_len) == (l)) as i32)) != 0) && ((((((((*(ci1)).is_compat()) as i32)) == ((((*(ci)).is_compat()) as i32))) as i32)) != 0)) as i32)) != 0) && ((is_short_tab((*(ci1)).decomp_data, l)) != 0)) as i32)) != 0)) as i32)) != 0) as i32)) != 0 { 84 } else { 83 }; continue;
}
// C line 3018
86 => {
let _ = { let assigned = ptr::addr_of_mut!(*(unicode_db).offset(((i).wrapping_add(n)) as isize)); ci1 = assigned; assigned };
vm_block = 85; continue;
}
// C line 3017
87 => {
vm_block = 81; continue;
}
// C line 3016
88 => {
vm_block = if ((!((((((((((i).wrapping_add(n)) <= ((1114111 as i32))) as i32)) != 0) && (((((n) < (len_max)) as i32)) != 0)) as i32)) != 0) as i32)) != 0 { 87 } else { 86 }; continue;
}
// C line 3013
89 => {
let _ = { let assigned = *(de); *(tab_de).offset((i) as isize) = assigned; assigned };
vm_block = 88; continue;
}
// C line 3012
90 => {
vm_block = if (((((*(de)).cost) < ((*(tab_de).offset((i) as isize)).cost)) as i32)) != 0 { 89 } else { 88 }; continue;
}
// C line 3011
91 => {
let _ = { let assigned = (get_decomp_run_size(de)).wrapping_add((*(tab_de).offset(((i).wrapping_add(n)) as isize)).cost); (*(de)).cost = assigned; assigned };
vm_block = 90; continue;
}
// C line 3010
92 => {
let _ = { let assigned = ((l) as u8); (*(de)).c_len = assigned; assigned };
vm_block = 91; continue;
}
// C line 3009
93 => {
let _ = { let assigned = (((((DECOMP_TYPE_S1 as i32)).wrapping_add(l)).wrapping_sub((1 as i32))) as u8); (*(de)).v_type = assigned; assigned };
vm_block = 92; continue;
}
// C line 3008
94 => {
let _ = { let assigned = ((n) as u8); (*(de)).len = assigned; assigned };
vm_block = 93; continue;
}
// C line 3007
95 => {
let _ = { let assigned = i; (*(de)).code = assigned; assigned };
vm_block = 94; continue;
}
// C line 3005
96 => {
let _ = { let assigned = (1 as i32); n = assigned; assigned };
vm_block = 82; continue;
}
// C line 3004
97 => {
vm_block = if ((((((((l) <= ((5 as i32))) as i32)) != 0) && ((is_short_tab((*(ci)).decomp_data, l)) != 0)) as i32)) != 0 { 96 } else { 81 }; continue;
}
// C line 2966
98 => {
vm_block = 126; continue;
}
// C line 2999
99 => {
let _ = { let old = n; n = (n).wrapping_add(1); old };
vm_block = 98; continue;
}
// C line 2998
100 => {
vm_block = 97; continue;
}
// C line 2996
101 => {
vm_block = if ((!((((((((((*(ci1)).decomp_len) == (l)) as i32)) != 0) && ((((((((*(ci1)).is_compat()) as i32)) == ((((*(ci)).is_compat()) as i32))) as i32)) != 0)) as i32)) != 0) as i32)) != 0 { 100 } else { 99 }; continue;
}
// C line 2995
102 => {
let _ = { let assigned = ptr::addr_of_mut!(*(unicode_db).offset(((i).wrapping_add(n)) as isize)); ci1 = assigned; assigned };
vm_block = 101; continue;
}
// C line 2994
103 => {
vm_block = 97; continue;
}
// C line 2993
104 => {
vm_block = if ((!((((((((((i).wrapping_add(n)) <= ((1114111 as i32))) as i32)) != 0) && (((((n) < (len_max)) as i32)) != 0)) as i32)) != 0) as i32)) != 0 { 103 } else { 102 }; continue;
}
// C line 2991
105 => {
let _ = { let assigned = *(de); *(tab_de).offset((i) as isize) = assigned; assigned };
vm_block = 104; continue;
}
// C line 2990
106 => {
vm_block = if (((((*(de)).cost) < ((*(tab_de).offset((i) as isize)).cost)) as i32)) != 0 { 105 } else { 104 }; continue;
}
// C line 2989
107 => {
let _ = { let assigned = (get_decomp_run_size(de)).wrapping_add((*(tab_de).offset(((i).wrapping_add(n)) as isize)).cost); (*(de)).cost = assigned; assigned };
vm_block = 106; continue;
}
// C line 2988
108 => {
let _ = { let assigned = ((c_min) as u16); (*(de)).c_min = assigned; assigned };
vm_block = 107; continue;
}
// C line 2987
109 => {
let _ = { let assigned = ((l) as u8); (*(de)).c_len = assigned; assigned };
vm_block = 108; continue;
}
// C line 2984
110 => {
let _ = { let assigned = (((DECOMP_TYPE_B18 as i32)) as u8); (*(de)).v_type = assigned; assigned };
vm_block = 109; continue;
}
// C line 2986
111 => {
let _ = { let assigned = (((((DECOMP_TYPE_B1 as i32)).wrapping_add(l)).wrapping_sub((1 as i32))) as u8); (*(de)).v_type = assigned; assigned };
vm_block = 109; continue;
}
// C line 2983
112 => {
vm_block = if ((((l) == ((18 as i32))) as i32)) != 0 { 110 } else { 111 }; continue;
}
// C line 2982
113 => {
let _ = { let assigned = ((n) as u8); (*(de)).len = assigned; assigned };
vm_block = 112; continue;
}
// C line 2981
114 => {
let _ = { let assigned = i; (*(de)).code = assigned; assigned };
vm_block = 113; continue;
}
// C line 2980
115 => {
vm_block = 97; continue;
}
// C line 2979
116 => {
vm_block = if (((((c_max).wrapping_sub(c_min)) > ((254 as i32))) as i32)) != 0 { 115 } else { 114 }; continue;
}
// C line 2968
117 => {
vm_block = if ((((j) < (l)) as i32)) != 0 { 124 } else { 116 }; continue;
}
// C line ?
118 => {
let _ = { let old = j; j = (j).wrapping_add(1); old };
vm_block = 117; continue;
}
// C line 2973
119 => {
let _ = { let assigned = { let assigned = c; c_max = assigned; assigned }; c_min = assigned; assigned };
vm_block = 118; continue;
}
// C line 2976
120 => {
let _ = { let assigned = max_int(c_max, c); c_max = assigned; assigned };
vm_block = 118; continue;
}
// C line 2975
121 => {
let _ = { let assigned = min_int(c_min, c); c_min = assigned; assigned };
vm_block = 120; continue;
}
// C line ?
122 => {
vm_block = if ((((c_min) == (((1 as i32)).wrapping_neg())) as i32)) != 0 { 119 } else { 121 }; continue;
}
// C line 2970
123 => {
vm_block = if ((((c) == ((32 as i32))) as i32)) != 0 { 118 } else { 122 }; continue;
}
// C line 2969
124 => {
let _ = { let assigned = *((*(ci1)).decomp_data).offset((j) as isize); c = assigned; assigned };
vm_block = 123; continue;
}
// C line 2968
125 => {
let _ = { let assigned = (0 as i32); j = assigned; assigned };
vm_block = 117; continue;
}
// C line 2967
126 => {
let _ = { let assigned = ptr::addr_of_mut!(*(unicode_db).offset((((i).wrapping_add(n)).wrapping_sub((1 as i32))) as isize)); ci1 = assigned; assigned };
vm_block = 125; continue;
}
// C line 2965
127 => {
let _ = { let assigned = (1 as i32); n = assigned; assigned };
vm_block = 98; continue;
}
// C line 2964
128 => {
let _ = { let assigned = { let assigned = ((1 as i32)).wrapping_neg(); c_max = assigned; assigned }; c_min = assigned; assigned };
vm_block = 127; continue;
}
// C line 2962
129 => {
vm_block = if ((((((((l) <= ((8 as i32))) as i32)) != 0) || (((((l) == ((18 as i32))) as i32)) != 0)) as i32)) != 0 { 128 } else { 97 }; continue;
}
// C line 2934
130 => {
vm_block = 146; continue;
}
// C line 2958
131 => {
let _ = { let old = n; n = (n).wrapping_add(1); old };
vm_block = 130; continue;
}
// C line 2957
132 => {
vm_block = 129; continue;
}
// C line 2953
133 => {
vm_block = if ((!((((((((((*(ci1)).decomp_len) == ((0 as i32))) as i32)) != 0) || ((((((((((((((*(ci1)).decomp_len) == (l)) as i32)) != 0) && ((((((((*(ci1)).is_compat()) as i32)) == ((((*(ci)).is_compat()) as i32))) as i32)) != 0)) as i32)) != 0) && ((is_16bit((*(ci1)).decomp_data, l)) != 0)) as i32)) != 0)) as i32)) != 0) as i32)) != 0 { 132 } else { 131 }; continue;
}
// C line 2951
134 => {
let _ = { let assigned = ptr::addr_of_mut!(*(unicode_db).offset(((i).wrapping_add(n)) as isize)); ci1 = assigned; assigned };
vm_block = 133; continue;
}
// C line 2950
135 => {
vm_block = 129; continue;
}
// C line 2949
136 => {
vm_block = if ((!((((((((((i).wrapping_add(n)) <= ((1114111 as i32))) as i32)) != 0) && (((((n) < (len_max)) as i32)) != 0)) as i32)) != 0) as i32)) != 0 { 135 } else { 134 }; continue;
}
// C line 2946
137 => {
let _ = { let assigned = *(de); *(tab_de).offset((i) as isize) = assigned; assigned };
vm_block = 136; continue;
}
// C line 2945
138 => {
vm_block = if (((((*(de)).cost) < ((*(tab_de).offset((i) as isize)).cost)) as i32)) != 0 { 137 } else { 136 }; continue;
}
// C line 2944
139 => {
let _ = { let assigned = (get_decomp_run_size(de)).wrapping_add((*(tab_de).offset(((i).wrapping_add(n)) as isize)).cost); (*(de)).cost = assigned; assigned };
vm_block = 138; continue;
}
// C line 2943
140 => {
let _ = { let assigned = ((l) as u8); (*(de)).c_len = assigned; assigned };
vm_block = 139; continue;
}
// C line 2938
141 => {
let _ = { let assigned = (((DECOMP_TYPE_C1 as i32)) as u8); (*(de)).v_type = assigned; assigned };
vm_block = 140; continue;
}
// C line 2941
142 => {
let _ = { let assigned = (((((DECOMP_TYPE_L1 as i32)).wrapping_add(l)).wrapping_sub((1 as i32))) as u8); (*(de)).v_type = assigned; assigned };
vm_block = 140; continue;
}
// C line 2940
143 => {
let _ = if ((((!(((((l) <= ((8 as i32))) as i32)) != 0) as i32)) as i64)) != 0 { tool_assert_rtn(c"find_decomp_run".as_ptr(), c"unicode_gen.c".as_ptr(), (2940 as i32), c"l <= 8".as_ptr()) } else { { let _ = (0 as i32); } };
vm_block = 142; continue;
}
// C line 2937
144 => {
vm_block = if ((((((((l) == ((1 as i32))) as i32)) != 0) && (((((n) == ((1 as i32))) as i32)) != 0)) as i32)) != 0 { 141 } else { 143 }; continue;
}
// C line 2936
145 => {
let _ = { let assigned = ((n) as u8); (*(de)).len = assigned; assigned };
vm_block = 144; continue;
}
// C line 2935
146 => {
let _ = { let assigned = i; (*(de)).code = assigned; assigned };
vm_block = 145; continue;
}
// C line 2933
147 => {
let _ = { let assigned = (1 as i32); n = assigned; assigned };
vm_block = 130; continue;
}
// C line 2932
148 => {
vm_block = if ((((l) <= ((7 as i32))) as i32)) != 0 { 147 } else { 129 }; continue;
}
// C line 2929
149 => {
return;
}
// C line 2910
150 => {
vm_block = 163; continue;
}
// C line 2927
151 => {
let _ = { let old = n; n = (n).wrapping_add(1); old };
vm_block = 150; continue;
}
// C line 2926
152 => {
vm_block = 149; continue;
}
// C line 2923
153 => {
vm_block = if ((!((((((((((*(ci1)).decomp_len) == ((0 as i32))) as i32)) != 0) || ((((((((((*(ci1)).decomp_len) == (l)) as i32)) != 0) && ((((((((*(ci1)).is_compat()) as i32)) == ((((*(ci)).is_compat()) as i32))) as i32)) != 0)) as i32)) != 0)) as i32)) != 0) as i32)) != 0 { 152 } else { 151 }; continue;
}
// C line 2921
154 => {
let _ = { let assigned = ptr::addr_of_mut!(*(unicode_db).offset(((i).wrapping_add(n)) as isize)); ci1 = assigned; assigned };
vm_block = 153; continue;
}
// C line 2920
155 => {
vm_block = 149; continue;
}
// C line 2919
156 => {
vm_block = if ((!((((((((((i).wrapping_add(n)) <= ((1114111 as i32))) as i32)) != 0) && (((((n) < (len_max)) as i32)) != 0)) as i32)) != 0) as i32)) != 0 { 155 } else { 154 }; continue;
}
// C line 2917
157 => {
let _ = { let assigned = *(de); *(tab_de).offset((i) as isize) = assigned; assigned };
vm_block = 156; continue;
}
// C line 2916
158 => {
vm_block = if (((((*(de)).cost) < ((*(tab_de).offset((i) as isize)).cost)) as i32)) != 0 { 157 } else { 156 }; continue;
}
// C line 2915
159 => {
let _ = { let assigned = (get_decomp_run_size(de)).wrapping_add((*(tab_de).offset(((i).wrapping_add(n)) as isize)).cost); (*(de)).cost = assigned; assigned };
vm_block = 158; continue;
}
// C line 2914
160 => {
let _ = { let assigned = ((l) as u8); (*(de)).c_len = assigned; assigned };
vm_block = 159; continue;
}
// C line 2913
161 => {
let _ = { let assigned = (((((DECOMP_TYPE_LL1 as i32)).wrapping_add(l)).wrapping_sub((1 as i32))) as u8); (*(de)).v_type = assigned; assigned };
vm_block = 160; continue;
}
// C line 2912
162 => {
let _ = { let assigned = ((n) as u8); (*(de)).len = assigned; assigned };
vm_block = 161; continue;
}
// C line 2911
163 => {
let _ = { let assigned = i; (*(de)).code = assigned; assigned };
vm_block = 162; continue;
}
// C line 2909
164 => {
let _ = { let assigned = (1 as i32); n = assigned; assigned };
vm_block = 150; continue;
}
// C line 2907
165 => {
let _ = if ((((!(((((l) <= ((2 as i32))) as i32)) != 0) as i32)) as i64)) != 0 { tool_assert_rtn(c"find_decomp_run".as_ptr(), c"unicode_gen.c".as_ptr(), (2907 as i32), c"l <= 2".as_ptr()) } else { { let _ = (0 as i32); } };
vm_block = 164; continue;
}
// C line 2906
166 => {
vm_block = if ((!((is_16bit((*(ci)).decomp_data, l)) != 0) as i32)) != 0 { 165 } else { 148 }; continue;
}
// C line 2904
167 => {
let _ = { let assigned = (2147483647 as i32); (*(tab_de).offset((i) as isize)).cost = assigned; assigned };
vm_block = 166; continue;
}
// C line 2900
168 => {
let _ = { let assigned = (64 as i32); len_max = assigned; assigned };
vm_block = 167; continue;
}
// C line 2902
169 => {
let _ = { let assigned = (127 as i32); len_max = assigned; assigned };
vm_block = 167; continue;
}
// C line 2899
170 => {
vm_block = if ((((((((((!(((*(ci)).is_compat()) != 0) as i32)) != 0) && (((!(((*(ci)).is_excluded()) != 0) as i32)) != 0)) as i32)) != 0) && (((((l) == ((2 as i32))) as i32)) != 0)) as i32)) != 0 { 168 } else { 169 }; continue;
}
// C line 2894
171 => {
return;
}
// C line 2893
172 => {
let _ = { let assigned = (*(tab_de).offset(((i).wrapping_add((1 as i32))) as isize)).cost; (*(tab_de).offset((i) as isize)).cost = assigned; assigned };
vm_block = 171; continue;
}
// C line 2892
173 => {
vm_block = if ((((l) == ((0 as i32))) as i32)) != 0 { 172 } else { 170 }; continue;
}
// C line 2891
174 => {
let _ = { let assigned = (*(ci)).decomp_len; l = assigned; assigned };
vm_block = 173; continue;
}
// C line 2890
175 => {
let _ = { let assigned = ptr::addr_of_mut!(*(unicode_db).offset((i) as isize)); ci = assigned; assigned };
vm_block = 174; continue;
}
// C line 2886
176 => {
de = ptr::addr_of_mut!(de_s);
vm_block = 175; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: unicode_gen.c:3148. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn put16(mut data_buf: *mut u8, mut pidx: *mut i32, mut c: u16) -> () {
let mut vm_local_storage = Vec::<u64>::new();
let mut idx: i32 = core::mem::zeroed();
let mut vm_block: usize = 4;
loop { match vm_block {
// C line ?
0 => {
return ();
}
// C line 3154
1 => {
let _ = { let assigned = idx; *(pidx) = assigned; assigned };
vm_block = 0; continue;
}
// C line 3153
2 => {
let _ = { let assigned = (((((c) as i32)).wrapping_shr(((8 as i32)) as u32)) as u8); *(data_buf).offset(({ let old = idx; idx = (idx).wrapping_add(1); old }) as isize) = assigned; assigned };
vm_block = 1; continue;
}
// C line 3152
3 => {
let _ = { let assigned = ((c) as u8); *(data_buf).offset(({ let old = idx; idx = (idx).wrapping_add(1); old }) as isize) = assigned; assigned };
vm_block = 2; continue;
}
// C line 3151
4 => {
let _ = { let assigned = *(pidx); idx = assigned; assigned };
vm_block = 3; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: unicode_gen.c:3157. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn add_decomp_data(mut data_buf: *mut u8, mut pidx: *mut i32, mut de: *mut DecompEntry) -> () {
let mut vm_local_storage = Vec::<u64>::new();
let mut i: i32 = core::mem::zeroed();
let mut j: i32 = core::mem::zeroed();
let mut idx: i32 = core::mem::zeroed();
let mut c: i32 = core::mem::zeroed();
let mut ci: *mut CCInfo = core::mem::zeroed();
let mut n: i32 = core::mem::zeroed();
let mut p: i32 = core::mem::zeroed();
let mut k: i32 = core::mem::zeroed();
let mut vm_block: usize = 130;
loop { match vm_block {
// C line ?
0 => {
return ();
}
// C line 3288
1 => {
let _ = { let assigned = idx; *(pidx) = assigned; assigned };
vm_block = 0; continue;
}
// C line 3167
2 => {
let _ = { let assigned = ((*((*(ci)).decomp_data).offset(((0 as i32)) as isize)) as u16); (*(de)).data_index = assigned; assigned };
vm_block = 1; continue;
}
// C line 3166
3 => {
let _ = if ((((!((((((*(ci)).decomp_len) == ((1 as i32))) as i32)) != 0) as i32)) as i64)) != 0 { tool_assert_rtn(c"add_decomp_data".as_ptr(), c"unicode_gen.c".as_ptr(), (3166 as i32), c"ci->decomp_len == 1".as_ptr()) } else { { let _ = (0 as i32); } };
vm_block = 2; continue;
}
// C line 3165
4 => {
let _ = { let assigned = ptr::addr_of_mut!(*(unicode_db).offset(((*(de)).code) as isize)); ci = assigned; assigned };
vm_block = 3; continue;
}
// C line 3169
5 => {
vm_block = if ((((i) < ((((*(de)).len) as i32))) as i32)) != 0 { 14 } else { 1 }; continue;
}
// C line ?
6 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 5; continue;
}
// C line 3171
7 => {
vm_block = if ((((j) < ((((*(de)).c_len) as i32))) as i32)) != 0 { 12 } else { 6 }; continue;
}
// C line ?
8 => {
let _ = { let old = j; j = (j).wrapping_add(1); old };
vm_block = 7; continue;
}
// C line 3176
9 => {
let _ = put16(data_buf, ptr::addr_of_mut!(idx), ((c) as u16));
vm_block = 8; continue;
}
// C line 3173
10 => {
let _ = { let assigned = (0 as i32); c = assigned; assigned };
vm_block = 9; continue;
}
// C line 3175
11 => {
let _ = { let assigned = *((*(ci)).decomp_data).offset((j) as isize); c = assigned; assigned };
vm_block = 9; continue;
}
// C line 3172
12 => {
vm_block = if (((((*(ci)).decomp_len) == ((0 as i32))) as i32)) != 0 { 10 } else { 11 }; continue;
}
// C line 3171
13 => {
let _ = { let assigned = (0 as i32); j = assigned; assigned };
vm_block = 7; continue;
}
// C line 3170
14 => {
let _ = { let assigned = ptr::addr_of_mut!(*(unicode_db).offset((((*(de)).code).wrapping_add(i)) as isize)); ci = assigned; assigned };
vm_block = 13; continue;
}
// C line 3169
15 => {
let _ = { let assigned = (0 as i32); i = assigned; assigned };
vm_block = 5; continue;
}
// C line 3198
16 => {
let _ = { idx = (idx).wrapping_add(n); idx };
vm_block = 1; continue;
}
// C line 3185
17 => {
vm_block = if ((((i) < ((((*(de)).len) as i32))) as i32)) != 0 { 29 } else { 16 }; continue;
}
// C line ?
18 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 17; continue;
}
// C line 3187
19 => {
vm_block = if ((((j) < ((((*(de)).c_len) as i32))) as i32)) != 0 { 27 } else { 18 }; continue;
}
// C line ?
20 => {
let _ = { let old = j; j = (j).wrapping_add(1); old };
vm_block = 19; continue;
}
// C line 3195
21 => {
let _ = { let old = k; k = (k).wrapping_add(1); old };
vm_block = 20; continue;
}
// C line 3194
22 => {
let _ = { *(data_buf).offset((((idx).wrapping_add(p)).wrapping_add(((k) / ((4 as i32))))) as isize) = ((((((*(data_buf).offset((((idx).wrapping_add(p)).wrapping_add(((k) / ((4 as i32))))) as isize)) as i32)) | (((c).wrapping_shr(((16 as i32)) as u32)).wrapping_shl(((((k) % ((4 as i32)))).wrapping_mul((2 as i32))) as u32)))) as u8); *(data_buf).offset((((idx).wrapping_add(p)).wrapping_add(((k) / ((4 as i32))))) as isize) };
vm_block = 21; continue;
}
// C line 3193
23 => {
let _ = { let assigned = (((c).wrapping_shr(((8 as i32)) as u32)) as u8); *(data_buf).offset((((idx).wrapping_add((k).wrapping_mul((2 as i32)))).wrapping_add((1 as i32))) as isize) = assigned; assigned };
vm_block = 22; continue;
}
// C line 3192
24 => {
let _ = { let assigned = ((c) as u8); *(data_buf).offset(((idx).wrapping_add((k).wrapping_mul((2 as i32)))) as isize) = assigned; assigned };
vm_block = 23; continue;
}
// C line 3189
25 => {
let _ = { let assigned = (0 as i32); c = assigned; assigned };
vm_block = 24; continue;
}
// C line 3191
26 => {
let _ = { let assigned = *((*(ci)).decomp_data).offset((j) as isize); c = assigned; assigned };
vm_block = 24; continue;
}
// C line 3188
27 => {
vm_block = if (((((*(ci)).decomp_len) == ((0 as i32))) as i32)) != 0 { 25 } else { 26 }; continue;
}
// C line 3187
28 => {
let _ = { let assigned = (0 as i32); j = assigned; assigned };
vm_block = 19; continue;
}
// C line 3186
29 => {
let _ = { let assigned = ptr::addr_of_mut!(*(unicode_db).offset((((*(de)).code).wrapping_add(i)) as isize)); ci = assigned; assigned };
vm_block = 28; continue;
}
// C line 3185
30 => {
let _ = { let assigned = (0 as i32); i = assigned; assigned };
vm_block = 17; continue;
}
// C line 3184
31 => {
let _ = { let assigned = (0 as i32); k = assigned; assigned };
vm_block = 30; continue;
}
// C line 3183
32 => {
let _ = { let dst = ((((data_buf).offset(((idx) as isize))) as *mut c_void)) as *mut u8; core::ptr::write_bytes(dst, ((0 as i32)) as u8, (((n) as usize)) as usize); dst as *mut c_void };
vm_block = 31; continue;
}
// C line 3182
33 => {
let _ = { let assigned = (((((*(de)).len) as i32)).wrapping_mul((((*(de)).c_len) as i32))).wrapping_mul((2 as i32)); p = assigned; assigned };
vm_block = 32; continue;
}
// C line 3181
34 => {
let _ = { let assigned = ((((((((*(de)).len) as i32)).wrapping_mul((((*(de)).c_len) as i32))).wrapping_mul((18 as i32))).wrapping_add((7 as i32))) / ((8 as i32))); n = assigned; assigned };
vm_block = 33; continue;
}
// C line 3200
35 => {
vm_block = if ((((i) < ((((*(de)).len) as i32))) as i32)) != 0 { 46 } else { 1 }; continue;
}
// C line ?
36 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 35; continue;
}
// C line 3202
37 => {
vm_block = if ((((j) < ((((*(de)).c_len) as i32))) as i32)) != 0 { 44 } else { 36 }; continue;
}
// C line ?
38 => {
let _ = { let old = j; j = (j).wrapping_add(1); old };
vm_block = 37; continue;
}
// C line 3209
39 => {
let _ = { let assigned = ((c) as u8); *(data_buf).offset(({ let old = idx; idx = (idx).wrapping_add(1); old }) as isize) = assigned; assigned };
vm_block = 38; continue;
}
// C line 3208
40 => {
let _ = if ((((!(((((c) >= ((0 as i32))) as i32)) != 0) as i32)) as i64)) != 0 { tool_assert_rtn(c"add_decomp_data".as_ptr(), c"unicode_gen.c".as_ptr(), (3208 as i32), c"c >= 0".as_ptr()) } else { { let _ = (0 as i32); } };
vm_block = 39; continue;
}
// C line 3207
41 => {
let _ = { let assigned = get_short_code(c); c = assigned; assigned };
vm_block = 40; continue;
}
// C line 3204
42 => {
let _ = { let assigned = (0 as i32); c = assigned; assigned };
vm_block = 41; continue;
}
// C line 3206
43 => {
let _ = { let assigned = *((*(ci)).decomp_data).offset((j) as isize); c = assigned; assigned };
vm_block = 41; continue;
}
// C line 3203
44 => {
vm_block = if (((((*(ci)).decomp_len) == ((0 as i32))) as i32)) != 0 { 42 } else { 43 }; continue;
}
// C line 3202
45 => {
let _ = { let assigned = (0 as i32); j = assigned; assigned };
vm_block = 37; continue;
}
// C line 3201
46 => {
let _ = { let assigned = ptr::addr_of_mut!(*(unicode_db).offset((((*(de)).code).wrapping_add(i)) as isize)); ci = assigned; assigned };
vm_block = 45; continue;
}
// C line 3200
47 => {
let _ = { let assigned = (0 as i32); i = assigned; assigned };
vm_block = 35; continue;
}
// C line 3215
48 => {
vm_block = if ((((j) < ((((*(de)).c_len) as i32))) as i32)) != 0 { 50 } else { 1 }; continue;
}
// C line ?
49 => {
let _ = { let old = j; j = (j).wrapping_add(1); old };
vm_block = 48; continue;
}
// C line 3216
50 => {
let _ = put16(data_buf, ptr::addr_of_mut!(idx), ((*((*(ci)).decomp_data).offset((j) as isize)) as u16));
vm_block = 49; continue;
}
// C line 3215
51 => {
let _ = { let assigned = (0 as i32); j = assigned; assigned };
vm_block = 48; continue;
}
// C line 3214
52 => {
let _ = if ((((!((((((*(ci)).decomp_len) == ((((*(de)).c_len) as i32))) as i32)) != 0) as i32)) as i64)) != 0 { tool_assert_rtn(c"add_decomp_data".as_ptr(), c"unicode_gen.c".as_ptr(), (3214 as i32), c"ci->decomp_len == de->c_len".as_ptr()) } else { { let _ = (0 as i32); } };
vm_block = 51; continue;
}
// C line 3213
53 => {
let _ = { let assigned = ptr::addr_of_mut!(*(unicode_db).offset(((*(de)).code) as isize)); ci = assigned; assigned };
vm_block = 52; continue;
}
// C line 3221
54 => {
vm_block = if ((((i) < ((((*(de)).len) as i32))) as i32)) != 0 { 66 } else { 1 }; continue;
}
// C line ?
55 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 54; continue;
}
// C line 3223
56 => {
vm_block = if ((((j) < ((((*(de)).c_len) as i32))) as i32)) != 0 { 64 } else { 55 }; continue;
}
// C line ?
57 => {
let _ = { let old = j; j = (j).wrapping_add(1); old };
vm_block = 56; continue;
}
// C line 3232
58 => {
let _ = { let assigned = ((c) as u8); *(data_buf).offset(({ let old = idx; idx = (idx).wrapping_add(1); old }) as isize) = assigned; assigned };
vm_block = 57; continue;
}
// C line 3227
59 => {
let _ = { let assigned = (255 as i32); c = assigned; assigned };
vm_block = 58; continue;
}
// C line 3230
60 => {
let _ = if ((((!(((((((c) as u32)) <= ((((254 as i32)) as u32))) as i32)) != 0) as i32)) as i64)) != 0 { tool_assert_rtn(c"add_decomp_data".as_ptr(), c"unicode_gen.c".as_ptr(), (3230 as i32), c"(uint32_t)c <= 254".as_ptr()) } else { { let _ = (0 as i32); } };
vm_block = 58; continue;
}
// C line 3229
61 => {
let _ = { c = (c).wrapping_sub((((*(de)).c_min) as i32)); c };
vm_block = 60; continue;
}
// C line 3226
62 => {
vm_block = if ((((c) == ((32 as i32))) as i32)) != 0 { 59 } else { 61 }; continue;
}
// C line 3225
63 => {
let _ = { let assigned = *((*(ci)).decomp_data).offset((j) as isize); c = assigned; assigned };
vm_block = 62; continue;
}
// C line 3224
64 => {
let _ = if ((((!((((((*(ci)).decomp_len) == ((((*(de)).c_len) as i32))) as i32)) != 0) as i32)) as i64)) != 0 { tool_assert_rtn(c"add_decomp_data".as_ptr(), c"unicode_gen.c".as_ptr(), (3224 as i32), c"ci->decomp_len == de->c_len".as_ptr()) } else { { let _ = (0 as i32); } };
vm_block = 63; continue;
}
// C line 3223
65 => {
let _ = { let assigned = (0 as i32); j = assigned; assigned };
vm_block = 56; continue;
}
// C line 3222
66 => {
let _ = { let assigned = ptr::addr_of_mut!(*(unicode_db).offset((((*(de)).code).wrapping_add(i)) as isize)); ci = assigned; assigned };
vm_block = 65; continue;
}
// C line 3221
67 => {
let _ = { let assigned = (0 as i32); i = assigned; assigned };
vm_block = 54; continue;
}
// C line 3220
68 => {
let _ = { let assigned = (((c).wrapping_shr(((8 as i32)) as u32)) as u8); *(data_buf).offset(({ let old = idx; idx = (idx).wrapping_add(1); old }) as isize) = assigned; assigned };
vm_block = 67; continue;
}
// C line 3219
69 => {
let _ = { let assigned = ((c) as u8); *(data_buf).offset(({ let old = idx; idx = (idx).wrapping_add(1); old }) as isize) = assigned; assigned };
vm_block = 68; continue;
}
// C line 3218
70 => {
let _ = { let assigned = (((*(de)).c_min) as i32); c = assigned; assigned };
vm_block = 69; continue;
}
// C line 3237
71 => {
vm_block = if ((((i) < ((((*(de)).len) as i32))) as i32)) != 0 { 83 } else { 1 }; continue;
}
// C line ?
72 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 71; continue;
}
// C line 3251
73 => {
let _ = { let assigned = ((c) as u8); *(data_buf).offset(({ let old = idx; idx = (idx).wrapping_add(1); old }) as isize) = assigned; assigned };
vm_block = 72; continue;
}
// C line 3250
74 => {
let _ = if ((((!(((((c) >= ((0 as i32))) as i32)) != 0) as i32)) as i64)) != 0 { tool_assert_rtn(c"add_decomp_data".as_ptr(), c"unicode_gen.c".as_ptr(), (3250 as i32), c"c >= 0".as_ptr()) } else { { let _ = (0 as i32); } };
vm_block = 73; continue;
}
// C line 3249
75 => {
let _ = { let assigned = get_short_code(c); c = assigned; assigned };
vm_block = 74; continue;
}
// C line 3246
76 => {
let _ = { let assigned = (0 as i32); c = assigned; assigned };
vm_block = 75; continue;
}
// C line 3248
77 => {
let _ = { let assigned = *((*(ci)).decomp_data).offset(((1 as i32)) as isize); c = assigned; assigned };
vm_block = 75; continue;
}
// C line 3245
78 => {
vm_block = if (((((*(ci)).decomp_len) == ((0 as i32))) as i32)) != 0 { 76 } else { 77 }; continue;
}
// C line 3243
79 => {
let _ = put16(data_buf, ptr::addr_of_mut!(idx), ((c) as u16));
vm_block = 78; continue;
}
// C line 3240
80 => {
let _ = { let assigned = (0 as i32); c = assigned; assigned };
vm_block = 79; continue;
}
// C line 3242
81 => {
let _ = { let assigned = *((*(ci)).decomp_data).offset(((0 as i32)) as isize); c = assigned; assigned };
vm_block = 79; continue;
}
// C line 3239
82 => {
vm_block = if (((((*(ci)).decomp_len) == ((0 as i32))) as i32)) != 0 { 80 } else { 81 }; continue;
}
// C line 3238
83 => {
let _ = { let assigned = ptr::addr_of_mut!(*(unicode_db).offset((((*(de)).code).wrapping_add(i)) as isize)); ci = assigned; assigned };
vm_block = 82; continue;
}
// C line 3237
84 => {
let _ = { let assigned = (0 as i32); i = assigned; assigned };
vm_block = 71; continue;
}
// C line 3236
85 => {
let _ = if ((((!((((((((*(de)).c_len) as i32)) == ((2 as i32))) as i32)) != 0) as i32)) as i64)) != 0 { tool_assert_rtn(c"add_decomp_data".as_ptr(), c"unicode_gen.c".as_ptr(), (3236 as i32), c"de->c_len == 2".as_ptr()) } else { { let _ = (0 as i32); } };
vm_block = 84; continue;
}
// C line 3258
86 => {
vm_block = if ((((i) < ((((*(de)).len) as i32))) as i32)) != 0 { 90 } else { 1 }; continue;
}
// C line ?
87 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 86; continue;
}
// C line 3261
88 => {
let _ = put16(data_buf, ptr::addr_of_mut!(idx), ((*((*(ci)).decomp_data).offset(((1 as i32)) as isize)) as u16));
vm_block = 87; continue;
}
// C line 3260
89 => {
let _ = if ((((!((((((*(ci)).decomp_len) == ((3 as i32))) as i32)) != 0) as i32)) as i64)) != 0 { tool_assert_rtn(c"add_decomp_data".as_ptr(), c"unicode_gen.c".as_ptr(), (3260 as i32), c"ci->decomp_len == 3".as_ptr()) } else { { let _ = (0 as i32); } };
vm_block = 88; continue;
}
// C line 3259
90 => {
let _ = { let assigned = ptr::addr_of_mut!(*(unicode_db).offset((((*(de)).code).wrapping_add(i)) as isize)); ci = assigned; assigned };
vm_block = 89; continue;
}
// C line 3258
91 => {
let _ = { let assigned = (0 as i32); i = assigned; assigned };
vm_block = 86; continue;
}
// C line 3257
92 => {
let _ = put16(data_buf, ptr::addr_of_mut!(idx), ((*((*(ci)).decomp_data).offset(((2 as i32)) as isize)) as u16));
vm_block = 91; continue;
}
// C line 3256
93 => {
let _ = put16(data_buf, ptr::addr_of_mut!(idx), ((*((*(ci)).decomp_data).offset(((0 as i32)) as isize)) as u16));
vm_block = 92; continue;
}
// C line 3255
94 => {
let _ = if ((((!((((((*(ci)).decomp_len) == ((3 as i32))) as i32)) != 0) as i32)) as i64)) != 0 { tool_assert_rtn(c"add_decomp_data".as_ptr(), c"unicode_gen.c".as_ptr(), (3255 as i32), c"ci->decomp_len == 3".as_ptr()) } else { { let _ = (0 as i32); } };
vm_block = 93; continue;
}
// C line 3254
95 => {
let _ = { let assigned = ptr::addr_of_mut!(*(unicode_db).offset(((*(de)).code) as isize)); ci = assigned; assigned };
vm_block = 94; continue;
}
// C line 3264
96 => {
vm_block = if ((((i) < ((((*(de)).len) as i32))) as i32)) != 0 { 106 } else { 1 }; continue;
}
// C line ?
97 => {
let _ = { i = (i).wrapping_add((2 as i32)); i };
vm_block = 96; continue;
}
// C line 3273
98 => {
let _ = { let assigned = ((c) as u8); *(data_buf).offset(({ let old = idx; idx = (idx).wrapping_add(1); old }) as isize) = assigned; assigned };
vm_block = 97; continue;
}
// C line 3272
99 => {
let _ = if ((((!(((((c) >= ((0 as i32))) as i32)) != 0) as i32)) as i64)) != 0 { tool_assert_rtn(c"add_decomp_data".as_ptr(), c"unicode_gen.c".as_ptr(), (3272 as i32), c"c >= 0".as_ptr()) } else { { let _ = (0 as i32); } };
vm_block = 98; continue;
}
// C line 3271
100 => {
let _ = { let assigned = get_short_code(c); c = assigned; assigned };
vm_block = 99; continue;
}
// C line 3270
101 => {
let _ = { let assigned = *((*(ci)).decomp_data).offset(((1 as i32)) as isize); c = assigned; assigned };
vm_block = 100; continue;
}
// C line 3269
102 => {
let _ = { let assigned = ((c) as u8); *(data_buf).offset(({ let old = idx; idx = (idx).wrapping_add(1); old }) as isize) = assigned; assigned };
vm_block = 101; continue;
}
// C line 3268
103 => {
let _ = if ((((!(((((c) >= ((0 as i32))) as i32)) != 0) as i32)) as i64)) != 0 { tool_assert_rtn(c"add_decomp_data".as_ptr(), c"unicode_gen.c".as_ptr(), (3268 as i32), c"c >= 0".as_ptr()) } else { { let _ = (0 as i32); } };
vm_block = 102; continue;
}
// C line 3267
104 => {
let _ = { let assigned = get_short_code(c); c = assigned; assigned };
vm_block = 103; continue;
}
// C line 3266
105 => {
let _ = { let assigned = *((*(ci)).decomp_data).offset(((0 as i32)) as isize); c = assigned; assigned };
vm_block = 104; continue;
}
// C line 3265
106 => {
let _ = { let assigned = ptr::addr_of_mut!(*(unicode_db).offset((((*(de)).code).wrapping_add(i)) as isize)); ci = assigned; assigned };
vm_block = 105; continue;
}
// C line 3264
107 => {
let _ = { let assigned = (0 as i32); i = assigned; assigned };
vm_block = 96; continue;
}
// C line 3276
108 => {
vm_block = if ((((i) < ((((*(de)).len) as i32))) as i32)) != 0 { 116 } else { 1 }; continue;
}
// C line ?
109 => {
let _ = { i = (i).wrapping_add((2 as i32)); i };
vm_block = 108; continue;
}
// C line 3283
110 => {
let _ = { let assigned = ((c) as u8); *(data_buf).offset(({ let old = idx; idx = (idx).wrapping_add(1); old }) as isize) = assigned; assigned };
vm_block = 109; continue;
}
// C line 3282
111 => {
let _ = if ((((!(((((c) >= ((0 as i32))) as i32)) != 0) as i32)) as i64)) != 0 { tool_assert_rtn(c"add_decomp_data".as_ptr(), c"unicode_gen.c".as_ptr(), (3282 as i32), c"c >= 0".as_ptr()) } else { { let _ = (0 as i32); } };
vm_block = 110; continue;
}
// C line 3281
112 => {
let _ = { let assigned = get_short_code(c); c = assigned; assigned };
vm_block = 111; continue;
}
// C line 3280
113 => {
let _ = { let assigned = *((*(ci)).decomp_data).offset(((1 as i32)) as isize); c = assigned; assigned };
vm_block = 112; continue;
}
// C line 3279
114 => {
let _ = put16(data_buf, ptr::addr_of_mut!(idx), ((c) as u16));
vm_block = 113; continue;
}
// C line 3278
115 => {
let _ = { let assigned = *((*(ci)).decomp_data).offset(((0 as i32)) as isize); c = assigned; assigned };
vm_block = 114; continue;
}
// C line 3277
116 => {
let _ = { let assigned = ptr::addr_of_mut!(*(unicode_db).offset((((*(de)).code).wrapping_add(i)) as isize)); ci = assigned; assigned };
vm_block = 115; continue;
}
// C line 3276
117 => {
let _ = { let assigned = (0 as i32); i = assigned; assigned };
vm_block = 108; continue;
}
// C line 3286
118 => {
let _ = std::process::abort();
vm_block = 1; continue;
}
// C line 3275
119 => {
vm_block = if (((((((*(de)).v_type) as i32)) <= ((DECOMP_TYPE_LS2_UL as i32))) as i32)) != 0 { 117 } else { 118 }; continue;
}
// C line 3263
120 => {
vm_block = if (((((((*(de)).v_type) as i32)) <= ((DECOMP_TYPE_S2_UL as i32))) as i32)) != 0 { 107 } else { 119 }; continue;
}
// C line 3253
121 => {
vm_block = if (((((((*(de)).v_type) as i32)) <= ((DECOMP_TYPE_PAT3 as i32))) as i32)) != 0 { 95 } else { 120 }; continue;
}
// C line 3235
122 => {
vm_block = if (((((((*(de)).v_type) as i32)) <= ((DECOMP_TYPE_LS2 as i32))) as i32)) != 0 { 85 } else { 121 }; continue;
}
// C line 3217
123 => {
vm_block = if (((((((*(de)).v_type) as i32)) <= ((DECOMP_TYPE_B18 as i32))) as i32)) != 0 { 70 } else { 122 }; continue;
}
// C line 3212
124 => {
vm_block = if (((((((*(de)).v_type) as i32)) <= ((DECOMP_TYPE_I4_2 as i32))) as i32)) != 0 { 53 } else { 123 }; continue;
}
// C line 3199
125 => {
vm_block = if (((((((*(de)).v_type) as i32)) <= ((DECOMP_TYPE_S5 as i32))) as i32)) != 0 { 47 } else { 124 }; continue;
}
// C line 3179
126 => {
vm_block = if (((((((*(de)).v_type) as i32)) <= ((DECOMP_TYPE_LL2 as i32))) as i32)) != 0 { 34 } else { 125 }; continue;
}
// C line 3168
127 => {
vm_block = if (((((((*(de)).v_type) as i32)) <= ((DECOMP_TYPE_L7 as i32))) as i32)) != 0 { 15 } else { 126 }; continue;
}
// C line 3164
128 => {
vm_block = if (((((((*(de)).v_type) as i32)) <= ((DECOMP_TYPE_C1 as i32))) as i32)) != 0 { 4 } else { 127 }; continue;
}
// C line 3163
129 => {
let _ = { let assigned = ((idx) as u16); (*(de)).data_index = assigned; assigned };
vm_block = 128; continue;
}
// C line 3162
130 => {
let _ = { let assigned = *(pidx); idx = assigned; assigned };
vm_block = 129; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: unicode_gen.c:3292. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn dump_large_char() -> () {
let mut vm_local_storage = Vec::<u64>::new();
let mut i: i32 = core::mem::zeroed();
let mut j: i32 = core::mem::zeroed();
let mut ci: *mut CCInfo = core::mem::zeroed();
let mut vm_block: usize = 9;
loop { match vm_block {
// C line ?
0 => {
return ();
}
// C line 5
1 => {
vm_block = if ((((i) <= ((1114111 as i32))) as i32)) != 0 { 8 } else { 0 }; continue;
}
// C line ?
2 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 1; continue;
}
// C line 7
3 => {
vm_block = if ((((j) < ((*(ci)).decomp_len)) as i32)) != 0 { 6 } else { 2 }; continue;
}
// C line ?
4 => {
let _ = { let old = j; j = (j).wrapping_add(1); old };
vm_block = 3; continue;
}
// C line 9
5 => {
let _ = tool_printf(c"%05x\n".as_ptr(), &[ToolPrintArg::Int(*((*(ci)).decomp_data).offset((j) as isize) as u64)]);
vm_block = 4; continue;
}
// C line 8
6 => {
vm_block = if ((((*((*(ci)).decomp_data).offset((j) as isize)) > ((65535 as i32))) as i32)) != 0 { 5 } else { 4 }; continue;
}
// C line 7
7 => {
let _ = { let assigned = (0 as i32); j = assigned; assigned };
vm_block = 3; continue;
}
// C line 6
8 => {
ci = ptr::addr_of_mut!(*(unicode_db).offset((i) as isize));
vm_block = 7; continue;
}
// C line 5
9 => {
let _ = { let assigned = (0 as i32); i = assigned; assigned };
vm_block = 1; continue;
}
_ => std::process::abort(),
} }
}

#[cfg(all(not(feature="unicode-dump-decomp-table")))]
// Generated from the official C AST; original goto CFG retained.
// c: unicode_gen.c:3307. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn build_decompose_table(mut f: *mut FILE) -> () {
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
// C line 3407
1 => {
let _ = free(((tab_de) as *mut c_void));
vm_block = 0; continue;
}
// C line 3405
2 => {
let _ = free(((data_buf) as *mut c_void));
vm_block = 1; continue;
}
// C line 3403
3 => {
let _ = build_compose_table(f, tab_de);
vm_block = 2; continue;
}
// C line 3401
4 => {
let _ = tool_fprintf(f, c"\n};\n\n".as_ptr(), &[]);
vm_block = 3; continue;
}
// C line 3396
5 => {
vm_block = if ((((i) < (data_len)) as i32)) != 0 { 9 } else { 4 }; continue;
}
// C line ?
6 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 5; continue;
}
// C line 3399
7 => {
let _ = tool_fprintf(f, c" 0x%02x,".as_ptr(), &[ToolPrintArg::Int(((*(data_buf).offset((i) as isize)) as i32) as u64)]);
vm_block = 6; continue;
}
// C line 3398
8 => {
let _ = tool_fprintf(f, c"\n   ".as_ptr(), &[]);
vm_block = 7; continue;
}
// C line 3397
9 => {
vm_block = if ((((((i) % ((8 as i32)))) == ((0 as i32))) as i32)) != 0 { 8 } else { 7 }; continue;
}
// C line 3396
10 => {
let _ = { let assigned = (0 as i32); i = assigned; assigned };
vm_block = 5; continue;
}
// C line 3395
11 => {
let _ = tool_fprintf(f, c"static const uint8_t unicode_decomp_data[%d] = {".as_ptr(), &[ToolPrintArg::Int(data_len as u64)]);
vm_block = 10; continue;
}
// C line 3394
12 => {
let _ = { total_table_bytes = (total_table_bytes).wrapping_add(((data_len) as u32)); total_table_bytes };
vm_block = 11; continue;
}
// C line 3393
13 => {
let _ = { let old = total_tables; total_tables = (total_tables).wrapping_add(1); old };
vm_block = 12; continue;
}
// C line 3391
14 => {
let _ = tool_fprintf(f, c"\n};\n\n".as_ptr(), &[]);
vm_block = 13; continue;
}
// C line 3382
15 => {
vm_block = if ((((i) <= (code_max)) as i32)) != 0 { 22 } else { 14 }; continue;
}
// C line ?
16 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 15; continue;
}
// C line 3388
17 => {
let _ = { i = (i).wrapping_add(((((*(de)).len) as i32)).wrapping_sub((1 as i32))); i };
vm_block = 16; continue;
}
// C line 3387
18 => {
let _ = tool_fprintf(f, c" 0x%04x,".as_ptr(), &[ToolPrintArg::Int((((*(de)).data_index) as i32) as u64)]);
vm_block = 17; continue;
}
// C line 3386
19 => {
let _ = tool_fprintf(f, c"\n   ".as_ptr(), &[]);
vm_block = 18; continue;
}
// C line 3385
20 => {
vm_block = if (((((({ let old = count; count = (count).wrapping_add(1); old }) % ((8 as i32)))) == ((0 as i32))) as i32)) != 0 { 19 } else { 18 }; continue;
}
// C line 3384
21 => {
vm_block = if (((((((*(de)).len) as i32)) != ((0 as i32))) as i32)) != 0 { 20 } else { 16 }; continue;
}
// C line 3383
22 => {
let _ = { let assigned = ptr::addr_of_mut!(*(tab_de).offset((i) as isize)); de = assigned; assigned };
vm_block = 21; continue;
}
// C line 3382
23 => {
let _ = { let assigned = (0 as i32); i = assigned; assigned };
vm_block = 15; continue;
}
// C line 3381
24 => {
let _ = { let assigned = (0 as i32); count = assigned; assigned };
vm_block = 23; continue;
}
// C line 3380
25 => {
let _ = tool_fprintf(f, c"static const uint16_t unicode_decomp_table2[%d] = {".as_ptr(), &[ToolPrintArg::Int(array_len as u64)]);
vm_block = 24; continue;
}
// C line 3379
26 => {
let _ = { total_table_bytes = (((((total_table_bytes) as usize)).wrapping_add((((array_len) as usize)).wrapping_mul((size_of::<u16>() as usize)))) as u32); total_table_bytes };
vm_block = 25; continue;
}
// C line 3378
27 => {
let _ = { let old = total_tables; total_tables = (total_tables).wrapping_add(1); old };
vm_block = 26; continue;
}
// C line 3376
28 => {
let _ = tool_fprintf(f, c"\n};\n\n".as_ptr(), &[]);
vm_block = 27; continue;
}
// C line 3362
29 => {
vm_block = if ((((i) <= (code_max)) as i32)) != 0 { 37 } else { 28 }; continue;
}
// C line ?
30 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 29; continue;
}
// C line 3373
31 => {
let _ = { i = (i).wrapping_add(((((*(de)).len) as i32)).wrapping_sub((1 as i32))); i };
vm_block = 30; continue;
}
// C line 3372
32 => {
let _ = tool_fprintf(f, c" 0x%08x,".as_ptr(), &[ToolPrintArg::Int(v as u64)]);
vm_block = 31; continue;
}
// C line 3368
33 => {
let _ = { let assigned = ((((((((((*(de)).code).wrapping_shl((((32 as i32)).wrapping_sub((18 as i32))) as u32)) | (((((*(de)).len) as i32)).wrapping_shl(((((32 as i32)).wrapping_sub((18 as i32))).wrapping_sub((7 as i32))) as u32)))) | (((((*(de)).v_type) as i32)).wrapping_shl((((((32 as i32)).wrapping_sub((18 as i32))).wrapping_sub((7 as i32))).wrapping_sub((6 as i32))) as u32)))) | ((((*(unicode_db).offset(((*(de)).code) as isize)).is_compat()) as i32)))) as u32); v = assigned; assigned };
vm_block = 32; continue;
}
// C line 3367
34 => {
let _ = tool_fprintf(f, c"\n   ".as_ptr(), &[]);
vm_block = 33; continue;
}
// C line 3366
35 => {
vm_block = if (((((({ let old = count; count = (count).wrapping_add(1); old }) % ((4 as i32)))) == ((0 as i32))) as i32)) != 0 { 34 } else { 33 }; continue;
}
// C line 3364
36 => {
vm_block = if (((((((*(de)).len) as i32)) != ((0 as i32))) as i32)) != 0 { 35 } else { 30 }; continue;
}
// C line 3363
37 => {
let _ = { let assigned = ptr::addr_of_mut!(*(tab_de).offset((i) as isize)); de = assigned; assigned };
vm_block = 36; continue;
}
// C line 3362
38 => {
let _ = { let assigned = (0 as i32); i = assigned; assigned };
vm_block = 29; continue;
}
// C line 3361
39 => {
let _ = { let assigned = (0 as i32); count = assigned; assigned };
vm_block = 38; continue;
}
// C line 3360
40 => {
let _ = tool_fprintf(f, c"static const uint32_t unicode_decomp_table1[%d] = {".as_ptr(), &[ToolPrintArg::Int(array_len as u64)]);
vm_block = 39; continue;
}
// C line 3359
41 => {
let _ = { total_table_bytes = (((((total_table_bytes) as usize)).wrapping_add((((array_len) as usize)).wrapping_mul((size_of::<u32>() as usize)))) as u32); total_table_bytes };
vm_block = 40; continue;
}
// C line 3358
42 => {
let _ = { let old = total_tables; total_tables = (total_tables).wrapping_add(1); old };
vm_block = 41; continue;
}
// C line 3325
43 => {
vm_block = if ((((i) <= (code_max)) as i32)) != 0 { 49 } else { 42 }; continue;
}
// C line ?
44 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 43; continue;
}
// C line 3330
45 => {
let _ = { let old = array_len; array_len = (array_len).wrapping_add(1); old };
vm_block = 44; continue;
}
// C line 3329
46 => {
let _ = { i = (i).wrapping_add(((((*(de)).len) as i32)).wrapping_sub((1 as i32))); i };
vm_block = 45; continue;
}
// C line 3328
47 => {
let _ = add_decomp_data(data_buf, ptr::addr_of_mut!(data_len), de);
vm_block = 46; continue;
}
// C line 3327
48 => {
vm_block = if (((((((*(de)).len) as i32)) != ((0 as i32))) as i32)) != 0 { 47 } else { 44 }; continue;
}
// C line 3326
49 => {
let _ = { let assigned = ptr::addr_of_mut!(*(tab_de).offset((i) as isize)); de = assigned; assigned };
vm_block = 48; continue;
}
// C line 3325
50 => {
let _ = { let assigned = (0 as i32); i = assigned; assigned };
vm_block = 43; continue;
}
// C line 3324
51 => {
let _ = { let assigned = (0 as i32); array_len = assigned; assigned };
vm_block = 50; continue;
}
// C line 3323
52 => {
let _ = { let assigned = (0 as i32); data_len = assigned; assigned };
vm_block = 51; continue;
}
// C line 3322
53 => {
let _ = { let assigned = ((malloc((((100000 as i32)) as usize))) as *mut u8); data_buf = assigned; assigned };
vm_block = 52; continue;
}
// C line 3317
54 => {
vm_block = if ((((i) >= ((0 as i32))) as i32)) != 0 { 56 } else { 53 }; continue;
}
// C line ?
55 => {
let _ = { let old = i; i = (i).wrapping_sub(1); old };
vm_block = 54; continue;
}
// C line 3318
56 => {
let _ = find_decomp_run(tab_de, i);
vm_block = 55; continue;
}
// C line 3317
57 => {
let _ = { let assigned = code_max; i = assigned; assigned };
vm_block = 54; continue;
}
// C line 3315
58 => {
let _ = { let assigned = ((mallocz(((((code_max).wrapping_add((2 as i32))) as usize)).wrapping_mul((size_of::<DecompEntry>() as usize)))) as *mut DecompEntry); tab_de = assigned; assigned };
vm_block = 57; continue;
}
// C line 3313
59 => {
let _ = { let assigned = (1114111 as i32); code_max = assigned; assigned };
vm_block = 58; continue;
}
// C line 3310
60 => {
de = ptr::addr_of_mut!(de_s);
vm_block = 59; continue;
}
_ => std::process::abort(),
} }
}

#[cfg(all(feature="unicode-dump-decomp-table"))]
// Generated from the official C AST; original goto CFG retained.
// c: unicode_gen.c:3307. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn build_decompose_table(mut f: *mut FILE) -> () {
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
let mut size: i32 = core::mem::zeroed();
let mut size1: i32 = core::mem::zeroed();
let mut v: u32 = core::mem::zeroed();
let mut vm_block: usize = 72;
loop { match vm_block {
// C line ?
0 => {
return ();
}
// C line 3407
1 => {
let _ = free(((tab_de) as *mut c_void));
vm_block = 0; continue;
}
// C line 3405
2 => {
let _ = free(((data_buf) as *mut c_void));
vm_block = 1; continue;
}
// C line 3403
3 => {
let _ = build_compose_table(f, tab_de);
vm_block = 2; continue;
}
// C line 3401
4 => {
let _ = tool_fprintf(f, c"\n};\n\n".as_ptr(), &[]);
vm_block = 3; continue;
}
// C line 3396
5 => {
vm_block = if ((((i) < (data_len)) as i32)) != 0 { 9 } else { 4 }; continue;
}
// C line ?
6 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 5; continue;
}
// C line 3399
7 => {
let _ = tool_fprintf(f, c" 0x%02x,".as_ptr(), &[ToolPrintArg::Int(((*(data_buf).offset((i) as isize)) as i32) as u64)]);
vm_block = 6; continue;
}
// C line 3398
8 => {
let _ = tool_fprintf(f, c"\n   ".as_ptr(), &[]);
vm_block = 7; continue;
}
// C line 3397
9 => {
vm_block = if ((((((i) % ((8 as i32)))) == ((0 as i32))) as i32)) != 0 { 8 } else { 7 }; continue;
}
// C line 3396
10 => {
let _ = { let assigned = (0 as i32); i = assigned; assigned };
vm_block = 5; continue;
}
// C line 3395
11 => {
let _ = tool_fprintf(f, c"static const uint8_t unicode_decomp_data[%d] = {".as_ptr(), &[ToolPrintArg::Int(data_len as u64)]);
vm_block = 10; continue;
}
// C line 3394
12 => {
let _ = { total_table_bytes = (total_table_bytes).wrapping_add(((data_len) as u32)); total_table_bytes };
vm_block = 11; continue;
}
// C line 3393
13 => {
let _ = { let old = total_tables; total_tables = (total_tables).wrapping_add(1); old };
vm_block = 12; continue;
}
// C line 3391
14 => {
let _ = tool_fprintf(f, c"\n};\n\n".as_ptr(), &[]);
vm_block = 13; continue;
}
// C line 3382
15 => {
vm_block = if ((((i) <= (code_max)) as i32)) != 0 { 22 } else { 14 }; continue;
}
// C line ?
16 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 15; continue;
}
// C line 3388
17 => {
let _ = { i = (i).wrapping_add(((((*(de)).len) as i32)).wrapping_sub((1 as i32))); i };
vm_block = 16; continue;
}
// C line 3387
18 => {
let _ = tool_fprintf(f, c" 0x%04x,".as_ptr(), &[ToolPrintArg::Int((((*(de)).data_index) as i32) as u64)]);
vm_block = 17; continue;
}
// C line 3386
19 => {
let _ = tool_fprintf(f, c"\n   ".as_ptr(), &[]);
vm_block = 18; continue;
}
// C line 3385
20 => {
vm_block = if (((((({ let old = count; count = (count).wrapping_add(1); old }) % ((8 as i32)))) == ((0 as i32))) as i32)) != 0 { 19 } else { 18 }; continue;
}
// C line 3384
21 => {
vm_block = if (((((((*(de)).len) as i32)) != ((0 as i32))) as i32)) != 0 { 20 } else { 16 }; continue;
}
// C line 3383
22 => {
let _ = { let assigned = ptr::addr_of_mut!(*(tab_de).offset((i) as isize)); de = assigned; assigned };
vm_block = 21; continue;
}
// C line 3382
23 => {
let _ = { let assigned = (0 as i32); i = assigned; assigned };
vm_block = 15; continue;
}
// C line 3381
24 => {
let _ = { let assigned = (0 as i32); count = assigned; assigned };
vm_block = 23; continue;
}
// C line 3380
25 => {
let _ = tool_fprintf(f, c"static const uint16_t unicode_decomp_table2[%d] = {".as_ptr(), &[ToolPrintArg::Int(array_len as u64)]);
vm_block = 24; continue;
}
// C line 3379
26 => {
let _ = { total_table_bytes = (((((total_table_bytes) as usize)).wrapping_add((((array_len) as usize)).wrapping_mul((size_of::<u16>() as usize)))) as u32); total_table_bytes };
vm_block = 25; continue;
}
// C line 3378
27 => {
let _ = { let old = total_tables; total_tables = (total_tables).wrapping_add(1); old };
vm_block = 26; continue;
}
// C line 3376
28 => {
let _ = tool_fprintf(f, c"\n};\n\n".as_ptr(), &[]);
vm_block = 27; continue;
}
// C line 3362
29 => {
vm_block = if ((((i) <= (code_max)) as i32)) != 0 { 37 } else { 28 }; continue;
}
// C line ?
30 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 29; continue;
}
// C line 3373
31 => {
let _ = { i = (i).wrapping_add(((((*(de)).len) as i32)).wrapping_sub((1 as i32))); i };
vm_block = 30; continue;
}
// C line 3372
32 => {
let _ = tool_fprintf(f, c" 0x%08x,".as_ptr(), &[ToolPrintArg::Int(v as u64)]);
vm_block = 31; continue;
}
// C line 3368
33 => {
let _ = { let assigned = ((((((((((*(de)).code).wrapping_shl((((32 as i32)).wrapping_sub((18 as i32))) as u32)) | (((((*(de)).len) as i32)).wrapping_shl(((((32 as i32)).wrapping_sub((18 as i32))).wrapping_sub((7 as i32))) as u32)))) | (((((*(de)).v_type) as i32)).wrapping_shl((((((32 as i32)).wrapping_sub((18 as i32))).wrapping_sub((7 as i32))).wrapping_sub((6 as i32))) as u32)))) | ((((*(unicode_db).offset(((*(de)).code) as isize)).is_compat()) as i32)))) as u32); v = assigned; assigned };
vm_block = 32; continue;
}
// C line 3367
34 => {
let _ = tool_fprintf(f, c"\n   ".as_ptr(), &[]);
vm_block = 33; continue;
}
// C line 3366
35 => {
vm_block = if (((((({ let old = count; count = (count).wrapping_add(1); old }) % ((4 as i32)))) == ((0 as i32))) as i32)) != 0 { 34 } else { 33 }; continue;
}
// C line 3364
36 => {
vm_block = if (((((((*(de)).len) as i32)) != ((0 as i32))) as i32)) != 0 { 35 } else { 30 }; continue;
}
// C line 3363
37 => {
let _ = { let assigned = ptr::addr_of_mut!(*(tab_de).offset((i) as isize)); de = assigned; assigned };
vm_block = 36; continue;
}
// C line 3362
38 => {
let _ = { let assigned = (0 as i32); i = assigned; assigned };
vm_block = 29; continue;
}
// C line 3361
39 => {
let _ = { let assigned = (0 as i32); count = assigned; assigned };
vm_block = 38; continue;
}
// C line 3360
40 => {
let _ = tool_fprintf(f, c"static const uint32_t unicode_decomp_table1[%d] = {".as_ptr(), &[ToolPrintArg::Int(array_len as u64)]);
vm_block = 39; continue;
}
// C line 3359
41 => {
let _ = { total_table_bytes = (((((total_table_bytes) as usize)).wrapping_add((((array_len) as usize)).wrapping_mul((size_of::<u32>() as usize)))) as u32); total_table_bytes };
vm_block = 40; continue;
}
// C line 3358
42 => {
let _ = { let old = total_tables; total_tables = (total_tables).wrapping_add(1); old };
vm_block = 41; continue;
}
// C line 3353
43 => {
let _ = tool_printf(c"array_len=%d estimated size=%d bytes actual=%d bytes\n".as_ptr(), &[ToolPrintArg::Int(array_len as u64), ToolPrintArg::Int(size as u64), ToolPrintArg::Int(((array_len).wrapping_mul((6 as i32))).wrapping_add(data_len) as u64)]);
vm_block = 42; continue;
}
// C line 3341
44 => {
vm_block = if ((((i) <= (code_max)) as i32)) != 0 { 51 } else { 43 }; continue;
}
// C line ?
45 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 44; continue;
}
// C line 3349
46 => {
let _ = { size = (size).wrapping_add(size1); size };
vm_block = 45; continue;
}
// C line 3348
47 => {
let _ = { i = (i).wrapping_add(((((*(de)).len) as i32)).wrapping_sub((1 as i32))); i };
vm_block = 46; continue;
}
// C line 3345
48 => {
let _ = tool_printf(c"%05x %3d %6s %2d %1d %4d\n".as_ptr(), &[ToolPrintArg::Int(i as u64), ToolPrintArg::Int((((*(de)).len) as i32) as u64), ToolPrintArg::Str(*((decomp_type_str).as_mut_ptr()).offset(((*(de)).v_type) as isize) as *const c_char), ToolPrintArg::Int((((*(de)).c_len) as i32) as u64), ToolPrintArg::Int((((*(unicode_db).offset((i) as isize)).is_compat()) as i32) as u64), ToolPrintArg::Int(size1 as u64)]);
vm_block = 47; continue;
}
// C line 3344
49 => {
let _ = { let assigned = get_decomp_run_size(de); size1 = assigned; assigned };
vm_block = 48; continue;
}
// C line 3343
50 => {
vm_block = if (((((((*(de)).len) as i32)) != ((0 as i32))) as i32)) != 0 { 49 } else { 45 }; continue;
}
// C line 3342
51 => {
let _ = { let assigned = ptr::addr_of_mut!(*(tab_de).offset((i) as isize)); de = assigned; assigned };
vm_block = 50; continue;
}
// C line 3341
52 => {
let _ = { let assigned = (0 as i32); i = assigned; assigned };
vm_block = 44; continue;
}
// C line 3340
53 => {
let _ = { let assigned = (0 as i32); size = assigned; assigned };
vm_block = 52; continue;
}
// C line 3339
54 => {
let _ = tool_printf(c"START LEN   TYPE  L C SIZE\n".as_ptr(), &[]);
vm_block = 53; continue;
}
// C line 3325
55 => {
vm_block = if ((((i) <= (code_max)) as i32)) != 0 { 61 } else { 54 }; continue;
}
// C line ?
56 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 55; continue;
}
// C line 3330
57 => {
let _ = { let old = array_len; array_len = (array_len).wrapping_add(1); old };
vm_block = 56; continue;
}
// C line 3329
58 => {
let _ = { i = (i).wrapping_add(((((*(de)).len) as i32)).wrapping_sub((1 as i32))); i };
vm_block = 57; continue;
}
// C line 3328
59 => {
let _ = add_decomp_data(data_buf, ptr::addr_of_mut!(data_len), de);
vm_block = 58; continue;
}
// C line 3327
60 => {
vm_block = if (((((((*(de)).len) as i32)) != ((0 as i32))) as i32)) != 0 { 59 } else { 56 }; continue;
}
// C line 3326
61 => {
let _ = { let assigned = ptr::addr_of_mut!(*(tab_de).offset((i) as isize)); de = assigned; assigned };
vm_block = 60; continue;
}
// C line 3325
62 => {
let _ = { let assigned = (0 as i32); i = assigned; assigned };
vm_block = 55; continue;
}
// C line 3324
63 => {
let _ = { let assigned = (0 as i32); array_len = assigned; assigned };
vm_block = 62; continue;
}
// C line 3323
64 => {
let _ = { let assigned = (0 as i32); data_len = assigned; assigned };
vm_block = 63; continue;
}
// C line 3322
65 => {
let _ = { let assigned = ((malloc((((100000 as i32)) as usize))) as *mut u8); data_buf = assigned; assigned };
vm_block = 64; continue;
}
// C line 3317
66 => {
vm_block = if ((((i) >= ((0 as i32))) as i32)) != 0 { 68 } else { 65 }; continue;
}
// C line ?
67 => {
let _ = { let old = i; i = (i).wrapping_sub(1); old };
vm_block = 66; continue;
}
// C line 3318
68 => {
let _ = find_decomp_run(tab_de, i);
vm_block = 67; continue;
}
// C line 3317
69 => {
let _ = { let assigned = code_max; i = assigned; assigned };
vm_block = 66; continue;
}
// C line 3315
70 => {
let _ = { let assigned = ((mallocz(((((code_max).wrapping_add((2 as i32))) as usize)).wrapping_mul((size_of::<DecompEntry>() as usize)))) as *mut DecompEntry); tab_de = assigned; assigned };
vm_block = 69; continue;
}
// C line 3313
71 => {
let _ = { let assigned = (1114111 as i32); code_max = assigned; assigned };
vm_block = 70; continue;
}
// C line 3310
72 => {
de = ptr::addr_of_mut!(de_s);
vm_block = 71; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: unicode_gen.c:3417. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn ce_cmp(mut p1: *const c_void, mut p2: *const c_void) -> i32 {
let mut vm_local_storage = Vec::<u64>::new();
let mut ce1: *const ComposeEntry = core::mem::zeroed();
let mut ce2: *const ComposeEntry = core::mem::zeroed();
let mut i: i32 = core::mem::zeroed();
let mut vm_block: usize = 10;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 3429
1 => {
return (0 as i32);
}
// C line 3423
2 => {
vm_block = if ((((i) < ((2 as i32))) as i32)) != 0 { 7 } else { 1 }; continue;
}
// C line ?
3 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 2; continue;
}
// C line 3425
4 => {
return ((1 as i32)).wrapping_neg();
}
// C line 3427
5 => {
return (1 as i32);
}
// C line 3426
6 => {
vm_block = if ((((*(((*(ce1)).c).as_ptr()).offset((i) as isize)) > (*(((*(ce2)).c).as_ptr()).offset((i) as isize))) as i32)) != 0 { 5 } else { 3 }; continue;
}
// C line 3424
7 => {
vm_block = if ((((*(((*(ce1)).c).as_ptr()).offset((i) as isize)) < (*(((*(ce2)).c).as_ptr()).offset((i) as isize))) as i32)) != 0 { 4 } else { 6 }; continue;
}
// C line 3423
8 => {
let _ = { let assigned = (0 as i32); i = assigned; assigned };
vm_block = 2; continue;
}
// C line 3420
9 => {
ce2 = ((p2) as *const ComposeEntry);
vm_block = 8; continue;
}
// C line 3419
10 => {
ce1 = ((p1) as *const ComposeEntry);
vm_block = 9; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: unicode_gen.c:3433. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn get_decomp_pos(mut tab_de: *const DecompEntry, mut c: i32) -> i32 {
let mut vm_local_storage = Vec::<u64>::new();
let mut i: i32 = core::mem::zeroed();
let mut v: i32 = core::mem::zeroed();
let mut k: i32 = core::mem::zeroed();
let mut de: *const DecompEntry = core::mem::zeroed();
let mut vm_block: usize = 15;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 3453
1 => {
return ((1 as i32)).wrapping_neg();
}
// C line 3439
2 => {
vm_block = if ((((i) <= ((1114111 as i32))) as i32)) != 0 { 13 } else { 1 }; continue;
}
// C line ?
3 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 2; continue;
}
// C line 3450
4 => {
let _ = { let old = k; k = (k).wrapping_add(1); old };
vm_block = 3; continue;
}
// C line 3449
5 => {
let _ = { i = (i).wrapping_add(((((*(de)).len) as i32)).wrapping_sub((1 as i32))); i };
vm_block = 4; continue;
}
// C line 3447
6 => {
return v;
}
// C line 3446
7 => {
let _ = if ((((!(((((v) < ((65536 as i32))) as i32)) != 0) as i32)) as i64)) != 0 { tool_assert_rtn(c"get_decomp_pos".as_ptr(), c"unicode_gen.c".as_ptr(), (3446 as i32), c"v < 65536".as_ptr()) } else { { let _ = (0 as i32); } };
vm_block = 6; continue;
}
// C line 3445
8 => {
let _ = { v = ((v) | ((k).wrapping_shl(((6 as i32)) as u32))); v };
vm_block = 7; continue;
}
// C line 3444
9 => {
let _ = if ((((!(((((v) < ((64 as i32))) as i32)) != 0) as i32)) as i64)) != 0 { tool_assert_rtn(c"get_decomp_pos".as_ptr(), c"unicode_gen.c".as_ptr(), (3444 as i32), c"v < 64".as_ptr()) } else { { let _ = (0 as i32); } };
vm_block = 8; continue;
}
// C line 3443
10 => {
let _ = { let assigned = (c).wrapping_sub((*(de)).code); v = assigned; assigned };
vm_block = 9; continue;
}
// C line 3442
11 => {
vm_block = if ((((((((c) >= ((*(de)).code)) as i32)) != 0) && (((((c) < (((*(de)).code).wrapping_add((((*(de)).len) as i32)))) as i32)) != 0)) as i32)) != 0 { 10 } else { 5 }; continue;
}
// C line 3441
12 => {
vm_block = if (((((((*(de)).len) as i32)) != ((0 as i32))) as i32)) != 0 { 11 } else { 3 }; continue;
}
// C line 3440
13 => {
let _ = { let assigned = ptr::addr_of!(*(tab_de).offset((i) as isize)); de = assigned; assigned };
vm_block = 12; continue;
}
// C line 3439
14 => {
let _ = { let assigned = (0 as i32); i = assigned; assigned };
vm_block = 2; continue;
}
// C line 3438
15 => {
let _ = { let assigned = (0 as i32); k = assigned; assigned };
vm_block = 14; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: unicode_gen.c:3456. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn build_compose_table(mut f: *mut FILE, mut tab_de: *const DecompEntry) -> () {
let mut vm_local_storage = Vec::<u64>::new();
let mut i: i32 = core::mem::zeroed();
let mut v: i32 = core::mem::zeroed();
let mut tab_ce_len: i32 = core::mem::zeroed();
let mut ce: *mut ComposeEntry = core::mem::zeroed();
let mut tab_ce: *mut ComposeEntry = core::mem::zeroed();
let mut ci: *mut CCInfo = core::mem::zeroed();
let mut vm_block: usize = 28;
loop { match vm_block {
// C line ?
0 => {
return ();
}
// C line 3502
1 => {
let _ = free(((tab_ce) as *mut c_void));
vm_block = 0; continue;
}
// C line 3500
2 => {
let _ = tool_fprintf(f, c"\n};\n\n".as_ptr(), &[]);
vm_block = 1; continue;
}
// C line 3489
3 => {
vm_block = if ((((i) < (tab_ce_len)) as i32)) != 0 { 11 } else { 2 }; continue;
}
// C line ?
4 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 3; continue;
}
// C line 3498
5 => {
let _ = tool_fprintf(f, c" 0x%04x,".as_ptr(), &[ToolPrintArg::Int(v as u64)]);
vm_block = 4; continue;
}
// C line 3496
6 => {
let _ = tool_exit((1 as i32));
vm_block = 5; continue;
}
// C line 3494
7 => {
let _ = tool_printf(c"ERROR: entry for c=%04x not found\n".as_ptr(), &[ToolPrintArg::Int((*(tab_ce).offset((i) as isize)).p as u64)]);
vm_block = 6; continue;
}
// C line 3493
8 => {
vm_block = if ((((v) < ((0 as i32))) as i32)) != 0 { 7 } else { 5 }; continue;
}
// C line 3492
9 => {
let _ = { let assigned = get_decomp_pos(tab_de, (((*(tab_ce).offset((i) as isize)).p) as i32)); v = assigned; assigned };
vm_block = 8; continue;
}
// C line 3491
10 => {
let _ = tool_fprintf(f, c"\n   ".as_ptr(), &[]);
vm_block = 9; continue;
}
// C line 3490
11 => {
vm_block = if ((((((i) % ((8 as i32)))) == ((0 as i32))) as i32)) != 0 { 10 } else { 9 }; continue;
}
// C line 3489
12 => {
let _ = { let assigned = (0 as i32); i = assigned; assigned };
vm_block = 3; continue;
}
// C line 3488
13 => {
let _ = tool_fprintf(f, c"static const uint16_t unicode_comp_table[%u] = {".as_ptr(), &[ToolPrintArg::Int(tab_ce_len as u64)]);
vm_block = 12; continue;
}
// C line 3487
14 => {
let _ = { total_table_bytes = (((((total_table_bytes) as usize)).wrapping_add((((tab_ce_len) as usize)).wrapping_mul((size_of::<u16>() as usize)))) as u32); total_table_bytes };
vm_block = 13; continue;
}
// C line 3486
15 => {
let _ = { let old = total_tables; total_tables = (total_tables).wrapping_add(1); old };
vm_block = 14; continue;
}
// C line 3474
16 => {
let _ = tool_qsort(((tab_ce) as *mut c_void), ((tab_ce_len) as usize), (size_of::<ComposeEntry>() as usize), ce_cmp);
vm_block = 15; continue;
}
// C line 3463
17 => {
vm_block = if ((((i) <= ((1114111 as i32))) as i32)) != 0 { 25 } else { 16 }; continue;
}
// C line ?
18 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 17; continue;
}
// C line 3471
19 => {
let _ = { let assigned = ((i) as u32); (*(ce)).p = assigned; assigned };
vm_block = 18; continue;
}
// C line 3470
20 => {
let _ = { let assigned = ((*((*(ci)).decomp_data).offset(((1 as i32)) as isize)) as u32); *(((*(ce)).c).as_mut_ptr()).offset(((1 as i32)) as isize) = assigned; assigned };
vm_block = 19; continue;
}
// C line 3469
21 => {
let _ = { let assigned = ((*((*(ci)).decomp_data).offset(((0 as i32)) as isize)) as u32); *(((*(ce)).c).as_mut_ptr()).offset(((0 as i32)) as isize) = assigned; assigned };
vm_block = 20; continue;
}
// C line 3468
22 => {
let _ = { let assigned = ptr::addr_of_mut!(*(tab_ce).offset(({ let old = tab_ce_len; tab_ce_len = (tab_ce_len).wrapping_add(1); old }) as isize)); ce = assigned; assigned };
vm_block = 21; continue;
}
// C line 3467
23 => {
let _ = if ((((!(((((tab_ce_len) < ((10000 as i32))) as i32)) != 0) as i32)) as i64)) != 0 { tool_assert_rtn(c"build_compose_table".as_ptr(), c"unicode_gen.c".as_ptr(), (3467 as i32), c"tab_ce_len < COMPOSE_LEN_MAX".as_ptr()) } else { { let _ = (0 as i32); } };
vm_block = 22; continue;
}
// C line 3465
24 => {
vm_block = if (((((((((((((*(ci)).decomp_len) == ((2 as i32))) as i32)) != 0) && (((!(((*(ci)).is_compat()) != 0) as i32)) != 0)) as i32)) != 0) && (((!(((*(ci)).is_excluded()) != 0) as i32)) != 0)) as i32)) != 0 { 23 } else { 18 }; continue;
}
// C line 3464
25 => {
ci = ptr::addr_of_mut!(*(unicode_db).offset((i) as isize));
vm_block = 24; continue;
}
// C line 3463
26 => {
let _ = { let assigned = (0 as i32); i = assigned; assigned };
vm_block = 17; continue;
}
// C line 3462
27 => {
let _ = { let assigned = (0 as i32); tab_ce_len = assigned; assigned };
vm_block = 26; continue;
}
// C line 3461
28 => {
let _ = { let assigned = ((malloc(((size_of::<ComposeEntry>() as usize)).wrapping_mul((((10000 as i32)) as usize)))) as *mut ComposeEntry); tab_ce = assigned; assigned };
vm_block = 27; continue;
}
_ => std::process::abort(),
} }
}

#[cfg(all(feature="unicode-use-test"))]
// Generated from the official C AST; original goto CFG retained.
// c: unicode_gen.c:3506. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn check_decompose_table() -> () {
let mut vm_local_storage = Vec::<u64>::new();
let mut c: i32 = core::mem::zeroed();
let mut ci: *mut CCInfo = core::mem::zeroed();
let mut res: [i32; 18] = core::mem::zeroed();
let mut v_ref: *mut i32 = core::mem::zeroed();
let mut len: i32 = core::mem::zeroed();
let mut ref_len: i32 = core::mem::zeroed();
let mut is_compat: i32 = core::mem::zeroed();
let mut vm_block: usize = 17;
loop { match vm_block {
// C line ?
0 => {
return ();
}
// C line 3513
1 => {
vm_block = if ((((is_compat) <= ((1 as i32))) as i32)) != 0 { 16 } else { 0 }; continue;
}
// C line ?
2 => {
let _ = { let old = is_compat; is_compat = (is_compat).wrapping_add(1); old };
vm_block = 1; continue;
}
// C line 3514
3 => {
vm_block = if ((((c) < ((1114111 as i32))) as i32)) != 0 { 15 } else { 2 }; continue;
}
// C line ?
4 => {
let _ = { let old = c; c = (c).wrapping_add(1); old };
vm_block = 3; continue;
}
// C line 3527
5 => {
let _ = tool_exit((1 as i32));
vm_block = 4; continue;
}
// C line 3526
6 => {
let _ = dump_str(c"ref".as_ptr(), v_ref, ref_len);
vm_block = 5; continue;
}
// C line 3525
7 => {
let _ = dump_str(c"res".as_ptr(), (res).as_mut_ptr(), len);
vm_block = 6; continue;
}
// C line 3524
8 => {
let _ = tool_printf(c"ERROR c=%05x compat=%d\n".as_ptr(), &[ToolPrintArg::Int(c as u64), ToolPrintArg::Int(is_compat as u64)]);
vm_block = 7; continue;
}
// C line 3522
9 => {
vm_block = if ((((((((len) != (ref_len)) as i32)) != 0) || (((((tabcmp((res).as_mut_ptr(), v_ref, ref_len)) != ((0 as i32))) as i32)) != 0)) as i32)) != 0 { 8 } else { 4 }; continue;
}
// C line 3521
10 => {
let _ = { let assigned = tool_unicode_decomp_char((((res).as_mut_ptr()) as *mut u32), ((c) as u32), is_compat); len = assigned; assigned };
vm_block = 9; continue;
}
// C line 3519
11 => {
let _ = { let assigned = (0 as i32); ref_len = assigned; assigned };
vm_block = 10; continue;
}
// C line 3518
12 => {
vm_block = if ((((((!((is_compat) != 0) as i32)) != 0) && (((((*(ci)).is_compat()) as i32)) != 0)) as i32)) != 0 { 11 } else { 10 }; continue;
}
// C line 3517
13 => {
let _ = { let assigned = (*(ci)).decomp_data; v_ref = assigned; assigned };
vm_block = 12; continue;
}
// C line 3516
14 => {
let _ = { let assigned = (*(ci)).decomp_len; ref_len = assigned; assigned };
vm_block = 13; continue;
}
// C line 3515
15 => {
let _ = { let assigned = ptr::addr_of_mut!(*(unicode_db).offset((c) as isize)); ci = assigned; assigned };
vm_block = 14; continue;
}
// C line 3514
16 => {
let _ = { let assigned = (0 as i32); c = assigned; assigned };
vm_block = 3; continue;
}
// C line 3513
17 => {
let _ = { let assigned = (0 as i32); is_compat = assigned; assigned };
vm_block = 1; continue;
}
_ => std::process::abort(),
} }
}

#[cfg(all(feature="unicode-use-test"))]
// Generated from the official C AST; original goto CFG retained.
// c: unicode_gen.c:3533. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn check_compose_table() -> () {
let mut vm_local_storage = Vec::<u64>::new();
let mut i: i32 = core::mem::zeroed();
let mut p: i32 = core::mem::zeroed();
let mut ci: *mut CCInfo = core::mem::zeroed();
let mut vm_block: usize = 9;
loop { match vm_block {
// C line ?
0 => {
return ();
}
// C line 3538
1 => {
vm_block = if ((((i) <= ((1114111 as i32))) as i32)) != 0 { 8 } else { 0 }; continue;
}
// C line ?
2 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 1; continue;
}
// C line 3546
3 => {
let _ = tool_exit((1 as i32));
vm_block = 2; continue;
}
// C line 3544
4 => {
let _ = tool_printf(c"ERROR compose: c=%05x %05x -> %05x ref=%05x\n".as_ptr(), &[ToolPrintArg::Int(*((*(ci)).decomp_data).offset(((0 as i32)) as isize) as u64), ToolPrintArg::Int(*((*(ci)).decomp_data).offset(((1 as i32)) as isize) as u64), ToolPrintArg::Int(p as u64), ToolPrintArg::Int(i as u64)]);
vm_block = 3; continue;
}
// C line 3543
5 => {
vm_block = if ((((p) != (i)) as i32)) != 0 { 4 } else { 2 }; continue;
}
// C line 3542
6 => {
let _ = { let assigned = tool_unicode_compose_pair(((*((*(ci)).decomp_data).offset(((0 as i32)) as isize)) as u32), ((*((*(ci)).decomp_data).offset(((1 as i32)) as isize)) as u32)); p = assigned; assigned };
vm_block = 5; continue;
}
// C line 3540
7 => {
vm_block = if (((((((((((((*(ci)).decomp_len) == ((2 as i32))) as i32)) != 0) && (((!(((*(ci)).is_compat()) != 0) as i32)) != 0)) as i32)) != 0) && (((!(((*(ci)).is_excluded()) != 0) as i32)) != 0)) as i32)) != 0 { 6 } else { 2 }; continue;
}
// C line 3539
8 => {
ci = ptr::addr_of_mut!(*(unicode_db).offset((i) as isize));
vm_block = 7; continue;
}
// C line 3538
9 => {
let _ = { let assigned = (0 as i32); i = assigned; assigned };
vm_block = 1; continue;
}
_ => std::process::abort(),
} }
}

#[cfg(all(feature="unicode-use-test"))]
// Generated from the official C AST; original goto CFG retained.
// c: unicode_gen.c:3561. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn check_str(mut msg: *const c_char, mut num: i32, mut in_buf: *const i32, mut in_len: i32, mut buf1: *const i32, mut len1: i32, mut buf2: *const i32, mut len2: i32) -> () {
let mut vm_local_storage = Vec::<u64>::new();
let mut vm_block: usize = 6;
loop { match vm_block {
// C line ?
0 => {
return ();
}
// C line 3570
1 => {
let _ = tool_exit((1 as i32));
vm_block = 0; continue;
}
// C line 3569
2 => {
let _ = dump_str(c"ref".as_ptr(), buf2, len2);
vm_block = 1; continue;
}
// C line 3568
3 => {
let _ = dump_str(c"res".as_ptr(), buf1, len1);
vm_block = 2; continue;
}
// C line 3567
4 => {
let _ = dump_str(c" in".as_ptr(), in_buf, in_len);
vm_block = 3; continue;
}
// C line 3566
5 => {
let _ = tool_printf(c"%d: ERROR %s:\n".as_ptr(), &[ToolPrintArg::Int(num as u64), ToolPrintArg::Str(msg as *const c_char)]);
vm_block = 4; continue;
}
// C line 3565
6 => {
vm_block = if ((((((((len1) != (len2)) as i32)) != 0) || (((((tabcmp(buf1, buf2, len1)) != ((0 as i32))) as i32)) != 0)) as i32)) != 0 { 5 } else { 0 }; continue;
}
_ => std::process::abort(),
} }
}

#[cfg(all(feature="unicode-use-test", not(feature="unicode-profile")))]
// Generated from the official C AST; original goto CFG retained.
// c: unicode_gen.c:3574. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn check_cc_table() -> () {
let mut vm_local_storage = Vec::<u64>::new();
let mut cc: i32 = core::mem::zeroed();
let mut cc_ref: i32 = core::mem::zeroed();
let mut c: i32 = core::mem::zeroed();
let mut vm_block: usize = 8;
loop { match vm_block {
// C line ?
0 => {
return ();
}
// C line 3578
1 => {
vm_block = if ((((c) <= ((1114111 as i32))) as i32)) != 0 { 7 } else { 0 }; continue;
}
// C line ?
2 => {
let _ = { let old = c; c = (c).wrapping_add(1); old };
vm_block = 1; continue;
}
// C line 3584
3 => {
let _ = tool_exit((1 as i32));
vm_block = 2; continue;
}
// C line 3582
4 => {
let _ = tool_printf(c"ERROR: c=%04x cc=%d cc_ref=%d\n".as_ptr(), &[ToolPrintArg::Int(c as u64), ToolPrintArg::Int(cc as u64), ToolPrintArg::Int(cc_ref as u64)]);
vm_block = 3; continue;
}
// C line 3581
5 => {
vm_block = if ((((cc) != (cc_ref)) as i32)) != 0 { 4 } else { 2 }; continue;
}
// C line 3580
6 => {
let _ = { let assigned = tool_unicode_get_cc(((c) as u32)); cc = assigned; assigned };
vm_block = 5; continue;
}
// C line 3579
7 => {
let _ = { let assigned = (((*(unicode_db).offset((c) as isize)).combining_class) as i32); cc_ref = assigned; assigned };
vm_block = 6; continue;
}
// C line 3578
8 => {
let _ = { let assigned = (0 as i32); c = assigned; assigned };
vm_block = 1; continue;
}
_ => std::process::abort(),
} }
}

#[cfg(all(feature="unicode-use-test", feature="unicode-profile"))]
// Generated from the official C AST; original goto CFG retained.
// c: unicode_gen.c:3574. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn check_cc_table() -> () {
let mut vm_local_storage = Vec::<u64>::new();
let mut cc: i32 = core::mem::zeroed();
let mut cc_ref: i32 = core::mem::zeroed();
let mut c: i32 = core::mem::zeroed();
let mut ti: i64 = core::mem::zeroed();
let mut count: i64 = core::mem::zeroed();
let mut vm_block: usize = 18;
loop { match vm_block {
// C line ?
0 => {
return ();
}
// C line 3600
1 => {
let _ = tool_printf(c"cc time=%0.1f ns/char\n".as_ptr(), &[ToolPrintArg::Float(((((ti) as f64)) / (((count) as f64))) as f64)]);
vm_block = 0; continue;
}
// C line 3599
2 => {
let _ = { let assigned = (get_time_ns()).wrapping_sub(ti); ti = assigned; assigned };
vm_block = 1; continue;
}
// C line 3594
3 => {
vm_block = if ((((c) <= ((65535 as i32))) as i32)) != 0 { 7 } else { 2 }; continue;
}
// C line ?
4 => {
let _ = { let old = c; c = (c).wrapping_add(1); old };
vm_block = 3; continue;
}
// C line 3597
5 => {
let _ = { let old = count; count = (count).wrapping_add(1); old };
vm_block = 4; continue;
}
// C line 3596
6 => {
let _ = { let assigned = tool_unicode_get_cc(((c) as u32)); cc = assigned; assigned };
vm_block = 5; continue;
}
// C line 3595
7 => {
let _ = { let assigned = (((*(unicode_db).offset((c) as isize)).combining_class) as i32); cc_ref = assigned; assigned };
vm_block = 6; continue;
}
// C line 3594
8 => {
let _ = { let assigned = (32 as i32); c = assigned; assigned };
vm_block = 3; continue;
}
// C line 3592
9 => {
let _ = { let assigned = (((0 as i32)) as i64); count = assigned; assigned };
vm_block = 8; continue;
}
// C line 3591
10 => {
let _ = { let assigned = get_time_ns(); ti = assigned; assigned };
vm_block = 9; continue;
}
// C line 3578
11 => {
vm_block = if ((((c) <= ((1114111 as i32))) as i32)) != 0 { 17 } else { 10 }; continue;
}
// C line ?
12 => {
let _ = { let old = c; c = (c).wrapping_add(1); old };
vm_block = 11; continue;
}
// C line 3584
13 => {
let _ = tool_exit((1 as i32));
vm_block = 12; continue;
}
// C line 3582
14 => {
let _ = tool_printf(c"ERROR: c=%04x cc=%d cc_ref=%d\n".as_ptr(), &[ToolPrintArg::Int(c as u64), ToolPrintArg::Int(cc as u64), ToolPrintArg::Int(cc_ref as u64)]);
vm_block = 13; continue;
}
// C line 3581
15 => {
vm_block = if ((((cc) != (cc_ref)) as i32)) != 0 { 14 } else { 12 }; continue;
}
// C line 3580
16 => {
let _ = { let assigned = tool_unicode_get_cc(((c) as u32)); cc = assigned; assigned };
vm_block = 15; continue;
}
// C line 3579
17 => {
let _ = { let assigned = (((*(unicode_db).offset((c) as isize)).combining_class) as i32); cc_ref = assigned; assigned };
vm_block = 16; continue;
}
// C line 3578
18 => {
let _ = { let assigned = (0 as i32); c = assigned; assigned };
vm_block = 11; continue;
}
_ => std::process::abort(),
} }
}

#[cfg(all(feature="unicode-use-test"))]
// Generated from the official C AST; original goto CFG retained.
// c: unicode_gen.c:3606. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn normalization_test(mut filename: *const c_char) -> () {
let mut vm_local_storage = Vec::<u64>::new();
let mut f: *mut FILE = core::mem::zeroed();
let mut line: [c_char; 4096] = core::mem::zeroed();
let mut p: *mut c_char = core::mem::zeroed();
let mut in_str: *mut i32 = core::mem::zeroed();
let mut nfc_str: *mut i32 = core::mem::zeroed();
let mut nfd_str: *mut i32 = core::mem::zeroed();
let mut nfkc_str: *mut i32 = core::mem::zeroed();
let mut nfkd_str: *mut i32 = core::mem::zeroed();
let mut in_len: i32 = core::mem::zeroed();
let mut nfc_len: i32 = core::mem::zeroed();
let mut nfd_len: i32 = core::mem::zeroed();
let mut nfkc_len: i32 = core::mem::zeroed();
let mut nfkd_len: i32 = core::mem::zeroed();
let mut buf: *mut i32 = core::mem::zeroed();
let mut buf_len: i32 = core::mem::zeroed();
let mut pos: i32 = core::mem::zeroed();
let mut vm_block: usize = 37;
loop { match vm_block {
// C line ?
0 => {
return ();
}
// C line 3659
1 => {
let _ = fclose(f);
vm_block = 0; continue;
}
// C line 3620
2 => {
vm_block = 32; continue;
}
// C line 3657
3 => {
let _ = free(((nfkd_str) as *mut c_void));
vm_block = 2; continue;
}
// C line 3656
4 => {
let _ = free(((nfkc_str) as *mut c_void));
vm_block = 3; continue;
}
// C line 3655
5 => {
let _ = free(((nfd_str) as *mut c_void));
vm_block = 4; continue;
}
// C line 3654
6 => {
let _ = free(((nfc_str) as *mut c_void));
vm_block = 5; continue;
}
// C line 3653
7 => {
let _ = free(((in_str) as *mut c_void));
vm_block = 6; continue;
}
// C line 3651
8 => {
let _ = free(((buf) as *mut c_void));
vm_block = 7; continue;
}
// C line 3650
9 => {
let _ = check_str(c"nfkc".as_ptr(), pos, in_str, in_len, buf, buf_len, nfkc_str, nfkc_len);
vm_block = 8; continue;
}
// C line 3649
10 => {
let _ = { let assigned = tool_unicode_normalize(((ptr::addr_of_mut!(buf)) as *mut *mut u32), ((in_str) as *mut u32), in_len, (((UNICODE_NFKC as i32)) as UnicodeNormalizationEnum), core::ptr::null_mut::<c_void>(), None); buf_len = assigned; assigned };
vm_block = 9; continue;
}
// C line 3647
11 => {
let _ = free(((buf) as *mut c_void));
vm_block = 10; continue;
}
// C line 3646
12 => {
let _ = check_str(c"nfc".as_ptr(), pos, in_str, in_len, buf, buf_len, nfc_str, nfc_len);
vm_block = 11; continue;
}
// C line 3645
13 => {
let _ = { let assigned = tool_unicode_normalize(((ptr::addr_of_mut!(buf)) as *mut *mut u32), ((in_str) as *mut u32), in_len, (((UNICODE_NFC as i32)) as UnicodeNormalizationEnum), core::ptr::null_mut::<c_void>(), None); buf_len = assigned; assigned };
vm_block = 12; continue;
}
// C line 3643
14 => {
let _ = free(((buf) as *mut c_void));
vm_block = 13; continue;
}
// C line 3642
15 => {
let _ = check_str(c"nfkd".as_ptr(), pos, in_str, in_len, buf, buf_len, nfkd_str, nfkd_len);
vm_block = 14; continue;
}
// C line 3641
16 => {
let _ = { let assigned = tool_unicode_normalize(((ptr::addr_of_mut!(buf)) as *mut *mut u32), ((in_str) as *mut u32), in_len, (((UNICODE_NFKD as i32)) as UnicodeNormalizationEnum), core::ptr::null_mut::<c_void>(), None); buf_len = assigned; assigned };
vm_block = 15; continue;
}
// C line 3639
17 => {
let _ = free(((buf) as *mut c_void));
vm_block = 16; continue;
}
// C line 3638
18 => {
let _ = check_str(c"nfd".as_ptr(), pos, in_str, in_len, buf, buf_len, nfd_str, nfd_len);
vm_block = 17; continue;
}
// C line 3637
19 => {
let _ = { let assigned = tool_unicode_normalize(((ptr::addr_of_mut!(buf)) as *mut *mut u32), ((in_str) as *mut u32), in_len, (((UNICODE_NFD as i32)) as UnicodeNormalizationEnum), core::ptr::null_mut::<c_void>(), None); buf_len = assigned; assigned };
vm_block = 18; continue;
}
// C line 3633
20 => {
let _ = { let assigned = get_field_str(ptr::addr_of_mut!(nfkd_len), p, (4 as i32)); nfkd_str = assigned; assigned };
vm_block = 19; continue;
}
// C line 3632
21 => {
let _ = { let assigned = get_field_str(ptr::addr_of_mut!(nfkc_len), p, (3 as i32)); nfkc_str = assigned; assigned };
vm_block = 20; continue;
}
// C line 3631
22 => {
let _ = { let assigned = get_field_str(ptr::addr_of_mut!(nfd_len), p, (2 as i32)); nfd_str = assigned; assigned };
vm_block = 21; continue;
}
// C line 3630
23 => {
let _ = { let assigned = get_field_str(ptr::addr_of_mut!(nfc_len), p, (1 as i32)); nfc_str = assigned; assigned };
vm_block = 22; continue;
}
// C line 3629
24 => {
let _ = { let assigned = get_field_str(ptr::addr_of_mut!(in_len), p, (0 as i32)); in_str = assigned; assigned };
vm_block = 23; continue;
}
// C line 3628
25 => {
vm_block = 2; continue;
}
// C line 3627
26 => {
vm_block = if ((((((((((*(p)) as i32)) == ((35 as i32))) as i32)) != 0) || (((((((*(p)) as i32)) == ((64 as i32))) as i32)) != 0)) as i32)) != 0 { 25 } else { 24 }; continue;
}
// C line 3625
27 => {
vm_block = if (isspace(((*(p)) as i32))) != 0 { 28 } else { 26 }; continue;
}
// C line 3626
28 => {
let _ = { let old = p; p = (p).offset(1); old };
vm_block = 27; continue;
}
// C line 3624
29 => {
let _ = { let assigned = (line).as_mut_ptr(); p = assigned; assigned };
vm_block = 27; continue;
}
// C line 3623
30 => {
let _ = { let old = pos; pos = (pos).wrapping_add(1); old };
vm_block = 29; continue;
}
// C line 3622
31 => {
vm_block = 1; continue;
}
// C line 3621
32 => {
vm_block = if ((!(!(get_line((line).as_mut_ptr(), (((size_of::<[c_char; 4096]>() as usize)) as i32), f)).is_null()) as i32)) != 0 { 31 } else { 30 }; continue;
}
// C line 3619
33 => {
let _ = { let assigned = (0 as i32); pos = assigned; assigned };
vm_block = 2; continue;
}
// C line 3617
34 => {
let _ = tool_exit((1 as i32));
vm_block = 33; continue;
}
// C line 3616
35 => {
let _ = perror(filename);
vm_block = 34; continue;
}
// C line 3615
36 => {
vm_block = if ((!(!(f).is_null()) as i32)) != 0 { 35 } else { 33 }; continue;
}
// C line 3614
37 => {
let _ = { let assigned = fopen(filename, c"rb".as_ptr()); f = assigned; assigned };
vm_block = 36; continue;
}
_ => std::process::abort(),
} }
}
