/* Independent unmodified-C host-layer oracle, Bellard/Gordon MIT. */
#include "quickjs-libc.c"

static void show(JSContext *ctx, const char *name, JSValue value) {
    if (JS_IsException(value)) value = JS_GetException(ctx);
    const char *text = JS_ToCString(ctx, value);
    printf("%s:%s\n", name, text ? text : "<conversion exception>");
    JS_FreeCString(ctx, text);
    JS_FreeValue(ctx, value);
}
static JSValue inspect_json(JSContext *ctx, JSValueConst this_val,
                            int argc, JSValueConst *argv) {
    int n = js_module_test_json(ctx, argv[0]);
    if (JS_HasException(ctx)) return JS_EXCEPTION;
    return JS_NewInt32(ctx, n);
}
static JSValue inspect_attrs(JSContext *ctx, JSValueConst this_val,
                             int argc, JSValueConst *argv) {
    if (js_module_check_attributes(ctx, NULL, argv[0]) < 0) return JS_EXCEPTION;
    return JS_NewInt32(ctx, 0);
}
static JSValue inspect_option(JSContext *ctx, JSValueConst this_val,
                              int argc, JSValueConst *argv) {
    int value = 9;
    if (get_bool_option(ctx, &value, argv[0], "enabled") < 0) return JS_EXCEPTION;
    return JS_NewInt32(ctx, value);
}
static void evaluate(JSContext *ctx, const char *name, const char *source) {
    show(ctx, name, JS_Eval(ctx, source, strlen(source), "helpers.js", 0));
}
int main(int argc, char **argv) {
    JSRuntime *rt = JS_NewRuntime();
    JS_SetMaxStackSize(rt, 4 * 1024 * 1024);
    JSContext *ctx = JS_NewContext(rt);
    JSValue global = JS_GetGlobalObject(ctx);
    JS_SetPropertyStr(ctx, global, "jsonType", JS_NewCFunction(ctx, inspect_json, "jsonType", 1));
    JS_SetPropertyStr(ctx, global, "attrs", JS_NewCFunction(ctx, inspect_attrs, "attrs", 1));
    js_std_add_helpers(ctx, argc, argv);
    JS_SetPropertyStr(ctx, global, "getEnv", JS_NewCFunction(ctx, js_std_getenv, "getEnv", 1));
    JS_SetPropertyStr(ctx, global, "setEnv", JS_NewCFunction(ctx, js_std_setenv, "setEnv", 2));
    JS_SetPropertyStr(ctx, global, "unsetEnv", JS_NewCFunction(ctx, js_std_unsetenv, "unsetEnv", 1));
    JS_SetPropertyStr(ctx, global, "getEnviron", JS_NewCFunction(ctx, js_std_getenviron, "getEnviron", 0));
    JS_SetPropertyStr(ctx, global, "collect", JS_NewCFunction(ctx, js_std_gc, "collect", 0));
    JS_SetPropertyStr(ctx, global, "loadText", JS_NewCFunction(ctx, js_std_loadFile, "loadText", 1));
    JS_SetPropertyStr(ctx, global, "option", JS_NewCFunction(ctx, inspect_option, "option", 1));
    if (argc >= 4 && !strcmp(argv[2], "--exit")) {
        JS_SetPropertyStr(ctx, global, "stdExit", JS_NewCFunction(ctx, js_std_exit, "stdExit", 1));
        evaluate(ctx, "exit-case", argv[3]);
        return 2; /* Every selected case must exercise the real process exit. */
    }
    evaluate(ctx, "json-types", "[jsonType(undefined),jsonType({}),jsonType({type:'json'}),jsonType({type:'json5'}),jsonType({type:'JSON'}),jsonType({type:'json\\0'})].join(',')");
    evaluate(ctx, "attrs-values", "attrs({get type(){throw Error('must not read')}})");
    evaluate(ctx, "attrs-hidden", "attrs(Object.defineProperty({[Symbol()]:1},'other',{value:1}))");
    evaluate(ctx, "attrs-reject", "try{attrs({other:1})}catch(e){e.name+':'+e.message}");
    evaluate(ctx, "attrs-nul", "try{attrs({'other\\0':1})}catch(e){e.name+':'+e.message}");
    evaluate(ctx, "json-getter", "try{jsonType({get type(){throw Error('json getter')}})}catch(e){e.message}");
    evaluate(ctx, "attrs-proxy", "try{attrs(new Proxy({},{ownKeys(){throw Error('own keys')}}))}catch(e){e.message}");
    evaluate(ctx, "environment", "(()=>{let k='QJS_TRANSLATED_TOOLS_ORACLE_ENV';unsetEnv(k);let a=getEnv(k);setEnv(k,'héllo\\0ignored');let b=getEnv(k),c=getEnviron()[k];unsetEnv(k);return [a===undefined,b,c,getEnv(k)===undefined].join('|')})()");
    evaluate(ctx, "environment-coercion", "(()=>{let seen=[];setEnv({toString(){seen.push('key');return ''}},{toString(){seen.push('value');return 'x'}});return seen.join(',')})()");
    evaluate(ctx, "environment-error", "try{setEnv('QJS_ORACLE_TMP',{toString(){throw Error('value')}})}catch(e){e.message}");
    evaluate(ctx, "gc", "collect()===undefined");
    evaluate(ctx, "helpers", "[typeof print,typeof console.log,typeof __loadScript,typeof performance.now,performance.now()>0,scriptArgs.slice(1).length].join(',')");
    evaluate(ctx, "printer", "print('embedded\\0text',{answer:42},[1,2],undefined,42n);console.log('console',false);'done'");
    evaluate(ctx, "option", "[option({}),option({enabled:false}),option({enabled:'x'}),option({get enabled(){return null}})].join(',')");
    evaluate(ctx, "option-error", "try{option({get enabled(){throw Error('option getter')}})}catch(e){e.message}");
    evaluate(ctx, "load-text", "JSON.stringify([loadText(scriptArgs[1]),loadText('__quickjs_nonexistent_file__')])");
    size_t length = 999;
    uint8_t *buffer = js_load_file(ctx, &length, argv[1]);
    printf("file:%zu:%d:%d\n", length, buffer && buffer[length] == 0, buffer && !memcmp(buffer, "a\0b\n", 4));
    js_free(ctx, buffer);
    buffer = js_load_file(NULL, &length, argv[1]);
    printf("file-null:%zu:%d\n", length, buffer && buffer[length] == 0);
    free(buffer);
    buffer = js_load_file(ctx, &length, "__quickjs_nonexistent_file__");
    printf("file-missing:%d\n", buffer == NULL);
    const char *names[] = {"relative name.mjs", "scheme:module", argv[1], "__quickjs_nonexistent_file__"};
    for (int i = 0; i < 4; i++) {
        JSValue module = JS_Eval(ctx, "export {};", 10, names[i], JS_EVAL_TYPE_MODULE | JS_EVAL_FLAG_COMPILE_ONLY);
        int result = js_module_set_import_meta(ctx, module, i >= 2, i == 0);
        printf("meta-%d-result:%d\n", i, result);
        if (result < 0) show(ctx, "meta-error", JS_EXCEPTION);
        else {
            JSValue meta = JS_GetImportMeta(ctx, JS_VALUE_GET_PTR(module));
            JS_SetPropertyStr(ctx, global, "meta", meta);
            evaluate(ctx, "meta-value", "JSON.stringify([meta.url,meta.main,Object.keys(meta),Object.getOwnPropertyDescriptor(meta,'main')])");
        }
        JS_FreeValue(ctx, module);
    }
    JSValue json = JS_ParseJSON(ctx, "{\"answer\":42}", 13, "json:fixture");
    create_json_module(ctx, "json:fixture", json);
    const char *source = "import x from 'json:fixture'; globalThis.result=x.answer;";
    JSValue evaluated = JS_Eval(ctx, source, strlen(source), "json:consumer", JS_EVAL_TYPE_MODULE);
    if (JS_IsException(evaluated)) show(ctx, "json-module", evaluated);
    else { JS_FreeValue(ctx, evaluated); evaluate(ctx, "json-module", "result"); }
    const char *error = "throw Error('reported')";
    evaluated = JS_Eval(ctx, error, strlen(error), "error.js", 0);
    if (JS_IsException(evaluated)) js_std_dump_error(ctx);
    JS_FreeValue(ctx, evaluated);
    JS_FreeValue(ctx, global);
    JS_FreeContext(ctx);
    JS_FreeRuntime(rt);
    return 0;
}
