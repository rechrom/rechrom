#![allow(non_snake_case)]
use crate::include::core::SkSize::{SkISize, SkSize};
use crate::src::core::SkFloatingPoint::{
    sk_float_ceil2int, sk_float_floor2int, sk_float_round2int,
};
use crate::src::core::SkPoint::{SkIPoint, SkIVector};
use crate::src::core::SkRectPriv::SkRectPriv;
// Rust f32::min/max select a numeric argument on NaN; C++ std::min/max preserve their first argument when comparison is unordered.
fn cpp_min(a: f32, b: f32) -> f32 {
    if b < a {
        b
    } else {
        a
    }
}
fn cpp_max(a: f32, b: f32) -> f32 {
    if a < b {
        b
    } else {
        a
    }
}

use crate::src::core::SkPoint::SkPoint;
// Source: include/core/SkRect.h:37-41
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq)]
pub struct SkIRect {
    pub fLeft: i32,
    pub fTop: i32,
    pub fRight: i32,
    pub fBottom: i32,
}

// Source: include/core/SkRect.h:50-52
impl SkIRect {
    #[inline]
    pub const fn make_empty() -> Self {
        Self::make_ltrb(0, 0, 0, 0)
    }
}

// Source: include/core/SkRect.h:61-63
impl SkIRect {
    #[inline]
    pub const fn make_wh(w: i32, h: i32) -> Self {
        Self::make_ltrb(0, 0, w, h)
    }
}

// Source: include/core/SkRect.h:71-73
impl SkIRect {
    #[inline]
    pub const fn make_size(size: SkISize) -> Self {
        Self::make_ltrb(0, 0, size.fWidth, size.fHeight)
    }
}

// Source: include/core/SkRect.h:83-85
impl SkIRect {
    #[inline]
    pub const fn make_pt_size(pt: SkIPoint, size: SkISize) -> Self {
        Self::make_xywh(pt.fX, pt.fY, size.fWidth, size.fHeight)
    }
}

// Source: include/core/SkRect.h:96-98
impl SkIRect {
    #[inline]
    pub const fn make_ltrb(l: i32, t: i32, r: i32, b: i32) -> Self {
        Self {
            fLeft: l,
            fTop: t,
            fRight: r,
            fBottom: b,
        }
    }
}

// Source: include/core/SkRect.h:109-111
impl SkIRect {
    #[inline]
    pub const fn make_xywh(x: i32, y: i32, w: i32, h: i32) -> Self {
        Self::make_ltrb(x, y, x.saturating_add(w), y.saturating_add(h))
    }
}

// Source: include/core/SkRect.h:118-118
impl SkIRect {
    #[inline]
    pub const fn left(&self) -> i32 {
        self.fLeft
    }
}

// Source: include/core/SkRect.h:125-125
impl SkIRect {
    #[inline]
    pub const fn top(&self) -> i32 {
        self.fTop
    }
}

// Source: include/core/SkRect.h:132-132
impl SkIRect {
    #[inline]
    pub const fn right(&self) -> i32 {
        self.fRight
    }
}

// Source: include/core/SkRect.h:139-139
impl SkIRect {
    #[inline]
    pub const fn bottom(&self) -> i32 {
        self.fBottom
    }
}

// Source: include/core/SkRect.h:146-146
impl SkIRect {
    #[inline]
    pub const fn x(&self) -> i32 {
        self.fLeft
    }
}

// Source: include/core/SkRect.h:153-153
impl SkIRect {
    #[inline]
    pub const fn y(&self) -> i32 {
        self.fTop
    }
}

// Source: include/core/SkRect.h:156-156
impl SkIRect {
    #[inline]
    pub const fn top_left(&self) -> SkIPoint {
        SkIPoint::make(self.fLeft, self.fTop)
    }
}

// Source: include/core/SkRect.h:163-163
impl SkIRect {
    #[inline]
    pub const fn width(&self) -> i32 {
        self.fRight.wrapping_sub(self.fLeft)
    }
}

// Source: include/core/SkRect.h:170-170
impl SkIRect {
    #[inline]
    pub const fn height(&self) -> i32 {
        self.fBottom.wrapping_sub(self.fTop)
    }
}

// Source: include/core/SkRect.h:177-177
impl SkIRect {
    #[inline]
    pub const fn size(&self) -> SkISize {
        SkISize::Make(self.width(), self.height())
    }
}

// Source: include/core/SkRect.h:185-185
impl SkIRect {
    #[inline]
    pub const fn width64(&self) -> i64 {
        self.fRight as i64 - self.fLeft as i64
    }
}

// Source: include/core/SkRect.h:193-193
impl SkIRect {
    #[inline]
    pub const fn height64(&self) -> i64 {
        self.fBottom as i64 - self.fTop as i64
    }
}

// Source: include/core/SkRect.h:201-201
impl SkIRect {
    #[inline]
    pub const fn is_empty64(&self) -> bool {
        self.fRight <= self.fLeft || self.fBottom <= self.fTop
    }
}

// Source: include/core/SkRect.h:207-215
impl SkIRect {
    #[inline]
    pub fn is_empty(&self) -> bool {
        let w = self.width64();
        let h = self.height64();
        if w <= 0 || h <= 0 {
            return true;
        }
        (w | h) > i32::MAX as i64
    }
}

// Source: include/core/SkRect.h:224-239
impl PartialEq for SkIRect {
    #[inline]
    fn eq(&self, b: &Self) -> bool {
        self.fLeft == b.fLeft
            && self.fTop == b.fTop
            && self.fRight == b.fRight
            && self.fBottom == b.fBottom
    }
    #[inline]
    fn ne(&self, b: &Self) -> bool {
        self.fLeft != b.fLeft
            || self.fTop != b.fTop
            || self.fRight != b.fRight
            || self.fBottom != b.fBottom
    }
}

// Source: include/core/SkRect.h:247-247
impl SkIRect {
    #[inline]
    pub fn set_empty(&mut self) {
        *self = Self::make_empty();
    }
}

// Source: include/core/SkRect.h:258-263
impl SkIRect {
    #[inline]
    pub fn set_ltrb(&mut self, l: i32, t: i32, r: i32, b: i32) {
        self.fLeft = l;
        self.fTop = t;
        self.fRight = r;
        self.fBottom = b;
    }
}

