/* Unchanged official memory accounting: allocation-provider totals are tested separately. */
#include "quickjs.c"
#undef free
static void number(uint64_t value,int length) { for (int i=0;i<length;i++) putchar(value>>(i*8)&255); }
int main(int argc,char **argv) {
    assert(argc==2); FILE *file=fopen(argv[1],"rb"); assert(file);
    JSRuntime *rt=JS_NewRuntime(); assert(rt); JSContext *ctx=JS_NewContext(rt); assert(ctx);
    char *line=NULL;size_t capacity=0;ssize_t n;uint32_t id=0;
    while((n=getline(&line,&capacity,file))>=0) {
        if(n && line[n-1]=='\n') line[--n]=0;
        JSValue value=JS_Eval(ctx,line,n,"memory.js",0); assert(!JS_IsException(value));
        for(int phase=0;phase<2;phase++) {
            if(phase) JS_RunGC(rt);
            JSMemoryUsage usage; JS_ComputeMemoryUsage(rt,&usage);
            number(id,4);number(phase,4);
            number(usage.memory_used_size,8);
            number(usage.memory_used_count,8);
            number(usage.atom_count,8);
            number(usage.atom_size,8);
            number(usage.str_count,8);
            number(usage.str_size,8);
            number(usage.obj_count,8);
            number(usage.obj_size,8);
            number(usage.prop_count,8);
            number(usage.prop_size,8);
            number(usage.shape_count,8);
            number(usage.shape_size,8);
            number(usage.js_func_count,8);
            number(usage.js_func_size,8);
            number(usage.js_func_code_size,8);
            number(usage.js_func_pc2line_count,8);
            number(usage.js_func_pc2line_size,8);
            number(usage.c_func_count,8);
            number(usage.array_count,8);
            number(usage.fast_array_count,8);
            number(usage.fast_array_elements,8);
            number(usage.binary_object_count,8);
            number(usage.binary_object_size,8);
        }
        JS_FreeValue(ctx,value);id++;
    }
    free(line);fclose(file);JS_FreeContext(ctx);JS_FreeRuntime(rt);return 0;
}
