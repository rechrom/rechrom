// Copyright 2022 Google LLC. BSD-style license.
use core::ffi::{c_int, c_void};

// Source: include/private/SkAssert.h:52-52
// This is the actual platform producer declaration, not an invented abort body.
// Source: include/private/SkAssert.h:52
// The actual source port supplies this function; no invented C++ linker ABI.
pub use crate::src::ports::SkMemory_malloc::sk_abort_no_print;
// Source: skia/config/SkUserConfig.h:72-77
// Chromium supplies the actual user override before SkAssert's !defined guard.
unsafe extern "C" {
    pub fn SkAbort_FileLine(file: *const u8, line: c_int, format: *const u8, ...) -> !;
}

// Source: include/private/SkAssert.h:54-61
#[cfg(sk_build_for_google3)]
unsafe extern "C" {
    fn DumpStackTrace(
        skip_count: c_int,
        callback: unsafe extern "C" fn(*const u8, *mut c_void),
        argument: *mut c_void,
    );
}
#[cfg(sk_build_for_google3)]
#[doc(hidden)]
pub unsafe fn dump_google3_stack() {
    unsafe {
        DumpStackTrace(
            0,
            crate::src::core::SkDebug::SkDebugfForDumpStackTrace,
            core::ptr::null_mut(),
        );
    }
}
#[cfg(sk_build_for_google3)]
#[macro_export]
macro_rules! SK_DUMP_GOOGLE3_STACK {
    () => {
        unsafe { $crate::include::private::SkAssert::dump_google3_stack() }
    };
}
#[cfg(not(sk_build_for_google3))]
#[macro_export]
macro_rules! SK_DUMP_GOOGLE3_STACK {
    () => {{}};
}

