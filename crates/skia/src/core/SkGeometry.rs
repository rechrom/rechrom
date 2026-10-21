use crate::src::core::SkPoint::SkPoint;

impl SkAutoConicToQuads {
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }
    #[inline]
    fn reset_storage(&mut self, count: usize) -> &mut [SkPoint] {
        // Rust owner preserves the original 17-point inline threshold and exact heap extent.
        if count <= self.inline.len() {
            self.heap = None;
            &mut self.inline[..count]
        } else {
            self.heap = Some(vec![SkPoint::default(); count].into_boxed_slice());
            self.heap.as_deref_mut().unwrap()
        }
    }
    #[inline]
    fn storage(&self) -> &[SkPoint] {
        self.heap.as_deref().unwrap_or(&self.inline)
    }
}

fn mul_lanes(a: SkPoint, b: SkPoint) -> SkPoint {
    SkPoint::make(a.fX * b.fX, a.fY * b.fY)
}

fn broadcast(t: f32) -> SkPoint {
    SkPoint::make(t, t)
}

// Source: src/core/SkGeometry.h:24-26
pub fn from_point(point: &SkPoint) -> [f32; 2] {
    [point.fX, point.fY]
}

// Source: src/core/SkGeometry.h:28-32
pub fn to_point(x: [f32; 2]) -> SkPoint {
    SkPoint::make(x[0], x[1])
}

// Source: src/core/SkGeometry.h:34-36
#[inline]
fn times_2(value: SkPoint) -> SkPoint {
    value + value
}

// Source: src/core/SkGeometry.h:81-83
pub fn SkMeasureQuadRotation(pts: &[SkPoint; 3]) -> f32 {
    SkMeasureAngleBetweenVectors(pts[1] - pts[0], pts[2] - pts[1])
}

// Source: src/core/SkGeometry.h:93-95
pub fn SkChopQuadAtMidTangent(src: &[SkPoint; 3], dst: &mut [SkPoint; 5]) {
    SkChopQuadAt(src, dst, SkFindQuadMidTangent(src));
}

// Source: src/core/SkGeometry.h:197-199
pub fn SkChopCubicAtMidTangent(src: &[SkPoint; 4], dst: &mut [SkPoint; 7]) {
    SkChopCubicAt(src, dst, SkFindCubicMidTangent(src));
}

// Source-aligned Skia translation. Rust methods use snake_case; fields retain upstream names.

// Source: src/core/SkGeometry.h:266-273
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SkCubicType {
    kSerpentine,
    kLoop,
    kLocalCusp,
    kCuspAtInfinity,
    kQuadratic,
    kLineOrPoint,
}

// Source: src/core/SkGeometry.h:275-287
#[inline]
pub fn SkCubicIsDegenerate(kind: SkCubicType) -> bool {
    match kind {
        SkCubicType::kSerpentine
        | SkCubicType::kLoop
        | SkCubicType::kLocalCusp
        | SkCubicType::kCuspAtInfinity => false,
        SkCubicType::kQuadratic | SkCubicType::kLineOrPoint => true,
    }
}

// Source: src/core/SkGeometry.h:289-299
#[inline]
pub fn SkCubicTypeName(kind: SkCubicType) -> &'static str {
    match kind {
        SkCubicType::kSerpentine => "kSerpentine",
        SkCubicType::kLoop => "kLoop",
        SkCubicType::kLocalCusp => "kLocalCusp",
        SkCubicType::kCuspAtInfinity => "kCuspAtInfinity",
        SkCubicType::kQuadratic => "kQuadratic",
        SkCubicType::kLineOrPoint => "kLineOrPoint",
    }
}

// Source: src/core/SkGeometry.h:324-324
// Adapted Source: src/core/SkGeometry.h:324-324
// Rust safe default initializes fields that the native empty constructor leaves unspecified.
impl Default for SkConic {
    #[inline]
    fn default() -> Self {
        Self {
            fPts: [SkPoint::default(); 3],
            fW: 0.0,
        }
    }
}

// Source: src/core/SkGeometry.h:325-327
impl SkConic {
    #[inline]
    pub fn from_components(p0: SkPoint, p1: SkPoint, p2: SkPoint, w: f32) -> Self {
        let mut conic = Self::default();
        conic.set_components(p0, p1, p2, w);
        conic
    }
}

// Source: src/core/SkGeometry.h:329-331
impl SkConic {
    pub fn new(pts: &[SkPoint; 3], w: f32) -> Self {
        let mut conic = Self::default();
        conic.set(pts, w);
        conic
    }
}

// Source: src/core/SkGeometry.h:333-334
#[repr(C)]
#[derive(Clone, Copy)]
pub struct SkConic {
    pub fPts: [SkPoint; 3],
    pub fW: f32,
}

// Source: src/core/SkGeometry.h:336-339
impl SkConic {
    pub fn set(&mut self, pts: &[SkPoint; 3], w: f32) {
        self.fPts = *pts;
        self.setW(w);
    }
}

// Source: src/core/SkGeometry.h:341-346
impl SkConic {
    #[inline]
    pub fn set_components(&mut self, p0: SkPoint, p1: SkPoint, p2: SkPoint, w: f32) {
        self.fPts[0] = p0;
        self.fPts[1] = p1;
        self.fPts[2] = p2;
        self.setW(w);
    }
}

// Source: src/core/SkGeometry.h:348-355
impl SkConic {
    #[inline]
    pub fn setW(&mut self, w: f32) {
        if w.is_finite() {
            debug_assert!(w > 0.0);
        }
        self.fW = if w > 0.0 && w.is_finite() { w } else { 1.0 };
    }
}

// Source: src/core/SkGeometry.h:406-408
impl SkConic {
    pub const kMaxConicsForArc: usize = 5;
}

// Source: src/core/SkGeometry.h:421-421
// Adapted Source: src/core/SkGeometry.h:421-421
impl Default for SkQuadCoeff {
    #[inline]
    fn default() -> Self {
        Self {
            fA: SkPoint::default(),
            fB: SkPoint::default(),
            fC: SkPoint::default(),
        }
    }
}

// Source: src/core/SkGeometry.h:423-429
impl SkQuadCoeff {
    #[inline]
    pub fn new(a: SkPoint, b: SkPoint, c: SkPoint) -> Self {
        Self {
            fA: a,
            fB: b,
            fC: c,
        }
    }
}

// Source: src/core/SkGeometry.h:430-436
impl SkQuadCoeff {
    pub fn from_points(src: &[SkPoint; 3]) -> Self {
        let c = src[0];
        let p1 = src[1];
        let p2 = src[2];
        Self {
            fB: times_2(p1 - c),
            fA: p2 - times_2(p1) + c,
            fC: c,
        }
    }
}

// Source: src/core/SkGeometry.h:438-440
impl SkQuadCoeff {
    #[inline]
    pub fn eval(&self, t: SkPoint) -> SkPoint {
        mul_lanes(mul_lanes(self.fA, t) + self.fB, t) + self.fC
    }
}

// Source: src/core/SkGeometry.h:442-445
pub struct SkQuadCoeff {
    pub fA: SkPoint,
    pub fB: SkPoint,
    pub fC: SkPoint,
}

// Source: src/core/SkGeometry.h:448-462
impl SkConicCoeff {
    #[inline]
    fn new(conic: &SkConic) -> Self {
        let p0 = conic.fPts[0];
        let p1 = conic.fPts[1];
        let p2 = conic.fPts[2];
        let ww = broadcast(conic.fW);
        let p1w = mul_lanes(p1, ww);
        let numer = SkQuadCoeff {
            fC: p0,
            fA: p2 - times_2(p1w) + p0,
            fB: times_2(p1w - p0),
        };
        let c = broadcast(1.0);
        let b = times_2(ww - c);
        let a = broadcast(0.0) - b;
        Self {
            fNumer: numer,
            fDenom: SkQuadCoeff {
                fC: c,
                fB: b,
                fA: a,
            },
        }
    }
}

// Source: src/core/SkGeometry.h:464-469
impl SkConicCoeff {
    #[inline]
    fn eval(&self, t: f32) -> SkPoint {
        let tt = broadcast(t);
        let numer = self.fNumer.eval(tt);
        let denom = self.fDenom.eval(tt);
        SkPoint::make(numer.fX / denom.fX, numer.fY / denom.fY)
    }
}

// Source: src/core/SkGeometry.h:471-473
struct SkConicCoeff {
    fNumer: SkQuadCoeff,
    fDenom: SkQuadCoeff,
}

// Source: src/core/SkGeometry.h:476-486
impl SkCubicCoeff {
    pub fn new(src: &[SkPoint; 4]) -> Self {
        let p0 = src[0];
        let p1 = src[1];
        let p2 = src[2];
        let p3 = src[3];
        let three = broadcast(3.0);
        Self {
            fA: p3 + mul_lanes(three, p1 - p2) - p0,
            fB: mul_lanes(three, p2 - times_2(p1) + p0),
            fC: mul_lanes(three, p1 - p0),
            fD: p0,
        }
    }
}

// Source: src/core/SkGeometry.h:488-490
impl SkCubicCoeff {
    #[inline]
    pub fn eval(&self, t: SkPoint) -> SkPoint {
        mul_lanes(mul_lanes(mul_lanes(self.fA, t) + self.fB, t) + self.fC, t) + self.fD
    }
}

// Source: src/core/SkGeometry.h:492-496
pub struct SkCubicCoeff {
    pub fA: SkPoint,
    pub fB: SkPoint,
    pub fC: SkPoint,
    pub fD: SkPoint,
}

