// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:14484. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn JS_DumpObjectHeader(mut rt: *mut JSRuntime) -> () {
let mut vm_local_storage = Vec::<u64>::new();
let mut vm_block: usize = 1;
loop { match vm_block {
// C line ?
0 => {
return ();
}
// C line 14486
1 => {
let _ = quickjs_debug_write(&quickjs_format_print(c"%14s %4s %4s %14s %s\n".as_ptr(), &[QuickJSPrintArg::Str(c"ADDRESS".as_ptr() as *const c_char), QuickJSPrintArg::Str(c"REFS".as_ptr() as *const c_char), QuickJSPrintArg::Str(c"SHRF".as_ptr() as *const c_char), QuickJSPrintArg::Str(c"PROTO".as_ptr() as *const c_char), QuickJSPrintArg::Str(c"CONTENT".as_ptr() as *const c_char)]));
vm_block = 0; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:14491. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn JS_DumpObject(mut rt: *mut JSRuntime, mut p: *mut JSObject) -> () {
let mut vm_local_storage = Vec::<u64>::new();
let mut sh: *mut JSShape = core::mem::zeroed();
let mut options: JSPrintValueOptions = core::mem::zeroed();
let mut vm_block: usize = 11;
loop { match vm_block {
// C line ?
0 => {
return ();
}
// C line 14516
1 => {
let _ = quickjs_debug_write(&quickjs_format_print(c"\n".as_ptr(), &[]));
vm_block = 0; continue;
}
// C line 14514
2 => {
let _ = JS_PrintValueRT(rt, Some(js_dump_value_write as JSPrintValueWrite), ((core::ptr::null_mut::<c_void>()) as *mut c_void), JSValue { u: JSValueUnion { ptr: ((p) as *mut c_void) }, tag: (((JS_TAG_OBJECT as i32)) as i64) }, core::ptr::addr_of_mut!(options));
vm_block = 1; continue;
}
// C line 14513
3 => {
let _ = { let assigned = (1 as i32); (options).set_raw_dump((assigned) as _); assigned };
vm_block = 2; continue;
}
// C line 14512
4 => {
let _ = { let assigned = (1 as i32); (options).set_show_hidden((assigned) as _); assigned };
vm_block = 3; continue;
}
// C line 14511
5 => {
let _ = { let assigned = (((1 as i32)) as u32); (options).max_depth = assigned; assigned };
vm_block = 4; continue;
}
// C line 14510
6 => {
let _ = JS_PrintValueSetDefaultOptions(core::ptr::addr_of_mut!(options));
vm_block = 5; continue;
}
// C line 14502
7 => {
let _ = quickjs_debug_write(&quickjs_format_print(c"%3d%c %14p ".as_ptr(), &[QuickJSPrintArg::Int((*(js_rc(((sh) as *mut c_void)))).ref_count as u64), QuickJSPrintArg::Int(((*(c" *".as_ptr()).offset(((*(sh)).is_hashed) as isize)) as i32) as u64), QuickJSPrintArg::Ptr((((*(sh)).proto) as *mut c_void) as *const c_void)]));
vm_block = 6; continue;
}
// C line 14507
8 => {
let _ = quickjs_debug_write(&quickjs_format_print(c"%3s  %14s ".as_ptr(), &[QuickJSPrintArg::Str(c"-".as_ptr() as *const c_char), QuickJSPrintArg::Str(c"-".as_ptr() as *const c_char)]));
vm_block = 6; continue;
}
// C line 14501
9 => {
vm_block = if !(sh).is_null() { 7 } else { 8 }; continue;
}
// C line 14498
10 => {
let _ = quickjs_debug_write(&quickjs_format_print(c"%14p %4d ".as_ptr(), &[QuickJSPrintArg::Ptr(((p) as *mut c_void) as *const c_void), QuickJSPrintArg::Int((*(js_rc(((p) as *mut c_void)))).ref_count as u64)]));
vm_block = 9; continue;
}
// C line 14497
11 => {
let _ = { let assigned = (*(p)).shape; sh = assigned; assigned };
vm_block = 10; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:14519. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn JS_DumpGCObject(mut rt: *mut JSRuntime, mut p: *mut JSGCObjectHeader) -> () {
let mut vm_local_storage = Vec::<u64>::new();
let mut vm_block: usize = 19;
loop { match vm_block {
// C line ?
0 => {
return ();
}
// C line 14522
1 => {
let _ = JS_DumpObject(rt, ((p) as *mut JSObject));
vm_block = 0; continue;
}
// C line 14550
2 => {
let _ = quickjs_debug_write(&quickjs_format_print(c"\n".as_ptr(), &[]));
vm_block = 0; continue;
}
// C line 14548
3 => {
vm_block = 2; continue;
}
// C line 14547
4 => {
let _ = quickjs_debug_write(&quickjs_format_print(c"[unknown %d]".as_ptr(), &[QuickJSPrintArg::Int((((*(js_rc(((p) as *mut c_void)))).gc_obj_type()) as i32) as u64)]));
vm_block = 3; continue;
}
// C line 14545
5 => {
vm_block = 2; continue;
}
// C line 14544
6 => {
let _ = quickjs_debug_write(&quickjs_format_print(c"[module]".as_ptr(), &[]));
vm_block = 5; continue;
}
// C line 14542
7 => {
vm_block = 2; continue;
}
// C line 14541
8 => {
let _ = quickjs_debug_write(&quickjs_format_print(c"[js_context]".as_ptr(), &[]));
vm_block = 7; continue;
}
// C line 14539
9 => {
vm_block = 2; continue;
}
// C line 14538
10 => {
let _ = quickjs_debug_write(&quickjs_format_print(c"[async_function]".as_ptr(), &[]));
vm_block = 9; continue;
}
// C line 14536
11 => {
vm_block = 2; continue;
}
// C line 14535
12 => {
let _ = quickjs_debug_write(&quickjs_format_print(c"[var_ref]".as_ptr(), &[]));
vm_block = 11; continue;
}
// C line 14533
13 => {
vm_block = 2; continue;
}
// C line 14532
14 => {
let _ = quickjs_debug_write(&quickjs_format_print(c"[shape]".as_ptr(), &[]));
vm_block = 13; continue;
}
// C line 14530
15 => {
vm_block = 2; continue;
}
// C line 14529
16 => {
let _ = quickjs_debug_write(&quickjs_format_print(c"[function bytecode]".as_ptr(), &[]));
vm_block = 15; continue;
}
// C line 14527
17 => {
vm_block = match (((*(js_rc(((p) as *mut c_void)))).gc_obj_type()) as i32) { x if x == (JS_GC_OBJ_TYPE_MODULE as i32) => 6, x if x == (JS_GC_OBJ_TYPE_JS_CONTEXT as i32) => 8, x if x == (JS_GC_OBJ_TYPE_ASYNC_FUNCTION as i32) => 10, x if x == (JS_GC_OBJ_TYPE_VAR_REF as i32) => 12, x if x == (JS_GC_OBJ_TYPE_SHAPE as i32) => 14, x if x == (JS_GC_OBJ_TYPE_FUNCTION_BYTECODE as i32) => 16, _ => 4, }; continue;
}
// C line 14524
18 => {
let _ = quickjs_debug_write(&quickjs_format_print(c"%14p %4d ".as_ptr(), &[QuickJSPrintArg::Ptr(((p) as *mut c_void) as *const c_void), QuickJSPrintArg::Int((*(js_rc(((p) as *mut c_void)))).ref_count as u64)]));
vm_block = 17; continue;
}
// C line 14521
19 => {
vm_block = if (((((((*(js_rc(((p) as *mut c_void)))).gc_obj_type()) as i32)) == ((JS_GC_OBJ_TYPE_JS_OBJECT as i32))) as i32)) != 0 { 1 } else { 18 }; continue;
}
_ => std::process::abort(),
} }
}
