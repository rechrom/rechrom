use crate::src::core::SkPoint::SkPoint;

use crate::src::core::SkRect::SkRect;

use core::cell::Cell;

// Only these sealed known masks may be shared. All non-const source mutators require
// &mut self, including dirtyMatrixTypeCache and the private mask update helpers.
// The only remaining &self Cell writes are guarded by kUnknown_Mask (128):
// getType, rectStaysRect, and getPerspectiveTypeMaskOnly. Neither constant has that bit,
// and safe shared queries cannot introduce it. Arbitrary matrices retain Cell's !Sync.
struct KnownMatrixConstant(SkMatrix);

unsafe impl Sync for KnownMatrixConstant {}

const kScale_Shift: u32 = 1;

const kAffine_Shift: u32 = 2;

const kPerspective_Shift: u32 = 3;

const kRectStaysRect_Shift: u32 = 4;

// Source: include/core/SkMatrix.h:54-54
impl Default for SkMatrix {
    #[inline]
    fn default() -> Self {
        Self::with_mask([1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0], 16)
    }
}

// Source: include/core/SkMatrix.h:66-70
impl SkMatrix {
    #[inline]
    pub fn Scale(sx: f32, sy: f32) -> Self {
        let mut m = Self::default();
        m.setScale(sx, sy);
        m
    }
}

// Source: include/core/SkMatrix.h:82-86
impl SkMatrix {
    #[inline]
    pub fn Translate(dx: f32, dy: f32) -> Self {
        let mut m = Self::default();
        m.setTranslate(dx, dy);
        m
    }
}

// Source: include/core/SkMatrix.h:87-87
impl SkMatrix {
    #[inline]
    pub fn TranslateVector(t: SkPoint) -> Self {
        Self::Translate(t.fX, t.fY)
    }
}

// Source: include/core/SkMatrix.h:88-88
impl SkMatrix {
    #[inline]
    pub fn TranslateIVector(t: crate::src::core::SkPoint::SkIPoint) -> Self {
        Self::Translate(t.fX as f32, t.fY as f32)
    }
}

// Source: include/core/SkMatrix.h:97-101
impl SkMatrix {
    #[inline]
    pub fn RotateDeg(deg: f32) -> Self {
        let mut m = Self::default();
        m.setRotate(deg);
        m
    }
}

// Source: include/core/SkMatrix.h:102-106
impl SkMatrix {
    #[inline]
    pub fn RotateDegPivot(deg: f32, pt: SkPoint) -> Self {
        let mut m = Self::default();
        m.setRotatePivot(deg, pt.fX, pt.fY);
        m
    }
}

// Source: include/core/SkMatrix.h:107-109
impl SkMatrix {
    #[inline]
    pub fn RotateRad(rad: f32) -> Self {
        Self::RotateDeg(crate::include::core::SkScalar::SkRadiansToDegrees(rad))
    }
}

// Source: include/core/SkMatrix.h:117-121
impl SkMatrix {
    #[inline]
    pub fn Skew(kx: f32, ky: f32) -> Self {
        let mut m = Self::default();
        m.setSkew(kx, ky);
        m
    }
}

// Source: include/core/SkMatrix.h:129-134
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ScaleToFit {
    kFill_ScaleToFit,
    kStart_ScaleToFit,
    kCenter_ScaleToFit,
    kEnd_ScaleToFit,
}

// Source: include/core/SkMatrix.h:153-159
impl SkMatrix {
    #[allow(clippy::too_many_arguments)]
    #[inline]
    pub fn MakeAll(
        sx: f32,
        kx: f32,
        tx: f32,
        ky: f32,
        sy: f32,
        ty: f32,
        p0: f32,
        p1: f32,
        p2: f32,
    ) -> Self {
        let mut m = Self::default();
        m.setAll(sx, kx, tx, ky, sy, ty, p0, p1, p2);
        m
    }
}

// Source: include/core/SkMatrix.h:165-171
impl SkMatrix {
    pub const kIdentity_Mask: i32 = 0;
    pub const kTranslate_Mask: i32 = 1;
    pub const kScale_Mask: i32 = 2;
    pub const kAffine_Mask: i32 = 4;
    pub const kPerspective_Mask: i32 = 8;
}

// Source: include/core/SkMatrix.h:181-187
impl SkMatrix {
    #[inline]
    pub fn getType(&self) -> i32 {
        if self.fTypeMask.get() & Self::kUnknown_Mask != 0 {
            self.fTypeMask.set(self.computeTypeMask() as i32);
        }
        self.fTypeMask.get() & Self::kORableMasks
    }
}

// Source: include/core/SkMatrix.h:197-199
impl SkMatrix {
    #[inline]
    pub fn isIdentity(&self) -> bool {
        self.getType() == 0
    }
}

// Source: include/core/SkMatrix.h:210-212
impl SkMatrix {
    #[inline]
    pub fn isScaleTranslate(&self) -> bool {
        self.getType() & !3 == 0
    }
}

// Source: include/core/SkMatrix.h:222-222
impl SkMatrix {
    #[inline]
    pub fn isTranslate(&self) -> bool {
        self.getType() & !1 == 0
    }
}

// Source: include/core/SkMatrix.h:245-250
impl SkMatrix {
    #[inline]
    pub fn rectStaysRect(&self) -> bool {
        if self.fTypeMask.get() & Self::kUnknown_Mask != 0 {
            self.fTypeMask.set(self.computeTypeMask() as i32);
        }
        self.fTypeMask.get() & Self::kRectStaysRect_Mask != 0
    }
}

// Source: include/core/SkMatrix.h:273-273
impl SkMatrix {
    #[inline]
    pub fn preservesAxisAlignment(&self) -> bool {
        self.rectStaysRect()
    }
}

// Source: include/core/SkMatrix.h:286-289
impl SkMatrix {
    #[inline]
    pub fn hasPerspective(&self) -> bool {
        self.getPerspectiveTypeMaskOnly() & 8 != 0
    }
}

// Source: include/core/SkMatrix.h:327-335
impl SkMatrix {
    pub const kMScaleX: usize = 0;
    pub const kMSkewX: usize = 1;
    pub const kMTransX: usize = 2;
    pub const kMSkewY: usize = 3;
    pub const kMScaleY: usize = 4;
    pub const kMTransY: usize = 5;
    pub const kMPersp0: usize = 6;
    pub const kMPersp1: usize = 7;
    pub const kMPersp2: usize = 8;
}

// Source: include/core/SkMatrix.h:340-345
impl SkMatrix {
    pub const kAScaleX: usize = 0;
    pub const kASkewY: usize = 1;
    pub const kASkewX: usize = 2;
    pub const kAScaleY: usize = 3;
    pub const kATransX: usize = 4;
    pub const kATransY: usize = 5;
}

// Source: include/core/SkMatrix.h:354-357
impl core::ops::Index<usize> for SkMatrix {
    type Output = f32;
    #[inline]
    fn index(&self, index: usize) -> &f32 {
        debug_assert!(index < 9);
        &self.fMat[index]
    }
}

// Source: include/core/SkMatrix.h:366-369
impl SkMatrix {
    #[inline]
    pub fn get(&self, index: usize) -> f32 {
        debug_assert!(index < 9);
        self.fMat[index]
    }
}

// Source: include/core/SkMatrix.h:378-382
impl SkMatrix {
    #[inline]
    pub fn rc(&self, r: usize, c: usize) -> f32 {
        debug_assert!(r < 3 && c < 3);
        self.fMat[r * 3 + c]
    }
}

// Source: include/core/SkMatrix.h:389-389
impl SkMatrix {
    #[inline]
    pub fn getScaleX(&self) -> f32 {
        self.fMat[0]
    }
}

// Source: include/core/SkMatrix.h:396-396
impl SkMatrix {
    #[inline]
    pub fn getScaleY(&self) -> f32 {
        self.fMat[4]
    }
}

// Source: include/core/SkMatrix.h:404-404
impl SkMatrix {
    #[inline]
    pub fn getSkewY(&self) -> f32 {
        self.fMat[3]
    }
}

// Source: include/core/SkMatrix.h:412-412
impl SkMatrix {
    #[inline]
    pub fn getSkewX(&self) -> f32 {
        self.fMat[1]
    }
}

// Source: include/core/SkMatrix.h:419-419
impl SkMatrix {
    #[inline]
    pub fn getTranslateX(&self) -> f32 {
        self.fMat[2]
    }
}

// Source: include/core/SkMatrix.h:426-426
impl SkMatrix {
    #[inline]
    pub fn getTranslateY(&self) -> f32 {
        self.fMat[5]
    }
}

// Source: include/core/SkMatrix.h:432-432
impl SkMatrix {
    #[inline]
    pub fn getPerspX(&self) -> f32 {
        self.fMat[6]
    }
}

// Source: include/core/SkMatrix.h:438-438
impl SkMatrix {
    #[inline]
    pub fn getPerspY(&self) -> f32 {
        self.fMat[7]
    }
}

// Source: include/core/SkMatrix.h:450-454
impl core::ops::IndexMut<usize> for SkMatrix {
    #[inline]
    fn index_mut(&mut self, index: usize) -> &mut f32 {
        debug_assert!(index < 9);
        self.setTypeMask(128);
        &mut self.fMat[index]
    }
}

// Source: include/core/SkMatrix.h:463-468
impl SkMatrix {
    #[inline]
    pub fn set(&mut self, index: usize, value: f32) -> &mut Self {
        debug_assert!(index < 9);
        self.fMat[index] = value;
        self.setTypeMask(128);
        self
    }
}

// Source: include/core/SkMatrix.h:474-474
impl SkMatrix {
    #[inline]
    pub fn setScaleX(&mut self, v: f32) -> &mut Self {
        self.set(0, v)
    }
}

// Source: include/core/SkMatrix.h:480-480
impl SkMatrix {
    #[inline]
    pub fn setScaleY(&mut self, v: f32) -> &mut Self {
        self.set(4, v)
    }
}

// Source: include/core/SkMatrix.h:486-486
impl SkMatrix {
    #[inline]
    pub fn setSkewY(&mut self, v: f32) -> &mut Self {
        self.set(3, v)
    }
}

// Source: include/core/SkMatrix.h:492-492
impl SkMatrix {
    #[inline]
    pub fn setSkewX(&mut self, v: f32) -> &mut Self {
        self.set(1, v)
    }
}

// Source: include/core/SkMatrix.h:498-498
impl SkMatrix {
    #[inline]
    pub fn setTranslateX(&mut self, v: f32) -> &mut Self {
        self.set(2, v)
    }
}

// Source: include/core/SkMatrix.h:504-504
impl SkMatrix {
    #[inline]
    pub fn setTranslateY(&mut self, v: f32) -> &mut Self {
        self.set(5, v)
    }
}

// Source: include/core/SkMatrix.h:511-511
impl SkMatrix {
    #[inline]
    pub fn setPerspX(&mut self, v: f32) -> &mut Self {
        self.set(6, v)
    }
}

// Source: include/core/SkMatrix.h:518-518
impl SkMatrix {
    #[inline]
    pub fn setPerspY(&mut self, v: f32) -> &mut Self {
        self.set(7, v)
    }
}

// Source: include/core/SkMatrix.h:536-550
impl SkMatrix {
    #[allow(clippy::too_many_arguments)]
    #[inline]
    pub fn setAll(
        &mut self,
        sx: f32,
        kx: f32,
        tx: f32,
        ky: f32,
        sy: f32,
        ty: f32,
        p0: f32,
        p1: f32,
        p2: f32,
    ) -> &mut Self {
        self.fMat = [sx, kx, tx, ky, sy, ty, p0, p1, p2];
        self.setTypeMask(128);
        self
    }
}

