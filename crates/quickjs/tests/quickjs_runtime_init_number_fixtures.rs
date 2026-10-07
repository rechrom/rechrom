unsafe fn number_fixtures(out:&mut Vec<u8>,ctx:*mut JSContext,classes:&mut [JSClass]){classes[JS_CLASS_NUMBER as usize].finalizer=Some(js_object_data_finalizer);classes[JS_CLASS_BOOLEAN as usize].finalizer=Some(js_object_data_finalizer);let nums=[0.0,-0.0,1.0,-99.0,0.1,1e20,1e21,5e-324,1.7976931348623157e308,f64::INFINITY,-f64::INFINITY,f64::NAN];for number in nums{for box_kind in 0..3{for di in 0..9{for method in 0..13{let mut val=JS_NewFloat64(ctx,number);if box_kind!=0{let obj=JS_NewObjectProtoClass(ctx,JS_NULL,if box_kind==1{JS_CLASS_NUMBER}else{JS_CLASS_BOOLEAN});assert_eq!(JS_SetObjectData(ctx,obj,if box_kind==1{val}else{JS_NewBool(ctx,JS_ToBool(ctx,val))}),0);val=obj;}let digit=if di==8{JS_UNDEFINED}else{JS_NewInt32(ctx,[-2,0,1,2,6,20,100,101][di])};let mut args=[digit,JS_UNDEFINED,JS_UNDEFINED];let ret=match method{
0=>{args[0]=val;js_number_constructor(ctx,JS_UNDEFINED,1,args.as_mut_ptr())},
1=>{args[0]=val;js_number_isNaN(ctx,JS_UNDEFINED,1,args.as_mut_ptr())},
2=>{args[0]=val;js_number_isFinite(ctx,JS_UNDEFINED,1,args.as_mut_ptr())},
3=>{args[0]=val;js_number_isInteger(ctx,JS_UNDEFINED,1,args.as_mut_ptr())},
4=>{args[0]=val;js_number_isSafeInteger(ctx,JS_UNDEFINED,1,args.as_mut_ptr())},
5=>{js_number_valueOf(ctx,val,1,args.as_mut_ptr())},
6=>{js_number_toString(ctx,val,1,args.as_mut_ptr(), 0)},
7=>{js_number_toFixed(ctx,val,1,args.as_mut_ptr())},
8=>{js_number_toExponential(ctx,val,1,args.as_mut_ptr())},
9=>{js_number_toPrecision(ctx,val,1,args.as_mut_ptr())},
10=>{args[0]=val;js_boolean_constructor(ctx,JS_UNDEFINED,1,args.as_mut_ptr())},
11=>{js_boolean_toString(ctx,val,1,args.as_mut_ptr())},
12=>{js_boolean_valueOf(ctx,val,1,args.as_mut_ptr())},
_=>std::process::abort()};dump_val(out,ret);if JS_IsObject(ret)!=0{dump_object(out,JS_VALUE_GET_PTR(ret).cast());}dump_exception(out,ctx);JS_FreeValue(ctx,ret);JS_FreeValue(ctx,val);}}}}
 for text in [c"",c" -42xyz",c"0xFF",c"0b11",c"0o17",c"1e309",c" -Infinity ",c" 1.25e2tail",c"雪",c"9007199254740993",c" .5 ",c"-.0"]{for ri in 0..10{let mut args=[JS_NewString(ctx,text.as_ptr()),if ri==9{JS_UNDEFINED}else{JS_NewInt32(ctx,[-1,0,1,2,8,10,16,36,37][ri])}];let v=js_parseInt(ctx,JS_UNDEFINED,2,args.as_mut_ptr());dump_val(out,v);dump_exception(out,ctx);JS_FreeValue(ctx,v);let v=js_parseFloat(ctx,JS_UNDEFINED,1,args.as_mut_ptr());dump_val(out,v);dump_exception(out,ctx);JS_FreeValue(ctx,v);JS_FreeValue(ctx,args[0]);}}
}
