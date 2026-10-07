/* True host hooks and failing JSMallocFunctions, matching the Rust fixture. */
typedef struct {size_t call,fail_at,live;} BufferFaultState;
static void *buffer_fault_malloc(JSMallocState *s,size_t n){BufferFaultState *p=s->opaque;size_t call=p->call++;if(call==p->fail_at)return NULL;void *q=js_def_malloc(s,n);if(q)p->live++;return q;}
static void buffer_fault_free(JSMallocState *s,void *q){if(q)((BufferFaultState*)s->opaque)->live--;js_def_free(s,q);}
static void *buffer_fault_realloc(JSMallocState *s,void *q,size_t n){BufferFaultState *p=s->opaque;size_t call=p->call++;if(call==p->fail_at)return NULL;void *v=js_def_realloc(s,q,n);if(!q&&v)p->live++;if(q&&!n)p->live--;return v;}
typedef struct {uint8_t *blocks[8];int blocks_len;uint64_t events[16];int event_count;} BufferHostState;
static void *buffer_host_alloc(void *opaque,size_t n){BufferHostState *p=opaque;p->events[p->event_count++]=0x100000000ULL+n;uint8_t *q=malloc(n);assert(q);memset(q,0xa5,n);p->blocks[p->blocks_len++]=q;return q;}
static void buffer_host_dup(void *opaque,void *data){((BufferHostState*)opaque)->events[((BufferHostState*)opaque)->event_count++]=0x200000000ULL;}
static void buffer_host_free(void *opaque,void *data){((BufferHostState*)opaque)->events[((BufferHostState*)opaque)->event_count++]=0x300000000ULL;}
static void buffers_dump_api_value(JSContext *ctx,JSValue result){u32(JS_VALUE_GET_NORM_TAG(result));if(JS_IsException(result)){JSValue e=JS_GetException(ctx);size_t n=0;const char *s=JS_ToCStringLen2(ctx,&n,e,0);assert(s);u32(n);fwrite(s,1,n,stdout);JS_FreeCString(ctx,s);JS_FreeValue(ctx,e);}}
static void u64(uint64_t x){for(int i=0;i<8;i++)putchar((x>>(i*8))&255);}
static void buffers_host_oom_fixture(void){
 BufferHostState host={0};JSRuntime *rt=JS_NewRuntime();assert(rt);JSContext *ctx=JS_NewContextRaw(rt);assert(ctx);
 JSSharedArrayBufferFunctions hooks={buffer_host_alloc,buffer_host_free,buffer_host_dup,&host};JS_SetSharedArrayBufferFunctions(rt,&hooks);
 uint64_t maximum=31;JSValue buffer=js_array_buffer_constructor2(ctx,JS_UNDEFINED,7,&maximum,JS_CLASS_SHARED_ARRAY_BUFFER);assert(!JS_IsException(buffer));size_t length=0;uint8_t *data=JS_GetArrayBuffer(ctx,&length,buffer);assert(data);u32(length);fwrite(data,1,length,stdout);
 JSValue args[]={JS_NewInt32(ctx,23)};JSValue result=js_array_buffer_resize(ctx,buffer,1,args,JS_CLASS_SHARED_ARRAY_BUFFER);buffers_dump_api_value(ctx,result);JS_FreeValue(ctx,result);data=JS_GetArrayBuffer(ctx,&length,buffer);u32(length);fwrite(data,1,length,stdout);JS_DetachArrayBuffer(ctx,buffer);JS_GetArrayBuffer(ctx,&length,buffer);u32(length);JS_FreeValue(ctx,buffer);
 uint8_t external[17];memset(external,0x51,17);buffer=JS_NewArrayBuffer(ctx,external,17,NULL,NULL,1);assert(!JS_IsException(buffer));JS_FreeValue(ctx,buffer);
 JSValue bad=js_array_buffer_constructor3(ctx,JS_UNDEFINED,7,&maximum,JS_CLASS_ARRAY_BUFFER,external,NULL,NULL,0);buffers_dump_api_value(ctx,bad);JS_FreeValue(ctx,bad);JS_FreeContext(ctx);JS_FreeRuntime(rt);u32(host.event_count);for(int i=0;i<host.event_count;i++)u64(host.events[i]);for(int i=0;i<host.blocks_len;i++)free(host.blocks[i]);
 uint8_t copy[16384];memset(copy,0x37,sizeof(copy));
 for(int operation=0;operation<4;operation++)for(size_t fail_at=0;fail_at<12;fail_at++){
  BufferFaultState state={.fail_at=SIZE_MAX};JSMallocFunctions allocator={buffer_fault_malloc,buffer_fault_free,buffer_fault_realloc,js_def_malloc_usable_size};rt=JS_NewRuntime2(&allocator,&state);assert(rt);ctx=JS_NewContextRaw(rt);assert(ctx);
  maximum=65536;JSValue baseline=operation>=2?js_array_buffer_constructor2(ctx,JS_UNDEFINED,16384,&maximum,JS_CLASS_ARRAY_BUFFER):JS_UNDEFINED;assert(!JS_IsException(baseline));JSValue view=JS_UNDEFINED;
  if(operation>=2){JSValue argv[]={baseline,JS_UNDEFINED,JS_UNDEFINED};view=JS_NewTypedArray(ctx,1,argv,JS_TYPED_ARRAY_UINT8);assert(!JS_IsException(view));}
  state.call=0;state.fail_at=fail_at;
  if(operation==0)result=JS_NewArrayBufferCopy(ctx,copy,sizeof(copy));
  else if(operation==1){JSValue argv[]={JS_NewInt32(ctx,4096),JS_UNDEFINED,JS_UNDEFINED};result=JS_NewTypedArray(ctx,1,argv,JS_TYPED_ARRAY_UINT32);}
  else{JSValue argv[]={JS_NewInt32(ctx,65536)};result=operation==2?js_array_buffer_resize(ctx,baseline,1,argv,JS_CLASS_ARRAY_BUFFER):js_array_buffer_transfer(ctx,baseline,1,argv,1);}
  state.fail_at=SIZE_MAX;buffers_dump_api_value(ctx,result);
  if(operation!=1&&JS_IsObject(result)){length=0;data=JS_GetArrayBuffer(ctx,&length,result);if(data)u32(length);}
  if(operation>=2)u32(JS_VALUE_GET_OBJ(view)->u.array.count);
  JS_FreeValue(ctx,result);JS_FreeValue(ctx,view);JS_FreeValue(ctx,baseline);JS_FreeContext(ctx);JS_FreeRuntime(rt);assert(!state.live);u32(state.live);
 }
}
