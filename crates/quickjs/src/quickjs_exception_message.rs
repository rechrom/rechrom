// V8 96550751177dbdb761b1708431616374148bc0eb objects.cc
// Object::NoSideEffectsToMaybeString / NoSideEffectsErrorToString.
// VM-native data lookup deliberately bypasses accessors, proxies and host
// interceptors. No author JavaScript runs while constructing an exception message.
unsafe fn js_message_data_property(value: JSValue, atom: JSAtom) -> JSValue {
    if JS_VALUE_GET_TAG(value) != JS_TAG_OBJECT {
        return JS_UNDEFINED;
    }
    let mut object = JS_VALUE_GET_PTR(value).cast::<JSObject>();
    while !object.is_null() {
        if (*object).class_id as u32 == JS_CLASS_PROXY {
            return JS_UNDEFINED;
        }
        let mut property = ptr::null_mut();
        let shape = find_own_property(&mut property, object, atom);
        if !shape.is_null() {
            return if (*shape).flags() as i32 & JS_PROP_TMASK == JS_PROP_NORMAL {
                (*property).u.value
            } else {
                JS_UNDEFINED
            };
        }
        object = (*(*object).shape).proto;
    }
    JS_UNDEFINED
}
unsafe fn js_message_units(value: JSValue) -> Vec<u16> {
    if JS_IsString(value) != 0 {
        JS_CopyString16(value)
    } else {
        Vec::new()
    }
}
/// Return an owned primitive string, without executing property getters or
/// coercion hooks. Intrinsic identities were captured before page script ran.
pub unsafe fn JS_NoSideEffectsToString(
    ctx: *mut JSContext,
    mut value: JSValue,
    error_to_string: JSValue,
    object_to_string: JSValue,
) -> JSValue {
    while JS_VALUE_GET_TAG(value) == JS_TAG_OBJECT {
        let object = JS_VALUE_GET_PTR(value).cast::<JSObject>();
        if (*object).class_id as u32 != JS_CLASS_PROXY {
            break;
        }
        let proxy = (*object).u.opaque.cast::<JSProxyData>();
        value = if proxy.is_null() || (*proxy).is_revoked != 0 {
            JS_NULL
        } else {
            (*proxy).target
        };
    }
    if JS_VALUE_GET_TAG(value) != JS_TAG_OBJECT {
        if JS_VALUE_GET_TAG(value) == JS_TAG_BIG_INT {
            // bigint.cc:989-1019 caps the *magnitude's* native word count.
            let bigint = JS_VALUE_GET_PTR(value).cast::<JSBigInt>();
            let mut exponent = 0;
            let mantissa = js_bigint_get_mant_exp(ctx, &mut exponent, bigint);
            if mantissa != 0 && exponent >= 100 * usize::BITS as i32 {
                return JS_NewString16(
                    ctx,
                    &"<a very large BigInt>".encode_utf16().collect::<Vec<_>>(),
                );
            }
        }
        if JS_VALUE_GET_TAG(value) == JS_TAG_SYMBOL {
            let text = js_symbol_toString(ctx, value, 0, ptr::null_mut());
            if JS_IsException(text) != 0 {
                return text;
            }
            let units = JS_CopyString16(text);
            JS_FreeValue(ctx, text);
            // Symbol(description): source caps the description, not its wrapper.
            let units = if units.len() > 8 + 128 {
                let mut capped = units[..7 + 56].to_vec();
                capped.extend("...<omitted>...".encode_utf16());
                capped.extend_from_slice(&units[units.len() - 57..]);
                capped
            } else {
                units
            };
            return JS_NewString16(ctx, &units);
        }
        return JS_ToString(ctx, value);
    }
    let object = JS_VALUE_GET_PTR(value).cast::<JSObject>();
    if JS_IsFunction(ctx, value) != 0 {
        let class = (*object).class_id as u32;
        let mut text = if js_class_has_bytecode(class) != 0 {
            let bytecode = (*object).u.func.function_bytecode;
            if (*bytecode).has_debug() != 0 && !(*bytecode).debug.source.is_null() {
                let text = JS_NewStringLen(
                    ctx,
                    (*bytecode).debug.source,
                    (*bytecode).debug.source_len as usize,
                );
                if JS_IsException(text) != 0 {
                    return text;
                }
                let units = JS_CopyString16(text);
                JS_FreeValue(ctx, text);
                units
            } else {
                "function () { [native code] }".encode_utf16().collect()
            }
        } else {
            // Bound functions have the source's fixed native function text.
            let mut text: Vec<u16> = "function ".encode_utf16().collect();
            if class != JS_CLASS_BOUND_FUNCTION {
                text.extend(js_message_units(js_message_data_property(
                    value,
                    crate::quickjs_atom::JS_ATOM_name,
                )));
            }
            text.extend("() { [native code] }".encode_utf16());
            text
        };
        if text.len() > 128 {
            let last = text[text.len() - 2..].to_vec();
            text.truncate(111);
            text.extend("...<omitted>...".encode_utf16());
            text.extend(last);
        }
        return JS_NewString16(ctx, &text);
    }
    let to_string = js_message_data_property(value, crate::quickjs_atom::JS_ATOM_toString);
    if JS_IsError(ctx, value) != 0 || JS_SameValue(ctx, to_string, error_to_string) != 0 {
        let mut name = js_message_units(js_message_data_property(
            value,
            crate::quickjs_atom::JS_ATOM_name,
        ));
        let message = js_message_units(js_message_data_property(
            value,
            crate::quickjs_atom::JS_ATOM_message,
        ));
        if name.is_empty() {
            return JS_NewString16(ctx, &message);
        }
        if !message.is_empty() {
            name.extend(": ".encode_utf16());
            name.extend(message);
        }
        return JS_NewString16(ctx, &name);
    }
    if JS_SameValue(ctx, to_string, object_to_string) != 0 {
        let constructor = js_message_data_property(value, crate::quickjs_atom::JS_ATOM_constructor);
        if JS_IsFunction(ctx, constructor) != 0 {
            let name = js_message_units(js_message_data_property(
                constructor,
                crate::quickjs_atom::JS_ATOM_name,
            ));
            if !name.is_empty() {
                let mut text: Vec<u16> = "#<".encode_utf16().collect();
                text.extend(name);
                text.push('>' as u16);
                return JS_NewString16(ctx, &text);
            }
        }
    }
    let tag = js_message_data_property(value, crate::quickjs_atom::JS_ATOM_Symbol_toStringTag);
    let tag = if JS_IsString(tag) != 0 {
        JS_DupValue(ctx, tag)
    } else {
        JS_AtomToString(
            ctx,
            (*(*ctx).rt)
                .class_array
                .add((*object).class_id as usize)
                .as_ref()
                .expect("registered native class")
                .class_name,
        )
    };
    let mut units = js_message_units(tag);
    JS_FreeValue(ctx, tag);
    units.truncate(1000);
    let mut text: Vec<u16> = "[object ".encode_utf16().collect();
    text.extend(units);
    text.push(']' as u16);
    JS_NewString16(ctx, &text)
}
