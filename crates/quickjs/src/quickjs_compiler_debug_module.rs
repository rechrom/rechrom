// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:31131. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn js_dump_module(mut ctx: *mut JSContext, mut str: *const c_char, mut m: *mut JSModuleDef) -> () {
let mut vm_local_storage = Vec::<u64>::new();
let mut buf1: [c_char; 64] = core::mem::zeroed();
let mut module_status_str: [*const c_char; 6] = core::mem::zeroed();
let mut vm_block: usize = 2;
loop { match vm_block {
// C line ?
0 => {
return ();
}
// C line 31135
1 => {
let _ = quickjs_debug_write(&quickjs_format_print(c"%s: %s status=%s\n".as_ptr(), &[QuickJSPrintArg::Str(str as *const c_char), QuickJSPrintArg::Str(JS_AtomGetStr(ctx, (buf1).as_mut_ptr(), (((size_of::<[c_char; 64]>() as usize)) as i32), (*(m)).module_name) as *const c_char), QuickJSPrintArg::Str(*((module_status_str).as_ptr()).offset((((*(m)).status as JSModuleStatus)) as isize) as *const c_char)]));
vm_block = 0; continue;
}
// C line 31134
2 => {
module_status_str = [c"unlinked".as_ptr(), c"linking".as_ptr(), c"linked".as_ptr(), c"evaluating".as_ptr(), c"evaluating_async".as_ptr(), c"evaluated".as_ptr()];
vm_block = 1; continue;
}
_ => std::process::abort(),
} }
}