// Source: include/core/SkMatrix.h:558-560
impl SkMatrix {
    pub fn get9(&self, buffer: &mut [f32; 9]) {
        *buffer = self.fMat;
    }
}

// Source: include/core/SkMatrix.h:600-600
impl SkMatrix {
    #[inline]
    pub fn setIdentity(&mut self) -> &mut Self {
        self.reset()
    }
}

// Source: include/core/SkMatrix.h:613-613
impl SkMatrix {
    #[inline]
    pub fn setTranslateVector(&mut self, v: &SkPoint) -> &mut Self {
        self.setTranslate(v.fX, v.fY)
    }
}

// Source: include/core/SkMatrix.h:1120-1123
impl SkMatrix {
    #[inline]
    pub fn RectToRectOrIdentity(src: &SkRect, dst: &SkRect, stf: ScaleToFit) -> Self {
        Self::Rect2Rect(src, dst, stf).unwrap_or_else(|| Self::I().clone())
    }
}

// Source: include/core/SkMatrix.h:1126-1133
impl SkMatrix {
    #[inline]
    pub fn setRectToRect(&mut self, src: &SkRect, dst: &SkRect, stf: ScaleToFit) -> bool {
        if let Some(mx) = Self::Rect2Rect(src, dst, stf) {
            *self = mx;
            true
        } else {
            self.reset();
            false
        }
    }
}

// Source: include/core/SkMatrix.h:1135-1140
impl SkMatrix {
    #[inline]
    pub fn MakeRectToRect(src: &SkRect, dst: &SkRect, stf: ScaleToFit) -> Self {
        Self::Rect2Rect(src, dst, stf).unwrap_or_else(|| Self::I().clone())
    }
}

// Source: include/core/SkMatrix.h:1142-1145
impl SkMatrix {
    #[inline]
    pub fn RectToRect(src: &SkRect, dst: &SkRect, mode: ScaleToFit) -> Self {
        Self::MakeRectToRect(src, dst, mode)
    }
}

// Source: include/core/SkMatrix.h:1158-1164
impl SkMatrix {
    #[inline]
    pub fn setPolyToPoly(&mut self, src: &[SkPoint], dst: &[SkPoint]) -> bool {
        if let Some(mx) = Self::PolyToPoly(src, dst) {
            *self = mx;
            true
        } else {
            false
        }
    }
}

// Source: include/core/SkMatrix.h:1172-1180
impl SkMatrix {
    #[inline]
    pub fn invert_to(&self, inverse: Option<&mut Self>) -> bool {
        if let Some(inv) = self.invert() {
            if let Some(out) = inverse {
                *out = inv;
            }
            true
        } else {
            false
        }
    }
}

// Source: include/core/SkMatrix.h:1235-1239
impl SkMatrix {
    #[inline]
    pub fn normalizePerspective(&mut self) {
        if self.fMat[8] != 1.0 {
            self.doNormalizePerspective();
        }
    }
}

// Source: include/core/SkMatrix.h:1294-1296
impl SkMatrix {
    #[inline]
    pub fn mapPointsInPlace(&self, pts: &mut [crate::src::core::SkPoint::SkPoint]) {
        let count = pts.len();
        let ptr = pts.as_mut_ptr();
        unsafe {
            (self.getMapPtsProc())(self, ptr, ptr, count);
        }
    }
}

// Source: include/core/SkMatrix.h:1294-1296
impl SkMatrix {
    #[inline]
    pub fn mapPoints_in_place(&self, pts: &mut [SkPoint]) {
        let ptr = pts.as_mut_ptr();
        unsafe {
            (self.getMapPtsProc())(self, ptr, ptr, min_count(pts, pts) as usize);
        }
    }
}

// Source: include/core/SkMatrix.h:1320-1324
impl SkMatrix {
    #[inline]
    pub fn mapHomogeneousPoint(
        &self,
        src: crate::src::core::SkPoint3::SkPoint3,
    ) -> crate::src::core::SkPoint3::SkPoint3 {
        let mut dst = [crate::src::core::SkPoint3::SkPoint3::default()];
        self.mapHomogeneousPoints(&mut dst, &[src]);
        dst[0]
    }
}

// Source: include/core/SkMatrix.h:1334-1338
impl SkMatrix {
    #[inline]
    pub fn mapPointToHomogeneous(
        &self,
        src: crate::src::core::SkPoint::SkPoint,
    ) -> crate::src::core::SkPoint3::SkPoint3 {
        let mut dst = [crate::src::core::SkPoint3::SkPoint3::default()];
        self.mapPointsToHomogeneous(&mut dst, &[src]);
        dst[0]
    }
}

// Source: include/core/SkMatrix.h:1355-1361
impl SkMatrix {
    #[inline]
    pub fn mapPoint(
        &self,
        p: crate::src::core::SkPoint::SkPoint,
    ) -> crate::src::core::SkPoint::SkPoint {
        if self.hasPerspective() {
            self.mapPointPerspective(p)
        } else {
            self.mapPointAffine(p)
        }
    }
}

// Source: include/core/SkMatrix.h:1367-1373
impl SkMatrix {
    #[inline]
    pub fn mapPointAffine(
        &self,
        p: crate::src::core::SkPoint::SkPoint,
    ) -> crate::src::core::SkPoint::SkPoint {
        crate::SkASSERT!(@source b"include/core/SkMatrix.h\0".as_ptr(),1368; @condition b"!this->hasPerspective()\0".as_ptr(); !self.hasPerspective());
        crate::src::core::SkPoint::SkPoint::make(
            (p.fX * self.fMat[0] + p.fY * self.fMat[1]) + self.fMat[2],
            (p.fX * self.fMat[3] + p.fY * self.fMat[4]) + self.fMat[5],
        )
    }
}

// Source: include/core/SkMatrix.h:1389-1399
impl SkMatrix {
    #[inline]
    pub fn mapOrigin(&self) -> SkPoint {
        let mut x = self.getTranslateX();
        let mut y = self.getTranslateY();
        if self.hasPerspective() {
            let mut w = self.fMat[8];
            if w != 0.0 {
                w = 1.0 / w;
            }
            x *= w;
            y *= w;
        }
        SkPoint::make(x, y)
    }
}

// Source: include/core/SkMatrix.h:1455-1457
impl SkMatrix {
    #[inline]
    pub fn mapVectors_in_place(&self, vecs: &mut [SkPoint]) {
        let ptr = vecs.as_mut_ptr();
        unsafe {
            self.mapVectors_raw(ptr, ptr, min_count(vecs, vecs) as usize);
        }
    }
}

// Source: include/core/SkMatrix.h:1476-1479
impl SkMatrix {
    #[inline]
    pub fn mapVector(&self, vec: SkPoint) -> SkPoint {
        let mut out = [SkPoint::default()];
        self.mapVectors(&mut out, &[vec]);
        out[0]
    }
}

// Source: include/core/SkMatrix.h:1480-1482
impl SkMatrix {
    #[inline]
    pub fn mapVectorXY(&self, dx: f32, dy: f32) -> SkPoint {
        self.mapVector(SkPoint::make(dx, dy))
    }
}

// Source: include/core/SkMatrix.h:1507-1509
impl SkMatrix {
    #[inline]
    pub fn mapRect_in_place(&self, rect: &mut SkRect) -> bool {
        let src = *rect;
        self.mapRect(rect, &src)
    }
}

// Source: include/core/SkMatrix.h:1516-1520
impl SkMatrix {
    #[inline]
    pub fn mapRect_value(&self, src: &SkRect) -> SkRect {
        let mut dst = SkRect::MakeEmpty();
        self.mapRect(&mut dst, src);
        dst
    }
}

// Source: include/core/SkMatrix.h:1550-1553
impl SkMatrix {
    pub fn mapRectToQuad(&self, dst: &mut [SkPoint; 4], rect: &SkRect) {
        self.mapPoints(
            dst,
            &rect.toQuad(crate::include::core::SkPathTypes::SkPathDirection::kCW),
        );
    }
}

// Source: include/core/SkMatrix.h:1704-1708
impl SkMatrix {
    #[inline]
    pub fn Concat(a: &Self, b: &Self) -> Self {
        let mut result = Self::default();
        result.setConcat(a, b);
        result
    }
}

// Source: include/core/SkMatrix.h:1710-1712
impl core::ops::Mul<&SkMatrix> for &SkMatrix {
    type Output = SkMatrix;
    #[inline]
    fn mul(self, b: &SkMatrix) -> SkMatrix {
        SkMatrix::Concat(self, b)
    }
}

// Source: include/core/SkMatrix.h:1717-1719
impl SkMatrix {
    #[inline]
    pub fn dirtyMatrixTypeCache(&mut self) {
        self.setTypeMask(128);
    }
}

// Source: include/core/SkMatrix.h:1732-1734
impl SkMatrix {
    #[inline]
    pub fn setScaleTranslate(&mut self, sx: f32, sy: f32, tx: f32, ty: f32) {
        *self = Self::ScaleTranslate(sx, sy, tx, ty);
    }
}

// Source: include/core/SkMatrix.h:1741-1741
impl SkMatrix {
    #[inline]
    pub fn isFinite(&self) -> bool {
        self.fMat.iter().all(|v| v.is_finite())
    }
}

// Source: include/core/SkMatrix.h:1750-1750
impl SkMatrix {
    pub(crate) const kRectStaysRect_Mask: i32 = 0x10;
}

// Source: include/core/SkMatrix.h:1755-1755
impl SkMatrix {
    pub(crate) const kOnlyPerspectiveValid_Mask: i32 = 0x40;
}

// Source: include/core/SkMatrix.h:1757-1757
impl SkMatrix {
    pub(crate) const kUnknown_Mask: i32 = 0x80;
}

// Source: include/core/SkMatrix.h:1759-1762
impl SkMatrix {
    pub(crate) const kORableMasks: i32 =
        Self::kTranslate_Mask | Self::kScale_Mask | Self::kAffine_Mask | Self::kPerspective_Mask;
}

// Source: include/core/SkMatrix.h:1764-1768
impl SkMatrix {
    pub(crate) const kAllMasks: i32 = Self::kORableMasks | Self::kRectStaysRect_Mask;
}

// Source: include/core/SkMatrix.h:1770-1771
// Cell preserves upstream mutable type cache on const query methods; it intentionally does not make this type Sync.
#[derive(Clone, Debug)]
pub struct SkMatrix {
    pub(crate) fMat: [f32; 9],
    fTypeMask: Cell<i32>,
}

// Source: include/core/SkMatrix.h:1773-1779
impl SkMatrix {
    fn with_mask(fMat: [f32; 9], typeMask: i32) -> Self {
        Self {
            fMat,
            fTypeMask: Cell::new(typeMask),
        }
    }
}

// Source: include/core/SkMatrix.h:1786-1792
impl SkMatrix {
    #[inline]
    fn setTypeMask(&mut self, mask: i32) {
        debug_assert!(mask == 128 || mask & Self::kAllMasks == mask || (192 & mask) == 192);
        self.fTypeMask.set(mask);
    }
}

// Source: include/core/SkMatrix.h:1794-1797
impl SkMatrix {
    #[inline]
    fn orTypeMask(&mut self, mask: i32) {
        debug_assert!(mask & 15 == mask);
        self.fTypeMask.set(self.fTypeMask.get() | mask);
    }
}

