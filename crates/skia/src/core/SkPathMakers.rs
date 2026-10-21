use crate::include::core::SkPathTypes::SkPathDirection;
use crate::src::core::SkPoint::SkPoint;
use crate::src::core::SkRRect::{Corner, SkRRect};
use crate::src::core::SkRect::SkRect;

// Source: src/core/SkPathMakers.h:15-37
pub struct SkPath_PointIterator<const N: usize> {
    fPts: [SkPoint; N],
    fCurrent: usize,
    fAdvance: usize,
}
impl<const N: usize> SkPath_PointIterator<N> {
    #[inline]
    fn new(dir: SkPathDirection, start_index: u32) -> Self {
        Self {
            fPts: [SkPoint::make(0.0, 0.0); N],
            fCurrent: start_index as usize % N,
            fAdvance: if dir == SkPathDirection::kCW {
                1
            } else {
                N - 1
            },
        }
    }
    #[inline]
    pub fn current(&self) -> SkPoint {
        crate::SkASSERT!(@source b"src/core/SkPathMakers.h\0".as_ptr(),22; @condition b"fCurrent < N\0".as_ptr(); self.fCurrent < N);
        self.fPts[self.fCurrent]
    }
    #[inline]
    pub fn next(&mut self) -> SkPoint {
        self.fCurrent = (self.fCurrent + self.fAdvance) % N;
        self.current()
    }
}

// Source: src/core/SkPathMakers.h:39-49
pub struct SkPath_RectPointIterator {
    base: SkPath_PointIterator<4>,
}
impl SkPath_RectPointIterator {
    #[inline]
    pub fn new(rect: &SkRect, dir: SkPathDirection, start_index: u32) -> Self {
        let mut base = SkPath_PointIterator::new(dir, start_index);
        base.fPts = [
            SkPoint::make(rect.fLeft, rect.fTop),
            SkPoint::make(rect.fRight, rect.fTop),
            SkPoint::make(rect.fRight, rect.fBottom),
            SkPoint::make(rect.fLeft, rect.fBottom),
        ];
        Self { base }
    }
}
impl core::ops::Deref for SkPath_RectPointIterator {
    type Target = SkPath_PointIterator<4>;
    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.base
    }
}
impl core::ops::DerefMut for SkPath_RectPointIterator {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}

// Source: src/core/SkPathMakers.h:51-64
pub struct SkPath_OvalPointIterator {
    base: SkPath_PointIterator<4>,
}
impl SkPath_OvalPointIterator {
    #[inline]
    pub fn new(rect: &SkRect, dir: SkPathDirection, start_index: u32) -> Self {
        let mut base = SkPath_PointIterator::new(dir, start_index);
        let cx = rect.centerX();
        let cy = rect.centerY();
        base.fPts = [
            SkPoint::make(cx, rect.fTop),
            SkPoint::make(rect.fRight, cy),
            SkPoint::make(cx, rect.fBottom),
            SkPoint::make(rect.fLeft, cy),
        ];
        Self { base }
    }
}
impl core::ops::Deref for SkPath_OvalPointIterator {
    type Target = SkPath_PointIterator<4>;
    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.base
    }
}
impl core::ops::DerefMut for SkPath_OvalPointIterator {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}

// Source: src/core/SkPathMakers.h:66-86
pub struct SkPath_RRectPointIterator {
    base: SkPath_PointIterator<8>,
}
impl SkPath_RRectPointIterator {
    #[inline]
    pub fn new(rect: &SkRRect, dir: SkPathDirection, start_index: u32) -> Self {
        let mut base = SkPath_PointIterator::new(dir, start_index);
        let bounds = rect.getBounds();
        let l = bounds.fLeft;
        let t = bounds.fTop;
        let r = bounds.fRight;
        let b = bounds.fBottom;
        base.fPts = [
            SkPoint::make(l + rect.radii(Corner::kUpperLeft_Corner).fX, t),
            SkPoint::make(r - rect.radii(Corner::kUpperRight_Corner).fX, t),
            SkPoint::make(r, t + rect.radii(Corner::kUpperRight_Corner).fY),
            SkPoint::make(r, b - rect.radii(Corner::kLowerRight_Corner).fY),
            SkPoint::make(r - rect.radii(Corner::kLowerRight_Corner).fX, b),
            SkPoint::make(l + rect.radii(Corner::kLowerLeft_Corner).fX, b),
            SkPoint::make(l, b - rect.radii(Corner::kLowerLeft_Corner).fY),
            SkPoint::make(l, t + rect.radii(Corner::kUpperLeft_Corner).fY),
        ];
        Self { base }
    }
}
impl core::ops::Deref for SkPath_RRectPointIterator {
    type Target = SkPath_PointIterator<8>;
    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.base
    }
}
impl core::ops::DerefMut for SkPath_RRectPointIterator {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
