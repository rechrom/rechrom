/* Unchanged official QuickJS value printer oracle. */
#include "quickjs.c"
#undef free
static void u32(uint32_t n){for(int i=0;i<4;i++)putchar((n>>(i*8))&255);}
static void capture(void*opaque,const char*buf,size_t len){dbuf_put(opaque,(const uint8_t*)buf,len);}
int main(int argc,char**argv){assert(argc==2);JSRuntime*rt=JS_NewRuntime();assert(rt);JSContext*ctx=JS_NewContextRaw(rt);assert(ctx);assert(!JS_AddIntrinsicTypedArrays(ctx));assert(!JS_AddIntrinsicEval(ctx));assert(!JS_AddIntrinsicJSON(ctx));assert(!JS_AddIntrinsicRegExp(ctx));assert(!JS_AddIntrinsicMapSet(ctx));assert(!JS_AddIntrinsicDate(ctx));
char path[2048];snprintf(path,sizeof(path),"%s/quickjs_value_print_cases.txt",argv[1]);FILE*f=fopen(path,"rb");assert(f);char*line=NULL;size_t cap=0;ssize_t n;DynBuf b;dbuf_init(&b);int depths[]={0,1,2,8},strings[]={0,1,8,1000},items[]={0,1,3,100};
while((n=getline(&line,&cap,f))>=0){if(n>0&&line[n-1]=='\n')line[--n]=0;JSValue v=JS_Eval(ctx,line,n,"print.js",0);if(!strcmp(line,"Symbol(\"é\")")){JS_FreeValue(ctx,v);JSValue arg=JS_NewString(ctx,"é");v=js_symbol_constructor(ctx,JS_UNDEFINED,1,&arg);JS_FreeValue(ctx,arg);}if(!strcmp(line,"Symbol.iterator")){JS_FreeValue(ctx,v);v=JS_AtomToValue(ctx,JS_ATOM_Symbol_iterator);}if(JS_IsException(v))v=JS_GetException(ctx);
for(int d=0;d<4;d++)for(int s=0;s<4;s++)for(int i=0;i<4;i++)for(int h=0;h<2;h++)for(int raw=0;raw<2;raw++){if(raw&&JS_IsObject(v))continue;JSPrintValueOptions options={.max_depth=depths[d],.max_string_length=strings[s],.max_item_count=items[i],.show_hidden=h,.raw_dump=raw};b.size=0;JS_PrintValue(ctx,capture,&b,v,&options);u32(b.size);fwrite(b.buf,1,b.size,stdout);b.size=0;JS_PrintValueRT(rt,capture,&b,v,&options);u32(b.size);fwrite(b.buf,1,b.size,stdout);}JS_FreeValue(ctx,v);
}dbuf_free(&b);fclose(f);free(line);JS_FreeContext(ctx);JS_FreeRuntime(rt);return 0;}
