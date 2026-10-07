// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:37031. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn JS_EvalFunctionInternal(mut ctx: *mut JSContext, mut fun_obj: JSValue, mut this_obj: JSValue, mut var_refs: *mut *mut JSVarRef, mut sf: *mut JSStackFrame) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut ret_val: JSValue = core::mem::zeroed();
let mut tag: u32 = 0;
let mut m: *mut JSModuleDef = core::ptr::null_mut();
let mut vm_block: usize = 19;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 37062
1 => {
return ret_val;
}
// C line 37043
2 => {
let _ = { let assigned = JS_CallFree(ctx, fun_obj, this_obj, (0 as i32), core::ptr::null_mut::<JSValue>()); ret_val = assigned; assigned };
vm_block = 1; continue;
}
// C line 37042
3 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 37041
4 => {
vm_block = if (JS_IsException(fun_obj)) != 0 { 3 } else { 2 }; continue;
}
// C line 37040
5 => {
let _ = { let assigned = js_closure(ctx, fun_obj, var_refs, sf, (1 as i32)); fun_obj = assigned; assigned };
vm_block = 4; continue;
}
// C line 37056 labels: fail
6 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 37054
7 => {
vm_block = if (JS_IsException(ret_val)) != 0 { 6 } else { 1 }; continue;
}
// C line 37053
8 => {
let _ = { let assigned = js_evaluate_module(ctx, m); ret_val = assigned; assigned };
vm_block = 7; continue;
}
// C line 37052
9 => {
vm_block = 6; continue;
}
// C line 37051
10 => {
vm_block = if ((((js_link_module(ctx, m)) < ((0 as i32))) as i32)) != 0 { 9 } else { 8 }; continue;
}
// C line 37050
11 => {
vm_block = 6; continue;
}
// C line 37049
12 => {
vm_block = if ((((js_create_module_function(ctx, m)) < ((0 as i32))) as i32)) != 0 { 11 } else { 10 }; continue;
}
// C line 37048
13 => {
let _ = JS_FreeValue(ctx, fun_obj);
vm_block = 12; continue;
}
// C line 37046
14 => {
let _ = { let assigned = ((((fun_obj).u).ptr) as *mut JSModuleDef); m = assigned; assigned };
vm_block = 13; continue;
}
// C line 37060
15 => {
let _ = { let assigned = JS_ThrowTypeError(ctx, c"bytecode function expected".as_ptr()); ret_val = assigned; assigned };
vm_block = 1; continue;
}
// C line 37059
16 => {
let _ = JS_FreeValue(ctx, fun_obj);
vm_block = 15; continue;
}
// C line 37044
17 => {
vm_block = if ((((tag) == ((((JS_TAG_MODULE as i32)) as u32))) as i32)) != 0 { 14 } else { 16 }; continue;
}
// C line 37039
18 => {
vm_block = if ((((tag) == ((((JS_TAG_FUNCTION_BYTECODE as i32)) as u32))) as i32)) != 0 { 5 } else { 17 }; continue;
}
// C line 37038
19 => {
let _ = { let assigned = (((((fun_obj).tag) as i32)) as u32); tag = assigned; assigned };
vm_block = 18; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:37065. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
pub unsafe fn JS_EvalFunction(mut ctx: *mut JSContext, mut fun_obj: JSValue) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut vm_block: usize = 1;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 37067
1 => {
return JS_EvalFunctionInternal(ctx, fun_obj, (*(ctx)).global_obj, core::ptr::null_mut::<*mut JSVarRef>(), core::ptr::null_mut::<JSStackFrame>());
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:37071. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn __JS_EvalInternal(mut ctx: *mut JSContext, mut this_obj: JSValue, mut input: *const c_char, mut input_len: usize, mut filename: *const c_char, mut flags: i32, mut scope_idx: i32) -> JSValue {
let _host_source = JSHostSourceScope::new(ctx, input, input_len);
let mut vm_local_storage = Vec::<u64>::new();
let mut s1: JSParseState = core::mem::zeroed();
let mut s: *mut JSParseState = core::ptr::null_mut();
let mut err: i32 = 0;
let mut js_mode: i32 = 0;
let mut eval_type: i32 = 0;
let mut fun_obj: JSValue = core::mem::zeroed();
let mut ret_val: JSValue = core::mem::zeroed();
let mut sf: *mut JSStackFrame = core::ptr::null_mut();
let mut var_refs: *mut *mut JSVarRef = core::ptr::null_mut();
let mut b: *mut JSFunctionBytecode = core::ptr::null_mut();
let mut fd: *mut JSFunctionDef = core::ptr::null_mut();
let mut m: *mut JSModuleDef = core::ptr::null_mut();
let mut p: *mut JSObject = core::ptr::null_mut();
let mut module_name: JSAtom = 0;
let mut vm_block: usize = 78;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 37183
1 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 37182
2 => {
let _ = JS_FreeValue(ctx, JSValue { u: JSValueUnion { ptr: ((m) as *mut c_void) }, tag: (((JS_TAG_MODULE as i32)) as i64) });
vm_block = 1; continue;
}
// C line 37181 labels: fail1
3 => {
vm_block = if !(m).is_null() { 2 } else { 1 }; continue;
}
// C line 37178
4 => {
return ret_val;
}
// C line 37174
5 => {
let _ = { let assigned = fun_obj; ret_val = assigned; assigned };
vm_block = 4; continue;
}
// C line 37176
6 => {
let _ = { let assigned = JS_EvalFunctionInternal(ctx, fun_obj, this_obj, var_refs, sf); ret_val = assigned; assigned };
vm_block = 4; continue;
}
// C line 37173
7 => {
vm_block = if (((flags) & (((1 as i32)).wrapping_shl(((5 as i32)) as u32)))) != 0 { 5 } else { 6 }; continue;
}
// C line 37171
8 => {
let _ = { let assigned = JS_NewModuleValue(ctx, m); fun_obj = assigned; assigned };
vm_block = 7; continue;
}
// C line 37170
9 => {
vm_block = 3; continue;
}
// C line 37169
10 => {
vm_block = if js_host_should_resolve_module(flags) && ((((js_resolve_module(ctx, m)) < ((0 as i32))) as i32)) != 0 { 9 } else { 8 }; continue;
}
// C line 37168
11 => {
let _ = { let assigned = fun_obj; (*(m)).func_obj = assigned; assigned };
vm_block = 10; continue;
}
// C line 37167
12 => {
vm_block = if !(m).is_null() { 11 } else { 7 }; continue;
}
// C line 37165
13 => {
vm_block = 3; continue;
}
// C line 37164
14 => {
vm_block = if (JS_IsException(fun_obj)) != 0 { 13 } else { 12 }; continue;
}
// C line 37163
15 => {
let _ = { let assigned = js_create_function(ctx, fd); fun_obj = assigned; assigned };
js_host_register_evaluated_source(ctx, fun_obj);
vm_block = 14; continue;
}
// C line 37160
16 => {
let _ = { let assigned = (*(fd)).has_await; (*(m)).has_tla = (assigned) as i8; assigned };
vm_block = 15; continue;
}
// C line 37159
17 => {
vm_block = if ((((m) != (core::ptr::null_mut::<JSModuleDef>())) as i32)) != 0 { 16 } else { 15 }; continue;
}
// C line 37156
18 => {
vm_block = 3; continue;
}
// C line 37155
19 => {
let _ = js_free_function_def(ctx, fd);
vm_block = 18; continue;
}
// C line 37154 labels: fail
20 => {
let _ = free_token(s, core::ptr::addr_of_mut!((*(s)).token));
vm_block = 19; continue;
}
// C line 37152
21 => {
vm_block = if (err) != 0 { 20 } else { 17 }; continue;
}
// C line 37151
22 => {
let _ = { let assigned = js_parse_program(s); err = assigned; assigned };
vm_block = 21; continue;
}
// C line 37149
23 => {
let _ = { let assigned = (*(fd)).scope_level; (*(fd)).body_scope = assigned; assigned };
vm_block = 22; continue;
}
// C line 37148
24 => {
let _ = push_scope(s);
vm_block = 23; continue;
}
// C line 37146
25 => {
let _ = { let assigned = (!(((*(s)).is_module) != 0) as i32); (*(s)).allow_html_comments = assigned; assigned };
vm_block = 24; continue;
}
// C line 37145
26 => {
let _ = { let assigned = (((m) != (core::ptr::null_mut::<JSModuleDef>())) as i32); (*(s)).is_module = assigned; assigned };
vm_block = 25; continue;
}
// C line 37143
27 => {
let _ = { let assigned = (((JS_FUNC_ASYNC as i32)) as JSFunctionKindEnum); (*(fd)).func_kind = (assigned) as u8; assigned };
vm_block = 26; continue;
}
// C line 37142
28 => {
let _ = { let assigned = (1 as i32); (*(fd)).in_function_body = assigned; assigned };
vm_block = 27; continue;
}
// C line 37141
29 => {
vm_block = if ((((((((m) != (core::ptr::null_mut::<JSModuleDef>())) as i32)) != 0) || ((((flags) & (((1 as i32)).wrapping_shl(((7 as i32)) as u32)))) != 0)) as i32)) != 0 { 28 } else { 26 }; continue;
}
// C line 37140
30 => {
let _ = { let assigned = m; (*(fd)).module = assigned; assigned };
vm_block = 29; continue;
}
// C line 37138
31 => {
vm_block = 20; continue;
}
// C line 37137
32 => {
vm_block = if (add_closure_variables(ctx, fd, b, scope_idx)) != 0 { 31 } else { 30 }; continue;
}
// C line 37136
33 => {
vm_block = if !(b).is_null() { 32 } else { 30 }; continue;
}
// C line 37135
34 => {
let _ = { let assigned = JS_DupAtom(ctx, (((crate::quickjs_atom::JS_ATOM__eval_ as i32)) as JSAtom)); (*(fd)).func_name = assigned; assigned };
vm_block = 33; continue;
}
// C line 37134
35 => {
let _ = { let assigned = ((js_mode) as u8); (*(fd)).js_mode = (assigned) as u8; assigned };
vm_block = 34; continue;
}
// C line 37127
36 => {
let _ = { let assigned = (((*(b)).arguments_allowed()) as i32); (*(fd)).arguments_allowed = assigned; assigned };
vm_block = 35; continue;
}
// C line 37126
37 => {
let _ = { let assigned = (((*(b)).super_allowed()) as i32); (*(fd)).super_allowed = assigned; assigned };
vm_block = 36; continue;
}
// C line 37125
38 => {
let _ = { let assigned = (((*(b)).super_call_allowed()) as i32); (*(fd)).super_call_allowed = assigned; assigned };
vm_block = 37; continue;
}
// C line 37124
39 => {
let _ = { let assigned = (((*(b)).new_target_allowed()) as i32); (*(fd)).new_target_allowed = assigned; assigned };
vm_block = 38; continue;
}
// C line 37132
40 => {
let _ = { let assigned = (1 as i32); (*(fd)).arguments_allowed = assigned; assigned };
vm_block = 35; continue;
}
// C line 37131
41 => {
let _ = { let assigned = (0 as i32); (*(fd)).super_allowed = assigned; assigned };
vm_block = 40; continue;
}
// C line 37130
42 => {
let _ = { let assigned = (0 as i32); (*(fd)).super_call_allowed = assigned; assigned };
vm_block = 41; continue;
}
// C line 37129
43 => {
let _ = { let assigned = (0 as i32); (*(fd)).new_target_allowed = assigned; assigned };
vm_block = 42; continue;
}
// C line 37123
44 => {
vm_block = if ((((eval_type) == (((2 as i32)).wrapping_shl(((0 as i32)) as u32))) as i32)) != 0 { 39 } else { 43 }; continue;
}
// C line 37122
45 => {
let _ = { let assigned = (((eval_type) != (((2 as i32)).wrapping_shl(((0 as i32)) as u32))) as i32); (*(fd)).has_this_binding = assigned; assigned };
vm_block = 44; continue;
}
// C line 37121
46 => {
let _ = { let assigned = eval_type; (*(fd)).eval_type = assigned; assigned };
vm_block = 45; continue;
}
// C line 37120
47 => {
let _ = { let assigned = fd; (*(s)).cur_func = assigned; assigned };
vm_block = 46; continue;
}
// C line 37119
48 => {
vm_block = 3; continue;
}
// C line 37118
49 => {
vm_block = if ((!(!(fd).is_null()) as i32)) != 0 { 48 } else { 47 }; continue;
}
// C line 37116
50 => {
let _ = { let assigned = js_new_function_def(ctx, core::ptr::null_mut::<JSFunctionDef>(), (1 as i32), (0 as i32), filename, (*(s)).buf_start, core::ptr::addr_of_mut!((*(s)).get_line_col_cache)); fd = assigned; assigned };
vm_block = 49; continue;
}
// C line 37098
51 => {
let _ = { let assigned = (((*(b)).js_mode) as i32); js_mode = assigned; assigned };
vm_block = 50; continue;
}
// C line 37097
52 => {
let _ = { let assigned = (((*(p)).u).func).var_refs; var_refs = assigned; assigned };
vm_block = 51; continue;
}
// C line 37096
53 => {
let _ = { let assigned = (((*(p)).u).func).function_bytecode; b = assigned; assigned };
vm_block = 52; continue;
}
// C line 37095
54 => {
let _ = if ((((!((js_class_has_bytecode((((*(p)).class_id) as JSClassID))) != 0) as i32)) as i64)) != 0 { std::process::abort() } else { { let _ = (0 as i32); } };
vm_block = 53; continue;
}
// C line 37094
55 => {
let _ = { let assigned = (((((*(sf)).cur_func).u).ptr) as *mut JSObject); p = assigned; assigned };
vm_block = 54; continue;
}
// C line 37093
56 => {
let _ = if ((((!(((((((((*(sf)).cur_func).tag) as i32)) == ((JS_TAG_OBJECT as i32))) as i32)) != 0) as i32)) as i64)) != 0 { std::process::abort() } else { { let _ = (0 as i32); } };
vm_block = 55; continue;
}
// C line 37092
57 => {
let _ = if ((((!(((((sf) != (core::ptr::null_mut::<JSStackFrame>())) as i32)) != 0) as i32)) as i64)) != 0 { std::process::abort() } else { { let _ = (0 as i32); } };
vm_block = 56; continue;
}
// C line 37091
58 => {
let _ = { let assigned = (*((*(ctx)).rt)).current_stack_frame; sf = assigned; assigned };
vm_block = 57; continue;
}
// C line 37113
59 => {
let _ = { js_mode = (((((js_mode) as i32)) | (((1 as i32)).wrapping_shl(((0 as i32)) as u32)))) as i32; js_mode };
vm_block = 50; continue;
}
// C line 37112
60 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 37111
61 => {
vm_block = if ((!(!(m).is_null()) as i32)) != 0 { 60 } else { 59 }; continue;
}
// C line 37110
62 => {
let _ = { let assigned = js_new_module_def(ctx, module_name); m = assigned; assigned };
vm_block = 61; continue;
}
// C line 37109
63 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 37108
64 => {
vm_block = if ((((module_name) == ((((0 as i32)) as JSAtom))) as i32)) != 0 { 63 } else { 62 }; continue;
}
// C line 37107
65 => {
module_name = JS_NewAtom(ctx, filename);
vm_block = 64; continue;
}
// C line 37106
66 => {
vm_block = if ((((eval_type) == (((1 as i32)).wrapping_shl(((0 as i32)) as u32))) as i32)) != 0 { 65 } else { 50 }; continue;
}
// C line 37105
67 => {
let _ = { js_mode = (((((js_mode) as i32)) | (((1 as i32)).wrapping_shl(((0 as i32)) as u32)))) as i32; js_mode };
vm_block = 66; continue;
}
// C line 37104
68 => {
vm_block = if (((flags) & (((1 as i32)).wrapping_shl(((3 as i32)) as u32)))) != 0 { 67 } else { 66 }; continue;
}
// C line 37103
69 => {
let _ = { let assigned = (0 as i32); js_mode = assigned; assigned };
vm_block = 68; continue;
}
// C line 37102
70 => {
let _ = { let assigned = core::ptr::null_mut::<*mut JSVarRef>(); var_refs = assigned; assigned };
vm_block = 69; continue;
}
// C line 37101
71 => {
let _ = { let assigned = core::ptr::null_mut::<JSFunctionBytecode>(); b = assigned; assigned };
vm_block = 70; continue;
}
// C line 37100
72 => {
let _ = { let assigned = core::ptr::null_mut::<JSStackFrame>(); sf = assigned; assigned };
vm_block = 71; continue;
}
// C line 37089
73 => {
vm_block = if ((((eval_type) == (((2 as i32)).wrapping_shl(((0 as i32)) as u32))) as i32)) != 0 { 58 } else { 72 }; continue;
}
// C line 37088
74 => {
let _ = { let assigned = core::ptr::null_mut::<JSModuleDef>(); m = assigned; assigned };
vm_block = 73; continue;
}
// C line 37087
75 => {
let _ = { let assigned = ((flags) & (((3 as i32)).wrapping_shl(((0 as i32)) as u32))); eval_type = assigned; assigned };
vm_block = 74; continue;
}
// C line 37085
76 => {
let _ = skip_shebang(core::ptr::addr_of_mut!((*(s)).buf_ptr), (*(s)).buf_end);
vm_block = 75; continue;
}
// C line 37084
77 => {
let _ = js_parse_init(ctx, s, input, input_len, filename);
vm_block = 76; continue;
}
// C line 37075
78 => {
s = core::ptr::addr_of_mut!(s1);
vm_block = 77; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:37187. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn JS_EvalInternal(mut ctx: *mut JSContext, mut this_obj: JSValue, mut input: *const c_char, mut input_len: usize, mut filename: *const c_char, mut flags: i32, mut scope_idx: i32) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut backtrace_barrier: i32 = 0;
let mut saved_js_mode: i32 = 0;
let mut ret: JSValue = core::mem::zeroed();
let mut vm_block: usize = 11;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 37206
1 => {
return ret;
}
// C line 37205
2 => {
let _ = { let assigned = saved_js_mode; (*((*((*(ctx)).rt)).current_stack_frame)).js_mode = assigned; assigned };
vm_block = 1; continue;
}
// C line 37204
3 => {
vm_block = if (((((backtrace_barrier) != 0) && (!((*((*(ctx)).rt)).current_stack_frame).is_null())) as i32)) != 0 { 2 } else { 1 }; continue;
}
// C line 37202
4 => {
let _ = { let assigned = ((*(ctx)).eval_internal).expect("registered parser callback")(ctx, this_obj, input, input_len, filename, flags, scope_idx); ret = assigned; assigned };
vm_block = 3; continue;
}
// C line 37200
5 => {
let _ = { (*((*((*(ctx)).rt)).current_stack_frame)).js_mode = ((((((*((*((*(ctx)).rt)).current_stack_frame)).js_mode) as i32)) | (((1 as i32)).wrapping_shl(((3 as i32)) as u32)))) as i32; (*((*((*(ctx)).rt)).current_stack_frame)).js_mode };
vm_block = 4; continue;
}
// C line 37199
6 => {
let _ = { let assigned = (*((*((*(ctx)).rt)).current_stack_frame)).js_mode; saved_js_mode = assigned; assigned };
vm_block = 5; continue;
}
// C line 37198
7 => {
vm_block = if (((((backtrace_barrier) != 0) && (!((*((*(ctx)).rt)).current_stack_frame).is_null())) as i32)) != 0 { 6 } else { 4 }; continue;
}
// C line 37196
8 => {
return JS_ThrowTypeError(ctx, c"eval is not supported".as_ptr());
}
// C line 37195
9 => {
vm_block = if ((((!(((!(((!(((*(ctx)).eval_internal).is_some()) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 8 } else { 7 }; continue;
}
// C line 37192
10 => {
saved_js_mode = (0 as i32);
vm_block = 9; continue;
}
// C line 37191
11 => {
backtrace_barrier = (((((flags) & (((1 as i32)).wrapping_shl(((6 as i32)) as u32)))) != ((0 as i32))) as i32);
vm_block = 10; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:37209. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn JS_EvalObject(mut ctx: *mut JSContext, mut this_obj: JSValue, mut val: JSValue, mut flags: i32, mut scope_idx: i32) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut ret: JSValue = core::mem::zeroed();
let mut str: *const c_char = core::ptr::null();
let mut len: usize = 0;
let mut vm_block: usize = 8;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 37223
1 => {
return ret;
}
// C line 37222
2 => {
let _ = JS_FreeCString(ctx, str);
vm_block = 1; continue;
}
// C line 37221
3 => {
let _ = { let assigned = JS_EvalInternal(ctx, this_obj, str, len, c"<input>".as_ptr(), flags, scope_idx); ret = assigned; assigned };
vm_block = 2; continue;
}
// C line 37220
4 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 37219
5 => {
vm_block = if ((!(!(str).is_null()) as i32)) != 0 { 4 } else { 3 }; continue;
}
// C line 37218
6 => {
let _ = { let assigned = JS_ToCStringLen(ctx, core::ptr::addr_of_mut!(len), val); str = assigned; assigned };
vm_block = 5; continue;
}
// C line 37217
7 => {
return JS_DupValue(ctx, val);
}
// C line 37216
8 => {
vm_block = if ((!((JS_IsString(val)) != 0) as i32)) != 0 { 7 } else { 6 }; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:37226. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
pub unsafe fn JS_EvalThis(mut ctx: *mut JSContext, mut this_obj: JSValue, mut input: *const c_char, mut input_len: usize, mut filename: *const c_char, mut eval_flags: i32) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut eval_type: i32 = 0;
let mut ret: JSValue = core::mem::zeroed();
let mut vm_block: usize = 4;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 37237
1 => {
return ret;
}
// C line 37235
2 => {
let _ = { let assigned = JS_EvalInternal(ctx, this_obj, input, input_len, filename, eval_flags, ((1 as i32)).wrapping_neg()); ret = assigned; assigned };
vm_block = 1; continue;
}
// C line 37233
3 => {
let _ = if ((((!(((((((((eval_type) == (((0 as i32)).wrapping_shl(((0 as i32)) as u32))) as i32)) != 0) || (((((eval_type) == (((1 as i32)).wrapping_shl(((0 as i32)) as u32))) as i32)) != 0)) as i32)) != 0) as i32)) as i64)) != 0 { std::process::abort() } else { { let _ = (0 as i32); } };
vm_block = 2; continue;
}
// C line 37230
4 => {
eval_type = ((eval_flags) & (((3 as i32)).wrapping_shl(((0 as i32)) as u32)));
vm_block = 3; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:37240. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
pub unsafe fn JS_Eval(mut ctx: *mut JSContext, mut input: *const c_char, mut input_len: usize, mut filename: *const c_char, mut eval_flags: i32) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut vm_block: usize = 1;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 37243
1 => {
return JS_EvalThis(ctx, (*(ctx)).global_obj, input, input_len, filename, eval_flags);
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:37247. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
pub unsafe fn JS_ResolveModule(mut ctx: *mut JSContext, mut obj: JSValue) -> i32 {
let mut vm_local_storage = Vec::<u64>::new();
let mut m: *mut JSModuleDef = core::ptr::null_mut();
let mut vm_block: usize = 6;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 37256
1 => {
return (0 as i32);
}
// C line 37253
2 => {
return ((1 as i32)).wrapping_neg();
}
// C line 37252
3 => {
let _ = js_free_modules(ctx, (((JS_FREE_MODULE_NOT_RESOLVED as i32)) as JSFreeModuleEnum));
vm_block = 2; continue;
}
// C line 37251
4 => {
vm_block = if ((((js_resolve_module(ctx, m)) < ((0 as i32))) as i32)) != 0 { 3 } else { 1 }; continue;
}
// C line 37250
5 => {
m = ((((obj).u).ptr) as *mut JSModuleDef);
vm_block = 4; continue;
}
// C line 37249
6 => {
vm_block = if (((((((obj).tag) as i32)) == ((JS_TAG_MODULE as i32))) as i32)) != 0 { 5 } else { 1 }; continue;
}
_ => std::process::abort(),
} }
}
