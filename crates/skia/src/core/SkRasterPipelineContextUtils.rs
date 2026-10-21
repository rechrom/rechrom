// Copyright 2023 Google LLC. BSD-style license.
use crate::src::core::SkArenaAlloc::SkArenaAlloc;
use core::mem::{size_of, transmute_copy};

// Source: src/core/SkRasterPipelineContextUtils.h:19-20
// Rust cannot select a function's return type by sizeof(T). This explicit sum
// preserves the native value-versus-borrow choice without cloning large T.
pub enum UnpackedType<'a, T> {
    Value(T),
    Borrowed(&'a T),
}

// Source: src/core/SkRasterPipelineContextUtils.h:27-35
/// Safety: the pointer-sized context's complete object representation must be
/// initialized (including padding). Any borrowed storage in a large context
/// must outlive the arena, including its destructor chain. The packed pointer
/// is opaque bits in the small branch and must never be dereferenced.
pub unsafe fn Pack<T: Copy>(ctx: &T, alloc: &mut SkArenaAlloc) -> *mut core::ffi::c_void {
    if size_of::<T>() <= size_of::<*mut core::ffi::c_void>() {
        // Native sk_bit_cast has the same sizeof equality static_assert, so a
        // smaller-than-pointer T is not an admissible native instantiation.
        assert_eq!(size_of::<T>(), size_of::<*mut core::ffi::c_void>());
        transmute_copy(ctx)
    } else {
        alloc.make_with(|_, _| *ctx).cast()
    }
}

// Source: src/core/SkRasterPipelineContextUtils.h:27-35
// Language adaptation: the nontrivial native copy-constructor branch has an
// explicit Clone-only entrypoint. This adds no second source algorithm/count.
/// Safety: T::clone must implement the native copy constructor and must not
/// unwind after the arena installs its destructor footer. Every borrow retained
/// by the clone or accessed by its destructor must outlive the arena. The
/// resulting object belongs to alloc and must never be individually freed.
/// Small non-Copy objects are rejected before cloning or arena mutation; their
/// ownership cannot be duplicated into the pointer-bit branch.
pub unsafe fn PackLargeCloned<T: Clone>(
    ctx: &T,
    alloc: &mut SkArenaAlloc,
) -> *mut core::ffi::c_void {
    assert!(size_of::<T>() > size_of::<*mut core::ffi::c_void>());
    // make_with installs the actual native arena footer before constructing T.
    // Invoke clone only there, preserving constructor/allocation/drop ordering.
    alloc.make_with(|_, _| ctx.clone()).cast()
}

// Source: src/core/SkRasterPipelineContextUtils.h:42-51
/// Safety: ctx must come from Pack for precisely T. A large context's arena
/// allocation must remain live for the returned borrow's lifetime.
pub unsafe fn Unpack<'a, T: Copy>(ctx: *const T) -> UnpackedType<'a, T> {
    if size_of::<T>() <= size_of::<*const T>() {
        assert_eq!(size_of::<T>(), size_of::<*const T>());
        UnpackedType::Value(transmute_copy(&ctx))
    } else {
        UnpackedType::Borrowed(&*ctx)
    }
}

// Source: src/core/SkRasterPipelineContextUtils.h:42-51
// Language adaptation: a separate large-only borrowed result removes the Copy
// bound without weakening the small native bit-cast's Copy-only contract.
/// Safety: ctx denotes an initialized, aligned T in a live arena allocation of
/// the matching type. The arena, T, and all storage retained by T must outlive
/// the returned borrow, and no mutation/reset/drop may overlap that borrow.
/// Native large Unpack returns const T&: this performs no clone or ownership
/// transfer. An owned copy, when needed, remains an explicit caller clone.
pub unsafe fn UnpackLarge<'a, T>(ctx: *const T) -> &'a T {
    assert!(size_of::<T>() > size_of::<*const T>());
    &*ctx
}
