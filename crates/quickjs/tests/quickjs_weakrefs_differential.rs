const WEAK_FUNCTIONS: &[&str] = &[
    "js_weakref_is_target",
    "js_weakref_is_live",
    "js_weakref_free",
    "js_weakref_new",
    "map_normalize_key",
    "map_normalize_key_const",
    "map_hash32",
    "map_hash64",
    "map_hash_pointer",
    "map_hash_key",
    "map_delete_record_internal",
    "map_decref_record",
    "map_delete_weakrefs",
    "js_map_finalizer",
    "js_map_mark",
    "js_weakref_finalizer",
    "weakref_delete_weakref",
    "js_finrec_finalizer",
    "js_finrec_mark",
    "JS_GetOpaque",
    "js_bigint_set_si",
    "js_bigint_set_short",
];
fn weak_source(c: &str) -> String {
    let mut oracle = gc_source(c);
    for (begin, end) in [
        (
            "typedef struct JSWeakRefData {",
            "static void js_weakref_finalizer",
        ),
        (
            "typedef struct JSFinRecEntry {",
            "static void js_finrec_finalizer",
        ),
    ] {
        let a = c.find(begin).unwrap();
        let b = a + c[a..].find(end).unwrap();
        oracle.push_str(&c[a..b]);
    }
    oracle.push_str(
        "#define HASH_MUL32 0x61C88647\n#define HASH_MUL64 UINT64_C(0x61C8864680B583EB)\n",
    );
    source::append_functions(&mut oracle, c, WEAK_FUNCTIONS);
    oracle
}
fn weak_rng(a: &mut u64) -> u64 {
    *a ^= *a << 13;
    *a ^= *a >> 7;
    *a ^= *a << 17;
    *a
}
unsafe fn weak_snapshot(out: &mut Vec<u8>, h: &Host, s: *mut JSMapState) {
    num(out, (*s).record_count as u64, 4);
    num(
        out,
        ListIter::new(&mut (*s).records, false, false).count() as u64,
        4,
    );
    for el in ListIter::new(&mut (*s).records, false, false) {
        let r = el
            .cast::<u8>()
            .sub(offset_of!(JSMapRecord, link))
            .cast::<JSMapRecord>();
        for n in [
            (*r).ref_count,
            (*r).empty as i32,
            JS_VALUE_GET_TAG((*r).key),
            JS_VALUE_GET_INT((*r).value),
        ] {
            num(out, n as u64, 4);
        }
    }
    let mut buckets = 0;
    for j in 0..(*s).hash_size {
        let mut r = *(*s).hash_table.add(j as usize);
        while !r.is_null() {
            assert!((*r).empty == 0);
            assert!(js_weakref_is_live((*r).key) != 0);
            buckets += 1;
            r = (*r).hash_next;
        }
    }
    num(out, buckets, 4);
    num(out, h.calls as u64, 4);
    num(out, h.live as u64, 4);
    num(out, h.trace, 8);
}
#[test]
fn official_c_weak_ownership_hashing_and_map_cleanup_match() {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let upstream = root.join("../../vendor/quickjs-2026-06-04");
    let c = std::fs::read_to_string(upstream.join("quickjs.c")).unwrap();
    let mut oracle = weak_source(&c);
    oracle.push_str(
        include_str!("quickjs_atoms_oracle.c")
            .split("int main(void){")
            .next()
            .unwrap(),
    );
    oracle.push_str(include_str!("quickjs_weakrefs_oracle.c"));
    let directory =
        std::env::temp_dir().join(format!("quickjs-weak-oracle-{}", std::process::id()));
    std::fs::create_dir_all(&directory).unwrap();
    let path = directory.join("weak.c");
    let executable = directory.join("weak");
    std::fs::write(&path, oracle).unwrap();
    let mut command = include!("quickjs_oracle_config.rs");
    command.args(["-std=c11", "-O2", "-I"]).arg(upstream);
    if cfg!(feature = "short-opcodes") {
        command.arg("-DSHORT_OPCODES=1");
    }
    command.arg(&path).arg("-o").arg(&executable);
    let result = command.output().unwrap();
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    let c = std::process::Command::new(&executable).output().unwrap();
    let _ = std::fs::remove_dir_all(directory);
    assert!(
        c.status.success(),
        "C weak oracle failed: {}",
        String::from_utf8_lossy(&c.stderr)
    );
    let mut out = Vec::new();
    unsafe {
        let mut rt: Box<JSRuntime> = Box::new(core::mem::zeroed());
        let rt = &mut *rt as *mut JSRuntime;
        let mut h = host(0);
        initialize(rt, &mut h);
        for head in [
            ptr::addr_of_mut!((*rt).gc_obj_list),
            ptr::addr_of_mut!((*rt).gc_zero_ref_count_list),
            ptr::addr_of_mut!((*rt).tmp_obj_list),
            ptr::addr_of_mut!((*rt).context_list),
            ptr::addr_of_mut!((*rt).weakref_list),
        ] {
            init_list_head(head);
        }
        assert_eq!(JS_InitAtoms(rt), 0);
        (*rt).current_exception = JS_UNINITIALIZED;
        let mut ctx: JSContext = core::mem::zeroed();
        ctx.rt = rt;
        for n in [
            size_of::<JSWeakRefData>(),
            size_of::<JSFinRecEntry>(),
            size_of::<JSFinalizationRegistryData>(),
        ] {
            num(&mut out, n as u64, 4);
        }
        let mut rng = 0x123456789abcdef0;
        for i in 0..16384 {
            let a = weak_rng(&mut rng);
            let bits = 1 + i % 31;
            for n in [
                map_hash32(a as u32, bits),
                map_hash64(a, bits),
                map_hash_pointer(a as usize, bits),
            ] {
                num(&mut out, n as u64, 4);
            }
            let mut v = JS_NewInt32(&mut ctx, a as i32);
            num(&mut out, map_hash_key(v, bits) as u64, 4);
            v = __JS_NewFloat64(&mut ctx, a as i64 as f64 / 17.0);
            num(&mut out, map_hash_key(v, bits) as u64, 4);
            v = __JS_NewFloat64(&mut ctx, if i % 2 != 0 { -0.0 } else { f64::NAN });
            num(&mut out, map_hash_key(v, bits) as u64, 4);
            num(
                &mut out,
                JS_VALUE_GET_TAG(map_normalize_key_const(&mut ctx, v)) as u64,
                4,
            );
            num(
                &mut out,
                JS_VALUE_GET_INT(map_normalize_key(&mut ctx, v)) as u64,
                4,
            );
            for tag in [JS_TAG_OBJECT, JS_TAG_SYMBOL] {
                num(
                    &mut out,
                    map_hash_key(JS_MKPTR(tag, a as usize as *mut c_void), bits) as u64,
                    4,
                );
            }
            num(
                &mut out,
                map_hash_key(__JS_NewShortBigInt(&mut ctx, a as i64), bits) as u64,
                4,
            );
            let mut buf = [0 as js_limb_t; 9];
            let b = buf.as_mut_ptr().cast::<JSBigInt>();
            (*b).len = (1 + i % 8) as u32;
            for j in 0..(*b).len as usize {
                *ptr::addr_of_mut!((*b).tab).cast::<js_limb_t>().add(j) =
                    weak_rng(&mut rng) as js_limb_t;
            }
            num(
                &mut out,
                map_hash_key(JS_MKPTR(JS_TAG_BIG_INT, b.cast()), bits) as u64,
                4,
            );
            let str = [a as u16, (a >> 16) as u16, (a >> 32) as u16, 1, 0];
            let p = make_string(rt, &str, (i % 2) as u32);
            v = JS_MKPTR(JS_TAG_STRING, p.cast());
            num(&mut out, map_hash_key(v, bits) as u64, 4);
            JS_FreeValueRT(rt, v);
            let rope = js_mallocz_rt(rt, size_of::<JSStringRope>()).cast::<JSStringRope>();
            (*js_rc(rope.cast())).ref_count = 1;
            (*rope).left = JS_MKPTR(
                JS_TAG_STRING,
                make_string(rt, &str[..3], (i % 2) as u32).cast(),
            );
            (*rope).right = JS_MKPTR(JS_TAG_STRING, make_string(rt, &str[3..], 1).cast());
            (*rope).len = 5;
            (*rope).is_wide_char = 1;
            num(
                &mut out,
                map_hash_key(JS_MKPTR(JS_TAG_STRING_ROPE, rope.cast()), bits) as u64,
                4,
            );
            JS_FreeValueRT(rt, JS_MKPTR(JS_TAG_STRING_ROPE, rope.cast()));
        }
        for trial in 0..512 {
            let s = js_mallocz_rt(rt, size_of::<JSMapState>()).cast::<JSMapState>();
            init_list_head(&mut (*s).records);
            (*s).is_weak = 1;
            (*s).hash_bits = 4;
            (*s).hash_size = 16;
            (*s).hash_table = js_mallocz_rt(rt, 16 * size_of::<*mut JSMapRecord>()).cast();
            (*s).weakref_header.weakref_type = JS_WEAKREF_TYPE_MAP;
            list_add_tail(&mut (*s).weakref_header.link, &mut (*rt).weakref_list);
            let mut keys = [JS_UNDEFINED; 64];
            let mut records = [ptr::null_mut(); 64];
            let mut weakrefs = [ptr::null_mut(); 64];
            let mut dead = [false; 64];
            for i in 0..64 {
                let key = if i % 2 != 0 {
                    let atom = __JS_NewAtomInit(rt, c"weak-key".as_ptr(), 8, JS_ATOM_TYPE_SYMBOL);
                    JS_MKPTR(JS_TAG_SYMBOL, (*(*rt).atom_array.add(atom as usize)).cast())
                } else {
                    let p = js_mallocz_rt(rt, size_of::<JSObject>()).cast::<JSObject>();
                    (*js_rc(p.cast())).ref_count = 1;
                    JS_MKPTR(JS_TAG_OBJECT, p.cast())
                };
                keys[i] = key;
                num(&mut out, js_weakref_is_target(key) as u64, 4);
                num(&mut out, js_weakref_is_live(key) as u64, 4);
                let r = js_mallocz_rt(rt, size_of::<JSMapRecord>()).cast::<JSMapRecord>();
                records[i] = r;
                (*r).ref_count = if i % 3 == 0 { 2 } else { 1 };
                (*r).key = js_weakref_new(&mut ctx, key);
                (*r).value = JS_NewInt32(&mut ctx, i as i32);
                let bucket = map_hash_key(key, 4) as usize;
                (*r).hash_next = *(*s).hash_table.add(bucket);
                *(*s).hash_table.add(bucket) = r;
                list_add_tail(&mut (*r).link, &mut (*s).records);
                (*s).record_count += 1;
                let w = js_mallocz_rt(rt, size_of::<JSWeakRefData>()).cast::<JSWeakRefData>();
                weakrefs[i] = w;
                (*w).target = js_weakref_new(&mut ctx, key);
                (*w).weakref_header.weakref_type = JS_WEAKREF_TYPE_WEAKREF;
                list_add_tail(&mut (*w).weakref_header.link, &mut (*rt).weakref_list);
                dead[i] = (trial + i) % 3 != 0;
                if dead[i] {
                    (*js_rc(JS_VALUE_GET_PTR(key))).ref_count = 0;
                    if i % 2 != 0 {
                        JS_FreeAtomStruct(rt, JS_VALUE_GET_PTR(key).cast());
                    }
                }
            }
            if trial % 2 != 0 {
                ptr::write_bytes((*s).hash_table, 0, 16);
                for i in 0..64 {
                    if !dead[i] {
                        let r = records[i];
                        let bucket = map_hash_key(keys[i], 4) as usize;
                        (*r).hash_next = *(*s).hash_table.add(bucket);
                        *(*s).hash_table.add(bucket) = r;
                    }
                }
            }
            map_delete_weakrefs(rt, &mut (*s).weakref_header);
            weak_snapshot(&mut out, &h, s);
            for i in 0..64 {
                weakref_delete_weakref(rt, &mut (*weakrefs[i]).weakref_header);
                num(&mut out, JS_IsUndefined((*weakrefs[i]).target) as u64, 4);
                if dead[i] && i % 3 == 0 {
                    map_decref_record(rt, records[i]);
                }
            }
            weak_snapshot(&mut out, &h, s);
            for w in weakrefs {
                let mut holder: JSObject = core::mem::zeroed();
                holder.class_id = JS_CLASS_WEAK_REF as u16;
                holder.u.opaque = w.cast();
                js_weakref_finalizer(
                    rt,
                    JS_MKPTR(JS_TAG_OBJECT, ptr::from_mut(&mut holder).cast()),
                );
            }
            let mut holder: JSObject = core::mem::zeroed();
            holder.u.map_state = s;
            js_map_finalizer(
                rt,
                JS_MKPTR(JS_TAG_OBJECT, ptr::from_mut(&mut holder).cast()),
            );
            for i in 0..64 {
                if !dead[i] {
                    if i % 2 != 0 {
                        JS_FreeValueRT(rt, keys[i]);
                    } else {
                        js_free_rt(rt, JS_VALUE_GET_PTR(keys[i]));
                    }
                }
            }
            num(&mut out, list_empty(&mut (*rt).weakref_list) as u64, 4);
            snapshot(&mut out, rt, &h);
        }
        num(&mut out, js_weakref_is_target(JS_UNDEFINED) as u64, 4);
        num(&mut out, js_weakref_is_live(JS_UNDEFINED) as u64, 4);
        num(
            &mut out,
            JS_IsUndefined(js_weakref_new(&mut ctx, JS_UNDEFINED)) as u64,
            4,
        );
        js_weakref_free(rt, JS_UNDEFINED);
        cleanup(&mut out, rt, &mut h);
    }
    assert_eq!(out.len(), c.stdout.len());
    if let Some(i) = out.iter().zip(&c.stdout).position(|(r, c)| r != c) {
        panic!(
            "weak C/Rust mismatch at byte {i}: Rust={}, C={}",
            out[i], c.stdout[i]
        );
    }
    eprintln!(
        "quickjs.c weak ownership/map parity: 16384 hash fixtures, 32768 weak keys, {} bytes",
        out.len()
    );
}
