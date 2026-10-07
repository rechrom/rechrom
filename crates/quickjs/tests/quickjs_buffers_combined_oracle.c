/* Official C parser/VM/buffer algorithms, identical raw-context bootstrap. */
#include "quickjs.c"
#undef free
#undef malloc
#undef realloc
#undef calloc
static void u32(uint32_t x) {for(int i=0;i<4;i++)putchar((x>>(i*8))&255);}
#ifdef BUFFERS_HOST_OOM_TEST
#include "quickjs_buffers_host_oom.inc"
#endif
#ifdef BUFFERS_THREAD_TEST
#include "quickjs_buffers_atomics_threads.inc"
#endif
int main(int argc,char **argv) {
 assert(argc==2);FILE *f=fopen(argv[1],"rb");assert(f);
 JSRuntime *rt=JS_NewRuntime();assert(rt);JSContext *ctx=JS_NewContextRaw(rt);assert(ctx);
 JSCFunctionType ft={.generic_magic=js_function_constructor};
 ctx->function_ctor=JS_NewCFunction2(ctx,ft.generic,"Function",1,JS_CFUNC_constructor_or_func_magic,JS_FUNC_NORMAL);
 assert(!JS_IsException(ctx->function_ctor));
 ctx->array_proto_values=JS_GetProperty(ctx,ctx->class_proto[JS_CLASS_ARRAY],JS_ATOM_values);
 ctx->class_proto[JS_CLASS_ARRAY_ITERATOR]=JS_NewObjectProtoList(ctx,ctx->class_proto[JS_CLASS_ITERATOR],js_array_iterator_proto_funcs,countof(js_array_iterator_proto_funcs));
 assert(!JS_IsException(ctx->class_proto[JS_CLASS_ARRAY_ITERATOR]));assert(!JS_AddIntrinsicEval(ctx));assert(!JS_AddIntrinsicTypedArrays(ctx));rt->can_block=1;
 char *line=NULL;size_t cap=0;ssize_t n;uint32_t id=0;
 while((n=getline(&line,&cap,f))>=0){if(n&&line[n-1]=='\n')line[--n]=0;
 JSValue result=JS_Eval(ctx,line,n,"buffers.js",0);u32(id++);u32(JS_VALUE_GET_NORM_TAG(result));
 JSValue v=JS_IsException(result)?JS_GetException(ctx):result;size_t len=0;const char *s=JS_ToCStringLen2(ctx,&len,v,0);assert(s);u32(len);fwrite(s,1,len,stdout);JS_FreeCString(ctx,s);JS_FreeValue(ctx,v);}
 free(line);fclose(f);JS_FreeContext(ctx);JS_FreeRuntime(rt);
#ifdef BUFFERS_HOST_OOM_TEST
 buffers_host_oom_fixture();
#endif
#ifdef BUFFERS_THREAD_TEST
 buffers_threads_fixture();
#endif
return 0;
}
