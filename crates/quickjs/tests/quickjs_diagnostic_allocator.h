#undef malloc
#undef realloc
#undef free
static size_t diagnostic_capacity(const void*p){return p?*(const size_t*)((const uint8_t*)p-16):0;}
static void*diagnostic_malloc(JSMallocState*s,size_t n){if(s->malloc_size+n>s->malloc_limit)return NULL;uint8_t*p=malloc(n+16);if(!p)return NULL;*(size_t*)p=n;s->malloc_count++;s->malloc_size+=n;return p+16;}
static void diagnostic_free(JSMallocState*s,void*p){if(p){size_t n=diagnostic_capacity(p);s->malloc_count--;s->malloc_size-=n;free((uint8_t*)p-16);}}
static void*diagnostic_realloc(JSMallocState*s,void*p,size_t n){if(!p)return diagnostic_malloc(s,n);if(!n){diagnostic_free(s,p);return NULL;}size_t old=diagnostic_capacity(p);if(s->malloc_size-old+n>s->malloc_limit)return NULL;uint8_t*q=realloc((uint8_t*)p-16,n+16);if(!q)return NULL;*(size_t*)q=n;s->malloc_size=s->malloc_size-old+n;return q+16;}
static const JSMallocFunctions DIAGNOSTIC_MALLOC={diagnostic_malloc,diagnostic_free,diagnostic_realloc,diagnostic_capacity};
