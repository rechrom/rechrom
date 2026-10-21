// Copyright 2011 The Chromium Authors. BSD-style license.
// Source: base/debug/alias.cc:11-13
// The empty native implementation is intentionally NOT visible in this crate.
// Root must link its same concrete separate Rust TU with LTO/bitcode disabled.
// Actual Itanium C++ ordinary-function ABI: void(const void*).
#[cfg(all(pa_cpp_abi = "itanium", not(skia_alias_separate_tu)))]
unsafe extern "C" {
    #[link_name = "_ZN4base5debug5AliasEPKv"]
    pub fn Alias(var: *const core::ffi::c_void);
}
#[cfg(not(pa_cpp_abi = "itanium"))]
compile_error!("Alias requires actual selected C++ symbol/ABI binding; no metadata reinterpretation or visible empty substitute");

// Source: base/debug/alias.h:117-119
macro_rules! NO_CODE_FOLDING {
    (@source $native_line:expr) => {
        let line_number: core::ffi::c_int = $native_line;
        unsafe {
            $crate::base::debug::alias::Alias(core::ptr::addr_of!(line_number).cast());
        }
    };
}
pub(crate) use NO_CODE_FOLDING;

// Source: base/debug/alias.cc:11-13
// Same canonical file, separately compiled native TU only. The main crate
// retains only the external declaration above, so its optimizer cannot see this body.
#[cfg(all(skia_alias_separate_tu, pa_cpp_abi = "itanium"))]
#[inline(never)]
#[unsafe(export_name = "_ZN4base5debug5AliasEPKv")]
pub unsafe extern "C" fn Alias(_var: *const core::ffi::c_void) {}
