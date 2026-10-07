/* Read-only original host included for exact private layouts; no replacement C. */
#include "quickjs-libc.c"
#include <stddef.h>
#include "../../quickjs/tests/quickjs_diagnostic_allocator.h"
static size_t live;
static size_t fail_after=SIZE_MAX;
static int gate_fail(void){if(fail_after==SIZE_MAX)return 0;if(--fail_after==0){fail_after=SIZE_MAX;return 1;}return 0;}
static void *gate_alloc(JSMallocState *s,size_t n){if(gate_fail())return NULL;void*p=diagnostic_malloc(s,n);if(p)live++;return p;}
static void gate_free(JSMallocState*s,void*p){if(p)live--;diagnostic_free(s,p);}
static void *gate_realloc(JSMallocState*s,void*p,size_t n){if(!p)return gate_alloc(s,n);if(!n){gate_free(s,p);return NULL;}if(gate_fail())return NULL;return diagnostic_realloc(s,p,n);}
static JSMallocFunctions gate_allocator={gate_alloc,gate_free,gate_realloc,diagnostic_capacity};
#define L(T,F) printf("layout:%s:%zu:%zu:%zu\n",#T,sizeof(T),_Alignof(T),offsetof(T,F))
static void layouts(void){
 L(JSOSRWHandler,rw_func);L(JSOSSignalHandler,func);L(JSOSTimer,func);
 L(JSWorkerMessage,sab_tab_len);L(JSWaker,write_fd);L(JSWorkerMessagePipe,waker);
 L(JSWorkerMessageHandler,poll_fd_index);L(JSRejectedPromiseEntry,reason);L(JSThreadState,poll_fds_size);
}
static int oom_gates(void){
 const char *src="(()=>{let env={};for(let i=0;i<80;i++)env['host_key_'+i]={toString(){return 'value_'+i}};return [['/not-a-quickjs-file','hello'],{env,file:'/not-a-quickjs-file',cwd:'/tmp',usePath:false}]})()";
 for(int i=1;i<=150;i++){
  JSRuntime *rt=JS_NewRuntime2(&gate_allocator,NULL);js_std_init_handlers(rt);JSContext *ctx=JS_NewContext(rt);
  JSValue pair=JS_Eval(ctx,src,strlen(src),"exec-oom.js",JS_EVAL_TYPE_GLOBAL);if(JS_IsException(pair))return 7;
  JSValue args[2]={JS_GetPropertyUint32(ctx,pair,0),JS_GetPropertyUint32(ctx,pair,1)};
  fail_after=i;JSValue v=js_os_exec(ctx,JS_UNDEFINED,2,args);fail_after=SIZE_MAX;
  int exception=JS_IsException(v),status=-999;if(exception){JSValue e=JS_GetException(ctx);JS_FreeValue(ctx,e);}else JS_ToInt32(ctx,&status,v);
  JS_FreeValue(ctx,v);JS_FreeValue(ctx,args[0]);JS_FreeValue(ctx,args[1]);JS_FreeValue(ctx,pair);
  JS_FreeContext(ctx);js_std_free_handlers(rt);JS_FreeRuntime(rt);
  printf("oom:%d:%d:%d:%zu\n",i,exception,status,live);if(live)return 8;
 }
 return 0;
}
int main(int argc,char **argv){
 if(argc==2&&!strcmp(argv[1],"--exec-oom"))return oom_gates();
 if(argc!=2)return 2;layouts();
 JSRuntime *rt=JS_NewRuntime2(&gate_allocator,NULL);js_std_init_handlers(rt);JSContext *ctx=JS_NewContext(rt);
 js_init_module_std(ctx,"std");js_init_module_os(ctx,"os");
 JS_SetModuleLoaderFunc2(rt,NULL,js_module_loader,js_module_check_attributes,NULL);
 js_std_add_helpers(ctx,0,NULL);size_t len;uint8_t *source=js_load_file(ctx,&len,argv[1]);
 if(!source)return 3;JSValue value=JS_Eval(ctx,(char*)source,len,argv[1],JS_EVAL_TYPE_MODULE);js_free(ctx,source);
 value=js_std_await(ctx,value);if(JS_IsException(value)){js_std_dump_error(ctx);return 4;}JS_FreeValue(ctx,value);js_std_loop(ctx);
 struct termios tty;if(tcgetattr(0,&tty))return 5;
 printf("raw:%d:%d:%d:%d:%d:%d:%d\n",!!(tty.c_lflag&ECHO),!!(tty.c_lflag&ICANON),!!(tty.c_lflag&ISIG),!!(tty.c_oflag&OPOST),!!(tty.c_iflag&ICRNL),tty.c_cc[VMIN],tty.c_cc[VTIME]);
 JS_FreeContext(ctx);js_std_free_handlers(rt);JS_FreeRuntime(rt);printf("live:%zu\n",live);return live?6:0;
}
