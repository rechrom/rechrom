// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:20776. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn async_func_init(mut ctx: *mut JSContext, mut func_obj: JSValue, mut this_obj: JSValue, mut argc: i32, mut argv: *mut JSValue) -> *mut JSAsyncFunctionState {
let mut vm_local_storage = Vec::<u64>::new();
let mut s: *mut JSAsyncFunctionState = core::mem::zeroed();
let mut p: *mut JSObject = core::mem::zeroed();
let mut b: *mut JSFunctionBytecode = core::mem::zeroed();
let mut sf: *mut JSStackFrame = core::mem::zeroed();
let mut i: i32 = core::mem::zeroed();
let mut arg_buf_len: i32 = core::mem::zeroed();
let mut n: i32 = core::mem::zeroed();
let mut vm_block: usize = 37;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 20817
1 => {
return s;
}
// C line 20816
2 => {
let _ = { let assigned = (0 as i32); (*(s)).is_completed = assigned; assigned };
vm_block = 1; continue;
}
// C line 20815
3 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; *(((*(s)).resolving_funcs).as_mut_ptr()).offset(((1 as i32)) as isize) = assigned; assigned };
vm_block = 2; continue;
}
// C line 20814
4 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; *(((*(s)).resolving_funcs).as_mut_ptr()).offset(((0 as i32)) as isize) = assigned; assigned };
vm_block = 3; continue;
}
// C line 20812
5 => {
vm_block = if ((((i) < (n)) as i32)) != 0 { 7 } else { 4 }; continue;
}
// C line ?
6 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 5; continue;
}
// C line 20813
7 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; *((*(sf)).arg_buf).offset((i) as isize) = assigned; assigned };
vm_block = 6; continue;
}
// C line 20812
8 => {
let _ = { let assigned = argc; i = assigned; assigned };
vm_block = 5; continue;
}
// C line 20811
9 => {
let _ = { let assigned = (arg_buf_len).wrapping_add((((*(b)).var_count) as i32)); n = assigned; assigned };
vm_block = 8; continue;
}
// C line 20809
10 => {
vm_block = if ((((i) < (argc)) as i32)) != 0 { 12 } else { 9 }; continue;
}
// C line ?
11 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 10; continue;
}
// C line 20810
12 => {
let _ = { let assigned = JS_DupValue(ctx, *(argv).offset((i) as isize)); *((*(sf)).arg_buf).offset((i) as isize) = assigned; assigned };
vm_block = 11; continue;
}
// C line 20809
13 => {
let _ = { let assigned = (0 as i32); i = assigned; assigned };
vm_block = 10; continue;
}
// C line 20807
14 => {
vm_block = if ((((i) < ((((*(b)).var_ref_count) as i32))) as i32)) != 0 { 16 } else { 13 }; continue;
}
// C line ?
15 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 14; continue;
}
// C line 20808
16 => {
let _ = { let assigned = core::ptr::null_mut::<JSVarRef>(); *((*(sf)).var_refs).offset((i) as isize) = assigned; assigned };
vm_block = 15; continue;
}
// C line 20807
17 => {
let _ = { let assigned = (0 as i32); i = assigned; assigned };
vm_block = 14; continue;
}
// C line 20806
18 => {
let _ = { let assigned = ((((*(sf)).cur_sp).offset((((((*(b)).stack_size) as i32)) as isize))) as *mut *mut JSVarRef); (*(sf)).var_refs = assigned; assigned };
vm_block = 17; continue;
}
// C line 20805
19 => {
let _ = { let assigned = ((*(sf)).var_buf).offset((((((*(b)).var_count) as i32)) as isize)); (*(sf)).cur_sp = assigned; assigned };
vm_block = 18; continue;
}
// C line 20804
20 => {
let _ = { let assigned = ((*(sf)).arg_buf).offset(((arg_buf_len) as isize)); (*(sf)).var_buf = assigned; assigned };
vm_block = 19; continue;
}
// C line 20803
21 => {
let _ = { let assigned = arg_buf_len; (*(sf)).arg_count = assigned; assigned };
vm_block = 20; continue;
}
// C line 20802
22 => {
let _ = { let assigned = argc; (*(s)).argc = assigned; assigned };
vm_block = 21; continue;
}
// C line 20801
23 => {
let _ = { let assigned = JS_DupValue(ctx, this_obj); (*(s)).this_val = assigned; assigned };
vm_block = 22; continue;
}
// C line 20800
24 => {
let _ = { let assigned = JS_DupValue(ctx, func_obj); (*(sf)).cur_func = assigned; assigned };
vm_block = 23; continue;
}
// C line 20799
25 => {
let _ = { let assigned = (((s).offset((((1 as i32)) as isize))) as *mut JSValue); (*(sf)).arg_buf = assigned; assigned };
vm_block = 24; continue;
}
// C line 20798
26 => {
let _ = { let assigned = (*(b)).byte_code_buf; (*(sf)).cur_pc = assigned; assigned };
vm_block = 25; continue;
}
// C line 20797
27 => {
let _ = { let assigned = (((((*(b)).js_mode) as i32)) | (((1 as i32)).wrapping_shl(((2 as i32)) as u32))); (*(sf)).js_mode = assigned; assigned };
vm_block = 26; continue;
}
// C line 20796
28 => {
let _ = { let assigned = core::ptr::addr_of_mut!((*(s)).frame); sf = assigned; assigned };
vm_block = 27; continue;
}
// C line 20794
29 => {
let _ = add_gc_object((*(ctx)).rt, core::ptr::addr_of_mut!((*(s)).header), (((JS_GC_OBJ_TYPE_ASYNC_FUNCTION as i32)) as JSGCObjectTypeEnum));
vm_block = 28; continue;
}
// C line 20793
30 => {
let _ = { let assigned = (1 as i32); (*(js_rc(((s) as *mut c_void)))).ref_count = assigned; assigned };
vm_block = 29; continue;
}
// C line 20792
31 => {
let _ = { let dst = (((s) as *mut c_void)) as *mut u8; core::ptr::write_bytes(dst, ((0 as i32)) as u8, ((size_of::<JSAsyncFunctionState>() as usize)) as usize); dst as *mut c_void };
vm_block = 30; continue;
}
// C line 20791
32 => {
return core::ptr::null_mut::<JSAsyncFunctionState>();
}
// C line 20790
33 => {
vm_block = if ((!(!(s).is_null()) as i32)) != 0 { 32 } else { 31 }; continue;
}
// C line 20789
34 => {
let _ = { let assigned = ((js_malloc(ctx, (((size_of::<JSAsyncFunctionState>() as usize)).wrapping_add(((size_of::<JSValue>() as usize)).wrapping_mul(((((arg_buf_len).wrapping_add((((*(b)).var_count) as i32))).wrapping_add((((*(b)).stack_size) as i32))) as usize)))).wrapping_add(((size_of::<*mut JSVarRef>() as usize)).wrapping_mul((((*(b)).var_ref_count) as usize))))) as *mut JSAsyncFunctionState); s = assigned; assigned };
vm_block = 33; continue;
}
// C line 20788
35 => {
let _ = { let assigned = max_int((((*(b)).arg_count) as i32), argc); arg_buf_len = assigned; assigned };
vm_block = 34; continue;
}
// C line 20787
36 => {
let _ = { let assigned = (((*(p)).u).func).function_bytecode; b = assigned; assigned };
vm_block = 35; continue;
}
// C line 20786
37 => {
let _ = { let assigned = ((((func_obj).u).ptr) as *mut JSObject); p = assigned; assigned };
vm_block = 36; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:20834. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn async_func_resume(mut ctx: *mut JSContext, mut s: *mut JSAsyncFunctionState) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut rt: *mut JSRuntime = core::mem::zeroed();
let mut sf: *mut JSStackFrame = core::mem::zeroed();
let mut func_obj: JSValue = core::mem::zeroed();
let mut ret: JSValue = core::mem::zeroed();
let mut p: *mut JSObject = core::mem::zeroed();
let mut b: *mut JSFunctionBytecode = core::mem::zeroed();
let mut vm_block: usize = 17;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 20868
1 => {
return ret;
}
// C line 20866
2 => {
let _ = async_func_free_frame(rt, s);
vm_block = 1; continue;
}
// C line 20864
3 => {
let _ = close_var_refs(rt, b, sf);
vm_block = 2; continue;
}
// C line 20861
4 => {
let _ = { let assigned = (1 as i32); (*(s)).is_completed = assigned; assigned };
vm_block = 3; continue;
}
// C line 20858
5 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; *((*(sf)).cur_sp).offset((((1 as i32)).wrapping_neg()) as isize) = assigned; assigned };
vm_block = 4; continue;
}
// C line 20857
6 => {
let _ = { let assigned = *((*(sf)).cur_sp).offset((((1 as i32)).wrapping_neg()) as isize); ret = assigned; assigned };
vm_block = 5; continue;
}
// C line 20856
7 => {
vm_block = if (JS_IsUndefined(ret)) != 0 { 6 } else { 4 }; continue;
}
// C line 20854
8 => {
let _ = { let assigned = (((*(p)).u).func).function_bytecode; b = assigned; assigned };
vm_block = 7; continue;
}
// C line 20853
9 => {
let _ = { let assigned = (((((*(sf)).cur_func).u).ptr) as *mut JSObject); p = assigned; assigned };
vm_block = 8; continue;
}
// C line 20849
10 => {
vm_block = if (((((JS_IsException(ret)) != 0) || ((JS_IsUndefined(ret)) != 0)) as i32)) != 0 { 9 } else { 1 }; continue;
}
// C line 20842
11 => {
let _ = { let assigned = JS_ThrowStackOverflow(ctx); ret = assigned; assigned };
vm_block = 10; continue;
}
// C line 20846
12 => {
let _ = { let assigned = JS_CallInternal(ctx, func_obj, (*(s)).this_val, JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }, (*(s)).argc, (*(sf)).arg_buf, ((1 as i32)).wrapping_shl(((2 as i32)) as u32)); ret = assigned; assigned };
vm_block = 10; continue;
}
// C line 20845
13 => {
let _ = { let assigned = JSValue { u: JSValueUnion { ptr: ((s) as *mut c_void) }, tag: (((JS_TAG_INT as i32)) as i64) }; func_obj = assigned; assigned };
vm_block = 12; continue;
}
// C line 20841
14 => {
vm_block = if (js_check_stack_overflow((*(ctx)).rt, (((0 as i32)) as usize))) != 0 { 11 } else { 13 }; continue;
}
// C line 20840
15 => {
let _ = if ((((!(((!(((*(s)).is_completed) != 0) as i32)) != 0) as i32)) as i64)) != 0 { std::process::abort() } else { { let _ = (0 as i32); } };
vm_block = 14; continue;
}
// C line 20837
16 => {
sf = core::ptr::addr_of_mut!((*(s)).frame);
vm_block = 15; continue;
}
// C line 20836
17 => {
rt = (*(ctx)).rt;
vm_block = 16; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:20960. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_generator_next(mut ctx: *mut JSContext, mut this_val: JSValue, mut argc: i32, mut argv: *mut JSValue, mut pdone: *mut i32, mut magic: i32) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut s: *mut JSGeneratorData = core::mem::zeroed();
let mut sf: *mut JSStackFrame = core::mem::zeroed();
let mut ret: JSValue = core::mem::zeroed();
let mut func_ret: JSValue = core::mem::zeroed();
let mut vm_block: usize = 45;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 21039
1 => {
return ret;
}
// C line 21037
2 => {
vm_block = 1; continue;
}
// C line 21036
3 => {
let _ = { let assigned = JS_ThrowTypeError(ctx, c"cannot invoke a running generator".as_ptr()); ret = assigned; assigned };
vm_block = 2; continue;
}
// C line 21034
4 => {
vm_block = 1; continue;
}
// C line 21032
5 => {
vm_block = 4; continue;
}
// C line 21031
6 => {
let _ = { let assigned = JS_Throw(ctx, JS_DupValue(ctx, *(argv).offset(((0 as i32)) as isize))); ret = assigned; assigned };
vm_block = 5; continue;
}
// C line 21029
7 => {
vm_block = 4; continue;
}
// C line 21028
8 => {
let _ = { let assigned = JS_DupValue(ctx, *(argv).offset(((0 as i32)) as isize)); ret = assigned; assigned };
vm_block = 7; continue;
}
// C line 21026
9 => {
vm_block = 4; continue;
}
// C line 21025
10 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; ret = assigned; assigned };
vm_block = 9; continue;
}
// C line 21022 labels: done
11 => {
vm_block = match magic { x if x == (2 as i32) => 6, x if x == (1 as i32) => 8, x if x == (0 as i32) => 10, _ => 10, }; continue;
}
// C line 21018
12 => {
vm_block = 1; continue;
}
// C line 21004
13 => {
return func_ret;
}
// C line 21003
14 => {
let _ = free_generator_stack(ctx, s);
vm_block = 13; continue;
}
// C line 21013
15 => {
let _ = { let assigned = (2 as i32); *(pdone) = assigned; assigned };
vm_block = 12; continue;
}
// C line 21011
16 => {
let _ = { let assigned = (((JS_GENERATOR_STATE_SUSPENDED_YIELD_STAR as i32)) as JSGeneratorStateEnum); (*(s)).state = assigned; assigned };
vm_block = 15; continue;
}
// C line 21015
17 => {
let _ = { let assigned = (0 as i32); *(pdone) = assigned; assigned };
vm_block = 12; continue;
}
// C line 21010
18 => {
vm_block = if ((((((((func_ret).u).uint64) as i32)) == ((2 as i32))) as i32)) != 0 { 16 } else { 17 }; continue;
}
// C line 21009
19 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; *((*(sf)).cur_sp).offset((((1 as i32)).wrapping_neg()) as isize) = assigned; assigned };
vm_block = 18; continue;
}
// C line 21008
20 => {
let _ = { let assigned = *((*(sf)).cur_sp).offset((((1 as i32)).wrapping_neg()) as isize); ret = assigned; assigned };
vm_block = 19; continue;
}
// C line 21006
21 => {
let _ = if ((((!((((((((func_ret).tag) as i32)) == ((JS_TAG_INT as i32))) as i32)) != 0) as i32)) as i64)) != 0 { std::process::abort() } else { { let _ = (0 as i32); } };
vm_block = 20; continue;
}
// C line 21001
22 => {
vm_block = if ((*((*(s)).func_state)).is_completed) != 0 { 14 } else { 21 }; continue;
}
// C line 21000
23 => {
let _ = { let assigned = (((JS_GENERATOR_STATE_SUSPENDED_YIELD as i32)) as JSGeneratorStateEnum); (*(s)).state = assigned; assigned };
vm_block = 22; continue;
}
// C line 20999
24 => {
let _ = { let assigned = async_func_resume(ctx, (*(s)).func_state); func_ret = assigned; assigned };
vm_block = 23; continue;
}
// C line 20998
25 => {
let _ = { let assigned = (((JS_GENERATOR_STATE_EXECUTING as i32)) as JSGeneratorStateEnum); (*(s)).state = assigned; assigned };
vm_block = 24; continue;
}
// C line 20990
26 => {
let _ = { let assigned = (1 as i32); (*((*(s)).func_state)).throw_flag = assigned; assigned };
vm_block = 25; continue;
}
// C line 20989
27 => {
let _ = JS_Throw(ctx, ret);
vm_block = 26; continue;
}
// C line ? labels: exec_no_arg
28 => {
let _ = { let assigned = (0 as i32); (*((*(s)).func_state)).throw_flag = assigned; assigned };
vm_block = 25; continue;
}
// C line 20994
29 => {
let _ = { let old = (*(sf)).cur_sp; (*(sf)).cur_sp = ((*(sf)).cur_sp).offset(1); old };
vm_block = 28; continue;
}
// C line 20993
30 => {
let _ = { let assigned = JS_NewInt32(ctx, magic); *((*(sf)).cur_sp).offset(((0 as i32)) as isize) = assigned; assigned };
vm_block = 29; continue;
}
// C line 20992
31 => {
let _ = { let assigned = ret; *((*(sf)).cur_sp).offset((((1 as i32)).wrapping_neg()) as isize) = assigned; assigned };
vm_block = 30; continue;
}
// C line 20987
32 => {
vm_block = if ((((((((magic) == ((2 as i32))) as i32)) != 0) && ((((((((*(s)).state) as u32)) == ((((JS_GENERATOR_STATE_SUSPENDED_YIELD as i32)) as u32))) as i32)) != 0)) as i32)) != 0 { 27 } else { 31 }; continue;
}
// C line 20986
33 => {
let _ = { let assigned = JS_DupValue(ctx, *(argv).offset(((0 as i32)) as isize)); ret = assigned; assigned };
vm_block = 32; continue;
}
// C line 20984
34 => {
let _ = { let assigned = core::ptr::addr_of_mut!((*((*(s)).func_state)).frame); sf = assigned; assigned };
vm_block = 33; continue;
}
// C line 20981
35 => {
vm_block = 1; continue;
}
// C line 20976
36 => {
vm_block = 28; continue;
}
// C line 20979
37 => {
vm_block = 11; continue;
}
// C line 20978
38 => {
let _ = free_generator_stack(ctx, s);
vm_block = 37; continue;
}
// C line 20975
39 => {
vm_block = if ((((magic) == ((0 as i32))) as i32)) != 0 { 36 } else { 38 }; continue;
}
// C line 20974
40 => {
let _ = { let assigned = core::ptr::addr_of_mut!((*((*(s)).func_state)).frame); sf = assigned; assigned };
vm_block = 39; continue;
}
// C line 20971
41 => {
vm_block = match (((*(s)).state) as u32) { x if x == (((JS_GENERATOR_STATE_EXECUTING as i32)) as u32) => 3, x if x == (((JS_GENERATOR_STATE_COMPLETED as i32)) as u32) => 11, x if x == (((JS_GENERATOR_STATE_SUSPENDED_YIELD as i32)) as u32) => 34, x if x == (((JS_GENERATOR_STATE_SUSPENDED_YIELD_STAR as i32)) as u32) => 34, x if x == (((JS_GENERATOR_STATE_SUSPENDED_START as i32)) as u32) => 40, _ => 40, }; continue;
}
// C line 20970
42 => {
return JS_ThrowTypeError(ctx, c"not a generator".as_ptr());
}
// C line 20969
43 => {
vm_block = if ((!(!(s).is_null()) as i32)) != 0 { 42 } else { 41 }; continue;
}
// C line 20968
44 => {
let _ = { let assigned = (1 as i32); *(pdone) = assigned; assigned };
vm_block = 43; continue;
}
// C line 20964
45 => {
s = ((JS_GetOpaque(this_val, (((JS_CLASS_GENERATOR as i32)) as JSClassID))) as *mut JSGeneratorData);
vm_block = 44; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:21042. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_generator_function_call(mut ctx: *mut JSContext, mut func_obj: JSValue, mut this_obj: JSValue, mut argc: i32, mut argv: *mut JSValue, mut flags: i32) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut obj: JSValue = core::mem::zeroed();
let mut func_ret: JSValue = core::mem::zeroed();
let mut s: *mut JSGeneratorData = core::mem::zeroed();
let mut vm_block: usize = 20;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 21074
1 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 21073
2 => {
let _ = js_free(ctx, ((s) as *mut c_void));
vm_block = 1; continue;
}
// C line ? labels: fail
3 => {
let _ = free_generator_stack_rt((*(ctx)).rt, s);
vm_block = 2; continue;
}
// C line 21070
4 => {
return obj;
}
// C line 21069
5 => {
let _ = JS_SetOpaque(obj, ((s) as *mut c_void));
vm_block = 4; continue;
}
// C line 21068
6 => {
vm_block = 3; continue;
}
// C line 21067
7 => {
vm_block = if (JS_IsException(obj)) != 0 { 6 } else { 5 }; continue;
}
// C line 21066
8 => {
let _ = { let assigned = js_create_from_ctor(ctx, func_obj, (JS_CLASS_GENERATOR as i32)); obj = assigned; assigned };
vm_block = 7; continue;
}
// C line 21064
9 => {
let _ = JS_FreeValue(ctx, func_ret);
vm_block = 8; continue;
}
// C line 21063
10 => {
vm_block = 3; continue;
}
// C line 21062
11 => {
vm_block = if (JS_IsException(func_ret)) != 0 { 10 } else { 9 }; continue;
}
// C line 21061
12 => {
let _ = { let assigned = async_func_resume(ctx, (*(s)).func_state); func_ret = assigned; assigned };
vm_block = 11; continue;
}
// C line 21057
13 => {
vm_block = 3; continue;
}
// C line 21056
14 => {
let _ = { let assigned = (((JS_GENERATOR_STATE_COMPLETED as i32)) as JSGeneratorStateEnum); (*(s)).state = assigned; assigned };
vm_block = 13; continue;
}
// C line 21055
15 => {
vm_block = if ((!(!((*(s)).func_state).is_null()) as i32)) != 0 { 14 } else { 12 }; continue;
}
// C line 21054
16 => {
let _ = { let assigned = async_func_init(ctx, func_obj, this_obj, argc, argv); (*(s)).func_state = assigned; assigned };
vm_block = 15; continue;
}
// C line 21053
17 => {
let _ = { let assigned = (((JS_GENERATOR_STATE_SUSPENDED_START as i32)) as JSGeneratorStateEnum); (*(s)).state = assigned; assigned };
vm_block = 16; continue;
}
// C line 21052
18 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 21051
19 => {
vm_block = if ((!(!(s).is_null()) as i32)) != 0 { 18 } else { 17 }; continue;
}
// C line 21050
20 => {
let _ = { let assigned = ((js_mallocz(ctx, (size_of::<JSGeneratorData>() as usize))) as *mut JSGeneratorData); s = assigned; assigned };
vm_block = 19; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:21098. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_async_function_resolve_create(mut ctx: *mut JSContext, mut s: *mut JSAsyncFunctionState, mut resolving_funcs: *mut JSValue) -> i32 {
let mut vm_local_storage = Vec::<u64>::new();
let mut i: i32 = core::mem::zeroed();
let mut p: *mut JSObject = core::mem::zeroed();
let mut vm_block: usize = 12;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 21118
1 => {
return (0 as i32);
}
// C line 21105
2 => {
vm_block = if ((((i) < ((2 as i32))) as i32)) != 0 { 11 } else { 1 }; continue;
}
// C line ?
3 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 2; continue;
}
// C line 21116
4 => {
let _ = { let assigned = s; ((*(p)).u).async_function_data = assigned; assigned };
vm_block = 3; continue;
}
// C line 21115
5 => {
let _ = { let old = (*(js_rc(((s) as *mut c_void)))).ref_count; (*(js_rc(((s) as *mut c_void)))).ref_count = ((*(js_rc(((s) as *mut c_void)))).ref_count).wrapping_add(1); old };
vm_block = 4; continue;
}
// C line 21114
6 => {
let _ = { let assigned = ((((*(resolving_funcs).offset((i) as isize)).u).ptr) as *mut JSObject); p = assigned; assigned };
vm_block = 5; continue;
}
// C line 21112
7 => {
return ((1 as i32)).wrapping_neg();
}
// C line 21111
8 => {
let _ = JS_FreeValue(ctx, *(resolving_funcs).offset(((0 as i32)) as isize));
vm_block = 7; continue;
}
// C line 21110
9 => {
vm_block = if ((((i) == ((1 as i32))) as i32)) != 0 { 8 } else { 7 }; continue;
}
// C line 21109
10 => {
vm_block = if (JS_IsException(*(resolving_funcs).offset((i) as isize))) != 0 { 9 } else { 6 }; continue;
}
// C line 21106
11 => {
let _ = { let assigned = JS_NewObjectProtoClass(ctx, (*(ctx)).function_proto, ((((JS_CLASS_ASYNC_FUNCTION_RESOLVE as i32)).wrapping_add(i)) as JSClassID)); *(resolving_funcs).offset((i) as isize) = assigned; assigned };
vm_block = 10; continue;
}
// C line 21105
12 => {
let _ = { let assigned = (0 as i32); i = assigned; assigned };
vm_block = 2; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:21121. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_async_function_resume(mut ctx: *mut JSContext, mut s: *mut JSAsyncFunctionState) -> () {
let mut vm_local_storage = Vec::<u64>::new();
let mut func_ret: JSValue = core::mem::zeroed();
let mut ret2: JSValue = core::mem::zeroed();
let mut error: JSValue = core::mem::zeroed();
let mut value: JSValue = core::mem::zeroed();
let mut promise: JSValue = core::mem::zeroed();
let mut resolving_funcs: [JSValue; 2] = core::mem::zeroed();
let mut resolving_funcs1: [JSValue; 2] = core::mem::zeroed();
let mut i: i32 = core::mem::zeroed();
let mut res: i32 = core::mem::zeroed();
let mut vm_block: usize = 32;
loop { match vm_block {
// C line ?
0 => {
return ();
}
// C line 21134
1 => {
let _ = JS_FreeValue(ctx, ret2);
vm_block = 0; continue;
}
// C line 21133
2 => {
let _ = JS_FreeValue(ctx, error);
vm_block = 1; continue;
}
// C line 21131
3 => {
let _ = { let assigned = JS_Call(ctx, *(((*(s)).resolving_funcs).as_mut_ptr()).offset(((1 as i32)) as isize), JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }, (1 as i32), core::ptr::addr_of_mut!(error)); ret2 = assigned; assigned };
vm_block = 2; continue;
}
// C line ? labels: fail
4 => {
let _ = { let assigned = JS_GetException(ctx); error = assigned; assigned };
vm_block = 3; continue;
}
// C line 21140
5 => {
let _ = JS_FreeValue(ctx, ret2);
vm_block = 0; continue;
}
// C line 21139
6 => {
let _ = JS_FreeValue(ctx, func_ret);
vm_block = 5; continue;
}
// C line 21137
7 => {
let _ = { let assigned = JS_Call(ctx, *(((*(s)).resolving_funcs).as_mut_ptr()).offset(((0 as i32)) as isize), JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }, (1 as i32), core::ptr::addr_of_mut!(func_ret)); ret2 = assigned; assigned };
vm_block = 6; continue;
}
// C line 21127
8 => {
vm_block = if (JS_IsException(func_ret)) != 0 { 4 } else { 7 }; continue;
}
// C line 21172
9 => {
vm_block = 4; continue;
}
// C line 21171
10 => {
vm_block = if (res) != 0 { 9 } else { 0 }; continue;
}
// C line 21169
11 => {
vm_block = if ((((i) < ((2 as i32))) as i32)) != 0 { 13 } else { 10 }; continue;
}
// C line ?
12 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 11; continue;
}
// C line 21170
13 => {
let _ = JS_FreeValue(ctx, *((resolving_funcs).as_mut_ptr()).offset((i) as isize));
vm_block = 12; continue;
}
// C line 21169
14 => {
let _ = { let assigned = (0 as i32); i = assigned; assigned };
vm_block = 11; continue;
}
// C line 21168
15 => {
let _ = JS_FreeValue(ctx, promise);
vm_block = 14; continue;
}
// C line 21165
16 => {
let _ = { let assigned = perform_promise_then(ctx, promise, (resolving_funcs).as_mut_ptr(), (resolving_funcs1).as_mut_ptr()); res = assigned; assigned };
vm_block = 15; continue;
}
// C line 21163
17 => {
vm_block = if ((((i) < ((2 as i32))) as i32)) != 0 { 19 } else { 16 }; continue;
}
// C line ?
18 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 17; continue;
}
// C line 21164
19 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; *((resolving_funcs1).as_mut_ptr()).offset((i) as isize) = assigned; assigned };
vm_block = 18; continue;
}
// C line 21163
20 => {
let _ = { let assigned = (0 as i32); i = assigned; assigned };
vm_block = 17; continue;
}
// C line 21158
21 => {
vm_block = 4; continue;
}
// C line 21157
22 => {
let _ = JS_FreeValue(ctx, promise);
vm_block = 21; continue;
}
// C line 21156
23 => {
vm_block = if (js_async_function_resolve_create(ctx, s, (resolving_funcs).as_mut_ptr())) != 0 { 22 } else { 20 }; continue;
}
// C line 21155
24 => {
vm_block = 4; continue;
}
// C line 21154
25 => {
vm_block = if (JS_IsException(promise)) != 0 { 24 } else { 23 }; continue;
}
// C line 21153
26 => {
let _ = JS_FreeValue(ctx, value);
vm_block = 25; continue;
}
// C line 21151
27 => {
let _ = { let assigned = js_promise_resolve(ctx, (*(ctx)).promise_ctor, (1 as i32), core::ptr::addr_of_mut!(value), (0 as i32)); promise = assigned; assigned };
vm_block = 26; continue;
}
// C line 21150
28 => {
let _ = JS_FreeValue(ctx, func_ret);
vm_block = 27; continue;
}
// C line 21147
29 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; *(((*(s)).frame).cur_sp).offset((((1 as i32)).wrapping_neg()) as isize) = assigned; assigned };
vm_block = 28; continue;
}
// C line 21146
30 => {
let _ = { let assigned = *(((*(s)).frame).cur_sp).offset((((1 as i32)).wrapping_neg()) as isize); value = assigned; assigned };
vm_block = 29; continue;
}
// C line 21126
31 => {
vm_block = if ((*(s)).is_completed) != 0 { 8 } else { 30 }; continue;
}
// C line 21125
32 => {
let _ = { let assigned = async_func_resume(ctx, s); func_ret = assigned; assigned };
vm_block = 31; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:21176. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_async_function_resolve_call(mut ctx: *mut JSContext, mut func_obj: JSValue, mut this_obj: JSValue, mut argc: i32, mut argv: *mut JSValue, mut flags: i32) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut p: *mut JSObject = core::mem::zeroed();
let mut s: *mut JSAsyncFunctionState = core::mem::zeroed();
let mut is_reject: i32 = core::mem::zeroed();
let mut arg: JSValue = core::mem::zeroed();
let mut vm_block: usize = 12;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 21199
1 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) };
}
// C line 21198
2 => {
let _ = js_async_function_resume(ctx, s);
vm_block = 1; continue;
}
// C line 21193
3 => {
let _ = JS_Throw(ctx, JS_DupValue(ctx, arg));
vm_block = 2; continue;
}
// C line 21196
4 => {
let _ = { let assigned = JS_DupValue(ctx, arg); *(((*(s)).frame).cur_sp).offset((((1 as i32)).wrapping_neg()) as isize) = assigned; assigned };
vm_block = 2; continue;
}
// C line 21192
5 => {
vm_block = if (is_reject) != 0 { 3 } else { 4 }; continue;
}
// C line 21191
6 => {
let _ = { let assigned = is_reject; (*(s)).throw_flag = assigned; assigned };
vm_block = 5; continue;
}
// C line 21188
7 => {
let _ = { let assigned = *(argv).offset(((0 as i32)) as isize); arg = assigned; assigned };
vm_block = 6; continue;
}
// C line 21190
8 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; arg = assigned; assigned };
vm_block = 6; continue;
}
// C line 21187
9 => {
vm_block = if ((((argc) > ((0 as i32))) as i32)) != 0 { 7 } else { 8 }; continue;
}
// C line 21184
10 => {
is_reject = ((((*(p)).class_id) as i32)).wrapping_sub((JS_CLASS_ASYNC_FUNCTION_RESOLVE as i32));
vm_block = 9; continue;
}
// C line 21183
11 => {
s = ((*(p)).u).async_function_data;
vm_block = 10; continue;
}
// C line 21182
12 => {
p = ((((func_obj).u).ptr) as *mut JSObject);
vm_block = 11; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:21202. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_async_function_call(mut ctx: *mut JSContext, mut func_obj: JSValue, mut this_obj: JSValue, mut argc: i32, mut argv: *mut JSValue, mut flags: i32) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut promise: JSValue = core::mem::zeroed();
let mut s: *mut JSAsyncFunctionState = core::mem::zeroed();
let mut vm_block: usize = 10;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 21223
1 => {
return promise;
}
// C line 21221
2 => {
let _ = async_func_free((*(ctx)).rt, s);
vm_block = 1; continue;
}
// C line 21219
3 => {
let _ = js_async_function_resume(ctx, s);
vm_block = 2; continue;
}
// C line 21216
4 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 21215
5 => {
let _ = async_func_free((*(ctx)).rt, s);
vm_block = 4; continue;
}
// C line 21214
6 => {
vm_block = if (JS_IsException(promise)) != 0 { 5 } else { 3 }; continue;
}
// C line 21213
7 => {
let _ = { let assigned = JS_NewPromiseCapability(ctx, ((*(s)).resolving_funcs).as_mut_ptr()); promise = assigned; assigned };
vm_block = 6; continue;
}
// C line 21211
8 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 21210
9 => {
vm_block = if ((!(!(s).is_null()) as i32)) != 0 { 8 } else { 7 }; continue;
}
// C line 21209
10 => {
let _ = { let assigned = async_func_init(ctx, func_obj, this_obj, argc, argv); s = assigned; assigned };
vm_block = 9; continue;
}
_ => std::process::abort(),
} }
}