// Source: include/core/SkMatrix.h:1799-1803
impl SkMatrix {
    #[inline]
    fn clearTypeMask(&mut self, mask: i32) {
        debug_assert!(mask & Self::kAllMasks == mask);
        self.fTypeMask.set(self.fTypeMask.get() & !mask);
    }
}

// Source: include/core/SkMatrix.h:1805-1811
impl SkMatrix {
    #[inline]
    fn getPerspectiveTypeMaskOnly(&self) -> i32 {
        if self.fTypeMask.get() & Self::kUnknown_Mask != 0 && self.fTypeMask.get() & 64 == 0 {
            self.fTypeMask.set(self.computePerspectiveTypeMask() as i32);
        }
        self.fTypeMask.get() & Self::kORableMasks
    }
}

// Source: include/core/SkMatrix.h:1816-1821
impl SkMatrix {
    #[inline]
    pub fn isTriviallyIdentity(&self) -> bool {
        if self.fTypeMask.get() & Self::kUnknown_Mask != 0 {
            return false;
        }
        self.fTypeMask.get() & Self::kORableMasks == 0
    }
}

// Source: include/core/SkMatrix.h:1823-1829
impl SkMatrix {
    #[inline]
    fn updateTranslateMask(&mut self) {
        if (self.fMat[2] != 0.0) | (self.fMat[5] != 0.0) {
            self.fTypeMask.set(self.fTypeMask.get() | 1);
        } else {
            self.fTypeMask.set(self.fTypeMask.get() & !1);
        }
    }
}

// Source: include/core/SkMatrix.h:1838-1839
pub(crate) type MapPtsProc = unsafe fn(
    &SkMatrix,
    *mut crate::src::core::SkPoint::SkPoint,
    *const crate::src::core::SkPoint::SkPoint,
    usize,
);

// Source: include/core/SkMatrix.h:1841-1844
impl SkMatrix {
    #[inline]
    fn GetMapPtsProc(mask: i32) -> MapPtsProc {
        debug_assert!(mask & !Self::kAllMasks == 0);
        Self::gMapPtsProcs[(mask & Self::kAllMasks) as usize]
    }
}

// Source: include/core/SkMatrix.h:1846-1848
impl SkMatrix {
    #[inline]
    pub(crate) fn getMapPtsProc(&self) -> MapPtsProc {
        Self::GetMapPtsProc(self.getType())
    }
}

// Source: src/core/SkMatrix.cpp:30-32
// Adapted Source: src/core/SkMatrix.cpp:30-32
fn min_count<S, T>(a: &[S], b: &[T]) -> i32 {
    let count = a.len().min(b.len());
    // Native SkToInt asserts this bound in debug. Enforce it before safe Rust dispatch in every build.
    assert!(
        count <= i32::MAX as usize,
        "native point span count exceeds i32"
    );
    count as i32
}

// Source: src/core/SkMatrix.cpp:34-51
impl SkMatrix {
    pub fn doNormalizePerspective(&mut self) {
        if self.fMat[6] == 0.0 && self.fMat[7] == 0.0 {
            let p2 = self.fMat[8];
            if p2 != 0.0 && p2 != 1.0 {
                let inv = 1.0f64 / p2 as f64;
                for i in 0..6 {
                    self.fMat[i] = (self.fMat[i] as f64 * inv) as f32;
                }
                self.fMat[8] = 1.0;
            }
            self.setTypeMask(128);
        }
    }
}

// Source: src/core/SkMatrix.cpp:53-53
impl SkMatrix {
    pub fn reset(&mut self) -> &mut Self {
        *self = Self::default();
        self
    }
}

// Source: src/core/SkMatrix.cpp:55-59
impl SkMatrix {
    pub fn set9(&mut self, buffer: &[f32; 9]) -> &mut Self {
        self.fMat = *buffer;
        self.setTypeMask(128);
        self
    }
}

// Source: src/core/SkMatrix.cpp:61-73
impl SkMatrix {
    pub fn setAffine(&mut self, buffer: &[f32; 6]) -> &mut Self {
        self.fMat = [
            buffer[0], buffer[2], buffer[4], buffer[1], buffer[3], buffer[5], 0.0, 0.0, 1.0,
        ];
        self.setTypeMask(128);
        self
    }
}

// Source: src/core/SkMatrix.cpp:76-82
const kTranslate_Shift: u32 = 0;

// Source: src/core/SkMatrix.cpp:84-84
const kScalar1Int: i32 = 0x3f800000;

// Source: src/core/SkMatrix.cpp:86-99
impl SkMatrix {
    fn computePerspectiveTypeMask(&self) -> u8 {
        if self.fMat[6] != 0.0 || self.fMat[7] != 0.0 || self.fMat[8] != 1.0 {
            return 15;
        }
        192
    }
}

// Source: src/core/SkMatrix.cpp:101-160
impl SkMatrix {
    fn computeTypeMask(&self) -> u8 {
        let mut mask = 0u8;
        if self.fMat[6] != 0.0 || self.fMat[7] != 0.0 || self.fMat[8] != 1.0 {
            return 15;
        }
        if self.fMat[2] != 0.0 || self.fMat[5] != 0.0 {
            mask |= 1;
        }
        let m00 = crate::src::core::SkFloatBits::SkScalarAs2sCompliment(self.fMat[0]);
        let m01 = crate::src::core::SkFloatBits::SkScalarAs2sCompliment(self.fMat[1]);
        let m10 = crate::src::core::SkFloatBits::SkScalarAs2sCompliment(self.fMat[3]);
        let m11 = crate::src::core::SkFloatBits::SkScalarAs2sCompliment(self.fMat[4]);
        if m01 | m10 != 0 {
            mask |= 6;
            let m01 = (m01 != 0) as u8;
            let m10 = (m10 != 0) as u8;
            let dp0 = (m00 | m11 == 0) as u8;
            let ds1 = m01 & m10;
            mask |= (dp0 & ds1) << kRectStaysRect_Shift;
        } else {
            if (m00 ^ kScalar1Int) | (m11 ^ kScalar1Int) != 0 {
                mask |= 2;
            }
            let m00 = (m00 != 0) as u8;
            let m11 = (m11 != 0) as u8;
            mask |= (m00 & m11) << kRectStaysRect_Shift;
        }
        mask
    }
}

// Source: src/core/SkMatrix.cpp:164-171
impl PartialEq for SkMatrix {
    fn eq(&self, b: &Self) -> bool {
        self.fMat == b.fMat
    }

    // Source: include/core/SkMatrix.h:1596-1598
    #[inline]
    fn ne(&self, b: &Self) -> bool {
        !self.eq(b)
    }
}

// Source: src/core/SkMatrix.cpp:176-180
fn is_degenerate_2x2(sx: f32, kx: f32, ky: f32, sy: f32) -> bool {
    let dot = sx * sy - kx * ky;
    let tol = crate::include::core::SkScalar::SK_ScalarNearlyZero;
    crate::include::core::SkScalar::SkScalarNearlyZero(dot, tol * tol)
}

// Source: src/core/SkMatrix.cpp:184-211
impl SkMatrix {
    pub fn isSimilarity(&self, tol: f32) -> bool {
        use crate::include::core::SkScalar::{
            SK_ScalarNearlyZero, SkScalarNearlyEqual, SkScalarNearlyZero,
        };
        let mask = self.getType();
        if mask <= 1 {
            return true;
        }
        if mask & 8 != 0 {
            return false;
        }
        let mx = self.fMat[0];
        let my = self.fMat[4];
        if mask & 4 == 0 {
            return !SkScalarNearlyZero(mx, SK_ScalarNearlyZero)
                && SkScalarNearlyEqual(mx.abs(), my.abs(), SK_ScalarNearlyZero);
        }
        let sx = self.fMat[1];
        let sy = self.fMat[3];
        if is_degenerate_2x2(mx, sx, sy, my) {
            return false;
        }
        (SkScalarNearlyEqual(mx, my, tol) && SkScalarNearlyEqual(sx, -sy, tol))
            || (SkScalarNearlyEqual(mx, -my, tol) && SkScalarNearlyEqual(sx, sy, tol))
    }
}

// Source: src/core/SkMatrix.cpp:213-241
impl SkMatrix {
    pub fn preservesRightAngles(&self, tol: f32) -> bool {
        let mask = self.getType();
        if mask <= 1 {
            return true;
        }
        if mask & 8 != 0 {
            return false;
        }
        debug_assert!(mask & (4 | 2) != 0);
        let mx = self.fMat[0];
        let my = self.fMat[4];
        let sx = self.fMat[1];
        let sy = self.fMat[3];
        if is_degenerate_2x2(mx, sx, sy, my) {
            return false;
        }
        let vec = [
            crate::src::core::SkPoint::SkPoint::make(mx, sy),
            crate::src::core::SkPoint::SkPoint::make(sx, my),
        ];
        crate::include::core::SkScalar::SkScalarNearlyZero(vec[0].dot(vec[1]), tol * tol)
    }
}

// Source: src/core/SkMatrix.cpp:245-247
#[inline]
fn sdot(a: f32, b: f32, c: f32, d: f32) -> f32 {
    a * b + c * d
}

// Source: src/core/SkMatrix.cpp:249-252
#[inline]
fn sdot3(a: f32, b: f32, c: f32, d: f32, e: f32, f: f32) -> f32 {
    a * b + c * d + e * f
}

// Source: src/core/SkMatrix.cpp:254-256
#[inline]
fn scross(a: f32, b: f32, c: f32, d: f32) -> f32 {
    a * b - c * d
}

// Source: src/core/SkMatrix.cpp:258-265
impl SkMatrix {
    pub fn setTranslate(&mut self, dx: f32, dy: f32) -> &mut Self {
        *self = Self::with_mask(
            [1.0, 0.0, dx, 0.0, 1.0, dy, 0.0, 0.0, 1.0],
            if dx != 0.0 || dy != 0.0 { 17 } else { 16 },
        );
        self
    }
}

// Source: src/core/SkMatrix.cpp:267-283
impl SkMatrix {
    pub fn preTranslate(&mut self, dx: f32, dy: f32) -> &mut Self {
        let mask = self.getType();
        if mask <= 1 {
            self.fMat[2] += dx;
            self.fMat[5] += dy;
        } else if mask & 8 != 0 {
            let mut m = Self::default();
            m.setTranslate(dx, dy);
            return self.preConcat(&m);
        } else {
            self.fMat[2] += self.fMat[0] * dx + self.fMat[1] * dy;
            self.fMat[5] += self.fMat[3] * dx + self.fMat[4] * dy;
        }
        self.updateTranslateMask();
        self
    }
}

// Source: src/core/SkMatrix.cpp:285-296
impl SkMatrix {
    pub fn postTranslate(&mut self, dx: f32, dy: f32) -> &mut Self {
        if self.hasPerspective() {
            let mut m = Self::default();
            m.setTranslate(dx, dy);
            self.postConcat(&m);
        } else {
            self.fMat[2] += dx;
            self.fMat[5] += dy;
            self.updateTranslateMask();
        }
        self
    }
}

// Source: src/core/SkMatrix.cpp:300-307
impl SkMatrix {
    pub fn setScalePivot(&mut self, sx: f32, sy: f32, px: f32, py: f32) -> &mut Self {
        if sx == 1.0 && sy == 1.0 {
            self.reset();
        } else {
            self.setScaleTranslate(sx, sy, px - sx * px, py - sy * py);
        }
        self
    }
}

