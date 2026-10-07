use super::super::cutils::dbuf_default_realloc;
use super::*;
struct Host {
    calls: i32,
    fail: i32,
    live: i32,
    trace: u64,
    allocs: std::collections::HashMap<usize, usize>,
}
fn event(h: &mut Host, op: u64, n: usize) {
    h.trace = (h.trace ^ op).wrapping_mul(1099511628211);
    h.trace = (h.trace ^ n as u64).wrapping_mul(1099511628211);
    h.calls += 1;
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
        h.live -= 1;
        dbuf_default_realloc(ptr::null_mut(), p, 0);
    }
}
unsafe fn host_realloc(s: *mut JSMallocState, p: *mut c_void, n: usize) -> *mut c_void {
    let h = &mut *(*s).opaque.cast::<Host>();
    event(h, 3, n);
    if h.calls == h.fail {
        return ptr::null_mut();
    }
    let old = h.allocs[&(p as usize)];
    let q = dbuf_default_realloc(ptr::null_mut(), p, n);
    if !q.is_null() {
        if n > old {
            ptr::write_bytes(q.cast::<u8>().add(old), 0xa5, n - old);
        }
        h.allocs.remove(&(p as usize));
        h.allocs.insert(q as usize, n);
    }
    q
}
// Default allocator's hidden header contains its exact requested size. Use a
// test-local callback and explicit map lookup through a separate injected
// thread-local pointer, since the C usable-size callback takes no opaque.
thread_local! {static ALLOCS:core::cell::Cell<*const Host>=const{core::cell::Cell::new(ptr::null())};}
unsafe fn host_usable_size(p: *const c_void) -> usize {
    ALLOCS.with(|cell| (&(*cell.get()).allocs)[&(p as usize)])
}
fn num(out: &mut Vec<u8>, n: u64, size: usize) {
    out.extend_from_slice(&n.to_le_bytes()[..size]);
}
unsafe fn snapshot(out: &mut Vec<u8>, s: *mut JSMallocContext, p: *mut c_void, h: &Host) {
    num(out, (!p.is_null()) as u64, 4);
    if !p.is_null() {
        let b = js_rc(p);
        let n = __js_malloc_usable_size(s, p.cast());
        num(out, n as u64, 8);
        num(out, (*b).u.block_idx as u64, 4);
        num(out, (*b).block_size_idx as u64, 4);
        if n != 0 {
            num(out, ((*b).gc_obj_type_and_mark & 127) as u64, 4);
            num(out, ((*b).gc_obj_type_and_mark >> 7) as u64, 4);
            num(out, (*b).ref_count as u64, 4);
            let mut hash = 1469598103934665603u64;
            for i in 0..n {
                hash = (hash ^ *p.cast::<u8>().add(i) as u64).wrapping_mul(1099511628211);
            }
            num(out, hash, 8);
        }
    }
    num(out, h.calls as u64, 4);
    num(out, h.live as u64, 4);
    num(out, h.trace, 8);
    for i in 0..JS_MALLOC_BLOCK_SIZE_COUNT {
        let head = ptr::addr_of_mut!((*s).arena_list[i]);
        let mut el = (*head).next;
        let mut n = 0;
        while el != head {
            n += 1;
            el = (*el).next;
        }
        num(out, n, 4);
        el = (*head).next;
        while el != head {
            let ar = el
                .cast::<u8>()
                .sub(offset_of!(JSMallocArena, link))
                .cast::<JSMallocArena>();
            num(out, (*ar).n_used_blocks as u64, 4);
            num(out, (*ar).n_blocks as u64, 4);
            num(out, (*ar).first_free_block as u64, 4);
            el = (*el).next;
        }
        let head = ptr::addr_of_mut!((*s).free_arena_list[i]);
        let mut el = (*head).next;
        let mut n = 0;
        while el != head {
            n += 1;
            el = (*el).next;
        }
        num(out, n, 4);
    }
}
#[test]
fn official_c_arena_allocator_layout_sequences_and_failures_match() {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let upstream = root.join("../../vendor/quickjs-2026-06-04");
    let c = std::fs::read_to_string(upstream.join("quickjs.c")).unwrap();
    let structure = c
        .split_once("#define JS_MALLOC_ALIGN")
        .unwrap()
        .1
        .split_once("/* end JS Malloc */")
        .unwrap()
        .0;
    let implementation = c
        .split_once("static const uint16_t js_malloc_block_sizes")
        .unwrap()
        .1
        .split_once("static __maybe_unused void js_malloc_dump_arenas")
        .unwrap()
        .0;
    let directory =
        std::env::temp_dir().join(format!("quickjs-malloc-oracle-{}", std::process::id()));
    std::fs::create_dir_all(&directory).unwrap();
    let source = directory.join("allocator.c");
    let file = directory.join("allocator");
    std::fs::write(&source,format!("#include <stdlib.h>\n#include <stdio.h>\n#include <math.h>\n#include <stddef.h>\n#include <assert.h>\n#include \"cutils.h\"\n#include \"list.h\"\n#include \"quickjs.h\"\n#define JS_MALLOC_ALIGN{structure}\nstatic const uint16_t js_malloc_block_sizes{implementation}\n{}",include_str!("quickjs_malloc_oracle.c"))).unwrap();
    let result = include!("quickjs_oracle_config.rs")
        .args([
            "-std=c11",
            "-O2",
            if cfg!(feature = "malloc-iter") {
                "-DJS_MALLOC_USE_ITER=1"
            } else {
                "-UJS_MALLOC_USE_ITER"
            },
            "-I",
        ])
        .arg(upstream)
        .arg(&source)
        .arg("-o")
        .arg(&file)
        .output()
        .unwrap();
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    let c = std::process::Command::new(&file).output().unwrap();
    let _ = std::fs::remove_dir_all(directory);
    assert!(
        c.status.success(),
        "C oracle failed {}",
        String::from_utf8_lossy(&c.stderr)
    );
    let mut out = Vec::new();
    macro_rules! layout{($($t:ty),*)=>{$(num(&mut out,size_of::<$t>() as u64,4);num(&mut out,core::mem::align_of::<$t>() as u64,4);)*};}
    layout!(
        JSMallocBlockHeader,
        JSMallocLargeBlockHeader,
        JSMallocArena,
        JSMallocContext
    );
    for n in [
        offset_of!(JSMallocBlockHeader, user_data),
        offset_of!(JSMallocBlockHeader, ref_count),
        offset_of!(JSMallocArena, blocks),
        offset_of!(JSMallocArena, n_used_blocks),
        offset_of!(JSMallocContext, zero_size_block),
        offset_of!(JSMallocContext, mf),
        offset_of!(JSMallocContext, malloc_state),
    ] {
        num(&mut out, n as u64, 4);
    }
    for n in js_malloc_block_sizes {
        num(&mut out, n as u64, 4);
    }
    for i in 0..=4096 {
        num(&mut out, get_block_size_index(i) as u64, 4);
    }
    unsafe {
        for fail in 0..=32 {
            let mut state: Box<JSMallocContext> = Box::new(core::mem::zeroed());
            let s = &mut *state as *mut JSMallocContext;
            js_malloc_init(s);
            let mut h = Host {
                calls: 0,
                fail,
                live: 0,
                trace: 1469598103934665603,
                allocs: Default::default(),
            };
            ALLOCS.with(|cell| cell.set(&h));
            (*s).malloc_state.opaque = (&mut h as *mut Host).cast();
            (*s).mf = JSMallocFunctions {
                js_malloc: Some(host_malloc),
                js_free: Some(host_free),
                js_realloc: Some(host_realloc),
                js_malloc_usable_size: Some(host_usable_size),
            };
            let mut blocks = [ptr::null_mut(); 256];
            let mut rng = 0x123456789abcdef0u64;
            for step in 0..4096 {
                rng ^= rng << 13;
                rng ^= rng >> 7;
                rng ^= rng << 17;
                let slot = ((rng >> 32) & 255) as usize;
                let n = if step < 768 {
                    step % 768
                } else {
                    ((rng >> 8) % 8192) as usize
                };
                let mut p = blocks[slot];
                if step % 5 == 0 {
                    __js_free(s, p);
                    blocks[slot] = ptr::null_mut();
                    p = ptr::null_mut();
                } else {
                    let q = __js_realloc(s, p, n);
                    snapshot(&mut out, s, q, &h);
                    if !q.is_null() || n == 0 {
                        blocks[slot] = q;
                        p = q;
                    }
                }
                if !p.is_null() {
                    let size = __js_malloc_usable_size(s, p.cast());
                    if size != 0 {
                        let b = js_rc(p);
                        (*b).gc_obj_type_and_mark =
                            (step & 127) as u8 | (((step >> 7) & 1) as u8) << 7;
                        (*b).ref_count = step as i32 + 1;
                        for i in 0..size {
                            *p.cast::<u8>().add(i) = (step + i) as u8;
                        }
                    }
                }
                snapshot(&mut out, s, p, &h);
            }
            for p in blocks {
                __js_free(s, p);
            }
            snapshot(&mut out, s, ptr::null_mut(), &h);
            assert_eq!(h.live, 0);
            let z = __js_malloc(s, 0);
            snapshot(&mut out, s, z, &h);
            let p = __js_realloc(s, z, 1);
            snapshot(&mut out, s, p, &h);
            __js_free(s, p);
            __js_free(s, z);
            snapshot(&mut out, s, ptr::null_mut(), &h);
            let p = __js_malloc(s, 1024);
            (*s).mf.js_malloc_usable_size = None;
            num(&mut out, __js_malloc_usable_size(s, p.cast()) as u64, 8);
            __js_free(s, p);
            num(&mut out, __js_malloc_usable_size(s, ptr::null()) as u64, 8);
            assert!(h.allocs.is_empty());
            ALLOCS.with(|cell| cell.set(ptr::null()));
        }
    }
    assert_eq!(out.len(), c.stdout.len());
    if let Some(i) = out.iter().zip(&c.stdout).position(|(r, c)| r != c) {
        panic!(
            "arena allocator C/Rust mismatch at byte {i}: Rust={}, C={}",
            out[i], c.stdout[i]
        );
    }
    eprintln!(
        "quickjs.c arena allocator C parity: 135168 operations, {} bytes",
        out.len()
    );
}
