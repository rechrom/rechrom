// Root module wiring for delivered external source translations.
// Unresolved true native dependencies remain visible; zero Skia row credit.
pub mod allocator;
pub mod auto_reset;
pub mod check;
pub mod check_op;
pub mod check_version_internal;
pub mod containers;
pub mod dcheck_is_on;
pub mod debug;
pub mod immediate_crash;
pub mod location;
pub mod logging;
pub mod memory;
pub mod not_fatal_until;
pub mod numerics;
pub mod posix;
pub mod process;
pub mod rand_util;
pub mod rand_util_posix;
pub mod strings;
pub mod synchronization;
pub mod threading;
pub mod time;

#[cfg(not(base_native_build_inputs_configured))]
compile_error!(
    "Supply actual resolved Chromium BUILDFLAG inputs before selecting external source branches"
);

pub mod sampling_heap_profiler;
