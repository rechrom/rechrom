#include "quickjs-libc.h"
#include <stdio.h>
int main(int argc,char **argv){if(argc!=2)return 2;JSRuntime *rt=JS_NewRuntime();js_std_init_handlers(rt);JSContext *ctx=JS_NewContext(rt);js_init_module_std(ctx,"std");size_t size;uint8_t *source=js_load_file(ctx,&size,argv[1]);if(!source)return 3;JSValue value=JS_Eval(ctx,(char*)source,size,argv[1],JS_EVAL_TYPE_MODULE);js_free(ctx,source);value=js_std_await(ctx,value);if(JS_IsException(value)){js_std_dump_error(ctx);return 4;}JS_FreeValue(ctx,value);js_std_loop(ctx);JS_FreeContext(ctx);js_std_free_handlers(rt);JS_FreeRuntime(rt);return 0;}
