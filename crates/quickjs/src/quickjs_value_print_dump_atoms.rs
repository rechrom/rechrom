// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:2981. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn JS_DumpChar(mut fo: *mut QuickJSPrintStream, mut c: i32, mut sep: i32) -> () {
let mut vm_local_storage = Vec::<u64>::new();
let mut vm_block: usize = 9;
loop { match vm_block {
// C line ?
0 => {
return ();
}
// C line 2985
1 => {
let _ = quickjs_stream_putc(c, fo as *mut QuickJSPrintStream);
vm_block = 0; continue;
}
// C line 2984
2 => {
let _ = quickjs_stream_putc((92 as i32), fo as *mut QuickJSPrintStream);
vm_block = 1; continue;
}
// C line 2987
3 => {
let _ = quickjs_stream_putc(c, fo as *mut QuickJSPrintStream);
vm_block = 0; continue;
}
// C line 2990
4 => {
let _ = quickjs_stream_putc((110 as i32), fo as *mut QuickJSPrintStream);
vm_block = 0; continue;
}
// C line 2989
5 => {
let _ = quickjs_stream_putc((92 as i32), fo as *mut QuickJSPrintStream);
vm_block = 4; continue;
}
// C line 2992
6 => {
let _ = quickjs_stream_write(fo as *mut QuickJSPrintStream, &quickjs_format_print(c"\\u%04x".as_ptr(), &[QuickJSPrintArg::Int(c as u64)]));
vm_block = 0; continue;
}
// C line 2988
7 => {
vm_block = if ((((c) == ((10 as i32))) as i32)) != 0 { 5 } else { 6 }; continue;
}
// C line 2986
8 => {
vm_block = if ((((((((c) >= ((32 as i32))) as i32)) != 0) && (((((c) <= ((126 as i32))) as i32)) != 0)) as i32)) != 0 { 3 } else { 7 }; continue;
}
// C line 2983
9 => {
vm_block = if ((((((((c) == (sep)) as i32)) != 0) || (((((c) == ((92 as i32))) as i32)) != 0)) as i32)) != 0 { 2 } else { 8 }; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:2996. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn JS_DumpString(mut rt: *mut JSRuntime, mut p: *const JSString) -> () {
let mut vm_local_storage = Vec::<u64>::new();
let mut i: i32 = core::mem::zeroed();
let mut sep: i32 = core::mem::zeroed();
let mut vm_block: usize = 11;
loop { match vm_block {
// C line ?
0 => {
return ();
}
// C line 3010
1 => {
let _ = quickjs_stream_putc(sep,core::ptr::null_mut());
vm_block = 0; continue;
}
// C line 3007
2 => {
vm_block = if ((((i) < ((((*(p)).len()) as i32))) as i32)) != 0 { 4 } else { 1 }; continue;
}
// C line 3007
3 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 2; continue;
}
// C line 3008
4 => {
let _ = JS_DumpChar(core::ptr::null_mut::<c_void>() as *mut QuickJSPrintStream, string_get(p, i), sep);
vm_block = 3; continue;
}
// C line 3007
5 => {
let _ = { let assigned = (0 as i32); i = assigned; assigned };
vm_block = 2; continue;
}
// C line 3006
6 => {
let _ = quickjs_stream_putc(sep,core::ptr::null_mut());
vm_block = 5; continue;
}
// C line 3005
7 => {
let _ = { let assigned = if (((((*(js_rc(((p) as *mut c_void)))).ref_count) == ((1 as i32))) as i32)) != 0 { (34 as i32) } else { (39 as i32) }; sep = assigned; assigned };
vm_block = 6; continue;
}
// C line 3004
8 => {
let _ = quickjs_debug_write(&quickjs_format_print(c"%d".as_ptr(), &[QuickJSPrintArg::Int((*(js_rc(((p) as *mut c_void)))).ref_count as u64)]));
vm_block = 7; continue;
}
// C line 3002
9 => {
return;
}
// C line 3001
10 => {
let _ = quickjs_debug_write(&quickjs_format_print(c"<null>".as_ptr(), &[]));
vm_block = 9; continue;
}
// C line 3000
11 => {
vm_block = if ((((p) == (core::ptr::null_mut::<JSString>())) as i32)) != 0 { 10 } else { 8 }; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:3013. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn JS_DumpAtoms(mut rt: *mut JSRuntime) -> () {
let mut vm_local_storage = Vec::<u64>::new();
let mut p: *mut JSAtomStruct = core::mem::zeroed();
let mut h: i32 = core::mem::zeroed();
let mut i: i32 = core::mem::zeroed();
let mut vm_block: usize = 26;
loop { match vm_block {
// C line ?
0 => {
return ();
}
// C line 3045
1 => {
let _ = quickjs_debug_write(&quickjs_format_print(c"}\n".as_ptr(), &[]));
vm_block = 0; continue;
}
// C line 3036
2 => {
vm_block = if ((((i) < ((*(rt)).atom_size)) as i32)) != 0 { 9 } else { 1 }; continue;
}
// C line 3036
3 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 2; continue;
}
// C line 3042
4 => {
let _ = quickjs_debug_write(&quickjs_format_print(c" %d }\n".as_ptr(), &[QuickJSPrintArg::Int((*(p)).hash_next as u64)]));
vm_block = 3; continue;
}
// C line 3041
5 => {
let _ = JS_DumpString(rt, p);
vm_block = 4; continue;
}
// C line 3040
6 => {
vm_block = if ((!((((((((((((*(p)).len()) as i32)) == ((0 as i32))) as i32)) != 0) && ((((((((*(p)).is_wide_char()) as i32)) != ((0 as i32))) as i32)) != 0)) as i32)) != 0) as i32)) != 0 { 5 } else { 4 }; continue;
}
// C line 3039
7 => {
let _ = quickjs_debug_write(&quickjs_format_print(c"  %d: { %d %08x ".as_ptr(), &[QuickJSPrintArg::Int(i as u64), QuickJSPrintArg::Int((((*(p)).atom_type()) as i32) as u64), QuickJSPrintArg::Int((((*(p)).hash()) as i32) as u64)]));
vm_block = 6; continue;
}
// C line 3038
8 => {
vm_block = if ((!((atom_is_free(p)) != 0) as i32)) != 0 { 7 } else { 3 }; continue;
}
// C line 3037
9 => {
let _ = { let assigned = *((*(rt)).atom_array).offset((i) as isize); p = assigned; assigned };
vm_block = 8; continue;
}
// C line 3036
10 => {
let _ = { let assigned = (0 as i32); i = assigned; assigned };
vm_block = 2; continue;
}
// C line 3035
11 => {
let _ = quickjs_debug_write(&quickjs_format_print(c"JSAtom table: {\n".as_ptr(), &[]));
vm_block = 10; continue;
}
// C line 3034
12 => {
let _ = quickjs_debug_write(&quickjs_format_print(c"}\n".as_ptr(), &[]));
vm_block = 11; continue;
}
// C line 3021
13 => {
vm_block = if ((((i) < ((*(rt)).atom_hash_size)) as i32)) != 0 { 23 } else { 12 }; continue;
}
// C line 3021
14 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 13; continue;
}
// C line 3031
15 => {
let _ = quickjs_debug_write(&quickjs_format_print(c"\n".as_ptr(), &[]));
vm_block = 14; continue;
}
// C line 3025
16 => {
vm_block = if (h) != 0 { 20 } else { 15 }; continue;
}
// C line 3029
17 => {
let _ = { let assigned = (((*(p)).hash_next) as i32); h = assigned; assigned };
vm_block = 16; continue;
}
// C line 3028
18 => {
let _ = JS_DumpString(rt, p);
vm_block = 17; continue;
}
// C line 3027
19 => {
let _ = quickjs_debug_write(&quickjs_format_print(c" ".as_ptr(), &[]));
vm_block = 18; continue;
}
// C line 3026
20 => {
let _ = { let assigned = *((*(rt)).atom_array).offset((h) as isize); p = assigned; assigned };
vm_block = 19; continue;
}
// C line 3024
21 => {
let _ = quickjs_debug_write(&quickjs_format_print(c"  %d:".as_ptr(), &[QuickJSPrintArg::Int(i as u64)]));
vm_block = 16; continue;
}
// C line 3023
22 => {
vm_block = if (h) != 0 { 21 } else { 14 }; continue;
}
// C line 3022
23 => {
let _ = { let assigned = ((*((*(rt)).atom_hash).offset((i) as isize)) as i32); h = assigned; assigned };
vm_block = 22; continue;
}
// C line 3021
24 => {
let _ = { let assigned = (0 as i32); i = assigned; assigned };
vm_block = 13; continue;
}
// C line 3020
25 => {
let _ = quickjs_debug_write(&quickjs_format_print(c"JSAtom hash table: {\n".as_ptr(), &[]));
vm_block = 24; continue;
}
// C line 3018
26 => {
let _ = quickjs_debug_write(&quickjs_format_print(c"JSAtom count=%d size=%d hash_size=%d:\n".as_ptr(), &[QuickJSPrintArg::Int((*(rt)).atom_count as u64), QuickJSPrintArg::Int((*(rt)).atom_size as u64), QuickJSPrintArg::Int((*(rt)).atom_hash_size as u64)]));
vm_block = 25; continue;
}
_ => std::process::abort(),
} }
}
