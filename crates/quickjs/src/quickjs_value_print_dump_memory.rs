// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:7224. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
pub unsafe fn JS_DumpMemoryUsage(mut fp: *mut QuickJSPrintStream, mut s: *const JSMemoryUsage, mut rt: *mut JSRuntime) -> () {
let mut vm_local_storage = Vec::<u64>::new();
let mut object_types: [QuickJSMemoryObjectType; 5] = core::mem::zeroed();
let mut i: i32 = core::mem::zeroed();
let mut usage_size_ok: i32 = core::mem::zeroed();
let mut size: u32 = core::mem::zeroed();
let mut p: *mut c_void = core::mem::zeroed();
let mut size1: u32 = core::mem::zeroed();
let mut obj_classes: [i32; 64] = core::mem::zeroed();
let mut class_id: i32 = core::mem::zeroed();
let mut el: *mut list_head = core::mem::zeroed();
let mut gp: *mut JSGCObjectHeader = core::mem::zeroed();
let mut p_1: *mut JSObject = core::mem::zeroed();
let mut buf: [c_char; 64] = core::mem::zeroed();
let mut vm_block: usize = 62;
loop { match vm_block {
// C line ?
0 => {
return ();
}
// C line 7344
1 => {
let _ = quickjs_stream_write(fp as *mut QuickJSPrintStream, &quickjs_format_print(c"%-20s %8lld %8lld\n".as_ptr(), &[QuickJSPrintArg::Str(c"binary objects".as_ptr() as *const c_char), QuickJSPrintArg::Int((*(s)).binary_object_count as u64), QuickJSPrintArg::Int((*(s)).binary_object_size as u64)]));
vm_block = 0; continue;
}
// C line 7343
2 => {
vm_block = if ((*(s)).binary_object_count) != 0 { 1 } else { 0 }; continue;
}
// C line 7337
3 => {
let _ = quickjs_stream_write(fp as *mut QuickJSPrintStream, &quickjs_format_print(c"%-20s %8lld %8lld  (%0.1f per fast array)\n".as_ptr(), &[QuickJSPrintArg::Str(c"  elements".as_ptr() as *const c_char), QuickJSPrintArg::Int((*(s)).fast_array_elements as u64), QuickJSPrintArg::Int(((*(s)).fast_array_elements).wrapping_mul((((((size_of::<JSValue>() as usize)) as i32)) as i64)) as u64), QuickJSPrintArg::Float((((((*(s)).fast_array_elements) as f64)) / ((((*(s)).fast_array_count) as f64))) as f64)]));
vm_block = 2; continue;
}
// C line 7336
4 => {
let _ = quickjs_stream_write(fp as *mut QuickJSPrintStream, &quickjs_format_print(c"%-20s %8lld\n".as_ptr(), &[QuickJSPrintArg::Str(c"  fast arrays".as_ptr() as *const c_char), QuickJSPrintArg::Int((*(s)).fast_array_count as u64)]));
vm_block = 3; continue;
}
// C line 7335
5 => {
vm_block = if ((*(s)).fast_array_count) != 0 { 4 } else { 2 }; continue;
}
// C line 7334
6 => {
let _ = quickjs_stream_write(fp as *mut QuickJSPrintStream, &quickjs_format_print(c"%-20s %8lld\n".as_ptr(), &[QuickJSPrintArg::Str(c"arrays".as_ptr() as *const c_char), QuickJSPrintArg::Int((*(s)).array_count as u64)]));
vm_block = 5; continue;
}
// C line 7333
7 => {
vm_block = if ((*(s)).array_count) != 0 { 6 } else { 2 }; continue;
}
// C line 7331
8 => {
let _ = quickjs_stream_write(fp as *mut QuickJSPrintStream, &quickjs_format_print(c"%-20s %8lld\n".as_ptr(), &[QuickJSPrintArg::Str(c"C functions".as_ptr() as *const c_char), QuickJSPrintArg::Int((*(s)).c_func_count as u64)]));
vm_block = 7; continue;
}
// C line 7330
9 => {
vm_block = if ((*(s)).c_func_count) != 0 { 8 } else { 7 }; continue;
}
// C line 7324
10 => {
let _ = quickjs_stream_write(fp as *mut QuickJSPrintStream, &quickjs_format_print(c"%-20s %8lld %8lld  (%0.1f per function)\n".as_ptr(), &[QuickJSPrintArg::Str(c"  pc2line".as_ptr() as *const c_char), QuickJSPrintArg::Int((*(s)).js_func_pc2line_count as u64), QuickJSPrintArg::Int((*(s)).js_func_pc2line_size as u64), QuickJSPrintArg::Float((((((*(s)).js_func_pc2line_size) as f64)) / ((((*(s)).js_func_pc2line_count) as f64))) as f64)]));
vm_block = 9; continue;
}
// C line 7323
11 => {
vm_block = if ((*(s)).js_func_pc2line_count) != 0 { 10 } else { 9 }; continue;
}
// C line 7320
12 => {
let _ = quickjs_stream_write(fp as *mut QuickJSPrintStream, &quickjs_format_print(c"%-20s %8lld %8lld  (%0.1f per function)\n".as_ptr(), &[QuickJSPrintArg::Str(c"  bytecode".as_ptr() as *const c_char), QuickJSPrintArg::Int((*(s)).js_func_count as u64), QuickJSPrintArg::Int((*(s)).js_func_code_size as u64), QuickJSPrintArg::Float((((((*(s)).js_func_code_size) as f64)) / ((((*(s)).js_func_count) as f64))) as f64)]));
vm_block = 11; continue;
}
// C line 7318
13 => {
let _ = quickjs_stream_write(fp as *mut QuickJSPrintStream, &quickjs_format_print(c"%-20s %8lld %8lld\n".as_ptr(), &[QuickJSPrintArg::Str(c"bytecode functions".as_ptr() as *const c_char), QuickJSPrintArg::Int((*(s)).js_func_count as u64), QuickJSPrintArg::Int((*(s)).js_func_size as u64)]));
vm_block = 12; continue;
}
// C line 7317
14 => {
vm_block = if ((*(s)).js_func_count) != 0 { 13 } else { 9 }; continue;
}
// C line 7313
15 => {
let _ = quickjs_stream_write(fp as *mut QuickJSPrintStream, &quickjs_format_print(c"%-20s %8lld %8lld  (%0.1f per shape)\n".as_ptr(), &[QuickJSPrintArg::Str(c"  shapes".as_ptr() as *const c_char), QuickJSPrintArg::Int((*(s)).shape_count as u64), QuickJSPrintArg::Int((*(s)).shape_size as u64), QuickJSPrintArg::Float((((((*(s)).shape_size) as f64)) / ((((*(s)).shape_count) as f64))) as f64)]));
vm_block = 14; continue;
}
// C line 7310
16 => {
let _ = quickjs_stream_write(fp as *mut QuickJSPrintStream, &quickjs_format_print(c"%-20s %8lld %8lld  (%0.1f per object)\n".as_ptr(), &[QuickJSPrintArg::Str(c"  properties".as_ptr() as *const c_char), QuickJSPrintArg::Int((*(s)).prop_count as u64), QuickJSPrintArg::Int((*(s)).prop_size as u64), QuickJSPrintArg::Float((((((*(s)).prop_count) as f64)) / ((((*(s)).obj_count) as f64))) as f64)]));
vm_block = 15; continue;
}
// C line 7307
17 => {
let _ = quickjs_stream_write(fp as *mut QuickJSPrintStream, &quickjs_format_print(c"%-20s %8lld %8lld  (%0.1f per object)\n".as_ptr(), &[QuickJSPrintArg::Str(c"objects".as_ptr() as *const c_char), QuickJSPrintArg::Int((*(s)).obj_count as u64), QuickJSPrintArg::Int((*(s)).obj_size as u64), QuickJSPrintArg::Float((((((*(s)).obj_size) as f64)) / ((((*(s)).obj_count) as f64))) as f64)]));
vm_block = 16; continue;
}
// C line 7306
18 => {
vm_block = if ((*(s)).obj_count) != 0 { 17 } else { 14 }; continue;
}
// C line 7302
19 => {
let _ = quickjs_stream_write(fp as *mut QuickJSPrintStream, &quickjs_format_print(c"%-20s %8lld %8lld  (%0.1f per string)\n".as_ptr(), &[QuickJSPrintArg::Str(c"strings".as_ptr() as *const c_char), QuickJSPrintArg::Int((*(s)).str_count as u64), QuickJSPrintArg::Int((*(s)).str_size as u64), QuickJSPrintArg::Float((((((*(s)).str_size) as f64)) / ((((*(s)).str_count) as f64))) as f64)]));
vm_block = 18; continue;
}
// C line 7301
20 => {
vm_block = if ((*(s)).str_count) != 0 { 19 } else { 18 }; continue;
}
// C line 7297
21 => {
let _ = quickjs_stream_write(fp as *mut QuickJSPrintStream, &quickjs_format_print(c"%-20s %8lld %8lld  (%0.1f per atom)\n".as_ptr(), &[QuickJSPrintArg::Str(c"atoms".as_ptr() as *const c_char), QuickJSPrintArg::Int((*(s)).atom_count as u64), QuickJSPrintArg::Int((*(s)).atom_size as u64), QuickJSPrintArg::Float((((((*(s)).atom_size) as f64)) / ((((*(s)).atom_count) as f64))) as f64)]));
vm_block = 20; continue;
}
// C line 7296
22 => {
vm_block = if ((*(s)).atom_count) != 0 { 21 } else { 20 }; continue;
}
// C line 7291
23 => {
let _ = quickjs_stream_write(fp as *mut QuickJSPrintStream, &quickjs_format_print(c"%-20s %8lld %8lld  (%d overhead, %0.1f average slack)\n".as_ptr(), &[QuickJSPrintArg::Str(c"memory used".as_ptr() as *const c_char), QuickJSPrintArg::Int((*(s)).memory_used_count as u64), QuickJSPrintArg::Int((*(s)).memory_used_size as u64), QuickJSPrintArg::Int((0 as i32) as u64), QuickJSPrintArg::Float(((((((*(s)).malloc_size).wrapping_sub((*(s)).memory_used_size)) as f64)) / ((((*(s)).memory_used_count) as f64))) as f64)]));
vm_block = 22; continue;
}
// C line 7288
24 => {
let _ = quickjs_stream_write(fp as *mut QuickJSPrintStream, &quickjs_format_print(c"%-20s %8lld %8lld  (%0.1f per block)\n".as_ptr(), &[QuickJSPrintArg::Str(c"memory allocated".as_ptr() as *const c_char), QuickJSPrintArg::Int((*(s)).malloc_count as u64), QuickJSPrintArg::Int((*(s)).malloc_size as u64), QuickJSPrintArg::Float((((((*(s)).malloc_size) as f64)) / ((((*(s)).malloc_count) as f64))) as f64)]));
vm_block = 23; continue;
}
// C line 7287
25 => {
vm_block = if ((*(s)).malloc_count) != 0 { 24 } else { 22 }; continue;
}
// C line 7285
26 => {
let _ = quickjs_stream_write(fp as *mut QuickJSPrintStream, &quickjs_format_print(c"%-20s %8s %8s\n".as_ptr(), &[QuickJSPrintArg::Str(c"NAME".as_ptr() as *const c_char), QuickJSPrintArg::Str(c"COUNT".as_ptr() as *const c_char), QuickJSPrintArg::Str(c"SIZE".as_ptr() as *const c_char)]));
vm_block = 25; continue;
}
// C line 7282
27 => {
let _ = quickjs_stream_write(fp as *mut QuickJSPrintStream, &quickjs_format_print(c"\n".as_ptr(), &[]));
vm_block = 26; continue;
}
// C line 7280
28 => {
let _ = quickjs_stream_write(fp as *mut QuickJSPrintStream, &quickjs_format_print(c"  %5d  %2.0d %s\n".as_ptr(), &[QuickJSPrintArg::Int(*((obj_classes).as_mut_ptr()).offset(((JS_CLASS_INIT_COUNT as i32)) as isize) as u64), QuickJSPrintArg::Int((0 as i32) as u64), QuickJSPrintArg::Str(c"other".as_ptr() as *const c_char)]));
vm_block = 27; continue;
}
// C line 7279
29 => {
vm_block = if (*((obj_classes).as_mut_ptr()).offset(((JS_CLASS_INIT_COUNT as i32)) as isize)) != 0 { 28 } else { 27 }; continue;
}
// C line 7272
30 => {
vm_block = if ((((class_id) < ((JS_CLASS_INIT_COUNT as i32))) as i32)) != 0 { 33 } else { 29 }; continue;
}
// C line 7272
31 => {
let _ = { let old = class_id; class_id = (class_id).wrapping_add(1); old };
vm_block = 30; continue;
}
// C line 7275
32 => {
let _ = quickjs_stream_write(fp as *mut QuickJSPrintStream, &quickjs_format_print(c"  %5d  %2.0d %s\n".as_ptr(), &[QuickJSPrintArg::Int(*((obj_classes).as_mut_ptr()).offset((class_id) as isize) as u64), QuickJSPrintArg::Int(class_id as u64), QuickJSPrintArg::Str(JS_AtomGetStrRT(rt, (buf).as_mut_ptr(), (((size_of::<[c_char; 64]>() as usize)) as i32), (*((*(rt)).class_array).offset((class_id) as isize)).class_name) as *const c_char)]));
vm_block = 31; continue;
}
// C line 7273
33 => {
vm_block = if (((((*((obj_classes).as_mut_ptr()).offset((class_id) as isize)) != 0) && (((((class_id) < ((*(rt)).class_count)) as i32)) != 0)) as i32)) != 0 { 32 } else { 31 }; continue;
}
// C line 7272
34 => {
let _ = { let assigned = (1 as i32); class_id = assigned; assigned };
vm_block = 30; continue;
}
// C line 7271
35 => {
let _ = quickjs_stream_write(fp as *mut QuickJSPrintStream, &quickjs_format_print(c"  %5d  %2.0d %s\n".as_ptr(), &[QuickJSPrintArg::Int(*((obj_classes).as_mut_ptr()).offset(((0 as i32)) as isize) as u64), QuickJSPrintArg::Int((0 as i32) as u64), QuickJSPrintArg::Str(c"none".as_ptr() as *const c_char)]));
vm_block = 34; continue;
}
// C line 7270
36 => {
vm_block = if (*((obj_classes).as_mut_ptr()).offset(((0 as i32)) as isize)) != 0 { 35 } else { 34 }; continue;
}
// C line 7269
37 => {
let _ = quickjs_stream_write(fp as *mut QuickJSPrintStream, &quickjs_format_print(c"\nJSObject classes\n".as_ptr(), &[]));
vm_block = 36; continue;
}
// C line 7261
38 => {
vm_block = if ((((el) != (core::ptr::addr_of_mut!((*(rt)).gc_obj_list))) as i32)) != 0 { 43 } else { 37 }; continue;
}
// C line 7261
39 => {
let _ = { let assigned = (*(el)).next; el = assigned; assigned };
vm_block = 38; continue;
}
// C line 7266
40 => {
let _ = { let old = *((obj_classes).as_mut_ptr()).offset((crate::cutils_header::min_uint32((((*(p_1)).class_id) as u32), (((JS_CLASS_INIT_COUNT as i32)) as u32))) as isize); *((obj_classes).as_mut_ptr()).offset((crate::cutils_header::min_uint32((((*(p_1)).class_id) as u32), (((JS_CLASS_INIT_COUNT as i32)) as u32))) as isize) = (*((obj_classes).as_mut_ptr()).offset((crate::cutils_header::min_uint32((((*(p_1)).class_id) as u32), (((JS_CLASS_INIT_COUNT as i32)) as u32))) as isize)).wrapping_add(1); old };
vm_block = 39; continue;
}
// C line 7265
41 => {
let _ = { let assigned = ((gp) as *mut JSObject); p_1 = assigned; assigned };
vm_block = 40; continue;
}
// C line 7264
42 => {
vm_block = if (((((((*(js_rc(((gp) as *mut c_void)))).gc_obj_type()) as i32)) == ((JS_GC_OBJ_TYPE_JS_OBJECT as i32))) as i32)) != 0 { 41 } else { 39 }; continue;
}
// C line 7262
43 => {
gp = (((((el) as *mut u8)).offset(-(((core::mem::offset_of!(JSGCObjectHeader, link) as usize)) as isize))) as *mut JSGCObjectHeader);
vm_block = 42; continue;
}
// C line 7261
44 => {
let _ = { let assigned = (*(core::ptr::addr_of_mut!((*(rt)).gc_obj_list))).next; el = assigned; assigned };
vm_block = 38; continue;
}
// C line 7258
45 => {
obj_classes = [(0 as i32), core::mem::zeroed(), core::mem::zeroed(), core::mem::zeroed(), core::mem::zeroed(), core::mem::zeroed(), core::mem::zeroed(), core::mem::zeroed(), core::mem::zeroed(), core::mem::zeroed(), core::mem::zeroed(), core::mem::zeroed(), core::mem::zeroed(), core::mem::zeroed(), core::mem::zeroed(), core::mem::zeroed(), core::mem::zeroed(), core::mem::zeroed(), core::mem::zeroed(), core::mem::zeroed(), core::mem::zeroed(), core::mem::zeroed(), core::mem::zeroed(), core::mem::zeroed(), core::mem::zeroed(), core::mem::zeroed(), core::mem::zeroed(), core::mem::zeroed(), core::mem::zeroed(), core::mem::zeroed(), core::mem::zeroed(), core::mem::zeroed(), core::mem::zeroed(), core::mem::zeroed(), core::mem::zeroed(), core::mem::zeroed(), core::mem::zeroed(), core::mem::zeroed(), core::mem::zeroed(), core::mem::zeroed(), core::mem::zeroed(), core::mem::zeroed(), core::mem::zeroed(), core::mem::zeroed(), core::mem::zeroed(), core::mem::zeroed(), core::mem::zeroed(), core::mem::zeroed(), core::mem::zeroed(), core::mem::zeroed(), core::mem::zeroed(), core::mem::zeroed(), core::mem::zeroed(), core::mem::zeroed(), core::mem::zeroed(), core::mem::zeroed(), core::mem::zeroed(), core::mem::zeroed(), core::mem::zeroed(), core::mem::zeroed(), core::mem::zeroed(), core::mem::zeroed(), core::mem::zeroed(), core::mem::zeroed()];
vm_block = 44; continue;
}
// C line 7255
46 => {
let _ = quickjs_stream_write(fp as *mut QuickJSPrintStream, &quickjs_format_print(c"  malloc_usable_size unavailable\n".as_ptr(), &[]));
vm_block = 45; continue;
}
// C line 7254
47 => {
vm_block = if ((!((usage_size_ok) != 0) as i32)) != 0 { 46 } else { 45 }; continue;
}
// C line 7241
48 => {
vm_block = if ((((((i) as usize)) < ((((size_of::<[QuickJSMemoryObjectType; 5]>() as usize)) / ((size_of::<QuickJSMemoryObjectType>() as usize))))) as i32)) != 0 { 57 } else { 47 }; continue;
}
// C line 7241
49 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 48; continue;
}
// C line 7251
50 => {
let _ = js_free_rt(rt, p);
vm_block = 49; continue;
}
// C line 7248
51 => {
let _ = quickjs_stream_write(fp as *mut QuickJSPrintStream, &quickjs_format_print(c"  %3u + %-2u  %s\n".as_ptr(), &[QuickJSPrintArg::Int(size as u64), QuickJSPrintArg::Int((size1).wrapping_sub(size) as u64), QuickJSPrintArg::Str((*((object_types).as_ptr()).offset((i) as isize)).name as *const c_char)]));
vm_block = 50; continue;
}
// C line 7247
52 => {
let _ = { let assigned = (1 as i32); usage_size_ok = assigned; assigned };
vm_block = 51; continue;
}
// C line 7246
53 => {
vm_block = if ((((size1) >= (size)) as i32)) != 0 { 52 } else { 50 }; continue;
}
// C line 7245
54 => {
size1 = ((js_malloc_usable_size_rt(rt, p)) as u32);
vm_block = 53; continue;
}
// C line 7244
55 => {
vm_block = if !(p).is_null() { 54 } else { 49 }; continue;
}
// C line 7243
56 => {
p = js_malloc_rt(rt, ((size) as usize));
vm_block = 55; continue;
}
// C line 7242
57 => {
size = (((*((object_types).as_ptr()).offset((i) as isize)).size) as u32);
vm_block = 56; continue;
}
// C line 7241
58 => {
let _ = { let assigned = (0 as i32); i = assigned; assigned };
vm_block = 48; continue;
}
// C line 7240
59 => {
usage_size_ok = (0 as i32);
vm_block = 58; continue;
}
// C line 7230
60 => {
object_types = [QuickJSMemoryObjectType {name: c"JSRuntime".as_ptr(), size: (size_of::<JSRuntime>() as usize)}, QuickJSMemoryObjectType {name: c"JSContext".as_ptr(), size: (size_of::<JSContext>() as usize)}, QuickJSMemoryObjectType {name: c"JSObject".as_ptr(), size: (size_of::<JSObject>() as usize)}, QuickJSMemoryObjectType {name: c"JSString".as_ptr(), size: (size_of::<JSString>() as usize)}, QuickJSMemoryObjectType {name: c"JSFunctionBytecode".as_ptr(), size: (size_of::<JSFunctionBytecode>() as usize)}];
vm_block = 59; continue;
}
// C line 7229
61 => {
vm_block = if !(rt).is_null() { 60 } else { 26 }; continue;
}
// C line 7226
62 => {
let _ = quickjs_stream_write(fp as *mut QuickJSPrintStream, &quickjs_format_print(c"QuickJS memory usage -- 2026-06-04 version, %d-bit, malloc limit: %lld\n\n".as_ptr(), &[QuickJSPrintArg::Int(((((size_of::<*mut c_void>() as usize)) as i32)).wrapping_mul((8 as i32)) as u64), QuickJSPrintArg::Int((*(s)).malloc_limit as u64)]));
vm_block = 61; continue;
}
_ => std::process::abort(),
} }
}
