// Translation status: complete (transform/isValid dependency closure only)
// Copyright 2012 Google Inc. BSD-style license in LICENSE.
//! Source SkRRect value closure used by public rounded-rectangle transforms.
//! Raw LTRB and full 3x3 matrix values preserve empty/degenerate/perspective
//! states excluded by the preexisting tiny-skia Rect/affine Matrix ABI.
#![allow(non_camel_case_types, non_upper_case_globals, dead_code)]

// Source: include/core/SkRect.h:594-607,705-727,1374-1382.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct SkRRectBounds {
    pub left: f32,
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
}
impl SkRRectBounds {
    pub const fn MakeLTRB(left: f32, top: f32, right: f32, bottom: f32) -> Self {
        Self {
            left,
            top,
            right,
            bottom,
        }
    }
    fn isFinite(self) -> bool {
        [self.left, self.top, self.right, self.bottom]
            .iter()
            .all(|v| v.is_finite())
    }
    fn isEmpty(self) -> bool {
        !(self.left < self.right && self.top < self.bottom)
    }
    fn isSorted(self) -> bool {
        self.left <= self.right && self.top <= self.bottom
    }
    fn width(self) -> f32 {
        self.right - self.left
    }
    fn height(self) -> f32 {
        self.bottom - self.top
    }
    fn sorted(self) -> Self {
        Self::MakeLTRB(
            cpp_min(self.left, self.right),
            cpp_min(self.top, self.bottom),
            cpp_max(self.left, self.right),
            cpp_max(self.top, self.bottom),
        )
    }
    // Source: src/core/SkRectPriv.h:61-68; include/private/SkFloatingPoint.h:145-149.
    fn halfWidth(self) -> f32 {
        (0.5 * (-(self.left as f64) + self.right as f64)) as f32
    }
    fn halfHeight(self) -> f32 {
        (0.5 * (-(self.top as f64) + self.bottom as f64)) as f32
    }
}
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct SkRRectVector {
    pub x: f32,
    pub y: f32,
}
impl SkRRectVector {
    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
}
// Source: include/core/SkRRect.h:69-80,536-541.
#[repr(i32)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Type {
    #[default]
    kEmpty_Type = 0,
    kRect_Type = 1,
    kOval_Type = 2,
    kSimple_Type = 3,
    kNinePatch_Type = 4,
    kComplex_Type = 5,
}
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct SkRRect {
    rect: SkRRectBounds,
    radii: [SkRRectVector; 4],
    kind: Type,
}
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
// Source: src/core/SkRRect.cpp:97-126.
fn clamp_to_zero(r: &mut [SkRRectVector; 4]) -> bool {
    let mut all = true;
    for v in r {
        if v.x <= 0. || v.y <= 0. {
            v.x = 0.;
            v.y = 0.;
        } else {
            all = false;
        }
    }
    all
}
fn radii_are_nine_patch(r: &[SkRRectVector; 4]) -> bool {
    r[0].x == r[3].x && r[0].y == r[1].y && r[1].x == r[2].x && r[3].y == r[2].y
}
// Source: src/core/SkRRect.cpp:189-195,239-250.
fn compute_min_scale(a: f64, b: f64, limit: f64, current: f64) -> f64 {
    if a + b > limit {
        current.min(limit / (a + b))
    } else {
        current
    }
}
fn flush_to_zero(a: &mut f32, b: &mut f32) {
    debug_assert!(*a >= 0. && *b >= 0.);
    if *a + *b == *a {
        *b = 0.;
    } else if *a + *b == *b {
        *a = 0.;
    }
}
// Source: src/core/SkScaleToSides.h:23-70.
fn adjust_radii(limit: f64, scale: f64, a: &mut f32, b: &mut f32) {
    debug_assert!(scale < 1. && scale > 0.);
    *a = (*a as f64 * scale) as f32;
    *b = (*b as f64 * scale) as f32;
    if (*a + *b) as f64 > limit {
        let (min, max) = if *a > *b { (b, a) } else { (a, b) };
        let new_min = *min;
        let mut new_max = (limit - new_min as f64) as f32;
        while (new_max + new_min) as f64 > limit {
            new_max = if new_max > 0. {
                new_max.next_down()
            } else if new_max < 0. {
                new_max.next_up()
            } else {
                0.
            };
        }
        *max = new_max;
    }
}
// Source: src/core/SkRRect.cpp:759-763; src/utils/SkFloatUtils.h:55-169.
fn radius_valid(rad: f32, min: f32, max: f32) -> bool {
    min <= max && rad <= max - min && min + rad <= max && max - rad >= min && rad >= 0.
}
fn almost_equals(a: f32, b: f32) -> bool {
    if a.is_nan() || b.is_nan() {
        return false;
    }
    let biased = |v: f32| {
        let bits = v.to_bits();
        if bits & 0x80000000 != 0 {
            (!bits).wrapping_add(1)
        } else {
            bits | 0x80000000
        }
    };
    biased(a).abs_diff(biased(b)) <= 4
}
impl SkRRect {
    // Source: include/core/SkRRect.h:416-418. Translating the bounds must not
    // reclassify the shape or recompute/clamp its radii.
    pub fn offset(&mut self, dx: f32, dy: f32) {
        self.rect.left += dx;
        self.rect.right += dx;
        self.rect.top += dy;
        self.rect.bottom += dy;
    }
    pub fn getBounds(&self) -> SkRRectBounds {
        self.rect
    }
    pub fn radii(&self) -> &[SkRRectVector; 4] {
        &self.radii
    }
    pub fn getType(&self) -> Type {
        debug_assert!(self.isValid());
        self.kind
    }
    // Source: src/core/SkRRect.cpp:223-237.
    fn initializeRect(&mut self, rect: SkRRectBounds) -> bool {
        if !rect.isFinite() {
            *self = Self::default();
            return false;
        }
        self.rect = rect.sorted();
        if self.rect.isEmpty() {
            self.radii = [SkRRectVector::default(); 4];
            self.kind = Type::kEmpty_Type;
            return false;
        }
        true
    }
    // Source: include/core/SkRRect.h:130-137.
    pub fn setRect(&mut self, rect: SkRRectBounds) {
        if !self.initializeRect(rect) {
            return;
        }
        self.radii = [SkRRectVector::default(); 4];
        self.kind = Type::kRect_Type;
        debug_assert!(self.isValid());
    }
    // Source: src/core/SkRRect.cpp:33-53.
    pub fn setOval(&mut self, rect: SkRRectBounds) {
        if !self.initializeRect(rect) {
            return;
        }
        let x = self.rect.halfWidth();
        let y = self.rect.halfHeight();
        if x == 0. || y == 0. {
            self.radii = [SkRRectVector::default(); 4];
            self.kind = Type::kRect_Type;
        } else {
            self.radii = [SkRRectVector::new(x, y); 4];
            self.kind = Type::kOval_Type;
        }
        debug_assert!(self.isValid());
    }
    // Source: src/core/SkRRect.cpp:197-221.
    pub fn setRectRadii(&mut self, rect: SkRRectBounds, radii: [SkRRectVector; 4]) {
        if !self.initializeRect(rect) {
            return;
        }
        if !radii.iter().all(|r| r.x.is_finite() && r.y.is_finite()) {
            self.setRect(rect);
            return;
        }
        self.radii = radii;
        if clamp_to_zero(&mut self.radii) {
            self.setRect(rect);
            return;
        }
        self.scaleRadii();
        if !self.isValid() {
            self.setRect(rect);
        }
    }
    // Source: src/core/SkRRect.cpp:252-302.
    fn scaleRadii(&mut self) -> bool {
        let mut scale = 1.;
        let width = self.rect.right as f64 - self.rect.left as f64;
        let height = self.rect.bottom as f64 - self.rect.top as f64;
        scale = compute_min_scale(self.radii[0].x as f64, self.radii[1].x as f64, width, scale);
        scale = compute_min_scale(
            self.radii[1].y as f64,
            self.radii[2].y as f64,
            height,
            scale,
        );
        scale = compute_min_scale(self.radii[2].x as f64, self.radii[3].x as f64, width, scale);
        scale = compute_min_scale(
            self.radii[3].y as f64,
            self.radii[0].y as f64,
            height,
            scale,
        );
        // These are actual disjoint scalar fields of the same canonical owner.
        let [a, b, c, d] = &mut self.radii;
        flush_to_zero(&mut a.x, &mut b.x);
        flush_to_zero(&mut b.y, &mut c.y);
        flush_to_zero(&mut c.x, &mut d.x);
        flush_to_zero(&mut d.y, &mut a.y);
        if scale < 1. {
            adjust_radii(width, scale, &mut a.x, &mut b.x);
            adjust_radii(height, scale, &mut b.y, &mut c.y);
            adjust_radii(width, scale, &mut c.x, &mut d.x);
            adjust_radii(height, scale, &mut d.y, &mut a.y);
        }
        clamp_to_zero(&mut self.radii);
        self.computeType();
        scale < 1.
    }
    // Source: src/core/SkRRect.cpp:421-489.
    fn computeType(&mut self) {
        if self.rect.isEmpty() {
            debug_assert!(self.rect.isSorted());
            debug_assert!(self.radii.iter().all(|r| *r == SkRRectVector::default()));
            self.kind = Type::kEmpty_Type;
            debug_assert!(self.isValid());
            return;
        }
        let mut equal = true;
        let mut square = self.radii[0].x == 0. || self.radii[0].y == 0.;
        for i in 1..4 {
            if self.radii[i].x != 0. && self.radii[i].y != 0. {
                square = false;
            }
            if self.radii[i] != self.radii[i - 1] {
                equal = false;
            }
        }
        if square {
            self.kind = Type::kRect_Type;
            debug_assert!(self.isValid());
            return;
        }
        if equal {
            if self.radii[0].x >= self.rect.width() / 2.
                && self.radii[0].y >= self.rect.height() / 2.
            {
                self.kind = Type::kOval_Type;
                self.radii = [SkRRectVector::new(self.rect.halfWidth(), self.rect.halfHeight()); 4];
            } else {
                self.kind = Type::kSimple_Type;
            }
            debug_assert!(self.isValid());
            return;
        }
        self.kind = if radii_are_nine_patch(&self.radii) {
            Type::kNinePatch_Type
        } else {
            Type::kComplex_Type
        };
        if !self.isValid() {
            self.setRect(self.rect);
            debug_assert!(self.isValid());
        }
    }
    // Source: src/core/SkRRect.cpp:765-860.
    pub fn isValid(&self) -> bool {
        if !Self::AreRectAndRadiiValid(self.rect, &self.radii) {
            return false;
        }
        let mut zero = self.radii[0].x == 0. && self.radii[0].y == 0.;
        let mut square = self.radii[0].x == 0. || self.radii[0].y == 0.;
        let mut same = true;
        for i in 1..4 {
            if self.radii[i].x != 0. || self.radii[i].y != 0. {
                zero = false;
            }
            if self.radii[i] != self.radii[i - 1] {
                same = false;
            }
            if self.radii[i].x != 0. && self.radii[i].y != 0. {
                square = false;
            }
        }
        let nine = radii_are_nine_patch(&self.radii);
        match self.kind {
            Type::kEmpty_Type => {
                if !self.rect.isEmpty() || !zero || !same || !square {
                    return false;
                }
            }
            Type::kRect_Type => {
                if self.rect.isEmpty() || !zero || !same || !square {
                    return false;
                }
            }
            Type::kOval_Type => {
                if self.rect.isEmpty() || zero || !same || square {
                    return false;
                }
                let x = self.rect.halfWidth();
                let y = self.rect.halfHeight();
                for r in &self.radii {
                    if !((r.x - x).abs() <= 1. / 4096. || almost_equals(r.x, x))
                        || !((r.y - y).abs() <= 1. / 4096. || almost_equals(r.y, y))
                    {
                        return false;
                    }
                }
            }
            Type::kSimple_Type => {
                if self.rect.isEmpty() || zero || !same || square {
                    return false;
                }
            }
            Type::kNinePatch_Type => {
                if self.rect.isEmpty() || zero || same || square || !nine {
                    return false;
                }
            }
            Type::kComplex_Type => {
                if self.rect.isEmpty() || zero || same || square || nine {
                    return false;
                }
            }
        }
        true
    }
    fn AreRectAndRadiiValid(rect: SkRRectBounds, radii: &[SkRRectVector; 4]) -> bool {
        if !rect.isFinite() || !rect.isSorted() {
            return false;
        }
        for r in radii {
            if !radius_valid(r.x, rect.left, rect.right)
                || !radius_valid(r.y, rect.top, rect.bottom)
            {
                return false;
            }
        }
        true
    }
}

