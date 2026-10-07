#include "quickjs.c"
static void dump(JSContext *ctx, JSBigInt *r) {
    assert(r); fwrite(&r->len, sizeof(r->len), 1, stdout);
    fwrite(r->tab, sizeof(r->tab[0]), r->len, stdout); js_free(ctx, r);
}
int main(void) {
    JSRuntime rt = {0}; JSContext ctx = {0};
    js_malloc_init(&rt.malloc_ctx); rt.malloc_ctx.mf=def_malloc_funcs;
    rt.malloc_ctx.malloc_state.malloc_limit=SIZE_MAX; rt.malloc_gc_threshold=SIZE_MAX;
    rt.current_exception=JS_UNINITIALIZED; ctx.rt=&rt;
    uint64_t seed=0x78d347323219237bULL;
    for (uint32_t i=0;i<4096;i++) {
        seed^=seed<<13;seed^=seed>>7;seed^=seed<<17;
        const int64_t edges[]={0,1,-1,INT64_MIN,INT64_MAX,-2,2,-7};
        int64_t x=i<8?edges[i]:(int64_t)seed;
        seed=seed*6364136223846793005ULL+1442695040888963407ULL;
        int64_t y=seed==0?1:(int64_t)seed;
        JSBigInt *a=js_bigint_new_si64(&ctx,x),*b=js_bigint_new_si64(&ctx,y);
        const int ops[]={OP_and,OP_xor,OP_or};for(int k=0;k<3;k++)dump(&ctx,js_bigint_logic(&ctx,a,b,ops[k]));
        dump(&ctx,js_bigint_mul(&ctx,a,b));
        for(int rem=0;rem<2;rem++)dump(&ctx,js_bigint_divrem(&ctx,a,b,rem));
        dump(&ctx,js_bigint_not(&ctx,a));
        const uint32_t shifts[]={0,1,31,32,63,64,65,127,128,129,1024};
        for(int k=0;k<11;k++){dump(&ctx,js_bigint_shl(&ctx,a,shifts[k]));dump(&ctx,js_bigint_shr(&ctx,a,shifts[k]));}
        dump(&ctx,js_bigint_new_di(&ctx,((js_sdlimb_t)(uint64_t)x<<64)|(uint64_t)y));
        const int wide_shifts[]={65,127,191};
        for(int k=0;k<3;k++) {
            JSBigInt *aa=js_bigint_shl(&ctx,a,wide_shifts[k]),*bb=js_bigint_shl(&ctx,b,33);
            for(int rem=0;rem<2;rem++)dump(&ctx,js_bigint_divrem(&ctx,aa,bb,rem));
            dump(&ctx,js_bigint_mul(&ctx,aa,bb));js_free(&ctx,aa);js_free(&ctx,bb);
        }
        js_free(&ctx,a);js_free(&ctx,b);
        JSBigInt *base=js_bigint_new_si64(&ctx,(int64_t)(i%31)-15);
        const int exponents[]={0,1,2,3,7,16,31};
        for(int k=0;k<7;k++){JSBigInt *exp=js_bigint_new_si64(&ctx,exponents[k]);dump(&ctx,js_bigint_pow(&ctx,base,exp));js_free(&ctx,exp);}
        js_free(&ctx,base);
    }
    assert(rt.malloc_ctx.malloc_state.malloc_count==0);
    return 0;
}
