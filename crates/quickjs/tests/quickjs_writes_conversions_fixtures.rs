// Differential fixtures for complete property writes and scalar/object conversions.
unsafe fn writes_conversions_fixtures(out: &mut Vec<u8>, ctx: *mut JSContext, h: *mut Host) {
    let mut rng = 0x9a58723f264b17ceu64;
    for _ in 0..4096 {
        rng ^= rng << 13;
        rng ^= rng >> 7;
        rng ^= rng << 17;
        let val = __JS_NewFloat64(ctx, f64::from_bits(rng));
        let mut n = 0;
        num(out, JS_ToInt32Sat(ctx, &mut n, val) as u64, 4);
        num(out, n as u64, 4);
        num(
            out,
            JS_ToInt32Clamp(ctx, &mut n, val, -512, 512, 1024) as u64,
            4,
        );
        num(out, n as u64, 4);
        let mut q = 0;
        num(out, JS_ToInt64(ctx, &mut q, val) as u64, 4);
        num(out, q as u64, 8);
        num(out, JS_ToInt64Sat(ctx, &mut q, val) as u64, 4);
        num(out, q as u64, 8);
        num(
            out,
            JS_ToInt64Clamp(ctx, &mut q, val, -4096, 4096, 8192) as u64,
            4,
        );
        num(out, q as u64, 8);
        num(out, JS_ToUint8ClampFree(ctx, &mut n, val) as u64, 4);
        num(out, n as u64, 4);
        let integer = JS_ToIntegerFree(ctx, val);
        dump_val(out, integer);
        JS_FreeValue(ctx, integer);
        num(out, is_safe_integer(JS_VALUE_GET_FLOAT64(val)) as u64, 4);
        let mut index = 0x5555;
        num(out, JS_ToIndex(ctx, &mut index, val) as u64, 4);
        num(out, index, 8);
        dump_exception(out, ctx);
        num(out, JS_ToLengthFree(ctx, &mut q, val) as u64, 4);
        num(out, q as u64, 8);
    }
    for text in [
        c"",
        c"  \t\n",
        c"0",
        c"-0",
        c"0xabcdef1234567890123456789",
        c"-123456789012345678901234567890",
        c"+15",
        c"-0x1",
        c"0b101",
        c"0o77",
        c"1n",
        c"1e3",
        c"1.5",
        c"1_2",
        c"Infinity",
        c"42z",
        c"  9007199254740993 \u{a0}",
    ] {
        let val = JS_NewString(ctx, text.as_ptr());
        let converted = JS_ToBigInt(ctx, val);
        dump_val(out, converted);
        dump_exception(out, ctx);
        JS_FreeValue(ctx, converted);
        let mut n = 0;
        num(out, JS_ToBigInt64(ctx, &mut n, val) as u64, 4);
        num(out, n as u64, 8);
        dump_exception(out, ctx);
        JS_FreeValue(ctx, val);
    }
    for val in [
        JS_TRUE,
        JS_FALSE,
        JS_NULL,
        JS_UNDEFINED,
        JS_NewInt32(ctx, 1),
        JS_NewFloat64(ctx, 1.5),
        JS_NewBigInt64(ctx, i64::MIN),
        JS_NewBigUint64(ctx, u64::MAX),
    ] {
        let mut n = 0;
        num(out, JS_ToBigInt64(ctx, &mut n, val) as u64, 4);
        num(out, n as u64, 8);
        dump_exception(out, ctx);
        num(out, JS_ToInt64Ext(ctx, &mut n, val) as u64, 4);
        num(out, n as u64, 8);
        dump_exception(out, ctx);
        let boxed = JS_ToObject(ctx, val);
        dump_val(out, boxed);
        dump_exception(out, ctx);
        if JS_IsObject(boxed) != 0 {
            dump_val(
                out,
                (*JS_VALUE_GET_PTR(boxed).cast::<JSObject>()).u.object_data,
            );
        }
        JS_FreeValue(ctx, boxed);
        JS_FreeValue(ctx, val);
    }
    for fail in 0..64 {
        let proto = JS_NewObject(ctx);
        let obj = JS_NewObjectProto(ctx, proto);
        let receiver = JS_NewObject(ctx);
        let atom = __JS_AtomFromUInt32(42);
        JS_DefinePropertyValue(ctx, proto, atom, JS_NewInt32(ctx, 7), JS_PROP_C_W_E);
        if fail != 0 {
            (*h).fail = (*h).calls + fail;
        }
        for step in 0..24 {
            let ret = match step % 8 {
                0 => JS_SetPropertyInternal(
                    ctx,
                    obj,
                    atom,
                    JS_NewInt32(ctx, step),
                    obj,
                    JS_PROP_THROW,
                ),
                1 => JS_SetPropertyInternal(
                    ctx,
                    proto,
                    atom,
                    JS_NewInt32(ctx, step),
                    receiver,
                    JS_PROP_THROW,
                ),
                2 => JS_SetPropertyStr(ctx, obj, c"created".as_ptr(), JS_NewInt32(ctx, step)),
                3 => JS_DeletePropertyInt64(ctx, obj, 42, JS_PROP_THROW),
                4 => JS_DefineProperty(
                    ctx,
                    obj,
                    atom,
                    JS_UNDEFINED,
                    JS_UNDEFINED,
                    JS_UNDEFINED,
                    JS_PROP_HAS_GET
                        | JS_PROP_HAS_SET
                        | JS_PROP_HAS_CONFIGURABLE
                        | JS_PROP_CONFIGURABLE,
                ),
                5 => JS_SetPropertyInternal(
                    ctx,
                    obj,
                    atom,
                    JS_NewInt32(ctx, step),
                    obj,
                    JS_PROP_THROW,
                ),
                6 => JS_SetPropertyInternal(ctx, JS_NULL, atom, JS_NewInt32(ctx, step), JS_NULL, 0),
                _ => JS_SetPropertyInt64(ctx, receiver, -1, JS_NewInt32(ctx, step)),
            };
            num(out, ret as u64, 4);
            dump_exception(out, ctx);
        }
        (*h).fail = 0;
        JS_PreventExtensions(ctx, receiver);
        num(
            out,
            JS_SetPropertyInternal(
                ctx,
                proto,
                atom,
                JS_NewInt32(ctx, 1),
                receiver,
                JS_PROP_THROW,
            ) as u64,
            4,
        );
        dump_exception(out, ctx);
        dump_object(out, JS_VALUE_GET_PTR(obj).cast());
        dump_object(out, JS_VALUE_GET_PTR(receiver).cast());
        JS_FreeValue(ctx, receiver);
        JS_FreeValue(ctx, obj);
        JS_FreeValue(ctx, proto);
    }
    let arr = JS_NewArray(ctx);
    for idx in [0, 1, 2, 7, 3, 0, u32::MAX] {
        num(
            out,
            JS_SetPropertyUint32(ctx, arr, idx, JS_NewInt32(ctx, idx as i32)) as u64,
            4,
        );
        dump_exception(out, ctx);
    }
    num(
        out,
        JS_SetPropertyStr(ctx, arr, c"length".as_ptr(), JS_NewInt32(ctx, 2)) as u64,
        4,
    );
    dump_exception(out, ctx);
    dump_object(out, JS_VALUE_GET_PTR(arr).cast());
    JS_FreeValue(ctx, arr);
    for class in JS_CLASS_UINT8C_ARRAY..=JS_CLASS_FLOAT64_ARRAY {
        let obj = JS_NewObjectClass(ctx, class as i32);
        let p = JS_VALUE_GET_PTR(obj).cast::<JSObject>();
        let mut data = [0u64; 4];
        (*p).u.array.u.ptr = data.as_mut_ptr().cast();
        (*p).u.array.count = 4;
        for (step, d) in [
            0.5,
            1.5,
            2.5,
            254.5,
            255.5,
            -1.0,
            f64::NAN,
            65535.0,
            4294967295.0,
            f64::INFINITY,
        ]
        .into_iter()
        .enumerate()
        {
            let val = if class == JS_CLASS_BIG_INT64_ARRAY || class == JS_CLASS_BIG_UINT64_ARRAY {
                JS_NewBigInt64(ctx, ((step as i64) << 60).wrapping_sub(17))
            } else {
                JS_NewFloat64(ctx, d)
            };
            num(
                out,
                JS_SetPropertyValue(ctx, obj, JS_NewInt32(ctx, (step % 6) as i32), val, 0) as u64,
                4,
            );
            dump_exception(out, ctx);
            out.extend_from_slice(core::slice::from_raw_parts(data.as_ptr().cast::<u8>(), 32));
        }
        for key in [c"-0", c"1.5", c"Infinity", c"99", c"ordinary"] {
            let val = if class == JS_CLASS_BIG_INT64_ARRAY || class == JS_CLASS_BIG_UINT64_ARRAY {
                JS_NewString(ctx, c"-18446744073709551617".as_ptr())
            } else {
                JS_NewString(ctx, c"256.5".as_ptr())
            };
            num(
                out,
                JS_SetPropertyStr(ctx, obj, key.as_ptr(), val) as u64,
                4,
            );
            dump_exception(out, ctx);
        }
        (*p).u.array.u.ptr = ptr::null_mut();
        (*p).u.array.count = 0;
        JS_FreeValue(ctx, obj);
    }
}