// Source: include/private/SkAssert.h:64-78
// Source: skia/config/SkUserConfig.h:72-77
// Default selection is this repository's real Chromium override. Recognition
// of skia_use_default_abort does not enable or silently bypass the override.
#[cfg(not(skia_use_default_abort))]
#[macro_export]
macro_rules! SK_ABORT {
    (@source $file:expr,$line:expr; $format:expr $(,$argument:expr)* $(,)?)=>{{
        #[cfg(not(skia_use_default_abort))]
        unsafe{$crate::include::private::SkAssert::SkAbort_FileLine($file,$line as core::ffi::c_int,concat!($format,"\0").as_ptr() $(,$argument)*);}
        #[cfg(skia_use_default_abort)] {
            $crate::SKIA_LOG_E!(@source $file,$line; concat!($crate::__sk_dump_line_format!(),r#": fatal error: ""#,$format,r#"""#),$file,$line as core::ffi::c_int $(,$argument)*);
            $crate::SK_DUMP_GOOGLE3_STACK!();
            unsafe{$crate::include::private::SkAssert::sk_abort_no_print();}
        }
    }};

    ($format:expr $(,$argument:expr)* $(,)?)=>{compile_error!("SK_ABORT requires @source native_file_ptr,native_line; format,args")};
}
#[cfg(all(skia_use_default_abort, target_os = "windows"))]
#[doc(hidden)]
#[macro_export]
macro_rules! __sk_dump_line_format {
    () => {
        "%s(%d)"
    };
}
#[cfg(all(skia_use_default_abort, not(target_os = "windows")))]
#[doc(hidden)]
#[macro_export]
macro_rules! __sk_dump_line_format {
    () => {
        "%s:%d"
    };
}
#[cfg(skia_use_default_abort)]
#[macro_export]
macro_rules! SK_ABORT {
    (@source $file:expr,$line:expr; $format:expr $(,$argument:expr)* $(,)?)=>{{
        #[cfg(not(skia_use_default_abort))]
        unsafe{$crate::include::private::SkAssert::SkAbort_FileLine($file,$line as core::ffi::c_int,concat!($format,"\0").as_ptr() $(,$argument)*);}
        #[cfg(skia_use_default_abort)] {
            $crate::SKIA_LOG_E!(@source $file,$line; concat!($crate::__sk_dump_line_format!(),r#": fatal error: ""#,$format,r#"""#),$file,$line as core::ffi::c_int $(,$argument)*);
            $crate::SK_DUMP_GOOGLE3_STACK!();
            unsafe{$crate::include::private::SkAssert::sk_abort_no_print();}
        }
    }};

    ($message:expr $(,$argument:expr)* $(,)?)=>{compile_error!("SK_ABORT requires @source native_file_ptr,native_line; format,args")};
}

// Source: include/private/SkAssert.h:91-111
// Rust conditions are already bool. Failure formatting is evaluated only after
// the single condition evaluation has failed, just as the native lambdas do.
#[macro_export]
macro_rules! SkASSERT_RELEASE {
(@source $file:expr,$line:expr; @condition $native_condition:expr; $condition:expr $(,)?)=>{{
    if !$condition {$crate::SK_ABORT!(@source $file,$line; "check(%s)",$native_condition);}
}};
(@source $file:expr,$line:expr; $condition:expr $(,)?)=>{compile_error!("native assertion requires @condition original_cpp_expression_ptr; rust_bool")};($condition:expr $(,)?)=>{compile_error!("native assertion requires @source file,line; @condition original_cpp_expression_ptr; rust_bool")};}
#[macro_export]
macro_rules! SkASSERTF_RELEASE {
(@source $file:expr,$line:expr; @condition $native_condition:expr; $condition:expr,$format:expr $(,$argument:expr)* $(,)?)=>{{
    if !$condition {$crate::SK_ABORT!(@source $file,$line; concat!("assertf(%s): ",$format),$native_condition $(,$argument)*);}
}};
(@source $file:expr,$line:expr; $condition:expr,$format:expr $(,$argument:expr)* $(,)?)=>{compile_error!("native formatted assertion requires @condition original_cpp_expression_ptr; rust_bool,format,args")};($condition:expr,$format:expr $(,$argument:expr)* $(,)?)=>{compile_error!("native formatted assertion requires @source file,line; @condition original_cpp_expression_ptr; rust_bool,format,args")};}

// Source: include/private/SkAssert.h:113-129
#[cfg(sk_debug)]
#[macro_export]
macro_rules! SkASSERT {($($args:tt)*)=>{$crate::SkASSERT_RELEASE!($($args)*)};}
#[cfg(not(sk_debug))]
#[macro_export]
macro_rules! SkASSERT {
    ($($args:tt)*) => {{}};
}
#[cfg(sk_debug)]
#[macro_export]
macro_rules! SkASSERTF {($($args:tt)*)=>{$crate::SkASSERTF_RELEASE!($($args)*)};}
#[cfg(not(sk_debug))]
#[macro_export]
macro_rules! SkASSERTF {
    ($($args:tt)*) => {{}};
}
#[cfg(sk_debug)]
#[macro_export]
macro_rules! SkDEBUGFAIL {(@source $file:expr,$line:expr; $message:expr $(,)?)=>{$crate::SK_ABORT!(@source $file,$line; "%s",$message)};($message:expr $(,)?)=>{$crate::SK_ABORT!("%s",$message)};}
#[cfg(not(sk_debug))]
#[macro_export]
macro_rules! SkDEBUGFAIL {
    ($($args:tt)*) => {{}};
}
#[cfg(sk_debug)]
#[macro_export]
macro_rules! SkDEBUGFAILF {($($args:tt)*)=>{$crate::SK_ABORT!($($args)*)};}
#[cfg(not(sk_debug))]
#[macro_export]
macro_rules! SkDEBUGFAILF {
    ($($args:tt)*) => {{}};
}
#[cfg(sk_debug)]
#[macro_export]
macro_rules! SkAssertResult {(@source $file:expr,$line:expr; @condition $native_condition:expr; $condition:expr $(,)?)=>{$crate::SkASSERT!(@source $file,$line; @condition $native_condition; $condition)};(@source $file:expr,$line:expr; $condition:expr $(,)?)=>{$crate::SkASSERT!(@source $file,$line; $condition)};($condition:expr $(,)?)=>{$crate::SkASSERT!($condition)};}
#[cfg(not(sk_debug))]
#[macro_export]
macro_rules! SkAssertResult {
    (@source $file:expr,$line:expr; @condition $native_condition:expr; $condition:expr $(,)?) => {{
        if $condition {}
    }};
    (@source $file:expr,$line:expr; $condition:expr $(,)?) => {{
        if $condition {}
    }};
    ($condition:expr $(,)?) => {{
        if $condition {}
    }};
}

// Source: include/private/SkAssert.h:131-138
// Chromium uses Clang, including Windows: original __builtin_trap lowers to
// the actual target trap intrinsic, not libc abort or unreachable_unchecked.
#[cfg(not(sk_native_nonclang_msvc))]
#[macro_export]
macro_rules! SkUNREACHABLE {
    (@source $file:expr,$line:expr;) => {{
        core::intrinsics::abort()
    }};
    () => {{
        core::intrinsics::abort()
    }};
}

// Source: include/private/SkAssert.h:140-142
pub fn sk_print_index_out_of_bounds(i: usize, size: usize) -> ! {
    crate::SK_ABORT!(@source b"include/private/SkAssert.h\0".as_ptr(),141; "Index (%zu) out of bounds for size %zu.",i,size);
}
// Rust integer-template constraint and exact native static_cast<size_t>.
pub trait CollectionIndex: Copy + PartialOrd {
    fn zero() -> Self;
    fn as_size(self) -> usize;
}
macro_rules! collection_indices {($($ty:ty),*)=>{$(impl CollectionIndex for $ty {
    fn zero()->Self {0}fn as_size(self)->usize {self as usize}
})*};}
collection_indices!(u8, i8, u16, i16, u32, i32, u64, i64, u128, i128, usize, isize);
// Source: include/private/SkAssert.h:144-156
pub fn sk_collection_check_bounds<T: CollectionIndex>(i: T, size: T) -> T {
    if T::zero() <= i && i < size {
        return i;
    }
    #[cfg(sk_debug)]
    sk_print_index_out_of_bounds(i.as_size(), size.as_size());
    #[cfg(not(sk_debug))]
    crate::SkUNREACHABLE!();
}
// Source: include/private/SkAssert.h:158-160
pub fn sk_print_length_too_big(i: usize, size: usize) -> ! {
    crate::SK_ABORT!(@source b"include/private/SkAssert.h\0".as_ptr(),159; "Length (%zu) is too big for size %zu.",i,size);
}
// Source: include/private/SkAssert.h:162-174
pub fn sk_collection_check_length<T: CollectionIndex>(i: T, size: T) -> T {
    if T::zero() <= i && i <= size {
        return i;
    }
    #[cfg(sk_debug)]
    sk_print_length_too_big(i.as_size(), size.as_size());
    #[cfg(not(sk_debug))]
    crate::SkUNREACHABLE!();
}
// Source: include/private/SkAssert.h:176-184
pub fn sk_collection_not_empty(empty: bool) {
    if empty {
        #[cfg(sk_debug)]
        crate::SK_ABORT!(@source b"include/private/SkAssert.h\0".as_ptr(),179; "Collection is empty.");
        #[cfg(not(sk_debug))]
        crate::SkUNREACHABLE!();
    }
}
// Source: include/private/SkAssert.h:186-188
pub fn sk_print_size_too_big(size: usize, max_size: usize) -> ! {
    crate::SK_ABORT!(@source b"include/private/SkAssert.h\0".as_ptr(),187; "Size (%zu) can't be represented in bytes. Max size is %zu.",size,max_size);
}
// Source: include/private/SkAssert.h:190-201
#[inline(always)]
pub fn check_size_bytes_too_big<T>(size: usize) -> usize {
    let max_size = usize::MAX / core::mem::size_of::<T>();
    if size > max_size {
        #[cfg(sk_debug)]
        sk_print_size_too_big(size, max_size);
        #[cfg(not(sk_debug))]
        crate::SkUNREACHABLE!();
    }
    size
}

// Source: include/private/SkAssert.h:39-40
// Only the native older-GNU alternative evaluates its condition. Do not use
// this branch for Clang/MSVC's non-evaluated assumption expressions.
#[cfg(sk_native_gnu_before_13)]
#[macro_export]
macro_rules! SK_ASSUME {
    ($condition:expr) => {{
        if !$condition {
            unsafe {
                core::hint::unreachable_unchecked();
            }
        }
    }};
}
// Source: include/private/SkAssert.h:44-45
#[cfg(sk_native_no_assume)]
#[macro_export]
macro_rules! SK_ASSUME {
    ($($condition:tt)*) => {{}};
}

// Source: include/private/SkAssert.h:130-130
#[cfg(sk_native_nonclang_msvc)]
pub const FAST_FAIL_INVALID_ARG: u32 = 5;

// Source: include/private/SkAssert.h:133-134
// Actual __fastfail target instructions/register constraints, not __builtin_trap.
// Primary producer: clang/lib/CodeGen/CGBuiltin.cpp EmitMSVCBuiltinExpr(__fastfail),
// also documented at https://learn.microsoft.com/en-us/cpp/intrinsics/fastfail .
// All sequences are noreturn and side-effecting. Do not use nomem/pure: native
// compiler inline assembly also has its ordinary compiler memory barrier.
#[cfg(target_os = "windows")]
#[doc(hidden)]
pub unsafe fn native_fastfail(code: u32) -> ! {
    #[cfg(target_arch = "x86")]
    unsafe {
        core::arch::asm!("int 0x29",in("ecx") code,options(noreturn,nostack));
    }
    #[cfg(target_arch = "x86_64")]
    unsafe {
        core::arch::asm!("int 0x29",in("rcx") code as u64,options(noreturn,nostack));
    }
    // Windows ARM is Thumb; Clang emits udf #251 with argument in r0.
    #[cfg(target_arch = "arm")]
    unsafe {
        core::arch::asm!("udf #251",in("r0") code,options(noreturn,nostack));
    }
    #[cfg(target_arch = "aarch64")]
    unsafe {
        core::arch::asm!("brk #0xF003",in("w0") code,options(noreturn,nostack));
    }
    #[cfg(not(any(
        target_arch = "x86",
        target_arch = "x86_64",
        target_arch = "arm",
        target_arch = "aarch64"
    )))]
    compile_error!(
        "native __fastfail has no supported target instruction for this Windows architecture"
    );
}
// Source: include/private/SkAssert.h:133-133
#[cfg(sk_native_nonclang_msvc)]
pub fn sk_fast_fail() -> ! {
    unsafe { native_fastfail(FAST_FAIL_INVALID_ARG) }
}
// Source: include/private/SkAssert.h:134-134
#[cfg(sk_native_nonclang_msvc)]
#[macro_export]
macro_rules! SkUNREACHABLE {
    (@source $file:expr,$line:expr;) => {{
        $crate::include::private::SkAssert::sk_fast_fail()
    }};
    () => {{
        $crate::include::private::SkAssert::sk_fast_fail()
    }};
}