// Source: src/core/SkGeometry.h:505-507
impl Default for SkAutoConicToQuads {
    #[inline]
    fn default() -> Self {
        Self {
            inline: [SkPoint::default(); 17],
            heap: None,
            fQuadCount: 0,
        }
    }
}

// Source: src/core/SkGeometry.h:521-527
impl SkAutoConicToQuads {
    #[inline]
    pub fn computeConicQuads(&mut self, conic: &SkConic, tol: f32) -> &[SkPoint] {
        let pow2 = conic.computeQuadPOW2(tol);
        self.fQuadCount = 1 << pow2;
        let pts = self.reset_storage(1 + 2 * self.fQuadCount);
        let count = conic.chopIntoQuadsPOW2(pts, pow2);
        self.fQuadCount = count;
        &self.storage()[..1 + 2 * count]
    }
}

// Source: src/core/SkGeometry.h:529-544
impl SkAutoConicToQuads {
    #[inline]
    pub fn computeQuads(&mut self, pts: &[SkPoint], weight: f32, tol: f32) -> &[SkPoint] {
        if weight <= 0.0 {
            self.fQuadCount = 1;
            let out = self.reset_storage(3);
            out[0] = pts[0];
            out[1] = SkPoint::make((pts[0].fX + pts[2].fX) * 0.5, (pts[0].fY + pts[2].fY) * 0.5);
            out[2] = pts[2];
            return &self.storage()[..3];
        }
        let conic = SkConic::new(&[pts[0], pts[1], pts[2]], weight);
        self.computeConicQuads(&conic, tol)
    }
}

// Source: src/core/SkGeometry.h:546-548
impl SkAutoConicToQuads {
    pub fn computeQuadsArray(&mut self, pts: &[SkPoint; 3], weight: f32, tol: f32) -> &[SkPoint] {
        self.computeQuads(pts, weight, tol)
    }
}

// Source: src/core/SkGeometry.h:550-550
impl SkAutoConicToQuads {
    #[inline]
    pub fn countQuads(&self) -> usize {
        self.fQuadCount
    }
}

// Source: src/core/SkGeometry.h:553-559
// Adapted Source: src/core/SkGeometry.h:557-557
pub struct SkAutoConicToQuads {
    inline: [SkPoint; 17],
    heap: Option<Box<[SkPoint]>>,
    fQuadCount: usize,
}

// Source: src/core/SkGeometry.cpp:35-39
fn to_vector(x: [f32; 2]) -> SkPoint {
    SkPoint::make(x[0], x[1])
}

// Source: src/core/SkGeometry.cpp:43-50
fn is_not_monotonic(a: f32, b: f32, c: f32) -> bool {
    let ab = a - b;
    let mut bc = b - c;
    if ab < 0.0 {
        bc = -bc;
    }
    ab == 0.0 || bc < 0.0
}

// Source: src/core/SkGeometry.cpp:54-76
fn valid_unit_divide(mut numer: f32, mut denom: f32, ratio: &mut f32) -> usize {
    if numer < 0.0 {
        numer = -numer;
        denom = -denom;
    }
    if denom == 0.0 || numer == 0.0 || numer >= denom {
        return 0;
    }
    let r = numer / denom;
    if r.is_nan() {
        return 0;
    }
    debug_assert!(r >= 0.0 && r < 1.0);
    if r == 0.0 {
        return 0;
    }
    *ratio = r;
    1
}

// Source: src/core/SkGeometry.cpp:80-85
fn return_check_zero(value: usize) -> usize {
    if value == 0 {
        return 0;
    }
    value
}

// Source: src/core/SkGeometry.cpp:95-127
pub fn SkFindUnitQuadRoots(a: f32, b: f32, c: f32, roots: &mut [f32; 2]) -> usize {
    if a == 0.0 {
        return return_check_zero(valid_unit_divide(-c, b, &mut roots[0]));
    }
    let mut dr = b as f64 * b as f64 - 4.0 * a as f64 * c as f64;
    if dr < 0.0 {
        return return_check_zero(0);
    }
    dr = dr.sqrt();
    let r = dr as f32;
    if !r.is_finite() {
        return return_check_zero(0);
    }
    let q = if b < 0.0 {
        -(b - r) / 2.0
    } else {
        -(b + r) / 2.0
    };
    let mut count = valid_unit_divide(q, a, &mut roots[0]);
    count += valid_unit_divide(c, q, &mut roots[count]);
    if count == 2 {
        if roots[0] > roots[1] {
            roots.swap(0, 1);
        } else if roots[0] == roots[1] {
            count -= 1;
        }
    }
    return_check_zero(count)
}

// Source: src/core/SkGeometry.cpp:132-142
pub fn SkEvalQuadAtWithTangent(
    src: &[SkPoint; 3],
    t: f32,
    point: Option<&mut SkPoint>,
    tangent: Option<&mut SkPoint>,
) {
    debug_assert!(t >= 0.0 && t <= 1.0);
    if let Some(pt) = point {
        *pt = SkEvalQuadAt(src, t);
    }
    if let Some(tan) = tangent {
        *tan = SkEvalQuadTangentAt(src, t);
    }
}

// Source: src/core/SkGeometry.cpp:144-146
pub fn SkEvalQuadAt(src: &[SkPoint; 3], t: f32) -> SkPoint {
    to_vector(from_point(
        &SkQuadCoeff::from_points(src).eval(broadcast(t)),
    ))
}

// Source: src/core/SkGeometry.cpp:148-167
pub fn SkEvalQuadTangentAt(src: &[SkPoint; 3], t: f32) -> SkPoint {
    if (t == 0.0 && src[0] == src[1]) || (t == 1.0 && src[1] == src[2]) {
        return src[2] - src[0];
    }
    debug_assert!(t >= 0.0 && t <= 1.0);
    let p0 = src[0];
    let p1 = src[1];
    let p2 = src[2];
    let b = p1 - p0;
    let a = p2 - p1 - b;
    let tt = a * t + b;
    tt + tt
}

// Source: src/core/SkGeometry.cpp:169-173
fn interp(v0: SkPoint, v1: SkPoint, t: f32) -> SkPoint {
    v0 + (v1 - v0) * t
}

// Source: src/core/SkGeometry.cpp:175-191
pub fn SkChopQuadAt(src: &[SkPoint; 3], dst: &mut [SkPoint; 5], t: f32) {
    debug_assert!(t > 0.0 && t < 1.0);
    let p0 = src[0];
    let p1 = src[1];
    let p2 = src[2];
    let p01 = interp(p0, p1, t);
    let p12 = interp(p1, p2, t);
    dst[0] = p0;
    dst[1] = p01;
    dst[2] = interp(p01, p12, t);
    dst[3] = p12;
    dst[4] = p2;
}

// Source: src/core/SkGeometry.cpp:193-195
pub fn SkChopQuadAtHalf(src: &[SkPoint; 3], dst: &mut [SkPoint; 5]) {
    SkChopQuadAt(src, dst, 0.5);
}

// Source-aligned Skia translation. Rust methods use snake_case; fields retain upstream names.

// Source: src/core/SkGeometry.cpp:197-202
pub fn SkMeasureAngleBetweenVectors(a: SkPoint, b: SkPoint) -> f32 {
    let mut cosine = a.dot(b) / (a.dot(a) * b.dot(b)).sqrt();
    cosine = if cosine < 1.0 { cosine } else { 1.0 };
    cosine = if cosine < -1.0 { -1.0 } else { cosine };
    cosine.acos()
}

// Source: src/core/SkGeometry.cpp:204-229
pub fn SkFindBisector(a: SkPoint, b: SkPoint) -> SkPoint {
    let v = if a.dot(b) >= 0.0 {
        [a, b]
    } else if a.cross(b) >= 0.0 {
        [SkPoint::make(-a.fY, a.fX), SkPoint::make(b.fY, -b.fX)]
    } else {
        [SkPoint::make(a.fY, -a.fX), SkPoint::make(-b.fY, b.fX)]
    };
    let inv0 = 1.0 / (v[0].fX * v[0].fX + v[0].fY * v[0].fY).sqrt();
    let inv1 = 1.0 / (v[1].fX * v[1].fX + v[1].fY * v[1].fY).sqrt();
    let x0 = v[0].fX * inv0;
    let x1 = v[1].fX * inv1;
    let y0 = v[0].fY * inv0;
    let y1 = v[1].fY * inv1;
    SkPoint::make(x0 + x1, y0 + y1)
}

// Source: src/core/SkGeometry.cpp:231-258
pub fn SkFindQuadMidTangent(src: &[SkPoint; 3]) -> f32 {
    let tan0 = src[1] - src[0];
    let tan1 = src[2] - src[1];
    let bisector = SkFindBisector(tan0, -tan1);
    let mut t = tan0.dot(bisector) / (tan0 - tan1).dot(bisector);
    if !(t > 0.0 && t < 1.0) {
        t = 0.5;
    }
    t
}

// Source: src/core/SkGeometry.cpp:265-270
pub fn SkFindQuadExtrema(a: f32, b: f32, c: f32, t: &mut f32) -> usize {
    valid_unit_divide(a - b, a - b - b + c, t)
}

// Source: src/core/SkGeometry.cpp:272-274
fn flatten_double_quad_extrema(dst: &mut [SkPoint; 5]) {
    dst[1].fY = dst[2].fY;
    dst[3].fY = dst[2].fY;
}

