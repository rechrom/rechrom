/* Test driver only. Engine functions come from unchanged official quickjs.c. */
static void writes_conversions_fixtures(JSContext*ctx,struct host*h){
 uint64_t rng=0x9a58723f264b17ceULL;
 for(int i=0;i<4096;i++){
  rng^=rng<<13;rng^=rng>>7;rng^=rng<<17;JSFloat64Union u={.u64=rng};JSValue val=__JS_NewFloat64(ctx,u.d);int32_t n=0;int64_t q=0;uint64_t index=0x5555;
  num(JS_ToInt32Sat(ctx,&n,val),4);num(n,4);num(JS_ToInt32Clamp(ctx,&n,val,-512,512,1024),4);num(n,4);
  num(JS_ToInt64(ctx,&q,val),4);num(q,8);num(JS_ToInt64Sat(ctx,&q,val),4);num(q,8);num(JS_ToInt64Clamp(ctx,&q,val,-4096,4096,8192),4);num(q,8);
  num(JS_ToUint8ClampFree(ctx,&n,val),4);num(n,4);JSValue integer=JS_ToIntegerFree(ctx,val);dump_val(integer);JS_FreeValue(ctx,integer);num(is_safe_integer(JS_VALUE_GET_FLOAT64(val)),4);
  num(JS_ToIndex(ctx,&index,val),4);num(index,8);dump_exception(ctx);num(JS_ToLengthFree(ctx,&q,val),4);num(q,8);
 }
 const char*texts[]={"","  \t\n","0","-0","0xabcdef1234567890123456789","-123456789012345678901234567890","+15","-0x1","0b101","0o77","1n","1e3","1.5","1_2","Infinity","42z","  9007199254740993 \xc2\xa0"};
 for(int i=0;i<17;i++){JSValue val=JS_NewString(ctx,texts[i]);JSValue converted=JS_ToBigInt(ctx,val);dump_val(converted);dump_exception(ctx);JS_FreeValue(ctx,converted);int64_t n=0;num(JS_ToBigInt64(ctx,&n,val),4);num(n,8);dump_exception(ctx);JS_FreeValue(ctx,val);}
 JSValue vals[]={JS_TRUE,JS_FALSE,JS_NULL,JS_UNDEFINED,JS_NewInt32(ctx,1),JS_NewFloat64(ctx,1.5),JS_NewBigInt64(ctx,INT64_MIN),JS_NewBigUint64(ctx,UINT64_MAX)};
 for(int i=0;i<8;i++){JSValue val=vals[i];int64_t n=0;num(JS_ToBigInt64(ctx,&n,val),4);num(n,8);dump_exception(ctx);num(JS_ToInt64Ext(ctx,&n,val),4);num(n,8);dump_exception(ctx);JSValue boxed=JS_ToObject(ctx,val);dump_val(boxed);dump_exception(ctx);if(JS_IsObject(boxed))dump_val(JS_VALUE_GET_OBJ(boxed)->u.object_data);JS_FreeValue(ctx,boxed);JS_FreeValue(ctx,val);}
 for(int fail=0;fail<64;fail++){
  JSValue proto=JS_NewObject(ctx),obj=JS_NewObjectProto(ctx,proto),receiver=JS_NewObject(ctx);JSAtom atom=__JS_AtomFromUInt32(42);JS_DefinePropertyValue(ctx,proto,atom,JS_NewInt32(ctx,7),JS_PROP_C_W_E);if(fail)h->fail=h->calls+fail;
  for(int step=0;step<24;step++){int ret;
   switch(step%8){case 0:ret=JS_SetPropertyInternal(ctx,obj,atom,JS_NewInt32(ctx,step),obj,JS_PROP_THROW);break;case 1:ret=JS_SetPropertyInternal(ctx,proto,atom,JS_NewInt32(ctx,step),receiver,JS_PROP_THROW);break;case 2:ret=JS_SetPropertyStr(ctx,obj,"created",JS_NewInt32(ctx,step));break;case 3:ret=JS_DeletePropertyInt64(ctx,obj,42,JS_PROP_THROW);break;case 4:ret=JS_DefineProperty(ctx,obj,atom,JS_UNDEFINED,JS_UNDEFINED,JS_UNDEFINED,JS_PROP_HAS_GET|JS_PROP_HAS_SET|JS_PROP_HAS_CONFIGURABLE|JS_PROP_CONFIGURABLE);break;case 5:ret=JS_SetPropertyInternal(ctx,obj,atom,JS_NewInt32(ctx,step),obj,JS_PROP_THROW);break;case 6:ret=JS_SetPropertyInternal(ctx,JS_NULL,atom,JS_NewInt32(ctx,step),JS_NULL,0);break;default:ret=JS_SetPropertyInt64(ctx,receiver,-1,JS_NewInt32(ctx,step));break;}
   num(ret,4);dump_exception(ctx);
  }
  h->fail=0;JS_PreventExtensions(ctx,receiver);num(JS_SetPropertyInternal(ctx,proto,atom,JS_NewInt32(ctx,1),receiver,JS_PROP_THROW),4);dump_exception(ctx);dump_object(JS_VALUE_GET_OBJ(obj));dump_object(JS_VALUE_GET_OBJ(receiver));JS_FreeValue(ctx,receiver);JS_FreeValue(ctx,obj);JS_FreeValue(ctx,proto);
 }
 JSValue arr=JS_NewArray(ctx);uint32_t indices[]={0,1,2,7,3,0,UINT32_MAX};for(int i=0;i<7;i++){num(JS_SetPropertyUint32(ctx,arr,indices[i],JS_NewInt32(ctx,indices[i])),4);dump_exception(ctx);}num(JS_SetPropertyStr(ctx,arr,"length",JS_NewInt32(ctx,2)),4);dump_exception(ctx);dump_object(JS_VALUE_GET_OBJ(arr));JS_FreeValue(ctx,arr);
 for(int cls=JS_CLASS_UINT8C_ARRAY;cls<=JS_CLASS_FLOAT64_ARRAY;cls++){
  JSValue obj=JS_NewObjectClass(ctx,cls);JSObject*p=JS_VALUE_GET_OBJ(obj);uint64_t data[4]={0};p->u.array.u.ptr=data;p->u.array.count=4;double values[]={0.5,1.5,2.5,254.5,255.5,-1,NAN,65535,4294967295.0,INFINITY};
  for(int step=0;step<10;step++){JSValue val=(cls==JS_CLASS_BIG_INT64_ARRAY||cls==JS_CLASS_BIG_UINT64_ARRAY)?JS_NewBigInt64(ctx,(int64_t)(((uint64_t)step<<60)-17)):JS_NewFloat64(ctx,values[step]);num(JS_SetPropertyValue(ctx,obj,JS_NewInt32(ctx,step%6),val,0),4);dump_exception(ctx);fwrite(data,1,32,stdout);}
  const char*keys[]={"-0","1.5","Infinity","99","ordinary"};for(int i=0;i<5;i++){JSValue val=JS_NewString(ctx,(cls==JS_CLASS_BIG_INT64_ARRAY||cls==JS_CLASS_BIG_UINT64_ARRAY)?"-18446744073709551617":"256.5");num(JS_SetPropertyStr(ctx,obj,keys[i],val),4);dump_exception(ctx);}p->u.array.u.ptr=NULL;p->u.array.count=0;JS_FreeValue(ctx,obj);
 }
}
