//! qjs.c, QuickJS 2026-06-04. Bellard/Gordon, MIT.
//! CLI parsing and the complete tracing allocator. Runtime/REPL wiring is
//! added when the corresponding original quickjs-libc helpers are available.
#[cfg(windows)]
use crate::quickjs_libc_windows::crt as libc;
use quickjs::quickjs_header::*;
use std::{
    alloc::{alloc, dealloc, realloc, Layout},
    ffi::{c_void, CStr},
    io::Write,
};

#[derive(Debug)]
pub struct CliError {
    pub message: String,
    pub help: bool,
    pub exit: i32,
}
impl CliError {
    fn new(message: impl Into<String>, exit: i32) -> Self {
        Self {
            message: message.into(),
            help: false,
            exit,
        }
    }
}
#[derive(Debug)]
pub struct Options {
    pub optind: usize,
    pub expr: Option<String>,
    pub interactive: i32,
    pub dump_memory: i32,
    pub trace_memory: i32,
    pub empty_run: i32,
    pub module: i32,
    pub strict: i32,
    pub load_std: i32,
    pub dump_unhandled_promise_rejection: i32,
    pub memory_limit: usize,
    pub include_list: Vec<String>,
    pub strip_flags: i32,
    pub stack_size: usize,
}
impl Default for Options {
    fn default() -> Self {
        Self {
            optind: 1,
            expr: None,
            interactive: 0,
            dump_memory: 0,
            trace_memory: 0,
            empty_run: 0,
            module: -1,
            strict: 0,
            load_std: 0,
            dump_unhandled_promise_rejection: 1,
            memory_limit: 0,
            include_list: Vec::new(),
            strip_flags: 0,
            stack_size: 0,
        }
    }
}
// qjs.c:263..287. Identical strtod/suffix algorithm is shared with qjsc.c.
pub fn get_suffixed_size(value: &str) -> Result<usize, CliError> {
    crate::qjsc::get_suffixed_size(value).map_err(|e| CliError::new(e.message, 1))
}
// qjs.c:291..312.
pub fn help(version: &str) -> String {
    format!("QuickJS version {version}\nusage: qjs [options] [file [args]]\n-h  --help         list options\n-e  --eval EXPR    evaluate EXPR\n-i  --interactive  go to interactive mode\n-m  --module       load as ES6 module (default=autodetect)\n    --script       load as ES6 script (default=autodetect)\n    --strict       force strict mode\n-I  --include file include an additional file\n    --std          make 'std' and 'os' available to the loaded script\n-T  --trace        trace memory allocation\n-d  --dump         dump the memory usage stats\n    --memory-limit n  limit the memory usage to 'n' bytes (SI suffixes allowed)\n    --stack-size n    limit the stack size to 'n' bytes (SI suffixes allowed)\n    --no-unhandled-rejection  ignore unhandled promise rejections\n-s                    strip all the debug info\n    --strip-source    strip the source code\n-q  --quit         just instantiate the interpreter and quit\n")
}
// qjs.c:314..452, including the original unusual -I attached-argument behavior.
pub fn parse_options(argv: &[String]) -> Result<Options, CliError> {
    let bytes = argv
        .iter()
        .map(|s| s.as_bytes().to_vec())
        .collect::<Vec<_>>();
    let raw = parse_raw_options(&bytes).map_err(|e| CliError {
        message: String::from_utf8_lossy(&e.message).into_owned(),
        help: e.help,
        exit: e.exit,
    })?;
    let mut options = raw.scalar;
    options.expr = raw
        .expr
        .map(|s| String::from_utf8(s).expect("String argv substring remains UTF-8"));
    options.include_list = raw
        .include_list
        .into_iter()
        .map(|s| String::from_utf8(s).expect("String argv remains UTF-8"))
        .collect();
    Ok(options)
}

