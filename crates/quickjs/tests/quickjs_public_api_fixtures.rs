unsafe fn api_magic(ctx: *mut JSContext, _: JSValueConst, argc: i32, _: *mut JSValueConst, magic: i32) -> JSValue {
    JS_NewInt32(ctx, magic + argc)
}
unsafe fn api_native(ctx: *mut JSContext, _: JSValueConst, argc: i32, _: *mut JSValueConst) -> JSValue {
    JS_NewInt32(ctx, 20 + argc)
}
static mut api_exports: [JSCFunctionListEntry; 3] = [
    JS_PROP_INT32_DEF(c"number".as_ptr(), 7, JS_PROP_C_W_E),
    JS_PROP_STRING_DEF(c"text".as_ptr(), c"ok".as_ptr(), JS_PROP_C_W_E),
    JS_CFUNC_DEF(c"native".as_ptr(), 1, Some(api_native)),
];
unsafe fn api_init(ctx: *mut JSContext, module: *mut JSModuleDef) -> i32 {
    JS_SetModuleExportList(ctx, module, ptr::addr_of!(api_exports).cast(), 3)
}
pub unsafe fn public_api_fixture() -> Vec<u8> {
    let rt = JS_NewRuntime(); assert!(!rt.is_null());
    let ctx = JS_NewContext(rt); assert!(!ctx.is_null());
    let mut bytes = Vec::new();
    let fun = JS_NewCFunctionMagic(ctx, Some(api_magic), c"magic".as_ptr(), 1, JS_CFUNC_generic_magic, 17);
    assert_eq!(JS_IsException(fun), 0);
    let mut args = [JS_NewInt32(ctx, 1)];
    let returned = JS_Call(ctx, fun, JS_UNDEFINED, 1, args.as_mut_ptr());
    let mut integer=0; assert_eq!(JS_ToInt32(ctx, &mut integer, returned),0);
    bytes.extend_from_slice(&integer.to_le_bytes());
    JS_FreeValue(ctx, returned); JS_FreeValue(ctx, fun);
    let obj = JS_NewObject(ctx);
    JS_SetOpaque(obj, 0x12345usize as *mut c_void);
    let mut class_id=0; let opaque=JS_GetAnyOpaque(obj,&mut class_id);
    bytes.extend_from_slice(&class_id.to_le_bytes());
    bytes.extend_from_slice(&(opaque as usize as u64).to_le_bytes());
    JS_FreeValue(ctx,obj);
    let mut class_id=999; let opaque=JS_GetAnyOpaque(JS_NULL,&mut class_id);
    assert!(opaque.is_null());bytes.extend_from_slice(&class_id.to_le_bytes());
    let module=JS_NewCModule(ctx,c"native-module".as_ptr(),Some(api_init));assert!(!module.is_null());
    assert_eq!(JS_AddModuleExportList(ctx,module,ptr::addr_of!(api_exports).cast(),3),0);
    let source=c"import {number,text,native} from 'native-module';globalThis.api_result=number+':'+text+':'+native(1)";
    let value=JS_Eval(ctx,source.as_ptr(),source.to_bytes().len(),c"api.js".as_ptr(),JS_EVAL_TYPE_MODULE);
    assert_eq!(JS_IsException(value),0);JS_FreeValue(ctx,value);
    loop { let mut job_ctx=ptr::null_mut();let status=JS_ExecutePendingJob(rt,&mut job_ctx);assert!(status>=0);if status==0 {break;} }
    let global=JS_GetGlobalObject(ctx);let value=JS_GetPropertyStr(ctx,global,c"api_result".as_ptr());
    let mut len=0;let text=JS_ToCStringLen(ctx,&mut len,value);assert!(!text.is_null());
    bytes.extend_from_slice(core::slice::from_raw_parts(text.cast::<u8>(),len));
    JS_FreeCString(ctx,text);JS_FreeValue(ctx,value);JS_FreeValue(ctx,global);
    JS_FreeContext(ctx);JS_FreeRuntime(rt);bytes
}