// Source: include/core/SkRect.h:273-278
impl SkIRect {
    #[inline]
    pub fn set_xywh(&mut self, x: i32, y: i32, w: i32, h: i32) {
        *self = Self::make_xywh(x, y, w, h);
    }
}

// Source: include/core/SkRect.h:280-285
impl SkIRect {
    #[inline]
    pub fn set_wh(&mut self, w: i32, h: i32) {
        *self = Self::make_wh(w, h);
    }
}

// Source: include/core/SkRect.h:287-292
impl SkIRect {
    #[inline]
    pub fn set_size(&mut self, size: SkISize) {
        self.fLeft = 0;
        self.fTop = 0;
        self.fRight = size.fWidth;
        self.fBottom = size.fHeight;
    }
}

// Source: include/core/SkRect.h:305-310
impl SkIRect {
    #[inline]
    pub const fn make_offset(&self, dx: i32, dy: i32) -> Self {
        Self::make_ltrb(
            self.fLeft.saturating_add(dx),
            self.fTop.saturating_add(dy),
            self.fRight.saturating_add(dx),
            self.fBottom.saturating_add(dy),
        )
    }
}

// Source: include/core/SkRect.h:322-324
impl SkIRect {
    #[inline]
    pub const fn make_offset_vec(&self, offset: SkIVector) -> Self {
        self.make_offset(offset.fX, offset.fY)
    }
}

// Source: include/core/SkRect.h:337-342
impl SkIRect {
    #[inline]
    pub fn make_inset(&self, dx: i32, dy: i32) -> Self {
        Self::make_ltrb(
            self.fLeft.saturating_add(dx),
            self.fTop.saturating_add(dy),
            self.fRight.saturating_sub(dx),
            self.fBottom.saturating_sub(dy),
        )
    }
}

// Source: include/core/SkRect.h:355-360
impl SkIRect {
    #[inline]
    pub fn make_outset(&self, dx: i32, dy: i32) -> Self {
        Self::make_ltrb(
            self.fLeft.saturating_sub(dx),
            self.fTop.saturating_sub(dy),
            self.fRight.saturating_add(dx),
            self.fBottom.saturating_add(dy),
        )
    }
}

// Source: include/core/SkRect.h:372-377
impl SkIRect {
    #[inline]
    pub fn offset(&mut self, dx: i32, dy: i32) {
        *self = self.make_offset(dx, dy);
    }
}

// Source: include/core/SkRect.h:389-391
impl SkIRect {
    #[inline]
    pub fn offset_point(&mut self, delta: SkIPoint) {
        self.offset(delta.fX, delta.fY);
    }
}

// Source: include/core/SkRect.h:399-404
impl SkIRect {
    #[inline]
    pub fn offset_to(&mut self, x: i32, y: i32) {
        self.fRight = (self.fRight as i64 + x as i64 - self.fLeft as i64)
            .clamp(i32::MIN as i64, i32::MAX as i64) as i32;
        self.fBottom = (self.fBottom as i64 + y as i64 - self.fTop as i64)
            .clamp(i32::MIN as i64, i32::MAX as i64) as i32;
        self.fLeft = x;
        self.fTop = y;
    }
}

// Source: include/core/SkRect.h:416-421
impl SkIRect {
    #[inline]
    pub fn inset(&mut self, dx: i32, dy: i32) {
        *self = self.make_inset(dx, dy);
    }
}

// Source: include/core/SkRect.h:433-433
impl SkIRect {
    #[inline]
    pub fn outset(&mut self, dx: i32, dy: i32) {
        self.inset(dx.wrapping_neg(), dy.wrapping_neg());
    }
}

// Source: include/core/SkRect.h:451-456
impl SkIRect {
    #[inline]
    pub fn adjust(&mut self, l: i32, t: i32, r: i32, b: i32) {
        self.fLeft = self.fLeft.saturating_add(l);
        self.fTop = self.fTop.saturating_add(t);
        self.fRight = self.fRight.saturating_add(r);
        self.fBottom = self.fBottom.saturating_add(b);
    }
}

// Source: include/core/SkRect.h:468-470
impl SkIRect {
    #[inline]
    pub fn contains_xy(&self, x: i32, y: i32) -> bool {
        x >= self.fLeft && x < self.fRight && y >= self.fTop && y < self.fBottom
    }
}

// Source: include/core/SkRect.h:480-484
impl SkIRect {
    #[inline]
    pub fn contains_rect(&self, r: &Self) -> bool {
        !r.is_empty()
            && !self.is_empty()
            && self.fLeft <= r.fLeft
            && self.fTop <= r.fTop
            && self.fRight >= r.fRight
            && self.fBottom >= r.fBottom
    }
}

// Source: include/core/SkRect.h:504-508
impl SkIRect {
    #[inline]
    pub fn contains_no_empty_check(&self, r: &Self) -> bool {
        debug_assert!(!self.is_empty64() && !r.is_empty64());
        self.fLeft <= r.fLeft
            && self.fTop <= r.fTop
            && self.fRight >= r.fRight
            && self.fBottom >= r.fBottom
    }
}

// Source: include/core/SkRect.h:518-520
impl SkIRect {
    #[inline]
    pub fn intersect(&mut self, r: &Self) -> bool {
        self.intersect_ab(*self, *r)
    }
}

// Source: include/core/SkRect.h:540-542
impl SkIRect {
    #[inline]
    pub fn intersects(a: &Self, b: &Self) -> bool {
        Self::default().intersect_ab(*a, *b)
    }
}

// Source: include/core/SkRect.h:558-566
impl SkIRect {
    #[inline]
    pub fn sort(&mut self) {
        if self.fLeft > self.fRight {
            core::mem::swap(&mut self.fLeft, &mut self.fRight);
        }
        if self.fTop > self.fBottom {
            core::mem::swap(&mut self.fTop, &mut self.fBottom);
        }
    }
}

// Source: include/core/SkRect.h:574-577
impl SkIRect {
    #[inline]
    pub fn make_sorted(&self) -> Self {
        Self::make_ltrb(
            self.fLeft.min(self.fRight),
            self.fTop.min(self.fBottom),
            self.fLeft.max(self.fRight),
            self.fTop.max(self.fBottom),
        )
    }
}

