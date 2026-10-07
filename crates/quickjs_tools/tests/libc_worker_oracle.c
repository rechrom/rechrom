/* Include unchanged official host implementation to exercise its static seams. */
#include "quickjs-libc.c"
static JSValue gate_eval(JSContext *ctx,const char *code){JSValue v=JS_Eval(ctx,code,strlen(code),"worker-gate.js",JS_EVAL_TYPE_GLOBAL);if(JS_IsException(v)){js_std_dump_error(ctx);exit(2);}return v;}
static void gate_class(JSContext *ctx){JS_NewClassID(&js_worker_class_id);JS_NewClass(JS_GetRuntime(ctx),js_worker_class_id,&js_worker_class);JSValue proto=JS_NewObject(ctx);JS_SetPropertyFunctionList(ctx,proto,js_worker_proto_funcs,countof(js_worker_proto_funcs));JS_SetClassProto(ctx,js_worker_class_id,proto);}
static JSContext *gate_context(JSRuntime **prt){*prt=JS_NewRuntime();js_std_init_handlers(*prt);JSContext *ctx=JS_NewContext(*prt);gate_class(ctx);return ctx;}
static void gate_free(JSRuntime *rt,JSContext *ctx){JS_FreeContext(ctx);js_std_free_handlers(rt);JS_FreeRuntime(rt);}
static void gate_print(JSContext *ctx,JSValue v){const char *s=JS_ToCString(ctx,v);puts(s);JS_FreeCString(ctx,s);JS_FreeValue(ctx,v);}
int main(void){
 void *p=js_sab_alloc(NULL,64);*(uint64_t*)p=0x0123456789abcdef;for(int i=0;i<10000;i++){js_sab_dup(NULL,p);js_sab_free(NULL,p);}JSSABHeader *h=(void*)((uint8_t*)p-sizeof(JSSABHeader));printf("sab=%d,%d,%016llx\n",((uintptr_t)p%8)==0,h->ref_count,(unsigned long long)*(uint64_t*)p);js_sab_free(NULL,p);
 JSRuntime *ra,*rb;JSContext *a=gate_context(&ra),*b=gate_context(&rb);JSWorkerMessagePipe *ab=js_new_message_pipe(),*ba=js_new_message_pipe();JSValue sender=js_worker_ctor_internal(a,JS_UNDEFINED,ba,ab),receiver=js_worker_ctor_internal(b,JS_UNDEFINED,ab,ba);
 JSValue global=JS_GetGlobalObject(b);JS_SetPropertyStr(b,global,"receiver",JS_DupValue(b,receiver));JS_FreeValue(b,global);
 JSValue f=gate_eval(b,"var received=[];ev=>{received.push(ev.data.i);if(ev.data.self!==ev.data)throw 'cycle';if(ev.data.i===2)receiver.onmessage=null}");JS_FreeValue(b,js_worker_set_onmessage(b,receiver,f));JS_FreeValue(b,f);
 for(int i=0;i<3;i++){char code[128];snprintf(code,sizeof(code),"(()=>{let v={i:%d};v.self=v;return v})()",i);JSValue v=gate_eval(a,code);JS_FreeValue(a,js_worker_postMessage(a,sender,1,&v));JS_FreeValue(a,v);}
 JSWorkerData *w=JS_GetOpaque(receiver,js_worker_class_id);struct pollfd fd={.fd=ab->waker.read_fd,.events=POLLIN};printf("wake-before=%d\n",poll(&fd,1,0));for(int i=0;i<3;i++)if(handle_posted_message(rb,b,w->msg_handler)!=1)exit(3);fd.revents=0;printf("wake-after=%d,removed=%d\n",poll(&fd,1,0),w->msg_handler==NULL);gate_print(b,gate_eval(b,"JSON.stringify(received)"));
 JS_FreeValue(a,sender);JS_FreeValue(b,receiver);js_free_message_pipe(ab);js_free_message_pipe(ba);gate_free(ra,a);gate_free(rb,b);
 a=gate_context(&ra);b=gate_context(&rb);ab=js_new_message_pipe();ba=js_new_message_pipe();sender=js_worker_ctor_internal(a,JS_UNDEFINED,ba,ab);receiver=js_worker_ctor_internal(b,JS_UNDEFINED,ab,ba);
 f=gate_eval(b,"ev=>{let v=ev.data;if(v.self!==v||v.a.buffer!==v.b.buffer)throw 'identity';v.a[0]+=5;globalThis.received=JSON.stringify([v.a[0],v.b[0],v.a.length])}");JS_FreeValue(b,js_worker_set_onmessage(b,receiver,f));JS_FreeValue(b,f);
 JSValue v=gate_eval(a,"(()=>{let s=new SharedArrayBuffer(16),a=new Int32Array(s);a[0]=37;let v={a,b:new Int32Array(s)};v.self=v;return v})()");JS_FreeValue(a,js_worker_postMessage(a,sender,1,&v));JS_FreeValue(a,v);JS_FreeValue(a,sender);gate_free(ra,a);w=JS_GetOpaque(receiver,js_worker_class_id);if(handle_posted_message(rb,b,w->msg_handler)!=1)exit(4);gate_print(b,gate_eval(b,"received"));printf("empty=%d\n",handle_posted_message(rb,b,w->msg_handler));JS_FreeValue(b,receiver);js_free_message_pipe(ab);js_free_message_pipe(ba);gate_free(rb,b);return 0;
}
