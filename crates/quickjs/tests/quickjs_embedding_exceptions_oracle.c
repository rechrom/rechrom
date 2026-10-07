/* Default official C: diagnostics must not change any JS-visible bytes. */
#include "quickjs.h"
#include <stdio.h>
#include <string.h>
int main(void) {
 JSRuntime *rt=JS_NewRuntime(); JSContext *ctx=JS_NewContext(rt);
 const char *sources[]={
 "var e=new Error('kept');JSON.stringify([e.stack,Object.getOwnPropertyDescriptor(e,'stack'),Object.keys(e)])",
 "try{try{throw 'first'}finally{try{throw 'cleanup'}catch(e){}}}catch(e){JSON.stringify(e)}",
 "var old=new Error('old');try{throw old}catch(e){JSON.stringify([e===old,e.stack])}",
 "var events=[];try{for(const x of {[Symbol.iterator](){return {next(){return {value:1,done:false}},return(){events.push('close');throw 'ignored'}}}}){throw 'original'}}catch(e){JSON.stringify([e,events])}",
 "var result=[];try{throw 'a'}catch(e){result.push(e)}try{throw 'b'}catch(e){result.push(e)}JSON.stringify(result)"
 };
 for(int i=0;i<5;i++) {JSValue v=JS_Eval(ctx,sources[i],strlen(sources[i]),"parity.js",JS_EVAL_TYPE_GLOBAL);if(JS_IsException(v)){JSValue e=JS_GetException(ctx);const char *s=JS_ToCString(ctx,e);fprintf(stderr,"%s\n",s);JS_FreeCString(ctx,s);JS_FreeValue(ctx,e);return 1;}const char *s=JS_ToCString(ctx,v);puts(s);JS_FreeCString(ctx,s);JS_FreeValue(ctx,v);}
 JS_FreeContext(ctx);JS_FreeRuntime(rt);return 0;
}
