fn shape_id(pool: &[*mut JSShape; 512], p: *mut JSShape) -> u32 {
    if p.is_null() {
        0
    } else {
        pool.iter()
            .position(|&x| x == p)
            .map_or(u32::MAX, |i| i as u32 + 1)
    }
}
unsafe fn shape_snapshot(
    out: &mut Vec<u8>,
    rt: *mut JSRuntime,
    h: &Host,
    pool: &[*mut JSShape; 512],
) {
    for n in [
        (*rt).shape_hash_bits,
        (*rt).shape_hash_size,
        (*rt).shape_hash_count,
        h.calls,
        h.live,
    ] {
        num(out, n as u64, 4);
    }
    num(out, h.trace, 8);
    if (*rt).shape_hash.is_null() {
        return;
    }
    for i in 0..(*rt).shape_hash_size {
        let mut p = *(*rt).shape_hash.add(i as usize);
        let mut n = 0;
        while !p.is_null() {
            n += 1;
            p = (*p).shape_hash_next;
        }
        num(out, n, 4);
        let mut p = *(*rt).shape_hash.add(i as usize);
        while !p.is_null() {
            num(out, shape_id(pool, p) as u64, 4);
            p = (*p).shape_hash_next;
        }
    }
}
#[test]
fn official_c_shape_layout_hash_chains_lookup_and_oom_match() {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let upstream = root.join("../../vendor/quickjs-2026-06-04");
    let c = std::fs::read_to_string(upstream.join("quickjs.c")).unwrap();
    let mut oracle = source::source_prefix(&c);
    let a = c.find("typedef struct JSProperty {").unwrap();
    let b = a + c[a..].find("struct JSObject {").unwrap();
    oracle.push_str(&c[a..b]);
    source::append_functions(&mut oracle, &c, ATOM_FUNCTIONS);
    source::append_functions(
        &mut oracle,
        &c,
        &[
            "get_shape_size",
            "get_shape_prop",
            "init_shape_hash",
            "shape_hash",
            "get_shape_hash",
            "shape_initial_hash",
            "resize_shape_hash",
            "js_shape_hash_link",
            "js_shape_hash_unlink",
            "js_dup_shape",
            "find_hashed_shape_proto",
            "find_hashed_shape_prop",
        ],
    );
    oracle.push_str(
        include_str!("quickjs_atoms_oracle.c")
            .split("int main(void){")
            .next()
            .unwrap(),
    );
    oracle.push_str(include_str!("quickjs_shapes_oracle.c"));
    let directory =
        std::env::temp_dir().join(format!("quickjs-shapes-oracle-{}", std::process::id()));
    std::fs::create_dir_all(&directory).unwrap();
    let path = directory.join("shapes.c");
    let executable = directory.join("shapes");
    std::fs::write(&path, oracle).unwrap();
    let result = include!("quickjs_oracle_config.rs")
        .args(["-std=c11", "-O2", "-I"])
        .arg(upstream)
        .arg(&path)
        .arg("-o")
        .arg(&executable)
        .output()
        .unwrap();
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    let c = std::process::Command::new(&executable).output().unwrap();
    let _ = std::fs::remove_dir_all(directory);
    assert!(
        c.status.success(),
        "C shapes oracle failed: {}",
        String::from_utf8_lossy(&c.stderr)
    );
    let mut out = Vec::new();
    for (size, align) in [
        (size_of::<JSProperty>(), core::mem::align_of::<JSProperty>()),
        (
            size_of::<JSShapeProperty>(),
            core::mem::align_of::<JSShapeProperty>(),
        ),
        (size_of::<JSShape>(), core::mem::align_of::<JSShape>()),
    ] {
        num(&mut out, size as u64, 4);
        num(&mut out, align as u64, 4);
    }
    num(&mut out, offset_of!(JSShape, hash_table) as u64, 4);
    num(&mut out, offset_of!(JSShape, proto) as u64, 4);
    let values = [0, 1, 0x3ffffff, 0x4000000, u32::MAX];
    for i in 0..5 {
        for j in 0..128 {
            let mut p = JSShapeProperty {
                hash_next_and_flags: 0,
                atom: values[4 - i],
            };
            p.set_hash_next(values[i]);
            p.set_flags(j);
            for n in [p.hash_next_and_flags, p.hash_next(), p.flags(), p.atom] {
                num(&mut out, n as u64, 4);
            }
        }
    }
    unsafe {
        for fail in 0..=16 {
            let mut rt: Box<JSRuntime> = Box::new(core::mem::zeroed());
            let rt = &mut *rt as *mut JSRuntime;
            let mut h = host(if fail == 1 { 1 } else { 0 });
            initialize(rt, &mut h);
            num(&mut out, init_shape_hash(rt) as u64, 4);
            let mut pool = [ptr::null_mut::<JSShape>(); 512];
            if (*rt).shape_hash.is_null() {
                shape_snapshot(&mut out, rt, &h, &pool);
                num(&mut out, h.live as u64, 4);
                continue;
            }
            let mut linked = [false; 512];
            for i in 0..512 {
                let group = i / 8;
                let n = i % 8;
                let hash_size = 4 << (group % 3);
                let size = get_shape_size(hash_size, 8);
                pool[i] = js_mallocz_rt(rt, size).cast();
                let p = pool[i];
                (*js_rc(p.cast())).ref_count = (i % 7 + 1) as i32;
                (*p).prop_hash_mask = (hash_size - 1) as u32;
                (*p).prop_size = 8;
                (*p).prop_count = n as i32;
                (*p).proto = (0x123456789abc0000u64 + group as u64 * 8) as usize as *mut JSObject;
                (*p).hash = shape_initial_hash((*p).proto);
                let pr = get_shape_prop(p);
                for j in 0..n {
                    (*pr.add(j)).atom = (group * 17 + j + 1) as u32;
                    (*pr.add(j)).set_flags(((group + j) % 64) as u32);
                    (*p).hash = shape_hash(
                        shape_hash((*p).hash, (*pr.add(j)).atom),
                        (*pr.add(j)).flags(),
                    );
                }
                num(&mut out, size as u64, 4);
                num(&mut out, (pr as usize - p as usize) as u64, 4);
                num(
                    &mut out,
                    (*js_rc(js_dup_shape(p).cast())).ref_count as u64,
                    4,
                );
            }
            h.fail = if fail > 1 { h.calls + fail - 1 } else { 0 };
            let mut rng = 0x123456789abcdef0u64;
            for step in 0..4096 {
                rng ^= rng << 13;
                rng ^= rng >> 7;
                rng ^= rng << 17;
                let i = ((rng >> 32) & 511) as usize;
                if linked[i] {
                    js_shape_hash_unlink(rt, pool[i]);
                } else {
                    js_shape_hash_link(rt, pool[i]);
                }
                linked[i] = !linked[i];
                if step % 37 == 0 {
                    num(
                        &mut out,
                        resize_shape_hash(rt, 1 + (step / 37) % 10) as u64,
                        4,
                    );
                }
                num(
                    &mut out,
                    shape_id(&pool, find_hashed_shape_proto(rt, (*pool[i]).proto)) as u64,
                    4,
                );
                let n = (*pool[i]).prop_count;
                let atom = (i / 8) as u32 * 17 + n as u32 + 1;
                let flags = ((i / 8) as i32 + n) % 64;
                num(
                    &mut out,
                    shape_id(&pool, find_hashed_shape_prop(rt, pool[i], atom, flags)) as u64,
                    4,
                );
                num(
                    &mut out,
                    shape_id(&pool, find_hashed_shape_prop(rt, pool[i], atom, flags + 64)) as u64,
                    4,
                );
                if step % 128 == 0 {
                    shape_snapshot(&mut out, rt, &h, &pool);
                }
            }
            for i in 0..512 {
                if linked[i] {
                    js_shape_hash_unlink(rt, pool[i]);
                }
                js_free_rt(rt, pool[i].cast());
            }
            shape_snapshot(&mut out, rt, &h, &pool);
            js_free_rt(rt, (*rt).shape_hash.cast());
            num(&mut out, h.live as u64, 4);
            num(&mut out, h.calls as u64, 4);
            num(&mut out, h.trace, 8);
        }
    }
    assert_eq!(out.len(), c.stdout.len());
    if let Some(i) = out.iter().zip(&c.stdout).position(|(r, c)| r != c) {
        panic!(
            "shape C/Rust mismatch at byte {i}: Rust={}, C={}",
            out[i], c.stdout[i]
        );
    }
    eprintln!(
        "quickjs.c shape hash parity: 65536 operations, 17 OOM scenarios, {} bytes",
        out.len()
    );
}