/// Original platform malloc/free/realloc/usable-size boundary. Supplying a
/// native allocator preserves that platform's usable sizes. Unix defaults to
/// the original OS malloc boundary; rust_allocator provides exact-size Rust
/// allocation. Neither backend calls a C QuickJS implementation.
#[derive(Clone, Copy)]
pub struct TraceAllocator {
    pub malloc: unsafe fn(usize) -> *mut c_void,
    pub free: unsafe fn(*mut c_void),
    pub realloc: unsafe fn(*mut c_void, usize) -> *mut c_void,
    pub usable_size: unsafe fn(*const c_void) -> usize,
}
const ALLOCATION_HEADER: usize = 16;
unsafe fn rust_malloc(size: usize) -> *mut c_void {
    let Some(total) = size.checked_add(ALLOCATION_HEADER) else {
        return std::ptr::null_mut();
    };
    let Ok(layout) = Layout::from_size_align(total, 16) else {
        return std::ptr::null_mut();
    };
    let raw = alloc(layout);
    if raw.is_null() {
        return raw.cast();
    }
    raw.cast::<usize>().write(size);
    raw.add(ALLOCATION_HEADER).cast()
}
unsafe fn rust_usable_size(pointer: *const c_void) -> usize {
    if pointer.is_null() {
        0
    } else {
        pointer
            .cast::<u8>()
            .sub(ALLOCATION_HEADER)
            .cast::<usize>()
            .read()
    }
}
unsafe fn rust_free(pointer: *mut c_void) {
    if pointer.is_null() {
        return;
    }
    let size = rust_usable_size(pointer);
    dealloc(
        pointer.cast::<u8>().sub(ALLOCATION_HEADER),
        Layout::from_size_align_unchecked(size + ALLOCATION_HEADER, 16),
    );
}
unsafe fn rust_realloc(pointer: *mut c_void, size: usize) -> *mut c_void {
    if pointer.is_null() {
        return rust_malloc(size);
    }
    if size == 0 {
        rust_free(pointer);
        return std::ptr::null_mut();
    }
    let Some(total) = size.checked_add(ALLOCATION_HEADER) else {
        return std::ptr::null_mut();
    };
    if Layout::from_size_align(total, 16).is_err() {
        return std::ptr::null_mut();
    }
    let old = rust_usable_size(pointer);
    let raw = realloc(
        pointer.cast::<u8>().sub(ALLOCATION_HEADER),
        Layout::from_size_align_unchecked(old + ALLOCATION_HEADER, 16),
        total,
    );
    if raw.is_null() {
        return raw.cast();
    }
    raw.cast::<usize>().write(size);
    raw.add(ALLOCATION_HEADER).cast()
}
#[cfg(any(unix, windows))]
unsafe fn native_malloc(size: usize) -> *mut c_void {
    libc::malloc(size)
}
#[cfg(any(unix, windows))]
unsafe fn native_free(pointer: *mut c_void) {
    libc::free(pointer);
}
#[cfg(any(unix, windows))]
unsafe fn native_realloc(pointer: *mut c_void, size: usize) -> *mut c_void {
    libc::realloc(pointer, size)
}
#[cfg(any(
    target_os = "macos",
    target_os = "linux",
    target_os = "android",
    target_os = "freebsd",
    target_os = "dragonfly",
    target_os = "windows"
))]
unsafe fn native_usable_size(pointer: *const c_void) -> usize {
    #[cfg(target_os = "macos")]
    {
        libc::malloc_size(pointer)
    }
    #[cfg(any(target_os = "linux", target_os = "android"))]
    {
        libc::malloc_usable_size(pointer.cast_mut())
    }
    #[cfg(any(target_os = "freebsd", target_os = "dragonfly"))]
    {
        libc::malloc_usable_size(pointer)
    }
    #[cfg(windows)]
    {
        libc::_msize(pointer.cast_mut())
    }
}
impl TraceAllocator {
    pub fn rust_allocator() -> Self {
        Self {
            malloc: rust_malloc,
            free: rust_free,
            realloc: rust_realloc,
            usable_size: rust_usable_size,
        }
    }
}
// quickjs.c:2137..2214: the original native allocator policy, injected by the
// CLI host through JS_NewRuntime2. The engine's portable Rust default remains
// independent of native usable-size APIs and native allocation/free bindings.
unsafe fn host_def_malloc(s: *mut JSMallocState, size: usize) -> *mut c_void {
    assert_ne!(size, 0);
    if (*s).malloc_size.wrapping_add(size) > (*s).malloc_limit {
        return std::ptr::null_mut();
    }
    let p = native_malloc(size);
    if p.is_null() {
        return p;
    }
    (*s).malloc_count = (*s).malloc_count.wrapping_add(1);
    (*s).malloc_size = (*s)
        .malloc_size
        .wrapping_add(native_usable_size(p))
        .wrapping_add(if cfg!(target_vendor = "apple") { 0 } else { 8 });
    p
}
unsafe fn host_def_free(s: *mut JSMallocState, p: *mut c_void) {
    if p.is_null() {
        return;
    }
    (*s).malloc_count = (*s).malloc_count.wrapping_sub(1);
    (*s).malloc_size = (*s)
        .malloc_size
        .wrapping_sub(native_usable_size(p))
        .wrapping_sub(if cfg!(target_vendor = "apple") { 0 } else { 8 });
    native_free(p);
}
unsafe fn host_def_realloc(s: *mut JSMallocState, p: *mut c_void, size: usize) -> *mut c_void {
    if p.is_null() {
        if size == 0 {
            return std::ptr::null_mut();
        }
        return host_def_malloc(s, size);
    }
    let old_size = native_usable_size(p);
    if size == 0 {
        host_def_free(s, p);
        return std::ptr::null_mut();
    }
    if (*s).malloc_size.wrapping_add(size).wrapping_sub(old_size) > (*s).malloc_limit {
        return std::ptr::null_mut();
    }
    let q = native_realloc(p, size);
    if q.is_null() {
        return q;
    }
    (*s).malloc_size = (*s)
        .malloc_size
        .wrapping_add(native_usable_size(q))
        .wrapping_sub(old_size);
    q
}
fn native_default_mf() -> JSMallocFunctions {
    JSMallocFunctions {
        js_malloc: Some(host_def_malloc),
        js_free: Some(host_def_free),
        js_realloc: Some(host_def_realloc),
        js_malloc_usable_size: Some(native_usable_size),
    }
}
unsafe fn new_cli_runtime() -> *mut JSRuntime {
    quickjs::quickjs::JS_NewRuntime2(&native_default_mf(), std::ptr::null_mut())
}
impl Default for TraceAllocator {
    fn default() -> Self {
        #[cfg(any(unix, windows))]
        {
            Self {
                malloc: native_malloc,
                free: native_free,
                realloc: native_realloc,
                usable_size: native_usable_size,
            }
        }
        #[cfg(not(any(unix, windows)))]
        {
            Self::rust_allocator()
        }
    }
}
pub struct TraceMallocData {
    pub base: *mut u8,
    pub allocator: TraceAllocator,
    pub malloc_overhead: usize,
    pub write: fn(*mut c_void, &[u8]),
    pub write_opaque: *mut c_void,
}
fn stdout_write(_opaque: *mut c_void, bytes: &[u8]) {
    let _ = std::io::stdout().write_all(bytes);
}
impl Default for TraceMallocData {
    fn default() -> Self {
        Self {
            base: std::ptr::null_mut(),
            allocator: TraceAllocator::default(),
            malloc_overhead: if cfg!(target_os = "macos") { 0 } else { 8 },
            write: stdout_write,
            write_opaque: std::ptr::null_mut(),
        }
    }
}
pub enum TracePrintArg {
    Pointer(*const c_void),
    Size(usize),
}
// qjs.c:129..133. C's cross-allocation pointer subtraction is an address offset.
pub fn js_trace_malloc_ptr_offset(pointer: *const c_void, data: &TraceMallocData) -> u64 {
    (pointer as usize).wrapping_sub(data.base as usize) as u64
}
// qjs.c:136..153. The explicit allocator implements the platform boundary.
pub unsafe fn js_trace_malloc_usable_size(data: &TraceMallocData, pointer: *const c_void) -> usize {
    (data.allocator.usable_size)(pointer)
}
// qjs.c:156..190. Original %p/%zd interpreter; typed varargs replace C varargs.
pub unsafe fn js_trace_malloc_printf(
    state: *mut JSMallocState,
    fmt: &CStr,
    args: &[TracePrintArg],
) {
    let data = &*(*state).opaque.cast::<TraceMallocData>();
    let mut output = Vec::new();
    let bytes = fmt.to_bytes();
    let mut pos = 0;
    let mut arg = 0;
    while pos < bytes.len() {
        let c = bytes[pos];
        pos += 1;
        if c == b'%' {
            if bytes.get(pos) == Some(&b'p') {
                let TracePrintArg::Pointer(pointer) = args[arg] else {
                    unreachable!("typed %p contract")
                };
                arg += 1;
                if pointer.is_null() {
                    output.extend_from_slice(b"NULL");
                } else {
                    output.extend_from_slice(
                        format!(
                            "H{:+06}.{}",
                            js_trace_malloc_ptr_offset(pointer, data) as i64,
                            js_trace_malloc_usable_size(data, pointer)
                        )
                        .as_bytes(),
                    );
                }
                pos += 1;
                continue;
            }
            if bytes.get(pos) == Some(&b'z') && bytes.get(pos + 1) == Some(&b'd') {
                let TracePrintArg::Size(size) = args[arg] else {
                    unreachable!("typed %zd contract")
                };
                arg += 1;
                output.extend_from_slice(format!("{}", size as isize).as_bytes());
                pos += 2;
                continue;
            }
        }
        output.push(c);
    }
    (data.write)(data.write_opaque, &output);
}
// qjs.c:192..195.
pub unsafe fn js_trace_malloc_init(data: &mut TraceMallocData) {
    data.base = (data.allocator.malloc)(8).cast();
    (data.allocator.free)(data.base.cast());
}
// qjs.c:197..213.
pub unsafe fn js_trace_malloc(state: *mut JSMallocState, size: usize) -> *mut c_void {
    assert!(size != 0);
    if (*state).malloc_size.wrapping_add(size) > (*state).malloc_limit {
        return std::ptr::null_mut();
    }
    let data = &*(*state).opaque.cast::<TraceMallocData>();
    let pointer = (data.allocator.malloc)(size);
    js_trace_malloc_printf(
        state,
        c"A %zd -> %p\n",
        &[TracePrintArg::Size(size), TracePrintArg::Pointer(pointer)],
    );
    if !pointer.is_null() {
        (*state).malloc_count = (*state).malloc_count.wrapping_add(1);
        (*state).malloc_size = (*state)
            .malloc_size
            .wrapping_add(js_trace_malloc_usable_size(data, pointer))
            .wrapping_add(data.malloc_overhead);
    }
    pointer
}
// qjs.c:215..224.
pub unsafe fn js_trace_free(state: *mut JSMallocState, pointer: *mut c_void) {
    if pointer.is_null() {
        return;
    }
    let data = &*(*state).opaque.cast::<TraceMallocData>();
    js_trace_malloc_printf(state, c"F %p\n", &[TracePrintArg::Pointer(pointer)]);
    (*state).malloc_count = (*state).malloc_count.wrapping_sub(1);
    (*state).malloc_size = (*state)
        .malloc_size
        .wrapping_sub(js_trace_malloc_usable_size(data, pointer))
        .wrapping_sub(data.malloc_overhead);
    (data.allocator.free)(pointer);
}
// qjs.c:226..254.
pub unsafe fn js_trace_realloc(
    state: *mut JSMallocState,
    pointer: *mut c_void,
    size: usize,
) -> *mut c_void {
    if pointer.is_null() {
        return if size == 0 {
            std::ptr::null_mut()
        } else {
            js_trace_malloc(state, size)
        };
    }
    let data = &*(*state).opaque.cast::<TraceMallocData>();
    let old_size = js_trace_malloc_usable_size(data, pointer);
    if size == 0 {
        js_trace_malloc_printf(
            state,
            c"R %zd %p\n",
            &[TracePrintArg::Size(size), TracePrintArg::Pointer(pointer)],
        );
        (*state).malloc_count = (*state).malloc_count.wrapping_sub(1);
        (*state).malloc_size = (*state)
            .malloc_size
            .wrapping_sub(old_size)
            .wrapping_sub(data.malloc_overhead);
        (data.allocator.free)(pointer);
        return std::ptr::null_mut();
    }
    if (*state)
        .malloc_size
        .wrapping_add(size)
        .wrapping_sub(old_size)
        > (*state).malloc_limit
    {
        return std::ptr::null_mut();
    }
    js_trace_malloc_printf(
        state,
        c"R %zd %p",
        &[TracePrintArg::Size(size), TracePrintArg::Pointer(pointer)],
    );
    let result = (data.allocator.realloc)(pointer, size);
    js_trace_malloc_printf(state, c" -> %p\n", &[TracePrintArg::Pointer(result)]);
    if !result.is_null() {
        (*state).malloc_size = (*state)
            .malloc_size
            .wrapping_add(js_trace_malloc_usable_size(data, result))
            .wrapping_sub(old_size);
    }
    result
}
// qjs.c:256..261. Same table, with the actual injected usable-size function.
pub fn trace_mf(data: &TraceMallocData) -> JSMallocFunctions {
    JSMallocFunctions {
        js_malloc: Some(js_trace_malloc),
        js_free: Some(js_trace_free),
        js_realloc: Some(js_trace_realloc),
        js_malloc_usable_size: Some(data.allocator.usable_size),
    }
}

include!("qjs_raw.rs");
#[cfg(any(unix, windows))]
include!("qjs_eval.rs");

#[cfg(any(unix, windows))]
include!("qjs_runtime.rs");
