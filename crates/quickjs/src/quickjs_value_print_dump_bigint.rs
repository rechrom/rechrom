// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:11638. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_bigint_dump1(mut ctx: *mut JSContext, mut str: *const c_char, mut tab: *const js_limb_t, mut len: i32) -> () {
let mut vm_local_storage = Vec::<u64>::new();
let mut i: i32 = core::mem::zeroed();
let mut vm_block: usize = 6;
loop { match vm_block {
// C line ?
0 => {
return ();
}
// C line 11650
1 => {
let _ = quickjs_debug_write(&quickjs_format_print(c"\n".as_ptr(), &[]));
vm_block = 0; continue;
}
// C line 11643
2 => {
vm_block = if ((((i) >= ((0 as i32))) as i32)) != 0 { 4 } else { 1 }; continue;
}
// C line 11643
3 => {
let _ = { let old = i; i = (i).wrapping_sub(1); old };
vm_block = 2; continue;
}
// C line 11647
4 => {
let _ = quickjs_debug_write(&quickjs_format_print(c" %016llx".as_ptr(), &[QuickJSPrintArg::Int(*(tab).offset((i) as isize) as u64)]));
vm_block = 3; continue;
}
// C line 11643
5 => {
let _ = { let assigned = (len).wrapping_sub((1 as i32)); i = assigned; assigned };
vm_block = 2; continue;
}
// C line 11642
6 => {
let _ = quickjs_debug_write(&quickjs_format_print(c"%s: ".as_ptr(), &[QuickJSPrintArg::Str(str as *const c_char)]));
vm_block = 5; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:11653. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_bigint_dump(mut ctx: *mut JSContext, mut str: *const c_char, mut p: *const JSBigInt) -> () {
let mut vm_local_storage = Vec::<u64>::new();
let mut vm_block: usize = 1;
loop { match vm_block {
// C line ?
0 => {
return ();
}
// C line 11656
1 => {
let _ = js_bigint_dump1(ctx, str, ((*(p)).tab).as_ptr(), (((*(p)).len) as i32));
vm_block = 0; continue;
}
_ => std::process::abort(),
} }
}
