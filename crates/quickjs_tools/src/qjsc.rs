//! qjsc.c (QuickJS 2026-06-04), Fabrice Bellard, MIT.
//! The compiler's algorithms and generated C text follow the original tool.
//! Files, process execution and command-line strings belong to this host crate.
#![allow(unsafe_op_in_unsafe_fn)]
use quickjs::{quickjs::*, quickjs_header::*};
use std::{
    ffi::{c_char, c_void, CStr, CString},
    fs::{self, File},
    io::{self, Write},
    process::Command,
};

#[derive(Clone, Debug)]
pub struct ByteNameListEntry {
    pub name: Vec<u8>,
    pub short_name: Option<Vec<u8>>,
    pub flags: i32,
}
#[derive(Default, Debug)]
pub struct ByteNameList {
    pub array: Vec<ByteNameListEntry>,
}
// qjsc.c:82..101.
pub fn namelist_add_bytes(
    lp: &mut ByteNameList,
    name: &[u8],
    short_name: Option<&[u8]>,
    flags: i32,
) {
    if lp.array.len() == lp.array.capacity() {
        let next = lp.array.capacity() + (lp.array.capacity() >> 1) + 4;
        lp.array.reserve_exact(next - lp.array.len());
    }
    lp.array.push(ByteNameListEntry {
        name: name.into(),
        short_name: short_name.map(Vec::from),
        flags,
    });
}
// qjsc.c:103..113.
pub fn namelist_free_bytes(lp: &mut ByteNameList) {
    while lp.array.pop().is_some() {}
    lp.array = Vec::new();
}
// qjsc.c:115..124.
pub fn namelist_find_bytes<'a>(lp: &'a ByteNameList, name: &[u8]) -> Option<&'a ByteNameListEntry> {
    lp.array.iter().find(|e| e.name == name)
}
const FEATURES: [(&str, Option<&str>); 11] = [
    ("date", Some("Date")),
    ("eval", Some("Eval")),
    ("string-normalize", Some("StringNormalize")),
    ("regexp", Some("RegExp")),
    ("json", Some("JSON")),
    ("proxy", Some("Proxy")),
    ("map", Some("MapSet")),
    ("typedarray", Some("TypedArrays")),
    ("promise", Some("Promise")),
    ("module-loader", None),
    ("weakref", Some("WeakRef")),
];
const CNAME_TYPE_SCRIPT: i32 = 0;
const CNAME_TYPE_MODULE: i32 = 1;
const CNAME_TYPE_JSON_MODULE: i32 = 2;
const MAIN_C_TEMPLATE1: &str = "int main(int argc, char **argv)\n{\n  JSRuntime *rt;\n  JSContext *ctx;\n  rt = JS_NewRuntime();\n  js_std_set_worker_new_context_func(JS_NewCustomContext);\n  js_std_init_handlers(rt);\n";
const MAIN_C_TEMPLATE2: &str = "  js_std_loop(ctx);\n  js_std_free_handlers(rt);\n  JS_FreeContext(ctx);\n  JS_FreeRuntime(rt);\n  return 0;\n}\n";

