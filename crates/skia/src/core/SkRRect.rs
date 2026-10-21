use crate::src::core::SkPoint::SkPoint;
use crate::src::core::SkRect::SkRect;
use crate::src::core::SkRectPriv::SkRectPriv;

// Source: include/core/SkRRect.h:70-78
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum Type {
    kEmpty_Type = 0,
    kRect_Type = 1,
    kOval_Type = 2,
    kSimple_Type = 3,
    kNinePatch_Type = 4,
    kComplex_Type = 5,
}
impl Type {
    pub const kLastType: Self = Self::kComplex_Type;
}

// Source: include/core/SkRRect.h:80-83
impl SkRRect {
    #[inline]
    pub fn getType(&self) -> Type {
        crate::SkASSERT!(@source b"include/core/SkRRect.h\0".as_ptr(),81; @condition b"this->isValid()\0".as_ptr(); self.isValid());
        match self.fType {
            0 => Type::kEmpty_Type,
            1 => Type::kRect_Type,
            2 => Type::kOval_Type,
            3 => Type::kSimple_Type,
            4 => Type::kNinePatch_Type,
            5 => Type::kComplex_Type,
            _ => unreachable!("invalid RRect type"),
        }
    }
}

// Source: include/core/SkRRect.h:85-85
impl SkRRect {
    #[inline]
    pub fn r#type(&self) -> Type {
        self.getType()
    }
}

// Source: include/core/SkRRect.h:87-87
impl SkRRect {
    #[inline]
    pub fn isEmpty(&self) -> bool {
        self.getType() == Type::kEmpty_Type
    }
}

// Source: include/core/SkRRect.h:88-88
impl SkRRect {
    #[inline]
    pub fn isRect(&self) -> bool {
        self.getType() == Type::kRect_Type
    }
}

// Source: include/core/SkRRect.h:89-89
impl SkRRect {
    #[inline]
    pub fn isOval(&self) -> bool {
        self.getType() == Type::kOval_Type
    }
}

// Source: include/core/SkRRect.h:90-90
impl SkRRect {
    #[inline]
    pub fn isSimple(&self) -> bool {
        self.getType() == Type::kSimple_Type
    }
}

// Source: include/core/SkRRect.h:91-91
impl SkRRect {
    #[inline]
    pub fn isNinePatch(&self) -> bool {
        self.getType() == Type::kNinePatch_Type
    }
}

// Source: include/core/SkRRect.h:92-92
impl SkRRect {
    #[inline]
    pub fn isComplex(&self) -> bool {
        self.getType() == Type::kComplex_Type
    }
}

// Source: include/core/SkRRect.h:99-99
impl SkRRect {
    #[inline]
    pub fn width(&self) -> f32 {
        self.fRect.width()
    }
}

// Source: include/core/SkRRect.h:106-106
impl SkRRect {
    #[inline]
    pub fn height(&self) -> f32 {
        self.fRect.height()
    }
}

// Source: include/core/SkRRect.h:115-117
impl SkRRect {
    #[inline]
    pub fn getSimpleRadii(&self) -> SkPoint {
        self.fRadii[0]
    }
}

// Source: include/core/SkRRect.h:122-122
impl SkRRect {
    #[inline]
    pub fn setEmpty(&mut self) {
        *self = Self::default();
    }
}

// Source: include/core/SkRRect.h:130-139
impl SkRRect {
    #[inline]
    pub fn setRect(&mut self, rect: &SkRect) {
        if !self.initializeRect(rect) {
            return;
        }
        self.fRadii = [SkPoint::make(0.0, 0.0); 4];
        self.fType = 1;
        crate::SkASSERT!(@source b"include/core/SkRRect.h\0".as_ptr(),138; @condition b"this->isValid()\0".as_ptr(); self.isValid());
    }
}

// Source: include/core/SkRRect.h:146-146
impl SkRRect {
    #[inline]
    pub fn MakeEmpty() -> Self {
        Self::default()
    }
}

// Source: include/core/SkRRect.h:153-157
impl SkRRect {
    #[inline]
    pub fn MakeRect(r: &SkRect) -> Self {
        let mut rr = Self::default();
        rr.setRect(r);
        rr
    }
}

// Source: include/core/SkRRect.h:166-170
impl SkRRect {
    #[inline]
    pub fn MakeOval(r: &SkRect) -> Self {
        let mut rr = Self::default();
        rr.setOval(r);
        rr
    }
}

// Source: include/core/SkRRect.h:184-188
impl SkRRect {
    #[inline]
    pub fn MakeRectXY(r: &SkRect, x: f32, y: f32) -> Self {
        let mut rr = Self::default();
        rr.setRectXY(r, x, y);
        rr
    }
}

// Source: include/core/SkRRect.h:205-209
impl SkRRect {
    pub fn MakeRectRadii(r: &SkRect, radii: &[SkPoint; 4]) -> Self {
        let mut rr = Self::default();
        rr.setRectRadii(r, radii);
        rr
    }
}

// Source: include/core/SkRRect.h:276-281
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(usize)]
pub enum Corner {
    kUpperLeft_Corner = 0,
    kUpperRight_Corner = 1,
    kLowerRight_Corner = 2,
    kLowerLeft_Corner = 3,
}

// Source: include/core/SkRRect.h:289-289
impl SkRRect {
    #[inline]
    pub fn rect(&self) -> &SkRect {
        &self.fRect
    }
}

// Source: include/core/SkRRect.h:296-296
impl SkRRect {
    #[inline]
    pub fn radii(&self, corner: Corner) -> SkPoint {
        self.fRadii[corner as usize]
    }
}

