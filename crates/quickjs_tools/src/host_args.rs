// Native command-line byte strings for the original C tools. These host
// bindings stay outside the engine; Windows uses CRT narrow argv and ACP.
use std::{ffi::OsString, io};

#[cfg(unix)]
pub(crate) const CLOCKS_PER_SEC: i64 = 1_000_000;
#[cfg(windows)]
pub(crate) const CLOCKS_PER_SEC: i64 = 1000;

#[cfg(unix)]
pub(crate) fn args_bytes() -> io::Result<Vec<Vec<u8>>> {
    use std::os::unix::ffi::OsStringExt;
    Ok(std::env::args_os().map(OsString::into_vec).collect())
}

#[cfg(windows)]
pub(crate) fn args_bytes() -> io::Result<Vec<Vec<u8>>> {
    use std::{ffi::{c_char, CStr}, ptr};
    #[repr(C)]
    struct StartupInfo { newmode: i32 }
    extern "C" {
        fn __getmainargs(argc: *mut i32, argv: *mut *mut *mut c_char,
                         env: *mut *mut *mut c_char, wildcard: i32,
                         startup: *mut StartupInfo) -> i32;
    }
    unsafe {
        let mut argc = 0;
        let mut argv = ptr::null_mut();
        let mut env = ptr::null_mut();
        let mut startup = StartupInfo { newmode: 0 };
        if __getmainargs(&mut argc, &mut argv, &mut env, 0, &mut startup) < 0 {
            return Err(io::Error::last_os_error());
        }
        Ok((0..argc).map(|i| CStr::from_ptr(*argv.add(i as usize)).to_bytes().to_vec()).collect())
    }
}

#[cfg(windows)]
#[link(name = "kernel32")]
extern "system" {
    fn MultiByteToWideChar(codepage: u32, flags: u32, bytes: *const u8, count: i32,
                           wide: *mut u16, capacity: i32) -> i32;
    fn WideCharToMultiByte(codepage: u32, flags: u32, wide: *const u16, count: i32,
                           bytes: *mut u8, capacity: i32,
                           default_char: *const u8, used_default: *mut i32) -> i32;
}

#[cfg(windows)]
pub(crate) fn byte_os(bytes: &[u8]) -> io::Result<OsString> {
    use std::{os::windows::ffi::OsStringExt, ptr};
    if bytes.is_empty() { return Ok(OsString::new()); }
    let count = i32::try_from(bytes.len()).map_err(|_| io::Error::from(io::ErrorKind::InvalidInput))?;
    unsafe {
        let n = MultiByteToWideChar(0, 0, bytes.as_ptr(), count, ptr::null_mut(), 0);
        if n == 0 { return Err(io::Error::last_os_error()); }
        let mut wide = vec![0u16; n as usize];
        if MultiByteToWideChar(0, 0, bytes.as_ptr(), count, wide.as_mut_ptr(), n) == 0 {
            return Err(io::Error::last_os_error());
        }
        Ok(OsString::from_wide(&wide))
    }
}

#[cfg(windows)]
pub(crate) fn os_bytes(value: OsString) -> io::Result<Vec<u8>> {
    use std::{os::windows::ffi::OsStrExt, ptr};
    let wide = value.encode_wide().collect::<Vec<_>>();
    if wide.is_empty() { return Ok(Vec::new()); }
    let count = i32::try_from(wide.len()).map_err(|_| io::Error::from(io::ErrorKind::InvalidInput))?;
    unsafe {
        let n = WideCharToMultiByte(0, 0, wide.as_ptr(), count, ptr::null_mut(), 0, ptr::null(), ptr::null_mut());
        if n == 0 { return Err(io::Error::last_os_error()); }
        let mut bytes = vec![0; n as usize];
        if WideCharToMultiByte(0, 0, wide.as_ptr(), count, bytes.as_mut_ptr(), n, ptr::null(), ptr::null_mut()) == 0 {
            return Err(io::Error::last_os_error());
        }
        Ok(bytes)
    }
}