// Source: include/core/SkMatrix.h:424-440; src/core/SkMatrix.cpp:101-163.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SkRRectMatrix {
    pub values: [f32; 9],
}
impl SkRRectMatrix {
    pub const fn MakeAll(values: [f32; 9]) -> Self {
        Self { values }
    }
    fn typeMask(&self) -> u8 {
        let m = &self.values;
        let mut mask = 0;
        if m[6] != 0. || m[7] != 0. || m[8] != 1. {
            return 15;
        }
        if m[2] != 0. || m[5] != 0. {
            mask |= 1;
        }
        let bits = crate::include::private::SkFloatingPoint::f32_as_2s_compliment;
        let (m00, m01, m10, m11) = (bits(m[0]), bits(m[1]), bits(m[3]), bits(m[4]));
        if m01 | m10 != 0 {
            mask |= 6;
            if m00 | m11 == 0 && m01 != 0 && m10 != 0 {
                mask |= 16;
            }
        } else {
            if (m00 ^ (1f32.to_bits() as i32)) | (m11 ^ (1f32.to_bits() as i32)) != 0 {
                mask |= 2;
            }
            if m00 != 0 && m11 != 0 {
                mask |= 16;
            }
        }
        mask
    }
    fn isIdentity(&self) -> bool {
        self.typeMask() & 15 == 0
    }
    fn preservesAxisAlignment(&self) -> bool {
        self.typeMask() & 16 != 0
    }
    // Source: src/core/SkMatrix.cpp:895-961,988-1018.
    fn mapPoints(&self, points: &mut [SkRRectVector]) {
        let mask = self.typeMask() & 15;
        let m = &self.values;
        if mask == 0 {
            return;
        }
        for p in points {
            let (x, y) = (p.x, p.y);
            if mask == 1 {
                p.x = x + m[2];
                p.y = y + m[5];
            } else if mask <= 3 {
                p.x = x * m[0] + m[2];
                p.y = y * m[4] + m[5];
            } else {
                debug_assert_eq!(mask & 8, 0);
                p.x = (x * m[0] + y * m[1]) + m[2];
                p.y = (y * m[4] + x * m[3]) + m[5];
            }
        }
    }
    // Source: src/core/SkMatrix.cpp:1122-1173.
    fn mapRect(&self, rect: SkRRectBounds) -> SkRRectBounds {
        let m = &self.values;
        let mask = self.typeMask() & 15;
        if mask <= 3 {
            let mut r = [rect.left, rect.top, rect.right, rect.bottom];
            if mask <= 1 {
                r[0] += m[2];
                r[1] += m[5];
                r[2] += m[2];
                r[3] += m[5];
            } else {
                r[0] = r[0] * m[0] + m[2];
                r[1] = r[1] * m[4] + m[5];
                r[2] = r[2] * m[0] + m[2];
                r[3] = r[3] * m[4] + m[5];
            }
            use crate::src::core::SkVx::skvx::float4;
            let ltrb = float4::from(r);
            let reverse = float4::from([r[2], r[3], r[0], r[1]]);
            let minimum: [f32; 4] = ltrb.min(reverse).into();
            let maximum: [f32; 4] = ltrb.max(reverse).into();
            return SkRRectBounds::MakeLTRB(minimum[2], minimum[3], maximum[0], maximum[1]);
        }

        // Perspective/other non-axis-aligned matrices cannot reach this private
        // call: the canonical transform checks preservesAxisAlignment first.
        debug_assert!(self.preservesAxisAlignment());
        let mut quad = [
            SkRRectVector::new(rect.left, rect.top),
            SkRRectVector::new(rect.right, rect.top),
            SkRRectVector::new(rect.right, rect.bottom),
            SkRRectVector::new(rect.left, rect.bottom),
        ];
        self.mapPoints(&mut quad);
        // Source: src/core/SkRect.cpp:49-111,131-137.
        // Native Bounds detects infinity as well as NaN by accumulated 0*x.
        let mut r = SkRRectBounds::MakeLTRB(quad[0].x, quad[0].y, quad[0].x, quad[0].y);
        let (mut nx, mut ny) = (0f32, 0f32);
        for p in &quad {
            r.left = p.x.min(r.left);
            r.top = p.y.min(r.top);
            r.right = p.x.max(r.right);
            r.bottom = p.y.max(r.bottom);
            nx *= p.x;
            ny *= p.y;
        }
        if !(nx == 0. && ny == 0.) {
            return SkRRectBounds::MakeLTRB(f32::NAN, f32::NAN, f32::NAN, f32::NAN);
        }

        r
    }
}
// Source: src/core/SkPathMakers.h:15-48,66-90.
pub(crate) struct PointIterator<const N: usize> {
    points: [SkRRectVector; N],
    current: usize,
    advance: usize,
}
impl<const N: usize> PointIterator<N> {
    pub(crate) fn current(&self) -> SkRRectVector {
        self.points[self.current]
    }
    pub(crate) fn next(&mut self) -> SkRRectVector {
        self.current = (self.current + self.advance) % N;
        self.current()
    }
}
// Source: src/core/SkPathMakers.h:38-64. Shared by native Rect/Oval/RRect makers.
impl PointIterator<4> {
    pub(crate) fn rect(
        b: SkRRectBounds,
        dir: crate::include::core::SkPathTypes::SkPathDirection,
        index: usize,
    ) -> Self {
        Self {
            points: [
                SkRRectVector::new(b.left, b.top),
                SkRRectVector::new(b.right, b.top),
                SkRRectVector::new(b.right, b.bottom),
                SkRRectVector::new(b.left, b.bottom),
            ],
            current: index % 4,
            advance: if dir == crate::include::core::SkPathTypes::SkPathDirection::kCW {
                1
            } else {
                3
            },
        }
    }
    pub(crate) fn oval(
        b: SkRRectBounds,
        dir: crate::include::core::SkPathTypes::SkPathDirection,
        index: usize,
    ) -> Self {
        let x = (0.5 * (b.left as f64 + b.right as f64)) as f32;
        let y = (0.5 * (b.top as f64 + b.bottom as f64)) as f32;
        Self {
            points: [
                SkRRectVector::new(x, b.top),
                SkRRectVector::new(b.right, y),
                SkRRectVector::new(x, b.bottom),
                SkRRectVector::new(b.left, y),
            ],
            current: index % 4,
            advance: if dir == crate::include::core::SkPathTypes::SkPathDirection::kCW {
                1
            } else {
                3
            },
        }
    }
}
// Source: src/core/SkPathRawShapes.cpp:90-158,173-185; SkPathRawShapes.h:38-45.
pub(crate) use crate::include::core::SkPathTypes::SkPathVerb as Verb;
use Verb::{kClose as Close, kConic as Conic, kLine as Line, kMove as Move};
pub(crate) const LINE_START: [Verb; 10] = [
    Move, Line, Conic, Line, Conic, Line, Conic, Line, Conic, Close,
];
pub(crate) const CONIC_START: [Verb; 9] =
    [Move, Conic, Line, Conic, Line, Conic, Line, Conic, Close];
