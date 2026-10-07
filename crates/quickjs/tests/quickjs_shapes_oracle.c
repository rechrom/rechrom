static JSShape*pool[512];
static uint32_t shape_id(JSShape*p){if(!p)return 0;for(int i=0;i<512;i++)if(pool[i]==p)return i+1;return 0xffffffff;}
static void shape_snapshot(JSRuntime*rt,struct host*h){
 num(rt->shape_hash_bits,4);num(rt->shape_hash_size,4);num(rt->shape_hash_count,4);num(h->calls,4);num(h->live,4);num(h->trace,8);if(!rt->shape_hash)return;
 for(int i=0;i<rt->shape_hash_size;i++){int n=0;for(JSShape*p=rt->shape_hash[i];p;p=p->shape_hash_next)n++;num(n,4);for(JSShape*p=rt->shape_hash[i];p;p=p->shape_hash_next)num(shape_id(p),4);}
}
int main(void){
 num(sizeof(JSProperty),4);num(_Alignof(JSProperty),4);num(sizeof(JSShapeProperty),4);num(_Alignof(JSShapeProperty),4);num(sizeof(JSShape),4);num(_Alignof(JSShape),4);num(offsetof(JSShape,hash_table),4);num(offsetof(JSShape,proto),4);
 uint32_t values[]={0,1,0x3ffffff,0x4000000,0xffffffff};for(int i=0;i<5;i++)for(int j=0;j<128;j++){JSShapeProperty p={0};p.hash_next=values[i];p.flags=j;p.atom=values[4-i];num(*(uint32_t*)&p,4);num(p.hash_next,4);num(p.flags,4);num(p.atom,4);}
 for(int fail=0;fail<=16;fail++){
  JSRuntime rt={0};struct host h={0};h.trace=1469598103934665603ULL;js_malloc_init(&rt.malloc_ctx);rt.malloc_ctx.mf=(JSMallocFunctions){host_malloc,host_free,host_realloc,NULL};rt.malloc_ctx.malloc_state.opaque=&h;h.fail=fail==1?1:0;num(init_shape_hash(&rt),4);if(!rt.shape_hash){shape_snapshot(&rt,&h);num(h.live,4);continue;}
  int linked[512]={0};for(int i=0;i<512;i++){int group=i/8,n=i%8;int hash_size=4<<(group%3);size_t size=get_shape_size(hash_size,8);pool[i]=js_mallocz_rt(&rt,size);JSShape*p=pool[i];js_rc(p)->ref_count=i%7+1;p->prop_hash_mask=hash_size-1;p->prop_size=8;p->prop_count=n;p->proto=(JSObject*)(uintptr_t)(0x123456789abc0000ULL+group*8);p->hash=shape_initial_hash(p->proto);JSShapeProperty*pr=get_shape_prop(p);for(int j=0;j<n;j++){pr[j].atom=group*17+j+1;pr[j].flags=(group+j)%64;p->hash=shape_hash(shape_hash(p->hash,pr[j].atom),pr[j].flags);}num(size,4);num((uint8_t*)pr-(uint8_t*)p,4);num(js_rc(js_dup_shape(p))->ref_count,4);}
  h.fail=fail>1?h.calls+fail-1:0;uint64_t rng=0x123456789abcdef0ULL;
  for(int step=0;step<4096;step++){rng^=rng<<13;rng^=rng>>7;rng^=rng<<17;int i=(rng>>32)&511;if(linked[i])js_shape_hash_unlink(&rt,pool[i]);else js_shape_hash_link(&rt,pool[i]);linked[i]^=1;
   if(step%37==0)num(resize_shape_hash(&rt,1+(step/37)%10),4);
   num(shape_id(find_hashed_shape_proto(&rt,pool[i]->proto)),4);int n=pool[i]->prop_count;JSAtom atom=(i/8)*17+n+1;int flags=((i/8)+n)%64;num(shape_id(find_hashed_shape_prop(&rt,pool[i],atom,flags)),4);num(shape_id(find_hashed_shape_prop(&rt,pool[i],atom,flags+64)),4);if(step%128==0)shape_snapshot(&rt,&h);
  }
  for(int i=0;i<512;i++){if(linked[i])js_shape_hash_unlink(&rt,pool[i]);js_free_rt(&rt,pool[i]);}shape_snapshot(&rt,&h);js_free_rt(&rt,rt.shape_hash);num(h.live,4);num(h.calls,4);num(h.trace,8);
 }return 0;
}