// Source: include/core/SkRRect.h:300-300
impl SkRRect {
    #[inline]
    pub fn all_radii(&self) -> &[SkPoint] {
        &self.fRadii
    }
}

// Source: include/core/SkRRect.h:308-308
impl SkRRect {
    #[inline]
    pub fn getBounds(&self) -> &SkRect {
        &self.fRect
    }
}

// Source: include/core/SkRRect.h:319-321
impl PartialEq for SkRRect {
    #[inline]
    fn eq(&self, b: &Self) -> bool {
        self.fRect == b.fRect && self.fRadii == b.fRadii
    }

    // Source: include/core/SkRRect.h:332-334
    #[inline]
    fn ne(&self, b: &Self) -> bool {
        self.fRect != b.fRect || self.fRadii != b.fRadii
    }
}

// Source: include/core/SkRRect.h:370-372
impl SkRRect {
    #[inline]
    pub fn inset(&mut self, dx: f32, dy: f32) {
        let source = *self;
        source.inset_to(dx, dy, self);
    }
}

// Source: include/core/SkRRect.h:389-391
impl SkRRect {
    #[inline]
    pub fn outset_to(&self, dx: f32, dy: f32, dst: &mut Self) {
        self.inset_to(-dx, -dy, dst);
    }
}
// Source: include/core/SkRRect.h:407-409
impl SkRRect {
    #[inline]
    pub fn outset(&mut self, dx: f32, dy: f32) {
        self.inset(-dx, -dy);
    }
}
// Source: include/core/SkRRect.h:416-418
impl SkRRect {
    #[inline]
    pub fn offset(&mut self, dx: f32, dy: f32) {
        self.fRect.offset(dx, dy);
    }
}
// Source: include/core/SkRRect.h:426-428
impl SkRRect {
    #[inline]
    pub fn makeOffset(&self, dx: f32, dy: f32) -> Self {
        Self::from_rect_radii(self.fRect.makeOffset(dx, dy), self.fRadii, self.fType)
    }
}
// Source: include/core/SkRRect.h:520-523
impl SkRRect {
    fn from_rect_radii(rect: SkRect, radii: [SkPoint; 4], rrect_type: i32) -> Self {
        Self {
            fRect: rect,
            fRadii: radii,
            fType: rrect_type,
        }
    }
}
// Source: include/core/SkRRect.h:536-540
// Source: include/core/SkRRect.h:54-54
#[derive(Clone, Copy, Debug)]
pub struct SkRRect {
    pub(crate) fRect: SkRect,
    pub(crate) fRadii: [SkPoint; 4],
    fType: i32,
}
// Source: include/core/SkRRect.h:47-47
impl Default for SkRRect {
    #[inline]
    fn default() -> Self {
        Self {
            fRect: SkRect::MakeEmpty(),
            fRadii: [SkPoint::make(0.0, 0.0); 4],
            fType: 0,
        }
    }
}

// Source: src/core/SkRRect.cpp:33-53
impl SkRRect {
    pub fn setOval(&mut self, oval: &SkRect) {
        if !self.initializeRect(oval) {
            return;
        }
        let x = SkRectPriv::HalfWidth(&self.fRect);
        let y = SkRectPriv::HalfHeight(&self.fRect);
        if x == 0.0 || y == 0.0 {
            self.fRadii = [SkPoint::make(0.0, 0.0); 4];
            self.fType = 1;
        } else {
            for r in &mut self.fRadii {
                r.set(x, y);
            }
            self.fType = 2;
        }
        crate::SkASSERT!(@source b"src/core/SkRRect.cpp\0".as_ptr(),52; @condition b"this->isValid()\0".as_ptr(); self.isValid());
    }
}

// Source: src/core/SkRRect.cpp:55-93
impl SkRRect {
    pub fn setRectXY(&mut self, rect: &SkRect, mut x: f32, mut y: f32) {
        if !self.initializeRect(rect) {
            return;
        }
        if !x.is_finite() || !y.is_finite() {
            x = 0.0;
            y = 0.0;
        }
        if self.fRect.width() < x + x || self.fRect.height() < y + y {
            let a = self.fRect.width() / (x + x);
            let b = self.fRect.height() / (y + y);
            let scale = if b < a { b } else { a };
            debug_assert!(scale < 1.0);
            x *= scale;
            y *= scale;
        }
        if x <= 0.0 || y <= 0.0 {
            self.setRect(rect);
            return;
        }
        for r in &mut self.fRadii {
            r.set(x, y);
        }
        self.fType = 3;
        if x >= self.fRect.width() / 2.0 && y >= self.fRect.height() / 2.0 {
            self.fType = 2;
            x = SkRectPriv::HalfWidth(&self.fRect);
            y = SkRectPriv::HalfHeight(&self.fRect);
            for r in &mut self.fRadii {
                r.set(x, y);
            }
        }
        crate::SkASSERT!(@source b"src/core/SkRRect.cpp\0".as_ptr(),92; @condition b"this->isValid()\0".as_ptr(); self.isValid());
    }
}

// Source: src/core/SkRRect.cpp:95-113
fn clamp_to_zero(radii: &mut [SkPoint; 4]) -> bool {
    let mut all_corners_square = true;
    for r in radii {
        if r.fX <= 0.0 || r.fY <= 0.0 {
            r.fX = 0.0;
            r.fY = 0.0;
        } else {
            all_corners_square = false;
        }
    }
    all_corners_square
}

// Source: src/core/SkRRect.cpp:115-120
fn radii_are_nine_patch(r: &[SkPoint; 4]) -> bool {
    r[0].fX == r[3].fX && r[0].fY == r[1].fY && r[1].fX == r[2].fX && r[3].fY == r[2].fY
}