/// The original CONFIG_CC/CONFIG_PREFIX/CONFIG_LTO macros, supplied by the host.
#[derive(Clone, Debug)]
pub struct CompilerConfig {
    pub cc: Option<String>,
    pub prefix: String,
    pub lto: bool,
    pub version: String,
}
impl Default for CompilerConfig {
    fn default() -> Self {
        Self {
            cc: if cfg!(windows) {
                None
            } else {
                Some(option_env!("QJSC_CC").unwrap_or("cc").into())
            },
            prefix: option_env!("QJSC_PREFIX").unwrap_or("/usr/local").into(),
            lto: option_env!("QJSC_LTO") == Some("1"),
            version: "2026-06-04".into(),
        }
    }
}
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum OutputType {
    C,
    CMain,
    Executable,
}
#[derive(Debug)]
pub struct ByteOptions {
    pub out_filename: Option<Vec<u8>>,
    pub output_type: OutputType,
    pub cname: Option<Vec<u8>>,
    pub feature_bitmap: u64,
    pub module: i32,
    pub byte_swap: bool,
    pub verbose: i32,
    pub strip_flags: i32,
    pub use_lto: bool,
    pub stack_size: usize,
    pub c_ident_prefix: Vec<u8>,
    pub cmodule_list: ByteNameList,
    pub dynamic_module_list: ByteNameList,
    pub optind: usize,
}
impl Default for ByteOptions {
    fn default() -> Self {
        let mut cmodule_list = ByteNameList::default();
        namelist_add_bytes(&mut cmodule_list, b"std", Some(b"std"), 0);
        namelist_add_bytes(&mut cmodule_list, b"os", Some(b"os"), 0);
        Self {
            out_filename: None,
            output_type: OutputType::Executable,
            cname: None,
            feature_bitmap: u64::MAX,
            module: -1,
            byte_swap: false,
            verbose: 0,
            strip_flags: JS_STRIP_SOURCE,
            use_lto: false,
            stack_size: 0,
            c_ident_prefix: b"qjsc_".to_vec(),
            cmodule_list,
            dynamic_module_list: ByteNameList::default(),
            optind: 1,
        }
    }
}
#[derive(Debug)]
pub struct ToolError {
    pub message: String,
    pub message_bytes: Vec<u8>,
    pub help: bool,
}
impl ToolError {
    fn new(message: impl Into<String>) -> Self {
        let message = message.into();
        Self {
            message_bytes: message.as_bytes().to_vec(),
            message,
            help: false,
        }
    }
    fn bytes(message: Vec<u8>) -> Self {
        Self {
            message: byte_display(&message),
            message_bytes: message,
            help: false,
        }
    }
}
fn io_error_text(error: &io::Error) -> String {
    let text = error.to_string();
    if let Some(code) = error.raw_os_error() {
        text.strip_suffix(&format!(" (os error {code})"))
            .unwrap_or(&text)
            .to_owned()
    } else {
        text
    }
}
impl From<io::Error> for ToolError {
    fn from(e: io::Error) -> Self {
        Self::new(e.to_string())
    }
}
fn truncated(text: &[u8], size: usize) -> Vec<u8> {
    text[..text.len().min(size.saturating_sub(1))].to_vec()
}
// qjsc.c:126..156. Prefix and path are raw C bytes; classification is ASCII.
pub fn get_c_name_bytes(prefix: &[u8], file: &[u8], buf_size: usize) -> Vec<u8> {
    let p = file.rsplit(|&c| c == b'/').next().unwrap_or(file);
    let p = &p[..p.iter().rposition(|&c| c == b'.').unwrap_or(p.len())];
    let mut bytes = truncated(prefix, buf_size);
    for &c in p {
        if bytes.len() >= buf_size.saturating_sub(1) {
            break;
        }
        bytes.push(if c.is_ascii_alphanumeric() { c } else { b'_' });
    }
    bytes
}
// qjsc.c:158..172.
pub fn dump_hex(f: &mut dyn Write, buf: &[u8]) -> io::Result<()> {
    let mut col = 0;
    for b in buf {
        write!(f, " 0x{b:02x},")?;
        col += 1;
        if col == 8 {
            writeln!(f)?;
            col = 0;
        }
    }
    if col != 0 {
        writeln!(f)?;
    }
    Ok(())
}
// qjsc.c:217..237.
pub fn find_unique_cname_bytes(cname: &mut Vec<u8>, cname_size: usize, list: &ByteNameList) {
    assert!(cname_size >= 32);
    *cname = truncated(cname, cname_size - 15);
    let mut suffix = 1;
    loop {
        let mut candidate = cname.clone();
        candidate.push(b'_');
        candidate.extend(suffix.to_string().bytes());
        candidate = truncated(&candidate, 1024);
        if namelist_find_bytes(list, &candidate).is_none() {
            *cname = truncated(&candidate, cname_size);
            break;
        }
        suffix += 1;
    }
}
// qjsc.c:393..426.
pub fn help(config: &CompilerConfig) -> String {
    let mut s = format!(
        "QuickJS Compiler version {}\nusage: qjsc [options] [files]\n\noptions are:\n-c          only output bytecode to a C file\n-e          output main() and bytecode to a C file (default = executable output)\n-o output   set the output filename\n-N cname    set the C name of the generated data\n-m          compile as Javascript module (default=autodetect)\n-D module_name         compile a dynamically loaded module or worker\n-M module_name[,cname] add initialization code for an external C module\n-x          byte swapped output\n-p prefix   set the prefix of the generated C names\n-S n        set the maximum stack size to 'n' bytes (default=1048576)\n-s            strip all the debug info\n--keep-source keep the source code\n",
        config.version
    );
    if config.lto {
        s.push_str("-flto       use link time optimization\n-fno-[");
        s.push_str(&FEATURES.iter().map(|e| e.0).collect::<Vec<_>>().join("|"));
        s.push_str("]\n            disable selected language features (smaller code size)\n");
    }
    s
}
/// strtod's end-pointer controls the original suffix test; suffix letters accept trailing text.
// qjsc.c:529..552.
pub fn get_suffixed_size_bytes(text: &[u8]) -> Result<usize, ToolError> {
    let data = text;
    let mut p = 0;
    while p < data.len() && matches!(data[p], b' ' | b'\t' | b'\n' | b'\r' | 11 | 12) {
        p += 1;
    }
    let start = p;
    if p < data.len() && matches!(data[p], b'+' | b'-') {
        p += 1;
    }
    let is_hex = data.get(p) == Some(&b'0')
        && matches!(data.get(p + 1), Some(b'x' | b'X'))
        && (data.get(p + 2).is_some_and(u8::is_ascii_hexdigit)
            || data.get(p + 2) == Some(&b'.')
                && data.get(p + 3).is_some_and(u8::is_ascii_hexdigit));
    let mut special = None;
    if data
        .get(p..p + 3)
        .is_some_and(|v| v.eq_ignore_ascii_case(b"inf"))
    {
        p += 3;
        if data
            .get(p..p + 5)
            .is_some_and(|v| v.eq_ignore_ascii_case(b"inity"))
        {
            p += 5;
        }
        special = Some(if data.get(start) == Some(&b'-') {
            f64::NEG_INFINITY
        } else {
            f64::INFINITY
        });
    } else if data
        .get(p..p + 3)
        .is_some_and(|v| v.eq_ignore_ascii_case(b"nan"))
    {
        p += 3;
        if data.get(p) == Some(&b'(') {
            let mut end = p + 1;
            while data
                .get(end)
                .is_some_and(|c| c.is_ascii_alphanumeric() || *c == b'_')
            {
                end += 1;
            }
            if data.get(end) == Some(&b')') {
                p = end + 1;
            }
        }
        special = Some(f64::NAN);
    } else {
        if is_hex {
            p += 2;
        }
        let mut count = 0;
        while data.get(p).is_some_and(|c| {
            if is_hex {
                c.is_ascii_hexdigit()
            } else {
                c.is_ascii_digit()
            }
        }) {
            p += 1;
            count += 1;
        }
        if data.get(p) == Some(&b'.') {
            p += 1;
            while data.get(p).is_some_and(|c| {
                if is_hex {
                    c.is_ascii_hexdigit()
                } else {
                    c.is_ascii_digit()
                }
            }) {
                p += 1;
                count += 1;
            }
        }
        if count == 0 {
            p = 0;
        } else if data.get(p).is_some_and(|c| {
            if is_hex {
                matches!(c, b'p' | b'P')
            } else {
                matches!(c, b'e' | b'E')
            }
        }) {
            let old = p;
            p += 1;
            if data.get(p).is_some_and(|c| matches!(c, b'+' | b'-')) {
                p += 1;
            }
            let exp = p;
            while data.get(p).is_some_and(u8::is_ascii_digit) {
                p += 1;
            }
            if p == exp {
                p = old;
            }
        }
    }
    let number = if let Some(v) = special {
        v
    } else if p == 0 {
        0.0
    } else if is_hex {
        let cstr = CString::new(&data[start..p]).unwrap();
        let mut memory = quickjs::dtoa_header::JSATODTempMem::default();
        unsafe { quickjs::dtoa::js_atod(cstr.as_ptr(), std::ptr::null_mut(), 0, 0, &mut memory) }
    } else {
        std::str::from_utf8(&text[start..p])
            .ok()
            .and_then(|s| s.parse::<f64>().ok())
            .unwrap_or(0.0)
    };
    let value = number as usize;
    match data.get(p).copied() {
        Some(b'G') => Ok(value.wrapping_shl(30)),
        Some(b'M') => Ok(value.wrapping_shl(20)),
        Some(b'k' | b'K') => Ok(value.wrapping_shl(10)),
        None => Ok(value),
        _ => Err(raw_error(b"qjs: invalid suffix: ", &text[p..], b"\n")),
    }
}
// qjsc.c:561..574.
pub fn get_short_optarg_bytes(
    optind: &mut usize,
    opt: u8,
    arg: &[u8],
    argv: &[Vec<u8>],
) -> Result<Vec<u8>, ToolError> {
    if !arg.is_empty() {
        Ok(arg.into())
    } else if *optind < argv.len() {
        let s = argv[*optind].clone();
        *optind += 1;
        Ok(s)
    } else {
        Err(ToolError::new(format!(
            "qjsc: expecting parameter for -{}\n",
            opt as char
        )))
    }
}
// qjsc.c:576..735.
pub fn parse_options_bytes(argv: &[Vec<u8>]) -> Result<ByteOptions, ToolError> {
    let mut o = ByteOptions::default();
    while o.optind < argv.len() && argv[o.optind].starts_with(b"-") {
        let argument = argv[o.optind].clone();
        let mut arg = &argument[1..];
        if arg.is_empty() {
            break;
        }
        o.optind += 1;
        let mut longopt: &[u8] = b"";
        if arg.starts_with(b"-") {
            longopt = &arg[1..];
            arg = b"";
            if longopt.is_empty() {
                break;
            }
        }
        while !arg.is_empty() || !longopt.is_empty() {
            let opt = arg.first().copied().unwrap_or(0);
            if opt != 0 {
                arg = &arg[1..];
            }
            if opt == b'h' || opt == b'?' || longopt == b"help" {
                return Err(ToolError {
                    message: String::new(),
                    message_bytes: Vec::new(),
                    help: true,
                });
            }
            match opt {
                b'o' => {
                    o.out_filename = Some(get_short_optarg_bytes(&mut o.optind, opt, arg, argv)?);
                    break;
                }
                b'c' => o.output_type = OutputType::C,
                b'e' => o.output_type = OutputType::CMain,
                b'N' => {
                    o.cname = Some(get_short_optarg_bytes(&mut o.optind, opt, arg, argv)?);
                    break;
                }
                b'f' => {
                    let a = get_short_optarg_bytes(&mut o.optind, opt, arg, argv)?;
                    if a == b"lto" {
                        o.use_lto = true;
                    } else if let Some(p) = a.strip_prefix(b"no-") {
                        o.use_lto = true;
                        if let Some(i) = FEATURES.iter().position(|e| e.0.as_bytes() == p) {
                            o.feature_bitmap &= !(1u64 << i);
                        } else {
                            return Err(raw_error(b"unsupported feature: ", &a, b"\n"));
                        }
                    } else {
                        return Err(raw_error(b"unsupported feature: ", &a, b"\n"));
                    }
                    break;
                }
                b'm' => o.module = 1,
                b'M' => {
                    let a = truncated(
                        &get_short_optarg_bytes(&mut o.optind, opt, arg, argv)?,
                        1024,
                    );
                    let (path, cname) = if let Some(pos) = a.iter().position(|&c| c == b',') {
                        (a[..pos].to_vec(), a[pos + 1..].to_vec())
                    } else {
                        let c = get_c_name_bytes(&o.c_ident_prefix, &a, 1024);
                        (a, c)
                    };
                    namelist_add_bytes(&mut o.cmodule_list, &path, Some(&cname), 0);
                    break;
                }
                b'D' => {
                    let a = get_short_optarg_bytes(&mut o.optind, opt, arg, argv)?;
                    namelist_add_bytes(&mut o.dynamic_module_list, &a, None, 0);
                    break;
                }
                b'x' => o.byte_swap = true,
                b'v' => o.verbose += 1,
                b'p' => {
                    o.c_ident_prefix = get_short_optarg_bytes(&mut o.optind, opt, arg, argv)?;
                    break;
                }
                b'S' => {
                    o.stack_size = get_suffixed_size_bytes(&get_short_optarg_bytes(
                        &mut o.optind,
                        opt,
                        arg,
                        argv,
                    )?)?;
                    break;
                }
                b's' => o.strip_flags = JS_STRIP_DEBUG,
                0 if longopt == b"keep-source" => o.strip_flags = 0,
                _ => {
                    let mut error = if opt != 0 {
                        raw_error(b"qjsc: unknown option '-", &[opt], b"'\n")
                    } else {
                        raw_error(b"qjsc: unknown option '--", longopt, b"'\n")
                    };
                    error.help = true;
                    return Err(error);
                }
            }
            longopt = b"";
        }
    }
    if o.optind >= argv.len() {
        return Err(ToolError {
            message: String::new(),
            message_bytes: Vec::new(),
            help: true,
        });
    }
    Ok(o)
}

