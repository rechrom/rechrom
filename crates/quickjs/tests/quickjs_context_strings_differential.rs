// Tests staged source with explicit dependency boundaries. Error construction,
// ToPrimitive/ToString of non-string values and nonempty weak-GC lists are excluded;
// these test-only callbacks cannot be used by the production crate.
mod context_strings {
    use super::*;
    include!("../src/quickjs_context_alloc.rs");
    include!("../src/quickjs_strings.rs");
    include!("../src/quickjs_string_conversions.rs");
    include!("../src/quickjs_atom_strings.rs");
    include!("../src/quickjs_shape_construction.rs");
    include!("../src/quickjs_object_construction.rs");

    #[repr(C)]
    #[derive(Default)]
    struct BoundaryLog {
        oom: u32,
        internal: u32,
        type_error: u32,
    }
    unsafe fn JS_ThrowOutOfMemory(ctx: *mut JSContext) -> JSValue {
        (*(*ctx).user_opaque.cast::<BoundaryLog>()).oom += 1;
        JS_EXCEPTION
    }
    unsafe fn JS_ThrowInternalError(ctx: *mut JSContext, message: *const c_char) -> JSValue {
        assert_eq!(
            core::ffi::CStr::from_ptr(message).to_bytes(),
            b"string too long"
        );
        (*(*ctx).user_opaque.cast::<BoundaryLog>()).internal += 1;
        JS_EXCEPTION
    }
    unsafe fn JS_ThrowTypeError(ctx: *mut JSContext, message: *const c_char) -> JSValue {
        assert_eq!(
            core::ffi::CStr::from_ptr(message).to_bytes(),
            b"invalid object type"
        );
        (*(*ctx).user_opaque.cast::<BoundaryLog>()).type_error += 1;
        JS_EXCEPTION
    }
    unsafe fn JS_ToString(ctx: *mut JSContext, val: JSValueConst) -> JSValue {
        assert_eq!(
            JS_VALUE_GET_TAG(val),
            JS_TAG_STRING,
            "non-string conversion is outside this isolated oracle"
        );
        JS_DupValue(ctx, val)
    }
    unsafe fn JS_ToStringFree(_ctx: *mut JSContext, val: JSValue) -> JSValue {
        assert_eq!(
            JS_VALUE_GET_TAG(val),
            JS_TAG_STRING,
            "non-string conversion is outside this isolated oracle"
        );
        val
    }
    unsafe fn JS_RunGC(rt: *mut JSRuntime) {
        // The C extraction includes real cycle-GC bodies. Its boundary admits
        // an empty weak list only; use the production Rust collector here.
        assert_ne!(list_empty(&mut (*rt).weakref_list), 0);
        super::JS_RunGC(rt);
    }
    unsafe fn init(rt: *mut JSRuntime, ctx: *mut JSContext, h: &mut Host, log: &mut BoundaryLog) {
        initialize(rt, h);
        init_list_head(&mut (*rt).context_list);
        init_list_head(&mut (*rt).gc_obj_list);
        init_list_head(&mut (*rt).gc_zero_ref_count_list);
        init_list_head(&mut (*rt).weakref_list);
        (*rt).malloc_gc_threshold = usize::MAX;
        assert_eq!(JS_InitAtoms(rt), 0);
        assert_eq!(init_shape_hash(rt), 0);
        (*ctx).rt = rt;
        (*ctx).user_opaque = ptr::from_mut(log).cast();
    }
    unsafe fn finish(out: &mut Vec<u8>, rt: *mut JSRuntime, h: &mut Host) {
        assert_eq!((*rt).shape_hash_count, 0);
        assert_ne!(list_empty(&mut (*rt).gc_obj_list), 0);
        js_free_rt(rt, (*rt).shape_hash.cast());
        cleanup(out, rt, h);
        assert_eq!(h.live, 0);
    }
    unsafe fn value(out: &mut Vec<u8>, val: JSValue) {
        num(out, JS_VALUE_GET_TAG(val) as u64, 4);
        if JS_IsException(val) == 0 {
            dump_string(out, JS_VALUE_GET_PTR(val).cast());
        }
    }
    fn effects(out: &mut Vec<u8>, h: &Host, log: &BoundaryLog) {
        for n in [
            log.oom,
            log.internal,
            log.type_error,
            h.calls as u32,
            h.live as u32,
        ] {
            num(out, n as u64, 4);
        }
        num(out, h.trace, 8);
    }
    unsafe fn encoded(out: &mut Vec<u8>, ctx: *mut JSContext, v: JSValue, cesu8: i32) {
        let mut len = 0xa5a5usize;
        let p = JS_ToCStringLen2(ctx, &mut len, v, cesu8);
        num(out, p.is_null() as u64, 4);
        num(out, len as u64, 4);
        if !p.is_null() {
            out.extend_from_slice(core::slice::from_raw_parts(p.cast::<u8>(), len + 1));
        }
        // Observe the retained reference before releasing the C string.
        num(out, (*js_rc(JS_VALUE_GET_PTR(v))).ref_count as u64, 4);
        JS_FreeCString(ctx, p);
    }
    fn translated_functions() -> Vec<String> {
        let mut names = Vec::new();
        for file in [
            include_str!("../src/quickjs_context_alloc.rs"),
            include_str!("../src/quickjs_strings.rs"),
            include_str!("../src/quickjs_string_conversions.rs"),
            include_str!("../src/quickjs_atom_strings.rs"),
            include_str!("../src/quickjs_shape_construction.rs"),
            include_str!("../src/quickjs_object_construction.rs"),
        ] {
            for line in file.lines() {
                if let Some((_, after)) = line.split_once("fn ") {
                    let name = after.split_once('(').unwrap().0.trim();
                    // JS_NewString is an inline quickjs.h wrapper, already
                    // provided by the oracle header rather than quickjs.c.
                    if name != "JS_NewString"
                        && !ATOM_FUNCTIONS.contains(&name)
                        && !GC_FUNCTIONS.contains(&name)
                    {
                        names.push(name.to_owned());
                    }
                }
            }
        }
        names
    }
    #[test]
    fn official_c_staged_strings_atoms_shapes_and_object_construction_match() {
        let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let upstream = root.join("../../vendor/quickjs-2026-06-04");
        let c = std::fs::read_to_string(upstream.join("quickjs.c")).unwrap();
        let mut oracle = weak_source(&c);
        oracle.push_str("#include \"dtoa.h\"\n#define JS_STRING_LEN_MAX ((1 << 30) - 1)\n#define ATOM_GET_STR_BUF_SIZE 64\n");
        let a = c.find("typedef struct StringBuffer {").unwrap();
        let b = a + c[a..].find("} StringBuffer;").unwrap() + "} StringBuffer;".len();
        oracle.push_str(&c[a..b]);
        oracle.push_str(include_str!("quickjs_context_strings_boundaries.c"));
        let mut names = translated_functions();
        names.extend(
            [
                "count_ascii",
                "is_digit",
                "init_shape_hash",
                "resize_shape_hash",
                "shape_initial_hash",
                "js_shape_hash_link",
                "js_dup_shape",
                "find_hashed_shape_proto",
                "find_hashed_shape_prop",
                "find_own_property",
                "find_own_property1",
            ]
            .map(str::to_owned),
        );
        let names: Vec<_> = names.iter().map(String::as_str).collect();
        source::append_functions(&mut oracle, &c, &names);
        oracle.push_str(
            include_str!("quickjs_atoms_oracle.c")
                .split("int main(void){")
                .next()
                .unwrap(),
        );
        oracle.push_str(include_str!("quickjs_context_strings_oracle.c"));
        let directory = std::env::temp_dir().join(format!(
            "quickjs-context-strings-oracle-{}",
            std::process::id()
        ));
        std::fs::create_dir_all(&directory).unwrap();
        let path = directory.join("strings.c");
        let executable = directory.join("strings");
        std::fs::write(&path, oracle).unwrap();
        let result = include!("quickjs_oracle_config.rs")
            .args(["-std=c11", "-O2", "-I"])
            .arg(&upstream)
            .arg(&path)
            .arg(upstream.join("cutils.c"))
            .arg(upstream.join("dtoa.c"))
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
        assert!(c.status.success(), "{}", String::from_utf8_lossy(&c.stderr));
        let mut out = Vec::new();
        unsafe {
            fixtures(&mut out);
        }
        assert_eq!(out.len(), c.stdout.len(), "Rust/C output sizes");
        if let Some(index) = out.iter().zip(&c.stdout).position(|(a, b)| a != b) {
            panic!(
                "staged strings differ at byte {index}: Rust {}, C {}",
                out[index], c.stdout[index]
            );
        }
        eprintln!("staged strings/atoms/shapes/objects: {} matching bytes; production error, non-string conversion and weak-GC boundaries remain pending", out.len());
    }
    unsafe fn fixtures(out: &mut Vec<u8>) {
        // Filled with deterministic paired C/Rust fixtures below.
        string_fixtures(out);
        shape_fixtures(out);
    }
    unsafe fn string_fixtures(out: &mut Vec<u8>) {
        let mut rt: Box<JSRuntime> = Box::new(core::mem::zeroed());
        let rt = &mut *rt as *mut JSRuntime;
        let mut ctx: JSContext = core::mem::zeroed();
        let ctx = &mut ctx as *mut JSContext;
        let mut h = host(0);
        let mut log = BoundaryLog::default();
        init(rt, ctx, &mut h, &mut log);
        for bytes in 0..65536u32 {
            let buf = [bytes as u8, (bytes >> 8) as u8];
            let v = JS_NewStringLen(ctx, buf.as_ptr().cast(), buf.len());
            value(out, v);
            encoded(out, ctx, v, 0);
            encoded(out, ctx, v, 1);
            JS_FreeValue(ctx, v);
        }
        let mut rng = 0x123456789abcdef0u64;
        for i in 0..8192 {
            let mut text = [0u8; 128];
            let len = i % 65;
            for c in &mut text[..len] {
                *c = weak_rng(&mut rng) as u8;
            }
            let v = JS_NewStringLen(ctx, text.as_ptr().cast(), len);
            value(out, v);
            encoded(out, ctx, v, (i % 2) as i32);
            let p = JS_VALUE_GET_PTR(v).cast::<JSString>();
            let mut index = 0;
            while (index as u32) < (*p).len() {
                num(out, string_getc(p, &mut index) as u64, 4);
                num(out, index as u64, 4);
            }
            let length = (*p).len() as i32;
            for (start, end) in [(0, length), (0, 0), (length / 3, length * 2 / 3)] {
                let sub = js_sub_string(ctx, p, start, end);
                value(out, sub);
                JS_FreeValue(ctx, sub);
            }
            let atom = JS_NewAtomLen(ctx, text.as_ptr().cast(), len);
            num(out, atom as u64, 4);
            let s = JS_AtomToString(ctx, atom);
            value(out, s);
            JS_FreeValue(ctx, s);
            JS_FreeAtom(ctx, atom);
            JS_FreeValue(ctx, v);
            if i % 1024 == 0 {
                effects(out, &h, &log);
            }
        }
        for i in 0..8192 {
            let mut units = [0u16; 64];
            let len = i % 65;
            for c in &mut units[..len] {
                *c = weak_rng(&mut rng) as u16;
            }
            // Exercise paired/unpaired surrogates and wide-to-Latin1 substrings.
            if len >= 8 {
                units[..8].copy_from_slice(&[0xd800, 0xdc00, 0xd800, 0x61, 0xdc00, 0xff, 0, 0x80]);
            }
            let p = make_string(rt, &units[..len], (i % 2) as u32);
            let v = JS_MKPTR(JS_TAG_STRING, p.cast());
            encoded(out, ctx, v, 0);
            encoded(out, ctx, v, 1);
            let mut index = 0;
            while (index as usize) < len {
                num(out, string_getc(p, &mut index) as u64, 4);
                num(out, index as u64, 4);
            }
            for (start, end) in [(0, len), (0, 0), (len / 3, len * 2 / 3)] {
                let sub = js_sub_string(ctx, p, start as i32, end as i32);
                value(out, sub);
                JS_FreeValue(ctx, sub);
            }
            let mut b: StringBuffer = core::mem::zeroed();
            assert_eq!(string_buffer_init(ctx, &mut b, 1), 0);
            string_buffer_concat_value(&mut b, v);
            string_buffer_concat_value_free(&mut b, JS_DupValue(ctx, v));
            let result = string_buffer_end(&mut b);
            value(out, result);
            JS_FreeValue(ctx, result);
            let result =
                JS_ConcatString3(ctx, c"pre".as_ptr(), JS_DupValue(ctx, v), c"post".as_ptr());
            value(out, result);
            JS_FreeValue(ctx, result);
            JS_FreeValue(ctx, v);
            let c = js_new_string_char(ctx, i as u16);
            value(out, c);
            JS_FreeValue(ctx, c);
        }
        for n in [
            0i64,
            1,
            47,
            2147483647,
            2147483648,
            4294967294,
            4294967295,
            i64::MAX,
            i64::MIN,
            -1,
        ] {
            let atom = JS_NewAtomInt64(ctx, n);
            num(out, atom as u64, 4);
            let v = JS_AtomToValue(ctx, atom);
            value(out, v);
            JS_FreeValue(ctx, v);
            JS_FreeAtom(ctx, atom);
            let atom = JS_NewAtomUInt32(ctx, n as u32);
            num(out, atom as u64, 4);
            let v = JS_AtomToString(ctx, atom);
            value(out, v);
            JS_FreeValue(ctx, v);
            JS_FreeAtom(ctx, atom);
        }
        for atom_type in [
            JS_ATOM_TYPE_SYMBOL,
            JS_ATOM_TYPE_GLOBAL_SYMBOL,
            JS_ATOM_TYPE_PRIVATE,
        ] {
            for descr in [c"".as_ptr(), c"name".as_ptr()] {
                let atom = JS_NewAtom(ctx, descr);
                let symbol = JS_NewSymbolFromAtom(ctx, atom, atom_type);
                JS_FreeAtom(ctx, atom);
                let atom = js_get_atom_index(rt, JS_VALUE_GET_PTR(symbol).cast());
                for force in [0, 1] {
                    let v = __JS_AtomToValue(ctx, atom, force);
                    if force == 0 {
                        num(out, JS_VALUE_GET_TAG(v) as u64, 4);
                        dump_string(out, JS_VALUE_GET_PTR(v).cast());
                    } else {
                        value(out, v);
                    }
                    JS_FreeValue(ctx, v);
                }
                JS_FreeValue(ctx, symbol);
            }
        }
        let a = JS_NewAtom(ctx, c"base".as_ptr());
        let b = js_atom_concat_str(ctx, a, c"-suffix".as_ptr());
        let c = js_atom_concat_num(ctx, b, u32::MAX);
        for atom in [a, b, c] {
            let v = JS_AtomToString(ctx, atom);
            value(out, v);
            JS_FreeValue(ctx, v);
            JS_FreeAtom(ctx, atom);
        }
        effects(out, &h, &log);
        finish(out, rt, &mut h);
        // Inject a host allocation failure during initial allocation, growth and
        // Latin1-to-UTF16 widening. The real error callback boundary is observed.
        for trial in 0..64 {
            let mut rt: Box<JSRuntime> = Box::new(core::mem::zeroed());
            let rt = &mut *rt as *mut JSRuntime;
            let mut ctx: JSContext = core::mem::zeroed();
            let ctx = &mut ctx as *mut JSContext;
            let mut h = host(0);
            let mut log = BoundaryLog::default();
            init(rt, ctx, &mut h, &mut log);
            if trial != 0 {
                h.fail = h.calls + trial;
            }
            let mut b: StringBuffer = core::mem::zeroed();
            num(
                out,
                string_buffer_init2(ctx, &mut b, 1, trial % 2) as u64,
                4,
            );
            for step in 0..256 {
                let c = if step % 4 == 0 {
                    0x1f642
                } else if step % 4 == 1 {
                    0x100
                } else {
                    65
                };
                let ret = match step % 7 {
                    0 => string_buffer_putc(&mut b, c),
                    1 => string_buffer_putc8(&mut b, 0xff),
                    2 => string_buffer_putc16(&mut b, 0xd800),
                    3 => string_buffer_write8(&mut b, b"abcdefgh".as_ptr(), 8),
                    4 => string_buffer_write16(&mut b, [0x61u16, 0xff, 0x100, 0xdfff].as_ptr(), 4),
                    5 => string_buffer_fill(&mut b, 0x1ff, 3),
                    _ => string_buffer_puts8(&mut b, c"abcd".as_ptr()),
                };
                num(out, ret as u64, 4);
                for n in [b.len, b.size, b.is_wide_char, b.error_status] {
                    num(out, n as u64, 4);
                }
            }
            let v = string_buffer_end(&mut b);
            value(out, v);
            JS_FreeValue(ctx, v);
            // A failure during CString allocation retains C's input reference.
            h.fail = 0;
            let p = make_string(rt, &[0xffffu16; 1024], 1);
            let v = JS_MKPTR(JS_TAG_STRING, p.cast());
            h.fail = h.calls + 1;
            encoded(out, ctx, v, 0);
            h.fail = 0;
            JS_FreeValue(ctx, v);
            JS_FreeValue(ctx, v);
            let mut b: StringBuffer = core::mem::zeroed();
            string_buffer_init(ctx, &mut b, 1);
            num(
                out,
                string_buffer_realloc(&mut b, JS_STRING_LEN_MAX + 1, 0) as u64,
                4,
            );
            value(out, string_buffer_end(&mut b));
            effects(out, &h, &log);
            finish(out, rt, &mut h);
        }
    }
    unsafe fn shape_fixtures(out: &mut Vec<u8>) {
        for trial in 0..128 {
            let mut rt: Box<JSRuntime> = Box::new(core::mem::zeroed());
            let rt = &mut *rt as *mut JSRuntime;
            let mut ctx: JSContext = core::mem::zeroed();
            let ctx = &mut ctx as *mut JSContext;
            let mut h = host(0);
            let mut log = BoundaryLog::default();
            init(rt, ctx, &mut h, &mut log);
            let mut classes: Vec<JSClass> = (0..JS_CLASS_INIT_COUNT)
                .map(|_| core::mem::zeroed())
                .collect();
            (*rt).class_array = classes.as_mut_ptr();
            (*rt).class_count = JS_CLASS_INIT_COUNT as i32;
            let mut prototypes = vec![JS_NULL; JS_CLASS_INIT_COUNT as usize];
            (*ctx).class_proto = prototypes.as_mut_ptr();
            let a = JS_NewObject(ctx);
            let b = JS_NewObject(ctx);
            let pa = JS_VALUE_GET_PTR(a).cast::<JSObject>();
            let pb = JS_VALUE_GET_PTR(b).cast::<JSObject>();
            if trial > 0 {
                h.fail = h.calls + trial;
            }
            for i in 0..256 {
                let atom = __JS_AtomFromUInt32(i);
                let p = if i % 2 == 0 { pa } else { pb };
                let pr = add_property(ctx, p, atom, JS_PROP_C_W_E);
                num(out, pr.is_null() as u64, 4);
                if !pr.is_null() {
                    (*pr).u.value = JS_NewInt32(ctx, i as i32);
                }
                let sh = (*p).shape;
                for n in [
                    (*sh).is_hashed as u32,
                    (*sh).prop_count as u32,
                    (*sh).prop_size as u32,
                    (*sh).prop_hash_mask,
                    (*js_rc(sh.cast())).ref_count as u32,
                ] {
                    num(out, n as u64, 4);
                }
                // Shape hash itself includes ASLR-dependent prototype addresses;
                // empty prototype here yields the same portable hash.
                num(out, (*sh).hash as u64, 4);
            }
            h.fail = 0;
            // Two identical property sequences reuse a hash-consed shape, then
            // updating one descriptor must detach it and preserve property pointers.
            let x = JS_NewObject(ctx);
            let y = JS_NewObject(ctx);
            let px = JS_VALUE_GET_PTR(x).cast::<JSObject>();
            let py = JS_VALUE_GET_PTR(y).cast::<JSObject>();
            for i in 0..32 {
                for p in [px, py] {
                    let pr = add_property(ctx, p, __JS_AtomFromUInt32(i), JS_PROP_C_W_E);
                    assert!(!pr.is_null());
                    (*pr).u.value = JS_NewInt32(ctx, i as i32);
                }
                num(out, ((*px).shape == (*py).shape) as u64, 4);
            }
            let mut prs = get_shape_prop((*px).shape).add(7);
            num(
                out,
                js_update_property_flags(ctx, px, &mut prs, JS_PROP_CONFIGURABLE) as u64,
                4,
            );
            num(out, prs.offset_from(get_shape_prop((*px).shape)) as u64, 4);
            num(out, (*prs).flags() as u64, 4);
            num(out, ((*px).shape == (*py).shape) as u64, 4);
            num(out, (*get_shape_prop((*py).shape).add(7)).flags() as u64, 4);
            let mut prs = get_shape_prop((*py).shape).add(3);
            num(
                out,
                js_update_property_flags(ctx, py, &mut prs, JS_PROP_ENUMERABLE) as u64,
                4,
            );
            num(out, (*(*py).shape).is_hashed as u64, 4);
            JS_FreeValue(ctx, x);
            JS_FreeValue(ctx, y);
            // Clone and compact a non-hashed shape with alternating tombstones.
            let old = (*pa).shape;
            let sh = js_clone_shape(ctx, old);
            js_free_shape(rt, old);
            (*pa).shape = sh;
            let props = get_shape_prop(sh);
            for i in (0..(*sh).prop_count as usize).step_by(2) {
                JS_FreeAtom(ctx, (*props.add(i)).atom);
                (*props.add(i)).atom = JS_ATOM_NULL as u32;
                (*props.add(i)).set_flags(0);
                (*(*pa).prop.add(i)).u.value = JS_UNDEFINED;
                (*sh).deleted_prop_count += 1;
            }
            num(out, compact_properties(ctx, pa) as u64, 4);
            for i in 0..(*(*pa).shape).prop_count as usize {
                let pr = get_shape_prop((*pa).shape).add(i);
                num(out, (*pr).atom as u64, 4);
                num(out, (*pr).flags() as u64, 4);
                num(
                    out,
                    JS_VALUE_GET_INT((*(*pa).prop.add(i)).u.value) as u64,
                    4,
                );
                let mut property = ptr::null_mut();
                num(
                    out,
                    !find_own_property(&mut property, pa, (*pr).atom).is_null() as u64,
                    4,
                );
                num(out, property.offset_from((*pa).prop) as u64, 4);
            }
            JS_FreeValue(ctx, a);
            JS_FreeValue(ctx, b);
            // Ordinary, wrapped primitive, array and global class initialization.
            for class_id in [
                JS_CLASS_OBJECT,
                JS_CLASS_ERROR,
                JS_CLASS_NUMBER,
                JS_CLASS_ARRAY,
                JS_CLASS_GLOBAL_OBJECT,
                JS_CLASS_RAWJSON,
            ] {
                let v = JS_NewObjectProtoClassAlloc(ctx, JS_NULL, class_id, 8);
                num(out, JS_VALUE_GET_TAG(v) as u64, 4);
                let p = JS_VALUE_GET_PTR(v).cast::<JSObject>();
                for n in [
                    (*p).class_id as u32,
                    (*p).extensible() as u32,
                    (*p).fast_array() as u32,
                    (*p).is_exotic() as u32,
                    (*(*p).shape).prop_count as u32,
                ] {
                    num(out, n as u64, 4);
                }
                JS_FreeValue(ctx, v);
            }
            effects(out, &h, &log);
            finish(out, rt, &mut h);
        }
    }
    mod error_source {
        use super::*;
        include!("../src/quickjs_errors.rs");
        unsafe fn JS_DefinePropertyValue(
            _ctx: *mut JSContext,
            _obj: JSValueConst,
            _atom: JSAtom,
            _val: JSValue,
            _flags: i32,
        ) -> i32 {
            panic!("Error source is type-checked only; production DefineProperty is pending")
        }
    }
}