// Source: src/core/SkRRect.cpp:122-186
impl SkRRect {
    pub fn setNinePatch(&mut self, rect: &SkRect, mut l: f32, mut t: f32, mut r: f32, mut b: f32) {
        if !self.initializeRect(rect) {
            return;
        }
        if ![l, t, r, b].iter().all(|v| v.is_finite()) {
            self.setRect(rect);
            return;
        }
        // Arguments are finite here, so max preserves C++ ordering including -0.
        l = if l < 0.0 { 0.0 } else { l };
        t = if t < 0.0 { 0.0 } else { t };
        r = if r < 0.0 { 0.0 } else { r };
        b = if b < 0.0 { 0.0 } else { b };
        let mut scale = 1.0;
        if l + r > self.fRect.width() {
            scale = self.fRect.width() / (l + r);
        }
        if t + b > self.fRect.height() {
            let v = self.fRect.height() / (t + b);
            if v < scale {
                scale = v;
            }
        }
        if scale < 1.0 {
            l *= scale;
            t *= scale;
            r *= scale;
            b *= scale;
        }
        if l == r && t == b {
            if l >= self.fRect.width() / 2.0 && t >= self.fRect.height() / 2.0 {
                self.fType = 2;
                l = SkRectPriv::HalfWidth(&self.fRect);
                r = l;
                t = SkRectPriv::HalfHeight(&self.fRect);
                b = t;
            } else if l == 0.0 || t == 0.0 {
                self.fType = 1;
                l = 0.0;
                t = 0.0;
                r = 0.0;
                b = 0.0;
            } else {
                self.fType = 3;
            }
        } else {
            self.fType = 4;
        }
        self.fRadii[0].set(l, t);
        self.fRadii[1].set(r, t);
        self.fRadii[2].set(r, b);
        self.fRadii[3].set(l, b);
        if clamp_to_zero(&mut self.fRadii) {
            self.setRect(rect);
            return;
        }
        if self.fType == 4 && !radii_are_nine_patch(&self.fRadii) {
            self.fType = 5;
        }
        crate::SkASSERT!(@source b"src/core/SkRRect.cpp\0".as_ptr(),185; @condition b"this->isValid()\0".as_ptr(); self.isValid());
    }
}

// Source: src/core/SkRRect.cpp:191-196
fn compute_min_scale(rad1: f64, rad2: f64, limit: f64, cur_min: f64) -> f64 {
    if rad1 + rad2 > limit {
        let scale = limit / (rad1 + rad2);
        if scale < cur_min {
            scale
        } else {
            cur_min
        }
    } else {
        cur_min
    }
}

// Source: src/core/SkRRect.cpp:198-221
impl SkRRect {
    pub fn setRectRadii(&mut self, rect: &SkRect, radii: &[SkPoint; 4]) {
        if !self.initializeRect(rect) {
            return;
        }
        if !radii.iter().all(SkPoint::is_finite) {
            self.setRect(rect);
            return;
        }
        self.fRadii = *radii;
        if clamp_to_zero(&mut self.fRadii) {
            self.setRect(rect);
            return;
        }
        self.scaleRadii();
        if !self.isValid() {
            self.setRect(rect);
            return;
        }
    }
}

// Source: src/core/SkRRect.cpp:223-236
impl SkRRect {
    fn initializeRect(&mut self, rect: &SkRect) -> bool {
        if !rect.isFinite() {
            *self = Self::default();
            return false;
        }
        self.fRect = rect.makeSorted();
        if self.fRect.isEmpty() {
            self.fRadii = [SkPoint::make(0.0, 0.0); 4];
            self.fType = 0;
            return false;
        }
        true
    }
}

// Source: src/core/SkRRect.cpp:241-249
fn flush_to_zero(a: &mut f32, b: &mut f32) {
    debug_assert!(*a >= 0.0 && *b >= 0.0);
    if *a + *b == *a {
        *b = 0.0;
    } else if *a + *b == *b {
        *a = 0.0;
    }
}

// Source: src/core/SkRRect.cpp:251-294
impl SkRRect {
    fn scaleRadii(&mut self) -> bool {
        let mut scale = 1.0;
        let width = self.fRect.fRight as f64 - self.fRect.fLeft as f64;
        let height = self.fRect.fBottom as f64 - self.fRect.fTop as f64;
        for (a, b, is_x, limit) in [
            (0, 1, true, width),
            (1, 2, false, height),
            (2, 3, true, width),
            (3, 0, false, height),
        ] {
            let ra = if is_x {
                self.fRadii[a].fX
            } else {
                self.fRadii[a].fY
            };
            let rb = if is_x {
                self.fRadii[b].fX
            } else {
                self.fRadii[b].fY
            };
            scale = compute_min_scale(ra as f64, rb as f64, limit, scale);
        }
        // Copy each disjoint pair into values to express C++'s two writable references in Rust.
        for (a, b, is_x) in [(0, 1, true), (1, 2, false), (2, 3, true), (3, 0, false)] {
            let mut ra = if is_x {
                self.fRadii[a].fX
            } else {
                self.fRadii[a].fY
            };
            let mut rb = if is_x {
                self.fRadii[b].fX
            } else {
                self.fRadii[b].fY
            };
            flush_to_zero(&mut ra, &mut rb);
            if is_x {
                self.fRadii[a].fX = ra;
                self.fRadii[b].fX = rb;
            } else {
                self.fRadii[a].fY = ra;
                self.fRadii[b].fY = rb;
            }
        }
        if scale < 1.0 {
            for (a, b, is_x, limit) in [
                (0, 1, true, width),
                (1, 2, false, height),
                (2, 3, true, width),
                (3, 0, false, height),
            ] {
                let mut ra = if is_x {
                    self.fRadii[a].fX
                } else {
                    self.fRadii[a].fY
                };
                let mut rb = if is_x {
                    self.fRadii[b].fX
                } else {
                    self.fRadii[b].fY
                };
                crate::src::core::SkScaleToSides::SkScaleToSides::AdjustRadii(
                    limit, scale, &mut ra, &mut rb,
                );
                if is_x {
                    self.fRadii[a].fX = ra;
                    self.fRadii[b].fX = rb;
                } else {
                    self.fRadii[a].fY = ra;
                    self.fRadii[b].fY = rb;
                }
            }
        }
        clamp_to_zero(&mut self.fRadii);
        self.computeType();
        scale < 1.0
    }
}

