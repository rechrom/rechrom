// Real parser + VM + Buffer/TypedArray/DataView/Atomics bootstrap and results.
// RawContext plus genuine native Function object, Eval and TypedArray intrinsics.
// ArrayIterator prototype uses the exact BaseObjects initialization fragment.
// Full BaseObjects bootstrap is deliberately outside this differential fixture.
pub unsafe fn buffers_combined_fixture(lines: &[String]) -> Vec<u8> {
    let mut out=Vec::new();
    let rt=JS_NewRuntime();assert!(!rt.is_null());
    let ctx=JS_NewContextRaw(rt);assert!(!ctx.is_null());
    let ft=JSCFunctionType{generic_magic:Some(js_function_constructor)};
    (*ctx).function_ctor=JS_NewCFunction2(ctx,ft.generic,c"Function".as_ptr(),1,JS_CFUNC_constructor_or_func_magic,JS_FUNC_NORMAL as i32);
    assert_eq!(JS_IsException((*ctx).function_ctor),0);
    (*ctx).array_proto_values=JS_GetProperty(ctx,*(*ctx).class_proto.add(JS_CLASS_ARRAY as usize),crate::quickjs_atom::JS_ATOM_values);
    *(*ctx).class_proto.add(JS_CLASS_ARRAY_ITERATOR as usize)=JS_NewObjectProtoList(ctx,*(*ctx).class_proto.add(JS_CLASS_ITERATOR as usize),ptr::addr_of!(js_array_iterator_proto_funcs).cast(),js_array_iterator_proto_funcs.len() as i32);
    assert_eq!(JS_IsException(*(*ctx).class_proto.add(JS_CLASS_ARRAY_ITERATOR as usize)),0);
    assert_eq!(JS_AddIntrinsicEval(ctx),0);
    assert_eq!(JS_AddIntrinsicTypedArrays(ctx),0);
    (*rt).can_block=1;
    for (index,line) in lines.iter().enumerate() {
        let source=std::ffi::CString::new(line.as_bytes()).unwrap();
        let result=JS_Eval(ctx,source.as_ptr(),line.len(),c"buffers.js".as_ptr(),0);
        out.extend_from_slice(&(index as u32).to_le_bytes());
        out.extend_from_slice(&JS_VALUE_GET_NORM_TAG(result).to_le_bytes());
        let value=if JS_IsException(result)!=0 {JS_GetException(ctx)} else {result};
        let mut len=0;
        let encoded=JS_ToCStringLen2(ctx,&mut len,value,0);
        assert!(!encoded.is_null(),"string conversion case {index}");
        out.extend_from_slice(&(len as u32).to_le_bytes());
        out.extend_from_slice(core::slice::from_raw_parts(encoded.cast::<u8>(),len));
        JS_FreeCString(ctx,encoded);JS_FreeValue(ctx,value);
    }
    JS_FreeContext(ctx);JS_FreeRuntime(rt);out
}
