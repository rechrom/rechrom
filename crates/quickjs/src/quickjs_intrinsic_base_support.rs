// quickjs.c:54507. URIError's va_list formatting uses the shared Rust error
// formatter instead; all original call sites have literal messages.
unsafe fn js_throw_URIError(ctx:*mut JSContext,message:*const c_char)->i32 {
    // The original literal "expecting %%" is printf escaped.
    let bytes=std::ffi::CStr::from_ptr(message).to_bytes();
    let _=if bytes==b"expecting %%"{JS_ThrowError(ctx,JS_URI_ERROR,c"expecting %".as_ptr().into())}else{JS_ThrowError(ctx,JS_URI_ERROR,message.into())};
    -1
}
unsafe fn js_uri_memchr(p:*const c_void,c:i32,n:usize)->*mut c_void{
    let p=p.cast::<u8>();for i in 0..n{if *p.add(i)==c as u8{return p.add(i).cast_mut().cast();}}ptr::null_mut()
}