// Source: src/core/SkRRect.cpp:298-349
impl SkRRect {
    fn checkCornerContainment(&self, x: f32, y: f32) -> bool {
        let canonical;
        let index;
        if self.getType() == Type::kOval_Type {
            canonical = SkPoint::make(x - self.fRect.centerX(), y - self.fRect.centerY());
            index = 0;
        } else if x < self.fRect.fLeft + self.fRadii[0].fX
            && y < self.fRect.fTop + self.fRadii[0].fY
        {
            index = 0;
            canonical = SkPoint::make(
                x - (self.fRect.fLeft + self.fRadii[0].fX),
                y - (self.fRect.fTop + self.fRadii[0].fY),
            );
            debug_assert!(canonical.fX < 0.0 && canonical.fY < 0.0);
        } else if x < self.fRect.fLeft + self.fRadii[3].fX
            && y > self.fRect.fBottom - self.fRadii[3].fY
        {
            index = 3;
            canonical = SkPoint::make(
                x - (self.fRect.fLeft + self.fRadii[3].fX),
                y - (self.fRect.fBottom - self.fRadii[3].fY),
            );
            debug_assert!(canonical.fX < 0.0 && canonical.fY > 0.0);
        } else if x > self.fRect.fRight - self.fRadii[1].fX
            && y < self.fRect.fTop + self.fRadii[1].fY
        {
            index = 1;
            canonical = SkPoint::make(
                x - (self.fRect.fRight - self.fRadii[1].fX),
                y - (self.fRect.fTop + self.fRadii[1].fY),
            );
            debug_assert!(canonical.fX > 0.0 && canonical.fY < 0.0);
        } else if x > self.fRect.fRight - self.fRadii[2].fX
            && y > self.fRect.fBottom - self.fRadii[2].fY
        {
            index = 2;
            canonical = SkPoint::make(
                x - (self.fRect.fRight - self.fRadii[2].fX),
                y - (self.fRect.fBottom - self.fRadii[2].fY),
            );
            debug_assert!(canonical.fX > 0.0 && canonical.fY > 0.0);
        } else {
            return true;
        }
        let square = |x: f32| x * x;
        let distance = square(canonical.fX) * square(self.fRadii[index].fY)
            + square(canonical.fY) * square(self.fRadii[index].fX);
        distance <= square(self.fRadii[index].fX * self.fRadii[index].fY)
    }
}

// Source: src/core/SkRRect.cpp:351-360
impl SkRRectPriv {
    pub fn IsNearlySimpleCircular(rr: &SkRRect, tol: f32) -> bool {
        let radius = rr.fRadii[0].fX;
        crate::include::core::SkScalar::SkScalarNearlyEqual(radius, rr.fRadii[0].fY, tol)
            && crate::include::core::SkScalar::SkScalarNearlyEqual(radius, rr.fRadii[1].fX, tol)
            && crate::include::core::SkScalar::SkScalarNearlyEqual(radius, rr.fRadii[1].fY, tol)
            && crate::include::core::SkScalar::SkScalarNearlyEqual(radius, rr.fRadii[2].fX, tol)
            && crate::include::core::SkScalar::SkScalarNearlyEqual(radius, rr.fRadii[2].fY, tol)
            && crate::include::core::SkScalar::SkScalarNearlyEqual(radius, rr.fRadii[3].fX, tol)
            && crate::include::core::SkScalar::SkScalarNearlyEqual(radius, rr.fRadii[3].fY, tol)
    }
}

// Source: src/core/SkRRect.cpp:362-367
impl SkRRectPriv {
    pub fn AllCornersCircular(rr: &SkRRect, tol: f32) -> bool {
        rr.fRadii
            .iter()
            .all(|r| crate::include::core::SkScalar::SkScalarNearlyEqual(r.fX, r.fY, tol))
    }
}

// Source: src/core/SkRRect.cpp:369-380
#[allow(unexpected_cfgs)]
impl SkRRectPriv {
    pub fn IsRelativelyCircular(rx: f32, ry: f32, tol: f32) -> bool {
        let max = if rx < ry { ry } else { rx };
        #[cfg(SK_GRAPHITE_USE_LEGACY_RRECT_CLIP_SHADER)]
        {
            (rx - ry).abs() <= tol * max
        }
        #[cfg(not(SK_GRAPHITE_USE_LEGACY_RRECT_CLIP_SHADER))]
        {
            (rx <= tol && ry <= tol) || (rx - ry).abs() <= tol * max
        }
    }
}

// Source: src/core/SkRRect.cpp:382-387
impl SkRRectPriv {
    pub fn AllCornersRelativelyCircular(rr: &SkRRect, tol: f32) -> bool {
        rr.fRadii
            .iter()
            .all(|r| Self::IsRelativelyCircular(r.fX, r.fY, tol))
    }
}

