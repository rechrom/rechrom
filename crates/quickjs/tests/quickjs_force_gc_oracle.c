/* Public embedding API probe, linked to unchanged official C QuickJS. */
#include <assert.h>
#include <stdint.h>
#include <stdio.h>
#include "quickjs.c"
static void finalized(JSRuntime *rt, JSValue value) {
    uint32_t *count = JS_GetRuntimeOpaque(rt);
    ++*count;
}
int main(void) {
    JSRuntime *rt = JS_NewRuntime(); assert(rt);
    JSContext *ctx = JS_NewContext(rt); assert(ctx);
    uint32_t count = 0; JS_SetRuntimeOpaque(rt, &count);
    JSClassID id = 0; JS_NewClassID(&id);
    JSClassDef def = { .class_name = "RootProbe", .finalizer = finalized };
    assert(JS_NewClass(rt, id, &def) == 0);
    JS_SetGCThreshold(rt, SIZE_MAX);
    JSValue roots[16];
    for (int i = 0; i < 16; ++i) {
        roots[i] = JS_NewObjectClass(ctx, id);
        assert(!JS_IsException(roots[i]));
        assert(JS_SetPropertyStr(ctx, roots[i], "self", JS_DupValue(ctx, roots[i])) >= 0);
        assert(count == 0); /* every externally retained cycle stays live */
    }
    for (int i = 0; i < 8; ++i) JS_FreeValue(ctx, roots[i]);
    JSValue trigger = JS_NewObject(ctx); assert(!JS_IsException(trigger));
    printf("%u\n", count);
    for (int i = 8; i < 16; ++i) JS_FreeValue(ctx, roots[i]);
    JS_FreeValue(ctx, trigger); JS_RunGC(rt); printf("%u\n", count);
    JS_FreeContext(ctx); JS_FreeRuntime(rt); printf("%u\n", count);
    return 0;
}
