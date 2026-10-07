#define main quickjs_test262_original_main
#include "run-test262.c"
#undef main
int main(int argc, char **argv) {
  if(argc!=2)return 2;
  ThreadLocalStorage tls; init_thread_local_storage(&tls);
  JSRuntime *rt=JS_NewRuntime(); JS_SetRuntimeOpaque(rt,&tls); JS_SetCanBlock(rt,TRUE);
  JSContext *ctx=JS_NewContext(rt); outfile=stdout; add_helpers(ctx);
  size_t len; uint8_t *buf=js_load_file(ctx,&len,argv[1]); if(!buf)return 3;
  JSValue val=JS_Eval(ctx,(char*)buf,len,argv[1],JS_EVAL_TYPE_GLOBAL); js_free(ctx,buf);
  int failed=JS_IsException(val); if(failed)js_std_dump_error(ctx); JS_FreeValue(ctx,val);
  js_agent_free(ctx); printf("async=%d\n",tls.async_done);
  JS_FreeContext(ctx); JS_FreeRuntime(rt); return failed?4:0;
}
