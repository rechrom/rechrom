// POSIX adapters for original quickjs-libc.c host algorithms.
const OS_PATH_MAX: usize = libc::PATH_MAX as usize;
include!("quickjs_libc_os_bodies.rs");
include!("quickjs_libc_os_tables.rs");

#[cfg(target_os = "macos")]
extern "C" { fn _NSGetEnviron() -> *mut *mut *mut c_char; }
#[cfg(not(target_os = "macos"))]
extern "C" { static mut environ: *mut *mut c_char; }
unsafe fn os_environ_pointer() -> *mut *mut *mut c_char {
    #[cfg(target_os = "macos")] { _NSGetEnviron() }
    #[cfg(not(target_os = "macos"))] { ptr::addr_of_mut!(environ) }
}
unsafe fn os_execve(filename: *const c_char, argv: *mut *mut c_char, envp: *mut *mut c_char) -> i32 {
    libc::execve(filename, argv.cast(), envp.cast())
}
// glibc 2.34+ supplies this API, although the Rust libc dependency does not
// currently expose its declaration. This is a host OS binding, not a fallback.
#[cfg(all(feature = "host-have-closefrom", target_os = "linux", target_env = "gnu"))]
extern "C" { #[link_name = "closefrom"] fn OS_HOST_CLOSEFROM(fd: i32); }
#[cfg(feature = "host-have-closefrom")]
unsafe fn os_closefrom(fd: i32) {
    #[cfg(all(target_os = "linux", target_env = "gnu"))]
    { OS_HOST_CLOSEFROM(fd); }
    #[cfg(any(target_os = "freebsd", target_os = "netbsd", target_os = "openbsd", target_os = "dragonfly"))]
    { let _ = libc::closefrom(fd); }
}
unsafe extern "C" fn os_signal_handler(number: i32) {
    OS_PENDING_SIGNALS.fetch_or(1u64 << number, Ordering::Relaxed);
}

// C:1879..1899. Real terminal dimensions, including upstream minimum size.
pub unsafe fn js_os_ttyGetWinSize(ctx: *mut JSContext, _this: JSValueConst,
                                 _argc: i32, argv: *mut JSValueConst) -> JSValue {
    let mut fd = 0;
    if JS_ToInt32(ctx, &mut fd, *argv) != 0 { return JS_EXCEPTION; }
    let mut size: libc::winsize = std::mem::zeroed();
    if libc::ioctl(fd, libc::TIOCGWINSZ, &mut size) != 0 || size.ws_col < 4 || size.ws_row < 4 { return JS_NULL; }
    let array = JS_NewArray(ctx);
    if JS_IsException(array) != 0 { return array; }
    JS_DefinePropertyValueUint32(ctx, array, 0, JS_NewInt32(ctx, size.ws_col as i32), JS_PROP_C_W_E);
    JS_DefinePropertyValueUint32(ctx, array, 1, JS_NewInt32(ctx, size.ws_row as i32), JS_PROP_C_W_E);
    array
}
static mut OS_OLD_TTY: libc::termios = unsafe { std::mem::zeroed() };
extern "C" fn term_exit() { unsafe { libc::tcsetattr(0, libc::TCSANOW, ptr::addr_of!(OS_OLD_TTY)); } }
// C:1901..1935. Same process-global restoration and fd=0 atexit behavior.
pub unsafe fn js_os_ttySetRaw(ctx: *mut JSContext, _this: JSValueConst,
                             _argc: i32, argv: *mut JSValueConst) -> JSValue {
    let mut fd = 0;
    if JS_ToInt32(ctx, &mut fd, *argv) != 0 { return JS_EXCEPTION; }
    let mut tty: libc::termios = std::mem::zeroed();
    libc::tcgetattr(fd, &mut tty);
    OS_OLD_TTY = tty;
    tty.c_iflag &= !(libc::IGNBRK | libc::BRKINT | libc::PARMRK | libc::ISTRIP | libc::INLCR | libc::IGNCR | libc::ICRNL | libc::IXON);
    tty.c_oflag |= libc::OPOST;
    tty.c_lflag &= !(libc::ECHO | libc::ECHONL | libc::ICANON | libc::IEXTEN);
    tty.c_cflag &= !(libc::CSIZE | libc::PARENB);
    tty.c_cflag |= libc::CS8;
    tty.c_cc[libc::VMIN] = 1; tty.c_cc[libc::VTIME] = 0;
    libc::tcsetattr(fd, libc::TCSANOW, &tty);
    libc::atexit(term_exit);
    JS_UNDEFINED
}
// C:2791..2883. libc's platform layout supplies real stat data; JS field order
// and numeric constructors follow the source, including millisecond truncation.
pub unsafe fn js_os_stat(ctx: *mut JSContext, _this: JSValueConst,
                         _argc: i32, argv: *mut JSValueConst, is_lstat: i32) -> JSValue {
    let path = JS_ToCString(ctx, *argv);
    if path.is_null() { return JS_EXCEPTION; }
    let mut data: libc::stat = std::mem::zeroed();
    let result = if is_lstat != 0 { libc::lstat(path, &mut data) } else { libc::stat(path, &mut data) };
    let error = if result < 0 { *stdio_errno_pointer() } else { 0 };
    JS_FreeCString(ctx, path);
    let object = if result < 0 { JS_NULL } else {
        let object = JS_NewObject(ctx);
        if JS_IsException(object) != 0 { return JS_EXCEPTION; }
        for (name, value) in [(c"dev",data.st_dev as i64), (c"ino",data.st_ino as i64)] {
            JS_DefinePropertyValueStr(ctx, object, name.as_ptr(), JS_NewInt64(ctx, value), JS_PROP_C_W_E);
        }
        JS_DefinePropertyValueStr(ctx, object, c"mode".as_ptr(), JS_NewInt32(ctx, data.st_mode as i32), JS_PROP_C_W_E);
        for (name,value) in [(c"nlink",data.st_nlink as i64),(c"uid",data.st_uid as i64),(c"gid",data.st_gid as i64),
                            (c"rdev",data.st_rdev as i64),(c"size",data.st_size as i64),(c"blocks",data.st_blocks as i64),
                            (c"atime",data.st_atime.wrapping_mul(1000).wrapping_add(data.st_atime_nsec / 1000000)),
                            (c"mtime",data.st_mtime.wrapping_mul(1000).wrapping_add(data.st_mtime_nsec / 1000000)),
                            (c"ctime",data.st_ctime.wrapping_mul(1000).wrapping_add(data.st_ctime_nsec / 1000000))] {
            JS_DefinePropertyValueStr(ctx, object, name.as_ptr(), JS_NewInt64(ctx, value), JS_PROP_C_W_E);
        }
        object
    };
    make_obj_error(ctx, object, error)
}
// C:4012..4059. Exact module export table and production Worker class.
unsafe fn js_os_init(ctx: *mut JSContext, module: *mut JSModuleDef) -> i32 {
    js_os_enable_poll();
    js_worker_init(ctx, module);
    JS_SetModuleExportList(ctx, module, js_os_funcs.as_ptr(), js_os_funcs.len() as i32)
}
pub unsafe fn js_init_module_os(ctx: *mut JSContext, name: *const c_char) -> *mut JSModuleDef {
    let module = JS_NewCModule(ctx, name, Some(js_os_init));
    if module.is_null() { return module; }
    JS_AddModuleExportList(ctx, module, js_os_funcs.as_ptr(), js_os_funcs.len() as i32);
    JS_AddModuleExport(ctx, module, c"Worker".as_ptr());
    module
}