// Source: include/core/SkRect.h:584-584
impl SkIRect {
    pub fn as_int32s(&self) -> &[i32; 4] {
        unsafe { &*(core::ptr::from_ref(self).cast::<[i32; 4]>()) }
    }
}

// Source: include/core/SkRect.h:594-598
#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct SkRect {
    pub fLeft: f32,
    pub fTop: f32,
    pub fRight: f32,
    pub fBottom: f32,
}

// Source: include/core/SkRect.h:607-609
impl SkRect {
    #[inline]
    pub const fn MakeEmpty() -> Self {
        Self::MakeLTRB(0.0, 0.0, 0.0, 0.0)
    }
}

// Source: include/core/SkRect.h:621-623
impl SkRect {
    #[inline]
    pub const fn MakeWH(w: f32, h: f32) -> Self {
        Self::MakeLTRB(0.0, 0.0, w, h)
    }
}

// Source: include/core/SkRect.h:635-637
impl SkRect {
    #[inline]
    pub fn MakeIWH(w: i32, h: i32) -> Self {
        Self::MakeWH(w as f32, h as f32)
    }
}

// Source: include/core/SkRect.h:645-647
impl SkRect {
    #[inline]
    pub const fn MakeSize(size: SkSize) -> Self {
        Self::MakeLTRB(0.0, 0.0, size.fWidth, size.fHeight)
    }
}

// Source: include/core/SkRect.h:658-660
impl SkRect {
    #[inline]
    pub const fn MakeLTRB(l: f32, t: f32, r: f32, b: f32) -> Self {
        Self {
            fLeft: l,
            fTop: t,
            fRight: r,
            fBottom: b,
        }
    }
}

// Source: include/core/SkRect.h:671-673
impl SkRect {
    #[inline]
    pub const fn MakeXYWH(x: f32, y: f32, w: f32, h: f32) -> Self {
        Self::MakeLTRB(x, y, x + w, y + h)
    }
}

// Source: include/core/SkRect.h:681-683
impl SkRect {
    #[inline]
    pub fn MakeISize(size: SkISize) -> Self {
        Self::MakeIWH(size.fWidth, size.fHeight)
    }
}

// Source: include/core/SkRect.h:692-697
impl SkRect {
    #[inline]
    pub fn Make(r: &SkIRect) -> Self {
        Self::MakeLTRB(
            r.fLeft as f32,
            r.fTop as f32,
            r.fRight as f32,
            r.fBottom as f32,
        )
    }
}

// Source: include/core/SkRect.h:705-709
impl SkRect {
    #[inline]
    pub fn isEmpty(&self) -> bool {
        !(self.fLeft < self.fRight && self.fTop < self.fBottom)
    }
}

// Source: include/core/SkRect.h:717-717
impl SkRect {
    #[inline]
    pub fn isSorted(&self) -> bool {
        self.fLeft <= self.fRight && self.fTop <= self.fBottom
    }
}

// Source: include/core/SkRect.h:723-725
impl SkRect {
    #[inline]
    pub fn isFinite(&self) -> bool {
        self.fLeft.is_finite()
            && self.fTop.is_finite()
            && self.fRight.is_finite()
            && self.fBottom.is_finite()
    }
}

// Source: include/core/SkRect.h:732-732
impl SkRect {
    #[inline]
    pub const fn x(&self) -> f32 {
        self.fLeft
    }
}

// Source: include/core/SkRect.h:739-739
impl SkRect {
    #[inline]
    pub const fn y(&self) -> f32 {
        self.fTop
    }
}

// Source: include/core/SkRect.h:746-746
impl SkRect {
    #[inline]
    pub const fn left(&self) -> f32 {
        self.fLeft
    }
}

// Source: include/core/SkRect.h:753-753
impl SkRect {
    #[inline]
    pub const fn top(&self) -> f32 {
        self.fTop
    }
}

// Source: include/core/SkRect.h:760-760
impl SkRect {
    #[inline]
    pub const fn right(&self) -> f32 {
        self.fRight
    }
}

// Source: include/core/SkRect.h:767-767
impl SkRect {
    #[inline]
    pub const fn bottom(&self) -> f32 {
        self.fBottom
    }
}

// Source: include/core/SkRect.h:774-774
impl SkRect {
    #[inline]
    pub const fn width(&self) -> f32 {
        self.fRight - self.fLeft
    }
}

// Source: include/core/SkRect.h:781-781
impl SkRect {
    #[inline]
    pub const fn height(&self) -> f32 {
        self.fBottom - self.fTop
    }
}

// Source: include/core/SkRect.h:788-790
impl SkRect {
    #[inline]
    pub fn centerX(&self) -> f32 {
        (0.5 * (self.fLeft as f64 + self.fRight as f64)) as f32
    }
}

// Source: include/core/SkRect.h:797-799
impl SkRect {
    #[inline]
    pub fn centerY(&self) -> f32 {
        (0.5 * (self.fTop as f64 + self.fBottom as f64)) as f32
    }
}

// Source: include/core/SkRect.h:804-804
impl SkRect {
    #[inline]
    pub fn center(&self) -> SkPoint {
        SkPoint::make(self.centerX(), self.centerY())
    }
}

// Source: include/core/SkRect.h:816-835
impl PartialEq for SkRect {
    #[inline]
    fn eq(&self, b: &Self) -> bool {
        self.fLeft == b.fLeft
            && self.fTop == b.fTop
            && self.fRight == b.fRight
            && self.fBottom == b.fBottom
    }
    #[inline]
    fn ne(&self, b: &Self) -> bool {
        !(self == b)
    }
}

// Source: include/core/SkRect.h:837-840
impl SkRect {
    #[inline]
    pub fn TL(&self) -> SkPoint {
        SkPoint::make(self.fLeft, self.fTop)
    }
    #[inline]
    pub fn TR(&self) -> SkPoint {
        SkPoint::make(self.fRight, self.fTop)
    }
    #[inline]
    pub fn BL(&self) -> SkPoint {
        SkPoint::make(self.fLeft, self.fBottom)
    }
    #[inline]
    pub fn BR(&self) -> SkPoint {
        SkPoint::make(self.fRight, self.fBottom)
    }
}