// Source: src/core/SkGeometry.cpp:279-302
pub fn SkChopQuadAtYExtrema(src: &[SkPoint; 3], dst: &mut [SkPoint; 5]) -> usize {
    let a = src[0].fY;
    let mut b = src[1].fY;
    let c = src[2].fY;
    if is_not_monotonic(a, b, c) {
        let mut t = 0.0;
        if valid_unit_divide(a - b, a - b - b + c, &mut t) != 0 {
            SkChopQuadAt(src, dst, t);
            flatten_double_quad_extrema(dst);
            return 1;
        }
        b = if (a - b).abs() < (b - c).abs() { a } else { c };
    }
    dst[0] = SkPoint::make(src[0].fX, a);
    dst[1] = SkPoint::make(src[1].fX, b);
    dst[2] = SkPoint::make(src[2].fX, c);
    0
}

// Source: src/core/SkGeometry.cpp:307-330
pub fn SkChopQuadAtXExtrema(src: &[SkPoint; 3], dst: &mut [SkPoint; 5]) -> usize {
    let a = src[0].fX;
    let mut b = src[1].fX;
    let c = src[2].fX;
    if is_not_monotonic(a, b, c) {
        let mut t = 0.0;
        if valid_unit_divide(a - b, a - b - b + c, &mut t) != 0 {
            SkChopQuadAt(src, dst, t);
            dst[1].fX = dst[2].fX;
            dst[3].fX = dst[2].fX;
            return 1;
        }
        b = if (a - b).abs() < (b - c).abs() { a } else { c };
    }
    dst[0] = SkPoint::make(a, src[0].fY);
    dst[1] = SkPoint::make(b, src[1].fY);
    dst[2] = SkPoint::make(c, src[2].fY);
    0
}

// Source: src/core/SkGeometry.cpp:344-365
pub fn SkFindQuadMaxCurvature(src: &[SkPoint; 3]) -> f32 {
    let ax = src[1].fX - src[0].fX;
    let ay = src[1].fY - src[0].fY;
    let bx = src[0].fX - src[1].fX - src[1].fX + src[2].fX;
    let by = src[0].fY - src[1].fY - src[1].fY + src[2].fY;
    let mut numer = -(ax * bx + ay * by);
    let mut denom = bx * bx + by * by;
    if denom < 0.0 {
        numer = -numer;
        denom = -denom;
    }
    if numer <= 0.0 {
        return 0.0;
    }
    if numer >= denom {
        return 1.0;
    }
    let t = numer / denom;
    debug_assert!((t >= 0.0 && t < 1.0) || t.is_nan());
    t
}

// Source: src/core/SkGeometry.cpp:367-376
pub fn SkChopQuadAtMaxCurvature(src: &[SkPoint; 3], dst: &mut [SkPoint; 5]) -> usize {
    let t = SkFindQuadMaxCurvature(src);
    if t > 0.0 && t < 1.0 {
        SkChopQuadAt(src, dst, t);
        2
    } else {
        dst[..3].copy_from_slice(src);
        1
    }
}

// Source: src/core/SkGeometry.cpp:378-388
pub fn SkConvertQuadToCubic(src: &[SkPoint; 3], dst: &mut [SkPoint; 4]) {
    let scale = (2.0f64 / 3.0) as f32;
    let s0 = src[0];
    let s1 = src[1];
    let s2 = src[2];
    dst[0] = s0;
    dst[1] = s0 + (s1 - s0) * scale;
    dst[2] = s2 + (s1 - s2) * scale;
    dst[3] = s2;
}

// Source: src/core/SkGeometry.cpp:394-405
fn eval_cubic_derivative(src: &[SkPoint; 4], t: f32) -> SkPoint {
    let p0 = src[0];
    let p1 = src[1];
    let p2 = src[2];
    let p3 = src[3];
    let coeff = SkQuadCoeff {
        fA: p3 + (p1 - p2) * 3.0 - p0,
        fB: times_2(p2 - times_2(p1) + p0),
        fC: p1 - p0,
    };
    coeff.eval(broadcast(t))
}

// Source: src/core/SkGeometry.cpp:407-416
fn eval_cubic_2ndDerivative(src: &[SkPoint; 4], t: f32) -> SkPoint {
    let p0 = src[0];
    let p1 = src[1];
    let p2 = src[2];
    let p3 = src[3];
    let a = p3 + (p1 - p2) * 3.0 - p0;
    let b = p2 - times_2(p1) + p0;
    a * t + b
}

// Source: src/core/SkGeometry.cpp:418-446
pub fn SkEvalCubicAt(
    src: &[SkPoint; 4],
    t: f32,
    loc: Option<&mut SkPoint>,
    tangent: Option<&mut SkPoint>,
    curvature: Option<&mut SkPoint>,
) {
    debug_assert!(t >= 0.0 && t <= 1.0);
    if let Some(loc) = loc {
        *loc = SkCubicCoeff::new(src).eval(broadcast(t));
    }
    if let Some(tangent) = tangent {
        if (t == 0.0 && src[0] == src[1]) || (t == 1.0 && src[2] == src[3]) {
            *tangent = if t == 0.0 {
                src[2] - src[0]
            } else {
                src[3] - src[1]
            };
            if tangent.fX == 0.0 && tangent.fY == 0.0 {
                *tangent = src[3] - src[0];
            }
        } else {
            *tangent = eval_cubic_derivative(src, t);
        }
    }
    if let Some(curvature) = curvature {
        *curvature = eval_cubic_2ndDerivative(src, t);
    }
}

// Source: src/core/SkGeometry.cpp:454-462
pub fn SkFindCubicExtrema(a: f32, b: f32, c: f32, d: f32, roots: &mut [f32; 2]) -> usize {
    let aa = d - a + 3.0 * (b - c);
    let bb = 2.0 * (a - b - b + c);
    let cc = b - a;
    SkFindUnitQuadRoots(aa, bb, cc, roots)
}

// Source: src/core/SkGeometry.cpp:468-471
fn unchecked_mix(a: SkPoint, b: SkPoint, t: f32) -> SkPoint {
    (b - a) * t + a
}

// Source: src/core/SkGeometry.cpp:473-502
pub fn SkChopCubicAt(src: &[SkPoint; 4], dst: &mut [SkPoint; 7], t: f32) {
    debug_assert!(t >= 0.0 && t <= 1.0);
    if t == 1.0 {
        dst[..4].copy_from_slice(src);
        dst[4] = src[3];
        dst[5] = src[3];
        dst[6] = src[3];
        return;
    }
    let p0 = src[0];
    let p1 = src[1];
    let p2 = src[2];
    let p3 = src[3];
    let ab = unchecked_mix(p0, p1, t);
    let bc = unchecked_mix(p1, p2, t);
    let cd = unchecked_mix(p2, p3, t);
    let abc = unchecked_mix(ab, bc, t);
    let bcd = unchecked_mix(bc, cd, t);
    let abcd = unchecked_mix(abc, bcd, t);
    dst[0] = p0;
    dst[1] = ab;
    dst[2] = abc;
    dst[3] = abcd;
    dst[4] = bcd;
    dst[5] = cd;
    dst[6] = p3;
}

// Source: src/core/SkGeometry.cpp:504-539
pub fn SkChopCubicAtTwo(src: &[SkPoint; 4], dst: &mut [SkPoint; 10], t0: f32, t1: f32) {
    debug_assert!(0.0 <= t0 && t0 <= t1 && t1 <= 1.0);
    if t1 == 1.0 {
        SkChopCubicAt(src, (&mut dst[..7]).try_into().unwrap(), t0);
        dst[7..].fill(src[3]);
        return;
    }
    let ab = [
        unchecked_mix(src[0], src[1], t0),
        unchecked_mix(src[0], src[1], t1),
    ];
    let bc = [
        unchecked_mix(src[1], src[2], t0),
        unchecked_mix(src[1], src[2], t1),
    ];
    let cd = [
        unchecked_mix(src[2], src[3], t0),
        unchecked_mix(src[2], src[3], t1),
    ];
    let abc = [
        unchecked_mix(ab[0], bc[0], t0),
        unchecked_mix(ab[1], bc[1], t1),
    ];
    let bcd = [
        unchecked_mix(bc[0], cd[0], t0),
        unchecked_mix(bc[1], cd[1], t1),
    ];
    let abcd = [
        unchecked_mix(abc[0], bcd[0], t0),
        unchecked_mix(abc[1], bcd[1], t1),
    ];
    *dst = [
        src[0],
        ab[0],
        abc[0],
        abcd[0],
        unchecked_mix(abc[0], bcd[0], t1),
        unchecked_mix(abc[1], bcd[1], t0),
        abcd[1],
        bcd[1],
        cd[1],
        src[3],
    ];
}

// Source: src/core/SkGeometry.cpp:541-573
pub fn SkChopCubicAtMultiple(src: &[SkPoint; 4], dst: &mut [SkPoint], ts: &[f32]) {
    debug_assert!(ts.iter().all(|t| *t >= 0.0 && *t <= 1.0));
    debug_assert!(ts.windows(2).all(|v| v[0] <= v[1]));
    assert!(dst.len() >= 4 + ts.len() * 3);
    if ts.is_empty() {
        dst[..4].copy_from_slice(src);
        return;
    }
    let mut current = *src;
    let mut offset = 0;
    let mut i = 0;
    while i + 1 < ts.len() {
        let mut t0 = ts[i];
        let mut t1 = ts[i + 1];
        if i != 0 {
            let last = ts[i - 1];
            t0 = crate::include::private::SkTPin::SkTPin(((t0 - last) / (1.0 - last)), 0.0, 1.0);
            t1 = crate::include::private::SkTPin::SkTPin(((t1 - last) / (1.0 - last)), 0.0, 1.0);
        }
        SkChopCubicAtTwo(
            &current,
            (&mut dst[offset..offset + 10]).try_into().unwrap(),
            t0,
            t1,
        );
        offset += 6;
        current.copy_from_slice(&dst[offset..offset + 4]);
        i += 2;
    }
    if i < ts.len() {
        let mut t = ts[i];
        if i != 0 {
            let last = ts[i - 1];
            t = crate::include::private::SkTPin::SkTPin(((t - last) / (1.0 - last)), 0.0, 1.0);
        }
        SkChopCubicAt(
            &current,
            (&mut dst[offset..offset + 7]).try_into().unwrap(),
            t,
        );
    }
}

