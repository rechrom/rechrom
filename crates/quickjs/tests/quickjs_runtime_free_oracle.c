int main(void){
 for(int trial=0;trial<512;trial++){
  struct host h={0};h.trace=1469598103934665603ULL;JSMallocState ms={0};ms.opaque=&h;JSRuntime*rt=host_malloc(&ms,sizeof(*rt));memset(rt,0,sizeof(*rt));js_malloc_init(&rt->malloc_ctx);rt->malloc_ctx.mf=(JSMallocFunctions){host_malloc,host_free,host_realloc,NULL};rt->malloc_ctx.malloc_state.opaque=&h;init_list_head(&rt->context_list);init_list_head(&rt->gc_obj_list);init_list_head(&rt->gc_zero_ref_count_list);init_list_head(&rt->tmp_obj_list);init_list_head(&rt->weakref_list);init_list_head(&rt->job_list);JS_InitAtoms(rt);init_shape_hash(rt);rt->current_exception=JS_UNINITIALIZED;
  rt->class_count=JS_CLASS_INIT_COUNT;rt->class_array=js_mallocz_rt(rt,sizeof(JSClass)*rt->class_count);for(int j=1;j<rt->class_count;j++){rt->class_array[j].class_id=j;rt->class_array[j].class_name=__JS_NewAtomInit(rt,"held-class",10,JS_ATOM_TYPE_STRING);}
  JSContext*ctx=js_mallocz_rt(rt,sizeof(*ctx));js_rc(ctx)->ref_count=1;ctx->rt=rt;init_list_head(&ctx->loaded_modules);list_add_tail(&ctx->link,&rt->context_list);add_gc_object(rt,&ctx->header,JS_GC_OBJ_TYPE_JS_CONTEXT);ctx->class_proto=js_mallocz_rt(rt,sizeof(JSValue)*rt->class_count);
  JSObject*objects[4];for(int j=0;j<4;j++){JSObject*p=js_mallocz_rt(rt,sizeof(*p));objects[j]=p;js_rc(p)->ref_count=1;add_gc_object(rt,&p->header,JS_GC_OBJ_TYPE_JS_OBJECT);p->class_id=JS_CLASS_OBJECT;p->shape=js_mallocz_rt(rt,get_shape_size(4,1));js_rc(p->shape)->ref_count=1;add_gc_object(rt,&p->shape->header,JS_GC_OBJ_TYPE_SHAPE);p->shape->prop_hash_mask=3;p->shape->prop_count=1;p->shape->prop_size=1;get_shape_prop(p->shape)->atom=__JS_NewAtomInit(rt,"edge",4,JS_ATOM_TYPE_STRING);p->prop=js_mallocz_rt(rt,sizeof(JSProperty));}
  for(int j=0;j<4;j++)objects[j]->prop[0].u.value=JS_DupValueRT(rt,JS_MKPTR(JS_TAG_OBJECT,objects[(j+1)%4]));ctx->class_proto[1]=JS_DupValueRT(rt,JS_MKPTR(JS_TAG_OBJECT,objects[0]));
  for(int j=0;j<trial%8+1;j++){JSJobEntry*e=js_mallocz_rt(rt,sizeof(*e)+4*sizeof(JSValue));e->argc=4;e->realm=JS_DupContext(ctx);for(int k=0;k<4;k++)e->argv[k]=JS_DupValueRT(rt,JS_MKPTR(JS_TAG_OBJECT,objects[k]));list_add_tail(&e->link,&rt->job_list);}
  uint16_t units[5]={1,2,3,4,5};rt->current_exception=JS_MKPTR(JS_TAG_STRING,make_string(rt,units,5,trial%2));
  JS_FreeContext(ctx);for(int j=0;j<4;j++)JS_FreeValueRT(rt,JS_MKPTR(JS_TAG_OBJECT,objects[j]));JS_FreeRuntime(rt);num(h.calls,4);num(h.live,4);num(h.trace,8);assert(h.live==0);
 }
 return 0;
}