// Source: src/core/SkMatrix.cpp:309-317
impl SkMatrix {
    pub fn setScale(&mut self, sx: f32, sy: f32) -> &mut Self {
        let rect_mask = if sx == 0.0 || sy == 0.0 { 0 } else { 16 };
        *self = Self::with_mask(
            [sx, 0.0, 0.0, 0.0, sy, 0.0, 0.0, 0.0, 1.0],
            if sx == 1.0 && sy == 1.0 {
                rect_mask
            } else {
                2 | rect_mask
            },
        );
        self
    }
}

// Source: src/core/SkMatrix.cpp:319-327
impl SkMatrix {
    pub fn preScalePivot(&mut self, sx: f32, sy: f32, px: f32, py: f32) -> &mut Self {
        if sx == 1.0 && sy == 1.0 {
            return self;
        }
        let mut m = Self::default();
        m.setScalePivot(sx, sy, px, py);
        self.preConcat(&m)
    }
}

// Source: src/core/SkMatrix.cpp:329-362
impl SkMatrix {
    pub fn preScale(&mut self, sx: f32, sy: f32) -> &mut Self {
        if sx == 1.0 && sy == 1.0 {
            return self;
        }
        self.fMat[0] *= sx;
        self.fMat[3] *= sx;
        self.fMat[6] *= sx;
        self.fMat[1] *= sy;
        self.fMat[4] *= sy;
        self.fMat[7] *= sy;
        if self.fMat[0] == 1.0 && self.fMat[4] == 1.0 && self.fTypeMask.get() & (8 | 4) == 0 {
            self.clearTypeMask(2);
        } else {
            self.orTypeMask(2);
            if sx == 0.0 || sy == 0.0 {
                self.clearTypeMask(16);
            }
        }
        self
    }
}

// Source: src/core/SkMatrix.cpp:364-371
impl SkMatrix {
    pub fn postScalePivot(&mut self, sx: f32, sy: f32, px: f32, py: f32) -> &mut Self {
        if sx == 1.0 && sy == 1.0 {
            return self;
        }
        let mut m = Self::default();
        m.setScalePivot(sx, sy, px, py);
        self.postConcat(&m)
    }
}

// Source: src/core/SkMatrix.cpp:373-380
impl SkMatrix {
    pub fn postScale(&mut self, sx: f32, sy: f32) -> &mut Self {
        if sx == 1.0 && sy == 1.0 {
            return self;
        }
        let mut m = Self::default();
        m.setScale(sx, sy);
        self.postConcat(&m)
    }
}

// Source: src/core/SkMatrix.cpp:384-402
impl SkMatrix {
    pub fn postIDiv(&mut self, divx: i32, divy: i32) -> bool {
        if divx == 0 || divy == 0 {
            return false;
        }
        let invx = 1.0 / divx as f32;
        let invy = 1.0 / divy as f32;
        self.fMat[0] *= invx;
        self.fMat[1] *= invx;
        self.fMat[2] *= invx;
        self.fMat[3] *= invy;
        self.fMat[4] *= invy;
        self.fMat[5] *= invy;
        self.setTypeMask(128);
        true
    }
}

// Source: src/core/SkMatrix.cpp:406-422
impl SkMatrix {
    pub fn setSinCosPivot(&mut self, sin_v: f32, cos_v: f32, px: f32, py: f32) -> &mut Self {
        let one_minus_cos_v = 1.0 - cos_v;
        self.fMat = [
            cos_v,
            -sin_v,
            sin_v * py + one_minus_cos_v * px,
            sin_v,
            cos_v,
            -sin_v * px + one_minus_cos_v * py,
            0.0,
            0.0,
            1.0,
        ];
        self.setTypeMask(Self::kUnknown_Mask | Self::kOnlyPerspectiveValid_Mask);
        self
    }
}

// Source: src/core/SkMatrix.cpp:424-438
impl SkMatrix {
    pub fn setRSXform(&mut self, xform: &crate::src::core::SkRSXform::SkRSXform) -> &mut Self {
        self.fMat[0] = xform.fSCos;
        self.fMat[1] = -xform.fSSin;
        self.fMat[2] = xform.fTx;
        self.fMat[3] = xform.fSSin;
        self.fMat[4] = xform.fSCos;
        self.fMat[5] = xform.fTy;
        self.fMat[6] = 0.0;
        self.fMat[7] = 0.0;
        self.fMat[8] = 1.0;
        self.setTypeMask(Self::kUnknown_Mask | Self::kOnlyPerspectiveValid_Mask);
        self
    }
}

// Source: src/core/SkMatrix.cpp:440-454
impl SkMatrix {
    pub fn setSinCos(&mut self, sin_v: f32, cos_v: f32) -> &mut Self {
        self.fMat = [cos_v, -sin_v, 0.0, sin_v, cos_v, 0.0, 0.0, 0.0, 1.0];
        self.setTypeMask(Self::kUnknown_Mask | Self::kOnlyPerspectiveValid_Mask);
        self
    }
}

// Source: src/core/SkMatrix.cpp:456-459
impl SkMatrix {
    pub fn setRotatePivot(&mut self, degrees: f32, px: f32, py: f32) -> &mut Self {
        let rad = crate::include::core::SkScalar::SkDegreesToRadians(degrees);
        self.setSinCosPivot(
            crate::include::core::SkScalar::SkScalarSinSnapToZero(rad),
            crate::include::core::SkScalar::SkScalarCosSnapToZero(rad),
            px,
            py,
        )
    }
}

// Source: src/core/SkMatrix.cpp:461-464
impl SkMatrix {
    pub fn setRotate(&mut self, degrees: f32) -> &mut Self {
        let rad = crate::include::core::SkScalar::SkDegreesToRadians(degrees);
        self.setSinCos(
            crate::include::core::SkScalar::SkScalarSinSnapToZero(rad),
            crate::include::core::SkScalar::SkScalarCosSnapToZero(rad),
        )
    }
}

// Source: src/core/SkMatrix.cpp:466-470
impl SkMatrix {
    pub fn preRotatePivot(&mut self, degrees: f32, px: f32, py: f32) -> &mut Self {
        let mut m = Self::default();
        m.setRotatePivot(degrees, px, py);
        self.preConcat(&m)
    }
}

// Source: src/core/SkMatrix.cpp:472-476
impl SkMatrix {
    pub fn preRotate(&mut self, degrees: f32) -> &mut Self {
        let mut m = Self::default();
        m.setRotate(degrees);
        self.preConcat(&m)
    }
}

// Source: src/core/SkMatrix.cpp:478-482
impl SkMatrix {
    pub fn postRotatePivot(&mut self, degrees: f32, px: f32, py: f32) -> &mut Self {
        let mut m = Self::default();
        m.setRotatePivot(degrees, px, py);
        self.postConcat(&m)
    }
}

// Source: src/core/SkMatrix.cpp:484-488
impl SkMatrix {
    pub fn postRotate(&mut self, degrees: f32) -> &mut Self {
        let mut m = Self::default();
        m.setRotate(degrees);
        self.postConcat(&m)
    }
}

// Source: src/core/SkMatrix.cpp:492-498
impl SkMatrix {
    pub fn setSkewPivot(&mut self, sx: f32, sy: f32, px: f32, py: f32) -> &mut Self {
        *self = Self::with_mask(
            [1.0, sx, -sx * py, sy, 1.0, -sy * px, 0.0, 0.0, 1.0],
            128 | 64,
        );
        self
    }
}

// Source: src/core/SkMatrix.cpp:500-514
impl SkMatrix {
    pub fn setSkew(&mut self, sx: f32, sy: f32) -> &mut Self {
        self.fMat = [1.0, sx, 0.0, sy, 1.0, 0.0, 0.0, 0.0, 1.0];
        self.setTypeMask(128 | 64);
        self
    }
}

// Source: src/core/SkMatrix.cpp:516-520
impl SkMatrix {
    pub fn preSkewPivot(&mut self, sx: f32, sy: f32, px: f32, py: f32) -> &mut Self {
        let mut m = Self::default();
        m.setSkewPivot(sx, sy, px, py);
        self.preConcat(&m)
    }
}

// Source: src/core/SkMatrix.cpp:522-526
impl SkMatrix {
    pub fn preSkew(&mut self, sx: f32, sy: f32) -> &mut Self {
        let mut m = Self::default();
        m.setSkew(sx, sy);
        self.preConcat(&m)
    }
}

// Source: src/core/SkMatrix.cpp:528-532
impl SkMatrix {
    pub fn postSkewPivot(&mut self, sx: f32, sy: f32, px: f32, py: f32) -> &mut Self {
        let mut m = Self::default();
        m.setSkewPivot(sx, sy, px, py);
        self.postConcat(&m)
    }
}

// Source: src/core/SkMatrix.cpp:534-538
impl SkMatrix {
    pub fn postSkew(&mut self, sx: f32, sy: f32) -> &mut Self {
        let mut m = Self::default();
        m.setSkew(sx, sy);
        self.postConcat(&m)
    }
}

// Source: src/core/SkMatrix.cpp:542-557
impl SkMatrix {
    pub fn ScaleTranslate(sx: f32, sy: f32, tx: f32, ty: f32) -> Self {
        let mut mask = 0;
        if sx != 1.0 || sy != 1.0 {
            mask |= 2;
        }
        if tx != 0.0 || ty != 0.0 {
            mask |= 1;
        }
        if sx != 0.0 && sy != 0.0 {
            mask |= 16;
        }
        Self::with_mask([sx, 0.0, tx, 0.0, sy, ty, 0.0, 0.0, 1.0], mask)
    }
}

// Source: src/core/SkMatrix.cpp:559-599
impl SkMatrix {
    pub fn Rect2Rect(
        src: &crate::src::core::SkRect::SkRect,
        dst: &crate::src::core::SkRect::SkRect,
        fit: ScaleToFit,
    ) -> Option<Self> {
        if src.isEmpty() {
            return None;
        }
        let mut sx = dst.width() / src.width();
        let mut sy = dst.height() / src.height();
        let mut x_larger = false;
        if fit != ScaleToFit::kFill_ScaleToFit {
            if sx > sy {
                x_larger = true;
                sx = sy;
            } else {
                sy = sx;
            }
        }
        let mut tx = dst.fLeft - src.fLeft * sx;
        let mut ty = dst.fTop - src.fTop * sy;
        if fit == ScaleToFit::kCenter_ScaleToFit || fit == ScaleToFit::kEnd_ScaleToFit {
            let mut diff = if x_larger {
                dst.width() - src.width() * sy
            } else {
                dst.height() - src.height() * sy
            };
            if fit == ScaleToFit::kCenter_ScaleToFit {
                diff /= 2.0;
            }
            if x_larger {
                tx += diff;
            } else {
                ty += diff;
            }
        }
        Some(Self::ScaleTranslate(sx, sy, tx, ty))
    }
}

// Source: src/core/SkMatrix.cpp:603-605
fn muladdmul(a: f32, b: f32, c: f32, d: f32) -> f32 {
    (a as f64 * b as f64 + c as f64 * d as f64) as f32
}

// Source: src/core/SkMatrix.cpp:607-609
fn rowcol3(row: &[f32], col: &[f32]) -> f32 {
    row[0] * col[0] + row[1] * col[3] + row[2] * col[6]
}

// Source: src/core/SkMatrix.cpp:611-613
fn only_scale_and_translate(mask: i32) -> bool {
    mask & 12 == 0
}

