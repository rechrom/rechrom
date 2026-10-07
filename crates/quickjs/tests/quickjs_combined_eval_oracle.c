/* Full unchanged official C parser/compiler/VM with the same full JS_NewContext bootstrap. */
#include "quickjs.c"
#undef free
static void u32(uint32_t x) {
    for (int i = 0; i < 4; i++) putchar((x >> (i * 8)) & 255);
}
int main(int argc, char **argv) {
    assert(argc == 2);
    FILE *f = fopen(argv[1], "rb");
    assert(f);
    JSRuntime *rt = JS_NewRuntime();
    assert(rt);
    JS_SetMaxStackSize(rt, 4 * 1024 * 1024);
    JSContext *ctx = JS_NewContext(rt);
    assert(ctx);
    char *line = NULL;
    size_t capacity = 0;
    ssize_t n;
    uint32_t id = 0;
    while ((n = getline(&line, &capacity, f)) >= 0) {
        if (n && line[n - 1] == '\n') line[--n] = 0;
        JSValue result = JS_Eval(ctx, line, n, "combined.js", 0);
        u32(id++);
        u32(JS_VALUE_GET_NORM_TAG(result));
        JSValue value = JS_IsException(result) ? JS_GetException(ctx) : result;
        size_t len = 0;
        const char *encoded = JS_ToCStringLen2(ctx, &len, value, 0);
        assert(encoded);
        u32(len);
        fwrite(encoded, 1, len, stdout);
        JS_FreeCString(ctx, encoded);
        JS_FreeValue(ctx, value);
    }
    free(line);
    fclose(f);
    JS_FreeContext(ctx);
    JS_FreeRuntime(rt);
    return 0;
}
