// Source-aligned Skia translation. Rust methods use snake_case; fields retain upstream names.
#![allow(non_snake_case)]

use crate::include::core::SkScalar::{SK_ScalarNearlyZero, SkScalarNearlyEqual};
use crate::src::core::SkPoint::SkPoint;
use crate::src::core::SkRRect::SkRRect;
pub struct SkRRectPriv;

// Source: src/core/SkRRectPriv.h:18-20
impl SkRRectPriv {
    #[inline]
    pub fn IsCircle(rr: &SkRRect) -> bool {
        rr.isOval() && SkScalarNearlyEqual(rr.fRadii[0].fX, rr.fRadii[0].fY, SK_ScalarNearlyZero)
    }
}

// Source: src/core/SkRRectPriv.h:22-25
impl SkRRectPriv {
    #[inline]
    pub fn GetSimpleRadii(rr: &SkRRect) -> SkPoint {
        crate::SkASSERT!(@source b"src/core/SkRRectPriv.h\0".as_ptr(),23; @condition b"!rr.isComplex()\0".as_ptr(); !rr.isComplex());
        rr.fRadii[0]
    }
}

// Source: src/core/SkRRectPriv.h:27-29
impl SkRRectPriv {
    #[inline]
    pub fn IsSimpleCircular(rr: &SkRRect) -> bool {
        rr.isSimple() && SkScalarNearlyEqual(rr.fRadii[0].fX, rr.fRadii[0].fY, SK_ScalarNearlyZero)
    }
}

// Source: src/core/SkRRectPriv.h:35-37
impl SkRRectPriv {
    #[inline]
    pub fn EqualRadii(rr: &SkRRect) -> bool {
        rr.isRect() || Self::IsCircle(rr) || Self::IsSimpleCircular(rr)
    }
}

// Source: src/core/SkRRectPriv.h:39-39
impl SkRRectPriv {
    pub fn GetRadiiArray(rr: &SkRRect) -> &[SkPoint; 4] {
        &rr.fRadii
    }
}