// Source: src/core/SkGeometry.cpp:575-577
pub fn SkChopCubicAtHalf(src: &[SkPoint; 4], dst: &mut [SkPoint; 7]) {
    SkChopCubicAt(src, dst, 0.5);
}

// Source: src/core/SkGeometry.cpp:579-595
pub fn SkMeasureNonInflectCubicRotation(pts: &[SkPoint; 4]) -> f32 {
    let a = pts[1] - pts[0];
    let b = pts[2] - pts[1];
    let c = pts[3] - pts[2];
    if a.is_zero() {
        return SkMeasureAngleBetweenVectors(b, c);
    }
    if b.is_zero() {
        return SkMeasureAngleBetweenVectors(a, c);
    }
    if c.is_zero() {
        return SkMeasureAngleBetweenVectors(a, b);
    }
    2.0 * crate::include::core::SkScalar::SK_ScalarPI
        - SkMeasureAngleBetweenVectors(a, -b)
        - SkMeasureAngleBetweenVectors(b, -c)
}

// Source: src/core/SkGeometry.cpp:597-599
fn fma4(f: [f32; 4], m: f32, a: [f32; 4]) -> [f32; 4] {
    core::array::from_fn(|i| f[i].mul_add(m, a[i]))
}

// Source: src/core/SkGeometry.cpp:602-614
fn solve_quadratic_equation_for_midtangent_discr(a: f32, b: f32, c: f32, discr: f32) -> f32 {
    let q = -0.5 * (b + discr.sqrt().copysign(b));
    let half_qa = -0.5 * q * a;
    let mut t = if (q * q + half_qa).abs() < (a * c + half_qa).abs() {
        q / a
    } else {
        c / q
    };
    if !(t > 0.0 && t < 1.0) {
        t = 0.5;
    }
    t
}

// Source: src/core/SkGeometry.cpp:616-618
fn solve_quadratic_equation_for_midtangent(a: f32, b: f32, c: f32) -> f32 {
    solve_quadratic_equation_for_midtangent_discr(a, b, c, b * b - 4.0 * a * c)
}

// Source: src/core/SkGeometry.cpp:620-684
pub fn SkFindCubicMidTangent(src: &[SkPoint; 4]) -> f32 {
    let tan0 = if src[0] == src[1] {
        src[2] - src[0]
    } else {
        src[1] - src[0]
    };
    let tan1 = if src[2] == src[3] {
        src[3] - src[1]
    } else {
        src[3] - src[2]
    };
    let bisector = SkFindBisector(tan0, -tan1);
    const M: [[f32; 4]; 4] = [
        [-1.0, 2.0, -1.0, 0.0],
        [3.0, -4.0, 1.0, 0.0],
        [-3.0, 2.0, 0.0, 0.0],
        [0.0; 4],
    ];
    let cx = fma4(
        M[0],
        src[0].fX,
        fma4(
            M[1],
            src[1].fX,
            fma4(M[2], src[2].fX, [src[3].fX, 0.0, 0.0, 0.0]),
        ),
    );
    let cy = fma4(
        M[0],
        src[0].fY,
        fma4(
            M[1],
            src[1].fY,
            fma4(M[2], src[2].fY, [src[3].fY, 0.0, 0.0, 0.0]),
        ),
    );
    let mut coeff = core::array::from_fn::<_, 4, _>(|i| cx[i] * bisector.fX + cy[i] * bisector.fY);
    let mut t = 0.0;
    let mut a = coeff[0];
    let mut b = coeff[1];
    let c = coeff[2];
    let discr = b * b - 4.0 * a * c;
    if discr > 0.0 {
        solve_quadratic_equation_for_midtangent_discr(a, b, c, discr)
    } else {
        coeff = core::array::from_fn(|i| cx[i] * tan0.fX + cy[i] * tan0.fY);
        a = coeff[0];
        b = coeff[1];
        if a != 0.0 {
            t = -b / (2.0 * a);
        }
        if !(t > 0.0 && t < 1.0) {
            t = 0.5;
        }
        t
    }
}

// Source: src/core/SkGeometry.cpp:686-688
fn flatten_double_cubic_extrema(dst: &mut [SkPoint]) {
    dst[2].fY = dst[3].fY;
    dst[4].fY = dst[3].fY;
}

// Source: src/core/SkGeometry.cpp:698-712
pub fn SkChopCubicAtYExtrema(src: &[SkPoint; 4], dst: &mut [SkPoint; 10]) -> usize {
    let mut ts = [0.0; 2];
    let roots = SkFindCubicExtrema(src[0].fY, src[1].fY, src[2].fY, src[3].fY, &mut ts);
    SkChopCubicAtMultiple(src, dst, &ts[..roots]);
    if roots > 0 {
        flatten_double_cubic_extrema(dst);
        if roots == 2 {
            flatten_double_cubic_extrema(&mut dst[3..]);
        }
    }
    roots
}

// Source: src/core/SkGeometry.cpp:714-728
pub fn SkChopCubicAtXExtrema(src: &[SkPoint; 4], dst: &mut [SkPoint; 10]) -> usize {
    let mut ts = [0.0; 2];
    let roots = SkFindCubicExtrema(src[0].fX, src[1].fX, src[2].fX, src[3].fX, &mut ts);
    SkChopCubicAtMultiple(src, dst, &ts[..roots]);
    if roots > 0 {
        dst[2].fX = dst[3].fX;
        dst[4].fX = dst[3].fX;
        if roots == 2 {
            dst[5].fX = dst[6].fX;
            dst[7].fX = dst[6].fX;
        }
    }
    roots
}

// Source: src/core/SkGeometry.cpp:741-753
pub fn SkFindCubicInflections(src: &[SkPoint; 4], ts: &mut [f32; 2]) -> usize {
    let ax = src[1].fX - src[0].fX;
    let ay = src[1].fY - src[0].fY;
    let bx = src[2].fX - 2.0 * src[1].fX + src[0].fX;
    let by = src[2].fY - 2.0 * src[1].fY + src[0].fY;
    let cx = src[3].fX + 3.0 * (src[1].fX - src[2].fX) - src[0].fX;
    let cy = src[3].fY + 3.0 * (src[1].fY - src[2].fY) - src[0].fY;
    SkFindUnitQuadRoots(bx * cy - by * cx, ax * cy - ay * cx, ax * by - ay * bx, ts)
}

// Source: src/core/SkGeometry.cpp:755-767
pub fn SkChopCubicAtInflections(src: &[SkPoint; 4], dst: Option<&mut [SkPoint; 10]>) -> usize {
    let mut ts = [0.0; 2];
    let count = SkFindCubicInflections(src, &mut ts);
    if let Some(dst) = dst {
        if count == 0 {
            dst[..4].copy_from_slice(src);
        } else {
            SkChopCubicAtMultiple(src, dst, &ts[..count]);
        }
    }
    count + 1
}

// Source: src/core/SkGeometry.cpp:771-776
fn calc_dot_cross_cubic(p0: SkPoint, p1: SkPoint, p2: SkPoint) -> f64 {
    let x = p0.fX as f64 * (p1.fY as f64 - p2.fY as f64);
    let y = p0.fY as f64 * (p2.fX as f64 - p1.fX as f64);
    let w = p1.fX as f64 * p2.fY as f64 - p1.fY as f64 * p2.fX as f64;
    x + y + w
}

// Source: src/core/SkGeometry.cpp:782-789
fn previous_inverse_pow2(n: f64) -> f64 {
    let mut bits = n.to_bits();
    bits = ((1023u64 * 2 << 52) + ((1u64 << 52) - 1)).wrapping_sub(bits);
    bits &= 0x7ffu64 << 52;
    f64::from_bits(bits)
}

// Source: src/core/SkGeometry.cpp:791-807
fn write_cubic_inflection_roots(
    t0: f64,
    s0: f64,
    t1: f64,
    s1: f64,
    t: &mut [f64; 2],
    s: &mut [f64; 2],
) {
    t[0] = t0;
    s[0] = s0;
    t[1] = -t1.copysign(t1 * s1);
    s[1] = -s1.abs();
    if s[1].copysign(s[0]) * t[0] > -s[0].abs() * t[1] {
        t.swap(0, 1);
        s.swap(0, 1);
    }
}

