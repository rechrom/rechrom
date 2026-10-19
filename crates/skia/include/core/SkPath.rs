// Translation status: complete (immutable geometry/value dependency closure only)
// Copyright 2025 Google LLC. BSD-style license in LICENSE.
//! Public immutable SkPath value closure used by compositor property values.
//! Unlike the existing tiny-skia path ABI it retains empty/degenerate paths,
//! conics, and the native oval/rrect shape tag. No raster playback lives here.
#![allow(non_camel_case_types, non_upper_case_globals, dead_code)]
pub use super::SkPathTypes::{SkPathDirection, SkPathVerb};
use super::SkRRect::{
    deduce_rrect, PointIterator, RawRRect, SkRRect, Type as RRectType, CONIC_START, LINE_START,
};
pub use super::SkRRect::{SkRRectBounds as SkPathBounds, SkRRectVector as SkPathPoint};
use std::sync::{
    atomic::{AtomicU8, Ordering},
    Arc, OnceLock,
};
use SkPathVerb::{kClose as Z, kConic as K, kCubic as C, kLine as L, kMove as M, kQuad as Q};
// Source: include/core/SkPathTypes.h:16-44.
#[repr(u8)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SkPathFillType {
    #[default]
    kWinding = 0,
    kEvenOdd = 1,
    kInverseWinding = 2,
    kInverseEvenOdd = 3,
}
// Source: src/core/SkPathData.h:171-188, .cpp:320-346,355-367,485-506.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Shape {
    General,
    Oval { dir: SkPathDirection, index: usize },
    RRect { dir: SkPathDirection, index: usize },
}
#[derive(Debug)]
struct PathData {
    points: Vec<SkPathPoint>,
    verbs: Vec<SkPathVerb>,
    conics: Vec<f32>,
    bounds: SkPathBounds,
    mask: u8,
    shape: Shape,
    // Source: src/core/SkPathData.h:197-204; SkPathData.cpp:147-150.
    // The path allocation is immutable; only this source cache is mutable.
    convexity: AtomicU8,
}
impl PathData {
    fn empty() -> Self {
        Self {
            points: Vec::new(),
            verbs: Vec::new(),
            conics: Vec::new(),
            bounds: SkPathBounds::default(),
            mask: 0,
            shape: Shape::General,
            convexity: AtomicU8::new(PathConvexity::Unknown as u8),
        }
    }
}
fn empty_data() -> &'static Arc<PathData> {
    static EMPTY: OnceLock<Arc<PathData>> = OnceLock::new();
    EMPTY.get_or_init(|| Arc::new(PathData::empty()))
}
// Source: src/core/SkPath.cpp:736-758.
fn error_data() -> &'static Arc<PathData> {
    static ERROR: OnceLock<Arc<PathData>> = OnceLock::new();
    ERROR.get_or_init(|| Arc::new(PathData::empty()))
}
// Source: src/core/SkPath.cpp:32-62.
#[derive(Clone, Debug)]
pub struct SkPath {
    data: Arc<PathData>,
    fill_type: SkPathFillType,
    is_volatile: bool,
}
impl Default for SkPath {
    fn default() -> Self {
        Self {
            data: Arc::clone(empty_data()),
            fill_type: SkPathFillType::kWinding,
            is_volatile: false,
        }
    }
}
impl PartialEq for SkPath {
    fn eq(&self, b: &Self) -> bool {
        self.fill_type == b.fill_type
            && (Arc::ptr_eq(&self.data, &b.data)
                || (self.data.points == b.data.points
                    && self.data.verbs == b.data.verbs
                    && self.data.conics == b.data.conics))
    }
}
// Source: src/core/SkPathData.cpp:75-137.
fn valid_path_data(points: &[SkPathPoint], verbs: &[SkPathVerb], conics: &[f32]) -> bool {
    if verbs.is_empty() {
        return points.is_empty() && conics.is_empty();
    }
    if verbs[0] != M {
        return false;
    }
    let mut prev = M;
    let mut pc = 1;
    let mut cc = 0;
    for &v in &verbs[1..] {
        if prev == M && v == M {
            return false;
        }
        if prev == Z && v != M {
            return false;
        }
        pc += [1, 1, 2, 2, 3, 0][v as usize];
        cc += usize::from(v == K);
        prev = v;
    }
    if points.len() != pc || conics.len() != cc {
        return false;
    }
    for w in conics {
        if !(*w >= 0. && w.is_finite()) {
            return false;
        }
    }
    true
}
// Source: src/core/SkPathPriv.h:418-432; SkRect.cpp:49-111.
fn trimmed_bounds(points: &[SkPathPoint], verbs: &[SkPathVerb]) -> Option<SkPathBounds> {
    let mut points = points;
    if verbs.len() > 1 && verbs.last() == Some(&M) {
        let p = points.last()?;
        if !p.x.is_finite() || !p.y.is_finite() {
            return None;
        }
        points = &points[..points.len() - 1];
    }
    if points.is_empty() {
        return Some(SkPathBounds::default());
    }
    let first = points[0];
    let mut b = SkPathBounds::MakeLTRB(first.x, first.y, first.x, first.y);
    let (mut nx, mut ny) = (0f32, 0f32);
    for p in points {
        b.left = p.x.min(b.left);
        b.top = p.y.min(b.top);
        b.right = p.x.max(b.right);
        b.bottom = p.y.max(b.bottom);
        nx *= p.x;
        ny *= p.y;
    }
    if nx == 0. && ny == 0. {
        Some(b)
    } else {
        None
    }
}
// Source: src/core/SkPathRaw.cpp:13-30.
fn segment_mask(verbs: &[SkPathVerb]) -> u8 {
    verbs
        .iter()
        .fold(0, |mask, v| mask | [0, 1, 2, 4, 8, 0][*v as usize])
}
impl SkPath {
    // Source: src/core/SkPath.cpp:80-110; SkPath.h:427-441.
    pub fn getBounds(&self) -> SkPathBounds {
        self.data.bounds
    }
    pub fn points(&self) -> &[SkPathPoint] {
        &self.data.points
    }
    pub fn verbs(&self) -> &[SkPathVerb] {
        &self.data.verbs
    }
    pub fn conicWeights(&self) -> &[f32] {
        &self.data.conics
    }
    pub fn countPoints(&self) -> i32 {
        self.points().len().try_into().expect("SkPath point limit")
    }
    pub fn countVerbs(&self) -> i32 {
        self.verbs().len().try_into().expect("SkPath verb limit")
    }
    pub fn getFillType(&self) -> SkPathFillType {
        self.fill_type
    }
    pub fn setFillType(&mut self, fill: SkPathFillType) {
        self.fill_type = fill;
    }
    pub fn isInverseFillType(&self) -> bool {
        self.fill_type as u8 & 2 != 0
    }
    pub fn isEmpty(&self) -> bool {
        self.verbs().is_empty()
    }
    pub fn isFinite(&self) -> bool {
        !Arc::ptr_eq(&self.data, error_data())
    }
    pub fn isValid(&self) -> bool {
        self.isFinite()
    }
    // Source: include/core/SkPath.h:345-347.
    pub fn isVolatile(&self) -> bool {
        self.is_volatile
    }
    // Source: include/core/SkPath.h:704-707.
    pub fn setIsVolatile(&mut self, value: bool) -> &mut Self {
        self.is_volatile = value;
        self
    }
    // Source: src/core/SkPath.cpp:294-296,381-389,638-654;
    // src/core/SkPathData.cpp:425-440.
    pub fn isConvex(&self) -> bool {
        let mut convexity = PathConvexity::from_u8(self.data.convexity.load(Ordering::Relaxed));
        if convexity == PathConvexity::Unknown {
            convexity = self.compute_convexity();
        }
        debug_assert!(convexity != PathConvexity::Unknown);
        convexity.is_convex()
    }
    fn compute_convexity(&self) -> PathConvexity {
        let c = PathConvexity::from_u8(self.data.convexity.load(Ordering::Relaxed));
        if c != PathConvexity::Unknown {
            return c;
        }
        let mut c = PathConvexity::Concave;
        if self.isFinite() {
            c = compute_convexity(self.points(), self.verbs(), self.conicWeights());
        }
        debug_assert!(c != PathConvexity::Unknown);
        self.data.convexity.store(c as u8, Ordering::Relaxed);
        c
    }
    // Source: src/core/SkPathData.cpp:225-252,332-346,513-525.
    pub fn Raw(
        points: &[SkPathPoint],
        verbs: &[SkPathVerb],
        conics: &[f32],
        fill: SkPathFillType,
        is_volatile: bool,
    ) -> Self {
        if !valid_path_data(points, verbs, conics) {
            return Self {
                data: Arc::clone(error_data()),
                fill_type: fill,
                is_volatile,
            };
        }
        let Some(bounds) = trimmed_bounds(points, verbs) else {
            return Self {
                data: Arc::clone(error_data()),
                fill_type: fill,
                is_volatile,
            };
        };
        Self {
            data: Arc::new(PathData {
                points: points.to_vec(),
                verbs: verbs.to_vec(),
                conics: conics.to_vec(),
                bounds,
                mask: segment_mask(verbs),
                shape: Shape::General,
                convexity: AtomicU8::new(PathConvexity::Unknown as u8),
            }),
            fill_type: fill,
            is_volatile,
        }
    }
    fn shape_data(
        points: Vec<SkPathPoint>,
        verbs: Vec<SkPathVerb>,
        conics: Vec<f32>,
        bounds: SkPathBounds,
        shape: Shape,
    ) -> Self {
        let mask = segment_mask(&verbs);
        Self {
            data: Arc::new(PathData {
                points,
                verbs,
                conics,
                bounds,
                mask,
                shape,
                // Source: src/core/SkPathData.cpp:356-367,378-385.
                convexity: AtomicU8::new(match shape {
                    Shape::General => PathConvexity::Unknown,
                    Shape::Oval { dir, .. } | Shape::RRect { dir, .. } => {
                        PathConvexity::from_direction(dir)
                    }
                } as u8),
            }),
            fill_type: SkPathFillType::kWinding,
            is_volatile: false,
        }
    }
    // Source: src/core/SkPath.cpp:112-120; SkPathData.cpp:369-386;
    //         SkPathRawShapes.cpp:15-88; SkPathMakers.h:15-64.
    pub fn Rect(
        bounds: SkPathBounds,
        fill: SkPathFillType,
        dir: SkPathDirection,
        index: usize,
    ) -> Self {
        if !finite_bounds(bounds) {
            return Self {
                data: Arc::clone(error_data()),
                fill_type: fill,
                is_volatile: false,
            };
        }
        let mut iter = PointIterator::rect(bounds, dir, index & 3);
        let points = (0..4)
            .map(|i| if i == 0 { iter.current() } else { iter.next() })
            .collect();
        let mut path = Self::shape_data(
            points,
            vec![M, L, L, L, Z],
            Vec::new(),
            sorted(bounds),
            Shape::General,
        );
        path.fill_type = fill;
        path
    }
    pub fn Oval(bounds: SkPathBounds, dir: SkPathDirection, index: usize) -> Self {
        if !finite_bounds(bounds) {
            return Self {
                data: Arc::clone(error_data()),
                fill_type: SkPathFillType::kWinding,
                is_volatile: false,
            };
        }
        let index = index & 3;
        let mut oval = PointIterator::oval(bounds, dir, index);
        let mut rect = PointIterator::rect(
            bounds,
            dir,
            index + usize::from(dir == SkPathDirection::kCCW),
        );
        let mut points = Vec::with_capacity(9);
        points.push(oval.current());
        for _ in 0..4 {
            points.push(rect.next());
            points.push(oval.next());
        }
        Self::shape_data(
            points,
            vec![M, K, K, K, K, Z],
            vec![std::f32::consts::FRAC_1_SQRT_2; 4],
            sorted(bounds),
            Shape::Oval { dir, index },
        )
    }
    // Source: src/core/SkPath.cpp:122-140,698-706; SkPathPriv.h:47-56;
    //         SkPathData.cpp:388-399; SkPathRawShapes.cpp:173-185.
    pub fn RRect(rr: &SkRRect, dir: SkPathDirection, index: usize) -> Self {
        let index = index & 7;
        let bounds = rr.getBounds();
        match rr.getType() {
            RRectType::kRect_Type | RRectType::kEmpty_Type => {
                return Self::Rect(bounds, SkPathFillType::kWinding, dir, (index + 1) / 2)
            }
            RRectType::kOval_Type => return Self::Oval(bounds, dir, index / 2),
            _ => {}
        }
        if !rr.isValid() {
            return Self {
                data: Arc::clone(error_data()),
                fill_type: SkPathFillType::kWinding,
                is_volatile: false,
            };
        }
        let raw = RawRRect::with_direction(rr, dir == SkPathDirection::kCW, index);
        let verbs: &[SkPathVerb] = if raw.starts_conic {
            &CONIC_START
        } else {
            &LINE_START
        };
        let mut path = Self::Raw(
            &raw.storage[..raw.point_count],
            verbs,
            &[std::f32::consts::FRAC_1_SQRT_2; 4],
            SkPathFillType::kWinding,
            false,
        );
        if path.isFinite() {
            Arc::get_mut(&mut path.data).unwrap().shape = Shape::RRect { dir, index };
            // Source: src/core/SkPathData.cpp:356-367,388-399.
            path.data
                .convexity
                .store(PathConvexity::from_direction(dir) as u8, Ordering::Relaxed);
        }
        path
    }
    pub fn FromRRect(rr: &SkRRect) -> Self {
        Self::RRect(rr, SkPathDirection::kCW, 6)
    }
    // Source: src/core/SkPath.cpp:317-336; SkPathData.cpp:485-506.
    pub fn isOval(&self) -> Option<SkPathBounds> {
        if matches!(self.data.shape, Shape::Oval { .. }) {
            Some(self.getBounds())
        } else {
            None
        }
    }
    pub fn isRRect(&self) -> Option<SkRRect> {
        if matches!(self.data.shape, Shape::RRect { .. }) {
            Some(deduce_rrect(
                self.data.bounds,
                &self.data.points,
                &self.data.verbs,
            ))
        } else {
            None
        }
    }
    pub fn isRect(&self) -> Option<RectContour> {
        rect_contour(self.points(), self.verbs(), self.data.mask, false)
    }
}
fn finite_bounds(b: SkPathBounds) -> bool {
    [b.left, b.top, b.right, b.bottom]
        .iter()
        .all(|v| v.is_finite())
}
fn min(a: f32, b: f32) -> f32 {
    if b < a {
        b
    } else {
        a
    }
}
fn max(a: f32, b: f32) -> f32 {
    if a < b {
        b
    } else {
        a
    }
}
fn sorted(b: SkPathBounds) -> SkPathBounds {
    SkPathBounds::MakeLTRB(
        min(b.left, b.right),
        min(b.top, b.bottom),
        max(b.left, b.right),
        max(b.top, b.bottom),
    )
}
fn sub(a: SkPathPoint, b: SkPathPoint) -> SkPathPoint {
    SkPathPoint::new(a.x - b.x, a.y - b.y)
}