struct CompiledObject {
    name: Vec<u8>,
    bytes: Vec<u8>,
    kind: i32,
    module_name: Option<Vec<u8>>,
}
struct CompilerState {
    objects: Vec<CompiledObject>,
    options: ByteOptions,
    cname_list: ByteNameList,
    init_module_list: ByteNameList,
    outfile: File,
    dynamic_export: bool,
    io_error: Option<ToolError>,
}
// qjsc.c:179..209.
unsafe fn output_object_code(
    ctx: *mut JSContext,
    state: *mut CompilerState,
    obj: JSValue,
    c_name: &[u8],
    c_name_type: i32,
) -> Result<(), ToolError> {
    let mut flags = if c_name_type == CNAME_TYPE_JSON_MODULE {
        0
    } else {
        JS_WRITE_OBJ_BYTECODE
    };
    if (*state).options.byte_swap {
        flags |= JS_WRITE_OBJ_BSWAP;
    }
    let mut len = 0;
    let buf = JS_WriteObject(ctx, &mut len, obj, flags);
    if buf.is_null() {
        return Err(ToolError::new(dump_error(ctx)));
    }
    namelist_add_bytes(&mut (*state).cname_list, c_name, None, c_name_type);
    (*state).objects.push(CompiledObject {
        name: c_name.into(),
        bytes: std::slice::from_raw_parts(buf, len).to_vec(),
        kind: c_name_type,
        module_name: None,
    });
    let result = (|| {
        let fo = &mut (*state).outfile;
        fo.write_all(b"const uint32_t ")?;
        fo.write_all(c_name)?;
        write!(fo, "_size = {};\n\nconst uint8_t ", len as u32)?;
        fo.write_all(c_name)?;
        writeln!(fo, "[{}] = {{", len as u32)?;
        dump_hex(fo, std::slice::from_raw_parts(buf, len))?;
        write!(fo, "}};\n\n")
    })();
    js_free(ctx, buf.cast());
    result.map_err(ToolError::from)
}
// qjsc.c:211..215.
unsafe fn js_module_dummy_init(_ctx: *mut JSContext, _m: *mut JSModuleDef) -> i32 {
    std::process::abort()
}
unsafe fn dump_error(ctx: *mut JSContext) -> String {
    crate::quickjs_libc::js_std_dump_error(ctx);
    String::new()
}
unsafe fn read_source(ctx: *mut JSContext, name: *const c_char) -> Option<Vec<u8>> {
    let mut len = 0;
    let buf = crate::quickjs_libc::js_load_file(ctx, &mut len, name);
    if buf.is_null() {
        return None;
    }
    let result = std::slice::from_raw_parts(buf, len).to_vec();
    crate::quickjs_libc::js_free_file_buffer(ctx, buf, len);
    Some(result)
}
unsafe fn load_module(
    ctx: *mut JSContext,
    name: *const c_char,
    state: *mut CompilerState,
    attributes: JSValue,
) -> Result<*mut JSModuleDef, ToolError> {
    let module_name = CStr::from_ptr(name).to_bytes().to_vec();
    if let Some(e) = namelist_find_bytes(&(*state).options.cmodule_list, &module_name).cloned() {
        namelist_add_bytes(
            &mut (*state).init_module_list,
            &e.name,
            e.short_name.as_deref(),
            0,
        );
        return Ok(JS_NewCModule(ctx, name, Some(js_module_dummy_init)));
    }
    if module_name.ends_with(b".so") {
        io::stderr().write_all(&joined(&[
            b"Warning: binary module '",
            &module_name,
            b"' will be dynamically loaded\n",
        ]))?;
        (*state).dynamic_export = true;
        return Ok(JS_NewCModule(ctx, name, Some(js_module_dummy_init)));
    }
    let mut buf = match read_source(ctx, name) {
        Some(b) => b,
        None => {
            JS_ThrowReferenceError(
                ctx,
                JSErrorMessage::Pieces(&[b"could not load module filename '", &module_name, b"'"]),
            );
            return Ok(std::ptr::null_mut());
        }
    };
    let len = buf.len();
    buf.push(0);
    let res = crate::quickjs_libc::js_module_test_json(ctx, attributes);
    if module_name.ends_with(b".json") || res > 0 {
        let val = JS_ParseJSON2(
            ctx,
            buf.as_ptr().cast(),
            len,
            name,
            if res == 2 { JS_PARSE_JSON_EXT } else { 0 },
        );
        if JS_IsException(val) != 0 {
            return Ok(std::ptr::null_mut());
        }
        let m = JS_NewCModule(ctx, name, Some(js_module_dummy_init));
        if m.is_null() {
            JS_FreeValue(ctx, val);
            return Ok(m);
        }
        let mut cname = get_c_name_bytes(&(*state).options.c_ident_prefix, &module_name, 1024);
        if namelist_find_bytes(&(*state).cname_list, &cname).is_some() {
            find_unique_cname_bytes(&mut cname, 1024, &(*state).cname_list);
        }
        let result = (|| {
            (*state).outfile.write_all(b"static const uint8_t ")?;
            (*state).outfile.write_all(&cname)?;
            (*state).outfile.write_all(b"_module_name[] = {\n")?;
            dump_hex(
                &mut (*state).outfile,
                CStr::from_ptr(name).to_bytes_with_nul(),
            )?;
            write!(&mut (*state).outfile, "}};\n\n")?;
            output_object_code(ctx, state, val, &cname, CNAME_TYPE_JSON_MODULE)
        })();
        JS_FreeValue(ctx, val);
        result?;
        (*state).objects.last_mut().unwrap().module_name =
            Some(CStr::from_ptr(name).to_bytes_with_nul().to_vec());
        Ok(m)
    } else {
        let func_val = JS_Eval(
            ctx,
            buf.as_ptr().cast(),
            len,
            name,
            JS_EVAL_TYPE_MODULE | JS_EVAL_FLAG_COMPILE_ONLY,
        );
        if JS_IsException(func_val) != 0 {
            return Ok(std::ptr::null_mut());
        }
        let mut cname = get_c_name_bytes(&(*state).options.c_ident_prefix, &module_name, 1024);
        if namelist_find_bytes(&(*state).cname_list, &cname).is_some() {
            find_unique_cname_bytes(&mut cname, 1024, &(*state).cname_list);
        }
        let result = output_object_code(ctx, state, func_val, &cname, CNAME_TYPE_MODULE);
        let m = JS_VALUE_GET_PTR(func_val).cast();
        JS_FreeValue(ctx, func_val);
        result?;
        Ok(m)
    }
}
/// Runtime callbacks use a per-invocation state rather than the original process globals.
// qjsc.c:239..328.
pub unsafe fn jsc_module_loader(
    ctx: *mut JSContext,
    name: *const c_char,
    opaque: *mut c_void,
    attributes: JSValue,
) -> *mut JSModuleDef {
    let state = opaque.cast::<CompilerState>();
    match load_module(ctx, name, state, attributes) {
        Ok(m) => m,
        Err(e) => {
            (*state).io_error = Some(e);
            JS_ThrowReferenceError(ctx, c"qjsc: module output failed".as_ptr());
            std::ptr::null_mut()
        }
    }
}
// qjsc.c:330..373.
unsafe fn compile_file(
    ctx: *mut JSContext,
    state: *mut CompilerState,
    filename: &[u8],
    cname: Option<&[u8]>,
    mut module: i32,
) -> Result<(), ToolError> {
    let name = CString::new(filename).map_err(|e| ToolError::new(e.to_string()))?;
    let mut buf = read_source(ctx, name.as_ptr())
        .ok_or_else(|| raw_error(b"Could not load '", filename, b"'\n"))?;
    let len = buf.len();
    buf.push(0);
    let mut eval_flags = JS_EVAL_FLAG_COMPILE_ONLY;
    if module < 0 {
        module =
            (filename.ends_with(b".mjs") || JS_DetectModule(buf.as_ptr().cast(), len) != 0) as i32;
    }
    eval_flags |= if module != 0 {
        JS_EVAL_TYPE_MODULE
    } else {
        JS_EVAL_TYPE_GLOBAL
    };
    let obj = JS_Eval(ctx, buf.as_ptr().cast(), len, name.as_ptr(), eval_flags);
    if JS_IsException(obj) != 0 {
        return Err((*state)
            .io_error
            .take()
            .unwrap_or_else(|| ToolError::new(dump_error(ctx))));
    }
    let mut c_name = if let Some(n) = cname {
        truncated(n, 1024)
    } else {
        get_c_name_bytes(&(*state).options.c_ident_prefix, filename, 1024)
    };
    if cname.is_none() && namelist_find_bytes(&(*state).cname_list, &c_name).is_some() {
        find_unique_cname_bytes(&mut c_name, 1024, &(*state).cname_list);
    }
    let result = output_object_code(ctx, state, obj, &c_name, CNAME_TYPE_SCRIPT);
    JS_FreeValue(ctx, obj);
    result
}