// Source: include/core/SkRect.h:845-849
impl SkRect {
    pub fn toQuad(&self, dir: crate::include::core::SkPathTypes::SkPathDirection) -> [SkPoint; 4] {
        let mut out = [SkPoint::default(); 4];
        self.copyToQuad(&mut out, dir);
        out
    }
}

// Source: include/core/SkRect.h:853-864
impl SkRect {
    #[inline]
    pub fn copyToQuad(
        &self,
        pts: &mut [SkPoint],
        dir: crate::include::core::SkPathTypes::SkPathDirection,
    ) {
        crate::SkASSERT!(@source b"include/core/SkRect.h\0".as_ptr(),854; @condition b"pts.size() >= 4\0".as_ptr(); pts.len() >= 4);
        pts[0] = SkPoint::make(self.fLeft, self.fTop);
        pts[2] = SkPoint::make(self.fRight, self.fBottom);
        if dir == crate::include::core::SkPathTypes::SkPathDirection::kCW {
            pts[1] = SkPoint::make(self.fRight, self.fTop);
            pts[3] = SkPoint::make(self.fLeft, self.fBottom);
        } else {
            pts[1] = SkPoint::make(self.fLeft, self.fBottom);
            pts[3] = SkPoint::make(self.fRight, self.fTop);
        }
    }
}
// Source: include/core/SkRect.h:877-877
impl SkRect {
    #[inline]
    pub fn setEmpty(&mut self) {
        *self = Self::MakeEmpty();
    }
}

// Source: include/core/SkRect.h:884-889
impl SkRect {
    #[inline]
    pub fn set(&mut self, r: &SkIRect) {
        self.fLeft = r.fLeft as f32;
        self.fTop = r.fTop as f32;
        self.fRight = r.fRight as f32;
        self.fBottom = r.fBottom as f32;
    }
}

// Source: include/core/SkRect.h:900-905
impl SkRect {
    #[inline]
    pub fn setLTRB(&mut self, l: f32, t: f32, r: f32, b: f32) {
        self.fLeft = l;
        self.fTop = t;
        self.fRight = r;
        self.fBottom = b;
    }
}

// Source: include/core/SkRect.h:914-920
impl SkRect {
    #[inline]
    pub fn BoundsOrEmpty(points: &[SkPoint]) -> Self {
        if let Some(bounds) = Self::Bounds(points) {
            return bounds;
        }
        Self::MakeEmpty()
    }
}

// Source: include/core/SkRect.h:930-932
impl SkRect {
    #[inline]
    pub fn setBounds(&mut self, pts: &[SkPoint]) {
        self.setBoundsCheck(pts);
    }
}
// Source: include/core/SkRect.h:960-965
impl SkRect {
    #[inline]
    pub fn set_points(&mut self, p0: SkPoint, p1: SkPoint) {
        self.fLeft = if p1.fX < p0.fX { p1.fX } else { p0.fX };
        self.fRight = if p0.fX < p1.fX { p1.fX } else { p0.fX };
        self.fTop = if p1.fY < p0.fY { p1.fY } else { p0.fY };
        self.fBottom = if p0.fY < p1.fY { p1.fY } else { p0.fY };
    }
}
// Source: include/core/SkRect.h:975-980
impl SkRect {
    #[inline]
    pub fn setXYWH(&mut self, x: f32, y: f32, w: f32, h: f32) {
        self.fLeft = x;
        self.fTop = y;
        self.fRight = x + w;
        self.fBottom = y + h;
    }
}

// Source: include/core/SkRect.h:988-993
impl SkRect {
    #[inline]
    pub fn setWH(&mut self, w: f32, h: f32) {
        self.fLeft = 0.0;
        self.fTop = 0.0;
        self.fRight = w;
        self.fBottom = h;
    }
}

// Source: include/core/SkRect.h:994-996
impl SkRect {
    #[inline]
    pub fn setIWH(&mut self, w: i32, h: i32) {
        self.setWH(w as f32, h as f32);
    }
}

// Source: include/core/SkRect.h:1009-1011
impl SkRect {
    #[inline]
    pub const fn makeOffset(&self, dx: f32, dy: f32) -> Self {
        Self::MakeLTRB(
            self.fLeft + dx,
            self.fTop + dy,
            self.fRight + dx,
            self.fBottom + dy,
        )
    }
}

// Source: include/core/SkRect.h:1018-1018
impl SkRect {
    #[inline]
    pub const fn makeOffsetVec(&self, v: SkPoint) -> Self {
        self.makeOffset(v.fX, v.fY)
    }
}

// Source: include/core/SkRect.h:1031-1033
impl SkRect {
    #[inline]
    pub fn makeInset(&self, dx: f32, dy: f32) -> Self {
        Self::MakeLTRB(
            self.fLeft + dx,
            self.fTop + dy,
            self.fRight - dx,
            self.fBottom - dy,
        )
    }
}

// Source: include/core/SkRect.h:1046-1048
impl SkRect {
    #[inline]
    pub fn makeOutset(&self, dx: f32, dy: f32) -> Self {
        Self::MakeLTRB(
            self.fLeft - dx,
            self.fTop - dy,
            self.fRight + dx,
            self.fBottom + dy,
        )
    }
}

// Source: include/core/SkRect.h:1060-1065
impl SkRect {
    #[inline]
    pub fn offset(&mut self, dx: f32, dy: f32) {
        self.fLeft += dx;
        self.fTop += dy;
        self.fRight += dx;
        self.fBottom += dy;
    }
}

// Source: include/core/SkRect.h:1077-1079
impl SkRect {
    #[inline]
    pub fn offsetPoint(&mut self, delta: SkPoint) {
        self.offset(delta.fX, delta.fY);
    }
}

// Source: include/core/SkRect.h:1087-1092
impl SkRect {
    #[inline]
    pub fn offsetTo(&mut self, x: f32, y: f32) {
        self.fRight += x - self.fLeft;
        self.fBottom += y - self.fTop;
        self.fLeft = x;
        self.fTop = y;
    }
}

