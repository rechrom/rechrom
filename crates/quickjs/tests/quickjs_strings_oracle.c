int main(void){
 JSRuntime rt={0};struct host h={0};h.trace=1469598103934665603ULL;js_malloc_init(&rt.malloc_ctx);rt.malloc_ctx.mf=(JSMallocFunctions){host_malloc,host_free,host_realloc,NULL};rt.malloc_ctx.malloc_state.opaque=&h;uint64_t rng=0x123456789abcdef0ULL;
 for(int step=0;step<32768;step++){
  rng^=rng<<13;rng^=rng>>7;rng^=rng<<17;uint16_t a[128],b[128],dst[128];int n=step%65,w1=(step>>1)&1,w2=(step>>2)&1;
  for(int i=0;i<128;i++){a[i]=(rng>>((i%8)*8))+i*263;b[i]=step%3==0?a[i]:(uint16_t)(a[i]+step%257);if(!w1)a[i]&=255;if(!w2)b[i]&=255;}
  JSString*p=make_string(&rt,a,n,w1),*q=make_string(&rt,b,n+step%3,w2);int pos1=n?step%n:0,pos2=n?((step/65)%n):0,len=n-(pos1>pos2?pos1:pos2);
  num(js_string_memcmp(p,pos1,q,pos2,len),4);num(js_string_eq(NULL,p,q),4);num(js_string_eq(NULL,p,p),4);num(js_string_compare(NULL,p,q),4);num(js_string_compare(NULL,q,p),4);num(JS_IsEmptyString(JS_MKPTR(JS_TAG_STRING,p)),4);num(JS_IsEmptyString(JS_UNDEFINED),4);
  memset(dst,0xa5,sizeof(dst));copy_str16(dst+3,p,pos1,len);for(int i=0;i<128;i++)num(dst[i],2);
  num(hash_string(p,(uint32_t)rng),4);JSStringRope rope={0};rope.left=JS_MKPTR(JS_TAG_STRING,p);rope.right=JS_MKPTR(JS_TAG_STRING,q);JSStringRope root={0};root.left=JS_MKPTR(JS_TAG_STRING_ROPE,&rope);root.right=JS_MKPTR(JS_TAG_STRING,p);num(hash_string_rope(JS_MKPTR(JS_TAG_STRING_ROPE,&root),(uint32_t)rng),4);
  num(count_ascii(p->u.str8,(size_t)n<<w1),4);js_free_string(&rt,p);js_free_string(&rt,q);
 }
 num(h.calls,4);num(h.live,4);num(h.trace,8);return 0;
}