// Source: src/core/SkGeometry.cpp:809-879
pub fn SkClassifyCubic(
    p: &[SkPoint; 4],
    mut t: Option<&mut [f64; 2]>,
    mut s: Option<&mut [f64; 2]>,
    d: Option<&mut [f64; 4]>,
) -> SkCubicType {
    let a1 = calc_dot_cross_cubic(p[0], p[3], p[2]);
    let a2 = calc_dot_cross_cubic(p[1], p[0], p[3]);
    let a3 = calc_dot_cross_cubic(p[2], p[1], p[0]);
    let mut d3 = 3.0 * a3;
    let mut d2 = d3 - a2;
    let mut d1 = d2 - a2 + a1;
    let max = |a: f64, b: f64| if a < b { b } else { a };
    let dmax = max(max(d1.abs(), d2.abs()), d3.abs());
    let norm = previous_inverse_pow2(dmax);
    d1 *= norm;
    d2 *= norm;
    d3 *= norm;
    if let Some(d) = d {
        d[3] = d3;
        d[2] = d2;
        d[1] = d1;
        d[0] = 0.0;
    }
    if d1 != 0.0 {
        let discr = 3.0 * d2 * d2 - 4.0 * d1 * d3;
        if discr > 0.0 {
            if let (Some(t), Some(s)) = (t.as_deref_mut(), s.as_deref_mut()) {
                let q = 3.0 * d2 + (3.0 * discr).sqrt().copysign(d2);
                write_cubic_inflection_roots(q, 6.0 * d1, 2.0 * d3, q, t, s);
            }
            SkCubicType::kSerpentine
        } else if discr < 0.0 {
            if let (Some(t), Some(s)) = (t.as_deref_mut(), s.as_deref_mut()) {
                let q = d2 + (-discr).sqrt().copysign(d2);
                write_cubic_inflection_roots(q, 2.0 * d1, 2.0 * (d2 * d2 - d3 * d1), d1 * q, t, s);
            }
            SkCubicType::kLoop
        } else {
            if let (Some(t), Some(s)) = (t.as_deref_mut(), s.as_deref_mut()) {
                write_cubic_inflection_roots(d2, 2.0 * d1, d2, 2.0 * d1, t, s);
            }
            SkCubicType::kLocalCusp
        }
    } else if d2 != 0.0 {
        if let (Some(t), Some(s)) = (t.as_deref_mut(), s.as_deref_mut()) {
            write_cubic_inflection_roots(d3, 3.0 * d2, 1.0, 0.0, t, s);
        }
        SkCubicType::kCuspAtInfinity
    } else {
        if let (Some(t), Some(s)) = (t.as_deref_mut(), s.as_deref_mut()) {
            write_cubic_inflection_roots(1.0, 0.0, 1.0, 0.0, t, s);
        }
        if d3 != 0.0 {
            SkCubicType::kQuadratic
        } else {
            SkCubicType::kLineOrPoint
        }
    }
}

// Source: src/core/SkGeometry.cpp:881-890
fn bubble_sort<T: PartialOrd>(array: &mut [T]) {
    for i in (1..array.len()).rev() {
        for j in (1..=i).rev() {
            if array[j] < array[j - 1] {
                array.swap(j, j - 1);
            }
        }
    }
}

// Source: src/core/SkGeometry.cpp:896-908
fn collaps_duplicates(array: &mut [f32], mut count: usize) -> usize {
    let mut offset = 0;
    for n in (2..=count).rev() {
        if array[offset] == array[offset + 1] {
            for i in 1..n {
                array[offset + i - 1] = array[offset + i];
            }
            count -= 1;
        } else {
            offset += 1;
        }
    }
    count
}

// Source: src/core/SkGeometry.cpp:914-947
// Adapted Source: src/core/SkGeometry.cpp:915-917
#[cfg(sk_debug)]
fn test_collaps_duplicates() {
    static ONCE: std::sync::Once = std::sync::Once::new();
    ONCE.call_once(|| {
        let cases: [(&[f32], usize); 7] = [
            (&[0.0], 1),
            (&[0.0, 0.0], 1),
            (&[0.0, 1.0], 2),
            (&[0.0, 0.0, 0.0], 1),
            (&[0.0, 0.0, 1.0], 2),
            (&[0.0, 1.0, 1.0], 2),
            (&[0.0, 1.0, 2.0], 3),
        ];
        for (data, expected) in cases {
            let mut dst = [0.0; 3];
            dst[..data.len()].copy_from_slice(data);
            let count = collaps_duplicates(&mut dst, data.len());
            debug_assert_eq!(expected, count);
            for j in 1..count {
                crate::SkASSERT!(@source b"src/core/SkGeometry.cpp\0".as_ptr(),944; @condition b"dst[j-1] < dst[j]\0".as_ptr(); dst[j - 1] < dst[j]);
            }
        }
    });
}

// Source: src/core/SkGeometry.cpp:950-952
fn SkScalarCubeRoot(x: f32) -> f32 {
    x.powf(0.3333333)
}

// Source: src/core/SkGeometry.cpp:961-1008
fn solve_cubic_poly(coeff: &[f32; 4], ts: &mut [f32; 3]) -> usize {
    if crate::include::core::SkScalar::SkScalarNearlyZero(
        coeff[0],
        crate::include::core::SkScalar::SK_ScalarNearlyZero,
    ) {
        let mut roots = [0.0; 2];
        let n = SkFindUnitQuadRoots(coeff[1], coeff[2], coeff[3], &mut roots);
        ts[..n].copy_from_slice(&roots[..n]);
        return n;
    }
    debug_assert_ne!(coeff[0], 0.0);
    let inv = 1.0 / coeff[0];
    let a = coeff[1] * inv;
    let b = coeff[2] * inv;
    let c = coeff[3] * inv;
    let q = (a * a - b * 3.0) / 9.0;
    let r = (2.0 * a * a * a - 9.0 * a * b + 27.0 * c) / 54.0;
    let q3 = q * q * q;
    let delta = r * r - q3;
    let a_div3 = a / 3.0;
    if delta < 0.0 {
        let theta = crate::include::private::SkTPin::SkTPin((r / q3.sqrt()), -1.0, 1.0).acos();
        let neg2 = -2.0 * q.sqrt();
        let pi = crate::include::core::SkScalar::SK_ScalarPI;
        ts[0] = crate::include::private::SkTPin::SkTPin(
            (neg2 * (theta / 3.0).cos() - a_div3),
            0.0,
            1.0,
        );
        ts[1] = crate::include::private::SkTPin::SkTPin(
            (neg2 * ((theta + 2.0 * pi) / 3.0).cos() - a_div3),
            0.0,
            1.0,
        );
        ts[2] = crate::include::private::SkTPin::SkTPin(
            (neg2 * ((theta - 2.0 * pi) / 3.0).cos() - a_div3),
            0.0,
            1.0,
        );
        #[cfg(sk_debug)]
        test_collaps_duplicates();
        bubble_sort(ts);
        collaps_duplicates(ts, 3)
    } else {
        let mut a = r.abs() + delta.sqrt();
        a = SkScalarCubeRoot(a);
        if r > 0.0 {
            a = -a;
        }
        if a != 0.0 {
            a += q / a;
        }
        ts[0] = crate::include::private::SkTPin::SkTPin((a - a_div3), 0.0, 1.0);
        1
    }
}

// Source: src/core/SkGeometry.cpp:1021-1030
fn formulate_F1DotF2(src: [f32; 4], coeff: &mut [f32; 4]) {
    let a = src[1] - src[0];
    let b = src[2] - 2.0 * src[1] + src[0];
    let c = src[3] + 3.0 * (src[1] - src[2]) - src[0];
    coeff[0] = c * c;
    coeff[1] = 3.0 * b * c;
    coeff[2] = 2.0 * b * b + c * a;
    coeff[3] = a * b;
}

// Source: src/core/SkGeometry.cpp:1043-1058
pub fn SkFindCubicMaxCurvature(src: &[SkPoint; 4], ts: &mut [f32; 3]) -> usize {
    let mut x = [0.0; 4];
    let mut y = [0.0; 4];
    formulate_F1DotF2([src[0].fX, src[1].fX, src[2].fX, src[3].fX], &mut x);
    formulate_F1DotF2([src[0].fY, src[1].fY, src[2].fY, src[3].fY], &mut y);
    for i in 0..4 {
        x[i] += y[i];
    }
    solve_cubic_poly(&x, ts)
}

// Source: src/core/SkGeometry.cpp:1060-1087
pub fn SkChopCubicAtMaxCurvature(
    src: &[SkPoint; 4],
    dst: Option<&mut [SkPoint; 13]>,
    ts: Option<&mut [f32; 3]>,
) -> usize {
    let mut local = [0.0; 3];
    let ts = ts.unwrap_or(&mut local);
    let mut roots = [0.0; 3];
    let n = SkFindCubicMaxCurvature(src, &mut roots);
    let mut count = 0;
    for &root in &roots[..n] {
        if 0.0 < root && root < 1.0 {
            ts[count] = root;
            count += 1;
        }
    }
    if let Some(dst) = dst {
        if count == 0 {
            dst[..4].copy_from_slice(src);
        } else {
            SkChopCubicAtMultiple(src, dst, &ts[..count]);
        }
    }
    count + 1
}

// Source: src/core/SkGeometry.cpp:1091-1094
fn calc_cubic_precision(src: &[SkPoint; 4]) -> f32 {
    use crate::src::core::SkPointPriv::SkPointPriv;
    (SkPointPriv::DistanceToSqd(src[1], src[0])
        + SkPointPriv::DistanceToSqd(src[2], src[1])
        + SkPointPriv::DistanceToSqd(src[3], src[2]))
        * 1e-8
}

// Source: src/core/SkGeometry.cpp:1098-1107
fn on_same_side(src: &[SkPoint; 4], test: usize, line_index: usize) -> bool {
    let origin = src[line_index];
    let line = src[line_index + 1] - origin;
    let mut crosses = [0.0; 2];
    for i in 0..2 {
        crosses[i] = line.cross(src[test + i] - origin);
    }
    crosses[0] * crosses[1] >= 0.0
}