// Source: include/core/SkRect.h:1104-1109
impl SkRect {
    #[inline]
    pub fn inset(&mut self, dx: f32, dy: f32) {
        self.fLeft += dx;
        self.fTop += dy;
        self.fRight -= dx;
        self.fBottom -= dy;
    }
}

// Source: include/core/SkRect.h:1121-1121
impl SkRect {
    #[inline]
    pub fn outset(&mut self, dx: f32, dy: f32) {
        self.inset(-dx, -dy);
    }
}

// Source: include/core/SkRect.h:1148-1155
impl SkRect {
    #[inline]
    fn intersects_edges(a: Self, b: Self) -> bool {
        let l = cpp_max(a.fLeft, b.fLeft);
        let r = cpp_min(a.fRight, b.fRight);
        let t = cpp_max(a.fTop, b.fTop);
        let bot = cpp_min(a.fBottom, b.fBottom);
        l < r && t < bot
    }
}

// Source: include/core/SkRect.h:1165-1168
impl SkRect {
    #[inline]
    pub fn intersects(&self, r: &Self) -> bool {
        Self::intersects_edges(*self, *r)
    }
}

// Source: include/core/SkRect.h:1177-1180
impl SkRect {
    #[inline]
    pub fn Intersects(a: &Self, b: &Self) -> bool {
        Self::intersects_edges(*a, *b)
    }
}

// Source: include/core/SkRect.h:1202-1210
impl SkRect {
    #[inline]
    pub fn joinNonEmptyArg(&mut self, r: &Self) {
        crate::SkASSERT!(@source b"include/core/SkRect.h\0".as_ptr(),1203; @condition b"!r.isEmpty()\0".as_ptr(); !r.isEmpty());
        if self.fLeft >= self.fRight || self.fTop >= self.fBottom {
            *self = *r;
        } else {
            self.joinPossiblyEmptyRect(r);
        }
    }
}

// Source: include/core/SkRect.h:1218-1223
impl SkRect {
    #[inline]
    pub fn joinPossiblyEmptyRect(&mut self, r: &Self) {
        self.fLeft = cpp_min(self.fLeft, r.fLeft);
        self.fTop = cpp_min(self.fTop, r.fTop);
        self.fRight = cpp_max(self.fRight, r.fRight);
        self.fBottom = cpp_max(self.fBottom, r.fBottom);
    }
}

// Source: include/core/SkRect.h:1232-1234
impl SkRect {
    #[inline]
    pub fn contains_xy(&self, x: f32, y: f32) -> bool {
        x >= self.fLeft && x < self.fRight && y >= self.fTop && y < self.fBottom
    }
}

// Source: include/core/SkRect.h:1244-1249
impl SkRect {
    #[inline]
    pub fn contains(&self, r: &Self) -> bool {
        !r.isEmpty()
            && !self.isEmpty()
            && self.fLeft <= r.fLeft
            && self.fTop <= r.fTop
            && self.fRight >= r.fRight
            && self.fBottom >= r.fBottom
    }
}

// Source: include/core/SkRect.h:1259-1264
impl SkRect {
    #[inline]
    pub fn contains_irect(&self, r: &SkIRect) -> bool {
        !r.is_empty()
            && !self.isEmpty()
            && self.fLeft <= r.fLeft as f32
            && self.fTop <= r.fTop as f32
            && self.fRight >= r.fRight as f32
            && self.fBottom >= r.fBottom as f32
    }
}

// Source: include/core/SkRect.h:1272-1276
impl SkRect {
    #[inline]
    pub fn round_to(&self, dst: &mut SkIRect) {
        dst.set_ltrb(
            sk_float_round2int(self.fLeft),
            sk_float_round2int(self.fTop),
            sk_float_round2int(self.fRight),
            sk_float_round2int(self.fBottom),
        );
    }
}

// Source: include/core/SkRect.h:1285-1289
impl SkRect {
    #[inline]
    pub fn roundOutTo(&self, dst: &mut SkIRect) {
        dst.set_ltrb(
            sk_float_floor2int(self.fLeft),
            sk_float_floor2int(self.fTop),
            sk_float_ceil2int(self.fRight),
            sk_float_ceil2int(self.fBottom),
        );
    }
}

// Source: include/core/SkRect.h:1298-1301
impl SkRect {
    #[inline]
    pub fn roundOutRectTo(&self, dst: &mut Self) {
        dst.setLTRB(
            self.fLeft.floor(),
            self.fTop.floor(),
            self.fRight.ceil(),
            self.fBottom.ceil(),
        );
    }
}

// Source: include/core/SkRect.h:1310-1314
impl SkRect {
    #[inline]
    pub fn roundInTo(&self, dst: &mut SkIRect) {
        dst.set_ltrb(
            sk_float_ceil2int(self.fLeft),
            sk_float_ceil2int(self.fTop),
            sk_float_floor2int(self.fRight),
            sk_float_floor2int(self.fBottom),
        );
    }
}

// Source: include/core/SkRect.h:1322-1326
impl SkRect {
    #[inline]
    pub fn round(&self) -> SkIRect {
        let mut ir = SkIRect::default();
        self.round_to(&mut ir);
        ir
    }
}

// Source: include/core/SkRect.h:1335-1339
impl SkRect {
    #[inline]
    pub fn roundOut(&self) -> SkIRect {
        let mut ir = SkIRect::default();
        self.roundOutTo(&mut ir);
        ir
    }
}

// Source: include/core/SkRect.h:1347-1351
impl SkRect {
    #[inline]
    pub fn roundIn(&self) -> SkIRect {
        let mut ir = SkIRect::default();
        self.roundInTo(&mut ir);
        ir
    }
}

// Source: include/core/SkRect.h:1357-1366
impl SkRect {
    #[inline]
    pub fn sort(&mut self) {
        if self.fLeft > self.fRight {
            core::mem::swap(&mut self.fLeft, &mut self.fRight);
        }
        if self.fTop > self.fBottom {
            core::mem::swap(&mut self.fTop, &mut self.fBottom);
        }
    }
}