// Source: src/core/SkPathEnums.h:17-35,48-68.
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PathConvexity {
    ConvexCW,
    ConvexCCW,
    ConvexDegenerate,
    Concave,
    Unknown,
}
impl PathConvexity {
    fn from_u8(v: u8) -> Self {
        match v {
            0 => Self::ConvexCW,
            1 => Self::ConvexCCW,
            2 => Self::ConvexDegenerate,
            3 => Self::Concave,
            4 => Self::Unknown,
            _ => unreachable!("native convexity cache"),
        }
    }
    fn is_convex(self) -> bool {
        matches!(
            self,
            Self::ConvexCW | Self::ConvexCCW | Self::ConvexDegenerate
        )
    }
    fn from_direction(d: SkPathDirection) -> Self {
        match d {
            SkPathDirection::kCW => Self::ConvexCW,
            SkPathDirection::kCCW => Self::ConvexCCW,
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum FirstDirection {
    CW,
    CCW,
    Unknown,
}
impl FirstDirection {
    fn to_convexity(self) -> PathConvexity {
        match self {
            Self::CW => PathConvexity::ConvexCW,
            Self::CCW => PathConvexity::ConvexCCW,
            Self::Unknown => PathConvexity::ConvexDegenerate,
        }
    }
}

// Source: src/core/SkPathPriv.cpp:420-430.
fn scalar_sign(x: f32) -> i32 {
    i32::from(x < 0.)
}
const VALUE_NEVER_RETURNED_BY_SIGN: i32 = 2;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum DirChange {
    Unknown,
    Left,
    Right,
    Straight,
    Backwards,
    Invalid,
}

// Source: src/core/SkPathPriv.cpp:432-457,560-570.
struct Convexicator {
    first_pt: SkPathPoint,
    first_vec: SkPathPoint,
    last_pt: SkPathPoint,
    last_vec: SkPathPoint,
    expected_dir: DirChange,
    first_direction: FirstDirection,
    reversals: i32,
    is_finite: bool,
}
impl Convexicator {
    fn new() -> Self {
        Self {
            first_pt: SkPathPoint::default(),
            first_vec: SkPathPoint::default(),
            last_pt: SkPathPoint::default(),
            last_vec: SkPathPoint::default(),
            expected_dir: DirChange::Invalid,
            first_direction: FirstDirection::Unknown,
            reversals: 0,
            is_finite: true,
        }
    }
    fn get_first_direction(&self) -> FirstDirection {
        self.first_direction
    }
    fn set_move_pt(&mut self, pt: SkPathPoint) {
        self.first_pt = pt;
        self.last_pt = pt;
        self.expected_dir = DirChange::Invalid;
    }
    fn add_pt(&mut self, pt: SkPathPoint) -> bool {
        if self.last_pt == pt {
            return true;
        }
        if self.first_pt == self.last_pt
            && self.expected_dir == DirChange::Invalid
            && self.last_vec == SkPathPoint::new(0., 0.)
        {
            self.last_vec = sub(pt, self.last_pt);
            self.first_vec = self.last_vec;
        } else if !self.add_vec(sub(pt, self.last_pt)) {
            return false;
        }
        self.last_pt = pt;
        true
    }
    // Source: src/core/SkPathPriv.cpp:459-498.
    fn is_concave_by_sign(points: &[SkPathPoint]) -> bool {
        if points.len() <= 3 {
            return false;
        }
        let mut index = 1;
        let mut curr_pt = points[0];
        let first_pt = curr_pt;
        let (mut dxes, mut dyes) = (0, 0);
        let (mut last_sx, mut last_sy) =
            (VALUE_NEVER_RETURNED_BY_SIGN, VALUE_NEVER_RETURNED_BY_SIGN);
        for outer_loop in 0..2 {
            while index != points.len() {
                let point = if outer_loop == 0 {
                    points[index]
                } else {
                    first_pt
                };
                let vec = sub(point, curr_pt);
                if vec != SkPathPoint::new(0., 0.) {
                    if !(vec.x.is_finite() && vec.y.is_finite()) {
                        return true;
                    }
                    let sx = scalar_sign(vec.x);
                    let sy = scalar_sign(vec.y);
                    dxes += i32::from(sx != last_sx);
                    dyes += i32::from(sy != last_sy);
                    if dxes > 3 || dyes > 3 {
                        return true;
                    }
                    last_sx = sx;
                    last_sy = sy;
                }
                curr_pt = point;
                index += 1;
                if outer_loop != 0 {
                    break;
                }
            }
            // The source points at its stack firstPt for the final closing edge.
            index = 0;
        }
        false
    }
    // Source: src/core/SkPathPriv.cpp:500-513.
    fn close(&mut self) -> bool {
        self.add_pt(self.first_pt) && self.add_vec(self.first_vec)
    }
    fn is_finite(&self) -> bool {
        self.is_finite
    }
    fn reversals(&self) -> i32 {
        self.reversals
    }
    // Source: src/core/SkPathPriv.cpp:516-525; include/core/SkPoint.h:518-534.
    fn direction_change(&self, cur_vec: SkPathPoint) -> DirChange {
        let cross = self.last_vec.x * cur_vec.y - self.last_vec.y * cur_vec.x;
        if !cross.is_finite() {
            return DirChange::Unknown;
        }
        if cross == 0. {
            return if self.last_vec.x * cur_vec.x + self.last_vec.y * cur_vec.y < 0. {
                DirChange::Backwards
            } else {
                DirChange::Straight
            };
        }
        // SkScalarSignAsInt is +1 for positive cross and -1 for negative.
        if cross > 0. {
            DirChange::Right
        } else {
            DirChange::Left
        }
    }
    // Source: src/core/SkPathPriv.cpp:527-558.
    fn add_vec(&mut self, cur_vec: SkPathPoint) -> bool {
        let dir = self.direction_change(cur_vec);
        match dir {
            DirChange::Left | DirChange::Right => {
                if self.expected_dir == DirChange::Invalid {
                    self.expected_dir = dir;
                    self.first_direction = if dir == DirChange::Right {
                        FirstDirection::CW
                    } else {
                        FirstDirection::CCW
                    };
                } else if dir != self.expected_dir {
                    self.first_direction = FirstDirection::Unknown;
                    return false;
                }
                self.last_vec = cur_vec;
            }
            DirChange::Straight => {}
            DirChange::Backwards => {
                self.last_vec = cur_vec;
                self.reversals += 1;
                return self.reversals < 3;
            }
            DirChange::Unknown => {
                self.is_finite = false;
                return false;
            }
            DirChange::Invalid => unreachable!("Use of invalid direction change flag"),
        }
        true
    }
}

// Source: include/core/SkPathIter.h:20-77; src/core/SkPathIter.cpp:17-46.
// Fixed-size owned close storage maps the source iterator's two stack points;
// every other segment borrows the actual immutable PathData points.
enum IterPoints<'a> {
    Resident(&'a [SkPathPoint]),
    Close([SkPathPoint; 2]),
}
impl IterPoints<'_> {
    fn as_slice(&self) -> &[SkPathPoint] {
        match self {
            Self::Resident(points) => points,
            Self::Close(points) => points,
        }
    }
}
struct PathIterRec<'a> {
    points: IterPoints<'a>,
    conic_weight: f32,
    verb: SkPathVerb,
}
struct ValuePathIter<'a> {
    point_index: usize,
    verb_index: usize,
    conic_index: usize,
    points: &'a [SkPathPoint],
    verbs: &'a [SkPathVerb],
    conics: &'a [f32],
    close_point_storage: [SkPathPoint; 2],
}
impl<'a> ValuePathIter<'a> {
    fn new(points: &'a [SkPathPoint], mut verbs: &'a [SkPathVerb], conics: &'a [f32]) -> Self {
        if verbs.last() == Some(&M) {
            verbs = &verbs[..verbs.len() - 1];
        }
        Self {
            point_index: 0,
            verb_index: 0,
            conic_index: 0,
            points,
            verbs,
            conics,
            close_point_storage: [SkPathPoint::default(); 2],
        }
    }
    fn next(&mut self) -> Option<PathIterRec<'a>> {
        if self.verb_index >= self.verbs.len() {
            return None;
        }
        let mut w = -1.;
        let verb = self.verbs[self.verb_index];
        self.verb_index += 1;
        let n = match verb {
            M => {
                self.close_point_storage[1] = self.points[self.point_index];
                let p = self.point_index;
                self.point_index += 1;
                return Some(PathIterRec {
                    points: IterPoints::Resident(&self.points[p..p + 1]),
                    conic_weight: w,
                    verb,
                });
            }
            L => 1,
            Q => 2,
            K => {
                w = self.conics[self.conic_index];
                self.conic_index += 1;
                2
            }
            C => 3,
            Z => {
                debug_assert!(self.point_index > 0);
                self.close_point_storage[0] = self.points[self.point_index - 1];
                return Some(PathIterRec {
                    points: IterPoints::Close(self.close_point_storage),
                    conic_weight: w,
                    verb,
                });
            }
        };
        debug_assert!(self.point_index > 0);
        let start = self.point_index - 1;
        debug_assert!(n >= 1 && n <= 3);
        self.point_index += n;
        Some(PathIterRec {
            points: IterPoints::Resident(&self.points[start..start + n + 1]),
            conic_weight: w,
            verb,
        })
    }
}