// Source: src/core/SkGeometry.cpp:1112-1150
pub fn SkFindCubicCusp(src: &[SkPoint; 4]) -> f32 {
    if src[0] == src[1] {
        return -1.0;
    }
    if src[2] == src[3] {
        return -1.0;
    }
    if on_same_side(src, 0, 2) || on_same_side(src, 2, 0) {
        return -1.0;
    }
    let mut roots = [0.0; 3];
    let n = SkFindCubicMaxCurvature(src, &mut roots);
    for &t in &roots[..n] {
        if 0.0 >= t || t >= 1.0 {
            continue;
        }
        let d = eval_cubic_derivative(src, t);
        let magnitude = crate::src::core::SkPointPriv::SkPointPriv::LengthSqd(d);
        let precision = calc_cubic_precision(src);
        if magnitude < precision {
            return t;
        }
    }
    -1.0
}

// Source: src/core/SkGeometry.cpp:1152-1154
fn close_enough_to_zero(x: f64) -> bool {
    x.abs() < 0.00001
}

// Source: src/core/SkGeometry.cpp:1156-1186
fn first_axis_intersection(
    coefficients: &[f64; 8],
    y_direction: bool,
    axis_intercept: f64,
    solution: &mut f64,
) -> bool {
    use crate::src::core::SkCubics::SkCubics;
    let [a, b, c, mut d] = crate::src::core::SkBezierCurves::SkBezierCubic::ConvertToPolynomial(
        coefficients,
        y_direction,
    );
    d -= axis_intercept;
    let mut roots = [0.0; 3];
    let count = SkCubics::RootsValidT(a, b, c, d, &mut roots);
    if count == 0 {
        return false;
    }
    for &r in &roots[..count] {
        if close_enough_to_zero(SkCubics::EvalAt(a, b, c, d, r)) {
            *solution = r;
            return true;
        }
    }
    let count = SkCubics::BinarySearchRootsValidT(a, b, c, d, &mut roots);
    if count == 0 {
        return false;
    }
    for &r in &roots[..count] {
        if close_enough_to_zero(SkCubics::EvalAt(a, b, c, d, r)) {
            *solution = r;
            return true;
        }
    }
    false
}

// Source: src/core/SkGeometry.cpp:1188-1202
pub fn SkChopMonoCubicAtY(src: &[SkPoint; 4], value: f32, dst: &mut [SkPoint; 7]) -> bool {
    let coefficients = [
        src[0].fX as f64,
        src[0].fY as f64,
        src[1].fX as f64,
        src[1].fY as f64,
        src[2].fX as f64,
        src[2].fY as f64,
        src[3].fX as f64,
        src[3].fY as f64,
    ];
    let mut solution = 0.0;
    if first_axis_intersection(&coefficients, true, value as f64, &mut solution) {
        let mut pair = [0.0; 14];
        crate::src::core::SkBezierCurves::SkBezierCubic::Subdivide(
            &coefficients,
            solution,
            &mut pair,
        );
        for i in 0..7 {
            dst[i].fX = pair[2 * i] as f32;
            dst[i].fY = pair[2 * i + 1] as f32;
        }
        return true;
    }
    false
}

// Source: src/core/SkGeometry.cpp:1204-1218
pub fn SkChopMonoCubicAtX(src: &[SkPoint; 4], value: f32, dst: &mut [SkPoint; 7]) -> bool {
    let coefficients = [
        src[0].fX as f64,
        src[0].fY as f64,
        src[1].fX as f64,
        src[1].fY as f64,
        src[2].fX as f64,
        src[2].fY as f64,
        src[3].fX as f64,
        src[3].fY as f64,
    ];
    let mut solution = 0.0;
    if first_axis_intersection(&coefficients, false, value as f64, &mut solution) {
        let mut pair = [0.0; 14];
        crate::src::core::SkBezierCurves::SkBezierCubic::Subdivide(
            &coefficients,
            solution,
            &mut pair,
        );
        for i in 0..7 {
            dst[i].fX = pair[2 * i] as f32;
            dst[i].fY = pair[2 * i + 1] as f32;
        }
        return true;
    }
    false
}

// Source: src/core/SkGeometry.cpp:1251-1260
fn conic_deriv_coeff(src: &[f32; 3], w: f32, coeff: &mut [f32; 3]) {
    let p20 = src[2] - src[0];
    let p10 = src[1] - src[0];
    let wp10 = w * p10;
    coeff[0] = w * p20 - p20;
    coeff[1] = p20 - 2.0 * wp10;
    coeff[2] = wp10;
}

// Source: src/core/SkGeometry.cpp:1262-1275
fn conic_find_extrema(src: &[f32; 3], w: f32, t: &mut f32) -> bool {
    let mut coeff = [0.0; 3];
    conic_deriv_coeff(src, w, &mut coeff);
    let mut roots = [0.0; 2];
    let count = SkFindUnitQuadRoots(coeff[0], coeff[1], coeff[2], &mut roots);
    debug_assert!(count == 0 || count == 1);
    if count == 1 {
        *t = roots[0];
        true
    } else {
        false
    }
}

// Source: src/core/SkGeometry.cpp:1278-1284
fn p3d_interp(src: [f32; 3], t: f32) -> [f32; 3] {
    use crate::include::core::SkScalar::SkScalarInterp;
    let ab = SkScalarInterp(src[0], src[1], t);
    let bc = SkScalarInterp(src[1], src[2], t);
    [ab, SkScalarInterp(ab, bc, t), bc]
}

// Source: src/core/SkGeometry.cpp:1286-1290
fn ratquad_mapTo3D(
    src: &[SkPoint; 3],
    w: f32,
    dst: &mut [crate::src::core::SkPoint3::SkPoint3; 3],
) {
    dst[0].set(src[0].fX * 1.0, src[0].fY * 1.0, 1.0);
    dst[1].set(src[1].fX * w, src[1].fY * w, w);
    dst[2].set(src[2].fX * 1.0, src[2].fY * 1.0, 1.0);
}

// Source: src/core/SkGeometry.cpp:1292-1294
fn project_down(src: crate::src::core::SkPoint3::SkPoint3) -> SkPoint {
    SkPoint::make(src.fX / src.fZ, src.fY / src.fZ)
}

// Source: src/core/SkGeometry.cpp:1297-1326
impl SkConic {
    pub fn chopAt(&self, t: f32, dst: &mut [SkConic; 2]) -> bool {
        let mut tmp = [crate::src::core::SkPoint3::SkPoint3::default(); 3];
        ratquad_mapTo3D(&self.fPts, self.fW, &mut tmp);
        let xs = p3d_interp([tmp[0].fX, tmp[1].fX, tmp[2].fX], t);
        let ys = p3d_interp([tmp[0].fY, tmp[1].fY, tmp[2].fY], t);
        let zs = p3d_interp([tmp[0].fZ, tmp[1].fZ, tmp[2].fZ], t);
        let tmp2 = core::array::from_fn::<_, 3, _>(|i| {
            crate::src::core::SkPoint3::SkPoint3::Make(xs[i], ys[i], zs[i])
        });
        dst[0].fPts[0] = self.fPts[0];
        dst[0].fPts[1] = project_down(tmp2[0]);
        dst[0].fPts[2] = project_down(tmp2[1]);
        dst[1].fPts[0] = dst[0].fPts[2];
        dst[1].fPts[1] = project_down(tmp2[2]);
        dst[1].fPts[2] = self.fPts[2];
        let root = tmp2[1].fZ.sqrt();
        dst[0].fW = tmp2[0].fZ / root;
        dst[1].fW = tmp2[2].fZ / root;
        debug_assert_eq!(
            core::mem::size_of::<SkConic>(),
            7 * core::mem::size_of::<f32>()
        );
        dst.iter()
            .all(|c| c.fPts.iter().all(SkPoint::is_finite) && c.fW.is_finite())
    }
}

// Source-aligned Skia translation. Rust methods use snake_case; fields retain upstream names.

// Source: src/core/SkGeometry.cpp:1328-1358
impl SkConic {
    pub fn chopAtRange(&self, t1: f32, t2: f32, dst: &mut SkConic) {
        if t1 == 0.0 || t2 == 1.0 {
            if t1 == 0.0 && t2 == 1.0 {
                *dst = *self;
                return;
            } else {
                let mut pair = [SkConic::default(); 2];
                if self.chopAt(if t1 != 0.0 { t1 } else { t2 }, &mut pair) {
                    *dst = pair[(t1 != 0.0) as usize];
                    return;
                }
            }
        }
        let coeff = SkConicCoeff::new(self);
        let tt1 = broadcast(t1);
        let axy = coeff.fNumer.eval(tt1);
        let azz = coeff.fDenom.eval(tt1);
        let mid = broadcast((t1 + t2) / 2.0);
        let dxy = coeff.fNumer.eval(mid);
        let dzz = coeff.fDenom.eval(mid);
        let tt2 = broadcast(t2);
        let cxy = coeff.fNumer.eval(tt2);
        let czz = coeff.fDenom.eval(tt2);
        let bxy = times_2(dxy) - (axy + cxy) * 0.5;
        let bzz = times_2(dzz) - (azz + czz) * 0.5;
        dst.fPts[0] = SkPoint::make(axy.fX / azz.fX, axy.fY / azz.fY);
        dst.fPts[1] = SkPoint::make(bxy.fX / bzz.fX, bxy.fY / bzz.fY);
        dst.fPts[2] = SkPoint::make(cxy.fX / czz.fX, cxy.fY / czz.fY);
        let ww = SkPoint::make(
            bzz.fX / (azz.fX * czz.fX).sqrt(),
            bzz.fY / (azz.fY * czz.fY).sqrt(),
        );
        dst.fW = ww.fX;
    }
}

