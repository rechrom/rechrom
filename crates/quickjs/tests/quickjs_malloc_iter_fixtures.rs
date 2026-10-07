include!("quickjs_diagnostic_allocator.rs");
struct IterFixture {
    slots: Vec<(*mut c_void, usize)>,
    out: Vec<u8>,
    count: usize,
}
unsafe fn iter_fixture_write(opaque: *mut c_void, p: *mut c_void) {
    let t = &mut *opaque.cast::<IterFixture>();
    t.count += 1;
    for (i, (q, n)) in t.slots.iter().enumerate() {
        if !q.is_null() && (p == *q || p == js_rc(*q).cast()) {
            use std::io::Write;
            let b = core::slice::from_raw_parts(q.cast::<u8>(), (*n).min(11));
            let sum = b.iter().fold(0u32, |a, b| a + *b as u32);
            writeln!(t.out, "{}:{}:{}", i, n, sum).unwrap();
            return;
        }
    }
    if !t.slots.is_empty() {
        panic!("iteration returned freed or unknown allocation");
    }
}
unsafe fn iter_fixture_snapshot(s: *mut JSMallocContext, t: &mut IterFixture, name: &str) {
    use std::io::Write;
    writeln!(t.out, "phase {}", name).unwrap();
    t.count = 0;
    js_malloc_iter(s, Some(iter_fixture_write), (t as *mut IterFixture).cast());
    writeln!(
        t.out,
        "count {} stats {} {}",
        t.count,
        (*s).malloc_state.malloc_count,
        (*s).malloc_state.malloc_size
    )
    .unwrap();
}
pub unsafe fn malloc_iter_fixture() -> Vec<u8> {
    use std::io::Write;
    let mut s: JSMallocContext = core::mem::zeroed();
    js_malloc_init(&mut s);
    s.mf = core::ptr::read(&DIAGNOSTIC_MALLOC);
    s.malloc_state.malloc_limit = usize::MAX;
    let mut t = IterFixture {
        slots: Vec::new(),
        out: Vec::new(),
        count: 0,
    };
    writeln!(
        t.out,
        "layouts {} {} {} {}",
        size_of::<JSMallocArena>(),
        size_of::<JSMallocBlockHeader>(),
        size_of::<JSMallocLargeBlockHeader>(),
        size_of::<JSMallocContext>()
    )
    .unwrap();
    for i in 0..650 {
        let n = if i < 320 {
            8
        } else if i < 520 {
            32
        } else {
            (i - 519) * 8
        };
        let p = __js_malloc(&mut s, n);
        assert!(!p.is_null());
        core::ptr::write_bytes(p, (i % 251) as u8, n);
        t.slots.push((p, n));
    }
    t.slots.push((__js_malloc(&mut s, 0), 0));
    iter_fixture_snapshot(&mut s, &mut t, "allocated");
    for i in (0..650).step_by(3) {
        __js_free(&mut s, t.slots[i].0);
        t.slots[i].0 = core::ptr::null_mut();
    }
    iter_fixture_snapshot(&mut s, &mut t, "holes");
    for i in 0..180 {
        let n = if i % 2 == 0 { 8 } else { 40 };
        let p = __js_malloc(&mut s, n);
        core::ptr::write_bytes(p, ((650 + i) % 251) as u8, n);
        t.slots.push((p, n));
    }
    iter_fixture_snapshot(&mut s, &mut t, "reuse");
    for i in [1, 4, 7, 523, 527, 620, 622, 625, 628] {
        if t.slots[i].0.is_null() {
            continue;
        }
        let (p, old) = t.slots[i];
        let n = if i % 2 == 0 { 64 } else { 1500 };
        let q = __js_realloc(&mut s, p, n);
        assert!(!q.is_null());
        for j in 0..old.min(n) {
            assert_eq!(*q.cast::<u8>().add(j), (i % 251) as u8);
        }
        core::ptr::write_bytes(q, (i % 251) as u8, n);
        t.slots[i] = (q, n);
    }
    iter_fixture_snapshot(&mut s, &mut t, "reallocated");
    s.malloc_state.malloc_limit = s.malloc_state.malloc_size;
    for i in [1, 4, 7, 620, 622, 625, 628] {
        if t.slots[i].0.is_null() {
            continue;
        }
        let p = __js_realloc(&mut s, t.slots[i].0, 50000);
        assert!(p.is_null());
    }
    assert!(__js_malloc(&mut s, 50000).is_null());
    iter_fixture_snapshot(&mut s, &mut t, "oom");
    s.malloc_state.malloc_limit = usize::MAX;
    for (p, _) in &mut t.slots {
        __js_free(&mut s, *p);
        *p = core::ptr::null_mut();
    }
    iter_fixture_snapshot(&mut s, &mut t, "freed");
    assert_eq!(s.malloc_state.malloc_count, 0);
    let rt = JS_NewRuntime2(&DIAGNOSTIC_MALLOC, core::ptr::null_mut());
    assert!(!rt.is_null());
    let ctx = JS_NewContext(rt);
    assert!(!ctx.is_null());
    t.slots.clear();
    iter_fixture_snapshot(core::ptr::addr_of_mut!((*rt).malloc_ctx), &mut t, "context");
    let text = c"(()=>{for(let i=0;i<300;i++){let a={i};a.self=a;}})()";
    let v = JS_Eval(
        ctx,
        text.as_ptr(),
        text.to_bytes().len(),
        c"iter.js".as_ptr(),
        0,
    );
    assert_eq!(JS_IsException(v), 0);
    JS_FreeValue(ctx, v);
    iter_fixture_snapshot(core::ptr::addr_of_mut!((*rt).malloc_ctx), &mut t, "cycles");
    JS_RunGC(rt);
    iter_fixture_snapshot(core::ptr::addr_of_mut!((*rt).malloc_ctx), &mut t, "gc");
    JS_FreeContext(ctx);
    JS_RunGC(rt);
    iter_fixture_snapshot(
        core::ptr::addr_of_mut!((*rt).malloc_ctx),
        &mut t,
        "contextfreed",
    );
    JS_FreeRuntime(rt);
    t.out
}
