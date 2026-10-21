// Copyright 2019 The Chromium Authors. BSD-style license.
// Source: base/allocator/partition_allocator/src/partition_alloc/partition_alloc_base/immediate_crash.h:43-164
// The root bridges the actual native compiler/ISA selection, not host/debug.
// Native Apple's x86 empty sequence2 plus Clang's documented unreachable ud2
// is represented by its same emitted terminal sequence explicitly.
macro_rules! PA_IMMEDIATE_CRASH {()=>{{
 #[cfg(any(pa_immediate_crash="gcc_x86_apple",pa_immediate_crash="gcc_x86",pa_immediate_crash="msvc_clang_x86"))]
 unsafe{core::arch::asm!("int3","ud2",options(noreturn));}
 #[cfg(pa_immediate_crash="gcc_armel")]
 unsafe{core::arch::asm!("bkpt #0","udf #0",options(noreturn));}
 #[cfg(pa_immediate_crash="gcc_arm64")]
 unsafe{core::arch::asm!("brk #0","hlt #0",options(noreturn));}
 #[cfg(not(any(pa_immediate_crash="gcc_x86_apple",pa_immediate_crash="gcc_x86",pa_immediate_crash="msvc_clang_x86",pa_immediate_crash="gcc_armel",pa_immediate_crash="gcc_arm64")))]
 compile_error!("PA_IMMEDIATE_CRASH selected native compiler trap lowering is not implemented for this domain");
}};}
pub(crate) use PA_IMMEDIATE_CRASH;