// Source: src/core/SkGeometry.cpp:1360-1362
impl SkConic {
    pub fn evalAt(&self, t: f32) -> SkPoint {
        SkConicCoeff::new(self).eval(t)
    }
}

// Source: src/core/SkGeometry.cpp:1364-1384
impl SkConic {
    pub fn evalTangentAt(&self, t: f32) -> SkPoint {
        if (t == 0.0 && self.fPts[0] == self.fPts[1]) || (t == 1.0 && self.fPts[1] == self.fPts[2])
        {
            return self.fPts[2] - self.fPts[0];
        }
        let p0 = self.fPts[0];
        let p1 = self.fPts[1];
        let p2 = self.fPts[2];
        let ww = broadcast(self.fW);
        let p20 = p2 - p0;
        let p10 = p1 - p0;
        let c = mul_lanes(ww, p10);
        let a = mul_lanes(ww, p20) - p20;
        let b = p20 - c - c;
        SkQuadCoeff::new(a, b, c).eval(broadcast(t))
    }
}

// Source: src/core/SkGeometry.cpp:1386-1395
impl SkConic {
    pub fn evalAtWithTangent(
        &self,
        t: f32,
        pt: Option<&mut SkPoint>,
        tangent: Option<&mut SkPoint>,
    ) {
        debug_assert!(t >= 0.0 && t <= 1.0);
        if let Some(pt) = pt {
            *pt = self.evalAt(t);
        }
        if let Some(tangent) = tangent {
            *tangent = self.evalTangentAt(t);
        }
    }
}

// Source: src/core/SkGeometry.cpp:1397-1399
fn subdivide_w_value(w: f32) -> f32 {
    (0.5 + w * 0.5).sqrt()
}

// Source: src/core/SkGeometry.cpp:1402-1428
#[allow(unexpected_cfgs)]
#[cfg(SK_SUPPORT_LEGACY_CONIC_CHOP)]
impl SkConic {
    pub fn chop(&self, dst: &mut [SkConic; 2]) {
        let scale = 1.0 / (1.0 + self.fW);
        let new_w = subdivide_w_value(self.fW);
        let p0 = self.fPts[0];
        let p1 = self.fPts[1];
        let p2 = self.fPts[2];
        let ww = broadcast(self.fW);
        let wp1 = mul_lanes(ww, p1);
        let mut midpoint = (p0 + times_2(wp1) + p2) * scale * 0.5;
        if !midpoint.is_finite() {
            let w = self.fW as f64;
            let w2 = w * 2.0;
            let half = 1.0 / (1.0 + w) * 0.5;
            midpoint.fX =
                ((self.fPts[0].fX as f64 + w2 * self.fPts[1].fX as f64 + self.fPts[2].fX as f64)
                    * half) as f32;
            midpoint.fY =
                ((self.fPts[0].fY as f64 + w2 * self.fPts[1].fY as f64 + self.fPts[2].fY as f64)
                    * half) as f32;
        }
        dst[0].fPts[0] = self.fPts[0];
        dst[0].fPts[1] = (p0 + wp1) * scale;
        dst[0].fPts[2] = midpoint;
        dst[1].fPts[0] = midpoint;
        dst[1].fPts[1] = (wp1 + p2) * scale;
        dst[1].fPts[2] = self.fPts[2];
        dst[0].fW = new_w;
        dst[1].fW = new_w;
    }
}

// Source: src/core/SkGeometry.cpp:1430-1462
#[allow(unexpected_cfgs)]
#[cfg(not(SK_SUPPORT_LEGACY_CONIC_CHOP))]
impl SkConic {
    pub fn chop(&self, dst: &mut [SkConic; 2]) {
        let scale = 1.0 / (1.0 + self.fW);
        let t0 = self.fPts[0] * scale;
        let t1 = self.fPts[1] * (self.fW * scale);
        let t2 = self.fPts[2] * scale;
        let p1 = t0 + t1;
        let p3 = t1 + t2;
        let p2 = t0 * 0.5 + t1 + t2 * 0.5;
        crate::SkASSERT!(@source b"src/core/SkGeometry.cpp\0".as_ptr(),1451; @condition b"p1.isFinite() && p2.isFinite() && p3.isFinite()\0".as_ptr(); p1.is_finite() && p2.is_finite() && p3.is_finite());
        dst[0].fPts = [self.fPts[0], p1, p2];
        dst[1].fPts = [p2, p3, self.fPts[2]];
        dst[0].fW = subdivide_w_value(self.fW);
        dst[1].fW = dst[0].fW;
    }
}

// Source: src/core/SkGeometry.cpp:1469-1473
fn conic_quad_error(c: &SkConic) -> SkPoint {
    let a = c.fW - 1.0;
    let k = a / (4.0 * (2.0 + a));
    SkPoint::make(
        k * (c.fPts[0].fX - 2.0 * c.fPts[1].fX + c.fPts[2].fX),
        k * (c.fPts[0].fY - 2.0 * c.fPts[1].fY + c.fPts[2].fY),
    )
}

// Source: src/core/SkGeometry.cpp:1475-1478
impl SkConic {
    pub fn computeAsQuadError(&self, err: &mut SkPoint) {
        *err = conic_quad_error(self);
    }
}

// Source: src/core/SkGeometry.cpp:1480-1483
impl SkConic {
    pub fn asQuadTol(&self, tol: f32) -> bool {
        let p = conic_quad_error(self);
        p.fX * p.fX + p.fY * p.fY <= tol * tol
    }
}

// Source: src/core/SkGeometry.cpp:1486-1486
const K_MAX_CONIC_TO_QUAD_POW2: usize = 5;

// Source: src/core/SkGeometry.cpp:1488-1490
fn bad_conic_w(w: f32) -> bool {
    w < 0.0 || !w.is_finite()
}

// Source: src/core/SkGeometry.cpp:1492-1525
impl SkConic {
    pub fn computeQuadPOW2(&self, tol: f32) -> usize {
        if tol < 0.0
            || !tol.is_finite()
            || !self.fPts.iter().all(SkPoint::is_finite)
            || bad_conic_w(self.fW)
        {
            return 0;
        }
        let error_vector = conic_quad_error(self);
        let mut error =
            (error_vector.fX * error_vector.fX + error_vector.fY * error_vector.fY).sqrt();
        let mut pow2 = 0;
        while pow2 < K_MAX_CONIC_TO_QUAD_POW2 {
            if error <= tol {
                break;
            }
            error *= 0.25;
            pow2 += 1;
        }
        if false {
            let err =
                (error_vector.fX * error_vector.fX + error_vector.fY * error_vector.fY).sqrt();
            if err <= tol {
                return 0;
            }
            let tol2 = tol * tol;
            if tol2 == 0.0 {
                return K_MAX_CONIC_TO_QUAD_POW2;
            }
            let fpow2 = ((error_vector.fX * error_vector.fX + error_vector.fY * error_vector.fY)
                / tol2)
                .log2()
                * 0.25;
            let alt_pow2 = fpow2.ceil() as usize;
            if alt_pow2 != pow2 {
                // Source: src/core/SkGeometry.cpp:1520-1520
                unsafe {
                    crate::SkDebugf!(@source b"src/core/SkGeometry.cpp\0".as_ptr(),1520;
                        b"pow2 %d altPow2 %d fbits %g err %g tol %g\n\0".as_ptr(),
                        pow2 as i32,alt_pow2 as i32,fpow2 as f64,err as f64,tol as f64,
                    );
                }
            }
            pow2 = alt_pow2;
        }
        pow2
    }
}

// Source: src/core/SkGeometry.cpp:1529-1531
fn between(a: f32, b: f32, c: f32) -> bool {
    (a - b) * (c - b) <= 0.0
}

// Source: src/core/SkGeometry.cpp:1533-1572
fn subdivide(src: &SkConic, pts: &mut [SkPoint], level: usize) -> usize {
    if level == 0 {
        pts[..2].copy_from_slice(&src.fPts[1..]);
        2
    } else {
        let mut dst = [SkConic::default(); 2];
        src.chop(&mut dst);
        let start_y = src.fPts[0].fY;
        let end_y = src.fPts[2].fY;
        if between(start_y, src.fPts[1].fY, end_y) {
            let mid_y = dst[0].fPts[2].fY;
            if !between(start_y, mid_y, end_y) {
                let closer = if (mid_y - start_y).abs() < (mid_y - end_y).abs() {
                    start_y
                } else {
                    end_y
                };
                dst[0].fPts[2].fY = closer;
                dst[1].fPts[0].fY = closer;
            }
            if !between(start_y, dst[0].fPts[1].fY, dst[0].fPts[2].fY) {
                dst[0].fPts[1].fY = start_y;
            }
            if !between(dst[1].fPts[0].fY, dst[1].fPts[1].fY, end_y) {
                dst[1].fPts[1].fY = end_y;
            }
            crate::SkASSERT!(@source b"src/core/SkGeometry.cpp\0".as_ptr(),1564; @condition b"between(startY, dst[0].fPts[1].fY, dst[0].fPts[2].fY)\0".as_ptr(); between(start_y, dst[0].fPts[1].fY, dst[0].fPts[2].fY));
            crate::SkASSERT!(@source b"src/core/SkGeometry.cpp\0".as_ptr(),1565; @condition b"between(dst[0].fPts[1].fY, dst[0].fPts[2].fY, dst[1].fPts[1].fY)\0".as_ptr(); between(
                dst[0].fPts[1].fY,
                dst[0].fPts[2].fY,
                dst[1].fPts[1].fY
            ));
            crate::SkASSERT!(@source b"src/core/SkGeometry.cpp\0".as_ptr(),1566; @condition b"between(dst[0].fPts[2].fY, dst[1].fPts[1].fY, endY)\0".as_ptr(); between(dst[0].fPts[2].fY, dst[1].fPts[1].fY, end_y));
        }
        let n = subdivide(&dst[0], pts, level - 1);
        n + subdivide(&dst[1], &mut pts[n..], level - 1)
    }
}

