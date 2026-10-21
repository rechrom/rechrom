use crate::src::core::SkPoint::{set_point_length, SkIPoint, SkPoint};
use crate::src::core::SkRect::SkRect;
pub struct SkPointPriv;

// Source: src/core/SkPointPriv.h:16-20
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum Side {
    kLeft_Side = -1,
    kOn_Side = 0,
    kRight_Side = 1,
}

// Source: src/core/SkPointPriv.h:22-24
impl SkPointPriv {
    #[inline]
    pub fn AreFinite(array: &[SkPoint]) -> bool {
        array.iter().all(SkPoint::is_finite)
    }
}

// Source: src/core/SkPointPriv.h:26-26
impl SkPointPriv {
    pub fn AsScalars(pt: &SkPoint) -> &[f32; 2] {
        // SkPoint is repr(C), two adjacent f32 values.
        unsafe { &*(core::ptr::from_ref(pt).cast::<[f32; 2]>()) }
    }
}

// Source: src/core/SkPointPriv.h:28-30
impl SkPointPriv {
    #[inline]
    pub fn CanNormalize(dx: f32, dy: f32) -> bool {
        dx.is_finite() && dy.is_finite() && (dx != 0.0 || dy != 0.0)
    }
}

// Source: src/core/SkPointPriv.h:35-38
impl SkPointPriv {
    #[inline]
    pub fn DistanceToLineBetween(
        pt: SkPoint,
        a: SkPoint,
        b: SkPoint,
        side: Option<&mut Side>,
    ) -> f32 {
        Self::DistanceToLineBetweenSqd(pt, a, b, side).sqrt()
    }
}

// Source: src/core/SkPointPriv.h:43-46
impl SkPointPriv {
    #[inline]
    pub fn DistanceToLineSegmentBetween(pt: SkPoint, a: SkPoint, b: SkPoint) -> f32 {
        Self::DistanceToLineSegmentBetweenSqd(pt, a, b).sqrt()
    }
}

// Source: src/core/SkPointPriv.h:48-52
impl SkPointPriv {
    #[inline]
    pub fn DistanceToSqd(pt: SkPoint, a: SkPoint) -> f32 {
        let dx = pt.fX - a.fX;
        let dy = pt.fY - a.fY;
        dx * dx + dy * dy
    }
}

// Source: src/core/SkPointPriv.h:54-56
impl SkPointPriv {
    #[inline]
    pub fn EqualsWithinTolerance(p1: SkPoint, p2: SkPoint) -> bool {
        !Self::CanNormalize(p1.fX - p2.fX, p1.fY - p2.fY)
    }
}

// Source: src/core/SkPointPriv.h:58-61
impl SkPointPriv {
    #[inline]
    pub fn EqualsWithinToleranceWith(p: SkPoint, q: SkPoint, tol: f32) -> bool {
        debug_assert!(tol >= 0.0);
        (p.fX - q.fX).abs() <= tol && (p.fY - q.fY).abs() <= tol
    }
}

// Source: src/core/SkPointPriv.h:63-65
impl SkPointPriv {
    #[inline]
    pub fn LengthSqd(pt: SkPoint) -> f32 {
        SkPoint::dot_product(pt, pt)
    }
}

// Source: src/core/SkPointPriv.h:67-70
impl SkPointPriv {
    #[inline]
    pub fn Negate(pt: &mut SkIPoint) {
        pt.fX = pt.fX.wrapping_neg();
        pt.fY = pt.fY.wrapping_neg();
    }
}

// Source: src/core/SkPointPriv.h:72-77
impl SkPointPriv {
    #[inline]
    pub fn RotateCCWTo(src: SkPoint, dst: &mut SkPoint) {
        let tmp = src.fX;
        dst.fX = src.fY;
        dst.fY = -tmp;
    }
}

// Source: src/core/SkPointPriv.h:79-81
impl SkPointPriv {
    #[inline]
    pub fn RotateCCW(pt: &mut SkPoint) {
        Self::RotateCCWTo(*pt, pt);
    }
}

// Source: src/core/SkPointPriv.h:83-88
impl SkPointPriv {
    #[inline]
    pub fn RotateCWTo(src: SkPoint, dst: &mut SkPoint) {
        let tmp = src.fX;
        dst.fX = -src.fY;
        dst.fY = tmp;
    }
}

// Source: src/core/SkPointPriv.h:90-92
impl SkPointPriv {
    #[inline]
    pub fn RotateCW(pt: &mut SkPoint) {
        Self::RotateCWTo(*pt, pt);
    }
}

// Source: src/core/SkPointPriv.h:96-99
impl SkPointPriv {
    #[inline]
    pub fn MakeOrthog(vec: SkPoint, side: Side) -> SkPoint {
        debug_assert!(side != Side::kOn_Side);
        if side == Side::kRight_Side {
            SkPoint::make(-vec.fY, vec.fX)
        } else {
            SkPoint::make(vec.fY, -vec.fX)
        }
    }
}

// Source: src/core/SkPointPriv.h:102-110
impl SkPointPriv {
    /// # Safety
    /// v points to four writable, aligned SkPoint slots at the given byte stride.
    #[inline]
    pub unsafe fn SetRectFan(v: *mut SkPoint, l: f32, t: f32, r: f32, b: f32, stride: usize) {
        debug_assert!(stride >= core::mem::size_of::<SkPoint>());
        for (i, (x, y)) in [(l, t), (l, b), (r, b), (r, t)].into_iter().enumerate() {
            unsafe {
                (*v.byte_add(i * stride)).set(x, y);
            }
        }
    }
}

// Source: src/core/SkPointPriv.h:113-121
impl SkPointPriv {
    /// # Safety
    /// v points to four writable, aligned SkPoint slots at the given byte stride.
    #[inline]
    pub unsafe fn SetRectTriStrip(v: *mut SkPoint, l: f32, t: f32, r: f32, b: f32, stride: usize) {
        debug_assert!(stride >= core::mem::size_of::<SkPoint>());
        for (i, (x, y)) in [(l, t), (l, b), (r, t), (r, b)].into_iter().enumerate() {
            unsafe {
                (*v.byte_add(i * stride)).set(x, y);
            }
        }
    }
}

// Source: src/core/SkPointPriv.h:122-124
impl SkPointPriv {
    /// # Safety
    /// Same writable strided storage requirement as SetRectTriStrip.
    #[inline]
    pub unsafe fn SetRectTriStripRect(v: *mut SkPoint, rect: &SkRect, stride: usize) {
        unsafe {
            Self::SetRectTriStrip(v, rect.fLeft, rect.fTop, rect.fRight, rect.fBottom, stride);
        }
    }
}
