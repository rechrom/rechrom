// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:32577. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn add_closure_var(mut ctx: *mut JSContext, mut s: *mut JSFunctionDef, mut closure_type: JSClosureTypeEnum, mut var_idx: i32, mut var_name: JSAtom, mut is_const: i32, mut is_lexical: i32, mut var_kind: JSVarKindEnum) -> i32 {
let mut vm_local_storage = Vec::<u64>::new();
let mut cv: *mut JSClosureVar = core::ptr::null_mut();
let mut vm_block: usize = 13;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 32602
1 => {
return ((*(s)).closure_var_count).wrapping_sub((1 as i32));
}
// C line 32601
2 => {
let _ = { let assigned = JS_DupAtom(ctx, var_name); (*(cv)).var_name = assigned; assigned };
vm_block = 1; continue;
}
// C line 32600
3 => {
let _ = { let assigned = ((var_idx) as u16); (*(cv)).var_idx = assigned; assigned };
vm_block = 2; continue;
}
// C line 32599
4 => {
let _ = { let assigned = ((var_kind) as u8); (*(cv)).set_var_kind((assigned) as _); assigned };
vm_block = 3; continue;
}
// C line 32598
5 => {
let _ = { let assigned = ((is_lexical) as u8); (*(cv)).set_is_lexical((assigned) as _); assigned };
vm_block = 4; continue;
}
// C line 32597
6 => {
let _ = { let assigned = ((is_const) as u8); (*(cv)).set_is_const((assigned) as _); assigned };
vm_block = 5; continue;
}
// C line 32596
7 => {
let _ = { let assigned = closure_type; (*(cv)).set_closure_type((assigned) as _); assigned };
vm_block = 6; continue;
}
// C line 32595
8 => {
let _ = { let assigned = core::ptr::addr_of_mut!(*((*(s)).closure_var).offset(({ let old = (*(s)).closure_var_count; (*(s)).closure_var_count = ((*(s)).closure_var_count).wrapping_add(1); old }) as isize)); cv = assigned; assigned };
vm_block = 7; continue;
}
// C line 32594
9 => {
return ((1 as i32)).wrapping_neg();
}
// C line 32591
10 => {
vm_block = if (js_resize_array(ctx, ((core::ptr::addr_of_mut!((*(s)).closure_var)) as *mut *mut c_void), (((size_of::<JSClosureVar>() as usize)) as i32), core::ptr::addr_of_mut!((*(s)).closure_var_size), ((*(s)).closure_var_count).wrapping_add((1 as i32)))) != 0 { 9 } else { 8 }; continue;
}
// C line 32588
11 => {
return ((1 as i32)).wrapping_neg();
}
// C line 32587
12 => {
let _ = JS_ThrowInternalError(ctx, c"too many closure variables".as_ptr());
vm_block = 11; continue;
}
// C line 32586
13 => {
vm_block = if (((((*(s)).closure_var_count) >= ((65534 as i32))) as i32)) != 0 { 12 } else { 10 }; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:32605. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn find_closure_var(mut ctx: *mut JSContext, mut s: *mut JSFunctionDef, mut var_name: JSAtom) -> i32 {
let mut vm_local_storage = Vec::<u64>::new();
let mut i: i32 = 0;
let mut cv: *mut JSClosureVar = core::ptr::null_mut();
let mut vm_block: usize = 7;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 32614
1 => {
return ((1 as i32)).wrapping_neg();
}
// C line 32609
2 => {
vm_block = if ((((i) < ((*(s)).closure_var_count)) as i32)) != 0 { 6 } else { 1 }; continue;
}
// C line 32609
3 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 2; continue;
}
// C line 32612
4 => {
return i;
}
// C line 32611
5 => {
vm_block = if (((((*(cv)).var_name) == (var_name)) as i32)) != 0 { 4 } else { 3 }; continue;
}
// C line 32610
6 => {
cv = core::ptr::addr_of_mut!(*((*(s)).closure_var).offset((i) as isize));
vm_block = 5; continue;
}
// C line 32609
7 => {
let _ = { let assigned = (0 as i32); i = assigned; assigned };
vm_block = 2; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:32619. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn get_closure_var(mut ctx: *mut JSContext, mut s: *mut JSFunctionDef, mut fd: *mut JSFunctionDef, mut closure_type: JSClosureTypeEnum, mut var_idx: i32, mut var_name: JSAtom, mut is_const: i32, mut is_lexical: i32, mut var_kind: JSVarKindEnum) -> i32 {
let mut vm_local_storage = Vec::<u64>::new();
let mut i: i32 = 0;
let mut cv: *mut JSClosureVar = core::ptr::null_mut();
let mut vm_block: usize = 13;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 32641
1 => {
return add_closure_var(ctx, s, closure_type, var_idx, var_name, is_const, is_lexical, var_kind);
}
// C line 32636
2 => {
vm_block = if ((((i) < ((*(s)).closure_var_count)) as i32)) != 0 { 6 } else { 1 }; continue;
}
// C line 32636
3 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 2; continue;
}
// C line 32639
4 => {
return i;
}
// C line 32638
5 => {
vm_block = if (((((((((((*(cv)).var_idx) as i32)) == (var_idx)) as i32)) != 0) && ((((((((*(cv)).closure_type()) as u32)) == (((closure_type) as u32))) as i32)) != 0)) as i32)) != 0 { 4 } else { 3 }; continue;
}
// C line 32637
6 => {
cv = core::ptr::addr_of_mut!(*((*(s)).closure_var).offset((i) as isize));
vm_block = 5; continue;
}
// C line 32636
7 => {
let _ = { let assigned = (0 as i32); i = assigned; assigned };
vm_block = 2; continue;
}
// C line 32634
8 => {
let _ = { let assigned = (((JS_CLOSURE_REF as i32)) as JSClosureTypeEnum); closure_type = assigned; assigned };
vm_block = 7; continue;
}
// C line 32633
9 => {
vm_block = if ((((((closure_type) as u32)) != ((((JS_CLOSURE_GLOBAL_REF as i32)) as u32))) as i32)) != 0 { 8 } else { 7 }; continue;
}
// C line 32632
10 => {
return ((1 as i32)).wrapping_neg();
}
// C line 32631
11 => {
vm_block = if ((((var_idx) < ((0 as i32))) as i32)) != 0 { 10 } else { 9 }; continue;
}
// C line 32628
12 => {
let _ = { let assigned = get_closure_var(ctx, (*(s)).parent, fd, closure_type, var_idx, var_name, is_const, is_lexical, var_kind); var_idx = assigned; assigned };
vm_block = 11; continue;
}
// C line 32627
13 => {
vm_block = if ((((fd) != ((*(s)).parent)) as i32)) != 0 { 12 } else { 7 }; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:32645. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn get_with_scope_opcode(mut op: i32) -> i32 {
let mut vm_local_storage = Vec::<u64>::new();
let mut vm_block: usize = 3;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 32648
1 => {
return (OP_with_get_var as i32);
}
// C line 32650
2 => {
return ((OP_with_get_var as i32)).wrapping_add((op).wrapping_sub((OP_scope_get_var as i32)));
}
// C line 32647
3 => {
vm_block = if ((((op) == ((OP_scope_get_var_undef as i32))) as i32)) != 0 { 1 } else { 2 }; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:32653. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn can_opt_put_ref_value(mut bc_buf: *const u8, mut pos: i32) -> i32 {
let mut vm_local_storage = Vec::<u64>::new();
let mut opcode: i32 = 0;
let mut vm_block: usize = 2;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 32656
1 => {
return (((((((((*(bc_buf).offset(((pos).wrapping_add((1 as i32))) as isize)) as i32)) == ((OP_put_ref_value as i32))) as i32)) != 0) && (((((((((((((((((opcode) == ((OP_insert3 as i32))) as i32)) != 0) || (((((opcode) == ((OP_perm4 as i32))) as i32)) != 0)) as i32)) != 0) || (((((opcode) == ((OP_nop as i32))) as i32)) != 0)) as i32)) != 0) || (((((opcode) == ((OP_rot3l as i32))) as i32)) != 0)) as i32)) != 0)) as i32);
}
// C line 32655
2 => {
opcode = ((*(bc_buf).offset((pos) as isize)) as i32);
vm_block = 1; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:32663. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn can_opt_put_global_ref_value(mut bc_buf: *const u8, mut pos: i32) -> i32 {
let mut vm_local_storage = Vec::<u64>::new();
let mut opcode: i32 = 0;
let mut vm_block: usize = 2;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 32666
1 => {
return (((((((((*(bc_buf).offset(((pos).wrapping_add((1 as i32))) as isize)) as i32)) == ((OP_put_ref_value as i32))) as i32)) != 0) && (((((((((((((((((opcode) == ((OP_insert3 as i32))) as i32)) != 0) || (((((opcode) == ((OP_perm4 as i32))) as i32)) != 0)) as i32)) != 0) || (((((opcode) == ((OP_nop as i32))) as i32)) != 0)) as i32)) != 0) || (((((opcode) == ((OP_rot3l as i32))) as i32)) != 0)) as i32)) != 0)) as i32);
}
// C line 32665
2 => {
opcode = ((*(bc_buf).offset((pos) as isize)) as i32);
vm_block = 1; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:32673. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn optimize_scope_make_ref(mut ctx: *mut JSContext, mut s: *mut JSFunctionDef, mut bc: *mut DynBuf, mut bc_buf: *mut u8, mut ls: *mut LabelSlot, mut pos_next: i32, mut get_op: i32, mut var_idx: i32) -> i32 {
let mut vm_local_storage = Vec::<u64>::new();
let mut label_pos: i32 = 0;
let mut end_pos: i32 = 0;
let mut pos: i32 = 0;
let mut vm_block: usize = 16;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 32714
1 => {
return pos_next;
}
// C line 32712
2 => {
vm_block = if ((((pos) < (end_pos)) as i32)) != 0 { 3 } else { 1 }; continue;
}
// C line 32713
3 => {
let _ = { let assigned = (((OP_nop as i32)) as u8); *(bc_buf).offset(({ let old = pos; pos = (pos).wrapping_add(1); old }) as isize) = assigned; assigned };
vm_block = 2; continue;
}
// C line 32710
4 => {
let _ = { pos = ((((pos) as i32)).wrapping_add((3 as i32))) as i32; pos };
vm_block = 2; continue;
}
// C line 32709
5 => {
let _ = crate::cutils_header::put_u16(((bc_buf).offset(((pos) as isize))).offset((((1 as i32)) as isize)), ((var_idx) as u16));
vm_block = 4; continue;
}
// C line 32708
6 => {
let _ = { let assigned = (((get_op).wrapping_add((1 as i32))) as u8); *(bc_buf).offset((pos) as isize) = assigned; assigned };
vm_block = 5; continue;
}
// C line 32707
7 => {
let _ = { let assigned = (((OP_dup as i32)) as u8); *(bc_buf).offset(({ let old = pos; pos = (pos).wrapping_add(1); old }) as isize) = assigned; assigned };
vm_block = 6; continue;
}
// C line 32706
8 => {
vm_block = if ((((((*(bc_buf).offset((label_pos) as isize)) as i32)) == ((OP_insert3 as i32))) as i32)) != 0 { 7 } else { 6 }; continue;
}
// C line 32705
9 => {
let _ = { let assigned = (label_pos).wrapping_add((2 as i32)); end_pos = assigned; assigned };
vm_block = 8; continue;
}
// C line 32698
10 => {
let _ = if ((((!(((((((*(bc_buf).offset((pos) as isize)) as i32)) == ((OP_label as i32))) as i32)) != 0) as i32)) as i64)) != 0 { std::process::abort() } else { { let _ = (0 as i32); } };
vm_block = 9; continue;
}
// C line 32697
11 => {
let _ = { let assigned = (label_pos).wrapping_sub((5 as i32)); pos = assigned; assigned };
vm_block = 10; continue;
}
// C line 32696
12 => {
let _ = { let assigned = (*(ls)).pos; label_pos = assigned; assigned };
vm_block = 11; continue;
}
// C line 32691
13 => {
let _ = { let old = pos_next; pos_next = (pos_next).wrapping_add(1); old };
vm_block = 12; continue;
}
// C line 32690
14 => {
let _ = dbuf_put_u16(bc, ((var_idx) as u16));
vm_block = 13; continue;
}
// C line 32689
15 => {
let _ = dbuf_putc(bc, ((get_op) as u8));
vm_block = 14; continue;
}
// C line 32688
16 => {
vm_block = if ((((((*(bc_buf).offset((pos_next) as isize)) as i32)) == ((OP_get_ref_value as i32))) as i32)) != 0 { 15 } else { 12 }; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:32717. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn add_var_this(mut ctx: *mut JSContext, mut fd: *mut JSFunctionDef) -> i32 {
let mut vm_local_storage = Vec::<u64>::new();
let mut idx: i32 = 0;
let mut vd: *mut JSVarDef = core::ptr::null_mut();
let mut vm_block: usize = 5;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 32727
1 => {
return idx;
}
// C line 32724
2 => {
let _ = { let assigned = (((1 as i32)) as u8); (*(vd)).set_is_lexical((assigned) as _); assigned };
vm_block = 1; continue;
}
// C line 32722
3 => {
vd = core::ptr::addr_of_mut!(*((*(fd)).vars).offset((idx) as isize));
vm_block = 2; continue;
}
// C line 32721
4 => {
vm_block = if ((((((((idx) >= ((0 as i32))) as i32)) != 0) && (((*(fd)).is_derived_class_constructor) != 0)) as i32)) != 0 { 3 } else { 1 }; continue;
}
// C line 32720
5 => {
let _ = { let assigned = add_var(ctx, fd, (((crate::quickjs_atom::JS_ATOM_this as i32)) as JSAtom)); idx = assigned; assigned };
vm_block = 4; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:32730. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn resolve_pseudo_var(mut ctx: *mut JSContext, mut s: *mut JSFunctionDef, mut var_name: JSAtom) -> i32 {
let mut vm_local_storage = Vec::<u64>::new();
let mut var_idx: i32 = 0;
let mut vm_block: usize = 22;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 32766
1 => {
return var_idx;
}
// C line 32764
2 => {
vm_block = 1; continue;
}
// C line 32763
3 => {
let _ = { let assigned = ((1 as i32)).wrapping_neg(); var_idx = assigned; assigned };
vm_block = 2; continue;
}
// C line 32761
4 => {
vm_block = 1; continue;
}
// C line 32760
5 => {
let _ = { let assigned = (*(s)).this_var_idx; var_idx = assigned; assigned };
vm_block = 4; continue;
}
// C line 32759
6 => {
let _ = { let assigned = add_var_this(ctx, s); (*(s)).this_var_idx = assigned; assigned };
vm_block = 5; continue;
}
// C line 32758
7 => {
vm_block = if (((((*(s)).this_var_idx) < ((0 as i32))) as i32)) != 0 { 6 } else { 5 }; continue;
}
// C line 32755
8 => {
vm_block = 1; continue;
}
// C line 32754
9 => {
let _ = { let assigned = (*(s)).new_target_var_idx; var_idx = assigned; assigned };
vm_block = 8; continue;
}
// C line 32753
10 => {
let _ = { let assigned = add_var(ctx, s, var_name); (*(s)).new_target_var_idx = assigned; assigned };
vm_block = 9; continue;
}
// C line 32752
11 => {
vm_block = if (((((*(s)).new_target_var_idx) < ((0 as i32))) as i32)) != 0 { 10 } else { 9 }; continue;
}
// C line 32749
12 => {
vm_block = 1; continue;
}
// C line 32748
13 => {
let _ = { let assigned = (*(s)).this_active_func_var_idx; var_idx = assigned; assigned };
vm_block = 12; continue;
}
// C line 32747
14 => {
let _ = { let assigned = add_var(ctx, s, var_name); (*(s)).this_active_func_var_idx = assigned; assigned };
vm_block = 13; continue;
}
// C line 32746
15 => {
vm_block = if (((((*(s)).this_active_func_var_idx) < ((0 as i32))) as i32)) != 0 { 14 } else { 13 }; continue;
}
// C line 32743
16 => {
vm_block = 1; continue;
}
// C line 32742
17 => {
let _ = { let assigned = (*(s)).home_object_var_idx; var_idx = assigned; assigned };
vm_block = 16; continue;
}
// C line 32741
18 => {
let _ = { let assigned = add_var(ctx, s, var_name); (*(s)).home_object_var_idx = assigned; assigned };
vm_block = 17; continue;
}
// C line 32740
19 => {
vm_block = if (((((*(s)).home_object_var_idx) < ((0 as i32))) as i32)) != 0 { 18 } else { 17 }; continue;
}
// C line 32737
20 => {
vm_block = match var_name { x if x == (((crate::quickjs_atom::JS_ATOM_this as i32)) as JSAtom) => 7, x if x == (((crate::quickjs_atom::JS_ATOM_new_target as i32)) as JSAtom) => 11, x if x == (((crate::quickjs_atom::JS_ATOM_this_active_func as i32)) as JSAtom) => 15, x if x == (((crate::quickjs_atom::JS_ATOM_home_object as i32)) as JSAtom) => 19, _ => 3, }; continue;
}
// C line 32736
21 => {
return ((1 as i32)).wrapping_neg();
}
// C line 32735
22 => {
vm_block = if ((!(((*(s)).has_this_binding) != 0) as i32)) != 0 { 21 } else { 20 }; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:32771. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn var_object_test(mut ctx: *mut JSContext, mut s: *mut JSFunctionDef, mut var_name: JSAtom, mut op: i32, mut bc: *mut DynBuf, mut plabel_done: *mut i32, mut is_with: i32) -> () {
let mut vm_local_storage = Vec::<u64>::new();
let mut vm_block: usize = 11;
loop { match vm_block {
// C line ?
0 => {
return ();
}
// C line 32787
1 => {
let _ = { let old = (*(s)).jump_size; (*(s)).jump_size = ((*(s)).jump_size).wrapping_add(1); old };
vm_block = 0; continue;
}
// C line 32786
2 => {
let _ = update_label(s, *(plabel_done), (1 as i32));
vm_block = 1; continue;
}
// C line 32785
3 => {
let _ = dbuf_putc(bc, ((is_with) as u8));
vm_block = 2; continue;
}
// C line 32784
4 => {
let _ = dbuf_put_u32(bc, ((*(plabel_done)) as u32));
vm_block = 3; continue;
}
// C line 32781
5 => {
return;
}
// C line 32780
6 => {
let _ = dbuf_set_error(bc);
vm_block = 5; continue;
}
// C line 32779
7 => {
vm_block = if ((((*(plabel_done)) < ((0 as i32))) as i32)) != 0 { 6 } else { 4 }; continue;
}
// C line 32778
8 => {
let _ = { let assigned = new_label_fd(s); *(plabel_done) = assigned; assigned };
vm_block = 7; continue;
}
// C line 32777
9 => {
vm_block = if ((((*(plabel_done)) < ((0 as i32))) as i32)) != 0 { 8 } else { 4 }; continue;
}
// C line 32776
10 => {
let _ = dbuf_put_u32(bc, JS_DupAtom(ctx, var_name));
vm_block = 9; continue;
}
// C line 32775
11 => {
let _ = dbuf_putc(bc, ((get_with_scope_opcode(op)) as u8));
vm_block = 10; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:32790. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn capture_var(mut s: *mut JSFunctionDef, mut vd: *mut JSVarDef) -> () {
let mut vm_local_storage = Vec::<u64>::new();
let mut vm_block: usize = 3;
loop { match vm_block {
// C line ?
0 => {
return ();
}
// C line 32794
1 => {
let _ = { let assigned = (({ let old = (*(s)).var_ref_count; (*(s)).var_ref_count = ((*(s)).var_ref_count).wrapping_add(1); old }) as u16); (*(vd)).var_ref_idx = assigned; assigned };
vm_block = 0; continue;
}
// C line 32793
2 => {
let _ = { let assigned = (((1 as i32)) as u8); (*(vd)).set_is_captured((assigned) as _); assigned };
vm_block = 1; continue;
}
// C line 32792
3 => {
vm_block = if ((!(((*(vd)).is_captured()) != 0) as i32)) != 0 { 2 } else { 0 }; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:32799. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn resolve_scope_var(mut ctx: *mut JSContext, mut s: *mut JSFunctionDef, mut var_name: JSAtom, mut scope_level: i32, mut op: i32, mut bc: *mut DynBuf, mut bc_buf: *mut u8, mut ls: *mut LabelSlot, mut pos_next: i32, scope_lookup: &mut CompilerScopeLookup) -> i32 {
let mut vm_local_storage = Vec::<u64>::new();
let mut idx: i32 = 0;
let mut var_idx: i32 = 0;
let mut is_put: i32 = 0;
let mut label_done: i32 = 0;
let mut fd: *mut JSFunctionDef = core::ptr::null_mut();
let mut vd: *mut JSVarDef = core::ptr::null_mut();
let mut is_pseudo_var: i32 = 0;
let mut is_arg_scope: i32 = 0;
let mut get_op: i32 = 0;
let mut idx1: i32 = 0;
let mut cv: *mut JSClosureVar = core::ptr::null_mut();
let mut closure_type: JSClosureTypeEnum = core::mem::zeroed();
let mut is_with: i32 = 0;
let mut get_op_1: i32 = 0;
let mut vm_block: usize = 263;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 33277
1 => {
return pos_next;
}
// C line 33275
2 => {
let _ = { let assigned = (((*(bc)).size) as i32); (*((*(s)).label_slots).offset((label_done) as isize)).pos2 = assigned; assigned };
vm_block = 1; continue;
}
// C line 33274
3 => {
let _ = dbuf_put_u32(bc, ((label_done) as u32));
vm_block = 2; continue;
}
// C line 33273
4 => {
let _ = dbuf_putc(bc, (((OP_label as i32)) as u8));
vm_block = 3; continue;
}
// C line 33272 labels: done
5 => {
vm_block = if ((((label_done) >= ((0 as i32))) as i32)) != 0 { 4 } else { 1 }; continue;
}
// C line 33165
6 => {
vm_block = 5; continue;
}
// C line 33164
7 => {
let _ = dbuf_put_u32(bc, JS_DupAtom(ctx, var_name));
vm_block = 6; continue;
}
// C line 33163
8 => {
let _ = dbuf_putc(bc, (((OP_delete_var as i32)) as u8));
vm_block = 7; continue;
}
// C line 33161
9 => {
vm_block = 5; continue;
}
// C line 33160
10 => {
let _ = dbuf_put_u16(bc, ((idx) as u16));
vm_block = 9; continue;
}
// C line 33159
11 => {
let _ = dbuf_putc(bc, (((OP_put_var_init as i32)) as u8));
vm_block = 10; continue;
}
// C line 33157
12 => {
vm_block = 5; continue;
}
// C line 33156
13 => {
let _ = dbuf_put_u16(bc, ((idx) as u16));
vm_block = 12; continue;
}
// C line 33155
14 => {
let _ = dbuf_putc(bc, ((((OP_get_var_undef as i32)).wrapping_add((op).wrapping_sub((OP_scope_get_var_undef as i32)))) as u8));
vm_block = 13; continue;
}
// C line 33151
15 => {
vm_block = 5; continue;
}
// C line 33150
16 => {
let _ = dbuf_put_u16(bc, ((idx) as u16));
vm_block = 15; continue;
}
// C line 33149
17 => {
let _ = dbuf_putc(bc, (((OP_get_var as i32)) as u8));
vm_block = 16; continue;
}
// C line 33148
18 => {
let _ = dbuf_putc(bc, (((OP_undefined as i32)) as u8));
vm_block = 17; continue;
}
// C line 33144
19 => {
vm_block = 5; continue;
}
// C line 33137
20 => {
let _ = { let assigned = optimize_scope_make_ref(ctx, s, bc, bc_buf, ls, pos_next, (OP_get_var as i32), idx); pos_next = assigned; assigned };
vm_block = 19; continue;
}
// C line 33142
21 => {
let _ = dbuf_put_u32(bc, JS_DupAtom(ctx, var_name));
vm_block = 19; continue;
}
// C line 33141
22 => {
let _ = dbuf_putc(bc, (((OP_make_var_ref as i32)) as u8));
vm_block = 21; continue;
}
// C line 33136
23 => {
vm_block = if ((((((((label_done) == (((1 as i32)).wrapping_neg())) as i32)) != 0) && ((can_opt_put_global_ref_value(bc_buf, (*(ls)).pos)) != 0)) as i32)) != 0 { 20 } else { 22 }; continue;
}
// C line 33134 labels: has_global_idx
24 => {
vm_block = match op { x if x == (OP_scope_delete_var as i32) => 8, x if x == (OP_scope_put_var_init as i32) => 11, x if x == (OP_scope_put_var as i32) => 14, x if x == (OP_scope_get_var as i32) => 14, x if x == (OP_scope_get_var_undef as i32) => 14, x if x == (OP_scope_get_ref as i32) => 18, x if x == (OP_scope_make_ref as i32) => 23, _ => 5, }; continue;
}
// C line 33124
25 => {
let _ = { let assigned = get_closure_var(ctx, s, fd, (((JS_CLOSURE_GLOBAL_REF as i32)) as JSClosureTypeEnum), idx1, var_name, (0 as i32), (0 as i32), (((JS_VAR_NORMAL as i32)) as JSVarKindEnum)); idx = assigned; assigned };
vm_block = 24; continue;
}
// C line 33130
26 => {
let _ = { let assigned = idx1; idx = assigned; assigned };
vm_block = 24; continue;
}
// C line 33123
27 => {
vm_block = if ((((fd) != (s)) as i32)) != 0 { 25 } else { 26 }; continue;
}
// C line 33122
28 => {
return ((1 as i32)).wrapping_neg();
}
// C line 33121
29 => {
vm_block = if ((((idx1) < ((0 as i32))) as i32)) != 0 { 28 } else { 27 }; continue;
}
// C line 33119
30 => {
let _ = { let assigned = add_closure_var(ctx, fd, (((JS_CLOSURE_GLOBAL as i32)) as JSClosureTypeEnum), (0 as i32), var_name, (0 as i32), (0 as i32), (((JS_VAR_NORMAL as i32)) as JSVarKindEnum)); idx1 = assigned; assigned };
vm_block = 29; continue;
}
// C line 33074
31 => {
vm_block = if ((((idx1) < ((*(fd)).closure_var_count)) as i32)) != 0 { 51 } else { 30 }; continue;
}
// C line 33074
32 => {
let _ = { let old = idx1; idx1 = (idx1).wrapping_add(1); old };
vm_block = 31; continue;
}
// C line 33096
33 => {
vm_block = 24; continue;
}
// C line 33098
34 => {
vm_block = 98; continue;
}
// C line 33093
35 => {
vm_block = if (((((((((((((((*(cv)).closure_type()) as i32)) == ((JS_CLOSURE_GLOBAL as i32))) as i32)) != 0) || ((((((((*(cv)).closure_type()) as i32)) == ((JS_CLOSURE_GLOBAL_DECL as i32))) as i32)) != 0)) as i32)) != 0) || ((((((((*(cv)).closure_type()) as i32)) == ((JS_CLOSURE_GLOBAL_REF as i32))) as i32)) != 0)) as i32)) != 0 { 33 } else { 34 }; continue;
}
// C line 33085
36 => {
let _ = { let assigned = get_closure_var(ctx, s, fd, closure_type, idx1, (*(cv)).var_name, (((*(cv)).is_const()) as i32), (((*(cv)).is_lexical()) as i32), (((*(cv)).var_kind()) as JSVarKindEnum)); idx = assigned; assigned };
vm_block = 35; continue;
}
// C line 33082
37 => {
let _ = { let assigned = (((JS_CLOSURE_GLOBAL_REF as i32)) as JSClosureTypeEnum); closure_type = assigned; assigned };
vm_block = 36; continue;
}
// C line 33084
38 => {
let _ = { let assigned = (((JS_CLOSURE_REF as i32)) as JSClosureTypeEnum); closure_type = assigned; assigned };
vm_block = 36; continue;
}
// C line 33079
39 => {
vm_block = if (((((((((((((((*(cv)).closure_type()) as i32)) == ((JS_CLOSURE_GLOBAL as i32))) as i32)) != 0) || ((((((((*(cv)).closure_type()) as i32)) == ((JS_CLOSURE_GLOBAL_DECL as i32))) as i32)) != 0)) as i32)) != 0) || ((((((((*(cv)).closure_type()) as i32)) == ((JS_CLOSURE_GLOBAL_REF as i32))) as i32)) != 0)) as i32)) != 0 { 37 } else { 38 }; continue;
}
// C line 33091
40 => {
let _ = { let assigned = idx1; idx = assigned; assigned };
vm_block = 35; continue;
}
// C line 33077
41 => {
vm_block = if ((((fd) != (s)) as i32)) != 0 { 39 } else { 40 }; continue;
}
// C line 33114
42 => {
let _ = var_object_test(ctx, s, var_name, op, bc, core::ptr::addr_of_mut!(label_done), is_with);
vm_block = 32; continue;
}
// C line 33113
43 => {
let _ = dbuf_put_u16(bc, ((idx) as u16));
vm_block = 42; continue;
}
// C line 33112
44 => {
let _ = dbuf_putc(bc, (((OP_get_var_ref as i32)) as u8));
vm_block = 43; continue;
}
// C line 33104
45 => {
let _ = { let assigned = get_closure_var(ctx, s, fd, (((JS_CLOSURE_REF as i32)) as JSClosureTypeEnum), idx1, (*(cv)).var_name, (0 as i32), (0 as i32), (((JS_VAR_NORMAL as i32)) as JSVarKindEnum)); idx = assigned; assigned };
vm_block = 44; continue;
}
// C line 33110
46 => {
let _ = { let assigned = idx1; idx = assigned; assigned };
vm_block = 44; continue;
}
// C line 33103
47 => {
vm_block = if ((((fd) != (s)) as i32)) != 0 { 45 } else { 46 }; continue;
}
// C line 33102
48 => {
is_with = ((((*(cv)).var_name) == ((((crate::quickjs_atom::JS_ATOM__with_ as i32)) as JSAtom))) as i32);
vm_block = 47; continue;
}
// C line 33099
49 => {
vm_block = if (((((((((((((((((*(cv)).var_name) == ((((crate::quickjs_atom::JS_ATOM__var_ as i32)) as JSAtom))) as i32)) != 0) || ((((((*(cv)).var_name) == ((((crate::quickjs_atom::JS_ATOM__arg_var_ as i32)) as JSAtom))) as i32)) != 0)) as i32)) != 0) || ((((((*(cv)).var_name) == ((((crate::quickjs_atom::JS_ATOM__with_ as i32)) as JSAtom))) as i32)) != 0)) as i32)) != 0) && (((!((is_pseudo_var) != 0) as i32)) != 0)) as i32)) != 0 { 48 } else { 32 }; continue;
}
// C line 33076
50 => {
vm_block = if ((((var_name) == ((*(cv)).var_name)) as i32)) != 0 { 41 } else { 49 }; continue;
}
// C line 33075
51 => {
cv = core::ptr::addr_of_mut!(*((*(fd)).closure_var).offset((idx1) as isize));
vm_block = 50; continue;
}
// C line 33074
52 => {
idx1 = scope_lookup.find_closure(fd, var_name, is_pseudo_var != 0).unwrap_or(0);
vm_block = 31; continue;
}
// C line 33267
53 => {
vm_block = 5; continue;
}
// C line 33265
54 => {
vm_block = 53; continue;
}
// C line 33264
55 => {
let _ = dbuf_putc(bc, (((OP_push_false as i32)) as u8));
vm_block = 54; continue;
}
// C line 33262
56 => {
vm_block = 53; continue;
}
// C line 33261
57 => {
let _ = dbuf_put_u16(bc, ((idx) as u16));
vm_block = 56; continue;
}
// C line 33245
58 => {
let _ = dbuf_putc(bc, (((OP_put_var_ref_check_init as i32)) as u8));
vm_block = 57; continue;
}
// C line 33247
59 => {
let _ = dbuf_putc(bc, (((OP_put_var_ref as i32)) as u8));
vm_block = 57; continue;
}
// C line 33244
60 => {
vm_block = if ((((var_name) == ((((crate::quickjs_atom::JS_ATOM_this as i32)) as JSAtom))) as i32)) != 0 { 58 } else { 59 }; continue;
}
// C line 33249
61 => {
let _ = dbuf_putc(bc, (((OP_put_var_ref_check as i32)) as u8));
vm_block = 57; continue;
}
// C line 33242
62 => {
vm_block = if ((((op) == ((OP_scope_put_var_init as i32))) as i32)) != 0 { 60 } else { 61 }; continue;
}
// C line 33252
63 => {
let _ = dbuf_putc(bc, (((OP_put_var_ref as i32)) as u8));
vm_block = 57; continue;
}
// C line 33241
64 => {
vm_block = if ((*((*(s)).closure_var).offset((idx) as isize)).is_lexical()) != 0 { 62 } else { 63 }; continue;
}
// C line 33256
65 => {
let _ = dbuf_putc(bc, (((OP_get_var_ref_check as i32)) as u8));
vm_block = 57; continue;
}
// C line 33258
66 => {
let _ = dbuf_putc(bc, (((OP_get_var_ref as i32)) as u8));
vm_block = 57; continue;
}
// C line 33255
67 => {
vm_block = if ((*((*(s)).closure_var).offset((idx) as isize)).is_lexical()) != 0 { 65 } else { 66 }; continue;
}
// C line 33240
68 => {
vm_block = if (is_put) != 0 { 64 } else { 67 }; continue;
}
// C line 33238 labels: closure_scope_var
69 => {
let _ = { let assigned = (((((((op) == ((OP_scope_put_var as i32))) as i32)) != 0) || (((((op) == ((OP_scope_put_var_init as i32))) as i32)) != 0)) as i32); is_put = assigned; assigned };
vm_block = 68; continue;
}
// C line 33233
70 => {
vm_block = 69; continue;
}
// C line 33232
71 => {
let _ = dbuf_putc(bc, (((OP_undefined as i32)) as u8));
vm_block = 70; continue;
}
// C line 33228
72 => {
vm_block = 69; continue;
}
// C line 33226
73 => {
vm_block = 5; continue;
}
// C line 33225
74 => {
let _ = dbuf_putc(bc, (((OP_drop as i32)) as u8));
vm_block = 73; continue;
}
// C line 33223
75 => {
vm_block = if (((((((*((*(s)).closure_var).offset((idx) as isize)).var_kind()) as i32)) == ((JS_VAR_FUNCTION_NAME as i32))) as i32)) != 0 { 74 } else { 72 }; continue;
}
// C line 33221
76 => {
vm_block = 53; continue;
}
// C line 33202
77 => {
let _ = dbuf_put_u32(bc, JS_DupAtom(ctx, var_name));
vm_block = 76; continue;
}
// C line 33201
78 => {
let _ = dbuf_putc(bc, (((OP_push_atom_value as i32)) as u8));
vm_block = 77; continue;
}
// C line 33200
79 => {
let _ = dbuf_put_u32(bc, JS_DupAtom(ctx, var_name));
vm_block = 78; continue;
}
// C line 33199
80 => {
let _ = dbuf_putc(bc, (((OP_define_field as i32)) as u8));
vm_block = 79; continue;
}
// C line 33198
81 => {
let _ = dbuf_put_u16(bc, ((idx) as u16));
vm_block = 80; continue;
}
// C line 33197
82 => {
let _ = dbuf_putc(bc, (((OP_get_var_ref as i32)) as u8));
vm_block = 81; continue;
}
// C line 33196
83 => {
let _ = dbuf_putc(bc, (((OP_object as i32)) as u8));
vm_block = 82; continue;
}
// C line 33211
84 => {
let _ = { let assigned = optimize_scope_make_ref(ctx, s, bc, bc_buf, ls, pos_next, get_op_1, idx); pos_next = assigned; assigned };
vm_block = 76; continue;
}
// C line 33208
85 => {
let _ = { let assigned = (OP_get_var_ref_check as i32); get_op_1 = assigned; assigned };
vm_block = 84; continue;
}
// C line 33210
86 => {
let _ = { let assigned = (OP_get_var_ref as i32); get_op_1 = assigned; assigned };
vm_block = 84; continue;
}
// C line 33207
87 => {
vm_block = if ((*((*(s)).closure_var).offset((idx) as isize)).is_lexical()) != 0 { 85 } else { 86 }; continue;
}
// C line 33219
88 => {
let _ = dbuf_put_u16(bc, ((idx) as u16));
vm_block = 76; continue;
}
// C line 33218
89 => {
let _ = dbuf_put_u32(bc, JS_DupAtom(ctx, var_name));
vm_block = 88; continue;
}
// C line 33217
90 => {
let _ = dbuf_putc(bc, (((OP_make_var_ref_ref as i32)) as u8));
vm_block = 89; continue;
}
// C line 33204
91 => {
vm_block = if ((((((((label_done) == (((1 as i32)).wrapping_neg())) as i32)) != 0) && ((can_opt_put_ref_value(bc_buf, (*(ls)).pos)) != 0)) as i32)) != 0 { 87 } else { 90 }; continue;
}
// C line 33194
92 => {
vm_block = if (((((((*((*(s)).closure_var).offset((idx) as isize)).var_kind()) as i32)) == ((JS_VAR_FUNCTION_NAME as i32))) as i32)) != 0 { 83 } else { 91 }; continue;
}
// C line 33192
93 => {
vm_block = match op { x if x == (OP_scope_delete_var as i32) => 55, x if x == (OP_scope_put_var_init as i32) => 69, x if x == (OP_scope_get_var as i32) => 69, x if x == (OP_scope_get_var_undef as i32) => 69, x if x == (OP_scope_get_ref as i32) => 71, x if x == (OP_scope_put_var as i32) => 75, x if x == (OP_scope_make_ref as i32) => 92, _ => 53, }; continue;
}
// C line 33190
94 => {
vm_block = 5; continue;
}
// C line 33189
95 => {
let _ = dbuf_putc(bc, (((0 as i32)) as u8));
vm_block = 94; continue;
}
// C line 33188
96 => {
let _ = dbuf_put_u32(bc, JS_DupAtom(ctx, var_name));
vm_block = 95; continue;
}
// C line 33187
97 => {
let _ = dbuf_putc(bc, (((OP_throw_error as i32)) as u8));
vm_block = 96; continue;
}
// C line 33185 labels: has_idx
98 => {
vm_block = if ((((((((((((op) == ((OP_scope_put_var as i32))) as i32)) != 0) || (((((op) == ((OP_scope_make_ref as i32))) as i32)) != 0)) as i32)) != 0) && (((((*((*(s)).closure_var).offset((idx) as isize)).is_const()) as i32)) != 0)) as i32)) != 0 { 97 } else { 93 }; continue;
}
// C line 33183
99 => {
vm_block = if ((((idx) >= ((0 as i32))) as i32)) != 0 { 98 } else { 5 }; continue;
}
// C line 33171
100 => {
let _ = { let assigned = get_closure_var(ctx, s, fd, (((JS_CLOSURE_ARG as i32)) as JSClosureTypeEnum), (var_idx).wrapping_sub((536870912 as i32)), var_name, (0 as i32), (0 as i32), (((JS_VAR_NORMAL as i32)) as JSVarKindEnum)); idx = assigned; assigned };
vm_block = 99; continue;
}
// C line 33170
101 => {
let _ = capture_var(fd, core::ptr::addr_of_mut!(*((*(fd)).args).offset(((var_idx).wrapping_sub((536870912 as i32))) as isize)));
vm_block = 100; continue;
}
// C line 33176
102 => {
let _ = { let assigned = get_closure_var(ctx, s, fd, (((JS_CLOSURE_LOCAL as i32)) as JSClosureTypeEnum), var_idx, var_name, (((*((*(fd)).vars).offset((var_idx) as isize)).is_const()) as i32), (((*((*(fd)).vars).offset((var_idx) as isize)).is_lexical()) as i32), (((*((*(fd)).vars).offset((var_idx) as isize)).var_kind()) as JSVarKindEnum)); idx = assigned; assigned };
vm_block = 99; continue;
}
// C line 33175
103 => {
let _ = capture_var(fd, core::ptr::addr_of_mut!(*((*(fd)).vars).offset((var_idx) as isize)));
vm_block = 102; continue;
}
// C line 33169
104 => {
vm_block = if (((var_idx) & ((536870912 as i32)))) != 0 { 101 } else { 103 }; continue;
}
// C line 33072
105 => {
vm_block = if ((((((((var_idx) < ((0 as i32))) as i32)) != 0) && (((*(fd)).is_eval) != 0)) as i32)) != 0 { 52 } else { 104 }; continue;
}
// C line 33071
106 => {
let _ = { let assigned = s; fd = assigned; assigned };
vm_block = 105; continue;
}
// C line 33070
107 => {
vm_block = if ((!(!(fd).is_null()) as i32)) != 0 { 106 } else { 105 }; continue;
}
// C line 32989
108 => {
vm_block = if !((*(fd)).parent).is_null() { 163 } else { 107 }; continue;
}
// C line 33065
109 => {
vm_block = 107; continue;
}
// C line 33064
110 => {
vm_block = if ((*(fd)).is_eval) != 0 { 109 } else { 108 }; continue;
}
// C line 33061
111 => {
let _ = var_object_test(ctx, s, var_name, op, bc, core::ptr::addr_of_mut!(label_done), (0 as i32));
vm_block = 110; continue;
}
// C line 33060
112 => {
let _ = dbuf_put_u16(bc, ((idx) as u16));
vm_block = 111; continue;
}
// C line 33059
113 => {
let _ = dbuf_putc(bc, (((OP_get_var_ref as i32)) as u8));
vm_block = 112; continue;
}
// C line 33056
114 => {
let _ = { let assigned = get_closure_var(ctx, s, fd, (((JS_CLOSURE_LOCAL as i32)) as JSClosureTypeEnum), (*(fd)).arg_var_object_idx, (*(vd)).var_name, (0 as i32), (0 as i32), (((JS_VAR_NORMAL as i32)) as JSVarKindEnum)); idx = assigned; assigned };
vm_block = 113; continue;
}
// C line 33055
115 => {
let _ = capture_var(fd, vd);
vm_block = 114; continue;
}
// C line 33054
116 => {
let _ = { let assigned = core::ptr::addr_of_mut!(*((*(fd)).vars).offset(((*(fd)).arg_var_object_idx) as isize)); vd = assigned; assigned };
vm_block = 115; continue;
}
// C line 33053
117 => {
vm_block = if (((((((((*(fd)).arg_var_object_idx) >= ((0 as i32))) as i32)) != 0) && (((!((is_pseudo_var) != 0) as i32)) != 0)) as i32)) != 0 { 116 } else { 110 }; continue;
}
// C line 33049
118 => {
let _ = var_object_test(ctx, s, var_name, op, bc, core::ptr::addr_of_mut!(label_done), (0 as i32));
vm_block = 117; continue;
}
// C line 33048
119 => {
let _ = dbuf_put_u16(bc, ((idx) as u16));
vm_block = 118; continue;
}
// C line 33047
120 => {
let _ = dbuf_putc(bc, (((OP_get_var_ref as i32)) as u8));
vm_block = 119; continue;
}
// C line 33044
121 => {
let _ = { let assigned = get_closure_var(ctx, s, fd, (((JS_CLOSURE_LOCAL as i32)) as JSClosureTypeEnum), (*(fd)).var_object_idx, (*(vd)).var_name, (0 as i32), (0 as i32), (((JS_VAR_NORMAL as i32)) as JSVarKindEnum)); idx = assigned; assigned };
vm_block = 120; continue;
}
// C line 33043
122 => {
let _ = capture_var(fd, vd);
vm_block = 121; continue;
}
// C line 33042
123 => {
let _ = { let assigned = core::ptr::addr_of_mut!(*((*(fd)).vars).offset(((*(fd)).var_object_idx) as isize)); vd = assigned; assigned };
vm_block = 122; continue;
}
// C line 33041
124 => {
vm_block = if ((((((((((!((is_arg_scope) != 0) as i32)) != 0) && ((((((*(fd)).var_object_idx) >= ((0 as i32))) as i32)) != 0)) as i32)) != 0) && (((!((is_pseudo_var) != 0) as i32)) != 0)) as i32)) != 0 { 123 } else { 117 }; continue;
}
// C line 33037
125 => {
vm_block = 107; continue;
}
// C line 33036
126 => {
let _ = { let assigned = add_func_var(ctx, fd, var_name); var_idx = assigned; assigned };
vm_block = 125; continue;
}
// C line 33034
127 => {
vm_block = if ((((((*(fd)).is_func_expr) != 0) && ((((((*(fd)).func_name) == (var_name)) as i32)) != 0)) as i32)) != 0 { 126 } else { 124 }; continue;
}
// C line 33032
128 => {
vm_block = 107; continue;
}
// C line 33031
129 => {
let _ = { let assigned = add_arguments_var(ctx, fd); var_idx = assigned; assigned };
vm_block = 128; continue;
}
// C line 33030
130 => {
vm_block = if ((((((((var_name) == ((((crate::quickjs_atom::JS_ATOM_arguments as i32)) as JSAtom))) as i32)) != 0) && (((*(fd)).has_arguments_binding) != 0)) as i32)) != 0 { 129 } else { 127 }; continue;
}
// C line 33028
131 => {
vm_block = 107; continue;
}
// C line 33027
132 => {
vm_block = if ((((var_idx) >= ((0 as i32))) as i32)) != 0 { 131 } else { 130 }; continue;
}
// C line 33026
133 => {
let _ = { let assigned = resolve_pseudo_var(ctx, fd, var_name); var_idx = assigned; assigned };
vm_block = 132; continue;
}
// C line 33025
134 => {
vm_block = if (is_pseudo_var) != 0 { 133 } else { 130 }; continue;
}
// C line 33023
135 => {
vm_block = 107; continue;
}
// C line 33022
136 => {
vm_block = if ((((var_idx) >= ((0 as i32))) as i32)) != 0 { 135 } else { 134 }; continue;
}
// C line 33021
137 => {
let _ = { let assigned = find_var(ctx, fd, var_name); var_idx = assigned; assigned };
vm_block = 136; continue;
}
// C line 33020
138 => {
vm_block = if ((!((is_arg_scope) != 0) as i32)) != 0 { 137 } else { 134 }; continue;
}
// C line 33018
139 => {
vm_block = 107; continue;
}
// C line 33017
140 => {
vm_block = if ((((var_idx) >= ((0 as i32))) as i32)) != 0 { 139 } else { 138 }; continue;
}
// C line 33016
141 => {
let _ = { let assigned = (((idx) == (((2 as i32)).wrapping_neg())) as i32); is_arg_scope = assigned; assigned };
vm_block = 140; continue;
}
// C line 32992
142 => {
vm_block = if ((((idx) >= ((0 as i32))) as i32)) != 0 { 160 } else { 141 }; continue;
}
// C line 33014
143 => {
let _ = { let assigned = (*(vd)).scope_next; idx = assigned; assigned };
vm_block = 142; continue;
}
// C line 33004
144 => {
vm_block = 141; continue;
}
// C line 33003
145 => {
let _ = { let assigned = idx; var_idx = assigned; assigned };
vm_block = 144; continue;
}
// C line 33000
146 => {
vm_block = 5; continue;
}
// C line 32999
147 => {
let _ = dbuf_putc(bc, (((0 as i32)) as u8));
vm_block = 146; continue;
}
// C line 32998
148 => {
let _ = dbuf_put_u32(bc, JS_DupAtom(ctx, var_name));
vm_block = 147; continue;
}
// C line 32997
149 => {
let _ = dbuf_putc(bc, (((OP_throw_error as i32)) as u8));
vm_block = 148; continue;
}
// C line 32996
150 => {
vm_block = if ((*(vd)).is_const()) != 0 { 149 } else { 145 }; continue;
}
// C line 32995
151 => {
vm_block = if ((((((((op) == ((OP_scope_put_var as i32))) as i32)) != 0) || (((((op) == ((OP_scope_make_ref as i32))) as i32)) != 0)) as i32)) != 0 { 150 } else { 145 }; continue;
}
// C line 33011
152 => {
let _ = var_object_test(ctx, s, var_name, op, bc, core::ptr::addr_of_mut!(label_done), (1 as i32));
vm_block = 143; continue;
}
// C line 33010
153 => {
let _ = dbuf_put_u16(bc, ((idx) as u16));
vm_block = 152; continue;
}
// C line 33009
154 => {
let _ = dbuf_putc(bc, (((OP_get_var_ref as i32)) as u8));
vm_block = 153; continue;
}
// C line 33008
155 => {
vm_block = if ((((idx) >= ((0 as i32))) as i32)) != 0 { 154 } else { 143 }; continue;
}
// C line 33007
156 => {
let _ = { let assigned = get_closure_var(ctx, s, fd, (((JS_CLOSURE_LOCAL as i32)) as JSClosureTypeEnum), idx, (*(vd)).var_name, (0 as i32), (0 as i32), (((JS_VAR_NORMAL as i32)) as JSVarKindEnum)); idx = assigned; assigned };
vm_block = 155; continue;
}
// C line 33006
157 => {
let _ = capture_var(fd, vd);
vm_block = 156; continue;
}
// C line 33005
158 => {
vm_block = if (((((((((*(vd)).var_name) == ((((crate::quickjs_atom::JS_ATOM__with_ as i32)) as JSAtom))) as i32)) != 0) && (((!((is_pseudo_var) != 0) as i32)) != 0)) as i32)) != 0 { 157 } else { 143 }; continue;
}
// C line 32994
159 => {
vm_block = if (((((*(vd)).var_name) == (var_name)) as i32)) != 0 { 151 } else { 158 }; continue;
}
// C line 32993
160 => {
let _ = { let assigned = core::ptr::addr_of_mut!(*((*(fd)).vars).offset((idx) as isize)); vd = assigned; assigned };
vm_block = 159; continue;
}
// C line 32992
161 => {
let first = (*(*fd).scopes.offset(scope_level as isize)).first;
idx = scope_lookup.find(fd, first, var_name).unwrap_or(first);
vm_block = 142; continue;
}
// C line 32991
162 => {
let _ = { let assigned = (*(fd)).parent; fd = assigned; assigned };
vm_block = 161; continue;
}
// C line 32990
163 => {
let _ = { let assigned = (*(fd)).parent_scope_level; scope_level = assigned; assigned };
vm_block = 162; continue;
}
// C line 32989
164 => {
let _ = { let assigned = s; fd = assigned; assigned };
vm_block = 108; continue;
}
// C line 32985
165 => {
let _ = var_object_test(ctx, s, var_name, op, bc, core::ptr::addr_of_mut!(label_done), (0 as i32));
vm_block = 164; continue;
}
// C line 32984
166 => {
let _ = dbuf_put_u16(bc, (((*(s)).arg_var_object_idx) as u16));
vm_block = 165; continue;
}
// C line 32983
167 => {
let _ = dbuf_putc(bc, (((OP_get_loc as i32)) as u8));
vm_block = 166; continue;
}
// C line 32982
168 => {
vm_block = if (((((((((*(s)).arg_var_object_idx) >= ((0 as i32))) as i32)) != 0) && (((!((is_pseudo_var) != 0) as i32)) != 0)) as i32)) != 0 { 167 } else { 164 }; continue;
}
// C line 32979
169 => {
let _ = var_object_test(ctx, s, var_name, op, bc, core::ptr::addr_of_mut!(label_done), (0 as i32));
vm_block = 168; continue;
}
// C line 32978
170 => {
let _ = dbuf_put_u16(bc, (((*(s)).var_object_idx) as u16));
vm_block = 169; continue;
}
// C line 32977
171 => {
let _ = dbuf_putc(bc, (((OP_get_loc as i32)) as u8));
vm_block = 170; continue;
}
// C line 32976
172 => {
vm_block = if ((((((((((!((is_arg_scope) != 0) as i32)) != 0) && ((((((*(s)).var_object_idx) >= ((0 as i32))) as i32)) != 0)) as i32)) != 0) && (((!((is_pseudo_var) != 0) as i32)) != 0)) as i32)) != 0 { 171 } else { 168 }; continue;
}
// C line 32973
173 => {
vm_block = 5; continue;
}
// C line 32971
174 => {
vm_block = 173; continue;
}
// C line 32970
175 => {
let _ = dbuf_putc(bc, (((OP_push_false as i32)) as u8));
vm_block = 174; continue;
}
// C line 32968
176 => {
vm_block = 173; continue;
}
// C line 32938
177 => {
let _ = dbuf_put_u16(bc, (((var_idx).wrapping_sub((536870912 as i32))) as u16));
vm_block = 176; continue;
}
// C line 32937
178 => {
let _ = dbuf_putc(bc, ((((OP_get_arg as i32)).wrapping_add(is_put)) as u8));
vm_block = 177; continue;
}
// C line 32966
179 => {
let _ = dbuf_put_u16(bc, ((var_idx) as u16));
vm_block = 176; continue;
}
// C line 32945
180 => {
let _ = dbuf_putc(bc, (((OP_put_loc_check_init as i32)) as u8));
vm_block = 179; continue;
}
// C line 32947
181 => {
let _ = dbuf_putc(bc, (((OP_put_loc as i32)) as u8));
vm_block = 179; continue;
}
// C line 32944
182 => {
vm_block = if ((((var_name) == ((((crate::quickjs_atom::JS_ATOM_this as i32)) as JSAtom))) as i32)) != 0 { 180 } else { 181 }; continue;
}
// C line 32949
183 => {
let _ = dbuf_putc(bc, (((OP_put_loc_check as i32)) as u8));
vm_block = 179; continue;
}
// C line 32942
184 => {
vm_block = if ((((op) == ((OP_scope_put_var_init as i32))) as i32)) != 0 { 182 } else { 183 }; continue;
}
// C line 32952
185 => {
let _ = dbuf_putc(bc, (((OP_put_loc as i32)) as u8));
vm_block = 179; continue;
}
// C line 32941
186 => {
vm_block = if ((*((*(s)).vars).offset((var_idx) as isize)).is_lexical()) != 0 { 184 } else { 185 }; continue;
}
// C line 32958
187 => {
let _ = dbuf_putc(bc, (((OP_get_loc_checkthis as i32)) as u8));
vm_block = 179; continue;
}
// C line 32960
188 => {
let _ = dbuf_putc(bc, (((OP_get_loc_check as i32)) as u8));
vm_block = 179; continue;
}
// C line 32956
189 => {
vm_block = if ((((op) == ((OP_scope_get_var_checkthis as i32))) as i32)) != 0 { 187 } else { 188 }; continue;
}
// C line 32963
190 => {
let _ = dbuf_putc(bc, (((OP_get_loc as i32)) as u8));
vm_block = 179; continue;
}
// C line 32955
191 => {
vm_block = if ((*((*(s)).vars).offset((var_idx) as isize)).is_lexical()) != 0 { 189 } else { 190 }; continue;
}
// C line 32940
192 => {
vm_block = if (is_put) != 0 { 186 } else { 191 }; continue;
}
// C line 32936
193 => {
vm_block = if (((var_idx) & ((536870912 as i32)))) != 0 { 178 } else { 192 }; continue;
}
// C line 32935 labels: local_scope_var
194 => {
let _ = { let assigned = (((((((op) == ((OP_scope_put_var as i32))) as i32)) != 0) || (((((op) == ((OP_scope_put_var_init as i32))) as i32)) != 0)) as i32); is_put = assigned; assigned };
vm_block = 193; continue;
}
// C line 32929
195 => {
vm_block = 194; continue;
}
// C line 32928
196 => {
let _ = dbuf_putc(bc, (((OP_undefined as i32)) as u8));
vm_block = 195; continue;
}
// C line 32926
197 => {
vm_block = 194; continue;
}
// C line 32924
198 => {
vm_block = 5; continue;
}
// C line 32923
199 => {
let _ = dbuf_putc(bc, (((OP_drop as i32)) as u8));
vm_block = 198; continue;
}
// C line 32920
200 => {
vm_block = if ((((((!((((var_idx) & ((536870912 as i32)))) != 0) as i32)) != 0) && ((((((((*((*(s)).vars).offset((var_idx) as isize)).var_kind()) as i32)) == ((JS_VAR_FUNCTION_NAME as i32))) as i32)) != 0)) as i32)) != 0 { 199 } else { 197 }; continue;
}
// C line 32918
201 => {
vm_block = 173; continue;
}
// C line 32888
202 => {
let _ = dbuf_put_u32(bc, JS_DupAtom(ctx, var_name));
vm_block = 201; continue;
}
// C line 32887
203 => {
let _ = dbuf_putc(bc, (((OP_push_atom_value as i32)) as u8));
vm_block = 202; continue;
}
// C line 32886
204 => {
let _ = dbuf_put_u32(bc, JS_DupAtom(ctx, var_name));
vm_block = 203; continue;
}
// C line 32885
205 => {
let _ = dbuf_putc(bc, (((OP_define_field as i32)) as u8));
vm_block = 204; continue;
}
// C line 32884
206 => {
let _ = dbuf_put_u16(bc, ((var_idx) as u16));
vm_block = 205; continue;
}
// C line 32883
207 => {
let _ = dbuf_putc(bc, (((OP_get_loc as i32)) as u8));
vm_block = 206; continue;
}
// C line 32882
208 => {
let _ = dbuf_putc(bc, (((OP_object as i32)) as u8));
vm_block = 207; continue;
}
// C line 32901
209 => {
let _ = { let assigned = optimize_scope_make_ref(ctx, s, bc, bc_buf, ls, pos_next, get_op, var_idx); pos_next = assigned; assigned };
vm_block = 201; continue;
}
// C line 32894
210 => {
let _ = { var_idx = ((((var_idx) as i32)).wrapping_sub((536870912 as i32))) as i32; var_idx };
vm_block = 209; continue;
}
// C line 32893
211 => {
let _ = { let assigned = (OP_get_arg as i32); get_op = assigned; assigned };
vm_block = 210; continue;
}
// C line 32897
212 => {
let _ = { let assigned = (OP_get_loc_check as i32); get_op = assigned; assigned };
vm_block = 209; continue;
}
// C line 32899
213 => {
let _ = { let assigned = (OP_get_loc as i32); get_op = assigned; assigned };
vm_block = 209; continue;
}
// C line 32896
214 => {
vm_block = if ((*((*(s)).vars).offset((var_idx) as isize)).is_lexical()) != 0 { 212 } else { 213 }; continue;
}
// C line 32892
215 => {
vm_block = if (((var_idx) & ((536870912 as i32)))) != 0 { 211 } else { 214 }; continue;
}
// C line 32910
216 => {
let _ = dbuf_put_u16(bc, (((var_idx).wrapping_sub((536870912 as i32))) as u16));
vm_block = 201; continue;
}
// C line 32909
217 => {
let _ = dbuf_put_u32(bc, JS_DupAtom(ctx, var_name));
vm_block = 216; continue;
}
// C line 32908
218 => {
let _ = dbuf_putc(bc, (((OP_make_arg_ref as i32)) as u8));
vm_block = 217; continue;
}
// C line 32907
219 => {
let _ = capture_var(s, core::ptr::addr_of_mut!(*((*(s)).args).offset(((var_idx).wrapping_sub((536870912 as i32))) as isize)));
vm_block = 218; continue;
}
// C line 32915
220 => {
let _ = dbuf_put_u16(bc, ((var_idx) as u16));
vm_block = 201; continue;
}
// C line 32914
221 => {
let _ = dbuf_put_u32(bc, JS_DupAtom(ctx, var_name));
vm_block = 220; continue;
}
// C line 32913
222 => {
let _ = dbuf_putc(bc, (((OP_make_loc_ref as i32)) as u8));
vm_block = 221; continue;
}
// C line 32912
223 => {
let _ = capture_var(s, core::ptr::addr_of_mut!(*((*(s)).vars).offset((var_idx) as isize)));
vm_block = 222; continue;
}
// C line 32906
224 => {
vm_block = if (((var_idx) & ((536870912 as i32)))) != 0 { 219 } else { 223 }; continue;
}
// C line 32890
225 => {
vm_block = if ((((((((label_done) == (((1 as i32)).wrapping_neg())) as i32)) != 0) && ((can_opt_put_ref_value(bc_buf, (*(ls)).pos)) != 0)) as i32)) != 0 { 215 } else { 224 }; continue;
}
// C line 32879
226 => {
vm_block = if ((((((!((((var_idx) & ((536870912 as i32)))) != 0) as i32)) != 0) && ((((((((*((*(s)).vars).offset((var_idx) as isize)).var_kind()) as i32)) == ((JS_VAR_FUNCTION_NAME as i32))) as i32)) != 0)) as i32)) != 0 { 208 } else { 225 }; continue;
}
// C line 32877
227 => {
vm_block = match op { x if x == (OP_scope_delete_var as i32) => 175, x if x == (OP_scope_put_var_init as i32) => 194, x if x == (OP_scope_get_var as i32) => 194, x if x == (OP_scope_get_var_undef as i32) => 194, x if x == (OP_scope_get_var_checkthis as i32) => 194, x if x == (OP_scope_get_ref as i32) => 196, x if x == (OP_scope_put_var as i32) => 200, x if x == (OP_scope_make_ref as i32) => 226, _ => 173, }; continue;
}
// C line 32872
228 => {
vm_block = 5; continue;
}
// C line 32871
229 => {
let _ = dbuf_putc(bc, (((0 as i32)) as u8));
vm_block = 228; continue;
}
// C line 32870
230 => {
let _ = dbuf_put_u32(bc, JS_DupAtom(ctx, var_name));
vm_block = 229; continue;
}
// C line 32869
231 => {
let _ = dbuf_putc(bc, (((OP_throw_error as i32)) as u8));
vm_block = 230; continue;
}
// C line 32864
232 => {
vm_block = if ((((((((((((((((op) == ((OP_scope_put_var as i32))) as i32)) != 0) || (((((op) == ((OP_scope_make_ref as i32))) as i32)) != 0)) as i32)) != 0) && (((!((((var_idx) & ((536870912 as i32)))) != 0) as i32)) != 0)) as i32)) != 0) && (((((*((*(s)).vars).offset((var_idx) as isize)).is_const()) as i32)) != 0)) as i32)) != 0 { 231 } else { 227 }; continue;
}
// C line 32863
233 => {
vm_block = if ((((var_idx) >= ((0 as i32))) as i32)) != 0 { 232 } else { 172 }; continue;
}
// C line 32860
234 => {
let _ = { let assigned = add_func_var(ctx, s, var_name); var_idx = assigned; assigned };
vm_block = 233; continue;
}
// C line 32858
235 => {
vm_block = if ((((((((((((var_idx) < ((0 as i32))) as i32)) != 0) && (((*(s)).is_func_expr) != 0)) as i32)) != 0) && (((((var_name) == ((*(s)).func_name)) as i32)) != 0)) as i32)) != 0 { 234 } else { 233 }; continue;
}
// C line 32856
236 => {
let _ = { let assigned = add_arguments_var(ctx, s); var_idx = assigned; assigned };
vm_block = 235; continue;
}
// C line 32853
237 => {
vm_block = if ((((((((((((var_idx) < ((0 as i32))) as i32)) != 0) && (((((var_name) == ((((crate::quickjs_atom::JS_ATOM_arguments as i32)) as JSAtom))) as i32)) != 0)) as i32)) != 0) && (((*(s)).has_arguments_binding) != 0)) as i32)) != 0 { 236 } else { 235 }; continue;
}
// C line 32851
238 => {
let _ = { let assigned = resolve_pseudo_var(ctx, s, var_name); var_idx = assigned; assigned };
vm_block = 237; continue;
}
// C line 32850
239 => {
vm_block = if ((((((((var_idx) < ((0 as i32))) as i32)) != 0) && ((is_pseudo_var) != 0)) as i32)) != 0 { 238 } else { 237 }; continue;
}
// C line 32847
240 => {
let _ = { let assigned = scope_lookup.find_function_var(ctx, s, var_name); var_idx = assigned; assigned };
vm_block = 239; continue;
}
// C line 32846
241 => {
vm_block = if ((!((is_arg_scope) != 0) as i32)) != 0 { 240 } else { 239 }; continue;
}
// C line 32843
242 => {
vm_block = if ((((var_idx) < ((0 as i32))) as i32)) != 0 { 241 } else { 233 }; continue;
}
// C line 32842
243 => {
let _ = { let assigned = (((idx) == (((2 as i32)).wrapping_neg())) as i32); is_arg_scope = assigned; assigned };
vm_block = 242; continue;
}
// C line 32821
244 => {
vm_block = if ((((idx) >= ((0 as i32))) as i32)) != 0 { 259 } else { 243 }; continue;
}
// C line 32840
245 => {
let _ = { let assigned = (*(vd)).scope_next; idx = assigned; assigned };
vm_block = 244; continue;
}
// C line 32833
246 => {
vm_block = 243; continue;
}
// C line 32832
247 => {
let _ = { let assigned = idx; var_idx = assigned; assigned };
vm_block = 246; continue;
}
// C line 32829
248 => {
vm_block = 5; continue;
}
// C line 32828
249 => {
let _ = dbuf_putc(bc, (((0 as i32)) as u8));
vm_block = 248; continue;
}
// C line 32827
250 => {
let _ = dbuf_put_u32(bc, JS_DupAtom(ctx, var_name));
vm_block = 249; continue;
}
// C line 32826
251 => {
let _ = dbuf_putc(bc, (((OP_throw_error as i32)) as u8));
vm_block = 250; continue;
}
// C line 32825
252 => {
vm_block = if ((*(vd)).is_const()) != 0 { 251 } else { 247 }; continue;
}
// C line 32824
253 => {
vm_block = if ((((((((op) == ((OP_scope_put_var as i32))) as i32)) != 0) || (((((op) == ((OP_scope_make_ref as i32))) as i32)) != 0)) as i32)) != 0 { 252 } else { 247 }; continue;
}
// C line 32838
254 => {
let _ = var_object_test(ctx, s, var_name, op, bc, core::ptr::addr_of_mut!(label_done), (1 as i32));
vm_block = 245; continue;
}
// C line 32837
255 => {
let _ = dbuf_put_u16(bc, ((idx) as u16));
vm_block = 254; continue;
}
// C line 32836
256 => {
let _ = dbuf_putc(bc, (((OP_get_loc as i32)) as u8));
vm_block = 255; continue;
}
// C line 32835
257 => {
vm_block = if (((((((((*(vd)).var_name) == ((((crate::quickjs_atom::JS_ATOM__with_ as i32)) as JSAtom))) as i32)) != 0) && (((!((is_pseudo_var) != 0) as i32)) != 0)) as i32)) != 0 { 256 } else { 245 }; continue;
}
// C line 32823
258 => {
vm_block = if (((((*(vd)).var_name) == (var_name)) as i32)) != 0 { 253 } else { 257 }; continue;
}
// C line 32822
259 => {
let _ = { let assigned = core::ptr::addr_of_mut!(*((*(s)).vars).offset((idx) as isize)); vd = assigned; assigned };
vm_block = 258; continue;
}
// C line 32821
260 => {
let first = (*(*s).scopes.offset(scope_level as isize)).first;
idx = scope_lookup.find(s, first, var_name).unwrap_or(first);
vm_block = 244; continue;
}
// C line 32820
261 => {
let _ = { let assigned = ((1 as i32)).wrapping_neg(); var_idx = assigned; assigned };
vm_block = 260; continue;
}
// C line 32814
262 => {
let _ = { let assigned = (((((((((((((((var_name) == ((((crate::quickjs_atom::JS_ATOM_home_object as i32)) as JSAtom))) as i32)) != 0) || (((((var_name) == ((((crate::quickjs_atom::JS_ATOM_this_active_func as i32)) as JSAtom))) as i32)) != 0)) as i32)) != 0) || (((((var_name) == ((((crate::quickjs_atom::JS_ATOM_new_target as i32)) as JSAtom))) as i32)) != 0)) as i32)) != 0) || (((((var_name) == ((((crate::quickjs_atom::JS_ATOM_this as i32)) as JSAtom))) as i32)) != 0)) as i32); is_pseudo_var = assigned; assigned };
vm_block = 261; continue;
}
// C line 32810
263 => {
let _ = { let assigned = ((1 as i32)).wrapping_neg(); label_done = assigned; assigned };
vm_block = 262; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:33281. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn find_private_class_field_all(mut ctx: *mut JSContext, mut fd: *mut JSFunctionDef, mut name: JSAtom, mut scope_level: i32) -> i32 {
let mut vm_local_storage = Vec::<u64>::new();
let mut idx: i32 = 0;
let mut vm_block: usize = 6;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 33292
1 => {
return ((1 as i32)).wrapping_neg();
}
// C line 33287
2 => {
vm_block = if ((((idx) >= ((0 as i32))) as i32)) != 0 { 5 } else { 1 }; continue;
}
// C line 33290
3 => {
let _ = { let assigned = (*((*(fd)).vars).offset((idx) as isize)).scope_next; idx = assigned; assigned };
vm_block = 2; continue;
}
// C line 33289
4 => {
return idx;
}
// C line 33288
5 => {
vm_block = if (((((*((*(fd)).vars).offset((idx) as isize)).var_name) == (name)) as i32)) != 0 { 4 } else { 3 }; continue;
}
// C line 33286
6 => {
let _ = { let assigned = (*((*(fd)).scopes).offset((scope_level) as isize)).first; idx = assigned; assigned };
vm_block = 2; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:33295. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn get_loc_or_ref(mut bc: *mut DynBuf, mut is_ref: i32, mut idx: i32) -> () {
let mut vm_local_storage = Vec::<u64>::new();
let mut vm_block: usize = 4;
loop { match vm_block {
// C line ?
0 => {
return ();
}
// C line 33303
1 => {
let _ = dbuf_put_u16(bc, ((idx) as u16));
vm_block = 0; continue;
}
// C line 33300
2 => {
let _ = dbuf_putc(bc, (((OP_get_var_ref as i32)) as u8));
vm_block = 1; continue;
}
// C line 33302
3 => {
let _ = dbuf_putc(bc, (((OP_get_loc as i32)) as u8));
vm_block = 1; continue;
}
// C line 33299
4 => {
vm_block = if (is_ref) != 0 { 2 } else { 3 }; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:33306. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn resolve_scope_private_field1(mut ctx: *mut JSContext, mut pis_ref: *mut i32, mut pvar_kind: *mut i32, mut s: *mut JSFunctionDef, mut var_name: JSAtom, mut scope_level: i32) -> i32 {
let mut vm_local_storage = Vec::<u64>::new();
let mut idx: i32 = 0;
let mut var_kind: i32 = 0;
let mut fd: *mut JSFunctionDef = core::ptr::null_mut();
let mut is_ref: i32 = 0;
let mut cv: *mut JSClosureVar = core::ptr::null_mut();
let mut vm_block: usize = 34;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 33365
1 => {
return idx;
}
// C line 33364
2 => {
let _ = { let assigned = var_kind; *(pvar_kind) = assigned; assigned };
vm_block = 1; continue;
}
// C line 33363 labels: done
3 => {
let _ = { let assigned = is_ref; *(pis_ref) = assigned; assigned };
vm_block = 2; continue;
}
// C line 33317
4 => {
vm_block = 32; continue;
}
// C line 33360
5 => {
let _ = { let assigned = (1 as i32); is_ref = assigned; assigned };
vm_block = 4; continue;
}
// C line 33356
6 => {
return ((1 as i32)).wrapping_neg();
}
// C line 33354
7 => {
let _ = __JS_ThrowSyntaxErrorAtom(ctx, var_name, c"undefined private field '%s'".as_ptr());
vm_block = 6; continue;
}
// C line 33334
8 => {
vm_block = if ((((idx) < ((*(fd)).closure_var_count)) as i32)) != 0 { 18 } else { 7 }; continue;
}
// C line 33334
9 => {
let _ = { let old = idx; idx = (idx).wrapping_add(1); old };
vm_block = 8; continue;
}
// C line 33349
10 => {
vm_block = 3; continue;
}
// C line 33347
11 => {
return ((1 as i32)).wrapping_neg();
}
// C line 33346
12 => {
vm_block = if ((((idx) < ((0 as i32))) as i32)) != 0 { 11 } else { 10 }; continue;
}
// C line 33340
13 => {
let _ = { let assigned = get_closure_var(ctx, s, fd, (((JS_CLOSURE_REF as i32)) as JSClosureTypeEnum), idx, (*(cv)).var_name, (((*(cv)).is_const()) as i32), (((*(cv)).is_lexical()) as i32), (((*(cv)).var_kind()) as JSVarKindEnum)); idx = assigned; assigned };
vm_block = 12; continue;
}
// C line 33339
14 => {
vm_block = if ((((fd) != (s)) as i32)) != 0 { 13 } else { 10 }; continue;
}
// C line 33338
15 => {
let _ = { let assigned = (1 as i32); is_ref = assigned; assigned };
vm_block = 14; continue;
}
// C line 33337
16 => {
let _ = { let assigned = (((*(cv)).var_kind()) as i32); var_kind = assigned; assigned };
vm_block = 15; continue;
}
// C line 33336
17 => {
vm_block = if (((((*(cv)).var_name) == (var_name)) as i32)) != 0 { 16 } else { 9 }; continue;
}
// C line 33335
18 => {
cv = core::ptr::addr_of_mut!(*((*(fd)).closure_var).offset((idx) as isize));
vm_block = 17; continue;
}
// C line 33334
19 => {
let _ = { let assigned = (0 as i32); idx = assigned; assigned };
vm_block = 8; continue;
}
// C line 33332
20 => {
vm_block = if ((*(fd)).is_eval) != 0 { 19 } else { 7 }; continue;
}
// C line 33358
21 => {
let _ = { let assigned = (*(fd)).parent; fd = assigned; assigned };
vm_block = 5; continue;
}
// C line 33331
22 => {
vm_block = if ((!(!((*(fd)).parent).is_null()) as i32)) != 0 { 20 } else { 21 }; continue;
}
// C line 33330
23 => {
let _ = { let assigned = (*(fd)).parent_scope_level; scope_level = assigned; assigned };
vm_block = 22; continue;
}
// C line 33328
24 => {
vm_block = 3; continue;
}
// C line 33326
25 => {
return ((1 as i32)).wrapping_neg();
}
// C line 33325
26 => {
vm_block = if ((((idx) < ((0 as i32))) as i32)) != 0 { 25 } else { 24 }; continue;
}
// C line 33323
27 => {
let _ = { let assigned = get_closure_var(ctx, s, fd, (((JS_CLOSURE_LOCAL as i32)) as JSClosureTypeEnum), idx, var_name, (1 as i32), (1 as i32), (((JS_VAR_NORMAL as i32)) as JSVarKindEnum)); idx = assigned; assigned };
vm_block = 26; continue;
}
// C line 33322
28 => {
let _ = capture_var(fd, core::ptr::addr_of_mut!(*((*(fd)).vars).offset((idx) as isize)));
vm_block = 27; continue;
}
// C line 33321
29 => {
vm_block = if (is_ref) != 0 { 28 } else { 24 }; continue;
}
// C line 33320
30 => {
let _ = { let assigned = (((*((*(fd)).vars).offset((idx) as isize)).var_kind()) as i32); var_kind = assigned; assigned };
vm_block = 29; continue;
}
// C line 33319
31 => {
vm_block = if ((((idx) >= ((0 as i32))) as i32)) != 0 { 30 } else { 23 }; continue;
}
// C line 33318
32 => {
let _ = { let assigned = find_private_class_field_all(ctx, fd, var_name, scope_level); idx = assigned; assigned };
vm_block = 31; continue;
}
// C line 33316
33 => {
let _ = { let assigned = (0 as i32); is_ref = assigned; assigned };
vm_block = 4; continue;
}
// C line 33315
34 => {
let _ = { let assigned = s; fd = assigned; assigned };
vm_block = 33; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:33369. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn resolve_scope_private_field(mut ctx: *mut JSContext, mut s: *mut JSFunctionDef, mut var_name: JSAtom, mut scope_level: i32, mut op: i32, mut bc: *mut DynBuf) -> i32 {
let mut vm_local_storage = Vec::<u64>::new();
let mut idx: i32 = 0;
let mut var_kind: i32 = 0;
let mut is_ref: i32 = 0;
let mut setter_name: JSAtom = 0;
let mut vm_block: usize = 61;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 33466
1 => {
return (0 as i32);
}
// C line 33464
2 => {
let _ = std::process::abort();
vm_block = 1; continue;
}
// C line 33462
3 => {
vm_block = 1; continue;
}
// C line 33461
4 => {
let _ = dbuf_putc(bc, (((OP_private_in as i32)) as u8));
vm_block = 3; continue;
}
// C line 33460
5 => {
let _ = get_loc_or_ref(bc, is_ref, idx);
vm_block = 4; continue;
}
// C line 33458
6 => {
vm_block = 1; continue;
}
// C line 33456
7 => {
let _ = std::process::abort();
vm_block = 6; continue;
}
// C line 33454
8 => {
vm_block = 6; continue;
}
// C line 33452
9 => {
let _ = dbuf_putc(bc, (((OP_drop as i32)) as u8));
vm_block = 8; continue;
}
// C line 33451
10 => {
let _ = dbuf_put_u16(bc, (((1 as i32)) as u16));
vm_block = 9; continue;
}
// C line 33450
11 => {
let _ = dbuf_putc(bc, (((OP_call_method as i32)) as u8));
vm_block = 10; continue;
}
// C line 33448
12 => {
let _ = dbuf_putc(bc, (((OP_rot3l as i32)) as u8));
vm_block = 11; continue;
}
// C line 33447
13 => {
let _ = dbuf_putc(bc, (((OP_check_brand as i32)) as u8));
vm_block = 12; continue;
}
// C line 33445
14 => {
let _ = dbuf_putc(bc, (((OP_rot3r as i32)) as u8));
vm_block = 13; continue;
}
// C line 33443
15 => {
let _ = dbuf_putc(bc, (((OP_swap as i32)) as u8));
vm_block = 14; continue;
}
// C line 33442
16 => {
let _ = get_loc_or_ref(bc, is_ref, idx);
vm_block = 15; continue;
}
// C line 33441
17 => {
let _ = if ((((!(((((var_kind) == ((JS_VAR_PRIVATE_SETTER as i32))) as i32)) != 0) as i32)) as i64)) != 0 { std::process::abort() } else { { let _ = (0 as i32); } };
vm_block = 16; continue;
}
// C line 33440
18 => {
return ((1 as i32)).wrapping_neg();
}
// C line 33439
19 => {
vm_block = if ((((idx) < ((0 as i32))) as i32)) != 0 { 18 } else { 17 }; continue;
}
// C line 33438
20 => {
let _ = JS_FreeAtom(ctx, setter_name);
vm_block = 19; continue;
}
// C line 33435
21 => {
let _ = { let assigned = resolve_scope_private_field1(ctx, core::ptr::addr_of_mut!(is_ref), core::ptr::addr_of_mut!(var_kind), s, setter_name, scope_level); idx = assigned; assigned };
vm_block = 20; continue;
}
// C line 33434
22 => {
return ((1 as i32)).wrapping_neg();
}
// C line 33433
23 => {
vm_block = if ((((setter_name) == ((((0 as i32)) as JSAtom))) as i32)) != 0 { 22 } else { 21 }; continue;
}
// C line 33432
24 => {
setter_name = get_private_setter_name(ctx, var_name);
vm_block = 23; continue;
}
// C line 33428
25 => {
vm_block = 6; continue;
}
// C line 33427
26 => {
let _ = dbuf_putc(bc, (((0 as i32)) as u8));
vm_block = 25; continue;
}
// C line 33426
27 => {
let _ = dbuf_put_u32(bc, JS_DupAtom(ctx, var_name));
vm_block = 26; continue;
}
// C line 33425
28 => {
let _ = dbuf_putc(bc, (((OP_throw_error as i32)) as u8));
vm_block = 27; continue;
}
// C line 33421
29 => {
vm_block = 6; continue;
}
// C line 33420
30 => {
let _ = dbuf_putc(bc, (((OP_put_private_field as i32)) as u8));
vm_block = 29; continue;
}
// C line 33419
31 => {
let _ = get_loc_or_ref(bc, is_ref, idx);
vm_block = 30; continue;
}
// C line 33417
32 => {
vm_block = match var_kind { x if x == (JS_VAR_PRIVATE_GETTER_SETTER as i32) => 24, x if x == (JS_VAR_PRIVATE_SETTER as i32) => 24, x if x == (JS_VAR_PRIVATE_GETTER as i32) => 28, x if x == (JS_VAR_PRIVATE_METHOD as i32) => 28, x if x == (JS_VAR_PRIVATE_FIELD as i32) => 31, _ => 7, }; continue;
}
// C line 33415
33 => {
vm_block = 1; continue;
}
// C line 33413
34 => {
let _ = std::process::abort();
vm_block = 33; continue;
}
// C line 33411
35 => {
vm_block = 33; continue;
}
// C line 33410
36 => {
let _ = dbuf_putc(bc, (((0 as i32)) as u8));
vm_block = 35; continue;
}
// C line 33409
37 => {
let _ = dbuf_put_u32(bc, JS_DupAtom(ctx, var_name));
vm_block = 36; continue;
}
// C line 33408
38 => {
let _ = dbuf_putc(bc, (((OP_throw_error as i32)) as u8));
vm_block = 37; continue;
}
// C line 33405
39 => {
vm_block = 33; continue;
}
// C line 33404
40 => {
let _ = dbuf_put_u16(bc, (((0 as i32)) as u16));
vm_block = 39; continue;
}
// C line 33403
41 => {
let _ = dbuf_putc(bc, (((OP_call_method as i32)) as u8));
vm_block = 40; continue;
}
// C line 33402
42 => {
let _ = dbuf_putc(bc, (((OP_check_brand as i32)) as u8));
vm_block = 41; continue;
}
// C line 33401
43 => {
let _ = get_loc_or_ref(bc, is_ref, idx);
vm_block = 42; continue;
}
// C line 33400
44 => {
let _ = dbuf_putc(bc, (((OP_dup as i32)) as u8));
vm_block = 43; continue;
}
// C line 33399
45 => {
vm_block = if ((((op) == ((OP_scope_get_private_field2 as i32))) as i32)) != 0 { 44 } else { 43 }; continue;
}
// C line 33396
46 => {
vm_block = 33; continue;
}
// C line 33395
47 => {
let _ = dbuf_putc(bc, (((OP_nip as i32)) as u8));
vm_block = 46; continue;
}
// C line 33394
48 => {
vm_block = if ((((op) != ((OP_scope_get_private_field2 as i32))) as i32)) != 0 { 47 } else { 46 }; continue;
}
// C line 33393
49 => {
let _ = dbuf_putc(bc, (((OP_check_brand as i32)) as u8));
vm_block = 48; continue;
}
// C line 33392
50 => {
let _ = get_loc_or_ref(bc, is_ref, idx);
vm_block = 49; continue;
}
// C line 33390
51 => {
vm_block = 33; continue;
}
// C line 33389
52 => {
let _ = dbuf_putc(bc, (((OP_get_private_field as i32)) as u8));
vm_block = 51; continue;
}
// C line 33388
53 => {
let _ = get_loc_or_ref(bc, is_ref, idx);
vm_block = 52; continue;
}
// C line 33387
54 => {
let _ = dbuf_putc(bc, (((OP_dup as i32)) as u8));
vm_block = 53; continue;
}
// C line 33386
55 => {
vm_block = if ((((op) == ((OP_scope_get_private_field2 as i32))) as i32)) != 0 { 54 } else { 53 }; continue;
}
// C line 33384
56 => {
vm_block = match var_kind { x if x == (JS_VAR_PRIVATE_SETTER as i32) => 38, x if x == (JS_VAR_PRIVATE_GETTER_SETTER as i32) => 45, x if x == (JS_VAR_PRIVATE_GETTER as i32) => 45, x if x == (JS_VAR_PRIVATE_METHOD as i32) => 50, x if x == (JS_VAR_PRIVATE_FIELD as i32) => 55, _ => 34, }; continue;
}
// C line 33381
57 => {
vm_block = match op { x if x == (OP_scope_in_private_field as i32) => 5, x if x == (OP_scope_put_private_field as i32) => 32, x if x == (OP_scope_get_private_field2 as i32) => 56, x if x == (OP_scope_get_private_field as i32) => 56, _ => 2, }; continue;
}
// C line 33380
58 => {
let _ = if ((((!(((((var_kind) != ((JS_VAR_NORMAL as i32))) as i32)) != 0) as i32)) as i64)) != 0 { std::process::abort() } else { { let _ = (0 as i32); } };
vm_block = 57; continue;
}
// C line 33379
59 => {
return ((1 as i32)).wrapping_neg();
}
// C line 33378
60 => {
vm_block = if ((((idx) < ((0 as i32))) as i32)) != 0 { 59 } else { 58 }; continue;
}
// C line 33376
61 => {
let _ = { let assigned = resolve_scope_private_field1(ctx, core::ptr::addr_of_mut!(is_ref), core::ptr::addr_of_mut!(var_kind), s, var_name, scope_level); idx = assigned; assigned };
vm_block = 60; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:33469. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn mark_eval_captured_variables(mut ctx: *mut JSContext, mut s: *mut JSFunctionDef, mut scope_level: i32) -> () {
let mut vm_local_storage = Vec::<u64>::new();
let mut idx: i32 = 0;
let mut vd: *mut JSVarDef = core::ptr::null_mut();
let mut vm_block: usize = 5;
loop { match vm_block {
// C line ?
0 => {
return ();
}
// C line 33475
1 => {
vm_block = if ((((idx) >= ((0 as i32))) as i32)) != 0 { 4 } else { 0 }; continue;
}
// C line 33478
2 => {
let _ = { let assigned = (*(vd)).scope_next; idx = assigned; assigned };
vm_block = 1; continue;
}
// C line 33477
3 => {
let _ = capture_var(s, vd);
vm_block = 2; continue;
}
// C line 33476
4 => {
let _ = { let assigned = core::ptr::addr_of_mut!(*((*(s)).vars).offset((idx) as isize)); vd = assigned; assigned };
vm_block = 3; continue;
}
// C line 33475
5 => {
let _ = { let assigned = (*((*(s)).scopes).offset((scope_level) as isize)).first; idx = assigned; assigned };
vm_block = 1; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:33483. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn is_var_in_arg_scope(mut var_name: JSAtom, mut var_kind: JSVarKindEnum) -> i32 {
let mut vm_local_storage = Vec::<u64>::new();
let mut vm_block: usize = 1;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 33485
1 => {
return (((((((((((((((((((((((var_name) == ((((crate::quickjs_atom::JS_ATOM_home_object as i32)) as JSAtom))) as i32)) != 0) || (((((var_name) == ((((crate::quickjs_atom::JS_ATOM_this_active_func as i32)) as JSAtom))) as i32)) != 0)) as i32)) != 0) || (((((var_name) == ((((crate::quickjs_atom::JS_ATOM_new_target as i32)) as JSAtom))) as i32)) != 0)) as i32)) != 0) || (((((var_name) == ((((crate::quickjs_atom::JS_ATOM_this as i32)) as JSAtom))) as i32)) != 0)) as i32)) != 0) || (((((var_name) == ((((crate::quickjs_atom::JS_ATOM__arg_var_ as i32)) as JSAtom))) as i32)) != 0)) as i32)) != 0) || (((((((var_kind) as u32)) == ((((JS_VAR_FUNCTION_NAME as i32)) as u32))) as i32)) != 0)) as i32);
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:33493. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn add_eval_variables(mut ctx: *mut JSContext, mut s: *mut JSFunctionDef) -> () {
let mut vm_local_storage = Vec::<u64>::new();
let mut fd: *mut JSFunctionDef = core::ptr::null_mut();
let mut vd: *mut JSVarDef = core::ptr::null_mut();
let mut i: i32 = 0;
let mut scope_level: i32 = 0;
let mut scope_idx: i32 = 0;
let mut has_arguments_binding: i32 = 0;
let mut has_this_binding: i32 = 0;
let mut is_arg_scope: i32 = 0;
let mut idx: i32 = 0;
let mut cv: *mut JSClosureVar = core::ptr::null_mut();
let mut vm_block: usize = 90;
loop { match vm_block {
// C line ?
0 => {
return ();
}
// C line 33557
1 => {
vm_block = 56; continue;
}
// C line 33633
2 => {
vm_block = if ((((idx) < ((*(fd)).closure_var_count)) as i32)) != 0 { 6 } else { 1 }; continue;
}
// C line 33633
3 => {
let _ = { let old = idx; idx = (idx).wrapping_add(1); old };
vm_block = 2; continue;
}
// C line 33640
4 => {
let _ = get_closure_var(ctx, s, fd, (((JS_CLOSURE_REF as i32)) as JSClosureTypeEnum), idx, (*(cv)).var_name, (((*(cv)).is_const()) as i32), (((*(cv)).is_lexical()) as i32), (((*(cv)).var_kind()) as JSVarKindEnum));
vm_block = 3; continue;
}
// C line 33637
5 => {
vm_block = if (((((((((((((((*(cv)).closure_type()) as i32)) != ((JS_CLOSURE_GLOBAL_REF as i32))) as i32)) != 0) && ((((((((*(cv)).closure_type()) as i32)) != ((JS_CLOSURE_GLOBAL_DECL as i32))) as i32)) != 0)) as i32)) != 0) && ((((((((*(cv)).closure_type()) as i32)) != ((JS_CLOSURE_GLOBAL as i32))) as i32)) != 0)) as i32)) != 0 { 4 } else { 3 }; continue;
}
// C line 33634
6 => {
cv = core::ptr::addr_of_mut!(*((*(fd)).closure_var).offset((idx) as isize));
vm_block = 5; continue;
}
// C line 33633
7 => {
let _ = { let assigned = (0 as i32); idx = assigned; assigned };
vm_block = 2; continue;
}
// C line 33629
8 => {
vm_block = if ((*(fd)).is_eval) != 0 { 7 } else { 1 }; continue;
}
// C line 33605
9 => {
vm_block = if ((((i) < ((*(fd)).var_count)) as i32)) != 0 { 14 } else { 8 }; continue;
}
// C line 33605
10 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 9; continue;
}
// C line 33612
11 => {
let _ = get_closure_var(ctx, s, fd, (((JS_CLOSURE_LOCAL as i32)) as JSClosureTypeEnum), i, (*(vd)).var_name, (0 as i32), (((*(vd)).is_lexical()) as i32), (((JS_VAR_NORMAL as i32)) as JSVarKindEnum));
vm_block = 10; continue;
}
// C line 33611
12 => {
let _ = capture_var(fd, vd);
vm_block = 11; continue;
}
// C line 33608
13 => {
vm_block = if (((((((((((((*(vd)).scope_level) == ((0 as i32))) as i32)) != 0) && ((((((*(vd)).var_name) != ((((crate::quickjs_atom::JS_ATOM__ret_ as i32)) as JSAtom))) as i32)) != 0)) as i32)) != 0) && ((((((*(vd)).var_name) != ((((0 as i32)) as JSAtom))) as i32)) != 0)) as i32)) != 0 { 12 } else { 10 }; continue;
}
// C line 33606
14 => {
let _ = { let assigned = core::ptr::addr_of_mut!(*((*(fd)).vars).offset((i) as isize)); vd = assigned; assigned };
vm_block = 13; continue;
}
// C line 33605
15 => {
let _ = { let assigned = (0 as i32); i = assigned; assigned };
vm_block = 9; continue;
}
// C line 33596
16 => {
vm_block = if ((((i) < ((*(fd)).arg_count)) as i32)) != 0 { 21 } else { 15 }; continue;
}
// C line 33596
17 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 16; continue;
}
// C line 33600
18 => {
let _ = get_closure_var(ctx, s, fd, (((JS_CLOSURE_ARG as i32)) as JSClosureTypeEnum), i, (*(vd)).var_name, (0 as i32), (((*(vd)).is_lexical()) as i32), (((JS_VAR_NORMAL as i32)) as JSVarKindEnum));
vm_block = 17; continue;
}
// C line 33599
19 => {
let _ = capture_var(fd, vd);
vm_block = 18; continue;
}
// C line 33598
20 => {
vm_block = if (((((*(vd)).var_name) != ((((0 as i32)) as JSAtom))) as i32)) != 0 { 19 } else { 17 }; continue;
}
// C line 33597
21 => {
let _ = { let assigned = core::ptr::addr_of_mut!(*((*(fd)).args).offset((i) as isize)); vd = assigned; assigned };
vm_block = 20; continue;
}
// C line 33596
22 => {
let _ = { let assigned = (0 as i32); i = assigned; assigned };
vm_block = 16; continue;
}
// C line 33618
23 => {
vm_block = if ((((i) < ((*(fd)).var_count)) as i32)) != 0 { 28 } else { 8 }; continue;
}
// C line 33618
24 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 23; continue;
}
// C line 33623
25 => {
let _ = get_closure_var(ctx, s, fd, (((JS_CLOSURE_LOCAL as i32)) as JSClosureTypeEnum), i, (*(vd)).var_name, (0 as i32), (((*(vd)).is_lexical()) as i32), (((JS_VAR_NORMAL as i32)) as JSVarKindEnum));
vm_block = 24; continue;
}
// C line 33622
26 => {
let _ = capture_var(fd, vd);
vm_block = 25; continue;
}
// C line 33621
27 => {
vm_block = if (((((((((*(vd)).scope_level) == ((0 as i32))) as i32)) != 0) && ((is_var_in_arg_scope((*(vd)).var_name, (((*(vd)).var_kind()) as JSVarKindEnum))) != 0)) as i32)) != 0 { 26 } else { 24 }; continue;
}
// C line 33619
28 => {
let _ = { let assigned = core::ptr::addr_of_mut!(*((*(fd)).vars).offset((i) as isize)); vd = assigned; assigned };
vm_block = 27; continue;
}
// C line 33618
29 => {
let _ = { let assigned = (0 as i32); i = assigned; assigned };
vm_block = 23; continue;
}
// C line 33593
30 => {
vm_block = if ((!((is_arg_scope) != 0) as i32)) != 0 { 22 } else { 29 }; continue;
}
// C line 33592
31 => {
let _ = { let assigned = (((scope_idx) == (((2 as i32)).wrapping_neg())) as i32); is_arg_scope = assigned; assigned };
vm_block = 30; continue;
}
// C line 33585
32 => {
vm_block = if ((((scope_idx) >= ((0 as i32))) as i32)) != 0 { 36 } else { 31 }; continue;
}
// C line 33590
33 => {
let _ = { let assigned = (*(vd)).scope_next; scope_idx = assigned; assigned };
vm_block = 32; continue;
}
// C line 33588
34 => {
let _ = get_closure_var(ctx, s, fd, (((JS_CLOSURE_LOCAL as i32)) as JSClosureTypeEnum), scope_idx, (*(vd)).var_name, (((*(vd)).is_const()) as i32), (((*(vd)).is_lexical()) as i32), (((*(vd)).var_kind()) as JSVarKindEnum));
vm_block = 33; continue;
}
// C line 33587
35 => {
let _ = capture_var(fd, vd);
vm_block = 34; continue;
}
// C line 33586
36 => {
let _ = { let assigned = core::ptr::addr_of_mut!(*((*(fd)).vars).offset((scope_idx) as isize)); vd = assigned; assigned };
vm_block = 35; continue;
}
// C line 33584
37 => {
let _ = { let assigned = (*((*(fd)).scopes).offset((scope_level) as isize)).first; scope_idx = assigned; assigned };
vm_block = 32; continue;
}
// C line 33581
38 => {
let _ = add_func_var(ctx, fd, (*(fd)).func_name);
vm_block = 37; continue;
}
// C line 33580
39 => {
vm_block = if ((((((*(fd)).is_func_expr) != 0) && ((((((*(fd)).func_name) != ((((0 as i32)) as JSAtom))) as i32)) != 0)) as i32)) != 0 { 38 } else { 37 }; continue;
}
// C line 33577
40 => {
let _ = { let assigned = (1 as i32); has_arguments_binding = assigned; assigned };
vm_block = 39; continue;
}
// C line 33576
41 => {
let _ = add_arguments_var(ctx, fd);
vm_block = 40; continue;
}
// C line 33575
42 => {
vm_block = if ((((((!((has_arguments_binding) != 0) as i32)) != 0) && (((*(fd)).has_arguments_binding) != 0)) as i32)) != 0 { 41 } else { 39 }; continue;
}
// C line 33572
43 => {
let _ = { let assigned = (1 as i32); has_this_binding = assigned; assigned };
vm_block = 42; continue;
}
// C line 33571
44 => {
let _ = { let assigned = add_var(ctx, fd, (((crate::quickjs_atom::JS_ATOM_home_object as i32)) as JSAtom)); (*(fd)).home_object_var_idx = assigned; assigned };
vm_block = 43; continue;
}
// C line 33570
45 => {
vm_block = if ((((((*(fd)).has_home_object) != 0) && ((((((*(fd)).home_object_var_idx) < ((0 as i32))) as i32)) != 0)) as i32)) != 0 { 44 } else { 43 }; continue;
}
// C line 33569
46 => {
let _ = { let assigned = add_var(ctx, fd, (((crate::quickjs_atom::JS_ATOM_this_active_func as i32)) as JSAtom)); (*(fd)).this_active_func_var_idx = assigned; assigned };
vm_block = 45; continue;
}
// C line 33568
47 => {
vm_block = if ((((((*(fd)).is_derived_class_constructor) != 0) && ((((((*(fd)).this_active_func_var_idx) < ((0 as i32))) as i32)) != 0)) as i32)) != 0 { 46 } else { 45 }; continue;
}
// C line 33567
48 => {
let _ = { let assigned = add_var(ctx, fd, (((crate::quickjs_atom::JS_ATOM_new_target as i32)) as JSAtom)); (*(fd)).new_target_var_idx = assigned; assigned };
vm_block = 47; continue;
}
// C line 33566
49 => {
vm_block = if (((((*(fd)).new_target_var_idx) < ((0 as i32))) as i32)) != 0 { 48 } else { 47 }; continue;
}
// C line 33565
50 => {
let _ = { let assigned = add_var_this(ctx, fd); (*(fd)).this_var_idx = assigned; assigned };
vm_block = 49; continue;
}
// C line 33564
51 => {
vm_block = if (((((*(fd)).this_var_idx) < ((0 as i32))) as i32)) != 0 { 50 } else { 49 }; continue;
}
// C line 33563
52 => {
vm_block = if ((((((!((has_this_binding) != 0) as i32)) != 0) && (((*(fd)).has_this_binding) != 0)) as i32)) != 0 { 51 } else { 42 }; continue;
}
// C line 33561
53 => {
vm_block = 0; continue;
}
// C line 33560
54 => {
vm_block = if ((!(!(fd).is_null()) as i32)) != 0 { 53 } else { 52 }; continue;
}
// C line 33559
55 => {
let _ = { let assigned = (*(fd)).parent; fd = assigned; assigned };
vm_block = 54; continue;
}
// C line 33558
56 => {
let _ = { let assigned = (*(fd)).parent_scope_level; scope_level = assigned; assigned };
vm_block = 55; continue;
}
// C line 33556
57 => {
let _ = { let assigned = s; fd = assigned; assigned };
vm_block = 1; continue;
}
// C line 33553
58 => {
let _ = if ((((!(((((((*(s)).is_eval) != 0) || ((((((*(s)).closure_var_count) == ((0 as i32))) as i32)) != 0)) as i32)) != 0) as i32)) as i64)) != 0 { std::process::abort() } else { { let _ = (0 as i32); } };
vm_block = 57; continue;
}
// C line 33539
59 => {
vm_block = if ((((i) < ((*(s)).var_count)) as i32)) != 0 { 63 } else { 58 }; continue;
}
// C line 33539
60 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 59; continue;
}
// C line 33545
61 => {
let _ = capture_var(s, vd);
vm_block = 60; continue;
}
// C line 33542
62 => {
vm_block = if (((((((((((((*(vd)).scope_level) == ((0 as i32))) as i32)) != 0) && ((((((*(vd)).var_name) != ((((crate::quickjs_atom::JS_ATOM__ret_ as i32)) as JSAtom))) as i32)) != 0)) as i32)) != 0) && ((((((*(vd)).var_name) != ((((0 as i32)) as JSAtom))) as i32)) != 0)) as i32)) != 0 { 61 } else { 60 }; continue;
}
// C line 33540
63 => {
let _ = { let assigned = core::ptr::addr_of_mut!(*((*(s)).vars).offset((i) as isize)); vd = assigned; assigned };
vm_block = 62; continue;
}
// C line 33539
64 => {
let _ = { let assigned = (0 as i32); i = assigned; assigned };
vm_block = 59; continue;
}
// C line 33535
65 => {
vm_block = if ((((i) < ((*(s)).arg_count)) as i32)) != 0 { 68 } else { 64 }; continue;
}
// C line 33535
66 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 65; continue;
}
// C line 33537
67 => {
let _ = capture_var(s, vd);
vm_block = 66; continue;
}
// C line 33536
68 => {
let _ = { let assigned = core::ptr::addr_of_mut!(*((*(s)).args).offset((i) as isize)); vd = assigned; assigned };
vm_block = 67; continue;
}
// C line 33535
69 => {
let _ = { let assigned = (0 as i32); i = assigned; assigned };
vm_block = 65; continue;
}
// C line 33533
70 => {
let _ = add_func_var(ctx, s, (*(s)).func_name);
vm_block = 69; continue;
}
// C line 33532
71 => {
vm_block = if ((((((*(s)).is_func_expr) != 0) && ((((((*(s)).func_name) != ((((0 as i32)) as JSAtom))) as i32)) != 0)) as i32)) != 0 { 70 } else { 69 }; continue;
}
// C line 33530
72 => {
let _ = add_arguments_arg(ctx, s);
vm_block = 71; continue;
}
// C line 33529
73 => {
vm_block = if ((((((*(s)).has_parameter_expressions) != 0) && (((!(((((((*(s)).js_mode) as i32)) & (((1 as i32)).wrapping_shl(((0 as i32)) as u32)))) != 0) as i32)) != 0)) as i32)) != 0 { 72 } else { 71 }; continue;
}
// C line 33525
74 => {
let _ = add_arguments_var(ctx, s);
vm_block = 73; continue;
}
// C line 33524
75 => {
vm_block = if (has_arguments_binding) != 0 { 74 } else { 71 }; continue;
}
// C line 33523
76 => {
let _ = { let assigned = (*(s)).has_arguments_binding; has_arguments_binding = assigned; assigned };
vm_block = 75; continue;
}
// C line 33521
77 => {
let _ = { let assigned = add_var(ctx, s, (((crate::quickjs_atom::JS_ATOM_home_object as i32)) as JSAtom)); (*(s)).home_object_var_idx = assigned; assigned };
vm_block = 76; continue;
}
// C line 33520
78 => {
vm_block = if ((((((*(s)).has_home_object) != 0) && ((((((*(s)).home_object_var_idx) < ((0 as i32))) as i32)) != 0)) as i32)) != 0 { 77 } else { 76 }; continue;
}
// C line 33519
79 => {
let _ = { let assigned = add_var(ctx, s, (((crate::quickjs_atom::JS_ATOM_this_active_func as i32)) as JSAtom)); (*(s)).this_active_func_var_idx = assigned; assigned };
vm_block = 78; continue;
}
// C line 33518
80 => {
vm_block = if ((((((*(s)).is_derived_class_constructor) != 0) && ((((((*(s)).this_active_func_var_idx) < ((0 as i32))) as i32)) != 0)) as i32)) != 0 { 79 } else { 78 }; continue;
}
// C line 33517
81 => {
let _ = { let assigned = add_var(ctx, s, (((crate::quickjs_atom::JS_ATOM_new_target as i32)) as JSAtom)); (*(s)).new_target_var_idx = assigned; assigned };
vm_block = 80; continue;
}
// C line 33516
82 => {
vm_block = if (((((*(s)).new_target_var_idx) < ((0 as i32))) as i32)) != 0 { 81 } else { 80 }; continue;
}
// C line 33515
83 => {
let _ = { let assigned = add_var_this(ctx, s); (*(s)).this_var_idx = assigned; assigned };
vm_block = 82; continue;
}
// C line 33514
84 => {
vm_block = if (((((*(s)).this_var_idx) < ((0 as i32))) as i32)) != 0 { 83 } else { 82 }; continue;
}
// C line 33513
85 => {
vm_block = if (has_this_binding) != 0 { 84 } else { 76 }; continue;
}
// C line 33512
86 => {
let _ = { let assigned = (*(s)).has_this_binding; has_this_binding = assigned; assigned };
vm_block = 85; continue;
}
// C line 33507
87 => {
let _ = { let assigned = add_var(ctx, s, (((crate::quickjs_atom::JS_ATOM__arg_var_ as i32)) as JSAtom)); (*(s)).arg_var_object_idx = assigned; assigned };
vm_block = 86; continue;
}
// C line 33504
88 => {
vm_block = if ((*(s)).has_parameter_expressions) != 0 { 87 } else { 86 }; continue;
}
// C line 33503
89 => {
let _ = { let assigned = add_var(ctx, s, (((crate::quickjs_atom::JS_ATOM__var_ as i32)) as JSAtom)); (*(s)).var_object_idx = assigned; assigned };
vm_block = 88; continue;
}
// C line 33502
90 => {
vm_block = if ((((((!(((*(s)).is_eval) != 0) as i32)) != 0) && (((!(((((((*(s)).js_mode) as i32)) & (((1 as i32)).wrapping_shl(((0 as i32)) as u32)))) != 0) as i32)) != 0)) as i32)) != 0 { 89 } else { 86 }; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:33650. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn set_closure_from_var(mut ctx: *mut JSContext, mut cv: *mut JSClosureVar, mut vd: *mut JSBytecodeVarDef, mut var_idx: i32) -> () {
let mut vm_local_storage = Vec::<u64>::new();
let mut vm_block: usize = 6;
loop { match vm_block {
// C line ?
0 => {
return ();
}
// C line 33658
1 => {
let _ = { let assigned = JS_DupAtom(ctx, (*(vd)).var_name); (*(cv)).var_name = assigned; assigned };
vm_block = 0; continue;
}
// C line 33657
2 => {
let _ = { let assigned = ((var_idx) as u16); (*(cv)).var_idx = assigned; assigned };
vm_block = 1; continue;
}
// C line 33656
3 => {
let _ = { let assigned = (*(vd)).var_kind(); (*(cv)).set_var_kind((assigned) as _); assigned };
vm_block = 2; continue;
}
// C line 33655
4 => {
let _ = { let assigned = (*(vd)).is_lexical(); (*(cv)).set_is_lexical((assigned) as _); assigned };
vm_block = 3; continue;
}
// C line 33654
5 => {
let _ = { let assigned = (*(vd)).is_const(); (*(cv)).set_is_const((assigned) as _); assigned };
vm_block = 4; continue;
}
// C line 33653
6 => {
let _ = { let assigned = (((JS_CLOSURE_LOCAL as i32)) as JSClosureTypeEnum); (*(cv)).set_closure_type((assigned) as _); assigned };
vm_block = 5; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:33663. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn add_closure_variables(mut ctx: *mut JSContext, mut s: *mut JSFunctionDef, mut b: *mut JSFunctionBytecode, mut scope_idx: i32) -> i32 {
let mut vm_local_storage = Vec::<u64>::new();
let mut i: i32 = 0;
let mut count: i32 = 0;
let mut vd: *mut JSBytecodeVarDef = core::ptr::null_mut();
let mut is_arg_scope: i32 = 0;
let mut cv: *mut JSClosureVar = core::ptr::null_mut();
let mut cv_1: *mut JSClosureVar = core::ptr::null_mut();
let mut cv_2: *mut JSClosureVar = core::ptr::null_mut();
let mut cv_3: *mut JSClosureVar = core::ptr::null_mut();
let mut cv0: *mut JSClosureVar = core::ptr::null_mut();
let mut cv_4: *mut JSClosureVar = core::ptr::null_mut();
let mut vm_block: usize = 59;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 33745
1 => {
return (0 as i32);
}
// C line 33719
2 => {
vm_block = if ((((i) < ((*(b)).closure_var_count)) as i32)) != 0 { 15 } else { 1 }; continue;
}
// C line 33719
3 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 2; continue;
}
// C line 33743
4 => {
let _ = { let assigned = JS_DupAtom(ctx, (*(cv0)).var_name); (*(cv_4)).var_name = assigned; assigned };
vm_block = 3; continue;
}
// C line 33742
5 => {
let _ = { let assigned = ((i) as u16); (*(cv_4)).var_idx = assigned; assigned };
vm_block = 4; continue;
}
// C line 33741
6 => {
let _ = { let assigned = (*(cv0)).var_kind(); (*(cv_4)).set_var_kind((assigned) as _); assigned };
vm_block = 5; continue;
}
// C line 33740
7 => {
let _ = { let assigned = (*(cv0)).is_lexical(); (*(cv_4)).set_is_lexical((assigned) as _); assigned };
vm_block = 6; continue;
}
// C line 33739
8 => {
let _ = { let assigned = (*(cv0)).is_const(); (*(cv_4)).set_is_const((assigned) as _); assigned };
vm_block = 7; continue;
}
// C line 33738
9 => {
let _ = { let assigned = (((JS_CLOSURE_REF as i32)) as JSClosureTypeEnum); (*(cv_4)).set_closure_type((assigned) as _); assigned };
vm_block = 8; continue;
}
// C line 33737
10 => {
let _ = { let assigned = core::ptr::addr_of_mut!(*((*(s)).closure_var).offset(({ let old = (*(s)).closure_var_count; (*(s)).closure_var_count = ((*(s)).closure_var_count).wrapping_add(1); old }) as isize)); cv_4 = assigned; assigned };
vm_block = 9; continue;
}
// C line 33735
11 => {
let _ = std::process::abort();
vm_block = 10; continue;
}
// C line 33733
12 => {
vm_block = 3; continue;
}
// C line 33729
13 => {
vm_block = 10; continue;
}
// C line 33723
14 => {
vm_block = match (((*(cv0)).closure_type()) as i32) { x if x == (JS_CLOSURE_GLOBAL as i32) => 12, x if x == (JS_CLOSURE_GLOBAL_DECL as i32) => 12, x if x == (JS_CLOSURE_GLOBAL_REF as i32) => 12, x if x == (JS_CLOSURE_MODULE_IMPORT as i32) => 13, x if x == (JS_CLOSURE_MODULE_DECL as i32) => 13, x if x == (JS_CLOSURE_REF as i32) => 13, x if x == (JS_CLOSURE_ARG as i32) => 13, x if x == (JS_CLOSURE_LOCAL as i32) => 13, _ => 11, }; continue;
}
// C line 33720
15 => {
cv0 = core::ptr::addr_of_mut!(*((*(b)).closure_var).offset((i) as isize));
vm_block = 14; continue;
}
// C line 33719
16 => {
let _ = { let assigned = (0 as i32); i = assigned; assigned };
vm_block = 2; continue;
}
// C line 33702
17 => {
vm_block = if ((((i) < ((((*(b)).var_count) as i32))) as i32)) != 0 { 22 } else { 16 }; continue;
}
// C line 33702
18 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 17; continue;
}
// C line 33706
19 => {
let _ = set_closure_from_var(ctx, cv_2, vd, i);
vm_block = 18; continue;
}
// C line 33705
20 => {
cv_2 = core::ptr::addr_of_mut!(*((*(s)).closure_var).offset(({ let old = (*(s)).closure_var_count; (*(s)).closure_var_count = ((*(s)).closure_var_count).wrapping_add(1); old }) as isize));
vm_block = 19; continue;
}
// C line 33704
21 => {
vm_block = if ((((((!(((*(vd)).has_scope()) != 0) as i32)) != 0) && ((((((*(vd)).var_name) != ((((crate::quickjs_atom::JS_ATOM__ret_ as i32)) as JSAtom))) as i32)) != 0)) as i32)) != 0 { 20 } else { 18 }; continue;
}
// C line 33703
22 => {
let _ = { let assigned = core::ptr::addr_of_mut!(*((*(b)).vardefs).offset((((((*(b)).arg_count) as i32)).wrapping_add(i)) as isize)); vd = assigned; assigned };
vm_block = 21; continue;
}
// C line 33702
23 => {
let _ = { let assigned = (0 as i32); i = assigned; assigned };
vm_block = 17; continue;
}
// C line 33691
24 => {
vm_block = if ((((i) < ((((*(b)).arg_count) as i32))) as i32)) != 0 { 33 } else { 23 }; continue;
}
// C line 33691
25 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 24; continue;
}
// C line 33699
26 => {
let _ = { let assigned = JS_DupAtom(ctx, (*(vd)).var_name); (*(cv_1)).var_name = assigned; assigned };
vm_block = 25; continue;
}
// C line 33698
27 => {
let _ = { let assigned = ((i) as u16); (*(cv_1)).var_idx = assigned; assigned };
vm_block = 26; continue;
}
// C line 33697
28 => {
let _ = { let assigned = (((JS_VAR_NORMAL as i32)) as u8); (*(cv_1)).set_var_kind((assigned) as _); assigned };
vm_block = 27; continue;
}
// C line 33696
29 => {
let _ = { let assigned = (((0 as i32)) as u8); (*(cv_1)).set_is_lexical((assigned) as _); assigned };
vm_block = 28; continue;
}
// C line 33695
30 => {
let _ = { let assigned = (((0 as i32)) as u8); (*(cv_1)).set_is_const((assigned) as _); assigned };
vm_block = 29; continue;
}
// C line 33694
31 => {
let _ = { let assigned = (((JS_CLOSURE_ARG as i32)) as JSClosureTypeEnum); (*(cv_1)).set_closure_type((assigned) as _); assigned };
vm_block = 30; continue;
}
// C line 33693
32 => {
let _ = { let assigned = core::ptr::addr_of_mut!(*((*(b)).vardefs).offset((i) as isize)); vd = assigned; assigned };
vm_block = 31; continue;
}
// C line 33692
33 => {
cv_1 = core::ptr::addr_of_mut!(*((*(s)).closure_var).offset(({ let old = (*(s)).closure_var_count; (*(s)).closure_var_count = ((*(s)).closure_var_count).wrapping_add(1); old }) as isize));
vm_block = 32; continue;
}
// C line 33691
34 => {
let _ = { let assigned = (0 as i32); i = assigned; assigned };
vm_block = 24; continue;
}
// C line 33711
35 => {
vm_block = if ((((i) < ((((*(b)).var_count) as i32))) as i32)) != 0 { 40 } else { 16 }; continue;
}
// C line 33711
36 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 35; continue;
}
// C line 33715
37 => {
let _ = set_closure_from_var(ctx, cv_3, vd, i);
vm_block = 36; continue;
}
// C line 33714
38 => {
cv_3 = core::ptr::addr_of_mut!(*((*(s)).closure_var).offset(({ let old = (*(s)).closure_var_count; (*(s)).closure_var_count = ((*(s)).closure_var_count).wrapping_add(1); old }) as isize));
vm_block = 37; continue;
}
// C line 33713
39 => {
vm_block = if ((((((!(((*(vd)).has_scope()) != 0) as i32)) != 0) && ((is_var_in_arg_scope((*(vd)).var_name, (((*(vd)).var_kind()) as JSVarKindEnum))) != 0)) as i32)) != 0 { 38 } else { 36 }; continue;
}
// C line 33712
40 => {
let _ = { let assigned = core::ptr::addr_of_mut!(*((*(b)).vardefs).offset((((((*(b)).arg_count) as i32)).wrapping_add(i)) as isize)); vd = assigned; assigned };
vm_block = 39; continue;
}
// C line 33711
41 => {
let _ = { let assigned = (0 as i32); i = assigned; assigned };
vm_block = 35; continue;
}
// C line 33689
42 => {
vm_block = if ((!((is_arg_scope) != 0) as i32)) != 0 { 34 } else { 41 }; continue;
}
// C line 33688
43 => {
let _ = { let assigned = (((i) == (((2 as i32)).wrapping_neg())) as i32); is_arg_scope = assigned; assigned };
vm_block = 42; continue;
}
// C line 33680
44 => {
vm_block = if ((((i) >= ((0 as i32))) as i32)) != 0 { 49 } else { 43 }; continue;
}
// C line 33686
45 => {
let _ = { let assigned = (*(vd)).scope_next; i = assigned; assigned };
vm_block = 44; continue;
}
// C line 33684
46 => {
let _ = set_closure_from_var(ctx, cv, vd, i);
vm_block = 45; continue;
}
// C line 33683
47 => {
cv = core::ptr::addr_of_mut!(*((*(s)).closure_var).offset(({ let old = (*(s)).closure_var_count; (*(s)).closure_var_count = ((*(s)).closure_var_count).wrapping_add(1); old }) as isize));
vm_block = 46; continue;
}
// C line 33682
48 => {
vm_block = if ((*(vd)).has_scope()) != 0 { 47 } else { 45 }; continue;
}
// C line 33681
49 => {
let _ = { let assigned = core::ptr::addr_of_mut!(*((*(b)).vardefs).offset((((((*(b)).arg_count) as i32)).wrapping_add(i)) as isize)); vd = assigned; assigned };
vm_block = 48; continue;
}
// C line 33680
50 => {
let _ = { let assigned = scope_idx; i = assigned; assigned };
vm_block = 44; continue;
}
// C line 33678
51 => {
return ((1 as i32)).wrapping_neg();
}
// C line 33677
52 => {
vm_block = if ((!(!((*(s)).closure_var).is_null()) as i32)) != 0 { 51 } else { 50 }; continue;
}
// C line 33676
53 => {
let _ = { let assigned = ((js_malloc(ctx, ((size_of::<JSClosureVar>() as usize)).wrapping_mul(((count) as usize)))) as *mut JSClosureVar); (*(s)).closure_var = assigned; assigned };
vm_block = 52; continue;
}
// C line 33675
54 => {
return (0 as i32);
}
// C line 33674
55 => {
vm_block = if ((((count) == ((0 as i32))) as i32)) != 0 { 54 } else { 53 }; continue;
}
// C line 33673
56 => {
let _ = { let assigned = count; (*(s)).closure_var_size = assigned; assigned };
vm_block = 55; continue;
}
// C line 33672
57 => {
let _ = { let assigned = (0 as i32); (*(s)).closure_var_count = assigned; assigned };
vm_block = 56; continue;
}
// C line 33671
58 => {
let _ = { let assigned = core::ptr::null_mut::<JSClosureVar>(); (*(s)).closure_var = assigned; assigned };
vm_block = 57; continue;
}
// C line 33670
59 => {
let _ = { let assigned = (((((*(b)).arg_count) as i32)).wrapping_add((((*(b)).var_count) as i32))).wrapping_add((*(b)).closure_var_count); count = assigned; assigned };
vm_block = 58; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:33888. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn instantiate_hoisted_definitions(mut ctx: *mut JSContext, mut s: *mut JSFunctionDef, mut bc: *mut DynBuf) -> () {
// quickjs.c:33932-33957 scans the same immutable closure array for every
// global. Preserve its first matching-name/variable-environment precedence.
let mut closure_positions = std::collections::HashMap::new();
let mut first_environment = (*s).closure_var_count;
for index in 0..(*s).closure_var_count {
    let name = (*(*s).closure_var.offset(index as isize)).var_name;
    closure_positions.entry(name).or_insert(index);
    if name == crate::quickjs_atom::JS_ATOM__var_ || name == crate::quickjs_atom::JS_ATOM__arg_var_ {
        first_environment = first_environment.min(index);
    }
}
let mut vm_local_storage = Vec::<u64>::new();
let mut i: i32 = 0;
let mut idx: i32 = 0;
let mut label_next: i32 = 0;
let mut vd: *mut JSVarDef = core::ptr::null_mut();
let mut vd_1: *mut JSVarDef = core::ptr::null_mut();
let mut hf: *mut JSGlobalVar = core::ptr::null_mut();
let mut has_var_obj: i32 = 0;
let mut force_init: i32 = 0;
let mut cv: *mut JSClosureVar = core::ptr::null_mut();
let mut vm_block: usize = 73;
loop { match vm_block {
// C line ?
0 => {
return ();
}
// C line 33992
1 => {
let _ = { let assigned = (0 as i32); (*(s)).global_var_size = assigned; assigned };
vm_block = 0; continue;
}
// C line 33991
2 => {
let _ = { let assigned = (0 as i32); (*(s)).global_var_count = assigned; assigned };
vm_block = 1; continue;
}
// C line 33990
3 => {
let _ = { let assigned = core::ptr::null_mut::<JSGlobalVar>(); (*(s)).global_vars = assigned; assigned };
vm_block = 2; continue;
}
// C line 33989
4 => {
let _ = js_free(ctx, (((*(s)).global_vars) as *mut c_void));
vm_block = 3; continue;
}
// C line 33986
5 => {
let _ = { let assigned = (((*(bc)).size) as i32); (*((*(s)).label_slots).offset((label_next) as isize)).pos2 = assigned; assigned };
vm_block = 4; continue;
}
// C line 33985
6 => {
let _ = dbuf_put_u32(bc, ((label_next) as u32));
vm_block = 5; continue;
}
// C line 33984
7 => {
let _ = dbuf_putc(bc, (((OP_label as i32)) as u8));
vm_block = 6; continue;
}
// C line 33982
8 => {
let _ = dbuf_putc(bc, (((OP_return_undef as i32)) as u8));
vm_block = 7; continue;
}
// C line 33981
9 => {
vm_block = if !((*(s)).module).is_null() { 8 } else { 4 }; continue;
}
// C line 33932
10 => {
vm_block = if ((((i) < ((*(s)).global_var_count)) as i32)) != 0 { 43 } else { 9 }; continue;
}
// C line 33932
11 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 10; continue;
}
// C line 33978
12 => {
let _ = JS_FreeAtom(ctx, (*(hf)).var_name);
vm_block = 11; continue;
}
// C line 33971
13 => {
let _ = dbuf_put_u16(bc, ((idx) as u16));
vm_block = 12; continue;
}
// C line 33970
14 => {
let _ = dbuf_putc(bc, (((OP_put_var_ref as i32)) as u8));
vm_block = 13; continue;
}
// C line 33975
15 => {
let _ = dbuf_putc(bc, (((OP_drop as i32)) as u8));
vm_block = 12; continue;
}
// C line 33974
16 => {
let _ = dbuf_put_u32(bc, JS_DupAtom(ctx, (*(hf)).var_name));
vm_block = 15; continue;
}
// C line 33973
17 => {
let _ = dbuf_putc(bc, (((OP_define_field as i32)) as u8));
vm_block = 16; continue;
}
// C line 33969
18 => {
vm_block = if ((!((has_var_obj) != 0) as i32)) != 0 { 14 } else { 17 }; continue;
}
// C line 33964
19 => {
let _ = dbuf_put_u32(bc, JS_DupAtom(ctx, (((crate::quickjs_atom::JS_ATOM_default as i32)) as JSAtom)));
vm_block = 18; continue;
}
// C line 33963
20 => {
let _ = dbuf_putc(bc, (((OP_set_name as i32)) as u8));
vm_block = 19; continue;
}
// C line 33961
21 => {
vm_block = if (((((*(hf)).var_name) == ((((crate::quickjs_atom::JS_ATOM__default_ as i32)) as JSAtom))) as i32)) != 0 { 20 } else { 18 }; continue;
}
// C line 33960
22 => {
let _ = dbuf_put_u32(bc, (((*(hf)).cpool_idx) as u32));
vm_block = 21; continue;
}
// C line 33959
23 => {
let _ = dbuf_putc(bc, (((OP_fclosure as i32)) as u8));
vm_block = 22; continue;
}
// C line 33967
24 => {
let _ = dbuf_putc(bc, (((OP_undefined as i32)) as u8));
vm_block = 18; continue;
}
// C line 33958
25 => {
vm_block = if (((((*(hf)).cpool_idx) >= ((0 as i32))) as i32)) != 0 { 23 } else { 24 }; continue;
}
// C line 33957 labels: closure_found
26 => {
vm_block = if (((((((((*(hf)).cpool_idx) >= ((0 as i32))) as i32)) != 0) || ((force_init) != 0)) as i32)) != 0 { 25 } else { 12 }; continue;
}
// C line 33955
27 => {
let _ = std::process::abort();
vm_block = 26; continue;
}
// C line 33940
28 => {
vm_block = if ((((idx) < ((*(s)).closure_var_count)) as i32)) != 0 { 39 } else { 27 }; continue;
}
// C line 33940
29 => {
let _ = { let old = idx; idx = (idx).wrapping_add(1); old };
vm_block = 28; continue;
}
// C line 33952
30 => {
vm_block = 26; continue;
}
// C line 33951
31 => {
let _ = { let assigned = (1 as i32); force_init = assigned; assigned };
vm_block = 30; continue;
}
// C line 33950
32 => {
let _ = { let assigned = (1 as i32); has_var_obj = assigned; assigned };
vm_block = 31; continue;
}
// C line 33949
33 => {
let _ = dbuf_put_u16(bc, ((idx) as u16));
vm_block = 32; continue;
}
// C line 33948
34 => {
let _ = dbuf_putc(bc, (((OP_get_var_ref as i32)) as u8));
vm_block = 33; continue;
}
// C line 33946
35 => {
vm_block = if (((((((((*(cv)).var_name) == ((((crate::quickjs_atom::JS_ATOM__var_ as i32)) as JSAtom))) as i32)) != 0) || ((((((*(cv)).var_name) == ((((crate::quickjs_atom::JS_ATOM__arg_var_ as i32)) as JSAtom))) as i32)) != 0)) as i32)) != 0 { 34 } else { 29 }; continue;
}
// C line 33944
36 => {
vm_block = 26; continue;
}
// C line 33943
37 => {
let _ = { let assigned = (0 as i32); force_init = assigned; assigned };
vm_block = 36; continue;
}
// C line 33942
38 => {
vm_block = if (((((*(cv)).var_name) == ((*(hf)).var_name)) as i32)) != 0 { 37 } else { 35 }; continue;
}
// C line 33941
39 => {
cv = core::ptr::addr_of_mut!(*((*(s)).closure_var).offset((idx) as isize));
vm_block = 38; continue;
}
// C line 33940
40 => {
idx = closure_positions.get(&(*hf).var_name).copied().unwrap_or((*s).closure_var_count).min(first_environment);
vm_block = 28; continue;
}
// C line 33935
41 => {
force_init = (((*(hf)).force_init()) as i32);
vm_block = 40; continue;
}
// C line 33934
42 => {
has_var_obj = (0 as i32);
vm_block = 41; continue;
}
// C line 33933
43 => {
hf = core::ptr::addr_of_mut!(*((*(s)).global_vars).offset((i) as isize));
vm_block = 42; continue;
}
// C line 33932
44 => {
let _ = { let assigned = (0 as i32); i = assigned; assigned };
vm_block = 10; continue;
}
// C line 33926
45 => {
let _ = { let old = (*(s)).jump_size; (*(s)).jump_size = ((*(s)).jump_size).wrapping_add(1); old };
vm_block = 44; continue;
}
// C line 33925
46 => {
let _ = update_label(s, label_next, (1 as i32));
vm_block = 45; continue;
}
// C line 33924
47 => {
let _ = dbuf_put_u32(bc, ((label_next) as u32));
vm_block = 46; continue;
}
// C line 33923
48 => {
let _ = dbuf_putc(bc, (((OP_if_false as i32)) as u8));
vm_block = 47; continue;
}
// C line 33922
49 => {
let _ = dbuf_putc(bc, (((OP_push_this as i32)) as u8));
vm_block = 48; continue;
}
// C line 33919
50 => {
return;
}
// C line 33918
51 => {
let _ = dbuf_set_error(bc);
vm_block = 50; continue;
}
// C line 33917
52 => {
vm_block = if ((((label_next) < ((0 as i32))) as i32)) != 0 { 51 } else { 49 }; continue;
}
// C line 33916
53 => {
let _ = { let assigned = new_label_fd(s); label_next = assigned; assigned };
vm_block = 52; continue;
}
// C line 33915
54 => {
vm_block = if !((*(s)).module).is_null() { 53 } else { 44 }; continue;
}
// C line 33902
55 => {
vm_block = if ((((i) < ((*(s)).var_count)) as i32)) != 0 { 62 } else { 54 }; continue;
}
// C line 33902
56 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 55; continue;
}
// C line 33908
57 => {
let _ = dbuf_put_u16(bc, ((i) as u16));
vm_block = 56; continue;
}
// C line 33907
58 => {
let _ = dbuf_putc(bc, (((OP_put_loc as i32)) as u8));
vm_block = 57; continue;
}
// C line 33906
59 => {
let _ = dbuf_put_u32(bc, (((*(vd_1)).func_pool_idx) as u32));
vm_block = 58; continue;
}
// C line 33905
60 => {
let _ = dbuf_putc(bc, (((OP_fclosure as i32)) as u8));
vm_block = 59; continue;
}
// C line 33904
61 => {
vm_block = if (((((((((*(vd_1)).scope_level) == ((0 as i32))) as i32)) != 0) && ((((((*(vd_1)).func_pool_idx) >= ((0 as i32))) as i32)) != 0)) as i32)) != 0 { 60 } else { 56 }; continue;
}
// C line 33903
62 => {
vd_1 = core::ptr::addr_of_mut!(*((*(s)).vars).offset((i) as isize));
vm_block = 61; continue;
}
// C line 33902
63 => {
let _ = { let assigned = (0 as i32); i = assigned; assigned };
vm_block = 55; continue;
}
// C line 33893
64 => {
vm_block = if ((((i) < ((*(s)).arg_count)) as i32)) != 0 { 71 } else { 63 }; continue;
}
// C line 33893
65 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 64; continue;
}
// C line 33899
66 => {
let _ = dbuf_put_u16(bc, ((i) as u16));
vm_block = 65; continue;
}
// C line 33898
67 => {
let _ = dbuf_putc(bc, (((OP_put_arg as i32)) as u8));
vm_block = 66; continue;
}
// C line 33897
68 => {
let _ = dbuf_put_u32(bc, (((*(vd)).func_pool_idx) as u32));
vm_block = 67; continue;
}
// C line 33896
69 => {
let _ = dbuf_putc(bc, (((OP_fclosure as i32)) as u8));
vm_block = 68; continue;
}
// C line 33895
70 => {
vm_block = if (((((*(vd)).func_pool_idx) >= ((0 as i32))) as i32)) != 0 { 69 } else { 65 }; continue;
}
// C line 33894
71 => {
vd = core::ptr::addr_of_mut!(*((*(s)).args).offset((i) as isize));
vm_block = 70; continue;
}
// C line 33893
72 => {
let _ = { let assigned = (0 as i32); i = assigned; assigned };
vm_block = 64; continue;
}
// C line 33890
73 => {
label_next = ((1 as i32)).wrapping_neg();
vm_block = 72; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:33995. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn skip_dead_code(mut s: *mut JSFunctionDef, mut bc_buf: *const u8, mut bc_len: i32, mut pos: i32, mut linep: *mut i32) -> i32 {
let mut vm_local_storage = Vec::<u64>::new();
let mut op: i32 = 0;
let mut len: i32 = 0;
let mut label: i32 = 0;
let mut atom: JSAtom = 0;
let mut vm_block: usize = 2;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 34042
1 => {
return pos;
}
// C line 34000
2 => {
vm_block = if ((((pos) < (bc_len)) as i32)) != 0 { 22 } else { 1 }; continue;
}
// C line 34000
3 => {
let _ = { pos = ((((pos) as i32)).wrapping_add(len)) as i32; pos };
vm_block = 2; continue;
}
// C line 34004
4 => {
let _ = { let assigned = ((get_u32(((bc_buf).offset(((pos) as isize))).offset((((1 as i32)) as isize)))) as i32); *(linep) = assigned; assigned };
vm_block = 3; continue;
}
// C line 34016
5 => {
let _ = if ((((!((((((*((*(s)).label_slots).offset((label) as isize)).first_reloc) == (core::ptr::null_mut::<RelocEntry>())) as i32)) != 0) as i32)) as i64)) != 0 { std::process::abort() } else { { let _ = (0 as i32); } };
vm_block = 3; continue;
}
// C line 34009
6 => {
vm_block = 1; continue;
}
// C line 34008
7 => {
vm_block = if ((((update_label(s, label, (0 as i32))) > ((0 as i32))) as i32)) != 0 { 6 } else { 5 }; continue;
}
// C line 34007
8 => {
let _ = { let assigned = ((get_u32(((bc_buf).offset(((pos) as isize))).offset((((1 as i32)) as isize)))) as i32); label = assigned; assigned };
vm_block = 7; continue;
}
// C line 34038
9 => {
vm_block = 3; continue;
}
// C line 34036
10 => {
vm_block = 3; continue;
}
// C line 34035
11 => {
let _ = JS_FreeAtom((*(s)).ctx, atom);
vm_block = 10; continue;
}
// C line 34034
12 => {
let _ = { let assigned = get_u32(((bc_buf).offset(((pos) as isize))).offset((((1 as i32)) as isize))); atom = assigned; assigned };
vm_block = 11; continue;
}
// C line 34029
13 => {
let _ = update_label(s, label, ((1 as i32)).wrapping_neg());
vm_block = 12; continue;
}
// C line 34028
14 => {
let _ = { let assigned = ((get_u32(((bc_buf).offset(((pos) as isize))).offset((((5 as i32)) as isize)))) as i32); label = assigned; assigned };
vm_block = 13; continue;
}
// C line 34025
15 => {
vm_block = 3; continue;
}
// C line 34024
16 => {
let _ = update_label(s, label, ((1 as i32)).wrapping_neg());
vm_block = 15; continue;
}
// C line 34023
17 => {
let _ = { let assigned = ((get_u32(((bc_buf).offset(((pos) as isize))).offset((((1 as i32)) as isize)))) as i32); label = assigned; assigned };
vm_block = 16; continue;
}
// C line 34020
18 => {
vm_block = match (((*((opcode_info).as_ptr()).offset((op) as isize)).fmt) as i32) { x if x == (OP_FMT_atom_u16 as i32) => 12, x if x == (OP_FMT_atom_u8 as i32) => 12, x if x == (OP_FMT_atom as i32) => 12, x if x == (OP_FMT_atom_label_u16 as i32) => 14, x if x == (OP_FMT_atom_label_u8 as i32) => 14, x if x == (OP_FMT_label_u16 as i32) => 17, x if x == (OP_FMT_label as i32) => 17, _ => 9, }; continue;
}
// C line 34006
19 => {
vm_block = if ((((op) == ((OP_label as i32))) as i32)) != 0 { 8 } else { 18 }; continue;
}
// C line 34003
20 => {
vm_block = if ((((op) == ((OP_line_num as i32))) as i32)) != 0 { 4 } else { 19 }; continue;
}
// C line 34002
21 => {
let _ = { let assigned = (((*((opcode_info).as_ptr()).offset((op) as isize)).size) as i32); len = assigned; assigned };
vm_block = 20; continue;
}
// C line 34001
22 => {
let _ = { let assigned = ((*(bc_buf).offset((pos) as isize)) as i32); op = assigned; assigned };
vm_block = 21; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:34045. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn get_label_pos(mut s: *mut JSFunctionDef, mut label: i32) -> i32 {
let mut vm_local_storage = Vec::<u64>::new();
let mut i: i32 = 0;
let mut pos: i32 = 0;
let mut vm_block: usize = 13;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 34065
1 => {
return pos;
}
// C line 34048
2 => {
vm_block = if ((((i) < ((20 as i32))) as i32)) != 0 { 12 } else { 1 }; continue;
}
// C line 34048
3 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 2; continue;
}
// C line 34050
4 => {
vm_block = 11; continue;
}
// C line 34062
5 => {
vm_block = 3; continue;
}
// C line 34060
6 => {
return pos;
}
// C line 34058
7 => {
vm_block = 5; continue;
}
// C line 34057
8 => {
let _ = { let assigned = ((get_u32(((((*(s)).byte_code).buf).offset(((pos) as isize))).offset((((1 as i32)) as isize)))) as i32); label = assigned; assigned };
vm_block = 7; continue;
}
// C line 34055
9 => {
vm_block = 4; continue;
}
// C line 34054
10 => {
let _ = { pos = ((((pos) as i32)).wrapping_add((5 as i32))) as i32; pos };
vm_block = 9; continue;
}
// C line 34051
11 => {
vm_block = match ((*(((*(s)).byte_code).buf).offset((pos) as isize)) as i32) { x if x == (OP_goto as i32) => 8, x if x == (OP_label as i32) => 10, x if x == (OP_line_num as i32) => 10, _ => 6, }; continue;
}
// C line 34049
12 => {
let _ = { let assigned = (*((*(s)).label_slots).offset((label) as isize)).pos; pos = assigned; assigned };
vm_block = 4; continue;
}
// C line 34048
13 => {
let _ = { let assigned = (0 as i32); i = assigned; assigned };
vm_block = 2; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:34070. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn resolve_variables(mut ctx: *mut JSContext, mut s: *mut JSFunctionDef) -> i32 {
let mut scope_lookup = CompilerScopeLookup::default();
let mut vm_local_storage = Vec::<u64>::new();
let mut pos: i32 = 0;
let mut pos_next: i32 = 0;
let mut bc_len: i32 = 0;
let mut op: i32 = 0;
let mut len: i32 = 0;
let mut line_num: i32 = 0;
let mut i: i32 = 0;
let mut idx: i32 = 0;
let mut bc_buf: *mut u8 = core::ptr::null_mut();
let mut var_name: JSAtom = 0;
let mut bc_out: DynBuf = core::mem::zeroed();
let mut cc: CodeContext = core::mem::zeroed();
let mut scope: i32 = 0;
let mut hf: *mut JSGlobalVar = core::ptr::null_mut();
let mut cv: *mut JSClosureVar = core::ptr::null_mut();
let mut call_argc: i32 = 0;
let mut label: i32 = 0;
let mut ls: *mut LabelSlot = core::ptr::null_mut();
let mut ret: i32 = 0;
let mut label_1: i32 = 0;
let mut ls_1: *mut LabelSlot = core::ptr::null_mut();
let mut pos1: i32 = 0;
let mut line1: i32 = 0;
let mut line: i32 = 0;
let mut label_2: i32 = 0;
let mut ls_2: *mut LabelSlot = core::ptr::null_mut();
let mut scope_idx: i32 = 0;
let mut scope_1: i32 = 0;
let mut vd: *mut JSVarDef = core::ptr::null_mut();
let mut scope_idx_1: i32 = 0;
let mut scope_2: i32 = 0;
let mut vd_1: *mut JSVarDef = core::ptr::null_mut();
let mut name: JSAtom = 0;
let mut lab0: i32 = 0;
let mut lab1: i32 = 0;
let mut op1: i32 = 0;
let mut pos1_1: i32 = 0;
let mut line1_1: i32 = 0;
let mut pos2: i32 = 0;
let mut name_1: JSAtom = 0;
let mut vm_block: usize = 201;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 34426
1 => {
return ((1 as i32)).wrapping_neg();
}
// C line 34425
2 => {
let _ = { (*(s)).byte_code = core::ptr::read(core::ptr::addr_of!(bc_out)); core::ptr::read(core::ptr::addr_of!((*(s)).byte_code)) };
vm_block = 1; continue;
}
// C line 34424
3 => {
let _ = dbuf_free(core::ptr::addr_of_mut!((*(s)).byte_code));
vm_block = 2; continue;
}
// C line 34418 labels: fail
4 => {
vm_block = if ((((pos) < (bc_len)) as i32)) != 0 { 9 } else { 3 }; continue;
}
// C line 34418
5 => {
let _ = { let assigned = pos_next; pos = assigned; assigned };
vm_block = 4; continue;
}
// C line 34422
6 => {
let _ = crate::cutils::dbuf_put(core::ptr::addr_of_mut!(bc_out), (bc_buf).offset(((pos) as isize)), ((len) as usize));
vm_block = 5; continue;
}
// C line 34421
7 => {
let _ = { let assigned = (pos).wrapping_add(len); pos_next = assigned; assigned };
vm_block = 6; continue;
}
// C line 34420
8 => {
let _ = { let assigned = (((*((opcode_info).as_ptr()).offset((op) as isize)).size) as i32); len = assigned; assigned };
vm_block = 7; continue;
}
// C line 34419
9 => {
let _ = { let assigned = ((*(bc_buf).offset((pos) as isize)) as i32); op = assigned; assigned };
vm_block = 8; continue;
}
// C line 34414
10 => {
return (0 as i32);
}
// C line 34412
11 => {
return ((1 as i32)).wrapping_neg();
}
// C line 34411
12 => {
let _ = JS_ThrowOutOfMemory(ctx);
vm_block = 11; continue;
}
// C line 34410
13 => {
vm_block = if (dbuf_error(core::ptr::addr_of_mut!((*(s)).byte_code))) != 0 { 12 } else { 10 }; continue;
}
// C line 34409
14 => {
let _ = { (*(s)).byte_code = core::ptr::read(core::ptr::addr_of!(bc_out)); core::ptr::read(core::ptr::addr_of!((*(s)).byte_code)) };
vm_block = 13; continue;
}
// C line 34408
15 => {
let _ = dbuf_free(core::ptr::addr_of_mut!((*(s)).byte_code));
vm_block = 14; continue;
}
// C line 34120
16 => {
vm_block = if ((((pos) < (bc_len)) as i32)) != 0 { 178 } else { 15 }; continue;
}
// C line 34120
17 => {
let _ = { let assigned = pos_next; pos = assigned; assigned };
vm_block = 16; continue;
}
// C line 34403
18 => {
vm_block = 17; continue;
}
// C line 34402 labels: no_change
19 => {
let _ = crate::cutils::dbuf_put(core::ptr::addr_of_mut!(bc_out), (bc_buf).offset(((pos) as isize)), ((len) as usize));
vm_block = 18; continue;
}
// C line 34398
20 => {
vm_block = 17; continue;
}
// C line 34397
21 => {
let _ = dbuf_putc(core::ptr::addr_of_mut!(bc_out), (((OP_get_array_el as i32)) as u8));
vm_block = 20; continue;
}
// C line 34395
22 => {
vm_block = 17; continue;
}
// C line 34393
23 => {
let _ = dbuf_put_u32(core::ptr::addr_of_mut!(bc_out), name_1);
vm_block = 22; continue;
}
// C line 34392
24 => {
let _ = dbuf_putc(core::ptr::addr_of_mut!(bc_out), (((OP_get_field as i32)) as u8));
vm_block = 23; continue;
}
// C line 34391
25 => {
name_1 = get_u32(((bc_buf).offset(((pos) as isize))).offset((((1 as i32)) as isize)));
vm_block = 24; continue;
}
// C line 34387
26 => {
vm_block = 17; continue;
}
// C line 34384
27 => {
vm_block = 17; continue;
}
// C line 34380
28 => {
vm_block = 19; continue;
}
// C line 34376
29 => {
vm_block = 17; continue;
}
// C line 34374
30 => {
let _ = dbuf_put_u32(core::ptr::addr_of_mut!(bc_out), ((line_num) as u32));
vm_block = 29; continue;
}
// C line 34373
31 => {
let _ = dbuf_putc(core::ptr::addr_of_mut!(bc_out), (((OP_line_num as i32)) as u8));
vm_block = 30; continue;
}
// C line 34372
32 => {
let _ = { let old = (*(s)).line_number_size; (*(s)).line_number_size = ((*(s)).line_number_size).wrapping_add(1); old };
vm_block = 31; continue;
}
// C line 34371
33 => {
let _ = { let assigned = line1_1; line_num = assigned; assigned };
vm_block = 32; continue;
}
// C line 34370
34 => {
vm_block = if ((((((((line1_1) != (((1 as i32)).wrapping_neg())) as i32)) != 0) && (((((line1_1) != (line_num)) as i32)) != 0)) as i32)) != 0 { 33 } else { 29 }; continue;
}
// C line 34369
35 => {
let _ = { let assigned = pos1_1; pos_next = assigned; assigned };
vm_block = 34; continue;
}
// C line 34368
36 => {
let _ = dbuf_put_u32(core::ptr::addr_of_mut!(bc_out), (((cc).label) as u32));
vm_block = 35; continue;
}
// C line 34367
37 => {
let _ = dbuf_putc(core::ptr::addr_of_mut!(bc_out), ((op1) as u8));
vm_block = 36; continue;
}
// C line 34366
38 => {
let _ = update_label(s, (cc).label, (1 as i32));
vm_block = 37; continue;
}
// C line 34365
39 => {
let _ = update_label(s, lab0, ((1 as i32)).wrapping_neg());
vm_block = 38; continue;
}
// C line 34364
40 => {
let _ = { let old = (*(s)).jump_size; (*(s)).jump_size = ((*(s)).jump_size).wrapping_add(1); old };
vm_block = 39; continue;
}
// C line 34363
41 => {
vm_block = if (code_match(core::ptr::addr_of_mut!(cc), pos2, &[(op1) as i32, (((1 as i32)).wrapping_neg()) as i32])) != 0 { 40 } else { 28 }; continue;
}
// C line 34360
42 => {
vm_block = if (code_match(core::ptr::addr_of_mut!(cc), { let assigned = get_label_pos(s, lab1); pos2 = assigned; assigned }, &[((OP_dup as i32)) as i32, (op1) as i32, ((OP_drop as i32)) as i32, (((1 as i32)).wrapping_neg()) as i32])) != 0 { 43 } else { 41 }; continue;
}
// C line 34361
43 => {
let _ = { let assigned = (cc).label; lab1 = assigned; assigned };
vm_block = 42; continue;
}
// C line 34359
44 => {
let _ = { let assigned = (cc).line_num; line1_1 = assigned; assigned };
vm_block = 42; continue;
}
// C line 34358
45 => {
let _ = { let assigned = (cc).pos; pos1_1 = assigned; assigned };
vm_block = 44; continue;
}
// C line 34357
46 => {
let _ = { let assigned = (cc).op; op1 = assigned; assigned };
vm_block = 45; continue;
}
// C line 34356
47 => {
let _ = if ((((!(((((((((lab1) >= ((0 as i32))) as i32)) != 0) && (((((lab1) < ((*(s)).label_count)) as i32)) != 0)) as i32)) != 0) as i32)) as i64)) != 0 { std::process::abort() } else { { let _ = (0 as i32); } };
vm_block = 46; continue;
}
// C line 34355
48 => {
let _ = { let assigned = { let assigned = (cc).label; lab1 = assigned; assigned }; lab0 = assigned; assigned };
vm_block = 47; continue;
}
// C line 34353
49 => {
vm_block = if (code_match(core::ptr::addr_of_mut!(cc), pos_next, &[((((OP_if_false as i32)) | (((OP_if_true as i32)).wrapping_shl(((8 as i32)) as u32)))) as i32, ((OP_drop as i32)) as i32, (((1 as i32)).wrapping_neg()) as i32])) != 0 { 48 } else { 28 }; continue;
}
// C line 34350
50 => {
vm_block = if ((1 as i32)) != 0 { 49 } else { 28 }; continue;
}
// C line 34347
51 => {
vm_block = 19; continue;
}
// C line 34346
52 => {
let _ = { let old = (*(s)).jump_size; (*(s)).jump_size = ((*(s)).jump_size).wrapping_add(1); old };
vm_block = 51; continue;
}
// C line 34341
53 => {
vm_block = 19; continue;
}
// C line 34339
54 => {
vm_block = 17; continue;
}
// C line 34338
55 => {
vm_block = if ((((name) == ((((0 as i32)) as JSAtom))) as i32)) != 0 { 54 } else { 53 }; continue;
}
// C line 34337
56 => {
name = get_u32(((bc_buf).offset(((pos) as isize))).offset((((1 as i32)) as isize)));
vm_block = 55; continue;
}
// C line 34332
57 => {
vm_block = 17; continue;
}
// C line 34319
58 => {
vm_block = if ((((scope_idx_1) >= ((0 as i32))) as i32)) != 0 { 65 } else { 57 }; continue;
}
// C line 34326
59 => {
let _ = { let assigned = (*(vd_1)).scope_next; scope_idx_1 = assigned; assigned };
vm_block = 58; continue;
}
// C line 34324
60 => {
let _ = dbuf_put_u16(core::ptr::addr_of_mut!(bc_out), ((scope_idx_1) as u16));
vm_block = 59; continue;
}
// C line 34323
61 => {
let _ = dbuf_putc(core::ptr::addr_of_mut!(bc_out), (((OP_close_loc as i32)) as u8));
vm_block = 60; continue;
}
// C line 34322
62 => {
vm_block = if ((*(vd_1)).is_captured()) != 0 { 61 } else { 59 }; continue;
}
// C line 34328
63 => {
vm_block = 57; continue;
}
// C line 34321
64 => {
vm_block = if (((((*(vd_1)).scope_level) == (scope_2)) as i32)) != 0 { 62 } else { 63 }; continue;
}
// C line 34320
65 => {
vd_1 = core::ptr::addr_of_mut!(*((*(s)).vars).offset((scope_idx_1) as isize));
vm_block = 64; continue;
}
// C line 34319
66 => {
let _ = { let assigned = (*((*(s)).scopes).offset((scope_2) as isize)).first; scope_idx_1 = assigned; assigned };
vm_block = 58; continue;
}
// C line 34317
67 => {
scope_2 = ((get_u16(((bc_buf).offset(((pos) as isize))).offset((((1 as i32)) as isize)))) as i32);
vm_block = 66; continue;
}
// C line 34313
68 => {
vm_block = 17; continue;
}
// C line 34289
69 => {
vm_block = if ((((scope_idx) >= ((0 as i32))) as i32)) != 0 { 81 } else { 68 }; continue;
}
// C line 34307
70 => {
let _ = { let assigned = (*(vd)).scope_next; scope_idx = assigned; assigned };
vm_block = 69; continue;
}
// C line 34299
71 => {
let _ = dbuf_put_u16(core::ptr::addr_of_mut!(bc_out), ((scope_idx) as u16));
vm_block = 70; continue;
}
// C line 34298
72 => {
let _ = dbuf_putc(core::ptr::addr_of_mut!(bc_out), (((OP_put_loc as i32)) as u8));
vm_block = 71; continue;
}
// C line 34297
73 => {
let _ = dbuf_put_u32(core::ptr::addr_of_mut!(bc_out), (((*(vd)).func_pool_idx) as u32));
vm_block = 72; continue;
}
// C line 34296
74 => {
let _ = dbuf_putc(core::ptr::addr_of_mut!(bc_out), (((OP_fclosure as i32)) as u8));
vm_block = 73; continue;
}
// C line 34304
75 => {
let _ = dbuf_put_u16(core::ptr::addr_of_mut!(bc_out), ((scope_idx) as u16));
vm_block = 70; continue;
}
// C line 34303
76 => {
let _ = dbuf_putc(core::ptr::addr_of_mut!(bc_out), (((OP_set_loc_uninitialized as i32)) as u8));
vm_block = 75; continue;
}
// C line 34293
77 => {
vm_block = if (((((((((((*(vd)).var_kind()) as i32)) == ((JS_VAR_FUNCTION_DECL as i32))) as i32)) != 0) || ((((((((*(vd)).var_kind()) as i32)) == ((JS_VAR_NEW_FUNCTION_DECL as i32))) as i32)) != 0)) as i32)) != 0 { 74 } else { 76 }; continue;
}
// C line 34292
78 => {
vm_block = if ((((scope_idx) != ((*(s)).arguments_arg_idx)) as i32)) != 0 { 77 } else { 70 }; continue;
}
// C line 34309
79 => {
vm_block = 68; continue;
}
// C line 34291
80 => {
vm_block = if (((((*(vd)).scope_level) == (scope_1)) as i32)) != 0 { 78 } else { 79 }; continue;
}
// C line 34290
81 => {
vd = core::ptr::addr_of_mut!(*((*(s)).vars).offset((scope_idx) as isize));
vm_block = 80; continue;
}
// C line 34289
82 => {
let _ = { let assigned = (*((*(s)).scopes).offset((scope_1) as isize)).first; scope_idx = assigned; assigned };
vm_block = 69; continue;
}
// C line 34286
83 => {
let _ = instantiate_hoisted_definitions(ctx, s, core::ptr::addr_of_mut!(bc_out));
vm_block = 82; continue;
}
// C line 34285
84 => {
vm_block = if ((((scope_1) == ((*(s)).body_scope)) as i32)) != 0 { 83 } else { 82 }; continue;
}
// C line 34283
85 => {
scope_1 = ((get_u16(((bc_buf).offset(((pos) as isize))).offset((((1 as i32)) as isize)))) as i32);
vm_block = 84; continue;
}
// C line 34279
86 => {
vm_block = 19; continue;
}
// C line 34277
87 => {
let _ = { let assigned = ((((bc_out).size).wrapping_add((((*((opcode_info).as_ptr()).offset((op) as isize)).size) as usize))) as i32); (*(ls_2)).pos2 = assigned; assigned };
vm_block = 86; continue;
}
// C line 34276
88 => {
let _ = { let assigned = core::ptr::addr_of_mut!(*((*(s)).label_slots).offset((label_2) as isize)); ls_2 = assigned; assigned };
vm_block = 87; continue;
}
// C line 34275
89 => {
let _ = if ((((!(((((((((label_2) >= ((0 as i32))) as i32)) != 0) && (((((label_2) < ((*(s)).label_count)) as i32)) != 0)) as i32)) != 0) as i32)) as i64)) != 0 { std::process::abort() } else { { let _ = (0 as i32); } };
vm_block = 88; continue;
}
// C line 34274
90 => {
let _ = { let assigned = ((get_u32(((bc_buf).offset(((pos) as isize))).offset((((1 as i32)) as isize)))) as i32); label_2 = assigned; assigned };
vm_block = 89; continue;
}
// C line 34267
91 => {
vm_block = 19; continue;
}
// C line 34265
92 => {
vm_block = 17; continue;
}
// C line 34263
93 => {
let _ = dbuf_put_u32(core::ptr::addr_of_mut!(bc_out), ((line_num) as u32));
vm_block = 92; continue;
}
// C line 34262
94 => {
let _ = dbuf_putc(core::ptr::addr_of_mut!(bc_out), (((OP_line_num as i32)) as u8));
vm_block = 93; continue;
}
// C line 34261
95 => {
let _ = { let old = (*(s)).line_number_size; (*(s)).line_number_size = ((*(s)).line_number_size).wrapping_add(1); old };
vm_block = 94; continue;
}
// C line 34260
96 => {
let _ = { let assigned = line; line_num = assigned; assigned };
vm_block = 95; continue;
}
// C line 34259
97 => {
vm_block = if ((((((((((((pos) < (bc_len)) as i32)) != 0) && (((((line) >= ((0 as i32))) as i32)) != 0)) as i32)) != 0) && (((((line_num) != (line)) as i32)) != 0)) as i32)) != 0 { 96 } else { 92 }; continue;
}
// C line 34258
98 => {
let _ = { let assigned = pos; pos_next = assigned; assigned };
vm_block = 97; continue;
}
// C line 34257
99 => {
let _ = { let assigned = skip_dead_code(s, bc_buf, bc_len, (pos).wrapping_add(len), core::ptr::addr_of_mut!(line)); pos = assigned; assigned };
vm_block = 98; continue;
}
// C line 34256
100 => {
let _ = crate::cutils::dbuf_put(core::ptr::addr_of_mut!(bc_out), (bc_buf).offset(((pos) as isize)), ((len) as usize));
vm_block = 99; continue;
}
// C line 34255
101 => {
line = ((1 as i32)).wrapping_neg();
vm_block = 100; continue;
}
// C line 34253
102 => {
vm_block = if ((1 as i32)) != 0 { 101 } else { 91 }; continue;
}
// C line 34244
103 => {
let _ = { let old = (*(s)).jump_size; (*(s)).jump_size = ((*(s)).jump_size).wrapping_add(1); old };
vm_block = 102; continue;
}
// C line 34241
104 => {
vm_block = 19; continue;
}
// C line 34238
105 => {
vm_block = 17; continue;
}
// C line 34236
106 => {
let _ = dbuf_put_u32(core::ptr::addr_of_mut!(bc_out), ((line_num) as u32));
vm_block = 105; continue;
}
// C line 34235
107 => {
let _ = dbuf_putc(core::ptr::addr_of_mut!(bc_out), (((OP_line_num as i32)) as u8));
vm_block = 106; continue;
}
// C line 34234
108 => {
let _ = { let old = (*(s)).line_number_size; (*(s)).line_number_size = ((*(s)).line_number_size).wrapping_add(1); old };
vm_block = 107; continue;
}
// C line 34233
109 => {
let _ = { let assigned = (cc).line_num; line_num = assigned; assigned };
vm_block = 108; continue;
}
// C line 34232
110 => {
vm_block = if (((((((((cc).line_num) != (((1 as i32)).wrapping_neg())) as i32)) != 0) && ((((((cc).line_num) != (line_num)) as i32)) != 0)) as i32)) != 0 { 109 } else { 105 }; continue;
}
// C line 34231
111 => {
let _ = { let assigned = (cc).pos; pos_next = assigned; assigned };
vm_block = 110; continue;
}
// C line 34230
112 => {
let _ = dbuf_putc(core::ptr::addr_of_mut!(bc_out), (((cc).op) as u8));
vm_block = 111; continue;
}
// C line 34229
113 => {
vm_block = if (code_match(core::ptr::addr_of_mut!(cc), pos_next, &[((((OP_put_array_el as i32)) | (((OP_put_ref_value as i32)).wrapping_shl(((8 as i32)) as u32)))) as i32, ((OP_drop as i32)) as i32, (((1 as i32)).wrapping_neg()) as i32])) != 0 { 112 } else { 104 }; continue;
}
// C line 34227
114 => {
vm_block = if ((1 as i32)) != 0 { 113 } else { 104 }; continue;
}
// C line 34225
115 => {
vm_block = 19; continue;
}
// C line 34222
116 => {
vm_block = 17; continue;
}
// C line 34220
117 => {
let _ = dbuf_put_u32(core::ptr::addr_of_mut!(bc_out), ((line_num) as u32));
vm_block = 116; continue;
}
// C line 34219
118 => {
let _ = dbuf_putc(core::ptr::addr_of_mut!(bc_out), (((OP_line_num as i32)) as u8));
vm_block = 117; continue;
}
// C line 34218
119 => {
let _ = { let old = (*(s)).line_number_size; (*(s)).line_number_size = ((*(s)).line_number_size).wrapping_add(1); old };
vm_block = 118; continue;
}
// C line 34217
120 => {
let _ = { let assigned = line1; line_num = assigned; assigned };
vm_block = 119; continue;
}
// C line 34216
121 => {
vm_block = if ((((((((line1) != (((1 as i32)).wrapping_neg())) as i32)) != 0) && (((((line1) != (line_num)) as i32)) != 0)) as i32)) != 0 { 120 } else { 116 }; continue;
}
// C line 34215
122 => {
let _ = { let assigned = pos1; pos_next = assigned; assigned };
vm_block = 121; continue;
}
// C line 34214
123 => {
vm_block = if (code_match(core::ptr::addr_of_mut!(cc), pos1, &[((OP_return_undef as i32)) as i32, (((1 as i32)).wrapping_neg()) as i32])) != 0 { 122 } else { 115 }; continue;
}
// C line 34210
124 => {
vm_block = if (code_match(core::ptr::addr_of_mut!(cc), pos1, &[((OP_drop as i32)) as i32, (((1 as i32)).wrapping_neg()) as i32])) != 0 { 127 } else { 123 }; continue;
}
// C line 34212
125 => {
let _ = { let assigned = (cc).pos; pos1 = assigned; assigned };
vm_block = 124; continue;
}
// C line 34211
126 => {
let _ = { let assigned = (cc).line_num; line1 = assigned; assigned };
vm_block = 125; continue;
}
// C line 34211
127 => {
vm_block = if (((((cc).line_num) >= ((0 as i32))) as i32)) != 0 { 126 } else { 125 }; continue;
}
// C line 34209
128 => {
line1 = line_num;
vm_block = 124; continue;
}
// C line 34208
129 => {
pos1 = pos_next;
vm_block = 128; continue;
}
// C line 34204
130 => {
vm_block = if ((0 as i32)) != 0 { 129 } else { 115 }; continue;
}
// C line 34202
131 => {
vm_block = 19; continue;
}
// C line 34199
132 => {
vm_block = 17; continue;
}
// C line 34198
133 => {
let _ = { let old = (*(ls_1)).ref_count; (*(ls_1)).ref_count = ((*(ls_1)).ref_count).wrapping_sub(1); old };
vm_block = 132; continue;
}
// C line 34197
134 => {
vm_block = if (code_match(core::ptr::addr_of_mut!(cc), (*(ls_1)).pos, &[((OP_ret as i32)) as i32, (((1 as i32)).wrapping_neg()) as i32])) != 0 { 133 } else { 131 }; continue;
}
// C line 34196
135 => {
let _ = { let assigned = core::ptr::addr_of_mut!(*((*(s)).label_slots).offset((label_1) as isize)); ls_1 = assigned; assigned };
vm_block = 134; continue;
}
// C line 34195
136 => {
let _ = if ((((!(((((((((label_1) >= ((0 as i32))) as i32)) != 0) && (((((label_1) < ((*(s)).label_count)) as i32)) != 0)) as i32)) != 0) as i32)) as i64)) != 0 { std::process::abort() } else { { let _ = (0 as i32); } };
vm_block = 135; continue;
}
// C line 34194
137 => {
let _ = { let assigned = ((get_u32(((bc_buf).offset(((pos) as isize))).offset((((1 as i32)) as isize)))) as i32); label_1 = assigned; assigned };
vm_block = 136; continue;
}
// C line 34189
138 => {
vm_block = if ((1 as i32)) != 0 { 137 } else { 131 }; continue;
}
// C line 34188
139 => {
let _ = { let old = (*(s)).jump_size; (*(s)).jump_size = ((*(s)).jump_size).wrapping_add(1); old };
vm_block = 138; continue;
}
// C line 34186
140 => {
vm_block = 17; continue;
}
// C line 34184
141 => {
let _ = JS_FreeAtom(ctx, var_name);
vm_block = 140; continue;
}
// C line 34183
142 => {
vm_block = 4; continue;
}
// C line 34182
143 => {
vm_block = if ((((ret) < ((0 as i32))) as i32)) != 0 { 142 } else { 141 }; continue;
}
// C line 34181
144 => {
let _ = { let assigned = resolve_scope_private_field(ctx, s, var_name, scope, op, core::ptr::addr_of_mut!(bc_out)); ret = assigned; assigned };
vm_block = 143; continue;
}
// C line 34180
145 => {
let _ = { let assigned = ((get_u16(((bc_buf).offset(((pos) as isize))).offset((((5 as i32)) as isize)))) as i32); scope = assigned; assigned };
vm_block = 144; continue;
}
// C line 34179
146 => {
let _ = { let assigned = get_u32(((bc_buf).offset(((pos) as isize))).offset((((1 as i32)) as isize))); var_name = assigned; assigned };
vm_block = 145; continue;
}
// C line 34172
147 => {
vm_block = 17; continue;
}
// C line 34170
148 => {
let _ = JS_FreeAtom(ctx, var_name);
vm_block = 147; continue;
}
// C line 34168
149 => {
let _ = { let assigned = resolve_scope_var(ctx, s, var_name, scope, op, core::ptr::addr_of_mut!(bc_out), bc_buf, ls, pos_next, &mut scope_lookup); pos_next = assigned; assigned };
vm_block = 148; continue;
}
// C line 34167
150 => {
let _ = { let old = (*(ls)).ref_count; (*(ls)).ref_count = ((*(ls)).ref_count).wrapping_sub(1); old };
vm_block = 149; continue;
}
// C line 34166
151 => {
let _ = { let assigned = core::ptr::addr_of_mut!(*((*(s)).label_slots).offset((label) as isize)); ls = assigned; assigned };
vm_block = 150; continue;
}
// C line 34165
152 => {
let _ = { let assigned = ((get_u16(((bc_buf).offset(((pos) as isize))).offset((((9 as i32)) as isize)))) as i32); scope = assigned; assigned };
vm_block = 151; continue;
}
// C line 34164
153 => {
let _ = { let assigned = ((get_u32(((bc_buf).offset(((pos) as isize))).offset((((5 as i32)) as isize)))) as i32); label = assigned; assigned };
vm_block = 152; continue;
}
// C line 34163
154 => {
let _ = { let assigned = get_u32(((bc_buf).offset(((pos) as isize))).offset((((1 as i32)) as isize))); var_name = assigned; assigned };
vm_block = 153; continue;
}
// C line 34158
155 => {
vm_block = 17; continue;
}
// C line 34157
156 => {
let _ = JS_FreeAtom(ctx, var_name);
vm_block = 155; continue;
}
// C line 34155
157 => {
let _ = { let assigned = resolve_scope_var(ctx, s, var_name, scope, op, core::ptr::addr_of_mut!(bc_out), core::ptr::null_mut::<u8>(), core::ptr::null_mut::<LabelSlot>(), pos_next, &mut scope_lookup); pos_next = assigned; assigned };
vm_block = 156; continue;
}
// C line 34154
158 => {
let _ = { let assigned = ((get_u16(((bc_buf).offset(((pos) as isize))).offset((((5 as i32)) as isize)))) as i32); scope = assigned; assigned };
vm_block = 157; continue;
}
// C line 34153
159 => {
let _ = { let assigned = get_u32(((bc_buf).offset(((pos) as isize))).offset((((1 as i32)) as isize))); var_name = assigned; assigned };
vm_block = 158; continue;
}
// C line 34145
160 => {
vm_block = 17; continue;
}
// C line 34144
161 => {
let _ = dbuf_put_u16(core::ptr::addr_of_mut!(bc_out), ((((*((*(s)).scopes).offset((scope) as isize)).first).wrapping_sub(((2 as i32)).wrapping_neg())) as u16));
vm_block = 160; continue;
}
// C line 34143
162 => {
let _ = dbuf_putc(core::ptr::addr_of_mut!(bc_out), ((op) as u8));
vm_block = 161; continue;
}
// C line 34142
163 => {
let _ = mark_eval_captured_variables(ctx, s, scope);
vm_block = 162; continue;
}
// C line 34141
164 => {
let _ = { let assigned = ((get_u16(((bc_buf).offset(((pos) as isize))).offset((((1 as i32)) as isize)))) as i32); scope = assigned; assigned };
vm_block = 163; continue;
}
// C line 34139
165 => {
vm_block = 17; continue;
}
// C line 34137
166 => {
let _ = dbuf_put_u16(core::ptr::addr_of_mut!(bc_out), ((((*((*(s)).scopes).offset((scope) as isize)).first).wrapping_sub(((2 as i32)).wrapping_neg())) as u16));
vm_block = 165; continue;
}
// C line 34136
167 => {
let _ = dbuf_put_u16(core::ptr::addr_of_mut!(bc_out), ((call_argc) as u16));
vm_block = 166; continue;
}
// C line 34135
168 => {
let _ = dbuf_putc(core::ptr::addr_of_mut!(bc_out), ((op) as u8));
vm_block = 167; continue;
}
// C line 34134
169 => {
let _ = mark_eval_captured_variables(ctx, s, scope);
vm_block = 168; continue;
}
// C line 34133
170 => {
let _ = { let assigned = ((get_u16((((bc_buf).offset(((pos) as isize))).offset((((1 as i32)) as isize))).offset((((2 as i32)) as isize)))) as i32); scope = assigned; assigned };
vm_block = 169; continue;
}
// C line 34132
171 => {
call_argc = ((get_u16(((bc_buf).offset(((pos) as isize))).offset((((1 as i32)) as isize)))) as i32);
vm_block = 170; continue;
}
// C line 34128
172 => {
vm_block = 19; continue;
}
// C line 34127
173 => {
let _ = { let old = (*(s)).line_number_size; (*(s)).line_number_size = ((*(s)).line_number_size).wrapping_add(1); old };
vm_block = 172; continue;
}
// C line 34126
174 => {
let _ = { let assigned = ((get_u32(((bc_buf).offset(((pos) as isize))).offset((((1 as i32)) as isize)))) as i32); line_num = assigned; assigned };
vm_block = 173; continue;
}
// C line 34124
175 => {
vm_block = match op { x if x == (OP_get_array_el_opt_chain as i32) => 21, x if x == (OP_get_field_opt_chain as i32) => 25, x if x == (OP_set_class_name as i32) => 26, x if x == (OP_nop as i32) => 27, x if x == (OP_dup as i32) => 50, x if x == (OP_catch as i32) => 52, x if x == (OP_if_true as i32) => 52, x if x == (OP_if_false as i32) => 52, x if x == (OP_set_name as i32) => 56, x if x == (OP_leave_scope as i32) => 67, x if x == (OP_enter_scope as i32) => 85, x if x == (OP_label as i32) => 90, x if x == (OP_ret as i32) => 102, x if x == (OP_throw_error as i32) => 102, x if x == (OP_throw as i32) => 102, x if x == (OP_return_undef as i32) => 102, x if x == (OP_return as i32) => 102, x if x == (OP_tail_call_method as i32) => 102, x if x == (OP_tail_call as i32) => 102, x if x == (OP_goto as i32) => 103, x if x == (OP_insert3 as i32) => 114, x if x == (OP_drop as i32) => 130, x if x == (OP_gosub as i32) => 139, x if x == (OP_scope_in_private_field as i32) => 146, x if x == (OP_scope_put_private_field as i32) => 146, x if x == (OP_scope_get_private_field2 as i32) => 146, x if x == (OP_scope_get_private_field as i32) => 146, x if x == (OP_scope_make_ref as i32) => 154, x if x == (OP_scope_put_var_init as i32) => 159, x if x == (OP_scope_get_ref as i32) => 159, x if x == (OP_scope_delete_var as i32) => 159, x if x == (OP_scope_put_var as i32) => 159, x if x == (OP_scope_get_var as i32) => 159, x if x == (OP_scope_get_var_undef as i32) => 159, x if x == (OP_scope_get_var_checkthis as i32) => 159, x if x == (OP_apply_eval as i32) => 164, x if x == (OP_eval as i32) => 171, x if x == (OP_line_num as i32) => 174, _ => 19, }; continue;
}
// C line 34123
176 => {
let _ = { let assigned = (pos).wrapping_add(len); pos_next = assigned; assigned };
vm_block = 175; continue;
}
// C line 34122
177 => {
let _ = { let assigned = (((*((opcode_info).as_ptr()).offset((op) as isize)).size) as i32); len = assigned; assigned };
vm_block = 176; continue;
}
// C line 34121
178 => {
let _ = { let assigned = ((*(bc_buf).offset((pos) as isize)) as i32); op = assigned; assigned };
vm_block = 177; continue;
}
// C line 34120
179 => {
let _ = { let assigned = (0 as i32); pos = assigned; assigned };
vm_block = 16; continue;
}
// C line 34119
180 => {
let _ = { let assigned = (0 as i32); line_num = assigned; assigned };
vm_block = 179; continue;
}
// C line 34086
181 => {
vm_block = if ((((i) < ((*(s)).global_var_count)) as i32)) != 0 { 197 } else { 180 }; continue;
}
// C line 34086 labels: next
182 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 181; continue;
}
// C line 34090
183 => {
vm_block = if ((((idx) < ((*(s)).closure_var_count)) as i32)) != 0 { 195 } else { 182 }; continue;
}
// C line 34090
184 => {
let _ = { let old = idx; idx = (idx).wrapping_add(1); old };
vm_block = 183; continue;
}
// C line 34114
185 => {
vm_block = 182; continue;
}
// C line 34112
186 => {
vm_block = if (((((((((*(cv)).var_name) == ((((crate::quickjs_atom::JS_ATOM__var_ as i32)) as JSAtom))) as i32)) != 0) || ((((((*(cv)).var_name) == ((((crate::quickjs_atom::JS_ATOM__arg_var_ as i32)) as JSAtom))) as i32)) != 0)) as i32)) != 0 { 185 } else { 184 }; continue;
}
// C line 34110
187 => {
vm_block = 182; continue;
}
// C line 34108
188 => {
let _ = dbuf_putc(core::ptr::addr_of_mut!(bc_out), (((1 as i32)) as u8));
vm_block = 187; continue;
}
// C line 34107
189 => {
let _ = dbuf_put_u32(core::ptr::addr_of_mut!(bc_out), JS_DupAtom(ctx, (*(hf)).var_name));
vm_block = 188; continue;
}
// C line 34106
190 => {
let _ = dbuf_putc(core::ptr::addr_of_mut!(bc_out), (((OP_throw_error as i32)) as u8));
vm_block = 189; continue;
}
// C line 34099
191 => {
vm_block = if (((((((((*(s)).eval_type) == (((2 as i32)).wrapping_shl(((0 as i32)) as u32))) as i32)) != 0) && (((((*(cv)).is_lexical()) as i32)) != 0)) as i32)) != 0 { 190 } else { 187 }; continue;
}
// C line 34098
192 => {
vm_block = if (((((*(cv)).var_name) == ((*(hf)).var_name)) as i32)) != 0 { 191 } else { 186 }; continue;
}
// C line 34097
193 => {
vm_block = 182; continue;
}
// C line 34092
194 => {
vm_block = if (((((((((((((((((((((((*(cv)).closure_type()) as i32)) == ((JS_CLOSURE_GLOBAL_REF as i32))) as i32)) != 0) || ((((((((*(cv)).closure_type()) as i32)) == ((JS_CLOSURE_GLOBAL_DECL as i32))) as i32)) != 0)) as i32)) != 0) || ((((((((*(cv)).closure_type()) as i32)) == ((JS_CLOSURE_GLOBAL as i32))) as i32)) != 0)) as i32)) != 0) || ((((((((*(cv)).closure_type()) as i32)) == ((JS_CLOSURE_MODULE_DECL as i32))) as i32)) != 0)) as i32)) != 0) || ((((((((*(cv)).closure_type()) as i32)) == ((JS_CLOSURE_MODULE_IMPORT as i32))) as i32)) != 0)) as i32)) != 0 { 193 } else { 192 }; continue;
}
// C line 34091
195 => {
cv = core::ptr::addr_of_mut!(*((*(s)).closure_var).offset((idx) as isize));
vm_block = 194; continue;
}
// C line 34090
196 => {
let _ = { let assigned = (0 as i32); idx = assigned; assigned };
vm_block = 183; continue;
}
// C line 34087
197 => {
hf = core::ptr::addr_of_mut!(*((*(s)).global_vars).offset((i) as isize));
vm_block = 196; continue;
}
// C line 34086
198 => {
let _ = { let assigned = (0 as i32); i = assigned; assigned };
vm_block = 181; continue;
}
// C line 34081
199 => {
let _ = js_dbuf_bytecode_init(ctx, core::ptr::addr_of_mut!(bc_out));
vm_block = 198; continue;
}
// C line 34080
200 => {
let _ = { let assigned = { let assigned = ((((*(s)).byte_code).size) as i32); bc_len = assigned; assigned }; (cc).bc_len = assigned; assigned };
vm_block = 199; continue;
}
// C line 34079
201 => {
let _ = { let assigned = { let assigned = ((*(s)).byte_code).buf; bc_buf = assigned; assigned }; (cc).bc_buf = assigned; assigned };
vm_block = 200; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:34430. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn add_pc2line_info(mut s: *mut JSFunctionDef, mut pc: u32, mut source_pos: u32) -> () {
let mut vm_local_storage = Vec::<u64>::new();
let mut vm_block: usize = 6;
loop { match vm_block {
// C line ?
0 => {
return ();
}
// C line 34440
1 => {
let _ = { let assigned = ((source_pos) as i32); (*(s)).line_number_last = assigned; assigned };
vm_block = 0; continue;
}
// C line 34439
2 => {
let _ = { let assigned = ((pc) as i32); (*(s)).line_number_last_pc = assigned; assigned };
vm_block = 1; continue;
}
// C line 34438
3 => {
let _ = { let old = (*(s)).line_number_count; (*(s)).line_number_count = ((*(s)).line_number_count).wrapping_add(1); old };
vm_block = 2; continue;
}
// C line 34437
4 => {
let _ = { let assigned = source_pos; (*((*(s)).line_number_slots).offset(((*(s)).line_number_count) as isize)).source_pos = assigned; assigned };
vm_block = 3; continue;
}
// C line 34436
5 => {
let _ = { let assigned = pc; (*((*(s)).line_number_slots).offset(((*(s)).line_number_count) as isize)).pc = assigned; assigned };
vm_block = 4; continue;
}
// C line 34432
6 => {
vm_block = if (((((((((((((((((*(s)).line_number_slots) != (core::ptr::null_mut::<LineNumberSlot>())) as i32)) != 0) && ((((((*(s)).line_number_count) < ((*(s)).line_number_size)) as i32)) != 0)) as i32)) != 0) && (((((pc) >= ((((*(s)).line_number_last_pc) as u32))) as i32)) != 0)) as i32)) != 0) && (((((source_pos) != ((((*(s)).line_number_last) as u32))) as i32)) != 0)) as i32)) != 0 { 5 } else { 0 }; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:34450. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn compute_pc2line_info(mut s: *mut JSFunctionDef) -> () {
let mut vm_local_storage = Vec::<u64>::new();
let mut last_line_num: i32 = 0;
let mut last_col_num: i32 = 0;
let mut last_pc: u32 = 0;
let mut i: i32 = 0;
let mut line_num: i32 = 0;
let mut col_num: i32 = 0;
let mut buf_start: *const u8 = core::ptr::null();
let mut pc: u32 = 0;
let mut source_pos: u32 = 0;
let mut diff_pc: i32 = 0;
let mut diff_line: i32 = 0;
let mut diff_col: i32 = 0;
let mut vm_block: usize = 31;
loop { match vm_block {
// C line ?
0 => {
return ();
}
// C line 34465
1 => {
vm_block = if ((((i) < ((*(s)).line_number_count)) as i32)) != 0 { 23 } else { 0 }; continue;
}
// C line 34465
2 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 1; continue;
}
// C line 34498
3 => {
let _ = { let assigned = col_num; last_col_num = assigned; assigned };
vm_block = 2; continue;
}
// C line 34497
4 => {
let _ = { let assigned = line_num; last_line_num = assigned; assigned };
vm_block = 3; continue;
}
// C line 34496
5 => {
let _ = { let assigned = pc; last_pc = assigned; assigned };
vm_block = 4; continue;
}
// C line 34494
6 => {
let _ = dbuf_put_sleb128(core::ptr::addr_of_mut!((*(s)).pc2line), diff_col);
vm_block = 5; continue;
}
// C line 34486
7 => {
let _ = dbuf_putc(core::ptr::addr_of_mut!((*(s)).pc2line), (((((diff_line).wrapping_sub(((1 as i32)).wrapping_neg())).wrapping_add((diff_pc).wrapping_mul((5 as i32)))).wrapping_add((1 as i32))) as u8));
vm_block = 6; continue;
}
// C line 34492
8 => {
let _ = dbuf_put_sleb128(core::ptr::addr_of_mut!((*(s)).pc2line), diff_line);
vm_block = 6; continue;
}
// C line 34491
9 => {
let _ = dbuf_put_leb128(core::ptr::addr_of_mut!((*(s)).pc2line), ((diff_pc) as u32));
vm_block = 8; continue;
}
// C line 34490
10 => {
let _ = dbuf_putc(core::ptr::addr_of_mut!((*(s)).pc2line), (((0 as i32)) as u8));
vm_block = 9; continue;
}
// C line 34483
11 => {
vm_block = if ((((((((((((diff_line) >= (((1 as i32)).wrapping_neg())) as i32)) != 0) && (((((diff_line) < ((((1 as i32)).wrapping_neg()).wrapping_add((5 as i32)))) as i32)) != 0)) as i32)) != 0) && (((((diff_pc) <= (((((255 as i32)).wrapping_sub((1 as i32))) / ((5 as i32))))) as i32)) != 0)) as i32)) != 0 { 7 } else { 10 }; continue;
}
// C line 34481
12 => {
vm_block = 2; continue;
}
// C line 34480
13 => {
vm_block = if ((((((((diff_line) == ((0 as i32))) as i32)) != 0) && (((((diff_col) == ((0 as i32))) as i32)) != 0)) as i32)) != 0 { 12 } else { 11 }; continue;
}
// C line 34479
14 => {
let _ = { let assigned = (col_num).wrapping_sub(last_col_num); diff_col = assigned; assigned };
vm_block = 13; continue;
}
// C line 34478
15 => {
let _ = { let assigned = (line_num).wrapping_sub(last_line_num); diff_line = assigned; assigned };
vm_block = 14; continue;
}
// C line 34476
16 => {
let _ = { let assigned = get_line_col_cached((*(s)).get_line_col_cache, core::ptr::addr_of_mut!(col_num), (buf_start).offset(((source_pos) as isize))); line_num = assigned; assigned };
vm_block = 15; continue;
}
// C line 34474
17 => {
vm_block = 2; continue;
}
// C line 34473
18 => {
vm_block = if ((((diff_pc) < ((0 as i32))) as i32)) != 0 { 17 } else { 16 }; continue;
}
// C line 34472
19 => {
let _ = { let assigned = (((pc).wrapping_sub(last_pc)) as i32); diff_pc = assigned; assigned };
vm_block = 18; continue;
}
// C line 34471
20 => {
vm_block = 2; continue;
}
// C line 34470
21 => {
vm_block = if ((((source_pos) == (((((1 as i32)).wrapping_neg()) as u32))) as i32)) != 0 { 20 } else { 19 }; continue;
}
// C line 34467
22 => {
source_pos = (*((*(s)).line_number_slots).offset((i) as isize)).source_pos;
vm_block = 21; continue;
}
// C line 34466
23 => {
pc = (*((*(s)).line_number_slots).offset((i) as isize)).pc;
vm_block = 22; continue;
}
// C line 34465
24 => {
let _ = { let assigned = (0 as i32); i = assigned; assigned };
vm_block = 1; continue;
}
// C line 34463
25 => {
let _ = dbuf_put_leb128(core::ptr::addr_of_mut!((*(s)).pc2line), ((last_col_num) as u32));
vm_block = 24; continue;
}
// C line 34462
26 => {
let _ = dbuf_put_leb128(core::ptr::addr_of_mut!((*(s)).pc2line), ((last_line_num) as u32));
vm_block = 25; continue;
}
// C line 34459
27 => {
let _ = { let assigned = get_line_col_cached((*(s)).get_line_col_cache, core::ptr::addr_of_mut!(last_col_num), (buf_start).offset((((*(s)).source_pos) as isize))); last_line_num = assigned; assigned };
vm_block = 26; continue;
}
// C line 34457
28 => {
let _ = js_dbuf_init((*(s)).ctx, core::ptr::addr_of_mut!((*(s)).pc2line));
vm_block = 27; continue;
}
// C line 34456
29 => {
buf_start = (*((*(s)).get_line_col_cache)).buf_start;
vm_block = 28; continue;
}
// C line 34454
30 => {
last_pc = (((0 as i32)) as u32);
vm_block = 29; continue;
}
// C line 34452
31 => {
vm_block = if ((!(((*(s)).strip_debug()) != 0) as i32)) != 0 { 30 } else { 0 }; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:34503. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn add_reloc(mut ctx: *mut JSContext, mut ls: *mut LabelSlot, mut addr: u32, mut size: i32) -> *mut RelocEntry {
let mut vm_local_storage = Vec::<u64>::new();
let mut re: *mut RelocEntry = core::ptr::null_mut();
let mut vm_block: usize = 8;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 34513
1 => {
return re;
}
// C line 34512
2 => {
let _ = { let assigned = re; (*(ls)).first_reloc = assigned; assigned };
vm_block = 1; continue;
}
// C line 34511
3 => {
let _ = { let assigned = (*(ls)).first_reloc; (*(re)).next = assigned; assigned };
vm_block = 2; continue;
}
// C line 34510
4 => {
let _ = { let assigned = size; (*(re)).size = assigned; assigned };
vm_block = 3; continue;
}
// C line 34509
5 => {
let _ = { let assigned = addr; (*(re)).addr = assigned; assigned };
vm_block = 4; continue;
}
// C line 34508
6 => {
return core::ptr::null_mut::<RelocEntry>();
}
// C line 34507
7 => {
vm_block = if ((!(!(re).is_null()) as i32)) != 0 { 6 } else { 5 }; continue;
}
// C line 34506
8 => {
let _ = { let assigned = ((js_malloc(ctx, (size_of::<RelocEntry>() as usize))) as *mut RelocEntry); re = assigned; assigned };
vm_block = 7; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:34516. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn code_has_label(mut s: *mut CodeContext, mut pos: i32, mut label: i32) -> i32 {
let mut vm_local_storage = Vec::<u64>::new();
let mut op: i32 = 0;
let mut lab: i32 = 0;
let mut lab_1: i32 = 0;
let mut vm_block: usize = 2;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 34538
1 => {
return (0 as i32);
}
// C line 34518
2 => {
vm_block = if ((((pos) < ((*(s)).bc_len)) as i32)) != 0 { 17 } else { 1 }; continue;
}
// C line 34536
3 => {
vm_block = 1; continue;
}
// C line 34534
4 => {
return (1 as i32);
}
// C line 34533
5 => {
vm_block = if ((((lab_1) == (label)) as i32)) != 0 { 4 } else { 3 }; continue;
}
// C line 34532
6 => {
lab_1 = ((get_u32((((*(s)).bc_buf).offset(((pos) as isize))).offset((((1 as i32)) as isize)))) as i32);
vm_block = 5; continue;
}
// C line 34531
7 => {
vm_block = if ((((op) == ((OP_goto as i32))) as i32)) != 0 { 6 } else { 3 }; continue;
}
// C line 34529
8 => {
vm_block = 2; continue;
}
// C line 34528
9 => {
let _ = { pos = ((((pos) as i32)).wrapping_add((5 as i32))) as i32; pos };
vm_block = 8; continue;
}
// C line 34527
10 => {
return (1 as i32);
}
// C line 34526
11 => {
vm_block = if ((((lab) == (label)) as i32)) != 0 { 10 } else { 9 }; continue;
}
// C line 34525
12 => {
lab = ((get_u32((((*(s)).bc_buf).offset(((pos) as isize))).offset((((1 as i32)) as isize)))) as i32);
vm_block = 11; continue;
}
// C line 34524
13 => {
vm_block = if ((((op) == ((OP_label as i32))) as i32)) != 0 { 12 } else { 7 }; continue;
}
// C line 34522
14 => {
vm_block = 2; continue;
}
// C line 34521
15 => {
let _ = { pos = ((((pos) as i32)).wrapping_add((5 as i32))) as i32; pos };
vm_block = 14; continue;
}
// C line 34520
16 => {
vm_block = if ((((op) == ((OP_line_num as i32))) as i32)) != 0 { 15 } else { 13 }; continue;
}
// C line 34519
17 => {
op = ((*((*(s)).bc_buf).offset((pos) as isize)) as i32);
vm_block = 16; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:34544. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn find_jump_target(mut s: *mut JSFunctionDef, mut label0: i32, mut pop: *mut i32, mut pline: *mut i32) -> i32 {
let mut vm_local_storage = Vec::<u64>::new();
let mut i: i32 = 0;
let mut pos: i32 = 0;
let mut op: i32 = 0;
let mut label: i32 = 0;
let mut vm_block: usize = 25;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 34595
1 => {
return label;
}
// C line 34594
2 => {
let _ = update_label(s, label, (1 as i32));
vm_block = 1; continue;
}
// C line 34593 labels: done
3 => {
let _ = { let assigned = op; *(pop) = assigned; assigned };
vm_block = 2; continue;
}
// C line 34591
4 => {
let _ = { let assigned = label0; label = assigned; assigned };
vm_block = 3; continue;
}
// C line 34550
5 => {
vm_block = if ((((i) < ((10 as i32))) as i32)) != 0 { 22 } else { 4 }; continue;
}
// C line 34550
6 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 5; continue;
}
// C line 34553
7 => {
vm_block = 20; continue;
}
// C line 34575
8 => {
vm_block = 6; continue;
}
// C line 34573
9 => {
vm_block = 3; continue;
}
// C line 34570
10 => {
let _ = { let assigned = (OP_return_undef as i32); op = assigned; assigned };
vm_block = 9; continue;
}
// C line 34569
11 => {
vm_block = if ((((((*(((*(s)).byte_code).buf).offset((pos) as isize)) as i32)) == ((OP_return_undef as i32))) as i32)) != 0 { 10 } else { 9 }; continue;
}
// C line 34567
12 => {
vm_block = if ((((((*(((*(s)).byte_code).buf).offset(({ pos = (pos).wrapping_add(1); pos }) as isize)) as i32)) == ((OP_drop as i32))) as i32)) != 0 { 13 } else { 11 }; continue;
}
// C line 34568
13 => {
vm_block = 12; continue;
}
// C line 34564
14 => {
vm_block = 8; continue;
}
// C line 34563
15 => {
let _ = { let assigned = ((get_u32(((((*(s)).byte_code).buf).offset(((pos) as isize))).offset((((1 as i32)) as isize)))) as i32); label = assigned; assigned };
vm_block = 14; continue;
}
// C line 34561
16 => {
vm_block = 7; continue;
}
// C line 34560
17 => {
let _ = { pos = ((((pos) as i32)).wrapping_add((((*((opcode_info).as_ptr()).offset((op) as isize)).size) as i32))) as i32; pos };
vm_block = 16; continue;
}
// C line 34557
18 => {
let _ = { let assigned = ((get_u32(((((*(s)).byte_code).buf).offset(((pos) as isize))).offset((((1 as i32)) as isize)))) as i32); *(pline) = assigned; assigned };
vm_block = 17; continue;
}
// C line 34556
19 => {
vm_block = if !(pline).is_null() { 18 } else { 17 }; continue;
}
// C line 34554
20 => {
vm_block = match { let assigned = ((*(((*(s)).byte_code).buf).offset((pos) as isize)) as i32); op = assigned; assigned } { x if x == (OP_drop as i32) => 12, x if x == (OP_goto as i32) => 15, x if x == (OP_label as i32) => 17, x if x == (OP_line_num as i32) => 19, _ => 9, }; continue;
}
// C line 34552
21 => {
let _ = { let assigned = (*((*(s)).label_slots).offset((label) as isize)).pos2; pos = assigned; assigned };
vm_block = 7; continue;
}
// C line 34551
22 => {
let _ = if ((((!(((((((((label) >= ((0 as i32))) as i32)) != 0) && (((((label) < ((*(s)).label_count)) as i32)) != 0)) as i32)) != 0) as i32)) as i64)) != 0 { std::process::abort() } else { { let _ = (0 as i32); } };
vm_block = 21; continue;
}
// C line 34550
23 => {
let _ = { let assigned = (0 as i32); i = assigned; assigned };
vm_block = 5; continue;
}
// C line 34549
24 => {
let _ = update_label(s, label, ((1 as i32)).wrapping_neg());
vm_block = 23; continue;
}
// C line 34548
25 => {
let _ = { let assigned = label0; label = assigned; assigned };
vm_block = 24; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:34598. Bellard/Gordon MIT.
#[cfg(feature = "short-opcodes")]
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn push_short_int(mut bc_out: *mut DynBuf, mut val: i32) -> () {
let mut vm_local_storage = Vec::<u64>::new();
let mut vm_block: usize = 13;
loop { match vm_block {
// C line ?
0 => {
return ();
}
// C line 34617
1 => {
let _ = dbuf_put_u32(bc_out, ((val) as u32));
vm_block = 0; continue;
}
// C line 34616
2 => {
let _ = dbuf_putc(bc_out, (((OP_push_i32 as i32)) as u8));
vm_block = 1; continue;
}
// C line 34613
3 => {
return;
}
// C line 34612
4 => {
let _ = dbuf_put_u16(bc_out, ((val) as u16));
vm_block = 3; continue;
}
// C line 34611
5 => {
let _ = dbuf_putc(bc_out, (((OP_push_i16 as i32)) as u8));
vm_block = 4; continue;
}
// C line 34610
6 => {
vm_block = if ((((val) == (((((val) as i16)) as i32))) as i32)) != 0 { 5 } else { 2 }; continue;
}
// C line 34608
7 => {
return;
}
// C line 34607
8 => {
let _ = dbuf_putc(bc_out, ((val) as u8));
vm_block = 7; continue;
}
// C line 34606
9 => {
let _ = dbuf_putc(bc_out, (((OP_push_i8 as i32)) as u8));
vm_block = 8; continue;
}
// C line 34605
10 => {
vm_block = if ((((val) == (((((val) as i8)) as i32))) as i32)) != 0 { 9 } else { 6 }; continue;
}
// C line 34603
11 => {
return;
}
// C line 34602
12 => {
let _ = dbuf_putc(bc_out, ((((OP_push_0 as i32)).wrapping_add(val)) as u8));
vm_block = 11; continue;
}
// C line 34601
13 => {
vm_block = if ((((((((val) >= (((1 as i32)).wrapping_neg())) as i32)) != 0) && (((((val) <= ((7 as i32))) as i32)) != 0)) as i32)) != 0 { 12 } else { 10 }; continue;
}
_ => std::process::abort(),
} }
}
// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:34598. Bellard/Gordon MIT.
#[cfg(not(feature = "short-opcodes"))]
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn push_short_int(mut bc_out: *mut DynBuf, mut val: i32) -> () {
let mut vm_local_storage = Vec::<u64>::new();
let mut vm_block: usize = 2;
loop { match vm_block {
// C line ?
0 => {
return ();
}
// C line 34617
1 => {
let _ = dbuf_put_u32(bc_out, ((val) as u32));
vm_block = 0; continue;
}
// C line 34616
2 => {
let _ = dbuf_putc(bc_out, (((OP_push_i32 as i32)) as u8));
vm_block = 1; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:34620. Bellard/Gordon MIT.
#[cfg(feature = "short-opcodes")]
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn put_short_code(mut bc_out: *mut DynBuf, mut op: i32, mut idx: i32) -> () {
let mut vm_local_storage = Vec::<u64>::new();
let mut vm_block: usize = 35;
loop { match vm_block {
// C line ?
0 => {
return ();
}
// C line 34675
1 => {
let _ = dbuf_put_u16(bc_out, ((idx) as u16));
vm_block = 0; continue;
}
// C line 34674
2 => {
let _ = dbuf_putc(bc_out, ((op) as u8));
vm_block = 1; continue;
}
// C line 34670
3 => {
return;
}
// C line 34669
4 => {
let _ = dbuf_putc(bc_out, ((idx) as u8));
vm_block = 3; continue;
}
// C line 34668
5 => {
let _ = dbuf_putc(bc_out, (((OP_set_loc8 as i32)) as u8));
vm_block = 4; continue;
}
// C line 34666
6 => {
return;
}
// C line 34665
7 => {
let _ = dbuf_putc(bc_out, ((idx) as u8));
vm_block = 6; continue;
}
// C line 34664
8 => {
let _ = dbuf_putc(bc_out, (((OP_put_loc8 as i32)) as u8));
vm_block = 7; continue;
}
// C line 34662
9 => {
return;
}
// C line 34661
10 => {
let _ = dbuf_putc(bc_out, ((idx) as u8));
vm_block = 9; continue;
}
// C line 34660
11 => {
let _ = dbuf_putc(bc_out, (((OP_get_loc8 as i32)) as u8));
vm_block = 10; continue;
}
// C line 34658
12 => {
vm_block = match op { x if x == (OP_set_loc as i32) => 5, x if x == (OP_put_loc as i32) => 8, x if x == (OP_get_loc as i32) => 11, _ => 2, }; continue;
}
// C line 34657
13 => {
vm_block = if ((((idx) < ((256 as i32))) as i32)) != 0 { 12 } else { 2 }; continue;
}
// C line 34654
14 => {
return;
}
// C line 34653
15 => {
let _ = dbuf_putc(bc_out, ((((OP_call0 as i32)).wrapping_add(idx)) as u8));
vm_block = 14; continue;
}
// C line 34651
16 => {
return;
}
// C line 34650
17 => {
let _ = dbuf_putc(bc_out, ((((OP_set_var_ref0 as i32)).wrapping_add(idx)) as u8));
vm_block = 16; continue;
}
// C line 34648
18 => {
return;
}
// C line 34647
19 => {
let _ = dbuf_putc(bc_out, ((((OP_put_var_ref0 as i32)).wrapping_add(idx)) as u8));
vm_block = 18; continue;
}
// C line 34645
20 => {
return;
}
// C line 34644
21 => {
let _ = dbuf_putc(bc_out, ((((OP_get_var_ref0 as i32)).wrapping_add(idx)) as u8));
vm_block = 20; continue;
}
// C line 34642
22 => {
return;
}
// C line 34641
23 => {
let _ = dbuf_putc(bc_out, ((((OP_set_arg0 as i32)).wrapping_add(idx)) as u8));
vm_block = 22; continue;
}
// C line 34639
24 => {
return;
}
// C line 34638
25 => {
let _ = dbuf_putc(bc_out, ((((OP_put_arg0 as i32)).wrapping_add(idx)) as u8));
vm_block = 24; continue;
}
// C line 34636
26 => {
return;
}
// C line 34635
27 => {
let _ = dbuf_putc(bc_out, ((((OP_get_arg0 as i32)).wrapping_add(idx)) as u8));
vm_block = 26; continue;
}
// C line 34633
28 => {
return;
}
// C line 34632
29 => {
let _ = dbuf_putc(bc_out, ((((OP_set_loc0 as i32)).wrapping_add(idx)) as u8));
vm_block = 28; continue;
}
// C line 34630
30 => {
return;
}
// C line 34629
31 => {
let _ = dbuf_putc(bc_out, ((((OP_put_loc0 as i32)).wrapping_add(idx)) as u8));
vm_block = 30; continue;
}
// C line 34627
32 => {
return;
}
// C line 34626
33 => {
let _ = dbuf_putc(bc_out, ((((OP_get_loc0 as i32)).wrapping_add(idx)) as u8));
vm_block = 32; continue;
}
// C line 34624
34 => {
vm_block = match op { x if x == (OP_call as i32) => 15, x if x == (OP_set_var_ref as i32) => 17, x if x == (OP_put_var_ref as i32) => 19, x if x == (OP_get_var_ref as i32) => 21, x if x == (OP_set_arg as i32) => 23, x if x == (OP_put_arg as i32) => 25, x if x == (OP_get_arg as i32) => 27, x if x == (OP_set_loc as i32) => 29, x if x == (OP_put_loc as i32) => 31, x if x == (OP_get_loc as i32) => 33, _ => 13, }; continue;
}
// C line 34623
35 => {
vm_block = if ((((idx) < ((4 as i32))) as i32)) != 0 { 34 } else { 13 }; continue;
}
_ => std::process::abort(),
} }
}
// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:34620. Bellard/Gordon MIT.
#[cfg(not(feature = "short-opcodes"))]
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn put_short_code(mut bc_out: *mut DynBuf, mut op: i32, mut idx: i32) -> () {
let mut vm_local_storage = Vec::<u64>::new();
let mut vm_block: usize = 2;
loop { match vm_block {
// C line ?
0 => {
return ();
}
// C line 34675
1 => {
let _ = dbuf_put_u16(bc_out, ((idx) as u16));
vm_block = 0; continue;
}
// C line 34674
2 => {
let _ = dbuf_putc(bc_out, ((op) as u8));
vm_block = 1; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:34679. Bellard/Gordon MIT.
#[cfg(feature = "short-opcodes")]
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn resolve_labels(mut ctx: *mut JSContext, mut s: *mut JSFunctionDef) -> i32 {
let mut vm_local_storage = Vec::<u64>::new();
let mut pos: i32 = 0;
let mut pos_next: i32 = 0;
let mut bc_len: i32 = 0;
let mut op: i32 = 0;
let mut op1: i32 = 0;
let mut len: i32 = 0;
let mut i: i32 = 0;
let mut line_num: i32 = 0;
let mut bc_buf: *const u8 = core::ptr::null();
let mut bc_out: DynBuf = core::mem::zeroed();
let mut label_slots: *mut LabelSlot = core::ptr::null_mut();
let mut ls: *mut LabelSlot = core::ptr::null_mut();
let mut re: *mut RelocEntry = core::ptr::null_mut();
let mut re_next: *mut RelocEntry = core::ptr::null_mut();
let mut cc: CodeContext = core::mem::zeroed();
let mut label: i32 = 0;
let mut jp: *mut JumpSlot = core::ptr::null_mut();
let mut val: i32 = 0;
let mut diff: i32 = 0;
let mut argc: i32 = 0;
let mut line1: i32 = 0;
let mut pos1: i32 = 0;
let mut line1_1: i32 = 0;
let mut diff_1: i32 = 0;
let mut diff_2: i32 = 0;
let mut atom: JSAtom = 0;
let mut is_with: i32 = 0;
let mut idx: i32 = 0;
let mut atom_1: JSAtom = 0;
let mut atom_2: JSAtom = 0;
let mut op1_1: i32 = 0;
let mut line2: i32 = 0;
let mut idx_1: i32 = 0;
let mut idx_2: i32 = 0;
let mut idx_3: i32 = 0;
let mut op1_2: i32 = 0;
let mut idx_4: i32 = 0;
let mut op1_3: i32 = 0;
let mut op2: i32 = 0;
let mut patch_offsets: i32 = 0;
let mut ls_1: *mut LabelSlot = core::ptr::null_mut();
let mut jp1: *mut JumpSlot = core::ptr::null_mut();
let mut j: i32 = 0;
let mut pos_1: i32 = 0;
let mut diff_3: i32 = 0;
let mut delta: i32 = 0;
let mut jp1_1: *mut JumpSlot = core::ptr::null_mut();
let mut j_1: i32 = 0;
let mut diff1: i32 = 0;
let mut vm_block: usize = 589;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 35579
1 => {
return ((1 as i32)).wrapping_neg();
}
// C line 35578 labels: fail
2 => {
let _ = dbuf_free(core::ptr::addr_of_mut!(bc_out));
vm_block = 1; continue;
}
// C line 35575
3 => {
return (0 as i32);
}
// C line 35573
4 => {
return ((1 as i32)).wrapping_neg();
}
// C line 35572
5 => {
let _ = JS_ThrowOutOfMemory(ctx);
vm_block = 4; continue;
}
// C line 35571
6 => {
vm_block = if (dbuf_error(core::ptr::addr_of_mut!((*(s)).byte_code))) != 0 { 5 } else { 3 }; continue;
}
// C line 35570
7 => {
let _ = { let assigned = (1 as i32); (*(s)).use_short_opcodes = assigned; assigned };
vm_block = 6; continue;
}
// C line 35569
8 => {
let _ = { (*(s)).byte_code = core::ptr::read(core::ptr::addr_of!(bc_out)); core::ptr::read(core::ptr::addr_of!((*(s)).byte_code)) };
vm_block = 7; continue;
}
// C line 35568
9 => {
let _ = dbuf_free(core::ptr::addr_of_mut!((*(s)).byte_code));
vm_block = 8; continue;
}
// C line 35566
10 => {
let _ = { let assigned = core::ptr::null_mut::<LineNumberSlot>(); (*(s)).line_number_slots = assigned; assigned };
vm_block = 9; continue;
}
// C line 35565
11 => {
let _ = js_free(ctx, (((*(s)).line_number_slots) as *mut c_void));
vm_block = 10; continue;
}
// C line 35564
12 => {
let _ = compute_pc2line_info(s);
vm_block = 11; continue;
}
// C line 35562
13 => {
let _ = { let assigned = core::ptr::null_mut::<LabelSlot>(); (*(s)).label_slots = assigned; assigned };
vm_block = 12; continue;
}
// C line 35561
14 => {
let _ = js_free(ctx, (((*(s)).label_slots) as *mut c_void));
vm_block = 13; continue;
}
// C line 35559
15 => {
let _ = { let assigned = core::ptr::null_mut::<JumpSlot>(); (*(s)).jump_slots = assigned; assigned };
vm_block = 14; continue;
}
// C line 35558
16 => {
let _ = js_free(ctx, (((*(s)).jump_slots) as *mut c_void));
vm_block = 15; continue;
}
// C line 35542
17 => {
vm_block = if ((((j_1) < ((*(s)).jump_count)) as i32)) != 0 { 26 } else { 16 }; continue;
}
// C line 35542
18 => {
let _ = { let _ = { let old = j_1; j_1 = (j_1).wrapping_add(1); old }; { let old = jp1_1; jp1_1 = (jp1_1).offset(1); old } };
vm_block = 17; continue;
}
// C line 35553
19 => {
vm_block = 18; continue;
}
// C line 35552
20 => {
let _ = put_u32(((bc_out).buf).offset((((*(jp1_1)).pos) as isize)), ((diff1) as u32));
vm_block = 19; continue;
}
// C line 35550
21 => {
vm_block = 18; continue;
}
// C line 35549
22 => {
let _ = crate::cutils_header::put_u16(((bc_out).buf).offset((((*(jp1_1)).pos) as isize)), ((diff1) as u16));
vm_block = 21; continue;
}
// C line 35547
23 => {
vm_block = 18; continue;
}
// C line 35546
24 => {
let _ = crate::cutils_header::put_u8(((bc_out).buf).offset((((*(jp1_1)).pos) as isize)), ((diff1) as u8));
vm_block = 23; continue;
}
// C line 35544
25 => {
vm_block = match (*(jp1_1)).size { x if x == (4 as i32) => 20, x if x == (2 as i32) => 22, x if x == (1 as i32) => 24, _ => 18, }; continue;
}
// C line 35543
26 => {
diff1 = ((*((*(s)).label_slots).offset(((*(jp1_1)).label) as isize)).addr).wrapping_sub((*(jp1_1)).pos);
vm_block = 25; continue;
}
// C line 35542
27 => {
let _ = { let _ = { let assigned = (0 as i32); j_1 = assigned; assigned }; { let assigned = (*(s)).jump_slots; jp1_1 = assigned; assigned } };
vm_block = 17; continue;
}
// C line 35539
28 => {
vm_block = if (patch_offsets) != 0 { 27 } else { 16 }; continue;
}
// C line 35486
29 => {
vm_block = if ((((i) < ((*(s)).jump_count)) as i32)) != 0 { 65 } else { 28 }; continue;
}
// C line 35486
30 => {
let _ = { let _ = { let old = i; i = (i).wrapping_add(1); old }; { let old = jp; jp = (jp).offset(1); old } };
vm_block = 29; continue;
}
// C line 35536
31 => {
vm_block = 30; continue;
}
// C line 35509
32 => {
vm_block = 55; continue;
}
// C line 35505
33 => {
let _ = { let assigned = (({ let assigned = (OP_goto8 as i32); (*(jp)).op = assigned; assigned }) as u8); *((bc_out).buf).offset(((pos_1).wrapping_sub((1 as i32))) as isize) = assigned; assigned };
vm_block = 32; continue;
}
// C line 35507
34 => {
let _ = { let assigned = (({ let assigned = ((OP_if_false8 as i32)).wrapping_add((op).wrapping_sub((OP_if_false as i32))); (*(jp)).op = assigned; assigned }) as u8); *((bc_out).buf).offset(((pos_1).wrapping_sub((1 as i32))) as isize) = assigned; assigned };
vm_block = 32; continue;
}
// C line 35504
35 => {
vm_block = if ((((op) == ((OP_goto16 as i32))) as i32)) != 0 { 33 } else { 34 }; continue;
}
// C line 35503
36 => {
let _ = { let assigned = (1 as i32); (*(jp)).size = assigned; assigned };
vm_block = 35; continue;
}
// C line 35534
37 => {
vm_block = 30; continue;
}
// C line 35530
38 => {
vm_block = if ((((j) < ((*(s)).line_number_count)) as i32)) != 0 { 41 } else { 37 }; continue;
}
// C line 35530
39 => {
let _ = { let old = j; j = (j).wrapping_add(1); old };
vm_block = 38; continue;
}
// C line 35532
40 => {
let _ = { (*((*(s)).line_number_slots).offset((j) as isize)).pc = (((((*((*(s)).line_number_slots).offset((j) as isize)).pc) as u32)).wrapping_sub(((delta) as u32))) as u32; (*((*(s)).line_number_slots).offset((j) as isize)).pc };
vm_block = 39; continue;
}
// C line 35531
41 => {
vm_block = if (((((*((*(s)).line_number_slots).offset((j) as isize)).pc) > (((pos_1) as u32))) as i32)) != 0 { 40 } else { 39 }; continue;
}
// C line 35530
42 => {
let _ = { let assigned = (0 as i32); j = assigned; assigned };
vm_block = 38; continue;
}
// C line 35526
43 => {
vm_block = if ((((j) < ((*(s)).jump_count)) as i32)) != 0 { 46 } else { 42 }; continue;
}
// C line 35526
44 => {
let _ = { let _ = { let old = j; j = (j).wrapping_add(1); old }; { let old = jp1; jp1 = (jp1).offset(1); old } };
vm_block = 43; continue;
}
// C line 35528
45 => {
let _ = { (*(jp1)).pos = (((((*(jp1)).pos) as i32)).wrapping_sub(delta)) as i32; (*(jp1)).pos };
vm_block = 44; continue;
}
// C line 35527
46 => {
vm_block = if (((((*(jp1)).pos) > (pos_1)) as i32)) != 0 { 45 } else { 44 }; continue;
}
// C line 35526
47 => {
let _ = { let _ = { let assigned = (i).wrapping_add((1 as i32)); j = assigned; assigned }; { let assigned = (jp).offset((((1 as i32)) as isize)); jp1 = assigned; assigned } };
vm_block = 43; continue;
}
// C line 35522
48 => {
vm_block = if ((((j) < ((*(s)).label_count)) as i32)) != 0 { 51 } else { 47 }; continue;
}
// C line 35522
49 => {
let _ = { let _ = { let old = j; j = (j).wrapping_add(1); old }; { let old = ls_1; ls_1 = (ls_1).offset(1); old } };
vm_block = 48; continue;
}
// C line 35524
50 => {
let _ = { (*(ls_1)).addr = (((((*(ls_1)).addr) as i32)).wrapping_sub(delta)) as i32; (*(ls_1)).addr };
vm_block = 49; continue;
}
// C line 35523
51 => {
vm_block = if (((((*(ls_1)).addr) > (pos_1)) as i32)) != 0 { 50 } else { 49 }; continue;
}
// C line 35522
52 => {
let _ = { let _ = { let assigned = (0 as i32); j = assigned; assigned }; { let assigned = (*(s)).label_slots; ls_1 = assigned; assigned } };
vm_block = 48; continue;
}
// C line 35521
53 => {
let _ = { let old = patch_offsets; patch_offsets = (patch_offsets).wrapping_add(1); old };
vm_block = 52; continue;
}
// C line 35520
54 => {
let _ = { (bc_out).size = (((((bc_out).size) as usize)).wrapping_sub(((delta) as usize))) as usize; (bc_out).size };
vm_block = 53; continue;
}
// C line 35518 labels: shrink
55 => {
let _ = { let dst = ((((((bc_out).buf).offset(((pos_1) as isize))).offset((((*(jp)).size) as isize))) as *mut c_void)) as *mut u8; core::ptr::copy((((((((bc_out).buf).offset(((pos_1) as isize))).offset((((*(jp)).size) as isize))).offset(((delta) as isize))) as *const c_void)) as *const u8, dst, (((((bc_out).size).wrapping_sub(((pos_1) as usize))).wrapping_sub((((*(jp)).size) as usize))).wrapping_sub(((delta) as usize))) as usize); dst as *mut c_void };
vm_block = 54; continue;
}
// C line 35515
56 => {
let _ = { let assigned = (({ let assigned = (OP_goto16 as i32); (*(jp)).op = assigned; assigned }) as u8); *((bc_out).buf).offset(((pos_1).wrapping_sub((1 as i32))) as isize) = assigned; assigned };
vm_block = 55; continue;
}
// C line 35514
57 => {
let _ = { let assigned = (2 as i32); delta = assigned; assigned };
vm_block = 56; continue;
}
// C line 35513
58 => {
let _ = { let assigned = (2 as i32); (*(jp)).size = assigned; assigned };
vm_block = 57; continue;
}
// C line 35511
59 => {
vm_block = if ((((((((diff_3) == (((((diff_3) as i16)) as i32))) as i32)) != 0) && (((((op) == ((OP_goto as i32))) as i32)) != 0)) as i32)) != 0 { 58 } else { 31 }; continue;
}
// C line 35501
60 => {
vm_block = if ((((((((diff_3) >= (((128 as i32)).wrapping_neg())) as i32)) != 0) && (((((diff_3) <= (((127 as i32)).wrapping_add(delta))) as i32)) != 0)) as i32)) != 0 { 36 } else { 59 }; continue;
}
// C line 35500
61 => {
let _ = { let assigned = ((*((*(s)).label_slots).offset(((*(jp)).label) as isize)).addr).wrapping_sub(pos_1); diff_3 = assigned; assigned };
vm_block = 60; continue;
}
// C line 35499
62 => {
let _ = { let assigned = (*(jp)).pos; pos_1 = assigned; assigned };
vm_block = 61; continue;
}
// C line 35494
63 => {
let _ = { let assigned = (1 as i32); delta = assigned; assigned };
vm_block = 62; continue;
}
// C line 35492
64 => {
vm_block = match { let assigned = (*(jp)).op; op = assigned; assigned } { x if x == (OP_goto as i32) => 62, x if x == (OP_if_true as i32) => 62, x if x == (OP_if_false as i32) => 62, x if x == (OP_goto16 as i32) => 63, _ => 30, }; continue;
}
// C line 35491
65 => {
let _ = { let assigned = (3 as i32); delta = assigned; assigned };
vm_block = 64; continue;
}
// C line 35486
66 => {
let _ = { let _ = { let assigned = (0 as i32); i = assigned; assigned }; { let assigned = (*(s)).jump_slots; jp = assigned; assigned } };
vm_block = 29; continue;
}
// C line 35485
67 => {
patch_offsets = (0 as i32);
vm_block = 66; continue;
}
// C line 35483
68 => {
vm_block = if ((1 as i32)) != 0 { 67 } else { 16 }; continue;
}
// C line 35479
69 => {
vm_block = if ((((i) < ((*(s)).label_count)) as i32)) != 0 { 71 } else { 68 }; continue;
}
// C line 35479
70 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 69; continue;
}
// C line 35480
71 => {
let _ = if ((((!((((((*(label_slots).offset((i) as isize)).first_reloc) == (core::ptr::null_mut::<RelocEntry>())) as i32)) != 0) as i32)) as i64)) != 0 { std::process::abort() } else { { let _ = (0 as i32); } };
vm_block = 70; continue;
}
// C line 35479
72 => {
let _ = { let assigned = (0 as i32); i = assigned; assigned };
vm_block = 69; continue;
}
// C line 34781
73 => {
vm_block = if ((((pos) < (bc_len)) as i32)) != 0 { 530 } else { 72 }; continue;
}
// C line 34781
74 => {
let _ = { let assigned = pos_next; pos = assigned; assigned };
vm_block = 73; continue;
}
// C line 35474
75 => {
vm_block = 74; continue;
}
// C line 35473
76 => {
let _ = crate::cutils::dbuf_put(core::ptr::addr_of_mut!(bc_out), (bc_buf).offset(((pos) as isize)), ((len) as usize));
vm_block = 75; continue;
}
// C line 35472 labels: no_change
77 => {
let _ = add_pc2line_info(s, (((bc_out).size) as u32), ((line_num) as u32));
vm_block = 76; continue;
}
// C line 35467
78 => {
vm_block = 77; continue;
}
// C line 35462
79 => {
vm_block = 448; continue;
}
// C line 35461
80 => {
let _ = { let assigned = (OP_if_true as i32); op = assigned; assigned };
vm_block = 79; continue;
}
// C line 35460
81 => {
let _ = { let assigned = (cc).label; label = assigned; assigned };
vm_block = 80; continue;
}
// C line 35459
82 => {
let _ = { let assigned = (cc).pos; pos_next = assigned; assigned };
vm_block = 81; continue;
}
// C line 35458
83 => {
let _ = JS_FreeAtom(ctx, (cc).atom);
vm_block = 82; continue;
}
// C line 35457
84 => {
let _ = dbuf_putc(core::ptr::addr_of_mut!(bc_out), ((op2) as u8));
vm_block = 83; continue;
}
// C line 35456
85 => {
let _ = add_pc2line_info(s, (((bc_out).size) as u32), ((line_num) as u32));
vm_block = 84; continue;
}
// C line 35455
86 => {
let _ = { let assigned = (cc).line_num; line_num = assigned; assigned };
vm_block = 85; continue;
}
// C line 35455
87 => {
vm_block = if (((((cc).line_num) >= ((0 as i32))) as i32)) != 0 { 86 } else { 85 }; continue;
}
// C line 35453
88 => {
vm_block = if ((((((((op1_3) == ((OP_strict_neq as i32))) as i32)) != 0) && ((code_match(core::ptr::addr_of_mut!(cc), (cc).pos, &[((OP_if_false as i32)) as i32, (((1 as i32)).wrapping_neg()) as i32])) != 0)) as i32)) != 0 { 87 } else { 78 }; continue;
}
// C line 35451
89 => {
vm_block = 74; continue;
}
// C line 35450
90 => {
let _ = { let assigned = (cc).pos; pos_next = assigned; assigned };
vm_block = 89; continue;
}
// C line 35449
91 => {
let _ = JS_FreeAtom(ctx, (cc).atom);
vm_block = 90; continue;
}
// C line 35448
92 => {
let _ = dbuf_putc(core::ptr::addr_of_mut!(bc_out), ((op2) as u8));
vm_block = 91; continue;
}
// C line 35447
93 => {
let _ = add_pc2line_info(s, (((bc_out).size) as u32), ((line_num) as u32));
vm_block = 92; continue;
}
// C line 35446
94 => {
vm_block = if ((((op1_3) == ((OP_strict_eq as i32))) as i32)) != 0 { 93 } else { 88 }; continue;
}
// C line 35444
95 => {
vm_block = if ((((op2) >= ((0 as i32))) as i32)) != 0 { 94 } else { 78 }; continue;
}
// C line 35442
96 => {
vm_block = 95; continue;
}
// C line 35441
97 => {
let _ = { let assigned = (OP_typeof_is_function as i32); op2 = assigned; assigned };
vm_block = 96; continue;
}
// C line 35439
98 => {
vm_block = 95; continue;
}
// C line 35438
99 => {
let _ = { let assigned = (OP_typeof_is_undefined as i32); op2 = assigned; assigned };
vm_block = 98; continue;
}
// C line 35436
100 => {
vm_block = match (cc).atom { x if x == (((crate::quickjs_atom::JS_ATOM_function as i32)) as JSAtom) => 97, x if x == (((crate::quickjs_atom::JS_ATOM_undefined as i32)) as JSAtom) => 99, _ => 95, }; continue;
}
// C line 35435
101 => {
op2 = ((1 as i32)).wrapping_neg();
vm_block = 100; continue;
}
// C line 35434
102 => {
op1_3 = if (((((((((cc).op) == ((OP_strict_eq as i32))) as i32)) != 0) || ((((((cc).op) == ((OP_eq as i32))) as i32)) != 0)) as i32)) != 0 { (OP_strict_eq as i32) } else { (OP_strict_neq as i32) };
vm_block = 101; continue;
}
// C line 35433
103 => {
let _ = { let assigned = (cc).line_num; line_num = assigned; assigned };
vm_block = 102; continue;
}
// C line 35433
104 => {
vm_block = if (((((cc).line_num) >= ((0 as i32))) as i32)) != 0 { 103 } else { 102 }; continue;
}
// C line 35432
105 => {
vm_block = if (code_match(core::ptr::addr_of_mut!(cc), pos_next, &[((OP_push_atom_value as i32)) as i32, ((((((((OP_strict_eq as i32)) | (((OP_strict_neq as i32)).wrapping_shl(((8 as i32)) as u32)))) | (((OP_eq as i32)).wrapping_shl(((16 as i32)) as u32)))) | (((OP_neq as i32)).wrapping_shl(((24 as i32)) as u32)))) as i32, (((1 as i32)).wrapping_neg()) as i32])) != 0 { 104 } else { 78 }; continue;
}
// C line 35430
106 => {
vm_block = if ((1 as i32)) != 0 { 105 } else { 78 }; continue;
}
// C line 35426
107 => {
vm_block = 77; continue;
}
// C line 35423
108 => {
vm_block = 74; continue;
}
// C line 35422
109 => {
let _ = { let assigned = (cc).pos; pos_next = assigned; assigned };
vm_block = 108; continue;
}
// C line 35421
110 => {
let _ = dbuf_putc(core::ptr::addr_of_mut!(bc_out), (((OP_put_array_el as i32)) as u8));
vm_block = 109; continue;
}
// C line 35420
111 => {
let _ = dbuf_putc(core::ptr::addr_of_mut!(bc_out), ((((OP_dec as i32)).wrapping_add((op).wrapping_sub((OP_post_dec as i32)))) as u8));
vm_block = 110; continue;
}
// C line 35419
112 => {
let _ = add_pc2line_info(s, (((bc_out).size) as u32), ((line_num) as u32));
vm_block = 111; continue;
}
// C line 35418
113 => {
let _ = { let assigned = (cc).line_num; line_num = assigned; assigned };
vm_block = 112; continue;
}
// C line 35418
114 => {
vm_block = if (((((cc).line_num) >= ((0 as i32))) as i32)) != 0 { 113 } else { 112 }; continue;
}
// C line 35417
115 => {
vm_block = if (code_match(core::ptr::addr_of_mut!(cc), pos_next, &[((OP_perm4 as i32)) as i32, ((OP_put_array_el as i32)) as i32, ((OP_drop as i32)) as i32, (((1 as i32)).wrapping_neg()) as i32])) != 0 { 114 } else { 107 }; continue;
}
// C line 35415
116 => {
vm_block = 74; continue;
}
// C line 35414
117 => {
let _ = { let assigned = (cc).pos; pos_next = assigned; assigned };
vm_block = 116; continue;
}
// C line 35413
118 => {
let _ = dbuf_put_u32(core::ptr::addr_of_mut!(bc_out), (cc).atom);
vm_block = 117; continue;
}
// C line 35412
119 => {
let _ = dbuf_putc(core::ptr::addr_of_mut!(bc_out), (((OP_put_field as i32)) as u8));
vm_block = 118; continue;
}
// C line 35411
120 => {
let _ = dbuf_putc(core::ptr::addr_of_mut!(bc_out), ((((OP_dec as i32)).wrapping_add((op).wrapping_sub((OP_post_dec as i32)))) as u8));
vm_block = 119; continue;
}
// C line 35410
121 => {
let _ = add_pc2line_info(s, (((bc_out).size) as u32), ((line_num) as u32));
vm_block = 120; continue;
}
// C line 35409
122 => {
let _ = { let assigned = (cc).line_num; line_num = assigned; assigned };
vm_block = 121; continue;
}
// C line 35409
123 => {
vm_block = if (((((cc).line_num) >= ((0 as i32))) as i32)) != 0 { 122 } else { 121 }; continue;
}
// C line 35408
124 => {
vm_block = if (code_match(core::ptr::addr_of_mut!(cc), pos_next, &[((OP_perm3 as i32)) as i32, ((OP_put_field as i32)) as i32, ((OP_drop as i32)) as i32, (((1 as i32)).wrapping_neg()) as i32])) != 0 { 123 } else { 115 }; continue;
}
// C line 35406
125 => {
vm_block = 74; continue;
}
// C line 35405
126 => {
let _ = put_short_code(core::ptr::addr_of_mut!(bc_out), op1_2, idx_4);
vm_block = 125; continue;
}
// C line 35404
127 => {
let _ = dbuf_putc(core::ptr::addr_of_mut!(bc_out), ((((OP_dec as i32)).wrapping_add((op).wrapping_sub((OP_post_dec as i32)))) as u8));
vm_block = 126; continue;
}
// C line 35403
128 => {
let _ = add_pc2line_info(s, (((bc_out).size) as u32), ((line_num) as u32));
vm_block = 127; continue;
}
// C line 35401
129 => {
let _ = { let assigned = (cc).pos; pos_next = assigned; assigned };
vm_block = 128; continue;
}
// C line 35400
130 => {
let _ = { op1_2 = ((((op1_2) as i32)).wrapping_add((1 as i32))) as i32; op1_2 };
vm_block = 129; continue;
}
// C line 35399
131 => {
let _ = { let assigned = (cc).line_num; line_num = assigned; assigned };
vm_block = 130; continue;
}
// C line 35399
132 => {
vm_block = if (((((cc).line_num) >= ((0 as i32))) as i32)) != 0 { 131 } else { 130 }; continue;
}
// C line 35398
133 => {
vm_block = if (code_match(core::ptr::addr_of_mut!(cc), (cc).pos, &[((op1_2).wrapping_sub((1 as i32))) as i32, (idx_4) as i32, (((1 as i32)).wrapping_neg()) as i32])) != 0 { 132 } else { 128 }; continue;
}
// C line 35397
134 => {
let _ = { let assigned = (cc).pos; pos_next = assigned; assigned };
vm_block = 133; continue;
}
// C line 35396
135 => {
let _ = { let assigned = (cc).idx; idx_4 = assigned; assigned };
vm_block = 134; continue;
}
// C line 35395
136 => {
let _ = { let assigned = (cc).op; op1_2 = assigned; assigned };
vm_block = 135; continue;
}
// C line 35394
137 => {
let _ = { let assigned = (cc).line_num; line_num = assigned; assigned };
vm_block = 136; continue;
}
// C line 35394
138 => {
vm_block = if (((((cc).line_num) >= ((0 as i32))) as i32)) != 0 { 137 } else { 136 }; continue;
}
// C line 35393
139 => {
vm_block = if (code_match(core::ptr::addr_of_mut!(cc), pos_next, &[((((((OP_put_loc as i32)) | (((OP_put_arg as i32)).wrapping_shl(((8 as i32)) as u32)))) | (((OP_put_var_ref as i32)).wrapping_shl(((16 as i32)) as u32)))) as i32, (((1 as i32)).wrapping_neg()) as i32, ((OP_drop as i32)) as i32, (((1 as i32)).wrapping_neg()) as i32])) != 0 { 138 } else { 124 }; continue;
}
// C line 35386
140 => {
vm_block = if ((1 as i32)) != 0 { 139 } else { 107 }; continue;
}
// C line 35382
141 => {
vm_block = 77; continue;
}
// C line 35380
142 => {
vm_block = 74; continue;
}
// C line 35379
143 => {
let _ = put_short_code(core::ptr::addr_of_mut!(bc_out), op, idx_3);
vm_block = 142; continue;
}
// C line 35378
144 => {
let _ = add_pc2line_info(s, (((bc_out).size) as u32), ((line_num) as u32));
vm_block = 143; continue;
}
// C line 35376
145 => {
vm_block = 74; continue;
}
// C line 35375
146 => {
let _ = { let assigned = (cc).pos; pos_next = assigned; assigned };
vm_block = 145; continue;
}
// C line 35374
147 => {
let _ = put_short_code(core::ptr::addr_of_mut!(bc_out), (op).wrapping_add((1 as i32)), idx_3);
vm_block = 146; continue;
}
// C line 35373
148 => {
let _ = add_pc2line_info(s, (((bc_out).size) as u32), ((line_num) as u32));
vm_block = 147; continue;
}
// C line 35372
149 => {
let _ = { let assigned = (cc).line_num; line_num = assigned; assigned };
vm_block = 148; continue;
}
// C line 35372
150 => {
vm_block = if (((((cc).line_num) >= ((0 as i32))) as i32)) != 0 { 149 } else { 148 }; continue;
}
// C line 35371
151 => {
vm_block = if (code_match(core::ptr::addr_of_mut!(cc), pos_next, &[((op).wrapping_sub((1 as i32))) as i32, (idx_3) as i32, (((1 as i32)).wrapping_neg()) as i32])) != 0 { 150 } else { 144 }; continue;
}
// C line 35370
152 => {
let _ = { let assigned = ((get_u16(((bc_buf).offset(((pos) as isize))).offset((((1 as i32)) as isize)))) as i32); idx_3 = assigned; assigned };
vm_block = 151; continue;
}
// C line 35367
153 => {
vm_block = if ((1 as i32)) != 0 { 152 } else { 141 }; continue;
}
// C line 35361
154 => {
vm_block = 77; continue;
}
// C line 35359
155 => {
vm_block = 74; continue;
}
// C line 35358
156 => {
let _ = put_short_code(core::ptr::addr_of_mut!(bc_out), op, idx_2);
vm_block = 155; continue;
}
// C line 35357
157 => {
let _ = add_pc2line_info(s, (((bc_out).size) as u32), ((line_num) as u32));
vm_block = 156; continue;
}
// C line 35356
158 => {
let _ = { let assigned = ((get_u16(((bc_buf).offset(((pos) as isize))).offset((((1 as i32)) as isize)))) as i32); idx_2 = assigned; assigned };
vm_block = 157; continue;
}
// C line 35354
159 => {
vm_block = if ((1 as i32)) != 0 { 158 } else { 154 }; continue;
}
// C line 35350
160 => {
vm_block = 77; continue;
}
// C line 35348
161 => {
vm_block = 74; continue;
}
// C line 35347
162 => {
let _ = put_short_code(core::ptr::addr_of_mut!(bc_out), op, idx_1);
vm_block = 161; continue;
}
// C line 35346
163 => {
let _ = add_pc2line_info(s, (((bc_out).size) as u32), ((line_num) as u32));
vm_block = 162; continue;
}
// C line 35344
164 => {
vm_block = 74; continue;
}
// C line 35343
165 => {
let _ = { let assigned = (cc).pos; pos_next = assigned; assigned };
vm_block = 164; continue;
}
// C line 35342
166 => {
let _ = dbuf_putc(core::ptr::addr_of_mut!(bc_out), ((idx_1) as u8));
vm_block = 165; continue;
}
// C line 35341
167 => {
let _ = dbuf_putc(core::ptr::addr_of_mut!(bc_out), (((OP_add_loc as i32)) as u8));
vm_block = 166; continue;
}
// C line 35340
168 => {
let _ = put_short_code(core::ptr::addr_of_mut!(bc_out), (cc).op, (cc).idx);
vm_block = 167; continue;
}
// C line 35339
169 => {
let _ = add_pc2line_info(s, (((bc_out).size) as u32), ((line_num) as u32));
vm_block = 168; continue;
}
// C line 35338
170 => {
let _ = { let assigned = (cc).line_num; line_num = assigned; assigned };
vm_block = 169; continue;
}
// C line 35338
171 => {
vm_block = if (((((cc).line_num) >= ((0 as i32))) as i32)) != 0 { 170 } else { 169 }; continue;
}
// C line 35337
172 => {
vm_block = if (code_match(core::ptr::addr_of_mut!(cc), pos_next, &[((((((OP_get_loc as i32)) | (((OP_get_arg as i32)).wrapping_shl(((8 as i32)) as u32)))) | (((OP_get_var_ref as i32)).wrapping_shl(((16 as i32)) as u32)))) as i32, (((1 as i32)).wrapping_neg()) as i32, ((OP_add as i32)) as i32, ((OP_dup as i32)) as i32, ((OP_put_loc as i32)) as i32, (idx_1) as i32, ((OP_drop as i32)) as i32, (((1 as i32)).wrapping_neg()) as i32])) != 0 { 171 } else { 163 }; continue;
}
// C line 35330
173 => {
vm_block = 74; continue;
}
// C line 35329
174 => {
let _ = { let assigned = (cc).pos; pos_next = assigned; assigned };
vm_block = 173; continue;
}
// C line 35328
175 => {
let _ = dbuf_putc(core::ptr::addr_of_mut!(bc_out), ((idx_1) as u8));
vm_block = 174; continue;
}
// C line 35327
176 => {
let _ = dbuf_putc(core::ptr::addr_of_mut!(bc_out), (((OP_add_loc as i32)) as u8));
vm_block = 175; continue;
}
// C line 35326
177 => {
let _ = push_short_int(core::ptr::addr_of_mut!(bc_out), (cc).label);
vm_block = 176; continue;
}
// C line 35325
178 => {
let _ = add_pc2line_info(s, (((bc_out).size) as u32), ((line_num) as u32));
vm_block = 177; continue;
}
// C line 35324
179 => {
let _ = { let assigned = (cc).line_num; line_num = assigned; assigned };
vm_block = 178; continue;
}
// C line 35324
180 => {
vm_block = if (((((cc).line_num) >= ((0 as i32))) as i32)) != 0 { 179 } else { 178 }; continue;
}
// C line 35323
181 => {
vm_block = if (code_match(core::ptr::addr_of_mut!(cc), pos_next, &[((OP_push_i32 as i32)) as i32, ((OP_add as i32)) as i32, ((OP_dup as i32)) as i32, ((OP_put_loc as i32)) as i32, (idx_1) as i32, ((OP_drop as i32)) as i32, (((1 as i32)).wrapping_neg()) as i32])) != 0 { 180 } else { 172 }; continue;
}
// C line 35318
182 => {
vm_block = 74; continue;
}
// C line 35317
183 => {
let _ = { let assigned = (cc).pos; pos_next = assigned; assigned };
vm_block = 182; continue;
}
// C line 35316
184 => {
let _ = dbuf_putc(core::ptr::addr_of_mut!(bc_out), ((idx_1) as u8));
vm_block = 183; continue;
}
// C line 35315
185 => {
let _ = dbuf_putc(core::ptr::addr_of_mut!(bc_out), (((OP_add_loc as i32)) as u8));
vm_block = 184; continue;
}
// C line 35308
186 => {
let _ = dbuf_putc(core::ptr::addr_of_mut!(bc_out), (((OP_push_empty_string as i32)) as u8));
vm_block = 185; continue;
}
// C line 35307
187 => {
let _ = JS_FreeAtom(ctx, (cc).atom);
vm_block = 186; continue;
}
// C line 35313
188 => {
let _ = dbuf_put_u32(core::ptr::addr_of_mut!(bc_out), (cc).atom);
vm_block = 185; continue;
}
// C line 35312
189 => {
let _ = dbuf_putc(core::ptr::addr_of_mut!(bc_out), (((OP_push_atom_value as i32)) as u8));
vm_block = 188; continue;
}
// C line 35306
190 => {
vm_block = if (((((cc).atom) == ((((crate::quickjs_atom::JS_ATOM_empty_string as i32)) as JSAtom))) as i32)) != 0 { 187 } else { 189 }; continue;
}
// C line 35304
191 => {
let _ = add_pc2line_info(s, (((bc_out).size) as u32), ((line_num) as u32));
vm_block = 190; continue;
}
// C line 35303
192 => {
let _ = { let assigned = (cc).line_num; line_num = assigned; assigned };
vm_block = 191; continue;
}
// C line 35303
193 => {
vm_block = if (((((cc).line_num) >= ((0 as i32))) as i32)) != 0 { 192 } else { 191 }; continue;
}
// C line 35302
194 => {
vm_block = if (code_match(core::ptr::addr_of_mut!(cc), pos_next, &[((OP_push_atom_value as i32)) as i32, ((OP_add as i32)) as i32, ((OP_dup as i32)) as i32, ((OP_put_loc as i32)) as i32, (idx_1) as i32, ((OP_drop as i32)) as i32, (((1 as i32)).wrapping_neg()) as i32])) != 0 { 193 } else { 181 }; continue;
}
// C line 35297
195 => {
vm_block = 74; continue;
}
// C line 35296
196 => {
let _ = { let assigned = (cc).pos; pos_next = assigned; assigned };
vm_block = 195; continue;
}
// C line 35295
197 => {
let _ = dbuf_putc(core::ptr::addr_of_mut!(bc_out), ((idx_1) as u8));
vm_block = 196; continue;
}
// C line 35294
198 => {
let _ = dbuf_putc(core::ptr::addr_of_mut!(bc_out), ((if (((((((((cc).op) == ((OP_inc as i32))) as i32)) != 0) || ((((((cc).op) == ((OP_post_inc as i32))) as i32)) != 0)) as i32)) != 0 { (OP_inc_loc as i32) } else { (OP_dec_loc as i32) }) as u8));
vm_block = 197; continue;
}
// C line 35293
199 => {
let _ = add_pc2line_info(s, (((bc_out).size) as u32), ((line_num) as u32));
vm_block = 198; continue;
}
// C line 35292
200 => {
let _ = { let assigned = (cc).line_num; line_num = assigned; assigned };
vm_block = 199; continue;
}
// C line 35292
201 => {
vm_block = if (((((cc).line_num) >= ((0 as i32))) as i32)) != 0 { 200 } else { 199 }; continue;
}
// C line 35290
202 => {
vm_block = if (((((code_match(core::ptr::addr_of_mut!(cc), pos_next, &[((((OP_post_dec as i32)) | (((OP_post_inc as i32)).wrapping_shl(((8 as i32)) as u32)))) as i32, ((OP_put_loc as i32)) as i32, (idx_1) as i32, ((OP_drop as i32)) as i32, (((1 as i32)).wrapping_neg()) as i32])) != 0) || ((code_match(core::ptr::addr_of_mut!(cc), pos_next, &[((((OP_dec as i32)) | (((OP_inc as i32)).wrapping_shl(((8 as i32)) as u32)))) as i32, ((OP_dup as i32)) as i32, ((OP_put_loc as i32)) as i32, (idx_1) as i32, ((OP_drop as i32)) as i32, (((1 as i32)).wrapping_neg()) as i32])) != 0)) as i32)) != 0 { 201 } else { 194 }; continue;
}
// C line 35289
203 => {
vm_block = 77; continue;
}
// C line 35288
204 => {
vm_block = if ((((idx_1) >= ((256 as i32))) as i32)) != 0 { 203 } else { 202 }; continue;
}
// C line 35287
205 => {
let _ = { let assigned = ((get_u16(((bc_buf).offset(((pos) as isize))).offset((((1 as i32)) as isize)))) as i32); idx_1 = assigned; assigned };
vm_block = 204; continue;
}
// C line 35279
206 => {
vm_block = if ((1 as i32)) != 0 { 205 } else { 160 }; continue;
}
// C line 35276
207 => {
vm_block = 77; continue;
}
// C line 35273
208 => {
vm_block = 74; continue;
}
// C line 35272
209 => {
let _ = { let assigned = line2; line_num = assigned; assigned };
vm_block = 208; continue;
}
// C line 35272
210 => {
vm_block = if ((((line2) >= ((0 as i32))) as i32)) != 0 { 209 } else { 208 }; continue;
}
// C line 35271
211 => {
let _ = put_short_code(core::ptr::addr_of_mut!(bc_out), op1_1, (cc).idx);
vm_block = 210; continue;
}
// C line 35270
212 => {
let _ = add_pc2line_info(s, (((bc_out).size) as u32), ((line_num) as u32));
vm_block = 211; continue;
}
// C line 35267
213 => {
let _ = { let assigned = (cc).pos; pos_next = assigned; assigned };
vm_block = 212; continue;
}
// C line 35266
214 => {
let _ = { op1_1 = ((((op1_1) as i32)).wrapping_add((1 as i32))) as i32; op1_1 };
vm_block = 213; continue;
}
// C line 35265
215 => {
let _ = { let assigned = (cc).line_num; line2 = assigned; assigned };
vm_block = 214; continue;
}
// C line 35264
216 => {
vm_block = if (code_match(core::ptr::addr_of_mut!(cc), (cc).pos, &[((op1_1).wrapping_sub((1 as i32))) as i32, ((cc).idx) as i32, (((1 as i32)).wrapping_neg()) as i32])) != 0 { 215 } else { 212 }; continue;
}
// C line 35263
217 => {
let _ = { let assigned = (cc).pos; pos_next = assigned; assigned };
vm_block = 216; continue;
}
// C line 35262
218 => {
let _ = { op1_1 = ((((op1_1) as i32)).wrapping_sub((1 as i32))) as i32; op1_1 };
vm_block = 217; continue;
}
// C line 35261
219 => {
let _ = { let assigned = (cc).line_num; line_num = assigned; assigned };
vm_block = 218; continue;
}
// C line 35261
220 => {
vm_block = if (((((cc).line_num) >= ((0 as i32))) as i32)) != 0 { 219 } else { 218 }; continue;
}
// C line 35260
221 => {
vm_block = if (code_match(core::ptr::addr_of_mut!(cc), (cc).pos, &[((OP_drop as i32)) as i32, (((1 as i32)).wrapping_neg()) as i32])) != 0 { 220 } else { 212 }; continue;
}
// C line 35259
222 => {
let _ = { let assigned = (cc).pos; pos_next = assigned; assigned };
vm_block = 221; continue;
}
// C line 35258
223 => {
let _ = { let assigned = ((cc).op).wrapping_add((1 as i32)); op1_1 = assigned; assigned };
vm_block = 222; continue;
}
// C line 35257
224 => {
let _ = { let assigned = (cc).line_num; line_num = assigned; assigned };
vm_block = 223; continue;
}
// C line 35257
225 => {
vm_block = if (((((cc).line_num) >= ((0 as i32))) as i32)) != 0 { 224 } else { 223 }; continue;
}
// C line 35256
226 => {
vm_block = if (code_match(core::ptr::addr_of_mut!(cc), pos_next, &[((((((((OP_put_loc as i32)) | (((OP_put_loc_check as i32)).wrapping_shl(((8 as i32)) as u32)))) | (((OP_put_arg as i32)).wrapping_shl(((16 as i32)) as u32)))) | (((OP_put_var_ref as i32)).wrapping_shl(((24 as i32)) as u32)))) as i32, (((1 as i32)).wrapping_neg()) as i32, (((1 as i32)).wrapping_neg()) as i32])) != 0 { 225 } else { 207 }; continue;
}
// C line 35254
227 => {
line2 = ((1 as i32)).wrapping_neg();
vm_block = 226; continue;
}
// C line 35252
228 => {
vm_block = if ((1 as i32)) != 0 { 227 } else { 207 }; continue;
}
// C line 35249
229 => {
vm_block = 77; continue;
}
// C line 35246
230 => {
vm_block = 74; continue;
}
// C line 35245
231 => {
let _ = { let assigned = (cc).pos; pos_next = assigned; assigned };
vm_block = 230; continue;
}
// C line 35244
232 => {
let _ = dbuf_put_u32(core::ptr::addr_of_mut!(bc_out), (cc).atom);
vm_block = 231; continue;
}
// C line 35243
233 => {
let _ = dbuf_putc(core::ptr::addr_of_mut!(bc_out), (((OP_put_field as i32)) as u8));
vm_block = 232; continue;
}
// C line 35242
234 => {
let _ = add_pc2line_info(s, (((bc_out).size) as u32), ((line_num) as u32));
vm_block = 233; continue;
}
// C line 35241
235 => {
let _ = { let assigned = (cc).line_num; line_num = assigned; assigned };
vm_block = 234; continue;
}
// C line 35241
236 => {
vm_block = if (((((cc).line_num) >= ((0 as i32))) as i32)) != 0 { 235 } else { 234 }; continue;
}
// C line 35240
237 => {
vm_block = if (code_match(core::ptr::addr_of_mut!(cc), pos_next, &[((OP_put_field as i32)) as i32, ((OP_drop as i32)) as i32, (((1 as i32)).wrapping_neg()) as i32])) != 0 { 236 } else { 229 }; continue;
}
// C line 35236
238 => {
vm_block = if ((1 as i32)) != 0 { 237 } else { 229 }; continue;
}
// C line 35233
239 => {
vm_block = 77; continue;
}
// C line 35229
240 => {
vm_block = 448; continue;
}
// C line 35228
241 => {
let _ = { let assigned = (((((cc).op) ^ ((OP_if_false as i32)))) ^ ((OP_if_true as i32))); op = assigned; assigned };
vm_block = 240; continue;
}
// C line 35227
242 => {
let _ = { let assigned = (cc).label; label = assigned; assigned };
vm_block = 241; continue;
}
// C line 35226
243 => {
let _ = { let assigned = (cc).pos; pos_next = assigned; assigned };
vm_block = 242; continue;
}
// C line 35225
244 => {
let _ = dbuf_putc(core::ptr::addr_of_mut!(bc_out), (((OP_is_undefined as i32)) as u8));
vm_block = 243; continue;
}
// C line 35224
245 => {
let _ = add_pc2line_info(s, (((bc_out).size) as u32), ((line_num) as u32));
vm_block = 244; continue;
}
// C line 35223
246 => {
let _ = { let assigned = (cc).line_num; line_num = assigned; assigned };
vm_block = 245; continue;
}
// C line 35223
247 => {
vm_block = if (((((cc).line_num) >= ((0 as i32))) as i32)) != 0 { 246 } else { 245 }; continue;
}
// C line 35222
248 => {
vm_block = if (code_match(core::ptr::addr_of_mut!(cc), pos_next, &[((OP_strict_neq as i32)) as i32, ((((OP_if_false as i32)) | (((OP_if_true as i32)).wrapping_shl(((8 as i32)) as u32)))) as i32, (((1 as i32)).wrapping_neg()) as i32])) != 0 { 247 } else { 239 }; continue;
}
// C line 35219
249 => {
vm_block = 74; continue;
}
// C line 35218
250 => {
let _ = { let assigned = (cc).pos; pos_next = assigned; assigned };
vm_block = 249; continue;
}
// C line 35217
251 => {
let _ = dbuf_putc(core::ptr::addr_of_mut!(bc_out), (((OP_is_undefined as i32)) as u8));
vm_block = 250; continue;
}
// C line 35216
252 => {
let _ = add_pc2line_info(s, (((bc_out).size) as u32), ((line_num) as u32));
vm_block = 251; continue;
}
// C line 35215
253 => {
let _ = { let assigned = (cc).line_num; line_num = assigned; assigned };
vm_block = 252; continue;
}
// C line 35215
254 => {
vm_block = if (((((cc).line_num) >= ((0 as i32))) as i32)) != 0 { 253 } else { 252 }; continue;
}
// C line 35214
255 => {
vm_block = if (code_match(core::ptr::addr_of_mut!(cc), pos_next, &[((OP_strict_eq as i32)) as i32, (((1 as i32)).wrapping_neg()) as i32])) != 0 { 254 } else { 248 }; continue;
}
// C line 35210
256 => {
vm_block = 354; continue;
}
// C line 35209
257 => {
let _ = { let assigned = (0 as i32); val = assigned; assigned };
vm_block = 256; continue;
}
// C line 35208
258 => {
vm_block = if (code_match(core::ptr::addr_of_mut!(cc), pos_next, &[((((OP_if_false as i32)) | (((OP_if_true as i32)).wrapping_shl(((8 as i32)) as u32)))) as i32, (((1 as i32)).wrapping_neg()) as i32])) != 0 { 257 } else { 255 }; continue;
}
// C line 35205
259 => {
vm_block = 74; continue;
}
// C line 35204
260 => {
let _ = { let assigned = (cc).pos; pos_next = assigned; assigned };
vm_block = 259; continue;
}
// C line 35203
261 => {
let _ = dbuf_putc(core::ptr::addr_of_mut!(bc_out), (((OP_return_undef as i32)) as u8));
vm_block = 260; continue;
}
// C line 35202
262 => {
let _ = add_pc2line_info(s, (((bc_out).size) as u32), ((line_num) as u32));
vm_block = 261; continue;
}
// C line 35201
263 => {
let _ = { let assigned = (cc).line_num; line_num = assigned; assigned };
vm_block = 262; continue;
}
// C line 35201
264 => {
vm_block = if (((((cc).line_num) >= ((0 as i32))) as i32)) != 0 { 263 } else { 262 }; continue;
}
// C line 35200
265 => {
vm_block = if (code_match(core::ptr::addr_of_mut!(cc), pos_next, &[((OP_return as i32)) as i32, (((1 as i32)).wrapping_neg()) as i32])) != 0 { 264 } else { 258 }; continue;
}
// C line 35197
266 => {
vm_block = 74; continue;
}
// C line 35196
267 => {
let _ = { let assigned = (cc).pos; pos_next = assigned; assigned };
vm_block = 266; continue;
}
// C line 35195
268 => {
let _ = { let assigned = (cc).line_num; line_num = assigned; assigned };
vm_block = 267; continue;
}
// C line 35195
269 => {
vm_block = if (((((cc).line_num) >= ((0 as i32))) as i32)) != 0 { 268 } else { 267 }; continue;
}
// C line 35194
270 => {
vm_block = if (code_match(core::ptr::addr_of_mut!(cc), pos_next, &[((OP_drop as i32)) as i32, (((1 as i32)).wrapping_neg()) as i32])) != 0 { 269 } else { 265 }; continue;
}
// C line 35192
271 => {
vm_block = if ((1 as i32)) != 0 { 270 } else { 239 }; continue;
}
// C line 35189
272 => {
vm_block = 77; continue;
}
// C line 35186
273 => {
vm_block = 74; continue;
}
// C line 35183
274 => {
vm_block = if (((((((((code_match(core::ptr::addr_of_mut!(cc), pos_next, &[((((((OP_get_loc as i32)) | (((OP_get_arg as i32)).wrapping_shl(((8 as i32)) as u32)))) | (((OP_get_var_ref as i32)).wrapping_shl(((16 as i32)) as u32)))) as i32, (((1 as i32)).wrapping_neg()) as i32, ((OP_put_array_el as i32)) as i32, (((1 as i32)).wrapping_neg()) as i32])) != 0) || ((code_match(core::ptr::addr_of_mut!(cc), pos_next, &[((((((OP_push_i32 as i32)) | (((OP_push_const as i32)).wrapping_shl(((8 as i32)) as u32)))) | (((OP_push_atom_value as i32)).wrapping_shl(((16 as i32)) as u32)))) as i32, ((OP_put_array_el as i32)) as i32, (((1 as i32)).wrapping_neg()) as i32])) != 0)) as i32)) != 0) || ((code_match(core::ptr::addr_of_mut!(cc), pos_next, &[((((((((OP_undefined as i32)) | (((OP_null as i32)).wrapping_shl(((8 as i32)) as u32)))) | (((OP_push_true as i32)).wrapping_shl(((16 as i32)) as u32)))) | (((OP_push_false as i32)).wrapping_shl(((24 as i32)) as u32)))) as i32, ((OP_put_array_el as i32)) as i32, (((1 as i32)).wrapping_neg()) as i32])) != 0)) as i32)) != 0 { 273 } else { 272 }; continue;
}
// C line 35181
275 => {
vm_block = if ((1 as i32)) != 0 { 274 } else { 272 }; continue;
}
// C line 35178
276 => {
vm_block = 77; continue;
}
// C line 35174
277 => {
vm_block = 74; continue;
}
// C line 35173
278 => {
let _ = dbuf_putc(core::ptr::addr_of_mut!(bc_out), (((OP_push_empty_string as i32)) as u8));
vm_block = 277; continue;
}
// C line 35172
279 => {
let _ = add_pc2line_info(s, (((bc_out).size) as u32), ((line_num) as u32));
vm_block = 278; continue;
}
// C line 35171
280 => {
let _ = JS_FreeAtom(ctx, atom_2);
vm_block = 279; continue;
}
// C line 35170
281 => {
vm_block = if ((((atom_2) == ((((crate::quickjs_atom::JS_ATOM_empty_string as i32)) as JSAtom))) as i32)) != 0 { 280 } else { 276 }; continue;
}
// C line 35167
282 => {
vm_block = 74; continue;
}
// C line 35166
283 => {
let _ = { let assigned = (cc).pos; pos_next = assigned; assigned };
vm_block = 282; continue;
}
// C line 35165
284 => {
let _ = { let assigned = (cc).line_num; line_num = assigned; assigned };
vm_block = 283; continue;
}
// C line 35165
285 => {
vm_block = if (((((cc).line_num) >= ((0 as i32))) as i32)) != 0 { 284 } else { 283 }; continue;
}
// C line 35164
286 => {
let _ = JS_FreeAtom(ctx, atom_2);
vm_block = 285; continue;
}
// C line 35163
287 => {
vm_block = if (code_match(core::ptr::addr_of_mut!(cc), pos_next, &[((OP_drop as i32)) as i32, (((1 as i32)).wrapping_neg()) as i32])) != 0 { 286 } else { 281 }; continue;
}
// C line 35161
288 => {
atom_2 = get_u32(((bc_buf).offset(((pos) as isize))).offset((((1 as i32)) as isize)));
vm_block = 287; continue;
}
// C line 35160
289 => {
vm_block = if ((1 as i32)) != 0 { 288 } else { 276 }; continue;
}
// C line 35157
290 => {
vm_block = 77; continue;
}
// C line 35154
291 => {
vm_block = 74; continue;
}
// C line 35153
292 => {
let _ = dbuf_putc(core::ptr::addr_of_mut!(bc_out), (((OP_get_length as i32)) as u8));
vm_block = 291; continue;
}
// C line 35152
293 => {
let _ = add_pc2line_info(s, (((bc_out).size) as u32), ((line_num) as u32));
vm_block = 292; continue;
}
// C line 35151
294 => {
let _ = JS_FreeAtom(ctx, atom_1);
vm_block = 293; continue;
}
// C line 35150
295 => {
vm_block = if ((((atom_1) == ((((crate::quickjs_atom::JS_ATOM_length as i32)) as JSAtom))) as i32)) != 0 { 294 } else { 290 }; continue;
}
// C line 35149
296 => {
atom_1 = get_u32(((bc_buf).offset(((pos) as isize))).offset((((1 as i32)) as isize)));
vm_block = 295; continue;
}
// C line 35148
297 => {
vm_block = if ((1 as i32)) != 0 { 296 } else { 290 }; continue;
}
// C line 35145
298 => {
vm_block = 77; continue;
}
// C line 35142
299 => {
vm_block = 74; continue;
}
// C line 35141
300 => {
let _ = dbuf_putc(core::ptr::addr_of_mut!(bc_out), ((idx) as u8));
vm_block = 299; continue;
}
// C line 35140
301 => {
let _ = dbuf_putc(core::ptr::addr_of_mut!(bc_out), (((((OP_push_const8 as i32)).wrapping_add(op)).wrapping_sub((OP_push_const as i32))) as u8));
vm_block = 300; continue;
}
// C line 35139
302 => {
let _ = add_pc2line_info(s, (((bc_out).size) as u32), ((line_num) as u32));
vm_block = 301; continue;
}
// C line 35138
303 => {
vm_block = if ((((idx) < ((256 as i32))) as i32)) != 0 { 302 } else { 298 }; continue;
}
// C line 35137
304 => {
idx = ((get_u32(((bc_buf).offset(((pos) as isize))).offset((((1 as i32)) as isize)))) as i32);
vm_block = 303; continue;
}
// C line 35136
305 => {
vm_block = if ((1 as i32)) != 0 { 304 } else { 298 }; continue;
}
// C line 35131
306 => {
vm_block = 77; continue;
}
// C line 35128
307 => {
vm_block = 74; continue;
}
// C line 35127
308 => {
let _ = { let assigned = (cc).pos; pos_next = assigned; assigned };
vm_block = 307; continue;
}
// C line 35121
309 => {
let _ = { let assigned = (cc).line_num; line_num = assigned; assigned };
vm_block = 308; continue;
}
// C line 35121
310 => {
vm_block = if (((((cc).line_num) >= ((0 as i32))) as i32)) != 0 { 309 } else { 308 }; continue;
}
// C line 35125
311 => {
let _ = dbuf_put_u32(core::ptr::addr_of_mut!(bc_out), (((val).wrapping_neg()) as u32));
vm_block = 308; continue;
}
// C line 35124
312 => {
let _ = dbuf_putc(core::ptr::addr_of_mut!(bc_out), (((OP_push_bigint_i32 as i32)) as u8));
vm_block = 311; continue;
}
// C line 35123
313 => {
let _ = add_pc2line_info(s, (((bc_out).size) as u32), ((line_num) as u32));
vm_block = 312; continue;
}
// C line 35120
314 => {
vm_block = if (code_match(core::ptr::addr_of_mut!(cc), (cc).pos, &[((OP_drop as i32)) as i32, (((1 as i32)).wrapping_neg()) as i32])) != 0 { 310 } else { 313 }; continue;
}
// C line 35119
315 => {
let _ = { let assigned = (cc).line_num; line_num = assigned; assigned };
vm_block = 314; continue;
}
// C line 35119
316 => {
vm_block = if (((((cc).line_num) >= ((0 as i32))) as i32)) != 0 { 315 } else { 314 }; continue;
}
// C line 35117
317 => {
vm_block = if ((((((((val) != ((((2147483647 as i32)).wrapping_neg()).wrapping_sub((1 as i32)))) as i32)) != 0) && ((code_match(core::ptr::addr_of_mut!(cc), pos_next, &[((OP_neg as i32)) as i32, (((1 as i32)).wrapping_neg()) as i32])) != 0)) as i32)) != 0 { 316 } else { 306 }; continue;
}
// C line 35116
318 => {
let _ = { let assigned = crate::cutils_header::get_i32(((bc_buf).offset(((pos) as isize))).offset((((1 as i32)) as isize))); val = assigned; assigned };
vm_block = 317; continue;
}
// C line 35114
319 => {
vm_block = if ((1 as i32)) != 0 { 318 } else { 306 }; continue;
}
// C line 35111
320 => {
vm_block = 77; continue;
}
// C line 35109
321 => {
vm_block = 74; continue;
}
// C line 35108
322 => {
let _ = push_short_int(core::ptr::addr_of_mut!(bc_out), val);
vm_block = 321; continue;
}
// C line 35107
323 => {
let _ = add_pc2line_info(s, (((bc_out).size) as u32), ((line_num) as u32));
vm_block = 322; continue;
}
// C line 35105
324 => {
vm_block = 354; continue;
}
// C line 35104
325 => {
let _ = { let assigned = (((val) != ((0 as i32))) as i32); val = assigned; assigned };
vm_block = 324; continue;
}
// C line 35103
326 => {
vm_block = if (code_match(core::ptr::addr_of_mut!(cc), pos_next, &[((((OP_if_false as i32)) | (((OP_if_true as i32)).wrapping_shl(((8 as i32)) as u32)))) as i32, (((1 as i32)).wrapping_neg()) as i32])) != 0 { 325 } else { 323 }; continue;
}
// C line 35100
327 => {
vm_block = 74; continue;
}
// C line 35099
328 => {
let _ = { let assigned = (cc).pos; pos_next = assigned; assigned };
vm_block = 327; continue;
}
// C line 35098
329 => {
let _ = { let assigned = (cc).line_num; line_num = assigned; assigned };
vm_block = 328; continue;
}
// C line 35098
330 => {
vm_block = if (((((cc).line_num) >= ((0 as i32))) as i32)) != 0 { 329 } else { 328 }; continue;
}
// C line 35097
331 => {
vm_block = if (code_match(core::ptr::addr_of_mut!(cc), pos_next, &[((OP_drop as i32)) as i32, (((1 as i32)).wrapping_neg()) as i32])) != 0 { 330 } else { 326 }; continue;
}
// C line 35094
332 => {
vm_block = 74; continue;
}
// C line 35093
333 => {
let _ = { let assigned = (cc).pos; pos_next = assigned; assigned };
vm_block = 332; continue;
}
// C line 35088
334 => {
let _ = { let assigned = (cc).line_num; line_num = assigned; assigned };
vm_block = 333; continue;
}
// C line 35088
335 => {
vm_block = if (((((cc).line_num) >= ((0 as i32))) as i32)) != 0 { 334 } else { 333 }; continue;
}
// C line 35091
336 => {
let _ = push_short_int(core::ptr::addr_of_mut!(bc_out), (val).wrapping_neg());
vm_block = 333; continue;
}
// C line 35090
337 => {
let _ = add_pc2line_info(s, (((bc_out).size) as u32), ((line_num) as u32));
vm_block = 336; continue;
}
// C line 35087
338 => {
vm_block = if (code_match(core::ptr::addr_of_mut!(cc), (cc).pos, &[((OP_drop as i32)) as i32, (((1 as i32)).wrapping_neg()) as i32])) != 0 { 335 } else { 337 }; continue;
}
// C line 35086
339 => {
let _ = { let assigned = (cc).line_num; line_num = assigned; assigned };
vm_block = 338; continue;
}
// C line 35086
340 => {
vm_block = if (((((cc).line_num) >= ((0 as i32))) as i32)) != 0 { 339 } else { 338 }; continue;
}
// C line 35084
341 => {
vm_block = if ((((((((((((val) != ((((2147483647 as i32)).wrapping_neg()).wrapping_sub((1 as i32)))) as i32)) != 0) && (((((val) != ((0 as i32))) as i32)) != 0)) as i32)) != 0) && ((code_match(core::ptr::addr_of_mut!(cc), pos_next, &[((OP_neg as i32)) as i32, (((1 as i32)).wrapping_neg()) as i32])) != 0)) as i32)) != 0 { 340 } else { 331 }; continue;
}
// C line 35083
342 => {
let _ = { let assigned = crate::cutils_header::get_i32(((bc_buf).offset(((pos) as isize))).offset((((1 as i32)) as isize))); val = assigned; assigned };
vm_block = 341; continue;
}
// C line 35081
343 => {
vm_block = if ((1 as i32)) != 0 { 342 } else { 320 }; continue;
}
// C line 35078
344 => {
vm_block = 77; continue;
}
// C line 35067
345 => {
vm_block = 487; continue;
}
// C line 35066
346 => {
let _ = { let assigned = (cc).label; label = assigned; assigned };
vm_block = 345; continue;
}
// C line 35065
347 => {
let _ = { let assigned = (OP_goto as i32); op = assigned; assigned };
vm_block = 346; continue;
}
// C line 35064
348 => {
let _ = { let assigned = (cc).pos; pos_next = assigned; assigned };
vm_block = 347; continue;
}
// C line 35074
349 => {
vm_block = 74; continue;
}
// C line 35073
350 => {
let _ = update_label(s, (cc).label, ((1 as i32)).wrapping_neg());
vm_block = 349; continue;
}
// C line 35072
351 => {
let _ = { let assigned = (cc).pos; pos_next = assigned; assigned };
vm_block = 350; continue;
}
// C line 35060
352 => {
vm_block = if ((((val) == (((cc).op).wrapping_sub((OP_if_false as i32)))) as i32)) != 0 { 348 } else { 351 }; continue;
}
// C line 35059
353 => {
let _ = { let assigned = (cc).line_num; line_num = assigned; assigned };
vm_block = 352; continue;
}
// C line 35059 labels: has_constant_test
354 => {
vm_block = if (((((cc).line_num) >= ((0 as i32))) as i32)) != 0 { 353 } else { 352 }; continue;
}
// C line 35057
355 => {
vm_block = if (code_match(core::ptr::addr_of_mut!(cc), pos_next, &[((((OP_if_false as i32)) | (((OP_if_true as i32)).wrapping_shl(((8 as i32)) as u32)))) as i32, (((1 as i32)).wrapping_neg()) as i32])) != 0 { 354 } else { 344 }; continue;
}
// C line 35056
356 => {
let _ = { let assigned = (((op) == ((OP_push_true as i32))) as i32); val = assigned; assigned };
vm_block = 355; continue;
}
// C line 35055
357 => {
vm_block = if ((1 as i32)) != 0 { 356 } else { 344 }; continue;
}
// C line 35048
358 => {
vm_block = 448; continue;
}
// C line 35047
359 => {
let _ = { let assigned = (((((cc).op) ^ ((OP_if_false as i32)))) ^ ((OP_if_true as i32))); op = assigned; assigned };
vm_block = 358; continue;
}
// C line 35046
360 => {
let _ = { let assigned = (cc).label; label = assigned; assigned };
vm_block = 359; continue;
}
// C line 35045
361 => {
let _ = { let assigned = (cc).pos; pos_next = assigned; assigned };
vm_block = 360; continue;
}
// C line 35044
362 => {
let _ = dbuf_putc(core::ptr::addr_of_mut!(bc_out), (((OP_is_null as i32)) as u8));
vm_block = 361; continue;
}
// C line 35043
363 => {
let _ = add_pc2line_info(s, (((bc_out).size) as u32), ((line_num) as u32));
vm_block = 362; continue;
}
// C line 35042
364 => {
let _ = { let assigned = (cc).line_num; line_num = assigned; assigned };
vm_block = 363; continue;
}
// C line 35042
365 => {
vm_block = if (((((cc).line_num) >= ((0 as i32))) as i32)) != 0 { 364 } else { 363 }; continue;
}
// C line 35041
366 => {
vm_block = if (code_match(core::ptr::addr_of_mut!(cc), pos_next, &[((OP_strict_neq as i32)) as i32, ((((OP_if_false as i32)) | (((OP_if_true as i32)).wrapping_shl(((8 as i32)) as u32)))) as i32, (((1 as i32)).wrapping_neg()) as i32])) != 0 { 365 } else { 357 }; continue;
}
// C line 35038
367 => {
vm_block = 74; continue;
}
// C line 35037
368 => {
let _ = { let assigned = (cc).pos; pos_next = assigned; assigned };
vm_block = 367; continue;
}
// C line 35036
369 => {
let _ = dbuf_putc(core::ptr::addr_of_mut!(bc_out), (((OP_is_null as i32)) as u8));
vm_block = 368; continue;
}
// C line 35035
370 => {
let _ = add_pc2line_info(s, (((bc_out).size) as u32), ((line_num) as u32));
vm_block = 369; continue;
}
// C line 35034
371 => {
let _ = { let assigned = (cc).line_num; line_num = assigned; assigned };
vm_block = 370; continue;
}
// C line 35034
372 => {
vm_block = if (((((cc).line_num) >= ((0 as i32))) as i32)) != 0 { 371 } else { 370 }; continue;
}
// C line 35033
373 => {
vm_block = if (code_match(core::ptr::addr_of_mut!(cc), pos_next, &[((OP_strict_eq as i32)) as i32, (((1 as i32)).wrapping_neg()) as i32])) != 0 { 372 } else { 366 }; continue;
}
// C line 35031
374 => {
vm_block = if ((1 as i32)) != 0 { 373 } else { 357 }; continue;
}
// C line 35027
375 => {
vm_block = 77; continue;
}
// C line 35024
376 => {
vm_block = 74; continue;
}
// C line 35023
377 => {
let _ = { let assigned = (cc).line_num; line_num = assigned; assigned };
vm_block = 376; continue;
}
// C line 35023
378 => {
vm_block = if (((((cc).line_num) >= ((0 as i32))) as i32)) != 0 { 377 } else { 376 }; continue;
}
// C line 35022
379 => {
vm_block = if (code_match(core::ptr::addr_of_mut!(cc), pos_next, &[((OP_return_undef as i32)) as i32, (((1 as i32)).wrapping_neg()) as i32])) != 0 { 378 } else { 375 }; continue;
}
// C line 35020
380 => {
vm_block = if ((1 as i32)) != 0 { 379 } else { 375 }; continue;
}
// C line 35017
381 => {
vm_block = 74; continue;
}
// C line 35015
382 => {
let _ = dbuf_putc(core::ptr::addr_of_mut!(bc_out), ((is_with) as u8));
vm_block = 381; continue;
}
// C line 35013
383 => {
vm_block = 2; continue;
}
// C line 35012
384 => {
vm_block = if ((!(!(add_reloc(ctx, ls, ((((bc_out).size).wrapping_sub((((4 as i32)) as usize))) as u32), (4 as i32))).is_null()) as i32)) != 0 { 383 } else { 382 }; continue;
}
// C line 35010
385 => {
vm_block = if (((((*(ls)).addr) == (((1 as i32)).wrapping_neg())) as i32)) != 0 { 384 } else { 382 }; continue;
}
// C line 35009
386 => {
let _ = dbuf_put_u32(core::ptr::addr_of_mut!(bc_out), ((((((*(ls)).addr) as usize)).wrapping_sub((bc_out).size)) as u32));
vm_block = 385; continue;
}
// C line 35008
387 => {
let _ = dbuf_put_u32(core::ptr::addr_of_mut!(bc_out), atom);
vm_block = 386; continue;
}
// C line 35007
388 => {
let _ = dbuf_putc(core::ptr::addr_of_mut!(bc_out), ((op) as u8));
vm_block = 387; continue;
}
// C line 35005
389 => {
let _ = { let assigned = label; (*(jp)).label = assigned; assigned };
vm_block = 388; continue;
}
// C line 35004
390 => {
let _ = { let assigned = ((((bc_out).size).wrapping_add((((5 as i32)) as usize))) as i32); (*(jp)).pos = assigned; assigned };
vm_block = 389; continue;
}
// C line 35003
391 => {
let _ = { let assigned = (4 as i32); (*(jp)).size = assigned; assigned };
vm_block = 390; continue;
}
// C line 35002
392 => {
let _ = { let assigned = op; (*(jp)).op = assigned; assigned };
vm_block = 391; continue;
}
// C line 35001
393 => {
let _ = { let assigned = core::ptr::addr_of_mut!(*((*(s)).jump_slots).offset(({ let old = (*(s)).jump_count; (*(s)).jump_count = ((*(s)).jump_count).wrapping_add(1); old }) as isize)); jp = assigned; assigned };
vm_block = 392; continue;
}
// C line 34999
394 => {
let _ = add_pc2line_info(s, (((bc_out).size) as u32), ((line_num) as u32));
vm_block = 393; continue;
}
// C line 34998
395 => {
let _ = { let assigned = core::ptr::addr_of_mut!(*(label_slots).offset((label) as isize)); ls = assigned; assigned };
vm_block = 394; continue;
}
// C line 34997
396 => {
let _ = if ((((!(((((((((label) >= ((0 as i32))) as i32)) != 0) && (((((label) < ((*(s)).label_count)) as i32)) != 0)) as i32)) != 0) as i32)) as i64)) != 0 { std::process::abort() } else { { let _ = (0 as i32); } };
vm_block = 395; continue;
}
// C line 34995
397 => {
let _ = { let assigned = find_jump_target(s, label, core::ptr::addr_of_mut!(op1), core::ptr::null_mut::<i32>()); label = assigned; assigned };
vm_block = 396; continue;
}
// C line 34994
398 => {
vm_block = if ((1 as i32)) != 0 { 397 } else { 396 }; continue;
}
// C line 34993
399 => {
let _ = { let assigned = ((*(bc_buf).offset(((pos).wrapping_add((9 as i32))) as isize)) as i32); is_with = assigned; assigned };
vm_block = 398; continue;
}
// C line 34992
400 => {
let _ = { let assigned = ((get_u32(((bc_buf).offset(((pos) as isize))).offset((((5 as i32)) as isize)))) as i32); label = assigned; assigned };
vm_block = 399; continue;
}
// C line 34991
401 => {
let _ = { let assigned = get_u32(((bc_buf).offset(((pos) as isize))).offset((((1 as i32)) as isize))); atom = assigned; assigned };
vm_block = 400; continue;
}
// C line 34981
402 => {
vm_block = 74; continue;
}
// C line 34979
403 => {
vm_block = 2; continue;
}
// C line 34978
404 => {
vm_block = if ((!(!(add_reloc(ctx, ls, ((((bc_out).size).wrapping_sub((((4 as i32)) as usize))) as u32), (4 as i32))).is_null()) as i32)) != 0 { 403 } else { 402 }; continue;
}
// C line 34976
405 => {
vm_block = if (((((*(ls)).addr) == (((1 as i32)).wrapping_neg())) as i32)) != 0 { 404 } else { 402 }; continue;
}
// C line 34975
406 => {
let _ = dbuf_put_u32(core::ptr::addr_of_mut!(bc_out), ((((((*(ls)).addr) as usize)).wrapping_sub((bc_out).size)) as u32));
vm_block = 405; continue;
}
// C line 34974
407 => {
let _ = dbuf_putc(core::ptr::addr_of_mut!(bc_out), ((op) as u8));
vm_block = 406; continue;
}
// C line 34954
408 => {
vm_block = 74; continue;
}
// C line 34953
409 => {
vm_block = 2; continue;
}
// C line 34952
410 => {
vm_block = if ((!(!(add_reloc(ctx, ls, ((((bc_out).size).wrapping_sub((((2 as i32)) as usize))) as u32), (2 as i32))).is_null()) as i32)) != 0 { 409 } else { 408 }; continue;
}
// C line 34951
411 => {
let _ = dbuf_put_u16(core::ptr::addr_of_mut!(bc_out), (((0 as i32)) as u16));
vm_block = 410; continue;
}
// C line 34950
412 => {
let _ = dbuf_putc(core::ptr::addr_of_mut!(bc_out), (((OP_goto16 as i32)) as u8));
vm_block = 411; continue;
}
// C line 34949
413 => {
let _ = { let assigned = (OP_goto16 as i32); (*(jp)).op = assigned; assigned };
vm_block = 412; continue;
}
// C line 34948
414 => {
let _ = { let assigned = (2 as i32); (*(jp)).size = assigned; assigned };
vm_block = 413; continue;
}
// C line 34947
415 => {
vm_block = if ((((((((diff_1) < ((32768 as i32))) as i32)) != 0) && (((((op) == ((OP_goto as i32))) as i32)) != 0)) as i32)) != 0 { 414 } else { 407 }; continue;
}
// C line 34945
416 => {
vm_block = 74; continue;
}
// C line 34944
417 => {
vm_block = 2; continue;
}
// C line 34943
418 => {
vm_block = if ((!(!(add_reloc(ctx, ls, ((((bc_out).size).wrapping_sub((((1 as i32)) as usize))) as u32), (1 as i32))).is_null()) as i32)) != 0 { 417 } else { 416 }; continue;
}
// C line 34942
419 => {
let _ = dbuf_putc(core::ptr::addr_of_mut!(bc_out), (((0 as i32)) as u8));
vm_block = 418; continue;
}
// C line 34941
420 => {
let _ = dbuf_putc(core::ptr::addr_of_mut!(bc_out), ((((OP_if_false8 as i32)).wrapping_add((op).wrapping_sub((OP_if_false as i32)))) as u8));
vm_block = 419; continue;
}
// C line 34940
421 => {
let _ = { let assigned = ((OP_if_false8 as i32)).wrapping_add((op).wrapping_sub((OP_if_false as i32))); (*(jp)).op = assigned; assigned };
vm_block = 420; continue;
}
// C line 34939
422 => {
let _ = { let assigned = (1 as i32); (*(jp)).size = assigned; assigned };
vm_block = 421; continue;
}
// C line 34938
423 => {
vm_block = if ((((((((diff_1) < ((128 as i32))) as i32)) != 0) && (((((((((((((op) == ((OP_if_false as i32))) as i32)) != 0) || (((((op) == ((OP_if_true as i32))) as i32)) != 0)) as i32)) != 0) || (((((op) == ((OP_goto as i32))) as i32)) != 0)) as i32)) != 0)) as i32)) != 0 { 422 } else { 415 }; continue;
}
// C line 34937
424 => {
diff_1 = (((*(ls)).pos2).wrapping_sub(pos)).wrapping_sub((1 as i32));
vm_block = 423; continue;
}
// C line 34970
425 => {
vm_block = 74; continue;
}
// C line 34969
426 => {
let _ = dbuf_put_u16(core::ptr::addr_of_mut!(bc_out), ((diff_2) as u16));
vm_block = 425; continue;
}
// C line 34968
427 => {
let _ = dbuf_putc(core::ptr::addr_of_mut!(bc_out), (((OP_goto16 as i32)) as u8));
vm_block = 426; continue;
}
// C line 34967
428 => {
let _ = { let assigned = (OP_goto16 as i32); (*(jp)).op = assigned; assigned };
vm_block = 427; continue;
}
// C line 34966
429 => {
let _ = { let assigned = (2 as i32); (*(jp)).size = assigned; assigned };
vm_block = 428; continue;
}
// C line 34965
430 => {
vm_block = if ((((((((diff_2) == (((((diff_2) as i16)) as i32))) as i32)) != 0) && (((((op) == ((OP_goto as i32))) as i32)) != 0)) as i32)) != 0 { 429 } else { 407 }; continue;
}
// C line 34963
431 => {
vm_block = 74; continue;
}
// C line 34962
432 => {
let _ = dbuf_putc(core::ptr::addr_of_mut!(bc_out), ((diff_2) as u8));
vm_block = 431; continue;
}
// C line 34961
433 => {
let _ = dbuf_putc(core::ptr::addr_of_mut!(bc_out), ((((OP_if_false8 as i32)).wrapping_add((op).wrapping_sub((OP_if_false as i32)))) as u8));
vm_block = 432; continue;
}
// C line 34960
434 => {
let _ = { let assigned = ((OP_if_false8 as i32)).wrapping_add((op).wrapping_sub((OP_if_false as i32))); (*(jp)).op = assigned; assigned };
vm_block = 433; continue;
}
// C line 34959
435 => {
let _ = { let assigned = (1 as i32); (*(jp)).size = assigned; assigned };
vm_block = 434; continue;
}
// C line 34958
436 => {
vm_block = if ((((((((diff_2) == (((((diff_2) as i8)) as i32))) as i32)) != 0) && (((((((((((((op) == ((OP_if_false as i32))) as i32)) != 0) || (((((op) == ((OP_if_true as i32))) as i32)) != 0)) as i32)) != 0) || (((((op) == ((OP_goto as i32))) as i32)) != 0)) as i32)) != 0)) as i32)) != 0 { 435 } else { 430 }; continue;
}
// C line 34957
437 => {
diff_2 = (((((((*(ls)).addr) as usize)).wrapping_sub((bc_out).size)).wrapping_sub((((1 as i32)) as usize))) as i32);
vm_block = 436; continue;
}
// C line 34936
438 => {
vm_block = if (((((*(ls)).addr) == (((1 as i32)).wrapping_neg())) as i32)) != 0 { 424 } else { 437 }; continue;
}
// C line 34934
439 => {
let _ = { let assigned = label; (*(jp)).label = assigned; assigned };
vm_block = 438; continue;
}
// C line 34933
440 => {
let _ = { let assigned = ((((bc_out).size).wrapping_add((((1 as i32)) as usize))) as i32); (*(jp)).pos = assigned; assigned };
vm_block = 439; continue;
}
// C line 34932
441 => {
let _ = { let assigned = (4 as i32); (*(jp)).size = assigned; assigned };
vm_block = 440; continue;
}
// C line 34931
442 => {
let _ = { let assigned = op; (*(jp)).op = assigned; assigned };
vm_block = 441; continue;
}
// C line 34930
443 => {
let _ = { let assigned = core::ptr::addr_of_mut!(*((*(s)).jump_slots).offset(({ let old = (*(s)).jump_count; (*(s)).jump_count = ((*(s)).jump_count).wrapping_add(1); old }) as isize)); jp = assigned; assigned };
vm_block = 442; continue;
}
// C line 34928
444 => {
let _ = { let assigned = core::ptr::addr_of_mut!(*(label_slots).offset((label) as isize)); ls = assigned; assigned };
vm_block = 443; continue;
}
// C line 34927
445 => {
let _ = if ((((!(((((((((label) >= ((0 as i32))) as i32)) != 0) && (((((label) < ((*(s)).label_count)) as i32)) != 0)) as i32)) != 0) as i32)) as i64)) != 0 { std::process::abort() } else { { let _ = (0 as i32); } };
vm_block = 444; continue;
}
// C line 34925
446 => {
let _ = { let assigned = skip_dead_code(s, bc_buf, bc_len, pos_next, core::ptr::addr_of_mut!(line_num)); pos_next = assigned; assigned };
vm_block = 445; continue;
}
// C line 34924
447 => {
vm_block = if ((((op) == ((OP_goto as i32))) as i32)) != 0 { 446 } else { 445 }; continue;
}
// C line 34923 labels: has_label
448 => {
let _ = add_pc2line_info(s, (((bc_out).size) as u32), ((line_num) as u32));
vm_block = 447; continue;
}
// C line 34918
449 => {
let _ = { op = (((((op) as i32)) ^ ((((OP_if_true as i32)) ^ ((OP_if_false as i32)))))) as i32; op };
vm_block = 448; continue;
}
// C line 34917
450 => {
let _ = { let assigned = (cc).label; label = assigned; assigned };
vm_block = 449; continue;
}
// C line 34916
451 => {
let _ = update_label(s, label, ((1 as i32)).wrapping_neg());
vm_block = 450; continue;
}
// C line 34915
452 => {
let _ = { let assigned = pos1; pos_next = assigned; assigned };
vm_block = 451; continue;
}
// C line 34914
453 => {
let _ = { let assigned = line1_1; line_num = assigned; assigned };
vm_block = 452; continue;
}
// C line 34914
454 => {
vm_block = if ((((line1_1) != (((1 as i32)).wrapping_neg())) as i32)) != 0 { 453 } else { 452 }; continue;
}
// C line 34913
455 => {
vm_block = if (code_has_label(core::ptr::addr_of_mut!(cc), pos1, label)) != 0 { 454 } else { 448 }; continue;
}
// C line 34912
456 => {
line1_1 = (cc).line_num;
vm_block = 455; continue;
}
// C line 34911
457 => {
pos1 = (cc).pos;
vm_block = 456; continue;
}
// C line 34910
458 => {
vm_block = if (code_match(core::ptr::addr_of_mut!(cc), pos_next, &[((OP_goto as i32)) as i32, (((1 as i32)).wrapping_neg()) as i32])) != 0 { 457 } else { 448 }; continue;
}
// C line 34907
459 => {
vm_block = 74; continue;
}
// C line 34906
460 => {
let _ = dbuf_putc(core::ptr::addr_of_mut!(bc_out), (((OP_drop as i32)) as u8));
vm_block = 459; continue;
}
// C line 34905
461 => {
let _ = update_label(s, label, ((1 as i32)).wrapping_neg());
vm_block = 460; continue;
}
// C line 34904
462 => {
vm_block = if (code_has_label(core::ptr::addr_of_mut!(cc), pos_next, label)) != 0 { 461 } else { 458 }; continue;
}
// C line 34902
463 => {
let _ = { let assigned = find_jump_target(s, label, core::ptr::addr_of_mut!(op1), core::ptr::null_mut::<i32>()); label = assigned; assigned };
vm_block = 462; continue;
}
// C line 34901
464 => {
vm_block = if ((1 as i32)) != 0 { 463 } else { 448 }; continue;
}
// C line 34900
465 => {
let _ = { let assigned = ((get_u32(((bc_buf).offset(((pos) as isize))).offset((((1 as i32)) as isize)))) as i32); label = assigned; assigned };
vm_block = 464; continue;
}
// C line 34896
466 => {
vm_block = 448; continue;
}
// C line 34895
467 => {
let _ = { let assigned = ((get_u32(((bc_buf).offset(((pos) as isize))).offset((((1 as i32)) as isize)))) as i32); label = assigned; assigned };
vm_block = 466; continue;
}
// C line 34892
468 => {
vm_block = 448; continue;
}
// C line 34889
469 => {
vm_block = 74; continue;
}
// C line 34887
470 => {
let _ = update_label(s, label, ((1 as i32)).wrapping_neg());
vm_block = 469; continue;
}
// C line 34886
471 => {
vm_block = if ((((op1) == ((OP_ret as i32))) as i32)) != 0 { 470 } else { 468 }; continue;
}
// C line 34885
472 => {
let _ = { let assigned = find_jump_target(s, label, core::ptr::addr_of_mut!(op1), core::ptr::null_mut::<i32>()); label = assigned; assigned };
vm_block = 471; continue;
}
// C line 34884
473 => {
vm_block = if ((((((0 as i32)) != 0) && (((1 as i32)) != 0)) as i32)) != 0 { 472 } else { 468 }; continue;
}
// C line 34883
474 => {
let _ = { let assigned = ((get_u32(((bc_buf).offset(((pos) as isize))).offset((((1 as i32)) as isize)))) as i32); label = assigned; assigned };
vm_block = 473; continue;
}
// C line 34880
475 => {
vm_block = 448; continue;
}
// C line 34871
476 => {
vm_block = 74; continue;
}
// C line 34870
477 => {
let _ = { let assigned = skip_dead_code(s, bc_buf, bc_len, pos_next, core::ptr::addr_of_mut!(line_num)); pos_next = assigned; assigned };
vm_block = 476; continue;
}
// C line 34869
478 => {
let _ = dbuf_putc(core::ptr::addr_of_mut!(bc_out), ((op1) as u8));
vm_block = 477; continue;
}
// C line 34868
479 => {
let _ = add_pc2line_info(s, (((bc_out).size) as u32), ((line_num) as u32));
vm_block = 478; continue;
}
// C line 34867
480 => {
let _ = update_label(s, label, ((1 as i32)).wrapping_neg());
vm_block = 479; continue;
}
// C line 34863
481 => {
vm_block = if ((((((((((((op1) == ((OP_return as i32))) as i32)) != 0) || (((((op1) == ((OP_return_undef as i32))) as i32)) != 0)) as i32)) != 0) || (((((op1) == ((OP_throw as i32))) as i32)) != 0)) as i32)) != 0 { 480 } else { 475 }; continue;
}
// C line 34861
482 => {
vm_block = 74; continue;
}
// C line 34860
483 => {
let _ = update_label(s, label, ((1 as i32)).wrapping_neg());
vm_block = 482; continue;
}
// C line 34858
484 => {
vm_block = if (code_has_label(core::ptr::addr_of_mut!(cc), pos_next, label)) != 0 { 483 } else { 481 }; continue;
}
// C line 34857
485 => {
let _ = { let assigned = find_jump_target(s, label, core::ptr::addr_of_mut!(op1), core::ptr::addr_of_mut!(line1)); label = assigned; assigned };
vm_block = 484; continue;
}
// C line 34855
486 => {
line1 = ((1 as i32)).wrapping_neg();
vm_block = 485; continue;
}
// C line 34854 labels: has_goto
487 => {
vm_block = if ((1 as i32)) != 0 { 486 } else { 475 }; continue;
}
// C line 34852
488 => {
let _ = { let assigned = ((get_u32(((bc_buf).offset(((pos) as isize))).offset((((1 as i32)) as isize)))) as i32); label = assigned; assigned };
vm_block = 487; continue;
}
// C line 34849
489 => {
vm_block = 77; continue;
}
// C line 34848
490 => {
let _ = { let assigned = skip_dead_code(s, bc_buf, bc_len, pos_next, core::ptr::addr_of_mut!(line_num)); pos_next = assigned; assigned };
vm_block = 489; continue;
}
// C line 34841
491 => {
vm_block = 77; continue;
}
// C line 34839
492 => {
vm_block = 74; continue;
}
// C line 34838
493 => {
let _ = put_short_code(core::ptr::addr_of_mut!(bc_out), op, argc);
vm_block = 492; continue;
}
// C line 34837
494 => {
let _ = add_pc2line_info(s, (((bc_out).size) as u32), ((line_num) as u32));
vm_block = 493; continue;
}
// C line 34835
495 => {
vm_block = 74; continue;
}
// C line 34834
496 => {
let _ = { let assigned = skip_dead_code(s, bc_buf, bc_len, (cc).pos, core::ptr::addr_of_mut!(line_num)); pos_next = assigned; assigned };
vm_block = 495; continue;
}
// C line 34833
497 => {
let _ = put_short_code(core::ptr::addr_of_mut!(bc_out), (op).wrapping_add((1 as i32)), argc);
vm_block = 496; continue;
}
// C line 34832
498 => {
let _ = add_pc2line_info(s, (((bc_out).size) as u32), ((line_num) as u32));
vm_block = 497; continue;
}
// C line 34831
499 => {
let _ = { let assigned = (cc).line_num; line_num = assigned; assigned };
vm_block = 498; continue;
}
// C line 34831
500 => {
vm_block = if (((((cc).line_num) >= ((0 as i32))) as i32)) != 0 { 499 } else { 498 }; continue;
}
// C line 34830
501 => {
vm_block = if (code_match(core::ptr::addr_of_mut!(cc), pos_next, &[((OP_return as i32)) as i32, (((1 as i32)).wrapping_neg()) as i32])) != 0 { 500 } else { 494 }; continue;
}
// C line 34829
502 => {
let _ = { let assigned = ((get_u16(((bc_buf).offset(((pos) as isize))).offset((((1 as i32)) as isize)))) as i32); argc = assigned; assigned };
vm_block = 501; continue;
}
// C line 34822
503 => {
vm_block = 74; continue;
}
// C line 34820
504 => {
let _ = { let assigned = core::ptr::null_mut::<RelocEntry>(); (*(ls)).first_reloc = assigned; assigned };
vm_block = 503; continue;
}
// C line 34802
505 => {
vm_block = if ((((re) != (core::ptr::null_mut::<RelocEntry>())) as i32)) != 0 { 518 } else { 504 }; continue;
}
// C line 34802
506 => {
let _ = { let assigned = re_next; re = assigned; assigned };
vm_block = 505; continue;
}
// C line 34818
507 => {
let _ = js_free(ctx, ((re) as *mut c_void));
vm_block = 506; continue;
}
// C line 34816
508 => {
vm_block = 507; continue;
}
// C line 34815
509 => {
let _ = crate::cutils_header::put_u8(((bc_out).buf).offset((((*(re)).addr) as isize)), ((diff) as u8));
vm_block = 508; continue;
}
// C line 34814
510 => {
let _ = if ((((!(((((diff) == (((((diff) as i8)) as i32))) as i32)) != 0) as i32)) as i64)) != 0 { std::process::abort() } else { { let _ = (0 as i32); } };
vm_block = 509; continue;
}
// C line 34812
511 => {
vm_block = 507; continue;
}
// C line 34811
512 => {
let _ = crate::cutils_header::put_u16(((bc_out).buf).offset((((*(re)).addr) as isize)), ((diff) as u16));
vm_block = 511; continue;
}
// C line 34810
513 => {
let _ = if ((((!(((((diff) == (((((diff) as i16)) as i32))) as i32)) != 0) as i32)) as i64)) != 0 { std::process::abort() } else { { let _ = (0 as i32); } };
vm_block = 512; continue;
}
// C line 34808
514 => {
vm_block = 507; continue;
}
// C line 34807
515 => {
let _ = put_u32(((bc_out).buf).offset((((*(re)).addr) as isize)), ((diff) as u32));
vm_block = 514; continue;
}
// C line 34805
516 => {
vm_block = match (*(re)).size { x if x == (1 as i32) => 510, x if x == (2 as i32) => 513, x if x == (4 as i32) => 515, _ => 507, }; continue;
}
// C line 34804
517 => {
let _ = { let assigned = (*(re)).next; re_next = assigned; assigned };
vm_block = 516; continue;
}
// C line 34803
518 => {
diff = ((((((*(ls)).addr) as u32)).wrapping_sub((*(re)).addr)) as i32);
vm_block = 517; continue;
}
// C line 34802
519 => {
let _ = { let assigned = (*(ls)).first_reloc; re = assigned; assigned };
vm_block = 505; continue;
}
// C line 34800
520 => {
let _ = { let assigned = (((bc_out).size) as i32); (*(ls)).addr = assigned; assigned };
vm_block = 519; continue;
}
// C line 34799
521 => {
let _ = if ((((!((((((*(ls)).addr) == (((1 as i32)).wrapping_neg())) as i32)) != 0) as i32)) as i64)) != 0 { std::process::abort() } else { { let _ = (0 as i32); } };
vm_block = 520; continue;
}
// C line 34798
522 => {
let _ = { let assigned = core::ptr::addr_of_mut!(*(label_slots).offset((label) as isize)); ls = assigned; assigned };
vm_block = 521; continue;
}
// C line 34797
523 => {
let _ = if ((((!(((((((((label) >= ((0 as i32))) as i32)) != 0) && (((((label) < ((*(s)).label_count)) as i32)) != 0)) as i32)) != 0) as i32)) as i64)) != 0 { std::process::abort() } else { { let _ = (0 as i32); } };
vm_block = 522; continue;
}
// C line 34796
524 => {
let _ = { let assigned = ((get_u32(((bc_buf).offset(((pos) as isize))).offset((((1 as i32)) as isize)))) as i32); label = assigned; assigned };
vm_block = 523; continue;
}
// C line 34792
525 => {
vm_block = 74; continue;
}
// C line 34791
526 => {
let _ = { let assigned = ((get_u32(((bc_buf).offset(((pos) as isize))).offset((((1 as i32)) as isize)))) as i32); line_num = assigned; assigned };
vm_block = 525; continue;
}
// C line 34786
527 => {
vm_block = match op { x if x == (OP_typeof as i32) => 106, x if x == (OP_post_dec as i32) => 140, x if x == (OP_post_inc as i32) => 140, x if x == (OP_put_var_ref as i32) => 153, x if x == (OP_put_arg as i32) => 153, x if x == (OP_put_loc_check as i32) => 153, x if x == (OP_put_loc as i32) => 153, x if x == (OP_get_var_ref as i32) => 159, x if x == (OP_get_arg as i32) => 159, x if x == (OP_get_loc as i32) => 206, x if x == (OP_dup as i32) => 228, x if x == (OP_insert2 as i32) => 238, x if x == (OP_undefined as i32) => 271, x if x == (OP_to_propkey as i32) => 275, x if x == (OP_push_atom_value as i32) => 289, x if x == (OP_get_field as i32) => 297, x if x == (OP_fclosure as i32) => 305, x if x == (OP_push_const as i32) => 305, x if x == (OP_push_bigint_i32 as i32) => 319, x if x == (OP_push_i32 as i32) => 343, x if x == (OP_push_true as i32) => 357, x if x == (OP_push_false as i32) => 357, x if x == (OP_null as i32) => 374, x if x == (OP_drop as i32) => 380, x if x == (OP_with_get_ref as i32) => 401, x if x == (OP_with_make_ref as i32) => 401, x if x == (OP_with_delete_var as i32) => 401, x if x == (OP_with_put_var as i32) => 401, x if x == (OP_with_get_var as i32) => 401, x if x == (OP_if_false as i32) => 465, x if x == (OP_if_true as i32) => 465, x if x == (OP_catch as i32) => 467, x if x == (OP_gosub as i32) => 474, x if x == (OP_goto as i32) => 488, x if x == (OP_throw_error as i32) => 490, x if x == (OP_throw as i32) => 490, x if x == (OP_return_async as i32) => 490, x if x == (OP_return_undef as i32) => 490, x if x == (OP_return as i32) => 490, x if x == (OP_call_method as i32) => 502, x if x == (OP_call as i32) => 502, x if x == (OP_label as i32) => 524, x if x == (OP_line_num as i32) => 526, _ => 77, }; continue;
}
// C line 34785
528 => {
let _ = { let assigned = (pos).wrapping_add(len); pos_next = assigned; assigned };
vm_block = 527; continue;
}
// C line 34784
529 => {
let _ = { let assigned = (((*((opcode_info).as_ptr()).offset((op) as isize)).size) as i32); len = assigned; assigned };
vm_block = 528; continue;
}
// C line 34783
530 => {
let _ = { let assigned = ((*(bc_buf).offset((pos) as isize)) as i32); op = assigned; assigned };
vm_block = 529; continue;
}
// C line 34781
531 => {
let _ = { let assigned = (0 as i32); pos = assigned; assigned };
vm_block = 73; continue;
}
// C line 34778
532 => {
let _ = put_short_code(core::ptr::addr_of_mut!(bc_out), (OP_put_loc as i32), (*(s)).arg_var_object_idx);
vm_block = 531; continue;
}
// C line 34777
533 => {
let _ = dbuf_putc(core::ptr::addr_of_mut!(bc_out), (((OP_SPECIAL_OBJECT_VAR_OBJECT as i32)) as u8));
vm_block = 532; continue;
}
// C line 34776
534 => {
let _ = dbuf_putc(core::ptr::addr_of_mut!(bc_out), (((OP_special_object as i32)) as u8));
vm_block = 533; continue;
}
// C line 34775
535 => {
vm_block = if (((((*(s)).arg_var_object_idx) >= ((0 as i32))) as i32)) != 0 { 534 } else { 531 }; continue;
}
// C line 34773
536 => {
let _ = put_short_code(core::ptr::addr_of_mut!(bc_out), (OP_put_loc as i32), (*(s)).var_object_idx);
vm_block = 535; continue;
}
// C line 34772
537 => {
let _ = dbuf_putc(core::ptr::addr_of_mut!(bc_out), (((OP_SPECIAL_OBJECT_VAR_OBJECT as i32)) as u8));
vm_block = 536; continue;
}
// C line 34771
538 => {
let _ = dbuf_putc(core::ptr::addr_of_mut!(bc_out), (((OP_special_object as i32)) as u8));
vm_block = 537; continue;
}
// C line 34770
539 => {
vm_block = if (((((*(s)).var_object_idx) >= ((0 as i32))) as i32)) != 0 { 538 } else { 535 }; continue;
}
// C line 34767
540 => {
let _ = put_short_code(core::ptr::addr_of_mut!(bc_out), (OP_put_loc as i32), (*(s)).func_var_idx);
vm_block = 539; continue;
}
// C line 34766
541 => {
let _ = dbuf_putc(core::ptr::addr_of_mut!(bc_out), (((OP_SPECIAL_OBJECT_THIS_FUNC as i32)) as u8));
vm_block = 540; continue;
}
// C line 34765
542 => {
let _ = dbuf_putc(core::ptr::addr_of_mut!(bc_out), (((OP_special_object as i32)) as u8));
vm_block = 541; continue;
}
// C line 34764
543 => {
vm_block = if (((((*(s)).func_var_idx) >= ((0 as i32))) as i32)) != 0 { 542 } else { 539 }; continue;
}
// C line 34761
544 => {
let _ = put_short_code(core::ptr::addr_of_mut!(bc_out), (OP_put_loc as i32), (*(s)).arguments_var_idx);
vm_block = 543; continue;
}
// C line 34760
545 => {
let _ = put_short_code(core::ptr::addr_of_mut!(bc_out), (OP_set_loc as i32), (*(s)).arguments_arg_idx);
vm_block = 544; continue;
}
// C line 34759
546 => {
vm_block = if (((((*(s)).arguments_arg_idx) >= ((0 as i32))) as i32)) != 0 { 545 } else { 544 }; continue;
}
// C line 34749
547 => {
let _ = dbuf_putc(core::ptr::addr_of_mut!(bc_out), (((OP_SPECIAL_OBJECT_ARGUMENTS as i32)) as u8));
vm_block = 546; continue;
}
// C line 34748
548 => {
let _ = dbuf_putc(core::ptr::addr_of_mut!(bc_out), (((OP_special_object as i32)) as u8));
vm_block = 547; continue;
}
// C line 34756
549 => {
vm_block = if ((((i) < ((*(s)).arg_count)) as i32)) != 0 { 551 } else { 546 }; continue;
}
// C line 34756
550 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 549; continue;
}
// C line 34757
551 => {
let _ = capture_var(s, core::ptr::addr_of_mut!(*((*(s)).args).offset((i) as isize)));
vm_block = 550; continue;
}
// C line 34756
552 => {
let _ = { let assigned = (0 as i32); i = assigned; assigned };
vm_block = 549; continue;
}
// C line 34752
553 => {
let _ = dbuf_putc(core::ptr::addr_of_mut!(bc_out), (((OP_SPECIAL_OBJECT_MAPPED_ARGUMENTS as i32)) as u8));
vm_block = 552; continue;
}
// C line 34751
554 => {
let _ = dbuf_putc(core::ptr::addr_of_mut!(bc_out), (((OP_special_object as i32)) as u8));
vm_block = 553; continue;
}
// C line 34747
555 => {
vm_block = if ((((((((((*(s)).js_mode) as i32)) & (((1 as i32)).wrapping_shl(((0 as i32)) as u32)))) != 0) || (((!(((*(s)).has_simple_parameter_list) != 0) as i32)) != 0)) as i32)) != 0 { 548 } else { 554 }; continue;
}
// C line 34746
556 => {
vm_block = if (((((*(s)).arguments_var_idx) >= ((0 as i32))) as i32)) != 0 { 555 } else { 543 }; continue;
}
// C line 34739
557 => {
let _ = dbuf_put_u16(core::ptr::addr_of_mut!(bc_out), (((*(s)).this_var_idx) as u16));
vm_block = 556; continue;
}
// C line 34738
558 => {
let _ = dbuf_putc(core::ptr::addr_of_mut!(bc_out), (((OP_set_loc_uninitialized as i32)) as u8));
vm_block = 557; continue;
}
// C line 34742
559 => {
let _ = put_short_code(core::ptr::addr_of_mut!(bc_out), (OP_put_loc as i32), (*(s)).this_var_idx);
vm_block = 556; continue;
}
// C line 34741
560 => {
let _ = dbuf_putc(core::ptr::addr_of_mut!(bc_out), (((OP_push_this as i32)) as u8));
vm_block = 559; continue;
}
// C line 34737
561 => {
vm_block = if ((*(s)).is_derived_class_constructor) != 0 { 558 } else { 560 }; continue;
}
// C line 34736
562 => {
vm_block = if (((((*(s)).this_var_idx) >= ((0 as i32))) as i32)) != 0 { 561 } else { 556 }; continue;
}
// C line 34732
563 => {
let _ = put_short_code(core::ptr::addr_of_mut!(bc_out), (OP_put_loc as i32), (*(s)).new_target_var_idx);
vm_block = 562; continue;
}
// C line 34731
564 => {
let _ = dbuf_putc(core::ptr::addr_of_mut!(bc_out), (((OP_SPECIAL_OBJECT_NEW_TARGET as i32)) as u8));
vm_block = 563; continue;
}
// C line 34730
565 => {
let _ = dbuf_putc(core::ptr::addr_of_mut!(bc_out), (((OP_special_object as i32)) as u8));
vm_block = 564; continue;
}
// C line 34729
566 => {
vm_block = if (((((*(s)).new_target_var_idx) >= ((0 as i32))) as i32)) != 0 { 565 } else { 562 }; continue;
}
// C line 34726
567 => {
let _ = put_short_code(core::ptr::addr_of_mut!(bc_out), (OP_put_loc as i32), (*(s)).this_active_func_var_idx);
vm_block = 566; continue;
}
// C line 34725
568 => {
let _ = dbuf_putc(core::ptr::addr_of_mut!(bc_out), (((OP_SPECIAL_OBJECT_THIS_FUNC as i32)) as u8));
vm_block = 567; continue;
}
// C line 34724
569 => {
let _ = dbuf_putc(core::ptr::addr_of_mut!(bc_out), (((OP_special_object as i32)) as u8));
vm_block = 568; continue;
}
// C line 34723
570 => {
vm_block = if (((((*(s)).this_active_func_var_idx) >= ((0 as i32))) as i32)) != 0 { 569 } else { 566 }; continue;
}
// C line 34720
571 => {
let _ = put_short_code(core::ptr::addr_of_mut!(bc_out), (OP_put_loc as i32), (*(s)).home_object_var_idx);
vm_block = 570; continue;
}
// C line 34719
572 => {
let _ = dbuf_putc(core::ptr::addr_of_mut!(bc_out), (((OP_SPECIAL_OBJECT_HOME_OBJECT as i32)) as u8));
vm_block = 571; continue;
}
// C line 34718
573 => {
let _ = dbuf_putc(core::ptr::addr_of_mut!(bc_out), (((OP_special_object as i32)) as u8));
vm_block = 572; continue;
}
// C line 34717
574 => {
vm_block = if (((((*(s)).home_object_var_idx) >= ((0 as i32))) as i32)) != 0 { 573 } else { 570 }; continue;
}
// C line 34713
575 => {
let _ = { let assigned = (0 as i32); (*(s)).line_number_last_pc = assigned; assigned };
vm_block = 574; continue;
}
// C line 34712
576 => {
let _ = { let assigned = (((*(s)).source_pos) as i32); (*(s)).line_number_last = assigned; assigned };
vm_block = 575; continue;
}
// C line 34711
577 => {
return ((1 as i32)).wrapping_neg();
}
// C line 34710
578 => {
vm_block = if (((((*(s)).line_number_slots) == (core::ptr::null_mut::<LineNumberSlot>())) as i32)) != 0 { 577 } else { 576 }; continue;
}
// C line 34709
579 => {
let _ = { let assigned = ((js_mallocz((*(s)).ctx, ((size_of::<LineNumberSlot>() as usize)).wrapping_mul((((*(s)).line_number_size) as usize)))) as *mut LineNumberSlot); (*(s)).line_number_slots = assigned; assigned };
vm_block = 578; continue;
}
// C line 34708
580 => {
vm_block = if ((((((*(s)).line_number_size) != 0) && (((!(((*(s)).strip_debug()) != 0) as i32)) != 0)) as i32)) != 0 { 579 } else { 574 }; continue;
}
// C line 34704
581 => {
return ((1 as i32)).wrapping_neg();
}
// C line 34703
582 => {
vm_block = if (((((*(s)).jump_slots) == (core::ptr::null_mut::<JumpSlot>())) as i32)) != 0 { 581 } else { 580 }; continue;
}
// C line 34702
583 => {
let _ = { let assigned = ((js_mallocz((*(s)).ctx, ((size_of::<JumpSlot>() as usize)).wrapping_mul((((*(s)).jump_size) as usize)))) as *mut JumpSlot); (*(s)).jump_slots = assigned; assigned };
vm_block = 582; continue;
}
// C line 34701
584 => {
vm_block = if ((*(s)).jump_size) != 0 { 583 } else { 580 }; continue;
}
// C line 34698
585 => {
let _ = js_dbuf_bytecode_init(ctx, core::ptr::addr_of_mut!(bc_out));
vm_block = 584; continue;
}
// C line 34697
586 => {
let _ = { let assigned = { let assigned = ((((*(s)).byte_code).size) as i32); bc_len = assigned; assigned }; (cc).bc_len = assigned; assigned };
vm_block = 585; continue;
}
// C line 34696
587 => {
let _ = { let assigned = { let assigned = ((*(s)).byte_code).buf; bc_buf = assigned; assigned }; (cc).bc_buf = assigned; assigned };
vm_block = 586; continue;
}
// C line 34694
588 => {
let _ = { let assigned = (((*(s)).source_pos) as i32); line_num = assigned; assigned };
vm_block = 587; continue;
}
// C line 34692
589 => {
let _ = { let assigned = (*(s)).label_slots; label_slots = assigned; assigned };
vm_block = 588; continue;
}
_ => std::process::abort(),
} }
}
// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:34679. Bellard/Gordon MIT.
#[cfg(not(feature = "short-opcodes"))]
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn resolve_labels(mut ctx: *mut JSContext, mut s: *mut JSFunctionDef) -> i32 {
let mut vm_local_storage = Vec::<u64>::new();
let mut pos: i32 = 0;
let mut pos_next: i32 = 0;
let mut bc_len: i32 = 0;
let mut op: i32 = 0;
let mut op1: i32 = 0;
let mut len: i32 = 0;
let mut i: i32 = 0;
let mut line_num: i32 = 0;
let mut bc_buf: *const u8 = core::ptr::null();
let mut bc_out: DynBuf = core::mem::zeroed();
let mut label_slots: *mut LabelSlot = core::ptr::null_mut();
let mut ls: *mut LabelSlot = core::ptr::null_mut();
let mut re: *mut RelocEntry = core::ptr::null_mut();
let mut re_next: *mut RelocEntry = core::ptr::null_mut();
let mut cc: CodeContext = core::mem::zeroed();
let mut label: i32 = 0;
let mut val: i32 = 0;
let mut diff: i32 = 0;
let mut argc: i32 = 0;
let mut line1: i32 = 0;
let mut pos1: i32 = 0;
let mut line1_1: i32 = 0;
let mut atom: JSAtom = 0;
let mut is_with: i32 = 0;
let mut atom_1: JSAtom = 0;
let mut op1_1: i32 = 0;
let mut line2: i32 = 0;
let mut idx: i32 = 0;
let mut idx_1: i32 = 0;
let mut op1_2: i32 = 0;
let mut idx_2: i32 = 0;
let mut vm_block: usize = 398;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 35579
1 => {
return ((1 as i32)).wrapping_neg();
}
// C line 35578 labels: fail
2 => {
let _ = dbuf_free(core::ptr::addr_of_mut!(bc_out));
vm_block = 1; continue;
}
// C line 35575
3 => {
return (0 as i32);
}
// C line 35573
4 => {
return ((1 as i32)).wrapping_neg();
}
// C line 35572
5 => {
let _ = JS_ThrowOutOfMemory(ctx);
vm_block = 4; continue;
}
// C line 35571
6 => {
vm_block = if (dbuf_error(core::ptr::addr_of_mut!((*(s)).byte_code))) != 0 { 5 } else { 3 }; continue;
}
// C line 35570
7 => {
let _ = { let assigned = (1 as i32); (*(s)).use_short_opcodes = assigned; assigned };
vm_block = 6; continue;
}
// C line 35569
8 => {
let _ = { (*(s)).byte_code = core::ptr::read(core::ptr::addr_of!(bc_out)); core::ptr::read(core::ptr::addr_of!((*(s)).byte_code)) };
vm_block = 7; continue;
}
// C line 35568
9 => {
let _ = dbuf_free(core::ptr::addr_of_mut!((*(s)).byte_code));
vm_block = 8; continue;
}
// C line 35566
10 => {
let _ = { let assigned = core::ptr::null_mut::<LineNumberSlot>(); (*(s)).line_number_slots = assigned; assigned };
vm_block = 9; continue;
}
// C line 35565
11 => {
let _ = js_free(ctx, (((*(s)).line_number_slots) as *mut c_void));
vm_block = 10; continue;
}
// C line 35564
12 => {
let _ = compute_pc2line_info(s);
vm_block = 11; continue;
}
// C line 35562
13 => {
let _ = { let assigned = core::ptr::null_mut::<LabelSlot>(); (*(s)).label_slots = assigned; assigned };
vm_block = 12; continue;
}
// C line 35561
14 => {
let _ = js_free(ctx, (((*(s)).label_slots) as *mut c_void));
vm_block = 13; continue;
}
// C line 35479
15 => {
vm_block = if ((((i) < ((*(s)).label_count)) as i32)) != 0 { 17 } else { 14 }; continue;
}
// C line 35479
16 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 15; continue;
}
// C line 35480
17 => {
let _ = if ((((!((((((*(label_slots).offset((i) as isize)).first_reloc) == (core::ptr::null_mut::<RelocEntry>())) as i32)) != 0) as i32)) as i64)) != 0 { std::process::abort() } else { { let _ = (0 as i32); } };
vm_block = 16; continue;
}
// C line 35479
18 => {
let _ = { let assigned = (0 as i32); i = assigned; assigned };
vm_block = 15; continue;
}
// C line 34781
19 => {
vm_block = if ((((pos) < (bc_len)) as i32)) != 0 { 343 } else { 18 }; continue;
}
// C line 34781
20 => {
let _ = { let assigned = pos_next; pos = assigned; assigned };
vm_block = 19; continue;
}
// C line 35474
21 => {
vm_block = 20; continue;
}
// C line 35473
22 => {
let _ = crate::cutils::dbuf_put(core::ptr::addr_of_mut!(bc_out), (bc_buf).offset(((pos) as isize)), ((len) as usize));
vm_block = 21; continue;
}
// C line 35472 labels: no_change
23 => {
let _ = add_pc2line_info(s, (((bc_out).size) as u32), ((line_num) as u32));
vm_block = 22; continue;
}
// C line 35426
24 => {
vm_block = 23; continue;
}
// C line 35423
25 => {
vm_block = 20; continue;
}
// C line 35422
26 => {
let _ = { let assigned = (cc).pos; pos_next = assigned; assigned };
vm_block = 25; continue;
}
// C line 35421
27 => {
let _ = dbuf_putc(core::ptr::addr_of_mut!(bc_out), (((OP_put_array_el as i32)) as u8));
vm_block = 26; continue;
}
// C line 35420
28 => {
let _ = dbuf_putc(core::ptr::addr_of_mut!(bc_out), ((((OP_dec as i32)).wrapping_add((op).wrapping_sub((OP_post_dec as i32)))) as u8));
vm_block = 27; continue;
}
// C line 35419
29 => {
let _ = add_pc2line_info(s, (((bc_out).size) as u32), ((line_num) as u32));
vm_block = 28; continue;
}
// C line 35418
30 => {
let _ = { let assigned = (cc).line_num; line_num = assigned; assigned };
vm_block = 29; continue;
}
// C line 35418
31 => {
vm_block = if (((((cc).line_num) >= ((0 as i32))) as i32)) != 0 { 30 } else { 29 }; continue;
}
// C line 35417
32 => {
vm_block = if (code_match(core::ptr::addr_of_mut!(cc), pos_next, &[((OP_perm4 as i32)) as i32, ((OP_put_array_el as i32)) as i32, ((OP_drop as i32)) as i32, (((1 as i32)).wrapping_neg()) as i32])) != 0 { 31 } else { 24 }; continue;
}
// C line 35415
33 => {
vm_block = 20; continue;
}
// C line 35414
34 => {
let _ = { let assigned = (cc).pos; pos_next = assigned; assigned };
vm_block = 33; continue;
}
// C line 35413
35 => {
let _ = dbuf_put_u32(core::ptr::addr_of_mut!(bc_out), (cc).atom);
vm_block = 34; continue;
}
// C line 35412
36 => {
let _ = dbuf_putc(core::ptr::addr_of_mut!(bc_out), (((OP_put_field as i32)) as u8));
vm_block = 35; continue;
}
// C line 35411
37 => {
let _ = dbuf_putc(core::ptr::addr_of_mut!(bc_out), ((((OP_dec as i32)).wrapping_add((op).wrapping_sub((OP_post_dec as i32)))) as u8));
vm_block = 36; continue;
}
// C line 35410
38 => {
let _ = add_pc2line_info(s, (((bc_out).size) as u32), ((line_num) as u32));
vm_block = 37; continue;
}
// C line 35409
39 => {
let _ = { let assigned = (cc).line_num; line_num = assigned; assigned };
vm_block = 38; continue;
}
// C line 35409
40 => {
vm_block = if (((((cc).line_num) >= ((0 as i32))) as i32)) != 0 { 39 } else { 38 }; continue;
}
// C line 35408
41 => {
vm_block = if (code_match(core::ptr::addr_of_mut!(cc), pos_next, &[((OP_perm3 as i32)) as i32, ((OP_put_field as i32)) as i32, ((OP_drop as i32)) as i32, (((1 as i32)).wrapping_neg()) as i32])) != 0 { 40 } else { 32 }; continue;
}
// C line 35406
42 => {
vm_block = 20; continue;
}
// C line 35405
43 => {
let _ = put_short_code(core::ptr::addr_of_mut!(bc_out), op1_2, idx_2);
vm_block = 42; continue;
}
// C line 35404
44 => {
let _ = dbuf_putc(core::ptr::addr_of_mut!(bc_out), ((((OP_dec as i32)).wrapping_add((op).wrapping_sub((OP_post_dec as i32)))) as u8));
vm_block = 43; continue;
}
// C line 35403
45 => {
let _ = add_pc2line_info(s, (((bc_out).size) as u32), ((line_num) as u32));
vm_block = 44; continue;
}
// C line 35401
46 => {
let _ = { let assigned = (cc).pos; pos_next = assigned; assigned };
vm_block = 45; continue;
}
// C line 35400
47 => {
let _ = { op1_2 = ((((op1_2) as i32)).wrapping_add((1 as i32))) as i32; op1_2 };
vm_block = 46; continue;
}
// C line 35399
48 => {
let _ = { let assigned = (cc).line_num; line_num = assigned; assigned };
vm_block = 47; continue;
}
// C line 35399
49 => {
vm_block = if (((((cc).line_num) >= ((0 as i32))) as i32)) != 0 { 48 } else { 47 }; continue;
}
// C line 35398
50 => {
vm_block = if (code_match(core::ptr::addr_of_mut!(cc), (cc).pos, &[((op1_2).wrapping_sub((1 as i32))) as i32, (idx_2) as i32, (((1 as i32)).wrapping_neg()) as i32])) != 0 { 49 } else { 45 }; continue;
}
// C line 35397
51 => {
let _ = { let assigned = (cc).pos; pos_next = assigned; assigned };
vm_block = 50; continue;
}
// C line 35396
52 => {
let _ = { let assigned = (cc).idx; idx_2 = assigned; assigned };
vm_block = 51; continue;
}
// C line 35395
53 => {
let _ = { let assigned = (cc).op; op1_2 = assigned; assigned };
vm_block = 52; continue;
}
// C line 35394
54 => {
let _ = { let assigned = (cc).line_num; line_num = assigned; assigned };
vm_block = 53; continue;
}
// C line 35394
55 => {
vm_block = if (((((cc).line_num) >= ((0 as i32))) as i32)) != 0 { 54 } else { 53 }; continue;
}
// C line 35393
56 => {
vm_block = if (code_match(core::ptr::addr_of_mut!(cc), pos_next, &[((((((OP_put_loc as i32)) | (((OP_put_arg as i32)).wrapping_shl(((8 as i32)) as u32)))) | (((OP_put_var_ref as i32)).wrapping_shl(((16 as i32)) as u32)))) as i32, (((1 as i32)).wrapping_neg()) as i32, ((OP_drop as i32)) as i32, (((1 as i32)).wrapping_neg()) as i32])) != 0 { 55 } else { 41 }; continue;
}
// C line 35386
57 => {
vm_block = if ((1 as i32)) != 0 { 56 } else { 24 }; continue;
}
// C line 35382
58 => {
vm_block = 23; continue;
}
// C line 35380
59 => {
vm_block = 20; continue;
}
// C line 35379
60 => {
let _ = put_short_code(core::ptr::addr_of_mut!(bc_out), op, idx_1);
vm_block = 59; continue;
}
// C line 35378
61 => {
let _ = add_pc2line_info(s, (((bc_out).size) as u32), ((line_num) as u32));
vm_block = 60; continue;
}
// C line 35376
62 => {
vm_block = 20; continue;
}
// C line 35375
63 => {
let _ = { let assigned = (cc).pos; pos_next = assigned; assigned };
vm_block = 62; continue;
}
// C line 35374
64 => {
let _ = put_short_code(core::ptr::addr_of_mut!(bc_out), (op).wrapping_add((1 as i32)), idx_1);
vm_block = 63; continue;
}
// C line 35373
65 => {
let _ = add_pc2line_info(s, (((bc_out).size) as u32), ((line_num) as u32));
vm_block = 64; continue;
}
// C line 35372
66 => {
let _ = { let assigned = (cc).line_num; line_num = assigned; assigned };
vm_block = 65; continue;
}
// C line 35372
67 => {
vm_block = if (((((cc).line_num) >= ((0 as i32))) as i32)) != 0 { 66 } else { 65 }; continue;
}
// C line 35371
68 => {
vm_block = if (code_match(core::ptr::addr_of_mut!(cc), pos_next, &[((op).wrapping_sub((1 as i32))) as i32, (idx_1) as i32, (((1 as i32)).wrapping_neg()) as i32])) != 0 { 67 } else { 61 }; continue;
}
// C line 35370
69 => {
let _ = { let assigned = ((get_u16(((bc_buf).offset(((pos) as isize))).offset((((1 as i32)) as isize)))) as i32); idx_1 = assigned; assigned };
vm_block = 68; continue;
}
// C line 35367
70 => {
vm_block = if ((1 as i32)) != 0 { 69 } else { 58 }; continue;
}
// C line 35350
71 => {
vm_block = 23; continue;
}
// C line 35348
72 => {
vm_block = 20; continue;
}
// C line 35347
73 => {
let _ = put_short_code(core::ptr::addr_of_mut!(bc_out), op, idx);
vm_block = 72; continue;
}
// C line 35346
74 => {
let _ = add_pc2line_info(s, (((bc_out).size) as u32), ((line_num) as u32));
vm_block = 73; continue;
}
// C line 35344
75 => {
vm_block = 20; continue;
}
// C line 35343
76 => {
let _ = { let assigned = (cc).pos; pos_next = assigned; assigned };
vm_block = 75; continue;
}
// C line 35342
77 => {
let _ = dbuf_putc(core::ptr::addr_of_mut!(bc_out), ((idx) as u8));
vm_block = 76; continue;
}
// C line 35341
78 => {
let _ = dbuf_putc(core::ptr::addr_of_mut!(bc_out), (((OP_add_loc as i32)) as u8));
vm_block = 77; continue;
}
// C line 35340
79 => {
let _ = put_short_code(core::ptr::addr_of_mut!(bc_out), (cc).op, (cc).idx);
vm_block = 78; continue;
}
// C line 35339
80 => {
let _ = add_pc2line_info(s, (((bc_out).size) as u32), ((line_num) as u32));
vm_block = 79; continue;
}
// C line 35338
81 => {
let _ = { let assigned = (cc).line_num; line_num = assigned; assigned };
vm_block = 80; continue;
}
// C line 35338
82 => {
vm_block = if (((((cc).line_num) >= ((0 as i32))) as i32)) != 0 { 81 } else { 80 }; continue;
}
// C line 35337
83 => {
vm_block = if (code_match(core::ptr::addr_of_mut!(cc), pos_next, &[((((((OP_get_loc as i32)) | (((OP_get_arg as i32)).wrapping_shl(((8 as i32)) as u32)))) | (((OP_get_var_ref as i32)).wrapping_shl(((16 as i32)) as u32)))) as i32, (((1 as i32)).wrapping_neg()) as i32, ((OP_add as i32)) as i32, ((OP_dup as i32)) as i32, ((OP_put_loc as i32)) as i32, (idx) as i32, ((OP_drop as i32)) as i32, (((1 as i32)).wrapping_neg()) as i32])) != 0 { 82 } else { 74 }; continue;
}
// C line 35330
84 => {
vm_block = 20; continue;
}
// C line 35329
85 => {
let _ = { let assigned = (cc).pos; pos_next = assigned; assigned };
vm_block = 84; continue;
}
// C line 35328
86 => {
let _ = dbuf_putc(core::ptr::addr_of_mut!(bc_out), ((idx) as u8));
vm_block = 85; continue;
}
// C line 35327
87 => {
let _ = dbuf_putc(core::ptr::addr_of_mut!(bc_out), (((OP_add_loc as i32)) as u8));
vm_block = 86; continue;
}
// C line 35326
88 => {
let _ = push_short_int(core::ptr::addr_of_mut!(bc_out), (cc).label);
vm_block = 87; continue;
}
// C line 35325
89 => {
let _ = add_pc2line_info(s, (((bc_out).size) as u32), ((line_num) as u32));
vm_block = 88; continue;
}
// C line 35324
90 => {
let _ = { let assigned = (cc).line_num; line_num = assigned; assigned };
vm_block = 89; continue;
}
// C line 35324
91 => {
vm_block = if (((((cc).line_num) >= ((0 as i32))) as i32)) != 0 { 90 } else { 89 }; continue;
}
// C line 35323
92 => {
vm_block = if (code_match(core::ptr::addr_of_mut!(cc), pos_next, &[((OP_push_i32 as i32)) as i32, ((OP_add as i32)) as i32, ((OP_dup as i32)) as i32, ((OP_put_loc as i32)) as i32, (idx) as i32, ((OP_drop as i32)) as i32, (((1 as i32)).wrapping_neg()) as i32])) != 0 { 91 } else { 83 }; continue;
}
// C line 35318
93 => {
vm_block = 20; continue;
}
// C line 35317
94 => {
let _ = { let assigned = (cc).pos; pos_next = assigned; assigned };
vm_block = 93; continue;
}
// C line 35316
95 => {
let _ = dbuf_putc(core::ptr::addr_of_mut!(bc_out), ((idx) as u8));
vm_block = 94; continue;
}
// C line 35315
96 => {
let _ = dbuf_putc(core::ptr::addr_of_mut!(bc_out), (((OP_add_loc as i32)) as u8));
vm_block = 95; continue;
}
// C line 35313
97 => {
let _ = dbuf_put_u32(core::ptr::addr_of_mut!(bc_out), (cc).atom);
vm_block = 96; continue;
}
// C line 35312
98 => {
let _ = dbuf_putc(core::ptr::addr_of_mut!(bc_out), (((OP_push_atom_value as i32)) as u8));
vm_block = 97; continue;
}
// C line 35304
99 => {
let _ = add_pc2line_info(s, (((bc_out).size) as u32), ((line_num) as u32));
vm_block = 98; continue;
}
// C line 35303
100 => {
let _ = { let assigned = (cc).line_num; line_num = assigned; assigned };
vm_block = 99; continue;
}
// C line 35303
101 => {
vm_block = if (((((cc).line_num) >= ((0 as i32))) as i32)) != 0 { 100 } else { 99 }; continue;
}
// C line 35302
102 => {
vm_block = if (code_match(core::ptr::addr_of_mut!(cc), pos_next, &[((OP_push_atom_value as i32)) as i32, ((OP_add as i32)) as i32, ((OP_dup as i32)) as i32, ((OP_put_loc as i32)) as i32, (idx) as i32, ((OP_drop as i32)) as i32, (((1 as i32)).wrapping_neg()) as i32])) != 0 { 101 } else { 92 }; continue;
}
// C line 35297
103 => {
vm_block = 20; continue;
}
// C line 35296
104 => {
let _ = { let assigned = (cc).pos; pos_next = assigned; assigned };
vm_block = 103; continue;
}
// C line 35295
105 => {
let _ = dbuf_putc(core::ptr::addr_of_mut!(bc_out), ((idx) as u8));
vm_block = 104; continue;
}
// C line 35294
106 => {
let _ = dbuf_putc(core::ptr::addr_of_mut!(bc_out), ((if (((((((((cc).op) == ((OP_inc as i32))) as i32)) != 0) || ((((((cc).op) == ((OP_post_inc as i32))) as i32)) != 0)) as i32)) != 0 { (OP_inc_loc as i32) } else { (OP_dec_loc as i32) }) as u8));
vm_block = 105; continue;
}
// C line 35293
107 => {
let _ = add_pc2line_info(s, (((bc_out).size) as u32), ((line_num) as u32));
vm_block = 106; continue;
}
// C line 35292
108 => {
let _ = { let assigned = (cc).line_num; line_num = assigned; assigned };
vm_block = 107; continue;
}
// C line 35292
109 => {
vm_block = if (((((cc).line_num) >= ((0 as i32))) as i32)) != 0 { 108 } else { 107 }; continue;
}
// C line 35290
110 => {
vm_block = if (((((code_match(core::ptr::addr_of_mut!(cc), pos_next, &[((((OP_post_dec as i32)) | (((OP_post_inc as i32)).wrapping_shl(((8 as i32)) as u32)))) as i32, ((OP_put_loc as i32)) as i32, (idx) as i32, ((OP_drop as i32)) as i32, (((1 as i32)).wrapping_neg()) as i32])) != 0) || ((code_match(core::ptr::addr_of_mut!(cc), pos_next, &[((((OP_dec as i32)) | (((OP_inc as i32)).wrapping_shl(((8 as i32)) as u32)))) as i32, ((OP_dup as i32)) as i32, ((OP_put_loc as i32)) as i32, (idx) as i32, ((OP_drop as i32)) as i32, (((1 as i32)).wrapping_neg()) as i32])) != 0)) as i32)) != 0 { 109 } else { 102 }; continue;
}
// C line 35289
111 => {
vm_block = 23; continue;
}
// C line 35288
112 => {
vm_block = if ((((idx) >= ((256 as i32))) as i32)) != 0 { 111 } else { 110 }; continue;
}
// C line 35287
113 => {
let _ = { let assigned = ((get_u16(((bc_buf).offset(((pos) as isize))).offset((((1 as i32)) as isize)))) as i32); idx = assigned; assigned };
vm_block = 112; continue;
}
// C line 35279
114 => {
vm_block = if ((1 as i32)) != 0 { 113 } else { 71 }; continue;
}
// C line 35276
115 => {
vm_block = 23; continue;
}
// C line 35273
116 => {
vm_block = 20; continue;
}
// C line 35272
117 => {
let _ = { let assigned = line2; line_num = assigned; assigned };
vm_block = 116; continue;
}
// C line 35272
118 => {
vm_block = if ((((line2) >= ((0 as i32))) as i32)) != 0 { 117 } else { 116 }; continue;
}
// C line 35271
119 => {
let _ = put_short_code(core::ptr::addr_of_mut!(bc_out), op1_1, (cc).idx);
vm_block = 118; continue;
}
// C line 35270
120 => {
let _ = add_pc2line_info(s, (((bc_out).size) as u32), ((line_num) as u32));
vm_block = 119; continue;
}
// C line 35267
121 => {
let _ = { let assigned = (cc).pos; pos_next = assigned; assigned };
vm_block = 120; continue;
}
// C line 35266
122 => {
let _ = { op1_1 = ((((op1_1) as i32)).wrapping_add((1 as i32))) as i32; op1_1 };
vm_block = 121; continue;
}
// C line 35265
123 => {
let _ = { let assigned = (cc).line_num; line2 = assigned; assigned };
vm_block = 122; continue;
}
// C line 35264
124 => {
vm_block = if (code_match(core::ptr::addr_of_mut!(cc), (cc).pos, &[((op1_1).wrapping_sub((1 as i32))) as i32, ((cc).idx) as i32, (((1 as i32)).wrapping_neg()) as i32])) != 0 { 123 } else { 120 }; continue;
}
// C line 35263
125 => {
let _ = { let assigned = (cc).pos; pos_next = assigned; assigned };
vm_block = 124; continue;
}
// C line 35262
126 => {
let _ = { op1_1 = ((((op1_1) as i32)).wrapping_sub((1 as i32))) as i32; op1_1 };
vm_block = 125; continue;
}
// C line 35261
127 => {
let _ = { let assigned = (cc).line_num; line_num = assigned; assigned };
vm_block = 126; continue;
}
// C line 35261
128 => {
vm_block = if (((((cc).line_num) >= ((0 as i32))) as i32)) != 0 { 127 } else { 126 }; continue;
}
// C line 35260
129 => {
vm_block = if (code_match(core::ptr::addr_of_mut!(cc), (cc).pos, &[((OP_drop as i32)) as i32, (((1 as i32)).wrapping_neg()) as i32])) != 0 { 128 } else { 120 }; continue;
}
// C line 35259
130 => {
let _ = { let assigned = (cc).pos; pos_next = assigned; assigned };
vm_block = 129; continue;
}
// C line 35258
131 => {
let _ = { let assigned = ((cc).op).wrapping_add((1 as i32)); op1_1 = assigned; assigned };
vm_block = 130; continue;
}
// C line 35257
132 => {
let _ = { let assigned = (cc).line_num; line_num = assigned; assigned };
vm_block = 131; continue;
}
// C line 35257
133 => {
vm_block = if (((((cc).line_num) >= ((0 as i32))) as i32)) != 0 { 132 } else { 131 }; continue;
}
// C line 35256
134 => {
vm_block = if (code_match(core::ptr::addr_of_mut!(cc), pos_next, &[((((((((OP_put_loc as i32)) | (((OP_put_loc_check as i32)).wrapping_shl(((8 as i32)) as u32)))) | (((OP_put_arg as i32)).wrapping_shl(((16 as i32)) as u32)))) | (((OP_put_var_ref as i32)).wrapping_shl(((24 as i32)) as u32)))) as i32, (((1 as i32)).wrapping_neg()) as i32, (((1 as i32)).wrapping_neg()) as i32])) != 0 { 133 } else { 115 }; continue;
}
// C line 35254
135 => {
line2 = ((1 as i32)).wrapping_neg();
vm_block = 134; continue;
}
// C line 35252
136 => {
vm_block = if ((1 as i32)) != 0 { 135 } else { 115 }; continue;
}
// C line 35249
137 => {
vm_block = 23; continue;
}
// C line 35246
138 => {
vm_block = 20; continue;
}
// C line 35245
139 => {
let _ = { let assigned = (cc).pos; pos_next = assigned; assigned };
vm_block = 138; continue;
}
// C line 35244
140 => {
let _ = dbuf_put_u32(core::ptr::addr_of_mut!(bc_out), (cc).atom);
vm_block = 139; continue;
}
// C line 35243
141 => {
let _ = dbuf_putc(core::ptr::addr_of_mut!(bc_out), (((OP_put_field as i32)) as u8));
vm_block = 140; continue;
}
// C line 35242
142 => {
let _ = add_pc2line_info(s, (((bc_out).size) as u32), ((line_num) as u32));
vm_block = 141; continue;
}
// C line 35241
143 => {
let _ = { let assigned = (cc).line_num; line_num = assigned; assigned };
vm_block = 142; continue;
}
// C line 35241
144 => {
vm_block = if (((((cc).line_num) >= ((0 as i32))) as i32)) != 0 { 143 } else { 142 }; continue;
}
// C line 35240
145 => {
vm_block = if (code_match(core::ptr::addr_of_mut!(cc), pos_next, &[((OP_put_field as i32)) as i32, ((OP_drop as i32)) as i32, (((1 as i32)).wrapping_neg()) as i32])) != 0 { 144 } else { 137 }; continue;
}
// C line 35236
146 => {
vm_block = if ((1 as i32)) != 0 { 145 } else { 137 }; continue;
}
// C line 35233
147 => {
vm_block = 23; continue;
}
// C line 35210
148 => {
vm_block = 225; continue;
}
// C line 35209
149 => {
let _ = { let assigned = (0 as i32); val = assigned; assigned };
vm_block = 148; continue;
}
// C line 35208
150 => {
vm_block = if (code_match(core::ptr::addr_of_mut!(cc), pos_next, &[((((OP_if_false as i32)) | (((OP_if_true as i32)).wrapping_shl(((8 as i32)) as u32)))) as i32, (((1 as i32)).wrapping_neg()) as i32])) != 0 { 149 } else { 147 }; continue;
}
// C line 35205
151 => {
vm_block = 20; continue;
}
// C line 35204
152 => {
let _ = { let assigned = (cc).pos; pos_next = assigned; assigned };
vm_block = 151; continue;
}
// C line 35203
153 => {
let _ = dbuf_putc(core::ptr::addr_of_mut!(bc_out), (((OP_return_undef as i32)) as u8));
vm_block = 152; continue;
}
// C line 35202
154 => {
let _ = add_pc2line_info(s, (((bc_out).size) as u32), ((line_num) as u32));
vm_block = 153; continue;
}
// C line 35201
155 => {
let _ = { let assigned = (cc).line_num; line_num = assigned; assigned };
vm_block = 154; continue;
}
// C line 35201
156 => {
vm_block = if (((((cc).line_num) >= ((0 as i32))) as i32)) != 0 { 155 } else { 154 }; continue;
}
// C line 35200
157 => {
vm_block = if (code_match(core::ptr::addr_of_mut!(cc), pos_next, &[((OP_return as i32)) as i32, (((1 as i32)).wrapping_neg()) as i32])) != 0 { 156 } else { 150 }; continue;
}
// C line 35197
158 => {
vm_block = 20; continue;
}
// C line 35196
159 => {
let _ = { let assigned = (cc).pos; pos_next = assigned; assigned };
vm_block = 158; continue;
}
// C line 35195
160 => {
let _ = { let assigned = (cc).line_num; line_num = assigned; assigned };
vm_block = 159; continue;
}
// C line 35195
161 => {
vm_block = if (((((cc).line_num) >= ((0 as i32))) as i32)) != 0 { 160 } else { 159 }; continue;
}
// C line 35194
162 => {
vm_block = if (code_match(core::ptr::addr_of_mut!(cc), pos_next, &[((OP_drop as i32)) as i32, (((1 as i32)).wrapping_neg()) as i32])) != 0 { 161 } else { 157 }; continue;
}
// C line 35192
163 => {
vm_block = if ((1 as i32)) != 0 { 162 } else { 147 }; continue;
}
// C line 35189
164 => {
vm_block = 23; continue;
}
// C line 35186
165 => {
vm_block = 20; continue;
}
// C line 35183
166 => {
vm_block = if (((((((((code_match(core::ptr::addr_of_mut!(cc), pos_next, &[((((((OP_get_loc as i32)) | (((OP_get_arg as i32)).wrapping_shl(((8 as i32)) as u32)))) | (((OP_get_var_ref as i32)).wrapping_shl(((16 as i32)) as u32)))) as i32, (((1 as i32)).wrapping_neg()) as i32, ((OP_put_array_el as i32)) as i32, (((1 as i32)).wrapping_neg()) as i32])) != 0) || ((code_match(core::ptr::addr_of_mut!(cc), pos_next, &[((((((OP_push_i32 as i32)) | (((OP_push_const as i32)).wrapping_shl(((8 as i32)) as u32)))) | (((OP_push_atom_value as i32)).wrapping_shl(((16 as i32)) as u32)))) as i32, ((OP_put_array_el as i32)) as i32, (((1 as i32)).wrapping_neg()) as i32])) != 0)) as i32)) != 0) || ((code_match(core::ptr::addr_of_mut!(cc), pos_next, &[((((((((OP_undefined as i32)) | (((OP_null as i32)).wrapping_shl(((8 as i32)) as u32)))) | (((OP_push_true as i32)).wrapping_shl(((16 as i32)) as u32)))) | (((OP_push_false as i32)).wrapping_shl(((24 as i32)) as u32)))) as i32, ((OP_put_array_el as i32)) as i32, (((1 as i32)).wrapping_neg()) as i32])) != 0)) as i32)) != 0 { 165 } else { 164 }; continue;
}
// C line 35181
167 => {
vm_block = if ((1 as i32)) != 0 { 166 } else { 164 }; continue;
}
// C line 35178
168 => {
vm_block = 23; continue;
}
// C line 35167
169 => {
vm_block = 20; continue;
}
// C line 35166
170 => {
let _ = { let assigned = (cc).pos; pos_next = assigned; assigned };
vm_block = 169; continue;
}
// C line 35165
171 => {
let _ = { let assigned = (cc).line_num; line_num = assigned; assigned };
vm_block = 170; continue;
}
// C line 35165
172 => {
vm_block = if (((((cc).line_num) >= ((0 as i32))) as i32)) != 0 { 171 } else { 170 }; continue;
}
// C line 35164
173 => {
let _ = JS_FreeAtom(ctx, atom_1);
vm_block = 172; continue;
}
// C line 35163
174 => {
vm_block = if (code_match(core::ptr::addr_of_mut!(cc), pos_next, &[((OP_drop as i32)) as i32, (((1 as i32)).wrapping_neg()) as i32])) != 0 { 173 } else { 168 }; continue;
}
// C line 35161
175 => {
atom_1 = get_u32(((bc_buf).offset(((pos) as isize))).offset((((1 as i32)) as isize)));
vm_block = 174; continue;
}
// C line 35160
176 => {
vm_block = if ((1 as i32)) != 0 { 175 } else { 168 }; continue;
}
// C line 35131
177 => {
vm_block = 23; continue;
}
// C line 35128
178 => {
vm_block = 20; continue;
}
// C line 35127
179 => {
let _ = { let assigned = (cc).pos; pos_next = assigned; assigned };
vm_block = 178; continue;
}
// C line 35121
180 => {
let _ = { let assigned = (cc).line_num; line_num = assigned; assigned };
vm_block = 179; continue;
}
// C line 35121
181 => {
vm_block = if (((((cc).line_num) >= ((0 as i32))) as i32)) != 0 { 180 } else { 179 }; continue;
}
// C line 35125
182 => {
let _ = dbuf_put_u32(core::ptr::addr_of_mut!(bc_out), (((val).wrapping_neg()) as u32));
vm_block = 179; continue;
}
// C line 35124
183 => {
let _ = dbuf_putc(core::ptr::addr_of_mut!(bc_out), (((OP_push_bigint_i32 as i32)) as u8));
vm_block = 182; continue;
}
// C line 35123
184 => {
let _ = add_pc2line_info(s, (((bc_out).size) as u32), ((line_num) as u32));
vm_block = 183; continue;
}
// C line 35120
185 => {
vm_block = if (code_match(core::ptr::addr_of_mut!(cc), (cc).pos, &[((OP_drop as i32)) as i32, (((1 as i32)).wrapping_neg()) as i32])) != 0 { 181 } else { 184 }; continue;
}
// C line 35119
186 => {
let _ = { let assigned = (cc).line_num; line_num = assigned; assigned };
vm_block = 185; continue;
}
// C line 35119
187 => {
vm_block = if (((((cc).line_num) >= ((0 as i32))) as i32)) != 0 { 186 } else { 185 }; continue;
}
// C line 35117
188 => {
vm_block = if ((((((((val) != ((((2147483647 as i32)).wrapping_neg()).wrapping_sub((1 as i32)))) as i32)) != 0) && ((code_match(core::ptr::addr_of_mut!(cc), pos_next, &[((OP_neg as i32)) as i32, (((1 as i32)).wrapping_neg()) as i32])) != 0)) as i32)) != 0 { 187 } else { 177 }; continue;
}
// C line 35116
189 => {
let _ = { let assigned = crate::cutils_header::get_i32(((bc_buf).offset(((pos) as isize))).offset((((1 as i32)) as isize))); val = assigned; assigned };
vm_block = 188; continue;
}
// C line 35114
190 => {
vm_block = if ((1 as i32)) != 0 { 189 } else { 177 }; continue;
}
// C line 35111
191 => {
vm_block = 23; continue;
}
// C line 35109
192 => {
vm_block = 20; continue;
}
// C line 35108
193 => {
let _ = push_short_int(core::ptr::addr_of_mut!(bc_out), val);
vm_block = 192; continue;
}
// C line 35107
194 => {
let _ = add_pc2line_info(s, (((bc_out).size) as u32), ((line_num) as u32));
vm_block = 193; continue;
}
// C line 35105
195 => {
vm_block = 225; continue;
}
// C line 35104
196 => {
let _ = { let assigned = (((val) != ((0 as i32))) as i32); val = assigned; assigned };
vm_block = 195; continue;
}
// C line 35103
197 => {
vm_block = if (code_match(core::ptr::addr_of_mut!(cc), pos_next, &[((((OP_if_false as i32)) | (((OP_if_true as i32)).wrapping_shl(((8 as i32)) as u32)))) as i32, (((1 as i32)).wrapping_neg()) as i32])) != 0 { 196 } else { 194 }; continue;
}
// C line 35100
198 => {
vm_block = 20; continue;
}
// C line 35099
199 => {
let _ = { let assigned = (cc).pos; pos_next = assigned; assigned };
vm_block = 198; continue;
}
// C line 35098
200 => {
let _ = { let assigned = (cc).line_num; line_num = assigned; assigned };
vm_block = 199; continue;
}
// C line 35098
201 => {
vm_block = if (((((cc).line_num) >= ((0 as i32))) as i32)) != 0 { 200 } else { 199 }; continue;
}
// C line 35097
202 => {
vm_block = if (code_match(core::ptr::addr_of_mut!(cc), pos_next, &[((OP_drop as i32)) as i32, (((1 as i32)).wrapping_neg()) as i32])) != 0 { 201 } else { 197 }; continue;
}
// C line 35094
203 => {
vm_block = 20; continue;
}
// C line 35093
204 => {
let _ = { let assigned = (cc).pos; pos_next = assigned; assigned };
vm_block = 203; continue;
}
// C line 35088
205 => {
let _ = { let assigned = (cc).line_num; line_num = assigned; assigned };
vm_block = 204; continue;
}
// C line 35088
206 => {
vm_block = if (((((cc).line_num) >= ((0 as i32))) as i32)) != 0 { 205 } else { 204 }; continue;
}
// C line 35091
207 => {
let _ = push_short_int(core::ptr::addr_of_mut!(bc_out), (val).wrapping_neg());
vm_block = 204; continue;
}
// C line 35090
208 => {
let _ = add_pc2line_info(s, (((bc_out).size) as u32), ((line_num) as u32));
vm_block = 207; continue;
}
// C line 35087
209 => {
vm_block = if (code_match(core::ptr::addr_of_mut!(cc), (cc).pos, &[((OP_drop as i32)) as i32, (((1 as i32)).wrapping_neg()) as i32])) != 0 { 206 } else { 208 }; continue;
}
// C line 35086
210 => {
let _ = { let assigned = (cc).line_num; line_num = assigned; assigned };
vm_block = 209; continue;
}
// C line 35086
211 => {
vm_block = if (((((cc).line_num) >= ((0 as i32))) as i32)) != 0 { 210 } else { 209 }; continue;
}
// C line 35084
212 => {
vm_block = if ((((((((((((val) != ((((2147483647 as i32)).wrapping_neg()).wrapping_sub((1 as i32)))) as i32)) != 0) && (((((val) != ((0 as i32))) as i32)) != 0)) as i32)) != 0) && ((code_match(core::ptr::addr_of_mut!(cc), pos_next, &[((OP_neg as i32)) as i32, (((1 as i32)).wrapping_neg()) as i32])) != 0)) as i32)) != 0 { 211 } else { 202 }; continue;
}
// C line 35083
213 => {
let _ = { let assigned = crate::cutils_header::get_i32(((bc_buf).offset(((pos) as isize))).offset((((1 as i32)) as isize))); val = assigned; assigned };
vm_block = 212; continue;
}
// C line 35081
214 => {
vm_block = if ((1 as i32)) != 0 { 213 } else { 191 }; continue;
}
// C line 35078
215 => {
vm_block = 23; continue;
}
// C line 35067
216 => {
vm_block = 300; continue;
}
// C line 35066
217 => {
let _ = { let assigned = (cc).label; label = assigned; assigned };
vm_block = 216; continue;
}
// C line 35065
218 => {
let _ = { let assigned = (OP_goto as i32); op = assigned; assigned };
vm_block = 217; continue;
}
// C line 35064
219 => {
let _ = { let assigned = (cc).pos; pos_next = assigned; assigned };
vm_block = 218; continue;
}
// C line 35074
220 => {
vm_block = 20; continue;
}
// C line 35073
221 => {
let _ = update_label(s, (cc).label, ((1 as i32)).wrapping_neg());
vm_block = 220; continue;
}
// C line 35072
222 => {
let _ = { let assigned = (cc).pos; pos_next = assigned; assigned };
vm_block = 221; continue;
}
// C line 35060
223 => {
vm_block = if ((((val) == (((cc).op).wrapping_sub((OP_if_false as i32)))) as i32)) != 0 { 219 } else { 222 }; continue;
}
// C line 35059
224 => {
let _ = { let assigned = (cc).line_num; line_num = assigned; assigned };
vm_block = 223; continue;
}
// C line 35059 labels: has_constant_test
225 => {
vm_block = if (((((cc).line_num) >= ((0 as i32))) as i32)) != 0 { 224 } else { 223 }; continue;
}
// C line 35057
226 => {
vm_block = if (code_match(core::ptr::addr_of_mut!(cc), pos_next, &[((((OP_if_false as i32)) | (((OP_if_true as i32)).wrapping_shl(((8 as i32)) as u32)))) as i32, (((1 as i32)).wrapping_neg()) as i32])) != 0 { 225 } else { 215 }; continue;
}
// C line 35056
227 => {
let _ = { let assigned = (((op) == ((OP_push_true as i32))) as i32); val = assigned; assigned };
vm_block = 226; continue;
}
// C line 35055
228 => {
vm_block = if ((1 as i32)) != 0 { 227 } else { 215 }; continue;
}
// C line 35027
229 => {
vm_block = 23; continue;
}
// C line 35024
230 => {
vm_block = 20; continue;
}
// C line 35023
231 => {
let _ = { let assigned = (cc).line_num; line_num = assigned; assigned };
vm_block = 230; continue;
}
// C line 35023
232 => {
vm_block = if (((((cc).line_num) >= ((0 as i32))) as i32)) != 0 { 231 } else { 230 }; continue;
}
// C line 35022
233 => {
vm_block = if (code_match(core::ptr::addr_of_mut!(cc), pos_next, &[((OP_return_undef as i32)) as i32, (((1 as i32)).wrapping_neg()) as i32])) != 0 { 232 } else { 229 }; continue;
}
// C line 35020
234 => {
vm_block = if ((1 as i32)) != 0 { 233 } else { 229 }; continue;
}
// C line 35017
235 => {
vm_block = 20; continue;
}
// C line 35015
236 => {
let _ = dbuf_putc(core::ptr::addr_of_mut!(bc_out), ((is_with) as u8));
vm_block = 235; continue;
}
// C line 35013
237 => {
vm_block = 2; continue;
}
// C line 35012
238 => {
vm_block = if ((!(!(add_reloc(ctx, ls, ((((bc_out).size).wrapping_sub((((4 as i32)) as usize))) as u32), (4 as i32))).is_null()) as i32)) != 0 { 237 } else { 236 }; continue;
}
// C line 35010
239 => {
vm_block = if (((((*(ls)).addr) == (((1 as i32)).wrapping_neg())) as i32)) != 0 { 238 } else { 236 }; continue;
}
// C line 35009
240 => {
let _ = dbuf_put_u32(core::ptr::addr_of_mut!(bc_out), ((((((*(ls)).addr) as usize)).wrapping_sub((bc_out).size)) as u32));
vm_block = 239; continue;
}
// C line 35008
241 => {
let _ = dbuf_put_u32(core::ptr::addr_of_mut!(bc_out), atom);
vm_block = 240; continue;
}
// C line 35007
242 => {
let _ = dbuf_putc(core::ptr::addr_of_mut!(bc_out), ((op) as u8));
vm_block = 241; continue;
}
// C line 34999
243 => {
let _ = add_pc2line_info(s, (((bc_out).size) as u32), ((line_num) as u32));
vm_block = 242; continue;
}
// C line 34998
244 => {
let _ = { let assigned = core::ptr::addr_of_mut!(*(label_slots).offset((label) as isize)); ls = assigned; assigned };
vm_block = 243; continue;
}
// C line 34997
245 => {
let _ = if ((((!(((((((((label) >= ((0 as i32))) as i32)) != 0) && (((((label) < ((*(s)).label_count)) as i32)) != 0)) as i32)) != 0) as i32)) as i64)) != 0 { std::process::abort() } else { { let _ = (0 as i32); } };
vm_block = 244; continue;
}
// C line 34995
246 => {
let _ = { let assigned = find_jump_target(s, label, core::ptr::addr_of_mut!(op1), core::ptr::null_mut::<i32>()); label = assigned; assigned };
vm_block = 245; continue;
}
// C line 34994
247 => {
vm_block = if ((1 as i32)) != 0 { 246 } else { 245 }; continue;
}
// C line 34993
248 => {
let _ = { let assigned = ((*(bc_buf).offset(((pos).wrapping_add((9 as i32))) as isize)) as i32); is_with = assigned; assigned };
vm_block = 247; continue;
}
// C line 34992
249 => {
let _ = { let assigned = ((get_u32(((bc_buf).offset(((pos) as isize))).offset((((5 as i32)) as isize)))) as i32); label = assigned; assigned };
vm_block = 248; continue;
}
// C line 34991
250 => {
let _ = { let assigned = get_u32(((bc_buf).offset(((pos) as isize))).offset((((1 as i32)) as isize))); atom = assigned; assigned };
vm_block = 249; continue;
}
// C line 34981
251 => {
vm_block = 20; continue;
}
// C line 34979
252 => {
vm_block = 2; continue;
}
// C line 34978
253 => {
vm_block = if ((!(!(add_reloc(ctx, ls, ((((bc_out).size).wrapping_sub((((4 as i32)) as usize))) as u32), (4 as i32))).is_null()) as i32)) != 0 { 252 } else { 251 }; continue;
}
// C line 34976
254 => {
vm_block = if (((((*(ls)).addr) == (((1 as i32)).wrapping_neg())) as i32)) != 0 { 253 } else { 251 }; continue;
}
// C line 34975
255 => {
let _ = dbuf_put_u32(core::ptr::addr_of_mut!(bc_out), ((((((*(ls)).addr) as usize)).wrapping_sub((bc_out).size)) as u32));
vm_block = 254; continue;
}
// C line 34974
256 => {
let _ = dbuf_putc(core::ptr::addr_of_mut!(bc_out), ((op) as u8));
vm_block = 255; continue;
}
// C line 34928
257 => {
let _ = { let assigned = core::ptr::addr_of_mut!(*(label_slots).offset((label) as isize)); ls = assigned; assigned };
vm_block = 256; continue;
}
// C line 34927
258 => {
let _ = if ((((!(((((((((label) >= ((0 as i32))) as i32)) != 0) && (((((label) < ((*(s)).label_count)) as i32)) != 0)) as i32)) != 0) as i32)) as i64)) != 0 { std::process::abort() } else { { let _ = (0 as i32); } };
vm_block = 257; continue;
}
// C line 34925
259 => {
let _ = { let assigned = skip_dead_code(s, bc_buf, bc_len, pos_next, core::ptr::addr_of_mut!(line_num)); pos_next = assigned; assigned };
vm_block = 258; continue;
}
// C line 34924
260 => {
vm_block = if ((((op) == ((OP_goto as i32))) as i32)) != 0 { 259 } else { 258 }; continue;
}
// C line 34923 labels: has_label
261 => {
let _ = add_pc2line_info(s, (((bc_out).size) as u32), ((line_num) as u32));
vm_block = 260; continue;
}
// C line 34918
262 => {
let _ = { op = (((((op) as i32)) ^ ((((OP_if_true as i32)) ^ ((OP_if_false as i32)))))) as i32; op };
vm_block = 261; continue;
}
// C line 34917
263 => {
let _ = { let assigned = (cc).label; label = assigned; assigned };
vm_block = 262; continue;
}
// C line 34916
264 => {
let _ = update_label(s, label, ((1 as i32)).wrapping_neg());
vm_block = 263; continue;
}
// C line 34915
265 => {
let _ = { let assigned = pos1; pos_next = assigned; assigned };
vm_block = 264; continue;
}
// C line 34914
266 => {
let _ = { let assigned = line1_1; line_num = assigned; assigned };
vm_block = 265; continue;
}
// C line 34914
267 => {
vm_block = if ((((line1_1) != (((1 as i32)).wrapping_neg())) as i32)) != 0 { 266 } else { 265 }; continue;
}
// C line 34913
268 => {
vm_block = if (code_has_label(core::ptr::addr_of_mut!(cc), pos1, label)) != 0 { 267 } else { 261 }; continue;
}
// C line 34912
269 => {
line1_1 = (cc).line_num;
vm_block = 268; continue;
}
// C line 34911
270 => {
pos1 = (cc).pos;
vm_block = 269; continue;
}
// C line 34910
271 => {
vm_block = if (code_match(core::ptr::addr_of_mut!(cc), pos_next, &[((OP_goto as i32)) as i32, (((1 as i32)).wrapping_neg()) as i32])) != 0 { 270 } else { 261 }; continue;
}
// C line 34907
272 => {
vm_block = 20; continue;
}
// C line 34906
273 => {
let _ = dbuf_putc(core::ptr::addr_of_mut!(bc_out), (((OP_drop as i32)) as u8));
vm_block = 272; continue;
}
// C line 34905
274 => {
let _ = update_label(s, label, ((1 as i32)).wrapping_neg());
vm_block = 273; continue;
}
// C line 34904
275 => {
vm_block = if (code_has_label(core::ptr::addr_of_mut!(cc), pos_next, label)) != 0 { 274 } else { 271 }; continue;
}
// C line 34902
276 => {
let _ = { let assigned = find_jump_target(s, label, core::ptr::addr_of_mut!(op1), core::ptr::null_mut::<i32>()); label = assigned; assigned };
vm_block = 275; continue;
}
// C line 34901
277 => {
vm_block = if ((1 as i32)) != 0 { 276 } else { 261 }; continue;
}
// C line 34900
278 => {
let _ = { let assigned = ((get_u32(((bc_buf).offset(((pos) as isize))).offset((((1 as i32)) as isize)))) as i32); label = assigned; assigned };
vm_block = 277; continue;
}
// C line 34896
279 => {
vm_block = 261; continue;
}
// C line 34895
280 => {
let _ = { let assigned = ((get_u32(((bc_buf).offset(((pos) as isize))).offset((((1 as i32)) as isize)))) as i32); label = assigned; assigned };
vm_block = 279; continue;
}
// C line 34892
281 => {
vm_block = 261; continue;
}
// C line 34889
282 => {
vm_block = 20; continue;
}
// C line 34887
283 => {
let _ = update_label(s, label, ((1 as i32)).wrapping_neg());
vm_block = 282; continue;
}
// C line 34886
284 => {
vm_block = if ((((op1) == ((OP_ret as i32))) as i32)) != 0 { 283 } else { 281 }; continue;
}
// C line 34885
285 => {
let _ = { let assigned = find_jump_target(s, label, core::ptr::addr_of_mut!(op1), core::ptr::null_mut::<i32>()); label = assigned; assigned };
vm_block = 284; continue;
}
// C line 34884
286 => {
vm_block = if ((((((0 as i32)) != 0) && (((1 as i32)) != 0)) as i32)) != 0 { 285 } else { 281 }; continue;
}
// C line 34883
287 => {
let _ = { let assigned = ((get_u32(((bc_buf).offset(((pos) as isize))).offset((((1 as i32)) as isize)))) as i32); label = assigned; assigned };
vm_block = 286; continue;
}
// C line 34880
288 => {
vm_block = 261; continue;
}
// C line 34871
289 => {
vm_block = 20; continue;
}
// C line 34870
290 => {
let _ = { let assigned = skip_dead_code(s, bc_buf, bc_len, pos_next, core::ptr::addr_of_mut!(line_num)); pos_next = assigned; assigned };
vm_block = 289; continue;
}
// C line 34869
291 => {
let _ = dbuf_putc(core::ptr::addr_of_mut!(bc_out), ((op1) as u8));
vm_block = 290; continue;
}
// C line 34868
292 => {
let _ = add_pc2line_info(s, (((bc_out).size) as u32), ((line_num) as u32));
vm_block = 291; continue;
}
// C line 34867
293 => {
let _ = update_label(s, label, ((1 as i32)).wrapping_neg());
vm_block = 292; continue;
}
// C line 34863
294 => {
vm_block = if ((((((((((((op1) == ((OP_return as i32))) as i32)) != 0) || (((((op1) == ((OP_return_undef as i32))) as i32)) != 0)) as i32)) != 0) || (((((op1) == ((OP_throw as i32))) as i32)) != 0)) as i32)) != 0 { 293 } else { 288 }; continue;
}
// C line 34861
295 => {
vm_block = 20; continue;
}
// C line 34860
296 => {
let _ = update_label(s, label, ((1 as i32)).wrapping_neg());
vm_block = 295; continue;
}
// C line 34858
297 => {
vm_block = if (code_has_label(core::ptr::addr_of_mut!(cc), pos_next, label)) != 0 { 296 } else { 294 }; continue;
}
// C line 34857
298 => {
let _ = { let assigned = find_jump_target(s, label, core::ptr::addr_of_mut!(op1), core::ptr::addr_of_mut!(line1)); label = assigned; assigned };
vm_block = 297; continue;
}
// C line 34855
299 => {
line1 = ((1 as i32)).wrapping_neg();
vm_block = 298; continue;
}
// C line 34854 labels: has_goto
300 => {
vm_block = if ((1 as i32)) != 0 { 299 } else { 288 }; continue;
}
// C line 34852
301 => {
let _ = { let assigned = ((get_u32(((bc_buf).offset(((pos) as isize))).offset((((1 as i32)) as isize)))) as i32); label = assigned; assigned };
vm_block = 300; continue;
}
// C line 34849
302 => {
vm_block = 23; continue;
}
// C line 34848
303 => {
let _ = { let assigned = skip_dead_code(s, bc_buf, bc_len, pos_next, core::ptr::addr_of_mut!(line_num)); pos_next = assigned; assigned };
vm_block = 302; continue;
}
// C line 34841
304 => {
vm_block = 23; continue;
}
// C line 34839
305 => {
vm_block = 20; continue;
}
// C line 34838
306 => {
let _ = put_short_code(core::ptr::addr_of_mut!(bc_out), op, argc);
vm_block = 305; continue;
}
// C line 34837
307 => {
let _ = add_pc2line_info(s, (((bc_out).size) as u32), ((line_num) as u32));
vm_block = 306; continue;
}
// C line 34835
308 => {
vm_block = 20; continue;
}
// C line 34834
309 => {
let _ = { let assigned = skip_dead_code(s, bc_buf, bc_len, (cc).pos, core::ptr::addr_of_mut!(line_num)); pos_next = assigned; assigned };
vm_block = 308; continue;
}
// C line 34833
310 => {
let _ = put_short_code(core::ptr::addr_of_mut!(bc_out), (op).wrapping_add((1 as i32)), argc);
vm_block = 309; continue;
}
// C line 34832
311 => {
let _ = add_pc2line_info(s, (((bc_out).size) as u32), ((line_num) as u32));
vm_block = 310; continue;
}
// C line 34831
312 => {
let _ = { let assigned = (cc).line_num; line_num = assigned; assigned };
vm_block = 311; continue;
}
// C line 34831
313 => {
vm_block = if (((((cc).line_num) >= ((0 as i32))) as i32)) != 0 { 312 } else { 311 }; continue;
}
// C line 34830
314 => {
vm_block = if (code_match(core::ptr::addr_of_mut!(cc), pos_next, &[((OP_return as i32)) as i32, (((1 as i32)).wrapping_neg()) as i32])) != 0 { 313 } else { 307 }; continue;
}
// C line 34829
315 => {
let _ = { let assigned = ((get_u16(((bc_buf).offset(((pos) as isize))).offset((((1 as i32)) as isize)))) as i32); argc = assigned; assigned };
vm_block = 314; continue;
}
// C line 34822
316 => {
vm_block = 20; continue;
}
// C line 34820
317 => {
let _ = { let assigned = core::ptr::null_mut::<RelocEntry>(); (*(ls)).first_reloc = assigned; assigned };
vm_block = 316; continue;
}
// C line 34802
318 => {
vm_block = if ((((re) != (core::ptr::null_mut::<RelocEntry>())) as i32)) != 0 { 331 } else { 317 }; continue;
}
// C line 34802
319 => {
let _ = { let assigned = re_next; re = assigned; assigned };
vm_block = 318; continue;
}
// C line 34818
320 => {
let _ = js_free(ctx, ((re) as *mut c_void));
vm_block = 319; continue;
}
// C line 34816
321 => {
vm_block = 320; continue;
}
// C line 34815
322 => {
let _ = crate::cutils_header::put_u8(((bc_out).buf).offset((((*(re)).addr) as isize)), ((diff) as u8));
vm_block = 321; continue;
}
// C line 34814
323 => {
let _ = if ((((!(((((diff) == (((((diff) as i8)) as i32))) as i32)) != 0) as i32)) as i64)) != 0 { std::process::abort() } else { { let _ = (0 as i32); } };
vm_block = 322; continue;
}
// C line 34812
324 => {
vm_block = 320; continue;
}
// C line 34811
325 => {
let _ = crate::cutils_header::put_u16(((bc_out).buf).offset((((*(re)).addr) as isize)), ((diff) as u16));
vm_block = 324; continue;
}
// C line 34810
326 => {
let _ = if ((((!(((((diff) == (((((diff) as i16)) as i32))) as i32)) != 0) as i32)) as i64)) != 0 { std::process::abort() } else { { let _ = (0 as i32); } };
vm_block = 325; continue;
}
// C line 34808
327 => {
vm_block = 320; continue;
}
// C line 34807
328 => {
let _ = put_u32(((bc_out).buf).offset((((*(re)).addr) as isize)), ((diff) as u32));
vm_block = 327; continue;
}
// C line 34805
329 => {
vm_block = match (*(re)).size { x if x == (1 as i32) => 323, x if x == (2 as i32) => 326, x if x == (4 as i32) => 328, _ => 320, }; continue;
}
// C line 34804
330 => {
let _ = { let assigned = (*(re)).next; re_next = assigned; assigned };
vm_block = 329; continue;
}
// C line 34803
331 => {
diff = ((((((*(ls)).addr) as u32)).wrapping_sub((*(re)).addr)) as i32);
vm_block = 330; continue;
}
// C line 34802
332 => {
let _ = { let assigned = (*(ls)).first_reloc; re = assigned; assigned };
vm_block = 318; continue;
}
// C line 34800
333 => {
let _ = { let assigned = (((bc_out).size) as i32); (*(ls)).addr = assigned; assigned };
vm_block = 332; continue;
}
// C line 34799
334 => {
let _ = if ((((!((((((*(ls)).addr) == (((1 as i32)).wrapping_neg())) as i32)) != 0) as i32)) as i64)) != 0 { std::process::abort() } else { { let _ = (0 as i32); } };
vm_block = 333; continue;
}
// C line 34798
335 => {
let _ = { let assigned = core::ptr::addr_of_mut!(*(label_slots).offset((label) as isize)); ls = assigned; assigned };
vm_block = 334; continue;
}
// C line 34797
336 => {
let _ = if ((((!(((((((((label) >= ((0 as i32))) as i32)) != 0) && (((((label) < ((*(s)).label_count)) as i32)) != 0)) as i32)) != 0) as i32)) as i64)) != 0 { std::process::abort() } else { { let _ = (0 as i32); } };
vm_block = 335; continue;
}
// C line 34796
337 => {
let _ = { let assigned = ((get_u32(((bc_buf).offset(((pos) as isize))).offset((((1 as i32)) as isize)))) as i32); label = assigned; assigned };
vm_block = 336; continue;
}
// C line 34792
338 => {
vm_block = 20; continue;
}
// C line 34791
339 => {
let _ = { let assigned = ((get_u32(((bc_buf).offset(((pos) as isize))).offset((((1 as i32)) as isize)))) as i32); line_num = assigned; assigned };
vm_block = 338; continue;
}
// C line 34786
340 => {
vm_block = match op { x if x == (OP_post_dec as i32) => 57, x if x == (OP_post_inc as i32) => 57, x if x == (OP_put_var_ref as i32) => 70, x if x == (OP_put_arg as i32) => 70, x if x == (OP_put_loc_check as i32) => 70, x if x == (OP_put_loc as i32) => 70, x if x == (OP_get_loc as i32) => 114, x if x == (OP_dup as i32) => 136, x if x == (OP_insert2 as i32) => 146, x if x == (OP_undefined as i32) => 163, x if x == (OP_to_propkey as i32) => 167, x if x == (OP_push_atom_value as i32) => 176, x if x == (OP_push_bigint_i32 as i32) => 190, x if x == (OP_push_i32 as i32) => 214, x if x == (OP_push_true as i32) => 228, x if x == (OP_push_false as i32) => 228, x if x == (OP_null as i32) => 228, x if x == (OP_drop as i32) => 234, x if x == (OP_with_get_ref as i32) => 250, x if x == (OP_with_make_ref as i32) => 250, x if x == (OP_with_delete_var as i32) => 250, x if x == (OP_with_put_var as i32) => 250, x if x == (OP_with_get_var as i32) => 250, x if x == (OP_if_false as i32) => 278, x if x == (OP_if_true as i32) => 278, x if x == (OP_catch as i32) => 280, x if x == (OP_gosub as i32) => 287, x if x == (OP_goto as i32) => 301, x if x == (OP_throw_error as i32) => 303, x if x == (OP_throw as i32) => 303, x if x == (OP_return_async as i32) => 303, x if x == (OP_return_undef as i32) => 303, x if x == (OP_return as i32) => 303, x if x == (OP_call_method as i32) => 315, x if x == (OP_call as i32) => 315, x if x == (OP_label as i32) => 337, x if x == (OP_line_num as i32) => 339, _ => 23, }; continue;
}
// C line 34785
341 => {
let _ = { let assigned = (pos).wrapping_add(len); pos_next = assigned; assigned };
vm_block = 340; continue;
}
// C line 34784
342 => {
let _ = { let assigned = (((*((opcode_info).as_ptr()).offset((op) as isize)).size) as i32); len = assigned; assigned };
vm_block = 341; continue;
}
// C line 34783
343 => {
let _ = { let assigned = ((*(bc_buf).offset((pos) as isize)) as i32); op = assigned; assigned };
vm_block = 342; continue;
}
// C line 34781
344 => {
let _ = { let assigned = (0 as i32); pos = assigned; assigned };
vm_block = 19; continue;
}
// C line 34778
345 => {
let _ = put_short_code(core::ptr::addr_of_mut!(bc_out), (OP_put_loc as i32), (*(s)).arg_var_object_idx);
vm_block = 344; continue;
}
// C line 34777
346 => {
let _ = dbuf_putc(core::ptr::addr_of_mut!(bc_out), (((OP_SPECIAL_OBJECT_VAR_OBJECT as i32)) as u8));
vm_block = 345; continue;
}
// C line 34776
347 => {
let _ = dbuf_putc(core::ptr::addr_of_mut!(bc_out), (((OP_special_object as i32)) as u8));
vm_block = 346; continue;
}
// C line 34775
348 => {
vm_block = if (((((*(s)).arg_var_object_idx) >= ((0 as i32))) as i32)) != 0 { 347 } else { 344 }; continue;
}
// C line 34773
349 => {
let _ = put_short_code(core::ptr::addr_of_mut!(bc_out), (OP_put_loc as i32), (*(s)).var_object_idx);
vm_block = 348; continue;
}
// C line 34772
350 => {
let _ = dbuf_putc(core::ptr::addr_of_mut!(bc_out), (((OP_SPECIAL_OBJECT_VAR_OBJECT as i32)) as u8));
vm_block = 349; continue;
}
// C line 34771
351 => {
let _ = dbuf_putc(core::ptr::addr_of_mut!(bc_out), (((OP_special_object as i32)) as u8));
vm_block = 350; continue;
}
// C line 34770
352 => {
vm_block = if (((((*(s)).var_object_idx) >= ((0 as i32))) as i32)) != 0 { 351 } else { 348 }; continue;
}
// C line 34767
353 => {
let _ = put_short_code(core::ptr::addr_of_mut!(bc_out), (OP_put_loc as i32), (*(s)).func_var_idx);
vm_block = 352; continue;
}
// C line 34766
354 => {
let _ = dbuf_putc(core::ptr::addr_of_mut!(bc_out), (((OP_SPECIAL_OBJECT_THIS_FUNC as i32)) as u8));
vm_block = 353; continue;
}
// C line 34765
355 => {
let _ = dbuf_putc(core::ptr::addr_of_mut!(bc_out), (((OP_special_object as i32)) as u8));
vm_block = 354; continue;
}
// C line 34764
356 => {
vm_block = if (((((*(s)).func_var_idx) >= ((0 as i32))) as i32)) != 0 { 355 } else { 352 }; continue;
}
// C line 34761
357 => {
let _ = put_short_code(core::ptr::addr_of_mut!(bc_out), (OP_put_loc as i32), (*(s)).arguments_var_idx);
vm_block = 356; continue;
}
// C line 34760
358 => {
let _ = put_short_code(core::ptr::addr_of_mut!(bc_out), (OP_set_loc as i32), (*(s)).arguments_arg_idx);
vm_block = 357; continue;
}
// C line 34759
359 => {
vm_block = if (((((*(s)).arguments_arg_idx) >= ((0 as i32))) as i32)) != 0 { 358 } else { 357 }; continue;
}
// C line 34749
360 => {
let _ = dbuf_putc(core::ptr::addr_of_mut!(bc_out), (((OP_SPECIAL_OBJECT_ARGUMENTS as i32)) as u8));
vm_block = 359; continue;
}
// C line 34748
361 => {
let _ = dbuf_putc(core::ptr::addr_of_mut!(bc_out), (((OP_special_object as i32)) as u8));
vm_block = 360; continue;
}
// C line 34756
362 => {
vm_block = if ((((i) < ((*(s)).arg_count)) as i32)) != 0 { 364 } else { 359 }; continue;
}
// C line 34756
363 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 362; continue;
}
// C line 34757
364 => {
let _ = capture_var(s, core::ptr::addr_of_mut!(*((*(s)).args).offset((i) as isize)));
vm_block = 363; continue;
}
// C line 34756
365 => {
let _ = { let assigned = (0 as i32); i = assigned; assigned };
vm_block = 362; continue;
}
// C line 34752
366 => {
let _ = dbuf_putc(core::ptr::addr_of_mut!(bc_out), (((OP_SPECIAL_OBJECT_MAPPED_ARGUMENTS as i32)) as u8));
vm_block = 365; continue;
}
// C line 34751
367 => {
let _ = dbuf_putc(core::ptr::addr_of_mut!(bc_out), (((OP_special_object as i32)) as u8));
vm_block = 366; continue;
}
// C line 34747
368 => {
vm_block = if ((((((((((*(s)).js_mode) as i32)) & (((1 as i32)).wrapping_shl(((0 as i32)) as u32)))) != 0) || (((!(((*(s)).has_simple_parameter_list) != 0) as i32)) != 0)) as i32)) != 0 { 361 } else { 367 }; continue;
}
// C line 34746
369 => {
vm_block = if (((((*(s)).arguments_var_idx) >= ((0 as i32))) as i32)) != 0 { 368 } else { 356 }; continue;
}
// C line 34739
370 => {
let _ = dbuf_put_u16(core::ptr::addr_of_mut!(bc_out), (((*(s)).this_var_idx) as u16));
vm_block = 369; continue;
}
// C line 34738
371 => {
let _ = dbuf_putc(core::ptr::addr_of_mut!(bc_out), (((OP_set_loc_uninitialized as i32)) as u8));
vm_block = 370; continue;
}
// C line 34742
372 => {
let _ = put_short_code(core::ptr::addr_of_mut!(bc_out), (OP_put_loc as i32), (*(s)).this_var_idx);
vm_block = 369; continue;
}
// C line 34741
373 => {
let _ = dbuf_putc(core::ptr::addr_of_mut!(bc_out), (((OP_push_this as i32)) as u8));
vm_block = 372; continue;
}
// C line 34737
374 => {
vm_block = if ((*(s)).is_derived_class_constructor) != 0 { 371 } else { 373 }; continue;
}
// C line 34736
375 => {
vm_block = if (((((*(s)).this_var_idx) >= ((0 as i32))) as i32)) != 0 { 374 } else { 369 }; continue;
}
// C line 34732
376 => {
let _ = put_short_code(core::ptr::addr_of_mut!(bc_out), (OP_put_loc as i32), (*(s)).new_target_var_idx);
vm_block = 375; continue;
}
// C line 34731
377 => {
let _ = dbuf_putc(core::ptr::addr_of_mut!(bc_out), (((OP_SPECIAL_OBJECT_NEW_TARGET as i32)) as u8));
vm_block = 376; continue;
}
// C line 34730
378 => {
let _ = dbuf_putc(core::ptr::addr_of_mut!(bc_out), (((OP_special_object as i32)) as u8));
vm_block = 377; continue;
}
// C line 34729
379 => {
vm_block = if (((((*(s)).new_target_var_idx) >= ((0 as i32))) as i32)) != 0 { 378 } else { 375 }; continue;
}
// C line 34726
380 => {
let _ = put_short_code(core::ptr::addr_of_mut!(bc_out), (OP_put_loc as i32), (*(s)).this_active_func_var_idx);
vm_block = 379; continue;
}
// C line 34725
381 => {
let _ = dbuf_putc(core::ptr::addr_of_mut!(bc_out), (((OP_SPECIAL_OBJECT_THIS_FUNC as i32)) as u8));
vm_block = 380; continue;
}
// C line 34724
382 => {
let _ = dbuf_putc(core::ptr::addr_of_mut!(bc_out), (((OP_special_object as i32)) as u8));
vm_block = 381; continue;
}
// C line 34723
383 => {
vm_block = if (((((*(s)).this_active_func_var_idx) >= ((0 as i32))) as i32)) != 0 { 382 } else { 379 }; continue;
}
// C line 34720
384 => {
let _ = put_short_code(core::ptr::addr_of_mut!(bc_out), (OP_put_loc as i32), (*(s)).home_object_var_idx);
vm_block = 383; continue;
}
// C line 34719
385 => {
let _ = dbuf_putc(core::ptr::addr_of_mut!(bc_out), (((OP_SPECIAL_OBJECT_HOME_OBJECT as i32)) as u8));
vm_block = 384; continue;
}
// C line 34718
386 => {
let _ = dbuf_putc(core::ptr::addr_of_mut!(bc_out), (((OP_special_object as i32)) as u8));
vm_block = 385; continue;
}
// C line 34717
387 => {
vm_block = if (((((*(s)).home_object_var_idx) >= ((0 as i32))) as i32)) != 0 { 386 } else { 383 }; continue;
}
// C line 34713
388 => {
let _ = { let assigned = (0 as i32); (*(s)).line_number_last_pc = assigned; assigned };
vm_block = 387; continue;
}
// C line 34712
389 => {
let _ = { let assigned = (((*(s)).source_pos) as i32); (*(s)).line_number_last = assigned; assigned };
vm_block = 388; continue;
}
// C line 34711
390 => {
return ((1 as i32)).wrapping_neg();
}
// C line 34710
391 => {
vm_block = if (((((*(s)).line_number_slots) == (core::ptr::null_mut::<LineNumberSlot>())) as i32)) != 0 { 390 } else { 389 }; continue;
}
// C line 34709
392 => {
let _ = { let assigned = ((js_mallocz((*(s)).ctx, ((size_of::<LineNumberSlot>() as usize)).wrapping_mul((((*(s)).line_number_size) as usize)))) as *mut LineNumberSlot); (*(s)).line_number_slots = assigned; assigned };
vm_block = 391; continue;
}
// C line 34708
393 => {
vm_block = if ((((((*(s)).line_number_size) != 0) && (((!(((*(s)).strip_debug()) != 0) as i32)) != 0)) as i32)) != 0 { 392 } else { 387 }; continue;
}
// C line 34698
394 => {
let _ = js_dbuf_bytecode_init(ctx, core::ptr::addr_of_mut!(bc_out));
vm_block = 393; continue;
}
// C line 34697
395 => {
let _ = { let assigned = { let assigned = ((((*(s)).byte_code).size) as i32); bc_len = assigned; assigned }; (cc).bc_len = assigned; assigned };
vm_block = 394; continue;
}
// C line 34696
396 => {
let _ = { let assigned = { let assigned = ((*(s)).byte_code).buf; bc_buf = assigned; assigned }; (cc).bc_buf = assigned; assigned };
vm_block = 395; continue;
}
// C line 34694
397 => {
let _ = { let assigned = (((*(s)).source_pos) as i32); line_num = assigned; assigned };
vm_block = 396; continue;
}
// C line 34692
398 => {
let _ = { let assigned = (*(s)).label_slots; label_slots = assigned; assigned };
vm_block = 397; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:35595. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn ss_check(mut ctx: *mut JSContext, mut s: *mut StackSizeState, mut pos: i32, mut op: i32, mut stack_len: i32, mut catch_pos: i32) -> i32 {
let mut vm_local_storage = Vec::<u64>::new();
let mut vm_block: usize = 22;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 35633
1 => {
return (0 as i32);
}
// C line 35632
2 => {
let _ = { let assigned = pos; *((*(s)).pc_stack).offset(({ let old = (*(s)).pc_stack_len; (*(s)).pc_stack_len = ((*(s)).pc_stack_len).wrapping_add(1); old }) as isize) = assigned; assigned };
vm_block = 1; continue;
}
// C line 35631
3 => {
return ((1 as i32)).wrapping_neg();
}
// C line 35629
4 => {
vm_block = if (js_resize_array(ctx, ((core::ptr::addr_of_mut!((*(s)).pc_stack)) as *mut *mut c_void), (((size_of::<i32>() as usize)) as i32), core::ptr::addr_of_mut!((*(s)).pc_stack_size), ((*(s)).pc_stack_len).wrapping_add((1 as i32)))) != 0 { 3 } else { 2 }; continue;
}
// C line 35626
5 => {
let _ = { let assigned = catch_pos; *((*(s)).catch_pos_tab).offset((pos) as isize) = assigned; assigned };
vm_block = 4; continue;
}
// C line 35625
6 => {
let _ = { let assigned = ((stack_len) as u16); *((*(s)).stack_level_tab).offset((pos) as isize) = assigned; assigned };
vm_block = 5; continue;
}
// C line 35614
7 => {
return ((1 as i32)).wrapping_neg();
}
// C line 35612
8 => {
let _ = JS_ThrowInternalError_cargs(ctx, c"inconsistent stack size: %d %d (pc=%d)".as_ptr(), &[ParserFormatArg::Signed((((*((*(s)).stack_level_tab).offset((pos) as isize)) as i32)) as i32), ParserFormatArg::Signed((stack_len) as i32), ParserFormatArg::Signed((pos) as i32)]);
vm_block = 7; continue;
}
// C line 35618
9 => {
return ((1 as i32)).wrapping_neg();
}
// C line 35616
10 => {
let _ = JS_ThrowInternalError_cargs(ctx, c"inconsistent catch position: %d %d (pc=%d)".as_ptr(), &[ParserFormatArg::Signed((*((*(s)).catch_pos_tab).offset((pos) as isize)) as i32), ParserFormatArg::Signed((catch_pos) as i32), ParserFormatArg::Signed((pos) as i32)]);
vm_block = 9; continue;
}
// C line 35620
11 => {
return (0 as i32);
}
// C line 35615
12 => {
vm_block = if ((((*((*(s)).catch_pos_tab).offset((pos) as isize)) != (catch_pos)) as i32)) != 0 { 10 } else { 11 }; continue;
}
// C line 35611
13 => {
vm_block = if ((((((*((*(s)).stack_level_tab).offset((pos) as isize)) as i32)) != (stack_len)) as i32)) != 0 { 8 } else { 12 }; continue;
}
// C line 35609
14 => {
vm_block = if ((((((*((*(s)).stack_level_tab).offset((pos) as isize)) as i32)) != ((65535 as i32))) as i32)) != 0 { 13 } else { 6 }; continue;
}
// C line 35606
15 => {
return ((1 as i32)).wrapping_neg();
}
// C line 35605
16 => {
let _ = JS_ThrowInternalError_cargs(ctx, c"stack overflow (op=%d, pc=%d)".as_ptr(), &[ParserFormatArg::Signed((op) as i32), ParserFormatArg::Signed((pos) as i32)]);
vm_block = 15; continue;
}
// C line 35604
17 => {
vm_block = if (((((*(s)).stack_len_max) > ((65534 as i32))) as i32)) != 0 { 16 } else { 14 }; continue;
}
// C line 35603
18 => {
let _ = { let assigned = stack_len; (*(s)).stack_len_max = assigned; assigned };
vm_block = 17; continue;
}
// C line 35602
19 => {
vm_block = if ((((stack_len) > ((*(s)).stack_len_max)) as i32)) != 0 { 18 } else { 14 }; continue;
}
// C line 35600
20 => {
return ((1 as i32)).wrapping_neg();
}
// C line 35599
21 => {
let _ = JS_ThrowInternalError_cargs(ctx, c"bytecode buffer overflow (op=%d, pc=%d)".as_ptr(), &[ParserFormatArg::Signed((op) as i32), ParserFormatArg::Signed((pos) as i32)]);
vm_block = 20; continue;
}
// C line 35598
22 => {
vm_block = if ((((((pos) as u32)) >= ((((*(s)).bc_len) as u32))) as i32)) != 0 { 21 } else { 19 }; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:35636. Bellard/Gordon MIT.
#[cfg(feature = "short-opcodes")]
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn compute_stack_size(mut ctx: *mut JSContext, mut fd: *mut JSFunctionDef, mut pstack_size: *mut i32) -> i32 {
let mut vm_local_storage = Vec::<u64>::new();
let mut s_s: StackSizeState = core::mem::zeroed();
let mut s: *mut StackSizeState = core::ptr::null_mut();
let mut i: i32 = 0;
let mut diff: i32 = 0;
let mut n_pop: i32 = 0;
let mut pos_next: i32 = 0;
let mut stack_len: i32 = 0;
let mut pos: i32 = 0;
let mut op: i32 = 0;
let mut catch_pos: i32 = 0;
let mut catch_level: i32 = 0;
let mut oi: *const JSOpCode = core::ptr::null();
let mut bc_buf: *const u8 = core::ptr::null();
let mut level: i32 = 0;
let mut vm_block: usize = 124;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 35834
1 => {
return ((1 as i32)).wrapping_neg();
}
// C line 35833
2 => {
let _ = { let assigned = (0 as i32); *(pstack_size) = assigned; assigned };
vm_block = 1; continue;
}
// C line 35832
3 => {
let _ = js_free(ctx, (((*(s)).stack_level_tab) as *mut c_void));
vm_block = 2; continue;
}
// C line 35831
4 => {
let _ = js_free(ctx, (((*(s)).catch_pos_tab) as *mut c_void));
vm_block = 3; continue;
}
// C line 35830 labels: fail
5 => {
let _ = js_free(ctx, (((*(s)).pc_stack) as *mut c_void));
vm_block = 4; continue;
}
// C line 35828
6 => {
return (0 as i32);
}
// C line 35827
7 => {
let _ = { let assigned = (*(s)).stack_len_max; *(pstack_size) = assigned; assigned };
vm_block = 6; continue;
}
// C line 35826
8 => {
let _ = js_free(ctx, (((*(s)).stack_level_tab) as *mut c_void));
vm_block = 7; continue;
}
// C line 35825
9 => {
let _ = js_free(ctx, (((*(s)).catch_pos_tab) as *mut c_void));
vm_block = 8; continue;
}
// C line 35824
10 => {
let _ = js_free(ctx, (((*(s)).pc_stack) as *mut c_void));
vm_block = 9; continue;
}
// C line 35668 labels: done_insn
11 => {
vm_block = if (((((*(s)).pc_stack_len) > ((0 as i32))) as i32)) != 0 { 105 } else { 10 }; continue;
}
// C line 35821
12 => {
vm_block = 5; continue;
}
// C line 35820
13 => {
vm_block = if (ss_check(ctx, s, pos_next, op, stack_len, catch_pos)) != 0 { 12 } else { 11 }; continue;
}
// C line 35818
14 => {
vm_block = 13; continue;
}
// C line 35816
15 => {
vm_block = 13; continue;
}
// C line 35815
16 => {
let _ = { let assigned = *((*(s)).catch_pos_tab).offset((catch_pos) as isize); catch_pos = assigned; assigned };
vm_block = 15; continue;
}
// C line 35814
17 => {
let _ = { let old = stack_len; stack_len = (stack_len).wrapping_add(1); old };
vm_block = 16; continue;
}
// C line 35813
18 => {
let _ = { let old = stack_len; stack_len = (stack_len).wrapping_add(1); old };
vm_block = 17; continue;
}
// C line 35812
19 => {
vm_block = if ((((((*(bc_buf).offset((catch_pos) as isize)) as i32)) != ((OP_catch as i32))) as i32)) != 0 { 18 } else { 17 }; continue;
}
// C line 35811
20 => {
let _ = { let assigned = ((*((*(s)).stack_level_tab).offset((catch_pos) as isize)) as i32); stack_len = assigned; assigned };
vm_block = 19; continue;
}
// C line 35809
21 => {
vm_block = 5; continue;
}
// C line 35808
22 => {
let _ = JS_ThrowInternalError_cargs(ctx, c"nip_catch: no catch op (pc=%d)".as_ptr(), &[ParserFormatArg::Signed((pos) as i32)]);
vm_block = 21; continue;
}
// C line 35807
23 => {
vm_block = if ((((catch_pos) < ((0 as i32))) as i32)) != 0 { 22 } else { 20 }; continue;
}
// C line 35805
24 => {
vm_block = 13; continue;
}
// C line 35802
25 => {
let _ = { let assigned = *((*(s)).catch_pos_tab).offset((catch_pos) as isize); catch_pos = assigned; assigned };
vm_block = 24; continue;
}
// C line 35801
26 => {
vm_block = if ((((catch_level) == (level)) as i32)) != 0 { 25 } else { 24 }; continue;
}
// C line 35799
27 => {
let _ = { let old = level; level = (level).wrapping_add(1); old };
vm_block = 26; continue;
}
// C line 35798
28 => {
vm_block = if ((((((*(bc_buf).offset((catch_pos) as isize)) as i32)) != ((OP_catch as i32))) as i32)) != 0 { 27 } else { 26 }; continue;
}
// C line 35797
29 => {
let _ = { let assigned = ((*((*(s)).stack_level_tab).offset((catch_pos) as isize)) as i32); level = assigned; assigned };
vm_block = 28; continue;
}
// C line 35795 labels: check_catch
30 => {
vm_block = if ((((catch_pos) >= ((0 as i32))) as i32)) != 0 { 29 } else { 24 }; continue;
}
// C line 35790
31 => {
let _ = { let assigned = (stack_len).wrapping_add((2 as i32)); catch_level = assigned; assigned };
vm_block = 30; continue;
}
// C line 35788
32 => {
vm_block = 30; continue;
}
// C line 35787
33 => {
let _ = { let assigned = (stack_len).wrapping_sub((1 as i32)); catch_level = assigned; assigned };
vm_block = 32; continue;
}
// C line 35785
34 => {
vm_block = 30; continue;
}
// C line 35784
35 => {
let _ = { let assigned = (stack_len).wrapping_sub((1 as i32)); catch_level = assigned; assigned };
vm_block = 34; continue;
}
// C line 35782
36 => {
vm_block = 30; continue;
}
// C line 35781
37 => {
let _ = { let assigned = stack_len; catch_level = assigned; assigned };
vm_block = 36; continue;
}
// C line 35777
38 => {
vm_block = 13; continue;
}
// C line 35776
39 => {
let _ = { let assigned = pos; catch_pos = assigned; assigned };
vm_block = 38; continue;
}
// C line 35773
40 => {
vm_block = 13; continue;
}
// C line 35772
41 => {
let _ = { let assigned = pos; catch_pos = assigned; assigned };
vm_block = 40; continue;
}
// C line 35771
42 => {
vm_block = 5; continue;
}
// C line 35770
43 => {
vm_block = if (ss_check(ctx, s, ((pos).wrapping_add((1 as i32))).wrapping_add(diff), op, stack_len, catch_pos)) != 0 { 42 } else { 41 }; continue;
}
// C line 35769
44 => {
let _ = { let assigned = ((get_u32(((bc_buf).offset(((pos) as isize))).offset((((1 as i32)) as isize)))) as i32); diff = assigned; assigned };
vm_block = 43; continue;
}
// C line 35767
45 => {
vm_block = 13; continue;
}
// C line 35766
46 => {
vm_block = 5; continue;
}
// C line 35765
47 => {
vm_block = if (ss_check(ctx, s, ((pos).wrapping_add((5 as i32))).wrapping_add(diff), op, (stack_len).wrapping_sub((1 as i32)), catch_pos)) != 0 { 46 } else { 45 }; continue;
}
// C line 35764
48 => {
let _ = { let assigned = ((get_u32(((bc_buf).offset(((pos) as isize))).offset((((5 as i32)) as isize)))) as i32); diff = assigned; assigned };
vm_block = 47; continue;
}
// C line 35762
49 => {
vm_block = 13; continue;
}
// C line 35761
50 => {
vm_block = 5; continue;
}
// C line 35760
51 => {
vm_block = if (ss_check(ctx, s, ((pos).wrapping_add((5 as i32))).wrapping_add(diff), op, (stack_len).wrapping_add((2 as i32)), catch_pos)) != 0 { 50 } else { 49 }; continue;
}
// C line 35759
52 => {
let _ = { let assigned = ((get_u32(((bc_buf).offset(((pos) as isize))).offset((((5 as i32)) as isize)))) as i32); diff = assigned; assigned };
vm_block = 51; continue;
}
// C line 35756
53 => {
vm_block = 13; continue;
}
// C line 35755
54 => {
vm_block = 5; continue;
}
// C line 35754
55 => {
vm_block = if (ss_check(ctx, s, ((pos).wrapping_add((5 as i32))).wrapping_add(diff), op, (stack_len).wrapping_add((1 as i32)), catch_pos)) != 0 { 54 } else { 53 }; continue;
}
// C line 35753
56 => {
let _ = { let assigned = ((get_u32(((bc_buf).offset(((pos) as isize))).offset((((5 as i32)) as isize)))) as i32); diff = assigned; assigned };
vm_block = 55; continue;
}
// C line 35750
57 => {
vm_block = 13; continue;
}
// C line 35749
58 => {
vm_block = 5; continue;
}
// C line 35748
59 => {
vm_block = if (ss_check(ctx, s, ((pos).wrapping_add((1 as i32))).wrapping_add(diff), op, (stack_len).wrapping_add((1 as i32)), catch_pos)) != 0 { 58 } else { 57 }; continue;
}
// C line 35747
60 => {
let _ = { let assigned = ((get_u32(((bc_buf).offset(((pos) as isize))).offset((((1 as i32)) as isize)))) as i32); diff = assigned; assigned };
vm_block = 59; continue;
}
// C line 35745
61 => {
vm_block = 13; continue;
}
// C line 35744
62 => {
vm_block = 5; continue;
}
// C line 35743
63 => {
vm_block = if (ss_check(ctx, s, ((pos).wrapping_add((1 as i32))).wrapping_add(diff), op, stack_len, catch_pos)) != 0 { 62 } else { 61 }; continue;
}
// C line 35742
64 => {
let _ = { let assigned = ((get_u32(((bc_buf).offset(((pos) as isize))).offset((((1 as i32)) as isize)))) as i32); diff = assigned; assigned };
vm_block = 63; continue;
}
// C line 35738
65 => {
vm_block = 13; continue;
}
// C line 35737
66 => {
vm_block = 5; continue;
}
// C line 35736
67 => {
vm_block = if (ss_check(ctx, s, ((pos).wrapping_add((1 as i32))).wrapping_add(diff), op, stack_len, catch_pos)) != 0 { 66 } else { 65 }; continue;
}
// C line 35735
68 => {
let _ = { let assigned = ((((*(bc_buf).offset(((pos).wrapping_add((1 as i32))) as isize)) as i8)) as i32); diff = assigned; assigned };
vm_block = 67; continue;
}
// C line 35732
69 => {
vm_block = 13; continue;
}
// C line 35731
70 => {
let _ = { let assigned = ((pos).wrapping_add((1 as i32))).wrapping_add(diff); pos_next = assigned; assigned };
vm_block = 69; continue;
}
// C line 35730
71 => {
let _ = { let assigned = ((((*(bc_buf).offset(((pos).wrapping_add((1 as i32))) as isize)) as i8)) as i32); diff = assigned; assigned };
vm_block = 70; continue;
}
// C line 35728
72 => {
vm_block = 13; continue;
}
// C line 35727
73 => {
let _ = { let assigned = ((pos).wrapping_add((1 as i32))).wrapping_add(diff); pos_next = assigned; assigned };
vm_block = 72; continue;
}
// C line 35726
74 => {
let _ = { let assigned = ((((get_u16(((bc_buf).offset(((pos) as isize))).offset((((1 as i32)) as isize)))) as i16)) as i32); diff = assigned; assigned };
vm_block = 73; continue;
}
// C line 35723
75 => {
vm_block = 13; continue;
}
// C line 35722
76 => {
let _ = { let assigned = ((pos).wrapping_add((1 as i32))).wrapping_add(diff); pos_next = assigned; assigned };
vm_block = 75; continue;
}
// C line 35721
77 => {
let _ = { let assigned = ((get_u32(((bc_buf).offset(((pos) as isize))).offset((((1 as i32)) as isize)))) as i32); diff = assigned; assigned };
vm_block = 76; continue;
}
// C line 35719
78 => {
vm_block = 11; continue;
}
// C line 35710
79 => {
vm_block = match op { x if x == (OP_nip_catch as i32) => 23, x if x == (OP_iterator_close as i32) => 31, x if x == (OP_nip1 as i32) => 33, x if x == (OP_nip as i32) => 35, x if x == (OP_drop as i32) => 37, x if x == (OP_for_await_of_start as i32) => 39, x if x == (OP_for_of_start as i32) => 39, x if x == (OP_catch as i32) => 44, x if x == (OP_with_put_var as i32) => 48, x if x == (OP_with_get_ref as i32) => 52, x if x == (OP_with_make_ref as i32) => 52, x if x == (OP_with_delete_var as i32) => 56, x if x == (OP_with_get_var as i32) => 56, x if x == (OP_gosub as i32) => 60, x if x == (OP_if_false as i32) => 64, x if x == (OP_if_true as i32) => 64, x if x == (OP_if_false8 as i32) => 68, x if x == (OP_if_true8 as i32) => 68, x if x == (OP_goto8 as i32) => 71, x if x == (OP_goto16 as i32) => 74, x if x == (OP_goto as i32) => 77, x if x == (OP_ret as i32) => 78, x if x == (OP_throw_error as i32) => 78, x if x == (OP_throw as i32) => 78, x if x == (OP_return_async as i32) => 78, x if x == (OP_return_undef as i32) => 78, x if x == (OP_return as i32) => 78, x if x == (OP_tail_call_method as i32) => 78, x if x == (OP_tail_call as i32) => 78, _ => 14, }; continue;
}
// C line 35707
80 => {
vm_block = 5; continue;
}
// C line 35706
81 => {
let _ = JS_ThrowInternalError_cargs(ctx, c"stack overflow (op=%d, pc=%d)".as_ptr(), &[ParserFormatArg::Signed((op) as i32), ParserFormatArg::Signed((pos) as i32)]);
vm_block = 80; continue;
}
// C line 35705
82 => {
vm_block = if (((((*(s)).stack_len_max) > ((65534 as i32))) as i32)) != 0 { 81 } else { 79 }; continue;
}
// C line 35704
83 => {
let _ = { let assigned = stack_len; (*(s)).stack_len_max = assigned; assigned };
vm_block = 82; continue;
}
// C line 35703
84 => {
vm_block = if ((((stack_len) > ((*(s)).stack_len_max)) as i32)) != 0 { 83 } else { 79 }; continue;
}
// C line 35702
85 => {
let _ = { stack_len = ((((stack_len) as i32)).wrapping_add(((((*(oi)).n_push) as i32)).wrapping_sub(n_pop))) as i32; stack_len };
vm_block = 84; continue;
}
// C line 35700
86 => {
vm_block = 5; continue;
}
// C line 35699
87 => {
let _ = JS_ThrowInternalError_cargs(ctx, c"stack underflow (op=%d, pc=%d)".as_ptr(), &[ParserFormatArg::Signed((op) as i32), ParserFormatArg::Signed((pos) as i32)]);
vm_block = 86; continue;
}
// C line 35698
88 => {
vm_block = if ((((stack_len) < (n_pop)) as i32)) != 0 { 87 } else { 85 }; continue;
}
// C line 35689
89 => {
let _ = { n_pop = ((((n_pop) as u32)).wrapping_add(get_u16(((bc_buf).offset(((pos) as isize))).offset((((1 as i32)) as isize))))) as i32; n_pop };
vm_block = 88; continue;
}
// C line 35693
90 => {
let _ = { n_pop = ((((n_pop) as i32)).wrapping_add((op).wrapping_sub((OP_call0 as i32)))) as i32; n_pop };
vm_block = 88; continue;
}
// C line 35692
91 => {
vm_block = if (((((((*(oi)).fmt) as i32)) == ((OP_FMT_npopx as i32))) as i32)) != 0 { 90 } else { 88 }; continue;
}
// C line 35688
92 => {
vm_block = if (((((((((((*(oi)).fmt) as i32)) == ((OP_FMT_npop as i32))) as i32)) != 0) || ((((((((*(oi)).fmt) as i32)) == ((OP_FMT_npop_u16 as i32))) as i32)) != 0)) as i32)) != 0 { 89 } else { 91 }; continue;
}
// C line 35686
93 => {
let _ = { let assigned = (((*(oi)).n_pop) as i32); n_pop = assigned; assigned };
vm_block = 92; continue;
}
// C line 35684
94 => {
vm_block = 5; continue;
}
// C line 35683
95 => {
let _ = JS_ThrowInternalError_cargs(ctx, c"bytecode buffer overflow (op=%d, pc=%d)".as_ptr(), &[ParserFormatArg::Signed((op) as i32), ParserFormatArg::Signed((pos) as i32)]);
vm_block = 94; continue;
}
// C line 35682
96 => {
vm_block = if ((((pos_next) > ((*(s)).bc_len)) as i32)) != 0 { 95 } else { 93 }; continue;
}
// C line 35681
97 => {
let _ = { let assigned = (pos).wrapping_add((((*(oi)).size) as i32)); pos_next = assigned; assigned };
vm_block = 96; continue;
}
// C line 35677
98 => {
let _ = { let assigned = core::ptr::addr_of!(*((opcode_info).as_ptr()).offset((if ((((op) >= ((OP_TEMP_START as i32))) as i32)) != 0 { (op).wrapping_add(((OP_TEMP_END as i32)).wrapping_sub((OP_TEMP_START as i32))) } else { op }) as isize)); oi = assigned; assigned };
vm_block = 97; continue;
}
// C line 35675
99 => {
vm_block = 5; continue;
}
// C line 35674
100 => {
let _ = JS_ThrowInternalError_cargs(ctx, c"invalid opcode (op=%d, pc=%d)".as_ptr(), &[ParserFormatArg::Signed((op) as i32), ParserFormatArg::Signed((pos) as i32)]);
vm_block = 99; continue;
}
// C line 35673
101 => {
vm_block = if ((((((((op) == ((0 as i32))) as i32)) != 0) || (((((op) >= ((OP_COUNT as i32))) as i32)) != 0)) as i32)) != 0 { 100 } else { 98 }; continue;
}
// C line 35672
102 => {
let _ = { let assigned = ((*(bc_buf).offset((pos) as isize)) as i32); op = assigned; assigned };
vm_block = 101; continue;
}
// C line 35671
103 => {
let _ = { let assigned = *((*(s)).catch_pos_tab).offset((pos) as isize); catch_pos = assigned; assigned };
vm_block = 102; continue;
}
// C line 35670
104 => {
let _ = { let assigned = ((*((*(s)).stack_level_tab).offset((pos) as isize)) as i32); stack_len = assigned; assigned };
vm_block = 103; continue;
}
// C line 35669
105 => {
let _ = { let assigned = *((*(s)).pc_stack).offset(({ (*(s)).pc_stack_len = ((*(s)).pc_stack_len).wrapping_sub(1); (*(s)).pc_stack_len }) as isize); pos = assigned; assigned };
vm_block = 104; continue;
}
// C line 35666
106 => {
vm_block = 5; continue;
}
// C line 35665
107 => {
vm_block = if (ss_check(ctx, s, (0 as i32), (OP_invalid as i32), (0 as i32), ((1 as i32)).wrapping_neg())) != 0 { 106 } else { 11 }; continue;
}
// C line 35662
108 => {
let _ = { let assigned = (0 as i32); (*(s)).pc_stack_size = assigned; assigned };
vm_block = 107; continue;
}
// C line 35661
109 => {
let _ = { let assigned = (0 as i32); (*(s)).pc_stack_len = assigned; assigned };
vm_block = 108; continue;
}
// C line 35660
110 => {
let _ = { let assigned = (0 as i32); (*(s)).stack_len_max = assigned; assigned };
vm_block = 109; continue;
}
// C line 35658
111 => {
vm_block = 5; continue;
}
// C line 35657
112 => {
vm_block = if ((!(!((*(s)).catch_pos_tab).is_null()) as i32)) != 0 { 111 } else { 110 }; continue;
}
// C line 35655
113 => {
let _ = { let assigned = ((js_malloc(ctx, ((size_of::<i32>() as usize)).wrapping_mul((((*(s)).bc_len) as usize)))) as *mut i32); (*(s)).catch_pos_tab = assigned; assigned };
vm_block = 112; continue;
}
// C line 35654
114 => {
let _ = { let assigned = core::ptr::null_mut::<i32>(); (*(s)).pc_stack = assigned; assigned };
vm_block = 113; continue;
}
// C line 35652
115 => {
vm_block = if ((((i) < ((*(s)).bc_len)) as i32)) != 0 { 117 } else { 114 }; continue;
}
// C line 35652
116 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 115; continue;
}
// C line 35653
117 => {
let _ = { let assigned = (((65535 as i32)) as u16); *((*(s)).stack_level_tab).offset((i) as isize) = assigned; assigned };
vm_block = 116; continue;
}
// C line 35652
118 => {
let _ = { let assigned = (0 as i32); i = assigned; assigned };
vm_block = 115; continue;
}
// C line 35651
119 => {
return ((1 as i32)).wrapping_neg();
}
// C line 35650
120 => {
vm_block = if ((!(!((*(s)).stack_level_tab).is_null()) as i32)) != 0 { 119 } else { 118 }; continue;
}
// C line 35648
121 => {
let _ = { let assigned = ((js_malloc(ctx, ((size_of::<u16>() as usize)).wrapping_mul((((*(s)).bc_len) as usize)))) as *mut u16); (*(s)).stack_level_tab = assigned; assigned };
vm_block = 120; continue;
}
// C line 35646
122 => {
let _ = { let assigned = ((((*(fd)).byte_code).size) as i32); (*(s)).bc_len = assigned; assigned };
vm_block = 121; continue;
}
// C line 35645
123 => {
let _ = { let assigned = ((*(fd)).byte_code).buf; bc_buf = assigned; assigned };
vm_block = 122; continue;
}
// C line 35640
124 => {
s = core::ptr::addr_of_mut!(s_s);
vm_block = 123; continue;
}
_ => std::process::abort(),
} }
}
// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:35636. Bellard/Gordon MIT.
#[cfg(not(feature = "short-opcodes"))]
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn compute_stack_size(mut ctx: *mut JSContext, mut fd: *mut JSFunctionDef, mut pstack_size: *mut i32) -> i32 {
let mut vm_local_storage = Vec::<u64>::new();
let mut s_s: StackSizeState = core::mem::zeroed();
let mut s: *mut StackSizeState = core::ptr::null_mut();
let mut i: i32 = 0;
let mut diff: i32 = 0;
let mut n_pop: i32 = 0;
let mut pos_next: i32 = 0;
let mut stack_len: i32 = 0;
let mut pos: i32 = 0;
let mut op: i32 = 0;
let mut catch_pos: i32 = 0;
let mut catch_level: i32 = 0;
let mut oi: *const JSOpCode = core::ptr::null();
let mut bc_buf: *const u8 = core::ptr::null();
let mut level: i32 = 0;
let mut vm_block: usize = 112;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 35834
1 => {
return ((1 as i32)).wrapping_neg();
}
// C line 35833
2 => {
let _ = { let assigned = (0 as i32); *(pstack_size) = assigned; assigned };
vm_block = 1; continue;
}
// C line 35832
3 => {
let _ = js_free(ctx, (((*(s)).stack_level_tab) as *mut c_void));
vm_block = 2; continue;
}
// C line 35831
4 => {
let _ = js_free(ctx, (((*(s)).catch_pos_tab) as *mut c_void));
vm_block = 3; continue;
}
// C line 35830 labels: fail
5 => {
let _ = js_free(ctx, (((*(s)).pc_stack) as *mut c_void));
vm_block = 4; continue;
}
// C line 35828
6 => {
return (0 as i32);
}
// C line 35827
7 => {
let _ = { let assigned = (*(s)).stack_len_max; *(pstack_size) = assigned; assigned };
vm_block = 6; continue;
}
// C line 35826
8 => {
let _ = js_free(ctx, (((*(s)).stack_level_tab) as *mut c_void));
vm_block = 7; continue;
}
// C line 35825
9 => {
let _ = js_free(ctx, (((*(s)).catch_pos_tab) as *mut c_void));
vm_block = 8; continue;
}
// C line 35824
10 => {
let _ = js_free(ctx, (((*(s)).pc_stack) as *mut c_void));
vm_block = 9; continue;
}
// C line 35668 labels: done_insn
11 => {
vm_block = if (((((*(s)).pc_stack_len) > ((0 as i32))) as i32)) != 0 { 93 } else { 10 }; continue;
}
// C line 35821
12 => {
vm_block = 5; continue;
}
// C line 35820
13 => {
vm_block = if (ss_check(ctx, s, pos_next, op, stack_len, catch_pos)) != 0 { 12 } else { 11 }; continue;
}
// C line 35818
14 => {
vm_block = 13; continue;
}
// C line 35816
15 => {
vm_block = 13; continue;
}
// C line 35815
16 => {
let _ = { let assigned = *((*(s)).catch_pos_tab).offset((catch_pos) as isize); catch_pos = assigned; assigned };
vm_block = 15; continue;
}
// C line 35814
17 => {
let _ = { let old = stack_len; stack_len = (stack_len).wrapping_add(1); old };
vm_block = 16; continue;
}
// C line 35813
18 => {
let _ = { let old = stack_len; stack_len = (stack_len).wrapping_add(1); old };
vm_block = 17; continue;
}
// C line 35812
19 => {
vm_block = if ((((((*(bc_buf).offset((catch_pos) as isize)) as i32)) != ((OP_catch as i32))) as i32)) != 0 { 18 } else { 17 }; continue;
}
// C line 35811
20 => {
let _ = { let assigned = ((*((*(s)).stack_level_tab).offset((catch_pos) as isize)) as i32); stack_len = assigned; assigned };
vm_block = 19; continue;
}
// C line 35809
21 => {
vm_block = 5; continue;
}
// C line 35808
22 => {
let _ = JS_ThrowInternalError_cargs(ctx, c"nip_catch: no catch op (pc=%d)".as_ptr(), &[ParserFormatArg::Signed((pos) as i32)]);
vm_block = 21; continue;
}
// C line 35807
23 => {
vm_block = if ((((catch_pos) < ((0 as i32))) as i32)) != 0 { 22 } else { 20 }; continue;
}
// C line 35805
24 => {
vm_block = 13; continue;
}
// C line 35802
25 => {
let _ = { let assigned = *((*(s)).catch_pos_tab).offset((catch_pos) as isize); catch_pos = assigned; assigned };
vm_block = 24; continue;
}
// C line 35801
26 => {
vm_block = if ((((catch_level) == (level)) as i32)) != 0 { 25 } else { 24 }; continue;
}
// C line 35799
27 => {
let _ = { let old = level; level = (level).wrapping_add(1); old };
vm_block = 26; continue;
}
// C line 35798
28 => {
vm_block = if ((((((*(bc_buf).offset((catch_pos) as isize)) as i32)) != ((OP_catch as i32))) as i32)) != 0 { 27 } else { 26 }; continue;
}
// C line 35797
29 => {
let _ = { let assigned = ((*((*(s)).stack_level_tab).offset((catch_pos) as isize)) as i32); level = assigned; assigned };
vm_block = 28; continue;
}
// C line 35795 labels: check_catch
30 => {
vm_block = if ((((catch_pos) >= ((0 as i32))) as i32)) != 0 { 29 } else { 24 }; continue;
}
// C line 35790
31 => {
let _ = { let assigned = (stack_len).wrapping_add((2 as i32)); catch_level = assigned; assigned };
vm_block = 30; continue;
}
// C line 35788
32 => {
vm_block = 30; continue;
}
// C line 35787
33 => {
let _ = { let assigned = (stack_len).wrapping_sub((1 as i32)); catch_level = assigned; assigned };
vm_block = 32; continue;
}
// C line 35785
34 => {
vm_block = 30; continue;
}
// C line 35784
35 => {
let _ = { let assigned = (stack_len).wrapping_sub((1 as i32)); catch_level = assigned; assigned };
vm_block = 34; continue;
}
// C line 35782
36 => {
vm_block = 30; continue;
}
// C line 35781
37 => {
let _ = { let assigned = stack_len; catch_level = assigned; assigned };
vm_block = 36; continue;
}
// C line 35777
38 => {
vm_block = 13; continue;
}
// C line 35776
39 => {
let _ = { let assigned = pos; catch_pos = assigned; assigned };
vm_block = 38; continue;
}
// C line 35773
40 => {
vm_block = 13; continue;
}
// C line 35772
41 => {
let _ = { let assigned = pos; catch_pos = assigned; assigned };
vm_block = 40; continue;
}
// C line 35771
42 => {
vm_block = 5; continue;
}
// C line 35770
43 => {
vm_block = if (ss_check(ctx, s, ((pos).wrapping_add((1 as i32))).wrapping_add(diff), op, stack_len, catch_pos)) != 0 { 42 } else { 41 }; continue;
}
// C line 35769
44 => {
let _ = { let assigned = ((get_u32(((bc_buf).offset(((pos) as isize))).offset((((1 as i32)) as isize)))) as i32); diff = assigned; assigned };
vm_block = 43; continue;
}
// C line 35767
45 => {
vm_block = 13; continue;
}
// C line 35766
46 => {
vm_block = 5; continue;
}
// C line 35765
47 => {
vm_block = if (ss_check(ctx, s, ((pos).wrapping_add((5 as i32))).wrapping_add(diff), op, (stack_len).wrapping_sub((1 as i32)), catch_pos)) != 0 { 46 } else { 45 }; continue;
}
// C line 35764
48 => {
let _ = { let assigned = ((get_u32(((bc_buf).offset(((pos) as isize))).offset((((5 as i32)) as isize)))) as i32); diff = assigned; assigned };
vm_block = 47; continue;
}
// C line 35762
49 => {
vm_block = 13; continue;
}
// C line 35761
50 => {
vm_block = 5; continue;
}
// C line 35760
51 => {
vm_block = if (ss_check(ctx, s, ((pos).wrapping_add((5 as i32))).wrapping_add(diff), op, (stack_len).wrapping_add((2 as i32)), catch_pos)) != 0 { 50 } else { 49 }; continue;
}
// C line 35759
52 => {
let _ = { let assigned = ((get_u32(((bc_buf).offset(((pos) as isize))).offset((((5 as i32)) as isize)))) as i32); diff = assigned; assigned };
vm_block = 51; continue;
}
// C line 35756
53 => {
vm_block = 13; continue;
}
// C line 35755
54 => {
vm_block = 5; continue;
}
// C line 35754
55 => {
vm_block = if (ss_check(ctx, s, ((pos).wrapping_add((5 as i32))).wrapping_add(diff), op, (stack_len).wrapping_add((1 as i32)), catch_pos)) != 0 { 54 } else { 53 }; continue;
}
// C line 35753
56 => {
let _ = { let assigned = ((get_u32(((bc_buf).offset(((pos) as isize))).offset((((5 as i32)) as isize)))) as i32); diff = assigned; assigned };
vm_block = 55; continue;
}
// C line 35750
57 => {
vm_block = 13; continue;
}
// C line 35749
58 => {
vm_block = 5; continue;
}
// C line 35748
59 => {
vm_block = if (ss_check(ctx, s, ((pos).wrapping_add((1 as i32))).wrapping_add(diff), op, (stack_len).wrapping_add((1 as i32)), catch_pos)) != 0 { 58 } else { 57 }; continue;
}
// C line 35747
60 => {
let _ = { let assigned = ((get_u32(((bc_buf).offset(((pos) as isize))).offset((((1 as i32)) as isize)))) as i32); diff = assigned; assigned };
vm_block = 59; continue;
}
// C line 35745
61 => {
vm_block = 13; continue;
}
// C line 35744
62 => {
vm_block = 5; continue;
}
// C line 35743
63 => {
vm_block = if (ss_check(ctx, s, ((pos).wrapping_add((1 as i32))).wrapping_add(diff), op, stack_len, catch_pos)) != 0 { 62 } else { 61 }; continue;
}
// C line 35742
64 => {
let _ = { let assigned = ((get_u32(((bc_buf).offset(((pos) as isize))).offset((((1 as i32)) as isize)))) as i32); diff = assigned; assigned };
vm_block = 63; continue;
}
// C line 35723
65 => {
vm_block = 13; continue;
}
// C line 35722
66 => {
let _ = { let assigned = ((pos).wrapping_add((1 as i32))).wrapping_add(diff); pos_next = assigned; assigned };
vm_block = 65; continue;
}
// C line 35721
67 => {
let _ = { let assigned = ((get_u32(((bc_buf).offset(((pos) as isize))).offset((((1 as i32)) as isize)))) as i32); diff = assigned; assigned };
vm_block = 66; continue;
}
// C line 35719
68 => {
vm_block = 11; continue;
}
// C line 35710
69 => {
vm_block = match op { x if x == (OP_nip_catch as i32) => 23, x if x == (OP_iterator_close as i32) => 31, x if x == (OP_nip1 as i32) => 33, x if x == (OP_nip as i32) => 35, x if x == (OP_drop as i32) => 37, x if x == (OP_for_await_of_start as i32) => 39, x if x == (OP_for_of_start as i32) => 39, x if x == (OP_catch as i32) => 44, x if x == (OP_with_put_var as i32) => 48, x if x == (OP_with_get_ref as i32) => 52, x if x == (OP_with_make_ref as i32) => 52, x if x == (OP_with_delete_var as i32) => 56, x if x == (OP_with_get_var as i32) => 56, x if x == (OP_gosub as i32) => 60, x if x == (OP_if_false as i32) => 64, x if x == (OP_if_true as i32) => 64, x if x == (OP_goto as i32) => 67, x if x == (OP_ret as i32) => 68, x if x == (OP_throw_error as i32) => 68, x if x == (OP_throw as i32) => 68, x if x == (OP_return_async as i32) => 68, x if x == (OP_return_undef as i32) => 68, x if x == (OP_return as i32) => 68, x if x == (OP_tail_call_method as i32) => 68, x if x == (OP_tail_call as i32) => 68, _ => 14, }; continue;
}
// C line 35707
70 => {
vm_block = 5; continue;
}
// C line 35706
71 => {
let _ = JS_ThrowInternalError_cargs(ctx, c"stack overflow (op=%d, pc=%d)".as_ptr(), &[ParserFormatArg::Signed((op) as i32), ParserFormatArg::Signed((pos) as i32)]);
vm_block = 70; continue;
}
// C line 35705
72 => {
vm_block = if (((((*(s)).stack_len_max) > ((65534 as i32))) as i32)) != 0 { 71 } else { 69 }; continue;
}
// C line 35704
73 => {
let _ = { let assigned = stack_len; (*(s)).stack_len_max = assigned; assigned };
vm_block = 72; continue;
}
// C line 35703
74 => {
vm_block = if ((((stack_len) > ((*(s)).stack_len_max)) as i32)) != 0 { 73 } else { 69 }; continue;
}
// C line 35702
75 => {
let _ = { stack_len = ((((stack_len) as i32)).wrapping_add(((((*(oi)).n_push) as i32)).wrapping_sub(n_pop))) as i32; stack_len };
vm_block = 74; continue;
}
// C line 35700
76 => {
vm_block = 5; continue;
}
// C line 35699
77 => {
let _ = JS_ThrowInternalError_cargs(ctx, c"stack underflow (op=%d, pc=%d)".as_ptr(), &[ParserFormatArg::Signed((op) as i32), ParserFormatArg::Signed((pos) as i32)]);
vm_block = 76; continue;
}
// C line 35698
78 => {
vm_block = if ((((stack_len) < (n_pop)) as i32)) != 0 { 77 } else { 75 }; continue;
}
// C line 35689
79 => {
let _ = { n_pop = ((((n_pop) as u32)).wrapping_add(get_u16(((bc_buf).offset(((pos) as isize))).offset((((1 as i32)) as isize))))) as i32; n_pop };
vm_block = 78; continue;
}
// C line 35688
80 => {
vm_block = if (((((((((((*(oi)).fmt) as i32)) == ((OP_FMT_npop as i32))) as i32)) != 0) || ((((((((*(oi)).fmt) as i32)) == ((OP_FMT_npop_u16 as i32))) as i32)) != 0)) as i32)) != 0 { 79 } else { 78 }; continue;
}
// C line 35686
81 => {
let _ = { let assigned = (((*(oi)).n_pop) as i32); n_pop = assigned; assigned };
vm_block = 80; continue;
}
// C line 35684
82 => {
vm_block = 5; continue;
}
// C line 35683
83 => {
let _ = JS_ThrowInternalError_cargs(ctx, c"bytecode buffer overflow (op=%d, pc=%d)".as_ptr(), &[ParserFormatArg::Signed((op) as i32), ParserFormatArg::Signed((pos) as i32)]);
vm_block = 82; continue;
}
// C line 35682
84 => {
vm_block = if ((((pos_next) > ((*(s)).bc_len)) as i32)) != 0 { 83 } else { 81 }; continue;
}
// C line 35681
85 => {
let _ = { let assigned = (pos).wrapping_add((((*(oi)).size) as i32)); pos_next = assigned; assigned };
vm_block = 84; continue;
}
// C line 35677
86 => {
let _ = { let assigned = core::ptr::addr_of!(*((opcode_info).as_ptr()).offset((op) as isize)); oi = assigned; assigned };
vm_block = 85; continue;
}
// C line 35675
87 => {
vm_block = 5; continue;
}
// C line 35674
88 => {
let _ = JS_ThrowInternalError_cargs(ctx, c"invalid opcode (op=%d, pc=%d)".as_ptr(), &[ParserFormatArg::Signed((op) as i32), ParserFormatArg::Signed((pos) as i32)]);
vm_block = 87; continue;
}
// C line 35673
89 => {
vm_block = if ((((((((op) == ((0 as i32))) as i32)) != 0) || (((((op) >= ((OP_COUNT as i32))) as i32)) != 0)) as i32)) != 0 { 88 } else { 86 }; continue;
}
// C line 35672
90 => {
let _ = { let assigned = ((*(bc_buf).offset((pos) as isize)) as i32); op = assigned; assigned };
vm_block = 89; continue;
}
// C line 35671
91 => {
let _ = { let assigned = *((*(s)).catch_pos_tab).offset((pos) as isize); catch_pos = assigned; assigned };
vm_block = 90; continue;
}
// C line 35670
92 => {
let _ = { let assigned = ((*((*(s)).stack_level_tab).offset((pos) as isize)) as i32); stack_len = assigned; assigned };
vm_block = 91; continue;
}
// C line 35669
93 => {
let _ = { let assigned = *((*(s)).pc_stack).offset(({ (*(s)).pc_stack_len = ((*(s)).pc_stack_len).wrapping_sub(1); (*(s)).pc_stack_len }) as isize); pos = assigned; assigned };
vm_block = 92; continue;
}
// C line 35666
94 => {
vm_block = 5; continue;
}
// C line 35665
95 => {
vm_block = if (ss_check(ctx, s, (0 as i32), (OP_invalid as i32), (0 as i32), ((1 as i32)).wrapping_neg())) != 0 { 94 } else { 11 }; continue;
}
// C line 35662
96 => {
let _ = { let assigned = (0 as i32); (*(s)).pc_stack_size = assigned; assigned };
vm_block = 95; continue;
}
// C line 35661
97 => {
let _ = { let assigned = (0 as i32); (*(s)).pc_stack_len = assigned; assigned };
vm_block = 96; continue;
}
// C line 35660
98 => {
let _ = { let assigned = (0 as i32); (*(s)).stack_len_max = assigned; assigned };
vm_block = 97; continue;
}
// C line 35658
99 => {
vm_block = 5; continue;
}
// C line 35657
100 => {
vm_block = if ((!(!((*(s)).catch_pos_tab).is_null()) as i32)) != 0 { 99 } else { 98 }; continue;
}
// C line 35655
101 => {
let _ = { let assigned = ((js_malloc(ctx, ((size_of::<i32>() as usize)).wrapping_mul((((*(s)).bc_len) as usize)))) as *mut i32); (*(s)).catch_pos_tab = assigned; assigned };
vm_block = 100; continue;
}
// C line 35654
102 => {
let _ = { let assigned = core::ptr::null_mut::<i32>(); (*(s)).pc_stack = assigned; assigned };
vm_block = 101; continue;
}
// C line 35652
103 => {
vm_block = if ((((i) < ((*(s)).bc_len)) as i32)) != 0 { 105 } else { 102 }; continue;
}
// C line 35652
104 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 103; continue;
}
// C line 35653
105 => {
let _ = { let assigned = (((65535 as i32)) as u16); *((*(s)).stack_level_tab).offset((i) as isize) = assigned; assigned };
vm_block = 104; continue;
}
// C line 35652
106 => {
let _ = { let assigned = (0 as i32); i = assigned; assigned };
vm_block = 103; continue;
}
// C line 35651
107 => {
return ((1 as i32)).wrapping_neg();
}
// C line 35650
108 => {
vm_block = if ((!(!((*(s)).stack_level_tab).is_null()) as i32)) != 0 { 107 } else { 106 }; continue;
}
// C line 35648
109 => {
let _ = { let assigned = ((js_malloc(ctx, ((size_of::<u16>() as usize)).wrapping_mul((((*(s)).bc_len) as usize)))) as *mut u16); (*(s)).stack_level_tab = assigned; assigned };
vm_block = 108; continue;
}
// C line 35646
110 => {
let _ = { let assigned = ((((*(fd)).byte_code).size) as i32); (*(s)).bc_len = assigned; assigned };
vm_block = 109; continue;
}
// C line 35645
111 => {
let _ = { let assigned = ((*(fd)).byte_code).buf; bc_buf = assigned; assigned };
vm_block = 110; continue;
}
// C line 35640
112 => {
s = core::ptr::addr_of_mut!(s_s);
vm_block = 111; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:35837. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn add_global_variables(mut ctx: *mut JSContext, mut fd: *mut JSFunctionDef) -> i32 {
let mut vm_local_storage = Vec::<u64>::new();
let mut i: i32 = 0;
let mut idx: i32 = 0;
let mut m: *mut JSModuleDef = core::ptr::null_mut();
let mut me: *mut JSExportEntry = core::ptr::null_mut();
let mut hf: *mut JSGlobalVar = core::ptr::null_mut();
let mut need_global_closures: i32 = 0;
let mut cv: *mut JSClosureVar = core::ptr::null_mut();
let mut closure_type: JSClosureTypeEnum = core::mem::zeroed();
let mut var_kind: JSVarKindEnum = core::mem::zeroed();
let mut vm_block: usize = 35;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 35901
1 => {
return (0 as i32);
}
// C line 35888
2 => {
vm_block = if ((((i) < ((*(m)).export_entries_count)) as i32)) != 0 { 10 } else { 1 }; continue;
}
// C line 35888
3 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 2; continue;
}
// C line 35897
4 => {
let _ = { let assigned = idx; (((*(me)).u).local).var_idx = assigned; assigned };
vm_block = 3; continue;
}
// C line 35895
5 => {
return ((1 as i32)).wrapping_neg();
}
// C line 35893
6 => {
let _ = __JS_ThrowSyntaxErrorAtom(ctx, (*(me)).local_name, c"exported variable '%s' does not exist".as_ptr());
vm_block = 5; continue;
}
// C line 35892
7 => {
vm_block = if ((((idx) < ((0 as i32))) as i32)) != 0 { 6 } else { 4 }; continue;
}
// C line 35891
8 => {
let _ = { let assigned = find_closure_var(ctx, fd, (*(me)).local_name); idx = assigned; assigned };
vm_block = 7; continue;
}
// C line 35890
9 => {
vm_block = if (((((((*(me)).export_type) as u32)) == ((((JS_EXPORT_TYPE_LOCAL as i32)) as u32))) as i32)) != 0 { 8 } else { 3 }; continue;
}
// C line 35889
10 => {
let _ = { let assigned = core::ptr::addr_of_mut!(*((*(m)).export_entries).offset((i) as isize)); me = assigned; assigned };
vm_block = 9; continue;
}
// C line 35888
11 => {
let _ = { let assigned = (0 as i32); i = assigned; assigned };
vm_block = 2; continue;
}
// C line 35886
12 => {
vm_block = if !((*(fd)).module).is_null() { 11 } else { 1 }; continue;
}
// C line 35872
13 => {
vm_block = if ((((i) < ((*(fd)).global_var_count)) as i32)) != 0 { 20 } else { 12 }; continue;
}
// C line 35872
14 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 13; continue;
}
// C line 35882
15 => {
return ((1 as i32)).wrapping_neg();
}
// C line 35880
16 => {
vm_block = if ((((add_closure_var(ctx, fd, closure_type, i, (*(hf)).var_name, (((*(hf)).is_const()) as i32), (((*(hf)).is_lexical()) as i32), var_kind)) < ((0 as i32))) as i32)) != 0 { 15 } else { 14 }; continue;
}
// C line 35876
17 => {
let _ = { let assigned = (((JS_VAR_GLOBAL_FUNCTION_DECL as i32)) as JSVarKindEnum); var_kind = assigned; assigned };
vm_block = 16; continue;
}
// C line 35878
18 => {
let _ = { let assigned = (((JS_VAR_NORMAL as i32)) as JSVarKindEnum); var_kind = assigned; assigned };
vm_block = 16; continue;
}
// C line 35875
19 => {
vm_block = if (((((((((*(hf)).cpool_idx) >= ((0 as i32))) as i32)) != 0) && (((!(((*(hf)).is_lexical()) != 0) as i32)) != 0)) as i32)) != 0 { 17 } else { 18 }; continue;
}
// C line 35874
20 => {
let _ = { let assigned = core::ptr::addr_of_mut!(*((*(fd)).global_vars).offset((i) as isize)); hf = assigned; assigned };
vm_block = 19; continue;
}
// C line 35872
21 => {
let _ = { let assigned = (0 as i32); i = assigned; assigned };
vm_block = 13; continue;
}
// C line 35869
22 => {
let _ = { let assigned = (((JS_CLOSURE_MODULE_DECL as i32)) as JSClosureTypeEnum); closure_type = assigned; assigned };
vm_block = 21; continue;
}
// C line 35871
23 => {
let _ = { let assigned = (((JS_CLOSURE_GLOBAL_DECL as i32)) as JSClosureTypeEnum); closure_type = assigned; assigned };
vm_block = 21; continue;
}
// C line 35868
24 => {
vm_block = if !((*(fd)).module).is_null() { 22 } else { 23 }; continue;
}
// C line 35866
25 => {
vm_block = if (need_global_closures) != 0 { 24 } else { 12 }; continue;
}
// C line 35856
26 => {
vm_block = if ((((idx) < ((*(fd)).closure_var_count)) as i32)) != 0 { 31 } else { 25 }; continue;
}
// C line 35856
27 => {
let _ = { let old = idx; idx = (idx).wrapping_add(1); old };
vm_block = 26; continue;
}
// C line 35861
28 => {
vm_block = 25; continue;
}
// C line 35860
29 => {
let _ = { let assigned = (0 as i32); need_global_closures = assigned; assigned };
vm_block = 28; continue;
}
// C line 35858
30 => {
vm_block = if (((((((((*(cv)).var_name) == ((((crate::quickjs_atom::JS_ATOM__var_ as i32)) as JSAtom))) as i32)) != 0) || ((((((*(cv)).var_name) == ((((crate::quickjs_atom::JS_ATOM__arg_var_ as i32)) as JSAtom))) as i32)) != 0)) as i32)) != 0 { 29 } else { 27 }; continue;
}
// C line 35857
31 => {
cv = core::ptr::addr_of_mut!(*((*(fd)).closure_var).offset((idx) as isize));
vm_block = 30; continue;
}
// C line 35856
32 => {
let _ = { let assigned = (0 as i32); idx = assigned; assigned };
vm_block = 26; continue;
}
// C line 35854
33 => {
vm_block = if (((((((((*(fd)).eval_type) == (((2 as i32)).wrapping_shl(((0 as i32)) as u32))) as i32)) != 0) && (((!(((((((*(fd)).js_mode) as i32)) & (((1 as i32)).wrapping_shl(((0 as i32)) as u32)))) != 0) as i32)) != 0)) as i32)) != 0 { 32 } else { 25 }; continue;
}
// C line 35853
34 => {
let _ = { let assigned = (1 as i32); need_global_closures = assigned; assigned };
vm_block = 33; continue;
}
// C line 35840
35 => {
m = (*(fd)).module;
vm_block = 34; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:35907. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_create_function(mut ctx: *mut JSContext, mut fd: *mut JSFunctionDef) -> JSValue {
let mut vm_local_storage = Vec::<u64>::new();
let mut func_obj: JSValue = core::mem::zeroed();
let mut b: *mut JSFunctionBytecode = core::ptr::null_mut();
let mut el: *mut list_head = core::ptr::null_mut();
let mut el1: *mut list_head = core::ptr::null_mut();
let mut stack_size: i32 = 0;
let mut scope: i32 = 0;
let mut idx: i32 = 0;
let mut function_size: i32 = 0;
let mut byte_code_offset: i32 = 0;
let mut cpool_offset: i32 = 0;
let mut closure_var_offset: i32 = 0;
let mut vardefs_offset: i32 = 0;
let mut strip_var_debug: i32 = 0;
let mut vd: *mut JSVarDef = core::ptr::null_mut();
let mut sd: *mut JSVarScope = core::ptr::null_mut();
let mut vd_1: *mut JSVarDef = core::ptr::null_mut();
let mut fd1: *mut JSFunctionDef = core::ptr::null_mut();
let mut cpool_idx: i32 = 0;
let mut i: i32 = 0;
let mut vd_2: *mut JSVarDef = core::ptr::null_mut();
let mut vd1: *mut JSBytecodeVarDef = core::ptr::null_mut();
let mut vd_3: *mut JSVarDef = core::ptr::null_mut();
let mut vd1_1: *mut JSBytecodeVarDef = core::ptr::null_mut();
let mut i_1: i32 = 0;
let mut cv: *mut JSClosureVar = core::ptr::null_mut();
let mut vm_block: usize = 161;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 36163
1 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 36162 labels: fail
2 => {
let _ = js_free_function_def(ctx, fd);
vm_block = 1; continue;
}
// C line 36160
3 => {
return JSValue { u: JSValueUnion { ptr: ((b) as *mut c_void) }, tag: (((JS_TAG_FUNCTION_BYTECODE as i32)) as i64) };
}
// C line 36159
4 => {
let _ = js_free(ctx, ((fd) as *mut c_void));
vm_block = 3; continue;
}
// C line 36156
5 => {
let _ = list_del(core::ptr::addr_of_mut!((*(fd)).link));
vm_block = 4; continue;
}
// C line 36154
6 => {
vm_block = if !((*(fd)).parent).is_null() { 5 } else { 4 }; continue;
}
// C line 36146
7 => {
let _ = add_gc_object((*(ctx)).rt, core::ptr::addr_of_mut!((*(b)).header), (((JS_GC_OBJ_TYPE_FUNCTION_BYTECODE as i32)) as JSGCObjectTypeEnum));
vm_block = 6; continue;
}
// C line 36144
8 => {
let _ = { let assigned = JS_DupContext(ctx); (*(b)).realm = assigned; assigned };
vm_block = 7; continue;
}
// C line 36142
9 => {
let _ = { let assigned = ((((((((((*(fd)).eval_type) == (((2 as i32)).wrapping_shl(((0 as i32)) as u32))) as i32)) != 0) || ((((((*(fd)).eval_type) == (((3 as i32)).wrapping_shl(((0 as i32)) as u32))) as i32)) != 0)) as i32)) as u8); (*(b)).set_is_direct_or_indirect_eval((assigned) as _); assigned };
vm_block = 8; continue;
}
// C line 36141
10 => {
let _ = { let assigned = (((*(fd)).arguments_allowed) as u8); (*(b)).set_arguments_allowed((assigned) as _); assigned };
vm_block = 9; continue;
}
// C line 36140
11 => {
let _ = { let assigned = (((*(fd)).super_allowed) as u8); (*(b)).set_super_allowed((assigned) as _); assigned };
vm_block = 10; continue;
}
// C line 36139
12 => {
let _ = { let assigned = (((*(fd)).super_call_allowed) as u8); (*(b)).set_super_call_allowed((assigned) as _); assigned };
vm_block = 11; continue;
}
// C line 36138
13 => {
let _ = { let assigned = (((*(fd)).new_target_allowed) as u8); (*(b)).set_new_target_allowed((assigned) as _); assigned };
vm_block = 12; continue;
}
// C line 36136
14 => {
let _ = { let assigned = ((((((((((*(fd)).home_object_var_idx) >= ((0 as i32))) as i32)) != 0) || (((*(fd)).need_home_object) != 0)) as i32)) as u8); (*(b)).set_need_home_object((assigned) as _); assigned };
vm_block = 13; continue;
}
// C line 36135
15 => {
let _ = { let assigned = ((((*(fd)).func_kind as JSFunctionKindEnum)) as u8); (*(b)).set_func_kind((assigned) as _); assigned };
vm_block = 14; continue;
}
// C line 36134
16 => {
let _ = { let assigned = (((*(fd)).is_derived_class_constructor) as u8); (*(b)).set_is_derived_class_constructor((assigned) as _); assigned };
vm_block = 15; continue;
}
// C line 36133
17 => {
let _ = { let assigned = (*(fd)).js_mode; (*(b)).js_mode = assigned; assigned };
vm_block = 16; continue;
}
// C line 36132
18 => {
let _ = { let assigned = (((*(fd)).has_simple_parameter_list) as u8); (*(b)).set_has_simple_parameter_list((assigned) as _); assigned };
vm_block = 17; continue;
}
// C line 36131
19 => {
let _ = { let assigned = (((*(fd)).has_prototype) as u8); (*(b)).set_has_prototype((assigned) as _); assigned };
vm_block = 18; continue;
}
// C line 36129
20 => {
let _ = { let assigned = core::ptr::null_mut::<JSClosureVar>(); (*(fd)).closure_var = assigned; assigned };
vm_block = 19; continue;
}
// C line 36128
21 => {
let _ = js_free(ctx, (((*(fd)).closure_var) as *mut c_void));
vm_block = 20; continue;
}
// C line 36126
22 => {
let _ = { let dst = ((((*(b)).closure_var) as *mut c_void)) as *mut u8; core::ptr::copy_nonoverlapping(((((*(fd)).closure_var) as *const c_void)) as *const u8, dst, (((((*(b)).closure_var_count) as usize)).wrapping_mul((size_of::<JSClosureVar>() as usize))) as usize); dst as *mut c_void };
vm_block = 21; continue;
}
// C line 36125
23 => {
let _ = { let assigned = (((((((b) as *mut u8)).offset(((closure_var_offset) as isize))) as *mut c_void)) as *mut JSClosureVar); (*(b)).closure_var = assigned; assigned };
vm_block = 22; continue;
}
// C line 36113
24 => {
vm_block = if ((((i_1) < ((*(fd)).closure_var_count)) as i32)) != 0 { 29 } else { 23 }; continue;
}
// C line 36113
25 => {
let _ = { let old = i_1; i_1 = (i_1).wrapping_add(1); old };
vm_block = 24; continue;
}
// C line 36121
26 => {
let _ = { let assigned = (((0 as i32)) as JSAtom); (*(cv)).var_name = assigned; assigned };
vm_block = 25; continue;
}
// C line 36120
27 => {
let _ = JS_FreeAtom(ctx, (*(cv)).var_name);
vm_block = 26; continue;
}
// C line 36115
28 => {
vm_block = if (((((((((((((((((((((((*(cv)).closure_type()) as i32)) != ((JS_CLOSURE_GLOBAL_REF as i32))) as i32)) != 0) && ((((((((*(cv)).closure_type()) as i32)) != ((JS_CLOSURE_GLOBAL_DECL as i32))) as i32)) != 0)) as i32)) != 0) && ((((((((*(cv)).closure_type()) as i32)) != ((JS_CLOSURE_GLOBAL as i32))) as i32)) != 0)) as i32)) != 0) && ((((((((*(cv)).closure_type()) as i32)) != ((JS_CLOSURE_MODULE_DECL as i32))) as i32)) != 0)) as i32)) != 0) && ((((((((*(cv)).closure_type()) as i32)) != ((JS_CLOSURE_MODULE_IMPORT as i32))) as i32)) != 0)) as i32)) != 0 { 27 } else { 25 }; continue;
}
// C line 36114
29 => {
cv = core::ptr::addr_of_mut!(*((*(fd)).closure_var).offset((i_1) as isize));
vm_block = 28; continue;
}
// C line 36113
30 => {
let _ = { let assigned = (0 as i32); i_1 = assigned; assigned };
vm_block = 24; continue;
}
// C line 36111
31 => {
vm_block = if (strip_var_debug) != 0 { 30 } else { 23 }; continue;
}
// C line 36110
32 => {
vm_block = if ((*(b)).closure_var_count) != 0 { 31 } else { 21 }; continue;
}
// C line 36109
33 => {
let _ = { let assigned = (*(fd)).closure_var_count; (*(b)).closure_var_count = assigned; assigned };
vm_block = 32; continue;
}
// C line 36107
34 => {
let _ = js_free(ctx, (((*(fd)).scopes) as *mut c_void));
vm_block = 33; continue;
}
// C line 36106
35 => {
vm_block = if (((((*(fd)).scopes) != (((*(fd)).def_scope_array).as_mut_ptr())) as i32)) != 0 { 34 } else { 33 }; continue;
}
// C line 36088
36 => {
let _ = dbuf_free(core::ptr::addr_of_mut!((*(fd)).pc2line));
vm_block = 35; continue;
}
// C line 36087
37 => {
let _ = JS_FreeAtom(ctx, (*(fd)).filename);
vm_block = 36; continue;
}
// C line 36104
38 => {
let _ = { let assigned = (*(fd)).source_len; ((*(b)).debug).source_len = assigned; assigned };
vm_block = 35; continue;
}
// C line 36103
39 => {
let _ = { let assigned = (*(fd)).source; ((*(b)).debug).source = assigned; assigned };
vm_block = 38; continue;
}
// C line 36102
40 => {
let _ = { let assigned = ((((*(fd)).pc2line).size) as i32); ((*(b)).debug).pc2line_len = assigned; assigned };
vm_block = 39; continue;
}
// C line 36101
41 => {
let _ = { let assigned = ((*(fd)).pc2line).buf; ((*(b)).debug).pc2line_buf = assigned; assigned };
vm_block = 40; continue;
}
// C line 36100
42 => {
vm_block = if ((!(!(((*(b)).debug).pc2line_buf).is_null()) as i32)) != 0 { 41 } else { 40 }; continue;
}
// C line 36099
43 => {
let _ = { let assigned = ((js_realloc(ctx, ((((*(fd)).pc2line).buf) as *mut c_void), ((*(fd)).pc2line).size)) as *mut u8); ((*(b)).debug).pc2line_buf = assigned; assigned };
vm_block = 42; continue;
}
// C line 36094
44 => {
let _ = { let assigned = (*(fd)).filename; ((*(b)).debug).filename = assigned; assigned };
vm_block = 43; continue;
}
// C line 36093
45 => {
let _ = { let assigned = (((1 as i32)) as u8); (*(b)).set_has_debug((assigned) as _); assigned };
vm_block = 44; continue;
}
// C line 36086
46 => {
vm_block = if ((*(fd)).strip_debug()) != 0 { 37 } else { 45 }; continue;
}
// C line 36084
47 => {
let _ = { let assigned = ((stack_size) as u16); (*(b)).stack_size = assigned; assigned };
vm_block = 46; continue;
}
// C line 36082
48 => {
let _ = { let assigned = core::ptr::null_mut::<JSValue>(); (*(fd)).cpool = assigned; assigned };
vm_block = 47; continue;
}
// C line 36081
49 => {
let _ = js_free(ctx, (((*(fd)).cpool) as *mut c_void));
vm_block = 48; continue;
}
// C line 36079
50 => {
let _ = { let dst = ((((*(b)).cpool) as *mut c_void)) as *mut u8; core::ptr::copy_nonoverlapping(((((*(fd)).cpool) as *const c_void)) as *const u8, dst, (((((*(b)).cpool_count) as usize)).wrapping_mul((size_of::<JSValue>() as usize))) as usize); dst as *mut c_void };
vm_block = 49; continue;
}
// C line 36078
51 => {
let _ = { let assigned = (((((((b) as *mut u8)).offset(((cpool_offset) as isize))) as *mut c_void)) as *mut JSValue); (*(b)).cpool = assigned; assigned };
vm_block = 50; continue;
}
// C line 36077
52 => {
vm_block = if ((*(b)).cpool_count) != 0 { 51 } else { 49 }; continue;
}
// C line 36076
53 => {
let _ = { let assigned = (*(fd)).cpool_count; (*(b)).cpool_count = assigned; assigned };
vm_block = 52; continue;
}
// C line 36074
54 => {
let _ = js_free(ctx, (((*(fd)).vars) as *mut c_void));
vm_block = 53; continue;
}
// C line 36073
55 => {
let _ = js_free(ctx, (((*(fd)).args) as *mut c_void));
vm_block = 54; continue;
}
// C line 36072
56 => {
let _ = { let assigned = (((*(fd)).var_ref_count) as u16); (*(b)).var_ref_count = assigned; assigned };
vm_block = 55; continue;
}
// C line 36071
57 => {
let _ = { let assigned = (((*(fd)).defined_arg_count) as u16); (*(b)).defined_arg_count = assigned; assigned };
vm_block = 56; continue;
}
// C line 36070
58 => {
let _ = { let assigned = (((*(fd)).arg_count) as u16); (*(b)).arg_count = assigned; assigned };
vm_block = 57; continue;
}
// C line 36069
59 => {
let _ = { let assigned = (((*(fd)).var_count) as u16); (*(b)).var_count = assigned; assigned };
vm_block = 58; continue;
}
// C line 36052
60 => {
vm_block = if ((((i) < ((*(fd)).var_count)) as i32)) != 0 { 74 } else { 59 }; continue;
}
// C line 36052
61 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 60; continue;
}
// C line 36067
62 => {
let _ = { let assigned = (*(vd_3)).var_ref_idx; (*(vd1_1)).var_ref_idx = assigned; assigned };
vm_block = 61; continue;
}
// C line 36066
63 => {
let _ = { let assigned = (*(vd_3)).var_kind(); (*(vd1_1)).set_var_kind((assigned) as _); assigned };
vm_block = 62; continue;
}
// C line 36065
64 => {
let _ = { let assigned = (*(vd_3)).is_captured(); (*(vd1_1)).set_is_captured((assigned) as _); assigned };
vm_block = 63; continue;
}
// C line 36064
65 => {
let _ = { let assigned = (*(vd_3)).is_lexical(); (*(vd1_1)).set_is_lexical((assigned) as _); assigned };
vm_block = 64; continue;
}
// C line 36063
66 => {
let _ = { let assigned = (*(vd_3)).is_const(); (*(vd1_1)).set_is_const((assigned) as _); assigned };
vm_block = 65; continue;
}
// C line 36062
67 => {
let _ = { let assigned = (*(vd_3)).scope_next; (*(vd1_1)).scope_next = assigned; assigned };
vm_block = 66; continue;
}
// C line 36061
68 => {
let _ = { let assigned = ((((((*(vd_3)).scope_level) != ((0 as i32))) as i32)) as u8); (*(vd1_1)).set_has_scope((assigned) as _); assigned };
vm_block = 67; continue;
}
// C line 36057
69 => {
let _ = { let assigned = (((0 as i32)) as JSAtom); (*(vd1_1)).var_name = assigned; assigned };
vm_block = 68; continue;
}
// C line 36056
70 => {
let _ = JS_FreeAtom(ctx, (*(vd_3)).var_name);
vm_block = 69; continue;
}
// C line 36059
71 => {
let _ = { let assigned = (*(vd_3)).var_name; (*(vd1_1)).var_name = assigned; assigned };
vm_block = 68; continue;
}
// C line 36055
72 => {
vm_block = if (strip_var_debug) != 0 { 70 } else { 71 }; continue;
}
// C line 36054
73 => {
vd1_1 = core::ptr::addr_of_mut!(*((*(b)).vardefs).offset(((i).wrapping_add((*(fd)).arg_count)) as isize));
vm_block = 72; continue;
}
// C line 36053
74 => {
vd_3 = core::ptr::addr_of_mut!(*((*(fd)).vars).offset((i) as isize));
vm_block = 73; continue;
}
// C line 36052
75 => {
let _ = { let assigned = (0 as i32); i = assigned; assigned };
vm_block = 60; continue;
}
// C line 36034
76 => {
vm_block = if ((((i) < ((*(fd)).arg_count)) as i32)) != 0 { 90 } else { 75 }; continue;
}
// C line 36034
77 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 76; continue;
}
// C line 36049
78 => {
let _ = { let assigned = (*(vd_2)).var_ref_idx; (*(vd1)).var_ref_idx = assigned; assigned };
vm_block = 77; continue;
}
// C line 36048
79 => {
let _ = { let assigned = (*(vd_2)).var_kind(); (*(vd1)).set_var_kind((assigned) as _); assigned };
vm_block = 78; continue;
}
// C line 36047
80 => {
let _ = { let assigned = (*(vd_2)).is_captured(); (*(vd1)).set_is_captured((assigned) as _); assigned };
vm_block = 79; continue;
}
// C line 36046
81 => {
let _ = { let assigned = (*(vd_2)).is_lexical(); (*(vd1)).set_is_lexical((assigned) as _); assigned };
vm_block = 80; continue;
}
// C line 36045
82 => {
let _ = { let assigned = (*(vd_2)).is_const(); (*(vd1)).set_is_const((assigned) as _); assigned };
vm_block = 81; continue;
}
// C line 36044
83 => {
let _ = { let assigned = (*(vd_2)).scope_next; (*(vd1)).scope_next = assigned; assigned };
vm_block = 82; continue;
}
// C line 36043
84 => {
let _ = { let assigned = ((((((*(vd_2)).scope_level) != ((0 as i32))) as i32)) as u8); (*(vd1)).set_has_scope((assigned) as _); assigned };
vm_block = 83; continue;
}
// C line 36039
85 => {
let _ = { let assigned = (((0 as i32)) as JSAtom); (*(vd1)).var_name = assigned; assigned };
vm_block = 84; continue;
}
// C line 36038
86 => {
let _ = JS_FreeAtom(ctx, (*(vd_2)).var_name);
vm_block = 85; continue;
}
// C line 36041
87 => {
let _ = { let assigned = (*(vd_2)).var_name; (*(vd1)).var_name = assigned; assigned };
vm_block = 84; continue;
}
// C line 36037
88 => {
vm_block = if (strip_var_debug) != 0 { 86 } else { 87 }; continue;
}
// C line 36036
89 => {
vd1 = core::ptr::addr_of_mut!(*((*(b)).vardefs).offset((i) as isize));
vm_block = 88; continue;
}
// C line 36035
90 => {
vd_2 = core::ptr::addr_of_mut!(*((*(fd)).args).offset((i) as isize));
vm_block = 89; continue;
}
// C line 36034
91 => {
let _ = { let assigned = (0 as i32); i = assigned; assigned };
vm_block = 76; continue;
}
// C line 36033
92 => {
let _ = { let assigned = (((((((b) as *mut u8)).offset(((vardefs_offset) as isize))) as *mut c_void)) as *mut JSBytecodeVarDef); (*(b)).vardefs = assigned; assigned };
vm_block = 91; continue;
}
// C line 36031
93 => {
vm_block = if ((((((*(fd)).arg_count).wrapping_add((*(fd)).var_count)) > ((0 as i32))) as i32)) != 0 { 92 } else { 53 }; continue;
}
// C line 36030
94 => {
let _ = { let assigned = (*(fd)).func_name; (*(b)).func_name = assigned; assigned };
vm_block = 93; continue;
}
// C line 36029
95 => {
let _ = { let assigned = (((((*(fd)).strip_debug()) != 0) && (((!(((*(fd)).has_eval_call) != 0) as i32)) != 0)) as i32); strip_var_debug = assigned; assigned };
vm_block = 94; continue;
}
// C line 36027
96 => {
let _ = { let assigned = core::ptr::null_mut::<u8>(); ((*(fd)).byte_code).buf = assigned; assigned };
vm_block = 95; continue;
}
// C line 36026
97 => {
let _ = js_free(ctx, ((((*(fd)).byte_code).buf) as *mut c_void));
vm_block = 96; continue;
}
// C line 36025
98 => {
let _ = { let dst = ((((*(b)).byte_code_buf) as *mut c_void)) as *mut u8; core::ptr::copy_nonoverlapping((((((*(fd)).byte_code).buf) as *const c_void)) as *const u8, dst, (((*(fd)).byte_code).size) as usize); dst as *mut c_void };
vm_block = 97; continue;
}
// C line 36024
99 => {
let _ = { let assigned = ((((*(fd)).byte_code).size) as i32); (*(b)).byte_code_len = assigned; assigned };
vm_block = 98; continue;
}
// C line 36023
100 => {
let _ = { let assigned = (((((((b) as *mut u8)).offset(((byte_code_offset) as isize))) as *mut c_void)) as *mut u8); (*(b)).byte_code_buf = assigned; assigned };
vm_block = 99; continue;
}
// C line 36021
101 => {
let _ = { let assigned = (1 as i32); (*(js_rc(((b) as *mut c_void)))).ref_count = assigned; assigned };
vm_block = 100; continue;
}
// C line 36020
102 => {
vm_block = 2; continue;
}
// C line 36019
103 => {
vm_block = if ((!(!(b).is_null()) as i32)) != 0 { 102 } else { 101 }; continue;
}
// C line 36018
104 => {
let _ = { let assigned = ((js_mallocz(ctx, ((function_size) as usize))) as *mut JSFunctionBytecode); b = assigned; assigned };
vm_block = 103; continue;
}
// C line 36016
105 => {
let _ = { function_size = ((((function_size) as usize)).wrapping_add(((*(fd)).byte_code).size)) as i32; function_size };
vm_block = 104; continue;
}
// C line 36015
106 => {
let _ = { let assigned = function_size; byte_code_offset = assigned; assigned };
vm_block = 105; continue;
}
// C line 36014
107 => {
let _ = { function_size = ((((function_size) as usize)).wrapping_add(((((*(fd)).closure_var_count) as usize)).wrapping_mul((size_of::<JSClosureVar>() as usize)))) as i32; function_size };
vm_block = 106; continue;
}
// C line 36013
108 => {
let _ = { let assigned = function_size; closure_var_offset = assigned; assigned };
vm_block = 107; continue;
}
// C line 36012
109 => {
let _ = { function_size = ((((function_size) as usize)).wrapping_add((((((*(fd)).arg_count).wrapping_add((*(fd)).var_count)) as usize)).wrapping_mul((size_of::<JSBytecodeVarDef>() as usize)))) as i32; function_size };
vm_block = 108; continue;
}
// C line 36011
110 => {
let _ = { let assigned = function_size; vardefs_offset = assigned; assigned };
vm_block = 109; continue;
}
// C line 36010
111 => {
let _ = { function_size = ((((function_size) as usize)).wrapping_add(((((*(fd)).cpool_count) as usize)).wrapping_mul((size_of::<JSValue>() as usize)))) as i32; function_size };
vm_block = 110; continue;
}
// C line 36009
112 => {
let _ = { let assigned = function_size; cpool_offset = assigned; assigned };
vm_block = 111; continue;
}
// C line 36005
113 => {
let _ = { let assigned = (((core::mem::offset_of!(JSFunctionBytecode, debug) as usize)) as i32); function_size = assigned; assigned };
vm_block = 112; continue;
}
// C line 36007
114 => {
let _ = { let assigned = (((size_of::<JSFunctionBytecode>() as usize)) as i32); function_size = assigned; assigned };
vm_block = 112; continue;
}
// C line 36004
115 => {
vm_block = if ((*(fd)).strip_debug()) != 0 { 113 } else { 114 }; continue;
}
// C line 36002
116 => {
vm_block = 2; continue;
}
// C line 36001
117 => {
vm_block = if ((((compute_stack_size(ctx, fd, core::ptr::addr_of_mut!(stack_size))) < ((0 as i32))) as i32)) != 0 { 116 } else { 115 }; continue;
}
// C line 35999
118 => {
vm_block = 2; continue;
}
// C line 35998
119 => {
vm_block = if (resolve_labels(ctx, fd)) != 0 { 118 } else { 117 }; continue;
}
// C line 35984
120 => {
vm_block = 2; continue;
}
// C line 35983
121 => {
vm_block = if (resolve_variables(ctx, fd)) != 0 { 120 } else { 119 }; continue;
}
// C line 35957
122 => {
vm_block = if ((((el) != (core::ptr::addr_of_mut!((*(fd)).child_list))) as i32)) != 0 { 130 } else { 121 }; continue;
}
// C line 35957
123 => {
let _ = { let _ = { let assigned = el1; el = assigned; assigned }; { let assigned = (*(el)).next; el1 = assigned; assigned } };
vm_block = 122; continue;
}
// C line 35968
124 => {
let _ = { let assigned = func_obj; *((*(fd)).cpool).offset((cpool_idx) as isize) = assigned; assigned };
vm_block = 123; continue;
}
// C line 35967
125 => {
let _ = if ((((!(((((cpool_idx) >= ((0 as i32))) as i32)) != 0) as i32)) as i64)) != 0 { std::process::abort() } else { { let _ = (0 as i32); } };
vm_block = 124; continue;
}
// C line 35965
126 => {
vm_block = 2; continue;
}
// C line 35964
127 => {
vm_block = if (JS_IsException(func_obj)) != 0 { 126 } else { 125 }; continue;
}
// C line 35963
128 => {
let _ = { let assigned = js_create_function(ctx, fd1); func_obj = assigned; assigned };
vm_block = 127; continue;
}
// C line 35962
129 => {
let _ = { let assigned = (*(fd1)).parent_cpool_idx; cpool_idx = assigned; assigned };
vm_block = 128; continue;
}
// C line 35961
130 => {
let _ = { let assigned = (((((el) as *mut u8)).offset(-(((core::mem::offset_of!(JSFunctionDef, link) as usize)) as isize))) as *mut JSFunctionDef); fd1 = assigned; assigned };
vm_block = 129; continue;
}
// C line 35957
131 => {
let _ = { let _ = { let assigned = (*(core::ptr::addr_of_mut!((*(fd)).child_list))).next; el = assigned; assigned }; { let assigned = (*(el)).next; el1 = assigned; assigned } };
vm_block = 122; continue;
}
// C line 35953
132 => {
vm_block = 2; continue;
}
// C line 35952
133 => {
vm_block = if (add_global_variables(ctx, fd)) != 0 { 132 } else { 131 }; continue;
}
// C line 35951
134 => {
vm_block = if ((*(fd)).is_eval) != 0 { 133 } else { 131 }; continue;
}
// C line 35948
135 => {
let _ = add_eval_variables(ctx, fd);
vm_block = 134; continue;
}
// C line 35947
136 => {
vm_block = if ((*(fd)).has_eval_call) != 0 { 135 } else { 134 }; continue;
}
// C line 35935
137 => {
vm_block = if ((((idx) < ((*(fd)).var_count)) as i32)) != 0 { 142 } else { 136 }; continue;
}
// C line 35935
138 => {
let _ = { let old = idx; idx = (idx).wrapping_add(1); old };
vm_block = 137; continue;
}
// C line 35939
139 => {
let _ = { let assigned = (*((*(fd)).scopes).offset((scope) as isize)).first; (*(vd_1)).scope_next = assigned; assigned };
vm_block = 138; continue;
}
// C line 35938
140 => {
let _ = { let assigned = (*((*(fd)).scopes).offset(((*(vd_1)).scope_level) as isize)).parent; scope = assigned; assigned };
vm_block = 139; continue;
}
// C line 35937
141 => {
vm_block = if (((((((((*(vd_1)).scope_next) < ((0 as i32))) as i32)) != 0) && ((((((*(vd_1)).scope_level) > ((1 as i32))) as i32)) != 0)) as i32)) != 0 { 140 } else { 138 }; continue;
}
// C line 35936
142 => {
vd_1 = core::ptr::addr_of_mut!(*((*(fd)).vars).offset((idx) as isize));
vm_block = 141; continue;
}
// C line 35935
143 => {
let _ = { let assigned = (0 as i32); idx = assigned; assigned };
vm_block = 137; continue;
}
// C line 35930
144 => {
vm_block = if ((((scope) < ((*(fd)).scope_count)) as i32)) != 0 { 148 } else { 143 }; continue;
}
// C line 35930
145 => {
let _ = { let old = scope; scope = (scope).wrapping_add(1); old };
vm_block = 144; continue;
}
// C line 35933
146 => {
let _ = { let assigned = (*((*(fd)).scopes).offset(((*(sd)).parent) as isize)).first; (*(sd)).first = assigned; assigned };
vm_block = 145; continue;
}
// C line 35932
147 => {
vm_block = if (((((*(sd)).first) < ((0 as i32))) as i32)) != 0 { 146 } else { 145 }; continue;
}
// C line 35931
148 => {
sd = core::ptr::addr_of_mut!(*((*(fd)).scopes).offset((scope) as isize));
vm_block = 147; continue;
}
// C line 35930
149 => {
let _ = { let assigned = (2 as i32); scope = assigned; assigned };
vm_block = 144; continue;
}
// C line 35925
150 => {
vm_block = if ((((idx) < ((*(fd)).var_count)) as i32)) != 0 { 154 } else { 149 }; continue;
}
// C line 35925
151 => {
let _ = { let old = idx; idx = (idx).wrapping_add(1); old };
vm_block = 150; continue;
}
// C line 35928
152 => {
let _ = { let assigned = idx; (*((*(fd)).scopes).offset(((*(vd)).scope_level) as isize)).first = assigned; assigned };
vm_block = 151; continue;
}
// C line 35927
153 => {
let _ = { let assigned = (*((*(fd)).scopes).offset(((*(vd)).scope_level) as isize)).first; (*(vd)).scope_next = assigned; assigned };
vm_block = 152; continue;
}
// C line 35926
154 => {
vd = core::ptr::addr_of_mut!(*((*(fd)).vars).offset((idx) as isize));
vm_block = 153; continue;
}
// C line 35925
155 => {
let _ = { let assigned = (0 as i32); idx = assigned; assigned };
vm_block = 150; continue;
}
// C line 35923
156 => {
let _ = { let assigned = ((2 as i32)).wrapping_neg(); (*((*(fd)).scopes).offset(((1 as i32)) as isize)).first = assigned; assigned };
vm_block = 155; continue;
}
// C line 35921
157 => {
vm_block = if ((*(fd)).has_parameter_expressions) != 0 { 156 } else { 155 }; continue;
}
// C line 35918
158 => {
vm_block = if ((((scope) < ((*(fd)).scope_count)) as i32)) != 0 { 160 } else { 157 }; continue;
}
// C line 35918
159 => {
let _ = { let old = scope; scope = (scope).wrapping_add(1); old };
vm_block = 158; continue;
}
// C line 35919
160 => {
let _ = { let assigned = ((1 as i32)).wrapping_neg(); (*((*(fd)).scopes).offset((scope) as isize)).first = assigned; assigned };
vm_block = 159; continue;
}
// C line 35918
161 => {
let _ = { let assigned = (0 as i32); scope = assigned; assigned };
vm_block = 158; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:36210. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_parse_directives(mut s: *mut JSParseState) -> i32 {
let mut vm_local_storage = Vec::<u64>::new();
let mut str: [c_char; 20] = core::mem::zeroed();
let mut pos: JSParsePos = core::mem::zeroed();
let mut has_semi: i32 = 0;
let mut vm_block: usize = 25;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 36293
1 => {
return js_parse_seek_token(s, core::ptr::addr_of_mut!(pos));
}
// C line 36221
2 => {
vm_block = if ((((((*(s)).token).val) == ((TOK_STRING as i32))) as i32)) != 0 { 22 } else { 1 }; continue;
}
// C line 36290
3 => {
let _ = { (*((*(s)).cur_func)).js_mode = (((((((*((*(s)).cur_func)).js_mode) as i32)) | (((1 as i32)).wrapping_shl(((0 as i32)) as u32)))) as u8) as u8; (*((*(s)).cur_func)).js_mode };
vm_block = 2; continue;
}
// C line 36289
4 => {
let _ = { let assigned = (1 as i32); (*((*(s)).cur_func)).has_use_strict = assigned; assigned };
vm_block = 3; continue;
}
// C line 36288
5 => {
vm_block = if ((!((parser_strcmp((str).as_mut_ptr(), c"use strict".as_ptr())) != 0) as i32)) != 0 { 4 } else { 2 }; continue;
}
// C line 36287
6 => {
vm_block = 1; continue;
}
// C line 36286
7 => {
vm_block = if ((!((has_semi) != 0) as i32)) != 0 { 6 } else { 5 }; continue;
}
// C line 36284
8 => {
vm_block = 7; continue;
}
// C line 36282
9 => {
vm_block = 7; continue;
}
// C line 36281
10 => {
let _ = { let assigned = (1 as i32); has_semi = assigned; assigned };
vm_block = 9; continue;
}
// C line 36280
11 => {
vm_block = if ((*(s)).got_lf) != 0 { 10 } else { 9 }; continue;
}
// C line 36239
12 => {
vm_block = 7; continue;
}
// C line 36238
13 => {
let _ = { let assigned = (1 as i32); has_semi = assigned; assigned };
vm_block = 12; continue;
}
// C line 36235
14 => {
vm_block = 7; continue;
}
// C line 36234
15 => {
let _ = { let assigned = (1 as i32); has_semi = assigned; assigned };
vm_block = 14; continue;
}
// C line 36233
16 => {
return ((1 as i32)).wrapping_neg();
}
// C line 36232
17 => {
vm_block = if (next_token(s)) != 0 { 16 } else { 15 }; continue;
}
// C line 36230
18 => {
vm_block = match ((*(s)).token).val { x if x == (TOK_STATIC as i32) => 11, x if x == (TOK_PUBLIC as i32) => 11, x if x == (TOK_PROTECTED as i32) => 11, x if x == (TOK_PRIVATE as i32) => 11, x if x == (TOK_PACKAGE as i32) => 11, x if x == (TOK_LET as i32) => 11, x if x == (TOK_INTERFACE as i32) => 11, x if x == (TOK_SUPER as i32) => 11, x if x == (TOK_IMPORT as i32) => 11, x if x == (TOK_EXPORT as i32) => 11, x if x == (TOK_ENUM as i32) => 11, x if x == (TOK_CONST as i32) => 11, x if x == (TOK_CLASS as i32) => 11, x if x == (TOK_WITH as i32) => 11, x if x == (TOK_DEBUGGER as i32) => 11, x if x == (TOK_FUNCTION as i32) => 11, x if x == (TOK_TRY as i32) => 11, x if x == (TOK_THROW as i32) => 11, x if x == (TOK_SWITCH as i32) => 11, x if x == (TOK_FOR as i32) => 11, x if x == (TOK_WHILE as i32) => 11, x if x == (TOK_DO as i32) => 11, x if x == (TOK_NEW as i32) => 11, x if x == (TOK_TYPEOF as i32) => 11, x if x == (TOK_DELETE as i32) => 11, x if x == (TOK_THIS as i32) => 11, x if x == (TOK_VAR as i32) => 11, x if x == (TOK_RETURN as i32) => 11, x if x == (TOK_IF as i32) => 11, x if x == (TOK_TRUE as i32) => 11, x if x == (TOK_FALSE as i32) => 11, x if x == (TOK_NULL as i32) => 11, x if x == (TOK_INC as i32) => 11, x if x == (TOK_DEC as i32) => 11, x if x == (TOK_REGEXP as i32) => 11, x if x == (TOK_IDENT as i32) => 11, x if x == (TOK_TEMPLATE as i32) => 11, x if x == (TOK_STRING as i32) => 11, x if x == (TOK_NUMBER as i32) => 11, x if x == (TOK_EOF as i32) => 13, x if x == (125 as i32) => 13, x if x == (59 as i32) => 17, _ => 8, }; continue;
}
// C line 36229
19 => {
let _ = { let assigned = (0 as i32); has_semi = assigned; assigned };
vm_block = 18; continue;
}
// C line 36227
20 => {
return ((1 as i32)).wrapping_neg();
}
// C line 36226
21 => {
vm_block = if (next_token(s)) != 0 { 20 } else { 19 }; continue;
}
// C line 36223
22 => {
let _ = parser_copy_directive((str).as_mut_ptr(), (size_of::<[c_char; 20]>() as usize), ((((*(s)).token).ptr).offset((((1 as i32)) as isize))) as *const c_char, ((((((*(s)).buf_ptr).offset_from(((*(s)).token).ptr) as i64)).wrapping_sub((((2 as i32)) as i64))) as i32));
vm_block = 21; continue;
}
// C line 36219
23 => {
let _ = js_parse_get_pos(s, core::ptr::addr_of_mut!(pos));
vm_block = 2; continue;
}
// C line 36217
24 => {
return (0 as i32);
}
// C line 36216
25 => {
vm_block = if ((((((*(s)).token).val) != ((TOK_STRING as i32))) as i32)) != 0 { 24 } else { 23 }; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:36297. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn is_strict_future_keyword(mut atom: JSAtom) -> i32 {
let mut vm_local_storage = Vec::<u64>::new();
let mut vm_block: usize = 1;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 36299
1 => {
return (((((((atom) >= (((((crate::quickjs_atom::JS_ATOM_super as i32)).wrapping_add((1 as i32))) as JSAtom))) as i32)) != 0) && (((((atom) <= ((((crate::quickjs_atom::JS_ATOM_yield as i32)) as JSAtom))) as i32)) != 0)) as i32);
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:36302. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_parse_function_check_names(mut s: *mut JSParseState, mut fd: *mut JSFunctionDef, mut func_name: JSAtom) -> i32 {
let mut vm_local_storage = Vec::<u64>::new();
let mut name: JSAtom = 0;
let mut i: i32 = 0;
let mut idx: i32 = 0;
let mut vm_block: usize = 29;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 36351 labels: duplicate
1 => {
return js_parse_error(s, c"duplicate argument names not allowed in this context".as_ptr());
}
// C line 36348
2 => {
return (0 as i32);
}
// C line 36331
3 => {
vm_block = if ((((idx) < ((*(fd)).arg_count)) as i32)) != 0 { 16 } else { 2 }; continue;
}
// C line 36331
4 => {
let _ = { let old = idx; idx = (idx).wrapping_add(1); old };
vm_block = 3; continue;
}
// C line 36340
5 => {
vm_block = if ((((i) < ((*(fd)).var_count)) as i32)) != 0 { 8 } else { 4 }; continue;
}
// C line 36340
6 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 5; continue;
}
// C line 36343
7 => {
vm_block = 1; continue;
}
// C line 36341
8 => {
vm_block = if (((((((((*((*(fd)).vars).offset((i) as isize)).var_name) == (name)) as i32)) != 0) && ((((((*((*(fd)).vars).offset((i) as isize)).scope_level) == ((0 as i32))) as i32)) != 0)) as i32)) != 0 { 7 } else { 6 }; continue;
}
// C line 36340
9 => {
let _ = { let assigned = (0 as i32); i = assigned; assigned };
vm_block = 5; continue;
}
// C line 36334
10 => {
vm_block = if ((((i) < (idx)) as i32)) != 0 { 13 } else { 9 }; continue;
}
// C line 36334
11 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 10; continue;
}
// C line 36336
12 => {
vm_block = 1; continue;
}
// C line 36335
13 => {
vm_block = if (((((*((*(fd)).args).offset((i) as isize)).var_name) == (name)) as i32)) != 0 { 12 } else { 11 }; continue;
}
// C line 36334
14 => {
let _ = { let assigned = (0 as i32); i = assigned; assigned };
vm_block = 10; continue;
}
// C line 36333
15 => {
vm_block = if ((((name) != ((((0 as i32)) as JSAtom))) as i32)) != 0 { 14 } else { 4 }; continue;
}
// C line 36332
16 => {
let _ = { let assigned = (*((*(fd)).args).offset((idx) as isize)).var_name; name = assigned; assigned };
vm_block = 15; continue;
}
// C line 36331
17 => {
let _ = { let assigned = (0 as i32); idx = assigned; assigned };
vm_block = 3; continue;
}
// C line 36326
18 => {
vm_block = if ((((((((((((((((((((((*(fd)).js_mode) as i32)) & (((1 as i32)).wrapping_shl(((0 as i32)) as u32)))) != 0) || (((!(((*(fd)).has_simple_parameter_list) != 0) as i32)) != 0)) as i32)) != 0) || (((((((((((((*(fd)).func_type as JSParseFunctionEnum)) as i32)) == ((JS_PARSE_FUNC_METHOD as i32))) as i32)) != 0) && (((((((((*(fd)).func_kind as JSFunctionKindEnum)) as i32)) == ((JS_FUNC_ASYNC as i32))) as i32)) != 0)) as i32)) != 0)) as i32)) != 0) || (((((((((*(fd)).func_type as JSParseFunctionEnum)) as i32)) == ((JS_PARSE_FUNC_ARROW as i32))) as i32)) != 0)) as i32)) != 0) || (((((((((*(fd)).func_type as JSParseFunctionEnum)) as i32)) == ((JS_PARSE_FUNC_METHOD as i32))) as i32)) != 0)) as i32)) != 0 { 17 } else { 2 }; continue;
}
// C line 36316
19 => {
vm_block = if ((((idx) < ((*(fd)).arg_count)) as i32)) != 0 { 23 } else { 18 }; continue;
}
// C line 36316
20 => {
let _ = { let old = idx; idx = (idx).wrapping_add(1); old };
vm_block = 19; continue;
}
// C line 36321
21 => {
return js_parse_error(s, c"invalid argument name in strict code".as_ptr());
}
// C line 36319
22 => {
vm_block = if ((((((((((((name) == ((((crate::quickjs_atom::JS_ATOM_eval as i32)) as JSAtom))) as i32)) != 0) || (((((name) == ((((crate::quickjs_atom::JS_ATOM_arguments as i32)) as JSAtom))) as i32)) != 0)) as i32)) != 0) || ((is_strict_future_keyword(name)) != 0)) as i32)) != 0 { 21 } else { 20 }; continue;
}
// C line 36317
23 => {
let _ = { let assigned = (*((*(fd)).args).offset((idx) as isize)).var_name; name = assigned; assigned };
vm_block = 22; continue;
}
// C line 36316
24 => {
let _ = { let assigned = (0 as i32); idx = assigned; assigned };
vm_block = 19; continue;
}
// C line 36314
25 => {
return js_parse_error(s, c"invalid function name in strict code".as_ptr());
}
// C line 36312
26 => {
vm_block = if ((((((((((((func_name) == ((((crate::quickjs_atom::JS_ATOM_eval as i32)) as JSAtom))) as i32)) != 0) || (((((func_name) == ((((crate::quickjs_atom::JS_ATOM_arguments as i32)) as JSAtom))) as i32)) != 0)) as i32)) != 0) || ((is_strict_future_keyword(func_name)) != 0)) as i32)) != 0 { 25 } else { 24 }; continue;
}
// C line 36310
27 => {
return js_parse_error(s, c"\"use strict\" not allowed in function with default or destructuring parameter".as_ptr());
}
// C line 36309
28 => {
vm_block = if ((((((!(((*(fd)).has_simple_parameter_list) != 0) as i32)) != 0) && (((*(fd)).has_use_strict) != 0)) as i32)) != 0 { 27 } else { 26 }; continue;
}
// C line 36308
29 => {
vm_block = if ((((((*(fd)).js_mode) as i32)) & (((1 as i32)).wrapping_shl(((0 as i32)) as u32)))) != 0 { 28 } else { 18 }; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:36355. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_parse_function_class_fields_init(mut s: *mut JSParseState) -> *mut JSFunctionDef {
let mut vm_local_storage = Vec::<u64>::new();
let mut fd: *mut JSFunctionDef = core::ptr::null_mut();
let mut vm_block: usize = 16;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 36378
1 => {
return fd;
}
// C line 36377
2 => {
let _ = { let assigned = (((JS_PARSE_FUNC_METHOD as i32)) as JSParseFunctionEnum); (*(fd)).func_type = (assigned) as u8; assigned };
vm_block = 1; continue;
}
// C line 36376
3 => {
let _ = { let assigned = (((JS_FUNC_NORMAL as i32)) as JSFunctionKindEnum); (*(fd)).func_kind = (assigned) as u8; assigned };
vm_block = 2; continue;
}
// C line 36374
4 => {
let _ = { let assigned = (0 as i32); (*(fd)).arguments_allowed = assigned; assigned };
vm_block = 3; continue;
}
// C line 36373
5 => {
let _ = { let assigned = (*(fd)).has_home_object; (*(fd)).super_allowed = assigned; assigned };
vm_block = 4; continue;
}
// C line 36372
6 => {
let _ = { let assigned = (0 as i32); (*(fd)).super_call_allowed = assigned; assigned };
vm_block = 5; continue;
}
// C line 36371
7 => {
let _ = { let assigned = (1 as i32); (*(fd)).new_target_allowed = assigned; assigned };
vm_block = 6; continue;
}
// C line 36370
8 => {
let _ = { let assigned = (0 as i32); (*(fd)).is_derived_class_constructor = assigned; assigned };
vm_block = 7; continue;
}
// C line 36369
9 => {
let _ = { let assigned = (1 as i32); (*(fd)).has_this_binding = assigned; assigned };
vm_block = 8; continue;
}
// C line 36368
10 => {
let _ = { let assigned = (0 as i32); (*(fd)).has_arguments_binding = assigned; assigned };
vm_block = 9; continue;
}
// C line 36366
11 => {
let _ = { let assigned = (1 as i32); (*(fd)).has_home_object = assigned; assigned };
vm_block = 10; continue;
}
// C line 36365
12 => {
let _ = { let assigned = (0 as i32); (*(fd)).has_prototype = assigned; assigned };
vm_block = 11; continue;
}
// C line 36364
13 => {
let _ = { let assigned = (((0 as i32)) as JSAtom); (*(fd)).func_name = assigned; assigned };
vm_block = 12; continue;
}
// C line 36363
14 => {
return core::ptr::null_mut::<JSFunctionDef>();
}
// C line 36362
15 => {
vm_block = if ((!(!(fd).is_null()) as i32)) != 0 { 14 } else { 13 }; continue;
}
// C line 36359
16 => {
let _ = { let assigned = js_new_function_def((*(s)).ctx, (*(s)).cur_func, (0 as i32), (0 as i32), (*(s)).filename, (*(s)).buf_start, core::ptr::addr_of_mut!((*(s)).get_line_col_cache)); fd = assigned; assigned };
vm_block = 15; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:36383. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_parse_function_decl2(mut s: *mut JSParseState, mut func_type: JSParseFunctionEnum, mut func_kind: JSFunctionKindEnum, mut func_name: JSAtom, mut ptr: *const u8, mut export_flag: JSParseExportEnum, mut pfd: *mut *mut JSFunctionDef) -> i32 {
let mut vm_local_storage = Vec::<u64>::new();
let mut ctx: *mut JSContext = core::ptr::null_mut();
let mut fd: *mut JSFunctionDef = core::ptr::null_mut();
let mut is_expr: i32 = 0;
let mut func_idx: i32 = 0;
let mut lexical_func_idx: i32 = 0;
let mut has_opt_arg: i32 = 0;
let mut create_func_var: i32 = 0;
let mut hf: *mut JSGlobalVar = core::ptr::null_mut();
let mut hf_1: *mut JSGlobalVar = core::ptr::null_mut();
let mut name: JSAtom = 0;
let mut skip_bits: i32 = 0;
let mut name_1: JSAtom = 0;
let mut rest: i32 = 0;
let mut idx: i32 = 0;
let mut has_initializer: i32 = 0;
let mut label: i32 = 0;
let mut idx_1: i32 = 0;
let mut vd: *mut JSVarDef = core::ptr::null_mut();
let mut idx_2: i32 = 0;
let mut func_name_1: JSAtom = 0;
let mut hf_2: *mut JSGlobalVar = core::ptr::null_mut();
let mut var_idx: i32 = 0;
let mut func_var_name: JSAtom = 0;
let mut hf_3: *mut JSGlobalVar = core::ptr::null_mut();
let mut vm_block: usize = 329;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 36948
1 => {
return ((1 as i32)).wrapping_neg();
}
// C line 36947
2 => {
let _ = { let assigned = core::ptr::null_mut::<JSFunctionDef>(); *(pfd) = assigned; assigned };
vm_block = 1; continue;
}
// C line 36946
3 => {
vm_block = if !(pfd).is_null() { 2 } else { 1 }; continue;
}
// C line 36945
4 => {
let _ = js_free_function_def(ctx, fd);
vm_block = 3; continue;
}
// C line 36944 labels: fail
5 => {
let _ = { let assigned = (*(fd)).parent; (*(s)).cur_func = assigned; assigned };
vm_block = 4; continue;
}
// C line 36942
6 => {
return (0 as i32);
}
// C line 36860
7 => {
let _ = emit_u32(s, (((0 as i32)) as u32));
vm_block = 6; continue;
}
// C line 36859
8 => {
let _ = emit_op(s, (((OP_set_name as i32)) as u8));
vm_block = 7; continue;
}
// C line 36858
9 => {
vm_block = if ((((func_name_1) == ((((0 as i32)) as JSAtom))) as i32)) != 0 { 8 } else { 6 }; continue;
}
// C line 36857
10 => {
let _ = emit_u32(s, ((idx_2) as u32));
vm_block = 9; continue;
}
// C line 36856
11 => {
let _ = emit_op(s, (((OP_fclosure as i32)) as u8));
vm_block = 10; continue;
}
// C line 36852
12 => {
vm_block = if ((((((((((func_type) as u32)) != ((((JS_PARSE_FUNC_CLASS_CONSTRUCTOR as i32)) as u32))) as i32)) != 0) && (((((((func_type) as u32)) != ((((JS_PARSE_FUNC_DERIVED_CLASS_CONSTRUCTOR as i32)) as u32))) as i32)) != 0)) as i32)) != 0 { 11 } else { 6 }; continue;
}
// C line 36902
13 => {
let _ = emit_op(s, (((OP_drop as i32)) as u8));
vm_block = 6; continue;
}
// C line 36901
14 => {
let _ = { let assigned = idx_2; (*((*((*(s)).cur_func)).vars).offset((lexical_func_idx) as isize)).func_pool_idx = assigned; assigned };
vm_block = 13; continue;
}
// C line 36908
15 => {
let _ = emit_u16(s, (((*((*(s)).cur_func)).scope_level) as u16));
vm_block = 6; continue;
}
// C line 36907
16 => {
let _ = emit_atom(s, func_name_1);
vm_block = 15; continue;
}
// C line 36906
17 => {
let _ = emit_op(s, (((OP_scope_put_var_init as i32)) as u8));
vm_block = 16; continue;
}
// C line 36899
18 => {
vm_block = if ((((lexical_func_idx) >= ((0 as i32))) as i32)) != 0 { 14 } else { 17 }; continue;
}
// C line 36883
19 => {
let _ = emit_u16(s, (((0 as i32)) as u16));
vm_block = 18; continue;
}
// C line 36882
20 => {
let _ = emit_atom(s, func_name_1);
vm_block = 19; continue;
}
// C line 36881
21 => {
let _ = emit_op(s, (((OP_scope_put_var as i32)) as u8));
vm_block = 20; continue;
}
// C line 36880
22 => {
let _ = emit_op(s, (((OP_dup as i32)) as u8));
vm_block = 21; continue;
}
// C line 36878
23 => {
let _ = { let assigned = ((((((((((*((*(s)).cur_func)).js_mode) as i32)) & (((1 as i32)).wrapping_shl(((0 as i32)) as u32)))) != ((0 as i32))) as i32)) as u8); (*(hf_2)).set_force_init((assigned) as _); assigned };
vm_block = 22; continue;
}
// C line 36877
24 => {
let _ = { let assigned = (0 as i32); (*(hf_2)).scope_level = assigned; assigned };
vm_block = 23; continue;
}
// C line 36873
25 => {
vm_block = 5; continue;
}
// C line 36872
26 => {
vm_block = if ((!(!(hf_2).is_null()) as i32)) != 0 { 25 } else { 24 }; continue;
}
// C line 36871
27 => {
let _ = { let assigned = add_global_var(ctx, (*(s)).cur_func, func_name_1); hf_2 = assigned; assigned };
vm_block = 26; continue;
}
// C line 36896
28 => {
let _ = emit_u16(s, (((0 as i32)) as u16));
vm_block = 18; continue;
}
// C line 36895
29 => {
let _ = emit_atom(s, func_name_1);
vm_block = 28; continue;
}
// C line 36894
30 => {
let _ = emit_op(s, (((OP_scope_put_var as i32)) as u8));
vm_block = 29; continue;
}
// C line 36893
31 => {
let _ = emit_op(s, (((OP_dup as i32)) as u8));
vm_block = 30; continue;
}
// C line 36890
32 => {
vm_block = 5; continue;
}
// C line 36889
33 => {
vm_block = if ((((func_idx) < ((0 as i32))) as i32)) != 0 { 32 } else { 31 }; continue;
}
// C line 36888
34 => {
let _ = { let assigned = add_var(ctx, (*(s)).cur_func, func_name_1); func_idx = assigned; assigned };
vm_block = 33; continue;
}
// C line 36887
35 => {
vm_block = if ((((func_idx) < ((0 as i32))) as i32)) != 0 { 34 } else { 31 }; continue;
}
// C line 36886
36 => {
let _ = { let assigned = find_var(ctx, (*(s)).cur_func, func_name_1); func_idx = assigned; assigned };
vm_block = 35; continue;
}
// C line 36867
37 => {
vm_block = if ((*((*(s)).cur_func)).is_global_var) != 0 { 27 } else { 36 }; continue;
}
// C line 36866
38 => {
vm_block = if (create_func_var) != 0 { 37 } else { 18 }; continue;
}
// C line 36865
39 => {
let _ = emit_u32(s, ((idx_2) as u32));
vm_block = 38; continue;
}
// C line 36864
40 => {
let _ = emit_op(s, (((OP_fclosure as i32)) as u8));
vm_block = 39; continue;
}
// C line 36918
41 => {
let _ = { let assigned = idx_2; (*((*((*(s)).cur_func)).args).offset(((var_idx).wrapping_sub((536870912 as i32))) as isize)).func_pool_idx = assigned; assigned };
vm_block = 6; continue;
}
// C line 36920
42 => {
let _ = { let assigned = idx_2; (*((*((*(s)).cur_func)).vars).offset((var_idx) as isize)).func_pool_idx = assigned; assigned };
vm_block = 6; continue;
}
// C line 36917
43 => {
vm_block = if (((var_idx) & ((536870912 as i32)))) != 0 { 41 } else { 42 }; continue;
}
// C line 36915
44 => {
vm_block = 5; continue;
}
// C line 36914
45 => {
vm_block = if ((((var_idx) < ((0 as i32))) as i32)) != 0 { 44 } else { 43 }; continue;
}
// C line 36912
46 => {
var_idx = define_var(s, (*(s)).cur_func, func_name_1, (((JS_VAR_DEF_VAR as i32)) as JSVarDefEnum));
vm_block = 45; continue;
}
// C line 36937
47 => {
vm_block = 5; continue;
}
// C line 36935
48 => {
vm_block = if ((!(!(add_export_entry(s, (*((*(s)).cur_func)).module, func_var_name, if ((((((export_flag) as u32)) == ((((JS_PARSE_EXPORT_NAMED as i32)) as u32))) as i32)) != 0 { func_var_name } else { (((crate::quickjs_atom::JS_ATOM_default as i32)) as JSAtom) }, (((JS_EXPORT_TYPE_LOCAL as i32)) as JSExportTypeEnum))).is_null()) as i32)) != 0 { 47 } else { 6 }; continue;
}
// C line 36934
49 => {
vm_block = if ((((((export_flag) as u32)) != ((((JS_PARSE_EXPORT_NONE as i32)) as u32))) as i32)) != 0 { 48 } else { 6 }; continue;
}
// C line 36933
50 => {
let _ = { let assigned = idx_2; (*(hf_3)).cpool_idx = assigned; assigned };
vm_block = 49; continue;
}
// C line 36932
51 => {
vm_block = 5; continue;
}
// C line 36931
52 => {
vm_block = if ((!(!(hf_3).is_null()) as i32)) != 0 { 51 } else { 50 }; continue;
}
// C line 36930
53 => {
let _ = { let assigned = add_global_var(ctx, (*(s)).cur_func, func_var_name); hf_3 = assigned; assigned };
vm_block = 52; continue;
}
// C line 36926
54 => {
let _ = { let assigned = (((crate::quickjs_atom::JS_ATOM__default_ as i32)) as JSAtom); func_var_name = assigned; assigned };
vm_block = 53; continue;
}
// C line 36928
55 => {
let _ = { let assigned = func_name_1; func_var_name = assigned; assigned };
vm_block = 53; continue;
}
// C line 36925
56 => {
vm_block = if ((((func_name_1) == ((((0 as i32)) as JSAtom))) as i32)) != 0 { 54 } else { 55 }; continue;
}
// C line 36911
57 => {
vm_block = if ((!(((*((*(s)).cur_func)).is_global_var) != 0) as i32)) != 0 { 46 } else { 56 }; continue;
}
// C line 36863
58 => {
vm_block = if ((((((func_type) as u32)) == ((((JS_PARSE_FUNC_VAR as i32)) as u32))) as i32)) != 0 { 40 } else { 57 }; continue;
}
// C line 36850
59 => {
vm_block = if (is_expr) != 0 { 12 } else { 58 }; continue;
}
// C line 36848
60 => {
let _ = { let assigned = idx_2; (*(fd)).parent_cpool_idx = assigned; assigned };
vm_block = 59; continue;
}
// C line 36847
61 => {
let _ = { let assigned = cpool_add(s, JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_NULL as i32)) as i64) }); idx_2 = assigned; assigned };
vm_block = 60; continue;
}
// C line 36844
62 => {
func_name_1 = (*(fd)).func_name;
vm_block = 61; continue;
}
// C line 36839
63 => {
let _ = reparse_ident_token(s);
vm_block = 62; continue;
}
// C line 36833 labels: done
64 => {
let _ = { let assigned = (*(fd)).parent; (*(s)).cur_func = assigned; assigned };
vm_block = 63; continue;
}
// C line 36830
65 => {
let _ = emit_return(s, (0 as i32));
vm_block = 64; continue;
}
// C line 36829
66 => {
vm_block = if (js_is_live_code(s)) != 0 { 65 } else { 64 }; continue;
}
// C line 36825
67 => {
vm_block = 5; continue;
}
// C line 36823
68 => {
vm_block = if (next_token(s)) != 0 { 67 } else { 66 }; continue;
}
// C line 36820
69 => {
vm_block = 5; continue;
}
// C line 36819
70 => {
vm_block = if ((!(!((*(fd)).source).is_null()) as i32)) != 0 { 69 } else { 68 }; continue;
}
// C line 36818
71 => {
let _ = { let assigned = js_strndup(ctx, ((ptr) as *const c_char), (((*(fd)).source_len) as usize)); (*(fd)).source = assigned; assigned };
vm_block = 70; continue;
}
// C line 36817
72 => {
let _ = { let assigned = (((((*(s)).buf_ptr).offset_from(ptr) as i64)) as i32); (*(fd)).source_len = assigned; assigned };
vm_block = 71; continue;
}
// C line 36815
73 => {
vm_block = if ((!(((*(fd)).strip_source()) != 0) as i32)) != 0 { 72 } else { 68 }; continue;
}
// C line 36811
74 => {
vm_block = if ((((((*(s)).token).val) != ((125 as i32))) as i32)) != 0 { 76 } else { 73 }; continue;
}
// C line 36813
75 => {
vm_block = 5; continue;
}
// C line 36812
76 => {
vm_block = if (js_parse_source_element(s)) != 0 { 75 } else { 74 }; continue;
}
// C line 36809
77 => {
vm_block = 5; continue;
}
// C line 36808
78 => {
vm_block = if (js_parse_function_check_names(s, fd, func_name)) != 0 { 77 } else { 74 }; continue;
}
// C line 36805
79 => {
vm_block = 5; continue;
}
// C line 36804
80 => {
vm_block = if (js_parse_directives(s)) != 0 { 79 } else { 78 }; continue;
}
// C line 36801
81 => {
vm_block = 5; continue;
}
// C line 36800
82 => {
vm_block = if (js_parse_expect(s, (123 as i32))) != 0 { 81 } else { 80 }; continue;
}
// C line 36799
83 => {
vm_block = if ((((((func_type) as u32)) != ((((JS_PARSE_FUNC_CLASS_STATIC_INIT as i32)) as u32))) as i32)) != 0 { 82 } else { 80 }; continue;
}
// C line 36795
84 => {
vm_block = 64; continue;
}
// C line 36793
85 => {
vm_block = 5; continue;
}
// C line 36792
86 => {
vm_block = if ((!(!((*(fd)).source).is_null()) as i32)) != 0 { 85 } else { 84 }; continue;
}
// C line 36791
87 => {
let _ = { let assigned = js_strndup(ctx, ((ptr) as *const c_char), (((*(fd)).source_len) as usize)); (*(fd)).source = assigned; assigned };
vm_block = 86; continue;
}
// C line 36790
88 => {
let _ = { let assigned = (((((*(s)).last_ptr).offset_from(ptr) as i64)) as i32); (*(fd)).source_len = assigned; assigned };
vm_block = 87; continue;
}
// C line 36786
89 => {
vm_block = if ((!(((*(fd)).strip_source()) != 0) as i32)) != 0 { 88 } else { 84 }; continue;
}
// C line 36782
90 => {
let _ = emit_op(s, (((OP_return_async as i32)) as u8));
vm_block = 89; continue;
}
// C line 36784
91 => {
let _ = emit_op(s, (((OP_return as i32)) as u8));
vm_block = 89; continue;
}
// C line 36781
92 => {
vm_block = if ((((((func_kind) as u32)) != ((((JS_FUNC_NORMAL as i32)) as u32))) as i32)) != 0 { 90 } else { 91 }; continue;
}
// C line 36779
93 => {
vm_block = 5; continue;
}
// C line 36778
94 => {
vm_block = if (js_parse_assign_expr(s)) != 0 { 93 } else { 92 }; continue;
}
// C line 36776
95 => {
vm_block = 5; continue;
}
// C line 36775
96 => {
vm_block = if (js_parse_function_check_names(s, fd, func_name)) != 0 { 95 } else { 94 }; continue;
}
// C line 36774
97 => {
vm_block = if ((((((*(s)).token).val) != ((123 as i32))) as i32)) != 0 { 96 } else { 83 }; continue;
}
// C line 36772
98 => {
vm_block = 5; continue;
}
// C line 36771
99 => {
vm_block = if (next_token(s)) != 0 { 98 } else { 97 }; continue;
}
// C line 36770
100 => {
vm_block = if ((((((((((*(s)).token).val) == ((TOK_ARROW as i32))) as i32)) != 0) && (((((((func_type) as u32)) == ((((JS_PARSE_FUNC_ARROW as i32)) as u32))) as i32)) != 0)) as i32)) != 0 { 99 } else { 83 }; continue;
}
// C line 36768
101 => {
let _ = { let assigned = (*(fd)).scope_level; (*(fd)).body_scope = assigned; assigned };
vm_block = 100; continue;
}
// C line 36767
102 => {
let _ = push_scope(s);
vm_block = 101; continue;
}
// C line 36766
103 => {
let _ = { let assigned = (1 as i32); (*(fd)).in_function_body = assigned; assigned };
vm_block = 102; continue;
}
// C line 36762
104 => {
let _ = emit_op(s, (((OP_initial_yield as i32)) as u8));
vm_block = 103; continue;
}
// C line 36760
105 => {
vm_block = if ((((((((((func_kind) as u32)) == ((((JS_FUNC_GENERATOR as i32)) as u32))) as i32)) != 0) || (((((((func_kind) as u32)) == ((((JS_FUNC_ASYNC_GENERATOR as i32)) as u32))) as i32)) != 0)) as i32)) != 0 { 104 } else { 103 }; continue;
}
// C line 36757
106 => {
vm_block = 5; continue;
}
// C line 36756
107 => {
vm_block = if (next_token(s)) != 0 { 106 } else { 105 }; continue;
}
// C line 36753
108 => {
let _ = { let assigned = (*((*(fd)).scopes).offset(((*(fd)).scope_level) as isize)).first; (*(fd)).scope_first = assigned; assigned };
vm_block = 107; continue;
}
// C line 36752
109 => {
let _ = { let assigned = (0 as i32); (*(fd)).scope_level = assigned; assigned };
vm_block = 108; continue;
}
// C line 36749
110 => {
let _ = emit_u16(s, (((*(fd)).scope_level) as u16));
vm_block = 109; continue;
}
// C line 36748
111 => {
let _ = emit_op(s, (((OP_leave_scope as i32)) as u8));
vm_block = 110; continue;
}
// C line 36729
112 => {
vm_block = if ((((idx_1) >= ((0 as i32))) as i32)) != 0 { 126 } else { 111 }; continue;
}
// C line 36744
113 => {
let _ = { let assigned = (*(vd)).scope_next; idx_1 = assigned; assigned };
vm_block = 112; continue;
}
// C line 36742
114 => {
let _ = emit_u16(s, (((0 as i32)) as u16));
vm_block = 113; continue;
}
// C line 36741
115 => {
let _ = emit_atom(s, (*(vd)).var_name);
vm_block = 114; continue;
}
// C line 36740
116 => {
let _ = emit_op(s, (((OP_scope_put_var as i32)) as u8));
vm_block = 115; continue;
}
// C line 36739
117 => {
let _ = emit_u16(s, (((*(fd)).scope_level) as u16));
vm_block = 116; continue;
}
// C line 36738
118 => {
let _ = emit_atom(s, (*(vd)).var_name);
vm_block = 117; continue;
}
// C line 36737
119 => {
let _ = emit_op(s, (((OP_scope_get_var as i32)) as u8));
vm_block = 118; continue;
}
// C line 36736
120 => {
let _ = { let assigned = core::ptr::addr_of_mut!(*((*(fd)).vars).offset((idx_1) as isize)); vd = assigned; assigned };
vm_block = 119; continue;
}
// C line 36735
121 => {
vm_block = 5; continue;
}
// C line 36734
122 => {
vm_block = if ((((add_var(ctx, fd, (*(vd)).var_name)) < ((0 as i32))) as i32)) != 0 { 121 } else { 120 }; continue;
}
// C line 36733
123 => {
vm_block = if ((((find_var(ctx, fd, (*(vd)).var_name)) < ((0 as i32))) as i32)) != 0 { 122 } else { 113 }; continue;
}
// C line 36732
124 => {
vm_block = 111; continue;
}
// C line 36731
125 => {
vm_block = if (((((*(vd)).scope_level) != ((*(fd)).scope_level)) as i32)) != 0 { 124 } else { 123 }; continue;
}
// C line 36730
126 => {
vd = core::ptr::addr_of_mut!(*((*(fd)).vars).offset((idx_1) as isize));
vm_block = 125; continue;
}
// C line 36728
127 => {
let _ = { let assigned = (*((*(fd)).scopes).offset(((*(fd)).scope_level) as isize)).first; idx_1 = assigned; assigned };
vm_block = 112; continue;
}
// C line 36721
128 => {
vm_block = if ((*(fd)).has_parameter_expressions) != 0 { 127 } else { 107 }; continue;
}
// C line 36572
129 => {
let _ = { let assigned = (1 as i32); (*(fd)).defined_arg_count = assigned; assigned };
vm_block = 128; continue;
}
// C line 36571
130 => {
vm_block = 5; continue;
}
// C line 36570
131 => {
vm_block = if ((((add_arg(ctx, fd, name)) < ((0 as i32))) as i32)) != 0 { 130 } else { 129 }; continue;
}
// C line 36569
132 => {
let _ = { let assigned = ((((*(s)).token).u).ident).atom; name = assigned; assigned };
vm_block = 131; continue;
}
// C line 36567
133 => {
vm_block = 5; continue;
}
// C line 36566
134 => {
let _ = js_parse_error_reserved_identifier(s);
vm_block = 133; continue;
}
// C line 36565
135 => {
vm_block = if (((((*(s)).token).u).ident).is_reserved) != 0 { 134 } else { 132 }; continue;
}
// C line 36717
136 => {
vm_block = 5; continue;
}
// C line 36716 labels: fail_accessor
137 => {
let _ = js_parse_error(s, c"invalid number of arguments for getter or setter".as_ptr());
vm_block = 136; continue;
}
// C line 36713
138 => {
vm_block = if ((((((((((((((func_type) as u32)) == ((((JS_PARSE_FUNC_GETTER as i32)) as u32))) as i32)) != 0) && ((((((*(fd)).arg_count) != ((0 as i32))) as i32)) != 0)) as i32)) != 0) || (((((((((((func_type) as u32)) == ((((JS_PARSE_FUNC_SETTER as i32)) as u32))) as i32)) != 0) && ((((((*(fd)).arg_count) != ((1 as i32))) as i32)) != 0)) as i32)) != 0)) as i32)) != 0 { 137 } else { 128 }; continue;
}
// C line 36594
139 => {
vm_block = if ((((((*(s)).token).val) != ((41 as i32))) as i32)) != 0 { 232 } else { 138 }; continue;
}
// C line 36711
140 => {
vm_block = 5; continue;
}
// C line 36710
141 => {
vm_block = if (js_parse_expect(s, (44 as i32))) != 0 { 140 } else { 139 }; continue;
}
// C line 36709
142 => {
vm_block = 138; continue;
}
// C line 36708
143 => {
vm_block = if ((((((*(s)).token).val) == ((41 as i32))) as i32)) != 0 { 142 } else { 141 }; continue;
}
// C line 36706
144 => {
vm_block = 5; continue;
}
// C line 36705
145 => {
let _ = js_parse_expect(s, (41 as i32));
vm_block = 144; continue;
}
// C line 36704
146 => {
vm_block = if (((((rest) != 0) && (((((((*(s)).token).val) != ((41 as i32))) as i32)) != 0)) as i32)) != 0 { 145 } else { 143 }; continue;
}
// C line 36624
147 => {
let _ = { let old = (*(fd)).defined_arg_count; (*(fd)).defined_arg_count = ((*(fd)).defined_arg_count).wrapping_add(1); old };
vm_block = 146; continue;
}
// C line 36623
148 => {
vm_block = if ((!((has_opt_arg) != 0) as i32)) != 0 { 147 } else { 146 }; continue;
}
// C line 36622
149 => {
let _ = { let assigned = (1 as i32); has_opt_arg = assigned; assigned };
vm_block = 148; continue;
}
// C line 36621
150 => {
vm_block = if (has_initializer) != 0 { 149 } else { 148 }; continue;
}
// C line 36620
151 => {
vm_block = 5; continue;
}
// C line 36619
152 => {
vm_block = if ((((has_initializer) < ((0 as i32))) as i32)) != 0 { 151 } else { 150 }; continue;
}
// C line 36618
153 => {
let _ = { let assigned = js_parse_destructuring_element(s, if ((*(fd)).has_parameter_expressions) != 0 { (TOK_LET as i32) } else { (TOK_VAR as i32) }, (1 as i32), (1 as i32), ((1 as i32)).wrapping_neg(), (1 as i32), (0 as i32)); has_initializer = assigned; assigned };
vm_block = 152; continue;
}
// C line 36611
154 => {
let _ = emit_u16(s, (((*(fd)).arg_count) as u16));
vm_block = 153; continue;
}
// C line 36610
155 => {
let _ = emit_op(s, (((OP_rest as i32)) as u8));
vm_block = 154; continue;
}
// C line 36616
156 => {
let _ = emit_u16(s, ((idx) as u16));
vm_block = 153; continue;
}
// C line 36615
157 => {
let _ = emit_op(s, (((OP_get_arg as i32)) as u8));
vm_block = 156; continue;
}
// C line 36614
158 => {
let _ = { let assigned = add_arg(ctx, fd, (((0 as i32)) as JSAtom)); idx = assigned; assigned };
vm_block = 157; continue;
}
// C line 36609
159 => {
vm_block = if (rest) != 0 { 155 } else { 158 }; continue;
}
// C line 36608
160 => {
let _ = { let assigned = (0 as i32); (*(fd)).has_simple_parameter_list = assigned; assigned };
vm_block = 159; continue;
}
// C line 36659
161 => {
let _ = { let assigned = (1 as i32); has_opt_arg = assigned; assigned };
vm_block = 146; continue;
}
// C line 36658
162 => {
let _ = { let assigned = (0 as i32); (*(fd)).has_simple_parameter_list = assigned; assigned };
vm_block = 161; continue;
}
// C line 36657
163 => {
let _ = emit_u16(s, ((idx) as u16));
vm_block = 162; continue;
}
// C line 36656
164 => {
let _ = emit_op(s, (((OP_put_arg as i32)) as u8));
vm_block = 163; continue;
}
// C line 36654
165 => {
let _ = emit_u16(s, (((*(fd)).scope_level) as u16));
vm_block = 164; continue;
}
// C line 36653
166 => {
let _ = emit_atom(s, name_1);
vm_block = 165; continue;
}
// C line 36652
167 => {
let _ = emit_op(s, (((OP_scope_put_var_init as i32)) as u8));
vm_block = 166; continue;
}
// C line 36651
168 => {
let _ = emit_op(s, (((OP_dup as i32)) as u8));
vm_block = 167; continue;
}
// C line 36650
169 => {
vm_block = if ((*(fd)).has_parameter_expressions) != 0 { 168 } else { 164 }; continue;
}
// C line 36649
170 => {
let _ = emit_u16(s, ((idx) as u16));
vm_block = 169; continue;
}
// C line 36648
171 => {
let _ = emit_op(s, (((OP_rest as i32)) as u8));
vm_block = 170; continue;
}
// C line 36686
172 => {
let _ = emit_u16(s, (((*(fd)).scope_level) as u16));
vm_block = 146; continue;
}
// C line 36685
173 => {
let _ = emit_atom(s, name_1);
vm_block = 172; continue;
}
// C line 36684
174 => {
let _ = emit_op(s, (((OP_scope_put_var_init as i32)) as u8));
vm_block = 173; continue;
}
// C line 36683
175 => {
let _ = emit_label(s, label);
vm_block = 174; continue;
}
// C line 36682
176 => {
let _ = emit_u16(s, ((idx) as u16));
vm_block = 175; continue;
}
// C line 36681
177 => {
let _ = emit_op(s, (((OP_put_arg as i32)) as u8));
vm_block = 176; continue;
}
// C line 36680
178 => {
let _ = emit_op(s, (((OP_dup as i32)) as u8));
vm_block = 177; continue;
}
// C line 36679
179 => {
let _ = set_object_name(s, name_1);
vm_block = 178; continue;
}
// C line 36678
180 => {
vm_block = 5; continue;
}
// C line 36677
181 => {
vm_block = if (js_parse_assign_expr(s)) != 0 { 180 } else { 179 }; continue;
}
// C line 36676
182 => {
let _ = emit_op(s, (((OP_drop as i32)) as u8));
vm_block = 181; continue;
}
// C line 36675
183 => {
let _ = emit_goto(s, (OP_if_false as i32), label);
vm_block = 182; continue;
}
// C line 36674
184 => {
let _ = emit_op(s, (((OP_strict_eq as i32)) as u8));
vm_block = 183; continue;
}
// C line 36673
185 => {
let _ = emit_op(s, (((OP_undefined as i32)) as u8));
vm_block = 184; continue;
}
// C line 36672
186 => {
let _ = emit_op(s, (((OP_dup as i32)) as u8));
vm_block = 185; continue;
}
// C line 36671
187 => {
let _ = emit_u16(s, ((idx) as u16));
vm_block = 186; continue;
}
// C line 36670
188 => {
let _ = emit_op(s, (((OP_get_arg as i32)) as u8));
vm_block = 187; continue;
}
// C line 36669
189 => {
let _ = { let assigned = new_label(s); label = assigned; assigned };
vm_block = 188; continue;
}
// C line 36667
190 => {
vm_block = 5; continue;
}
// C line 36666
191 => {
vm_block = if (next_token(s)) != 0 { 190 } else { 189 }; continue;
}
// C line 36664
192 => {
let _ = { let assigned = (1 as i32); has_opt_arg = assigned; assigned };
vm_block = 191; continue;
}
// C line 36663
193 => {
let _ = { let assigned = (0 as i32); (*(fd)).has_simple_parameter_list = assigned; assigned };
vm_block = 192; continue;
}
// C line 36697
194 => {
let _ = emit_u16(s, (((*(fd)).scope_level) as u16));
vm_block = 146; continue;
}
// C line 36696
195 => {
let _ = emit_atom(s, name_1);
vm_block = 194; continue;
}
// C line 36695
196 => {
let _ = emit_op(s, (((OP_scope_put_var_init as i32)) as u8));
vm_block = 195; continue;
}
// C line 36694
197 => {
let _ = emit_u16(s, ((idx) as u16));
vm_block = 196; continue;
}
// C line 36693
198 => {
let _ = emit_op(s, (((OP_get_arg as i32)) as u8));
vm_block = 197; continue;
}
// C line 36691
199 => {
vm_block = if ((*(fd)).has_parameter_expressions) != 0 { 198 } else { 146 }; continue;
}
// C line 36689
200 => {
let _ = { let old = (*(fd)).defined_arg_count; (*(fd)).defined_arg_count = ((*(fd)).defined_arg_count).wrapping_add(1); old };
vm_block = 199; continue;
}
// C line 36688
201 => {
vm_block = if ((!((has_opt_arg) != 0) as i32)) != 0 { 200 } else { 199 }; continue;
}
// C line 36660
202 => {
vm_block = if ((((((*(s)).token).val) == ((61 as i32))) as i32)) != 0 { 193 } else { 201 }; continue;
}
// C line 36647
203 => {
vm_block = if (rest) != 0 { 171 } else { 202 }; continue;
}
// C line 36646
204 => {
vm_block = 5; continue;
}
// C line 36645
205 => {
vm_block = if (next_token(s)) != 0 { 204 } else { 203 }; continue;
}
// C line 36644
206 => {
vm_block = 5; continue;
}
// C line 36643
207 => {
vm_block = if ((((idx) < ((0 as i32))) as i32)) != 0 { 206 } else { 205 }; continue;
}
// C line 36642
208 => {
let _ = { let assigned = add_arg(ctx, fd, name_1); idx = assigned; assigned };
vm_block = 207; continue;
}
// C line 36639
209 => {
vm_block = 5; continue;
}
// C line 36638
210 => {
vm_block = if ((((define_var(s, fd, name_1, (((JS_VAR_DEF_LET as i32)) as JSVarDefEnum))) < ((0 as i32))) as i32)) != 0 { 209 } else { 208 }; continue;
}
// C line 36637
211 => {
vm_block = 5; continue;
}
// C line 36636
212 => {
vm_block = if (js_parse_check_duplicate_parameter(s, name_1)) != 0 { 211 } else { 210 }; continue;
}
// C line 36635
213 => {
vm_block = if ((*(fd)).has_parameter_expressions) != 0 { 212 } else { 208 }; continue;
}
// C line 36633
214 => {
vm_block = 5; continue;
}
// C line 36632
215 => {
let _ = js_parse_error_reserved_identifier(s);
vm_block = 214; continue;
}
// C line 36631
216 => {
vm_block = if ((((((((name_1) == ((((crate::quickjs_atom::JS_ATOM_yield as i32)) as JSAtom))) as i32)) != 0) && (((((((((*(fd)).func_kind as JSFunctionKindEnum)) as i32)) == ((JS_FUNC_GENERATOR as i32))) as i32)) != 0)) as i32)) != 0 { 215 } else { 213 }; continue;
}
// C line 36630
217 => {
let _ = { let assigned = ((((*(s)).token).u).ident).atom; name_1 = assigned; assigned };
vm_block = 216; continue;
}
// C line 36628
218 => {
vm_block = 5; continue;
}
// C line 36627
219 => {
let _ = js_parse_error_reserved_identifier(s);
vm_block = 218; continue;
}
// C line 36626
220 => {
vm_block = if (((((*(s)).token).u).ident).is_reserved) != 0 { 219 } else { 217 }; continue;
}
// C line 36702
221 => {
vm_block = 5; continue;
}
// C line 36701
222 => {
let _ = js_parse_error(s, c"missing formal parameter".as_ptr());
vm_block = 221; continue;
}
// C line 36625
223 => {
vm_block = if ((((((*(s)).token).val) == ((TOK_IDENT as i32))) as i32)) != 0 { 220 } else { 222 }; continue;
}
// C line 36607
224 => {
vm_block = if ((((((((((*(s)).token).val) == ((91 as i32))) as i32)) != 0) || (((((((*(s)).token).val) == ((123 as i32))) as i32)) != 0)) as i32)) != 0 { 160 } else { 223 }; continue;
}
// C line 36605
225 => {
vm_block = 5; continue;
}
// C line 36604
226 => {
vm_block = if (next_token(s)) != 0 { 225 } else { 224 }; continue;
}
// C line 36603
227 => {
let _ = { let assigned = (1 as i32); rest = assigned; assigned };
vm_block = 226; continue;
}
// C line 36602
228 => {
let _ = { let assigned = (0 as i32); (*(fd)).has_simple_parameter_list = assigned; assigned };
vm_block = 227; continue;
}
// C line 36601
229 => {
vm_block = 137; continue;
}
// C line 36600
230 => {
vm_block = if ((((((func_type) as u32)) == ((((JS_PARSE_FUNC_SETTER as i32)) as u32))) as i32)) != 0 { 229 } else { 228 }; continue;
}
// C line 36599
231 => {
vm_block = if ((((((*(s)).token).val) == ((TOK_ELLIPSIS as i32))) as i32)) != 0 { 230 } else { 224 }; continue;
}
// C line 36596
232 => {
rest = (0 as i32);
vm_block = 231; continue;
}
// C line 36591
233 => {
return ((1 as i32)).wrapping_neg();
}
// C line 36590
234 => {
vm_block = if ((((push_scope(s)) < ((0 as i32))) as i32)) != 0 { 233 } else { 139 }; continue;
}
// C line 36589
235 => {
let _ = { let assigned = ((1 as i32)).wrapping_neg(); (*(fd)).scope_level = assigned; assigned };
vm_block = 234; continue;
}
// C line 36588
236 => {
vm_block = if ((*(fd)).has_parameter_expressions) != 0 { 235 } else { 139 }; continue;
}
// C line 36582
237 => {
vm_block = 5; continue;
}
// C line 36581
238 => {
vm_block = if (next_token(s)) != 0 { 237 } else { 236 }; continue;
}
// C line 36580
239 => {
let _ = { let assigned = (1 as i32); (*(fd)).has_parameter_expressions = assigned; assigned };
vm_block = 238; continue;
}
// C line 36579
240 => {
vm_block = if (((skip_bits) & (((1 as i32)).wrapping_shl(((2 as i32)) as u32)))) != 0 { 239 } else { 238 }; continue;
}
// C line 36578
241 => {
let _ = js_parse_skip_parens_token(s, core::ptr::addr_of_mut!(skip_bits), (0 as i32));
vm_block = 240; continue;
}
// C line 36585
242 => {
vm_block = 5; continue;
}
// C line 36584
243 => {
vm_block = if (js_parse_expect(s, (40 as i32))) != 0 { 242 } else { 236 }; continue;
}
// C line 36574
244 => {
vm_block = if ((((((*(s)).token).val) == ((40 as i32))) as i32)) != 0 { 241 } else { 243 }; continue;
}
// C line 36573
245 => {
vm_block = if ((((((func_type) as u32)) != ((((JS_PARSE_FUNC_CLASS_STATIC_INIT as i32)) as u32))) as i32)) != 0 { 244 } else { 128 }; continue;
}
// C line 36563
246 => {
vm_block = if ((((((((((func_type) as u32)) == ((((JS_PARSE_FUNC_ARROW as i32)) as u32))) as i32)) != 0) && (((((((*(s)).token).val) == ((TOK_IDENT as i32))) as i32)) != 0)) as i32)) != 0 { 135 } else { 245 }; continue;
}
// C line 36562
247 => {
let _ = { let assigned = (0 as i32); has_opt_arg = assigned; assigned };
vm_block = 246; continue;
}
// C line 36561
248 => {
let _ = { let assigned = (0 as i32); (*(fd)).has_parameter_expressions = assigned; assigned };
vm_block = 247; continue;
}
// C line 36560
249 => {
let _ = { let assigned = (1 as i32); (*(fd)).has_simple_parameter_list = assigned; assigned };
vm_block = 248; continue;
}
// C line 36556
250 => {
let _ = emit_class_field_init(s);
vm_block = 249; continue;
}
// C line 36555
251 => {
vm_block = if ((((((func_type) as u32)) == ((((JS_PARSE_FUNC_CLASS_CONSTRUCTOR as i32)) as u32))) as i32)) != 0 { 250 } else { 249 }; continue;
}
// C line 36552
252 => {
let _ = emit_op(s, (((OP_check_ctor as i32)) as u8));
vm_block = 251; continue;
}
// C line 36549
253 => {
vm_block = if ((((((((((func_type) as u32)) == ((((JS_PARSE_FUNC_CLASS_CONSTRUCTOR as i32)) as u32))) as i32)) != 0) || (((((((func_type) as u32)) == ((((JS_PARSE_FUNC_DERIVED_CLASS_CONSTRUCTOR as i32)) as u32))) as i32)) != 0)) as i32)) != 0 { 252 } else { 251 }; continue;
}
// C line 36547
254 => {
let _ = { let assigned = func_type; (*(fd)).func_type = (assigned) as u8; assigned };
vm_block = 253; continue;
}
// C line 36546
255 => {
let _ = { let assigned = func_kind; (*(fd)).func_kind = (assigned) as u8; assigned };
vm_block = 254; continue;
}
// C line 36530
256 => {
let _ = { let assigned = (*((*(fd)).parent)).arguments_allowed; (*(fd)).arguments_allowed = assigned; assigned };
vm_block = 255; continue;
}
// C line 36529
257 => {
let _ = { let assigned = (*((*(fd)).parent)).super_allowed; (*(fd)).super_allowed = assigned; assigned };
vm_block = 256; continue;
}
// C line 36528
258 => {
let _ = { let assigned = (*((*(fd)).parent)).super_call_allowed; (*(fd)).super_call_allowed = assigned; assigned };
vm_block = 257; continue;
}
// C line 36527
259 => {
let _ = { let assigned = (*((*(fd)).parent)).new_target_allowed; (*(fd)).new_target_allowed = assigned; assigned };
vm_block = 258; continue;
}
// C line 36535
260 => {
let _ = { let assigned = (0 as i32); (*(fd)).arguments_allowed = assigned; assigned };
vm_block = 255; continue;
}
// C line 36534
261 => {
let _ = { let assigned = (1 as i32); (*(fd)).super_allowed = assigned; assigned };
vm_block = 260; continue;
}
// C line 36533
262 => {
let _ = { let assigned = (0 as i32); (*(fd)).super_call_allowed = assigned; assigned };
vm_block = 261; continue;
}
// C line 36532
263 => {
let _ = { let assigned = (1 as i32); (*(fd)).new_target_allowed = assigned; assigned };
vm_block = 262; continue;
}
// C line 36540
264 => {
let _ = { let assigned = (1 as i32); (*(fd)).arguments_allowed = assigned; assigned };
vm_block = 255; continue;
}
// C line 36539
265 => {
let _ = { let assigned = (*(fd)).has_home_object; (*(fd)).super_allowed = assigned; assigned };
vm_block = 264; continue;
}
// C line 36538
266 => {
let _ = { let assigned = (*(fd)).is_derived_class_constructor; (*(fd)).super_call_allowed = assigned; assigned };
vm_block = 265; continue;
}
// C line 36537
267 => {
let _ = { let assigned = (1 as i32); (*(fd)).new_target_allowed = assigned; assigned };
vm_block = 266; continue;
}
// C line 36531
268 => {
vm_block = if ((((((func_type) as u32)) == ((((JS_PARSE_FUNC_CLASS_STATIC_INIT as i32)) as u32))) as i32)) != 0 { 263 } else { 267 }; continue;
}
// C line 36526
269 => {
vm_block = if ((((((func_type) as u32)) == ((((JS_PARSE_FUNC_ARROW as i32)) as u32))) as i32)) != 0 { 259 } else { 268 }; continue;
}
// C line 36525
270 => {
let _ = { let assigned = (((((func_type) as u32)) == ((((JS_PARSE_FUNC_DERIVED_CLASS_CONSTRUCTOR as i32)) as u32))) as i32); (*(fd)).is_derived_class_constructor = assigned; assigned };
vm_block = 269; continue;
}
// C line 36524
271 => {
let _ = { let assigned = (*(fd)).has_arguments_binding; (*(fd)).has_this_binding = assigned; assigned };
vm_block = 270; continue;
}
// C line 36522
272 => {
let _ = { let assigned = (((((((((func_type) as u32)) != ((((JS_PARSE_FUNC_ARROW as i32)) as u32))) as i32)) != 0) && (((((((func_type) as u32)) != ((((JS_PARSE_FUNC_CLASS_STATIC_INIT as i32)) as u32))) as i32)) != 0)) as i32); (*(fd)).has_arguments_binding = assigned; assigned };
vm_block = 271; continue;
}
// C line 36517
273 => {
let _ = { let assigned = (((((((((((((((((((((func_type) as u32)) == ((((JS_PARSE_FUNC_METHOD as i32)) as u32))) as i32)) != 0) || (((((((func_type) as u32)) == ((((JS_PARSE_FUNC_GETTER as i32)) as u32))) as i32)) != 0)) as i32)) != 0) || (((((((func_type) as u32)) == ((((JS_PARSE_FUNC_SETTER as i32)) as u32))) as i32)) != 0)) as i32)) != 0) || (((((((func_type) as u32)) == ((((JS_PARSE_FUNC_CLASS_CONSTRUCTOR as i32)) as u32))) as i32)) != 0)) as i32)) != 0) || (((((((func_type) as u32)) == ((((JS_PARSE_FUNC_DERIVED_CLASS_CONSTRUCTOR as i32)) as u32))) as i32)) != 0)) as i32); (*(fd)).has_home_object = assigned; assigned };
vm_block = 272; continue;
}
// C line 36513
274 => {
let _ = { let assigned = (((((((((((((((((func_type) as u32)) == ((((JS_PARSE_FUNC_STATEMENT as i32)) as u32))) as i32)) != 0) || (((((((func_type) as u32)) == ((((JS_PARSE_FUNC_VAR as i32)) as u32))) as i32)) != 0)) as i32)) != 0) || (((((((func_type) as u32)) == ((((JS_PARSE_FUNC_EXPR as i32)) as u32))) as i32)) != 0)) as i32)) != 0) && (((((((func_kind) as u32)) == ((((JS_FUNC_NORMAL as i32)) as u32))) as i32)) != 0)) as i32); (*(fd)).has_prototype = assigned; assigned };
vm_block = 273; continue;
}
// C line 36511
275 => {
let _ = { let assigned = func_name; (*(fd)).func_name = assigned; assigned };
vm_block = 274; continue;
}
// C line 36510
276 => {
let _ = { let assigned = fd; (*(s)).cur_func = assigned; assigned };
vm_block = 275; continue;
}
// C line 36509
277 => {
let _ = { let assigned = fd; *(pfd) = assigned; assigned };
vm_block = 276; continue;
}
// C line 36508
278 => {
vm_block = if !(pfd).is_null() { 277 } else { 276 }; continue;
}
// C line 36506
279 => {
return ((1 as i32)).wrapping_neg();
}
// C line 36505
280 => {
let _ = JS_FreeAtom(ctx, func_name);
vm_block = 279; continue;
}
// C line 36504
281 => {
vm_block = if ((!(!(fd).is_null()) as i32)) != 0 { 280 } else { 278 }; continue;
}
// C line 36501
282 => {
let _ = { let assigned = js_new_function_def(ctx, fd, (0 as i32), is_expr, (*(s)).filename, ptr, core::ptr::addr_of_mut!((*(s)).get_line_col_cache)); fd = assigned; assigned };
vm_block = 281; continue;
}
// C line 36484
283 => {
return ((1 as i32)).wrapping_neg();
}
// C line 36483
284 => {
let _ = JS_FreeAtom(ctx, func_name);
vm_block = 283; continue;
}
// C line 36482
285 => {
let _ = js_parse_error(s, c"invalid redefinition of global identifier".as_ptr());
vm_block = 284; continue;
}
// C line 36481
286 => {
vm_block = if ((((!(hf_1).is_null()) && ((((((*(hf_1)).scope_level) == ((*(fd)).scope_level)) as i32)) != 0)) as i32)) != 0 { 285 } else { 282 }; continue;
}
// C line 36479
287 => {
let _ = { let assigned = find_global_var(fd, func_name); hf_1 = assigned; assigned };
vm_block = 286; continue;
}
// C line 36496
288 => {
return ((1 as i32)).wrapping_neg();
}
// C line 36495
289 => {
let _ = JS_FreeAtom(ctx, func_name);
vm_block = 288; continue;
}
// C line 36494
290 => {
vm_block = if ((((lexical_func_idx) < ((0 as i32))) as i32)) != 0 { 289 } else { 282 }; continue;
}
// C line 36490
291 => {
let _ = { let assigned = define_var(s, fd, func_name, ((if ((((((func_kind) as u32)) != ((((JS_FUNC_NORMAL as i32)) as u32))) as i32)) != 0 { (JS_VAR_DEF_NEW_FUNCTION_DECL as i32) } else { (JS_VAR_DEF_FUNCTION_DECL as i32) }) as JSVarDefEnum)); lexical_func_idx = assigned; assigned };
vm_block = 290; continue;
}
// C line 36472
292 => {
vm_block = if ((((((((((*(fd)).is_eval) != 0) && ((((((((((*(fd)).eval_type) == (((0 as i32)).wrapping_shl(((0 as i32)) as u32))) as i32)) != 0) || ((((((*(fd)).eval_type) == (((1 as i32)).wrapping_shl(((0 as i32)) as u32))) as i32)) != 0)) as i32)) != 0)) as i32)) != 0) && ((((((*(fd)).scope_level) == ((*(fd)).body_scope)) as i32)) != 0)) as i32)) != 0 { 287 } else { 291 }; continue;
}
// C line 36468
293 => {
let _ = { let assigned = (1 as i32); create_func_var = assigned; assigned };
vm_block = 292; continue;
}
// C line 36463
294 => {
vm_block = if ((((((((((((((((((!(((((((*(fd)).js_mode) as i32)) & (((1 as i32)).wrapping_shl(((0 as i32)) as u32)))) != 0) as i32)) != 0) && (((((((func_kind) as u32)) == ((((JS_FUNC_NORMAL as i32)) as u32))) as i32)) != 0)) as i32)) != 0) && (((((find_lexical_decl(ctx, fd, func_name, (*(fd)).scope_first, (0 as i32))) < ((0 as i32))) as i32)) != 0)) as i32)) != 0) && (((!((((((((({ let assigned = find_var(ctx, fd, func_name); func_idx = assigned; assigned }) >= ((0 as i32))) as i32)) != 0) && ((((func_idx) & ((536870912 as i32)))) != 0)) as i32)) != 0) as i32)) != 0)) as i32)) != 0) && (((!(((((((((func_name) == ((((crate::quickjs_atom::JS_ATOM_arguments as i32)) as JSAtom))) as i32)) != 0) && (((*(fd)).has_arguments_binding) != 0)) as i32)) != 0) as i32)) != 0)) as i32)) != 0 { 293 } else { 292 }; continue;
}
// C line 36462
295 => {
vm_block = if ((((((func_type) as u32)) == ((((JS_PARSE_FUNC_VAR as i32)) as u32))) as i32)) != 0 { 294 } else { 282 }; continue;
}
// C line 36458
296 => {
return ((1 as i32)).wrapping_neg();
}
// C line 36457
297 => {
let _ = JS_FreeAtom(ctx, func_name);
vm_block = 296; continue;
}
// C line 36456
298 => {
let _ = js_parse_error(s, c"invalid redefinition of global identifier in module code".as_ptr());
vm_block = 297; continue;
}
// C line 36455
299 => {
vm_block = if ((((!(hf).is_null()) && ((((((*(hf)).scope_level) == ((*(fd)).scope_level)) as i32)) != 0)) as i32)) != 0 { 298 } else { 295 }; continue;
}
// C line 36453
300 => {
let _ = { let assigned = find_global_var(fd, func_name); hf = assigned; assigned };
vm_block = 299; continue;
}
// C line 36450
301 => {
vm_block = if ((((((((((*(fd)).is_eval) != 0) && ((((((*(fd)).eval_type) == (((1 as i32)).wrapping_shl(((0 as i32)) as u32))) as i32)) != 0)) as i32)) != 0) && (((((((((((func_type) as u32)) == ((((JS_PARSE_FUNC_STATEMENT as i32)) as u32))) as i32)) != 0) || (((((((func_type) as u32)) == ((((JS_PARSE_FUNC_VAR as i32)) as u32))) as i32)) != 0)) as i32)) != 0)) as i32)) != 0 { 300 } else { 295 }; continue;
}
// C line 36438
302 => {
return ((1 as i32)).wrapping_neg();
}
// C line 36437
303 => {
let _ = JS_FreeAtom(ctx, func_name);
vm_block = 302; continue;
}
// C line 36436
304 => {
vm_block = if (next_token(s)) != 0 { 303 } else { 301 }; continue;
}
// C line 36435
305 => {
let _ = { let assigned = JS_DupAtom(ctx, ((((*(s)).token).u).ident).atom); func_name = assigned; assigned };
vm_block = 304; continue;
}
// C line 36443
306 => {
return js_parse_error(s, c"function name expected".as_ptr());
}
// C line 36441
307 => {
vm_block = if ((((((((((func_type) as u32)) != ((((JS_PARSE_FUNC_EXPR as i32)) as u32))) as i32)) != 0) && (((((((export_flag) as u32)) != ((((JS_PARSE_EXPORT_DEFAULT as i32)) as u32))) as i32)) != 0)) as i32)) != 0 { 306 } else { 301 }; continue;
}
// C line 36431
308 => {
vm_block = if ((((((((((*(s)).token).val) == ((TOK_IDENT as i32))) as i32)) != 0) || (((((((((((((((((((*(s)).token).val) == ((TOK_YIELD as i32))) as i32)) != 0) && (((!(((((((*(fd)).js_mode) as i32)) & (((1 as i32)).wrapping_shl(((0 as i32)) as u32)))) != 0) as i32)) != 0)) as i32)) != 0) || (((((((((((*(s)).token).val) == ((TOK_AWAIT as i32))) as i32)) != 0) && (((!(((*(s)).is_module) != 0) as i32)) != 0)) as i32)) != 0)) as i32)) != 0) && (((((((func_type) as u32)) == ((((JS_PARSE_FUNC_EXPR as i32)) as u32))) as i32)) != 0)) as i32)) != 0)) as i32)) != 0 { 305 } else { 307 }; continue;
}
// C line 36428
309 => {
return js_parse_error_reserved_identifier(s);
}
// C line 36420
310 => {
vm_block = if (((((((((((((*(s)).token).u).ident).is_reserved) != 0) || (((((((((((((((((*(s)).token).u).ident).atom) == ((((crate::quickjs_atom::JS_ATOM_yield as i32)) as JSAtom))) as i32)) != 0) && (((((((func_type) as u32)) == ((((JS_PARSE_FUNC_EXPR as i32)) as u32))) as i32)) != 0)) as i32)) != 0) && ((((((func_kind) as u32)) & ((((JS_FUNC_GENERATOR as i32)) as u32)))) != 0)) as i32)) != 0)) as i32)) != 0) || (((((((((((((*(s)).token).u).ident).atom) == ((((crate::quickjs_atom::JS_ATOM_await as i32)) as JSAtom))) as i32)) != 0) && (((((((((((((((func_type) as u32)) == ((((JS_PARSE_FUNC_EXPR as i32)) as u32))) as i32)) != 0) && ((((((func_kind) as u32)) & ((((JS_FUNC_ASYNC as i32)) as u32)))) != 0)) as i32)) != 0) || (((((((func_type) as u32)) == ((((JS_PARSE_FUNC_CLASS_STATIC_INIT as i32)) as u32))) as i32)) != 0)) as i32)) != 0)) as i32)) != 0)) as i32)) != 0 { 309 } else { 308 }; continue;
}
// C line 36419
311 => {
vm_block = if ((((((*(s)).token).val) == ((TOK_IDENT as i32))) as i32)) != 0 { 310 } else { 308 }; continue;
}
// C line 36416
312 => {
let _ = { func_kind = (((((func_kind) as u32)) | ((((JS_FUNC_GENERATOR as i32)) as u32)))) as JSFunctionKindEnum; func_kind };
vm_block = 311; continue;
}
// C line 36415
313 => {
return ((1 as i32)).wrapping_neg();
}
// C line 36414
314 => {
vm_block = if (next_token(s)) != 0 { 313 } else { 312 }; continue;
}
// C line 36413
315 => {
vm_block = if ((((((*(s)).token).val) == ((42 as i32))) as i32)) != 0 { 314 } else { 311 }; continue;
}
// C line 36412
316 => {
return ((1 as i32)).wrapping_neg();
}
// C line 36411
317 => {
vm_block = if (next_token(s)) != 0 { 316 } else { 315 }; continue;
}
// C line 36409
318 => {
let _ = { let assigned = (((JS_FUNC_ASYNC as i32)) as JSFunctionKindEnum); func_kind = assigned; assigned };
vm_block = 317; continue;
}
// C line 36408
319 => {
return ((1 as i32)).wrapping_neg();
}
// C line 36407
320 => {
vm_block = if (next_token(s)) != 0 { 319 } else { 318 }; continue;
}
// C line 36404
321 => {
vm_block = if ((((((((((((((func_kind) as u32)) == ((((JS_FUNC_NORMAL as i32)) as u32))) as i32)) != 0) && ((token_is_pseudo_keyword(s, (((crate::quickjs_atom::JS_ATOM_async as i32)) as JSAtom))) != 0)) as i32)) != 0) && (((((peek_token(s, (1 as i32))) != ((10 as i32))) as i32)) != 0)) as i32)) != 0 { 320 } else { 317 }; continue;
}
// C line 36447
322 => {
let _ = { let assigned = JS_DupAtom(ctx, func_name); func_name = assigned; assigned };
vm_block = 301; continue;
}
// C line 36446
323 => {
vm_block = if ((((((func_type) as u32)) != ((((JS_PARSE_FUNC_ARROW as i32)) as u32))) as i32)) != 0 { 322 } else { 301 }; continue;
}
// C line 36401
324 => {
vm_block = if ((((((((((((((func_type) as u32)) == ((((JS_PARSE_FUNC_STATEMENT as i32)) as u32))) as i32)) != 0) || (((((((func_type) as u32)) == ((((JS_PARSE_FUNC_VAR as i32)) as u32))) as i32)) != 0)) as i32)) != 0) || (((((((func_type) as u32)) == ((((JS_PARSE_FUNC_EXPR as i32)) as u32))) as i32)) != 0)) as i32)) != 0 { 321 } else { 323 }; continue;
}
// C line 36398
325 => {
let _ = { let assigned = (((((((((func_type) as u32)) != ((((JS_PARSE_FUNC_STATEMENT as i32)) as u32))) as i32)) != 0) && (((((((func_type) as u32)) != ((((JS_PARSE_FUNC_VAR as i32)) as u32))) as i32)) != 0)) as i32); is_expr = assigned; assigned };
vm_block = 324; continue;
}
// C line 36396
326 => {
create_func_var = (0 as i32);
vm_block = 325; continue;
}
// C line 36394
327 => {
lexical_func_idx = ((1 as i32)).wrapping_neg();
vm_block = 326; continue;
}
// C line 36392
328 => {
fd = (*(s)).cur_func;
vm_block = 327; continue;
}
// C line 36391
329 => {
ctx = (*(s)).ctx;
vm_block = 328; continue;
}
_ => std::process::abort(),
} }
}
