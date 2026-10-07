// Original quickjs.c #if 0 helpers. Bellard/Gordon MIT.
// These private source functions are not registered in the official default
// intrinsic tables. Translating them retains source coverage without enabling
// a new JavaScript property or changing the default API surface.
// c: quickjs.c:44628.
unsafe fn js_number___toInteger(ctx:*mut JSContext,_this:JSValueConst,_argc:i32,argv:*mut JSValueConst)->JSValue {
    JS_ToIntegerFree(ctx,JS_DupValue(ctx,*argv))
}
// c: quickjs.c:44634.
unsafe fn js_number___toLength(ctx:*mut JSContext,_this:JSValueConst,_argc:i32,argv:*mut JSValueConst)->JSValue {
    let mut value=0i64;
    if JS_ToLengthFree(ctx,&mut value,JS_DupValue(ctx,*argv))!=0{return JS_EXCEPTION;}
    JS_NewInt64(ctx,value)
}
// c: quickjs.c:45208.
unsafe fn js_string___isSpace(ctx:*mut JSContext,_this:JSValueConst,_argc:i32,argv:*mut JSValueConst)->JSValue {
    let mut c=0;
    if JS_ToInt32(ctx,&mut c,*argv)!=0{return JS_EXCEPTION;}
    JS_NewBool(ctx,crate::libunicode::lre_is_space(c as u32))
}
// c: quickjs.c:47282.
unsafe fn js___date_getTimezoneOffset(ctx:*mut JSContext,_this:JSValueConst,_argc:i32,argv:*mut JSValueConst)->JSValue {
    let mut value=0.0;
    if JS_ToFloat64(ctx,&mut value,*argv)!=0{return JS_EXCEPTION;}
    if value.is_nan(){__JS_NewFloat64(ctx,value)}else{JS_NewInt32(ctx,getTimezoneOffset(value as i64))}
}
// c: quickjs.c:47295.
unsafe fn js_get_prototype_from_ctor(ctx:*mut JSContext,ctor:JSValueConst,def_proto:JSValueConst)->JSValue {
    let mut proto=JS_GetProperty(ctx,ctor,crate::quickjs_atom::JS_ATOM_prototype);
    if JS_IsException(proto)!=0{return proto;}
    if JS_IsObject(proto)==0{JS_FreeValue(ctx,proto);proto=JS_DupValue(ctx,def_proto);}
    proto
}
// c: quickjs.c:47310.
unsafe fn js___date_create(ctx:*mut JSContext,_this:JSValueConst,_argc:i32,argv:*mut JSValueConst)->JSValue {
    let proto=js_get_prototype_from_ctor(ctx,*argv,*argv.add(1));
    if JS_IsException(proto)!=0{return proto;}
    let obj=JS_NewObjectProtoClass(ctx,proto,JS_CLASS_DATE);
    JS_FreeValue(ctx,proto);
    if JS_IsException(obj)==0{JS_SetObjectData(ctx,obj,JS_DupValue(ctx,*argv.add(2)));}
    obj
}