// Source: include/core/SkRect.h:1374-1377
impl SkRect {
    #[inline]
    pub fn makeSorted(&self) -> Self {
        Self::MakeLTRB(
            cpp_min(self.fLeft, self.fRight),
            cpp_min(self.fTop, self.fBottom),
            cpp_max(self.fLeft, self.fRight),
            cpp_max(self.fTop, self.fBottom),
        )
    }
}

// Source: include/core/SkRect.h:1384-1384
impl SkRect {
    pub fn asScalars(&self) -> &[f32; 4] {
        unsafe { &*(core::ptr::from_ref(self).cast::<[f32; 4]>()) }
    }
}

// Source: include/core/SkRect.h:1413-1417
impl SkIRect {
    #[inline]
    pub fn contains_frect(&self, r: &SkRect) -> bool {
        !r.isEmpty()
            && !self.is_empty()
            && self.fLeft as f32 <= r.fLeft
            && self.fTop as f32 <= r.fTop
            && self.fRight as f32 >= r.fRight
            && self.fBottom as f32 >= r.fBottom
    }
}

// Native declaration-name projections, reusing the existing SkIRect bodies.
// Source: include/core/SkRect.h:50-52
impl SkIRect {
    pub const fn MakeEmpty() -> Self {
        Self::make_empty()
    }
}
// Source: include/core/SkRect.h:61-63
impl SkIRect {
    pub const fn MakeWH(w: i32, h: i32) -> Self {
        Self::make_wh(w, h)
    }
}
// Source: include/core/SkRect.h:71-73
impl SkIRect {
    pub const fn MakeSize(size: &SkISize) -> Self {
        Self::make_size(*size)
    }
}
// Source: include/core/SkRect.h:83-85
impl SkIRect {
    pub const fn MakePtSize(pt: SkIPoint, size: SkISize) -> Self {
        Self::make_pt_size(pt, size)
    }
}
// Source: include/core/SkRect.h:96-98
impl SkIRect {
    pub const fn MakeLTRB(l: i32, t: i32, r: i32, b: i32) -> Self {
        Self::make_ltrb(l, t, r, b)
    }
}
// Source: include/core/SkRect.h:109-111
impl SkIRect {
    pub const fn MakeXYWH(x: i32, y: i32, w: i32, h: i32) -> Self {
        Self::make_xywh(x, y, w, h)
    }
}
// Source: src/core/SkRect.cpp:17-29
impl SkIRect {
    pub fn intersect_ab(&mut self, a: Self, b: Self) -> bool {
        let tmp = Self::make_ltrb(
            a.fLeft.max(b.fLeft),
            a.fTop.max(b.fTop),
            a.fRight.min(b.fRight),
            a.fBottom.min(b.fBottom),
        );
        if tmp.is_empty() {
            return false;
        }
        *self = tmp;
        true
    }
}

// Source: src/core/SkRect.cpp:31-46
impl SkIRect {
    pub fn join(&mut self, r: &Self) {
        if r.fLeft >= r.fRight || r.fTop >= r.fBottom {
            return;
        }
        if self.fLeft >= self.fRight || self.fTop >= self.fBottom {
            *self = *r;
        } else {
            if r.fLeft < self.fLeft {
                self.fLeft = r.fLeft;
            }
            if r.fTop < self.fTop {
                self.fTop = r.fTop;
            }
            if r.fRight > self.fRight {
                self.fRight = r.fRight;
            }
            if r.fBottom > self.fBottom {
                self.fBottom = r.fBottom;
            }
        }
    }
}

// Source: src/core/SkRect.cpp:50-119
impl SkRect {
    pub fn Bounds(points: &[SkPoint]) -> Option<Self> {
        if points.is_empty() {
            return Some(Self::MakeEmpty());
        }
        if cfg!(target_pointer_width = "64") {
            let mut l = points[0].fX;
            let mut t = points[0].fY;
            let mut r = l;
            let mut b = t;
            let mut nx = 0.0;
            let mut ny = 0.0;
            for p in points {
                l = p.fX.min(l);
                t = p.fY.min(t);
                r = p.fX.max(r);
                b = p.fY.max(b);
                nx *= p.fX;
                ny *= p.fY;
            }
            if nx == 0.0 && ny == 0.0 {
                return Some(Self::MakeLTRB(l, t, r, b));
            }
        } else {
            // Explicit four scalar lanes preserve upstream skvx load/shuffle/reduction order.
            let mut count = points.len();
            let mut cursor;
            let mut min = if count & 1 != 0 {
                cursor = 1;
                count -= 1;
                [points[0].fX, points[0].fY, points[0].fX, points[0].fY]
            } else {
                cursor = 2;
                count -= 2;
                [points[0].fX, points[0].fY, points[1].fX, points[1].fY]
            };
            let mut max = min;
            let mut accum = min.map(|v| v * 0.0);
            while count != 0 {
                let xy = [
                    points[cursor].fX,
                    points[cursor].fY,
                    points[cursor + 1].fX,
                    points[cursor + 1].fY,
                ];
                for i in 0..4 {
                    accum[i] *= xy[i];
                    min[i] = cpp_min(min[i], xy[i]);
                    max[i] = cpp_max(max[i], xy[i]);
                }
                cursor += 2;
                count -= 2;
            }
            if accum.iter().all(|v| v * 0.0 == 0.0) {
                return Some(Self::MakeLTRB(
                    cpp_min(min[0], min[2]),
                    cpp_min(min[1], min[3]),
                    cpp_max(max[0], max[2]),
                    cpp_max(max[1], max[3]),
                ));
            }
        }
        None
    }
}

// Source: src/core/SkRect.cpp:121-129
impl SkRect {
    pub fn setBoundsCheck(&mut self, pts: &[SkPoint]) -> bool {
        if let Some(bounds) = Self::Bounds(pts) {
            *self = bounds;
            true
        } else {
            *self = Self::MakeEmpty();
            false
        }
    }
}

// Source: src/core/SkRect.cpp:131-137
impl SkRect {
    pub fn setBoundsNoCheck(&mut self, pts: &[SkPoint]) {
        if let Some(bounds) = Self::Bounds(pts) {
            *self = bounds;
        } else {
            self.setLTRB(f32::NAN, f32::NAN, f32::NAN, f32::NAN);
        }
    }
}