// Source: src/core/SkMatrix.cpp:615-684
impl SkMatrix {
    pub fn setConcat(&mut self, a: &Self, b: &Self) -> &mut Self {
        let a_type = a.getType();
        let b_type = b.getType();
        if a.isTriviallyIdentity() {
            *self = b.clone();
        } else if b.isTriviallyIdentity() {
            *self = a.clone();
        } else if only_scale_and_translate(a_type | b_type) {
            self.setScaleTranslate(
                a.fMat[0] * b.fMat[0],
                a.fMat[4] * b.fMat[4],
                a.fMat[0] * b.fMat[2] + a.fMat[2],
                a.fMat[4] * b.fMat[5] + a.fMat[5],
            );
        } else {
            let mut tmp = Self::default();
            if (a_type | b_type) & 8 != 0 {
                for r in 0..3 {
                    for c in 0..3 {
                        tmp.fMat[r * 3 + c] = rowcol3(&a.fMat[r * 3..], &b.fMat[c..]);
                    }
                }
                tmp.setTypeMask(128);
            } else {
                tmp.fMat[0] = muladdmul(a.fMat[0], b.fMat[0], a.fMat[1], b.fMat[3]);
                tmp.fMat[1] = muladdmul(a.fMat[0], b.fMat[1], a.fMat[1], b.fMat[4]);
                tmp.fMat[2] = muladdmul(a.fMat[0], b.fMat[2], a.fMat[1], b.fMat[5]) + a.fMat[2];
                tmp.fMat[3] = muladdmul(a.fMat[3], b.fMat[0], a.fMat[4], b.fMat[3]);
                tmp.fMat[4] = muladdmul(a.fMat[3], b.fMat[1], a.fMat[4], b.fMat[4]);
                tmp.fMat[5] = muladdmul(a.fMat[3], b.fMat[2], a.fMat[4], b.fMat[5]) + a.fMat[5];
                tmp.fMat[6] = 0.0;
                tmp.fMat[7] = 0.0;
                tmp.fMat[8] = 1.0;
                tmp.setTypeMask(Self::kUnknown_Mask | Self::kOnlyPerspectiveValid_Mask);
            }
            *self = tmp;
        }
        self
    }
}

// Source: src/core/SkMatrix.cpp:686-693
impl SkMatrix {
    pub fn preConcat(&mut self, mat: &Self) -> &mut Self {
        if !mat.isIdentity() {
            let a = self.clone();
            self.setConcat(&a, mat);
        }
        self
    }
}

// Source: src/core/SkMatrix.cpp:695-702
impl SkMatrix {
    pub fn postConcat(&mut self, mat: &Self) -> &mut Self {
        if !mat.isIdentity() {
            let b = self.clone();
            self.setConcat(mat, &b);
        }
        self
    }
}

// Source: src/core/SkMatrix.cpp:712-715
fn scross_dscale(a: f32, b: f32, c: f32, d: f32, scale: f64) -> f32 {
    (scross(a, b, c, d) as f64 * scale) as f32
}

// Source: src/core/SkMatrix.cpp:717-719
fn dcross(a: f64, b: f64, c: f64, d: f64) -> f64 {
    a * b - c * d
}

// Source: src/core/SkMatrix.cpp:721-724
fn dcross_dscale(a: f64, b: f64, c: f64, d: f64, scale: f64) -> f32 {
    (dcross(a, b, c, d) * scale) as f32
}

// Source: src/core/SkMatrix.cpp:726-743
fn sk_determinant(m: &[f32; 9], is_perspective: bool) -> f64 {
    if is_perspective {
        m[0] as f64 * dcross(m[4] as f64, m[8] as f64, m[5] as f64, m[7] as f64)
            + m[1] as f64 * dcross(m[5] as f64, m[6] as f64, m[3] as f64, m[8] as f64)
            + m[2] as f64 * dcross(m[3] as f64, m[7] as f64, m[4] as f64, m[6] as f64)
    } else {
        dcross(m[0] as f64, m[4] as f64, m[1] as f64, m[3] as f64)
    }
}

// Source: src/core/SkMatrix.cpp:745-756
fn sk_inv_determinant(m: &[f32; 9], is_perspective: bool) -> f64 {
    let det = sk_determinant(m, is_perspective);
    let near = crate::include::core::SkScalar::SK_ScalarNearlyZero;
    if crate::include::core::SkScalar::SkScalarNearlyZero(det as f32, near * near * near) {
        return 0.0;
    }
    1.0 / det
}

// Source: src/core/SkMatrix.cpp:758-765
impl SkMatrix {
    pub fn SetAffineIdentity(a: &mut [f32; 6]) {
        *a = [1.0, 0.0, 0.0, 1.0, 0.0, 0.0];
    }
}

// Source: src/core/SkMatrix.cpp:767-780
impl SkMatrix {
    pub fn asAffine(&self, affine: Option<&mut [f32; 6]>) -> bool {
        if self.hasPerspective() {
            return false;
        }
        if let Some(a) = affine {
            *a = [
                self.fMat[0],
                self.fMat[3],
                self.fMat[1],
                self.fMat[4],
                self.fMat[2],
                self.fMat[5],
            ];
        }
        true
    }
}

// Source: src/core/SkMatrix.cpp:782-785
impl SkMatrix {
    pub fn mapPoints(
        &self,
        dst: &mut [crate::src::core::SkPoint::SkPoint],
        src: &[crate::src::core::SkPoint::SkPoint],
    ) {
        let count = min_count(dst, src) as usize;
        unsafe {
            (self.getMapPtsProc())(self, dst.as_mut_ptr(), src.as_ptr(), count);
        }
    }
}

// Source: src/core/SkMatrix.cpp:787-816
impl SkMatrix {
    fn ComputeInv(dst: &mut [f32; 9], src: &[f32; 9], inv_det: f64, is_persp: bool) {
        if is_persp {
            dst[0] = scross_dscale(src[4], src[8], src[5], src[7], inv_det);
            dst[1] = scross_dscale(src[2], src[7], src[1], src[8], inv_det);
            dst[2] = scross_dscale(src[1], src[5], src[2], src[4], inv_det);
            dst[3] = scross_dscale(src[5], src[6], src[3], src[8], inv_det);
            dst[4] = scross_dscale(src[0], src[8], src[2], src[6], inv_det);
            dst[5] = scross_dscale(src[2], src[3], src[0], src[5], inv_det);
            dst[6] = scross_dscale(src[3], src[7], src[4], src[6], inv_det);
            dst[7] = scross_dscale(src[1], src[6], src[0], src[7], inv_det);
            dst[8] = scross_dscale(src[0], src[4], src[1], src[3], inv_det);
        } else {
            dst[0] = (src[4] as f64 * inv_det) as f32;
            dst[1] = (-src[1] as f64 * inv_det) as f32;
            dst[2] = dcross_dscale(
                src[1] as f64,
                src[5] as f64,
                src[4] as f64,
                src[2] as f64,
                inv_det,
            );
            dst[3] = (-src[3] as f64 * inv_det) as f32;
            dst[4] = (src[0] as f64 * inv_det) as f32;
            dst[5] = dcross_dscale(
                src[3] as f64,
                src[2] as f64,
                src[0] as f64,
                src[5] as f64,
                inv_det,
            );
            dst[6] = 0.0;
            dst[7] = 0.0;
            dst[8] = 1.0;
        }
    }
}

// Source: src/core/SkMatrix.cpp:818-881
impl SkMatrix {
    pub fn invert(&self) -> Option<Self> {
        let mask = self.getType();
        if mask == 0 {
            return Some(self.clone());
        }
        if mask & !3 == 0 {
            if mask & 2 != 0 {
                let inv_sx = 1.0 / self.fMat[0];
                let inv_sy = 1.0 / self.fMat[4];
                if !inv_sx.is_finite() || !inv_sy.is_finite() {
                    return None;
                }
                let inv_tx = -self.fMat[2] * inv_sx;
                let inv_ty = -self.fMat[5] * inv_sy;
                if !inv_tx.is_finite() || !inv_ty.is_finite() {
                    return None;
                }
                return Some(Self::with_mask(
                    [inv_sx, 0.0, inv_tx, 0.0, inv_sy, inv_ty, 0.0, 0.0, 1.0],
                    mask | 16,
                ));
            }
            if !self.fMat[2].is_finite() || !self.fMat[5].is_finite() {
                return None;
            }
            return Some(Self::Translate(-self.fMat[2], -self.fMat[5]));
        }
        let is_persp = mask & 8 != 0;
        let inv_det = sk_inv_determinant(&self.fMat, is_persp);
        if inv_det == 0.0 {
            return None;
        }
        let mut inv = Self::default();
        Self::ComputeInv(&mut inv.fMat, &self.fMat, inv_det, is_persp);
        if !inv.isFinite() {
            return None;
        }
        inv.setTypeMask(self.fTypeMask.get());
        Some(inv)
    }
}

// Source: src/core/SkMatrix.cpp:883-891
impl SkMatrix {
    pub fn mapPointPerspective(
        &self,
        p: crate::src::core::SkPoint::SkPoint,
    ) -> crate::src::core::SkPoint::SkPoint {
        let x = sdot(p.fX, self.fMat[0], p.fY, self.fMat[1]) + self.fMat[2];
        let y = sdot(p.fX, self.fMat[3], p.fY, self.fMat[4]) + self.fMat[5];
        let mut z = sdot(p.fX, self.fMat[6], p.fY, self.fMat[7]) + self.fMat[8];
        if z != 0.0 {
            z = 1.0 / z;
        }
        crate::src::core::SkPoint::SkPoint::make(x * z, y * z)
    }
}

// Source: src/core/SkMatrix.cpp:895-901
impl SkMatrix {
    unsafe fn Identity_pts(
        m: &Self,
        mut dst: *mut crate::src::core::SkPoint::SkPoint,
        mut src: *const crate::src::core::SkPoint::SkPoint,
        mut count: usize,
    ) {
        unsafe {
            crate::SkASSERT!(@source b"src/core/SkMatrix.cpp\0".as_ptr(),896; @condition b"m.getType() == 0\0".as_ptr(); (m.getType())==(0));
            if dst.cast_const() != src && count > 0 {
                core::ptr::copy_nonoverlapping(src, dst, count);
            }
        }
    }
}

// Source: src/core/SkMatrix.cpp:903-929
impl SkMatrix {
    unsafe fn Trans_pts(
        m: &Self,
        mut dst: *mut crate::src::core::SkPoint::SkPoint,
        mut src: *const crate::src::core::SkPoint::SkPoint,
        mut count: usize,
    ) {
        unsafe {
            debug_assert!(m.getType() <= 1);
            if count > 0 {
                let tx = m.getTranslateX();
                let ty = m.getTranslateY();
                if count & 1 != 0 {
                    let p = src.read();
                    (*dst).fX = p.fX + tx;
                    (*dst).fY = p.fY + ty;
                    src = src.add(1);
                    dst = dst.add(1);
                }
                count >>= 1;
                if count & 1 != 0 {
                    for lane in 0..2 {
                        let p = src.add(lane).read();
                        dst.add(lane)
                            .write(crate::src::core::SkPoint::SkPoint::make(
                                p.fX + tx,
                                p.fY + ty,
                            ));
                    }
                    src = src.add(2);
                    dst = dst.add(2);
                }
                count >>= 1;
                for _ in 0..count {
                    for lane in 0..4 {
                        let p = src.add(lane).read();
                        dst.add(lane)
                            .write(crate::src::core::SkPoint::SkPoint::make(
                                p.fX + tx,
                                p.fY + ty,
                            ));
                    }
                    src = src.add(4);
                    dst = dst.add(4);
                }
            }
        }
    }
}