// Source: src/core/SkRRect.cpp:389-404
impl SkRRect {
    pub fn containsPoint(&self, p: &SkPoint) -> bool {
        if !self.getBounds().contains_xy(p.fX, p.fY) {
            return false;
        }
        if self.isRect() {
            return true;
        }
        self.checkCornerContainment(p.fX, p.fY)
    }
}

// Source: src/core/SkRRect.cpp:406-425
impl SkRRect {
    pub fn contains(&self, r: &SkRect) -> bool {
        if !self.getBounds().contains(r) {
            return false;
        }
        if self.isRect() {
            return true;
        }
        self.checkCornerContainment(r.fLeft, r.fTop)
            && self.checkCornerContainment(r.fRight, r.fTop)
            && self.checkCornerContainment(r.fRight, r.fBottom)
            && self.checkCornerContainment(r.fLeft, r.fBottom)
    }
}

// Source: src/core/SkRRect.cpp:428-482
impl SkRRect {
    fn computeType(&mut self) {
        if self.fRect.isEmpty() {
            crate::SkASSERT!(@source b"src/core/SkRRect.cpp\0".as_ptr(),430; @condition b"fRect.isSorted()\0".as_ptr(); self.fRect.isSorted());
            debug_assert!(self.fRadii.iter().all(|r| *r == SkPoint::make(0.0, 0.0)));
            self.fType = 0;
            debug_assert!(self.isValid());
            return;
        }
        let mut all_equal = true;
        let mut all_square = self.fRadii[0].fX == 0.0 || self.fRadii[0].fY == 0.0;
        for i in 1..4 {
            if self.fRadii[i].fX != 0.0 && self.fRadii[i].fY != 0.0 {
                all_square = false;
            }
            if self.fRadii[i] != self.fRadii[i - 1] {
                all_equal = false;
            }
        }
        if all_square {
            self.fType = 1;
            debug_assert!(self.isValid());
            return;
        }
        if all_equal {
            if self.fRadii[0].fX >= self.fRect.width() / 2.0
                && self.fRadii[0].fY >= self.fRect.height() / 2.0
            {
                self.fType = 2;
                for r in &mut self.fRadii {
                    r.set(
                        SkRectPriv::HalfWidth(&self.fRect),
                        SkRectPriv::HalfHeight(&self.fRect),
                    );
                }
            } else {
                self.fType = 3;
            }
            debug_assert!(self.isValid());
            return;
        }
        self.fType = if radii_are_nine_patch(&self.fRadii) {
            4
        } else {
            5
        };
        if !self.isValid() {
            let rect = self.fRect;
            self.setRect(&rect);
            debug_assert!(self.isValid());
        }
    }
}

// Source: src/core/SkRRect.cpp:484-518
#[allow(unexpected_cfgs)]
impl SkRRect {
    pub fn transform(&self, matrix: &crate::src::core::SkMatrix::SkMatrix) -> Option<Self> {
        #[cfg(SK_SUPPORT_LEGACY_RRECT_TRANSFORM)]
        {
            let mut rr = Self::default();
            if self.transform_to(matrix, Some(&mut rr)) {
                Some(rr)
            } else {
                None
            }
        }
        #[cfg(not(SK_SUPPORT_LEGACY_RRECT_TRANSFORM))]
        {
            if matrix.isIdentity() {
                return Some(*self);
            }
            if !matrix.preservesAxisAlignment() {
                return None;
            }
            let mut rect = SkRect::MakeEmpty();
            matrix.mapRect(&mut rect, &self.fRect);
            if !rect.isFinite() {
                return None;
            }
            match self.getType() {
                Type::kEmpty_Type => return Some(Self::MakeEmpty()),
                Type::kRect_Type => return Some(Self::MakeRect(&rect)),
                Type::kOval_Type => return Some(Self::MakeOval(&rect)),
                _ => {}
            }
            let mut raw = crate::src::core::SkPathRawShapes::RRect::new(
                self,
                crate::include::core::SkPathTypes::SkPathDirection::kCW,
                6,
            );
            matrix.mapPointsInPlace(&mut raw.fStorage);
            let raw = raw.raw();
            Some(
                crate::src::core::SkPathPriv::SkPathPriv::DeduceRRectFromContour(
                    &rect,
                    raw.fPoints,
                    raw.fVerbs,
                ),
            )
        }
    }
}

