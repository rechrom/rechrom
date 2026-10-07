/* Complete unchanged official C JSON/binary/parser/compiler/VM reference. */
#include "quickjs.c"
#undef free
#undef malloc
typedef struct {void*ptr;size_t size,refs;} SABEntry;typedef struct{SABEntry entries[256];int count;} SABHost;
static void*sab_alloc(void*opaque,size_t size){SABHost*h=opaque;void*p=malloc(size?size:1);if(p){assert(h->count<256);h->entries[h->count++]=(SABEntry){p,size,1};}return p;}
static void sab_dup(void*opaque,void*ptr){SABHost*h=opaque;for(int i=0;i<h->count;i++)if(h->entries[i].ptr==ptr){h->entries[i].refs++;return;}abort();}
static void sab_free(void*opaque,void*ptr){SABHost*h=opaque;for(int i=0;i<h->count;i++)if(h->entries[i].ptr==ptr){if(--h->entries[i].refs==0){free(ptr);h->entries[i]=h->entries[--h->count];}return;}abort();}
static void emit_bytes(const uint8_t*data,size_t len,uint8_t**sab_tab,size_t sab_len){uint8_t*copy=malloc(len?len:1);memcpy(copy,data,len);for(size_t i=0;i<sab_len;i++){uint64_t pointer=(uintptr_t)sab_tab[i],canonical=UINT64_C(0x7f00000000000000)+i;for(size_t j=0;j+8<=len;j++)if(!memcmp(copy+j,&pointer,8))memcpy(copy+j,&canonical,8);}fwrite(copy,1,len,stdout);free(copy);}
static void u32(uint32_t n){for(int i=0;i<4;i++)putchar((n>>(i*8))&255);}
static void value(JSContext*ctx,JSValue val){u32(JS_VALUE_GET_NORM_TAG(val));int error=JS_IsException(val);JSValue original=error?JS_GetException(ctx):JS_DupValue(ctx,val);val=error?JS_GetPropertyStr(ctx,original,"message"):JS_IsObject(original)?JS_JSONStringify(ctx,original,JS_UNDEFINED,JS_UNDEFINED):JS_DupValue(ctx,original);JS_FreeValue(ctx,original);size_t len=0;const char*c=JS_ToCStringLen2(ctx,&len,val,0);assert(c);u32(len);fwrite(c,1,len,stdout);JS_FreeCString(ctx,c);JS_FreeValue(ctx,val);}
static int module_init(JSContext*ctx,JSModuleDef*m){return JS_SetModuleExport(ctx,m,"x",JS_NewInt32(ctx,42));}
static JSContext*context(JSRuntime*rt){JSContext*ctx=JS_NewContextRaw(rt);assert(ctx);
#ifdef SERIALIZATION_BUFFERS
assert(!JS_AddIntrinsicTypedArrays(ctx));
#endif
assert(!JS_AddIntrinsicEval(ctx));assert(!JS_AddIntrinsicJSON(ctx));assert(!JS_AddIntrinsicPromise(ctx));JSModuleDef*m=JS_NewCModule(ctx,"missing",module_init);assert(m);assert(!JS_AddModuleExport(ctx,m,"x"));return ctx;}
static FILE*cases(const char*dir,const char*name){char path[2048];snprintf(path,sizeof(path),"%s/%s",dir,name);FILE*f=fopen(path,"rb");assert(f);return f;}
static ssize_t next(FILE*f,char**line,size_t*cap){ssize_t n=getline(line,cap,f);if(n>0&&(*line)[n-1]=='\n')(*line)[--n]=0;return n;}
int main(int argc,char**argv){assert(argc==2);JSRuntime*rt=JS_NewRuntime();assert(rt);SABHost sab_host={0};JSSharedArrayBufferFunctions sab_functions={sab_alloc,sab_free,sab_dup,&sab_host};JS_SetSharedArrayBufferFunctions(rt,&sab_functions);JSContext*ctx=context(rt);char*line=NULL;size_t cap=0;ssize_t n;FILE*f=cases(argv[1],"quickjs_json_cases.txt");
while((n=next(f,&line,&cap))>=0){JSValue v=JS_Eval(ctx,line,n,"serialization.js",0);value(ctx,v);JS_FreeValue(ctx,v);}fclose(f);
f=cases(argv[1],"quickjs_json_direct_cases.txt");while((n=next(f,&line,&cap))>=0){for(int flags=0;flags<2;flags++){JSValue p=JS_ParseJSON2(ctx,line,n,"json-input",flags);JSValue v=JS_IsException(p)?p:JS_JSONStringify(ctx,p,JS_UNDEFINED,JS_UNDEFINED);value(ctx,v);JS_FreeValue(ctx,v);JS_FreeValue(ctx,p);}}fclose(f);JS_FreeContext(ctx);
const char*binary_files[]={"quickjs_serialization_cases.txt",
#ifdef SERIALIZATION_BUFFERS
"quickjs_serialization_buffer_cases.txt",
#endif
};
for(int file=0;file<countof(binary_files);file++){f=cases(argv[1],binary_files[file]);int flags_list[]={0,1,2,3,4,5,6,7,8,9,10,11,12,13,14,15};
while((n=next(f,&line,&cap))>=0){ctx=context(rt);JSValue v=JS_Eval(ctx,line,n,"serialization.js",0);
if(JS_IsException(v)){u32(UINT32_MAX);value(ctx,v);JS_FreeContext(ctx);continue;}
for(int i=0;i<countof(flags_list);i++){int flags=flags_list[i];u32(flags);size_t len=0,sab_len=0;uint8_t**sab_tab=NULL;uint8_t*data=JS_WriteObject2(ctx,&len,v,flags,&sab_tab,&sab_len);u32(data!=NULL);if(!data){value(ctx,JS_EXCEPTION);continue;}u32(len);emit_bytes(data,len,sab_tab,sab_len);
if(flags==0||flags==8){for(size_t truncated=0;truncated<len;truncated++){JSValue b=JS_ReadObject(ctx,data,truncated,flags&8);JSValue encoded=JS_IsException(b)?b:JS_JSONStringify(ctx,b,JS_UNDEFINED,JS_UNDEFINED);value(ctx,encoded);JS_FreeValue(ctx,encoded);JS_FreeValue(ctx,b);}}
for(int rom=0;rom<2;rom++){JSValue b=JS_ReadObject(ctx,data,len,(flags&13)|rom*2);JSValue encoded=JS_IsException(b)?b:JS_JSONStringify(ctx,b,JS_UNDEFINED,JS_UNDEFINED);value(ctx,encoded);JS_FreeValue(ctx,encoded);JS_FreeValue(ctx,b);}js_free(ctx,sab_tab);js_free(ctx,data);
}JS_FreeValue(ctx,v);JS_FreeContext(ctx);}fclose(f);}
const char*compile_files[]={"quickjs_serialization_compile_cases.txt","quickjs_serialization_module_cases.txt"};for(int file=0;file<2;file++){f=cases(argv[1],compile_files[file]);while((n=next(f,&line,&cap))>=0){ctx=context(rt);uint8_t*retained[2]={0};JSValue v=JS_Eval(ctx,line,n,"serialization.js",JS_EVAL_FLAG_COMPILE_ONLY|(file?JS_EVAL_TYPE_MODULE:0));
for(int rom=0;rom<2;rom++){size_t len=0;uint8_t*data=JS_WriteObject(ctx,&len,v,JS_WRITE_OBJ_BYTECODE);u32(data!=NULL);if(data){u32(len);fwrite(data,1,len,stdout);JSValue b=JS_ReadObject(ctx,data,len,JS_READ_OBJ_BYTECODE|rom*JS_READ_OBJ_ROM_DATA);JSValue r;if(JS_IsException(b))r=b;else if(file&&JS_ResolveModule(ctx,b)<0){JS_FreeValue(ctx,b);r=JS_EXCEPTION;}else r=JS_EvalFunction(ctx,b);value(ctx,r);JS_FreeValue(ctx,r);retained[rom]=data;}else value(ctx,JS_EXCEPTION);}JS_FreeValue(ctx,v);JS_FreeContext(ctx);for(int j=0;j<2;j++)js_free_rt(rt,retained[j]);
}fclose(f);}free(line);JS_FreeRuntime(rt);assert(sab_host.count==0);return 0;}
