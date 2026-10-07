// Actual translated BigInt algorithms, compared with the official C source.
// Raw normalized limb arrays are checked; formatting cannot hide discrepancies.
unsafe fn vm_bigint_dump(out: &mut Vec<u8>, ctx: *mut JSContext, r: *mut JSBigInt) {
    assert!(!r.is_null());
    out.extend_from_slice(&(*r).len.to_le_bytes());
    for i in 0..(*r).len as usize { out.extend_from_slice(&(*bigint_tab(r).add(i)).to_le_bytes()); }
    js_free(ctx, r.cast());
}
pub unsafe fn vm_bigint_fixture() -> Vec<u8> {
    let mut out = Vec::new();
    let mut rt: JSRuntime = core::mem::zeroed();
    let mut ctx: JSContext = core::mem::zeroed();
    js_malloc_init(&mut rt.malloc_ctx);
    rt.malloc_ctx.mf = ptr::read(ptr::addr_of!(def_malloc_funcs));
    rt.malloc_ctx.malloc_state.malloc_limit = usize::MAX;
    rt.malloc_gc_threshold = usize::MAX;
    rt.current_exception = JS_UNINITIALIZED;
    ctx.rt = &mut rt;
    let mut seed = 0x78d3_4732_3219_237bu64;
    for i in 0..4096u32 {
        seed ^= seed.wrapping_shl(13); seed ^= seed.wrapping_shr(7); seed ^= seed.wrapping_shl(17);
        let x = if i < 8 { [0, 1, -1, i64::MIN, i64::MAX, -2, 2, -7][i as usize] } else { seed as i64 };
        seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        let y = if seed == 0 { 1 } else { seed as i64 };
        let a = js_bigint_new_si64(&mut ctx, x); let b = js_bigint_new_si64(&mut ctx, y);
        for op in [OP_and, OP_xor, OP_or] { let r = js_bigint_logic(&mut ctx, a, b, op as i32); vm_bigint_dump(&mut out, &mut ctx, r); }
        let r = js_bigint_mul(&mut ctx, a, b); vm_bigint_dump(&mut out, &mut ctx, r);
        for rem in 0..=1 { let r = js_bigint_divrem(&mut ctx, a, b, rem); vm_bigint_dump(&mut out, &mut ctx, r); }
        let r = js_bigint_not(&mut ctx, a); vm_bigint_dump(&mut out, &mut ctx, r);
        for shift in [0, 1, 31, 32, 63, 64, 65, 127, 128, 129, 1024] {
            let r = js_bigint_shl(&mut ctx, a, shift); vm_bigint_dump(&mut out, &mut ctx, r);
            let r = js_bigint_shr(&mut ctx, a, shift); vm_bigint_dump(&mut out, &mut ctx, r);
        }
        let r = js_bigint_new_di(&mut ctx, ((x as i128) << 64) | y as u64 as i128); vm_bigint_dump(&mut out, &mut ctx, r);
        for shift in [65, 127, 191] {
            let aa = js_bigint_shl(&mut ctx, a, shift);
            let bb = js_bigint_shl(&mut ctx, b, 33);
            for rem in 0..=1 { let r = js_bigint_divrem(&mut ctx, aa, bb, rem); vm_bigint_dump(&mut out, &mut ctx, r); }
            let r = js_bigint_mul(&mut ctx, aa, bb); vm_bigint_dump(&mut out, &mut ctx, r);
            js_free(&mut ctx, aa.cast()); js_free(&mut ctx, bb.cast());
        }
        js_free(&mut ctx, a.cast()); js_free(&mut ctx, b.cast());
        let base = js_bigint_new_si64(&mut ctx, (i % 31) as i64 - 15);
        for exponent in [0, 1, 2, 3, 7, 16, 31] {
            let exp = js_bigint_new_si64(&mut ctx, exponent);
            let r = js_bigint_pow(&mut ctx, base, exp); vm_bigint_dump(&mut out, &mut ctx, r);
            js_free(&mut ctx, exp.cast());
        }
        js_free(&mut ctx, base.cast());
    }
    assert_eq!(rt.malloc_ctx.malloc_state.malloc_count, 0);
    out
}
