// Copyright 2016 Google Inc. BSD-style license.
// Source: src/core/SkGlobalInitialization_core.cpp:11-18
pub fn RegisterFlattenablesIfNeeded() {
    static ONCE: crate::include::private::SkOnce::SkOnce =
        crate::include::private::SkOnce::SkOnce::new();
    // The sole initialization thread fills and sorts the native fixed table;
    // SkOnce release/acquire publishes its complete immutable prefix to readers.
    unsafe {
        crate::include::private::SkOnce::SkOnce::call_once(core::ptr::addr_of!(ONCE), || {
            crate::src::ports::SkGlobalInitialization_default::InitEffects();
            crate::src::ports::SkGlobalInitialization_default::InitImageFilters();
            super::SkFlattenable::Finalize();
        })
    };
}
