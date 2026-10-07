/* Test driver appended to unchanged original C declarations/functions. */
static void num(uint64_t n,int size){for(int i=0;i<size;i++)putchar((n>>(8*i))&255);}
struct host{int calls,fail,live;uint64_t trace;void*pointers[16384];size_t sizes[16384];int count;};
static int ptr_index(struct host*h,void*p){for(int i=0;i<h->count;i++)if(h->pointers[i]==p)return i;return -1;}
static void event(struct host*h,int op,size_t n){h->calls++;h->trace=(h->trace^op)*1099511628211ULL;h->trace=(h->trace^n)*1099511628211ULL;}
static void *host_malloc(JSMallocState*s,size_t n){struct host*h=s->opaque;event(h,1,n);if(h->calls==h->fail)return NULL;void*p=malloc(n);if(p){memset(p,0xa5,n);h->pointers[h->count]=p;h->sizes[h->count++]=n;h->live++;}return p;}
static void host_free(JSMallocState*s,void*p){struct host*h=s->opaque;int i=ptr_index(h,p);event(h,2,i<0?0:h->sizes[i]);if(p){free(p);h->live--;h->count--;h->pointers[i]=h->pointers[h->count];h->sizes[i]=h->sizes[h->count];}}
static void *host_realloc(JSMallocState*s,void*p,size_t n){struct host*h=s->opaque;event(h,3,n);if(h->calls==h->fail)return NULL;int i=ptr_index(h,p);size_t old=i<0?0:h->sizes[i];void*q=realloc(p,n);if(q){if(n>old)memset((uint8_t*)q+old,0xa5,n-old);if(i<0){i=h->count++;h->live++;}h->pointers[i]=q;h->sizes[i]=n;}return q;}
static void dump_string(const JSString*p){num(p->len,4);num(p->is_wide_char,4);num(p->hash,4);num(p->atom_type,4);num(p->hash_next,4);num(js_rc((void*)p)->ref_count,4);for(int i=0;i<p->len;i++)num(string_get(p,i),4);}
static void snapshot(JSRuntime*rt,struct host*h){
 num(rt->atom_hash_size,4);num(rt->atom_count,4);num(rt->atom_size,4);num(rt->atom_count_resize,4);num(rt->atom_free_index,4);num(h->calls,4);num(h->live,4);num(h->trace,8);
 for(int i=0;i<rt->atom_hash_size;i++)num(rt->atom_hash[i],4);
 for(int i=0;i<rt->atom_size;i++){JSAtomStruct*p=rt->atom_array[i];num(atom_is_free(p),4);if(atom_is_free(p))num(atom_get_free(p),4);else dump_string(p);}
}
static void cleanup(JSRuntime*rt,struct host*h){
 for(int i=0;i<rt->atom_size;i++)if(!atom_is_free(rt->atom_array[i]))js_free_rt(rt,rt->atom_array[i]);js_free_rt(rt,rt->atom_array);js_free_rt(rt,rt->atom_hash);
 num(h->live,4);num(h->trace,8);num(h->calls,4);
 /* Reclaim any retained blocks in failed initialization without touching C. */
 for(int i=0;i<h->count;i++)free(h->pointers[i]);
}
static JSString *make_string(JSRuntime*rt,const uint16_t*units,int len,int wide){JSString*p=js_alloc_string_rt(rt,len,wide);if(p){for(int i=0;i<len;i++){if(wide)p->u.str16[i]=units[i];else p->u.str8[i]=units[i];}if(!wide)p->u.str8[len]=0;}return p;}
int main(void){
#define SIZE(T) num(sizeof(T),4);num(_Alignof(T),4)
#define OFF(T,F) num(offsetof(T,F),4)
 SIZE(JSRuntime);SIZE(JSContext);SIZE(JSClass);SIZE(JSStackFrame);SIZE(JSGCObjectHeader);SIZE(JSWeakRefHeader);SIZE(JSVarRef);SIZE(JSBigInt);SIZE(JSBigIntBuf);SIZE(JSFloat64Union);SIZE(JSString);SIZE(JSStringRope);
 OFF(JSRuntime,rt_info);OFF(JSRuntime,atom_hash);OFF(JSRuntime,context_list);OFF(JSRuntime,malloc_gc_threshold);OFF(JSRuntime,current_exception);OFF(JSRuntime,current_stack_frame);OFF(JSRuntime,job_list);OFF(JSRuntime,u);OFF(JSRuntime,sab_funcs);OFF(JSRuntime,shape_hash);OFF(JSRuntime,user_opaque);
 OFF(JSContext,rt);OFF(JSContext,binary_object_size);OFF(JSContext,class_proto);OFF(JSContext,native_error_proto);OFF(JSContext,loaded_modules);OFF(JSContext,eval_internal);OFF(JSContext,user_opaque);OFF(JSVarRef,pvalue);OFF(JSVarRef,value);OFF(JSBigInt,tab);OFF(JSString,u);OFF(JSStringRope,left);
 uint32_t vals[]={0,1,0x7fffffff,0x80000000,0xffffffff};for(int i=0;i<5;i++)for(int wide=0;wide<2;wide++)for(int type=0;type<8;type++){JSString s={0};s.len=vals[i];s.is_wide_char=wide;s.hash=vals[4-i];s.atom_type=type;num(*(uint32_t*)&s,4);num(((uint32_t*)&s)[1],4);num(s.len,4);num(s.is_wide_char,4);num(s.hash,4);num(s.atom_type,4);}
 for(int fail=0;fail<=64;fail++){
  JSRuntime rt={0};struct host h={0};h.fail=fail;h.trace=1469598103934665603ULL;js_malloc_init(&rt.malloc_ctx);rt.malloc_ctx.mf=(JSMallocFunctions){host_malloc,host_free,host_realloc,NULL};rt.malloc_ctx.malloc_state.opaque=&h;int ret=JS_InitAtoms(&rt);num(ret,4);snapshot(&rt,&h);cleanup(&rt,&h);
 }
 for(int fail=0;fail<=16;fail++){
  JSRuntime rt={0};JSContext ctx={0};ctx.rt=&rt;struct host h={0};h.trace=1469598103934665603ULL;js_malloc_init(&rt.malloc_ctx);rt.malloc_ctx.mf=(JSMallocFunctions){host_malloc,host_free,host_realloc,NULL};rt.malloc_ctx.malloc_state.opaque=&h;if(JS_InitAtoms(&rt))return 2;h.fail=h.calls+fail;
  JSAtom ids[2048]={0};uint64_t rng=0x123456789abcdef0;
  for(int step=0;step<6144;step++){
   rng^=rng<<13;rng^=rng>>7;rng^=rng<<17;int slot=(rng>>32)&2047;int op=step<2048?0:step%7;JSAtom id=0;
   if(op==0||op==1||op==2){uint16_t units[32];int len=op==2?0:3+(rng%24),wide=(rng>>8)&1;for(int i=0;i<len;i++)units[i]=(uint16_t)((rng>>((i%8)*8))+step%151);if(!wide)for(int i=0;i<len;i++)units[i]&=255;JSString*p=op==2?NULL:make_string(&rt,units,len,wide);if(op==2||p)id=__JS_NewAtom(&rt,p,op==0?JS_ATOM_TYPE_STRING:op==1?JS_ATOM_TYPE_GLOBAL_SYMBOL:JS_ATOM_TYPE_SYMBOL);if(ids[slot])JS_FreeAtomRT(&rt,ids[slot]);ids[slot]=id;}
   else if(op==3){if(ids[slot])id=JS_DupAtom(&ctx,ids[slot]);if(id)JS_FreeAtom(&ctx,id);}
   else if(op==4){char str[32];int len=snprintf(str,sizeof(str),"name-%d",step%151);id=__JS_NewAtomInit(&rt,str,len,JS_ATOM_TYPE_STRING);if(ids[slot])JS_FreeAtomRT(&rt,ids[slot]);ids[slot]=id;}
   else if(op==5){char str[32];int len=snprintf(str,sizeof(str),"name-%d",step%151);id=__JS_FindAtom(&rt,str,len,JS_ATOM_TYPE_GLOBAL_SYMBOL);if(id)JS_FreeAtomRT(&rt,id);}
   else{if(ids[slot])JS_FreeAtomRT(&rt,ids[slot]);ids[slot]=0;}
   num(id,4);if(id){num(JS_AtomGetKind(&ctx,id),4);num(JS_AtomIsString(&ctx,id),4);num(JS_AtomSymbolHasDescription(&ctx,id),4);}if(step%128==0)snapshot(&rt,&h);
  }
  for(int i=0;i<2048;i++)if(ids[i])JS_FreeAtomRT(&rt,ids[i]);snapshot(&rt,&h);
  // Reusing an atom string across string/symbol/private types must preserve
  // ownership and copy exactly when the original string is already an atom.
  for(int type=1;type<=4;type++){JSAtom a=__JS_NewAtomInit(&rt,"shared",6,JS_ATOM_TYPE_STRING);if(a){JSAtomStruct*p=rt.atom_array[a];js_rc(p)->ref_count++;JSAtom b=__JS_NewAtom(&rt,p,type);num(b,4);if(b)JS_FreeAtomRT(&rt,b);JS_FreeAtomRT(&rt,a);}}
  snapshot(&rt,&h);cleanup(&rt,&h);
 }
 const char*texts[]={"","0","00","1","012","2147483647","2147483648","4294967295","4294967296","9999999999","12345678901","-1","+1","1.0","1e2"," 1","1 ","NaN"};
 JSRuntime rt={0};struct host h={0};h.trace=1469598103934665603ULL;js_malloc_init(&rt.malloc_ctx);rt.malloc_ctx.mf=(JSMallocFunctions){host_malloc,host_free,host_realloc,NULL};rt.malloc_ctx.malloc_state.opaque=&h;JS_InitAtoms(&rt);JSContext ctx={0};ctx.rt=&rt;
 for(int k=0;k<18;k++)for(int wide=0;wide<2;wide++){uint16_t units[32];int len=strlen(texts[k]);for(int i=0;i<len;i++)units[i]=(uint8_t)texts[k][i];JSString*p=make_string(&rt,units,len,wide);uint32_t n=0xa5a5a5a5;num(is_num_string(&n,p),4);num(n,4);num(hash_string(p,1),4);JSAtom a=JS_NewAtomStr(&ctx,p);num(a,4);JS_FreeAtomRT(&rt,a);}
 cleanup(&rt,&h);return 0;
}
