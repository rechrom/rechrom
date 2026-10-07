// quickjs.c:6853-7222. Original memory accounting algorithms. MIT.
#[repr(C)]
struct JSMemoryUsage_helper {
    memory_used_count: f64,
    str_count: f64,
    str_size: f64,
    js_func_count: i64,
    js_func_size: f64,
    js_func_code_size: i64,
    js_func_pc2line_count: i64,
    js_func_pc2line_size: i64,
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:6867. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn compute_jsstring_size(mut str: *mut JSString, mut hp: *mut JSMemoryUsage_helper) -> () {
let mut vm_local_storage = Vec::<u64>::new();
let mut s_ref_count: f64 = core::mem::zeroed();
let mut vm_block: usize = 4;
loop { match vm_block {
// C line ?
0 => {
return ();
}
// C line 6872
1 => {
let _ = { let assigned = (((((*(hp)).str_size) as f64)+((((((((((size_of::<JSString>() as usize)).wrapping_add(((((((*(str)).len()) as i32)).wrapping_shl(((((*(str)).is_wide_char()) as i32)) as u32)) as usize))).wrapping_add((((1 as i32)) as usize))).wrapping_sub((((*(str)).is_wide_char()) as usize))) as f64)) / (s_ref_count))) as f64))) as f64; (*(hp)).str_size = assigned; assigned };
vm_block = 0; continue;
}
// C line 6871
2 => {
let _ = { let assigned = (((((*(hp)).str_count) as f64)+(((((((1 as i32)) as f64)) / (s_ref_count))) as f64))) as f64; (*(hp)).str_count = assigned; assigned };
vm_block = 1; continue;
}
// C line 6870
3 => {
s_ref_count = (((*(js_rc(((str) as *mut c_void)))).ref_count) as f64);
vm_block = 2; continue;
}
// C line 6869
4 => {
vm_block = if ((!(((*(str)).atom_type()) != 0) as i32)) != 0 { 3 } else { 0 }; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:6877. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn compute_bytecode_size(mut b: *mut JSFunctionBytecode, mut hp: *mut JSMemoryUsage_helper) -> () {
let mut vm_local_storage = Vec::<u64>::new();
let mut memory_used_count: i32 = core::mem::zeroed();
let mut js_func_size: i32 = core::mem::zeroed();
let mut i: i32 = core::mem::zeroed();
let mut val: JSValue = core::mem::zeroed();
let mut vm_block: usize = 27;
loop { match vm_block {
// C line ?
0 => {
return ();
}
// C line 6913
1 => {
let _ = { let assigned = (((((*(hp)).memory_used_count) as f64)+((((memory_used_count) as f64)) as f64))) as f64; (*(hp)).memory_used_count = assigned; assigned };
vm_block = 0; continue;
}
// C line 6912
2 => {
let _ = { let assigned = ((((*(hp)).js_func_count) as i64).wrapping_add((((((1 as i32)) as i64)) as i64))) as i64; (*(hp)).js_func_count = assigned; assigned };
vm_block = 1; continue;
}
// C line 6911
3 => {
let _ = { let assigned = (((((*(hp)).js_func_size) as f64)+((((js_func_size) as f64)) as f64))) as f64; (*(hp)).js_func_size = assigned; assigned };
vm_block = 2; continue;
}
// C line 6908
4 => {
let _ = { let assigned = ((((*(hp)).js_func_pc2line_size) as i64).wrapping_add(((((((*(b)).debug).pc2line_len) as i64)) as i64))) as i64; (*(hp)).js_func_pc2line_size = assigned; assigned };
vm_block = 3; continue;
}
// C line 6907
5 => {
let _ = { let assigned = ((((*(hp)).js_func_pc2line_count) as i64).wrapping_add((((((1 as i32)) as i64)) as i64))) as i64; (*(hp)).js_func_pc2line_count = assigned; assigned };
vm_block = 4; continue;
}
// C line 6906
6 => {
let _ = { let old = memory_used_count; memory_used_count = (memory_used_count).wrapping_add(1); old };
vm_block = 5; continue;
}
// C line 6905
7 => {
vm_block = if (((*(b)).debug).pc2line_len) != 0 { 6 } else { 3 }; continue;
}
// C line 6903
8 => {
let _ = { let assigned = (((js_func_size) as i32).wrapping_add((((((*(b)).debug).source_len).wrapping_add((1 as i32))) as i32))) as i32; js_func_size = assigned; assigned };
vm_block = 7; continue;
}
// C line 6902
9 => {
let _ = { let old = memory_used_count; memory_used_count = (memory_used_count).wrapping_add(1); old };
vm_block = 8; continue;
}
// C line 6901
10 => {
vm_block = if !(((*(b)).debug).source).is_null() { 9 } else { 7 }; continue;
}
// C line 6900
11 => {
let _ = { let assigned = (((js_func_size) as usize).wrapping_add(((((size_of::<JSFunctionBytecode>() as usize)).wrapping_sub(offset_of!(JSFunctionBytecode, debug))) as usize))) as i32; js_func_size = assigned; assigned };
vm_block = 10; continue;
}
// C line 6899
12 => {
vm_block = if ((*(b)).has_debug()) != 0 { 11 } else { 3 }; continue;
}
// C line 6897
13 => {
let _ = { let assigned = ((((*(hp)).js_func_code_size) as i64).wrapping_add((((((*(b)).byte_code_len) as i64)) as i64))) as i64; (*(hp)).js_func_code_size = assigned; assigned };
vm_block = 12; continue;
}
// C line 6896
14 => {
vm_block = if ((((((!(((*(b)).read_only_bytecode()) != 0) as i32)) != 0) && (!((*(b)).byte_code_buf).is_null())) as i32)) != 0 { 13 } else { 12 }; continue;
}
// C line 6894
15 => {
let _ = { let assigned = (((js_func_size) as usize).wrapping_add(((((((*(b)).closure_var_count) as usize)).wrapping_mul((size_of::<JSClosureVar>() as usize))) as usize))) as i32; js_func_size = assigned; assigned };
vm_block = 14; continue;
}
// C line 6893
16 => {
vm_block = if !((*(b)).closure_var).is_null() { 15 } else { 14 }; continue;
}
// C line 6888
17 => {
vm_block = if ((((i) < ((*(b)).cpool_count)) as i32)) != 0 { 20 } else { 16 }; continue;
}
// C line ?
18 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 17; continue;
}
// C line 6890
19 => {
let _ = compute_value_size(val, hp);
vm_block = 18; continue;
}
// C line 6889
20 => {
val = *((*(b)).cpool).offset((i) as isize);
vm_block = 19; continue;
}
// C line 6888
21 => {
let _ = { let assigned = (0 as i32); i = assigned; assigned };
vm_block = 17; continue;
}
// C line 6887
22 => {
let _ = { let assigned = (((js_func_size) as usize).wrapping_add(((((((*(b)).cpool_count) as usize)).wrapping_mul((size_of::<JSValue>() as usize))) as usize))) as i32; js_func_size = assigned; assigned };
vm_block = 21; continue;
}
// C line 6886
23 => {
vm_block = if !((*(b)).cpool).is_null() { 22 } else { 16 }; continue;
}
// C line 6884
24 => {
let _ = { let assigned = (((js_func_size) as usize).wrapping_add((((((((((*(b)).arg_count) as i32)).wrapping_add((((*(b)).var_count) as i32))) as usize)).wrapping_mul((size_of::<JSBytecodeVarDef>() as usize))) as usize))) as i32; js_func_size = assigned; assigned };
vm_block = 23; continue;
}
// C line 6883
25 => {
vm_block = if !((*(b)).vardefs).is_null() { 24 } else { 23 }; continue;
}
// C line 6882
26 => {
let _ = { let assigned = ((offset_of!(JSFunctionBytecode, debug)) as i32); js_func_size = assigned; assigned };
vm_block = 25; continue;
}
// C line 6881
27 => {
let _ = { let assigned = (0 as i32); memory_used_count = assigned; assigned };
vm_block = 26; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:6916. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn compute_value_size(mut val: JSValue, mut hp: *mut JSMemoryUsage_helper) -> () {
let mut vm_local_storage = Vec::<u64>::new();
let mut vm_block: usize = 4;
loop { match vm_block {
// C line ?
0 => {
return ();
}
// C line 6924
1 => {
vm_block = 0; continue;
}
// C line 6921
2 => {
vm_block = 0; continue;
}
// C line 6920
3 => {
let _ = compute_jsstring_size(((((val).u).ptr) as *mut JSString), hp);
vm_block = 2; continue;
}
// C line 6918
4 => {
vm_block = match (((val).tag) as i32) { x if x == (JS_TAG_BIG_INT as i32) => 1, x if x == (JS_TAG_STRING as i32) => 3, _ => 0, }; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:6928. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
pub unsafe fn JS_ComputeMemoryUsage(mut rt: *mut JSRuntime, mut s: *mut JSMemoryUsage) -> () {
let mut vm_local_storage = Vec::<u64>::new();
let mut el: *mut list_head = core::mem::zeroed();
let mut el1: *mut list_head = core::mem::zeroed();
let mut i: i32 = core::mem::zeroed();
let mut mem: JSMemoryUsage_helper = core::mem::zeroed();
let mut hp: *mut JSMemoryUsage_helper = core::mem::zeroed();
let mut ctx: *mut JSContext = core::mem::zeroed();
let mut sh: *mut JSShape = core::mem::zeroed();
let mut hash_size: i32 = core::mem::zeroed();
let mut m: *mut JSModuleDef = core::mem::zeroed();
let mut me: *mut JSExportEntry = core::mem::zeroed();
let mut gp: *mut JSGCObjectHeader = core::mem::zeroed();
let mut p: *mut JSObject = core::mem::zeroed();
let mut sh_1: *mut JSShape = core::mem::zeroed();
let mut prs: *mut JSShapeProperty = core::mem::zeroed();
let mut pr: *mut JSProperty = core::mem::zeroed();
let mut hash_size_1: i32 = core::mem::zeroed();
let mut b: *mut JSFunctionBytecode = core::mem::zeroed();
let mut var_refs: *mut *mut JSVarRef = core::mem::zeroed();
let mut ref_count: f64 = core::mem::zeroed();
let mut bf: *mut JSBoundFunction = core::mem::zeroed();
let mut fd: *mut JSCFunctionDataRecord = core::mem::zeroed();
let mut it: *mut JSForInIterator = core::mem::zeroed();
let mut abuf: *mut JSArrayBuffer = core::mem::zeroed();
let mut sh_2: *mut JSShape = core::mem::zeroed();
let mut hash_size_2: i32 = core::mem::zeroed();
let mut p_1: *mut JSAtomStruct = core::mem::zeroed();
let mut vm_block: usize = 185;
loop { match vm_block {
// C line ?
0 => {
return ();
}
// C line 7219
1 => {
let _ = { let assigned = ((((*(s)).memory_used_size) as i64).wrapping_add(((((((((((*(s)).atom_size).wrapping_add((*(s)).str_size)).wrapping_add((*(s)).obj_size)).wrapping_add((*(s)).prop_size)).wrapping_add((*(s)).shape_size)).wrapping_add((*(s)).js_func_size)).wrapping_add((*(s)).js_func_code_size)).wrapping_add((*(s)).js_func_pc2line_size)) as i64))) as i64; (*(s)).memory_used_size = assigned; assigned };
vm_block = 0; continue;
}
// C line 7215
2 => {
let _ = { let assigned = (((((*(s)).memory_used_count) as f64)+((((((((((((((((mem).memory_used_count).round()) + ((((*(s)).atom_count) as f64)))) + ((((*(s)).str_count) as f64)))) + ((((*(s)).obj_count) as f64)))) + ((((*(s)).shape_count) as f64)))) + ((((*(s)).js_func_count) as f64)))) + ((((*(s)).js_func_pc2line_count) as f64)))) as f64))) as i64; (*(s)).memory_used_count = assigned; assigned };
vm_block = 1; continue;
}
// C line 7214
3 => {
let _ = { let assigned = (mem).js_func_pc2line_size; (*(s)).js_func_pc2line_size = assigned; assigned };
vm_block = 2; continue;
}
// C line 7213
4 => {
let _ = { let assigned = (mem).js_func_pc2line_count; (*(s)).js_func_pc2line_count = assigned; assigned };
vm_block = 3; continue;
}
// C line 7212
5 => {
let _ = { let assigned = (mem).js_func_code_size; (*(s)).js_func_code_size = assigned; assigned };
vm_block = 4; continue;
}
// C line 7211
6 => {
let _ = { let assigned = ((((mem).js_func_size).round()) as i64); (*(s)).js_func_size = assigned; assigned };
vm_block = 5; continue;
}
// C line 7210
7 => {
let _ = { let assigned = (mem).js_func_count; (*(s)).js_func_count = assigned; assigned };
vm_block = 6; continue;
}
// C line 7209
8 => {
let _ = { let assigned = ((((mem).str_size).round()) as i64); (*(s)).str_size = assigned; assigned };
vm_block = 7; continue;
}
// C line 7208
9 => {
let _ = { let assigned = ((((mem).str_count).round()) as i64); (*(s)).str_count = assigned; assigned };
vm_block = 8; continue;
}
// C line 7201
10 => {
vm_block = if ((((i) < ((*(rt)).atom_size)) as i32)) != 0 { 14 } else { 9 }; continue;
}
// C line ?
11 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 10; continue;
}
// C line 7204
12 => {
let _ = { let assigned = ((((*(s)).atom_size) as u64).wrapping_add(((((((((size_of::<JSAtomStruct>() as usize)).wrapping_add(((((((*(p_1)).len()) as i32)).wrapping_shl(((((*(p_1)).is_wide_char()) as i32)) as u32)) as usize))).wrapping_add((((1 as i32)) as usize))).wrapping_sub((((*(p_1)).is_wide_char()) as usize))) as u64)) as u64))) as i64; (*(s)).atom_size = assigned; assigned };
vm_block = 11; continue;
}
// C line 7203
13 => {
vm_block = if ((!((atom_is_free(p_1)) != 0) as i32)) != 0 { 12 } else { 11 }; continue;
}
// C line 7202
14 => {
p_1 = *((*(rt)).atom_array).offset((i) as isize);
vm_block = 13; continue;
}
// C line 7201
15 => {
let _ = { let assigned = (0 as i32); i = assigned; assigned };
vm_block = 10; continue;
}
// C line 7199
16 => {
let _ = { let assigned = (((((size_of::<*mut JSAtomStruct>() as usize)).wrapping_mul((((*(rt)).atom_size) as usize))).wrapping_add(((size_of::<u32>() as usize)).wrapping_mul((((*(rt)).atom_hash_size) as usize)))) as i64); (*(s)).atom_size = assigned; assigned };
vm_block = 15; continue;
}
// C line 7198
17 => {
let _ = { let assigned = (((*(rt)).atom_count) as i64); (*(s)).atom_count = assigned; assigned };
vm_block = 16; continue;
}
// C line 7197
18 => {
let _ = { let assigned = ((((*(s)).memory_used_count) as i64).wrapping_add((((((2 as i32)) as i64)) as i64))) as i64; (*(s)).memory_used_count = assigned; assigned };
vm_block = 17; continue;
}
// C line 7187
19 => {
vm_block = if ((((i) < ((*(rt)).shape_hash_size)) as i32)) != 0 { 26 } else { 18 }; continue;
}
// C line ?
20 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 19; continue;
}
// C line 7189
21 => {
vm_block = if ((((sh_2) != (core::ptr::null_mut::<JSShape>())) as i32)) != 0 { 25 } else { 20 }; continue;
}
// C line ?
22 => {
let _ = { let assigned = (*(sh_2)).shape_hash_next; sh_2 = assigned; assigned };
vm_block = 21; continue;
}
// C line 7192
23 => {
let _ = { let assigned = ((((*(s)).shape_size) as u64).wrapping_add(((((get_shape_size(((hash_size_2) as usize), (((*(sh_2)).prop_size) as usize))) as u64)) as u64))) as i64; (*(s)).shape_size = assigned; assigned };
vm_block = 22; continue;
}
// C line 7191
24 => {
let _ = { let old = (*(s)).shape_count; (*(s)).shape_count = ((*(s)).shape_count).wrapping_add(1); old };
vm_block = 23; continue;
}
// C line 7190
25 => {
hash_size_2 = ((((*(sh_2)).prop_hash_mask).wrapping_add((((1 as i32)) as u32))) as i32);
vm_block = 24; continue;
}
// C line 7189
26 => {
let _ = { let assigned = *((*(rt)).shape_hash).offset((i) as isize); sh_2 = assigned; assigned };
vm_block = 21; continue;
}
// C line 7187
27 => {
let _ = { let assigned = (0 as i32); i = assigned; assigned };
vm_block = 19; continue;
}
// C line 7186
28 => {
let _ = { let assigned = ((((*(s)).memory_used_size) as u64).wrapping_add(((((((size_of::<*mut JSShape>() as usize)).wrapping_mul((((*(rt)).shape_hash_size) as usize))) as u64)) as u64))) as i64; (*(s)).memory_used_size = assigned; assigned };
vm_block = 27; continue;
}
// C line 7185
29 => {
let _ = { let old = (*(s)).memory_used_count; (*(s)).memory_used_count = ((*(s)).memory_used_count).wrapping_add(1); old };
vm_block = 28; continue;
}
// C line 7182
30 => {
let _ = { let assigned = ((((*(s)).obj_size) as u64).wrapping_add(((((((*(s)).obj_count) as u64)).wrapping_mul((((size_of::<JSObject>() as usize)) as u64))) as u64))) as i64; (*(s)).obj_size = assigned; assigned };
vm_block = 29; continue;
}
// C line 6990
31 => {
vm_block = if ((((el) != (core::ptr::addr_of_mut!((*(rt)).gc_obj_list))) as i32)) != 0 { 137 } else { 30 }; continue;
}
// C line ?
32 => {
let _ = { let assigned = (*(el)).next; el = assigned; assigned };
vm_block = 31; continue;
}
// C line 7179
33 => {
vm_block = 32; continue;
}
// C line 7177
34 => {
let _ = { let assigned = ((((*(s)).memory_used_count) as i64).wrapping_add((((((1 as i32)) as i64)) as i64))) as i64; (*(s)).memory_used_count = assigned; assigned };
vm_block = 33; continue;
}
// C line 7176
35 => {
vm_block = if !(((*(p)).u).opaque).is_null() { 34 } else { 33 }; continue;
}
// C line 7142
36 => {
vm_block = 32; continue;
}
// C line 7138
37 => {
let _ = { let assigned = ((((*(s)).memory_used_size) as i64).wrapping_add((((((*(abuf)).byte_length) as i64)) as i64))) as i64; (*(s)).memory_used_size = assigned; assigned };
vm_block = 36; continue;
}
// C line 7137
38 => {
let _ = { let assigned = ((((*(s)).memory_used_count) as i64).wrapping_add((((((1 as i32)) as i64)) as i64))) as i64; (*(s)).memory_used_count = assigned; assigned };
vm_block = 37; continue;
}
// C line 7136
39 => {
vm_block = if !((*(abuf)).data).is_null() { 38 } else { 36 }; continue;
}
// C line 7135
40 => {
let _ = { let assigned = ((((*(s)).memory_used_size) as u64).wrapping_add((((((size_of::<JSArrayBuffer>() as usize)) as u64)) as u64))) as i64; (*(s)).memory_used_size = assigned; assigned };
vm_block = 39; continue;
}
// C line 7134
41 => {
let _ = { let assigned = ((((*(s)).memory_used_count) as i64).wrapping_add((((((1 as i32)) as i64)) as i64))) as i64; (*(s)).memory_used_count = assigned; assigned };
vm_block = 40; continue;
}
// C line 7133
42 => {
vm_block = if !(abuf).is_null() { 41 } else { 36 }; continue;
}
// C line 7132
43 => {
abuf = ((*(p)).u).array_buffer;
vm_block = 42; continue;
}
// C line 7128
44 => {
vm_block = 32; continue;
}
// C line 7125
45 => {
let _ = { let assigned = ((((*(s)).memory_used_size) as u64).wrapping_add((((((size_of::<JSForInIterator>() as usize)) as u64)) as u64))) as i64; (*(s)).memory_used_size = assigned; assigned };
vm_block = 44; continue;
}
// C line 7124
46 => {
let _ = { let assigned = ((((*(s)).memory_used_count) as i64).wrapping_add((((((1 as i32)) as i64)) as i64))) as i64; (*(s)).memory_used_count = assigned; assigned };
vm_block = 45; continue;
}
// C line 7123
47 => {
let _ = compute_value_size((*(it)).obj, hp);
vm_block = 46; continue;
}
// C line 7122
48 => {
vm_block = if !(it).is_null() { 47 } else { 44 }; continue;
}
// C line 7121
49 => {
it = ((*(p)).u).for_in_iterator;
vm_block = 48; continue;
}
// C line 7117
50 => {
vm_block = 32; continue;
}
// C line 7116
51 => {
let _ = compute_jsstring_size((((*(p)).u).regexp).bytecode, hp);
vm_block = 50; continue;
}
// C line 7115
52 => {
let _ = compute_jsstring_size((((*(p)).u).regexp).pattern, hp);
vm_block = 51; continue;
}
// C line 7113
53 => {
vm_block = 32; continue;
}
// C line 7110
54 => {
let _ = { let assigned = ((((*(s)).memory_used_size) as u64).wrapping_add(((((((size_of::<JSCFunctionDataRecord>() as usize)).wrapping_add(((((*(fd)).data_len) as usize)).wrapping_mul((size_of::<JSValue>() as usize)))) as u64)) as u64))) as i64; (*(s)).memory_used_size = assigned; assigned };
vm_block = 53; continue;
}
// C line 7109
55 => {
let _ = { let assigned = ((((*(s)).memory_used_count) as i64).wrapping_add((((((1 as i32)) as i64)) as i64))) as i64; (*(s)).memory_used_count = assigned; assigned };
vm_block = 54; continue;
}
// C line 7106
56 => {
vm_block = if ((((i) < ((((*(fd)).data_len) as i32))) as i32)) != 0 { 58 } else { 55 }; continue;
}
// C line ?
57 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 56; continue;
}
// C line 7107
58 => {
let _ = compute_value_size(*(((*(fd)).data).as_mut_ptr()).offset((i) as isize), hp);
vm_block = 57; continue;
}
// C line 7106
59 => {
let _ = { let assigned = (0 as i32); i = assigned; assigned };
vm_block = 56; continue;
}
// C line 7105
60 => {
vm_block = if !(fd).is_null() { 59 } else { 53 }; continue;
}
// C line 7104
61 => {
fd = ((*(p)).u).c_function_data_record;
vm_block = 60; continue;
}
// C line 7101
62 => {
vm_block = 32; continue;
}
// C line 7099
63 => {
let _ = { let assigned = ((((*(s)).memory_used_size) as u64).wrapping_add(((((((size_of::<JSBoundFunction>() as usize)).wrapping_add(((((*(bf)).argc) as usize)).wrapping_mul((size_of::<JSValue>() as usize)))) as u64)) as u64))) as i64; (*(s)).memory_used_size = assigned; assigned };
vm_block = 62; continue;
}
// C line 7098
64 => {
let _ = { let assigned = ((((*(s)).memory_used_count) as i64).wrapping_add((((((1 as i32)) as i64)) as i64))) as i64; (*(s)).memory_used_count = assigned; assigned };
vm_block = 63; continue;
}
// C line 7095
65 => {
vm_block = if ((((i) < ((*(bf)).argc)) as i32)) != 0 { 67 } else { 64 }; continue;
}
// C line ?
66 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 65; continue;
}
// C line 7096
67 => {
let _ = compute_value_size(*(((*(bf)).argv).as_mut_ptr()).offset((i) as isize), hp);
vm_block = 66; continue;
}
// C line 7095
68 => {
let _ = { let assigned = (0 as i32); i = assigned; assigned };
vm_block = 65; continue;
}
// C line 7093
69 => {
bf = ((*(p)).u).bound_function;
vm_block = 68; continue;
}
// C line 7090
70 => {
vm_block = 32; continue;
}
// C line 7076
71 => {
vm_block = if ((((i) < ((*(b)).closure_var_count)) as i32)) != 0 { 78 } else { 70 }; continue;
}
// C line ?
72 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 71; continue;
}
// C line 7084
73 => {
let _ = compute_value_size(((*(*(var_refs).offset((i) as isize))).u).value, hp);
vm_block = 72; continue;
}
// C line 7082
74 => {
vm_block = if (((((*(*(var_refs).offset((i) as isize))).pvalue) == (core::ptr::addr_of_mut!(((*(*(var_refs).offset((i) as isize))).u).value))) as i32)) != 0 { 73 } else { 72 }; continue;
}
// C line 7080
75 => {
let _ = { let assigned = (((((*(s)).js_func_size) as f64)+(((((((size_of::<JSVarRef>() as usize)) as f64)) / (ref_count))) as f64))) as i64; (*(s)).js_func_size = assigned; assigned };
vm_block = 74; continue;
}
// C line 7079
76 => {
let _ = { let assigned = (((((*(s)).memory_used_count) as f64)+(((((((1 as i32)) as f64)) / (ref_count))) as f64))) as i64; (*(s)).memory_used_count = assigned; assigned };
vm_block = 75; continue;
}
// C line 7078
77 => {
ref_count = (((*(js_rc(((*(var_refs).offset((i) as isize)) as *mut c_void)))).ref_count) as f64);
vm_block = 76; continue;
}
// C line 7077
78 => {
vm_block = if !(*(var_refs).offset((i) as isize)).is_null() { 77 } else { 72 }; continue;
}
// C line 7076
79 => {
let _ = { let assigned = (0 as i32); i = assigned; assigned };
vm_block = 71; continue;
}
// C line 7075
80 => {
let _ = { let assigned = ((((*(s)).js_func_size) as u64).wrapping_add(((((((((*(b)).closure_var_count) as usize)).wrapping_mul((size_of::<*mut JSVarRef>() as usize))) as u64)) as u64))) as i64; (*(s)).js_func_size = assigned; assigned };
vm_block = 79; continue;
}
// C line 7074
81 => {
let _ = { let old = (*(s)).memory_used_count; (*(s)).memory_used_count = ((*(s)).memory_used_count).wrapping_add(1); old };
vm_block = 80; continue;
}
// C line 7073
82 => {
vm_block = if !(var_refs).is_null() { 81 } else { 70 }; continue;
}
// C line 7071
83 => {
var_refs = (((*(p)).u).func).var_refs;
vm_block = 82; continue;
}
// C line 7070
84 => {
b = (((*(p)).u).func).function_bytecode;
vm_block = 83; continue;
}
// C line 7067
85 => {
vm_block = 32; continue;
}
// C line 7066
86 => {
let _ = { let old = (*(s)).c_func_count; (*(s)).c_func_count = ((*(s)).c_func_count).wrapping_add(1); old };
vm_block = 85; continue;
}
// C line 7064
87 => {
vm_block = 32; continue;
}
// C line 7063
88 => {
let _ = compute_value_size(((*(p)).u).object_data, hp);
vm_block = 87; continue;
}
// C line 7056
89 => {
vm_block = 32; continue;
}
// C line 7051
90 => {
vm_block = if ((((((i) as u32)) < ((((*(p)).u).array).count)) as i32)) != 0 { 92 } else { 89 }; continue;
}
// C line ?
91 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 90; continue;
}
// C line 7052
92 => {
let _ = compute_value_size(*((*(*(((((*(p)).u).array).u).var_refs).offset((i) as isize))).pvalue), hp);
vm_block = 91; continue;
}
// C line 7051
93 => {
let _ = { let assigned = (0 as i32); i = assigned; assigned };
vm_block = 90; continue;
}
// C line 7050
94 => {
let _ = { let assigned = ((((*(s)).fast_array_elements) as i64).wrapping_add((((((((*(p)).u).array).count) as i64)) as i64))) as i64; (*(s)).fast_array_elements = assigned; assigned };
vm_block = 93; continue;
}
// C line 7048
95 => {
let _ = { let assigned = ((((*(s)).memory_used_size) as u64).wrapping_add(((((((((((*(p)).u).array).count) as usize)).wrapping_mul((size_of::<*mut JSVarRef>() as usize))) as u64)) as u64))) as i64; (*(s)).memory_used_size = assigned; assigned };
vm_block = 94; continue;
}
// C line 7047
96 => {
let _ = { let old = (*(s)).memory_used_count; (*(s)).memory_used_count = ((*(s)).memory_used_count).wrapping_add(1); old };
vm_block = 95; continue;
}
// C line 7046
97 => {
vm_block = if !(((((*(p)).u).array).u).values).is_null() { 96 } else { 89 }; continue;
}
// C line 7045
98 => {
let _ = { let old = (*(s)).fast_array_count; (*(s)).fast_array_count = ((*(s)).fast_array_count).wrapping_add(1); old };
vm_block = 97; continue;
}
// C line 7044
99 => {
vm_block = if ((*(p)).fast_array()) != 0 { 98 } else { 89 }; continue;
}
// C line 7042
100 => {
vm_block = 32; continue;
}
// C line 7037
101 => {
vm_block = if ((((((i) as u32)) < ((((*(p)).u).array).count)) as i32)) != 0 { 103 } else { 100 }; continue;
}
// C line ?
102 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 101; continue;
}
// C line 7038
103 => {
let _ = compute_value_size(*(((((*(p)).u).array).u).values).offset((i) as isize), hp);
vm_block = 102; continue;
}
// C line 7037
104 => {
let _ = { let assigned = (0 as i32); i = assigned; assigned };
vm_block = 101; continue;
}
// C line 7036
105 => {
let _ = { let assigned = ((((*(s)).fast_array_elements) as i64).wrapping_add((((((((*(p)).u).array).count) as i64)) as i64))) as i64; (*(s)).fast_array_elements = assigned; assigned };
vm_block = 104; continue;
}
// C line 7034
106 => {
let _ = { let assigned = ((((*(s)).memory_used_size) as u64).wrapping_add(((((((((((*(p)).u).array).count) as usize)).wrapping_mul((size_of::<JSValue>() as usize))) as u64)) as u64))) as i64; (*(s)).memory_used_size = assigned; assigned };
vm_block = 105; continue;
}
// C line 7033
107 => {
let _ = { let old = (*(s)).memory_used_count; (*(s)).memory_used_count = ((*(s)).memory_used_count).wrapping_add(1); old };
vm_block = 106; continue;
}
// C line 7032
108 => {
vm_block = if !(((((*(p)).u).array).u).values).is_null() { 107 } else { 100 }; continue;
}
// C line 7031
109 => {
let _ = { let old = (*(s)).fast_array_count; (*(s)).fast_array_count = ((*(s)).fast_array_count).wrapping_add(1); old };
vm_block = 108; continue;
}
// C line 7030
110 => {
vm_block = if ((*(p)).fast_array()) != 0 { 109 } else { 100 }; continue;
}
// C line 7029
111 => {
let _ = { let old = (*(s)).array_count; (*(s)).array_count = ((*(s)).array_count).wrapping_add(1); old };
vm_block = 110; continue;
}
// C line 7026
112 => {
vm_block = match (((*(p)).class_id) as i32) { x if x == (JS_CLASS_ASYNC_GENERATOR as i32) => 35, x if x == (JS_CLASS_ASYNC_FROM_SYNC_ITERATOR as i32) => 35, x if x == (JS_CLASS_ASYNC_FUNCTION_REJECT as i32) => 35, x if x == (JS_CLASS_ASYNC_FUNCTION_RESOLVE as i32) => 35, x if x == (JS_CLASS_PROMISE_REJECT_FUNCTION as i32) => 35, x if x == (JS_CLASS_PROMISE_RESOLVE_FUNCTION as i32) => 35, x if x == (JS_CLASS_PROMISE as i32) => 35, x if x == (JS_CLASS_PROXY as i32) => 35, x if x == (JS_CLASS_STRING_ITERATOR as i32) => 35, x if x == (JS_CLASS_ARRAY_ITERATOR as i32) => 35, x if x == (JS_CLASS_SET_ITERATOR as i32) => 35, x if x == (JS_CLASS_MAP_ITERATOR as i32) => 35, x if x == (JS_CLASS_WEAKSET as i32) => 35, x if x == (JS_CLASS_WEAKMAP as i32) => 35, x if x == (JS_CLASS_SET as i32) => 35, x if x == (JS_CLASS_MAP as i32) => 35, x if x == (JS_CLASS_DATAVIEW as i32) => 35, x if x == (JS_CLASS_FLOAT64_ARRAY as i32) => 35, x if x == (JS_CLASS_FLOAT32_ARRAY as i32) => 35, x if x == (JS_CLASS_FLOAT16_ARRAY as i32) => 35, x if x == (JS_CLASS_BIG_UINT64_ARRAY as i32) => 35, x if x == (JS_CLASS_BIG_INT64_ARRAY as i32) => 35, x if x == (JS_CLASS_UINT32_ARRAY as i32) => 35, x if x == (JS_CLASS_INT32_ARRAY as i32) => 35, x if x == (JS_CLASS_UINT16_ARRAY as i32) => 35, x if x == (JS_CLASS_INT16_ARRAY as i32) => 35, x if x == (JS_CLASS_UINT8_ARRAY as i32) => 35, x if x == (JS_CLASS_INT8_ARRAY as i32) => 35, x if x == (JS_CLASS_UINT8C_ARRAY as i32) => 35, x if x == (JS_CLASS_GENERATOR as i32) => 35, x if x == (JS_CLASS_SHARED_ARRAY_BUFFER as i32) => 43, x if x == (JS_CLASS_ARRAY_BUFFER as i32) => 43, x if x == (JS_CLASS_FOR_IN_ITERATOR as i32) => 49, x if x == (JS_CLASS_REGEXP as i32) => 52, x if x == (JS_CLASS_C_FUNCTION_DATA as i32) => 61, x if x == (JS_CLASS_BOUND_FUNCTION as i32) => 69, x if x == (JS_CLASS_BYTECODE_FUNCTION as i32) => 84, x if x == (JS_CLASS_C_FUNCTION as i32) => 86, x if x == (JS_CLASS_BIG_INT as i32) => 88, x if x == (JS_CLASS_DATE as i32) => 88, x if x == (JS_CLASS_SYMBOL as i32) => 88, x if x == (JS_CLASS_BOOLEAN as i32) => 88, x if x == (JS_CLASS_STRING as i32) => 88, x if x == (JS_CLASS_NUMBER as i32) => 88, x if x == (JS_CLASS_MAPPED_ARGUMENTS as i32) => 99, x if x == (JS_CLASS_ARGUMENTS as i32) => 111, x if x == (JS_CLASS_ARRAY as i32) => 111, _ => 35, }; continue;
}
// C line 7023
113 => {
let _ = { let assigned = ((((*(s)).shape_size) as u64).wrapping_add(((((get_shape_size(((hash_size_1) as usize), (((*(sh_1)).prop_size) as usize))) as u64)) as u64))) as i64; (*(s)).shape_size = assigned; assigned };
vm_block = 112; continue;
}
// C line 7022
114 => {
let _ = { let old = (*(s)).shape_count; (*(s)).shape_count = ((*(s)).shape_count).wrapping_add(1); old };
vm_block = 113; continue;
}
// C line 7021
115 => {
hash_size_1 = ((((*(sh_1)).prop_hash_mask).wrapping_add((((1 as i32)) as u32))) as i32);
vm_block = 114; continue;
}
// C line 7020
116 => {
vm_block = if ((!(((*(sh_1)).is_hashed) != 0) as i32)) != 0 { 115 } else { 112 }; continue;
}
// C line 7011
117 => {
vm_block = if ((((i) < ((*(sh_1)).prop_count)) as i32)) != 0 { 122 } else { 116 }; continue;
}
// C line ?
118 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 117; continue;
}
// C line 7016
119 => {
let _ = { let old = prs; prs = (prs).offset(1); old };
vm_block = 118; continue;
}
// C line 7014
120 => {
let _ = compute_value_size(((*(pr)).u).value, hp);
vm_block = 119; continue;
}
// C line 7013
121 => {
vm_block = if (((((((((*(prs)).atom) != ((((0 as i32)) as JSAtom))) as i32)) != 0) && (((!(((((((*(prs)).flags()) as i32)) & (((3 as i32)).wrapping_shl(((4 as i32)) as u32)))) != 0) as i32)) != 0)) as i32)) != 0 { 120 } else { 119 }; continue;
}
// C line 7012
122 => {
pr = core::ptr::addr_of_mut!(*((*(p)).prop).offset((i) as isize));
vm_block = 121; continue;
}
// C line 7011
123 => {
let _ = { let assigned = (0 as i32); i = assigned; assigned };
vm_block = 117; continue;
}
// C line 7010
124 => {
let _ = { let assigned = get_shape_prop(sh_1); prs = assigned; assigned };
vm_block = 123; continue;
}
// C line 7009
125 => {
let _ = { let assigned = ((((*(s)).prop_count) as i64).wrapping_add((((((*(sh_1)).prop_count) as i64)) as i64))) as i64; (*(s)).prop_count = assigned; assigned };
vm_block = 124; continue;
}
// C line 7008
126 => {
let _ = { let assigned = ((((*(s)).prop_size) as u64).wrapping_add(((((((((*(sh_1)).prop_size) as usize)).wrapping_mul((size_of::<JSProperty>() as usize))) as u64)) as u64))) as i64; (*(s)).prop_size = assigned; assigned };
vm_block = 125; continue;
}
// C line 7007
127 => {
let _ = { let old = (*(s)).memory_used_count; (*(s)).memory_used_count = ((*(s)).memory_used_count).wrapping_add(1); old };
vm_block = 126; continue;
}
// C line 7006
128 => {
vm_block = if !((*(p)).prop).is_null() { 127 } else { 116 }; continue;
}
// C line 7005
129 => {
let _ = { let old = (*(s)).obj_count; (*(s)).obj_count = ((*(s)).obj_count).wrapping_add(1); old };
vm_block = 128; continue;
}
// C line 7004
130 => {
let _ = { let assigned = (*(p)).shape; sh_1 = assigned; assigned };
vm_block = 129; continue;
}
// C line 7003
131 => {
let _ = { let assigned = ((gp) as *mut JSObject); p = assigned; assigned };
vm_block = 130; continue;
}
// C line 6999
132 => {
vm_block = 32; continue;
}
// C line 6998
133 => {
let _ = compute_bytecode_size(((gp) as *mut JSFunctionBytecode), hp);
vm_block = 132; continue;
}
// C line 7001
134 => {
vm_block = 32; continue;
}
// C line 7000
135 => {
vm_block = if (((((((*(js_rc(((gp) as *mut c_void)))).gc_obj_type()) as i32)) != ((JS_GC_OBJ_TYPE_JS_OBJECT as i32))) as i32)) != 0 { 134 } else { 131 }; continue;
}
// C line 6997
136 => {
vm_block = if (((((((*(js_rc(((gp) as *mut c_void)))).gc_obj_type()) as i32)) == ((JS_GC_OBJ_TYPE_FUNCTION_BYTECODE as i32))) as i32)) != 0 { 133 } else { 135 }; continue;
}
// C line 6991
137 => {
gp = (((((el) as *mut u8)).offset(-(((core::mem::offset_of!(JSGCObjectHeader, link) as usize)) as isize))) as *mut JSGCObjectHeader);
vm_block = 136; continue;
}
// C line ?
138 => {
let _ = { let assigned = (*(core::ptr::addr_of_mut!((*(rt)).gc_obj_list))).next; el = assigned; assigned };
vm_block = 31; continue;
}
// C line 6942
139 => {
vm_block = if ((((el) != (core::ptr::addr_of_mut!((*(rt)).context_list))) as i32)) != 0 { 177 } else { 138 }; continue;
}
// C line ?
140 => {
let _ = { let assigned = (*(el)).next; el = assigned; assigned };
vm_block = 139; continue;
}
// C line 6957
141 => {
vm_block = if ((((el1) != (core::ptr::addr_of_mut!((*(ctx)).loaded_modules))) as i32)) != 0 { 166 } else { 140 }; continue;
}
// C line ?
142 => {
let _ = { let assigned = (*(el1)).next; el1 = assigned; assigned };
vm_block = 141; continue;
}
// C line 6986
143 => {
let _ = compute_value_size((*(m)).func_obj, hp);
vm_block = 142; continue;
}
// C line 6985
144 => {
let _ = compute_value_size((*(m)).module_ns, hp);
vm_block = 143; continue;
}
// C line 6983
145 => {
let _ = { let assigned = ((((*(s)).memory_used_size) as u64).wrapping_add(((((((((*(m)).import_entries_count) as usize)).wrapping_mul((size_of::<JSImportEntry>() as usize))) as u64)) as u64))) as i64; (*(s)).memory_used_size = assigned; assigned };
vm_block = 144; continue;
}
// C line 6982
146 => {
let _ = { let assigned = ((((*(s)).memory_used_count) as i64).wrapping_add((((((1 as i32)) as i64)) as i64))) as i64; (*(s)).memory_used_count = assigned; assigned };
vm_block = 145; continue;
}
// C line 6981
147 => {
vm_block = if !((*(m)).import_entries).is_null() { 146 } else { 144 }; continue;
}
// C line 6979
148 => {
let _ = { let assigned = ((((*(s)).memory_used_size) as u64).wrapping_add(((((((((*(m)).star_export_entries_count) as usize)).wrapping_mul((size_of::<JSStarExportEntry>() as usize))) as u64)) as u64))) as i64; (*(s)).memory_used_size = assigned; assigned };
vm_block = 147; continue;
}
// C line 6978
149 => {
let _ = { let assigned = ((((*(s)).memory_used_count) as i64).wrapping_add((((((1 as i32)) as i64)) as i64))) as i64; (*(s)).memory_used_count = assigned; assigned };
vm_block = 148; continue;
}
// C line 6977
150 => {
vm_block = if !((*(m)).star_export_entries).is_null() { 149 } else { 147 }; continue;
}
// C line 6968
151 => {
vm_block = if ((((i) < ((*(m)).export_entries_count)) as i32)) != 0 { 156 } else { 150 }; continue;
}
// C line ?
152 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 151; continue;
}
// C line 6973
153 => {
let _ = compute_value_size(((*((((*(me)).u).local).var_ref)).u).value, hp);
vm_block = 152; continue;
}
// C line 6972
154 => {
let _ = { let assigned = ((((*(s)).memory_used_count) as i64).wrapping_add((((((1 as i32)) as i64)) as i64))) as i64; (*(s)).memory_used_count = assigned; assigned };
vm_block = 153; continue;
}
// C line 6970
155 => {
vm_block = if (((((((((((*(me)).export_type) as u32)) == ((((JS_EXPORT_TYPE_LOCAL as i32)) as u32))) as i32)) != 0) && (!((((*(me)).u).local).var_ref).is_null())) as i32)) != 0 { 154 } else { 152 }; continue;
}
// C line 6969
156 => {
me = core::ptr::addr_of_mut!(*((*(m)).export_entries).offset((i) as isize));
vm_block = 155; continue;
}
// C line 6968
157 => {
let _ = { let assigned = (0 as i32); i = assigned; assigned };
vm_block = 151; continue;
}
// C line 6967
158 => {
let _ = { let assigned = ((((*(s)).memory_used_size) as u64).wrapping_add(((((((((*(m)).export_entries_count) as usize)).wrapping_mul((size_of::<JSExportEntry>() as usize))) as u64)) as u64))) as i64; (*(s)).memory_used_size = assigned; assigned };
vm_block = 157; continue;
}
// C line 6966
159 => {
let _ = { let assigned = ((((*(s)).memory_used_count) as i64).wrapping_add((((((1 as i32)) as i64)) as i64))) as i64; (*(s)).memory_used_count = assigned; assigned };
vm_block = 158; continue;
}
// C line 6965
160 => {
vm_block = if !((*(m)).export_entries).is_null() { 159 } else { 150 }; continue;
}
// C line 6963
161 => {
let _ = { let assigned = ((((*(s)).memory_used_size) as u64).wrapping_add(((((((((*(m)).req_module_entries_count) as usize)).wrapping_mul((size_of::<JSReqModuleEntry>() as usize))) as u64)) as u64))) as i64; (*(s)).memory_used_size = assigned; assigned };
vm_block = 160; continue;
}
// C line 6962
162 => {
let _ = { let assigned = ((((*(s)).memory_used_count) as i64).wrapping_add((((((1 as i32)) as i64)) as i64))) as i64; (*(s)).memory_used_count = assigned; assigned };
vm_block = 161; continue;
}
// C line 6961
163 => {
vm_block = if !((*(m)).req_module_entries).is_null() { 162 } else { 160 }; continue;
}
// C line 6960
164 => {
let _ = { let assigned = ((((*(s)).memory_used_size) as u64).wrapping_add((((((size_of::<JSModuleDef>() as usize)) as u64)) as u64))) as i64; (*(s)).memory_used_size = assigned; assigned };
vm_block = 163; continue;
}
// C line 6959
165 => {
let _ = { let assigned = ((((*(s)).memory_used_count) as i64).wrapping_add((((((1 as i32)) as i64)) as i64))) as i64; (*(s)).memory_used_count = assigned; assigned };
vm_block = 164; continue;
}
// C line 6958
166 => {
m = (((((el1) as *mut u8)).offset(-(((core::mem::offset_of!(JSModuleDef, link) as usize)) as isize))) as *mut JSModuleDef);
vm_block = 165; continue;
}
// C line ?
167 => {
let _ = { let assigned = (*(core::ptr::addr_of_mut!((*(ctx)).loaded_modules))).next; el1 = assigned; assigned };
vm_block = 141; continue;
}
// C line 6955
168 => {
let _ = { let assigned = ((((*(s)).shape_size) as u64).wrapping_add(((((get_shape_size(((hash_size) as usize), (((*(sh)).prop_size) as usize))) as u64)) as u64))) as i64; (*(s)).shape_size = assigned; assigned };
vm_block = 167; continue;
}
// C line 6954
169 => {
let _ = { let old = (*(s)).shape_count; (*(s)).shape_count = ((*(s)).shape_count).wrapping_add(1); old };
vm_block = 168; continue;
}
// C line 6953
170 => {
hash_size = ((((*(sh)).prop_hash_mask).wrapping_add((((1 as i32)) as u32))) as i32);
vm_block = 169; continue;
}
// C line 6952
171 => {
vm_block = if ((((!(sh).is_null()) && (((!(((*(sh)).is_hashed) != 0) as i32)) != 0)) as i32)) != 0 { 170 } else { 167 }; continue;
}
// C line 6949
172 => {
let _ = { let assigned = ((((*(s)).binary_object_size) as i64).wrapping_add((((((*(ctx)).binary_object_size) as i64)) as i64))) as i64; (*(s)).binary_object_size = assigned; assigned };
vm_block = 171; continue;
}
// C line 6948
173 => {
let _ = { let assigned = ((((*(s)).binary_object_count) as i64).wrapping_add((((((*(ctx)).binary_object_count) as i64)) as i64))) as i64; (*(s)).binary_object_count = assigned; assigned };
vm_block = 172; continue;
}
// C line 6946
174 => {
let _ = { let assigned = ((((*(s)).memory_used_size) as u64).wrapping_add(((((((size_of::<JSContext>() as usize)).wrapping_add(((size_of::<JSValue>() as usize)).wrapping_mul((((*(rt)).class_count) as usize)))) as u64)) as u64))) as i64; (*(s)).memory_used_size = assigned; assigned };
vm_block = 173; continue;
}
// C line 6945
175 => {
let _ = { let assigned = ((((*(s)).memory_used_count) as i64).wrapping_add((((((2 as i32)) as i64)) as i64))) as i64; (*(s)).memory_used_count = assigned; assigned };
vm_block = 174; continue;
}
// C line 6944
176 => {
sh = (*(ctx)).array_shape;
vm_block = 175; continue;
}
// C line 6943
177 => {
ctx = (((((el) as *mut u8)).offset(-(((core::mem::offset_of!(JSContext, link) as usize)) as isize))) as *mut JSContext);
vm_block = 176; continue;
}
// C line ?
178 => {
let _ = { let assigned = (*(core::ptr::addr_of_mut!((*(rt)).context_list))).next; el = assigned; assigned };
vm_block = 139; continue;
}
// C line 6940
179 => {
let _ = { let assigned = ((((size_of::<JSRuntime>() as usize)).wrapping_add(((size_of::<JSValue>() as usize)).wrapping_mul((((*(rt)).class_count) as usize)))) as i64); (*(s)).memory_used_size = assigned; assigned };
vm_block = 178; continue;
}
// C line 6939
180 => {
let _ = { let assigned = (((2 as i32)) as i64); (*(s)).memory_used_count = assigned; assigned };
vm_block = 179; continue;
}
// C line 6937
181 => {
let _ = { let assigned = (((((*(rt)).malloc_ctx).malloc_state).malloc_limit) as i64); (*(s)).malloc_limit = assigned; assigned };
vm_block = 180; continue;
}
// C line 6936
182 => {
let _ = { let assigned = (((((*(rt)).malloc_ctx).malloc_state).malloc_size) as i64); (*(s)).malloc_size = assigned; assigned };
vm_block = 181; continue;
}
// C line 6935
183 => {
let _ = { let assigned = (((((*(rt)).malloc_ctx).malloc_state).malloc_count) as i64); (*(s)).malloc_count = assigned; assigned };
vm_block = 182; continue;
}
// C line 6934
184 => {
let _ = { let dst = (((s) as *mut c_void)) as *mut u8; core::ptr::write_bytes(dst, ((0 as i32)) as u8, ((size_of::<JSMemoryUsage>() as usize)) as usize); dst as *mut c_void };
vm_block = 183; continue;
}
// C line 6932
185 => {
mem = JSMemoryUsage_helper { memory_used_count: (((0 as i32)) as f64), str_count: 0.0, str_size: 0.0, js_func_count: 0, js_func_size: 0.0, js_func_code_size: 0, js_func_pc2line_count: 0, js_func_pc2line_size: 0 };
hp = core::ptr::addr_of_mut!(mem);
vm_block = 184; continue;
}
_ => std::process::abort(),
} }
}
