// quickjs.c:37410..37431 and 38266..38295. Bellard/Gordon MIT.
#[cfg(feature="dump-read-object")]
static bc_tag_str: [&core::ffi::CStr;20]=[
 c"invalid",c"null",c"undefined",c"false",c"true",c"int32",c"float64",c"string",c"object",c"array",c"bigint",c"template",c"function",c"module",c"TypedArray",c"ArrayBuffer",c"SharedArrayBuffer",c"Date",c"ObjectValue",c"ObjectReference",
];
// Typed varargs/stdio boundary; fmt itself controls original brace nesting.
#[cfg(feature="dump-read-object")]
unsafe fn bc_read_trace(s:*mut BCReaderState,fmt:*const c_char,args:&[QuickJSPrintArg]) {
 if (*s).ptr_last.is_null(){(*s).ptr_last=(*s).buf_start;}
 let mut n=0i32;let mut n0=0i32;
 if (*s).ptr > (*s).ptr_last || (*s).ptr==(*s).buf_start {
  n0=quickjs_debug_write(&quickjs_format_print(c"%04x: ".as_ptr(),&[QuickJSPrintArg::Int((*s).ptr_last.offset_from((*s).buf_start) as i32 as u64)]));n=n.wrapping_add(n0);
 }
 let mut i=0i32;
 while (*s).ptr_last<(*s).ptr {
  if i&7==0 && i>0 {let _=quickjs_debug_write(&quickjs_format_print(c"\n%*s".as_ptr(),&[QuickJSPrintArg::Int(n0 as u64),QuickJSPrintArg::Str(c"".as_ptr())]));n=n0;}
  let byte=*(*s).ptr_last;(*s).ptr_last=(*s).ptr_last.add(1);
  n=n.wrapping_add(quickjs_debug_write(&quickjs_format_print(c" %02x".as_ptr(),&[QuickJSPrintArg::Int(byte as u64)])));i=i.wrapping_add(1);
 }
 if *fmt as u8==b'}'{(*s).level=(*s).level.wrapping_sub(1);}
 let column=32i32.wrapping_add((*s).level.wrapping_mul(2));
 if n<column {let _=quickjs_debug_write(&quickjs_format_print(c"%*s".as_ptr(),&[QuickJSPrintArg::Int(column.wrapping_sub(n)as u64),QuickJSPrintArg::Str(c"".as_ptr())]));}
 let _=quickjs_debug_write(&quickjs_format_print(fmt,args));
 if std::ffi::CStr::from_ptr(fmt).to_bytes().contains(&b'{'){(*s).level=(*s).level.wrapping_add(1);}
}

// Original C indexes the diagnostic table before validating the tag. An
// out-of-range tag has undefined C behavior. Preserve every valid diagnostic,
// then let the unchanged reader switch reject malformed tags with SyntaxError.
#[cfg(feature = "dump-read-object")]
fn bc_read_tag_name(tag: usize) -> &'static core::ffi::CStr {
    bc_tag_str.get(tag).copied().unwrap_or(c"invalid")
}