// Source: src/core/SkPathPriv.cpp:572-582.
fn trim_trailing_moves<'a, 'b>(points: &mut &'a [SkPathPoint], verbs: &mut &'b [SkPathVerb]) {
    let mut count = verbs.len();
    while count > 0 && verbs[count - 1] == M {
        count -= 1;
    }
    let delta = verbs.len() - count;
    if delta != 0 {
        debug_assert!(points.len() >= delta);
        *points = &points[..points.len() - delta];
        *verbs = &verbs[..count];
    }
}
// Source: src/core/SkPathPriv.cpp:584-657.
fn compute_convexity(
    mut points: &[SkPathPoint],
    mut verbs: &[SkPathVerb],
    conics: &[f32],
) -> PathConvexity {
    debug_assert!(points.iter().all(|p| p.x.is_finite() && p.y.is_finite()));
    trim_trailing_moves(&mut points, &mut verbs);
    if verbs.is_empty() {
        return PathConvexity::ConvexDegenerate;
    }
    if Convexicator::is_concave_by_sign(points) {
        return PathConvexity::Concave;
    }
    let mut contour_count = 0;
    let mut needs_close = false;
    let mut state = Convexicator::new();
    let mut iter = ValuePathIter::new(points, verbs, conics);
    while let Some(rec) = iter.next() {
        let verb = rec.verb;
        let pts = rec.points.as_slice();
        if contour_count == 0 {
            if verb == M {
                state.set_move_pt(pts[0]);
            } else {
                contour_count += 1;
                needs_close = true;
            }
        }
        if contour_count == 1 {
            if verb == Z || verb == M {
                if !state.close() {
                    return PathConvexity::Concave;
                }
                needs_close = false;
                contour_count += 1;
            } else {
                // Source: src/core/SkPathPriv.h:264-281.
                let count = [1, 1, 2, 2, 3, 0][verb as usize];
                debug_assert!(count > 0);
                for i in 1..=count {
                    if !state.add_pt(pts[i]) {
                        return PathConvexity::Concave;
                    }
                }
            }
        } else if verb != M {
            return PathConvexity::Concave;
        }
    }
    if needs_close && !state.close() {
        return PathConvexity::Concave;
    }
    let first = state.get_first_direction();
    if first == FirstDirection::Unknown && state.reversals() >= 3 {
        return PathConvexity::Concave;
    }
    first.to_convexity()
}

