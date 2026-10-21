// UNVERIFIED: validation prohibited by user.
use super::{SkSLGraphiteModules::GraphiteModules, SkSLModule::ModuleType};
use core::{
    ffi::{c_char, CStr},
    sync::atomic::{AtomicPtr, Ordering},
};
// Source: src/sksl/SkSLModuleDataDefault.cpp:14-32
// Each resource is the exact decoded adjacent C++ literal data. These bytes
// receive zero translated-code credit. Native debug/optimize-size choice retained.
#[cfg(any(SK_ENABLE_OPTIMIZE_SIZE, not(debug_assertions)))]
static SKSL_COMPUTE: &[u8] = include_bytes!("generated/sksl_compute.minified.sksl");
#[cfg(all(not(SK_ENABLE_OPTIMIZE_SIZE), debug_assertions))]
static SKSL_COMPUTE: &[u8] = include_bytes!("generated/sksl_compute.unoptimized.sksl");
#[cfg(any(SK_ENABLE_OPTIMIZE_SIZE, not(debug_assertions)))]
static SKSL_FRAG: &[u8] = include_bytes!("generated/sksl_frag.minified.sksl");
#[cfg(all(not(SK_ENABLE_OPTIMIZE_SIZE), debug_assertions))]
static SKSL_FRAG: &[u8] = include_bytes!("generated/sksl_frag.unoptimized.sksl");
#[cfg(any(SK_ENABLE_OPTIMIZE_SIZE, not(debug_assertions)))]
static SKSL_GPU: &[u8] = include_bytes!("generated/sksl_gpu.minified.sksl");
#[cfg(all(not(SK_ENABLE_OPTIMIZE_SIZE), debug_assertions))]
static SKSL_GPU: &[u8] = include_bytes!("generated/sksl_gpu.unoptimized.sksl");
#[cfg(any(SK_ENABLE_OPTIMIZE_SIZE, not(debug_assertions)))]
static SKSL_PUBLIC: &[u8] = include_bytes!("generated/sksl_public.minified.sksl");
#[cfg(all(not(SK_ENABLE_OPTIMIZE_SIZE), debug_assertions))]
static SKSL_PUBLIC: &[u8] = include_bytes!("generated/sksl_public.unoptimized.sksl");
#[cfg(any(SK_ENABLE_OPTIMIZE_SIZE, not(debug_assertions)))]
static SKSL_RT_SHADER: &[u8] = include_bytes!("generated/sksl_rt_shader.minified.sksl");
#[cfg(all(not(SK_ENABLE_OPTIMIZE_SIZE), debug_assertions))]
static SKSL_RT_SHADER: &[u8] = include_bytes!("generated/sksl_rt_shader.unoptimized.sksl");
#[cfg(any(SK_ENABLE_OPTIMIZE_SIZE, not(debug_assertions)))]
static SKSL_SHARED: &[u8] = include_bytes!("generated/sksl_shared.minified.sksl");
#[cfg(all(not(SK_ENABLE_OPTIMIZE_SIZE), debug_assertions))]
static SKSL_SHARED: &[u8] = include_bytes!("generated/sksl_shared.unoptimized.sksl");
#[cfg(any(SK_ENABLE_OPTIMIZE_SIZE, not(debug_assertions)))]
static SKSL_VERT: &[u8] = include_bytes!("generated/sksl_vert.minified.sksl");
#[cfg(all(not(SK_ENABLE_OPTIMIZE_SIZE), debug_assertions))]
static SKSL_VERT: &[u8] = include_bytes!("generated/sksl_vert.unoptimized.sksl");
// Source: src/sksl/SkSLModuleDataDefault.cpp:34-38
// Only pointer publication is atomic; the caller retains the original external
// immutable NUL-terminated storage lease. No owner, copy, or allocation is added.
static EMPTY: &[u8] = b"\0";
static sdata_sksl_graphite_frag: AtomicPtr<c_char> = AtomicPtr::new(EMPTY.as_ptr() as *mut c_char);
static sdata_sksl_graphite_vert: AtomicPtr<c_char> = AtomicPtr::new(EMPTY.as_ptr() as *mut c_char);
// Source: src/sksl/SkSLModuleDataDefault.cpp:42-68
pub fn GetModuleData(kind: ModuleType, _filename: &str) -> Vec<u8> {
    let bytes = match kind {
        ModuleType::sksl_shared => SKSL_SHARED,
        ModuleType::sksl_compute => SKSL_COMPUTE,
        ModuleType::sksl_frag => SKSL_FRAG,
        ModuleType::sksl_gpu => SKSL_GPU,
        ModuleType::sksl_public => SKSL_PUBLIC,
        ModuleType::sksl_rt_shader => SKSL_RT_SHADER,
        ModuleType::sksl_vert => SKSL_VERT,
        ModuleType::sksl_graphite_frag | ModuleType::sksl_graphite_vert => {
            let p = if kind == ModuleType::sksl_graphite_frag {
                sdata_sksl_graphite_frag.load(Ordering::Acquire)
            } else {
                sdata_sksl_graphite_vert.load(Ordering::Acquire)
            };
            return if p.is_null() {
                Vec::new()
            } else {
                unsafe { CStr::from_ptr(p).to_bytes().to_vec() }
            };
        }
        _ => crate::SkUNREACHABLE!(@source b"src/sksl/SkSLModuleDataDefault.cpp\0".as_ptr(),64;),
    };
    // std::string(const char*) ends at the first NUL, excluding its terminator.
    bytes[..bytes.iter().position(|b| *b == 0).unwrap_or(bytes.len())].to_vec()
}
// Source: src/sksl/SkSLModuleDataDefault.cpp:71-77
/// Safety: both pointers describe externally owned, immutable, NUL-terminated
/// strings. Their storage remains readable on every consuming thread for the
/// entire period in which the pointers can be observed (normally process life).
/// Initialization is performed once, with no concurrent setter invocation, as
/// required by the native loader. Pointer atomics do not extend storage leases.
pub unsafe fn SetGraphiteModuleData(modules: &GraphiteModules) {
    let previous = sdata_sksl_graphite_frag.load(Ordering::Acquire);
    debug_assert!(
        !previous.is_null() && *previous == 0,
        "We should only initialize this once"
    );
    sdata_sksl_graphite_frag.store(modules.fFragmentShader as *mut c_char, Ordering::Release);
    sdata_sksl_graphite_vert.store(modules.fVertexShader as *mut c_char, Ordering::Release);
    debug_assert!(!modules.fFragmentShader.is_null() && *modules.fFragmentShader != 0);
    debug_assert!(!modules.fVertexShader.is_null() && *modules.fVertexShader != 0);
}