// Source: src/core/SkRRect.cpp:520-660
#[allow(unexpected_cfgs)]
impl SkRRect {
    pub fn transform_to(
        &self,
        matrix: &crate::src::core::SkMatrix::SkMatrix,
        dst: Option<&mut Self>,
    ) -> bool {
        #[cfg(SK_SUPPORT_LEGACY_RRECT_TRANSFORM)]
        {
            let Some(dst) = dst else {
                return false;
            };
            debug_assert!(!core::ptr::eq(self, dst));
            if matrix.isIdentity() {
                *dst = *self;
                return true;
            }
            if !matrix.preservesAxisAlignment() {
                return false;
            }
            let mut rect = SkRect::MakeEmpty();
            if !matrix.mapRect(&mut rect, &self.fRect) {
                return false;
            }
            if !rect.isFinite() || rect.isEmpty() {
                return false;
            }
            dst.fRect = rect;
            dst.fType = self.fType;
            if self.fType == Type::kRect_Type as i32 {
                debug_assert!(dst.isValid());
                return true;
            }
            if self.fType == Type::kOval_Type as i32 {
                for r in &mut dst.fRadii {
                    r.fX = rect.width() / 2.0;
                    r.fY = rect.height() / 2.0;
                }
                debug_assert!(dst.isValid());
                return true;
            }
            let mut x_scale = matrix.getScaleX();
            let mut y_scale = matrix.getScaleY();
            if !matrix.isScaleTranslate() {
                debug_assert!(
                    matrix.getScaleX() == 0.0
                        && matrix.getScaleY() == 0.0
                        && matrix.getSkewX() != 0.0
                        && matrix.getSkewY() != 0.0
                );
                let cw = matrix.getSkewX() < 0.0;
                y_scale = matrix.getSkewY() * if cw { 1.0 } else { -1.0 };
                x_scale = matrix.getSkewX() * if cw { -1.0 } else { 1.0 };
                let dir = if cw { 3 } else { 1 };
                for i in 0..4 {
                    let src = if i + dir >= 4 { (i + dir) % 4 } else { i + dir };
                    dst.fRadii[i].fX = self.fRadii[src].fY;
                    dst.fRadii[i].fY = self.fRadii[src].fX;
                }
            } else {
                dst.fRadii = self.fRadii;
            }
            let flip_x = x_scale < 0.0;
            if flip_x {
                x_scale = -x_scale;
            }
            let flip_y = y_scale < 0.0;
            if flip_y {
                y_scale = -y_scale;
            }
            for r in &mut dst.fRadii {
                r.fX *= x_scale;
                r.fY *= y_scale;
            }
            if flip_x {
                if flip_y {
                    dst.fRadii.swap(0, 2);
                    dst.fRadii.swap(1, 3);
                } else {
                    dst.fRadii.swap(1, 0);
                    dst.fRadii.swap(2, 3);
                }
            } else if flip_y {
                dst.fRadii.swap(0, 3);
                dst.fRadii.swap(1, 2);
            }
            dst.scaleRadii();
            if !Self::AreRectAndRadiiValid(&dst.fRect, &dst.fRadii) {
                return false;
            }
            debug_assert!(dst.isValid());
            true
        }
        #[cfg(not(SK_SUPPORT_LEGACY_RRECT_TRANSFORM))]
        {
            if let Some(rr) = self.transform(matrix) {
                if let Some(dst) = dst {
                    *dst = rr;
                }
                return true;
            }
            false
        }
    }
}

// Source-aligned Skia translation. Rust methods use snake_case; fields retain upstream names.

use crate::src::core::SkRRectPriv::SkRRectPriv;
// Source: src/core/SkRRect.cpp:664-697
impl SkRRect {
    pub fn inset_to(&self, dx: f32, dy: f32, dst: &mut Self) {
        let mut r = self.fRect.makeInset(dx, dy);
        let mut degenerate = false;
        if r.fRight <= r.fLeft {
            degenerate = true;
            let mid = crate::src::core::SkFloatingPoint::sk_float_midpoint(r.fLeft, r.fRight);
            r.fLeft = mid;
            r.fRight = mid;
        }
        if r.fBottom <= r.fTop {
            degenerate = true;
            let mid = crate::src::core::SkFloatingPoint::sk_float_midpoint(r.fTop, r.fBottom);
            r.fTop = mid;
            r.fBottom = mid;
        }
        if degenerate {
            dst.fRect = r;
            dst.fRadii = [SkPoint::default(); 4];
            dst.fType = Type::kEmpty_Type as i32;
            return;
        }
        if !r.isFinite() {
            *dst = Self::default();
            return;
        }
        let mut radii = self.fRadii;
        for p in &mut radii {
            if p.fX != 0.0 {
                p.fX -= dx;
            }
            if p.fY != 0.0 {
                p.fY -= dy;
            }
        }
        dst.setRectRadii(&r, &radii);
    }
}

// Source: src/core/SkRRect.cpp:760-763
fn are_radius_check_predicates_valid(rad: f32, min: f32, max: f32) -> bool {
    (min <= max) && (rad <= max - min) && (min + rad <= max) && (max - rad >= min) && rad >= 0.0
}

// Source: src/core/SkRRect.cpp:765-846
impl SkRRect {
    pub fn isValid(&self) -> bool {
        if !Self::AreRectAndRadiiValid(&self.fRect, &self.fRadii) {
            return false;
        }
        let mut all_zero = self.fRadii[0].fX == 0.0 && self.fRadii[0].fY == 0.0;
        let mut all_square = self.fRadii[0].fX == 0.0 || self.fRadii[0].fY == 0.0;
        let mut all_same = true;
        for i in 1..4 {
            if self.fRadii[i].fX != 0.0 || self.fRadii[i].fY != 0.0 {
                all_zero = false;
            }
            if self.fRadii[i] != self.fRadii[i - 1] {
                all_same = false;
            }
            if self.fRadii[i].fX != 0.0 && self.fRadii[i].fY != 0.0 {
                all_square = false;
            }
        }
        let patches = radii_are_nine_patch(&self.fRadii);
        if self.fType < 0 || self.fType > 5 {
            return false;
        }
        match self.fType {
            0 => {
                if !self.fRect.isEmpty() || !all_zero || !all_same || !all_square {
                    return false;
                }
            }
            1 => {
                if self.fRect.isEmpty() || !all_zero || !all_same || !all_square {
                    return false;
                }
            }
            2 => {
                if self.fRect.isEmpty() || all_zero || !all_same || all_square {
                    return false;
                }
                let width = SkRectPriv::HalfWidth(&self.fRect);
                let height = SkRectPriv::HalfHeight(&self.fRect);
                let desired_width =
                    crate::src::utils::SkFloatUtils::SkFloatingPointF32::<4>::new(width);
                let desired_height =
                    crate::src::utils::SkFloatUtils::SkFloatingPointF32::<4>::new(height);
                for r in &self.fRadii {
                    let x = crate::src::utils::SkFloatUtils::SkFloatingPointF32::<4>::new(r.fX);
                    let y = crate::src::utils::SkFloatUtils::SkFloatingPointF32::<4>::new(r.fY);
                    let x_matches =
                        (r.fX - width).abs() <= 1.0 / 4096.0 || x.AlmostEquals(&desired_width);
                    let y_matches =
                        (r.fY - height).abs() <= 1.0 / 4096.0 || y.AlmostEquals(&desired_height);
                    if !x_matches || !y_matches {
                        return false;
                    }
                }
            }
            3 => {
                if self.fRect.isEmpty() || all_zero || !all_same || all_square {
                    return false;
                }
            }
            4 => {
                if self.fRect.isEmpty() || all_zero || all_same || all_square || !patches {
                    return false;
                }
            }
            5 => {
                if self.fRect.isEmpty() || all_zero || all_same || all_square || patches {
                    return false;
                }
            }
            _ => unreachable!(),
        }
        true
    }
}