// Source: src/core/SkRect.cpp:139-145
impl SkRect {
    fn intersection(a: Self, b: Self) -> Option<Self> {
        let l = cpp_max(a.fLeft, b.fLeft);
        let r = cpp_min(a.fRight, b.fRight);
        let t = cpp_max(a.fTop, b.fTop);
        let bot = cpp_min(a.fBottom, b.fBottom);
        if !(l < r && t < bot) {
            return None;
        }
        Some(Self::MakeLTRB(l, t, r, bot))
    }
}

// Source: src/core/SkRect.cpp:147-151
impl SkRect {
    pub fn intersect(&mut self, r: &Self) -> bool {
        if let Some(v) = Self::intersection(*r, *self) {
            *self = v;
            true
        } else {
            false
        }
    }
}

// Source: src/core/SkRect.cpp:153-157
impl SkRect {
    pub fn intersect_ab(&mut self, a: Self, b: Self) -> bool {
        if let Some(v) = Self::intersection(a, b) {
            *self = v;
            true
        } else {
            false
        }
    }
}

// Source: src/core/SkRect.cpp:159-172
impl SkRect {
    pub fn join(&mut self, r: &Self) {
        if r.isEmpty() {
            return;
        }
        if self.isEmpty() {
            *self = *r;
        } else {
            self.fLeft = cpp_min(self.fLeft, r.fLeft);
            self.fTop = cpp_min(self.fTop, r.fTop);
            self.fRight = cpp_max(self.fRight, r.fRight);
            self.fBottom = cpp_max(self.fBottom, r.fBottom);
        }
    }
}

// Source: src/core/SkRect.cpp:213-287
// The source template is instantiated separately for the two concrete coordinate types.
macro_rules! subtract_rect {
    ($name:ident, $ty:ty, $empty:ident, $make_empty:ident, $intersects:expr, $difference:expr) => {
        fn $name(a: &$ty, b: &$ty, out: &mut $ty) -> bool {
            if a.$empty() || b.$empty() || !($intersects)(a, b) {
                *out = *a;
                return true;
            }
            let a_height = a.height() as f32;
            let a_width = a.width() as f32;
            let (mut left, mut right, mut top, mut bottom) = (0.0, 0.0, 0.0, 0.0);
            let mut positive_count = 0;
            if b.fLeft > a.fLeft {
                left = ($difference)(b.fLeft, a.fLeft) / a_width;
                positive_count += 1;
            }
            if a.fRight > b.fRight {
                right = ($difference)(a.fRight, b.fRight) / a_width;
                positive_count += 1;
            }
            if b.fTop > a.fTop {
                top = ($difference)(b.fTop, a.fTop) / a_height;
                positive_count += 1;
            }
            if a.fBottom > b.fBottom {
                bottom = ($difference)(a.fBottom, b.fBottom) / a_height;
                positive_count += 1;
            }
            if positive_count == 0 {
                *out = <$ty>::$make_empty();
                return true;
            }
            *out = *a;
            if left > right && left > top && left > bottom {
                out.fRight = b.fLeft;
            } else if right > top && right > bottom {
                out.fLeft = b.fRight;
            } else if top > bottom {
                out.fBottom = b.fTop;
            } else {
                debug_assert!(bottom > 0.0);
                out.fTop = b.fBottom;
            }
            debug_assert!(!($intersects)(out, b));
            positive_count == 1
        }
    };
}
subtract_rect!(
    subtract_irect,
    SkIRect,
    is_empty,
    make_empty,
    SkIRect::intersects,
    |a: i32, b: i32| a.wrapping_sub(b) as f32
);
subtract_rect!(
    subtract_frect,
    SkRect,
    isEmpty,
    MakeEmpty,
    |a: &SkRect, b: &SkRect| a.intersects(b),
    |a: f32, b: f32| a - b
);

// Source: src/core/SkRect.cpp:289-291
impl crate::src::core::SkRectPriv::SkRectPriv {
    pub fn Subtract(a: &SkRect, b: &SkRect, out: &mut SkRect) -> bool {
        subtract_frect(a, b, out)
    }
}

// Source: src/core/SkRect.cpp:293-295
impl crate::src::core::SkRectPriv::SkRectPriv {
    pub fn SubtractIRect(a: &SkIRect, b: &SkIRect, out: &mut SkIRect) -> bool {
        subtract_irect(a, b, out)
    }
}

