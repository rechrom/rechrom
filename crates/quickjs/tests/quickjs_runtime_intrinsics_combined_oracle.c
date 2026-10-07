/* Full unchanged official C standard context and execution; no host JS backend. */
#include "quickjs.c"
#undef free
static void u32(uint32_t x){for(int i=0;i<4;i++)putchar((x>>(i*8))&255);}
int main(int argc,char**argv){assert(argc==2);FILE*f=fopen(argv[1],"rb");assert(f);JSRuntime*rt=JS_NewRuntime();assert(rt);JSContext*ctx=JS_NewContext(rt);assert(ctx);char*line=NULL;size_t cap=0;ssize_t n;uint32_t id=0;
while((n=getline(&line,&cap,f))>=0){if(n&&line[n-1]=='\n')line[--n]=0;JSValue result=JS_Eval(ctx,line,n,"combined.js",0);u32(id++);u32(JS_VALUE_GET_NORM_TAG(result));JSValue value=JS_IsException(result)?JS_GetException(ctx):result;size_t len=0;const char*encoded=JS_ToCStringLen2(ctx,&len,value,0);assert(encoded);u32(len);fwrite(encoded,1,len,stdout);JS_FreeCString(ctx,encoded);JS_FreeValue(ctx,value);JSContext*job_ctx=NULL;int jobs=0,res;while((res=JS_ExecutePendingJob(rt,&job_ctx))!=0){assert(res>0);assert(++jobs<10000);}u32(jobs);}
free(line);fclose(f);JS_FreeContext(ctx);JS_FreeRuntime(rt);return 0;}
