/* Unchanged official C runtime/context failure paths. Custom host callbacks
   match the Rust callback policy and expose allocation ownership separately. */
#include "quickjs.c"
#undef malloc
#undef realloc
#undef free
struct IntrinsicOomHost {uint64_t attempts,fail_at,live,bytes,trace;};
static void trace(struct IntrinsicOomHost*h,uint64_t op,size_t n){h->trace=(h->trace^op)*1099511628211ULL;h->trace=(h->trace^n)*1099511628211ULL;}
static void*oom_malloc_cb(JSMallocState*s,size_t n){struct IntrinsicOomHost*h=s->opaque;h->attempts++;trace(h,1,n);if(h->attempts==h->fail_at)return NULL;uint8_t*p=malloc(n+16);if(!p)return NULL;*(size_t*)p=n;h->live++;h->bytes+=n;s->malloc_count++;s->malloc_size+=n;return p+16;}
static void oom_free_cb(JSMallocState*s,void*p){if(!p)return;struct IntrinsicOomHost*h=s->opaque;uint8_t*b=(uint8_t*)p-16;size_t n=*(size_t*)b;trace(h,3,n);h->live--;h->bytes-=n;s->malloc_count--;s->malloc_size-=n;free(b);}
static void*oom_realloc_cb(JSMallocState*s,void*p,size_t n){if(!p)return oom_malloc_cb(s,n);if(n==0){oom_free_cb(s,p);return NULL;}struct IntrinsicOomHost*h=s->opaque;h->attempts++;trace(h,2,n);if(h->attempts==h->fail_at)return NULL;uint8_t*b=(uint8_t*)p-16;size_t old=*(size_t*)b;uint8_t*new=realloc(b,n+16);if(!new)return NULL;*(size_t*)new=n;h->bytes=h->bytes+n-old;s->malloc_size=s->malloc_size+n-old;return new+16;}
static size_t oom_usable_cb(const void*p){return p?*(size_t*)((uint8_t*)p-16):0;}
static void u64(uint64_t x){for(int i=0;i<8;i++)putchar((x>>(i*8))&255);}
int main(int argc,char**argv){assert(argc==2);uint64_t fail=strtoull(argv[1],NULL,10);struct IntrinsicOomHost h={.fail_at=fail,.trace=1469598103934665603ULL};JSMallocFunctions mf={oom_malloc_cb,oom_free_cb,oom_realloc_cb,oom_usable_cb};JSRuntime*rt=JS_NewRuntime2(&mf,&h);JSContext*ctx=rt?JS_NewContext(rt):NULL;u64(rt!=NULL);u64(ctx!=NULL);u64(h.attempts);u64(h.live);u64(h.bytes);u64(h.trace);fflush(stdout);if(ctx)JS_FreeContext(ctx);if(rt)JS_FreeRuntime(rt);u64(h.attempts);u64(h.live);u64(h.bytes);u64(h.trace);fflush(stdout);assert(h.live==0);assert(h.bytes==0);return 0;}
