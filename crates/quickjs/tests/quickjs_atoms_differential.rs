const ATOM_FUNCTIONS: &[&str] = &[
    "js_malloc_rt",
    "js_free_rt",
    "js_realloc_rt",
    "js_malloc_usable_size_rt",
    "js_mallocz_rt",
    "atom_get_free",
    "atom_is_free",
    "atom_set_free",
    "js_alloc_string_rt",
    "js_free_string",
    "__JS_AtomIsConst",
    "__JS_AtomIsTaggedInt",
    "__JS_AtomFromUInt32",
    "__JS_AtomToUInt32",
    "is_num",
    "string_get",
    "is_num_string",
    "hash_string8",
    "hash_string16",
    "hash_string",
    "hash_string_rope",
    "memcmp16_8",
    "memcmp16",
    "js_string_memcmp",
    "JS_ResizeAtomHash",
    "JS_InitAtoms",
    "JS_DupAtomRT",
    "JS_DupAtom",
    "JS_AtomGetKind",
    "JS_AtomIsString",
    "js_get_atom_index",
    "__JS_NewAtom",
    "__JS_NewAtomInit",
    "__JS_FindAtom",
    "JS_FreeAtomStruct",
    "__JS_FreeAtom",
    "JS_NewAtomStr",
    "JS_FreeAtom",
    "JS_FreeAtomRT",
    "JS_AtomSymbolHasDescription",
];
use super::super::cutils::dbuf_default_realloc;
use super::*;
#[path = "quickjs_test_source.rs"]
mod source;
struct Host {
    calls: i32,
    fail: i32,
    live: i32,
    trace: u64,
    allocs: std::collections::HashMap<usize, usize>,
}
fn event(h: &mut Host, op: u64, n: usize) {
    h.calls += 1;
    h.trace = (h.trace ^ op).wrapping_mul(1099511628211);
    h.trace = (h.trace ^ n as u64).wrapping_mul(1099511628211);
}
unsafe fn host_malloc(s: *mut JSMallocState, n: usize) -> *mut c_void {
    let h = &mut *(*s).opaque.cast::<Host>();
    event(h, 1, n);
    if h.calls == h.fail {
        return ptr::null_mut();
    }
    let p = dbuf_default_realloc(ptr::null_mut(), ptr::null_mut(), n);
    if !p.is_null() {
        ptr::write_bytes(p.cast::<u8>(), 0xa5, n);
        h.live += 1;
        h.allocs.insert(p as usize, n);
    }
    p
}
unsafe fn host_free(s: *mut JSMallocState, p: *mut c_void) {
    let h = &mut *(*s).opaque.cast::<Host>();
    let n = h.allocs.remove(&(p as usize)).unwrap_or(0);
    event(h, 2, n);
    if !p.is_null() {
        dbuf_default_realloc(ptr::null_mut(), p, 0);
        h.live -= 1;
    }
}
unsafe fn host_realloc(s: *mut JSMallocState, p: *mut c_void, n: usize) -> *mut c_void {
    let h = &mut *(*s).opaque.cast::<Host>();
    event(h, 3, n);
    if h.calls == h.fail {
        return ptr::null_mut();
    }
    let old = h.allocs.get(&(p as usize)).copied().unwrap_or(0);
    let q = dbuf_default_realloc(ptr::null_mut(), p, n);
    if !q.is_null() {
        if n > old {
            ptr::write_bytes(q.cast::<u8>().add(old), 0xa5, n - old);
        }
        if h.allocs.remove(&(p as usize)).is_none() {
            h.live += 1;
        }
        h.allocs.insert(q as usize, n);
    }
    q
}
fn num(out: &mut Vec<u8>, n: u64, size: usize) {
    out.extend_from_slice(&n.to_le_bytes()[..size]);
}
unsafe fn dump_string(out: &mut Vec<u8>, p: *mut JSString) {
    for n in [
        (*p).len(),
        (*p).is_wide_char(),
        (*p).hash(),
        (*p).atom_type(),
        (*p).hash_next,
        (*js_rc(p.cast())).ref_count as u32,
    ] {
        num(out, n as u64, 4);
    }
    for i in 0..(*p).len() {
        num(out, string_get(p, i as i32) as u64, 4);
    }
}
unsafe fn snapshot(out: &mut Vec<u8>, rt: *mut JSRuntime, h: &Host) {
    for n in [
        (*rt).atom_hash_size,
        (*rt).atom_count,
        (*rt).atom_size,
        (*rt).atom_count_resize,
        (*rt).atom_free_index,
        h.calls,
        h.live,
    ] {
        num(out, n as u64, 4);
    }
    num(out, h.trace, 8);
    for i in 0..(*rt).atom_hash_size {
        num(out, *(*rt).atom_hash.add(i as usize) as u64, 4);
    }
    for i in 0..(*rt).atom_size {
        let p = *(*rt).atom_array.add(i as usize);
        num(out, atom_is_free(p) as u64, 4);
        if atom_is_free(p) != 0 {
            num(out, atom_get_free(p) as u64, 4);
        } else {
            dump_string(out, p);
        }
    }
}
unsafe fn cleanup(out: &mut Vec<u8>, rt: *mut JSRuntime, h: &mut Host) {
    for i in 0..(*rt).atom_size {
        let p = *(*rt).atom_array.add(i as usize);
        if atom_is_free(p) == 0 {
            js_free_rt(rt, p.cast());
        }
    }
    js_free_rt(rt, (*rt).atom_array.cast());
    js_free_rt(rt, (*rt).atom_hash.cast());
    num(out, h.live as u64, 4);
    num(out, h.trace, 8);
    num(out, h.calls as u64, 4);
    for (p, _) in h.allocs.drain() {
        dbuf_default_realloc(ptr::null_mut(), p as *mut c_void, 0);
    }
}
unsafe fn initialize(rt: *mut JSRuntime, h: &mut Host) {
    js_malloc_init(&mut (*rt).malloc_ctx);
    (*rt).malloc_ctx.mf = JSMallocFunctions {
        js_malloc: Some(host_malloc),
        js_free: Some(host_free),
        js_realloc: Some(host_realloc),
        js_malloc_usable_size: None,
    };
    (*rt).malloc_ctx.malloc_state.opaque = (h as *mut Host).cast();
}
unsafe fn make_string(rt: *mut JSRuntime, units: &[u16], wide: u32) -> *mut JSString {
    let p = js_alloc_string_rt(rt, units.len() as i32, wide as i32);
    if !p.is_null() {
        for (i, &u) in units.iter().enumerate() {
            if wide != 0 {
                *string_data16(p).add(i) = u;
            } else {
                *string_data8(p).add(i) = u as u8;
            }
        }
        if wide == 0 {
            *string_data8(p).add(units.len()) = 0;
        }
    }
    p
}
fn host(fail: i32) -> Host {
    Host {
        calls: 0,
        fail,
        live: 0,
        trace: 1469598103934665603,
        allocs: Default::default(),
    }
}
#[test]
fn official_c_runtime_layouts_atom_storage_and_failure_states_match() {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let upstream = root.join("../../vendor/quickjs-2026-06-04");
    let c = std::fs::read_to_string(upstream.join("quickjs.c")).unwrap();
    let mut oracle = source::source_prefix(&c);
    source::append_functions(&mut oracle, &c, ATOM_FUNCTIONS);
    oracle.push_str(include_str!("quickjs_atoms_oracle.c"));
    let directory =
        std::env::temp_dir().join(format!("quickjs-atoms-oracle-{}", std::process::id()));
    std::fs::create_dir_all(&directory).unwrap();
    let source_path = directory.join("atoms.c");
    let executable = directory.join("atoms");
    std::fs::write(&source_path, oracle).unwrap();
    let result = include!("quickjs_oracle_config.rs")
        .args(["-std=c11", "-O2", "-I"])
        .arg(upstream)
        .arg(&source_path)
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
        "C oracle failed: {}",
        String::from_utf8_lossy(&c.stderr)
    );
    let mut out = Vec::new();
    macro_rules! layout{($($t:ty),*)=>{$(num(&mut out,size_of::<$t>() as u64,4);num(&mut out,core::mem::align_of::<$t>() as u64,4);)*};}
    layout!(
        JSRuntime,
        JSContext,
        JSClass,
        JSStackFrame,
        JSGCObjectHeader,
        JSWeakRefHeader,
        JSVarRef,
        JSBigInt,
        JSBigIntBuf,
        JSFloat64Union,
        JSString,
        JSStringRope
    );
    for n in [
        offset_of!(JSRuntime, rt_info),
        offset_of!(JSRuntime, atom_hash),
        offset_of!(JSRuntime, context_list),
        offset_of!(JSRuntime, malloc_gc_threshold),
        offset_of!(JSRuntime, current_exception),
        offset_of!(JSRuntime, current_stack_frame),
        offset_of!(JSRuntime, job_list),
        offset_of!(JSRuntime, u),
        offset_of!(JSRuntime, sab_funcs),
        offset_of!(JSRuntime, shape_hash),
        offset_of!(JSRuntime, user_opaque),
        offset_of!(JSContext, rt),
        offset_of!(JSContext, binary_object_size),
        offset_of!(JSContext, class_proto),
        offset_of!(JSContext, native_error_proto),
        offset_of!(JSContext, loaded_modules),
        offset_of!(JSContext, eval_internal),
        offset_of!(JSContext, user_opaque),
        offset_of!(JSVarRef, pvalue),
        offset_of!(JSVarRef, u),
        offset_of!(JSBigInt, tab),
        offset_of!(JSString, u),
        offset_of!(JSStringRope, left),
    ] {
        num(&mut out, n as u64, 4);
    }
    let vals = [0, 1, 0x7fffffff, 0x80000000, 0xffffffff];
    for i in 0..5 {
        for wide in 0..2 {
            for atom_type in 0..8 {
                let mut s: JSString = unsafe { core::mem::zeroed() };
                s.set_len(vals[i]);
                s.set_is_wide_char(wide);
                s.set_hash(vals[4 - i]);
                s.set_atom_type(atom_type);
                for n in [
                    s.len_and_wide,
                    s.hash_and_type,
                    s.len(),
                    s.is_wide_char(),
                    s.hash(),
                    s.atom_type(),
                ] {
                    num(&mut out, n as u64, 4);
                }
            }
        }
    }
    unsafe {
        for fail in 0..=64 {
            let mut rt: Box<JSRuntime> = Box::new(core::mem::zeroed());
            let rt = &mut *rt as *mut JSRuntime;
            let mut h = host(fail);
            initialize(rt, &mut h);
            num(&mut out, JS_InitAtoms(rt) as u64, 4);
            snapshot(&mut out, rt, &h);
            cleanup(&mut out, rt, &mut h);
        }
        for fail in 0..=16 {
            let mut rt: Box<JSRuntime> = Box::new(core::mem::zeroed());
            let rt = &mut *rt as *mut JSRuntime;
            let mut ctx: JSContext = core::mem::zeroed();
            ctx.rt = rt;
            let mut h = host(0);
            initialize(rt, &mut h);
            assert_eq!(JS_InitAtoms(rt), 0);
            h.fail = h.calls + fail;
            let mut ids = [0u32; 2048];
            let mut rng = 0x123456789abcdef0u64;
            for step in 0..6144 {
                rng ^= rng << 13;
                rng ^= rng >> 7;
                rng ^= rng << 17;
                let slot = ((rng >> 32) & 2047) as usize;
                let op = if step < 2048 { 0 } else { step % 7 };
                let mut id = 0;
                if op <= 2 {
                    let len = if op == 2 { 0 } else { 3 + (rng % 24) as usize };
                    let wide = ((rng >> 8) & 1) as u32;
                    let units: Vec<u16> = (0..len)
                        .map(|i| {
                            let x =
                                ((rng >> ((i % 8) * 8)).wrapping_add((step % 151) as u64)) as u16;
                            if wide != 0 {
                                x
                            } else {
                                x & 255
                            }
                        })
                        .collect();
                    let p = if op == 2 {
                        ptr::null_mut()
                    } else {
                        make_string(rt, &units, wide)
                    };
                    if op == 2 || !p.is_null() {
                        id = __JS_NewAtom(
                            rt,
                            p,
                            if op == 0 {
                                JS_ATOM_TYPE_STRING
                            } else if op == 1 {
                                JS_ATOM_TYPE_GLOBAL_SYMBOL
                            } else {
                                JS_ATOM_TYPE_SYMBOL
                            },
                        );
                    }
                    if ids[slot] != 0 {
                        JS_FreeAtomRT(rt, ids[slot]);
                    }
                    ids[slot] = id;
                } else if op == 3 {
                    if ids[slot] != 0 {
                        id = JS_DupAtom(&mut ctx, ids[slot]);
                    }
                    if id != 0 {
                        JS_FreeAtom(&mut ctx, id);
                    }
                } else if op == 4 {
                    let text = std::ffi::CString::new(format!("name-{}", step % 151)).unwrap();
                    id = __JS_NewAtomInit(
                        rt,
                        text.as_ptr(),
                        text.as_bytes().len() as i32,
                        JS_ATOM_TYPE_STRING,
                    );
                    if ids[slot] != 0 {
                        JS_FreeAtomRT(rt, ids[slot]);
                    }
                    ids[slot] = id;
                } else if op == 5 {
                    let text = std::ffi::CString::new(format!("name-{}", step % 151)).unwrap();
                    id = __JS_FindAtom(
                        rt,
                        text.as_ptr(),
                        text.as_bytes().len(),
                        JS_ATOM_TYPE_GLOBAL_SYMBOL,
                    );
                    if id != 0 {
                        JS_FreeAtomRT(rt, id);
                    }
                } else {
                    if ids[slot] != 0 {
                        JS_FreeAtomRT(rt, ids[slot]);
                    }
                    ids[slot] = 0;
                }
                num(&mut out, id as u64, 4);
                if id != 0 {
                    num(&mut out, JS_AtomGetKind(&mut ctx, id) as u64, 4);
                    num(&mut out, JS_AtomIsString(&mut ctx, id) as u64, 4);
                    num(
                        &mut out,
                        JS_AtomSymbolHasDescription(&mut ctx, id) as u64,
                        4,
                    );
                }
                if step % 128 == 0 {
                    snapshot(&mut out, rt, &h);
                }
            }
            for id in ids {
                if id != 0 {
                    JS_FreeAtomRT(rt, id);
                }
            }
            snapshot(&mut out, rt, &h);
            for atom_type in 1..=4 {
                let a = __JS_NewAtomInit(rt, b"shared\0".as_ptr().cast(), 6, JS_ATOM_TYPE_STRING);
                if a != 0 {
                    let p = *(*rt).atom_array.add(a as usize);
                    (*js_rc(p.cast())).ref_count += 1;
                    let b = __JS_NewAtom(rt, p, atom_type);
                    num(&mut out, b as u64, 4);
                    if b != 0 {
                        JS_FreeAtomRT(rt, b);
                    }
                    JS_FreeAtomRT(rt, a);
                }
            }
            snapshot(&mut out, rt, &h);
            cleanup(&mut out, rt, &mut h);
        }
        let mut rt: Box<JSRuntime> = Box::new(core::mem::zeroed());
        let rt = &mut *rt as *mut JSRuntime;
        let mut h = host(0);
        initialize(rt, &mut h);
        assert_eq!(JS_InitAtoms(rt), 0);
        let mut ctx: JSContext = core::mem::zeroed();
        ctx.rt = rt;
        for text in [
            "",
            "0",
            "00",
            "1",
            "012",
            "2147483647",
            "2147483648",
            "4294967295",
            "4294967296",
            "9999999999",
            "12345678901",
            "-1",
            "+1",
            "1.0",
            "1e2",
            " 1",
            "1 ",
            "NaN",
        ] {
            for wide in 0..2 {
                let units: Vec<u16> = text.bytes().map(|b| b as u16).collect();
                let p = make_string(rt, &units, wide);
                let mut n = 0xa5a5a5a5;
                num(&mut out, is_num_string(&mut n, p) as u64, 4);
                num(&mut out, n as u64, 4);
                num(&mut out, hash_string(p, 1) as u64, 4);
                let a = JS_NewAtomStr(&mut ctx, p);
                num(&mut out, a as u64, 4);
                JS_FreeAtomRT(rt, a);
            }
        }
        cleanup(&mut out, rt, &mut h);
    }
    assert_eq!(out.len(), c.stdout.len());
    if let Some(i) = out.iter().zip(&c.stdout).position(|(r, c)| r != c) {
        panic!(
            "atom C/Rust mismatch at byte {i}: Rust={}, C={}",
            out[i], c.stdout[i]
        );
    }
    eprintln!(
        "quickjs.c runtime/atom C parity: 104448 atom operations, {} bytes",
        out.len()
    );
}

include!("quickjs_classes_differential.rs");
include!("quickjs_strings_differential.rs");
include!("quickjs_runtime_controls_differential.rs");
include!("quickjs_shapes_differential.rs");
include!("quickjs_objects_differential.rs");
include!("quickjs_gc_differential.rs");

include!("quickjs_weakrefs_differential.rs");

include!("quickjs_gc_marks_differential.rs");

include!("quickjs_conversions_differential.rs");

include!("quickjs_class_lifecycle_differential.rs");

include!("quickjs_runtime_free_differential.rs");
include!("quickjs_context_strings_differential.rs");
include!("quickjs_debug_differential.rs");
include!("quickjs_properties_differential.rs");

include!("quickjs_parser_differential.rs");

include!("quickjs_runtime_init_functions.rs");
