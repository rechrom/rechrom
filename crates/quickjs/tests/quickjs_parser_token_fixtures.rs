pub unsafe fn parser_token_fixture() {
    let rt = JS_NewRuntime();
    let ctx = JS_NewContext(rt);
    assert!(!ctx.is_null());
    let mut s: JSParseState = core::mem::zeroed();
    let mut fd: JSFunctionDef = core::mem::zeroed();
    for text in [
        c"var alpha=12.25; if (alpha!==0) alpha++; null false true this let await yield",
        c"'abc' \"é\\n\" `hello` <<= => ... ?? ?. += && ||",
        c"function f(a){return a+1;} class X extends Y {} //comment\n",
    ] {
        js_parse_init(
            ctx,
            &mut s,
            text.as_ptr(),
            text.to_bytes().len(),
            c"token.js".as_ptr(),
        );
        s.cur_func = &mut fd;
        loop {
            assert_eq!(next_token(&mut s), 0);
            dump_token(&mut s, core::ptr::addr_of!(s.token));
            if s.token.val == TOK_EOF {
                break;
            }
        }
        free_token(&mut s, &mut s.token);
    }
    s.token.val = TOK_NUMBER;
    let mut bits = 0x0123456789abcdefu64;
    for _ in 0..12000 {
        bits = bits
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        s.token.u.num.val = __JS_NewFloat64(ctx, f64::from_bits(bits));
        dump_token(&mut s, core::ptr::addr_of!(s.token));
    }
    for n in [
        0.0,
        -0.0,
        1e-5,
        1e-4,
        1e13,
        1e14,
        1.23456789012345,
        9.99999999999995e-5,
        99999999999999.5,
    ] {
        s.token.u.num.val = __JS_NewFloat64(ctx, n);
        dump_token(&mut s, core::ptr::addr_of!(s.token));
    }
    s.token.val = TOK_REGEXP;
    s.token.u.regexp = JSTokenRegexp {
        body: JS_NewString(ctx, c"a[é]+\\/".as_ptr()),
        flags: JS_NewString(ctx, c"gimu".as_ptr()),
    };
    dump_token(&mut s, core::ptr::addr_of!(s.token));
    free_token(&mut s, &mut s.token);
    s.token.val = 301;
    dump_token(&mut s, core::ptr::addr_of!(s.token));
    s.token.val = TOK_NULL;
    s.token.u.ident.atom = crate::quickjs_atom::JS_ATOM_null;
    let other = JSToken {
        val: 301,
        ..s.token
    };
    dump_token(&mut s, &other);
    s.token.val = TOK_ERROR;
    JS_FreeContext(ctx);
    JS_FreeRuntime(rt);
}
