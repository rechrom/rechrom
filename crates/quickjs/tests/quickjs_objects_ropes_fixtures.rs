unsafe fn objects_ropes_fixtures(out: &mut Vec<u8>, ctx: *mut JSContext, h: *mut Host) {
    for fail in 0..32 {
        let obj = JS_NewObject(ctx);
        let desc = JS_NewObject(ctx);
        let atom = crate::quickjs_atom::JS_ATOM_name;
        if fail != 0 {
            (*h).fail = (*h).calls + fail;
        }
        for step in 0..12 {
            JS_SetPropertyStr(ctx, desc, c"value".as_ptr(), JS_NewInt32(ctx, step));
            JS_SetPropertyStr(ctx, desc, c"enumerable".as_ptr(), JS_NewBool(ctx, step & 1));
            JS_SetPropertyStr(ctx, desc, c"configurable".as_ptr(), JS_TRUE);
            JS_SetPropertyStr(ctx, desc, c"writable".as_ptr(), JS_TRUE);
            num(
                out,
                JS_DefinePropertyDesc(ctx, obj, atom, desc, JS_PROP_THROW) as u64,
                4,
            );
            dump_exception(out, ctx);
            let mut args = [obj, JS_AtomToValue(ctx, atom), desc];
            let value = js_object_getOwnPropertyDescriptor(
                ctx,
                JS_UNDEFINED,
                2,
                args.as_mut_ptr(),
                step & 1,
            );
            dump_val(out, value);
            if JS_IsObject(value) != 0 {
                dump_object(out, JS_VALUE_GET_PTR(value).cast());
            }
            JS_FreeValue(ctx, value);
            JS_FreeValue(ctx, args[1]);
            dump_exception(out, ctx);
        }
        (*h).fail = 0;
        let excluded = JS_NewObject(ctx);
        JS_DefinePropertyValue(ctx, excluded, atom, JS_TRUE, JS_PROP_C_W_E);
        let target = JS_NewObject(ctx);
        num(
            out,
            JS_CopyDataProperties(ctx, target, obj, excluded, 0) as u64,
            4,
        );
        dump_exception(out, ctx);
        dump_object(out, JS_VALUE_GET_PTR(target).cast());
        num(
            out,
            JS_CopyDataProperties(ctx, target, obj, JS_UNDEFINED, 1) as u64,
            4,
        );
        dump_exception(out, ctx);
        dump_object(out, JS_VALUE_GET_PTR(target).cast());
        JS_FreeValue(ctx, excluded);
        JS_FreeValue(ctx, target);
        for kind in 0..3 {
            let v = JS_GetOwnPropertyNames2(ctx, obj, JS_GPN_STRING_MASK | JS_GPN_ENUM_ONLY, kind);
            dump_val(out, v);
            if JS_IsObject(v) != 0 {
                dump_object(out, JS_VALUE_GET_PTR(v).cast());
            }
            JS_FreeValue(ctx, v);
            dump_exception(out, ctx);
        }
        let mut arg = [obj];
        let sealed = js_object_seal(ctx, JS_UNDEFINED, 1, arg.as_mut_ptr(), fail & 1);
        dump_val(out, sealed);
        JS_FreeValue(ctx, sealed);
        dump_exception(out, ctx);
        for frozen in 0..2 {
            let v = js_object_isSealed(ctx, JS_UNDEFINED, 1, arg.as_mut_ptr(), frozen);
            dump_val(out, v);
            JS_FreeValue(ctx, v);
            dump_exception(out, ctx);
        }
        JS_FreeValue(ctx, desc);
        JS_FreeValue(ctx, obj);
    }
    let mut proxy_data: JSProxyData = core::mem::zeroed();
    let proxy = JS_NewObjectClass(ctx, JS_CLASS_PROXY as i32);
    let p = JS_VALUE_GET_PTR(proxy).cast::<JSObject>();
    (*p).u.proxy_data = &mut proxy_data;
    proxy_data.target = JS_NewArray(ctx);
    num(out, JS_IsArray(ctx, proxy) as u64, 4);
    dump_exception(out, ctx);
    proxy_data.is_revoked = 1;
    num(out, JS_IsArray(ctx, proxy) as u64, 4);
    dump_exception(out, ctx);
    JS_FreeValue(ctx, proxy_data.target);
    (*p).u.proxy_data = ptr::null_mut();
    JS_FreeValue(ctx, proxy);
    let mut rope = JS_AtomToString(ctx, crate::quickjs_atom::JS_ATOM_empty_string);
    for step in 0..192 {
        let units: Vec<u16> = (0..(513 + step % 13))
            .map(|i| ((i + step) % 503 + 32) as u16)
            .collect();
        let leaf = js_new_string16_len(ctx, units.as_ptr(), units.len() as i32);
        rope = if step % 7 == 0 {
            JS_ConcatString(ctx, leaf, rope)
        } else {
            JS_ConcatString(ctx, rope, leaf)
        };
        num(out, JS_VALUE_GET_TAG(rope) as u64, 4);
        let len = string_rope_get_len(rope);
        num(out, len as u64, 4);
        if JS_VALUE_GET_TAG(rope) == JS_TAG_STRING_ROPE {
            let r = JS_VALUE_GET_PTR(rope).cast::<JSStringRope>();
            num(out, (*r).depth as u64, 4);
            num(out, (*r).is_wide_char as u64, 4);
        } else {
            num(out, 0, 4);
            num(
                out,
                (*JS_VALUE_GET_PTR(rope).cast::<JSString>()).is_wide_char() as u64,
                4,
            );
        }
        for idx in [0, len / 2, len - 1] {
            num(out, string_rope_get(rope, idx) as u64, 4);
        }
    }
    let flat = JS_ToString(ctx, rope);
    dump_val(out, flat);
    JS_FreeValue(ctx, flat);
    let suffix = JS_NewString(ctx, c"tail".as_ptr());
    rope = JS_ConcatString(ctx, rope, suffix);
    let flat = JS_ToStringFree(ctx, rope);
    dump_val(out, flat);
    JS_FreeValue(ctx, flat);
    for fail in 1..32 {
        let left = JS_NewString(ctx, c"left".as_ptr());
        let units = [0x100u16; 1024];
        let right = js_new_string16_len(ctx, units.as_ptr(), 1024);
        (*h).fail = (*h).calls + fail;
        let value = JS_ConcatString(ctx, left, right);
        (*h).fail = 0;
        let flat = JS_ToStringFree(ctx, value);
        dump_val(out, flat);
        JS_FreeValue(ctx, flat);
        dump_exception(out, ctx);
    }
}
