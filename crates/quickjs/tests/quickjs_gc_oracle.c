static JSObject*objects[64];static JSShape*shapes[64];
static uint32_t gc_id(JSGCObjectHeader*p){for(int i=0;i<64;i++){if(p==(JSGCObjectHeader*)objects[i])return i+1;if(p==(JSGCObjectHeader*)shapes[i])return i+65;}return 0xffffffff;}
static void record_finalizer(JSRuntime*rt,JSValue v){JSObject*p=JS_VALUE_GET_OBJ(v);num(gc_id(&p->header),4);num(rt->gc_phase,4);num(p->free_mark,4);num(p->shape==NULL,4);num(p->prop==NULL,4);num(JS_IsLiveObject(rt,v),4);}
static void dump_gc_list(struct list_head*head){int n=0;struct list_head*el;list_for_each(el,head)n++;num(n,4);list_for_each(el,head){JSGCObjectHeader*p=list_entry(el,JSGCObjectHeader,link);num(gc_id(p),4);num(js_rc(p)->gc_obj_type,4);num(js_rc(p)->ref_count,4);num(js_rc(p)->mark,4);}}
static void gc_snapshot(JSRuntime*rt,struct host*h){num(rt->gc_phase,4);dump_gc_list(&rt->gc_obj_list);dump_gc_list(&rt->tmp_obj_list);dump_gc_list(&rt->gc_zero_ref_count_list);num(h->calls,4);num(h->live,4);num(h->trace,8);}
static void collect(JSRuntime*rt,struct host*h){gc_decref(rt);gc_snapshot(rt,h);gc_scan(rt);gc_snapshot(rt,h);gc_free_cycles(rt);gc_snapshot(rt,h);}
int main(void){
 JSRuntime rt={0};struct host h={0};h.trace=1469598103934665603ULL;js_malloc_init(&rt.malloc_ctx);rt.malloc_ctx.mf=(JSMallocFunctions){host_malloc,host_free,host_realloc,NULL};rt.malloc_ctx.malloc_state.opaque=&h;init_list_head(&rt.gc_obj_list);init_list_head(&rt.gc_zero_ref_count_list);init_list_head(&rt.tmp_obj_list);init_list_head(&rt.context_list);JS_InitAtoms(&rt);rt.current_exception=JS_UNINITIALIZED;
 JSClass classes[JS_CLASS_INIT_COUNT+1]={0};rt.class_count=JS_CLASS_INIT_COUNT+1;rt.class_array=classes;classes[JS_CLASS_INIT_COUNT].finalizer=record_finalizer;JSContext ctx={0};ctx.rt=&rt;
 JSAtom names[16];for(int i=0;i<16;i++){char str[32];int n=snprintf(str,sizeof(str),"property-%d",i);names[i]=__JS_NewAtomInit(&rt,str,n,JS_ATOM_TYPE_STRING);}
 uint64_t rng=0x123456789abcdef0ULL;
 for(int trial=0;trial<512;trial++){
  for(int i=0;i<64;i++){objects[i]=js_mallocz_rt(&rt,sizeof(JSObject));js_rc(objects[i])->ref_count=1;add_gc_object(&rt,&objects[i]->header,JS_GC_OBJ_TYPE_JS_OBJECT);objects[i]->class_id=JS_CLASS_INIT_COUNT;objects[i]->weakref_count=(trial%4==0&&i%7==0)?1:0;shapes[i]=js_mallocz_rt(&rt,get_shape_size(4,4));js_rc(shapes[i])->ref_count=1;add_gc_object(&rt,&shapes[i]->header,JS_GC_OBJ_TYPE_SHAPE);shapes[i]->prop_hash_mask=3;shapes[i]->prop_size=4;shapes[i]->prop_count=4;objects[i]->shape=shapes[i];objects[i]->prop=js_mallocz_rt(&rt,sizeof(JSProperty)*4);}
  for(int i=0;i<64;i++){
   JSShapeProperty*pr=get_shape_prop(shapes[i]);for(int j=0;j<4;j++){rng^=rng<<13;rng^=rng>>7;rng^=rng<<17;pr[j].atom=JS_DupAtomRT(&rt,names[(i+j)%16]);int kind=rng%4,target=(rng>>32)&63;
    if(kind==0){objects[i]->prop[j].u.value=JS_DupValueRT(&rt,JS_MKPTR(JS_TAG_OBJECT,objects[target]));}
    else if(kind==1){objects[i]->prop[j].u.value=JS_NewInt32(&ctx,rng);}
    else if(kind==2){pr[j].flags=JS_PROP_GETSET;objects[i]->prop[j].u.getset.getter=objects[target];JS_DupValueRT(&rt,JS_MKPTR(JS_TAG_OBJECT,objects[target]));if(rng&8){objects[i]->prop[j].u.getset.setter=objects[(target+1)&63];JS_DupValueRT(&rt,JS_MKPTR(JS_TAG_OBJECT,objects[(target+1)&63]));}}
    else{uint16_t units[5]={(uint16_t)rng,(uint16_t)(rng>>16),1,2,3};JSString*s=make_string(&rt,units,5,(rng>>12)&1);objects[i]->prop[j].u.value=JS_MKPTR(JS_TAG_STRING,s);}
   }
   if((trial+i)%3==0){shapes[i]->proto=objects[(i+13)&63];JS_DupValueRT(&rt,JS_MKPTR(JS_TAG_OBJECT,shapes[i]->proto));}
  }
  gc_snapshot(&rt,&h);for(int i=0;i<64;i+=2)JS_FreeValue(&ctx,JS_MKPTR(JS_TAG_OBJECT,objects[i]));collect(&rt,&h);for(int i=1;i<64;i+=2)JS_FreeValueRT(&rt,JS_MKPTR(JS_TAG_OBJECT,objects[i]));collect(&rt,&h);
  if(trial%4==0)for(int i=0;i<64;i+=7){num(JS_IsLiveObject(&rt,JS_MKPTR(JS_TAG_OBJECT,objects[i])),4);num(objects[i]->class_id,4);num(objects[i]->u.opaque==NULL,4);num(js_rc(objects[i])->ref_count,4);num(js_rc(objects[i])->mark,4);js_free_rt(&rt,objects[i]);}
  num(list_empty(&rt.gc_obj_list),4);num(list_empty(&rt.tmp_obj_list),4);snapshot(&rt,&h);
 }
 // Immediate-value ownership and pending exception replacement/take semantics.
 for(int i=0;i<2048;i++){num(JS_HasException(&ctx),4);JSValue val=JS_NewInt32(&ctx,i);num(JS_VALUE_GET_TAG(JS_Throw(&ctx,val)),4);JS_SetUncatchableException(&ctx,1);num(JS_VALUE_GET_TAG(JS_Throw(&ctx,JS_NewInt32(&ctx,i+1))),4);num(rt.current_exception_is_uncatchable,4);num(JS_VALUE_GET_INT(JS_GetException(&ctx)),4);num(JS_HasException(&ctx),4);}
 for(int i=0;i<16;i++)JS_FreeAtomRT(&rt,names[i]);cleanup(&rt,&h);return 0;
}