// Source: src/core/SkRect.cpp:298-303
impl SkRectPriv {
    pub fn QuadContainsIRect(
        m: &crate::src::core::SkMatrix::SkMatrix,
        a: &SkIRect,
        b: &SkIRect,
        tol: f32,
    ) -> bool {
        Self::QuadContainsRect(
            &crate::src::core::SkM44::SkM44::from(m),
            &SkRect::Make(a),
            &SkRect::Make(b),
            tol,
        )
    }
}
// Source: src/core/SkRect.cpp:305-307
impl SkRectPriv {
    pub fn QuadContainsRect(
        m: &crate::src::core::SkM44::SkM44,
        a: &SkRect,
        b: &SkRect,
        tol: f32,
    ) -> bool {
        Self::QuadContainsRectMask(m, a, b, tol)
            .iter()
            .all(|&v| v != 0)
    }
}
// Source: src/core/SkRect.cpp:309-358
impl SkRectPriv {
    // Four scalar lanes preserve native T,R,B,L order and all-bits-set comparison masks.
    pub fn QuadContainsRectMask(
        m: &crate::src::core::SkM44::SkM44,
        a: &SkRect,
        b: &SkRect,
        tol: f32,
    ) -> [i32; 4] {
        #[cfg(sk_debug)]
        {
            let mut inverse = crate::src::core::SkM44::SkM44::default();
            debug_assert!(m.invert(&mut inverse));
        }
        if a.isEmpty() {
            return [0; 4];
        }
        let ax = [a.fLeft, a.fRight, a.fRight, a.fLeft];
        let ay = [a.fTop, a.fTop, a.fBottom, a.fBottom];
        let x: [f32; 4] =
            core::array::from_fn(|i| m.rc(0, 0) * ax[i] + m.rc(0, 1) * ay[i] + m.rc(0, 3));
        let y: [f32; 4] =
            core::array::from_fn(|i| m.rc(1, 0) * ax[i] + m.rc(1, 1) * ay[i] + m.rc(1, 3));
        let w: [f32; 4] =
            core::array::from_fn(|i| m.rc(3, 0) * ax[i] + m.rc(3, 1) * ay[i] + m.rc(3, 3));
        if w.iter().all(|&v| v < 0.0) {
            return [0; 4];
        }
        let la: [f32; 4] = core::array::from_fn(|i| y[i] * w[(i + 1) % 4] - w[i] * y[(i + 1) % 4]);
        let lb: [f32; 4] = core::array::from_fn(|i| w[i] * x[(i + 1) % 4] - x[i] * w[(i + 1) % 4]);
        let lc: [f32; 4] = core::array::from_fn(|i| x[i] * y[(i + 1) % 4] - y[i] * x[(i + 1) % 4]);
        let sign = if la[0] * lb[1] - lb[0] * la[1] < 0.0 {
            -1.0
        } else {
            1.0
        };
        let inset = b.makeInset(tol, tol);
        core::array::from_fn(|i| {
            let d0 = sign * (la[i] * inset.fLeft + lb[i] * inset.fTop + lc[i]);
            let d1 = sign * (la[i] * inset.fRight + lb[i] * inset.fTop + lc[i]);
            let d2 = sign * (la[i] * inset.fRight + lb[i] * inset.fBottom + lc[i]);
            let d3 = sign * (la[i] * inset.fLeft + lb[i] * inset.fBottom + lc[i]);
            -((d0 >= 0.0) as i32)
                & -((d1 >= 0.0) as i32)
                & -((d2 >= 0.0) as i32)
                & -((d3 >= 0.0) as i32)
        })
    }
}
// Source: src/core/SkRect.cpp:360-393
impl SkRectPriv {
    pub fn ClosestDisjointEdge(src: &SkIRect, dst: &SkIRect) -> SkIRect {
        use crate::include::private::SkTPin::SkTPin;
        if src.is_empty() || dst.is_empty() {
            return SkIRect::make_empty();
        }
        let mut l = src.fLeft;
        let mut r = src.fRight;
        if r <= dst.fLeft {
            l = r - 1;
        } else if l >= dst.fRight {
            r = l + 1;
        } else {
            l = SkTPin(l, dst.fLeft, dst.fRight);
            r = SkTPin(r, dst.fLeft, dst.fRight);
        }
        let mut t = src.fTop;
        let mut b = src.fBottom;
        if b <= dst.fTop {
            t = b - 1;
        } else if t >= dst.fBottom {
            b = t + 1;
        } else {
            t = SkTPin(t, dst.fTop, dst.fBottom);
            b = SkTPin(b, dst.fTop, dst.fBottom);
        }
        SkIRect::make_ltrb(l, t, r, b)
    }
}
// Source: include/core/SkRect.h:867-869
impl SkRect {
    pub fn toQuad_to(&self, quad: &mut [SkPoint; 4]) {
        self.copyToQuad(
            quad,
            crate::include::core::SkPathTypes::SkPathDirection::kCW,
        );
    }
}

// Source: src/core/SkRect.cpp:178-183
fn set_scalar(
    storage: &mut crate::src::core::SkString::SkString,
    value: f32,
    as_type: crate::src::core::SkStringUtils::SkScalarAsStringType,
) -> *const u8 {
    storage.reset();
    crate::src::core::SkStringUtils::SkAppendScalar(storage, value, as_type);
    storage.c_str()
}
// Source: include/core/SkRect.h:1393-1394
// Source: src/core/SkRect.cpp:185-205
impl SkRect {
    pub fn dumpToString(&self, as_hex: bool) -> crate::src::core::SkString::SkString {
        use crate::src::core::{
            SkString::SkString,
            SkStringUtils::{SkAppendScalarDec, SkScalarAsStringType::*},
        };
        let as_type = if as_hex {
            kHex_SkScalarAsStringType
        } else {
            kDec_SkScalarAsStringType
        };
        let mut line = SkString::default();
        if as_hex {
            let mut tmp = SkString::default();
            unsafe {
                line.printf(
                    b"SkRect::MakeLTRB(%s, /* %f */\n\0".as_ptr(),
                    set_scalar(&mut tmp, self.fLeft, as_type),
                    self.fLeft as f64,
                );
                line.appendf(
                    b"                 %s, /* %f */\n\0".as_ptr(),
                    set_scalar(&mut tmp, self.fTop, as_type),
                    self.fTop as f64,
                );
                line.appendf(
                    b"                 %s, /* %f */\n\0".as_ptr(),
                    set_scalar(&mut tmp, self.fRight, as_type),
                    self.fRight as f64,
                );
                line.appendf(
                    b"                 %s  /* %f */);\0".as_ptr(),
                    set_scalar(&mut tmp, self.fBottom, as_type),
                    self.fBottom as f64,
                );
            }
        } else {
            let (mut l, mut t, mut r, mut b) = (
                SkString::default(),
                SkString::default(),
                SkString::default(),
                SkString::default(),
            );
            SkAppendScalarDec(&mut l, self.fLeft);
            SkAppendScalarDec(&mut t, self.fTop);
            SkAppendScalarDec(&mut r, self.fRight);
            SkAppendScalarDec(&mut b, self.fBottom);
            unsafe {
                line.printf(
                    b"SkRect::MakeLTRB(%s, %s, %s, %s);\0".as_ptr(),
                    l.c_str(),
                    t.c_str(),
                    r.c_str(),
                    b.c_str(),
                );
            }
        }
        line
    }
    // Source: src/core/SkRect.cpp:207-209
    pub fn dump_as_hex(&self, as_hex: bool) {
        let line = self.dumpToString(as_hex);
        unsafe {
            crate::SkDebugf!(@source b"src/core/SkRect.cpp\0".as_ptr(),208; b"%s\n\0".as_ptr(),line.c_str());
        }
    }
    // Source: include/core/SkRect.h:1401-1401
    pub fn dump(&self) {
        self.dump_as_hex(false);
    }
    // Source: include/core/SkRect.h:1410-1410
    pub fn dumpHex(&self) {
        self.dump_as_hex(true);
    }
}
