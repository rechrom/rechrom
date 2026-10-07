use super::*;
use core::mem::{align_of, offset_of, size_of};
include!("quickjs_header_constants_dump.rs");
fn num(out: &mut Vec<u8>, n: u64, size: usize) {
    out.extend_from_slice(&n.to_le_bytes()[..size]);
}
fn value(out: &mut Vec<u8>, v: JSValue) {
    #[cfg(target_pointer_width = "64")]
    {
        num(out, unsafe { v.u.uint64 }, 8);
        num(out, v.tag as u64, 8);
    }
    #[cfg(target_pointer_width = "32")]
    {
        num(out, v, 8);
    }
    for n in [
        JS_VALUE_GET_NORM_TAG(v),
        JS_VALUE_IS_NAN(v),
        JS_VALUE_HAS_REF_COUNT(v),
        JS_IsNumber(v),
        JS_IsBigInt(core::ptr::null_mut(), v),
        JS_IsBool(v),
        JS_IsNull(v),
        JS_IsUndefined(v),
        JS_IsException(v),
        JS_IsUninitialized(v),
        JS_IsString(v),
        JS_IsSymbol(v),
        JS_IsObject(v),
        JS_VALUE_IS_BOTH_INT(v, JS_FALSE),
        JS_VALUE_IS_BOTH_INT(v, JS_NewInt32(core::ptr::null_mut(), 1)),
        JS_VALUE_IS_BOTH_FLOAT(v, __JS_NewFloat64(core::ptr::null_mut(), 1.5)),
    ] {
        num(out, n as u64, 4);
    }
}
fn floats(out: &mut Vec<u8>, bits: u64) {
    let d = f64::from_bits(bits);
    value(out, __JS_NewFloat64(core::ptr::null_mut(), d));
    value(out, JS_NewFloat64(core::ptr::null_mut(), d));
}
#[test]
#[cfg(target_pointer_width = "64")]
fn official_c_jsvalue_layout_bits_constructors_and_refcounts_match() {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let file = std::env::temp_dir().join(format!("quickjs-value-oracle-{}", std::process::id()));
    let result = include!("quickjs_oracle_config.rs")
        .args(["-std=c11", "-O2", "-I"])
        .arg(root.join("../../vendor/quickjs-2026-06-04"))
        .arg(root.join("tests/quickjs_value_oracle.c"))
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
    let _ = std::fs::remove_file(file);
    assert!(c.status.success());
    let mut out = Vec::new();
    macro_rules! layout{($($t:ty),*)=>{$(num(&mut out,size_of::<$t>() as u64,4);num(&mut out,align_of::<$t>() as u64,4);)*};}
    layout!(
        JSValue,
        JSRefCountHeader,
        JSMallocState,
        JSMallocFunctions,
        JSPropertyEnum,
        JSPropertyDescriptor,
        JSSharedArrayBufferFunctions
    );
    for n in [
        offset_of!(JSValue, u),
        offset_of!(JSValue, tag),
        offset_of!(JSMallocState, opaque),
        offset_of!(JSPropertyDescriptor, value),
        offset_of!(JSPropertyDescriptor, getter),
        offset_of!(JSPropertyDescriptor, setter),
        offset_of!(JSSharedArrayBufferFunctions, sab_opaque),
    ] {
        num(&mut out, n as u64, 4);
    }
    num(&mut out, JS_LIMB_BITS as u64, 4);
    num(&mut out, JS_SHORT_BIG_INT_BITS as u64, 4);
    dump_constants(&mut out);
    layout!(
        JSMemoryUsage,
        JSClassExoticMethods,
        JSClassDef,
        JSCFunctionType,
        JSCFunctionListEntry,
        JSPrintValueOptions
    );
    for n in [
        offset_of!(JSMemoryUsage, binary_object_size),
        offset_of!(JSClassDef, exotic),
        offset_of!(JSCFunctionListEntry, u),
        offset_of!(JSCFunctionListEntry, magic),
        offset_of!(JSPrintValueOptions, max_depth),
    ] {
        num(&mut out, n as u64, 4);
    }
    for n in [
        JS_CFUNC_generic,
        JS_CFUNC_generic_magic,
        JS_CFUNC_constructor,
        JS_CFUNC_constructor_magic,
        JS_CFUNC_constructor_or_func,
        JS_CFUNC_constructor_or_func_magic,
        JS_CFUNC_f_f,
        JS_CFUNC_f_f_f,
        JS_CFUNC_getter,
        JS_CFUNC_setter,
        JS_CFUNC_getter_magic,
        JS_CFUNC_setter_magic,
        JS_CFUNC_iterator_next,
        JS_PROMISE_PENDING,
        JS_PROMISE_FULFILLED,
        JS_PROMISE_REJECTED,
    ] {
        num(&mut out, n as u64, 4);
    }
    for i in [-129, -128, -1, 0, 1, 127, 128, 255, 256] {
        for j in [-129, -128, -1, 0, 1, 127, 128, 255, 256] {
            let mut p = JSPrintValueOptions::default();
            p.set_show_hidden(i);
            p.set_raw_dump(j);
            num(&mut out, p.show_hidden() as u64, 4);
            num(&mut out, p.raw_dump() as u64, 4);
            num(&mut out, (p.flags & 65535) as u64, 4);
        }
    }
    let name = b"entry\0".as_ptr().cast();
    let target = b"target\0".as_ptr().cast();
    let entries = [
        JS_CFUNC_DEF(name, 2, None),
        JS_CFUNC_MAGIC_DEF(name, 3, None, -42),
        JS_CFUNC_SPECIAL_DEF(name, 4, JS_CFUNC_f_f, JSCFunctionType { f_f: None }),
        JS_ITERATOR_NEXT_DEF(name, 5, None, 17),
        JS_CGETSET_DEF(name, None, None),
        JS_CGETSET_MAGIC_DEF(name, None, None, -7),
        JS_PROP_STRING_DEF(name, b"text\0".as_ptr().cast(), 7),
        JS_PROP_INT32_DEF(name, i32::MIN, 3),
        JS_PROP_INT64_DEF(name, i64::MIN, 5),
        JS_PROP_DOUBLE_DEF(name, -0.0, 6),
        JS_PROP_UNDEFINED_DEF(name, 2),
        JS_PROP_ATOM_DEF(name, 123, 1),
        JS_PROP_BOOL_DEF(name, 1, 7),
        JS_OBJECT_DEF(name, core::ptr::null(), 3, 6),
        JS_ALIAS_DEF(name, target),
        JS_ALIAS_BASE_DEF(name, target, 2),
        JS_PROP_DOUBLE_DEF(name, f64::NAN, 0),
    ];
    unsafe {
        for e in entries {
            let text = core::ffi::CStr::from_ptr(e.name).to_bytes();
            num(&mut out, text.len() as u64, 4);
            out.extend_from_slice(text);
            num(&mut out, e.prop_flags as u64, 4);
            num(&mut out, e.def_type as u64, 4);
            num(&mut out, e.magic as u64, 4);
            match e.def_type as i32 {
                JS_DEF_CFUNC => {
                    num(&mut out, e.u.func.length as u64, 4);
                    num(&mut out, e.u.func.cproto as u64, 4);
                    num(&mut out, e.u.func.cfunc.generic.is_some() as u64, 4);
                }
                JS_DEF_CGETSET | JS_DEF_CGETSET_MAGIC => {
                    num(&mut out, e.u.getset.get.generic.is_some() as u64, 4);
                    num(&mut out, e.u.getset.set.generic.is_some() as u64, 4);
                }
                JS_DEF_PROP_STRING => {
                    let text = core::ffi::CStr::from_ptr(e.u.str).to_bytes();
                    num(&mut out, text.len() as u64, 4);
                    out.extend_from_slice(text);
                }
                JS_DEF_PROP_INT64 => num(&mut out, e.u.i64 as u64, 8),
                JS_DEF_PROP_DOUBLE => num(&mut out, e.u.f64.to_bits(), 8),
                JS_DEF_OBJECT => {
                    num(&mut out, (!e.u.prop_list.tab.is_null()) as u64, 4);
                    num(&mut out, e.u.prop_list.len as u64, 4);
                }
                JS_DEF_ALIAS => {
                    let text = core::ffi::CStr::from_ptr(e.u.alias.name).to_bytes();
                    num(&mut out, text.len() as u64, 4);
                    out.extend_from_slice(text);
                    num(&mut out, e.u.alias.base as u64, 4);
                }
                _ => num(&mut out, e.u.i32 as u64, 4),
            }
        }
    }
    for v in [
        JS_NULL,
        JS_UNDEFINED,
        JS_FALSE,
        JS_TRUE,
        JS_EXCEPTION,
        JS_UNINITIALIZED,
        JS_NAN,
    ] {
        value(&mut out, v);
    }
    let ctx = core::ptr::null_mut();
    let ints = [i32::MIN, -1, 0, 1, i32::MAX];
    for i in ints {
        value(&mut out, JS_NewBool(ctx, i));
        value(&mut out, JS_NewInt32(ctx, i));
        value(&mut out, JS_NewCatchOffset(ctx, i));
        value(&mut out, JS_NewUint32(ctx, i as u32));
    }
    for i in [
        i64::MIN,
        i64::MIN + 1,
        -9007199254740993,
        -2147483649,
        i32::MIN as i64,
        -1,
        0,
        i32::MAX as i64,
        2147483648,
        9007199254740991,
        9007199254740993,
        i64::MAX,
    ] {
        value(&mut out, JS_NewInt64(ctx, i));
        value(&mut out, __JS_NewShortBigInt(ctx, i));
    }
    for tag in -16..=16 {
        for i in ints {
            value(&mut out, JS_MKVAL(tag, i));
        }
    }
    for tag in -9..=8 {
        value(&mut out, JS_MKPTR(tag, 0x123456789abcusize as *mut c_void));
    }
    let boundaries: [u64; 20] = [
        0,
        0x8000000000000000,
        1,
        0x8000000000000001,
        0x000fffffffffffff,
        0x0010000000000000,
        0x3ff0000000000000,
        0xbff0000000000000,
        0x41dfffffffc00000,
        0x41e0000000000000,
        0xc1e0000000000000,
        0xc1e0000000200000,
        0x7fefffffffffffff,
        0x7ff0000000000000,
        0xfff0000000000000,
        0x7ff0000000000001,
        0xfff0000000000001,
        0x7ff8000000000000,
        0x7fffffffffffffff,
        0xfff8000000000042,
    ];
    for b in boundaries {
        for offset in -1i64..=1 {
            floats(&mut out, b.wrapping_add(offset as u64));
        }
    }
    let mut x = 0x123456789abcdef0u64;
    for _ in 0..100000 {
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        floats(&mut out, x);
    }
    for x in 0u64..=65535 {
        floats(&mut out, x << 48);
    }
    #[repr(C)]
    struct Cell {
        rc: i32,
        payload: u32,
    }
    let mut cell = Cell { rc: 1, payload: 0 };
    for tag in -9..=-1 {
        cell.rc = 1;
        let p = core::ptr::addr_of_mut!(cell.payload);
        let v = JS_MKPTR(tag, p.cast());
        unsafe {
            let dup = JS_DupValue(ctx, v);
            num(&mut out, cell.rc as u64, 4);
            num(&mut out, (JS_VALUE_GET_PTR(dup) == p.cast()) as u64, 4);
            let dup = JS_DupValueRT(core::ptr::null_mut(), v);
            num(&mut out, cell.rc as u64, 4);
            num(&mut out, (JS_VALUE_GET_PTR(dup) == p.cast()) as u64, 4);
        }
    }
    assert_eq!(out.len(), c.stdout.len());
    if let Some(i) = out.iter().zip(&c.stdout).position(|(r, c)| r != c) {
        panic!(
            "JSValue C/Rust mismatch at byte {i}: Rust={}, C={}",
            out[i], c.stdout[i]
        );
    }
    eprintln!("JSValue header C parity: {} bytes", out.len());
}