// Source: src/core/SkRRect.cpp:848-859
impl SkRRect {
    fn AreRectAndRadiiValid(rect: &SkRect, radii: &[SkPoint; 4]) -> bool {
        if !rect.isFinite() || !rect.isSorted() {
            return false;
        }
        for r in radii {
            if !are_radius_check_predicates_valid(r.fX, rect.fLeft, rect.fRight)
                || !are_radius_check_predicates_valid(r.fY, rect.fTop, rect.fBottom)
            {
                return false;
            }
        }
        true
    }
}

// Source-aligned Skia translation. Rust methods use snake_case; fields retain upstream names.

// Source: src/core/SkRRect.cpp:862-922
impl SkRRectPriv {
    pub fn InnerBounds(rr: &SkRRect) -> SkRect {
        if rr.isEmpty() || rr.isRect() {
            return *rr.rect();
        }
        let mut inner = *rr.getBounds();
        let tl = rr.fRadii[0];
        let tr = rr.fRadii[1];
        let bl = rr.fRadii[3];
        let br = rr.fRadii[2];
        let max = |a: f32, b: f32| if a < b { b } else { a };
        let left = max(tl.fX, bl.fX);
        let top = max(tl.fY, tr.fY);
        let right = max(tr.fX, br.fX);
        let bottom = max(bl.fY, br.fY);
        let dw = left + right;
        let dh = top + bottom;
        let horiz = (inner.width() - dw) * inner.height();
        let vert = (inner.height() - dh) * inner.width();
        const SCALE: f32 = (1.0 - crate::include::core::SkScalar::SK_ScalarRoot2Over2) + 1e-5;
        let area = (inner.width() - SCALE * dw) * (inner.height() - SCALE * dh);
        if horiz > vert && horiz > area {
            inner.fLeft += left;
            inner.fRight -= right;
        } else if vert > area {
            inner.fTop += top;
            inner.fBottom -= bottom;
        } else if area > 0.0 {
            inner.fLeft += SCALE * left;
            inner.fRight -= SCALE * right;
            inner.fTop += SCALE * top;
            inner.fBottom -= SCALE * bottom;
        } else {
            return SkRect::MakeEmpty();
        }
        debug_assert!(inner.isSorted() && !inner.isEmpty());
        inner
    }
}

