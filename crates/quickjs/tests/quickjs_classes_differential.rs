// Included in the atom differential module to share its instrumented allocator.
unsafe fn test_finalizer(_rt: *mut JSRuntime, _val: JSValue) {}
unsafe fn test_gc_mark(_rt: *mut JSRuntime, _val: JSValueConst, _f: Option<JS_MarkFunc>) {}
unsafe fn test_call(
    _ctx: *mut JSContext,
    _v: JSValueConst,
    _t: JSValueConst,
    _argc: i32,
    _argv: *mut JSValueConst,
    _flags: i32,
) -> JSValue {
    JS_UNDEFINED
}
fn hash_num(h: u64, n: u64) -> u64 {
    (h ^ n).wrapping_mul(1099511628211)
}
unsafe fn class_snapshot(out: &mut Vec<u8>, rt: *mut JSRuntime, ctx: &[JSContext], h: &Host) {
    num(out, (*rt).class_count as u64, 4);
    let mut hash = 1469598103934665603;
    for i in 0..(*rt).class_count {
        let c = &*(*rt).class_array.add(i as usize);
        for n in [
            c.class_id as u64,
            c.class_name as u64,
            c.finalizer.is_some() as u64,
            c.gc_mark.is_some() as u64,
            c.call.is_some() as u64,
            !c.exotic.is_null() as u64,
        ] {
            hash = hash_num(hash, n);
        }
    }
    num(out, hash, 8);
    for c in ctx {
        let n = h
            .allocs
            .get(&(c.class_proto as usize).wrapping_sub(size_of::<JSMallocLargeBlockHeader>()))
            .map_or(0, |size| {
                (size - size_of::<JSMallocLargeBlockHeader>()) / size_of::<JSValue>()
            });
        num(out, n as u64, 4);
        let mut hash = 1469598103934665603;
        for i in 0..n {
            let v = *c.class_proto.add(i);
            hash = hash_num(hash, JS_VALUE_GET_TAG(v) as u64);
            hash = hash_num(hash, v.u.uint64);
        }
        num(out, hash, 8);
    }
    num(out, h.calls as u64, 4);
    num(out, h.live as u64, 4);
    num(out, h.trace, 8);
}
#[repr(C)]
struct TestGC {
    h: JSGCObjectHeader,
    id: u32,
}
unsafe fn test_mark(rt: *mut JSRuntime, p: *mut JSGCObjectHeader) {
    let out = &mut *(*rt).user_opaque.cast::<Vec<u8>>();
    num(out, (*p.cast::<TestGC>()).id as u64, 4);
}
#[test]
fn official_c_class_registration_oom_and_gc_helpers_match() {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let upstream = root.join("../../vendor/quickjs-2026-06-04");
    let c = std::fs::read_to_string(upstream.join("quickjs.c")).unwrap();
    let mut oracle = source::source_prefix(&c);
    source::append_functions(&mut oracle, &c, ATOM_FUNCTIONS);
    oracle.push_str("static JSClassID js_class_id_alloc = JS_CLASS_INIT_COUNT;\n");
    oracle.push_str("#ifdef CONFIG_ATOMICS\n#include <pthread.h>\nstatic pthread_mutex_t js_class_id_mutex = PTHREAD_MUTEX_INITIALIZER;\n#endif\n");
    source::append_functions(
        &mut oracle,
        &c,
        &[
            "JS_NewClassID",
            "JS_IsRegisteredClass",
            "JS_NewClass1",
            "JS_NewClass",
            "add_gc_object",
            "remove_gc_object",
            "JS_MarkValue",
        ],
    );
    oracle.push_str(
        include_str!("quickjs_atoms_oracle.c")
            .split("int main(void){")
            .next()
            .unwrap(),
    );
    oracle.push_str(include_str!("quickjs_classes_oracle.c"));
    let directory =
        std::env::temp_dir().join(format!("quickjs-classes-oracle-{}", std::process::id()));
    std::fs::create_dir_all(&directory).unwrap();
    let path = directory.join("classes.c");
    let executable = directory.join("classes");
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
        "C classes oracle failed: {}",
        String::from_utf8_lossy(&c.stderr)
    );
    let mut out = Vec::new();
    unsafe {
        let mut ids = [0; 256];
        for id in &mut ids {
            num(&mut out, JS_NewClassID(id) as u64, 4);
            num(&mut out, JS_NewClassID(id) as u64, 4);
        }
        let mut custom = 0x87654321;
        num(&mut out, JS_NewClassID(&mut custom) as u64, 4);
        num(&mut out, custom as u64, 4);
        let classes = [
            0,
            1,
            JS_CLASS_INIT_COUNT - 1,
            JS_CLASS_INIT_COUNT,
            127,
            128,
            191,
            255,
            256,
            511,
            1023,
            2047,
            65535,
            65536,
            u32::MAX,
            1,
            255,
            0,
        ];
        for contexts in 0..=3 {
            for fail in 0..=40 {
                let mut rt: Box<JSRuntime> = Box::new(core::mem::zeroed());
                let rt = &mut *rt as *mut JSRuntime;
                let mut ctx: Vec<JSContext> = (0..contexts).map(|_| core::mem::zeroed()).collect();
                let mut h = host(0);
                initialize(rt, &mut h);
                init_list_head(&mut (*rt).context_list);
                assert_eq!(JS_InitAtoms(rt), 0);
                for c in &mut ctx {
                    c.rt = rt;
                    list_add_tail(&mut c.link, &mut (*rt).context_list);
                }
                h.fail = h.calls + fail;
                let mut exotic: JSClassExoticMethods = core::mem::zeroed();
                for step in 0..18 {
                    let name = std::ffi::CString::new(if step % 3 == 0 {
                        "Object".to_owned()
                    } else {
                        format!("test-{}", step % 7)
                    })
                    .unwrap();
                    let def = JSClassDef {
                        class_name: name.as_ptr(),
                        finalizer: if step & 1 != 0 {
                            Some(test_finalizer)
                        } else {
                            None
                        },
                        gc_mark: if step & 2 != 0 {
                            Some(test_gc_mark)
                        } else {
                            None
                        },
                        call: if step & 4 != 0 { Some(test_call) } else { None },
                        exotic: if step & 8 != 0 {
                            &mut exotic
                        } else {
                            ptr::null_mut()
                        },
                    };
                    num(&mut out, JS_NewClass(rt, classes[step], &def) as u64, 4);
                    num(&mut out, JS_IsRegisteredClass(rt, classes[step]) as u64, 4);
                    class_snapshot(&mut out, rt, &ctx, &h);
                    for (k, c) in ctx.iter_mut().enumerate() {
                        if !c.class_proto.is_null() && (*rt).class_count != 0 {
                            *c.class_proto = JS_NewInt32(c, (step * 7 + k) as i32);
                        }
                    }
                }
                for i in 0..(*rt).class_count {
                    let c = &*(*rt).class_array.add(i as usize);
                    if c.class_id != 0 {
                        JS_FreeAtomRT(rt, c.class_name);
                    }
                }
                js_free_rt(rt, (*rt).class_array.cast());
                for c in &mut ctx {
                    js_free_rt(rt, c.class_proto.cast());
                    list_del(&mut c.link);
                }
                snapshot(&mut out, rt, &h);
                cleanup(&mut out, rt, &mut h);
            }
        }
        let mut rt: Box<JSRuntime> = Box::new(core::mem::zeroed());
        let rt = &mut *rt as *mut JSRuntime;
        let mut h = host(0);
        initialize(rt, &mut h);
        init_list_head(&mut (*rt).gc_obj_list);
        (*rt).user_opaque = ptr::from_mut(&mut out).cast();
        let mut g = [ptr::null_mut::<TestGC>(); 64];
        for i in 0..64 {
            g[i] = js_mallocz_rt(rt, size_of::<TestGC>()).cast();
            (*g[i]).id = i as u32;
            (*js_rc(g[i].cast())).ref_count = i as i32 + 1;
            (*js_rc(g[i].cast())).gc_obj_type_and_mark = 99 | 128;
            add_gc_object(rt, &mut (*g[i]).h, (i % 7) as u32);
            let rc = &*js_rc(g[i].cast());
            num(&mut out, (rc.gc_obj_type_and_mark & 127) as u64, 4);
            num(&mut out, (rc.gc_obj_type_and_mark >> 7) as u64, 4);
            num(&mut out, rc.ref_count as u64, 4);
        }
        for el in ListIter::new(&mut (*rt).gc_obj_list, false, false) {
            num(
                &mut out,
                (*el.cast::<u8>()
                    .sub(offset_of!(TestGC, h) + offset_of!(JSGCObjectHeader, link))
                    .cast::<TestGC>())
                .id as u64,
                4,
            );
        }
        for i in (0..64).step_by(2) {
            remove_gc_object(&mut (*g[i]).h);
        }
        for el in ListIter::new(&mut (*rt).gc_obj_list, false, false) {
            num(
                &mut out,
                (*el.cast::<u8>()
                    .sub(offset_of!(TestGC, h) + offset_of!(JSGCObjectHeader, link))
                    .cast::<TestGC>())
                .id as u64,
                4,
            );
        }
        for tag in -12..=12 {
            for p in g {
                JS_MarkValue(rt, JS_MKPTR(tag, p.cast()), test_mark);
            }
        }
        for i in (1..64).step_by(2) {
            remove_gc_object(&mut (*g[i]).h);
        }
        num(&mut out, list_empty(&mut (*rt).gc_obj_list) as u64, 4);
        for p in g {
            js_free_rt(rt, p.cast());
        }
        num(&mut out, h.live as u64, 4);
        num(&mut out, h.trace, 8);
    }
    assert_eq!(out.len(), c.stdout.len());
    if let Some(i) = out.iter().zip(&c.stdout).position(|(r, c)| r != c) {
        panic!(
            "class C/Rust mismatch at byte {i}: Rust={}, C={}",
            out[i], c.stdout[i]
        );
    }
    eprintln!(
        "quickjs.c class/GC parity: 2952 registrations, 164 fault scenarios, {} bytes",
        out.len()
    );
}