// Source: src/core/SkGeometry.cpp:1574-1614
impl SkConic {
    pub fn chopIntoQuadsPOW2(&self, pts: &mut [SkPoint], mut pow2: usize) -> usize {
        debug_assert!(pow2 <= K_MAX_CONIC_TO_QUAD_POW2);
        if bad_conic_w(self.fW) {
            pow2 = 0;
        }
        assert!(pts.len() >= 2 * (1 << pow2) + 1);
        pts[0] = self.fPts[0];
        let mut special_lines = false;
        if pow2 == K_MAX_CONIC_TO_QUAD_POW2 {
            let mut dst = [SkConic::default(); 2];
            self.chop(&mut dst);
            use crate::src::core::SkPointPriv::SkPointPriv;
            if SkPointPriv::EqualsWithinTolerance(dst[0].fPts[1], dst[0].fPts[2])
                && SkPointPriv::EqualsWithinTolerance(dst[1].fPts[0], dst[1].fPts[1])
            {
                pts[1..4].fill(dst[0].fPts[1]);
                pts[4] = dst[1].fPts[2];
                pow2 = 1;
                special_lines = true;
            }
        }
        if !special_lines {
            let count = subdivide(self, &mut pts[1..], pow2);
            debug_assert_eq!(count, 2 * (1 << pow2));
        }
        let quad_count = 1 << pow2;
        let point_count = 2 * quad_count + 1;
        if !pts[..point_count].iter().all(SkPoint::is_finite) {
            pts[1..point_count - 1].fill(self.fPts[1]);
        }
        quad_count
    }
}

// Source: src/core/SkGeometry.cpp:1611-1646
impl SkConic {
    pub fn findMidTangent(&self) -> f32 {
        let tan0 = self.fPts[1] - self.fPts[0];
        let tan1 = self.fPts[2] - self.fPts[1];
        let bisector = SkFindBisector(tan0, -tan1);
        let a = (self.fPts[2] - self.fPts[0]) * (self.fW - 1.0);
        let b = (self.fPts[2] - self.fPts[0]) - (self.fPts[1] - self.fPts[0]) * (self.fW * 2.0);
        let c = (self.fPts[1] - self.fPts[0]) * self.fW;
        solve_quadratic_equation_for_midtangent(bisector.dot(a), bisector.dot(b), bisector.dot(c))
    }
}

// Source: src/core/SkGeometry.cpp:1648-1650
impl SkConic {
    pub fn findXExtrema(&self, t: &mut f32) -> bool {
        conic_find_extrema(
            &[self.fPts[0].fX, self.fPts[1].fX, self.fPts[2].fX],
            self.fW,
            t,
        )
    }
}

// Source: src/core/SkGeometry.cpp:1652-1654
impl SkConic {
    pub fn findYExtrema(&self, t: &mut f32) -> bool {
        conic_find_extrema(
            &[self.fPts[0].fY, self.fPts[1].fY, self.fPts[2].fY],
            self.fW,
            t,
        )
    }
}

// Source: src/core/SkGeometry.cpp:1656-1672
impl SkConic {
    pub fn chopAtXExtrema(&self, dst: &mut [SkConic; 2]) -> bool {
        let mut t = 0.0;
        if self.findXExtrema(&mut t) {
            if !self.chopAt(t, dst) {
                return false;
            }
            let value = dst[0].fPts[2].fX;
            dst[0].fPts[1].fX = value;
            dst[1].fPts[0].fX = value;
            dst[1].fPts[1].fX = value;
            return true;
        }
        false
    }
}

// Source: src/core/SkGeometry.cpp:1674-1690
impl SkConic {
    pub fn chopAtYExtrema(&self, dst: &mut [SkConic; 2]) -> bool {
        let mut t = 0.0;
        if self.findYExtrema(&mut t) {
            if !self.chopAt(t, dst) {
                return false;
            }
            let value = dst[0].fPts[2].fY;
            dst[0].fPts[1].fY = value;
            dst[1].fPts[0].fY = value;
            dst[1].fPts[1].fY = value;
            return true;
        }
        false
    }
}

// Source: src/core/SkGeometry.cpp:1692-1706
impl SkConic {
    pub fn computeTightBounds(&self, bounds: &mut crate::src::core::SkRect::SkRect) {
        let mut pts = [SkPoint::default(); 4];
        pts[0] = self.fPts[0];
        pts[1] = self.fPts[2];
        let mut count = 2;
        let mut t = 0.0;
        if self.findXExtrema(&mut t) {
            pts[count] = self.evalAt(t);
            count += 1;
        }
        if self.findYExtrema(&mut t) {
            pts[count] = self.evalAt(t);
            count += 1;
        }
        *bounds = crate::src::core::SkRect::SkRect::BoundsOrEmpty(&pts[..count]);
    }
}

// Source: src/core/SkGeometry.cpp:1708-1710
impl SkConic {
    pub fn computeFastBounds(&self, bounds: &mut crate::src::core::SkRect::SkRect) {
        *bounds = crate::src::core::SkRect::SkRect::BoundsOrEmpty(&self.fPts);
    }
}

// Source: src/core/SkGeometry.cpp:1719-1736
impl SkConic {
    pub fn TransformW(
        pts: &[SkPoint; 3],
        w: f32,
        matrix: &crate::src::core::SkMatrix::SkMatrix,
    ) -> f32 {
        if !matrix.hasPerspective() {
            return w;
        }
        let mut src = [crate::src::core::SkPoint3::SkPoint3::default(); 3];
        let mut dst = src;
        ratquad_mapTo3D(pts, w, &mut src);
        matrix.mapHomogeneousPoints(&mut dst, &src);
        let w0 = dst[0].fZ as f64;
        let w1 = dst[1].fZ as f64;
        let w2 = dst[2].fZ as f64;
        ((w1 * w1) / (w0 * w2)).sqrt() as f32
    }
}

// Source: src/core/SkGeometry.cpp:1738-1827
impl SkConic {
    pub fn BuildUnitArc(
        start: &SkPoint,
        stop: &SkPoint,
        dir: crate::include::core::SkPathTypes::SkPathDirection,
        user_matrix: Option<&crate::src::core::SkMatrix::SkMatrix>,
        dst: &mut [SkConic; 5],
    ) -> usize {
        use crate::include::core::SkPathTypes::SkPathDirection;
        use crate::include::core::SkScalar::{SK_ScalarNearlyZero, SK_ScalarRoot2Over2};
        let x = SkPoint::dot_product(*start, *stop);
        let mut y = SkPoint::cross_product(*start, *stop);
        let abs_y = y.abs();
        if abs_y <= SK_ScalarNearlyZero
            && x > 0.0
            && ((y >= 0.0 && dir == SkPathDirection::kCW)
                || (y <= 0.0 && dir == SkPathDirection::kCCW))
        {
            return 0;
        }
        if dir == SkPathDirection::kCCW {
            y = -y;
        }
        let mut quadrant = 0;
        if y == 0.0 {
            quadrant = 2;
            debug_assert!((x + 1.0).abs() <= SK_ScalarNearlyZero);
        } else if x == 0.0 {
            debug_assert!(abs_y - 1.0 <= SK_ScalarNearlyZero);
            quadrant = if y > 0.0 { 1 } else { 3 };
        } else {
            if y < 0.0 {
                quadrant += 2;
            }
            if (x < 0.0) != (y < 0.0) {
                quadrant += 1;
            }
        }
        let q = [
            SkPoint::make(1.0, 0.0),
            SkPoint::make(1.0, 1.0),
            SkPoint::make(0.0, 1.0),
            SkPoint::make(-1.0, 1.0),
            SkPoint::make(-1.0, 0.0),
            SkPoint::make(-1.0, -1.0),
            SkPoint::make(0.0, -1.0),
            SkPoint::make(1.0, -1.0),
        ];
        let mut count = quadrant;
        for i in 0..count {
            dst[i].set(
                (&q[2 * i..2 * i + 3]).try_into().unwrap(),
                SK_ScalarRoot2Over2,
            );
        }
        let final_p = SkPoint::make(x, y);
        let last = q[quadrant * 2];
        let dot = SkPoint::dot_product(last, final_p);
        if dot.is_nan() {
            return 0;
        }
        debug_assert!(dot >= 0.0 && dot <= 1.0 + SK_ScalarNearlyZero);
        if dot < 1.0 {
            let mut off = SkPoint::make(last.fX + x, last.fY + y);
            let half = ((1.0 + dot) / 2.0).sqrt();
            off.set_length(1.0 / half);
            if !crate::src::core::SkPointPriv::SkPointPriv::EqualsWithinTolerance(last, off) {
                dst[count].set(&[last, off, final_p], half);
                count += 1;
            }
        }
        let mut matrix = crate::src::core::SkMatrix::SkMatrix::default();
        matrix.setSinCos(start.fY, start.fX);
        if dir == SkPathDirection::kCCW {
            matrix.preScale(1.0, -1.0);
        }
        if let Some(user) = user_matrix {
            matrix.postConcat(user);
        }
        for conic in &mut dst[..count] {
            matrix.mapPointsInPlace(&mut conic.fPts);
        }
        count
    }
}