// Source: src/core/SkRRect.cpp:924-1035
impl SkRRectPriv {
    pub fn ConservativeIntersect(a: &SkRRect, b: &SkRRect) -> SkRRect {
        let corner = |r: &SkRect, c: usize| match c {
            0 => SkPoint::make(r.fLeft, r.fTop),
            1 => SkPoint::make(r.fRight, r.fTop),
            2 => SkPoint::make(r.fRight, r.fBottom),
            3 => SkPoint::make(r.fLeft, r.fBottom),
            _ => unreachable!(),
        };
        let inside = |c: usize, a: SkPoint, b: SkPoint| match c {
            0 => a.fX >= b.fX && a.fY >= b.fY,
            1 => a.fX <= b.fX && a.fY >= b.fY,
            2 => a.fX <= b.fX && a.fY <= b.fY,
            3 => a.fX >= b.fX && a.fY <= b.fY,
            _ => unreachable!(),
        };
        let radii = |r: &SkRect, c: usize, radius: &mut SkPoint| {
            let test = corner(r, c);
            let ac = corner(a.rect(), c);
            let bc = corner(b.rect(), c);
            if test == ac && test == bc {
                let ar = a.fRadii[c];
                let br = b.fRadii[c];
                if ar.fX >= br.fX && ar.fY >= br.fY {
                    *radius = ar;
                    true
                } else if br.fX >= ar.fX && br.fY >= ar.fY {
                    *radius = br;
                    true
                } else {
                    false
                }
            } else if test == ac {
                *radius = a.fRadii[c];
                if *radius == b.fRadii[c] {
                    inside(c, ac, bc)
                } else {
                    b.checkCornerContainment(ac.fX, ac.fY)
                }
            } else if test == bc {
                *radius = b.fRadii[c];
                if *radius == a.fRadii[c] {
                    inside(c, bc, ac)
                } else {
                    a.checkCornerContainment(bc.fX, bc.fY)
                }
            } else {
                *radius = SkPoint::make(0.0, 0.0);
                a.checkCornerContainment(test.fX, test.fY)
                    && b.checkCornerContainment(test.fX, test.fY)
            }
        };
        let mut intersection = SkRRect::default();
        if !intersection.fRect.intersect_ab(*a.rect(), *b.rect()) {
            return SkRRect::MakeEmpty();
        }
        for c in [0usize, 1, 2, 3] {
            if !radii(&intersection.fRect, c, &mut intersection.fRadii[c]) {
                return SkRRect::MakeEmpty();
            }
        }
        if !SkRRect::AreRectAndRadiiValid(&intersection.fRect, &intersection.fRadii)
            || intersection.scaleRadii()
        {
            return SkRRect::MakeEmpty();
        }
        intersection.computeType();
        intersection
    }
}
// Source: include/core/SkRRect.h:459-459
impl SkRRect {
    pub const kSizeInMemory: usize = 12 * core::mem::size_of::<f32>();
}
// Source: src/core/SkRRect.cpp:701-705
impl SkRRect {
    pub fn writeToMemory(&self, buffer: &mut [u8]) -> usize {
        assert!(buffer.len() >= Self::kSizeInMemory);
        // The original copies only the four bounds and eight radius scalars, excluding cached fType.
        let values = [
            self.fRect.fLeft,
            self.fRect.fTop,
            self.fRect.fRight,
            self.fRect.fBottom,
            self.fRadii[0].fX,
            self.fRadii[0].fY,
            self.fRadii[1].fX,
            self.fRadii[1].fY,
            self.fRadii[2].fX,
            self.fRadii[2].fY,
            self.fRadii[3].fX,
            self.fRadii[3].fY,
        ];
        for (i, v) in values.iter().enumerate() {
            buffer[i * 4..i * 4 + 4].copy_from_slice(&v.to_ne_bytes());
        }
        Self::kSizeInMemory
    }
}
// Source: src/core/SkRRect.cpp:707-710
impl crate::src::core::SkRRectPriv::SkRRectPriv {
    /// Safety: the writer retains its exclusive backing-storage lease for this write.
    pub unsafe fn WriteToBuffer(rr: &SkRRect, buffer: &mut crate::src::core::SkBuffer::SkWBuffer) {
        // Rust's object layout is not the native struct layout. Use the actual
        // existing scalar wire encoder, copying the same bounds/radii bytes only.
        let mut bytes = [0u8; SkRRect::kSizeInMemory];
        rr.writeToMemory(&mut bytes);
        unsafe {
            buffer.write(bytes.as_ptr(), SkRRect::kSizeInMemory);
        }
    }
}
// Source: src/core/SkRRect.cpp:712-723
impl SkRRect {
    pub fn readFromMemory(&mut self, buffer: &[u8]) -> usize {
        if buffer.len() < Self::kSizeInMemory {
            return 0;
        }
        let raw: [f32; 12] = core::array::from_fn(|i| {
            f32::from_ne_bytes(buffer[i * 4..i * 4 + 4].try_into().unwrap())
        });
        let rect = SkRect::MakeLTRB(raw[0], raw[1], raw[2], raw[3]);
        let radii = core::array::from_fn(|i| SkPoint::make(raw[4 + i * 2], raw[5 + i * 2]));
        self.setRectRadii(&rect, &radii);
        Self::kSizeInMemory
    }
}

// Source: src/core/SkRRect.cpp:725-732
impl crate::src::core::SkRRectPriv::SkRRectPriv {
    /// Safety: reader backing remains readable through this call.
    pub unsafe fn ReadFromBuffer(
        buffer: &mut crate::src::core::SkBuffer::SkRBuffer,
        rr: &mut SkRRect,
    ) -> bool {
        if buffer.available() < SkRRect::kSizeInMemory {
            return false;
        }
        // Native storage contains only the wire prefix, never a valid cached type.
        // Keep it as raw initialized bytes until the actual decoder sets the radii/type.
        let mut storage = [0u8; SkRRect::kSizeInMemory];
        let read = unsafe { buffer.read(storage.as_mut_ptr(), SkRRect::kSizeInMemory) };
        read && rr.readFromMemory(&storage) == SkRRect::kSizeInMemory
    }
}

// Source: include/core/SkRRect.h:500-501
// Source: src/core/SkRRect.cpp:734-751
impl SkRRect {
    pub fn dumpToString(&self, as_hex: bool) -> crate::src::core::SkString::SkString {
        use crate::src::core::{
            SkString::SkString,
            SkStringUtils::{SkAppendScalar, SkScalarAsStringType::*},
        };
        let as_type = if as_hex {
            kHex_SkScalarAsStringType
        } else {
            kDec_SkScalarAsStringType
        };
        let mut line = self.fRect.dumpToString(as_hex);
        unsafe {
            line.appendf(b"\nconst SkPoint corners[] = {\n\0".as_ptr());
        }
        for i in 0..4 {
            let (mut x, mut y) = (SkString::default(), SkString::default());
            SkAppendScalar(&mut x, self.fRadii[i].fX, as_type);
            SkAppendScalar(&mut y, self.fRadii[i].fY, as_type);
            unsafe {
                line.appendf(b"    { %s, %s },\0".as_ptr(), x.c_str(), y.c_str());
            }
            if as_hex {
                unsafe {
                    line.appendf(
                        b" /* %f %f */\0".as_ptr(),
                        self.fRadii[i].fX as f64,
                        self.fRadii[i].fY as f64,
                    );
                }
            }
            line.append_view(b"\n");
        }
        line.append_view(b"};");
        line
    }
    // Source: src/core/SkRRect.cpp:753-753
    pub fn dump_as_hex(&self, as_hex: bool) {
        let line = self.dumpToString(as_hex);
        unsafe {
            crate::SkDebugf!(@source b"src/core/SkRRect.cpp\0".as_ptr(),753; b"%s\n\0".as_ptr(),line.c_str());
        }
    }
    // Source: include/core/SkRRect.h:508-508
    pub fn dump(&self) {
        self.dump_as_hex(false);
    }
    // Source: include/core/SkRRect.h:515-515
    pub fn dumpHex(&self) {
        self.dump_as_hex(true);
    }
}
