unsafe fn value_print_capture(opaque:*mut c_void, buf:*const c_char,len:usize) {
    let out=&mut*opaque.cast::<Vec<u8>>();out.extend_from_slice(core::slice::from_raw_parts(buf.cast::<u8>(),len));
}
pub unsafe fn value_print_fixture()->Vec<u8> {
    let rt=JS_NewRuntime();assert!(!rt.is_null());let ctx=serialization_context(rt);
    assert_eq!(JS_AddIntrinsicRegExp(ctx),0);assert_eq!(JS_AddIntrinsicMapSet(ctx),0);assert_eq!(JS_AddIntrinsicDate(ctx),0);
    let mut out=Vec::new();let mut capture=Vec::<u8>::new();
    for source in include_str!("quickjs_value_print_cases.txt").lines() {
        let input=std::ffi::CString::new(source).unwrap();let mut v=JS_Eval(ctx,input.as_ptr(),source.len(),c"print.js".as_ptr(),0);
        if source=="Symbol(\"é\")" {JS_FreeValue(ctx,v);let mut arg=JS_NewString(ctx,c"é".as_ptr());v=js_symbol_constructor(ctx,JS_UNDEFINED,1,&mut arg);JS_FreeValue(ctx,arg);}
        if source=="Symbol.iterator" {JS_FreeValue(ctx,v);v=JS_AtomToValue(ctx,crate::quickjs_atom::JS_ATOM_Symbol_iterator);}
        if JS_IsException(v)!=0 {v=JS_GetException(ctx);}
        for max_depth in [0,1,2,8] { for max_string in [0,1,8,1000] { for max_items in [0,1,3,100] { for hidden in 0..2 { for raw in 0..2 {
            if raw!=0 && JS_IsObject(v)!=0 {continue;}
            let mut options:JSPrintValueOptions=core::mem::zeroed();options.max_depth=max_depth;options.max_string_length=max_string;options.max_item_count=max_items;options.set_show_hidden(hidden);options.set_raw_dump(raw);
            capture.clear();JS_PrintValue(ctx,Some(value_print_capture),core::ptr::addr_of_mut!(capture).cast(),v,&options);
            serialization_u32(&mut out,capture.len() as u32);out.extend_from_slice(&capture);
            capture.clear();JS_PrintValueRT(rt,Some(value_print_capture),core::ptr::addr_of_mut!(capture).cast(),v,&options);
            serialization_u32(&mut out,capture.len() as u32);out.extend_from_slice(&capture);
        }}}}}
        JS_FreeValue(ctx,v);
    }
    JS_FreeContext(ctx);JS_FreeRuntime(rt);out
}
