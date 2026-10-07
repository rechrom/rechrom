// Copyright 2006 The Android Open Source Project
// Copyright 2020 Yevhenii Reizner
//
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

//! A [tiny-skia](https://github.com/linebender/tiny-skia) Bezier path implementation.
//!
//! Provides a memory-efficient Bezier path container, path builder, path stroker and path dasher.
//!
//! Also provides some basic geometry types, but they will be moved to an external crate eventually.
//!
//! Note that all types use single precision floats (`f32`), just like [Skia](https://skia.org/).

#![allow(clippy::approx_constant)]
#![allow(clippy::collapsible_if)]
#![allow(clippy::eq_op)]
#![allow(clippy::excessive_precision)]
#![allow(clippy::identity_op)]
#![allow(clippy::manual_range_contains)]
#![allow(clippy::neg_cmp_op_on_partial_ord)]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::upper_case_acronyms)]
#![allow(clippy::wrong_self_convention)]

#[cfg(not(any(feature = "std", feature = "no-std-float")))]
compile_error!("You have to activate either the `std` or the `no-std-float` feature.");

#[cfg(feature = "std")]
extern crate std;

extern crate alloc;

pub(crate) use crate::src::utils::SkDashPath as dash;
pub(crate) mod f32x2_t {
    pub use crate::src::core::SkVx::skvx::float2 as f32x2;
}
pub(crate) mod f32x4_t {
    pub use crate::src::core::SkVx::skvx::PathFloat4 as f32x4;
}
pub(crate) use crate::include::core::SkRect as rect;
pub(crate) use crate::include::core::SkScalar as scalar;
pub(crate) use crate::include::core::SkSize as size;
pub(crate) use crate::include::private::SkFloatingPoint as floating_point;
pub use crate::src::core::SkGeometry as path_geometry;
pub(crate) use crate::src::core::SkMatrix as transform;
pub(crate) use crate::src::core::SkPath as path;
pub(crate) use crate::src::core::SkPathBuilder as path_builder;
pub(crate) use crate::src::core::SkStroke as stroker;

pub use dash::StrokeDash;
pub use f32x2_t::f32x2;
pub use floating_point::*;
pub use path::*;
pub use path_builder::*;
pub use rect::*;
pub use scalar::*;
pub use size::*;
pub use stroker::*;
pub use transform::*;

/// An integer length that is guarantee to be > 0
pub(crate) type LengthU32 = core::num::NonZeroU32;

pub use crate::include::core::SkPoint::Point;
