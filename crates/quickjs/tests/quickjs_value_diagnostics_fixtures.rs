include!("quickjs_diagnostic_allocator.rs");
pub unsafe fn value_diagnostics_fixture() {
    let rt=if cfg!(diagnostics_allocator) {JS_NewRuntime2(&DIAGNOSTIC_MALLOC,core::ptr::null_mut())} else {JS_NewRuntime()};assert!(!rt.is_null());let ctx=serialization_context(rt);
    for c in -3..260 {JS_DumpChar(core::ptr::null_mut(),c,34);}
    for text in [c"",c"a\"b\\c\n",c"é"] {
        let v=JS_NewString(ctx,text.as_ptr());JS_DumpString(rt,JS_VALUE_GET_PTR(v).cast::<JSString>());let v2=JS_DupValue(ctx,v);JS_DumpString(rt,JS_VALUE_GET_PTR(v).cast::<JSString>());JS_FreeValue(ctx,v2);JS_FreeValue(ctx,v);
    }
    let mut x=JS_Eval(ctx,c"123456789012345678901234567890n".as_ptr(),31,c"dump.js".as_ptr(),0);
    if JS_IsException(x)!=0 {JS_FreeValue(ctx,JS_GetException(ctx));x=JS_NewBigInt64(ctx,9223372036854775807);}
    if JS_VALUE_GET_TAG(x)==JS_TAG_BIG_INT {js_bigint_dump(ctx,c"big".as_ptr(),JS_VALUE_GET_PTR(x).cast());}
    JS_FreeValue(ctx,x);
    JS_DumpObjectHeader(rt);JS_DumpObject(rt,JS_VALUE_GET_OBJ((*ctx).global_obj));JS_DumpGCObject(rt,core::ptr::addr_of_mut!((*ctx).header));
    JS_DumpShapes(rt);JS_DumpAtoms(rt);js_malloc_dump_arenas(core::ptr::addr_of_mut!((*rt).malloc_ctx));
    for i in 0..3 {
        let mut m:JSMemoryUsage=core::mem::zeroed();
        if i>0 {let fields=core::slice::from_raw_parts_mut(core::ptr::addr_of_mut!(m).cast::<i64>(),size_of::<JSMemoryUsage>()/8);for (j,n) in fields.iter_mut().enumerate(){*n=if i==1 {(j as i64+1)*3} else {j as i64+1};}}
        JS_DumpMemoryUsage(core::ptr::null_mut(),&m,core::ptr::null_mut());
    }
    #[cfg(diagnostics_memory_rt)] {let mut m:JSMemoryUsage=core::mem::zeroed();JS_ComputeMemoryUsage(rt,&mut m);JS_DumpMemoryUsage(core::ptr::null_mut(),&m,rt);}
    JS_FreeContext(ctx);JS_FreeRuntime(rt);
}
