// Generated from the official C AST; full opcode dispatch and original goto CFG.
// C: quickjs.c:17771..20601. Bellard/Gordon MIT.
#[cfg(feature = "short-opcodes")]
#[allow(unused_parens, unused_mut, unused_assignments)]
#[deny(unreachable_patterns)]
unsafe fn JS_CallInternal(mut caller_ctx: *mut JSContext, mut func_obj: JSValue, mut this_obj: JSValue, mut new_target: JSValue, mut argc: i32, mut argv: *mut JSValue, mut flags: i32) -> JSValue {
let mut vm_local_storage = VmCallFrameStorage::new();
let mut rt: *mut JSRuntime = core::ptr::null_mut();
let mut ctx: *mut JSContext = core::ptr::null_mut();
let mut p: *mut JSObject = core::ptr::null_mut();
let mut b: *mut JSFunctionBytecode = core::ptr::null_mut();
let mut sf_s: JSStackFrame = core::mem::zeroed();
let mut sf: *mut JSStackFrame = core::ptr::null_mut();
let mut pc: *const u8 = core::ptr::null();
let mut opcode: i32 = 0;
let mut arg_allocated_size: i32 = 0;
let mut i: i32 = 0;
let mut local_buf: *mut JSValue = core::ptr::null_mut();
let mut stack_buf: *mut JSValue = core::ptr::null_mut();
let mut var_buf: *mut JSValue = core::ptr::null_mut();
let mut arg_buf: *mut JSValue = core::ptr::null_mut();
let mut sp: *mut JSValue = core::ptr::null_mut();
let mut ret_val: JSValue = core::mem::zeroed();
let mut pval: *mut JSValue = core::ptr::null_mut();
let mut var_refs: *mut *mut JSVarRef = core::ptr::null_mut();
let mut alloca_size: usize = 0;
let mut s: *mut JSAsyncFunctionState = core::ptr::null_mut();
let mut call_func: Option<JSClassCall> = core::mem::zeroed();
let mut n: i32 = 0;
let mut call_argc: i32 = 0;
let mut call_argv: *mut JSValue = core::ptr::null_mut();
let mut val: JSValue = core::mem::zeroed();
let mut tag: u32 = 0;
let mut arg: i32 = 0;
let mut p1: *mut JSObject = core::ptr::null_mut();
let mut first: i32 = 0;
let mut tmp: JSValue = core::mem::zeroed();
let mut tmp_1: JSValue = core::mem::zeroed();
let mut tmp_2: JSValue = core::mem::zeroed();
let mut tmp_3: JSValue = core::mem::zeroed();
let mut tmp_4: JSValue = core::mem::zeroed();
let mut tmp_5: JSValue = core::mem::zeroed();
let mut tmp_6: JSValue = core::mem::zeroed();
let mut tmp_7: JSValue = core::mem::zeroed();
let mut tmp1: JSValue = core::mem::zeroed();
let mut tmp2: JSValue = core::mem::zeroed();
let mut bfunc: JSValue = core::mem::zeroed();
let mut magic: i32 = 0;
let mut v_super: JSValue = core::mem::zeroed();
let mut ret: JSValue = core::mem::zeroed();
let mut ret_1: i32 = 0;
let mut atom: JSAtom = 0;
let mut v_type: i32 = 0;
let mut obj: JSValue = core::mem::zeroed();
let mut scope_idx: i32 = 0;
let mut scope_idx_1: i32 = 0;
let mut len: u32 = 0;
let mut tab: *mut JSValue = core::ptr::null_mut();
let mut obj_1: JSValue = core::mem::zeroed();
let mut proto: JSValue = core::mem::zeroed();
let mut val_1: JSValue = core::mem::zeroed();
let mut idx: i32 = 0;
let mut val_2: JSValue = core::mem::zeroed();
let mut cv: *mut JSClosureVar = core::ptr::null_mut();
let mut idx_1: i32 = 0;
let mut ret_2: i32 = 0;
let mut var_ref: *mut JSVarRef = core::ptr::null_mut();
let mut cv_1: *mut JSClosureVar = core::ptr::null_mut();
let mut idx_2: i32 = 0;
let mut idx_3: i32 = 0;
let mut idx_4: i32 = 0;
let mut idx_5: i32 = 0;
let mut idx_6: i32 = 0;
let mut idx_7: i32 = 0;
let mut idx_8: i32 = 0;
let mut val_3: JSValue = core::mem::zeroed();
let mut idx_9: i32 = 0;
let mut idx_10: i32 = 0;
let mut idx_11: i32 = 0;
let mut val_4: JSValue = core::mem::zeroed();
let mut idx_12: i32 = 0;
let mut idx_13: i32 = 0;
let mut idx_14: i32 = 0;
let mut idx_15: i32 = 0;
let mut idx_16: i32 = 0;
let mut idx_17: i32 = 0;
let mut idx_18: i32 = 0;
let mut idx_19: i32 = 0;
let mut idx_20: i32 = 0;
let mut var_ref_1: *mut JSVarRef = core::ptr::null_mut();
let mut pr: *mut JSProperty = core::ptr::null_mut();
let mut atom_1: JSAtom = 0;
let mut idx_21: i32 = 0;
let mut atom_2: JSAtom = 0;
let mut res: i32 = 0;
let mut op1: JSValue = core::mem::zeroed();
let mut res_1: i32 = 0;
let mut op1_1: JSValue = core::mem::zeroed();
let mut res_2: i32 = 0;
let mut op1_2: JSValue = core::mem::zeroed();
let mut res_3: i32 = 0;
let mut op1_3: JSValue = core::mem::zeroed();
let mut diff: i32 = 0;
let mut diff_1: i32 = 0;
let mut op1_4: JSValue = core::mem::zeroed();
let mut pos: u32 = 0;
let mut offset: i32 = 0;
let mut ret_val_1: JSValue = core::mem::zeroed();
let mut ret_3: JSValue = core::mem::zeroed();
let mut method: JSValue = core::mem::zeroed();
let mut ret_4: JSValue = core::mem::zeroed();
let mut ret_flag: i32 = 0;
let mut flags_1: i32 = 0;
let mut res_4: i32 = 0;
let mut op1_5: JSValue = core::mem::zeroed();
let mut val_5: JSValue = core::mem::zeroed();
let mut obj_2: JSValue = core::mem::zeroed();
let mut atom_3: JSAtom = 0;
let mut p_1: *mut JSObject = core::ptr::null_mut();
let mut pr_1: *mut JSProperty = core::ptr::null_mut();
let mut prs: *mut JSShapeProperty = core::ptr::null_mut();
let mut val_6: JSValue = core::mem::zeroed();
let mut obj_3: JSValue = core::mem::zeroed();
let mut atom_4: JSAtom = 0;
let mut p_2: *mut JSObject = core::ptr::null_mut();
let mut pr_2: *mut JSProperty = core::ptr::null_mut();
let mut prs_1: *mut JSShapeProperty = core::ptr::null_mut();
let mut val_7: JSValue = core::mem::zeroed();
let mut obj_4: JSValue = core::mem::zeroed();
let mut atom_5: JSAtom = 0;
let mut p_3: *mut JSObject = core::ptr::null_mut();
let mut pr_3: *mut JSProperty = core::ptr::null_mut();
let mut prs_2: *mut JSShapeProperty = core::ptr::null_mut();
let mut ret_5: i32 = 0;
let mut obj_5: JSValue = core::mem::zeroed();
let mut atom_6: JSAtom = 0;
let mut p_4: *mut JSObject = core::ptr::null_mut();
let mut pr_4: *mut JSProperty = core::ptr::null_mut();
let mut prs_3: *mut JSShapeProperty = core::ptr::null_mut();
let mut atom_7: JSAtom = 0;
let mut val_8: JSValue = core::mem::zeroed();
let mut val_9: JSValue = core::mem::zeroed();
let mut ret_6: i32 = 0;
let mut ret_7: i32 = 0;
let mut ret_8: i32 = 0;
let mut atom_8: JSAtom = 0;
let mut ret_9: i32 = 0;
let mut atom_9: JSAtom = 0;
let mut ret_10: i32 = 0;
let mut proto_1: JSValue = core::mem::zeroed();
let mut getter: JSValue = core::mem::zeroed();
let mut setter: JSValue = core::mem::zeroed();
let mut value: JSValue = core::mem::zeroed();
let mut obj_6: JSValue = core::mem::zeroed();
let mut atom_10: JSAtom = 0;
let mut flags_2: i32 = 0;
let mut ret_11: i32 = 0;
let mut op_flags: i32 = 0;
let mut is_computed: i32 = 0;
let mut class_flags: i32 = 0;
let mut atom_11: JSAtom = 0;
let mut val_10: JSValue = core::mem::zeroed();
let mut obj_7: JSValue = core::mem::zeroed();
let mut prop: JSValue = core::mem::zeroed();
let mut p_5: *mut JSObject = core::ptr::null_mut();
let mut idx_22: u32 = 0;
let mut val_11: JSValue = core::mem::zeroed();
let mut obj_8: JSValue = core::mem::zeroed();
let mut prop_1: JSValue = core::mem::zeroed();
let mut p_6: *mut JSObject = core::ptr::null_mut();
let mut idx_23: u32 = 0;
let mut val_12: JSValue = core::mem::zeroed();
let mut p_7: *mut JSObject = core::ptr::null_mut();
let mut idx_24: u32 = 0;
let mut val_13: JSValue = core::mem::zeroed();
let mut atom_12: JSAtom = 0;
let mut ret_12: i32 = 0;
let mut val_14: JSValue = core::mem::zeroed();
let mut atom_13: JSAtom = 0;
let mut ret_13: i32 = 0;
let mut p_8: *mut JSObject = core::ptr::null_mut();
let mut idx_25: u32 = 0;
let mut new_len: u32 = 0;
let mut array_len: u32 = 0;
let mut ret_14: i32 = 0;
let mut atom_14: JSAtom = 0;
let mut ret_15: i32 = 0;
let mut atom_15: JSAtom = 0;
let mut ret_16: i32 = 0;
let mut mask: i32 = 0;
let mut op1_6: JSValue = core::mem::zeroed();
let mut op2: JSValue = core::mem::zeroed();
let mut r: i64 = 0;
let mut d1: f64 = 0.0;
let mut d2: f64 = 0.0;
let mut op2_1: JSValue = core::mem::zeroed();
let mut pv: *mut JSValue = core::ptr::null_mut();
let mut idx_26: i32 = 0;
let mut r_1: i64 = 0;
let mut ops: [JSValue; 2] = core::mem::zeroed();
let mut op1_7: JSValue = core::mem::zeroed();
let mut op2_2: JSValue = core::mem::zeroed();
let mut r_2: i64 = 0;
let mut d1_1: f64 = 0.0;
let mut d2_1: f64 = 0.0;
let mut op1_8: JSValue = core::mem::zeroed();
let mut op2_3: JSValue = core::mem::zeroed();
let mut d: f64 = 0.0;
let mut v1: i32 = 0;
let mut v2: i32 = 0;
let mut r_3: i64 = 0;
let mut d1_2: f64 = 0.0;
let mut d2_2: f64 = 0.0;
let mut op1_9: JSValue = core::mem::zeroed();
let mut op2_4: JSValue = core::mem::zeroed();
let mut v1_1: i32 = 0;
let mut v2_1: i32 = 0;
let mut op1_10: JSValue = core::mem::zeroed();
let mut op2_5: JSValue = core::mem::zeroed();
let mut v1_2: i32 = 0;
let mut v2_2: i32 = 0;
let mut r_4: i32 = 0;
let mut op1_11: JSValue = core::mem::zeroed();
let mut tag_1: u32 = 0;
let mut op1_12: JSValue = core::mem::zeroed();
let mut tag_2: u32 = 0;
let mut val_15: i32 = 0;
let mut d_1: f64 = 0.0;
let mut op1_13: JSValue = core::mem::zeroed();
let mut val_16: i32 = 0;
let mut op1_14: JSValue = core::mem::zeroed();
let mut val_17: i32 = 0;
let mut op1_15: JSValue = core::mem::zeroed();
let mut val_18: i32 = 0;
let mut op1_16: JSValue = core::mem::zeroed();
let mut val_19: i32 = 0;
let mut op1_17: JSValue = core::mem::zeroed();
let mut val_20: i32 = 0;
let mut idx_27: i32 = 0;
let mut op1_18: JSValue = core::mem::zeroed();
let mut val_21: i32 = 0;
let mut idx_28: i32 = 0;
let mut op1_19: JSValue = core::mem::zeroed();
let mut op1_20: JSValue = core::mem::zeroed();
let mut op2_6: JSValue = core::mem::zeroed();
let mut v1_3: u32 = 0;
let mut v2_3: u32 = 0;
let mut op1_21: JSValue = core::mem::zeroed();
let mut op2_7: JSValue = core::mem::zeroed();
let mut v2_4: u32 = 0;
let mut op1_22: JSValue = core::mem::zeroed();
let mut op2_8: JSValue = core::mem::zeroed();
let mut v2_5: u32 = 0;
let mut op1_23: JSValue = core::mem::zeroed();
let mut op2_9: JSValue = core::mem::zeroed();
let mut op1_24: JSValue = core::mem::zeroed();
let mut op2_10: JSValue = core::mem::zeroed();
let mut op1_25: JSValue = core::mem::zeroed();
let mut op2_11: JSValue = core::mem::zeroed();
let mut op1_26: JSValue = core::mem::zeroed();
let mut op2_12: JSValue = core::mem::zeroed();
let mut op1_27: JSValue = core::mem::zeroed();
let mut op2_13: JSValue = core::mem::zeroed();
let mut op1_28: JSValue = core::mem::zeroed();
let mut op2_14: JSValue = core::mem::zeroed();
let mut op1_29: JSValue = core::mem::zeroed();
let mut op2_15: JSValue = core::mem::zeroed();
let mut op1_30: JSValue = core::mem::zeroed();
let mut op2_16: JSValue = core::mem::zeroed();
let mut op1_31: JSValue = core::mem::zeroed();
let mut op2_17: JSValue = core::mem::zeroed();
let mut op1_32: JSValue = core::mem::zeroed();
let mut op2_18: JSValue = core::mem::zeroed();
let mut op1_33: JSValue = core::mem::zeroed();
let mut op2_19: JSValue = core::mem::zeroed();
let mut op1_34: JSValue = core::mem::zeroed();
let mut atom_16: JSAtom = 0;
let mut atom_17: JSAtom = 0;
let mut ret_17: i32 = 0;
let mut atom_18: JSAtom = 0;
let mut diff_2: i32 = 0;
let mut obj_9: JSValue = core::mem::zeroed();
let mut val_22: JSValue = core::mem::zeroed();
let mut ret_18: i32 = 0;
let mut is_with: i32 = 0;
let mut val_23: JSValue = core::mem::zeroed();
let mut pos_1: i32 = 0;
let mut vm_block: usize = 1855;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 20592
2 => {
let _ = { let assigned = (*(sf)).prev_frame; (*(rt)).current_stack_frame = assigned; assigned };
// C line 20593
return ret_val;
}
// C line ? labels: done_generator
4 => {
let _ = { let assigned = pc; (*(sf)).cur_pc = assigned; assigned };
// C line 20580
let _ = { let assigned = sp; (*(sf)).cur_sp = assigned; assigned };
vm_block = 2; continue;
}
// C line 20588
5 => {
vm_block = if ((((pval) < (sp)) as i32)) != 0 { 7 } else { 2 }; continue;
}
// C line 20589
7 => {
let _ = JS_FreeValue(ctx, *(pval));
// C line ?
let _ = { let old = pval; pval = (pval).offset(1); old };
vm_block = 5; continue;
}
// C line 20588
8 => {
js_host_free_frame_diagnostics(rt, sf);
let _ = { let assigned = local_buf; pval = assigned; assigned };
vm_block = 5; continue;
}
// C line 20585
9 => {
let _ = close_var_refs(rt, b, sf);
vm_block = 8; continue;
}
// C line 20583 labels: done
10 => {
vm_block = if ((((!(((!((((((((*(b)).var_ref_count) as i32)) != ((0 as i32))) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 9 } else { 8 }; continue;
}
// C line 20577
11 => {
vm_block = if (((((((*(b)).func_kind()) as i32)) != ((JS_FUNC_NORMAL as i32))) as i32)) != 0 { 4 } else { 10 }; continue;
}
// C line 20573
12 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) }; ret_val = assigned; assigned };
vm_block = 11; continue;
}
// C line 20554
13 => {
vm_block = if ((((sp) > (stack_buf)) as i32)) != 0 { 25 } else { 12 }; continue;
}
// C line 20561
16 => {
let _ = JS_FreeValue(ctx, *(sp).offset((((1 as i32)).wrapping_neg()) as isize));
// C line 20562
let _ = { let old = sp; sp = (sp).offset(-1); old };
// C line 20563
let _ = JS_IteratorClose(ctx, *(sp).offset((((1 as i32)).wrapping_neg()) as isize), (1 as i32));
vm_block = 13; continue;
}
// C line 20565
20 => {
js_host_catch_exception(rt, sf, sp);
let _ = { let assigned = (*(rt)).current_exception; *({ let old = sp; sp = (sp).offset(1); old }) = assigned; assigned };
// C line 20566
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNINITIALIZED as i32)) as i64) }; (*(rt)).current_exception = assigned; assigned };
// C line 20567
let _ = { let assigned = ((*(b)).byte_code_buf).offset(((pos_1) as isize)); pc = assigned; assigned };
// C line 20568
vm_block = 30; continue;
}
// C line 20559
21 => {
vm_block = if ((((pos_1) == ((0 as i32))) as i32)) != 0 { 16 } else { 20 }; continue;
}
// C line 20558
22 => {
pos_1 = ((((val_23).u).uint64) as i32);
vm_block = 21; continue;
}
// C line 20557
23 => {
vm_block = if (((((((val_23).tag) as i32)) == ((JS_TAG_CATCH_OFFSET as i32))) as i32)) != 0 { 22 } else { 13 }; continue;
}
// C line 20555
25 => {
val_23 = *({ sp = (sp).offset(-1); sp });
// C line 20556
let _ = JS_FreeValue(ctx, val_23);
vm_block = 23; continue;
}
// C line 20553
26 => {
vm_block = if ((!(((*(rt)).current_exception_is_uncatchable) != 0) as i32)) != 0 { 13 } else { 12 }; continue;
}
// C line 20550
28 => {
let _ = { let assigned = pc; (*(sf)).cur_pc = assigned; assigned };
// C line 20551
let _ = build_backtrace(ctx, (*(rt)).current_exception, ((core::ptr::null_mut::<c_void>()) as *const c_char), (0 as i32), (0 as i32), (0 as i32));
vm_block = 26; continue;
}
// C line 20546 labels: exception
29 => {
js_host_vm_exception(ctx, b, pc);
vm_block = if (is_backtrace_needed(ctx, (*(rt)).current_exception)) != 0 { 28 } else { 26 }; continue;
}
// C line 17900 labels: restart
30 => {
vm_block = 1783; continue;
}
// C line 20540
32 => {
let _ = JS_ThrowInternalError(ctx, format_args!("invalid opcode: pc={} opcode=0x{:02x}", (((((pc).offset_from((*(b)).byte_code_buf) as i64)).wrapping_sub((((1 as i32)) as i64))) as i32), opcode));
// C line 20542
vm_block = 29; continue;
}
// C line ? labels: free_and_set_false
35 => {
let _ = JS_FreeValue(ctx, *(sp).offset((((1 as i32)).wrapping_neg()) as isize));
// C line 20536
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_BOOL as i32)) as i64) }; *(sp).offset((((1 as i32)).wrapping_neg()) as isize) = assigned; assigned };
// C line 20537
vm_block = 30; continue;
}
// C line ? labels: set_true
37 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((1 as i32)) as u32)) as u64) }, tag: (((JS_TAG_BOOL as i32)) as i64) }; *(sp).offset((((1 as i32)).wrapping_neg()) as isize) = assigned; assigned };
// C line 20533
vm_block = 30; continue;
}
// C line ? labels: free_and_set_true
38 => {
let _ = JS_FreeValue(ctx, *(sp).offset((((1 as i32)).wrapping_neg()) as isize));
vm_block = 37; continue;
}
// C line 20524
39 => {
vm_block = 38; continue;
}
// C line 20526
40 => {
vm_block = 35; continue;
}
// C line 20523
41 => {
vm_block = if ((((js_operator_typeof(ctx, *(sp).offset((((1 as i32)).wrapping_neg()) as isize))) == ((JS_ATOM_function as i32))) as i32)) != 0 { 39 } else { 40 }; continue;
}
// C line 20518
42 => {
vm_block = 38; continue;
}
// C line 20520
43 => {
vm_block = 35; continue;
}
// C line 20517
44 => {
vm_block = if ((((js_operator_typeof(ctx, *(sp).offset((((1 as i32)).wrapping_neg()) as isize))) == ((JS_ATOM_undefined as i32))) as i32)) != 0 { 42 } else { 43 }; continue;
}
// C line 20510
45 => {
vm_block = 37; continue;
}
// C line 20512
46 => {
vm_block = 35; continue;
}
// C line 20509
47 => {
vm_block = if (((((((*(sp).offset((((1 as i32)).wrapping_neg()) as isize)).tag) as i32)) == ((JS_TAG_NULL as i32))) as i32)) != 0 { 45 } else { 46 }; continue;
}
// C line 20504
48 => {
vm_block = 37; continue;
}
// C line 20506
49 => {
vm_block = 35; continue;
}
// C line 20503
50 => {
vm_block = if (((((((*(sp).offset((((1 as i32)).wrapping_neg()) as isize)).tag) as i32)) == ((JS_TAG_UNDEFINED as i32))) as i32)) != 0 { 48 } else { 49 }; continue;
}
// C line 20497
51 => {
vm_block = 37; continue;
}
// C line 20499
52 => {
vm_block = 35; continue;
}
// C line 20495
53 => {
vm_block = if (((((((((((*(sp).offset((((1 as i32)).wrapping_neg()) as isize)).tag) as i32)) == ((JS_TAG_UNDEFINED as i32))) as i32)) != 0) || ((((((((*(sp).offset((((1 as i32)).wrapping_neg()) as isize)).tag) as i32)) == ((JS_TAG_NULL as i32))) as i32)) != 0)) as i32)) != 0 { 51 } else { 52 }; continue;
}
// C line 20493
54 => {
vm_block = 30; continue;
}
// C line 20489
56 => {
let _ = { let assigned = JS_NewInt32(ctx, (3 as i32)); ret_val = assigned; assigned };
// C line 20490
vm_block = 4; continue;
}
// C line 20486
58 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; ret_val = assigned; assigned };
// C line 20487
vm_block = 4; continue;
}
// C line 20483
60 => {
let _ = { let assigned = JS_NewInt32(ctx, (2 as i32)); ret_val = assigned; assigned };
// C line 20484
vm_block = 4; continue;
}
// C line 20479
62 => {
let _ = { let assigned = JS_NewInt32(ctx, (1 as i32)); ret_val = assigned; assigned };
// C line 20480
vm_block = 4; continue;
}
// C line 20476
64 => {
let _ = { let assigned = JS_NewInt32(ctx, (0 as i32)); ret_val = assigned; assigned };
// C line 20477
vm_block = 4; continue;
}
// C line 20473
65 => {
vm_block = 30; continue;
}
// C line 20465
66 => {
let _ = { pc = (pc).offset((((diff_2).wrapping_sub((5 as i32))) as isize)); pc };
vm_block = 65; continue;
}
// C line 20462
68 => {
let _ = { let assigned = val_22; *({ let old = sp; sp = (sp).offset(1); old }) = assigned; assigned };
// C line 20463
vm_block = 66; continue;
}
// C line 20456
69 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; val_22 = assigned; assigned };
vm_block = 68; continue;
}
// C line 20460
70 => {
vm_block = 29; continue;
}
// C line 20459
71 => {
vm_block = if ((((!(((!((JS_IsException(val_22)) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 70 } else { 68 }; continue;
}
// C line 20458
72 => {
let _ = { let assigned = JS_GetProperty(ctx, obj_9, atom_18); val_22 = assigned; assigned };
vm_block = 71; continue;
}
// C line 20455
73 => {
vm_block = if ((!((ret_18) != 0) as i32)) != 0 { 69 } else { 72 }; continue;
}
// C line 20454
74 => {
vm_block = 29; continue;
}
// C line 20453
75 => {
vm_block = if ((((!(((!(((((ret_18) < ((0 as i32))) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 74 } else { 73 }; continue;
}
// C line 20452
76 => {
let _ = { let assigned = JS_HasProperty(ctx, obj_9, atom_18); ret_18 = assigned; assigned };
vm_block = 75; continue;
}
// C line 20447
78 => {
let _ = { let assigned = JS_AtomToValue(ctx, atom_18); *({ let old = sp; sp = (sp).offset(1); old }) = assigned; assigned };
// C line 20448
vm_block = 66; continue;
}
// C line 20442
81 => {
let _ = JS_FreeValue(ctx, *(sp).offset((((1 as i32)).wrapping_neg()) as isize));
// C line 20443
let _ = { let assigned = JS_NewBool(ctx, ret_18); *(sp).offset((((1 as i32)).wrapping_neg()) as isize) = assigned; assigned };
// C line 20444
vm_block = 66; continue;
}
// C line 20441
82 => {
vm_block = 29; continue;
}
// C line 20440
83 => {
vm_block = if ((((!(((!(((((ret_18) < ((0 as i32))) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 82 } else { 81 }; continue;
}
// C line 20439
84 => {
let _ = { let assigned = JS_DeleteProperty(ctx, obj_9, atom_18, (0 as i32)); ret_18 = assigned; assigned };
vm_block = 83; continue;
}
// C line 20437
85 => {
vm_block = 66; continue;
}
// C line 20436
86 => {
vm_block = 29; continue;
}
// C line 20435
87 => {
vm_block = if ((((!(((!(((((ret_18) < ((0 as i32))) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 86 } else { 85 }; continue;
}
// C line 20431
90 => {
let _ = { let assigned = JS_SetPropertyInternal(ctx, obj_9, atom_18, *(sp).offset((((2 as i32)).wrapping_neg()) as isize), obj_9, ((1 as i32)).wrapping_shl(((15 as i32)) as u32)); ret_18 = assigned; assigned };
// C line 20433
let _ = JS_FreeValue(ctx, *(sp).offset((((1 as i32)).wrapping_neg()) as isize));
// C line 20434
let _ = { sp = (sp).offset(-(((2 as i32)) as isize)); sp };
vm_block = 87; continue;
}
// C line 20427
92 => {
let _ = JS_ThrowReferenceErrorNotDefined(ctx, atom_18);
// C line 20428
vm_block = 29; continue;
}
// C line 20426
93 => {
vm_block = if (is_strict_mode(ctx)) != 0 { 92 } else { 90 }; continue;
}
// C line 20425
94 => {
vm_block = 29; continue;
}
// C line 20424
95 => {
vm_block = if ((((ret_18) < ((0 as i32))) as i32)) != 0 { 94 } else { 93 }; continue;
}
// C line 20423
96 => {
vm_block = if ((((!(((!(((((ret_18) <= ((0 as i32))) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 95 } else { 90 }; continue;
}
// C line 20422
97 => {
let _ = { let assigned = JS_HasProperty(ctx, obj_9, atom_18); ret_18 = assigned; assigned };
vm_block = 96; continue;
}
// C line 20418
99 => {
let _ = set_value(ctx, core::ptr::addr_of_mut!(*(sp).offset((((1 as i32)).wrapping_neg()) as isize)), val_22);
// C line 20419
vm_block = 66; continue;
}
// C line 20412
100 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; val_22 = assigned; assigned };
vm_block = 99; continue;
}
// C line 20409
102 => {
let _ = JS_ThrowReferenceErrorNotDefined(ctx, atom_18);
// C line 20410
vm_block = 29; continue;
}
// C line 20408
103 => {
vm_block = if (is_strict_mode(ctx)) != 0 { 102 } else { 100 }; continue;
}
// C line 20407
104 => {
vm_block = 29; continue;
}
// C line 20406
105 => {
vm_block = if ((((ret_18) < ((0 as i32))) as i32)) != 0 { 104 } else { 103 }; continue;
}
// C line 20416
106 => {
vm_block = 29; continue;
}
// C line 20415
107 => {
vm_block = if ((((!(((!((JS_IsException(val_22)) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 106 } else { 99 }; continue;
}
// C line 20414
108 => {
let _ = { let assigned = JS_GetProperty(ctx, obj_9, atom_18); val_22 = assigned; assigned };
vm_block = 107; continue;
}
// C line 20405
109 => {
vm_block = if ((((!(((!(((((ret_18) <= ((0 as i32))) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 105 } else { 108 }; continue;
}
// C line 20404
110 => {
let _ = { let assigned = JS_HasProperty(ctx, obj_9, atom_18); ret_18 = assigned; assigned };
vm_block = 109; continue;
}
// C line 20401
111 => {
vm_block = match opcode { x if x == (OP_with_get_ref as i32) => 76, x if x == (OP_with_make_ref as i32) => 78, x if x == (OP_with_delete_var as i32) => 84, x if x == (OP_with_put_var as i32) => 97, x if x == (OP_with_get_var as i32) => 110, _ => 66, }; continue;
}
// C line 20399
112 => {
vm_block = 119; continue;
}
// C line 20398
113 => {
vm_block = if (ret_18) != 0 { 112 } else { 111 }; continue;
}
// C line 20397
114 => {
vm_block = 29; continue;
}
// C line 20396
115 => {
vm_block = if ((((!(((!(((((ret_18) < ((0 as i32))) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 114 } else { 113 }; continue;
}
// C line 20395
116 => {
let _ = { let assigned = js_has_unscopable(ctx, obj_9, atom_18); ret_18 = assigned; assigned };
vm_block = 115; continue;
}
// C line 20394
117 => {
vm_block = if (is_with) != 0 { 116 } else { 111 }; continue;
}
// C line ? labels: no_with
119 => {
let _ = JS_FreeValue(ctx, *(sp).offset((((1 as i32)).wrapping_neg()) as isize));
// C line 20470
let _ = { let old = sp; sp = (sp).offset(-1); old };
vm_block = 65; continue;
}
// C line 20393
120 => {
vm_block = if (ret_18) != 0 { 117 } else { 119 }; continue;
}
// C line 20392
121 => {
vm_block = 29; continue;
}
// C line 20391
122 => {
vm_block = if ((((!(((!(((((ret_18) < ((0 as i32))) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 121 } else { 120 }; continue;
}
// C line 20383
129 => {
let _ = { let assigned = crate::cutils_header::get_u32(pc); atom_18 = assigned; assigned };
// C line 20384
let _ = { let assigned = ((crate::cutils_header::get_u32((pc).offset((((4 as i32)) as isize)))) as i32); diff_2 = assigned; assigned };
// C line 20385
let _ = { let assigned = ((*(pc).offset(((8 as i32)) as isize)) as i32); is_with = assigned; assigned };
// C line 20386
let _ = { pc = (pc).offset((((9 as i32)) as isize)); pc };
// C line 20387
let _ = { let assigned = pc; (*(sf)).cur_pc = assigned; assigned };
// C line 20389
let _ = { let assigned = *(sp).offset((((1 as i32)).wrapping_neg()) as isize); obj_9 = assigned; assigned };
// C line 20390
let _ = { let assigned = JS_HasProperty(ctx, obj_9, atom_18); ret_18 = assigned; assigned };
vm_block = 122; continue;
}
// C line 20360
130 => {
vm_block = 30; continue;
}
// C line 20356
133 => {
let _ = JS_FreeValue(ctx, *(sp).offset((((1 as i32)).wrapping_neg()) as isize));
// C line 20357
let _ = { let assigned = ret_val; *(sp).offset((((1 as i32)).wrapping_neg()) as isize) = assigned; assigned };
// C line 20358
vm_block = 130; continue;
}
// C line 20355
134 => {
vm_block = 29; continue;
}
// C line 20354
135 => {
vm_block = if (JS_IsException(ret_val)) != 0 { 134 } else { 133 }; continue;
}
// C line ?
137 => {
let _ = { let assigned = pc; (*(sf)).cur_pc = assigned; assigned };
// C line 20353
let _ = { let assigned = JS_ToPropertyKey(ctx, *(sp).offset((((1 as i32)).wrapping_neg()) as isize)); ret_val = assigned; assigned };
vm_block = 135; continue;
}
// C line 20350
138 => {
vm_block = 130; continue;
}
// C line 20346
139 => {
vm_block = match (((*(sp).offset((((1 as i32)).wrapping_neg()) as isize)).tag) as i32) { x if x == (JS_TAG_SYMBOL as i32) => 138, x if x == (JS_TAG_STRING as i32) => 138, x if x == (JS_TAG_INT as i32) => 138, _ => 137, }; continue;
}
// C line 20343
140 => {
vm_block = 30; continue;
}
// C line 20340
142 => {
let _ = JS_FreeValue(ctx, *(sp).offset((((1 as i32)).wrapping_neg()) as isize));
// C line 20341
let _ = { let assigned = ret_val; *(sp).offset((((1 as i32)).wrapping_neg()) as isize) = assigned; assigned };
vm_block = 140; continue;
}
// C line 20339
143 => {
vm_block = 29; continue;
}
// C line 20338
144 => {
vm_block = if (JS_IsException(ret_val)) != 0 { 143 } else { 142 }; continue;
}
// C line 20336
146 => {
let _ = { let assigned = pc; (*(sf)).cur_pc = assigned; assigned };
// C line 20337
let _ = { let assigned = JS_ToObject(ctx, *(sp).offset((((1 as i32)).wrapping_neg()) as isize)); ret_val = assigned; assigned };
vm_block = 144; continue;
}
// C line 20335
147 => {
vm_block = if (((((((*(sp).offset((((1 as i32)).wrapping_neg()) as isize)).tag) as i32)) != ((JS_TAG_OBJECT as i32))) as i32)) != 0 { 146 } else { 140 }; continue;
}
// C line 20330
149 => {
let _ = { let assigned = JS_NewBool(ctx, ret_17); *({ let old = sp; sp = (sp).offset(1); old }) = assigned; assigned };
// C line 20332
vm_block = 30; continue;
}
// C line 20329
150 => {
vm_block = 29; continue;
}
// C line 20328
151 => {
vm_block = if ((((!(((!(((((ret_17) < ((0 as i32))) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 150 } else { 149 }; continue;
}
// C line 20323
155 => {
let _ = { let assigned = crate::cutils_header::get_u32(pc); atom_17 = assigned; assigned };
// C line 20324
let _ = { pc = (pc).offset((((4 as i32)) as isize)); pc };
// C line 20325
let _ = { let assigned = pc; (*(sf)).cur_pc = assigned; assigned };
// C line 20327
let _ = { let assigned = JS_DeleteGlobalVar(ctx, atom_17); ret_17 = assigned; assigned };
vm_block = 151; continue;
}
// C line 20316
157 => {
let _ = { let old = sp; sp = (sp).offset(-1); old };
// C line 20317
vm_block = 30; continue;
}
// C line 20315
158 => {
vm_block = 29; continue;
}
// C line 20314
159 => {
vm_block = if (js_operator_delete(ctx, sp)) != 0 { 158 } else { 157 }; continue;
}
// C line 20313
160 => {
let _ = { let assigned = pc; (*(sf)).cur_pc = assigned; assigned };
vm_block = 159; continue;
}
// C line 20306
165 => {
let _ = { let assigned = *(sp).offset((((1 as i32)).wrapping_neg()) as isize); op1_34 = assigned; assigned };
// C line 20307
let _ = { let assigned = ((js_operator_typeof(ctx, op1_34)) as JSAtom); atom_16 = assigned; assigned };
// C line 20308
let _ = JS_FreeValue(ctx, op1_34);
// C line 20309
let _ = { let assigned = JS_AtomToString(ctx, atom_16); *(sp).offset((((1 as i32)).wrapping_neg()) as isize) = assigned; assigned };
// C line 20311
vm_block = 30; continue;
}
// C line 20299
167 => {
let _ = { let old = sp; sp = (sp).offset(-1); old };
// C line 20300
vm_block = 30; continue;
}
// C line 20298
168 => {
vm_block = 29; continue;
}
// C line 20297
169 => {
vm_block = if (js_operator_instanceof(ctx, sp)) != 0 { 168 } else { 167 }; continue;
}
// C line 20296
170 => {
let _ = { let assigned = pc; (*(sf)).cur_pc = assigned; assigned };
vm_block = 169; continue;
}
// C line 20293
172 => {
let _ = { let old = sp; sp = (sp).offset(-1); old };
// C line 20294
vm_block = 30; continue;
}
// C line 20292
173 => {
vm_block = 29; continue;
}
// C line 20291
174 => {
vm_block = if (js_operator_private_in(ctx, sp)) != 0 { 173 } else { 172 }; continue;
}
// C line 20290
175 => {
let _ = { let assigned = pc; (*(sf)).cur_pc = assigned; assigned };
vm_block = 174; continue;
}
// C line 20287
177 => {
let _ = { let old = sp; sp = (sp).offset(-1); old };
// C line 20288
vm_block = 30; continue;
}
// C line 20286
178 => {
vm_block = 29; continue;
}
// C line 20285
179 => {
vm_block = if (js_operator_in(ctx, sp)) != 0 { 178 } else { 177 }; continue;
}
// C line 20284
180 => {
let _ = { let assigned = pc; (*(sf)).cur_pc = assigned; assigned };
vm_block = 179; continue;
}
// C line 20281
181 => {
vm_block = 30; continue;
}
// C line 20281
183 => {
let _ = { let assigned = JS_NewBool(ctx, (((((((op1_33).u).uint64) as i32)) != (((((op2_19).u).uint64) as i32))) as i32)); *(sp).offset((((2 as i32)).wrapping_neg()) as isize) = assigned; assigned };
// C line 20281
let _ = { let old = sp; sp = (sp).offset(-1); old };
vm_block = 181; continue;
}
// C line 20281
184 => {
let _ = { let old = sp; sp = (sp).offset(-1); old };
vm_block = 181; continue;
}
// C line 20281
185 => {
vm_block = 29; continue;
}
// C line 20281
186 => {
vm_block = if (js_strict_eq_slow(ctx, sp, (1 as i32))) != 0 { 185 } else { 184 }; continue;
}
// C line 20281
187 => {
let _ = { let assigned = pc; (*(sf)).cur_pc = assigned; assigned };
vm_block = 186; continue;
}
// C line 20281
188 => {
vm_block = if ((((!(((!((((((((((op1_33).tag) as i32)) | ((((op2_19).tag) as i32)))) == ((0 as i32))) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 183 } else { 187 }; continue;
}
// C line 20281
190 => {
let _ = { let assigned = *(sp).offset((((2 as i32)).wrapping_neg()) as isize); op1_33 = assigned; assigned };
// C line 20281
let _ = { let assigned = *(sp).offset((((1 as i32)).wrapping_neg()) as isize); op2_19 = assigned; assigned };
vm_block = 188; continue;
}
// C line 20280
191 => {
vm_block = 30; continue;
}
// C line 20280
193 => {
let _ = { let assigned = JS_NewBool(ctx, (((((((op1_32).u).uint64) as i32)) == (((((op2_18).u).uint64) as i32))) as i32)); *(sp).offset((((2 as i32)).wrapping_neg()) as isize) = assigned; assigned };
// C line 20280
let _ = { let old = sp; sp = (sp).offset(-1); old };
vm_block = 191; continue;
}
// C line 20280
194 => {
let _ = { let old = sp; sp = (sp).offset(-1); old };
vm_block = 191; continue;
}
// C line 20280
195 => {
vm_block = 29; continue;
}
// C line 20280
196 => {
vm_block = if (js_strict_eq_slow(ctx, sp, (0 as i32))) != 0 { 195 } else { 194 }; continue;
}
// C line 20280
197 => {
let _ = { let assigned = pc; (*(sf)).cur_pc = assigned; assigned };
vm_block = 196; continue;
}
// C line 20280
198 => {
vm_block = if ((((!(((!((((((((((op1_32).tag) as i32)) | ((((op2_18).tag) as i32)))) == ((0 as i32))) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 193 } else { 197 }; continue;
}
// C line 20280
200 => {
let _ = { let assigned = *(sp).offset((((2 as i32)).wrapping_neg()) as isize); op1_32 = assigned; assigned };
// C line 20280
let _ = { let assigned = *(sp).offset((((1 as i32)).wrapping_neg()) as isize); op2_18 = assigned; assigned };
vm_block = 198; continue;
}
// C line 20279
201 => {
vm_block = 30; continue;
}
// C line 20279
203 => {
let _ = { let assigned = JS_NewBool(ctx, (((((((op1_31).u).uint64) as i32)) != (((((op2_17).u).uint64) as i32))) as i32)); *(sp).offset((((2 as i32)).wrapping_neg()) as isize) = assigned; assigned };
// C line 20279
let _ = { let old = sp; sp = (sp).offset(-1); old };
vm_block = 201; continue;
}
// C line 20279
204 => {
let _ = { let old = sp; sp = (sp).offset(-1); old };
vm_block = 201; continue;
}
// C line 20279
205 => {
vm_block = 29; continue;
}
// C line 20279
206 => {
vm_block = if (js_eq_slow(ctx, sp, (1 as i32))) != 0 { 205 } else { 204 }; continue;
}
// C line 20279
207 => {
let _ = { let assigned = pc; (*(sf)).cur_pc = assigned; assigned };
vm_block = 206; continue;
}
// C line 20279
208 => {
vm_block = if ((((!(((!((((((((((op1_31).tag) as i32)) | ((((op2_17).tag) as i32)))) == ((0 as i32))) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 203 } else { 207 }; continue;
}
// C line 20279
210 => {
let _ = { let assigned = *(sp).offset((((2 as i32)).wrapping_neg()) as isize); op1_31 = assigned; assigned };
// C line 20279
let _ = { let assigned = *(sp).offset((((1 as i32)).wrapping_neg()) as isize); op2_17 = assigned; assigned };
vm_block = 208; continue;
}
// C line 20278
211 => {
vm_block = 30; continue;
}
// C line 20278
213 => {
let _ = { let assigned = JS_NewBool(ctx, (((((((op1_30).u).uint64) as i32)) == (((((op2_16).u).uint64) as i32))) as i32)); *(sp).offset((((2 as i32)).wrapping_neg()) as isize) = assigned; assigned };
// C line 20278
let _ = { let old = sp; sp = (sp).offset(-1); old };
vm_block = 211; continue;
}
// C line 20278
214 => {
let _ = { let old = sp; sp = (sp).offset(-1); old };
vm_block = 211; continue;
}
// C line 20278
215 => {
vm_block = 29; continue;
}
// C line 20278
216 => {
vm_block = if (js_eq_slow(ctx, sp, (0 as i32))) != 0 { 215 } else { 214 }; continue;
}
// C line 20278
217 => {
let _ = { let assigned = pc; (*(sf)).cur_pc = assigned; assigned };
vm_block = 216; continue;
}
// C line 20278
218 => {
vm_block = if ((((!(((!((((((((((op1_30).tag) as i32)) | ((((op2_16).tag) as i32)))) == ((0 as i32))) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 213 } else { 217 }; continue;
}
// C line 20278
220 => {
let _ = { let assigned = *(sp).offset((((2 as i32)).wrapping_neg()) as isize); op1_30 = assigned; assigned };
// C line 20278
let _ = { let assigned = *(sp).offset((((1 as i32)).wrapping_neg()) as isize); op2_16 = assigned; assigned };
vm_block = 218; continue;
}
// C line 20277
221 => {
vm_block = 30; continue;
}
// C line 20277
223 => {
let _ = { let assigned = JS_NewBool(ctx, (((((((op1_29).u).uint64) as i32)) >= (((((op2_15).u).uint64) as i32))) as i32)); *(sp).offset((((2 as i32)).wrapping_neg()) as isize) = assigned; assigned };
// C line 20277
let _ = { let old = sp; sp = (sp).offset(-1); old };
vm_block = 221; continue;
}
// C line 20277
224 => {
let _ = { let old = sp; sp = (sp).offset(-1); old };
vm_block = 221; continue;
}
// C line 20277
225 => {
vm_block = 29; continue;
}
// C line 20277
226 => {
vm_block = if (js_relational_slow(ctx, sp, opcode)) != 0 { 225 } else { 224 }; continue;
}
// C line 20277
227 => {
let _ = { let assigned = pc; (*(sf)).cur_pc = assigned; assigned };
vm_block = 226; continue;
}
// C line 20277
228 => {
vm_block = if ((((!(((!((((((((((op1_29).tag) as i32)) | ((((op2_15).tag) as i32)))) == ((0 as i32))) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 223 } else { 227 }; continue;
}
// C line 20277
230 => {
let _ = { let assigned = *(sp).offset((((2 as i32)).wrapping_neg()) as isize); op1_29 = assigned; assigned };
// C line 20277
let _ = { let assigned = *(sp).offset((((1 as i32)).wrapping_neg()) as isize); op2_15 = assigned; assigned };
vm_block = 228; continue;
}
// C line 20276
231 => {
vm_block = 30; continue;
}
// C line 20276
233 => {
let _ = { let assigned = JS_NewBool(ctx, (((((((op1_28).u).uint64) as i32)) > (((((op2_14).u).uint64) as i32))) as i32)); *(sp).offset((((2 as i32)).wrapping_neg()) as isize) = assigned; assigned };
// C line 20276
let _ = { let old = sp; sp = (sp).offset(-1); old };
vm_block = 231; continue;
}
// C line 20276
234 => {
let _ = { let old = sp; sp = (sp).offset(-1); old };
vm_block = 231; continue;
}
// C line 20276
235 => {
vm_block = 29; continue;
}
// C line 20276
236 => {
vm_block = if (js_relational_slow(ctx, sp, opcode)) != 0 { 235 } else { 234 }; continue;
}
// C line 20276
237 => {
let _ = { let assigned = pc; (*(sf)).cur_pc = assigned; assigned };
vm_block = 236; continue;
}
// C line 20276
238 => {
vm_block = if ((((!(((!((((((((((op1_28).tag) as i32)) | ((((op2_14).tag) as i32)))) == ((0 as i32))) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 233 } else { 237 }; continue;
}
// C line 20276
240 => {
let _ = { let assigned = *(sp).offset((((2 as i32)).wrapping_neg()) as isize); op1_28 = assigned; assigned };
// C line 20276
let _ = { let assigned = *(sp).offset((((1 as i32)).wrapping_neg()) as isize); op2_14 = assigned; assigned };
vm_block = 238; continue;
}
// C line 20275
241 => {
vm_block = 30; continue;
}
// C line 20275
243 => {
let _ = { let assigned = JS_NewBool(ctx, (((((((op1_27).u).uint64) as i32)) <= (((((op2_13).u).uint64) as i32))) as i32)); *(sp).offset((((2 as i32)).wrapping_neg()) as isize) = assigned; assigned };
// C line 20275
let _ = { let old = sp; sp = (sp).offset(-1); old };
vm_block = 241; continue;
}
// C line 20275
244 => {
let _ = { let old = sp; sp = (sp).offset(-1); old };
vm_block = 241; continue;
}
// C line 20275
245 => {
vm_block = 29; continue;
}
// C line 20275
246 => {
vm_block = if (js_relational_slow(ctx, sp, opcode)) != 0 { 245 } else { 244 }; continue;
}
// C line 20275
247 => {
let _ = { let assigned = pc; (*(sf)).cur_pc = assigned; assigned };
vm_block = 246; continue;
}
// C line 20275
248 => {
vm_block = if ((((!(((!((((((((((op1_27).tag) as i32)) | ((((op2_13).tag) as i32)))) == ((0 as i32))) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 243 } else { 247 }; continue;
}
// C line 20275
250 => {
let _ = { let assigned = *(sp).offset((((2 as i32)).wrapping_neg()) as isize); op1_27 = assigned; assigned };
// C line 20275
let _ = { let assigned = *(sp).offset((((1 as i32)).wrapping_neg()) as isize); op2_13 = assigned; assigned };
vm_block = 248; continue;
}
// C line 20274
251 => {
vm_block = 30; continue;
}
// C line 20274
253 => {
let _ = { let assigned = JS_NewBool(ctx, (((((((op1_26).u).uint64) as i32)) < (((((op2_12).u).uint64) as i32))) as i32)); *(sp).offset((((2 as i32)).wrapping_neg()) as isize) = assigned; assigned };
// C line 20274
let _ = { let old = sp; sp = (sp).offset(-1); old };
vm_block = 251; continue;
}
// C line 20274
254 => {
let _ = { let old = sp; sp = (sp).offset(-1); old };
vm_block = 251; continue;
}
// C line 20274
255 => {
vm_block = 29; continue;
}
// C line 20274
256 => {
vm_block = if (js_relational_slow(ctx, sp, opcode)) != 0 { 255 } else { 254 }; continue;
}
// C line 20274
257 => {
let _ = { let assigned = pc; (*(sf)).cur_pc = assigned; assigned };
vm_block = 256; continue;
}
// C line 20274
258 => {
vm_block = if ((((!(((!((((((((((op1_26).tag) as i32)) | ((((op2_12).tag) as i32)))) == ((0 as i32))) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 253 } else { 257 }; continue;
}
// C line 20274
260 => {
let _ = { let assigned = *(sp).offset((((2 as i32)).wrapping_neg()) as isize); op1_26 = assigned; assigned };
// C line 20274
let _ = { let assigned = *(sp).offset((((1 as i32)).wrapping_neg()) as isize); op2_12 = assigned; assigned };
vm_block = 258; continue;
}
// C line 20253
261 => {
vm_block = 30; continue;
}
// C line 20242
263 => {
let _ = { let assigned = JS_NewInt32(ctx, ((((((op1_25).u).uint64) as i32)) ^ (((((op2_11).u).uint64) as i32)))); *(sp).offset((((2 as i32)).wrapping_neg()) as isize) = assigned; assigned };
// C line 20245
let _ = { let old = sp; sp = (sp).offset(-1); old };
vm_block = 261; continue;
}
// C line 20250
264 => {
let _ = { let old = sp; sp = (sp).offset(-1); old };
vm_block = 261; continue;
}
// C line 20249
265 => {
vm_block = 29; continue;
}
// C line 20248
266 => {
vm_block = if (js_binary_logic_slow(ctx, sp, opcode)) != 0 { 265 } else { 264 }; continue;
}
// C line 20247
267 => {
let _ = { let assigned = pc; (*(sf)).cur_pc = assigned; assigned };
vm_block = 266; continue;
}
// C line 20241
268 => {
vm_block = if ((((!(((!((((((((((op1_25).tag) as i32)) | ((((op2_11).tag) as i32)))) == ((0 as i32))) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 263 } else { 267 }; continue;
}
// C line 20239
270 => {
let _ = { let assigned = *(sp).offset((((2 as i32)).wrapping_neg()) as isize); op1_25 = assigned; assigned };
// C line 20240
let _ = { let assigned = *(sp).offset((((1 as i32)).wrapping_neg()) as isize); op2_11 = assigned; assigned };
vm_block = 268; continue;
}
// C line 20235
271 => {
vm_block = 30; continue;
}
// C line 20224
273 => {
let _ = { let assigned = JS_NewInt32(ctx, ((((((op1_24).u).uint64) as i32)) | (((((op2_10).u).uint64) as i32)))); *(sp).offset((((2 as i32)).wrapping_neg()) as isize) = assigned; assigned };
// C line 20227
let _ = { let old = sp; sp = (sp).offset(-1); old };
vm_block = 271; continue;
}
// C line 20232
274 => {
let _ = { let old = sp; sp = (sp).offset(-1); old };
vm_block = 271; continue;
}
// C line 20231
275 => {
vm_block = 29; continue;
}
// C line 20230
276 => {
vm_block = if (js_binary_logic_slow(ctx, sp, opcode)) != 0 { 275 } else { 274 }; continue;
}
// C line 20229
277 => {
let _ = { let assigned = pc; (*(sf)).cur_pc = assigned; assigned };
vm_block = 276; continue;
}
// C line 20223
278 => {
vm_block = if ((((!(((!((((((((((op1_24).tag) as i32)) | ((((op2_10).tag) as i32)))) == ((0 as i32))) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 273 } else { 277 }; continue;
}
// C line 20221
280 => {
let _ = { let assigned = *(sp).offset((((2 as i32)).wrapping_neg()) as isize); op1_24 = assigned; assigned };
// C line 20222
let _ = { let assigned = *(sp).offset((((1 as i32)).wrapping_neg()) as isize); op2_10 = assigned; assigned };
vm_block = 278; continue;
}
// C line 20217
281 => {
vm_block = 30; continue;
}
// C line 20206
283 => {
let _ = { let assigned = JS_NewInt32(ctx, ((((((op1_23).u).uint64) as i32)) & (((((op2_9).u).uint64) as i32)))); *(sp).offset((((2 as i32)).wrapping_neg()) as isize) = assigned; assigned };
// C line 20209
let _ = { let old = sp; sp = (sp).offset(-1); old };
vm_block = 281; continue;
}
// C line 20214
284 => {
let _ = { let old = sp; sp = (sp).offset(-1); old };
vm_block = 281; continue;
}
// C line 20213
285 => {
vm_block = 29; continue;
}
// C line 20212
286 => {
vm_block = if (js_binary_logic_slow(ctx, sp, opcode)) != 0 { 285 } else { 284 }; continue;
}
// C line 20211
287 => {
let _ = { let assigned = pc; (*(sf)).cur_pc = assigned; assigned };
vm_block = 286; continue;
}
// C line 20205
288 => {
vm_block = if ((((!(((!((((((((((op1_23).tag) as i32)) | ((((op2_9).tag) as i32)))) == ((0 as i32))) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 283 } else { 287 }; continue;
}
// C line 20203
290 => {
let _ = { let assigned = *(sp).offset((((2 as i32)).wrapping_neg()) as isize); op1_23 = assigned; assigned };
// C line 20204
let _ = { let assigned = *(sp).offset((((1 as i32)).wrapping_neg()) as isize); op2_9 = assigned; assigned };
vm_block = 288; continue;
}
// C line 20199
291 => {
vm_block = 30; continue;
}
// C line 20187
295 => {
let _ = { let assigned = ((((((op2_8).u).uint64) as i32)) as u32); v2_5 = assigned; assigned };
// C line 20188
let _ = { v2_5 = ((v2_5) & ((((31 as i32)) as u32))); v2_5 };
// C line 20189
let _ = { let assigned = JS_NewInt32(ctx, (((((op1_22).u).uint64) as i32)).wrapping_shr((v2_5) as u32)); *(sp).offset((((2 as i32)).wrapping_neg()) as isize) = assigned; assigned };
// C line 20191
let _ = { let old = sp; sp = (sp).offset(-1); old };
vm_block = 291; continue;
}
// C line 20196
296 => {
let _ = { let old = sp; sp = (sp).offset(-1); old };
vm_block = 291; continue;
}
// C line 20195
297 => {
vm_block = 29; continue;
}
// C line 20194
298 => {
vm_block = if (js_binary_logic_slow(ctx, sp, opcode)) != 0 { 297 } else { 296 }; continue;
}
// C line 20193
299 => {
let _ = { let assigned = pc; (*(sf)).cur_pc = assigned; assigned };
vm_block = 298; continue;
}
// C line 20185
300 => {
vm_block = if ((((!(((!((((((((((op1_22).tag) as i32)) | ((((op2_8).tag) as i32)))) == ((0 as i32))) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 295 } else { 299 }; continue;
}
// C line 20183
302 => {
let _ = { let assigned = *(sp).offset((((2 as i32)).wrapping_neg()) as isize); op1_22 = assigned; assigned };
// C line 20184
let _ = { let assigned = *(sp).offset((((1 as i32)).wrapping_neg()) as isize); op2_8 = assigned; assigned };
vm_block = 300; continue;
}
// C line 20179
303 => {
vm_block = 30; continue;
}
// C line 20166
307 => {
let _ = { let assigned = ((((((op2_7).u).uint64) as i32)) as u32); v2_4 = assigned; assigned };
// C line 20167
let _ = { v2_4 = ((v2_4) & ((((31 as i32)) as u32))); v2_4 };
// C line 20168
let _ = { let assigned = JS_NewUint32(ctx, (((((((op1_21).u).uint64) as i32)) as u32)).wrapping_shr((v2_4) as u32)); *(sp).offset((((2 as i32)).wrapping_neg()) as isize) = assigned; assigned };
// C line 20171
let _ = { let old = sp; sp = (sp).offset(-1); old };
vm_block = 303; continue;
}
// C line 20176
308 => {
let _ = { let old = sp; sp = (sp).offset(-1); old };
vm_block = 303; continue;
}
// C line 20175
309 => {
vm_block = 29; continue;
}
// C line 20174
310 => {
vm_block = if (js_shr_slow(ctx, sp)) != 0 { 309 } else { 308 }; continue;
}
// C line 20173
311 => {
let _ = { let assigned = pc; (*(sf)).cur_pc = assigned; assigned };
vm_block = 310; continue;
}
// C line 20164
312 => {
vm_block = if ((((!(((!((((((((((op1_21).tag) as i32)) | ((((op2_7).tag) as i32)))) == ((0 as i32))) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 307 } else { 311 }; continue;
}
// C line 20162
314 => {
let _ = { let assigned = *(sp).offset((((2 as i32)).wrapping_neg()) as isize); op1_21 = assigned; assigned };
// C line 20163
let _ = { let assigned = *(sp).offset((((1 as i32)).wrapping_neg()) as isize); op2_7 = assigned; assigned };
vm_block = 312; continue;
}
// C line 20158
315 => {
vm_block = 30; continue;
}
// C line 20146
320 => {
let _ = { let assigned = ((((((op1_20).u).uint64) as i32)) as u32); v1_3 = assigned; assigned };
// C line 20147
let _ = { let assigned = ((((((op2_6).u).uint64) as i32)) as u32); v2_3 = assigned; assigned };
// C line 20148
let _ = { v2_3 = ((v2_3) & ((((31 as i32)) as u32))); v2_3 };
// C line 20149
let _ = { let assigned = JS_NewInt32(ctx, (((v1_3).wrapping_shl((v2_3) as u32)) as i32)); *(sp).offset((((2 as i32)).wrapping_neg()) as isize) = assigned; assigned };
// C line 20150
let _ = { let old = sp; sp = (sp).offset(-1); old };
vm_block = 315; continue;
}
// C line 20155
321 => {
let _ = { let old = sp; sp = (sp).offset(-1); old };
vm_block = 315; continue;
}
// C line 20154
322 => {
vm_block = 29; continue;
}
// C line 20153
323 => {
vm_block = if (js_binary_logic_slow(ctx, sp, opcode)) != 0 { 322 } else { 321 }; continue;
}
// C line 20152
324 => {
let _ = { let assigned = pc; (*(sf)).cur_pc = assigned; assigned };
vm_block = 323; continue;
}
// C line 20144
325 => {
vm_block = if ((((!(((!((((((((((op1_20).tag) as i32)) | ((((op2_6).tag) as i32)))) == ((0 as i32))) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 320 } else { 324 }; continue;
}
// C line 20142
327 => {
let _ = { let assigned = *(sp).offset((((2 as i32)).wrapping_neg()) as isize); op1_20 = assigned; assigned };
// C line 20143
let _ = { let assigned = *(sp).offset((((1 as i32)).wrapping_neg()) as isize); op2_6 = assigned; assigned };
vm_block = 325; continue;
}
// C line 20137
328 => {
vm_block = 30; continue;
}
// C line 20130
329 => {
let _ = { let assigned = JS_NewInt32(ctx, (!(((((op1_19).u).uint64) as i32)))); *(sp).offset((((1 as i32)).wrapping_neg()) as isize) = assigned; assigned };
vm_block = 328; continue;
}
// C line 20134
330 => {
vm_block = 29; continue;
}
// C line 20133
331 => {
vm_block = if (js_not_slow(ctx, sp)) != 0 { 330 } else { 328 }; continue;
}
// C line 20132
332 => {
let _ = { let assigned = pc; (*(sf)).cur_pc = assigned; assigned };
vm_block = 331; continue;
}
// C line 20129
333 => {
vm_block = if (((((((op1_19).tag) as i32)) == ((JS_TAG_INT as i32))) as i32)) != 0 { 329 } else { 332 }; continue;
}
// C line 20128
334 => {
let _ = { let assigned = *(sp).offset((((1 as i32)).wrapping_neg()) as isize); op1_19 = assigned; assigned };
vm_block = 333; continue;
}
// C line 20124
335 => {
vm_block = 30; continue;
}
// C line 20112
336 => {
let _ = { let assigned = JS_NewInt32(ctx, (val_21).wrapping_sub((1 as i32))); *(var_buf).offset((idx_28) as isize) = assigned; assigned };
vm_block = 335; continue;
}
// C line 20111
337 => {
vm_block = 344; continue;
}
// C line 20110
338 => {
vm_block = if ((((!(((!(((((val_21) == ((((2147483647 as i32)).wrapping_neg()).wrapping_sub((1 as i32)))) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 337 } else { 336 }; continue;
}
// C line 20109
339 => {
let _ = { let assigned = ((((op1_18).u).uint64) as i32); val_21 = assigned; assigned };
vm_block = 338; continue;
}
// C line 20121
340 => {
let _ = set_value(ctx, core::ptr::addr_of_mut!(*(var_buf).offset((idx_28) as isize)), op1_18);
vm_block = 335; continue;
}
// C line 20120
341 => {
vm_block = 29; continue;
}
// C line 20119
342 => {
vm_block = if (js_unary_arith_slow(ctx, (core::ptr::addr_of_mut!(op1_18)).offset((((1 as i32)) as isize)), (OP_dec as i32))) != 0 { 341 } else { 340 }; continue;
}
// C line ? labels: dec_loc_slow
344 => {
let _ = { let assigned = pc; (*(sf)).cur_pc = assigned; assigned };
// C line 20118
let _ = { let assigned = JS_DupValue(ctx, op1_18); op1_18 = assigned; assigned };
vm_block = 342; continue;
}
// C line 20108
345 => {
vm_block = if (((((((op1_18).tag) as i32)) == ((JS_TAG_INT as i32))) as i32)) != 0 { 339 } else { 344 }; continue;
}
// C line 20104
348 => {
let _ = { let assigned = ((*(pc)) as i32); idx_28 = assigned; assigned };
// C line 20105
let _ = { pc = (pc).offset((((1 as i32)) as isize)); pc };
// C line 20107
let _ = { let assigned = *(var_buf).offset((idx_28) as isize); op1_18 = assigned; assigned };
vm_block = 345; continue;
}
// C line 20098
349 => {
vm_block = 30; continue;
}
// C line 20086
350 => {
let _ = { let assigned = JS_NewInt32(ctx, (val_20).wrapping_add((1 as i32))); *(var_buf).offset((idx_27) as isize) = assigned; assigned };
vm_block = 349; continue;
}
// C line 20085
351 => {
vm_block = 358; continue;
}
// C line 20084
352 => {
vm_block = if ((((!(((!(((((val_20) == ((2147483647 as i32))) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 351 } else { 350 }; continue;
}
// C line 20083
353 => {
let _ = { let assigned = ((((op1_17).u).uint64) as i32); val_20 = assigned; assigned };
vm_block = 352; continue;
}
// C line 20095
354 => {
let _ = set_value(ctx, core::ptr::addr_of_mut!(*(var_buf).offset((idx_27) as isize)), op1_17);
vm_block = 349; continue;
}
// C line 20094
355 => {
vm_block = 29; continue;
}
// C line 20093
356 => {
vm_block = if (js_unary_arith_slow(ctx, (core::ptr::addr_of_mut!(op1_17)).offset((((1 as i32)) as isize)), (OP_inc as i32))) != 0 { 355 } else { 354 }; continue;
}
// C line ? labels: inc_loc_slow
358 => {
let _ = { let assigned = pc; (*(sf)).cur_pc = assigned; assigned };
// C line 20092
let _ = { let assigned = JS_DupValue(ctx, op1_17); op1_17 = assigned; assigned };
vm_block = 356; continue;
}
// C line 20082
359 => {
vm_block = if (((((((op1_17).tag) as i32)) == ((JS_TAG_INT as i32))) as i32)) != 0 { 353 } else { 358 }; continue;
}
// C line 20078
362 => {
let _ = { let assigned = ((*(pc)) as i32); idx_27 = assigned; assigned };
// C line 20079
let _ = { pc = (pc).offset((((1 as i32)) as isize)); pc };
// C line 20081
let _ = { let assigned = *(var_buf).offset((idx_27) as isize); op1_17 = assigned; assigned };
vm_block = 359; continue;
}
// C line 20070
364 => {
let _ = { let old = sp; sp = (sp).offset(1); old };
// C line 20072
vm_block = 30; continue;
}
// C line 20063
365 => {
let _ = { let assigned = JS_NewInt32(ctx, (val_19).wrapping_sub((1 as i32))); *(sp).offset(((0 as i32)) as isize) = assigned; assigned };
vm_block = 364; continue;
}
// C line 20062
366 => {
vm_block = 371; continue;
}
// C line 20061
367 => {
vm_block = if ((((!(((!(((((val_19) == ((((2147483647 as i32)).wrapping_neg()).wrapping_sub((1 as i32)))) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 366 } else { 365 }; continue;
}
// C line 20060
368 => {
let _ = { let assigned = ((((op1_16).u).uint64) as i32); val_19 = assigned; assigned };
vm_block = 367; continue;
}
// C line 20068
369 => {
vm_block = 29; continue;
}
// C line 20067
370 => {
vm_block = if (js_post_inc_slow(ctx, sp, opcode)) != 0 { 369 } else { 364 }; continue;
}
// C line ? labels: post_dec_slow
371 => {
let _ = { let assigned = pc; (*(sf)).cur_pc = assigned; assigned };
vm_block = 370; continue;
}
// C line 20059
372 => {
vm_block = if (((((((op1_16).tag) as i32)) == ((JS_TAG_INT as i32))) as i32)) != 0 { 368 } else { 371 }; continue;
}
// C line 20058
373 => {
let _ = { let assigned = *(sp).offset((((1 as i32)).wrapping_neg()) as isize); op1_16 = assigned; assigned };
vm_block = 372; continue;
}
// C line 20051
375 => {
let _ = { let old = sp; sp = (sp).offset(1); old };
// C line 20053
vm_block = 30; continue;
}
// C line 20044
376 => {
let _ = { let assigned = JS_NewInt32(ctx, (val_18).wrapping_add((1 as i32))); *(sp).offset(((0 as i32)) as isize) = assigned; assigned };
vm_block = 375; continue;
}
// C line 20043
377 => {
vm_block = 382; continue;
}
// C line 20042
378 => {
vm_block = if ((((!(((!(((((val_18) == ((2147483647 as i32))) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 377 } else { 376 }; continue;
}
// C line 20041
379 => {
let _ = { let assigned = ((((op1_15).u).uint64) as i32); val_18 = assigned; assigned };
vm_block = 378; continue;
}
// C line 20049
380 => {
vm_block = 29; continue;
}
// C line 20048
381 => {
vm_block = if (js_post_inc_slow(ctx, sp, opcode)) != 0 { 380 } else { 375 }; continue;
}
// C line ? labels: post_inc_slow
382 => {
let _ = { let assigned = pc; (*(sf)).cur_pc = assigned; assigned };
vm_block = 381; continue;
}
// C line 20040
383 => {
vm_block = if (((((((op1_15).tag) as i32)) == ((JS_TAG_INT as i32))) as i32)) != 0 { 379 } else { 382 }; continue;
}
// C line 20039
384 => {
let _ = { let assigned = *(sp).offset((((1 as i32)).wrapping_neg()) as isize); op1_15 = assigned; assigned };
vm_block = 383; continue;
}
// C line 20034
385 => {
vm_block = 30; continue;
}
// C line 20026
386 => {
let _ = { let assigned = JS_NewInt32(ctx, (val_17).wrapping_sub((1 as i32))); *(sp).offset((((1 as i32)).wrapping_neg()) as isize) = assigned; assigned };
vm_block = 385; continue;
}
// C line 20025
387 => {
vm_block = 392; continue;
}
// C line 20024
388 => {
vm_block = if ((((!(((!(((((val_17) == ((((2147483647 as i32)).wrapping_neg()).wrapping_sub((1 as i32)))) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 387 } else { 386 }; continue;
}
// C line 20023
389 => {
let _ = { let assigned = ((((op1_14).u).uint64) as i32); val_17 = assigned; assigned };
vm_block = 388; continue;
}
// C line 20031
390 => {
vm_block = 29; continue;
}
// C line 20030
391 => {
vm_block = if (js_unary_arith_slow(ctx, sp, opcode)) != 0 { 390 } else { 385 }; continue;
}
// C line ? labels: dec_slow
392 => {
let _ = { let assigned = pc; (*(sf)).cur_pc = assigned; assigned };
vm_block = 391; continue;
}
// C line 20022
393 => {
vm_block = if (((((((op1_14).tag) as i32)) == ((JS_TAG_INT as i32))) as i32)) != 0 { 389 } else { 392 }; continue;
}
// C line 20021
394 => {
let _ = { let assigned = *(sp).offset((((1 as i32)).wrapping_neg()) as isize); op1_14 = assigned; assigned };
vm_block = 393; continue;
}
// C line 20016
395 => {
vm_block = 30; continue;
}
// C line 20008
396 => {
let _ = { let assigned = JS_NewInt32(ctx, (val_16).wrapping_add((1 as i32))); *(sp).offset((((1 as i32)).wrapping_neg()) as isize) = assigned; assigned };
vm_block = 395; continue;
}
// C line 20007
397 => {
vm_block = 402; continue;
}
// C line 20006
398 => {
vm_block = if ((((!(((!(((((val_16) == ((2147483647 as i32))) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 397 } else { 396 }; continue;
}
// C line 20005
399 => {
let _ = { let assigned = ((((op1_13).u).uint64) as i32); val_16 = assigned; assigned };
vm_block = 398; continue;
}
// C line 20013
400 => {
vm_block = 29; continue;
}
// C line 20012
401 => {
vm_block = if (js_unary_arith_slow(ctx, sp, opcode)) != 0 { 400 } else { 395 }; continue;
}
// C line ? labels: inc_slow
402 => {
let _ = { let assigned = pc; (*(sf)).cur_pc = assigned; assigned };
vm_block = 401; continue;
}
// C line 20004
403 => {
vm_block = if (((((((op1_13).tag) as i32)) == ((JS_TAG_INT as i32))) as i32)) != 0 { 399 } else { 402 }; continue;
}
// C line 20003
404 => {
let _ = { let assigned = *(sp).offset((((1 as i32)).wrapping_neg()) as isize); op1_13 = assigned; assigned };
vm_block = 403; continue;
}
// C line 19998
405 => {
vm_block = 30; continue;
}
// C line 19987
406 => {
let _ = { let assigned = JS_NewInt32(ctx, (val_15).wrapping_neg()); *(sp).offset((((1 as i32)).wrapping_neg()) as isize) = assigned; assigned };
vm_block = 405; continue;
}
// C line 19984
408 => {
let _ = { let assigned = (-(((val_15) as f64))); d_1 = assigned; assigned };
// C line 19985
vm_block = 414; continue;
}
// C line 19983
409 => {
vm_block = if ((((!(((!(((((val_15) == ((((2147483647 as i32)).wrapping_neg()).wrapping_sub((1 as i32)))) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 408 } else { 406 }; continue;
}
// C line 19980
411 => {
let _ = { let assigned = (-((0 as f64))); d_1 = assigned; assigned };
// C line 19981
vm_block = 414; continue;
}
// C line 19979
412 => {
vm_block = if ((((!(((!(((((val_15) == ((0 as i32))) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 411 } else { 409 }; continue;
}
// C line 19977
413 => {
let _ = { let assigned = ((((op1_12).u).uint64) as i32); val_15 = assigned; assigned };
vm_block = 412; continue;
}
// C line ? labels: neg_fp_res
414 => {
let _ = { let assigned = __JS_NewFloat64(ctx, d_1); *(sp).offset((((1 as i32)).wrapping_neg()) as isize) = assigned; assigned };
vm_block = 405; continue;
}
// C line 19989
415 => {
let _ = { let assigned = (-(((op1_12).u).float64)); d_1 = assigned; assigned };
vm_block = 414; continue;
}
// C line 19995
416 => {
vm_block = 29; continue;
}
// C line 19994
417 => {
vm_block = if (js_unary_arith_slow(ctx, sp, opcode)) != 0 { 416 } else { 405 }; continue;
}
// C line 19993
418 => {
let _ = { let assigned = pc; (*(sf)).cur_pc = assigned; assigned };
vm_block = 417; continue;
}
// C line 19988
419 => {
vm_block = if ((((tag_2) == ((((JS_TAG_FLOAT64 as i32)) as u32))) as i32)) != 0 { 415 } else { 418 }; continue;
}
// C line 19974
420 => {
vm_block = if ((((((((((((tag_2) == ((((JS_TAG_INT as i32)) as u32))) as i32)) != 0) || (((((tag_2) == ((((JS_TAG_BOOL as i32)) as u32))) as i32)) != 0)) as i32)) != 0) || (((((tag_2) == ((((JS_TAG_NULL as i32)) as u32))) as i32)) != 0)) as i32)) != 0 { 413 } else { 419 }; continue;
}
// C line 19972
422 => {
let _ = { let assigned = *(sp).offset((((1 as i32)).wrapping_neg()) as isize); op1_12 = assigned; assigned };
// C line 19973
let _ = { let assigned = (((((op1_12).tag) as i32)) as u32); tag_2 = assigned; assigned };
vm_block = 420; continue;
}
// C line 19965
423 => {
vm_block = 30; continue;
}
// C line 19958
424 => {
let _ = { let assigned = JS_NewInt32(ctx, ((((op1_11).u).uint64) as i32)); *(sp).offset((((1 as i32)).wrapping_neg()) as isize) = assigned; assigned };
vm_block = 423; continue;
}
// C line 19962
425 => {
vm_block = 29; continue;
}
// C line 19961
426 => {
vm_block = if (js_unary_arith_slow(ctx, sp, opcode)) != 0 { 425 } else { 423 }; continue;
}
// C line 19960
427 => {
let _ = { let assigned = pc; (*(sf)).cur_pc = assigned; assigned };
vm_block = 426; continue;
}
// C line ?
428 => {
vm_block = if ((((((((tag_1) == ((((JS_TAG_NULL as i32)) as u32))) as i32)) != 0) || (((((tag_1) == ((((JS_TAG_BOOL as i32)) as u32))) as i32)) != 0)) as i32)) != 0 { 424 } else { 427 }; continue;
}
// C line 19956
429 => {
vm_block = if ((((((((tag_1) == ((((JS_TAG_INT as i32)) as u32))) as i32)) != 0) || (((((tag_1) == ((((JS_TAG_FLOAT64 as i32)) as u32))) as i32)) != 0)) as i32)) != 0 { 423 } else { 428 }; continue;
}
// C line 19954
431 => {
let _ = { let assigned = *(sp).offset((((1 as i32)).wrapping_neg()) as isize); op1_11 = assigned; assigned };
// C line 19955
let _ = { let assigned = (((((op1_11).tag) as i32)) as u32); tag_1 = assigned; assigned };
vm_block = 429; continue;
}
// C line 19947
433 => {
let _ = { let old = sp; sp = (sp).offset(-1); old };
// C line 19948
vm_block = 30; continue;
}
// C line 19946
434 => {
vm_block = 29; continue;
}
// C line 19945
435 => {
vm_block = if (js_binary_arith_slow(ctx, sp, opcode)) != 0 { 434 } else { 433 }; continue;
}
// C line ? labels: binary_arith_slow
436 => {
let _ = { let assigned = pc; (*(sf)).cur_pc = assigned; assigned };
vm_block = 435; continue;
}
// C line 19934
440 => {
let _ = { let assigned = ((v1_2) % (v2_2)); r_4 = assigned; assigned };
// C line 19935
let _ = { let assigned = JS_NewInt32(ctx, r_4); *(sp).offset((((2 as i32)).wrapping_neg()) as isize) = assigned; assigned };
// C line 19936
let _ = { let old = sp; sp = (sp).offset(-1); old };
// C line 19941
vm_block = 30; continue;
}
// C line 19933
441 => {
vm_block = 436; continue;
}
// C line 19932
442 => {
vm_block = if ((((!(((!(((((((((v1_2) < ((0 as i32))) as i32)) != 0) || (((((v2_2) <= ((0 as i32))) as i32)) != 0)) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 441 } else { 440 }; continue;
}
// C line 19928
444 => {
let _ = { let assigned = ((((op1_10).u).uint64) as i32); v1_2 = assigned; assigned };
// C line 19929
let _ = { let assigned = ((((op2_5).u).uint64) as i32); v2_2 = assigned; assigned };
vm_block = 442; continue;
}
// C line 19938
445 => {
vm_block = 436; continue;
}
// C line 19926
446 => {
vm_block = if ((((!(((!((((((((((op1_10).tag) as i32)) | ((((op2_5).tag) as i32)))) == ((0 as i32))) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 444 } else { 445 }; continue;
}
// C line 19924
448 => {
let _ = { let assigned = *(sp).offset((((2 as i32)).wrapping_neg()) as isize); op1_10 = assigned; assigned };
// C line 19925
let _ = { let assigned = *(sp).offset((((1 as i32)).wrapping_neg()) as isize); op2_5 = assigned; assigned };
vm_block = 446; continue;
}
// C line 19912
453 => {
let _ = { let assigned = ((((op1_9).u).uint64) as i32); v1_1 = assigned; assigned };
// C line 19913
let _ = { let assigned = ((((op2_4).u).uint64) as i32); v2_1 = assigned; assigned };
// C line 19914
let _ = { let assigned = JS_NewFloat64(ctx, ((((v1_1) as f64)) / (((v2_1) as f64)))); *(sp).offset((((2 as i32)).wrapping_neg()) as isize) = assigned; assigned };
// C line 19915
let _ = { let old = sp; sp = (sp).offset(-1); old };
// C line 19920
vm_block = 30; continue;
}
// C line 19917
454 => {
vm_block = 436; continue;
}
// C line 19910
455 => {
vm_block = if ((((!(((!((((((((((op1_9).tag) as i32)) | ((((op2_4).tag) as i32)))) == ((0 as i32))) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 453 } else { 454 }; continue;
}
// C line 19908
457 => {
let _ = { let assigned = *(sp).offset((((2 as i32)).wrapping_neg()) as isize); op1_9 = assigned; assigned };
// C line 19909
let _ = { let assigned = *(sp).offset((((1 as i32)).wrapping_neg()) as isize); op2_4 = assigned; assigned };
vm_block = 455; continue;
}
// C line 19904
458 => {
vm_block = 30; continue;
}
// C line 19877
460 => {
let _ = { let assigned = JS_NewInt32(ctx, ((r_3) as i32)); *(sp).offset((((2 as i32)).wrapping_neg()) as isize) = assigned; assigned };
// C line 19878
let _ = { let old = sp; sp = (sp).offset(-1); old };
vm_block = 458; continue;
}
// C line 19874
462 => {
let _ = { let assigned = (-((0 as f64))); d = assigned; assigned };
// C line 19875
vm_block = 471; continue;
}
// C line 19873
463 => {
vm_block = if ((((!(((!(((((((((r_3) == ((((0 as i32)) as i64))) as i32)) != 0) && (((((((v1) | (v2))) < ((0 as i32))) as i32)) != 0)) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 462 } else { 460 }; continue;
}
// C line 19869
465 => {
let _ = { let assigned = ((r_3) as f64); d = assigned; assigned };
// C line 19870
vm_block = 471; continue;
}
// C line 19868
466 => {
vm_block = if ((((!(((!(((((((((r_3) as i32)) as i64)) != (r_3)) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 465 } else { 463 }; continue;
}
// C line 19865
469 => {
let _ = { let assigned = ((((op1_8).u).uint64) as i32); v1 = assigned; assigned };
// C line 19866
let _ = { let assigned = ((((op2_3).u).uint64) as i32); v2 = assigned; assigned };
// C line 19867
let _ = { let assigned = (((v1) as i64)).wrapping_mul(((v2) as i64)); r_3 = assigned; assigned };
vm_block = 466; continue;
}
// C line ? labels: mul_fp_res
471 => {
let _ = { let assigned = __JS_NewFloat64(ctx, d); *(sp).offset((((2 as i32)).wrapping_neg()) as isize) = assigned; assigned };
// C line 19899
let _ = { let old = sp; sp = (sp).offset(-1); old };
vm_block = 458; continue;
}
// C line 19896
472 => {
let _ = { let assigned = ((d1_2) * (d2_2)); d = assigned; assigned };
vm_block = 471; continue;
}
// C line 19890
473 => {
let _ = { let assigned = ((op2_3).u).float64; d2_2 = assigned; assigned };
vm_block = 472; continue;
}
// C line 19892
474 => {
let _ = { let assigned = ((((((op2_3).u).uint64) as i32)) as f64); d2_2 = assigned; assigned };
vm_block = 472; continue;
}
// C line 19894
475 => {
vm_block = 436; continue;
}
// C line 19891
476 => {
vm_block = if (((((((op2_3).tag) as i32)) == ((JS_TAG_INT as i32))) as i32)) != 0 { 474 } else { 475 }; continue;
}
// C line 19889
477 => {
vm_block = if (((((((((op2_3).tag) as i32)) as u32)) == ((((JS_TAG_FLOAT64 as i32)) as u32))) as i32)) != 0 { 473 } else { 476 }; continue;
}
// C line 19883
478 => {
let _ = { let assigned = ((op1_8).u).float64; d1_2 = assigned; assigned };
vm_block = 477; continue;
}
// C line 19885
479 => {
let _ = { let assigned = ((((((op1_8).u).uint64) as i32)) as f64); d1_2 = assigned; assigned };
vm_block = 477; continue;
}
// C line 19887
480 => {
vm_block = 436; continue;
}
// C line 19884
481 => {
vm_block = if (((((((op1_8).tag) as i32)) == ((JS_TAG_INT as i32))) as i32)) != 0 { 479 } else { 480 }; continue;
}
// C line 19882
482 => {
vm_block = if (((((((((op1_8).tag) as i32)) as u32)) == ((((JS_TAG_FLOAT64 as i32)) as u32))) as i32)) != 0 { 478 } else { 481 }; continue;
}
// C line 19901
483 => {
vm_block = 436; continue;
}
// C line 19879
484 => {
vm_block = if (((((((((((((op1_8).tag) as i32)) as u32)) == ((((JS_TAG_FLOAT64 as i32)) as u32))) as i32)) != 0) || ((((((((((op2_3).tag) as i32)) as u32)) == ((((JS_TAG_FLOAT64 as i32)) as u32))) as i32)) != 0)) as i32)) != 0 { 482 } else { 483 }; continue;
}
// C line 19862
485 => {
vm_block = if ((((!(((!((((((((((op1_8).tag) as i32)) | ((((op2_3).tag) as i32)))) == ((0 as i32))) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 469 } else { 484 }; continue;
}
// C line 19860
487 => {
let _ = { let assigned = *(sp).offset((((2 as i32)).wrapping_neg()) as isize); op1_8 = assigned; assigned };
// C line 19861
let _ = { let assigned = *(sp).offset((((1 as i32)).wrapping_neg()) as isize); op2_3 = assigned; assigned };
vm_block = 485; continue;
}
// C line 19855
488 => {
vm_block = 30; continue;
}
// C line 19831
489 => {
let _ = { let old = sp; sp = (sp).offset(-1); old };
vm_block = 488; continue;
}
// C line 19827
490 => {
let _ = { let assigned = __JS_NewFloat64(ctx, ((r_2) as f64)); *(sp).offset((((2 as i32)).wrapping_neg()) as isize) = assigned; assigned };
vm_block = 489; continue;
}
// C line 19829
491 => {
let _ = { let assigned = JS_NewInt32(ctx, ((r_2) as i32)); *(sp).offset((((2 as i32)).wrapping_neg()) as isize) = assigned; assigned };
vm_block = 489; continue;
}
// C line 19826
492 => {
vm_block = if ((((!(((!(((((((((r_2) as i32)) as i64)) != (r_2)) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 490 } else { 491 }; continue;
}
// C line 19825
493 => {
let _ = { let assigned = (((((((op1_7).u).uint64) as i32)) as i64)).wrapping_sub(((((((op2_2).u).uint64) as i32)) as i64)); r_2 = assigned; assigned };
vm_block = 492; continue;
}
// C line 19849
495 => {
let _ = { let assigned = __JS_NewFloat64(ctx, ((d1_1) - (d2_1))); *(sp).offset((((2 as i32)).wrapping_neg()) as isize) = assigned; assigned };
// C line 19850
let _ = { let old = sp; sp = (sp).offset(-1); old };
vm_block = 488; continue;
}
// C line 19843
496 => {
let _ = { let assigned = ((op2_2).u).float64; d2_1 = assigned; assigned };
vm_block = 495; continue;
}
// C line 19845
497 => {
let _ = { let assigned = ((((((op2_2).u).uint64) as i32)) as f64); d2_1 = assigned; assigned };
vm_block = 495; continue;
}
// C line 19847
498 => {
vm_block = 436; continue;
}
// C line 19844
499 => {
vm_block = if (((((((op2_2).tag) as i32)) == ((JS_TAG_INT as i32))) as i32)) != 0 { 497 } else { 498 }; continue;
}
// C line 19842
500 => {
vm_block = if (((((((((op2_2).tag) as i32)) as u32)) == ((((JS_TAG_FLOAT64 as i32)) as u32))) as i32)) != 0 { 496 } else { 499 }; continue;
}
// C line 19836
501 => {
let _ = { let assigned = ((op1_7).u).float64; d1_1 = assigned; assigned };
vm_block = 500; continue;
}
// C line 19838
502 => {
let _ = { let assigned = ((((((op1_7).u).uint64) as i32)) as f64); d1_1 = assigned; assigned };
vm_block = 500; continue;
}
// C line 19840
503 => {
vm_block = 436; continue;
}
// C line 19837
504 => {
vm_block = if (((((((op1_7).tag) as i32)) == ((JS_TAG_INT as i32))) as i32)) != 0 { 502 } else { 503 }; continue;
}
// C line 19835
505 => {
vm_block = if (((((((((op1_7).tag) as i32)) as u32)) == ((((JS_TAG_FLOAT64 as i32)) as u32))) as i32)) != 0 { 501 } else { 504 }; continue;
}
// C line 19852
506 => {
vm_block = 436; continue;
}
// C line 19832
507 => {
vm_block = if (((((((((((((op1_7).tag) as i32)) as u32)) == ((((JS_TAG_FLOAT64 as i32)) as u32))) as i32)) != 0) || ((((((((((op2_2).tag) as i32)) as u32)) == ((((JS_TAG_FLOAT64 as i32)) as u32))) as i32)) != 0)) as i32)) != 0 { 505 } else { 506 }; continue;
}
// C line 19823
508 => {
vm_block = if ((((!(((!((((((((((op1_7).tag) as i32)) | ((((op2_2).tag) as i32)))) == ((0 as i32))) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 493 } else { 507 }; continue;
}
// C line 19821
510 => {
let _ = { let assigned = *(sp).offset((((2 as i32)).wrapping_neg()) as isize); op1_7 = assigned; assigned };
// C line 19822
let _ = { let assigned = *(sp).offset((((1 as i32)).wrapping_neg()) as isize); op2_2 = assigned; assigned };
vm_block = 508; continue;
}
// C line 19817
511 => {
vm_block = 30; continue;
}
// C line 19787
512 => {
let _ = { let old = sp; sp = (sp).offset(-1); old };
vm_block = 511; continue;
}
// C line 19783
513 => {
let _ = { let assigned = __JS_NewFloat64(ctx, ((r_1) as f64)); *(pv) = assigned; assigned };
vm_block = 512; continue;
}
// C line 19785
514 => {
let _ = { let assigned = JS_NewInt32(ctx, ((r_1) as i32)); *(pv) = assigned; assigned };
vm_block = 512; continue;
}
// C line 19782
515 => {
vm_block = if ((((!(((!(((((((((r_1) as i32)) as i64)) != (r_1)) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 513 } else { 514 }; continue;
}
// C line 19781
516 => {
let _ = { let assigned = (((((((*(pv)).u).uint64) as i32)) as i64)).wrapping_add(((((((op2_1).u).uint64) as i32)) as i64)); r_1 = assigned; assigned };
vm_block = 515; continue;
}
// C line 19789
518 => {
let _ = { let assigned = __JS_NewFloat64(ctx, ((((*(pv)).u).float64) + (((op2_1).u).float64))); *(pv) = assigned; assigned };
// C line 19791
let _ = { let old = sp; sp = (sp).offset(-1); old };
vm_block = 511; continue;
}
// C line 19797
519 => {
let _ = JS_FreeValue(ctx, op2_1);
vm_block = 511; continue;
}
// C line 19802
520 => {
let _ = set_value(ctx, pv, op2_1);
vm_block = 511; continue;
}
// C line 19801
521 => {
vm_block = 29; continue;
}
// C line 19800
522 => {
vm_block = if (JS_IsException(op2_1)) != 0 { 521 } else { 520 }; continue;
}
// C line 19799
523 => {
let _ = { let assigned = JS_ConcatString(ctx, JS_DupValue(ctx, *(pv)), op2_1); op2_1 = assigned; assigned };
vm_block = 522; continue;
}
// C line 19796
524 => {
vm_block = if (JS_ConcatStringInPlace(ctx, ((((*(pv)).u).ptr) as *mut JSString), op2_1)) != 0 { 519 } else { 523 }; continue;
}
// C line 19794
526 => {
let _ = { let old = sp; sp = (sp).offset(-1); old };
// C line 19795
let _ = { let assigned = pc; (*(sf)).cur_pc = assigned; assigned };
vm_block = 524; continue;
}
// C line 19814
527 => {
let _ = set_value(ctx, pv, *((ops).as_mut_ptr()).offset(((0 as i32)) as isize));
vm_block = 511; continue;
}
// C line 19813
528 => {
vm_block = 29; continue;
}
// C line 19812
529 => {
vm_block = if (js_add_slow(ctx, ((ops).as_mut_ptr()).offset((((2 as i32)) as isize)))) != 0 { 528 } else { 527 }; continue;
}
// C line 19808
533 => {
let _ = { let assigned = pc; (*(sf)).cur_pc = assigned; assigned };
// C line 19809
let _ = { let assigned = JS_DupValue(ctx, *(pv)); *((ops).as_mut_ptr()).offset(((0 as i32)) as isize) = assigned; assigned };
// C line 19810
let _ = { let assigned = op2_1; *((ops).as_mut_ptr()).offset(((1 as i32)) as isize) = assigned; assigned };
// C line 19811
let _ = { let old = sp; sp = (sp).offset(-1); old };
vm_block = 529; continue;
}
// C line 19792
534 => {
vm_block = if (((((((((((*(pv)).tag) as i32)) == ((JS_TAG_STRING as i32))) as i32)) != 0) && ((((((((op2_1).tag) as i32)) == ((JS_TAG_STRING as i32))) as i32)) != 0)) as i32)) != 0 { 526 } else { 533 }; continue;
}
// C line 19788
535 => {
vm_block = if (((((((((((((*(pv)).tag) as i32)) as u32)) == ((((JS_TAG_FLOAT64 as i32)) as u32))) as i32)) != 0) && ((((((((((op2_1).tag) as i32)) as u32)) == ((((JS_TAG_FLOAT64 as i32)) as u32))) as i32)) != 0)) as i32)) != 0 { 518 } else { 534 }; continue;
}
// C line 19779
536 => {
vm_block = if ((((!(((!((((((((((*(pv)).tag) as i32)) | ((((op2_1).tag) as i32)))) == ((0 as i32))) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 516 } else { 535 }; continue;
}
// C line 19774
540 => {
let _ = { let assigned = ((*(pc)) as i32); idx_26 = assigned; assigned };
// C line 19775
let _ = { pc = (pc).offset((((1 as i32)) as isize)); pc };
// C line 19777
let _ = { let assigned = *(sp).offset((((1 as i32)).wrapping_neg()) as isize); op2_1 = assigned; assigned };
// C line 19778
let _ = { let assigned = core::ptr::addr_of_mut!(*(var_buf).offset((idx_26) as isize)); pv = assigned; assigned };
vm_block = 536; continue;
}
// C line 19768
541 => {
vm_block = 30; continue;
}
// C line 19735
542 => {
let _ = { let old = sp; sp = (sp).offset(-1); old };
vm_block = 541; continue;
}
// C line 19731
543 => {
let _ = { let assigned = __JS_NewFloat64(ctx, ((r) as f64)); *(sp).offset((((2 as i32)).wrapping_neg()) as isize) = assigned; assigned };
vm_block = 542; continue;
}
// C line 19733
544 => {
let _ = { let assigned = JS_NewInt32(ctx, ((r) as i32)); *(sp).offset((((2 as i32)).wrapping_neg()) as isize) = assigned; assigned };
vm_block = 542; continue;
}
// C line 19730
545 => {
vm_block = if ((((!(((!(((((((((r) as i32)) as i64)) != (r)) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 543 } else { 544 }; continue;
}
// C line 19729
546 => {
let _ = { let assigned = (((((((op1_6).u).uint64) as i32)) as i64)).wrapping_add(((((((op2).u).uint64) as i32)) as i64)); r = assigned; assigned };
vm_block = 545; continue;
}
// C line 19753
548 => {
let _ = { let assigned = __JS_NewFloat64(ctx, ((d1) + (d2))); *(sp).offset((((2 as i32)).wrapping_neg()) as isize) = assigned; assigned };
// C line 19754
let _ = { let old = sp; sp = (sp).offset(-1); old };
vm_block = 541; continue;
}
// C line 19747
549 => {
let _ = { let assigned = ((op2).u).float64; d2 = assigned; assigned };
vm_block = 548; continue;
}
// C line 19749
550 => {
let _ = { let assigned = ((((((op2).u).uint64) as i32)) as f64); d2 = assigned; assigned };
vm_block = 548; continue;
}
// C line 19751
551 => {
vm_block = 566; continue;
}
// C line 19748
552 => {
vm_block = if (((((((op2).tag) as i32)) == ((JS_TAG_INT as i32))) as i32)) != 0 { 550 } else { 551 }; continue;
}
// C line 19746
553 => {
vm_block = if (((((((((op2).tag) as i32)) as u32)) == ((((JS_TAG_FLOAT64 as i32)) as u32))) as i32)) != 0 { 549 } else { 552 }; continue;
}
// C line 19740
554 => {
let _ = { let assigned = ((op1_6).u).float64; d1 = assigned; assigned };
vm_block = 553; continue;
}
// C line 19742
555 => {
let _ = { let assigned = ((((((op1_6).u).uint64) as i32)) as f64); d1 = assigned; assigned };
vm_block = 553; continue;
}
// C line 19744
556 => {
vm_block = 566; continue;
}
// C line 19741
557 => {
vm_block = if (((((((op1_6).tag) as i32)) == ((JS_TAG_INT as i32))) as i32)) != 0 { 555 } else { 556 }; continue;
}
// C line 19739
558 => {
vm_block = if (((((((((op1_6).tag) as i32)) as u32)) == ((((JS_TAG_FLOAT64 as i32)) as u32))) as i32)) != 0 { 554 } else { 557 }; continue;
}
// C line 19759
559 => {
vm_block = 29; continue;
}
// C line 19758
560 => {
vm_block = if (JS_IsException(*(sp).offset((((1 as i32)).wrapping_neg()) as isize))) != 0 { 559 } else { 541 }; continue;
}
// C line 19756
562 => {
let _ = { let assigned = JS_ConcatString(ctx, op1_6, op2); *(sp).offset((((2 as i32)).wrapping_neg()) as isize) = assigned; assigned };
// C line 19757
let _ = { let old = sp; sp = (sp).offset(-1); old };
vm_block = 560; continue;
}
// C line 19765
563 => {
let _ = { let old = sp; sp = (sp).offset(-1); old };
vm_block = 541; continue;
}
// C line 19764
564 => {
vm_block = 29; continue;
}
// C line 19763
565 => {
vm_block = if (js_add_slow(ctx, sp)) != 0 { 564 } else { 563 }; continue;
}
// C line ? labels: add_slow_case
566 => {
let _ = { let assigned = pc; (*(sf)).cur_pc = assigned; assigned };
vm_block = 565; continue;
}
// C line 19755
567 => {
vm_block = if (((((JS_IsString(op1_6)) != 0) && ((JS_IsString(op2)) != 0)) as i32)) != 0 { 562 } else { 566 }; continue;
}
// C line 19736
568 => {
vm_block = if (((((((((((((op1_6).tag) as i32)) as u32)) == ((((JS_TAG_FLOAT64 as i32)) as u32))) as i32)) != 0) || ((((((((((op2).tag) as i32)) as u32)) == ((((JS_TAG_FLOAT64 as i32)) as u32))) as i32)) != 0)) as i32)) != 0 { 558 } else { 567 }; continue;
}
// C line 19727
569 => {
vm_block = if ((((!(((!((((((((((op1_6).tag) as i32)) | ((((op2).tag) as i32)))) == ((0 as i32))) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 546 } else { 568 }; continue;
}
// C line 19725
571 => {
let _ = { let assigned = *(sp).offset((((2 as i32)).wrapping_neg()) as isize); op1_6 = assigned; assigned };
// C line 19726
let _ = { let assigned = *(sp).offset((((1 as i32)).wrapping_neg()) as isize); op2 = assigned; assigned };
vm_block = 569; continue;
}
// C line 19720
572 => {
vm_block = 30; continue;
}
// C line 19718
573 => {
vm_block = 29; continue;
}
// C line 19715
574 => {
vm_block = if (JS_CopyDataProperties(ctx, *(sp).offset(((((1 as i32)).wrapping_neg()).wrapping_sub(((mask) & ((3 as i32))))) as isize), *(sp).offset(((((1 as i32)).wrapping_neg()).wrapping_sub((((mask).wrapping_shr(((2 as i32)) as u32)) & ((7 as i32))))) as isize), *(sp).offset(((((1 as i32)).wrapping_neg()).wrapping_sub((((mask).wrapping_shr(((5 as i32)) as u32)) & ((7 as i32))))) as isize), (0 as i32))) != 0 { 573 } else { 572 }; continue;
}
// C line 19713
576 => {
let _ = { let assigned = ((*({ let old = pc; pc = (pc).offset(1); old })) as i32); mask = assigned; assigned };
// C line 19714
let _ = { let assigned = pc; (*(sf)).cur_pc = assigned; assigned };
vm_block = 574; continue;
}
// C line 19701
578 => {
let _ = JS_FreeValue(ctx, *({ sp = (sp).offset(-1); sp }));
// C line 19703
vm_block = 30; continue;
}
// C line 19700
579 => {
vm_block = 29; continue;
}
// C line 19699
580 => {
vm_block = if (js_append_enumerate(ctx, sp)) != 0 { 579 } else { 578 }; continue;
}
// C line 19698
581 => {
let _ = { let assigned = pc; (*(sf)).cur_pc = assigned; assigned };
vm_block = 580; continue;
}
// C line 19694
582 => {
vm_block = 30; continue;
}
// C line 19692
583 => {
vm_block = 29; continue;
}
// C line 19691
584 => {
vm_block = if ((((!(((!(((((ret_16) < ((0 as i32))) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 583 } else { 582 }; continue;
}
// C line 19688
586 => {
let _ = { let assigned = JS_DefinePropertyValueValue(ctx, *(sp).offset((((3 as i32)).wrapping_neg()) as isize), JS_DupValue(ctx, *(sp).offset((((2 as i32)).wrapping_neg()) as isize)), *(sp).offset((((1 as i32)).wrapping_neg()) as isize), ((((((((1 as i32)).wrapping_shl(((0 as i32)) as u32)) | (((1 as i32)).wrapping_shl(((1 as i32)) as u32)))) | (((1 as i32)).wrapping_shl(((2 as i32)) as u32)))) | (((1 as i32)).wrapping_shl(((14 as i32)) as u32)))); ret_16 = assigned; assigned };
// C line 19690
let _ = { sp = (sp).offset(-(((1 as i32)) as isize)); sp };
vm_block = 584; continue;
}
// C line 19683
587 => {
vm_block = 30; continue;
}
// C line 19681
588 => {
vm_block = 29; continue;
}
// C line 19680
589 => {
vm_block = if ((((ret_15) < ((0 as i32))) as i32)) != 0 { 588 } else { 587 }; continue;
}
// C line 19673
595 => {
let _ = { let assigned = JS_SetPropertyInternal(ctx, *(sp).offset((((3 as i32)).wrapping_neg()) as isize), atom_15, *(sp).offset((((1 as i32)).wrapping_neg()) as isize), *(sp).offset((((4 as i32)).wrapping_neg()) as isize), ((1 as i32)).wrapping_shl(((15 as i32)) as u32)); ret_15 = assigned; assigned };
// C line 19675
let _ = JS_FreeAtom(ctx, atom_15);
// C line 19676
let _ = JS_FreeValue(ctx, *(sp).offset((((4 as i32)).wrapping_neg()) as isize));
// C line 19677
let _ = JS_FreeValue(ctx, *(sp).offset((((3 as i32)).wrapping_neg()) as isize));
// C line 19678
let _ = JS_FreeValue(ctx, *(sp).offset((((2 as i32)).wrapping_neg()) as isize));
// C line 19679
let _ = { sp = (sp).offset(-(((4 as i32)) as isize)); sp };
vm_block = 589; continue;
}
// C line 19672
596 => {
vm_block = 29; continue;
}
// C line 19671
597 => {
vm_block = if ((((!(((!(((((atom_15) == ((((0 as i32)) as JSAtom))) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 596 } else { 595 }; continue;
}
// C line 19670
598 => {
let _ = { let assigned = JS_ValueToAtom(ctx, *(sp).offset((((2 as i32)).wrapping_neg()) as isize)); atom_15 = assigned; assigned };
vm_block = 597; continue;
}
// C line 19667
600 => {
let _ = JS_ThrowTypeErrorNotAnObject(ctx);
// C line 19668
vm_block = 29; continue;
}
// C line 19666
601 => {
vm_block = if (((((((*(sp).offset((((3 as i32)).wrapping_neg()) as isize)).tag) as i32)) != ((JS_TAG_OBJECT as i32))) as i32)) != 0 { 600 } else { 598 }; continue;
}
// C line 19665
602 => {
let _ = { let assigned = pc; (*(sf)).cur_pc = assigned; assigned };
vm_block = 601; continue;
}
// C line 19659
603 => {
vm_block = 30; continue;
}
// C line 19657
604 => {
vm_block = 29; continue;
}
// C line 19656
605 => {
vm_block = if ((((!(((!(((((ret_14) < ((0 as i32))) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 604 } else { 603 }; continue;
}
// C line 19651
610 => {
let _ = { let assigned = JS_SetPropertyInternal(ctx, *(sp).offset((((3 as i32)).wrapping_neg()) as isize), atom_14, *(sp).offset((((1 as i32)).wrapping_neg()) as isize), *(sp).offset((((3 as i32)).wrapping_neg()) as isize), ((1 as i32)).wrapping_shl(((15 as i32)) as u32)); ret_14 = assigned; assigned };
// C line 19652
let _ = JS_FreeAtom(ctx, atom_14);
// C line 19653
let _ = JS_FreeValue(ctx, *(sp).offset((((2 as i32)).wrapping_neg()) as isize));
// C line 19654
let _ = JS_FreeValue(ctx, *(sp).offset((((3 as i32)).wrapping_neg()) as isize));
// C line 19655
let _ = { sp = (sp).offset(-(((3 as i32)) as isize)); sp };
vm_block = 605; continue;
}
// C line 19646
613 => {
let _ = JS_ThrowReferenceErrorNotDefined(ctx, atom_14);
// C line 19647
let _ = JS_FreeAtom(ctx, atom_14);
// C line 19648
vm_block = 29; continue;
}
// C line 19645
614 => {
vm_block = if (is_strict_mode(ctx)) != 0 { 613 } else { 610 }; continue;
}
// C line 19642
616 => {
let _ = JS_FreeAtom(ctx, atom_14);
// C line 19643
vm_block = 29; continue;
}
// C line 19641
617 => {
vm_block = if ((((!(((!(((((ret_14) < ((0 as i32))) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 616 } else { 614 }; continue;
}
// C line 19640
618 => {
vm_block = if ((((!(((!(((((ret_14) <= ((0 as i32))) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 617 } else { 610 }; continue;
}
// C line 19639
619 => {
let _ = { let assigned = JS_HasProperty(ctx, *(sp).offset((((3 as i32)).wrapping_neg()) as isize), atom_14); ret_14 = assigned; assigned };
vm_block = 618; continue;
}
// C line 19632
622 => {
let _ = JS_ThrowReferenceErrorNotDefined(ctx, atom_14);
// C line 19633
let _ = JS_FreeAtom(ctx, atom_14);
// C line 19634
vm_block = 29; continue;
}
// C line 19636
623 => {
let _ = { let assigned = JS_DupValue(ctx, (*(ctx)).global_obj); *(sp).offset((((3 as i32)).wrapping_neg()) as isize) = assigned; assigned };
vm_block = 619; continue;
}
// C line 19631
624 => {
vm_block = if (is_strict_mode(ctx)) != 0 { 622 } else { 623 }; continue;
}
// C line 19630
625 => {
vm_block = if ((((!(((!((JS_IsUndefined(*(sp).offset((((3 as i32)).wrapping_neg()) as isize))) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 624 } else { 619 }; continue;
}
// C line 19629
626 => {
vm_block = 29; continue;
}
// C line 19628
627 => {
vm_block = if ((((!(((!(((((atom_14) == ((((0 as i32)) as JSAtom))) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 626 } else { 625 }; continue;
}
// C line 19626
629 => {
let _ = { let assigned = pc; (*(sf)).cur_pc = assigned; assigned };
// C line 19627
let _ = { let assigned = JS_ValueToAtom(ctx, *(sp).offset((((2 as i32)).wrapping_neg()) as isize)); atom_14 = assigned; assigned };
vm_block = 627; continue;
}
// C line 19620
630 => {
vm_block = 30; continue;
}
// C line 19608
632 => {
let _ = JS_FreeValue(ctx, *(sp).offset((((3 as i32)).wrapping_neg()) as isize));
// C line 19609
let _ = { sp = (sp).offset(-(((3 as i32)) as isize)); sp };
vm_block = 630; continue;
}
// C line 19603
634 => {
let _ = { let assigned = new_len; (((*(p_8)).u).array).count = assigned; assigned };
// C line 19604
let _ = { let assigned = *(sp).offset((((1 as i32)).wrapping_neg()) as isize); *(((((*(p_8)).u).array).u).values).offset((idx_25) as isize) = assigned; assigned };
vm_block = 632; continue;
}
// C line 19601
635 => {
let _ = { let assigned = JS_NewInt32(ctx, ((new_len) as i32)); ((*((*(p_8)).prop).offset(((0 as i32)) as isize)).u).value = assigned; assigned };
vm_block = 634; continue;
}
// C line 19600
636 => {
vm_block = 658; continue;
}
// C line 19599
637 => {
vm_block = if ((((!(((!(((!(((((((*(get_shape_prop((*(p_8)).shape))).flags()) as i32)) & (((1 as i32)).wrapping_shl(((1 as i32)) as u32)))) != 0) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 636 } else { 635 }; continue;
}
// C line 19598
638 => {
vm_block = if ((((new_len) > (array_len)) as i32)) != 0 { 637 } else { 634 }; continue;
}
// C line 19597
639 => {
let _ = { let assigned = ((((((((*((*(p_8)).prop).offset(((0 as i32)) as isize)).u).value).u).uint64) as i32)) as u32); array_len = assigned; assigned };
vm_block = 638; continue;
}
// C line 19596
640 => {
vm_block = 658; continue;
}
// C line 19595
641 => {
vm_block = if ((((!(((!(((((new_len) > (((((*(p_8)).u).array).u1).size)) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 640 } else { 639 }; continue;
}
// C line 19594
642 => {
let _ = { let assigned = (idx_25).wrapping_add((((1 as i32)) as u32)); new_len = assigned; assigned };
vm_block = 641; continue;
}
// C line 19592
643 => {
vm_block = 658; continue;
}
// C line 19591
644 => {
vm_block = if ((((!(((!((((((((((*((*(p_8)).prop).offset(((0 as i32)) as isize)).u).value).tag) as i32)) != ((JS_TAG_INT as i32))) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 643 } else { 642 }; continue;
}
// C line 19589
645 => {
vm_block = 658; continue;
}
// C line 19586
646 => {
vm_block = if ((((!(((!(((((((((((((idx_25) != ((((*(p_8)).u).array).count)) as i32)) != 0) || (((!(((*(p_8)).fast_array()) != 0) as i32)) != 0)) as i32)) != 0) || (((!((can_extend_fast_array(p_8)) != 0) as i32)) != 0)) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 645 } else { 644 }; continue;
}
// C line 19606
647 => {
let _ = set_value(ctx, core::ptr::addr_of_mut!(*(((((*(p_8)).u).array).u).values).offset((idx_25) as isize)), *(sp).offset((((1 as i32)).wrapping_neg()) as isize));
vm_block = 632; continue;
}
// C line 19584
648 => {
vm_block = if ((((!(((!(((((idx_25) >= ((((*(p_8)).u).array).count)) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 646 } else { 647 }; continue;
}
// C line 19583
649 => {
vm_block = 658; continue;
}
// C line 19582
650 => {
vm_block = if ((((!(((!((((((((*(p_8)).class_id) as i32)) != ((JS_CLASS_ARRAY as i32))) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 649 } else { 648 }; continue;
}
// C line 19580
652 => {
let _ = { let assigned = ((((*(sp).offset((((3 as i32)).wrapping_neg()) as isize)).u).ptr) as *mut JSObject); p_8 = assigned; assigned };
// C line 19581
let _ = { let assigned = ((((((*(sp).offset((((2 as i32)).wrapping_neg()) as isize)).u).uint64) as i32)) as u32); idx_25 = assigned; assigned };
vm_block = 650; continue;
}
// C line 19617
653 => {
vm_block = 29; continue;
}
// C line 19616
654 => {
vm_block = if ((((!(((!(((((ret_13) < ((0 as i32))) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 653 } else { 630 }; continue;
}
// C line ? labels: put_array_el_slow_path
658 => {
let _ = { let assigned = pc; (*(sf)).cur_pc = assigned; assigned };
// C line 19613
let _ = { let assigned = JS_SetPropertyValue(ctx, *(sp).offset((((3 as i32)).wrapping_neg()) as isize), *(sp).offset((((2 as i32)).wrapping_neg()) as isize), *(sp).offset((((1 as i32)).wrapping_neg()) as isize), ((1 as i32)).wrapping_shl(((15 as i32)) as u32)); ret_13 = assigned; assigned };
// C line 19614
let _ = JS_FreeValue(ctx, *(sp).offset((((3 as i32)).wrapping_neg()) as isize));
// C line 19615
let _ = { sp = (sp).offset(-(((3 as i32)) as isize)); sp };
vm_block = 654; continue;
}
// C line 19578
659 => {
vm_block = if ((((!(((!((((((((((((*(sp).offset((((3 as i32)).wrapping_neg()) as isize)).tag) as i32)) == ((JS_TAG_OBJECT as i32))) as i32)) != 0) && ((((((((*(sp).offset((((2 as i32)).wrapping_neg()) as isize)).tag) as i32)) == ((JS_TAG_INT as i32))) as i32)) != 0)) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 652 } else { 658 }; continue;
}
// C line 19564
665 => {
let _ = JS_FreeValue(ctx, *(sp).offset((((1 as i32)).wrapping_neg()) as isize));
// C line 19565
let _ = JS_FreeValue(ctx, *(sp).offset((((2 as i32)).wrapping_neg()) as isize));
// C line 19566
let _ = JS_FreeValue(ctx, *(sp).offset((((3 as i32)).wrapping_neg()) as isize));
// C line 19567
let _ = { let assigned = val_14; *(sp).offset((((3 as i32)).wrapping_neg()) as isize) = assigned; assigned };
// C line 19568
let _ = { sp = (sp).offset(-(((2 as i32)) as isize)); sp };
// C line 19570
vm_block = 30; continue;
}
// C line 19563
666 => {
vm_block = 29; continue;
}
// C line 19562
667 => {
vm_block = if ((((!(((!((JS_IsException(val_14)) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 666 } else { 665 }; continue;
}
// C line 19560
669 => {
let _ = { let assigned = JS_GetPropertyInternal(ctx, *(sp).offset((((2 as i32)).wrapping_neg()) as isize), atom_13, *(sp).offset((((3 as i32)).wrapping_neg()) as isize), (0 as i32)); val_14 = assigned; assigned };
// C line 19561
let _ = JS_FreeAtom(ctx, atom_13);
vm_block = 667; continue;
}
// C line 19559
670 => {
vm_block = 29; continue;
}
// C line 19558
671 => {
vm_block = if ((((!(((!(((((atom_13) == ((((0 as i32)) as JSAtom))) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 670 } else { 669 }; continue;
}
// C line 19556
673 => {
let _ = { let assigned = pc; (*(sf)).cur_pc = assigned; assigned };
// C line 19557
let _ = { let assigned = JS_ValueToAtom(ctx, *(sp).offset((((1 as i32)).wrapping_neg()) as isize)); atom_13 = assigned; assigned };
vm_block = 671; continue;
}
// C line 19547
676 => {
let _ = { let assigned = val_13; *(sp).offset(((0 as i32)) as isize) = assigned; assigned };
// C line 19548
let _ = { let old = sp; sp = (sp).offset(1); old };
// C line 19550
vm_block = 30; continue;
}
// C line 19546
677 => {
vm_block = 29; continue;
}
// C line 19545
678 => {
vm_block = if ((((!(((!((JS_IsException(val_13)) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 677 } else { 676 }; continue;
}
// C line 19544
679 => {
let _ = JS_FreeAtom(ctx, atom_12);
vm_block = 678; continue;
}
// C line 19540
680 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; val_13 = assigned; assigned };
vm_block = 679; continue;
}
// C line 19536
683 => {
let _ = JS_ThrowReferenceErrorNotDefined(ctx, atom_12);
// C line 19537
let _ = JS_FreeAtom(ctx, atom_12);
// C line 19538
vm_block = 29; continue;
}
// C line 19535
684 => {
vm_block = if (is_strict_mode(ctx)) != 0 { 683 } else { 680 }; continue;
}
// C line 19532
686 => {
let _ = JS_FreeAtom(ctx, atom_12);
// C line 19533
vm_block = 29; continue;
}
// C line 19531
687 => {
vm_block = if ((((ret_12) < ((0 as i32))) as i32)) != 0 { 686 } else { 684 }; continue;
}
// C line 19542
688 => {
let _ = { let assigned = JS_GetProperty(ctx, *(sp).offset((((2 as i32)).wrapping_neg()) as isize), atom_12); val_13 = assigned; assigned };
vm_block = 679; continue;
}
// C line 19530
689 => {
vm_block = if ((((!(((!(((((ret_12) <= ((0 as i32))) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 687 } else { 688 }; continue;
}
// C line 19529
690 => {
let _ = { let assigned = JS_HasProperty(ctx, *(sp).offset((((2 as i32)).wrapping_neg()) as isize), atom_12); ret_12 = assigned; assigned };
vm_block = 689; continue;
}
// C line 19525
693 => {
let _ = JS_ThrowReferenceErrorNotDefined(ctx, atom_12);
// C line 19526
let _ = JS_FreeAtom(ctx, atom_12);
// C line 19527
vm_block = 29; continue;
}
// C line 19524
694 => {
vm_block = if ((((!(((!((JS_IsUndefined(*(sp).offset((((2 as i32)).wrapping_neg()) as isize))) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 693 } else { 690 }; continue;
}
// C line 19523
695 => {
vm_block = 29; continue;
}
// C line 19522
696 => {
vm_block = if ((((atom_12) == ((((0 as i32)) as JSAtom))) as i32)) != 0 { 695 } else { 694 }; continue;
}
// C line 19520
698 => {
let _ = { let assigned = pc; (*(sf)).cur_pc = assigned; assigned };
// C line 19521
let _ = { let assigned = JS_ValueToAtom(ctx, *(sp).offset((((1 as i32)).wrapping_neg()) as isize)); atom_12 = assigned; assigned };
vm_block = 696; continue;
}
// C line 19510
700 => {
let _ = { let assigned = val_12; *({ let old = sp; sp = (sp).offset(1); old }) = assigned; assigned };
// C line 19512
vm_block = 30; continue;
}
// C line 19482
701 => {
let _ = { let assigned = JS_DupValue(ctx, *(((((*(p_7)).u).array).u).values).offset((idx_24) as isize)); val_12 = assigned; assigned };
vm_block = 700; continue;
}
// C line 19481
702 => {
vm_block = 723; continue;
}
// C line 19480
703 => {
vm_block = if ((((!(((!(((((idx_24) >= ((((*(p_7)).u).array).count)) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 702 } else { 701 }; continue;
}
// C line 19479
704 => {
vm_block = 723; continue;
}
// C line 19478
705 => {
vm_block = if ((((!(((!((((((((*(p_7)).class_id) as i32)) != ((JS_CLASS_ARRAY as i32))) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 704 } else { 703 }; continue;
}
// C line 19476
707 => {
let _ = { let assigned = ((((*(sp).offset((((2 as i32)).wrapping_neg()) as isize)).u).ptr) as *mut JSObject); p_7 = assigned; assigned };
// C line 19477
let _ = { let assigned = ((((((*(sp).offset((((1 as i32)).wrapping_neg()) as isize)).u).uint64) as i32)) as u32); idx_24 = assigned; assigned };
vm_block = 705; continue;
}
// C line 19508
708 => {
vm_block = 29; continue;
}
// C line 19507
709 => {
vm_block = if ((((!(((!((JS_IsException(val_12)) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 708 } else { 700 }; continue;
}
// C line 19505
711 => {
let _ = { let assigned = pc; (*(sf)).cur_pc = assigned; assigned };
// C line 19506
let _ = { let assigned = JS_GetPropertyValue(ctx, *(sp).offset((((2 as i32)).wrapping_neg()) as isize), JS_DupValue(ctx, *(sp).offset((((1 as i32)).wrapping_neg()) as isize))); val_12 = assigned; assigned };
vm_block = 709; continue;
}
// C line 19501
714 => {
let _ = JS_FreeValue(ctx, *(sp).offset((((1 as i32)).wrapping_neg()) as isize));
// C line 19502
let _ = { let assigned = ret_val; *(sp).offset((((1 as i32)).wrapping_neg()) as isize) = assigned; assigned };
// C line 19503
vm_block = 711; continue;
}
// C line 19500
715 => {
vm_block = 29; continue;
}
// C line 19499
716 => {
vm_block = if (JS_IsException(ret_val)) != 0 { 715 } else { 714 }; continue;
}
// C line 19497
718 => {
let _ = { let assigned = pc; (*(sf)).cur_pc = assigned; assigned };
// C line 19498
let _ = { let assigned = JS_ToPropertyKey(ctx, *(sp).offset((((1 as i32)).wrapping_neg()) as isize)); ret_val = assigned; assigned };
vm_block = 716; continue;
}
// C line 19494
720 => {
let _ = JS_ThrowTypeError(ctx, c"value has no property".as_ptr());
// C line 19495
vm_block = 29; continue;
}
// C line 19493
721 => {
vm_block = if ((((!(((!((((((JS_IsUndefined(*(sp).offset((((2 as i32)).wrapping_neg()) as isize))) != 0) || ((JS_IsNull(*(sp).offset((((2 as i32)).wrapping_neg()) as isize))) != 0)) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 720 } else { 718 }; continue;
}
// C line 19490
722 => {
vm_block = 711; continue;
}
// C line 19485 labels: get_array_el3_slow_path
723 => {
vm_block = match (((*(sp).offset((((1 as i32)).wrapping_neg()) as isize)).tag) as i32) { x if x == (JS_TAG_SYMBOL as i32) => 722, x if x == (JS_TAG_STRING as i32) => 722, x if x == (JS_TAG_INT as i32) => 722, _ => 721, }; continue;
}
// C line 19474
724 => {
vm_block = if ((((!(((!((((((((((((*(sp).offset((((2 as i32)).wrapping_neg()) as isize)).tag) as i32)) == ((JS_TAG_OBJECT as i32))) as i32)) != 0) && ((((((((*(sp).offset((((1 as i32)).wrapping_neg()) as isize)).tag) as i32)) == ((JS_TAG_INT as i32))) as i32)) != 0)) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 707 } else { 723 }; continue;
}
// C line 19466
725 => {
vm_block = 30; continue;
}
// C line 19465
726 => {
let _ = { let assigned = val_11; *(sp).offset((((1 as i32)).wrapping_neg()) as isize) = assigned; assigned };
vm_block = 725; continue;
}
// C line 19465
729 => {
let _ = JS_FreeValue(ctx, obj_8);
// C line 19465
let _ = { let assigned = val_11; *(sp).offset((((2 as i32)).wrapping_neg()) as isize) = assigned; assigned };
// C line 19465
let _ = { let old = sp; sp = (sp).offset(-1); old };
vm_block = 725; continue;
}
// C line 19465
730 => {
vm_block = if ((1 as i32)) != 0 { 726 } else { 729 }; continue;
}
// C line 19465
731 => {
let _ = { let assigned = JS_DupValue(ctx, *(((((*(p_6)).u).array).u).values).offset((idx_23) as isize)); val_11 = assigned; assigned };
vm_block = 730; continue;
}
// C line 19465
732 => {
vm_block = 744; continue;
}
// C line 19465
733 => {
vm_block = if ((((!(((!(((((idx_23) >= ((((*(p_6)).u).array).count)) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 732 } else { 731 }; continue;
}
// C line 19465
734 => {
vm_block = 744; continue;
}
// C line 19465
735 => {
vm_block = if ((((!(((!((((((((*(p_6)).class_id) as i32)) != ((JS_CLASS_ARRAY as i32))) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 734 } else { 733 }; continue;
}
// C line 19465
737 => {
let _ = { let assigned = ((((obj_8).u).ptr) as *mut JSObject); p_6 = assigned; assigned };
// C line 19465
let _ = { let assigned = ((((((prop_1).u).uint64) as i32)) as u32); idx_23 = assigned; assigned };
vm_block = 735; continue;
}
// C line 19465
738 => {
vm_block = 29; continue;
}
// C line 19465
739 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; *(sp).offset((((1 as i32)).wrapping_neg()) as isize) = assigned; assigned };
vm_block = 738; continue;
}
// C line 19465
740 => {
let _ = { let old = sp; sp = (sp).offset(-1); old };
vm_block = 738; continue;
}
// C line 19465
741 => {
vm_block = if ((1 as i32)) != 0 { 739 } else { 740 }; continue;
}
// C line 19465
742 => {
vm_block = if ((((!(((!((JS_IsException(val_11)) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 741 } else { 730 }; continue;
}
// C line 19465 labels: get_array_el2_slow_path
744 => {
let _ = { let assigned = pc; (*(sf)).cur_pc = assigned; assigned };
// C line 19465
let _ = { let assigned = JS_GetPropertyValue(ctx, obj_8, prop_1); val_11 = assigned; assigned };
vm_block = 742; continue;
}
// C line 19465
745 => {
vm_block = if ((((!(((!((((((((((((obj_8).tag) as i32)) == ((JS_TAG_OBJECT as i32))) as i32)) != 0) && ((((((((prop_1).tag) as i32)) == ((JS_TAG_INT as i32))) as i32)) != 0)) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 737 } else { 744 }; continue;
}
// C line 19465
747 => {
let _ = { let assigned = *(sp).offset((((2 as i32)).wrapping_neg()) as isize); obj_8 = assigned; assigned };
// C line 19465
let _ = { let assigned = *(sp).offset((((1 as i32)).wrapping_neg()) as isize); prop_1 = assigned; assigned };
vm_block = 745; continue;
}
// C line 19462
748 => {
vm_block = 30; continue;
}
// C line 19461
749 => {
let _ = { let assigned = val_10; *(sp).offset((((1 as i32)).wrapping_neg()) as isize) = assigned; assigned };
vm_block = 748; continue;
}
// C line 19461
752 => {
let _ = JS_FreeValue(ctx, obj_7);
// C line 19461
let _ = { let assigned = val_10; *(sp).offset((((2 as i32)).wrapping_neg()) as isize) = assigned; assigned };
// C line 19461
let _ = { let old = sp; sp = (sp).offset(-1); old };
vm_block = 748; continue;
}
// C line 19461
753 => {
vm_block = if ((0 as i32)) != 0 { 749 } else { 752 }; continue;
}
// C line 19461
754 => {
let _ = { let assigned = JS_DupValue(ctx, *(((((*(p_5)).u).array).u).values).offset((idx_22) as isize)); val_10 = assigned; assigned };
vm_block = 753; continue;
}
// C line 19461
755 => {
vm_block = 767; continue;
}
// C line 19461
756 => {
vm_block = if ((((!(((!(((((idx_22) >= ((((*(p_5)).u).array).count)) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 755 } else { 754 }; continue;
}
// C line 19461
757 => {
vm_block = 767; continue;
}
// C line 19461
758 => {
vm_block = if ((((!(((!((((((((*(p_5)).class_id) as i32)) != ((JS_CLASS_ARRAY as i32))) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 757 } else { 756 }; continue;
}
// C line 19461
760 => {
let _ = { let assigned = ((((obj_7).u).ptr) as *mut JSObject); p_5 = assigned; assigned };
// C line 19461
let _ = { let assigned = ((((((prop).u).uint64) as i32)) as u32); idx_22 = assigned; assigned };
vm_block = 758; continue;
}
// C line 19461
761 => {
vm_block = 29; continue;
}
// C line 19461
762 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; *(sp).offset((((1 as i32)).wrapping_neg()) as isize) = assigned; assigned };
vm_block = 761; continue;
}
// C line 19461
763 => {
let _ = { let old = sp; sp = (sp).offset(-1); old };
vm_block = 761; continue;
}
// C line 19461
764 => {
vm_block = if ((0 as i32)) != 0 { 762 } else { 763 }; continue;
}
// C line 19461
765 => {
vm_block = if ((((!(((!((JS_IsException(val_10)) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 764 } else { 753 }; continue;
}
// C line 19461 labels: get_array_el_slow_path
767 => {
let _ = { let assigned = pc; (*(sf)).cur_pc = assigned; assigned };
// C line 19461
let _ = { let assigned = JS_GetPropertyValue(ctx, obj_7, prop); val_10 = assigned; assigned };
vm_block = 765; continue;
}
// C line 19461
768 => {
vm_block = if ((((!(((!((((((((((((obj_7).tag) as i32)) == ((JS_TAG_OBJECT as i32))) as i32)) != 0) && ((((((((prop).tag) as i32)) == ((JS_TAG_INT as i32))) as i32)) != 0)) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 760 } else { 767 }; continue;
}
// C line 19461
770 => {
let _ = { let assigned = *(sp).offset((((2 as i32)).wrapping_neg()) as isize); obj_7 = assigned; assigned };
// C line 19461
let _ = { let assigned = *(sp).offset((((1 as i32)).wrapping_neg()) as isize); prop = assigned; assigned };
vm_block = 768; continue;
}
// C line 19420
771 => {
vm_block = 30; continue;
}
// C line 19418
772 => {
vm_block = 29; continue;
}
// C line 19415
773 => {
vm_block = if ((((js_op_define_class(ctx, sp, atom_11, class_flags, var_refs, sf, (((opcode) == ((OP_define_class_computed as i32))) as i32))) < ((0 as i32))) as i32)) != 0 { 772 } else { 771 }; continue;
}
// C line 19412
776 => {
let _ = { let assigned = crate::cutils_header::get_u32(pc); atom_11 = assigned; assigned };
// C line 19413
let _ = { let assigned = ((*(pc).offset(((4 as i32)) as isize)) as i32); class_flags = assigned; assigned };
// C line 19414
let _ = { pc = (pc).offset((((5 as i32)) as isize)); pc };
vm_block = 773; continue;
}
// C line 19404
777 => {
vm_block = 30; continue;
}
// C line 19402
778 => {
vm_block = 29; continue;
}
// C line 19401
779 => {
vm_block = if ((((!(((!(((((ret_11) < ((0 as i32))) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 778 } else { 777 }; continue;
}
// C line 19400
780 => {
let _ = { sp = (sp).offset(-((((1 as i32)).wrapping_add(is_computed)) as isize)); sp };
vm_block = 779; continue;
}
// C line 19397
782 => {
let _ = JS_FreeAtom(ctx, atom_10);
// C line 19398
let _ = JS_FreeValue(ctx, *(sp).offset((((2 as i32)).wrapping_neg()) as isize));
vm_block = 780; continue;
}
// C line 19396
783 => {
vm_block = if (is_computed) != 0 { 782 } else { 780 }; continue;
}
// C line 19395
784 => {
let _ = JS_FreeValue(ctx, *(sp).offset((((1 as i32)).wrapping_neg()) as isize));
vm_block = 783; continue;
}
// C line 19392
785 => {
let _ = { let assigned = JS_DefineProperty(ctx, obj_6, atom_10, value, getter, setter, flags_2); ret_11 = assigned; assigned };
vm_block = 784; continue;
}
// C line 19391
786 => {
vm_block = if ((((ret_11) >= ((0 as i32))) as i32)) != 0 { 785 } else { 784 }; continue;
}
// C line 19390
787 => {
let _ = { let assigned = js_method_set_properties(ctx, *(sp).offset((((1 as i32)).wrapping_neg()) as isize), atom_10, flags_2, obj_6); ret_11 = assigned; assigned };
vm_block = 786; continue;
}
// C line 19381
789 => {
let _ = { let assigned = *(sp).offset((((1 as i32)).wrapping_neg()) as isize); value = assigned; assigned };
// C line 19382
let _ = { flags_2 = ((flags_2) | (((((((1 as i32)).wrapping_shl(((13 as i32)) as u32)) | (((1 as i32)).wrapping_shl(((9 as i32)) as u32)))) | (((1 as i32)).wrapping_shl(((1 as i32)) as u32))))); flags_2 };
vm_block = 787; continue;
}
// C line 19384
791 => {
let _ = { let assigned = *(sp).offset((((1 as i32)).wrapping_neg()) as isize); getter = assigned; assigned };
// C line 19385
let _ = { flags_2 = ((flags_2) | (((1 as i32)).wrapping_shl(((11 as i32)) as u32))); flags_2 };
vm_block = 787; continue;
}
// C line 19387
793 => {
let _ = { let assigned = *(sp).offset((((1 as i32)).wrapping_neg()) as isize); setter = assigned; assigned };
// C line 19388
let _ = { flags_2 = ((flags_2) | (((1 as i32)).wrapping_shl(((12 as i32)) as u32))); flags_2 };
vm_block = 787; continue;
}
// C line 19383
794 => {
vm_block = if ((((op_flags) == ((1 as i32))) as i32)) != 0 { 791 } else { 793 }; continue;
}
// C line 19380
795 => {
vm_block = if ((((op_flags) == ((0 as i32))) as i32)) != 0 { 789 } else { 794 }; continue;
}
// C line 19376
799 => {
let _ = { op_flags = ((op_flags) & ((3 as i32))); op_flags };
// C line 19377
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; value = assigned; assigned };
// C line 19378
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; getter = assigned; assigned };
// C line 19379
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; setter = assigned; assigned };
vm_block = 795; continue;
}
// C line 19375
800 => {
let _ = { flags_2 = ((flags_2) | (((1 as i32)).wrapping_shl(((2 as i32)) as u32))); flags_2 };
vm_block = 799; continue;
}
// C line 19374
801 => {
vm_block = if (((op_flags) & ((4 as i32)))) != 0 { 800 } else { 799 }; continue;
}
// C line 19369
804 => {
let _ = { let assigned = ((*({ let old = pc; pc = (pc).offset(1); old })) as i32); op_flags = assigned; assigned };
// C line 19371
let _ = { let assigned = *(sp).offset(((((2 as i32)).wrapping_neg()).wrapping_sub(is_computed)) as isize); obj_6 = assigned; assigned };
// C line 19372
let _ = { let assigned = ((((((((1 as i32)).wrapping_shl(((8 as i32)) as u32)) | (((1 as i32)).wrapping_shl(((0 as i32)) as u32)))) | (((1 as i32)).wrapping_shl(((10 as i32)) as u32)))) | (((1 as i32)).wrapping_shl(((14 as i32)) as u32))); flags_2 = assigned; assigned };
vm_block = 801; continue;
}
// C line 19364
805 => {
let _ = { opcode = (opcode).wrapping_add(((OP_define_method as i32)).wrapping_sub((OP_define_method_computed as i32))); opcode };
vm_block = 804; continue;
}
// C line 19363
806 => {
vm_block = 29; continue;
}
// C line 19362
807 => {
vm_block = if ((((!(((!(((((atom_10) == ((((0 as i32)) as JSAtom))) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 806 } else { 805 }; continue;
}
// C line 19361
808 => {
let _ = { let assigned = JS_ValueToAtom(ctx, *(sp).offset((((2 as i32)).wrapping_neg()) as isize)); atom_10 = assigned; assigned };
vm_block = 807; continue;
}
// C line 19366
810 => {
let _ = { let assigned = crate::cutils_header::get_u32(pc); atom_10 = assigned; assigned };
// C line 19367
let _ = { pc = (pc).offset((((4 as i32)) as isize)); pc };
vm_block = 804; continue;
}
// C line 19360
811 => {
vm_block = if (is_computed) != 0 { 808 } else { 810 }; continue;
}
// C line 19359
812 => {
let _ = { let assigned = (((opcode) == ((OP_define_method_computed as i32))) as i32); is_computed = assigned; assigned };
vm_block = 811; continue;
}
// C line 19344
814 => {
let _ = js_method_set_home_object(ctx, *(sp).offset((((1 as i32)).wrapping_neg()) as isize), *(sp).offset((((2 as i32)).wrapping_neg()) as isize));
// C line 19345
vm_block = 30; continue;
}
// C line 19339
817 => {
let _ = JS_FreeValue(ctx, proto_1);
// C line 19340
let _ = { let old = sp; sp = (sp).offset(-1); old };
// C line 19342
vm_block = 30; continue;
}
// C line 19337
818 => {
vm_block = 29; continue;
}
// C line 19336
819 => {
vm_block = if ((((JS_SetPrototypeInternal(ctx, *(sp).offset((((2 as i32)).wrapping_neg()) as isize), proto_1, (1 as i32))) < ((0 as i32))) as i32)) != 0 { 818 } else { 817 }; continue;
}
// C line 19335
820 => {
vm_block = if (((((JS_IsObject(proto_1)) != 0) || ((JS_IsNull(proto_1)) != 0)) as i32)) != 0 { 819 } else { 817 }; continue;
}
// C line 19333
822 => {
let _ = { let assigned = pc; (*(sf)).cur_pc = assigned; assigned };
// C line 19334
let _ = { let assigned = *(sp).offset((((1 as i32)).wrapping_neg()) as isize); proto_1 = assigned; assigned };
vm_block = 820; continue;
}
// C line 19329
823 => {
vm_block = 30; continue;
}
// C line 19327
824 => {
vm_block = 29; continue;
}
// C line 19326
825 => {
vm_block = if ((((!(((!(((((ret_10) < ((0 as i32))) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 824 } else { 823 }; continue;
}
// C line 19325
826 => {
let _ = { let assigned = JS_DefineObjectNameComputed(ctx, *(sp).offset((((1 as i32)).wrapping_neg()) as isize), *(sp).offset((((2 as i32)).wrapping_neg()) as isize), ((1 as i32)).wrapping_shl(((0 as i32)) as u32)); ret_10 = assigned; assigned };
vm_block = 825; continue;
}
// C line 19321
827 => {
vm_block = 30; continue;
}
// C line 19319
828 => {
vm_block = 29; continue;
}
// C line 19318
829 => {
vm_block = if ((((!(((!(((((ret_9) < ((0 as i32))) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 828 } else { 827 }; continue;
}
// C line 19314
832 => {
let _ = { let assigned = crate::cutils_header::get_u32(pc); atom_9 = assigned; assigned };
// C line 19315
let _ = { pc = (pc).offset((((4 as i32)) as isize)); pc };
// C line 19317
let _ = { let assigned = JS_DefineObjectName(ctx, *(sp).offset((((1 as i32)).wrapping_neg()) as isize), atom_9, ((1 as i32)).wrapping_shl(((0 as i32)) as u32)); ret_9 = assigned; assigned };
vm_block = 829; continue;
}
// C line 19308
833 => {
vm_block = 30; continue;
}
// C line 19306
834 => {
vm_block = 29; continue;
}
// C line 19305
835 => {
vm_block = if ((((!(((!(((((ret_8) < ((0 as i32))) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 834 } else { 833 }; continue;
}
// C line 19299
839 => {
let _ = { let assigned = crate::cutils_header::get_u32(pc); atom_8 = assigned; assigned };
// C line 19300
let _ = { pc = (pc).offset((((4 as i32)) as isize)); pc };
// C line 19302
let _ = { let assigned = JS_DefinePropertyValue(ctx, *(sp).offset((((2 as i32)).wrapping_neg()) as isize), atom_8, *(sp).offset((((1 as i32)).wrapping_neg()) as isize), ((((((((1 as i32)).wrapping_shl(((0 as i32)) as u32)) | (((1 as i32)).wrapping_shl(((1 as i32)) as u32)))) | (((1 as i32)).wrapping_shl(((2 as i32)) as u32)))) | (((1 as i32)).wrapping_shl(((14 as i32)) as u32)))); ret_8 = assigned; assigned };
// C line 19304
let _ = { let old = sp; sp = (sp).offset(-1); old };
vm_block = 835; continue;
}
// C line 19293
840 => {
vm_block = 30; continue;
}
// C line 19291
841 => {
vm_block = 29; continue;
}
// C line 19290
842 => {
vm_block = if ((((!(((!(((((ret_7) < ((0 as i32))) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 841 } else { 840 }; continue;
}
// C line 19287
845 => {
let _ = { let assigned = JS_DefinePrivateField(ctx, *(sp).offset((((3 as i32)).wrapping_neg()) as isize), *(sp).offset((((2 as i32)).wrapping_neg()) as isize), *(sp).offset((((1 as i32)).wrapping_neg()) as isize)); ret_7 = assigned; assigned };
// C line 19288
let _ = JS_FreeValue(ctx, *(sp).offset((((2 as i32)).wrapping_neg()) as isize));
// C line 19289
let _ = { sp = (sp).offset(-(((2 as i32)) as isize)); sp };
vm_block = 842; continue;
}
// C line 19282
846 => {
vm_block = 30; continue;
}
// C line 19280
847 => {
vm_block = 29; continue;
}
// C line 19279
848 => {
vm_block = if ((((!(((!(((((ret_6) < ((0 as i32))) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 847 } else { 846 }; continue;
}
// C line 19275
852 => {
let _ = { let assigned = JS_SetPrivateField(ctx, *(sp).offset((((3 as i32)).wrapping_neg()) as isize), *(sp).offset((((1 as i32)).wrapping_neg()) as isize), *(sp).offset((((2 as i32)).wrapping_neg()) as isize)); ret_6 = assigned; assigned };
// C line 19276
let _ = JS_FreeValue(ctx, *(sp).offset((((3 as i32)).wrapping_neg()) as isize));
// C line 19277
let _ = JS_FreeValue(ctx, *(sp).offset((((1 as i32)).wrapping_neg()) as isize));
// C line 19278
let _ = { sp = (sp).offset(-(((3 as i32)) as isize)); sp };
vm_block = 848; continue;
}
// C line 19270
853 => {
vm_block = 30; continue;
}
// C line 19268
854 => {
vm_block = 29; continue;
}
// C line 19267
855 => {
vm_block = if ((((!(((!((JS_IsException(val_9)) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 854 } else { 853 }; continue;
}
// C line 19262
860 => {
let _ = { let assigned = JS_GetPrivateField(ctx, *(sp).offset((((2 as i32)).wrapping_neg()) as isize), *(sp).offset((((1 as i32)).wrapping_neg()) as isize)); val_9 = assigned; assigned };
// C line 19263
let _ = JS_FreeValue(ctx, *(sp).offset((((1 as i32)).wrapping_neg()) as isize));
// C line 19264
let _ = JS_FreeValue(ctx, *(sp).offset((((2 as i32)).wrapping_neg()) as isize));
// C line 19265
let _ = { let assigned = val_9; *(sp).offset((((2 as i32)).wrapping_neg()) as isize) = assigned; assigned };
// C line 19266
let _ = { let old = sp; sp = (sp).offset(-1); old };
vm_block = 855; continue;
}
// C line 19254
862 => {
let _ = { let assigned = val_8; *({ let old = sp; sp = (sp).offset(1); old }) = assigned; assigned };
// C line 19256
vm_block = 30; continue;
}
// C line 19253
863 => {
vm_block = 29; continue;
}
// C line 19252
864 => {
vm_block = if (JS_IsException(val_8)) != 0 { 863 } else { 862 }; continue;
}
// C line 19249
867 => {
let _ = { let assigned = crate::cutils_header::get_u32(pc); atom_7 = assigned; assigned };
// C line 19250
let _ = { pc = (pc).offset((((4 as i32)) as isize)); pc };
// C line 19251
let _ = { let assigned = JS_NewSymbolFromAtom(ctx, atom_7, ((JS_ATOM_TYPE_PRIVATE as i32)) as u32); val_8 = assigned; assigned };
vm_block = 864; continue;
}
// C line 19242
868 => {
vm_block = 30; continue;
}
// C line 19224
871 => {
let _ = set_value(ctx, core::ptr::addr_of_mut!(((*(pr_4)).u).value), *(sp).offset((((1 as i32)).wrapping_neg()) as isize));
// C line 19228
let _ = JS_FreeValue(ctx, obj_5);
// C line 19229
let _ = { sp = (sp).offset(-(((2 as i32)) as isize)); sp };
vm_block = 868; continue;
}
// C line 19226
872 => {
vm_block = 883; continue;
}
// C line 19221
873 => {
vm_block = if ((((!(((!((((((((((*(prs_3)).flags()) as i32)) & (((((((3 as i32)).wrapping_shl(((4 as i32)) as u32)) | (((1 as i32)).wrapping_shl(((1 as i32)) as u32)))) | (((1 as i32)).wrapping_shl(((3 as i32)) as u32)))))) == (((1 as i32)).wrapping_shl(((1 as i32)) as u32))) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 871 } else { 872 }; continue;
}
// C line 19220
874 => {
vm_block = 883; continue;
}
// C line 19219
875 => {
vm_block = if ((!(!(prs_3).is_null()) as i32)) != 0 { 874 } else { 873 }; continue;
}
// C line 19217
877 => {
let _ = { let assigned = ((((obj_5).u).ptr) as *mut JSObject); p_4 = assigned; assigned };
// C line 19218
let _ = { let assigned = find_own_property(core::ptr::addr_of_mut!(pr_4), p_4, atom_6); prs_3 = assigned; assigned };
vm_block = 875; continue;
}
// C line 19238
878 => {
vm_block = 29; continue;
}
// C line 19237
879 => {
vm_block = if ((((!(((!(((((ret_5) < ((0 as i32))) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 878 } else { 868 }; continue;
}
// C line ? labels: put_field_slow_path
883 => {
let _ = { let assigned = pc; (*(sf)).cur_pc = assigned; assigned };
// C line 19233
let _ = { let assigned = JS_SetPropertyInternal(ctx, obj_5, atom_6, *(sp).offset((((1 as i32)).wrapping_neg()) as isize), obj_5, ((1 as i32)).wrapping_shl(((15 as i32)) as u32)); ret_5 = assigned; assigned };
// C line 19235
let _ = JS_FreeValue(ctx, obj_5);
// C line 19236
let _ = { sp = (sp).offset(-(((2 as i32)) as isize)); sp };
vm_block = 879; continue;
}
// C line 19216
884 => {
vm_block = if ((((!(((!((((((((obj_5).tag) as i32)) == ((JS_TAG_OBJECT as i32))) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 877 } else { 883 }; continue;
}
// C line 19212
887 => {
let _ = { let assigned = crate::cutils_header::get_u32(pc); atom_6 = assigned; assigned };
// C line 19213
let _ = { pc = (pc).offset((((4 as i32)) as isize)); pc };
// C line 19215
let _ = { let assigned = *(sp).offset((((2 as i32)).wrapping_neg()) as isize); obj_5 = assigned; assigned };
vm_block = 884; continue;
}
// C line 19200
888 => {
vm_block = 30; continue;
}
// C line 19199
889 => {
let _ = { let assigned = val_7; *({ let old = sp; sp = (sp).offset(1); old }) = assigned; assigned };
vm_block = 888; continue;
}
// C line 19199
891 => {
let _ = JS_FreeValue(ctx, *(sp).offset((((1 as i32)).wrapping_neg()) as isize));
// C line 19199
let _ = { let assigned = val_7; *(sp).offset((((1 as i32)).wrapping_neg()) as isize) = assigned; assigned };
vm_block = 888; continue;
}
// C line 19199
892 => {
vm_block = if ((0 as i32)) != 0 { 889 } else { 891 }; continue;
}
// C line 19199
893 => {
// C line 19199
let _ = { let assigned = find_own_property(core::ptr::addr_of_mut!(pr_3), p_3, atom_5); prs_2 = assigned; assigned };
vm_block = 905; continue;
}
// C line 19199
895 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; val_7 = assigned; assigned };
// C line 19199
vm_block = 892; continue;
}
// C line 19199
896 => {
vm_block = if ((!(!(p_3).is_null()) as i32)) != 0 { 895 } else { 893 }; continue;
}
// C line 19199
897 => {
let _ = { let assigned = (*((*(p_3)).shape)).proto; p_3 = assigned; assigned };
vm_block = 896; continue;
}
// C line 19199
899 => {
let _ = { let assigned = JSValue { u: JSValueUnion { ptr: ((p_3) as *mut c_void) }, tag: (((JS_TAG_OBJECT as i32)) as i64) }; obj_4 = assigned; assigned };
// C line 19199
vm_block = 911; continue;
}
// C line 19199
900 => {
vm_block = if ((((!(((!(((*(p_3)).is_exotic()) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 899 } else { 897 }; continue;
}
// C line 19199
902 => {
let _ = { let assigned = JS_DupValue(ctx, ((*(pr_3)).u).value); val_7 = assigned; assigned };
// C line 19199
vm_block = 892; continue;
}
// C line 19199
903 => {
vm_block = 911; continue;
}
// C line 19199
904 => {
vm_block = if ((((!(((!(((((((*(prs_2)).flags()) as i32)) & (((3 as i32)).wrapping_shl(((4 as i32)) as u32)))) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 903 } else { 902 }; continue;
}
// C line 19199
905 => {
vm_block = if !(prs_2).is_null() { 904 } else { 900 }; continue;
}
// C line 19199
907 => {
let _ = { let assigned = ((((obj_4).u).ptr) as *mut JSObject); p_3 = assigned; assigned };
vm_block = 893; continue;
}
// C line 19199
908 => {
vm_block = 29; continue;
}
// C line 19199
909 => {
vm_block = if ((((!(((!((JS_IsException(val_7)) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 908 } else { 892 }; continue;
}
// C line 19199 labels: get_length_slow_path
911 => {
let _ = { let assigned = pc; (*(sf)).cur_pc = assigned; assigned };
// C line 19199
let _ = { let assigned = JS_GetPropertyInternal(ctx, obj_4, atom_5, *(sp).offset((((1 as i32)).wrapping_neg()) as isize), (0 as i32)); val_7 = assigned; assigned };
vm_block = 909; continue;
}
// C line 19199
912 => {
vm_block = if ((((!(((!((((((((obj_4).tag) as i32)) == ((JS_TAG_OBJECT as i32))) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 907 } else { 911 }; continue;
}
// C line 19199
913 => {
let _ = { let assigned = *(sp).offset((((1 as i32)).wrapping_neg()) as isize); obj_4 = assigned; assigned };
vm_block = 912; continue;
}
// C line 19199
914 => {
let _ = { let assigned = (((JS_ATOM_length as i32)) as JSAtom); atom_5 = assigned; assigned };
vm_block = 913; continue;
}
// C line 19199
916 => {
let _ = { let assigned = crate::cutils_header::get_u32(pc); atom_5 = assigned; assigned };
// C line 19199
let _ = { pc = (pc).offset((((4 as i32)) as isize)); pc };
vm_block = 913; continue;
}
// C line 19199
917 => {
vm_block = if ((1 as i32)) != 0 { 914 } else { 916 }; continue;
}
// C line 19195
918 => {
vm_block = 30; continue;
}
// C line 19194
919 => {
let _ = { let assigned = val_6; *({ let old = sp; sp = (sp).offset(1); old }) = assigned; assigned };
vm_block = 918; continue;
}
// C line 19194
921 => {
let _ = JS_FreeValue(ctx, *(sp).offset((((1 as i32)).wrapping_neg()) as isize));
// C line 19194
let _ = { let assigned = val_6; *(sp).offset((((1 as i32)).wrapping_neg()) as isize) = assigned; assigned };
vm_block = 918; continue;
}
// C line 19194
922 => {
vm_block = if ((1 as i32)) != 0 { 919 } else { 921 }; continue;
}
// C line 19194
923 => {
// C line 19194
let _ = { let assigned = find_own_property(core::ptr::addr_of_mut!(pr_2), p_2, atom_4); prs_1 = assigned; assigned };
vm_block = 935; continue;
}
// C line 19194
925 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; val_6 = assigned; assigned };
// C line 19194
vm_block = 922; continue;
}
// C line 19194
926 => {
vm_block = if ((!(!(p_2).is_null()) as i32)) != 0 { 925 } else { 923 }; continue;
}
// C line 19194
927 => {
let _ = { let assigned = (*((*(p_2)).shape)).proto; p_2 = assigned; assigned };
vm_block = 926; continue;
}
// C line 19194
929 => {
let _ = { let assigned = JSValue { u: JSValueUnion { ptr: ((p_2) as *mut c_void) }, tag: (((JS_TAG_OBJECT as i32)) as i64) }; obj_3 = assigned; assigned };
// C line 19194
vm_block = 941; continue;
}
// C line 19194
930 => {
vm_block = if ((((!(((!(((*(p_2)).is_exotic()) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 929 } else { 927 }; continue;
}
// C line 19194
932 => {
let _ = { let assigned = JS_DupValue(ctx, ((*(pr_2)).u).value); val_6 = assigned; assigned };
// C line 19194
vm_block = 922; continue;
}
// C line 19194
933 => {
vm_block = 941; continue;
}
// C line 19194
934 => {
vm_block = if ((((!(((!(((((((*(prs_1)).flags()) as i32)) & (((3 as i32)).wrapping_shl(((4 as i32)) as u32)))) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 933 } else { 932 }; continue;
}
// C line 19194
935 => {
vm_block = if !(prs_1).is_null() { 934 } else { 930 }; continue;
}
// C line 19194
937 => {
let _ = { let assigned = ((((obj_3).u).ptr) as *mut JSObject); p_2 = assigned; assigned };
vm_block = 923; continue;
}
// C line 19194
938 => {
vm_block = 29; continue;
}
// C line 19194
939 => {
vm_block = if ((((!(((!((JS_IsException(val_6)) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 938 } else { 922 }; continue;
}
// C line 19194 labels: get_field2_slow_path
941 => {
let _ = { let assigned = pc; (*(sf)).cur_pc = assigned; assigned };
// C line 19194
let _ = { let assigned = JS_GetPropertyInternal(ctx, obj_3, atom_4, *(sp).offset((((1 as i32)).wrapping_neg()) as isize), (0 as i32)); val_6 = assigned; assigned };
vm_block = 939; continue;
}
// C line 19194
942 => {
vm_block = if ((((!(((!((((((((obj_3).tag) as i32)) == ((JS_TAG_OBJECT as i32))) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 937 } else { 941 }; continue;
}
// C line 19194
943 => {
let _ = { let assigned = *(sp).offset((((1 as i32)).wrapping_neg()) as isize); obj_3 = assigned; assigned };
js_host_error_stack_read(ctx, obj_3, atom_4);
vm_block = 942; continue;
}
// C line 19194
944 => {
let _ = { let assigned = (((JS_ATOM_length as i32)) as JSAtom); atom_4 = assigned; assigned };
vm_block = 943; continue;
}
// C line 19194
946 => {
let _ = { let assigned = crate::cutils_header::get_u32(pc); atom_4 = assigned; assigned };
// C line 19194
let _ = { pc = (pc).offset((((4 as i32)) as isize)); pc };
vm_block = 943; continue;
}
// C line 19194
947 => {
vm_block = if ((0 as i32)) != 0 { 944 } else { 946 }; continue;
}
// C line 19191
948 => {
vm_block = 30; continue;
}
// C line 19190
949 => {
let _ = { let assigned = val_5; *({ let old = sp; sp = (sp).offset(1); old }) = assigned; assigned };
vm_block = 948; continue;
}
// C line 19190
951 => {
let _ = JS_FreeValue(ctx, *(sp).offset((((1 as i32)).wrapping_neg()) as isize));
// C line 19190
let _ = { let assigned = val_5; *(sp).offset((((1 as i32)).wrapping_neg()) as isize) = assigned; assigned };
vm_block = 948; continue;
}
// C line 19190
952 => {
vm_block = if ((0 as i32)) != 0 { 949 } else { 951 }; continue;
}
// C line 19190
953 => {
// C line 19190
let _ = { let assigned = find_own_property(core::ptr::addr_of_mut!(pr_1), p_1, atom_3); prs = assigned; assigned };
vm_block = 965; continue;
}
// C line 19190
955 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; val_5 = assigned; assigned };
// C line 19190
vm_block = 952; continue;
}
// C line 19190
956 => {
vm_block = if ((!(!(p_1).is_null()) as i32)) != 0 { 955 } else { 953 }; continue;
}
// C line 19190
957 => {
let _ = { let assigned = (*((*(p_1)).shape)).proto; p_1 = assigned; assigned };
vm_block = 956; continue;
}
// C line 19190
959 => {
let _ = { let assigned = JSValue { u: JSValueUnion { ptr: ((p_1) as *mut c_void) }, tag: (((JS_TAG_OBJECT as i32)) as i64) }; obj_2 = assigned; assigned };
// C line 19190
vm_block = 971; continue;
}
// C line 19190
960 => {
vm_block = if ((((!(((!(((*(p_1)).is_exotic()) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 959 } else { 957 }; continue;
}
// C line 19190
962 => {
let _ = { let assigned = JS_DupValue(ctx, ((*(pr_1)).u).value); val_5 = assigned; assigned };
// C line 19190
vm_block = 952; continue;
}
// C line 19190
963 => {
vm_block = 971; continue;
}
// C line 19190
964 => {
vm_block = if ((((!(((!(((((((*(prs)).flags()) as i32)) & (((3 as i32)).wrapping_shl(((4 as i32)) as u32)))) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 963 } else { 962 }; continue;
}
// C line 19190
965 => {
vm_block = if !(prs).is_null() { 964 } else { 960 }; continue;
}
// C line 19190
967 => {
let _ = { let assigned = ((((obj_2).u).ptr) as *mut JSObject); p_1 = assigned; assigned };
vm_block = 953; continue;
}
// C line 19190
968 => {
vm_block = 29; continue;
}
// C line 19190
969 => {
vm_block = if ((((!(((!((JS_IsException(val_5)) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 968 } else { 952 }; continue;
}
// C line 19190 labels: get_field_slow_path
971 => {
let _ = { let assigned = pc; (*(sf)).cur_pc = assigned; assigned };
// C line 19190
let _ = { let assigned = JS_GetPropertyInternal(ctx, obj_2, atom_3, *(sp).offset((((1 as i32)).wrapping_neg()) as isize), (0 as i32)); val_5 = assigned; assigned };
vm_block = 969; continue;
}
// C line 19190
972 => {
vm_block = if ((((!(((!((((((((obj_2).tag) as i32)) == ((JS_TAG_OBJECT as i32))) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 967 } else { 971 }; continue;
}
// C line 19190
973 => {
let _ = { let assigned = *(sp).offset((((1 as i32)).wrapping_neg()) as isize); obj_2 = assigned; assigned };
js_host_error_stack_read(ctx, obj_2, atom_3);
vm_block = 972; continue;
}
// C line 19190
974 => {
let _ = { let assigned = (((JS_ATOM_length as i32)) as JSAtom); atom_3 = assigned; assigned };
vm_block = 973; continue;
}
// C line 19190
976 => {
let _ = { let assigned = crate::cutils_header::get_u32(pc); atom_3 = assigned; assigned };
// C line 19190
let _ = { pc = (pc).offset((((4 as i32)) as isize)); pc };
vm_block = 973; continue;
}
// C line 19190
977 => {
vm_block = if ((0 as i32)) != 0 { 974 } else { 976 }; continue;
}
// C line 19129
979 => {
let _ = { let assigned = JS_NewBool(ctx, (!((res_4) != 0) as i32)); *(sp).offset((((1 as i32)).wrapping_neg()) as isize) = assigned; assigned };
// C line 19131
vm_block = 30; continue;
}
// C line 19125
980 => {
let _ = { let assigned = (((((((op1_5).u).uint64) as i32)) != ((0 as i32))) as i32); res_4 = assigned; assigned };
vm_block = 979; continue;
}
// C line 19127
981 => {
let _ = { let assigned = JS_ToBoolFree(ctx, op1_5); res_4 = assigned; assigned };
vm_block = 979; continue;
}
// C line 19124
982 => {
vm_block = if (((((((((op1_5).tag) as i32)) as u32)) <= ((((JS_TAG_UNDEFINED as i32)) as u32))) as i32)) != 0 { 980 } else { 981 }; continue;
}
// C line 19123
983 => {
let _ = { let assigned = *(sp).offset((((1 as i32)).wrapping_neg()) as isize); op1_5 = assigned; assigned };
vm_block = 982; continue;
}
// C line 19113
986 => {
let _ = { let assigned = JS_NewBool(ctx, ret_flag); *(sp).offset(((0 as i32)) as isize) = assigned; assigned };
// C line 19114
let _ = { sp = (sp).offset((((1 as i32)) as isize)); sp };
// C line 19116
vm_block = 30; continue;
}
// C line 19097
987 => {
let _ = { let assigned = (1 as i32); ret_flag = assigned; assigned };
vm_block = 986; continue;
}
// C line 19109
990 => {
let _ = JS_FreeValue(ctx, *(sp).offset((((1 as i32)).wrapping_neg()) as isize));
// C line 19110
let _ = { let assigned = ret_4; *(sp).offset((((1 as i32)).wrapping_neg()) as isize) = assigned; assigned };
// C line 19111
let _ = { let assigned = (0 as i32); ret_flag = assigned; assigned };
vm_block = 986; continue;
}
// C line 19108
991 => {
vm_block = 29; continue;
}
// C line 19107
992 => {
vm_block = if (JS_IsException(ret_4)) != 0 { 991 } else { 990 }; continue;
}
// C line 19101
993 => {
let _ = { let assigned = JS_CallFree(ctx, method, *(sp).offset((((4 as i32)).wrapping_neg()) as isize), (0 as i32), ((core::ptr::null_mut::<c_void>()) as *mut JSValue)); ret_4 = assigned; assigned };
vm_block = 992; continue;
}
// C line 19104
994 => {
let _ = { let assigned = JS_CallFree(ctx, method, *(sp).offset((((4 as i32)).wrapping_neg()) as isize), (1 as i32), (sp).offset(-(((1 as i32)) as isize))); ret_4 = assigned; assigned };
vm_block = 992; continue;
}
// C line 19099
995 => {
vm_block = if (((flags_1) & ((2 as i32)))) != 0 { 993 } else { 994 }; continue;
}
// C line 19096
996 => {
vm_block = if (((((JS_IsUndefined(method)) != 0) || ((JS_IsNull(method)) != 0)) as i32)) != 0 { 987 } else { 995 }; continue;
}
// C line 19095
997 => {
vm_block = 29; continue;
}
// C line 19094
998 => {
vm_block = if (JS_IsException(method)) != 0 { 997 } else { 996 }; continue;
}
// C line 19090
1001 => {
let _ = { let assigned = ((*({ let old = pc; pc = (pc).offset(1); old })) as i32); flags_1 = assigned; assigned };
// C line 19091
let _ = { let assigned = pc; (*(sf)).cur_pc = assigned; assigned };
// C line 19092
let _ = { let assigned = JS_GetProperty(ctx, *(sp).offset((((4 as i32)).wrapping_neg()) as isize), ((if (((flags_1) & ((1 as i32)))) != 0 { (JS_ATOM_throw as i32) } else { (crate::quickjs_atom::JS_ATOM_return as i32) }) as JSAtom)); method = assigned; assigned };
vm_block = 998; continue;
}
// C line 19079
1004 => {
let _ = JS_FreeValue(ctx, *(sp).offset((((1 as i32)).wrapping_neg()) as isize));
// C line 19080
let _ = { let assigned = ret_3; *(sp).offset((((1 as i32)).wrapping_neg()) as isize) = assigned; assigned };
// C line 19082
vm_block = 30; continue;
}
// C line 19078
1005 => {
vm_block = 29; continue;
}
// C line 19077
1006 => {
vm_block = if (JS_IsException(ret_3)) != 0 { 1005 } else { 1004 }; continue;
}
// C line 19074
1008 => {
let _ = { let assigned = pc; (*(sf)).cur_pc = assigned; assigned };
// C line 19075
let _ = { let assigned = JS_Call(ctx, *(sp).offset((((3 as i32)).wrapping_neg()) as isize), *(sp).offset((((4 as i32)).wrapping_neg()) as isize), (1 as i32), (sp).offset(-(((1 as i32)) as isize))); ret_3 = assigned; assigned };
vm_block = 1006; continue;
}
// C line 19066
1010 => {
let _ = { let assigned = ret_val_1; *(sp).offset((((1 as i32)).wrapping_neg()) as isize) = assigned; assigned };
// C line 19068
vm_block = 30; continue;
}
// C line 19062
1013 => {
let _ = JS_ThrowInternalError(ctx, c"nip_catch".as_ptr());
// C line 19063
let _ = JS_FreeValue(ctx, ret_val_1);
// C line 19064
vm_block = 29; continue;
}
// C line 19061
1014 => {
vm_block = if ((((!(((!(((((sp) == (stack_buf)) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 1013 } else { 1010 }; continue;
}
// C line 19057
1015 => {
vm_block = if ((((((((sp) > (stack_buf)) as i32)) != 0) && ((((((((*(sp).offset((((1 as i32)).wrapping_neg()) as isize)).tag) as i32)) != ((JS_TAG_CATCH_OFFSET as i32))) as i32)) != 0)) as i32)) != 0 { 1016 } else { 1014 }; continue;
}
// C line 19059
1016 => {
let _ = JS_FreeValue(ctx, *({ sp = (sp).offset(-1); sp }));
vm_block = 1015; continue;
}
// C line 19056
1017 => {
let _ = { let assigned = *({ sp = (sp).offset(-1); sp }); ret_val_1 = assigned; assigned };
vm_block = 1015; continue;
}
// C line 19050
1019 => {
let _ = { let old = sp; sp = (sp).offset(-1); old };
// C line 19051
vm_block = 30; continue;
}
// C line 19048
1020 => {
let _ = JS_FreeValue(ctx, *(sp).offset((((1 as i32)).wrapping_neg()) as isize));
vm_block = 1019; continue;
}
// C line 19047
1021 => {
vm_block = 29; continue;
}
// C line 19046
1022 => {
vm_block = if (JS_IteratorClose(ctx, *(sp).offset((((1 as i32)).wrapping_neg()) as isize), (0 as i32))) != 0 { 1021 } else { 1020 }; continue;
}
// C line 19045
1023 => {
let _ = { let assigned = pc; (*(sf)).cur_pc = assigned; assigned };
vm_block = 1022; continue;
}
// C line 19044
1024 => {
vm_block = if ((!((JS_IsUndefined(*(sp).offset((((1 as i32)).wrapping_neg()) as isize))) != 0) as i32)) != 0 { 1023 } else { 1019 }; continue;
}
// C line 19041
1027 => {
let _ = { let old = sp; sp = (sp).offset(-1); old };
// C line 19042
let _ = JS_FreeValue(ctx, *(sp).offset((((1 as i32)).wrapping_neg()) as isize));
// C line 19043
let _ = { let old = sp; sp = (sp).offset(-1); old };
vm_block = 1024; continue;
}
// C line 19037
1028 => {
vm_block = 30; continue;
}
// C line 19034
1030 => {
let _ = JS_ThrowTypeError(ctx, c"iterator must return an object".as_ptr());
// C line 19035
vm_block = 29; continue;
}
// C line 19033
1031 => {
vm_block = if ((((!(((!(((!((JS_IsObject(*(sp).offset((((1 as i32)).wrapping_neg()) as isize))) != 0) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 1030 } else { 1028 }; continue;
}
// C line 19030
1033 => {
let _ = { sp = (sp).offset((((1 as i32)) as isize)); sp };
// C line 19031
vm_block = 30; continue;
}
// C line 19029
1034 => {
vm_block = 29; continue;
}
// C line 19028
1035 => {
vm_block = if (js_iterator_get_value_done(ctx, sp)) != 0 { 1034 } else { 1033 }; continue;
}
// C line 19027
1036 => {
let _ = { let assigned = pc; (*(sf)).cur_pc = assigned; assigned };
vm_block = 1035; continue;
}
// C line 19023
1039 => {
let _ = { sp = (sp).offset((((1 as i32)) as isize)); sp };
// C line 19024
let _ = { let assigned = JS_NewCatchOffset(ctx, (0 as i32)); *({ let old = sp; sp = (sp).offset(1); old }) = assigned; assigned };
// C line 19025
vm_block = 30; continue;
}
// C line 19022
1040 => {
vm_block = 29; continue;
}
// C line 19021
1041 => {
vm_block = if (js_for_of_start(ctx, sp, (1 as i32))) != 0 { 1040 } else { 1039 }; continue;
}
// C line 19020
1042 => {
let _ = { let assigned = pc; (*(sf)).cur_pc = assigned; assigned };
vm_block = 1041; continue;
}
// C line 19017
1044 => {
let _ = { let old = sp; sp = (sp).offset(1); old };
// C line 19018
vm_block = 30; continue;
}
// C line 19016
1045 => {
vm_block = 29; continue;
}
// C line 19015
1046 => {
vm_block = if (js_for_await_of_next(ctx, sp)) != 0 { 1045 } else { 1044 }; continue;
}
// C line 19014
1047 => {
let _ = { let assigned = pc; (*(sf)).cur_pc = assigned; assigned };
vm_block = 1046; continue;
}
// C line 19010
1049 => {
let _ = { sp = (sp).offset((((2 as i32)) as isize)); sp };
// C line 19012
vm_block = 30; continue;
}
// C line 19009
1050 => {
vm_block = 29; continue;
}
// C line 19008
1051 => {
vm_block = if (js_for_of_next(ctx, sp, offset)) != 0 { 1050 } else { 1049 }; continue;
}
// C line 19005
1054 => {
offset = (((3 as i32)).wrapping_neg()).wrapping_sub(((*(pc).offset(((0 as i32)) as isize)) as i32));
// C line 19006
let _ = { pc = (pc).offset((((1 as i32)) as isize)); pc };
// C line 19007
let _ = { let assigned = pc; (*(sf)).cur_pc = assigned; assigned };
vm_block = 1051; continue;
}
// C line 19000
1057 => {
let _ = { sp = (sp).offset((((1 as i32)) as isize)); sp };
// C line 19001
let _ = { let assigned = JS_NewCatchOffset(ctx, (0 as i32)); *({ let old = sp; sp = (sp).offset(1); old }) = assigned; assigned };
// C line 19002
vm_block = 30; continue;
}
// C line 18999
1058 => {
vm_block = 29; continue;
}
// C line 18998
1059 => {
vm_block = if (js_for_of_start(ctx, sp, (0 as i32))) != 0 { 1058 } else { 1057 }; continue;
}
// C line 18997
1060 => {
let _ = { let assigned = pc; (*(sf)).cur_pc = assigned; assigned };
vm_block = 1059; continue;
}
// C line 18994
1062 => {
let _ = { sp = (sp).offset((((2 as i32)) as isize)); sp };
// C line 18995
vm_block = 30; continue;
}
// C line 18993
1063 => {
vm_block = 29; continue;
}
// C line 18992
1064 => {
vm_block = if (js_for_in_next(ctx, sp)) != 0 { 1063 } else { 1062 }; continue;
}
// C line 18991
1065 => {
let _ = { let assigned = pc; (*(sf)).cur_pc = assigned; assigned };
vm_block = 1064; continue;
}
// C line 18989
1066 => {
vm_block = 30; continue;
}
// C line 18988
1067 => {
vm_block = 29; continue;
}
// C line 18987
1068 => {
vm_block = if (js_for_in_start(ctx, sp)) != 0 { 1067 } else { 1066 }; continue;
}
// C line 18986
1069 => {
let _ = { let assigned = pc; (*(sf)).cur_pc = assigned; assigned };
vm_block = 1068; continue;
}
// C line 18980
1072 => {
let _ = { let old = sp; sp = (sp).offset(-1); old };
// C line 18981
let _ = { let assigned = ((*(b)).byte_code_buf).offset(((pos) as isize)); pc = assigned; assigned };
// C line 18983
vm_block = 30; continue;
}
// C line ? labels: ret_fail
1074 => {
let _ = JS_ThrowInternalError(ctx, c"invalid ret value".as_ptr());
// C line 18978
vm_block = 29; continue;
}
// C line 18975
1075 => {
vm_block = if ((((!(((!(((((pos) >= ((((*(b)).byte_code_len) as u32))) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 1074 } else { 1072 }; continue;
}
// C line 18974
1076 => {
let _ = { let assigned = ((((((op1_4).u).uint64) as i32)) as u32); pos = assigned; assigned };
vm_block = 1075; continue;
}
// C line 18973
1077 => {
vm_block = 1074; continue;
}
// C line 18972
1078 => {
vm_block = if ((((!(((!((((((((op1_4).tag) as i32)) != ((JS_TAG_INT as i32))) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 1077 } else { 1076 }; continue;
}
// C line 18971
1079 => {
let _ = { let assigned = *(sp).offset((((1 as i32)).wrapping_neg()) as isize); op1_4 = assigned; assigned };
vm_block = 1078; continue;
}
// C line 18960
1084 => {
let _ = { let assigned = ((crate::cutils_header::get_u32(pc)) as i32); diff_1 = assigned; assigned };
// C line 18962
let _ = { let assigned = JS_NewInt32(ctx, (((((pc).offset((((4 as i32)) as isize))).offset_from((*(b)).byte_code_buf) as i64)) as i32)); *(sp).offset(((0 as i32)) as isize) = assigned; assigned };
// C line 18963
let _ = { let old = sp; sp = (sp).offset(1); old };
// C line 18964
let _ = { pc = (pc).offset(((diff_1) as isize)); pc };
// C line 18966
vm_block = 30; continue;
}
// C line 18951
1089 => {
let _ = { let assigned = ((crate::cutils_header::get_u32(pc)) as i32); diff = assigned; assigned };
// C line 18952
let _ = { let assigned = JS_NewCatchOffset(ctx, (((((pc).offset(((diff) as isize))).offset_from((*(b)).byte_code_buf) as i64)) as i32)); *(sp).offset(((0 as i32)) as isize) = assigned; assigned };
// C line 18953
let _ = { let old = sp; sp = (sp).offset(1); old };
// C line 18954
let _ = { pc = (pc).offset((((4 as i32)) as isize)); pc };
// C line 18956
vm_block = 30; continue;
}
// C line 18946
1090 => {
vm_block = 30; continue;
}
// C line 18944
1091 => {
vm_block = 29; continue;
}
// C line 18943
1092 => {
vm_block = if ((((!(((!((js_poll_interrupts_at(ctx, pc)) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 1091 } else { 1090 }; continue;
}
// C line 18941
1093 => {
let _ = { pc = (pc).offset((((((((*(pc).offset((((1 as i32)).wrapping_neg()) as isize)) as i8)) as i32)).wrapping_sub((1 as i32))) as isize)); pc };
vm_block = 1092; continue;
}
// C line 18940
1094 => {
vm_block = if ((!((res_3) != 0) as i32)) != 0 { 1093 } else { 1092 }; continue;
}
// C line 18939
1095 => {
let _ = { let old = sp; sp = (sp).offset(-1); old };
vm_block = 1094; continue;
}
// C line 18935
1096 => {
let _ = { let assigned = ((((op1_3).u).uint64) as i32); res_3 = assigned; assigned };
vm_block = 1095; continue;
}
// C line 18937
1097 => {
let _ = { let assigned = JS_ToBoolFree(ctx, op1_3); res_3 = assigned; assigned };
vm_block = 1095; continue;
}
// C line 18934
1098 => {
vm_block = if (((((((((op1_3).tag) as i32)) as u32)) <= ((((JS_TAG_UNDEFINED as i32)) as u32))) as i32)) != 0 { 1096 } else { 1097 }; continue;
}
// C line 18932
1100 => {
let _ = { let assigned = *(sp).offset((((1 as i32)).wrapping_neg()) as isize); op1_3 = assigned; assigned };
// C line 18933
let _ = { pc = (pc).offset((((1 as i32)) as isize)); pc };
vm_block = 1098; continue;
}
// C line 18926
1101 => {
vm_block = 30; continue;
}
// C line 18924
1102 => {
vm_block = 29; continue;
}
// C line 18923
1103 => {
vm_block = if ((((!(((!((js_poll_interrupts_at(ctx, pc)) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 1102 } else { 1101 }; continue;
}
// C line 18921
1104 => {
let _ = { pc = (pc).offset((((((((*(pc).offset((((1 as i32)).wrapping_neg()) as isize)) as i8)) as i32)).wrapping_sub((1 as i32))) as isize)); pc };
vm_block = 1103; continue;
}
// C line 18920
1105 => {
vm_block = if (res_2) != 0 { 1104 } else { 1103 }; continue;
}
// C line 18919
1106 => {
let _ = { let old = sp; sp = (sp).offset(-1); old };
vm_block = 1105; continue;
}
// C line 18915
1107 => {
let _ = { let assigned = ((((op1_2).u).uint64) as i32); res_2 = assigned; assigned };
vm_block = 1106; continue;
}
// C line 18917
1108 => {
let _ = { let assigned = JS_ToBoolFree(ctx, op1_2); res_2 = assigned; assigned };
vm_block = 1106; continue;
}
// C line 18914
1109 => {
vm_block = if (((((((((op1_2).tag) as i32)) as u32)) <= ((((JS_TAG_UNDEFINED as i32)) as u32))) as i32)) != 0 { 1107 } else { 1108 }; continue;
}
// C line 18912
1111 => {
let _ = { let assigned = *(sp).offset((((1 as i32)).wrapping_neg()) as isize); op1_2 = assigned; assigned };
// C line 18913
let _ = { pc = (pc).offset((((1 as i32)) as isize)); pc };
vm_block = 1109; continue;
}
// C line 18905
1112 => {
vm_block = 30; continue;
}
// C line 18903
1113 => {
vm_block = 29; continue;
}
// C line 18902
1114 => {
vm_block = if ((((!(((!((js_poll_interrupts_at(ctx, pc)) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 1113 } else { 1112 }; continue;
}
// C line 18900
1115 => {
let _ = { pc = (pc).offset((((((crate::cutils_header::get_u32((pc).offset(-(((4 as i32)) as isize)))) as i32)).wrapping_sub((4 as i32))) as isize)); pc };
vm_block = 1114; continue;
}
// C line 18899
1116 => {
vm_block = if ((!((res_1) != 0) as i32)) != 0 { 1115 } else { 1114 }; continue;
}
// C line 18898
1117 => {
let _ = { let old = sp; sp = (sp).offset(-1); old };
vm_block = 1116; continue;
}
// C line 18894
1118 => {
let _ = { let assigned = ((((op1_1).u).uint64) as i32); res_1 = assigned; assigned };
vm_block = 1117; continue;
}
// C line 18896
1119 => {
let _ = { let assigned = JS_ToBoolFree(ctx, op1_1); res_1 = assigned; assigned };
vm_block = 1117; continue;
}
// C line 18893
1120 => {
vm_block = if (((((((((op1_1).tag) as i32)) as u32)) <= ((((JS_TAG_UNDEFINED as i32)) as u32))) as i32)) != 0 { 1118 } else { 1119 }; continue;
}
// C line 18890
1122 => {
let _ = { let assigned = *(sp).offset((((1 as i32)).wrapping_neg()) as isize); op1_1 = assigned; assigned };
// C line 18891
let _ = { pc = (pc).offset((((4 as i32)) as isize)); pc };
vm_block = 1120; continue;
}
// C line 18884
1123 => {
vm_block = 30; continue;
}
// C line 18882
1124 => {
vm_block = 29; continue;
}
// C line 18881
1125 => {
vm_block = if ((((!(((!((js_poll_interrupts_at(ctx, pc)) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 1124 } else { 1123 }; continue;
}
// C line 18879
1126 => {
let _ = { pc = (pc).offset((((((crate::cutils_header::get_u32((pc).offset(-(((4 as i32)) as isize)))) as i32)).wrapping_sub((4 as i32))) as isize)); pc };
vm_block = 1125; continue;
}
// C line 18878
1127 => {
vm_block = if (res) != 0 { 1126 } else { 1125 }; continue;
}
// C line 18877
1128 => {
let _ = { let old = sp; sp = (sp).offset(-1); old };
vm_block = 1127; continue;
}
// C line 18873
1129 => {
let _ = { let assigned = ((((op1).u).uint64) as i32); res = assigned; assigned };
vm_block = 1128; continue;
}
// C line 18875
1130 => {
let _ = { let assigned = JS_ToBoolFree(ctx, op1); res = assigned; assigned };
vm_block = 1128; continue;
}
// C line 18872
1131 => {
vm_block = if (((((((((op1).tag) as i32)) as u32)) <= ((((JS_TAG_UNDEFINED as i32)) as u32))) as i32)) != 0 { 1129 } else { 1130 }; continue;
}
// C line 18870
1133 => {
let _ = { let assigned = *(sp).offset((((1 as i32)).wrapping_neg()) as isize); op1 = assigned; assigned };
// C line 18871
let _ = { pc = (pc).offset((((4 as i32)) as isize)); pc };
vm_block = 1131; continue;
}
// C line 18863
1134 => {
vm_block = 30; continue;
}
// C line 18862
1135 => {
vm_block = 29; continue;
}
// C line 18861
1136 => {
vm_block = if ((((!(((!((js_poll_interrupts_at(ctx, pc)) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 1135 } else { 1134 }; continue;
}
// C line 18860
1137 => {
let _ = { pc = (pc).offset(((((((*(pc).offset(((0 as i32)) as isize)) as i8)) as i32)) as isize)); pc };
vm_block = 1136; continue;
}
// C line 18858
1138 => {
vm_block = 30; continue;
}
// C line 18857
1139 => {
vm_block = 29; continue;
}
// C line 18856
1140 => {
vm_block = if ((((!(((!((js_poll_interrupts_at(ctx, pc)) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 1139 } else { 1138 }; continue;
}
// C line 18855
1141 => {
let _ = { pc = (pc).offset(((((((crate::cutils_header::get_u16(pc)) as i16)) as i32)) as isize)); pc };
vm_block = 1140; continue;
}
// C line 18852
1142 => {
vm_block = 30; continue;
}
// C line 18851
1143 => {
vm_block = 29; continue;
}
// C line 18850
1144 => {
vm_block = if ((((!(((!((js_poll_interrupts_at(ctx, pc)) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 1143 } else { 1142 }; continue;
}
// C line 18849
1145 => {
let _ = { pc = (pc).offset(((((crate::cutils_header::get_u32(pc)) as i32)) as isize)); pc };
vm_block = 1144; continue;
}
// C line 18844
1147 => {
let _ = { sp = (sp).offset((((2 as i32)) as isize)); sp };
// C line 18846
vm_block = 30; continue;
}
// C line 18843
1148 => {
vm_block = 29; continue;
}
// C line 18842
1149 => {
vm_block = if (JS_GetGlobalVarRef(ctx, atom_2, sp)) != 0 { 1148 } else { 1147 }; continue;
}
// C line 18838
1152 => {
let _ = { let assigned = crate::cutils_header::get_u32(pc); atom_2 = assigned; assigned };
// C line 18839
let _ = { pc = (pc).offset((((4 as i32)) as isize)); pc };
// C line 18840
let _ = { let assigned = pc; (*(sf)).cur_pc = assigned; assigned };
vm_block = 1149; continue;
}
// C line 18831
1155 => {
let _ = { let assigned = var_ref_1; ((*(pr)).u).var_ref = assigned; assigned };
// C line 18832
let _ = { let assigned = JS_AtomToValue(ctx, atom_1); *({ let old = sp; sp = (sp).offset(1); old }) = assigned; assigned };
// C line 18834
vm_block = 30; continue;
}
// C line 18828
1157 => {
let _ = free_var_ref(rt, var_ref_1);
// C line 18829
vm_block = 29; continue;
}
// C line 18827
1158 => {
vm_block = if ((!(!(pr).is_null()) as i32)) != 0 { 1157 } else { 1155 }; continue;
}
// C line 18825
1159 => {
let _ = { let assigned = add_property(ctx, ((((*(sp).offset((((1 as i32)).wrapping_neg()) as isize)).u).ptr) as *mut JSObject), atom_1, ((((1 as i32)).wrapping_shl(((1 as i32)) as u32)) | (((2 as i32)).wrapping_shl(((4 as i32)) as u32)))); pr = assigned; assigned };
vm_block = 1158; continue;
}
// C line 18818
1161 => {
let _ = { let assigned = *(var_refs).offset((idx_21) as isize); var_ref_1 = assigned; assigned };
// C line 18819
let _ = { let old = (*(js_rc(((var_ref_1) as *mut c_void)))).ref_count; (*(js_rc(((var_ref_1) as *mut c_void)))).ref_count = ((*(js_rc(((var_ref_1) as *mut c_void)))).ref_count).wrapping_add(1); old };
vm_block = 1159; continue;
}
// C line 18823
1162 => {
vm_block = 29; continue;
}
// C line 18822
1163 => {
vm_block = if ((!(!(var_ref_1).is_null()) as i32)) != 0 { 1162 } else { 1159 }; continue;
}
// C line 18821
1164 => {
let _ = { let assigned = get_var_ref(ctx, sf, idx_21, (((opcode) == ((OP_make_arg_ref as i32))) as i32)); var_ref_1 = assigned; assigned };
vm_block = 1163; continue;
}
// C line 18817
1165 => {
vm_block = if ((((opcode) == ((OP_make_var_ref_ref as i32))) as i32)) != 0 { 1161 } else { 1164 }; continue;
}
// C line 18816
1166 => {
vm_block = 29; continue;
}
// C line 18815
1167 => {
vm_block = if ((((!(((!((JS_IsException(*(sp).offset((((1 as i32)).wrapping_neg()) as isize))) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 1166 } else { 1165 }; continue;
}
// C line 18811
1171 => {
let _ = { let assigned = crate::cutils_header::get_u32(pc); atom_1 = assigned; assigned };
// C line 18812
let _ = { let assigned = ((crate::cutils_header::get_u16((pc).offset((((4 as i32)) as isize)))) as i32); idx_21 = assigned; assigned };
// C line 18813
let _ = { pc = (pc).offset((((6 as i32)) as isize)); pc };
// C line 18814
let _ = { let assigned = JS_NewObjectProto(ctx, JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_NULL as i32)) as i64) }); *({ let old = sp; sp = (sp).offset(1); old }) = assigned; assigned };
vm_block = 1167; continue;
}
// C line 18797
1175 => {
let _ = { let assigned = ((crate::cutils_header::get_u16(pc)) as i32); idx_20 = assigned; assigned };
// C line 18798
let _ = { pc = (pc).offset((((2 as i32)) as isize)); pc };
// C line 18799
let _ = close_lexical_var(ctx, b, sf, idx_20);
// C line 18801
vm_block = 30; continue;
}
// C line 18790
1178 => {
let _ = set_value(ctx, core::ptr::addr_of_mut!(*(var_buf).offset((idx_19) as isize)), *(sp).offset((((1 as i32)).wrapping_neg()) as isize));
// C line 18791
let _ = { let old = sp; sp = (sp).offset(-1); old };
// C line 18793
vm_block = 30; continue;
}
// C line 18787
1180 => {
let _ = JS_ThrowReferenceError(ctx, c"'this' can be initialized only once".as_ptr());
// C line 18788
vm_block = 29; continue;
}
// C line 18786
1181 => {
vm_block = if ((((!(((!(((!((JS_IsUninitialized(*(var_buf).offset((idx_19) as isize))) != 0) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 1180 } else { 1178 }; continue;
}
// C line 18784
1183 => {
let _ = { let assigned = ((crate::cutils_header::get_u16(pc)) as i32); idx_19 = assigned; assigned };
// C line 18785
let _ = { pc = (pc).offset((((2 as i32)) as isize)); pc };
vm_block = 1181; continue;
}
// C line 18778
1185 => {
let _ = set_value(ctx, core::ptr::addr_of_mut!(*(var_buf).offset((idx_18) as isize)), JS_DupValue(ctx, *(sp).offset((((1 as i32)).wrapping_neg()) as isize)));
// C line 18780
vm_block = 30; continue;
}
// C line 18775
1187 => {
let _ = JS_ThrowReferenceErrorUninitialized2(ctx, b, idx_18, (0 as i32));
// C line 18776
vm_block = 29; continue;
}
// C line 18774
1188 => {
vm_block = if ((((!(((!((JS_IsUninitialized(*(var_buf).offset((idx_18) as isize))) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 1187 } else { 1185 }; continue;
}
// C line 18772
1190 => {
let _ = { let assigned = ((crate::cutils_header::get_u16(pc)) as i32); idx_18 = assigned; assigned };
// C line 18773
let _ = { pc = (pc).offset((((2 as i32)) as isize)); pc };
vm_block = 1188; continue;
}
// C line 18765
1193 => {
let _ = set_value(ctx, core::ptr::addr_of_mut!(*(var_buf).offset((idx_17) as isize)), *(sp).offset((((1 as i32)).wrapping_neg()) as isize));
// C line 18766
let _ = { let old = sp; sp = (sp).offset(-1); old };
// C line 18768
vm_block = 30; continue;
}
// C line 18762
1195 => {
let _ = JS_ThrowReferenceErrorUninitialized2(ctx, b, idx_17, (0 as i32));
// C line 18763
vm_block = 29; continue;
}
// C line 18761
1196 => {
vm_block = if ((((!(((!((JS_IsUninitialized(*(var_buf).offset((idx_17) as isize))) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 1195 } else { 1193 }; continue;
}
// C line 18759
1198 => {
let _ = { let assigned = ((crate::cutils_header::get_u16(pc)) as i32); idx_17 = assigned; assigned };
// C line 18760
let _ = { pc = (pc).offset((((2 as i32)) as isize)); pc };
vm_block = 1196; continue;
}
// C line 18752
1201 => {
let _ = { let assigned = JS_DupValue(ctx, *(var_buf).offset((idx_16) as isize)); *(sp).offset(((0 as i32)) as isize) = assigned; assigned };
// C line 18753
let _ = { let old = sp; sp = (sp).offset(1); old };
// C line 18755
vm_block = 30; continue;
}
// C line 18749
1203 => {
let _ = JS_ThrowReferenceErrorUninitialized2(caller_ctx, b, idx_16, (0 as i32));
// C line 18750
vm_block = 29; continue;
}
// C line 18748
1204 => {
vm_block = if ((((!(((!((JS_IsUninitialized(*(var_buf).offset((idx_16) as isize))) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 1203 } else { 1201 }; continue;
}
// C line 18746
1206 => {
let _ = { let assigned = ((crate::cutils_header::get_u16(pc)) as i32); idx_16 = assigned; assigned };
// C line 18747
let _ = { pc = (pc).offset((((2 as i32)) as isize)); pc };
vm_block = 1204; continue;
}
// C line 18739
1209 => {
let _ = { let assigned = JS_DupValue(ctx, *(var_buf).offset((idx_15) as isize)); *(sp).offset(((0 as i32)) as isize) = assigned; assigned };
// C line 18740
let _ = { let old = sp; sp = (sp).offset(1); old };
// C line 18742
vm_block = 30; continue;
}
// C line 18736
1211 => {
let _ = JS_ThrowReferenceErrorUninitialized2(ctx, b, idx_15, (0 as i32));
// C line 18737
vm_block = 29; continue;
}
// C line 18735
1212 => {
vm_block = if ((((!(((!((JS_IsUninitialized(*(var_buf).offset((idx_15) as isize))) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 1211 } else { 1209 }; continue;
}
// C line 18733
1214 => {
let _ = { let assigned = ((crate::cutils_header::get_u16(pc)) as i32); idx_15 = assigned; assigned };
// C line 18734
let _ = { pc = (pc).offset((((2 as i32)) as isize)); pc };
vm_block = 1212; continue;
}
// C line 18725
1218 => {
let _ = { let assigned = ((crate::cutils_header::get_u16(pc)) as i32); idx_14 = assigned; assigned };
// C line 18726
let _ = { pc = (pc).offset((((2 as i32)) as isize)); pc };
// C line 18727
let _ = set_value(ctx, core::ptr::addr_of_mut!(*(var_buf).offset((idx_14) as isize)), JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNINITIALIZED as i32)) as i64) });
// C line 18729
vm_block = 30; continue;
}
// C line 18718
1221 => {
let _ = set_value(ctx, (*(*(var_refs).offset((idx_13) as isize))).pvalue, *(sp).offset((((1 as i32)).wrapping_neg()) as isize));
// C line 18719
let _ = { let old = sp; sp = (sp).offset(-1); old };
// C line 18721
vm_block = 30; continue;
}
// C line 18715
1223 => {
let _ = JS_ThrowReferenceErrorUninitialized2(ctx, b, idx_13, (1 as i32));
// C line 18716
vm_block = 29; continue;
}
// C line 18714
1224 => {
vm_block = if ((((!(((!(((!((JS_IsUninitialized(*((*(*(var_refs).offset((idx_13) as isize))).pvalue))) != 0) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 1223 } else { 1221 }; continue;
}
// C line 18712
1226 => {
let _ = { let assigned = ((crate::cutils_header::get_u16(pc)) as i32); idx_13 = assigned; assigned };
// C line 18713
let _ = { pc = (pc).offset((((2 as i32)) as isize)); pc };
vm_block = 1224; continue;
}
// C line 18705
1229 => {
let _ = set_value(ctx, (*(*(var_refs).offset((idx_12) as isize))).pvalue, *(sp).offset((((1 as i32)).wrapping_neg()) as isize));
// C line 18706
let _ = { let old = sp; sp = (sp).offset(-1); old };
// C line 18708
vm_block = 30; continue;
}
// C line 18702
1231 => {
let _ = JS_ThrowReferenceErrorUninitialized2(ctx, b, idx_12, (1 as i32));
// C line 18703
vm_block = 29; continue;
}
// C line 18701
1232 => {
vm_block = if ((((!(((!((JS_IsUninitialized(*((*(*(var_refs).offset((idx_12) as isize))).pvalue))) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 1231 } else { 1229 }; continue;
}
// C line 18699
1234 => {
let _ = { let assigned = ((crate::cutils_header::get_u16(pc)) as i32); idx_12 = assigned; assigned };
// C line 18700
let _ = { pc = (pc).offset((((2 as i32)) as isize)); pc };
vm_block = 1232; continue;
}
// C line 18692
1237 => {
let _ = { let assigned = JS_DupValue(ctx, val_4); *(sp).offset(((0 as i32)) as isize) = assigned; assigned };
// C line 18693
let _ = { let old = sp; sp = (sp).offset(1); old };
// C line 18695
vm_block = 30; continue;
}
// C line 18689
1239 => {
let _ = JS_ThrowReferenceErrorUninitialized2(ctx, b, idx_11, (1 as i32));
// C line 18690
vm_block = 29; continue;
}
// C line 18688
1240 => {
vm_block = if ((((!(((!((JS_IsUninitialized(val_4)) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 1239 } else { 1237 }; continue;
}
// C line 18685
1243 => {
let _ = { let assigned = ((crate::cutils_header::get_u16(pc)) as i32); idx_11 = assigned; assigned };
// C line 18686
let _ = { pc = (pc).offset((((2 as i32)) as isize)); pc };
// C line 18687
let _ = { let assigned = *((*(*(var_refs).offset((idx_11) as isize))).pvalue); val_4 = assigned; assigned };
vm_block = 1240; continue;
}
// C line 18676
1247 => {
let _ = { let assigned = ((crate::cutils_header::get_u16(pc)) as i32); idx_10 = assigned; assigned };
// C line 18677
let _ = { pc = (pc).offset((((2 as i32)) as isize)); pc };
// C line 18678
let _ = set_value(ctx, (*(*(var_refs).offset((idx_10) as isize))).pvalue, JS_DupValue(ctx, *(sp).offset((((1 as i32)).wrapping_neg()) as isize)));
// C line 18680
vm_block = 30; continue;
}
// C line 18667
1252 => {
let _ = { let assigned = ((crate::cutils_header::get_u16(pc)) as i32); idx_9 = assigned; assigned };
// C line 18668
let _ = { pc = (pc).offset((((2 as i32)) as isize)); pc };
// C line 18669
let _ = set_value(ctx, (*(*(var_refs).offset((idx_9) as isize))).pvalue, *(sp).offset((((1 as i32)).wrapping_neg()) as isize));
// C line 18670
let _ = { let old = sp; sp = (sp).offset(-1); old };
// C line 18672
vm_block = 30; continue;
}
// C line 18657
1258 => {
let _ = { let assigned = ((crate::cutils_header::get_u16(pc)) as i32); idx_8 = assigned; assigned };
// C line 18658
let _ = { pc = (pc).offset((((2 as i32)) as isize)); pc };
// C line 18659
let _ = { let assigned = *((*(*(var_refs).offset((idx_8) as isize))).pvalue); val_3 = assigned; assigned };
// C line 18660
let _ = { let assigned = JS_DupValue(ctx, val_3); *(sp).offset(((0 as i32)) as isize) = assigned; assigned };
// C line 18661
let _ = { let old = sp; sp = (sp).offset(1); old };
// C line 18663
vm_block = 30; continue;
}
// C line ?
1260 => {
let _ = set_value(ctx, (*(*(var_refs).offset(((3 as i32)) as isize))).pvalue, JS_DupValue(ctx, *(sp).offset((((1 as i32)).wrapping_neg()) as isize)));
// C line 18650
vm_block = 30; continue;
}
// C line ?
1262 => {
let _ = set_value(ctx, (*(*(var_refs).offset(((2 as i32)) as isize))).pvalue, JS_DupValue(ctx, *(sp).offset((((1 as i32)).wrapping_neg()) as isize)));
// C line 18649
vm_block = 30; continue;
}
// C line ?
1264 => {
let _ = set_value(ctx, (*(*(var_refs).offset(((1 as i32)) as isize))).pvalue, JS_DupValue(ctx, *(sp).offset((((1 as i32)).wrapping_neg()) as isize)));
// C line 18648
vm_block = 30; continue;
}
// C line ?
1266 => {
let _ = set_value(ctx, (*(*(var_refs).offset(((0 as i32)) as isize))).pvalue, JS_DupValue(ctx, *(sp).offset((((1 as i32)).wrapping_neg()) as isize)));
// C line 18647
vm_block = 30; continue;
}
// C line ?
1268 => {
let _ = set_value(ctx, (*(*(var_refs).offset(((3 as i32)) as isize))).pvalue, *({ sp = (sp).offset(-1); sp }));
// C line 18646
vm_block = 30; continue;
}
// C line ?
1270 => {
let _ = set_value(ctx, (*(*(var_refs).offset(((2 as i32)) as isize))).pvalue, *({ sp = (sp).offset(-1); sp }));
// C line 18645
vm_block = 30; continue;
}
// C line ?
1272 => {
let _ = set_value(ctx, (*(*(var_refs).offset(((1 as i32)) as isize))).pvalue, *({ sp = (sp).offset(-1); sp }));
// C line 18644
vm_block = 30; continue;
}
// C line ?
1274 => {
let _ = set_value(ctx, (*(*(var_refs).offset(((0 as i32)) as isize))).pvalue, *({ sp = (sp).offset(-1); sp }));
// C line 18643
vm_block = 30; continue;
}
// C line ?
1276 => {
let _ = { let assigned = JS_DupValue(ctx, *((*(*(var_refs).offset(((3 as i32)) as isize))).pvalue)); *({ let old = sp; sp = (sp).offset(1); old }) = assigned; assigned };
// C line 18642
vm_block = 30; continue;
}
// C line ?
1278 => {
let _ = { let assigned = JS_DupValue(ctx, *((*(*(var_refs).offset(((2 as i32)) as isize))).pvalue)); *({ let old = sp; sp = (sp).offset(1); old }) = assigned; assigned };
// C line 18641
vm_block = 30; continue;
}
// C line ?
1280 => {
let _ = { let assigned = JS_DupValue(ctx, *((*(*(var_refs).offset(((1 as i32)) as isize))).pvalue)); *({ let old = sp; sp = (sp).offset(1); old }) = assigned; assigned };
// C line 18640
vm_block = 30; continue;
}
// C line ?
1282 => {
let _ = { let assigned = JS_DupValue(ctx, *((*(*(var_refs).offset(((0 as i32)) as isize))).pvalue)); *({ let old = sp; sp = (sp).offset(1); old }) = assigned; assigned };
// C line 18639
vm_block = 30; continue;
}
// C line ?
1284 => {
let _ = set_value(ctx, core::ptr::addr_of_mut!(*(arg_buf).offset(((3 as i32)) as isize)), JS_DupValue(ctx, *(sp).offset((((1 as i32)).wrapping_neg()) as isize)));
// C line 18638
vm_block = 30; continue;
}
// C line ?
1286 => {
let _ = set_value(ctx, core::ptr::addr_of_mut!(*(arg_buf).offset(((2 as i32)) as isize)), JS_DupValue(ctx, *(sp).offset((((1 as i32)).wrapping_neg()) as isize)));
// C line 18637
vm_block = 30; continue;
}
// C line ?
1288 => {
let _ = set_value(ctx, core::ptr::addr_of_mut!(*(arg_buf).offset(((1 as i32)) as isize)), JS_DupValue(ctx, *(sp).offset((((1 as i32)).wrapping_neg()) as isize)));
// C line 18636
vm_block = 30; continue;
}
// C line ?
1290 => {
let _ = set_value(ctx, core::ptr::addr_of_mut!(*(arg_buf).offset(((0 as i32)) as isize)), JS_DupValue(ctx, *(sp).offset((((1 as i32)).wrapping_neg()) as isize)));
// C line 18635
vm_block = 30; continue;
}
// C line ?
1292 => {
let _ = set_value(ctx, core::ptr::addr_of_mut!(*(arg_buf).offset(((3 as i32)) as isize)), *({ sp = (sp).offset(-1); sp }));
// C line 18634
vm_block = 30; continue;
}
// C line ?
1294 => {
let _ = set_value(ctx, core::ptr::addr_of_mut!(*(arg_buf).offset(((2 as i32)) as isize)), *({ sp = (sp).offset(-1); sp }));
// C line 18633
vm_block = 30; continue;
}
// C line ?
1296 => {
let _ = set_value(ctx, core::ptr::addr_of_mut!(*(arg_buf).offset(((1 as i32)) as isize)), *({ sp = (sp).offset(-1); sp }));
// C line 18632
vm_block = 30; continue;
}
// C line ?
1298 => {
let _ = set_value(ctx, core::ptr::addr_of_mut!(*(arg_buf).offset(((0 as i32)) as isize)), *({ sp = (sp).offset(-1); sp }));
// C line 18631
vm_block = 30; continue;
}
// C line ?
1300 => {
let _ = { let assigned = JS_DupValue(ctx, *(arg_buf).offset(((3 as i32)) as isize)); *({ let old = sp; sp = (sp).offset(1); old }) = assigned; assigned };
// C line 18630
vm_block = 30; continue;
}
// C line ?
1302 => {
let _ = { let assigned = JS_DupValue(ctx, *(arg_buf).offset(((2 as i32)) as isize)); *({ let old = sp; sp = (sp).offset(1); old }) = assigned; assigned };
// C line 18629
vm_block = 30; continue;
}
// C line ?
1304 => {
let _ = { let assigned = JS_DupValue(ctx, *(arg_buf).offset(((1 as i32)) as isize)); *({ let old = sp; sp = (sp).offset(1); old }) = assigned; assigned };
// C line 18628
vm_block = 30; continue;
}
// C line ?
1306 => {
let _ = { let assigned = JS_DupValue(ctx, *(arg_buf).offset(((0 as i32)) as isize)); *({ let old = sp; sp = (sp).offset(1); old }) = assigned; assigned };
// C line 18627
vm_block = 30; continue;
}
// C line ?
1308 => {
let _ = set_value(ctx, core::ptr::addr_of_mut!(*(var_buf).offset(((3 as i32)) as isize)), JS_DupValue(ctx, *(sp).offset((((1 as i32)).wrapping_neg()) as isize)));
// C line 18626
vm_block = 30; continue;
}
// C line ?
1310 => {
let _ = set_value(ctx, core::ptr::addr_of_mut!(*(var_buf).offset(((2 as i32)) as isize)), JS_DupValue(ctx, *(sp).offset((((1 as i32)).wrapping_neg()) as isize)));
// C line 18625
vm_block = 30; continue;
}
// C line ?
1312 => {
let _ = set_value(ctx, core::ptr::addr_of_mut!(*(var_buf).offset(((1 as i32)) as isize)), JS_DupValue(ctx, *(sp).offset((((1 as i32)).wrapping_neg()) as isize)));
// C line 18624
vm_block = 30; continue;
}
// C line ?
1314 => {
let _ = set_value(ctx, core::ptr::addr_of_mut!(*(var_buf).offset(((0 as i32)) as isize)), JS_DupValue(ctx, *(sp).offset((((1 as i32)).wrapping_neg()) as isize)));
// C line 18623
vm_block = 30; continue;
}
// C line ?
1316 => {
let _ = set_value(ctx, core::ptr::addr_of_mut!(*(var_buf).offset(((3 as i32)) as isize)), *({ sp = (sp).offset(-1); sp }));
// C line 18622
vm_block = 30; continue;
}
// C line ?
1318 => {
let _ = set_value(ctx, core::ptr::addr_of_mut!(*(var_buf).offset(((2 as i32)) as isize)), *({ sp = (sp).offset(-1); sp }));
// C line 18621
vm_block = 30; continue;
}
// C line ?
1320 => {
let _ = set_value(ctx, core::ptr::addr_of_mut!(*(var_buf).offset(((1 as i32)) as isize)), *({ sp = (sp).offset(-1); sp }));
// C line 18620
vm_block = 30; continue;
}
// C line ?
1322 => {
let _ = set_value(ctx, core::ptr::addr_of_mut!(*(var_buf).offset(((0 as i32)) as isize)), *({ sp = (sp).offset(-1); sp }));
// C line 18619
vm_block = 30; continue;
}
// C line ?
1324 => {
let _ = { let assigned = JS_DupValue(ctx, *(var_buf).offset(((3 as i32)) as isize)); *({ let old = sp; sp = (sp).offset(1); old }) = assigned; assigned };
// C line 18618
vm_block = 30; continue;
}
// C line ?
1326 => {
let _ = { let assigned = JS_DupValue(ctx, *(var_buf).offset(((2 as i32)) as isize)); *({ let old = sp; sp = (sp).offset(1); old }) = assigned; assigned };
// C line 18617
vm_block = 30; continue;
}
// C line ?
1328 => {
let _ = { let assigned = JS_DupValue(ctx, *(var_buf).offset(((1 as i32)) as isize)); *({ let old = sp; sp = (sp).offset(1); old }) = assigned; assigned };
// C line 18616
vm_block = 30; continue;
}
// C line ?
1330 => {
let _ = { let assigned = JS_DupValue(ctx, *(var_buf).offset(((0 as i32)) as isize)); *({ let old = sp; sp = (sp).offset(1); old }) = assigned; assigned };
// C line 18615
vm_block = 30; continue;
}
// C line ?
1332 => {
let _ = set_value(ctx, core::ptr::addr_of_mut!(*(var_buf).offset((*({ let old = pc; pc = (pc).offset(1); old })) as isize)), JS_DupValue(ctx, *(sp).offset((((1 as i32)).wrapping_neg()) as isize)));
// C line 18613
vm_block = 30; continue;
}
// C line ?
1334 => {
let _ = set_value(ctx, core::ptr::addr_of_mut!(*(var_buf).offset((*({ let old = pc; pc = (pc).offset(1); old })) as isize)), *({ sp = (sp).offset(-1); sp }));
// C line 18612
vm_block = 30; continue;
}
// C line ?
1336 => {
let _ = { let assigned = JS_DupValue(ctx, *(var_buf).offset((*({ let old = pc; pc = (pc).offset(1); old })) as isize)); *({ let old = sp; sp = (sp).offset(1); old }) = assigned; assigned };
// C line 18611
vm_block = 30; continue;
}
// C line 18604
1340 => {
let _ = { let assigned = ((crate::cutils_header::get_u16(pc)) as i32); idx_7 = assigned; assigned };
// C line 18605
let _ = { pc = (pc).offset((((2 as i32)) as isize)); pc };
// C line 18606
let _ = set_value(ctx, core::ptr::addr_of_mut!(*(arg_buf).offset((idx_7) as isize)), JS_DupValue(ctx, *(sp).offset((((1 as i32)).wrapping_neg()) as isize)));
// C line 18608
vm_block = 30; continue;
}
// C line 18595
1345 => {
let _ = { let assigned = ((crate::cutils_header::get_u16(pc)) as i32); idx_6 = assigned; assigned };
// C line 18596
let _ = { pc = (pc).offset((((2 as i32)) as isize)); pc };
// C line 18597
let _ = set_value(ctx, core::ptr::addr_of_mut!(*(arg_buf).offset((idx_6) as isize)), *(sp).offset((((1 as i32)).wrapping_neg()) as isize));
// C line 18598
let _ = { let old = sp; sp = (sp).offset(-1); old };
// C line 18600
vm_block = 30; continue;
}
// C line 18586
1350 => {
let _ = { let assigned = ((crate::cutils_header::get_u16(pc)) as i32); idx_5 = assigned; assigned };
// C line 18587
let _ = { pc = (pc).offset((((2 as i32)) as isize)); pc };
// C line 18588
let _ = { let assigned = JS_DupValue(ctx, *(arg_buf).offset((idx_5) as isize)); *(sp).offset(((0 as i32)) as isize) = assigned; assigned };
// C line 18589
let _ = { let old = sp; sp = (sp).offset(1); old };
// C line 18591
vm_block = 30; continue;
}
// C line 18578
1354 => {
let _ = { let assigned = ((crate::cutils_header::get_u16(pc)) as i32); idx_4 = assigned; assigned };
// C line 18579
let _ = { pc = (pc).offset((((2 as i32)) as isize)); pc };
// C line 18580
let _ = set_value(ctx, core::ptr::addr_of_mut!(*(var_buf).offset((idx_4) as isize)), JS_DupValue(ctx, *(sp).offset((((1 as i32)).wrapping_neg()) as isize)));
// C line 18582
vm_block = 30; continue;
}
// C line 18569
1359 => {
let _ = { let assigned = ((crate::cutils_header::get_u16(pc)) as i32); idx_3 = assigned; assigned };
// C line 18570
let _ = { pc = (pc).offset((((2 as i32)) as isize)); pc };
// C line 18571
let _ = set_value(ctx, core::ptr::addr_of_mut!(*(var_buf).offset((idx_3) as isize)), *(sp).offset((((1 as i32)).wrapping_neg()) as isize));
// C line 18572
let _ = { let old = sp; sp = (sp).offset(-1); old };
// C line 18574
vm_block = 30; continue;
}
// C line 18560
1364 => {
let _ = { let assigned = ((crate::cutils_header::get_u16(pc)) as i32); idx_2 = assigned; assigned };
// C line 18561
let _ = { pc = (pc).offset((((2 as i32)) as isize)); pc };
// C line 18562
let _ = { let assigned = JS_DupValue(ctx, *(var_buf).offset((idx_2) as isize)); *(sp).offset(((0 as i32)) as isize) = assigned; assigned };
// C line 18563
let _ = { let old = sp; sp = (sp).offset(1); old };
// C line 18565
vm_block = 30; continue;
}
// C line 18556
1365 => {
vm_block = 30; continue;
}
// C line 18534
1366 => {
vm_block = 29; continue;
}
// C line 18531
1367 => {
let _ = JS_ThrowReferenceErrorUninitialized(ctx, (*(cv_1)).var_name);
vm_block = 1366; continue;
}
// C line 18533
1368 => {
let _ = JS_ThrowTypeErrorReadOnly(ctx, ((1 as i32)).wrapping_shl(((14 as i32)) as u32), (*(cv_1)).var_name);
vm_block = 1366; continue;
}
// C line 18530
1369 => {
vm_block = if (JS_IsUninitialized(*((*(var_ref)).pvalue))) != 0 { 1367 } else { 1368 }; continue;
}
// C line 18529
1370 => {
vm_block = 1386; continue;
}
// C line 18528
1371 => {
vm_block = if ((((opcode) == ((OP_put_var_init as i32))) as i32)) != 0 { 1370 } else { 1369 }; continue;
}
// C line 18548
1372 => {
vm_block = 29; continue;
}
// C line 18547
1373 => {
vm_block = if ((((ret_2) < ((0 as i32))) as i32)) != 0 { 1372 } else { 1365 }; continue;
}
// C line 18544
1375 => {
let _ = { let assigned = JS_SetPropertyInternal(ctx, (*(ctx)).global_obj, (*(cv_1)).var_name, *(sp).offset((((1 as i32)).wrapping_neg()) as isize), (*(ctx)).global_obj, ((1 as i32)).wrapping_shl(((15 as i32)) as u32)); ret_2 = assigned; assigned };
// C line 18546
let _ = { let old = sp; sp = (sp).offset(-1); old };
vm_block = 1373; continue;
}
// C line 18541
1377 => {
let _ = JS_ThrowReferenceErrorNotDefined(ctx, (*(cv_1)).var_name);
// C line 18542
vm_block = 29; continue;
}
// C line 18540
1378 => {
vm_block = if ((((((((ret_2) == ((0 as i32))) as i32)) != 0) && ((is_strict_mode(ctx)) != 0)) as i32)) != 0 { 1377 } else { 1375 }; continue;
}
// C line 18539
1379 => {
vm_block = 29; continue;
}
// C line 18538
1380 => {
vm_block = if ((((ret_2) < ((0 as i32))) as i32)) != 0 { 1379 } else { 1378 }; continue;
}
// C line 18536
1382 => {
let _ = { let assigned = pc; (*(sf)).cur_pc = assigned; assigned };
// C line 18537
let _ = { let assigned = JS_HasProperty(ctx, (*(ctx)).global_obj, (*(cv_1)).var_name); ret_2 = assigned; assigned };
vm_block = 1380; continue;
}
// C line 18527
1383 => {
vm_block = if ((*(var_ref)).is_lexical) != 0 { 1371 } else { 1382 }; continue;
}
// C line 18526
1384 => {
cv_1 = core::ptr::addr_of_mut!(*((*(b)).closure_var).offset((idx_1) as isize));
vm_block = 1383; continue;
}
// C line ? labels: put_var_ok
1386 => {
let _ = set_value(ctx, (*(var_ref)).pvalue, *(sp).offset((((1 as i32)).wrapping_neg()) as isize));
// C line 18553
let _ = { let old = sp; sp = (sp).offset(-1); old };
vm_block = 1365; continue;
}
// C line 18524
1387 => {
vm_block = if ((((!(((!((((((JS_IsUninitialized(*((*(var_ref)).pvalue))) != 0) || (((((*(var_ref)).is_const) as i32)) != 0)) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 1384 } else { 1386 }; continue;
}
// C line 18521
1390 => {
let _ = { let assigned = ((crate::cutils_header::get_u16(pc)) as i32); idx_1 = assigned; assigned };
// C line 18522
let _ = { pc = (pc).offset((((2 as i32)) as isize)); pc };
// C line 18523
let _ = { let assigned = *(var_refs).offset((idx_1) as isize); var_ref = assigned; assigned };
vm_block = 1387; continue;
}
// C line 18512
1392 => {
let _ = { let old = sp; sp = (sp).offset(1); old };
// C line 18514
vm_block = 30; continue;
}
// C line 18498
1394 => {
let _ = JS_ThrowReferenceErrorUninitialized(ctx, (*(cv)).var_name);
// C line 18499
vm_block = 29; continue;
}
// C line 18507
1395 => {
vm_block = 29; continue;
}
// C line 18506
1396 => {
vm_block = if (JS_IsException(*(sp).offset(((0 as i32)) as isize))) != 0 { 1395 } else { 1392 }; continue;
}
// C line 18501
1398 => {
let _ = { let assigned = pc; (*(sf)).cur_pc = assigned; assigned };
// C line 18502
let _ = { let assigned = JS_GetPropertyInternal(ctx, (*(ctx)).global_obj, (*(cv)).var_name, (*(ctx)).global_obj, (opcode).wrapping_sub((OP_get_var_undef as i32))); *(sp).offset(((0 as i32)) as isize) = assigned; assigned };
vm_block = 1396; continue;
}
// C line 18497
1399 => {
vm_block = if ((*(cv)).is_lexical()) != 0 { 1394 } else { 1398 }; continue;
}
// C line 18496
1400 => {
cv = core::ptr::addr_of_mut!(*((*(b)).closure_var).offset((idx) as isize));
vm_block = 1399; continue;
}
// C line 18510
1401 => {
let _ = { let assigned = JS_DupValue(ctx, val_2); *(sp).offset(((0 as i32)) as isize) = assigned; assigned };
vm_block = 1392; continue;
}
// C line 18495
1402 => {
vm_block = if ((((!(((!((JS_IsUninitialized(val_2)) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 1400 } else { 1401 }; continue;
}
// C line 18492
1405 => {
let _ = { let assigned = ((crate::cutils_header::get_u16(pc)) as i32); idx = assigned; assigned };
// C line 18493
let _ = { pc = (pc).offset((((2 as i32)) as isize)); pc };
// C line 18494
let _ = { let assigned = *((*(*(var_refs).offset((idx) as isize))).pvalue); val_2 = assigned; assigned };
vm_block = 1402; continue;
}
// C line 18480
1410 => {
let _ = JS_FreeValue(ctx, *(sp).offset((((2 as i32)).wrapping_neg()) as isize));
// C line 18481
let _ = JS_FreeValue(ctx, *(sp).offset((((1 as i32)).wrapping_neg()) as isize));
// C line 18482
let _ = { let old = sp; sp = (sp).offset(-1); old };
// C line 18483
let _ = { let assigned = val_1; *(sp).offset((((1 as i32)).wrapping_neg()) as isize) = assigned; assigned };
// C line 18485
vm_block = 30; continue;
}
// C line 18479
1411 => {
vm_block = 29; continue;
}
// C line 18478
1412 => {
vm_block = if (JS_IsException(val_1)) != 0 { 1411 } else { 1410 }; continue;
}
// C line 18476
1414 => {
let _ = { let assigned = pc; (*(sf)).cur_pc = assigned; assigned };
// C line 18477
let _ = { let assigned = js_dynamic_import(ctx, *(sp).offset((((2 as i32)).wrapping_neg()) as isize), *(sp).offset((((1 as i32)).wrapping_neg()) as isize)); val_1 = assigned; assigned };
vm_block = 1412; continue;
}
// C line 18468
1417 => {
let _ = JS_FreeValue(ctx, *(sp).offset((((1 as i32)).wrapping_neg()) as isize));
// C line 18469
let _ = { let assigned = proto; *(sp).offset((((1 as i32)).wrapping_neg()) as isize) = assigned; assigned };
// C line 18471
vm_block = 30; continue;
}
// C line 18467
1418 => {
vm_block = 29; continue;
}
// C line 18466
1419 => {
vm_block = if (JS_IsException(proto)) != 0 { 1418 } else { 1417 }; continue;
}
// C line 18464
1421 => {
let _ = { let assigned = pc; (*(sf)).cur_pc = assigned; assigned };
// C line 18465
let _ = { let assigned = JS_GetPrototype(ctx, *(sp).offset((((1 as i32)).wrapping_neg()) as isize)); proto = assigned; assigned };
vm_block = 1419; continue;
}
// C line 18459
1422 => {
vm_block = 30; continue;
}
// C line 18457
1423 => {
vm_block = 29; continue;
}
// C line 18456
1424 => {
vm_block = if (JS_IsException(*(sp).offset((((1 as i32)).wrapping_neg()) as isize))) != 0 { 1423 } else { 1422 }; continue;
}
// C line 18454
1426 => {
let _ = { let assigned = JS_NewRegexp(ctx, *(sp).offset((((2 as i32)).wrapping_neg()) as isize), *(sp).offset((((1 as i32)).wrapping_neg()) as isize)); *(sp).offset((((2 as i32)).wrapping_neg()) as isize) = assigned; assigned };
// C line 18455
let _ = { let old = sp; sp = (sp).offset(-1); old };
vm_block = 1424; continue;
}
// C line 18445
1431 => {
let _ = JS_FreeValue(ctx, *(sp).offset((((2 as i32)).wrapping_neg()) as isize));
// C line 18446
let _ = JS_FreeValue(ctx, *(sp).offset((((1 as i32)).wrapping_neg()) as isize));
// C line 18447
let _ = { sp = (sp).offset(-(((2 as i32)) as isize)); sp };
// C line 18448
let _ = { let assigned = ret_val; *({ let old = sp; sp = (sp).offset(1); old }) = assigned; assigned };
// C line 18450
vm_block = 30; continue;
}
// C line 18444
1432 => {
vm_block = 29; continue;
}
// C line 18443
1433 => {
vm_block = if ((((!(((!((JS_IsException(ret_val)) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 1432 } else { 1431 }; continue;
}
// C line 18442
1434 => {
let _ = free_arg_list(ctx, tab, len);
vm_block = 1433; continue;
}
// C line 18436
1435 => {
let _ = { let assigned = JS_EvalObject(ctx, JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }, obj_1, ((2 as i32)).wrapping_shl(((0 as i32)) as u32), scope_idx_1); ret_val = assigned; assigned };
vm_block = 1434; continue;
}
// C line 18433
1436 => {
let _ = { let assigned = *(tab).offset(((0 as i32)) as isize); obj_1 = assigned; assigned };
vm_block = 1435; continue;
}
// C line 18435
1437 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; obj_1 = assigned; assigned };
vm_block = 1435; continue;
}
// C line 18432
1438 => {
vm_block = if ((((len) >= ((((1 as i32)) as u32))) as i32)) != 0 { 1436 } else { 1437 }; continue;
}
// C line 18439
1439 => {
let _ = { let assigned = JS_Call(ctx, *(sp).offset((((2 as i32)).wrapping_neg()) as isize), JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }, ((len) as i32), tab); ret_val = assigned; assigned };
vm_block = 1434; continue;
}
// C line 18431
1440 => {
vm_block = if (js_same_value(ctx, *(sp).offset((((2 as i32)).wrapping_neg()) as isize), (*(ctx)).eval_obj)) != 0 { 1438 } else { 1439 }; continue;
}
// C line 18430
1441 => {
vm_block = 29; continue;
}
// C line 18429
1442 => {
vm_block = if ((!(!(tab).is_null()) as i32)) != 0 { 1441 } else { 1440 }; continue;
}
// C line 18425
1446 => {
let _ = { let assigned = (((crate::cutils_header::get_u16(pc)).wrapping_add(((((2 as i32)).wrapping_neg()) as u32))) as i32); scope_idx_1 = assigned; assigned };
// C line 18426
let _ = { pc = (pc).offset((((2 as i32)) as isize)); pc };
// C line 18427
let _ = { let assigned = pc; (*(sf)).cur_pc = assigned; assigned };
// C line 18428
let _ = { let assigned = build_arg_list(ctx, core::ptr::addr_of_mut!(len), *(sp).offset((((1 as i32)).wrapping_neg()) as isize)); tab = assigned; assigned };
vm_block = 1442; continue;
}
// C line 18413
1449 => {
let _ = { sp = (sp).offset(-(((call_argc).wrapping_add((1 as i32))) as isize)); sp };
// C line 18414
let _ = { let assigned = ret_val; *({ let old = sp; sp = (sp).offset(1); old }) = assigned; assigned };
// C line 18416
vm_block = 30; continue;
}
// C line 18411
1450 => {
vm_block = if ((((i) < (call_argc)) as i32)) != 0 { 1452 } else { 1449 }; continue;
}
// C line 18412
1452 => {
let _ = JS_FreeValue(ctx, *(call_argv).offset((i) as isize));
// C line ?
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 1450; continue;
}
// C line 18411
1453 => {
let _ = { let assigned = ((1 as i32)).wrapping_neg(); i = assigned; assigned };
vm_block = 1450; continue;
}
// C line 18410
1454 => {
vm_block = 29; continue;
}
// C line 18409
1455 => {
vm_block = if ((((!(((!((JS_IsException(ret_val)) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 1454 } else { 1453 }; continue;
}
// C line 18403
1456 => {
let _ = { let assigned = JS_EvalObject(ctx, JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }, obj, ((2 as i32)).wrapping_shl(((0 as i32)) as u32), scope_idx); ret_val = assigned; assigned };
vm_block = 1455; continue;
}
// C line 18400
1457 => {
let _ = { let assigned = *(call_argv).offset(((0 as i32)) as isize); obj = assigned; assigned };
vm_block = 1456; continue;
}
// C line 18402
1458 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; obj = assigned; assigned };
vm_block = 1456; continue;
}
// C line 18399
1459 => {
vm_block = if ((((call_argc) >= ((1 as i32))) as i32)) != 0 { 1457 } else { 1458 }; continue;
}
// C line 18406
1460 => {
let _ = { let assigned = JS_CallInternal(ctx, *(call_argv).offset((((1 as i32)).wrapping_neg()) as isize), JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }, JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }, call_argc, call_argv, (0 as i32)); ret_val = assigned; assigned };
vm_block = 1455; continue;
}
// C line 18398
1461 => {
vm_block = if (js_same_value(ctx, *(call_argv).offset((((1 as i32)).wrapping_neg()) as isize), (*(ctx)).eval_obj)) != 0 { 1459 } else { 1460 }; continue;
}
// C line 18393
1466 => {
let _ = { let assigned = ((crate::cutils_header::get_u16(pc)) as i32); call_argc = assigned; assigned };
// C line 18394
let _ = { let assigned = (((crate::cutils_header::get_u16((pc).offset((((2 as i32)) as isize)))).wrapping_add(((((2 as i32)).wrapping_neg()) as u32))) as i32); scope_idx = assigned; assigned };
// C line 18395
let _ = { pc = (pc).offset((((4 as i32)) as isize)); pc };
// C line 18396
let _ = { let assigned = (sp).offset(-((call_argc) as isize)); call_argv = assigned; assigned };
// C line 18397
let _ = { let assigned = pc; (*(sf)).cur_pc = assigned; assigned };
vm_block = 1461; continue;
}
// C line 18387
1467 => {
vm_block = 29; continue;
}
// C line 18371
1468 => {
let _ = JS_ThrowTypeErrorReadOnly(ctx, ((1 as i32)).wrapping_shl(((14 as i32)) as u32), atom);
vm_block = 1467; continue;
}
// C line 18374
1469 => {
let _ = JS_ThrowSyntaxErrorVarRedeclaration(ctx, atom);
vm_block = 1467; continue;
}
// C line 18377
1470 => {
let _ = JS_ThrowReferenceErrorUninitialized(ctx, atom);
vm_block = 1467; continue;
}
// C line 18380
1471 => {
let _ = JS_ThrowReferenceError(ctx, c"unsupported reference to 'super'".as_ptr());
vm_block = 1467; continue;
}
// C line 18383
1472 => {
let _ = JS_ThrowTypeError(ctx, c"iterator does not have a throw method".as_ptr());
vm_block = 1467; continue;
}
// C line 18385
1473 => {
let _ = JS_ThrowInternalError(ctx, format_args!("invalid throw var type {}", v_type));
vm_block = 1467; continue;
}
// C line 18382
1474 => {
vm_block = if ((((v_type) == ((4 as i32))) as i32)) != 0 { 1472 } else { 1473 }; continue;
}
// C line 18379
1475 => {
vm_block = if ((((v_type) == ((3 as i32))) as i32)) != 0 { 1471 } else { 1474 }; continue;
}
// C line 18376
1476 => {
vm_block = if ((((v_type) == ((2 as i32))) as i32)) != 0 { 1470 } else { 1475 }; continue;
}
// C line 18373
1477 => {
vm_block = if ((((v_type) == ((1 as i32))) as i32)) != 0 { 1469 } else { 1476 }; continue;
}
// C line 18370
1478 => {
vm_block = if ((((v_type) == ((0 as i32))) as i32)) != 0 { 1468 } else { 1477 }; continue;
}
// C line 18367
1481 => {
let _ = { let assigned = crate::cutils_header::get_u32(pc); atom = assigned; assigned };
// C line 18368
let _ = { let assigned = ((*(pc).offset(((4 as i32)) as isize)) as i32); v_type = assigned; assigned };
// C line 18369
let _ = { pc = (pc).offset((((5 as i32)) as isize)); pc };
vm_block = 1478; continue;
}
// C line 18355
1483 => {
sp = sp.offset(-1);
let _ = js_host_vm_throw(ctx, b, sf, pc, sp, *sp);
// C line 18356
vm_block = 29; continue;
}
// C line 18349
1487 => {
let _ = JS_FreeValue(ctx, *(sp).offset((((2 as i32)).wrapping_neg()) as isize));
// C line 18350
let _ = JS_FreeValue(ctx, *(sp).offset((((1 as i32)).wrapping_neg()) as isize));
// C line 18351
let _ = { sp = (sp).offset(-(((2 as i32)) as isize)); sp };
// C line 18352
vm_block = 30; continue;
}
// C line 18348
1488 => {
vm_block = 29; continue;
}
// C line 18347
1489 => {
vm_block = if ((((JS_AddBrand(ctx, *(sp).offset((((2 as i32)).wrapping_neg()) as isize), *(sp).offset((((1 as i32)).wrapping_neg()) as isize))) < ((0 as i32))) as i32)) != 0 { 1488 } else { 1487 }; continue;
}
// C line 18345
1490 => {
vm_block = 30; continue;
}
// C line 18341
1492 => {
let _ = JS_ThrowTypeError(ctx, c"invalid brand on object".as_ptr());
// C line 18342
vm_block = 29; continue;
}
// C line 18340
1493 => {
vm_block = if ((!((ret_1) != 0) as i32)) != 0 { 1492 } else { 1490 }; continue;
}
// C line 18339
1494 => {
vm_block = 29; continue;
}
// C line 18338
1495 => {
vm_block = if ((((ret_1) < ((0 as i32))) as i32)) != 0 { 1494 } else { 1493 }; continue;
}
// C line 18337
1496 => {
ret_1 = JS_CheckBrand(ctx, *(sp).offset((((2 as i32)).wrapping_neg()) as isize), *(sp).offset((((1 as i32)).wrapping_neg()) as isize));
vm_block = 1495; continue;
}
// C line 18332
1498 => {
let _ = { let assigned = ret; *({ let old = sp; sp = (sp).offset(1); old }) = assigned; assigned };
// C line 18334
vm_block = 30; continue;
}
// C line 18331
1499 => {
vm_block = 29; continue;
}
// C line 18330
1500 => {
vm_block = if (JS_IsException(ret)) != 0 { 1499 } else { 1498 }; continue;
}
// C line 18328
1502 => {
let _ = { let assigned = JS_CallConstructor2(ctx, v_super, new_target, argc, argv); ret = assigned; assigned };
// C line 18329
let _ = JS_FreeValue(ctx, v_super);
vm_block = 1500; continue;
}
// C line 18327
1503 => {
vm_block = 29; continue;
}
// C line 18326
1504 => {
vm_block = if (JS_IsException(v_super)) != 0 { 1503 } else { 1502 }; continue;
}
// C line 18325
1505 => {
let _ = { let assigned = JS_GetPrototype(ctx, func_obj); v_super = assigned; assigned };
vm_block = 1504; continue;
}
// C line 18324
1506 => {
vm_block = 1511; continue;
}
// C line 18323
1507 => {
vm_block = if (JS_IsUndefined(new_target)) != 0 { 1506 } else { 1505 }; continue;
}
// C line 18322
1508 => {
let _ = { let assigned = pc; (*(sf)).cur_pc = assigned; assigned };
vm_block = 1507; continue;
}
// C line 18318
1509 => {
vm_block = 30; continue;
}
// C line ? labels: non_ctor_call
1511 => {
let _ = JS_ThrowTypeError(ctx, c"class constructors must be invoked with 'new'".as_ptr());
// C line 18316
vm_block = 29; continue;
}
// C line 18313
1512 => {
vm_block = if (JS_IsUndefined(new_target)) != 0 { 1511 } else { 1509 }; continue;
}
// C line 18310
1514 => {
let _ = { let old = sp; sp = (sp).offset(1); old };
// C line 18311
vm_block = 30; continue;
}
// C line 18306
1515 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((1 as i32)) as u32)) as u64) }, tag: (((JS_TAG_BOOL as i32)) as i64) }; *(sp).offset(((0 as i32)) as isize) = assigned; assigned };
vm_block = 1514; continue;
}
// C line 18303
1517 => {
let _ = JS_ThrowTypeError(caller_ctx, c"derived class constructor must return an object or undefined".as_ptr());
// C line 18304
vm_block = 29; continue;
}
// C line 18302
1518 => {
vm_block = if ((!((JS_IsUndefined(*(sp).offset((((1 as i32)).wrapping_neg()) as isize))) != 0) as i32)) != 0 { 1517 } else { 1515 }; continue;
}
// C line 18308
1519 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_BOOL as i32)) as i64) }; *(sp).offset(((0 as i32)) as isize) = assigned; assigned };
vm_block = 1514; continue;
}
// C line 18301
1520 => {
vm_block = if ((!((JS_IsObject(*(sp).offset((((1 as i32)).wrapping_neg()) as isize))) != 0) as i32)) != 0 { 1518 } else { 1519 }; continue;
}
// C line 18296
1522 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; ret_val = assigned; assigned };
// C line 18297
vm_block = 10; continue;
}
// C line 18293
1524 => {
let _ = { let assigned = *({ sp = (sp).offset(-1); sp }); ret_val = assigned; assigned };
// C line 18294
vm_block = 10; continue;
}
// C line 18285
1530 => {
let _ = JS_FreeValue(ctx, *(sp).offset((((3 as i32)).wrapping_neg()) as isize));
// C line 18286
let _ = JS_FreeValue(ctx, *(sp).offset((((2 as i32)).wrapping_neg()) as isize));
// C line 18287
let _ = JS_FreeValue(ctx, *(sp).offset((((1 as i32)).wrapping_neg()) as isize));
// C line 18288
let _ = { sp = (sp).offset(-(((3 as i32)) as isize)); sp };
// C line 18289
let _ = { let assigned = ret_val; *({ let old = sp; sp = (sp).offset(1); old }) = assigned; assigned };
// C line 18291
vm_block = 30; continue;
}
// C line 18284
1531 => {
vm_block = 29; continue;
}
// C line 18283
1532 => {
vm_block = if ((((!(((!((JS_IsException(ret_val)) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 1531 } else { 1530 }; continue;
}
// C line 18278
1536 => {
let _ = { let assigned = ((crate::cutils_header::get_u16(pc)) as i32); magic = assigned; assigned };
// C line 18279
let _ = { pc = (pc).offset((((2 as i32)) as isize)); pc };
// C line 18280
let _ = { let assigned = pc; (*(sf)).cur_pc = assigned; assigned };
// C line 18282
let _ = { let assigned = js_function_apply(ctx, *(sp).offset((((3 as i32)).wrapping_neg()) as isize), (2 as i32), core::ptr::addr_of_mut!(*(sp).offset((((2 as i32)).wrapping_neg()) as isize)), magic); ret_val = assigned; assigned };
vm_block = 1532; continue;
}
// C line 18272
1538 => {
let _ = { let assigned = ret_val; *({ let old = sp; sp = (sp).offset(1); old }) = assigned; assigned };
// C line 18273
vm_block = 30; continue;
}
// C line 18271
1539 => {
vm_block = 29; continue;
}
// C line 18270
1540 => {
vm_block = if ((((!(((!((JS_IsException(ret_val)) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 1539 } else { 1538 }; continue;
}
// C line 18266
1544 => {
let _ = { let assigned = ((crate::cutils_header::get_u16(pc)) as i32); call_argc = assigned; assigned };
// C line 18267
let _ = { pc = (pc).offset((((2 as i32)) as isize)); pc };
// C line 18268
let _ = { let assigned = js_create_array_free(ctx, call_argc, (sp).offset(-((call_argc) as isize))); ret_val = assigned; assigned };
// C line 18269
let _ = { sp = (sp).offset(-((call_argc) as isize)); sp };
vm_block = 1540; continue;
}
// C line 18261
1547 => {
let _ = { sp = (sp).offset(-(((call_argc).wrapping_add((2 as i32))) as isize)); sp };
// C line 18262
let _ = { let assigned = ret_val; *({ let old = sp; sp = (sp).offset(1); old }) = assigned; assigned };
// C line 18264
vm_block = 30; continue;
}
// C line 18259
1548 => {
vm_block = if ((((i) < (call_argc)) as i32)) != 0 { 1550 } else { 1547 }; continue;
}
// C line 18260
1550 => {
let _ = JS_FreeValue(ctx, *(call_argv).offset((i) as isize));
// C line ?
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 1548; continue;
}
// C line 18259
1551 => {
let _ = { let assigned = ((2 as i32)).wrapping_neg(); i = assigned; assigned };
vm_block = 1548; continue;
}
// C line 18258
1552 => {
vm_block = 10; continue;
}
// C line 18257
1553 => {
vm_block = if ((((opcode) == ((OP_tail_call_method as i32))) as i32)) != 0 { 1552 } else { 1551 }; continue;
}
// C line 18256
1554 => {
vm_block = 29; continue;
}
// C line 18255
1555 => {
vm_block = if ((((!(((!((JS_IsException(ret_val)) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 1554 } else { 1553 }; continue;
}
// C line 18249
1560 => {
let _ = { let assigned = ((crate::cutils_header::get_u16(pc)) as i32); call_argc = assigned; assigned };
// C line 18250
let _ = { pc = (pc).offset((((2 as i32)) as isize)); pc };
// C line 18251
let _ = { let assigned = (sp).offset(-((call_argc) as isize)); call_argv = assigned; assigned };
// C line 18252
let _ = { let assigned = pc; (*(sf)).cur_pc = assigned; assigned };
// C line 18253
let _ = { let assigned = JS_CallInternal(ctx, *(call_argv).offset((((1 as i32)).wrapping_neg()) as isize), *(call_argv).offset((((2 as i32)).wrapping_neg()) as isize), JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }, call_argc, call_argv, (0 as i32)); ret_val = assigned; assigned };
vm_block = 1555; continue;
}
// C line 18242
1563 => {
let _ = { sp = (sp).offset(-(((call_argc).wrapping_add((2 as i32))) as isize)); sp };
// C line 18243
let _ = { let assigned = ret_val; *({ let old = sp; sp = (sp).offset(1); old }) = assigned; assigned };
// C line 18245
vm_block = 30; continue;
}
// C line 18240
1564 => {
vm_block = if ((((i) < (call_argc)) as i32)) != 0 { 1566 } else { 1563 }; continue;
}
// C line 18241
1566 => {
let _ = JS_FreeValue(ctx, *(call_argv).offset((i) as isize));
// C line ?
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 1564; continue;
}
// C line 18240
1567 => {
let _ = { let assigned = ((2 as i32)).wrapping_neg(); i = assigned; assigned };
vm_block = 1564; continue;
}
// C line 18239
1568 => {
vm_block = 29; continue;
}
// C line 18238
1569 => {
vm_block = if ((((!(((!((JS_IsException(ret_val)) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 1568 } else { 1567 }; continue;
}
// C line 18231
1574 => {
let _ = { let assigned = ((crate::cutils_header::get_u16(pc)) as i32); call_argc = assigned; assigned };
// C line 18232
let _ = { pc = (pc).offset((((2 as i32)) as isize)); pc };
// C line 18233
let _ = { let assigned = (sp).offset(-((call_argc) as isize)); call_argv = assigned; assigned };
// C line 18234
let _ = { let assigned = pc; (*(sf)).cur_pc = assigned; assigned };
// C line 18235
let _ = { let assigned = JS_CallConstructorInternal(ctx, *(call_argv).offset((((2 as i32)).wrapping_neg()) as isize), *(call_argv).offset((((1 as i32)).wrapping_neg()) as isize), call_argc, call_argv, (0 as i32)); ret_val = assigned; assigned };
vm_block = 1569; continue;
}
// C line 18225
1577 => {
let _ = { sp = (sp).offset(-(((call_argc).wrapping_add((1 as i32))) as isize)); sp };
// C line 18226
let _ = { let assigned = ret_val; *({ let old = sp; sp = (sp).offset(1); old }) = assigned; assigned };
// C line 18228
vm_block = 30; continue;
}
// C line 18223
1578 => {
vm_block = if ((((i) < (call_argc)) as i32)) != 0 { 1580 } else { 1577 }; continue;
}
// C line 18224
1580 => {
let _ = JS_FreeValue(ctx, *(call_argv).offset((i) as isize));
// C line ?
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 1578; continue;
}
// C line 18223
1581 => {
let _ = { let assigned = ((1 as i32)).wrapping_neg(); i = assigned; assigned };
vm_block = 1578; continue;
}
// C line 18222
1582 => {
vm_block = 10; continue;
}
// C line 18221
1583 => {
vm_block = if ((((opcode) == ((OP_tail_call as i32))) as i32)) != 0 { 1582 } else { 1581 }; continue;
}
// C line 18220
1584 => {
vm_block = 29; continue;
}
// C line 18219
1585 => {
vm_block = if ((((!(((!((JS_IsException(ret_val)) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 1584 } else { 1583 }; continue;
}
// C line ? labels: has_call_argc
1588 => {
let _ = { let assigned = (sp).offset(-((call_argc) as isize)); call_argv = assigned; assigned };
// C line 18216
let _ = { let assigned = pc; (*(sf)).cur_pc = assigned; assigned };
// C line 18217
let _ = { let assigned = JS_CallInternal(ctx, *(call_argv).offset((((1 as i32)).wrapping_neg()) as isize), JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }, JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }, call_argc, call_argv, (0 as i32)); ret_val = assigned; assigned };
vm_block = 1585; continue;
}
// C line 18211
1591 => {
let _ = { let assigned = ((crate::cutils_header::get_u16(pc)) as i32); call_argc = assigned; assigned };
// C line 18212
let _ = { pc = (pc).offset((((2 as i32)) as isize)); pc };
// C line 18213
vm_block = 1588; continue;
}
// C line 18205
1593 => {
let _ = { let assigned = (opcode).wrapping_sub((OP_call0 as i32)); call_argc = assigned; assigned };
// C line 18206
vm_block = 1588; continue;
}
// C line 18199
1594 => {
vm_block = 30; continue;
}
// C line 18197
1595 => {
vm_block = 29; continue;
}
// C line 18196
1596 => {
vm_block = if ((((!(((!((JS_IsException(*(sp).offset((((1 as i32)).wrapping_neg()) as isize))) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 1595 } else { 1594 }; continue;
}
// C line 18193
1599 => {
bfunc = JS_DupValue(ctx, *((*(b)).cpool).offset((crate::cutils_header::get_u32(pc)) as isize));
// C line 18194
let _ = { pc = (pc).offset((((4 as i32)) as isize)); pc };
// C line 18195
let _ = { let assigned = js_closure(ctx, bfunc, var_refs, sf, (0 as i32)); *({ let old = sp; sp = (sp).offset(1); old }) = assigned; assigned };
vm_block = 1596; continue;
}
// C line 18182
1606 => {
let _ = { let assigned = *(sp).offset((((4 as i32)).wrapping_neg()) as isize); tmp1 = assigned; assigned };
// C line 18183
let _ = { let assigned = *(sp).offset((((3 as i32)).wrapping_neg()) as isize); tmp2 = assigned; assigned };
// C line 18184
let _ = { let assigned = *(sp).offset((((2 as i32)).wrapping_neg()) as isize); *(sp).offset((((4 as i32)).wrapping_neg()) as isize) = assigned; assigned };
// C line 18185
let _ = { let assigned = *(sp).offset((((1 as i32)).wrapping_neg()) as isize); *(sp).offset((((3 as i32)).wrapping_neg()) as isize) = assigned; assigned };
// C line 18186
let _ = { let assigned = tmp1; *(sp).offset((((2 as i32)).wrapping_neg()) as isize) = assigned; assigned };
// C line 18187
let _ = { let assigned = tmp2; *(sp).offset((((1 as i32)).wrapping_neg()) as isize) = assigned; assigned };
// C line 18189
vm_block = 30; continue;
}
// C line 18174
1610 => {
let _ = { let assigned = *(sp).offset((((2 as i32)).wrapping_neg()) as isize); tmp_7 = assigned; assigned };
// C line 18175
let _ = { let assigned = *(sp).offset((((1 as i32)).wrapping_neg()) as isize); *(sp).offset((((2 as i32)).wrapping_neg()) as isize) = assigned; assigned };
// C line 18176
let _ = { let assigned = tmp_7; *(sp).offset((((1 as i32)).wrapping_neg()) as isize) = assigned; assigned };
// C line 18178
vm_block = 30; continue;
}
// C line 18164
1616 => {
let _ = { let assigned = *(sp).offset((((2 as i32)).wrapping_neg()) as isize); tmp_6 = assigned; assigned };
// C line 18165
let _ = { let assigned = *(sp).offset((((3 as i32)).wrapping_neg()) as isize); *(sp).offset((((2 as i32)).wrapping_neg()) as isize) = assigned; assigned };
// C line 18166
let _ = { let assigned = *(sp).offset((((4 as i32)).wrapping_neg()) as isize); *(sp).offset((((3 as i32)).wrapping_neg()) as isize) = assigned; assigned };
// C line 18167
let _ = { let assigned = *(sp).offset((((5 as i32)).wrapping_neg()) as isize); *(sp).offset((((4 as i32)).wrapping_neg()) as isize) = assigned; assigned };
// C line 18168
let _ = { let assigned = tmp_6; *(sp).offset((((5 as i32)).wrapping_neg()) as isize) = assigned; assigned };
// C line 18170
vm_block = 30; continue;
}
// C line 18155
1621 => {
let _ = { let assigned = *(sp).offset((((2 as i32)).wrapping_neg()) as isize); tmp_5 = assigned; assigned };
// C line 18156
let _ = { let assigned = *(sp).offset((((3 as i32)).wrapping_neg()) as isize); *(sp).offset((((2 as i32)).wrapping_neg()) as isize) = assigned; assigned };
// C line 18157
let _ = { let assigned = *(sp).offset((((4 as i32)).wrapping_neg()) as isize); *(sp).offset((((3 as i32)).wrapping_neg()) as isize) = assigned; assigned };
// C line 18158
let _ = { let assigned = tmp_5; *(sp).offset((((4 as i32)).wrapping_neg()) as isize) = assigned; assigned };
// C line 18160
vm_block = 30; continue;
}
// C line 18146
1626 => {
let _ = { let assigned = *(sp).offset((((1 as i32)).wrapping_neg()) as isize); tmp_4 = assigned; assigned };
// C line 18147
let _ = { let assigned = *(sp).offset((((2 as i32)).wrapping_neg()) as isize); *(sp).offset((((1 as i32)).wrapping_neg()) as isize) = assigned; assigned };
// C line 18148
let _ = { let assigned = *(sp).offset((((3 as i32)).wrapping_neg()) as isize); *(sp).offset((((2 as i32)).wrapping_neg()) as isize) = assigned; assigned };
// C line 18149
let _ = { let assigned = tmp_4; *(sp).offset((((3 as i32)).wrapping_neg()) as isize) = assigned; assigned };
// C line 18151
vm_block = 30; continue;
}
// C line 18135
1633 => {
let _ = { let assigned = *(sp).offset((((5 as i32)).wrapping_neg()) as isize); tmp_3 = assigned; assigned };
// C line 18136
let _ = { let assigned = *(sp).offset((((4 as i32)).wrapping_neg()) as isize); *(sp).offset((((5 as i32)).wrapping_neg()) as isize) = assigned; assigned };
// C line 18137
let _ = { let assigned = *(sp).offset((((3 as i32)).wrapping_neg()) as isize); *(sp).offset((((4 as i32)).wrapping_neg()) as isize) = assigned; assigned };
// C line 18138
let _ = { let assigned = *(sp).offset((((2 as i32)).wrapping_neg()) as isize); *(sp).offset((((3 as i32)).wrapping_neg()) as isize) = assigned; assigned };
// C line 18139
let _ = { let assigned = *(sp).offset((((1 as i32)).wrapping_neg()) as isize); *(sp).offset((((2 as i32)).wrapping_neg()) as isize) = assigned; assigned };
// C line 18140
let _ = { let assigned = tmp_3; *(sp).offset((((1 as i32)).wrapping_neg()) as isize) = assigned; assigned };
// C line 18142
vm_block = 30; continue;
}
// C line 18125
1639 => {
let _ = { let assigned = *(sp).offset((((4 as i32)).wrapping_neg()) as isize); tmp_2 = assigned; assigned };
// C line 18126
let _ = { let assigned = *(sp).offset((((3 as i32)).wrapping_neg()) as isize); *(sp).offset((((4 as i32)).wrapping_neg()) as isize) = assigned; assigned };
// C line 18127
let _ = { let assigned = *(sp).offset((((2 as i32)).wrapping_neg()) as isize); *(sp).offset((((3 as i32)).wrapping_neg()) as isize) = assigned; assigned };
// C line 18128
let _ = { let assigned = *(sp).offset((((1 as i32)).wrapping_neg()) as isize); *(sp).offset((((2 as i32)).wrapping_neg()) as isize) = assigned; assigned };
// C line 18129
let _ = { let assigned = tmp_2; *(sp).offset((((1 as i32)).wrapping_neg()) as isize) = assigned; assigned };
// C line 18131
vm_block = 30; continue;
}
// C line 18116
1644 => {
let _ = { let assigned = *(sp).offset((((3 as i32)).wrapping_neg()) as isize); tmp_1 = assigned; assigned };
// C line 18117
let _ = { let assigned = *(sp).offset((((2 as i32)).wrapping_neg()) as isize); *(sp).offset((((3 as i32)).wrapping_neg()) as isize) = assigned; assigned };
// C line 18118
let _ = { let assigned = *(sp).offset((((1 as i32)).wrapping_neg()) as isize); *(sp).offset((((2 as i32)).wrapping_neg()) as isize) = assigned; assigned };
// C line 18119
let _ = { let assigned = tmp_1; *(sp).offset((((1 as i32)).wrapping_neg()) as isize) = assigned; assigned };
// C line 18121
vm_block = 30; continue;
}
// C line 18108
1648 => {
let _ = { let assigned = *(sp).offset((((2 as i32)).wrapping_neg()) as isize); tmp = assigned; assigned };
// C line 18109
let _ = { let assigned = *(sp).offset((((3 as i32)).wrapping_neg()) as isize); *(sp).offset((((2 as i32)).wrapping_neg()) as isize) = assigned; assigned };
// C line 18110
let _ = { let assigned = tmp; *(sp).offset((((3 as i32)).wrapping_neg()) as isize) = assigned; assigned };
// C line 18112
vm_block = 30; continue;
}
// C line 18098
1655 => {
let _ = { let assigned = *(sp).offset((((1 as i32)).wrapping_neg()) as isize); *(sp).offset(((0 as i32)) as isize) = assigned; assigned };
// C line 18099
let _ = { let assigned = *(sp).offset((((2 as i32)).wrapping_neg()) as isize); *(sp).offset((((1 as i32)).wrapping_neg()) as isize) = assigned; assigned };
// C line 18100
let _ = { let assigned = *(sp).offset((((3 as i32)).wrapping_neg()) as isize); *(sp).offset((((2 as i32)).wrapping_neg()) as isize) = assigned; assigned };
// C line 18101
let _ = { let assigned = *(sp).offset((((4 as i32)).wrapping_neg()) as isize); *(sp).offset((((3 as i32)).wrapping_neg()) as isize) = assigned; assigned };
// C line 18102
let _ = { let assigned = JS_DupValue(ctx, *(sp).offset(((0 as i32)) as isize)); *(sp).offset((((4 as i32)).wrapping_neg()) as isize) = assigned; assigned };
// C line 18103
let _ = { let old = sp; sp = (sp).offset(1); old };
// C line 18104
vm_block = 30; continue;
}
// C line 18091
1661 => {
let _ = { let assigned = *(sp).offset((((1 as i32)).wrapping_neg()) as isize); *(sp).offset(((0 as i32)) as isize) = assigned; assigned };
// C line 18092
let _ = { let assigned = *(sp).offset((((2 as i32)).wrapping_neg()) as isize); *(sp).offset((((1 as i32)).wrapping_neg()) as isize) = assigned; assigned };
// C line 18093
let _ = { let assigned = *(sp).offset((((3 as i32)).wrapping_neg()) as isize); *(sp).offset((((2 as i32)).wrapping_neg()) as isize) = assigned; assigned };
// C line 18094
let _ = { let assigned = JS_DupValue(ctx, *(sp).offset(((0 as i32)) as isize)); *(sp).offset((((3 as i32)).wrapping_neg()) as isize) = assigned; assigned };
// C line 18095
let _ = { let old = sp; sp = (sp).offset(1); old };
// C line 18096
vm_block = 30; continue;
}
// C line 18085
1666 => {
let _ = { let assigned = *(sp).offset((((1 as i32)).wrapping_neg()) as isize); *(sp).offset(((0 as i32)) as isize) = assigned; assigned };
// C line 18086
let _ = { let assigned = *(sp).offset((((2 as i32)).wrapping_neg()) as isize); *(sp).offset((((1 as i32)).wrapping_neg()) as isize) = assigned; assigned };
// C line 18087
let _ = { let assigned = JS_DupValue(ctx, *(sp).offset(((0 as i32)) as isize)); *(sp).offset((((2 as i32)).wrapping_neg()) as isize) = assigned; assigned };
// C line 18088
let _ = { let old = sp; sp = (sp).offset(1); old };
// C line 18089
vm_block = 30; continue;
}
// C line 18080
1670 => {
let _ = { let assigned = *(sp).offset((((1 as i32)).wrapping_neg()) as isize); *(sp).offset(((0 as i32)) as isize) = assigned; assigned };
// C line 18081
let _ = { let assigned = JS_DupValue(ctx, *(sp).offset((((2 as i32)).wrapping_neg()) as isize)); *(sp).offset((((1 as i32)).wrapping_neg()) as isize) = assigned; assigned };
// C line 18082
let _ = { let old = sp; sp = (sp).offset(1); old };
// C line 18083
vm_block = 30; continue;
}
// C line 18074
1675 => {
let _ = { let assigned = JS_DupValue(ctx, *(sp).offset((((3 as i32)).wrapping_neg()) as isize)); *(sp).offset(((0 as i32)) as isize) = assigned; assigned };
// C line 18075
let _ = { let assigned = JS_DupValue(ctx, *(sp).offset((((2 as i32)).wrapping_neg()) as isize)); *(sp).offset(((1 as i32)) as isize) = assigned; assigned };
// C line 18076
let _ = { let assigned = JS_DupValue(ctx, *(sp).offset((((1 as i32)).wrapping_neg()) as isize)); *(sp).offset(((2 as i32)) as isize) = assigned; assigned };
// C line 18077
let _ = { sp = (sp).offset((((3 as i32)) as isize)); sp };
// C line 18078
vm_block = 30; continue;
}
// C line 18069
1679 => {
let _ = { let assigned = JS_DupValue(ctx, *(sp).offset((((2 as i32)).wrapping_neg()) as isize)); *(sp).offset(((0 as i32)) as isize) = assigned; assigned };
// C line 18070
let _ = { let assigned = JS_DupValue(ctx, *(sp).offset((((1 as i32)).wrapping_neg()) as isize)); *(sp).offset(((1 as i32)) as isize) = assigned; assigned };
// C line 18071
let _ = { sp = (sp).offset((((2 as i32)) as isize)); sp };
// C line 18072
vm_block = 30; continue;
}
// C line 18065
1682 => {
let _ = { let assigned = JS_DupValue(ctx, *(sp).offset((((1 as i32)).wrapping_neg()) as isize)); *(sp).offset(((0 as i32)) as isize) = assigned; assigned };
// C line 18066
let _ = { let old = sp; sp = (sp).offset(1); old };
// C line 18067
vm_block = 30; continue;
}
// C line 18059
1687 => {
let _ = JS_FreeValue(ctx, *(sp).offset((((3 as i32)).wrapping_neg()) as isize));
// C line 18060
let _ = { let assigned = *(sp).offset((((2 as i32)).wrapping_neg()) as isize); *(sp).offset((((3 as i32)).wrapping_neg()) as isize) = assigned; assigned };
// C line 18061
let _ = { let assigned = *(sp).offset((((1 as i32)).wrapping_neg()) as isize); *(sp).offset((((2 as i32)).wrapping_neg()) as isize) = assigned; assigned };
// C line 18062
let _ = { let old = sp; sp = (sp).offset(-1); old };
// C line 18063
vm_block = 30; continue;
}
// C line 18054
1691 => {
let _ = JS_FreeValue(ctx, *(sp).offset((((2 as i32)).wrapping_neg()) as isize));
// C line 18055
let _ = { let assigned = *(sp).offset((((1 as i32)).wrapping_neg()) as isize); *(sp).offset((((2 as i32)).wrapping_neg()) as isize) = assigned; assigned };
// C line 18056
let _ = { let old = sp; sp = (sp).offset(-1); old };
// C line 18057
vm_block = 30; continue;
}
// C line 18050
1694 => {
let _ = JS_FreeValue(ctx, *(sp).offset((((1 as i32)).wrapping_neg()) as isize));
// C line 18051
let _ = { let old = sp; sp = (sp).offset(-1); old };
// C line 18052
vm_block = 30; continue;
}
// C line 18047
1695 => {
vm_block = 30; continue;
}
// C line 18045
1696 => {
vm_block = 29; continue;
}
// C line 18044
1697 => {
vm_block = if ((((!(((!((JS_IsException(*(sp).offset((((1 as i32)).wrapping_neg()) as isize))) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 1696 } else { 1695 }; continue;
}
// C line 18040
1701 => {
first = ((crate::cutils_header::get_u16(pc)) as i32);
// C line 18041
let _ = { pc = (pc).offset((((2 as i32)) as isize)); pc };
// C line 18042
let _ = { let assigned = min_int(first, argc); first = assigned; assigned };
// C line 18043
let _ = { let assigned = js_create_array(ctx, (argc).wrapping_sub(first), (argv).offset(((first) as isize))); *({ let old = sp; sp = (sp).offset(1); old }) = assigned; assigned };
vm_block = 1697; continue;
}
// C line 18037
1702 => {
vm_block = 30; continue;
}
// C line ?
1703 => {
let _ = std::process::abort();
vm_block = 1702; continue;
}
// C line 18032
1704 => {
vm_block = 1702; continue;
}
// C line 18031
1705 => {
vm_block = 29; continue;
}
// C line 18030
1706 => {
vm_block = if ((((!(((!((JS_IsException(*(sp).offset((((1 as i32)).wrapping_neg()) as isize))) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 1705 } else { 1704 }; continue;
}
// C line 18029
1707 => {
let _ = { let assigned = js_import_meta(ctx); *({ let old = sp; sp = (sp).offset(1); old }) = assigned; assigned };
vm_block = 1706; continue;
}
// C line 18027
1708 => {
vm_block = 1702; continue;
}
// C line 18026
1709 => {
vm_block = 29; continue;
}
// C line 18025
1710 => {
vm_block = if ((((!(((!((JS_IsException(*(sp).offset((((1 as i32)).wrapping_neg()) as isize))) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 1709 } else { 1708 }; continue;
}
// C line 18024
1711 => {
let _ = { let assigned = JS_NewObjectProto(ctx, JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_NULL as i32)) as i64) }); *({ let old = sp; sp = (sp).offset(1); old }) = assigned; assigned };
vm_block = 1710; continue;
}
// C line 18022
1712 => {
vm_block = 1702; continue;
}
// C line 18018
1713 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; *({ let old = sp; sp = (sp).offset(1); old }) = assigned; assigned };
vm_block = 1712; continue;
}
// C line 18020
1714 => {
let _ = { let assigned = JS_DupValue(ctx, JSValue { u: JSValueUnion { ptr: ((p1) as *mut c_void) }, tag: (((JS_TAG_OBJECT as i32)) as i64) }); *({ let old = sp; sp = (sp).offset(1); old }) = assigned; assigned };
vm_block = 1712; continue;
}
// C line 18017
1715 => {
vm_block = if ((((!(((!(((!(!(p1).is_null()) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 1713 } else { 1714 }; continue;
}
// C line 18016
1716 => {
let _ = { let assigned = (((*(p)).u).func).home_object; p1 = assigned; assigned };
vm_block = 1715; continue;
}
// C line 18011
1718 => {
let _ = { let assigned = JS_DupValue(ctx, new_target); *({ let old = sp; sp = (sp).offset(1); old }) = assigned; assigned };
// C line 18012
vm_block = 1702; continue;
}
// C line 18008
1720 => {
let _ = { let assigned = JS_DupValue(ctx, (*(sf)).cur_func); *({ let old = sp; sp = (sp).offset(1); old }) = assigned; assigned };
// C line 18009
vm_block = 1702; continue;
}
// C line 18006
1721 => {
vm_block = 1702; continue;
}
// C line 18005
1722 => {
vm_block = 29; continue;
}
// C line 18004
1723 => {
vm_block = if ((((!(((!((JS_IsException(*(sp).offset((((1 as i32)).wrapping_neg()) as isize))) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 1722 } else { 1721 }; continue;
}
// C line 18002
1724 => {
let _ = { let assigned = js_build_mapped_arguments(ctx, argc, argv, sf, min_int(argc, (((*(b)).arg_count) as i32))); *({ let old = sp; sp = (sp).offset(1); old }) = assigned; assigned };
vm_block = 1723; continue;
}
// C line 18000
1725 => {
vm_block = 1702; continue;
}
// C line 17999
1726 => {
vm_block = 29; continue;
}
// C line 17998
1727 => {
vm_block = if ((((!(((!((JS_IsException(*(sp).offset((((1 as i32)).wrapping_neg()) as isize))) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 1726 } else { 1725 }; continue;
}
// C line 17997
1728 => {
let _ = { let assigned = js_build_arguments(ctx, argc, argv); *({ let old = sp; sp = (sp).offset(1); old }) = assigned; assigned };
vm_block = 1727; continue;
}
// C line 17995
1729 => {
vm_block = match arg { x if x == (OP_SPECIAL_OBJECT_IMPORT_META as i32) => 1707, x if x == (OP_SPECIAL_OBJECT_VAR_OBJECT as i32) => 1711, x if x == (OP_SPECIAL_OBJECT_HOME_OBJECT as i32) => 1716, x if x == (OP_SPECIAL_OBJECT_NEW_TARGET as i32) => 1718, x if x == (OP_SPECIAL_OBJECT_THIS_FUNC as i32) => 1720, x if x == (OP_SPECIAL_OBJECT_MAPPED_ARGUMENTS as i32) => 1724, x if x == (OP_SPECIAL_OBJECT_ARGUMENTS as i32) => 1728, _ => 1703, }; continue;
}
// C line 17994
1730 => {
arg = ((*({ let old = pc; pc = (pc).offset(1); old })) as i32);
vm_block = 1729; continue;
}
// C line 17991
1731 => {
vm_block = 30; continue;
}
// C line 17990
1732 => {
vm_block = 29; continue;
}
// C line 17989
1733 => {
vm_block = if ((((!(((!((JS_IsException(*(sp).offset((((1 as i32)).wrapping_neg()) as isize))) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 1732 } else { 1731 }; continue;
}
// C line 17988
1734 => {
let _ = { let assigned = JS_NewObject(ctx); *({ let old = sp; sp = (sp).offset(1); old }) = assigned; assigned };
vm_block = 1733; continue;
}
// C line 17985
1736 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((1 as i32)) as u32)) as u64) }, tag: (((JS_TAG_BOOL as i32)) as i64) }; *({ let old = sp; sp = (sp).offset(1); old }) = assigned; assigned };
// C line 17986
vm_block = 30; continue;
}
// C line 17982
1738 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_BOOL as i32)) as i64) }; *({ let old = sp; sp = (sp).offset(1); old }) = assigned; assigned };
// C line 17983
vm_block = 30; continue;
}
// C line 17978
1740 => {
let _ = { let assigned = val; *({ let old = sp; sp = (sp).offset(1); old }) = assigned; assigned };
// C line 17980
vm_block = 30; continue;
}
// C line 17968
1741 => {
let _ = { let assigned = JS_DupValue(ctx, (*(ctx)).global_obj); val = assigned; assigned };
vm_block = 1740; continue;
}
// C line 17972
1742 => {
vm_block = 29; continue;
}
// C line 17971
1743 => {
vm_block = if (JS_IsException(val)) != 0 { 1742 } else { 1740 }; continue;
}
// C line 17970
1744 => {
let _ = { let assigned = JS_ToObject(ctx, this_obj); val = assigned; assigned };
vm_block = 1743; continue;
}
// C line 17967
1745 => {
vm_block = if ((((((((tag) == ((((JS_TAG_NULL as i32)) as u32))) as i32)) != 0) || (((((tag) == ((((JS_TAG_UNDEFINED as i32)) as u32))) as i32)) != 0)) as i32)) != 0 { 1741 } else { 1744 }; continue;
}
// C line 17966
1746 => {
vm_block = 1749; continue;
}
// C line 17965
1747 => {
vm_block = if ((((!(((!(((((tag) == ((((JS_TAG_OBJECT as i32)) as u32))) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 1746 } else { 1745 }; continue;
}
// C line 17964
1748 => {
tag = (((((this_obj).tag) as i32)) as u32);
vm_block = 1747; continue;
}
// C line ? labels: normal_this
1749 => {
let _ = { let assigned = JS_DupValue(ctx, this_obj); val = assigned; assigned };
vm_block = 1740; continue;
}
// C line 17963
1750 => {
vm_block = if ((!(((((((*(b)).js_mode) as i32)) & (((1 as i32)).wrapping_shl(((0 as i32)) as u32)))) != 0) as i32)) != 0 { 1748 } else { 1749 }; continue;
}
// C line 17957
1752 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_NULL as i32)) as i64) }; *({ let old = sp; sp = (sp).offset(1); old }) = assigned; assigned };
// C line 17958
vm_block = 30; continue;
}
// C line 17954
1754 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; *({ let old = sp; sp = (sp).offset(1); old }) = assigned; assigned };
// C line 17955
vm_block = 30; continue;
}
// C line 17950
1757 => {
let _ = { let assigned = JS_AtomToValue(ctx, crate::cutils_header::get_u32(pc)); *({ let old = sp; sp = (sp).offset(1); old }) = assigned; assigned };
// C line 17951
let _ = { pc = (pc).offset((((4 as i32)) as isize)); pc };
// C line 17952
vm_block = 30; continue;
}
// C line 17946
1759 => {
let _ = { let assigned = JS_AtomToString(ctx, (((JS_ATOM_empty_string as i32)) as JSAtom)); *({ let old = sp; sp = (sp).offset(1); old }) = assigned; assigned };
// C line 17947
vm_block = 30; continue;
}
// C line 17944
1760 => {
vm_block = 30; continue;
}
// C line 17943
1761 => {
vm_block = 29; continue;
}
// C line 17942
1762 => {
vm_block = if ((((!(((!((JS_IsException(*(sp).offset((((1 as i32)).wrapping_neg()) as isize))) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 1761 } else { 1760 }; continue;
}
// C line 17941
1763 => {
let _ = { let assigned = js_closure(ctx, JS_DupValue(ctx, *((*(b)).cpool).offset((*({ let old = pc; pc = (pc).offset(1); old })) as isize)), var_refs, sf, (0 as i32)); *({ let old = sp; sp = (sp).offset(1); old }) = assigned; assigned };
vm_block = 1762; continue;
}
// C line 17938
1765 => {
let _ = { let assigned = JS_DupValue(ctx, *((*(b)).cpool).offset((*({ let old = pc; pc = (pc).offset(1); old })) as isize)); *({ let old = sp; sp = (sp).offset(1); old }) = assigned; assigned };
// C line 17939
vm_block = 30; continue;
}
// C line 17934
1768 => {
let _ = { let assigned = JS_NewInt32(ctx, get_i16(pc)); *({ let old = sp; sp = (sp).offset(1); old }) = assigned; assigned };
// C line 17935
let _ = { pc = (pc).offset((((2 as i32)) as isize)); pc };
// C line 17936
vm_block = 30; continue;
}
// C line 17930
1771 => {
let _ = { let assigned = JS_NewInt32(ctx, get_i8(pc)); *({ let old = sp; sp = (sp).offset(1); old }) = assigned; assigned };
// C line 17931
let _ = { pc = (pc).offset((((1 as i32)) as isize)); pc };
// C line 17932
vm_block = 30; continue;
}
// C line 17927
1773 => {
let _ = { let assigned = JS_NewInt32(ctx, (opcode).wrapping_sub((OP_push_0 as i32))); *({ let old = sp; sp = (sp).offset(1); old }) = assigned; assigned };
// C line 17928
vm_block = 30; continue;
}
// C line 17914
1776 => {
let _ = { let assigned = JS_DupValue(ctx, *((*(b)).cpool).offset((crate::cutils_header::get_u32(pc)) as isize)); *({ let old = sp; sp = (sp).offset(1); old }) = assigned; assigned };
// C line 17915
let _ = { pc = (pc).offset((((4 as i32)) as isize)); pc };
// C line 17916
vm_block = 30; continue;
}
// C line 17910
1779 => {
let _ = { let assigned = __JS_NewShortBigInt(ctx, ((((crate::cutils_header::get_u32(pc)) as i32)) as i64)); *({ let old = sp; sp = (sp).offset(1); old }) = assigned; assigned };
// C line 17911
let _ = { pc = (pc).offset((((4 as i32)) as isize)); pc };
// C line 17912
vm_block = 30; continue;
}
// C line 17906
1782 => {
let _ = { let assigned = JS_NewInt32(ctx, ((crate::cutils_header::get_u32(pc)) as i32)); *({ let old = sp; sp = (sp).offset(1); old }) = assigned; assigned };
// C line 17907
let _ = { pc = (pc).offset((((4 as i32)) as isize)); pc };
// C line 17908
vm_block = 30; continue;
}
// C line 17904
1783 => {
// The official *pc++ bytecode switch: retain constant patterns at opt0.
// Equality guards otherwise compare each opcode sequentially.
vm_block = match ({ let assigned = ((*({ let old = pc; pc = (pc).offset(1); old })) as i32); opcode = assigned; assigned } as u16) { OP_invalid => 32, OP_typeof_is_function => 41, OP_typeof_is_undefined => 44, OP_is_null => 47, OP_is_undefined => 50, OP_is_undefined_or_null => 53, OP_nop => 54, OP_initial_yield => 56, OP_return_async => 58, OP_async_yield_star => 60, OP_yield_star => 60, OP_yield => 62, OP_await => 64, OP_with_get_ref => 129, OP_with_make_ref => 129, OP_with_delete_var => 129, OP_with_put_var => 129, OP_with_get_var => 129, OP_to_propkey => 139, OP_to_object => 147, OP_delete_var => 155, OP_delete => 160, OP_typeof => 165, OP_instanceof => 170, OP_private_in => 175, OP_in => 180, OP_strict_neq => 190, OP_strict_eq => 200, OP_neq => 210, OP_eq => 220, OP_gte => 230, OP_gt => 240, OP_lte => 250, OP_lt => 260, OP_xor => 270, OP_or => 280, OP_and => 290, OP_sar => 302, OP_shr => 314, OP_shl => 327, OP_not => 334, OP_dec_loc => 348, OP_inc_loc => 362, OP_post_dec => 373, OP_post_inc => 384, OP_dec => 394, OP_inc => 404, OP_neg => 422, OP_plus => 431, OP_pow => 436, OP_mod => 448, OP_div => 457, OP_mul => 487, OP_sub => 510, OP_add_loc => 540, OP_add => 571, OP_copy_data_properties => 576, OP_append => 581, OP_define_array_el => 586, OP_put_super_value => 602, OP_put_ref_value => 629, OP_put_array_el => 659, OP_get_super_value => 673, OP_get_ref_value => 698, OP_get_array_el3 => 724, OP_get_array_el2 => 747, OP_get_array_el => 770, OP_define_class_computed => 776, OP_define_class => 776, OP_define_method_computed => 812, OP_define_method => 812, OP_set_home_object => 814, OP_set_proto => 822, OP_set_name_computed => 826, OP_set_name => 832, OP_define_field => 839, OP_define_private_field => 845, OP_put_private_field => 852, OP_get_private_field => 860, OP_private_symbol => 867, OP_put_field => 887, OP_get_length => 917, OP_get_field2 => 947, OP_get_field => 977, OP_lnot => 983, OP_iterator_call => 1001, OP_iterator_next => 1008, OP_nip_catch => 1017, OP_iterator_close => 1027, OP_iterator_check_object => 1031, OP_iterator_get_value_done => 1036, OP_for_await_of_start => 1042, OP_for_await_of_next => 1047, OP_for_of_next => 1054, OP_for_of_start => 1060, OP_for_in_next => 1065, OP_for_in_start => 1069, OP_ret => 1079, OP_gosub => 1084, OP_catch => 1089, OP_if_false8 => 1100, OP_if_true8 => 1111, OP_if_false => 1122, OP_if_true => 1133, OP_goto8 => 1137, OP_goto16 => 1141, OP_goto => 1145, OP_make_var_ref => 1152, OP_make_var_ref_ref => 1171, OP_make_arg_ref => 1171, OP_make_loc_ref => 1171, OP_close_loc => 1175, OP_put_loc_check_init => 1183, OP_set_loc_check => 1190, OP_put_loc_check => 1198, OP_get_loc_checkthis => 1206, OP_get_loc_check => 1214, OP_set_loc_uninitialized => 1218, OP_put_var_ref_check_init => 1226, OP_put_var_ref_check => 1234, OP_get_var_ref_check => 1243, OP_set_var_ref => 1247, OP_put_var_ref => 1252, OP_get_var_ref => 1258, OP_set_var_ref3 => 1260, OP_set_var_ref2 => 1262, OP_set_var_ref1 => 1264, OP_set_var_ref0 => 1266, OP_put_var_ref3 => 1268, OP_put_var_ref2 => 1270, OP_put_var_ref1 => 1272, OP_put_var_ref0 => 1274, OP_get_var_ref3 => 1276, OP_get_var_ref2 => 1278, OP_get_var_ref1 => 1280, OP_get_var_ref0 => 1282, OP_set_arg3 => 1284, OP_set_arg2 => 1286, OP_set_arg1 => 1288, OP_set_arg0 => 1290, OP_put_arg3 => 1292, OP_put_arg2 => 1294, OP_put_arg1 => 1296, OP_put_arg0 => 1298, OP_get_arg3 => 1300, OP_get_arg2 => 1302, OP_get_arg1 => 1304, OP_get_arg0 => 1306, OP_set_loc3 => 1308, OP_set_loc2 => 1310, OP_set_loc1 => 1312, OP_set_loc0 => 1314, OP_put_loc3 => 1316, OP_put_loc2 => 1318, OP_put_loc1 => 1320, OP_put_loc0 => 1322, OP_get_loc3 => 1324, OP_get_loc2 => 1326, OP_get_loc1 => 1328, OP_get_loc0 => 1330, OP_set_loc8 => 1332, OP_put_loc8 => 1334, OP_get_loc8 => 1336, OP_set_arg => 1340, OP_put_arg => 1345, OP_get_arg => 1350, OP_set_loc => 1354, OP_put_loc => 1359, OP_get_loc => 1364, OP_put_var_init => 1390, OP_put_var => 1390, OP_get_var => 1405, OP_get_var_undef => 1405, OP_import => 1414, OP_get_super => 1421, OP_regexp => 1426, OP_apply_eval => 1446, OP_eval => 1466, OP_throw_error => 1481, OP_throw => 1483, OP_add_brand => 1489, OP_check_brand => 1496, OP_init_ctor => 1508, OP_check_ctor => 1512, OP_check_ctor_return => 1520, OP_return_undef => 1522, OP_return => 1524, OP_apply => 1536, OP_array_from => 1544, OP_tail_call_method => 1560, OP_call_method => 1560, OP_call_constructor => 1574, OP_tail_call => 1591, OP_call => 1591, OP_call3 => 1593, OP_call2 => 1593, OP_call1 => 1593, OP_call0 => 1593, OP_fclosure => 1599, OP_swap2 => 1606, OP_swap => 1610, OP_perm5 => 1616, OP_perm4 => 1621, OP_rot3r => 1626, OP_rot5l => 1633, OP_rot4l => 1639, OP_rot3l => 1644, OP_perm3 => 1648, OP_insert4 => 1655, OP_insert3 => 1661, OP_insert2 => 1666, OP_dup1 => 1670, OP_dup3 => 1675, OP_dup2 => 1679, OP_dup => 1682, OP_nip1 => 1687, OP_nip => 1691, OP_drop => 1694, OP_rest => 1701, OP_special_object => 1730, OP_object => 1734, OP_push_true => 1736, OP_push_false => 1738, OP_push_this => 1750, OP_null => 1752, OP_undefined => 1754, OP_push_atom_value => 1757, OP_push_empty_string => 1759, OP_fclosure8 => 1763, OP_push_const8 => 1765, OP_push_i16 => 1768, OP_push_i8 => 1771, OP_push_7 => 1773, OP_push_6 => 1773, OP_push_5 => 1773, OP_push_4 => 1773, OP_push_3 => 1773, OP_push_2 => 1773, OP_push_1 => 1773, OP_push_0 => 1773, OP_push_minus1 => 1773, OP_push_const => 1776, OP_push_bigint_i32 => 1779, OP_push_i32 => 1782, _ => 32, }; continue;
}
// C line 17893
1788 => {
let _ = { let assigned = stack_buf; sp = assigned; assigned };
// C line 17894
let _ = { let assigned = (*(b)).byte_code_buf; pc = assigned; assigned };
// C line 17895
let _ = { let assigned = (*(rt)).current_stack_frame; (*(sf)).prev_frame = assigned; assigned };
// C line 17896
let _ = { let assigned = sf; (*(rt)).current_stack_frame = assigned; assigned };
// C line 17897
let _ = { let assigned = (*(b)).realm; ctx = assigned; assigned };
vm_block = 30; continue;
}
// C line 17891
1789 => {
vm_block = if ((((i) < ((((*(b)).var_ref_count) as i32))) as i32)) != 0 { 1791 } else { 1788 }; continue;
}
// C line 17892
1791 => {
let _ = { let assigned = ((core::ptr::null_mut::<c_void>()) as *mut JSVarRef); *((*(sf)).var_refs).offset((i) as isize) = assigned; assigned };
// C line ?
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 1789; continue;
}
// C line 17889
1794 => {
let _ = { let assigned = (var_buf).offset((((((*(b)).var_count) as i32)) as isize)); stack_buf = assigned; assigned };
// C line 17890
let _ = { let assigned = (((stack_buf).offset((((((*(b)).stack_size) as i32)) as isize))) as *mut *mut JSVarRef); (*(sf)).var_refs = assigned; assigned };
// C line 17891
let _ = { let assigned = (0 as i32); i = assigned; assigned };
vm_block = 1789; continue;
}
// C line 17886
1795 => {
vm_block = if ((((i) < ((((*(b)).var_count) as i32))) as i32)) != 0 { 1797 } else { 1794 }; continue;
}
// C line 17887
1797 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; *(var_buf).offset((i) as isize) = assigned; assigned };
// C line ?
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 1795; continue;
}
// C line 17882
1801 => {
let _ = { let assigned = (local_buf).offset(((arg_allocated_size) as isize)); var_buf = assigned; assigned };
// C line 17883
let _ = { let assigned = var_buf; (*(sf)).var_buf = assigned; assigned };
// C line 17884
let _ = { let assigned = arg_buf; (*(sf)).arg_buf = assigned; assigned };
// C line 17886
let _ = { let assigned = (0 as i32); i = assigned; assigned };
vm_block = 1795; continue;
}
// C line 17880
1802 => {
let _ = { let assigned = (((*(b)).arg_count) as i32); (*(sf)).arg_count = assigned; assigned };
vm_block = 1801; continue;
}
// C line 17878
1803 => {
vm_block = if ((((i) < ((((*(b)).arg_count) as i32))) as i32)) != 0 { 1805 } else { 1802 }; continue;
}
// C line 17879
1805 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; *(arg_buf).offset((i) as isize) = assigned; assigned };
// C line ?
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 1803; continue;
}
// C line 17876
1806 => {
vm_block = if ((((i) < (n)) as i32)) != 0 { 1808 } else { 1803 }; continue;
}
// C line 17877
1808 => {
let _ = { let assigned = JS_DupValue(caller_ctx, *(argv).offset((i) as isize)); *(arg_buf).offset((i) as isize) = assigned; assigned };
// C line ?
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 1806; continue;
}
// C line 17874
1811 => {
n = min_int(argc, (((*(b)).arg_count) as i32));
// C line 17875
let _ = { let assigned = local_buf; arg_buf = assigned; assigned };
// C line 17876
let _ = { let assigned = (0 as i32); i = assigned; assigned };
vm_block = 1806; continue;
}
// C line 17873
1812 => {
vm_block = if ((((!(((!((arg_allocated_size) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 1811 } else { 1801 }; continue;
}
// C line 17866
1818 => {
let _ = { let assigned = (((*(b)).js_mode) as i32); (*(sf)).js_mode = assigned; assigned };
// C line 17867
let _ = { let assigned = argv; arg_buf = assigned; assigned };
// C line 17868
let _ = { let assigned = argc; (*(sf)).arg_count = assigned; assigned };
// C line 17869
let _ = { let assigned = func_obj; (*(sf)).cur_func = assigned; assigned };
// C line 17870
let _ = { let assigned = (((*(p)).u).func).var_refs; var_refs = assigned; assigned };
// C line 17872
let _ = { let assigned = ((match vm_local_storage.allocate((alloca_size) as usize, caller_ctx) { Some(p) => p, None => return JS_EXCEPTION }) as *mut JSValue); local_buf = assigned; assigned };
vm_block = 1812; continue;
}
// C line 17864
1819 => {
return JS_ThrowStackOverflow(caller_ctx);
}
// C line 17863
1820 => {
vm_block = if (js_check_stack_overflow(rt, alloca_size)) != 0 { 1819 } else { 1818 }; continue;
}
// C line 17860
1821 => {
let _ = { let assigned = (((size_of::<JSValue>() as usize)).wrapping_mul(((((arg_allocated_size).wrapping_add((((*(b)).var_count) as i32))).wrapping_add((((*(b)).stack_size) as i32))) as usize))).wrapping_add(((size_of::<*mut JSVarRef>() as usize)).wrapping_mul((((*(b)).var_ref_count) as usize))); alloca_size = assigned; assigned };
vm_block = 1820; continue;
}
// C line 17855
1822 => {
let _ = { let assigned = (((*(b)).arg_count) as i32); arg_allocated_size = assigned; assigned };
vm_block = 1821; continue;
}
// C line 17857
1823 => {
let _ = { let assigned = (0 as i32); arg_allocated_size = assigned; assigned };
vm_block = 1821; continue;
}
// C line 17854
1824 => {
vm_block = if ((((!(((!(((((((((argc) < ((((*(b)).arg_count) as i32))) as i32)) != 0) || ((((flags) & (((1 as i32)).wrapping_shl(((1 as i32)) as u32)))) != 0)) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 1822 } else { 1823 }; continue;
}
// C line 17852
1825 => {
let _ = { let assigned = (((*(p)).u).func).function_bytecode; b = assigned; assigned };
vm_block = 1824; continue;
}
// C line 17849
1826 => {
return (call_func).expect("checked class call")(caller_ctx, func_obj, this_obj, argc, argv, flags);
}
// C line ? labels: not_a_function
1827 => {
return JS_ThrowTypeError(caller_ctx, c"not a function".as_ptr());
}
// C line 17845
1828 => {
vm_block = if ((!((call_func).is_some()) as i32)) != 0 { 1827 } else { 1826 }; continue;
}
// C line 17844
1829 => {
let _ = { let assigned = (*((*(rt)).class_array).offset(((*(p)).class_id) as isize)).call; call_func = assigned; assigned };
vm_block = 1828; continue;
}
// C line 17842
1830 => {
vm_block = if ((((!(((!((((((((*(p)).class_id) as i32)) != ((JS_CLASS_BYTECODE_FUNCTION as i32))) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 1829 } else { 1825 }; continue;
}
// C line 17841
1831 => {
let _ = { let assigned = ((((func_obj).u).ptr) as *mut JSObject); p = assigned; assigned };
vm_block = 1830; continue;
}
// C line 17834
1832 => {
vm_block = 29; continue;
}
// C line 17836
1833 => {
vm_block = 30; continue;
}
// C line 17833
1834 => {
vm_block = if ((*(s)).throw_flag) != 0 { 1832 } else { 1833 }; continue;
}
// C line 17817
1848 => {
s = ((((func_obj).u).ptr) as *mut JSAsyncFunctionState);
// C line 17820
let _ = { let assigned = core::ptr::addr_of_mut!((*(s)).frame); sf = assigned; assigned };
// C line 17821
let _ = { let assigned = (((((*(sf)).cur_func).u).ptr) as *mut JSObject); p = assigned; assigned };
// C line 17822
let _ = { let assigned = (((*(p)).u).func).function_bytecode; b = assigned; assigned };
// C line 17823
let _ = { let assigned = (*(b)).realm; ctx = assigned; assigned };
// C line 17824
let _ = { let assigned = (((*(p)).u).func).var_refs; var_refs = assigned; assigned };
// C line 17825
let _ = { let assigned = { let assigned = (*(sf)).arg_buf; arg_buf = assigned; assigned }; local_buf = assigned; assigned };
// C line 17826
let _ = { let assigned = (*(sf)).var_buf; var_buf = assigned; assigned };
// C line 17827
let _ = { let assigned = ((*(sf)).var_buf).offset((((((*(b)).var_count) as i32)) as isize)); stack_buf = assigned; assigned };
// C line 17828
let _ = { let assigned = (*(sf)).cur_sp; sp = assigned; assigned };
// C line 17829
let _ = { let assigned = ((core::ptr::null_mut::<c_void>()) as *mut JSValue); (*(sf)).cur_sp = assigned; assigned };
// C line 17830
let _ = { let assigned = (*(sf)).cur_pc; pc = assigned; assigned };
// C line 17831
let _ = { let assigned = (*(rt)).current_stack_frame; (*(sf)).prev_frame = assigned; assigned };
// C line 17832
let _ = { let assigned = sf; (*(rt)).current_stack_frame = assigned; assigned };
vm_block = 1834; continue;
}
// C line 17838
1849 => {
vm_block = 1827; continue;
}
// C line 17816
1850 => {
vm_block = if (((flags) & (((1 as i32)).wrapping_shl(((2 as i32)) as u32)))) != 0 { 1848 } else { 1849 }; continue;
}
// C line 17815
1851 => {
vm_block = if ((((!(((!((((((((func_obj).tag) as i32)) != ((JS_TAG_OBJECT as i32))) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 1850 } else { 1831 }; continue;
}
// C line 17814
1852 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 17813
1853 => {
vm_block = if (js_poll_interrupts(caller_ctx)) != 0 { 1852 } else { 1851 }; continue;
}
// C line 17776
1855 => {
rt = (*(caller_ctx)).rt;
// C line 17780
sf = core::ptr::addr_of_mut!(sf_s);
vm_block = 1853; continue;
}
_ => std::process::abort(),
} }
}
