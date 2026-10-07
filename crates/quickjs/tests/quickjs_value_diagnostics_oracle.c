#include "quickjs.c"
#include "quickjs_diagnostic_allocator.h"
int main(void){JSRuntime*rt;
#ifdef DIAGNOSTICS_ALLOCATOR
rt=JS_NewRuntime2(&DIAGNOSTIC_MALLOC,NULL);
#else
rt=JS_NewRuntime();
#endif
assert(rt);JSContext*ctx=JS_NewContextRaw(rt);assert(ctx);assert(!JS_AddIntrinsicEval(ctx));assert(!JS_AddIntrinsicJSON(ctx));assert(!JS_AddIntrinsicPromise(ctx));JSModuleDef*m0=JS_NewCModule(ctx,"missing",NULL);assert(m0);assert(!JS_AddModuleExport(ctx,m0,"x"));for(int c=-3;c<260;c++)JS_DumpChar(stdout,c,34);const char*text[]={"","a\"b\\c\n","é"};for(int i=0;i<3;i++){JSValue v=JS_NewString(ctx,text[i]);JS_DumpString(rt,JS_VALUE_GET_STRING(v));JSValue v2=JS_DupValue(ctx,v);JS_DumpString(rt,JS_VALUE_GET_STRING(v));JS_FreeValue(ctx,v2);JS_FreeValue(ctx,v);}JSValue x=JS_Eval(ctx,"123456789012345678901234567890n",31,"dump.js",0);if(JS_IsException(x)){JS_FreeValue(ctx,JS_GetException(ctx));x=JS_NewBigInt64(ctx,9223372036854775807LL);}if(JS_VALUE_GET_TAG(x)==JS_TAG_BIG_INT)js_bigint_dump(ctx,"big",JS_VALUE_GET_PTR(x));JS_FreeValue(ctx,x);JS_DumpObjectHeader(rt);JS_DumpObject(rt,JS_VALUE_GET_OBJ(ctx->global_obj));JS_DumpGCObject(rt,&ctx->header);JS_DumpShapes(rt);JS_DumpAtoms(rt);js_malloc_dump_arenas(&rt->malloc_ctx);
for(int i=0;i<3;i++){JSMemoryUsage m={0};if(i>0){int64_t*p=(int64_t*)&m;for(size_t j=0;j<sizeof(m)/8;j++)p[j]=i==1?(j+1)*3:j+1;}JS_DumpMemoryUsage(stdout,&m,NULL);}
#ifdef DIAGNOSTICS_MEMORY_RT
JSMemoryUsage m={0};JS_ComputeMemoryUsage(rt,&m);JS_DumpMemoryUsage(stdout,&m,rt);
#endif
JS_FreeContext(ctx);JS_FreeRuntime(rt);return 0;}