// Source: src/core/SkMatrix.cpp:931-962
impl SkMatrix {
    unsafe fn Scale_pts(
        m: &Self,
        mut dst: *mut crate::src::core::SkPoint::SkPoint,
        mut src: *const crate::src::core::SkPoint::SkPoint,
        mut count: usize,
    ) {
        unsafe {
            debug_assert!(m.getType() <= 3);
            if count > 0 {
                let tx = m.getTranslateX();
                let ty = m.getTranslateY();
                let sx = m.getScaleX();
                let sy = m.getScaleY();
                if count & 1 != 0 {
                    let p = src.read();
                    (*dst).fX = p.fX * sx + tx;
                    (*dst).fY = p.fY * sy + ty;
                    src = src.add(1);
                    dst = dst.add(1);
                }
                count >>= 1;
                if count & 1 != 0 {
                    for lane in 0..2 {
                        let p = src.add(lane).read();
                        dst.add(lane)
                            .write(crate::src::core::SkPoint::SkPoint::make(
                                p.fX * sx + tx,
                                p.fY * sy + ty,
                            ));
                    }
                    src = src.add(2);
                    dst = dst.add(2);
                }
                count >>= 1;
                for _ in 0..count {
                    for lane in 0..4 {
                        let p = src.add(lane).read();
                        dst.add(lane)
                            .write(crate::src::core::SkPoint::SkPoint::make(
                                p.fX * sx + tx,
                                p.fY * sy + ty,
                            ));
                    }
                    src = src.add(4);
                    dst = dst.add(4);
                }
            }
        }
    }
}

// Source: src/core/SkMatrix.cpp:964-986
impl SkMatrix {
    unsafe fn Persp_pts(
        m: &Self,
        mut dst: *mut crate::src::core::SkPoint::SkPoint,
        mut src: *const crate::src::core::SkPoint::SkPoint,
        mut count: usize,
    ) {
        unsafe {
            crate::SkASSERT!(@source b"src/core/SkMatrix.cpp\0".as_ptr(),966; @condition b"m.hasPerspective()\0".as_ptr(); m.hasPerspective());
            if count > 0 {
                loop {
                    let sy = (*src).fY;
                    let sx = (*src).fX;
                    src = src.add(1);
                    let x = sdot(sx, m.fMat[0], sy, m.fMat[1]) + m.fMat[2];
                    let y = sdot(sx, m.fMat[3], sy, m.fMat[4]) + m.fMat[5];
                    let mut z = sdot(sx, m.fMat[6], sy, m.fMat[7]) + m.fMat[8];
                    if z != 0.0 {
                        z = 1.0 / z;
                    }
                    (*dst).fY = y * z;
                    (*dst).fX = x * z;
                    dst = dst.add(1);
                    count -= 1;
                    if count == 0 {
                        break;
                    }
                }
            }
        }
    }
}

// Source: src/core/SkMatrix.cpp:988-1018
impl SkMatrix {
    unsafe fn Affine_vpts(
        m: &Self,
        mut dst: *mut crate::src::core::SkPoint::SkPoint,
        mut src: *const crate::src::core::SkPoint::SkPoint,
        mut count: usize,
    ) {
        unsafe {
            debug_assert!(m.getType() != 8);
            if count > 0 {
                let tx = m.getTranslateX();
                let ty = m.getTranslateY();
                let sx = m.getScaleX();
                let sy = m.getScaleY();
                let kx = m.getSkewX();
                let ky = m.getSkewY();
                let trailing_element = count & 1 != 0;
                count >>= 1;
                for _ in 0..count {
                    let p0 = src.read();
                    let p1 = src.add(1).read();
                    dst.write(crate::src::core::SkPoint::SkPoint::make(
                        p0.fX * sx + p0.fY * kx + tx,
                        p0.fY * sy + p0.fX * ky + ty,
                    ));
                    dst.add(1).write(crate::src::core::SkPoint::SkPoint::make(
                        p1.fX * sx + p1.fY * kx + tx,
                        p1.fY * sy + p1.fX * ky + ty,
                    ));
                    src = src.add(2);
                    dst = dst.add(2);
                }
                if trailing_element {
                    let p = src.read();
                    dst.write(crate::src::core::SkPoint::SkPoint::make(
                        p.fX * sx + p.fY * kx + tx,
                        p.fY * sy + p.fX * ky + ty,
                    ));
                }
            }
        }
    }
}

// Source: src/core/SkMatrix.cpp:1020-1030
impl SkMatrix {
    const gMapPtsProcs: [MapPtsProc; 16] = [
        Self::Identity_pts,
        Self::Trans_pts,
        Self::Scale_pts,
        Self::Scale_pts,
        Self::Affine_vpts,
        Self::Affine_vpts,
        Self::Affine_vpts,
        Self::Affine_vpts,
        Self::Persp_pts,
        Self::Persp_pts,
        Self::Persp_pts,
        Self::Persp_pts,
        Self::Persp_pts,
        Self::Persp_pts,
        Self::Persp_pts,
        Self::Persp_pts,
    ];
}

// Source: src/core/SkMatrix.cpp:1034-1072
impl crate::src::core::SkMatrixPriv::SkMatrixPriv {
    // Strided source/destination must be valid for count records; overlap is identical-base only.
    pub unsafe fn MapHomogeneousPointsWithStride(
        mx: &SkMatrix,
        mut dst: *mut crate::src::core::SkPoint3::SkPoint3,
        dst_stride: usize,
        mut src: *const crate::src::core::SkPoint3::SkPoint3,
        src_stride: usize,
        count: usize,
    ) {
        debug_assert!((!dst.is_null() && !src.is_null() && count > 0) || count == 0);
        if count > 0 {
            if mx.isIdentity() {
                if !core::ptr::eq(src, dst) {
                    if src_stride == core::mem::size_of::<crate::src::core::SkPoint3::SkPoint3>()
                        && dst_stride == src_stride
                    {
                        unsafe {
                            core::ptr::copy_nonoverlapping(src, dst, count);
                        }
                    } else {
                        for _ in 0..count {
                            unsafe {
                                dst.write(src.read());
                                dst = dst.wrapping_byte_add(dst_stride);
                                src = src.wrapping_byte_add(src_stride);
                            }
                        }
                    }
                }
                return;
            }
            for _ in 0..count {
                let p = unsafe { src.read() };
                src = unsafe { src.wrapping_byte_add(src_stride) };
                let mat = &mx.fMat;
                let x = sdot3(p.fX, mat[0], p.fY, mat[1], p.fZ, mat[2]);
                let y = sdot3(p.fX, mat[3], p.fY, mat[4], p.fZ, mat[5]);
                let w = sdot3(p.fX, mat[6], p.fY, mat[7], p.fZ, mat[8]);
                unsafe {
                    dst.write(crate::src::core::SkPoint3::SkPoint3::Make(x, y, w));
                    dst = dst.wrapping_byte_add(dst_stride);
                }
            }
        }
    }
}

// Source: src/core/SkMatrix.cpp:1074-1078
impl SkMatrix {
    pub fn mapHomogeneousPoints(
        &self,
        dst: &mut [crate::src::core::SkPoint3::SkPoint3],
        src: &[crate::src::core::SkPoint3::SkPoint3],
    ) {
        let count = min_count(dst, src) as usize;
        unsafe {
            crate::src::core::SkMatrixPriv::SkMatrixPriv::MapHomogeneousPointsWithStride(
                self,
                dst.as_mut_ptr(),
                core::mem::size_of::<crate::src::core::SkPoint3::SkPoint3>(),
                src.as_ptr(),
                core::mem::size_of::<crate::src::core::SkPoint3::SkPoint3>(),
                count,
            );
        }
    }
}

// Source: src/core/SkMatrix.cpp:1080-1104
impl SkMatrix {
    pub fn mapPointsToHomogeneous(
        &self,
        dst: &mut [crate::src::core::SkPoint3::SkPoint3],
        src: &[crate::src::core::SkPoint::SkPoint],
    ) {
        let count = min_count(dst, src) as usize;
        if self.isIdentity() {
            for i in 0..count {
                dst[i] = crate::src::core::SkPoint3::SkPoint3::Make(src[i].fX, src[i].fY, 1.0);
            }
        } else if self.hasPerspective() {
            for i in 0..count {
                dst[i] = crate::src::core::SkPoint3::SkPoint3::Make(
                    self.fMat[0] * src[i].fX + self.fMat[1] * src[i].fY + self.fMat[2],
                    self.fMat[3] * src[i].fX + self.fMat[4] * src[i].fY + self.fMat[5],
                    self.fMat[6] * src[i].fX + self.fMat[7] * src[i].fY + self.fMat[8],
                );
            }
        } else {
            for i in 0..count {
                dst[i] = crate::src::core::SkPoint3::SkPoint3::Make(
                    self.fMat[0] * src[i].fX + self.fMat[1] * src[i].fY + self.fMat[2],
                    self.fMat[3] * src[i].fX + self.fMat[4] * src[i].fY + self.fMat[5],
                    1.0,
                );
            }
        }
    }
}

// Source: src/core/SkMatrix.cpp:1108-1122
impl SkMatrix {
    pub fn mapVectors(&self, dst: &mut [SkPoint], src: &[SkPoint]) {
        unsafe {
            self.mapVectors_raw(dst.as_mut_ptr(), src.as_ptr(), min_count(dst, src) as usize);
        }
    }
    // Native pointer interface allows identical src/dst without manufacturing aliased Rust references.
    unsafe fn mapVectors_raw(&self, dst: *mut SkPoint, src: *const SkPoint, count: usize) {
        unsafe {
            if self.hasPerspective() {
                let origin = self.mapPointPerspective(SkPoint::make(0.0, 0.0));
                for i in (0..count).rev() {
                    dst.add(i)
                        .write(self.mapPointPerspective(src.add(i).read()) - origin);
                }
            } else {
                let mut tmp = self.clone();
                tmp.fMat[2] = 0.0;
                tmp.fMat[5] = 0.0;
                tmp.clearTypeMask(1);
                (tmp.getMapPtsProc())(&tmp, dst, src, count);
            }
        }
    }
}

// Source: src/core/SkMatrix.cpp:1124-1131
fn sort_as_rect(r: crate::src::core::SkRect::SkRect) -> crate::src::core::SkRect::SkRect {
    crate::src::core::SkRect::SkRect::MakeLTRB(
        if r.fLeft < r.fRight {
            r.fLeft
        } else {
            r.fRight
        },
        if r.fTop < r.fBottom {
            r.fTop
        } else {
            r.fBottom
        },
        if r.fLeft < r.fRight {
            r.fRight
        } else {
            r.fLeft
        },
        if r.fTop < r.fBottom {
            r.fBottom
        } else {
            r.fTop
        },
    )
}

// Source: src/core/SkMatrix.cpp:1133-1144
impl SkMatrix {
    pub fn mapRectScaleTranslate(
        &self,
        dst: &mut crate::src::core::SkRect::SkRect,
        src: &crate::src::core::SkRect::SkRect,
    ) {
        crate::SkASSERT!(@source b"src/core/SkMatrix.cpp\0".as_ptr(),1135; @condition b"this->isScaleTranslate()\0".as_ptr(); self.isScaleTranslate());
        let sx = self.fMat[0];
        let sy = self.fMat[4];
        let tx = self.fMat[2];
        let ty = self.fMat[5];
        *dst = sort_as_rect(crate::src::core::SkRect::SkRect::MakeLTRB(
            src.fLeft * sx + tx,
            src.fTop * sy + ty,
            src.fRight * sx + tx,
            src.fBottom * sy + ty,
        ));
    }
}

