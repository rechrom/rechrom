unsafe fn dormant_dump(out:&mut Vec<u8>,ctx:*mut JSContext,value:JSValue){
    out.extend_from_slice(&(JS_VALUE_GET_NORM_TAG(value) as i32).to_le_bytes());
    let val=if JS_IsException(value)!=0{JS_GetException(ctx)}else{JS_DupValue(ctx,value)};
    let mut len=0;let encoded=JS_ToCStringLen2(ctx,&mut len,val,0);assert!(!encoded.is_null());
    out.extend_from_slice(&(len as u32).to_le_bytes());out.extend_from_slice(core::slice::from_raw_parts(encoded.cast(),len));
    JS_FreeCString(ctx,encoded);JS_FreeValue(ctx,val);
}
pub unsafe fn dormant_intrinsics_fixture()->Vec<u8>{
    let rt=JS_NewRuntime();assert!(!rt.is_null());let ctx=JS_NewContext(rt);assert!(!ctx.is_null());let mut out=Vec::new();
    let numbers=[0.0,-0.0,0.1,-0.1,1.9,-1.9,5e-324,1e20,9007199254740991.0,9007199254740992.0,f64::INFINITY,-f64::INFINITY,f64::NAN];
    for number in numbers{let mut arg=JS_NewFloat64(ctx,number);for helper in [js_number___toInteger,js_number___toLength,js_string___isSpace,js___date_getTimezoneOffset]{let value=helper(ctx,JS_UNDEFINED,1,&mut arg);dormant_dump(&mut out,ctx,value);JS_FreeValue(ctx,value);}}
    for c in -1..=0x10000{let mut arg=JS_NewInt32(ctx,c);let value=js_string___isSpace(ctx,JS_UNDEFINED,1,&mut arg);out.push(JS_VALUE_GET_INT(value) as u8);JS_FreeValue(ctx,value);}
    let def_proto=JS_NewObject(ctx);
    for text in [c"({})",c"({prototype:1})",c"({prototype:null})",c"({prototype:{x:42}})",c"({get prototype(){return {x:43}}})",c"({get prototype(){throw new Error('proto')}})"]{
        let ctor=JS_Eval(ctx,text.as_ptr(),text.to_bytes().len(),c"dormant.js".as_ptr(),0);assert!(JS_IsException(ctor)==0);
        let proto=js_get_prototype_from_ctor(ctx,ctor,def_proto);dormant_dump(&mut out,ctx,proto);out.push((JS_VALUE_GET_PTR(proto)==JS_VALUE_GET_PTR(def_proto)) as u8);JS_FreeValue(ctx,proto);
        let data=JS_NewString(ctx,c"retained-date-data".as_ptr());let mut args=[ctor,def_proto,data];let obj=js___date_create(ctx,JS_UNDEFINED,3,args.as_mut_ptr());out.extend_from_slice(&(JS_VALUE_GET_NORM_TAG(obj) as i32).to_le_bytes());if JS_IsException(obj)!=0{dormant_dump(&mut out,ctx,obj);}
        if JS_IsObject(obj)!=0{out.extend_from_slice(&((*JS_VALUE_GET_OBJ(obj)).class_id as u32).to_le_bytes());dormant_dump(&mut out,ctx,(*JS_VALUE_GET_OBJ(obj)).u.object_data);}
        JS_FreeValue(ctx,obj);JS_FreeValue(ctx,data);JS_FreeValue(ctx,ctor);
    }
    JS_FreeValue(ctx,def_proto);JS_FreeContext(ctx);JS_FreeRuntime(rt);out
}
