pub unsafe fn compiler_debug_fixture() {
    let rt=JS_NewRuntime();assert!(!rt.is_null());let ctx=serialization_context(rt);
    for source in include_str!("quickjs_serialization_compile_cases.txt").lines() {
        let input=std::ffi::CString::new(source).unwrap();let v=JS_Eval(ctx,input.as_ptr(),source.len(),c"debug.js".as_ptr(),JS_EVAL_FLAG_COMPILE_ONLY);
        assert_eq!(JS_IsException(v),0);js_dump_function_bytecode(ctx,JS_VALUE_GET_PTR(v).cast());JS_FreeValue(ctx,v);
    }
    for source in include_str!("quickjs_serialization_module_cases.txt").lines() {
        let input=std::ffi::CString::new(source).unwrap();let v=JS_Eval(ctx,input.as_ptr(),source.len(),c"debug.mjs".as_ptr(),JS_EVAL_FLAG_COMPILE_ONLY|JS_EVAL_TYPE_MODULE);
        assert_eq!(JS_IsException(v),0);js_dump_module(ctx,c"compiled".as_ptr(),JS_VALUE_GET_PTR(v).cast());JS_FreeValue(ctx,v);
    }
    let inputs:&[&[u8]]=&[&[],&[0],&[1,1,42,0],&[128],&[0,0,255,255,255,255,255]];
    for data in inputs {dump_pc2line(ctx,data.as_ptr(),data.len() as i32);}
    for source in [c"",c"one",c"one\ntwo\n",c"a\nb\nc"] {print_lines(source.as_ptr(),0,3);}
    JS_FreeContext(ctx);JS_FreeRuntime(rt);
}
