//! Layout checks of the production Windows host modules, through the crate API.
//! Compile-only; no native Windows OS service is mocked or executed.
#![cfg(windows)]
#![allow(
    non_snake_case,
    non_camel_case_types,
    non_upper_case_globals,
    unused_imports,
    dead_code,
    unused_mut,
    unused_parens,
    unused_assignments
)]
use quickjs_tools::{quickjs_libc_windows, quickjs_libc_windows_state};
const _: () =
    assert!(core::mem::size_of::<quickjs_libc_windows::CONSOLE_SCREEN_BUFFER_INFO>() == 22);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::size_of::<quickjs_libc_windows::WinStat>() == 48);

#[cfg(target_pointer_width = "64")]
const _: () = {
    use quickjs_libc_windows_state::*;
    assert!(core::mem::size_of::<JSThreadState>() == 104);
    assert!(core::mem::size_of::<JSWorkerMessagePipe>() == 40);
    assert!(core::mem::offset_of!(JSWorkerMessagePipe, mutex) == 8);
    assert!(core::mem::offset_of!(JSWorkerMessagePipe, waker) == 32);
    assert!(core::mem::size_of::<quickjs_libc_windows::crt::pthread_attr_t>() == 32);
};
