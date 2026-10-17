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
let mut var_refs: *mut *mut JSVarRef = core::ptr::null_mut();
let mut call_argc: i32 = 0;
let mut call_argv: *mut JSValue = core::ptr::null_mut();
let mut len: u32 = 0;
let mut atom_3: JSAtom = 0;
let mut pr_1: *mut JSProperty = core::ptr::null_mut();
let mut atom_4: JSAtom = 0;
let mut pr_2: *mut JSProperty = core::ptr::null_mut();
let mut pr_3: *mut JSProperty = core::ptr::null_mut();
let mut getter: JSValue = core::mem::zeroed();
let mut setter: JSValue = core::mem::zeroed();
let mut value: JSValue = core::mem::zeroed();
let mut obj_5: JSValue = core::mem::zeroed();
let mut atom_9: JSAtom = 0;
let mut flags_2: i32 = 0;
let mut is_computed: i32 = 0;
let mut val_9: JSValue = core::mem::zeroed();
let mut obj_6: JSValue = core::mem::zeroed();
let mut prop: JSValue = core::mem::zeroed();
let mut val_10: JSValue = core::mem::zeroed();
let mut obj_7: JSValue = core::mem::zeroed();
let mut prop_1: JSValue = core::mem::zeroed();
let mut val_11: JSValue = core::mem::zeroed();
let mut atom_13: JSAtom = 0;
let mut ops: [JSValue; 2] = core::mem::zeroed();
let mut op2_3: JSValue = core::mem::zeroed();
let mut d: f64 = 0.0;
let mut d1_2: f64 = 0.0;
let mut op1_15: JSValue = core::mem::zeroed();
let mut op1_16: JSValue = core::mem::zeroed();
let mut atom_17: JSAtom = 0;
let mut diff_2: i32 = 0;
let mut obj_8: JSValue = core::mem::zeroed();
let mut val_21: JSValue = core::mem::zeroed();
let mut ret_18: i32 = 0;
let mut vm_block: u16 = 288;
// The C restart label performs the bytecode switch directly. Fetch before
// lowering the remaining C control-flow blocks, so every opcode does not
// dispatch once to restart and again to its operation. Forward-only lowering
// nodes are folded into their destinations. Single-predecessor C blocks use
// direct Rust branches instead of returning through the state dispatcher.
// All C statements and interrupt polls retain their original order.
'vm_dispatch: loop {
while vm_block == 285 {
// The official *pc++ bytecode switch: retain constant patterns at opt0.
// Equality guards otherwise compare each opcode sequentially.
vm_block = ({ let assigned = ((*({ let old = pc; pc = (pc).offset(1); old })) as i32); opcode = assigned; assigned } as u16);
if vm_block == OP_nop { vm_block = 285; }
}
match vm_block {
// Direct C opcode cases; non-opcode control labels are outside the byte range.


// C line ?
256 => {
std::process::abort();
}

// C line 20583 labels: done
257 => {
let mut pval: *mut JSValue;
if ((((!(((!((((((((*(b)).var_ref_count) as i32)) != ((0 as i32))) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 {
// C line 20585
let _ = close_var_refs(rt, b, sf);
{

// C line 20588
js_host_free_frame_diagnostics(rt, sf);
let _ = { let assigned = local_buf; pval = assigned; assigned };
{

// C line 20588
'cfg_5: loop {
if ((((pval) < (sp)) as i32)) != 0 {
// C line 20589
let _ = JS_FreeValue(ctx, *(pval));
// C line ?
let _ = { let old = pval; pval = (pval).offset(1); old };
continue 'cfg_5;
} else {
{

// C line 20592
let _ = { let assigned = (*(sf)).prev_frame; (*(rt)).current_stack_frame = assigned; assigned };
// C line 20593
return ret_val;
}
}
}
}
}
} else {
{

// C line 20588
js_host_free_frame_diagnostics(rt, sf);
let _ = { let assigned = local_buf; pval = assigned; assigned };
{

// C line 20588
'cfg_5: loop {
if ((((pval) < (sp)) as i32)) != 0 {
// C line 20589
let _ = JS_FreeValue(ctx, *(pval));
// C line ?
let _ = { let old = pval; pval = (pval).offset(1); old };
continue 'cfg_5;
} else {
{

// C line 20592
let _ = { let assigned = (*(sf)).prev_frame; (*(rt)).current_stack_frame = assigned; assigned };
// C line 20593
return ret_val;
}
}
}
}
}
}
}

// C line 20553
258 => {
let mut pos_1: i32;
let mut val_22: JSValue;
if ((!(((*(rt)).current_exception_is_uncatchable) != 0) as i32)) != 0 {
{

// C line 20554
'cfg_13: loop {
if ((((sp) > (stack_buf)) as i32)) != 0 {
// C line 20555
val_22 = *({ sp = (sp).offset(-1); sp });
// C line 20556
let _ = JS_FreeValue(ctx, val_22);
// C line 20557
if (((((((val_22).tag) as i32)) == ((JS_TAG_CATCH_OFFSET as i32))) as i32)) != 0 {
// C line 20558
pos_1 = ((((val_22).u).uint64) as i32);
// C line 20559
if ((((pos_1) == ((0 as i32))) as i32)) != 0 {
// C line 20561
let _ = JS_FreeValue(ctx, *(sp).offset((((1 as i32)).wrapping_neg()) as isize));
// C line 20562
let _ = { let old = sp; sp = (sp).offset(-1); old };
// C line 20563
let _ = JS_IteratorClose(ctx, *(sp).offset((((1 as i32)).wrapping_neg()) as isize), (1 as i32));
continue 'cfg_13;
} else {
// C line 20565
js_host_catch_exception(rt, sf, sp);
let _ = { let assigned = (*(rt)).current_exception; *({ let old = sp; sp = (sp).offset(1); old }) = assigned; assigned };
// C line 20566
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNINITIALIZED as i32)) as i64) }; (*(rt)).current_exception = assigned; assigned };
// C line 20567
let _ = { let assigned = ((*(b)).byte_code_buf).offset(((pos_1) as isize)); pc = assigned; assigned };
// C line 20568
vm_block = 285; continue 'vm_dispatch;
}
} else {
continue 'cfg_13;
}
} else {
{

// C line 20573
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) }; ret_val = assigned; assigned };
// C line 20577
if (((((((*(b)).func_kind()) as i32)) != ((JS_FUNC_NORMAL as i32))) as i32)) != 0 {
{

// C line ? labels: done_generator
let _ = { let assigned = pc; (*(sf)).cur_pc = assigned; assigned };
// C line 20580
let _ = { let assigned = sp; (*(sf)).cur_sp = assigned; assigned };
{

// C line 20592
let _ = { let assigned = (*(sf)).prev_frame; (*(rt)).current_stack_frame = assigned; assigned };
// C line 20593
return ret_val;
}
}
} else {
vm_block = 257; continue 'vm_dispatch;
}
}
}
}
}
} else {
{

// C line 20573
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) }; ret_val = assigned; assigned };
// C line 20577
if (((((((*(b)).func_kind()) as i32)) != ((JS_FUNC_NORMAL as i32))) as i32)) != 0 {
{

// C line ? labels: done_generator
let _ = { let assigned = pc; (*(sf)).cur_pc = assigned; assigned };
// C line 20580
let _ = { let assigned = sp; (*(sf)).cur_sp = assigned; assigned };
{

// C line 20592
let _ = { let assigned = (*(sf)).prev_frame; (*(rt)).current_stack_frame = assigned; assigned };
// C line 20593
return ret_val;
}
}
} else {
vm_block = 257; continue 'vm_dispatch;
}
}
}
}

// C line 20546 labels: exception
259 => {
js_host_vm_exception(ctx, b, pc);
if (is_backtrace_needed(ctx, (*(rt)).current_exception)) != 0 {
// C line 20550
let _ = { let assigned = pc; (*(sf)).cur_pc = assigned; assigned };
// C line 20551
let _ = build_backtrace(ctx, (*(rt)).current_exception, ((core::ptr::null_mut::<c_void>()) as *const c_char), (0 as i32), (0 as i32), (0 as i32));
vm_block = 258; continue 'vm_dispatch;
} else {
vm_block = 258; continue 'vm_dispatch;
}
}

// C line 20540
OP_invalid => {
let _ = JS_ThrowInternalError(ctx, format_args!("invalid opcode: pc={} opcode=0x{:02x}", (((((pc).offset_from((*(b)).byte_code_buf) as i64)).wrapping_sub((((1 as i32)) as i64))) as i32), opcode));
// C line 20542
vm_block = 259; continue 'vm_dispatch;
}

// C line 20495
OP_is_undefined_or_null => {
if (((((((((((*(sp).offset((((1 as i32)).wrapping_neg()) as isize)).tag) as i32)) == ((JS_TAG_UNDEFINED as i32))) as i32)) != 0) || ((((((((*(sp).offset((((1 as i32)).wrapping_neg()) as isize)).tag) as i32)) == ((JS_TAG_NULL as i32))) as i32)) != 0)) as i32)) != 0 {
// C line ? labels: set_true
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((1 as i32)) as u32)) as u64) }, tag: (((JS_TAG_BOOL as i32)) as i64) }; *(sp).offset((((1 as i32)).wrapping_neg()) as isize) = assigned; assigned };
// C line 20533
vm_block = 285; continue 'vm_dispatch;
} else {
// C line ? labels: free_and_set_false
let _ = JS_FreeValue(ctx, *(sp).offset((((1 as i32)).wrapping_neg()) as isize));
// C line 20536
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_BOOL as i32)) as i64) }; *(sp).offset((((1 as i32)).wrapping_neg()) as isize) = assigned; assigned };
// C line 20537
vm_block = 285; continue 'vm_dispatch;
}
}

// C line 20489
OP_initial_yield => {
let _ = { let assigned = JS_NewInt32(ctx, (3 as i32)); ret_val = assigned; assigned };
// C line 20490
{

// C line ? labels: done_generator
let _ = { let assigned = pc; (*(sf)).cur_pc = assigned; assigned };
// C line 20580
let _ = { let assigned = sp; (*(sf)).cur_sp = assigned; assigned };
{

// C line 20592
let _ = { let assigned = (*(sf)).prev_frame; (*(rt)).current_stack_frame = assigned; assigned };
// C line 20593
return ret_val;
}
}
}

// C line 20486
OP_return_async => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; ret_val = assigned; assigned };
// C line 20487
{

// C line ? labels: done_generator
let _ = { let assigned = pc; (*(sf)).cur_pc = assigned; assigned };
// C line 20580
let _ = { let assigned = sp; (*(sf)).cur_sp = assigned; assigned };
{

// C line 20592
let _ = { let assigned = (*(sf)).prev_frame; (*(rt)).current_stack_frame = assigned; assigned };
// C line 20593
return ret_val;
}
}
}

// C line 20483
OP_async_yield_star | OP_yield_star => {
let _ = { let assigned = JS_NewInt32(ctx, (2 as i32)); ret_val = assigned; assigned };
// C line 20484
{

// C line ? labels: done_generator
let _ = { let assigned = pc; (*(sf)).cur_pc = assigned; assigned };
// C line 20580
let _ = { let assigned = sp; (*(sf)).cur_sp = assigned; assigned };
{

// C line 20592
let _ = { let assigned = (*(sf)).prev_frame; (*(rt)).current_stack_frame = assigned; assigned };
// C line 20593
return ret_val;
}
}
}

// C line 20479
OP_yield => {
let _ = { let assigned = JS_NewInt32(ctx, (1 as i32)); ret_val = assigned; assigned };
// C line 20480
{

// C line ? labels: done_generator
let _ = { let assigned = pc; (*(sf)).cur_pc = assigned; assigned };
// C line 20580
let _ = { let assigned = sp; (*(sf)).cur_sp = assigned; assigned };
{

// C line 20592
let _ = { let assigned = (*(sf)).prev_frame; (*(rt)).current_stack_frame = assigned; assigned };
// C line 20593
return ret_val;
}
}
}

// C line 20476
OP_await => {
let _ = { let assigned = JS_NewInt32(ctx, (0 as i32)); ret_val = assigned; assigned };
// C line 20477
{

// C line ? labels: done_generator
let _ = { let assigned = pc; (*(sf)).cur_pc = assigned; assigned };
// C line 20580
let _ = { let assigned = sp; (*(sf)).cur_sp = assigned; assigned };
{

// C line 20592
let _ = { let assigned = (*(sf)).prev_frame; (*(rt)).current_stack_frame = assigned; assigned };
// C line 20593
return ret_val;
}
}
}

// C line 20465
260 => {
let _ = { pc = (pc).offset((((diff_2).wrapping_sub((5 as i32))) as isize)); pc };
vm_block = 285; continue 'vm_dispatch;
}

