/* Dependency-isolated source oracle: the translated C bodies are unchanged.
   Production errors, non-string conversion and nonempty weak lists are excluded. */
struct boundary_log { uint32_t oom, internal, type_error; };
JSValue JS_ThrowOutOfMemory(JSContext *ctx) { ((struct boundary_log *)ctx->user_opaque)->oom++; return JS_EXCEPTION; }
JSValue JS_ThrowInternalError(JSContext *ctx, const char *message, ...) { assert(!strcmp(message,"string too long")); ((struct boundary_log *)ctx->user_opaque)->internal++; return JS_EXCEPTION; }
JSValue JS_ThrowTypeError(JSContext *ctx, const char *message, ...) { assert(!strcmp(message,"invalid object type")); ((struct boundary_log *)ctx->user_opaque)->type_error++; return JS_EXCEPTION; }
JSValue JS_ToString(JSContext *ctx, JSValueConst val) { assert(JS_VALUE_GET_TAG(val)==JS_TAG_STRING); return JS_DupValue(ctx,val); }
static JSValue JS_ToStringFree(JSContext *ctx, JSValue val) { assert(JS_VALUE_GET_TAG(val)==JS_TAG_STRING); return val; }
/* With no weak records, original TRUE collection reaches these unchanged
   cycle-GC bodies without weak cleanup or finalization jobs. */
void JS_RunGC(JSRuntime *rt) {
    assert(list_empty(&rt->weakref_list));
    gc_decref(rt);
    gc_scan(rt);
    gc_free_cycles(rt);
}
