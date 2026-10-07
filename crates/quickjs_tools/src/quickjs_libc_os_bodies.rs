// Generated from the official C AST; original goto CFG retained.
// c: quickjs-libc.c:1735. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
pub unsafe fn js_os_open(mut ctx: *mut JSContext, mut this_val: JSValue, mut argc: i32, mut argv: *mut JSValue) -> JSValue {

let mut filename: *const c_char = core::mem::zeroed();
let mut flags: i32 = core::mem::zeroed();
let mut mode: i32 = core::mem::zeroed();
let mut ret: i32 = core::mem::zeroed();
let mut vm_block: usize = 13;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 1762
1 => {
return JS_NewInt32(ctx, ret);
}
// C line 1761
2 => {
let _ = JS_FreeCString(ctx, filename);
vm_block = 1; continue;
}
// C line 1760
3 => {
let _ = { let assigned = ((js_get_errno(((libc::open(filename, flags, mode)) as isize))) as i32); ret = assigned; assigned };
vm_block = 2; continue;
}
// C line 1750
4 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line ? labels: fail
5 => {
let _ = JS_FreeCString(ctx, filename);
vm_block = 4; continue;
}
// C line 1747
6 => {
vm_block = if (JS_ToInt32(ctx, core::ptr::addr_of_mut!(mode), *(argv).offset(((2 as i32)) as isize))) != 0 { 5 } else { 3 }; continue;
}
// C line 1753
7 => {
let _ = { let assigned = (438 as i32); mode = assigned; assigned };
vm_block = 3; continue;
}
// C line 1746
8 => {
vm_block = if ((((((((argc) >= ((3 as i32))) as i32)) != 0) && (((!((JS_IsUndefined(*(argv).offset(((2 as i32)) as isize))) != 0) as i32)) != 0)) as i32)) != 0 { 6 } else { 7 }; continue;
}
// C line 1745
9 => {
vm_block = 5; continue;
}
// C line 1744
10 => {
vm_block = if (JS_ToInt32(ctx, core::ptr::addr_of_mut!(flags), *(argv).offset(((1 as i32)) as isize))) != 0 { 9 } else { 8 }; continue;
}
// C line 1743
11 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 1742
12 => {
vm_block = if ((!(!(filename).is_null()) as i32)) != 0 { 11 } else { 10 }; continue;
}
// C line 1741
13 => {
let _ = { let assigned = JS_ToCString(ctx, *(argv).offset(((0 as i32)) as isize)); filename = assigned; assigned };
vm_block = 12; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs-libc.c:1765. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
pub unsafe fn js_os_close(mut ctx: *mut JSContext, mut this_val: JSValue, mut argc: i32, mut argv: *mut JSValue) -> JSValue {

let mut fd: i32 = core::mem::zeroed();
let mut ret: i32 = core::mem::zeroed();
let mut vm_block: usize = 4;
loop { match vm_block {
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
let _ = { let assigned = ((js_get_errno(((libc::close(fd)) as isize))) as i32); ret = assigned; assigned };
vm_block = 1; continue;
}
// C line 1770
3 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 1769
4 => {
vm_block = if (JS_ToInt32(ctx, core::ptr::addr_of_mut!(fd), *(argv).offset(((0 as i32)) as isize))) != 0 { 3 } else { 2 }; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs-libc.c:1775. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
pub unsafe fn js_os_seek(mut ctx: *mut JSContext, mut this_val: JSValue, mut argc: i32, mut argv: *mut JSValue) -> JSValue {

let mut fd: i32 = core::mem::zeroed();
let mut whence: i32 = core::mem::zeroed();
let mut pos: i64 = core::mem::zeroed();
let mut ret: i64 = core::mem::zeroed();
let mut is_bigint: i32 = core::mem::zeroed();
let mut vm_block: usize = 13;
loop { match vm_block {
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
vm_block = if (is_bigint) != 0 { 1 } else { 2 }; continue;
}
// C line 1791
4 => {
let _ = { let assigned = (((*(stdio_errno_pointer())).wrapping_neg()) as i64); ret = assigned; assigned };
vm_block = 3; continue;
}
// C line 1790
5 => {
vm_block = if ((((ret) == (((((1 as i32)).wrapping_neg()) as i64))) as i32)) != 0 { 4 } else { 3 }; continue;
}
// C line 1789
6 => {
let _ = { let assigned = libc::lseek(fd, pos, whence); ret = assigned; assigned };
vm_block = 5; continue;
}
// C line 1788
7 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 1787
8 => {
vm_block = if (JS_ToInt32(ctx, core::ptr::addr_of_mut!(whence), *(argv).offset(((2 as i32)) as isize))) != 0 { 7 } else { 6 }; continue;
}
// C line 1786
9 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 1785
10 => {
vm_block = if (JS_ToInt64Ext(ctx, core::ptr::addr_of_mut!(pos), *(argv).offset(((1 as i32)) as isize))) != 0 { 9 } else { 8 }; continue;
}
// C line 1784
11 => {
let _ = { let assigned = JS_IsBigInt(ctx, *(argv).offset(((1 as i32)) as isize)); is_bigint = assigned; assigned };
vm_block = 10; continue;
}
// C line 1783
12 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 1782
13 => {
vm_block = if (JS_ToInt32(ctx, core::ptr::addr_of_mut!(fd), *(argv).offset(((0 as i32)) as isize))) != 0 { 12 } else { 11 }; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs-libc.c:1798. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
pub unsafe fn js_os_read_write(mut ctx: *mut JSContext, mut this_val: JSValue, mut argc: i32, mut argv: *mut JSValue, mut magic: i32) -> JSValue {

let mut fd: i32 = core::mem::zeroed();
let mut pos: u64 = core::mem::zeroed();
let mut len: u64 = core::mem::zeroed();
let mut size: usize = core::mem::zeroed();
let mut ret: isize = core::mem::zeroed();
let mut buf: *mut u8 = core::mem::zeroed();
let mut vm_block: usize = 15;
loop { match vm_block {
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
let _ = { let assigned = js_get_errno(libc::write(fd, (((buf).offset(((pos) as isize))) as *const c_void), ((len) as usize))); ret = assigned; assigned };
vm_block = 1; continue;
}
// C line 1821
3 => {
let _ = { let assigned = js_get_errno(libc::read(fd, (((buf).offset(((pos) as isize))) as *mut c_void), ((len) as usize))); ret = assigned; assigned };
vm_block = 1; continue;
}
// C line 1818
4 => {
vm_block = if (magic) != 0 { 2 } else { 3 }; continue;
}
// C line 1817
5 => {
return JS_ThrowRangeError(ctx, c"read/write array buffer overflow".as_ptr());
}
// C line 1816
6 => {
vm_block = if (((((pos).wrapping_add(len)) > (((size) as u64))) as i32)) != 0 { 5 } else { 4 }; continue;
}
// C line 1815
7 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 1814
8 => {
vm_block = if ((!(!(buf).is_null()) as i32)) != 0 { 7 } else { 6 }; continue;
}
// C line 1813
9 => {
let _ = { let assigned = JS_GetArrayBuffer(ctx, core::ptr::addr_of_mut!(size), *(argv).offset(((1 as i32)) as isize)); buf = assigned; assigned };
vm_block = 8; continue;
}
// C line 1812
10 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 1811
11 => {
vm_block = if (JS_ToIndex(ctx, core::ptr::addr_of_mut!(len), *(argv).offset(((3 as i32)) as isize))) != 0 { 10 } else { 9 }; continue;
}
// C line 1810
12 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 1809
13 => {
vm_block = if (JS_ToIndex(ctx, core::ptr::addr_of_mut!(pos), *(argv).offset(((2 as i32)) as isize))) != 0 { 12 } else { 11 }; continue;
}
// C line 1808
14 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 1807
15 => {
vm_block = if (JS_ToInt32(ctx, core::ptr::addr_of_mut!(fd), *(argv).offset(((0 as i32)) as isize))) != 0 { 14 } else { 13 }; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs-libc.c:1825. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
pub unsafe fn js_os_isatty(mut ctx: *mut JSContext, mut this_val: JSValue, mut argc: i32, mut argv: *mut JSValue) -> JSValue {

let mut fd: i32 = core::mem::zeroed();
let mut vm_block: usize = 3;
loop { match vm_block {
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
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 1829
3 => {
vm_block = if (JS_ToInt32(ctx, core::ptr::addr_of_mut!(fd), *(argv).offset(((0 as i32)) as isize))) != 0 { 2 } else { 1 }; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs-libc.c:1939. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
pub unsafe fn js_os_remove(mut ctx: *mut JSContext, mut this_val: JSValue, mut argc: i32, mut argv: *mut JSValue) -> JSValue {

let mut filename: *const c_char = core::mem::zeroed();
let mut ret: i32 = core::mem::zeroed();
let mut vm_block: usize = 7;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 1962
1 => {
return JS_NewInt32(ctx, ret);
}
// C line 1961
2 => {
let _ = JS_FreeCString(ctx, filename);
vm_block = 1; continue;
}
// C line 1960
3 => {
let _ = { let assigned = ((js_get_errno(((ret) as isize))) as i32); ret = assigned; assigned };
vm_block = 2; continue;
}
// C line 1958
4 => {
let _ = { let assigned = libc::remove(filename); ret = assigned; assigned };
vm_block = 3; continue;
}
// C line 1947
5 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 1946
6 => {
vm_block = if ((!(!(filename).is_null()) as i32)) != 0 { 5 } else { 4 }; continue;
}
// C line 1945
7 => {
let _ = { let assigned = JS_ToCString(ctx, *(argv).offset(((0 as i32)) as isize)); filename = assigned; assigned };
vm_block = 6; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs-libc.c:1965. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
pub unsafe fn js_os_rename(mut ctx: *mut JSContext, mut this_val: JSValue, mut argc: i32, mut argv: *mut JSValue) -> JSValue {

let mut oldpath: *const c_char = core::mem::zeroed();
let mut newpath: *const c_char = core::mem::zeroed();
let mut ret: i32 = core::mem::zeroed();
let mut vm_block: usize = 11;
loop { match vm_block {
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
vm_block = 1; continue;
}
// C line 1980
3 => {
let _ = JS_FreeCString(ctx, oldpath);
vm_block = 2; continue;
}
// C line 1979
4 => {
let _ = { let assigned = ((js_get_errno(((libc::rename(oldpath, newpath)) as isize))) as i32); ret = assigned; assigned };
vm_block = 3; continue;
}
// C line 1977
5 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 1976
6 => {
let _ = JS_FreeCString(ctx, oldpath);
vm_block = 5; continue;
}
// C line 1975
7 => {
vm_block = if ((!(!(newpath).is_null()) as i32)) != 0 { 6 } else { 4 }; continue;
}
// C line 1974
8 => {
let _ = { let assigned = JS_ToCString(ctx, *(argv).offset(((1 as i32)) as isize)); newpath = assigned; assigned };
vm_block = 7; continue;
}
// C line 1973
9 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 1972
10 => {
vm_block = if ((!(!(oldpath).is_null()) as i32)) != 0 { 9 } else { 8 }; continue;
}
// C line 1971
11 => {
let _ = { let assigned = JS_ToCString(ctx, *(argv).offset(((0 as i32)) as isize)); oldpath = assigned; assigned };
vm_block = 10; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs-libc.c:1985. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
pub unsafe fn is_main_thread(mut rt: *mut JSRuntime) -> i32 {

let mut ts: *mut JSThreadState = core::mem::zeroed();
let mut vm_block: usize = 2;
loop { match vm_block {
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
vm_block = 1; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs-libc.c:1991. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
pub unsafe fn find_rh(mut ts: *mut JSThreadState, mut fd: i32) -> *mut JSOSRWHandler {

let mut rh: *mut JSOSRWHandler = core::mem::zeroed();
let mut el: *mut list_head = core::mem::zeroed();
let mut vm_block: usize = 7;
loop { match vm_block {
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
vm_block = if ((((el) != (core::ptr::addr_of_mut!((*(ts)).os_rw_handlers))) as i32)) != 0 { 6 } else { 1 }; continue;
}
// C line ?
3 => {
let _ = { let assigned = (*(el)).next; el = assigned; assigned };
vm_block = 2; continue;
}
// C line 1999
4 => {
return rh;
}
// C line 1998
5 => {
vm_block = if (((((*(rh)).fd) == (fd)) as i32)) != 0 { 4 } else { 3 }; continue;
}
// C line 1997
6 => {
let _ = { let assigned = (((((el) as *mut u8)).offset(-(((core::mem::offset_of!(JSOSRWHandler, link) as usize)) as isize))) as *mut JSOSRWHandler); rh = assigned; assigned };
vm_block = 5; continue;
}
// C line ?
7 => {
let _ = { let assigned = (*(core::ptr::addr_of_mut!((*(ts)).os_rw_handlers))).next; el = assigned; assigned };
vm_block = 2; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs-libc.c:2014. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
pub unsafe fn js_os_setReadHandler(mut ctx: *mut JSContext, mut this_val: JSValue, mut argc: i32, mut argv: *mut JSValue, mut magic: i32) -> JSValue {

let mut rt: *mut JSRuntime = core::mem::zeroed();
let mut ts: *mut JSThreadState = core::mem::zeroed();
let mut rh: *mut JSOSRWHandler = core::mem::zeroed();
let mut fd: i32 = core::mem::zeroed();
let mut func: JSValue = core::mem::zeroed();
let mut vm_block: usize = 26;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 2053
1 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) };
}
// C line 2034
2 => {
let _ = free_rw_handler(JS_GetRuntime(ctx), rh);
vm_block = 1; continue;
}
// C line 2031
3 => {
vm_block = if (((((JS_IsNull(*(((*(rh)).rw_func).as_mut_ptr()).offset(((0 as i32)) as isize))) != 0) && ((JS_IsNull(*(((*(rh)).rw_func).as_mut_ptr()).offset(((1 as i32)) as isize))) != 0)) as i32)) != 0 { 2 } else { 1 }; continue;
}
// C line 2030
4 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_NULL as i32)) as i64) }; *(((*(rh)).rw_func).as_mut_ptr()).offset((magic) as isize) = assigned; assigned };
vm_block = 3; continue;
}
// C line 2029
5 => {
let _ = JS_FreeValue(ctx, *(((*(rh)).rw_func).as_mut_ptr()).offset((magic) as isize));
vm_block = 4; continue;
}
// C line 2028
6 => {
vm_block = if !(rh).is_null() { 5 } else { 1 }; continue;
}
// C line 2027
7 => {
let _ = { let assigned = find_rh(ts, fd); rh = assigned; assigned };
vm_block = 6; continue;
}
// C line 2051
8 => {
let _ = { let assigned = JS_DupValue(ctx, func); *(((*(rh)).rw_func).as_mut_ptr()).offset((magic) as isize) = assigned; assigned };
vm_block = 1; continue;
}
// C line 2050
9 => {
let _ = JS_FreeValue(ctx, *(((*(rh)).rw_func).as_mut_ptr()).offset((magic) as isize));
vm_block = 8; continue;
}
// C line 2048
10 => {
let _ = list_add_tail(core::ptr::addr_of_mut!((*(rh)).link), core::ptr::addr_of_mut!((*(ts)).os_rw_handlers));
vm_block = 9; continue;
}
// C line 2047
11 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_NULL as i32)) as i64) }; *(((*(rh)).rw_func).as_mut_ptr()).offset(((1 as i32)) as isize) = assigned; assigned };
vm_block = 10; continue;
}
// C line 2046
12 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_NULL as i32)) as i64) }; *(((*(rh)).rw_func).as_mut_ptr()).offset(((0 as i32)) as isize) = assigned; assigned };
vm_block = 11; continue;
}
// C line 2045
13 => {
let _ = { let assigned = fd; (*(rh)).fd = assigned; assigned };
vm_block = 12; continue;
}
// C line 2044
14 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 2043
15 => {
vm_block = if ((!(!(rh).is_null()) as i32)) != 0 { 14 } else { 13 }; continue;
}
// C line 2042
16 => {
let _ = { let assigned = ((js_mallocz(ctx, (core::mem::size_of::<JSOSRWHandler>() as usize))) as *mut JSOSRWHandler); rh = assigned; assigned };
vm_block = 15; continue;
}
// C line 2041
17 => {
vm_block = if ((!(!(rh).is_null()) as i32)) != 0 { 16 } else { 9 }; continue;
}
// C line 2040
18 => {
let _ = { let assigned = find_rh(ts, fd); rh = assigned; assigned };
vm_block = 17; continue;
}
// C line 2039
19 => {
return JS_ThrowTypeError(ctx, c"not a function".as_ptr());
}
// C line 2038
20 => {
vm_block = if ((!((JS_IsFunction(ctx, func)) != 0) as i32)) != 0 { 19 } else { 18 }; continue;
}
// C line 2026
21 => {
vm_block = if (JS_IsNull(func)) != 0 { 7 } else { 20 }; continue;
}
// C line 2025
22 => {
let _ = { let assigned = *(argv).offset(((1 as i32)) as isize); func = assigned; assigned };
vm_block = 21; continue;
}
// C line 2024
23 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 2023
24 => {
vm_block = if (JS_ToInt32(ctx, core::ptr::addr_of_mut!(fd), *(argv).offset(((0 as i32)) as isize))) != 0 { 23 } else { 22 }; continue;
}
// C line 2018
25 => {
ts = ((JS_GetRuntimeOpaque(rt)) as *mut JSThreadState);
vm_block = 24; continue;
}
// C line 2017
26 => {
rt = JS_GetRuntime(ctx);
vm_block = 25; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs-libc.c:2056. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
pub unsafe fn find_sh(mut ts: *mut JSThreadState, mut sig_num: i32) -> *mut JSOSSignalHandler {

let mut sh: *mut JSOSSignalHandler = core::mem::zeroed();
let mut el: *mut list_head = core::mem::zeroed();
let mut vm_block: usize = 7;
loop { match vm_block {
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
vm_block = if ((((el) != (core::ptr::addr_of_mut!((*(ts)).os_signal_handlers))) as i32)) != 0 { 6 } else { 1 }; continue;
}
// C line ?
3 => {
let _ = { let assigned = (*(el)).next; el = assigned; assigned };
vm_block = 2; continue;
}
// C line 2063
4 => {
return sh;
}
// C line 2062
5 => {
vm_block = if (((((*(sh)).sig_num) == (sig_num)) as i32)) != 0 { 4 } else { 3 }; continue;
}
// C line 2061
6 => {
let _ = { let assigned = (((((el) as *mut u8)).offset(-(((core::mem::offset_of!(JSOSSignalHandler, link) as usize)) as isize))) as *mut JSOSSignalHandler); sh = assigned; assigned };
vm_block = 5; continue;
}
// C line ?
7 => {
let _ = { let assigned = (*(core::ptr::addr_of_mut!((*(ts)).os_signal_handlers))).next; el = assigned; assigned };
vm_block = 2; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs-libc.c:2084. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
pub unsafe fn js_os_signal(mut ctx: *mut JSContext, mut this_val: JSValue, mut argc: i32, mut argv: *mut JSValue) -> JSValue {

let mut rt: *mut JSRuntime = core::mem::zeroed();
let mut ts: *mut JSThreadState = core::mem::zeroed();
let mut sh: *mut JSOSSignalHandler = core::mem::zeroed();
let mut sig_num: u32 = core::mem::zeroed();
let mut func: JSValue = core::mem::zeroed();
let mut handler: usize = core::mem::zeroed();
let mut vm_block: usize = 30;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 2128
1 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) };
}
// C line 2112
2 => {
let _ = libc::signal(((sig_num) as i32), handler);
vm_block = 1; continue;
}
// C line 2109
3 => {
let _ = { let assigned = (((libc::SIG_DFL as i32)) as usize); handler = assigned; assigned };
vm_block = 2; continue;
}
// C line 2111
4 => {
let _ = { let assigned = (((libc::SIG_IGN as i32)) as usize); handler = assigned; assigned };
vm_block = 2; continue;
}
// C line 2108
5 => {
vm_block = if (JS_IsNull(func)) != 0 { 3 } else { 4 }; continue;
}
// C line 2106
6 => {
let _ = free_sh(JS_GetRuntime(ctx), sh);
vm_block = 5; continue;
}
// C line 2105
7 => {
vm_block = if !(sh).is_null() { 6 } else { 5 }; continue;
}
// C line 2104
8 => {
let _ = { let assigned = find_sh(ts, ((sig_num) as i32)); sh = assigned; assigned };
vm_block = 7; continue;
}
// C line 2126
9 => {
let _ = libc::signal(((sig_num) as i32), (os_signal_handler as *const () as usize));
vm_block = 1; continue;
}
// C line 2125
10 => {
let _ = { let assigned = JS_DupValue(ctx, func); (*(sh)).func = assigned; assigned };
vm_block = 9; continue;
}
// C line 2124
11 => {
let _ = JS_FreeValue(ctx, (*(sh)).func);
vm_block = 10; continue;
}
// C line 2122
12 => {
let _ = list_add_tail(core::ptr::addr_of_mut!((*(sh)).link), core::ptr::addr_of_mut!((*(ts)).os_signal_handlers));
vm_block = 11; continue;
}
// C line 2121
13 => {
let _ = { let assigned = ((sig_num) as i32); (*(sh)).sig_num = assigned; assigned };
vm_block = 12; continue;
}
// C line 2120
14 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 2119
15 => {
vm_block = if ((!(!(sh).is_null()) as i32)) != 0 { 14 } else { 13 }; continue;
}
// C line 2118
16 => {
let _ = { let assigned = ((js_mallocz(ctx, (core::mem::size_of::<JSOSSignalHandler>() as usize))) as *mut JSOSSignalHandler); sh = assigned; assigned };
vm_block = 15; continue;
}
// C line 2117
17 => {
vm_block = if ((!(!(sh).is_null()) as i32)) != 0 { 16 } else { 11 }; continue;
}
// C line 2116
18 => {
let _ = { let assigned = find_sh(ts, ((sig_num) as i32)); sh = assigned; assigned };
vm_block = 17; continue;
}
// C line 2115
19 => {
return JS_ThrowTypeError(ctx, c"not a function".as_ptr());
}
// C line 2114
20 => {
vm_block = if ((!((JS_IsFunction(ctx, func)) != 0) as i32)) != 0 { 19 } else { 18 }; continue;
}
// C line 2103
21 => {
vm_block = if (((((JS_IsNull(func)) != 0) || ((JS_IsUndefined(func)) != 0)) as i32)) != 0 { 8 } else { 20 }; continue;
}
// C line 2101
22 => {
let _ = { let assigned = *(argv).offset(((1 as i32)) as isize); func = assigned; assigned };
vm_block = 21; continue;
}
// C line 2100
23 => {
return JS_ThrowRangeError(ctx, c"invalid signal number".as_ptr());
}
// C line 2099
24 => {
vm_block = if ((((sig_num) >= ((((64 as i32)) as u32))) as i32)) != 0 { 23 } else { 22 }; continue;
}
// C line 2098
25 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 2097
26 => {
vm_block = if (JS_ToUint32(ctx, core::ptr::addr_of_mut!(sig_num), *(argv).offset(((0 as i32)) as isize))) != 0 { 25 } else { 24 }; continue;
}
// C line 2095
27 => {
return JS_ThrowTypeError(ctx, c"signal handler can only be set in the main thread".as_ptr());
}
// C line 2094
28 => {
vm_block = if ((!((is_main_thread(rt)) != 0) as i32)) != 0 { 27 } else { 26 }; continue;
}
// C line 2088
29 => {
ts = ((JS_GetRuntimeOpaque(rt)) as *mut JSThreadState);
vm_block = 28; continue;
}
// C line 2087
30 => {
rt = JS_GetRuntime(ctx);
vm_block = 29; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs-libc.c:2175. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
pub unsafe fn js_os_setTimeout(mut ctx: *mut JSContext, mut this_val: JSValue, mut argc: i32, mut argv: *mut JSValue) -> JSValue {

let mut rt: *mut JSRuntime = core::mem::zeroed();
let mut ts: *mut JSThreadState = core::mem::zeroed();
let mut delay: i64 = core::mem::zeroed();
let mut func: JSValue = core::mem::zeroed();
let mut th: *mut JSOSTimer = core::mem::zeroed();
let mut vm_block: usize = 18;
loop { match vm_block {
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
let _ = list_add_tail(core::ptr::addr_of_mut!((*(th)).link), core::ptr::addr_of_mut!((*(ts)).os_timers));
vm_block = 1; continue;
}
// C line 2198
3 => {
let _ = { let assigned = JS_DupValue(ctx, func); (*(th)).func = assigned; assigned };
vm_block = 2; continue;
}
// C line 2197
4 => {
let _ = { let assigned = (get_time_ms()).wrapping_add(delay); (*(th)).timeout = assigned; assigned };
vm_block = 3; continue;
}
// C line 2194
5 => {
let _ = { let assigned = (1 as i32); (*(ts)).next_timer_id = assigned; assigned };
vm_block = 4; continue;
}
// C line 2196
6 => {
let _ = { let old = (*(ts)).next_timer_id; (*(ts)).next_timer_id = ((*(ts)).next_timer_id).wrapping_add(1); old };
vm_block = 4; continue;
}
// C line 2193
7 => {
vm_block = if (((((*(ts)).next_timer_id) == ((2147483647 as i32))) as i32)) != 0 { 5 } else { 6 }; continue;
}
// C line 2192
8 => {
let _ = { let assigned = (*(ts)).next_timer_id; (*(th)).timer_id = assigned; assigned };
vm_block = 7; continue;
}
// C line 2191
9 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 2190
10 => {
vm_block = if ((!(!(th).is_null()) as i32)) != 0 { 9 } else { 8 }; continue;
}
// C line 2189
11 => {
let _ = { let assigned = ((js_mallocz(ctx, (core::mem::size_of::<JSOSTimer>() as usize))) as *mut JSOSTimer); th = assigned; assigned };
vm_block = 10; continue;
}
// C line 2188
12 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 2187
13 => {
vm_block = if (JS_ToInt64(ctx, core::ptr::addr_of_mut!(delay), *(argv).offset(((1 as i32)) as isize))) != 0 { 12 } else { 11 }; continue;
}
// C line 2186
14 => {
return JS_ThrowTypeError(ctx, c"not a function".as_ptr());
}
// C line 2185
15 => {
vm_block = if ((!((JS_IsFunction(ctx, func)) != 0) as i32)) != 0 { 14 } else { 13 }; continue;
}
// C line 2184
16 => {
let _ = { let assigned = *(argv).offset(((0 as i32)) as isize); func = assigned; assigned };
vm_block = 15; continue;
}
// C line 2179
17 => {
ts = ((JS_GetRuntimeOpaque(rt)) as *mut JSThreadState);
vm_block = 16; continue;
}
// C line 2178
18 => {
rt = JS_GetRuntime(ctx);
vm_block = 17; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs-libc.c:2203. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
pub unsafe fn find_timer_by_id(mut ts: *mut JSThreadState, mut timer_id: i32) -> *mut JSOSTimer {

let mut el: *mut list_head = core::mem::zeroed();
let mut th: *mut JSOSTimer = core::mem::zeroed();
let mut vm_block: usize = 9;
loop { match vm_block {
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
vm_block = if ((((el) != (core::ptr::addr_of_mut!((*(ts)).os_timers))) as i32)) != 0 { 6 } else { 1 }; continue;
}
// C line ?
3 => {
let _ = { let assigned = (*(el)).next; el = assigned; assigned };
vm_block = 2; continue;
}
// C line 2211
4 => {
return th;
}
// C line 2210
5 => {
vm_block = if (((((*(th)).timer_id) == (timer_id)) as i32)) != 0 { 4 } else { 3 }; continue;
}
// C line 2209
6 => {
th = (((((el) as *mut u8)).offset(-(((core::mem::offset_of!(JSOSTimer, link) as usize)) as isize))) as *mut JSOSTimer);
vm_block = 5; continue;
}
// C line ?
7 => {
let _ = { let assigned = (*(core::ptr::addr_of_mut!((*(ts)).os_timers))).next; el = assigned; assigned };
vm_block = 2; continue;
}
// C line 2207
8 => {
return core::ptr::null_mut::<JSOSTimer>();
}
// C line 2206
9 => {
vm_block = if ((((timer_id) <= ((0 as i32))) as i32)) != 0 { 8 } else { 7 }; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs-libc.c:2216. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
pub unsafe fn js_os_clearTimeout(mut ctx: *mut JSContext, mut this_val: JSValue, mut argc: i32, mut argv: *mut JSValue) -> JSValue {

let mut rt: *mut JSRuntime = core::mem::zeroed();
let mut ts: *mut JSThreadState = core::mem::zeroed();
let mut th: *mut JSOSTimer = core::mem::zeroed();
let mut timer_id: i32 = core::mem::zeroed();
let mut vm_block: usize = 9;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 2230
1 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) };
}
// C line 2229
2 => {
let _ = free_timer(rt, th);
vm_block = 1; continue;
}
// C line 2228
3 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_UNDEFINED as i32)) as i64) };
}
// C line 2227
4 => {
vm_block = if ((!(!(th).is_null()) as i32)) != 0 { 3 } else { 2 }; continue;
}
// C line 2226
5 => {
let _ = { let assigned = find_timer_by_id(ts, timer_id); th = assigned; assigned };
vm_block = 4; continue;
}
// C line 2225
6 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 2224
7 => {
vm_block = if (JS_ToInt32(ctx, core::ptr::addr_of_mut!(timer_id), *(argv).offset(((0 as i32)) as isize))) != 0 { 6 } else { 5 }; continue;
}
// C line 2220
8 => {
ts = ((JS_GetRuntimeOpaque(rt)) as *mut JSThreadState);
vm_block = 7; continue;
}
// C line 2219
9 => {
rt = JS_GetRuntime(ctx);
vm_block = 8; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs-libc.c:2234. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
pub unsafe fn js_os_sleepAsync(mut ctx: *mut JSContext, mut this_val: JSValue, mut argc: i32, mut argv: *mut JSValue) -> JSValue {

let mut rt: *mut JSRuntime = core::mem::zeroed();
let mut ts: *mut JSThreadState = core::mem::zeroed();
let mut delay: i64 = core::mem::zeroed();
let mut th: *mut JSOSTimer = core::mem::zeroed();
let mut promise: JSValue = core::mem::zeroed();
let mut resolving_funcs: [JSValue; 2] = core::mem::zeroed();
let mut vm_block: usize = 20;
loop { match vm_block {
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
let _ = JS_FreeValue(ctx, *((resolving_funcs).as_mut_ptr()).offset(((1 as i32)) as isize));
vm_block = 1; continue;
}
// C line 2260
3 => {
let _ = JS_FreeValue(ctx, *((resolving_funcs).as_mut_ptr()).offset(((0 as i32)) as isize));
vm_block = 2; continue;
}
// C line 2259
4 => {
let _ = list_add_tail(core::ptr::addr_of_mut!((*(th)).link), core::ptr::addr_of_mut!((*(ts)).os_timers));
vm_block = 3; continue;
}
// C line 2258
5 => {
let _ = { let assigned = JS_DupValue(ctx, *((resolving_funcs).as_mut_ptr()).offset(((0 as i32)) as isize)); (*(th)).func = assigned; assigned };
vm_block = 4; continue;
}
// C line 2257
6 => {
let _ = { let assigned = (get_time_ms()).wrapping_add(delay); (*(th)).timeout = assigned; assigned };
vm_block = 5; continue;
}
// C line 2256
7 => {
let _ = { let assigned = ((1 as i32)).wrapping_neg(); (*(th)).timer_id = assigned; assigned };
vm_block = 6; continue;
}
// C line 2254
8 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 2253
9 => {
let _ = JS_FreeValue(ctx, *((resolving_funcs).as_mut_ptr()).offset(((1 as i32)) as isize));
vm_block = 8; continue;
}
// C line 2252
10 => {
let _ = JS_FreeValue(ctx, *((resolving_funcs).as_mut_ptr()).offset(((0 as i32)) as isize));
vm_block = 9; continue;
}
// C line 2251
11 => {
let _ = JS_FreeValue(ctx, promise);
vm_block = 10; continue;
}
// C line 2250
12 => {
vm_block = if ((!(!(th).is_null()) as i32)) != 0 { 11 } else { 7 }; continue;
}
// C line 2249
13 => {
let _ = { let assigned = ((js_mallocz(ctx, (core::mem::size_of::<JSOSTimer>() as usize))) as *mut JSOSTimer); th = assigned; assigned };
vm_block = 12; continue;
}
// C line 2247
14 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 2246
15 => {
vm_block = if (JS_IsException(promise)) != 0 { 14 } else { 13 }; continue;
}
// C line 2245
16 => {
let _ = { let assigned = JS_NewPromiseCapability(ctx, (resolving_funcs).as_mut_ptr()); promise = assigned; assigned };
vm_block = 15; continue;
}
// C line 2244
17 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 2243
18 => {
vm_block = if (JS_ToInt64(ctx, core::ptr::addr_of_mut!(delay), *(argv).offset(((0 as i32)) as isize))) != 0 { 17 } else { 16 }; continue;
}
// C line 2238
19 => {
ts = ((JS_GetRuntimeOpaque(rt)) as *mut JSThreadState);
vm_block = 18; continue;
}
// C line 2237
20 => {
rt = JS_GetRuntime(ctx);
vm_block = 19; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs-libc.c:2660. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
pub unsafe fn make_obj_error(mut ctx: *mut JSContext, mut obj: JSValue, mut err: i32) -> JSValue {

let mut arr: JSValue = core::mem::zeroed();
let mut vm_block: usize = 8;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 2674
1 => {
return arr;
}
// C line 2672
2 => {
let _ = JS_DefinePropertyValueUint32(ctx, arr, (((1 as i32)) as u32), JS_NewInt32(ctx, err), ((((((1 as i32)).wrapping_shl(((0 as i32)) as u32)) | (((1 as i32)).wrapping_shl(((1 as i32)) as u32)))) | (((1 as i32)).wrapping_shl(((2 as i32)) as u32))));
vm_block = 1; continue;
}
// C line 2670
3 => {
let _ = JS_DefinePropertyValueUint32(ctx, arr, (((0 as i32)) as u32), obj, ((((((1 as i32)).wrapping_shl(((0 as i32)) as u32)) | (((1 as i32)).wrapping_shl(((1 as i32)) as u32)))) | (((1 as i32)).wrapping_shl(((2 as i32)) as u32))));
vm_block = 2; continue;
}
// C line 2669
4 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 2668
5 => {
vm_block = if (JS_IsException(arr)) != 0 { 4 } else { 3 }; continue;
}
// C line 2667
6 => {
let _ = { let assigned = JS_NewArray(ctx); arr = assigned; assigned };
vm_block = 5; continue;
}
// C line 2666
7 => {
return obj;
}
// C line 2665
8 => {
vm_block = if (JS_IsException(obj)) != 0 { 7 } else { 6 }; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs-libc.c:2677. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
pub unsafe fn make_string_error(mut ctx: *mut JSContext, mut buf: *const c_char, mut err: i32) -> JSValue {

let mut vm_block: usize = 1;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 2681
1 => {
return make_obj_error(ctx, JS_NewString(ctx, buf), err);
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs-libc.c:2685. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
pub unsafe fn js_os_getcwd(mut ctx: *mut JSContext, mut this_val: JSValue, mut argc: i32, mut argv: *mut JSValue) -> JSValue {

let mut buf: [c_char; OS_PATH_MAX] = core::mem::zeroed();
let mut err: i32 = core::mem::zeroed();
let mut vm_block: usize = 5;
loop { match vm_block {
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
let _ = { let assigned = *(stdio_errno_pointer()); err = assigned; assigned };
vm_block = 1; continue;
}
// C line 2692
3 => {
let _ = { let assigned = (((0 as i32)) as c_char); *((buf).as_mut_ptr()).offset(((0 as i32)) as isize) = assigned; assigned };
vm_block = 2; continue;
}
// C line 2695
4 => {
let _ = { let assigned = (0 as i32); err = assigned; assigned };
vm_block = 1; continue;
}
// C line 2691
5 => {
vm_block = if ((!(!(libc::getcwd((buf).as_mut_ptr(), (core::mem::size_of::<[c_char; OS_PATH_MAX]>() as usize))).is_null()) as i32)) != 0 { 3 } else { 4 }; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs-libc.c:2700. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
pub unsafe fn js_os_chdir(mut ctx: *mut JSContext, mut this_val: JSValue, mut argc: i32, mut argv: *mut JSValue) -> JSValue {

let mut target: *const c_char = core::mem::zeroed();
let mut err: i32 = core::mem::zeroed();
let mut vm_block: usize = 6;
loop { match vm_block {
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
vm_block = 1; continue;
}
// C line 2709
3 => {
let _ = { let assigned = ((js_get_errno(((libc::chdir(target)) as isize))) as i32); err = assigned; assigned };
vm_block = 2; continue;
}
// C line 2708
4 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 2707
5 => {
vm_block = if ((!(!(target).is_null()) as i32)) != 0 { 4 } else { 3 }; continue;
}
// C line 2706
6 => {
let _ = { let assigned = JS_ToCString(ctx, *(argv).offset(((0 as i32)) as isize)); target = assigned; assigned };
vm_block = 5; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs-libc.c:2714. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
pub unsafe fn js_os_mkdir(mut ctx: *mut JSContext, mut this_val: JSValue, mut argc: i32, mut argv: *mut JSValue) -> JSValue {

let mut mode: i32 = core::mem::zeroed();
let mut ret: i32 = core::mem::zeroed();
let mut path: *const c_char = core::mem::zeroed();
let mut vm_block: usize = 10;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 2736
1 => {
return JS_NewInt32(ctx, ret);
}
// C line 2735
2 => {
let _ = JS_FreeCString(ctx, path);
vm_block = 1; continue;
}
// C line 2733
3 => {
let _ = { let assigned = ((js_get_errno(((libc::mkdir(path, ((mode) as libc::mode_t))) as isize))) as i32); ret = assigned; assigned };
vm_block = 2; continue;
}
// C line 2728
4 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 2727
5 => {
vm_block = if ((!(!(path).is_null()) as i32)) != 0 { 4 } else { 3 }; continue;
}
// C line 2726
6 => {
let _ = { let assigned = JS_ToCString(ctx, *(argv).offset(((0 as i32)) as isize)); path = assigned; assigned };
vm_block = 5; continue;
}
// C line 2722
7 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 2721
8 => {
vm_block = if (JS_ToInt32(ctx, core::ptr::addr_of_mut!(mode), *(argv).offset(((1 as i32)) as isize))) != 0 { 7 } else { 6 }; continue;
}
// C line 2724
9 => {
let _ = { let assigned = (511 as i32); mode = assigned; assigned };
vm_block = 6; continue;
}
// C line 2720
10 => {
vm_block = if ((((argc) >= ((2 as i32))) as i32)) != 0 { 8 } else { 9 }; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs-libc.c:2740. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
pub unsafe fn js_os_readdir(mut ctx: *mut JSContext, mut this_val: JSValue, mut argc: i32, mut argv: *mut JSValue) -> JSValue {

let mut path: *const c_char = core::mem::zeroed();
let mut f: *mut libc::DIR = core::mem::zeroed();
let mut d: *mut libc::dirent = core::mem::zeroed();
let mut obj: JSValue = core::mem::zeroed();
let mut err: i32 = core::mem::zeroed();
let mut len: u32 = core::mem::zeroed();
let mut vm_block: usize = 24;
loop { match vm_block {
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
vm_block = 1; continue;
}
// C line 2767
3 => {
vm_block = 9; continue;
}
// C line 2774
4 => {
let _ = JS_DefinePropertyValueUint32(ctx, obj, { let old = len; len = (len).wrapping_add(1); old }, JS_NewString(ctx, ((*(d)).d_name).as_mut_ptr()), ((((((1 as i32)).wrapping_shl(((0 as i32)) as u32)) | (((1 as i32)).wrapping_shl(((1 as i32)) as u32)))) | (((1 as i32)).wrapping_shl(((2 as i32)) as u32))));
vm_block = 3; continue;
}
// C line 2772
5 => {
vm_block = 2; continue;
}
// C line 2771
6 => {
let _ = { let assigned = *(stdio_errno_pointer()); err = assigned; assigned };
vm_block = 5; continue;
}
// C line 2770
7 => {
vm_block = if ((!(!(d).is_null()) as i32)) != 0 { 6 } else { 4 }; continue;
}
// C line 2769
8 => {
let _ = { let assigned = libc::readdir(f); d = assigned; assigned };
vm_block = 7; continue;
}
// C line 2768
9 => {
let _ = { let assigned = (0 as i32); *(stdio_errno_pointer()) = assigned; assigned };
vm_block = 8; continue;
}
// C line 2766
10 => {
let _ = { let assigned = (((0 as i32)) as u32); len = assigned; assigned };
vm_block = 3; continue;
}
// C line 2765
11 => {
vm_block = 1; continue;
}
// C line 2764
12 => {
vm_block = if ((!(!(f).is_null()) as i32)) != 0 { 11 } else { 10 }; continue;
}
// C line 2763
13 => {
let _ = JS_FreeCString(ctx, path);
vm_block = 12; continue;
}
// C line 2760
14 => {
let _ = { let assigned = *(stdio_errno_pointer()); err = assigned; assigned };
vm_block = 13; continue;
}
// C line 2762
15 => {
let _ = { let assigned = (0 as i32); err = assigned; assigned };
vm_block = 13; continue;
}
// C line 2759
16 => {
vm_block = if ((!(!(f).is_null()) as i32)) != 0 { 14 } else { 15 }; continue;
}
// C line 2758
17 => {
let _ = { let assigned = libc::opendir(path); f = assigned; assigned };
vm_block = 16; continue;
}
// C line 2756
18 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 2755
19 => {
let _ = JS_FreeCString(ctx, path);
vm_block = 18; continue;
}
// C line 2754
20 => {
vm_block = if (JS_IsException(obj)) != 0 { 19 } else { 17 }; continue;
}
// C line 2753
21 => {
let _ = { let assigned = JS_NewArray(ctx); obj = assigned; assigned };
vm_block = 20; continue;
}
// C line 2752
22 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 2751
23 => {
vm_block = if ((!(!(path).is_null()) as i32)) != 0 { 22 } else { 21 }; continue;
}
// C line 2750
24 => {
let _ = { let assigned = JS_ToCString(ctx, *(argv).offset(((0 as i32)) as isize)); path = assigned; assigned };
vm_block = 23; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs-libc.c:2784. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
pub unsafe fn timespec_to_ms(mut tv: *const libc::timespec) -> i64 {

let mut vm_block: usize = 1;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 2786
1 => {
return (((((*(tv)).tv_sec) as i64)).wrapping_mul((((1000 as i32)) as i64))).wrapping_add((((*(tv)).tv_nsec) / ((((1000000 as i32)) as i64))));
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs-libc.c:2886. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
pub unsafe fn ms_to_timeval(mut tv: *mut libc::timeval, mut v: u64) -> () {

let mut vm_block: usize = 2;
loop { match vm_block {
// C line ?
0 => {
return ();
}
// C line 2889
1 => {
let _ = { let assigned = (((((v) % ((((1000 as i32)) as u64)))).wrapping_mul((((1000 as i32)) as u64))) as libc::suseconds_t); (*(tv)).tv_usec = assigned; assigned };
vm_block = 0; continue;
}
// C line 2888
2 => {
let _ = { let assigned = ((((v) / ((((1000 as i32)) as u64)))) as libc::time_t); (*(tv)).tv_sec = assigned; assigned };
vm_block = 1; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs-libc.c:2893. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
pub unsafe fn js_os_utimes(mut ctx: *mut JSContext, mut this_val: JSValue, mut argc: i32, mut argv: *mut JSValue) -> JSValue {

let mut path: *const c_char = core::mem::zeroed();
let mut atime: i64 = core::mem::zeroed();
let mut mtime: i64 = core::mem::zeroed();
let mut ret: i32 = core::mem::zeroed();
let mut times: [libc::timeval; 2] = core::mem::zeroed();
let mut vm_block: usize = 12;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 2923
1 => {
return JS_NewInt32(ctx, ret);
}
// C line 2922
2 => {
let _ = JS_FreeCString(ctx, path);
vm_block = 1; continue;
}
// C line 2919
3 => {
let _ = { let assigned = ((js_get_errno(((libc::utimes(path, (times).as_mut_ptr())) as isize))) as i32); ret = assigned; assigned };
vm_block = 2; continue;
}
// C line 2918
4 => {
let _ = ms_to_timeval(core::ptr::addr_of_mut!(*((times).as_mut_ptr()).offset(((1 as i32)) as isize)), ((mtime) as u64));
vm_block = 3; continue;
}
// C line 2917
5 => {
let _ = ms_to_timeval(core::ptr::addr_of_mut!(*((times).as_mut_ptr()).offset(((0 as i32)) as isize)), ((atime) as u64));
vm_block = 4; continue;
}
// C line 2906
6 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 2905
7 => {
vm_block = if ((!(!(path).is_null()) as i32)) != 0 { 6 } else { 5 }; continue;
}
// C line 2904
8 => {
let _ = { let assigned = JS_ToCString(ctx, *(argv).offset(((0 as i32)) as isize)); path = assigned; assigned };
vm_block = 7; continue;
}
// C line 2903
9 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 2902
10 => {
vm_block = if (JS_ToInt64(ctx, core::ptr::addr_of_mut!(mtime), *(argv).offset(((2 as i32)) as isize))) != 0 { 9 } else { 8 }; continue;
}
// C line 2901
11 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 2900
12 => {
vm_block = if (JS_ToInt64(ctx, core::ptr::addr_of_mut!(atime), *(argv).offset(((1 as i32)) as isize))) != 0 { 11 } else { 10 }; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs-libc.c:2927. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
pub unsafe fn js_os_sleep(mut ctx: *mut JSContext, mut this_val: JSValue, mut argc: i32, mut argv: *mut JSValue) -> JSValue {

let mut delay: i64 = core::mem::zeroed();
let mut ret: i32 = core::mem::zeroed();
let mut ts: libc::timespec = core::mem::zeroed();
let mut vm_block: usize = 8;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 2953
1 => {
return JS_NewInt32(ctx, ret);
}
// C line 2950
2 => {
let _ = { let assigned = ((js_get_errno(((libc::nanosleep(core::ptr::addr_of_mut!(ts), core::ptr::null_mut::<libc::timespec>())) as isize))) as i32); ret = assigned; assigned };
vm_block = 1; continue;
}
// C line 2949
3 => {
let _ = { let assigned = (((delay) % ((((1000 as i32)) as i64)))).wrapping_mul((((1000000 as i32)) as i64)); (ts).tv_nsec = assigned; assigned };
vm_block = 2; continue;
}
// C line 2948
4 => {
let _ = { let assigned = ((((delay) / ((((1000 as i32)) as i64)))) as libc::time_t); (ts).tv_sec = assigned; assigned };
vm_block = 3; continue;
}
// C line 2936
5 => {
let _ = { let assigned = (((0 as i32)) as i64); delay = assigned; assigned };
vm_block = 4; continue;
}
// C line 2935
6 => {
vm_block = if ((((delay) < ((((0 as i32)) as i64))) as i32)) != 0 { 5 } else { 4 }; continue;
}
// C line 2934
7 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 2933
8 => {
vm_block = if (JS_ToInt64(ctx, core::ptr::addr_of_mut!(delay), *(argv).offset(((0 as i32)) as isize))) != 0 { 7 } else { 6 }; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs-libc.c:2969. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
pub unsafe fn js_os_realpath(mut ctx: *mut JSContext, mut this_val: JSValue, mut argc: i32, mut argv: *mut JSValue) -> JSValue {

let mut path: *const c_char = core::mem::zeroed();
let mut buf: [c_char; OS_PATH_MAX] = core::mem::zeroed();
let mut res: *mut c_char = core::mem::zeroed();
let mut err: i32 = core::mem::zeroed();
let mut vm_block: usize = 10;
loop { match vm_block {
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
let _ = { let assigned = *(stdio_errno_pointer()); err = assigned; assigned };
vm_block = 1; continue;
}
// C line 2982
3 => {
let _ = { let assigned = (((0 as i32)) as c_char); *((buf).as_mut_ptr()).offset(((0 as i32)) as isize) = assigned; assigned };
vm_block = 2; continue;
}
// C line 2985
4 => {
let _ = { let assigned = (0 as i32); err = assigned; assigned };
vm_block = 1; continue;
}
// C line 2981
5 => {
vm_block = if ((!(!(res).is_null()) as i32)) != 0 { 3 } else { 4 }; continue;
}
// C line 2980
6 => {
let _ = JS_FreeCString(ctx, path);
vm_block = 5; continue;
}
// C line 2979
7 => {
let _ = { let assigned = libc::realpath(path, (buf).as_mut_ptr()); res = assigned; assigned };
vm_block = 6; continue;
}
// C line 2978
8 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 2977
9 => {
vm_block = if ((!(!(path).is_null()) as i32)) != 0 { 8 } else { 7 }; continue;
}
// C line 2976
10 => {
let _ = { let assigned = JS_ToCString(ctx, *(argv).offset(((0 as i32)) as isize)); path = assigned; assigned };
vm_block = 9; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs-libc.c:2991. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
pub unsafe fn js_os_symlink(mut ctx: *mut JSContext, mut this_val: JSValue, mut argc: i32, mut argv: *mut JSValue) -> JSValue {

let mut target: *const c_char = core::mem::zeroed();
let mut linkpath: *const c_char = core::mem::zeroed();
let mut err: i32 = core::mem::zeroed();
let mut vm_block: usize = 11;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 3008
1 => {
return JS_NewInt32(ctx, err);
}
// C line 3007
2 => {
let _ = JS_FreeCString(ctx, linkpath);
vm_block = 1; continue;
}
// C line 3006
3 => {
let _ = JS_FreeCString(ctx, target);
vm_block = 2; continue;
}
// C line 3005
4 => {
let _ = { let assigned = ((js_get_errno(((libc::symlink(target, linkpath)) as isize))) as i32); err = assigned; assigned };
vm_block = 3; continue;
}
// C line 3003
5 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 3002
6 => {
let _ = JS_FreeCString(ctx, target);
vm_block = 5; continue;
}
// C line 3001
7 => {
vm_block = if ((!(!(linkpath).is_null()) as i32)) != 0 { 6 } else { 4 }; continue;
}
// C line 3000
8 => {
let _ = { let assigned = JS_ToCString(ctx, *(argv).offset(((1 as i32)) as isize)); linkpath = assigned; assigned };
vm_block = 7; continue;
}
// C line 2999
9 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 2998
10 => {
vm_block = if ((!(!(target).is_null()) as i32)) != 0 { 9 } else { 8 }; continue;
}
// C line 2997
11 => {
let _ = { let assigned = JS_ToCString(ctx, *(argv).offset(((0 as i32)) as isize)); target = assigned; assigned };
vm_block = 10; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs-libc.c:3012. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
pub unsafe fn js_os_readlink(mut ctx: *mut JSContext, mut this_val: JSValue, mut argc: i32, mut argv: *mut JSValue) -> JSValue {

let mut path: *const c_char = core::mem::zeroed();
let mut buf: [c_char; OS_PATH_MAX] = core::mem::zeroed();
let mut err: i32 = core::mem::zeroed();
let mut res: isize = core::mem::zeroed();
let mut vm_block: usize = 11;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 3032
1 => {
return make_string_error(ctx, (buf).as_mut_ptr(), err);
}
// C line 3031
2 => {
let _ = JS_FreeCString(ctx, path);
vm_block = 1; continue;
}
// C line 3026
3 => {
let _ = { let assigned = *(stdio_errno_pointer()); err = assigned; assigned };
vm_block = 2; continue;
}
// C line 3025
4 => {
let _ = { let assigned = (((0 as i32)) as c_char); *((buf).as_mut_ptr()).offset(((0 as i32)) as isize) = assigned; assigned };
vm_block = 3; continue;
}
// C line 3029
5 => {
let _ = { let assigned = (0 as i32); err = assigned; assigned };
vm_block = 2; continue;
}
// C line 3028
6 => {
let _ = { let assigned = (((0 as i32)) as c_char); *((buf).as_mut_ptr()).offset((res) as isize) = assigned; assigned };
vm_block = 5; continue;
}
// C line 3024
7 => {
vm_block = if ((((res) < ((((0 as i32)) as isize))) as i32)) != 0 { 4 } else { 6 }; continue;
}
// C line 3023
8 => {
let _ = { let assigned = libc::readlink(path, (buf).as_mut_ptr(), ((core::mem::size_of::<[c_char; OS_PATH_MAX]>() as usize)).wrapping_sub((((1 as i32)) as usize))); res = assigned; assigned };
vm_block = 7; continue;
}
// C line 3022
9 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 3021
10 => {
vm_block = if ((!(!(path).is_null()) as i32)) != 0 { 9 } else { 8 }; continue;
}
// C line 3020
11 => {
let _ = { let assigned = JS_ToCString(ctx, *(argv).offset(((0 as i32)) as isize)); path = assigned; assigned };
vm_block = 10; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs-libc.c:3035. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
pub unsafe fn build_envp(mut ctx: *mut JSContext, mut obj: JSValue) -> *mut *mut c_char {

let mut len: u32 = core::mem::zeroed();
let mut i: u32 = core::mem::zeroed();
let mut tab: *mut JSPropertyEnum = core::mem::zeroed();
let mut envp: *mut *mut c_char = core::mem::zeroed();
let mut pair: *mut c_char = core::mem::zeroed();
let mut key: *const c_char = core::mem::zeroed();
let mut str: *const c_char = core::mem::zeroed();
let mut val: JSValue = core::mem::zeroed();
let mut key_len: usize = core::mem::zeroed();
let mut str_len: usize = core::mem::zeroed();
let mut vm_block: usize = 43;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 3089
1 => {
vm_block = 10; continue;
}
// C line 3087
2 => {
let _ = { let assigned = core::ptr::null_mut::<*mut c_char>(); envp = assigned; assigned };
vm_block = 1; continue;
}
// C line 3086
3 => {
let _ = js_free(ctx, ((envp) as *mut c_void));
vm_block = 2; continue;
}
// C line 3084
4 => {
vm_block = if ((((i) < (len)) as i32)) != 0 { 6 } else { 3 }; continue;
}
// C line ?
5 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 4; continue;
}
// C line 3085
6 => {
let _ = js_free(ctx, ((*(envp).offset((i) as isize)) as *mut c_void));
vm_block = 5; continue;
}
// C line 3084
7 => {
let _ = { let assigned = (((0 as i32)) as u32); i = assigned; assigned };
vm_block = 4; continue;
}
// C line 3083 labels: fail
8 => {
vm_block = if !(envp).is_null() { 7 } else { 1 }; continue;
}
// C line 3081
9 => {
return envp;
}
// C line ? labels: done
10 => {
let _ = JS_FreePropertyEnum(ctx, tab, len);
vm_block = 9; continue;
}
// C line 3050
11 => {
vm_block = if ((((i) < (len)) as i32)) != 0 { 37 } else { 10 }; continue;
}
// C line ?
12 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 11; continue;
}
// C line 3077
13 => {
let _ = JS_FreeCString(ctx, str);
vm_block = 12; continue;
}
// C line 3076
14 => {
let _ = JS_FreeCString(ctx, key);
vm_block = 13; continue;
}
// C line 3075
15 => {
let _ = { let assigned = pair; *(envp).offset((i) as isize) = assigned; assigned };
vm_block = 14; continue;
}
// C line 3074
16 => {
let _ = { let assigned = (((0 as i32)) as c_char); *(pair).offset((((key_len).wrapping_add((((1 as i32)) as usize))).wrapping_add(str_len)) as isize) = assigned; assigned };
vm_block = 15; continue;
}
// C line 3073
17 => {
let _ = { let dst = (((((pair).offset(((key_len) as isize))).offset((((1 as i32)) as isize))) as *mut c_void)) as *mut u8; core::ptr::copy_nonoverlapping((((str) as *const c_void)) as *const u8, dst, (str_len) as usize); dst as *mut c_void };
vm_block = 16; continue;
}
// C line 3072
18 => {
let _ = { let assigned = (((61 as i32)) as c_char); *(pair).offset((key_len) as isize) = assigned; assigned };
vm_block = 17; continue;
}
// C line 3071
19 => {
let _ = { let dst = (((pair) as *mut c_void)) as *mut u8; core::ptr::copy_nonoverlapping((((key) as *const c_void)) as *const u8, dst, (key_len) as usize); dst as *mut c_void };
vm_block = 18; continue;
}
// C line 3069
20 => {
vm_block = 8; continue;
}
// C line 3068
21 => {
let _ = JS_FreeCString(ctx, str);
vm_block = 20; continue;
}
// C line 3067
22 => {
let _ = JS_FreeCString(ctx, key);
vm_block = 21; continue;
}
// C line 3066
23 => {
vm_block = if ((!(!(pair).is_null()) as i32)) != 0 { 22 } else { 19 }; continue;
}
// C line 3065
24 => {
let _ = { let assigned = ((js_malloc(ctx, ((key_len).wrapping_add(str_len)).wrapping_add((((2 as i32)) as usize)))) as *mut c_char); pair = assigned; assigned };
vm_block = 23; continue;
}
// C line 3064
25 => {
let _ = { let assigned = libc::strlen(str); str_len = assigned; assigned };
vm_block = 24; continue;
}
// C line 3063
26 => {
let _ = { let assigned = libc::strlen(key); key_len = assigned; assigned };
vm_block = 25; continue;
}
// C line 3061
27 => {
vm_block = 8; continue;
}
// C line 3060
28 => {
let _ = JS_FreeCString(ctx, str);
vm_block = 27; continue;
}
// C line 3059
29 => {
vm_block = if ((!(!(key).is_null()) as i32)) != 0 { 28 } else { 26 }; continue;
}
// C line 3058
30 => {
let _ = { let assigned = JS_AtomToCString(ctx, (*(tab).offset((i) as isize)).atom); key = assigned; assigned };
vm_block = 29; continue;
}
// C line 3057
31 => {
vm_block = 8; continue;
}
// C line 3056
32 => {
vm_block = if ((!(!(str).is_null()) as i32)) != 0 { 31 } else { 30 }; continue;
}
// C line 3055
33 => {
let _ = JS_FreeValue(ctx, val);
vm_block = 32; continue;
}
// C line 3054
34 => {
let _ = { let assigned = JS_ToCString(ctx, val); str = assigned; assigned };
vm_block = 33; continue;
}
// C line 3053
35 => {
vm_block = 8; continue;
}
// C line 3052
36 => {
vm_block = if (JS_IsException(val)) != 0 { 35 } else { 34 }; continue;
}
// C line 3051
37 => {
let _ = { let assigned = JS_GetProperty(ctx, obj, (*(tab).offset((i) as isize)).atom); val = assigned; assigned };
vm_block = 36; continue;
}
// C line 3050
38 => {
let _ = { let assigned = (((0 as i32)) as u32); i = assigned; assigned };
vm_block = 11; continue;
}
// C line 3049
39 => {
vm_block = 8; continue;
}
// C line 3048
40 => {
vm_block = if ((!(!(envp).is_null()) as i32)) != 0 { 39 } else { 38 }; continue;
}
// C line 3047
41 => {
let _ = { let assigned = ((js_mallocz(ctx, ((core::mem::size_of::<*mut c_char>() as usize)).wrapping_mul((((len) as usize)).wrapping_add((((1 as i32)) as usize))))) as *mut *mut c_char); envp = assigned; assigned };
vm_block = 40; continue;
}
// C line 3046
42 => {
return core::ptr::null_mut::<*mut c_char>();
}
// C line 3044
43 => {
vm_block = if ((((JS_GetOwnPropertyNames(ctx, core::ptr::addr_of_mut!(tab), core::ptr::addr_of_mut!(len), obj, ((((1 as i32)).wrapping_shl(((0 as i32)) as u32)) | (((1 as i32)).wrapping_shl(((4 as i32)) as u32))))) < ((0 as i32))) as i32)) != 0 { 42 } else { 41 }; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs-libc.c:3093. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
pub unsafe fn my_execvpe(mut filename: *const c_char, mut argv: *mut *mut c_char, mut envp: *mut *mut c_char) -> i32 {

let mut path: *mut c_char = core::mem::zeroed();
let mut p: *mut c_char = core::mem::zeroed();
let mut p_next: *mut c_char = core::mem::zeroed();
let mut p1: *mut c_char = core::mem::zeroed();
let mut buf: [c_char; OS_PATH_MAX] = core::mem::zeroed();
let mut filename_len: usize = core::mem::zeroed();
let mut path_len: usize = core::mem::zeroed();
let mut eacces_error: i32 = core::mem::zeroed();
let mut vm_block: usize = 35;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 3145
1 => {
return ((1 as i32)).wrapping_neg();
}
// C line 3144
2 => {
let _ = { let assigned = (13 as i32); *(stdio_errno_pointer()) = assigned; assigned };
vm_block = 1; continue;
}
// C line 3143
3 => {
vm_block = if (eacces_error) != 0 { 2 } else { 1 }; continue;
}
// C line 3113
4 => {
vm_block = if ((((p) != (core::ptr::null_mut::<c_char>())) as i32)) != 0 { 23 } else { 3 }; continue;
}
// C line ?
5 => {
let _ = { let assigned = p_next; p = assigned; assigned };
vm_block = 4; continue;
}
// C line ?
6 => {
return ((1 as i32)).wrapping_neg();
}
// C line 3138
7 => {
vm_block = 5; continue;
}
// C line 3135
8 => {
vm_block = 5; continue;
}
// C line 3134
9 => {
let _ = { let assigned = quickjs::cutils_header::TRUE; eacces_error = assigned; assigned };
vm_block = 8; continue;
}
// C line 3132
10 => {
vm_block = match *(stdio_errno_pointer()) { x if x == (20 as i32) => 7, x if x == (2 as i32) => 7, x if x == (13 as i32) => 9, _ => 6, }; continue;
}
// C line 3130
11 => {
let _ = os_execve((buf).as_mut_ptr(), argv, envp);
vm_block = 10; continue;
}
// C line 3128
12 => {
let _ = { let assigned = (((0 as i32)) as c_char); *((buf).as_mut_ptr()).offset((((path_len).wrapping_add((((1 as i32)) as usize))).wrapping_add(filename_len)) as isize) = assigned; assigned };
vm_block = 11; continue;
}
// C line 3127
13 => {
let _ = { let dst = ((((((buf).as_mut_ptr()).offset(((path_len) as isize))).offset((((1 as i32)) as isize))) as *mut c_void)) as *mut u8; core::ptr::copy_nonoverlapping((((filename) as *const c_void)) as *const u8, dst, (filename_len) as usize); dst as *mut c_void };
vm_block = 12; continue;
}
// C line 3126
14 => {
let _ = { let assigned = (((47 as i32)) as c_char); *((buf).as_mut_ptr()).offset((path_len) as isize) = assigned; assigned };
vm_block = 13; continue;
}
// C line 3125
15 => {
let _ = { let dst = ((((buf).as_mut_ptr()) as *mut c_void)) as *mut u8; core::ptr::copy_nonoverlapping((((p) as *const c_void)) as *const u8, dst, (path_len) as usize); dst as *mut c_void };
vm_block = 14; continue;
}
// C line 3124
16 => {
vm_block = 5; continue;
}
// C line 3123
17 => {
vm_block = if (((((((path_len).wrapping_add((((1 as i32)) as usize))).wrapping_add(filename_len)).wrapping_add((((1 as i32)) as usize))) > ((((OS_PATH_MAX as i32)) as usize))) as i32)) != 0 { 16 } else { 15 }; continue;
}
// C line 3117
18 => {
let _ = { let assigned = libc::strlen(p); path_len = assigned; assigned };
vm_block = 17; continue;
}
// C line 3116
19 => {
let _ = { let assigned = core::ptr::null_mut::<c_char>(); p_next = assigned; assigned };
vm_block = 18; continue;
}
// C line 3120
20 => {
let _ = { let assigned = ((((p1).offset_from(p) as i64)) as usize); path_len = assigned; assigned };
vm_block = 17; continue;
}
// C line 3119
21 => {
let _ = { let assigned = (p1).offset((((1 as i32)) as isize)); p_next = assigned; assigned };
vm_block = 20; continue;
}
// C line 3115
22 => {
vm_block = if ((!(!(p1).is_null()) as i32)) != 0 { 19 } else { 21 }; continue;
}
// C line 3114
23 => {
let _ = { let assigned = libc::strchr(p, (58 as i32)); p1 = assigned; assigned };
vm_block = 22; continue;
}
// C line 3113
24 => {
let _ = { let assigned = path; p = assigned; assigned };
vm_block = 4; continue;
}
// C line 3112
25 => {
let _ = { let assigned = path; p = assigned; assigned };
vm_block = 24; continue;
}
// C line 3111
26 => {
let _ = { let assigned = quickjs::cutils_header::FALSE; eacces_error = assigned; assigned };
vm_block = 25; continue;
}
// C line 3110
27 => {
let _ = { let assigned = c"/bin:/usr/bin".as_ptr().cast_mut(); path = assigned; assigned };
vm_block = 26; continue;
}
// C line 3109
28 => {
vm_block = if ((!(!(path).is_null()) as i32)) != 0 { 27 } else { 26 }; continue;
}
// C line 3108
29 => {
let _ = { let assigned = libc::getenv(c"PATH".as_ptr()); path = assigned; assigned };
vm_block = 28; continue;
}
// C line 3106
30 => {
return os_execve(filename, argv, envp);
}
// C line 3105
31 => {
vm_block = if !(libc::strchr(filename, (47 as i32))).is_null() { 30 } else { 29 }; continue;
}
// C line 3103
32 => {
return ((1 as i32)).wrapping_neg();
}
// C line 3102
33 => {
let _ = { let assigned = (2 as i32); *(stdio_errno_pointer()) = assigned; assigned };
vm_block = 32; continue;
}
// C line 3101
34 => {
vm_block = if ((((filename_len) == ((((0 as i32)) as usize))) as i32)) != 0 { 33 } else { 31 }; continue;
}
// C line 3100
35 => {
let _ = { let assigned = libc::strlen(filename); filename_len = assigned; assigned };
vm_block = 34; continue;
}
_ => std::process::abort(),
} }
}

#[cfg(not(feature = "host-have-closefrom"))]
// Generated from the official C AST; original goto CFG retained.
// c: quickjs-libc.c:3149. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
pub unsafe fn js_os_exec(mut ctx: *mut JSContext, mut this_val: JSValue, mut argc: i32, mut argv: *mut JSValue) -> JSValue {

let mut options: JSValue = core::mem::zeroed();
let mut args: JSValue = core::mem::zeroed();
let mut val: JSValue = core::mem::zeroed();
let mut ret_val: JSValue = core::mem::zeroed();
let mut exec_argv: *mut *const c_char = core::mem::zeroed();
let mut file: *const c_char = core::mem::zeroed();
let mut str: *const c_char = core::mem::zeroed();
let mut cwd: *const c_char = core::mem::zeroed();
let mut envp: *mut *mut c_char = core::mem::zeroed();
let mut exec_argc: u32 = core::mem::zeroed();
let mut i: u32 = core::mem::zeroed();
let mut ret: i32 = core::mem::zeroed();
let mut pid: i32 = core::mem::zeroed();
let mut status: i32 = core::mem::zeroed();
let mut block_flag: i32 = core::mem::zeroed();
let mut use_path: i32 = core::mem::zeroed();
let mut std_name: [*const c_char; 3] = core::mem::zeroed();
let mut std_fds: [i32; 3] = core::mem::zeroed();
let mut uid: u32 = core::mem::zeroed();
let mut gid: u32 = core::mem::zeroed();
let mut fd: i32 = core::mem::zeroed();
let mut fd_max: i32 = core::mem::zeroed();
let mut p: *mut *mut c_char = core::mem::zeroed();
let mut vm_block: usize = 151;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 3361
1 => {
vm_block = 16; continue;
}
// C line ? labels: exception
2 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) }; ret_val = assigned; assigned };
vm_block = 1; continue;
}
// C line 3358
3 => {
return ret_val;
}
// C line 3356
4 => {
let _ = js_free(ctx, ((envp) as *mut c_void));
vm_block = 3; continue;
}
// C line 3352
5 => {
vm_block = if ((((*(p)) != (core::ptr::null_mut::<c_char>())) as i32)) != 0 { 7 } else { 4 }; continue;
}
// C line 3354
6 => {
let _ = { let old = p; p = (p).offset(1); old };
vm_block = 5; continue;
}
// C line 3353
7 => {
let _ = js_free(ctx, ((*(p)) as *mut c_void));
vm_block = 6; continue;
}
// C line 3351
8 => {
let _ = { let assigned = envp; p = assigned; assigned };
vm_block = 5; continue;
}
// C line 3349
9 => {
vm_block = if ((((!(envp).is_null()) && (((((envp) != (*(os_environ_pointer()))) as i32)) != 0)) as i32)) != 0 { 8 } else { 3 }; continue;
}
// C line 3348
10 => {
let _ = js_free(ctx, ((exec_argv) as *mut c_void));
vm_block = 9; continue;
}
// C line 3346
11 => {
vm_block = if ((((i) < (exec_argc)) as i32)) != 0 { 13 } else { 10 }; continue;
}
// C line ?
12 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 11; continue;
}
// C line 3347
13 => {
let _ = JS_FreeCString(ctx, *(exec_argv).offset((i) as isize));
vm_block = 12; continue;
}
// C line 3346
14 => {
let _ = { let assigned = (((0 as i32)) as u32); i = assigned; assigned };
vm_block = 11; continue;
}
// C line 3345
15 => {
let _ = JS_FreeCString(ctx, cwd);
vm_block = 14; continue;
}
// C line ? labels: done
16 => {
let _ = JS_FreeCString(ctx, file);
vm_block = 15; continue;
}
// C line 3342
17 => {
let _ = { let assigned = JS_NewInt32(ctx, ret); ret_val = assigned; assigned };
vm_block = 16; continue;
}
// C line 3327
18 => {
vm_block = 26; continue;
}
// C line 3332
19 => {
vm_block = 17; continue;
}
// C line 3331
20 => {
let _ = { let assigned = (((*(core::ptr::addr_of_mut!(status))).wrapping_shr(((8 as i32)) as u32)) & ((255 as i32))); ret = assigned; assigned };
vm_block = 19; continue;
}
// C line 3335
21 => {
vm_block = 17; continue;
}
// C line 3334
22 => {
let _ = { let assigned = (((*(core::ptr::addr_of_mut!(status))) & ((127 as i32)))).wrapping_neg(); ret = assigned; assigned };
vm_block = 21; continue;
}
// C line 3333
23 => {
vm_block = if ((((((((((*(core::ptr::addr_of_mut!(status))) & ((127 as i32)))) != ((127 as i32))) as i32)) != 0) && (((((((*(core::ptr::addr_of_mut!(status))) & ((127 as i32)))) != ((0 as i32))) as i32)) != 0)) as i32)) != 0 { 22 } else { 18 }; continue;
}
// C line 3330
24 => {
vm_block = if ((((((*(core::ptr::addr_of_mut!(status))) & ((127 as i32)))) == ((0 as i32))) as i32)) != 0 { 20 } else { 23 }; continue;
}
// C line 3329
25 => {
vm_block = if ((((ret) == (pid)) as i32)) != 0 { 24 } else { 18 }; continue;
}
// C line 3328
26 => {
let _ = { let assigned = libc::waitpid(pid, core::ptr::addr_of_mut!(status), (0 as i32)); ret = assigned; assigned };
vm_block = 25; continue;
}
// C line 3340
27 => {
let _ = { let assigned = pid; ret = assigned; assigned };
vm_block = 17; continue;
}
// C line 3326
28 => {
vm_block = if (block_flag) != 0 { 18 } else { 27 }; continue;
}
// C line 3323
29 => {
let _ = libc::_exit((127 as i32));
vm_block = 28; continue;
}
// C line 3320
30 => {
let _ = { let assigned = my_execvpe(file, ((exec_argv) as *mut *mut c_char), envp); ret = assigned; assigned };
vm_block = 29; continue;
}
// C line 3322
31 => {
let _ = { let assigned = os_execve(file, ((exec_argv) as *mut *mut c_char), envp); ret = assigned; assigned };
vm_block = 29; continue;
}
// C line 3319
32 => {
vm_block = if (use_path) != 0 { 30 } else { 31 }; continue;
}
// C line 3318
33 => {
let _ = { let assigned = *(exec_argv).offset(((0 as i32)) as isize); file = assigned; assigned };
vm_block = 32; continue;
}
// C line 3317
34 => {
vm_block = if ((!(!(file).is_null()) as i32)) != 0 { 33 } else { 32 }; continue;
}
// C line 3314
35 => {
let _ = libc::_exit((127 as i32));
vm_block = 34; continue;
}
// C line 3313
36 => {
vm_block = if ((((libc::setuid(uid)) < ((0 as i32))) as i32)) != 0 { 35 } else { 34 }; continue;
}
// C line 3312
37 => {
vm_block = if ((((uid) != (((((1 as i32)).wrapping_neg()) as u32))) as i32)) != 0 { 36 } else { 34 }; continue;
}
// C line 3310
38 => {
let _ = libc::_exit((127 as i32));
vm_block = 37; continue;
}
// C line 3309
39 => {
vm_block = if ((((libc::setgid(gid)) < ((0 as i32))) as i32)) != 0 { 38 } else { 37 }; continue;
}
// C line 3308
40 => {
vm_block = if ((((gid) != (((((1 as i32)).wrapping_neg()) as u32))) as i32)) != 0 { 39 } else { 37 }; continue;
}
// C line 3306
41 => {
let _ = libc::_exit((127 as i32));
vm_block = 40; continue;
}
// C line 3305
42 => {
vm_block = if ((((libc::chdir(cwd)) < ((0 as i32))) as i32)) != 0 { 41 } else { 40 }; continue;
}
// C line 3304
43 => {
vm_block = if !(cwd).is_null() { 42 } else { 40 }; continue;
}
// C line 3300
44 => {
vm_block = if ((((i) < (((fd_max) as u32))) as i32)) != 0 { 46 } else { 43 }; continue;
}
// C line ?
45 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 44; continue;
}
// C line 3301
46 => {
let _ = libc::close(((i) as i32));
vm_block = 45; continue;
}
// C line 3300
47 => {
let _ = { let assigned = (((3 as i32)) as u32); i = assigned; assigned };
vm_block = 44; continue;
}
// C line 3299
48 => {
fd_max = quickjs::cutils_header::min_int(((libc::sysconf((libc::_SC_OPEN_MAX as i32))) as i32), (1024 as i32));
vm_block = 47; continue;
}
// C line 3276
49 => {
vm_block = if ((((i) < ((((3 as i32)) as u32))) as i32)) != 0 { 53 } else { 48 }; continue;
}
// C line ?
50 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 49; continue;
}
// C line 3279
51 => {
let _ = libc::_exit((127 as i32));
vm_block = 50; continue;
}
// C line 3278
52 => {
vm_block = if ((((libc::dup2(*((std_fds).as_mut_ptr()).offset((i) as isize), ((i) as i32))) < ((0 as i32))) as i32)) != 0 { 51 } else { 50 }; continue;
}
// C line 3277
53 => {
vm_block = if ((((((*((std_fds).as_mut_ptr()).offset((i) as isize)) as u32)) != (i)) as i32)) != 0 { 52 } else { 50 }; continue;
}
// C line 3276
54 => {
let _ = { let assigned = (((0 as i32)) as u32); i = assigned; assigned };
vm_block = 49; continue;
}
// C line 3272
55 => {
vm_block = if ((((pid) == ((0 as i32))) as i32)) != 0 { 54 } else { 28 }; continue;
}
// C line 3270
56 => {
vm_block = 2; continue;
}
// C line 3269
57 => {
let _ = JS_ThrowTypeError(ctx, c"fork error".as_ptr());
vm_block = 56; continue;
}
// C line 3268
58 => {
vm_block = if ((((pid) < ((0 as i32))) as i32)) != 0 { 57 } else { 55 }; continue;
}
// C line 3267
59 => {
let _ = { let assigned = libc::fork(); pid = assigned; assigned };
vm_block = 58; continue;
}
// C line 3263
60 => {
vm_block = 2; continue;
}
// C line 3262
61 => {
vm_block = if (ret) != 0 { 60 } else { 59 }; continue;
}
// C line 3261
62 => {
let _ = JS_FreeValue(ctx, val);
vm_block = 61; continue;
}
// C line 3260
63 => {
let _ = { let assigned = JS_ToUint32(ctx, core::ptr::addr_of_mut!(gid), val); ret = assigned; assigned };
vm_block = 62; continue;
}
// C line 3259
64 => {
vm_block = if ((!((JS_IsUndefined(val)) != 0) as i32)) != 0 { 63 } else { 59 }; continue;
}
// C line 3258
65 => {
vm_block = 2; continue;
}
// C line 3257
66 => {
vm_block = if (JS_IsException(val)) != 0 { 65 } else { 64 }; continue;
}
// C line 3256
67 => {
let _ = { let assigned = JS_GetPropertyStr(ctx, options, c"gid".as_ptr()); val = assigned; assigned };
vm_block = 66; continue;
}
// C line 3253
68 => {
vm_block = 2; continue;
}
// C line 3252
69 => {
vm_block = if (ret) != 0 { 68 } else { 67 }; continue;
}
// C line 3251
70 => {
let _ = JS_FreeValue(ctx, val);
vm_block = 69; continue;
}
// C line 3250
71 => {
let _ = { let assigned = JS_ToUint32(ctx, core::ptr::addr_of_mut!(uid), val); ret = assigned; assigned };
vm_block = 70; continue;
}
// C line 3249
72 => {
vm_block = if ((!((JS_IsUndefined(val)) != 0) as i32)) != 0 { 71 } else { 67 }; continue;
}
// C line 3248
73 => {
vm_block = 2; continue;
}
// C line 3247
74 => {
vm_block = if (JS_IsException(val)) != 0 { 73 } else { 72 }; continue;
}
// C line 3246
75 => {
let _ = { let assigned = JS_GetPropertyStr(ctx, options, c"uid".as_ptr()); val = assigned; assigned };
vm_block = 74; continue;
}
// C line 3243
76 => {
vm_block = 2; continue;
}
// C line 3242
77 => {
vm_block = if ((!(!(envp).is_null()) as i32)) != 0 { 76 } else { 75 }; continue;
}
// C line 3241
78 => {
let _ = JS_FreeValue(ctx, val);
vm_block = 77; continue;
}
// C line 3240
79 => {
let _ = { let assigned = build_envp(ctx, val); envp = assigned; assigned };
vm_block = 78; continue;
}
// C line 3239
80 => {
vm_block = if ((!((JS_IsUndefined(val)) != 0) as i32)) != 0 { 79 } else { 75 }; continue;
}
// C line 3238
81 => {
vm_block = 2; continue;
}
// C line 3237
82 => {
vm_block = if (JS_IsException(val)) != 0 { 81 } else { 80 }; continue;
}
// C line 3236
83 => {
let _ = { let assigned = JS_GetPropertyStr(ctx, options, c"env".as_ptr()); val = assigned; assigned };
vm_block = 82; continue;
}
// C line 3222
84 => {
vm_block = if ((((i) < ((((3 as i32)) as u32))) as i32)) != 0 { 94 } else { 83 }; continue;
}
// C line ?
85 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 84; continue;
}
// C line 3232
86 => {
let _ = { let assigned = fd; *((std_fds).as_mut_ptr()).offset((i) as isize) = assigned; assigned };
vm_block = 85; continue;
}
// C line 3231
87 => {
vm_block = 2; continue;
}
// C line 3230
88 => {
vm_block = if (ret) != 0 { 87 } else { 86 }; continue;
}
// C line 3229
89 => {
let _ = JS_FreeValue(ctx, val);
vm_block = 88; continue;
}
// C line 3228
90 => {
let _ = { let assigned = JS_ToInt32(ctx, core::ptr::addr_of_mut!(fd), val); ret = assigned; assigned };
vm_block = 89; continue;
}
// C line 3226
91 => {
vm_block = if ((!((JS_IsUndefined(val)) != 0) as i32)) != 0 { 90 } else { 85 }; continue;
}
// C line 3225
92 => {
vm_block = 2; continue;
}
// C line 3224
93 => {
vm_block = if (JS_IsException(val)) != 0 { 92 } else { 91 }; continue;
}
// C line 3223
94 => {
let _ = { let assigned = JS_GetPropertyStr(ctx, options, *((std_name).as_ptr()).offset((i) as isize)); val = assigned; assigned };
vm_block = 93; continue;
}
// C line 3222
95 => {
let _ = { let assigned = (((0 as i32)) as u32); i = assigned; assigned };
vm_block = 84; continue;
}
// C line 3218
96 => {
vm_block = 2; continue;
}
// C line 3217
97 => {
vm_block = if ((!(!(cwd).is_null()) as i32)) != 0 { 96 } else { 95 }; continue;
}
// C line 3216
98 => {
let _ = JS_FreeValue(ctx, val);
vm_block = 97; continue;
}
// C line 3215
99 => {
let _ = { let assigned = JS_ToCString(ctx, val); cwd = assigned; assigned };
vm_block = 98; continue;
}
// C line 3214
100 => {
vm_block = if ((!((JS_IsUndefined(val)) != 0) as i32)) != 0 { 99 } else { 95 }; continue;
}
// C line 3213
101 => {
vm_block = 2; continue;
}
// C line 3212
102 => {
vm_block = if (JS_IsException(val)) != 0 { 101 } else { 100 }; continue;
}
// C line 3211
103 => {
let _ = { let assigned = JS_GetPropertyStr(ctx, options, c"cwd".as_ptr()); val = assigned; assigned };
vm_block = 102; continue;
}
// C line 3208
104 => {
vm_block = 2; continue;
}
// C line 3207
105 => {
vm_block = if ((!(!(file).is_null()) as i32)) != 0 { 104 } else { 103 }; continue;
}
// C line 3206
106 => {
let _ = JS_FreeValue(ctx, val);
vm_block = 105; continue;
}
// C line 3205
107 => {
let _ = { let assigned = JS_ToCString(ctx, val); file = assigned; assigned };
vm_block = 106; continue;
}
// C line 3204
108 => {
vm_block = if ((!((JS_IsUndefined(val)) != 0) as i32)) != 0 { 107 } else { 103 }; continue;
}
// C line 3203
109 => {
vm_block = 2; continue;
}
// C line 3202
110 => {
vm_block = if (JS_IsException(val)) != 0 { 109 } else { 108 }; continue;
}
// C line 3201
111 => {
let _ = { let assigned = JS_GetPropertyStr(ctx, options, c"file".as_ptr()); val = assigned; assigned };
vm_block = 110; continue;
}
// C line 3199
112 => {
vm_block = 2; continue;
}
// C line 3198
113 => {
vm_block = if (stdio_bool_option(ctx, core::ptr::addr_of_mut!(use_path), options, c"usePath".as_ptr())) != 0 { 112 } else { 111 }; continue;
}
// C line 3197
114 => {
vm_block = 2; continue;
}
// C line 3196
115 => {
vm_block = if (stdio_bool_option(ctx, core::ptr::addr_of_mut!(block_flag), options, c"block".as_ptr())) != 0 { 114 } else { 113 }; continue;
}
// C line 3194
116 => {
let _ = { let assigned = *(argv).offset(((1 as i32)) as isize); options = assigned; assigned };
vm_block = 115; continue;
}
// C line 3193
117 => {
vm_block = if ((((argc) >= ((2 as i32))) as i32)) != 0 { 116 } else { 59 }; continue;
}
// C line 3189
118 => {
vm_block = if ((((i) < ((((3 as i32)) as u32))) as i32)) != 0 { 120 } else { 117 }; continue;
}
// C line ?
119 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 118; continue;
}
// C line 3190
120 => {
let _ = { let assigned = ((i) as i32); *((std_fds).as_mut_ptr()).offset((i) as isize) = assigned; assigned };
vm_block = 119; continue;
}
// C line 3189
121 => {
let _ = { let assigned = (((0 as i32)) as u32); i = assigned; assigned };
vm_block = 118; continue;
}
// C line 3187
122 => {
let _ = { let assigned = core::ptr::null_mut::<c_char>(); *(exec_argv).offset((exec_argc) as isize) = assigned; assigned };
vm_block = 121; continue;
}
// C line 3177
123 => {
vm_block = if ((((i) < (exec_argc)) as i32)) != 0 { 132 } else { 122 }; continue;
}
// C line ?
124 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 123; continue;
}
// C line 3185
125 => {
let _ = { let assigned = str; *(exec_argv).offset((i) as isize) = assigned; assigned };
vm_block = 124; continue;
}
// C line 3184
126 => {
vm_block = 2; continue;
}
// C line 3183
127 => {
vm_block = if ((!(!(str).is_null()) as i32)) != 0 { 126 } else { 125 }; continue;
}
// C line 3182
128 => {
let _ = JS_FreeValue(ctx, val);
vm_block = 127; continue;
}
// C line 3181
129 => {
let _ = { let assigned = JS_ToCString(ctx, val); str = assigned; assigned };
vm_block = 128; continue;
}
// C line 3180
130 => {
vm_block = 2; continue;
}
// C line 3179
131 => {
vm_block = if (JS_IsException(val)) != 0 { 130 } else { 129 }; continue;
}
// C line 3178
132 => {
let _ = { let assigned = JS_GetPropertyUint32(ctx, args, i); val = assigned; assigned };
vm_block = 131; continue;
}
// C line 3177
133 => {
let _ = { let assigned = (((0 as i32)) as u32); i = assigned; assigned };
vm_block = 123; continue;
}
// C line 3176
134 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 3175
135 => {
vm_block = if ((!(!(exec_argv).is_null()) as i32)) != 0 { 134 } else { 133 }; continue;
}
// C line 3174
136 => {
let _ = { let assigned = ((js_mallocz(ctx, ((core::mem::size_of::<*const c_char>() as usize)).wrapping_mul((((exec_argc).wrapping_add((((1 as i32)) as u32))) as usize)))) as *mut *const c_char); exec_argv = assigned; assigned };
vm_block = 135; continue;
}
// C line 3172
137 => {
return JS_ThrowTypeError(ctx, c"invalid number of arguments".as_ptr());
}
// C line 3171
138 => {
vm_block = if ((((((((exec_argc) < ((((1 as i32)) as u32))) as i32)) != 0) || (((((exec_argc) > ((((65535 as i32)) as u32))) as i32)) != 0)) as i32)) != 0 { 137 } else { 136 }; continue;
}
// C line 3169
139 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 3168
140 => {
vm_block = if (ret) != 0 { 139 } else { 138 }; continue;
}
// C line 3167
141 => {
let _ = JS_FreeValue(ctx, val);
vm_block = 140; continue;
}
// C line 3166
142 => {
let _ = { let assigned = JS_ToUint32(ctx, core::ptr::addr_of_mut!(exec_argc), val); ret = assigned; assigned };
vm_block = 141; continue;
}
// C line 3165
143 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 3164
144 => {
vm_block = if (JS_IsException(val)) != 0 { 143 } else { 142 }; continue;
}
// C line 3163
145 => {
let _ = { let assigned = JS_GetPropertyStr(ctx, args, c"length".as_ptr()); val = assigned; assigned };
vm_block = 144; continue;
}
// C line 3161
146 => {
uid = ((((1 as i32)).wrapping_neg()) as u32);
gid = ((((1 as i32)).wrapping_neg()) as u32);
vm_block = 145; continue;
}
// C line 3159
147 => {
std_name = [c"stdin".as_ptr(), c"stdout".as_ptr(), c"stderr".as_ptr()];
vm_block = 146; continue;
}
// C line 3158
148 => {
block_flag = quickjs::cutils_header::TRUE;
use_path = quickjs::cutils_header::TRUE;
vm_block = 147; continue;
}
// C line 3155
149 => {
envp = *(os_environ_pointer());
vm_block = 148; continue;
}
// C line 3154
150 => {
file = core::ptr::null_mut::<c_char>();
cwd = core::ptr::null_mut::<c_char>();
vm_block = 149; continue;
}
// C line 3152
151 => {
args = *(argv).offset(((0 as i32)) as isize);
vm_block = 150; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs-libc.c:3365. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
pub unsafe fn js_os_getpid(mut ctx: *mut JSContext, mut this_val: JSValue, mut argc: i32, mut argv: *mut JSValue) -> JSValue {

let mut vm_block: usize = 1;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 3368
1 => {
return JS_NewInt32(ctx, libc::getpid());
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs-libc.c:3372. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
pub unsafe fn js_os_waitpid(mut ctx: *mut JSContext, mut this_val: JSValue, mut argc: i32, mut argv: *mut JSValue) -> JSValue {

let mut pid: i32 = core::mem::zeroed();
let mut status: i32 = core::mem::zeroed();
let mut options: i32 = core::mem::zeroed();
let mut ret: i32 = core::mem::zeroed();
let mut obj: JSValue = core::mem::zeroed();
let mut vm_block: usize = 14;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 3396
1 => {
return obj;
}
// C line 3394
2 => {
let _ = JS_DefinePropertyValueUint32(ctx, obj, (((1 as i32)) as u32), JS_NewInt32(ctx, status), ((((((1 as i32)).wrapping_shl(((0 as i32)) as u32)) | (((1 as i32)).wrapping_shl(((1 as i32)) as u32)))) | (((1 as i32)).wrapping_shl(((2 as i32)) as u32))));
vm_block = 1; continue;
}
// C line 3392
3 => {
let _ = JS_DefinePropertyValueUint32(ctx, obj, (((0 as i32)) as u32), JS_NewInt32(ctx, ret), ((((((1 as i32)).wrapping_shl(((0 as i32)) as u32)) | (((1 as i32)).wrapping_shl(((1 as i32)) as u32)))) | (((1 as i32)).wrapping_shl(((2 as i32)) as u32))));
vm_block = 2; continue;
}
// C line 3391
4 => {
return obj;
}
// C line 3390
5 => {
vm_block = if (JS_IsException(obj)) != 0 { 4 } else { 3 }; continue;
}
// C line 3389
6 => {
let _ = { let assigned = JS_NewArray(ctx); obj = assigned; assigned };
vm_block = 5; continue;
}
// C line 3386
7 => {
let _ = { let assigned = (0 as i32); status = assigned; assigned };
vm_block = 6; continue;
}
// C line 3385
8 => {
let _ = { let assigned = (*(stdio_errno_pointer())).wrapping_neg(); ret = assigned; assigned };
vm_block = 7; continue;
}
// C line 3384
9 => {
vm_block = if ((((ret) < ((0 as i32))) as i32)) != 0 { 8 } else { 6 }; continue;
}
// C line 3383
10 => {
let _ = { let assigned = libc::waitpid(pid, core::ptr::addr_of_mut!(status), options); ret = assigned; assigned };
vm_block = 9; continue;
}
// C line 3381
11 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 3380
12 => {
vm_block = if (JS_ToInt32(ctx, core::ptr::addr_of_mut!(options), *(argv).offset(((1 as i32)) as isize))) != 0 { 11 } else { 10 }; continue;
}
// C line 3379
13 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 3378
14 => {
vm_block = if (JS_ToInt32(ctx, core::ptr::addr_of_mut!(pid), *(argv).offset(((0 as i32)) as isize))) != 0 { 13 } else { 12 }; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs-libc.c:3400. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
pub unsafe fn js_os_pipe(mut ctx: *mut JSContext, mut this_val: JSValue, mut argc: i32, mut argv: *mut JSValue) -> JSValue {

let mut pipe_fds: [i32; 2] = core::mem::zeroed();
let mut ret: i32 = core::mem::zeroed();
let mut obj: JSValue = core::mem::zeroed();
let mut vm_block: usize = 9;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 3416
1 => {
return obj;
}
// C line 3414
2 => {
let _ = JS_DefinePropertyValueUint32(ctx, obj, (((1 as i32)) as u32), JS_NewInt32(ctx, *((pipe_fds).as_mut_ptr()).offset(((1 as i32)) as isize)), ((((((1 as i32)).wrapping_shl(((0 as i32)) as u32)) | (((1 as i32)).wrapping_shl(((1 as i32)) as u32)))) | (((1 as i32)).wrapping_shl(((2 as i32)) as u32))));
vm_block = 1; continue;
}
// C line 3412
3 => {
let _ = JS_DefinePropertyValueUint32(ctx, obj, (((0 as i32)) as u32), JS_NewInt32(ctx, *((pipe_fds).as_mut_ptr()).offset(((0 as i32)) as isize)), ((((((1 as i32)).wrapping_shl(((0 as i32)) as u32)) | (((1 as i32)).wrapping_shl(((1 as i32)) as u32)))) | (((1 as i32)).wrapping_shl(((2 as i32)) as u32))));
vm_block = 2; continue;
}
// C line 3411
4 => {
return obj;
}
// C line 3410
5 => {
vm_block = if (JS_IsException(obj)) != 0 { 4 } else { 3 }; continue;
}
// C line 3409
6 => {
let _ = { let assigned = JS_NewArray(ctx); obj = assigned; assigned };
vm_block = 5; continue;
}
// C line 3408
7 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_NULL as i32)) as i64) };
}
// C line 3407
8 => {
vm_block = if ((((ret) < ((0 as i32))) as i32)) != 0 { 7 } else { 6 }; continue;
}
// C line 3406
9 => {
let _ = { let assigned = libc::pipe((pipe_fds).as_mut_ptr()); ret = assigned; assigned };
vm_block = 8; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs-libc.c:3420. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
pub unsafe fn js_os_kill(mut ctx: *mut JSContext, mut this_val: JSValue, mut argc: i32, mut argv: *mut JSValue) -> JSValue {

let mut pid: i32 = core::mem::zeroed();
let mut sig: i32 = core::mem::zeroed();
let mut ret: i32 = core::mem::zeroed();
let mut vm_block: usize = 6;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 3430
1 => {
return JS_NewInt32(ctx, ret);
}
// C line 3429
2 => {
let _ = { let assigned = ((js_get_errno(((libc::kill(pid, sig)) as isize))) as i32); ret = assigned; assigned };
vm_block = 1; continue;
}
// C line 3428
3 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 3427
4 => {
vm_block = if (JS_ToInt32(ctx, core::ptr::addr_of_mut!(sig), *(argv).offset(((1 as i32)) as isize))) != 0 { 3 } else { 2 }; continue;
}
// C line 3426
5 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 3425
6 => {
vm_block = if (JS_ToInt32(ctx, core::ptr::addr_of_mut!(pid), *(argv).offset(((0 as i32)) as isize))) != 0 { 5 } else { 4 }; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs-libc.c:3434. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
pub unsafe fn js_os_dup(mut ctx: *mut JSContext, mut this_val: JSValue, mut argc: i32, mut argv: *mut JSValue) -> JSValue {

let mut fd: i32 = core::mem::zeroed();
let mut ret: i32 = core::mem::zeroed();
let mut vm_block: usize = 4;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 3442
1 => {
return JS_NewInt32(ctx, ret);
}
// C line 3441
2 => {
let _ = { let assigned = ((js_get_errno(((libc::dup(fd)) as isize))) as i32); ret = assigned; assigned };
vm_block = 1; continue;
}
// C line 3440
3 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 3439
4 => {
vm_block = if (JS_ToInt32(ctx, core::ptr::addr_of_mut!(fd), *(argv).offset(((0 as i32)) as isize))) != 0 { 3 } else { 2 }; continue;
}
_ => std::process::abort(),
} }
}

// Generated from the official C AST; original goto CFG retained.
// c: quickjs-libc.c:3446. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
pub unsafe fn js_os_dup2(mut ctx: *mut JSContext, mut this_val: JSValue, mut argc: i32, mut argv: *mut JSValue) -> JSValue {

let mut fd: i32 = core::mem::zeroed();
let mut fd2: i32 = core::mem::zeroed();
let mut ret: i32 = core::mem::zeroed();
let mut vm_block: usize = 6;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 3456
1 => {
return JS_NewInt32(ctx, ret);
}
// C line 3455
2 => {
let _ = { let assigned = ((js_get_errno(((libc::dup2(fd, fd2)) as isize))) as i32); ret = assigned; assigned };
vm_block = 1; continue;
}
// C line 3454
3 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 3453
4 => {
vm_block = if (JS_ToInt32(ctx, core::ptr::addr_of_mut!(fd2), *(argv).offset(((1 as i32)) as isize))) != 0 { 3 } else { 2 }; continue;
}
// C line 3452
5 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 3451
6 => {
vm_block = if (JS_ToInt32(ctx, core::ptr::addr_of_mut!(fd), *(argv).offset(((0 as i32)) as isize))) != 0 { 5 } else { 4 }; continue;
}
_ => std::process::abort(),
} }
}

#[cfg(feature = "host-have-closefrom")]
// Generated from the official C AST; original goto CFG retained.
// c: quickjs-libc.c:3149. Bellard/Gordon MIT.
#[allow(unused_parens, unused_mut, unused_assignments, unreachable_code)]
pub unsafe fn js_os_exec(mut ctx: *mut JSContext, mut this_val: JSValue, mut argc: i32, mut argv: *mut JSValue) -> JSValue {

let mut options: JSValue = core::mem::zeroed();
let mut args: JSValue = core::mem::zeroed();
let mut val: JSValue = core::mem::zeroed();
let mut ret_val: JSValue = core::mem::zeroed();
let mut exec_argv: *mut *const c_char = core::mem::zeroed();
let mut file: *const c_char = core::mem::zeroed();
let mut str: *const c_char = core::mem::zeroed();
let mut cwd: *const c_char = core::mem::zeroed();
let mut envp: *mut *mut c_char = core::mem::zeroed();
let mut exec_argc: u32 = core::mem::zeroed();
let mut i: u32 = core::mem::zeroed();
let mut ret: i32 = core::mem::zeroed();
let mut pid: i32 = core::mem::zeroed();
let mut status: i32 = core::mem::zeroed();
let mut block_flag: i32 = core::mem::zeroed();
let mut use_path: i32 = core::mem::zeroed();
let mut std_name: [*const c_char; 3] = core::mem::zeroed();
let mut std_fds: [i32; 3] = core::mem::zeroed();
let mut uid: u32 = core::mem::zeroed();
let mut gid: u32 = core::mem::zeroed();
let mut fd: i32 = core::mem::zeroed();
let mut p: *mut *mut c_char = core::mem::zeroed();
let mut vm_block: usize = 147;
loop { match vm_block {
// C line ?
0 => {
std::process::abort();
}
// C line 3361
1 => {
vm_block = 16; continue;
}
// C line ? labels: exception
2 => {
let _ = { let assigned = JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) }; ret_val = assigned; assigned };
vm_block = 1; continue;
}
// C line 3358
3 => {
return ret_val;
}
// C line 3356
4 => {
let _ = js_free(ctx, ((envp) as *mut c_void));
vm_block = 3; continue;
}
// C line 3352
5 => {
vm_block = if ((((*(p)) != (core::ptr::null_mut::<c_char>())) as i32)) != 0 { 7 } else { 4 }; continue;
}
// C line 3354
6 => {
let _ = { let old = p; p = (p).offset(1); old };
vm_block = 5; continue;
}
// C line 3353
7 => {
let _ = js_free(ctx, ((*(p)) as *mut c_void));
vm_block = 6; continue;
}
// C line 3351
8 => {
let _ = { let assigned = envp; p = assigned; assigned };
vm_block = 5; continue;
}
// C line 3349
9 => {
vm_block = if ((((!(envp).is_null()) && (((((envp) != (*(os_environ_pointer()))) as i32)) != 0)) as i32)) != 0 { 8 } else { 3 }; continue;
}
// C line 3348
10 => {
let _ = js_free(ctx, ((exec_argv) as *mut c_void));
vm_block = 9; continue;
}
// C line 3346
11 => {
vm_block = if ((((i) < (exec_argc)) as i32)) != 0 { 13 } else { 10 }; continue;
}
// C line ?
12 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 11; continue;
}
// C line 3347
13 => {
let _ = JS_FreeCString(ctx, *(exec_argv).offset((i) as isize));
vm_block = 12; continue;
}
// C line 3346
14 => {
let _ = { let assigned = (((0 as i32)) as u32); i = assigned; assigned };
vm_block = 11; continue;
}
// C line 3345
15 => {
let _ = JS_FreeCString(ctx, cwd);
vm_block = 14; continue;
}
// C line ? labels: done
16 => {
let _ = JS_FreeCString(ctx, file);
vm_block = 15; continue;
}
// C line 3342
17 => {
let _ = { let assigned = JS_NewInt32(ctx, ret); ret_val = assigned; assigned };
vm_block = 16; continue;
}
// C line 3327
18 => {
vm_block = 26; continue;
}
// C line 3332
19 => {
vm_block = 17; continue;
}
// C line 3331
20 => {
let _ = { let assigned = (((*(core::ptr::addr_of_mut!(status))).wrapping_shr(((8 as i32)) as u32)) & ((255 as i32))); ret = assigned; assigned };
vm_block = 19; continue;
}
// C line 3335
21 => {
vm_block = 17; continue;
}
// C line 3334
22 => {
let _ = { let assigned = (((*(core::ptr::addr_of_mut!(status))) & ((127 as i32)))).wrapping_neg(); ret = assigned; assigned };
vm_block = 21; continue;
}
// C line 3333
23 => {
vm_block = if ((((((((((*(core::ptr::addr_of_mut!(status))) & ((127 as i32)))) != ((127 as i32))) as i32)) != 0) && (((((((*(core::ptr::addr_of_mut!(status))) & ((127 as i32)))) != ((0 as i32))) as i32)) != 0)) as i32)) != 0 { 22 } else { 18 }; continue;
}
// C line 3330
24 => {
vm_block = if ((((((*(core::ptr::addr_of_mut!(status))) & ((127 as i32)))) == ((0 as i32))) as i32)) != 0 { 20 } else { 23 }; continue;
}
// C line 3329
25 => {
vm_block = if ((((ret) == (pid)) as i32)) != 0 { 24 } else { 18 }; continue;
}
// C line 3328
26 => {
let _ = { let assigned = libc::waitpid(pid, core::ptr::addr_of_mut!(status), (0 as i32)); ret = assigned; assigned };
vm_block = 25; continue;
}
// C line 3340
27 => {
let _ = { let assigned = pid; ret = assigned; assigned };
vm_block = 17; continue;
}
// C line 3326
28 => {
vm_block = if (block_flag) != 0 { 18 } else { 27 }; continue;
}
// C line 3323
29 => {
let _ = libc::_exit((127 as i32));
vm_block = 28; continue;
}
// C line 3320
30 => {
let _ = { let assigned = my_execvpe(file, ((exec_argv) as *mut *mut c_char), envp); ret = assigned; assigned };
vm_block = 29; continue;
}
// C line 3322
31 => {
let _ = { let assigned = os_execve(file, ((exec_argv) as *mut *mut c_char), envp); ret = assigned; assigned };
vm_block = 29; continue;
}
// C line 3319
32 => {
vm_block = if (use_path) != 0 { 30 } else { 31 }; continue;
}
// C line 3318
33 => {
let _ = { let assigned = *(exec_argv).offset(((0 as i32)) as isize); file = assigned; assigned };
vm_block = 32; continue;
}
// C line 3317
34 => {
vm_block = if ((!(!(file).is_null()) as i32)) != 0 { 33 } else { 32 }; continue;
}
// C line 3314
35 => {
let _ = libc::_exit((127 as i32));
vm_block = 34; continue;
}
// C line 3313
36 => {
vm_block = if ((((libc::setuid(uid)) < ((0 as i32))) as i32)) != 0 { 35 } else { 34 }; continue;
}
// C line 3312
37 => {
vm_block = if ((((uid) != (((((1 as i32)).wrapping_neg()) as u32))) as i32)) != 0 { 36 } else { 34 }; continue;
}
// C line 3310
38 => {
let _ = libc::_exit((127 as i32));
vm_block = 37; continue;
}
// C line 3309
39 => {
vm_block = if ((((libc::setgid(gid)) < ((0 as i32))) as i32)) != 0 { 38 } else { 37 }; continue;
}
// C line 3308
40 => {
vm_block = if ((((gid) != (((((1 as i32)).wrapping_neg()) as u32))) as i32)) != 0 { 39 } else { 37 }; continue;
}
// C line 3306
41 => {
let _ = libc::_exit((127 as i32));
vm_block = 40; continue;
}
// C line 3305
42 => {
vm_block = if ((((libc::chdir(cwd)) < ((0 as i32))) as i32)) != 0 { 41 } else { 40 }; continue;
}
// C line 3304
43 => {
vm_block = if !(cwd).is_null() { 42 } else { 40 }; continue;
}
// C line 3289
44 => {
let _ = os_closefrom((3 as i32));
vm_block = 43; continue;
}
// C line 3276
45 => {
vm_block = if ((((i) < ((((3 as i32)) as u32))) as i32)) != 0 { 49 } else { 44 }; continue;
}
// C line ?
46 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 45; continue;
}
// C line 3279
47 => {
let _ = libc::_exit((127 as i32));
vm_block = 46; continue;
}
// C line 3278
48 => {
vm_block = if ((((libc::dup2(*((std_fds).as_mut_ptr()).offset((i) as isize), ((i) as i32))) < ((0 as i32))) as i32)) != 0 { 47 } else { 46 }; continue;
}
// C line 3277
49 => {
vm_block = if ((((((*((std_fds).as_mut_ptr()).offset((i) as isize)) as u32)) != (i)) as i32)) != 0 { 48 } else { 46 }; continue;
}
// C line 3276
50 => {
let _ = { let assigned = (((0 as i32)) as u32); i = assigned; assigned };
vm_block = 45; continue;
}
// C line 3272
51 => {
vm_block = if ((((pid) == ((0 as i32))) as i32)) != 0 { 50 } else { 28 }; continue;
}
// C line 3270
52 => {
vm_block = 2; continue;
}
// C line 3269
53 => {
let _ = JS_ThrowTypeError(ctx, c"fork error".as_ptr());
vm_block = 52; continue;
}
// C line 3268
54 => {
vm_block = if ((((pid) < ((0 as i32))) as i32)) != 0 { 53 } else { 51 }; continue;
}
// C line 3267
55 => {
let _ = { let assigned = libc::fork(); pid = assigned; assigned };
vm_block = 54; continue;
}
// C line 3263
56 => {
vm_block = 2; continue;
}
// C line 3262
57 => {
vm_block = if (ret) != 0 { 56 } else { 55 }; continue;
}
// C line 3261
58 => {
let _ = JS_FreeValue(ctx, val);
vm_block = 57; continue;
}
// C line 3260
59 => {
let _ = { let assigned = JS_ToUint32(ctx, core::ptr::addr_of_mut!(gid), val); ret = assigned; assigned };
vm_block = 58; continue;
}
// C line 3259
60 => {
vm_block = if ((!((JS_IsUndefined(val)) != 0) as i32)) != 0 { 59 } else { 55 }; continue;
}
// C line 3258
61 => {
vm_block = 2; continue;
}
// C line 3257
62 => {
vm_block = if (JS_IsException(val)) != 0 { 61 } else { 60 }; continue;
}
// C line 3256
63 => {
let _ = { let assigned = JS_GetPropertyStr(ctx, options, c"gid".as_ptr()); val = assigned; assigned };
vm_block = 62; continue;
}
// C line 3253
64 => {
vm_block = 2; continue;
}
// C line 3252
65 => {
vm_block = if (ret) != 0 { 64 } else { 63 }; continue;
}
// C line 3251
66 => {
let _ = JS_FreeValue(ctx, val);
vm_block = 65; continue;
}
// C line 3250
67 => {
let _ = { let assigned = JS_ToUint32(ctx, core::ptr::addr_of_mut!(uid), val); ret = assigned; assigned };
vm_block = 66; continue;
}
// C line 3249
68 => {
vm_block = if ((!((JS_IsUndefined(val)) != 0) as i32)) != 0 { 67 } else { 63 }; continue;
}
// C line 3248
69 => {
vm_block = 2; continue;
}
// C line 3247
70 => {
vm_block = if (JS_IsException(val)) != 0 { 69 } else { 68 }; continue;
}
// C line 3246
71 => {
let _ = { let assigned = JS_GetPropertyStr(ctx, options, c"uid".as_ptr()); val = assigned; assigned };
vm_block = 70; continue;
}
// C line 3243
72 => {
vm_block = 2; continue;
}
// C line 3242
73 => {
vm_block = if ((!(!(envp).is_null()) as i32)) != 0 { 72 } else { 71 }; continue;
}
// C line 3241
74 => {
let _ = JS_FreeValue(ctx, val);
vm_block = 73; continue;
}
// C line 3240
75 => {
let _ = { let assigned = build_envp(ctx, val); envp = assigned; assigned };
vm_block = 74; continue;
}
// C line 3239
76 => {
vm_block = if ((!((JS_IsUndefined(val)) != 0) as i32)) != 0 { 75 } else { 71 }; continue;
}
// C line 3238
77 => {
vm_block = 2; continue;
}
// C line 3237
78 => {
vm_block = if (JS_IsException(val)) != 0 { 77 } else { 76 }; continue;
}
// C line 3236
79 => {
let _ = { let assigned = JS_GetPropertyStr(ctx, options, c"env".as_ptr()); val = assigned; assigned };
vm_block = 78; continue;
}
// C line 3222
80 => {
vm_block = if ((((i) < ((((3 as i32)) as u32))) as i32)) != 0 { 90 } else { 79 }; continue;
}
// C line ?
81 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 80; continue;
}
// C line 3232
82 => {
let _ = { let assigned = fd; *((std_fds).as_mut_ptr()).offset((i) as isize) = assigned; assigned };
vm_block = 81; continue;
}
// C line 3231
83 => {
vm_block = 2; continue;
}
// C line 3230
84 => {
vm_block = if (ret) != 0 { 83 } else { 82 }; continue;
}
// C line 3229
85 => {
let _ = JS_FreeValue(ctx, val);
vm_block = 84; continue;
}
// C line 3228
86 => {
let _ = { let assigned = JS_ToInt32(ctx, core::ptr::addr_of_mut!(fd), val); ret = assigned; assigned };
vm_block = 85; continue;
}
// C line 3226
87 => {
vm_block = if ((!((JS_IsUndefined(val)) != 0) as i32)) != 0 { 86 } else { 81 }; continue;
}
// C line 3225
88 => {
vm_block = 2; continue;
}
// C line 3224
89 => {
vm_block = if (JS_IsException(val)) != 0 { 88 } else { 87 }; continue;
}
// C line 3223
90 => {
let _ = { let assigned = JS_GetPropertyStr(ctx, options, *((std_name).as_ptr()).offset((i) as isize)); val = assigned; assigned };
vm_block = 89; continue;
}
// C line 3222
91 => {
let _ = { let assigned = (((0 as i32)) as u32); i = assigned; assigned };
vm_block = 80; continue;
}
// C line 3218
92 => {
vm_block = 2; continue;
}
// C line 3217
93 => {
vm_block = if ((!(!(cwd).is_null()) as i32)) != 0 { 92 } else { 91 }; continue;
}
// C line 3216
94 => {
let _ = JS_FreeValue(ctx, val);
vm_block = 93; continue;
}
// C line 3215
95 => {
let _ = { let assigned = JS_ToCString(ctx, val); cwd = assigned; assigned };
vm_block = 94; continue;
}
// C line 3214
96 => {
vm_block = if ((!((JS_IsUndefined(val)) != 0) as i32)) != 0 { 95 } else { 91 }; continue;
}
// C line 3213
97 => {
vm_block = 2; continue;
}
// C line 3212
98 => {
vm_block = if (JS_IsException(val)) != 0 { 97 } else { 96 }; continue;
}
// C line 3211
99 => {
let _ = { let assigned = JS_GetPropertyStr(ctx, options, c"cwd".as_ptr()); val = assigned; assigned };
vm_block = 98; continue;
}
// C line 3208
100 => {
vm_block = 2; continue;
}
// C line 3207
101 => {
vm_block = if ((!(!(file).is_null()) as i32)) != 0 { 100 } else { 99 }; continue;
}
// C line 3206
102 => {
let _ = JS_FreeValue(ctx, val);
vm_block = 101; continue;
}
// C line 3205
103 => {
let _ = { let assigned = JS_ToCString(ctx, val); file = assigned; assigned };
vm_block = 102; continue;
}
// C line 3204
104 => {
vm_block = if ((!((JS_IsUndefined(val)) != 0) as i32)) != 0 { 103 } else { 99 }; continue;
}
// C line 3203
105 => {
vm_block = 2; continue;
}
// C line 3202
106 => {
vm_block = if (JS_IsException(val)) != 0 { 105 } else { 104 }; continue;
}
// C line 3201
107 => {
let _ = { let assigned = JS_GetPropertyStr(ctx, options, c"file".as_ptr()); val = assigned; assigned };
vm_block = 106; continue;
}
// C line 3199
108 => {
vm_block = 2; continue;
}
// C line 3198
109 => {
vm_block = if (stdio_bool_option(ctx, core::ptr::addr_of_mut!(use_path), options, c"usePath".as_ptr())) != 0 { 108 } else { 107 }; continue;
}
// C line 3197
110 => {
vm_block = 2; continue;
}
// C line 3196
111 => {
vm_block = if (stdio_bool_option(ctx, core::ptr::addr_of_mut!(block_flag), options, c"block".as_ptr())) != 0 { 110 } else { 109 }; continue;
}
// C line 3194
112 => {
let _ = { let assigned = *(argv).offset(((1 as i32)) as isize); options = assigned; assigned };
vm_block = 111; continue;
}
// C line 3193
113 => {
vm_block = if ((((argc) >= ((2 as i32))) as i32)) != 0 { 112 } else { 55 }; continue;
}
// C line 3189
114 => {
vm_block = if ((((i) < ((((3 as i32)) as u32))) as i32)) != 0 { 116 } else { 113 }; continue;
}
// C line ?
115 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 114; continue;
}
// C line 3190
116 => {
let _ = { let assigned = ((i) as i32); *((std_fds).as_mut_ptr()).offset((i) as isize) = assigned; assigned };
vm_block = 115; continue;
}
// C line 3189
117 => {
let _ = { let assigned = (((0 as i32)) as u32); i = assigned; assigned };
vm_block = 114; continue;
}
// C line 3187
118 => {
let _ = { let assigned = core::ptr::null_mut::<c_char>(); *(exec_argv).offset((exec_argc) as isize) = assigned; assigned };
vm_block = 117; continue;
}
// C line 3177
119 => {
vm_block = if ((((i) < (exec_argc)) as i32)) != 0 { 128 } else { 118 }; continue;
}
// C line ?
120 => {
let _ = { let old = i; i = (i).wrapping_add(1); old };
vm_block = 119; continue;
}
// C line 3185
121 => {
let _ = { let assigned = str; *(exec_argv).offset((i) as isize) = assigned; assigned };
vm_block = 120; continue;
}
// C line 3184
122 => {
vm_block = 2; continue;
}
// C line 3183
123 => {
vm_block = if ((!(!(str).is_null()) as i32)) != 0 { 122 } else { 121 }; continue;
}
// C line 3182
124 => {
let _ = JS_FreeValue(ctx, val);
vm_block = 123; continue;
}
// C line 3181
125 => {
let _ = { let assigned = JS_ToCString(ctx, val); str = assigned; assigned };
vm_block = 124; continue;
}
// C line 3180
126 => {
vm_block = 2; continue;
}
// C line 3179
127 => {
vm_block = if (JS_IsException(val)) != 0 { 126 } else { 125 }; continue;
}
// C line 3178
128 => {
let _ = { let assigned = JS_GetPropertyUint32(ctx, args, i); val = assigned; assigned };
vm_block = 127; continue;
}
// C line 3177
129 => {
let _ = { let assigned = (((0 as i32)) as u32); i = assigned; assigned };
vm_block = 119; continue;
}
// C line 3176
130 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 3175
131 => {
vm_block = if ((!(!(exec_argv).is_null()) as i32)) != 0 { 130 } else { 129 }; continue;
}
// C line 3174
132 => {
let _ = { let assigned = ((js_mallocz(ctx, ((core::mem::size_of::<*const c_char>() as usize)).wrapping_mul((((exec_argc).wrapping_add((((1 as i32)) as u32))) as usize)))) as *mut *const c_char); exec_argv = assigned; assigned };
vm_block = 131; continue;
}
// C line 3172
133 => {
return JS_ThrowTypeError(ctx, c"invalid number of arguments".as_ptr());
}
// C line 3171
134 => {
vm_block = if ((((((((exec_argc) < ((((1 as i32)) as u32))) as i32)) != 0) || (((((exec_argc) > ((((65535 as i32)) as u32))) as i32)) != 0)) as i32)) != 0 { 133 } else { 132 }; continue;
}
// C line 3169
135 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 3168
136 => {
vm_block = if (ret) != 0 { 135 } else { 134 }; continue;
}
// C line 3167
137 => {
let _ = JS_FreeValue(ctx, val);
vm_block = 136; continue;
}
// C line 3166
138 => {
let _ = { let assigned = JS_ToUint32(ctx, core::ptr::addr_of_mut!(exec_argc), val); ret = assigned; assigned };
vm_block = 137; continue;
}
// C line 3165
139 => {
return JSValue { u: JSValueUnion { uint64: (((((0 as i32)) as u32)) as u64) }, tag: (((JS_TAG_EXCEPTION as i32)) as i64) };
}
// C line 3164
140 => {
vm_block = if (JS_IsException(val)) != 0 { 139 } else { 138 }; continue;
}
// C line 3163
141 => {
let _ = { let assigned = JS_GetPropertyStr(ctx, args, c"length".as_ptr()); val = assigned; assigned };
vm_block = 140; continue;
}
// C line 3161
142 => {
uid = ((((1 as i32)).wrapping_neg()) as u32);
gid = ((((1 as i32)).wrapping_neg()) as u32);
vm_block = 141; continue;
}
// C line 3159
143 => {
std_name = [c"stdin".as_ptr(), c"stdout".as_ptr(), c"stderr".as_ptr()];
vm_block = 142; continue;
}
// C line 3158
144 => {
block_flag = quickjs::cutils_header::TRUE;
use_path = quickjs::cutils_header::TRUE;
vm_block = 143; continue;
}
// C line 3155
145 => {
envp = *(os_environ_pointer());
vm_block = 144; continue;
}
// C line 3154
146 => {
file = core::ptr::null_mut::<c_char>();
cwd = core::ptr::null_mut::<c_char>();
vm_block = 145; continue;
}
// C line 3152
147 => {
args = *(argv).offset(((0 as i32)) as isize);
vm_block = 146; continue;
}
_ => std::process::abort(),
} }
}
