/* Native threads are used only by the independent C oracle, never Rust runtime. */
typedef struct {uint8_t *memory;int index;int big;char result[16];size_t length;} BufferWaitThread;
static JSValue buffer_thread_ctx(uint8_t *memory,JSRuntime **rt_out,JSContext **ctx_out,int big) {
 JSRuntime *rt=JS_NewRuntime();assert(rt);JSContext *ctx=JS_NewContextRaw(rt);assert(ctx);JS_SetCanBlock(rt,1);
 JSValue buffer=JS_NewArrayBuffer(ctx,memory,16,NULL,NULL,1);assert(!JS_IsException(buffer));
 JSValue args[]={buffer,JS_UNDEFINED,JS_UNDEFINED};JSValue array=JS_NewTypedArray(ctx,1,args,big?JS_TYPED_ARRAY_BIG_INT64:JS_TYPED_ARRAY_INT32);assert(!JS_IsException(array));JS_FreeValue(ctx,buffer);*rt_out=rt;*ctx_out=ctx;return array;
}
static void *buffer_wait_thread(void *opaque) {
 BufferWaitThread *state=opaque;JSRuntime *rt;JSContext *ctx;JSValue array=buffer_thread_ctx(state->memory,&rt,&ctx,state->big);
 JSValue value=state->big?JS_NewBigInt64(ctx,0):JS_NewInt32(ctx,0);JSValue args[]={array,JS_NewInt32(ctx,state->index),value,JS_NewInt32(ctx,30000)};
 JSValue result=js_atomics_wait(ctx,JS_UNDEFINED,4,args);assert(!JS_IsException(result));const char *text=JS_ToCStringLen2(ctx,&state->length,result,0);assert(text&&state->length<16);memcpy(state->result,text,state->length);JS_FreeCString(ctx,text);JS_FreeValue(ctx,result);JS_FreeValue(ctx,value);JS_FreeValue(ctx,array);JS_FreeContext(ctx);JS_FreeRuntime(rt);return NULL;
}
static void buffers_threads_fixture(void) {
 uint8_t *memory=calloc(1,16);assert(memory);pthread_t threads[3];BufferWaitThread states[3]={{.memory=memory,.index=0},{.memory=memory,.index=0,.big=1},{.memory=memory,.index=1}};
 for(int i=0;i<3;i++) {assert(!pthread_create(&threads[i],NULL,buffer_wait_thread,&states[i]));
  struct timespec start,now;clock_gettime(CLOCK_MONOTONIC,&start);
  for(;;){pthread_mutex_lock(&js_atomics_mutex);int count=0;struct list_head *el;list_for_each(el,&js_atomics_waiter_list)count++;pthread_mutex_unlock(&js_atomics_mutex);if(count==i+1)break;clock_gettime(CLOCK_MONOTONIC,&now);assert(now.tv_sec-start.tv_sec<10);sched_yield();}
 }
 JSRuntime *rt;JSContext *ctx;JSValue array=buffer_thread_ctx(memory,&rt,&ctx,0);int indexes[]={0,0,1,0};
 for(int i=0;i<4;i++){JSValue args[]={array,JS_NewInt32(ctx,indexes[i]),JS_NewInt32(ctx,1)};JSValue result=js_atomics_notify(ctx,JS_UNDEFINED,3,args);assert(!JS_IsException(result));u32(JS_VALUE_GET_INT(result));JS_FreeValue(ctx,result);}
 for(int i=0;i<3;i++){assert(!pthread_join(threads[i],NULL));assert(states[i].length==2&&!memcmp(states[i].result,"ok",2));u32(states[i].length);fwrite(states[i].result,1,states[i].length,stdout);}
 pthread_mutex_lock(&js_atomics_mutex);assert(list_empty(&js_atomics_waiter_list));pthread_mutex_unlock(&js_atomics_mutex);JS_FreeValue(ctx,array);JS_FreeContext(ctx);JS_FreeRuntime(rt);free(memory);
}
