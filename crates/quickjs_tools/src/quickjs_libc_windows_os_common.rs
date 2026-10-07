// Same official C CFG; native functions map to Windows CRT.
pub unsafe fn js_os_close(
    mut ctx: *mut JSContext,
    mut this_val: JSValue,
    mut argc: i32,
    mut argv: *mut JSValue,
) -> JSValue {
    let mut fd: i32 = core::mem::zeroed();
    let mut ret: i32 = core::mem::zeroed();
    let mut vm_block: usize = 4;
    loop {
        match vm_block {
            // C line ?
            0 => {
                std::process::abort();
            }
            // C line 1772
            1 => {
                return JS_NewInt32(ctx, ret);
            }
            // C line 1771
            2 => {
                let _ = {
                    let assigned = ((js_get_errno(((libc::close(fd)) as isize))) as i32);
                    ret = assigned;
                    assigned
                };
                vm_block = 1;
                continue;
            }
            // C line 1770
            3 => {
                return JSValue {
                    u: JSValueUnion {
                        uint64: (((0 as i32) as u32) as u64),
                    },
                    tag: ((JS_TAG_EXCEPTION as i32) as i64),
                };
            }
            // C line 1769
            4 => {
                vm_block = if (JS_ToInt32(
                    ctx,
                    core::ptr::addr_of_mut!(fd),
                    *(argv).offset((0 as i32) as isize),
                )) != 0
                {
                    3
                } else {
                    2
                };
                continue;
            }
            _ => std::process::abort(),
        }
    }
}
// Same official C CFG; native functions map to Windows CRT.
pub unsafe fn js_os_seek(
    mut ctx: *mut JSContext,
    mut this_val: JSValue,
    mut argc: i32,
    mut argv: *mut JSValue,
) -> JSValue {
    let mut fd: i32 = core::mem::zeroed();
    let mut whence: i32 = core::mem::zeroed();
    let mut pos: i64 = core::mem::zeroed();
    let mut ret: i64 = core::mem::zeroed();
    let mut is_bigint: i32 = core::mem::zeroed();
    let mut vm_block: usize = 13;
    loop {
        match vm_block {
            // C line ?
            0 => {
                std::process::abort();
            }
            // C line 1793
            1 => {
                return JS_NewBigInt64(ctx, ret);
            }
            // C line 1795
            2 => {
                return JS_NewInt64(ctx, ret);
            }
            // C line 1792
            3 => {
                vm_block = if (is_bigint) != 0 { 1 } else { 2 };
                continue;
            }
            // C line 1791
            4 => {
                let _ = {
                    let assigned = (((*(stdio_errno_pointer())).wrapping_neg()) as i64);
                    ret = assigned;
                    assigned
                };
                vm_block = 3;
                continue;
            }
            // C line 1790
            5 => {
                vm_block = if (((ret) == (((1 as i32).wrapping_neg()) as i64)) as i32) != 0 {
                    4
                } else {
                    3
                };
                continue;
            }
            // C line 1789
            6 => {
                let _ = {
                    let assigned = libc::lseek(fd, pos, whence);
                    ret = assigned;
                    assigned
                };
                vm_block = 5;
                continue;
            }
            // C line 1788
            7 => {
                return JSValue {
                    u: JSValueUnion {
                        uint64: (((0 as i32) as u32) as u64),
                    },
                    tag: ((JS_TAG_EXCEPTION as i32) as i64),
                };
            }
            // C line 1787
            8 => {
                vm_block = if (JS_ToInt32(
                    ctx,
                    core::ptr::addr_of_mut!(whence),
                    *(argv).offset((2 as i32) as isize),
                )) != 0
                {
                    7
                } else {
                    6
                };
                continue;
            }
            // C line 1786
            9 => {
                return JSValue {
                    u: JSValueUnion {
                        uint64: (((0 as i32) as u32) as u64),
                    },
                    tag: ((JS_TAG_EXCEPTION as i32) as i64),
                };
            }
            // C line 1785
            10 => {
                vm_block = if (JS_ToInt64Ext(
                    ctx,
                    core::ptr::addr_of_mut!(pos),
                    *(argv).offset((1 as i32) as isize),
                )) != 0
                {
                    9
                } else {
                    8
                };
                continue;
            }
            // C line 1784
            11 => {
                let _ = {
                    let assigned = JS_IsBigInt(ctx, *(argv).offset((1 as i32) as isize));
                    is_bigint = assigned;
                    assigned
                };
                vm_block = 10;
                continue;
            }
            // C line 1783
            12 => {
                return JSValue {
                    u: JSValueUnion {
                        uint64: (((0 as i32) as u32) as u64),
                    },
                    tag: ((JS_TAG_EXCEPTION as i32) as i64),
                };
            }
            // C line 1782
            13 => {
                vm_block = if (JS_ToInt32(
                    ctx,
                    core::ptr::addr_of_mut!(fd),
                    *(argv).offset((0 as i32) as isize),
                )) != 0
                {
                    12
                } else {
                    11
                };
                continue;
            }
            _ => std::process::abort(),
        }
    }
}
// Same official C CFG; native functions map to Windows CRT.
pub unsafe fn js_os_read_write(
    mut ctx: *mut JSContext,
    mut this_val: JSValue,
    mut argc: i32,
    mut argv: *mut JSValue,
    mut magic: i32,
) -> JSValue {
    let mut fd: i32 = core::mem::zeroed();
    let mut pos: u64 = core::mem::zeroed();
    let mut len: u64 = core::mem::zeroed();
    let mut size: usize = core::mem::zeroed();
    let mut ret: isize = core::mem::zeroed();
    let mut buf: *mut u8 = core::mem::zeroed();
    let mut vm_block: usize = 15;
    loop {
        match vm_block {
            // C line ?
            0 => {
                std::process::abort();
            }
            // C line 1822
            1 => {
                return JS_NewInt64(ctx, ((ret) as i64));
            }
            // C line 1819
            2 => {
                let _ = {
                    let assigned = js_get_errno(libc::write(
                        fd,
                        (((buf).offset(((pos) as isize))) as *const c_void),
                        ((len) as usize),
                    ));
                    ret = assigned;
                    assigned
                };
                vm_block = 1;
                continue;
            }
            // C line 1821
            3 => {
                let _ = {
                    let assigned = js_get_errno(libc::read(
                        fd,
                        (((buf).offset(((pos) as isize))) as *mut c_void),
                        ((len) as usize),
                    ));
                    ret = assigned;
                    assigned
                };
                vm_block = 1;
                continue;
            }
            // C line 1818
            4 => {
                vm_block = if (magic) != 0 { 2 } else { 3 };
                continue;
            }
            // C line 1817
            5 => {
                return JS_ThrowRangeError(ctx, c"read/write array buffer overflow".as_ptr());
            }
            // C line 1816
            6 => {
                vm_block = if ((((pos).wrapping_add(len)) > ((size) as u64)) as i32) != 0 {
                    5
                } else {
                    4
                };
                continue;
            }
            // C line 1815
            7 => {
                return JSValue {
                    u: JSValueUnion {
                        uint64: (((0 as i32) as u32) as u64),
                    },
                    tag: ((JS_TAG_EXCEPTION as i32) as i64),
                };
            }
            // C line 1814
            8 => {
                vm_block = if (!(!(buf).is_null()) as i32) != 0 {
                    7
                } else {
                    6
                };
                continue;
            }
            // C line 1813
            9 => {
                let _ = {
                    let assigned = JS_GetArrayBuffer(
                        ctx,
                        core::ptr::addr_of_mut!(size),
                        *(argv).offset((1 as i32) as isize),
                    );
                    buf = assigned;
                    assigned
                };
                vm_block = 8;
                continue;
            }
            // C line 1812
            10 => {
                return JSValue {
                    u: JSValueUnion {
                        uint64: (((0 as i32) as u32) as u64),
                    },
                    tag: ((JS_TAG_EXCEPTION as i32) as i64),
                };
            }
            // C line 1811
            11 => {
                vm_block = if (JS_ToIndex(
                    ctx,
                    core::ptr::addr_of_mut!(len),
                    *(argv).offset((3 as i32) as isize),
                )) != 0
                {
                    10
                } else {
                    9
                };
                continue;
            }
            // C line 1810
            12 => {
                return JSValue {
                    u: JSValueUnion {
                        uint64: (((0 as i32) as u32) as u64),
                    },
                    tag: ((JS_TAG_EXCEPTION as i32) as i64),
                };
            }
            // C line 1809
            13 => {
                vm_block = if (JS_ToIndex(
                    ctx,
                    core::ptr::addr_of_mut!(pos),
                    *(argv).offset((2 as i32) as isize),
                )) != 0
                {
                    12
                } else {
                    11
                };
                continue;
            }
            // C line 1808
            14 => {
                return JSValue {
                    u: JSValueUnion {
                        uint64: (((0 as i32) as u32) as u64),
                    },
                    tag: ((JS_TAG_EXCEPTION as i32) as i64),
                };
            }
            // C line 1807
            15 => {
                vm_block = if (JS_ToInt32(
                    ctx,
                    core::ptr::addr_of_mut!(fd),
                    *(argv).offset((0 as i32) as isize),
                )) != 0
                {
                    14
                } else {
                    13
                };
                continue;
            }
            _ => std::process::abort(),
        }
    }
}
// Same official C CFG; native functions map to Windows CRT.
pub unsafe fn js_os_isatty(
    mut ctx: *mut JSContext,
    mut this_val: JSValue,
    mut argc: i32,
    mut argv: *mut JSValue,
) -> JSValue {
    let mut fd: i32 = core::mem::zeroed();
    let mut vm_block: usize = 3;
    loop {
        match vm_block {
            // C line ?
            0 => {
                std::process::abort();
            }
            // C line 1831
            1 => {
                return JS_NewBool(ctx, libc::isatty(fd));
            }
            // C line 1830
            2 => {
                return JSValue {
                    u: JSValueUnion {
                        uint64: (((0 as i32) as u32) as u64),
                    },
                    tag: ((JS_TAG_EXCEPTION as i32) as i64),
                };
            }
            // C line 1829
            3 => {
                vm_block = if (JS_ToInt32(
                    ctx,
                    core::ptr::addr_of_mut!(fd),
                    *(argv).offset((0 as i32) as isize),
                )) != 0
                {
                    2
                } else {
                    1
                };
                continue;
            }
            _ => std::process::abort(),
        }
    }
}
// Same official C CFG; native functions map to Windows CRT.
pub unsafe fn js_os_rename(
    mut ctx: *mut JSContext,
    mut this_val: JSValue,
    mut argc: i32,
    mut argv: *mut JSValue,
) -> JSValue {
    let mut oldpath: *const c_char = core::mem::zeroed();
    let mut newpath: *const c_char = core::mem::zeroed();
    let mut ret: i32 = core::mem::zeroed();
    let mut vm_block: usize = 11;
    loop {
        match vm_block {
            // C line ?
            0 => {
                std::process::abort();
            }
            // C line 1982
            1 => {
                return JS_NewInt32(ctx, ret);
            }
            // C line 1981
            2 => {
                let _ = JS_FreeCString(ctx, newpath);
                vm_block = 1;
                continue;
            }
            // C line 1980
            3 => {
                let _ = JS_FreeCString(ctx, oldpath);
                vm_block = 2;
                continue;
            }
            // C line 1979
            4 => {
                let _ = {
                    let assigned =
                        ((js_get_errno(((libc::rename(oldpath, newpath)) as isize))) as i32);
                    ret = assigned;
                    assigned
                };
                vm_block = 3;
                continue;
            }
            // C line 1977
            5 => {
                return JSValue {
                    u: JSValueUnion {
                        uint64: (((0 as i32) as u32) as u64),
                    },
                    tag: ((JS_TAG_EXCEPTION as i32) as i64),
                };
            }
            // C line 1976
            6 => {
                let _ = JS_FreeCString(ctx, oldpath);
                vm_block = 5;
                continue;
            }
            // C line 1975
            7 => {
                vm_block = if (!(!(newpath).is_null()) as i32) != 0 {
                    6
                } else {
                    4
                };
                continue;
            }
            // C line 1974
            8 => {
                let _ = {
                    let assigned = JS_ToCString(ctx, *(argv).offset((1 as i32) as isize));
                    newpath = assigned;
                    assigned
                };
                vm_block = 7;
                continue;
            }
            // C line 1973
            9 => {
                return JSValue {
                    u: JSValueUnion {
                        uint64: (((0 as i32) as u32) as u64),
                    },
                    tag: ((JS_TAG_EXCEPTION as i32) as i64),
                };
            }
            // C line 1972
            10 => {
                vm_block = if (!(!(oldpath).is_null()) as i32) != 0 {
                    9
                } else {
                    8
                };
                continue;
            }
            // C line 1971
            11 => {
                let _ = {
                    let assigned = JS_ToCString(ctx, *(argv).offset((0 as i32) as isize));
                    oldpath = assigned;
                    assigned
                };
                vm_block = 10;
                continue;
            }
            _ => std::process::abort(),
        }
    }
}
// Same official C CFG; native functions map to Windows CRT.
pub unsafe fn is_main_thread(mut rt: *mut JSRuntime) -> i32 {
    let mut ts: *mut JSThreadState = core::mem::zeroed();
    let mut vm_block: usize = 2;
    loop {
        match vm_block {
            // C line ?
            0 => {
                std::process::abort();
            }
            // C line 1988
            1 => {
                return (!(!((*(ts)).recv_pipe).is_null()) as i32);
            }
            // C line 1987
            2 => {
                ts = ((JS_GetRuntimeOpaque(rt)) as *mut JSThreadState);
                vm_block = 1;
                continue;
            }
            _ => std::process::abort(),
        }
    }
}
// Same official C CFG; native functions map to Windows CRT.
pub unsafe fn find_rh(mut ts: *mut JSThreadState, mut fd: i32) -> *mut JSOSRWHandler {
    let mut rh: *mut JSOSRWHandler = core::mem::zeroed();
    let mut el: *mut list_head = core::mem::zeroed();
    let mut vm_block: usize = 7;
    loop {
        match vm_block {
            // C line ?
            0 => {
                std::process::abort();
            }
            // C line 2001
            1 => {
                return core::ptr::null_mut::<JSOSRWHandler>();
            }
            // C line 1996
            2 => {
                vm_block =
                    if (((el) != (core::ptr::addr_of_mut!((*(ts)).os_rw_handlers))) as i32) != 0 {
                        6
                    } else {
                        1
                    };
                continue;
            }
            // C line ?
            3 => {
                let _ = {
                    let assigned = (*(el)).next;
                    el = assigned;
                    assigned
                };
                vm_block = 2;
                continue;
            }
            // C line 1999
            4 => {
                return rh;
            }
            // C line 1998
            5 => {
                vm_block = if ((((*(rh)).fd) == (fd)) as i32) != 0 {
                    4
                } else {
                    3
                };
                continue;
            }
            // C line 1997
            6 => {
                let _ = {
                    let assigned = ((((el) as *mut u8)
                        .offset(-((core::mem::offset_of!(JSOSRWHandler, link) as usize) as isize)))
                        as *mut JSOSRWHandler);
                    rh = assigned;
                    assigned
                };
                vm_block = 5;
                continue;
            }
            // C line ?
            7 => {
                let _ = {
                    let assigned = (*(core::ptr::addr_of_mut!((*(ts)).os_rw_handlers))).next;
                    el = assigned;
                    assigned
                };
                vm_block = 2;
                continue;
            }
            _ => std::process::abort(),
        }
    }
}
// Same official C CFG; native functions map to Windows CRT.
pub unsafe fn js_os_setReadHandler(
    mut ctx: *mut JSContext,
    mut this_val: JSValue,
    mut argc: i32,
    mut argv: *mut JSValue,
    mut magic: i32,
) -> JSValue {
    let mut rt: *mut JSRuntime = core::mem::zeroed();
    let mut ts: *mut JSThreadState = core::mem::zeroed();
    let mut rh: *mut JSOSRWHandler = core::mem::zeroed();
    let mut fd: i32 = core::mem::zeroed();
    let mut func: JSValue = core::mem::zeroed();
    let mut vm_block: usize = 26;
    loop {
        match vm_block {
            // C line ?
            0 => {
                std::process::abort();
            }
            // C line 2053
            1 => {
                return JSValue {
                    u: JSValueUnion {
                        uint64: (((0 as i32) as u32) as u64),
                    },
                    tag: ((JS_TAG_UNDEFINED as i32) as i64),
                };
            }
            // C line 2034
            2 => {
                let _ = free_rw_handler(JS_GetRuntime(ctx), rh);
                vm_block = 1;
                continue;
            }
            // C line 2031
            3 => {
                vm_block = if ((((JS_IsNull(
                    *(((*(rh)).rw_func).as_mut_ptr()).offset((0 as i32) as isize),
                )) != 0)
                    && ((JS_IsNull(*(((*(rh)).rw_func).as_mut_ptr()).offset((1 as i32) as isize)))
                        != 0)) as i32)
                    != 0
                {
                    2
                } else {
                    1
                };
                continue;
            }
            // C line 2030
            4 => {
                let _ = {
                    let assigned = JSValue {
                        u: JSValueUnion {
                            uint64: (((0 as i32) as u32) as u64),
                        },
                        tag: ((JS_TAG_NULL as i32) as i64),
                    };
                    *(((*(rh)).rw_func).as_mut_ptr()).offset((magic) as isize) = assigned;
                    assigned
                };
                vm_block = 3;
                continue;
            }
            // C line 2029
            5 => {
                let _ = JS_FreeValue(
                    ctx,
                    *(((*(rh)).rw_func).as_mut_ptr()).offset((magic) as isize),
                );
                vm_block = 4;
                continue;
            }
            // C line 2028
            6 => {
                vm_block = if !(rh).is_null() { 5 } else { 1 };
                continue;
            }
            // C line 2027
            7 => {
                let _ = {
                    let assigned = find_rh(ts, fd);
                    rh = assigned;
                    assigned
                };
                vm_block = 6;
                continue;
            }
            // C line 2051
            8 => {
                let _ = {
                    let assigned = JS_DupValue(ctx, func);
                    *(((*(rh)).rw_func).as_mut_ptr()).offset((magic) as isize) = assigned;
                    assigned
                };
                vm_block = 1;
                continue;
            }
            // C line 2050
            9 => {
                let _ = JS_FreeValue(
                    ctx,
                    *(((*(rh)).rw_func).as_mut_ptr()).offset((magic) as isize),
                );
                vm_block = 8;
                continue;
            }
            // C line 2048
            10 => {
                let _ = list_add_tail(
                    core::ptr::addr_of_mut!((*(rh)).link),
                    core::ptr::addr_of_mut!((*(ts)).os_rw_handlers),
                );
                vm_block = 9;
                continue;
            }
            // C line 2047
            11 => {
                let _ = {
                    let assigned = JSValue {
                        u: JSValueUnion {
                            uint64: (((0 as i32) as u32) as u64),
                        },
                        tag: ((JS_TAG_NULL as i32) as i64),
                    };
                    *(((*(rh)).rw_func).as_mut_ptr()).offset((1 as i32) as isize) = assigned;
                    assigned
                };
                vm_block = 10;
                continue;
            }
            // C line 2046
            12 => {
                let _ = {
                    let assigned = JSValue {
                        u: JSValueUnion {
                            uint64: (((0 as i32) as u32) as u64),
                        },
                        tag: ((JS_TAG_NULL as i32) as i64),
                    };
                    *(((*(rh)).rw_func).as_mut_ptr()).offset((0 as i32) as isize) = assigned;
                    assigned
                };
                vm_block = 11;
                continue;
            }
            // C line 2045
            13 => {
                let _ = {
                    let assigned = fd;
                    (*(rh)).fd = assigned;
                    assigned
                };
                vm_block = 12;
                continue;
            }
            // C line 2044
            14 => {
                return JSValue {
                    u: JSValueUnion {
                        uint64: (((0 as i32) as u32) as u64),
                    },
                    tag: ((JS_TAG_EXCEPTION as i32) as i64),
                };
            }
            // C line 2043
            15 => {
                vm_block = if (!(!(rh).is_null()) as i32) != 0 {
                    14
                } else {
                    13
                };
                continue;
            }
            // C line 2042
            16 => {
                let _ = {
                    let assigned =
                        ((js_mallocz(ctx, (core::mem::size_of::<JSOSRWHandler>() as usize)))
                            as *mut JSOSRWHandler);
                    rh = assigned;
                    assigned
                };
                vm_block = 15;
                continue;
            }
            // C line 2041
            17 => {
                vm_block = if (!(!(rh).is_null()) as i32) != 0 {
                    16
                } else {
                    9
                };
                continue;
            }
            // C line 2040
            18 => {
                let _ = {
                    let assigned = find_rh(ts, fd);
                    rh = assigned;
                    assigned
                };
                vm_block = 17;
                continue;
            }
            // C line 2039
            19 => {
                return JS_ThrowTypeError(ctx, c"not a function".as_ptr());
            }
            // C line 2038
            20 => {
                vm_block = if (!((JS_IsFunction(ctx, func)) != 0) as i32) != 0 {
                    19
                } else {
                    18
                };
                continue;
            }
            // C line 2026
            21 => {
                vm_block = if (JS_IsNull(func)) != 0 { 7 } else { 20 };
                continue;
            }
            // C line 2025
            22 => {
                let _ = {
                    let assigned = *(argv).offset((1 as i32) as isize);
                    func = assigned;
                    assigned
                };
                vm_block = 21;
                continue;
            }
            // C line 2024
            23 => {
                return JSValue {
                    u: JSValueUnion {
                        uint64: (((0 as i32) as u32) as u64),
                    },
                    tag: ((JS_TAG_EXCEPTION as i32) as i64),
                };
            }
            // C line 2023
            24 => {
                vm_block = if (JS_ToInt32(
                    ctx,
                    core::ptr::addr_of_mut!(fd),
                    *(argv).offset((0 as i32) as isize),
                )) != 0
                {
                    23
                } else {
                    22
                };
                continue;
            }
            // C line 2018
            25 => {
                ts = ((JS_GetRuntimeOpaque(rt)) as *mut JSThreadState);
                vm_block = 24;
                continue;
            }
            // C line 2017
            26 => {
                rt = JS_GetRuntime(ctx);
                vm_block = 25;
                continue;
            }
            _ => std::process::abort(),
        }
    }
}
// Same official C CFG; native functions map to Windows CRT.
pub unsafe fn find_sh(mut ts: *mut JSThreadState, mut sig_num: i32) -> *mut JSOSSignalHandler {
    let mut sh: *mut JSOSSignalHandler = core::mem::zeroed();
    let mut el: *mut list_head = core::mem::zeroed();
    let mut vm_block: usize = 7;
    loop {
        match vm_block {
            // C line ?
            0 => {
                std::process::abort();
            }
            // C line 2065
            1 => {
                return core::ptr::null_mut::<JSOSSignalHandler>();
            }
            // C line 2060
            2 => {
                vm_block = if (((el) != (core::ptr::addr_of_mut!((*(ts)).os_signal_handlers)))
                    as i32)
                    != 0
                {
                    6
                } else {
                    1
                };
                continue;
            }
            // C line ?
            3 => {
                let _ = {
                    let assigned = (*(el)).next;
                    el = assigned;
                    assigned
                };
                vm_block = 2;
                continue;
            }
            // C line 2063
            4 => {
                return sh;
            }
            // C line 2062
            5 => {
                vm_block = if ((((*(sh)).sig_num) == (sig_num)) as i32) != 0 {
                    4
                } else {
                    3
                };
                continue;
            }
            // C line 2061
            6 => {
                let _ = {
                    let assigned = ((((el) as *mut u8).offset(
                        -((core::mem::offset_of!(JSOSSignalHandler, link) as usize) as isize),
                    )) as *mut JSOSSignalHandler);
                    sh = assigned;
                    assigned
                };
                vm_block = 5;
                continue;
            }
            // C line ?
            7 => {
                let _ = {
                    let assigned = (*(core::ptr::addr_of_mut!((*(ts)).os_signal_handlers))).next;
                    el = assigned;
                    assigned
                };
                vm_block = 2;
                continue;
            }
            _ => std::process::abort(),
        }
    }
}
// Same official C CFG; native functions map to Windows CRT.
pub unsafe fn js_os_signal(
    mut ctx: *mut JSContext,
    mut this_val: JSValue,
    mut argc: i32,
    mut argv: *mut JSValue,
) -> JSValue {
    let mut rt: *mut JSRuntime = core::mem::zeroed();
    let mut ts: *mut JSThreadState = core::mem::zeroed();
    let mut sh: *mut JSOSSignalHandler = core::mem::zeroed();
    let mut sig_num: u32 = core::mem::zeroed();
    let mut func: JSValue = core::mem::zeroed();
    let mut handler: usize = core::mem::zeroed();
    let mut vm_block: usize = 30;
    loop {
        match vm_block {
            // C line ?
            0 => {
                std::process::abort();
            }
            // C line 2128
            1 => {
                return JSValue {
                    u: JSValueUnion {
                        uint64: (((0 as i32) as u32) as u64),
                    },
                    tag: ((JS_TAG_UNDEFINED as i32) as i64),
                };
            }
            // C line 2112
            2 => {
                let _ = libc::signal(((sig_num) as i32), handler);
                vm_block = 1;
                continue;
            }
            // C line 2109
            3 => {
                let _ = {
                    let assigned = ((libc::SIG_DFL as i32) as usize);
                    handler = assigned;
                    assigned
                };
                vm_block = 2;
                continue;
            }
            // C line 2111
            4 => {
                let _ = {
                    let assigned = ((libc::SIG_IGN as i32) as usize);
                    handler = assigned;
                    assigned
                };
                vm_block = 2;
                continue;
            }
            // C line 2108
            5 => {
                vm_block = if (JS_IsNull(func)) != 0 { 3 } else { 4 };
                continue;
            }
            // C line 2106
            6 => {
                let _ = free_sh(JS_GetRuntime(ctx), sh);
                vm_block = 5;
                continue;
            }
            // C line 2105
            7 => {
                vm_block = if !(sh).is_null() { 6 } else { 5 };
                continue;
            }
            // C line 2104
            8 => {
                let _ = {
                    let assigned = find_sh(ts, ((sig_num) as i32));
                    sh = assigned;
                    assigned
                };
                vm_block = 7;
                continue;
            }
            // C line 2126
            9 => {
                let _ = libc::signal(
                    ((sig_num) as i32),
                    (os_signal_handler as *const () as usize),
                );
                vm_block = 1;
                continue;
            }
            // C line 2125
            10 => {
                let _ = {
                    let assigned = JS_DupValue(ctx, func);
                    (*(sh)).func = assigned;
                    assigned
                };
                vm_block = 9;
                continue;
            }
            // C line 2124
            11 => {
                let _ = JS_FreeValue(ctx, (*(sh)).func);
                vm_block = 10;
                continue;
            }
            // C line 2122
            12 => {
                let _ = list_add_tail(
                    core::ptr::addr_of_mut!((*(sh)).link),
                    core::ptr::addr_of_mut!((*(ts)).os_signal_handlers),
                );
                vm_block = 11;
                continue;
            }
            // C line 2121
            13 => {
                let _ = {
                    let assigned = ((sig_num) as i32);
                    (*(sh)).sig_num = assigned;
                    assigned
                };
                vm_block = 12;
                continue;
            }
            // C line 2120
            14 => {
                return JSValue {
                    u: JSValueUnion {
                        uint64: (((0 as i32) as u32) as u64),
                    },
                    tag: ((JS_TAG_EXCEPTION as i32) as i64),
                };
            }
            // C line 2119
            15 => {
                vm_block = if (!(!(sh).is_null()) as i32) != 0 {
                    14
                } else {
                    13
                };
                continue;
            }
            // C line 2118
            16 => {
                let _ = {
                    let assigned =
                        ((js_mallocz(ctx, (core::mem::size_of::<JSOSSignalHandler>() as usize)))
                            as *mut JSOSSignalHandler);
                    sh = assigned;
                    assigned
                };
                vm_block = 15;
                continue;
            }
            // C line 2117
            17 => {
                vm_block = if (!(!(sh).is_null()) as i32) != 0 {
                    16
                } else {
                    11
                };
                continue;
            }
            // C line 2116
            18 => {
                let _ = {
                    let assigned = find_sh(ts, ((sig_num) as i32));
                    sh = assigned;
                    assigned
                };
                vm_block = 17;
                continue;
            }
            // C line 2115
            19 => {
                return JS_ThrowTypeError(ctx, c"not a function".as_ptr());
            }
            // C line 2114
            20 => {
                vm_block = if (!((JS_IsFunction(ctx, func)) != 0) as i32) != 0 {
                    19
                } else {
                    18
                };
                continue;
            }
            // C line 2103
            21 => {
                vm_block =
                    if ((((JS_IsNull(func)) != 0) || ((JS_IsUndefined(func)) != 0)) as i32) != 0 {
                        8
                    } else {
                        20
                    };
                continue;
            }
            // C line 2101
            22 => {
                let _ = {
                    let assigned = *(argv).offset((1 as i32) as isize);
                    func = assigned;
                    assigned
                };
                vm_block = 21;
                continue;
            }
            // C line 2100
            23 => {
                return JS_ThrowRangeError(ctx, c"invalid signal number".as_ptr());
            }
            // C line 2099
            24 => {
                vm_block = if (((sig_num) >= ((64 as i32) as u32)) as i32) != 0 {
                    23
                } else {
                    22
                };
                continue;
            }
            // C line 2098
            25 => {
                return JSValue {
                    u: JSValueUnion {
                        uint64: (((0 as i32) as u32) as u64),
                    },
                    tag: ((JS_TAG_EXCEPTION as i32) as i64),
                };
            }
            // C line 2097
            26 => {
                vm_block = if (JS_ToUint32(
                    ctx,
                    core::ptr::addr_of_mut!(sig_num),
                    *(argv).offset((0 as i32) as isize),
                )) != 0
                {
                    25
                } else {
                    24
                };
                continue;
            }
            // C line 2095
            27 => {
                return JS_ThrowTypeError(
                    ctx,
                    c"signal handler can only be set in the main thread".as_ptr(),
                );
            }
            // C line 2094
            28 => {
                vm_block = if (!((is_main_thread(rt)) != 0) as i32) != 0 {
                    27
                } else {
                    26
                };
                continue;
            }
            // C line 2088
            29 => {
                ts = ((JS_GetRuntimeOpaque(rt)) as *mut JSThreadState);
                vm_block = 28;
                continue;
            }
            // C line 2087
            30 => {
                rt = JS_GetRuntime(ctx);
                vm_block = 29;
                continue;
            }
            _ => std::process::abort(),
        }
    }
}
// Same official C CFG; native functions map to Windows CRT.
pub unsafe fn js_os_setTimeout(
    mut ctx: *mut JSContext,
    mut this_val: JSValue,
    mut argc: i32,
    mut argv: *mut JSValue,
) -> JSValue {
    let mut rt: *mut JSRuntime = core::mem::zeroed();
    let mut ts: *mut JSThreadState = core::mem::zeroed();
    let mut delay: i64 = core::mem::zeroed();
    let mut func: JSValue = core::mem::zeroed();
    let mut th: *mut JSOSTimer = core::mem::zeroed();
    let mut vm_block: usize = 18;
    loop {
        match vm_block {
            // C line ?
            0 => {
                std::process::abort();
            }
            // C line 2200
            1 => {
                return JS_NewInt32(ctx, (*(th)).timer_id);
            }
            // C line 2199
            2 => {
                let _ = list_add_tail(
                    core::ptr::addr_of_mut!((*(th)).link),
                    core::ptr::addr_of_mut!((*(ts)).os_timers),
                );
                vm_block = 1;
                continue;
            }
            // C line 2198
            3 => {
                let _ = {
                    let assigned = JS_DupValue(ctx, func);
                    (*(th)).func = assigned;
                    assigned
                };
                vm_block = 2;
                continue;
            }
            // C line 2197
            4 => {
                let _ = {
                    let assigned = (get_time_ms()).wrapping_add(delay);
                    (*(th)).timeout = assigned;
                    assigned
                };
                vm_block = 3;
                continue;
            }
            // C line 2194
            5 => {
                let _ = {
                    let assigned = (1 as i32);
                    (*(ts)).next_timer_id = assigned;
                    assigned
                };
                vm_block = 4;
                continue;
            }
            // C line 2196
            6 => {
                let _ = {
                    let old = (*(ts)).next_timer_id;
                    (*(ts)).next_timer_id = ((*(ts)).next_timer_id).wrapping_add(1);
                    old
                };
                vm_block = 4;
                continue;
            }
            // C line 2193
            7 => {
                vm_block = if ((((*(ts)).next_timer_id) == (2147483647 as i32)) as i32) != 0 {
                    5
                } else {
                    6
                };
                continue;
            }
            // C line 2192
            8 => {
                let _ = {
                    let assigned = (*(ts)).next_timer_id;
                    (*(th)).timer_id = assigned;
                    assigned
                };
                vm_block = 7;
                continue;
            }
            // C line 2191
            9 => {
                return JSValue {
                    u: JSValueUnion {
                        uint64: (((0 as i32) as u32) as u64),
                    },
                    tag: ((JS_TAG_EXCEPTION as i32) as i64),
                };
            }
            // C line 2190
            10 => {
                vm_block = if (!(!(th).is_null()) as i32) != 0 {
                    9
                } else {
                    8
                };
                continue;
            }
            // C line 2189
            11 => {
                let _ = {
                    let assigned = ((js_mallocz(ctx, (core::mem::size_of::<JSOSTimer>() as usize)))
                        as *mut JSOSTimer);
                    th = assigned;
                    assigned
                };
                vm_block = 10;
                continue;
            }
            // C line 2188
            12 => {
                return JSValue {
                    u: JSValueUnion {
                        uint64: (((0 as i32) as u32) as u64),
                    },
                    tag: ((JS_TAG_EXCEPTION as i32) as i64),
                };
            }
            // C line 2187
            13 => {
                vm_block = if (JS_ToInt64(
                    ctx,
                    core::ptr::addr_of_mut!(delay),
                    *(argv).offset((1 as i32) as isize),
                )) != 0
                {
                    12
                } else {
                    11
                };
                continue;
            }
            // C line 2186
            14 => {
                return JS_ThrowTypeError(ctx, c"not a function".as_ptr());
            }
            // C line 2185
            15 => {
                vm_block = if (!((JS_IsFunction(ctx, func)) != 0) as i32) != 0 {
                    14
                } else {
                    13
                };
                continue;
            }
            // C line 2184
            16 => {
                let _ = {
                    let assigned = *(argv).offset((0 as i32) as isize);
                    func = assigned;
                    assigned
                };
                vm_block = 15;
                continue;
            }
            // C line 2179
            17 => {
                ts = ((JS_GetRuntimeOpaque(rt)) as *mut JSThreadState);
                vm_block = 16;
                continue;
            }
            // C line 2178
            18 => {
                rt = JS_GetRuntime(ctx);
                vm_block = 17;
                continue;
            }
            _ => std::process::abort(),
        }
    }
}
// Same official C CFG; native functions map to Windows CRT.
pub unsafe fn find_timer_by_id(mut ts: *mut JSThreadState, mut timer_id: i32) -> *mut JSOSTimer {
    let mut el: *mut list_head = core::mem::zeroed();
    let mut th: *mut JSOSTimer = core::mem::zeroed();
    let mut vm_block: usize = 9;
    loop {
        match vm_block {
            // C line ?
            0 => {
                std::process::abort();
            }
            // C line 2213
            1 => {
                return core::ptr::null_mut::<JSOSTimer>();
            }
            // C line 2208
            2 => {
                vm_block = if (((el) != (core::ptr::addr_of_mut!((*(ts)).os_timers))) as i32) != 0 {
                    6
                } else {
                    1
                };
                continue;
            }
            // C line ?
            3 => {
                let _ = {
                    let assigned = (*(el)).next;
                    el = assigned;
                    assigned
                };
                vm_block = 2;
                continue;
            }
            // C line 2211
            4 => {
                return th;
            }
            // C line 2210
            5 => {
                vm_block = if ((((*(th)).timer_id) == (timer_id)) as i32) != 0 {
                    4
                } else {
                    3
                };
                continue;
            }
            // C line 2209
            6 => {
                th = ((((el) as *mut u8)
                    .offset(-((core::mem::offset_of!(JSOSTimer, link) as usize) as isize)))
                    as *mut JSOSTimer);
                vm_block = 5;
                continue;
            }
            // C line ?
            7 => {
                let _ = {
                    let assigned = (*(core::ptr::addr_of_mut!((*(ts)).os_timers))).next;
                    el = assigned;
                    assigned
                };
                vm_block = 2;
                continue;
            }
            // C line 2207
            8 => {
                return core::ptr::null_mut::<JSOSTimer>();
            }
            // C line 2206
            9 => {
                vm_block = if (((timer_id) <= (0 as i32)) as i32) != 0 {
                    8
                } else {
                    7
                };
                continue;
            }
            _ => std::process::abort(),
        }
    }
}
// Same official C CFG; native functions map to Windows CRT.
pub unsafe fn js_os_clearTimeout(
    mut ctx: *mut JSContext,
    mut this_val: JSValue,
    mut argc: i32,
    mut argv: *mut JSValue,
) -> JSValue {
    let mut rt: *mut JSRuntime = core::mem::zeroed();
    let mut ts: *mut JSThreadState = core::mem::zeroed();
    let mut th: *mut JSOSTimer = core::mem::zeroed();
    let mut timer_id: i32 = core::mem::zeroed();
    let mut vm_block: usize = 9;
    loop {
        match vm_block {
            // C line ?
            0 => {
                std::process::abort();
            }
            // C line 2230
            1 => {
                return JSValue {
                    u: JSValueUnion {
                        uint64: (((0 as i32) as u32) as u64),
                    },
                    tag: ((JS_TAG_UNDEFINED as i32) as i64),
                };
            }
            // C line 2229
            2 => {
                let _ = free_timer(rt, th);
                vm_block = 1;
                continue;
            }
            // C line 2228
            3 => {
                return JSValue {
                    u: JSValueUnion {
                        uint64: (((0 as i32) as u32) as u64),
                    },
                    tag: ((JS_TAG_UNDEFINED as i32) as i64),
                };
            }
            // C line 2227
            4 => {
                vm_block = if (!(!(th).is_null()) as i32) != 0 {
                    3
                } else {
                    2
                };
                continue;
            }
            // C line 2226
            5 => {
                let _ = {
                    let assigned = find_timer_by_id(ts, timer_id);
                    th = assigned;
                    assigned
                };
                vm_block = 4;
                continue;
            }
            // C line 2225
            6 => {
                return JSValue {
                    u: JSValueUnion {
                        uint64: (((0 as i32) as u32) as u64),
                    },
                    tag: ((JS_TAG_EXCEPTION as i32) as i64),
                };
            }
            // C line 2224
            7 => {
                vm_block = if (JS_ToInt32(
                    ctx,
                    core::ptr::addr_of_mut!(timer_id),
                    *(argv).offset((0 as i32) as isize),
                )) != 0
                {
                    6
                } else {
                    5
                };
                continue;
            }
            // C line 2220
            8 => {
                ts = ((JS_GetRuntimeOpaque(rt)) as *mut JSThreadState);
                vm_block = 7;
                continue;
            }
            // C line 2219
            9 => {
                rt = JS_GetRuntime(ctx);
                vm_block = 8;
                continue;
            }
            _ => std::process::abort(),
        }
    }
}
// Same official C CFG; native functions map to Windows CRT.
pub unsafe fn js_os_sleepAsync(
    mut ctx: *mut JSContext,
    mut this_val: JSValue,
    mut argc: i32,
    mut argv: *mut JSValue,
) -> JSValue {
    let mut rt: *mut JSRuntime = core::mem::zeroed();
    let mut ts: *mut JSThreadState = core::mem::zeroed();
    let mut delay: i64 = core::mem::zeroed();
    let mut th: *mut JSOSTimer = core::mem::zeroed();
    let mut promise: JSValue = core::mem::zeroed();
    let mut resolving_funcs: [JSValue; 2] = core::mem::zeroed();
    let mut vm_block: usize = 20;
    loop {
        match vm_block {
            // C line ?
            0 => {
                std::process::abort();
            }
            // C line 2262
            1 => {
                return promise;
            }
            // C line 2261
            2 => {
                let _ = JS_FreeValue(
                    ctx,
                    *((resolving_funcs).as_mut_ptr()).offset((1 as i32) as isize),
                );
                vm_block = 1;
                continue;
            }
            // C line 2260
            3 => {
                let _ = JS_FreeValue(
                    ctx,
                    *((resolving_funcs).as_mut_ptr()).offset((0 as i32) as isize),
                );
                vm_block = 2;
                continue;
            }
            // C line 2259
            4 => {
                let _ = list_add_tail(
                    core::ptr::addr_of_mut!((*(th)).link),
                    core::ptr::addr_of_mut!((*(ts)).os_timers),
                );
                vm_block = 3;
                continue;
            }
            // C line 2258
            5 => {
                let _ = {
                    let assigned = JS_DupValue(
                        ctx,
                        *((resolving_funcs).as_mut_ptr()).offset((0 as i32) as isize),
                    );
                    (*(th)).func = assigned;
                    assigned
                };
                vm_block = 4;
                continue;
            }
            // C line 2257
            6 => {
                let _ = {
                    let assigned = (get_time_ms()).wrapping_add(delay);
                    (*(th)).timeout = assigned;
                    assigned
                };
                vm_block = 5;
                continue;
            }
            // C line 2256
            7 => {
                let _ = {
                    let assigned = (1 as i32).wrapping_neg();
                    (*(th)).timer_id = assigned;
                    assigned
                };
                vm_block = 6;
                continue;
            }
            // C line 2254
            8 => {
                return JSValue {
                    u: JSValueUnion {
                        uint64: (((0 as i32) as u32) as u64),
                    },
                    tag: ((JS_TAG_EXCEPTION as i32) as i64),
                };
            }
            // C line 2253
            9 => {
                let _ = JS_FreeValue(
                    ctx,
                    *((resolving_funcs).as_mut_ptr()).offset((1 as i32) as isize),
                );
                vm_block = 8;
                continue;
            }
            // C line 2252
            10 => {
                let _ = JS_FreeValue(
                    ctx,
                    *((resolving_funcs).as_mut_ptr()).offset((0 as i32) as isize),
                );
                vm_block = 9;
                continue;
            }
            // C line 2251
            11 => {
                let _ = JS_FreeValue(ctx, promise);
                vm_block = 10;
                continue;
            }
            // C line 2250
            12 => {
                vm_block = if (!(!(th).is_null()) as i32) != 0 {
                    11
                } else {
                    7
                };
                continue;
            }
            // C line 2249
            13 => {
                let _ = {
                    let assigned = ((js_mallocz(ctx, (core::mem::size_of::<JSOSTimer>() as usize)))
                        as *mut JSOSTimer);
                    th = assigned;
                    assigned
                };
                vm_block = 12;
                continue;
            }
            // C line 2247
            14 => {
                return JSValue {
                    u: JSValueUnion {
                        uint64: (((0 as i32) as u32) as u64),
                    },
                    tag: ((JS_TAG_EXCEPTION as i32) as i64),
                };
            }
            // C line 2246
            15 => {
                vm_block = if (JS_IsException(promise)) != 0 {
                    14
                } else {
                    13
                };
                continue;
            }
            // C line 2245
            16 => {
                let _ = {
                    let assigned = JS_NewPromiseCapability(ctx, (resolving_funcs).as_mut_ptr());
                    promise = assigned;
                    assigned
                };
                vm_block = 15;
                continue;
            }
            // C line 2244
            17 => {
                return JSValue {
                    u: JSValueUnion {
                        uint64: (((0 as i32) as u32) as u64),
                    },
                    tag: ((JS_TAG_EXCEPTION as i32) as i64),
                };
            }
            // C line 2243
            18 => {
                vm_block = if (JS_ToInt64(
                    ctx,
                    core::ptr::addr_of_mut!(delay),
                    *(argv).offset((0 as i32) as isize),
                )) != 0
                {
                    17
                } else {
                    16
                };
                continue;
            }
            // C line 2238
            19 => {
                ts = ((JS_GetRuntimeOpaque(rt)) as *mut JSThreadState);
                vm_block = 18;
                continue;
            }
            // C line 2237
            20 => {
                rt = JS_GetRuntime(ctx);
                vm_block = 19;
                continue;
            }
            _ => std::process::abort(),
        }
    }
}
// Same official C CFG; native functions map to Windows CRT.
pub unsafe fn make_string_error(
    mut ctx: *mut JSContext,
    mut buf: *const c_char,
    mut err: i32,
) -> JSValue {
    let mut vm_block: usize = 1;
    loop {
        match vm_block {
            // C line ?
            0 => {
                std::process::abort();
            }
            // C line 2681
            1 => {
                return make_obj_error(ctx, JS_NewString(ctx, buf), err);
            }
            _ => std::process::abort(),
        }
    }
}
// Same official C CFG; native functions map to Windows CRT.
pub unsafe fn js_os_getcwd(
    mut ctx: *mut JSContext,
    mut this_val: JSValue,
    mut argc: i32,
    mut argv: *mut JSValue,
) -> JSValue {
    let mut buf: [c_char; WIN_PATH_MAX] = core::mem::zeroed();
    let mut err: i32 = core::mem::zeroed();
    let mut vm_block: usize = 5;
    loop {
        match vm_block {
            // C line ?
            0 => {
                std::process::abort();
            }
            // C line 2697
            1 => {
                return make_string_error(ctx, (buf).as_mut_ptr(), err);
            }
            // C line 2693
            2 => {
                let _ = {
                    let assigned = *(stdio_errno_pointer());
                    err = assigned;
                    assigned
                };
                vm_block = 1;
                continue;
            }
            // C line 2692
            3 => {
                let _ = {
                    let assigned = ((0 as i32) as c_char);
                    *((buf).as_mut_ptr()).offset((0 as i32) as isize) = assigned;
                    assigned
                };
                vm_block = 2;
                continue;
            }
            // C line 2695
            4 => {
                let _ = {
                    let assigned = (0 as i32);
                    err = assigned;
                    assigned
                };
                vm_block = 1;
                continue;
            }
            // C line 2691
            5 => {
                vm_block = if (!(!(libc::getcwd(
                    (buf).as_mut_ptr(),
                    (core::mem::size_of::<[c_char; WIN_PATH_MAX]>() as usize),
                ))
                .is_null()) as i32)
                    != 0
                {
                    3
                } else {
                    4
                };
                continue;
            }
            _ => std::process::abort(),
        }
    }
}
// Same official C CFG; native functions map to Windows CRT.
pub unsafe fn js_os_chdir(
    mut ctx: *mut JSContext,
    mut this_val: JSValue,
    mut argc: i32,
    mut argv: *mut JSValue,
) -> JSValue {
    let mut target: *const c_char = core::mem::zeroed();
    let mut err: i32 = core::mem::zeroed();
    let mut vm_block: usize = 6;
    loop {
        match vm_block {
            // C line ?
            0 => {
                std::process::abort();
            }
            // C line 2711
            1 => {
                return JS_NewInt32(ctx, err);
            }
            // C line 2710
            2 => {
                let _ = JS_FreeCString(ctx, target);
                vm_block = 1;
                continue;
            }
            // C line 2709
            3 => {
                let _ = {
                    let assigned = ((js_get_errno(((libc::chdir(target)) as isize))) as i32);
                    err = assigned;
                    assigned
                };
                vm_block = 2;
                continue;
            }
            // C line 2708
            4 => {
                return JSValue {
                    u: JSValueUnion {
                        uint64: (((0 as i32) as u32) as u64),
                    },
                    tag: ((JS_TAG_EXCEPTION as i32) as i64),
                };
            }
            // C line 2707
            5 => {
                vm_block = if (!(!(target).is_null()) as i32) != 0 {
                    4
                } else {
                    3
                };
                continue;
            }
            // C line 2706
            6 => {
                let _ = {
                    let assigned = JS_ToCString(ctx, *(argv).offset((0 as i32) as isize));
                    target = assigned;
                    assigned
                };
                vm_block = 5;
                continue;
            }
            _ => std::process::abort(),
        }
    }
}
// Same official C CFG; native functions map to Windows CRT.
pub unsafe fn js_os_readdir(
    mut ctx: *mut JSContext,
    mut this_val: JSValue,
    mut argc: i32,
    mut argv: *mut JSValue,
) -> JSValue {
    let mut path: *const c_char = core::mem::zeroed();
    let mut f: *mut libc::DIR = core::mem::zeroed();
    let mut d: *mut libc::dirent = core::mem::zeroed();
    let mut obj: JSValue = core::mem::zeroed();
    let mut err: i32 = core::mem::zeroed();
    let mut len: u32 = core::mem::zeroed();
    let mut vm_block: usize = 24;
    loop {
        match vm_block {
            // C line ?
            0 => {
                std::process::abort();
            }
            // C line ? labels: done
            1 => {
                return make_obj_error(ctx, obj, err);
            }
            // C line 2778
            2 => {
                let _ = libc::closedir(f);
                vm_block = 1;
                continue;
            }
            // C line 2767
            3 => {
                vm_block = 9;
                continue;
            }
            // C line 2774
            4 => {
                let _ = JS_DefinePropertyValueUint32(
                    ctx,
                    obj,
                    {
                        let old = len;
                        len = (len).wrapping_add(1);
                        old
                    },
                    JS_NewString(ctx, ((*(d)).d_name).as_mut_ptr()),
                    ((((1 as i32).wrapping_shl((0 as i32) as u32))
                        | ((1 as i32).wrapping_shl((1 as i32) as u32)))
                        | ((1 as i32).wrapping_shl((2 as i32) as u32))),
                );
                vm_block = 3;
                continue;
            }
            // C line 2772
            5 => {
                vm_block = 2;
                continue;
            }
            // C line 2771
            6 => {
                let _ = {
                    let assigned = *(stdio_errno_pointer());
                    err = assigned;
                    assigned
                };
                vm_block = 5;
                continue;
            }
            // C line 2770
            7 => {
                vm_block = if (!(!(d).is_null()) as i32) != 0 {
                    6
                } else {
                    4
                };
                continue;
            }
            // C line 2769
            8 => {
                let _ = {
                    let assigned = libc::readdir(f);
                    d = assigned;
                    assigned
                };
                vm_block = 7;
                continue;
            }
            // C line 2768
            9 => {
                let _ = {
                    let assigned = (0 as i32);
                    *(stdio_errno_pointer()) = assigned;
                    assigned
                };
                vm_block = 8;
                continue;
            }
            // C line 2766
            10 => {
                let _ = {
                    let assigned = ((0 as i32) as u32);
                    len = assigned;
                    assigned
                };
                vm_block = 3;
                continue;
            }
            // C line 2765
            11 => {
                vm_block = 1;
                continue;
            }
            // C line 2764
            12 => {
                vm_block = if (!(!(f).is_null()) as i32) != 0 {
                    11
                } else {
                    10
                };
                continue;
            }
            // C line 2763
            13 => {
                let _ = JS_FreeCString(ctx, path);
                vm_block = 12;
                continue;
            }
            // C line 2760
            14 => {
                let _ = {
                    let assigned = *(stdio_errno_pointer());
                    err = assigned;
                    assigned
                };
                vm_block = 13;
                continue;
            }
            // C line 2762
            15 => {
                let _ = {
                    let assigned = (0 as i32);
                    err = assigned;
                    assigned
                };
                vm_block = 13;
                continue;
            }
            // C line 2759
            16 => {
                vm_block = if (!(!(f).is_null()) as i32) != 0 {
                    14
                } else {
                    15
                };
                continue;
            }
            // C line 2758
            17 => {
                let _ = {
                    let assigned = libc::opendir(path);
                    f = assigned;
                    assigned
                };
                vm_block = 16;
                continue;
            }
            // C line 2756
            18 => {
                return JSValue {
                    u: JSValueUnion {
                        uint64: (((0 as i32) as u32) as u64),
                    },
                    tag: ((JS_TAG_EXCEPTION as i32) as i64),
                };
            }
            // C line 2755
            19 => {
                let _ = JS_FreeCString(ctx, path);
                vm_block = 18;
                continue;
            }
            // C line 2754
            20 => {
                vm_block = if (JS_IsException(obj)) != 0 { 19 } else { 17 };
                continue;
            }
            // C line 2753
            21 => {
                let _ = {
                    let assigned = JS_NewArray(ctx);
                    obj = assigned;
                    assigned
                };
                vm_block = 20;
                continue;
            }
            // C line 2752
            22 => {
                return JSValue {
                    u: JSValueUnion {
                        uint64: (((0 as i32) as u32) as u64),
                    },
                    tag: ((JS_TAG_EXCEPTION as i32) as i64),
                };
            }
            // C line 2751
            23 => {
                vm_block = if (!(!(path).is_null()) as i32) != 0 {
                    22
                } else {
                    21
                };
                continue;
            }
            // C line 2750
            24 => {
                let _ = {
                    let assigned = JS_ToCString(ctx, *(argv).offset((0 as i32) as isize));
                    path = assigned;
                    assigned
                };
                vm_block = 23;
                continue;
            }
            _ => std::process::abort(),
        }
    }
}
// Same official C CFG; native functions map to Windows CRT.
pub unsafe fn js_os_realpath(
    mut ctx: *mut JSContext,
    mut this_val: JSValue,
    mut argc: i32,
    mut argv: *mut JSValue,
) -> JSValue {
    let mut path: *const c_char = core::mem::zeroed();
    let mut buf: [c_char; WIN_PATH_MAX] = core::mem::zeroed();
    let mut res: *mut c_char = core::mem::zeroed();
    let mut err: i32 = core::mem::zeroed();
    let mut vm_block: usize = 10;
    loop {
        match vm_block {
            // C line ?
            0 => {
                std::process::abort();
            }
            // C line 2987
            1 => {
                return make_string_error(ctx, (buf).as_mut_ptr(), err);
            }
            // C line 2983
            2 => {
                let _ = {
                    let assigned = *(stdio_errno_pointer());
                    err = assigned;
                    assigned
                };
                vm_block = 1;
                continue;
            }
            // C line 2982
            3 => {
                let _ = {
                    let assigned = ((0 as i32) as c_char);
                    *((buf).as_mut_ptr()).offset((0 as i32) as isize) = assigned;
                    assigned
                };
                vm_block = 2;
                continue;
            }
            // C line 2985
            4 => {
                let _ = {
                    let assigned = (0 as i32);
                    err = assigned;
                    assigned
                };
                vm_block = 1;
                continue;
            }
            // C line 2981
            5 => {
                vm_block = if (!(!(res).is_null()) as i32) != 0 {
                    3
                } else {
                    4
                };
                continue;
            }
            // C line 2980
            6 => {
                let _ = JS_FreeCString(ctx, path);
                vm_block = 5;
                continue;
            }
            // C line 2979
            7 => {
                let _ = {
                    let assigned = realpath(path, (buf).as_mut_ptr());
                    res = assigned;
                    assigned
                };
                vm_block = 6;
                continue;
            }
            // C line 2978
            8 => {
                return JSValue {
                    u: JSValueUnion {
                        uint64: (((0 as i32) as u32) as u64),
                    },
                    tag: ((JS_TAG_EXCEPTION as i32) as i64),
                };
            }
            // C line 2977
            9 => {
                vm_block = if (!(!(path).is_null()) as i32) != 0 {
                    8
                } else {
                    7
                };
                continue;
            }
            // C line 2976
            10 => {
                let _ = {
                    let assigned = JS_ToCString(ctx, *(argv).offset((0 as i32) as isize));
                    path = assigned;
                    assigned
                };
                vm_block = 9;
                continue;
            }
            _ => std::process::abort(),
        }
    }
}