// Source: src/core/SkMatrix.cpp:1146-1171
impl SkMatrix {
    pub fn mapRect(
        &self,
        dst: &mut crate::src::core::SkRect::SkRect,
        src: &crate::src::core::SkRect::SkRect,
    ) -> bool {
        use crate::src::core::SkRect::SkRect;
        if self.getType() <= 1 {
            let tx = self.fMat[2];
            let ty = self.fMat[5];
            *dst = sort_as_rect(SkRect::MakeLTRB(
                src.fLeft + tx,
                src.fTop + ty,
                src.fRight + tx,
                src.fBottom + ty,
            ));
            return true;
        }
        if self.isScaleTranslate() {
            self.mapRectScaleTranslate(dst, src);
            true
        } else if self.hasPerspective() {
            let mut builder = crate::src::core::SkPathBuilder::SkPathBuilder::new();
            builder.addRect(
                src,
                crate::include::core::SkPathTypes::SkPathDirection::kCW,
                0,
            );
            builder.transform(self);
            *dst = builder.computeBounds();
            false
        } else {
            let mut quad = [
                crate::src::core::SkPoint::SkPoint::make(src.fLeft, src.fTop),
                crate::src::core::SkPoint::SkPoint::make(src.fRight, src.fTop),
                crate::src::core::SkPoint::SkPoint::make(src.fRight, src.fBottom),
                crate::src::core::SkPoint::SkPoint::make(src.fLeft, src.fBottom),
            ];
            self.mapPointsInPlace(&mut quad);
            dst.setBoundsNoCheck(&quad);
            self.rectStaysRect()
        }
    }
}

// Source: src/core/SkMatrix.cpp:1173-1185
impl SkMatrix {
    pub fn mapRadius(&self, radius: f32) -> f32 {
        let src = [
            crate::src::core::SkPoint::SkPoint::make(radius, 0.0),
            crate::src::core::SkPoint::SkPoint::make(0.0, radius),
        ];
        let mut vec = src;
        self.mapVectors(&mut vec, &src);
        let d0 = vec[0].length();
        let d1 = vec[1].length();
        (d0 * d1).sqrt()
    }
}

// Source: src/core/SkMatrix.cpp:1210-1212
fn checkForZero(x: f32) -> bool {
    x * x == 0.0
}

// Source: src/core/SkMatrix.cpp:1214-1228
impl SkMatrix {
    fn Poly2Proc(src: &[crate::src::core::SkPoint::SkPoint], dst: &mut Self) -> bool {
        dst.fMat[0] = src[1].fY - src[0].fY;
        dst.fMat[3] = src[0].fX - src[1].fX;
        dst.fMat[6] = 0.0;
        dst.fMat[1] = src[1].fX - src[0].fX;
        dst.fMat[4] = src[1].fY - src[0].fY;
        dst.fMat[7] = 0.0;
        dst.fMat[2] = src[0].fX;
        dst.fMat[5] = src[0].fY;
        dst.fMat[8] = 1.0;
        dst.setTypeMask(128);
        true
    }
}

// Source: src/core/SkMatrix.cpp:1230-1244
impl SkMatrix {
    fn Poly3Proc(src: &[crate::src::core::SkPoint::SkPoint], dst: &mut Self) -> bool {
        dst.fMat[0] = src[2].fX - src[0].fX;
        dst.fMat[3] = src[2].fY - src[0].fY;
        dst.fMat[6] = 0.0;
        dst.fMat[1] = src[1].fX - src[0].fX;
        dst.fMat[4] = src[1].fY - src[0].fY;
        dst.fMat[7] = 0.0;
        dst.fMat[2] = src[0].fX;
        dst.fMat[5] = src[0].fY;
        dst.fMat[8] = 1.0;
        dst.setTypeMask(128);
        true
    }
}

// Source: src/core/SkMatrix.cpp:1246-1300
impl SkMatrix {
    fn Poly4Proc(src: &[crate::src::core::SkPoint::SkPoint], dst: &mut Self) -> bool {
        let x0 = src[2].fX - src[0].fX;
        let y0 = src[2].fY - src[0].fY;
        let x1 = src[2].fX - src[1].fX;
        let y1 = src[2].fY - src[1].fY;
        let x2 = src[2].fX - src[3].fX;
        let y2 = src[2].fY - src[3].fY;
        let a1;
        if if x2 > 0.0 {
            if y2 > 0.0 {
                x2 > y2
            } else {
                x2 > -y2
            }
        } else if y2 > 0.0 {
            -x2 > y2
        } else {
            x2 < y2
        } {
            let denom = x1 * y2 / x2 - y1;
            if checkForZero(denom) {
                return false;
            }
            a1 = (((x0 - x1) * y2 / x2) - y0 + y1) / denom;
        } else {
            let denom = x1 - y1 * x2 / y2;
            if checkForZero(denom) {
                return false;
            }
            a1 = (x0 - x1 - (y0 - y1) * x2 / y2) / denom;
        }
        let a2;
        if if x1 > 0.0 {
            if y1 > 0.0 {
                x1 > y1
            } else {
                x1 > -y1
            }
        } else if y1 > 0.0 {
            -x1 > y1
        } else {
            x1 < y1
        } {
            let denom = y2 - x2 * y1 / x1;
            if checkForZero(denom) {
                return false;
            }
            a2 = (y0 - y2 - (x0 - x2) * y1 / x1) / denom;
        } else {
            let denom = y2 * x1 / y1 - x2;
            if checkForZero(denom) {
                return false;
            }
            a2 = ((y0 - y2) * x1 / y1 - x0 + x2) / denom;
        }
        dst.fMat[0] = a2 * src[3].fX + src[3].fX - src[0].fX;
        dst.fMat[3] = a2 * src[3].fY + src[3].fY - src[0].fY;
        dst.fMat[6] = a2;
        dst.fMat[1] = a1 * src[1].fX + src[1].fX - src[0].fX;
        dst.fMat[4] = a1 * src[1].fY + src[1].fY - src[0].fY;
        dst.fMat[7] = a1;
        dst.fMat[2] = src[0].fX;
        dst.fMat[5] = src[0].fY;
        dst.fMat[8] = 1.0;
        dst.setTypeMask(128);
        true
    }
}

// Source: src/core/SkMatrix.cpp:1306-1338
impl SkMatrix {
    pub fn PolyToPoly(
        src: &[crate::src::core::SkPoint::SkPoint],
        dst: &[crate::src::core::SkPoint::SkPoint],
    ) -> Option<Self> {
        if src.len() != dst.len() || src.len() > 4 {
            return None;
        }
        let procs: [fn(&[crate::src::core::SkPoint::SkPoint], &mut Self) -> bool; 3] =
            [Self::Poly2Proc, Self::Poly3Proc, Self::Poly4Proc];
        match src.len() {
            0 => Some(Self::default()),
            1 => {
                let d = dst[0] - src[0];
                Some(Self::Translate(d.fX, d.fY))
            }
            2..=4 => {
                let proc = procs[src.len() - 2];
                let mut temp = Self::default();
                if !proc(src, &mut temp) {
                    return None;
                }
                let inv = temp.invert()?;
                if !proc(dst, &mut temp) {
                    return None;
                }
                Some({
                    let mut result = Self::default();
                    result.setConcat(&temp, &inv);
                    result
                })
            }
            _ => crate::SkUNREACHABLE!(@source b"src/core/SkMatrix.cpp\0".as_ptr(),1337;),
        }
    }
}

// Source: src/core/SkMatrix.cpp:1342-1346
#[repr(u8)]
enum MinMaxOrBoth {
    kMin_MinMaxOrBoth,
    kMax_MinMaxOrBoth,
    kBoth_MinMaxOrBoth,
}

// Source: src/core/SkMatrix.cpp:1348-1440
fn get_scale_factor<const KIND: u8>(mask: i32, m: &[f32; 9], out: &mut [f32; 2]) -> bool {
    let min = MinMaxOrBoth::kMin_MinMaxOrBoth as u8;
    let max = MinMaxOrBoth::kMax_MinMaxOrBoth as u8;
    let both = MinMaxOrBoth::kBoth_MinMaxOrBoth as u8;
    let cmin = |a: f32, b: f32| if b < a { b } else { a };
    let cmax = |a: f32, b: f32| if a < b { b } else { a };
    if mask & 8 != 0 {
        return false;
    }
    if mask == 0 {
        out[0] = 1.0;
        if KIND == both {
            out[1] = 1.0;
        }
        return true;
    }
    if mask & 4 == 0 {
        if KIND == min {
            out[0] = cmin(m[0].abs(), m[4].abs());
        } else if KIND == max {
            out[0] = cmax(m[0].abs(), m[4].abs());
        } else {
            out[0] = m[0].abs();
            out[1] = m[4].abs();
            if out[0] > out[1] {
                out.swap(0, 1);
            }
        }
        return true;
    }
    let a = m[0] * m[0] + m[3] * m[3];
    let b = m[0] * m[1] + m[4] * m[3];
    let c = m[1] * m[1] + m[4] * m[4];
    let b_sqd = b * b;
    if b_sqd
        <= crate::include::core::SkScalar::SK_ScalarNearlyZero
            * crate::include::core::SkScalar::SK_ScalarNearlyZero
    {
        if KIND == min {
            out[0] = cmin(a, c);
        } else if KIND == max {
            out[0] = cmax(a, c);
        } else {
            out[0] = a;
            out[1] = c;
            if out[0] > out[1] {
                out.swap(0, 1);
            }
        }
    } else {
        let aminusc = a - c;
        let half = (a + c) / 2.0;
        let x = (aminusc * aminusc + 4.0 * b_sqd).sqrt() / 2.0;
        if KIND == min {
            out[0] = half - x;
        } else if KIND == max {
            out[0] = half + x;
        } else {
            out[0] = half - x;
            out[1] = half + x;
        }
    }
    if !out[0].is_finite() {
        return false;
    }
    if out[0] < 0.0 {
        out[0] = 0.0;
    }
    out[0] = out[0].sqrt();
    if KIND == both {
        if !out[1].is_finite() {
            return false;
        }
        if out[1] < 0.0 {
            out[1] = 0.0;
        }
        out[1] = out[1].sqrt();
    }
    true
}

// Source: src/core/SkMatrix.cpp:1442-1449
impl SkMatrix {
    pub fn getMinScale(&self) -> f32 {
        let mut factor = [0.0; 2];
        if get_scale_factor::<{ MinMaxOrBoth::kMin_MinMaxOrBoth as u8 }>(
            self.getType(),
            &self.fMat,
            &mut factor,
        ) {
            factor[0]
        } else {
            -1.0
        }
    }
}

// Source: src/core/SkMatrix.cpp:1451-1458
impl SkMatrix {
    pub fn getMaxScale(&self) -> f32 {
        let mut factor = [0.0; 2];
        if get_scale_factor::<{ MinMaxOrBoth::kMax_MinMaxOrBoth as u8 }>(
            self.getType(),
            &self.fMat,
            &mut factor,
        ) {
            factor[0]
        } else {
            -1.0
        }
    }
}

// Source: src/core/SkMatrix.cpp:1460-1462
impl SkMatrix {
    pub fn getMinMaxScales(&self, factors: &mut [f32; 2]) -> bool {
        get_scale_factor::<{ MinMaxOrBoth::kBoth_MinMaxOrBoth as u8 }>(
            self.getType(),
            &self.fMat,
            factors,
        )
    }
}

// Source: src/core/SkMatrix.cpp:1464-1468
impl SkMatrix {
    pub fn I() -> &'static Self {
        static ID: KnownMatrixConstant = KnownMatrixConstant(SkMatrix {
            fMat: [1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0],
            fTypeMask: std::cell::Cell::new(16),
        });
        let matrix = &ID.0;
        debug_assert!(matrix.isIdentity());
        matrix
    }
}

