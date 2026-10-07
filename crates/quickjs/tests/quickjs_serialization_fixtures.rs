include!("quickjs_serialization_sab_host.rs");
fn serialization_u32(out: &mut Vec<u8>, n: u32) { out.extend_from_slice(&n.to_le_bytes()); }
unsafe fn serialization_value(out: &mut Vec<u8>, ctx: *mut JSContext, val: JSValue) {
    serialization_u32(out, JS_VALUE_GET_NORM_TAG(val) as u32);
    let is_exception=JS_IsException(val)!=0;
    let original=if is_exception {JS_GetException(ctx)} else {JS_DupValue(ctx,val)};
    let val=if is_exception {JS_GetPropertyStr(ctx,original,c"message".as_ptr())} else if JS_IsObject(original)!=0 {JS_JSONStringify(ctx,original,JS_UNDEFINED,JS_UNDEFINED)} else {JS_DupValue(ctx,original)};
    JS_FreeValue(ctx,original);
    let mut len = 0usize;
    let c = JS_ToCStringLen2(ctx, &mut len, val, 0);
    assert!(!c.is_null()); serialization_u32(out, len as u32);
    out.extend_from_slice(core::slice::from_raw_parts(c.cast::<u8>(), len));
    JS_FreeCString(ctx, c); JS_FreeValue(ctx, val);
}
unsafe fn serialization_module_init(ctx:*mut JSContext,m:*mut JSModuleDef)->i32 {JS_SetModuleExport(ctx,m,c"x".as_ptr(),JS_NewInt32(ctx,42))}
unsafe fn serialization_context(rt: *mut JSRuntime) -> *mut JSContext {
    let ctx = JS_NewContextRaw(rt); assert!(!ctx.is_null());
    #[cfg(serialization_buffers)] { assert_eq!(JS_AddIntrinsicTypedArrays(ctx),0); }
    assert_eq!(JS_AddIntrinsicEval(ctx), 0); assert_eq!(JS_AddIntrinsicJSON(ctx), 0); assert_eq!(JS_AddIntrinsicPromise(ctx),0);let module=JS_NewCModule(ctx,c"missing".as_ptr(),Some(serialization_module_init));assert!(!module.is_null());assert_eq!(JS_AddModuleExport(ctx,module,c"x".as_ptr()),0); ctx
}
pub unsafe fn serialization_fixture() -> Vec<u8> {
    let rt = JS_NewRuntime(); assert!(!rt.is_null()); let mut out = Vec::new();
    let mut sab_host=Box::new(SerializationSABHost {entries:Vec::new()});let sab_functions=JSSharedArrayBufferFunctions{sab_alloc:Some(serialization_sab_alloc),sab_free:Some(serialization_sab_free),sab_dup:Some(serialization_sab_dup),sab_opaque:core::ptr::addr_of_mut!(*sab_host).cast()};JS_SetSharedArrayBufferFunctions(rt,&sab_functions);
    let ctx = serialization_context(rt);
    for source in include_str!("quickjs_json_cases.txt").lines() {
        let input = std::ffi::CString::new(source).unwrap();
        let val = JS_Eval(ctx, input.as_ptr(), source.len(), c"serialization.js".as_ptr(), 0);
        serialization_value(&mut out, ctx, val); JS_FreeValue(ctx, val);
    }
    for source in include_str!("quickjs_json_direct_cases.txt").lines() {
        for flags in 0..2 {
            let input=std::ffi::CString::new(source).unwrap();
            let parsed=JS_ParseJSON2(ctx,input.as_ptr(),source.len(),c"json-input".as_ptr(),flags);
            let value=if JS_IsException(parsed)!=0 {parsed} else {JS_JSONStringify(ctx,parsed,JS_UNDEFINED,JS_UNDEFINED)};
            serialization_value(&mut out,ctx,value);JS_FreeValue(ctx,value);JS_FreeValue(ctx,parsed);
        }
    }
    JS_FreeContext(ctx);
    let cases=include_str!("quickjs_serialization_cases.txt").to_string();
    #[cfg(serialization_buffers)] let cases=cases+include_str!("quickjs_serialization_buffer_cases.txt");
    for source in cases.lines() {
        let ctx = serialization_context(rt); let input = std::ffi::CString::new(source).unwrap();
        let val = JS_Eval(ctx, input.as_ptr(), source.len(), c"serialization.js".as_ptr(), 0);
        if JS_IsException(val) != 0 { serialization_u32(&mut out, u32::MAX); serialization_value(&mut out,ctx,val); JS_FreeContext(ctx); continue; }
        for flags in 0..16 {
            serialization_u32(&mut out, flags as u32);
            let mut len = 0usize;let mut sab_tab=core::ptr::null_mut();let mut sab_len=0usize;let data=JS_WriteObject2(ctx,&mut len,val,flags,&mut sab_tab,&mut sab_len);
            serialization_u32(&mut out, (!data.is_null()) as u32);
            if data.is_null() { serialization_value(&mut out, ctx, JS_EXCEPTION); continue; }
            serialization_u32(&mut out, len as u32); serialization_emit_bytes(&mut out,data,len,sab_tab,sab_len);
            if flags==0 || flags==8 {
                for truncated in 0..len {
                    let value=JS_ReadObject(ctx,data,truncated,flags&8);
                    let encoded=if JS_IsException(value)!=0 {value} else {JS_JSONStringify(ctx,value,JS_UNDEFINED,JS_UNDEFINED)};
                    serialization_value(&mut out,ctx,encoded);JS_FreeValue(ctx,encoded);JS_FreeValue(ctx,value);
                }
            }
            for rom in 0..2 {
                let value = JS_ReadObject(ctx, data, len, flags & 13 | rom * 2);
                let encoded = if JS_IsException(value) != 0 { value } else { JS_JSONStringify(ctx, value, JS_UNDEFINED, JS_UNDEFINED) };
                serialization_value(&mut out, ctx, encoded); JS_FreeValue(ctx, encoded); JS_FreeValue(ctx, value);
            }
            js_free(ctx,sab_tab.cast());js_free(ctx, data.cast());
        }
        JS_FreeValue(ctx, val); JS_FreeContext(ctx);
    }
    for (source,module) in include_str!("quickjs_serialization_compile_cases.txt").lines().map(|s|(s,false)).chain(include_str!("quickjs_serialization_module_cases.txt").lines().map(|s|(s,true))) {
        let ctx = serialization_context(rt); let input = std::ffi::CString::new(source).unwrap();
        let mut retained=Vec::new();
        let val = JS_Eval(ctx, input.as_ptr(), source.len(), c"serialization.js".as_ptr(), JS_EVAL_FLAG_COMPILE_ONLY | if module {JS_EVAL_TYPE_MODULE} else {0});
        for rom in 0..2 {
            let mut len=0usize; let data=JS_WriteObject(ctx,&mut len,val,JS_WRITE_OBJ_BYTECODE);
            serialization_u32(&mut out,(!data.is_null()) as u32);
            if !data.is_null() {serialization_u32(&mut out,len as u32);out.extend_from_slice(core::slice::from_raw_parts(data,len));
                let b=JS_ReadObject(ctx,data,len,JS_READ_OBJ_BYTECODE|rom*JS_READ_OBJ_ROM_DATA);
                let r=if JS_IsException(b)!=0 { b } else if module && JS_ResolveModule(ctx,b)<0 {JS_FreeValue(ctx,b);JS_EXCEPTION} else {JS_EvalFunction(ctx,b)};
                serialization_value(&mut out,ctx,r);JS_FreeValue(ctx,r);retained.push(data);
            } else {serialization_value(&mut out,ctx,JS_EXCEPTION);}
        }
        JS_FreeValue(ctx,val);JS_FreeContext(ctx);for data in retained {js_free_rt(rt,data.cast());}
    }
    JS_FreeRuntime(rt);assert!(sab_host.entries.is_empty());out
}
