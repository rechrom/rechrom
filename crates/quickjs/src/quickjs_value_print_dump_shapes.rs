// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:5566. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn JS_DumpShape(mut rt: *mut JSRuntime, mut i: i32, mut sh: *mut JSShape) -> () {
let mut vm_local_storage = Vec::<u64>::new();
let mut atom_buf: [c_char; 64] = core::mem::zeroed();
let mut j: i32 = core::mem::zeroed();
let mut vm_block: usize = 6;
loop { match vm_block {
// C line ?
0 => {
return ();
}
// C line 5579
1 => {
let _ = quickjs_debug_write(&quickjs_format_print(c"\n".as_ptr(), &[]));
vm_block = 0; continue;
}
// C line 5575
2 => {
vm_block = if ((((j) < ((*(sh)).prop_count)) as i32)) != 0 { 4 } else { 1 }; continue;
}
// C line 5575
3 => {
let _ = { let old = j; j = (j).wrapping_add(1); old };
vm_block = 2; continue;
}
// C line 5576
4 => {
let _ = quickjs_debug_write(&quickjs_format_print(c" %s".as_ptr(), &[QuickJSPrintArg::Str(JS_AtomGetStrRT(rt, (atom_buf).as_mut_ptr(), (((size_of::<[c_char; 64]>() as usize)) as i32), (*(get_shape_prop(sh)).offset((j) as isize)).atom) as *const c_char)]));
vm_block = 3; continue;
}
// C line 5575
5 => {
let _ = { let assigned = (0 as i32); j = assigned; assigned };
vm_block = 2; continue;
}
// C line 5572
6 => {
let _ = quickjs_debug_write(&quickjs_format_print(c"%5d %3d%c %14p %5d %5d".as_ptr(), &[QuickJSPrintArg::Int(i as u64), QuickJSPrintArg::Int((*(js_rc(((sh) as *mut c_void)))).ref_count as u64), QuickJSPrintArg::Int(((*(c" *".as_ptr()).offset(((*(sh)).is_hashed) as isize)) as i32) as u64), QuickJSPrintArg::Ptr((((*(sh)).proto) as *mut c_void) as *const c_void), QuickJSPrintArg::Int((*(sh)).prop_size as u64), QuickJSPrintArg::Int((*(sh)).prop_count as u64)]));
vm_block = 5; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:5582. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn JS_DumpShapes(mut rt: *mut JSRuntime) -> () {
let mut vm_local_storage = Vec::<u64>::new();
let mut i: i32 = core::mem::zeroed();
let mut sh: *mut JSShape = core::mem::zeroed();
let mut el: *mut list_head = core::mem::zeroed();
let mut p: *mut JSObject = core::mem::zeroed();
let mut gp: *mut JSGCObjectHeader = core::mem::zeroed();
let mut vm_block: usize = 19;
loop { match vm_block {
// C line ?
0 => {
return ();
}
// C line 5608
1 => {
let _ = quickjs_debug_write(&quickjs_format_print(c"}\n".as_ptr(), &[]));
vm_block = 0; continue;
}
// C line 5599
2 => {
vm_block = if ((((el) != (core::ptr::addr_of_mut!((*(rt)).gc_obj_list))) as i32)) != 0 { 8 } else { 1 }; continue;
}
// C line 5599
3 => {
let _ = { let assigned = (*(el)).next; el = assigned; assigned };
vm_block = 2; continue;
}
// C line 5604
4 => {
let _ = JS_DumpShape(rt, ((1 as i32)).wrapping_neg(), (*(p)).shape);
vm_block = 3; continue;
}
// C line 5603
5 => {
vm_block = if ((!(((*((*(p)).shape)).is_hashed) != 0) as i32)) != 0 { 4 } else { 3 }; continue;
}
// C line 5602
6 => {
let _ = { let assigned = ((gp) as *mut JSObject); p = assigned; assigned };
vm_block = 5; continue;
}
// C line 5601
7 => {
vm_block = if (((((((*(js_rc(((gp) as *mut c_void)))).gc_obj_type()) as i32)) == ((JS_GC_OBJ_TYPE_JS_OBJECT as i32))) as i32)) != 0 { 6 } else { 3 }; continue;
}
// C line 5600
8 => {
let _ = { let assigned = (((((el) as *mut u8)).offset(-(((core::mem::offset_of!(JSGCObjectHeader, link) as usize)) as isize))) as *mut JSGCObjectHeader); gp = assigned; assigned };
vm_block = 7; continue;
}
// C line 5599
9 => {
let _ = { let assigned = (*(core::ptr::addr_of_mut!((*(rt)).gc_obj_list))).next; el = assigned; assigned };
vm_block = 2; continue;
}
// C line 5592
10 => {
vm_block = if ((((i) < ((*(rt)).shape_hash_size)) as i32)) != 0 { 16 } else { 9 }; continue;
}
// C line 5592
11 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 10; continue;
}
// C line 5593
12 => {
vm_block = if ((((sh) != (core::ptr::null_mut::<JSShape>())) as i32)) != 0 { 15 } else { 11 }; continue;
}
// C line 5593
13 => {
let _ = { let assigned = (*(sh)).shape_hash_next; sh = assigned; assigned };
vm_block = 12; continue;
}
// C line 5595
14 => {
let _ = if ((((!(((*(sh)).is_hashed) != 0) as i32)) as i64)) != 0 { std::process::abort() } else { { let _ = (0 as i32); } };
vm_block = 13; continue;
}
// C line 5594
15 => {
let _ = JS_DumpShape(rt, i, sh);
vm_block = 14; continue;
}
// C line 5593
16 => {
let _ = { let assigned = *((*(rt)).shape_hash).offset((i) as isize); sh = assigned; assigned };
vm_block = 12; continue;
}
// C line 5592
17 => {
let _ = { let assigned = (0 as i32); i = assigned; assigned };
vm_block = 10; continue;
}
// C line 5591
18 => {
let _ = quickjs_debug_write(&quickjs_format_print(c"%5s %4s %14s %5s %5s %s\n".as_ptr(), &[QuickJSPrintArg::Str(c"SLOT".as_ptr() as *const c_char), QuickJSPrintArg::Str(c"REFS".as_ptr() as *const c_char), QuickJSPrintArg::Str(c"PROTO".as_ptr() as *const c_char), QuickJSPrintArg::Str(c"SIZE".as_ptr() as *const c_char), QuickJSPrintArg::Str(c"COUNT".as_ptr() as *const c_char), QuickJSPrintArg::Str(c"PROPS".as_ptr() as *const c_char)]));
vm_block = 17; continue;
}
// C line 5590
19 => {
let _ = quickjs_debug_write(&quickjs_format_print(c"JSShapes: {\n".as_ptr(), &[]));
vm_block = 18; continue;
}
_ => std::process::abort(),
} }
}