// qjsc.c:431..447.
pub fn exec_cmd_bytes(argv: &[Vec<u8>]) -> io::Result<i32> {
    let args = argv
        .iter()
        .map(|s| byte_os(s).map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e.message)))
        .collect::<io::Result<Vec<_>>>()?;
    let status = Command::new(&args[0]).args(&args[1..]).status()?;
    Ok(status.code().unwrap_or(1))
}
pub fn executable_argv_bytes(
    config: &CompilerConfig,
    out_filename: &[u8],
    cfilename: &[u8],
    use_lto: bool,
    dynamic_export: bool,
    exename: &[u8],
) -> Result<Vec<Vec<u8>>, ToolError> {
    let cc = config
        .cc
        .as_ref()
        .ok_or_else(|| ToolError::new("Executable output is not supported for this target\n"))?
        .as_bytes()
        .to_vec();
    let exename = truncated(exename, 1024);
    let exe_dir = exename
        .iter()
        .rposition(|&c| c == b'/')
        .map(|i| &exename[..i])
        .unwrap_or(b".");
    let (inc_dir, lib_dir) = if File::open(byte_path(&joined(&[exe_dir, b"/quickjs.h"]))?).is_ok() {
        (exe_dir.to_vec(), exe_dir.to_vec())
    } else {
        (
            truncated(
                &joined(&[config.prefix.as_bytes(), b"/include/quickjs"]),
                1024,
            ),
            truncated(&joined(&[config.prefix.as_bytes(), b"/lib/quickjs"]), 1024),
        )
    };
    let mut args = vec![cc, b"-O2".to_vec()];
    let suffix = if config.lto && use_lto {
        args.push(b"-flto".to_vec());
        b".lto".as_slice()
    } else {
        b""
    };
    args.extend([
        b"-D".to_vec(),
        b"_GNU_SOURCE".to_vec(),
        b"-I".to_vec(),
        inc_dir,
        b"-o".to_vec(),
        out_filename.to_vec(),
    ]);
    if dynamic_export {
        args.push(b"-rdynamic".to_vec());
    }
    args.push(cfilename.to_vec());
    args.push(truncated(
        &joined(&[&lib_dir, b"/libquickjs", suffix, b".a"]),
        1024,
    ));
    args.extend([b"-lm".to_vec(), b"-ldl".to_vec(), b"-lpthread".to_vec()]);
    Ok(args)
}
fn print_command(args: &[Vec<u8>]) -> io::Result<()> {
    let mut out = io::stdout().lock();
    for arg in args {
        out.write_all(arg)?;
        out.write_all(b" ")?;
    }
    out.write_all(b"\n")
}
fn output_executable(
    config: &CompilerConfig,
    out_filename: &[u8],
    cfilename: &[u8],
    use_lto: bool,
    verbose: i32,
    dynamic_export: bool,
    exename: &[u8],
) -> Result<i32, ToolError> {
    let args = executable_argv_bytes(
        config,
        out_filename,
        cfilename,
        use_lto,
        dynamic_export,
        exename,
    )?;
    if verbose != 0 {
        print_command(&args)?;
    }
    let result = exec_cmd_bytes(&args).unwrap_or(1);
    let _ = fs::remove_file(byte_path(cfilename)?);
    Ok(result)
}
// qjsc.c:773..849.
fn output_main(state: &mut CompilerState) -> Result<(), ToolError> {
    let fo = &mut state.outfile;
    write!(
        fo,
        "static JSContext *JS_NewCustomContext(JSRuntime *rt)\n{{\n  JSContext *ctx = JS_NewContextRaw(rt);\n  if (!ctx)\n    return NULL;\n  JS_AddIntrinsicBaseObjects(ctx);\n"
    )?;
    for (i, (_, name)) in FEATURES.iter().enumerate() {
        if state.options.feature_bitmap & (1u64 << i) != 0 {
            if let Some(name) = name {
                writeln!(fo, "  JS_AddIntrinsic{name}(ctx);")?;
            }
        }
    }
    for e in &state.init_module_list.array {
        let short = e.short_name.as_deref().unwrap_or(b"");
        fo.write_all(&joined(&[
            b"  {\n    extern JSModuleDef *js_init_module_",
            short,
            b"(JSContext *ctx, const char *name);\n    js_init_module_",
            short,
            b"(ctx, \"",
            &e.name,
            b"\");\n  }\n",
        ]))?;
    }
    for e in &state.cname_list.array {
        if e.flags == CNAME_TYPE_MODULE {
            fo.write_all(&joined(&[
                b"  js_std_eval_binary(ctx, ",
                &e.name,
                b", ",
                &e.name,
                b"_size, 1);\n",
            ]))?;
        } else if e.flags == CNAME_TYPE_JSON_MODULE {
            fo.write_all(&joined(&[
                b"  js_std_eval_binary_json_module(ctx, ",
                &e.name,
                b", ",
                &e.name,
                b"_size, (const char *)",
                &e.name,
                b"_module_name);\n",
            ]))?;
        }
    }
    write!(fo, "  return ctx;\n}}\n\n{MAIN_C_TEMPLATE1}")?;
    if state.options.stack_size != 0 {
        writeln!(
            fo,
            "  JS_SetMaxStackSize(rt, {});",
            state.options.stack_size as u32
        )?;
    }
    if state.options.feature_bitmap & (1 << 9) != 0 {
        writeln!(
            fo,
            "  JS_SetModuleLoaderFunc2(rt, NULL, js_module_loader, js_module_check_attributes, NULL);"
        )?;
    }
    write!(
        fo,
        "  ctx = JS_NewCustomContext(rt);\n  js_std_add_helpers(ctx, argc, argv);\n"
    )?;
    for e in &state.cname_list.array {
        if e.flags == CNAME_TYPE_SCRIPT {
            fo.write_all(&joined(&[
                b"  js_std_eval_binary(ctx, ",
                &e.name,
                b", ",
                &e.name,
                b"_size, 0);\n",
            ]))?;
        }
    }
    write!(fo, "{MAIN_C_TEMPLATE2}")?;
    Ok(())
}
// qjsc.c:576..876.
fn run_backend(
    argv: &[Vec<u8>],
    config: &CompilerConfig,
    rust_backend: bool,
) -> Result<i32, ToolError> {
    let options = parse_options_bytes(argv)?;
    let out_filename = options.out_filename.clone().unwrap_or_else(|| {
        if options.output_type == OutputType::Executable {
            b"a.out".as_slice()
        } else {
            b"out.c".as_slice()
        }
        .to_vec()
    });
    let cfilename = if options.output_type == OutputType::Executable {
        if cfg!(any(windows, target_os = "android")) {
            format!("out{}.c", std::process::id()).into_bytes()
        } else {
            format!("/tmp/out{}.c", std::process::id()).into_bytes()
        }
    } else {
        truncated(&out_filename, 1024)
    };
    let outfile = File::create(byte_path(&cfilename)?).map_err(|e| {
        ToolError::bytes(joined(&[
            &cfilename,
            b": ",
            io_error_text(&e).as_bytes(),
            b"\n",
        ]))
    })?;
    let mut state = Box::new(CompilerState {
        objects: Vec::new(),
        options,
        cname_list: ByteNameList::default(),
        init_module_list: ByteNameList::default(),
        outfile,
        dynamic_export: false,
        io_error: None,
    });
    unsafe {
        let rt = JS_NewRuntime();
        if rt.is_null() {
            return Err(ToolError::new("qjsc: out of memory\n"));
        }
        let ctx = JS_NewContext(rt);
        if ctx.is_null() {
            JS_FreeRuntime(rt);
            return Err(ToolError::new("qjsc: out of memory\n"));
        }
        JS_SetStripInfo(rt, state.options.strip_flags);
        let state_ptr = (&mut *state) as *mut CompilerState;
        JS_SetModuleLoaderFunc2(rt, None, Some(jsc_module_loader), None, state_ptr.cast());
        let result = (|| {
            write!(
                &mut state.outfile,
                "/* File generated automatically by the QuickJS compiler. */\n\n"
            )?;
            if state.options.output_type != OutputType::C {
                write!(&mut state.outfile, "#include \"quickjs-libc.h\"\n\n")?;
            } else {
                write!(&mut state.outfile, "#include <inttypes.h>\n\n")?;
            }
            let mut cname = state.options.cname.clone();
            let module = state.options.module;
            for filename in &argv[state.options.optind..] {
                compile_file(ctx, state_ptr, filename, cname.as_deref(), module)?;
                cname = None;
            }
            let dynamic = state.options.dynamic_module_list.array.clone();
            for e in dynamic {
                let name =
                    CString::new(e.name.clone()).map_err(|x| ToolError::new(x.to_string()))?;
                if jsc_module_loader(ctx, name.as_ptr(), state_ptr.cast(), JS_UNDEFINED).is_null() {
                    return Err(state.io_error.take().unwrap_or_else(|| {
                        raw_error(b"Could not load dynamic module '", &e.name, b"'\n")
                    }));
                }
            }
            if state.options.output_type != OutputType::C {
                output_main(&mut state)?;
            }
            state.outfile.flush()?;
            Ok(())
        })();
        JS_FreeContext(ctx);
        JS_FreeRuntime(rt);
        result?;
    }
    let output_type = state.options.output_type;
    let use_lto = state.options.use_lto;
    let verbose = state.options.verbose;
    let dynamic_export = state.dynamic_export;
    if output_type == OutputType::Executable && rust_backend {
        let result =
            output_rust_executable(&state, &out_filename, use_lto, verbose, dynamic_export);
        let _ = fs::remove_file(byte_path(&cfilename)?);
        return result;
    }
    namelist_free_bytes(&mut state.cname_list);
    namelist_free_bytes(&mut state.options.cmodule_list);
    namelist_free_bytes(&mut state.init_module_list);
    drop(state);
    if output_type == OutputType::Executable {
        output_executable(
            config,
            &out_filename,
            &cfilename,
            use_lto,
            verbose,
            dynamic_export,
            argv.first().map(Vec::as_slice).unwrap_or(b"qjsc"),
        )
    } else {
        Ok(0)
    }
}
pub fn run(argv: &[String], config: &CompilerConfig) -> Result<i32, ToolError> {
    run_backend(
        &argv
            .iter()
            .map(|s| s.as_bytes().to_vec())
            .collect::<Vec<_>>(),
        config,
        false,
    )
}
/// Pure-Rust executable host. -c/-e retain the original exact C output format.
pub fn run_rust(argv: &[String], config: &CompilerConfig) -> Result<i32, ToolError> {
    run_backend(
        &argv
            .iter()
            .map(|s| s.as_bytes().to_vec())
            .collect::<Vec<_>>(),
        config,
        true,
    )
}

