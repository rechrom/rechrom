/* Appended after unchanged allocator declarations/functions extracted from
   official quickjs.c by the Rust test. No translated C implementation here. */
static void num(uint64_t n,int size){for(int i=0;i<size;i++)putchar((n>>(8*i))&255);}
struct host{int calls,fail,live;uint64_t trace;};
struct allocation{size_t size;uint64_t padding;uint8_t data[];};
static struct allocation *allocation_header(const void *p){return (struct allocation*)((uint8_t*)p-sizeof(struct allocation));}
static void event(struct host*h,int op,size_t n){h->trace=(h->trace^op)*1099511628211ULL;h->trace=(h->trace^n)*1099511628211ULL;h->calls++;}
static void *host_malloc(JSMallocState*s,size_t n){struct host*h=s->opaque;event(h,1,n);if(h->calls==h->fail)return NULL;struct allocation*a=malloc(sizeof(*a)+n);if(!a)return NULL;a->size=n;memset(a->data,0xa5,n);h->live++;return a->data;}
static void host_free(JSMallocState*s,void*p){struct host*h=s->opaque;event(h,2,p?allocation_header(p)->size:0);if(p){h->live--;free(allocation_header(p));}}
static void *host_realloc(JSMallocState*s,void*p,size_t n){struct host*h=s->opaque;event(h,3,n);if(h->calls==h->fail)return NULL;struct allocation*a=allocation_header(p);size_t old=a->size;a=realloc(a,sizeof(*a)+n);if(!a)return NULL;if(n>old)memset(a->data+old,0xa5,n-old);a->size=n;return a->data;}
static size_t host_usable_size(const void*p){return allocation_header(p)->size;}
static uint64_t hash_data(const uint8_t*p,size_t n){uint64_t h=1469598103934665603ULL;for(size_t i=0;i<n;i++)h=(h^p[i])*1099511628211ULL;return h;}
static void snapshot(JSMallocContext*s,void*p,struct host*h){
 num(p!=NULL,4);if(p){JSMallocBlockHeader*b=js_rc(p);size_t n=__js_malloc_usable_size(s,p);num(n,8);num(b->u.block_idx,4);num(b->block_size_idx,4);if(n){num(b->gc_obj_type,4);num(b->mark,4);num(b->ref_count,4);num(hash_data(p,n),8);}}
 num(h->calls,4);num(h->live,4);num(h->trace,8);
 for(int i=0;i<JS_MALLOC_BLOCK_SIZE_COUNT;i++){
  struct list_head*el;int n=0;list_for_each(el,&s->arena_list[i])n++;num(n,4);
  list_for_each(el,&s->arena_list[i]){JSMallocArena*ar=list_entry(el,JSMallocArena,link);num(ar->n_used_blocks,4);num(ar->n_blocks,4);num(ar->first_free_block,4);}
  n=0;list_for_each(el,&s->free_arena_list[i])n++;num(n,4);
 }
}
int main(void){
#define SIZE(T) num(sizeof(T),4);num(_Alignof(T),4)
#define OFF(T,F) num(offsetof(T,F),4)
 SIZE(JSMallocBlockHeader);SIZE(JSMallocLargeBlockHeader);SIZE(JSMallocArena);SIZE(JSMallocContext);OFF(JSMallocBlockHeader,user_data);OFF(JSMallocBlockHeader,ref_count);OFF(JSMallocArena,blocks);OFF(JSMallocArena,n_used_blocks);OFF(JSMallocContext,zero_size_block);OFF(JSMallocContext,mf);OFF(JSMallocContext,malloc_state);
 for(int i=0;i<31;i++)num(js_malloc_block_sizes[i],4);for(int i=0;i<=4096;i++)num(get_block_size_index(i),4);
 for(int fail=0;fail<=32;fail++){
  JSMallocContext s;js_malloc_init(&s);struct host h={0};h.fail=fail;h.trace=1469598103934665603ULL;s.malloc_state.opaque=&h;s.mf=(JSMallocFunctions){host_malloc,host_free,host_realloc,host_usable_size};
  void *blocks[256]={0};uint64_t rng=0x123456789abcdef0;
  for(int step=0;step<4096;step++){
   rng^=rng<<13;rng^=rng>>7;rng^=rng<<17;int slot=(rng>>32)&255;size_t n=step<768?(size_t)(step%768):(size_t)((rng>>8)%8192);void *p=blocks[slot];
   if(step%5==0){__js_free(&s,p);blocks[slot]=NULL;p=NULL;}
   else{void*q=__js_realloc(&s,p,n);snapshot(&s,q,&h);if(q||!n){blocks[slot]=q;p=q;}}
   if(p){size_t size=__js_malloc_usable_size(&s,p);if(size){JSMallocBlockHeader*b=js_rc(p);b->gc_obj_type=step&127;b->mark=(step>>7)&1;b->ref_count=step+1;for(size_t i=0;i<size;i++)((uint8_t*)p)[i]=(step+i)&255;}}
   snapshot(&s,p,&h);
  }
  for(int i=0;i<256;i++)__js_free(&s,blocks[i]);snapshot(&s,NULL,&h);if(h.live)return 2;
  void*z=__js_malloc(&s,0);snapshot(&s,z,&h);void*p=__js_realloc(&s,z,1);snapshot(&s,p,&h);__js_free(&s,p);__js_free(&s,z);snapshot(&s,NULL,&h);
  // Exercise the optional and zero-reporting usable-size callback paths.
  p=__js_malloc(&s,1024);s.mf.js_malloc_usable_size=NULL;num(__js_malloc_usable_size(&s,p),8);__js_free(&s,p);num(__js_malloc_usable_size(&s,NULL),8);
 }
 return 0;
}
