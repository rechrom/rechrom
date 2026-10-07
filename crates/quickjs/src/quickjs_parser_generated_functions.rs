// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:36951. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_parse_function_decl(mut s: *mut JSParseState, mut func_type: JSParseFunctionEnum, mut func_kind: JSFunctionKindEnum, mut func_name: JSAtom, mut ptr: *const u8) -> i32 {
let mut vm_local_storage = Vec::<u64>::new();
let mut vm_block: usize = 1;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 36957
1 => {
return js_parse_function_decl2(s, func_type, func_kind, func_name, ptr, (((JS_PARSE_EXPORT_NONE as i32)) as JSParseExportEnum), core::ptr::null_mut::<*mut JSFunctionDef>());
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:36961. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_parse_program(mut s: *mut JSParseState) -> i32 {
let _global_var_scope = CompilerGlobalVarScope::new((*s).cur_func);
let mut vm_local_storage = Vec::<u64>::new();
let mut fd: *mut JSFunctionDef = core::ptr::null_mut();
let mut idx: i32 = 0;
let mut vm_block: usize = 26;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 37010
1 => {
return (0 as i32);
}
// C line 37005
2 => {
let _ = emit_return(s, (1 as i32));
vm_block = 1; continue;
}
// C line 37000
3 => {
let _ = emit_atom(s, (((crate::quickjs_atom::JS_ATOM_value as i32)) as JSAtom));
vm_block = 2; continue;
}
// C line 36999
4 => {
let _ = emit_op(s, (((OP_put_field as i32)) as u8));
vm_block = 3; continue;
}
// C line 36997
5 => {
let _ = emit_u16(s, (((*(fd)).eval_ret_idx) as u16));
vm_block = 4; continue;
}
// C line 36996
6 => {
let _ = emit_op(s, (((OP_get_loc as i32)) as u8));
vm_block = 5; continue;
}
// C line 36994
7 => {
let _ = emit_op(s, (((OP_dup as i32)) as u8));
vm_block = 6; continue;
}
// C line 36993
8 => {
let _ = emit_op(s, (((OP_object as i32)) as u8));
vm_block = 7; continue;
}
// C line 37003
9 => {
let _ = emit_u16(s, (((*(fd)).eval_ret_idx) as u16));
vm_block = 2; continue;
}
// C line 37002
10 => {
let _ = emit_op(s, (((OP_get_loc as i32)) as u8));
vm_block = 9; continue;
}
// C line 36990
11 => {
vm_block = if ((((((((*(fd)).func_kind as JSFunctionKindEnum)) as i32)) == ((JS_FUNC_ASYNC as i32))) as i32)) != 0 { 8 } else { 10 }; continue;
}
// C line 37007
12 => {
let _ = emit_return(s, (0 as i32));
vm_block = 1; continue;
}
// C line 36988
13 => {
vm_block = if ((!(((*(s)).is_module) != 0) as i32)) != 0 { 11 } else { 12 }; continue;
}
// C line 36983
14 => {
vm_block = if ((((((*(s)).token).val) != ((TOK_EOF as i32))) as i32)) != 0 { 16 } else { 13 }; continue;
}
// C line 36985
15 => {
return ((1 as i32)).wrapping_neg();
}
// C line 36984
16 => {
vm_block = if (js_parse_source_element(s)) != 0 { 15 } else { 14 }; continue;
}
// C line 36980
17 => {
return ((1 as i32)).wrapping_neg();
}
// C line 36979
18 => {
vm_block = if ((((idx) < ((0 as i32))) as i32)) != 0 { 17 } else { 14 }; continue;
}
// C line 36978
19 => {
let _ = { let assigned = { let assigned = add_var((*(s)).ctx, fd, (((crate::quickjs_atom::JS_ATOM__ret_ as i32)) as JSAtom)); idx = assigned; assigned }; (*(fd)).eval_ret_idx = assigned; assigned };
vm_block = 18; continue;
}
// C line 36976
20 => {
vm_block = if ((!(((*(s)).is_module) != 0) as i32)) != 0 { 19 } else { 14 }; continue;
}
// C line 36972
21 => {
let _ = { let assigned = ((((((((((((*(fd)).eval_type) == (((0 as i32)).wrapping_shl(((0 as i32)) as u32))) as i32)) != 0) || ((((((*(fd)).eval_type) == (((1 as i32)).wrapping_shl(((0 as i32)) as u32))) as i32)) != 0)) as i32)) != 0) || (((!(((((((*(fd)).js_mode) as i32)) & (((1 as i32)).wrapping_shl(((0 as i32)) as u32)))) != 0) as i32)) != 0)) as i32); (*(fd)).is_global_var = assigned; assigned };
vm_block = 20; continue;
}
// C line 36970
22 => {
return ((1 as i32)).wrapping_neg();
}
// C line 36969
23 => {
vm_block = if (js_parse_directives(s)) != 0 { 22 } else { 21 }; continue;
}
// C line 36967
24 => {
return ((1 as i32)).wrapping_neg();
}
// C line 36966
25 => {
vm_block = if (next_token(s)) != 0 { 24 } else { 23 }; continue;
}
// C line 36963
26 => {
fd = (*(s)).cur_func;
vm_block = 25; continue;
}
_ => std::process::abort(),
} }
}
