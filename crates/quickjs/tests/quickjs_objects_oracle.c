static JSValue fn_a(JSContext*ctx,JSValueConst t,int argc,JSValueConst*argv){return JS_UNDEFINED;}
static JSValue fn_b(JSContext*ctx,JSValueConst t,int argc,JSValueConst*argv){return JS_NULL;}
static JSValue class_call(JSContext*ctx,JSValueConst v,JSValueConst t,int argc,JSValueConst*argv,int flags){return JS_UNDEFINED;}
int main(void){
#include "quickjs_object_layouts.inc"
 for(int v=0;v<256;v++){
  JSClosureVar c={0};c.closure_type=v;c.is_lexical=v>>3;c.is_const=v>>4;c.var_kind=v>>2;num(*(uint8_t*)&c,1);num(((uint8_t*)&c)[1],1);num(c.closure_type,4);num(c.is_lexical,4);num(c.is_const,4);num(c.var_kind,4);
  JSBytecodeVarDef d={0};d.is_const=v;d.is_lexical=v>>1;d.is_captured=v>>2;d.has_scope=v>>3;d.var_kind=v>>4;num(((uint8_t*)&d)[offsetof(JSBytecodeVarDef,scope_next)+sizeof(int)],1);
  JSFunctionBytecode b={0};b.has_prototype=v;b.has_simple_parameter_list=v>>1;b.is_derived_class_constructor=v>>2;b.need_home_object=v>>3;b.func_kind=v>>4;b.new_target_allowed=v>>6;b.super_call_allowed=v>>7;b.super_allowed=~v;b.arguments_allowed=(~v)>>1;b.has_debug=(~v)>>2;b.read_only_bytecode=(~v)>>3;b.is_direct_or_indirect_eval=(~v)>>4;uint8_t*p=(uint8_t*)&b+offsetof(JSFunctionBytecode,js_mode)+1;num(p[0],1);num(p[1],1);num(b.func_kind,4);num(b.has_debug,4);
  JSObject o={0};o.is_std_array_prototype=v;o.extensible=v>>1;o.free_mark=v>>2;o.is_exotic=v>>3;o.fast_array=v>>4;o.is_constructor=v>>5;o.has_immutable_prototype=v>>6;o.tmp_mark=v>>7;o.is_HTMLDDA=v;num(((uint8_t*)&o)[sizeof(JSGCObjectHeader)],1);num(((uint8_t*)&o)[sizeof(JSGCObjectHeader)+1],1);
  JSModuleDef m={0};m.has_tla=v;m.resolved=~v;m.func_created=v-128;m.status=v;m.eval_has_exception=v;num(m.has_tla,4);num(m.resolved,4);num(m.func_created,4);num(m.status,4);num(m.eval_has_exception,4);
 }
 JSRuntime rt={0};JSContext ctx={0};ctx.rt=&rt;JSClass classes[JS_CLASS_INIT_COUNT]={0};rt.class_count=JS_CLASS_INIT_COUNT;rt.class_array=classes;for(int i=0;i<JS_CLASS_INIT_COUNT;i++)classes[i].call=i&1?class_call:NULL;
 JSProxyData proxy={0};JSObject object={0};uint64_t rng=0x123456789abcdef0ULL;
 for(int step=0;step<16384;step++){
  rng^=rng<<13;rng^=rng>>7;rng^=rng<<17;object.class_id=step%JS_CLASS_INIT_COUNT;object.u.opaque=NULL;if(object.class_id==JS_CLASS_PROXY){proxy.is_func=step%256;object.u.proxy_data=&proxy;}
  if(object.class_id==JS_CLASS_C_FUNCTION){object.u.cfunc.c_function.generic=step&1?fn_a:fn_b;object.u.cfunc.magic=(int16_t)rng;}
  JSValue val=step%7==0?JS_UNDEFINED:JS_MKPTR(JS_TAG_OBJECT,&object);num(JS_GetClassID(val),4);num(JS_IsFunction(&ctx,val),4);num(JS_IsCFunction(&ctx,val,fn_a,(int16_t)rng),4);num(JS_IsCFunction(&ctx,val,fn_b,(int16_t)rng),4);num(JS_IsCFunction(&ctx,val,NULL,(int16_t)rng),4);num(JS_SetConstructorBit(&ctx,val,(int)(uint32_t)rng),4);num(JS_IsConstructor(&ctx,val),4);num(JS_IsError(&ctx,val),4);JS_SetUncatchableException(&ctx,(int)(uint32_t)rng);num(rt.current_exception_is_uncatchable,4);JS_SetIsHTMLDDA(&ctx,val);num(JS_IsHTMLDDA(&ctx,val),4);
  JS_SetOpaque(val,(void*)(uintptr_t)rng);num((uintptr_t)JS_GetOpaque(val,object.class_id),8);num((uintptr_t)JS_GetOpaque(val,(uint32_t)object.class_id+1),8);
 }
 struct host h={0};h.trace=1469598103934665603ULL;js_malloc_init(&rt.malloc_ctx);rt.malloc_ctx.mf=(JSMallocFunctions){host_malloc,host_free,host_realloc,NULL};rt.malloc_ctx.malloc_state.opaque=&h;JSContext *heap_ctx=js_mallocz_rt(&rt,sizeof(JSContext));js_rc(heap_ctx)->ref_count=1;for(int i=0;i<1024;i++){num(JS_DupContext(heap_ctx)==heap_ctx,4);num(js_rc(heap_ctx)->ref_count,4);}js_free_rt(&rt,heap_ctx);
 init_list_head(&rt.job_list);struct list_head jobs[32];for(int i=0;i<32;i++){num(JS_IsJobPending(&rt),4);list_add_tail(&jobs[i],&rt.job_list);}for(int i=0;i<32;i++){list_del(&jobs[i]);num(JS_IsJobPending(&rt),4);}
 for(int n=0;n<=256;n++){
  int hash_size=4;while(hash_size<n)hash_size*=2;JSShape*sh=js_mallocz_rt(&rt,get_shape_size(hash_size,n));sh->prop_hash_mask=hash_size-1;sh->prop_size=n;sh->prop_count=n;JSShapeProperty*pr=get_shape_prop(sh);JSProperty*values=js_mallocz_rt(&rt,sizeof(JSProperty)*n);object.shape=sh;object.prop=values;
  for(int i=0;i<n;i++){JSAtom atom=1+(i*263)%129;pr[i].atom=atom;int k=atom&sh->prop_hash_mask;pr[i].hash_next=sh->hash_table[k];sh->hash_table[k]=i+1;values[i].u.value=JS_NewInt32(&ctx,i);}
  for(int atom=0;atom<512;atom++){JSShapeProperty*a=find_own_property1(&object,atom);JSProperty*v=(JSProperty*)(uintptr_t)1;JSShapeProperty*b=find_own_property(&v,&object,atom);num(a?1+a-pr:0,4);num(b?1+b-pr:0,4);num(v?1+v-values:0,4);if(v)num(JS_VALUE_GET_INT(v->u.value),4);}
  js_free_rt(&rt,sh);js_free_rt(&rt,values);
 }
 for(int n=0;n<256;n++){JSProperty p={0};p.u.init.realm_and_id=((uintptr_t)0x123456789abcdef0ULL)|(n&3);num((uintptr_t)js_autoinit_get_realm(&p),8);num(js_autoinit_get_id(&p),4);}num(h.calls,4);num(h.live,4);num(h.trace,8);return 0;
}
