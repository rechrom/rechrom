// Generated from the official C AST; original goto CFG retained.
// c: quickjs.c:22100. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
unsafe fn dump_token(mut s: *mut JSParseState, mut token: *const JSToken) -> () {
let mut vm_local_storage = Vec::<u64>::new();
let mut d: f64 = core::mem::zeroed();
let mut buf: [c_char; 64] = core::mem::zeroed();
let mut str: *const c_char = core::mem::zeroed();
let mut str_1: *const c_char = core::mem::zeroed();
let mut str_2: *const c_char = core::mem::zeroed();
let mut str2: *const c_char = core::mem::zeroed();
let mut vm_block: usize = 28;
loop { match vm_block {
// C line ?
0 => {
return ();
}
// C line 22157
1 => {
vm_block = 0; continue;
}
// C line 22151
2 => {
vm_block = 24; continue;
}
// C line 22153
3 => {
let _ = quickjs_debug_write(&quickjs_format_print(c"token: %d\n".as_ptr(), &[QuickJSPrintArg::Int((*(token)).val as u64)]));
vm_block = 1; continue;
}
// C line 22155
4 => {
let _ = quickjs_debug_write(&quickjs_format_print(c"token: '%c'\n".as_ptr(), &[QuickJSPrintArg::Int((*(token)).val as u64)]));
vm_block = 1; continue;
}
// C line 22152
5 => {
vm_block = if ((((((*(s)).token).val) >= ((256 as i32))) as i32)) != 0 { 3 } else { 4 }; continue;
}
// C line 22150
6 => {
vm_block = if ((((((((((*(s)).token).val) >= ((TOK_NULL as i32))) as i32)) != 0) && (((((((*(s)).token).val) <= ((TOK_AWAIT as i32))) as i32)) != 0)) as i32)) != 0 { 2 } else { 5 }; continue;
}
// C line 22148
7 => {
vm_block = 0; continue;
}
// C line 22147
8 => {
let _ = quickjs_debug_write(&quickjs_format_print(c"eof\n".as_ptr(), &[]));
vm_block = 7; continue;
}
// C line 22145
9 => {
vm_block = 0; continue;
}
// C line 22143
10 => {
let _ = JS_FreeCString((*(s)).ctx, str2);
vm_block = 9; continue;
}
// C line 22142
11 => {
let _ = JS_FreeCString((*(s)).ctx, str_2);
vm_block = 10; continue;
}
// C line 22141
12 => {
let _ = quickjs_debug_write(&quickjs_format_print(c"regexp: '%s' '%s'\n".as_ptr(), &[QuickJSPrintArg::Str(str_2 as *const c_char), QuickJSPrintArg::Str(str2 as *const c_char)]));
vm_block = 11; continue;
}
// C line 22140
13 => {
let _ = { let assigned = JS_ToCString((*(s)).ctx, (((*(token)).u).regexp).flags); str2 = assigned; assigned };
vm_block = 12; continue;
}
// C line 22139
14 => {
let _ = { let assigned = JS_ToCString((*(s)).ctx, (((*(token)).u).regexp).body); str_2 = assigned; assigned };
vm_block = 13; continue;
}
// C line 22135
15 => {
vm_block = 0; continue;
}
// C line 22133
16 => {
let _ = JS_FreeCString((*(s)).ctx, str_1);
vm_block = 15; continue;
}
// C line 22132
17 => {
let _ = quickjs_debug_write(&quickjs_format_print(c"template: `%s`\n".as_ptr(), &[QuickJSPrintArg::Str(str_1 as *const c_char)]));
vm_block = 16; continue;
}
// C line 22131
18 => {
let _ = { let assigned = JS_ToCString((*(s)).ctx, (((*(token)).u).str).str); str_1 = assigned; assigned };
vm_block = 17; continue;
}
// C line 22127
19 => {
vm_block = 0; continue;
}
// C line 22125
20 => {
let _ = JS_FreeCString((*(s)).ctx, str);
vm_block = 19; continue;
}
// C line 22124
21 => {
let _ = quickjs_debug_write(&quickjs_format_print(c"string: '%s'\n".as_ptr(), &[QuickJSPrintArg::Str(str as *const c_char)]));
vm_block = 20; continue;
}
// C line 22123
22 => {
let _ = { let assigned = JS_ToCString((*(s)).ctx, (((*(token)).u).str).str); str = assigned; assigned };
vm_block = 21; continue;
}
// C line 22118
23 => {
vm_block = 0; continue;
}
// C line 22115 labels: dump_atom
24 => {
let _ = quickjs_debug_write(&quickjs_format_print(c"ident: '%s'\n".as_ptr(), &[QuickJSPrintArg::Str(JS_AtomGetStr((*(s)).ctx, (buf).as_mut_ptr(), (((size_of::<[c_char; 64]>() as usize)) as i32), (((*(token)).u).ident).atom) as *const c_char)]));
vm_block = 23; continue;
}
// C line 22110
25 => {
vm_block = 0; continue;
}
// C line 22108
26 => {
let _ = quickjs_debug_write(&quickjs_format_print(c"number: %.14g\n".as_ptr(), &[QuickJSPrintArg::Float(d as f64)]));
vm_block = 25; continue;
}
// C line 22107
27 => {
let _ = JS_ToFloat64((*(s)).ctx, core::ptr::addr_of_mut!(d), (((*(token)).u).num).val);
vm_block = 26; continue;
}
// C line 22103
28 => {
vm_block = match (*(token)).val { x if x == (TOK_EOF as i32) => 8, x if x == (TOK_REGEXP as i32) => 14, x if x == (TOK_TEMPLATE as i32) => 18, x if x == (TOK_STRING as i32) => 22, x if x == (TOK_IDENT as i32) => 24, x if x == (TOK_NUMBER as i32) => 27, _ => 6, }; continue;
}
_ => std::process::abort(),
} }
}
