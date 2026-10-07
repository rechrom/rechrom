static void finalizer(JSRuntime *rt,JSValue val){}
static void gc_mark(JSRuntime *rt,JSValue val,JS_MarkFunc *f){}
static JSValue call(JSContext *ctx,JSValueConst v,JSValueConst t,int argc,JSValueConst *argv,int flags){return JS_UNDEFINED;}
static uint64_t hash_num(uint64_t h,uint64_t n){return (h^n)*1099511628211ULL;}
static void class_snapshot(JSRuntime*rt,JSContext*ctx,int count,struct host*h){
 num(rt->class_count,4);uint64_t hash=1469598103934665603ULL;
 for(int i=0;i<rt->class_count;i++){JSClass*c=&rt->class_array[i];hash=hash_num(hash,c->class_id);hash=hash_num(hash,c->class_name);hash=hash_num(hash,c->finalizer!=NULL);hash=hash_num(hash,c->gc_mark!=NULL);hash=hash_num(hash,c->call!=NULL);hash=hash_num(hash,c->exotic!=NULL);}num(hash,8);
 for(int k=0;k<count;k++){int index=ptr_index(h,(uint8_t*)ctx[k].class_proto-sizeof(JSMallocLargeBlockHeader));int n=index<0?0:(h->sizes[index]-sizeof(JSMallocLargeBlockHeader))/sizeof(JSValue);num(n,4);hash=1469598103934665603ULL;for(int i=0;i<n;i++){hash=hash_num(hash,JS_VALUE_GET_TAG(ctx[k].class_proto[i]));hash=hash_num(hash,ctx[k].class_proto[i].u.uint64);}num(hash,8);}
 num(h->calls,4);num(h->live,4);num(h->trace,8);
}
struct test_gc{JSGCObjectHeader h;uint32_t id;};
static void mark(JSRuntime*rt,JSGCObjectHeader*p){num(((struct test_gc*)p)->id,4);}
int main(void){
 JSClassID ids[256]={0};for(int i=0;i<256;i++){num(JS_NewClassID(&ids[i]),4);num(JS_NewClassID(&ids[i]),4);}JSClassID custom=0x87654321;num(JS_NewClassID(&custom),4);num(custom,4);
 JSClassID classes[]={0,1,JS_CLASS_INIT_COUNT-1,JS_CLASS_INIT_COUNT,127,128,191,255,256,511,1023,2047,65535,65536,0xffffffff,1,255,0};
 for(int contexts=0;contexts<=3;contexts++)for(int fail=0;fail<=40;fail++){
  JSRuntime rt={0};JSContext ctx[3]={{0}};struct host h={0};h.trace=1469598103934665603ULL;js_malloc_init(&rt.malloc_ctx);rt.malloc_ctx.mf=(JSMallocFunctions){host_malloc,host_free,host_realloc,NULL};rt.malloc_ctx.malloc_state.opaque=&h;init_list_head(&rt.context_list);JS_InitAtoms(&rt);
  for(int k=0;k<contexts;k++){ctx[k].rt=&rt;list_add_tail(&ctx[k].link,&rt.context_list);}h.fail=h.calls+fail;
  JSClassExoticMethods exotic={0};
  for(int step=0;step<18;step++){char name[32];snprintf(name,sizeof(name),step%3==0?"Object":"test-%d",step%7);JSClassDef def={name,step&1?finalizer:NULL,step&2?gc_mark:NULL,step&4?call:NULL,step&8?&exotic:NULL};num(JS_NewClass(&rt,classes[step],&def),4);num(JS_IsRegisteredClass(&rt,classes[step]),4);class_snapshot(&rt,ctx,contexts,&h);
   for(int k=0;k<contexts;k++)if(ctx[k].class_proto&&rt.class_count)ctx[k].class_proto[0]=JS_NewInt32(&ctx[k],step*7+k);
  }
  for(int i=0;i<rt.class_count;i++)if(rt.class_array[i].class_id)JS_FreeAtomRT(&rt,rt.class_array[i].class_name);
  js_free_rt(&rt,rt.class_array);for(int k=0;k<contexts;k++){js_free_rt(&rt,ctx[k].class_proto);list_del(&ctx[k].link);}snapshot(&rt,&h);cleanup(&rt,&h);
 }
 JSRuntime rt={0};struct host h={0};h.trace=1469598103934665603ULL;js_malloc_init(&rt.malloc_ctx);rt.malloc_ctx.mf=(JSMallocFunctions){host_malloc,host_free,host_realloc,NULL};rt.malloc_ctx.malloc_state.opaque=&h;init_list_head(&rt.gc_obj_list);
 struct test_gc *g[64];for(int i=0;i<64;i++){g[i]=js_mallocz_rt(&rt,sizeof(*g[i]));g[i]->id=i;js_rc(g[i])->ref_count=i+1;js_rc(g[i])->gc_obj_type=99;js_rc(g[i])->mark=1;add_gc_object(&rt,&g[i]->h,i%7);num(js_rc(g[i])->gc_obj_type,4);num(js_rc(g[i])->mark,4);num(js_rc(g[i])->ref_count,4);}
 struct list_head*el;list_for_each(el,&rt.gc_obj_list)num(list_entry(el,struct test_gc,h.link)->id,4);
 for(int i=0;i<64;i+=2)remove_gc_object(&g[i]->h);list_for_each(el,&rt.gc_obj_list)num(list_entry(el,struct test_gc,h.link)->id,4);
 for(int tag=-12;tag<=12;tag++)for(int i=0;i<64;i++)JS_MarkValue(&rt,JS_MKPTR(tag,g[i]),mark);
 for(int i=1;i<64;i+=2)remove_gc_object(&g[i]->h);num(list_empty(&rt.gc_obj_list),4);for(int i=0;i<64;i++)js_free_rt(&rt,g[i]);num(h.live,4);num(h.trace,8);return 0;
}
