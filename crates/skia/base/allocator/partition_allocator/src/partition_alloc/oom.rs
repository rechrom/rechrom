// Copyright 2021 The Chromium Authors. BSD-style license.
use super::partition_alloc_base::debug::alias::Alias;
use super::partition_alloc_base::immediate_crash::PA_IMMEDIATE_CRASH;
// Source: base/allocator/partition_allocator/src/partition_alloc/oom.h:32-32
// Source: base/allocator/partition_allocator/src/partition_alloc/oom.cc:24-24
#[cfg_attr(
    pa_cpp_abi = "itanium",
    unsafe(export_name = "_ZN15partition_alloc10g_oom_sizeE")
)]
pub static mut g_oom_size: usize = 0;
// Source: base/allocator/partition_allocator/src/partition_alloc/oom.h:43-43
#[cfg(pa_is_win)]
pub mod win {
    pub const kOomExceptionCode: u32 = 0xe0000008;
}
// Source: base/allocator/partition_allocator/src/partition_alloc/oom.cc:92-94
// Native PA_NOT_TAIL_CALLED remains a precise language-operation dependency;
// plain calls below are not claimed to guarantee retained parent stack frames.
/// Safety: the native unchecked g_oom_size publication has no concurrent writer;
/// OOM callbacks/OS handlers satisfy their original no-unwind and lifetime leases.
#[cfg_attr(
    all(pa_cpp_abi = "itanium", pa_size_t_abi = "ulong"),
    unsafe(export_name = "_ZN15partition_alloc27TerminateBecauseOutOfMemoryEm")
)]
#[cfg_attr(
    all(pa_cpp_abi = "itanium", pa_size_t_abi = "uint"),
    unsafe(export_name = "_ZN15partition_alloc27TerminateBecauseOutOfMemoryEj")
)]
#[cfg_attr(
    all(pa_cpp_abi = "itanium", pa_size_t_abi = "ullong"),
    unsafe(export_name = "_ZN15partition_alloc27TerminateBecauseOutOfMemoryEy")
)]
pub unsafe extern "C-unwind" fn TerminateBecauseOutOfMemory(size: usize) -> ! {
    unsafe { internal::OnNoMemoryInternal(size) }
}
pub mod internal {
    use super::*;
    // Source: base/allocator/partition_allocator/src/partition_alloc/oom.cc:31-35
    #[inline(never)] // actual PA_NOINLINE, NOT a substitute for PA_NOT_TAIL_CALLED
    #[cfg_attr(
        all(pa_cpp_abi = "itanium", pa_size_t_abi = "ulong"),
        unsafe(export_name = "_ZN15partition_alloc8internal18OnNoMemoryInternalEm")
    )]
    #[cfg_attr(
        all(pa_cpp_abi = "itanium", pa_size_t_abi = "uint"),
        unsafe(export_name = "_ZN15partition_alloc8internal18OnNoMemoryInternalEj")
    )]
    #[cfg_attr(
        all(pa_cpp_abi = "itanium", pa_size_t_abi = "ullong"),
        unsafe(export_name = "_ZN15partition_alloc8internal18OnNoMemoryInternalEy")
    )]
    pub(super) unsafe extern "C-unwind" fn OnNoMemoryInternal(size: usize) -> ! {
        unsafe {
            core::ptr::addr_of_mut!(g_oom_size).write(size);
        }
        let tmp_size = size;
        unsafe {
            Alias(core::ptr::addr_of!(tmp_size).cast());
        }
        // Source: base/allocator/partition_allocator/src/partition_alloc/oom.cc:55-73
        #[cfg(pa_is_win)]
        unsafe {
            let mut exception_args = [size, usize::MAX, usize::MAX];
            let mut memory_status = MEMORYSTATUSEX::zero();
            memory_status.dwLength = core::mem::size_of::<MEMORYSTATUSEX>() as u32;
            if GlobalMemoryStatusEx(&mut memory_status) != 0 {
                exception_args[1] = memory_status.ullTotalPageFile as usize;
                exception_args[2] = memory_status.ullAvailPageFile as usize;
            }
            Alias(core::ptr::addr_of!(memory_status).cast());
            RaiseException(
                super::win::kOomExceptionCode,
                1,
                exception_args.len() as u32,
                exception_args.as_ptr(),
            );
            _exit(super::win::kOomExceptionCode as core::ffi::c_int);
        }
        // Source: base/allocator/partition_allocator/src/partition_alloc/oom.cc:86-86
        #[cfg(not(pa_is_win))]
        {
            PA_IMMEDIATE_CRASH!();
        }
    }
    // Source: base/allocator/partition_allocator/src/partition_alloc/oom.cc:101-105
    #[inline(never)]
    #[cfg_attr(
        all(pa_cpp_abi = "itanium", pa_size_t_abi = "ulong"),
        unsafe(export_name = "_ZN15partition_alloc8internal10OnNoMemoryEm")
    )]
    #[cfg_attr(
        all(pa_cpp_abi = "itanium", pa_size_t_abi = "uint"),
        unsafe(export_name = "_ZN15partition_alloc8internal10OnNoMemoryEj")
    )]
    #[cfg_attr(
        all(pa_cpp_abi = "itanium", pa_size_t_abi = "ullong"),
        unsafe(export_name = "_ZN15partition_alloc8internal10OnNoMemoryEy")
    )]
    pub unsafe extern "C-unwind" fn OnNoMemory(size: usize) -> ! {
        unsafe {
            super::super::oom_callback::internal::RunPartitionAllocOomCallback();
            TerminateBecauseOutOfMemory(size);
        }
        // Source: base/allocator/partition_allocator/src/partition_alloc/oom.cc:104-104
        PA_IMMEDIATE_CRASH!();
    }
    // Source: base/allocator/partition_allocator/src/partition_alloc/oom.cc:107-111
    // Definition is unconditional in the actual .cc; only its header declaration is POSIX-gated.
    #[inline(never)]
    #[cfg_attr(
        pa_cpp_abi = "itanium",
        unsafe(export_name = "_ZN15partition_alloc8internal12OnErrnoNoMemEv")
    )]
    pub unsafe extern "C-unwind" fn OnErrnoNoMem() -> ! {
        unsafe {
            super::super::oom_callback::internal::RunPartitionAllocOomCallback();
            TerminateBecauseOutOfMemory(0);
        }
        // Source: base/allocator/partition_allocator/src/partition_alloc/oom.cc:110-110
        PA_IMMEDIATE_CRASH!();
    }
    // External SDK ABI mapping only, zero Skia source credit.
    #[cfg(pa_is_win)]
    #[repr(C)]
    struct MEMORYSTATUSEX {
        dwLength: u32,
        dwMemoryLoad: u32,
        ullTotalPhys: u64,
        ullAvailPhys: u64,
        ullTotalPageFile: u64,
        ullAvailPageFile: u64,
        ullTotalVirtual: u64,
        ullAvailVirtual: u64,
        ullAvailExtendedVirtual: u64,
    }
    #[cfg(pa_is_win)]
    impl MEMORYSTATUSEX {
        fn zero() -> Self {
            Self {
                dwLength: 0,
                dwMemoryLoad: 0,
                ullTotalPhys: 0,
                ullAvailPhys: 0,
                ullTotalPageFile: 0,
                ullAvailPageFile: 0,
                ullTotalVirtual: 0,
                ullAvailVirtual: 0,
                ullAvailExtendedVirtual: 0,
            }
        }
    }
    #[cfg(pa_is_win)]
    #[link(name = "kernel32")]
    unsafe extern "system-unwind" {
        fn GlobalMemoryStatusEx(status: *mut MEMORYSTATUSEX) -> i32;
        fn RaiseException(code: u32, flags: u32, count: u32, args: *const usize);
    }
    #[cfg(pa_is_win)]
    unsafe extern "C" {
        fn _exit(status: core::ffi::c_int) -> !;
    }
}
// Source: base/allocator/partition_allocator/src/partition_alloc/oom.h:70-75
macro_rules! OOM_CRASH{($size:expr)=>{{
 let _guard=$crate::base::allocator::partition_allocator::src::partition_alloc::allocation_guard::ScopedAllowAllocations::new();
 unsafe{$crate::base::allocator::partition_allocator::src::partition_alloc::oom::internal::OnNoMemory($size)}
}};}
pub(crate) use OOM_CRASH;

// External symbol metadata must use the actual native size_t fundamental type.
#[cfg(all(
    pa_cpp_abi = "itanium",
    not(any(
        pa_size_t_abi = "ulong",
        pa_size_t_abi = "uint",
        pa_size_t_abi = "ullong"
    ))
))]
compile_error!("OOM exported C++ symbol metadata requires actual native size_t ABI input");
