// Full standard context, real compiler/VM/native calls and pending jobs.
pub unsafe fn runtime_intrinsics_combined_fixture() -> Vec<u8> {
    let rt=JS_NewRuntime();assert!(!rt.is_null());
    let ctx=JS_NewContext(rt);assert!(!ctx.is_null());
    let mut bytes=Vec::new();
    for(id,source)in include_str!("quickjs_runtime_intrinsics_combined_cases.txt").lines().enumerate(){
        let text=std::ffi::CString::new(source).unwrap();
        let result=JS_Eval(ctx,text.as_ptr(),source.len(),c"combined.js".as_ptr(),0);
        bytes.extend_from_slice(&(id as u32).to_le_bytes());
        bytes.extend_from_slice(&(JS_VALUE_GET_NORM_TAG(result) as u32).to_le_bytes());
        let value=if JS_IsException(result)!=0{JS_GetException(ctx)}else{result};
        let mut len=0usize;let encoded=JS_ToCStringLen2(ctx,&mut len,value,0);assert!(!encoded.is_null(),"case {id}");
        bytes.extend_from_slice(&(len as u32).to_le_bytes());bytes.extend_from_slice(core::slice::from_raw_parts(encoded.cast(),len));
        JS_FreeCString(ctx,encoded);JS_FreeValue(ctx,value);
        let mut job_ctx=ptr::null_mut();let mut jobs=0;
        loop{let res=JS_ExecutePendingJob(rt,&mut job_ctx);if res==0{break;}assert!(res>0,"job case {id}");jobs+=1;assert!(jobs<10000);}
        bytes.extend_from_slice(&(jobs as u32).to_le_bytes());
    }
    JS_FreeContext(ctx);JS_FreeRuntime(rt);bytes
}