// Source: src/core/SkPathPriv.h:289-307; SkPathPriv.cpp:80-143.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RectContour {
    pub rect: SkPathBounds,
    pub is_closed: bool,
    pub direction: SkPathDirection,
    pub point_count: usize,
    pub verb_count: usize,
}
fn trivial_rect(points: &[SkPathPoint], verbs: &[SkPathVerb]) -> Option<RectContour> {
    if points.len() != 4 || verbs != [M, L, L, L, Z] {
        return None;
    }
    let a = sub(points[1], points[0]);
    let b = sub(points[2], points[1]);
    let c = sub(points[3], points[2]);
    let d = sub(points[0], points[3]);
    let ortho =
        |a: SkPathPoint, b: SkPathPoint| ((a.x == 0.) ^ (b.x == 0.)) & ((a.y == 0.) ^ (b.y == 0.));
    if !(((a.x == 0.) ^ (a.y == 0.)) & ortho(a, b) & ortho(b, c) & ortho(c, d)) {
        return None;
    }
    let rect = sorted(SkPathBounds::MakeLTRB(
        points[0].x,
        points[0].y,
        points[2].x,
        points[2].y,
    ));
    let direction = if a.x * b.y - a.y * b.x > 0. {
        SkPathDirection::kCW
    } else {
        SkPathDirection::kCCW
    };
    Some(RectContour {
        rect,
        is_closed: true,
        direction,
        point_count: 4,
        verb_count: 5,
    })
}
// Source: src/core/SkPathPriv.cpp:145-299.
fn rect_contour(
    points: &[SkPathPoint],
    verbs: &[SkPathVerb],
    mask: u8,
    allow_partial: bool,
) -> Option<RectContour> {
    if mask != 1 || points.len() < 4 || verbs.len() < 4 {
        return None;
    }
    if let Some(rect) = trivial_rect(points, verbs) {
        return Some(rect);
    }
    let mut curr = 0;
    let mut corners = 0;
    let mut line_start = SkPathPoint::default();
    let (mut first, mut last): (Option<usize>, Option<usize>) = (None, None);
    let mut first_corner = SkPathPoint::default();
    let mut third_corner = SkPathPoint::default();
    let mut p = 0;
    let mut save = None;
    let mut directions = [-1i8; 5];
    let mut closed_or_moved = false;
    let mut auto_close = false;
    let mut insert_close = false;
    while curr < verbs.len() && (!allow_partial || !auto_close) {
        let verb = if insert_close { Z } else { verbs[curr] };
        match verb {
            Z | L => {
                if verb == Z {
                    save = Some(p);
                    auto_close = true;
                    insert_close = false;
                } else {
                    last = Some(p);
                }
                let line_end = if verb == Z {
                    points[first.unwrap()]
                } else {
                    let end = points[p];
                    p += 1;
                    end
                };
                let delta = sub(line_end, line_start);
                if delta.x != 0. && delta.y != 0. {
                    return None;
                }
                if !delta.x.is_finite() || !delta.y.is_finite() {
                    return None;
                }
                if line_start != line_end {
                    let dir =
                        i8::from(delta.x != 0.) | (i8::from(delta.x > 0. || delta.y > 0.) << 1);
                    if corners == 0 {
                        directions[0] = dir;
                        corners = 1;
                        closed_or_moved = false;
                        line_start = line_end;
                    } else {
                        if closed_or_moved {
                            return None;
                        }
                        if !(auto_close && dir == directions[0]) {
                            closed_or_moved = auto_close;
                            if directions[corners - 1] == dir {
                                if corners == 3 && verb == L {
                                    third_corner = line_end;
                                }
                                line_start = line_end;
                            } else {
                                directions[corners] = dir;
                                corners += 1;
                                match corners {
                                    2 => first_corner = line_start,
                                    3 => {
                                        if directions[0] ^ directions[2] != 2 {
                                            return None;
                                        }
                                        third_corner = line_end;
                                    }
                                    4 => {
                                        if directions[1] ^ directions[3] != 2 {
                                            return None;
                                        }
                                    }
                                    _ => return None,
                                }
                                line_start = line_end;
                            }
                        }
                    }
                }
            }
            Q | K | C => return None,
            M => {
                if allow_partial && !auto_close && directions[0] >= 0 {
                    insert_close = true;
                    curr = curr.wrapping_sub(1);
                    continue;
                }
                if corners == 0 {
                    first = Some(p);
                } else {
                    let close = sub(points[first.unwrap()], points[last.unwrap()]);
                    if close.x != 0. && close.y != 0. {
                        return None;
                    }
                }
                line_start = points[p];
                p += 1;
                closed_or_moved = true;
            }
        }
        curr += 1;
    }
    if corners < 3 || corners > 4 {
        return None;
    }
    let close = sub(points[first.unwrap()], points[last.unwrap()]);
    if close.x != 0. && close.y != 0. {
        return None;
    }
    Some(RectContour {
        rect: sorted(SkPathBounds::MakeLTRB(
            first_corner.x,
            first_corner.y,
            third_corner.x,
            third_corner.y,
        )),
        is_closed: auto_close,
        direction: if directions[0] == ((directions[1] + 1) & 3) {
            SkPathDirection::kCW
        } else {
            SkPathDirection::kCCW
        },
        point_count: save.unwrap_or(0),
        verb_count: curr,
    })
}

impl SkPath {
    /// Native SkPaint::getFillPath followed by SkPath::computeTightBounds,
    /// used by Blink platform/geometry/path.cc:126-165. Stroke approximation
    /// belongs to the existing Skia CPU stroker, not Paint or Compositor.
    pub fn stroke_bounds(&self, width: f32, resolution_scale: f32) -> Option<SkPathBounds> {
        let path = crate::compat::geometry::raster_path_from_native(self)?;
        let stroke = crate::raster::Stroke {
            width,
            ..Default::default()
        };
        let bounds = if width == 0. {
            path.compute_tight_bounds()?
        } else {
            path.stroke(&stroke, resolution_scale)?
                .compute_tight_bounds()?
        };
        Some(SkPathBounds {
            left: bounds.left(),
            top: bounds.top(),
            right: bounds.right(),
            bottom: bounds.bottom(),
        })
    }
}