fn byte_display(bytes: &[u8]) -> String {
    match std::str::from_utf8(bytes) {
        Ok(s) => s.to_owned(),
        Err(_) => bytes
            .iter()
            .map(|&b| {
                if b.is_ascii() {
                    char::from(b).to_string()
                } else {
                    format!("\\x{b:02x}")
                }
            })
            .collect(),
    }
}
fn joined(parts: &[&[u8]]) -> Vec<u8> {
    let mut out = Vec::new();
    for part in parts {
        out.extend_from_slice(part);
    }
    out
}
fn raw_error(prefix: &[u8], bytes: &[u8], suffix: &[u8]) -> ToolError {
    ToolError::bytes(joined(&[prefix, bytes, suffix]))
}
#[cfg(unix)]
fn os_bytes(value: std::ffi::OsString) -> Result<Vec<u8>, ToolError> {
    use std::os::unix::ffi::OsStringExt;
    Ok(value.into_vec())
}
#[cfg(windows)]
fn os_bytes(value: std::ffi::OsString) -> Result<Vec<u8>, ToolError> {
    crate::quickjs_libc::os_bytes(value).map_err(|e| ToolError::new(format!("qjsc: {e}\n")))
}
#[cfg(not(any(unix, windows)))]
fn os_bytes(value: std::ffi::OsString) -> Result<Vec<u8>, ToolError> {
    value.into_string().map(String::into_bytes).map_err(|_| {
        ToolError::new("qjsc: host argument cannot be represented by the UTF-8 adapter\n")
    })
}
fn byte_os(value: &[u8]) -> Result<std::ffi::OsString, ToolError> {
    #[cfg(unix)]
    {
        use std::os::unix::ffi::OsStrExt;
        Ok(std::ffi::OsStr::from_bytes(value).to_os_string())
    }
    #[cfg(windows)]
    {
        crate::quickjs_libc::byte_os(value).map_err(|e| ToolError::new(format!("qjsc: {e}\n")))
    }
    #[cfg(not(any(unix, windows)))]
    {
        Ok(std::str::from_utf8(value)
            .map_err(|_| ToolError::new("qjsc: non UTF-8 path unsupported on this host\n"))?
            .into())
    }
}
fn byte_path(value: &[u8]) -> Result<std::path::PathBuf, ToolError> {
    Ok(byte_os(value)?.into())
}
fn string_adapter(value: Vec<u8>) -> Result<String, ToolError> {
    String::from_utf8(value).map_err(|_| {
        ToolError::new("qjsc: raw bytes cannot be represented by the String adapter\n")
    })
}
#[derive(Clone, Debug)]
pub struct NameListEntry {
    pub name: String,
    pub short_name: Option<String>,
    pub flags: i32,
}
#[derive(Default, Debug)]
pub struct NameList {
    pub array: Vec<NameListEntry>,
}
// qjsc.c:82..101.
pub fn namelist_add(lp: &mut NameList, name: &str, short_name: Option<&str>, flags: i32) {
    if lp.array.len() == lp.array.capacity() {
        let next = lp.array.capacity() + (lp.array.capacity() >> 1) + 4;
        lp.array.reserve_exact(next - lp.array.len());
    }
    lp.array.push(NameListEntry {
        name: name.into(),
        short_name: short_name.map(String::from),
        flags,
    });
}
// qjsc.c:103..113.
pub fn namelist_free(lp: &mut NameList) {
    while lp.array.pop().is_some() {}
    lp.array = Vec::new();
}
// qjsc.c:115..124.
pub fn namelist_find<'a>(lp: &'a NameList, name: &str) -> Option<&'a NameListEntry> {
    lp.array.iter().find(|e| e.name == name)
}
#[derive(Debug)]
pub struct Options {
    pub out_filename: Option<String>,
    pub output_type: OutputType,
    pub cname: Option<String>,
    pub feature_bitmap: u64,
    pub module: i32,
    pub byte_swap: bool,
    pub verbose: i32,
    pub strip_flags: i32,
    pub use_lto: bool,
    pub stack_size: usize,
    pub c_ident_prefix: String,
    pub cmodule_list: NameList,
    pub dynamic_module_list: NameList,
    pub optind: usize,
}
impl Default for Options {
    fn default() -> Self {
        let mut cmodule_list = NameList::default();
        namelist_add(&mut cmodule_list, "std", Some("std"), 0);
        namelist_add(&mut cmodule_list, "os", Some("os"), 0);
        Self {
            out_filename: None,
            output_type: OutputType::Executable,
            cname: None,
            feature_bitmap: u64::MAX,
            module: -1,
            byte_swap: false,
            verbose: 0,
            strip_flags: JS_STRIP_SOURCE,
            use_lto: false,
            stack_size: 0,
            c_ident_prefix: "qjsc_".into(),
            cmodule_list,
            dynamic_module_list: NameList::default(),
            optind: 1,
        }
    }
}

