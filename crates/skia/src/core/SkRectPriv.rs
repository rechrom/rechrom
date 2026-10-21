use crate::src::core::SkPoint::SkPoint;
use crate::src::core::SkRect::{SkIRect, SkRect};
pub struct SkRectPriv;

// Source: src/core/SkRectPriv.h:22-27
impl SkRectPriv {
    #[inline]
    pub fn MakeILarge() -> SkIRect {
        let large = 1 << 29;
        SkIRect::make_ltrb(-large, -large, large, large)
    }
}

// Source: src/core/SkRectPriv.h:29-31
impl SkRectPriv {
    #[inline]
    pub fn MakeILargestInverted() -> SkIRect {
        SkIRect::make_ltrb(i32::MAX, i32::MAX, i32::MIN, i32::MIN)
    }
}

// Source: src/core/SkRectPriv.h:33-37
impl SkRectPriv {
    #[inline]
    pub fn MakeLargeS32() -> SkRect {
        let mut r = SkRect::default();
        r.set(&Self::MakeILarge());
        r
    }
}

// Source: src/core/SkRectPriv.h:39-41
impl SkRectPriv {
    #[inline]
    pub fn MakeLargest() -> SkRect {
        SkRect::MakeLTRB(-f32::MAX, -f32::MAX, f32::MAX, f32::MAX)
    }
}

// Source: src/core/SkRectPriv.h:43-45
impl SkRectPriv {
    #[inline]
    pub const fn MakeLargestInverted() -> SkRect {
        SkRect::MakeLTRB(f32::MAX, f32::MAX, -f32::MAX, -f32::MAX)
    }
}

// Source: src/core/SkRectPriv.h:47-52
impl SkRectPriv {
    #[inline]
    pub fn GrowToInclude(r: &mut SkRect, p: SkPoint) {
        r.fLeft = if r.fLeft < p.fX { r.fLeft } else { p.fX };
        r.fRight = if p.fX < r.fRight { r.fRight } else { p.fX };
        r.fTop = if r.fTop < p.fY { r.fTop } else { p.fY };
        r.fBottom = if p.fY < r.fBottom { r.fBottom } else { p.fY };
    }
}

// Source: src/core/SkRectPriv.h:56-59
impl SkRectPriv {
    #[inline]
    pub fn FitsInFixed(r: &SkRect) -> bool {
        use crate::src::core::SkMathPriv::SkFitsInFixed;
        SkFitsInFixed(r.fLeft)
            && SkFitsInFixed(r.fTop)
            && SkFitsInFixed(r.fRight)
            && SkFitsInFixed(r.fBottom)
    }
}
// Source: src/core/SkRectPriv.h:62-64
impl SkRectPriv {
    #[inline]
    pub fn HalfWidth(r: &SkRect) -> f32 {
        (0.5 * (-r.fLeft as f64 + r.fRight as f64)) as f32
    }
}

// Source: src/core/SkRectPriv.h:66-68
impl SkRectPriv {
    #[inline]
    pub fn HalfHeight(r: &SkRect) -> f32 {
        (0.5 * (-r.fTop as f64 + r.fBottom as f64)) as f32
    }
}
// Source: src/core/SkRectPriv.h:78-82
impl SkRectPriv {
    #[inline]
    pub fn SubtractRectValue(a: &SkRect, b: &SkRect) -> SkRect {
        let mut diff = SkRect::MakeEmpty();
        Self::Subtract(a, b, &mut diff);
        diff
    }
}
// Source: src/core/SkRectPriv.h:83-87
impl SkRectPriv {
    #[inline]
    pub fn SubtractIRectValue(a: &SkIRect, b: &SkIRect) -> SkIRect {
        let mut diff = SkIRect::make_empty();
        Self::SubtractIRect(a, b, &mut diff);
        diff
    }
}
