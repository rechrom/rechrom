// Generated from the official C AST; full opcode dispatch and original goto CFG.
// C: quickjs.c:17771..20601. Bellard/Gordon MIT.
#[cfg(not(feature = "short-opcodes"))]
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
let mut diff: i32 = 0;
let mut diff_1: i32 = 0;
let mut op1_2: JSValue = core::mem::zeroed();
let mut pos: u32 = 0;
let mut offset: i32 = 0;
let mut ret_val_1: JSValue = core::mem::zeroed();
let mut ret_3: JSValue = core::mem::zeroed();
let mut method: JSValue = core::mem::zeroed();
let mut ret_4: JSValue = core::mem::zeroed();
let mut ret_flag: i32 = 0;
let mut flags_1: i32 = 0;
let mut res_2: i32 = 0;
let mut op1_3: JSValue = core::mem::zeroed();
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
let mut ret_5: i32 = 0;
let mut obj_4: JSValue = core::mem::zeroed();
let mut atom_5: JSAtom = 0;
let mut p_3: *mut JSObject = core::ptr::null_mut();
let mut pr_3: *mut JSProperty = core::ptr::null_mut();
let mut prs_2: *mut JSShapeProperty = core::ptr::null_mut();
let mut atom_6: JSAtom = 0;
let mut val_7: JSValue = core::mem::zeroed();
let mut val_8: JSValue = core::mem::zeroed();
let mut ret_6: i32 = 0;
let mut ret_7: i32 = 0;
let mut ret_8: i32 = 0;
let mut atom_7: JSAtom = 0;
let mut ret_9: i32 = 0;
let mut atom_8: JSAtom = 0;
let mut ret_10: i32 = 0;
let mut proto_1: JSValue = core::mem::zeroed();
let mut getter: JSValue = core::mem::zeroed();
let mut setter: JSValue = core::mem::zeroed();
let mut value: JSValue = core::mem::zeroed();
let mut obj_5: JSValue = core::mem::zeroed();
let mut atom_9: JSAtom = 0;
let mut flags_2: i32 = 0;
let mut ret_11: i32 = 0;
let mut op_flags: i32 = 0;
let mut is_computed: i32 = 0;
let mut class_flags: i32 = 0;
let mut atom_10: JSAtom = 0;
let mut val_9: JSValue = core::mem::zeroed();
let mut obj_6: JSValue = core::mem::zeroed();
let mut prop: JSValue = core::mem::zeroed();
let mut p_4: *mut JSObject = core::ptr::null_mut();
let mut idx_22: u32 = 0;
let mut val_10: JSValue = core::mem::zeroed();
let mut obj_7: JSValue = core::mem::zeroed();
let mut prop_1: JSValue = core::mem::zeroed();
let mut p_5: *mut JSObject = core::ptr::null_mut();
let mut idx_23: u32 = 0;
let mut val_11: JSValue = core::mem::zeroed();
let mut p_6: *mut JSObject = core::ptr::null_mut();
let mut idx_24: u32 = 0;
let mut val_12: JSValue = core::mem::zeroed();
let mut atom_11: JSAtom = 0;
let mut ret_12: i32 = 0;
let mut val_13: JSValue = core::mem::zeroed();
let mut atom_12: JSAtom = 0;
let mut ret_13: i32 = 0;
let mut p_7: *mut JSObject = core::ptr::null_mut();
let mut idx_25: u32 = 0;
let mut new_len: u32 = 0;
let mut array_len: u32 = 0;
let mut ret_14: i32 = 0;
let mut atom_13: JSAtom = 0;
let mut ret_15: i32 = 0;
let mut atom_14: JSAtom = 0;
let mut ret_16: i32 = 0;
let mut mask: i32 = 0;
let mut op1_4: JSValue = core::mem::zeroed();
let mut op2: JSValue = core::mem::zeroed();
let mut r: i64 = 0;
let mut d1: f64 = 0.0;
let mut d2: f64 = 0.0;
let mut op2_1: JSValue = core::mem::zeroed();
let mut pv: *mut JSValue = core::ptr::null_mut();
let mut idx_26: i32 = 0;
let mut r_1: i64 = 0;
let mut ops: [JSValue; 2] = core::mem::zeroed();
let mut op1_5: JSValue = core::mem::zeroed();
let mut op2_2: JSValue = core::mem::zeroed();
let mut r_2: i64 = 0;
let mut d1_1: f64 = 0.0;
let mut d2_1: f64 = 0.0;
let mut op1_6: JSValue = core::mem::zeroed();
let mut op2_3: JSValue = core::mem::zeroed();
let mut d: f64 = 0.0;
let mut v1: i32 = 0;
let mut v2: i32 = 0;
let mut r_3: i64 = 0;
let mut d1_2: f64 = 0.0;
let mut d2_2: f64 = 0.0;
let mut op1_7: JSValue = core::mem::zeroed();
let mut op2_4: JSValue = core::mem::zeroed();
let mut v1_1: i32 = 0;
let mut v2_1: i32 = 0;
let mut op1_8: JSValue = core::mem::zeroed();
let mut op2_5: JSValue = core::mem::zeroed();
let mut v1_2: i32 = 0;
let mut v2_2: i32 = 0;
let mut r_4: i32 = 0;
let mut op1_9: JSValue = core::mem::zeroed();
let mut tag_1: u32 = 0;
let mut op1_10: JSValue = core::mem::zeroed();
let mut tag_2: u32 = 0;
let mut val_14: i32 = 0;
let mut d_1: f64 = 0.0;
let mut op1_11: JSValue = core::mem::zeroed();
let mut val_15: i32 = 0;
let mut op1_12: JSValue = core::mem::zeroed();
let mut val_16: i32 = 0;
let mut op1_13: JSValue = core::mem::zeroed();
let mut val_17: i32 = 0;
let mut op1_14: JSValue = core::mem::zeroed();
let mut val_18: i32 = 0;
let mut op1_15: JSValue = core::mem::zeroed();
let mut val_19: i32 = 0;
let mut idx_27: i32 = 0;
let mut op1_16: JSValue = core::mem::zeroed();
let mut val_20: i32 = 0;
let mut idx_28: i32 = 0;
let mut op1_17: JSValue = core::mem::zeroed();
let mut op1_18: JSValue = core::mem::zeroed();
let mut op2_6: JSValue = core::mem::zeroed();
let mut v1_3: u32 = 0;
let mut v2_3: u32 = 0;
let mut op1_19: JSValue = core::mem::zeroed();
let mut op2_7: JSValue = core::mem::zeroed();
let mut v2_4: u32 = 0;
let mut op1_20: JSValue = core::mem::zeroed();
let mut op2_8: JSValue = core::mem::zeroed();
let mut v2_5: u32 = 0;
let mut op1_21: JSValue = core::mem::zeroed();
let mut op2_9: JSValue = core::mem::zeroed();
let mut op1_22: JSValue = core::mem::zeroed();
let mut op2_10: JSValue = core::mem::zeroed();
let mut op1_23: JSValue = core::mem::zeroed();
let mut op2_11: JSValue = core::mem::zeroed();
let mut op1_24: JSValue = core::mem::zeroed();
let mut op2_12: JSValue = core::mem::zeroed();
let mut op1_25: JSValue = core::mem::zeroed();
let mut op2_13: JSValue = core::mem::zeroed();
let mut op1_26: JSValue = core::mem::zeroed();
let mut op2_14: JSValue = core::mem::zeroed();
let mut op1_27: JSValue = core::mem::zeroed();
let mut op2_15: JSValue = core::mem::zeroed();
let mut op1_28: JSValue = core::mem::zeroed();
let mut op2_16: JSValue = core::mem::zeroed();
let mut op1_29: JSValue = core::mem::zeroed();
let mut op2_17: JSValue = core::mem::zeroed();
let mut op1_30: JSValue = core::mem::zeroed();
let mut op2_18: JSValue = core::mem::zeroed();
let mut op1_31: JSValue = core::mem::zeroed();
let mut op2_19: JSValue = core::mem::zeroed();
let mut op1_32: JSValue = core::mem::zeroed();
let mut atom_15: JSAtom = 0;
let mut atom_16: JSAtom = 0;
let mut ret_17: i32 = 0;
let mut atom_17: JSAtom = 0;
let mut diff_2: i32 = 0;
let mut obj_8: JSValue = core::mem::zeroed();
let mut val_21: JSValue = core::mem::zeroed();
let mut ret_18: i32 = 0;
let mut is_with: i32 = 0;
let mut val_22: JSValue = core::mem::zeroed();
let mut pos_1: i32 = 0;
let mut vm_block: usize = 1686;
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
pos_1 = ((((val_22).u).uint64) as i32);
vm_block = 21; continue;
}
// C line 20557
23 => {
vm_block = if (((((((val_22).tag) as i32)) == ((JS_TAG_CATCH_OFFSET as i32))) as i32)) != 0 { 22 } else { 13 }; continue;
}
// C line 20555
25 => {
val_22 = *({ sp = (sp).offset(-1); sp });
// C line 20556
let _ = JS_FreeValue(ctx, val_22);
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
vm_block = 1614; continue;
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
// C line 20497
38 => {
vm_block = 37; continue;
}
// C line 20499
39 => {
vm_block = 35; continue;
}
// C line 20495
40 => {
vm_block = if (((((((((((*(sp).offset((((1 as i32)).wrapping_neg()) as isize)).tag) as i32)) == ((JS_TAG_UNDEFINED as i32))) as i32)) != 0) || ((((((((*(sp).offset((((1 as i32)).wrapping_neg()) as isize)).tag) as i32)) == ((JS_TAG_NULL as i32))) as i32)) != 0)) as i32)) != 0 { 38 } else { 39 }; continue;
}
// C line 20493
41 => {
vm_block = 30; continue;
}
// C line 20489
43 => {
let _ = { let assigned = JS_NewInt32(ctx, (3 as i32)); ret_val = assigned; assigned };
// C line 20490
vm_block = 4; continue;
}
// C line 20486
45 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; ret_val = assigned; assigned };
// C line 20487
vm_block = 4; continue;
}
// C line 20483
47 => {
let _ = { let assigned = JS_NewInt32(ctx, (2 as i32)); ret_val = assigned; assigned };
// C line 20484
vm_block = 4; continue;
}
// C line 20479
49 => {
let _ = { let assigned = JS_NewInt32(ctx, (1 as i32)); ret_val = assigned; assigned };
// C line 20480
vm_block = 4; continue;
}
// C line 20476
51 => {
let _ = { let assigned = JS_NewInt32(ctx, (0 as i32)); ret_val = assigned; assigned };
// C line 20477
vm_block = 4; continue;
}
// C line 20473
52 => {
vm_block = 30; continue;
}
// C line 20465
53 => {
let _ = { pc = (pc).offset((((diff_2).wrapping_sub((5 as i32))) as isize)); pc };
vm_block = 52; continue;
}
// C line 20462
55 => {
let _ = { let assigned = val_21; *({ let old = sp; sp = (sp).offset(1); old }) = assigned; assigned };
// C line 20463
vm_block = 53; continue;
}
// C line 20456
56 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; val_21 = assigned; assigned };
vm_block = 55; continue;
}
// C line 20460
57 => {
vm_block = 29; continue;
}
// C line 20459
58 => {
vm_block = if ((((!(((!((JS_IsException(val_21)) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 57 } else { 55 }; continue;
}
// C line 20458
59 => {
let _ = { let assigned = JS_GetProperty(ctx, obj_8, atom_17); val_21 = assigned; assigned };
vm_block = 58; continue;
}
// C line 20455
60 => {
vm_block = if ((!((ret_18) != 0) as i32)) != 0 { 56 } else { 59 }; continue;
}
// C line 20454
61 => {
vm_block = 29; continue;
}
// C line 20453
62 => {
vm_block = if ((((!(((!(((((ret_18) < ((0 as i32))) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 61 } else { 60 }; continue;
}
// C line 20452
63 => {
let _ = { let assigned = JS_HasProperty(ctx, obj_8, atom_17); ret_18 = assigned; assigned };
vm_block = 62; continue;
}
// C line 20447
65 => {
let _ = { let assigned = JS_AtomToValue(ctx, atom_17); *({ let old = sp; sp = (sp).offset(1); old }) = assigned; assigned };
// C line 20448
vm_block = 53; continue;
}
// C line 20442
68 => {
let _ = JS_FreeValue(ctx, *(sp).offset((((1 as i32)).wrapping_neg()) as isize));
// C line 20443
let _ = { let assigned = JS_NewBool(ctx, ret_18); *(sp).offset((((1 as i32)).wrapping_neg()) as isize) = assigned; assigned };
// C line 20444
vm_block = 53; continue;
}
// C line 20441
69 => {
vm_block = 29; continue;
}
// C line 20440
70 => {
vm_block = if ((((!(((!(((((ret_18) < ((0 as i32))) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 69 } else { 68 }; continue;
}
// C line 20439
71 => {
let _ = { let assigned = JS_DeleteProperty(ctx, obj_8, atom_17, (0 as i32)); ret_18 = assigned; assigned };
vm_block = 70; continue;
}
// C line 20437
72 => {
vm_block = 53; continue;
}
// C line 20436
73 => {
vm_block = 29; continue;
}
// C line 20435
74 => {
vm_block = if ((((!(((!(((((ret_18) < ((0 as i32))) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 73 } else { 72 }; continue;
}
// C line 20431
77 => {
let _ = { let assigned = JS_SetPropertyInternal(ctx, obj_8, atom_17, *(sp).offset((((2 as i32)).wrapping_neg()) as isize), obj_8, ((1 as i32)).wrapping_shl(((15 as i32)) as u32)); ret_18 = assigned; assigned };
// C line 20433
let _ = JS_FreeValue(ctx, *(sp).offset((((1 as i32)).wrapping_neg()) as isize));
// C line 20434
let _ = { sp = (sp).offset(-(((2 as i32)) as isize)); sp };
vm_block = 74; continue;
}
// C line 20427
79 => {
let _ = JS_ThrowReferenceErrorNotDefined(ctx, atom_17);
// C line 20428
vm_block = 29; continue;
}
// C line 20426
80 => {
vm_block = if (is_strict_mode(ctx)) != 0 { 79 } else { 77 }; continue;
}
// C line 20425
81 => {
vm_block = 29; continue;
}
// C line 20424
82 => {
vm_block = if ((((ret_18) < ((0 as i32))) as i32)) != 0 { 81 } else { 80 }; continue;
}
// C line 20423
83 => {
vm_block = if ((((!(((!(((((ret_18) <= ((0 as i32))) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 82 } else { 77 }; continue;
}
// C line 20422
84 => {
let _ = { let assigned = JS_HasProperty(ctx, obj_8, atom_17); ret_18 = assigned; assigned };
vm_block = 83; continue;
}
// C line 20418
86 => {
let _ = set_value(ctx, core::ptr::addr_of_mut!(*(sp).offset((((1 as i32)).wrapping_neg()) as isize)), val_21);
// C line 20419
vm_block = 53; continue;
}
// C line 20412
87 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; val_21 = assigned; assigned };
vm_block = 86; continue;
}
// C line 20409
89 => {
let _ = JS_ThrowReferenceErrorNotDefined(ctx, atom_17);
// C line 20410
vm_block = 29; continue;
}
// C line 20408
90 => {
vm_block = if (is_strict_mode(ctx)) != 0 { 89 } else { 87 }; continue;
}
// C line 20407
91 => {
vm_block = 29; continue;
}
// C line 20406
92 => {
vm_block = if ((((ret_18) < ((0 as i32))) as i32)) != 0 { 91 } else { 90 }; continue;
}
// C line 20416
93 => {
vm_block = 29; continue;
}
// C line 20415
94 => {
vm_block = if ((((!(((!((JS_IsException(val_21)) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 93 } else { 86 }; continue;
}
// C line 20414
95 => {
let _ = { let assigned = JS_GetProperty(ctx, obj_8, atom_17); val_21 = assigned; assigned };
vm_block = 94; continue;
}
// C line 20405
96 => {
vm_block = if ((((!(((!(((((ret_18) <= ((0 as i32))) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 92 } else { 95 }; continue;
}
// C line 20404
97 => {
let _ = { let assigned = JS_HasProperty(ctx, obj_8, atom_17); ret_18 = assigned; assigned };
vm_block = 96; continue;
}
// C line 20401
98 => {
vm_block = match opcode { x if x == (OP_with_get_ref as i32) => 63, x if x == (OP_with_make_ref as i32) => 65, x if x == (OP_with_delete_var as i32) => 71, x if x == (OP_with_put_var as i32) => 84, x if x == (OP_with_get_var as i32) => 97, _ => 53, }; continue;
}
// C line 20399
99 => {
vm_block = 106; continue;
}
// C line 20398
100 => {
vm_block = if (ret_18) != 0 { 99 } else { 98 }; continue;
}
// C line 20397
101 => {
vm_block = 29; continue;
}
// C line 20396
102 => {
vm_block = if ((((!(((!(((((ret_18) < ((0 as i32))) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 101 } else { 100 }; continue;
}
// C line 20395
103 => {
let _ = { let assigned = js_has_unscopable(ctx, obj_8, atom_17); ret_18 = assigned; assigned };
vm_block = 102; continue;
}
// C line 20394
104 => {
vm_block = if (is_with) != 0 { 103 } else { 98 }; continue;
}
// C line ? labels: no_with
106 => {
let _ = JS_FreeValue(ctx, *(sp).offset((((1 as i32)).wrapping_neg()) as isize));
// C line 20470
let _ = { let old = sp; sp = (sp).offset(-1); old };
vm_block = 52; continue;
}
// C line 20393
107 => {
vm_block = if (ret_18) != 0 { 104 } else { 106 }; continue;
}
// C line 20392
108 => {
vm_block = 29; continue;
}
// C line 20391
109 => {
vm_block = if ((((!(((!(((((ret_18) < ((0 as i32))) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 108 } else { 107 }; continue;
}
// C line 20383
116 => {
let _ = { let assigned = crate::cutils_header::get_u32(pc); atom_17 = assigned; assigned };
// C line 20384
let _ = { let assigned = ((crate::cutils_header::get_u32((pc).offset((((4 as i32)) as isize)))) as i32); diff_2 = assigned; assigned };
// C line 20385
let _ = { let assigned = ((*(pc).offset(((8 as i32)) as isize)) as i32); is_with = assigned; assigned };
// C line 20386
let _ = { pc = (pc).offset((((9 as i32)) as isize)); pc };
// C line 20387
let _ = { let assigned = pc; (*(sf)).cur_pc = assigned; assigned };
// C line 20389
let _ = { let assigned = *(sp).offset((((1 as i32)).wrapping_neg()) as isize); obj_8 = assigned; assigned };
// C line 20390
let _ = { let assigned = JS_HasProperty(ctx, obj_8, atom_17); ret_18 = assigned; assigned };
vm_block = 109; continue;
}
// C line 20360
117 => {
vm_block = 30; continue;
}
// C line 20356
120 => {
let _ = JS_FreeValue(ctx, *(sp).offset((((1 as i32)).wrapping_neg()) as isize));
// C line 20357
let _ = { let assigned = ret_val; *(sp).offset((((1 as i32)).wrapping_neg()) as isize) = assigned; assigned };
// C line 20358
vm_block = 117; continue;
}
// C line 20355
121 => {
vm_block = 29; continue;
}
// C line 20354
122 => {
vm_block = if (JS_IsException(ret_val)) != 0 { 121 } else { 120 }; continue;
}
// C line ?
124 => {
let _ = { let assigned = pc; (*(sf)).cur_pc = assigned; assigned };
// C line 20353
let _ = { let assigned = JS_ToPropertyKey(ctx, *(sp).offset((((1 as i32)).wrapping_neg()) as isize)); ret_val = assigned; assigned };
vm_block = 122; continue;
}
// C line 20350
125 => {
vm_block = 117; continue;
}
// C line 20346
126 => {
vm_block = match (((*(sp).offset((((1 as i32)).wrapping_neg()) as isize)).tag) as i32) { x if x == (JS_TAG_SYMBOL as i32) => 125, x if x == (JS_TAG_STRING as i32) => 125, x if x == (JS_TAG_INT as i32) => 125, _ => 124, }; continue;
}
// C line 20343
127 => {
vm_block = 30; continue;
}
// C line 20340
129 => {
let _ = JS_FreeValue(ctx, *(sp).offset((((1 as i32)).wrapping_neg()) as isize));
// C line 20341
let _ = { let assigned = ret_val; *(sp).offset((((1 as i32)).wrapping_neg()) as isize) = assigned; assigned };
vm_block = 127; continue;
}
// C line 20339
130 => {
vm_block = 29; continue;
}
// C line 20338
131 => {
vm_block = if (JS_IsException(ret_val)) != 0 { 130 } else { 129 }; continue;
}
// C line 20336
133 => {
let _ = { let assigned = pc; (*(sf)).cur_pc = assigned; assigned };
// C line 20337
let _ = { let assigned = JS_ToObject(ctx, *(sp).offset((((1 as i32)).wrapping_neg()) as isize)); ret_val = assigned; assigned };
vm_block = 131; continue;
}
// C line 20335
134 => {
vm_block = if (((((((*(sp).offset((((1 as i32)).wrapping_neg()) as isize)).tag) as i32)) != ((JS_TAG_OBJECT as i32))) as i32)) != 0 { 133 } else { 127 }; continue;
}
// C line 20330
136 => {
let _ = { let assigned = JS_NewBool(ctx, ret_17); *({ let old = sp; sp = (sp).offset(1); old }) = assigned; assigned };
// C line 20332
vm_block = 30; continue;
}
// C line 20329
137 => {
vm_block = 29; continue;
}
// C line 20328
138 => {
vm_block = if ((((!(((!(((((ret_17) < ((0 as i32))) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 137 } else { 136 }; continue;
}
// C line 20323
142 => {
let _ = { let assigned = crate::cutils_header::get_u32(pc); atom_16 = assigned; assigned };
// C line 20324
let _ = { pc = (pc).offset((((4 as i32)) as isize)); pc };
// C line 20325
let _ = { let assigned = pc; (*(sf)).cur_pc = assigned; assigned };
// C line 20327
let _ = { let assigned = JS_DeleteGlobalVar(ctx, atom_16); ret_17 = assigned; assigned };
vm_block = 138; continue;
}
// C line 20316
144 => {
let _ = { let old = sp; sp = (sp).offset(-1); old };
// C line 20317
vm_block = 30; continue;
}
// C line 20315
145 => {
vm_block = 29; continue;
}
// C line 20314
146 => {
vm_block = if (js_operator_delete(ctx, sp)) != 0 { 145 } else { 144 }; continue;
}
// C line 20313
147 => {
let _ = { let assigned = pc; (*(sf)).cur_pc = assigned; assigned };
vm_block = 146; continue;
}
// C line 20306
152 => {
let _ = { let assigned = *(sp).offset((((1 as i32)).wrapping_neg()) as isize); op1_32 = assigned; assigned };
// C line 20307
let _ = { let assigned = ((js_operator_typeof(ctx, op1_32)) as JSAtom); atom_15 = assigned; assigned };
// C line 20308
let _ = JS_FreeValue(ctx, op1_32);
// C line 20309
let _ = { let assigned = JS_AtomToString(ctx, atom_15); *(sp).offset((((1 as i32)).wrapping_neg()) as isize) = assigned; assigned };
// C line 20311
vm_block = 30; continue;
}
// C line 20299
154 => {
let _ = { let old = sp; sp = (sp).offset(-1); old };
// C line 20300
vm_block = 30; continue;
}
// C line 20298
155 => {
vm_block = 29; continue;
}
// C line 20297
156 => {
vm_block = if (js_operator_instanceof(ctx, sp)) != 0 { 155 } else { 154 }; continue;
}
// C line 20296
157 => {
let _ = { let assigned = pc; (*(sf)).cur_pc = assigned; assigned };
vm_block = 156; continue;
}
// C line 20293
159 => {
let _ = { let old = sp; sp = (sp).offset(-1); old };
// C line 20294
vm_block = 30; continue;
}
// C line 20292
160 => {
vm_block = 29; continue;
}
// C line 20291
161 => {
vm_block = if (js_operator_private_in(ctx, sp)) != 0 { 160 } else { 159 }; continue;
}
// C line 20290
162 => {
let _ = { let assigned = pc; (*(sf)).cur_pc = assigned; assigned };
vm_block = 161; continue;
}
// C line 20287
164 => {
let _ = { let old = sp; sp = (sp).offset(-1); old };
// C line 20288
vm_block = 30; continue;
}
// C line 20286
165 => {
vm_block = 29; continue;
}
// C line 20285
166 => {
vm_block = if (js_operator_in(ctx, sp)) != 0 { 165 } else { 164 }; continue;
}
// C line 20284
167 => {
let _ = { let assigned = pc; (*(sf)).cur_pc = assigned; assigned };
vm_block = 166; continue;
}
// C line 20281
168 => {
vm_block = 30; continue;
}
// C line 20281
170 => {
let _ = { let assigned = JS_NewBool(ctx, (((((((op1_31).u).uint64) as i32)) != (((((op2_19).u).uint64) as i32))) as i32)); *(sp).offset((((2 as i32)).wrapping_neg()) as isize) = assigned; assigned };
// C line 20281
let _ = { let old = sp; sp = (sp).offset(-1); old };
vm_block = 168; continue;
}
// C line 20281
171 => {
let _ = { let old = sp; sp = (sp).offset(-1); old };
vm_block = 168; continue;
}
// C line 20281
172 => {
vm_block = 29; continue;
}
// C line 20281
173 => {
vm_block = if (js_strict_eq_slow(ctx, sp, (1 as i32))) != 0 { 172 } else { 171 }; continue;
}
// C line 20281
174 => {
let _ = { let assigned = pc; (*(sf)).cur_pc = assigned; assigned };
vm_block = 173; continue;
}
// C line 20281
175 => {
vm_block = if ((((!(((!((((((((((op1_31).tag) as i32)) | ((((op2_19).tag) as i32)))) == ((0 as i32))) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 170 } else { 174 }; continue;
}
// C line 20281
177 => {
let _ = { let assigned = *(sp).offset((((2 as i32)).wrapping_neg()) as isize); op1_31 = assigned; assigned };
// C line 20281
let _ = { let assigned = *(sp).offset((((1 as i32)).wrapping_neg()) as isize); op2_19 = assigned; assigned };
vm_block = 175; continue;
}
// C line 20280
178 => {
vm_block = 30; continue;
}
// C line 20280
180 => {
let _ = { let assigned = JS_NewBool(ctx, (((((((op1_30).u).uint64) as i32)) == (((((op2_18).u).uint64) as i32))) as i32)); *(sp).offset((((2 as i32)).wrapping_neg()) as isize) = assigned; assigned };
// C line 20280
let _ = { let old = sp; sp = (sp).offset(-1); old };
vm_block = 178; continue;
}
// C line 20280
181 => {
let _ = { let old = sp; sp = (sp).offset(-1); old };
vm_block = 178; continue;
}
// C line 20280
182 => {
vm_block = 29; continue;
}
// C line 20280
183 => {
vm_block = if (js_strict_eq_slow(ctx, sp, (0 as i32))) != 0 { 182 } else { 181 }; continue;
}
// C line 20280
184 => {
let _ = { let assigned = pc; (*(sf)).cur_pc = assigned; assigned };
vm_block = 183; continue;
}
// C line 20280
185 => {
vm_block = if ((((!(((!((((((((((op1_30).tag) as i32)) | ((((op2_18).tag) as i32)))) == ((0 as i32))) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 180 } else { 184 }; continue;
}
// C line 20280
187 => {
let _ = { let assigned = *(sp).offset((((2 as i32)).wrapping_neg()) as isize); op1_30 = assigned; assigned };
// C line 20280
let _ = { let assigned = *(sp).offset((((1 as i32)).wrapping_neg()) as isize); op2_18 = assigned; assigned };
vm_block = 185; continue;
}
// C line 20279
188 => {
vm_block = 30; continue;
}
// C line 20279
190 => {
let _ = { let assigned = JS_NewBool(ctx, (((((((op1_29).u).uint64) as i32)) != (((((op2_17).u).uint64) as i32))) as i32)); *(sp).offset((((2 as i32)).wrapping_neg()) as isize) = assigned; assigned };
// C line 20279
let _ = { let old = sp; sp = (sp).offset(-1); old };
vm_block = 188; continue;
}
// C line 20279
191 => {
let _ = { let old = sp; sp = (sp).offset(-1); old };
vm_block = 188; continue;
}
// C line 20279
192 => {
vm_block = 29; continue;
}
// C line 20279
193 => {
vm_block = if (js_eq_slow(ctx, sp, (1 as i32))) != 0 { 192 } else { 191 }; continue;
}
// C line 20279
194 => {
let _ = { let assigned = pc; (*(sf)).cur_pc = assigned; assigned };
vm_block = 193; continue;
}
// C line 20279
195 => {
vm_block = if ((((!(((!((((((((((op1_29).tag) as i32)) | ((((op2_17).tag) as i32)))) == ((0 as i32))) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 190 } else { 194 }; continue;
}
// C line 20279
197 => {
let _ = { let assigned = *(sp).offset((((2 as i32)).wrapping_neg()) as isize); op1_29 = assigned; assigned };
// C line 20279
let _ = { let assigned = *(sp).offset((((1 as i32)).wrapping_neg()) as isize); op2_17 = assigned; assigned };
vm_block = 195; continue;
}
// C line 20278
198 => {
vm_block = 30; continue;
}
// C line 20278
200 => {
let _ = { let assigned = JS_NewBool(ctx, (((((((op1_28).u).uint64) as i32)) == (((((op2_16).u).uint64) as i32))) as i32)); *(sp).offset((((2 as i32)).wrapping_neg()) as isize) = assigned; assigned };
// C line 20278
let _ = { let old = sp; sp = (sp).offset(-1); old };
vm_block = 198; continue;
}
// C line 20278
201 => {
let _ = { let old = sp; sp = (sp).offset(-1); old };
vm_block = 198; continue;
}
// C line 20278
202 => {
vm_block = 29; continue;
}
// C line 20278
203 => {
vm_block = if (js_eq_slow(ctx, sp, (0 as i32))) != 0 { 202 } else { 201 }; continue;
}
// C line 20278
204 => {
let _ = { let assigned = pc; (*(sf)).cur_pc = assigned; assigned };
vm_block = 203; continue;
}
// C line 20278
205 => {
vm_block = if ((((!(((!((((((((((op1_28).tag) as i32)) | ((((op2_16).tag) as i32)))) == ((0 as i32))) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 200 } else { 204 }; continue;
}
// C line 20278
207 => {
let _ = { let assigned = *(sp).offset((((2 as i32)).wrapping_neg()) as isize); op1_28 = assigned; assigned };
// C line 20278
let _ = { let assigned = *(sp).offset((((1 as i32)).wrapping_neg()) as isize); op2_16 = assigned; assigned };
vm_block = 205; continue;
}
// C line 20277
208 => {
vm_block = 30; continue;
}
// C line 20277
210 => {
let _ = { let assigned = JS_NewBool(ctx, (((((((op1_27).u).uint64) as i32)) >= (((((op2_15).u).uint64) as i32))) as i32)); *(sp).offset((((2 as i32)).wrapping_neg()) as isize) = assigned; assigned };
// C line 20277
let _ = { let old = sp; sp = (sp).offset(-1); old };
vm_block = 208; continue;
}
// C line 20277
211 => {
let _ = { let old = sp; sp = (sp).offset(-1); old };
vm_block = 208; continue;
}
// C line 20277
212 => {
vm_block = 29; continue;
}
// C line 20277
213 => {
vm_block = if (js_relational_slow(ctx, sp, opcode)) != 0 { 212 } else { 211 }; continue;
}
// C line 20277
214 => {
let _ = { let assigned = pc; (*(sf)).cur_pc = assigned; assigned };
vm_block = 213; continue;
}
// C line 20277
215 => {
vm_block = if ((((!(((!((((((((((op1_27).tag) as i32)) | ((((op2_15).tag) as i32)))) == ((0 as i32))) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 210 } else { 214 }; continue;
}
// C line 20277
217 => {
let _ = { let assigned = *(sp).offset((((2 as i32)).wrapping_neg()) as isize); op1_27 = assigned; assigned };
// C line 20277
let _ = { let assigned = *(sp).offset((((1 as i32)).wrapping_neg()) as isize); op2_15 = assigned; assigned };
vm_block = 215; continue;
}
// C line 20276
218 => {
vm_block = 30; continue;
}
// C line 20276
220 => {
let _ = { let assigned = JS_NewBool(ctx, (((((((op1_26).u).uint64) as i32)) > (((((op2_14).u).uint64) as i32))) as i32)); *(sp).offset((((2 as i32)).wrapping_neg()) as isize) = assigned; assigned };
// C line 20276
let _ = { let old = sp; sp = (sp).offset(-1); old };
vm_block = 218; continue;
}
// C line 20276
221 => {
let _ = { let old = sp; sp = (sp).offset(-1); old };
vm_block = 218; continue;
}
// C line 20276
222 => {
vm_block = 29; continue;
}
// C line 20276
223 => {
vm_block = if (js_relational_slow(ctx, sp, opcode)) != 0 { 222 } else { 221 }; continue;
}
// C line 20276
224 => {
let _ = { let assigned = pc; (*(sf)).cur_pc = assigned; assigned };
vm_block = 223; continue;
}
// C line 20276
225 => {
vm_block = if ((((!(((!((((((((((op1_26).tag) as i32)) | ((((op2_14).tag) as i32)))) == ((0 as i32))) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 220 } else { 224 }; continue;
}
// C line 20276
227 => {
let _ = { let assigned = *(sp).offset((((2 as i32)).wrapping_neg()) as isize); op1_26 = assigned; assigned };
// C line 20276
let _ = { let assigned = *(sp).offset((((1 as i32)).wrapping_neg()) as isize); op2_14 = assigned; assigned };
vm_block = 225; continue;
}
// C line 20275
228 => {
vm_block = 30; continue;
}
// C line 20275
230 => {
let _ = { let assigned = JS_NewBool(ctx, (((((((op1_25).u).uint64) as i32)) <= (((((op2_13).u).uint64) as i32))) as i32)); *(sp).offset((((2 as i32)).wrapping_neg()) as isize) = assigned; assigned };
// C line 20275
let _ = { let old = sp; sp = (sp).offset(-1); old };
vm_block = 228; continue;
}
// C line 20275
231 => {
let _ = { let old = sp; sp = (sp).offset(-1); old };
vm_block = 228; continue;
}
// C line 20275
232 => {
vm_block = 29; continue;
}
// C line 20275
233 => {
vm_block = if (js_relational_slow(ctx, sp, opcode)) != 0 { 232 } else { 231 }; continue;
}
// C line 20275
234 => {
let _ = { let assigned = pc; (*(sf)).cur_pc = assigned; assigned };
vm_block = 233; continue;
}
// C line 20275
235 => {
vm_block = if ((((!(((!((((((((((op1_25).tag) as i32)) | ((((op2_13).tag) as i32)))) == ((0 as i32))) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 230 } else { 234 }; continue;
}
// C line 20275
237 => {
let _ = { let assigned = *(sp).offset((((2 as i32)).wrapping_neg()) as isize); op1_25 = assigned; assigned };
// C line 20275
let _ = { let assigned = *(sp).offset((((1 as i32)).wrapping_neg()) as isize); op2_13 = assigned; assigned };
vm_block = 235; continue;
}
// C line 20274
238 => {
vm_block = 30; continue;
}
// C line 20274
240 => {
let _ = { let assigned = JS_NewBool(ctx, (((((((op1_24).u).uint64) as i32)) < (((((op2_12).u).uint64) as i32))) as i32)); *(sp).offset((((2 as i32)).wrapping_neg()) as isize) = assigned; assigned };
// C line 20274
let _ = { let old = sp; sp = (sp).offset(-1); old };
vm_block = 238; continue;
}
// C line 20274
241 => {
let _ = { let old = sp; sp = (sp).offset(-1); old };
vm_block = 238; continue;
}
// C line 20274
242 => {
vm_block = 29; continue;
}
// C line 20274
243 => {
vm_block = if (js_relational_slow(ctx, sp, opcode)) != 0 { 242 } else { 241 }; continue;
}
// C line 20274
244 => {
let _ = { let assigned = pc; (*(sf)).cur_pc = assigned; assigned };
vm_block = 243; continue;
}
// C line 20274
245 => {
vm_block = if ((((!(((!((((((((((op1_24).tag) as i32)) | ((((op2_12).tag) as i32)))) == ((0 as i32))) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 240 } else { 244 }; continue;
}
// C line 20274
247 => {
let _ = { let assigned = *(sp).offset((((2 as i32)).wrapping_neg()) as isize); op1_24 = assigned; assigned };
// C line 20274
let _ = { let assigned = *(sp).offset((((1 as i32)).wrapping_neg()) as isize); op2_12 = assigned; assigned };
vm_block = 245; continue;
}
// C line 20253
248 => {
vm_block = 30; continue;
}
// C line 20242
250 => {
let _ = { let assigned = JS_NewInt32(ctx, ((((((op1_23).u).uint64) as i32)) ^ (((((op2_11).u).uint64) as i32)))); *(sp).offset((((2 as i32)).wrapping_neg()) as isize) = assigned; assigned };
// C line 20245
let _ = { let old = sp; sp = (sp).offset(-1); old };
vm_block = 248; continue;
}
// C line 20250
251 => {
let _ = { let old = sp; sp = (sp).offset(-1); old };
vm_block = 248; continue;
}
// C line 20249
252 => {
vm_block = 29; continue;
}
// C line 20248
253 => {
vm_block = if (js_binary_logic_slow(ctx, sp, opcode)) != 0 { 252 } else { 251 }; continue;
}
// C line 20247
254 => {
let _ = { let assigned = pc; (*(sf)).cur_pc = assigned; assigned };
vm_block = 253; continue;
}
// C line 20241
255 => {
vm_block = if ((((!(((!((((((((((op1_23).tag) as i32)) | ((((op2_11).tag) as i32)))) == ((0 as i32))) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 250 } else { 254 }; continue;
}
// C line 20239
257 => {
let _ = { let assigned = *(sp).offset((((2 as i32)).wrapping_neg()) as isize); op1_23 = assigned; assigned };
// C line 20240
let _ = { let assigned = *(sp).offset((((1 as i32)).wrapping_neg()) as isize); op2_11 = assigned; assigned };
vm_block = 255; continue;
}
// C line 20235
258 => {
vm_block = 30; continue;
}
// C line 20224
260 => {
let _ = { let assigned = JS_NewInt32(ctx, ((((((op1_22).u).uint64) as i32)) | (((((op2_10).u).uint64) as i32)))); *(sp).offset((((2 as i32)).wrapping_neg()) as isize) = assigned; assigned };
// C line 20227
let _ = { let old = sp; sp = (sp).offset(-1); old };
vm_block = 258; continue;
}
// C line 20232
261 => {
let _ = { let old = sp; sp = (sp).offset(-1); old };
vm_block = 258; continue;
}
// C line 20231
262 => {
vm_block = 29; continue;
}
// C line 20230
263 => {
vm_block = if (js_binary_logic_slow(ctx, sp, opcode)) != 0 { 262 } else { 261 }; continue;
}
// C line 20229
264 => {
let _ = { let assigned = pc; (*(sf)).cur_pc = assigned; assigned };
vm_block = 263; continue;
}
// C line 20223
265 => {
vm_block = if ((((!(((!((((((((((op1_22).tag) as i32)) | ((((op2_10).tag) as i32)))) == ((0 as i32))) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 260 } else { 264 }; continue;
}
// C line 20221
267 => {
let _ = { let assigned = *(sp).offset((((2 as i32)).wrapping_neg()) as isize); op1_22 = assigned; assigned };
// C line 20222
let _ = { let assigned = *(sp).offset((((1 as i32)).wrapping_neg()) as isize); op2_10 = assigned; assigned };
vm_block = 265; continue;
}
// C line 20217
268 => {
vm_block = 30; continue;
}
// C line 20206
270 => {
let _ = { let assigned = JS_NewInt32(ctx, ((((((op1_21).u).uint64) as i32)) & (((((op2_9).u).uint64) as i32)))); *(sp).offset((((2 as i32)).wrapping_neg()) as isize) = assigned; assigned };
// C line 20209
let _ = { let old = sp; sp = (sp).offset(-1); old };
vm_block = 268; continue;
}
// C line 20214
271 => {
let _ = { let old = sp; sp = (sp).offset(-1); old };
vm_block = 268; continue;
}
// C line 20213
272 => {
vm_block = 29; continue;
}
// C line 20212
273 => {
vm_block = if (js_binary_logic_slow(ctx, sp, opcode)) != 0 { 272 } else { 271 }; continue;
}
// C line 20211
274 => {
let _ = { let assigned = pc; (*(sf)).cur_pc = assigned; assigned };
vm_block = 273; continue;
}
// C line 20205
275 => {
vm_block = if ((((!(((!((((((((((op1_21).tag) as i32)) | ((((op2_9).tag) as i32)))) == ((0 as i32))) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 270 } else { 274 }; continue;
}
// C line 20203
277 => {
let _ = { let assigned = *(sp).offset((((2 as i32)).wrapping_neg()) as isize); op1_21 = assigned; assigned };
// C line 20204
let _ = { let assigned = *(sp).offset((((1 as i32)).wrapping_neg()) as isize); op2_9 = assigned; assigned };
vm_block = 275; continue;
}
// C line 20199
278 => {
vm_block = 30; continue;
}
// C line 20187
282 => {
let _ = { let assigned = ((((((op2_8).u).uint64) as i32)) as u32); v2_5 = assigned; assigned };
// C line 20188
let _ = { v2_5 = ((v2_5) & ((((31 as i32)) as u32))); v2_5 };
// C line 20189
let _ = { let assigned = JS_NewInt32(ctx, (((((op1_20).u).uint64) as i32)).wrapping_shr((v2_5) as u32)); *(sp).offset((((2 as i32)).wrapping_neg()) as isize) = assigned; assigned };
// C line 20191
let _ = { let old = sp; sp = (sp).offset(-1); old };
vm_block = 278; continue;
}
// C line 20196
283 => {
let _ = { let old = sp; sp = (sp).offset(-1); old };
vm_block = 278; continue;
}
// C line 20195
284 => {
vm_block = 29; continue;
}
// C line 20194
285 => {
vm_block = if (js_binary_logic_slow(ctx, sp, opcode)) != 0 { 284 } else { 283 }; continue;
}
// C line 20193
286 => {
let _ = { let assigned = pc; (*(sf)).cur_pc = assigned; assigned };
vm_block = 285; continue;
}
// C line 20185
287 => {
vm_block = if ((((!(((!((((((((((op1_20).tag) as i32)) | ((((op2_8).tag) as i32)))) == ((0 as i32))) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 282 } else { 286 }; continue;
}
// C line 20183
289 => {
let _ = { let assigned = *(sp).offset((((2 as i32)).wrapping_neg()) as isize); op1_20 = assigned; assigned };
// C line 20184
let _ = { let assigned = *(sp).offset((((1 as i32)).wrapping_neg()) as isize); op2_8 = assigned; assigned };
vm_block = 287; continue;
}
// C line 20179
290 => {
vm_block = 30; continue;
}
// C line 20166
294 => {
let _ = { let assigned = ((((((op2_7).u).uint64) as i32)) as u32); v2_4 = assigned; assigned };
// C line 20167
let _ = { v2_4 = ((v2_4) & ((((31 as i32)) as u32))); v2_4 };
// C line 20168
let _ = { let assigned = JS_NewUint32(ctx, (((((((op1_19).u).uint64) as i32)) as u32)).wrapping_shr((v2_4) as u32)); *(sp).offset((((2 as i32)).wrapping_neg()) as isize) = assigned; assigned };
// C line 20171
let _ = { let old = sp; sp = (sp).offset(-1); old };
vm_block = 290; continue;
}
// C line 20176
295 => {
let _ = { let old = sp; sp = (sp).offset(-1); old };
vm_block = 290; continue;
}
// C line 20175
296 => {
vm_block = 29; continue;
}
// C line 20174
297 => {
vm_block = if (js_shr_slow(ctx, sp)) != 0 { 296 } else { 295 }; continue;
}
// C line 20173
298 => {
let _ = { let assigned = pc; (*(sf)).cur_pc = assigned; assigned };
vm_block = 297; continue;
}
// C line 20164
299 => {
vm_block = if ((((!(((!((((((((((op1_19).tag) as i32)) | ((((op2_7).tag) as i32)))) == ((0 as i32))) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 294 } else { 298 }; continue;
}
// C line 20162
301 => {
let _ = { let assigned = *(sp).offset((((2 as i32)).wrapping_neg()) as isize); op1_19 = assigned; assigned };
// C line 20163
let _ = { let assigned = *(sp).offset((((1 as i32)).wrapping_neg()) as isize); op2_7 = assigned; assigned };
vm_block = 299; continue;
}
// C line 20158
302 => {
vm_block = 30; continue;
}
// C line 20146
307 => {
let _ = { let assigned = ((((((op1_18).u).uint64) as i32)) as u32); v1_3 = assigned; assigned };
// C line 20147
let _ = { let assigned = ((((((op2_6).u).uint64) as i32)) as u32); v2_3 = assigned; assigned };
// C line 20148
let _ = { v2_3 = ((v2_3) & ((((31 as i32)) as u32))); v2_3 };
// C line 20149
let _ = { let assigned = JS_NewInt32(ctx, (((v1_3).wrapping_shl((v2_3) as u32)) as i32)); *(sp).offset((((2 as i32)).wrapping_neg()) as isize) = assigned; assigned };
// C line 20150
let _ = { let old = sp; sp = (sp).offset(-1); old };
vm_block = 302; continue;
}
// C line 20155
308 => {
let _ = { let old = sp; sp = (sp).offset(-1); old };
vm_block = 302; continue;
}
// C line 20154
309 => {
vm_block = 29; continue;
}
// C line 20153
310 => {
vm_block = if (js_binary_logic_slow(ctx, sp, opcode)) != 0 { 309 } else { 308 }; continue;
}
// C line 20152
311 => {
let _ = { let assigned = pc; (*(sf)).cur_pc = assigned; assigned };
vm_block = 310; continue;
}
// C line 20144
312 => {
vm_block = if ((((!(((!((((((((((op1_18).tag) as i32)) | ((((op2_6).tag) as i32)))) == ((0 as i32))) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 307 } else { 311 }; continue;
}
// C line 20142
314 => {
let _ = { let assigned = *(sp).offset((((2 as i32)).wrapping_neg()) as isize); op1_18 = assigned; assigned };
// C line 20143
let _ = { let assigned = *(sp).offset((((1 as i32)).wrapping_neg()) as isize); op2_6 = assigned; assigned };
vm_block = 312; continue;
}
// C line 20137
315 => {
vm_block = 30; continue;
}
// C line 20130
316 => {
let _ = { let assigned = JS_NewInt32(ctx, (!(((((op1_17).u).uint64) as i32)))); *(sp).offset((((1 as i32)).wrapping_neg()) as isize) = assigned; assigned };
vm_block = 315; continue;
}
// C line 20134
317 => {
vm_block = 29; continue;
}
// C line 20133
318 => {
vm_block = if (js_not_slow(ctx, sp)) != 0 { 317 } else { 315 }; continue;
}
// C line 20132
319 => {
let _ = { let assigned = pc; (*(sf)).cur_pc = assigned; assigned };
vm_block = 318; continue;
}
// C line 20129
320 => {
vm_block = if (((((((op1_17).tag) as i32)) == ((JS_TAG_INT as i32))) as i32)) != 0 { 316 } else { 319 }; continue;
}
// C line 20128
321 => {
let _ = { let assigned = *(sp).offset((((1 as i32)).wrapping_neg()) as isize); op1_17 = assigned; assigned };
vm_block = 320; continue;
}
// C line 20124
322 => {
vm_block = 30; continue;
}
// C line 20112
323 => {
let _ = { let assigned = JS_NewInt32(ctx, (val_20).wrapping_sub((1 as i32))); *(var_buf).offset((idx_28) as isize) = assigned; assigned };
vm_block = 322; continue;
}
// C line 20111
324 => {
vm_block = 331; continue;
}
// C line 20110
325 => {
vm_block = if ((((!(((!(((((val_20) == ((((2147483647 as i32)).wrapping_neg()).wrapping_sub((1 as i32)))) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 324 } else { 323 }; continue;
}
// C line 20109
326 => {
let _ = { let assigned = ((((op1_16).u).uint64) as i32); val_20 = assigned; assigned };
vm_block = 325; continue;
}
// C line 20121
327 => {
let _ = set_value(ctx, core::ptr::addr_of_mut!(*(var_buf).offset((idx_28) as isize)), op1_16);
vm_block = 322; continue;
}
// C line 20120
328 => {
vm_block = 29; continue;
}
// C line 20119
329 => {
vm_block = if (js_unary_arith_slow(ctx, (core::ptr::addr_of_mut!(op1_16)).offset((((1 as i32)) as isize)), (OP_dec as i32))) != 0 { 328 } else { 327 }; continue;
}
// C line ? labels: dec_loc_slow
331 => {
let _ = { let assigned = pc; (*(sf)).cur_pc = assigned; assigned };
// C line 20118
let _ = { let assigned = JS_DupValue(ctx, op1_16); op1_16 = assigned; assigned };
vm_block = 329; continue;
}
// C line 20108
332 => {
vm_block = if (((((((op1_16).tag) as i32)) == ((JS_TAG_INT as i32))) as i32)) != 0 { 326 } else { 331 }; continue;
}
// C line 20104
335 => {
let _ = { let assigned = ((*(pc)) as i32); idx_28 = assigned; assigned };
// C line 20105
let _ = { pc = (pc).offset((((1 as i32)) as isize)); pc };
// C line 20107
let _ = { let assigned = *(var_buf).offset((idx_28) as isize); op1_16 = assigned; assigned };
vm_block = 332; continue;
}
// C line 20098
336 => {
vm_block = 30; continue;
}
// C line 20086
337 => {
let _ = { let assigned = JS_NewInt32(ctx, (val_19).wrapping_add((1 as i32))); *(var_buf).offset((idx_27) as isize) = assigned; assigned };
vm_block = 336; continue;
}
// C line 20085
338 => {
vm_block = 345; continue;
}
// C line 20084
339 => {
vm_block = if ((((!(((!(((((val_19) == ((2147483647 as i32))) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 338 } else { 337 }; continue;
}
// C line 20083
340 => {
let _ = { let assigned = ((((op1_15).u).uint64) as i32); val_19 = assigned; assigned };
vm_block = 339; continue;
}
// C line 20095
341 => {
let _ = set_value(ctx, core::ptr::addr_of_mut!(*(var_buf).offset((idx_27) as isize)), op1_15);
vm_block = 336; continue;
}
// C line 20094
342 => {
vm_block = 29; continue;
}
// C line 20093
343 => {
vm_block = if (js_unary_arith_slow(ctx, (core::ptr::addr_of_mut!(op1_15)).offset((((1 as i32)) as isize)), (OP_inc as i32))) != 0 { 342 } else { 341 }; continue;
}
// C line ? labels: inc_loc_slow
345 => {
let _ = { let assigned = pc; (*(sf)).cur_pc = assigned; assigned };
// C line 20092
let _ = { let assigned = JS_DupValue(ctx, op1_15); op1_15 = assigned; assigned };
vm_block = 343; continue;
}
// C line 20082
346 => {
vm_block = if (((((((op1_15).tag) as i32)) == ((JS_TAG_INT as i32))) as i32)) != 0 { 340 } else { 345 }; continue;
}
// C line 20078
349 => {
let _ = { let assigned = ((*(pc)) as i32); idx_27 = assigned; assigned };
// C line 20079
let _ = { pc = (pc).offset((((1 as i32)) as isize)); pc };
// C line 20081
let _ = { let assigned = *(var_buf).offset((idx_27) as isize); op1_15 = assigned; assigned };
vm_block = 346; continue;
}
// C line 20070
351 => {
let _ = { let old = sp; sp = (sp).offset(1); old };
// C line 20072
vm_block = 30; continue;
}
// C line 20063
352 => {
let _ = { let assigned = JS_NewInt32(ctx, (val_18).wrapping_sub((1 as i32))); *(sp).offset(((0 as i32)) as isize) = assigned; assigned };
vm_block = 351; continue;
}
// C line 20062
353 => {
vm_block = 358; continue;
}
// C line 20061
354 => {
vm_block = if ((((!(((!(((((val_18) == ((((2147483647 as i32)).wrapping_neg()).wrapping_sub((1 as i32)))) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 353 } else { 352 }; continue;
}
// C line 20060
355 => {
let _ = { let assigned = ((((op1_14).u).uint64) as i32); val_18 = assigned; assigned };
vm_block = 354; continue;
}
// C line 20068
356 => {
vm_block = 29; continue;
}
// C line 20067
357 => {
vm_block = if (js_post_inc_slow(ctx, sp, opcode)) != 0 { 356 } else { 351 }; continue;
}
// C line ? labels: post_dec_slow
358 => {
let _ = { let assigned = pc; (*(sf)).cur_pc = assigned; assigned };
vm_block = 357; continue;
}
// C line 20059
359 => {
vm_block = if (((((((op1_14).tag) as i32)) == ((JS_TAG_INT as i32))) as i32)) != 0 { 355 } else { 358 }; continue;
}
// C line 20058
360 => {
let _ = { let assigned = *(sp).offset((((1 as i32)).wrapping_neg()) as isize); op1_14 = assigned; assigned };
vm_block = 359; continue;
}
// C line 20051
362 => {
let _ = { let old = sp; sp = (sp).offset(1); old };
// C line 20053
vm_block = 30; continue;
}
// C line 20044
363 => {
let _ = { let assigned = JS_NewInt32(ctx, (val_17).wrapping_add((1 as i32))); *(sp).offset(((0 as i32)) as isize) = assigned; assigned };
vm_block = 362; continue;
}
// C line 20043
364 => {
vm_block = 369; continue;
}
// C line 20042
365 => {
vm_block = if ((((!(((!(((((val_17) == ((2147483647 as i32))) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 364 } else { 363 }; continue;
}
// C line 20041
366 => {
let _ = { let assigned = ((((op1_13).u).uint64) as i32); val_17 = assigned; assigned };
vm_block = 365; continue;
}
// C line 20049
367 => {
vm_block = 29; continue;
}
// C line 20048
368 => {
vm_block = if (js_post_inc_slow(ctx, sp, opcode)) != 0 { 367 } else { 362 }; continue;
}
// C line ? labels: post_inc_slow
369 => {
let _ = { let assigned = pc; (*(sf)).cur_pc = assigned; assigned };
vm_block = 368; continue;
}
// C line 20040
370 => {
vm_block = if (((((((op1_13).tag) as i32)) == ((JS_TAG_INT as i32))) as i32)) != 0 { 366 } else { 369 }; continue;
}
// C line 20039
371 => {
let _ = { let assigned = *(sp).offset((((1 as i32)).wrapping_neg()) as isize); op1_13 = assigned; assigned };
vm_block = 370; continue;
}
// C line 20034
372 => {
vm_block = 30; continue;
}
// C line 20026
373 => {
let _ = { let assigned = JS_NewInt32(ctx, (val_16).wrapping_sub((1 as i32))); *(sp).offset((((1 as i32)).wrapping_neg()) as isize) = assigned; assigned };
vm_block = 372; continue;
}
// C line 20025
374 => {
vm_block = 379; continue;
}
// C line 20024
375 => {
vm_block = if ((((!(((!(((((val_16) == ((((2147483647 as i32)).wrapping_neg()).wrapping_sub((1 as i32)))) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 374 } else { 373 }; continue;
}
// C line 20023
376 => {
let _ = { let assigned = ((((op1_12).u).uint64) as i32); val_16 = assigned; assigned };
vm_block = 375; continue;
}
// C line 20031
377 => {
vm_block = 29; continue;
}
// C line 20030
378 => {
vm_block = if (js_unary_arith_slow(ctx, sp, opcode)) != 0 { 377 } else { 372 }; continue;
}
// C line ? labels: dec_slow
379 => {
let _ = { let assigned = pc; (*(sf)).cur_pc = assigned; assigned };
vm_block = 378; continue;
}
// C line 20022
380 => {
vm_block = if (((((((op1_12).tag) as i32)) == ((JS_TAG_INT as i32))) as i32)) != 0 { 376 } else { 379 }; continue;
}
// C line 20021
381 => {
let _ = { let assigned = *(sp).offset((((1 as i32)).wrapping_neg()) as isize); op1_12 = assigned; assigned };
vm_block = 380; continue;
}
// C line 20016
382 => {
vm_block = 30; continue;
}
// C line 20008
383 => {
let _ = { let assigned = JS_NewInt32(ctx, (val_15).wrapping_add((1 as i32))); *(sp).offset((((1 as i32)).wrapping_neg()) as isize) = assigned; assigned };
vm_block = 382; continue;
}
// C line 20007
384 => {
vm_block = 389; continue;
}
// C line 20006
385 => {
vm_block = if ((((!(((!(((((val_15) == ((2147483647 as i32))) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 384 } else { 383 }; continue;
}
// C line 20005
386 => {
let _ = { let assigned = ((((op1_11).u).uint64) as i32); val_15 = assigned; assigned };
vm_block = 385; continue;
}
// C line 20013
387 => {
vm_block = 29; continue;
}
// C line 20012
388 => {
vm_block = if (js_unary_arith_slow(ctx, sp, opcode)) != 0 { 387 } else { 382 }; continue;
}
// C line ? labels: inc_slow
389 => {
let _ = { let assigned = pc; (*(sf)).cur_pc = assigned; assigned };
vm_block = 388; continue;
}
// C line 20004
390 => {
vm_block = if (((((((op1_11).tag) as i32)) == ((JS_TAG_INT as i32))) as i32)) != 0 { 386 } else { 389 }; continue;
}
// C line 20003
391 => {
let _ = { let assigned = *(sp).offset((((1 as i32)).wrapping_neg()) as isize); op1_11 = assigned; assigned };
vm_block = 390; continue;
}
// C line 19998
392 => {
vm_block = 30; continue;
}
// C line 19987
393 => {
let _ = { let assigned = JS_NewInt32(ctx, (val_14).wrapping_neg()); *(sp).offset((((1 as i32)).wrapping_neg()) as isize) = assigned; assigned };
vm_block = 392; continue;
}
// C line 19984
395 => {
let _ = { let assigned = (-(((val_14) as f64))); d_1 = assigned; assigned };
// C line 19985
vm_block = 401; continue;
}
// C line 19983
396 => {
vm_block = if ((((!(((!(((((val_14) == ((((2147483647 as i32)).wrapping_neg()).wrapping_sub((1 as i32)))) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 395 } else { 393 }; continue;
}
// C line 19980
398 => {
let _ = { let assigned = (-((0 as f64))); d_1 = assigned; assigned };
// C line 19981
vm_block = 401; continue;
}
// C line 19979
399 => {
vm_block = if ((((!(((!(((((val_14) == ((0 as i32))) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 398 } else { 396 }; continue;
}
// C line 19977
400 => {
let _ = { let assigned = ((((op1_10).u).uint64) as i32); val_14 = assigned; assigned };
vm_block = 399; continue;
}
// C line ? labels: neg_fp_res
401 => {
let _ = { let assigned = __JS_NewFloat64(ctx, d_1); *(sp).offset((((1 as i32)).wrapping_neg()) as isize) = assigned; assigned };
vm_block = 392; continue;
}
// C line 19989
402 => {
let _ = { let assigned = (-(((op1_10).u).float64)); d_1 = assigned; assigned };
vm_block = 401; continue;
}
// C line 19995
403 => {
vm_block = 29; continue;
}
// C line 19994
404 => {
vm_block = if (js_unary_arith_slow(ctx, sp, opcode)) != 0 { 403 } else { 392 }; continue;
}
// C line 19993
405 => {
let _ = { let assigned = pc; (*(sf)).cur_pc = assigned; assigned };
vm_block = 404; continue;
}
// C line 19988
406 => {
vm_block = if ((((tag_2) == ((((JS_TAG_FLOAT64 as i32)) as u32))) as i32)) != 0 { 402 } else { 405 }; continue;
}
// C line 19974
407 => {
vm_block = if ((((((((((((tag_2) == ((((JS_TAG_INT as i32)) as u32))) as i32)) != 0) || (((((tag_2) == ((((JS_TAG_BOOL as i32)) as u32))) as i32)) != 0)) as i32)) != 0) || (((((tag_2) == ((((JS_TAG_NULL as i32)) as u32))) as i32)) != 0)) as i32)) != 0 { 400 } else { 406 }; continue;
}
// C line 19972
409 => {
let _ = { let assigned = *(sp).offset((((1 as i32)).wrapping_neg()) as isize); op1_10 = assigned; assigned };
// C line 19973
let _ = { let assigned = (((((op1_10).tag) as i32)) as u32); tag_2 = assigned; assigned };
vm_block = 407; continue;
}
// C line 19965
410 => {
vm_block = 30; continue;
}
// C line 19958
411 => {
let _ = { let assigned = JS_NewInt32(ctx, ((((op1_9).u).uint64) as i32)); *(sp).offset((((1 as i32)).wrapping_neg()) as isize) = assigned; assigned };
vm_block = 410; continue;
}
// C line 19962
412 => {
vm_block = 29; continue;
}
// C line 19961
413 => {
vm_block = if (js_unary_arith_slow(ctx, sp, opcode)) != 0 { 412 } else { 410 }; continue;
}
// C line 19960
414 => {
let _ = { let assigned = pc; (*(sf)).cur_pc = assigned; assigned };
vm_block = 413; continue;
}
// C line ?
415 => {
vm_block = if ((((((((tag_1) == ((((JS_TAG_NULL as i32)) as u32))) as i32)) != 0) || (((((tag_1) == ((((JS_TAG_BOOL as i32)) as u32))) as i32)) != 0)) as i32)) != 0 { 411 } else { 414 }; continue;
}
// C line 19956
416 => {
vm_block = if ((((((((tag_1) == ((((JS_TAG_INT as i32)) as u32))) as i32)) != 0) || (((((tag_1) == ((((JS_TAG_FLOAT64 as i32)) as u32))) as i32)) != 0)) as i32)) != 0 { 410 } else { 415 }; continue;
}
// C line 19954
418 => {
let _ = { let assigned = *(sp).offset((((1 as i32)).wrapping_neg()) as isize); op1_9 = assigned; assigned };
// C line 19955
let _ = { let assigned = (((((op1_9).tag) as i32)) as u32); tag_1 = assigned; assigned };
vm_block = 416; continue;
}
// C line 19947
420 => {
let _ = { let old = sp; sp = (sp).offset(-1); old };
// C line 19948
vm_block = 30; continue;
}
// C line 19946
421 => {
vm_block = 29; continue;
}
// C line 19945
422 => {
vm_block = if (js_binary_arith_slow(ctx, sp, opcode)) != 0 { 421 } else { 420 }; continue;
}
// C line ? labels: binary_arith_slow
423 => {
let _ = { let assigned = pc; (*(sf)).cur_pc = assigned; assigned };
vm_block = 422; continue;
}
// C line 19934
427 => {
let _ = { let assigned = ((v1_2) % (v2_2)); r_4 = assigned; assigned };
// C line 19935
let _ = { let assigned = JS_NewInt32(ctx, r_4); *(sp).offset((((2 as i32)).wrapping_neg()) as isize) = assigned; assigned };
// C line 19936
let _ = { let old = sp; sp = (sp).offset(-1); old };
// C line 19941
vm_block = 30; continue;
}
// C line 19933
428 => {
vm_block = 423; continue;
}
// C line 19932
429 => {
vm_block = if ((((!(((!(((((((((v1_2) < ((0 as i32))) as i32)) != 0) || (((((v2_2) <= ((0 as i32))) as i32)) != 0)) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 428 } else { 427 }; continue;
}
// C line 19928
431 => {
let _ = { let assigned = ((((op1_8).u).uint64) as i32); v1_2 = assigned; assigned };
// C line 19929
let _ = { let assigned = ((((op2_5).u).uint64) as i32); v2_2 = assigned; assigned };
vm_block = 429; continue;
}
// C line 19938
432 => {
vm_block = 423; continue;
}
// C line 19926
433 => {
vm_block = if ((((!(((!((((((((((op1_8).tag) as i32)) | ((((op2_5).tag) as i32)))) == ((0 as i32))) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 431 } else { 432 }; continue;
}
// C line 19924
435 => {
let _ = { let assigned = *(sp).offset((((2 as i32)).wrapping_neg()) as isize); op1_8 = assigned; assigned };
// C line 19925
let _ = { let assigned = *(sp).offset((((1 as i32)).wrapping_neg()) as isize); op2_5 = assigned; assigned };
vm_block = 433; continue;
}
// C line 19912
440 => {
let _ = { let assigned = ((((op1_7).u).uint64) as i32); v1_1 = assigned; assigned };
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
441 => {
vm_block = 423; continue;
}
// C line 19910
442 => {
vm_block = if ((((!(((!((((((((((op1_7).tag) as i32)) | ((((op2_4).tag) as i32)))) == ((0 as i32))) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 440 } else { 441 }; continue;
}
// C line 19908
444 => {
let _ = { let assigned = *(sp).offset((((2 as i32)).wrapping_neg()) as isize); op1_7 = assigned; assigned };
// C line 19909
let _ = { let assigned = *(sp).offset((((1 as i32)).wrapping_neg()) as isize); op2_4 = assigned; assigned };
vm_block = 442; continue;
}
// C line 19904
445 => {
vm_block = 30; continue;
}
// C line 19877
447 => {
let _ = { let assigned = JS_NewInt32(ctx, ((r_3) as i32)); *(sp).offset((((2 as i32)).wrapping_neg()) as isize) = assigned; assigned };
// C line 19878
let _ = { let old = sp; sp = (sp).offset(-1); old };
vm_block = 445; continue;
}
// C line 19874
449 => {
let _ = { let assigned = (-((0 as f64))); d = assigned; assigned };
// C line 19875
vm_block = 458; continue;
}
// C line 19873
450 => {
vm_block = if ((((!(((!(((((((((r_3) == ((((0 as i32)) as i64))) as i32)) != 0) && (((((((v1) | (v2))) < ((0 as i32))) as i32)) != 0)) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 449 } else { 447 }; continue;
}
// C line 19869
452 => {
let _ = { let assigned = ((r_3) as f64); d = assigned; assigned };
// C line 19870
vm_block = 458; continue;
}
// C line 19868
453 => {
vm_block = if ((((!(((!(((((((((r_3) as i32)) as i64)) != (r_3)) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 452 } else { 450 }; continue;
}
// C line 19865
456 => {
let _ = { let assigned = ((((op1_6).u).uint64) as i32); v1 = assigned; assigned };
// C line 19866
let _ = { let assigned = ((((op2_3).u).uint64) as i32); v2 = assigned; assigned };
// C line 19867
let _ = { let assigned = (((v1) as i64)).wrapping_mul(((v2) as i64)); r_3 = assigned; assigned };
vm_block = 453; continue;
}
// C line ? labels: mul_fp_res
458 => {
let _ = { let assigned = __JS_NewFloat64(ctx, d); *(sp).offset((((2 as i32)).wrapping_neg()) as isize) = assigned; assigned };
// C line 19899
let _ = { let old = sp; sp = (sp).offset(-1); old };
vm_block = 445; continue;
}
// C line 19896
459 => {
let _ = { let assigned = ((d1_2) * (d2_2)); d = assigned; assigned };
vm_block = 458; continue;
}
// C line 19890
460 => {
let _ = { let assigned = ((op2_3).u).float64; d2_2 = assigned; assigned };
vm_block = 459; continue;
}
// C line 19892
461 => {
let _ = { let assigned = ((((((op2_3).u).uint64) as i32)) as f64); d2_2 = assigned; assigned };
vm_block = 459; continue;
}
// C line 19894
462 => {
vm_block = 423; continue;
}
// C line 19891
463 => {
vm_block = if (((((((op2_3).tag) as i32)) == ((JS_TAG_INT as i32))) as i32)) != 0 { 461 } else { 462 }; continue;
}
// C line 19889
464 => {
vm_block = if (((((((((op2_3).tag) as i32)) as u32)) == ((((JS_TAG_FLOAT64 as i32)) as u32))) as i32)) != 0 { 460 } else { 463 }; continue;
}
// C line 19883
465 => {
let _ = { let assigned = ((op1_6).u).float64; d1_2 = assigned; assigned };
vm_block = 464; continue;
}
// C line 19885
466 => {
let _ = { let assigned = ((((((op1_6).u).uint64) as i32)) as f64); d1_2 = assigned; assigned };
vm_block = 464; continue;
}
// C line 19887
467 => {
vm_block = 423; continue;
}
// C line 19884
468 => {
vm_block = if (((((((op1_6).tag) as i32)) == ((JS_TAG_INT as i32))) as i32)) != 0 { 466 } else { 467 }; continue;
}
// C line 19882
469 => {
vm_block = if (((((((((op1_6).tag) as i32)) as u32)) == ((((JS_TAG_FLOAT64 as i32)) as u32))) as i32)) != 0 { 465 } else { 468 }; continue;
}
// C line 19901
470 => {
vm_block = 423; continue;
}
// C line 19879
471 => {
vm_block = if (((((((((((((op1_6).tag) as i32)) as u32)) == ((((JS_TAG_FLOAT64 as i32)) as u32))) as i32)) != 0) || ((((((((((op2_3).tag) as i32)) as u32)) == ((((JS_TAG_FLOAT64 as i32)) as u32))) as i32)) != 0)) as i32)) != 0 { 469 } else { 470 }; continue;
}
// C line 19862
472 => {
vm_block = if ((((!(((!((((((((((op1_6).tag) as i32)) | ((((op2_3).tag) as i32)))) == ((0 as i32))) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 456 } else { 471 }; continue;
}
// C line 19860
474 => {
let _ = { let assigned = *(sp).offset((((2 as i32)).wrapping_neg()) as isize); op1_6 = assigned; assigned };
// C line 19861
let _ = { let assigned = *(sp).offset((((1 as i32)).wrapping_neg()) as isize); op2_3 = assigned; assigned };
vm_block = 472; continue;
}
// C line 19855
475 => {
vm_block = 30; continue;
}
// C line 19831
476 => {
let _ = { let old = sp; sp = (sp).offset(-1); old };
vm_block = 475; continue;
}
// C line 19827
477 => {
let _ = { let assigned = __JS_NewFloat64(ctx, ((r_2) as f64)); *(sp).offset((((2 as i32)).wrapping_neg()) as isize) = assigned; assigned };
vm_block = 476; continue;
}
// C line 19829
478 => {
let _ = { let assigned = JS_NewInt32(ctx, ((r_2) as i32)); *(sp).offset((((2 as i32)).wrapping_neg()) as isize) = assigned; assigned };
vm_block = 476; continue;
}
// C line 19826
479 => {
vm_block = if ((((!(((!(((((((((r_2) as i32)) as i64)) != (r_2)) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 477 } else { 478 }; continue;
}
// C line 19825
480 => {
let _ = { let assigned = (((((((op1_5).u).uint64) as i32)) as i64)).wrapping_sub(((((((op2_2).u).uint64) as i32)) as i64)); r_2 = assigned; assigned };
vm_block = 479; continue;
}
// C line 19849
482 => {
let _ = { let assigned = __JS_NewFloat64(ctx, ((d1_1) - (d2_1))); *(sp).offset((((2 as i32)).wrapping_neg()) as isize) = assigned; assigned };
// C line 19850
let _ = { let old = sp; sp = (sp).offset(-1); old };
vm_block = 475; continue;
}
// C line 19843
483 => {
let _ = { let assigned = ((op2_2).u).float64; d2_1 = assigned; assigned };
vm_block = 482; continue;
}
// C line 19845
484 => {
let _ = { let assigned = ((((((op2_2).u).uint64) as i32)) as f64); d2_1 = assigned; assigned };
vm_block = 482; continue;
}
// C line 19847
485 => {
vm_block = 423; continue;
}
// C line 19844
486 => {
vm_block = if (((((((op2_2).tag) as i32)) == ((JS_TAG_INT as i32))) as i32)) != 0 { 484 } else { 485 }; continue;
}
// C line 19842
487 => {
vm_block = if (((((((((op2_2).tag) as i32)) as u32)) == ((((JS_TAG_FLOAT64 as i32)) as u32))) as i32)) != 0 { 483 } else { 486 }; continue;
}
// C line 19836
488 => {
let _ = { let assigned = ((op1_5).u).float64; d1_1 = assigned; assigned };
vm_block = 487; continue;
}
// C line 19838
489 => {
let _ = { let assigned = ((((((op1_5).u).uint64) as i32)) as f64); d1_1 = assigned; assigned };
vm_block = 487; continue;
}
// C line 19840
490 => {
vm_block = 423; continue;
}
// C line 19837
491 => {
vm_block = if (((((((op1_5).tag) as i32)) == ((JS_TAG_INT as i32))) as i32)) != 0 { 489 } else { 490 }; continue;
}
// C line 19835
492 => {
vm_block = if (((((((((op1_5).tag) as i32)) as u32)) == ((((JS_TAG_FLOAT64 as i32)) as u32))) as i32)) != 0 { 488 } else { 491 }; continue;
}
// C line 19852
493 => {
vm_block = 423; continue;
}
// C line 19832
494 => {
vm_block = if (((((((((((((op1_5).tag) as i32)) as u32)) == ((((JS_TAG_FLOAT64 as i32)) as u32))) as i32)) != 0) || ((((((((((op2_2).tag) as i32)) as u32)) == ((((JS_TAG_FLOAT64 as i32)) as u32))) as i32)) != 0)) as i32)) != 0 { 492 } else { 493 }; continue;
}
// C line 19823
495 => {
vm_block = if ((((!(((!((((((((((op1_5).tag) as i32)) | ((((op2_2).tag) as i32)))) == ((0 as i32))) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 480 } else { 494 }; continue;
}
// C line 19821
497 => {
let _ = { let assigned = *(sp).offset((((2 as i32)).wrapping_neg()) as isize); op1_5 = assigned; assigned };
// C line 19822
let _ = { let assigned = *(sp).offset((((1 as i32)).wrapping_neg()) as isize); op2_2 = assigned; assigned };
vm_block = 495; continue;
}
// C line 19817
498 => {
vm_block = 30; continue;
}
// C line 19787
499 => {
let _ = { let old = sp; sp = (sp).offset(-1); old };
vm_block = 498; continue;
}
// C line 19783
500 => {
let _ = { let assigned = __JS_NewFloat64(ctx, ((r_1) as f64)); *(pv) = assigned; assigned };
vm_block = 499; continue;
}
// C line 19785
501 => {
let _ = { let assigned = JS_NewInt32(ctx, ((r_1) as i32)); *(pv) = assigned; assigned };
vm_block = 499; continue;
}
// C line 19782
502 => {
vm_block = if ((((!(((!(((((((((r_1) as i32)) as i64)) != (r_1)) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 500 } else { 501 }; continue;
}
// C line 19781
503 => {
let _ = { let assigned = (((((((*(pv)).u).uint64) as i32)) as i64)).wrapping_add(((((((op2_1).u).uint64) as i32)) as i64)); r_1 = assigned; assigned };
vm_block = 502; continue;
}
// C line 19789
505 => {
let _ = { let assigned = __JS_NewFloat64(ctx, ((((*(pv)).u).float64) + (((op2_1).u).float64))); *(pv) = assigned; assigned };
// C line 19791
let _ = { let old = sp; sp = (sp).offset(-1); old };
vm_block = 498; continue;
}
// C line 19797
506 => {
let _ = JS_FreeValue(ctx, op2_1);
vm_block = 498; continue;
}
// C line 19802
507 => {
let _ = set_value(ctx, pv, op2_1);
vm_block = 498; continue;
}
// C line 19801
508 => {
vm_block = 29; continue;
}
// C line 19800
509 => {
vm_block = if (JS_IsException(op2_1)) != 0 { 508 } else { 507 }; continue;
}
// C line 19799
510 => {
let _ = { let assigned = JS_ConcatString(ctx, JS_DupValue(ctx, *(pv)), op2_1); op2_1 = assigned; assigned };
vm_block = 509; continue;
}
// C line 19796
511 => {
vm_block = if (JS_ConcatStringInPlace(ctx, ((((*(pv)).u).ptr) as *mut JSString), op2_1)) != 0 { 506 } else { 510 }; continue;
}
// C line 19794
513 => {
let _ = { let old = sp; sp = (sp).offset(-1); old };
// C line 19795
let _ = { let assigned = pc; (*(sf)).cur_pc = assigned; assigned };
vm_block = 511; continue;
}
// C line 19814
514 => {
let _ = set_value(ctx, pv, *((ops).as_mut_ptr()).offset(((0 as i32)) as isize));
vm_block = 498; continue;
}
// C line 19813
515 => {
vm_block = 29; continue;
}
// C line 19812
516 => {
vm_block = if (js_add_slow(ctx, ((ops).as_mut_ptr()).offset((((2 as i32)) as isize)))) != 0 { 515 } else { 514 }; continue;
}
// C line 19808
520 => {
let _ = { let assigned = pc; (*(sf)).cur_pc = assigned; assigned };
// C line 19809
let _ = { let assigned = JS_DupValue(ctx, *(pv)); *((ops).as_mut_ptr()).offset(((0 as i32)) as isize) = assigned; assigned };
// C line 19810
let _ = { let assigned = op2_1; *((ops).as_mut_ptr()).offset(((1 as i32)) as isize) = assigned; assigned };
// C line 19811
let _ = { let old = sp; sp = (sp).offset(-1); old };
vm_block = 516; continue;
}
// C line 19792
521 => {
vm_block = if (((((((((((*(pv)).tag) as i32)) == ((JS_TAG_STRING as i32))) as i32)) != 0) && ((((((((op2_1).tag) as i32)) == ((JS_TAG_STRING as i32))) as i32)) != 0)) as i32)) != 0 { 513 } else { 520 }; continue;
}
// C line 19788
522 => {
vm_block = if (((((((((((((*(pv)).tag) as i32)) as u32)) == ((((JS_TAG_FLOAT64 as i32)) as u32))) as i32)) != 0) && ((((((((((op2_1).tag) as i32)) as u32)) == ((((JS_TAG_FLOAT64 as i32)) as u32))) as i32)) != 0)) as i32)) != 0 { 505 } else { 521 }; continue;
}
// C line 19779
523 => {
vm_block = if ((((!(((!((((((((((*(pv)).tag) as i32)) | ((((op2_1).tag) as i32)))) == ((0 as i32))) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 503 } else { 522 }; continue;
}
// C line 19774
527 => {
let _ = { let assigned = ((*(pc)) as i32); idx_26 = assigned; assigned };
// C line 19775
let _ = { pc = (pc).offset((((1 as i32)) as isize)); pc };
// C line 19777
let _ = { let assigned = *(sp).offset((((1 as i32)).wrapping_neg()) as isize); op2_1 = assigned; assigned };
// C line 19778
let _ = { let assigned = core::ptr::addr_of_mut!(*(var_buf).offset((idx_26) as isize)); pv = assigned; assigned };
vm_block = 523; continue;
}
// C line 19768
528 => {
vm_block = 30; continue;
}
// C line 19735
529 => {
let _ = { let old = sp; sp = (sp).offset(-1); old };
vm_block = 528; continue;
}
// C line 19731
530 => {
let _ = { let assigned = __JS_NewFloat64(ctx, ((r) as f64)); *(sp).offset((((2 as i32)).wrapping_neg()) as isize) = assigned; assigned };
vm_block = 529; continue;
}
// C line 19733
531 => {
let _ = { let assigned = JS_NewInt32(ctx, ((r) as i32)); *(sp).offset((((2 as i32)).wrapping_neg()) as isize) = assigned; assigned };
vm_block = 529; continue;
}
// C line 19730
532 => {
vm_block = if ((((!(((!(((((((((r) as i32)) as i64)) != (r)) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 530 } else { 531 }; continue;
}
// C line 19729
533 => {
let _ = { let assigned = (((((((op1_4).u).uint64) as i32)) as i64)).wrapping_add(((((((op2).u).uint64) as i32)) as i64)); r = assigned; assigned };
vm_block = 532; continue;
}
// C line 19753
535 => {
let _ = { let assigned = __JS_NewFloat64(ctx, ((d1) + (d2))); *(sp).offset((((2 as i32)).wrapping_neg()) as isize) = assigned; assigned };
// C line 19754
let _ = { let old = sp; sp = (sp).offset(-1); old };
vm_block = 528; continue;
}
// C line 19747
536 => {
let _ = { let assigned = ((op2).u).float64; d2 = assigned; assigned };
vm_block = 535; continue;
}
// C line 19749
537 => {
let _ = { let assigned = ((((((op2).u).uint64) as i32)) as f64); d2 = assigned; assigned };
vm_block = 535; continue;
}
// C line 19751
538 => {
vm_block = 553; continue;
}
// C line 19748
539 => {
vm_block = if (((((((op2).tag) as i32)) == ((JS_TAG_INT as i32))) as i32)) != 0 { 537 } else { 538 }; continue;
}
// C line 19746
540 => {
vm_block = if (((((((((op2).tag) as i32)) as u32)) == ((((JS_TAG_FLOAT64 as i32)) as u32))) as i32)) != 0 { 536 } else { 539 }; continue;
}
// C line 19740
541 => {
let _ = { let assigned = ((op1_4).u).float64; d1 = assigned; assigned };
vm_block = 540; continue;
}
// C line 19742
542 => {
let _ = { let assigned = ((((((op1_4).u).uint64) as i32)) as f64); d1 = assigned; assigned };
vm_block = 540; continue;
}
// C line 19744
543 => {
vm_block = 553; continue;
}
// C line 19741
544 => {
vm_block = if (((((((op1_4).tag) as i32)) == ((JS_TAG_INT as i32))) as i32)) != 0 { 542 } else { 543 }; continue;
}
// C line 19739
545 => {
vm_block = if (((((((((op1_4).tag) as i32)) as u32)) == ((((JS_TAG_FLOAT64 as i32)) as u32))) as i32)) != 0 { 541 } else { 544 }; continue;
}
// C line 19759
546 => {
vm_block = 29; continue;
}
// C line 19758
547 => {
vm_block = if (JS_IsException(*(sp).offset((((1 as i32)).wrapping_neg()) as isize))) != 0 { 546 } else { 528 }; continue;
}
// C line 19756
549 => {
let _ = { let assigned = JS_ConcatString(ctx, op1_4, op2); *(sp).offset((((2 as i32)).wrapping_neg()) as isize) = assigned; assigned };
// C line 19757
let _ = { let old = sp; sp = (sp).offset(-1); old };
vm_block = 547; continue;
}
// C line 19765
550 => {
let _ = { let old = sp; sp = (sp).offset(-1); old };
vm_block = 528; continue;
}
// C line 19764
551 => {
vm_block = 29; continue;
}
// C line 19763
552 => {
vm_block = if (js_add_slow(ctx, sp)) != 0 { 551 } else { 550 }; continue;
}
// C line ? labels: add_slow_case
553 => {
let _ = { let assigned = pc; (*(sf)).cur_pc = assigned; assigned };
vm_block = 552; continue;
}
// C line 19755
554 => {
vm_block = if (((((JS_IsString(op1_4)) != 0) && ((JS_IsString(op2)) != 0)) as i32)) != 0 { 549 } else { 553 }; continue;
}
// C line 19736
555 => {
vm_block = if (((((((((((((op1_4).tag) as i32)) as u32)) == ((((JS_TAG_FLOAT64 as i32)) as u32))) as i32)) != 0) || ((((((((((op2).tag) as i32)) as u32)) == ((((JS_TAG_FLOAT64 as i32)) as u32))) as i32)) != 0)) as i32)) != 0 { 545 } else { 554 }; continue;
}
// C line 19727
556 => {
vm_block = if ((((!(((!((((((((((op1_4).tag) as i32)) | ((((op2).tag) as i32)))) == ((0 as i32))) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 533 } else { 555 }; continue;
}
// C line 19725
558 => {
let _ = { let assigned = *(sp).offset((((2 as i32)).wrapping_neg()) as isize); op1_4 = assigned; assigned };
// C line 19726
let _ = { let assigned = *(sp).offset((((1 as i32)).wrapping_neg()) as isize); op2 = assigned; assigned };
vm_block = 556; continue;
}
// C line 19720
559 => {
vm_block = 30; continue;
}
// C line 19718
560 => {
vm_block = 29; continue;
}
// C line 19715
561 => {
vm_block = if (JS_CopyDataProperties(ctx, *(sp).offset(((((1 as i32)).wrapping_neg()).wrapping_sub(((mask) & ((3 as i32))))) as isize), *(sp).offset(((((1 as i32)).wrapping_neg()).wrapping_sub((((mask).wrapping_shr(((2 as i32)) as u32)) & ((7 as i32))))) as isize), *(sp).offset(((((1 as i32)).wrapping_neg()).wrapping_sub((((mask).wrapping_shr(((5 as i32)) as u32)) & ((7 as i32))))) as isize), (0 as i32))) != 0 { 560 } else { 559 }; continue;
}
// C line 19713
563 => {
let _ = { let assigned = ((*({ let old = pc; pc = (pc).offset(1); old })) as i32); mask = assigned; assigned };
// C line 19714
let _ = { let assigned = pc; (*(sf)).cur_pc = assigned; assigned };
vm_block = 561; continue;
}
// C line 19701
565 => {
let _ = JS_FreeValue(ctx, *({ sp = (sp).offset(-1); sp }));
// C line 19703
vm_block = 30; continue;
}
// C line 19700
566 => {
vm_block = 29; continue;
}
// C line 19699
567 => {
vm_block = if (js_append_enumerate(ctx, sp)) != 0 { 566 } else { 565 }; continue;
}
// C line 19698
568 => {
let _ = { let assigned = pc; (*(sf)).cur_pc = assigned; assigned };
vm_block = 567; continue;
}
// C line 19694
569 => {
vm_block = 30; continue;
}
// C line 19692
570 => {
vm_block = 29; continue;
}
// C line 19691
571 => {
vm_block = if ((((!(((!(((((ret_16) < ((0 as i32))) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 570 } else { 569 }; continue;
}
// C line 19688
573 => {
let _ = { let assigned = JS_DefinePropertyValueValue(ctx, *(sp).offset((((3 as i32)).wrapping_neg()) as isize), JS_DupValue(ctx, *(sp).offset((((2 as i32)).wrapping_neg()) as isize)), *(sp).offset((((1 as i32)).wrapping_neg()) as isize), ((((((((1 as i32)).wrapping_shl(((0 as i32)) as u32)) | (((1 as i32)).wrapping_shl(((1 as i32)) as u32)))) | (((1 as i32)).wrapping_shl(((2 as i32)) as u32)))) | (((1 as i32)).wrapping_shl(((14 as i32)) as u32)))); ret_16 = assigned; assigned };
// C line 19690
let _ = { sp = (sp).offset(-(((1 as i32)) as isize)); sp };
vm_block = 571; continue;
}
// C line 19683
574 => {
vm_block = 30; continue;
}
// C line 19681
575 => {
vm_block = 29; continue;
}
// C line 19680
576 => {
vm_block = if ((((ret_15) < ((0 as i32))) as i32)) != 0 { 575 } else { 574 }; continue;
}
// C line 19673
582 => {
let _ = { let assigned = JS_SetPropertyInternal(ctx, *(sp).offset((((3 as i32)).wrapping_neg()) as isize), atom_14, *(sp).offset((((1 as i32)).wrapping_neg()) as isize), *(sp).offset((((4 as i32)).wrapping_neg()) as isize), ((1 as i32)).wrapping_shl(((15 as i32)) as u32)); ret_15 = assigned; assigned };
// C line 19675
let _ = JS_FreeAtom(ctx, atom_14);
// C line 19676
let _ = JS_FreeValue(ctx, *(sp).offset((((4 as i32)).wrapping_neg()) as isize));
// C line 19677
let _ = JS_FreeValue(ctx, *(sp).offset((((3 as i32)).wrapping_neg()) as isize));
// C line 19678
let _ = JS_FreeValue(ctx, *(sp).offset((((2 as i32)).wrapping_neg()) as isize));
// C line 19679
let _ = { sp = (sp).offset(-(((4 as i32)) as isize)); sp };
vm_block = 576; continue;
}
// C line 19672
583 => {
vm_block = 29; continue;
}
// C line 19671
584 => {
vm_block = if ((((!(((!(((((atom_14) == ((((0 as i32)) as JSAtom))) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 583 } else { 582 }; continue;
}
// C line 19670
585 => {
let _ = { let assigned = JS_ValueToAtom(ctx, *(sp).offset((((2 as i32)).wrapping_neg()) as isize)); atom_14 = assigned; assigned };
vm_block = 584; continue;
}
// C line 19667
587 => {
let _ = JS_ThrowTypeErrorNotAnObject(ctx);
// C line 19668
vm_block = 29; continue;
}
// C line 19666
588 => {
vm_block = if (((((((*(sp).offset((((3 as i32)).wrapping_neg()) as isize)).tag) as i32)) != ((JS_TAG_OBJECT as i32))) as i32)) != 0 { 587 } else { 585 }; continue;
}
// C line 19665
589 => {
let _ = { let assigned = pc; (*(sf)).cur_pc = assigned; assigned };
vm_block = 588; continue;
}
// C line 19659
590 => {
vm_block = 30; continue;
}
// C line 19657
591 => {
vm_block = 29; continue;
}
// C line 19656
592 => {
vm_block = if ((((!(((!(((((ret_14) < ((0 as i32))) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 591 } else { 590 }; continue;
}
// C line 19651
597 => {
let _ = { let assigned = JS_SetPropertyInternal(ctx, *(sp).offset((((3 as i32)).wrapping_neg()) as isize), atom_13, *(sp).offset((((1 as i32)).wrapping_neg()) as isize), *(sp).offset((((3 as i32)).wrapping_neg()) as isize), ((1 as i32)).wrapping_shl(((15 as i32)) as u32)); ret_14 = assigned; assigned };
// C line 19652
let _ = JS_FreeAtom(ctx, atom_13);
// C line 19653
let _ = JS_FreeValue(ctx, *(sp).offset((((2 as i32)).wrapping_neg()) as isize));
// C line 19654
let _ = JS_FreeValue(ctx, *(sp).offset((((3 as i32)).wrapping_neg()) as isize));
// C line 19655
let _ = { sp = (sp).offset(-(((3 as i32)) as isize)); sp };
vm_block = 592; continue;
}
// C line 19646
600 => {
let _ = JS_ThrowReferenceErrorNotDefined(ctx, atom_13);
// C line 19647
let _ = JS_FreeAtom(ctx, atom_13);
// C line 19648
vm_block = 29; continue;
}
// C line 19645
601 => {
vm_block = if (is_strict_mode(ctx)) != 0 { 600 } else { 597 }; continue;
}
// C line 19642
603 => {
let _ = JS_FreeAtom(ctx, atom_13);
// C line 19643
vm_block = 29; continue;
}
// C line 19641
604 => {
vm_block = if ((((!(((!(((((ret_14) < ((0 as i32))) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 603 } else { 601 }; continue;
}
// C line 19640
605 => {
vm_block = if ((((!(((!(((((ret_14) <= ((0 as i32))) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 604 } else { 597 }; continue;
}
// C line 19639
606 => {
let _ = { let assigned = JS_HasProperty(ctx, *(sp).offset((((3 as i32)).wrapping_neg()) as isize), atom_13); ret_14 = assigned; assigned };
vm_block = 605; continue;
}
// C line 19632
609 => {
let _ = JS_ThrowReferenceErrorNotDefined(ctx, atom_13);
// C line 19633
let _ = JS_FreeAtom(ctx, atom_13);
// C line 19634
vm_block = 29; continue;
}
// C line 19636
610 => {
let _ = { let assigned = JS_DupValue(ctx, (*(ctx)).global_obj); *(sp).offset((((3 as i32)).wrapping_neg()) as isize) = assigned; assigned };
vm_block = 606; continue;
}
// C line 19631
611 => {
vm_block = if (is_strict_mode(ctx)) != 0 { 609 } else { 610 }; continue;
}
// C line 19630
612 => {
vm_block = if ((((!(((!((JS_IsUndefined(*(sp).offset((((3 as i32)).wrapping_neg()) as isize))) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 611 } else { 606 }; continue;
}
// C line 19629
613 => {
vm_block = 29; continue;
}
// C line 19628
614 => {
vm_block = if ((((!(((!(((((atom_13) == ((((0 as i32)) as JSAtom))) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 613 } else { 612 }; continue;
}
// C line 19626
616 => {
let _ = { let assigned = pc; (*(sf)).cur_pc = assigned; assigned };
// C line 19627
let _ = { let assigned = JS_ValueToAtom(ctx, *(sp).offset((((2 as i32)).wrapping_neg()) as isize)); atom_13 = assigned; assigned };
vm_block = 614; continue;
}
// C line 19620
617 => {
vm_block = 30; continue;
}
// C line 19608
619 => {
let _ = JS_FreeValue(ctx, *(sp).offset((((3 as i32)).wrapping_neg()) as isize));
// C line 19609
let _ = { sp = (sp).offset(-(((3 as i32)) as isize)); sp };
vm_block = 617; continue;
}
// C line 19603
621 => {
let _ = { let assigned = new_len; (((*(p_7)).u).array).count = assigned; assigned };
// C line 19604
let _ = { let assigned = *(sp).offset((((1 as i32)).wrapping_neg()) as isize); *(((((*(p_7)).u).array).u).values).offset((idx_25) as isize) = assigned; assigned };
vm_block = 619; continue;
}
// C line 19601
622 => {
let _ = { let assigned = JS_NewInt32(ctx, ((new_len) as i32)); ((*((*(p_7)).prop).offset(((0 as i32)) as isize)).u).value = assigned; assigned };
vm_block = 621; continue;
}
// C line 19600
623 => {
vm_block = 645; continue;
}
// C line 19599
624 => {
vm_block = if ((((!(((!(((!(((((((*(get_shape_prop((*(p_7)).shape))).flags()) as i32)) & (((1 as i32)).wrapping_shl(((1 as i32)) as u32)))) != 0) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 623 } else { 622 }; continue;
}
// C line 19598
625 => {
vm_block = if ((((new_len) > (array_len)) as i32)) != 0 { 624 } else { 621 }; continue;
}
// C line 19597
626 => {
let _ = { let assigned = ((((((((*((*(p_7)).prop).offset(((0 as i32)) as isize)).u).value).u).uint64) as i32)) as u32); array_len = assigned; assigned };
vm_block = 625; continue;
}
// C line 19596
627 => {
vm_block = 645; continue;
}
// C line 19595
628 => {
vm_block = if ((((!(((!(((((new_len) > (((((*(p_7)).u).array).u1).size)) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 627 } else { 626 }; continue;
}
// C line 19594
629 => {
let _ = { let assigned = (idx_25).wrapping_add((((1 as i32)) as u32)); new_len = assigned; assigned };
vm_block = 628; continue;
}
// C line 19592
630 => {
vm_block = 645; continue;
}
// C line 19591
631 => {
vm_block = if ((((!(((!((((((((((*((*(p_7)).prop).offset(((0 as i32)) as isize)).u).value).tag) as i32)) != ((JS_TAG_INT as i32))) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 630 } else { 629 }; continue;
}
// C line 19589
632 => {
vm_block = 645; continue;
}
// C line 19586
633 => {
vm_block = if ((((!(((!(((((((((((((idx_25) != ((((*(p_7)).u).array).count)) as i32)) != 0) || (((!(((*(p_7)).fast_array()) != 0) as i32)) != 0)) as i32)) != 0) || (((!((can_extend_fast_array(p_7)) != 0) as i32)) != 0)) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 632 } else { 631 }; continue;
}
// C line 19606
634 => {
let _ = set_value(ctx, core::ptr::addr_of_mut!(*(((((*(p_7)).u).array).u).values).offset((idx_25) as isize)), *(sp).offset((((1 as i32)).wrapping_neg()) as isize));
vm_block = 619; continue;
}
// C line 19584
635 => {
vm_block = if ((((!(((!(((((idx_25) >= ((((*(p_7)).u).array).count)) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 633 } else { 634 }; continue;
}
// C line 19583
636 => {
vm_block = 645; continue;
}
// C line 19582
637 => {
vm_block = if ((((!(((!((((((((*(p_7)).class_id) as i32)) != ((JS_CLASS_ARRAY as i32))) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 636 } else { 635 }; continue;
}
// C line 19580
639 => {
let _ = { let assigned = ((((*(sp).offset((((3 as i32)).wrapping_neg()) as isize)).u).ptr) as *mut JSObject); p_7 = assigned; assigned };
// C line 19581
let _ = { let assigned = ((((((*(sp).offset((((2 as i32)).wrapping_neg()) as isize)).u).uint64) as i32)) as u32); idx_25 = assigned; assigned };
vm_block = 637; continue;
}
// C line 19617
640 => {
vm_block = 29; continue;
}
// C line 19616
641 => {
vm_block = if ((((!(((!(((((ret_13) < ((0 as i32))) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 640 } else { 617 }; continue;
}
// C line ? labels: put_array_el_slow_path
645 => {
let _ = { let assigned = pc; (*(sf)).cur_pc = assigned; assigned };
// C line 19613
let _ = { let assigned = JS_SetPropertyValue(ctx, *(sp).offset((((3 as i32)).wrapping_neg()) as isize), *(sp).offset((((2 as i32)).wrapping_neg()) as isize), *(sp).offset((((1 as i32)).wrapping_neg()) as isize), ((1 as i32)).wrapping_shl(((15 as i32)) as u32)); ret_13 = assigned; assigned };
// C line 19614
let _ = JS_FreeValue(ctx, *(sp).offset((((3 as i32)).wrapping_neg()) as isize));
// C line 19615
let _ = { sp = (sp).offset(-(((3 as i32)) as isize)); sp };
vm_block = 641; continue;
}
// C line 19578
646 => {
vm_block = if ((((!(((!((((((((((((*(sp).offset((((3 as i32)).wrapping_neg()) as isize)).tag) as i32)) == ((JS_TAG_OBJECT as i32))) as i32)) != 0) && ((((((((*(sp).offset((((2 as i32)).wrapping_neg()) as isize)).tag) as i32)) == ((JS_TAG_INT as i32))) as i32)) != 0)) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 639 } else { 645 }; continue;
}
// C line 19564
652 => {
let _ = JS_FreeValue(ctx, *(sp).offset((((1 as i32)).wrapping_neg()) as isize));
// C line 19565
let _ = JS_FreeValue(ctx, *(sp).offset((((2 as i32)).wrapping_neg()) as isize));
// C line 19566
let _ = JS_FreeValue(ctx, *(sp).offset((((3 as i32)).wrapping_neg()) as isize));
// C line 19567
let _ = { let assigned = val_13; *(sp).offset((((3 as i32)).wrapping_neg()) as isize) = assigned; assigned };
// C line 19568
let _ = { sp = (sp).offset(-(((2 as i32)) as isize)); sp };
// C line 19570
vm_block = 30; continue;
}
// C line 19563
653 => {
vm_block = 29; continue;
}
// C line 19562
654 => {
vm_block = if ((((!(((!((JS_IsException(val_13)) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 653 } else { 652 }; continue;
}
// C line 19560
656 => {
let _ = { let assigned = JS_GetPropertyInternal(ctx, *(sp).offset((((2 as i32)).wrapping_neg()) as isize), atom_12, *(sp).offset((((3 as i32)).wrapping_neg()) as isize), (0 as i32)); val_13 = assigned; assigned };
// C line 19561
let _ = JS_FreeAtom(ctx, atom_12);
vm_block = 654; continue;
}
// C line 19559
657 => {
vm_block = 29; continue;
}
// C line 19558
658 => {
vm_block = if ((((!(((!(((((atom_12) == ((((0 as i32)) as JSAtom))) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 657 } else { 656 }; continue;
}
// C line 19556
660 => {
let _ = { let assigned = pc; (*(sf)).cur_pc = assigned; assigned };
// C line 19557
let _ = { let assigned = JS_ValueToAtom(ctx, *(sp).offset((((1 as i32)).wrapping_neg()) as isize)); atom_12 = assigned; assigned };
vm_block = 658; continue;
}
// C line 19547
663 => {
let _ = { let assigned = val_12; *(sp).offset(((0 as i32)) as isize) = assigned; assigned };
// C line 19548
let _ = { let old = sp; sp = (sp).offset(1); old };
// C line 19550
vm_block = 30; continue;
}
// C line 19546
664 => {
vm_block = 29; continue;
}
// C line 19545
665 => {
vm_block = if ((((!(((!((JS_IsException(val_12)) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 664 } else { 663 }; continue;
}
// C line 19544
666 => {
let _ = JS_FreeAtom(ctx, atom_11);
vm_block = 665; continue;
}
// C line 19540
667 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; val_12 = assigned; assigned };
vm_block = 666; continue;
}
// C line 19536
670 => {
let _ = JS_ThrowReferenceErrorNotDefined(ctx, atom_11);
// C line 19537
let _ = JS_FreeAtom(ctx, atom_11);
// C line 19538
vm_block = 29; continue;
}
// C line 19535
671 => {
vm_block = if (is_strict_mode(ctx)) != 0 { 670 } else { 667 }; continue;
}
// C line 19532
673 => {
let _ = JS_FreeAtom(ctx, atom_11);
// C line 19533
vm_block = 29; continue;
}
// C line 19531
674 => {
vm_block = if ((((ret_12) < ((0 as i32))) as i32)) != 0 { 673 } else { 671 }; continue;
}
// C line 19542
675 => {
let _ = { let assigned = JS_GetProperty(ctx, *(sp).offset((((2 as i32)).wrapping_neg()) as isize), atom_11); val_12 = assigned; assigned };
vm_block = 666; continue;
}
// C line 19530
676 => {
vm_block = if ((((!(((!(((((ret_12) <= ((0 as i32))) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 674 } else { 675 }; continue;
}
// C line 19529
677 => {
let _ = { let assigned = JS_HasProperty(ctx, *(sp).offset((((2 as i32)).wrapping_neg()) as isize), atom_11); ret_12 = assigned; assigned };
vm_block = 676; continue;
}
// C line 19525
680 => {
let _ = JS_ThrowReferenceErrorNotDefined(ctx, atom_11);
// C line 19526
let _ = JS_FreeAtom(ctx, atom_11);
// C line 19527
vm_block = 29; continue;
}
// C line 19524
681 => {
vm_block = if ((((!(((!((JS_IsUndefined(*(sp).offset((((2 as i32)).wrapping_neg()) as isize))) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 680 } else { 677 }; continue;
}
// C line 19523
682 => {
vm_block = 29; continue;
}
// C line 19522
683 => {
vm_block = if ((((atom_11) == ((((0 as i32)) as JSAtom))) as i32)) != 0 { 682 } else { 681 }; continue;
}
// C line 19520
685 => {
let _ = { let assigned = pc; (*(sf)).cur_pc = assigned; assigned };
// C line 19521
let _ = { let assigned = JS_ValueToAtom(ctx, *(sp).offset((((1 as i32)).wrapping_neg()) as isize)); atom_11 = assigned; assigned };
vm_block = 683; continue;
}
// C line 19510
687 => {
let _ = { let assigned = val_11; *({ let old = sp; sp = (sp).offset(1); old }) = assigned; assigned };
// C line 19512
vm_block = 30; continue;
}
// C line 19482
688 => {
let _ = { let assigned = JS_DupValue(ctx, *(((((*(p_6)).u).array).u).values).offset((idx_24) as isize)); val_11 = assigned; assigned };
vm_block = 687; continue;
}
// C line 19481
689 => {
vm_block = 710; continue;
}
// C line 19480
690 => {
vm_block = if ((((!(((!(((((idx_24) >= ((((*(p_6)).u).array).count)) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 689 } else { 688 }; continue;
}
// C line 19479
691 => {
vm_block = 710; continue;
}
// C line 19478
692 => {
vm_block = if ((((!(((!((((((((*(p_6)).class_id) as i32)) != ((JS_CLASS_ARRAY as i32))) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 691 } else { 690 }; continue;
}
// C line 19476
694 => {
let _ = { let assigned = ((((*(sp).offset((((2 as i32)).wrapping_neg()) as isize)).u).ptr) as *mut JSObject); p_6 = assigned; assigned };
// C line 19477
let _ = { let assigned = ((((((*(sp).offset((((1 as i32)).wrapping_neg()) as isize)).u).uint64) as i32)) as u32); idx_24 = assigned; assigned };
vm_block = 692; continue;
}
// C line 19508
695 => {
vm_block = 29; continue;
}
// C line 19507
696 => {
vm_block = if ((((!(((!((JS_IsException(val_11)) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 695 } else { 687 }; continue;
}
// C line 19505
698 => {
let _ = { let assigned = pc; (*(sf)).cur_pc = assigned; assigned };
// C line 19506
let _ = { let assigned = JS_GetPropertyValue(ctx, *(sp).offset((((2 as i32)).wrapping_neg()) as isize), JS_DupValue(ctx, *(sp).offset((((1 as i32)).wrapping_neg()) as isize))); val_11 = assigned; assigned };
vm_block = 696; continue;
}
// C line 19501
701 => {
let _ = JS_FreeValue(ctx, *(sp).offset((((1 as i32)).wrapping_neg()) as isize));
// C line 19502
let _ = { let assigned = ret_val; *(sp).offset((((1 as i32)).wrapping_neg()) as isize) = assigned; assigned };
// C line 19503
vm_block = 698; continue;
}
// C line 19500
702 => {
vm_block = 29; continue;
}
// C line 19499
703 => {
vm_block = if (JS_IsException(ret_val)) != 0 { 702 } else { 701 }; continue;
}
// C line 19497
705 => {
let _ = { let assigned = pc; (*(sf)).cur_pc = assigned; assigned };
// C line 19498
let _ = { let assigned = JS_ToPropertyKey(ctx, *(sp).offset((((1 as i32)).wrapping_neg()) as isize)); ret_val = assigned; assigned };
vm_block = 703; continue;
}
// C line 19494
707 => {
let _ = JS_ThrowTypeError(ctx, c"value has no property".as_ptr());
// C line 19495
vm_block = 29; continue;
}
// C line 19493
708 => {
vm_block = if ((((!(((!((((((JS_IsUndefined(*(sp).offset((((2 as i32)).wrapping_neg()) as isize))) != 0) || ((JS_IsNull(*(sp).offset((((2 as i32)).wrapping_neg()) as isize))) != 0)) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 707 } else { 705 }; continue;
}
// C line 19490
709 => {
vm_block = 698; continue;
}
// C line 19485 labels: get_array_el3_slow_path
710 => {
vm_block = match (((*(sp).offset((((1 as i32)).wrapping_neg()) as isize)).tag) as i32) { x if x == (JS_TAG_SYMBOL as i32) => 709, x if x == (JS_TAG_STRING as i32) => 709, x if x == (JS_TAG_INT as i32) => 709, _ => 708, }; continue;
}
// C line 19474
711 => {
vm_block = if ((((!(((!((((((((((((*(sp).offset((((2 as i32)).wrapping_neg()) as isize)).tag) as i32)) == ((JS_TAG_OBJECT as i32))) as i32)) != 0) && ((((((((*(sp).offset((((1 as i32)).wrapping_neg()) as isize)).tag) as i32)) == ((JS_TAG_INT as i32))) as i32)) != 0)) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 694 } else { 710 }; continue;
}
// C line 19466
712 => {
vm_block = 30; continue;
}
// C line 19465
713 => {
let _ = { let assigned = val_10; *(sp).offset((((1 as i32)).wrapping_neg()) as isize) = assigned; assigned };
vm_block = 712; continue;
}
// C line 19465
716 => {
let _ = JS_FreeValue(ctx, obj_7);
// C line 19465
let _ = { let assigned = val_10; *(sp).offset((((2 as i32)).wrapping_neg()) as isize) = assigned; assigned };
// C line 19465
let _ = { let old = sp; sp = (sp).offset(-1); old };
vm_block = 712; continue;
}
// C line 19465
717 => {
vm_block = if ((1 as i32)) != 0 { 713 } else { 716 }; continue;
}
// C line 19465
718 => {
let _ = { let assigned = JS_DupValue(ctx, *(((((*(p_5)).u).array).u).values).offset((idx_23) as isize)); val_10 = assigned; assigned };
vm_block = 717; continue;
}
// C line 19465
719 => {
vm_block = 731; continue;
}
// C line 19465
720 => {
vm_block = if ((((!(((!(((((idx_23) >= ((((*(p_5)).u).array).count)) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 719 } else { 718 }; continue;
}
// C line 19465
721 => {
vm_block = 731; continue;
}
// C line 19465
722 => {
vm_block = if ((((!(((!((((((((*(p_5)).class_id) as i32)) != ((JS_CLASS_ARRAY as i32))) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 721 } else { 720 }; continue;
}
// C line 19465
724 => {
let _ = { let assigned = ((((obj_7).u).ptr) as *mut JSObject); p_5 = assigned; assigned };
// C line 19465
let _ = { let assigned = ((((((prop_1).u).uint64) as i32)) as u32); idx_23 = assigned; assigned };
vm_block = 722; continue;
}
// C line 19465
725 => {
vm_block = 29; continue;
}
// C line 19465
726 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; *(sp).offset((((1 as i32)).wrapping_neg()) as isize) = assigned; assigned };
vm_block = 725; continue;
}
// C line 19465
727 => {
let _ = { let old = sp; sp = (sp).offset(-1); old };
vm_block = 725; continue;
}
// C line 19465
728 => {
vm_block = if ((1 as i32)) != 0 { 726 } else { 727 }; continue;
}
// C line 19465
729 => {
vm_block = if ((((!(((!((JS_IsException(val_10)) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 728 } else { 717 }; continue;
}
// C line 19465 labels: get_array_el2_slow_path
731 => {
let _ = { let assigned = pc; (*(sf)).cur_pc = assigned; assigned };
// C line 19465
let _ = { let assigned = JS_GetPropertyValue(ctx, obj_7, prop_1); val_10 = assigned; assigned };
vm_block = 729; continue;
}
// C line 19465
732 => {
vm_block = if ((((!(((!((((((((((((obj_7).tag) as i32)) == ((JS_TAG_OBJECT as i32))) as i32)) != 0) && ((((((((prop_1).tag) as i32)) == ((JS_TAG_INT as i32))) as i32)) != 0)) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 724 } else { 731 }; continue;
}
// C line 19465
734 => {
let _ = { let assigned = *(sp).offset((((2 as i32)).wrapping_neg()) as isize); obj_7 = assigned; assigned };
// C line 19465
let _ = { let assigned = *(sp).offset((((1 as i32)).wrapping_neg()) as isize); prop_1 = assigned; assigned };
vm_block = 732; continue;
}
// C line 19462
735 => {
vm_block = 30; continue;
}
// C line 19461
736 => {
let _ = { let assigned = val_9; *(sp).offset((((1 as i32)).wrapping_neg()) as isize) = assigned; assigned };
vm_block = 735; continue;
}
// C line 19461
739 => {
let _ = JS_FreeValue(ctx, obj_6);
// C line 19461
let _ = { let assigned = val_9; *(sp).offset((((2 as i32)).wrapping_neg()) as isize) = assigned; assigned };
// C line 19461
let _ = { let old = sp; sp = (sp).offset(-1); old };
vm_block = 735; continue;
}
// C line 19461
740 => {
vm_block = if ((0 as i32)) != 0 { 736 } else { 739 }; continue;
}
// C line 19461
741 => {
let _ = { let assigned = JS_DupValue(ctx, *(((((*(p_4)).u).array).u).values).offset((idx_22) as isize)); val_9 = assigned; assigned };
vm_block = 740; continue;
}
// C line 19461
742 => {
vm_block = 754; continue;
}
// C line 19461
743 => {
vm_block = if ((((!(((!(((((idx_22) >= ((((*(p_4)).u).array).count)) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 742 } else { 741 }; continue;
}
// C line 19461
744 => {
vm_block = 754; continue;
}
// C line 19461
745 => {
vm_block = if ((((!(((!((((((((*(p_4)).class_id) as i32)) != ((JS_CLASS_ARRAY as i32))) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 744 } else { 743 }; continue;
}
// C line 19461
747 => {
let _ = { let assigned = ((((obj_6).u).ptr) as *mut JSObject); p_4 = assigned; assigned };
// C line 19461
let _ = { let assigned = ((((((prop).u).uint64) as i32)) as u32); idx_22 = assigned; assigned };
vm_block = 745; continue;
}
// C line 19461
748 => {
vm_block = 29; continue;
}
// C line 19461
749 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; *(sp).offset((((1 as i32)).wrapping_neg()) as isize) = assigned; assigned };
vm_block = 748; continue;
}
// C line 19461
750 => {
let _ = { let old = sp; sp = (sp).offset(-1); old };
vm_block = 748; continue;
}
// C line 19461
751 => {
vm_block = if ((0 as i32)) != 0 { 749 } else { 750 }; continue;
}
// C line 19461
752 => {
vm_block = if ((((!(((!((JS_IsException(val_9)) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 751 } else { 740 }; continue;
}
// C line 19461 labels: get_array_el_slow_path
754 => {
let _ = { let assigned = pc; (*(sf)).cur_pc = assigned; assigned };
// C line 19461
let _ = { let assigned = JS_GetPropertyValue(ctx, obj_6, prop); val_9 = assigned; assigned };
vm_block = 752; continue;
}
// C line 19461
755 => {
vm_block = if ((((!(((!((((((((((((obj_6).tag) as i32)) == ((JS_TAG_OBJECT as i32))) as i32)) != 0) && ((((((((prop).tag) as i32)) == ((JS_TAG_INT as i32))) as i32)) != 0)) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 747 } else { 754 }; continue;
}
// C line 19461
757 => {
let _ = { let assigned = *(sp).offset((((2 as i32)).wrapping_neg()) as isize); obj_6 = assigned; assigned };
// C line 19461
let _ = { let assigned = *(sp).offset((((1 as i32)).wrapping_neg()) as isize); prop = assigned; assigned };
vm_block = 755; continue;
}
// C line 19420
758 => {
vm_block = 30; continue;
}
// C line 19418
759 => {
vm_block = 29; continue;
}
// C line 19415
760 => {
vm_block = if ((((js_op_define_class(ctx, sp, atom_10, class_flags, var_refs, sf, (((opcode) == ((OP_define_class_computed as i32))) as i32))) < ((0 as i32))) as i32)) != 0 { 759 } else { 758 }; continue;
}
// C line 19412
763 => {
let _ = { let assigned = crate::cutils_header::get_u32(pc); atom_10 = assigned; assigned };
// C line 19413
let _ = { let assigned = ((*(pc).offset(((4 as i32)) as isize)) as i32); class_flags = assigned; assigned };
// C line 19414
let _ = { pc = (pc).offset((((5 as i32)) as isize)); pc };
vm_block = 760; continue;
}
// C line 19404
764 => {
vm_block = 30; continue;
}
// C line 19402
765 => {
vm_block = 29; continue;
}
// C line 19401
766 => {
vm_block = if ((((!(((!(((((ret_11) < ((0 as i32))) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 765 } else { 764 }; continue;
}
// C line 19400
767 => {
let _ = { sp = (sp).offset(-((((1 as i32)).wrapping_add(is_computed)) as isize)); sp };
vm_block = 766; continue;
}
// C line 19397
769 => {
let _ = JS_FreeAtom(ctx, atom_9);
// C line 19398
let _ = JS_FreeValue(ctx, *(sp).offset((((2 as i32)).wrapping_neg()) as isize));
vm_block = 767; continue;
}
// C line 19396
770 => {
vm_block = if (is_computed) != 0 { 769 } else { 767 }; continue;
}
// C line 19395
771 => {
let _ = JS_FreeValue(ctx, *(sp).offset((((1 as i32)).wrapping_neg()) as isize));
vm_block = 770; continue;
}
// C line 19392
772 => {
let _ = { let assigned = JS_DefineProperty(ctx, obj_5, atom_9, value, getter, setter, flags_2); ret_11 = assigned; assigned };
vm_block = 771; continue;
}
// C line 19391
773 => {
vm_block = if ((((ret_11) >= ((0 as i32))) as i32)) != 0 { 772 } else { 771 }; continue;
}
// C line 19390
774 => {
let _ = { let assigned = js_method_set_properties(ctx, *(sp).offset((((1 as i32)).wrapping_neg()) as isize), atom_9, flags_2, obj_5); ret_11 = assigned; assigned };
vm_block = 773; continue;
}
// C line 19381
776 => {
let _ = { let assigned = *(sp).offset((((1 as i32)).wrapping_neg()) as isize); value = assigned; assigned };
// C line 19382
let _ = { flags_2 = ((flags_2) | (((((((1 as i32)).wrapping_shl(((13 as i32)) as u32)) | (((1 as i32)).wrapping_shl(((9 as i32)) as u32)))) | (((1 as i32)).wrapping_shl(((1 as i32)) as u32))))); flags_2 };
vm_block = 774; continue;
}
// C line 19384
778 => {
let _ = { let assigned = *(sp).offset((((1 as i32)).wrapping_neg()) as isize); getter = assigned; assigned };
// C line 19385
let _ = { flags_2 = ((flags_2) | (((1 as i32)).wrapping_shl(((11 as i32)) as u32))); flags_2 };
vm_block = 774; continue;
}
// C line 19387
780 => {
let _ = { let assigned = *(sp).offset((((1 as i32)).wrapping_neg()) as isize); setter = assigned; assigned };
// C line 19388
let _ = { flags_2 = ((flags_2) | (((1 as i32)).wrapping_shl(((12 as i32)) as u32))); flags_2 };
vm_block = 774; continue;
}
// C line 19383
781 => {
vm_block = if ((((op_flags) == ((1 as i32))) as i32)) != 0 { 778 } else { 780 }; continue;
}
// C line 19380
782 => {
vm_block = if ((((op_flags) == ((0 as i32))) as i32)) != 0 { 776 } else { 781 }; continue;
}
// C line 19376
786 => {
let _ = { op_flags = ((op_flags) & ((3 as i32))); op_flags };
// C line 19377
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; value = assigned; assigned };
// C line 19378
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; getter = assigned; assigned };
// C line 19379
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; setter = assigned; assigned };
vm_block = 782; continue;
}
// C line 19375
787 => {
let _ = { flags_2 = ((flags_2) | (((1 as i32)).wrapping_shl(((2 as i32)) as u32))); flags_2 };
vm_block = 786; continue;
}
// C line 19374
788 => {
vm_block = if (((op_flags) & ((4 as i32)))) != 0 { 787 } else { 786 }; continue;
}
// C line 19369
791 => {
let _ = { let assigned = ((*({ let old = pc; pc = (pc).offset(1); old })) as i32); op_flags = assigned; assigned };
// C line 19371
let _ = { let assigned = *(sp).offset(((((2 as i32)).wrapping_neg()).wrapping_sub(is_computed)) as isize); obj_5 = assigned; assigned };
// C line 19372
let _ = { let assigned = ((((((((1 as i32)).wrapping_shl(((8 as i32)) as u32)) | (((1 as i32)).wrapping_shl(((0 as i32)) as u32)))) | (((1 as i32)).wrapping_shl(((10 as i32)) as u32)))) | (((1 as i32)).wrapping_shl(((14 as i32)) as u32))); flags_2 = assigned; assigned };
vm_block = 788; continue;
}
// C line 19364
792 => {
let _ = { opcode = (opcode).wrapping_add(((OP_define_method as i32)).wrapping_sub((OP_define_method_computed as i32))); opcode };
vm_block = 791; continue;
}
// C line 19363
793 => {
vm_block = 29; continue;
}
// C line 19362
794 => {
vm_block = if ((((!(((!(((((atom_9) == ((((0 as i32)) as JSAtom))) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 793 } else { 792 }; continue;
}
// C line 19361
795 => {
let _ = { let assigned = JS_ValueToAtom(ctx, *(sp).offset((((2 as i32)).wrapping_neg()) as isize)); atom_9 = assigned; assigned };
vm_block = 794; continue;
}
// C line 19366
797 => {
let _ = { let assigned = crate::cutils_header::get_u32(pc); atom_9 = assigned; assigned };
// C line 19367
let _ = { pc = (pc).offset((((4 as i32)) as isize)); pc };
vm_block = 791; continue;
}
// C line 19360
798 => {
vm_block = if (is_computed) != 0 { 795 } else { 797 }; continue;
}
// C line 19359
799 => {
let _ = { let assigned = (((opcode) == ((OP_define_method_computed as i32))) as i32); is_computed = assigned; assigned };
vm_block = 798; continue;
}
// C line 19344
801 => {
let _ = js_method_set_home_object(ctx, *(sp).offset((((1 as i32)).wrapping_neg()) as isize), *(sp).offset((((2 as i32)).wrapping_neg()) as isize));
// C line 19345
vm_block = 30; continue;
}
// C line 19339
804 => {
let _ = JS_FreeValue(ctx, proto_1);
// C line 19340
let _ = { let old = sp; sp = (sp).offset(-1); old };
// C line 19342
vm_block = 30; continue;
}
// C line 19337
805 => {
vm_block = 29; continue;
}
// C line 19336
806 => {
vm_block = if ((((JS_SetPrototypeInternal(ctx, *(sp).offset((((2 as i32)).wrapping_neg()) as isize), proto_1, (1 as i32))) < ((0 as i32))) as i32)) != 0 { 805 } else { 804 }; continue;
}
// C line 19335
807 => {
vm_block = if (((((JS_IsObject(proto_1)) != 0) || ((JS_IsNull(proto_1)) != 0)) as i32)) != 0 { 806 } else { 804 }; continue;
}
// C line 19333
809 => {
let _ = { let assigned = pc; (*(sf)).cur_pc = assigned; assigned };
// C line 19334
let _ = { let assigned = *(sp).offset((((1 as i32)).wrapping_neg()) as isize); proto_1 = assigned; assigned };
vm_block = 807; continue;
}
// C line 19329
810 => {
vm_block = 30; continue;
}
// C line 19327
811 => {
vm_block = 29; continue;
}
// C line 19326
812 => {
vm_block = if ((((!(((!(((((ret_10) < ((0 as i32))) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 811 } else { 810 }; continue;
}
// C line 19325
813 => {
let _ = { let assigned = JS_DefineObjectNameComputed(ctx, *(sp).offset((((1 as i32)).wrapping_neg()) as isize), *(sp).offset((((2 as i32)).wrapping_neg()) as isize), ((1 as i32)).wrapping_shl(((0 as i32)) as u32)); ret_10 = assigned; assigned };
vm_block = 812; continue;
}
// C line 19321
814 => {
vm_block = 30; continue;
}
// C line 19319
815 => {
vm_block = 29; continue;
}
// C line 19318
816 => {
vm_block = if ((((!(((!(((((ret_9) < ((0 as i32))) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 815 } else { 814 }; continue;
}
// C line 19314
819 => {
let _ = { let assigned = crate::cutils_header::get_u32(pc); atom_8 = assigned; assigned };
// C line 19315
let _ = { pc = (pc).offset((((4 as i32)) as isize)); pc };
// C line 19317
let _ = { let assigned = JS_DefineObjectName(ctx, *(sp).offset((((1 as i32)).wrapping_neg()) as isize), atom_8, ((1 as i32)).wrapping_shl(((0 as i32)) as u32)); ret_9 = assigned; assigned };
vm_block = 816; continue;
}
// C line 19308
820 => {
vm_block = 30; continue;
}
// C line 19306
821 => {
vm_block = 29; continue;
}
// C line 19305
822 => {
vm_block = if ((((!(((!(((((ret_8) < ((0 as i32))) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 821 } else { 820 }; continue;
}
// C line 19299
826 => {
let _ = { let assigned = crate::cutils_header::get_u32(pc); atom_7 = assigned; assigned };
// C line 19300
let _ = { pc = (pc).offset((((4 as i32)) as isize)); pc };
// C line 19302
let _ = { let assigned = JS_DefinePropertyValue(ctx, *(sp).offset((((2 as i32)).wrapping_neg()) as isize), atom_7, *(sp).offset((((1 as i32)).wrapping_neg()) as isize), ((((((((1 as i32)).wrapping_shl(((0 as i32)) as u32)) | (((1 as i32)).wrapping_shl(((1 as i32)) as u32)))) | (((1 as i32)).wrapping_shl(((2 as i32)) as u32)))) | (((1 as i32)).wrapping_shl(((14 as i32)) as u32)))); ret_8 = assigned; assigned };
// C line 19304
let _ = { let old = sp; sp = (sp).offset(-1); old };
vm_block = 822; continue;
}
// C line 19293
827 => {
vm_block = 30; continue;
}
// C line 19291
828 => {
vm_block = 29; continue;
}
// C line 19290
829 => {
vm_block = if ((((!(((!(((((ret_7) < ((0 as i32))) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 828 } else { 827 }; continue;
}
// C line 19287
832 => {
let _ = { let assigned = JS_DefinePrivateField(ctx, *(sp).offset((((3 as i32)).wrapping_neg()) as isize), *(sp).offset((((2 as i32)).wrapping_neg()) as isize), *(sp).offset((((1 as i32)).wrapping_neg()) as isize)); ret_7 = assigned; assigned };
// C line 19288
let _ = JS_FreeValue(ctx, *(sp).offset((((2 as i32)).wrapping_neg()) as isize));
// C line 19289
let _ = { sp = (sp).offset(-(((2 as i32)) as isize)); sp };
vm_block = 829; continue;
}
// C line 19282
833 => {
vm_block = 30; continue;
}
// C line 19280
834 => {
vm_block = 29; continue;
}
// C line 19279
835 => {
vm_block = if ((((!(((!(((((ret_6) < ((0 as i32))) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 834 } else { 833 }; continue;
}
// C line 19275
839 => {
let _ = { let assigned = JS_SetPrivateField(ctx, *(sp).offset((((3 as i32)).wrapping_neg()) as isize), *(sp).offset((((1 as i32)).wrapping_neg()) as isize), *(sp).offset((((2 as i32)).wrapping_neg()) as isize)); ret_6 = assigned; assigned };
// C line 19276
let _ = JS_FreeValue(ctx, *(sp).offset((((3 as i32)).wrapping_neg()) as isize));
// C line 19277
let _ = JS_FreeValue(ctx, *(sp).offset((((1 as i32)).wrapping_neg()) as isize));
// C line 19278
let _ = { sp = (sp).offset(-(((3 as i32)) as isize)); sp };
vm_block = 835; continue;
}
// C line 19270
840 => {
vm_block = 30; continue;
}
// C line 19268
841 => {
vm_block = 29; continue;
}
// C line 19267
842 => {
vm_block = if ((((!(((!((JS_IsException(val_8)) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 841 } else { 840 }; continue;
}
// C line 19262
847 => {
let _ = { let assigned = JS_GetPrivateField(ctx, *(sp).offset((((2 as i32)).wrapping_neg()) as isize), *(sp).offset((((1 as i32)).wrapping_neg()) as isize)); val_8 = assigned; assigned };
// C line 19263
let _ = JS_FreeValue(ctx, *(sp).offset((((1 as i32)).wrapping_neg()) as isize));
// C line 19264
let _ = JS_FreeValue(ctx, *(sp).offset((((2 as i32)).wrapping_neg()) as isize));
// C line 19265
let _ = { let assigned = val_8; *(sp).offset((((2 as i32)).wrapping_neg()) as isize) = assigned; assigned };
// C line 19266
let _ = { let old = sp; sp = (sp).offset(-1); old };
vm_block = 842; continue;
}
// C line 19254
849 => {
let _ = { let assigned = val_7; *({ let old = sp; sp = (sp).offset(1); old }) = assigned; assigned };
// C line 19256
vm_block = 30; continue;
}
// C line 19253
850 => {
vm_block = 29; continue;
}
// C line 19252
851 => {
vm_block = if (JS_IsException(val_7)) != 0 { 850 } else { 849 }; continue;
}
// C line 19249
854 => {
let _ = { let assigned = crate::cutils_header::get_u32(pc); atom_6 = assigned; assigned };
// C line 19250
let _ = { pc = (pc).offset((((4 as i32)) as isize)); pc };
// C line 19251
let _ = { let assigned = JS_NewSymbolFromAtom(ctx, atom_6, ((JS_ATOM_TYPE_PRIVATE as i32)) as u32); val_7 = assigned; assigned };
vm_block = 851; continue;
}
// C line 19242
855 => {
vm_block = 30; continue;
}
// C line 19224
858 => {
let _ = set_value(ctx, core::ptr::addr_of_mut!(((*(pr_3)).u).value), *(sp).offset((((1 as i32)).wrapping_neg()) as isize));
// C line 19228
let _ = JS_FreeValue(ctx, obj_4);
// C line 19229
let _ = { sp = (sp).offset(-(((2 as i32)) as isize)); sp };
vm_block = 855; continue;
}
// C line 19226
859 => {
vm_block = 870; continue;
}
// C line 19221
860 => {
vm_block = if ((((!(((!((((((((((*(prs_2)).flags()) as i32)) & (((((((3 as i32)).wrapping_shl(((4 as i32)) as u32)) | (((1 as i32)).wrapping_shl(((1 as i32)) as u32)))) | (((1 as i32)).wrapping_shl(((3 as i32)) as u32)))))) == (((1 as i32)).wrapping_shl(((1 as i32)) as u32))) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 858 } else { 859 }; continue;
}
// C line 19220
861 => {
vm_block = 870; continue;
}
// C line 19219
862 => {
vm_block = if ((!(!(prs_2).is_null()) as i32)) != 0 { 861 } else { 860 }; continue;
}
// C line 19217
864 => {
let _ = { let assigned = ((((obj_4).u).ptr) as *mut JSObject); p_3 = assigned; assigned };
// C line 19218
let _ = { let assigned = find_own_property(core::ptr::addr_of_mut!(pr_3), p_3, atom_5); prs_2 = assigned; assigned };
vm_block = 862; continue;
}
// C line 19238
865 => {
vm_block = 29; continue;
}
// C line 19237
866 => {
vm_block = if ((((!(((!(((((ret_5) < ((0 as i32))) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 865 } else { 855 }; continue;
}
// C line ? labels: put_field_slow_path
870 => {
let _ = { let assigned = pc; (*(sf)).cur_pc = assigned; assigned };
// C line 19233
let _ = { let assigned = JS_SetPropertyInternal(ctx, obj_4, atom_5, *(sp).offset((((1 as i32)).wrapping_neg()) as isize), obj_4, ((1 as i32)).wrapping_shl(((15 as i32)) as u32)); ret_5 = assigned; assigned };
// C line 19235
let _ = JS_FreeValue(ctx, obj_4);
// C line 19236
let _ = { sp = (sp).offset(-(((2 as i32)) as isize)); sp };
vm_block = 866; continue;
}
// C line 19216
871 => {
vm_block = if ((((!(((!((((((((obj_4).tag) as i32)) == ((JS_TAG_OBJECT as i32))) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 864 } else { 870 }; continue;
}
// C line 19212
874 => {
let _ = { let assigned = crate::cutils_header::get_u32(pc); atom_5 = assigned; assigned };
// C line 19213
let _ = { pc = (pc).offset((((4 as i32)) as isize)); pc };
// C line 19215
let _ = { let assigned = *(sp).offset((((2 as i32)).wrapping_neg()) as isize); obj_4 = assigned; assigned };
vm_block = 871; continue;
}
// C line 19195
875 => {
vm_block = 30; continue;
}
// C line 19194
876 => {
let _ = { let assigned = val_6; *({ let old = sp; sp = (sp).offset(1); old }) = assigned; assigned };
vm_block = 875; continue;
}
// C line 19194
878 => {
let _ = JS_FreeValue(ctx, *(sp).offset((((1 as i32)).wrapping_neg()) as isize));
// C line 19194
let _ = { let assigned = val_6; *(sp).offset((((1 as i32)).wrapping_neg()) as isize) = assigned; assigned };
vm_block = 875; continue;
}
// C line 19194
879 => {
vm_block = if ((1 as i32)) != 0 { 876 } else { 878 }; continue;
}
// C line 19194
880 => {
// C line 19194
let _ = { let assigned = find_own_property(core::ptr::addr_of_mut!(pr_2), p_2, atom_4); prs_1 = assigned; assigned };
vm_block = 892; continue;
}
// C line 19194
882 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; val_6 = assigned; assigned };
// C line 19194
vm_block = 879; continue;
}
// C line 19194
883 => {
vm_block = if ((!(!(p_2).is_null()) as i32)) != 0 { 882 } else { 880 }; continue;
}
// C line 19194
884 => {
let _ = { let assigned = (*((*(p_2)).shape)).proto; p_2 = assigned; assigned };
vm_block = 883; continue;
}
// C line 19194
886 => {
let _ = { let assigned = JSValue { u: JSValueUnion { ptr: ((p_2) as *mut c_void) }, tag: (((JS_TAG_OBJECT as i32)) as i64) }; obj_3 = assigned; assigned };
// C line 19194
vm_block = 898; continue;
}
// C line 19194
887 => {
vm_block = if ((((!(((!(((*(p_2)).is_exotic()) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 886 } else { 884 }; continue;
}
// C line 19194
889 => {
let _ = { let assigned = JS_DupValue(ctx, ((*(pr_2)).u).value); val_6 = assigned; assigned };
// C line 19194
vm_block = 879; continue;
}
// C line 19194
890 => {
vm_block = 898; continue;
}
// C line 19194
891 => {
vm_block = if ((((!(((!(((((((*(prs_1)).flags()) as i32)) & (((3 as i32)).wrapping_shl(((4 as i32)) as u32)))) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 890 } else { 889 }; continue;
}
// C line 19194
892 => {
vm_block = if !(prs_1).is_null() { 891 } else { 887 }; continue;
}
// C line 19194
894 => {
let _ = { let assigned = ((((obj_3).u).ptr) as *mut JSObject); p_2 = assigned; assigned };
vm_block = 880; continue;
}
// C line 19194
895 => {
vm_block = 29; continue;
}
// C line 19194
896 => {
vm_block = if ((((!(((!((JS_IsException(val_6)) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 895 } else { 879 }; continue;
}
// C line 19194 labels: get_field2_slow_path
898 => {
let _ = { let assigned = pc; (*(sf)).cur_pc = assigned; assigned };
// C line 19194
let _ = { let assigned = JS_GetPropertyInternal(ctx, obj_3, atom_4, *(sp).offset((((1 as i32)).wrapping_neg()) as isize), (0 as i32)); val_6 = assigned; assigned };
vm_block = 896; continue;
}
// C line 19194
899 => {
vm_block = if ((((!(((!((((((((obj_3).tag) as i32)) == ((JS_TAG_OBJECT as i32))) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 894 } else { 898 }; continue;
}
// C line 19194
900 => {
let _ = { let assigned = *(sp).offset((((1 as i32)).wrapping_neg()) as isize); obj_3 = assigned; assigned };
js_host_error_stack_read(ctx, obj_3, atom_4);
vm_block = 899; continue;
}
// C line 19194
901 => {
let _ = { let assigned = (((JS_ATOM_length as i32)) as JSAtom); atom_4 = assigned; assigned };
vm_block = 900; continue;
}
// C line 19194
903 => {
let _ = { let assigned = crate::cutils_header::get_u32(pc); atom_4 = assigned; assigned };
// C line 19194
let _ = { pc = (pc).offset((((4 as i32)) as isize)); pc };
vm_block = 900; continue;
}
// C line 19194
904 => {
vm_block = if ((0 as i32)) != 0 { 901 } else { 903 }; continue;
}
// C line 19191
905 => {
vm_block = 30; continue;
}
// C line 19190
906 => {
let _ = { let assigned = val_5; *({ let old = sp; sp = (sp).offset(1); old }) = assigned; assigned };
vm_block = 905; continue;
}
// C line 19190
908 => {
let _ = JS_FreeValue(ctx, *(sp).offset((((1 as i32)).wrapping_neg()) as isize));
// C line 19190
let _ = { let assigned = val_5; *(sp).offset((((1 as i32)).wrapping_neg()) as isize) = assigned; assigned };
vm_block = 905; continue;
}
// C line 19190
909 => {
vm_block = if ((0 as i32)) != 0 { 906 } else { 908 }; continue;
}
// C line 19190
910 => {
// C line 19190
let _ = { let assigned = find_own_property(core::ptr::addr_of_mut!(pr_1), p_1, atom_3); prs = assigned; assigned };
vm_block = 922; continue;
}
// C line 19190
912 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; val_5 = assigned; assigned };
// C line 19190
vm_block = 909; continue;
}
// C line 19190
913 => {
vm_block = if ((!(!(p_1).is_null()) as i32)) != 0 { 912 } else { 910 }; continue;
}
// C line 19190
914 => {
let _ = { let assigned = (*((*(p_1)).shape)).proto; p_1 = assigned; assigned };
vm_block = 913; continue;
}
// C line 19190
916 => {
let _ = { let assigned = JSValue { u: JSValueUnion { ptr: ((p_1) as *mut c_void) }, tag: (((JS_TAG_OBJECT as i32)) as i64) }; obj_2 = assigned; assigned };
// C line 19190
vm_block = 928; continue;
}
// C line 19190
917 => {
vm_block = if ((((!(((!(((*(p_1)).is_exotic()) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 916 } else { 914 }; continue;
}
// C line 19190
919 => {
let _ = { let assigned = JS_DupValue(ctx, ((*(pr_1)).u).value); val_5 = assigned; assigned };
// C line 19190
vm_block = 909; continue;
}
// C line 19190
920 => {
vm_block = 928; continue;
}
// C line 19190
921 => {
vm_block = if ((((!(((!(((((((*(prs)).flags()) as i32)) & (((3 as i32)).wrapping_shl(((4 as i32)) as u32)))) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 920 } else { 919 }; continue;
}
// C line 19190
922 => {
vm_block = if !(prs).is_null() { 921 } else { 917 }; continue;
}
// C line 19190
924 => {
let _ = { let assigned = ((((obj_2).u).ptr) as *mut JSObject); p_1 = assigned; assigned };
vm_block = 910; continue;
}
// C line 19190
925 => {
vm_block = 29; continue;
}
// C line 19190
926 => {
vm_block = if ((((!(((!((JS_IsException(val_5)) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 925 } else { 909 }; continue;
}
// C line 19190 labels: get_field_slow_path
928 => {
let _ = { let assigned = pc; (*(sf)).cur_pc = assigned; assigned };
// C line 19190
let _ = { let assigned = JS_GetPropertyInternal(ctx, obj_2, atom_3, *(sp).offset((((1 as i32)).wrapping_neg()) as isize), (0 as i32)); val_5 = assigned; assigned };
vm_block = 926; continue;
}
// C line 19190
929 => {
vm_block = if ((((!(((!((((((((obj_2).tag) as i32)) == ((JS_TAG_OBJECT as i32))) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 924 } else { 928 }; continue;
}
// C line 19190
930 => {
let _ = { let assigned = *(sp).offset((((1 as i32)).wrapping_neg()) as isize); obj_2 = assigned; assigned };
js_host_error_stack_read(ctx, obj_2, atom_3);
vm_block = 929; continue;
}
// C line 19190
931 => {
let _ = { let assigned = (((JS_ATOM_length as i32)) as JSAtom); atom_3 = assigned; assigned };
vm_block = 930; continue;
}
// C line 19190
933 => {
let _ = { let assigned = crate::cutils_header::get_u32(pc); atom_3 = assigned; assigned };
// C line 19190
let _ = { pc = (pc).offset((((4 as i32)) as isize)); pc };
vm_block = 930; continue;
}
// C line 19190
934 => {
vm_block = if ((0 as i32)) != 0 { 931 } else { 933 }; continue;
}
// C line 19129
936 => {
let _ = { let assigned = JS_NewBool(ctx, (!((res_2) != 0) as i32)); *(sp).offset((((1 as i32)).wrapping_neg()) as isize) = assigned; assigned };
// C line 19131
vm_block = 30; continue;
}
// C line 19125
937 => {
let _ = { let assigned = (((((((op1_3).u).uint64) as i32)) != ((0 as i32))) as i32); res_2 = assigned; assigned };
vm_block = 936; continue;
}
// C line 19127
938 => {
let _ = { let assigned = JS_ToBoolFree(ctx, op1_3); res_2 = assigned; assigned };
vm_block = 936; continue;
}
// C line 19124
939 => {
vm_block = if (((((((((op1_3).tag) as i32)) as u32)) <= ((((JS_TAG_UNDEFINED as i32)) as u32))) as i32)) != 0 { 937 } else { 938 }; continue;
}
// C line 19123
940 => {
let _ = { let assigned = *(sp).offset((((1 as i32)).wrapping_neg()) as isize); op1_3 = assigned; assigned };
vm_block = 939; continue;
}
// C line 19113
943 => {
let _ = { let assigned = JS_NewBool(ctx, ret_flag); *(sp).offset(((0 as i32)) as isize) = assigned; assigned };
// C line 19114
let _ = { sp = (sp).offset((((1 as i32)) as isize)); sp };
// C line 19116
vm_block = 30; continue;
}
// C line 19097
944 => {
let _ = { let assigned = (1 as i32); ret_flag = assigned; assigned };
vm_block = 943; continue;
}
// C line 19109
947 => {
let _ = JS_FreeValue(ctx, *(sp).offset((((1 as i32)).wrapping_neg()) as isize));
// C line 19110
let _ = { let assigned = ret_4; *(sp).offset((((1 as i32)).wrapping_neg()) as isize) = assigned; assigned };
// C line 19111
let _ = { let assigned = (0 as i32); ret_flag = assigned; assigned };
vm_block = 943; continue;
}
// C line 19108
948 => {
vm_block = 29; continue;
}
// C line 19107
949 => {
vm_block = if (JS_IsException(ret_4)) != 0 { 948 } else { 947 }; continue;
}
// C line 19101
950 => {
let _ = { let assigned = JS_CallFree(ctx, method, *(sp).offset((((4 as i32)).wrapping_neg()) as isize), (0 as i32), ((core::ptr::null_mut::<c_void>()) as *mut JSValue)); ret_4 = assigned; assigned };
vm_block = 949; continue;
}
// C line 19104
951 => {
let _ = { let assigned = JS_CallFree(ctx, method, *(sp).offset((((4 as i32)).wrapping_neg()) as isize), (1 as i32), (sp).offset(-(((1 as i32)) as isize))); ret_4 = assigned; assigned };
vm_block = 949; continue;
}
// C line 19099
952 => {
vm_block = if (((flags_1) & ((2 as i32)))) != 0 { 950 } else { 951 }; continue;
}
// C line 19096
953 => {
vm_block = if (((((JS_IsUndefined(method)) != 0) || ((JS_IsNull(method)) != 0)) as i32)) != 0 { 944 } else { 952 }; continue;
}
// C line 19095
954 => {
vm_block = 29; continue;
}
// C line 19094
955 => {
vm_block = if (JS_IsException(method)) != 0 { 954 } else { 953 }; continue;
}
// C line 19090
958 => {
let _ = { let assigned = ((*({ let old = pc; pc = (pc).offset(1); old })) as i32); flags_1 = assigned; assigned };
// C line 19091
let _ = { let assigned = pc; (*(sf)).cur_pc = assigned; assigned };
// C line 19092
let _ = { let assigned = JS_GetProperty(ctx, *(sp).offset((((4 as i32)).wrapping_neg()) as isize), ((if (((flags_1) & ((1 as i32)))) != 0 { (JS_ATOM_throw as i32) } else { (crate::quickjs_atom::JS_ATOM_return as i32) }) as JSAtom)); method = assigned; assigned };
vm_block = 955; continue;
}
// C line 19079
961 => {
let _ = JS_FreeValue(ctx, *(sp).offset((((1 as i32)).wrapping_neg()) as isize));
// C line 19080
let _ = { let assigned = ret_3; *(sp).offset((((1 as i32)).wrapping_neg()) as isize) = assigned; assigned };
// C line 19082
vm_block = 30; continue;
}
// C line 19078
962 => {
vm_block = 29; continue;
}
// C line 19077
963 => {
vm_block = if (JS_IsException(ret_3)) != 0 { 962 } else { 961 }; continue;
}
// C line 19074
965 => {
let _ = { let assigned = pc; (*(sf)).cur_pc = assigned; assigned };
// C line 19075
let _ = { let assigned = JS_Call(ctx, *(sp).offset((((3 as i32)).wrapping_neg()) as isize), *(sp).offset((((4 as i32)).wrapping_neg()) as isize), (1 as i32), (sp).offset(-(((1 as i32)) as isize))); ret_3 = assigned; assigned };
vm_block = 963; continue;
}
// C line 19066
967 => {
let _ = { let assigned = ret_val_1; *(sp).offset((((1 as i32)).wrapping_neg()) as isize) = assigned; assigned };
// C line 19068
vm_block = 30; continue;
}
// C line 19062
970 => {
let _ = JS_ThrowInternalError(ctx, c"nip_catch".as_ptr());
// C line 19063
let _ = JS_FreeValue(ctx, ret_val_1);
// C line 19064
vm_block = 29; continue;
}
// C line 19061
971 => {
vm_block = if ((((!(((!(((((sp) == (stack_buf)) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 970 } else { 967 }; continue;
}
// C line 19057
972 => {
vm_block = if ((((((((sp) > (stack_buf)) as i32)) != 0) && ((((((((*(sp).offset((((1 as i32)).wrapping_neg()) as isize)).tag) as i32)) != ((JS_TAG_CATCH_OFFSET as i32))) as i32)) != 0)) as i32)) != 0 { 973 } else { 971 }; continue;
}
// C line 19059
973 => {
let _ = JS_FreeValue(ctx, *({ sp = (sp).offset(-1); sp }));
vm_block = 972; continue;
}
// C line 19056
974 => {
let _ = { let assigned = *({ sp = (sp).offset(-1); sp }); ret_val_1 = assigned; assigned };
vm_block = 972; continue;
}
// C line 19050
976 => {
let _ = { let old = sp; sp = (sp).offset(-1); old };
// C line 19051
vm_block = 30; continue;
}
// C line 19048
977 => {
let _ = JS_FreeValue(ctx, *(sp).offset((((1 as i32)).wrapping_neg()) as isize));
vm_block = 976; continue;
}
// C line 19047
978 => {
vm_block = 29; continue;
}
// C line 19046
979 => {
vm_block = if (JS_IteratorClose(ctx, *(sp).offset((((1 as i32)).wrapping_neg()) as isize), (0 as i32))) != 0 { 978 } else { 977 }; continue;
}
// C line 19045
980 => {
let _ = { let assigned = pc; (*(sf)).cur_pc = assigned; assigned };
vm_block = 979; continue;
}
// C line 19044
981 => {
vm_block = if ((!((JS_IsUndefined(*(sp).offset((((1 as i32)).wrapping_neg()) as isize))) != 0) as i32)) != 0 { 980 } else { 976 }; continue;
}
// C line 19041
984 => {
let _ = { let old = sp; sp = (sp).offset(-1); old };
// C line 19042
let _ = JS_FreeValue(ctx, *(sp).offset((((1 as i32)).wrapping_neg()) as isize));
// C line 19043
let _ = { let old = sp; sp = (sp).offset(-1); old };
vm_block = 981; continue;
}
// C line 19037
985 => {
vm_block = 30; continue;
}
// C line 19034
987 => {
let _ = JS_ThrowTypeError(ctx, c"iterator must return an object".as_ptr());
// C line 19035
vm_block = 29; continue;
}
// C line 19033
988 => {
vm_block = if ((((!(((!(((!((JS_IsObject(*(sp).offset((((1 as i32)).wrapping_neg()) as isize))) != 0) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 987 } else { 985 }; continue;
}
// C line 19030
990 => {
let _ = { sp = (sp).offset((((1 as i32)) as isize)); sp };
// C line 19031
vm_block = 30; continue;
}
// C line 19029
991 => {
vm_block = 29; continue;
}
// C line 19028
992 => {
vm_block = if (js_iterator_get_value_done(ctx, sp)) != 0 { 991 } else { 990 }; continue;
}
// C line 19027
993 => {
let _ = { let assigned = pc; (*(sf)).cur_pc = assigned; assigned };
vm_block = 992; continue;
}
// C line 19023
996 => {
let _ = { sp = (sp).offset((((1 as i32)) as isize)); sp };
// C line 19024
let _ = { let assigned = JS_NewCatchOffset(ctx, (0 as i32)); *({ let old = sp; sp = (sp).offset(1); old }) = assigned; assigned };
// C line 19025
vm_block = 30; continue;
}
// C line 19022
997 => {
vm_block = 29; continue;
}
// C line 19021
998 => {
vm_block = if (js_for_of_start(ctx, sp, (1 as i32))) != 0 { 997 } else { 996 }; continue;
}
// C line 19020
999 => {
let _ = { let assigned = pc; (*(sf)).cur_pc = assigned; assigned };
vm_block = 998; continue;
}
// C line 19017
1001 => {
let _ = { let old = sp; sp = (sp).offset(1); old };
// C line 19018
vm_block = 30; continue;
}
// C line 19016
1002 => {
vm_block = 29; continue;
}
// C line 19015
1003 => {
vm_block = if (js_for_await_of_next(ctx, sp)) != 0 { 1002 } else { 1001 }; continue;
}
// C line 19014
1004 => {
let _ = { let assigned = pc; (*(sf)).cur_pc = assigned; assigned };
vm_block = 1003; continue;
}
// C line 19010
1006 => {
let _ = { sp = (sp).offset((((2 as i32)) as isize)); sp };
// C line 19012
vm_block = 30; continue;
}
// C line 19009
1007 => {
vm_block = 29; continue;
}
// C line 19008
1008 => {
vm_block = if (js_for_of_next(ctx, sp, offset)) != 0 { 1007 } else { 1006 }; continue;
}
// C line 19005
1011 => {
offset = (((3 as i32)).wrapping_neg()).wrapping_sub(((*(pc).offset(((0 as i32)) as isize)) as i32));
// C line 19006
let _ = { pc = (pc).offset((((1 as i32)) as isize)); pc };
// C line 19007
let _ = { let assigned = pc; (*(sf)).cur_pc = assigned; assigned };
vm_block = 1008; continue;
}
// C line 19000
1014 => {
let _ = { sp = (sp).offset((((1 as i32)) as isize)); sp };
// C line 19001
let _ = { let assigned = JS_NewCatchOffset(ctx, (0 as i32)); *({ let old = sp; sp = (sp).offset(1); old }) = assigned; assigned };
// C line 19002
vm_block = 30; continue;
}
// C line 18999
1015 => {
vm_block = 29; continue;
}
// C line 18998
1016 => {
vm_block = if (js_for_of_start(ctx, sp, (0 as i32))) != 0 { 1015 } else { 1014 }; continue;
}
// C line 18997
1017 => {
let _ = { let assigned = pc; (*(sf)).cur_pc = assigned; assigned };
vm_block = 1016; continue;
}
// C line 18994
1019 => {
let _ = { sp = (sp).offset((((2 as i32)) as isize)); sp };
// C line 18995
vm_block = 30; continue;
}
// C line 18993
1020 => {
vm_block = 29; continue;
}
// C line 18992
1021 => {
vm_block = if (js_for_in_next(ctx, sp)) != 0 { 1020 } else { 1019 }; continue;
}
// C line 18991
1022 => {
let _ = { let assigned = pc; (*(sf)).cur_pc = assigned; assigned };
vm_block = 1021; continue;
}
// C line 18989
1023 => {
vm_block = 30; continue;
}
// C line 18988
1024 => {
vm_block = 29; continue;
}
// C line 18987
1025 => {
vm_block = if (js_for_in_start(ctx, sp)) != 0 { 1024 } else { 1023 }; continue;
}
// C line 18986
1026 => {
let _ = { let assigned = pc; (*(sf)).cur_pc = assigned; assigned };
vm_block = 1025; continue;
}
// C line 18980
1029 => {
let _ = { let old = sp; sp = (sp).offset(-1); old };
// C line 18981
let _ = { let assigned = ((*(b)).byte_code_buf).offset(((pos) as isize)); pc = assigned; assigned };
// C line 18983
vm_block = 30; continue;
}
// C line ? labels: ret_fail
1031 => {
let _ = JS_ThrowInternalError(ctx, c"invalid ret value".as_ptr());
// C line 18978
vm_block = 29; continue;
}
// C line 18975
1032 => {
vm_block = if ((((!(((!(((((pos) >= ((((*(b)).byte_code_len) as u32))) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 1031 } else { 1029 }; continue;
}
// C line 18974
1033 => {
let _ = { let assigned = ((((((op1_2).u).uint64) as i32)) as u32); pos = assigned; assigned };
vm_block = 1032; continue;
}
// C line 18973
1034 => {
vm_block = 1031; continue;
}
// C line 18972
1035 => {
vm_block = if ((((!(((!((((((((op1_2).tag) as i32)) != ((JS_TAG_INT as i32))) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 1034 } else { 1033 }; continue;
}
// C line 18971
1036 => {
let _ = { let assigned = *(sp).offset((((1 as i32)).wrapping_neg()) as isize); op1_2 = assigned; assigned };
vm_block = 1035; continue;
}
// C line 18960
1041 => {
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
1046 => {
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
// C line 18905
1047 => {
vm_block = 30; continue;
}
// C line 18903
1048 => {
vm_block = 29; continue;
}
// C line 18902
1049 => {
vm_block = if ((((!(((!((js_poll_interrupts_at(ctx, pc)) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 1048 } else { 1047 }; continue;
}
// C line 18900
1050 => {
let _ = { pc = (pc).offset((((((crate::cutils_header::get_u32((pc).offset(-(((4 as i32)) as isize)))) as i32)).wrapping_sub((4 as i32))) as isize)); pc };
vm_block = 1049; continue;
}
// C line 18899
1051 => {
vm_block = if ((!((res_1) != 0) as i32)) != 0 { 1050 } else { 1049 }; continue;
}
// C line 18898
1052 => {
let _ = { let old = sp; sp = (sp).offset(-1); old };
vm_block = 1051; continue;
}
// C line 18894
1053 => {
let _ = { let assigned = ((((op1_1).u).uint64) as i32); res_1 = assigned; assigned };
vm_block = 1052; continue;
}
// C line 18896
1054 => {
let _ = { let assigned = JS_ToBoolFree(ctx, op1_1); res_1 = assigned; assigned };
vm_block = 1052; continue;
}
// C line 18893
1055 => {
vm_block = if (((((((((op1_1).tag) as i32)) as u32)) <= ((((JS_TAG_UNDEFINED as i32)) as u32))) as i32)) != 0 { 1053 } else { 1054 }; continue;
}
// C line 18890
1057 => {
let _ = { let assigned = *(sp).offset((((1 as i32)).wrapping_neg()) as isize); op1_1 = assigned; assigned };
// C line 18891
let _ = { pc = (pc).offset((((4 as i32)) as isize)); pc };
vm_block = 1055; continue;
}
// C line 18884
1058 => {
vm_block = 30; continue;
}
// C line 18882
1059 => {
vm_block = 29; continue;
}
// C line 18881
1060 => {
vm_block = if ((((!(((!((js_poll_interrupts_at(ctx, pc)) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 1059 } else { 1058 }; continue;
}
// C line 18879
1061 => {
let _ = { pc = (pc).offset((((((crate::cutils_header::get_u32((pc).offset(-(((4 as i32)) as isize)))) as i32)).wrapping_sub((4 as i32))) as isize)); pc };
vm_block = 1060; continue;
}
// C line 18878
1062 => {
vm_block = if (res) != 0 { 1061 } else { 1060 }; continue;
}
// C line 18877
1063 => {
let _ = { let old = sp; sp = (sp).offset(-1); old };
vm_block = 1062; continue;
}
// C line 18873
1064 => {
let _ = { let assigned = ((((op1).u).uint64) as i32); res = assigned; assigned };
vm_block = 1063; continue;
}
// C line 18875
1065 => {
let _ = { let assigned = JS_ToBoolFree(ctx, op1); res = assigned; assigned };
vm_block = 1063; continue;
}
// C line 18872
1066 => {
vm_block = if (((((((((op1).tag) as i32)) as u32)) <= ((((JS_TAG_UNDEFINED as i32)) as u32))) as i32)) != 0 { 1064 } else { 1065 }; continue;
}
// C line 18870
1068 => {
let _ = { let assigned = *(sp).offset((((1 as i32)).wrapping_neg()) as isize); op1 = assigned; assigned };
// C line 18871
let _ = { pc = (pc).offset((((4 as i32)) as isize)); pc };
vm_block = 1066; continue;
}
// C line 18852
1069 => {
vm_block = 30; continue;
}
// C line 18851
1070 => {
vm_block = 29; continue;
}
// C line 18850
1071 => {
vm_block = if ((((!(((!((js_poll_interrupts_at(ctx, pc)) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 1070 } else { 1069 }; continue;
}
// C line 18849
1072 => {
let _ = { pc = (pc).offset(((((crate::cutils_header::get_u32(pc)) as i32)) as isize)); pc };
vm_block = 1071; continue;
}
// C line 18844
1074 => {
let _ = { sp = (sp).offset((((2 as i32)) as isize)); sp };
// C line 18846
vm_block = 30; continue;
}
// C line 18843
1075 => {
vm_block = 29; continue;
}
// C line 18842
1076 => {
vm_block = if (JS_GetGlobalVarRef(ctx, atom_2, sp)) != 0 { 1075 } else { 1074 }; continue;
}
// C line 18838
1079 => {
let _ = { let assigned = crate::cutils_header::get_u32(pc); atom_2 = assigned; assigned };
// C line 18839
let _ = { pc = (pc).offset((((4 as i32)) as isize)); pc };
// C line 18840
let _ = { let assigned = pc; (*(sf)).cur_pc = assigned; assigned };
vm_block = 1076; continue;
}
// C line 18831
1082 => {
let _ = { let assigned = var_ref_1; ((*(pr)).u).var_ref = assigned; assigned };
// C line 18832
let _ = { let assigned = JS_AtomToValue(ctx, atom_1); *({ let old = sp; sp = (sp).offset(1); old }) = assigned; assigned };
// C line 18834
vm_block = 30; continue;
}
// C line 18828
1084 => {
let _ = free_var_ref(rt, var_ref_1);
// C line 18829
vm_block = 29; continue;
}
// C line 18827
1085 => {
vm_block = if ((!(!(pr).is_null()) as i32)) != 0 { 1084 } else { 1082 }; continue;
}
// C line 18825
1086 => {
let _ = { let assigned = add_property(ctx, ((((*(sp).offset((((1 as i32)).wrapping_neg()) as isize)).u).ptr) as *mut JSObject), atom_1, ((((1 as i32)).wrapping_shl(((1 as i32)) as u32)) | (((2 as i32)).wrapping_shl(((4 as i32)) as u32)))); pr = assigned; assigned };
vm_block = 1085; continue;
}
// C line 18818
1088 => {
let _ = { let assigned = *(var_refs).offset((idx_21) as isize); var_ref_1 = assigned; assigned };
// C line 18819
let _ = { let old = (*(js_rc(((var_ref_1) as *mut c_void)))).ref_count; (*(js_rc(((var_ref_1) as *mut c_void)))).ref_count = ((*(js_rc(((var_ref_1) as *mut c_void)))).ref_count).wrapping_add(1); old };
vm_block = 1086; continue;
}
// C line 18823
1089 => {
vm_block = 29; continue;
}
// C line 18822
1090 => {
vm_block = if ((!(!(var_ref_1).is_null()) as i32)) != 0 { 1089 } else { 1086 }; continue;
}
// C line 18821
1091 => {
let _ = { let assigned = get_var_ref(ctx, sf, idx_21, (((opcode) == ((OP_make_arg_ref as i32))) as i32)); var_ref_1 = assigned; assigned };
vm_block = 1090; continue;
}
// C line 18817
1092 => {
vm_block = if ((((opcode) == ((OP_make_var_ref_ref as i32))) as i32)) != 0 { 1088 } else { 1091 }; continue;
}
// C line 18816
1093 => {
vm_block = 29; continue;
}
// C line 18815
1094 => {
vm_block = if ((((!(((!((JS_IsException(*(sp).offset((((1 as i32)).wrapping_neg()) as isize))) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 1093 } else { 1092 }; continue;
}
// C line 18811
1098 => {
let _ = { let assigned = crate::cutils_header::get_u32(pc); atom_1 = assigned; assigned };
// C line 18812
let _ = { let assigned = ((crate::cutils_header::get_u16((pc).offset((((4 as i32)) as isize)))) as i32); idx_21 = assigned; assigned };
// C line 18813
let _ = { pc = (pc).offset((((6 as i32)) as isize)); pc };
// C line 18814
let _ = { let assigned = JS_NewObjectProto(ctx, JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_NULL as i32)) as i64) }); *({ let old = sp; sp = (sp).offset(1); old }) = assigned; assigned };
vm_block = 1094; continue;
}
// C line 18797
1102 => {
let _ = { let assigned = ((crate::cutils_header::get_u16(pc)) as i32); idx_20 = assigned; assigned };
// C line 18798
let _ = { pc = (pc).offset((((2 as i32)) as isize)); pc };
// C line 18799
let _ = close_lexical_var(ctx, b, sf, idx_20);
// C line 18801
vm_block = 30; continue;
}
// C line 18790
1105 => {
let _ = set_value(ctx, core::ptr::addr_of_mut!(*(var_buf).offset((idx_19) as isize)), *(sp).offset((((1 as i32)).wrapping_neg()) as isize));
// C line 18791
let _ = { let old = sp; sp = (sp).offset(-1); old };
// C line 18793
vm_block = 30; continue;
}
// C line 18787
1107 => {
let _ = JS_ThrowReferenceError(ctx, c"'this' can be initialized only once".as_ptr());
// C line 18788
vm_block = 29; continue;
}
// C line 18786
1108 => {
vm_block = if ((((!(((!(((!((JS_IsUninitialized(*(var_buf).offset((idx_19) as isize))) != 0) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 1107 } else { 1105 }; continue;
}
// C line 18784
1110 => {
let _ = { let assigned = ((crate::cutils_header::get_u16(pc)) as i32); idx_19 = assigned; assigned };
// C line 18785
let _ = { pc = (pc).offset((((2 as i32)) as isize)); pc };
vm_block = 1108; continue;
}
// C line 18778
1112 => {
let _ = set_value(ctx, core::ptr::addr_of_mut!(*(var_buf).offset((idx_18) as isize)), JS_DupValue(ctx, *(sp).offset((((1 as i32)).wrapping_neg()) as isize)));
// C line 18780
vm_block = 30; continue;
}
// C line 18775
1114 => {
let _ = JS_ThrowReferenceErrorUninitialized2(ctx, b, idx_18, (0 as i32));
// C line 18776
vm_block = 29; continue;
}
// C line 18774
1115 => {
vm_block = if ((((!(((!((JS_IsUninitialized(*(var_buf).offset((idx_18) as isize))) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 1114 } else { 1112 }; continue;
}
// C line 18772
1117 => {
let _ = { let assigned = ((crate::cutils_header::get_u16(pc)) as i32); idx_18 = assigned; assigned };
// C line 18773
let _ = { pc = (pc).offset((((2 as i32)) as isize)); pc };
vm_block = 1115; continue;
}
// C line 18765
1120 => {
let _ = set_value(ctx, core::ptr::addr_of_mut!(*(var_buf).offset((idx_17) as isize)), *(sp).offset((((1 as i32)).wrapping_neg()) as isize));
// C line 18766
let _ = { let old = sp; sp = (sp).offset(-1); old };
// C line 18768
vm_block = 30; continue;
}
// C line 18762
1122 => {
let _ = JS_ThrowReferenceErrorUninitialized2(ctx, b, idx_17, (0 as i32));
// C line 18763
vm_block = 29; continue;
}
// C line 18761
1123 => {
vm_block = if ((((!(((!((JS_IsUninitialized(*(var_buf).offset((idx_17) as isize))) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 1122 } else { 1120 }; continue;
}
// C line 18759
1125 => {
let _ = { let assigned = ((crate::cutils_header::get_u16(pc)) as i32); idx_17 = assigned; assigned };
// C line 18760
let _ = { pc = (pc).offset((((2 as i32)) as isize)); pc };
vm_block = 1123; continue;
}
// C line 18752
1128 => {
let _ = { let assigned = JS_DupValue(ctx, *(var_buf).offset((idx_16) as isize)); *(sp).offset(((0 as i32)) as isize) = assigned; assigned };
// C line 18753
let _ = { let old = sp; sp = (sp).offset(1); old };
// C line 18755
vm_block = 30; continue;
}
// C line 18749
1130 => {
let _ = JS_ThrowReferenceErrorUninitialized2(caller_ctx, b, idx_16, (0 as i32));
// C line 18750
vm_block = 29; continue;
}
// C line 18748
1131 => {
vm_block = if ((((!(((!((JS_IsUninitialized(*(var_buf).offset((idx_16) as isize))) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 1130 } else { 1128 }; continue;
}
// C line 18746
1133 => {
let _ = { let assigned = ((crate::cutils_header::get_u16(pc)) as i32); idx_16 = assigned; assigned };
// C line 18747
let _ = { pc = (pc).offset((((2 as i32)) as isize)); pc };
vm_block = 1131; continue;
}
// C line 18739
1136 => {
let _ = { let assigned = JS_DupValue(ctx, *(var_buf).offset((idx_15) as isize)); *(sp).offset(((0 as i32)) as isize) = assigned; assigned };
// C line 18740
let _ = { let old = sp; sp = (sp).offset(1); old };
// C line 18742
vm_block = 30; continue;
}
// C line 18736
1138 => {
let _ = JS_ThrowReferenceErrorUninitialized2(ctx, b, idx_15, (0 as i32));
// C line 18737
vm_block = 29; continue;
}
// C line 18735
1139 => {
vm_block = if ((((!(((!((JS_IsUninitialized(*(var_buf).offset((idx_15) as isize))) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 1138 } else { 1136 }; continue;
}
// C line 18733
1141 => {
let _ = { let assigned = ((crate::cutils_header::get_u16(pc)) as i32); idx_15 = assigned; assigned };
// C line 18734
let _ = { pc = (pc).offset((((2 as i32)) as isize)); pc };
vm_block = 1139; continue;
}
// C line 18725
1145 => {
let _ = { let assigned = ((crate::cutils_header::get_u16(pc)) as i32); idx_14 = assigned; assigned };
// C line 18726
let _ = { pc = (pc).offset((((2 as i32)) as isize)); pc };
// C line 18727
let _ = set_value(ctx, core::ptr::addr_of_mut!(*(var_buf).offset((idx_14) as isize)), JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNINITIALIZED as i32)) as i64) });
// C line 18729
vm_block = 30; continue;
}
// C line 18718
1148 => {
let _ = set_value(ctx, (*(*(var_refs).offset((idx_13) as isize))).pvalue, *(sp).offset((((1 as i32)).wrapping_neg()) as isize));
// C line 18719
let _ = { let old = sp; sp = (sp).offset(-1); old };
// C line 18721
vm_block = 30; continue;
}
// C line 18715
1150 => {
let _ = JS_ThrowReferenceErrorUninitialized2(ctx, b, idx_13, (1 as i32));
// C line 18716
vm_block = 29; continue;
}
// C line 18714
1151 => {
vm_block = if ((((!(((!(((!((JS_IsUninitialized(*((*(*(var_refs).offset((idx_13) as isize))).pvalue))) != 0) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 1150 } else { 1148 }; continue;
}
// C line 18712
1153 => {
let _ = { let assigned = ((crate::cutils_header::get_u16(pc)) as i32); idx_13 = assigned; assigned };
// C line 18713
let _ = { pc = (pc).offset((((2 as i32)) as isize)); pc };
vm_block = 1151; continue;
}
// C line 18705
1156 => {
let _ = set_value(ctx, (*(*(var_refs).offset((idx_12) as isize))).pvalue, *(sp).offset((((1 as i32)).wrapping_neg()) as isize));
// C line 18706
let _ = { let old = sp; sp = (sp).offset(-1); old };
// C line 18708
vm_block = 30; continue;
}
// C line 18702
1158 => {
let _ = JS_ThrowReferenceErrorUninitialized2(ctx, b, idx_12, (1 as i32));
// C line 18703
vm_block = 29; continue;
}
// C line 18701
1159 => {
vm_block = if ((((!(((!((JS_IsUninitialized(*((*(*(var_refs).offset((idx_12) as isize))).pvalue))) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 1158 } else { 1156 }; continue;
}
// C line 18699
1161 => {
let _ = { let assigned = ((crate::cutils_header::get_u16(pc)) as i32); idx_12 = assigned; assigned };
// C line 18700
let _ = { pc = (pc).offset((((2 as i32)) as isize)); pc };
vm_block = 1159; continue;
}
// C line 18692
1164 => {
let _ = { let assigned = JS_DupValue(ctx, val_4); *(sp).offset(((0 as i32)) as isize) = assigned; assigned };
// C line 18693
let _ = { let old = sp; sp = (sp).offset(1); old };
// C line 18695
vm_block = 30; continue;
}
// C line 18689
1166 => {
let _ = JS_ThrowReferenceErrorUninitialized2(ctx, b, idx_11, (1 as i32));
// C line 18690
vm_block = 29; continue;
}
// C line 18688
1167 => {
vm_block = if ((((!(((!((JS_IsUninitialized(val_4)) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 1166 } else { 1164 }; continue;
}
// C line 18685
1170 => {
let _ = { let assigned = ((crate::cutils_header::get_u16(pc)) as i32); idx_11 = assigned; assigned };
// C line 18686
let _ = { pc = (pc).offset((((2 as i32)) as isize)); pc };
// C line 18687
let _ = { let assigned = *((*(*(var_refs).offset((idx_11) as isize))).pvalue); val_4 = assigned; assigned };
vm_block = 1167; continue;
}
// C line 18676
1174 => {
let _ = { let assigned = ((crate::cutils_header::get_u16(pc)) as i32); idx_10 = assigned; assigned };
// C line 18677
let _ = { pc = (pc).offset((((2 as i32)) as isize)); pc };
// C line 18678
let _ = set_value(ctx, (*(*(var_refs).offset((idx_10) as isize))).pvalue, JS_DupValue(ctx, *(sp).offset((((1 as i32)).wrapping_neg()) as isize)));
// C line 18680
vm_block = 30; continue;
}
// C line 18667
1179 => {
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
1185 => {
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
// C line 18604
1189 => {
let _ = { let assigned = ((crate::cutils_header::get_u16(pc)) as i32); idx_7 = assigned; assigned };
// C line 18605
let _ = { pc = (pc).offset((((2 as i32)) as isize)); pc };
// C line 18606
let _ = set_value(ctx, core::ptr::addr_of_mut!(*(arg_buf).offset((idx_7) as isize)), JS_DupValue(ctx, *(sp).offset((((1 as i32)).wrapping_neg()) as isize)));
// C line 18608
vm_block = 30; continue;
}
// C line 18595
1194 => {
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
1199 => {
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
1203 => {
let _ = { let assigned = ((crate::cutils_header::get_u16(pc)) as i32); idx_4 = assigned; assigned };
// C line 18579
let _ = { pc = (pc).offset((((2 as i32)) as isize)); pc };
// C line 18580
let _ = set_value(ctx, core::ptr::addr_of_mut!(*(var_buf).offset((idx_4) as isize)), JS_DupValue(ctx, *(sp).offset((((1 as i32)).wrapping_neg()) as isize)));
// C line 18582
vm_block = 30; continue;
}
// C line 18569
1208 => {
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
1213 => {
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
1214 => {
vm_block = 30; continue;
}
// C line 18534
1215 => {
vm_block = 29; continue;
}
// C line 18531
1216 => {
let _ = JS_ThrowReferenceErrorUninitialized(ctx, (*(cv_1)).var_name);
vm_block = 1215; continue;
}
// C line 18533
1217 => {
let _ = JS_ThrowTypeErrorReadOnly(ctx, ((1 as i32)).wrapping_shl(((14 as i32)) as u32), (*(cv_1)).var_name);
vm_block = 1215; continue;
}
// C line 18530
1218 => {
vm_block = if (JS_IsUninitialized(*((*(var_ref)).pvalue))) != 0 { 1216 } else { 1217 }; continue;
}
// C line 18529
1219 => {
vm_block = 1235; continue;
}
// C line 18528
1220 => {
vm_block = if ((((opcode) == ((OP_put_var_init as i32))) as i32)) != 0 { 1219 } else { 1218 }; continue;
}
// C line 18548
1221 => {
vm_block = 29; continue;
}
// C line 18547
1222 => {
vm_block = if ((((ret_2) < ((0 as i32))) as i32)) != 0 { 1221 } else { 1214 }; continue;
}
// C line 18544
1224 => {
let _ = { let assigned = JS_SetPropertyInternal(ctx, (*(ctx)).global_obj, (*(cv_1)).var_name, *(sp).offset((((1 as i32)).wrapping_neg()) as isize), (*(ctx)).global_obj, ((1 as i32)).wrapping_shl(((15 as i32)) as u32)); ret_2 = assigned; assigned };
// C line 18546
let _ = { let old = sp; sp = (sp).offset(-1); old };
vm_block = 1222; continue;
}
// C line 18541
1226 => {
let _ = JS_ThrowReferenceErrorNotDefined(ctx, (*(cv_1)).var_name);
// C line 18542
vm_block = 29; continue;
}
// C line 18540
1227 => {
vm_block = if ((((((((ret_2) == ((0 as i32))) as i32)) != 0) && ((is_strict_mode(ctx)) != 0)) as i32)) != 0 { 1226 } else { 1224 }; continue;
}
// C line 18539
1228 => {
vm_block = 29; continue;
}
// C line 18538
1229 => {
vm_block = if ((((ret_2) < ((0 as i32))) as i32)) != 0 { 1228 } else { 1227 }; continue;
}
// C line 18536
1231 => {
let _ = { let assigned = pc; (*(sf)).cur_pc = assigned; assigned };
// C line 18537
let _ = { let assigned = JS_HasProperty(ctx, (*(ctx)).global_obj, (*(cv_1)).var_name); ret_2 = assigned; assigned };
vm_block = 1229; continue;
}
// C line 18527
1232 => {
vm_block = if ((*(var_ref)).is_lexical) != 0 { 1220 } else { 1231 }; continue;
}
// C line 18526
1233 => {
cv_1 = core::ptr::addr_of_mut!(*((*(b)).closure_var).offset((idx_1) as isize));
vm_block = 1232; continue;
}
// C line ? labels: put_var_ok
1235 => {
let _ = set_value(ctx, (*(var_ref)).pvalue, *(sp).offset((((1 as i32)).wrapping_neg()) as isize));
// C line 18553
let _ = { let old = sp; sp = (sp).offset(-1); old };
vm_block = 1214; continue;
}
// C line 18524
1236 => {
vm_block = if ((((!(((!((((((JS_IsUninitialized(*((*(var_ref)).pvalue))) != 0) || (((((*(var_ref)).is_const) as i32)) != 0)) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 1233 } else { 1235 }; continue;
}
// C line 18521
1239 => {
let _ = { let assigned = ((crate::cutils_header::get_u16(pc)) as i32); idx_1 = assigned; assigned };
// C line 18522
let _ = { pc = (pc).offset((((2 as i32)) as isize)); pc };
// C line 18523
let _ = { let assigned = *(var_refs).offset((idx_1) as isize); var_ref = assigned; assigned };
vm_block = 1236; continue;
}
// C line 18512
1241 => {
let _ = { let old = sp; sp = (sp).offset(1); old };
// C line 18514
vm_block = 30; continue;
}
// C line 18498
1243 => {
let _ = JS_ThrowReferenceErrorUninitialized(ctx, (*(cv)).var_name);
// C line 18499
vm_block = 29; continue;
}
// C line 18507
1244 => {
vm_block = 29; continue;
}
// C line 18506
1245 => {
vm_block = if (JS_IsException(*(sp).offset(((0 as i32)) as isize))) != 0 { 1244 } else { 1241 }; continue;
}
// C line 18501
1247 => {
let _ = { let assigned = pc; (*(sf)).cur_pc = assigned; assigned };
// C line 18502
let _ = { let assigned = JS_GetPropertyInternal(ctx, (*(ctx)).global_obj, (*(cv)).var_name, (*(ctx)).global_obj, (opcode).wrapping_sub((OP_get_var_undef as i32))); *(sp).offset(((0 as i32)) as isize) = assigned; assigned };
vm_block = 1245; continue;
}
// C line 18497
1248 => {
vm_block = if ((*(cv)).is_lexical()) != 0 { 1243 } else { 1247 }; continue;
}
// C line 18496
1249 => {
cv = core::ptr::addr_of_mut!(*((*(b)).closure_var).offset((idx) as isize));
vm_block = 1248; continue;
}
// C line 18510
1250 => {
let _ = { let assigned = JS_DupValue(ctx, val_2); *(sp).offset(((0 as i32)) as isize) = assigned; assigned };
vm_block = 1241; continue;
}
// C line 18495
1251 => {
vm_block = if ((((!(((!((JS_IsUninitialized(val_2)) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 1249 } else { 1250 }; continue;
}
// C line 18492
1254 => {
let _ = { let assigned = ((crate::cutils_header::get_u16(pc)) as i32); idx = assigned; assigned };
// C line 18493
let _ = { pc = (pc).offset((((2 as i32)) as isize)); pc };
// C line 18494
let _ = { let assigned = *((*(*(var_refs).offset((idx) as isize))).pvalue); val_2 = assigned; assigned };
vm_block = 1251; continue;
}
// C line 18480
1259 => {
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
1260 => {
vm_block = 29; continue;
}
// C line 18478
1261 => {
vm_block = if (JS_IsException(val_1)) != 0 { 1260 } else { 1259 }; continue;
}
// C line 18476
1263 => {
let _ = { let assigned = pc; (*(sf)).cur_pc = assigned; assigned };
// C line 18477
let _ = { let assigned = js_dynamic_import(ctx, *(sp).offset((((2 as i32)).wrapping_neg()) as isize), *(sp).offset((((1 as i32)).wrapping_neg()) as isize)); val_1 = assigned; assigned };
vm_block = 1261; continue;
}
// C line 18468
1266 => {
let _ = JS_FreeValue(ctx, *(sp).offset((((1 as i32)).wrapping_neg()) as isize));
// C line 18469
let _ = { let assigned = proto; *(sp).offset((((1 as i32)).wrapping_neg()) as isize) = assigned; assigned };
// C line 18471
vm_block = 30; continue;
}
// C line 18467
1267 => {
vm_block = 29; continue;
}
// C line 18466
1268 => {
vm_block = if (JS_IsException(proto)) != 0 { 1267 } else { 1266 }; continue;
}
// C line 18464
1270 => {
let _ = { let assigned = pc; (*(sf)).cur_pc = assigned; assigned };
// C line 18465
let _ = { let assigned = JS_GetPrototype(ctx, *(sp).offset((((1 as i32)).wrapping_neg()) as isize)); proto = assigned; assigned };
vm_block = 1268; continue;
}
// C line 18459
1271 => {
vm_block = 30; continue;
}
// C line 18457
1272 => {
vm_block = 29; continue;
}
// C line 18456
1273 => {
vm_block = if (JS_IsException(*(sp).offset((((1 as i32)).wrapping_neg()) as isize))) != 0 { 1272 } else { 1271 }; continue;
}
// C line 18454
1275 => {
let _ = { let assigned = JS_NewRegexp(ctx, *(sp).offset((((2 as i32)).wrapping_neg()) as isize), *(sp).offset((((1 as i32)).wrapping_neg()) as isize)); *(sp).offset((((2 as i32)).wrapping_neg()) as isize) = assigned; assigned };
// C line 18455
let _ = { let old = sp; sp = (sp).offset(-1); old };
vm_block = 1273; continue;
}
// C line 18445
1280 => {
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
1281 => {
vm_block = 29; continue;
}
// C line 18443
1282 => {
vm_block = if ((((!(((!((JS_IsException(ret_val)) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 1281 } else { 1280 }; continue;
}
// C line 18442
1283 => {
let _ = free_arg_list(ctx, tab, len);
vm_block = 1282; continue;
}
// C line 18436
1284 => {
let _ = { let assigned = JS_EvalObject(ctx, JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }, obj_1, ((2 as i32)).wrapping_shl(((0 as i32)) as u32), scope_idx_1); ret_val = assigned; assigned };
vm_block = 1283; continue;
}
// C line 18433
1285 => {
let _ = { let assigned = *(tab).offset(((0 as i32)) as isize); obj_1 = assigned; assigned };
vm_block = 1284; continue;
}
// C line 18435
1286 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; obj_1 = assigned; assigned };
vm_block = 1284; continue;
}
// C line 18432
1287 => {
vm_block = if ((((len) >= ((((1 as i32)) as u32))) as i32)) != 0 { 1285 } else { 1286 }; continue;
}
// C line 18439
1288 => {
let _ = { let assigned = JS_Call(ctx, *(sp).offset((((2 as i32)).wrapping_neg()) as isize), JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }, ((len) as i32), tab); ret_val = assigned; assigned };
vm_block = 1283; continue;
}
// C line 18431
1289 => {
vm_block = if (js_same_value(ctx, *(sp).offset((((2 as i32)).wrapping_neg()) as isize), (*(ctx)).eval_obj)) != 0 { 1287 } else { 1288 }; continue;
}
// C line 18430
1290 => {
vm_block = 29; continue;
}
// C line 18429
1291 => {
vm_block = if ((!(!(tab).is_null()) as i32)) != 0 { 1290 } else { 1289 }; continue;
}
// C line 18425
1295 => {
let _ = { let assigned = (((crate::cutils_header::get_u16(pc)).wrapping_add(((((2 as i32)).wrapping_neg()) as u32))) as i32); scope_idx_1 = assigned; assigned };
// C line 18426
let _ = { pc = (pc).offset((((2 as i32)) as isize)); pc };
// C line 18427
let _ = { let assigned = pc; (*(sf)).cur_pc = assigned; assigned };
// C line 18428
let _ = { let assigned = build_arg_list(ctx, core::ptr::addr_of_mut!(len), *(sp).offset((((1 as i32)).wrapping_neg()) as isize)); tab = assigned; assigned };
vm_block = 1291; continue;
}
// C line 18413
1298 => {
let _ = { sp = (sp).offset(-(((call_argc).wrapping_add((1 as i32))) as isize)); sp };
// C line 18414
let _ = { let assigned = ret_val; *({ let old = sp; sp = (sp).offset(1); old }) = assigned; assigned };
// C line 18416
vm_block = 30; continue;
}
// C line 18411
1299 => {
vm_block = if ((((i) < (call_argc)) as i32)) != 0 { 1301 } else { 1298 }; continue;
}
// C line 18412
1301 => {
let _ = JS_FreeValue(ctx, *(call_argv).offset((i) as isize));
// C line ?
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 1299; continue;
}
// C line 18411
1302 => {
let _ = { let assigned = ((1 as i32)).wrapping_neg(); i = assigned; assigned };
vm_block = 1299; continue;
}
// C line 18410
1303 => {
vm_block = 29; continue;
}
// C line 18409
1304 => {
vm_block = if ((((!(((!((JS_IsException(ret_val)) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 1303 } else { 1302 }; continue;
}
// C line 18403
1305 => {
let _ = { let assigned = JS_EvalObject(ctx, JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }, obj, ((2 as i32)).wrapping_shl(((0 as i32)) as u32), scope_idx); ret_val = assigned; assigned };
vm_block = 1304; continue;
}
// C line 18400
1306 => {
let _ = { let assigned = *(call_argv).offset(((0 as i32)) as isize); obj = assigned; assigned };
vm_block = 1305; continue;
}
// C line 18402
1307 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; obj = assigned; assigned };
vm_block = 1305; continue;
}
// C line 18399
1308 => {
vm_block = if ((((call_argc) >= ((1 as i32))) as i32)) != 0 { 1306 } else { 1307 }; continue;
}
// C line 18406
1309 => {
let _ = { let assigned = JS_CallInternal(ctx, *(call_argv).offset((((1 as i32)).wrapping_neg()) as isize), JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }, JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }, call_argc, call_argv, (0 as i32)); ret_val = assigned; assigned };
vm_block = 1304; continue;
}
// C line 18398
1310 => {
vm_block = if (js_same_value(ctx, *(call_argv).offset((((1 as i32)).wrapping_neg()) as isize), (*(ctx)).eval_obj)) != 0 { 1308 } else { 1309 }; continue;
}
// C line 18393
1315 => {
let _ = { let assigned = ((crate::cutils_header::get_u16(pc)) as i32); call_argc = assigned; assigned };
// C line 18394
let _ = { let assigned = (((crate::cutils_header::get_u16((pc).offset((((2 as i32)) as isize)))).wrapping_add(((((2 as i32)).wrapping_neg()) as u32))) as i32); scope_idx = assigned; assigned };
// C line 18395
let _ = { pc = (pc).offset((((4 as i32)) as isize)); pc };
// C line 18396
let _ = { let assigned = (sp).offset(-((call_argc) as isize)); call_argv = assigned; assigned };
// C line 18397
let _ = { let assigned = pc; (*(sf)).cur_pc = assigned; assigned };
vm_block = 1310; continue;
}
// C line 18387
1316 => {
vm_block = 29; continue;
}
// C line 18371
1317 => {
let _ = JS_ThrowTypeErrorReadOnly(ctx, ((1 as i32)).wrapping_shl(((14 as i32)) as u32), atom);
vm_block = 1316; continue;
}
// C line 18374
1318 => {
let _ = JS_ThrowSyntaxErrorVarRedeclaration(ctx, atom);
vm_block = 1316; continue;
}
// C line 18377
1319 => {
let _ = JS_ThrowReferenceErrorUninitialized(ctx, atom);
vm_block = 1316; continue;
}
// C line 18380
1320 => {
let _ = JS_ThrowReferenceError(ctx, c"unsupported reference to 'super'".as_ptr());
vm_block = 1316; continue;
}
// C line 18383
1321 => {
let _ = JS_ThrowTypeError(ctx, c"iterator does not have a throw method".as_ptr());
vm_block = 1316; continue;
}
// C line 18385
1322 => {
let _ = JS_ThrowInternalError(ctx, format_args!("invalid throw var type {}", v_type));
vm_block = 1316; continue;
}
// C line 18382
1323 => {
vm_block = if ((((v_type) == ((4 as i32))) as i32)) != 0 { 1321 } else { 1322 }; continue;
}
// C line 18379
1324 => {
vm_block = if ((((v_type) == ((3 as i32))) as i32)) != 0 { 1320 } else { 1323 }; continue;
}
// C line 18376
1325 => {
vm_block = if ((((v_type) == ((2 as i32))) as i32)) != 0 { 1319 } else { 1324 }; continue;
}
// C line 18373
1326 => {
vm_block = if ((((v_type) == ((1 as i32))) as i32)) != 0 { 1318 } else { 1325 }; continue;
}
// C line 18370
1327 => {
vm_block = if ((((v_type) == ((0 as i32))) as i32)) != 0 { 1317 } else { 1326 }; continue;
}
// C line 18367
1330 => {
let _ = { let assigned = crate::cutils_header::get_u32(pc); atom = assigned; assigned };
// C line 18368
let _ = { let assigned = ((*(pc).offset(((4 as i32)) as isize)) as i32); v_type = assigned; assigned };
// C line 18369
let _ = { pc = (pc).offset((((5 as i32)) as isize)); pc };
vm_block = 1327; continue;
}
// C line 18355
1332 => {
sp = sp.offset(-1);
let _ = js_host_vm_throw(ctx, b, sf, pc, sp, *sp);
// C line 18356
vm_block = 29; continue;
}
// C line 18349
1336 => {
let _ = JS_FreeValue(ctx, *(sp).offset((((2 as i32)).wrapping_neg()) as isize));
// C line 18350
let _ = JS_FreeValue(ctx, *(sp).offset((((1 as i32)).wrapping_neg()) as isize));
// C line 18351
let _ = { sp = (sp).offset(-(((2 as i32)) as isize)); sp };
// C line 18352
vm_block = 30; continue;
}
// C line 18348
1337 => {
vm_block = 29; continue;
}
// C line 18347
1338 => {
vm_block = if ((((JS_AddBrand(ctx, *(sp).offset((((2 as i32)).wrapping_neg()) as isize), *(sp).offset((((1 as i32)).wrapping_neg()) as isize))) < ((0 as i32))) as i32)) != 0 { 1337 } else { 1336 }; continue;
}
// C line 18345
1339 => {
vm_block = 30; continue;
}
// C line 18341
1341 => {
let _ = JS_ThrowTypeError(ctx, c"invalid brand on object".as_ptr());
// C line 18342
vm_block = 29; continue;
}
// C line 18340
1342 => {
vm_block = if ((!((ret_1) != 0) as i32)) != 0 { 1341 } else { 1339 }; continue;
}
// C line 18339
1343 => {
vm_block = 29; continue;
}
// C line 18338
1344 => {
vm_block = if ((((ret_1) < ((0 as i32))) as i32)) != 0 { 1343 } else { 1342 }; continue;
}
// C line 18337
1345 => {
ret_1 = JS_CheckBrand(ctx, *(sp).offset((((2 as i32)).wrapping_neg()) as isize), *(sp).offset((((1 as i32)).wrapping_neg()) as isize));
vm_block = 1344; continue;
}
// C line 18332
1347 => {
let _ = { let assigned = ret; *({ let old = sp; sp = (sp).offset(1); old }) = assigned; assigned };
// C line 18334
vm_block = 30; continue;
}
// C line 18331
1348 => {
vm_block = 29; continue;
}
// C line 18330
1349 => {
vm_block = if (JS_IsException(ret)) != 0 { 1348 } else { 1347 }; continue;
}
// C line 18328
1351 => {
let _ = { let assigned = JS_CallConstructor2(ctx, v_super, new_target, argc, argv); ret = assigned; assigned };
// C line 18329
let _ = JS_FreeValue(ctx, v_super);
vm_block = 1349; continue;
}
// C line 18327
1352 => {
vm_block = 29; continue;
}
// C line 18326
1353 => {
vm_block = if (JS_IsException(v_super)) != 0 { 1352 } else { 1351 }; continue;
}
// C line 18325
1354 => {
let _ = { let assigned = JS_GetPrototype(ctx, func_obj); v_super = assigned; assigned };
vm_block = 1353; continue;
}
// C line 18324
1355 => {
vm_block = 1360; continue;
}
// C line 18323
1356 => {
vm_block = if (JS_IsUndefined(new_target)) != 0 { 1355 } else { 1354 }; continue;
}
// C line 18322
1357 => {
let _ = { let assigned = pc; (*(sf)).cur_pc = assigned; assigned };
vm_block = 1356; continue;
}
// C line 18318
1358 => {
vm_block = 30; continue;
}
// C line ? labels: non_ctor_call
1360 => {
let _ = JS_ThrowTypeError(ctx, c"class constructors must be invoked with 'new'".as_ptr());
// C line 18316
vm_block = 29; continue;
}
// C line 18313
1361 => {
vm_block = if (JS_IsUndefined(new_target)) != 0 { 1360 } else { 1358 }; continue;
}
// C line 18310
1363 => {
let _ = { let old = sp; sp = (sp).offset(1); old };
// C line 18311
vm_block = 30; continue;
}
// C line 18306
1364 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((1 as i32)) as u32)) as u64) }, tag: (((JS_TAG_BOOL as i32)) as i64) }; *(sp).offset(((0 as i32)) as isize) = assigned; assigned };
vm_block = 1363; continue;
}
// C line 18303
1366 => {
let _ = JS_ThrowTypeError(caller_ctx, c"derived class constructor must return an object or undefined".as_ptr());
// C line 18304
vm_block = 29; continue;
}
// C line 18302
1367 => {
vm_block = if ((!((JS_IsUndefined(*(sp).offset((((1 as i32)).wrapping_neg()) as isize))) != 0) as i32)) != 0 { 1366 } else { 1364 }; continue;
}
// C line 18308
1368 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_BOOL as i32)) as i64) }; *(sp).offset(((0 as i32)) as isize) = assigned; assigned };
vm_block = 1363; continue;
}
// C line 18301
1369 => {
vm_block = if ((!((JS_IsObject(*(sp).offset((((1 as i32)).wrapping_neg()) as isize))) != 0) as i32)) != 0 { 1367 } else { 1368 }; continue;
}
// C line 18296
1371 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; ret_val = assigned; assigned };
// C line 18297
vm_block = 10; continue;
}
// C line 18293
1373 => {
let _ = { let assigned = *({ sp = (sp).offset(-1); sp }); ret_val = assigned; assigned };
// C line 18294
vm_block = 10; continue;
}
// C line 18285
1379 => {
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
1380 => {
vm_block = 29; continue;
}
// C line 18283
1381 => {
vm_block = if ((((!(((!((JS_IsException(ret_val)) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 1380 } else { 1379 }; continue;
}
// C line 18278
1385 => {
let _ = { let assigned = ((crate::cutils_header::get_u16(pc)) as i32); magic = assigned; assigned };
// C line 18279
let _ = { pc = (pc).offset((((2 as i32)) as isize)); pc };
// C line 18280
let _ = { let assigned = pc; (*(sf)).cur_pc = assigned; assigned };
// C line 18282
let _ = { let assigned = js_function_apply(ctx, *(sp).offset((((3 as i32)).wrapping_neg()) as isize), (2 as i32), core::ptr::addr_of_mut!(*(sp).offset((((2 as i32)).wrapping_neg()) as isize)), magic); ret_val = assigned; assigned };
vm_block = 1381; continue;
}
// C line 18272
1387 => {
let _ = { let assigned = ret_val; *({ let old = sp; sp = (sp).offset(1); old }) = assigned; assigned };
// C line 18273
vm_block = 30; continue;
}
// C line 18271
1388 => {
vm_block = 29; continue;
}
// C line 18270
1389 => {
vm_block = if ((((!(((!((JS_IsException(ret_val)) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 1388 } else { 1387 }; continue;
}
// C line 18266
1393 => {
let _ = { let assigned = ((crate::cutils_header::get_u16(pc)) as i32); call_argc = assigned; assigned };
// C line 18267
let _ = { pc = (pc).offset((((2 as i32)) as isize)); pc };
// C line 18268
let _ = { let assigned = js_create_array_free(ctx, call_argc, (sp).offset(-((call_argc) as isize))); ret_val = assigned; assigned };
// C line 18269
let _ = { sp = (sp).offset(-((call_argc) as isize)); sp };
vm_block = 1389; continue;
}
// C line 18261
1396 => {
let _ = { sp = (sp).offset(-(((call_argc).wrapping_add((2 as i32))) as isize)); sp };
// C line 18262
let _ = { let assigned = ret_val; *({ let old = sp; sp = (sp).offset(1); old }) = assigned; assigned };
// C line 18264
vm_block = 30; continue;
}
// C line 18259
1397 => {
vm_block = if ((((i) < (call_argc)) as i32)) != 0 { 1399 } else { 1396 }; continue;
}
// C line 18260
1399 => {
let _ = JS_FreeValue(ctx, *(call_argv).offset((i) as isize));
// C line ?
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 1397; continue;
}
// C line 18259
1400 => {
let _ = { let assigned = ((2 as i32)).wrapping_neg(); i = assigned; assigned };
vm_block = 1397; continue;
}
// C line 18258
1401 => {
vm_block = 10; continue;
}
// C line 18257
1402 => {
vm_block = if ((((opcode) == ((OP_tail_call_method as i32))) as i32)) != 0 { 1401 } else { 1400 }; continue;
}
// C line 18256
1403 => {
vm_block = 29; continue;
}
// C line 18255
1404 => {
vm_block = if ((((!(((!((JS_IsException(ret_val)) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 1403 } else { 1402 }; continue;
}
// C line 18249
1409 => {
let _ = { let assigned = ((crate::cutils_header::get_u16(pc)) as i32); call_argc = assigned; assigned };
// C line 18250
let _ = { pc = (pc).offset((((2 as i32)) as isize)); pc };
// C line 18251
let _ = { let assigned = (sp).offset(-((call_argc) as isize)); call_argv = assigned; assigned };
// C line 18252
let _ = { let assigned = pc; (*(sf)).cur_pc = assigned; assigned };
// C line 18253
let _ = { let assigned = JS_CallInternal(ctx, *(call_argv).offset((((1 as i32)).wrapping_neg()) as isize), *(call_argv).offset((((2 as i32)).wrapping_neg()) as isize), JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }, call_argc, call_argv, (0 as i32)); ret_val = assigned; assigned };
vm_block = 1404; continue;
}
// C line 18242
1412 => {
let _ = { sp = (sp).offset(-(((call_argc).wrapping_add((2 as i32))) as isize)); sp };
// C line 18243
let _ = { let assigned = ret_val; *({ let old = sp; sp = (sp).offset(1); old }) = assigned; assigned };
// C line 18245
vm_block = 30; continue;
}
// C line 18240
1413 => {
vm_block = if ((((i) < (call_argc)) as i32)) != 0 { 1415 } else { 1412 }; continue;
}
// C line 18241
1415 => {
let _ = JS_FreeValue(ctx, *(call_argv).offset((i) as isize));
// C line ?
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 1413; continue;
}
// C line 18240
1416 => {
let _ = { let assigned = ((2 as i32)).wrapping_neg(); i = assigned; assigned };
vm_block = 1413; continue;
}
// C line 18239
1417 => {
vm_block = 29; continue;
}
// C line 18238
1418 => {
vm_block = if ((((!(((!((JS_IsException(ret_val)) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 1417 } else { 1416 }; continue;
}
// C line 18231
1423 => {
let _ = { let assigned = ((crate::cutils_header::get_u16(pc)) as i32); call_argc = assigned; assigned };
// C line 18232
let _ = { pc = (pc).offset((((2 as i32)) as isize)); pc };
// C line 18233
let _ = { let assigned = (sp).offset(-((call_argc) as isize)); call_argv = assigned; assigned };
// C line 18234
let _ = { let assigned = pc; (*(sf)).cur_pc = assigned; assigned };
// C line 18235
let _ = { let assigned = JS_CallConstructorInternal(ctx, *(call_argv).offset((((2 as i32)).wrapping_neg()) as isize), *(call_argv).offset((((1 as i32)).wrapping_neg()) as isize), call_argc, call_argv, (0 as i32)); ret_val = assigned; assigned };
vm_block = 1418; continue;
}
// C line 18225
1426 => {
let _ = { sp = (sp).offset(-(((call_argc).wrapping_add((1 as i32))) as isize)); sp };
// C line 18226
let _ = { let assigned = ret_val; *({ let old = sp; sp = (sp).offset(1); old }) = assigned; assigned };
// C line 18228
vm_block = 30; continue;
}
// C line 18223
1427 => {
vm_block = if ((((i) < (call_argc)) as i32)) != 0 { 1429 } else { 1426 }; continue;
}
// C line 18224
1429 => {
let _ = JS_FreeValue(ctx, *(call_argv).offset((i) as isize));
// C line ?
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 1427; continue;
}
// C line 18223
1430 => {
let _ = { let assigned = ((1 as i32)).wrapping_neg(); i = assigned; assigned };
vm_block = 1427; continue;
}
// C line 18222
1431 => {
vm_block = 10; continue;
}
// C line 18221
1432 => {
vm_block = if ((((opcode) == ((OP_tail_call as i32))) as i32)) != 0 { 1431 } else { 1430 }; continue;
}
// C line 18220
1433 => {
vm_block = 29; continue;
}
// C line 18219
1434 => {
vm_block = if ((((!(((!((JS_IsException(ret_val)) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 1433 } else { 1432 }; continue;
}
// C line ? labels: has_call_argc
1437 => {
let _ = { let assigned = (sp).offset(-((call_argc) as isize)); call_argv = assigned; assigned };
// C line 18216
let _ = { let assigned = pc; (*(sf)).cur_pc = assigned; assigned };
// C line 18217
let _ = { let assigned = JS_CallInternal(ctx, *(call_argv).offset((((1 as i32)).wrapping_neg()) as isize), JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }, JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }, call_argc, call_argv, (0 as i32)); ret_val = assigned; assigned };
vm_block = 1434; continue;
}
// C line 18211
1440 => {
let _ = { let assigned = ((crate::cutils_header::get_u16(pc)) as i32); call_argc = assigned; assigned };
// C line 18212
let _ = { pc = (pc).offset((((2 as i32)) as isize)); pc };
// C line 18213
vm_block = 1437; continue;
}
// C line 18199
1441 => {
vm_block = 30; continue;
}
// C line 18197
1442 => {
vm_block = 29; continue;
}
// C line 18196
1443 => {
vm_block = if ((((!(((!((JS_IsException(*(sp).offset((((1 as i32)).wrapping_neg()) as isize))) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 1442 } else { 1441 }; continue;
}
// C line 18193
1446 => {
bfunc = JS_DupValue(ctx, *((*(b)).cpool).offset((crate::cutils_header::get_u32(pc)) as isize));
// C line 18194
let _ = { pc = (pc).offset((((4 as i32)) as isize)); pc };
// C line 18195
let _ = { let assigned = js_closure(ctx, bfunc, var_refs, sf, (0 as i32)); *({ let old = sp; sp = (sp).offset(1); old }) = assigned; assigned };
vm_block = 1443; continue;
}
// C line 18182
1453 => {
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
1457 => {
let _ = { let assigned = *(sp).offset((((2 as i32)).wrapping_neg()) as isize); tmp_7 = assigned; assigned };
// C line 18175
let _ = { let assigned = *(sp).offset((((1 as i32)).wrapping_neg()) as isize); *(sp).offset((((2 as i32)).wrapping_neg()) as isize) = assigned; assigned };
// C line 18176
let _ = { let assigned = tmp_7; *(sp).offset((((1 as i32)).wrapping_neg()) as isize) = assigned; assigned };
// C line 18178
vm_block = 30; continue;
}
// C line 18164
1463 => {
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
1468 => {
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
1473 => {
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
1480 => {
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
1486 => {
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
1491 => {
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
1495 => {
let _ = { let assigned = *(sp).offset((((2 as i32)).wrapping_neg()) as isize); tmp = assigned; assigned };
// C line 18109
let _ = { let assigned = *(sp).offset((((3 as i32)).wrapping_neg()) as isize); *(sp).offset((((2 as i32)).wrapping_neg()) as isize) = assigned; assigned };
// C line 18110
let _ = { let assigned = tmp; *(sp).offset((((3 as i32)).wrapping_neg()) as isize) = assigned; assigned };
// C line 18112
vm_block = 30; continue;
}
// C line 18098
1502 => {
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
1508 => {
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
1513 => {
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
1517 => {
let _ = { let assigned = *(sp).offset((((1 as i32)).wrapping_neg()) as isize); *(sp).offset(((0 as i32)) as isize) = assigned; assigned };
// C line 18081
let _ = { let assigned = JS_DupValue(ctx, *(sp).offset((((2 as i32)).wrapping_neg()) as isize)); *(sp).offset((((1 as i32)).wrapping_neg()) as isize) = assigned; assigned };
// C line 18082
let _ = { let old = sp; sp = (sp).offset(1); old };
// C line 18083
vm_block = 30; continue;
}
// C line 18074
1522 => {
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
1526 => {
let _ = { let assigned = JS_DupValue(ctx, *(sp).offset((((2 as i32)).wrapping_neg()) as isize)); *(sp).offset(((0 as i32)) as isize) = assigned; assigned };
// C line 18070
let _ = { let assigned = JS_DupValue(ctx, *(sp).offset((((1 as i32)).wrapping_neg()) as isize)); *(sp).offset(((1 as i32)) as isize) = assigned; assigned };
// C line 18071
let _ = { sp = (sp).offset((((2 as i32)) as isize)); sp };
// C line 18072
vm_block = 30; continue;
}
// C line 18065
1529 => {
let _ = { let assigned = JS_DupValue(ctx, *(sp).offset((((1 as i32)).wrapping_neg()) as isize)); *(sp).offset(((0 as i32)) as isize) = assigned; assigned };
// C line 18066
let _ = { let old = sp; sp = (sp).offset(1); old };
// C line 18067
vm_block = 30; continue;
}
// C line 18059
1534 => {
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
1538 => {
let _ = JS_FreeValue(ctx, *(sp).offset((((2 as i32)).wrapping_neg()) as isize));
// C line 18055
let _ = { let assigned = *(sp).offset((((1 as i32)).wrapping_neg()) as isize); *(sp).offset((((2 as i32)).wrapping_neg()) as isize) = assigned; assigned };
// C line 18056
let _ = { let old = sp; sp = (sp).offset(-1); old };
// C line 18057
vm_block = 30; continue;
}
// C line 18050
1541 => {
let _ = JS_FreeValue(ctx, *(sp).offset((((1 as i32)).wrapping_neg()) as isize));
// C line 18051
let _ = { let old = sp; sp = (sp).offset(-1); old };
// C line 18052
vm_block = 30; continue;
}
// C line 18047
1542 => {
vm_block = 30; continue;
}
// C line 18045
1543 => {
vm_block = 29; continue;
}
// C line 18044
1544 => {
vm_block = if ((((!(((!((JS_IsException(*(sp).offset((((1 as i32)).wrapping_neg()) as isize))) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 1543 } else { 1542 }; continue;
}
// C line 18040
1548 => {
first = ((crate::cutils_header::get_u16(pc)) as i32);
// C line 18041
let _ = { pc = (pc).offset((((2 as i32)) as isize)); pc };
// C line 18042
let _ = { let assigned = min_int(first, argc); first = assigned; assigned };
// C line 18043
let _ = { let assigned = js_create_array(ctx, (argc).wrapping_sub(first), (argv).offset(((first) as isize))); *({ let old = sp; sp = (sp).offset(1); old }) = assigned; assigned };
vm_block = 1544; continue;
}
// C line 18037
1549 => {
vm_block = 30; continue;
}
// C line ?
1550 => {
let _ = std::process::abort();
vm_block = 1549; continue;
}
// C line 18032
1551 => {
vm_block = 1549; continue;
}
// C line 18031
1552 => {
vm_block = 29; continue;
}
// C line 18030
1553 => {
vm_block = if ((((!(((!((JS_IsException(*(sp).offset((((1 as i32)).wrapping_neg()) as isize))) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 1552 } else { 1551 }; continue;
}
// C line 18029
1554 => {
let _ = { let assigned = js_import_meta(ctx); *({ let old = sp; sp = (sp).offset(1); old }) = assigned; assigned };
vm_block = 1553; continue;
}
// C line 18027
1555 => {
vm_block = 1549; continue;
}
// C line 18026
1556 => {
vm_block = 29; continue;
}
// C line 18025
1557 => {
vm_block = if ((((!(((!((JS_IsException(*(sp).offset((((1 as i32)).wrapping_neg()) as isize))) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 1556 } else { 1555 }; continue;
}
// C line 18024
1558 => {
let _ = { let assigned = JS_NewObjectProto(ctx, JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_NULL as i32)) as i64) }); *({ let old = sp; sp = (sp).offset(1); old }) = assigned; assigned };
vm_block = 1557; continue;
}
// C line 18022
1559 => {
vm_block = 1549; continue;
}
// C line 18018
1560 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; *({ let old = sp; sp = (sp).offset(1); old }) = assigned; assigned };
vm_block = 1559; continue;
}
// C line 18020
1561 => {
let _ = { let assigned = JS_DupValue(ctx, JSValue { u: JSValueUnion { ptr: ((p1) as *mut c_void) }, tag: (((JS_TAG_OBJECT as i32)) as i64) }); *({ let old = sp; sp = (sp).offset(1); old }) = assigned; assigned };
vm_block = 1559; continue;
}
// C line 18017
1562 => {
vm_block = if ((((!(((!(((!(!(p1).is_null()) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 1560 } else { 1561 }; continue;
}
// C line 18016
1563 => {
let _ = { let assigned = (((*(p)).u).func).home_object; p1 = assigned; assigned };
vm_block = 1562; continue;
}
// C line 18011
1565 => {
let _ = { let assigned = JS_DupValue(ctx, new_target); *({ let old = sp; sp = (sp).offset(1); old }) = assigned; assigned };
// C line 18012
vm_block = 1549; continue;
}
// C line 18008
1567 => {
let _ = { let assigned = JS_DupValue(ctx, (*(sf)).cur_func); *({ let old = sp; sp = (sp).offset(1); old }) = assigned; assigned };
// C line 18009
vm_block = 1549; continue;
}
// C line 18006
1568 => {
vm_block = 1549; continue;
}
// C line 18005
1569 => {
vm_block = 29; continue;
}
// C line 18004
1570 => {
vm_block = if ((((!(((!((JS_IsException(*(sp).offset((((1 as i32)).wrapping_neg()) as isize))) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 1569 } else { 1568 }; continue;
}
// C line 18002
1571 => {
let _ = { let assigned = js_build_mapped_arguments(ctx, argc, argv, sf, min_int(argc, (((*(b)).arg_count) as i32))); *({ let old = sp; sp = (sp).offset(1); old }) = assigned; assigned };
vm_block = 1570; continue;
}
// C line 18000
1572 => {
vm_block = 1549; continue;
}
// C line 17999
1573 => {
vm_block = 29; continue;
}
// C line 17998
1574 => {
vm_block = if ((((!(((!((JS_IsException(*(sp).offset((((1 as i32)).wrapping_neg()) as isize))) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 1573 } else { 1572 }; continue;
}
// C line 17997
1575 => {
let _ = { let assigned = js_build_arguments(ctx, argc, argv); *({ let old = sp; sp = (sp).offset(1); old }) = assigned; assigned };
vm_block = 1574; continue;
}
// C line 17995
1576 => {
vm_block = match arg { x if x == (OP_SPECIAL_OBJECT_IMPORT_META as i32) => 1554, x if x == (OP_SPECIAL_OBJECT_VAR_OBJECT as i32) => 1558, x if x == (OP_SPECIAL_OBJECT_HOME_OBJECT as i32) => 1563, x if x == (OP_SPECIAL_OBJECT_NEW_TARGET as i32) => 1565, x if x == (OP_SPECIAL_OBJECT_THIS_FUNC as i32) => 1567, x if x == (OP_SPECIAL_OBJECT_MAPPED_ARGUMENTS as i32) => 1571, x if x == (OP_SPECIAL_OBJECT_ARGUMENTS as i32) => 1575, _ => 1550, }; continue;
}
// C line 17994
1577 => {
arg = ((*({ let old = pc; pc = (pc).offset(1); old })) as i32);
vm_block = 1576; continue;
}
// C line 17991
1578 => {
vm_block = 30; continue;
}
// C line 17990
1579 => {
vm_block = 29; continue;
}
// C line 17989
1580 => {
vm_block = if ((((!(((!((JS_IsException(*(sp).offset((((1 as i32)).wrapping_neg()) as isize))) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 1579 } else { 1578 }; continue;
}
// C line 17988
1581 => {
let _ = { let assigned = JS_NewObject(ctx); *({ let old = sp; sp = (sp).offset(1); old }) = assigned; assigned };
vm_block = 1580; continue;
}
// C line 17985
1583 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((1 as i32)) as u32)) as u64) }, tag: (((JS_TAG_BOOL as i32)) as i64) }; *({ let old = sp; sp = (sp).offset(1); old }) = assigned; assigned };
// C line 17986
vm_block = 30; continue;
}
// C line 17982
1585 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_BOOL as i32)) as i64) }; *({ let old = sp; sp = (sp).offset(1); old }) = assigned; assigned };
// C line 17983
vm_block = 30; continue;
}
// C line 17978
1587 => {
let _ = { let assigned = val; *({ let old = sp; sp = (sp).offset(1); old }) = assigned; assigned };
// C line 17980
vm_block = 30; continue;
}
// C line 17968
1588 => {
let _ = { let assigned = JS_DupValue(ctx, (*(ctx)).global_obj); val = assigned; assigned };
vm_block = 1587; continue;
}
// C line 17972
1589 => {
vm_block = 29; continue;
}
// C line 17971
1590 => {
vm_block = if (JS_IsException(val)) != 0 { 1589 } else { 1587 }; continue;
}
// C line 17970
1591 => {
let _ = { let assigned = JS_ToObject(ctx, this_obj); val = assigned; assigned };
vm_block = 1590; continue;
}
// C line 17967
1592 => {
vm_block = if ((((((((tag) == ((((JS_TAG_NULL as i32)) as u32))) as i32)) != 0) || (((((tag) == ((((JS_TAG_UNDEFINED as i32)) as u32))) as i32)) != 0)) as i32)) != 0 { 1588 } else { 1591 }; continue;
}
// C line 17966
1593 => {
vm_block = 1596; continue;
}
// C line 17965
1594 => {
vm_block = if ((((!(((!(((((tag) == ((((JS_TAG_OBJECT as i32)) as u32))) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 1593 } else { 1592 }; continue;
}
// C line 17964
1595 => {
tag = (((((this_obj).tag) as i32)) as u32);
vm_block = 1594; continue;
}
// C line ? labels: normal_this
1596 => {
let _ = { let assigned = JS_DupValue(ctx, this_obj); val = assigned; assigned };
vm_block = 1587; continue;
}
// C line 17963
1597 => {
vm_block = if ((!(((((((*(b)).js_mode) as i32)) & (((1 as i32)).wrapping_shl(((0 as i32)) as u32)))) != 0) as i32)) != 0 { 1595 } else { 1596 }; continue;
}
// C line 17957
1599 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_NULL as i32)) as i64) }; *({ let old = sp; sp = (sp).offset(1); old }) = assigned; assigned };
// C line 17958
vm_block = 30; continue;
}
// C line 17954
1601 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; *({ let old = sp; sp = (sp).offset(1); old }) = assigned; assigned };
// C line 17955
vm_block = 30; continue;
}
// C line 17950
1604 => {
let _ = { let assigned = JS_AtomToValue(ctx, crate::cutils_header::get_u32(pc)); *({ let old = sp; sp = (sp).offset(1); old }) = assigned; assigned };
// C line 17951
let _ = { pc = (pc).offset((((4 as i32)) as isize)); pc };
// C line 17952
vm_block = 30; continue;
}
// C line 17914
1607 => {
let _ = { let assigned = JS_DupValue(ctx, *((*(b)).cpool).offset((crate::cutils_header::get_u32(pc)) as isize)); *({ let old = sp; sp = (sp).offset(1); old }) = assigned; assigned };
// C line 17915
let _ = { pc = (pc).offset((((4 as i32)) as isize)); pc };
// C line 17916
vm_block = 30; continue;
}
// C line 17910
1610 => {
let _ = { let assigned = __JS_NewShortBigInt(ctx, ((((crate::cutils_header::get_u32(pc)) as i32)) as i64)); *({ let old = sp; sp = (sp).offset(1); old }) = assigned; assigned };
// C line 17911
let _ = { pc = (pc).offset((((4 as i32)) as isize)); pc };
// C line 17912
vm_block = 30; continue;
}
// C line 17906
1613 => {
let _ = { let assigned = JS_NewInt32(ctx, ((crate::cutils_header::get_u32(pc)) as i32)); *({ let old = sp; sp = (sp).offset(1); old }) = assigned; assigned };
// C line 17907
let _ = { pc = (pc).offset((((4 as i32)) as isize)); pc };
// C line 17908
vm_block = 30; continue;
}
// C line 17904
1614 => {
// The official *pc++ bytecode switch: retain constant patterns at opt0.
// Equality guards otherwise compare each opcode sequentially.
vm_block = match ({ let assigned = ((*({ let old = pc; pc = (pc).offset(1); old })) as i32); opcode = assigned; assigned } as u16) { OP_invalid => 32, OP_is_undefined_or_null => 40, OP_nop => 41, OP_initial_yield => 43, OP_return_async => 45, OP_async_yield_star => 47, OP_yield_star => 47, OP_yield => 49, OP_await => 51, OP_with_get_ref => 116, OP_with_make_ref => 116, OP_with_delete_var => 116, OP_with_put_var => 116, OP_with_get_var => 116, OP_to_propkey => 126, OP_to_object => 134, OP_delete_var => 142, OP_delete => 147, OP_typeof => 152, OP_instanceof => 157, OP_private_in => 162, OP_in => 167, OP_strict_neq => 177, OP_strict_eq => 187, OP_neq => 197, OP_eq => 207, OP_gte => 217, OP_gt => 227, OP_lte => 237, OP_lt => 247, OP_xor => 257, OP_or => 267, OP_and => 277, OP_sar => 289, OP_shr => 301, OP_shl => 314, OP_not => 321, OP_dec_loc => 335, OP_inc_loc => 349, OP_post_dec => 360, OP_post_inc => 371, OP_dec => 381, OP_inc => 391, OP_neg => 409, OP_plus => 418, OP_pow => 423, OP_mod => 435, OP_div => 444, OP_mul => 474, OP_sub => 497, OP_add_loc => 527, OP_add => 558, OP_copy_data_properties => 563, OP_append => 568, OP_define_array_el => 573, OP_put_super_value => 589, OP_put_ref_value => 616, OP_put_array_el => 646, OP_get_super_value => 660, OP_get_ref_value => 685, OP_get_array_el3 => 711, OP_get_array_el2 => 734, OP_get_array_el => 757, OP_define_class_computed => 763, OP_define_class => 763, OP_define_method_computed => 799, OP_define_method => 799, OP_set_home_object => 801, OP_set_proto => 809, OP_set_name_computed => 813, OP_set_name => 819, OP_define_field => 826, OP_define_private_field => 832, OP_put_private_field => 839, OP_get_private_field => 847, OP_private_symbol => 854, OP_put_field => 874, OP_get_field2 => 904, OP_get_field => 934, OP_lnot => 940, OP_iterator_call => 958, OP_iterator_next => 965, OP_nip_catch => 974, OP_iterator_close => 984, OP_iterator_check_object => 988, OP_iterator_get_value_done => 993, OP_for_await_of_start => 999, OP_for_await_of_next => 1004, OP_for_of_next => 1011, OP_for_of_start => 1017, OP_for_in_next => 1022, OP_for_in_start => 1026, OP_ret => 1036, OP_gosub => 1041, OP_catch => 1046, OP_if_false => 1057, OP_if_true => 1068, OP_goto => 1072, OP_make_var_ref => 1079, OP_make_var_ref_ref => 1098, OP_make_arg_ref => 1098, OP_make_loc_ref => 1098, OP_close_loc => 1102, OP_put_loc_check_init => 1110, OP_set_loc_check => 1117, OP_put_loc_check => 1125, OP_get_loc_checkthis => 1133, OP_get_loc_check => 1141, OP_set_loc_uninitialized => 1145, OP_put_var_ref_check_init => 1153, OP_put_var_ref_check => 1161, OP_get_var_ref_check => 1170, OP_set_var_ref => 1174, OP_put_var_ref => 1179, OP_get_var_ref => 1185, OP_set_arg => 1189, OP_put_arg => 1194, OP_get_arg => 1199, OP_set_loc => 1203, OP_put_loc => 1208, OP_get_loc => 1213, OP_put_var_init => 1239, OP_put_var => 1239, OP_get_var => 1254, OP_get_var_undef => 1254, OP_import => 1263, OP_get_super => 1270, OP_regexp => 1275, OP_apply_eval => 1295, OP_eval => 1315, OP_throw_error => 1330, OP_throw => 1332, OP_add_brand => 1338, OP_check_brand => 1345, OP_init_ctor => 1357, OP_check_ctor => 1361, OP_check_ctor_return => 1369, OP_return_undef => 1371, OP_return => 1373, OP_apply => 1385, OP_array_from => 1393, OP_tail_call_method => 1409, OP_call_method => 1409, OP_call_constructor => 1423, OP_tail_call => 1440, OP_call => 1440, OP_fclosure => 1446, OP_swap2 => 1453, OP_swap => 1457, OP_perm5 => 1463, OP_perm4 => 1468, OP_rot3r => 1473, OP_rot5l => 1480, OP_rot4l => 1486, OP_rot3l => 1491, OP_perm3 => 1495, OP_insert4 => 1502, OP_insert3 => 1508, OP_insert2 => 1513, OP_dup1 => 1517, OP_dup3 => 1522, OP_dup2 => 1526, OP_dup => 1529, OP_nip1 => 1534, OP_nip => 1538, OP_drop => 1541, OP_rest => 1548, OP_special_object => 1577, OP_object => 1581, OP_push_true => 1583, OP_push_false => 1585, OP_push_this => 1597, OP_null => 1599, OP_undefined => 1601, OP_push_atom_value => 1604, OP_push_const => 1607, OP_push_bigint_i32 => 1610, OP_push_i32 => 1613, _ => 32, }; continue;
}
// C line 17893
1619 => {
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
1620 => {
vm_block = if ((((i) < ((((*(b)).var_ref_count) as i32))) as i32)) != 0 { 1622 } else { 1619 }; continue;
}
// C line 17892
1622 => {
let _ = { let assigned = ((core::ptr::null_mut::<c_void>()) as *mut JSVarRef); *((*(sf)).var_refs).offset((i) as isize) = assigned; assigned };
// C line ?
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 1620; continue;
}
// C line 17889
1625 => {
let _ = { let assigned = (var_buf).offset((((((*(b)).var_count) as i32)) as isize)); stack_buf = assigned; assigned };
// C line 17890
let _ = { let assigned = (((stack_buf).offset((((((*(b)).stack_size) as i32)) as isize))) as *mut *mut JSVarRef); (*(sf)).var_refs = assigned; assigned };
// C line 17891
let _ = { let assigned = (0 as i32); i = assigned; assigned };
vm_block = 1620; continue;
}
// C line 17886
1626 => {
vm_block = if ((((i) < ((((*(b)).var_count) as i32))) as i32)) != 0 { 1628 } else { 1625 }; continue;
}
// C line 17887
1628 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; *(var_buf).offset((i) as isize) = assigned; assigned };
// C line ?
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 1626; continue;
}
// C line 17882
1632 => {
let _ = { let assigned = (local_buf).offset(((arg_allocated_size) as isize)); var_buf = assigned; assigned };
// C line 17883
let _ = { let assigned = var_buf; (*(sf)).var_buf = assigned; assigned };
// C line 17884
let _ = { let assigned = arg_buf; (*(sf)).arg_buf = assigned; assigned };
// C line 17886
let _ = { let assigned = (0 as i32); i = assigned; assigned };
vm_block = 1626; continue;
}
// C line 17880
1633 => {
let _ = { let assigned = (((*(b)).arg_count) as i32); (*(sf)).arg_count = assigned; assigned };
vm_block = 1632; continue;
}
// C line 17878
1634 => {
vm_block = if ((((i) < ((((*(b)).arg_count) as i32))) as i32)) != 0 { 1636 } else { 1633 }; continue;
}
// C line 17879
1636 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; *(arg_buf).offset((i) as isize) = assigned; assigned };
// C line ?
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 1634; continue;
}
// C line 17876
1637 => {
vm_block = if ((((i) < (n)) as i32)) != 0 { 1639 } else { 1634 }; continue;
}
// C line 17877
1639 => {
let _ = { let assigned = JS_DupValue(caller_ctx, *(argv).offset((i) as isize)); *(arg_buf).offset((i) as isize) = assigned; assigned };
// C line ?
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 1637; continue;
}
// C line 17874
1642 => {
n = min_int(argc, (((*(b)).arg_count) as i32));
// C line 17875
let _ = { let assigned = local_buf; arg_buf = assigned; assigned };
// C line 17876
let _ = { let assigned = (0 as i32); i = assigned; assigned };
vm_block = 1637; continue;
}
// C line 17873
1643 => {
vm_block = if ((((!(((!((arg_allocated_size) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 1642 } else { 1632 }; continue;
}
// C line 17866
1649 => {
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
vm_block = 1643; continue;
}
// C line 17864
1650 => {
return JS_ThrowStackOverflow(caller_ctx);
}
// C line 17863
1651 => {
vm_block = if (js_check_stack_overflow(rt, alloca_size)) != 0 { 1650 } else { 1649 }; continue;
}
// C line 17860
1652 => {
let _ = { let assigned = (((size_of::<JSValue>() as usize)).wrapping_mul(((((arg_allocated_size).wrapping_add((((*(b)).var_count) as i32))).wrapping_add((((*(b)).stack_size) as i32))) as usize))).wrapping_add(((size_of::<*mut JSVarRef>() as usize)).wrapping_mul((((*(b)).var_ref_count) as usize))); alloca_size = assigned; assigned };
vm_block = 1651; continue;
}
// C line 17855
1653 => {
let _ = { let assigned = (((*(b)).arg_count) as i32); arg_allocated_size = assigned; assigned };
vm_block = 1652; continue;
}
// C line 17857
1654 => {
let _ = { let assigned = (0 as i32); arg_allocated_size = assigned; assigned };
vm_block = 1652; continue;
}
// C line 17854
1655 => {
vm_block = if ((((!(((!(((((((((argc) < ((((*(b)).arg_count) as i32))) as i32)) != 0) || ((((flags) & (((1 as i32)).wrapping_shl(((1 as i32)) as u32)))) != 0)) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 1653 } else { 1654 }; continue;
}
// C line 17852
1656 => {
let _ = { let assigned = (((*(p)).u).func).function_bytecode; b = assigned; assigned };
vm_block = 1655; continue;
}
// C line 17849
1657 => {
return (call_func).expect("checked class call")(caller_ctx, func_obj, this_obj, argc, argv, flags);
}
// C line ? labels: not_a_function
1658 => {
return JS_ThrowTypeError(caller_ctx, c"not a function".as_ptr());
}
// C line 17845
1659 => {
vm_block = if ((!((call_func).is_some()) as i32)) != 0 { 1658 } else { 1657 }; continue;
}
// C line 17844
1660 => {
let _ = { let assigned = (*((*(rt)).class_array).offset(((*(p)).class_id) as isize)).call; call_func = assigned; assigned };
vm_block = 1659; continue;
}
// C line 17842
1661 => {
vm_block = if ((((!(((!((((((((*(p)).class_id) as i32)) != ((JS_CLASS_BYTECODE_FUNCTION as i32))) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 1660 } else { 1656 }; continue;
}
// C line 17841
1662 => {
let _ = { let assigned = ((((func_obj).u).ptr) as *mut JSObject); p = assigned; assigned };
vm_block = 1661; continue;
}
// C line 17834
1663 => {
vm_block = 29; continue;
}
// C line 17836
1664 => {
vm_block = 30; continue;
}
// C line 17833
1665 => {
vm_block = if ((*(s)).throw_flag) != 0 { 1663 } else { 1664 }; continue;
}
// C line 17817
1679 => {
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
vm_block = 1665; continue;
}
// C line 17838
1680 => {
vm_block = 1658; continue;
}
// C line 17816
1681 => {
vm_block = if (((flags) & (((1 as i32)).wrapping_shl(((2 as i32)) as u32)))) != 0 { 1679 } else { 1680 }; continue;
}
// C line 17815
1682 => {
vm_block = if ((((!(((!((((((((func_obj).tag) as i32)) != ((JS_TAG_OBJECT as i32))) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 { 1681 } else { 1662 }; continue;
}
// C line 17814
1683 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 17813
1684 => {
vm_block = if (js_poll_interrupts(caller_ctx)) != 0 { 1683 } else { 1682 }; continue;
}
// C line 17776
1686 => {
rt = (*(caller_ctx)).rt;
// C line 17780
sf = core::ptr::addr_of_mut!(sf_s);
vm_block = 1684; continue;
}
_ => std::process::abort(),
} }
}
