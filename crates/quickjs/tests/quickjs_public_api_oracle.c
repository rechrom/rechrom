#include "quickjs.c"
static void number(uint64_t value,int length){for(int i=0;i<length;i++)putchar(value>>(i*8)&255);}
static JSValue api_magic(JSContext *ctx,JSValueConst self,int argc,JSValueConst *argv,int magic){return JS_NewInt32(ctx,magic+argc);}
static JSValue api_native(JSContext *ctx,JSValueConst self,int argc,JSValueConst *argv){return JS_NewInt32(ctx,20+argc);}
static const JSCFunctionListEntry exports[]={JS_PROP_INT32_DEF("number",7,JS_PROP_C_W_E),JS_PROP_STRING_DEF("text","ok",JS_PROP_C_W_E),JS_CFUNC_DEF("native",1,api_native)};
static int api_init(JSContext *ctx,JSModuleDef *module){return JS_SetModuleExportList(ctx,module,exports,3);}
int main(int argc,char **argv){
 JSRuntime *rt=JS_NewRuntime();assert(rt);JSContext *ctx=JS_NewContext(rt);assert(ctx);
 JSValue fun=JS_NewCFunctionMagic(ctx,api_magic,"magic",1,JS_CFUNC_generic_magic,17);assert(!JS_IsException(fun));
 JSValue args[]={JS_NewInt32(ctx,1)};JSValue value=JS_Call(ctx,fun,JS_UNDEFINED,1,args);int integer=0;assert(!JS_ToInt32(ctx,&integer,value));number(integer,4);JS_FreeValue(ctx,value);JS_FreeValue(ctx,fun);
 JSValue obj=JS_NewObject(ctx);JS_SetOpaque(obj,(void *)(uintptr_t)0x12345);JSClassID class_id=0;void *opaque=JS_GetAnyOpaque(obj,&class_id);number(class_id,4);number((uintptr_t)opaque,8);JS_FreeValue(ctx,obj);
 class_id=999;opaque=JS_GetAnyOpaque(JS_NULL,&class_id);assert(!opaque);number(class_id,4);
 JSModuleDef *module=JS_NewCModule(ctx,"native-module",api_init);assert(module);assert(!JS_AddModuleExportList(ctx,module,exports,3));
 const char *source="import {number,text,native} from 'native-module';globalThis.api_result=number+':'+text+':'+native(1)";
 value=JS_Eval(ctx,source,strlen(source),"api.js",JS_EVAL_TYPE_MODULE);assert(!JS_IsException(value));JS_FreeValue(ctx,value);
 JSContext *job_ctx;int status;while((status=JS_ExecutePendingJob(rt,&job_ctx))>0){}assert(status==0);
 JSValue global=JS_GetGlobalObject(ctx);value=JS_GetPropertyStr(ctx,global,"api_result");size_t len;const char *text=JS_ToCStringLen(ctx,&len,value);assert(text);fwrite(text,1,len,stdout);JS_FreeCString(ctx,text);JS_FreeValue(ctx,value);JS_FreeValue(ctx,global);
 JS_FreeContext(ctx);JS_FreeRuntime(rt);return 0;
}