fn byte_namelist(list: &NameList) -> ByteNameList {
    ByteNameList {
        array: list
            .array
            .iter()
            .map(|e| ByteNameListEntry {
                name: e.name.as_bytes().to_vec(),
                short_name: e.short_name.as_ref().map(|s| s.as_bytes().to_vec()),
                flags: e.flags,
            })
            .collect(),
    }
}
fn string_namelist(list: ByteNameList) -> Result<NameList, ToolError> {
    Ok(NameList {
        array: list
            .array
            .into_iter()
            .map(|e| {
                Ok(NameListEntry {
                    name: string_adapter(e.name)?,
                    short_name: e.short_name.map(string_adapter).transpose()?,
                    flags: e.flags,
                })
            })
            .collect::<Result<Vec<_>, ToolError>>()?,
    })
}
/// UTF-8 convenience adapter. An unrepresentable byte truncation is displayed
/// with hexadecimal escapes; get_c_name_bytes preserves the actual C bytes.
pub fn get_c_name(prefix: &str, file: &str, size: usize) -> String {
    byte_display(&get_c_name_bytes(prefix.as_bytes(), file.as_bytes(), size))
}
pub fn find_unique_cname(name: &mut String, size: usize, list: &NameList) {
    let mut raw = name.as_bytes().to_vec();
    find_unique_cname_bytes(&mut raw, size, &byte_namelist(list));
    *name = byte_display(&raw);
}
pub fn get_suffixed_size(text: &str) -> Result<usize, ToolError> {
    get_suffixed_size_bytes(text.as_bytes())
}
pub fn get_short_optarg(
    i: &mut usize,
    opt: u8,
    arg: &str,
    argv: &[String],
) -> Result<String, ToolError> {
    string_adapter(get_short_optarg_bytes(
        i,
        opt,
        arg.as_bytes(),
        &argv
            .iter()
            .map(|s| s.as_bytes().to_vec())
            .collect::<Vec<_>>(),
    )?)
}
/// UTF-8 convenience adapter; returns an explicit error if original C byte
/// truncation would leave a field that cannot be represented by String.
pub fn parse_options(argv: &[String]) -> Result<Options, ToolError> {
    let b = parse_options_bytes(
        &argv
            .iter()
            .map(|s| s.as_bytes().to_vec())
            .collect::<Vec<_>>(),
    )?;
    Ok(Options {
        out_filename: b.out_filename.map(string_adapter).transpose()?,
        output_type: b.output_type,
        cname: b.cname.map(string_adapter).transpose()?,
        feature_bitmap: b.feature_bitmap,
        module: b.module,
        byte_swap: b.byte_swap,
        verbose: b.verbose,
        strip_flags: b.strip_flags,
        use_lto: b.use_lto,
        stack_size: b.stack_size,
        c_ident_prefix: string_adapter(b.c_ident_prefix)?,
        cmodule_list: string_namelist(b.cmodule_list)?,
        dynamic_module_list: string_namelist(b.dynamic_module_list)?,
        optind: b.optind,
    })
}
pub fn exec_cmd(argv: &[String]) -> io::Result<i32> {
    exec_cmd_bytes(
        &argv
            .iter()
            .map(|s| s.as_bytes().to_vec())
            .collect::<Vec<_>>(),
    )
}
pub fn executable_argv(
    c: &CompilerConfig,
    out: &str,
    source: &str,
    lto: bool,
    dynamic: bool,
    exe: &str,
) -> Result<Vec<String>, ToolError> {
    executable_argv_bytes(
        c,
        out.as_bytes(),
        source.as_bytes(),
        lto,
        dynamic,
        exe.as_bytes(),
    )?
    .into_iter()
    .map(string_adapter)
    .collect()
}

include!("qjsc_rust.rs");
// qjsc.c:576..876.
pub fn main_entry() -> i32 {
    let argv = match crate::quickjs_libc::args_bytes() {
        Ok(v) => v,
        Err(e) => {
            eprintln!("qjsc: {e}");
            return 1;
        }
    };
    let config = CompilerConfig::default();
    let legacy = match std::env::var_os("QJSC_BACKEND") {
        Some(value) if value == "rust" => false,
        Some(value) if value == "c" => true,
        _ => option_env!("QJSC_CC").is_some(),
    };
    match run_backend(&argv, &config, !legacy) {
        Ok(code) => code,
        Err(e) => {
            let _ = io::stderr().write_all(&e.message_bytes);
            if e.help {
                print!("{}", help(&config));
            }
            1
        }
    }
}
