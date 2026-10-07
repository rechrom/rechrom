// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:13692. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_putc(mut s: *mut JSPrintValueState, mut c: c_char) -> () {
let mut vm_local_storage = Vec::<u64>::new();
let mut vm_block: usize = 1;
loop { match vm_block {
// C line ?
0 => {
return ();
}
// C line 13694
1 => {
let _ = ((*(s)).write_func).expect("registered parser callback")((*(s)).write_opaque, core::ptr::addr_of_mut!(c), (((1 as i32)) as usize));
vm_block = 0; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:13697. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_puts(mut s: *mut JSPrintValueState, mut str: *const c_char) -> () {
let mut vm_local_storage = Vec::<u64>::new();
let mut vm_block: usize = 1;
loop { match vm_block {
// C line ?
0 => {
return ();
}
// C line 13699
1 => {
let _ = ((*(s)).write_func).expect("registered parser callback")((*(s)).write_opaque, str, parser_strlen(str));
vm_block = 0; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:13713. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_print_float64(mut s: *mut JSPrintValueState, mut d: f64) -> () {
let mut vm_local_storage = Vec::<u64>::new();
let mut dtoa_mem: crate::dtoa::JSDTOATempMem = core::mem::zeroed();
let mut buf: [c_char; 32] = core::mem::zeroed();
let mut len: i32 = core::mem::zeroed();
let mut vm_block: usize = 2;
loop { match vm_block {
// C line ?
0 => {
return ();
}
// C line 13719
1 => {
let _ = ((*(s)).write_func).expect("registered parser callback")((*(s)).write_opaque, (buf).as_mut_ptr(), ((len) as usize));
vm_block = 0; continue;
}
// C line 13718
2 => {
let _ = { let assigned = crate::dtoa::js_dtoa((buf).as_mut_ptr(), d, (10 as i32), (0 as i32), ((((0 as i32)).wrapping_shl(((0 as i32)) as u32)) | (((1 as i32)).wrapping_shl(((4 as i32)) as u32))), core::ptr::addr_of_mut!(dtoa_mem)); len = assigned; assigned };
vm_block = 1; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:13722. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_string_get_length(mut val: JSValue) -> u32 {
let mut vm_local_storage = Vec::<u64>::new();
let mut p: *mut JSString = core::mem::zeroed();
let mut r: *mut JSStringRope = core::mem::zeroed();
let mut vm_block: usize = 7;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 13726
1 => {
return (*(p)).len();
}
// C line 13725
2 => {
p = ((((val).u).ptr) as *mut JSString);
vm_block = 1; continue;
}
// C line 13729
3 => {
return (*(r)).len;
}
// C line 13728
4 => {
r = ((((val).u).ptr) as *mut JSStringRope);
vm_block = 3; continue;
}
// C line 13731
5 => {
return (((0 as i32)) as u32);
}
// C line 13727
6 => {
vm_block = if (((((((val).tag) as i32)) == ((JS_TAG_STRING_ROPE as i32))) as i32)) != 0 { 4 } else { 5 }; continue;
}
// C line 13724
7 => {
vm_block = if (((((((val).tag) as i32)) == ((JS_TAG_STRING as i32))) as i32)) != 0 { 2 } else { 6 }; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:13736. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_print_string1(mut s: *mut JSPrintValueState, mut p: *mut JSString, mut len: i32, mut sep: i32) -> () {
let mut vm_local_storage = Vec::<u64>::new();
let mut buf: [u8; 6] = core::mem::zeroed();
let mut l: i32 = core::mem::zeroed();
let mut i: i32 = core::mem::zeroed();
let mut c: i32 = core::mem::zeroed();
let mut c1: i32 = core::mem::zeroed();
let mut vm_block: usize = 37;
loop { match vm_block {
// C line ?
0 => {
return ();
}
// C line 13741
1 => {
vm_block = if ((((i) < (len)) as i32)) != 0 { 36 } else { 0 }; continue;
}
// C line 13741
2 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 1; continue;
}
// C line 13788
3 => {
vm_block = 2; continue;
}
// C line 13768
4 => {
let _ = js_putc(s, ((c) as c_char));
vm_block = 3; continue;
}
// C line 13772 labels: escape
5 => {
let _ = js_printf(s, &quickjs_format_print(c"\\u%04x".as_ptr(), &[QuickJSPrintArg::Int(c as u64)]));
vm_block = 3; continue;
}
// C line 13786
6 => {
let _ = ((*(s)).write_func).expect("registered parser callback")((*(s)).write_opaque, (((buf).as_mut_ptr()) as *mut c_char), ((l) as usize));
vm_block = 3; continue;
}
// C line 13785
7 => {
let _ = { let assigned = unicode_to_utf8((buf).as_mut_ptr(), ((c) as u32)); l = assigned; assigned };
vm_block = 6; continue;
}
// C line 13781
8 => {
let _ = { let assigned = ((crate::cutils_header::from_surrogate(((c) as u32), ((c1) as u32))) as i32); c = assigned; assigned };
vm_block = 7; continue;
}
// C line 13780
9 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 8; continue;
}
// C line 13779
10 => {
vm_block = 5; continue;
}
// C line 13778
11 => {
vm_block = if ((!((crate::cutils_header::is_lo_surrogate(((c1) as u32))) != 0) as i32)) != 0 { 10 } else { 9 }; continue;
}
// C line 13777
12 => {
let _ = { let assigned = string_get(p, (i).wrapping_add((1 as i32))); c1 = assigned; assigned };
vm_block = 11; continue;
}
// C line 13776
13 => {
vm_block = 5; continue;
}
// C line 13775
14 => {
vm_block = if (((((i).wrapping_add((1 as i32))) >= (len)) as i32)) != 0 { 13 } else { 12 }; continue;
}
// C line 13783
15 => {
vm_block = 5; continue;
}
// C line 13782
16 => {
vm_block = if (crate::cutils_header::is_lo_surrogate(((c) as u32))) != 0 { 15 } else { 7 }; continue;
}
// C line 13774
17 => {
vm_block = if (crate::cutils_header::is_hi_surrogate(((c) as u32))) != 0 { 14 } else { 16 }; continue;
}
// C line 13769
18 => {
vm_block = if ((((((((c) < ((32 as i32))) as i32)) != 0) || (((((((((c) >= ((127 as i32))) as i32)) != 0) && (((((c) <= ((159 as i32))) as i32)) != 0)) as i32)) != 0)) as i32)) != 0 { 5 } else { 17 }; continue;
}
// C line 13767
19 => {
vm_block = if ((((((((c) >= ((32 as i32))) as i32)) != 0) && (((((c) <= ((126 as i32))) as i32)) != 0)) as i32)) != 0 { 4 } else { 18 }; continue;
}
// C line 13766
20 => {
vm_block = 24; continue;
}
// C line 13765
21 => {
vm_block = if ((((c) == (sep)) as i32)) != 0 { 20 } else { 19 }; continue;
}
// C line 13763
22 => {
vm_block = 2; continue;
}
// C line 13762
23 => {
let _ = js_putc(s, ((c) as c_char));
vm_block = 22; continue;
}
// C line 13761 labels: quote
24 => {
let _ = js_putc(s, (((92 as i32)) as c_char));
vm_block = 23; continue;
}
// C line 13758
25 => {
vm_block = 24; continue;
}
// C line 13757
26 => {
let _ = { let assigned = (102 as i32); c = assigned; assigned };
vm_block = 25; continue;
}
// C line 13755
27 => {
vm_block = 24; continue;
}
// C line 13754
28 => {
let _ = { let assigned = (98 as i32); c = assigned; assigned };
vm_block = 27; continue;
}
// C line 13752
29 => {
vm_block = 24; continue;
}
// C line 13751
30 => {
let _ = { let assigned = (110 as i32); c = assigned; assigned };
vm_block = 29; continue;
}
// C line 13749
31 => {
vm_block = 24; continue;
}
// C line 13748
32 => {
let _ = { let assigned = (114 as i32); c = assigned; assigned };
vm_block = 31; continue;
}
// C line 13746
33 => {
vm_block = 24; continue;
}
// C line 13745
34 => {
let _ = { let assigned = (116 as i32); c = assigned; assigned };
vm_block = 33; continue;
}
// C line 13743
35 => {
vm_block = match c { x if x == (92 as i32) => 24, x if x == (12 as i32) => 26, x if x == (8 as i32) => 28, x if x == (10 as i32) => 30, x if x == (13 as i32) => 32, x if x == (9 as i32) => 34, _ => 21, }; continue;
}
// C line 13742
36 => {
let _ = { let assigned = string_get(p, i); c = assigned; assigned };
vm_block = 35; continue;
}
// C line 13741
37 => {
let _ = { let assigned = (0 as i32); i = assigned; assigned };
vm_block = 1; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:13793. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_print_string_rec(mut s: *mut JSPrintValueState, mut val: JSValue, mut sep: i32, mut pos: u32) -> () {
let mut vm_local_storage = Vec::<u64>::new();
let mut p: *mut JSString = core::mem::zeroed();
let mut len: u32 = core::mem::zeroed();
let mut r: *mut JSStringRope = core::mem::zeroed();
let mut vm_block: usize = 10;
loop { match vm_block {
// C line ?
0 => {
return ();
}
// C line 13801
1 => {
let _ = js_print_string1(s, p, ((len) as i32), sep);
vm_block = 0; continue;
}
// C line 13800
2 => {
let _ = { let assigned = crate::cutils_header::min_uint32((*(p)).len(), (((*(s)).options).max_string_length).wrapping_sub(pos)); len = assigned; assigned };
vm_block = 1; continue;
}
// C line 13799
3 => {
vm_block = if ((((pos) < (((*(s)).options).max_string_length)) as i32)) != 0 { 2 } else { 0 }; continue;
}
// C line 13797
4 => {
p = ((((val).u).ptr) as *mut JSString);
vm_block = 3; continue;
}
// C line 13806
5 => {
let _ = js_print_string_rec(s, (*(r)).right, sep, (pos).wrapping_add(js_string_get_length((*(r)).left)));
vm_block = 0; continue;
}
// C line 13805
6 => {
let _ = js_print_string_rec(s, (*(r)).left, sep, pos);
vm_block = 5; continue;
}
// C line 13804
7 => {
r = ((((val).u).ptr) as *mut JSStringRope);
vm_block = 6; continue;
}
// C line 13808
8 => {
let _ = js_printf(s, &quickjs_format_print(c"<invalid string tag %d>".as_ptr(), &[QuickJSPrintArg::Int((((val).tag) as i32) as u64)]));
vm_block = 0; continue;
}
// C line 13803
9 => {
vm_block = if (((((((val).tag) as i32)) == ((JS_TAG_STRING_ROPE as i32))) as i32)) != 0 { 7 } else { 8 }; continue;
}
// C line 13796
10 => {
vm_block = if (((((((val).tag) as i32)) == ((JS_TAG_STRING as i32))) as i32)) != 0 { 4 } else { 9 }; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:13812. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_print_string(mut s: *mut JSPrintValueState, mut val: JSValue) -> () {
let mut vm_local_storage = Vec::<u64>::new();
let mut sep: i32 = core::mem::zeroed();
let mut p: *mut JSString = core::mem::zeroed();
let mut n: u32 = core::mem::zeroed();
let mut vm_block: usize = 11;
loop { match vm_block {
// C line ?
0 => {
return ();
}
// C line 13827
1 => {
let _ = js_printf(s, &quickjs_format_print(c"... %u more character%s".as_ptr(), &[QuickJSPrintArg::Int(n as u64), QuickJSPrintArg::Str(if ((((n) > ((((1 as i32)) as u32))) as i32)) != 0 { c"s".as_ptr() } else { c"".as_ptr() } as *const c_char)]));
vm_block = 0; continue;
}
// C line 13826
2 => {
n = (js_string_get_length(val)).wrapping_sub(((*(s)).options).max_string_length);
vm_block = 1; continue;
}
// C line 13825
3 => {
vm_block = if ((((js_string_get_length(val)) > (((*(s)).options).max_string_length)) as i32)) != 0 { 2 } else { 0 }; continue;
}
// C line 13824
4 => {
let _ = js_putc(s, ((sep) as c_char));
vm_block = 3; continue;
}
// C line 13823
5 => {
let _ = js_print_string_rec(s, val, sep, (((0 as i32)) as u32));
vm_block = 4; continue;
}
// C line 13822
6 => {
let _ = js_putc(s, ((sep) as c_char));
vm_block = 5; continue;
}
// C line 13818
7 => {
let _ = { let assigned = if (((((*(js_rc(((p) as *mut c_void)))).ref_count) == ((1 as i32))) as i32)) != 0 { (34 as i32) } else { (39 as i32) }; sep = assigned; assigned };
vm_block = 6; continue;
}
// C line 13817
8 => {
let _ = js_printf(s, &quickjs_format_print(c"%d".as_ptr(), &[QuickJSPrintArg::Int((*(js_rc(((p) as *mut c_void)))).ref_count as u64)]));
vm_block = 7; continue;
}
// C line 13816
9 => {
p = ((((val).u).ptr) as *mut JSString);
vm_block = 8; continue;
}
// C line 13820
10 => {
let _ = { let assigned = (34 as i32); sep = assigned; assigned };
vm_block = 6; continue;
}
// C line 13815
11 => {
vm_block = if (((((((*(s)).options).raw_dump()) != 0) && ((((((((val).tag) as i32)) == ((JS_TAG_STRING as i32))) as i32)) != 0)) as i32)) != 0 { 9 } else { 10 }; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:13831. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_print_raw_string(mut s: *mut JSPrintValueState, mut val: JSValue) -> () {
let mut vm_local_storage = Vec::<u64>::new();
let mut cstr: *const c_char = core::mem::zeroed();
let mut len: usize = core::mem::zeroed();
let mut vm_block: usize = 4;
loop { match vm_block {
// C line ?
0 => {
return ();
}
// C line 13838
1 => {
let _ = JS_FreeCString((*(s)).ctx, cstr);
vm_block = 0; continue;
}
// C line 13837
2 => {
let _ = ((*(s)).write_func).expect("registered parser callback")((*(s)).write_opaque, cstr, len);
vm_block = 1; continue;
}
// C line 13836
3 => {
vm_block = if !(cstr).is_null() { 2 } else { 0 }; continue;
}
// C line 13835
4 => {
let _ = { let assigned = JS_ToCStringLen((*(s)).ctx, core::ptr::addr_of_mut!(len), val); cstr = assigned; assigned };
vm_block = 3; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:13842. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn is_ascii_ident(mut p: *const JSString) -> i32 {
let mut vm_local_storage = Vec::<u64>::new();
let mut i: i32 = core::mem::zeroed();
let mut c: i32 = core::mem::zeroed();
let mut vm_block: usize = 9;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 13854
1 => {
return (1 as i32);
}
// C line 13848
2 => {
vm_block = if ((((i) < ((((*(p)).len()) as i32))) as i32)) != 0 { 6 } else { 1 }; continue;
}
// C line 13848
3 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 2; continue;
}
// C line 13852
4 => {
return (0 as i32);
}
// C line 13850
5 => {
vm_block = if ((!(((((((((((((((((((((c) >= ((97 as i32))) as i32)) != 0) && (((((c) <= ((122 as i32))) as i32)) != 0)) as i32)) != 0) || (((((((((c) >= ((65 as i32))) as i32)) != 0) && (((((c) <= ((90 as i32))) as i32)) != 0)) as i32)) != 0)) as i32)) != 0) || (((((((((c) == ((95 as i32))) as i32)) != 0) || (((((c) == ((36 as i32))) as i32)) != 0)) as i32)) != 0)) as i32)) != 0) || (((((((((((((c) >= ((48 as i32))) as i32)) != 0) && (((((c) <= ((57 as i32))) as i32)) != 0)) as i32)) != 0) && (((((i) > ((0 as i32))) as i32)) != 0)) as i32)) != 0)) as i32)) != 0) as i32)) != 0 { 4 } else { 3 }; continue;
}
// C line 13849
6 => {
let _ = { let assigned = string_get(p, i); c = assigned; assigned };
vm_block = 5; continue;
}
// C line 13848
7 => {
let _ = { let assigned = (0 as i32); i = assigned; assigned };
vm_block = 2; continue;
}
// C line 13847
8 => {
return (0 as i32);
}
// C line 13846
9 => {
vm_block = if (((((((*(p)).len()) as i32)) == ((0 as i32))) as i32)) != 0 { 8 } else { 7 }; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:13857. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_print_atom(mut s: *mut JSPrintValueState, mut atom: JSAtom) -> () {
let mut vm_local_storage = Vec::<u64>::new();
let mut i: i32 = core::mem::zeroed();
let mut p: *mut JSString = core::mem::zeroed();
let mut vm_block: usize = 14;
loop { match vm_block {
// C line ?
0 => {
return ();
}
// C line 13861
1 => {
let _ = js_printf(s, &quickjs_format_print(c"%u".as_ptr(), &[QuickJSPrintArg::Int(__JS_AtomToUInt32(atom) as u64)]));
vm_block = 0; continue;
}
// C line 13863
2 => {
let _ = js_puts(s, c"<null>".as_ptr());
vm_block = 0; continue;
}
// C line 13869
3 => {
vm_block = if ((((i) < ((((*(p)).len()) as i32))) as i32)) != 0 { 5 } else { 0 }; continue;
}
// C line 13869
4 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 3; continue;
}
// C line 13870
5 => {
let _ = js_putc(s, ((string_get(p, i)) as c_char));
vm_block = 4; continue;
}
// C line 13869
6 => {
let _ = { let assigned = (0 as i32); i = assigned; assigned };
vm_block = 3; continue;
}
// C line 13875
7 => {
let _ = js_putc(s, (((34 as i32)) as c_char));
vm_block = 0; continue;
}
// C line 13874
8 => {
let _ = js_print_string1(s, p, (((*(p)).len()) as i32), (34 as i32));
vm_block = 7; continue;
}
// C line 13873
9 => {
let _ = js_putc(s, (((34 as i32)) as c_char));
vm_block = 8; continue;
}
// C line 13868
10 => {
vm_block = if (is_ascii_ident(p)) != 0 { 6 } else { 9 }; continue;
}
// C line 13867
11 => {
let _ = { let assigned = *((*((*(s)).rt)).atom_array).offset((atom) as isize); p = assigned; assigned };
vm_block = 10; continue;
}
// C line 13865
12 => {
let _ = if ((((!(((((atom) < ((((*((*(s)).rt)).atom_size) as JSAtom))) as i32)) != 0) as i32)) as i64)) != 0 { std::process::abort() } else { { let _ = (0 as i32); } };
vm_block = 11; continue;
}
// C line 13862
13 => {
vm_block = if ((((atom) == ((((0 as i32)) as JSAtom))) as i32)) != 0 { 2 } else { 12 }; continue;
}
// C line 13860
14 => {
vm_block = if (__JS_AtomIsTaggedInt(atom)) != 0 { 1 } else { 13 }; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:13881. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_print_array_get_length(mut p: *mut JSObject) -> u32 {
let mut vm_local_storage = Vec::<u64>::new();
let mut pr: *mut JSProperty = core::mem::zeroed();
let mut prs: *mut JSShapeProperty = core::mem::zeroed();
let mut val: JSValue = core::mem::zeroed();
let mut vm_block: usize = 10;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 13899
1 => {
return (((0 as i32)) as u32);
}
// C line 13897
2 => {
return ((((val).u).float64) as u32);
}
// C line 13895
3 => {
return ((((((val).u).uint64) as i32)) as u32);
}
// C line 13893
4 => {
vm_block = match (((val).tag) as i32) { x if x == (JS_TAG_FLOAT64 as i32) => 2, x if x == (JS_TAG_INT as i32) => 3, _ => 1, }; continue;
}
// C line 13892
5 => {
let _ = { let assigned = ((*(pr)).u).value; val = assigned; assigned };
vm_block = 4; continue;
}
// C line 13891
6 => {
return (((0 as i32)) as u32);
}
// C line 13890
7 => {
vm_block = if (((((((((*(prs)).flags()) as i32)) & (((3 as i32)).wrapping_shl(((4 as i32)) as u32)))) != (((0 as i32)).wrapping_shl(((4 as i32)) as u32))) as i32)) != 0 { 6 } else { 5 }; continue;
}
// C line 13889
8 => {
return (((0 as i32)) as u32);
}
// C line 13888
9 => {
vm_block = if ((!(!(prs).is_null()) as i32)) != 0 { 8 } else { 7 }; continue;
}
// C line 13887
10 => {
let _ = { let assigned = find_own_property(core::ptr::addr_of_mut!(pr), p, (((crate::quickjs_atom::JS_ATOM_length as i32)) as JSAtom)); prs = assigned; assigned };
vm_block = 9; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:13903. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_print_comma(mut s: *mut JSPrintValueState, mut pcomma_state: *mut i32) -> () {
let mut vm_local_storage = Vec::<u64>::new();
let mut vm_block: usize = 7;
loop { match vm_block {
// C line ?
0 => {
return ();
}
// C line 13915
1 => {
let _ = { let assigned = (1 as i32); *(pcomma_state) = assigned; assigned };
vm_block = 0; continue;
}
// C line 13913
2 => {
vm_block = 1; continue;
}
// C line 13912
3 => {
let _ = js_printf(s, &quickjs_format_print(c" { ".as_ptr(), &[]));
vm_block = 2; continue;
}
// C line 13910
4 => {
vm_block = 1; continue;
}
// C line 13909
5 => {
let _ = js_printf(s, &quickjs_format_print(c", ".as_ptr(), &[]));
vm_block = 4; continue;
}
// C line 13907
6 => {
vm_block = 1; continue;
}
// C line 13905
7 => {
vm_block = match *(pcomma_state) { x if x == (2 as i32) => 3, x if x == (1 as i32) => 5, x if x == (0 as i32) => 6, _ => 1, }; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:13918. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_print_more_items(mut s: *mut JSPrintValueState, mut pcomma_state: *mut i32, mut n: u32) -> () {
let mut vm_local_storage = Vec::<u64>::new();
let mut vm_block: usize = 2;
loop { match vm_block {
// C line ?
0 => {
return ();
}
// C line 13922
1 => {
let _ = js_printf(s, &quickjs_format_print(c"... %u more item%s".as_ptr(), &[QuickJSPrintArg::Int(n as u64), QuickJSPrintArg::Str(if ((((n) > ((((1 as i32)) as u32))) as i32)) != 0 { c"s".as_ptr() } else { c"".as_ptr() } as *const c_char)]));
vm_block = 0; continue;
}
// C line 13921
2 => {
let _ = js_print_comma(s, pcomma_state);
vm_block = 1; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:13926. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_print_regexp(mut s: *mut JSPrintValueState, mut p1: *mut JSObject) -> () {
let mut vm_local_storage = Vec::<u64>::new();
let mut re: *mut JSRegExp = core::mem::zeroed();
let mut p: *mut JSString = core::mem::zeroed();
let mut i: i32 = core::mem::zeroed();
let mut n: i32 = core::mem::zeroed();
let mut c: i32 = core::mem::zeroed();
let mut c2: i32 = core::mem::zeroed();
let mut bra: i32 = core::mem::zeroed();
let mut flags: i32 = core::mem::zeroed();
let mut regexp_flags: [c_char; 8] = core::mem::zeroed();
let mut vm_block: usize = 44;
loop { match vm_block {
// C line ?
0 => {
return ();
}
// C line 13984
1 => {
vm_block = if ((((((i) as usize)) < ((((size_of::<[c_char; 8]>() as usize)) / ((size_of::<c_char>() as usize))))) as i32)) != 0 { 4 } else { 0 }; continue;
}
// C line 13984
2 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 1; continue;
}
// C line 13986
3 => {
let _ = js_putc(s, *((regexp_flags).as_ptr()).offset((i) as isize));
vm_block = 2; continue;
}
// C line 13985
4 => {
vm_block = if ((((flags).wrapping_shr((i) as u32)) & ((1 as i32)))) != 0 { 3 } else { 2 }; continue;
}
// C line 13984
5 => {
let _ = { let assigned = (0 as i32); i = assigned; assigned };
vm_block = 1; continue;
}
// C line 13983
6 => {
let _ = { let assigned = crate::libregexp::lre_get_flags((((*((*(re)).bytecode)).u).str8).as_mut_ptr()); flags = assigned; assigned };
vm_block = 5; continue;
}
// C line 13981
7 => {
let _ = js_putc(s, (((47 as i32)) as c_char));
vm_block = 6; continue;
}
// C line 13941
8 => {
let _ = js_puts(s, c"(?:)".as_ptr());
vm_block = 7; continue;
}
// C line 13944
9 => {
vm_block = if ((((i) < (n)) as i32)) != 0 { 34 } else { 7 }; continue;
}
// C line 13978
10 => {
let _ = js_putc(s, ((c2) as c_char));
vm_block = 9; continue;
}
// C line 13977
11 => {
vm_block = if ((((c2) >= ((0 as i32))) as i32)) != 0 { 10 } else { 9 }; continue;
}
// C line 13976
12 => {
let _ = js_putc(s, ((c) as c_char));
vm_block = 11; continue;
}
// C line 13974
13 => {
vm_block = 12; continue;
}
// C line 13972
14 => {
let _ = { let assigned = (47 as i32); c2 = assigned; assigned };
vm_block = 13; continue;
}
// C line 13971
15 => {
let _ = { let assigned = (92 as i32); c = assigned; assigned };
vm_block = 14; continue;
}
// C line 13970
16 => {
vm_block = if ((!((bra) != 0) as i32)) != 0 { 15 } else { 13 }; continue;
}
// C line 13968
17 => {
vm_block = 12; continue;
}
// C line 13967
18 => {
let _ = { let assigned = (114 as i32); c2 = assigned; assigned };
vm_block = 17; continue;
}
// C line 13966
19 => {
let _ = { let assigned = (92 as i32); c = assigned; assigned };
vm_block = 18; continue;
}
// C line 13964
20 => {
vm_block = 12; continue;
}
// C line 13963
21 => {
let _ = { let assigned = (110 as i32); c2 = assigned; assigned };
vm_block = 20; continue;
}
// C line 13962
22 => {
let _ = { let assigned = (92 as i32); c = assigned; assigned };
vm_block = 21; continue;
}
// C line 13960
23 => {
vm_block = 12; continue;
}
// C line 13958
24 => {
let _ = { let assigned = (1 as i32); bra = assigned; assigned };
vm_block = 23; continue;
}
// C line 13957
25 => {
let _ = { let assigned = string_get(p, { let old = i; i = (i).wrapping_add(1); old }); c2 = assigned; assigned };
vm_block = 24; continue;
}
// C line 13956
26 => {
vm_block = if ((((((((i) < (n)) as i32)) != 0) && (((((string_get(p, i)) == ((93 as i32))) as i32)) != 0)) as i32)) != 0 { 25 } else { 24 }; continue;
}
// C line 13955
27 => {
vm_block = if ((!((bra) != 0) as i32)) != 0 { 26 } else { 23 }; continue;
}
// C line 13953
28 => {
vm_block = 12; continue;
}
// C line 13952
29 => {
let _ = { let assigned = (0 as i32); bra = assigned; assigned };
vm_block = 28; continue;
}
// C line 13950
30 => {
vm_block = 12; continue;
}
// C line 13949
31 => {
let _ = { let assigned = string_get(p, { let old = i; i = (i).wrapping_add(1); old }); c2 = assigned; assigned };
vm_block = 30; continue;
}
// C line 13948
32 => {
vm_block = if ((((i) < (n)) as i32)) != 0 { 31 } else { 30 }; continue;
}
// C line 13946
33 => {
vm_block = match { let assigned = string_get(p, { let old = i; i = (i).wrapping_add(1); old }); c = assigned; assigned } { x if x == (47 as i32) => 16, x if x == (13 as i32) => 19, x if x == (10 as i32) => 22, x if x == (91 as i32) => 27, x if x == (93 as i32) => 29, x if x == (92 as i32) => 32, _ => 12, }; continue;
}
// C line 13945
34 => {
let _ = { let assigned = ((1 as i32)).wrapping_neg(); c2 = assigned; assigned };
vm_block = 33; continue;
}
// C line 13944
35 => {
let _ = { let _ = { let assigned = (0 as i32); i = assigned; assigned }; { let assigned = (((*(p)).len()) as i32); n = assigned; assigned } };
vm_block = 9; continue;
}
// C line 13943
36 => {
let _ = { let assigned = (0 as i32); bra = assigned; assigned };
vm_block = 35; continue;
}
// C line 13940
37 => {
vm_block = if (((((((*(p)).len()) as i32)) == ((0 as i32))) as i32)) != 0 { 8 } else { 36 }; continue;
}
// C line 13939
38 => {
let _ = js_putc(s, (((47 as i32)) as c_char));
vm_block = 37; continue;
}
// C line 13938
39 => {
let _ = { let assigned = (*(re)).pattern; p = assigned; assigned };
vm_block = 38; continue;
}
// C line 13936
40 => {
return;
}
// C line 13935
41 => {
let _ = js_puts(s, c"[uninitialized_regexp]".as_ptr());
vm_block = 40; continue;
}
// C line 13933
42 => {
vm_block = if ((((((!(!((*(re)).pattern).is_null()) as i32)) != 0) || (((!(!((*(re)).bytecode).is_null()) as i32)) != 0)) as i32)) != 0 { 41 } else { 39 }; continue;
}
// C line 13931
43 => {
regexp_flags = [(((103 as i32)) as c_char), (((105 as i32)) as c_char), (((109 as i32)) as c_char), (((115 as i32)) as c_char), (((117 as i32)) as c_char), (((121 as i32)) as c_char), (((100 as i32)) as c_char), (((118 as i32)) as c_char)];
vm_block = 42; continue;
}
// C line 13928
44 => {
re = core::ptr::addr_of_mut!(((*(p1)).u).regexp);
vm_block = 43; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:13992. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_print_error(mut s: *mut JSPrintValueState, mut p: *mut JSObject) -> () {
let mut vm_local_storage = Vec::<u64>::new();
let mut str: *const c_char = core::mem::zeroed();
let mut len: usize = core::mem::zeroed();
let mut vm_block: usize = 18;
loop { match vm_block {
// C line ?
0 => {
return ();
}
// C line 14024
1 => {
let _ = JS_FreeCString((*(s)).ctx, str);
vm_block = 0; continue;
}
// C line 14022
2 => {
let _ = ((*(s)).write_func).expect("registered parser callback")((*(s)).write_opaque, str, len);
vm_block = 1; continue;
}
// C line 14021
3 => {
let _ = { let old = len; len = (len).wrapping_sub(1); old };
vm_block = 2; continue;
}
// C line 14020
4 => {
vm_block = if ((((((((len) > ((((0 as i32)) as usize))) as i32)) != 0) && (((((((*(str).offset(((len).wrapping_sub((((1 as i32)) as usize))) as isize)) as i32)) == ((10 as i32))) as i32)) != 0)) as i32)) != 0 { 3 } else { 2 }; continue;
}
// C line 14019
5 => {
let _ = { let assigned = parser_strlen(str); len = assigned; assigned };
vm_block = 4; continue;
}
// C line 14015
6 => {
let _ = js_putc(s, (((10 as i32)) as c_char));
vm_block = 5; continue;
}
// C line 14014
7 => {
vm_block = if !(str).is_null() { 6 } else { 0 }; continue;
}
// C line 14013
8 => {
let _ = { let assigned = get_prop_string((*(s)).ctx, JSValue { u: JSValueUnion { ptr: ((p) as *mut c_void) }, tag: (((JS_TAG_OBJECT as i32)) as i64) }, (((crate::quickjs_atom::JS_ATOM_stack as i32)) as JSAtom)); str = assigned; assigned };
vm_block = 7; continue;
}
// C line 14010
9 => {
let _ = JS_FreeCString((*(s)).ctx, str);
vm_block = 8; continue;
}
// C line 14008
10 => {
let _ = js_puts(s, str);
vm_block = 9; continue;
}
// C line 14007
11 => {
let _ = js_puts(s, c": ".as_ptr());
vm_block = 10; continue;
}
// C line 14006
12 => {
vm_block = if ((((!(str).is_null()) && (((((((*(str).offset(((0 as i32)) as isize)) as i32)) != ((0 as i32))) as i32)) != 0)) as i32)) != 0 { 11 } else { 9 }; continue;
}
// C line 14005
13 => {
let _ = { let assigned = get_prop_string((*(s)).ctx, JSValue { u: JSValueUnion { ptr: ((p) as *mut c_void) }, tag: (((JS_TAG_OBJECT as i32)) as i64) }, (((crate::quickjs_atom::JS_ATOM_message as i32)) as JSAtom)); str = assigned; assigned };
vm_block = 12; continue;
}
// C line 13999
14 => {
let _ = js_puts(s, c"Error".as_ptr());
vm_block = 13; continue;
}
// C line 14002
15 => {
let _ = JS_FreeCString((*(s)).ctx, str);
vm_block = 13; continue;
}
// C line 14001
16 => {
let _ = js_puts(s, str);
vm_block = 15; continue;
}
// C line 13998
17 => {
vm_block = if ((!(!(str).is_null()) as i32)) != 0 { 14 } else { 16 }; continue;
}
// C line 13997
18 => {
let _ = { let assigned = get_prop_string((*(s)).ctx, JSValue { u: JSValueUnion { ptr: ((p) as *mut c_void) }, tag: (((JS_TAG_OBJECT as i32)) as i64) }, (((crate::quickjs_atom::JS_ATOM_name as i32)) as JSAtom)); str = assigned; assigned };
vm_block = 17; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:14028. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_print_object(mut s: *mut JSPrintValueState, mut p: *mut JSObject) -> () {
let mut vm_local_storage = Vec::<u64>::new();
let mut rt: *mut JSRuntime = core::mem::zeroed();
let mut sh: *mut JSShape = core::mem::zeroed();
let mut prs: *mut JSShapeProperty = core::mem::zeroed();
let mut pr: *mut JSProperty = core::mem::zeroed();
let mut comma_state: i32 = core::mem::zeroed();
let mut is_array: i32 = core::mem::zeroed();
let mut i: u32 = core::mem::zeroed();
let mut len: u32 = core::mem::zeroed();
let mut n: u32 = core::mem::zeroed();
let mut len1: u32 = core::mem::zeroed();
let mut size: u32 = core::mem::zeroed();
let mut len1_1: u32 = core::mem::zeroed();
let mut v: i64 = core::mem::zeroed();
let mut ptr: *const u8 = core::mem::zeroed();
let mut func_name_str: *const c_char = core::mem::zeroed();
let mut ms: *mut JSMapState = core::mem::zeroed();
let mut el: *mut list_head = core::mem::zeroed();
let mut mr: *mut JSMapRecord = core::mem::zeroed();
let mut str: JSValue = core::mem::zeroed();
let mut j: u32 = core::mem::zeroed();
let mut b: *mut JSFunctionBytecode = core::mem::zeroed();
let mut var_refs: *mut *mut JSVarRef = core::mem::zeroed();
let mut vm_block: usize = 163;
loop { match vm_block {
// C line ?
0 => {
return ();
}
// C line 14262
1 => {
let _ = js_printf(s, &quickjs_format_print(c" }".as_ptr(), &[]));
vm_block = 0; continue;
}
// C line 14261
2 => {
vm_block = if ((((comma_state) != ((2 as i32))) as i32)) != 0 { 1 } else { 0 }; continue;
}
// C line 14265
3 => {
let _ = js_printf(s, &quickjs_format_print(c" ]".as_ptr(), &[]));
vm_block = 0; continue;
}
// C line 14260
4 => {
vm_block = if ((!((is_array) != 0) as i32)) != 0 { 2 } else { 3 }; continue;
}
// C line 14256
5 => {
let _ = js_print_value(s, JSValue { u: JSValueUnion { ptr: (((((*(p)).u).func).home_object) as *mut c_void) }, tag: (((JS_TAG_OBJECT as i32)) as i64) });
vm_block = 4; continue;
}
// C line 14255
6 => {
let _ = js_printf(s, &quickjs_format_print(c"[[HomeObject]]: ".as_ptr(), &[]));
vm_block = 5; continue;
}
// C line 14254
7 => {
let _ = js_print_comma(s, core::ptr::addr_of_mut!(comma_state));
vm_block = 6; continue;
}
// C line 14253
8 => {
vm_block = if !((((*(p)).u).func).home_object).is_null() { 7 } else { 4 }; continue;
}
// C line 14251
9 => {
let _ = js_printf(s, &quickjs_format_print(c" ]".as_ptr(), &[]));
vm_block = 8; continue;
}
// C line 14246
10 => {
vm_block = if ((((i) < ((((*(b)).closure_var_count) as u32))) as i32)) != 0 { 14 } else { 9 }; continue;
}
// C line 14246
11 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 10; continue;
}
// C line 14249
12 => {
let _ = js_print_value(s, ((*(*(var_refs).offset((i) as isize))).u).value);
vm_block = 11; continue;
}
// C line 14248
13 => {
let _ = js_printf(s, &quickjs_format_print(c", ".as_ptr(), &[]));
vm_block = 12; continue;
}
// C line 14247
14 => {
vm_block = if ((((i) != ((((0 as i32)) as u32))) as i32)) != 0 { 13 } else { 12 }; continue;
}
// C line 14246
15 => {
let _ = { let assigned = (((0 as i32)) as u32); i = assigned; assigned };
vm_block = 10; continue;
}
// C line 14245
16 => {
let _ = js_printf(s, &quickjs_format_print(c"[[Closure]]: [".as_ptr(), &[]));
vm_block = 15; continue;
}
// C line 14244
17 => {
let _ = js_print_comma(s, core::ptr::addr_of_mut!(comma_state));
vm_block = 16; continue;
}
// C line 14242
18 => {
let _ = { let assigned = (((*(p)).u).func).var_refs; var_refs = assigned; assigned };
vm_block = 17; continue;
}
// C line 14240
19 => {
vm_block = if ((*(b)).closure_var_count) != 0 { 18 } else { 8 }; continue;
}
// C line 14239
20 => {
b = (((*(p)).u).func).function_bytecode;
vm_block = 19; continue;
}
// C line 14238
21 => {
vm_block = if (((((((*(s)).options).raw_dump()) != 0) && ((js_class_has_bytecode((((*(p)).class_id) as JSClassID))) != 0)) as i32)) != 0 { 20 } else { 4 }; continue;
}
// C line 14236
22 => {
let _ = js_print_more_items(s, core::ptr::addr_of_mut!(comma_state), (j).wrapping_sub(((*(s)).options).max_item_count));
vm_block = 21; continue;
}
// C line 14235
23 => {
vm_block = if ((((j) > (((*(s)).options).max_item_count)) as i32)) != 0 { 22 } else { 21 }; continue;
}
// C line 14185
24 => {
vm_block = if ((((i) < ((((*(sh)).prop_count) as u32))) as i32)) != 0 { 51 } else { 23 }; continue;
}
// C line 14185
25 => {
let _ = { let _ = { let old = i; i = (i).wrapping_add(1); old }; { let old = prs; prs = (prs).offset(1); old } };
vm_block = 24; continue;
}
// C line 14232
26 => {
let _ = { let old = j; j = (j).wrapping_add(1); old };
vm_block = 25; continue;
}
// C line 14200
27 => {
let _ = js_printf(s, &quickjs_format_print(c"[Getter %p Setter %p]".as_ptr(), &[QuickJSPrintArg::Ptr((((*(pr)).u).getset).getter as *const c_void), QuickJSPrintArg::Ptr((((*(pr)).u).getset).setter as *const c_void)]));
vm_block = 26; continue;
}
// C line 14204
28 => {
let _ = js_printf(s, &quickjs_format_print(c"[Getter/Setter]".as_ptr(), &[]));
vm_block = 26; continue;
}
// C line 14206
29 => {
let _ = js_printf(s, &quickjs_format_print(c"[Setter]".as_ptr(), &[]));
vm_block = 26; continue;
}
// C line 14208
30 => {
let _ = js_printf(s, &quickjs_format_print(c"[Getter]".as_ptr(), &[]));
vm_block = 26; continue;
}
// C line 14205
31 => {
vm_block = if !((((*(pr)).u).getset).setter).is_null() { 29 } else { 30 }; continue;
}
// C line 14203
32 => {
vm_block = if ((((!((((*(pr)).u).getset).getter).is_null()) && (!((((*(pr)).u).getset).setter).is_null())) as i32)) != 0 { 28 } else { 31 }; continue;
}
// C line 14199
33 => {
vm_block = if (((*(s)).options).raw_dump()) != 0 { 27 } else { 32 }; continue;
}
// C line 14213
34 => {
let _ = js_printf(s, &quickjs_format_print(c"[varref %p]".as_ptr(), &[QuickJSPrintArg::Ptr(((((*(pr)).u).var_ref) as *mut c_void) as *const c_void)]));
vm_block = 26; continue;
}
// C line 14215
35 => {
let _ = js_print_value(s, *((*(((*(pr)).u).var_ref)).pvalue));
vm_block = 26; continue;
}
// C line 14212
36 => {
vm_block = if (((*(s)).options).raw_dump()) != 0 { 34 } else { 35 }; continue;
}
// C line 14219
37 => {
let _ = js_printf(s, &quickjs_format_print(c"[autoinit %p %d %p]".as_ptr(), &[QuickJSPrintArg::Ptr(((js_autoinit_get_realm(pr)) as *mut c_void) as *const c_void), QuickJSPrintArg::Int(((js_autoinit_get_id(pr)) as u32) as u64), QuickJSPrintArg::Ptr((((*(pr)).u).init).opaque as *const c_void)]));
vm_block = 26; continue;
}
// C line 14226
38 => {
let _ = js_printf(s, &quickjs_format_print(c"[autoinit]".as_ptr(), &[]));
vm_block = 26; continue;
}
// C line 14218
39 => {
vm_block = if (((*(s)).options).raw_dump()) != 0 { 37 } else { 38 }; continue;
}
// C line 14229
40 => {
let _ = js_print_value(s, ((*(pr)).u).value);
vm_block = 26; continue;
}
// C line 14217
41 => {
vm_block = if (((((((((*(prs)).flags()) as i32)) & (((3 as i32)).wrapping_shl(((4 as i32)) as u32)))) == (((3 as i32)).wrapping_shl(((4 as i32)) as u32))) as i32)) != 0 { 39 } else { 40 }; continue;
}
// C line 14211
42 => {
vm_block = if (((((((((*(prs)).flags()) as i32)) & (((3 as i32)).wrapping_shl(((4 as i32)) as u32)))) == (((2 as i32)).wrapping_shl(((4 as i32)) as u32))) as i32)) != 0 { 36 } else { 41 }; continue;
}
// C line 14198
43 => {
vm_block = if (((((((((*(prs)).flags()) as i32)) & (((3 as i32)).wrapping_shl(((4 as i32)) as u32)))) == (((1 as i32)).wrapping_shl(((4 as i32)) as u32))) as i32)) != 0 { 33 } else { 42 }; continue;
}
// C line 14195
44 => {
let _ = js_printf(s, &quickjs_format_print(c": ".as_ptr(), &[]));
vm_block = 43; continue;
}
// C line 14194
45 => {
let _ = js_print_atom(s, (*(prs)).atom);
vm_block = 44; continue;
}
// C line 14193
46 => {
let _ = js_print_comma(s, core::ptr::addr_of_mut!(comma_state));
vm_block = 45; continue;
}
// C line 14192
47 => {
let _ = { let assigned = core::ptr::addr_of_mut!(*((*(p)).prop).offset((i) as isize)); pr = assigned; assigned };
vm_block = 46; continue;
}
// C line 14191
48 => {
vm_block = if ((((j) < (((*(s)).options).max_item_count)) as i32)) != 0 { 47 } else { 26 }; continue;
}
// C line 14189
49 => {
vm_block = 25; continue;
}
// C line 14187
50 => {
vm_block = if ((((((!(((((((*(prs)).flags()) as i32)) & (((1 as i32)).wrapping_shl(((2 as i32)) as u32)))) != 0) as i32)) != 0) && (((!((((*(s)).options).show_hidden()) != 0) as i32)) != 0)) as i32)) != 0 { 49 } else { 48 }; continue;
}
// C line 14186
51 => {
vm_block = if (((((*(prs)).atom) != ((((0 as i32)) as JSAtom))) as i32)) != 0 { 50 } else { 25 }; continue;
}
// C line 14185
52 => {
let _ = { let _ = { let assigned = (((0 as i32)) as u32); i = assigned; assigned }; { let assigned = get_shape_prop(sh); prs = assigned; assigned } };
vm_block = 24; continue;
}
// C line 14184
53 => {
let _ = { let assigned = (((0 as i32)) as u32); j = assigned; assigned };
vm_block = 52; continue;
}
// C line 14181
54 => {
vm_block = if !(sh).is_null() { 53 } else { 21 }; continue;
}
// C line 14180
55 => {
let _ = { let assigned = (*(p)).shape; sh = assigned; assigned };
vm_block = 54; continue;
}
// C line 14058
56 => {
let _ = js_printf(s, &quickjs_format_print(c"<%u empty item%s>".as_ptr(), &[QuickJSPrintArg::Int(n as u64), QuickJSPrintArg::Str(if ((((n) > ((((1 as i32)) as u32))) as i32)) != 0 { c"s".as_ptr() } else { c"".as_ptr() } as *const c_char)]));
vm_block = 55; continue;
}
// C line 14057
57 => {
let _ = js_print_comma(s, core::ptr::addr_of_mut!(comma_state));
vm_block = 56; continue;
}
// C line 14056
58 => {
let _ = { let assigned = (len).wrapping_sub((((*(p)).u).array).count); n = assigned; assigned };
vm_block = 57; continue;
}
// C line 14055
59 => {
vm_block = if (((((((*(p)).u).array).count) < (len)) as i32)) != 0 { 58 } else { 55 }; continue;
}
// C line 14054
60 => {
let _ = js_print_more_items(s, core::ptr::addr_of_mut!(comma_state), ((((*(p)).u).array).count).wrapping_sub(len1));
vm_block = 59; continue;
}
// C line 14053
61 => {
vm_block = if ((((len1) < ((((*(p)).u).array).count)) as i32)) != 0 { 60 } else { 59 }; continue;
}
// C line 14049
62 => {
vm_block = if ((((i) < (len1)) as i32)) != 0 { 65 } else { 61 }; continue;
}
// C line 14049
63 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 62; continue;
}
// C line 14051
64 => {
let _ = js_print_value(s, *(((((*(p)).u).array).u).values).offset((i) as isize));
vm_block = 63; continue;
}
// C line 14050
65 => {
let _ = js_print_comma(s, core::ptr::addr_of_mut!(comma_state));
vm_block = 64; continue;
}
// C line 14049
66 => {
let _ = { let assigned = (((0 as i32)) as u32); i = assigned; assigned };
vm_block = 62; continue;
}
// C line 14048
67 => {
let _ = { let assigned = crate::cutils_header::min_uint32((((*(p)).u).array).count, ((*(s)).options).max_item_count); len1 = assigned; assigned };
vm_block = 66; continue;
}
// C line 14046
68 => {
let _ = { let assigned = js_print_array_get_length(p); len = assigned; assigned };
vm_block = 67; continue;
}
// C line 14044
69 => {
vm_block = if ((*(p)).fast_array()) != 0 { 68 } else { 55 }; continue;
}
// C line 14042
70 => {
let _ = js_printf(s, &quickjs_format_print(c"[ ".as_ptr(), &[]));
vm_block = 69; continue;
}
// C line 14041
71 => {
let _ = { let assigned = (1 as i32); is_array = assigned; assigned };
vm_block = 70; continue;
}
// C line 14114
72 => {
let _ = js_print_more_items(s, core::ptr::addr_of_mut!(comma_state), ((((*(p)).u).array).count).wrapping_sub(len1_1));
vm_block = 55; continue;
}
// C line 14113
73 => {
vm_block = if ((((len1_1) < ((((*(p)).u).array).count)) as i32)) != 0 { 72 } else { 55 }; continue;
}
// C line 14071
74 => {
vm_block = if ((((i) < (len1_1)) as i32)) != 0 { 101 } else { 73 }; continue;
}
// C line 14071
75 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 74; continue;
}
// C line 14110
76 => {
vm_block = 75; continue;
}
// C line 14109
77 => {
let _ = js_print_float64(s, *(((ptr) as *mut f64)));
vm_block = 76; continue;
}
// C line 14107
78 => {
vm_block = 75; continue;
}
// C line 14106
79 => {
let _ = js_print_float64(s, ((*(((ptr) as *mut f32))) as f64));
vm_block = 78; continue;
}
// C line 14104
80 => {
vm_block = 75; continue;
}
// C line 14103
81 => {
let _ = js_print_float64(s, crate::cutils::fromfp16(*(((ptr) as *mut u16))));
vm_block = 80; continue;
}
// C line 14101
82 => {
vm_block = 75; continue;
}
// C line 14100
83 => {
let _ = js_printf(s, &quickjs_format_print(c"%llu".as_ptr(), &[QuickJSPrintArg::Int(*(((ptr) as *mut u64)) as u64)]));
vm_block = 82; continue;
}
// C line 14098
84 => {
vm_block = 75; continue;
}
// C line 14097 labels: ta_int64
85 => {
let _ = js_printf(s, &quickjs_format_print(c"%lld".as_ptr(), &[QuickJSPrintArg::Int(v as u64)]));
vm_block = 84; continue;
}
// C line 14095
86 => {
let _ = { let assigned = *(((ptr) as *mut i64)); v = assigned; assigned };
vm_block = 85; continue;
}
// C line 14093
87 => {
vm_block = 85; continue;
}
// C line 14092
88 => {
let _ = { let assigned = ((*(((ptr) as *mut u32))) as i64); v = assigned; assigned };
vm_block = 87; continue;
}
// C line 14090
89 => {
vm_block = 85; continue;
}
// C line 14089
90 => {
let _ = { let assigned = ((*(((ptr) as *mut i32))) as i64); v = assigned; assigned };
vm_block = 89; continue;
}
// C line 14087
91 => {
vm_block = 85; continue;
}
// C line 14086
92 => {
let _ = { let assigned = ((*(((ptr) as *mut u16))) as i64); v = assigned; assigned };
vm_block = 91; continue;
}
// C line 14084
93 => {
vm_block = 85; continue;
}
// C line 14083
94 => {
let _ = { let assigned = ((*(((ptr) as *mut i16))) as i64); v = assigned; assigned };
vm_block = 93; continue;
}
// C line 14081
95 => {
vm_block = 85; continue;
}
// C line 14080
96 => {
let _ = { let assigned = ((*(((ptr) as *mut i8))) as i64); v = assigned; assigned };
vm_block = 95; continue;
}
// C line 14078
97 => {
vm_block = 85; continue;
}
// C line 14077
98 => {
let _ = { let assigned = ((*(ptr)) as i64); v = assigned; assigned };
vm_block = 97; continue;
}
// C line 14074
99 => {
vm_block = match (((*(p)).class_id) as i32) { x if x == (JS_CLASS_FLOAT64_ARRAY as i32) => 77, x if x == (JS_CLASS_FLOAT32_ARRAY as i32) => 79, x if x == (JS_CLASS_FLOAT16_ARRAY as i32) => 81, x if x == (JS_CLASS_BIG_UINT64_ARRAY as i32) => 83, x if x == (JS_CLASS_BIG_INT64_ARRAY as i32) => 86, x if x == (JS_CLASS_UINT32_ARRAY as i32) => 88, x if x == (JS_CLASS_INT32_ARRAY as i32) => 90, x if x == (JS_CLASS_UINT16_ARRAY as i32) => 92, x if x == (JS_CLASS_INT16_ARRAY as i32) => 94, x if x == (JS_CLASS_INT8_ARRAY as i32) => 96, x if x == (JS_CLASS_UINT8_ARRAY as i32) => 98, x if x == (JS_CLASS_UINT8C_ARRAY as i32) => 98, _ => 75, }; continue;
}
// C line 14073
100 => {
let _ = js_print_comma(s, core::ptr::addr_of_mut!(comma_state));
vm_block = 99; continue;
}
// C line 14072
101 => {
ptr = (((((*(p)).u).array).u).uint8_ptr).offset((((i).wrapping_mul(size)) as isize));
vm_block = 100; continue;
}
// C line 14071
102 => {
let _ = { let assigned = (((0 as i32)) as u32); i = assigned; assigned };
vm_block = 74; continue;
}
// C line 14070
103 => {
let _ = { let assigned = crate::cutils_header::min_uint32((((*(p)).u).array).count, ((*(s)).options).max_item_count); len1_1 = assigned; assigned };
vm_block = 102; continue;
}
// C line 14069
104 => {
let _ = { let assigned = (1 as i32); is_array = assigned; assigned };
vm_block = 103; continue;
}
// C line 14067
105 => {
let _ = js_printf(s, &quickjs_format_print(c"(%u) [ ".as_ptr(), &[QuickJSPrintArg::Int((((*(p)).u).array).count as u64)]));
vm_block = 104; continue;
}
// C line 14066
106 => {
let _ = js_print_atom(s, (*((*(rt)).class_array).offset(((*(p)).class_id) as isize)).class_name);
vm_block = 105; continue;
}
// C line 14062
107 => {
size = ((((1 as i32)).wrapping_shl((((*((typed_array_size_log2).as_ptr()).offset((((((*(p)).class_id) as i32)).wrapping_sub((JS_CLASS_UINT8C_ARRAY as i32))) as isize)) as i32)) as u32)) as u32);
vm_block = 106; continue;
}
// C line 14131
108 => {
let _ = { let assigned = (2 as i32); comma_state = assigned; assigned };
vm_block = 55; continue;
}
// C line 14130
109 => {
let _ = js_printf(s, &quickjs_format_print(c"]".as_ptr(), &[]));
vm_block = 108; continue;
}
// C line 14128
110 => {
let _ = JS_FreeCString((*(s)).ctx, func_name_str);
vm_block = 109; continue;
}
// C line 14125
111 => {
let _ = js_puts(s, c"(anonymous)".as_ptr());
vm_block = 110; continue;
}
// C line 14127
112 => {
let _ = js_puts(s, func_name_str);
vm_block = 110; continue;
}
// C line 14124
113 => {
vm_block = if ((((((!(!(func_name_str).is_null()) as i32)) != 0) || (((((((*(func_name_str).offset(((0 as i32)) as isize)) as i32)) == ((0 as i32))) as i32)) != 0)) as i32)) != 0 { 111 } else { 112 }; continue;
}
// C line 14123
114 => {
let _ = { let assigned = get_prop_string((*(s)).ctx, JSValue { u: JSValueUnion { ptr: ((p) as *mut c_void) }, tag: (((JS_TAG_OBJECT as i32)) as i64) }, (((crate::quickjs_atom::JS_ATOM_name as i32)) as JSAtom)); func_name_str = assigned; assigned };
vm_block = 113; continue;
}
// C line 14122
115 => {
let _ = js_putc(s, (((32 as i32)) as c_char));
vm_block = 114; continue;
}
// C line 14120
116 => {
vm_block = if ((((((!((((*(s)).options).raw_dump()) != 0) as i32)) != 0) && (!((*(s)).ctx).is_null())) as i32)) != 0 { 115 } else { 109 }; continue;
}
// C line 14118
117 => {
let _ = js_printf(s, &quickjs_format_print(c"[Function".as_ptr(), &[]));
vm_block = 116; continue;
}
// C line 14156
118 => {
let _ = js_print_more_items(s, core::ptr::addr_of_mut!(comma_state), ((*(ms)).record_count).wrapping_sub(i));
vm_block = 55; continue;
}
// C line 14155
119 => {
vm_block = if ((((i) < ((*(ms)).record_count)) as i32)) != 0 { 118 } else { 55 }; continue;
}
// C line 14141
120 => {
vm_block = if ((((el) != (core::ptr::addr_of_mut!((*(ms)).records))) as i32)) != 0 { 132 } else { 119 }; continue;
}
// C line 14141
121 => {
let _ = { let assigned = (*(el)).next; el = assigned; assigned };
vm_block = 120; continue;
}
// C line 14153
122 => {
vm_block = 119; continue;
}
// C line 14152
123 => {
vm_block = if ((((i) >= (((*(s)).options).max_item_count)) as i32)) != 0 { 122 } else { 121 }; continue;
}
// C line 14151
124 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 123; continue;
}
// C line 14149
125 => {
let _ = js_print_value(s, (*(mr)).value);
vm_block = 124; continue;
}
// C line 14148
126 => {
let _ = js_printf(s, &quickjs_format_print(c" => ".as_ptr(), &[]));
vm_block = 125; continue;
}
// C line 14147
127 => {
vm_block = if (((((((*(p)).class_id) as i32)) == ((JS_CLASS_MAP as i32))) as i32)) != 0 { 126 } else { 124 }; continue;
}
// C line 14146
128 => {
let _ = js_print_value(s, (*(mr)).key);
vm_block = 127; continue;
}
// C line 14145
129 => {
vm_block = 121; continue;
}
// C line 14144
130 => {
vm_block = if ((*(mr)).empty) != 0 { 129 } else { 128 }; continue;
}
// C line 14143
131 => {
let _ = js_print_comma(s, core::ptr::addr_of_mut!(comma_state));
vm_block = 130; continue;
}
// C line 14142
132 => {
mr = (((((el) as *mut u8)).offset(-(((core::mem::offset_of!(JSMapRecord, link) as usize)) as isize))) as *mut JSMapRecord);
vm_block = 131; continue;
}
// C line 14141
133 => {
let _ = { let assigned = (*(core::ptr::addr_of_mut!((*(ms)).records))).next; el = assigned; assigned };
vm_block = 120; continue;
}
// C line 14140
134 => {
let _ = { let assigned = (((0 as i32)) as u32); i = assigned; assigned };
vm_block = 133; continue;
}
// C line 14139
135 => {
let _ = js_printf(s, &quickjs_format_print(c"(%u) { ".as_ptr(), &[QuickJSPrintArg::Int((*(ms)).record_count as u64)]));
vm_block = 134; continue;
}
// C line 14138
136 => {
let _ = js_print_atom(s, (*((*(rt)).class_array).offset(((*(p)).class_id) as isize)).class_name);
vm_block = 135; continue;
}
// C line 14137
137 => {
vm_block = 153; continue;
}
// C line 14136
138 => {
vm_block = if ((!(!(ms).is_null()) as i32)) != 0 { 137 } else { 136 }; continue;
}
// C line 14133
139 => {
ms = ((((*(p)).u).opaque) as *mut JSMapState);
vm_block = 138; continue;
}
// C line 14159
140 => {
let _ = { let assigned = (2 as i32); comma_state = assigned; assigned };
vm_block = 55; continue;
}
// C line 14158
141 => {
let _ = js_print_regexp(s, p);
vm_block = 140; continue;
}
// C line 14167
142 => {
let _ = { let assigned = (2 as i32); comma_state = assigned; assigned };
vm_block = 55; continue;
}
// C line 14166
143 => {
let _ = JS_FreeValueRT((*(s)).rt, str);
vm_block = 142; continue;
}
// C line 14165
144 => {
let _ = js_print_raw_string(s, str);
vm_block = 143; continue;
}
// C line 14164
145 => {
vm_block = 153; continue;
}
// C line 14163
146 => {
vm_block = if (JS_IsException(str)) != 0 { 145 } else { 144 }; continue;
}
// C line 14162
147 => {
str = get_date_string((*(s)).ctx, JSValue { u: JSValueUnion { ptr: ((p) as *mut c_void) }, tag: (((JS_TAG_OBJECT as i32)) as i64) }, (0 as i32), core::ptr::null_mut::<JSValue>(), (35 as i32));
vm_block = 146; continue;
}
// C line 14170
148 => {
let _ = { let assigned = (2 as i32); comma_state = assigned; assigned };
vm_block = 55; continue;
}
// C line 14169
149 => {
let _ = js_print_error(s, p);
vm_block = 148; continue;
}
// C line 14177
150 => {
let _ = js_printf(s, &quickjs_format_print(c"{ ".as_ptr(), &[]));
vm_block = 55; continue;
}
// C line 14175
151 => {
let _ = js_printf(s, &quickjs_format_print(c" ".as_ptr(), &[]));
vm_block = 150; continue;
}
// C line 14174
152 => {
let _ = js_print_atom(s, (*((*(rt)).class_array).offset(((*(p)).class_id) as isize)).class_name);
vm_block = 151; continue;
}
// C line 14173 labels: default_obj
153 => {
vm_block = if (((((((*(p)).class_id) as i32)) != ((JS_CLASS_OBJECT as i32))) as i32)) != 0 { 152 } else { 150 }; continue;
}
// C line 14168
154 => {
vm_block = if (((((((((((*(p)).class_id) as i32)) == ((JS_CLASS_ERROR as i32))) as i32)) != 0) && (!((*(s)).ctx).is_null())) as i32)) != 0 { 149 } else { 153 }; continue;
}
// C line 14160
155 => {
vm_block = if (((((((((((*(p)).class_id) as i32)) == ((JS_CLASS_DATE as i32))) as i32)) != 0) && (!((*(s)).ctx).is_null())) as i32)) != 0 { 147 } else { 154 }; continue;
}
// C line 14157
156 => {
vm_block = if (((((((((((*(p)).class_id) as i32)) == ((JS_CLASS_REGEXP as i32))) as i32)) != 0) && (!((*(s)).ctx).is_null())) as i32)) != 0 { 141 } else { 155 }; continue;
}
// C line 14132
157 => {
vm_block = if (((((((((((*(p)).class_id) as i32)) == ((JS_CLASS_MAP as i32))) as i32)) != 0) || ((((((((*(p)).class_id) as i32)) == ((JS_CLASS_SET as i32))) as i32)) != 0)) as i32)) != 0 { 139 } else { 156 }; continue;
}
// C line 14115
158 => {
vm_block = if (((((((((((*(p)).class_id) as i32)) == ((JS_CLASS_BYTECODE_FUNCTION as i32))) as i32)) != 0) || (((((((((*((*(rt)).class_array).offset(((*(p)).class_id) as isize)).call).is_some() as i32)) != 0) && ((((((((*(p)).class_id) as i32)) != ((JS_CLASS_PROXY as i32))) as i32)) != 0)) as i32)) != 0)) as i32)) != 0 { 117 } else { 157 }; continue;
}
// C line 14061
159 => {
vm_block = if (((((((((((*(p)).class_id) as i32)) >= ((JS_CLASS_UINT8C_ARRAY as i32))) as i32)) != 0) && ((((((((*(p)).class_id) as i32)) <= ((JS_CLASS_FLOAT64_ARRAY as i32))) as i32)) != 0)) as i32)) != 0 { 107 } else { 158 }; continue;
}
// C line 14040
160 => {
vm_block = if (((((((*(p)).class_id) as i32)) == ((JS_CLASS_ARRAY as i32))) as i32)) != 0 { 71 } else { 159 }; continue;
}
// C line 14039
161 => {
let _ = { let assigned = (0 as i32); is_array = assigned; assigned };
vm_block = 160; continue;
}
// C line 14038
162 => {
let _ = { let assigned = (0 as i32); comma_state = assigned; assigned };
vm_block = 161; continue;
}
// C line 14030
163 => {
rt = (*(s)).rt;
vm_block = 162; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:14269. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_print_stack_index(mut s: *mut JSPrintValueState, mut p: *mut JSObject) -> i32 {
let mut vm_local_storage = Vec::<u64>::new();
let mut i: i32 = core::mem::zeroed();
let mut vm_block: usize = 6;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 14275
1 => {
return ((1 as i32)).wrapping_neg();
}
// C line 14272
2 => {
vm_block = if ((((i) < ((*(s)).level)) as i32)) != 0 { 5 } else { 1 }; continue;
}
// C line 14272
3 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 2; continue;
}
// C line 14274
4 => {
return i;
}
// C line 14273
5 => {
vm_block = if ((((*(((*(s)).print_stack).as_mut_ptr()).offset((i) as isize)) == (p)) as i32)) != 0 { 4 } else { 3 }; continue;
}
// C line 14272
6 => {
let _ = { let assigned = (0 as i32); i = assigned; assigned };
vm_block = 2; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:14278. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_print_value(mut s: *mut JSPrintValueState, mut val: JSValue) -> () {
let mut vm_local_storage = Vec::<u64>::new();
let mut tag: u32 = core::mem::zeroed();
let mut str: *const c_char = core::mem::zeroed();
let mut str_1: JSValue = core::mem::zeroed();
let mut p: *mut JSBigInt = core::mem::zeroed();
let mut sgn: i32 = core::mem::zeroed();
let mut i: i32 = core::mem::zeroed();
let mut r: *mut JSStringRope = core::mem::zeroed();
let mut b: *mut JSFunctionBytecode = core::mem::zeroed();
let mut p_1: *mut JSObject = core::mem::zeroed();
let mut idx: i32 = core::mem::zeroed();
let mut atom: JSAtom = core::mem::zeroed();
let mut p_2: *mut JSAtomStruct = core::mem::zeroed();
let mut vm_block: usize = 77;
loop { match vm_block {
// C line ?
0 => {
return ();
}
// C line 14397
1 => {
vm_block = 0; continue;
}
// C line 14396
2 => {
let _ = js_printf(s, &quickjs_format_print(c"[unknown tag %d]".as_ptr(), &[QuickJSPrintArg::Int(tag as u64)]));
vm_block = 1; continue;
}
// C line 14394
3 => {
vm_block = 0; continue;
}
// C line 14393
4 => {
let _ = js_puts(s, c"[module]".as_ptr());
vm_block = 3; continue;
}
// C line 14391
5 => {
vm_block = 0; continue;
}
// C line 14389
6 => {
let _ = js_putc(s, (((41 as i32)) as c_char));
vm_block = 5; continue;
}
// C line 14388
7 => {
let _ = js_print_atom(s, js_get_atom_index((*(s)).rt, p_2));
vm_block = 6; continue;
}
// C line 14387
8 => {
let _ = js_puts(s, c"Symbol(".as_ptr());
vm_block = 7; continue;
}
// C line 14386
9 => {
p_2 = ((((val).u).ptr) as *mut JSAtomStruct);
vm_block = 8; continue;
}
// C line 14383
10 => {
vm_block = 0; continue;
}
// C line 14368
11 => {
let _ = js_printf(s, &quickjs_format_print(c"[circular %d]".as_ptr(), &[QuickJSPrintArg::Int(idx as u64)]));
vm_block = 10; continue;
}
// C line 14372
12 => {
let _ = { let old = (*(s)).level; (*(s)).level = ((*(s)).level).wrapping_sub(1); old };
vm_block = 10; continue;
}
// C line 14371
13 => {
let _ = js_print_object(s, ((((val).u).ptr) as *mut JSObject));
vm_block = 12; continue;
}
// C line 14370
14 => {
let _ = { let assigned = p_1; *(((*(s)).print_stack).as_mut_ptr()).offset(({ let old = (*(s)).level; (*(s)).level = ((*(s)).level).wrapping_add(1); old }) as isize) = assigned; assigned };
vm_block = 13; continue;
}
// C line 14380
15 => {
let _ = js_putc(s, (((93 as i32)) as c_char));
vm_block = 10; continue;
}
// C line 14378
16 => {
let _ = js_printf(s, &quickjs_format_print(c" %p".as_ptr(), &[QuickJSPrintArg::Ptr(((p_1) as *mut c_void) as *const c_void)]));
vm_block = 15; continue;
}
// C line 14377
17 => {
vm_block = if (((*(s)).options).raw_dump()) != 0 { 16 } else { 15 }; continue;
}
// C line 14376
18 => {
let _ = js_print_atom(s, atom);
vm_block = 17; continue;
}
// C line 14375
19 => {
let _ = js_putc(s, (((91 as i32)) as c_char));
vm_block = 18; continue;
}
// C line 14374
20 => {
atom = (*((*((*(s)).rt)).class_array).offset(((*(p_1)).class_id) as isize)).class_name;
vm_block = 19; continue;
}
// C line 14369
21 => {
vm_block = if (((((((*(s)).level) as u32)) < (((*(s)).options).max_depth)) as i32)) != 0 { 14 } else { 20 }; continue;
}
// C line 14367
22 => {
vm_block = if ((((idx) >= ((0 as i32))) as i32)) != 0 { 11 } else { 21 }; continue;
}
// C line 14366
23 => {
let _ = { let assigned = js_print_stack_index(s, p_1); idx = assigned; assigned };
vm_block = 22; continue;
}
// C line 14364
24 => {
p_1 = ((((val).u).ptr) as *mut JSObject);
vm_block = 23; continue;
}
// C line 14361
25 => {
vm_block = 0; continue;
}
// C line 14359
26 => {
let _ = js_putc(s, (((93 as i32)) as c_char));
vm_block = 25; continue;
}
// C line 14358
27 => {
let _ = js_print_atom(s, (*(b)).func_name);
vm_block = 26; continue;
}
// C line 14357
28 => {
let _ = js_puts(s, c"[bytecode ".as_ptr());
vm_block = 27; continue;
}
// C line 14356
29 => {
b = ((((val).u).ptr) as *mut JSFunctionBytecode);
vm_block = 28; continue;
}
// C line 14353
30 => {
vm_block = 0; continue;
}
// C line 14349
31 => {
let _ = js_printf(s, &quickjs_format_print(c"[rope len=%d depth=%d]".as_ptr(), &[QuickJSPrintArg::Int((*(r)).len as u64), QuickJSPrintArg::Int((((*(r)).depth) as i32) as u64)]));
vm_block = 30; continue;
}
// C line 14348
32 => {
r = ((((val).u).ptr) as *mut JSStringRope);
vm_block = 31; continue;
}
// C line 14351
33 => {
let _ = js_print_string(s, val);
vm_block = 30; continue;
}
// C line 14347
34 => {
vm_block = if (((((((*(s)).options).raw_dump()) != 0) && (((((tag) == ((((JS_TAG_STRING_ROPE as i32)) as u32))) as i32)) != 0)) as i32)) != 0 { 32 } else { 33 }; continue;
}
// C line 14344
35 => {
vm_block = 0; continue;
}
// C line 14320
36 => {
let _ = JS_FreeValueRT((*(s)).rt, str_1);
vm_block = 35; continue;
}
// C line 14319
37 => {
let _ = js_putc(s, (((110 as i32)) as c_char));
vm_block = 36; continue;
}
// C line 14318
38 => {
let _ = js_print_raw_string(s, str_1);
vm_block = 37; continue;
}
// C line 14317
39 => {
vm_block = 55; continue;
}
// C line 14316
40 => {
vm_block = if (JS_IsException(str_1)) != 0 { 39 } else { 38 }; continue;
}
// C line 14315
41 => {
str_1 = js_bigint_to_string((*(s)).ctx, val);
vm_block = 40; continue;
}
// C line 14342
42 => {
let _ = js_putc(s, (((41 as i32)) as c_char));
vm_block = 35; continue;
}
// C line 14341
43 => {
vm_block = if (sgn) != 0 { 42 } else { 35 }; continue;
}
// C line 14340
44 => {
let _ = js_putc(s, (((110 as i32)) as c_char));
vm_block = 43; continue;
}
// C line 14331
45 => {
vm_block = if ((((i) >= ((0 as i32))) as i32)) != 0 { 49 } else { 44 }; continue;
}
// C line 14331
46 => {
let _ = { let old = i; i = (i).wrapping_sub(1); old };
vm_block = 45; continue;
}
// C line 14337
47 => {
let _ = js_printf(s, &quickjs_format_print(c"%016llx".as_ptr(), &[QuickJSPrintArg::Int(*(((*(p)).tab).as_mut_ptr()).offset((i) as isize) as u64)]));
vm_block = 46; continue;
}
// C line 14333
48 => {
let _ = js_putc(s, (((95 as i32)) as c_char));
vm_block = 47; continue;
}
// C line 14332
49 => {
vm_block = if ((((((i) as u32)) != (((*(p)).len).wrapping_sub((((1 as i32)) as u32)))) as i32)) != 0 { 48 } else { 47 }; continue;
}
// C line 14331
50 => {
let _ = { let assigned = ((((*(p)).len).wrapping_sub((((1 as i32)) as u32))) as i32); i = assigned; assigned };
vm_block = 45; continue;
}
// C line 14330
51 => {
let _ = js_printf(s, &quickjs_format_print(c"0x".as_ptr(), &[]));
vm_block = 50; continue;
}
// C line 14329
52 => {
let _ = js_printf(s, &quickjs_format_print(c"BigInt.asIntN(%d,".as_ptr(), &[QuickJSPrintArg::Int(((*(p)).len).wrapping_mul((((64 as i32)) as u32)) as u64)]));
vm_block = 51; continue;
}
// C line 14328
53 => {
vm_block = if (sgn) != 0 { 52 } else { 51 }; continue;
}
// C line 14327
54 => {
let _ = { let assigned = js_bigint_sign(p); sgn = assigned; assigned };
vm_block = 53; continue;
}
// C line 14325 labels: raw_bigint
55 => {
let _ = { let assigned = ((((val).u).ptr) as *mut JSBigInt); p = assigned; assigned };
vm_block = 54; continue;
}
// C line 14314
56 => {
vm_block = if ((((((!((((*(s)).options).raw_dump()) != 0) as i32)) != 0) && (!((*(s)).ctx).is_null())) as i32)) != 0 { 41 } else { 55 }; continue;
}
// C line 14312
57 => {
vm_block = 0; continue;
}
// C line 14311
58 => {
let _ = js_printf(s, &quickjs_format_print(c"%lldn".as_ptr(), &[QuickJSPrintArg::Int(((val).u).short_big_int as u64)]));
vm_block = 57; continue;
}
// C line 14309
59 => {
vm_block = 0; continue;
}
// C line 14308
60 => {
let _ = js_print_float64(s, ((val).u).float64);
vm_block = 59; continue;
}
// C line 14306
61 => {
vm_block = 0; continue;
}
// C line 14305 labels: print_str
62 => {
let _ = js_puts(s, str);
vm_block = 61; continue;
}
// C line 14303
63 => {
let _ = { let assigned = c"undefined".as_ptr(); str = assigned; assigned };
vm_block = 62; continue;
}
// C line 14301
64 => {
vm_block = 62; continue;
}
// C line 14300
65 => {
let _ = { let assigned = c"uninitialized".as_ptr(); str = assigned; assigned };
vm_block = 64; continue;
}
// C line 14298
66 => {
vm_block = 62; continue;
}
// C line 14297
67 => {
let _ = { let assigned = c"exception".as_ptr(); str = assigned; assigned };
vm_block = 66; continue;
}
// C line 14295
68 => {
vm_block = 62; continue;
}
// C line 14294
69 => {
let _ = { let assigned = c"null".as_ptr(); str = assigned; assigned };
vm_block = 68; continue;
}
// C line 14292
70 => {
vm_block = 62; continue;
}
// C line 14289
71 => {
let _ = { let assigned = c"true".as_ptr(); str = assigned; assigned };
vm_block = 70; continue;
}
// C line 14291
72 => {
let _ = { let assigned = c"false".as_ptr(); str = assigned; assigned };
vm_block = 70; continue;
}
// C line 14288
73 => {
vm_block = if (((((val).u).uint64) as i32)) != 0 { 71 } else { 72 }; continue;
}
// C line 14286
74 => {
vm_block = 0; continue;
}
// C line 14285
75 => {
let _ = js_printf(s, &quickjs_format_print(c"%d".as_ptr(), &[QuickJSPrintArg::Int(((((val).u).uint64) as i32) as u64)]));
vm_block = 74; continue;
}
// C line 14283
76 => {
vm_block = match tag { x if x == (((JS_TAG_MODULE as i32)) as u32) => 4, x if x == (((JS_TAG_SYMBOL as i32)) as u32) => 9, x if x == (((JS_TAG_OBJECT as i32)) as u32) => 24, x if x == (((JS_TAG_FUNCTION_BYTECODE as i32)) as u32) => 29, x if x == (((JS_TAG_STRING_ROPE as i32)) as u32) => 34, x if x == (((JS_TAG_STRING as i32)) as u32) => 34, x if x == (((JS_TAG_BIG_INT as i32)) as u32) => 56, x if x == (((JS_TAG_SHORT_BIG_INT as i32)) as u32) => 58, x if x == (((JS_TAG_FLOAT64 as i32)) as u32) => 60, x if x == (((JS_TAG_UNDEFINED as i32)) as u32) => 63, x if x == (((JS_TAG_UNINITIALIZED as i32)) as u32) => 65, x if x == (((JS_TAG_EXCEPTION as i32)) as u32) => 67, x if x == (((JS_TAG_NULL as i32)) as u32) => 69, x if x == (((JS_TAG_BOOL as i32)) as u32) => 73, x if x == (((JS_TAG_INT as i32)) as u32) => 75, _ => 2, }; continue;
}
// C line 14280
77 => {
tag = (((((val).tag) as i32)) as u32);
vm_block = 76; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:14401. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
pub unsafe fn JS_PrintValueSetDefaultOptions(mut options: *mut JSPrintValueOptions) -> () {
let mut vm_local_storage = Vec::<u64>::new();
let mut vm_block: usize = 4;
loop { match vm_block {
// C line ?
0 => {
return ();
}
// C line 14406
1 => {
let _ = { let assigned = (((100 as i32)) as u32); (*(options)).max_item_count = assigned; assigned };
vm_block = 0; continue;
}
// C line 14405
2 => {
let _ = { let assigned = (((1000 as i32)) as u32); (*(options)).max_string_length = assigned; assigned };
vm_block = 1; continue;
}
// C line 14404
3 => {
let _ = { let assigned = (((2 as i32)) as u32); (*(options)).max_depth = assigned; assigned };
vm_block = 2; continue;
}
// C line 14403
4 => {
let _ = { let dst = (((options) as *mut c_void)) as *mut u8; core::ptr::write_bytes(dst, ((0 as i32)) as u8, ((size_of::<JSPrintValueOptions>() as usize)) as usize); dst as *mut c_void };
vm_block = 3; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:14409. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn JS_PrintValueInternal(mut rt: *mut JSRuntime, mut ctx: *mut JSContext, mut write_func: Option<JSPrintValueWrite>, mut write_opaque: *mut c_void, mut val: JSValue, mut options: *const JSPrintValueOptions) -> () {
let mut vm_local_storage = Vec::<u64>::new();
let mut ss: JSPrintValueState = core::mem::zeroed();
let mut s: *mut JSPrintValueState = core::mem::zeroed();
let mut vm_block: usize = 17;
loop { match vm_block {
// C line ?
0 => {
return ();
}
// C line 14431
1 => {
let _ = js_print_value(s, val);
vm_block = 0; continue;
}
// C line 14430
2 => {
let _ = { let assigned = (0 as i32); (*(s)).level = assigned; assigned };
vm_block = 1; continue;
}
// C line 14429
3 => {
let _ = { let assigned = write_opaque; (*(s)).write_opaque = assigned; assigned };
vm_block = 2; continue;
}
// C line 14428
4 => {
let _ = { let assigned = write_func; (*(s)).write_func = assigned; assigned };
vm_block = 3; continue;
}
// C line 14427
5 => {
let _ = { let assigned = ctx; (*(s)).ctx = assigned; assigned };
vm_block = 4; continue;
}
// C line 14426
6 => {
let _ = { let assigned = rt; (*(s)).rt = assigned; assigned };
vm_block = 5; continue;
}
// C line 14425
7 => {
let _ = { let assigned = (4294967295 as u32); ((*(s)).options).max_item_count = assigned; assigned };
vm_block = 6; continue;
}
// C line 14424
8 => {
vm_block = if ((((((*(s)).options).max_item_count) == ((((0 as i32)) as u32))) as i32)) != 0 { 7 } else { 6 }; continue;
}
// C line 14423
9 => {
let _ = { let assigned = (4294967295 as u32); ((*(s)).options).max_string_length = assigned; assigned };
vm_block = 8; continue;
}
// C line 14422
10 => {
vm_block = if ((((((*(s)).options).max_string_length) == ((((0 as i32)) as u32))) as i32)) != 0 { 9 } else { 8 }; continue;
}
// C line 14419
11 => {
let _ = { let assigned = (((8 as i32)) as u32); ((*(s)).options).max_depth = assigned; assigned };
vm_block = 10; continue;
}
// C line 14421
12 => {
let _ = { let assigned = ((crate::cutils_header::min_int(((((*(s)).options).max_depth) as i32), (8 as i32))) as u32); ((*(s)).options).max_depth = assigned; assigned };
vm_block = 10; continue;
}
// C line 14418
13 => {
vm_block = if ((((((*(s)).options).max_depth) <= ((((0 as i32)) as u32))) as i32)) != 0 { 11 } else { 12 }; continue;
}
// C line 14415
14 => {
let _ = { let assigned = *(options); (*(s)).options = assigned; assigned };
vm_block = 13; continue;
}
// C line 14417
15 => {
let _ = JS_PrintValueSetDefaultOptions(core::ptr::addr_of_mut!((*(s)).options));
vm_block = 13; continue;
}
// C line 14414
16 => {
vm_block = if !(options).is_null() { 14 } else { 15 }; continue;
}
// C line 14413
17 => {
s = core::ptr::addr_of_mut!(ss);
vm_block = 16; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:14434. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
pub unsafe fn JS_PrintValueRT(mut rt: *mut JSRuntime, mut write_func: Option<JSPrintValueWrite>, mut write_opaque: *mut c_void, mut val: JSValue, mut options: *const JSPrintValueOptions) -> () {
let mut vm_local_storage = Vec::<u64>::new();
let mut vm_block: usize = 1;
loop { match vm_block {
// C line ?
0 => {
return ();
}
// C line 14437
1 => {
let _ = JS_PrintValueInternal(rt, core::ptr::null_mut::<JSContext>(), write_func, write_opaque, val, options);
vm_block = 0; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:14440. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
pub unsafe fn JS_PrintValue(mut ctx: *mut JSContext, mut write_func: Option<JSPrintValueWrite>, mut write_opaque: *mut c_void, mut val: JSValue, mut options: *const JSPrintValueOptions) -> () {
let mut vm_local_storage = Vec::<u64>::new();
let mut vm_block: usize = 1;
loop { match vm_block {
// C line ?
0 => {
return ();
}
// C line 14443
1 => {
let _ = JS_PrintValueInternal((*(ctx)).rt, ctx, write_func, write_opaque, val, options);
vm_block = 0; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:14452. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn print_atom(mut ctx: *mut JSContext, mut atom: JSAtom) -> () {
let mut vm_local_storage = Vec::<u64>::new();
let mut ss: JSPrintValueState = core::mem::zeroed();
let mut s: *mut JSPrintValueState = core::mem::zeroed();
let mut vm_block: usize = 7;
loop { match vm_block {
// C line ?
0 => {
return ();
}
// C line 14460
1 => {
let _ = js_print_atom(s, atom);
vm_block = 0; continue;
}
// C line 14459
2 => {
let _ = { let assigned = ((core::ptr::null_mut::<c_void>()) as *mut c_void); (*(s)).write_opaque = assigned; assigned };
vm_block = 1; continue;
}
// C line 14458
3 => {
let _ = { let assigned = Some(js_dump_value_write as JSPrintValueWrite); (*(s)).write_func = assigned; assigned };
vm_block = 2; continue;
}
// C line 14457
4 => {
let _ = { let assigned = ctx; (*(s)).ctx = assigned; assigned };
vm_block = 3; continue;
}
// C line 14456
5 => {
let _ = { let assigned = (*(ctx)).rt; (*(s)).rt = assigned; assigned };
vm_block = 4; continue;
}
// C line 14455
6 => {
let _ = { let dst = (((s) as *mut c_void)) as *mut u8; core::ptr::write_bytes(dst, ((0 as i32)) as u8, ((size_of::<JSPrintValueState>() as usize)) as usize); dst as *mut c_void };
vm_block = 5; continue;
}
// C line 14454
7 => {
s = core::ptr::addr_of_mut!(ss);
vm_block = 6; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:14463. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn JS_DumpAtom(mut ctx: *mut JSContext, mut str: *const c_char, mut atom: JSAtom) -> () {
let mut vm_local_storage = Vec::<u64>::new();
let mut vm_block: usize = 3;
loop { match vm_block {
// C line ?
0 => {
return ();
}
// C line 14467
1 => {
let _ = quickjs_debug_write(&quickjs_format_print(c"\n".as_ptr(), &[]));
vm_block = 0; continue;
}
// C line 14466
2 => {
let _ = print_atom(ctx, atom);
vm_block = 1; continue;
}
// C line 14465
3 => {
let _ = quickjs_debug_write(&quickjs_format_print(c"%s=".as_ptr(), &[QuickJSPrintArg::Str(str as *const c_char)]));
vm_block = 2; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:14470. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn JS_DumpValue(mut ctx: *mut JSContext, mut str: *const c_char, mut val: JSValue) -> () {
let mut vm_local_storage = Vec::<u64>::new();
let mut vm_block: usize = 3;
loop { match vm_block {
// C line ?
0 => {
return ();
}
// C line 14474
1 => {
let _ = quickjs_debug_write(&quickjs_format_print(c"\n".as_ptr(), &[]));
vm_block = 0; continue;
}
// C line 14473
2 => {
let _ = JS_PrintValue(ctx, Some(js_dump_value_write as JSPrintValueWrite), ((core::ptr::null_mut::<c_void>()) as *mut c_void), val, core::ptr::null_mut::<JSPrintValueOptions>());
vm_block = 1; continue;
}
// C line 14472
3 => {
let _ = quickjs_debug_write(&quickjs_format_print(c"%s=".as_ptr(), &[QuickJSPrintArg::Str(str as *const c_char)]));
vm_block = 2; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:14477. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn JS_DumpValueRT(mut rt: *mut JSRuntime, mut str: *const c_char, mut val: JSValue) -> () {
let mut vm_local_storage = Vec::<u64>::new();
let mut vm_block: usize = 3;
loop { match vm_block {
// C line ?
0 => {
return ();
}
// C line 14481
1 => {
let _ = quickjs_debug_write(&quickjs_format_print(c"\n".as_ptr(), &[]));
vm_block = 0; continue;
}
// C line 14480
2 => {
let _ = JS_PrintValueRT(rt, Some(js_dump_value_write as JSPrintValueWrite), ((core::ptr::null_mut::<c_void>()) as *mut c_void), val, core::ptr::null_mut::<JSPrintValueOptions>());
vm_block = 1; continue;
}
// C line 14479
3 => {
let _ = quickjs_debug_write(&quickjs_format_print(c"%s=".as_ptr(), &[QuickJSPrintArg::Str(str as *const c_char)]));
vm_block = 2; continue;
}
_ => std::process::abort(),
} }
}
