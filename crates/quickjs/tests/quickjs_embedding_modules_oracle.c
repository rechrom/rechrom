/* Unchanged official C default path, public API only. */
#include "quickjs.h"
#include <stdio.h>
#include <string.h>
static int rejected;
static void tracker(JSContext *ctx,JSValueConst promise,JSValueConst reason,JS_BOOL handled,void *opaque){if(!handled)rejected++;else rejected--;}
static JSModuleDef *loader(JSContext *ctx,const char *name,void *opaque){
 printf("load=%s\n",name);
 const char *code=!strcmp(name,"a")?"import 'leaf';order.push('a')":!strcmp(name,"b")?"order.push('b')":!strcmp(name,"leaf")?"order.push('leaf')":NULL;
 if(!code){JS_ThrowReferenceError(ctx,"missing");return NULL;}
 JSValue value=JS_Eval(ctx,code,strlen(code),name,JS_EVAL_TYPE_MODULE|JS_EVAL_FLAG_COMPILE_ONLY);
 if(JS_IsException(value))return NULL;
 JSModuleDef *module=JS_VALUE_GET_PTR(value);JS_FreeValue(ctx,value);return module;
}
static JSValue eval(JSContext *ctx,const char *code,int flags){JSValue value=JS_Eval(ctx,code,strlen(code),"root.js",flags);if(JS_IsException(value)){JSValue e=JS_GetException(ctx);const char *p=JS_ToCString(ctx,e);fprintf(stderr,"%s\n",p);JS_FreeCString(ctx,p);JS_FreeValue(ctx,e);}return value;}
static void drain(JSRuntime *rt){JSContext *ctx;while(JS_ExecutePendingJob(rt,&ctx)>0);}
int main(void){
 JSRuntime *rt=JS_NewRuntime();JSContext *ctx=JS_NewContext(rt);JS_SetModuleLoaderFunc(rt,NULL,loader,NULL);JS_SetHostPromiseRejectionTracker(rt,tracker,NULL);
 JS_FreeValue(ctx,eval(ctx,"var order=[]",JS_EVAL_TYPE_GLOBAL));
 JSValue module=eval(ctx,"import 'a';import 'b';order.push('root')",JS_EVAL_TYPE_MODULE|JS_EVAL_FLAG_COMPILE_ONLY);
 JS_FreeValue(ctx,JS_EvalFunction(ctx,module));
 JSValue value=eval(ctx,"order.join('|')",JS_EVAL_TYPE_GLOBAL);const char *p=JS_ToCString(ctx,value);printf("order=%s\n",p);JS_FreeCString(ctx,p);JS_FreeValue(ctx,value);
 JS_FreeValue(ctx,eval(ctx,"import('missing').catch(()=>{})",JS_EVAL_TYPE_GLOBAL));drain(rt);
 JS_FreeValue(ctx,eval(ctx,"Promise.reject('same');Promise.reject('same')",JS_EVAL_TYPE_GLOBAL));
 JS_FreeValue(ctx,eval(ctx,"throw Error('module')",JS_EVAL_TYPE_MODULE));drain(rt);
 printf("unhandled=%d\n",rejected);
 value=eval(ctx,"JSON.stringify([Reflect.ownKeys(Object.getPrototypeOf(Uint8Array.prototype)).map(String),Object.getOwnPropertyDescriptor(Object.getPrototypeOf(Uint8Array.prototype),'toString')])",JS_EVAL_TYPE_GLOBAL);p=JS_ToCString(ctx,value);printf("typed-proto=%s\n",p);JS_FreeCString(ctx,p);JS_FreeValue(ctx,value);
 JS_FreeContext(ctx);JS_FreeRuntime(rt);
}