// Source: src/core/SkMatrix.cpp:1470-1477
impl SkMatrix {
    pub fn InvalidMatrix() -> &'static Self {
        static INVALID: KnownMatrixConstant = KnownMatrixConstant(SkMatrix {
            fMat: [f32::MAX; 9],
            fTypeMask: std::cell::Cell::new(1 | 2 | 4 | 8),
        });
        &INVALID.0
    }
}

// Source: src/core/SkMatrix.cpp:1479-1499
impl SkMatrix {
    pub fn decomposeScale(
        &self,
        scale: Option<&mut crate::include::core::SkSize::SkSize>,
        remaining: Option<&mut SkMatrix>,
    ) -> bool {
        if self.hasPerspective() {
            return false;
        }
        let sx = crate::src::core::SkPoint::SkPoint::length_xy(self.getScaleX(), self.getSkewY());
        let sy = crate::src::core::SkPoint::SkPoint::length_xy(self.getSkewX(), self.getScaleY());
        use crate::include::core::SkScalar::{SK_ScalarNearlyZero, SkScalarNearlyZero};
        if !sx.is_finite()
            || !sy.is_finite()
            || SkScalarNearlyZero(sx, SK_ScalarNearlyZero)
            || SkScalarNearlyZero(sy, SK_ScalarNearlyZero)
        {
            return false;
        }
        if let Some(scale) = scale {
            scale.set(sx, sy);
        }
        if let Some(remaining) = remaining {
            *remaining = self.clone();
            remaining.preScale(1.0 / sx, 1.0 / sy);
        }
        true
    }
}

// Source: src/core/SkMatrix.cpp:1503-1510
impl SkMatrix {
    pub(crate) fn writeToMemory(&self, buffer: Option<&mut [u8]>) -> usize {
        const SIZE: usize = 9 * core::mem::size_of::<f32>();
        if let Some(bytes) = buffer {
            assert!(bytes.len() >= SIZE);
            for (i, v) in self.fMat.iter().enumerate() {
                bytes[i * 4..i * 4 + 4].copy_from_slice(&v.to_ne_bytes());
            }
        }
        SIZE
    }
}

// Source: src/core/SkMatrix.cpp:1512-1522
impl SkMatrix {
    pub(crate) fn readFromMemory(&mut self, buffer: &[u8]) -> usize {
        const SIZE: usize = 9 * core::mem::size_of::<f32>();
        if buffer.len() < SIZE {
            return 0;
        }
        for (i, v) in self.fMat.iter_mut().enumerate() {
            *v = f32::from_ne_bytes(buffer[i * 4..i * 4 + 4].try_into().unwrap());
        }
        self.setTypeMask(Self::kUnknown_Mask);
        self.getType();
        SIZE
    }
}

// Source-aligned Skia translation. Rust methods use snake_case; fields retain upstream names.
// Source: src/core/SkMatrix.cpp:1534-1600
pub fn SkTreatAsSprite(
    mat: &SkMatrix,
    size: &crate::include::core::SkSize::SkISize,
    sampling: &crate::include::core::SkSamplingOptions::SkSamplingOptions,
    isAntiAlias: bool,
) -> bool {
    use crate::include::core::SkSamplingOptions::SkFilterMode;
    use crate::src::core::SkRect::SkIRect;
    use crate::src::core::SkSamplingPriv::SkSamplingPriv;
    if !SkSamplingPriv::NoChangeWithIdentityMatrix(sampling) {
        return false;
    }
    const AA_BITS: u32 = 4;
    let subpixel_bits = if isAntiAlias { AA_BITS } else { 0 };
    if mat.getType() & !(SkMatrix::kScale_Mask | SkMatrix::kTranslate_Mask) != 0 {
        return false;
    }
    if sampling.filter() == SkFilterMode::kLinear
        && (mat.getTranslateX() != (mat.getTranslateX() as i32) as f32
            || mat.getTranslateY() != (mat.getTranslateY() as i32) as f32)
    {
        return false;
    }
    if subpixel_bits == 0 && mat.getType() & !SkMatrix::kTranslate_Mask == 0 {
        return true;
    }
    if mat.getScaleX() < 0.0 || mat.getScaleY() < 0.0 {
        return false;
    }
    let mut dst = SkRect::MakeEmpty();
    let mut isrc = SkIRect::make_size(*size);
    {
        let src = SkRect::Make(&isrc);
        mat.mapRect(&mut dst, &src);
    }
    isrc.offset(
        crate::include::core::SkScalar::SkScalarRoundToInt(mat.getTranslateX()),
        crate::include::core::SkScalar::SkScalarRoundToInt(mat.getTranslateY()),
    );
    if subpixel_bits != 0 {
        isrc.fLeft = isrc.fLeft.wrapping_shl(subpixel_bits);
        isrc.fTop = isrc.fTop.wrapping_shl(subpixel_bits);
        isrc.fRight = isrc.fRight.wrapping_shl(subpixel_bits);
        isrc.fBottom = isrc.fBottom.wrapping_shl(subpixel_bits);
        let scale = (1 << subpixel_bits) as f32;
        dst.fLeft *= scale;
        dst.fTop *= scale;
        dst.fRight *= scale;
        dst.fBottom *= scale;
    }
    isrc == dst.round()
}

// Source: src/core/SkMatrix.cpp:1609-1702
pub fn SkDecomposeUpper2x2(
    matrix: &SkMatrix,
    rotation1: Option<&mut crate::src::core::SkPoint::SkPoint>,
    scale: Option<&mut crate::src::core::SkPoint::SkPoint>,
    rotation2: Option<&mut crate::src::core::SkPoint::SkPoint>,
) -> bool {
    use crate::include::core::SkScalar::{
        SK_ScalarNearlyZero, SkScalarNearlyEqual, SkScalarNearlyZero,
    };
    let a = matrix.fMat[0];
    let b = matrix.fMat[1];
    let c = matrix.fMat[3];
    let d = matrix.fMat[4];
    if is_degenerate_2x2(a, b, c, d) {
        return false;
    }
    let (w1, w2);
    let (mut cos1, mut sin1);
    let (cos2, sin2);
    let (mut cosq, mut sinq);
    let (sa, sb, sd);
    if SkScalarNearlyEqual(b, c, SK_ScalarNearlyZero) {
        cosq = 1.0;
        sinq = 0.0;
        sa = a as f64;
        sb = b as f64;
        sd = d as f64;
    } else {
        cosq = a + d;
        sinq = c - b;
        let recip = 1.0 / (cosq * cosq + sinq * sinq).sqrt();
        cosq *= recip;
        sinq *= recip;
        sa = (a * cosq + c * sinq) as f64;
        sb = (b * cosq + d * sinq) as f64;
        sd = (-b * sinq + d * cosq) as f64;
    }
    if SkScalarNearlyZero(sb as f32, SK_ScalarNearlyZero) {
        cos1 = 1.0;
        sin1 = 0.0;
        w1 = sa;
        w2 = sd;
        cos2 = cosq;
        sin2 = sinq;
    } else {
        let diff = sa - sd;
        let discriminant = (diff * diff + 4.0 * sb * sb).sqrt();
        let trace = sa + sd;
        if diff > 0.0 {
            w1 = 0.5 * (trace + discriminant);
            w2 = 0.5 * (trace - discriminant);
        } else {
            w1 = 0.5 * (trace - discriminant);
            w2 = 0.5 * (trace + discriminant);
        }
        cos1 = sb as f32;
        sin1 = (w1 - sa) as f32;
        let recip = 1.0 / (cos1 * cos1 + sin1 * sin1).sqrt();
        cos1 *= recip;
        sin1 *= recip;
        cos2 = cos1 * cosq - sin1 * sinq;
        sin2 = sin1 * cosq + cos1 * sinq;
        sin1 = -sin1;
    }
    if let Some(scale) = scale {
        scale.fX = w1 as f32;
        scale.fY = w2 as f32;
    }
    if let Some(r) = rotation1 {
        r.fX = cos1;
        r.fY = sin1;
    }
    if let Some(r) = rotation2 {
        r.fX = cos2;
        r.fY = sin2;
    }
    true
}

// Source: src/core/SkMatrix.cpp:1706-1737
impl crate::src::core::SkMatrixPriv::SkMatrixPriv {
    pub fn DifferentialAreaScale(m: &SkMatrix, p: &crate::src::core::SkPoint::SkPoint) -> f32 {
        let xyw = m.mapPointToHomogeneous(*p);
        if xyw.fZ < crate::include::core::SkScalar::SK_ScalarNearlyZero {
            return f32::INFINITY;
        }
        let jacobian = SkMatrix::MakeAll(
            xyw.fX,
            xyw.fY,
            xyw.fZ,
            m.getScaleX(),
            m.getSkewY(),
            m.getPerspX(),
            m.getSkewX(),
            m.getScaleY(),
            m.getPerspY(),
        );
        let mut denom = 1.0 / (xyw.fZ as f64);
        denom = denom * denom * denom;
        ((sk_determinant(&jacobian.fMat, true) * denom) as f32).abs()
    }
}

// Source-aligned Skia translation. Rust methods use snake_case; fields retain upstream names.

// Source: src/core/SkMatrix.cpp:1739-1792
impl crate::src::core::SkMatrixPriv::SkMatrixPriv {
    pub fn NearlyAffine(
        m: &SkMatrix,
        bounds: &crate::src::core::SkRect::SkRect,
        mut tolerance: f32,
    ) -> bool {
        if !m.hasPerspective() {
            return true;
        }
        let mut xyw = [crate::src::core::SkPoint3::SkPoint3::default(); 4];
        m.mapPointsToHomogeneous(
            &mut xyw,
            &bounds.toQuad(crate::include::core::SkPathTypes::SkPathDirection::kCW),
        );
        let v1 =
            crate::src::core::SkPoint3::SkPoint3::Make(m.getScaleX(), m.getSkewY(), m.getPerspX());
        let v2 =
            crate::src::core::SkPoint3::SkPoint3::Make(m.getSkewX(), m.getScaleY(), m.getPerspY());
        let cross = v1.cross(&v2);
        if xyw[0].fZ < crate::include::core::SkScalar::SK_ScalarNearlyZero {
            return false;
        }
        let mut det = cross.dot(&xyw[0]) as f64;
        let mut denom = 1.0 / xyw[0].fZ as f64;
        denom = denom * denom * denom;
        let a0 = ((det * denom) as f32).abs();
        tolerance *= tolerance;
        for p in &xyw[1..] {
            if p.fZ < crate::include::core::SkScalar::SK_ScalarNearlyZero {
                return false;
            }
            det = cross.dot(p) as f64;
            denom = 1.0 / p.fZ as f64;
            denom = denom * denom * denom;
            let a = ((det * denom) as f32).abs();
            if !crate::include::core::SkScalar::SkScalarNearlyEqual(a0, a, tolerance) {
                return false;
            }
        }
        true
    }
}

// Source: src/core/SkMatrix.cpp:1794-1805
impl crate::src::core::SkMatrixPriv::SkMatrixPriv {
    pub fn ComputeResScaleForStroking(matrix: &SkMatrix) -> f32 {
        let sx =
            crate::src::core::SkPoint::SkPoint::length_xy(matrix.getScaleX(), matrix.getSkewY());
        let sy =
            crate::src::core::SkPoint::SkPoint::length_xy(matrix.getSkewX(), matrix.getScaleY());
        if sx.is_finite() && sy.is_finite() {
            let scale = if sx < sy { sy } else { sx };
            if scale > 0.0 {
                return scale;
            }
        }
        1.0
    }
}