pub(crate) struct RawRRect {
    pub(crate) storage: [SkRRectVector; 13],
    pub(crate) point_count: usize,
    pub(crate) starts_conic: bool,
}
impl RawRRect {
    fn new(rr: &SkRRect) -> Self {
        Self::with_direction(rr, true, 6)
    }
    pub(crate) fn with_direction(rr: &SkRRect, cw: bool, index: usize) -> Self {
        let starts_conic = (index & 1) == usize::from(cw);
        let point_count = 13 - usize::from(starts_conic);
        let b = rr.rect;
        let r = &rr.radii;
        let mut riter = PointIterator {
            points: [
                SkRRectVector::new(b.left + r[0].x, b.top),
                SkRRectVector::new(b.right - r[1].x, b.top),
                SkRRectVector::new(b.right, b.top + r[1].y),
                SkRRectVector::new(b.right, b.bottom - r[2].y),
                SkRRectVector::new(b.right - r[2].x, b.bottom),
                SkRRectVector::new(b.left + r[3].x, b.bottom),
                SkRRectVector::new(b.left, b.bottom - r[3].y),
                SkRRectVector::new(b.left, b.top + r[0].y),
            ],
            current: index % 8,
            advance: if cw { 1 } else { 7 },
        };
        let mut rectiter = PointIterator {
            points: [
                SkRRectVector::new(b.left, b.top),
                SkRRectVector::new(b.right, b.top),
                SkRRectVector::new(b.right, b.bottom),
                SkRRectVector::new(b.left, b.bottom),
            ],
            current: (index / 2 + usize::from(!cw)) % 4,
            advance: if cw { 1 } else { 3 },
        };
        let mut storage = [SkRRectVector::default(); 13];
        storage[0] = riter.current();
        if starts_conic {
            for i in 0..3 {
                storage[i * 3 + 1] = rectiter.next();
                storage[i * 3 + 2] = riter.next();
                storage[i * 3 + 3] = riter.next();
            }
            storage[10] = rectiter.next();
            storage[11] = riter.next();
        } else {
            for i in 0..4 {
                storage[i * 3 + 1] = riter.next();
                storage[i * 3 + 2] = rectiter.next();
                storage[i * 3 + 3] = riter.next();
            }
        }
        Self {
            storage,
            point_count,
            starts_conic,
        }
    }
}
// Source: src/core/SkPathPriv.cpp:1614-1675.
pub(crate) fn deduce_rrect(
    bounds: SkRRectBounds,
    pts: &[SkRRectVector],
    verbs: &[Verb],
) -> SkRRect {
    let mut radii = [SkRRectVector::default(); 4];
    let mut p = 0;
    for verb in verbs {
        match verb {
            Verb::kMove => {
                debug_assert_eq!(p, 0);
                p += 1;
            }
            Verb::kLine => {
                debug_assert!(pts[p].x - pts[p - 1].x == 0. || pts[p].y - pts[p - 1].y == 0.);
                p += 1;
            }
            Verb::kConic => {
                let a = SkRRectVector::new(pts[p].x - pts[p - 1].x, pts[p].y - pts[p - 1].y);
                let b = SkRRectVector::new(pts[p + 1].x - pts[p].x, pts[p + 1].y - pts[p].y);
                let d;
                if a.x != 0. {
                    debug_assert!(b.x == 0. && a.y == 0.);
                    d = SkRRectVector::new(a.x.abs(), b.y.abs());
                } else if a.y == 0. {
                    debug_assert!(b.x == 0. || b.y == 0.);
                    d = SkRRectVector::new(b.x.abs(), b.y.abs());
                } else {
                    debug_assert_eq!(b.y, 0.);
                    d = SkRRectVector::new(b.x.abs(), a.y.abs());
                }
                let corner = if pts[p].x == bounds.left {
                    if pts[p].y == bounds.top {
                        0
                    } else {
                        3
                    }
                } else if pts[p].y == bounds.top {
                    1
                } else {
                    2
                };
                debug_assert_eq!(radii[corner], SkRRectVector::default());
                radii[corner] = d;
                p += 2;
            }
            Verb::kClose => {}
            Verb::kQuad | Verb::kCubic => {
                debug_assert!(false, "unexpected RRect curve");
            }
        }
    }
    let mut rr = SkRRect::default();
    rr.setRectRadii(bounds, radii);
    rr
}
// Source: src/core/SkRRect.cpp:491-518,653-664.
// This is the current upstream nonlegacy branch. No legacy radius-scaling
// approximation is used; radii are deduced from the transformed conic points.
impl SkRRect {
    pub fn transform(&self, matrix: &SkRRectMatrix) -> Option<Self> {
        if matrix.isIdentity() {
            return Some(*self);
        }
        if !matrix.preservesAxisAlignment() {
            return None;
        }
        let new_rect = matrix.mapRect(self.rect);
        if !new_rect.isFinite() {
            return None;
        }
        let mut result = Self::default();
        match self.getType() {
            Type::kEmpty_Type => return Some(result),
            Type::kRect_Type => {
                result.setRect(new_rect);
                return Some(result);
            }
            Type::kOval_Type => {
                result.setOval(new_rect);
                return Some(result);
            }
            _ => {}
        }
        let mut raw = RawRRect::new(self);
        matrix.mapPoints(&mut raw.storage);
        Some(deduce_rrect(new_rect, &raw.storage, &LINE_START))
    }
    pub fn transform_into(&self, matrix: &SkRRectMatrix, dst: Option<&mut Self>) -> bool {
        if let Some(rr) = self.transform(matrix) {
            if let Some(dst) = dst {
                *dst = rr;
            }
            return true;
        }
        false
    }
}