// C line 20452
261 => {
let _ = { let assigned = JS_HasProperty(ctx, obj_8, atom_17); ret_18 = assigned; assigned };
// C line 20453
if ((((!(((!(((((ret_18) < ((0 as i32))) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 {
vm_block = 259; continue 'vm_dispatch;
} else {
// C line 20455
if ((!((ret_18) != 0) as i32)) != 0 {
// C line 20456
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; val_21 = assigned; assigned };
{

// C line 20462
let _ = { let assigned = val_21; *({ let old = sp; sp = (sp).offset(1); old }) = assigned; assigned };
// C line 20463
vm_block = 260; continue 'vm_dispatch;
}
} else {
// C line 20458
let _ = { let assigned = JS_GetProperty(ctx, obj_8, atom_17); val_21 = assigned; assigned };
// C line 20459
if ((((!(((!((JS_IsException(val_21)) != 0) as i32)) != 0) as i32)) as i64)) != 0 {
vm_block = 259; continue 'vm_dispatch;
} else {
{

// C line 20462
let _ = { let assigned = val_21; *({ let old = sp; sp = (sp).offset(1); old }) = assigned; assigned };
// C line 20463
vm_block = 260; continue 'vm_dispatch;
}
}
}
}
}

// C line 20447
262 => {
let _ = { let assigned = JS_AtomToValue(ctx, atom_17); *({ let old = sp; sp = (sp).offset(1); old }) = assigned; assigned };
// C line 20448
vm_block = 260; continue 'vm_dispatch;
}

// C line 20439
263 => {
let _ = { let assigned = JS_DeleteProperty(ctx, obj_8, atom_17, (0 as i32)); ret_18 = assigned; assigned };
// C line 20440
if ((((!(((!(((((ret_18) < ((0 as i32))) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 {
vm_block = 259; continue 'vm_dispatch;
} else {
// C line 20442
let _ = JS_FreeValue(ctx, *(sp).offset((((1 as i32)).wrapping_neg()) as isize));
// C line 20443
let _ = { let assigned = JS_NewBool(ctx, ret_18); *(sp).offset((((1 as i32)).wrapping_neg()) as isize) = assigned; assigned };
// C line 20444
vm_block = 260; continue 'vm_dispatch;
}
}

// C line 20422
264 => {
let _ = { let assigned = JS_HasProperty(ctx, obj_8, atom_17); ret_18 = assigned; assigned };
// C line 20423
if ((((!(((!(((((ret_18) <= ((0 as i32))) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 {
// C line 20424
if ((((ret_18) < ((0 as i32))) as i32)) != 0 {
vm_block = 259; continue 'vm_dispatch;
} else {
// C line 20426
if (is_strict_mode(ctx)) != 0 {
// C line 20427
let _ = JS_ThrowReferenceErrorNotDefined(ctx, atom_17);
// C line 20428
vm_block = 259; continue 'vm_dispatch;
} else {
{

// C line 20431
let _ = { let assigned = JS_SetPropertyInternal(ctx, obj_8, atom_17, *(sp).offset((((2 as i32)).wrapping_neg()) as isize), obj_8, ((1 as i32)).wrapping_shl(((15 as i32)) as u32)); ret_18 = assigned; assigned };
// C line 20433
let _ = JS_FreeValue(ctx, *(sp).offset((((1 as i32)).wrapping_neg()) as isize));
// C line 20434
let _ = { sp = (sp).offset(-(((2 as i32)) as isize)); sp };
// C line 20435
if ((((!(((!(((((ret_18) < ((0 as i32))) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 {
vm_block = 259; continue 'vm_dispatch;
} else {
vm_block = 260; continue 'vm_dispatch;
}
}
}
}
} else {
{

// C line 20431
let _ = { let assigned = JS_SetPropertyInternal(ctx, obj_8, atom_17, *(sp).offset((((2 as i32)).wrapping_neg()) as isize), obj_8, ((1 as i32)).wrapping_shl(((15 as i32)) as u32)); ret_18 = assigned; assigned };
// C line 20433
let _ = JS_FreeValue(ctx, *(sp).offset((((1 as i32)).wrapping_neg()) as isize));
// C line 20434
let _ = { sp = (sp).offset(-(((2 as i32)) as isize)); sp };
// C line 20435
if ((((!(((!(((((ret_18) < ((0 as i32))) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 {
vm_block = 259; continue 'vm_dispatch;
} else {
vm_block = 260; continue 'vm_dispatch;
}
}
}
}

// C line 20404
265 => {
let _ = { let assigned = JS_HasProperty(ctx, obj_8, atom_17); ret_18 = assigned; assigned };
// C line 20405
if ((((!(((!(((((ret_18) <= ((0 as i32))) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 {
// C line 20406
if ((((ret_18) < ((0 as i32))) as i32)) != 0 {
vm_block = 259; continue 'vm_dispatch;
} else {
// C line 20408
if (is_strict_mode(ctx)) != 0 {
// C line 20409
let _ = JS_ThrowReferenceErrorNotDefined(ctx, atom_17);
// C line 20410
vm_block = 259; continue 'vm_dispatch;
} else {
// C line 20412
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; val_21 = assigned; assigned };
{

// C line 20418
let _ = set_value(ctx, core::ptr::addr_of_mut!(*(sp).offset((((1 as i32)).wrapping_neg()) as isize)), val_21);
// C line 20419
vm_block = 260; continue 'vm_dispatch;
}
}
}
} else {
// C line 20414
let _ = { let assigned = JS_GetProperty(ctx, obj_8, atom_17); val_21 = assigned; assigned };
// C line 20415
if ((((!(((!((JS_IsException(val_21)) != 0) as i32)) != 0) as i32)) as i64)) != 0 {
vm_block = 259; continue 'vm_dispatch;
} else {
{

// C line 20418
let _ = set_value(ctx, core::ptr::addr_of_mut!(*(sp).offset((((1 as i32)).wrapping_neg()) as isize)), val_21);
// C line 20419
vm_block = 260; continue 'vm_dispatch;
}
}
}
}

// C line 20383
OP_with_get_ref | OP_with_make_ref | OP_with_delete_var | OP_with_put_var | OP_with_get_var => {
let mut is_with: i32;
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
// C line 20391
if ((((!(((!(((((ret_18) < ((0 as i32))) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 {
vm_block = 259; continue 'vm_dispatch;
} else {
// C line 20393
if (ret_18) != 0 {
// C line 20394
if (is_with) != 0 {
// C line 20395
let _ = { let assigned = js_has_unscopable(ctx, obj_8, atom_17); ret_18 = assigned; assigned };
// C line 20396
if ((((!(((!(((((ret_18) < ((0 as i32))) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 {
vm_block = 259; continue 'vm_dispatch;
} else {
// C line 20398
if (ret_18) != 0 {
{

// C line ? labels: no_with
let _ = JS_FreeValue(ctx, *(sp).offset((((1 as i32)).wrapping_neg()) as isize));
// C line 20470
let _ = { let old = sp; sp = (sp).offset(-1); old };
vm_block = 285; continue 'vm_dispatch;
}
} else {
{

// C line 20401
vm_block = match opcode { x if x == (OP_with_get_ref as i32) => 261, x if x == (OP_with_make_ref as i32) => 262, x if x == (OP_with_delete_var as i32) => 263, x if x == (OP_with_put_var as i32) => 264, x if x == (OP_with_get_var as i32) => 265, _ => 260, }; continue 'vm_dispatch;
}
}
}
} else {
{

// C line 20401
vm_block = match opcode { x if x == (OP_with_get_ref as i32) => 261, x if x == (OP_with_make_ref as i32) => 262, x if x == (OP_with_delete_var as i32) => 263, x if x == (OP_with_put_var as i32) => 264, x if x == (OP_with_get_var as i32) => 265, _ => 260, }; continue 'vm_dispatch;
}
}
} else {
{

// C line ? labels: no_with
let _ = JS_FreeValue(ctx, *(sp).offset((((1 as i32)).wrapping_neg()) as isize));
// C line 20470
let _ = { let old = sp; sp = (sp).offset(-1); old };
vm_block = 285; continue 'vm_dispatch;
}
}
}
}

// C line ?
266 => {
let _ = { let assigned = pc; (*(sf)).cur_pc = assigned; assigned };
// C line 20353
let _ = { let assigned = JS_ToPropertyKey(ctx, *(sp).offset((((1 as i32)).wrapping_neg()) as isize)); ret_val = assigned; assigned };
// C line 20354
if (JS_IsException(ret_val)) != 0 {
vm_block = 259; continue 'vm_dispatch;
} else {
// C line 20356
let _ = JS_FreeValue(ctx, *(sp).offset((((1 as i32)).wrapping_neg()) as isize));
// C line 20357
let _ = { let assigned = ret_val; *(sp).offset((((1 as i32)).wrapping_neg()) as isize) = assigned; assigned };
// C line 20358
vm_block = 285; continue 'vm_dispatch;
}
}

// C line 20346
OP_to_propkey => {
vm_block = match (((*(sp).offset((((1 as i32)).wrapping_neg()) as isize)).tag) as i32) { x if x == (JS_TAG_SYMBOL as i32) => 285, x if x == (JS_TAG_STRING as i32) => 285, x if x == (JS_TAG_INT as i32) => 285, _ => 266, }; continue 'vm_dispatch;
}

// C line 20335
OP_to_object => {
if (((((((*(sp).offset((((1 as i32)).wrapping_neg()) as isize)).tag) as i32)) != ((JS_TAG_OBJECT as i32))) as i32)) != 0 {
// C line 20336
let _ = { let assigned = pc; (*(sf)).cur_pc = assigned; assigned };
// C line 20337
let _ = { let assigned = JS_ToObject(ctx, *(sp).offset((((1 as i32)).wrapping_neg()) as isize)); ret_val = assigned; assigned };
// C line 20338
if (JS_IsException(ret_val)) != 0 {
vm_block = 259; continue 'vm_dispatch;
} else {
// C line 20340
let _ = JS_FreeValue(ctx, *(sp).offset((((1 as i32)).wrapping_neg()) as isize));
// C line 20341
let _ = { let assigned = ret_val; *(sp).offset((((1 as i32)).wrapping_neg()) as isize) = assigned; assigned };
vm_block = 285; continue 'vm_dispatch;
}
} else {
vm_block = 285; continue 'vm_dispatch;
}
}

// C line 20323
OP_delete_var => {
let mut ret_17: i32;
let mut atom_16: JSAtom;
let _ = { let assigned = crate::cutils_header::get_u32(pc); atom_16 = assigned; assigned };
// C line 20324
let _ = { pc = (pc).offset((((4 as i32)) as isize)); pc };
// C line 20325
let _ = { let assigned = pc; (*(sf)).cur_pc = assigned; assigned };
// C line 20327
let _ = { let assigned = JS_DeleteGlobalVar(ctx, atom_16); ret_17 = assigned; assigned };
// C line 20328
if ((((!(((!(((((ret_17) < ((0 as i32))) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 {
vm_block = 259; continue 'vm_dispatch;
} else {
// C line 20330
let _ = { let assigned = JS_NewBool(ctx, ret_17); *({ let old = sp; sp = (sp).offset(1); old }) = assigned; assigned };
// C line 20332
vm_block = 285; continue 'vm_dispatch;
}
}

// C line 20313
OP_delete => {
let _ = { let assigned = pc; (*(sf)).cur_pc = assigned; assigned };
// C line 20314
if (js_operator_delete(ctx, sp)) != 0 {
vm_block = 259; continue 'vm_dispatch;
} else {
// C line 20316
let _ = { let old = sp; sp = (sp).offset(-1); old };
// C line 20317
vm_block = 285; continue 'vm_dispatch;
}
}

// C line 20306
OP_typeof => {
let mut atom_15: JSAtom;
let mut op1_32: JSValue;
let _ = { let assigned = *(sp).offset((((1 as i32)).wrapping_neg()) as isize); op1_32 = assigned; assigned };
// C line 20307
let _ = { let assigned = ((js_operator_typeof(ctx, op1_32)) as JSAtom); atom_15 = assigned; assigned };
// C line 20308
let _ = JS_FreeValue(ctx, op1_32);
// C line 20309
let _ = { let assigned = JS_AtomToString(ctx, atom_15); *(sp).offset((((1 as i32)).wrapping_neg()) as isize) = assigned; assigned };
// C line 20311
vm_block = 285; continue 'vm_dispatch;
}

// C line 20296
OP_instanceof => {
let _ = { let assigned = pc; (*(sf)).cur_pc = assigned; assigned };
// C line 20297
if (js_operator_instanceof(ctx, sp)) != 0 {
vm_block = 259; continue 'vm_dispatch;
} else {
// C line 20299
let _ = { let old = sp; sp = (sp).offset(-1); old };
// C line 20300
vm_block = 285; continue 'vm_dispatch;
}
}

// C line 20290
OP_private_in => {
let _ = { let assigned = pc; (*(sf)).cur_pc = assigned; assigned };
// C line 20291
if (js_operator_private_in(ctx, sp)) != 0 {
vm_block = 259; continue 'vm_dispatch;
} else {
// C line 20293
let _ = { let old = sp; sp = (sp).offset(-1); old };
// C line 20294
vm_block = 285; continue 'vm_dispatch;
}
}

// C line 20284
OP_in => {
let _ = { let assigned = pc; (*(sf)).cur_pc = assigned; assigned };
// C line 20285
if (js_operator_in(ctx, sp)) != 0 {
vm_block = 259; continue 'vm_dispatch;
} else {
// C line 20287
let _ = { let old = sp; sp = (sp).offset(-1); old };
// C line 20288
vm_block = 285; continue 'vm_dispatch;
}
}

// C line 20281
OP_strict_neq => {
let mut op2_19: JSValue;
let mut op1_31: JSValue;
let _ = { let assigned = *(sp).offset((((2 as i32)).wrapping_neg()) as isize); op1_31 = assigned; assigned };
// C line 20281
let _ = { let assigned = *(sp).offset((((1 as i32)).wrapping_neg()) as isize); op2_19 = assigned; assigned };
// C line 20281
if ((((!(((!((((((((((op1_31).tag) as i32)) | ((((op2_19).tag) as i32)))) == ((0 as i32))) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 {
// C line 20281
let _ = { let assigned = JS_NewBool(ctx, (((((((op1_31).u).uint64) as i32)) != (((((op2_19).u).uint64) as i32))) as i32)); *(sp).offset((((2 as i32)).wrapping_neg()) as isize) = assigned; assigned };
// C line 20281
let _ = { let old = sp; sp = (sp).offset(-1); old };
vm_block = 285; continue 'vm_dispatch;
} else {
// C line 20281
let _ = { let assigned = pc; (*(sf)).cur_pc = assigned; assigned };
// C line 20281
if (js_strict_eq_slow(ctx, sp, (1 as i32))) != 0 {
vm_block = 259; continue 'vm_dispatch;
} else {
// C line 20281
let _ = { let old = sp; sp = (sp).offset(-1); old };
vm_block = 285; continue 'vm_dispatch;
}
}
}

// C line 20280
OP_strict_eq => {
let mut op2_18: JSValue;
let mut op1_30: JSValue;
let _ = { let assigned = *(sp).offset((((2 as i32)).wrapping_neg()) as isize); op1_30 = assigned; assigned };
// C line 20280
let _ = { let assigned = *(sp).offset((((1 as i32)).wrapping_neg()) as isize); op2_18 = assigned; assigned };
// C line 20280
if ((((!(((!((((((((((op1_30).tag) as i32)) | ((((op2_18).tag) as i32)))) == ((0 as i32))) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 {
// C line 20280
let _ = { let assigned = JS_NewBool(ctx, (((((((op1_30).u).uint64) as i32)) == (((((op2_18).u).uint64) as i32))) as i32)); *(sp).offset((((2 as i32)).wrapping_neg()) as isize) = assigned; assigned };
// C line 20280
let _ = { let old = sp; sp = (sp).offset(-1); old };
vm_block = 285; continue 'vm_dispatch;
} else {
// C line 20280
let _ = { let assigned = pc; (*(sf)).cur_pc = assigned; assigned };
// C line 20280
if (js_strict_eq_slow(ctx, sp, (0 as i32))) != 0 {
vm_block = 259; continue 'vm_dispatch;
} else {
// C line 20280
let _ = { let old = sp; sp = (sp).offset(-1); old };
vm_block = 285; continue 'vm_dispatch;
}
}
}

// C line 20279
OP_neq => {
let mut op2_17: JSValue;
let mut op1_29: JSValue;
let _ = { let assigned = *(sp).offset((((2 as i32)).wrapping_neg()) as isize); op1_29 = assigned; assigned };
// C line 20279
let _ = { let assigned = *(sp).offset((((1 as i32)).wrapping_neg()) as isize); op2_17 = assigned; assigned };
// C line 20279
if ((((!(((!((((((((((op1_29).tag) as i32)) | ((((op2_17).tag) as i32)))) == ((0 as i32))) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 {
// C line 20279
let _ = { let assigned = JS_NewBool(ctx, (((((((op1_29).u).uint64) as i32)) != (((((op2_17).u).uint64) as i32))) as i32)); *(sp).offset((((2 as i32)).wrapping_neg()) as isize) = assigned; assigned };
// C line 20279
let _ = { let old = sp; sp = (sp).offset(-1); old };
vm_block = 285; continue 'vm_dispatch;
} else {
// C line 20279
let _ = { let assigned = pc; (*(sf)).cur_pc = assigned; assigned };
// C line 20279
if (js_eq_slow(ctx, sp, (1 as i32))) != 0 {
vm_block = 259; continue 'vm_dispatch;
} else {
// C line 20279
let _ = { let old = sp; sp = (sp).offset(-1); old };
vm_block = 285; continue 'vm_dispatch;
}
}
}

// C line 20278
OP_eq => {
let mut op2_16: JSValue;
let mut op1_28: JSValue;
let _ = { let assigned = *(sp).offset((((2 as i32)).wrapping_neg()) as isize); op1_28 = assigned; assigned };
// C line 20278
let _ = { let assigned = *(sp).offset((((1 as i32)).wrapping_neg()) as isize); op2_16 = assigned; assigned };
// C line 20278
if ((((!(((!((((((((((op1_28).tag) as i32)) | ((((op2_16).tag) as i32)))) == ((0 as i32))) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 {
// C line 20278
let _ = { let assigned = JS_NewBool(ctx, (((((((op1_28).u).uint64) as i32)) == (((((op2_16).u).uint64) as i32))) as i32)); *(sp).offset((((2 as i32)).wrapping_neg()) as isize) = assigned; assigned };
// C line 20278
let _ = { let old = sp; sp = (sp).offset(-1); old };
vm_block = 285; continue 'vm_dispatch;
} else {
// C line 20278
let _ = { let assigned = pc; (*(sf)).cur_pc = assigned; assigned };
// C line 20278
if (js_eq_slow(ctx, sp, (0 as i32))) != 0 {
vm_block = 259; continue 'vm_dispatch;
} else {
// C line 20278
let _ = { let old = sp; sp = (sp).offset(-1); old };
vm_block = 285; continue 'vm_dispatch;
}
}
}

// C line 20277
OP_gte => {
let mut op2_15: JSValue;
let mut op1_27: JSValue;
let _ = { let assigned = *(sp).offset((((2 as i32)).wrapping_neg()) as isize); op1_27 = assigned; assigned };
// C line 20277
let _ = { let assigned = *(sp).offset((((1 as i32)).wrapping_neg()) as isize); op2_15 = assigned; assigned };
// C line 20277
if ((((!(((!((((((((((op1_27).tag) as i32)) | ((((op2_15).tag) as i32)))) == ((0 as i32))) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 {
// C line 20277
let _ = { let assigned = JS_NewBool(ctx, (((((((op1_27).u).uint64) as i32)) >= (((((op2_15).u).uint64) as i32))) as i32)); *(sp).offset((((2 as i32)).wrapping_neg()) as isize) = assigned; assigned };
// C line 20277
let _ = { let old = sp; sp = (sp).offset(-1); old };
vm_block = 285; continue 'vm_dispatch;
} else {
// C line 20277
let _ = { let assigned = pc; (*(sf)).cur_pc = assigned; assigned };
// C line 20277
if (js_relational_slow(ctx, sp, opcode)) != 0 {
vm_block = 259; continue 'vm_dispatch;
} else {
// C line 20277
let _ = { let old = sp; sp = (sp).offset(-1); old };
vm_block = 285; continue 'vm_dispatch;
}
}
}

// C line 20276
OP_gt => {
let mut op2_14: JSValue;
let mut op1_26: JSValue;
let _ = { let assigned = *(sp).offset((((2 as i32)).wrapping_neg()) as isize); op1_26 = assigned; assigned };
// C line 20276
let _ = { let assigned = *(sp).offset((((1 as i32)).wrapping_neg()) as isize); op2_14 = assigned; assigned };
// C line 20276
if ((((!(((!((((((((((op1_26).tag) as i32)) | ((((op2_14).tag) as i32)))) == ((0 as i32))) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 {
// C line 20276
let _ = { let assigned = JS_NewBool(ctx, (((((((op1_26).u).uint64) as i32)) > (((((op2_14).u).uint64) as i32))) as i32)); *(sp).offset((((2 as i32)).wrapping_neg()) as isize) = assigned; assigned };
// C line 20276
let _ = { let old = sp; sp = (sp).offset(-1); old };
vm_block = 285; continue 'vm_dispatch;
} else {
// C line 20276
let _ = { let assigned = pc; (*(sf)).cur_pc = assigned; assigned };
// C line 20276
if (js_relational_slow(ctx, sp, opcode)) != 0 {
vm_block = 259; continue 'vm_dispatch;
} else {
// C line 20276
let _ = { let old = sp; sp = (sp).offset(-1); old };
vm_block = 285; continue 'vm_dispatch;
}
}
}

// C line 20275
OP_lte => {
let mut op2_13: JSValue;
let mut op1_25: JSValue;
let _ = { let assigned = *(sp).offset((((2 as i32)).wrapping_neg()) as isize); op1_25 = assigned; assigned };
// C line 20275
let _ = { let assigned = *(sp).offset((((1 as i32)).wrapping_neg()) as isize); op2_13 = assigned; assigned };
// C line 20275
if ((((!(((!((((((((((op1_25).tag) as i32)) | ((((op2_13).tag) as i32)))) == ((0 as i32))) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 {
// C line 20275
let _ = { let assigned = JS_NewBool(ctx, (((((((op1_25).u).uint64) as i32)) <= (((((op2_13).u).uint64) as i32))) as i32)); *(sp).offset((((2 as i32)).wrapping_neg()) as isize) = assigned; assigned };
// C line 20275
let _ = { let old = sp; sp = (sp).offset(-1); old };
vm_block = 285; continue 'vm_dispatch;
} else {
// C line 20275
let _ = { let assigned = pc; (*(sf)).cur_pc = assigned; assigned };
// C line 20275
if (js_relational_slow(ctx, sp, opcode)) != 0 {
vm_block = 259; continue 'vm_dispatch;
} else {
// C line 20275
let _ = { let old = sp; sp = (sp).offset(-1); old };
vm_block = 285; continue 'vm_dispatch;
}
}
}

// C line 20274
OP_lt => {
let mut op2_12: JSValue;
let mut op1_24: JSValue;
let _ = { let assigned = *(sp).offset((((2 as i32)).wrapping_neg()) as isize); op1_24 = assigned; assigned };
// C line 20274
let _ = { let assigned = *(sp).offset((((1 as i32)).wrapping_neg()) as isize); op2_12 = assigned; assigned };
// C line 20274
if ((((!(((!((((((((((op1_24).tag) as i32)) | ((((op2_12).tag) as i32)))) == ((0 as i32))) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 {
// C line 20274
let _ = { let assigned = JS_NewBool(ctx, (((((((op1_24).u).uint64) as i32)) < (((((op2_12).u).uint64) as i32))) as i32)); *(sp).offset((((2 as i32)).wrapping_neg()) as isize) = assigned; assigned };
// C line 20274
let _ = { let old = sp; sp = (sp).offset(-1); old };
vm_block = 285; continue 'vm_dispatch;
} else {
// C line 20274
let _ = { let assigned = pc; (*(sf)).cur_pc = assigned; assigned };
// C line 20274
if (js_relational_slow(ctx, sp, opcode)) != 0 {
vm_block = 259; continue 'vm_dispatch;
} else {
// C line 20274
let _ = { let old = sp; sp = (sp).offset(-1); old };
vm_block = 285; continue 'vm_dispatch;
}
}
}

// C line 20239
OP_xor => {
let mut op2_11: JSValue;
let mut op1_23: JSValue;
let _ = { let assigned = *(sp).offset((((2 as i32)).wrapping_neg()) as isize); op1_23 = assigned; assigned };
// C line 20240
let _ = { let assigned = *(sp).offset((((1 as i32)).wrapping_neg()) as isize); op2_11 = assigned; assigned };
// C line 20241
if ((((!(((!((((((((((op1_23).tag) as i32)) | ((((op2_11).tag) as i32)))) == ((0 as i32))) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 {
// C line 20242
let _ = { let assigned = JS_NewInt32(ctx, ((((((op1_23).u).uint64) as i32)) ^ (((((op2_11).u).uint64) as i32)))); *(sp).offset((((2 as i32)).wrapping_neg()) as isize) = assigned; assigned };
// C line 20245
let _ = { let old = sp; sp = (sp).offset(-1); old };
vm_block = 285; continue 'vm_dispatch;
} else {
// C line 20247
let _ = { let assigned = pc; (*(sf)).cur_pc = assigned; assigned };
// C line 20248
if (js_binary_logic_slow(ctx, sp, opcode)) != 0 {
vm_block = 259; continue 'vm_dispatch;
} else {
// C line 20250
let _ = { let old = sp; sp = (sp).offset(-1); old };
vm_block = 285; continue 'vm_dispatch;
}
}
}

// C line 20221
OP_or => {
let mut op2_10: JSValue;
let mut op1_22: JSValue;
let _ = { let assigned = *(sp).offset((((2 as i32)).wrapping_neg()) as isize); op1_22 = assigned; assigned };
// C line 20222
let _ = { let assigned = *(sp).offset((((1 as i32)).wrapping_neg()) as isize); op2_10 = assigned; assigned };
// C line 20223
if ((((!(((!((((((((((op1_22).tag) as i32)) | ((((op2_10).tag) as i32)))) == ((0 as i32))) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 {
// C line 20224
let _ = { let assigned = JS_NewInt32(ctx, ((((((op1_22).u).uint64) as i32)) | (((((op2_10).u).uint64) as i32)))); *(sp).offset((((2 as i32)).wrapping_neg()) as isize) = assigned; assigned };
// C line 20227
let _ = { let old = sp; sp = (sp).offset(-1); old };
vm_block = 285; continue 'vm_dispatch;
} else {
// C line 20229
let _ = { let assigned = pc; (*(sf)).cur_pc = assigned; assigned };
// C line 20230
if (js_binary_logic_slow(ctx, sp, opcode)) != 0 {
vm_block = 259; continue 'vm_dispatch;
} else {
// C line 20232
let _ = { let old = sp; sp = (sp).offset(-1); old };
vm_block = 285; continue 'vm_dispatch;
}
}
}

// C line 20203
OP_and => {
let mut op2_9: JSValue;
let mut op1_21: JSValue;
let _ = { let assigned = *(sp).offset((((2 as i32)).wrapping_neg()) as isize); op1_21 = assigned; assigned };
// C line 20204
let _ = { let assigned = *(sp).offset((((1 as i32)).wrapping_neg()) as isize); op2_9 = assigned; assigned };
// C line 20205
if ((((!(((!((((((((((op1_21).tag) as i32)) | ((((op2_9).tag) as i32)))) == ((0 as i32))) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 {
// C line 20206
let _ = { let assigned = JS_NewInt32(ctx, ((((((op1_21).u).uint64) as i32)) & (((((op2_9).u).uint64) as i32)))); *(sp).offset((((2 as i32)).wrapping_neg()) as isize) = assigned; assigned };
// C line 20209
let _ = { let old = sp; sp = (sp).offset(-1); old };
vm_block = 285; continue 'vm_dispatch;
} else {
// C line 20211
let _ = { let assigned = pc; (*(sf)).cur_pc = assigned; assigned };
// C line 20212
if (js_binary_logic_slow(ctx, sp, opcode)) != 0 {
vm_block = 259; continue 'vm_dispatch;
} else {
// C line 20214
let _ = { let old = sp; sp = (sp).offset(-1); old };
vm_block = 285; continue 'vm_dispatch;
}
}
}

// C line 20183
OP_sar => {
let mut v2_5: u32;
let mut op2_8: JSValue;
let mut op1_20: JSValue;
let _ = { let assigned = *(sp).offset((((2 as i32)).wrapping_neg()) as isize); op1_20 = assigned; assigned };
// C line 20184
let _ = { let assigned = *(sp).offset((((1 as i32)).wrapping_neg()) as isize); op2_8 = assigned; assigned };
// C line 20185
if ((((!(((!((((((((((op1_20).tag) as i32)) | ((((op2_8).tag) as i32)))) == ((0 as i32))) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 {
// C line 20187
let _ = { let assigned = ((((((op2_8).u).uint64) as i32)) as u32); v2_5 = assigned; assigned };
// C line 20188
let _ = { v2_5 = ((v2_5) & ((((31 as i32)) as u32))); v2_5 };
// C line 20189
let _ = { let assigned = JS_NewInt32(ctx, (((((op1_20).u).uint64) as i32)).wrapping_shr((v2_5) as u32)); *(sp).offset((((2 as i32)).wrapping_neg()) as isize) = assigned; assigned };
// C line 20191
let _ = { let old = sp; sp = (sp).offset(-1); old };
vm_block = 285; continue 'vm_dispatch;
} else {
// C line 20193
let _ = { let assigned = pc; (*(sf)).cur_pc = assigned; assigned };
// C line 20194
if (js_binary_logic_slow(ctx, sp, opcode)) != 0 {
vm_block = 259; continue 'vm_dispatch;
} else {
// C line 20196
let _ = { let old = sp; sp = (sp).offset(-1); old };
vm_block = 285; continue 'vm_dispatch;
}
}
}

// C line 20162
OP_shr => {
let mut v2_4: u32;
let mut op2_7: JSValue;
let mut op1_19: JSValue;
let _ = { let assigned = *(sp).offset((((2 as i32)).wrapping_neg()) as isize); op1_19 = assigned; assigned };
// C line 20163
let _ = { let assigned = *(sp).offset((((1 as i32)).wrapping_neg()) as isize); op2_7 = assigned; assigned };
// C line 20164
if ((((!(((!((((((((((op1_19).tag) as i32)) | ((((op2_7).tag) as i32)))) == ((0 as i32))) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 {
// C line 20166
let _ = { let assigned = ((((((op2_7).u).uint64) as i32)) as u32); v2_4 = assigned; assigned };
// C line 20167
let _ = { v2_4 = ((v2_4) & ((((31 as i32)) as u32))); v2_4 };
// C line 20168
let _ = { let assigned = JS_NewUint32(ctx, (((((((op1_19).u).uint64) as i32)) as u32)).wrapping_shr((v2_4) as u32)); *(sp).offset((((2 as i32)).wrapping_neg()) as isize) = assigned; assigned };
// C line 20171
let _ = { let old = sp; sp = (sp).offset(-1); old };
vm_block = 285; continue 'vm_dispatch;
} else {
// C line 20173
let _ = { let assigned = pc; (*(sf)).cur_pc = assigned; assigned };
// C line 20174
if (js_shr_slow(ctx, sp)) != 0 {
vm_block = 259; continue 'vm_dispatch;
} else {
// C line 20176
let _ = { let old = sp; sp = (sp).offset(-1); old };
vm_block = 285; continue 'vm_dispatch;
}
}
}

// C line 20142
OP_shl => {
let mut v2_3: u32;
let mut v1_3: u32;
let mut op2_6: JSValue;
let mut op1_18: JSValue;
let _ = { let assigned = *(sp).offset((((2 as i32)).wrapping_neg()) as isize); op1_18 = assigned; assigned };
// C line 20143
let _ = { let assigned = *(sp).offset((((1 as i32)).wrapping_neg()) as isize); op2_6 = assigned; assigned };
// C line 20144
if ((((!(((!((((((((((op1_18).tag) as i32)) | ((((op2_6).tag) as i32)))) == ((0 as i32))) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 {
// C line 20146
let _ = { let assigned = ((((((op1_18).u).uint64) as i32)) as u32); v1_3 = assigned; assigned };
// C line 20147
let _ = { let assigned = ((((((op2_6).u).uint64) as i32)) as u32); v2_3 = assigned; assigned };
// C line 20148
let _ = { v2_3 = ((v2_3) & ((((31 as i32)) as u32))); v2_3 };
// C line 20149
let _ = { let assigned = JS_NewInt32(ctx, (((v1_3).wrapping_shl((v2_3) as u32)) as i32)); *(sp).offset((((2 as i32)).wrapping_neg()) as isize) = assigned; assigned };
// C line 20150
let _ = { let old = sp; sp = (sp).offset(-1); old };
vm_block = 285; continue 'vm_dispatch;
} else {
// C line 20152
let _ = { let assigned = pc; (*(sf)).cur_pc = assigned; assigned };
// C line 20153
if (js_binary_logic_slow(ctx, sp, opcode)) != 0 {
vm_block = 259; continue 'vm_dispatch;
} else {
// C line 20155
let _ = { let old = sp; sp = (sp).offset(-1); old };
vm_block = 285; continue 'vm_dispatch;
}
}
}

// C line 20128
OP_not => {
let mut op1_17: JSValue;
let _ = { let assigned = *(sp).offset((((1 as i32)).wrapping_neg()) as isize); op1_17 = assigned; assigned };
// C line 20129
if (((((((op1_17).tag) as i32)) == ((JS_TAG_INT as i32))) as i32)) != 0 {
// C line 20130
let _ = { let assigned = JS_NewInt32(ctx, (!(((((op1_17).u).uint64) as i32)))); *(sp).offset((((1 as i32)).wrapping_neg()) as isize) = assigned; assigned };
vm_block = 285; continue 'vm_dispatch;
} else {
// C line 20132
let _ = { let assigned = pc; (*(sf)).cur_pc = assigned; assigned };
// C line 20133
if (js_not_slow(ctx, sp)) != 0 {
vm_block = 259; continue 'vm_dispatch;
} else {
vm_block = 285; continue 'vm_dispatch;
}
}
}

// C line 20104
OP_dec_loc => {
let mut idx_28: i32;
let mut val_20: i32;
let _ = { let assigned = ((*(pc)) as i32); idx_28 = assigned; assigned };
// C line 20105
let _ = { pc = (pc).offset((((1 as i32)) as isize)); pc };
// C line 20107
let _ = { let assigned = *(var_buf).offset((idx_28) as isize); op1_16 = assigned; assigned };
// C line 20108
if (((((((op1_16).tag) as i32)) == ((JS_TAG_INT as i32))) as i32)) != 0 {
// C line 20109
let _ = { let assigned = ((((op1_16).u).uint64) as i32); val_20 = assigned; assigned };
// C line 20110
if ((((!(((!(((((val_20) == ((((2147483647 as i32)).wrapping_neg()).wrapping_sub((1 as i32)))) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 {
{

// C line ? labels: dec_loc_slow
let _ = { let assigned = pc; (*(sf)).cur_pc = assigned; assigned };
// C line 20118
let _ = { let assigned = JS_DupValue(ctx, op1_16); op1_16 = assigned; assigned };
// C line 20119
if (js_unary_arith_slow(ctx, (core::ptr::addr_of_mut!(op1_16)).offset((((1 as i32)) as isize)), (OP_dec as i32))) != 0 {
vm_block = 259; continue 'vm_dispatch;
} else {
// C line 20121
let _ = set_value(ctx, core::ptr::addr_of_mut!(*(var_buf).offset((idx_28) as isize)), op1_16);
vm_block = 285; continue 'vm_dispatch;
}
}
} else {
// C line 20112
let _ = { let assigned = JS_NewInt32(ctx, (val_20).wrapping_sub((1 as i32))); *(var_buf).offset((idx_28) as isize) = assigned; assigned };
vm_block = 285; continue 'vm_dispatch;
}
} else {
{

// C line ? labels: dec_loc_slow
let _ = { let assigned = pc; (*(sf)).cur_pc = assigned; assigned };
// C line 20118
let _ = { let assigned = JS_DupValue(ctx, op1_16); op1_16 = assigned; assigned };
// C line 20119
if (js_unary_arith_slow(ctx, (core::ptr::addr_of_mut!(op1_16)).offset((((1 as i32)) as isize)), (OP_dec as i32))) != 0 {
vm_block = 259; continue 'vm_dispatch;
} else {
// C line 20121
let _ = set_value(ctx, core::ptr::addr_of_mut!(*(var_buf).offset((idx_28) as isize)), op1_16);
vm_block = 285; continue 'vm_dispatch;
}
}
}
}

// C line 20078
OP_inc_loc => {
let mut idx_27: i32;
let mut val_19: i32;
let _ = { let assigned = ((*(pc)) as i32); idx_27 = assigned; assigned };
// C line 20079
let _ = { pc = (pc).offset((((1 as i32)) as isize)); pc };
// C line 20081
let _ = { let assigned = *(var_buf).offset((idx_27) as isize); op1_15 = assigned; assigned };
// C line 20082
if (((((((op1_15).tag) as i32)) == ((JS_TAG_INT as i32))) as i32)) != 0 {
// C line 20083
let _ = { let assigned = ((((op1_15).u).uint64) as i32); val_19 = assigned; assigned };
// C line 20084
if ((((!(((!(((((val_19) == ((2147483647 as i32))) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 {
{

// C line ? labels: inc_loc_slow
let _ = { let assigned = pc; (*(sf)).cur_pc = assigned; assigned };
// C line 20092
let _ = { let assigned = JS_DupValue(ctx, op1_15); op1_15 = assigned; assigned };
// C line 20093
if (js_unary_arith_slow(ctx, (core::ptr::addr_of_mut!(op1_15)).offset((((1 as i32)) as isize)), (OP_inc as i32))) != 0 {
vm_block = 259; continue 'vm_dispatch;
} else {
// C line 20095
let _ = set_value(ctx, core::ptr::addr_of_mut!(*(var_buf).offset((idx_27) as isize)), op1_15);
vm_block = 285; continue 'vm_dispatch;
}
}
} else {
// C line 20086
let _ = { let assigned = JS_NewInt32(ctx, (val_19).wrapping_add((1 as i32))); *(var_buf).offset((idx_27) as isize) = assigned; assigned };
vm_block = 285; continue 'vm_dispatch;
}
} else {
{

// C line ? labels: inc_loc_slow
let _ = { let assigned = pc; (*(sf)).cur_pc = assigned; assigned };
// C line 20092
let _ = { let assigned = JS_DupValue(ctx, op1_15); op1_15 = assigned; assigned };
// C line 20093
if (js_unary_arith_slow(ctx, (core::ptr::addr_of_mut!(op1_15)).offset((((1 as i32)) as isize)), (OP_inc as i32))) != 0 {
vm_block = 259; continue 'vm_dispatch;
} else {
// C line 20095
let _ = set_value(ctx, core::ptr::addr_of_mut!(*(var_buf).offset((idx_27) as isize)), op1_15);
vm_block = 285; continue 'vm_dispatch;
}
}
}
}

// C line 20058
OP_post_dec => {
let mut val_18: i32;
let mut op1_14: JSValue;
let _ = { let assigned = *(sp).offset((((1 as i32)).wrapping_neg()) as isize); op1_14 = assigned; assigned };
// C line 20059
if (((((((op1_14).tag) as i32)) == ((JS_TAG_INT as i32))) as i32)) != 0 {
// C line 20060
let _ = { let assigned = ((((op1_14).u).uint64) as i32); val_18 = assigned; assigned };
// C line 20061
if ((((!(((!(((((val_18) == ((((2147483647 as i32)).wrapping_neg()).wrapping_sub((1 as i32)))) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 {
{

// C line ? labels: post_dec_slow
let _ = { let assigned = pc; (*(sf)).cur_pc = assigned; assigned };
// C line 20067
if (js_post_inc_slow(ctx, sp, opcode)) != 0 {
vm_block = 259; continue 'vm_dispatch;
} else {
{

// C line 20070
let _ = { let old = sp; sp = (sp).offset(1); old };
// C line 20072
vm_block = 285; continue 'vm_dispatch;
}
}
}
} else {
// C line 20063
let _ = { let assigned = JS_NewInt32(ctx, (val_18).wrapping_sub((1 as i32))); *(sp).offset(((0 as i32)) as isize) = assigned; assigned };
{

// C line 20070
let _ = { let old = sp; sp = (sp).offset(1); old };
// C line 20072
vm_block = 285; continue 'vm_dispatch;
}
}
} else {
{

// C line ? labels: post_dec_slow
let _ = { let assigned = pc; (*(sf)).cur_pc = assigned; assigned };
// C line 20067
if (js_post_inc_slow(ctx, sp, opcode)) != 0 {
vm_block = 259; continue 'vm_dispatch;
} else {
{

// C line 20070
let _ = { let old = sp; sp = (sp).offset(1); old };
// C line 20072
vm_block = 285; continue 'vm_dispatch;
}
}
}
}
}

// C line 20039
OP_post_inc => {
let mut val_17: i32;
let mut op1_13: JSValue;
let _ = { let assigned = *(sp).offset((((1 as i32)).wrapping_neg()) as isize); op1_13 = assigned; assigned };
// C line 20040
if (((((((op1_13).tag) as i32)) == ((JS_TAG_INT as i32))) as i32)) != 0 {
// C line 20041
let _ = { let assigned = ((((op1_13).u).uint64) as i32); val_17 = assigned; assigned };
// C line 20042
if ((((!(((!(((((val_17) == ((2147483647 as i32))) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 {
{

// C line ? labels: post_inc_slow
let _ = { let assigned = pc; (*(sf)).cur_pc = assigned; assigned };
// C line 20048
if (js_post_inc_slow(ctx, sp, opcode)) != 0 {
vm_block = 259; continue 'vm_dispatch;
} else {
{

// C line 20051
let _ = { let old = sp; sp = (sp).offset(1); old };
// C line 20053
vm_block = 285; continue 'vm_dispatch;
}
}
}
} else {
// C line 20044
let _ = { let assigned = JS_NewInt32(ctx, (val_17).wrapping_add((1 as i32))); *(sp).offset(((0 as i32)) as isize) = assigned; assigned };
{

// C line 20051
let _ = { let old = sp; sp = (sp).offset(1); old };
// C line 20053
vm_block = 285; continue 'vm_dispatch;
}
}
} else {
{

// C line ? labels: post_inc_slow
let _ = { let assigned = pc; (*(sf)).cur_pc = assigned; assigned };
// C line 20048
if (js_post_inc_slow(ctx, sp, opcode)) != 0 {
vm_block = 259; continue 'vm_dispatch;
} else {
{

// C line 20051
let _ = { let old = sp; sp = (sp).offset(1); old };
// C line 20053
vm_block = 285; continue 'vm_dispatch;
}
}
}
}
}

// C line 20021
OP_dec => {
let mut val_16: i32;
let mut op1_12: JSValue;
let _ = { let assigned = *(sp).offset((((1 as i32)).wrapping_neg()) as isize); op1_12 = assigned; assigned };
// C line 20022
if (((((((op1_12).tag) as i32)) == ((JS_TAG_INT as i32))) as i32)) != 0 {
// C line 20023
let _ = { let assigned = ((((op1_12).u).uint64) as i32); val_16 = assigned; assigned };
// C line 20024
if ((((!(((!(((((val_16) == ((((2147483647 as i32)).wrapping_neg()).wrapping_sub((1 as i32)))) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 {
{

// C line ? labels: dec_slow
let _ = { let assigned = pc; (*(sf)).cur_pc = assigned; assigned };
// C line 20030
if (js_unary_arith_slow(ctx, sp, opcode)) != 0 {
vm_block = 259; continue 'vm_dispatch;
} else {
vm_block = 285; continue 'vm_dispatch;
}
}
} else {
// C line 20026
let _ = { let assigned = JS_NewInt32(ctx, (val_16).wrapping_sub((1 as i32))); *(sp).offset((((1 as i32)).wrapping_neg()) as isize) = assigned; assigned };
vm_block = 285; continue 'vm_dispatch;
}
} else {
{

// C line ? labels: dec_slow
let _ = { let assigned = pc; (*(sf)).cur_pc = assigned; assigned };
// C line 20030
if (js_unary_arith_slow(ctx, sp, opcode)) != 0 {
vm_block = 259; continue 'vm_dispatch;
} else {
vm_block = 285; continue 'vm_dispatch;
}
}
}
}

// C line 20003
OP_inc => {
let mut val_15: i32;
let mut op1_11: JSValue;
let _ = { let assigned = *(sp).offset((((1 as i32)).wrapping_neg()) as isize); op1_11 = assigned; assigned };
// C line 20004
if (((((((op1_11).tag) as i32)) == ((JS_TAG_INT as i32))) as i32)) != 0 {
// C line 20005
let _ = { let assigned = ((((op1_11).u).uint64) as i32); val_15 = assigned; assigned };
// C line 20006
if ((((!(((!(((((val_15) == ((2147483647 as i32))) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 {
{

// C line ? labels: inc_slow
let _ = { let assigned = pc; (*(sf)).cur_pc = assigned; assigned };
// C line 20012
if (js_unary_arith_slow(ctx, sp, opcode)) != 0 {
vm_block = 259; continue 'vm_dispatch;
} else {
vm_block = 285; continue 'vm_dispatch;
}
}
} else {
// C line 20008
let _ = { let assigned = JS_NewInt32(ctx, (val_15).wrapping_add((1 as i32))); *(sp).offset((((1 as i32)).wrapping_neg()) as isize) = assigned; assigned };
vm_block = 285; continue 'vm_dispatch;
}
} else {
{

// C line ? labels: inc_slow
let _ = { let assigned = pc; (*(sf)).cur_pc = assigned; assigned };
// C line 20012
if (js_unary_arith_slow(ctx, sp, opcode)) != 0 {
vm_block = 259; continue 'vm_dispatch;
} else {
vm_block = 285; continue 'vm_dispatch;
}
}
}
}

// C line 19972
OP_neg => {
let mut d_1: f64;
let mut val_14: i32;
let mut tag_2: u32;
let mut op1_10: JSValue;
let _ = { let assigned = *(sp).offset((((1 as i32)).wrapping_neg()) as isize); op1_10 = assigned; assigned };
// C line 19973
let _ = { let assigned = (((((op1_10).tag) as i32)) as u32); tag_2 = assigned; assigned };
// C line 19974
if ((((((((((((tag_2) == ((((JS_TAG_INT as i32)) as u32))) as i32)) != 0) || (((((tag_2) == ((((JS_TAG_BOOL as i32)) as u32))) as i32)) != 0)) as i32)) != 0) || (((((tag_2) == ((((JS_TAG_NULL as i32)) as u32))) as i32)) != 0)) as i32)) != 0 {
// C line 19977
let _ = { let assigned = ((((op1_10).u).uint64) as i32); val_14 = assigned; assigned };
// C line 19979
if ((((!(((!(((((val_14) == ((0 as i32))) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 {
// C line 19980
let _ = { let assigned = (-((0 as f64))); d_1 = assigned; assigned };
// C line 19981
{

// C line ? labels: neg_fp_res
let _ = { let assigned = __JS_NewFloat64(ctx, d_1); *(sp).offset((((1 as i32)).wrapping_neg()) as isize) = assigned; assigned };
vm_block = 285; continue 'vm_dispatch;
}
} else {
// C line 19983
if ((((!(((!(((((val_14) == ((((2147483647 as i32)).wrapping_neg()).wrapping_sub((1 as i32)))) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 {
// C line 19984
let _ = { let assigned = (-(((val_14) as f64))); d_1 = assigned; assigned };
// C line 19985
{

// C line ? labels: neg_fp_res
let _ = { let assigned = __JS_NewFloat64(ctx, d_1); *(sp).offset((((1 as i32)).wrapping_neg()) as isize) = assigned; assigned };
vm_block = 285; continue 'vm_dispatch;
}
} else {
// C line 19987
let _ = { let assigned = JS_NewInt32(ctx, (val_14).wrapping_neg()); *(sp).offset((((1 as i32)).wrapping_neg()) as isize) = assigned; assigned };
vm_block = 285; continue 'vm_dispatch;
}
}
} else {
// C line 19988
if ((((tag_2) == ((((JS_TAG_FLOAT64 as i32)) as u32))) as i32)) != 0 {
// C line 19989
let _ = { let assigned = (-(((op1_10).u).float64)); d_1 = assigned; assigned };
{

// C line ? labels: neg_fp_res
let _ = { let assigned = __JS_NewFloat64(ctx, d_1); *(sp).offset((((1 as i32)).wrapping_neg()) as isize) = assigned; assigned };
vm_block = 285; continue 'vm_dispatch;
}
} else {
// C line 19993
let _ = { let assigned = pc; (*(sf)).cur_pc = assigned; assigned };
// C line 19994
if (js_unary_arith_slow(ctx, sp, opcode)) != 0 {
vm_block = 259; continue 'vm_dispatch;
} else {
vm_block = 285; continue 'vm_dispatch;
}
}
}
}

// C line 19954
OP_plus => {
let mut tag_1: u32;
let mut op1_9: JSValue;
let _ = { let assigned = *(sp).offset((((1 as i32)).wrapping_neg()) as isize); op1_9 = assigned; assigned };
// C line 19955
let _ = { let assigned = (((((op1_9).tag) as i32)) as u32); tag_1 = assigned; assigned };
// C line 19956
if ((((((((tag_1) == ((((JS_TAG_INT as i32)) as u32))) as i32)) != 0) || (((((tag_1) == ((((JS_TAG_FLOAT64 as i32)) as u32))) as i32)) != 0)) as i32)) != 0 {
vm_block = 285; continue 'vm_dispatch;
} else {
// C line ?
if ((((((((tag_1) == ((((JS_TAG_NULL as i32)) as u32))) as i32)) != 0) || (((((tag_1) == ((((JS_TAG_BOOL as i32)) as u32))) as i32)) != 0)) as i32)) != 0 {
// C line 19958
let _ = { let assigned = JS_NewInt32(ctx, ((((op1_9).u).uint64) as i32)); *(sp).offset((((1 as i32)).wrapping_neg()) as isize) = assigned; assigned };
vm_block = 285; continue 'vm_dispatch;
} else {
// C line 19960
let _ = { let assigned = pc; (*(sf)).cur_pc = assigned; assigned };
// C line 19961
if (js_unary_arith_slow(ctx, sp, opcode)) != 0 {
vm_block = 259; continue 'vm_dispatch;
} else {
vm_block = 285; continue 'vm_dispatch;
}
}
}
}

// C line ? labels: binary_arith_slow
OP_pow => {
let _ = { let assigned = pc; (*(sf)).cur_pc = assigned; assigned };
// C line 19945
if (js_binary_arith_slow(ctx, sp, opcode)) != 0 {
vm_block = 259; continue 'vm_dispatch;
} else {
// C line 19947
let _ = { let old = sp; sp = (sp).offset(-1); old };
// C line 19948
vm_block = 285; continue 'vm_dispatch;
}
}

// C line 19924
OP_mod => {
let mut r_4: i32;
let mut v2_2: i32;
let mut v1_2: i32;
let mut op2_5: JSValue;
let mut op1_8: JSValue;
let _ = { let assigned = *(sp).offset((((2 as i32)).wrapping_neg()) as isize); op1_8 = assigned; assigned };
// C line 19925
let _ = { let assigned = *(sp).offset((((1 as i32)).wrapping_neg()) as isize); op2_5 = assigned; assigned };
// C line 19926
if ((((!(((!((((((((((op1_8).tag) as i32)) | ((((op2_5).tag) as i32)))) == ((0 as i32))) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 {
// C line 19928
let _ = { let assigned = ((((op1_8).u).uint64) as i32); v1_2 = assigned; assigned };
// C line 19929
let _ = { let assigned = ((((op2_5).u).uint64) as i32); v2_2 = assigned; assigned };
// C line 19932
if ((((!(((!(((((((((v1_2) < ((0 as i32))) as i32)) != 0) || (((((v2_2) <= ((0 as i32))) as i32)) != 0)) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 {
vm_block = OP_pow; continue 'vm_dispatch;
} else {
// C line 19934
let _ = { let assigned = ((v1_2) % (v2_2)); r_4 = assigned; assigned };
// C line 19935
let _ = { let assigned = JS_NewInt32(ctx, r_4); *(sp).offset((((2 as i32)).wrapping_neg()) as isize) = assigned; assigned };
// C line 19936
let _ = { let old = sp; sp = (sp).offset(-1); old };
// C line 19941
vm_block = 285; continue 'vm_dispatch;
}
} else {
vm_block = OP_pow; continue 'vm_dispatch;
}
}

// C line 19908
OP_div => {
let mut v2_1: i32;
let mut v1_1: i32;
let mut op2_4: JSValue;
let mut op1_7: JSValue;
let _ = { let assigned = *(sp).offset((((2 as i32)).wrapping_neg()) as isize); op1_7 = assigned; assigned };
// C line 19909
let _ = { let assigned = *(sp).offset((((1 as i32)).wrapping_neg()) as isize); op2_4 = assigned; assigned };
// C line 19910
if ((((!(((!((((((((((op1_7).tag) as i32)) | ((((op2_4).tag) as i32)))) == ((0 as i32))) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 {
// C line 19912
let _ = { let assigned = ((((op1_7).u).uint64) as i32); v1_1 = assigned; assigned };
// C line 19913
let _ = { let assigned = ((((op2_4).u).uint64) as i32); v2_1 = assigned; assigned };
// C line 19914
let _ = { let assigned = JS_NewFloat64(ctx, ((((v1_1) as f64)) / (((v2_1) as f64)))); *(sp).offset((((2 as i32)).wrapping_neg()) as isize) = assigned; assigned };
// C line 19915
let _ = { let old = sp; sp = (sp).offset(-1); old };
// C line 19920
vm_block = 285; continue 'vm_dispatch;
} else {
vm_block = OP_pow; continue 'vm_dispatch;
}
}

// C line 19889
267 => {
let mut d2_2: f64;
if (((((((((op2_3).tag) as i32)) as u32)) == ((((JS_TAG_FLOAT64 as i32)) as u32))) as i32)) != 0 {
// C line 19890
let _ = { let assigned = ((op2_3).u).float64; d2_2 = assigned; assigned };
{

// C line 19896
let _ = { let assigned = ((d1_2) * (d2_2)); d = assigned; assigned };
{

// C line ? labels: mul_fp_res
let _ = { let assigned = __JS_NewFloat64(ctx, d); *(sp).offset((((2 as i32)).wrapping_neg()) as isize) = assigned; assigned };
// C line 19899
let _ = { let old = sp; sp = (sp).offset(-1); old };
vm_block = 285; continue 'vm_dispatch;
}
}
} else {
// C line 19891
if (((((((op2_3).tag) as i32)) == ((JS_TAG_INT as i32))) as i32)) != 0 {
// C line 19892
let _ = { let assigned = ((((((op2_3).u).uint64) as i32)) as f64); d2_2 = assigned; assigned };
{

// C line 19896
let _ = { let assigned = ((d1_2) * (d2_2)); d = assigned; assigned };
{

// C line ? labels: mul_fp_res
let _ = { let assigned = __JS_NewFloat64(ctx, d); *(sp).offset((((2 as i32)).wrapping_neg()) as isize) = assigned; assigned };
// C line 19899
let _ = { let old = sp; sp = (sp).offset(-1); old };
vm_block = 285; continue 'vm_dispatch;
}
}
} else {
vm_block = OP_pow; continue 'vm_dispatch;
}
}
}

// C line 19860
OP_mul => {
let mut r_3: i64;
let mut v2: i32;
let mut v1: i32;
let mut op1_6: JSValue;
let _ = { let assigned = *(sp).offset((((2 as i32)).wrapping_neg()) as isize); op1_6 = assigned; assigned };
// C line 19861
let _ = { let assigned = *(sp).offset((((1 as i32)).wrapping_neg()) as isize); op2_3 = assigned; assigned };
// C line 19862
if ((((!(((!((((((((((op1_6).tag) as i32)) | ((((op2_3).tag) as i32)))) == ((0 as i32))) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 {
// C line 19865
let _ = { let assigned = ((((op1_6).u).uint64) as i32); v1 = assigned; assigned };
// C line 19866
let _ = { let assigned = ((((op2_3).u).uint64) as i32); v2 = assigned; assigned };
// C line 19867
let _ = { let assigned = (((v1) as i64)).wrapping_mul(((v2) as i64)); r_3 = assigned; assigned };
// C line 19868
if ((((!(((!(((((((((r_3) as i32)) as i64)) != (r_3)) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 {
// C line 19869
let _ = { let assigned = ((r_3) as f64); d = assigned; assigned };
// C line 19870
{

// C line ? labels: mul_fp_res
let _ = { let assigned = __JS_NewFloat64(ctx, d); *(sp).offset((((2 as i32)).wrapping_neg()) as isize) = assigned; assigned };
// C line 19899
let _ = { let old = sp; sp = (sp).offset(-1); old };
vm_block = 285; continue 'vm_dispatch;
}
} else {
// C line 19873
if ((((!(((!(((((((((r_3) == ((((0 as i32)) as i64))) as i32)) != 0) && (((((((v1) | (v2))) < ((0 as i32))) as i32)) != 0)) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 {
// C line 19874
let _ = { let assigned = (-((0 as f64))); d = assigned; assigned };
// C line 19875
{

// C line ? labels: mul_fp_res
let _ = { let assigned = __JS_NewFloat64(ctx, d); *(sp).offset((((2 as i32)).wrapping_neg()) as isize) = assigned; assigned };
// C line 19899
let _ = { let old = sp; sp = (sp).offset(-1); old };
vm_block = 285; continue 'vm_dispatch;
}
} else {
// C line 19877
let _ = { let assigned = JS_NewInt32(ctx, ((r_3) as i32)); *(sp).offset((((2 as i32)).wrapping_neg()) as isize) = assigned; assigned };
// C line 19878
let _ = { let old = sp; sp = (sp).offset(-1); old };
vm_block = 285; continue 'vm_dispatch;
}
}
} else {
// C line 19879
if (((((((((((((op1_6).tag) as i32)) as u32)) == ((((JS_TAG_FLOAT64 as i32)) as u32))) as i32)) != 0) || ((((((((((op2_3).tag) as i32)) as u32)) == ((((JS_TAG_FLOAT64 as i32)) as u32))) as i32)) != 0)) as i32)) != 0 {
// C line 19882
if (((((((((op1_6).tag) as i32)) as u32)) == ((((JS_TAG_FLOAT64 as i32)) as u32))) as i32)) != 0 {
// C line 19883
let _ = { let assigned = ((op1_6).u).float64; d1_2 = assigned; assigned };
vm_block = 267; continue 'vm_dispatch;
} else {
// C line 19884
if (((((((op1_6).tag) as i32)) == ((JS_TAG_INT as i32))) as i32)) != 0 {
// C line 19885
let _ = { let assigned = ((((((op1_6).u).uint64) as i32)) as f64); d1_2 = assigned; assigned };
vm_block = 267; continue 'vm_dispatch;
} else {
vm_block = OP_pow; continue 'vm_dispatch;
}
}
} else {
vm_block = OP_pow; continue 'vm_dispatch;
}
}
}

// C line 19821
OP_sub => {
let mut d2_1: f64;
let mut d1_1: f64;
let mut r_2: i64;
let mut op2_2: JSValue;
let mut op1_5: JSValue;
let _ = { let assigned = *(sp).offset((((2 as i32)).wrapping_neg()) as isize); op1_5 = assigned; assigned };
// C line 19822
let _ = { let assigned = *(sp).offset((((1 as i32)).wrapping_neg()) as isize); op2_2 = assigned; assigned };
// C line 19823
if ((((!(((!((((((((((op1_5).tag) as i32)) | ((((op2_2).tag) as i32)))) == ((0 as i32))) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 {
// C line 19825
let _ = { let assigned = (((((((op1_5).u).uint64) as i32)) as i64)).wrapping_sub(((((((op2_2).u).uint64) as i32)) as i64)); r_2 = assigned; assigned };
// C line 19826
if ((((!(((!(((((((((r_2) as i32)) as i64)) != (r_2)) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 {
// C line 19827
let _ = { let assigned = __JS_NewFloat64(ctx, ((r_2) as f64)); *(sp).offset((((2 as i32)).wrapping_neg()) as isize) = assigned; assigned };
{

// C line 19831
let _ = { let old = sp; sp = (sp).offset(-1); old };
vm_block = 285; continue 'vm_dispatch;
}
} else {
// C line 19829
let _ = { let assigned = JS_NewInt32(ctx, ((r_2) as i32)); *(sp).offset((((2 as i32)).wrapping_neg()) as isize) = assigned; assigned };
{

// C line 19831
let _ = { let old = sp; sp = (sp).offset(-1); old };
vm_block = 285; continue 'vm_dispatch;
}
}
} else {
// C line 19832
if (((((((((((((op1_5).tag) as i32)) as u32)) == ((((JS_TAG_FLOAT64 as i32)) as u32))) as i32)) != 0) || ((((((((((op2_2).tag) as i32)) as u32)) == ((((JS_TAG_FLOAT64 as i32)) as u32))) as i32)) != 0)) as i32)) != 0 {
// C line 19835
if (((((((((op1_5).tag) as i32)) as u32)) == ((((JS_TAG_FLOAT64 as i32)) as u32))) as i32)) != 0 {
// C line 19836
let _ = { let assigned = ((op1_5).u).float64; d1_1 = assigned; assigned };
{

// C line 19842
if (((((((((op2_2).tag) as i32)) as u32)) == ((((JS_TAG_FLOAT64 as i32)) as u32))) as i32)) != 0 {
// C line 19843
let _ = { let assigned = ((op2_2).u).float64; d2_1 = assigned; assigned };
{

// C line 19849
let _ = { let assigned = __JS_NewFloat64(ctx, ((d1_1) - (d2_1))); *(sp).offset((((2 as i32)).wrapping_neg()) as isize) = assigned; assigned };
// C line 19850
let _ = { let old = sp; sp = (sp).offset(-1); old };
vm_block = 285; continue 'vm_dispatch;
}
} else {
// C line 19844
if (((((((op2_2).tag) as i32)) == ((JS_TAG_INT as i32))) as i32)) != 0 {
// C line 19845
let _ = { let assigned = ((((((op2_2).u).uint64) as i32)) as f64); d2_1 = assigned; assigned };
{

// C line 19849
let _ = { let assigned = __JS_NewFloat64(ctx, ((d1_1) - (d2_1))); *(sp).offset((((2 as i32)).wrapping_neg()) as isize) = assigned; assigned };
// C line 19850
let _ = { let old = sp; sp = (sp).offset(-1); old };
vm_block = 285; continue 'vm_dispatch;
}
} else {
vm_block = OP_pow; continue 'vm_dispatch;
}
}
}
} else {
// C line 19837
if (((((((op1_5).tag) as i32)) == ((JS_TAG_INT as i32))) as i32)) != 0 {
// C line 19838
let _ = { let assigned = ((((((op1_5).u).uint64) as i32)) as f64); d1_1 = assigned; assigned };
{

// C line 19842
if (((((((((op2_2).tag) as i32)) as u32)) == ((((JS_TAG_FLOAT64 as i32)) as u32))) as i32)) != 0 {
// C line 19843
let _ = { let assigned = ((op2_2).u).float64; d2_1 = assigned; assigned };
{

// C line 19849
let _ = { let assigned = __JS_NewFloat64(ctx, ((d1_1) - (d2_1))); *(sp).offset((((2 as i32)).wrapping_neg()) as isize) = assigned; assigned };
// C line 19850
let _ = { let old = sp; sp = (sp).offset(-1); old };
vm_block = 285; continue 'vm_dispatch;
}
} else {
// C line 19844
if (((((((op2_2).tag) as i32)) == ((JS_TAG_INT as i32))) as i32)) != 0 {
// C line 19845
let _ = { let assigned = ((((((op2_2).u).uint64) as i32)) as f64); d2_1 = assigned; assigned };
{

// C line 19849
let _ = { let assigned = __JS_NewFloat64(ctx, ((d1_1) - (d2_1))); *(sp).offset((((2 as i32)).wrapping_neg()) as isize) = assigned; assigned };
// C line 19850
let _ = { let old = sp; sp = (sp).offset(-1); old };
vm_block = 285; continue 'vm_dispatch;
}
} else {
vm_block = OP_pow; continue 'vm_dispatch;
}
}
}
} else {
vm_block = OP_pow; continue 'vm_dispatch;
}
}
} else {
vm_block = OP_pow; continue 'vm_dispatch;
}
}
}

// C line 19774
OP_add_loc => {
let mut r_1: i64;
let mut idx_26: i32;
let mut pv: *mut JSValue;
let mut op2_1: JSValue;
let _ = { let assigned = ((*(pc)) as i32); idx_26 = assigned; assigned };
// C line 19775
let _ = { pc = (pc).offset((((1 as i32)) as isize)); pc };
// C line 19777
let _ = { let assigned = *(sp).offset((((1 as i32)).wrapping_neg()) as isize); op2_1 = assigned; assigned };
// C line 19778
let _ = { let assigned = core::ptr::addr_of_mut!(*(var_buf).offset((idx_26) as isize)); pv = assigned; assigned };
// C line 19779
if ((((!(((!((((((((((*(pv)).tag) as i32)) | ((((op2_1).tag) as i32)))) == ((0 as i32))) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 {
// C line 19781
let _ = { let assigned = (((((((*(pv)).u).uint64) as i32)) as i64)).wrapping_add(((((((op2_1).u).uint64) as i32)) as i64)); r_1 = assigned; assigned };
// C line 19782
if ((((!(((!(((((((((r_1) as i32)) as i64)) != (r_1)) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 {
// C line 19783
let _ = { let assigned = __JS_NewFloat64(ctx, ((r_1) as f64)); *(pv) = assigned; assigned };
{

// C line 19787
let _ = { let old = sp; sp = (sp).offset(-1); old };
vm_block = 285; continue 'vm_dispatch;
}
} else {
// C line 19785
let _ = { let assigned = JS_NewInt32(ctx, ((r_1) as i32)); *(pv) = assigned; assigned };
{

// C line 19787
let _ = { let old = sp; sp = (sp).offset(-1); old };
vm_block = 285; continue 'vm_dispatch;
}
}
} else {
// C line 19788
if (((((((((((((*(pv)).tag) as i32)) as u32)) == ((((JS_TAG_FLOAT64 as i32)) as u32))) as i32)) != 0) && ((((((((((op2_1).tag) as i32)) as u32)) == ((((JS_TAG_FLOAT64 as i32)) as u32))) as i32)) != 0)) as i32)) != 0 {
// C line 19789
let _ = { let assigned = __JS_NewFloat64(ctx, ((((*(pv)).u).float64) + (((op2_1).u).float64))); *(pv) = assigned; assigned };
// C line 19791
let _ = { let old = sp; sp = (sp).offset(-1); old };
vm_block = 285; continue 'vm_dispatch;
} else {
// C line 19792
if (((((((((((*(pv)).tag) as i32)) == ((JS_TAG_STRING as i32))) as i32)) != 0) && ((((((((op2_1).tag) as i32)) == ((JS_TAG_STRING as i32))) as i32)) != 0)) as i32)) != 0 {
// C line 19794
let _ = { let old = sp; sp = (sp).offset(-1); old };
// C line 19795
let _ = { let assigned = pc; (*(sf)).cur_pc = assigned; assigned };
// C line 19796
if (JS_ConcatStringInPlace(ctx, ((((*(pv)).u).ptr) as *mut JSString), op2_1)) != 0 {
// C line 19797
let _ = JS_FreeValue(ctx, op2_1);
vm_block = 285; continue 'vm_dispatch;
} else {
// C line 19799
let _ = { let assigned = JS_ConcatString(ctx, JS_DupValue(ctx, *(pv)), op2_1); op2_1 = assigned; assigned };
// C line 19800
if (JS_IsException(op2_1)) != 0 {
vm_block = 259; continue 'vm_dispatch;
} else {
// C line 19802
let _ = set_value(ctx, pv, op2_1);
vm_block = 285; continue 'vm_dispatch;
}
}
} else {
// C line 19808
let _ = { let assigned = pc; (*(sf)).cur_pc = assigned; assigned };
// C line 19809
let _ = { let assigned = JS_DupValue(ctx, *(pv)); *((ops).as_mut_ptr()).offset(((0 as i32)) as isize) = assigned; assigned };
// C line 19810
let _ = { let assigned = op2_1; *((ops).as_mut_ptr()).offset(((1 as i32)) as isize) = assigned; assigned };
// C line 19811
let _ = { let old = sp; sp = (sp).offset(-1); old };
// C line 19812
if (js_add_slow(ctx, ((ops).as_mut_ptr()).offset((((2 as i32)) as isize)))) != 0 {
vm_block = 259; continue 'vm_dispatch;
} else {
// C line 19814
let _ = set_value(ctx, pv, *((ops).as_mut_ptr()).offset(((0 as i32)) as isize));
vm_block = 285; continue 'vm_dispatch;
}
}
}
}
}

// C line 19725
OP_add => {
let mut d2: f64;
let mut d1: f64;
let mut r: i64;
let mut op2: JSValue;
let mut op1_4: JSValue;
let _ = { let assigned = *(sp).offset((((2 as i32)).wrapping_neg()) as isize); op1_4 = assigned; assigned };
// C line 19726
let _ = { let assigned = *(sp).offset((((1 as i32)).wrapping_neg()) as isize); op2 = assigned; assigned };
// C line 19727
if ((((!(((!((((((((((op1_4).tag) as i32)) | ((((op2).tag) as i32)))) == ((0 as i32))) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 {
// C line 19729
let _ = { let assigned = (((((((op1_4).u).uint64) as i32)) as i64)).wrapping_add(((((((op2).u).uint64) as i32)) as i64)); r = assigned; assigned };
// C line 19730
if ((((!(((!(((((((((r) as i32)) as i64)) != (r)) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 {
// C line 19731
let _ = { let assigned = __JS_NewFloat64(ctx, ((r) as f64)); *(sp).offset((((2 as i32)).wrapping_neg()) as isize) = assigned; assigned };
{

// C line 19735
let _ = { let old = sp; sp = (sp).offset(-1); old };
vm_block = 285; continue 'vm_dispatch;
}
} else {
// C line 19733
let _ = { let assigned = JS_NewInt32(ctx, ((r) as i32)); *(sp).offset((((2 as i32)).wrapping_neg()) as isize) = assigned; assigned };
{

// C line 19735
let _ = { let old = sp; sp = (sp).offset(-1); old };
vm_block = 285; continue 'vm_dispatch;
}
}
} else {
// C line 19736
if (((((((((((((op1_4).tag) as i32)) as u32)) == ((((JS_TAG_FLOAT64 as i32)) as u32))) as i32)) != 0) || ((((((((((op2).tag) as i32)) as u32)) == ((((JS_TAG_FLOAT64 as i32)) as u32))) as i32)) != 0)) as i32)) != 0 {
// C line 19739
if (((((((((op1_4).tag) as i32)) as u32)) == ((((JS_TAG_FLOAT64 as i32)) as u32))) as i32)) != 0 {
// C line 19740
let _ = { let assigned = ((op1_4).u).float64; d1 = assigned; assigned };
{

// C line 19746
if (((((((((op2).tag) as i32)) as u32)) == ((((JS_TAG_FLOAT64 as i32)) as u32))) as i32)) != 0 {
// C line 19747
let _ = { let assigned = ((op2).u).float64; d2 = assigned; assigned };
{

// C line 19753
let _ = { let assigned = __JS_NewFloat64(ctx, ((d1) + (d2))); *(sp).offset((((2 as i32)).wrapping_neg()) as isize) = assigned; assigned };
// C line 19754
let _ = { let old = sp; sp = (sp).offset(-1); old };
vm_block = 285; continue 'vm_dispatch;
}
} else {
// C line 19748
if (((((((op2).tag) as i32)) == ((JS_TAG_INT as i32))) as i32)) != 0 {
// C line 19749
let _ = { let assigned = ((((((op2).u).uint64) as i32)) as f64); d2 = assigned; assigned };
{

// C line 19753
let _ = { let assigned = __JS_NewFloat64(ctx, ((d1) + (d2))); *(sp).offset((((2 as i32)).wrapping_neg()) as isize) = assigned; assigned };
// C line 19754
let _ = { let old = sp; sp = (sp).offset(-1); old };
vm_block = 285; continue 'vm_dispatch;
}
} else {
{

// C line ? labels: add_slow_case
let _ = { let assigned = pc; (*(sf)).cur_pc = assigned; assigned };
// C line 19763
if (js_add_slow(ctx, sp)) != 0 {
vm_block = 259; continue 'vm_dispatch;
} else {
// C line 19765
let _ = { let old = sp; sp = (sp).offset(-1); old };
vm_block = 285; continue 'vm_dispatch;
}
}
}
}
}
} else {
// C line 19741
if (((((((op1_4).tag) as i32)) == ((JS_TAG_INT as i32))) as i32)) != 0 {
// C line 19742
let _ = { let assigned = ((((((op1_4).u).uint64) as i32)) as f64); d1 = assigned; assigned };
{

// C line 19746
if (((((((((op2).tag) as i32)) as u32)) == ((((JS_TAG_FLOAT64 as i32)) as u32))) as i32)) != 0 {
// C line 19747
let _ = { let assigned = ((op2).u).float64; d2 = assigned; assigned };
{

// C line 19753
let _ = { let assigned = __JS_NewFloat64(ctx, ((d1) + (d2))); *(sp).offset((((2 as i32)).wrapping_neg()) as isize) = assigned; assigned };
// C line 19754
let _ = { let old = sp; sp = (sp).offset(-1); old };
vm_block = 285; continue 'vm_dispatch;
}
} else {
// C line 19748
if (((((((op2).tag) as i32)) == ((JS_TAG_INT as i32))) as i32)) != 0 {
// C line 19749
let _ = { let assigned = ((((((op2).u).uint64) as i32)) as f64); d2 = assigned; assigned };
{

// C line 19753
let _ = { let assigned = __JS_NewFloat64(ctx, ((d1) + (d2))); *(sp).offset((((2 as i32)).wrapping_neg()) as isize) = assigned; assigned };
// C line 19754
let _ = { let old = sp; sp = (sp).offset(-1); old };
vm_block = 285; continue 'vm_dispatch;
}
} else {
{

// C line ? labels: add_slow_case
let _ = { let assigned = pc; (*(sf)).cur_pc = assigned; assigned };
// C line 19763
if (js_add_slow(ctx, sp)) != 0 {
vm_block = 259; continue 'vm_dispatch;
} else {
// C line 19765
let _ = { let old = sp; sp = (sp).offset(-1); old };
vm_block = 285; continue 'vm_dispatch;
}
}
}
}
}
} else {
{

// C line ? labels: add_slow_case
let _ = { let assigned = pc; (*(sf)).cur_pc = assigned; assigned };
// C line 19763
if (js_add_slow(ctx, sp)) != 0 {
vm_block = 259; continue 'vm_dispatch;
} else {
// C line 19765
let _ = { let old = sp; sp = (sp).offset(-1); old };
vm_block = 285; continue 'vm_dispatch;
}
}
}
}
} else {
// C line 19755
if (((((JS_IsString(op1_4)) != 0) && ((JS_IsString(op2)) != 0)) as i32)) != 0 {
// C line 19756
let _ = { let assigned = JS_ConcatString(ctx, op1_4, op2); *(sp).offset((((2 as i32)).wrapping_neg()) as isize) = assigned; assigned };
// C line 19757
let _ = { let old = sp; sp = (sp).offset(-1); old };
// C line 19758
if (JS_IsException(*(sp).offset((((1 as i32)).wrapping_neg()) as isize))) != 0 {
vm_block = 259; continue 'vm_dispatch;
} else {
vm_block = 285; continue 'vm_dispatch;
}
} else {
{

// C line ? labels: add_slow_case
let _ = { let assigned = pc; (*(sf)).cur_pc = assigned; assigned };
// C line 19763
if (js_add_slow(ctx, sp)) != 0 {
vm_block = 259; continue 'vm_dispatch;
} else {
// C line 19765
let _ = { let old = sp; sp = (sp).offset(-1); old };
vm_block = 285; continue 'vm_dispatch;
}
}
}
}
}
}

// C line 19713
OP_copy_data_properties => {
let mut mask: i32;
let _ = { let assigned = ((*({ let old = pc; pc = (pc).offset(1); old })) as i32); mask = assigned; assigned };
// C line 19714
let _ = { let assigned = pc; (*(sf)).cur_pc = assigned; assigned };
// C line 19715
if (JS_CopyDataProperties(ctx, *(sp).offset(((((1 as i32)).wrapping_neg()).wrapping_sub(((mask) & ((3 as i32))))) as isize), *(sp).offset(((((1 as i32)).wrapping_neg()).wrapping_sub((((mask).wrapping_shr(((2 as i32)) as u32)) & ((7 as i32))))) as isize), *(sp).offset(((((1 as i32)).wrapping_neg()).wrapping_sub((((mask).wrapping_shr(((5 as i32)) as u32)) & ((7 as i32))))) as isize), (0 as i32))) != 0 {
vm_block = 259; continue 'vm_dispatch;
} else {
vm_block = 285; continue 'vm_dispatch;
}
}

// C line 19698
OP_append => {
let _ = { let assigned = pc; (*(sf)).cur_pc = assigned; assigned };
// C line 19699
if (js_append_enumerate(ctx, sp)) != 0 {
vm_block = 259; continue 'vm_dispatch;
} else {
// C line 19701
let _ = JS_FreeValue(ctx, *({ sp = (sp).offset(-1); sp }));
// C line 19703
vm_block = 285; continue 'vm_dispatch;
}
}

// C line 19688
OP_define_array_el => {
let mut ret_16: i32;
let _ = { let assigned = JS_DefinePropertyValueValue(ctx, *(sp).offset((((3 as i32)).wrapping_neg()) as isize), JS_DupValue(ctx, *(sp).offset((((2 as i32)).wrapping_neg()) as isize)), *(sp).offset((((1 as i32)).wrapping_neg()) as isize), ((((((((1 as i32)).wrapping_shl(((0 as i32)) as u32)) | (((1 as i32)).wrapping_shl(((1 as i32)) as u32)))) | (((1 as i32)).wrapping_shl(((2 as i32)) as u32)))) | (((1 as i32)).wrapping_shl(((14 as i32)) as u32)))); ret_16 = assigned; assigned };
// C line 19690
let _ = { sp = (sp).offset(-(((1 as i32)) as isize)); sp };
// C line 19691
if ((((!(((!(((((ret_16) < ((0 as i32))) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 {
vm_block = 259; continue 'vm_dispatch;
} else {
vm_block = 285; continue 'vm_dispatch;
}
}

// C line 19665
OP_put_super_value => {
let mut atom_14: JSAtom;
let mut ret_15: i32;
let _ = { let assigned = pc; (*(sf)).cur_pc = assigned; assigned };
// C line 19666
if (((((((*(sp).offset((((3 as i32)).wrapping_neg()) as isize)).tag) as i32)) != ((JS_TAG_OBJECT as i32))) as i32)) != 0 {
// C line 19667
let _ = JS_ThrowTypeErrorNotAnObject(ctx);
// C line 19668
vm_block = 259; continue 'vm_dispatch;
} else {
// C line 19670
let _ = { let assigned = JS_ValueToAtom(ctx, *(sp).offset((((2 as i32)).wrapping_neg()) as isize)); atom_14 = assigned; assigned };
// C line 19671
if ((((!(((!(((((atom_14) == ((((0 as i32)) as JSAtom))) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 {
vm_block = 259; continue 'vm_dispatch;
} else {
// C line 19673
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
// C line 19680
if ((((ret_15) < ((0 as i32))) as i32)) != 0 {
vm_block = 259; continue 'vm_dispatch;
} else {
vm_block = 285; continue 'vm_dispatch;
}
}
}
}

// C line 19639
268 => {
let mut ret_14: i32;
let _ = { let assigned = JS_HasProperty(ctx, *(sp).offset((((3 as i32)).wrapping_neg()) as isize), atom_13); ret_14 = assigned; assigned };
// C line 19640
if ((((!(((!(((((ret_14) <= ((0 as i32))) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 {
// C line 19641
if ((((!(((!(((((ret_14) < ((0 as i32))) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 {
// C line 19642
let _ = JS_FreeAtom(ctx, atom_13);
// C line 19643
vm_block = 259; continue 'vm_dispatch;
} else {
// C line 19645
if (is_strict_mode(ctx)) != 0 {
// C line 19646
let _ = JS_ThrowReferenceErrorNotDefined(ctx, atom_13);
// C line 19647
let _ = JS_FreeAtom(ctx, atom_13);
// C line 19648
vm_block = 259; continue 'vm_dispatch;
} else {
{

// C line 19651
let _ = { let assigned = JS_SetPropertyInternal(ctx, *(sp).offset((((3 as i32)).wrapping_neg()) as isize), atom_13, *(sp).offset((((1 as i32)).wrapping_neg()) as isize), *(sp).offset((((3 as i32)).wrapping_neg()) as isize), ((1 as i32)).wrapping_shl(((15 as i32)) as u32)); ret_14 = assigned; assigned };
// C line 19652
let _ = JS_FreeAtom(ctx, atom_13);
// C line 19653
let _ = JS_FreeValue(ctx, *(sp).offset((((2 as i32)).wrapping_neg()) as isize));
// C line 19654
let _ = JS_FreeValue(ctx, *(sp).offset((((3 as i32)).wrapping_neg()) as isize));
// C line 19655
let _ = { sp = (sp).offset(-(((3 as i32)) as isize)); sp };
// C line 19656
if ((((!(((!(((((ret_14) < ((0 as i32))) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 {
vm_block = 259; continue 'vm_dispatch;
} else {
vm_block = 285; continue 'vm_dispatch;
}
}
}
}
} else {
{

// C line 19651
let _ = { let assigned = JS_SetPropertyInternal(ctx, *(sp).offset((((3 as i32)).wrapping_neg()) as isize), atom_13, *(sp).offset((((1 as i32)).wrapping_neg()) as isize), *(sp).offset((((3 as i32)).wrapping_neg()) as isize), ((1 as i32)).wrapping_shl(((15 as i32)) as u32)); ret_14 = assigned; assigned };
// C line 19652
let _ = JS_FreeAtom(ctx, atom_13);
// C line 19653
let _ = JS_FreeValue(ctx, *(sp).offset((((2 as i32)).wrapping_neg()) as isize));
// C line 19654
let _ = JS_FreeValue(ctx, *(sp).offset((((3 as i32)).wrapping_neg()) as isize));
// C line 19655
let _ = { sp = (sp).offset(-(((3 as i32)) as isize)); sp };
// C line 19656
if ((((!(((!(((((ret_14) < ((0 as i32))) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 {
vm_block = 259; continue 'vm_dispatch;
} else {
vm_block = 285; continue 'vm_dispatch;
}
}
}
}

// C line 19626
OP_put_ref_value => {
let _ = { let assigned = pc; (*(sf)).cur_pc = assigned; assigned };
// C line 19627
let _ = { let assigned = JS_ValueToAtom(ctx, *(sp).offset((((2 as i32)).wrapping_neg()) as isize)); atom_13 = assigned; assigned };
// C line 19628
if ((((!(((!(((((atom_13) == ((((0 as i32)) as JSAtom))) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 {
vm_block = 259; continue 'vm_dispatch;
} else {
// C line 19630
if ((((!(((!((JS_IsUndefined(*(sp).offset((((3 as i32)).wrapping_neg()) as isize))) != 0) as i32)) != 0) as i32)) as i64)) != 0 {
// C line 19631
if (is_strict_mode(ctx)) != 0 {
// C line 19632
let _ = JS_ThrowReferenceErrorNotDefined(ctx, atom_13);
// C line 19633
let _ = JS_FreeAtom(ctx, atom_13);
// C line 19634
vm_block = 259; continue 'vm_dispatch;
} else {
// C line 19636
let _ = { let assigned = JS_DupValue(ctx, (*(ctx)).global_obj); *(sp).offset((((3 as i32)).wrapping_neg()) as isize) = assigned; assigned };
vm_block = 268; continue 'vm_dispatch;
}
} else {
vm_block = 268; continue 'vm_dispatch;
}
}
}

// C line 19578
OP_put_array_el => {
let mut array_len: u32;
let mut new_len: u32;
let mut idx_25: u32;
let mut p_7: *mut JSObject;
let mut ret_13: i32;
if ((((!(((!((((((((((((*(sp).offset((((3 as i32)).wrapping_neg()) as isize)).tag) as i32)) == ((JS_TAG_OBJECT as i32))) as i32)) != 0) && ((((((((*(sp).offset((((2 as i32)).wrapping_neg()) as isize)).tag) as i32)) == ((JS_TAG_INT as i32))) as i32)) != 0)) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 {
// C line 19580
let _ = { let assigned = ((((*(sp).offset((((3 as i32)).wrapping_neg()) as isize)).u).ptr) as *mut JSObject); p_7 = assigned; assigned };
// C line 19581
let _ = { let assigned = ((((((*(sp).offset((((2 as i32)).wrapping_neg()) as isize)).u).uint64) as i32)) as u32); idx_25 = assigned; assigned };
// C line 19582
if ((((!(((!((((((((*(p_7)).class_id) as i32)) != ((JS_CLASS_ARRAY as i32))) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 {
{

// C line ? labels: put_array_el_slow_path
let _ = { let assigned = pc; (*(sf)).cur_pc = assigned; assigned };
// C line 19613
let _ = { let assigned = JS_SetPropertyValue(ctx, *(sp).offset((((3 as i32)).wrapping_neg()) as isize), *(sp).offset((((2 as i32)).wrapping_neg()) as isize), *(sp).offset((((1 as i32)).wrapping_neg()) as isize), ((1 as i32)).wrapping_shl(((15 as i32)) as u32)); ret_13 = assigned; assigned };
// C line 19614
let _ = JS_FreeValue(ctx, *(sp).offset((((3 as i32)).wrapping_neg()) as isize));
// C line 19615
let _ = { sp = (sp).offset(-(((3 as i32)) as isize)); sp };
// C line 19616
if ((((!(((!(((((ret_13) < ((0 as i32))) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 {
vm_block = 259; continue 'vm_dispatch;
} else {
vm_block = 285; continue 'vm_dispatch;
}
}
} else {
// C line 19584
if ((((!(((!(((((idx_25) >= ((((*(p_7)).u).array).count)) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 {
// C line 19586
if ((((!(((!(((((((((((((idx_25) != ((((*(p_7)).u).array).count)) as i32)) != 0) || (((!(((*(p_7)).fast_array()) != 0) as i32)) != 0)) as i32)) != 0) || (((!((can_extend_fast_array(p_7)) != 0) as i32)) != 0)) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 {
{

// C line ? labels: put_array_el_slow_path
let _ = { let assigned = pc; (*(sf)).cur_pc = assigned; assigned };
// C line 19613
let _ = { let assigned = JS_SetPropertyValue(ctx, *(sp).offset((((3 as i32)).wrapping_neg()) as isize), *(sp).offset((((2 as i32)).wrapping_neg()) as isize), *(sp).offset((((1 as i32)).wrapping_neg()) as isize), ((1 as i32)).wrapping_shl(((15 as i32)) as u32)); ret_13 = assigned; assigned };
// C line 19614
let _ = JS_FreeValue(ctx, *(sp).offset((((3 as i32)).wrapping_neg()) as isize));
// C line 19615
let _ = { sp = (sp).offset(-(((3 as i32)) as isize)); sp };
// C line 19616
if ((((!(((!(((((ret_13) < ((0 as i32))) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 {
vm_block = 259; continue 'vm_dispatch;
} else {
vm_block = 285; continue 'vm_dispatch;
}
}
} else {
// C line 19591
if ((((!(((!((((((((((*((*(p_7)).prop).offset(((0 as i32)) as isize)).u).value).tag) as i32)) != ((JS_TAG_INT as i32))) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 {
{

// C line ? labels: put_array_el_slow_path
let _ = { let assigned = pc; (*(sf)).cur_pc = assigned; assigned };
// C line 19613
let _ = { let assigned = JS_SetPropertyValue(ctx, *(sp).offset((((3 as i32)).wrapping_neg()) as isize), *(sp).offset((((2 as i32)).wrapping_neg()) as isize), *(sp).offset((((1 as i32)).wrapping_neg()) as isize), ((1 as i32)).wrapping_shl(((15 as i32)) as u32)); ret_13 = assigned; assigned };
// C line 19614
let _ = JS_FreeValue(ctx, *(sp).offset((((3 as i32)).wrapping_neg()) as isize));
// C line 19615
let _ = { sp = (sp).offset(-(((3 as i32)) as isize)); sp };
// C line 19616
if ((((!(((!(((((ret_13) < ((0 as i32))) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 {
vm_block = 259; continue 'vm_dispatch;
} else {
vm_block = 285; continue 'vm_dispatch;
}
}
} else {
// C line 19594
let _ = { let assigned = (idx_25).wrapping_add((((1 as i32)) as u32)); new_len = assigned; assigned };
// C line 19595
if ((((!(((!(((((new_len) > (((((*(p_7)).u).array).u1).size)) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 {
{

// C line ? labels: put_array_el_slow_path
let _ = { let assigned = pc; (*(sf)).cur_pc = assigned; assigned };
// C line 19613
let _ = { let assigned = JS_SetPropertyValue(ctx, *(sp).offset((((3 as i32)).wrapping_neg()) as isize), *(sp).offset((((2 as i32)).wrapping_neg()) as isize), *(sp).offset((((1 as i32)).wrapping_neg()) as isize), ((1 as i32)).wrapping_shl(((15 as i32)) as u32)); ret_13 = assigned; assigned };
// C line 19614
let _ = JS_FreeValue(ctx, *(sp).offset((((3 as i32)).wrapping_neg()) as isize));
// C line 19615
let _ = { sp = (sp).offset(-(((3 as i32)) as isize)); sp };
// C line 19616
if ((((!(((!(((((ret_13) < ((0 as i32))) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 {
vm_block = 259; continue 'vm_dispatch;
} else {
vm_block = 285; continue 'vm_dispatch;
}
}
} else {
// C line 19597
let _ = { let assigned = ((((((((*((*(p_7)).prop).offset(((0 as i32)) as isize)).u).value).u).uint64) as i32)) as u32); array_len = assigned; assigned };
// C line 19598
if ((((new_len) > (array_len)) as i32)) != 0 {
// C line 19599
if ((((!(((!(((!(((((((*(get_shape_prop((*(p_7)).shape))).flags()) as i32)) & (((1 as i32)).wrapping_shl(((1 as i32)) as u32)))) != 0) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 {
{

// C line ? labels: put_array_el_slow_path
let _ = { let assigned = pc; (*(sf)).cur_pc = assigned; assigned };
// C line 19613
let _ = { let assigned = JS_SetPropertyValue(ctx, *(sp).offset((((3 as i32)).wrapping_neg()) as isize), *(sp).offset((((2 as i32)).wrapping_neg()) as isize), *(sp).offset((((1 as i32)).wrapping_neg()) as isize), ((1 as i32)).wrapping_shl(((15 as i32)) as u32)); ret_13 = assigned; assigned };
// C line 19614
let _ = JS_FreeValue(ctx, *(sp).offset((((3 as i32)).wrapping_neg()) as isize));
// C line 19615
let _ = { sp = (sp).offset(-(((3 as i32)) as isize)); sp };
// C line 19616
if ((((!(((!(((((ret_13) < ((0 as i32))) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 {
vm_block = 259; continue 'vm_dispatch;
} else {
vm_block = 285; continue 'vm_dispatch;
}
}
} else {
// C line 19601
let _ = { let assigned = JS_NewInt32(ctx, ((new_len) as i32)); ((*((*(p_7)).prop).offset(((0 as i32)) as isize)).u).value = assigned; assigned };
{

// C line 19603
let _ = { let assigned = new_len; (((*(p_7)).u).array).count = assigned; assigned };
// C line 19604
let _ = { let assigned = *(sp).offset((((1 as i32)).wrapping_neg()) as isize); *(((((*(p_7)).u).array).u).values).offset((idx_25) as isize) = assigned; assigned };
{

// C line 19608
let _ = JS_FreeValue(ctx, *(sp).offset((((3 as i32)).wrapping_neg()) as isize));
// C line 19609
let _ = { sp = (sp).offset(-(((3 as i32)) as isize)); sp };
vm_block = 285; continue 'vm_dispatch;
}
}
}
} else {
{

// C line 19603
let _ = { let assigned = new_len; (((*(p_7)).u).array).count = assigned; assigned };
// C line 19604
let _ = { let assigned = *(sp).offset((((1 as i32)).wrapping_neg()) as isize); *(((((*(p_7)).u).array).u).values).offset((idx_25) as isize) = assigned; assigned };
{

// C line 19608
let _ = JS_FreeValue(ctx, *(sp).offset((((3 as i32)).wrapping_neg()) as isize));
// C line 19609
let _ = { sp = (sp).offset(-(((3 as i32)) as isize)); sp };
vm_block = 285; continue 'vm_dispatch;
}
}
}
}
}
}
} else {
// C line 19606
let _ = set_value(ctx, core::ptr::addr_of_mut!(*(((((*(p_7)).u).array).u).values).offset((idx_25) as isize)), *(sp).offset((((1 as i32)).wrapping_neg()) as isize));
{

// C line 19608
let _ = JS_FreeValue(ctx, *(sp).offset((((3 as i32)).wrapping_neg()) as isize));
// C line 19609
let _ = { sp = (sp).offset(-(((3 as i32)) as isize)); sp };
vm_block = 285; continue 'vm_dispatch;
}
}
}
} else {
{

// C line ? labels: put_array_el_slow_path
let _ = { let assigned = pc; (*(sf)).cur_pc = assigned; assigned };
// C line 19613
let _ = { let assigned = JS_SetPropertyValue(ctx, *(sp).offset((((3 as i32)).wrapping_neg()) as isize), *(sp).offset((((2 as i32)).wrapping_neg()) as isize), *(sp).offset((((1 as i32)).wrapping_neg()) as isize), ((1 as i32)).wrapping_shl(((15 as i32)) as u32)); ret_13 = assigned; assigned };
// C line 19614
let _ = JS_FreeValue(ctx, *(sp).offset((((3 as i32)).wrapping_neg()) as isize));
// C line 19615
let _ = { sp = (sp).offset(-(((3 as i32)) as isize)); sp };
// C line 19616
if ((((!(((!(((((ret_13) < ((0 as i32))) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 {
vm_block = 259; continue 'vm_dispatch;
} else {
vm_block = 285; continue 'vm_dispatch;
}
}
}
}

// C line 19556
OP_get_super_value => {
let mut atom_12: JSAtom;
let mut val_13: JSValue;
let _ = { let assigned = pc; (*(sf)).cur_pc = assigned; assigned };
// C line 19557
let _ = { let assigned = JS_ValueToAtom(ctx, *(sp).offset((((1 as i32)).wrapping_neg()) as isize)); atom_12 = assigned; assigned };
// C line 19558
if ((((!(((!(((((atom_12) == ((((0 as i32)) as JSAtom))) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 {
vm_block = 259; continue 'vm_dispatch;
} else {
// C line 19560
let _ = { let assigned = JS_GetPropertyInternal(ctx, *(sp).offset((((2 as i32)).wrapping_neg()) as isize), atom_12, *(sp).offset((((3 as i32)).wrapping_neg()) as isize), (0 as i32)); val_13 = assigned; assigned };
// C line 19561
let _ = JS_FreeAtom(ctx, atom_12);
// C line 19562
if ((((!(((!((JS_IsException(val_13)) != 0) as i32)) != 0) as i32)) as i64)) != 0 {
vm_block = 259; continue 'vm_dispatch;
} else {
// C line 19564
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
vm_block = 285; continue 'vm_dispatch;
}
}
}

// C line 19520
OP_get_ref_value => {
let mut ret_12: i32;
let mut atom_11: JSAtom;
let mut val_12: JSValue;
let _ = { let assigned = pc; (*(sf)).cur_pc = assigned; assigned };
// C line 19521
let _ = { let assigned = JS_ValueToAtom(ctx, *(sp).offset((((1 as i32)).wrapping_neg()) as isize)); atom_11 = assigned; assigned };
// C line 19522
if ((((atom_11) == ((((0 as i32)) as JSAtom))) as i32)) != 0 {
vm_block = 259; continue 'vm_dispatch;
} else {
// C line 19524
if ((((!(((!((JS_IsUndefined(*(sp).offset((((2 as i32)).wrapping_neg()) as isize))) != 0) as i32)) != 0) as i32)) as i64)) != 0 {
// C line 19525
let _ = JS_ThrowReferenceErrorNotDefined(ctx, atom_11);
// C line 19526
let _ = JS_FreeAtom(ctx, atom_11);
// C line 19527
vm_block = 259; continue 'vm_dispatch;
} else {
// C line 19529
let _ = { let assigned = JS_HasProperty(ctx, *(sp).offset((((2 as i32)).wrapping_neg()) as isize), atom_11); ret_12 = assigned; assigned };
// C line 19530
if ((((!(((!(((((ret_12) <= ((0 as i32))) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 {
// C line 19531
if ((((ret_12) < ((0 as i32))) as i32)) != 0 {
// C line 19532
let _ = JS_FreeAtom(ctx, atom_11);
// C line 19533
vm_block = 259; continue 'vm_dispatch;
} else {
// C line 19535
if (is_strict_mode(ctx)) != 0 {
// C line 19536
let _ = JS_ThrowReferenceErrorNotDefined(ctx, atom_11);
// C line 19537
let _ = JS_FreeAtom(ctx, atom_11);
// C line 19538
vm_block = 259; continue 'vm_dispatch;
} else {
// C line 19540
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; val_12 = assigned; assigned };
{

// C line 19544
let _ = JS_FreeAtom(ctx, atom_11);
// C line 19545
if ((((!(((!((JS_IsException(val_12)) != 0) as i32)) != 0) as i32)) as i64)) != 0 {
vm_block = 259; continue 'vm_dispatch;
} else {
// C line 19547
let _ = { let assigned = val_12; *(sp).offset(((0 as i32)) as isize) = assigned; assigned };
// C line 19548
let _ = { let old = sp; sp = (sp).offset(1); old };
// C line 19550
vm_block = 285; continue 'vm_dispatch;
}
}
}
}
} else {
// C line 19542
let _ = { let assigned = JS_GetProperty(ctx, *(sp).offset((((2 as i32)).wrapping_neg()) as isize), atom_11); val_12 = assigned; assigned };
{

// C line 19544
let _ = JS_FreeAtom(ctx, atom_11);
// C line 19545
if ((((!(((!((JS_IsException(val_12)) != 0) as i32)) != 0) as i32)) as i64)) != 0 {
vm_block = 259; continue 'vm_dispatch;
} else {
// C line 19547
let _ = { let assigned = val_12; *(sp).offset(((0 as i32)) as isize) = assigned; assigned };
// C line 19548
let _ = { let old = sp; sp = (sp).offset(1); old };
// C line 19550
vm_block = 285; continue 'vm_dispatch;
}
}
}
}
}
}

// C line 19505
269 => {
let _ = { let assigned = pc; (*(sf)).cur_pc = assigned; assigned };
// C line 19506
let _ = { let assigned = JS_GetPropertyValue(ctx, *(sp).offset((((2 as i32)).wrapping_neg()) as isize), JS_DupValue(ctx, *(sp).offset((((1 as i32)).wrapping_neg()) as isize))); val_11 = assigned; assigned };
// C line 19507
if ((((!(((!((JS_IsException(val_11)) != 0) as i32)) != 0) as i32)) as i64)) != 0 {
vm_block = 259; continue 'vm_dispatch;
} else {
{

// C line 19510
let _ = { let assigned = val_11; *({ let old = sp; sp = (sp).offset(1); old }) = assigned; assigned };
// C line 19512
vm_block = 285; continue 'vm_dispatch;
}
}
}

// C line 19493
270 => {
if ((((!(((!((((((JS_IsUndefined(*(sp).offset((((2 as i32)).wrapping_neg()) as isize))) != 0) || ((JS_IsNull(*(sp).offset((((2 as i32)).wrapping_neg()) as isize))) != 0)) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 {
// C line 19494
let _ = JS_ThrowTypeError(ctx, c"value has no property".as_ptr());
// C line 19495
vm_block = 259; continue 'vm_dispatch;
} else {
// C line 19497
let _ = { let assigned = pc; (*(sf)).cur_pc = assigned; assigned };
// C line 19498
let _ = { let assigned = JS_ToPropertyKey(ctx, *(sp).offset((((1 as i32)).wrapping_neg()) as isize)); ret_val = assigned; assigned };
// C line 19499
if (JS_IsException(ret_val)) != 0 {
vm_block = 259; continue 'vm_dispatch;
} else {
// C line 19501
let _ = JS_FreeValue(ctx, *(sp).offset((((1 as i32)).wrapping_neg()) as isize));
// C line 19502
let _ = { let assigned = ret_val; *(sp).offset((((1 as i32)).wrapping_neg()) as isize) = assigned; assigned };
// C line 19503
vm_block = 269; continue 'vm_dispatch;
}
}
}

// C line 19474
OP_get_array_el3 => {
let mut idx_24: u32;
let mut p_6: *mut JSObject;
if ((((!(((!((((((((((((*(sp).offset((((2 as i32)).wrapping_neg()) as isize)).tag) as i32)) == ((JS_TAG_OBJECT as i32))) as i32)) != 0) && ((((((((*(sp).offset((((1 as i32)).wrapping_neg()) as isize)).tag) as i32)) == ((JS_TAG_INT as i32))) as i32)) != 0)) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 {
// C line 19476
let _ = { let assigned = ((((*(sp).offset((((2 as i32)).wrapping_neg()) as isize)).u).ptr) as *mut JSObject); p_6 = assigned; assigned };
// C line 19477
let _ = { let assigned = ((((((*(sp).offset((((1 as i32)).wrapping_neg()) as isize)).u).uint64) as i32)) as u32); idx_24 = assigned; assigned };
// C line 19478
if ((((!(((!((((((((*(p_6)).class_id) as i32)) != ((JS_CLASS_ARRAY as i32))) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 {
{

// C line 19485 labels: get_array_el3_slow_path
vm_block = match (((*(sp).offset((((1 as i32)).wrapping_neg()) as isize)).tag) as i32) { x if x == (JS_TAG_SYMBOL as i32) => 269, x if x == (JS_TAG_STRING as i32) => 269, x if x == (JS_TAG_INT as i32) => 269, _ => 270, }; continue 'vm_dispatch;
}
} else {
// C line 19480
if ((((!(((!(((((idx_24) >= ((((*(p_6)).u).array).count)) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 {
{

// C line 19485 labels: get_array_el3_slow_path
vm_block = match (((*(sp).offset((((1 as i32)).wrapping_neg()) as isize)).tag) as i32) { x if x == (JS_TAG_SYMBOL as i32) => 269, x if x == (JS_TAG_STRING as i32) => 269, x if x == (JS_TAG_INT as i32) => 269, _ => 270, }; continue 'vm_dispatch;
}
} else {
// C line 19482
let _ = { let assigned = JS_DupValue(ctx, *(((((*(p_6)).u).array).u).values).offset((idx_24) as isize)); val_11 = assigned; assigned };
{

// C line 19510
let _ = { let assigned = val_11; *({ let old = sp; sp = (sp).offset(1); old }) = assigned; assigned };
// C line 19512
vm_block = 285; continue 'vm_dispatch;
}
}
}
} else {
{

// C line 19485 labels: get_array_el3_slow_path
vm_block = match (((*(sp).offset((((1 as i32)).wrapping_neg()) as isize)).tag) as i32) { x if x == (JS_TAG_SYMBOL as i32) => 269, x if x == (JS_TAG_STRING as i32) => 269, x if x == (JS_TAG_INT as i32) => 269, _ => 270, }; continue 'vm_dispatch;
}
}
}

// C line 19465 labels: get_array_el2_slow_path
271 => {
let _ = { let assigned = pc; (*(sf)).cur_pc = assigned; assigned };
// C line 19465
let _ = { let assigned = JS_GetPropertyValue(ctx, obj_7, prop_1); val_10 = assigned; assigned };
// C line 19465
if ((((!(((!((JS_IsException(val_10)) != 0) as i32)) != 0) as i32)) as i64)) != 0 {
// C line 19465
if ((1 as i32)) != 0 {
// C line 19465
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; *(sp).offset((((1 as i32)).wrapping_neg()) as isize) = assigned; assigned };
vm_block = 259; continue 'vm_dispatch;
} else {
// C line 19465
let _ = { let old = sp; sp = (sp).offset(-1); old };
vm_block = 259; continue 'vm_dispatch;
}
} else {
{

// C line 19465
if ((1 as i32)) != 0 {
// C line 19465
let _ = { let assigned = val_10; *(sp).offset((((1 as i32)).wrapping_neg()) as isize) = assigned; assigned };
vm_block = 285; continue 'vm_dispatch;
} else {
// C line 19465
let _ = JS_FreeValue(ctx, obj_7);
// C line 19465
let _ = { let assigned = val_10; *(sp).offset((((2 as i32)).wrapping_neg()) as isize) = assigned; assigned };
// C line 19465
let _ = { let old = sp; sp = (sp).offset(-1); old };
vm_block = 285; continue 'vm_dispatch;
}
}
}
}

// C line 19465
OP_get_array_el2 => {
let mut idx_23: u32;
let mut p_5: *mut JSObject;
let _ = { let assigned = *(sp).offset((((2 as i32)).wrapping_neg()) as isize); obj_7 = assigned; assigned };
// C line 19465
let _ = { let assigned = *(sp).offset((((1 as i32)).wrapping_neg()) as isize); prop_1 = assigned; assigned };
// C line 19465
if ((((!(((!((((((((((((obj_7).tag) as i32)) == ((JS_TAG_OBJECT as i32))) as i32)) != 0) && ((((((((prop_1).tag) as i32)) == ((JS_TAG_INT as i32))) as i32)) != 0)) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 {
// C line 19465
let _ = { let assigned = ((((obj_7).u).ptr) as *mut JSObject); p_5 = assigned; assigned };
// C line 19465
let _ = { let assigned = ((((((prop_1).u).uint64) as i32)) as u32); idx_23 = assigned; assigned };
// C line 19465
if ((((!(((!((((((((*(p_5)).class_id) as i32)) != ((JS_CLASS_ARRAY as i32))) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 {
vm_block = 271; continue 'vm_dispatch;
} else {
// C line 19465
if ((((!(((!(((((idx_23) >= ((((*(p_5)).u).array).count)) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 {
vm_block = 271; continue 'vm_dispatch;
} else {
// C line 19465
let _ = { let assigned = JS_DupValue(ctx, *(((((*(p_5)).u).array).u).values).offset((idx_23) as isize)); val_10 = assigned; assigned };
{

// C line 19465
if ((1 as i32)) != 0 {
// C line 19465
let _ = { let assigned = val_10; *(sp).offset((((1 as i32)).wrapping_neg()) as isize) = assigned; assigned };
vm_block = 285; continue 'vm_dispatch;
} else {
// C line 19465
let _ = JS_FreeValue(ctx, obj_7);
// C line 19465
let _ = { let assigned = val_10; *(sp).offset((((2 as i32)).wrapping_neg()) as isize) = assigned; assigned };
// C line 19465
let _ = { let old = sp; sp = (sp).offset(-1); old };
vm_block = 285; continue 'vm_dispatch;
}
}
}
}
} else {
vm_block = 271; continue 'vm_dispatch;
}
}

// C line 19461 labels: get_array_el_slow_path
272 => {
let _ = { let assigned = pc; (*(sf)).cur_pc = assigned; assigned };
// C line 19461
let _ = { let assigned = JS_GetPropertyValue(ctx, obj_6, prop); val_9 = assigned; assigned };
// C line 19461
if ((((!(((!((JS_IsException(val_9)) != 0) as i32)) != 0) as i32)) as i64)) != 0 {
// C line 19461
if ((0 as i32)) != 0 {
// C line 19461
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; *(sp).offset((((1 as i32)).wrapping_neg()) as isize) = assigned; assigned };
vm_block = 259; continue 'vm_dispatch;
} else {
// C line 19461
let _ = { let old = sp; sp = (sp).offset(-1); old };
vm_block = 259; continue 'vm_dispatch;
}
} else {
{

// C line 19461
if ((0 as i32)) != 0 {
// C line 19461
let _ = { let assigned = val_9; *(sp).offset((((1 as i32)).wrapping_neg()) as isize) = assigned; assigned };
vm_block = 285; continue 'vm_dispatch;
} else {
// C line 19461
let _ = JS_FreeValue(ctx, obj_6);
// C line 19461
let _ = { let assigned = val_9; *(sp).offset((((2 as i32)).wrapping_neg()) as isize) = assigned; assigned };
// C line 19461
let _ = { let old = sp; sp = (sp).offset(-1); old };
vm_block = 285; continue 'vm_dispatch;
}
}
}
}

// C line 19461
OP_get_array_el => {
let mut idx_22: u32;
let mut p_4: *mut JSObject;
let _ = { let assigned = *(sp).offset((((2 as i32)).wrapping_neg()) as isize); obj_6 = assigned; assigned };
// C line 19461
let _ = { let assigned = *(sp).offset((((1 as i32)).wrapping_neg()) as isize); prop = assigned; assigned };
// C line 19461
if ((((!(((!((((((((((((obj_6).tag) as i32)) == ((JS_TAG_OBJECT as i32))) as i32)) != 0) && ((((((((prop).tag) as i32)) == ((JS_TAG_INT as i32))) as i32)) != 0)) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 {
// C line 19461
let _ = { let assigned = ((((obj_6).u).ptr) as *mut JSObject); p_4 = assigned; assigned };
// C line 19461
let _ = { let assigned = ((((((prop).u).uint64) as i32)) as u32); idx_22 = assigned; assigned };
// C line 19461
if ((((!(((!((((((((*(p_4)).class_id) as i32)) != ((JS_CLASS_ARRAY as i32))) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 {
vm_block = 272; continue 'vm_dispatch;
} else {
// C line 19461
if ((((!(((!(((((idx_22) >= ((((*(p_4)).u).array).count)) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 {
vm_block = 272; continue 'vm_dispatch;
} else {
// C line 19461
let _ = { let assigned = JS_DupValue(ctx, *(((((*(p_4)).u).array).u).values).offset((idx_22) as isize)); val_9 = assigned; assigned };
{

// C line 19461
if ((0 as i32)) != 0 {
// C line 19461
let _ = { let assigned = val_9; *(sp).offset((((1 as i32)).wrapping_neg()) as isize) = assigned; assigned };
vm_block = 285; continue 'vm_dispatch;
} else {
// C line 19461
let _ = JS_FreeValue(ctx, obj_6);
// C line 19461
let _ = { let assigned = val_9; *(sp).offset((((2 as i32)).wrapping_neg()) as isize) = assigned; assigned };
// C line 19461
let _ = { let old = sp; sp = (sp).offset(-1); old };
vm_block = 285; continue 'vm_dispatch;
}
}
}
}
} else {
vm_block = 272; continue 'vm_dispatch;
}
}

// C line 19412
OP_define_class_computed | OP_define_class => {
let mut atom_10: JSAtom;
let mut class_flags: i32;
let _ = { let assigned = crate::cutils_header::get_u32(pc); atom_10 = assigned; assigned };
// C line 19413
let _ = { let assigned = ((*(pc).offset(((4 as i32)) as isize)) as i32); class_flags = assigned; assigned };
// C line 19414
let _ = { pc = (pc).offset((((5 as i32)) as isize)); pc };
// C line 19415
if ((((js_op_define_class(ctx, sp, atom_10, class_flags, var_refs, sf, (((opcode) == ((OP_define_class_computed as i32))) as i32))) < ((0 as i32))) as i32)) != 0 {
vm_block = 259; continue 'vm_dispatch;
} else {
vm_block = 285; continue 'vm_dispatch;
}
}

// C line 19390
273 => {
let mut ret_11: i32;
let _ = { let assigned = js_method_set_properties(ctx, *(sp).offset((((1 as i32)).wrapping_neg()) as isize), atom_9, flags_2, obj_5); ret_11 = assigned; assigned };
// C line 19391
if ((((ret_11) >= ((0 as i32))) as i32)) != 0 {
// C line 19392
let _ = { let assigned = JS_DefineProperty(ctx, obj_5, atom_9, value, getter, setter, flags_2); ret_11 = assigned; assigned };
{

// C line 19395
let _ = JS_FreeValue(ctx, *(sp).offset((((1 as i32)).wrapping_neg()) as isize));
// C line 19396
if (is_computed) != 0 {
// C line 19397
let _ = JS_FreeAtom(ctx, atom_9);
// C line 19398
let _ = JS_FreeValue(ctx, *(sp).offset((((2 as i32)).wrapping_neg()) as isize));
{

// C line 19400
let _ = { sp = (sp).offset(-((((1 as i32)).wrapping_add(is_computed)) as isize)); sp };
// C line 19401
if ((((!(((!(((((ret_11) < ((0 as i32))) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 {
vm_block = 259; continue 'vm_dispatch;
} else {
vm_block = 285; continue 'vm_dispatch;
}
}
} else {
{

// C line 19400
let _ = { sp = (sp).offset(-((((1 as i32)).wrapping_add(is_computed)) as isize)); sp };
// C line 19401
if ((((!(((!(((((ret_11) < ((0 as i32))) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 {
vm_block = 259; continue 'vm_dispatch;
} else {
vm_block = 285; continue 'vm_dispatch;
}
}
}
}
} else {
{

// C line 19395
let _ = JS_FreeValue(ctx, *(sp).offset((((1 as i32)).wrapping_neg()) as isize));
// C line 19396
if (is_computed) != 0 {
// C line 19397
let _ = JS_FreeAtom(ctx, atom_9);
// C line 19398
let _ = JS_FreeValue(ctx, *(sp).offset((((2 as i32)).wrapping_neg()) as isize));
{

// C line 19400
let _ = { sp = (sp).offset(-((((1 as i32)).wrapping_add(is_computed)) as isize)); sp };
// C line 19401
if ((((!(((!(((((ret_11) < ((0 as i32))) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 {
vm_block = 259; continue 'vm_dispatch;
} else {
vm_block = 285; continue 'vm_dispatch;
}
}
} else {
{

// C line 19400
let _ = { sp = (sp).offset(-((((1 as i32)).wrapping_add(is_computed)) as isize)); sp };
// C line 19401
if ((((!(((!(((((ret_11) < ((0 as i32))) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 {
vm_block = 259; continue 'vm_dispatch;
} else {
vm_block = 285; continue 'vm_dispatch;
}
}
}
}
}
}

// C line 19369
274 => {
let mut op_flags: i32;
let _ = { let assigned = ((*({ let old = pc; pc = (pc).offset(1); old })) as i32); op_flags = assigned; assigned };
// C line 19371
let _ = { let assigned = *(sp).offset(((((2 as i32)).wrapping_neg()).wrapping_sub(is_computed)) as isize); obj_5 = assigned; assigned };
// C line 19372
let _ = { let assigned = ((((((((1 as i32)).wrapping_shl(((8 as i32)) as u32)) | (((1 as i32)).wrapping_shl(((0 as i32)) as u32)))) | (((1 as i32)).wrapping_shl(((10 as i32)) as u32)))) | (((1 as i32)).wrapping_shl(((14 as i32)) as u32))); flags_2 = assigned; assigned };
// C line 19374
if (((op_flags) & ((4 as i32)))) != 0 {
// C line 19375
let _ = { flags_2 = ((flags_2) | (((1 as i32)).wrapping_shl(((2 as i32)) as u32))); flags_2 };
{

// C line 19376
let _ = { op_flags = ((op_flags) & ((3 as i32))); op_flags };
// C line 19377
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; value = assigned; assigned };
// C line 19378
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; getter = assigned; assigned };
// C line 19379
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; setter = assigned; assigned };
// C line 19380
if ((((op_flags) == ((0 as i32))) as i32)) != 0 {
// C line 19381
let _ = { let assigned = *(sp).offset((((1 as i32)).wrapping_neg()) as isize); value = assigned; assigned };
// C line 19382
let _ = { flags_2 = ((flags_2) | (((((((1 as i32)).wrapping_shl(((13 as i32)) as u32)) | (((1 as i32)).wrapping_shl(((9 as i32)) as u32)))) | (((1 as i32)).wrapping_shl(((1 as i32)) as u32))))); flags_2 };
vm_block = 273; continue 'vm_dispatch;
} else {
// C line 19383
if ((((op_flags) == ((1 as i32))) as i32)) != 0 {
// C line 19384
let _ = { let assigned = *(sp).offset((((1 as i32)).wrapping_neg()) as isize); getter = assigned; assigned };
// C line 19385
let _ = { flags_2 = ((flags_2) | (((1 as i32)).wrapping_shl(((11 as i32)) as u32))); flags_2 };
vm_block = 273; continue 'vm_dispatch;
} else {
// C line 19387
let _ = { let assigned = *(sp).offset((((1 as i32)).wrapping_neg()) as isize); setter = assigned; assigned };
// C line 19388
let _ = { flags_2 = ((flags_2) | (((1 as i32)).wrapping_shl(((12 as i32)) as u32))); flags_2 };
vm_block = 273; continue 'vm_dispatch;
}
}
}
} else {
{

// C line 19376
let _ = { op_flags = ((op_flags) & ((3 as i32))); op_flags };
// C line 19377
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; value = assigned; assigned };
// C line 19378
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; getter = assigned; assigned };
// C line 19379
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; setter = assigned; assigned };
// C line 19380
if ((((op_flags) == ((0 as i32))) as i32)) != 0 {
// C line 19381
let _ = { let assigned = *(sp).offset((((1 as i32)).wrapping_neg()) as isize); value = assigned; assigned };
// C line 19382
let _ = { flags_2 = ((flags_2) | (((((((1 as i32)).wrapping_shl(((13 as i32)) as u32)) | (((1 as i32)).wrapping_shl(((9 as i32)) as u32)))) | (((1 as i32)).wrapping_shl(((1 as i32)) as u32))))); flags_2 };
vm_block = 273; continue 'vm_dispatch;
} else {
// C line 19383
if ((((op_flags) == ((1 as i32))) as i32)) != 0 {
// C line 19384
let _ = { let assigned = *(sp).offset((((1 as i32)).wrapping_neg()) as isize); getter = assigned; assigned };
// C line 19385
let _ = { flags_2 = ((flags_2) | (((1 as i32)).wrapping_shl(((11 as i32)) as u32))); flags_2 };
vm_block = 273; continue 'vm_dispatch;
} else {
// C line 19387
let _ = { let assigned = *(sp).offset((((1 as i32)).wrapping_neg()) as isize); setter = assigned; assigned };
// C line 19388
let _ = { flags_2 = ((flags_2) | (((1 as i32)).wrapping_shl(((12 as i32)) as u32))); flags_2 };
vm_block = 273; continue 'vm_dispatch;
}
}
}
}
}

// C line 19359
OP_define_method_computed | OP_define_method => {
let _ = { let assigned = (((opcode) == ((OP_define_method_computed as i32))) as i32); is_computed = assigned; assigned };
// C line 19360
if (is_computed) != 0 {
// C line 19361
let _ = { let assigned = JS_ValueToAtom(ctx, *(sp).offset((((2 as i32)).wrapping_neg()) as isize)); atom_9 = assigned; assigned };
// C line 19362
if ((((!(((!(((((atom_9) == ((((0 as i32)) as JSAtom))) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 {
vm_block = 259; continue 'vm_dispatch;
} else {
// C line 19364
let _ = { opcode = (opcode).wrapping_add(((OP_define_method as i32)).wrapping_sub((OP_define_method_computed as i32))); opcode };
vm_block = 274; continue 'vm_dispatch;
}
} else {
// C line 19366
let _ = { let assigned = crate::cutils_header::get_u32(pc); atom_9 = assigned; assigned };
// C line 19367
let _ = { pc = (pc).offset((((4 as i32)) as isize)); pc };
vm_block = 274; continue 'vm_dispatch;
}
}

// C line 19344
OP_set_home_object => {
let _ = js_method_set_home_object(ctx, *(sp).offset((((1 as i32)).wrapping_neg()) as isize), *(sp).offset((((2 as i32)).wrapping_neg()) as isize));
// C line 19345
vm_block = 285; continue 'vm_dispatch;
}

// C line 19333
OP_set_proto => {
let mut proto_1: JSValue;
let _ = { let assigned = pc; (*(sf)).cur_pc = assigned; assigned };
// C line 19334
let _ = { let assigned = *(sp).offset((((1 as i32)).wrapping_neg()) as isize); proto_1 = assigned; assigned };
// C line 19335
if (((((JS_IsObject(proto_1)) != 0) || ((JS_IsNull(proto_1)) != 0)) as i32)) != 0 {
// C line 19336
if ((((JS_SetPrototypeInternal(ctx, *(sp).offset((((2 as i32)).wrapping_neg()) as isize), proto_1, (1 as i32))) < ((0 as i32))) as i32)) != 0 {
vm_block = 259; continue 'vm_dispatch;
} else {
{

// C line 19339
let _ = JS_FreeValue(ctx, proto_1);
// C line 19340
let _ = { let old = sp; sp = (sp).offset(-1); old };
// C line 19342
vm_block = 285; continue 'vm_dispatch;
}
}
} else {
{

// C line 19339
let _ = JS_FreeValue(ctx, proto_1);
// C line 19340
let _ = { let old = sp; sp = (sp).offset(-1); old };
// C line 19342
vm_block = 285; continue 'vm_dispatch;
}
}
}

// C line 19325
OP_set_name_computed => {
let mut ret_10: i32;
let _ = { let assigned = JS_DefineObjectNameComputed(ctx, *(sp).offset((((1 as i32)).wrapping_neg()) as isize), *(sp).offset((((2 as i32)).wrapping_neg()) as isize), ((1 as i32)).wrapping_shl(((0 as i32)) as u32)); ret_10 = assigned; assigned };
// C line 19326
if ((((!(((!(((((ret_10) < ((0 as i32))) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 {
vm_block = 259; continue 'vm_dispatch;
} else {
vm_block = 285; continue 'vm_dispatch;
}
}

// C line 19314
OP_set_name => {
let mut atom_8: JSAtom;
let mut ret_9: i32;
let _ = { let assigned = crate::cutils_header::get_u32(pc); atom_8 = assigned; assigned };
// C line 19315
let _ = { pc = (pc).offset((((4 as i32)) as isize)); pc };
// C line 19317
let _ = { let assigned = JS_DefineObjectName(ctx, *(sp).offset((((1 as i32)).wrapping_neg()) as isize), atom_8, ((1 as i32)).wrapping_shl(((0 as i32)) as u32)); ret_9 = assigned; assigned };
// C line 19318
if ((((!(((!(((((ret_9) < ((0 as i32))) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 {
vm_block = 259; continue 'vm_dispatch;
} else {
vm_block = 285; continue 'vm_dispatch;
}
}

// C line 19299
OP_define_field => {
let mut atom_7: JSAtom;
let mut ret_8: i32;
let _ = { let assigned = crate::cutils_header::get_u32(pc); atom_7 = assigned; assigned };
// C line 19300
let _ = { pc = (pc).offset((((4 as i32)) as isize)); pc };
// C line 19302
let _ = { let assigned = JS_DefinePropertyValue(ctx, *(sp).offset((((2 as i32)).wrapping_neg()) as isize), atom_7, *(sp).offset((((1 as i32)).wrapping_neg()) as isize), ((((((((1 as i32)).wrapping_shl(((0 as i32)) as u32)) | (((1 as i32)).wrapping_shl(((1 as i32)) as u32)))) | (((1 as i32)).wrapping_shl(((2 as i32)) as u32)))) | (((1 as i32)).wrapping_shl(((14 as i32)) as u32)))); ret_8 = assigned; assigned };
// C line 19304
let _ = { let old = sp; sp = (sp).offset(-1); old };
// C line 19305
if ((((!(((!(((((ret_8) < ((0 as i32))) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 {
vm_block = 259; continue 'vm_dispatch;
} else {
vm_block = 285; continue 'vm_dispatch;
}
}

// C line 19287
OP_define_private_field => {
let mut ret_7: i32;
let _ = { let assigned = JS_DefinePrivateField(ctx, *(sp).offset((((3 as i32)).wrapping_neg()) as isize), *(sp).offset((((2 as i32)).wrapping_neg()) as isize), *(sp).offset((((1 as i32)).wrapping_neg()) as isize)); ret_7 = assigned; assigned };
// C line 19288
let _ = JS_FreeValue(ctx, *(sp).offset((((2 as i32)).wrapping_neg()) as isize));
// C line 19289
let _ = { sp = (sp).offset(-(((2 as i32)) as isize)); sp };
// C line 19290
if ((((!(((!(((((ret_7) < ((0 as i32))) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 {
vm_block = 259; continue 'vm_dispatch;
} else {
vm_block = 285; continue 'vm_dispatch;
}
}

// C line 19275
OP_put_private_field => {
let mut ret_6: i32;
let _ = { let assigned = JS_SetPrivateField(ctx, *(sp).offset((((3 as i32)).wrapping_neg()) as isize), *(sp).offset((((1 as i32)).wrapping_neg()) as isize), *(sp).offset((((2 as i32)).wrapping_neg()) as isize)); ret_6 = assigned; assigned };
// C line 19276
let _ = JS_FreeValue(ctx, *(sp).offset((((3 as i32)).wrapping_neg()) as isize));
// C line 19277
let _ = JS_FreeValue(ctx, *(sp).offset((((1 as i32)).wrapping_neg()) as isize));
// C line 19278
let _ = { sp = (sp).offset(-(((3 as i32)) as isize)); sp };
// C line 19279
if ((((!(((!(((((ret_6) < ((0 as i32))) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 {
vm_block = 259; continue 'vm_dispatch;
} else {
vm_block = 285; continue 'vm_dispatch;
}
}

// C line 19262
OP_get_private_field => {
let mut val_8: JSValue;
let _ = { let assigned = JS_GetPrivateField(ctx, *(sp).offset((((2 as i32)).wrapping_neg()) as isize), *(sp).offset((((1 as i32)).wrapping_neg()) as isize)); val_8 = assigned; assigned };
// C line 19263
let _ = JS_FreeValue(ctx, *(sp).offset((((1 as i32)).wrapping_neg()) as isize));
// C line 19264
let _ = JS_FreeValue(ctx, *(sp).offset((((2 as i32)).wrapping_neg()) as isize));
// C line 19265
let _ = { let assigned = val_8; *(sp).offset((((2 as i32)).wrapping_neg()) as isize) = assigned; assigned };
// C line 19266
let _ = { let old = sp; sp = (sp).offset(-1); old };
// C line 19267
if ((((!(((!((JS_IsException(val_8)) != 0) as i32)) != 0) as i32)) as i64)) != 0 {
vm_block = 259; continue 'vm_dispatch;
} else {
vm_block = 285; continue 'vm_dispatch;
}
}

// C line 19249
OP_private_symbol => {
let mut val_7: JSValue;
let mut atom_6: JSAtom;
let _ = { let assigned = crate::cutils_header::get_u32(pc); atom_6 = assigned; assigned };
// C line 19250
let _ = { pc = (pc).offset((((4 as i32)) as isize)); pc };
// C line 19251
let _ = { let assigned = JS_NewSymbolFromAtom(ctx, atom_6, ((JS_ATOM_TYPE_PRIVATE as i32)) as u32); val_7 = assigned; assigned };
// C line 19252
if (JS_IsException(val_7)) != 0 {
vm_block = 259; continue 'vm_dispatch;
} else {
// C line 19254
let _ = { let assigned = val_7; *({ let old = sp; sp = (sp).offset(1); old }) = assigned; assigned };
// C line 19256
vm_block = 285; continue 'vm_dispatch;
}
}

// C line 19212
OP_put_field => {
let mut prs_2: *mut JSShapeProperty;
let mut p_3: *mut JSObject;
let mut atom_5: JSAtom;
let mut obj_4: JSValue;
let mut ret_5: i32;
let _ = { let assigned = crate::cutils_header::get_u32(pc); atom_5 = assigned; assigned };
// C line 19213
let _ = { pc = (pc).offset((((4 as i32)) as isize)); pc };
// C line 19215
let _ = { let assigned = *(sp).offset((((2 as i32)).wrapping_neg()) as isize); obj_4 = assigned; assigned };
// C line 19216
if ((((!(((!((((((((obj_4).tag) as i32)) == ((JS_TAG_OBJECT as i32))) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 {
// C line 19217
let _ = { let assigned = ((((obj_4).u).ptr) as *mut JSObject); p_3 = assigned; assigned };
// C line 19218
let _ = { let assigned = find_own_property(core::ptr::addr_of_mut!(pr_3), p_3, atom_5); prs_2 = assigned; assigned };
// C line 19219
if ((!(!(prs_2).is_null()) as i32)) != 0 {
{

// C line ? labels: put_field_slow_path
let _ = { let assigned = pc; (*(sf)).cur_pc = assigned; assigned };
// C line 19233
let _ = { let assigned = JS_SetPropertyInternal(ctx, obj_4, atom_5, *(sp).offset((((1 as i32)).wrapping_neg()) as isize), obj_4, ((1 as i32)).wrapping_shl(((15 as i32)) as u32)); ret_5 = assigned; assigned };
// C line 19235
let _ = JS_FreeValue(ctx, obj_4);
// C line 19236
let _ = { sp = (sp).offset(-(((2 as i32)) as isize)); sp };
// C line 19237
if ((((!(((!(((((ret_5) < ((0 as i32))) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 {
vm_block = 259; continue 'vm_dispatch;
} else {
vm_block = 285; continue 'vm_dispatch;
}
}
} else {
// C line 19221
if ((((!(((!((((((((((*(prs_2)).flags()) as i32)) & (((((((3 as i32)).wrapping_shl(((4 as i32)) as u32)) | (((1 as i32)).wrapping_shl(((1 as i32)) as u32)))) | (((1 as i32)).wrapping_shl(((3 as i32)) as u32)))))) == (((1 as i32)).wrapping_shl(((1 as i32)) as u32))) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 {
// C line 19224
let _ = set_value(ctx, core::ptr::addr_of_mut!(((*(pr_3)).u).value), *(sp).offset((((1 as i32)).wrapping_neg()) as isize));
// C line 19228
let _ = JS_FreeValue(ctx, obj_4);
// C line 19229
let _ = { sp = (sp).offset(-(((2 as i32)) as isize)); sp };
vm_block = 285; continue 'vm_dispatch;
} else {
{

// C line ? labels: put_field_slow_path
let _ = { let assigned = pc; (*(sf)).cur_pc = assigned; assigned };
// C line 19233
let _ = { let assigned = JS_SetPropertyInternal(ctx, obj_4, atom_5, *(sp).offset((((1 as i32)).wrapping_neg()) as isize), obj_4, ((1 as i32)).wrapping_shl(((15 as i32)) as u32)); ret_5 = assigned; assigned };
// C line 19235
let _ = JS_FreeValue(ctx, obj_4);
// C line 19236
let _ = { sp = (sp).offset(-(((2 as i32)) as isize)); sp };
// C line 19237
if ((((!(((!(((((ret_5) < ((0 as i32))) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 {
vm_block = 259; continue 'vm_dispatch;
} else {
vm_block = 285; continue 'vm_dispatch;
}
}
}
}
} else {
{

// C line ? labels: put_field_slow_path
let _ = { let assigned = pc; (*(sf)).cur_pc = assigned; assigned };
// C line 19233
let _ = { let assigned = JS_SetPropertyInternal(ctx, obj_4, atom_5, *(sp).offset((((1 as i32)).wrapping_neg()) as isize), obj_4, ((1 as i32)).wrapping_shl(((15 as i32)) as u32)); ret_5 = assigned; assigned };
// C line 19235
let _ = JS_FreeValue(ctx, obj_4);
// C line 19236
let _ = { sp = (sp).offset(-(((2 as i32)) as isize)); sp };
// C line 19237
if ((((!(((!(((((ret_5) < ((0 as i32))) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 {
vm_block = 259; continue 'vm_dispatch;
} else {
vm_block = 285; continue 'vm_dispatch;
}
}
}
}

// C line 19194
275 => {
let mut prs_1: *mut JSShapeProperty;
let mut p_2: *mut JSObject;
let mut obj_3: JSValue;
let mut val_6: JSValue;
let _ = { let assigned = *(sp).offset((((1 as i32)).wrapping_neg()) as isize); obj_3 = assigned; assigned };
js_host_error_stack_read(ctx, obj_3, atom_4);
// C line 19194
if ((((!(((!((((((((obj_3).tag) as i32)) == ((JS_TAG_OBJECT as i32))) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 {
// C line 19194
let _ = { let assigned = ((((obj_3).u).ptr) as *mut JSObject); p_2 = assigned; assigned };
{

// C line 19194
'cfg_880: loop {
// C line 19194
let _ = { let assigned = find_own_property(core::ptr::addr_of_mut!(pr_2), p_2, atom_4); prs_1 = assigned; assigned };
// C line 19194
if !(prs_1).is_null() {
// C line 19194
if ((((!(((!(((((((*(prs_1)).flags()) as i32)) & (((3 as i32)).wrapping_shl(((4 as i32)) as u32)))) != 0) as i32)) != 0) as i32)) as i64)) != 0 {
{

// C line 19194 labels: get_field2_slow_path
let _ = { let assigned = pc; (*(sf)).cur_pc = assigned; assigned };
// C line 19194
let _ = { let assigned = JS_GetPropertyInternal(ctx, obj_3, atom_4, *(sp).offset((((1 as i32)).wrapping_neg()) as isize), (0 as i32)); val_6 = assigned; assigned };
// C line 19194
if ((((!(((!((JS_IsException(val_6)) != 0) as i32)) != 0) as i32)) as i64)) != 0 {
vm_block = 259; continue 'vm_dispatch;
} else {
{

// C line 19194
if ((1 as i32)) != 0 {
// C line 19194
let _ = { let assigned = val_6; *({ let old = sp; sp = (sp).offset(1); old }) = assigned; assigned };
vm_block = 285; continue 'vm_dispatch;
} else {
// C line 19194
let _ = JS_FreeValue(ctx, *(sp).offset((((1 as i32)).wrapping_neg()) as isize));
// C line 19194
let _ = { let assigned = val_6; *(sp).offset((((1 as i32)).wrapping_neg()) as isize) = assigned; assigned };
vm_block = 285; continue 'vm_dispatch;
}
}
}
}
} else {
// C line 19194
let _ = { let assigned = JS_DupValue(ctx, ((*(pr_2)).u).value); val_6 = assigned; assigned };
// C line 19194
{

// C line 19194
if ((1 as i32)) != 0 {
// C line 19194
let _ = { let assigned = val_6; *({ let old = sp; sp = (sp).offset(1); old }) = assigned; assigned };
vm_block = 285; continue 'vm_dispatch;
} else {
// C line 19194
let _ = JS_FreeValue(ctx, *(sp).offset((((1 as i32)).wrapping_neg()) as isize));
// C line 19194
let _ = { let assigned = val_6; *(sp).offset((((1 as i32)).wrapping_neg()) as isize) = assigned; assigned };
vm_block = 285; continue 'vm_dispatch;
}
}
}
} else {
// C line 19194
if ((((!(((!(((*(p_2)).is_exotic()) != 0) as i32)) != 0) as i32)) as i64)) != 0 {
// C line 19194
let _ = { let assigned = JSValue { u: JSValueUnion { ptr: ((p_2) as *mut c_void) }, tag: (((JS_TAG_OBJECT as i32)) as i64) }; obj_3 = assigned; assigned };
// C line 19194
{

// C line 19194 labels: get_field2_slow_path
let _ = { let assigned = pc; (*(sf)).cur_pc = assigned; assigned };
// C line 19194
let _ = { let assigned = JS_GetPropertyInternal(ctx, obj_3, atom_4, *(sp).offset((((1 as i32)).wrapping_neg()) as isize), (0 as i32)); val_6 = assigned; assigned };
// C line 19194
if ((((!(((!((JS_IsException(val_6)) != 0) as i32)) != 0) as i32)) as i64)) != 0 {
vm_block = 259; continue 'vm_dispatch;
} else {
{

// C line 19194
if ((1 as i32)) != 0 {
// C line 19194
let _ = { let assigned = val_6; *({ let old = sp; sp = (sp).offset(1); old }) = assigned; assigned };
vm_block = 285; continue 'vm_dispatch;
} else {
// C line 19194
let _ = JS_FreeValue(ctx, *(sp).offset((((1 as i32)).wrapping_neg()) as isize));
// C line 19194
let _ = { let assigned = val_6; *(sp).offset((((1 as i32)).wrapping_neg()) as isize) = assigned; assigned };
vm_block = 285; continue 'vm_dispatch;
}
}
}
}
} else {
// C line 19194
let _ = { let assigned = (*((*(p_2)).shape)).proto; p_2 = assigned; assigned };
// C line 19194
if ((!(!(p_2).is_null()) as i32)) != 0 {
// C line 19194
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; val_6 = assigned; assigned };
// C line 19194
{

// C line 19194
if ((1 as i32)) != 0 {
// C line 19194
let _ = { let assigned = val_6; *({ let old = sp; sp = (sp).offset(1); old }) = assigned; assigned };
vm_block = 285; continue 'vm_dispatch;
} else {
// C line 19194
let _ = JS_FreeValue(ctx, *(sp).offset((((1 as i32)).wrapping_neg()) as isize));
// C line 19194
let _ = { let assigned = val_6; *(sp).offset((((1 as i32)).wrapping_neg()) as isize) = assigned; assigned };
vm_block = 285; continue 'vm_dispatch;
}
}
} else {
continue 'cfg_880;
}
}
}
}
}
} else {
{

// C line 19194 labels: get_field2_slow_path
let _ = { let assigned = pc; (*(sf)).cur_pc = assigned; assigned };
// C line 19194
let _ = { let assigned = JS_GetPropertyInternal(ctx, obj_3, atom_4, *(sp).offset((((1 as i32)).wrapping_neg()) as isize), (0 as i32)); val_6 = assigned; assigned };
// C line 19194
if ((((!(((!((JS_IsException(val_6)) != 0) as i32)) != 0) as i32)) as i64)) != 0 {
vm_block = 259; continue 'vm_dispatch;
} else {
{

// C line 19194
if ((1 as i32)) != 0 {
// C line 19194
let _ = { let assigned = val_6; *({ let old = sp; sp = (sp).offset(1); old }) = assigned; assigned };
vm_block = 285; continue 'vm_dispatch;
} else {
// C line 19194
let _ = JS_FreeValue(ctx, *(sp).offset((((1 as i32)).wrapping_neg()) as isize));
// C line 19194
let _ = { let assigned = val_6; *(sp).offset((((1 as i32)).wrapping_neg()) as isize) = assigned; assigned };
vm_block = 285; continue 'vm_dispatch;
}
}
}
}
}
}

// C line 19194
OP_get_field2 => {
if ((0 as i32)) != 0 {
// C line 19194
let _ = { let assigned = (((JS_ATOM_length as i32)) as JSAtom); atom_4 = assigned; assigned };
vm_block = 275; continue 'vm_dispatch;
} else {
// C line 19194
let _ = { let assigned = crate::cutils_header::get_u32(pc); atom_4 = assigned; assigned };
// C line 19194
let _ = { pc = (pc).offset((((4 as i32)) as isize)); pc };
vm_block = 275; continue 'vm_dispatch;
}
}

// C line 19190
276 => {
let mut prs: *mut JSShapeProperty;
let mut p_1: *mut JSObject;
let mut obj_2: JSValue;
let mut val_5: JSValue;
let _ = { let assigned = *(sp).offset((((1 as i32)).wrapping_neg()) as isize); obj_2 = assigned; assigned };
js_host_error_stack_read(ctx, obj_2, atom_3);
// C line 19190
if ((((!(((!((((((((obj_2).tag) as i32)) == ((JS_TAG_OBJECT as i32))) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 {
// C line 19190
let _ = { let assigned = ((((obj_2).u).ptr) as *mut JSObject); p_1 = assigned; assigned };
{

// C line 19190
'cfg_910: loop {
// C line 19190
let _ = { let assigned = find_own_property(core::ptr::addr_of_mut!(pr_1), p_1, atom_3); prs = assigned; assigned };
// C line 19190
if !(prs).is_null() {
// C line 19190
if ((((!(((!(((((((*(prs)).flags()) as i32)) & (((3 as i32)).wrapping_shl(((4 as i32)) as u32)))) != 0) as i32)) != 0) as i32)) as i64)) != 0 {
{

// C line 19190 labels: get_field_slow_path
let _ = { let assigned = pc; (*(sf)).cur_pc = assigned; assigned };
// C line 19190
let _ = { let assigned = JS_GetPropertyInternal(ctx, obj_2, atom_3, *(sp).offset((((1 as i32)).wrapping_neg()) as isize), (0 as i32)); val_5 = assigned; assigned };
// C line 19190
if ((((!(((!((JS_IsException(val_5)) != 0) as i32)) != 0) as i32)) as i64)) != 0 {
vm_block = 259; continue 'vm_dispatch;
} else {
{

// C line 19190
if ((0 as i32)) != 0 {
// C line 19190
let _ = { let assigned = val_5; *({ let old = sp; sp = (sp).offset(1); old }) = assigned; assigned };
vm_block = 285; continue 'vm_dispatch;
} else {
// C line 19190
let _ = JS_FreeValue(ctx, *(sp).offset((((1 as i32)).wrapping_neg()) as isize));
// C line 19190
let _ = { let assigned = val_5; *(sp).offset((((1 as i32)).wrapping_neg()) as isize) = assigned; assigned };
vm_block = 285; continue 'vm_dispatch;
}
}
}
}
} else {
// C line 19190
let _ = { let assigned = JS_DupValue(ctx, ((*(pr_1)).u).value); val_5 = assigned; assigned };
// C line 19190
{

// C line 19190
if ((0 as i32)) != 0 {
// C line 19190
let _ = { let assigned = val_5; *({ let old = sp; sp = (sp).offset(1); old }) = assigned; assigned };
vm_block = 285; continue 'vm_dispatch;
} else {
// C line 19190
let _ = JS_FreeValue(ctx, *(sp).offset((((1 as i32)).wrapping_neg()) as isize));
// C line 19190
let _ = { let assigned = val_5; *(sp).offset((((1 as i32)).wrapping_neg()) as isize) = assigned; assigned };
vm_block = 285; continue 'vm_dispatch;
}
}
}
} else {
// C line 19190
if ((((!(((!(((*(p_1)).is_exotic()) != 0) as i32)) != 0) as i32)) as i64)) != 0 {
// C line 19190
let _ = { let assigned = JSValue { u: JSValueUnion { ptr: ((p_1) as *mut c_void) }, tag: (((JS_TAG_OBJECT as i32)) as i64) }; obj_2 = assigned; assigned };
// C line 19190
{

// C line 19190 labels: get_field_slow_path
let _ = { let assigned = pc; (*(sf)).cur_pc = assigned; assigned };
// C line 19190
let _ = { let assigned = JS_GetPropertyInternal(ctx, obj_2, atom_3, *(sp).offset((((1 as i32)).wrapping_neg()) as isize), (0 as i32)); val_5 = assigned; assigned };
// C line 19190
if ((((!(((!((JS_IsException(val_5)) != 0) as i32)) != 0) as i32)) as i64)) != 0 {
vm_block = 259; continue 'vm_dispatch;
} else {
{

// C line 19190
if ((0 as i32)) != 0 {
// C line 19190
let _ = { let assigned = val_5; *({ let old = sp; sp = (sp).offset(1); old }) = assigned; assigned };
vm_block = 285; continue 'vm_dispatch;
} else {
// C line 19190
let _ = JS_FreeValue(ctx, *(sp).offset((((1 as i32)).wrapping_neg()) as isize));
// C line 19190
let _ = { let assigned = val_5; *(sp).offset((((1 as i32)).wrapping_neg()) as isize) = assigned; assigned };
vm_block = 285; continue 'vm_dispatch;
}
}
}
}
} else {
// C line 19190
let _ = { let assigned = (*((*(p_1)).shape)).proto; p_1 = assigned; assigned };
// C line 19190
if ((!(!(p_1).is_null()) as i32)) != 0 {
// C line 19190
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; val_5 = assigned; assigned };
// C line 19190
{

// C line 19190
if ((0 as i32)) != 0 {
// C line 19190
let _ = { let assigned = val_5; *({ let old = sp; sp = (sp).offset(1); old }) = assigned; assigned };
vm_block = 285; continue 'vm_dispatch;
} else {
// C line 19190
let _ = JS_FreeValue(ctx, *(sp).offset((((1 as i32)).wrapping_neg()) as isize));
// C line 19190
let _ = { let assigned = val_5; *(sp).offset((((1 as i32)).wrapping_neg()) as isize) = assigned; assigned };
vm_block = 285; continue 'vm_dispatch;
}
}
} else {
continue 'cfg_910;
}
}
}
}
}
} else {
{

// C line 19190 labels: get_field_slow_path
let _ = { let assigned = pc; (*(sf)).cur_pc = assigned; assigned };
// C line 19190
let _ = { let assigned = JS_GetPropertyInternal(ctx, obj_2, atom_3, *(sp).offset((((1 as i32)).wrapping_neg()) as isize), (0 as i32)); val_5 = assigned; assigned };
// C line 19190
if ((((!(((!((JS_IsException(val_5)) != 0) as i32)) != 0) as i32)) as i64)) != 0 {
vm_block = 259; continue 'vm_dispatch;
} else {
{

// C line 19190
if ((0 as i32)) != 0 {
// C line 19190
let _ = { let assigned = val_5; *({ let old = sp; sp = (sp).offset(1); old }) = assigned; assigned };
vm_block = 285; continue 'vm_dispatch;
} else {
// C line 19190
let _ = JS_FreeValue(ctx, *(sp).offset((((1 as i32)).wrapping_neg()) as isize));
// C line 19190
let _ = { let assigned = val_5; *(sp).offset((((1 as i32)).wrapping_neg()) as isize) = assigned; assigned };
vm_block = 285; continue 'vm_dispatch;
}
}
}
}
}
}

// C line 19190
OP_get_field => {
if ((0 as i32)) != 0 {
// C line 19190
let _ = { let assigned = (((JS_ATOM_length as i32)) as JSAtom); atom_3 = assigned; assigned };
vm_block = 276; continue 'vm_dispatch;
} else {
// C line 19190
let _ = { let assigned = crate::cutils_header::get_u32(pc); atom_3 = assigned; assigned };
// C line 19190
let _ = { pc = (pc).offset((((4 as i32)) as isize)); pc };
vm_block = 276; continue 'vm_dispatch;
}
}

// C line 19123
OP_lnot => {
let mut op1_3: JSValue;
let mut res_2: i32;
let _ = { let assigned = *(sp).offset((((1 as i32)).wrapping_neg()) as isize); op1_3 = assigned; assigned };
// C line 19124
if (((((((((op1_3).tag) as i32)) as u32)) <= ((((JS_TAG_UNDEFINED as i32)) as u32))) as i32)) != 0 {
// C line 19125
let _ = { let assigned = (((((((op1_3).u).uint64) as i32)) != ((0 as i32))) as i32); res_2 = assigned; assigned };
{

// C line 19129
let _ = { let assigned = JS_NewBool(ctx, (!((res_2) != 0) as i32)); *(sp).offset((((1 as i32)).wrapping_neg()) as isize) = assigned; assigned };
// C line 19131
vm_block = 285; continue 'vm_dispatch;
}
} else {
// C line 19127
let _ = { let assigned = JS_ToBoolFree(ctx, op1_3); res_2 = assigned; assigned };
{

// C line 19129
let _ = { let assigned = JS_NewBool(ctx, (!((res_2) != 0) as i32)); *(sp).offset((((1 as i32)).wrapping_neg()) as isize) = assigned; assigned };
// C line 19131
vm_block = 285; continue 'vm_dispatch;
}
}
}

// C line 19090
OP_iterator_call => {
let mut flags_1: i32;
let mut ret_flag: i32;
let mut ret_4: JSValue;
let mut method: JSValue;
let _ = { let assigned = ((*({ let old = pc; pc = (pc).offset(1); old })) as i32); flags_1 = assigned; assigned };
// C line 19091
let _ = { let assigned = pc; (*(sf)).cur_pc = assigned; assigned };
// C line 19092
let _ = { let assigned = JS_GetProperty(ctx, *(sp).offset((((4 as i32)).wrapping_neg()) as isize), ((if (((flags_1) & ((1 as i32)))) != 0 { (JS_ATOM_throw as i32) } else { (crate::quickjs_atom::JS_ATOM_return as i32) }) as JSAtom)); method = assigned; assigned };
// C line 19094
if (JS_IsException(method)) != 0 {
vm_block = 259; continue 'vm_dispatch;
} else {
// C line 19096
if (((((JS_IsUndefined(method)) != 0) || ((JS_IsNull(method)) != 0)) as i32)) != 0 {
// C line 19097
let _ = { let assigned = (1 as i32); ret_flag = assigned; assigned };
{

// C line 19113
let _ = { let assigned = JS_NewBool(ctx, ret_flag); *(sp).offset(((0 as i32)) as isize) = assigned; assigned };
// C line 19114
let _ = { sp = (sp).offset((((1 as i32)) as isize)); sp };
// C line 19116
vm_block = 285; continue 'vm_dispatch;
}
} else {
// C line 19099
if (((flags_1) & ((2 as i32)))) != 0 {
// C line 19101
let _ = { let assigned = JS_CallFree(ctx, method, *(sp).offset((((4 as i32)).wrapping_neg()) as isize), (0 as i32), ((core::ptr::null_mut::<c_void>()) as *mut JSValue)); ret_4 = assigned; assigned };
{

// C line 19107
if (JS_IsException(ret_4)) != 0 {
vm_block = 259; continue 'vm_dispatch;
} else {
// C line 19109
let _ = JS_FreeValue(ctx, *(sp).offset((((1 as i32)).wrapping_neg()) as isize));
// C line 19110
let _ = { let assigned = ret_4; *(sp).offset((((1 as i32)).wrapping_neg()) as isize) = assigned; assigned };
// C line 19111
let _ = { let assigned = (0 as i32); ret_flag = assigned; assigned };
{

// C line 19113
let _ = { let assigned = JS_NewBool(ctx, ret_flag); *(sp).offset(((0 as i32)) as isize) = assigned; assigned };
// C line 19114
let _ = { sp = (sp).offset((((1 as i32)) as isize)); sp };
// C line 19116
vm_block = 285; continue 'vm_dispatch;
}
}
}
} else {
// C line 19104
let _ = { let assigned = JS_CallFree(ctx, method, *(sp).offset((((4 as i32)).wrapping_neg()) as isize), (1 as i32), (sp).offset(-(((1 as i32)) as isize))); ret_4 = assigned; assigned };
{

// C line 19107
if (JS_IsException(ret_4)) != 0 {
vm_block = 259; continue 'vm_dispatch;
} else {
// C line 19109
let _ = JS_FreeValue(ctx, *(sp).offset((((1 as i32)).wrapping_neg()) as isize));
// C line 19110
let _ = { let assigned = ret_4; *(sp).offset((((1 as i32)).wrapping_neg()) as isize) = assigned; assigned };
// C line 19111
let _ = { let assigned = (0 as i32); ret_flag = assigned; assigned };
{

// C line 19113
let _ = { let assigned = JS_NewBool(ctx, ret_flag); *(sp).offset(((0 as i32)) as isize) = assigned; assigned };
// C line 19114
let _ = { sp = (sp).offset((((1 as i32)) as isize)); sp };
// C line 19116
vm_block = 285; continue 'vm_dispatch;
}
}
}
}
}
}
}

// C line 19074
OP_iterator_next => {
let mut ret_3: JSValue;
let _ = { let assigned = pc; (*(sf)).cur_pc = assigned; assigned };
// C line 19075
let _ = { let assigned = JS_Call(ctx, *(sp).offset((((3 as i32)).wrapping_neg()) as isize), *(sp).offset((((4 as i32)).wrapping_neg()) as isize), (1 as i32), (sp).offset(-(((1 as i32)) as isize))); ret_3 = assigned; assigned };
// C line 19077
if (JS_IsException(ret_3)) != 0 {
vm_block = 259; continue 'vm_dispatch;
} else {
// C line 19079
let _ = JS_FreeValue(ctx, *(sp).offset((((1 as i32)).wrapping_neg()) as isize));
// C line 19080
let _ = { let assigned = ret_3; *(sp).offset((((1 as i32)).wrapping_neg()) as isize) = assigned; assigned };
// C line 19082
vm_block = 285; continue 'vm_dispatch;
}
}

// C line 19056
OP_nip_catch => {
let mut ret_val_1: JSValue;
let _ = { let assigned = *({ sp = (sp).offset(-1); sp }); ret_val_1 = assigned; assigned };
{

// C line 19057
'cfg_972: loop {
if ((((((((sp) > (stack_buf)) as i32)) != 0) && ((((((((*(sp).offset((((1 as i32)).wrapping_neg()) as isize)).tag) as i32)) != ((JS_TAG_CATCH_OFFSET as i32))) as i32)) != 0)) as i32)) != 0 {
// C line 19059
let _ = JS_FreeValue(ctx, *({ sp = (sp).offset(-1); sp }));
continue 'cfg_972;
} else {
// C line 19061
if ((((!(((!(((((sp) == (stack_buf)) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 {
// C line 19062
let _ = JS_ThrowInternalError(ctx, c"nip_catch".as_ptr());
// C line 19063
let _ = JS_FreeValue(ctx, ret_val_1);
// C line 19064
vm_block = 259; continue 'vm_dispatch;
} else {
// C line 19066
let _ = { let assigned = ret_val_1; *(sp).offset((((1 as i32)).wrapping_neg()) as isize) = assigned; assigned };
// C line 19068
vm_block = 285; continue 'vm_dispatch;
}
}
}
}
}

// C line 19041
OP_iterator_close => {
let _ = { let old = sp; sp = (sp).offset(-1); old };
// C line 19042
let _ = JS_FreeValue(ctx, *(sp).offset((((1 as i32)).wrapping_neg()) as isize));
// C line 19043
let _ = { let old = sp; sp = (sp).offset(-1); old };
// C line 19044
if ((!((JS_IsUndefined(*(sp).offset((((1 as i32)).wrapping_neg()) as isize))) != 0) as i32)) != 0 {
// C line 19045
let _ = { let assigned = pc; (*(sf)).cur_pc = assigned; assigned };
// C line 19046
if (JS_IteratorClose(ctx, *(sp).offset((((1 as i32)).wrapping_neg()) as isize), (0 as i32))) != 0 {
vm_block = 259; continue 'vm_dispatch;
} else {
// C line 19048
let _ = JS_FreeValue(ctx, *(sp).offset((((1 as i32)).wrapping_neg()) as isize));
{

// C line 19050
let _ = { let old = sp; sp = (sp).offset(-1); old };
// C line 19051
vm_block = 285; continue 'vm_dispatch;
}
}
} else {
{

// C line 19050
let _ = { let old = sp; sp = (sp).offset(-1); old };
// C line 19051
vm_block = 285; continue 'vm_dispatch;
}
}
}

// C line 19033
OP_iterator_check_object => {
if ((((!(((!(((!((JS_IsObject(*(sp).offset((((1 as i32)).wrapping_neg()) as isize))) != 0) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 {
// C line 19034
let _ = JS_ThrowTypeError(ctx, c"iterator must return an object".as_ptr());
// C line 19035
vm_block = 259; continue 'vm_dispatch;
} else {
vm_block = 285; continue 'vm_dispatch;
}
}

// C line 19027
OP_iterator_get_value_done => {
let _ = { let assigned = pc; (*(sf)).cur_pc = assigned; assigned };
// C line 19028
if (js_iterator_get_value_done(ctx, sp)) != 0 {
vm_block = 259; continue 'vm_dispatch;
} else {
// C line 19030
let _ = { sp = (sp).offset((((1 as i32)) as isize)); sp };
// C line 19031
vm_block = 285; continue 'vm_dispatch;
}
}

// C line 19020
OP_for_await_of_start => {
let _ = { let assigned = pc; (*(sf)).cur_pc = assigned; assigned };
// C line 19021
if (js_for_of_start(ctx, sp, (1 as i32))) != 0 {
vm_block = 259; continue 'vm_dispatch;
} else {
// C line 19023
let _ = { sp = (sp).offset((((1 as i32)) as isize)); sp };
// C line 19024
let _ = { let assigned = JS_NewCatchOffset(ctx, (0 as i32)); *({ let old = sp; sp = (sp).offset(1); old }) = assigned; assigned };
// C line 19025
vm_block = 285; continue 'vm_dispatch;
}
}

// C line 19014
OP_for_await_of_next => {
let _ = { let assigned = pc; (*(sf)).cur_pc = assigned; assigned };
// C line 19015
if (js_for_await_of_next(ctx, sp)) != 0 {
vm_block = 259; continue 'vm_dispatch;
} else {
// C line 19017
let _ = { let old = sp; sp = (sp).offset(1); old };
// C line 19018
vm_block = 285; continue 'vm_dispatch;
}
}

// C line 19005
OP_for_of_next => {
let mut offset: i32;
offset = (((3 as i32)).wrapping_neg()).wrapping_sub(((*(pc).offset(((0 as i32)) as isize)) as i32));
// C line 19006
let _ = { pc = (pc).offset((((1 as i32)) as isize)); pc };
// C line 19007
let _ = { let assigned = pc; (*(sf)).cur_pc = assigned; assigned };
// C line 19008
if (js_for_of_next(ctx, sp, offset)) != 0 {
vm_block = 259; continue 'vm_dispatch;
} else {
// C line 19010
let _ = { sp = (sp).offset((((2 as i32)) as isize)); sp };
// C line 19012
vm_block = 285; continue 'vm_dispatch;
}
}

// C line 18997
OP_for_of_start => {
let _ = { let assigned = pc; (*(sf)).cur_pc = assigned; assigned };
// C line 18998
if (js_for_of_start(ctx, sp, (0 as i32))) != 0 {
vm_block = 259; continue 'vm_dispatch;
} else {
// C line 19000
let _ = { sp = (sp).offset((((1 as i32)) as isize)); sp };
// C line 19001
let _ = { let assigned = JS_NewCatchOffset(ctx, (0 as i32)); *({ let old = sp; sp = (sp).offset(1); old }) = assigned; assigned };
// C line 19002
vm_block = 285; continue 'vm_dispatch;
}
}

// C line 18991
OP_for_in_next => {
let _ = { let assigned = pc; (*(sf)).cur_pc = assigned; assigned };
// C line 18992
if (js_for_in_next(ctx, sp)) != 0 {
vm_block = 259; continue 'vm_dispatch;
} else {
// C line 18994
let _ = { sp = (sp).offset((((2 as i32)) as isize)); sp };
// C line 18995
vm_block = 285; continue 'vm_dispatch;
}
}

// C line 18986
OP_for_in_start => {
let _ = { let assigned = pc; (*(sf)).cur_pc = assigned; assigned };
// C line 18987
if (js_for_in_start(ctx, sp)) != 0 {
vm_block = 259; continue 'vm_dispatch;
} else {
vm_block = 285; continue 'vm_dispatch;
}
}

// C line 18971
OP_ret => {
let mut pos: u32;
let mut op1_2: JSValue;
let _ = { let assigned = *(sp).offset((((1 as i32)).wrapping_neg()) as isize); op1_2 = assigned; assigned };
// C line 18972
if ((((!(((!((((((((op1_2).tag) as i32)) != ((JS_TAG_INT as i32))) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 {
{

// C line ? labels: ret_fail
let _ = JS_ThrowInternalError(ctx, c"invalid ret value".as_ptr());
// C line 18978
vm_block = 259; continue 'vm_dispatch;
}
} else {
// C line 18974
let _ = { let assigned = ((((((op1_2).u).uint64) as i32)) as u32); pos = assigned; assigned };
// C line 18975
if ((((!(((!(((((pos) >= ((((*(b)).byte_code_len) as u32))) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 {
{

// C line ? labels: ret_fail
let _ = JS_ThrowInternalError(ctx, c"invalid ret value".as_ptr());
// C line 18978
vm_block = 259; continue 'vm_dispatch;
}
} else {
// C line 18980
let _ = { let old = sp; sp = (sp).offset(-1); old };
// C line 18981
let _ = { let assigned = ((*(b)).byte_code_buf).offset(((pos) as isize)); pc = assigned; assigned };
// C line 18983
vm_block = 285; continue 'vm_dispatch;
}
}
}

// C line 18960
OP_gosub => {
let mut diff_1: i32;
let _ = { let assigned = ((crate::cutils_header::get_u32(pc)) as i32); diff_1 = assigned; assigned };
// C line 18962
let _ = { let assigned = JS_NewInt32(ctx, (((((pc).offset((((4 as i32)) as isize))).offset_from((*(b)).byte_code_buf) as i64)) as i32)); *(sp).offset(((0 as i32)) as isize) = assigned; assigned };
// C line 18963
let _ = { let old = sp; sp = (sp).offset(1); old };
// C line 18964
let _ = { pc = (pc).offset(((diff_1) as isize)); pc };
// C line 18966
vm_block = 285; continue 'vm_dispatch;
}

// C line 18951
OP_catch => {
let mut diff: i32;
let _ = { let assigned = ((crate::cutils_header::get_u32(pc)) as i32); diff = assigned; assigned };
// C line 18952
let _ = { let assigned = JS_NewCatchOffset(ctx, (((((pc).offset(((diff) as isize))).offset_from((*(b)).byte_code_buf) as i64)) as i32)); *(sp).offset(((0 as i32)) as isize) = assigned; assigned };
// C line 18953
let _ = { let old = sp; sp = (sp).offset(1); old };
// C line 18954
let _ = { pc = (pc).offset((((4 as i32)) as isize)); pc };
// C line 18956
vm_block = 285; continue 'vm_dispatch;
}

// C line 18890
OP_if_false => {
let mut op1_1: JSValue;
let mut res_1: i32;
let _ = { let assigned = *(sp).offset((((1 as i32)).wrapping_neg()) as isize); op1_1 = assigned; assigned };
// C line 18891
let _ = { pc = (pc).offset((((4 as i32)) as isize)); pc };
// C line 18893
if (((((((((op1_1).tag) as i32)) as u32)) <= ((((JS_TAG_UNDEFINED as i32)) as u32))) as i32)) != 0 {
// C line 18894
let _ = { let assigned = ((((op1_1).u).uint64) as i32); res_1 = assigned; assigned };
{

// C line 18898
let _ = { let old = sp; sp = (sp).offset(-1); old };
// C line 18899
if ((!((res_1) != 0) as i32)) != 0 {
// C line 18900
let _ = { pc = (pc).offset((((((crate::cutils_header::get_u32((pc).offset(-(((4 as i32)) as isize)))) as i32)).wrapping_sub((4 as i32))) as isize)); pc };
{

// C line 18902
if ((((!(((!((js_poll_interrupts_at(ctx, pc)) != 0) as i32)) != 0) as i32)) as i64)) != 0 {
vm_block = 259; continue 'vm_dispatch;
} else {
vm_block = 285; continue 'vm_dispatch;
}
}
} else {
{

// C line 18902
if ((((!(((!((js_poll_interrupts_at(ctx, pc)) != 0) as i32)) != 0) as i32)) as i64)) != 0 {
vm_block = 259; continue 'vm_dispatch;
} else {
vm_block = 285; continue 'vm_dispatch;
}
}
}
}
} else {
// C line 18896
let _ = { let assigned = JS_ToBoolFree(ctx, op1_1); res_1 = assigned; assigned };
{

// C line 18898
let _ = { let old = sp; sp = (sp).offset(-1); old };
// C line 18899
if ((!((res_1) != 0) as i32)) != 0 {
// C line 18900
let _ = { pc = (pc).offset((((((crate::cutils_header::get_u32((pc).offset(-(((4 as i32)) as isize)))) as i32)).wrapping_sub((4 as i32))) as isize)); pc };
{

// C line 18902
if ((((!(((!((js_poll_interrupts_at(ctx, pc)) != 0) as i32)) != 0) as i32)) as i64)) != 0 {
vm_block = 259; continue 'vm_dispatch;
} else {
vm_block = 285; continue 'vm_dispatch;
}
}
} else {
{

// C line 18902
if ((((!(((!((js_poll_interrupts_at(ctx, pc)) != 0) as i32)) != 0) as i32)) as i64)) != 0 {
vm_block = 259; continue 'vm_dispatch;
} else {
vm_block = 285; continue 'vm_dispatch;
}
}
}
}
}
}

// C line 18870
OP_if_true => {
let mut op1: JSValue;
let mut res: i32;
let _ = { let assigned = *(sp).offset((((1 as i32)).wrapping_neg()) as isize); op1 = assigned; assigned };
// C line 18871
let _ = { pc = (pc).offset((((4 as i32)) as isize)); pc };
// C line 18872
if (((((((((op1).tag) as i32)) as u32)) <= ((((JS_TAG_UNDEFINED as i32)) as u32))) as i32)) != 0 {
// C line 18873
let _ = { let assigned = ((((op1).u).uint64) as i32); res = assigned; assigned };
{

// C line 18877
let _ = { let old = sp; sp = (sp).offset(-1); old };
// C line 18878
if (res) != 0 {
// C line 18879
let _ = { pc = (pc).offset((((((crate::cutils_header::get_u32((pc).offset(-(((4 as i32)) as isize)))) as i32)).wrapping_sub((4 as i32))) as isize)); pc };
{

// C line 18881
if ((((!(((!((js_poll_interrupts_at(ctx, pc)) != 0) as i32)) != 0) as i32)) as i64)) != 0 {
vm_block = 259; continue 'vm_dispatch;
} else {
vm_block = 285; continue 'vm_dispatch;
}
}
} else {
{

// C line 18881
if ((((!(((!((js_poll_interrupts_at(ctx, pc)) != 0) as i32)) != 0) as i32)) as i64)) != 0 {
vm_block = 259; continue 'vm_dispatch;
} else {
vm_block = 285; continue 'vm_dispatch;
}
}
}
}
} else {
// C line 18875
let _ = { let assigned = JS_ToBoolFree(ctx, op1); res = assigned; assigned };
{

// C line 18877
let _ = { let old = sp; sp = (sp).offset(-1); old };
// C line 18878
if (res) != 0 {
// C line 18879
let _ = { pc = (pc).offset((((((crate::cutils_header::get_u32((pc).offset(-(((4 as i32)) as isize)))) as i32)).wrapping_sub((4 as i32))) as isize)); pc };
{

// C line 18881
if ((((!(((!((js_poll_interrupts_at(ctx, pc)) != 0) as i32)) != 0) as i32)) as i64)) != 0 {
vm_block = 259; continue 'vm_dispatch;
} else {
vm_block = 285; continue 'vm_dispatch;
}
}
} else {
{

// C line 18881
if ((((!(((!((js_poll_interrupts_at(ctx, pc)) != 0) as i32)) != 0) as i32)) as i64)) != 0 {
vm_block = 259; continue 'vm_dispatch;
} else {
vm_block = 285; continue 'vm_dispatch;
}
}
}
}
}
}

// C line 18849
OP_goto => {
let _ = { pc = (pc).offset(((((crate::cutils_header::get_u32(pc)) as i32)) as isize)); pc };
// C line 18850
if ((((!(((!((js_poll_interrupts_at(ctx, pc)) != 0) as i32)) != 0) as i32)) as i64)) != 0 {
vm_block = 259; continue 'vm_dispatch;
} else {
vm_block = 285; continue 'vm_dispatch;
}
}

// C line 18838
OP_make_var_ref => {
let mut atom_2: JSAtom;
let _ = { let assigned = crate::cutils_header::get_u32(pc); atom_2 = assigned; assigned };
// C line 18839
let _ = { pc = (pc).offset((((4 as i32)) as isize)); pc };
// C line 18840
let _ = { let assigned = pc; (*(sf)).cur_pc = assigned; assigned };
// C line 18842
if (JS_GetGlobalVarRef(ctx, atom_2, sp)) != 0 {
vm_block = 259; continue 'vm_dispatch;
} else {
// C line 18844
let _ = { sp = (sp).offset((((2 as i32)) as isize)); sp };
// C line 18846
vm_block = 285; continue 'vm_dispatch;
}
}

// C line 18811
OP_make_var_ref_ref | OP_make_arg_ref | OP_make_loc_ref => {
let mut idx_21: i32;
let mut atom_1: JSAtom;
let mut pr: *mut JSProperty;
let mut var_ref_1: *mut JSVarRef;
let _ = { let assigned = crate::cutils_header::get_u32(pc); atom_1 = assigned; assigned };
// C line 18812
let _ = { let assigned = ((crate::cutils_header::get_u16((pc).offset((((4 as i32)) as isize)))) as i32); idx_21 = assigned; assigned };
// C line 18813
let _ = { pc = (pc).offset((((6 as i32)) as isize)); pc };
// C line 18814
let _ = { let assigned = JS_NewObjectProto(ctx, JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_NULL as i32)) as i64) }); *({ let old = sp; sp = (sp).offset(1); old }) = assigned; assigned };
// C line 18815
if ((((!(((!((JS_IsException(*(sp).offset((((1 as i32)).wrapping_neg()) as isize))) != 0) as i32)) != 0) as i32)) as i64)) != 0 {
vm_block = 259; continue 'vm_dispatch;
} else {
// C line 18817
if ((((opcode) == ((OP_make_var_ref_ref as i32))) as i32)) != 0 {
// C line 18818
let _ = { let assigned = *(var_refs).offset((idx_21) as isize); var_ref_1 = assigned; assigned };
// C line 18819
let _ = { let old = (*(js_rc(((var_ref_1) as *mut c_void)))).ref_count; (*(js_rc(((var_ref_1) as *mut c_void)))).ref_count = ((*(js_rc(((var_ref_1) as *mut c_void)))).ref_count).wrapping_add(1); old };
{

// C line 18825
let _ = { let assigned = add_property(ctx, ((((*(sp).offset((((1 as i32)).wrapping_neg()) as isize)).u).ptr) as *mut JSObject), atom_1, ((((1 as i32)).wrapping_shl(((1 as i32)) as u32)) | (((2 as i32)).wrapping_shl(((4 as i32)) as u32)))); pr = assigned; assigned };
// C line 18827
if ((!(!(pr).is_null()) as i32)) != 0 {
// C line 18828
let _ = free_var_ref(rt, var_ref_1);
// C line 18829
vm_block = 259; continue 'vm_dispatch;
} else {
// C line 18831
let _ = { let assigned = var_ref_1; ((*(pr)).u).var_ref = assigned; assigned };
// C line 18832
let _ = { let assigned = JS_AtomToValue(ctx, atom_1); *({ let old = sp; sp = (sp).offset(1); old }) = assigned; assigned };
// C line 18834
vm_block = 285; continue 'vm_dispatch;
}
}
} else {
// C line 18821
let _ = { let assigned = get_var_ref(ctx, sf, idx_21, (((opcode) == ((OP_make_arg_ref as i32))) as i32)); var_ref_1 = assigned; assigned };
// C line 18822
if ((!(!(var_ref_1).is_null()) as i32)) != 0 {
vm_block = 259; continue 'vm_dispatch;
} else {
{

// C line 18825
let _ = { let assigned = add_property(ctx, ((((*(sp).offset((((1 as i32)).wrapping_neg()) as isize)).u).ptr) as *mut JSObject), atom_1, ((((1 as i32)).wrapping_shl(((1 as i32)) as u32)) | (((2 as i32)).wrapping_shl(((4 as i32)) as u32)))); pr = assigned; assigned };
// C line 18827
if ((!(!(pr).is_null()) as i32)) != 0 {
// C line 18828
let _ = free_var_ref(rt, var_ref_1);
// C line 18829
vm_block = 259; continue 'vm_dispatch;
} else {
// C line 18831
let _ = { let assigned = var_ref_1; ((*(pr)).u).var_ref = assigned; assigned };
// C line 18832
let _ = { let assigned = JS_AtomToValue(ctx, atom_1); *({ let old = sp; sp = (sp).offset(1); old }) = assigned; assigned };
// C line 18834
vm_block = 285; continue 'vm_dispatch;
}
}
}
}
}
}

// C line 18797
OP_close_loc => {
let mut idx_20: i32;
let _ = { let assigned = ((crate::cutils_header::get_u16(pc)) as i32); idx_20 = assigned; assigned };
// C line 18798
let _ = { pc = (pc).offset((((2 as i32)) as isize)); pc };
// C line 18799
let _ = close_lexical_var(ctx, b, sf, idx_20);
// C line 18801
vm_block = 285; continue 'vm_dispatch;
}

// C line 18784
OP_put_loc_check_init => {
let mut idx_19: i32;
let _ = { let assigned = ((crate::cutils_header::get_u16(pc)) as i32); idx_19 = assigned; assigned };
// C line 18785
let _ = { pc = (pc).offset((((2 as i32)) as isize)); pc };
// C line 18786
if ((((!(((!(((!((JS_IsUninitialized(*(var_buf).offset((idx_19) as isize))) != 0) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 {
// C line 18787
let _ = JS_ThrowReferenceError(ctx, c"'this' can be initialized only once".as_ptr());
// C line 18788
vm_block = 259; continue 'vm_dispatch;
} else {
// C line 18790
let _ = set_value(ctx, core::ptr::addr_of_mut!(*(var_buf).offset((idx_19) as isize)), *(sp).offset((((1 as i32)).wrapping_neg()) as isize));
// C line 18791
let _ = { let old = sp; sp = (sp).offset(-1); old };
// C line 18793
vm_block = 285; continue 'vm_dispatch;
}
}

// C line 18772
OP_set_loc_check => {
let mut idx_18: i32;
let _ = { let assigned = ((crate::cutils_header::get_u16(pc)) as i32); idx_18 = assigned; assigned };
// C line 18773
let _ = { pc = (pc).offset((((2 as i32)) as isize)); pc };
// C line 18774
if ((((!(((!((JS_IsUninitialized(*(var_buf).offset((idx_18) as isize))) != 0) as i32)) != 0) as i32)) as i64)) != 0 {
// C line 18775
let _ = JS_ThrowReferenceErrorUninitialized2(ctx, b, idx_18, (0 as i32));
// C line 18776
vm_block = 259; continue 'vm_dispatch;
} else {
// C line 18778
let _ = set_value(ctx, core::ptr::addr_of_mut!(*(var_buf).offset((idx_18) as isize)), JS_DupValue(ctx, *(sp).offset((((1 as i32)).wrapping_neg()) as isize)));
// C line 18780
vm_block = 285; continue 'vm_dispatch;
}
}

// C line 18759
OP_put_loc_check => {
let mut idx_17: i32;
let _ = { let assigned = ((crate::cutils_header::get_u16(pc)) as i32); idx_17 = assigned; assigned };
// C line 18760
let _ = { pc = (pc).offset((((2 as i32)) as isize)); pc };
// C line 18761
if ((((!(((!((JS_IsUninitialized(*(var_buf).offset((idx_17) as isize))) != 0) as i32)) != 0) as i32)) as i64)) != 0 {
// C line 18762
let _ = JS_ThrowReferenceErrorUninitialized2(ctx, b, idx_17, (0 as i32));
// C line 18763
vm_block = 259; continue 'vm_dispatch;
} else {
// C line 18765
let _ = set_value(ctx, core::ptr::addr_of_mut!(*(var_buf).offset((idx_17) as isize)), *(sp).offset((((1 as i32)).wrapping_neg()) as isize));
// C line 18766
let _ = { let old = sp; sp = (sp).offset(-1); old };
// C line 18768
vm_block = 285; continue 'vm_dispatch;
}
}

// C line 18746
OP_get_loc_checkthis => {
let mut idx_16: i32;
let _ = { let assigned = ((crate::cutils_header::get_u16(pc)) as i32); idx_16 = assigned; assigned };
// C line 18747
let _ = { pc = (pc).offset((((2 as i32)) as isize)); pc };
// C line 18748
if ((((!(((!((JS_IsUninitialized(*(var_buf).offset((idx_16) as isize))) != 0) as i32)) != 0) as i32)) as i64)) != 0 {
// C line 18749
let _ = JS_ThrowReferenceErrorUninitialized2(caller_ctx, b, idx_16, (0 as i32));
// C line 18750
vm_block = 259; continue 'vm_dispatch;
} else {
// C line 18752
let _ = { let assigned = JS_DupValue(ctx, *(var_buf).offset((idx_16) as isize)); *(sp).offset(((0 as i32)) as isize) = assigned; assigned };
// C line 18753
let _ = { let old = sp; sp = (sp).offset(1); old };
// C line 18755
vm_block = 285; continue 'vm_dispatch;
}
}

// C line 18733
OP_get_loc_check => {
let mut idx_15: i32;
let _ = { let assigned = ((crate::cutils_header::get_u16(pc)) as i32); idx_15 = assigned; assigned };
// C line 18734
let _ = { pc = (pc).offset((((2 as i32)) as isize)); pc };
// C line 18735
if ((((!(((!((JS_IsUninitialized(*(var_buf).offset((idx_15) as isize))) != 0) as i32)) != 0) as i32)) as i64)) != 0 {
// C line 18736
let _ = JS_ThrowReferenceErrorUninitialized2(ctx, b, idx_15, (0 as i32));
// C line 18737
vm_block = 259; continue 'vm_dispatch;
} else {
// C line 18739
let _ = { let assigned = JS_DupValue(ctx, *(var_buf).offset((idx_15) as isize)); *(sp).offset(((0 as i32)) as isize) = assigned; assigned };
// C line 18740
let _ = { let old = sp; sp = (sp).offset(1); old };
// C line 18742
vm_block = 285; continue 'vm_dispatch;
}
}

// C line 18725
OP_set_loc_uninitialized => {
let mut idx_14: i32;
let _ = { let assigned = ((crate::cutils_header::get_u16(pc)) as i32); idx_14 = assigned; assigned };
// C line 18726
let _ = { pc = (pc).offset((((2 as i32)) as isize)); pc };
// C line 18727
let _ = set_value(ctx, core::ptr::addr_of_mut!(*(var_buf).offset((idx_14) as isize)), JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNINITIALIZED as i32)) as i64) });
// C line 18729
vm_block = 285; continue 'vm_dispatch;
}

// C line 18712
OP_put_var_ref_check_init => {
let mut idx_13: i32;
let _ = { let assigned = ((crate::cutils_header::get_u16(pc)) as i32); idx_13 = assigned; assigned };
// C line 18713
let _ = { pc = (pc).offset((((2 as i32)) as isize)); pc };
// C line 18714
if ((((!(((!(((!((JS_IsUninitialized(*((*(*(var_refs).offset((idx_13) as isize))).pvalue))) != 0) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 {
// C line 18715
let _ = JS_ThrowReferenceErrorUninitialized2(ctx, b, idx_13, (1 as i32));
// C line 18716
vm_block = 259; continue 'vm_dispatch;
} else {
// C line 18718
let _ = set_value(ctx, (*(*(var_refs).offset((idx_13) as isize))).pvalue, *(sp).offset((((1 as i32)).wrapping_neg()) as isize));
// C line 18719
let _ = { let old = sp; sp = (sp).offset(-1); old };
// C line 18721
vm_block = 285; continue 'vm_dispatch;
}
}

// C line 18699
OP_put_var_ref_check => {
let mut idx_12: i32;
let _ = { let assigned = ((crate::cutils_header::get_u16(pc)) as i32); idx_12 = assigned; assigned };
// C line 18700
let _ = { pc = (pc).offset((((2 as i32)) as isize)); pc };
// C line 18701
if ((((!(((!((JS_IsUninitialized(*((*(*(var_refs).offset((idx_12) as isize))).pvalue))) != 0) as i32)) != 0) as i32)) as i64)) != 0 {
// C line 18702
let _ = JS_ThrowReferenceErrorUninitialized2(ctx, b, idx_12, (1 as i32));
// C line 18703
vm_block = 259; continue 'vm_dispatch;
} else {
// C line 18705
let _ = set_value(ctx, (*(*(var_refs).offset((idx_12) as isize))).pvalue, *(sp).offset((((1 as i32)).wrapping_neg()) as isize));
// C line 18706
let _ = { let old = sp; sp = (sp).offset(-1); old };
// C line 18708
vm_block = 285; continue 'vm_dispatch;
}
}

// C line 18685
OP_get_var_ref_check => {
let mut val_4: JSValue;
let mut idx_11: i32;
let _ = { let assigned = ((crate::cutils_header::get_u16(pc)) as i32); idx_11 = assigned; assigned };
// C line 18686
let _ = { pc = (pc).offset((((2 as i32)) as isize)); pc };
// C line 18687
let _ = { let assigned = *((*(*(var_refs).offset((idx_11) as isize))).pvalue); val_4 = assigned; assigned };
// C line 18688
if ((((!(((!((JS_IsUninitialized(val_4)) != 0) as i32)) != 0) as i32)) as i64)) != 0 {
// C line 18689
let _ = JS_ThrowReferenceErrorUninitialized2(ctx, b, idx_11, (1 as i32));
// C line 18690
vm_block = 259; continue 'vm_dispatch;
} else {
// C line 18692
let _ = { let assigned = JS_DupValue(ctx, val_4); *(sp).offset(((0 as i32)) as isize) = assigned; assigned };
// C line 18693
let _ = { let old = sp; sp = (sp).offset(1); old };
// C line 18695
vm_block = 285; continue 'vm_dispatch;
}
}

// C line 18676
OP_set_var_ref => {
let mut idx_10: i32;
let _ = { let assigned = ((crate::cutils_header::get_u16(pc)) as i32); idx_10 = assigned; assigned };
// C line 18677
let _ = { pc = (pc).offset((((2 as i32)) as isize)); pc };
// C line 18678
let _ = set_value(ctx, (*(*(var_refs).offset((idx_10) as isize))).pvalue, JS_DupValue(ctx, *(sp).offset((((1 as i32)).wrapping_neg()) as isize)));
// C line 18680
vm_block = 285; continue 'vm_dispatch;
}

// C line 18667
OP_put_var_ref => {
let mut idx_9: i32;
let _ = { let assigned = ((crate::cutils_header::get_u16(pc)) as i32); idx_9 = assigned; assigned };
// C line 18668
let _ = { pc = (pc).offset((((2 as i32)) as isize)); pc };
// C line 18669
let _ = set_value(ctx, (*(*(var_refs).offset((idx_9) as isize))).pvalue, *(sp).offset((((1 as i32)).wrapping_neg()) as isize));
// C line 18670
let _ = { let old = sp; sp = (sp).offset(-1); old };
// C line 18672
vm_block = 285; continue 'vm_dispatch;
}

// C line 18657
OP_get_var_ref => {
let mut val_3: JSValue;
let mut idx_8: i32;
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
vm_block = 285; continue 'vm_dispatch;
}

// C line 18604
OP_set_arg => {
let mut idx_7: i32;
let _ = { let assigned = ((crate::cutils_header::get_u16(pc)) as i32); idx_7 = assigned; assigned };
// C line 18605
let _ = { pc = (pc).offset((((2 as i32)) as isize)); pc };
// C line 18606
let _ = set_value(ctx, core::ptr::addr_of_mut!(*(arg_buf).offset((idx_7) as isize)), JS_DupValue(ctx, *(sp).offset((((1 as i32)).wrapping_neg()) as isize)));
// C line 18608
vm_block = 285; continue 'vm_dispatch;
}

// C line 18595
OP_put_arg => {
let mut idx_6: i32;
let _ = { let assigned = ((crate::cutils_header::get_u16(pc)) as i32); idx_6 = assigned; assigned };
// C line 18596
let _ = { pc = (pc).offset((((2 as i32)) as isize)); pc };
// C line 18597
let _ = set_value(ctx, core::ptr::addr_of_mut!(*(arg_buf).offset((idx_6) as isize)), *(sp).offset((((1 as i32)).wrapping_neg()) as isize));
// C line 18598
let _ = { let old = sp; sp = (sp).offset(-1); old };
// C line 18600
vm_block = 285; continue 'vm_dispatch;
}

// C line 18586
OP_get_arg => {
let mut idx_5: i32;
let _ = { let assigned = ((crate::cutils_header::get_u16(pc)) as i32); idx_5 = assigned; assigned };
// C line 18587
let _ = { pc = (pc).offset((((2 as i32)) as isize)); pc };
// C line 18588
let _ = { let assigned = JS_DupValue(ctx, *(arg_buf).offset((idx_5) as isize)); *(sp).offset(((0 as i32)) as isize) = assigned; assigned };
// C line 18589
let _ = { let old = sp; sp = (sp).offset(1); old };
// C line 18591
vm_block = 285; continue 'vm_dispatch;
}

// C line 18578
OP_set_loc => {
let mut idx_4: i32;
let _ = { let assigned = ((crate::cutils_header::get_u16(pc)) as i32); idx_4 = assigned; assigned };
// C line 18579
let _ = { pc = (pc).offset((((2 as i32)) as isize)); pc };
// C line 18580
let _ = set_value(ctx, core::ptr::addr_of_mut!(*(var_buf).offset((idx_4) as isize)), JS_DupValue(ctx, *(sp).offset((((1 as i32)).wrapping_neg()) as isize)));
// C line 18582
vm_block = 285; continue 'vm_dispatch;
}

// C line 18569
OP_put_loc => {
let mut idx_3: i32;
let _ = { let assigned = ((crate::cutils_header::get_u16(pc)) as i32); idx_3 = assigned; assigned };
// C line 18570
let _ = { pc = (pc).offset((((2 as i32)) as isize)); pc };
// C line 18571
let _ = set_value(ctx, core::ptr::addr_of_mut!(*(var_buf).offset((idx_3) as isize)), *(sp).offset((((1 as i32)).wrapping_neg()) as isize));
// C line 18572
let _ = { let old = sp; sp = (sp).offset(-1); old };
// C line 18574
vm_block = 285; continue 'vm_dispatch;
}

// C line 18560
OP_get_loc => {
let mut idx_2: i32;
let _ = { let assigned = ((crate::cutils_header::get_u16(pc)) as i32); idx_2 = assigned; assigned };
// C line 18561
let _ = { pc = (pc).offset((((2 as i32)) as isize)); pc };
// C line 18562
let _ = { let assigned = JS_DupValue(ctx, *(var_buf).offset((idx_2) as isize)); *(sp).offset(((0 as i32)) as isize) = assigned; assigned };
// C line 18563
let _ = { let old = sp; sp = (sp).offset(1); old };
// C line 18565
vm_block = 285; continue 'vm_dispatch;
}

// C line 18521
OP_put_var_init | OP_put_var => {
let mut cv_1: *mut JSClosureVar;
let mut var_ref: *mut JSVarRef;
let mut ret_2: i32;
let mut idx_1: i32;
let _ = { let assigned = ((crate::cutils_header::get_u16(pc)) as i32); idx_1 = assigned; assigned };
// C line 18522
let _ = { pc = (pc).offset((((2 as i32)) as isize)); pc };
// C line 18523
let _ = { let assigned = *(var_refs).offset((idx_1) as isize); var_ref = assigned; assigned };
// C line 18524
if ((((!(((!((((((JS_IsUninitialized(*((*(var_ref)).pvalue))) != 0) || (((((*(var_ref)).is_const) as i32)) != 0)) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 {
// C line 18526
cv_1 = core::ptr::addr_of_mut!(*((*(b)).closure_var).offset((idx_1) as isize));
// C line 18527
if ((*(var_ref)).is_lexical) != 0 {
// C line 18528
if ((((opcode) == ((OP_put_var_init as i32))) as i32)) != 0 {
{

// C line ? labels: put_var_ok
let _ = set_value(ctx, (*(var_ref)).pvalue, *(sp).offset((((1 as i32)).wrapping_neg()) as isize));
// C line 18553
let _ = { let old = sp; sp = (sp).offset(-1); old };
vm_block = 285; continue 'vm_dispatch;
}
} else {
// C line 18530
if (JS_IsUninitialized(*((*(var_ref)).pvalue))) != 0 {
// C line 18531
let _ = JS_ThrowReferenceErrorUninitialized(ctx, (*(cv_1)).var_name);
vm_block = 259; continue 'vm_dispatch;
} else {
// C line 18533
let _ = JS_ThrowTypeErrorReadOnly(ctx, ((1 as i32)).wrapping_shl(((14 as i32)) as u32), (*(cv_1)).var_name);
vm_block = 259; continue 'vm_dispatch;
}
}
} else {
// C line 18536
let _ = { let assigned = pc; (*(sf)).cur_pc = assigned; assigned };
// C line 18537
let _ = { let assigned = JS_HasProperty(ctx, (*(ctx)).global_obj, (*(cv_1)).var_name); ret_2 = assigned; assigned };
// C line 18538
if ((((ret_2) < ((0 as i32))) as i32)) != 0 {
vm_block = 259; continue 'vm_dispatch;
} else {
// C line 18540
if ((((((((ret_2) == ((0 as i32))) as i32)) != 0) && ((is_strict_mode(ctx)) != 0)) as i32)) != 0 {
// C line 18541
let _ = JS_ThrowReferenceErrorNotDefined(ctx, (*(cv_1)).var_name);
// C line 18542
vm_block = 259; continue 'vm_dispatch;
} else {
// C line 18544
let _ = { let assigned = JS_SetPropertyInternal(ctx, (*(ctx)).global_obj, (*(cv_1)).var_name, *(sp).offset((((1 as i32)).wrapping_neg()) as isize), (*(ctx)).global_obj, ((1 as i32)).wrapping_shl(((15 as i32)) as u32)); ret_2 = assigned; assigned };
// C line 18546
let _ = { let old = sp; sp = (sp).offset(-1); old };
// C line 18547
if ((((ret_2) < ((0 as i32))) as i32)) != 0 {
vm_block = 259; continue 'vm_dispatch;
} else {
vm_block = 285; continue 'vm_dispatch;
}
}
}
}
} else {
{

// C line ? labels: put_var_ok
let _ = set_value(ctx, (*(var_ref)).pvalue, *(sp).offset((((1 as i32)).wrapping_neg()) as isize));
// C line 18553
let _ = { let old = sp; sp = (sp).offset(-1); old };
vm_block = 285; continue 'vm_dispatch;
}
}
}

// C line 18492
OP_get_var | OP_get_var_undef => {
let mut cv: *mut JSClosureVar;
let mut val_2: JSValue;
let mut idx: i32;
let _ = { let assigned = ((crate::cutils_header::get_u16(pc)) as i32); idx = assigned; assigned };
// C line 18493
let _ = { pc = (pc).offset((((2 as i32)) as isize)); pc };
// C line 18494
let _ = { let assigned = *((*(*(var_refs).offset((idx) as isize))).pvalue); val_2 = assigned; assigned };
// C line 18495
if ((((!(((!((JS_IsUninitialized(val_2)) != 0) as i32)) != 0) as i32)) as i64)) != 0 {
// C line 18496
cv = core::ptr::addr_of_mut!(*((*(b)).closure_var).offset((idx) as isize));
// C line 18497
if ((*(cv)).is_lexical()) != 0 {
// C line 18498
let _ = JS_ThrowReferenceErrorUninitialized(ctx, (*(cv)).var_name);
// C line 18499
vm_block = 259; continue 'vm_dispatch;
} else {
// C line 18501
let _ = { let assigned = pc; (*(sf)).cur_pc = assigned; assigned };
// C line 18502
let _ = { let assigned = JS_GetPropertyInternal(ctx, (*(ctx)).global_obj, (*(cv)).var_name, (*(ctx)).global_obj, (opcode).wrapping_sub((OP_get_var_undef as i32))); *(sp).offset(((0 as i32)) as isize) = assigned; assigned };
// C line 18506
if (JS_IsException(*(sp).offset(((0 as i32)) as isize))) != 0 {
vm_block = 259; continue 'vm_dispatch;
} else {
{

// C line 18512
let _ = { let old = sp; sp = (sp).offset(1); old };
// C line 18514
vm_block = 285; continue 'vm_dispatch;
}
}
}
} else {
// C line 18510
let _ = { let assigned = JS_DupValue(ctx, val_2); *(sp).offset(((0 as i32)) as isize) = assigned; assigned };
{

// C line 18512
let _ = { let old = sp; sp = (sp).offset(1); old };
// C line 18514
vm_block = 285; continue 'vm_dispatch;
}
}
}

// C line 18476
OP_import => {
let mut val_1: JSValue;
let _ = { let assigned = pc; (*(sf)).cur_pc = assigned; assigned };
// C line 18477
let _ = { let assigned = js_dynamic_import(ctx, *(sp).offset((((2 as i32)).wrapping_neg()) as isize), *(sp).offset((((1 as i32)).wrapping_neg()) as isize)); val_1 = assigned; assigned };
// C line 18478
if (JS_IsException(val_1)) != 0 {
vm_block = 259; continue 'vm_dispatch;
} else {
// C line 18480
let _ = JS_FreeValue(ctx, *(sp).offset((((2 as i32)).wrapping_neg()) as isize));
// C line 18481
let _ = JS_FreeValue(ctx, *(sp).offset((((1 as i32)).wrapping_neg()) as isize));
// C line 18482
let _ = { let old = sp; sp = (sp).offset(-1); old };
// C line 18483
let _ = { let assigned = val_1; *(sp).offset((((1 as i32)).wrapping_neg()) as isize) = assigned; assigned };
// C line 18485
vm_block = 285; continue 'vm_dispatch;
}
}

// C line 18464
OP_get_super => {
let mut proto: JSValue;
let _ = { let assigned = pc; (*(sf)).cur_pc = assigned; assigned };
// C line 18465
let _ = { let assigned = JS_GetPrototype(ctx, *(sp).offset((((1 as i32)).wrapping_neg()) as isize)); proto = assigned; assigned };
// C line 18466
if (JS_IsException(proto)) != 0 {
vm_block = 259; continue 'vm_dispatch;
} else {
// C line 18468
let _ = JS_FreeValue(ctx, *(sp).offset((((1 as i32)).wrapping_neg()) as isize));
// C line 18469
let _ = { let assigned = proto; *(sp).offset((((1 as i32)).wrapping_neg()) as isize) = assigned; assigned };
// C line 18471
vm_block = 285; continue 'vm_dispatch;
}
}

// C line 18454
OP_regexp => {
let _ = { let assigned = JS_NewRegexp(ctx, *(sp).offset((((2 as i32)).wrapping_neg()) as isize), *(sp).offset((((1 as i32)).wrapping_neg()) as isize)); *(sp).offset((((2 as i32)).wrapping_neg()) as isize) = assigned; assigned };
// C line 18455
let _ = { let old = sp; sp = (sp).offset(-1); old };
// C line 18456
if (JS_IsException(*(sp).offset((((1 as i32)).wrapping_neg()) as isize))) != 0 {
vm_block = 259; continue 'vm_dispatch;
} else {
vm_block = 285; continue 'vm_dispatch;
}
}

// C line 18425
OP_apply_eval => {
let mut obj_1: JSValue;
let mut tab: *mut JSValue;
let mut scope_idx_1: i32;
let _ = { let assigned = (((crate::cutils_header::get_u16(pc)).wrapping_add(((((2 as i32)).wrapping_neg()) as u32))) as i32); scope_idx_1 = assigned; assigned };
// C line 18426
let _ = { pc = (pc).offset((((2 as i32)) as isize)); pc };
// C line 18427
let _ = { let assigned = pc; (*(sf)).cur_pc = assigned; assigned };
// C line 18428
let _ = { let assigned = build_arg_list(ctx, core::ptr::addr_of_mut!(len), *(sp).offset((((1 as i32)).wrapping_neg()) as isize)); tab = assigned; assigned };
// C line 18429
if ((!(!(tab).is_null()) as i32)) != 0 {
vm_block = 259; continue 'vm_dispatch;
} else {
// C line 18431
if (js_same_value(ctx, *(sp).offset((((2 as i32)).wrapping_neg()) as isize), (*(ctx)).eval_obj)) != 0 {
// C line 18432
if ((((len) >= ((((1 as i32)) as u32))) as i32)) != 0 {
// C line 18433
let _ = { let assigned = *(tab).offset(((0 as i32)) as isize); obj_1 = assigned; assigned };
{

// C line 18436
let _ = { let assigned = JS_EvalObject(ctx, JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }, obj_1, ((2 as i32)).wrapping_shl(((0 as i32)) as u32), scope_idx_1); ret_val = assigned; assigned };
{

// C line 18442
let _ = free_arg_list(ctx, tab, len);
// C line 18443
if ((((!(((!((JS_IsException(ret_val)) != 0) as i32)) != 0) as i32)) as i64)) != 0 {
vm_block = 259; continue 'vm_dispatch;
} else {
// C line 18445
let _ = JS_FreeValue(ctx, *(sp).offset((((2 as i32)).wrapping_neg()) as isize));
// C line 18446
let _ = JS_FreeValue(ctx, *(sp).offset((((1 as i32)).wrapping_neg()) as isize));
// C line 18447
let _ = { sp = (sp).offset(-(((2 as i32)) as isize)); sp };
// C line 18448
let _ = { let assigned = ret_val; *({ let old = sp; sp = (sp).offset(1); old }) = assigned; assigned };
// C line 18450
vm_block = 285; continue 'vm_dispatch;
}
}
}
} else {
// C line 18435
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; obj_1 = assigned; assigned };
{

// C line 18436
let _ = { let assigned = JS_EvalObject(ctx, JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }, obj_1, ((2 as i32)).wrapping_shl(((0 as i32)) as u32), scope_idx_1); ret_val = assigned; assigned };
{

// C line 18442
let _ = free_arg_list(ctx, tab, len);
// C line 18443
if ((((!(((!((JS_IsException(ret_val)) != 0) as i32)) != 0) as i32)) as i64)) != 0 {
vm_block = 259; continue 'vm_dispatch;
} else {
// C line 18445
let _ = JS_FreeValue(ctx, *(sp).offset((((2 as i32)).wrapping_neg()) as isize));
// C line 18446
let _ = JS_FreeValue(ctx, *(sp).offset((((1 as i32)).wrapping_neg()) as isize));
// C line 18447
let _ = { sp = (sp).offset(-(((2 as i32)) as isize)); sp };
// C line 18448
let _ = { let assigned = ret_val; *({ let old = sp; sp = (sp).offset(1); old }) = assigned; assigned };
// C line 18450
vm_block = 285; continue 'vm_dispatch;
}
}
}
}
} else {
// C line 18439
let _ = { let assigned = JS_Call(ctx, *(sp).offset((((2 as i32)).wrapping_neg()) as isize), JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }, ((len) as i32), tab); ret_val = assigned; assigned };
{

// C line 18442
let _ = free_arg_list(ctx, tab, len);
// C line 18443
if ((((!(((!((JS_IsException(ret_val)) != 0) as i32)) != 0) as i32)) as i64)) != 0 {
vm_block = 259; continue 'vm_dispatch;
} else {
// C line 18445
let _ = JS_FreeValue(ctx, *(sp).offset((((2 as i32)).wrapping_neg()) as isize));
// C line 18446
let _ = JS_FreeValue(ctx, *(sp).offset((((1 as i32)).wrapping_neg()) as isize));
// C line 18447
let _ = { sp = (sp).offset(-(((2 as i32)) as isize)); sp };
// C line 18448
let _ = { let assigned = ret_val; *({ let old = sp; sp = (sp).offset(1); old }) = assigned; assigned };
// C line 18450
vm_block = 285; continue 'vm_dispatch;
}
}
}
}
}

// C line 18393
OP_eval => {
let mut scope_idx: i32;
let mut obj: JSValue;
let _ = { let assigned = ((crate::cutils_header::get_u16(pc)) as i32); call_argc = assigned; assigned };
// C line 18394
let _ = { let assigned = (((crate::cutils_header::get_u16((pc).offset((((2 as i32)) as isize)))).wrapping_add(((((2 as i32)).wrapping_neg()) as u32))) as i32); scope_idx = assigned; assigned };
// C line 18395
let _ = { pc = (pc).offset((((4 as i32)) as isize)); pc };
// C line 18396
let _ = { let assigned = (sp).offset(-((call_argc) as isize)); call_argv = assigned; assigned };
// C line 18397
let _ = { let assigned = pc; (*(sf)).cur_pc = assigned; assigned };
// C line 18398
if (js_same_value(ctx, *(call_argv).offset((((1 as i32)).wrapping_neg()) as isize), (*(ctx)).eval_obj)) != 0 {
// C line 18399
if ((((call_argc) >= ((1 as i32))) as i32)) != 0 {
// C line 18400
let _ = { let assigned = *(call_argv).offset(((0 as i32)) as isize); obj = assigned; assigned };
{

// C line 18403
let _ = { let assigned = JS_EvalObject(ctx, JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }, obj, ((2 as i32)).wrapping_shl(((0 as i32)) as u32), scope_idx); ret_val = assigned; assigned };
{

// C line 18409
if ((((!(((!((JS_IsException(ret_val)) != 0) as i32)) != 0) as i32)) as i64)) != 0 {
vm_block = 259; continue 'vm_dispatch;
} else {
// C line 18411
let _ = { let assigned = ((1 as i32)).wrapping_neg(); i = assigned; assigned };
{

// C line 18411
'cfg_1299: loop {
if ((((i) < (call_argc)) as i32)) != 0 {
// C line 18412
let _ = JS_FreeValue(ctx, *(call_argv).offset((i) as isize));
// C line ?
let _ = { let old = i; i = (i).wrapping_add(1); old };
continue 'cfg_1299;
} else {
// C line 18413
let _ = { sp = (sp).offset(-(((call_argc).wrapping_add((1 as i32))) as isize)); sp };
// C line 18414
let _ = { let assigned = ret_val; *({ let old = sp; sp = (sp).offset(1); old }) = assigned; assigned };
// C line 18416
vm_block = 285; continue 'vm_dispatch;
}
}
}
}
}
}
} else {
// C line 18402
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; obj = assigned; assigned };
{

// C line 18403
let _ = { let assigned = JS_EvalObject(ctx, JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }, obj, ((2 as i32)).wrapping_shl(((0 as i32)) as u32), scope_idx); ret_val = assigned; assigned };
{

// C line 18409
if ((((!(((!((JS_IsException(ret_val)) != 0) as i32)) != 0) as i32)) as i64)) != 0 {
vm_block = 259; continue 'vm_dispatch;
} else {
// C line 18411
let _ = { let assigned = ((1 as i32)).wrapping_neg(); i = assigned; assigned };
{

// C line 18411
'cfg_1299: loop {
if ((((i) < (call_argc)) as i32)) != 0 {
// C line 18412
let _ = JS_FreeValue(ctx, *(call_argv).offset((i) as isize));
// C line ?
let _ = { let old = i; i = (i).wrapping_add(1); old };
continue 'cfg_1299;
} else {
// C line 18413
let _ = { sp = (sp).offset(-(((call_argc).wrapping_add((1 as i32))) as isize)); sp };
// C line 18414
let _ = { let assigned = ret_val; *({ let old = sp; sp = (sp).offset(1); old }) = assigned; assigned };
// C line 18416
vm_block = 285; continue 'vm_dispatch;
}
}
}
}
}
}
}
} else {
// C line 18406
let _ = { let assigned = JS_CallInternal(ctx, *(call_argv).offset((((1 as i32)).wrapping_neg()) as isize), JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }, JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }, call_argc, call_argv, (0 as i32)); ret_val = assigned; assigned };
{

// C line 18409
if ((((!(((!((JS_IsException(ret_val)) != 0) as i32)) != 0) as i32)) as i64)) != 0 {
vm_block = 259; continue 'vm_dispatch;
} else {
// C line 18411
let _ = { let assigned = ((1 as i32)).wrapping_neg(); i = assigned; assigned };
{

// C line 18411
'cfg_1299: loop {
if ((((i) < (call_argc)) as i32)) != 0 {
// C line 18412
let _ = JS_FreeValue(ctx, *(call_argv).offset((i) as isize));
// C line ?
let _ = { let old = i; i = (i).wrapping_add(1); old };
continue 'cfg_1299;
} else {
// C line 18413
let _ = { sp = (sp).offset(-(((call_argc).wrapping_add((1 as i32))) as isize)); sp };
// C line 18414
let _ = { let assigned = ret_val; *({ let old = sp; sp = (sp).offset(1); old }) = assigned; assigned };
// C line 18416
vm_block = 285; continue 'vm_dispatch;
}
}
}
}
}
}
}

// C line 18367
OP_throw_error => {
let mut v_type: i32;
let mut atom: JSAtom;
let _ = { let assigned = crate::cutils_header::get_u32(pc); atom = assigned; assigned };
// C line 18368
let _ = { let assigned = ((*(pc).offset(((4 as i32)) as isize)) as i32); v_type = assigned; assigned };
// C line 18369
let _ = { pc = (pc).offset((((5 as i32)) as isize)); pc };
// C line 18370
if ((((v_type) == ((0 as i32))) as i32)) != 0 {
// C line 18371
let _ = JS_ThrowTypeErrorReadOnly(ctx, ((1 as i32)).wrapping_shl(((14 as i32)) as u32), atom);
vm_block = 259; continue 'vm_dispatch;
} else {
// C line 18373
if ((((v_type) == ((1 as i32))) as i32)) != 0 {
// C line 18374
let _ = JS_ThrowSyntaxErrorVarRedeclaration(ctx, atom);
vm_block = 259; continue 'vm_dispatch;
} else {
// C line 18376
if ((((v_type) == ((2 as i32))) as i32)) != 0 {
// C line 18377
let _ = JS_ThrowReferenceErrorUninitialized(ctx, atom);
vm_block = 259; continue 'vm_dispatch;
} else {
// C line 18379
if ((((v_type) == ((3 as i32))) as i32)) != 0 {
// C line 18380
let _ = JS_ThrowReferenceError(ctx, c"unsupported reference to 'super'".as_ptr());
vm_block = 259; continue 'vm_dispatch;
} else {
// C line 18382
if ((((v_type) == ((4 as i32))) as i32)) != 0 {
// C line 18383
let _ = JS_ThrowTypeError(ctx, c"iterator does not have a throw method".as_ptr());
vm_block = 259; continue 'vm_dispatch;
} else {
// C line 18385
let _ = JS_ThrowInternalError(ctx, format_args!("invalid throw var type {}", v_type));
vm_block = 259; continue 'vm_dispatch;
}
}
}
}
}
}

// C line 18355
OP_throw => {
sp = sp.offset(-1);
let _ = js_host_vm_throw(ctx, b, sf, pc, sp, *sp);
// C line 18356
vm_block = 259; continue 'vm_dispatch;
}

// C line 18347
OP_add_brand => {
if ((((JS_AddBrand(ctx, *(sp).offset((((2 as i32)).wrapping_neg()) as isize), *(sp).offset((((1 as i32)).wrapping_neg()) as isize))) < ((0 as i32))) as i32)) != 0 {
vm_block = 259; continue 'vm_dispatch;
} else {
// C line 18349
let _ = JS_FreeValue(ctx, *(sp).offset((((2 as i32)).wrapping_neg()) as isize));
// C line 18350
let _ = JS_FreeValue(ctx, *(sp).offset((((1 as i32)).wrapping_neg()) as isize));
// C line 18351
let _ = { sp = (sp).offset(-(((2 as i32)) as isize)); sp };
// C line 18352
vm_block = 285; continue 'vm_dispatch;
}
}

// C line 18337
OP_check_brand => {
let mut ret_1: i32;
ret_1 = JS_CheckBrand(ctx, *(sp).offset((((2 as i32)).wrapping_neg()) as isize), *(sp).offset((((1 as i32)).wrapping_neg()) as isize));
// C line 18338
if ((((ret_1) < ((0 as i32))) as i32)) != 0 {
vm_block = 259; continue 'vm_dispatch;
} else {
// C line 18340
if ((!((ret_1) != 0) as i32)) != 0 {
// C line 18341
let _ = JS_ThrowTypeError(ctx, c"invalid brand on object".as_ptr());
// C line 18342
vm_block = 259; continue 'vm_dispatch;
} else {
vm_block = 285; continue 'vm_dispatch;
}
}
}

// C line 18322
OP_init_ctor => {
let mut ret: JSValue;
let mut v_super: JSValue;
let _ = { let assigned = pc; (*(sf)).cur_pc = assigned; assigned };
// C line 18323
if (JS_IsUndefined(new_target)) != 0 {
{

// C line ? labels: non_ctor_call
let _ = JS_ThrowTypeError(ctx, c"class constructors must be invoked with 'new'".as_ptr());
// C line 18316
vm_block = 259; continue 'vm_dispatch;
}
} else {
// C line 18325
let _ = { let assigned = JS_GetPrototype(ctx, func_obj); v_super = assigned; assigned };
// C line 18326
if (JS_IsException(v_super)) != 0 {
vm_block = 259; continue 'vm_dispatch;
} else {
// C line 18328
let _ = { let assigned = JS_CallConstructor2(ctx, v_super, new_target, argc, argv); ret = assigned; assigned };
// C line 18329
let _ = JS_FreeValue(ctx, v_super);
// C line 18330
if (JS_IsException(ret)) != 0 {
vm_block = 259; continue 'vm_dispatch;
} else {
// C line 18332
let _ = { let assigned = ret; *({ let old = sp; sp = (sp).offset(1); old }) = assigned; assigned };
// C line 18334
vm_block = 285; continue 'vm_dispatch;
}
}
}
}

// C line 18313
OP_check_ctor => {
if (JS_IsUndefined(new_target)) != 0 {
{

// C line ? labels: non_ctor_call
let _ = JS_ThrowTypeError(ctx, c"class constructors must be invoked with 'new'".as_ptr());
// C line 18316
vm_block = 259; continue 'vm_dispatch;
}
} else {
vm_block = 285; continue 'vm_dispatch;
}
}

// C line 18301
OP_check_ctor_return => {
if ((!((JS_IsObject(*(sp).offset((((1 as i32)).wrapping_neg()) as isize))) != 0) as i32)) != 0 {
// C line 18302
if ((!((JS_IsUndefined(*(sp).offset((((1 as i32)).wrapping_neg()) as isize))) != 0) as i32)) != 0 {
// C line 18303
let _ = JS_ThrowTypeError(caller_ctx, c"derived class constructor must return an object or undefined".as_ptr());
// C line 18304
vm_block = 259; continue 'vm_dispatch;
} else {
// C line 18306
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((1 as i32)) as u32)) as u64) }, tag: (((JS_TAG_BOOL as i32)) as i64) }; *(sp).offset(((0 as i32)) as isize) = assigned; assigned };
{

// C line 18310
let _ = { let old = sp; sp = (sp).offset(1); old };
// C line 18311
vm_block = 285; continue 'vm_dispatch;
}
}
} else {
// C line 18308
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_BOOL as i32)) as i64) }; *(sp).offset(((0 as i32)) as isize) = assigned; assigned };
{

// C line 18310
let _ = { let old = sp; sp = (sp).offset(1); old };
// C line 18311
vm_block = 285; continue 'vm_dispatch;
}
}
}

// C line 18296
OP_return_undef => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; ret_val = assigned; assigned };
// C line 18297
vm_block = 257; continue 'vm_dispatch;
}

// C line 18293
OP_return => {
let _ = { let assigned = *({ sp = (sp).offset(-1); sp }); ret_val = assigned; assigned };
// C line 18294
vm_block = 257; continue 'vm_dispatch;
}

// C line 18278
OP_apply => {
let mut magic: i32;
let _ = { let assigned = ((crate::cutils_header::get_u16(pc)) as i32); magic = assigned; assigned };
// C line 18279
let _ = { pc = (pc).offset((((2 as i32)) as isize)); pc };
// C line 18280
let _ = { let assigned = pc; (*(sf)).cur_pc = assigned; assigned };
// C line 18282
let _ = { let assigned = js_function_apply(ctx, *(sp).offset((((3 as i32)).wrapping_neg()) as isize), (2 as i32), core::ptr::addr_of_mut!(*(sp).offset((((2 as i32)).wrapping_neg()) as isize)), magic); ret_val = assigned; assigned };
// C line 18283
if ((((!(((!((JS_IsException(ret_val)) != 0) as i32)) != 0) as i32)) as i64)) != 0 {
vm_block = 259; continue 'vm_dispatch;
} else {
// C line 18285
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
vm_block = 285; continue 'vm_dispatch;
}
}

// C line 18266
OP_array_from => {
let _ = { let assigned = ((crate::cutils_header::get_u16(pc)) as i32); call_argc = assigned; assigned };
// C line 18267
let _ = { pc = (pc).offset((((2 as i32)) as isize)); pc };
// C line 18268
let _ = { let assigned = js_create_array_free(ctx, call_argc, (sp).offset(-((call_argc) as isize))); ret_val = assigned; assigned };
// C line 18269
let _ = { sp = (sp).offset(-((call_argc) as isize)); sp };
// C line 18270
if ((((!(((!((JS_IsException(ret_val)) != 0) as i32)) != 0) as i32)) as i64)) != 0 {
vm_block = 259; continue 'vm_dispatch;
} else {
// C line 18272
let _ = { let assigned = ret_val; *({ let old = sp; sp = (sp).offset(1); old }) = assigned; assigned };
// C line 18273
vm_block = 285; continue 'vm_dispatch;
}
}

// C line 18249
OP_tail_call_method | OP_call_method => {
let _ = { let assigned = ((crate::cutils_header::get_u16(pc)) as i32); call_argc = assigned; assigned };
// C line 18250
let _ = { pc = (pc).offset((((2 as i32)) as isize)); pc };
// C line 18251
let _ = { let assigned = (sp).offset(-((call_argc) as isize)); call_argv = assigned; assigned };
// C line 18252
let _ = { let assigned = pc; (*(sf)).cur_pc = assigned; assigned };
// C line 18253
let _ = { let assigned = JS_CallInternal(ctx, *(call_argv).offset((((1 as i32)).wrapping_neg()) as isize), *(call_argv).offset((((2 as i32)).wrapping_neg()) as isize), JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }, call_argc, call_argv, (0 as i32)); ret_val = assigned; assigned };
// C line 18255
if ((((!(((!((JS_IsException(ret_val)) != 0) as i32)) != 0) as i32)) as i64)) != 0 {
vm_block = 259; continue 'vm_dispatch;
} else {
// C line 18257
if ((((opcode) == ((OP_tail_call_method as i32))) as i32)) != 0 {
vm_block = 257; continue 'vm_dispatch;
} else {
// C line 18259
let _ = { let assigned = ((2 as i32)).wrapping_neg(); i = assigned; assigned };
{

// C line 18259
'cfg_1397: loop {
if ((((i) < (call_argc)) as i32)) != 0 {
// C line 18260
let _ = JS_FreeValue(ctx, *(call_argv).offset((i) as isize));
// C line ?
let _ = { let old = i; i = (i).wrapping_add(1); old };
continue 'cfg_1397;
} else {
// C line 18261
let _ = { sp = (sp).offset(-(((call_argc).wrapping_add((2 as i32))) as isize)); sp };
// C line 18262
let _ = { let assigned = ret_val; *({ let old = sp; sp = (sp).offset(1); old }) = assigned; assigned };
// C line 18264
vm_block = 285; continue 'vm_dispatch;
}
}
}
}
}
}

// C line 18231
OP_call_constructor => {
let _ = { let assigned = ((crate::cutils_header::get_u16(pc)) as i32); call_argc = assigned; assigned };
// C line 18232
let _ = { pc = (pc).offset((((2 as i32)) as isize)); pc };
// C line 18233
let _ = { let assigned = (sp).offset(-((call_argc) as isize)); call_argv = assigned; assigned };
// C line 18234
let _ = { let assigned = pc; (*(sf)).cur_pc = assigned; assigned };
// C line 18235
let _ = { let assigned = JS_CallConstructorInternal(ctx, *(call_argv).offset((((2 as i32)).wrapping_neg()) as isize), *(call_argv).offset((((1 as i32)).wrapping_neg()) as isize), call_argc, call_argv, (0 as i32)); ret_val = assigned; assigned };
// C line 18238
if ((((!(((!((JS_IsException(ret_val)) != 0) as i32)) != 0) as i32)) as i64)) != 0 {
vm_block = 259; continue 'vm_dispatch;
} else {
// C line 18240
let _ = { let assigned = ((2 as i32)).wrapping_neg(); i = assigned; assigned };
{

// C line 18240
'cfg_1413: loop {
if ((((i) < (call_argc)) as i32)) != 0 {
// C line 18241
let _ = JS_FreeValue(ctx, *(call_argv).offset((i) as isize));
// C line ?
let _ = { let old = i; i = (i).wrapping_add(1); old };
continue 'cfg_1413;
} else {
// C line 18242
let _ = { sp = (sp).offset(-(((call_argc).wrapping_add((2 as i32))) as isize)); sp };
// C line 18243
let _ = { let assigned = ret_val; *({ let old = sp; sp = (sp).offset(1); old }) = assigned; assigned };
// C line 18245
vm_block = 285; continue 'vm_dispatch;
}
}
}
}
}

// C line 18211
OP_tail_call | OP_call => {
let _ = { let assigned = ((crate::cutils_header::get_u16(pc)) as i32); call_argc = assigned; assigned };
// C line 18212
let _ = { pc = (pc).offset((((2 as i32)) as isize)); pc };
// C line 18213
// C line ? labels: has_call_argc
let _ = { let assigned = (sp).offset(-((call_argc) as isize)); call_argv = assigned; assigned };
// C line 18216
let _ = { let assigned = pc; (*(sf)).cur_pc = assigned; assigned };
// C line 18217
let _ = { let assigned = JS_CallInternal(ctx, *(call_argv).offset((((1 as i32)).wrapping_neg()) as isize), JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }, JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }, call_argc, call_argv, (0 as i32)); ret_val = assigned; assigned };
// C line 18219
if ((((!(((!((JS_IsException(ret_val)) != 0) as i32)) != 0) as i32)) as i64)) != 0 {
vm_block = 259; continue 'vm_dispatch;
} else {
// C line 18221
if ((((opcode) == ((OP_tail_call as i32))) as i32)) != 0 {
vm_block = 257; continue 'vm_dispatch;
} else {
// C line 18223
let _ = { let assigned = ((1 as i32)).wrapping_neg(); i = assigned; assigned };
{

// C line 18223
'cfg_1427: loop {
if ((((i) < (call_argc)) as i32)) != 0 {
// C line 18224
let _ = JS_FreeValue(ctx, *(call_argv).offset((i) as isize));
// C line ?
let _ = { let old = i; i = (i).wrapping_add(1); old };
continue 'cfg_1427;
} else {
// C line 18225
let _ = { sp = (sp).offset(-(((call_argc).wrapping_add((1 as i32))) as isize)); sp };
// C line 18226
let _ = { let assigned = ret_val; *({ let old = sp; sp = (sp).offset(1); old }) = assigned; assigned };
// C line 18228
vm_block = 285; continue 'vm_dispatch;
}
}
}
}
}
}

// C line 18193
OP_fclosure => {
let mut bfunc: JSValue;
bfunc = JS_DupValue(ctx, *((*(b)).cpool).offset((crate::cutils_header::get_u32(pc)) as isize));
// C line 18194
let _ = { pc = (pc).offset((((4 as i32)) as isize)); pc };
// C line 18195
let _ = { let assigned = js_closure(ctx, bfunc, var_refs, sf, (0 as i32)); *({ let old = sp; sp = (sp).offset(1); old }) = assigned; assigned };
// C line 18196
if ((((!(((!((JS_IsException(*(sp).offset((((1 as i32)).wrapping_neg()) as isize))) != 0) as i32)) != 0) as i32)) as i64)) != 0 {
vm_block = 259; continue 'vm_dispatch;
} else {
vm_block = 285; continue 'vm_dispatch;
}
}

// C line 18182
OP_swap2 => {
let mut tmp2: JSValue;
let mut tmp1: JSValue;
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
vm_block = 285; continue 'vm_dispatch;
}

// C line 18174
OP_swap => {
let mut tmp_7: JSValue;
let _ = { let assigned = *(sp).offset((((2 as i32)).wrapping_neg()) as isize); tmp_7 = assigned; assigned };
// C line 18175
let _ = { let assigned = *(sp).offset((((1 as i32)).wrapping_neg()) as isize); *(sp).offset((((2 as i32)).wrapping_neg()) as isize) = assigned; assigned };
// C line 18176
let _ = { let assigned = tmp_7; *(sp).offset((((1 as i32)).wrapping_neg()) as isize) = assigned; assigned };
// C line 18178
vm_block = 285; continue 'vm_dispatch;
}

// C line 18164
OP_perm5 => {
let mut tmp_6: JSValue;
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
vm_block = 285; continue 'vm_dispatch;
}

// C line 18155
OP_perm4 => {
let mut tmp_5: JSValue;
let _ = { let assigned = *(sp).offset((((2 as i32)).wrapping_neg()) as isize); tmp_5 = assigned; assigned };
// C line 18156
let _ = { let assigned = *(sp).offset((((3 as i32)).wrapping_neg()) as isize); *(sp).offset((((2 as i32)).wrapping_neg()) as isize) = assigned; assigned };
// C line 18157
let _ = { let assigned = *(sp).offset((((4 as i32)).wrapping_neg()) as isize); *(sp).offset((((3 as i32)).wrapping_neg()) as isize) = assigned; assigned };
// C line 18158
let _ = { let assigned = tmp_5; *(sp).offset((((4 as i32)).wrapping_neg()) as isize) = assigned; assigned };
// C line 18160
vm_block = 285; continue 'vm_dispatch;
}

// C line 18146
OP_rot3r => {
let mut tmp_4: JSValue;
let _ = { let assigned = *(sp).offset((((1 as i32)).wrapping_neg()) as isize); tmp_4 = assigned; assigned };
// C line 18147
let _ = { let assigned = *(sp).offset((((2 as i32)).wrapping_neg()) as isize); *(sp).offset((((1 as i32)).wrapping_neg()) as isize) = assigned; assigned };
// C line 18148
let _ = { let assigned = *(sp).offset((((3 as i32)).wrapping_neg()) as isize); *(sp).offset((((2 as i32)).wrapping_neg()) as isize) = assigned; assigned };
// C line 18149
let _ = { let assigned = tmp_4; *(sp).offset((((3 as i32)).wrapping_neg()) as isize) = assigned; assigned };
// C line 18151
vm_block = 285; continue 'vm_dispatch;
}

// C line 18135
OP_rot5l => {
let mut tmp_3: JSValue;
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
vm_block = 285; continue 'vm_dispatch;
}

// C line 18125
OP_rot4l => {
let mut tmp_2: JSValue;
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
vm_block = 285; continue 'vm_dispatch;
}

// C line 18116
OP_rot3l => {
let mut tmp_1: JSValue;
let _ = { let assigned = *(sp).offset((((3 as i32)).wrapping_neg()) as isize); tmp_1 = assigned; assigned };
// C line 18117
let _ = { let assigned = *(sp).offset((((2 as i32)).wrapping_neg()) as isize); *(sp).offset((((3 as i32)).wrapping_neg()) as isize) = assigned; assigned };
// C line 18118
let _ = { let assigned = *(sp).offset((((1 as i32)).wrapping_neg()) as isize); *(sp).offset((((2 as i32)).wrapping_neg()) as isize) = assigned; assigned };
// C line 18119
let _ = { let assigned = tmp_1; *(sp).offset((((1 as i32)).wrapping_neg()) as isize) = assigned; assigned };
// C line 18121
vm_block = 285; continue 'vm_dispatch;
}

// C line 18108
OP_perm3 => {
let mut tmp: JSValue;
let _ = { let assigned = *(sp).offset((((2 as i32)).wrapping_neg()) as isize); tmp = assigned; assigned };
// C line 18109
let _ = { let assigned = *(sp).offset((((3 as i32)).wrapping_neg()) as isize); *(sp).offset((((2 as i32)).wrapping_neg()) as isize) = assigned; assigned };
// C line 18110
let _ = { let assigned = tmp; *(sp).offset((((3 as i32)).wrapping_neg()) as isize) = assigned; assigned };
// C line 18112
vm_block = 285; continue 'vm_dispatch;
}

// C line 18098
OP_insert4 => {
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
vm_block = 285; continue 'vm_dispatch;
}

// C line 18091
OP_insert3 => {
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
vm_block = 285; continue 'vm_dispatch;
}

// C line 18085
OP_insert2 => {
let _ = { let assigned = *(sp).offset((((1 as i32)).wrapping_neg()) as isize); *(sp).offset(((0 as i32)) as isize) = assigned; assigned };
// C line 18086
let _ = { let assigned = *(sp).offset((((2 as i32)).wrapping_neg()) as isize); *(sp).offset((((1 as i32)).wrapping_neg()) as isize) = assigned; assigned };
// C line 18087
let _ = { let assigned = JS_DupValue(ctx, *(sp).offset(((0 as i32)) as isize)); *(sp).offset((((2 as i32)).wrapping_neg()) as isize) = assigned; assigned };
// C line 18088
let _ = { let old = sp; sp = (sp).offset(1); old };
// C line 18089
vm_block = 285; continue 'vm_dispatch;
}

// C line 18080
OP_dup1 => {
let _ = { let assigned = *(sp).offset((((1 as i32)).wrapping_neg()) as isize); *(sp).offset(((0 as i32)) as isize) = assigned; assigned };
// C line 18081
let _ = { let assigned = JS_DupValue(ctx, *(sp).offset((((2 as i32)).wrapping_neg()) as isize)); *(sp).offset((((1 as i32)).wrapping_neg()) as isize) = assigned; assigned };
// C line 18082
let _ = { let old = sp; sp = (sp).offset(1); old };
// C line 18083
vm_block = 285; continue 'vm_dispatch;
}

// C line 18074
OP_dup3 => {
let _ = { let assigned = JS_DupValue(ctx, *(sp).offset((((3 as i32)).wrapping_neg()) as isize)); *(sp).offset(((0 as i32)) as isize) = assigned; assigned };
// C line 18075
let _ = { let assigned = JS_DupValue(ctx, *(sp).offset((((2 as i32)).wrapping_neg()) as isize)); *(sp).offset(((1 as i32)) as isize) = assigned; assigned };
// C line 18076
let _ = { let assigned = JS_DupValue(ctx, *(sp).offset((((1 as i32)).wrapping_neg()) as isize)); *(sp).offset(((2 as i32)) as isize) = assigned; assigned };
// C line 18077
let _ = { sp = (sp).offset((((3 as i32)) as isize)); sp };
// C line 18078
vm_block = 285; continue 'vm_dispatch;
}

// C line 18069
OP_dup2 => {
let _ = { let assigned = JS_DupValue(ctx, *(sp).offset((((2 as i32)).wrapping_neg()) as isize)); *(sp).offset(((0 as i32)) as isize) = assigned; assigned };
// C line 18070
let _ = { let assigned = JS_DupValue(ctx, *(sp).offset((((1 as i32)).wrapping_neg()) as isize)); *(sp).offset(((1 as i32)) as isize) = assigned; assigned };
// C line 18071
let _ = { sp = (sp).offset((((2 as i32)) as isize)); sp };
// C line 18072
vm_block = 285; continue 'vm_dispatch;
}

// C line 18065
OP_dup => {
let _ = { let assigned = JS_DupValue(ctx, *(sp).offset((((1 as i32)).wrapping_neg()) as isize)); *(sp).offset(((0 as i32)) as isize) = assigned; assigned };
// C line 18066
let _ = { let old = sp; sp = (sp).offset(1); old };
// C line 18067
vm_block = 285; continue 'vm_dispatch;
}

// C line 18059
OP_nip1 => {
let _ = JS_FreeValue(ctx, *(sp).offset((((3 as i32)).wrapping_neg()) as isize));
// C line 18060
let _ = { let assigned = *(sp).offset((((2 as i32)).wrapping_neg()) as isize); *(sp).offset((((3 as i32)).wrapping_neg()) as isize) = assigned; assigned };
// C line 18061
let _ = { let assigned = *(sp).offset((((1 as i32)).wrapping_neg()) as isize); *(sp).offset((((2 as i32)).wrapping_neg()) as isize) = assigned; assigned };
// C line 18062
let _ = { let old = sp; sp = (sp).offset(-1); old };
// C line 18063
vm_block = 285; continue 'vm_dispatch;
}

// C line 18054
OP_nip => {
let _ = JS_FreeValue(ctx, *(sp).offset((((2 as i32)).wrapping_neg()) as isize));
// C line 18055
let _ = { let assigned = *(sp).offset((((1 as i32)).wrapping_neg()) as isize); *(sp).offset((((2 as i32)).wrapping_neg()) as isize) = assigned; assigned };
// C line 18056
let _ = { let old = sp; sp = (sp).offset(-1); old };
// C line 18057
vm_block = 285; continue 'vm_dispatch;
}

// C line 18050
OP_drop => {
let _ = JS_FreeValue(ctx, *(sp).offset((((1 as i32)).wrapping_neg()) as isize));
// C line 18051
let _ = { let old = sp; sp = (sp).offset(-1); old };
// C line 18052
vm_block = 285; continue 'vm_dispatch;
}

// C line 18040
OP_rest => {
let mut first: i32;
first = ((crate::cutils_header::get_u16(pc)) as i32);
// C line 18041
let _ = { pc = (pc).offset((((2 as i32)) as isize)); pc };
// C line 18042
let _ = { let assigned = min_int(first, argc); first = assigned; assigned };
// C line 18043
let _ = { let assigned = js_create_array(ctx, (argc).wrapping_sub(first), (argv).offset(((first) as isize))); *({ let old = sp; sp = (sp).offset(1); old }) = assigned; assigned };
// C line 18044
if ((((!(((!((JS_IsException(*(sp).offset((((1 as i32)).wrapping_neg()) as isize))) != 0) as i32)) != 0) as i32)) as i64)) != 0 {
vm_block = 259; continue 'vm_dispatch;
} else {
vm_block = 285; continue 'vm_dispatch;
}
}

// C line ?
277 => {
let _ = std::process::abort();
vm_block = 285; continue 'vm_dispatch;
}

// C line 18029
278 => {
let _ = { let assigned = js_import_meta(ctx); *({ let old = sp; sp = (sp).offset(1); old }) = assigned; assigned };
// C line 18030
if ((((!(((!((JS_IsException(*(sp).offset((((1 as i32)).wrapping_neg()) as isize))) != 0) as i32)) != 0) as i32)) as i64)) != 0 {
vm_block = 259; continue 'vm_dispatch;
} else {
vm_block = 285; continue 'vm_dispatch;
}
}

// C line 18024
279 => {
let _ = { let assigned = JS_NewObjectProto(ctx, JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_NULL as i32)) as i64) }); *({ let old = sp; sp = (sp).offset(1); old }) = assigned; assigned };
// C line 18025
if ((((!(((!((JS_IsException(*(sp).offset((((1 as i32)).wrapping_neg()) as isize))) != 0) as i32)) != 0) as i32)) as i64)) != 0 {
vm_block = 259; continue 'vm_dispatch;
} else {
vm_block = 285; continue 'vm_dispatch;
}
}

// C line 18016
280 => {
let mut p1: *mut JSObject;
let _ = { let assigned = (((*(p)).u).func).home_object; p1 = assigned; assigned };
// C line 18017
if ((((!(((!(((!(!(p1).is_null()) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 {
// C line 18018
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; *({ let old = sp; sp = (sp).offset(1); old }) = assigned; assigned };
vm_block = 285; continue 'vm_dispatch;
} else {
// C line 18020
let _ = { let assigned = JS_DupValue(ctx, JSValue { u: JSValueUnion { ptr: ((p1) as *mut c_void) }, tag: (((JS_TAG_OBJECT as i32)) as i64) }); *({ let old = sp; sp = (sp).offset(1); old }) = assigned; assigned };
vm_block = 285; continue 'vm_dispatch;
}
}

// C line 18011
281 => {
let _ = { let assigned = JS_DupValue(ctx, new_target); *({ let old = sp; sp = (sp).offset(1); old }) = assigned; assigned };
// C line 18012
vm_block = 285; continue 'vm_dispatch;
}

// C line 18008
282 => {
let _ = { let assigned = JS_DupValue(ctx, (*(sf)).cur_func); *({ let old = sp; sp = (sp).offset(1); old }) = assigned; assigned };
// C line 18009
vm_block = 285; continue 'vm_dispatch;
}

// C line 18002
283 => {
let _ = { let assigned = js_build_mapped_arguments(ctx, argc, argv, sf, min_int(argc, (((*(b)).arg_count) as i32))); *({ let old = sp; sp = (sp).offset(1); old }) = assigned; assigned };
// C line 18004
if ((((!(((!((JS_IsException(*(sp).offset((((1 as i32)).wrapping_neg()) as isize))) != 0) as i32)) != 0) as i32)) as i64)) != 0 {
vm_block = 259; continue 'vm_dispatch;
} else {
vm_block = 285; continue 'vm_dispatch;
}
}

// C line 17997
284 => {
let _ = { let assigned = js_build_arguments(ctx, argc, argv); *({ let old = sp; sp = (sp).offset(1); old }) = assigned; assigned };
// C line 17998
if ((((!(((!((JS_IsException(*(sp).offset((((1 as i32)).wrapping_neg()) as isize))) != 0) as i32)) != 0) as i32)) as i64)) != 0 {
vm_block = 259; continue 'vm_dispatch;
} else {
vm_block = 285; continue 'vm_dispatch;
}
}

// C line 17994
OP_special_object => {
let mut arg: i32;
arg = ((*({ let old = pc; pc = (pc).offset(1); old })) as i32);
// C line 17995
vm_block = match arg { x if x == (OP_SPECIAL_OBJECT_IMPORT_META as i32) => 278, x if x == (OP_SPECIAL_OBJECT_VAR_OBJECT as i32) => 279, x if x == (OP_SPECIAL_OBJECT_HOME_OBJECT as i32) => 280, x if x == (OP_SPECIAL_OBJECT_NEW_TARGET as i32) => 281, x if x == (OP_SPECIAL_OBJECT_THIS_FUNC as i32) => 282, x if x == (OP_SPECIAL_OBJECT_MAPPED_ARGUMENTS as i32) => 283, x if x == (OP_SPECIAL_OBJECT_ARGUMENTS as i32) => 284, _ => 277, }; continue 'vm_dispatch;
}

// C line 17988
OP_object => {
let _ = { let assigned = JS_NewObject(ctx); *({ let old = sp; sp = (sp).offset(1); old }) = assigned; assigned };
// C line 17989
if ((((!(((!((JS_IsException(*(sp).offset((((1 as i32)).wrapping_neg()) as isize))) != 0) as i32)) != 0) as i32)) as i64)) != 0 {
vm_block = 259; continue 'vm_dispatch;
} else {
vm_block = 285; continue 'vm_dispatch;
}
}

// C line 17985
OP_push_true => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((1 as i32)) as u32)) as u64) }, tag: (((JS_TAG_BOOL as i32)) as i64) }; *({ let old = sp; sp = (sp).offset(1); old }) = assigned; assigned };
// C line 17986
vm_block = 285; continue 'vm_dispatch;
}

// C line 17982
OP_push_false => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_BOOL as i32)) as i64) }; *({ let old = sp; sp = (sp).offset(1); old }) = assigned; assigned };
// C line 17983
vm_block = 285; continue 'vm_dispatch;
}

// C line 17963
OP_push_this => {
let mut tag: u32;
let mut val: JSValue;
if ((!(((((((*(b)).js_mode) as i32)) & (((1 as i32)).wrapping_shl(((0 as i32)) as u32)))) != 0) as i32)) != 0 {
// C line 17964
tag = (((((this_obj).tag) as i32)) as u32);
// C line 17965
if ((((!(((!(((((tag) == ((((JS_TAG_OBJECT as i32)) as u32))) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 {
{

// C line ? labels: normal_this
let _ = { let assigned = JS_DupValue(ctx, this_obj); val = assigned; assigned };
{

// C line 17978
let _ = { let assigned = val; *({ let old = sp; sp = (sp).offset(1); old }) = assigned; assigned };
// C line 17980
vm_block = 285; continue 'vm_dispatch;
}
}
} else {
// C line 17967
if ((((((((tag) == ((((JS_TAG_NULL as i32)) as u32))) as i32)) != 0) || (((((tag) == ((((JS_TAG_UNDEFINED as i32)) as u32))) as i32)) != 0)) as i32)) != 0 {
// C line 17968
let _ = { let assigned = JS_DupValue(ctx, (*(ctx)).global_obj); val = assigned; assigned };
{

// C line 17978
let _ = { let assigned = val; *({ let old = sp; sp = (sp).offset(1); old }) = assigned; assigned };
// C line 17980
vm_block = 285; continue 'vm_dispatch;
}
} else {
// C line 17970
let _ = { let assigned = JS_ToObject(ctx, this_obj); val = assigned; assigned };
// C line 17971
if (JS_IsException(val)) != 0 {
vm_block = 259; continue 'vm_dispatch;
} else {
{

// C line 17978
let _ = { let assigned = val; *({ let old = sp; sp = (sp).offset(1); old }) = assigned; assigned };
// C line 17980
vm_block = 285; continue 'vm_dispatch;
}
}
}
}
} else {
{

// C line ? labels: normal_this
let _ = { let assigned = JS_DupValue(ctx, this_obj); val = assigned; assigned };
{

// C line 17978
let _ = { let assigned = val; *({ let old = sp; sp = (sp).offset(1); old }) = assigned; assigned };
// C line 17980
vm_block = 285; continue 'vm_dispatch;
}
}
}
}

// C line 17957
OP_null => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_NULL as i32)) as i64) }; *({ let old = sp; sp = (sp).offset(1); old }) = assigned; assigned };
// C line 17958
vm_block = 285; continue 'vm_dispatch;
}

// C line 17954
OP_undefined => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; *({ let old = sp; sp = (sp).offset(1); old }) = assigned; assigned };
// C line 17955
vm_block = 285; continue 'vm_dispatch;
}

// C line 17950
OP_push_atom_value => {
let _ = { let assigned = JS_AtomToValue(ctx, crate::cutils_header::get_u32(pc)); *({ let old = sp; sp = (sp).offset(1); old }) = assigned; assigned };
// C line 17951
let _ = { pc = (pc).offset((((4 as i32)) as isize)); pc };
// C line 17952
vm_block = 285; continue 'vm_dispatch;
}

// C line 17914
OP_push_const => {
let _ = { let assigned = JS_DupValue(ctx, *((*(b)).cpool).offset((crate::cutils_header::get_u32(pc)) as isize)); *({ let old = sp; sp = (sp).offset(1); old }) = assigned; assigned };
// C line 17915
let _ = { pc = (pc).offset((((4 as i32)) as isize)); pc };
// C line 17916
vm_block = 285; continue 'vm_dispatch;
}

// C line 17910
OP_push_bigint_i32 => {
let _ = { let assigned = __JS_NewShortBigInt(ctx, ((((crate::cutils_header::get_u32(pc)) as i32)) as i64)); *({ let old = sp; sp = (sp).offset(1); old }) = assigned; assigned };
// C line 17911
let _ = { pc = (pc).offset((((4 as i32)) as isize)); pc };
// C line 17912
vm_block = 285; continue 'vm_dispatch;
}

// C line 17906
OP_push_i32 => {
let _ = { let assigned = JS_NewInt32(ctx, ((crate::cutils_header::get_u32(pc)) as i32)); *({ let old = sp; sp = (sp).offset(1); old }) = assigned; assigned };
// C line 17907
let _ = { pc = (pc).offset((((4 as i32)) as isize)); pc };
// C line 17908
vm_block = 285; continue 'vm_dispatch;
}

// C line 17882
286 => {
let _ = { let assigned = (local_buf).offset(((arg_allocated_size) as isize)); var_buf = assigned; assigned };
// C line 17883
let _ = { let assigned = var_buf; (*(sf)).var_buf = assigned; assigned };
// C line 17884
let _ = { let assigned = arg_buf; (*(sf)).arg_buf = assigned; assigned };
// C line 17886
let _ = { let assigned = (0 as i32); i = assigned; assigned };
{

// C line 17886
'cfg_1626: loop {
if ((((i) < ((((*(b)).var_count) as i32))) as i32)) != 0 {
// C line 17887
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; *(var_buf).offset((i) as isize) = assigned; assigned };
// C line ?
let _ = { let old = i; i = (i).wrapping_add(1); old };
continue 'cfg_1626;
} else {
// C line 17889
let _ = { let assigned = (var_buf).offset((((((*(b)).var_count) as i32)) as isize)); stack_buf = assigned; assigned };
// C line 17890
let _ = { let assigned = (((stack_buf).offset((((((*(b)).stack_size) as i32)) as isize))) as *mut *mut JSVarRef); (*(sf)).var_refs = assigned; assigned };
// C line 17891
let _ = { let assigned = (0 as i32); i = assigned; assigned };
{

// C line 17891
'cfg_1620: loop {
if ((((i) < ((((*(b)).var_ref_count) as i32))) as i32)) != 0 {
// C line 17892
let _ = { let assigned = ((core::ptr::null_mut::<c_void>()) as *mut JSVarRef); *((*(sf)).var_refs).offset((i) as isize) = assigned; assigned };
// C line ?
let _ = { let old = i; i = (i).wrapping_add(1); old };
continue 'cfg_1620;
} else {
// C line 17893
let _ = { let assigned = stack_buf; sp = assigned; assigned };
// C line 17894
let _ = { let assigned = (*(b)).byte_code_buf; pc = assigned; assigned };
// C line 17895
let _ = { let assigned = (*(rt)).current_stack_frame; (*(sf)).prev_frame = assigned; assigned };
// C line 17896
let _ = { let assigned = sf; (*(rt)).current_stack_frame = assigned; assigned };
// C line 17897
let _ = { let assigned = (*(b)).realm; ctx = assigned; assigned };
vm_block = 285; continue 'vm_dispatch;
}
}
}
}
}
}
}

// C line 17860
287 => {
let mut n: i32;
let mut alloca_size: usize;
let _ = { let assigned = (((size_of::<JSValue>() as usize)).wrapping_mul(((((arg_allocated_size).wrapping_add((((*(b)).var_count) as i32))).wrapping_add((((*(b)).stack_size) as i32))) as usize))).wrapping_add(((size_of::<*mut JSVarRef>() as usize)).wrapping_mul((((*(b)).var_ref_count) as usize))); alloca_size = assigned; assigned };
// C line 17863
if (js_check_stack_overflow(rt, alloca_size)) != 0 {
// C line 17864
return JS_ThrowStackOverflow(caller_ctx);
} else {
// C line 17866
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
// C line 17873
if ((((!(((!((arg_allocated_size) != 0) as i32)) != 0) as i32)) as i64)) != 0 {
// C line 17874
n = min_int(argc, (((*(b)).arg_count) as i32));
// C line 17875
let _ = { let assigned = local_buf; arg_buf = assigned; assigned };
// C line 17876
let _ = { let assigned = (0 as i32); i = assigned; assigned };
{

// C line 17876
'cfg_1637: loop {
if ((((i) < (n)) as i32)) != 0 {
// C line 17877
let _ = { let assigned = JS_DupValue(caller_ctx, *(argv).offset((i) as isize)); *(arg_buf).offset((i) as isize) = assigned; assigned };
// C line ?
let _ = { let old = i; i = (i).wrapping_add(1); old };
continue 'cfg_1637;
} else {
{

// C line 17878
'cfg_1634: loop {
if ((((i) < ((((*(b)).arg_count) as i32))) as i32)) != 0 {
// C line 17879
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) }; *(arg_buf).offset((i) as isize) = assigned; assigned };
// C line ?
let _ = { let old = i; i = (i).wrapping_add(1); old };
continue 'cfg_1634;
} else {
// C line 17880
let _ = { let assigned = (((*(b)).arg_count) as i32); (*(sf)).arg_count = assigned; assigned };
vm_block = 286; continue 'vm_dispatch;
}
}
}
}
}
}
} else {
vm_block = 286; continue 'vm_dispatch;
}
}
}

// C line 17776
288 => {
let mut call_func: Option<JSClassCall>;
let mut s: *mut JSAsyncFunctionState;
rt = (*(caller_ctx)).rt;
// C line 17780
sf = core::ptr::addr_of_mut!(sf_s);
// C line 17813
if (js_poll_interrupts(caller_ctx)) != 0 {
// C line 17814
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
} else {
// C line 17815
if ((((!(((!((((((((func_obj).tag) as i32)) != ((JS_TAG_OBJECT as i32))) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 {
// C line 17816
if (((flags) & (((1 as i32)).wrapping_shl(((2 as i32)) as u32)))) != 0 {
// C line 17817
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
// C line 17833
if ((*(s)).throw_flag) != 0 {
vm_block = 259; continue 'vm_dispatch;
} else {
vm_block = 285; continue 'vm_dispatch;
}
} else {
{

// C line ? labels: not_a_function
return JS_ThrowTypeError(caller_ctx, c"not a function".as_ptr());
}
}
} else {
// C line 17841
let _ = { let assigned = ((((func_obj).u).ptr) as *mut JSObject); p = assigned; assigned };
// C line 17842
if ((((!(((!((((((((*(p)).class_id) as i32)) != ((JS_CLASS_BYTECODE_FUNCTION as i32))) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 {
// C line 17844
let _ = { let assigned = (*((*(rt)).class_array).offset(((*(p)).class_id) as isize)).call; call_func = assigned; assigned };
// C line 17845
if ((!((call_func).is_some()) as i32)) != 0 {
{

// C line ? labels: not_a_function
return JS_ThrowTypeError(caller_ctx, c"not a function".as_ptr());
}
} else {
// C line 17849
return (call_func).expect("checked class call")(caller_ctx, func_obj, this_obj, argc, argv, flags);
}
} else {
// C line 17852
let _ = { let assigned = (((*(p)).u).func).function_bytecode; b = assigned; assigned };
// C line 17854
if ((((!(((!(((((((((argc) < ((((*(b)).arg_count) as i32))) as i32)) != 0) || ((((flags) & (((1 as i32)).wrapping_shl(((1 as i32)) as u32)))) != 0)) as i32)) != 0) as i32)) != 0) as i32)) as i64)) != 0 {
// C line 17855
let _ = { let assigned = (((*(b)).arg_count) as i32); arg_allocated_size = assigned; assigned };
vm_block = 287; continue 'vm_dispatch;
} else {
// C line 17857
let _ = { let assigned = (0 as i32); arg_allocated_size = assigned; assigned };
vm_block = 287; continue 'vm_dispatch;
}
}
}
}
}

0..=255 => {
let _ = JS_ThrowInternalError(ctx, format_args!("invalid opcode: pc={} opcode=0x{:02x}", (((((pc).offset_from((*(b)).byte_code_buf) as i64)).wrapping_sub((((1 as i32)) as i64))) as i32), opcode));
// C line 20542
vm_block = 259; continue 'vm_dispatch;
}

_ => std::process::abort(),
} }
}
