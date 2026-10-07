//! Host utilities translated from official QuickJS.
//! Files, processes and OS services belong here, outside the engine crate.
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]
#[cfg(all(
    feature = "host-have-closefrom",
    not(any(
        all(target_os = "linux", target_env = "gnu"),
        target_os = "freebsd",
        target_os = "netbsd",
        target_os = "openbsd",
        target_os = "dragonfly"
    ))
))]
compile_error!("host-have-closefrom requires a host libc providing the original closefrom API");

pub mod qjs;
pub mod qjsc;
pub mod quickjs_libc;
pub mod quickjs_libc_header;
#[cfg(unix)]
pub mod quickjs_libc_state;
#[cfg(windows)]
pub mod quickjs_libc_windows;
#[cfg(windows)]
pub mod quickjs_libc_windows_state;
#[cfg(windows)]
pub mod quickjs_libc_windows_worker;
#[cfg(unix)]
pub mod quickjs_libc_worker;
pub mod run_test262;
pub mod unicode_gen;
pub mod unicode_gen_def;