// Source: src/core/SkMatrix.cpp:883-892,1107-1120.
impl SkRRectMatrix {
    pub fn mapVector(&self, vector: SkRRectVector) -> SkRRectVector {
        if self.typeMask() & 8 != 0 {
            let map = |p: SkRRectVector| {
                let m = &self.values;
                let x = (p.x * m[0] + p.y * m[1]) + m[2];
                let y = (p.x * m[3] + p.y * m[4]) + m[5];
                let mut z = (p.x * m[6] + p.y * m[7]) + m[8];
                if z != 0. {
                    z = 1. / z;
                }
                SkRRectVector::new(x * z, y * z)
            };
            let origin = map(SkRRectVector::default());
            let result = map(vector);
            SkRRectVector::new(result.x - origin.x, result.y - origin.y)
        } else {
            let mut tmp = *self;
            tmp.values[2] = 0.;
            tmp.values[5] = 0.;
            let mut vectors = [vector];
            tmp.mapPoints(&mut vectors);
            vectors[0]
        }
    }
}
pub type SkMatrix3x3 = SkRRectMatrix;

impl SkRRect {
    // Source: src/core/SkRRect.cpp:406-425.
    pub fn contains(&self, r: SkRRectBounds) -> bool {
        let b = self.rect;
        if r.isEmpty()
            || b.left > r.left
            || b.top > r.top
            || b.right < r.right
            || b.bottom < r.bottom
        {
            return false;
        }
        if self.kind == Type::kRect_Type {
            return true;
        }
        self.checkCornerContainment(r.left, r.top)
            && self.checkCornerContainment(r.right, r.top)
            && self.checkCornerContainment(r.right, r.bottom)
            && self.checkCornerContainment(r.left, r.bottom)
    }
    // Source: src/core/SkRRect.cpp:298-344.
    fn checkCornerContainment(&self, x: f32, y: f32) -> bool {
        let r = self.rect;
        let f = &self.radii;
        let (index, px, py) = if self.kind == Type::kOval_Type {
            (
                0,
                x - (0.5 * (r.left as f64 + r.right as f64)) as f32,
                y - (0.5 * (r.top as f64 + r.bottom as f64)) as f32,
            )
        } else if x < r.left + f[0].x && y < r.top + f[0].y {
            (0, x - (r.left + f[0].x), y - (r.top + f[0].y))
        } else if x < r.left + f[3].x && y > r.bottom - f[3].y {
            (3, x - (r.left + f[3].x), y - (r.bottom - f[3].y))
        } else if x > r.right - f[1].x && y < r.top + f[1].y {
            (1, x - (r.right - f[1].x), y - (r.top + f[1].y))
        } else if x > r.right - f[2].x && y > r.bottom - f[2].y {
            (2, x - (r.right - f[2].x), y - (r.bottom - f[2].y))
        } else {
            return true;
        };
        let radius = f[index];
        let dist = (px * px) * (radius.y * radius.y) + (py * py) * (radius.x * radius.x);
        let product = radius.x * radius.y;
        dist <= product * product
    }
}
