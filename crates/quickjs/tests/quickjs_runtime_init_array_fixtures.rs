unsafe fn dump_array_fixture(out:&mut Vec<u8>,ctx:*mut JSContext,v:JSValue){dump_val(out,v);if JS_IsObject(v)!=0{dump_object(out,JS_VALUE_GET_PTR(v).cast());let l=JS_GetProperty(ctx,v,crate::quickjs_atom::JS_ATOM_length);dump_val(out,l);let mut n=0i64;JS_ToInt64(ctx,&mut n,l);JS_FreeValue(ctx,l);for i in 0..n{let x=JS_GetPropertyInt64(ctx,v,i);dump_val(out,x);JS_FreeValue(ctx,x);}}}
unsafe fn array_fixtures(out:&mut Vec<u8>,ctx:*mut JSContext,classes:&mut [JSClass]){
 classes[JS_CLASS_ARRAY_ITERATOR as usize].finalizer=Some(js_array_iterator_finalizer);
 for kind in 0..3 {for length in [0,1,2,5,16] {for idx in [-20,-1,0,1,20] {for method in 0..24 {
 let obj=if kind==2{JS_NewObject(ctx)}else{JS_NewArray(ctx)};
 for i in 0..length{if kind==1&&i%3==1{continue;}let v=match i%6{0=>JS_NewInt32(ctx,3-i),1=>JS_NewString(ctx,c"雪z".as_ptr()),2=>JS_UNDEFINED,3=>JS_NULL,4=>JS_NewFloat64(ctx,-0.0),_=>JS_NewBool(ctx,1)};assert!(JS_DefinePropertyValueUint32(ctx,obj,i as u32,v,JS_PROP_C_W_E)>=0);}
 assert!(JS_SetProperty(ctx,obj,crate::quickjs_atom::JS_ATOM_length,JS_NewInt32(ctx,length))>=0);
 let mut args=[JS_NewInt32(ctx,idx),JS_NewInt32(ctx,2),JS_NewInt32(ctx,77)];
 let ret=match method{
0=>{js_array_at(ctx,obj,1,args.as_mut_ptr())},
1=>{js_array_with(ctx,obj,2,args.as_mut_ptr())},
2=>{args[0]=JS_NewInt32(ctx,0);args[1]=JS_NewInt32(ctx,idx);js_array_includes(ctx,obj,2,args.as_mut_ptr())},
3=>{args[0]=JS_NewInt32(ctx,0);args[1]=JS_NewInt32(ctx,idx);js_array_indexOf(ctx,obj,2,args.as_mut_ptr())},
4=>{args[0]=JS_NewInt32(ctx,0);args[1]=JS_NewInt32(ctx,idx);js_array_lastIndexOf(ctx,obj,2,args.as_mut_ptr())},
5=>{js_array_fill(ctx,obj,3,args.as_mut_ptr())},
6=>{js_array_pop(ctx,obj,0,args.as_mut_ptr(), 0)},
7=>{js_array_pop(ctx,obj,0,args.as_mut_ptr(), 1)},
8=>{js_array_push(ctx,obj,2,args.as_mut_ptr(), 0)},
9=>{js_array_push(ctx,obj,2,args.as_mut_ptr(), 1)},
10=>{js_array_reverse(ctx,obj,0,args.as_mut_ptr())},
11=>{js_array_toReversed(ctx,obj,0,args.as_mut_ptr())},
12=>{js_array_slice(ctx,obj,2,args.as_mut_ptr(), 0)},
13=>{js_array_slice(ctx,obj,3,args.as_mut_ptr(), 1)},
14=>{js_array_toSpliced(ctx,obj,3,args.as_mut_ptr())},
15=>{js_array_copyWithin(ctx,obj,3,args.as_mut_ptr())},
16=>{args[0]=JS_UNDEFINED;js_array_sort(ctx,obj,0,args.as_mut_ptr())},
17=>{args[0]=JS_UNDEFINED;js_array_toSorted(ctx,obj,0,args.as_mut_ptr())},
18=>{args[0]=JS_NewString(ctx,c"|雪|".as_ptr());js_array_join(ctx,obj,1,args.as_mut_ptr(), 0)},
19=>{js_array_concat(ctx,obj,2,args.as_mut_ptr())},
20=>{js_array_of(ctx,JS_UNDEFINED,3,args.as_mut_ptr())},
21=>{args[0]=obj;args[1]=JS_UNDEFINED;js_array_from(ctx,JS_UNDEFINED,1,args.as_mut_ptr())},
22=>{args[0]=JS_NewInt32(ctx,2);js_array_flatten(ctx,obj,1,args.as_mut_ptr(), 0)},
23=>{args[0]=JS_UNDEFINED;js_array_toString(ctx,obj,0,args.as_mut_ptr())},
_=>std::process::abort()};
 num(out,kind as u64,4);num(out,length as u64,4);num(out,idx as u64,4);num(out,method as u64,4);dump_array_fixture(out,ctx,ret);dump_array_fixture(out,ctx,obj);dump_exception(out,ctx);if method==18{JS_FreeValue(ctx,args[0]);}JS_FreeValue(ctx,ret);JS_FreeValue(ctx,obj);
 }}}}
}
