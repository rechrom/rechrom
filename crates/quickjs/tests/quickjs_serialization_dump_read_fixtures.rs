unsafe fn dump_read_result(ctx:*mut JSContext,v:JSValue){
 let tag=JS_VALUE_GET_NORM_TAG(v);let text=format!("result {}\n",tag);quickjs_debug_write(text.as_bytes());
 if JS_IsException(v)!=0 {JS_FreeValue(ctx,JS_GetException(ctx));}else{JS_FreeValue(ctx,v);}
}
unsafe fn dump_read_context(rt:*mut JSRuntime)->*mut JSContext {let ctx=JS_NewContext(rt);assert!(!ctx.is_null());let m=JS_NewCModule(ctx,c"missing".as_ptr(),None);assert!(!m.is_null());assert_eq!(JS_AddModuleExport(ctx,m,c"x".as_ptr()),0);ctx}
pub unsafe fn serialization_dump_read_fixture(){
 let rt=JS_NewRuntime();assert!(!rt.is_null());JS_SetMaxStackSize(rt,4*1024*1024);
 quickjs_debug_write(format!("ABI {} {} {}\n",size_of::<BCReaderState>(),offset_of!(BCReaderState,ptr_last),offset_of!(BCReaderState,level)).as_bytes());
 for (id,source) in include_str!("quickjs_serialization_dump_read_cases.txt").lines().enumerate(){
  let ctx=dump_read_context(rt);assert!(!ctx.is_null());let input=std::ffi::CString::new(source).unwrap();let val=JS_Eval(ctx,input.as_ptr(),source.len(),c"dump-read.js".as_ptr(),0);assert_eq!(JS_IsException(val),0,"case {id}");
  for flags in [0,1,8,9] {
   let mut len=0;let data=JS_WriteObject(ctx,&mut len,val,flags);
   if data.is_null(){quickjs_debug_write(format!("write failed {id} {flags}\n").as_bytes());JS_FreeValue(ctx,JS_GetException(ctx));continue;}
   for rom in 0..2 {quickjs_debug_write(format!("case {id} {flags} {rom} {len}\n").as_bytes());let v=JS_ReadObject(ctx,data,len,flags|rom*2);dump_read_result(ctx,v);}
   if flags==8 {for prefix in 0..len {quickjs_debug_write(format!("prefix {id} {prefix}\n").as_bytes());let v=JS_ReadObject(ctx,data,prefix,flags);dump_read_result(ctx,v);}}
   js_free(ctx,data.cast());
  }
  JS_FreeValue(ctx,val);JS_FreeContext(ctx);
 }
 for (id,(source,module)) in include_str!("quickjs_serialization_compile_cases.txt").lines().map(|s|(s,false)).chain(include_str!("quickjs_serialization_module_cases.txt").lines().map(|s|(s,true))).enumerate(){
  let ctx=dump_read_context(rt);let input=std::ffi::CString::new(source).unwrap();let val=JS_Eval(ctx,input.as_ptr(),source.len(),c"dump-read.js".as_ptr(),JS_EVAL_FLAG_COMPILE_ONLY|if module{JS_EVAL_TYPE_MODULE}else{0});assert_eq!(JS_IsException(val),0);
  let mut retained=Vec::new();for rom in 0..2 {let mut len=0;let data=JS_WriteObject(ctx,&mut len,val,JS_WRITE_OBJ_BYTECODE);assert!(!data.is_null());quickjs_debug_write(format!("compiled {id} {rom} {len}\n").as_bytes());let v=JS_ReadObject(ctx,data,len,JS_READ_OBJ_BYTECODE|rom*JS_READ_OBJ_ROM_DATA);dump_read_result(ctx,v);retained.push(data);}
  JS_FreeValue(ctx,val);JS_FreeContext(ctx);for p in retained{js_free_rt(rt,p.cast());}
 }
 let ctx=dump_read_context(rt);for (id,b) in [&[5u8,0,0][..],&[5u8,0,16,0,0,0,0,0,0,0,0,0,0][..]].iter().enumerate(){for flags in [0,4,8,12]{quickjs_debug_write(format!("direct {id} {flags}\n").as_bytes());let v=JS_ReadObject(ctx,b.as_ptr(),b.len(),flags);dump_read_result(ctx,v);}}
 JS_FreeContext(ctx);JS_FreeRuntime(rt);
}

// Rust-only guard for the original C diagnostic table's undefined tag domain.
// There is no claimed C stdout equivalence for tags outside 0..20.
pub unsafe fn serialization_dump_read_malformed_fixture() {
    let rt = JS_NewRuntime(); assert!(!rt.is_null());
    JS_SetMaxStackSize(rt, 4 * 1024 * 1024);
    let ctx = JS_NewContext(rt); assert!(!ctx.is_null());
    for tag in 20..=255u8 {
        for flags in [0, JS_READ_OBJ_BYTECODE, JS_READ_OBJ_REFERENCE, JS_READ_OBJ_ROM_DATA] {
            let buffer = [5u8, 0, tag];
            let result = JS_ReadObject(ctx, buffer.as_ptr(), buffer.len(), flags);
            assert_ne!(JS_IsException(result), 0, "invalid tag {tag}");
            let exception = JS_GetException(ctx);
            assert_ne!(JS_IsError(ctx, exception), 0);
            let name = JS_GetPropertyStr(ctx, exception, c"name".as_ptr());
            let text = JS_ToCString(ctx, name); assert!(!text.is_null());
            assert_eq!(std::ffi::CStr::from_ptr(text), c"SyntaxError");
            JS_FreeCString(ctx, text); JS_FreeValue(ctx, name); JS_FreeValue(ctx, exception);
        }
    }
    JS_FreeContext(ctx); JS_FreeRuntime(rt);
    quickjs_debug_write(b"PASS 944 malformed-tag reads: SyntaxError, no panic\n");
}
