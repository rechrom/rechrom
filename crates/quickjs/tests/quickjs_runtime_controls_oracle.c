static int interrupt(JSRuntime*rt,void*opaque){return 0;}
static void *sab_alloc(void*opaque,size_t n){return NULL;}
static void sab_free(void*opaque,void*p){}
static void sab_dup(void*opaque,void*p){}
int main(void){
 JSRuntime rt={0};JSContext ctx={0};JSStackFrame frame={0};ctx.rt=&rt;JS_SetRuntimeInfo(NULL,"info");
 uint64_t rng=0x123456789abcdef0ULL;
 for(int step=0;step<16384;step++){
  rng^=rng<<13;rng^=rng>>7;rng^=rng<<17;JS_SetRuntimeOpaque(&rt,(void*)(uintptr_t)rng);JS_SetContextOpaque(&ctx,(void*)(uintptr_t)~rng);JS_SetMemoryLimit(&rt,rng);JS_SetGCThreshold(&rt,~rng);JS_SetRuntimeInfo(&rt,step&1?"info":NULL);JS_SetInterruptHandler(&rt,step&1?interrupt:NULL,(void*)(uintptr_t)(rng>>1));JS_SetCanBlock(&rt,(int)(uint32_t)rng);JS_SetStripInfo(&rt,(int)(uint32_t)rng);
  JSSharedArrayBufferFunctions sf={step&1?sab_alloc:NULL,step&2?sab_free:NULL,step&4?sab_dup:NULL,(void*)(uintptr_t)(rng>>2)};JS_SetSharedArrayBufferFunctions(&rt,&sf);
  rt.stack_top=rng;JS_SetMaxStackSize(&rt,step%8?~rng:0);rt.current_stack_frame=step&1?&frame:NULL;frame.js_mode=(int)(uint32_t)rng;
  num((uintptr_t)JS_GetRuntimeOpaque(&rt),8);num((uintptr_t)JS_GetContextOpaque(&ctx),8);num(JS_GetRuntime(&ctx)==&rt,4);num(rt.malloc_ctx.malloc_state.malloc_limit,8);num(rt.malloc_gc_threshold,8);num(rt.rt_info!=NULL,4);num(rt.interrupt_handler!=NULL,4);num((uintptr_t)rt.interrupt_opaque,8);num(rt.can_block,4);num(JS_GetStripInfo(&rt),4);
  num(rt.sab_funcs.sab_alloc!=NULL,4);num(rt.sab_funcs.sab_free!=NULL,4);num(rt.sab_funcs.sab_dup!=NULL,4);num((uintptr_t)rt.sab_funcs.sab_opaque,8);num(rt.stack_size,8);num(rt.stack_limit,8);num(is_strict_mode(&ctx),4);
 }return 0;
}
