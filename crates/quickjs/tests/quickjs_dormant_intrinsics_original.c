/* Exact six default-disabled source function copies; official quickjs.c itself
   is included unchanged by the oracle. No other #if0 block is enabled. */

static JSValue js_number___toInteger(JSContext *ctx, JSValueConst this_val,
                                     int argc, JSValueConst *argv)
{
    return JS_ToIntegerFree(ctx, JS_DupValue(ctx, argv[0]));
}

static JSValue js_number___toLength(JSContext *ctx, JSValueConst this_val,
                                    int argc, JSValueConst *argv)
{
    int64_t v;
    if (JS_ToLengthFree(ctx, &v, JS_DupValue(ctx, argv[0])))
        return JS_EXCEPTION;
    return JS_NewInt64(ctx, v);
}

static JSValue js_string___isSpace(JSContext *ctx, JSValueConst this_val,
                                   int argc, JSValueConst *argv)
{
    int c;
    if (JS_ToInt32(ctx, &c, argv[0]))
        return JS_EXCEPTION;
    return JS_NewBool(ctx, lre_is_space(c));
}

static JSValue js___date_getTimezoneOffset(JSContext *ctx, JSValueConst this_val,
                                           int argc, JSValueConst *argv)
{
    double dd;

    if (JS_ToFloat64(ctx, &dd, argv[0]))
        return JS_EXCEPTION;
    if (isnan(dd))
        return __JS_NewFloat64(ctx, dd);
    else
        return JS_NewInt32(ctx, getTimezoneOffset((int64_t)dd));
}

static JSValue js_get_prototype_from_ctor(JSContext *ctx, JSValueConst ctor,
                                          JSValueConst def_proto)
{
    JSValue proto;
    proto = JS_GetProperty(ctx, ctor, JS_ATOM_prototype);
    if (JS_IsException(proto))
        return proto;
    if (!JS_IsObject(proto)) {
        JS_FreeValue(ctx, proto);
        proto = JS_DupValue(ctx, def_proto);
    }
    return proto;
}

static JSValue js___date_create(JSContext *ctx, JSValueConst this_val,
                                int argc, JSValueConst *argv)
{
    JSValue obj, proto;
    proto = js_get_prototype_from_ctor(ctx, argv[0], argv[1]);
    if (JS_IsException(proto))
        return proto;
    obj = JS_NewObjectProtoClass(ctx, proto, JS_CLASS_DATE);
    JS_FreeValue(ctx, proto);
    if (!JS_IsException(obj))
        JS_SetObjectData(ctx, obj, JS_DupValue(ctx, argv[2]));
    return obj;
}
